//! A6g's playable PR, the engine's half: the decision log is written by the
//! engine, so it is the record of the game and not of how its seats were set
//! up (`state::decision_log`). Since SU-4 the record is the engine's text, and
//! the engine's replay plays it again.

use std::sync::{Arc, Mutex};

use mtgsim::cards::random_deck::random_deck;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::state::decision_log::{self, AnswerLine, BuiltStart, GameStart};
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::GameState;
use mtgsim::types::ids::PlayerId;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, DispatchDecisionProvider, SeatMode};
use mtgsim::ui::display::format_event_log;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;
use mtgsim::ui::replay::Replay;
use rand::SeedableRng;
use rand::rngs::StdRng;

const TURNS: u32 = 8;

/// The start of a game dealt at `seed` to `players` seats, as `fuzz_games`
/// deals one.
fn dealt(seed: u64, players: usize) -> GameStart {
    let pool = CardRegistry::performance_pool();
    let mut deck_rng = StdRng::seed_from_u64(seed);
    let decks = (0..players)
        .map(|_| random_deck(&pool, &mut deck_rng, &[], 1, 60).iter().map(|card| card.name.clone()).collect())
        .collect();
    GameStart::Dealt { seed, config: GameConfig::unrestricted(), decks }
}

/// `start` built, recording its events and writing its record into the
/// returned text.
fn recorded(start: &GameStart) -> (BuiltStart, Arc<Mutex<String>>) {
    let mut built = start.build(&CardRegistry::default_registry()).expect("a dealt start builds");
    let text = Arc::new(Mutex::new(decision_log::opening(start).iter().map(|line| format!("{line}\n")).collect::<String>()));
    let (writer, answers) = (Arc::clone(&text), Arc::new(Mutex::new(0)));
    let game = built.game_mut();
    game.state.record_events();
    game.state.log_decisions(move |game, decision| {
        let mut number = answers.lock().unwrap();
        *number += 1;
        writer.lock().unwrap().push_str(&format!("{}\n", AnswerLine::of(game, *number, decision)));
    });
    (built, text)
}

/// Its first `TURNS` turns, the opening hands before them.
fn play(built: &mut BuiltStart, dp: &dyn DecisionProvider) {
    let ran = built.game_mut().until_stopped(|game| {
        game.setup(dp)?;
        while !game.is_over() && game.state.turn_number <= TURNS {
            game.run_turn(dp)?;
        }
        Ok(())
    });
    ran.expect("the turns play");
}

/// Passes at every priority prompt and takes the least anything else allows,
/// stopping at every priority point or not.
struct Passer {
    stops: bool,
}

impl DecisionProvider for Passer {
    fn pick_n(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        (0..bounds.0.min(options.len())).collect()
    }
    fn pick_number(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, min: u64, _: u64) -> u64 {
        min
    }
    fn allocate(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, total: u64, _: &[ChoiceOption], mins: &[u64], _: Option<&[u64]>) -> Vec<u64> {
        let mut amounts = mins.to_vec();
        amounts[0] += total - mins.iter().sum::<u64>();
        amounts
    }
    fn choose_ordering(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        (0..items.len()).collect()
    }
    fn seat_mode(&self, _: PlayerId) -> SeatMode {
        SeatMode { stops_at_every_priority_point: self.stops }
    }
}

/// The owner's rule at the design review: the log is the same whatever
/// answered. Seat 0 passes everything, once letting the engine take each
/// `Pass`-only priority point and once stopping for each as full control does,
/// and both games write the same record, the engine's passes among it.
#[test]
fn a_seat_that_stops_at_every_priority_point_writes_the_log_the_engine_does() {
    let logs: Vec<String> = [false, true]
        .into_iter()
        .map(|stops| {
            let (mut built, text) = recorded(&dealt(11, 2));
            let dp = DispatchDecisionProvider::new(vec![
                Box::new(Passer { stops }),
                Box::new(ManaWindowStop::new(RandomDecisionProvider::seeded(11))),
            ]);
            play(&mut built, &dp);
            text.lock().unwrap().clone()
        })
        .collect();
    assert_eq!(logs[0], logs[1]);
    let answers = decision_log::read(&logs[0]).expect("the record reads").answers;
    let forced = |line: &&AnswerLine| line.forced && line.player == 0 && line.kind == "PriorityAction";
    assert!(answers.iter().filter(forced).count() > 10, "seat 0's passes where it could do nothing else are logged");
    assert!(answers.iter().any(|line| !line.forced), "and the choices beside them");
}

/// The record is complete: random seats play a game, and its text, read back
/// and replayed by `ui::replay`, plays the same game again, at two seats and
/// four.
#[test]
fn a_game_replayed_from_its_log_is_the_same_game() {
    for players in [2, 4] {
        let start = dealt(23, players);
        let (mut original, text) = recorded(&start);
        let agents = (0..players as u64)
            .map(|seat| Box::new(ManaWindowStop::new(RandomDecisionProvider::seeded(23 + seat))) as Box<dyn DecisionProvider>)
            .collect();
        play(&mut original, &DispatchDecisionProvider::new(agents));
        let log = decision_log::read(&text.lock().unwrap()).expect("the record reads");
        assert!(log.answers.iter().any(|line| line.kind != "PriorityAction"), "the game asked more than priority");

        let mut replayed = log.start.build(&CardRegistry::default_registry()).expect("the record's start builds");
        replayed.game_mut().state.record_events();
        let replay = Replay::of(log);
        play(&mut replayed, &replay);

        assert_eq!(replay.remaining(), 0, "every answer the record holds was asked for");
        assert_eq!(format_event_log(&replayed.game().state), format_event_log(&original.game().state), "{players} seats");
    }
}
