//! SU-4, the replay (`setup-architecture.md` §7.1–§7.3): the stop that ends a
//! run from inside a prompt, the decision log's text, and the replay that
//! answers from it.

use std::cell::RefCell;

use mtgsim::cards::registry::CardRegistry;
use mtgsim::scenario::Scenario;
use mtgsim::state::game::{Game, Halt};
use mtgsim::state::game_state::GameState;
use mtgsim::types::ids::PlayerId;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider, Stop};

/// A Lightning Bolt castable at the first priority prompt, so that prompt is
/// asked rather than passed for the seat.
const BOLT_IN_HAND: &str = "hand 0: Lightning Bolt\nbattlefield: Mountain | controller 0\nbattlefield: Grizzly Bears | controller 1\n";

fn board(text: &str) -> Game {
    Scenario::parse(text).and_then(|s| s.build(&CardRegistry::default_registry())).unwrap_or_else(|r| panic!("{r}")).game
}

/// Raises its stop at the first prompt, having written the board as that
/// prompt shows it.
struct Stopper {
    stop: Stop,
    board_at_prompt: RefCell<Option<String>>,
}

impl Stopper {
    fn new(stop: Stop) -> Stopper {
        Stopper { stop, board_at_prompt: RefCell::new(None) }
    }

    fn stop_here(&self, game: &GameState) -> ! {
        *self.board_at_prompt.borrow_mut() = Some(Scenario::write(game).to_string());
        self.stop.clone().raise()
    }
}

impl DecisionProvider for Stopper {
    fn pick_n(&self, game: &GameState, _: PlayerId, _: &ChoiceContext, _: &[ChoiceOption], _: (usize, usize)) -> Vec<usize> {
        self.stop_here(game)
    }
    fn pick_number(&self, game: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, _: u64) -> u64 {
        self.stop_here(game)
    }
    fn allocate(&self, game: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, _: &[ChoiceOption], _: &[u64], _: Option<&[u64]>) -> Vec<u64> {
        self.stop_here(game)
    }
    fn choose_ordering(&self, game: &GameState, _: PlayerId, _: &ChoiceContext, _: &[ChoiceOption]) -> Vec<usize> {
        self.stop_here(game)
    }
}

/// A stop ends the run inside its prompt, and `until_stopped` returns it.
/// The board is the one the prompt showed, and every run entry refuses the
/// game after, since the frames unwound held work in flight.
#[test]
fn a_stopped_game_is_read_and_never_continued() {
    let mut game = board(BOLT_IN_HAND);
    let stopper = Stopper::new(Stop::Superseded);
    assert_eq!(game.until_stopped(|game| game.resume(&stopper)), Err(Halt::Stopped(Stop::Superseded)));
    assert_eq!(Some(Scenario::write(&game.state).to_string()), stopper.board_at_prompt.take(), "the board its prompt showed");

    let seats = ScriptedDecisionProvider::new();
    let entries = [game.run_turn(&seats), game.setup(&seats), game.state.run_priority_round(&seats).map(|_| ())];
    for refused in entries {
        assert!(refused.is_err_and(|why| why.contains("a stopped game is read, never continued")));
    }
}

/// Only a stop is caught: an engine bug's panic passes through
/// `until_stopped` as it would have, payload and all.
#[test]
fn a_panic_that_is_not_a_stop_passes_through() {
    struct Bug;
    impl DecisionProvider for Bug {
        fn pick_n(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: &[ChoiceOption], _: (usize, usize)) -> Vec<usize> {
            panic!("an engine bug")
        }
        fn pick_number(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, _: u64) -> u64 {
            panic!("an engine bug")
        }
        fn allocate(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, _: &[ChoiceOption], _: &[u64], _: Option<&[u64]>) -> Vec<u64> {
            panic!("an engine bug")
        }
        fn choose_ordering(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: &[ChoiceOption]) -> Vec<usize> {
            panic!("an engine bug")
        }
    }
    let mut game = board(BOLT_IN_HAND);
    let payload = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| game.until_stopped(|game| game.resume(&Bug)))).unwrap_err();
    assert_eq!(payload.downcast_ref::<&str>(), Some(&"an engine bug"));
}
