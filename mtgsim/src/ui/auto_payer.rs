// AutoPayer — the payment prompt that has a right answer, as a decorator.
//
// `backlog.md` §2.18's "auto-payment oracle", the half that answers: what a
// GUI's auto-pay button and an AI harness both want, and what
// `cost-architecture.md` §3.4 named when it said "the engine keeps asking; a
// payer answers".
//
// **What it answers, and the criterion.** A prompt belongs here when every
// legal answer leaves the same game. Not "the answers are similar enough" —
// the same game, so being asked cannot change anything.
//
//   CR 601.2f  `OrderCostReductions` — always: by `cost-architecture.md`
//              §3.4's theorem every order yields the identical total, so the
//              prompt is one the CR mandates and that cannot matter.
//
// **What it does not answer.** CR 601.2h's generic split: a split with surplus
// in the pool decides what is left up for the rest of the step (§3.4), and a
// split with one legal answer is the engine's (`ui::ask::forced_allocation`)
// — a prompt with one legal answer belongs to the engine, not to a middleware
// (`backlog.md` §2.22, which retired the branch that used to answer it here).
// `ChooseSacrificeForCost`: which creature dies is a strategic choice, and a
// client that wants an auto-sacrifice policy stacks its own decorator.
//
// The criterion is matched over `ChoiceKind` at the one site below, so a new
// payment prompt has to pick a side rather than inherit one.
//
// **Stack invariant: at most one decorator answers any one prompt.** Stated
// in full in `mana_window_stop`, which owns the window's stop and is the
// other half of a paying client's stack.

use crate::state::game_state::GameState;
use crate::types::ids::PlayerId;
use crate::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use crate::ui::decision::DecisionProvider;

/// Answers CR 601.2f's ordering prompt from the prompt itself, and passes
/// every other decision to `D`.
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
        // Every allocation is the wrapped provider's. The generic split that
        // reaches here has two or more legal answers — the engine took the
        // forced one before asking — and which mana pays the generic is the
        // player's (`cost-architecture.md` §3.4).
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
            kind: ChoiceKind::GenericManaAllocation {
                spell_or_ability_id: crate::types::ids::ObjectId::UNASSIGNED,
                mana_cost: ManaCost::build(&[], 2),
            },
        }
    }

    fn expects_split(answer: Vec<u64>) -> ScriptedDecisionProvider {
        let inner = ScriptedDecisionProvider::new();
        inner.expect_allocation(
            ChoiceKind::GenericManaAllocation {
                spell_or_ability_id: crate::types::ids::ObjectId::UNASSIGNED,
                mana_cost: ManaCost::zero(),
            },
            answer,
        );
        inner
    }

    /// **The review's board.** A {2}{U} three-drop cast off a pool of
    /// {U}{U}{U}{G}{G}, holding {U}{U} for Counterspell: the {U} pip reserves
    /// one blue, leaving caps of 2 blue and 2 green against 2 generic owed.
    /// Three legal answers, and which one is taken decides whether the counter
    /// is still live — so the payer must not take it.
    #[test]
    fn defers_a_split_that_decides_what_is_left_in_the_pool() {
        let game = setup_two_player_game();
        let dp = AutoPayer::new(expects_split(vec![0, 2]));
        let alloc = dp.allocate(&game, 0, &split_ctx(), 2, &buckets(2), &[0, 0], Some(&[2, 2]));
        assert_eq!(alloc, vec![0, 2], "the player spent green and kept blue up");
    }

    /// A split with one legal answer is the engine's, not this decorator's:
    /// caps summing to exactly what is owed never reach a provider from a game
    /// (`ui::ask::forced_allocation`), and a client that drives the provider by
    /// another route gets the wrapped provider's answer, never a second copy of
    /// the engine's. Retired A4k, 2026-09-18 (`backlog.md` §2.22).
    #[test]
    fn a_forced_split_is_the_wrapped_providers_too() {
        let game = setup_two_player_game();
        let dp = AutoPayer::new(expects_split(vec![1, 2, 1]));
        let alloc = dp.allocate(&game, 0, &split_ctx(), 4, &buckets(3), &[0, 0, 0], Some(&[1, 2, 1]));
        assert_eq!(alloc, vec![1, 2, 1], "the answer is the inner provider's, not computed here");
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
