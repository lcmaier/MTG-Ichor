//! CR 601.2h's "in any order", asked only where the orders' outcomes differ
//! and only among orders that complete
//! (`choices-architecture.md#decision-6--when-the-engine-asks-an-order-the-cr-hands-a-player`).
//!
//! **Two parts commute when neither reads what the other changes and they do
//! not both add to one ordered thing.** The engine skips the question only
//! where it can show that, since a skipped question that mattered takes a
//! choice the CR gives the player and nothing would show it. So every table
//! here is an exhaustive match: a new `Cost`, `AmountExpr`, `Condition`,
//! `ObjectFilter` or `EffectModification` arm does not compile until it says
//! what it reads or changes, and one that cannot say reads
//! [`PaymentChannels::EVERYTHING`], which intersects what every other part
//! changes, so it is always asked. `fuzz_games --audit` pays every order on a clone wherever this
//! says the parts commute (`GameState::audit_payment_orders`).
//!
//! **What reads a part** is three things, each below: an object another part
//! moves, as it last existed (Omnath sacrificed before or after the mana);
//! an ability watching a part's event that reads what another part changed
//! ("if you have 10 or less life"); and an ability of a moved object, which
//! sees a part's event only if that part came first.

use std::ops::{BitOr, BitOrAssign};

use crate::engine::actions::{ActionContext, GameActionKind, GameActionKindMask};
use crate::engine::audit::{assert_frames_agree, assert_shortcut_agrees, orders_to_audit, permanents_as_they_are, Frames};
use crate::engine::costs::{Payment, PaymentPlan};
use crate::engine::layers::cda::pt_amounts;
use crate::engine::layers::types::{ContinuousEffect, EffectModification, EffectOrigin, PtValue};
use crate::engine::replacement::replacement_watchers;
use crate::events::unasked::{Elision, Unasked, UnaskedAnswer};
use crate::oracle::characteristics::{get_effective_abilities, get_effective_controller, needs_haste_to_tap, ObjectInformation};
use crate::state::game_state::GameState;
use crate::state::history_log::RowKind;
use crate::types::costs::Cost;
use crate::types::effects::{AmountExpr, Condition, ObjectFilter, ObjectSet, PlayerFact};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::keywords::KeywordFlag;
use crate::types::triggers::{EventKind, EventKindMask};
use crate::types::zones::{Zone, ZoneSet};
use crate::ui::ask::ask_order_cost_payments;
use crate::ui::choice_types::{ChoiceKind, ChoiceOption, CostPart, Rejection};
use crate::ui::decision::FirstLegalAnswer;

/// The channels a payment's parts can change, and so all a reader is asked
/// about here, as `engine::layers::board`'s `Channels` are what a layer can
/// write. Mana and life are amounts, so two parts that each take some commute;
/// what they leave is a sum.
///
/// **A reader names what it reads, even where no `Cost` arm changes it yet**:
/// a library, and the resolutions counted this turn. A cost that comes to
/// change one, a mill, then meets its readers without anyone revisiting them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PaymentChannels(u16);

impl PaymentChannels {
    pub(crate) const NONE: PaymentChannels = PaymentChannels(0);
    const POOL: PaymentChannels = PaymentChannels(1 << 0);
    const LIFE: PaymentChannels = PaymentChannels(1 << 1);
    const TAPPED: PaymentChannels = PaymentChannels(1 << 2);
    const COUNTERS: PaymentChannels = PaymentChannels(1 << 3);
    const BATTLEFIELD: PaymentChannels = PaymentChannels(1 << 4);
    const GRAVEYARD: PaymentChannels = PaymentChannels(1 << 5);
    const HAND: PaymentChannels = PaymentChannels(1 << 6);
    const EXILE: PaymentChannels = PaymentChannels(1 << 7);
    const LIBRARY: PaymentChannels = PaymentChannels(1 << 8);
    const HISTORY: PaymentChannels = PaymentChannels(1 << 9);
    /// How many spells or abilities have resolved this turn (CR 603.7h's
    /// count), apart from `HISTORY`, which every part's event adds to.
    const RESOLUTIONS: PaymentChannels = PaymentChannels(1 << 10);
    /// A reader's own tapped status, "as long as this is untapped": no part
    /// changes it unless the reader is what a part taps (`as_read_by`).
    const ITS_OWN_TAPPED_STATUS: PaymentChannels = PaymentChannels(1 << 11);
    /// What a reader that cannot say states: it intersects what every other
    /// part changes.
    const EVERYTHING: PaymentChannels = PaymentChannels(u16::MAX);

    /// Whether the two share a channel.
    fn intersects(self, other: PaymentChannels) -> bool {
        self.0 & other.0 != 0
    }

