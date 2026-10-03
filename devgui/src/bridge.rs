//! The engine's worker thread, and the seats that ask the window.
//!
//! The window plays every seat (`setup-architecture.md` §7's "Seats").
//! `Game::run` calls a seat and waits for its answer, so the game gets a
//! thread of its own and the window never waits on the engine. At each
//! decision the seat builds a [`Snapshot`] and a [`Prompt`] on this thread,
//! sends them, wakes the window, and blocks until the answer comes back.

use std::any::Any;
use std::cell::Cell;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;

use mtgsim::cards::random_deck::random_deck;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::state::decision_log::{self, AnswerLine, GameStart, LoggedDecision};
use mtgsim::state::game::Halt;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameResult, GameState};
use mtgsim::types::ids::PlayerId;
use mtgsim::ui::auto_payer::AutoPayer;
use mtgsim::ui::auto_yield::{AutoYield, Yield, Yields, pass_index};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, DispatchDecisionProvider};
use mtgsim::ui::full_control::{FullControl, FullControlSwitch};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::prompt::{Answer, Prompt, Reply};
use crate::snapshot::Snapshot;

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
    /// A dealt game's seats, dealt as `fuzz_games --players` deals them; a
    /// scenario states its own.
    pub players: usize,
    /// Where the decision log goes; `None` keeps none.
    pub log_path: Option<PathBuf>,
    /// A board to start from instead of dealt decks, read again at each
    /// start, so Reload picks up an edit. Its seed is `seed`, which the
    /// command line's `--seed` overrides.
    pub scenario: Option<PathBuf>,
}

/// What the engine thread tells the window.
#[derive(Debug)]
pub enum ToWindow {
    /// The seat `prompt` names has a decision to make, and holds this yield
    /// while it does.
    Prompt { snapshot: Snapshot, prompt: Prompt, yielding: Option<Yield> },
    /// `Game::run` returned.
    Finished { snapshot: Snapshot, outcome: Outcome },
    /// The engine thread panicked: an `ask_*` validator, or an engine bug.
    Panicked { message: String },
    /// The scenario did not load: the file's line, and what to change.
    Refused { message: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Won(PlayerId),
    Draw,
    /// `Game::run` returned an error.
    Error(String),
}

/// The window's ends of the two channels, the engine thread, and full
/// control's switch above every seat, which the window flips from its own
/// thread and a seat reads at its next prompt (`mtgsim::ui::full_control`).
pub struct EngineHandle {
    pub from_engine: Receiver<ToWindow>,
    pub answers: Sender<Reply>,
    pub thread: JoinHandle<()>,
    pub full_control: FullControlSwitch,
}

/// Start a game on its own thread. `wake` runs after every message, so a
/// window that repaints only on input still shows it.
pub fn spawn_game(setup: GameSetup, wake: Arc<dyn Fn() + Send + Sync>) -> EngineHandle {
    let (to_window, from_engine) = mpsc::channel();
    let (answers, from_window) = mpsc::channel();
    let full_control = FullControlSwitch::default();
    let switch = full_control.clone();
    let thread = std::thread::Builder::new()
        .name("engine".to_string())
        .spawn(move || {
            let played = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                play(&setup, &to_window, from_window, switch, &wake)
            }));
            if let Err(payload) = played
                && !payload.is::<WindowGone>()
            {
                let _ = to_window.send(ToWindow::Panicked { message: panic_message(payload.as_ref()) });
                wake();
            }
        })
        .expect("the OS refused a thread for the engine");
    EngineHandle { from_engine, answers, thread, full_control }
}

