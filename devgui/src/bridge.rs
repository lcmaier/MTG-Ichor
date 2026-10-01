//! The engine's worker thread, and the seat that asks the window.
//!
//! `Game::run` calls the seat and waits for its answer, so the game gets a
//! thread of its own and the window never waits on the engine. At each decision
//! the seat builds a [`Snapshot`] and a [`Prompt`] on this thread, sends them,
//! wakes the window, and blocks until the answer comes back.

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

use mtgsim::cards::random_deck::random_deck;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameResult, GameState};
use mtgsim::types::ids::PlayerId;
use mtgsim::ui::auto_payer::AutoPayer;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, DispatchDecisionProvider, SeatMode};
use mtgsim::ui::display::format_phase;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::prompt::{Answer, Prompt};
use crate::snapshot::Snapshot;

/// The seat the window plays.
pub const WINDOW_SEAT: PlayerId = 0;

const DECK_SIZE: usize = 60;

/// `fuzz_games`' two pools.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pool {
    /// `CardRegistry::performance_pool`, the measured one.
    Performance,
    /// `CardRegistry::default_registry`: every registered card.
    Stress,
}

/// Everything that decides a game besides the window's answers.
#[derive(Clone, Debug)]
pub struct GameSetup {
    pub seed: u64,
    pub pool: Pool,
    /// Where the decision log goes; `None` keeps none.
    pub log_path: Option<PathBuf>,
}

/// What the engine thread tells the window.
#[derive(Debug)]
pub enum ToWindow {
    /// Seat 0 has a decision to make.
    Prompt { snapshot: Snapshot, prompt: Prompt },
    /// `Game::run` returned.
    Finished { snapshot: Snapshot, outcome: Outcome },
    /// The engine thread panicked: an `ask_*` validator, or an engine bug.
    Panicked { message: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Won(PlayerId),
    Draw,
    /// `Game::run` returned an error.
    Error(String),
}

/// The window's ends of the two channels, and the engine thread.
pub struct EngineHandle {
    pub from_engine: Receiver<ToWindow>,
    pub answers: Sender<Answer>,
    pub thread: JoinHandle<()>,
}

/// Start a game on its own thread. `wake` runs after every message, so a
/// window that repaints only on input still shows it.
pub fn spawn_game(setup: GameSetup, wake: Arc<dyn Fn() + Send + Sync>) -> EngineHandle {
    let (to_window, from_engine) = mpsc::channel();
    let (answers, from_window) = mpsc::channel();
    let thread = std::thread::Builder::new()
        .name("engine".to_string())
        .spawn(move || {
            let played = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                play(&setup, &to_window, from_window, &wake)
            }));
            if let Err(payload) = played {
                // The window may be gone too, which is how a closed window ends this thread.
                let _ = to_window.send(ToWindow::Panicked { message: panic_message(payload.as_ref()) });
                wake();
            }
        })
        .expect("the OS refused a thread for the engine");
    EngineHandle { from_engine, answers, thread }
}

fn play(setup: &GameSetup, to_window: &Sender<ToWindow>, from_window: Receiver<Answer>, wake: &Arc<dyn Fn() + Send + Sync>) {
    let registry = match setup.pool {
        Pool::Performance => CardRegistry::performance_pool(),
        Pool::Stress => CardRegistry::default_registry(),
    };
    // Three streams from the one seed: the decks, the shuffle, the bot.
    let mut deck_rng = StdRng::seed_from_u64(setup.seed);
    let decks: Vec<Vec<Arc<CardData>>> =
        (0..2).map(|_| random_deck(&registry, &mut deck_rng, &[], 1, DECK_SIZE)).collect();
    let log = Rc::new(RefCell::new(DecisionLog::open(setup, &decks)));
    let mut game = Game::new(GameConfig::unrestricted(), decks).expect("two decks always make a game");
    game.state.record_events();
    game.reseed(setup.seed.wrapping_add(1));

    let events_shown = Rc::new(Cell::new(0));
    let seat = GuiSeat {
        to_window: to_window.clone(),
        from_window,
        wake: Arc::clone(wake),
        events_shown: Rc::clone(&events_shown),
        log: Rc::clone(&log),
    };
    // `cli_play`'s stacks: CR 601.2g's window closes once the cost is paid.
    let dp = DispatchDecisionProvider::new(vec![
        Box::new(AutoPayer::new(ManaWindowStop::new(seat))),
        Box::new(ManaWindowStop::new(RandomDecisionProvider::seeded(setup.seed.wrapping_add(2)))),
    ]);
    let outcome = match game.setup(&dp).and_then(|()| game.run(&dp)) {
        Ok(GameResult::Winner(player)) => Outcome::Won(player),
        Ok(GameResult::Draw) => Outcome::Draw,
        Err(error) => Outcome::Error(error),
    };
    log.borrow_mut().outcome(&outcome);
    let snapshot = Snapshot::build(&game.state, events_shown.get());
    let _ = to_window.send(ToWindow::Finished { snapshot, outcome });
    wake();
}