    /// What counting `zones` reads: their contents.
    fn of_zones(zones: ZoneSet) -> PaymentChannels {
        [
            (Zone::Battlefield, PaymentChannels::BATTLEFIELD),
            (Zone::Graveyard, PaymentChannels::GRAVEYARD),
            (Zone::Hand, PaymentChannels::HAND),
            (Zone::Exile, PaymentChannels::EXILE),
            (Zone::Library, PaymentChannels::LIBRARY),
        ]
        .into_iter()
        .filter(|(zone, _)| zones.contains(*zone))
        .fold(PaymentChannels::NONE, |read, (_, quantity)| read | quantity)
    }
}

impl BitOr for PaymentChannels {
    type Output = PaymentChannels;
    fn bitor(self, other: PaymentChannels) -> PaymentChannels {
        PaymentChannels(self.0 | other.0)
    }
}

impl BitOrAssign for PaymentChannels {
    fn bitor_assign(&mut self, other: PaymentChannels) {
        self.0 |= other.0;
    }
}

/// What paying `cost` reads, changes and adds to, whatever it is paid with.
struct CostFootprint {
    reads: PaymentChannels,
    changes: PaymentChannels,
    /// The ordered things it adds to (a graveyard, CR 404.2; the history
    /// log's rows of one kind, since "the first … each turn" reads which came
    /// first). Two parts adding to one have an order that shows.
    appends: PaymentChannels,
    /// What its event is, to whatever watches for it. Paying mana announces
    /// nothing: CR 700.14's "expend" reads the whole cost once it is paid, so
    /// it puts no order among the parts.
    events: EventKindMask,
    actions: GameActionKindMask,
}

fn footprint_of(cost: &Cost) -> CostFootprint {
    use PaymentChannels as Q;
    let (no_events, no_actions) = (EventKindMask::EMPTY, GameActionKindMask::EMPTY);
    let moves = |from: Q, to: Q, reads: Q| CostFootprint {
        reads,
        changes: from | to | Q::HISTORY,
        appends: to,
        events: EventKindMask::of(EventKind::ZoneChange),
        actions: GameActionKindMask::of(GameActionKind::ZoneChange),
    };
    match cost {
        Cost::Mana(_) => CostFootprint { reads: Q::NONE, changes: Q::POOL, appends: Q::NONE, events: no_events, actions: no_actions },
        Cost::TapSelf => CostFootprint {
            reads: Q::NONE,
            changes: Q::TAPPED,
            appends: Q::NONE,
            events: EventKindMask::of(EventKind::Tapped),
            actions: GameActionKindMask::of(GameActionKind::Tap),
        },
        Cost::UntapSelf => CostFootprint {
            reads: Q::NONE,
            changes: Q::TAPPED,
            appends: Q::NONE,
            events: EventKindMask::of(EventKind::Untapped),
            actions: GameActionKindMask::of(GameActionKind::Untap),
        },
        Cost::PayLife(_) => CostFootprint {
            reads: Q::NONE,
            changes: Q::LIFE | Q::HISTORY,
            appends: Q::LIFE,
            events: EventKindMask::of(EventKind::LifeChanged),
            actions: GameActionKindMask::of(GameActionKind::LoseLife),
        },
        Cost::SacrificeSelf | Cost::Sacrifice(..) => moves(Q::BATTLEFIELD, Q::GRAVEYARD, Q::NONE),
        // "A part that moves into the zone another takes from": each of these
        // reads the zone its cards are chosen in.
        Cost::Discard(..) => moves(Q::HAND, Q::GRAVEYARD, Q::HAND),
        Cost::ExileFromGraveyard(..) => moves(Q::GRAVEYARD, Q::EXILE, Q::GRAVEYARD),
        Cost::RemoveCounters(..) => CostFootprint {
            reads: Q::NONE,
            changes: Q::COUNTERS,
            appends: Q::COUNTERS,
            events: EventKindMask::of(EventKind::CountersRemovedFrom),
            actions: GameActionKindMask::of(GameActionKind::RemoveCounters),
        },
        Cost::AddCounters(..) => CostFootprint {
            reads: Q::NONE,
            changes: Q::COUNTERS,
            appends: Q::COUNTERS,
            events: EventKindMask::of(EventKind::CountersPutOn),
            actions: GameActionKindMask::of(GameActionKind::AddCounters),
        },
    }
}

