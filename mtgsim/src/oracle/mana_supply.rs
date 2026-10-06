//! What a player can pay with, and whether it pays a cost: the check behind
//! the priority question's offer (`mana-architecture.md` §3).
//!
//! **The inventory**, [`ManaSupply`], is taken once per player per priority
//! point, at the first card or ability that reaches the mana check. An entry
//! is one use of what a permanent's mana abilities share, its {T} or the
//! permanent itself, with what each way of using it makes once the board's
//! replacement effects and triggered mana abilities have acted on the
//! production (§3.4). The pool's mana is an entry too.
//!
//! **The check**, [`ManaSupply::covers`], is Gale's supply–demand condition
//! (Hall's, for b-matchings): a cost is payable exactly when each set of its
//! colored pip kinds asks for no more mana than the entries able to pay one
//! of them make, and the whole cost for no more than everything (§3.2).
//! Where a mana's type is not its own choice, several mana of one chosen
//! type or ways of different sizes, the check tries each choice (§3.3).
//!
//! **It answers whether a payment exists, never which.** Comparing payments
//! runs once per payment, on a person's seat (§3.12, item 162's rule).

use std::sync::Arc;

use crate::engine::layers::compute_characteristics;
use crate::engine::layers::condition::settled_holds;
use crate::engine::replacement::{applies_to_production, replacement_of};
use crate::engine::resolve::ResolutionContext;
use crate::engine::triggers::is_mana_ability;
use crate::engine::zone_function::functions_in;
use crate::objects::card_data::{AbilityDef, AbilityType};
use crate::oracle::characteristics::{controller_or_owner, controls, get_effective_abilities, has_summoning_sickness};
use crate::state::game_state::GameState;
use crate::types::costs::Cost;
use crate::types::effects::{AmountExpr, Effect, EffectRecipient, ObjectFilter, Primitive};
use crate::types::ids::{AbilityId, IdSet, ObjectId, PlayerId, Timestamp};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::types::replacement::{AmountRewrite, EventPattern, GameActionTemplate, ReplacementDef, Rewrite, TemplateAmount};
use crate::types::triggers::{TriggerEvent, TriggerSubject};
use crate::types::zones::Zone;

/// Mana by type, in [`ManaType`]'s order: W, U, B, R, G, C (CR 106.1b).
type Bag = [u64; 6];

const NO_MANA: Bag = [0; 6];

const ALL_TYPES: [ManaType; 6] =
    [ManaType::White, ManaType::Blue, ManaType::Black, ManaType::Red, ManaType::Green, ManaType::Colorless];

fn slot(mana_type: ManaType) -> usize {
    mana_type as usize
}

fn sum(a: &Bag, b: &Bag) -> Bag {
    std::array::from_fn(|t| a[t] + b[t])
}

/// Whether `a` holds at least `b`'s mana of every type.
fn holds(a: &Bag, b: &Bag) -> bool {
    (0..6).all(|t| a[t] >= b[t])
}

/// A set of the six types, a bit each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Types(u8);

impl Types {
    fn one(slot: usize) -> Types {
        Types(1 << slot)
    }

    fn of(made: &Bag) -> Types {
        Types((0..6).filter(|&t| made[t] > 0).fold(0, |set, t| set | 1 << t))
    }

    fn has(self, slot: usize) -> bool {
        self.0 & (1 << slot) != 0
    }

    fn meets(self, other: Types) -> bool {
        self.0 & other.0 != 0
    }

    fn union(self, other: Types) -> Types {
        Types(self.0 | other.0)
    }

    /// Every nonempty subset.
    fn subsets(self) -> impl Iterator<Item = Types> {
        let all = self.0;
        let mut next = all;
        std::iter::from_fn(move || {
            if next == 0 {
                return None;
            }
            let current = next;
            next = (next - 1) & all;
            Some(Types(current))
        })
    }
}

/// One way of using an entry, and what it makes.
#[derive(Debug, Clone, PartialEq)]
struct Way {
    makes: Bag,
    /// It taps the permanent, so a cost that taps the same permanent rules it
    /// out.
    taps: bool,
    /// It sacrifices the permanent, so a cost that sacrifices the same
    /// permanent rules it out.
    sacrifices: bool,
}

impl Way {
    /// Never worse than `other`: at least its mana of every type, for no more
    /// of the permanent. The second half keeps a way a cost's exclusion could
    /// leave alone (§3.3).
    fn dominates(&self, other: &Way) -> bool {
        holds(&self.makes, &other.makes)
            && (!self.taps || other.taps)
            && (!self.sacrifices || other.sacrifices)
    }
}

/// `ways` less every way another dominates, in their order.
fn undominated(ways: Vec<Way>) -> Vec<Way> {
    let mut kept: Vec<Way> = Vec::new();
    for way in ways {
        if kept.iter().any(|k| k.dominates(&way)) {
            continue;
        }
        kept.retain(|k| !way.dominates(k));
        kept.push(way);
    }
    kept
}

/// One use of what a permanent's mana abilities share, or the pool's mana.
#[derive(Debug, Clone, PartialEq)]
struct Entry {
    /// `None` for the pool.
    permanent: Option<ObjectId>,
    ways: Vec<Way>,
}

/// "Sacrifice a [filter]: Add mana" (Krark-Clan Ironworks): used once for
/// each permanent it can sacrifice (CR 701.21a), itself last.
#[derive(Debug, Clone, PartialEq)]
struct Outlet {
    /// Outlets of one ability share their fodder, so they are counted once.
    definition: AbilityId,
    needs: u32,
    /// The player's permanents its filter matches, itself among them.
    fodder: Vec<ObjectId>,
    /// What one activation can make.
    ways: Vec<Bag>,
}

/// "{cost}, {T}: Double the amount of each type of unspent mana you have"
/// (Doubling Cube).
#[derive(Debug, Clone, PartialEq)]
struct Doubler {
    permanent: ObjectId,
    input: Demand,
    /// What one mana left in the pool becomes: itself and what the doubling
    /// adds for it, 2, or 3 under Mana Reflection.
    factor: u64,
}

/// What `player` can pay mana with at one moment (§3.1).
#[derive(Debug, Clone)]
pub struct ManaSupply {
    player: PlayerId,
    entries: Vec<Entry>,
    outlets: Vec<Outlet>,
    doublers: Vec<Doubler>,
}

/// What a mana cost is paid for besides its mana: the payment's other costs.
/// CR 601.2h pays them after the window, so what they take is not there to
/// make mana with: a cost's own {T} rules out its source's tap, its own
/// sacrifice the source as anything's fodder (§3.3).
#[derive(Debug, Clone, Copy)]
pub struct Payment<'a> {
    /// The permanent whose ability is activated; `None` for a spell.
    pub source: Option<ObjectId>,
    pub other_costs: &'a [Cost],
}

impl ManaSupply {
    /// What `player` can pay with now: the pool, and every mana ability of a
    /// permanent they control whose other costs can be paid, in timestamp
    /// order.
    pub fn take(game: &GameState, player: PlayerId) -> ManaSupply {
        let watchers = production_watchers(game);
        let mine = permanents_of(game, player);
        let mut supply = ManaSupply { player, entries: Vec::new(), outlets: Vec::new(), doublers: Vec::new() };
        if let Some(state) = game.players.get(player) {
            let mut pool = NO_MANA;
            for (&mana_type, &n) in state.mana_pool.available() {
                pool[slot(mana_type)] += n;
            }
            if pool.iter().any(|&n| n > 0) {
                supply.entries.push(Entry { permanent: None, ways: vec![Way { makes: pool, taps: false, sacrifices: false }] });
            }
        }
        for &id in &mine {
            let mut ways: Vec<Way> = Vec::new();
            // Sacrificing it for mana leaves its {T} free to use first.
            let mut alone: Vec<Way> = Vec::new();
            for ability in get_effective_abilities(game, id).iter().filter(|a| a.ability_type == AbilityType::Mana) {
                match activation_of(ability) {
                    Activation::Once { taps, sacrifices } => {
                        if game.can_pay_costs(&ability.costs, player, id).is_err() {
                            continue;
                        }
                        let Some(base) = production_of(game, &ability.effect, id, player) else { continue };
                        for makes in made_by(game, &watchers, player, id, taps, base) {
                            let way = Way { makes, taps, sacrifices };
                            if sacrifices && !taps { alone.push(way) } else { ways.push(way) }
                        }
                    }
                    Activation::PerSacrifice { filter, needs } => {
                        let fodder: Vec<ObjectId> = mine
                            .iter()
                            .copied()
                            .filter(|&f| game.object_matches_filter(f, filter, player).unwrap_or(false))
                            .collect();
                        if fodder.len() < needs as usize {
                            continue;
                        }
                        let Some(base) = production_of(game, &ability.effect, id, player) else { continue };
                        let ways = made_by(game, &watchers, player, id, false, base);
                        supply.outlets.push(Outlet { definition: ability.id.definition(), needs, fodder, ways });
                    }
                    Activation::Doubling { input } => {
                        let Some(input) = Demand::of(input) else { continue };
                        if game.can_pay_costs(&[Cost::TapSelf], player, id).is_err() {
                            continue;
                        }
                        // One mana left in the pool, doubled: the doubling is a tap
                        // for mana (CR 106.12), so a multiplier on taps scales it.
                        let mut unit = NO_MANA;
                        unit[slot(ManaType::Colorless)] = 1;
                        let made = rewritten(game, &watchers, player, id, true, unit);
                        // A retype would make the doubled mana all one type, which no
                        // printed doubler meets; the check does not read it.
                        if made.iter().any(|m| Types::of(m) != Types::one(slot(ManaType::Colorless))) {
                            continue;
                        }
                        let added = made.iter().map(|m| m[slot(ManaType::Colorless)]).max().unwrap_or(0);
                        if added > 0 {
                            supply.doublers.push(Doubler { permanent: id, input, factor: 1 + added });
                        }
                    }
                    Activation::Unread => {}
                }
            }
            let tapped: Vec<Way> = ways.iter().filter(|w| !w.sacrifices).cloned().collect();
            for t in &tapped {
                for s in &alone {
                    ways.push(Way { makes: sum(&t.makes, &s.makes), taps: true, sacrifices: true });
                }
            }
            ways.extend(alone);
            let ways = undominated(ways);
            if !ways.is_empty() {
                supply.entries.push(Entry { permanent: Some(id), ways });
            }
        }
        supply
    }

