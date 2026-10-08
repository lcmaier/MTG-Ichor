//! What the window shows and what a click means, in plain Rust.
//!
//! [`WindowState`] takes the engine's messages and the player's [`Input`]s and
//! hands back an answer once one is complete. [`BoardView`] and [`PromptView`]
//! are the board and the question as the drawing needs them: grouped, labeled,
//! and marked clickable, chosen, or the prompt's subject. `app` draws them and
//! reports clicks; nothing here knows egui.
//!
//! No case per `ChoiceKind`: a click is read off the primitive and off the
//! board things each option names (`prompt::OptionView::refs`).

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use mtgsim::events::event::EventSeq;
use mtgsim::types::ids::PlayerId;
use mtgsim::types::triggers::DelayedDuration;
use mtgsim::ui::auto_yield::Yield;
use mtgsim::ui::waiting::{TriggersIn, Waiting};
use mtgsim::ui::why::{Why, WhyAbout, WhyLine};

use crate::bridge::{Outcome, ToWindow};
use crate::editor::EditorInput;
use crate::prompt::{Answer, BoardRef, Primitive, Prompt, Reply};
use crate::save::{Destination, Tools};
use crate::snapshot::{CardView, LogLine, PermanentView, PlayerView, Snapshot};
pub use crate::snapshot::{TypeLineView, TypeWordView};

/// What the window shows: the game, or the board editor beside it
/// (`setup-architecture.md` §7b, decision 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Play,
    Edit,
}

/// Something the player did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    /// Clicked option `i`'s button.
    OptionButton(usize),
    /// Clicked a card or a player on the board.
    Board(BoardRef),
    /// Set a `pick_number`'s number.
    Number(u64),
    /// One more into bucket `i` of an allocation.
    OneMore(usize),
    /// One fewer in bucket `i` of an allocation.
    OneFewer(usize),
    /// The confirm button.
    Done,
    /// Start the answer over.
    Reset,
    /// Build the game again from its scenario file, read again.
    Reload,
    /// Go back to the window's previous question: a game built again and
    /// replayed to it, asking it again.
    Undo,
    /// Mark the open question's place in the save, to come back to.
    Savestate,
    /// Build the game again at this place in the save: a savestate's, or
    /// the one the window's line last left.
    MoveTo(usize),
    /// Write the board at this prompt to a scenario file.
    SaveBoard,
    /// Pass at this priority prompt, and keep passing until the yield ends.
    Yield(Yield),
    /// End the seat's yield; the prompt stays open.
    StopYielding,
    /// Turn full control on or off, which `Session::input` carries to the seat.
    FullControl(bool),
    /// A shortcut key went down: a held key's repeats arrive too, and answer
    /// nothing.
    Key { key: Key, repeat: bool },
    /// Show the game or the editor.
    Mode(Mode),
    /// Open the board the game is at in the editor.
    EditThisBoard,
    /// Open the board the game began from in the editor.
    EditTheScenario,
    /// A click in the editor, or one of its buttons the session acts on.
    Editor(EditorInput),
    /// Ask why of an object or a player: a right-click on it on the board,
    /// or a click on its name in the why panel.
    Why(BoardRef),
    /// Ask what an event did: a right-click on its line in the log.
    WhyEvent(EventSeq),
    /// The why panel goes back to the object asked about before this one.
    WhyBack,
    /// Close the why panel.
    WhyClose,
}

/// A key the window reads, as the drawing reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// `1` to `9`: that option's button.
    Digit(u8),
    Enter,
    Space,
    Escape,
    F2,
    F4,
    F6,
}

/// What each key does, for the prompt's foot.
pub const KEYS: &str = "Keys: 1–9 an option · Enter confirm · Space pass · Esc start over · F2 pass until the stack changes · F4 until end of turn · F6 until this player's next turn";

/// How long after a prompt arrives its input is dropped (`codebase-state.md`
/// item 201): egui's double-click window, so the second click of a double
/// click, or a quick click aimed at the prompt before, cannot answer one the
/// person has not seen.
pub const SETTLE_SECONDS: f64 = 0.3;

/// What is left by `now` of a beat that began at `began`, while some is.
fn beat_left(now: f64, began: f64) -> Option<f64> {
    let left = SETTLE_SECONDS - (now - began);
    (left > 0.0).then_some(left)
}

/// The answer being put together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    /// Options in click order, and a pair's first half clicked on the board
    /// with its second half still to come.
    Picks { chosen: Vec<usize>, half: Option<BoardRef> },
    Number(u64),
    Allocation(Vec<u64>),
    Order(Vec<usize>),
}

impl Selection {
    pub fn start(prompt: &Prompt) -> Selection {
        match &prompt.primitive {
            Primitive::PickN { .. } => Selection::Picks { chosen: Vec::new(), half: None },
            Primitive::Number { min, .. } => Selection::Number(*min),
            Primitive::Allocate { mins, .. } => Selection::Allocation(mins.clone()),
            Primitive::Order => Selection::Order(Vec::new()),
        }
    }

    /// Whether `input` would change the answer in progress or complete it:
    /// a click the window may offer. Run on a copy, so it is the input's own
    /// logic that answers and no second copy of the rules.
    pub fn is_live(&self, prompt: &Prompt, input: Input) -> bool {
        let mut probe = self.clone();
        probe.apply(prompt, input).is_some() || probe != *self
    }

    /// The answer, once `input` completes a legal one. Legal by `ui/ask.rs`'s
    /// validators, so the engine never asserts on a window's answer.
    pub fn apply(&mut self, prompt: &Prompt, input: Input) -> Option<Answer> {
        if input == Input::Reset {
            *self = Selection::start(prompt);
            return None;
        }
        match (&prompt.primitive, self) {
            (Primitive::PickN { min, max }, Selection::Picks { chosen, half }) => {
                pick(prompt, (*min, *max), chosen, half, input)
            }
            (Primitive::Number { min, max }, Selection::Number(value)) => match input {
                Input::Number(n) => {
                    *value = n.clamp(*min, *max);
                    None
                }
                Input::Done => Some(Answer::Number(*value)),
                _ => None,
            },
            (Primitive::Allocate { total, mins, maxs }, Selection::Allocation(amounts)) => {
                allocate(prompt, *total, mins, maxs.as_deref(), amounts, input)
            }
            (Primitive::Order, Selection::Order(order)) => {
                let item = match input {
                    Input::OptionButton(i) => Some(i),
                    Input::Board(target) => (0..prompt.options.len())
                        .find(|i| prompt.options[*i].refs.first() == Some(&target) && !order.contains(i)),
                    Input::Done => {
                        return (order.len() == prompt.options.len()).then(|| Answer::Order(order.clone()));
                    }
                    _ => None,
                };
                if let Some(i) = item
                    && i < prompt.options.len()
                    && !order.contains(&i)
                {
                    order.push(i);
                }
                None
            }
            // `start` builds the selection from the same prompt, so the shapes always agree.
            _ => None,
        }
    }
}

fn pick(
    prompt: &Prompt,
    (min, max): (usize, usize),
    chosen: &mut Vec<usize>,
    half: &mut Option<BoardRef>,
    input: Input,
) -> Option<Answer> {
    let clicked = match input {
        Input::OptionButton(i) if i < prompt.options.len() => Some(i),
        Input::Board(target) => board_option(prompt, half, target),
        Input::Done => return (min..=max).contains(&chosen.len()).then(|| Answer::Picks(chosen.clone())),
        _ => None,
    }?;
    if max == 1 {
        return Some(Answer::Picks(vec![clicked]));
    }
    if let Some(at) = chosen.iter().position(|i| *i == clicked) {
        chosen.remove(at);
    } else if chosen.len() < max {
        chosen.push(clicked);
    }
    None
}

/// A board click as an option: the one option naming `target` first, or the
/// pair `target` completes. When several pairs start at `target`, it becomes
/// the half and the next click picks the pair.
fn board_option(prompt: &Prompt, half: &mut Option<BoardRef>, target: BoardRef) -> Option<usize> {
    if let Some(first) = half.take()
        && let Some(i) = prompt.options.iter().position(|o| o.refs == [first, target])
    {
        return Some(i);
    }
    let starting: Vec<usize> =
        (0..prompt.options.len()).filter(|i| prompt.options[*i].refs.first() == Some(&target)).collect();
    match starting.as_slice() {
        [] => None,
        [only] => Some(*only),
        several if several.iter().all(|i| prompt.options[*i].refs.len() == 2) => {
            *half = Some(target);
            None
        }
        // Two abilities of one permanent, say: the prompt's buttons tell them apart.
        _ => None,
    }
}

fn allocate(
    prompt: &Prompt,
    total: u64,
    mins: &[u64],
    maxs: Option<&[u64]>,
    amounts: &mut [u64],
    input: Input,
) -> Option<Answer> {
    let (bucket, more) = match input {
        Input::OneMore(bucket) | Input::OptionButton(bucket) => (bucket, true),
        Input::OneFewer(bucket) => (bucket, false),
        Input::Board(target) => (prompt.options.iter().position(|o| o.refs.first() == Some(&target))?, true),
        Input::Done => return (amounts.iter().sum::<u64>() == total).then(|| Answer::Allocation(amounts.to_vec())),
        _ => return None,
    };
    let left = total.saturating_sub(amounts.iter().sum());
    let cap = maxs.map_or(u64::MAX, |maxs| maxs[bucket]);
    match amounts.get_mut(bucket) {
        Some(amount) if more && left > 0 && *amount < cap => *amount += 1,
        Some(amount) if !more && *amount > mins[bucket] => *amount -= 1,
        _ => {}
    }
    None
}

/// What kept a game from starting, which a person fixes, as against an
/// engine panic, which is a bug to report (`engineering-practices.md` §10.1,
/// question 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The scenario did not load: its file's line, and what to change.
    Scenario,
    /// The game's decision log could not be made (`codebase-state.md` item 200).
    Record,
    /// The file `--load` names did not load, or its start does not build.
    Load,
}