/// What `amount` reads. A resolution's leaves read nothing here: a payment
/// ends before anything resolves.
fn amount_reads(amount: &AmountExpr) -> PaymentChannels {
    use PaymentChannels as Q;
    match amount {
        AmountExpr::UnspentMana(_) => Q::POOL,
        AmountExpr::History(..) => Q::HISTORY,
        AmountExpr::SourceCounters(_) => Q::COUNTERS,
        AmountExpr::CountOf(set) | AmountExpr::CardTypesAmong(set) => match set {
            ObjectSet::Filter { filter, zones } => Q::of_zones(*zones) | filter_reads(filter),
            ObjectSet::SourceOnly | ObjectSet::Fixed(_) | ObjectSet::Host => Q::NONE,
        },
        AmountExpr::Plus(inner, _) | AmountExpr::Multiply(inner, _) => amount_reads(inner),
        // Another object's characteristics, which read what that object's do.
        AmountExpr::SourcePower | AmountExpr::TriggeringPower(_) | AmountExpr::TriggeringToughness(_) => Q::EVERYTHING,
        AmountExpr::Fixed(_)
        | AmountExpr::X
        | AmountExpr::AffectedManaValue
        | AmountExpr::StartingLifeTotal
        | AmountExpr::TriggeringAmount
        | AmountExpr::TargetPower
        | AmountExpr::TargetToughness
        | AmountExpr::DamageDealtThisWay
        | AmountExpr::ExcessDamageDealtThisWay
        | AmountExpr::DiscardedThisWay
        | AmountExpr::ReplacedAmount
        | AmountExpr::DamagePrevented
        | AmountExpr::ValueOf(..) => Q::NONE,
    }
}

/// What matching `filter` reads, among what a payment can change.
fn filter_reads(filter: &ObjectFilter) -> PaymentChannels {
    use PaymentChannels as Q;
    match filter {
        ObjectFilter::WithCounters { .. } => Q::COUNTERS,
        ObjectFilter::ThisTurn(_) | ObjectFilter::ThisCombat(_) => Q::HISTORY,
        // A power, which reads whatever that creature's does.
        ObjectFilter::PowerLE(_) => Q::EVERYTHING,
        ObjectFilter::And(a, b) | ObjectFilter::Or(a, b) => filter_reads(a) | filter_reads(b),
        ObjectFilter::Not(inner) => filter_reads(inner),
        ObjectFilter::All
        | ObjectFilter::ByType(_)
        | ObjectFilter::BySubtype(_)
        | ObjectFilter::BySupertype(_)
        | ObjectFilter::ByColor(_)
        | ObjectFilter::ByController(_)
        | ObjectFilter::ByOwner(_)
        | ObjectFilter::Token
        | ObjectFilter::NotSource
        | ObjectFilter::OtherThanInstance(_) => Q::NONE,
    }
}

fn player_fact_reads(fact: &PlayerFact) -> PaymentChannels {
    use PaymentChannels as Q;
    match fact {
        PlayerFact::ControlsPermanent(filter) => Q::BATTLEFIELD | filter_reads(filter),
        PlayerFact::LifeAtLeast(threshold) | PlayerFact::LifeAtMost(threshold) => Q::LIFE | amount_reads(threshold),
        PlayerFact::HandEmpty => Q::HAND,
        PlayerFact::CardInGraveyard(filter) => Q::GRAVEYARD | filter_reads(filter),
        PlayerFact::LibraryEmpty => Q::LIBRARY,
    }
}

/// What holding `condition` reads.
fn condition_reads(condition: &Condition) -> PaymentChannels {
    use PaymentChannels as Q;
    match condition {
        Condition::Player { fact, .. } | Condition::DefendingPlayer { fact, .. } => player_fact_reads(fact),
        Condition::All(clauses) => clauses.iter().fold(Q::NONE, |read, clause| read | condition_reads(clause)),
        Condition::Not(clause) => condition_reads(clause),
        Condition::HostMatches(filter) => filter_reads(filter),
        Condition::SourceUntapped | Condition::SourceTapped => Q::ITS_OWN_TAPPED_STATUS,
        Condition::CounterCount { .. } => Q::COUNTERS,
        Condition::ThisTurn(..) | Condition::LastTurn(..) | Condition::SinceYourLastTurn(..) | Condition::ThisGame(..) => Q::HISTORY,
        Condition::SpellWasKicked
        | Condition::ModeChosen(_)
        | Condition::CounterCountReached { .. }
        | Condition::CostAnswer(_) => Q::NONE,
        Condition::ResolvedThisTurn(_) => Q::RESOLUTIONS,
    }
}

fn pt_reads(value: &PtValue) -> PaymentChannels {
    match value {
        PtValue::Fixed(_) => PaymentChannels::NONE,
        PtValue::Dynamic(amount) => amount_reads(amount),
    }
}

