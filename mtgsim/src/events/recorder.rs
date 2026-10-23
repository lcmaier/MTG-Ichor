//! The event recorder: the whole performed stream, for whatever reads it
//! (`codebase-state.md` item 42).
//!
//! The engine keeps a record only until its last reader has run
//! ([`EventWindow`]), so the performed stream is not game state: what a card
//! can ask of the past is the history log's rows (`state::history_log`). What
//! wants all of it — `fuzz_games`' fixture rows, `--dump-events`, the fork
//! test, a GUI's game log, a test asserting on what happened — attaches a
//! recorder, and the window hands it each record as it flushes, in stream order.
//! Beside the records it keeps the questions the engine answered without
//! asking (`super::unasked`), each placed before the record it preceded.
//!
//! **Not a buffer on the state**, for the trace sink's reason (`state::trace`):
//! `GameState` derives `Clone` and a search forks it at every decision, so what
//! rides the state is a [`RecorderHandle`], a pointer to the shared store and a
//! branch number. Its `Clone` is the fork: the clone's stream is its parent's up
//! to the fork, then its own, and neither branch sees the other's records.
//!
//! [`EventWindow`]: super::event::EventWindow

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use super::event::{EventRecord, GameEvent};
use super::unasked::{Unasked, UnaskedAnswer, UnaskedChoice};
use crate::state::game_state::GameState;
use crate::types::ids::PlayerId;
use crate::ui::choice_types::ChoiceKind;

/// Every branch's records, shared by every handle of one game.
///
/// A `Mutex` rather than a `RefCell` so `GameState` stays `Send`, as the trace
/// sink's is.
pub struct EventRecorder {
    branches: Mutex<Vec<Branch>>,
}

struct Branch {
    /// The branch this one was forked from, and how much of it it inherited.
    parent: Option<Inherited>,
    /// The records this branch's window flushed.
    own: Vec<EventRecord>,
    /// The questions this branch's game answered without asking.
    unasked: Vec<UnaskedChoice>,
}

/// A fork's parent, and how many of each of the parent's lists, its
/// ancestors' included, the fork inherited.
#[derive(Clone, Copy)]
struct Inherited {
    branch: usize,
    records: usize,
    unasked: usize,
}

impl EventRecorder {
    fn lock(&self) -> MutexGuard<'_, Vec<Branch>> {
        self.branches.lock().expect("event recorder poisoned")
    }
}

/// What a [`GameState`] holds: the shared recorder and the branch this state
/// records as.
pub struct RecorderHandle {
    recorder: Arc<EventRecorder>,
    branch: usize,
}

impl RecorderHandle {
    /// A new recorder with no records, and its root handle.
    pub fn new() -> RecorderHandle {
        let root = Branch { parent: None, own: Vec::new(), unasked: Vec::new() };
        RecorderHandle { recorder: Arc::new(EventRecorder { branches: Mutex::new(vec![root]) }), branch: 0 }
    }

    pub(crate) fn extend(&self, records: impl IntoIterator<Item = EventRecord>) {
        self.recorder.lock()[self.branch].own.extend(records);
    }

    pub(crate) fn note(&self, unasked: UnaskedChoice) {
        self.recorder.lock()[self.branch].unasked.push(unasked);
    }

    /// This branch's stream: its ancestors' records up to each fork, then its
    /// own, oldest first.
    pub fn records(&self) -> Vec<EventRecord> {
        self.inherited_and_own(|branch| &branch.own, |inherited| inherited.records)
    }

    /// The questions this branch's game answered without asking, inherited
    /// as the records are.
    pub fn unasked(&self) -> Vec<UnaskedChoice> {
        self.inherited_and_own(|branch| &branch.unasked, |inherited| inherited.unasked)
    }

    /// One of a branch's lists as its game sees it: each ancestor's up to the
    /// fork below it, then this branch's own.
    fn inherited_and_own<T: Clone>(&self, list: impl Fn(&Branch) -> &Vec<T>, count: impl Fn(&Inherited) -> usize) -> Vec<T> {
        let branches = self.recorder.lock();
        // From this branch up to the root, each with how much of its list
        // the branch below it inherited: a prefix of the whole, its own
        // ancestors' included.
        let mut chain = vec![(self.branch, usize::MAX)];
        let mut at = self.branch;
        while let Some(parent) = branches[at].parent {
            chain.push((parent.branch, count(&parent)));
            at = parent.branch;
        }
        let mut items = Vec::new();
        for &(branch, inherited) in chain.iter().rev() {
            items.extend(list(&branches[branch]).iter().cloned());
            items.truncate(inherited);
        }
        items
    }
}

/// The fork, which is why it is written out: a derived `Clone` would give the
/// copy the same branch, and two games would record into one stream. This one
/// opens a branch that inherits the parent's stream as it stands.
impl Clone for RecorderHandle {
    fn clone(&self) -> Self {
        let mut branches = self.recorder.lock();
        let parent = &branches[self.branch];
        let inherited = Inherited {
            branch: self.branch,
            records: parent.own.len() + parent.parent.map_or(0, |p| p.records),
            unasked: parent.unasked.len() + parent.parent.map_or(0, |p| p.unasked),
        };
        branches.push(Branch { parent: Some(inherited), own: Vec::new(), unasked: Vec::new() });
        RecorderHandle { recorder: Arc::clone(&self.recorder), branch: branches.len() - 1 }
    }
}

