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
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;

use mtgsim::cards::random_deck::random_deck;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::scenario::Scenario;
use mtgsim::state::decision_log::{self, AnswerLine, GameStart, Log, LoggedDecision};
use mtgsim::state::game::Halt;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameResult, GameState};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::ui::auto_payer::AutoPayer;
use mtgsim::ui::auto_yield::{AutoYield, Yield, Yields, pass_index};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, DispatchDecisionProvider, SeatMode, Stop};
use mtgsim::ui::full_control::{FullControl, FullControlSwitch};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::replay::{Replay, ReplayControl};
use mtgsim::ui::why::{Why, why};
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::prompt::{Answer, Prompt, Reply};
use crate::save::{self, Save};
use crate::snapshot::Snapshot;
use crate::view_model::Refusal;

const DECK_SIZE: usize = 60;

/// `fuzz_games`' two pools.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pool {
    /// `CardRegistry::performance_pool`, the measured one.
    Performance,
    /// `CardRegistry::default_registry`: every registered card.
    Stress,
}

/// How a game begins, as the command line or the editor's Play asks for one:
/// everything that decides it besides the window's answers.
#[derive(Clone, Debug)]
pub struct GameSetup {
    /// `--seed`'s, or the clock's for a dealt game. `None` plays a scenario
    /// at the seed its file says, read at each start, so Reload picks up an
    /// edited `seed` line (`codebase-state.md` item 200).
    pub seed: Option<u64>,
    pub pool: Pool,
    /// A dealt game's seats, dealt as `fuzz_games --players` deals them; a
    /// scenario states its own.
    pub players: usize,
    /// A board to start from instead of dealt decks, read again at each
    /// start, so Reload picks up an edit.
    pub scenario: Option<PathBuf>,
}

impl GameSetup {
    /// Where the game begins: a scenario's file read now, at the setup's seed
    /// or else its own; or decks dealt from the seed as `fuzz_games` deals
    /// them, so a fuzz game's printed seed deals the same game here, the
    /// decks off the seed itself and the shuffle off `RandomStreams`.
    pub fn start(&self) -> Result<GameStart, String> {
        match &self.scenario {
            Some(path) => {
                let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
                // A file that does not parse is refused as the game is built.
                let own = || Scenario::parse(&text).map_or(0, |scenario| scenario.seed);
                let seed = self.seed.unwrap_or_else(own);
                Ok(GameStart::Scenario { path: path.display().to_string(), seed, text })
            }
            None => {
                let pool = match self.pool {
                    Pool::Performance => CardRegistry::performance_pool(),
                    Pool::Stress => CardRegistry::default_registry(),
                };
                let seed = self.seed.unwrap_or_default();
                let mut deck_rng = StdRng::seed_from_u64(seed);
                let mut dealt = || random_deck(&pool, &mut deck_rng, &[], 1, DECK_SIZE).iter().map(|card| card.name.clone()).collect();
                let decks = (0..self.players).map(|_| dealt()).collect();
                Ok(GameStart::Dealt { seed, config: GameConfig::unrestricted(), decks })
            }
        }
    }
}

/// A game for the engine's thread: where it begins, the answers it replays
/// before the seats are asked, and the record it writes, if any.
pub struct Play {
    pub start: GameStart,
    /// The line to replay from the start: a rebuild's, to the place the
    /// window moves to. Empty plays from the start, a scenario's setup
    /// actions with it.
    pub line: Vec<AnswerLine>,
    /// The layer memo's debug audits stay on while the line replays. Off
    /// for a rebuild, whose answers this build audited as the window gave
    /// them (`setup-architecture.md` §7.1); back on at the hand-over.
    pub audited: bool,
    pub record: Option<Writer>,
    /// The engine that wrote a loaded line, which a divergence names beside
    /// this one when they differ; `None` within a session.
    pub written_by: Option<String>,
    /// The object the why panel shows, whose why each question carries: the
    /// panel outlasts a rebuild, as full control does.
    pub watching: Option<ObjectId>,
}

impl Play {
    /// `start`'s game, played from its start and recorded nowhere.
    pub fn new(start: GameStart) -> Play {
        Play { start, line: Vec::new(), audited: true, record: None, written_by: None, watching: None }
    }
}

