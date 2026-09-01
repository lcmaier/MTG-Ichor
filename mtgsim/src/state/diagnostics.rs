//! Seed-deterministic counters for how much work the engine did.
//!
//! # Why these exist, and why they are not a profiler
//!
//! `engineering-practices.md` §3 refuses to store `ms/game`, and it is right to:
//! a stored timing number is machine drift, so the only honest timing
//! measurement is an **interleaved A/B in one sitting**. That leaves a gap the
//! fixtures table cannot fill — an A/B has to be *run*, by someone who already
//! suspects something, against two builds they had a reason to compare.
//!
//! **Both of RC-2's performance findings were found by accident**, and one of
//! them had been live since the phase before. `gather` swept every permanent
//! with a full layer walk per proposed action the moment any replacement source
//! was on the board, which RC-2 made the common case; it cost 10.3% of total
//! game time and nothing reported it. What *would* have reported it is a count
//! of layer walks per game, because that number roughly tripled while every
//! seed-deterministic row in the fixtures table stayed exactly the same.
//!
//! So these are the same kind of measurement as "creatures died: 8.6" — a pure
//! function of the seed and the card pool, comparable across machines and across
//! months, and a change to one means **the engine's cost model moved** the way a
//! change to that row means its behaviour moved. They belong in the fixtures
//! table for the same reason and are read the same way.
//!
//! # What is deliberately not here
//!
//! No timers, no allocation counts, no per-call-site breakdown. Each of those is
//! either machine-dependent (the first two) or a profiler's job (the third), and
//! a diagnostic that cannot go in the fixtures table is a diagnostic nobody will
//! look at twice. Four counters, each naming a decision the engine makes a lot.

use std::cell::Cell;

/// Counts of engine work performed, reset per game.
///
/// `Cell` rather than plain fields because the two hottest producers —
/// `compute_characteristics` and `is_prohibited` — take `&GameState`, and making
/// them take `&mut` to count would put a mutable borrow through the entire
/// oracle layer for a diagnostic. Not `Atomic`: a `GameState` belongs to one
/// game on one thread, and `fuzz_games` distributes whole games rather than
/// sharing one.
///
/// **Cloned with the `GameState`, deliberately.** A search that clones a state
/// to explore a branch inherits the count so far, so a branch's cost is a
/// subtraction rather than a separate accounting. Nothing does that yet; the
/// alternative — resetting on clone — would need `Clone` by hand and would be a
/// guess about a consumer that does not exist.
#[derive(Debug, Clone, Default)]
pub struct EngineCounters {
    layer_walks: Cell<u64>,
    layer_frames: Cell<u64>,
    replacement_gathers: Cell<u64>,
    restriction_queries: Cell<u64>,
}

impl EngineCounters {
    /// A top-level `compute_characteristics` call — one full CR 613 layer walk.
    ///
    /// **The headline number.** Almost every cost question in this engine
    /// reduces to how many of these a game does, because the walk is the
    /// expensive thing and everything else is bookkeeping around it.
    pub fn record_layer_walk(&self) {
        self.layer_walks.set(self.layer_walks.get() + 1);
    }

    /// One frame actually computed — a `compute_to_ceiling` body that ran rather
    /// than a cache hit.
    ///
    /// Read against [`Self::layer_walks`]: the ratio is how much CR 613.7a's
    /// existence re-check costs. A walk that needs no sub-frame is 1:1; one that
    /// asks whether five other objects still have their static abilities is not.
    /// That ratio is the thing `layers-architecture.md` §5.2's descending ceiling
    /// exists to bound, and until now nothing measured it.
    pub fn record_layer_frame(&self) {
        self.layer_frames.set(self.layer_frames.get() + 1);
    }

    /// One `engine::replacement::gather` — a proposed action that reached the
    /// CR 616.1 pipeline and asked what applies to it.
    pub fn record_replacement_gather(&self) {
        self.replacement_gathers.set(self.replacement_gathers.get() + 1);
    }

    /// One `engine::restriction::is_prohibited` — a CR 101.2 question.
    ///
    /// Worth its own counter rather than folding into the gather count: it is
    /// asked on **every iteration** of the CR 616.1 loop, so the two diverge
    /// exactly when a replacement effect rewrites an event more than once.
    pub fn record_restriction_query(&self) {
        self.restriction_queries.set(self.restriction_queries.get() + 1);
    }

    pub fn layer_walks(&self) -> u64 {
        self.layer_walks.get()
    }

    pub fn layer_frames(&self) -> u64 {
        self.layer_frames.get()
    }

    pub fn replacement_gathers(&self) -> u64 {
        self.replacement_gathers.get()
    }

    pub fn restriction_queries(&self) -> u64 {
        self.restriction_queries.get()
    }
}