    /// Whether this inventory pays `cost` for `payment`. A symbol no payment
    /// path pays yet is refused, as `ManaPool::pay` refuses it (§3.2): the
    /// offer must agree with the payment (`cost-architecture.md` §3.6).
    pub fn covers(&self, game: &GameState, cost: &ManaCost, payment: &Payment<'_>) -> bool {
        let Some(demand) = Demand::of(cost) else { return false };
        demand.total() == 0 || self.pays(&demand, &Taken::of(game, self.player, payment, !self.outlets.is_empty()))
    }

    fn pays(&self, demand: &Demand, taken: &Taken) -> bool {
        let doublers: Vec<&Doubler> = self.doublers.iter().filter(|d| taken.tapped != Some(d.permanent)).collect();
        // Every other entry is used before a doubler, since mana made after
        // it is not doubled; with no doubler first, the common board.
        (0..=doublers.len()).any(|used| {
            let used = &doublers[..used];
            let asked = used.iter().rev().fold(*demand, |asked, d| asked.before_doubling(d.factor, &d.input));
            self.pieces(taken, used).pay(&asked)
        })
    }

    /// The supply this payment leaves, split by how each mana's type is
    /// chosen.
    fn pieces(&self, taken: &Taken, doubling: &[&Doubler]) -> Pieces {
        let mut pieces = Pieces::default();
        for entry in &self.entries {
            let doubles = doubling.iter().any(|d| Some(d.permanent) == entry.permanent);
            let ways: Vec<Bag> = entry
                .ways
                .iter()
                .filter(|w| taken.allows(entry.permanent, w) && !(w.taps && doubles))
                .map(|w| w.makes)
                .collect();
            pieces.add(&ways, 1);
        }
        let mut counted: Vec<AbilityId> = Vec::new();
        for outlet in &self.outlets {
            if counted.contains(&outlet.definition) {
                continue;
            }
            counted.push(outlet.definition);
            let activations = taken.fodder_left(&outlet.fodder) / u64::from(outlet.needs.max(1));
            pieces.add(&outlet.ways, activations);
        }
        pieces
    }
}

/// A permanent's mana ability that can be activated now, and one type it
/// makes: the random agent's view of the window's options, to tap first
/// what makes a pip still owed (`ui::random`).
#[derive(Debug, Clone)]
pub struct ManaSource {
    pub permanent_id: ObjectId,
    pub ability_id: AbilityId,
    pub produces: ManaType,
}

/// Every mana ability of `player_id`'s permanents whose costs can be paid
/// now, a row for each type it makes on this board, in timestamp order: what
/// a Forest under Deep Water makes is blue, and a land Wild Growth enchants
/// makes green besides its own, as the check counts them.
pub fn available_mana_sources(game: &GameState, player_id: PlayerId) -> Vec<ManaSource> {
    let watchers = production_watchers(game);
    let mut sources = Vec::new();
    for id in permanents_of(game, player_id) {
        for ability in get_effective_abilities(game, id).iter() {
            if ability.ability_type != AbilityType::Mana || game.can_pay_costs(&ability.costs, player_id, id).is_err() {
                continue;
            }
            for produces in types_made(game, &watchers, ability, id, player_id) {
                sources.push(ManaSource { permanent_id: id, ability_id: ability.id, produces });
            }
        }
    }
    sources
}

/// The types `ability` makes now, after what the board does to its
/// production; for a shape the inventory does not read, the fixed amounts it
/// prints.
fn types_made(
    game: &GameState,
    watchers: &ProductionWatchers,
    ability: &AbilityDef,
    permanent: ObjectId,
    player: PlayerId,
) -> Vec<ManaType> {
    let taps = match activation_of(ability) {
        Activation::Once { taps, .. } => Some(taps),
        Activation::PerSacrifice { .. } => Some(false),
        Activation::Doubling { .. } | Activation::Unread => None,
    };
    let made = match (taps, production_of(game, &ability.effect, permanent, player)) {
        (Some(taps), Some(base)) => made_by(game, watchers, player, permanent, taps, base),
        _ => {
            let Effect::Atom(Primitive::ProduceMana(output), _) = &ability.effect else { return Vec::new() };
            let fixed = output.mana.iter().filter(|(_, amount)| matches!(amount, AmountExpr::Fixed(n) if *n > 0));
            return fixed.map(|(mana_type, _)| *mana_type).collect();
        }
    };
    let types = made.iter().fold(Types::default(), |set, m| set.union(Types::of(m)));
    ALL_TYPES.into_iter().filter(|&t| types.has(slot(t))).collect()
}

/// The mana abilities CR 601.2g's window offers a player, read once a window
/// (§3.8). Between its prompts only whether each one's costs can be paid
/// changes, until the layer epoch moves (a sacrifice), when it is read again.
pub struct WindowOffer {
    player: PlayerId,
    epoch: u64,
    abilities: Vec<OfferedAbility>,
}

/// A mana ability the window may offer: its permanent and the first instance
/// of its definition there, since two grants of one ability are one choice.
struct OfferedAbility {
    permanent: ObjectId,
    ability: AbilityId,
    paid: Payable,
}

/// How the window re-asks whether an ability's costs can be paid.
enum Payable {
    /// {T} alone: while the permanent is untapped, since whether it is
    /// summoning-sick (CR 302.6) moves only with the epoch.
    WhileUntapped { sick: bool },
    /// Anything else, asked whole each time.
    Costs(Vec<Cost>),
}

impl WindowOffer {
    pub fn take(game: &GameState, player: PlayerId) -> WindowOffer {
        let mut seen: IdSet<(ObjectId, AbilityId)> = IdSet::default();
        let mut abilities = Vec::new();
        for permanent in permanents_of(game, player) {
            for ability in get_effective_abilities(game, permanent).iter() {
                if ability.ability_type != AbilityType::Mana || !offered_in_the_window(game, ability, permanent, player) {
                    continue;
                }
                if !seen.insert((permanent, ability.id.definition())) {
                    continue;
                }
                let paid = match ability.costs.as_slice() {
                    [Cost::TapSelf] => Payable::WhileUntapped { sick: has_summoning_sickness(game, permanent) },
                    costs => Payable::Costs(costs.to_vec()),
                };
                abilities.push(OfferedAbility { permanent, ability: ability.id, paid });
            }
        }
        WindowOffer { player, epoch: game.layer_epoch(), abilities }
    }

    /// Whether the board this offer was read from is still the board.
    pub fn is_current(&self, game: &GameState) -> bool {
        game.layer_epoch() == self.epoch
    }

    /// The abilities offered now, in timestamp order and then the order of
    /// each permanent's abilities: the order the agent's answer is a position
    /// in (`CLAUDE.md`'s determinism rule).
    pub fn options(&self, game: &GameState) -> Vec<(ObjectId, AbilityId)> {
        self.abilities
            .iter()
            .filter(|offered| match &offered.paid {
                Payable::WhileUntapped { sick } => {
                    !sick && game.battlefield.get(&offered.permanent).is_some_and(|entry| !entry.tapped)
                }
                Payable::Costs(costs) => game.can_pay_costs(costs, self.player, offered.permanent).is_ok(),
            })
            .map(|offered| (offered.permanent, offered.ability))
            .collect()
    }
}

/// An ability the window offers: what it has always offered, one that makes
/// a fixed amount of mana, and whatever else the inventory counts, so that a
/// payment the check found is one the window can make (`cost-architecture.md`
/// §3.6). Doubling Cube is the one registered card that adds.
fn offered_in_the_window(game: &GameState, ability: &AbilityDef, permanent: ObjectId, player: PlayerId) -> bool {
    let makes_fixed_mana = matches!(&ability.effect, Effect::Atom(Primitive::ProduceMana(output), _)
        if output.mana.iter().any(|(_, amount)| matches!(amount, AmountExpr::Fixed(n) if *n > 0)));
    makes_fixed_mana
        || match activation_of(ability) {
            Activation::Doubling { .. } => true,
            Activation::Once { .. } | Activation::PerSacrifice { .. } => {
                production_of(game, &ability.effect, permanent, player).is_some()
            }
            Activation::Unread => false,
        }
}

