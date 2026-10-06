//! What a player can pay with, and whether it pays a cost: the check behind
//! the priority question's offer (`mana-architecture.md` §3).
//!
//! **The inventory**, [`ManaSupply`], is read once per player per priority
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

use std::cell::OnceCell;
use std::sync::Arc;

use crate::engine::layers::compute_characteristics;
use crate::engine::layers::condition::settled_holds;
use crate::engine::replacement::{applies_to_mana_production, replacement_of};
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
type ManaBag = [u64; 6];

const NO_MANA: ManaBag = [0; 6];

const ALL_MANA_TYPES: [ManaType; 6] =
    [ManaType::White, ManaType::Blue, ManaType::Black, ManaType::Red, ManaType::Green, ManaType::Colorless];

fn slot(mana_type: ManaType) -> usize {
    mana_type as usize
}

fn sum(a: &ManaBag, b: &ManaBag) -> ManaBag {
    std::array::from_fn(|t| a[t] + b[t])
}

/// Whether `a` holds at least `b`'s mana of every type.
fn holds(a: &ManaBag, b: &ManaBag) -> bool {
    (0..6).all(|t| a[t] >= b[t])
}

/// A set of the six types, a bit each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct ManaTypes(u8);

impl ManaTypes {
    fn one(slot: usize) -> ManaTypes {
        ManaTypes(1 << slot)
    }

    fn of(made: &ManaBag) -> ManaTypes {
        ManaTypes((0..6).filter(|&t| made[t] > 0).fold(0, |set, t| set | 1 << t))
    }

    fn has(self, slot: usize) -> bool {
        self.0 & (1 << slot) != 0
    }

    fn meets(self, other: ManaTypes) -> bool {
        self.0 & other.0 != 0
    }

    fn union(self, other: ManaTypes) -> ManaTypes {
        ManaTypes(self.0 | other.0)
    }

    /// Every nonempty subset.
    fn subsets(self) -> impl Iterator<Item = ManaTypes> {
        let all = self.0;
        let mut next = all;
        std::iter::from_fn(move || {
            if next == 0 {
                return None;
            }
            let current = next;
            next = (next - 1) & all;
            Some(ManaTypes(current))
        })
    }
}

/// One way of using an entry, and what it makes.
#[derive(Debug, Clone, PartialEq)]
struct EntryWay {
    makes: ManaBag,
    /// It taps the permanent, so a cost that taps the same permanent rules it
    /// out.
    taps: bool,
    /// It sacrifices the permanent, so a cost that sacrifices the same
    /// permanent rules it out.
    sacrifices: bool,
}

impl EntryWay {
    /// Never worse than `other`: at least its mana of every type, for no more
    /// of the permanent. The second half keeps a way a cost's exclusion could
    /// leave alone (§3.3).
    fn dominates(&self, other: &EntryWay) -> bool {
        holds(&self.makes, &other.makes)
            && (!self.taps || other.taps)
            && (!self.sacrifices || other.sacrifices)
    }
}

/// `ways` less every way another dominates, in their order.
fn undominated(ways: Vec<EntryWay>) -> Vec<EntryWay> {
    let mut kept: Vec<EntryWay> = Vec::new();
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
struct SupplyEntry {
    /// `None` for the pool.
    permanent: Option<ObjectId>,
    ways: Vec<EntryWay>,
}

/// "Sacrifice a [filter]: Add mana" (Krark-Clan Ironworks): used once for
/// each permanent it can sacrifice (CR 701.21a), itself last.
#[derive(Debug, Clone, PartialEq)]
struct SacrificeOutlet {
    permanent: ObjectId,
    /// Outlets of one ability share their fodder, so they are counted once.
    definition: AbilityId,
    needs: u32,
    /// The player's permanents its filter matches, itself among them.
    fodder: Vec<ObjectId>,
    /// What one activation can make.
    ways: Vec<ManaBag>,
}

/// A mana ability that adds the unspent mana in the pool again, for an input
/// paid first: Doubling Cube's "{3}, {T}: Double the amount of each type of
/// unspent mana you have".
#[derive(Debug, Clone, PartialEq)]
struct PoolMultiplier {
    permanent: ObjectId,
    input: ManaDemand,
    /// What one mana left in the pool becomes: itself and what the ability
    /// adds for it, 2 for Doubling Cube, 3 under Mana Reflection, 4 under
    /// Nyxbloom Ancient.
    factor: u64,
}

/// What `player` can pay mana with at one moment (§3.1).
#[derive(Debug, Clone)]
pub struct ManaSupply {
    player: PlayerId,
    entries: Vec<SupplyEntry>,
    sacrifice_outlets: Vec<SacrificeOutlet>,
    pool_multipliers: Vec<PoolMultiplier>,
    /// The split for a payment that reserves none of what the entries spend
    /// and uses no pool multiplier, nearly every check: made once, at the first.
    plain_split: OnceCell<SplitSupply>,
}

/// What a mana cost is paid for besides its mana: the payment's other costs.
/// CR 601.2h pays them after the window, so what they take is not there to
/// make mana with: a cost's own {T} rules out its source's tap, its own
/// sacrifice the source as anything's fodder (§3.3).
#[derive(Debug, Clone, Copy)]
pub struct NonManaCosts<'a> {
    /// The permanent whose ability is activated; `None` for a spell.
    pub source: Option<ObjectId>,
    pub costs: &'a [Cost],
}

