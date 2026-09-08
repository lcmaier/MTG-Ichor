// AutoPayer — the payment prompts that have a right answer, as a decorator.
//
// `backlog.md` §2.18's "auto-payment oracle": the thing a GUI's auto-pay
// button and an AI harness both want, and what `cost-architecture.md` §3.4
// named when it said "the engine keeps asking; a payer answers".
//
// **What it answers, and the criterion.** A prompt belongs here when every
// legal answer leaves the same game state except for mana. Mana in a pool is
// spent by the payment or emptied at end of step (CR 500.4) — it is not
// durable — so choosing among those answers is paying, not playing.
//
//   CR 601.2h  `GenericManaAllocation` — which mana pays the generic part
//   CR 601.2f  `OrderCostReductions`   — the order two or more reductions apply
//
// **What it deliberately does not answer.** `ChooseSacrificeForCost` fails the
// criterion: different answers leave different permanents on the battlefield
// and put different `ZoneChange` events into the stream the trigger phase
// reads. Which creature to sacrifice for Altar's Reap is a strategic choice,
// and a payer that answers it is playing the game rather than paying for it.
// §3.4 listed it here before CM-3 built the prompt; that is corrected. A
// client that wants an auto-sacrifice policy stacks its own decorator for that
// kind, in the AI harness or the GUI where strategy lives.
//
// The criterion is matched exhaustively over `ChoiceKind` at the two sites
// below, so CP-1's announcement prompt and item 72's reversal each have to
// pick a side rather than inherit one.
//
// **Stack invariant: one decorator per `ChoiceKind`.** See `mana_window_stop`,
// which owns the window's stop and is the other half of a paying client's
// stack. Disjoint kinds mean composition commutes.

use crate::state::game_state::GameState;
use crate::types::ids::PlayerId;
use crate::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use crate::ui::decision::DecisionProvider;

/// Answers CR 601.2f's and 601.2h's payment prompts from the prompt itself,
/// and passes every other decision to `D`.
pub struct AutoPayer<D> {
    inner: D,
}

impl<D: DecisionProvider> AutoPayer<D> {
    pub fn new(inner: D) -> Self {
        AutoPayer { inner }
    }

    /// The wrapped provider, for a caller that needs to inspect it.
    pub fn inner(&self) -> &D {
        &self.inner
    }
}

/// Fill `total` into the buckets in the order they were offered, each up to its
/// own cap, starting from the minimums.
///
/// The caps are `ask_choose_generic_mana_allocation`'s, which already leave
/// every pip of the cost its own mana (CR 601.2h, `codebase-state.md` 16c), so
/// this needs no payment law of its own and must not re-derive any: the deleted
/// `auto_allocate_generic` subtracted the pips a second time, which is the bug
/// 16d paid for once already.
///
/// Bucket order is the pool's types sorted by discriminant, so the answer is
/// the same in every process — a solver that iterated a `HashMap` here would be
/// a determinism leak at a decision boundary.
fn fill_in_bucket_order(total: u64, mins: &[u64], maxs: Option<&[u64]>) -> Vec<u64> {
    let n = mins.len();
    let mut alloc = mins.to_vec();
    let mut remaining = total.saturating_sub(alloc.iter().sum::<u64>());
    for i in 0..n {
        if remaining == 0 {
            break;
        }
        let cap = maxs.map_or(u64::MAX, |m| m[i]);
        let give = remaining.min(cap.saturating_sub(alloc[i]));
        alloc[i] += give;
        remaining -= give;
    }
    alloc
}