/// Seat 0: every question goes to the window. One with a single legal answer
/// never reaches a provider: the engine answers it before asking.
struct GuiSeat {
    to_window: Sender<ToWindow>,
    from_window: Receiver<Answer>,
    wake: Arc<dyn Fn() + Send + Sync>,
    /// Shared with `play`, whose final board picks the log up from here.
    events_shown: Rc<Cell<usize>>,
    log: Rc<RefCell<DecisionLog>>,
}

impl GuiSeat {
    fn answer(&self, game: &GameState, prompt: Prompt) -> Answer {
        let snapshot = Snapshot::build(game, self.events_shown.get());
        self.events_shown.set(snapshot.events_seen);
        let kind = prompt.kind.clone();
        self.to_window
            .send(ToWindow::Prompt { snapshot, prompt })
            .expect("the window closed during the game");
        (self.wake)();
        let answer = self.from_window.recv().expect("the window closed with a prompt open");
        self.log.borrow_mut().answer(game, &kind, &answer);
        answer
    }
}

impl DecisionProvider for GuiSeat {
    fn pick_n(
        &self,
        game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        match self.answer(game, Prompt::pick_n(game, context, options, bounds)) {
            Answer::Picks(picks) => picks,
            other => panic!("a pick_n was answered with {other:?}"),
        }
    }

    fn pick_number(&self, game: &GameState, _player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        match self.answer(game, Prompt::number(game, context, min, max)) {
            Answer::Number(number) => number,
            other => panic!("a pick_number was answered with {other:?}"),
        }
    }

    fn allocate(
        &self,
        game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        let prompt = Prompt::allocate(game, context, total, buckets, per_bucket_mins, per_bucket_maxs);
        match self.answer(game, prompt) {
            Answer::Allocation(amounts) => amounts,
            other => panic!("an allocate was answered with {other:?}"),
        }
    }

    fn choose_ordering(&self, game: &GameState, _player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        match self.answer(game, Prompt::order(game, context, items)) {
            Answer::Order(order) => order,
            other => panic!("a choose_ordering was answered with {other:?}"),
        }
    }

    /// A person, who may choose a cast again after canceling it.
    fn seat_mode(&self, _player: PlayerId) -> SeatMode {
        SeatMode { person: true, ..SeatMode::default() }
    }
}

/// The seed, the decks and every answer seat 0 gave, a line each and flushed
/// as written, so a game that panics leaves its whole record. Replaying it is
/// PR 2's; keeping it now is what makes anything the spike shows reproducible.
struct DecisionLog {
    file: Option<File>,
    answers: usize,
}

impl DecisionLog {
    fn open(setup: &GameSetup, decks: &[Vec<Arc<CardData>>]) -> DecisionLog {
        let file = setup.log_path.as_ref().map(|path| {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
            }
            File::create(path).unwrap_or_else(|e| panic!("cannot create the decision log {}: {e}", path.display()))
        });
        let mut log = DecisionLog { file, answers: 0 };
        log.line(&format!("seed {}", setup.seed));
        log.line(&format!("pool {:?}", setup.pool));
        for (seat, deck) in decks.iter().enumerate() {
            // A decklist, before the game: the cards' definitions, not objects.
            let names: Vec<&str> = deck.iter().map(|card| card.name.as_str()).collect();
            log.line(&format!("deck {seat} {}", names.join("; ")));
        }
        log
    }

    /// `answer 23 [turn 3, Beginning — Draw] PriorityAction Picks([0])`.
    fn answer(&mut self, game: &GameState, kind: &str, answer: &Answer) {
        self.answers += 1;
        let when = format!("turn {}, {}", game.turn_number, format_phase(game));
        let line = format!("answer {} [{when}] {kind} {answer:?}", self.answers);
        self.line(&line);
    }

    fn outcome(&mut self, outcome: &Outcome) {
        self.line(&format!("outcome {outcome:?}"));
    }

    fn line(&mut self, text: &str) {
        if let Some(file) = &mut self.file {
            writeln!(file, "{text}").and_then(|()| file.flush()).expect("cannot write the decision log");
        }
    }
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        text.to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "the engine thread panicked with a payload that is not text".to_string()
    }
}