impl ManaSupply {
    /// What `player` can pay with now: the pool, and every mana ability of a
    /// permanent they control whose other costs can be paid, in timestamp
    /// order.
    pub fn read(game: &GameState, player: PlayerId) -> ManaSupply {
        let watchers = mana_production_watchers(game);
        // Nothing on the board or in the registry changes a production, as on
        // nearly every board: each tap makes what it prints.
        let quiet = watchers.replacements.is_empty()
            && watchers.triggers.is_empty()
            && !game.replacement_effects.iter().any(|row| matches!(row.def.pattern, EventPattern::ProduceMana { .. }));
        let mine = permanents_of(game, player);
        let mut supply = ManaSupply {
            player,
            entries: Vec::new(),
            sacrifice_outlets: Vec::new(),
            pool_multipliers: Vec::new(),
            plain_split: OnceCell::new(),
        };
        if let Some(state) = game.players.get(player) {
            let mut pool = NO_MANA;
            for (&mana_type, &n) in state.mana_pool.available() {
                pool[slot(mana_type)] += n;
            }
            if pool.iter().any(|&n| n > 0) {
                let way = EntryWay { makes: pool, taps: false, sacrifices: false };
                supply.entries.push(SupplyEntry { permanent: None, ways: vec![way] });
            }
        }
        for &id in &mine {
            let mut ways: Vec<EntryWay> = Vec::new();
            // Sacrificing it for mana leaves its {T} free to use first.
            let mut alone: Vec<EntryWay> = Vec::new();
            for ability in get_effective_abilities(game, id).iter().filter(|a| a.ability_type == AbilityType::Mana) {
                match mana_ability_cost_of(ability) {
                    ManaAbilityCost::SpendsItsTapOrItself { taps, sacrifices } => {
                        if game.can_pay_costs(&ability.costs, player, id).is_err() {
                            continue;
                        }
                        let Some(base) = mana_production_of(game, &ability.effect, id, player) else { continue };
                        let into = if sacrifices && !taps { &mut alone } else { &mut ways };
                        if quiet {
                            into.push(EntryWay { makes: base, taps, sacrifices });
                        } else {
                            into.extend(mana_made_by(game, &watchers, player, id, taps, base).into_iter().map(|makes| EntryWay {
                                makes,
                                taps,
                                sacrifices,
                            }));
                        }
                    }
                    ManaAbilityCost::SacrificesOthers { filter, needs } => {
                        let fodder: Vec<ObjectId> = mine
                            .iter()
                            .copied()
                            .filter(|&f| game.object_matches_filter(f, filter, player).unwrap_or(false))
                            .collect();
                        if fodder.len() < needs as usize {
                            continue;
                        }
                        let Some(base) = mana_production_of(game, &ability.effect, id, player) else { continue };
                        let ways = if quiet { vec![base] } else { mana_made_by(game, &watchers, player, id, false, base) };
                        let definition = ability.id.definition();
                        supply.sacrifice_outlets.push(SacrificeOutlet { permanent: id, definition, needs, fodder, ways });
                    }
                    ManaAbilityCost::MultipliesThePool { input } => {
                        let Some(input) = ManaDemand::of(input) else { continue };
                        if game.can_pay_costs(&[Cost::TapSelf], player, id).is_err() {
                            continue;
                        }
                        // One mana left in the pool, and what the ability adds for it,
                        // which is a tap for mana (CR 106.12): a multiplier on taps
                        // scales it.
                        let mut unit = NO_MANA;
                        unit[slot(ManaType::Colorless)] = 1;
                        let made = mana_after_replacements(game, &watchers, player, id, true, unit);
                        // A retype would make the added mana all one type, which no
                        // printed pool multiplier meets; the check does not read it.
                        if made.iter().any(|m| ManaTypes::of(m) != ManaTypes::one(slot(ManaType::Colorless))) {
                            continue;
                        }
                        let added = made.iter().map(|m| m[slot(ManaType::Colorless)]).max().unwrap_or(0);
                        if added > 0 {
                            supply.pool_multipliers.push(PoolMultiplier { permanent: id, input, factor: 1 + added });
                        }
                    }
                    ManaAbilityCost::NotCounted => {}
                }
            }
            if !alone.is_empty() {
                let tapped: Vec<EntryWay> = ways.iter().filter(|w| !w.sacrifices).cloned().collect();
                for t in &tapped {
                    for s in &alone {
                        ways.push(EntryWay { makes: sum(&t.makes, &s.makes), taps: true, sacrifices: true });
                    }
                }
                ways.extend(alone);
            }
            let ways = if ways.len() > 1 { undominated(ways) } else { ways };
            if !ways.is_empty() {
                supply.entries.push(SupplyEntry { permanent: Some(id), ways });
            }
        }
        supply
    }

    /// Whether this inventory pays `cost` for `payment`. A symbol no payment
    /// path pays yet is refused, as `ManaPool::pay` refuses it (§3.2): the
    /// offer must agree with the payment (`cost-architecture.md` §3.6).
    pub fn covers(&self, game: &GameState, cost: &ManaCost, non_mana: &NonManaCosts<'_>) -> bool {
        let Some(demand) = ManaDemand::of(cost) else { return false };
        let reserved = ReservedByCosts::of(game, self.player, non_mana, !self.sacrifice_outlets.is_empty());
        demand.total() == 0 || self.pays(&demand, &reserved)
    }

    fn pays(&self, demand: &ManaDemand, reserved: &ReservedByCosts) -> bool {
        let leaves_the_entries = !self.entries.iter().any(|e| e.permanent.is_some_and(|p| reserved.reserves(p)))
            && (self.sacrifice_outlets.is_empty() || reserved.reserves_no_fodder());
        let plain = if leaves_the_entries {
            self.plain_split.get_or_init(|| self.split(&ReservedByCosts::default(), &[])).pays(demand)
        } else {
            self.split(reserved, &[]).pays(demand)
        };
        if plain {
            return true;
        }
        // Every other entry is used before a pool multiplier, since mana made
        // after it is not multiplied; what cannot be, comes after and pays as
        // made.
        let pool_multipliers: Vec<&PoolMultiplier> =
            self.pool_multipliers.iter().filter(|d| !reserved.reserves(d.permanent)).collect();
        (1..=pool_multipliers.len()).any(|used| {
            let used = &pool_multipliers[..used];
            let after = self.made_after_multiplying(reserved, used);
            let asked = used.iter().rev().fold(demand.less(&after), |asked, d| asked.before_multiplying(d.factor, &d.input));
            self.split(reserved, used).pays(&asked)
        })
    }

    /// What a sacrifice outlet makes after the pool multipliers in `used`: a
    /// multiplier it can sacrifice is sacrificed after it adds, and so is the last outlet
    /// of the ability, which must still be there to do it (Krark-Clan
    /// Ironworks beside Doubling Cube).
    fn made_after_multiplying(&self, reserved: &ReservedByCosts, used: &[&PoolMultiplier]) -> ManaBag {
        let mut after = NO_MANA;
        for outlet in self.sacrifice_outlets_counted() {
            let late = used.iter().filter(|d| outlet.fodder.contains(&d.permanent)).count() as u64;
            if late == 0 {
                continue;
            }
            let last = u64::from(outlet.fodder.contains(&outlet.permanent));
            let activations = (late + last).min(reserved.fodder_left(&outlet.fodder)) / u64::from(outlet.needs.max(1));
            // Its first way, which is every printed outlet's only one.
            if let Some(made) = outlet.ways.first() {
                after = std::array::from_fn(|t| after[t] + made[t] * activations);
            }
        }
        after
    }

