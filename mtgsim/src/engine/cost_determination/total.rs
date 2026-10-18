//! CR 601.2f — determining the total cost, in the rule's order.
//!
//! > The total cost is the mana cost or alternative cost (as determined in
//! > rule 601.2b), plus all additional costs and cost increases, and minus
//! > all cost reductions. If multiple cost reductions apply, the player may
//! > apply them in any order. If the mana component of the total cost is
//! > reduced to nothing by cost reduction effects, it is considered to be
//! > {0}. It can't be reduced to less than {0}. Once the total cost is
//! > determined, any effects that directly affect the total cost are applied.
//! > Then the resulting total cost becomes "locked in."
//!
//! One entry point, [`determine_total_cost`], which is the whole of the step
//! (`cost-architecture.md` §3.3): assemble what CR 601.2b chose — the base or
//! alternative cost with its X, then the additional costs — merge every
//! mana cost into the one mana component, gather the cost effects that apply,
//! add the increases, subtract the reductions in the player's order under
//! CR 118.7a–d, apply the direct-total effects, and lock — which is that the
//! value returned is what CR 601.2h pays and nothing re-reads the board in
//! between. Payment stays in `engine::costs`.

use super::gather::{cost_modifications_for, CostModificationInstance};
use crate::engine::layers::compute::settled_amount;
use crate::state::game_state::GameState;
use crate::types::cost_modification::CostChange;
use crate::types::costs::{AdditionalCost, AlternativeCost, Cost};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::ui::ask::ask_order_cost_reductions;
use crate::ui::decision::DecisionProvider;

/// CR 601.2f, whole: from the choices CR 601.2b recorded to the locked total.
///
/// `spell` is the object whose cost this is, on the stack since CR 601.2a;
/// `caster` is who answers the reduction-order prompt, through `dp`, when two
/// or more reductions apply (CR 601.2f, "in any order"); one asks nothing.
/// The non-mana costs come back in their order behind **one** mana component
/// — CR 601.2f's "the mana component of the total cost" is singular and
/// CR 118.8 pays additional costs "at the same time".
pub fn determine_total_cost(
    game: &GameState,
    caster: PlayerId,
    spell: ObjectId,
    base_mana_cost: &ManaCost,
    chosen_alt_cost: Option<&AlternativeCost>,
    chosen_additional_costs: &[&AdditionalCost],
    x_value: u64,
    dp: &dyn DecisionProvider,
) -> Vec<Cost> {
    let mut total = assemble_total_cost(base_costs(base_mana_cost, chosen_alt_cost), chosen_additional_costs);
    let instances = cost_modifications_for(game, spell);
    modify(game, &mut total.mana, &instances, |sources| ask_order_cost_reductions(dp, game, caster, spell, sources));
    // Step 6 — locked in: this value is paid, and nothing re-reads the board.
    total.locked_at(x_value)
}

/// Step 1 — the base. An alternative cost replaces the mana cost entirely
/// (CR 118.9a).
fn base_costs(base_mana_cost: &ManaCost, chosen_alt_cost: Option<&AlternativeCost>) -> Vec<Cost> {
    match chosen_alt_cost {
        Some(alt) => alt.costs().to_vec(),
        None => vec![Cost::Mana(base_mana_cost.clone())],
    }
}

/// Steps 1 and 2 — the base and the additional costs the player chose
/// (CR 118.8), whichever base was chosen: CR 118.9d applies them to an
/// alternative cost too. Every mana cost among them is one component, and the
/// other costs keep their order. The base's X is the announced X, in an
/// alternative cost as in the mana cost (CR 107.3a); an X in an additional
/// cost stays a symbol, which no payment pays yet.
fn assemble_total_cost(base: Vec<Cost>, additional: &[&AdditionalCost]) -> TotalCostInX {
    let mut total = TotalCostInX { mana: ManaComponent::EMPTY, non_mana: Vec::new(), has_mana_cost: false };
    for cost in base {
        match cost {
            Cost::Mana(base) => {
                total.mana.add_base(&base);
                total.has_mana_cost = true;
            }
            other => total.non_mana.push(other),
        }
    }
    for cost in additional.iter().flat_map(|additional| additional.costs()) {
        match cost {
            Cost::Mana(more) => {
                total.mana.add(more);
                total.has_mana_cost = true;
            }
            other => total.non_mana.push(other.clone()),
        }
    }
    total
}