/// What the engine thread tells the window.
#[derive(Debug)]
pub enum ToWindow {
    /// The seat `prompt` names has a decision to make, and holds this yield
    /// while it does. `why` is the why of the object the panel shows, at this
    /// question, boxed: the variant is the enum's largest already.
    Prompt { snapshot: Snapshot, prompt: Prompt, yielding: Option<Yield>, why: Option<Box<Why>> },
    /// The why the window asked for at the open question
    /// (`setup-architecture.md` §7c).
    Why(Why),
    /// `Game::run` returned.
    Finished { snapshot: Snapshot, outcome: Outcome },
    /// The engine thread panicked: an `ask_*` validator, or an engine bug.
    Panicked { message: String },
    /// The game did not start: the scenario's file, or a loaded record's
    /// start, does not build, or a setup line cannot be played.
    Refused { refusal: Refusal, message: String },
    /// A replay stopped at an answer this build does not take, `message`
    /// saying where and why, and plays on from the one before it: a line
    /// `replaying` answers long, replayed again.
    Diverged { message: String, replaying: usize },
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
pub fn spawn_game(game: Play, wake: Arc<dyn Fn() + Send + Sync>) -> EngineHandle {
    let (to_window, from_engine) = mpsc::channel();
    let (answers, from_window) = mpsc::channel();
    let full_control = FullControlSwitch::default();
    let switch = full_control.clone();
    let thread = std::thread::Builder::new()
        .name("engine".to_string())
        .spawn(move || {
            let played = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                play(game, &to_window, from_window, switch, &wake)
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
    game: Play,
    to_window: &Sender<ToWindow>,
    from_window: Receiver<Reply>,
    full_control: FullControlSwitch,
    wake: &Arc<dyn Fn() + Send + Sync>,
) {
    let Play { start, mut line, mut audited, record, written_by, watching } = game;
    // A loaded record's start may name what this build no longer has.
    let refusal = if written_by.is_some() { Refusal::Load } else { Refusal::Scenario };
    let refuse = |message: String| {
        let _ = to_window.send(ToWindow::Refused { refusal, message });
        wake();
    };
    // The engine writes the log, every seat's answers and its own passes, so
    // the record is the game's whatever answered at each seat.
    let build = || {
        let mut built = start.build(&CardRegistry::default_registry())?;
        if let Some(record) = record.clone() {
            built.game_mut().state.log_decisions(move |game, decision| record.answer(game, decision));
        }
        built.game_mut().state.record_events();
        Ok::<_, String>(built)
    };
    let mut built = match build() {
        Ok(built) => built,
        Err(message) => return refuse(message),
    };

    let events_logged = Rc::new(Cell::new(0));
    let watching = Rc::new(Cell::new(watching));
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
                record: record.clone(),
                watching: Rc::clone(&watching),
            };
            let decorated = AutoYield::new(AutoPayer::new(ManaWindowStop::new(seat.clone())), yields.clone());
            let routed = FullControl::new(decorated, seat, full_control.clone()).superseding(yields);
            Box::new(routed) as Box<dyn DecisionProvider>
        })
        .collect();
    let dp = DispatchDecisionProvider::new(seats);
    let ended = |outcome: decision_log::Outcome| {
        if let Some(record) = &record {
            record.outcome(&outcome);
        }
    };
    let played = loop {
        if line.is_empty() {
            if let Some(record) = &record {
                record.hand_over();
            }
            // A scenario's setup actions play first, every seat's, so the
            // window's first prompt comes once they have built their stack.
            break built.play(&dp);
        }
        let seats = HandOver { seats: &dp, record: record.clone(), handed: Cell::new(false) };
        let replay = match &written_by {
            Some(engine) => {
                let log = Log { engine: engine.clone(), start: start.clone(), answers: line.clone(), outcome: None };
                Replay::of(log)
            }
            None => Replay::new(line.clone()),
        };
        let replay = replay.then(&seats);
        if let Some(record) = &record {
            record.replaying(replay.control());
        }
        if !audited {
            built.game().state.pause_layer_audit();
        }
        // A line this build no longer plays says where it stopped and plays
        // on from the answer before it. A stopped game is never continued
        // (decision 1), so it is built again and replayed that far, its
        // audits paused: they have just checked those answers.
        match built.replay(&replay) {
            Err(Halt::Stopped(Stop::Diverged { line: at, why })) => {
                let message = Stop::Diverged { line: at, why }.to_string();
                let _ = to_window.send(ToWindow::Diverged { message, replaying: at - 1 });
                wake();
                if let Some(record) = &record {
                    record.diverged(at - 1);
                }
                line.truncate(at - 1);
                audited = false;
                built = match build() {
                    Ok(built) => built,
                    Err(message) => return refuse(message),
                };
            }
            played => break played,
        }
    };
    let outcome = match played {
        Ok(GameResult::Winner(player)) => Outcome::Won(player),
        Ok(GameResult::Draw) => Outcome::Draw,
        Err(Halt::Error(error)) => Outcome::Error(error),
        // A newer rebuild took the record over; this game is nobody's.
        Err(Halt::Stopped(Stop::Superseded)) => return,
        // A setup line the engine cannot play is the file's to fix, as a
        // line the loader refuses is.
        Err(Halt::Stopped(stop)) => {
            ended(decision_log::Outcome::Stopped(stop.to_string()));
            return refuse(stop.to_string());
        }
    };
    ended(match &outcome {
        Outcome::Won(player) => decision_log::Outcome::Won(*player),
        Outcome::Draw => decision_log::Outcome::Draw,
        Outcome::Error(error) => decision_log::Outcome::Error(error.clone()),
    });
    let snapshot = Snapshot::build(&built.game().state, events_logged.get());
    let _ = to_window.send(ToWindow::Finished { snapshot, outcome });
    wake();
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
    /// The game's record, which marks each place the window is asked.
    record: Option<Writer>,
    /// The object the why panel shows, shared by every seat: the window has
    /// one panel.
    watching: Rc<Cell<Option<ObjectId>>>,
}

