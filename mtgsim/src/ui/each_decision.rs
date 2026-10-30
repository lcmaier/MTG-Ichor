// OnEachDecision — a hook at every prompt the wrapped provider must decide.
//
// A decision is `ui::ask`'s: a prompt with two or more legal answers
// (`codebase-state.md` item 138), decided here by the same four predicates
// `ui::ask` counts with, so an instrument cannot drift from the count.
// Innermost in a stack, under every decorator that answers a prompt itself
// (`ManaWindowStop`), it sees what reaches the agent: `fuzz_games --census`
// counts there, and the clone instruments clone there.
//
// It answers nothing and changes no answer, so a stack with it plays the
// games the stack without it plays.

use crate::state::game_state::GameState;
use crate::types::ids::PlayerId;
use crate::ui::ask::{forced_allocation, only_number, only_order, only_pick};
use crate::ui::choice_types::{ChoiceContext, ChoiceOption};
use crate::ui::decision::{DecisionProvider, SeatMode};

/// Calls `hook` with the board and the prompt at each prompt with two or more
/// legal answers, then passes the prompt to `D`.
pub struct OnEachDecision<D, F> {
    inner: D,
    hook: F,
}

impl<D, F: Fn(&GameState, &ChoiceContext)> OnEachDecision<D, F> {
    pub fn new(inner: D, hook: F) -> Self {
        OnEachDecision { inner, hook }
    }
}

impl<D: DecisionProvider, F: Fn(&GameState, &ChoiceContext)> DecisionProvider for OnEachDecision<D, F> {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        if only_pick(options.len(), bounds).is_none() {
            (self.hook)(game, context);
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
        if only_number(min, max).is_none() {
            (self.hook)(game, context);
        }
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
        if forced_allocation(total, per_bucket_mins, per_bucket_maxs).is_none() {
            (self.hook)(game, context);
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
        if only_order(items.len()).is_none() {
            (self.hook)(game, context);
        }
        self.inner.choose_ordering(game, player, context, items)
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.inner.seat_mode(player)
    }
}
