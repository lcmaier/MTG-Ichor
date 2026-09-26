//! The engine's two identities, both process-stable: the same game at the same
//! seed mints the same ids in any process, on any thread
//! (`codebase-state.md` item 144; `CLAUDE.md`, "Determinism at the decision
//! boundary"). Every other id in the tree was already a counter; these two
//! were v4 UUIDs until 2026-09-16, and the cost was SipHash over sixteen bytes
//! at every memo, object and battlefield lookup — a third of a game.
//!
//! Two newtypes rather than two aliases of one integer, so that the thirteen
//! `(ObjectId, AbilityId)` sites cannot swap their halves silently.
//!
//! Beside them, the scalars an object is *referred to* by rather than named
//! by: CR 613.7's [`Timestamp`], CR 400.7's [`ZoneChangeEpoch`], and the pair
//! of the two that every durable reference to an object is made of,
//! [`ObjectRef`].
//!
//! The hasher at the bottom of this file, and why it is written here rather
//! than taken from a crate, is `plans/id-hasher.md`.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::sync::OnceLock;

/// Player identifier — index into the players array
pub type PlayerId = usize;

/// CR 613.7's timestamp: the ordering key within one layer, and the key of
/// every ordered battlefield sweep (CLAUDE.md, "Determinism at the decision
/// boundary"). Allocated from `GameState::next_timestamp` by `add_object` and
/// `move_object`, reassigned by 613.7e.
///
/// Here rather than in `engine::layers::types`, which re-exports it, for the
/// reason `ObjectSet` sits in `types::effects`: `GameObject` carries one and
/// `src/objects/` has no `crate::engine` edge to spend.
pub type Timestamp = u64;

/// CR 400.7's epoch: which existence of an object a reference means.
///
/// Allocated by `move_object`, the engine's one performer of zone changes, so
/// a remembered `(id, epoch)` pair stops resolving the moment the object moves
/// again — which is CR 603.6's "unable to be found in the zone it went to" and
/// 400.7's new object in one comparison. Pregame objects carry `0`, strictly
/// earlier than any move.
///
/// An alias and not a newtype, unlike the two ids above: those are newtypes
/// because `ObjectId` and `AbilityId` meet as a pair at thirteen sites and
/// could swap halves silently, and an epoch is paired with nothing.
pub type ZoneChangeEpoch = u64;

/// Unique identifier for a game object (card, token, copy, ability on stack, etc.)
///
/// Stamped by `GameState::add_object` from the state's own counter — the one
/// door into the store, beside CR 613.7d's timestamp — so a `GameObject` that
/// has not been added carries [`Self::UNASSIGNED`] and nothing else. A fork
/// clones the counter with the state, so two diverging branches mint the same
/// next id for different objects; nothing merges branches, so nothing sees it.
///
/// Never an order: `battlefield_ordered` keys on the CR 613.7 timestamp, and
/// this type deliberately has no `Ord`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(u64);

impl ObjectId {
    /// The id of an object that is not in a store: what `GameObject::new`
    /// hands out until `add_object` stamps a real one, and the id a test
    /// uses for "an object the store has never heard of". The counter starts
    /// at one, so no stored object ever carries it.
    pub const UNASSIGNED: ObjectId = ObjectId(0);

    /// The next id from a store's counter — `GameState::add_object`'s, or a
    /// test's own.
    pub(crate) fn from_counter(next: &mut u64) -> ObjectId {
        let id = ObjectId(*next);
        *next += 1;
        id
    }

