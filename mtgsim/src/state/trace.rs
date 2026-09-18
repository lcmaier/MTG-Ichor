//! The trace sink — tier 2 of `engineering-practices.md` §7.1 (row A4c).
//!
//! # What it is
//!
//! An observer the engine writes to at its emit points — each batch entering
//! [`GameState::execute_actions`], each CR 616.1 iteration, each top-level
//! layer walk, each prompt at the decision boundary, and each performed event
//! — as one JSON object per line. Off by default: [`GameState::trace`] is one
//! branch on an `Option`, and every payload is built inside the closure it is
//! handed, so a game nobody traces pays the branch and nothing else.
//!
//! What comes out is a page's *spine* (§7's step rows, generated) and never
//! its argument: which reads were consulted, in what order, with what answer.
//! `plans/trace_spine.py` turns one game's lines into that page.
//!
//! # What it is not
//!
//! **Not a participant.** Nothing here reads [`GameState::rng`], asks a
//! `DecisionProvider`, or changes control flow at an emit point; the check is
//! that a binary with the sink compiled in and off is `IDENTICAL` to `main` on
//! every counter, and so is one with it on (`plans/fuzz_ab.py`). **Not a
//! buffer on the state.** `GameState` derives `Clone` and a search forks it at
//! every decision (`tests/priority_fork_test.rs`), so what rides the state is a
//! [`TraceHandle`] — a pointer to the shared sink plus a branch number — and
//! its hand-written `Clone` is the fork marker: a clone writes one `fork`
//! record and every record it writes afterwards carries the new branch.
//! **Not `serde`.** Main item 141 leaves the wire format to Phase 10, and a
//! sink that pulled the dependency in first would decide it by accident; the
//! handful of record kinds are written by [`Record`], a builder that knows
//! how to escape a string and separate fields; the records themselves are
//! `engine::trace_records`, one function per kind.
//!
//! # The two traps, and where they are avoided
//!
//! Nothing here iterates `game.battlefield` or any id-keyed map into the
//! output: [`GameState::trace_objects`] sorts the store by id before it
//! writes, and every emit point names ids the engine already ordered. The
//! end-to-end check is three traced runs of one seed under three
//! `MTGSIM_HASH_SEED`s, byte-identical.

use std::fmt;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::events::event::GameEvent;
use crate::state::game_state::GameState;
use crate::types::ids::ObjectId;

/// The build's commit, baked in by `stamp_commit.rs` so a trace says which engine
/// wrote it — the per-game header the two-version diff will key on.
pub const COMMIT: &str = match option_env!("MTGSIM_COMMIT") {
    Some(c) => c,
    None => "unknown",
};

// ---------------------------------------------------------------------------
// The sink
// ---------------------------------------------------------------------------

/// One output every handle writes to, shared by every branch of one game.
///
/// A `Mutex` rather than a `RefCell` so that `GameState` stays `Send`: a
/// harness builds each game on its own thread and never shares one, but a
/// state that could not be *moved* to a thread would close a door the
/// parallel-play use case walks through.
pub struct TraceSink {
    inner: Mutex<Inner>,
}

struct Inner {
    out: BufWriter<Box<dyn Write + Send>>,
    /// Total order of every record written, across branches.
    seq: u64,
    /// The next branch number a fork takes. Branch 0 is the root handle's.
    next_branch: u64,
}

impl TraceSink {
    /// A sink over any writer — a file, a shared buffer in a test.
    pub fn to_writer(out: impl Write + Send + 'static) -> Arc<TraceSink> {
        Arc::new(TraceSink {
            inner: Mutex::new(Inner {
                out: BufWriter::new(Box::new(out)),
                seq: 0,
                next_branch: 1,
            }),
        })
    }

    /// A sink writing JSON lines to `path`, created or truncated.
    pub fn to_file(path: impl AsRef<Path>) -> io::Result<Arc<TraceSink>> {
        Ok(Self::to_writer(File::create(path)?))
    }

    /// Flush what has been written so far. A `BufWriter` flushes on drop, but a
    /// handle may outlive the moment a reader wants the file whole.
    pub fn flush(&self) -> io::Result<()> {
        self.inner.lock().expect("trace sink poisoned").out.flush()
    }

