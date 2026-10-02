//! A6g's playable PR, the engine's half: the decision log is written by the
//! engine, so it is the record of the game and not of how its seats were set
//! up (`state::decision_log`).

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use mtgsim::cards::random_deck::random_deck;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::state::decision_log::{LoggedAnswer, LoggedDecision};
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::GameState;
use mtgsim::types::ids::PlayerId;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, DispatchDecisionProvider, SeatMode};
use mtgsim::ui::display::format_event_log;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;
use rand::SeedableRng;
use rand::rngs::StdRng;

const TURNS: u32 = 8;

/// A dealt game at `seed`, recording its events and logging its decisions
/// into the returned list.
fn dealt(seed: u64) -> (Game, Arc<Mutex<Vec<Line>>>) {
    let registry = CardRegistry::performance_pool();
    let mut deck_rng = StdRng::seed_from_u64(seed);
    let decks = (0..2).map(|_| random_deck(&registry, &mut deck_rng, &[], 1, 60)).collect();
    let mut game = Game::new(GameConfig::unrestricted(), decks).expect("two decks make a game");
    game.reseed(seed);
    game.state.record_events();
    let lines = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&lines);
    game.state.log_decisions(move |_, decision| log.lock().unwrap().push(Line::from(decision)));
    (game, lines)
}

fn play(game: &mut Game, dp: &dyn DecisionProvider) {
    game.setup(dp).expect("setup");
    while !game.is_over() && game.state.turn_number <= TURNS {
        game.run_turn(dp).expect("turn");
    }
}

/// A logged decision, owned. The kind by its variant's name, which is all a
/// replay's check needs.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Line {
    player: PlayerId,
    kind: String,
    answer: Answer,
    forced: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Answer {
    Picks(Vec<usize>),
    Number(u64),
    Allocation(Vec<u64>),
    Order(Vec<usize>),
}

fn variant(kind: &impl std::fmt::Debug) -> String {
    let debug = format!("{kind:?}");
    debug.split(|c: char| !c.is_alphanumeric()).next().unwrap_or_default().to_string()
}

impl From<&LoggedDecision<'_>> for Line {
    fn from(decision: &LoggedDecision) -> Line {
        let answer = match decision.answer {
            LoggedAnswer::Picks(picks) => Answer::Picks(picks.to_vec()),
            LoggedAnswer::Number(number) => Answer::Number(number),
            LoggedAnswer::Allocation(amounts) => Answer::Allocation(amounts.to_vec()),
            LoggedAnswer::Order(order) => Answer::Order(order.to_vec()),
        };
        Line { player: decision.player, kind: variant(decision.kind), answer, forced: decision.forced }
    }
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
/// and both games write the same lines, the engine's passes among them.
#[test]
fn a_seat_that_stops_at_every_priority_point_writes_the_log_the_engine_does() {
    let logs: Vec<Vec<Line>> = [false, true]
        .into_iter()
        .map(|stops| {
            let (mut game, lines) = dealt(11);
            let dp = DispatchDecisionProvider::new(vec![
                Box::new(Passer { stops }),
                Box::new(ManaWindowStop::new(RandomDecisionProvider::seeded(11))),
            ]);
            play(&mut game, &dp);
            lines.lock().unwrap().clone()
        })
        .collect();
    assert_eq!(logs[0], logs[1]);
    let forced = |line: &&Line| line.forced && line.player == 0 && line.kind == "PriorityAction";
    assert!(logs[0].iter().filter(forced).count() > 10, "seat 0's passes where it could do nothing else are logged");
    assert!(logs[0].iter().any(|line| !line.forced), "and the choices beside them");
}

/// Answers every prompt with the next logged line, checking it asks the
/// logged question, and stops at every priority point so the engine's own
/// passes are asked too.
struct Replay {
    lines: Mutex<VecDeque<Line>>,
}

impl Replay {
    fn next(&self, player: PlayerId, context: &ChoiceContext) -> Answer {
        let line = self.lines.lock().unwrap().pop_front().expect("a prompt the log does not have");
        assert_eq!((line.player, line.kind.as_str()), (player, variant(&context.kind).as_str()), "the replay asked another question");
        line.answer
    }
}

impl DecisionProvider for Replay {
    fn pick_n(&self, _: &GameState, player: PlayerId, context: &ChoiceContext, _: &[ChoiceOption], _: (usize, usize)) -> Vec<usize> {
        match self.next(player, context) {
            Answer::Picks(picks) => picks,
            other => panic!("a pick_n logged as {other:?}"),
        }
    }
    fn pick_number(&self, _: &GameState, player: PlayerId, context: &ChoiceContext, _: u64, _: u64) -> u64 {
        match self.next(player, context) {
            Answer::Number(number) => number,
            other => panic!("a pick_number logged as {other:?}"),
        }
    }
    fn allocate(&self, _: &GameState, player: PlayerId, context: &ChoiceContext, _: u64, _: &[ChoiceOption], _: &[u64], _: Option<&[u64]>) -> Vec<u64> {
        match self.next(player, context) {
            Answer::Allocation(amounts) => amounts,
            other => panic!("an allocate logged as {other:?}"),
        }
    }
    fn choose_ordering(&self, _: &GameState, player: PlayerId, context: &ChoiceContext, _: &[ChoiceOption]) -> Vec<usize> {
        match self.next(player, context) {
            Answer::Order(order) => order,
            other => panic!("a choose_ordering logged as {other:?}"),
        }
    }
    fn seat_mode(&self, _: PlayerId) -> SeatMode {
        SeatMode { stops_at_every_priority_point: true }
    }
}

/// The log is complete: two random seats play a game, and its log, answering
/// every seat's every question in order, plays the same game again.
#[test]
fn a_game_replayed_from_its_log_is_the_same_game() {
    let (mut original, lines) = dealt(23);
    let agents = DispatchDecisionProvider::new(vec![
        Box::new(ManaWindowStop::new(RandomDecisionProvider::seeded(23))),
        Box::new(ManaWindowStop::new(RandomDecisionProvider::seeded(24))),
    ]);
    play(&mut original, &agents);
    let logged = lines.lock().unwrap().clone();

    let (mut replayed, _) = dealt(23);
    let replay = Replay { lines: Mutex::new(logged.iter().cloned().collect()) };
    play(&mut replayed, &replay);

    assert!(replay.lines.lock().unwrap().is_empty(), "every logged answer was asked for");
    assert_eq!(format_event_log(&replayed.state), format_event_log(&original.state));
    assert!(logged.iter().any(|line| line.kind != "PriorityAction"), "the game asked more than priority: {}", logged.len());
}