/// What applying `modification` reads: only a power or toughness can be an
/// amount.
fn modification_reads(modification: &EffectModification) -> PaymentChannels {
    use EffectModification::*;
    match modification {
        SetPowerToughness { power, toughness } | ModifyPowerToughness { power, toughness } => pt_reads(power) | pt_reads(toughness),
        CopyFrom(_) | SetController(_) | AddType(_) | RemoveType(_) | SetTypes(_) | AddSubtype(_) | RemoveSubtype(_)
        | SetSubtypes(_) | AddSupertype(_) | RemoveSupertype(_) | SetSupertypes(_) | AddColor(_) | SetColors(_)
        | RemoveAllColors | GrantKeywordFlag(_) | RemoveKeywordFlag(_) | GrantAbility(_) | LoseAbility(_) | LoseAllAbilities
        | SwitchPowerToughness => PaymentChannels::NONE,
    }
}

/// What `reads`, read by `reader`, amount to in a payment for `source`: a
/// part taps only its source, so another object's own tapped status is not
/// among what the payment changes.
fn as_read_by(reads: PaymentChannels, reader: ObjectId, source: ObjectId) -> PaymentChannels {
    match reads.intersects(PaymentChannels::ITS_OWN_TAPPED_STATUS) && reader == source {
        true => reads | PaymentChannels::TAPPED,
        false => reads,
    }
}

/// One part of a payment on this board: what its cost touches, with what the
/// objects it uses and what watches its event add.
struct PaymentPart<'a> {
    cost: &'a Cost,
    reads: PaymentChannels,
    changes: PaymentChannels,
    appends: PaymentChannels,
    events: EventKindMask,
    actions: GameActionKindMask,
    /// What it takes out of its zone.
    moves: Vec<ObjectId>,
    /// What must still be where it is when it is paid.
    needs: Vec<ObjectId>,
}

impl PaymentPart<'_> {
    /// Whether `self` and `other` commute, as far as the two can state it.
    /// Two parts alike in every word and using nothing are one cost twice,
    /// either order the same game.
    fn commutes_with(&self, other: &PaymentPart, game: &GameState) -> bool {
        let one_cost_twice = self.cost == other.cost && self.moves.is_empty() && other.moves.is_empty();
        one_cost_twice
            || (!self.reads.intersects(other.changes)
                && !other.reads.intersects(self.changes)
                && !self.appends.intersects(other.appends)
                && !game.a_moved_object_watches(self, other)
                && !game.a_moved_object_watches(other, self))
    }
}

/// The orders that complete, as what must come before what: `precedes[a][b]`
/// is that part `a` is paid ahead of part `b` in every one of them.
pub(crate) struct MustPrecede {
    precedes: Vec<Vec<bool>>,
}

impl MustPrecede {
    /// Every edge `precedes` implies, or `None` when two parts each need the
    /// other first, so that no order completes.
    fn closed(mut precedes: Vec<Vec<bool>>) -> Option<MustPrecede> {
        let n = precedes.len();
        for via in 0..n {
            for a in 0..n {
                for b in 0..n {
                    precedes[a][b] |= precedes[a][via] && precedes[via][b];
                }
            }
        }
        (0..n).all(|a| !precedes[a][a]).then_some(MustPrecede { precedes })
    }

    fn settled(&self, a: usize, b: usize) -> bool {
        self.precedes[a][b] || self.precedes[b][a]
    }

    /// The part `order` leaves unpayable, if any, by paying what it needs
    /// gone first.
    fn part_left_unpaid_by(&self, order: &[usize]) -> Option<usize> {
        order.iter().enumerate().find_map(|(place, &later)| {
            order[..place].iter().any(|&earlier| self.precedes[later][earlier]).then_some(later)
        })
    }

    /// The order every part is paid in: the parts the player was asked about
    /// in `chosen`'s order, and each other part in the plan's own order
    /// (`payment_order_rank`'s), paid as soon as what must precede it has
    /// been. `chosen` is one `part_left_unpaid_by` accepted.
    fn whole_order_with(&self, chosen: &[usize]) -> Vec<usize> {
        let n = self.precedes.len();
        let mut paid = vec![false; n];
        let mut next_chosen = 0;
        let mut order = Vec::with_capacity(n);
        while order.len() < n {
            let ready = (0..n).find(|&part| {
                !paid[part]
                    && (0..n).all(|other| paid[other] || !self.precedes[other][part])
                    && chosen.iter().position(|&p| p == part).is_none_or(|place| place == next_chosen)
            });
            let Some(part) = ready else { break };
            paid[part] = true;
            next_chosen += usize::from(chosen.contains(&part));
            order.push(part);
        }
        debug_assert!(order.len() == n, "an order `part_left_unpaid_by` accepted has no whole order: {chosen:?}");
        order
    }
}

