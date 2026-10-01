// AutoYield — a priority prompt answered with `Pass` while a yield the person
// set still holds.
//
// `backlog.md` §2.22 row 4: Arena's three yields, each read off the
// `&GameState` every prompt carries. Human seats only: an agent's pass with
// something else on offer is a decision, and a rule that makes it is the
// agent itself. It ships with full control because each is the other's
// counterweight. A turn that fast-forwards tells the table its seat holds
// nothing at instant speed, and the switch is what puts the prompts back:
// under full control a seat's prompts never reach this decorator.
//
// **A yield is set at a priority prompt the person answers, and that answer
// is a pass.** The provider at the bottom of the seat holds a `Yields` handle
// and sets it there, so the answer that set it is the record a replay needs.

use std::cell::RefCell;
use std::rc::Rc;

use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use crate::ui::decision::{DecisionProvider, PriorityAction, SeatMode};

/// How long a yield passes priority for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Yield {
    /// Arena's "end turn": until the turn ends.
    UntilEndOfTurn,
    /// Until the stack is not what it was — something added to it or gone from
    /// it, which is when a response may be wanted.
    UntilStackChanges,
    /// Until the seat's own next turn begins.
    UntilYourNextTurn,
}

/// A yield as set, with the board it is measured from.
#[derive(Clone, Debug)]
struct Held {
    until: Yield,
    turn: u32,
    stack: Vec<ObjectId>,
}

impl Held {
    fn holds(&self, game: &GameState, player: PlayerId) -> bool {
        match self.until {
            Yield::UntilEndOfTurn => game.turn_number == self.turn,
            Yield::UntilStackChanges => game.stack == self.stack,
            Yield::UntilYourNextTurn => game.turn_number == self.turn || game.active_player != player,
        }
    }
}

/// The yield a seat's person has set, shared by the provider that sets it and
/// the [`AutoYield`] that reads it.
#[derive(Clone, Debug, Default)]
pub struct Yields(Rc<RefCell<Option<Held>>>);

impl Yields {
    /// Pass until `until`, measured from `game` as it is now.
    pub fn set(&self, game: &GameState, until: Yield) {
        *self.0.borrow_mut() = Some(Held { until, turn: game.turn_number, stack: game.stack.clone() });
    }

    pub fn clear(&self) {
        *self.0.borrow_mut() = None;
    }

    /// Whether a yield holds for `player`'s prompt now. One that has ended is
    /// cleared, so it cannot hold again.
    pub(crate) fn holds(&self, game: &GameState, player: PlayerId) -> bool {
        let mut held = self.0.borrow_mut();
        let holds = held.as_ref().is_some_and(|h| h.holds(game, player));
        if !holds {
            *held = None;
        }
        holds
    }
}

/// Where a priority prompt offers `Pass`, which it always does
/// (`engine::priority`).
pub(crate) fn pass_index(options: &[ChoiceOption]) -> usize {
    options
        .iter()
        .position(|o| matches!(o, ChoiceOption::Action(PriorityAction::Pass)))
        .expect("every priority prompt offers Pass")
}

/// Passes `D`'s seat's priority while its person's yield holds, and passes
/// every other decision to `D`.
pub struct AutoYield<D> {
    inner: D,
    yields: Yields,
}

impl<D: DecisionProvider> AutoYield<D> {
    pub fn new(inner: D, yields: Yields) -> Self {
        AutoYield { inner, yields }
    }
}

impl<D: DecisionProvider> DecisionProvider for AutoYield<D> {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        if matches!(context.kind, ChoiceKind::PriorityAction) && self.yields.holds(game, player) {
            return vec![pass_index(options)];
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

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.inner.seat_mode(player)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::setup_two_player_game;
    use crate::types::ids::new_object_id;
    use crate::ui::decision::ScriptedDecisionProvider;
    use crate::ui::full_control::{FullControl, FullControlSwitch};

    const PRIORITY: ChoiceContext = ChoiceContext { kind: ChoiceKind::PriorityAction };

    /// `[Pass, Cast]`: the seat has something to do, so a pass is a choice.
    fn pass_or_cast() -> Vec<ChoiceOption> {
        vec![
            ChoiceOption::Action(PriorityAction::Pass),
            ChoiceOption::Action(PriorityAction::CastSpell(new_object_id())),
        ]
    }