/// What a payment's other costs take from the board.
#[derive(Default)]
struct Taken {
    tapped: Option<ObjectId>,
    sacrificed: Option<ObjectId>,
    /// Each other sacrifice: the permanents it may take, and how many.
    sacrifices: Vec<(Vec<ObjectId>, u32)>,
}

impl Taken {
    /// `fodder_read` is whether anything reads a sacrifice's candidates:
    /// without an outlet nothing does, and they are not looked up.
    fn of(game: &GameState, player: PlayerId, payment: &Payment<'_>, fodder_read: bool) -> Taken {
        let mut taken = Taken { tapped: None, sacrificed: None, sacrifices: Vec::new() };
        for cost in payment.other_costs {
            match cost {
                Cost::TapSelf => taken.tapped = payment.source,
                Cost::SacrificeSelf => taken.sacrificed = payment.source,
                Cost::Sacrifice(filter, needs) if fodder_read => {
                    let candidates = permanents_of(game, player)
                        .into_iter()
                        .filter(|&id| game.object_matches_filter(id, filter, player).unwrap_or(false))
                        .collect();
                    taken.sacrifices.push((candidates, *needs));
                }
                // The mana is the cost being checked, and the rest take nothing
                // a mana ability the inventory reads makes mana from.
                Cost::Sacrifice(..)
                | Cost::Mana(_)
                | Cost::UntapSelf
                | Cost::PayLife(_)
                | Cost::Discard(..)
                | Cost::ExileFromGraveyard(..)
                | Cost::RemoveCounters(..)
                | Cost::AddCounters(..) => {}
            }
        }
        taken
    }

    fn allows(&self, permanent: Option<ObjectId>, way: &Way) -> bool {
        let Some(permanent) = permanent else { return true };
        !(way.taps && self.tapped == Some(permanent)) && !(way.sacrifices && self.sacrificed == Some(permanent))
    }

    /// How much of `fodder` this payment's own sacrifices leave: each takes
    /// what it can from outside the fodder first.
    fn fodder_left(&self, fodder: &[ObjectId]) -> u64 {
        let mut left = fodder.len() as u64;
        if self.sacrificed.is_some_and(|s| fodder.contains(&s)) {
            left -= 1;
        }
        for (candidates, needs) in &self.sacrifices {
            let outside = candidates.iter().filter(|c| !fodder.contains(c) && Some(**c) != self.sacrificed).count() as u64;
            left = left.saturating_sub(u64::from(*needs).saturating_sub(outside));
        }
        left
    }
}

/// How a mana ability's cost is paid, which decides how often the inventory
/// can use it (§3.3).
enum Activation<'a> {
    /// {T}, a sacrifice of itself, or both: once.
    Once { taps: bool, sacrifices: bool },
    /// "Sacrifice a [filter]": once for each permanent it can sacrifice.
    PerSacrifice { filter: &'a ObjectFilter, needs: u32 },
    Doubling { input: &'a ManaCost },
    /// A cost this phase does not read: a converter fed by other mana
    /// (MA-3), life, counters, an untap, or nothing at all.
    Unread,
}

fn activation_of(ability: &AbilityDef) -> Activation<'_> {
    let (mut taps, mut sacrifices, mut sacrifice, mut mana, mut other) = (false, false, None, None, false);
    for cost in &ability.costs {
        match cost {
            Cost::TapSelf => taps = true,
            Cost::SacrificeSelf => sacrifices = true,
            Cost::Sacrifice(filter, needs) if sacrifice.is_none() => sacrifice = Some((filter, *needs)),
            Cost::Mana(input) if mana.is_none() => mana = Some(input),
            Cost::Sacrifice(..)
            | Cost::Mana(_)
            | Cost::UntapSelf
            | Cost::PayLife(_)
            | Cost::Discard(..)
            | Cost::ExileFromGraveyard(..)
            | Cost::RemoveCounters(..)
            | Cost::AddCounters(..) => other = true,
        }
    }
    match (sacrifice, mana, other) {
        (None, None, false) if taps || sacrifices => Activation::Once { taps, sacrifices },
        (Some((filter, needs)), None, false) if !taps && !sacrifices => Activation::PerSacrifice { filter, needs },
        (None, Some(input), false) if taps && !sacrifices && doubles_the_pool(&ability.effect) => {
            Activation::Doubling { input }
        }
        _ => Activation::Unread,
    }
}

/// Doubling Cube's effect: each type's unspent mana, added again.
fn doubles_the_pool(effect: &Effect) -> bool {
    let Effect::Atom(Primitive::ProduceMana(output), _) = effect else { return false };
    output.special.is_empty()
        && output.mana.len() == ALL_TYPES.len()
        && ALL_TYPES
            .iter()
            .all(|t| output.mana.iter().any(|(made, amount)| made == t && *amount == AmountExpr::UnspentMana(*t)))
}

/// What a mana ability's effect adds for `player`, before any replacement
/// effect: `None` when it adds nothing a payment can spend now.
fn production_of(game: &GameState, effect: &Effect, source: ObjectId, player: PlayerId) -> Option<Bag> {
    let mut resolution = ResolutionContext::untargeted(source, player);
    resolution.ability_source = game.object_ref(source);
    let mut made = NO_MANA;
    produce(game, effect, &resolution, &mut made)?;
    made.iter().any(|&n| n > 0).then_some(made)
}

fn produce(game: &GameState, effect: &Effect, resolution: &ResolutionContext, made: &mut Bag) -> Option<()> {
    match effect {
        Effect::Atom(Primitive::ProduceMana(output), _) => {
            for (mana_type, amount) in &output.mana {
                // The pool at resolution, after the window's other activations:
                // not a number the inventory can take now.
                if matches!(amount, AmountExpr::UnspentMana(_)) {
                    return None;
                }
                made[slot(*mana_type)] += game.evaluate_amount(amount, resolution).ok()?;
            }
            // `output.special` is restricted mana, which no payment spends until
            // item 33's pass, so it is not counted (§3.6).
            Some(())
        }
        Effect::Sequence(effects) => effects.iter().try_for_each(|effect| produce(game, effect, resolution, made)),
        // `resolve_mana_effect` refuses the rest, so the activation would fail.
        _ => None,
    }
}

/// The permanents `player` controls, oldest first (CR 613.7's timestamps,
/// which never tie), sorting only those (§3.5).
///
/// `controls` is asked in the map's order only while it walks no layers,
/// which it promises while no effect changes control: a walk writes a trace
/// record, and a record written in hash order would differ by process.
fn permanents_of(game: &GameState, player: PlayerId) -> Vec<ObjectId> {
    if game.continuous_effects.summary().any_control_changing {
        return game.battlefield_ids_ordered().into_iter().filter(|&id| controls(game, id, player)).collect();
    }
    let mut mine: Vec<(Timestamp, ObjectId)> = game
        .battlefield
        .iter()
        .filter(|&(&id, _)| controls(game, id, player))
        .map(|(&id, entry)| (entry.timestamp, id))
        .collect();
    mine.sort_unstable_by_key(|&(timestamp, _)| timestamp);
    mine.into_iter().map(|(_, id)| id).collect()
}

/// An ability, by its permanent and its place on that permanent's effective
/// list, which holds while the layer epoch it was read at does.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Located {
    permanent: ObjectId,
    index: usize,
}

/// The abilities on the battlefield that change what a production makes:
/// static replacement abilities watching one (CR 106.6a, 106.12b) and
/// triggered mana abilities (CR 605.1b), read off effective ability lists as
/// the gather and the dispatcher read them (§3.4).
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ProductionWatchers {
    replacements: Vec<Located>,
    triggers: Vec<Located>,
}

/// The board's watchers, from the memo when the layer epoch has not moved.
///
/// Every input is a layer-walk input (effective abilities, controllers,
/// attachments), so the frame memo's epoch argument covers this cache too
/// (`layers-architecture.md` §12, "7a"). A row's "as long as" and the
/// registry's resolution-made effects are not, and are read live.
fn production_watchers(game: &GameState) -> Arc<ProductionWatchers> {
    let epoch = game.layer_epoch();
    if let Some(found) = game.layer_memo.production_watchers(epoch) {
        #[cfg(debug_assertions)]
        if game.layer_memo.audited() {
            audit_watchers(game, &found);
        }
        return found;
    }
    let found = Arc::new(scan_watchers(game));
    game.layer_memo.insert_production_watchers(epoch, Arc::clone(&found));
    found
}

fn scan_watchers(game: &GameState) -> ProductionWatchers {
    let mut found = ProductionWatchers::default();
    for permanent in game.battlefield_ids_ordered() {
        let Some(chars) = compute_characteristics(game, permanent) else { continue };
        for (index, ability) in chars.abilities.iter().enumerate() {
            let at = Located { permanent, index };
            if let Effect::Triggered(def) = &ability.effect {
                if is_mana_ability(def) {
                    found.triggers.push(at);
                }
            } else if ability.ability_type == AbilityType::Static
                && functions_in(ability, &chars.types, Zone::Battlefield)
                && replacement_of(ability).is_some_and(|(def, _)| matches!(def.pattern, EventPattern::ProduceMana { .. }))
            {
                found.replacements.push(at);
            }
        }
    }
    found
}

