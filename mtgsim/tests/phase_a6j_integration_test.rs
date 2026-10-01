//! A6j: who plays a seat, as far as a priority window depends on it
//! (`codebase-state.md` item 192).
//!
//! A cast the player cancels, by stopping CR 601.2g's window with the cost
//! unpaid, looks exactly like one an agent could not pay: both rewind. A
//! window's blacklist and retry budget exist so an agent's re-picks terminate,
//! and nothing in CR 601.2 or 732 forbids a person another attempt, so a
//! person's seat takes neither and the canceled cast is offered again.

use mtgsim::cards::creatures;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{place_forest, put_in_hand, setup_two_player_game};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::mana::ManaCost;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{ScriptedDecisionProvider, SeatMode};

/// Grizzly Bears in hand and two Forests to pay for it, in player 0's main
/// phase, so player 0's list is `[Pass, Cast]` and player 1's is `Pass` alone.
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

/// Seven cancels, past the six an agent's window would charge before it
/// forced a pass, and the Bears are offered every time; then the person
/// passes, and the round ends.
#[test]
fn a_persons_canceled_cast_is_offered_again_and_charged_to_nothing() {
    let (mut game, bears) = bears_and_two_forests();
    let person = ScriptedDecisionProvider::new().with_seat_mode(SeatMode { person: true, ..SeatMode::default() });
    for _ in 0..7 {
        person.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
        person.expect_pick_n(window(bears), vec![]);
    }
    person.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);

    assert_eq!(game.run_priority_round(&person).unwrap(), PriorityResult::PhaseEnds);
    assert!(game.players[0].hand.contains(&bears), "every cast was canceled");
}

/// The agent's seat on the same board: the rejected cast is dropped for the
/// window, so what is left is `Pass` alone, which the engine takes.
#[test]
fn an_agents_rejected_cast_is_dropped_for_the_window() {
    let (mut game, bears) = bears_and_two_forests();
    let agent = ScriptedDecisionProvider::new();
    agent.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    agent.expect_pick_n(window(bears), vec![]);

    assert_eq!(game.run_priority_round(&agent).unwrap(), PriorityResult::PhaseEnds);
    assert!(game.players[0].hand.contains(&bears), "the cast was rejected");
}