impl Refusal {
    /// What did not happen, as the header and the prompt's place say it.
    pub fn heading(self) -> &'static str {
        match self {
            Refusal::Scenario => "The scenario did not load",
            Refusal::Record => "The decision log could not be made",
            Refusal::Load => "The save did not load",
        }
    }

    /// What to do about it.
    pub fn hint(self) -> &'static str {
        match self {
            Refusal::Scenario => "Fix the file, then click Reload; or Edit the scenario.",
            Refusal::Record => "Make its folder writable, then start the game again.",
            Refusal::Load => "--load takes a save, <log>.save, or a decision log.",
        }
    }
}

/// Everything the window knows.
#[derive(Clone, Debug, Default)]
pub struct WindowState {
    pub board: Option<Snapshot>,
    pub prompt: Option<Prompt>,
    pub selection: Option<Selection>,
    /// Every log line so far, oldest first.
    pub log: Vec<LogLine>,
    pub outcome: Option<Outcome>,
    pub panic: Option<String>,
    /// Why no game started, and the refusal's words.
    pub refused: Option<(Refusal, String)>,
    /// Where a replayed line stopped, as this build no longer takes it: the
    /// game plays on from the answer before.
    pub diverged: Option<String>,
    /// The yield the seat holds at the open prompt.
    pub yielding: Option<Yield>,
    /// Full control is on: the seat is asked at every priority point, with
    /// its decorators and its yields off.
    pub full_control: bool,
    /// The window's clock, in seconds, as egui keeps it; `None` in a test
    /// that keeps none, where nothing settles.
    pub now: Option<f64>,
    /// When the open prompt arrived, by `now`.
    pub prompt_at: Option<f64>,
    /// When the why panel last opened or closed beside the board, sliding
    /// the board under the pointer, by `now`.
    pub board_moved_at: Option<f64>,
    /// When what the window shows was last replaced under the pointer: the
    /// game and the editor switched, or another board opened, by `now`.
    pub replaced_at: Option<f64>,
    /// Prompts received, which keys each prompt's widgets apart: focus on one
    /// prompt's button cannot pass to the next prompt's.
    pub prompts: u64,
    /// A rebuild's replay, until the engine asks at the place it replays to.
    pub replaying: Option<Progress>,
    /// What the save lets the tools do, as the session last read it.
    pub tools: Tools,
    /// Why the game's record stopped being written, if it has: what the
    /// header says until another game starts.
    pub unwritten: Option<String>,
    /// What the why panel was asked about, what it shows last; empty while
    /// the panel is closed (`setup-architecture.md` §7c).
    pub why_path: Vec<WhyAbout>,
    /// The engine's last answer for what the panel shows.
    pub why: Option<Why>,
    /// The why the panel waits on a replay's trace for, which the session
    /// starts and supersedes (`why_replay`).
    pub reading_the_trace: Option<TraceRequest>,
    /// Requests made of a replay so far, which number the next.
    pub trace_requests: u64,
}

/// A why the window asks of a replay: about what, and its number, which the
/// answer carries back, so an answer to a request since replaced is dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraceRequest {
    pub about: WhyAbout,
    pub number: u64,
}

/// A replay the window counts while it waits (`setup-architecture.md` §7.3).
#[derive(Clone, Debug)]
pub struct Progress {
    /// The answers replayed so far, which the engine's thread counts.
    pub done: Arc<AtomicUsize>,
    /// The answers its line holds.
    pub of: usize,
}

/// What Undo answer's place says while it is off.
pub const NO_EARLIER_QUESTION: &str = "No earlier question to go back to.";

/// What the menu says while it lists nothing.
pub const NO_SAVESTATES: &str = "No savestates yet: Savestate marks the open question.";

/// What the prompt's place says above a divergence's message.
pub const DIVERGED: [&str; 2] = ["The loaded line stops here: this build does not take its next answer.", "Play goes on from the answer before it."];

/// The game's tools, beside Reload in the header (`setup-architecture.md`
/// §7.3): Undo answer, Savestate, and the menu of savestates and the line
/// most recently left.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolsView {
    pub undo: ToolButton,
    /// Why Undo answer is off, while it is.
    pub undo_off: Option<&'static str>,
    pub savestate: ToolButton,
    /// The savestates in the order set, then "back to where I was"; each
    /// off while the window's line is at its place.
    pub menu: Vec<ToolButton>,
}

/// A header control: a click on it is `input`, while it is live.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolButton {
    pub label: String,
    pub input: Input,
    pub live: bool,
}

impl WindowState {
    pub fn receive(&mut self, message: ToWindow) {
        let shown = self.why_shown();
        self.take(message);
        self.note_the_panel(shown);
    }

    fn take(&mut self, message: ToWindow) {
        match message {
            ToWindow::Prompt { snapshot, prompt, yielding, why } => {
                self.replaying = None;
                self.log.extend(snapshot.log.iter().cloned());
                self.yielding = yielding;
                self.prompt_at = self.now;
                self.prompts += 1;
                self.selection = Some(Selection::start(&prompt));
                let reads_the_trace = prompt.why_reads_the_trace;
                self.prompt = Some(prompt);
                self.board = Some(snapshot);
                // The panel follows an object or a player to this question:
                // the seat's answer, or else a replay's. An event's answer
                // stands, and a panel closed while this was on its way shows
                // nothing.
                match self.why_path.last().copied() {
                    Some(about @ (WhyAbout::Object(_) | WhyAbout::Player(_))) if reads_the_trace => self.read_the_trace(about),
                    Some(WhyAbout::Object(_) | WhyAbout::Player(_)) => {
                        self.why = why.map(|why| *why);
                        self.reading_the_trace = None;
                    }
                    Some(WhyAbout::Event(_)) | None => {}
                }
            }
            ToWindow::Why(why) => {
                if matches!(self.why_path.last(), Some(WhyAbout::Object(_) | WhyAbout::Player(_))) {
                    self.why = Some(why);
                }
            }
            ToWindow::WhyFromTrace { request, why } => {
                if self.reading_the_trace.is_some_and(|reading| reading.number == request) {
                    self.why = Some(why);
                    self.reading_the_trace = None;
                }
            }
            ToWindow::Finished { snapshot, outcome } => {
                self.replaying = None;
                self.log.extend(snapshot.log.iter().cloned());
                self.board = Some(snapshot);
                self.prompt = None;
                self.selection = None;
                self.outcome = Some(outcome);
            }
            ToWindow::Panicked { message } => {
                self.replaying = None;
                self.prompt = None;
                self.selection = None;
                self.panic = Some(message);
            }
            ToWindow::Refused { refusal, message } => {
                self.replaying = None;
                self.refused = Some((refusal, message));
            }
            ToWindow::Diverged { message, replaying } => {
                self.diverged = Some(message);
                if let Some(progress) = &mut self.replaying {
                    progress.of = replaying;
                }
            }
        }
    }

    /// What to send the engine's thread, once `input` makes something: an
    /// answer or a yield, either of which closes the prompt, or "stop
    /// yielding", which leaves it open. The board stays until the engine's
    /// next message.
    pub fn input(&mut self, input: Input) -> Option<Reply> {
        let shown = self.why_shown();
        let reply = self.reply_to(input);
        self.note_the_panel(shown);
        reply
    }

    fn reply_to(&mut self, input: Input) -> Option<Reply> {
        let reply = match input {
            // The window's own controls, and the editor's, which `Session::input` acts on.
            Input::Reload
            | Input::Undo
            | Input::Savestate
            | Input::MoveTo(_)
            | Input::SaveBoard
            | Input::Mode(_)
            | Input::EditThisBoard
            | Input::EditTheScenario
            | Input::Editor(_) => return None,
            Input::FullControl(on) => {
                self.full_control = on;
                return None;
            }
            // A why answers nothing, so it closes no prompt and the beat after
            // one arrives need not drop it. It is asked at an open question,
            // or of a game that has ended.
            Input::Why(target) => return self.ask_why(target.why_about()),
            Input::WhyEvent(event) => return self.ask_why(WhyAbout::Event(event)),
            Input::WhyBack => {
                if !self.why_is_live() || self.why_path.len() < 2 {
                    return None;
                }
                self.why_path.pop();
                return self.why_path.last().copied().and_then(|about| self.answer_from(about));
            }
            Input::WhyClose => {
                (self.why_path, self.why, self.reading_the_trace) = (Vec::new(), None, None);
                return Some(Reply::Why(None));
            }
            // Aimed at the prompt before: it arrived too recently to be read.
            _ if self.settling_for().is_some() => return None,
            // Aimed at the board before the why panel slid it.
            Input::Board(_) if self.board_settling_for().is_some() => return None,
            // A shortcut acts on the press; a held key's repeat answers nothing.
            Input::Key { repeat: true, .. } => return None,
            Input::Key { key, .. } => return self.click_for(key).and_then(|click| self.input(click)),
            Input::StopYielding => return self.yielding.take().map(|_| Reply::StopYielding),
            Input::Yield(until) => self.yield_is_live(until).then_some(Reply::Yield(until))?,
            input => Reply::Answer(self.selection.as_mut()?.apply(self.prompt.as_ref()?, input)?),
        };
        self.prompt = None;
        self.selection = None;
        Some(reply)
    }

    /// The click `key` stands for at the open prompt.
    fn click_for(&self, key: Key) -> Option<Input> {
        let prompt = self.prompt.as_ref()?;
        match key {
            Key::Digit(n) => Some(Input::OptionButton(usize::from(n).checked_sub(1)?)),
            Key::Enter => Some(Input::Done),
            Key::Space => prompt.pass.map(Input::OptionButton),
            Key::Escape => Some(Input::Reset),
            Key::F2 => Some(Input::Yield(Yield::UntilStackChanges)),
            Key::F4 => Some(Input::Yield(Yield::UntilEndOfTurn)),
            Key::F6 => Some(Input::Yield(Yield::UntilYourNextTurn)),
        }
    }

    /// `about` asked why of, the panel's path stepping to it unless it shows
    /// it already, when the right-click is live.
    fn ask_why(&mut self, about: WhyAbout) -> Option<Reply> {
        if !self.why_is_live() {
            return None;
        }
        if self.why_path.last() != Some(&about) {
            self.why_path.push(about);
        }
        self.answer_from(about)
    }

