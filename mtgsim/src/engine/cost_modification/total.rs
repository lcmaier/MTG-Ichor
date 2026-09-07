//! CR 601.2f — the total cost, in the rule's order.
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
//! Six steps (`cost-architecture.md` §3.3): merge the mana component, gather,
//! increases, reductions in the player's order under CR 118.7a–d, the
//! direct-total effects, and the lock — which is that the value returned is
//! what CR 601.2h pays and nothing re-reads the board in between.

use super::gather::{gather, CostModificationInstance};
use crate::state::game_state::GameState;
use crate::types::cost_modification::CostChange;
use crate::types::costs::Cost;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::ui::ask::ask_order_cost_reductions;
use crate::ui::decision::DecisionProvider;

/// CR 601.2f, from the assembled costs to the locked total.
///
/// `assembled` is the base or alternative cost, an X expansion and the
/// chosen additional costs, as `costs::assemble_total_cost` lists them; the
/// non-mana costs come back in their order, and every `Cost::Mana` among
/// them comes back as **one** — CR 601.2f's "the mana component of the total
/// cost" is singular and CR 118.8 pays additional costs "at the same time".
/// The caster is asked to order the reductions when two or more apply
/// (CR 601.2f, "in any order"); one reduction asks nothing.
pub fn determine_total_cost(
    game: &GameState,
    caster: PlayerId,
    spell: ObjectId,
    assembled: Vec<Cost>,
    dp: &dyn DecisionProvider,
) -> Vec<Cost> {
    let (mut mana, others, slot) = split_mana_component(assembled);
    let instances = gather(game, spell);

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
    // not yet representable, respectively.
    let reductions: Vec<&CostModificationInstance> = instances
        .iter()
        .filter(|inst| matches!(inst.def.change, CostChange::Reduce(_)))
        .collect();
    let order: Vec<usize> = if reductions.len() >= 2 {
        let sources: Vec<ObjectId> = reductions.iter().map(|inst| inst.source).collect();
        ask_order_cost_reductions(dp, game, caster, spell, &sources)
    } else {
        (0..reductions.len()).collect()
    };
    for idx in order {
        if let CostChange::Reduce(amount) = &reductions[idx].def.change {
            apply_reduction(&mut mana, amount);
        }
    }

    // Step 5 — "any effects that directly affect the total cost". Two
    // Trinispheres agree, so no order is chosen.
    for inst in &instances {
        if let CostChange::TotalAtLeast(n) = inst.def.change {
            apply_total_at_least(&mut mana, n);
        }
    }

    // Step 6 — locked in: this value is paid, and nothing re-reads the board.
    rebuild(mana, others, slot)
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
    let instances = gather(game, card);
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
        if let CostChange::Reduce(amount) = &inst.def.change {
            apply_reduction(&mut mana, amount);
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

/// Split the assembled costs into one mana component, the non-mana costs in
/// their order, and where the component sat among them (`None` when there
/// was no mana cost at all — an alternative cost of "pay 2 life").
fn split_mana_component(costs: Vec<Cost>) -> (ManaCost, Vec<Cost>, Option<usize>) {
    let mut symbols: Vec<ManaSymbol> = Vec::new();
    let mut others = Vec::new();
    let mut slot = None;
    for cost in costs {
        match cost {
            Cost::Mana(mc) => {
                if slot.is_none() {
                    slot = Some(others.len());
                }
                symbols.extend(mc.symbols);
            }
            other => others.push(other),
        }
    }
    (canonical(symbols), others, slot)
}

/// Put the component back where the first mana cost was. A component that
/// appeared from nothing — an increase on an alternative cost with no mana
/// in it (CR 118.9d) — goes last; one that was there and is now {0} stays,
/// because "considered to be {0}" is still a mana cost of {0}.
fn rebuild(mana: ManaCost, mut others: Vec<Cost>, slot: Option<usize>) -> Vec<Cost> {
    match slot {
        Some(at) => others.insert(at, Cost::Mana(mana)),
        None if !mana.symbols.is_empty() => others.push(Cost::Mana(mana)),
        None => {}
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
        assert_eq!(reduced(&[G, G, R], &[W, W], ), "{R}");
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

    /// CR 601.2f's "the mana component of the total cost", singular: a base
    /// cost and a kicker's mana are one component, and it sits where the
    /// first mana cost sat.
    #[test]
    fn every_mana_cost_in_the_assembled_list_is_one_component() {
        let (mana, others, slot) = split_mana_component(vec![
            Cost::PayLife(2),
            Cost::Mana(cost(&[G, R])),
            Cost::Tap,
            Cost::Mana(cost(&[R])),
        ]);
        assert_eq!(mana.to_string(), "{1}{R}{R}");
        assert_eq!(others, vec![Cost::PayLife(2), Cost::Tap]);
        assert_eq!(slot, Some(1));
        assert_eq!(
            rebuild(mana.clone(), others, slot),
            vec![Cost::PayLife(2), Cost::Mana(mana), Cost::Tap]
        );
    }

    /// An alternative cost with no mana in it gains a component only when a
    /// modification puts something in it (CR 118.9d).
    #[test]
    fn a_component_appears_only_when_a_modification_gives_it_something() {
        assert_eq!(rebuild(cost(&[]), vec![Cost::PayLife(2)], None), vec![Cost::PayLife(2)]);
        assert_eq!(
            rebuild(cost(&[G]), vec![Cost::PayLife(2)], None),
            vec![Cost::PayLife(2), Cost::Mana(cost(&[G]))]
        );
        // And one that was there stays, even at {0}.
        assert_eq!(rebuild(cost(&[]), vec![], Some(0)), vec![Cost::Mana(cost(&[]))]);
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
}
