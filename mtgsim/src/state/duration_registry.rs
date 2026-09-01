//! The storage and expiry half every effect registry shares (CR 514.2).
//!
//! `ReplacementEffectRegistry` *is* one of these — a type alias, because it
//! needs nothing else. `ContinuousEffectRegistry` owns one and delegates,
//! because it does. A third customer arrives with delayed triggers (CR 603.7)
//! and a fourth with RS-1's restrictions.
//!
//! Composition rather than free functions over `&mut Vec<Row>`: this type
//! answers "how do rows live and die", and a wrapper — where there is one —
//! answers "what is this registry *for*", so each stays readable in one file.
//!
//! # Why this is abstracted at all, given "don't refactor speculatively"
//!
//! Not the line count, and **not CR 613.7** — an earlier draft of this comment
//! cited it and was wrong. CR 613.7 is timestamp ordering, which is the one
//! thing the two registries do *not* share; `SortKey` exists to let them differ
//! on it. What is genuinely shared is **CR 514.2**, reaching both registries
//! through CR 611.2a's "lasts as long as stated by the spell or ability".
//!
//! The load-bearing half is forward-looking and greppable: the engine now
//! decides what a `Duration` *means* in exactly two places, both below
//! (`grep -rn 'matches!(.*Duration::' src/`). `Duration` is a closed enum
//! scheduled to grow — `codebase-state.md` item 14 needs a resolution-scoped
//! variant before "can't be regenerated" is right, and both expiry methods
//! already carry a note about `UntilEndOfYourNextTurn`. A registry that missed
//! a new arm would be wrong in a way its own passing tests could not show.
//!
//! The cost is real: a delegating method says less at the call site than the
//! loop it replaced. It is paid only where the wrapper has its own surface to
//! justify it, which is why `ReplacementEffectRegistry` is a type alias for
//! this type and `ContinuousEffectRegistry` is not.
//!
//! # What stays in a wrapper
//!
//! Anything that is not "a row exists until something ends it" — which today is
//! `ContinuousEffectRegistry`'s CR 613.6 summary flags and `effects_in_layer`'s
//! layer slicing, and nothing else. This type deliberately knows nothing about
//! layers or events.

use crate::types::effects::Duration;
use crate::types::ids::{ObjectId, PlayerId};

/// The id space every duration registry allocates from.
///
/// One type rather than one per registry because the allocation discipline is
/// the shared thing: monotonic, never reused. Wrappers keep their own aliases
/// (`EffectId`, `ReplacementEffectId`) so their signatures still say which
/// registry an id came from.
pub type RowId = u64;

/// What [`DurationRegistry`] needs to know about a row to store it and to end
/// it at the right moment.
///
/// Six accessors, and every one is load-bearing rather than convenience:
/// `id`/`set_id` are the allocation discipline, `source` is `remove_by_source`,
/// `duration` + `controller` + `created_on_turn` are exactly the three fields
/// CR 514.2's and the player-relative expiry rules read.
pub trait DurationRow {
    /// The order rows are *stored* in, ahead of the id tiebreak.
    ///
    /// A registry whose reads need no order beyond registration order uses
    /// `()`: every key ties, the id breaks the tie, and `add` degenerates to a
    /// push. A registry that binary-searches its rows (CR 613.7's
    /// `(layer, timestamp)`) names the key here, and `add` keeps the invariant.
    type SortKey: Ord;

    /// This row's unique id. Never reused — it is part of the CR 614.5
    /// applied-set key in one wrapper and the CR 613.7a sub-order in the other.
    fn id(&self) -> RowId;

    /// Stamp the id `add` allocated. Called once, before the row is stored.
    fn set_id(&mut self, id: RowId);

    /// The object this row came from: the source of the spell or ability whose
    /// resolution created it (CR 611.2a), or the permanent whose static ability
    /// generates it (CR 611.3).
    fn source(&self) -> ObjectId;

    /// When the row stops existing.
    fn duration(&self) -> Duration;

    /// The player a player-relative duration is relative to (CR 611.2c's
    /// locked-at-resolution value). Resolves "your" in "until your next turn".
    fn controller(&self) -> PlayerId;

