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
//! [`Quantities::EVERYTHING`], which meets every other part and is always
//! asked. `fuzz_games --audit` pays every order on a clone wherever this
//! says the parts commute (`GameState::audit_payment_orders`).
//!
//! **What reads a part** is three things, each below: an object another part
//! moves, as it last existed (Omnath sacrificed before or after the mana);
//! an ability watching a part's event that reads what another part changed
//! ("if you have 10 or less life"); and an ability of a moved object, which
//! sees a part's event only if that part came first.

use std::ops::{BitOr, BitOrAssign};

use crate::engine::actions::{ActionContext, GameActionKind, GameActionKindMask};
use crate::engine::audit::assert_shortcut_agrees;
use crate::engine::costs::{Payment, PaymentPlan};
use crate::engine::layers::cda::pt_amounts;
use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::types::{ContinuousEffect, EffectModification, EffectOrigin, PtValue};
use crate::engine::replacement::replacement_watchers;
use crate::events::unasked::{Elision, Unasked, UnaskedAnswer};
use crate::oracle::characteristics::{get_effective_abilities, get_effective_controller, needs_haste_to_tap, ObjectInformation};
use crate::state::game_state::GameState;
use crate::state::history_log::RowKind;
use crate::types::costs::Cost;
use crate::types::effects::{AmountExpr, Condition, ObjectFilter, ObjectSet, PlayerFact};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::triggers::{EventKind, EventKindMask};
use crate::types::zones::{Zone, ZoneSet};
use crate::ui::ask::ask_order_cost_payments;
use crate::ui::choice_types::{ChoiceKind, ChoiceOption, Rejection};
use crate::ui::decision::FirstLegalAnswer;

/// The most parts `audit_payment_orders` pays in every order of: 120 clones.
/// No printed cost comes near it.
const MOST_AUDITED_PARTS: usize = 5;

/// What the parts of a payment can change, and so all a reader is asked about
/// here. Mana and life are amounts, so two parts that each take some commute;
/// what they leave is a sum. A library is absent: no `Cost` arm touches one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Quantities(u16);

impl Quantities {
    pub(crate) const NONE: Quantities = Quantities(0);
    const POOL: Quantities = Quantities(1 << 0);
    const LIFE: Quantities = Quantities(1 << 1);
    const TAPPED: Quantities = Quantities(1 << 2);
    const COUNTERS: Quantities = Quantities(1 << 3);
    const BATTLEFIELD: Quantities = Quantities(1 << 4);
    const GRAVEYARD: Quantities = Quantities(1 << 5);
    const HAND: Quantities = Quantities(1 << 6);
    const EXILE: Quantities = Quantities(1 << 7);
    const HISTORY: Quantities = Quantities(1 << 8);
    /// What a reader that cannot say states: it meets every other part.
    const EVERYTHING: Quantities = Quantities(u16::MAX);

    fn meets(self, other: Quantities) -> bool {
        self.0 & other.0 != 0
    }

    /// What counting `zones` reads: their contents.
    fn of_zones(zones: ZoneSet) -> Quantities {
        [
            (Zone::Battlefield, Quantities::BATTLEFIELD),
            (Zone::Graveyard, Quantities::GRAVEYARD),
            (Zone::Hand, Quantities::HAND),
            (Zone::Exile, Quantities::EXILE),
        ]
        .into_iter()
        .filter(|(zone, _)| zones.contains(*zone))
        .fold(Quantities::NONE, |read, (_, quantity)| read | quantity)
    }
}

impl BitOr for Quantities {
    type Output = Quantities;
    fn bitor(self, other: Quantities) -> Quantities {
        Quantities(self.0 | other.0)
    }
}

impl BitOrAssign for Quantities {
    fn bitor_assign(&mut self, other: Quantities) {
        self.0 |= other.0;
    }
}