fn play(
    setup: &GameSetup,
    to_window: &Sender<ToWindow>,
    from_window: Receiver<Reply>,
    full_control: FullControlSwitch,
    wake: &Arc<dyn Fn() + Send + Sync>,
) {
    let refuse = |message: String| {
        let _ = to_window.send(ToWindow::Refused { message });
        wake();
    };
    let start = match start_of(setup) {
        Ok(start) => start,
        Err(message) => return refuse(message),
    };
    let mut built = match start.build(&CardRegistry::default_registry()) {
        Ok(built) => built,
        Err(message) => return refuse(message),
    };
    // The engine writes the log, every seat's answers and its own passes, so
    // the record is the game's whatever answered at each seat.
    let log = Arc::new(Mutex::new(DecisionLog::open(setup, &start)));
    let writer = Arc::clone(&log);
    let game = built.game_mut();
    game.state.log_decisions(move |game, decision| locked(&writer).answer(game, decision));
    game.state.record_events();

    let events_logged = Rc::new(Cell::new(0));
    let from_window = Rc::new(from_window);
    // Every seat the window's, each with `cli_play`'s stack and a yield of
    // its own: CR 601.2g's window closes once the cost is paid, and a yield
    // passes for the person, all of it off under full control's one switch.
    let seats: Vec<Box<dyn DecisionProvider>> = (0..built.game().state.players.len())
        .map(|_| {
            let yields = Yields::default();
            let seat = GuiSeat {
                to_window: to_window.clone(),
                from_window: Rc::clone(&from_window),
                wake: Arc::clone(wake),
                events_logged: Rc::clone(&events_logged),
                yields: yields.clone(),
            };
            let decorated = AutoYield::new(AutoPayer::new(ManaWindowStop::new(seat.clone())), yields.clone());
            let routed = FullControl::new(decorated, seat, full_control.clone()).superseding(yields);
            Box::new(routed) as Box<dyn DecisionProvider>
        })
        .collect();
    let dp = DispatchDecisionProvider::new(seats);
    // A scenario's setup actions play first, every seat's, so the window's
    // first prompt comes once they have built their stack.
    let outcome = match built.play(&dp) {
        Ok(GameResult::Winner(player)) => Outcome::Won(player),
        Ok(GameResult::Draw) => Outcome::Draw,
        Err(Halt::Error(error)) => Outcome::Error(error),
        // A setup line the engine cannot play is the file's to fix, as a
        // line the loader refuses is.
        Err(Halt::Stopped(stop)) => {
            locked(&log).outcome(&decision_log::Outcome::Stopped(stop.to_string()));
            return refuse(stop.to_string());
        }
    };
    let ended = match &outcome {
        Outcome::Won(player) => decision_log::Outcome::Won(*player),
        Outcome::Draw => decision_log::Outcome::Draw,
        Outcome::Error(error) => decision_log::Outcome::Error(error.clone()),
    };
    locked(&log).outcome(&ended);
    let snapshot = Snapshot::build(&built.game().state, events_logged.get());
    let _ = to_window.send(ToWindow::Finished { snapshot, outcome });
    wake();
}

/// Where `setup`'s game begins: a scenario's file read now, so Reload picks
/// up an edit, at the setup's seed; or decks dealt from the seed as
/// `fuzz_games` deals them, so a fuzz game's printed seed deals the same game
/// here, the decks off the seed itself and the shuffle off `RandomStreams`.
fn start_of(setup: &GameSetup) -> Result<GameStart, String> {
    match &setup.scenario {
        Some(path) => {
            let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            Ok(GameStart::Scenario { path: path.display().to_string(), seed: setup.seed, text })
        }
        None => {
            let pool = match setup.pool {
                Pool::Performance => CardRegistry::performance_pool(),
                Pool::Stress => CardRegistry::default_registry(),
            };
            let mut deck_rng = StdRng::seed_from_u64(setup.seed);
            let mut dealt = || random_deck(&pool, &mut deck_rng, &[], 1, DECK_SIZE).iter().map(|card| card.name.clone()).collect();
            let decks = (0..setup.players).map(|_| dealt()).collect();
            Ok(GameStart::Dealt { seed: setup.seed, config: GameConfig::unrestricted(), decks })
        }
    }
}

/// A seat the window plays: every question that reaches it goes to the
/// window, over the channel every seat shares. One with a single legal answer
/// never does unless full control is on: the engine answers it before asking.
/// Each seat is two values, the provider at the bottom of its decorated stack
/// and full control's raw one, which share its yield.
#[derive(Clone)]
struct GuiSeat {
    to_window: Sender<ToWindow>,
    from_window: Rc<Receiver<Reply>>,
    wake: Arc<dyn Fn() + Send + Sync>,
    /// Shared with `play`, whose final board picks the log up from here.
    events_logged: Rc<Cell<usize>>,
    /// This seat's yield, which the window sets and this seat's `AutoYield`
    /// reads.
    yields: Yields,
}