    /// Who answers a why about `about`: the open question's seat, about now,
    /// or else a replay's trace, for an event, at a question whose why reads
    /// the trace, and once the game has ended. The seat is told what the
    /// panel follows either way, which for an event, answered once, is
    /// nothing.
    fn answer_from(&mut self, about: WhyAbout) -> Option<Reply> {
        let follow = BoardRef::of(about);
        let at_the_seat = follow.is_some() && self.prompt.as_ref().is_some_and(|prompt| !prompt.why_reads_the_trace);
        if at_the_seat {
            self.reading_the_trace = None;
        } else {
            self.read_the_trace(about);
        }
        self.prompt.as_ref().map(|_| Reply::Why(follow))
    }

    /// The panel waits on a replay for `about`'s why, a new request.
    fn read_the_trace(&mut self, about: WhyAbout) {
        self.trace_requests += 1;
        (self.why, self.reading_the_trace) = (None, Some(TraceRequest { about, number: self.trace_requests }));
    }

    /// A why can be asked now: at an open question, or of a game that has
    /// ended, which a replay of its whole line answers.
    pub fn why_is_live(&self) -> bool {
        self.prompt.is_some() || self.outcome.is_some()
    }

    /// The object or player the panel follows from question to question.
    pub fn following(&self) -> Option<BoardRef> {
        self.why_path.last().copied().and_then(BoardRef::of)
    }

    /// What a right-click on a log line does now, while it asks.
    pub fn log_hint(&self) -> Option<&'static str> {
        self.why_is_live().then_some(WHY_ON_A_LOG_LINE)
    }

    /// The window's clock, which the drawing reads from egui at each frame.
    pub fn tick(&mut self, now: f64) {
        self.now = Some(now);
    }

    /// How much longer the open prompt drops input, while it does.
    pub fn settling_for(&self) -> Option<f64> {
        beat_left(self.now?, self.prompt_at?)
    }

    /// How much longer a click on the board is dropped, while one is: the
    /// prompt's beat, or the one after the why panel slid the board.
    pub fn board_settling_for(&self) -> Option<f64> {
        let moved = self.now.zip(self.board_moved_at).and_then(|(now, at)| beat_left(now, at));
        self.settling_for().into_iter().chain(moved).reduce(f64::max)
    }

    /// What the window shows was replaced under the pointer, so every click
    /// is dropped for a beat, as at a new prompt (`Session::input`).
    pub fn replaced(&mut self) {
        self.replaced_at = self.now;
    }

    /// How much longer every click is dropped, after a replacement.
    pub fn replacing_for(&self) -> Option<f64> {
        beat_left(self.now?, self.replaced_at?)
    }

    /// Whether the why panel shows, as `why_view` decides.
    fn why_shown(&self) -> bool {
        self.why.is_some() || (self.reading_the_trace.is_some() && !self.why_path.is_empty())
    }

    /// The board moved if the panel beside it opened or closed since `shown`.
    fn note_the_panel(&mut self, shown: bool) {
        if self.why_shown() != shown {
            self.board_moved_at = self.now;
        }
    }

    /// Whether `until` can be set at the open prompt: a priority prompt, with
    /// full control off, since it supersedes yields, and for the yield that
    /// waits on the stack, a stack to wait on.
    fn yield_is_live(&self, until: Yield) -> bool {
        let at_priority = self.prompt.as_ref().is_some_and(|prompt| prompt.pass.is_some());
        let over_a_stack = self.board.as_ref().is_some_and(|board| !board.stack.is_empty());
        at_priority && !self.full_control && (until != Yield::UntilStackChanges || over_a_stack)
    }

    /// What the window is doing, for the header.
    pub fn status(&self) -> String {
        if let Some((refusal, _)) = self.refused {
            refusal.heading().to_string()
        } else if self.panic.is_some() {
            "The engine panicked".to_string()
        } else if let Some(outcome) = &self.outcome {
            match outcome {
                Outcome::Won(player) => format!("Game over: Player {player} wins"),
                Outcome::Draw => "Game over: a draw".to_string(),
                Outcome::Error(error) => format!("The engine returned an error: {error}"),
            }
        } else if let Some(prompt) = &self.prompt {
            format!("Player {} to decide", prompt.player)
        } else if let Some(replaying) = &self.replaying {
            format!("Replaying: {} of {} answers", replaying.done.load(Ordering::Relaxed), replaying.of)
        } else {
            "The engine is playing".to_string()
        }
    }

    /// What the board's place says while there is no board.
    pub fn no_board(&self) -> &'static str {
        match self.refused {
            Some((Refusal::Scenario, _)) => "No board: the scenario did not load.",
            Some((Refusal::Record, _)) => "No board: the decision log could not be made.",
            Some((Refusal::Load, _)) => "No board: the save did not load.",
            None if self.panic.is_some() => "No board: the engine panicked before its first prompt.",
            None if self.replaying.is_some() => "Replaying the game to its question: the board shows once the question is asked.",
            None => "Waiting for the engine's first prompt.",
        }
    }

    /// The tools as the header shows them: Undo answer goes back past the
    /// open question, or the one the replay is on its way to, or else to
    /// the question last answered.
    pub fn tools_view(&self) -> ToolsView {
        let open = self.prompt.is_some() || self.replaying.is_some();
        let live = if open { self.tools.undo_open } else { self.tools.undo_answered };
        let undo = ToolButton { label: "Undo answer".to_string(), input: Input::Undo, live };
        let marks = self.prompt.is_some() && !self.tools.savestate_here;
        let savestate = ToolButton { label: "Savestate".to_string(), input: Input::Savestate, live: marks };
        let menu = self.tools.destinations.iter().map(|destination| {
            let (label, at) = match destination {
                Destination::Savestate { at, name } => (name.clone(), *at),
                Destination::Left(at) => ("Back to where I was".to_string(), *at),
            };
            ToolButton { label, input: Input::MoveTo(at), live: at != self.tools.current }
        });
        ToolsView { undo, undo_off: (!live).then_some(NO_EARLIER_QUESTION), savestate, menu: menu.collect() }
    }

    /// The name a savestate at the open question takes: its turn and step.
    pub fn savestate_name(&self) -> Option<String> {
        let board = self.board.as_ref().filter(|_| self.prompt.is_some())?;
        Some(format!("Turn {} · {}", board.turn, board.phase))
    }

    pub fn board_view(&self) -> Option<BoardView> {
        let board = self.board.as_ref()?;
        let mut marks = Marks::new(self.prompt.as_ref(), self.selection.as_ref());
        marks.why_is_live = self.why_is_live();
        if self.board_settling_for().is_some() {
            marks.clickable.clear();
        }
        Some(BoardView::new(board, &marks))
    }

    pub fn prompt_view(&self) -> Option<PromptView> {
        let prompt = self.prompt.as_ref()?;
        let mut view = PromptView::new(prompt, self.selection.as_ref()?, self.board.as_ref());
        view.serial = self.prompts;
        if prompt.pass.is_some() {
            view.yields = [Yield::UntilEndOfTurn, Yield::UntilStackChanges, Yield::UntilYourNextTurn]
                .into_iter()
                .map(|until| SeatButton {
                    label: format!("Pass {}", until_words(until, prompt.player)),
                    input: Input::Yield(until),
                    live: self.yield_is_live(until),
                })
                .collect();
        }
        view.yielding = self.yielding.map(|until| {
            let stop = SeatButton { label: "Stop yielding".to_string(), input: Input::StopYielding, live: true };
            (format!("Passing {}", until_words(until, prompt.player)), stop)
        });
        Some(if self.settling_for().is_some() { view.settling() } else { view })
    }

    /// The why panel, while it is open and the engine has answered, or a
    /// replay is reading the trace for it. Its links ask why, so while none
    /// can be asked they are off and the panel says so.
    pub fn why_view(&self) -> Option<WhyView> {
        let live = self.why_is_live();
        let back = live && self.why_path.len() > 1;
        let Some(why) = &self.why else {
            let reading = self.reading_the_trace.is_some() && !self.why_path.is_empty();
            let title = "reading the trace".to_string();
            return reading.then(|| WhyView { title, back, note: Some(WHY_READING_THE_TRACE), sections: Vec::new() });
        };
        let line = |line: &WhyLine| WhyLineView {
            text: line.text.clone(),
            rule: line.rule.map(|rule| format!("CR {rule}")),
            depth: line.depth,
            links: line
                .names
                .iter()
                .map(|(id, label)| WhyLink { label: label.clone(), input: Input::Why(BoardRef::Object(*id)), live })
                .collect(),
        };
        let sections = why
            .sections
            .iter()
            .map(|section| WhySectionView { heading: section.heading.clone(), lines: section.lines.iter().map(line).collect() })
            .collect();
        let note = if self.reading_the_trace.is_some() {
            Some(WHY_READING_THE_TRACE)
        } else {
            (!live).then_some(WHY_AT_A_QUESTION)
        };
        Some(WhyView { title: why.title.clone(), back, note, sections })
    }
}

/// What the why panel says while no question is open and the game goes on.
pub const WHY_AT_A_QUESTION: &str = "The answer at the last question: a why is asked while a question is open.";

/// What the why panel says while a replay reads the trace for it.
pub const WHY_READING_THE_TRACE: &str = "Reading the trace: the game is replaying to this question.";

/// What an object's hover says a right-click does, while one asks and while
/// none does, when no seat waits and the game goes on.
pub const WHY_ON_RIGHT_CLICK: &str = "Right-click: why it is so";
pub const WHY_ONLY_AT_A_QUESTION: &str = "A right-click asks why only while a question is open, or once the game is over";

/// What the log's heading says a right-click on a line does.
pub const WHY_ON_A_LOG_LINE: &str = "Right-click a line: what happened";