impl GameState {
    /// CR 601.2h's order for `plan`: asked of `player` where it can change
    /// the game, among the orders that complete, and otherwise the plan's
    /// own, recorded as not asked. Taken after the plan's other choices, since
    /// which permanent pays a sacrifice decides whether an order matters.
    ///
    /// Refused when no order completes: two parts have chosen one object, or
    /// life payments each passing CR 118.3's check are together more than the
    /// player has. "Unpayable costs can't be paid", and "partial payments are
    /// not allowed", so it is refused here, with nothing paid.
    pub(super) fn settle_payment_order(&self, plan: &mut PaymentPlan, player: PlayerId, source: ObjectId, ctx: &ActionContext) -> Result<(), String> {
        if plan.ordered.len() < 2 {
            return Ok(());
        }
        let life: u64 = plan.ordered.iter().map(|cost| if let Cost::PayLife(n) = cost { *n } else { 0 }).sum();
        if life > 0 && self.players[player].life_total < life as i64 {
            return Err(format!("No order pays {:?}: {life} life in all", plan.ordered));
        }
        let parts: Vec<PaymentPart> =
            plan.ordered.iter().enumerate().map(|(place, cost)| self.part_of(cost, plan.sacrifices.get(&place).map(Vec::as_slice), source)).collect();
        let must = MustPrecede::closed(self.direct_edges(&parts, source))
            .ok_or_else(|| format!("No order pays {:?}: two parts use one object", plan.ordered))?;

        let n = parts.len();
        let mut asked = vec![false; n];
        for a in 0..n {
            for b in a + 1..n {
                if !must.settled(a, b) && !parts[a].commutes_with(&parts[b], self) {
                    (asked[a], asked[b]) = (true, true);
                }
            }
        }
        let items: Vec<usize> = (0..n).filter(|&part| asked[part]).collect();
        if items.is_empty() {
            let why = match (0..n).all(|a| (a + 1..n).all(|b| must.settled(a, b))) {
                true => Unasked::OneLegalAnswer,
                false => Unasked::OneGame(Elision::PaymentOrder),
            };
            self.note_unasked(player, why, || {
                let order = (0..n).map(|part| ChoiceOption::CostPart(plan.part(part))).collect();
                (ChoiceKind::OrderCostPayments { spell_or_ability_id: source }, UnaskedAnswer::Order(order))
            });
            if self.audit_on() {
                self.audit_payment_orders(plan, &must, &[], player, source);
            }
            return Ok(());
        }

        let chosen = self.ask_payment_order(plan, &must, &items, player, source, ctx);
        if self.audit_on() {
            self.audit_payment_orders(plan, &must, &chosen, player, source);
        }
        plan.pay_in(&must.whole_order_with(&chosen));
        Ok(())
    }

    /// `items`' order, asked until the answer completes: an order that pays
    /// what another part needs first is refused, naming that part, since CR
    /// 601.2h's "unpayable costs can't be paid" holds of the order too.
    fn ask_payment_order(
        &self,
        plan: &PaymentPlan,
        must: &MustPrecede,
        items: &[usize],
        player: PlayerId,
        source: ObjectId,
        ctx: &ActionContext,
    ) -> Vec<usize> {
        let parts: Vec<CostPart> = items.iter().map(|&part| plan.part(part)).collect();
        let mut rejected = None;
        loop {
            let answer = ask_order_cost_payments(ctx.dp, self, player, source, &parts, rejected.take());
            let chosen: Vec<usize> = answer.into_iter().map(|at| items[at]).collect();
            match must.part_left_unpaid_by(&chosen) {
                None => return chosen,
                Some(unpaid) => rejected = Some(Rejection::OrderLeavesACostUnpaid(plan.ordered[unpaid].clone())),
            }
        }
    }