    /// The turn the row was created on, so a player-relative duration does not
    /// expire on the turn that made it.
    fn created_on_turn(&self) -> u32;

    /// See [`DurationRow::SortKey`].
    fn sort_key(&self) -> Self::SortKey;
}

/// A `Vec` of rows kept in `(sort_key, id)` order, plus the id counter and the
/// CR 514.2 expiry hooks.
#[derive(Debug, Clone)]
pub struct DurationRegistry<T: DurationRow> {
    rows: Vec<T>,
    next_id: RowId,
}

impl<T: DurationRow> DurationRegistry<T> {
    pub fn new() -> Self {
        DurationRegistry { rows: Vec::new(), next_id: 1 }
    }

    /// Store a row, stamping it with a fresh id. Returns that id.
    ///
    /// The id is allocated *before* the insertion point is chosen, because the
    /// id is the second half of the sort key: ids ascend and are never reused,
    /// so a row always sorts after every row already sharing its `SortKey`.
    /// That is what CR 613.7a's "the relative order of those timestamps remains
    /// the same" asks for when a whole object's static abilities share one
    /// timestamp — and, for a `SortKey` of `()`, it is what makes this a push
    /// and preserves the registration order a CR 616.1 prompt is offered in.
    pub fn add(&mut self, mut row: T) -> RowId {
        let id = self.next_id;
        self.next_id += 1;
        row.set_id(id);

        let key = (row.sort_key(), id);
        let pos = self.rows.partition_point(|r| (r.sort_key(), r.id()) < key);
        self.rows.insert(pos, row);
        id
    }

    /// Remove one row by id. Returns it if it was there.
    pub fn remove(&mut self, id: RowId) -> Option<T> {
        let pos = self.rows.iter().position(|r| r.id() == id)?;
        Some(self.rows.remove(pos))
    }

    /// Remove every row created by a given source object.
    pub fn remove_by_source(&mut self, source: ObjectId) -> Vec<T> {
        self.retain(|r| r.source() != source)
    }

    /// Remove every row failing `keep`, returning them; order preserved.
    ///
    /// One pass. The obvious `while i < len { if .. { v.remove(i) } }` is
    /// `O(n²)`, since each `Vec::remove` shifts the tail — and `swap_remove` is
    /// not an option, because it destroys the ordering invariant `add`
    /// maintains and `effects_in_layer` binary-searches.
    pub fn retain(&mut self, keep: impl Fn(&T) -> bool) -> Vec<T> {
        let mut removed = Vec::new();
        let mut kept = Vec::with_capacity(self.rows.len());
        for row in self.rows.drain(..) {
            if keep(&row) {
                kept.push(row);
            } else {
                removed.push(row);
            }
        }
        self.rows = kept;
        removed
    }

    /// Remove rows that expire during the cleanup step (CR 514.2).
    ///
    /// Handles:
    /// - `UntilEndOfTurn` — always expires at cleanup.
    /// - `UntilEndOfYourNextTurn` (future) — expires at cleanup if the active
    ///   player matches the row's controller AND the current turn is after the
    ///   turn the row was created on.
    pub fn remove_expired_at_cleanup(
        &mut self,
        active_player: PlayerId,
        current_turn: u32,
    ) -> Vec<T> {
        // Suppress unused variable warnings until multi-turn durations are added
        // (UntilEndOfYourNextTurn would read both).
        let _ = (active_player, current_turn);
        self.retain(|r| !matches!(r.duration(), Duration::UntilEndOfTurn))
    }

    /// Remove rows that expire at the start of a player's turn.
    ///
    /// Handles:
    /// - `UntilYourNextTurn` — expires at the beginning of the controller's
    ///   next turn (checked at untap step). Only fires if the current turn is
    ///   strictly after the turn the row was created on (prevents immediate
    ///   expiry when created on your own turn).
    pub fn remove_expired_at_turn_start(
        &mut self,
        active_player: PlayerId,
        current_turn: u32,
    ) -> Vec<T> {
        self.retain(|r| {
            !matches!(r.duration(), Duration::UntilYourNextTurn)
                || r.controller() != active_player
                || current_turn <= r.created_on_turn()
        })
    }

