//! The engine's two identities, both process-stable: the same game at the same
//! seed mints the same ids in any process, on any thread
//! (`codebase-state.md` item 144; `CLAUDE.md`, "Determinism at the decision
//! boundary"). Every other id in the tree was already a counter; these two
//! were v4 UUIDs until 2026-09-16, and the cost was SipHash over sixteen bytes
//! at every memo, object and battlefield lookup — a third of a game.
//!
//! Two newtypes over one integer rather than two aliases of it, so that the
//! thirteen `(ObjectId, AbilityId)` sites cannot swap their halves silently.
//!
//! The hasher at the bottom of this file, and why it is written here rather
//! than taken from a crate, is `plans/id-hasher.md`.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hasher};
use std::sync::OnceLock;

/// Player identifier — index into the players array
pub type PlayerId = usize;

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

/// Unique identifier for an ability definition on a card.
///
/// **Per definition, not per object.** A plural token creation shares one
/// `Arc<CardData>` across equal defs and a copy keeps its source's defs, so
/// the engine keys an ability *on an object* as the pair `(ObjectId,
/// AbilityId)` and finds one by `a.id == id` within one object's effective
/// list — never across objects. Two fixtures both built as
/// `CardDataBuilder::new("Grizzly Bears")` therefore share ids by design.
///
/// The top two bits say where an id came from, so the three derivations can
/// never collide with each other; the rest is the derivation's own:
///
/// | bits | role | derived from |
/// |---|---|---|
/// | `00` | [`Self::UNASSIGNED`] | — |
/// | `01` | printed | the card's name and the def's ordinal in a walk of the card ([`Self::printed`]) |
/// | `10` | on an object | the object and a tag ([`Self::derived_on`]) |
/// | `11` | a test's | a process counter ([`new_ability_id`]) |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AbilityId(u64);

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
    pub const UNASSIGNED: AbilityId = AbilityId(0);

    /// A printed ability of the card named `card_name`: `ordinal` is its
    /// index in the printed list, or, past the list's end, its place among
    /// the defs nested in the printed effects. Pure in its inputs, so the
    /// same card built twice, in two processes, has the same ids.
    pub fn printed(card_name: &str, ordinal: u32) -> AbilityId {
        AbilityId(ROLE_PRINTED | ((fnv1a_64(card_name.as_bytes()) ^ ordinal as u64) & !ROLE_MASK))
    }

    /// An ability an effect synthesizes on `object` with nowhere to store a
    /// minted id — CR 305.6's intrinsic mana ability, built inside every
    /// recompute of the object's frame and handed out as an activation handle
    /// the next recompute must match. `tag` says which of the object's
    /// synthesized abilities this is (the land type's discriminant, there).
    pub fn derived_on(object: ObjectId, tag: u8) -> AbilityId {
        AbilityId(ROLE_ON_OBJECT | (((object.0 << 8) | tag as u64) & !ROLE_MASK))
    }
}

impl std::fmt::Display for AbilityId {
    /// Hexadecimal: the value is a derivation, and a decimal reading of it
    /// says nothing. Diagnostics only — a log names an ability by index.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", self.0)
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
    AbilityId(ROLE_TEST | NEXT.fetch_add(1, Ordering::Relaxed))
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

/// A `HashMap` keyed by an id or an id pair.
pub type IdMap<K, V> = HashMap<K, V, IdHash>;
/// A `HashSet` of ids or id pairs.
pub type IdSet<K> = HashSet<K, IdHash>;

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
        assert_eq!(printed.0 & ROLE_MASK, ROLE_PRINTED);
        assert_eq!(on_object.0 & ROLE_MASK, ROLE_ON_OBJECT);
        assert_eq!(test.0 & ROLE_MASK, ROLE_TEST);
        assert_ne!(AbilityId::UNASSIGNED.0 & ROLE_MASK, ROLE_PRINTED);
        assert_ne!(new_ability_id(), test);
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