/// Steps 3–5: the modifications, the reductions in `order`'s.
fn modify(
    game: &GameState,
    mana: &mut ManaComponent,
    instances: &[CostModificationInstance],
    order: impl FnOnce(&[ObjectId]) -> Vec<usize>,
) {
    // Step 3 — increases. Addition commutes; the gather order is for the log.
    for inst in instances {
        if let CostChange::Increase(amount) = &inst.def.change {
            mana.increase(amount);
        }
    }

    // Step 4 — reductions, in the order the player chooses. With the symbols
    // the engine pays today the order never changes the answer (§3.4's
    // theorem), and the prompt is asked anyway because CR 601.2f makes it
    // the player's; the two conditions under which it would matter — a
    // hybrid reduction symbol, a floored reduction — are refused below and
    // not yet representable, respectively. A dynamic amount does not add a
    // third: nothing in this step touches the board, so every `ReduceGeneric`
    // reads the same board whenever it is applied.
    let reductions: Vec<&CostModificationInstance> = instances.iter().filter(|inst| is_reduction(inst)).collect();
    let order: Vec<usize> = if reductions.len() >= 2 {
        let sources: Vec<ObjectId> = reductions.iter().map(|inst| inst.source).collect();
        order(&sources)
    } else {
        (0..reductions.len()).collect()
    };
    for idx in order {
        apply_one_reduction(game, mana, reductions[idx]);
    }

    // Step 5 — "any effects that directly affect the total cost". Two
    // Trinispheres agree, so no order is chosen.
    for inst in instances {
        if let CostChange::TotalAtLeast(n) = inst.def.change {
            mana.raise_to(n);
        }
    }
}

/// The mana component `castable_spells` should expect for `card` (its
/// printed cost, as it would stand at CR 601.2f if cast now). No prompt: the
/// reductions are applied in gather order, which by §3.4's theorem gives the
/// total the cast will lock in whatever order the player later chooses. An X
/// stays a symbol: a caller previewing a cost at some X expands it first
/// (`ManaCost::with_x_as`), as 601.2f's first step does.
///
/// The preview and the cast read the same arithmetic, which is what keeps
/// enumeration and enforcement agreeing (`cost-architecture.md` §3.6): a
/// Thalia on the board would otherwise offer spells the cast then rolls back,
/// and an Electromancer would withhold ones the player can afford.
pub fn preview_mana_cost(game: &GameState, card: ObjectId, printed: &ManaCost) -> ManaCost {
    let instances = cost_modifications_for(game, card);
    if instances.is_empty() {
        return printed.clone();
    }
    let mut mana = ManaComponent::EMPTY;
    mana.add(printed);
    modify(game, &mut mana, &instances, |sources| (0..sources.len()).collect());
    mana.at(0)
}

/// The total CR 601.2b's choices make, with X a variable: what the
/// announcement of X reads (`oracle::x_values`). No prompt, and the
/// reductions in gather order, as in [`preview_mana_cost`].
pub fn total_cost_in_x(
    game: &GameState,
    spell: ObjectId,
    base_mana_cost: &ManaCost,
    chosen_alt_cost: Option<&AlternativeCost>,
    chosen_additional_costs: &[&AdditionalCost],
) -> TotalCostInX {
    let mut total = assemble_total_cost(base_costs(base_mana_cost, chosen_alt_cost), chosen_additional_costs);
    let instances = cost_modifications_for(game, spell);
    modify(game, &mut total.mana, &instances, |sources| (0..sources.len()).collect());
    total
}

/// CR 601.2f's total with X a variable: steps 1 and 2 assemble it, steps 3–5
/// modify its mana, and step 6 locks it at the announced X
/// ([`Self::locked_at`]).
#[derive(Debug, Clone, PartialEq)]
pub struct TotalCostInX {
    /// "The mana component of the total cost", singular: every mana cost
    /// among the costs is one.
    pub mana: ManaComponent,
    /// The other costs, in their order.
    pub non_mana: Vec<Cost>,
    /// Whether a cost assembled had mana in it: such a component stays at
    /// {0}, and one an increase gives a mana-less alternative cost
    /// (CR 118.9d) is added only if it has something in it.
    has_mana_cost: bool,
}