    /// The integer, for a derivation that mixes it into another id.
    pub fn raw(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for ObjectId {
    /// `#17` — an id reads as one wherever it lands in a log line, beside a
    /// name or without one.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Unique identifier for an ability on an object: which definition, and —
/// for one a Layer 6 effect granted — which grant.
///
/// **Per definition, not per object.** A plural token creation shares one
/// `Arc<CardData>` across equal defs and a copy keeps its source's defs, so
/// the engine keys an ability *on an object* as the pair `(ObjectId,
/// AbilityId)` and finds one by `a.id == id` within one object's effective
/// list — never across objects. Two fixtures both built as
/// `CardDataBuilder::new("Grizzly Bears")` therefore share ids by design.
///
/// **Per grant, on top of that.** Two grants of one def put two instances on
/// one object (Diffusion Sliver's ruling: "cumulative"), and each triggers,
/// is activated and replaces on its own, so [`Self::granted_by`] tags the
/// granted copy with the granting row. Equality is per instance; a site that
/// means "this ability, whichever instance" — CR 113.10b's removal, the mana
/// window — compares [`Self::definition`]s (`triggers-architecture.md` §3.6).
///
/// The definition's top two bits say where it came from, so the three
/// derivations can never collide with each other; the rest is the
/// derivation's own:
///
/// | bits | role | derived from |
/// |---|---|---|
/// | `00` | [`Self::UNASSIGNED`] | — |
/// | `01` | printed | the card's name and the def's ordinal in a walk of the card ([`Self::printed`]) |
/// | `10` | on an object | the object and a tag ([`Self::derived_on`]) |
/// | `11` | a test's | a process counter ([`new_ability_id`]) |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AbilityId {
    definition: u64,
    /// The `EffectId` of the Layer 6 row that granted this instance, or `0`
    /// for one the object has by any other route. Registry ids start at one.
    grant: u64,
}

const ROLE_SHIFT: u32 = 62;
const ROLE_MASK: u64 = 0b11 << ROLE_SHIFT;
const ROLE_PRINTED: u64 = 0b01 << ROLE_SHIFT;
const ROLE_ON_OBJECT: u64 = 0b10 << ROLE_SHIFT;
#[cfg(any(test, feature = "test-support"))]
const ROLE_TEST: u64 = 0b11 << ROLE_SHIFT;

impl AbilityId {
    /// What a card file writes. `CardDataBuilder::build` replaces it with
    /// [`Self::printed`] on every def it can reach from the card — the printed
    /// list first, then every def nested in an effect (a granted ability, a
    /// token's abilities) — so a def that reaches an object still carrying
    /// this was never built into a card.
    pub const UNASSIGNED: AbilityId = AbilityId { definition: 0, grant: 0 };

    const fn defined(definition: u64) -> AbilityId {
        AbilityId { definition, grant: 0 }
    }

    /// A printed ability of the card named `card_name`: `ordinal` is its
    /// index in the printed list, or, past the list's end, its place among
    /// the defs nested in the printed effects. Pure in its inputs, so the
    /// same card built twice, in two processes, has the same ids.
    pub fn printed(card_name: &str, ordinal: u32) -> AbilityId {
        AbilityId::defined(ROLE_PRINTED | ((fnv1a_64(card_name.as_bytes()) ^ ordinal as u64) & !ROLE_MASK))
    }

    /// An ability an effect synthesizes on `object` with nowhere to store a
    /// minted id — CR 305.6's intrinsic mana ability, built inside every
    /// recompute of the object's frame and handed out as an activation handle
    /// the next recompute must match. `tag` says which of the object's
    /// synthesized abilities this is (the land type's discriminant, there).
    pub fn derived_on(object: ObjectId, tag: u8) -> AbilityId {
        AbilityId::defined(ROLE_ON_OBJECT | (((object.0 << 8) | tag as u64) & !ROLE_MASK))
    }

    /// This ability as the Layer 6 row `row` grants it, minted where the
    /// grant is applied (`compute.rs`). The row is the
    /// grant: it exists exactly as long as the grant does, its id is a
    /// counter no other row reuses, and nothing re-issues a row for a grant
    /// that continues (`CLAUDE.md` forbids reconciling the registry), so the
    /// instance keeps this id for its whole life and no other instance gets
    /// it. Not the source's epoch: a resolution's row outlives the spell's
    /// move to the graveyard and the ability object CR 608.2n deletes.
    pub fn granted_by(self, row: u64) -> AbilityId {
        debug_assert!(row != 0, "registry ids start at one; 0 means not granted");
        AbilityId { definition: self.definition, grant: row }
    }

    /// The ability, whichever instance: the id with its grant dropped.
    pub fn definition(self) -> AbilityId {
        AbilityId::defined(self.definition)
    }
}

impl std::fmt::Display for AbilityId {
    /// Hexadecimal: the value is a derivation, and a decimal reading of it
    /// says nothing. Diagnostics only — a log names an ability by index. A
    /// granted instance adds its row, `0x…/g7`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", self.definition)?;
        if self.grant != 0 {
            write!(f, "/g{}", self.grant)?;
        }
        Ok(())
    }
}

/// FNV-1a over `bytes`, 64 bits. Six lines rather than a dependency: the
/// only property [`AbilityId::printed`] needs is that one string always
/// gives one number.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// An `ObjectId` for a test that needs one without a store — a fake source
/// for a registry row, a `PermanentState` built bare. Distinct per call from
/// a process counter that starts far above any store's, so a test that mixes
/// both never sees them meet. Test-only: a production object is stamped by
/// `add_object` and nowhere else.
#[cfg(any(test, feature = "test-support"))]
pub fn new_object_id() -> ObjectId {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1 << 40);
    ObjectId(NEXT.fetch_add(1, Ordering::Relaxed))
}

