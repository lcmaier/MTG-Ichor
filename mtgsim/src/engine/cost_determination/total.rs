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
//! alternative cost with X expanded, then the additional costs — merge every
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
    // Step 1 — the base. An alternative cost replaces the mana cost entirely
    // (CR 118.9a); otherwise the printed cost with every X expanded into the
    // generic mana CR 601.2b's announced value is worth (CR 107.3).
    let mut assembled: Vec<Cost> = match chosen_alt_cost {
        Some(alt) => alt.costs().to_vec(),
        None => {
            let x_count = base_mana_cost.x_count() as usize;
            let mut symbols: Vec<ManaSymbol> = base_mana_cost
                .symbols
                .iter()
                .filter(|s| !matches!(s, ManaSymbol::X))
                .copied()
                .collect();
            symbols.extend(std::iter::repeat(ManaSymbol::Generic).take(x_value as usize * x_count));
            vec![Cost::Mana(ManaCost::from_symbols(symbols))]
        }
    };

    // Step 2 — the additional costs the player chose (CR 118.8), whichever
    // base was chosen: CR 118.9d applies them to an alternative cost too.
    for additional in chosen_additional_costs {
        assembled.extend(additional.costs().iter().cloned());
    }

    modify(game, caster, spell, assembled, dp)
}