impl TotalCostInX {
    /// The locked total at an announced X, with the mana component **first**.
    /// CR 601.2h lets the player pay the first group's costs "in any order"
    /// and the engine picks one: the mana payment is the one that can still
    /// fail after `can_pay_costs` — an illegal generic split — and a failure
    /// mid-list leaves the costs before it paid, so the fallible payment goes
    /// first.
    fn locked_at(self, x: u64) -> Vec<Cost> {
        let mana = self.mana.at(x);
        let mut costs = self.non_mana;
        if self.has_mana_cost || !mana.symbols.is_empty() {
            costs.insert(0, Cost::Mana(mana));
        }
        costs
    }
}

// ---------------------------------------------------------------------------
// The mana component
// ---------------------------------------------------------------------------

/// CR 601.2f's "mana component of the total cost", as its steps compute it:
/// every symbol but generic and the base's X, in order; the generic part a
/// count; and the base's X a variable. X adds only generic mana and no
/// modification reads it (`representable`), so the component at any
/// announced X is one evaluation, [`ManaComponent::at`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManaComponent {
    pips: Vec<ManaSymbol>,
    generic: u64,
    /// The base's X symbols, each the announced X in generic mana (CR 107.3a).
    x_symbol_count: u64,
    /// The generic mana the reductions take off, at most all of it
    /// (CR 601.2f: "It can't be reduced to less than {0}"). A colored
    /// reduction takes a pip of its color first, so only what it finds no pip
    /// for counts here (CR 118.7d).
    generic_reduction: u64,
    /// The least mana value a direct effect on the total allows (Trinisphere).
    at_least: u8,
}

impl ManaComponent {
    /// No mana: what assembly starts from.
    pub const EMPTY: ManaComponent =
        ManaComponent { pips: Vec::new(), generic: 0, x_symbol_count: 0, generic_reduction: 0, at_least: 0 };

    /// A base cost's mana, whose X is the announced X.
    fn add_base(&mut self, mana: &ManaCost) {
        for symbol in &mana.symbols {
            match symbol {
                ManaSymbol::X => self.x_symbol_count += 1,
                ManaSymbol::Generic => self.generic += 1,
                other => self.pips.push(*other),
            }
        }
    }

    /// Mana whose symbols are kept as printed, an X among them.
    fn add(&mut self, mana: &ManaCost) {
        for symbol in &mana.symbols {
            match symbol {
                ManaSymbol::Generic => self.generic += 1,
                other => self.pips.push(*other),
            }
        }
    }

    /// "cost {N} more to cast": add the symbols as printed.
    fn increase(&mut self, amount: &ManaCost) {
        for symbol in amount.symbols.iter().filter(|symbol| representable(symbol, "increase")) {
            match symbol {
                ManaSymbol::Generic => self.generic += 1,
                other => self.pips.push(*other),
            }
        }
    }

    /// "cost {N} less to cast", under CR 118.7a–d and 118.7g.
    ///
    /// Per symbol of the reduction: generic and snow come off generic (118.7a,
    /// 118.7g); a colored or colorless symbol takes a pip of its own type if
    /// the cost has one and otherwise comes off generic (118.7b–d — the
    /// "excess" cases fall out of doing this one symbol at a time). What comes
    /// off generic is kept as a count and taken at [`Self::generic_at`], which
    /// saturates at zero (601.2f) as each reduction in turn would.
    fn reduce(&mut self, amount: &ManaCost) {
        for symbol in &amount.symbols {
            match symbol {
                ManaSymbol::Generic | ManaSymbol::Snow => self.generic_reduction += 1,
                ManaSymbol::Colored(_) | ManaSymbol::Colorless => {
                    let wanted = pip_type(symbol);
                    match self.pips.iter().position(|p| pip_type(p) == wanted) {
                        Some(at) => {
                            self.pips.remove(at);
                        }
                        None => self.generic_reduction += 1,
                    }
                }
                other => {
                    representable(other, "reduction");
                }
            }
        }
    }

    /// Trinisphere: the mana value raised to `n` with generic mana.
    fn raise_to(&mut self, n: u8) {
        self.at_least = self.at_least.max(n);
    }

    /// The generic mana the component asks at an announced X.
    pub fn generic_at(&self, x: u64) -> u64 {
        let pips_value: u64 = self.pips.iter().map(|symbol| u64::from(symbol.mana_value())).sum();
        let raised = u64::from(self.at_least).saturating_sub(pips_value);
        (self.generic + self.x_symbol_count.saturating_mul(x)).saturating_sub(self.generic_reduction).max(raised)
    }

    /// How many times the announced X is paid: the base's X symbols
    /// (CR 107.3a).
    pub fn x_symbol_count(&self) -> u64 {
        self.x_symbol_count
    }