/// An `AbilityId` distinct per call, for a test that authors a def whose
/// identity it will read back (CR 113.10b's "every instance of this id").
/// `CardDataBuilder::build` keeps an id that is already assigned, so a def
/// with one of these keeps it through the card. Test-only: a card file
/// writes [`AbilityId::UNASSIGNED`] and the builder derives the id.
#[cfg(any(test, feature = "test-support"))]
pub fn new_ability_id() -> AbilityId {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    AbilityId::defined(ROLE_TEST | NEXT.fetch_add(1, Ordering::Relaxed))
}

// ---------------------------------------------------------------------------
// The hasher every id-keyed map uses
// ---------------------------------------------------------------------------

/// The hasher of every map and set keyed by an id or an id pair
/// ([`IdMap`], [`IdSet`]).
///
/// SipHash defends against hostile keys, and no untrusted key ever reaches
/// these maps; what they need is a mix that spreads a run of sequential
/// counter values across both halves of the hash, because hashbrown reads
/// the top seven bits as its tag and the low bits as the bucket.
///
/// **Seeded per process, from `MTGSIM_HASH_SEED`.** With ids process-stable,
/// a fixed hasher would make every map's iteration order process-stable too,
/// and the three-run determinism check in CI would stop catching a sweep that
/// leaks that order into a decision. Three runs under three seeds restore the
/// property the check had under `RandomState`. Unset, the seed is a constant,
/// so an unseeded run is reproducible.
#[derive(Debug, Clone, Copy)]
pub struct IdHash {
    seed: u64,
}

impl Default for IdHash {
    fn default() -> Self {
        static SEED: OnceLock<u64> = OnceLock::new();
        let seed = *SEED.get_or_init(|| {
            std::env::var("MTGSIM_HASH_SEED")
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .unwrap_or(0x4D54_4749_4348_4F52)
        });
        IdHash { seed }
    }
}

impl BuildHasher for IdHash {
    type Hasher = IdHasher;

    fn build_hasher(&self) -> IdHasher {
        IdHasher { state: self.seed }
    }
}

/// [`IdHash`]'s hasher: one multiply and one fold per word written.
#[derive(Debug, Clone, Copy)]
pub struct IdHasher {
    state: u64,
}

impl IdHasher {
    #[inline]
    fn mix(&mut self, word: u64) {
        let x = (self.state ^ word).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        self.state = x ^ (x >> 29);
    }
}

impl Hasher for IdHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.state
    }

    /// The general path, for a key that is not an id: fold the bytes a word
    /// at a time. The derived `Hash` of an id calls [`Self::write_u64`]
    /// directly and never comes here.
    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut word = [0u8; 8];
            word[..chunk.len()].copy_from_slice(chunk);
            self.mix(u64::from_le_bytes(word));
        }
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.mix(i);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.mix(i as u64);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.mix(i as u64);
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.mix(i as u64);
    }
}