/// Steps 3–6: the modifications, and the lock.
fn modify(
    game: &GameState,
    caster: PlayerId,
    spell: ObjectId,
    assembled: Vec<Cost>,
    dp: &dyn DecisionProvider,
) -> Vec<Cost> {
    let (mut mana, others, had_mana) = split_mana_component(assembled);
    let instances = cost_modifications_for(game, spell);

    // Step 3 — increases. Addition commutes; the gather order is for the log.
    for inst in &instances {
        if let CostChange::Increase(amount) = &inst.def.change {
            apply_increase(&mut mana, amount);
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
    let reductions: Vec<&CostModificationInstance> =
        instances.iter().filter(|inst| is_reduction(inst)).collect();
    let order: Vec<usize> = if reductions.len() >= 2 {
        let sources: Vec<ObjectId> = reductions.iter().map(|inst| inst.source).collect();
        ask_order_cost_reductions(dp, game, caster, spell, &sources)
    } else {
        (0..reductions.len()).collect()
    };
    for idx in order {
        apply_one_reduction(game, &mut mana, reductions[idx]);
    }

    // Step 5 — "any effects that directly affect the total cost". Two
    // Trinispheres agree, so no order is chosen.
    for inst in &instances {
        if let CostChange::TotalAtLeast(n) = inst.def.change {
            apply_total_at_least(&mut mana, n);
        }
    }

    // Step 6 — locked in: this value is paid, and nothing re-reads the board.
    rebuild(mana, others, had_mana)
}

/// The mana component `castable_spells` should expect for `card` (its
/// printed cost, as it would stand at CR 601.2f if cast now). No prompt: the
/// reductions are applied in gather order, which by §3.4's theorem gives the
/// total the cast will lock in whatever order the player later chooses.
///
/// The preview and the cast read the same arithmetic, which is what keeps
/// enumeration and enforcement agreeing (`cost-architecture.md` §3.6): a
/// Thalia on the board no longer offers spells the cast then rolls back, and
/// an Electromancer no longer withholds ones the player can afford.
pub fn preview_mana_cost(game: &GameState, card: ObjectId, printed: &ManaCost) -> ManaCost {
    let instances = cost_modifications_for(game, card);
    if instances.is_empty() {
        return printed.clone();
    }
    let mut mana = printed.clone();
    for inst in &instances {
        if let CostChange::Increase(amount) = &inst.def.change {
            apply_increase(&mut mana, amount);
        }
    }
    for inst in &instances {
        if is_reduction(inst) {
            apply_one_reduction(game, &mut mana, inst);
        }
    }
    for inst in &instances {
        if let CostChange::TotalAtLeast(n) = inst.def.change {
            apply_total_at_least(&mut mana, n);
        }
    }
    mana
}

// ---------------------------------------------------------------------------
// The mana component
// ---------------------------------------------------------------------------

/// Split the assembled costs into one mana component and the non-mana costs
/// in their order; the flag says whether there was any mana cost at all (an
/// alternative cost of "pay 2 life" has none).
fn split_mana_component(costs: Vec<Cost>) -> (ManaCost, Vec<Cost>, bool) {
    let mut symbols: Vec<ManaSymbol> = Vec::new();
    let mut others = Vec::new();
    let mut had_mana = false;
    for cost in costs {
        match cost {
            Cost::Mana(mc) => {
                had_mana = true;
                symbols.extend(mc.symbols);
            }
            other => others.push(other),
        }
    }
    (canonical(symbols), others, had_mana)
}

/// The total, with the mana component **first**. CR 601.2h lets the player
/// pay the first group's costs "in any order" and the engine picks one: the
/// mana payment is the one that can still fail after `can_pay_costs` — an
/// illegal generic split — and a failure mid-list leaves the costs before it
/// paid, so the fallible payment goes first. A component that was there and
/// is now {0} stays, because "considered to be {0}" is still a mana cost of
/// {0}; one that appeared from nothing — an increase on a mana-less
/// alternative cost (CR 118.9d) — is added only if it has something in it.
fn rebuild(mana: ManaCost, mut others: Vec<Cost>, had_mana: bool) -> Vec<Cost> {
    if had_mana || !mana.symbols.is_empty() {
        others.insert(0, Cost::Mana(mana));
    }
    others
}

/// The conventional order `ManaCost::build` writes: generic first, then every
/// other symbol in the order it appeared. Coalescing generic is also what
/// keeps `Display` printing {3} rather than {1}{1}{1} after an increase.
fn canonical(symbols: Vec<ManaSymbol>) -> ManaCost {
    let generic = symbols.iter().filter(|s| **s == ManaSymbol::Generic).count();
    let mut out: Vec<ManaSymbol> = vec![ManaSymbol::Generic; generic];
    out.extend(symbols.into_iter().filter(|s| *s != ManaSymbol::Generic));
    ManaCost::from_symbols(out)
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

/// "cost {N} more to cast": add the symbols as printed.
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
fn apply_one_reduction(game: &GameState, mana: &mut ManaCost, inst: &CostModificationInstance) {
    match &inst.def.change {
        CostChange::Reduce(amount) => apply_reduction(mana, amount),
        // CR 118.7a — generic only, whatever the count, and generic saturates
        // at zero. `settled_amount` returning `None` is an amount with no
        // static evaluator, whose own assert has already fired: reduce
        // nothing rather than guess a number.
        CostChange::ReduceGeneric(expr) => {
            let n = settled_amount(expr, game, inst.source).unwrap_or(0).max(0) as usize;
            if n > 0 {
                apply_reduction(mana, &ManaCost::from_symbols(vec![ManaSymbol::Generic; n]));
            }
        }
        CostChange::Increase(_) | CostChange::TotalAtLeast(_) => {}
    }
}

fn apply_increase(mana: &mut ManaCost, amount: &ManaCost) {
    let mut symbols = std::mem::take(&mut mana.symbols);
    for symbol in &amount.symbols {
        if representable(symbol, "increase") {
            symbols.push(*symbol);
        }
    }
    *mana = canonical(symbols);
}

/// "cost {N} less to cast", under CR 118.7a–d and 118.7g.
///
/// Per symbol of the reduction: generic and snow come off generic (118.7a,
/// 118.7g); a colored or colorless symbol takes a pip of its own type if the
/// cost has one and otherwise comes off generic (118.7b–d — the "excess"
/// cases fall out of doing this one symbol at a time). Generic saturates at
/// zero (601.2f).
fn apply_reduction(mana: &mut ManaCost, amount: &ManaCost) {
    let mut pips: Vec<ManaSymbol> = mana
        .symbols
        .iter()
        .copied()
        .filter(|s| *s != ManaSymbol::Generic)
        .collect();
    let mut generic = mana.symbols.len() - pips.len();
    let mut generic_off = 0usize;
    for symbol in &amount.symbols {
        match symbol {
            ManaSymbol::Generic | ManaSymbol::Snow => generic_off += 1,
            ManaSymbol::Colored(_) | ManaSymbol::Colorless => {
                let wanted = pip_type(symbol);
                match pips.iter().position(|p| pip_type(p) == wanted) {
                    Some(at) => {
                        pips.remove(at);
                    }
                    None => generic_off += 1,
                }
            }
            other => {
                representable(other, "reduction");
            }
        }
    }
    generic = generic.saturating_sub(generic_off);
    let mut symbols = vec![ManaSymbol::Generic; generic];
    symbols.extend(pips);
    *mana = canonical(symbols);
}

/// Trinisphere: the mana value is raised to `n` with generic mana.
fn apply_total_at_least(mana: &mut ManaCost, n: u8) {
    let value = mana.mana_value();
    if value < n {
        let mut symbols = std::mem::take(&mut mana.symbols);
        symbols.extend(std::iter::repeat(ManaSymbol::Generic).take((n - value) as usize));
        *mana = canonical(symbols);
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

    fn reduced(base: &[ManaSymbol], by: &[ManaSymbol]) -> String {
        let mut mana = cost(base);
        apply_reduction(&mut mana, &cost(by));
        mana.to_string()
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
        let mut mana = cost(&[G, GR]);
        for by in [&[G][..], &[G][..], &[GR][..]] {
            apply_reduction(&mut mana, &cost(by));
        }
        assert!(mana.symbols.is_empty());
        assert_eq!(mana.mana_value(), 0);
    }

    /// CR 118.7g — snow reduces generic.
    #[test]
    fn a_snow_reduction_comes_off_generic() {
        assert_eq!(reduced(&[G, G, R], &[ManaSymbol::Snow]), "{1}{R}");
    }

    #[test]
    fn an_increase_adds_symbols_as_printed_and_keeps_generic_first() {
        let mut mana = cost(&[G, R]);
        apply_increase(&mut mana, &cost(&[G]));
        assert_eq!(mana.to_string(), "{2}{R}");
        apply_increase(&mut mana, &cost(&[W]));
        assert_eq!(mana.to_string(), "{2}{R}{W}");
    }

    #[test]
    fn total_at_least_raises_the_mana_value_with_generic_and_never_lowers_it() {
        let mut mana = cost(&[R]);
        apply_total_at_least(&mut mana, 3);
        assert_eq!(mana.to_string(), "{2}{R}");
        let mut mana = cost(&[G, G, G, R]);
        apply_total_at_least(&mut mana, 3);
        assert_eq!(mana.to_string(), "{3}{R}");
        let mut mana = cost(&[]);
        apply_total_at_least(&mut mana, 3);
        assert_eq!(mana.to_string(), "{3}");
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
                    let mut ab = cost(base);
                    apply_reduction(&mut ab, &cost(a));
                    apply_reduction(&mut ab, &cost(b));
                    let mut ba = cost(base);
                    apply_reduction(&mut ba, &cost(b));
                    apply_reduction(&mut ba, &cost(a));
                    assert_eq!(ab, ba, "{} less {} and {} in either order", cost(base), cost(a), cost(b));
                }
            }
        }
    }

    // --- the component ------------------------------------------------------

    /// CR 601.2f's "the mana component of the total cost", singular: a base
    /// cost and a kicker's mana are one component, and it goes first.
    #[test]
    fn every_mana_cost_in_the_assembled_list_is_one_component_and_it_goes_first() {
        let (mana, others, had_mana) = split_mana_component(vec![
            Cost::PayLife(2),
            Cost::Mana(cost(&[G, R])),
            Cost::Tap,
            Cost::Mana(cost(&[R])),
        ]);
        assert_eq!(mana.to_string(), "{1}{R}{R}");
        assert_eq!(others, vec![Cost::PayLife(2), Cost::Tap]);
        assert!(had_mana);
        assert_eq!(
            rebuild(mana.clone(), others, had_mana),
            vec![Cost::Mana(mana), Cost::PayLife(2), Cost::Tap]
        );
    }

    /// An alternative cost with no mana in it gains a component only when a
    /// modification puts something in it (CR 118.9d); one that was there
    /// stays, even at {0}.
    #[test]
    fn a_component_appears_only_when_a_modification_gives_it_something() {
        assert_eq!(rebuild(cost(&[]), vec![Cost::PayLife(2)], false), vec![Cost::PayLife(2)]);
        assert_eq!(
            rebuild(cost(&[G]), vec![Cost::PayLife(2)], false),
            vec![Cost::Mana(cost(&[G])), Cost::PayLife(2)]
        );
        assert_eq!(rebuild(cost(&[]), vec![], true), vec![Cost::Mana(cost(&[]))]);
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

    /// CR 118.9a — an alternative cost replaces the mana cost entirely.
    #[test]
    fn an_alternative_cost_replaces_the_mana_cost() {
        let base = ManaCost::build(&[ManaType::Red], 2);
        let alt = AlternativeCost::Custom("Pay 1 life".to_string(), vec![Cost::PayLife(1)]);
        assert_eq!(determine(&base, Some(&alt), &[], 0), vec![Cost::PayLife(1)]);
    }

    /// CR 118.8 — a kicker's mana joins the base cost's in one component.
    /// Until CM-1 it was a second `Cost::Mana` that the generic split and the
    /// mana window both read past.
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

    // --- CM-2: the spell's own cost abilities (CR 113.6d, 702.41a) ---------

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
            builder = builder.affinity_for(ObjectFilter::ByType(CardType::Artifact));
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
    // COVERS: ATOM-702.41a-001
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
    /// cost source and a card that prints no cost ability asks the layer
    /// system *nothing*. Without this, source 2 would put a frame on every
    /// card in hand at every castability preview, on every board — a layer
    /// walk per hand card per epoch, bought for a mechanic the card does not
    /// have.
    #[test]
    fn the_preview_asks_the_layer_system_nothing_for_a_card_with_no_cost_ability() {
        let mut game = setup_two_player_game();
        let plain = put_in_hand(&mut game, vanilla_creature(1, 1, &[]), 0);
        let printed = ManaCost::build(&[ManaType::Red], 1);

        let queries = |g: &GameState| g.counters.layer_walks() + g.counters.memo_hits();
        let before = queries(&game);
        assert_eq!(preview_mana_cost(&game, plain, &printed), printed);
        assert_eq!(queries(&game), before, "the gate skipped the frame entirely");

        // And the leg that opens it, on the same board: a printed cost
        // ability is what makes the frame worth computing.
        let affinity = put_in_hand(&mut game, affinity_spell(&[G], 1), 0);
        let before = queries(&game);
        let _ = preview_mana_cost(&game, affinity, &printed);
        assert!(queries(&game) > before, "the spell's own ability list was read");
    }
}