    /// What the component asks at every X but its generic part: what the
    /// inventory pays beside the generic mana.
    pub fn pips(&self) -> ManaCost {
        ManaCost::from_symbols(self.pips.clone())
    }

    /// The generic part with X at 0, before it saturates: what is there
    /// besides X less what the reductions take off, negative when they take
    /// more than there is.
    pub fn generic_besides_x(&self) -> i64 {
        self.generic as i64 - self.generic_reduction as i64
    }

    /// The generic mana `generic_budget` leaves for the X symbols once the
    /// rest of the generic part is paid, or `None` when not even X = 0's
    /// generic part fits. A larger X never asks less (no modification reads
    /// X), so every X whose symbols this pays fits, and the most X is this
    /// over [`Self::x_symbol_count`], rounded down.
    pub fn generic_left_for_x(&self, generic_budget: u64) -> Option<u64> {
        if self.generic_at(0) > generic_budget {
            return None;
        }
        Some(generic_budget.saturating_add(self.generic_reduction).saturating_sub(self.generic))
    }

    /// The component at an announced X, generic first and the rest in their
    /// order, as `ManaCost::build` writes it: which is also what keeps
    /// `Display` printing {3} rather than {1}{1}{1} after an increase.
    pub fn at(&self, x: u64) -> ManaCost {
        let mut symbols = vec![ManaSymbol::Generic; self.generic_at(x) as usize];
        symbols.extend(self.pips.iter().copied());
        ManaCost::from_symbols(symbols)
    }
}

/// The pip `symbol` of a *cost* is, for reduction purposes: `{C}` has two
/// spellings (`Colorless` and `Colored(Colorless)`), and both are colorless.
fn pip_type(symbol: &ManaSymbol) -> Option<ManaType> {
    match symbol {
        ManaSymbol::Colored(t) => Some(*t),
        ManaSymbol::Colorless => Some(ManaType::Colorless),
        _ => None,
    }
}

/// Is this instance a cost reduction — CR 601.2f's second position, and so a
/// candidate for its ordering prompt?
///
/// A [`CostChange::ReduceGeneric`] that evaluates to zero is still a
/// reduction that *applies*: the rule orders the reductions, not the ones
/// that would change the answer.
fn is_reduction(inst: &CostModificationInstance) -> bool {
    matches!(inst.def.change, CostChange::Reduce(_) | CostChange::ReduceGeneric(_))
}

/// One reduction, at its position in the player's order.
///
/// A dynamic amount is read *here* rather than at gather. The two moments are
/// one — nothing between them touches the board — and reading it at
/// application keeps a `CostModificationInstance` the definition that was
/// gathered rather than a definition plus a snapshot of a number, which is
/// one fewer thing that can disagree with itself.
fn apply_one_reduction(game: &GameState, mana: &mut ManaComponent, inst: &CostModificationInstance) {
    match &inst.def.change {
        CostChange::Reduce(amount) => mana.reduce(amount),
        // CR 118.7a — generic only, whatever the count, and generic saturates
        // at zero. `settled_amount` returning `None` is an amount with no
        // static evaluator, whose own assert has already fired: reduce
        // nothing rather than guess a number.
        CostChange::ReduceGeneric(expr) => {
            let n = settled_amount(expr, game, inst.source).unwrap_or(0).max(0) as usize;
            if n > 0 {
                mana.reduce(&ManaCost::from_symbols(vec![ManaSymbol::Generic; n]));
            }
        }
        CostChange::Increase(_) | CostChange::TotalAtLeast(_) => {}
    }
}