    fn write(&self, branch: u64, record: Record) {
        let mut inner = self.inner.lock().expect("trace sink poisoned");
        inner.seq += 1;
        let seq = inner.seq;
        // A write failure is not the engine's to report: the sink is an
        // observer, and an observer that could fail a game would be a
        // participant. The record is dropped and the game goes on.
        let _ = writeln!(inner.out, "{{\"seq\":{},\"branch\":{},{}}}", seq, branch, record.body);
    }

    /// Allocate a branch for a forked handle and record the fork.
    fn fork(&self, from: u64) -> u64 {
        let to = {
            let mut inner = self.inner.lock().expect("trace sink poisoned");
            let to = inner.next_branch;
            inner.next_branch += 1;
            to
        };
        let mut r = Record::new("fork");
        r.field_u64("from", from);
        r.field_u64("to", to);
        self.write(from, r);
        to
    }
}

/// What a [`GameState`] holds: the shared sink and the branch this state
/// writes as.
///
/// **`Clone` is the fork marker.** `GameState` derives `Clone`, so cloning a
/// traced state clones this, and the clone takes a fresh branch number from
/// the sink and writes a `fork` record naming both. That is the whole of what
/// a forked search's trace needs to be read back as a tree.
pub struct TraceHandle {
    sink: Arc<TraceSink>,
    branch: u64,
}

impl TraceHandle {
    /// The root handle, branch 0.
    pub fn new(sink: &Arc<TraceSink>) -> TraceHandle {
        TraceHandle { sink: Arc::clone(sink), branch: 0 }
    }

    /// The sink this handle writes to.
    pub fn sink(&self) -> &Arc<TraceSink> {
        &self.sink
    }

    /// Which branch this handle writes as.
    pub fn branch(&self) -> u64 {
        self.branch
    }

    fn write(&self, record: Record) {
        self.sink.write(self.branch, record);
    }
}

impl Clone for TraceHandle {
    fn clone(&self) -> Self {
        TraceHandle { sink: Arc::clone(&self.sink), branch: self.sink.fork(self.branch) }
    }
}

impl fmt::Debug for TraceHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TraceHandle").field("branch", &self.branch).finish()
    }
}

// ---------------------------------------------------------------------------
// Records
// ---------------------------------------------------------------------------

/// One JSON object in progress — the fields of a record after its `seq`,
/// `branch` and `kind`, which every record carries.
///
/// A builder rather than a value tree because there is nothing to hold: a
/// record is written the moment it is complete, and the only work a value
/// tree would save is the comma bookkeeping this does with a stack of
/// "first element" flags. Nested objects and arrays open and close through
/// [`Self::begin_object`], [`Self::begin_array`] and [`Self::end`].
pub struct Record {
    body: String,
    /// One entry per open container: whether its next element is the first,
    /// and whether it closes with `]` or `}`.
    open: Vec<Container>,
    /// A key was just written, so the next value takes no separator.
    after_key: bool,
}

struct Container {
    first: bool,
    array: bool,
}

impl Record {
    /// A record of the given kind, with no other field yet.
    pub fn new(kind: &str) -> Record {
        let mut r = Record {
            body: String::new(),
            open: vec![Container { first: true, array: false }],
            after_key: false,
        };
        r.field_str("kind", kind);
        r
    }

    fn separate(&mut self) {
        if self.after_key {
            self.after_key = false;
            return;
        }
        if let Some(container) = self.open.last_mut() {
            if container.first {
                container.first = false;
            } else {
                self.body.push(',');
            }
        }
    }

    fn push_string(&mut self, s: &str) {
        self.body.push('"');
        for c in s.chars() {
            match c {
                '"' => self.body.push_str("\\\""),
                '\\' => self.body.push_str("\\\\"),
                '\n' => self.body.push_str("\\n"),
                '\r' => self.body.push_str("\\r"),
                '\t' => self.body.push_str("\\t"),
                c if (c as u32) < 0x20 => {
                    self.body.push_str(&format!("\\u{:04x}", c as u32));
                }
                c => self.body.push(c),
            }
        }
        self.body.push('"');
    }