/// What paying `cost` reads, changes and adds to, whatever it is paid with.
struct CostReach {
    reads: Quantities,
    changes: Quantities,
    /// The ordered things it adds to (a graveyard, CR 404.2; the history
    /// log's rows of one kind, since "the first … each turn" reads which came
    /// first). Two parts adding to one have an order that shows.
    appends: Quantities,
    /// What its event is, to whatever watches for it. Paying mana is no
    /// event: the pool changes and nothing is announced.
    events: EventKindMask,
    actions: GameActionKindMask,
}

fn reach_of(cost: &Cost) -> CostReach {
    use Quantities as Q;
    let (no_events, no_actions) = (EventKindMask::EMPTY, GameActionKindMask::EMPTY);
    let moves = |from: Q, to: Q, reads: Q| CostReach {
        reads,
        changes: from | to | Q::HISTORY,
        appends: to,
        events: EventKindMask::of(EventKind::ZoneChange),
        actions: GameActionKindMask::of(GameActionKind::ZoneChange),
    };
    match cost {
        Cost::Mana(_) => CostReach { reads: Q::NONE, changes: Q::POOL, appends: Q::NONE, events: no_events, actions: no_actions },
        Cost::TapSelf => CostReach {
            reads: Q::NONE,
            changes: Q::TAPPED,
            appends: Q::NONE,
            events: EventKindMask::of(EventKind::Tapped),
            actions: GameActionKindMask::of(GameActionKind::Tap),
        },
        Cost::UntapSelf => CostReach {
            reads: Q::NONE,
            changes: Q::TAPPED,
            appends: Q::NONE,
            events: EventKindMask::of(EventKind::Untapped),
            actions: GameActionKindMask::of(GameActionKind::Untap),
        },
        Cost::PayLife(_) => CostReach {
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
        Cost::RemoveCounters(..) => CostReach {
            reads: Q::NONE,
            changes: Q::COUNTERS,
            appends: Q::COUNTERS,
            events: EventKindMask::of(EventKind::CountersRemovedFrom),
            actions: GameActionKindMask::of(GameActionKind::RemoveCounters),
        },
        Cost::AddCounters(..) => CostReach {
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
fn amount_reads(amount: &AmountExpr) -> Quantities {
    use Quantities as Q;
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
fn filter_reads(filter: &ObjectFilter) -> Quantities {
    use Quantities as Q;
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

fn player_fact_reads(fact: &PlayerFact) -> Quantities {
    use Quantities as Q;
    match fact {
        PlayerFact::ControlsPermanent(filter) => Q::BATTLEFIELD | filter_reads(filter),
        PlayerFact::LifeAtLeast(threshold) | PlayerFact::LifeAtMost(threshold) => Q::LIFE | amount_reads(threshold),
        PlayerFact::HandEmpty => Q::HAND,
        PlayerFact::CardInGraveyard(filter) => Q::GRAVEYARD | filter_reads(filter),
        PlayerFact::LibraryEmpty => Q::NONE,
    }
}

/// What holding `condition` reads.
fn condition_reads(condition: &Condition) -> Quantities {
    use Quantities as Q;
    match condition {
        Condition::Player { fact, .. } | Condition::DefendingPlayer { fact, .. } => player_fact_reads(fact),
        Condition::All(clauses) => clauses.iter().fold(Q::NONE, |read, clause| read | condition_reads(clause)),
        Condition::Not(clause) => condition_reads(clause),
        Condition::HostMatches(filter) => filter_reads(filter),
        Condition::SourceUntapped | Condition::SourceTapped => Q::TAPPED,
        Condition::CounterCount { .. } => Q::COUNTERS,
        Condition::ThisTurn(..) | Condition::LastTurn(..) | Condition::SinceYourLastTurn(..) | Condition::ThisGame(..) => Q::HISTORY,
        Condition::SpellWasKicked
        | Condition::ModeChosen(_)
        | Condition::CounterCountReached { .. }
        | Condition::ResolvedThisTurn(_)
        | Condition::CostAnswer(_) => Q::NONE,
    }
}

fn pt_reads(value: &PtValue) -> Quantities {
    match value {
        PtValue::Fixed(_) => Quantities::NONE,
        PtValue::Dynamic(amount) => amount_reads(amount),
    }
}

/// What applying `modification` reads: only a power or toughness can be an
/// amount.
fn modification_reads(modification: &EffectModification) -> Quantities {
    use EffectModification::*;
    match modification {
        SetPowerToughness { power, toughness } | ModifyPowerToughness { power, toughness } => pt_reads(power) | pt_reads(toughness),
        CopyFrom(_) | SetController(_) | AddType(_) | RemoveType(_) | SetTypes(_) | AddSubtype(_) | RemoveSubtype(_)
        | SetSubtypes(_) | AddSupertype(_) | RemoveSupertype(_) | SetSupertypes(_) | AddColor(_) | SetColors(_)
        | RemoveAllColors | GrantKeywordFlag(_) | RemoveKeywordFlag(_) | GrantAbility(_) | LoseAbility(_) | LoseAllAbilities
        | SwitchPowerToughness => Quantities::NONE,
    }
}

/// One part of a payment on this board: its cost's reach, with what the
/// objects it uses and what watches its event add.
struct Part {
    reads: Quantities,
    changes: Quantities,
    appends: Quantities,
    events: EventKindMask,
    actions: GameActionKindMask,
    /// What it takes out of its zone.
    moves: Vec<ObjectId>,
    /// What must still be where it is when it is paid.
    needs: Vec<ObjectId>,
    /// Whether a move ahead of it could leave it unpayable, as a creature
    /// hasty only by the permanent sacrificed beside its {T}.
    fails_after_a_move: bool,
}

impl Part {
    /// Whether `self` and `other` commute: decision 6's argument, as far as
    /// the two can state it. Two parts alike in every word and using nothing
    /// are one cost twice, either order the same game.
    fn commutes_with(&self, other: &Part, game: &GameState) -> bool {
        !self.reads.meets(other.changes)
            && !other.reads.meets(self.changes)
            && !self.appends.meets(other.appends)
            && !game.a_moved_object_watches(self, other)
            && !game.a_moved_object_watches(other, self)
    }
}

/// The orders that complete, as what must come before what: `precedes[a][b]`
/// is that part `a` is paid ahead of part `b` in every one of them.
pub(crate) struct MustPrecede {
    precedes: Vec<Vec<bool>>,
}

impl MustPrecede {
    /// Every edge `direct` implies, or `None` when two parts each need the
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
    fn broken_by(&self, order: &[usize]) -> Option<usize> {
        order.iter().enumerate().find_map(|(place, &later)| {
            order[..place].iter().any(|&earlier| self.precedes[later][earlier]).then_some(later)
        })
    }

    /// Every part in the order the engine would pay them, moved only as far
    /// as `asked`, the order chosen among the parts asked about, needs.
    fn merged(&self, asked: &[usize]) -> Vec<usize> {
        let n = self.precedes.len();
        let mut paid = vec![false; n];
        let mut next_asked = 0;
        let mut order = Vec::with_capacity(n);
        while order.len() < n {
            let ready = (0..n).find(|&part| {
                !paid[part]
                    && (0..n).all(|other| paid[other] || !self.precedes[other][part])
                    && asked.iter().position(|&p| p == part).is_none_or(|place| place == next_asked)
            });
            let Some(part) = ready else { break };
            paid[part] = true;
            next_asked += usize::from(asked.contains(&part));
            order.push(part);
        }
        debug_assert!(order.len() == n, "an order `broken_by` accepted cannot be merged: {asked:?}");
        order
    }
}

impl GameState {
    /// CR 601.2h's order for `plan`: asked of `player` where it can change
    /// the game, among the orders that complete, and otherwise the plan's
    /// own, recorded as not asked. Taken after the plan's other choices, since
    /// which permanent pays a sacrifice decides whether an order matters.
    ///
    /// Refused when no order completes: two parts have chosen one object, so
    /// whichever is paid first leaves the other unpayable, and "unpayable costs
    /// can't be paid". Refused here, before anything has moved, so that the
    /// failure comes ahead of every payment that cannot be taken back.
    pub(super) fn settle_payment_order(&self, plan: &mut PaymentPlan, player: PlayerId, source: ObjectId, ctx: &ActionContext) -> Result<(), String> {
        if plan.ordered.len() < 2 {
            return Ok(());
        }
        let parts: Vec<Part> =
            plan.ordered.iter().enumerate().map(|(place, cost)| self.part_of(cost, plan.sacrifices.get(&place).map(Vec::as_slice), source)).collect();
        let must = MustPrecede::closed(self.direct_edges(plan, &parts, player))
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
                let order = plan.ordered.iter().cloned().map(ChoiceOption::CostPart).collect();
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
        plan.pay_in(&must.merged(&chosen));
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
        let costs: Vec<Cost> = items.iter().map(|&part| plan.ordered[part].clone()).collect();
        let mut rejected = None;
        loop {
            let answer = ask_order_cost_payments(ctx.dp, self, player, source, &costs, rejected.take());
            let chosen: Vec<usize> = answer.into_iter().map(|at| items[at]).collect();
            match must.broken_by(&chosen) {
                None => return chosen,
                Some(unpaid) => rejected = Some(Rejection::OrderLeavesACostUnpaid(plan.ordered[unpaid].clone())),
            }
        }
    }

    /// `cost` as this board pays it, `chosen` being what pays a sacrifice.
    fn part_of(&self, cost: &Cost, chosen: Option<&[ObjectId]>, source: ObjectId) -> Part {
        let CostReach { mut reads, changes, appends, events, actions } = reach_of(cost);
        let chosen = chosen.unwrap_or_default();
        let (moves, needs, needs_haste): (&[ObjectId], &[ObjectId], bool) = match cost {
            Cost::TapSelf | Cost::UntapSelf => (&[], std::slice::from_ref(&source), needs_haste_to_tap(self, source)),
            Cost::SacrificeSelf => (std::slice::from_ref(&source), std::slice::from_ref(&source), false),
            Cost::Sacrifice(..) => (chosen, chosen, false),
            // A discard, an exile and a counter are refused by `can_pay_costs`
            // until `remaining-costs` plans what each uses.
            Cost::Mana(_)
            | Cost::PayLife(_)
            | Cost::Discard(..)
            | Cost::ExileFromGraveyard(..)
            | Cost::RemoveCounters(..)
            | Cost::AddCounters(..) => (&[], &[], false),
        };
        for &object in moves {
            reads |= self.characteristics_read(object);
        }
        reads |= self.what_its_watchers_read(events, actions);
        Part { reads, changes, appends, events, actions, moves: moves.to_vec(), needs: needs.to_vec(), fails_after_a_move: needs_haste }
    }

    /// What `object`'s characteristics read now, which is what it last
    /// existed as if a part moves it: its own CDAs, and each row that reaches
    /// it (`engine::layers`' walk, read rather than run).
    fn characteristics_read(&self, object: ObjectId) -> Quantities {
        let own = get_effective_abilities(self, object);
        let mut reads = own
            .iter()
            .filter(|ability| ability.is_characteristic_defining)
            .flat_map(|ability| pt_amounts(ability))
            .fold(Quantities::NONE, |read, amount| read | amount_reads(amount));
        for row in self.continuous_effects.iter() {
            let row_reads = self.row_reads(row);
            if row_reads != Quantities::NONE && self.row_reaches(row, object) {
                reads |= row_reads;
            }
        }
        reads
    }

    /// What a row reads wherever it applies: its amounts, which objects it
    /// picks out, and its "as long as" (CR 604.2).
    fn row_reads(&self, row: &ContinuousEffect) -> Quantities {
        let filter = match &row.affected_objects {
            ObjectSet::Filter { filter, .. } => filter_reads(filter),
            ObjectSet::SourceOnly | ObjectSet::Fixed(_) | ObjectSet::Host => Quantities::NONE,
        };
        let condition = match row.origin {
            EffectOrigin::Resolution => Quantities::NONE,
            EffectOrigin::StaticAbility { ability, clause } => get_effective_abilities(self, row.source)
                .iter()
                .find(|def| def.id == ability)
                .and_then(|def| clause.condition_in(&def.effect))
                .map_or(Quantities::NONE, condition_reads),
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
    fn what_its_watchers_read(&self, events: EventKindMask, actions: GameActionKindMask) -> Quantities {
        let mut reads = Quantities::NONE;
        if events == EventKindMask::EMPTY {
            return reads;
        }
        for watcher in self.objects_that_may_watch(events) {
            for ability in get_effective_abilities(self, watcher).iter() {
                if let Some(def) = ability.effect.as_trigger()
                    && def.record_kinds().intersects(events)
                    && let Some(condition) = &def.intervening_if
                {
                    reads |= condition_reads(condition);
                }
            }
        }
        for watcher in replacement_watchers(self, actions) {
            for ability in get_effective_abilities(self, watcher).iter() {
                for (condition, def) in ability.effect.replacements() {
                    if def.pattern.action_kinds().intersects(actions)
                        && let Some(condition) = condition
                    {
                        reads |= condition_reads(condition);
                    }
                }
            }
        }
        reads
    }

    /// Whether an object `mover` moves has an ability watching `watched`'s
    /// event: it sees the event only if `watched` is paid first.
    fn a_moved_object_watches(&self, mover: &Part, watched: &Part) -> bool {
        mover.moves.iter().any(|&object| {
            get_effective_abilities(self, object).iter().any(|ability| {
                ability.effect.as_trigger().is_some_and(|def| def.record_kinds().intersects(watched.events))
                    || ability.effect.replacements().any(|(_, def)| def.pattern.action_kinds().intersects(watched.actions))
            })
        })
    }

    /// What the plan forces, before closing it: a part ahead of whatever moves
    /// what it needs, and anything that could fail ahead of every move, so
    /// that nothing fails after a payment that cannot be taken back (CR 732.1,
    /// `engine::costs`).
    fn direct_edges(&self, plan: &PaymentPlan, parts: &[Part], player: PlayerId) -> Vec<Vec<bool>> {
        let n = parts.len();
        let mut precedes = vec![vec![false; n]; n];
        if parts.iter().all(|part| part.moves.is_empty()) {
            return precedes;
        }
        let (mana_pays, life_pays) = self.what_pays_in_any_order(plan, player);
        for a in 0..n {
            let could_fail = parts[a].fails_after_a_move
                || match &plan.ordered[a] {
                    Cost::Mana(_) => !mana_pays,
                    Cost::PayLife(_) => !life_pays,
                    Cost::TapSelf
                    | Cost::UntapSelf
                    | Cost::SacrificeSelf
                    | Cost::Sacrifice(..)
                    | Cost::Discard(..)
                    | Cost::ExileFromGraveyard(..)
                    | Cost::RemoveCounters(..)
                    | Cost::AddCounters(..) => false,
                };
            for b in (0..n).filter(|&b| b != a) {
                let needs_what_b_moves = parts[a].needs.iter().any(|object| parts[b].moves.contains(object));
                precedes[a][b] = needs_what_b_moves || (could_fail && !parts[b].moves.is_empty());
            }
        }
        precedes
    }

    /// Whether `plan`'s mana, and its life, are shown to be paid whenever they
    /// are paid: a split can spend what a pip needs, and two life payments
    /// that each pass CR 118.3's check can together be more than the player has.
    fn what_pays_in_any_order(&self, plan: &PaymentPlan, player: PlayerId) -> (bool, bool) {
        let mana_pays = plan.mana_can_be_paid_by(&self.players[player].mana_pool);
        let life: u64 = plan.ordered.iter().map(|cost| if let Cost::PayLife(n) = cost { *n } else { 0 }).sum();
        (mana_pays, self.players[player].life_total >= life as i64)
    }

    /// Decision 6's check, with the shortcut off: `plan` paid in every order
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
        if n > MOST_AUDITED_PARTS {
            return;
        }
        let keeps_chosen = |order: &[usize]| order.iter().filter(|part| chosen.contains(part)).eq(chosen.iter());
        let orders: Vec<Vec<usize>> = orders_of(n).into_iter().filter(|order| must.broken_by(order).is_none() && keeps_chosen(order)).collect();
        let Some((first, others)) = orders.split_first() else { return };
        let boards = self.unobserved(|| {
            let first = self.board_after_paying(plan, first, player, source);
            let others: Vec<_> = others.iter().map(|order| (order, self.board_after_paying(plan, order, player, source))).collect();
            (first, others)
        });
        for (order, board) in boards.1 {
            assert_shortcut_agrees("CR 601.2h's payment order", || format!("{:?} in the order {order:?}", plan.ordered), boards.0.clone(), board);
        }
    }

    /// The board a clone of this game is left with by paying `plan` in
    /// `order`, written out whole.
    fn board_after_paying(&self, plan: &PaymentPlan, order: &[usize], player: PlayerId, source: ObjectId) -> Vec<String> {
        let mut fork = self.clone();
        let mut plan = plan.clone();
        plan.pay_in(order);
        let answers = FirstLegalAnswer;
        match fork.pay_costs(&plan, player, source, &ActionContext::paying_cost(&answers, player, source)) {
            Ok(payment) => fork.board_written_out(&payment),
            Err(refused) => vec![refused],
        }
    }

    /// Everything a payment can change, a line a thing: zones and their
    /// order, the permanents and what each now is, the objects the payment
    /// used as they last existed, the triggers waiting, which their controller
    /// orders as they go on the stack, and the turn's history rows, each
    /// kind's in order and the kinds apart, since only a kind's order is read.
    fn board_written_out(&self, payment: &Payment) -> Vec<String> {
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
            let what = compute_characteristics(self, id).map(|chars| (chars.power, chars.toughness, chars.controller));
            lines.push(format!("{id}: tapped {tapped}, {counters} counters, {what:?}"));
        }
        for paid in &payment.paid_with {
            let last = paid.used.object().and_then(|existence| self.lki(existence)).map(|lki| lki.characteristics());
            lines.push(format!("{:?} last existed as {:?}", paid.used, last.map(|chars| (chars.power, chars.toughness))));
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

/// Every order of `n` things.
pub(crate) fn orders_of(n: usize) -> Vec<Vec<usize>> {
    let mut orders = vec![Vec::new()];
    for part in 0..n {
        orders = orders
            .into_iter()
            .flat_map(|order: Vec<usize>| {
                (0..=order.len()).map(move |at| {
                    let mut longer = order.clone();
                    longer.insert(at, part);
                    longer
                })
            })
            .collect();
    }
    orders
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
        answers.expect_order(
            ChoiceKind::OrderCostPayments { spell_or_ability_id: omnath },
            costs.iter().cloned().map(ChoiceOption::CostPart).collect(),
        );
        let plan = game.plan_payment(&costs, 0, omnath, &ActionContext::paying_cost(&answers, 0, omnath)).expect("payable");
        let nothing_forced = MustPrecede::closed(vec![vec![false; 2]; 2]).expect("no edges, no cycle");
        game.audit_payment_orders(&plan, &nothing_forced, &[], 0, omnath);
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
        assert_eq!(must.broken_by(&[1, 0]), Some(0), "part 0 must come ahead of part 1");
        assert_eq!(must.broken_by(&[0, 1]), None);
        assert_eq!(must.merged(&[2, 1]), vec![0, 2, 1], "part 1 waits for part 0 and for part 2");
    }
}
