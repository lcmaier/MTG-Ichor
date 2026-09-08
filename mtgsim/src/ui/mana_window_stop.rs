// ManaWindowStop — the CR 605.3a stop, as a DecisionProvider decorator.
//
// CR 605.3a lets a player activate mana abilities "whenever they are casting a
// spell or activating an ability that requires a mana payment", with no "until
// it is paid". Until CM-4 the engine ended the 601.2g window the moment
// `can_pay_costs` succeeded — a *payer's* policy sitting in the engine's loop,
// and the reason the Ironworks loop's step 3 was impossible
// (`cost-architecture.md` §3.11, `codebase-state.md` item 70). The engine now
// offers the window until the player declines; declining once the cost is
// covered is this decorator's job.
//
// **Why it is its own decorator and not part of `AutoPayer`.** The two toggle
// independently, for clients that want different things. A human turning off
// auto-pay wants the window to keep offering — floating mana mid-cast is the
// feature CR 605.3a describes and the thing the engine could not do before.
// An agent with no stop needs one: `RandomDecisionProvider`'s `AnyWillDo` arm
// never declines while a source is offered, so without this its only
// terminator is `WINDOW_ACTIVATION_CAP`.
//
// **Stack invariant: at most one decorator answers any one prompt.** Not one
// per `ChoiceKind`: §2.18's tap solver will answer `ManaAbilityWindow` too, and
// coexists because the two *predicates* are disjoint — the stop answers only
// once the component is covered, the solver only while it is not. What the
// invariant buys is that composition commutes, so a client may list its
// middleware in any order; two decorators answering one prompt would make the
// outermost win and turn the stack into a sequence. The abstraction that
// eventually replaces this forwarding boilerplate should `debug_assert!` it,
// and must not be called `Layer` — that word is CR 613's here.

use crate::state::game_state::GameState;
use crate::types::ids::PlayerId;
use crate::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use crate::ui::decision::DecisionProvider;

/// Declines CR 601.2g's mana-ability window once the locked mana component is
/// covered, and passes every other decision to `D`.
///
/// It does not *pick* an ability — the wrapped provider still chooses which
/// source to tap, which is what keeps a human's taps the human's. It answers
/// exactly one question: "keep going?"
pub struct ManaWindowStop<D> {
    inner: D,
}

impl<D: DecisionProvider> ManaWindowStop<D> {
    pub fn new(inner: D) -> Self {
        ManaWindowStop { inner }
    }

    /// The wrapped provider, for a caller that needs to inspect it.
    pub fn inner(&self) -> &D {
        &self.inner
    }
}

impl<D: DecisionProvider> DecisionProvider for ManaWindowStop<D> {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        // `remaining_cost` is the locked mana component minus what the pool
        // already covers (`remaining_cost_after_pool`), so "no symbols left"
        // is exactly "this payment needs no more mana". An empty pick is the
        // window's decline (bounds are (0, 1); see `ask_activate_mana_ability`).
        //
        // This is deliberately *not* the engine's old stop, which was
        // `can_pay_costs` over the whole cost list. The two differ on one
        // board — mana covered, some non-mana cost unpayable — and there no
        // number of further mana abilities could have helped, so the old loop
        // was asking a question with no answer. A mana ability pays CR 601.2h's
        // mana; the rest of the total is not its business.
        if let ChoiceKind::ManaAbilityWindow { remaining_cost, .. } = &context.kind {
            if remaining_cost.symbols.is_empty() {
                return Vec::new();
            }
        }
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
        self.inner.allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize> {
        self.inner.choose_ordering(game, player, context, items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::setup_two_player_game;
    use crate::types::ids::new_object_id;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::ui::decision::{PriorityAction, ScriptedDecisionProvider};

    fn window(remaining: ManaCost) -> ChoiceContext {
        ChoiceContext {
            kind: ChoiceKind::ManaAbilityWindow {
                spell_or_ability_id: new_object_id(),
                remaining_cost: remaining,
            },
        }
    }

    fn one_option() -> Vec<ChoiceOption> {
        vec![ChoiceOption::Action(PriorityAction::ActivateAbility(
            new_object_id(),
            crate::types::ids::new_ability_id(),
        ))]
    }

    /// Nothing left owed: the decorator declines and the wrapped provider is
    /// never asked. A `ScriptedDecisionProvider` with an empty queue panics on
    /// any call, so "never asked" is what not panicking means here.
    #[test]
    fn declines_a_covered_window_without_consulting_the_inner_provider() {
        let game = setup_two_player_game();
        let dp = ManaWindowStop::new(ScriptedDecisionProvider::new());
        let ctx = window(ManaCost::zero());
        assert!(dp.pick_n(&game, 0, &ctx, &one_option(), (0, 1)).is_empty());
    }

    /// A pip still owed: the decorator has no opinion and the wrapped
    /// provider's answer comes back untouched.
    #[test]
    fn passes_an_uncovered_window_through() {
        let game = setup_two_player_game();
        let inner = ScriptedDecisionProvider::new();
        inner.expect_pick_n(
            ChoiceKind::ManaAbilityWindow {
                spell_or_ability_id: new_object_id(),
                remaining_cost: ManaCost::zero(),
            },
            vec![0],
        );
        let dp = ManaWindowStop::new(inner);
        let ctx = window(ManaCost::build(&[ManaType::Green], 0));
        assert_eq!(dp.pick_n(&game, 0, &ctx, &one_option(), (0, 1)), vec![0]);
    }

    /// Generic still owed reads the same as a pip: `{1}` is one symbol.
    #[test]
    fn generic_still_owed_is_not_covered() {
        let game = setup_two_player_game();
        let inner = ScriptedDecisionProvider::new();
        inner.expect_pick_n(
            ChoiceKind::ManaAbilityWindow {
                spell_or_ability_id: new_object_id(),
                remaining_cost: ManaCost::zero(),
            },
            vec![0],
        );
        let dp = ManaWindowStop::new(inner);
        let ctx = window(ManaCost::build(&[], 1));
        assert_eq!(dp.pick_n(&game, 0, &ctx, &one_option(), (0, 1)), vec![0]);
    }

    /// Every other kind passes through untouched — the decorator owns one
    /// `ChoiceKind` and no more.
    #[test]
    fn a_prompt_that_is_not_the_window_is_the_inner_providers() {
        let game = setup_two_player_game();
        let inner = ScriptedDecisionProvider::new();
        inner.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
        let dp = ManaWindowStop::new(inner);
        let ctx = ChoiceContext { kind: ChoiceKind::PriorityAction };
        assert_eq!(dp.pick_n(&game, 0, &ctx, &one_option(), (0, 1)), vec![0]);
    }
}