/// An object remembered by id **and** epoch (CR 400.7): a later move makes it
/// a new object the reference cannot find.
///
/// The pair every durable reference to an object is made of — [`AbilityIdentity`]'s
/// source, a trigger's bound subject, TR-3's delayed registry — so it lives
/// beside the two halves rather than in the subsystem that needed it first.
///
/// [`AbilityIdentity`]: crate::state::game_state::AbilityIdentity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectRef {
    pub id: ObjectId,
    pub zone_change_epoch: ZoneChangeEpoch,
}

/// A `HashMap` keyed by an id or an id pair.
pub type IdMap<K, V> = HashMap<K, V, IdHash>;
/// A `HashSet` of ids or id pairs.
pub type IdSet<K> = HashSet<K, IdHash>;

/// A map whose clone holds what the map holds, not the most it ever held: a
/// state map a game fills and empties — the object store, the battlefield,
/// the stack's entries.
///
/// A `HashMap` keeps its high-water capacity, which saves a reallocation in
/// play and costs every fork, which copies it (floor 3, `codebase-state.md`
/// item 183). So a clone of a map whose `capacity` is past twice its length
/// is rebuilt at its length, and any other is `HashMap`'s own, which copies
/// the table without rehashing. Twice is where growth leaves a map: its
/// table doubles when it fills, so from its first doubling on it holds at
/// least half its capacity, and the bar trips for a map that has shrunk
/// below its last doubling, whose rebuilt table is at most half the size.
///
/// Removals leave tombstones that `capacity` does not count, so a table
/// churned dense can read under the bar and be copied whole; a spike that
/// empties, the case this is for, reads over it. A rebuilt map iterates in
/// another order, which nothing may observe (`CLAUDE.md`, "Determinism at
/// the decision boundary").
#[derive(Default)]
pub struct FitOnClone<M>(M);

impl<K: Eq + Hash + Clone, V: Clone, S: BuildHasher + Clone> Clone for FitOnClone<HashMap<K, V, S>> {
    fn clone(&self) -> Self {
        let map = &self.0;
        if map.capacity() <= 2 * map.len() {
            return FitOnClone(map.clone());
        }
        let mut fit = HashMap::with_capacity_and_hasher(map.len(), map.hasher().clone());
        fit.extend(map.iter().map(|(key, value)| (key.clone(), value.clone())));
        FitOnClone(fit)
    }
}

impl<M> Deref for FitOnClone<M> {
    type Target = M;
    fn deref(&self) -> &M {
        &self.0
    }
}

impl<M> DerefMut for FitOnClone<M> {
    fn deref_mut(&mut self) -> &mut M {
        &mut self.0
    }
}