impl GuiSeat {
    /// Send `prompt` and wait for the window's reply: an answer, or at a
    /// priority prompt a yield. "Stop yielding" is carried out here, and the
    /// prompt stays open.
    fn ask(&self, game: &GameState, prompt: Prompt) -> Reply {
        let snapshot = Snapshot::build(game, self.events_logged.get());
        self.events_logged.set(snapshot.events_logged);
        let yielding = self.yields.holding(game, prompt.player);
        if self.to_window.send(ToWindow::Prompt { snapshot, prompt, yielding }).is_err() {
            std::panic::resume_unwind(Box::new(WindowGone));
        }
        (self.wake)();
        loop {
            match self.from_window.recv() {
                Ok(Reply::StopYielding) => self.yields.clear(),
                Ok(reply) => return reply,
                Err(_) => std::panic::resume_unwind(Box::new(WindowGone)),
            }
        }
    }

    fn answer(&self, game: &GameState, prompt: Prompt) -> Answer {
        match self.ask(game, prompt) {
            Reply::Answer(answer) => answer,
            other => panic!("{other:?} answered a prompt that is not a priority prompt"),
        }
    }
}

impl DecisionProvider for GuiSeat {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        match self.ask(game, Prompt::pick_n(game, player, context, options, bounds)) {
            Reply::Answer(Answer::Picks(picks)) => picks,
            // A yield answers the priority prompt it is set at with a pass;
            // the window offers a stack yield only over a stack, the one the
            // seat refuses.
            Reply::Yield(until)
                if matches!(context.kind, ChoiceKind::PriorityAction) && self.yields.set(game, until) =>
            {
                vec![pass_index(options)]
            }
            other => panic!("a pick_n was answered with {other:?}"),
        }
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        match self.answer(game, Prompt::number(game, player, context, min, max)) {
            Answer::Number(number) => number,
            other => panic!("a pick_number was answered with {other:?}"),
        }
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
        let prompt = Prompt::allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs);
        match self.answer(game, prompt) {
            Answer::Allocation(amounts) => amounts,
            other => panic!("an allocate was answered with {other:?}"),
        }
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        match self.answer(game, Prompt::order(game, player, context, items)) {
            Answer::Order(order) => order,
            other => panic!("a choose_ordering was answered with {other:?}"),
        }
    }
}

/// What a game whose window has gone unwinds with: the window closed, or
/// Reload started another game. `resume_unwind` raises it without the panic
/// hook, so the normal end of a superseded game prints nothing, and
/// `spawn_game` sends no `Panicked` for it.
struct WindowGone;

/// The game's record in the engine's text (`mtgsim::state::decision_log`), a
/// line each and flushed as written, so a game that panics leaves its whole
/// record, and every game the window plays is one a later replay reads.
struct DecisionLog {
    file: Option<File>,
    answers: usize,
}

impl DecisionLog {
    fn open(setup: &GameSetup, start: &GameStart) -> DecisionLog {
        let file = setup.log_path.as_ref().map(|path| {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
            }
            File::create(path).unwrap_or_else(|e| panic!("cannot create the decision log {}: {e}", path.display()))
        });
        let mut log = DecisionLog { file, answers: 0 };
        for line in decision_log::opening(start) {
            log.line(&line);
        }
        log
    }

    fn answer(&mut self, game: &GameState, decision: &LoggedDecision) {
        self.answers += 1;
        let line = AnswerLine::of(game, self.answers, decision).to_string();
        self.line(&line);
    }

    fn outcome(&mut self, outcome: &decision_log::Outcome) {
        self.line(&outcome.to_string());
    }

    fn line(&mut self, text: &str) {
        if let Some(file) = &mut self.file {
            writeln!(file, "{text}").and_then(|()| file.flush()).expect("cannot write the decision log");
        }
    }
}

/// The log behind its lock, which a panic while writing a line leaves as it was.
fn locked(log: &Mutex<DecisionLog>) -> MutexGuard<'_, DecisionLog> {
    log.lock().unwrap_or_else(PoisonError::into_inner)
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