    /// A key; the next value written is its value.
    pub fn key(&mut self, key: &str) -> &mut Self {
        self.separate();
        self.push_string(key);
        self.body.push(':');
        self.after_key = true;
        self
    }

    pub fn str(&mut self, value: &str) -> &mut Self {
        self.separate();
        self.push_string(value);
        self
    }

    pub fn u64(&mut self, value: u64) -> &mut Self {
        self.separate();
        self.body.push_str(&value.to_string());
        self
    }

    pub fn i64(&mut self, value: i64) -> &mut Self {
        self.separate();
        self.body.push_str(&value.to_string());
        self
    }

    pub fn bool(&mut self, value: bool) -> &mut Self {
        self.separate();
        self.body.push_str(if value { "true" } else { "false" });
        self
    }

    pub fn null(&mut self) -> &mut Self {
        self.separate();
        self.body.push_str("null");
        self
    }

    pub fn begin_object(&mut self) -> &mut Self {
        self.separate();
        self.body.push('{');
        self.open.push(Container { first: true, array: false });
        self
    }

    pub fn begin_array(&mut self) -> &mut Self {
        self.separate();
        self.body.push('[');
        self.open.push(Container { first: true, array: true });
        self
    }

    /// Close the innermost open object or array.
    pub fn end(&mut self) -> &mut Self {
        // The record's own object is closed by the writer, never here.
        debug_assert!(self.open.len() > 1, "end() with no open container");
        if let Some(container) = self.open.pop() {
            self.body.push(if container.array { ']' } else { '}' });
        }
        self
    }

    pub fn field_str(&mut self, key: &str, value: &str) -> &mut Self {
        self.key(key).str(value)
    }

    pub fn field_u64(&mut self, key: &str, value: u64) -> &mut Self {
        self.key(key).u64(value)
    }

    pub fn field_i64(&mut self, key: &str, value: i64) -> &mut Self {
        self.key(key).i64(value)
    }

    pub fn field_bool(&mut self, key: &str, value: bool) -> &mut Self {
        self.key(key).bool(value)
    }

    pub fn field_opt_u64(&mut self, key: &str, value: Option<u64>) -> &mut Self {
        self.key(key);
        match value {
            Some(v) => self.u64(v),
            None => self.null(),
        }
    }

    pub fn field_opt_i64(&mut self, key: &str, value: Option<i64>) -> &mut Self {
        self.key(key);
        match value {
            Some(v) => self.i64(v),
            None => self.null(),
        }
    }

    pub fn field_opt_str(&mut self, key: &str, value: Option<&str>) -> &mut Self {
        self.key(key);
        match value {
            Some(v) => self.str(v),
            None => self.null(),
        }
    }

    pub fn field_strs<S: AsRef<str>>(&mut self, key: &str, values: &[S]) -> &mut Self {
        self.key(key).begin_array();
        for v in values {
            self.str(v.as_ref());
        }
        self.end()
    }

    pub fn field_u64s(&mut self, key: &str, values: &[u64]) -> &mut Self {
        self.key(key).begin_array();
        for v in values {
            self.u64(*v);
        }
        self.end()
    }

    pub fn field_usizes(&mut self, key: &str, values: &[usize]) -> &mut Self {
        self.key(key).begin_array();
        for v in values {
            self.u64(*v as u64);
        }
        self.end()
    }
}

/// `{:?}` of an engine value with every `ObjectId(N)` read as `#N` — the
/// spelling `ObjectId`'s `Display` uses, so an id looks the same in a record,
/// a log line and a page.
pub fn render_debug(value: &impl fmt::Debug) -> String {
    tidy_ids(&format!("{:?}", value))
}