/// The memo's debug mode, as the frame memo has one: every hit is checked
/// against a fresh scan, whose own reads are un-counted so a debug build's
/// rows are a release build's.
#[cfg(debug_assertions)]
fn audit_watchers(game: &GameState, served: &ProductionWatchers) {
    let counts = game.diagnostics.clone();
    let fresh = scan_watchers(game);
    game.diagnostics.rewind_to(&counts);
    debug_assert_eq!(
        &fresh,
        served,
        "the production-watcher memo served a stale scan at epoch {}: a write to an effective ability \
         list, a controller or an attachment skipped `GameState::bump_layer_epoch`",
        game.layer_epoch()
    );
}

/// What a production by `producer` for `player` can come to: through the
/// replacement effects that apply to it, and with what the triggered mana
/// abilities it sets off add (CR 605.1b, 605.4a).
fn made_by(
    game: &GameState,
    watchers: &ProductionWatchers,
    player: PlayerId,
    producer: ObjectId,
    tapped: bool,
    base: Bag,
) -> Vec<Bag> {
    let rewritten = rewritten(game, watchers, player, producer, tapped, base);
    if watchers.triggers.is_empty() {
        return rewritten;
    }
    let mut out: Vec<Bag> = Vec::new();
    for made in rewritten {
        let mut ways = vec![made];
        for &at in &watchers.triggers {
            let added = trigger_adds(game, watchers, at, player, producer, tapped, &made);
            if !added.is_empty() {
                ways = ways.iter().flat_map(|way| added.iter().map(move |a| sum(way, a))).collect();
            }
        }
        out.extend(ways);
    }
    undominated_bags(out)
}

/// What a production can become under the replacement effects that apply to
/// it, the board's and the registry's.
fn rewritten(
    game: &GameState,
    watchers: &ProductionWatchers,
    player: PlayerId,
    producer: ObjectId,
    tapped: bool,
    base: Bag,
) -> Vec<Bag> {
    let mut rewrites: Vec<ProductionRewrite> = Vec::new();
    for &at in &watchers.replacements {
        let abilities = get_effective_abilities(game, at.permanent);
        let Some((def, condition)) = abilities.get(at.index).and_then(replacement_of) else { continue };
        if condition.is_some_and(|condition| !settled_holds(condition, game, at.permanent, None)) {
            continue;
        }
        let controller = controller_or_owner(game, at.permanent).unwrap_or(0);
        if applies_to_production(game, def, at.permanent, controller, player, producer, tapped) {
            rewrites.push(ProductionRewrite::of(def));
        }
    }
    for row in game.replacement_effects.iter() {
        if applies_to_production(game, &row.def, row.source, row.controller, player, producer, tapped) {
            rewrites.push(ProductionRewrite::of(&row.def));
        }
    }
    if rewrites.is_empty() {
        return vec![base];
    }
    fold(base, &rewrites)
}

/// What the triggered mana ability at `at` adds to `player`'s pool when a
/// production by `producer` that made `made` sets it off: nothing when no arm
/// matches the production (the dispatcher's predicates), its intervening "if"
/// fails (CR 603.4), or its mana is another player's.
fn trigger_adds(
    game: &GameState,
    watchers: &ProductionWatchers,
    at: Located,
    player: PlayerId,
    producer: ObjectId,
    tapped: bool,
    made: &Bag,
) -> Vec<Bag> {
    let abilities = get_effective_abilities(game, at.permanent);
    let Some(Effect::Triggered(def)) = abilities.get(at.index).map(|ability| &ability.effect) else {
        return Vec::new();
    };
    let controller = controller_or_owner(game, at.permanent).unwrap_or(0);
    let host = game.battlefield.get(&at.permanent).and_then(|entry| entry.attached_to);
    let fires = def.condition.events().iter().any(|arm| match arm {
        TriggerEvent::ManaAdded { source, tapped_for_mana, mana } => {
            tapped_for_mana.is_none_or(|t| t == tapped)
                && mana.is_none_or(|m| made[slot(m)] > 0)
                && match source {
                    TriggerSubject::Any => true,
                    TriggerSubject::ThisObject => producer == at.permanent,
                    TriggerSubject::Host => host == Some(producer),
                    TriggerSubject::Filter(filter) => game
                        .object_matches_filter_of_source(producer, filter, controller, at.permanent, None)
                        .unwrap_or(false),
                }
        }
        // `is_mana_ability` admitted only `ManaAdded` arms.
        _ => false,
    });
    if !fires || def.intervening_if.as_ref().is_some_and(|c| !settled_holds(c, game, at.permanent, Some(controller))) {
        return Vec::new();
    }
    let mut adds = NO_MANA;
    if !trigger_mana(game, &def.effect, at.permanent, controller, host, player, &mut adds) || adds == NO_MANA {
        return Vec::new();
    }
    // CR 106.12: the trigger's source was not tapped, so a "tapped for mana"
    // replacement does not see its mana (Mana Reflection's ruling).
    rewritten(game, watchers, player, at.permanent, false, adds)
}

/// The mana a triggered mana ability's effect gives `player`, as
/// `resolve_mana_trigger_effect` decides whose it is: the host's controller
/// for `Host`, its own controller otherwise. `false` when an amount cannot
/// be read before it resolves.
fn trigger_mana(
    game: &GameState,
    effect: &Effect,
    source: ObjectId,
    controller: PlayerId,
    host: Option<ObjectId>,
    player: PlayerId,
    adds: &mut Bag,
) -> bool {
    match effect {
        Effect::Atom(Primitive::ProduceMana(output), recipient) => {
            let receives = match recipient {
                EffectRecipient::Host => host.and_then(|h| controller_or_owner(game, h)).unwrap_or(controller),
                _ => controller,
            };
            if receives != player {
                return true;
            }
            let resolution = ResolutionContext::untargeted(source, receives);
            for (mana_type, amount) in &output.mana {
                match game.evaluate_amount(amount, &resolution) {
                    Ok(n) => adds[slot(*mana_type)] += n,
                    Err(_) => return false,
                }
            }
            true
        }
        Effect::Sequence(effects) => {
            effects.iter().all(|effect| trigger_mana(game, effect, source, controller, host, player, adds))
        }
        _ => false,
    }
}

/// What one replacement effect does to a production, read as the pipeline's
/// production arms apply it (CR 106.6a, 106.12b).
#[derive(Debug, Clone, Copy, PartialEq)]
struct ProductionRewrite {
    kind: RewriteKind,
    /// "You may": declining is the player's (CR 614.5's one opportunity).
    optional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RewriteKind {
    Multiply(u64),
    Retype(usize),
    SetTo(usize, u64),
    /// No production: prevented, replaced by another event, or a pairing the
    /// pipeline refuses, which fails the activation.
    Nothing,
}

impl ProductionRewrite {
    fn of(def: &ReplacementDef) -> ProductionRewrite {
        let kind = match &def.rewrite {
            Rewrite::Amount(AmountRewrite::Multiplier(n)) => RewriteKind::Multiply(*n),
            Rewrite::Instead(GameActionTemplate::ProduceMana { mana_type, amount }) => match amount {
                TemplateAmount::ReplacedAmount => RewriteKind::Retype(slot(*mana_type)),
                TemplateAmount::Fixed(n) => RewriteKind::SetTo(slot(*mana_type), *n),
            },
            Rewrite::Prevent
            | Rewrite::Instead(_)
            | Rewrite::Amount(_)
            | Rewrite::EnterWith(_)
            | Rewrite::EnterAfterMoving(_)
            | Rewrite::EnterUnderControlOf(_)
            | Rewrite::EnterAsCopy(_)
            | Rewrite::Retarget(_) => RewriteKind::Nothing,
        };
        ProductionRewrite { kind, optional: def.optional }
    }
}

impl RewriteKind {
    fn apply(self, made: Bag) -> Option<Bag> {
        match self {
            RewriteKind::Multiply(n) => Some(made.map(|m| m.saturating_mul(n))),
            RewriteKind::Retype(t) => {
                let mut out = NO_MANA;
                out[t] = made.iter().sum();
                Some(out)
            }
            RewriteKind::SetTo(t, n) => {
                let mut out = NO_MANA;
                out[t] = n;
                Some(out)
            }
            RewriteKind::Nothing => None,
        }
    }
}

/// Past this many effects on one production, orders are not tried one by
/// one; the check leans yes instead (§3.3).
const MAX_ORDERED_REWRITES: usize = 6;

/// Every production `rewrites` can make of `base`: in every order its player
/// may choose (CR 616.1), each applied once (CR 614.5) and an optional one
/// declinable, less the dominated.
fn fold(base: Bag, rewrites: &[ProductionRewrite]) -> Vec<Bag> {
    if rewrites.len() > MAX_ORDERED_REWRITES {
        return most_of_every_type(base, rewrites);
    }
    let mut out = Vec::new();
    apply_in_every_order(base, rewrites, 0, &mut out);
    undominated_bags(out)
}

fn apply_in_every_order(made: Bag, rewrites: &[ProductionRewrite], applied: u32, out: &mut Vec<Bag>) {
    let mut last = true;
    for (i, rewrite) in rewrites.iter().enumerate() {
        if applied & (1 << i) != 0 {
            continue;
        }
        last = false;
        match rewrite.kind.apply(made) {
            Some(next) => apply_in_every_order(next, rewrites, applied | 1 << i, out),
            // Nothing after it applies to a production.
            None => out.push(NO_MANA),
        }
        if rewrite.optional {
            apply_in_every_order(made, rewrites, applied | 1 << i, out);
        }
    }
    if last {
        out.push(made);
    }
}

/// An outcome no order can beat, of each type an order could leave: the
/// most mana any order makes.
fn most_of_every_type(base: Bag, rewrites: &[ProductionRewrite]) -> Vec<Bag> {
    let mut most: u64 = base.iter().sum();
    let mut types = Types::of(&base);
    for rewrite in rewrites {
        match rewrite.kind {
            RewriteKind::SetTo(t, n) => {
                most = most.max(n);
                types = types.union(Types::one(t));
            }
            RewriteKind::Retype(t) => types = types.union(Types::one(t)),
            RewriteKind::Multiply(_) | RewriteKind::Nothing => {}
        }
    }
    for rewrite in rewrites {
        if let RewriteKind::Multiply(n) = rewrite.kind {
            most = most.saturating_mul(n.max(1));
        }
    }
    (0..6)
        .filter(|&t| types.has(t))
        .map(|t| {
            let mut made = NO_MANA;
            made[t] = most;
            made
        })
        .collect()
}

fn undominated_bags(bags: Vec<Bag>) -> Vec<Bag> {
    let mut kept: Vec<Bag> = Vec::new();
    for bag in bags {
        if kept.iter().any(|k| holds(k, &bag)) {
            continue;
        }
        kept.retain(|k| !holds(&bag, k));
        kept.push(bag);
    }
    kept
}

/// A cost's pips, by type, and its generic mana.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Demand {
    pips: Bag,
    generic: u64,
}

