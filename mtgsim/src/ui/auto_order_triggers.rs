// AutoOrderTriggers — CR 603.3b's order taken for a person, in the order the
// prompt lists their triggers.
//
// `backlog.md` §2.22 row 8. The engine already declines to ask when every
// order leaves the same game (`trigger_order_cannot_change_outcome`); what
// reaches a seat is a real choice, and taking it for the person is a policy.
// The order the prompt lists is trigger order, which within one event is the
// sources' timestamp order (CR 613.7), and the first goes on the stack lowest
// (`ask_order_triggers`), so the newest source's trigger resolves first.
//
// Human seats only, inside `FullControl`'s decorated stack, so full control is
// the toggle: off, the order is taken for the person; on, they are asked. An
// agent's seat never takes it, since ordering is a decision and the agent
// makes its own.

use crate::state::game_state::GameState;
use crate::types::ids::PlayerId;
use crate::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use crate::ui::decision::{DecisionProvider, SeatMode};

/// Answers `D`'s seat's trigger order in the order it is offered, and passes
/// every other decision to `D`.
pub struct AutoOrderTriggers<D> {
    inner: D,
}

impl<D: DecisionProvider> AutoOrderTriggers<D> {
    pub fn new(inner: D) -> Self {
        AutoOrderTriggers { inner }
    }
}

impl<D: DecisionProvider> DecisionProvider for AutoOrderTriggers<D> {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
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
        if matches!(context.kind, ChoiceKind::OrderTriggers { .. }) {
            return (0..items.len()).collect();
        }
        self.inner.choose_ordering(game, player, context, items)
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.inner.seat_mode(player)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::setup_two_player_game;
    use crate::types::ids::new_object_id;
    use crate::types::triggers::TriggerTier;
    use crate::ui::decision::ScriptedDecisionProvider;
    use crate::ui::full_control::{FullControl, FullControlSwitch};

    fn three_triggers() -> Vec<ChoiceOption> {
        (0..3).map(|_| ChoiceOption::Object(new_object_id())).collect()
    }

    fn order_triggers() -> ChoiceContext {
        ChoiceContext::new(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First })
    }

    /// The person beneath is scripted with nothing, so a prompt that reached
    /// them would panic. Under full control it does reach them.
    #[test]
    fn the_trigger_order_is_taken_as_offered_unless_full_control_is_on() {
        let game = setup_two_player_game();
        let raw = ScriptedDecisionProvider::new();
        raw.expect_ordering(order_triggers().kind, vec![2, 1, 0]);
        let switch = FullControlSwitch::default();
        let seat = FullControl::new(AutoOrderTriggers::new(ScriptedDecisionProvider::new()), raw, switch.clone());

        assert_eq!(seat.choose_ordering(&game, 0, &order_triggers(), &three_triggers()), vec![0, 1, 2], "taken for them");
        switch.set(true);
        assert_eq!(seat.choose_ordering(&game, 0, &order_triggers(), &three_triggers()), vec![2, 1, 0], "asked");
    }

    /// The order of cards scried to the bottom is an ordering too, and the
    /// person makes it.
    #[test]
    fn every_other_prompt_reaches_the_person() {
        let game = setup_two_player_game();
        let person = ScriptedDecisionProvider::new();
        let scry_order = ChoiceContext::new(ChoiceKind::ScryOrder { source: None, bottom: true });
        person.expect_ordering(scry_order.kind.clone(), vec![1, 0]);
        let seat = AutoOrderTriggers::new(person);

        assert_eq!(seat.choose_ordering(&game, 0, &scry_order, &three_triggers()[..2]), vec![1, 0]);
    }
}