    /// One outlet of each ability: outlets of one ability share their fodder.
    fn sacrifice_outlets_counted(&self) -> impl Iterator<Item = &SacrificeOutlet> {
        self.sacrifice_outlets.iter().enumerate().filter_map(|(i, outlet)| {
            (!self.sacrifice_outlets[..i].iter().any(|earlier| earlier.definition == outlet.definition)).then_some(outlet)
        })
    }

    /// The supply this payment leaves, split by how each mana's type is
    /// chosen.
    fn split(&self, reserved: &ReservedByCosts, multiplying: &[&PoolMultiplier]) -> SplitSupply {
        let mut pieces = SplitSupply::default();
        for entry in &self.entries {
            let multiplies = multiplying.iter().any(|d| Some(d.permanent) == entry.permanent);
            let ways: Vec<ManaBag> = entry
                .ways
                .iter()
                .filter(|w| reserved.allows(entry.permanent, w) && !(w.taps && multiplies))
                .map(|w| w.makes)
                .collect();
            pieces.add(&ways, 1);
        }
        for outlet in self.sacrifice_outlets_counted() {
            // Less what is sacrificed after a pool multiplier (`made_after_multiplying`).
            let late = multiplying.iter().filter(|d| outlet.fodder.contains(&d.permanent)).count() as u64;
            let last = u64::from(late > 0 && outlet.fodder.contains(&outlet.permanent));
            let activations = reserved.fodder_left(&outlet.fodder).saturating_sub(late + last) / u64::from(outlet.needs.max(1));
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
    let watchers = mana_production_watchers(game);
    let mut sources = Vec::new();
    for id in permanents_of(game, player_id) {
        for ability in get_effective_abilities(game, id).iter() {
            if ability.ability_type != AbilityType::Mana || game.can_pay_costs(&ability.costs, player_id, id).is_err() {
                continue;
            }
            for produces in mana_types_made(game, &watchers, ability, id, player_id) {
                sources.push(ManaSource { permanent_id: id, ability_id: ability.id, produces });
            }
        }
    }
    sources
}

/// The types `ability` makes now, after what the board does to its
/// production; for a shape the inventory does not read, the fixed amounts it
/// prints.
fn mana_types_made(
    game: &GameState,
    watchers: &ManaProductionWatchers,
    ability: &AbilityDef,
    permanent: ObjectId,
    player: PlayerId,
) -> Vec<ManaType> {
    let taps = match mana_ability_cost_of(ability) {
        ManaAbilityCost::SpendsItsTapOrItself { taps, .. } => Some(taps),
        ManaAbilityCost::SacrificesOthers { .. } => Some(false),
        ManaAbilityCost::MultipliesThePool { .. } | ManaAbilityCost::NotCounted => None,
    };
    let made = match (taps, mana_production_of(game, &ability.effect, permanent, player)) {
        (Some(taps), Some(base)) => mana_made_by(game, watchers, player, permanent, taps, base),
        _ => {
            let Effect::Atom(Primitive::ProduceMana(output), _) = &ability.effect else { return Vec::new() };
            let fixed = output.mana.iter().filter(|(_, amount)| matches!(amount, AmountExpr::Fixed(n) if *n > 0));
            return fixed.map(|(mana_type, _)| *mana_type).collect();
        }
    };
    let types = made.iter().fold(ManaTypes::default(), |set, m| set.union(ManaTypes::of(m)));
    ALL_MANA_TYPES.into_iter().filter(|&t| types.has(slot(t))).collect()
}

/// The mana abilities CR 601.2g's window offers a player, read once a window
/// (§3.8). Between its prompts only whether each one's costs can be paid
/// changes, until the layer epoch moves (a sacrifice), when it is read again.
pub struct ManaAbilityWindowOffer {
    player: PlayerId,
    epoch: u64,
    abilities: Vec<OfferedManaAbility>,
}

/// A mana ability the window may offer: its permanent and the first instance
/// of its definition there, since two grants of one ability are one choice.
struct OfferedManaAbility {
    permanent: ObjectId,
    ability: AbilityId,
    paid: CostRecheck,
}

/// How the window re-asks whether an ability's costs can be paid.
enum CostRecheck {
    /// {T} alone: while the permanent is untapped, since whether it is
    /// summoning-sick (CR 302.6) moves only with the epoch.
    WhileUntapped { sick: bool },
    /// Anything else, asked whole each time.
    Costs(Vec<Cost>),
}

impl ManaAbilityWindowOffer {
    pub fn read(game: &GameState, player: PlayerId) -> ManaAbilityWindowOffer {
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
                    [Cost::TapSelf] => CostRecheck::WhileUntapped { sick: has_summoning_sickness(game, permanent) },
                    costs => CostRecheck::Costs(costs.to_vec()),
                };
                abilities.push(OfferedManaAbility { permanent, ability: ability.id, paid });
            }
        }
        ManaAbilityWindowOffer { player, epoch: game.layer_epoch(), abilities }
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
                CostRecheck::WhileUntapped { sick } => {
                    !sick && game.battlefield.get(&offered.permanent).is_some_and(|entry| !entry.tapped)
                }
                CostRecheck::Costs(costs) => game.can_pay_costs(costs, self.player, offered.permanent).is_ok(),
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
        || match mana_ability_cost_of(ability) {
            ManaAbilityCost::MultipliesThePool { .. } => true,
            ManaAbilityCost::SpendsItsTapOrItself { .. } | ManaAbilityCost::SacrificesOthers { .. } => {
                mana_production_of(game, &ability.effect, permanent, player).is_some()
            }
            ManaAbilityCost::NotCounted => false,
        }
}

/// What a payment's costs besides its mana reserve for themselves: a {T} its
/// source's tap, a sacrifice its permanents.
#[derive(Default)]
struct ReservedByCosts {
    tapped: Option<ObjectId>,
    sacrificed: Option<ObjectId>,
    /// Each other sacrifice: the permanents it may take, and how many.
    sacrifices: Vec<(Vec<ObjectId>, u32)>,
}

