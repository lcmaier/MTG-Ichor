# The id hasher — what it is, and why it is ours

> Written 2026-09-16 with A4g (PR #158), the PR that replaced the v4 and v5 UUID ids
> with process-stable integers and put `types::ids::IdHash` on every map keyed
> by one. An explainer, not an authority: the code is `mtgsim/src/types/ids.rs`,
> the decision is `codebase-state.md` item 144 (archived), the measurement is
> `layers-architecture.md` §12's second 2026-09-16 re-read. If this file and
> the code disagree, the code is right and this file is wrong.

## 1. What a map asks of a hasher

A Rust `HashMap` does not hash its keys. It holds a `BuildHasher`, asks it for
a fresh `Hasher` on every lookup, feeds the key's `Hash` impl into that hasher
one word at a time, and reads 64 bits back from `finish()`. The map is
hashbrown, and hashbrown reads those 64 bits in two places:

- the **top seven bits** become the key's *tag*, a byte stored beside the
  bucket and compared sixteen at a time with SIMD before any key is touched;
- the **low bits**, masked to the table size, pick the *bucket* the probe
  starts at.

So a hash is good for hashbrown when both ends vary: distinct keys should land
in distinct buckets, and two keys that share a bucket should usually have
different tags, so that the probe rejects them without a key comparison. A
hash whose top seven bits are constant makes every entry in a probe group
look like a candidate; a hash whose low bits are constant piles every key
into one chain.

## 2. What the default cost

`std`'s default is `RandomState`, which is SipHash-1-3 keyed with two random
words drawn once per thread. SipHash is a keyed pseudorandom function built
to resist hash flooding — an attacker who can choose keys cannot make them
collide — and it pays for that with a fixed setup, a compression round per
eight bytes, and a finalization of four rounds, whatever the key's length.

Every memo, object and battlefield lookup in this engine went through it:
13.5 million `is_creature` probes in 200 four-seat `stress` games, 10.5
million `object_matches_filter`, 7.0 million `has_type`. With the ids as
16-byte UUIDs that was `hash_one::<&Uuid>` at 22.1% of all instructions on
2026-09-15, and 32.6% after A4f removed the clones around it — the largest
single row in the profile. Halving the key to eight bytes (A4g's first arm,
the type swap alone) took a tenth off that row, which says the cost was
mostly per call, not per byte.

Two UUID versions left with A4g. v4, minted from the operating system's
randomness for every object and every printed ability def, was the one the
profile saw. v5 lived at one site: `land_types.rs` derived the CR 305.6
intrinsic mana ability's id as a SHA-1 over the object id and a land-type
byte, because that ability is synthesized inside every recompute of the
frame and has nowhere to store a minted id — the one place the engine
already wanted a derived id, and the shape `AbilityId::derived_on` now
gives over the integer, without the hash.

## 3. What our keys are, and what that lets us skip

Both ids are now integers the game mints itself: an `ObjectId` is a counter
in `GameState`, stamped in `add_object`; an `AbilityId` is derived from the
card name and the def's ordinal, or from an object and a tag for a
synthesized ability. Three things follow.

- **No untrusted key ever reaches these maps.** A deck cannot choose its
  object ids, and a card file cannot choose an ability id's value. Hash
  flooding needs an attacker who picks keys; there is none. SipHash's one
  property we would be paying for is one we cannot use.
- **The keys are sequential.** Object ids run 1, 2, 3, … per game; a
  four-seat Commander game mints a few hundred. Sequential keys are the case
  the identity hash gets wrong (§4) and any multiply gets right.
- **The keys are process-stable.** This is the point of A4g, and it is what
  turns the hasher's seed into something we have to *choose* rather than
  something `RandomState` chose for us (§6).

## 4. Why not the identity hash, and why not a crate

**Identity.** The cheapest hasher returns the key. For sequential ids the low
bits differ, so the buckets are fine — but the top seven bits are zero for
every id below 2^57, so every key in a probe group carries the same tag and
hashbrown compares keys instead of tags. Worse for pairs: `(ObjectId,
AbilityId)` is two words, and an identity hasher has to combine them
somehow; XOR alone makes `(a, b)` and `(b, a)` collide and cancels shared
bits. The identity hash is what `nohash-hasher` offers, and it is the wrong
shape for this table.

**A crate.** `rustc-hash` (FxHash), `ahash` and `foldhash` are all good and
all faster than SipHash. Three reasons none of them is here:

1. **The mix is four lines.** FxHash's is one multiply and a rotate;
   foldhash's integer path is a multiply and a fold. There is nothing in a
   crate that a 64-bit multiply in `ids.rs` does not do for this key shape,
   and a dependency that saves four lines costs a supply-chain review the
   owner has declined more than once (`codebase-state.md` item 138, lever 7,
   records the stance for the allocator; the `uuid` crate this PR removed
   brought thirty crates with it).
2. **The seed has to be ours.** `ahash` and `foldhash` seed themselves from
   process randomness by default, which is exactly the property A4g had to
   take away: a map whose iteration order differs per process on purpose is
   what the three-run determinism check needs *only* when it is told to
   (§6). Their fixed-seed modes exist, but then the environment-variable
   read, the constant default and the CI wiring are ours anyway, and the
   crate is left supplying the one multiply.
3. **The hasher is part of the determinism contract**, and the contract is
   written in this tree (`CLAUDE.md`, "Determinism at the decision boundary";
   `engineering-practices.md` §3.1). A behavior the project's reproducibility
   rests on should be readable in the project, in full, by whoever next
   changes how ids are minted.

## 5. The mix, line by line

```rust
fn mix(&mut self, word: u64) {
    let x = (self.state ^ word).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    self.state = x ^ (x >> 29);
}
```

- **`self.state ^ word`.** The state starts at the seed, so the first word
  is mixed *with* the seed: two seeds hash one key to two values, which is
  what CI relies on (§6). For a pair the second word is mixed with the
  first's result, so both halves reach the hash and `(a, b)` and `(b, a)`
  differ (`a_pair_hashes_both_halves`).
- **`wrapping_mul(0x9E37_79B9_7F4A_7C15)`.** The constant is 2^64 divided by
  the golden ratio, rounded to odd — Knuth's Fibonacci hashing, the same
  constant inside splitmix64 and foldhash. Multiplying by an odd constant is
  a bijection on `u64` (distinct inputs stay distinct) and a carry only
  travels upward, so input bit *i* influences every output bit at or above
  *i*: the top seven bits, hashbrown's tag, end up depending on the whole
  word. The low bits of a product depend only on the low bits of the input,
  which is fine — sequential keys already differ there — and is why the tag
  is the end that needed the multiply. `the_hasher_spreads_a_run_of_counter_ids_across_the_tag_bits`
  reads that: 64 sequential ids must show more than 16 distinct tags.
- **`x ^ (x >> 29)`.** Folds the well-mixed high bits down into the bucket
  bits, so a key that differs from another only in its high bits — the two
  role bits of an `AbilityId`, the test range of a `new_object_id()` — still
  gets a different bucket. The shift is not sacred; anything from about 24
  to 33 works, and splitmix64's finalizer does three such rounds. One is
  enough here because the keys are not adversarial.

The general `write(&[u8])` path folds a byte slice a word at a time, so a
key that is not an id (nothing today) still hashes; the derived `Hash` of an
id newtype calls `write_u64` directly and never goes there.

## 6. The seed, and the check it re-arms

With `RandomState`, every map in every process iterated in its own random
order, and a sweep that leaked that order into a decision, a log or a count
showed up as a diff between two `fuzz_games` runs at one seed. That was the
three-run determinism check, and it came free.

With process-stable ids *and* a fixed hasher, the order would be the same in
every process, and such a sweep would agree with itself three times. So
`IdHash` reads `MTGSIM_HASH_SEED` once per process (a constant when unset,
so an unseeded run is still reproducible), CI's determinism step runs its
three runs under seeds 1, 2 and 3, and `fuzz_ab.py` gives each timing round
its own. Within one process every map shares the seed, which is why
`tests/determinism_test.rs` cannot see an order leak end to end and says so:
it tests the mechanism (the ordered sweeps come out in timestamp order) and
leaves the end-to-end check to CI.

`two_seeds_hash_one_key_differently` pins that the seed reaches the hash at
all — a seed that did not would leave the check disarmed silently.

## 7. What it does not do, and what it does not reach

It is not flood-resistant, on purpose, for §3's reason; if a future surface
ever lets an outside party choose a key that reaches one of these maps —
wire ids from a remote client (item 141) would be the first candidate — that
surface validates or remaps the ids at its boundary, and the maps stay as
they are.

It reaches the 27 declarations keyed by an id or an id pair. It does not
reach the frame's `HashSet<CardType>` and `HashSet<Subtype>`, which are
still SipHash and were 8.6 G of the 76.1 G four-seat `stress` profile after
A4g — the last `sip.rs` in the game, and a bitset question more than a
hasher one (`layers-architecture.md` §12, "What follows").

## 8. What it bought

Four seats on `stress`, 200 games, instructions a game: 622.4 M after A4f,
568.8 M with the type swap alone, 380.7 M with the hasher — −38.8%. The
`hash_one` row has no successor; the memo probe fell 72%; every
probe-shaped function fell by a third to a half; the functions the hasher
does not reach did not move by an instruction. Native, per decision: −37.3%
at four seats, −34.0% at two. `fuzz-record.md`'s A4g block has the rest.