impl Demand {
    /// `None` for a symbol no payment path pays yet: hybrid and its kin are
    /// CP-1's, {S} item 33's, {X} MA-2's.
    fn of(cost: &ManaCost) -> Option<Demand> {
        let mut demand = Demand::default();
        for symbol in &cost.symbols {
            match symbol {
                ManaSymbol::Colored(mana_type) => demand.pips[slot(*mana_type)] += 1,
                ManaSymbol::Colorless => demand.pips[slot(ManaType::Colorless)] += 1,
                ManaSymbol::Generic => demand.generic += 1,
                ManaSymbol::Hybrid(..)
                | ManaSymbol::MonoHybrid(_)
                | ManaSymbol::Phyrexian(_)
                | ManaSymbol::HybridPhyrexian(..)
                | ManaSymbol::Snow
                | ManaSymbol::X => return None,
            }
        }
        Some(demand)
    }

    fn total(&self) -> u64 {
        self.pips.iter().sum::<u64>() + self.generic
    }

    fn kinds(&self) -> Types {
        Types::of(&self.pips)
    }

    /// What must be made before a doubler for this demand to be met after it.
    ///
    /// Every mana left in the pool becomes `factor` of its own type, so a pip
    /// of a type needs `ceil(n / factor)` of that type left, and the whole
    /// demand `ceil(total / factor)` left of any; the doubler's input is paid
    /// first, from what was made (§3.3). One Hall check at the end is then
    /// exact, with no enumeration of how the input was paid.
    fn before_doubling(&self, factor: u64, input: &Demand) -> Demand {
        let pips: Bag = std::array::from_fn(|t| self.pips[t].div_ceil(factor) + input.pips[t]);
        let per_type: u64 = self.pips.iter().map(|n| n.div_ceil(factor)).sum();
        let needed = per_type.max(self.total().div_ceil(factor)) + input.total();
        Demand { pips, generic: needed - pips.iter().sum::<u64>() }
    }
}

/// The supply one payment leaves, split by how each mana's type is chosen.
#[derive(Debug, Default)]
struct Pieces {
    /// Mana that is its own choice of type, by the set it can be: §3.2's
    /// table.
    free: Vec<(Types, u64)>,
    /// Entries with ways of more than one mana, several of one chosen type or
    /// ways of different sizes, with how many such entries.
    choices: Vec<(Vec<Bag>, u64)>,
}

/// Past this many leaves the check stops trying and leans yes: an
/// over-offer costs a rewind, an under-offer hides a legal play (§3.3).
const MAX_LEAVES: usize = 4096;

impl Pieces {
    fn add(&mut self, ways: &[Bag], copies: u64) {
        if copies == 0 || ways.is_empty() {
            return;
        }
        if let [made] = ways {
            for t in (0..6).filter(|&t| made[t] > 0) {
                self.add_free(Types::one(t), made[t] * copies);
            }
            return;
        }
        // One mana of any type in a set: a dual, Everywhere.
        if ways.iter().all(|way| way.iter().sum::<u64>() == 1) {
            let types = ways.iter().fold(Types::default(), |set, way| set.union(Types::of(way)));
            self.add_free(types, copies);
            return;
        }
        match self.choices.iter_mut().find(|(known, _)| known.as_slice() == ways) {
            Some((_, n)) => *n += copies,
            None => self.choices.push((ways.to_vec(), copies)),
        }
    }

    fn add_free(&mut self, types: Types, n: u64) {
        match self.free.iter_mut().find(|(known, _)| *known == types) {
            Some((_, have)) => *have += n,
            None => self.free.push((types, n)),
        }
    }

    fn pay(&self, demand: &Demand) -> bool {
        let asked = demand.kinds();
        let groups: Vec<(Vec<Bag>, u64)> =
            self.choices.iter().map(|(ways, copies)| (views(ways, asked), *copies)).collect();
        let first_copies = groups.first().map_or(0, |(_, copies)| *copies);
        Search { pieces: self, groups: &groups, demand, leaves: 0 }.from(0, 0, first_copies, NO_MANA)
    }

    /// Gale's condition (Hall's, for b-matchings): every set of the asked pip
    /// kinds asks for no more mana than the supply that can pay one of them,
    /// and the whole demand for no more than all of it (§3.2).
    fn feasible(&self, extra: &Bag, demand: &Demand) -> bool {
        let total = self.free.iter().map(|(_, n)| n).sum::<u64>() + extra.iter().sum::<u64>();
        if total < demand.total() {
            return false;
        }
        demand.kinds().subsets().all(|kinds| {
            let asked: u64 = (0..6).filter(|&t| kinds.has(t)).map(|t| demand.pips[t]).sum();
            let reach = self.free.iter().filter(|(types, _)| types.meets(kinds)).map(|(_, n)| n).sum::<u64>()
                + (0..6).filter(|&t| kinds.has(t)).map(|t| extra[t]).sum::<u64>();
            asked <= reach
        })
    }
}

/// Every way of spreading each group's copies over its ways, as a multiset,
/// ending in one Hall check each.
struct Search<'a> {
    pieces: &'a Pieces,
    groups: &'a [(Vec<Bag>, u64)],
    demand: &'a Demand,
    leaves: usize,
}

impl Search<'_> {
    /// From `group`'s `way` on, with `left` of its copies still to place and
    /// `extra` the mana the choices so far make.
    fn from(&mut self, group: usize, way: usize, left: u64, extra: Bag) -> bool {
        let groups = self.groups;
        let Some((ways, _)) = groups.get(group) else {
            self.leaves += 1;
            return self.pieces.feasible(&extra, self.demand);
        };
        if self.leaves >= MAX_LEAVES {
            return true;
        }
        if way + 1 == ways.len() {
            let extra = std::array::from_fn(|t| extra[t] + ways[way][t] * left);
            let next_copies = groups.get(group + 1).map_or(0, |(_, copies)| *copies);
            return self.from(group + 1, 0, next_copies, extra);
        }
        (0..=left).rev().any(|n| {
            let extra = std::array::from_fn(|t| extra[t] + ways[way][t] * n);
            self.from(group, way + 1, left - n, extra)
        })
    }

}

