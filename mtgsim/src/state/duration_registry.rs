//! The storage and expiry half every effect registry shares (CR 514.2).
//!
//! `ReplacementEffectRegistry` and the restrictions registry *are* ones of
//! these — type aliases, because they need nothing else.
//! `ContinuousEffectRegistry` owns one and delegates, because it does. A
//! further customer arrives with delayed triggers (CR 603.7).
//!
//! Composition rather than free functions over `&mut Vec<Row>`: this type
//! answers "how do rows live and die", and a wrapper — where there is one —
//! answers "what is this registry *for*", so each stays readable in one file.
//!
//! # Why this is abstracted at all, given "don't refactor speculatively"
//!
//! Not the line count, and **not CR 613.7**, which is timestamp ordering — the
//! one thing the registries do *not* share; `SortKey` exists to let them differ
//! on it. What is genuinely shared is **CR 514.2**, reaching every registry
//! through CR 611.2a's "lasts as long as stated by the spell or ability".
//!
//! The load-bearing half is forward-looking and greppable: the engine decides
//! what a `Duration` *means* in exactly two places, both below
//! (`grep -rn 'matches!(.*Duration::' src/`). `Duration` is a closed enum that
//! grows with cards (both expiry methods already carry a note about
//! `UntilEndOfYourNextTurn`), and a registry that missed a new arm would be
//! wrong in a way its own passing tests could not show.
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

use std::sync::Arc;

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

    /// CR 400.7 — does this row refer to `object` by identity, so that its
    /// move must end the reference? An affected set captured as the effect
    /// began (CR 611.2c), or the source CR 609.7a had a player choose.
    fn refers_to(&self, object: ObjectId) -> bool;

    /// CR 400.7 — stop referring to `object`, which has become a new object.
    /// Returns whether the row is still about anything, object or player; one
    /// that is not is dropped, since nothing it was about exists.
    fn remove_reference_to(&mut self, object: ObjectId) -> bool;
}

/// A `Vec` of rows kept in `(sort_key, id)` order, plus the id counter and the
/// CR 514.2 expiry hooks.
#[derive(Debug, Clone)]
pub struct DurationRegistry<T: DurationRow> {
    /// Each row behind its own `Arc`, so a fork shares them: cloning the
    /// registry is one allocation however deep the rows' payloads are, and a
    /// write copies only the row it edits (`Arc::make_mut`).
    rows: Vec<Arc<T>>,
    next_id: RowId,
    /// Bumped by every mutator that changed a row — added one, removed one,
    /// or edited one in place. What a wrapper reads to tell a write from a
    /// no-op: comparing `len` before and after cannot see an in-place edit,
    /// and could not see a closure that added and removed in one call.
    generation: u64,
}

impl<T: DurationRow> DurationRegistry<T> {
    pub fn new() -> Self {
        DurationRegistry { rows: Vec::new(), next_id: 1, generation: 0 }
    }

    /// How many times the rows have changed. Only ever grows.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// The id the next [`Self::add`] will stamp, read without advancing it.
    pub fn next_id(&self) -> RowId {
        self.next_id
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
        self.rows.insert(pos, Arc::new(row));
        self.generation += 1;
        id
    }

    /// Remove one row by id. Returns it if it was there — shared, since no
    /// caller needs to own it and taking it out of the `Arc` would copy a row
    /// a fork still holds.
    pub fn remove(&mut self, id: RowId) -> Option<Arc<T>> {
        let pos = self.rows.iter().position(|r| r.id() == id)?;
        self.generation += 1;
        Some(self.rows.remove(pos))
    }