/// The why panel: what the engine says made an object the way it is
/// (`setup-architecture.md` §7c).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhyView {
    /// The object, as the board names it.
    pub title: String,
    /// Back is live: there is an object asked about before this one, and a
    /// question open to ask at.
    pub back: bool,
    /// Why the panel's links are off, while they are.
    pub note: Option<&'static str>,
    pub sections: Vec<WhySectionView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhySectionView {
    pub heading: String,
    pub lines: Vec<WhyLineView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhyLineView {
    pub text: String,
    /// The rule it rests on: `CR 613.1f`.
    pub rule: Option<String>,
    /// How far it sits under the line before it.
    pub depth: u8,
    /// Each object it names: a click asks that object's why.
    pub links: Vec<WhyLink>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhyLink {
    pub label: String,
    pub input: Input,
    pub live: bool,
}

/// How long a yield passes for, in the person's words: a seat's yield passes
/// for that seat, so the next turn it waits for is `player`'s.
fn until_words(until: Yield, player: PlayerId) -> String {
    match until {
        Yield::UntilEndOfTurn => "until end of turn".to_string(),
        Yield::UntilStackChanges => "until the stack changes".to_string(),
        Yield::UntilYourNextTurn => format!("until Player {player}'s next turn"),
    }
}

/// One card, permanent, stack object or player, as the drawing shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub target: Option<BoardRef>,
    pub title: String,
    /// P/T, counters, status; the type line for a card.
    pub detail: String,
    /// An option names it, so a click means something.
    pub clickable: bool,
    pub chosen: bool,
    /// The prompt is about it (`ChoiceKind::subject()`).
    pub subject: bool,
    pub tapped: bool,
    /// Shown on hover, what it is now; empty for none.
    pub hover: String,
    /// Shown on hover beside it: the card as printed, a face an entry.
    pub printed: Option<Arc<[String]>>,
    /// Shown on hover under the first line of `hover`, a permanent's.
    pub type_line: Option<Arc<TypeLineView>>,
    /// Shown on hover last, an object's: what a right-click on it does now.
    pub why_hint: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneView {
    /// The header, with the zone's count where it has one.
    pub name: String,
    /// The zone without its count, which keys the header's open state, so a
    /// card arriving does not close a graveyard the person opened.
    pub key: &'static str,
    pub items: Vec<Item>,
    /// Shown open rather than collapsed.
    pub open: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeatView {
    pub player: Item,
    pub zones: Vec<ZoneView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardView {
    pub header: String,
    /// Seat 0 last, nearest the prompt, and the others above it in seat
    /// order, whichever seat is asked.
    pub seats: Vec<SeatView>,
    /// Top first.
    pub stack: Vec<Item>,
    pub pending_triggers: Vec<Item>,
    pub exile: Vec<Item>,
    pub command: Vec<Item>,
    /// What the game is holding for later, closed until opened; `None`
    /// while nothing is, as an empty exile is not shown.
    pub waiting: Option<ZoneView>,
}

/// Which board things a click would move the answer along, which the answer
/// has chosen, which the prompt is about, and which player it asks.
#[derive(Default)]
struct Marks {
    clickable: Vec<BoardRef>,
    chosen: Vec<BoardRef>,
    subject: Option<BoardRef>,
    asked: Option<PlayerId>,
    /// A right-click asks why now (`WindowState::why_is_live`).
    why_is_live: bool,
}

impl Marks {
    fn new(prompt: Option<&Prompt>, selection: Option<&Selection>) -> Marks {
        let (Some(prompt), Some(selection)) = (prompt, selection) else {
            return Marks::default();
        };
        let first = |i: &usize| prompt.options[*i].refs.first().copied();
        // A pair's half waiting for its second click offers only its partners.
        let candidates: Vec<BoardRef> = match selection {
            Selection::Picks { half: Some(half), .. } => {
                prompt.options.iter().filter(|o| o.refs.first() == Some(half)).filter_map(|o| o.refs.get(1).copied()).collect()
            }
            _ => prompt.options.iter().flat_map(|o| o.refs.iter().copied()).collect(),
        };
        let mut clickable: Vec<BoardRef> = Vec::new();
        for target in candidates {
            if !clickable.contains(&target) && selection.is_live(prompt, Input::Board(target)) {
                clickable.push(target);
            }
        }
        let chosen = match selection {
            Selection::Picks { chosen, half } => chosen.iter().filter_map(first).chain(*half).collect(),
            Selection::Order(order) => order.iter().filter_map(first).collect(),
            Selection::Allocation(_) | Selection::Number(_) => Vec::new(),
        };
        Marks { clickable, chosen, subject: prompt.subject.map(BoardRef::Object), asked: Some(prompt.player), why_is_live: true }
    }

    fn item(&self, target: BoardRef, title: String, detail: String, tapped: bool) -> Item {
        Item {
            target: Some(target),
            title,
            detail,
            clickable: self.clickable.contains(&target),
            chosen: self.chosen.contains(&target),
            subject: self.subject == Some(target),
            tapped,
            hover: String::new(),
            printed: None,
            type_line: None,
            why_hint: Some(if self.why_is_live { WHY_ON_RIGHT_CLICK } else { WHY_ONLY_AT_A_QUESTION }),
        }
    }

    fn card(&self, card: &CardView) -> Item {
        let title = match &card.mana_cost {
            Some(cost) => format!("{} {cost}", card.name),
            None => card.name.clone(),
        };
        let mut item = self.item(BoardRef::Object(card.id), title, card.type_line.text.clone(), false);
        item.printed = Some(Arc::clone(&card.printed));
        item
    }

    fn permanent(&self, permanent: &PermanentView) -> Item {
        let mut detail = Vec::new();
        if let Some((power, toughness)) = permanent.power_toughness {
            detail.push(format!("{power}/{toughness}"));
        }
        if permanent.damage > 0 {
            detail.push(format!("{} damage", permanent.damage));
        }
        detail.extend(permanent.keywords.iter().cloned());
        detail.extend(permanent.counters.iter().map(|(kind, n)| format!("{kind} ×{n}")));
        let flags = [
            (permanent.tapped, "tapped"),
            (permanent.summoning_sick, "summoning sick"),
            (permanent.phased_out, "phased out"),
            (permanent.face_down, "face down"),
        ];
        detail.extend(flags.iter().filter(|(on, _)| *on).map(|(_, word)| word.to_string()));
        if let Some(target) = &permanent.attacking {
            detail.push(format!("attacking {target}"));
        }
        for attacker in &permanent.blocking {
            detail.push(format!("blocking {attacker}"));
        }
        if let Some(host) = &permanent.attached_to {
            detail.push(format!("attached to {host}"));
        }
        let title = format!("{} ({})", permanent.card.name, permanent.card.id);
        let mut item = self.item(BoardRef::Object(permanent.card.id), title, detail.join(" · "), permanent.tapped);
        item.hover = permanent.engine_text.clone();
        item.printed = Some(Arc::clone(&permanent.card.printed));
        item.type_line = Some(Arc::clone(&permanent.card.type_line));
        item
    }

    fn player(&self, player: &PlayerView) -> Item {
        let asked = if self.asked == Some(player.id) { " (to decide)" } else { "" };
        let mut detail = vec![format!("{} life", player.life)];
        if !player.mana_pool.is_empty() {
            let pool: Vec<String> = player.mana_pool.iter().map(|(symbol, n)| symbol.repeat(*n as usize)).collect();
            detail.push(format!("pool {}", pool.concat()));
        }
        detail.extend(player.counters.iter().map(|(kind, n)| format!("{kind} {n}")));
        if player.lost {
            detail.push("lost".to_string());
        }
        self.item(BoardRef::Player(player.id), format!("Player {}{asked}", player.id), detail.join(" · "), false)
    }
}

impl BoardView {
    fn new(board: &Snapshot, marks: &Marks) -> BoardView {
        // One order at every prompt, so the board does not reshuffle as the
        // asked seat changes at each pass of priority.
        let mut seats: Vec<&PlayerView> = board.players.iter().skip(1).collect();
        seats.extend(board.players.first());
        BoardView {
            header: format!("Turn {} · Player {}'s turn · {}", board.turn, board.active_player, board.phase),
            seats: seats.into_iter().map(|player| seat(player, marks)).collect(),
            stack: board
                .stack
                .iter()
                .map(|item| {
                    let what = match item.is_spell {
                        Some(true) => "",
                        Some(false) => " — ability",
                        None => " — being cast or activated",
                    };
                    let mut detail = vec![format!("Player {}'s", item.controller)];
                    if !item.targets.is_empty() {
                        detail.push(format!("targets {}", item.targets.join(", ")));
                    }
                    if let Some(x) = item.x {
                        detail.push(format!("X = {x}"));
                    }
                    let title = format!("{} ({}){what}", item.name, item.id);
                    marks.item(BoardRef::Object(item.id), title, detail.join(" · "), false)
                })
                .collect(),
            pending_triggers: board
                .pending_triggers
                .iter()
                .map(|trigger| {
                    note(trigger.source.clone(), format!("Player {}'s trigger, not yet on the stack", trigger.controller))
                })
                .collect(),
            exile: board.exile.iter().map(|card| owned(marks, card)).collect(),
            command: board.command.iter().map(|card| owned(marks, card)).collect(),
            waiting: waiting_zone(&board.waiting),
        }
    }
}

/// A line on the board that names nothing a click could choose.
fn note(title: String, detail: String) -> Item {
    Item {
        target: None,
        title,
        detail,
        clickable: false,
        chosen: false,
        subject: false,
        tapped: false,
        hover: String::new(),
        printed: None,
        type_line: None,
        why_hint: None,
    }
}

/// The Waiting panel: each delayed trigger in the order it was made, its
/// words and when it can trigger, then each extra turn in the order it will
/// be taken.
fn waiting_zone(waiting: &Waiting) -> Option<ZoneView> {
    let triggers = waiting.delayed_triggers.iter().map(|trigger| {
        let fires = match trigger.duration {
            DelayedDuration::Once => "once",
            DelayedDuration::ThisTurn => "each time this turn",
        };
        let turn = match trigger.turn {
            TriggersIn::AnyTurn => "in any turn".to_string(),
            TriggersIn::TurnAfter(turn) => format!("in a turn after turn {turn}"),
            TriggersIn::ExtraTurn { in_progress: true, .. } => "in this extra turn".to_string(),
            TriggersIn::ExtraTurn { player, in_progress: false } => format!("in Player {player}'s extra turn"),
        };
        note(
            format!("{}'s delayed trigger {}", trigger.source, trigger.id.0),
            format!("\"{}\"\nPlayer {}'s · {fires} · {turn}", trigger.text, trigger.controller),
        )
    });
    let turns = waiting.extra_turns.iter().enumerate().map(|(place, turn)| {
        let when = if place == 0 { "taken next" } else { "taken after the one above" };
        note(format!("Player {}'s extra turn", turn.player), when.to_string())
    });
    let items: Vec<Item> = triggers.chain(turns).collect();
    (!items.is_empty()).then(|| ZoneView { name: format!("Waiting ({})", items.len()), key: "waiting", items, open: false })
}

fn owned(marks: &Marks, card: &CardView) -> Item {
    let mut item = marks.card(card);
    item.detail = format!("{} · Player {}'s", item.detail, card.owner);
    item
}

/// A seat's zones, the battlefield grouped the way `ui/display.rs` groups it:
/// creatures, then lands, then the rest, so a land creature is a creature.
fn seat(player: &PlayerView, marks: &Marks) -> SeatView {
    let group = |keep: &dyn Fn(&PermanentView) -> bool| -> Vec<Item> {
        player.battlefield.iter().filter(|p| keep(p)).map(|p| marks.permanent(p)).collect()
    };
    let battlefield = [
        ("Creatures", group(&|p| p.is_creature)),
        ("Lands", group(&|p| !p.is_creature && p.is_land)),
        ("Other permanents", group(&|p| !p.is_creature && !p.is_land)),
    ];
    let mut zones: Vec<ZoneView> = battlefield
        .into_iter()
        .filter(|(_, items)| !items.is_empty())
        .map(|(name, items)| ZoneView { name: name.to_string(), key: name, items, open: true })
        .collect();
    let counted = |name: &'static str, cards: &[CardView], open: bool| ZoneView {
        name: format!("{name} ({})", cards.len()),
        key: name,
        items: cards.iter().map(|card| marks.card(card)).collect(),
        open,
    };
    zones.push(counted("Hand", &player.hand, true));
    zones.push(counted("Graveyard", &player.graveyard, false));
    zones.push(counted("Library", &player.library, false));
    SeatView { player: marks.player(player), zones }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionButton {
    pub label: String,
    pub chosen: bool,
    /// Its place in an ordering, from 1.
    pub place: Option<usize>,
    /// An allocation's bucket, drawn as its amount between "−" and "+".
    pub amount: Option<Amount>,
    /// A click on it moves the answer along.
    pub live: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Amount {
    pub value: u64,
    pub can_lower: bool,
    pub can_raise: bool,
}

/// A `pick_number`'s field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberField {
    pub min: u64,
    pub max: u64,
    pub value: u64,
}

/// A control over the seat rather than an option of the prompt: a click on
/// it is `input`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeatButton {
    pub label: String,
    pub input: Input,
    pub live: bool,
}

/// The confirm button.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoneButton {
    pub label: String,
    pub live: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptView {
    /// The question, after the player it asks: `Player 1: Declare blockers`.
    pub question: String,
    /// Why the seat is asked again, when it is.
    pub rejected: Option<String>,
    /// What makes an answer complete.
    pub rule: String,
    pub options: Vec<OptionButton>,
    pub number: Option<NumberField>,
    /// The confirm button, when the prompt has one.
    pub done: Option<DoneButton>,
    /// "Start over" has something to undo.
    pub can_reset: bool,
    /// At a priority prompt, the yields: each passes now and keeps passing.
    pub yields: Vec<SeatButton>,
    /// The yield the seat holds, in words, and the button that ends it.
    pub yielding: Option<(String, SeatButton)>,
    /// `WindowState::prompts` when this prompt arrived.
    pub serial: u64,
}

impl PromptView {
    fn new(prompt: &Prompt, selection: &Selection, board: Option<&Snapshot>) -> PromptView {
        let live = |input: Input| selection.is_live(prompt, input);
        let options = prompt.options.iter().enumerate();
        let button = |i: usize, label: &str| OptionButton {
            label: label.to_string(),
            chosen: false,
            place: None,
            amount: None,
            live: live(Input::OptionButton(i)),
        };
        let done_button = |label: &str| DoneButton { label: label.to_string(), live: live(Input::Done) };
        let (rule, options, number, done): (String, Vec<OptionButton>, _, _) = match (&prompt.primitive, selection) {
            (Primitive::PickN { min, max }, Selection::Picks { chosen, half }) => {
                let rule = match half {
                    Some(half) => format!("now click what {} goes with", name_of(board, half)),
                    None => pick_rule(*min, *max),
                };
                // A lone option with a way out is a "may": yes or no.
                let may = *min == 0 && *max == 1 && prompt.options.len() == 1;
                let options = options
                    .map(|(i, o)| OptionButton {
                        chosen: chosen.contains(&i),
                        ..button(i, &if may { format!("Yes: {}", o.label) } else { o.label.clone() })
                    })
                    .collect();
                let done = match (*min, *max) {
                    (0, 1) => Some(done_button(if may { "No" } else { "Decline" })),
                    (_, 1) => None,
                    _ => Some(done_button("Done")),
                };
                (rule, options, None, done)
            }
            (Primitive::Number { min, max }, Selection::Number(value)) => {
                let top = if *max == u64::MAX { "any".to_string() } else { max.to_string() };
                let field = NumberField { min: *min, max: *max, value: *value };
                (format!("a number from {min} to {top}"), Vec::new(), Some(field), Some(done_button("Done")))
            }
            (Primitive::Allocate { total, .. }, Selection::Allocation(amounts)) => {
                let left = total.saturating_sub(amounts.iter().sum());
                let options = options
                    .map(|(i, o)| {
                        let amount =
                            Amount { value: amounts[i], can_lower: live(Input::OneFewer(i)), can_raise: live(Input::OneMore(i)) };
                        OptionButton { amount: Some(amount), ..button(i, &o.label) }
                    })
                    .collect();
                (format!("divide {total}: {left} left"), options, None, Some(done_button("Done")))
            }
            (Primitive::Order, Selection::Order(order)) => {
                let options = options
                    .map(|(i, o)| {
                        let place = order.iter().position(|placed| *placed == i).map(|at| at + 1);
                        OptionButton { chosen: place.is_some(), place, ..button(i, &o.label) }
                    })
                    .collect();
                let rule = format!("click them in order: {} of {} placed", order.len(), prompt.options.len());
                (rule, options, None, Some(done_button("Done")))
            }
            _ => (String::new(), Vec::new(), None, None),
        };
        let has_reset = !matches!(prompt.primitive, Primitive::PickN { max: 1, .. } | Primitive::Number { .. });
        let can_reset = has_reset && live(Input::Reset);
        PromptView {
            question: format!("Player {}: {}", prompt.player, prompt.question),
            rejected: prompt.rejected.clone(),
            rule,
            options,
            number,
            done,
            can_reset,
            yields: Vec::new(),
            yielding: None,
            serial: 0,
        }
    }

    /// Every control shown and none live: the beat after the prompt arrived.
    fn settling(mut self) -> PromptView {
        for option in &mut self.options {
            option.live = false;
            if let Some(amount) = &mut option.amount {
                (amount.can_lower, amount.can_raise) = (false, false);
            }
        }
        if let Some(done) = &mut self.done {
            done.live = false;
        }
        self.can_reset = false;
        for button in self.yields.iter_mut().chain(self.yielding.as_mut().map(|(_, stop)| stop)) {
            button.live = false;
        }
        self
    }
}

fn pick_rule(min: usize, max: usize) -> String {
    match (min, max) {
        (1, 1) => "choose one".to_string(),
        (0, 1) => "choose one, or decline".to_string(),
        (0, max) => format!("choose up to {max}"),
        (min, max) if min == max => format!("choose {min}"),
        (min, max) => format!("choose {min} to {max}"),
    }
}

/// `target` as the board titles it: `Hill Giant (#17)`, `Player 1`. A pair's
/// half is a permanent, so the battlefield is where it is looked for.
fn name_of(board: Option<&Snapshot>, target: &BoardRef) -> String {
    match target {
        BoardRef::Object(id) => board
            .and_then(|board| board.players.iter().flat_map(|p| &p.battlefield).find(|p| p.card.id == *id))
            .map_or_else(|| id.to_string(), |permanent| format!("{} ({id})", permanent.card.name)),
        BoardRef::Player(player) => format!("Player {player}"),
    }
}

#[cfg(test)]
mod tests {
    use mtgsim::objects::card_data::CardDataBuilder;
    use mtgsim::test_support::{
        card_of_type, forest, lightning_bolt, put_in_hand, put_on_battlefield, set_attacking, set_blocking,
        setup_two_player_game, vanilla_creature,
    };
    use mtgsim::types::card_types::CardType;
    use mtgsim::types::ids::ObjectId;

    use super::*;
    use crate::prompt::OptionView;
    use crate::snapshot::StackItem;
    use BoardRef::{Object, Player};

    struct Board {
        snapshot: Snapshot,
        bear: ObjectId,
        forest: ObjectId,
        arbor: ObjectId,
        relic: ObjectId,
        bolt: ObjectId,
        their_bear: ObjectId,
        their_giant: ObjectId,
    }

    fn board() -> Board {
        let mut game = setup_two_player_game();
        let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        let forest = put_on_battlefield(&mut game, forest(), 0);
        let dryad_arbor = CardDataBuilder::new("Dryad Arbor")
            .card_type(CardType::Land)
            .card_type(CardType::Creature)
            .power_toughness(1, 1)
            .build();
        let arbor = put_on_battlefield(&mut game, dryad_arbor, 0);
        let relic = put_on_battlefield(&mut game, card_of_type("Relic", CardType::Artifact), 0);
        let bolt = put_in_hand(&mut game, lightning_bolt(), 0);
        let their_bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
        let their_giant = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
        set_attacking(&mut game, their_bear, 0);
        set_attacking(&mut game, their_giant, 0);
        set_blocking(&mut game, arbor, vec![their_giant]);
        let snapshot = Snapshot::build(&game, 0);
        Board { snapshot, bear, forest, arbor, relic, bolt, their_bear, their_giant }
    }

    fn deciding(board: &Board, prompt: Prompt) -> WindowState {
        WindowState {
            board: Some(board.snapshot.clone()),
            selection: Some(Selection::start(&prompt)),
            prompt: Some(prompt),
            ..WindowState::default()
        }
    }

    fn prompt(primitive: Primitive, options: Vec<OptionView>) -> Prompt {
        Prompt {
            player: 0,
            kind: "Test".to_string(),
            question: String::new(),
            subject: None,
            pass: None,
            rejected: None,
            why_reads_the_trace: false,
            primitive,
            options,
        }
    }

    fn option(label: &str, refs: Vec<BoardRef>) -> OptionView {
        OptionView { label: label.to_string(), refs }
    }

    fn unnamed(count: usize) -> Vec<OptionView> {
        (0..count).map(|i| option(&i.to_string(), Vec::new())).collect()
    }

    fn item(view: &BoardView, id: ObjectId) -> Item {
        view.seats
            .iter()
            .flat_map(|seat| seat.zones.iter().flat_map(|zone| zone.items.iter()))
            .find(|item| item.target == Some(Object(id)))
            .cloned()
            .unwrap_or_else(|| panic!("{id} is not on the board"))
    }

    #[test]
    fn a_hover_greys_the_land_types_blood_moon_took_and_a_hand_card_prints_in_order() {
        let mut game = setup_two_player_game();
        let bayou = put_on_battlefield(&mut game, mtgsim::cards::dual_lands::bayou(), 0);
        put_on_battlefield(&mut game, mtgsim::cards::phase_ld_cards::blood_moon(), 1);
        let priest = put_in_hand(&mut game, mtgsim::cards::phase_rc_cards::containment_priest(), 0);
        let view = WindowState { board: Some(Snapshot::build(&game, 0)), ..WindowState::default() }.board_view().unwrap();

        let line = item(&view, bayou).type_line.expect("a permanent's hover has its type line");
        let words = |section: &[TypeWordView]| section.iter().map(|w| (w.text.clone(), w.faded)).collect::<Vec<_>>();
        assert_eq!(words(&line.front), [("Land".to_string(), false)]);
        assert_eq!(
            words(&line.subtypes),
            [("Swamp".to_string(), true), ("Forest".to_string(), true), ("Mountain".to_string(), false)]
        );
        assert_eq!(line.text, "Land — Mountain");
        assert_eq!(item(&view, priest).detail, "Creature — Human Cleric");
        assert_eq!(item(&view, priest).type_line, None, "a hand card's line is its detail");
    }

    /// The Waiting panel's rows (`ui::waiting`): Blessed Wine's draw and
    /// Final Fortune's loss in the order they were made, each with its words
    /// and when it can trigger, then the extra turn; closed until opened.
    #[test]
    fn the_waiting_panel_lists_each_delayed_trigger_then_each_extra_turn() {
        use mtgsim::cards::phase_tr3a_cards::{blessed_wine, final_fortune};
        use mtgsim::engine::resolve::ResolutionContext;
        let mut game = setup_two_player_game();
        let wine = put_in_hand(&mut game, blessed_wine(), 0);
        let fortune = put_in_hand(&mut game, final_fortune(), 0);
        let nothing = WindowState { board: Some(Snapshot::build(&game, 0)), ..WindowState::default() }.board_view().unwrap();
        assert_eq!(nothing.waiting, None, "not shown while nothing waits");
        for (id, ability) in [(wine, 1), (fortune, 0)] {
            let effect = game.get_object(id).unwrap().card_data.abilities[ability].effect.clone();
            game.resolve_effect(&effect, &ResolutionContext::untargeted(id, 0), &mtgsim::test_support::test_dp()).unwrap();
        }
        let view = WindowState { board: Some(Snapshot::build(&game, 0)), ..WindowState::default() }.board_view().unwrap();
        let waiting = view.waiting.expect("shown once something waits");

        let rows: Vec<(String, String)> = waiting.items.iter().map(|item| (item.title.clone(), item.detail.clone())).collect();
        let turn = game.turn_number;
        assert_eq!(
            rows,
            [
                (
                    format!("Blessed Wine ({wine})'s delayed trigger 1"),
                    format!("\"Draw a card at the beginning of the next turn's upkeep.\"\nPlayer 0's · once · in a turn after turn {turn}"),
                ),
                (
                    format!("Final Fortune ({fortune})'s delayed trigger 2"),
                    "\"At the beginning of that turn's end step, you lose the game.\"\nPlayer 0's · once · in Player 0's extra turn".to_string(),
                ),
                ("Player 0's extra turn".to_string(), "taken next".to_string()),
            ]
        );
        assert_eq!((waiting.name.as_str(), waiting.open), ("Waiting (3)", false));
        assert!(waiting.items.iter().all(|item| !item.clickable && item.target.is_none()), "nothing in it is a choice");
    }

    #[test]
    fn the_battlefield_groups_as_the_cli_does_and_a_land_creature_is_a_creature() {
        let b = board();
        let view = WindowState { board: Some(b.snapshot.clone()), ..WindowState::default() }.board_view().unwrap();
        let mine = view.seats.last().unwrap();
        assert_eq!(mine.player.target, Some(Player(0)), "seat 0 is drawn last, nearest the prompt");
        let names: Vec<&str> = mine.zones.iter().map(|zone| zone.name.as_str()).collect();
        assert_eq!(names, ["Creatures", "Lands", "Other permanents", "Hand (1)", "Graveyard (0)", "Library (0)"]);
        let keys: Vec<&str> = mine.zones.iter().map(|zone| zone.key).collect();
        assert_eq!(keys, ["Creatures", "Lands", "Other permanents", "Hand", "Graveyard", "Library"], "no count in a key");
        let targets = |zone: &ZoneView| zone.items.iter().map(|item| item.target).collect::<Vec<_>>();
        assert_eq!(targets(&mine.zones[0]), [Some(Object(b.bear)), Some(Object(b.arbor))]);
        assert_eq!(targets(&mine.zones[1]), [Some(Object(b.forest))]);
        assert_eq!(targets(&mine.zones[2]), [Some(Object(b.relic))]);
        assert!(item(&view, b.bear).detail.starts_with("2/2"), "{}", item(&view, b.bear).detail);
        assert!(!item(&view, b.relic).detail.contains('/'), "no power or toughness off a creature");
        assert_eq!(item(&view, b.relic).printed.as_deref().unwrap_or_default(), ["Relic\nArtifact".to_string()], "the card as printed beside the hover");
        assert!(item(&view, b.their_bear).detail.contains("attacking Player 0"));
        let giant = item(&view, b.their_giant).title;
        assert!(item(&view, b.arbor).detail.ends_with(&format!("blocking {giant}")), "named as the board titles it");
    }

    #[test]
    fn a_single_pick_answers_on_the_click_on_its_button_or_on_the_board() {
        let b = board();
        let options = vec![option("Pass", Vec::new()), option("Cast Lightning Bolt", vec![Object(b.bolt)])];
        let state = deciding(&b, prompt(Primitive::PickN { min: 1, max: 1 }, options));
        let view = state.board_view().unwrap();
        assert!(item(&view, b.bolt).clickable);
        assert!(!item(&view, b.bear).clickable, "no option names it");
        assert_eq!(state.clone().input(Input::Board(Object(b.bear))), None);
        assert_eq!(state.clone().input(Input::OptionButton(0)), Some(Reply::Answer(Answer::Picks(vec![0]))));
        let mut clicked = state.clone();
        assert_eq!(clicked.input(Input::Board(Object(b.bolt))), Some(Reply::Answer(Answer::Picks(vec![1]))));
        assert!(clicked.prompt.is_none(), "the answer closes the prompt");
    }

    #[test]
    fn a_blocker_with_two_attackers_to_choose_between_takes_a_second_click() {
        let b = board();
        let blocks = vec![
            option("bear blocks bear", vec![Object(b.bear), Object(b.their_bear)]),
            option("bear blocks giant", vec![Object(b.bear), Object(b.their_giant)]),
            option("arbor blocks bear", vec![Object(b.arbor), Object(b.their_bear)]),
        ];
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 0, max: 3 }, blocks));
        assert_eq!(state.input(Input::Board(Object(b.bear))), None);
        let view = state.board_view().unwrap();
        assert!(item(&view, b.bear).chosen, "the half shows as chosen");
        assert!(item(&view, b.their_bear).clickable && item(&view, b.their_giant).clickable);
        assert!(!item(&view, b.arbor).clickable, "only the half's partners, until it is completed");
        let bear = item(&view, b.bear).title;
        assert_eq!(state.prompt_view().unwrap().rule, format!("now click what {bear} goes with"));
        state.input(Input::Board(Object(b.their_giant)));
        state.input(Input::Board(Object(b.arbor)));
        assert_eq!(state.input(Input::Done), Some(Reply::Answer(Answer::Picks(vec![1, 2]))));
    }

    #[test]
    fn a_click_on_what_a_pair_ends_at_starts_nothing() {
        let b = board();
        let blocks = vec![option("bear blocks bear", vec![Object(b.bear), Object(b.their_bear)])];
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 0, max: 1 }, blocks));
        assert!(!item(&state.board_view().unwrap(), b.their_bear).clickable);
        assert_eq!(state.input(Input::Board(Object(b.their_bear))), None);
        assert_eq!(state.selection, Some(Selection::Picks { chosen: Vec::new(), half: None }), "no half waits for a partner");
    }

    #[test]
    fn a_lone_option_with_a_way_out_reads_as_yes_or_no() {
        let b = board();
        let state = deciding(&b, prompt(Primitive::PickN { min: 0, max: 1 }, vec![option("Blood Artist", Vec::new())]));
        let view = state.prompt_view().unwrap();
        assert_eq!(view.options[0].label, "Yes: Blood Artist");
        assert_eq!(view.done, Some(DoneButton { label: "No".to_string(), live: true }));
        assert_eq!(state.clone().input(Input::Done), Some(Reply::Answer(Answer::Picks(Vec::new()))));
    }

    #[test]
    fn done_is_live_only_inside_the_bounds() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 2, max: 3 }, unnamed(4)));
        state.input(Input::OptionButton(0));
        assert_eq!(state.prompt_view().unwrap().done.map(|done| done.live), Some(false));
        assert_eq!(state.input(Input::Done), None);
        for i in [3, 1, 2] {
            state.input(Input::OptionButton(i));
        }
        assert_eq!(state.prompt_view().unwrap().done.map(|done| done.live), Some(true));
        state.input(Input::OptionButton(3));
        assert_eq!(
            state.input(Input::Done),
            Some(Reply::Answer(Answer::Picks(vec![0, 1]))),
            "a pick past the maximum is refused, and a second click undoes one"
        );
    }

    #[test]
    fn an_allocation_stays_inside_each_bucket_and_is_done_when_the_total_is_placed() {
        let b = board();
        let buckets = vec![option("bear", vec![Object(b.their_bear)]), option("player", vec![Player(1)])];
        let split = Primitive::Allocate { total: 3, mins: vec![1, 0], maxs: Some(vec![2, 3]) };
        let mut state = deciding(&b, prompt(split, buckets));
        state.input(Input::OneFewer(0));
        for _ in 0..3 {
            state.input(Input::OneMore(0));
        }
        let amount = state.prompt_view().unwrap().options[0].amount;
        assert_eq!(amount, Some(Amount { value: 2, can_lower: true, can_raise: false }), "held between its minimum and its maximum");
        assert_eq!(state.input(Input::Done), None, "one still to place");
        state.input(Input::Board(Player(1)));
        assert_eq!(state.input(Input::Done), Some(Reply::Answer(Answer::Allocation(vec![2, 1]))));
    }

    #[test]
    fn an_ordering_is_the_click_order_and_starts_over_on_reset() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::Order, unnamed(3)));
        for i in [2, 0, 2] {
            state.input(Input::OptionButton(i));
        }
        assert_eq!(state.prompt_view().unwrap().options[2].place, Some(1));
        assert_eq!(state.input(Input::Done), None);
        state.input(Input::Reset);
        for i in [1, 2, 0] {
            state.input(Input::OptionButton(i));
        }
        assert_eq!(state.input(Input::Done), Some(Reply::Answer(Answer::Order(vec![1, 2, 0]))));
    }

    #[test]
    fn a_number_is_held_to_its_range() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::Number { min: 2, max: 5 }, Vec::new()));
        state.input(Input::Number(9));
        assert_eq!(state.input(Input::Done), Some(Reply::Answer(Answer::Number(5))));
    }

    #[test]
    fn a_click_the_window_offers_always_moves_the_answer_along() {
        let b = board();
        // Two abilities of one permanent: its buttons tell them apart, and a
        // click on the permanent could mean either, so it is no target.
        let abilities = vec![option("ability 0", vec![Object(b.forest)]), option("ability 1", vec![Object(b.forest)])];
        let state = deciding(&b, prompt(Primitive::PickN { min: 0, max: 1 }, abilities));
        assert!(!item(&state.board_view().unwrap(), b.forest).clickable);

        let mut order = deciding(&b, prompt(Primitive::Order, unnamed(2)));
        assert!(!order.prompt_view().unwrap().can_reset, "nothing to start over yet");
        order.input(Input::OptionButton(1));
        let view = order.prompt_view().unwrap();
        assert!(view.options[0].live && !view.options[1].live, "a placed option's button is spent");
        assert!(view.can_reset);

        let mut picks = deciding(&b, prompt(Primitive::PickN { min: 0, max: 2 }, unnamed(3)));
        picks.input(Input::OptionButton(0));
        picks.input(Input::OptionButton(2));
        let live: Vec<bool> = picks.prompt_view().unwrap().options.iter().map(|o| o.live).collect();
        assert_eq!(live, [true, false, true], "a pick past the maximum is refused, and a chosen one can be undone");
    }

    #[test]
    fn the_prompts_subject_is_marked_on_the_board() {
        let b = board();
        let mut about = prompt(Primitive::PickN { min: 1, max: 1 }, unnamed(2));
        about.subject = Some(b.bear);
        let view = deciding(&b, about).board_view().unwrap();
        assert!(item(&view, b.bear).subject);
        assert!(!item(&view, b.arbor).subject);
    }

    fn at_priority(b: &Board) -> WindowState {
        let options = vec![option("Pass", Vec::new()), option("Cast Lightning Bolt", vec![Object(b.bolt)])];
        deciding(b, Prompt { pass: Some(0), ..prompt(Primitive::PickN { min: 1, max: 1 }, options) })
    }

    /// A yield answers the priority prompt it is set at; the one that waits
    /// on the stack needs a stack, and full control turns them all off.
    #[test]
    fn a_yield_answers_a_priority_prompt_and_full_control_supersedes_it() {
        let b = board();
        let state = at_priority(&b);
        let live: Vec<bool> = state.prompt_view().unwrap().yields.iter().map(|button| button.live).collect();
        assert_eq!(live, [true, false, true], "an empty stack has nothing to wait on");
        assert_eq!(state.clone().input(Input::Yield(Yield::UntilStackChanges)), None);
        let mut over_a_stack = state.clone();
        let spell = StackItem { id: b.bolt, name: "Lightning Bolt".to_string(), is_spell: Some(true), controller: 1, targets: Vec::new(), x: None };
        over_a_stack.board.as_mut().unwrap().stack.push(spell);
        assert!(over_a_stack.prompt_view().unwrap().yields[1].live);
        let mut yielded = state.clone();
        assert_eq!(yielded.input(Input::Yield(Yield::UntilEndOfTurn)), Some(Reply::Yield(Yield::UntilEndOfTurn)));
        assert!(yielded.prompt.is_none(), "the yield answered the prompt");

        let mut full = state.clone();
        full.input(Input::FullControl(true));
        assert!(full.prompt_view().unwrap().yields.iter().all(|button| !button.live));
        assert_eq!(full.input(Input::Yield(Yield::UntilEndOfTurn)), None);
    }

    /// Only a priority prompt offers a yield; while one holds, any prompt
    /// offers to stop it, which leaves the prompt open.
    #[test]
    fn stopping_a_yield_leaves_the_prompt_open() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 0, max: 3 }, unnamed(3)));
        state.yielding = Some(Yield::UntilYourNextTurn);
        let view = state.prompt_view().unwrap();
        assert!(view.yields.is_empty());
        let (words, stop) = view.yielding.unwrap();
        assert_eq!(words, "Passing until Player 0's next turn");
        assert_eq!(state.input(stop.input), Some(Reply::StopYielding));
        assert!(state.prompt.is_some() && state.yielding.is_none());
        assert_eq!(state.input(Input::StopYielding), None, "nothing left to stop");
    }

    /// Item 201: a click in the moment after a prompt arrives was aimed at
    /// the one before, so it is dropped, and the prompt shows nothing live
    /// until the moment has passed. A held key's repeat never answers.
    #[test]
    fn input_in_the_beat_after_a_prompt_arrives_is_dropped() {
        let b = board();
        let mut state = WindowState::default();
        state.tick(10.0);
        state.receive(ToWindow::Prompt { snapshot: b.snapshot.clone(), prompt: prompt(Primitive::PickN { min: 1, max: 1 }, unnamed(2)), yielding: None, why: None });
        state.tick(10.1);
        assert_eq!(state.input(Input::OptionButton(0)), None, "the second click of a double click");
        assert!(state.prompt_view().unwrap().options.iter().all(|option| !option.live));
        state.tick(10.0 + 2.0 * SETTLE_SECONDS);
        assert_eq!(state.input(Input::Key { key: Key::Digit(1), repeat: true }), None, "a held key");
        assert_eq!(state.input(Input::Key { key: Key::Digit(2), repeat: false }), Some(Reply::Answer(Answer::Picks(vec![1]))));
    }

    /// The why panel opening or closing slides the board under the pointer,
    /// so a click on the board in the beat after either is dropped, and the
    /// board offers nothing until it has passed: a double click on the
    /// panel's close would otherwise answer a target on the player line the
    /// close was drawn over. The prompt's own buttons do not move.
    #[test]
    fn a_board_click_in_the_beat_after_the_why_panel_opens_or_closes_is_dropped() {
        let b = board();
        let player = BoardRef::Player(0);
        let mut state = WindowState::default();
        state.tick(10.0);
        state.receive(ToWindow::Prompt {
            snapshot: b.snapshot.clone(),
            prompt: prompt(Primitive::PickN { min: 1, max: 1 }, vec![option("Player 0", vec![player])]),
            yielding: None,
            why: None,
        });
        let offered = |state: &WindowState| state.board_view().unwrap().seats.iter().any(|seat| seat.player.clickable);
        state.tick(11.0);
        assert!(offered(&state), "the prompt's beat has passed");
        assert_eq!(state.input(Input::Why(BoardRef::Object(b.bear))), Some(Reply::Why(Some(BoardRef::Object(b.bear)))));
        assert!(offered(&state), "nothing shows until the seat answers");
        state.receive(ToWindow::Why(Why { title: "Bear".to_string(), sections: Vec::new() }));
        state.tick(11.1);
        assert!(!offered(&state), "the panel opened");
        assert_eq!(state.input(Input::Board(player)), None);
        assert!(state.prompt_view().unwrap().options[0].live, "the prompt's buttons stayed where they were");
        state.tick(12.0);
        assert_eq!(state.input(Input::WhyClose), Some(Reply::Why(None)));
        state.tick(12.1);
        assert_eq!(state.input(Input::Board(player)), None, "the second click of a double click on the close");
        state.tick(12.0 + 2.0 * SETTLE_SECONDS);
        assert_eq!(state.input(Input::Board(player)), Some(Reply::Answer(Answer::Picks(vec![0]))));
    }

    /// A key is a click the prompt already offers: Space is the pass a
    /// priority prompt offers, Enter the confirm button, F4 a yield.
    #[test]
    fn a_key_stands_for_a_click_the_prompt_offers() {
        let b = board();
        let state = at_priority(&b);
        let pressed = |key| state.clone().input(Input::Key { key, repeat: false });
        assert_eq!(pressed(Key::Space), Some(Reply::Answer(Answer::Picks(vec![0]))));
        assert_eq!(pressed(Key::F4), Some(Reply::Yield(Yield::UntilEndOfTurn)));
        assert_eq!(pressed(Key::Digit(9)), None, "no ninth option");
        assert_eq!(pressed(Key::Enter), None, "a single pick has no confirm button");
        let decline = deciding(&b, prompt(Primitive::PickN { min: 0, max: 1 }, unnamed(1)));
        assert_eq!(decline.clone().input(Input::Key { key: Key::Enter, repeat: false }), Some(Reply::Answer(Answer::Picks(Vec::new()))));
        assert_eq!(decline.clone().input(Input::Key { key: Key::Space, repeat: false }), None, "no pass outside priority");
    }

    #[test]
    fn a_panic_closes_the_prompt_and_says_so() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 1, max: 1 }, unnamed(2)));
        let message = "ask_choose_priority_action: DP returned index 9".to_string();
        state.receive(ToWindow::Panicked { message });
        assert!(state.prompt.is_none() && state.prompt_view().is_none());
        assert_eq!(state.status(), "The engine panicked");
        assert_eq!(state.input(Input::OptionButton(0)), None);
        let before_any_board = WindowState { panic: Some("early".to_string()), ..WindowState::default() };
        assert_eq!(before_any_board.no_board(), "No board: the engine panicked before its first prompt.");
    }

    /// The window plays every seat (`setup-architecture.md` §7's "Seats"):
    /// a prompt says whose it is, the board marks that seat and keeps one
    /// order, and a yield names the turn it waits for by its seat.
    #[test]
    fn a_prompt_names_the_seat_it_asks_and_the_board_keeps_its_order() {
        let b = board();
        let options = vec![option("Pass", Vec::new()), option("Cast Lightning Bolt", vec![Object(b.bolt)])];
        let asking = |player| {
            let question = "You have priority".to_string();
            let priority = prompt(Primitive::PickN { min: 1, max: 1 }, options.clone());
            deciding(&b, Prompt { player, pass: Some(0), question, ..priority })
        };
        let theirs = asking(1);
        assert_eq!(theirs.status(), "Player 1 to decide");
        let view = theirs.prompt_view().unwrap();
        assert_eq!(view.question, "Player 1: You have priority");
        let next_turn = view.yields.iter().find(|button| button.input == Input::Yield(Yield::UntilYourNextTurn)).unwrap();
        assert_eq!(next_turn.label, "Pass until Player 1's next turn", "a seat's yield waits for that seat's turn");
        for (asked, state) in [(0, asking(0)), (1, theirs)] {
            let titles: Vec<String> = state.board_view().unwrap().seats.iter().map(|seat| seat.player.title.clone()).collect();
            let named = |p: PlayerId| if p == asked { format!("Player {p} (to decide)") } else { format!("Player {p}") };
            assert_eq!(titles, [named(1), named(0)], "one order whichever seat is asked");
        }
        let won = WindowState { outcome: Some(Outcome::Won(0)), ..WindowState::default() };
        assert_eq!(won.status(), "Game over: Player 0 wins");
    }

    /// A why answers nothing: it leaves the prompt and the answer in progress
    /// as they were, and is asked only at an open question, where a seat is
    /// waiting to answer it.
    #[test]
    fn a_why_answers_nothing_and_is_asked_only_at_an_open_question() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 1, max: 2 }, unnamed(3)));
        state.input(Input::OptionButton(1));
        let (prompt_before, selection_before) = (state.prompt.clone(), state.selection.clone());
        let bear = BoardRef::Object(b.bear);
        assert_eq!(state.input(Input::Why(bear)), Some(Reply::Why(Some(bear))));
        assert_eq!((&state.prompt, &state.selection), (&prompt_before, &selection_before));
        assert_eq!(state.input(Input::Why(bear)), Some(Reply::Why(Some(bear))), "asked again, as a refresh");
        assert_eq!(state.why_path, [bear.why_about()], "and not a second step back");
        let player = BoardRef::Player(1);
        assert_eq!(state.input(Input::Why(player)), Some(Reply::Why(Some(player))), "a player's line asks too");
        let idle = WindowState { board: Some(b.snapshot.clone()), ..WindowState::default() };
        assert_eq!(idle.clone().input(Input::Why(bear)), None, "no seat is waiting to answer");
        let hint = |state: &WindowState| item(&state.board_view().expect("a board"), b.bear).why_hint;
        assert_eq!(hint(&state), Some(WHY_ON_RIGHT_CLICK));
        assert_eq!(hint(&idle), Some(WHY_ONLY_AT_A_QUESTION), "and its hover says so");
    }

    /// Back returns to the object asked about before, while there is one;
    /// closing empties the panel and tells the seats to stop answering.
    #[test]
    fn back_returns_along_the_objects_asked_and_close_empties_the_panel() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 1, max: 1 }, unnamed(2)));
        state.input(Input::Why(BoardRef::Object(b.bear)));
        state.input(Input::Why(BoardRef::Object(b.relic)));
        assert_eq!(state.input(Input::WhyBack), Some(Reply::Why(Some(BoardRef::Object(b.bear)))));
        assert_eq!(state.input(Input::WhyBack), None, "nothing before the first");
        state.why = Some(Why { title: "Bear".to_string(), sections: Vec::new() });
        assert_eq!(state.input(Input::WhyClose), Some(Reply::Why(None)));
        assert!(state.why_path.is_empty() && state.why_view().is_none());
    }

    /// Each object a line names is a link while a question is open; with
    /// none open the panel keeps its last answer, its links off, and says so.
    #[test]
    fn the_panels_links_are_live_only_at_an_open_question() {
        use mtgsim::ui::why::WhySection;
        let b = board();
        let named = WhyLine { text: "Layer 6".to_string(), rule: Some("613.1f"), names: vec![(b.relic, "Relic (#4)".to_string())], depth: 0 };
        let why = Why { title: "Bear (#1)".to_string(), sections: vec![WhySection { heading: "What the layers did".to_string(), lines: vec![named] }] };
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 1, max: 1 }, unnamed(2)));
        (state.why_path, state.why) = (vec![WhyAbout::Object(b.bear)], Some(why));
        let view = state.why_view().expect("an answer to show");
        let line = &view.sections[0].lines[0];
        assert_eq!((line.rule.as_deref(), line.depth), (Some("CR 613.1f"), 0));
        assert_eq!(line.links, [WhyLink { label: "Relic (#4)".to_string(), input: Input::Why(BoardRef::Object(b.relic)), live: true }]);
        assert_eq!((view.back, view.note), (false, None));
        state.prompt = None;
        let idle = state.why_view().expect("kept with no question open");
        assert!(!idle.sections[0].lines[0].links[0].live);
        assert_eq!(idle.note, Some(WHY_AT_A_QUESTION));
    }

    fn answer(title: &str) -> Why {
        Why { title: title.to_string(), sections: Vec::new() }
    }

    /// A log line's why is a replay's to answer: the panel waits on it, saying
    /// so, the seat stops following, and only the answer to the request the
    /// panel waits on is shown. A question arriving meanwhile leaves it be.
    #[test]
    fn a_log_lines_why_is_read_from_the_trace() {
        let b = board();
        let mut state = deciding(&b, prompt(Primitive::PickN { min: 1, max: 1 }, unnamed(2)));
        state.input(Input::Why(Object(b.bear)));
        let event = WhyAbout::Event(EventSeq(7));
        assert_eq!(state.input(Input::WhyEvent(EventSeq(7))), Some(Reply::Why(None)), "the seat stops following");
        assert_eq!(state.reading_the_trace, Some(TraceRequest { about: event, number: 1 }));
        let waiting = state.why_view().expect("the panel says it waits");
        assert_eq!((waiting.note, waiting.sections.len(), waiting.back), (Some(WHY_READING_THE_TRACE), 0, true));

        state.receive(ToWindow::WhyFromTrace { request: 0, why: answer("a request since replaced") });
        assert_eq!(state.why, None);
        let next = ToWindow::Prompt { snapshot: b.snapshot.clone(), prompt: prompt(Primitive::PickN { min: 1, max: 1 }, unnamed(2)), yielding: None, why: None };
        state.receive(next);
        assert_eq!(state.reading_the_trace.map(|r| r.number), Some(1), "an event's answer is still on its way");
        state.receive(ToWindow::WhyFromTrace { request: 1, why: answer("DamageDealt") });
        assert_eq!((state.why.as_ref().map(|why| why.title.as_str()), state.reading_the_trace), (Some("DamageDealt"), None));
        assert_eq!(state.input(Input::WhyBack), Some(Reply::Why(Some(Object(b.bear)))), "back to the seat's answer");
        assert_eq!(state.why_path, [WhyAbout::Object(b.bear)]);
    }

    /// At a question whose why reads the trace, the seat follows the object
    /// and a replay answers; at the next such question the panel asks again.
    /// Once the game is over, with no seat, every why is a replay's.
    #[test]
    fn a_question_whose_why_reads_the_trace_and_a_game_over_ask_a_replay() {
        let b = board();
        let reads = Prompt { why_reads_the_trace: true, ..prompt(Primitive::Order, unnamed(2)) };
        let mut state = deciding(&b, reads.clone());
        assert_eq!(state.input(Input::Why(Object(b.bear))), Some(Reply::Why(Some(Object(b.bear)))));
        assert_eq!(state.reading_the_trace.map(|r| (r.about, r.number)), Some((WhyAbout::Object(b.bear), 1)));
        state.receive(ToWindow::Prompt { snapshot: b.snapshot.clone(), prompt: reads, yielding: None, why: None });
        assert_eq!(state.reading_the_trace.map(|r| r.number), Some(2), "followed through the next one");

        let mut over = WindowState { board: Some(b.snapshot.clone()), outcome: Some(Outcome::Won(0)), ..WindowState::default() };
        assert_eq!(over.input(Input::WhyEvent(EventSeq(3))), None, "no seat to tell");
        assert_eq!(over.reading_the_trace.map(|r| r.about), Some(WhyAbout::Event(EventSeq(3))));
        assert_eq!(over.log_hint(), Some(WHY_ON_A_LOG_LINE));
        assert_eq!(item(&over.board_view().unwrap(), b.bear).why_hint, Some(WHY_ON_RIGHT_CLICK));
        over.receive(ToWindow::WhyFromTrace { request: 1, why: answer("an event") });
        assert!(over.why_view().unwrap().note.is_none(), "a game over answers, and its links ask");
    }
}