impl fmt::Debug for RecorderHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecorderHandle").field("branch", &self.branch).finish()
    }
}

/// One branch's stream, copied out when read: the read API the window does not
/// have. Nothing in the engine reads a recorder, since what the rules need from
/// the past is materialized on the state. This is for observers: a harness's
/// rows, a game log, a test's assertions.
#[derive(Debug, Clone)]
pub struct RecordedEvents {
    records: Vec<EventRecord>,
    unasked: Vec<UnaskedChoice>,
}

impl RecordedEvents {
    /// Every record, oldest first.
    pub fn records(&self) -> &[EventRecord] {
        &self.records
    }

    /// Every question the engine answered without asking, oldest first, each
    /// before the record its `before` names.
    pub fn unasked(&self) -> &[UnaskedChoice] {
        &self.unasked
    }

    /// Just the events, for readers that do not care which batch or resolution
    /// they came from — the display path, and most assertions.
    pub fn events(&self) -> impl Iterator<Item = &GameEvent> + '_ {
        self.records.iter().map(|r| &r.event)
    }

    /// The records after the first `index` — pass an earlier [`Self::len`] to
    /// ask "what happened since I last looked".
    pub fn records_from(&self, index: usize) -> &[EventRecord] {
        &self.records[index.min(self.records.len())..]
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl GameState {
    /// Record every event from here on, for [`Self::recorded_events`]. The
    /// test setup helpers and `fuzz_games` call this; a game nobody reads the
    /// history of keeps none.
    pub fn record_events(&mut self) {
        self.events.attach_recorder(RecorderHandle::new());
    }

    /// Everything performed since [`Self::record_events`], including the
    /// records the window has yet to flush.
    ///
    /// # Panics
    ///
    /// When no recorder is attached: a game that records nothing has no
    /// history to read, and an empty answer would pass for one.
    pub fn recorded_events(&self) -> RecordedEvents {
        let recorder = self
            .events
            .recorder()
            .expect("no event recorder is attached: call `record_events` when the game is built");
        let mut records = recorder.records();
        records.extend(self.events.held().iter().cloned());
        RecordedEvents { records, unasked: recorder.unasked() }
    }

    /// Record that `player` was not asked a question and why, building the
    /// question and its answer only when something records.
    pub(crate) fn note_unasked(&self, player: PlayerId, why: Unasked, question: impl FnOnce() -> (ChoiceKind, UnaskedAnswer)) {
        if let Some(recorder) = self.events.recorder() {
            let (kind, answer) = question();
            recorder.note(UnaskedChoice { before: self.events.next_seq(), player, kind, answer, why });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event::{EventId, EventStamp};

    fn record(seq: usize) -> EventRecord {
        EventRecord {
            seq: EventId(seq),
            event: GameEvent::StateBasedActionPerformed,
            stamp: EventStamp { batch: None, cause: crate::events::cause::EventCause::Harness },
            names: None,
            places: crate::events::event::RecordPlaces::NOT_COUNTED,
        }
    }

    fn seqs(records: &[EventRecord]) -> Vec<usize> {
        records.iter().map(|r| r.seq.0).collect()
    }

    /// A question not asked before record `before`.
    fn unasked(before: usize) -> UnaskedChoice {
        UnaskedChoice {
            before: EventId(before),
            player: 0,
            kind: ChoiceKind::PriorityAction,
            answer: UnaskedAnswer::Picks(Vec::new()),
            why: Unasked::OneLegalAnswer,
        }
    }

    /// The questions not asked follow a fork as its records do: the parent's
    /// up to the fork, then its own.
    #[test]
    fn a_fork_inherits_the_questions_not_asked_up_to_the_fork() {
        let root = RecorderHandle::new();
        root.note(unasked(0));
        let fork = root.clone();
        root.note(unasked(1));
        fork.note(unasked(7));

        let befores = |handle: &RecorderHandle| handle.unasked().iter().map(|q| q.before.0).collect::<Vec<_>>();
        assert_eq!(befores(&root), vec![0, 1]);
        assert_eq!(befores(&fork), vec![0, 7]);
    }

    /// A fork reads its parent's stream up to the fork and then its own, and
    /// the parent never sees the fork's records.
    #[test]
    fn a_fork_inherits_the_stream_up_to_the_fork() {
        let root = RecorderHandle::new();
        root.extend([record(0), record(1)]);
        let fork = root.clone();
        root.extend([record(2)]);
        fork.extend([record(7)]);

        assert_eq!(seqs(&root.records()), vec![0, 1, 2]);
        assert_eq!(seqs(&fork.records()), vec![0, 1, 7]);
    }

    /// A fork of a fork inherits through both, and a sibling taken later
    /// inherits what its parent had by then.
    #[test]
    fn forks_nest_and_each_inherits_its_own_prefix() {
        let root = RecorderHandle::new();
        root.extend([record(0)]);
        let a = root.clone();
        a.extend([record(1)]);
        let b = a.clone();
        a.extend([record(2)]);
        b.extend([record(5)]);
        root.extend([record(9)]);
        let c = root.clone();

        assert_eq!(seqs(&b.records()), vec![0, 1, 5]);
        assert_eq!(seqs(&a.records()), vec![0, 1, 2]);
        assert_eq!(seqs(&c.records()), vec![0, 9]);
    }
}