    /// Edit rows in place: `edit` runs on every row `picks` accepts, and on no
    /// other. Returns how many it edited; when that is nonzero the rows are
    /// re-sorted — a *stable* sort on `(sort_key, id)`, so rows that end up
    /// sharing a key keep their id order, which is the registration order —
    /// and the generation is bumped.
    ///
    /// The pick is read-only because the edit is a copy: a row a fork shares
    /// is copied before it is written, so `picks` must accept only rows the
    /// edit will change, or a no-op write copies a row for nothing.
    ///
    /// The one in-place mutator. Exists for CR 613.7a's third sentence: when
    /// an object receives a new timestamp, "each continuous effect generated
    /// by static abilities of that object receives a new timestamp as well,
    /// but the relative order of those timestamps remains the same". Ids
    /// survive, unlike a remove-and-re-add, so nothing holding one dangles.
    pub fn update_rows(&mut self, picks: impl Fn(&T) -> bool, mut edit: impl FnMut(&mut T)) -> usize
    where
        T: Clone,
    {
        let mut changed = 0;
        for row in &mut self.rows {
            if picks(row) {
                edit(Arc::make_mut(row));
                changed += 1;
            }
        }
        if changed > 0 {
            self.rows.sort_by_key(|r| (r.sort_key(), r.id()));
            self.generation += 1;
        }
        changed
    }

    /// Remove every row created by a given source object.
    pub fn remove_by_source(&mut self, source: ObjectId) -> Vec<Arc<T>> {
        self.retain(|r| r.source() != source)
    }

    /// CR 400.7 — `object` has moved and "becomes a new object with no memory
    /// of, or relation to, its previous existence": every row that refers to
    /// it stops, and a row left about nothing is dropped. A row `spared`
    /// accepts is left as it is, for the rule's own exceptions. Returns
    /// whether a row changed.
    ///
    /// Beside [`Self::remove_by_source`], never instead of it: that one ends
    /// the rows an object's static abilities generate, and this one the rows
    /// that are *about* it (`copy-effects-architecture.md` §5.3).
    ///
    /// Scans first, so a move no row refers to, which is nearly every move,
    /// copies no row and leaves [`Self::generation`] alone. Removing a
    /// reference changes no row's key, so the order needs no re-sort.
    pub fn remove_references_to(&mut self, object: ObjectId, spared: impl Fn(&T) -> bool) -> bool
    where
        T: Clone,
    {
        let refers = |row: &T| row.refers_to(object) && !spared(row);
        if !self.rows.iter().any(|row| refers(row)) {
            return false;
        }
        self.rows.retain_mut(|row| !refers(row) || Arc::make_mut(row).remove_reference_to(object));
        self.generation += 1;
        true
    }

    /// Remove every row failing `keep`, returning them; order preserved.
    ///
    /// One pass. The obvious `while i < len { if .. { v.remove(i) } }` is
    /// `O(n²)`, since each `Vec::remove` shifts the tail — and `swap_remove` is
    /// not an option, because it destroys the ordering invariant `add`
    /// maintains and `effects_in_layer` binary-searches.
    pub fn retain(&mut self, keep: impl Fn(&T) -> bool) -> Vec<Arc<T>> {
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
        if !removed.is_empty() {
            self.generation += 1;
        }
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
    ) -> Vec<Arc<T>> {
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
    ) -> Vec<Arc<T>> {
        self.retain(|r| {
            !matches!(r.duration(), Duration::UntilYourNextTurn)
                || r.controller() != active_player
                || current_turn <= r.created_on_turn()
        })
    }