/// `ObjectId(17)` → `#17`, everywhere in `s`. Whether `#17` is the right
/// spelling at all is `backlog.md` §2.36's question; this follows the log's
/// convention until it is answered.
pub fn tidy_ids(s: &str) -> String {
    const TAG: &str = "ObjectId(";
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(at) = rest.find(TAG) {
        out.push_str(&rest[..at]);
        let after = &rest[at + TAG.len()..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && after.as_bytes().get(digits) == Some(&b')') {
            out.push('#');
            out.push_str(&after[..digits]);
            rest = &after[digits + 1..];
        } else {
            out.push_str(TAG);
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

// ---------------------------------------------------------------------------
// The state's side
// ---------------------------------------------------------------------------

impl GameState {
    /// Attach a handle; every emit point writes to it from here on.
    pub fn install_trace(&mut self, handle: TraceHandle) {
        self.trace = Some(handle);
    }

    /// Detach the handle, returning it.
    pub fn take_trace(&mut self) -> Option<TraceHandle> {
        self.trace.take()
    }

    /// The handle, if one is installed.
    pub fn trace_handle(&self) -> Option<&TraceHandle> {
        self.trace.as_ref()
    }

    /// Is a sink attached? The one branch an emit point pays when it is not.
    #[inline]
    pub fn trace_on(&self) -> bool {
        self.trace.is_some()
    }

    /// Write a record, building it only if a sink is attached.
    ///
    /// **This is the whole cost model.** `build` runs behind the `Option`
    /// check, so an emit point pays for its payload — rendering an action,
    /// sorting a type set — only in a traced game.
    #[inline]
    pub fn trace(&self, build: impl FnOnce() -> Record) {
        if let Some(handle) = &self.trace {
            handle.write(build());
        }
    }

    /// The per-game header: who wrote this trace and from what.
    ///
    /// `seed` is the harness's game seed where there is one; a scripted test
    /// has none and says so.
    pub fn trace_game(&self, label: &str, seed: Option<u64>) {
        self.trace(|| {
            let mut r = Record::new("game");
            r.field_str("label", label);
            r.field_opt_u64("seed", seed);
            r.field_u64("players", self.players.len() as u64);
            r.field_str("commit", COMMIT);
            r
        });
    }

    /// The names table, one record per object in the store, by id.
    ///
    /// Written once, at the end, so every record before it can name an object
    /// by id alone. Sorted by id: the store is a hash map and its order is
    /// the process's.
    pub fn trace_objects(&self) {
        if !self.trace_on() {
            return;
        }
        let mut ids: Vec<ObjectId> = self.objects.keys().copied().collect();
        ids.sort_by_key(|id| id.raw());
        for id in ids {
            self.trace(|| {
                let object = &self.objects[&id];
                let mut r = Record::new("object");
                r.field_u64("id", id.raw());
                r.field_str("name", &crate::ui::display::card_name(self, id));
                r.field_u64("owner", object.owner as u64);
                r.field_bool("token", object.is_token);
                r
            });
        }
    }

    /// Record and log a performed event — the one door every emitter uses.
    ///
    /// The text is `ui::display::format_event`'s, the same line
    /// `--dump-events` writes, so the dump is a projection of the trace and
    /// not a second rendering that could drift from it.
    pub(crate) fn emit_event(&mut self, event: GameEvent) {
        self.trace(|| {
            let stamp = self.events.current_stamp();
            let mut r = Record::new("event");
            r.field_u64("index", self.events.len() as u64);
            r.field_opt_u64("batch", stamp.batch.map(|b| b.0));
            r.field_opt_u64("resolution", stamp.resolution.map(|s| s.source.raw()));
            r.field_str("text", &crate::ui::display::format_event(self, &event));
            r
        });
        self.events.emit(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(r: Record) -> String {
        r.body
    }

    #[test]
    fn a_record_separates_fields_and_nests_containers() {
        let mut r = Record::new("t");
        r.field_u64("n", 3).field_str("s", "a\"b\\c\n");
        r.key("list").begin_array().u64(1).str("x").end();
        r.key("obj").begin_object().field_bool("b", true).key("inner").begin_array().end().end();
        r.field_opt_u64("none", None);
        assert_eq!(
            body(r),
            r#""kind":"t","n":3,"s":"a\"b\\c\n","list":[1,"x"],"obj":{"b":true,"inner":[]},"none":null"#
        );
    }

    #[test]
    fn tidy_ids_rewrites_only_well_formed_ids() {
        assert_eq!(tidy_ids("Object(ObjectId(17)) ObjectId(x) ObjectId(3"), "Object(#17) ObjectId(x) ObjectId(3");
    }
}
