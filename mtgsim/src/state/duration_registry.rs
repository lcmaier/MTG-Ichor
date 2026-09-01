//! The storage and expiry half every effect registry shares (CR 514.2, 613.7).
//!
//! `ContinuousEffectRegistry` and `ReplacementEffectRegistry` each *own* one of
//! these and delegate to it; a third arrives with delayed triggers (CR 603.7).
//! Composition rather than inheritance-by-copy, and rather than free functions
//! over `&mut Vec<Row>`: each registry stays readable in one file, because this
//! type answers "how do rows live and die" and the wrapper answers "what is
//! this registry *for*".
//!
//! # Why this is abstracted at all, given "don't refactor speculatively"
//!
//! Not the line count — three copies of fifteen lines is cheap. It is that the
//! duplicated logic is **CR 514.2 and CR 613.7**, a rules concern, and its
//! failure mode is a duration bug fixed in one registry and not the others,
//! invisible because each has its own passing tests.
//! `cant-effects-architecture.md` §9 finding 7 has the full argument.
//!
//! # What stays in the wrapper
//!
//! Anything that is not "a row exists until something ends it": the CR 613.6
//! summary flags, `effects_in_layer`'s layer slicing, `Uses::Once` consumption.
//! This type deliberately knows nothing about layers or events.

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

    /// The object whose spell, ability or static ability created this row.
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

    /// The `(sort_key, id)` ordering invariant `add` maintains. Cheap enough to
    /// `debug_assert` on reads and after mutations, so a future code path that
    /// breaks the order fails a test rather than silently misordering a layer.
    pub fn is_sorted(&self) -> bool {
        self.rows
            .windows(2)
            .all(|w| (w[0].sort_key(), w[0].id()) < (w[1].sort_key(), w[1].id()))
    }
}

impl<T: DurationRow> Default for DurationRegistry<T> {
    fn default() -> Self {
        Self::new()
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