    /// Every row, in stored order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.rows.iter().map(|row| &**row)
    }

    /// Every row as a slice, for a wrapper that binary-searches the order
    /// `add` maintains. Read-only: no wrapper can reach the `Vec` itself, so
    /// the ordering and id invariants stay this type's to keep.
    pub fn as_slice(&self) -> &[Arc<T>] {
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
    /// exactly when each neighboring pair is — so this is "no row outranks the
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
    use crate::types::ids::new_object_id;

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
        /// The objects the row is about, for the CR 400.7 tests; `about_a_player`
        /// is the half that survives their going.
        named: Vec<ObjectId>,
        about_a_player: bool,
    }

    impl Row {
        fn new(source: ObjectId, duration: Duration) -> Self {
            Row {
                id: 0,
                source,
                duration,
                controller: 0,
                created_on_turn: 1,
                rank: 0,
                named: Vec::new(),
                about_a_player: false,
            }
        }

        fn about_objects(named: &[ObjectId]) -> Self {
            Row { named: named.to_vec(), ..Row::new(new_object_id(), Duration::Indefinite) }
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
        fn refers_to(&self, object: ObjectId) -> bool {
            self.named.contains(&object)
        }
        fn remove_reference_to(&mut self, object: ObjectId) -> bool {
            self.named.retain(|&id| id != object);
            !self.named.is_empty() || self.about_a_player
        }
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
        fn refers_to(&self, object: ObjectId) -> bool {
            self.0.refers_to(object)
        }
        fn remove_reference_to(&mut self, object: ObjectId) -> bool {
            self.0.remove_reference_to(object)
        }
    }

    /// CR 400.7: a move ends every row's reference to the mover. A row about
    /// two objects keeps applying to the one that stayed, a row about the
    /// mover alone goes, a row with a player half keeps that half, and a row
    /// the caller spares is untouched. Which rows a move spares is CR 400.7's
    /// list of exceptions, the caller's (`zones.rs`, `break_references_to`).
    #[test]
    fn a_move_prunes_each_row_that_refers_to_the_mover_and_drops_one_about_it_alone() {
        let mut reg: DurationRegistry<Row> = DurationRegistry::new();
        let (mover, stayer) = (new_object_id(), new_object_id());
        let both = reg.add(Row::about_objects(&[mover, stayer]));
        let alone = reg.add(Row::about_objects(&[mover]));
        let with_a_player = reg.add(Row { about_a_player: true, ..Row::about_objects(&[mover]) });
        let spared = reg.add(Row { rank: 1, ..Row::about_objects(&[mover]) });
        let g0 = reg.generation();

        assert!(reg.remove_references_to(mover, |row| row.rank == 1));
        assert_eq!(reg.generation(), g0 + 1);
        let ids: Vec<RowId> = reg.iter().map(|r| r.id).collect();
        assert_eq!(ids, vec![both, with_a_player, spared], "the row about the mover alone is dropped");
        assert!(!ids.contains(&alone));
        let named = |id: RowId| reg.iter().find(|r| r.id == id).map(|r| r.named.clone()).unwrap_or_default();
        assert_eq!(named(both), vec![stayer]);
        assert!(named(with_a_player).is_empty());
        assert_eq!(named(spared), vec![mover], "spared: CR 400.7a and 400.7c's exceptions");
    }

    /// The common move is one no row refers to: nothing is copied and the
    /// generation, which the layer memo reads, does not move.
    #[test]
    fn a_move_no_row_refers_to_changes_nothing() {
        let mut reg: DurationRegistry<Row> = DurationRegistry::new();
        reg.add(Row::about_objects(&[new_object_id()]));
        let fork = reg.clone();
        let g0 = reg.generation();
        assert!(!reg.remove_references_to(new_object_id(), |_| false));
        assert_eq!(reg.generation(), g0);
        assert!(Arc::ptr_eq(&reg.rows[0], &fork.rows[0]), "no row copied");
    }

    #[test]
    fn ids_ascend_and_are_never_reused() {
        // The id is part of the CR 614.5 applied-set key in one wrapper and the
        // CR 613.7a sub-order in the other; a reused id corrupts both.
        let mut reg: DurationRegistry<Row> = DurationRegistry::new();
        let src = new_object_id();
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
        let sources: Vec<ObjectId> = (0..4).map(|_| new_object_id()).collect();
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
        let src = new_object_id();
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

    /// `update_rows` is the one in-place mutator: rows whose key it changes
    /// move to where `add` would have put them, rows sharing the new key keep
    /// their id order, ids never change, and the generation moves once per
    /// call that changed something and not at all otherwise. The generation is
    /// what `ContinuousEffectRegistry::mutations` reads, so the last point is
    /// the layer memo's correctness.
    #[test]
    fn update_rows_resorts_keeps_ids_and_bumps_the_generation_once() {
        let mut reg: DurationRegistry<KeyedRow> = DurationRegistry::new();
        let src = new_object_id();
        let keyed = |rank: u8| {
            let mut r = Row::new(src, Duration::WhileSourceOnBattlefield);
            r.rank = rank;
            KeyedRow(r)
        };
        let a = reg.add(keyed(1));
        let b = reg.add(keyed(1));
        let c = reg.add(keyed(5));
        let g0 = reg.generation();

        let changed = reg.update_rows(|r| r.0.rank == 1, |r| r.0.rank = 9);
        assert_eq!(changed, 2);
        assert_eq!(reg.generation(), g0 + 1);
        assert!(reg.is_sorted());
        let ids: Vec<RowId> = reg.iter().map(|r| r.id()).collect();
        assert_eq!(ids, vec![c, a, b], "moved behind c, still in registration order");

        assert_eq!(reg.update_rows(|_| false, |_| {}), 0);
        assert_eq!(reg.generation(), g0 + 1, "a pass that changed nothing is not a write");
    }

    /// A fork shares every row, and `update_rows` copies only the rows `picks`
    /// accepts: a pass that picks nothing leaves every row shared, and one
    /// that picks one leaves the rest shared and the original's copy as it was.
    #[test]
    fn update_rows_copies_no_row_it_does_not_edit() {
        let mut reg: DurationRegistry<Row> = DurationRegistry::new();
        let sources: Vec<ObjectId> = (0..3).map(|_| new_object_id()).collect();
        for s in &sources {
            reg.add(Row::new(*s, Duration::UntilEndOfTurn));
        }
        let mut fork = reg.clone();
        let shared = |fork: &DurationRegistry<Row>| -> Vec<bool> {
            reg.rows.iter().zip(&fork.rows).map(|(a, b)| Arc::ptr_eq(a, b)).collect()
        };
        assert_eq!(shared(&fork), vec![true; 3], "a clone copies no row");

        assert_eq!(fork.update_rows(|_| false, |_| {}), 0);
        assert_eq!(shared(&fork), vec![true; 3]);

        assert_eq!(fork.update_rows(|r| r.source == sources[1], |r| r.created_on_turn = 2), 1);
        assert_eq!(shared(&fork), vec![true, false, true]);
        let turns: Vec<u32> = reg.iter().map(|r| r.created_on_turn).collect();
        assert_eq!(turns, vec![1, 1, 1], "the original is untouched");
    }

    /// Every path that changes the rows moves the generation; a removal that
    /// found nothing does not.
    #[test]
    fn generation_counts_writes_not_calls() {
        let mut reg: DurationRegistry<Row> = DurationRegistry::new();
        let src = new_object_id();
        assert_eq!(reg.generation(), 0);
        let id = reg.add(Row::new(src, Duration::UntilEndOfTurn));
        assert_eq!(reg.generation(), 1);
        assert!(reg.remove(999).is_none());
        assert_eq!(reg.generation(), 1);
        assert!(reg.remove_expired_at_turn_start(0, 5).is_empty());
        assert_eq!(reg.generation(), 1);
        assert_eq!(reg.remove_expired_at_cleanup(0, 5).len(), 1);
        assert_eq!(reg.generation(), 2);
        assert!(reg.remove(id).is_none());
        assert_eq!(reg.generation(), 2);
    }

    #[test]
    fn as_slice_admits_the_binary_search_effects_in_layer_needs() {
        let mut reg: DurationRegistry<KeyedRow> = DurationRegistry::new();
        let src = new_object_id();
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
        let src = new_object_id();
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
        let src = new_object_id();
        reg.add(Row::new(src, Duration::UntilEndOfTurn));
        reg.add(Row::new(src, Duration::WhileSourceOnBattlefield));
        reg.add(Row::new(src, Duration::Indefinite));

        assert_eq!(reg.remove_expired_at_cleanup(0, 1).len(), 1);
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn until_your_next_turn_needs_the_right_player_and_a_later_turn() {
        let mut reg: DurationRegistry<Row> = DurationRegistry::new();
        let src = new_object_id();
        reg.add(Row::new(src, Duration::UntilYourNextTurn));

        assert_eq!(reg.remove_expired_at_turn_start(0, 1).len(), 0, "same turn");
        assert_eq!(reg.remove_expired_at_turn_start(1, 2).len(), 0, "wrong player");
        assert_eq!(reg.remove_expired_at_turn_start(0, 3).len(), 1);
        assert!(reg.is_empty());
    }

    // The extra-turn case is `tests/phase_re1_integration_test.rs`'s
    // `until_your_next_turn_expires_on_a_real_extra_turn`, which makes the
    // claim against a turn the engine actually took.
}