impl GuiSeat {
    /// Send `prompt` and wait for the window's reply: an answer, or at a
    /// priority prompt a yield. "Stop yielding" and a why are carried out
    /// here, and the prompt stays open.
    fn ask(&self, game: &GameState, prompt: Prompt) -> Reply {
        let snapshot = Snapshot::build(game, self.events_logged.get());
        self.events_logged.set(snapshot.events_logged);
        if let Some(record) = &self.record {
            record.window_asked();
        }
        let yielding = self.yields.holding(game, prompt.player);
        let why = self.watching.get().map(|id| Box::new(why(game, id)));
        if self.to_window.send(ToWindow::Prompt { snapshot, prompt, yielding, why }).is_err() {
            std::panic::resume_unwind(Box::new(WindowGone));
        }
        (self.wake)();
        loop {
            match self.from_window.recv() {
                Ok(Reply::StopYielding) => self.yields.clear(),
                Ok(Reply::Why(about)) => self.answer_why(game, about),
                Ok(reply) => return reply,
                Err(_) => std::panic::resume_unwind(Box::new(WindowGone)),
            }
        }
    }

    /// The panel shows `about` from now on, its why answered at once from
    /// the board at this question; `None` closes it.
    fn answer_why(&self, game: &GameState, about: Option<ObjectId>) {
        self.watching.set(about);
        let Some(id) = about else { return };
        if self.to_window.send(ToWindow::Why(why(game, id))).is_err() {
            std::panic::resume_unwind(Box::new(WindowGone));
        }
        (self.wake)();
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

/// The seats behind a replay: the replay's first question to them, or its
/// first look at their mode, is where it hands the game over, so the record
/// writes the line it held before anything new is answered.
struct HandOver<'a> {
    seats: &'a dyn DecisionProvider,
    record: Option<Writer>,
    handed: Cell<bool>,
}

impl HandOver<'_> {
    fn seats(&self) -> &dyn DecisionProvider {
        if !self.handed.replace(true)
            && let Some(record) = &self.record
        {
            record.hand_over();
        }
        self.seats
    }
}

impl DecisionProvider for HandOver<'_> {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        self.seats().pick_n(game, player, context, options, bounds)
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.seats().pick_number(game, player, context, min, max)
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
        self.seats().allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.seats().choose_ordering(game, player, context, items)
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.seats().seat_mode(player)
    }
}

/// What a game whose window has gone unwinds with: the window closed, or
/// Reload started another game. `resume_unwind` raises it without the panic
/// hook, so the normal end of a superseded game prints nothing, and
/// `spawn_game` sends no `Panicked` for it.
struct WindowGone;