    /// `cost` as this board pays it: its footprint, the objects it takes away
    /// (`moves`) and the ones that must still be in place when it is paid
    /// (`needs`), and what those objects and the abilities watching its event
    /// read. `chosen` is what the plan picked to pay a sacrifice.
    fn part_of<'a>(&self, cost: &'a Cost, chosen: Option<&[ObjectId]>, source: ObjectId) -> PaymentPart<'a> {
        let CostFootprint { mut reads, changes, appends, events, actions } = footprint_of(cost);
        let chosen = chosen.unwrap_or_default();
        let this = std::slice::from_ref(&source);
        let (moves, needs): (&[ObjectId], &[ObjectId]) = match cost {
            // Amounts, so no object is used.
            Cost::Mana(_) | Cost::PayLife(_) => (&[], &[]),
            // The source stays, and must still be there.
            Cost::TapSelf | Cost::UntapSelf | Cost::RemoveCounters(..) | Cost::AddCounters(..) => (&[], this),
            Cost::SacrificeSelf => (this, this),
            Cost::Sacrifice(..) => (chosen, chosen),
            // Their cards are chosen as each is paid, and the plan does not
            // hold that choice yet: `can_pay_costs` refuses both
            // (`CannotPay::Unchecked`) until `remaining-costs` moves it into
            // the plan, so no plan reaches here with one.
            Cost::Discard(..) | Cost::ExileFromGraveyard(..) => (&[], &[]),
        };
        for &object in moves {
            reads |= self.characteristics_read(object, source);
        }
        reads |= self.what_its_watchers_read(events, actions, source);
        PaymentPart { cost, reads, changes, appends, events, actions, moves: moves.to_vec(), needs: needs.to_vec() }
    }

    /// What `object`'s characteristics read now, which is what it last
    /// existed as if a part moves it: its own CDAs, and each row that reaches
    /// it (`engine::layers`' walk, read rather than run).
    fn characteristics_read(&self, object: ObjectId, source: ObjectId) -> PaymentChannels {
        let own = get_effective_abilities(self, object);
        let mut reads = own
            .iter()
            .filter(|ability| ability.is_characteristic_defining)
            .flat_map(|ability| pt_amounts(ability))
            .fold(PaymentChannels::NONE, |read, amount| read | amount_reads(amount));
        for row in self.continuous_effects.iter() {
            let row_reads = as_read_by(self.row_reads(row), row.source, source);
            if row_reads != PaymentChannels::NONE && self.row_reaches(row, object) {
                reads |= row_reads;
            }
        }
        reads
    }

    /// What a row reads wherever it applies: its amounts, which objects it
    /// picks out, and its "as long as" (CR 604.2).
    fn row_reads(&self, row: &ContinuousEffect) -> PaymentChannels {
        let filter = match &row.affected_objects {
            ObjectSet::Filter { filter, .. } => filter_reads(filter),
            ObjectSet::SourceOnly | ObjectSet::Fixed(_) | ObjectSet::Host => PaymentChannels::NONE,
        };
        let condition = match row.origin {
            EffectOrigin::Resolution => PaymentChannels::NONE,
            EffectOrigin::StaticAbility { ability, clause } => get_effective_abilities(self, row.source)
                .iter()
                .find(|def| def.id == ability)
                .and_then(|def| clause.condition_in(&def.effect))
                .map_or(PaymentChannels::NONE, condition_reads),
        };
        modification_reads(&row.modification) | filter | condition
    }

    /// Whether `row` reaches `object` now. Its condition is not asked: a row
    /// that could reach it reads what it reads to find out.
    fn row_reaches(&self, row: &ContinuousEffect, object: ObjectId) -> bool {
        match &row.affected_objects {
            ObjectSet::Filter { filter, zones } => {
                let you = get_effective_controller(self, row.source).unwrap_or(row.controller);
                self.objects.get(&object).is_some_and(|found| zones.contains(found.zone))
                    && self.object_matches_filter_of_source(ObjectInformation::Current(object), filter, you, row.source)
            }
            named => named.named_objects(self, row.source).is_some_and(|mut ids| ids.any(|id| id == object)),
        }
    }

    /// What the abilities reacting to an event read as it happens: a
    /// triggered ability's intervening "if" (CR 603.4) and a replacement's
    /// "as long as". The event comes when its part is paid, so another part
    /// changing what they read decides whether they react.
    fn what_its_watchers_read(&self, events: EventKindMask, actions: GameActionKindMask, source: ObjectId) -> PaymentChannels {
        let mut reads = PaymentChannels::NONE;
        if events == EventKindMask::EMPTY {
            return reads;
        }
        for watcher in self.objects_that_may_watch(events) {
            for ability in get_effective_abilities(self, watcher).iter() {
                if let Some(def) = ability.effect.as_trigger()
                    && def.record_kinds().intersects(events)
                    && let Some(condition) = &def.intervening_if
                {
                    reads |= as_read_by(condition_reads(condition), watcher, source);
                }
            }
        }
        for watcher in replacement_watchers(self, actions) {
            for ability in get_effective_abilities(self, watcher).iter() {
                for (condition, def) in ability.effect.replacements() {
                    if def.pattern.action_kinds().intersects(actions)
                        && let Some(condition) = condition
                    {
                        reads |= as_read_by(condition_reads(condition), watcher, source);
                    }
                }
            }
        }
        reads
    }

    /// Whether an object `mover` moves has an ability watching `watched`'s
    /// event: it sees the event only if `watched` is paid first.
    fn a_moved_object_watches(&self, mover: &PaymentPart, watched: &PaymentPart) -> bool {
        mover.moves.iter().any(|&object| {
            get_effective_abilities(self, object).iter().any(|ability| {
                ability.effect.as_trigger().is_some_and(|def| def.record_kinds().intersects(watched.events))
                    || ability.effect.replacements().any(|(_, def)| def.pattern.action_kinds().intersects(watched.actions))
            })
        })
    }

    /// What the plan forces, before closing it: a part ahead of whatever moves
    /// what it needs, so that nothing fails after a payment that cannot be
    /// taken back (CR 732.1, `engine::costs`).
    fn direct_edges(&self, parts: &[PaymentPart], source: ObjectId) -> Vec<Vec<bool>> {
        let n = parts.len();
        let mut precedes = vec![vec![false; n]; n];
        for a in 0..n {
            for b in (0..n).filter(|&b| b != a) {
                let needs_what_b_moves = parts[a].needs.iter().any(|object| parts[b].moves.contains(object));
                precedes[a][b] = needs_what_b_moves || (parts[a].cost.is_tap_or_untap_symbol() && self.haste_leaves_with(source, &parts[b].moves));
            }
        }
        precedes
    }

    /// Whether `source` pays {T} or {Q} only by a haste that `moved` gives it
    /// (CR 302.6). Haste of its own stays, so that order completes.
    fn haste_leaves_with(&self, source: ObjectId, moved: &[ObjectId]) -> bool {
        !moved.is_empty()
            && needs_haste_to_tap(self, source)
            && self.continuous_effects.iter().any(|row| {
                moved.contains(&row.source)
                    && row.modification == EffectModification::GrantKeywordFlag(KeywordFlag::Haste)
                    && self.row_reaches(row, source)
            })
    }

    /// The shortcut's check, with the shortcut off: `plan` paid in every order
    /// that completes and keeps `chosen`, each on a clone of the board, and
    /// the boards compared whole. CR 601.2h's order is closed, since all a
    /// payment changes exists when it ends, so this is complete for every
    /// board it meets. A difference is a premise that does not hold here.
    pub(crate) fn audit_payment_orders(
        &self,
        plan: &PaymentPlan,
        must: &MustPrecede,
        chosen: &[usize],
        player: PlayerId,
        source: ObjectId,
    ) {
        let n = plan.ordered.len();
        let keeps_chosen = |order: &[usize]| order.iter().filter(|part| chosen.contains(part)).eq(chosen.iter());
        let orders: Vec<Vec<usize>> = orders_to_audit(n).into_iter().filter(|order| must.part_left_unpaid_by(order).is_none() && keeps_chosen(order)).collect();
        let Some((first, others)) = orders.split_first() else { return };
        let (first, others) = self.unobserved(|| {
            let first = self.board_after_paying(plan, first, player, source);
            let others: Vec<_> = others.iter().map(|order| (order, self.board_after_paying(plan, order, player, source))).collect();
            (first, others)
        });
        for (order, (written, frames)) in others {
            let question = || format!("{:?} in the order {order:?}", plan.ordered);
            assert_shortcut_agrees("CR 601.2h's payment order", question, first.0.clone(), written);
            assert_frames_agree("CR 601.2h's payment order", question, &first.1, &frames);
        }
    }

    /// The board a clone of this game is left with by paying `plan` in
    /// `order`: everything but what objects are, written out, and what they
    /// are, whole.
    fn board_after_paying(&self, plan: &PaymentPlan, order: &[usize], player: PlayerId, source: ObjectId) -> (Vec<String>, Frames) {
        let mut fork = self.clone();
        let mut plan = plan.clone();
        plan.pay_in(order);
        let answers = FirstLegalAnswer;
        match fork.pay_costs(&plan, player, source, &ActionContext::paying_cost(&answers, player, source)) {
            Ok(payment) => (fork.board_written_out(), fork.characteristics_after_payment(&payment)),
            Err(refused) => (vec![refused], Frames::new()),
        }
    }

    /// Each permanent's characteristics now, and those of each object
    /// `payment` used as it last existed: every characteristic, by id.
    fn characteristics_after_payment(&self, payment: &Payment) -> Frames {
        let mut frames = permanents_as_they_are(self);
        for existence in payment.paid_with.iter().filter_map(|paid| paid.used.object()) {
            if let Some(lki) = self.lki(existence) {
                frames.insert(existence.id, std::sync::Arc::clone(lki.frame()));
            }
        }
        frames
    }

    /// Everything a payment can change but what objects are, a line a thing:
    /// zones and their order, each permanent's status, the triggers waiting,
    /// which their controller orders as they go on the stack, and the turn's
    /// history rows, each kind's in order and the kinds apart, since only a
    /// kind's order is read.
    fn board_written_out(&self) -> Vec<String> {
        let mut lines = Vec::new();
        for (seat, player) in self.players.iter().enumerate() {
            let mut pool: Vec<(u8, u64)> = player.mana_pool.available().iter().map(|(kind, n)| (*kind as u8, *n)).filter(|(_, n)| *n > 0).collect();
            pool.sort_unstable();
            lines.push(format!(
                "P{seat}: life {}, pool {pool:?} and {:?}, hand {:?}, library {:?}, graveyard {:?}",
                player.life_total,
                player.mana_pool.special_atoms(),
                player.hand,
                player.library,
                player.graveyard
            ));
        }
        lines.push(format!("exile {:?}", self.zone_ids_ordered(Zone::Exile)));
        for id in self.battlefield_ids_ordered() {
            let tapped = self.battlefield.get(&id).is_some_and(|entry| entry.tapped);
            let counters = self.battlefield.get(&id).map_or(0, |entry| entry.counters_of(None));
            lines.push(format!("{id}: tapped {tapped}, {counters} counters"));
        }
        lines.extend(self.pending_triggers.iter().map(|waiting| format!("waiting: P{} {}", waiting.controller, waiting.source_card.name)));
        for kind in RowKind::ALL {
            let rows: Vec<String> = self
                .history
                .log
                .rows_of_turn(self.turn_number)
                .filter(|row| row.fact.kind() == kind)
                .map(|row| format!("{:?} {:?} {:?}", row.player, row.object, row.fact))
                .collect();
            lines.push(format!("{kind:?}: {rows:?}"));
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::authoring::creature;
    use crate::cards::fixtures::omnath_locus_of_mana_fixture;
    use crate::test_support::{put_on_battlefield, setup_two_player_game};
    use crate::types::mana::{ManaCost, ManaType};
    use crate::ui::decision::ScriptedDecisionProvider;

    /// The audit bites: handed parts the property would call a question, as
    /// though it had said they commute, it pays both orders and finds Omnath
    /// last existed as two different creatures. An audit that cannot disagree
    /// proves nothing by agreeing.
    #[test]
    #[should_panic(expected = "CR 601.2h's payment order")]
    fn the_audit_finds_a_difference_nothing_told_it_about() {
        let mut game = setup_two_player_game();
        let omnath = put_on_battlefield(&mut game, omnath_locus_of_mana_fixture(), 0);
        game.players[0].mana_pool.add(ManaType::Green, 6);
        let costs = [Cost::Mana(ManaCost::build(&[ManaType::Green, ManaType::Green], 2)), Cost::Sacrifice(creature(), 1)];
        let answers = ScriptedDecisionProvider::new();
        let part = |cost: &Cost, paid_with: Vec<ObjectId>| ChoiceOption::CostPart(CostPart { cost: cost.clone(), paid_with });
        answers.expect_order(ChoiceKind::OrderCostPayments { spell_or_ability_id: omnath }, vec![part(&costs[0], Vec::new()), part(&costs[1], vec![omnath])]);
        let plan = game.plan_payment(&costs, 0, omnath, &ActionContext::paying_cost(&answers, 0, omnath)).expect("payable");
        let nothing_forced = MustPrecede::closed(vec![vec![false; 2]; 2]).expect("no edges, no cycle");
        game.audit_payment_orders(&plan, &nothing_forced, &[], 0, omnath);
    }

    /// A reader names what it reads where no cost changes it yet, so a cost
    /// that comes to mill meets "your library has no cards in it" unasked.
    #[test]
    fn a_library_and_the_resolutions_are_channels_of_their_own() {
        assert_eq!(player_fact_reads(&PlayerFact::LibraryEmpty), PaymentChannels::LIBRARY);
        assert_eq!(condition_reads(&Condition::ResolvedThisTurn(2)), PaymentChannels::RESOLUTIONS);
        let life = footprint_of(&Cost::PayLife(1)).changes;
        assert!(life.intersects(PaymentChannels::HISTORY) && !life.intersects(PaymentChannels::RESOLUTIONS), "a payment resolves nothing");
    }

    #[test]
    fn two_parts_each_needing_the_other_first_have_no_order_that_completes() {
        assert!(MustPrecede::closed(vec![vec![false, true], vec![true, false]]).is_none());
    }

    /// An order is refused for its first broken edge, and a chosen order
    /// merges into the plan's without moving a part it says nothing about.
    #[test]
    fn a_chosen_order_merges_around_what_it_does_not_name() {
        let must = MustPrecede::closed(vec![vec![false, true, false], vec![false; 3], vec![false; 3]]).expect("no cycle");
        assert_eq!(must.part_left_unpaid_by(&[1, 0]), Some(0), "part 0 must come ahead of part 1");
        assert_eq!(must.part_left_unpaid_by(&[0, 1]), None);
        assert_eq!(must.whole_order_with(&[2, 1]), vec![0, 2, 1], "part 1 waits for part 0 and for part 2");
    }
}
