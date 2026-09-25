//! The event recorder: the whole performed stream, for whatever reads it
//! (`codebase-state.md` item 42).
//!
//! The engine keeps a record only until its last reader has run
//! ([`EventWindow`]), so a game's history is not game state. What wants all of
//! it — `fuzz_games`' fixture rows, `--dump-events`, the fork test, a GUI's
//! game log, a test asserting on what happened — attaches a recorder, and the
//! window hands it each record as it flushes, in stream order.
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
use crate::state::game_state::GameState;

/// Every branch's records, shared by every handle of one game.
///
/// A `Mutex` rather than a `RefCell` so `GameState` stays `Send`, as the trace
/// sink's is.
pub struct EventRecorder {
    branches: Mutex<Vec<Branch>>,
}

struct Branch {
    /// The branch this one was forked from, and how many of that branch's
    /// records it inherited.
    parent: Option<(usize, usize)>,
    /// The records this branch's window flushed.
    own: Vec<EventRecord>,
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
        let root = Branch { parent: None, own: Vec::new() };
        RecorderHandle { recorder: Arc::new(EventRecorder { branches: Mutex::new(vec![root]) }), branch: 0 }
    }

    pub(crate) fn extend(&self, records: impl IntoIterator<Item = EventRecord>) {
        self.recorder.lock()[self.branch].own.extend(records);
    }

    /// This branch's stream: its ancestors' records up to each fork, then its
    /// own, oldest first.
    pub fn records(&self) -> Vec<EventRecord> {
        let branches = self.recorder.lock();
        // From this branch up to the root, each with how much of its stream
        // the branch below it inherited: a prefix of the whole, its own
        // ancestors' records included.
        let mut chain = vec![(self.branch, usize::MAX)];
        let mut at = self.branch;
        while let Some((parent, inherited)) = branches[at].parent {
            chain.push((parent, inherited));
            at = parent;
        }
        let mut records = Vec::new();
        for &(branch, inherited) in chain.iter().rev() {
            records.extend(branches[branch].own.iter().cloned());
            records.truncate(inherited);
        }
        records
    }
}

/// The fork, which is why it is written out: a derived `Clone` would give the
/// copy the same branch, and two games would record into one stream. This one
/// opens a branch that inherits the parent's stream as it stands.
impl Clone for RecorderHandle {
    fn clone(&self) -> Self {
        let mut branches = self.recorder.lock();
        let inherited = stream_len(&branches, self.branch);
        branches.push(Branch { parent: Some((self.branch, inherited)), own: Vec::new() });
        RecorderHandle { recorder: Arc::clone(&self.recorder), branch: branches.len() - 1 }
    }
}

/// How many records `branch`'s stream holds.
fn stream_len(branches: &[Branch], branch: usize) -> usize {
    let own = branches[branch].own.len();
    own + branches[branch].parent.map_or(0, |(_, inherited)| inherited)
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
}

impl RecordedEvents {
    /// Every record, oldest first.
    pub fn records(&self) -> &[EventRecord] {
        &self.records
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
        RecordedEvents { records }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event::{EventSeq, EventStamp};

    fn record(seq: usize) -> EventRecord {
        EventRecord { seq: EventSeq(seq), event: GameEvent::StateBasedActionPerformed, stamp: EventStamp::default() }
    }

    fn seqs(records: &[EventRecord]) -> Vec<usize> {
        records.iter().map(|r| r.seq.0).collect()
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