/// A game's record on disk, in the engine's text: its decision log, the line
/// of play the window is on, and beside it its save, the journal of every
/// line the session played (`setup-architecture.md` §7.2, decision 4). Each
/// line is written as it comes, so a game that panics leaves its whole
/// record, and every game the window plays is one a later replay reads.
///
/// One engine thread writes it at a time, a [`Writer`]. A rebuild's thread
/// takes it over from the one before, whose writes are dropped from then on
/// and whose replay stops at its next answer, and it holds the answers it
/// replays until it hands the game to the seats, where it writes the log
/// again: a replay superseded on its way writes nothing.
pub struct Record {
    pub save: Save,
    log: File,
    log_path: PathBuf,
    journal: File,
    /// The writer that may write now, by number.
    writer: u64,
    /// That writer's replay, which the next writer supersedes.
    replay: Option<Arc<ReplayControl>>,
    /// That writer's replayed answers, in the log's words, until it hands over.
    replayed: Vec<String>,
    /// That writer has handed the game to the seats.
    live: bool,
    /// The answers on the log's line, which number the next.
    logged: usize,
    /// Why the record stopped being written, if it has.
    pub failed: Option<String>,
}

/// Which of a record's two files a line goes to.
enum RecordFile {
    Log,
    Save,
}

impl Record {
    /// A record for `save`'s game at `log_path`, its folder made, and the
    /// save beside it written to its last entry. Made on the window's thread
    /// before the game starts, so a folder that cannot be written is refused
    /// in the window (`codebase-state.md` item 200).
    pub fn create(save: Save, log_path: PathBuf) -> Result<Record, String> {
        let create = |path: &Path, what: &str| {
            let made = path.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|()| File::create(path));
            made.map_err(|e| format!("cannot create the {what} {}: {e}", path.display()))
        };
        let log = create(&log_path, "decision log")?;
        let save_path = save::path_for(&log_path);
        let mut journal = create(&save_path, "save")?;
        journal.write_all(save.text().as_bytes()).map_err(|e| format!("cannot write the save {}: {e}", save_path.display()))?;
        let record = Record { save, log, log_path, journal, writer: 0, replay: None, replayed: Vec::new(), live: false, logged: 0, failed: None };
        Ok(record)
    }

    /// Shut the writer writing now and supersede its replay; the next
    /// writer's number.
    pub fn shut(&mut self) -> u64 {
        self.writer += 1;
        if let Some(replay) = self.replay.take() {
            replay.supersede();
        }
        (self.replayed, self.live, self.logged) = (Vec::new(), false, 0);
        self.writer
    }

    /// The window's line moved to `place`: an undo, a savestate, back to
    /// where it was.
    pub fn move_to(&mut self, place: usize) {
        let line = self.save.move_to(place);
        self.write(RecordFile::Save, &line);
    }

    /// A savestate at the place the window is asked, named `name`. Written
    /// at the click: the engine's thread waits at that prompt meanwhile.
    pub fn savestate(&mut self, name: String) {
        let line = self.save.savestate(name);
        self.write(RecordFile::Save, &line);
    }

    /// `line` at the end of a file. A write that fails stops the record
    /// there, and the header says why.
    fn write(&mut self, to: RecordFile, line: &str) {
        if self.failed.is_some() {
            return;
        }
        let (file, path) = match to {
            RecordFile::Log => (&mut self.log, self.log_path.clone()),
            RecordFile::Save => (&mut self.journal, save::path_for(&self.log_path)),
        };
        if let Err(e) = writeln!(file, "{line}") {
            self.failed = Some(format!("cannot write {}: {e}", path.display()));
        }
    }

    /// The log written again from its start: the line the writer replayed.
    fn rewrite_log(&mut self) {
        let mut text: String = decision_log::opening(&self.save.start).into_iter().map(|line| line + "\n").collect();
        text.extend(self.replayed.drain(..).map(|line| line + "\n"));
        let rewritten = self.log.set_len(0).and_then(|()| self.log.rewind()).and_then(|()| self.log.write_all(text.as_bytes()));
        if let Err(e) = rewritten
            && self.failed.is_none()
        {
            self.failed = Some(format!("cannot write {}: {e}", self.log_path.display()));
        }
    }
}

/// An engine thread's hold on its game's record: what it writes goes in only
/// while no newer thread has taken the record over.
#[derive(Clone)]
pub struct Writer {
    record: Arc<Mutex<Record>>,
    number: u64,
    /// The answers this writer has replayed, which the window counts.
    replayed: Arc<AtomicUsize>,
}

impl Writer {
    /// `record`'s next writer, the one writing now shut first.
    pub fn take_over(record: &Arc<Mutex<Record>>) -> Writer {
        let number = locked(record).shut();
        Writer { record: Arc::clone(record), number, replayed: Arc::default() }
    }