    /// The seat's person, asked once, casting.
    fn asked_once() -> ScriptedDecisionProvider {
        let person = ScriptedDecisionProvider::new();
        person.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
        person
    }

    #[test]
    fn until_end_of_turn_passes_this_turn_and_is_gone_the_next() {
        let mut game = setup_two_player_game();
        let yields = Yields::default();
        yields.set(&game, Yield::UntilEndOfTurn);
        let seat = AutoYield::new(asked_once(), yields);

        assert_eq!(seat.pick_n(&game, 0, &PRIORITY, &pass_or_cast(), (1, 1)), vec![0], "yielded");
        game.turn_number += 1;
        assert_eq!(seat.pick_n(&game, 0, &PRIORITY, &pass_or_cast(), (1, 1)), vec![1], "asked");
        game.turn_number -= 1;
        assert!(!seat.yields.holds(&game, 0), "an ended yield does not come back");
    }

    #[test]
    fn until_the_stack_changes_passes_while_it_is_what_it_was() {
        let mut game = setup_two_player_game();
        game.stack.push(new_object_id());
        let yields = Yields::default();
        yields.set(&game, Yield::UntilStackChanges);
        let seat = AutoYield::new(asked_once(), yields);

        assert_eq!(seat.pick_n(&game, 1, &PRIORITY, &pass_or_cast(), (1, 1)), vec![0], "yielded");
        game.stack.push(new_object_id());
        assert_eq!(seat.pick_n(&game, 1, &PRIORITY, &pass_or_cast(), (1, 1)), vec![1], "a response is wanted");
    }

    /// Set on seat 0's own turn: it passes through the opponent's, and its
    /// next turn asks.
    #[test]
    fn until_your_next_turn_passes_until_the_seat_is_active_again() {
        let mut game = setup_two_player_game();
        let yields = Yields::default();
        yields.set(&game, Yield::UntilYourNextTurn);
        let seat = AutoYield::new(asked_once(), yields);

        assert_eq!(seat.pick_n(&game, 0, &PRIORITY, &pass_or_cast(), (1, 1)), vec![0], "the rest of this turn");
        game.turn_number += 1;
        game.active_player = 1;
        assert_eq!(seat.pick_n(&game, 0, &PRIORITY, &pass_or_cast(), (1, 1)), vec![0], "the opponent's turn");
        game.turn_number += 1;
        game.active_player = 0;
        assert_eq!(seat.pick_n(&game, 0, &PRIORITY, &pass_or_cast(), (1, 1)), vec![1], "its own turn asks");
    }

    /// A yield passes priority and nothing else: blocks are still declared.
    #[test]
    fn a_yield_answers_no_prompt_but_priority() {
        let game = setup_two_player_game();
        let yields = Yields::default();
        yields.set(&game, Yield::UntilEndOfTurn);
        let person = ScriptedDecisionProvider::new();
        person.expect_pick_n(ChoiceKind::DeclareBlockers, vec![]);
        let seat = AutoYield::new(person, yields);
        let blockers = ChoiceContext { kind: ChoiceKind::DeclareBlockers };

        assert!(seat.pick_n(&game, 1, &blockers, &[ChoiceOption::Object(new_object_id())], (0, 1)).is_empty());
    }

    /// The counterweight: full control routes past the yield, and the person
    /// is asked though it holds.
    #[test]
    fn full_control_puts_a_yielded_prompt_back() {
        let game = setup_two_player_game();
        let yields = Yields::default();
        yields.set(&game, Yield::UntilEndOfTurn);
        let switch = FullControlSwitch::default();
        let seat = FullControl::new(AutoYield::new(ScriptedDecisionProvider::new(), yields), asked_once(), switch.clone());

        assert_eq!(seat.pick_n(&game, 0, &PRIORITY, &pass_or_cast(), (1, 1)), vec![0], "yielded");
        switch.set(true);
        assert_eq!(seat.pick_n(&game, 0, &PRIORITY, &pass_or_cast(), (1, 1)), vec![1], "asked");
    }
}