/// The symbols an increase or a reduction may carry today. Hybrid and
/// Phyrexian are the payment half's (`cost-architecture.md` CP-1), and a
/// hybrid *reduction* is the one shape under which CR 601.2f's ordering
/// choice has two outcomes (118.7e) — refused in debug, dropped in release,
/// never guessed at. X has no meaning in a modification at all.
fn representable(symbol: &ManaSymbol, what: &str) -> bool {
    match symbol {
        ManaSymbol::Generic | ManaSymbol::Colored(_) | ManaSymbol::Colorless | ManaSymbol::Snow => true,
        other => {
            debug_assert!(
                false,
                "a cost {what} carrying {other:?}: hybrid and Phyrexian symbols are not \
                 payable yet (CP-1), and CR 118.7e makes a hybrid reduction the one case \
                 where the order of reductions changes the total. Dropped rather than \
                 guessed at."
            );
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::{CardData, CardDataBuilder};
    use crate::test_support::{
        card_of_type, put_in_hand, put_on_battlefield, setup_two_player_game, test_dp,
        vanilla_creature,
    };
    use crate::types::card_types::CardType;
    use crate::types::effects::ObjectFilter;

    fn cost(s: &[ManaSymbol]) -> ManaCost {
        ManaCost::from_symbols(s.to_vec())
    }
    const G: ManaSymbol = ManaSymbol::Generic;
    const R: ManaSymbol = ManaSymbol::Colored(ManaType::Red);
    const W: ManaSymbol = ManaSymbol::Colored(ManaType::White);
    const GR: ManaSymbol = ManaSymbol::Colored(ManaType::Green);
    const C: ManaSymbol = ManaSymbol::Colorless;

    /// `symbols` as one component, kept as printed.
    fn component(symbols: &[ManaSymbol]) -> ManaComponent {
        let mut mana = ManaComponent::EMPTY;
        mana.add(&cost(symbols));
        mana
    }

    fn reduced(base: &[ManaSymbol], by: &[ManaSymbol]) -> String {
        let mut mana = component(base);
        mana.reduce(&cost(by));
        mana.at(0).to_string()
    }

    /// `determine_total_cost` on a board with no cost effects: a spell in
    /// hand, an empty scripted provider, so any prompt would panic.
    fn determine(
        base: &ManaCost,
        alt: Option<&AlternativeCost>,
        additional: &[&AdditionalCost],
        x_value: u64,
    ) -> Vec<Cost> {
        let mut game = setup_two_player_game();
        let spell = put_in_hand(&mut game, vanilla_creature(1, 1, &[]), 0);
        determine_total_cost(&game, 0, spell, base, alt, additional, x_value, &test_dp())
    }

    // --- the arithmetic (CR 118.7) --------------------------------------

    // COVERS: ATOM-118.7a-001
    #[test]
    fn a_generic_reduction_touches_only_the_generic_component() {
        assert_eq!(reduced(&[G, G, R], &[G]), "{1}{R}");
        assert_eq!(reduced(&[R], &[G]), "{R}", "nothing generic to reduce");
        assert_eq!(reduced(&[G, R, R], &[G, G, G]), "{R}{R}", "the pips are untouched");
    }

    // COVERS: ATOM-118.7b-001
    #[test]
    fn a_colored_reduction_the_cost_has_no_pip_for_comes_off_generic() {
        assert_eq!(reduced(&[G, R], &[GR]), "{R}");
        assert_eq!(reduced(&[G, G, R], &[W, W]), "{R}");
    }

    // COVERS: ATOM-118.7c-001
    #[test]
    fn excess_colored_reduction_overflows_to_generic() {
        // {2}{R} less {R}{R}: one red pip, and the excess {R} comes off generic.
        assert_eq!(reduced(&[G, G, R], &[R, R]), "{1}");
        // `Display` renders an empty component as nothing, not as {0}.
        assert_eq!(reduced(&[G, R], &[R, R]), "");
    }

    // COVERS: ATOM-118.7d-001
    #[test]
    fn excess_colorless_reduction_overflows_to_generic() {
        assert_eq!(reduced(&[G, G, C], &[C, C]), "{1}");
        // `{C}` in a cost may be spelled either way; both are colorless pips.
        assert_eq!(reduced(&[G, ManaSymbol::Colored(ManaType::Colorless)], &[C]), "{1}");
    }

    // COVERS-PARTIAL: ATOM-601.2f-002
    #[test]
    fn generic_saturates_at_zero_and_a_nothing_component_is_zero() {
        // An empty component — CR 601.2f's "considered to be {0}" — which
        // `Display` renders as nothing.
        assert_eq!(reduced(&[G], &[G, G, G]), "");
        assert_eq!(reduced(&[G, GR], &[G, G, GR]), "");
        let mut mana = component(&[G, GR]);
        for by in [&[G][..], &[G][..], &[GR][..]] {
            mana.reduce(&cost(by));
        }
        assert!(mana.at(0).symbols.is_empty());
        assert_eq!(mana.at(0).mana_value(), 0);
    }

    /// CR 118.7g — snow reduces generic.
    #[test]
    fn a_snow_reduction_comes_off_generic() {
        assert_eq!(reduced(&[G, G, R], &[ManaSymbol::Snow]), "{1}{R}");
    }

    #[test]
    fn an_increase_adds_symbols_as_printed_and_keeps_generic_first() {
        let mut mana = component(&[G, R]);
        mana.increase(&cost(&[G]));
        assert_eq!(mana.at(0).to_string(), "{2}{R}");
        mana.increase(&cost(&[W]));
        assert_eq!(mana.at(0).to_string(), "{2}{R}{W}");
    }

    #[test]
    fn total_at_least_raises_the_mana_value_with_generic_and_never_lowers_it() {
        let mut mana = component(&[R]);
        mana.raise_to(3);
        assert_eq!(mana.at(0).to_string(), "{2}{R}");
        let mut mana = component(&[G, G, G, R]);
        mana.raise_to(3);
        assert_eq!(mana.at(0).to_string(), "{3}{R}");
        let mut mana = component(&[]);
        mana.raise_to(3);
        assert_eq!(mana.at(0).to_string(), "{3}");
    }

    /// §3.4's theorem, checked the other way: for every pair of reductions
    /// the engine can represent, both orders give one answer.
    #[test]
    fn reductions_commute_under_118_7a_to_d() {
        let bases: [&[ManaSymbol]; 5] = [&[G, R, R], &[G, G, GR], &[R], &[G, C], &[G, G, G, W, R]];
        let amounts: [&[ManaSymbol]; 5] = [&[G], &[R], &[GR, GR], &[C], &[G, G, R]];
        for base in bases {
            for a in amounts {
                for b in amounts {
                    let mut ab = component(base);
                    ab.reduce(&cost(a));
                    ab.reduce(&cost(b));
                    let mut ba = component(base);
                    ba.reduce(&cost(b));
                    ba.reduce(&cost(a));
                    assert_eq!(ab.at(0), ba.at(0), "{} less {} and {} in either order", cost(base), cost(a), cost(b));
                }
            }
        }
    }

    // --- the component ------------------------------------------------------

    /// CR 601.2f's "the mana component of the total cost", singular: a base
    /// cost and a kicker's mana are one component, and it goes first.
    #[test]
    fn every_mana_cost_in_the_assembled_list_is_one_component_and_it_goes_first() {
        let total = assemble_total_cost(
            vec![Cost::PayLife(2), Cost::Mana(cost(&[G, R])), Cost::TapSelf, Cost::Mana(cost(&[R]))],
            &[],
        );
        let mana = total.mana.at(0);
        assert_eq!(mana.to_string(), "{1}{R}{R}");
        assert_eq!(total.non_mana, vec![Cost::PayLife(2), Cost::TapSelf]);
        assert!(total.has_mana_cost);
        assert_eq!(total.locked_at(0), vec![Cost::Mana(mana), Cost::PayLife(2), Cost::TapSelf]);
    }

    /// An alternative cost with no mana in it gains a component only when a
    /// modification puts something in it (CR 118.9d); one that was there
    /// stays, even at {0}.
    #[test]
    fn a_component_appears_only_when_a_modification_gives_it_something() {
        let total = |mana: &[ManaSymbol], non_mana: Vec<Cost>, has_mana_cost: bool| TotalCostInX {
            mana: component(mana),
            non_mana,
            has_mana_cost,
        };
        assert_eq!(total(&[], vec![Cost::PayLife(2)], false).locked_at(0), vec![Cost::PayLife(2)]);
        assert_eq!(
            total(&[G], vec![Cost::PayLife(2)], false).locked_at(0),
            vec![Cost::Mana(cost(&[G])), Cost::PayLife(2)]
        );
        assert_eq!(total(&[], vec![], true).locked_at(0), vec![Cost::Mana(cost(&[]))]);
    }

    // --- the assembly (CR 601.2b's choices, 107.3, 118.8, 118.9) -----------

    #[test]
    fn a_plain_mana_cost_is_one_component() {
        let base = ManaCost::build(&[ManaType::Red], 1);
        assert_eq!(determine(&base, None, &[], 0), vec![Cost::Mana(base)]);
    }

    /// CR 107.3 — X is the value announced at CR 601.2b, in generic mana,
    /// once per X in the cost, and the X symbols themselves are gone.
    #[test]
    fn x_expands_into_generic_for_each_x_symbol() {
        let one_x = ManaCost::from_symbols(vec![ManaSymbol::X, R]);
        assert_eq!(determine(&one_x, None, &[], 3), vec![Cost::Mana(ManaCost::build(&[ManaType::Red], 3))]);
        assert_eq!(determine(&one_x, None, &[], 0), vec![Cost::Mana(ManaCost::build(&[ManaType::Red], 0))]);
        let two_x = ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::X]);
        assert_eq!(determine(&two_x, None, &[], 2), vec![Cost::Mana(ManaCost::build(&[], 4))]);
    }

    /// CR 107.3a — "any X in its mana cost or in any alternative cost or
    /// additional cost it has equals the announced value": an alternative
    /// cost's own X (miracle's `{X}{R}`) is expanded as the mana cost's is.
    #[test]
    fn an_alternative_costs_x_is_the_announced_x() {
        let base = ManaCost::from_symbols(vec![ManaSymbol::X, R, R]);
        let miracle = AlternativeCost::Custom("Miracle".to_string(), vec![Cost::Mana(cost(&[ManaSymbol::X, R]))]);
        assert_eq!(determine(&base, Some(&miracle), &[], 3), vec![Cost::Mana(ManaCost::build(&[ManaType::Red], 3))]);
    }

    /// An X in an additional cost (Thieving Skydiver's kicker) is not
    /// announced at 601.2b yet, so it stays a symbol no payment pays, as
    /// before (`mana-architecture.md` §3.7: CP-2's amount shape).
    #[test]
    fn an_additional_costs_x_stays_a_symbol() {
        let base = ManaCost::build(&[ManaType::Blue], 1);
        let kicker = AdditionalCost::Kicker(vec![Cost::Mana(cost(&[ManaSymbol::X]))]);
        assert_eq!(
            determine(&base, None, &[&kicker], 3),
            vec![Cost::Mana(ManaCost::from_symbols(vec![G, ManaSymbol::Colored(ManaType::Blue), ManaSymbol::X]))]
        );
    }

    /// CR 118.9a — an alternative cost replaces the mana cost entirely.
    #[test]
    fn an_alternative_cost_replaces_the_mana_cost() {
        let base = ManaCost::build(&[ManaType::Red], 2);
        let alt = AlternativeCost::Custom("Pay 1 life".to_string(), vec![Cost::PayLife(1)]);
        assert_eq!(determine(&base, Some(&alt), &[], 0), vec![Cost::PayLife(1)]);
    }

    /// CR 118.8 — a kicker's mana joins the base cost's in one component.
    #[test]
    fn a_kickers_mana_joins_the_one_component() {
        let base = ManaCost::build(&[ManaType::Red], 1);
        let kicker = AdditionalCost::Kicker(vec![Cost::Mana(ManaCost::build(&[ManaType::Red], 0))]);
        assert_eq!(
            determine(&base, None, &[&kicker], 0),
            vec![Cost::Mana(ManaCost::build(&[ManaType::Red, ManaType::Red], 1))]
        );
    }

    /// CR 118.9d — additional costs apply to an alternative cost as well, and
    /// the mana among them comes first.
    #[test]
    fn additional_costs_apply_to_an_alternative_cost_too() {
        let base = ManaCost::build(&[ManaType::Red], 3);
        let alt = AlternativeCost::Custom("Pay 2 life".to_string(), vec![Cost::PayLife(2)]);
        let kicker = AdditionalCost::Kicker(vec![Cost::Mana(ManaCost::build(&[ManaType::Green], 0))]);
        assert_eq!(
            determine(&base, Some(&alt), &[&kicker], 0),
            vec![Cost::Mana(ManaCost::build(&[ManaType::Green], 0)), Cost::PayLife(2)]
        );
    }

    // --- The spell's own cost abilities (CR 113.6d, 702.41a) --------------

    /// An artifact creature with `instances` copies of affinity for
    /// artifacts. Its own name is invented and it is registered nowhere
    /// (`engineering-practices.md` §3); the printed cards are
    /// `cards::phase_cm_cards`.
    fn affinity_spell(mana: &[ManaSymbol], instances: usize) -> std::sync::Arc<CardData> {
        let mut builder = CardDataBuilder::new("Affinity Lesson")
            .mana_cost(cost(mana))
            .card_type(CardType::Artifact)
            .card_type(CardType::Creature)
            .power_toughness(1, 1)
            .rules_text("Affinity for artifacts");
        for _ in 0..instances {
            builder = builder.affinity_for("Affinity for artifacts", ObjectFilter::ByType(CardType::Artifact));
        }
        builder.build()
    }

    /// What the castability preview says the spell costs with `artifacts`
    /// artifacts on P0's battlefield. The spell stays in hand, where the
    /// preview reads it.
    fn previewed(mana: &[ManaSymbol], artifacts: usize, instances: usize) -> String {
        let mut game = setup_two_player_game();
        for n in 0..artifacts {
            put_on_battlefield(&mut game, card_of_type(&format!("Rock {n}"), CardType::Artifact), 0);
        }
        let spell = put_in_hand(&mut game, affinity_spell(mana, instances), 0);
        preview_mana_cost(&game, spell, &cost(mana)).to_string()
    }

    /// CR 702.41a — "This spell costs {1} less to cast for each [text] you
    /// control", read off the finished board at CR 601.2f.
    // COVERS-PARTIAL: ATOM-702.41a-001
    #[test]
    fn affinity_reduces_generic_by_the_count_it_names() {
        assert_eq!(previewed(&[G, G, G, G, G, G], 4, 1), "{2}", "the atom's board: {{6}}, four artifacts");
        // The spell is itself an artifact card and it is in hand: `CountOf`
        // enumerates the battlefield, so it never counts itself.
        assert_eq!(previewed(&[G, G, G, G, G, G], 0, 1), "{6}", "nothing on the battlefield to count");
    }

    /// Generic saturates at zero (CR 601.2f) and only generic moves
    /// (CR 118.7a): affinity never eats a colored pip.
    #[test]
    fn affinity_saturates_at_zero_and_leaves_the_pips_alone() {
        assert_eq!(previewed(&[G, G, R], 9, 1), "{R}");
        assert_eq!(previewed(&[R, R], 3, 1), "{R}{R}", "no generic to reduce");
        assert_eq!(previewed(&[G, G, G, G], 4, 1), "", "a component reduced to nothing is {{0}}");
    }

    /// CR 702.41b — "if a spell has multiple instances of affinity, each of
    /// them applies". Two abilities on the effective list are two instances
    /// in the gather, and the second reduces what the first left.
    #[test]
    fn two_instances_of_affinity_each_apply() {
        assert_eq!(previewed(&[G, G, G, G, G, G], 2, 1), "{4}");
        assert_eq!(previewed(&[G, G, G, G, G, G], 2, 2), "{2}");
    }

    /// **The gate's third leg** (`cost-architecture.md` §3.1). A board with no
    /// cost source asks the layer system *nothing* about a card in hand that
    /// cannot modify its own cost. Without this, source 2 would put a frame
    /// on every card in hand at every castability preview, on every board — a
    /// layer walk per hand card per epoch, bought for a mechanic the card
    /// does not have.
    ///
    /// **The subject is half the question.** Thalia prints a cost ability and
    /// a Thalia in hand modifies nothing of her own, so the leg asks
    /// `CostSubject::applies_to_its_own_object` rather than "prints one at
    /// all". Asking the body alone cost five non-member walks per 200
    /// measured games (2026-09-07) — small, and exactly what the `fuzz_ab` arm
    /// that must reproduce `main` is for.
    #[test]
    fn the_preview_asks_the_layer_system_nothing_for_a_card_that_cannot_reduce_itself() {
        let mut game = setup_two_player_game();
        let plain = put_in_hand(&mut game, vanilla_creature(1, 1, &[]), 0);
        let printed = ManaCost::build(&[ManaType::Red], 1);

        let queries = |g: &GameState| g.diagnostics.layer_walks() + g.diagnostics.memo_hits();
        let before = queries(&game);
        assert_eq!(preview_mana_cost(&game, plain, &printed), printed);
        assert_eq!(queries(&game), before, "the gate skipped the frame entirely");

        // Thalia prints a cost ability whose subject is other spells, and she
        // is in hand rather than on the battlefield: nothing to ask.
        let thalia = put_in_hand(&mut game, crate::cards::phase_cm_cards::thalia_guardian_of_thraben(), 0);
        let before = queries(&game);
        let _ = preview_mana_cost(&game, thalia, &printed);
        assert_eq!(queries(&game), before, "a `Spells` subject in hand is not source 2's");

        // And the leg that opens it, on the same board: a printed cost
        // ability is what makes the frame worth computing.
        let affinity = put_in_hand(&mut game, affinity_spell(&[G], 1), 0);
        let before = queries(&game);
        let _ = preview_mana_cost(&game, affinity, &printed);
        assert!(queries(&game) > before, "the spell's own ability list was read");
    }
}