    /// The answers replayed so far, read from the window's thread.
    pub fn replayed(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.replayed)
    }

    /// The record, while this writer may write.
    fn record(&self) -> Option<MutexGuard<'_, Record>> {
        let record = locked(&self.record);
        (record.writer == self.number).then_some(record)
    }

    /// The writer's replay, which a newer writer supersedes; at once, if
    /// one already has.
    fn replaying(&self, control: Arc<ReplayControl>) {
        match self.record() {
            Some(mut record) => record.replay = Some(control),
            None => control.supersede(),
        }
    }

    /// The game's next answer: held while the writer replays, and once it
    /// has handed over, written to the log and to the save.
    fn answer(&self, game: &GameState, decision: &LoggedDecision) {
        let Some(mut record) = self.record() else { return };
        record.logged += 1;
        let line = AnswerLine::of(game, record.logged, decision);
        if !record.live {
            record.replayed.push(line.to_string());
            self.replayed.store(record.replayed.len(), Ordering::Relaxed);
            return;
        }
        record.write(RecordFile::Log, &line.to_string());
        let saved = record.save.answer(line);
        record.write(RecordFile::Save, &saved);
    }

    /// The writer hands the game to the seats: the log is its line from the
    /// start, and what comes next is new.
    fn hand_over(&self) {
        let Some(mut record) = self.record() else { return };
        if !record.live {
            record.live = true;
            record.rewrite_log();
        }
    }

    /// The replay stopped at a line this build does not take, after
    /// `answers` it did: the save's line moves back to the place they reach,
    /// and the writer holds its next replay's answers afresh.
    fn diverged(&self, answers: usize) {
        let Some(mut record) = self.record() else { return };
        let place = record.save.along(record.save.current(), answers);
        record.move_to(place);
        (record.replayed, record.logged) = (Vec::new(), 0);
        self.replayed.store(0, Ordering::Relaxed);
    }

    /// The window is asked at the save's current place.
    fn window_asked(&self) {
        let Some(mut record) = self.record() else { return };
        if let Some(line) = record.save.window_asked() {
            record.write(RecordFile::Save, &line);
        }
    }

    fn outcome(&self, outcome: &decision_log::Outcome) {
        if let Some(mut record) = self.record() {
            record.write(RecordFile::Log, &outcome.to_string());
        }
    }
}

/// A record behind its lock, which a panic while writing a line leaves as it was.
pub fn locked(record: &Mutex<Record>) -> MutexGuard<'_, Record> {
    record.lock().unwrap_or_else(PoisonError::into_inner)
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

#[cfg(test)]
mod tests {
    use mtgsim::state::decision_log::LoggedAnswer;
    use mtgsim::test_support::setup_two_player_game;
    use mtgsim::ui::decision::PriorityAction;

    use super::*;

    /// A writer taken over writes nothing more, to either file; the one
    /// taking over holds what it replays until it hands over, then writes
    /// the log again from its start, and the save takes only what is new.
    #[test]
    fn a_writer_taken_over_writes_nothing_more() {
        let dir = std::env::temp_dir().join("devgui-record-writers");
        let _ = std::fs::remove_dir_all(&dir);
        let start = GameStart::Dealt { seed: 1, config: GameConfig::unrestricted(), decks: vec![Vec::new(); 2] };
        let record = Arc::new(Mutex::new(Record::create(Save::new(start.clone()), dir.join("seed-1.log")).unwrap()));
        let game = setup_two_player_game();
        let options = [ChoiceOption::Action(PriorityAction::Pass)];
        let pass = LoggedDecision { player: 0, kind: &ChoiceKind::PriorityAction, options: &options, answer: LoggedAnswer::Picks(&[0]), forced: true };
        let read = |path: PathBuf| std::fs::read_to_string(path).unwrap();
        let opening = decision_log::opening(&start).join("\n") + "\n";
        let answered = "answer 1 [turn 1, precombat main] player 0 PriorityAction forced picks pass\n";

        let first = Writer::take_over(&record);
        first.hand_over();
        first.answer(&game, &pass);
        let second = Writer::take_over(&record);
        first.answer(&game, &pass);
        first.outcome(&decision_log::Outcome::Draw);
        assert_eq!(read(dir.join("seed-1.log")), format!("{opening}{answered}"), "the first writer, shut");
        second.answer(&game, &pass);
        assert_eq!(read(dir.join("seed-1.log")), format!("{opening}{answered}"), "the second's answer held");
        second.hand_over();
        assert_eq!(read(dir.join("seed-1.log")), format!("{opening}{answered}"), "the log written again");
        assert_eq!(read(save::path_for(&dir.join("seed-1.log"))), format!("{opening}{answered}"), "one answer saved");
    }
}