    /// Every row, in stored order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.rows.iter()
    }

    /// Every row as a slice, for a wrapper that binary-searches the order
    /// `add` maintains. Read-only: no wrapper can reach the `Vec` itself, so
    /// the ordering and id invariants stay this type's to keep.
    pub fn as_slice(&self) -> &[T] {
        &self.rows
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether the rows are still in the `(sort_key, id)` order [`Self::add`]
    /// places them in.
    ///
    /// `windows(2)` yields every adjacent pair, and a sequence is ordered
    /// exactly when each neighbouring pair is — so this is "no row outranks the
    /// one after it". The comparison is strict `<` rather than `<=`, which also
    /// asserts no two rows tie: they cannot, because the id is the last
    /// component and ids are unique.
    ///
    /// Cheap enough to `debug_assert` on reads and after mutations, and worth
    /// asserting because `effects_in_layer` *binary-searches* this order —
    /// against unsorted rows a binary search returns a confidently wrong answer
    /// rather than an error, so a future code path that bypasses `add` should
    /// fail a test rather than silently misorder a layer.
    pub fn is_sorted(&self) -> bool {
        self.rows
            .windows(2)
            .all(|w| (w[0].sort_key(), w[0].id()) < (w[1].sort_key(), w[1].id()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    /// The minimum a row can be: registration order, no sort key. Stands in for
    /// `RegisteredReplacementEffect` without dragging a `ReplacementDef` in.
    #[derive(Debug, Clone)]
    struct Row {
        id: RowId,
        source: ObjectId,
        duration: Duration,
        controller: PlayerId,
        created_on_turn: u32,
        /// Only the keyed tests read this; `sort_key` is `()` regardless, so a
        /// registry of `Row` is a plain append-only log.
        rank: u8,
    }

    impl Row {
        fn new(source: ObjectId, duration: Duration) -> Self {
            Row { id: 0, source, duration, controller: 0, created_on_turn: 1, rank: 0 }
        }
    }

    impl DurationRow for Row {
        type SortKey = ();
        fn id(&self) -> RowId {
            self.id
        }
        fn set_id(&mut self, id: RowId) {
            self.id = id;
        }
        fn source(&self) -> ObjectId {
            self.source
        }
        fn duration(&self) -> Duration {
            self.duration
        }
        fn controller(&self) -> PlayerId {
            self.controller
        }
        fn created_on_turn(&self) -> u32 {
            self.created_on_turn
        }
        fn sort_key(&self) -> Self::SortKey {}
    }

    /// The same row with a real key, standing in for `ContinuousEffect`'s
    /// `(layer, timestamp)`.
    #[derive(Debug, Clone)]
    struct KeyedRow(Row);

    impl DurationRow for KeyedRow {
        type SortKey = u8;
        fn id(&self) -> RowId {
            self.0.id
        }
        fn set_id(&mut self, id: RowId) {
            self.0.id = id;
        }
        fn source(&self) -> ObjectId {
            self.0.source
        }
        fn duration(&self) -> Duration {
            self.0.duration
        }
        fn controller(&self) -> PlayerId {
            self.0.controller
        }
        fn created_on_turn(&self) -> u32 {
            self.0.created_on_turn
        }
        fn sort_key(&self) -> u8 {
            self.0.rank
        }
    }

    #[test]
    fn ids_ascend_and_are_never_reused() {
        // The id is part of the CR 614.5 applied-set key in one wrapper and the
        // CR 613.7a sub-order in the other; a reused id corrupts both.
        let mut reg: DurationRegistry<Row> = DurationRegistry::new();
        let src = Uuid::new_v4();
        let a = reg.add(Row::new(src, Duration::UntilEndOfTurn));
        reg.remove(a);
        let b = reg.add(Row::new(src, Duration::UntilEndOfTurn));
        assert_eq!(a, 1);
        assert_ne!(a, b);
    }

    #[test]
    fn an_unkeyed_registry_is_an_append_only_log() {
        // `SortKey = ()` means every key ties and the ascending id decides, so
        // `add` lands at the end. That is what preserves the order a CR 616.1
        // prompt offers candidates in.
        let mut reg: DurationRegistry<Row> = DurationRegistry::new();
        let sources: Vec<ObjectId> = (0..4).map(|_| Uuid::new_v4()).collect();
        for s in &sources {
            reg.add(Row::new(*s, Duration::UntilEndOfTurn));
        }
        reg.remove_by_source(sources[1]);
        let order: Vec<ObjectId> = reg.iter().map(|r| r.source).collect();
        assert_eq!(order, vec![sources[0], sources[2], sources[3]]);
        assert!(reg.is_sorted());
    }

    #[test]
    fn a_keyed_registry_stays_sorted_through_out_of_order_adds() {
        let mut reg: DurationRegistry<KeyedRow> = DurationRegistry::new();
        let src = Uuid::new_v4();
        for rank in [5, 2, 9, 2] {
            let mut row = Row::new(src, Duration::UntilEndOfTurn);
            row.rank = rank;
            reg.add(KeyedRow(row));
        }
        let keys: Vec<(u8, RowId)> = reg.iter().map(|r| (r.sort_key(), r.id())).collect();
        // Equal keys keep insertion order via the id tiebreak: the rank-2 row
        // added first (id 2) still precedes the one added last (id 4).
        assert_eq!(keys, vec![(2, 2), (2, 4), (5, 1), (9, 3)]);
        assert!(reg.is_sorted());
    }

    #[test]
    fn as_slice_admits_the_binary_search_effects_in_layer_needs() {
        let mut reg: DurationRegistry<KeyedRow> = DurationRegistry::new();
        let src = Uuid::new_v4();
        for rank in [1, 2, 2, 3] {
            let mut row = Row::new(src, Duration::UntilEndOfTurn);
            row.rank = rank;
            reg.add(KeyedRow(row));
        }
        let all = reg.as_slice();
        let lo = all.partition_point(|r| r.sort_key() < 2);
        let hi = all.partition_point(|r| r.sort_key() <= 2);
        assert_eq!(&all[lo..hi].len(), &2);
    }

    #[test]
    fn removal_preserves_order() {
        let mut reg: DurationRegistry<KeyedRow> = DurationRegistry::new();
        let src = Uuid::new_v4();
        for rank in 0..6u8 {
            let mut row = Row::new(src, Duration::UntilEndOfTurn);
            row.rank = rank;
            reg.add(KeyedRow(row));
        }
        reg.retain(|r| r.sort_key() % 2 == 0);
        assert_eq!(reg.len(), 3);
        assert!(reg.is_sorted());
    }

    #[test]
    fn until_end_of_turn_expires_at_cleanup_and_nothing_else_does() {
        let mut reg: DurationRegistry<Row> = DurationRegistry::new();
        let src = Uuid::new_v4();
        reg.add(Row::new(src, Duration::UntilEndOfTurn));
        reg.add(Row::new(src, Duration::WhileSourceOnBattlefield));
        reg.add(Row::new(src, Duration::Indefinite));

        assert_eq!(reg.remove_expired_at_cleanup(0, 1).len(), 1);
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn until_your_next_turn_needs_the_right_player_and_a_later_turn() {
        let mut reg: DurationRegistry<Row> = DurationRegistry::new();
        let src = Uuid::new_v4();
        reg.add(Row::new(src, Duration::UntilYourNextTurn));

        assert_eq!(reg.remove_expired_at_turn_start(0, 1).len(), 0, "same turn");
        assert_eq!(reg.remove_expired_at_turn_start(1, 2).len(), 0, "wrong player");
        assert_eq!(reg.remove_expired_at_turn_start(0, 3).len(), 1);
        assert!(reg.is_empty());
    }

    #[test]
    fn until_your_next_turn_expires_on_an_extra_turn() {
        // Extra turns advance turn_number, so an extra turn for the controller
        // IS their "next turn" and the row correctly expires at its start.
        let mut reg: DurationRegistry<Row> = DurationRegistry::new();
        let src = Uuid::new_v4();
        reg.add(Row::new(src, Duration::UntilYourNextTurn));

        assert_eq!(reg.remove_expired_at_turn_start(0, 2).len(), 1);
    }
}
