//! A6j and item 193: a priority window offers a rejected action again, and
//! says that it rejected it (`codebase-state.md` items 192 and 193).
//!
//! A cast the player cancels, by stopping CR 601.2g's window with the cost
//! unpaid, looks exactly like one an agent could not pay: both are reversed
//! (CR 732.1). Nothing in CR 601.2 or 732 forbids another attempt (CR 732.2
//! lets the player redo it), so every seat is offered the action again and
//! charged nothing, and the re-ask names the action it reversed. Not choosing
//! it again is an agent's policy (`ui::random`), not the engine's.

use mtgsim::cards::creatures;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{RejectionRecorder, place_forest, put_in_hand, setup_two_player_game};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaCost;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption, Rejection};
use mtgsim::ui::decision::{DecisionProvider, PriorityAction, ScriptedDecisionProvider};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

/// Grizzly Bears in hand and two Forests to pay for it exactly, in player
/// 0's main phase, so player 0's list is `[Pass, Cast]` and player 1's is
/// `Pass` alone.
fn bears_and_two_forests() -> (GameState, ObjectId) {
    let mut game = setup_two_player_game();
    let bears = put_in_hand(&mut game, creatures::grizzly_bears(), 0);
    place_forest(&mut game, 0);
    place_forest(&mut game, 0);
    (game, bears)
}

/// The window for the Bears' {1}{G}; the script matches the kind alone.
fn window(bears: ObjectId) -> ChoiceKind {
    ChoiceKind::ManaAbilityWindow { spell_or_ability_id: bears, remaining_cost: ManaCost::zero() }
}

/// Seven cancels, and the Bears are offered every time, each re-ask naming
/// the cast it reversed; then the player passes, and the round ends.
#[test]
fn a_canceled_cast_is_offered_again_and_charged_to_nothing() {
    let (mut game, bears) = bears_and_two_forests();
    let script = ScriptedDecisionProvider::new();
    for _ in 0..7 {
        script.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
        script.expect_pick_n(window(bears), vec![]);
    }
    script.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    let seat = RejectionRecorder::new(script);

    assert_eq!(game.run_priority_round(&seat).unwrap(), PriorityResult::PhaseEnds);
    assert!(game.players[0].hand.contains(&bears), "every cast was canceled");
    let reversed = Some(Rejection::Reversed(PriorityAction::CastSpell(bears)));
    let mut expected = vec![None];
    expected.extend(std::iter::repeat_n(reversed, 7));
    assert_eq!(seat.rejected_at("PriorityAction"), expected, "the first ask rejects nothing, and each re-ask names the cast");
}

/// The cast from hand, under `ManaWindowStop` as a client stacks it: stopped
/// with {1}{G} unpaid, so CR 732.1 reverses it; the re-ask says so, and the
/// second cast taps both Forests, which pay exactly, and goes on the stack.
#[test]
fn a_reversed_cast_is_named_on_the_re_ask_and_can_be_cast_again() {
    let (mut game, bears) = bears_and_two_forests();
    let script = ScriptedDecisionProvider::new();
    script.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    script.expect_pick_n(window(bears), vec![]);
    script.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    script.expect_pick_n(window(bears), vec![0]);
    script.expect_pick_n(window(bears), vec![0]);
    let seat = RejectionRecorder::new(ManaWindowStop::new(script));

    assert_eq!(game.run_priority_round(&seat).unwrap(), PriorityResult::ActionTaken);
    assert!(game.stack.contains(&bears), "the second cast was paid");
    assert_eq!(
        seat.rejected_at("PriorityAction"),
        [None, Some(Rejection::Reversed(PriorityAction::CastSpell(bears)))],
    );
    assert_eq!(seat.rejected_at("ManaAbilityWindow"), [None, None, None], "a window is no re-ask");
}

/// Chooses the cast and stops its window, whatever it is told.
struct Stubborn;

impl DecisionProvider for Stubborn {
    fn pick_n(&self, _: &GameState, _: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], _: (usize, usize)) -> Vec<usize> {
        match context.kind {
            ChoiceKind::PriorityAction => vec![options.len() - 1],
            _ => Vec::new(),
        }
    }
    fn pick_number(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, min: u64, _: u64) -> u64 {
        min
    }
    fn allocate(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, _: &[ChoiceOption], mins: &[u64], _: Option<&[u64]>) -> Vec<u64> {
        mins.to_vec()
    }
    fn choose_ordering(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        (0..items.len()).collect()
    }
}

/// The hang guard: a provider that keeps choosing what it was told failed
/// ends the game with an error at the thousandth rejection, which no person
/// reaches.
#[test]
fn a_seat_that_keeps_choosing_a_rejected_action_ends_the_game_with_an_error() {
    let (mut game, _) = bears_and_two_forests();

    let error = game.run_priority_round(&Stubborn).unwrap_err();
    assert!(error.contains("rejected 1000 times"), "{error}");
}