/// The map's own, so a state prints as it did.
impl<M: fmt::Debug> fmt::Debug for FitOnClone<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_printed_id_is_a_function_of_the_name_and_the_ordinal() {
        assert_eq!(AbilityId::printed("Grizzly Bears", 0), AbilityId::printed("Grizzly Bears", 0));
        assert_ne!(AbilityId::printed("Grizzly Bears", 0), AbilityId::printed("Grizzly Bears", 1));
        assert_ne!(AbilityId::printed("Grizzly Bears", 0), AbilityId::printed("Llanowar Elves", 0));
    }

    #[test]
    fn the_three_derivations_live_in_different_ranges() {
        let mut counter = 1;
        let object = ObjectId::from_counter(&mut counter);
        let printed = AbilityId::printed("Forest", 0);
        let on_object = AbilityId::derived_on(object, 3);
        let test = new_ability_id();
        assert_eq!(printed.definition & ROLE_MASK, ROLE_PRINTED);
        assert_eq!(on_object.definition & ROLE_MASK, ROLE_ON_OBJECT);
        assert_eq!(test.definition & ROLE_MASK, ROLE_TEST);
        assert_ne!(AbilityId::UNASSIGNED.definition & ROLE_MASK, ROLE_PRINTED);
        assert_ne!(new_ability_id(), test);
    }

    #[test]
    fn two_grants_of_one_ability_are_two_instances_of_one_definition() {
        let printed = AbilityId::printed("Diffusion Sliver", 3);
        let (first, second) = (printed.granted_by(7), printed.granted_by(8));
        assert_ne!(first, second);
        assert_ne!(first, printed);
        assert_eq!(first.definition(), printed);
        assert_eq!(second.definition(), printed);
        assert_eq!(first.to_string(), format!("{printed}/g7"));
    }

    #[test]
    fn an_id_on_an_object_is_distinct_per_object_and_per_tag() {
        let mut counter = 1;
        let a = ObjectId::from_counter(&mut counter);
        let b = ObjectId::from_counter(&mut counter);
        assert_ne!(AbilityId::derived_on(a, 1), AbilityId::derived_on(b, 1));
        assert_ne!(AbilityId::derived_on(a, 1), AbilityId::derived_on(a, 2));
    }

    #[test]
    fn a_test_object_id_never_meets_a_store_counter() {
        let mut counter = 1;
        let stored = ObjectId::from_counter(&mut counter);
        let test = new_object_id();
        assert_ne!(stored, test);
        assert!(test.0 >= 1 << 40);
        assert_ne!(new_object_id(), test);
    }

    #[test]
    fn the_hasher_spreads_a_run_of_counter_ids_across_the_tag_bits() {
        // hashbrown's tag is the top seven bits: a mix that left them
        // constant across sequential keys would probe every group entry.
        let build = IdHash { seed: 7 };
        let tags: HashSet<u64> = (1..=64u64)
            .map(|i| build.hash_one(ObjectId(i)) >> 57)
            .collect();
        assert!(tags.len() > 16, "{} distinct tags over 64 sequential ids", tags.len());
    }

    /// Item 183: a map that once held a hundred entries and holds three
    /// clones at three's size, and the original keeps its capacity for play.
    #[test]
    fn a_clone_holds_what_the_map_holds_not_the_most_it_held() {
        let mut map: FitOnClone<IdMap<ObjectId, u64>> = FitOnClone::default();
        for i in 1..=100u64 {
            map.insert(ObjectId(i), i);
        }
        map.retain(|id, _| id.0 <= 3);
        let fork = map.clone();
        assert_eq!(*fork, *map, "the same entries");
        assert!(fork.capacity() < 8, "{} slots for three entries", fork.capacity());
        assert!(map.capacity() > 2 * map.len(), "the original keeps its table for play");
        // A map at its steady size is cloned as `HashMap` clones it.
        let steady = fork.clone();
        assert_eq!(steady.capacity(), fork.capacity());
    }

    /// The seed is what CI's three-run step varies: a seed that did not
    /// reach the hash would leave every map in one order in every process.
    #[test]
    fn two_seeds_hash_one_key_differently() {
        let one = IdHash { seed: 1 };
        let two = IdHash { seed: 2 };
        assert_ne!(one.hash_one(ObjectId(17)), two.hash_one(ObjectId(17)));
        assert_eq!(one.hash_one(ObjectId(17)), IdHash { seed: 1 }.hash_one(ObjectId(17)));
    }

    #[test]
    fn a_pair_hashes_both_halves() {
        let build = IdHash { seed: 7 };
        let a = (ObjectId(1), AbilityId::printed("Forest", 0));
        let b = (ObjectId(1), AbilityId::printed("Forest", 1));
        let c = (ObjectId(2), AbilityId::printed("Forest", 0));
        assert_ne!(build.hash_one(a), build.hash_one(b));
        assert_ne!(build.hash_one(a), build.hash_one(c));
    }

    #[test]
    fn an_id_displays_as_a_hash_number() {
        assert_eq!(ObjectId(17).to_string(), "#17");
    }
}