impl<D: DecisionProvider> DecisionProvider for AutoPayer<D> {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        // Nothing this payer answers is a `pick_n`, and that is the criterion
        // showing through rather than an accident: `ChooseSacrificeForCost` is
        // the one payment prompt of this shape and it is the one left out.
        self.inner.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        min: u64,
        max: u64,
    ) -> u64 {
        self.inner.pick_number(game, player, context, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        if matches!(context.kind, ChoiceKind::GenericManaAllocation { .. }) {
            return fill_in_bucket_order(total, per_bucket_mins, per_bucket_maxs);
        }
        self.inner.allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize> {
        // Gather order — battlefield timestamp, the spell's own ability last.
        // By `cost-architecture.md` §3.4's theorem the order never changes the
        // total with the symbols the engine pays today, and gather order is
        // what `preview_mana_cost` applies, so enumeration, the castability
        // preview and the payment all read one number. The theorem's two expiry
        // conditions — a hybrid reduction symbol (CR 118.7e) and a `not_below`
        // reduction — are this arm's too: the phase that admits either owes
        // this line the minimizing order §3.4 describes.
        if matches!(context.kind, ChoiceKind::OrderCostReductions { .. }) {
            return (0..items.len()).collect();
        }
        self.inner.choose_ordering(game, player, context, items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::setup_two_player_game;
    use crate::types::ids::new_object_id;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::ui::decision::ScriptedDecisionProvider;

    fn buckets(n: usize) -> Vec<ChoiceOption> {
        (0..n).map(|_| ChoiceOption::ManaType(ManaType::Green)).collect()
    }

    fn split_ctx() -> ChoiceContext {
        ChoiceContext {
            kind: ChoiceKind::GenericManaAllocation { mana_cost: ManaCost::build(&[], 2) },
        }
    }

    /// The split is answered from the prompt's own caps, in bucket order, with
    /// the inner provider never consulted.
    #[test]
    fn fills_the_generic_split_in_bucket_order() {
        let game = setup_two_player_game();
        let dp = AutoPayer::new(ScriptedDecisionProvider::new());
        let alloc = dp.allocate(&game, 0, &split_ctx(), 3, &buckets(3), &[0, 0, 0], Some(&[1, 5, 5]));
        assert_eq!(alloc, vec![1, 2, 0], "first bucket capped at 1, the rest to the second");
        assert_eq!(alloc.iter().sum::<u64>(), 3);
    }

    /// Minimums are the floor, not a starting suggestion.
    #[test]
    fn the_split_starts_from_the_minimums() {
        let game = setup_two_player_game();
        let dp = AutoPayer::new(ScriptedDecisionProvider::new());
        let alloc = dp.allocate(&game, 0, &split_ctx(), 4, &buckets(2), &[1, 1], Some(&[9, 9]));
        assert_eq!(alloc, vec![3, 1]);
    }

    /// Same prompt, same answer, every time — no RNG reaches this decorator.
    #[test]
    fn the_split_is_a_function_of_the_prompt() {
        let game = setup_two_player_game();
        let dp = AutoPayer::new(ScriptedDecisionProvider::new());
        let once = dp.allocate(&game, 0, &split_ctx(), 3, &buckets(3), &[0, 0, 0], Some(&[2, 2, 2]));
        for _ in 0..8 {
            let again =
                dp.allocate(&game, 0, &split_ctx(), 3, &buckets(3), &[0, 0, 0], Some(&[2, 2, 2]));
            assert_eq!(once, again);
        }
    }

    /// CR 601.2f's ordering prompt is answered with gather order, which is what
    /// `preview_mana_cost` applies — so the preview and the payment agree.
    #[test]
    fn orders_cost_reductions_in_gather_order() {
        let game = setup_two_player_game();
        let dp = AutoPayer::new(ScriptedDecisionProvider::new());
        let ctx = ChoiceContext {
            kind: ChoiceKind::OrderCostReductions { spell_id: new_object_id() },
        };
        let items = vec![ChoiceOption::Object(new_object_id()); 3];
        assert_eq!(dp.choose_ordering(&game, 0, &ctx, &items), vec![0, 1, 2]);
    }

    /// The sacrifice choice is the wrapped provider's — it is a strategic
    /// choice, and the payer does not play the game.
    #[test]
    fn the_sacrifice_choice_passes_through() {
        let game = setup_two_player_game();
        let inner = ScriptedDecisionProvider::new();
        inner.expect_pick_n(
            ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id: new_object_id(), count: 1 },
            vec![1],
        );
        let dp = AutoPayer::new(inner);
        let ctx = ChoiceContext {
            kind: ChoiceKind::ChooseSacrificeForCost {
                spell_or_ability_id: new_object_id(),
                count: 1,
            },
        };
        let options = vec![ChoiceOption::Object(new_object_id()); 2];
        assert_eq!(dp.pick_n(&game, 0, &ctx, &options, (1, 1)), vec![1]);
    }

    /// An allocation that is not the generic split — combat damage — is the
    /// wrapped provider's too.
    #[test]
    fn an_allocation_that_is_not_the_generic_split_passes_through() {
        let game = setup_two_player_game();
        let inner = ScriptedDecisionProvider::new();
        let attacker_id = new_object_id();
        inner.expect_allocation(ChoiceKind::AssignCombatDamage { attacker_id }, vec![2, 1]);
        let dp = AutoPayer::new(inner);
        let ctx = ChoiceContext { kind: ChoiceKind::AssignCombatDamage { attacker_id } };
        assert_eq!(dp.allocate(&game, 0, &ctx, 3, &buckets(2), &[0, 0], None), vec![2, 1]);
    }
}