impl ReservedByCosts {
    /// `fodder_read` is whether anything reads a sacrifice's candidates:
    /// without an outlet nothing does, and they are not looked up.
    fn of(game: &GameState, player: PlayerId, non_mana: &NonManaCosts<'_>, fodder_read: bool) -> ReservedByCosts {
        let mut reserved = ReservedByCosts { tapped: None, sacrificed: None, sacrifices: Vec::new() };
        for cost in non_mana.costs {
            match cost {
                Cost::TapSelf => reserved.tapped = non_mana.source,
                Cost::SacrificeSelf => reserved.sacrificed = non_mana.source,
                Cost::Sacrifice(filter, needs) if fodder_read => {
                    let candidates = permanents_of(game, player)
                        .into_iter()
                        .filter(|&id| game.object_matches_filter(id, filter, player).unwrap_or(false))
                        .collect();
                    reserved.sacrifices.push((candidates, *needs));
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
        reserved
    }

    fn allows(&self, permanent: Option<ObjectId>, way: &EntryWay) -> bool {
        let Some(permanent) = permanent else { return true };
        !(way.taps && self.tapped == Some(permanent)) && !(way.sacrifices && self.sacrificed == Some(permanent))
    }

    /// Whether the payment taps or sacrifices `permanent` itself.
    fn reserves(&self, permanent: ObjectId) -> bool {
        self.tapped == Some(permanent) || self.sacrificed == Some(permanent)
    }

    fn reserves_no_fodder(&self) -> bool {
        self.sacrificed.is_none() && self.sacrifices.is_empty()
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
enum ManaAbilityCost<'a> {
    /// {T}, a sacrifice of itself, or both: once.
    SpendsItsTapOrItself { taps: bool, sacrifices: bool },
    /// "Sacrifice a [filter]": once for each permanent it can sacrifice.
    SacrificesOthers { filter: &'a ObjectFilter, needs: u32 },
    MultipliesThePool { input: &'a ManaCost },
    /// A cost this phase does not read: a converter fed by other mana
    /// (MA-3), life, counters, an untap, or nothing at all.
    NotCounted,
}

fn mana_ability_cost_of(ability: &AbilityDef) -> ManaAbilityCost<'_> {
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
        (None, None, false) if taps || sacrifices => ManaAbilityCost::SpendsItsTapOrItself { taps, sacrifices },
        (Some((filter, needs)), None, false) if !taps && !sacrifices => ManaAbilityCost::SacrificesOthers { filter, needs },
        (None, Some(input), false) if taps && !sacrifices && multiplies_the_pool(&ability.effect) => {
            ManaAbilityCost::MultipliesThePool { input }
        }
        _ => ManaAbilityCost::NotCounted,
    }
}

/// Doubling Cube's effect: each type's unspent mana, added again.
fn multiplies_the_pool(effect: &Effect) -> bool {
    let Effect::Atom(Primitive::ProduceMana(output), _) = effect else { return false };
    output.special.is_empty()
        && output.mana.len() == ALL_MANA_TYPES.len()
        && ALL_MANA_TYPES
            .iter()
            .all(|t| output.mana.iter().any(|(made, amount)| made == t && *amount == AmountExpr::UnspentMana(*t)))
}

/// What a mana ability's effect adds for `player`, before any replacement
/// effect: `None` when it adds nothing a payment can spend now.
fn mana_production_of(game: &GameState, effect: &Effect, source: ObjectId, player: PlayerId) -> Option<ManaBag> {
    let mut resolution = ResolutionContext::untargeted(source, player);
    resolution.ability_source = game.object_ref(source);
    let mut made = NO_MANA;
    add_mana_produced(game, effect, &resolution, &mut made)?;
    made.iter().any(|&n| n > 0).then_some(made)
}

fn add_mana_produced(game: &GameState, effect: &Effect, resolution: &ResolutionContext, made: &mut ManaBag) -> Option<()> {
    match effect {
        Effect::Atom(Primitive::ProduceMana(output), _) => {
            for (mana_type, amount) in &output.mana {
                // The pool at resolution, after the window's other activations:
                // not a number the inventory can read now.
                if matches!(amount, AmountExpr::UnspentMana(_)) {
                    return None;
                }
                made[slot(*mana_type)] += game.evaluate_amount(amount, resolution).ok()?;
            }
            // `output.special` is restricted mana, which no payment spends until
            // item 33's pass, so it is not counted (§3.6).
            Some(())
        }
        Effect::Sequence(effects) => effects.iter().try_for_each(|effect| add_mana_produced(game, effect, resolution, made)),
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
    // With no effect changing control, a permanent's controller is its
    // entry's, the base `controls` reads without a layer walk.
    let mut mine: Vec<(Timestamp, ObjectId)> = game
        .battlefield
        .iter()
        .filter(|&(&id, entry)| {
            debug_assert_eq!(entry.controller == player, controls(game, id, player));
            entry.controller == player
        })
        .map(|(&id, entry)| (entry.timestamp, id))
        .collect();
    mine.sort_unstable_by_key(|&(timestamp, _)| timestamp);
    mine.into_iter().map(|(_, id)| id).collect()
}

/// An ability, by its permanent and its place on that permanent's effective
/// list, which holds while the layer epoch it was read at does.
#[derive(Debug, Clone, Copy, PartialEq)]
struct AbilityAt {
    permanent: ObjectId,
    index: usize,
}

/// The abilities on the battlefield that change what a production makes:
/// static replacement abilities watching one (CR 106.6a, 106.12b) and
/// triggered mana abilities (CR 605.1b), read off effective ability lists as
/// the gather and the dispatcher read them (§3.4).
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ManaProductionWatchers {
    replacements: Vec<AbilityAt>,
    triggers: Vec<AbilityAt>,
}

/// The board's watchers, from the memo when the layer epoch has not moved.
///
/// Every input is a layer-walk input (effective abilities, controllers,
/// attachments), so the frame memo's epoch argument covers this cache too
/// (`layers-architecture.md` §12, "7a"). A row's "as long as" and the
/// registry's resolution-made effects are not, and are read live.
fn mana_production_watchers(game: &GameState) -> Arc<ManaProductionWatchers> {
    let epoch = game.layer_epoch();
    if let Some(found) = game.layer_memo.mana_production_watchers(epoch) {
        #[cfg(debug_assertions)]
        if game.layer_memo.audited() {
            audit_mana_production_watchers(game, &found);
        }
        return found;
    }
    let found = Arc::new(scan_mana_production_watchers(game));
    game.layer_memo.insert_mana_production_watchers(epoch, Arc::clone(&found));
    found
}

fn scan_mana_production_watchers(game: &GameState) -> ManaProductionWatchers {
    let mut found = ManaProductionWatchers::default();
    for permanent in game.battlefield_ids_ordered() {
        let Some(chars) = compute_characteristics(game, permanent) else { continue };
        for (index, ability) in chars.abilities.iter().enumerate() {
            let at = AbilityAt { permanent, index };
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
fn audit_mana_production_watchers(game: &GameState, served: &ManaProductionWatchers) {
    let counts = game.diagnostics.clone();
    let fresh = scan_mana_production_watchers(game);
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
fn mana_made_by(
    game: &GameState,
    watchers: &ManaProductionWatchers,
    player: PlayerId,
    producer: ObjectId,
    tapped: bool,
    base: ManaBag,
) -> Vec<ManaBag> {
    let mana_after_replacements = mana_after_replacements(game, watchers, player, producer, tapped, base);
    if watchers.triggers.is_empty() {
        return mana_after_replacements;
    }
    let mut out: Vec<ManaBag> = Vec::new();
    for made in mana_after_replacements {
        let mut ways = vec![made];
        for &at in &watchers.triggers {
            let added = mana_trigger_adds(game, watchers, at, player, producer, tapped, &made);
            if !added.is_empty() {
                ways = ways.iter().flat_map(|way| added.iter().map(move |a| sum(way, a))).collect();
            }
        }
        out.extend(ways);
    }
    undominated_mana_bags(out)
}

/// What a production can become under the replacement effects that apply to
/// it, the board's and the registry's.
fn mana_after_replacements(
    game: &GameState,
    watchers: &ManaProductionWatchers,
    player: PlayerId,
    producer: ObjectId,
    tapped: bool,
    base: ManaBag,
) -> Vec<ManaBag> {
    let mut rewrites: Vec<ManaRewrite> = Vec::new();
    for &at in &watchers.replacements {
        let abilities = get_effective_abilities(game, at.permanent);
        let Some((def, condition)) = abilities.get(at.index).and_then(replacement_of) else { continue };
        if condition.is_some_and(|condition| !settled_holds(condition, game, at.permanent, None)) {
            continue;
        }
        let controller = controller_or_owner(game, at.permanent).unwrap_or(0);
        if applies_to_mana_production(game, def, at.permanent, controller, player, producer, tapped) {
            rewrites.push(ManaRewrite::of(def));
        }
    }
    for row in game.replacement_effects.iter() {
        if applies_to_mana_production(game, &row.def, row.source, row.controller, player, producer, tapped) {
            rewrites.push(ManaRewrite::of(&row.def));
        }
    }
    if rewrites.is_empty() {
        return vec![base];
    }
    fold_mana_rewrites(base, &rewrites)
}

/// What the triggered mana ability at `at` adds to `player`'s pool when a
/// production by `producer` that made `made` sets it off: nothing when no arm
/// matches the production (the dispatcher's predicates), its intervening "if"
/// fails (CR 603.4), or its mana is another player's.
fn mana_trigger_adds(
    game: &GameState,
    watchers: &ManaProductionWatchers,
    at: AbilityAt,
    player: PlayerId,
    producer: ObjectId,
    tapped: bool,
    made: &ManaBag,
) -> Vec<ManaBag> {
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
    mana_after_replacements(game, watchers, player, at.permanent, false, adds)
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
    adds: &mut ManaBag,
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
struct ManaRewrite {
    kind: ManaRewriteKind,
    /// "You may": declining is the player's (CR 614.5's one opportunity).
    optional: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ManaRewriteKind {
    Multiply(u64),
    Retype(usize),
    SetTo(usize, u64),
    /// No production: prevented, replaced by another event, or a pairing the
    /// pipeline refuses, which fails the activation.
    Nothing,
}

impl ManaRewrite {
    fn of(def: &ReplacementDef) -> ManaRewrite {
        let kind = match &def.rewrite {
            Rewrite::Amount(AmountRewrite::Multiplier(n)) => ManaRewriteKind::Multiply(*n),
            Rewrite::Instead(GameActionTemplate::ProduceMana { mana_type, amount }) => match amount {
                TemplateAmount::ReplacedAmount => ManaRewriteKind::Retype(slot(*mana_type)),
                TemplateAmount::Fixed(n) => ManaRewriteKind::SetTo(slot(*mana_type), *n),
            },
            Rewrite::Prevent
            | Rewrite::Instead(_)
            | Rewrite::Amount(_)
            | Rewrite::EnterWith(_)
            | Rewrite::EnterAfterMoving(_)
            | Rewrite::EnterUnderControlOf(_)
            | Rewrite::EnterAsCopy(_)
            | Rewrite::Retarget(_) => ManaRewriteKind::Nothing,
        };
        ManaRewrite { kind, optional: def.optional }
    }
}

impl ManaRewriteKind {
    fn apply(self, made: ManaBag) -> Option<ManaBag> {
        match self {
            ManaRewriteKind::Multiply(n) => Some(made.map(|m| m.saturating_mul(n))),
            ManaRewriteKind::Retype(t) => {
                let mut out = NO_MANA;
                out[t] = made.iter().sum();
                Some(out)
            }
            ManaRewriteKind::SetTo(t, n) => {
                let mut out = NO_MANA;
                out[t] = n;
                Some(out)
            }
            ManaRewriteKind::Nothing => None,
        }
    }
}

/// Past this many effects on one production, orders are not tried one by
/// one; the check leans yes instead (§3.3).
const MAX_ORDERED_MANA_REWRITES: usize = 6;

/// Every production `rewrites` can make of `base`: in every order its player
/// may choose (CR 616.1), each applied once (CR 614.5) and an optional one
/// declinable, less the dominated.
fn fold_mana_rewrites(base: ManaBag, rewrites: &[ManaRewrite]) -> Vec<ManaBag> {
    if rewrites.len() > MAX_ORDERED_MANA_REWRITES {
        return most_mana_of_every_type(base, rewrites);
    }
    let mut out = Vec::new();
    apply_mana_rewrites_in_every_order(base, rewrites, 0, &mut out);
    undominated_mana_bags(out)
}

fn apply_mana_rewrites_in_every_order(made: ManaBag, rewrites: &[ManaRewrite], applied: u32, out: &mut Vec<ManaBag>) {
    let mut last = true;
    for (i, rewrite) in rewrites.iter().enumerate() {
        if applied & (1 << i) != 0 {
            continue;
        }
        last = false;
        match rewrite.kind.apply(made) {
            Some(next) => apply_mana_rewrites_in_every_order(next, rewrites, applied | 1 << i, out),
            // Nothing after it applies to a production.
            None => out.push(NO_MANA),
        }
        if rewrite.optional {
            apply_mana_rewrites_in_every_order(made, rewrites, applied | 1 << i, out);
        }
    }
    if last {
        out.push(made);
    }
}

/// An outcome no order can beat, of each type an order could leave: the
/// most mana any order makes.
fn most_mana_of_every_type(base: ManaBag, rewrites: &[ManaRewrite]) -> Vec<ManaBag> {
    let mut most: u64 = base.iter().sum();
    let mut types = ManaTypes::of(&base);
    for rewrite in rewrites {
        match rewrite.kind {
            ManaRewriteKind::SetTo(t, n) => {
                most = most.max(n);
                types = types.union(ManaTypes::one(t));
            }
            ManaRewriteKind::Retype(t) => types = types.union(ManaTypes::one(t)),
            ManaRewriteKind::Multiply(_) | ManaRewriteKind::Nothing => {}
        }
    }
    for rewrite in rewrites {
        if let ManaRewriteKind::Multiply(n) = rewrite.kind {
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

fn undominated_mana_bags(bags: Vec<ManaBag>) -> Vec<ManaBag> {
    let mut kept: Vec<ManaBag> = Vec::new();
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
struct ManaDemand {
    pips: ManaBag,
    generic: u64,
}

impl ManaDemand {
    /// `None` for a symbol no payment path pays yet: hybrid and its kin are
    /// CP-1's, {S} item 33's, {X} MA-2's.
    fn of(cost: &ManaCost) -> Option<ManaDemand> {
        let mut demand = ManaDemand::default();
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

    fn kinds(&self) -> ManaTypes {
        ManaTypes::of(&self.pips)
    }

    /// The demand left once `made` has paid what it can: a mana pays a pip
    /// of its own type first, since generic takes any.
    fn less(&self, made: &ManaBag) -> ManaDemand {
        let mut left = *self;
        let mut spare = 0;
        for t in 0..6 {
            let paid = made[t].min(left.pips[t]);
            left.pips[t] -= paid;
            spare += made[t] - paid;
        }
        left.generic = left.generic.saturating_sub(spare);
        left
    }

    /// What must be made before a pool multiplier for this demand to be met
    /// after it.
    ///
    /// Every mana left in the pool becomes `factor` of its own type, so a pip
    /// of a type needs `ceil(n / factor)` of that type left, and the whole
    /// demand `ceil(total / factor)` left of any; the multiplier's input is paid
    /// first, from what was made (§3.3). One Hall check at the end is then
    /// exact, with no enumeration of how the input was paid.
    fn before_multiplying(&self, factor: u64, input: &ManaDemand) -> ManaDemand {
        let pips: ManaBag = std::array::from_fn(|t| self.pips[t].div_ceil(factor) + input.pips[t]);
        let per_type: u64 = self.pips.iter().map(|n| n.div_ceil(factor)).sum();
        let needed = per_type.max(self.total().div_ceil(factor)) + input.total();
        ManaDemand { pips, generic: needed - pips.iter().sum::<u64>() }
    }
}

/// The supply one payment leaves, split by how each mana's type is chosen.
#[derive(Debug, Clone, Default)]
struct SplitSupply {
    /// Mana that is its own choice of type, by the set it can be: §3.2's
    /// table.
    free: Vec<(ManaTypes, u64)>,
    /// Entries with ways of more than one mana, several of one chosen type or
    /// ways of different sizes, with how many such entries.
    choices: Vec<(Vec<ManaBag>, u64)>,
}

/// Past this many leaves the check stops trying and leans yes: an
/// over-offer costs a rewind, an under-offer hides a legal play (§3.3).
const MAX_CHOICE_LEAVES: usize = 4096;

impl SplitSupply {
    fn add(&mut self, ways: &[ManaBag], copies: u64) {
        if copies == 0 || ways.is_empty() {
            return;
        }
        if let [made] = ways {
            for t in (0..6).filter(|&t| made[t] > 0) {
                self.add_free(ManaTypes::one(t), made[t] * copies);
            }
            return;
        }
        // One mana of any type in a set: a dual, Everywhere.
        if ways.iter().all(|way| way.iter().sum::<u64>() == 1) {
            let types = ways.iter().fold(ManaTypes::default(), |set, way| set.union(ManaTypes::of(way)));
            self.add_free(types, copies);
            return;
        }
        match self.choices.iter_mut().find(|(known, _)| known.as_slice() == ways) {
            Some((_, n)) => *n += copies,
            None => self.choices.push((ways.to_vec(), copies)),
        }
    }

    fn add_free(&mut self, types: ManaTypes, n: u64) {
        match self.free.iter_mut().find(|(known, _)| *known == types) {
            Some((_, have)) => *have += n,
            None => self.free.push((types, n)),
        }
    }

    fn pays(&self, demand: &ManaDemand) -> bool {
        let asked = demand.kinds();
        let groups: Vec<(Vec<ManaBag>, u64)> =
            self.choices.iter().map(|(ways, copies)| (views(ways, asked), *copies)).collect();
        let first_copies = groups.first().map_or(0, |(_, copies)| *copies);
        ChoiceSearch { pieces: self, groups: &groups, demand, leaves: 0 }.from(0, 0, first_copies, NO_MANA)
    }

    /// Gale's condition (Hall's, for b-matchings): every set of the asked pip
    /// kinds asks for no more mana than the supply that can pay one of them,
    /// and the whole demand for no more than all of it (§3.2).
    fn feasible(&self, extra: &ManaBag, demand: &ManaDemand) -> bool {
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
struct ChoiceSearch<'a> {
    pieces: &'a SplitSupply,
    groups: &'a [(Vec<ManaBag>, u64)],
    demand: &'a ManaDemand,
    leaves: usize,
}

impl ChoiceSearch<'_> {
    /// From `group`'s `way` on, with `left` of its copies still to place and
    /// `extra` the mana the choices so far make.
    fn from(&mut self, group: usize, way: usize, left: u64, extra: ManaBag) -> bool {
        let groups = self.groups;
        let Some((ways, _)) = groups.get(group) else {
            self.leaves += 1;
            return self.pieces.feasible(&extra, self.demand);
        };
        if self.leaves >= MAX_CHOICE_LEAVES {
            return true;
        }
        if way + 1 == ways.len() {
            let extra = std::array::from_fn(|t| extra[t] + ways[way][t] * left);
            let next_copies = groups.get(group + 1).map_or(0, |(_, copies)| *copies);
            return self.from(group + 1, 0, next_copies, extra);
        }
        let most = useful_copies(ways, way, self.demand).min(left);
        (0..=most).rev().any(|n| {
            let extra = std::array::from_fn(|t| extra[t] + ways[way][t] * n);
            self.from(group, way + 1, left - n, extra)
        })
    }
}

/// How many copies of `ways[way]` can help pay `demand` beyond what the
/// group's last way gives, which takes the copies left: when every way makes
/// as much (one choice of type, as Everywheres under Mana Reflection are),
/// enough of one asked type to pay its pips, since a copy past them pays only
/// generic, as a copy of the last way does. Twenty such Everywheres are then
/// a few choices a cost, not thousands. Ways of different sizes are every
/// count, since a bigger way's copies pay more generic than the last's.
fn useful_copies(ways: &[ManaBag], way: usize, demand: &ManaDemand) -> u64 {
    let size = |view: &ManaBag| view.iter().sum::<u64>();
    let view = &ways[way];
    match (0..6).find(|&t| view[t] > 0) {
        Some(t)
            if ManaTypes::of(view) == ManaTypes::one(t)
                && demand.pips[t] > 0
                && ways.iter().all(|other| size(other) == size(view)) =>
        {
            demand.pips[t].div_ceil(view[t])
        }
        _ => u64::MAX,
    }

}

/// `ways` as a cost of `asked` kinds tells them apart: each asked type's mana,
/// and every other type's in one unasked slot, since any of it pays only
/// generic. Equal views are kept once and dominated ones dropped.
fn views(ways: &[ManaBag], asked: ManaTypes) -> Vec<ManaBag> {
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
    undominated_mana_bags(seen.collect())
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
        ManaSupply::read(game, 0).covers(game, cost, &NonManaCosts { source: None, costs: &[] })
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
        let ability = NonManaCosts { source: Some(breaker), costs: &other };
        let supply = ManaSupply::read(&game, 0);
        assert!(supply.covers(&game, &cost(&[], 3), &NonManaCosts { source: None, costs: &[] }), "a spell may tap it");
        assert!(!supply.covers(&game, &cost(&[], 3), &ability));
        put_on_battlefield(&mut game, plains(), 0);
        assert!(ManaSupply::read(&game, 0).covers(&game, &cost(&[], 3), &ability));
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
            let ability = NonManaCosts { source: Some(stone), costs: &other };
            ManaSupply::read(game, 0).covers(game, &cost(&[], generic), &ability)
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
        let spell = NonManaCosts { source: None, costs: &sacrifice };
        let supply = ManaSupply::read(&game, 0);
        assert!(supply.covers(&game, &cost(&[], 4), &NonManaCosts { source: None, costs: &[] }));
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

    /// Ironworks can sacrifice the Cube only after the Cube has doubled, and
    /// itself after that, so their {C}{C}s are not doubled: eight Plains less
    /// three, doubled, and four colorless, fourteen and not more.
    #[test]
    fn what_ironworks_sacrifices_after_the_doubling_is_not_doubled() {
        let mut game = setup_two_player_game();
        for _ in 0..8 {
            put_on_battlefield(&mut game, plains(), 0);
        }
        put_on_battlefield(&mut game, krark_clan_ironworks(), 0);
        put_on_battlefield(&mut game, doubling_cube(), 0);
        assert!(covers(&game, &cost(&[], 14)));
        assert!(!covers(&game, &cost(&[], 15)), "the Cube cannot feed Ironworks before it doubles");
        assert!(covers(&game, &cost(&[White; 10], 4)), "ten white from the doubling, four colorless after");
    }

    /// Ways of different sizes keep every count: two copies of {W}{W}{W}-or-{U}
    /// pay {W}{5} only both as white, which a count capped at what the white
    /// pip needs would never try.
    #[test]
    fn copies_of_ways_of_different_sizes_are_counted_every_way() {
        let entry = |_| SupplyEntry {
            permanent: Some(new_object_id()),
            ways: vec![
                EntryWay { makes: [3, 0, 0, 0, 0, 0], taps: true, sacrifices: false },
                EntryWay { makes: [0, 1, 0, 0, 0, 0], taps: true, sacrifices: false },
            ],
        };
        let supply = ManaSupply {
            player: 0,
            entries: (0..2).map(entry).collect(),
            sacrifice_outlets: Vec::new(),
            pool_multipliers: Vec::new(),
            plain_split: OnceCell::new(),
        };
        let demand = ManaDemand { pips: [1, 0, 0, 0, 0, 0], generic: 5 };
        assert!(supply.pays(&demand, &ReservedByCosts::default()));
        assert!(pays_by_search(&supply, &demand));
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

    /// The watcher scan is read once a layer epoch, for every player's
    /// inventory, and again once the board moves.
    #[test]
    fn the_watcher_scan_is_taken_once_an_epoch() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, mana_reflection(), 0);
        ManaSupply::read(&game, 0);
        let epoch = game.layer_epoch();
        let first = game.layer_memo.mana_production_watchers(epoch).unwrap();
        ManaSupply::read(&game, 1);
        assert!(Arc::ptr_eq(&first, &game.layer_memo.mana_production_watchers(epoch).unwrap()), "one scan");
        put_on_battlefield(&mut game, plains(), 0);
        assert!(game.layer_memo.mana_production_watchers(game.layer_epoch()).is_none(), "the board moved");
        ManaSupply::read(&game, 0);
        assert!(game.layer_memo.mana_production_watchers(game.layer_epoch()).is_some());
    }

    /// The memo's debug mode: a write that changes what the scan finds and
    /// skips its bump is caught on the next hit rather than served.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "production-watcher memo served a stale scan")]
    fn a_skipped_bump_is_caught_by_the_watcher_audit() {
        let mut game = setup_two_player_game();
        let reflection = put_on_battlefield(&mut game, mana_reflection(), 0);
        ManaSupply::read(&game, 0);
        game.battlefield.remove(&reflection);
        ManaSupply::read(&game, 0);
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
        let offer = ManaAbilityWindowOffer::read(&game, 0);
        let options = offer.options(&game);
        assert_eq!(options.len(), 7, "a Plains, Everywhere's five, Ironworks; not a creature this turn's");
        assert_eq!(options[0].0, land, "timestamp order");

        game.battlefield.get_mut(&land).unwrap().tapped = true;
        assert!(offer.is_current(&game), "a tap reads no layer");
        assert_eq!(offer.options(&game).len(), 6);

        game.change_zone(ironworks, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx()).unwrap();
        assert!(!offer.is_current(&game));
        assert_eq!(ManaAbilityWindowOffer::read(&game, 0).options(&game).len(), 5);
    }

    /// Two retypes on one production leave either type, in its player's
    /// order (CR 616.1); a multiplier commutes with both.
    #[test]
    fn a_production_is_rewritten_in_every_order() {
        let rewrite = |kind| ManaRewrite { kind, optional: false };
        let rewrites = [
            rewrite(ManaRewriteKind::Retype(slot(Blue))),
            rewrite(ManaRewriteKind::Multiply(2)),
            rewrite(ManaRewriteKind::Retype(slot(Colorless))),
        ];
        let mut green = NO_MANA;
        green[slot(Green)] = 1;
        let mut made = fold_mana_rewrites(green, &rewrites);
        made.sort();
        assert_eq!(made, vec![[0, 0, 0, 0, 0, 2], [0, 2, 0, 0, 0, 0]]);

        let declinable = ManaRewrite { optional: true, ..rewrites[0] };
        assert_eq!(fold_mana_rewrites(green, &[declinable]).len(), 2, "applied, or declined");
        let past_the_cap = vec![rewrite(ManaRewriteKind::Multiply(2)); MAX_ORDERED_MANA_REWRITES + 1];
        assert_eq!(fold_mana_rewrites(green, &past_the_cap), vec![[0, 0, 0, 0, 128, 0]]);
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
            let watchers = mana_production_watchers(game);
            for id in permanents_of(game, 0) {
                for ability in get_effective_abilities(game, id).iter().filter(|a| a.ability_type == AbilityType::Mana) {
                    let taps = match mana_ability_cost_of(ability) {
                        ManaAbilityCost::SpendsItsTapOrItself { taps, .. } => taps,
                        ManaAbilityCost::SacrificesOthers { .. } => false,
                        ManaAbilityCost::MultipliesThePool { .. } | ManaAbilityCost::NotCounted => continue,
                    };
                    let base = mana_production_of(game, &ability.effect, id, 0).unwrap();
                    let mut predicted = mana_made_by(game, &watchers, 0, id, taps, base);
                    let mut made: Vec<ManaBag> = Vec::new();
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

    /// Every way of using every entry, then each pool multiplier in turn with its
    /// input paid every way it can be: the answer the check must agree with.
    fn pays_by_search(supply: &ManaSupply, demand: &ManaDemand) -> bool {
        let mut made = vec![NO_MANA];
        for entry in &supply.entries {
            made = made.iter().flat_map(|m| entry.ways.iter().map(move |w| sum(m, &w.makes))).collect();
        }
        made.iter().any(|pool| pays_after(*pool, &supply.pool_multipliers, demand))
    }

    fn pays_after(pool: ManaBag, pool_multipliers: &[PoolMultiplier], demand: &ManaDemand) -> bool {
        if (0..6).all(|t| pool[t] >= demand.pips[t]) && pool.iter().sum::<u64>() >= demand.total() {
            return true;
        }
        let Some((multiplier, rest)) = pool_multipliers.split_first() else { return false };
        let pips_paid = (0..6).try_fold(pool, |mut left, t| {
            left[t] = left[t].checked_sub(multiplier.input.pips[t])?;
            Some(left)
        });
        let Some(pips_paid) = pips_paid else { return false };
        spend(pips_paid, 0, multiplier.input.generic)
            .into_iter()
            .any(|left| pays_after(left.map(|n| n * multiplier.factor), rest, demand))
    }

    /// Every pool left by spending `n` generic from `pool`'s types `t` on.
    fn spend(pool: ManaBag, t: usize, n: u64) -> Vec<ManaBag> {
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

    fn one(t: usize, n: u64) -> ManaBag {
        let mut made = NO_MANA;
        made[t] = n;
        made
    }

    /// A random entry of one of §3.3's shapes: one mana of a set of types,
    /// several of one chosen type, or ways of different sizes.
    fn random_entry(rng: &mut StdRng) -> SupplyEntry {
        let ways = rng.random_range(1..=3);
        let makes: Vec<ManaBag> = match rng.random_range(0..3) {
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
        let ways = makes.into_iter().map(|makes| EntryWay { makes, taps: false, sacrifices: false }).collect();
        SupplyEntry { permanent: Some(new_object_id()), ways: undominated(ways) }
    }

    /// The property MA-1 rests on: on small random boards, Hall's condition
    /// with §3.3's shapes answers what trying every payment answers.
    #[test]
    fn the_check_agrees_with_an_exhaustive_search() {
        let mut rng = StdRng::seed_from_u64(162);
        let (mut yes, mut no) = (0, 0);
        for _ in 0..3000 {
            let mut entries: Vec<SupplyEntry> = (0..rng.random_range(0..=5)).map(|_| random_entry(&mut rng)).collect();
            // Copies of one entry, which the check counts as a group.
            if let Some(copied) = entries.first().cloned().filter(|_| rng.random_bool(0.4)) {
                for _ in 0..rng.random_range(1..=2) {
                    entries.push(SupplyEntry { permanent: Some(new_object_id()), ..copied.clone() });
                }
            }
            let pool_multipliers = (0..rng.random_range(0..=2))
                .map(|_| {
                    let mut input = ManaDemand { pips: NO_MANA, generic: rng.random_range(1..=3) };
                    if rng.random_bool(0.3) {
                        input.pips[rng.random_range(0..6)] = 1;
                    }
                    PoolMultiplier { permanent: new_object_id(), input, factor: rng.random_range(2..=3) }
                })
                .collect();
            let supply = ManaSupply {
                player: 0,
                entries,
                sacrifice_outlets: Vec::new(),
                pool_multipliers,
                plain_split: OnceCell::new(),
            };
            let mut demand = ManaDemand { pips: NO_MANA, generic: rng.random_range(0..=6) };
            for _ in 0..rng.random_range(0..=3) {
                demand.pips[rng.random_range(0..6)] += rng.random_range(1..=3);
            }
            let expected = pays_by_search(&supply, &demand);
            assert_eq!(supply.pays(&demand, &ReservedByCosts::default()), expected, "{supply:#?}\n{demand:?}");
            if expected { yes += 1 } else { no += 1 }
        }
        assert!(yes > 500 && no > 500, "both answers, often: {yes} yes, {no} no");
    }
}
