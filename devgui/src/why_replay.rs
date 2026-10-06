//! A why about then (`setup-architecture.md` §7c, decision 3): what an event
//! did, and what a question's options are from the trace, are read from a
//! replay. The game is built again from its start and the window's line is
//! replayed into it, as Undo answer's rebuild replays it, with three
//! differences: the trace sink is on, nothing is recorded, and the seat behind
//! the line reads the why at the first question the line does not answer,
//! which is the window's open one, and stops the run there. A line that ends
//! with the game is answered from the game's end. The game's own thread keeps
//! waiting at its question meanwhile.

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread::JoinHandle;

use mtgsim::cards::registry::CardRegistry;
use mtgsim::state::decision_log::{AnswerLine, GameStart};
use mtgsim::state::game::Halt;
use mtgsim::state::game_state::GameState;
use mtgsim::state::trace::{TraceHandle, TraceMemory, TraceRecord, TraceSink};
use mtgsim::types::ids::PlayerId;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, SeatMode, Stop};
use mtgsim::ui::replay::{Replay, ReplayControl};
use mtgsim::ui::why::{OpenQuestion, Why, WhyAbout, WhyLine, WhySection, why_from_trace};

use crate::bridge::panic_message;
use crate::view_model::TraceRequest;

/// One why on its way from a replay: which request it answers, and the
/// channel its answer comes back on.
pub struct WhyReplay {
    pub request: u64,
    pub answer: Receiver<Why>,
    control: Arc<ReplayControl>,
    pub thread: JoinHandle<()>,
}

impl WhyReplay {
    /// `request` answered on a thread of its own, from a replay of `line`,
    /// the window's line to its open question or to the game's end. `wake`
    /// runs once the answer is sent.
    pub fn start(start: GameStart, line: Vec<AnswerLine>, request: TraceRequest, wake: Arc<dyn Fn() + Send + Sync>) -> WhyReplay {
        let (send, answer) = mpsc::channel();
        let control = Arc::new(ReplayControl::default());
        let stop = Arc::clone(&control);
        let thread = std::thread::Builder::new()
            .name("why replay".to_string())
            .spawn(move || {
                let read = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| read(&start, line, request.about, stop)));
                let why = match read {
                    Ok(Some(why)) => why,
                    // Superseded: a newer request is the window's now.
                    Ok(None) => return,
                    Err(payload) => trouble(request.about, "The replay panicked", &panic_message(payload.as_ref())),
                };
                // Fails only once the window has dropped the request, which wants nothing.
                if send.send(why).is_ok() {
                    wake();
                }
            })
            .expect("the OS refused a thread for a why's replay");
        WhyReplay { request: request.number, answer, control, thread }
    }

    /// A newer request replaces this one: it stops at its next answer, and
    /// what it would have said goes nowhere.
    pub fn supersede(&self) {
        self.control.supersede();
    }
}

/// The game built from `start`, `line` replayed into it with the sink on,
/// and `about`'s why read where the line ends; `None` once superseded.
fn read(start: &GameStart, line: Vec<AnswerLine>, about: WhyAbout, stop: Arc<ReplayControl>) -> Option<Why> {
    let mut built = match start.build(&CardRegistry::default_registry()) {
        Ok(built) => built,
        Err(refusal) => return Some(trouble(about, "The game did not build again", &refusal)),
    };
    let memory = TraceMemory::default();
    let state = &mut built.game_mut().state;
    state.install_trace(TraceHandle::new(&TraceSink::to_writer(memory.clone())));
    // This build checked every answer on the line as it was given.
    state.pause_layer_audit();
    let seat = AtTheQuestion { about, memory: memory.clone(), answer: RefCell::new(None) };
    let replay = Replay::new(line).with_control(stop).then(&seat);
    match built.replay(&replay) {
        Err(Halt::Stopped(Stop::Superseded)) => None,
        Err(Halt::Stopped(Stop::LogSpent { .. })) => seat.answer.take(),
        // The line ends with the game: what happened is all in its trace.
        Ok(_) | Err(Halt::Error(_)) => Some(answer(&built.game().state, about, None, &memory)),
        Err(Halt::Stopped(stop)) => Some(trouble(about, "The replay stopped short of the question", &stop.to_string())),
    }
}

/// The seat behind a why's replay. The line answers every question before
/// the open one, so the first question this is asked is the open one: it
/// reads the why there, against the board that question shows, and stops.
struct AtTheQuestion {
    about: WhyAbout,
    memory: TraceMemory,
    answer: RefCell<Option<Why>>,
}

impl AtTheQuestion {
    fn read(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption]) -> ! {
        let question = OpenQuestion { player, context, options };
        *self.answer.borrow_mut() = Some(answer(game, self.about, Some(&question), &self.memory));
        Stop::LogSpent { answered: 0 }.raise()
    }
}

impl DecisionProvider for AtTheQuestion {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], _: (usize, usize)) -> Vec<usize> {
        self.read(game, player, context, options)
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, _: u64, _: u64) -> u64 {
        self.read(game, player, context, &[])
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        _: u64,
        buckets: &[ChoiceOption],
        _: &[u64],
        _: Option<&[u64]>,
    ) -> Vec<u64> {
        self.read(game, player, context, buckets)
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.read(game, player, context, items)
    }

    /// Asked at every priority point, as a replay with no seat is, so the
    /// first question past the line is the next one the line's game asked.
    fn seat_mode(&self, _: PlayerId) -> SeatMode {
        SeatMode { stops_at_every_priority_point: true }
    }
}

/// `about`'s why from what the sink has written so far, its own branch's.
fn answer(game: &GameState, about: WhyAbout, at: Option<&OpenQuestion>, memory: &TraceMemory) -> Why {
    let branch = game.trace_handle().map(|handle| {
        // Flushed first: the sink buffers.
        let _ = handle.sink().flush();
        handle.branch()
    });
    match TraceRecord::read_all(&memory.text()) {
        Ok(records) => {
            let records: Vec<TraceRecord> = records.into_iter().filter(|record| Some(record.branch) == branch).collect();
            why_from_trace(game, about, at, &records)
        }
        Err(why) => trouble(about, "The trace did not read back", &why),
    }
}

/// An answer that is a bug report: what went wrong, in the panel.
fn trouble(about: WhyAbout, what: &str, detail: &str) -> Why {
    let lines = vec![WhyLine { text: format!("{what}: {detail}"), rule: None, names: Vec::new(), depth: 0 }];
    let title = match about {
        WhyAbout::Object(id) => id.to_string(),
        WhyAbout::Player(player) => format!("Player {player}"),
        WhyAbout::Event(event) => format!("Event {}", event.0),
    };
    Why { title, sections: vec![WhySection { heading: "The replay".to_string(), lines }] }
}