/// `ways` as a cost of `asked` kinds tells them apart: each asked type's mana,
/// and every other type's in one unasked slot, since any of it pays only
/// generic. Equal views are kept once and dominated ones dropped.
fn views(ways: &[Bag], asked: Types) -> Vec<Bag> {
    let other = (0..6).find(|&t| !asked.has(t));
    let seen = ways.iter().map(|way| {
        let mut view = NO_MANA;
        for t in 0..6 {
            match other {
                Some(o) if !asked.has(t) => view[o] += way[t],
                _ => view[t] += way[t],
            }
        }
        view
    });
    undominated_bags(seen.collect())
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    use super::*;
    use crate::cards::artifacts::{darksteel_myr, sol_ring};
    use crate::cards::basic_lands::{forest, island, plains};
    use crate::cards::dual_lands::{badlands, everywhere, tundra};
    use crate::cards::phase_cm_cards::{krark_clan_ironworks, mind_stone};
    use crate::cards::phase_ld_cards::march_of_the_machines;
    use crate::cards::phase_lf_cards::citanul_hierophants;
    use crate::cards::phase_rc_cards::chainbreaker;
    use crate::cards::phase_re9_cards::{deep_water, doubling_cube, mana_reflection, pale_moon};
    use crate::cards::phase_tr1_cards::wild_growth;
    use crate::engine::targeting::ChosenTargets;
    use crate::objects::card_data::{ActivationRestriction, CardData, CardDataBuilder};
    use crate::test_support::{put_on_battlefield, put_on_battlefield_this_turn, setup_two_player_game, test_dp};
    use crate::types::card_types::CardType;
    use crate::types::effects::ManaOutput;
    use crate::types::ids::new_object_id;
    use crate::types::mana::{ManaAtom, ManaPersistence, ManaRestriction};

    use ManaType::{Black, Blue, Colorless, Green, Red, White};

    fn cost(colored: &[ManaType], generic: u8) -> ManaCost {
        ManaCost::build(colored, generic)
    }

    /// Whether player 0 can pay `cost` for a spell with no other costs.
    fn covers(game: &GameState, cost: &ManaCost) -> bool {
        ManaSupply::take(game, 0).covers(game, cost, &Payment { source: None, other_costs: &[] })
    }

    /// The costs of `id`'s first activated ability besides its mana: what its
    /// payment takes.
    fn other_costs(game: &GameState, id: ObjectId) -> Vec<Cost> {
        let abilities = get_effective_abilities(game, id);
        let ability = abilities.iter().find(|a| a.ability_type == AbilityType::Activated).unwrap();
        ability.costs.iter().filter(|c| !matches!(c, Cost::Mana(_))).cloned().collect()
    }

    /// One Everywhere is one mana of any color, not five sources: SU-7's
    /// review board offered Grizzly Bears' {1}{G} from it.
    #[test]
    fn everywhere_is_one_mana_of_any_color() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, everywhere(), 0);
        assert!(covers(&game, &cost(&[Green], 0)));
        assert!(covers(&game, &cost(&[Black], 0)));
        assert!(!covers(&game, &cost(&[Green], 1)), "one tap, one mana");
    }

    /// The greedy count's under-offer that dropping a permanent's other
    /// sources would have made: {W}{U} from a Tundra and a Plains.
    #[test]
    fn a_dual_pays_the_pip_its_partner_cannot() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, tundra(), 0);
        put_on_battlefield(&mut game, plains(), 0);
        assert!(covers(&game, &cost(&[White, Blue], 0)));
        assert!(!covers(&game, &cost(&[Blue, Blue], 0)));
    }

    /// Sol Ring's one tap makes {C}{C}: with a Plains it pays {3}.
    #[test]
    fn sol_ring_makes_two() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, sol_ring(), 0);
        put_on_battlefield(&mut game, plains(), 0);
        assert!(covers(&game, &cost(&[], 3)));
        assert!(covers(&game, &cost(&[Colorless, Colorless, White], 0)));
        assert!(!covers(&game, &cost(&[], 4)));
    }

    /// Mana Reflection doubles a tap into two of the one type it makes, so an
    /// Everywhere under it is {W}{W} or {U}{U}, never {W}{U} (§3.3).
    #[test]
    fn a_doubled_land_of_several_types_makes_one_type_twice() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, mana_reflection(), 0);
        put_on_battlefield(&mut game, everywhere(), 0);
        assert!(covers(&game, &cost(&[White, White], 0)));
        assert!(covers(&game, &cost(&[], 2)));
        assert!(!covers(&game, &cost(&[White, Blue], 0)), "two of one type, not one of each");
        put_on_battlefield(&mut game, tundra(), 0);
        assert!(covers(&game, &cost(&[White, White], 2)), "Humility's cost: white twice and blue twice");
        assert!(!covers(&game, &cost(&[White, White, Blue], 2)));
    }

    /// Wild Growth's {G} comes with its land's tap and is not doubled (Mana
    /// Reflection's ruling): a Badlands pays Thornweald Archer's {1}{G}.
    #[test]
    fn a_triggered_mana_ability_adds_to_its_hosts_tap() {
        let mut game = setup_two_player_game();
        let land = put_on_battlefield(&mut game, badlands(), 0);
        let aura = put_on_battlefield(&mut game, wild_growth(), 0);
        assert!(game.attach(aura, land));
        assert!(covers(&game, &cost(&[Green], 1)));
        assert!(covers(&game, &cost(&[Black, Green], 0)));
        assert!(!covers(&game, &cost(&[Green, Green], 0)));

        put_on_battlefield(&mut game, mana_reflection(), 0);
        assert!(covers(&game, &cost(&[Black, Black, Green], 0)), "the land's black doubled");
        assert!(!covers(&game, &cost(&[Black, Green, Green], 0)), "the trigger's green is not");
        assert!(!covers(&game, &cost(&[Black, Red, Green], 0)), "two of one type");
    }

    /// Sol Ring made a creature and granted "{T}: Add {G}": one tap makes
    /// {C}{C} or {G}, so the entry keeps both ways and never pays {G}{G}
    /// (§3.3's ways of different sizes).
    #[test]
    fn ways_of_different_sizes_are_tried_one_at_a_time() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, sol_ring(), 0);
        put_on_battlefield(&mut game, march_of_the_machines(), 0);
        // This turn's, so its own granted tap cannot pay (CR 302.6).
        put_on_battlefield_this_turn(&mut game, citanul_hierophants(), 0);
        assert!(covers(&game, &cost(&[Colorless, Colorless], 0)));
        assert!(covers(&game, &cost(&[Green], 0)));
        assert!(!covers(&game, &cost(&[Green, Green], 0)), "no tap of Sol Ring makes two green");
        assert!(!covers(&game, &cost(&[Colorless, Green], 0)));
    }

    /// An ability whose cost taps its source cannot tap it for mana too:
    /// Chainbreaker with the Hierophants' grant pays its {3} with three other
    /// mana and not its own {G} (§3.3, the cost's own consumption).
    #[test]
    fn a_cost_that_taps_its_source_rules_out_that_source() {
        let mut game = setup_two_player_game();
        let breaker = put_on_battlefield(&mut game, chainbreaker(), 0);
        put_on_battlefield_this_turn(&mut game, citanul_hierophants(), 0);
        put_on_battlefield(&mut game, sol_ring(), 0);
        let other = other_costs(&game, breaker);
        let ability = Payment { source: Some(breaker), other_costs: &other };
        let supply = ManaSupply::take(&game, 0);
        assert!(supply.covers(&game, &cost(&[], 3), &Payment { source: None, other_costs: &[] }), "a spell may tap it");
        assert!(!supply.covers(&game, &cost(&[], 3), &ability));
        put_on_battlefield(&mut game, plains(), 0);
        assert!(ManaSupply::take(&game, 0).covers(&game, &cost(&[], 3), &ability));
    }

    /// Krark-Clan Ironworks makes {C}{C} for each artifact it can sacrifice,
    /// itself last, and a tapped Sol Ring is still an artifact to sacrifice.
    #[test]
    fn an_outlet_makes_mana_for_each_permanent_it_can_sacrifice() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, krark_clan_ironworks(), 0);
        put_on_battlefield(&mut game, sol_ring(), 0);
        assert!(covers(&game, &cost(&[], 6)), "Sol Ring's tap, then both sacrificed");
        assert!(!covers(&game, &cost(&[], 7)));
    }

    /// `cost-architecture.md` §3.11's puzzle: Mind Stone's draw sacrifices
    /// Mind Stone, so it can neither feed Ironworks nor tap for its {C}.
    #[test]
    fn a_cost_that_sacrifices_its_source_rules_it_out_as_fodder() {
        let mut game = setup_two_player_game();
        let stone = put_on_battlefield(&mut game, mind_stone(), 0);
        let other = other_costs(&game, stone);
        let paying = |game: &GameState, generic| {
            let ability = Payment { source: Some(stone), other_costs: &other };
            ManaSupply::take(game, 0).covers(game, &cost(&[], generic), &ability)
        };
        assert!(!paying(&game, 1), "its own mana needs the tap its cost takes");
        put_on_battlefield(&mut game, krark_clan_ironworks(), 0);
        assert!(paying(&game, 2), "Ironworks sacrificing itself");
        assert!(!paying(&game, 3), "not Mind Stone too");
    }

    /// A spell's own sacrifice comes out of an outlet's fodder, from outside
    /// it where it can (Bone Splinters with an artifact creature).
    #[test]
    fn a_spells_sacrifice_takes_from_the_fodder_last() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, krark_clan_ironworks(), 0);
        put_on_battlefield(&mut game, darksteel_myr(), 0);
        let sacrifice = [Cost::Sacrifice(ObjectFilter::ByType(CardType::Creature), 1)];
        let spell = Payment { source: None, other_costs: &sacrifice };
        let supply = ManaSupply::take(&game, 0);
        assert!(supply.covers(&game, &cost(&[], 4), &Payment { source: None, other_costs: &[] }));
        assert!(supply.covers(&game, &cost(&[], 2), &spell));
        assert!(!supply.covers(&game, &cost(&[], 3), &spell), "the Myr goes to the spell, not to Ironworks");
    }

    /// Doubling Cube: every other mana first, its {3} paid from it, and each
    /// mana left becomes two of its type (§3.3).
    #[test]
    fn doubling_cube_doubles_what_its_input_leaves() {
        let mut game = setup_two_player_game();
        for _ in 0..4 {
            put_on_battlefield(&mut game, plains(), 0);
        }
        for _ in 0..3 {
            put_on_battlefield(&mut game, island(), 0);
        }
        assert!(!covers(&game, &cost(&[], 8)));
        put_on_battlefield(&mut game, doubling_cube(), 0);
        assert!(covers(&game, &cost(&[], 8)), "seven, less three, doubled");
        assert!(!covers(&game, &cost(&[], 9)));
        assert!(covers(&game, &cost(&[Blue, Blue, Blue, Blue], 4)), "the input paid with Plains");
        assert!(!covers(&game, &cost(&[Blue; 7], 1)), "three Islands make six blue at most");
    }

    /// The Cube's doubling is a tap for mana, so Mana Reflection doubles what
    /// it adds: each mana left becomes three.
    #[test]
    fn a_multiplier_scales_the_doubling() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, mana_reflection(), 0);
        for _ in 0..4 {
            put_on_battlefield(&mut game, plains(), 0);
        }
        put_on_battlefield(&mut game, doubling_cube(), 0);
        assert!(covers(&game, &cost(&[], 15)), "eight white, less three, tripled");
        assert!(!covers(&game, &cost(&[], 16)));
    }

    /// Resolve `card`'s first ability for player 0, as a resolution would.
    fn resolve_first_ability(game: &mut GameState, card: Arc<CardData>) {
        let id = put_on_battlefield(game, card.clone(), 0);
        let ctx = ResolutionContext {
            source: id,
            ability_source: None,
            controller: 0,
            targets: ChosenTargets::NONE,
            replaced_amount: None,
            damage_prevented: None,
            trigger: None,
        };
        game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
    }

    /// A retype changes what a tap makes, and ignoring one would hide a play
    /// as well as offer one: a Forest under Deep Water pays {U} and not {G}.
    /// Two retypes on one tap are its player's order (CR 616.1): either type.
    #[test]
    fn a_retype_reads_what_the_land_makes_under_it() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, forest(), 0);
        resolve_first_ability(&mut game, deep_water());
        assert!(covers(&game, &cost(&[Blue], 0)));
        assert!(!covers(&game, &cost(&[Green], 0)));

        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, tundra(), 0);
        resolve_first_ability(&mut game, deep_water());
        resolve_first_ability(&mut game, pale_moon());
        assert!(covers(&game, &cost(&[Blue], 0)));
        assert!(covers(&game, &cost(&[Colorless], 0)));
        assert!(!covers(&game, &cost(&[White], 0)));
    }

    fn land_making(name: &str, output: ManaOutput) -> Arc<CardData> {
        CardDataBuilder::new(name)
            .card_type(CardType::Land)
            .ability(AbilityDef {
                rules_text: "".into(),
                is_characteristic_defining: false,
                activation_restriction: ActivationRestriction::None,
                id: AbilityId::UNASSIGNED,
                instances: Vec::new(),
                ability_type: AbilityType::Mana,
                costs: vec![Cost::TapSelf],
                effect: Effect::Atom(Primitive::ProduceMana(output), EffectRecipient::Implicit),
            })
            .build()
    }

    /// Restricted mana is not counted until a payment spends it (§3.6, item
    /// 33): `ManaPool::pay` reads only the free pool.
    #[test]
    fn restricted_mana_is_not_counted() {
        let restricted = ManaAtom {
            mana_type: Green,
            source_id: None,
            restrictions: vec![ManaRestriction::OnlyForSpellTypes(vec![CardType::Creature])],
            grants: Vec::new(),
            persistence: ManaPersistence::Normal,
        };
        let mut game = setup_two_player_game();
        let output = ManaOutput { mana: Vec::new(), special: vec![restricted.clone()] };
        put_on_battlefield(&mut game, land_making("Fixture: restricted green", output), 0);
        assert!(!covers(&game, &cost(&[Green], 0)));
        let output = ManaOutput { mana: vec![(Green, AmountExpr::Fixed(1))], special: vec![restricted] };
        put_on_battlefield(&mut game, land_making("Fixture: one free green, one restricted", output), 0);
        assert!(covers(&game, &cost(&[Green], 0)));
        assert!(!covers(&game, &cost(&[Green, Green], 0)));
    }

    /// Each ability's own costs decide: a tapped land and another player's
    /// are no source, and a tapped Morgue Toad is, since "Sacrifice this
    /// creature: Add {U}{R}" takes no tap (CR 605.1a).
    #[test]
    fn a_source_is_read_by_its_abilitys_own_costs() {
        let morgue_toad = CardDataBuilder::new("Morgue Toad")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .ability(AbilityDef {
                rules_text: "Sacrifice this creature: Add {U}{R}.".into(),
                is_characteristic_defining: false,
                activation_restriction: ActivationRestriction::None,
                id: AbilityId::UNASSIGNED,
                instances: Vec::new(),
                ability_type: AbilityType::Mana,
                costs: vec![Cost::SacrificeSelf],
                effect: Effect::Atom(
                    Primitive::ProduceMana(ManaOutput {
                        mana: vec![(Blue, AmountExpr::Fixed(1)), (Red, AmountExpr::Fixed(1))],
                        special: Vec::new(),
                    }),
                    EffectRecipient::Implicit,
                ),
            })
            .build();
        let mut game = setup_two_player_game();
        let land = put_on_battlefield(&mut game, forest(), 0);
        game.battlefield.get_mut(&land).unwrap().tapped = true;
        put_on_battlefield(&mut game, forest(), 1);
        assert!(available_mana_sources(&game, 0).is_empty());
        assert!(!covers(&game, &cost(&[Green], 0)));

        let toad = put_on_battlefield(&mut game, morgue_toad, 0);
        game.battlefield.get_mut(&toad).unwrap().tapped = true;
        assert_eq!(available_mana_sources(&game, 0).len(), 2, "a row for each type it makes");
        assert!(covers(&game, &cost(&[Blue, Red], 0)));
    }

    /// A symbol no payment path pays yet is refused, however much mana there
    /// is, as `ManaPool::pay` refuses it; {0} is free.
    #[test]
    fn a_symbol_nothing_pays_is_refused() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, everywhere(), 0);
        put_on_battlefield(&mut game, everywhere(), 0);
        assert!(!covers(&game, &ManaCost::from_symbols(vec![ManaSymbol::Hybrid(White, Blue)])));
        assert!(!covers(&game, &ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(Red)])));
        assert!(covers(&game, &ManaCost::zero()));
    }

    #[test]
    fn the_pool_is_supply() {
        let mut game = setup_two_player_game();
        game.players[0].mana_pool.add(White, 1);
        put_on_battlefield(&mut game, plains(), 0);
        assert!(covers(&game, &cost(&[White, White], 0)));
        assert!(!covers(&game, &cost(&[White, White, White], 0)));
    }

    /// The watcher scan is taken once a layer epoch, for every player's
    /// inventory, and again once the board moves.
    #[test]
    fn the_watcher_scan_is_taken_once_an_epoch() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, mana_reflection(), 0);
        ManaSupply::take(&game, 0);
        let epoch = game.layer_epoch();
        let first = game.layer_memo.production_watchers(epoch).unwrap();
        ManaSupply::take(&game, 1);
        assert!(Arc::ptr_eq(&first, &game.layer_memo.production_watchers(epoch).unwrap()), "one scan");
        put_on_battlefield(&mut game, plains(), 0);
        assert!(game.layer_memo.production_watchers(game.layer_epoch()).is_none(), "the board moved");
        ManaSupply::take(&game, 0);
        assert!(game.layer_memo.production_watchers(game.layer_epoch()).is_some());
    }

    /// The memo's debug mode: a write that changes what the scan finds and
    /// skips its bump is caught on the next hit rather than served.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "production-watcher memo served a stale scan")]
    fn a_skipped_bump_is_caught_by_the_watcher_audit() {
        let mut game = setup_two_player_game();
        let reflection = put_on_battlefield(&mut game, mana_reflection(), 0);
        ManaSupply::take(&game, 0);
        game.battlefield.remove(&reflection);
        ManaSupply::take(&game, 0);
    }

    /// The window's offer is read once: a tap, which moves no epoch, is seen
    /// by re-asking the costs, and a sacrifice moves the board, after which
    /// the offer is stale and read again (§3.8).
    #[test]
    fn the_window_offer_rechecks_costs_and_is_read_again_when_the_board_moves() {
        use crate::engine::actions::ZoneChangeCause;
        use crate::test_support::test_ctx;

        let mut game = setup_two_player_game();
        let land = put_on_battlefield(&mut game, plains(), 0);
        put_on_battlefield(&mut game, everywhere(), 0);
        let ironworks = put_on_battlefield(&mut game, krark_clan_ironworks(), 0);
        put_on_battlefield_this_turn(&mut game, citanul_hierophants(), 0);
        let offer = WindowOffer::take(&game, 0);
        let options = offer.options(&game);
        assert_eq!(options.len(), 7, "a Plains, Everywhere's five, Ironworks; not a creature this turn's");
        assert_eq!(options[0].0, land, "timestamp order");

        game.battlefield.get_mut(&land).unwrap().tapped = true;
        assert!(offer.is_current(&game), "a tap reads no layer");
        assert_eq!(offer.options(&game).len(), 6);

        game.change_zone(ironworks, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx()).unwrap();
        assert!(!offer.is_current(&game));
        assert_eq!(WindowOffer::take(&game, 0).options(&game).len(), 5);
    }

    /// Two retypes on one production leave either type, in its player's
    /// order (CR 616.1); a multiplier commutes with both.
    #[test]
    fn a_production_is_rewritten_in_every_order() {
        let rewrite = |kind| ProductionRewrite { kind, optional: false };
        let rewrites = [
            rewrite(RewriteKind::Retype(slot(Blue))),
            rewrite(RewriteKind::Multiply(2)),
            rewrite(RewriteKind::Retype(slot(Colorless))),
        ];
        let mut green = NO_MANA;
        green[slot(Green)] = 1;
        let mut made = fold(green, &rewrites);
        made.sort();
        assert_eq!(made, vec![[0, 0, 0, 0, 0, 2], [0, 2, 0, 0, 0, 0]]);

        let declinable = ProductionRewrite { optional: true, ..rewrites[0] };
        assert_eq!(fold(green, &[declinable]).len(), 2, "applied, or declined");
        let past_the_cap = vec![rewrite(RewriteKind::Multiply(2)); MAX_ORDERED_REWRITES + 1];
        assert_eq!(fold(green, &past_the_cap), vec![[0, 0, 0, 0, 128, 0]]);
    }

    /// The inventory's reading of a tap, against the engine making it: on each
    /// board, every once-a-use mana ability, activated through
    /// `activate_mana_ability` under providers that answer CR 616.1's order
    /// every way, adds exactly the productions the inventory predicts.
    #[test]
    fn what_the_inventory_predicts_a_tap_makes_is_what_the_engine_makes() {
        use crate::cards::phase_re9_cards::nyxbloom_ancient;
        use crate::engine::actions::ActionContext;
        use crate::ui::random::RandomDecisionProvider;

        let pool = |game: &GameState| {
            let mut made = NO_MANA;
            for (&t, &n) in game.players[0].mana_pool.available() {
                made[slot(t)] = n;
            }
            made
        };
        let mut boards: Vec<GameState> = Vec::new();
        let mut game = setup_two_player_game();
        let land = put_on_battlefield(&mut game, badlands(), 0);
        let aura = put_on_battlefield(&mut game, wild_growth(), 0);
        game.attach(aura, land);
        put_on_battlefield(&mut game, mana_reflection(), 0);
        put_on_battlefield(&mut game, nyxbloom_ancient(), 0);
        boards.push(game);
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, tundra(), 0);
        put_on_battlefield(&mut game, everywhere(), 0);
        put_on_battlefield(&mut game, mana_reflection(), 0);
        resolve_first_ability(&mut game, deep_water());
        resolve_first_ability(&mut game, pale_moon());
        boards.push(game);
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, sol_ring(), 0);
        put_on_battlefield(&mut game, march_of_the_machines(), 0);
        put_on_battlefield(&mut game, citanul_hierophants(), 0);
        put_on_battlefield(&mut game, krark_clan_ironworks(), 0);
        boards.push(game);

        let mut checked = 0;
        for game in &boards {
            let watchers = production_watchers(game);
            for id in permanents_of(game, 0) {
                for ability in get_effective_abilities(game, id).iter().filter(|a| a.ability_type == AbilityType::Mana) {
                    let taps = match activation_of(ability) {
                        Activation::Once { taps, .. } => taps,
                        Activation::PerSacrifice { .. } => false,
                        Activation::Doubling { .. } | Activation::Unread => continue,
                    };
                    let base = production_of(game, &ability.effect, id, 0).unwrap();
                    let mut predicted = made_by(game, &watchers, 0, id, taps, base);
                    let mut made: Vec<Bag> = Vec::new();
                    for seed in 0..16 {
                        let mut fork = game.clone();
                        let before = pool(&fork);
                        let dp = RandomDecisionProvider::seeded(seed);
                        fork.activate_mana_ability(0, id, ability.id, &ActionContext::new(&dp)).unwrap();
                        let after = pool(&fork);
                        let delta = std::array::from_fn(|t| after[t] - before[t]);
                        if !made.contains(&delta) {
                            made.push(delta);
                        }
                    }
                    predicted.sort();
                    made.sort();
                    assert_eq!(predicted, made, "{}: {}", ability.rules_text.words, crate::ui::display::named(game, id));
                    checked += 1;
                }
            }
        }
        assert!(checked >= 10, "{checked} abilities checked");
    }

    /// Every way of using every entry, then each doubler in turn with its
    /// input paid every way it can be: the answer the check must agree with.
    fn pays_by_search(supply: &ManaSupply, demand: &Demand) -> bool {
        let mut made = vec![NO_MANA];
        for entry in &supply.entries {
            made = made.iter().flat_map(|m| entry.ways.iter().map(move |w| sum(m, &w.makes))).collect();
        }
        made.iter().any(|pool| pays_after(*pool, &supply.doublers, demand))
    }

    fn pays_after(pool: Bag, doublers: &[Doubler], demand: &Demand) -> bool {
        if (0..6).all(|t| pool[t] >= demand.pips[t]) && pool.iter().sum::<u64>() >= demand.total() {
            return true;
        }
        let Some((doubler, rest)) = doublers.split_first() else { return false };
        let pips_paid = (0..6).try_fold(pool, |mut left, t| {
            left[t] = left[t].checked_sub(doubler.input.pips[t])?;
            Some(left)
        });
        let Some(pips_paid) = pips_paid else { return false };
        spend(pips_paid, 0, doubler.input.generic)
            .into_iter()
            .any(|left| pays_after(left.map(|n| n * doubler.factor), rest, demand))
    }

    /// Every pool left by spending `n` generic from `pool`'s types `t` on.
    fn spend(pool: Bag, t: usize, n: u64) -> Vec<Bag> {
        if n == 0 {
            return vec![pool];
        }
        if t == 6 {
            return Vec::new();
        }
        (0..=n.min(pool[t]))
            .flat_map(|k| {
                let mut left = pool;
                left[t] -= k;
                spend(left, t + 1, n - k)
            })
            .collect()
    }

    fn one(t: usize, n: u64) -> Bag {
        let mut made = NO_MANA;
        made[t] = n;
        made
    }

    /// A random entry of one of §3.3's shapes: one mana of a set of types,
    /// several of one chosen type, or ways of different sizes.
    fn random_entry(rng: &mut StdRng) -> Entry {
        let ways = rng.random_range(1..=3);
        let makes: Vec<Bag> = match rng.random_range(0..3) {
            0 => (0..ways).map(|_| one(rng.random_range(0..6), 1)).collect(),
            1 => {
                let n = rng.random_range(2..=3);
                (0..ways).map(|_| one(rng.random_range(0..6), n)).collect()
            }
            _ => (0..ways)
                .map(|_| {
                    let first = one(rng.random_range(0..6), rng.random_range(1..=2));
                    sum(&first, &one(rng.random_range(0..6), rng.random_range(0..=2)))
                })
                .collect(),
        };
        let ways = makes.into_iter().map(|makes| Way { makes, taps: false, sacrifices: false }).collect();
        Entry { permanent: Some(new_object_id()), ways: undominated(ways) }
    }

    /// The property MA-1 rests on: on small random boards, Hall's condition
    /// with §3.3's shapes answers what trying every payment answers.
    #[test]
    fn the_check_agrees_with_an_exhaustive_search() {
        let mut rng = StdRng::seed_from_u64(162);
        let (mut yes, mut no) = (0, 0);
        for _ in 0..3000 {
            let entries = (0..rng.random_range(0..=5)).map(|_| random_entry(&mut rng)).collect();
            let doublers = (0..rng.random_range(0..=2))
                .map(|_| {
                    let mut input = Demand { pips: NO_MANA, generic: rng.random_range(1..=3) };
                    if rng.random_bool(0.3) {
                        input.pips[rng.random_range(0..6)] = 1;
                    }
                    Doubler { permanent: new_object_id(), input, factor: rng.random_range(2..=3) }
                })
                .collect();
            let supply = ManaSupply { player: 0, entries, outlets: Vec::new(), doublers };
            let mut demand = Demand { pips: NO_MANA, generic: rng.random_range(0..=6) };
            for _ in 0..rng.random_range(0..=3) {
                demand.pips[rng.random_range(0..6)] += rng.random_range(1..=3);
            }
            let expected = pays_by_search(&supply, &demand);
            assert_eq!(supply.pays(&demand, &Taken::default()), expected, "{supply:#?}\n{demand:?}");
            if expected { yes += 1 } else { no += 1 }
        }
        assert!(yes > 500 && no > 500, "both answers, often: {yes} yes, {no} no");
    }
}
