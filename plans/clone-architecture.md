# The clone — what a fork shares, and what it copies

> **Status:** design, 2026-10-21, for `roadmap-v2.md` B10; no code. The
> decisions in §3 wait on the owner's review, and the build's slot is decision
> 8. Every number in §2 was measured on 2026-10-21 on `b86c59c1` (#265's
> merge), and §2.0 says how.
> **Authority:** how `GameState` holds per-object state so that a clone shares
> it, namely the object store, the battlefield's entries, the stack's entries
> and order, the triggers waiting to be put on the stack, and the layer memo's
> frames; what a write must go through to keep a fork apart from its parent;
> and what reads floors 2 and 3 mid-turn. The floors themselves are
> `engineering-practices.md` §3.1's. On what exists, `codebase-state.md` wins;
> on what is being built, this file does; `CLAUDE.md`'s critical path owns the
> ordering.
> **Graduates:** `codebase-state.md` item 183's open half (a clone during the
> spike, and `FitOnClone`'s rule, which the owner gave this design on
> 2026-09-26) and item 293 (the
> state-based check's two walks of every object, folded in by the owner on
> 2026-10-21).
> **Companions:**
> - `engineering-practices.md` §3.1, the floors and their board; §11, the
>   invariant gates.
> - `layers-architecture.md` §12 "7a", the memo whose frames §3 decision 2
>   bounds; §5.2, the termination argument those frames serve, which nothing
>   here touches.
> - `triggers-architecture.md` §4, the dispatcher's queue and placement, which
>   write two of the stores.
> - `state/history_log.rs`, the precedent: rows in fixed chunks behind `Arc`s,
>   so a fork's first write copies one chunk.
> - `plans/references/ai-performance-floors.md`, where the floors come from.
>
> Phase codes are `CL-*`; branches are `state/cl-<n>-…`.

---

## 0. Where this sits, and why a document

**Not on the event path.** No proposal, replacement or trigger changes. This is
how `GameState` is stored. A search (the AI harness, and in Phase 10 parallel
play) clones the state at a decision and plays an answer on the clone, a fork
(`plans/glossary.md`). Floors 2 and 3 bound that clone: 10 µs, 64 allocations
as CI's proxy, and 128 KB. Today a clone deep-copies every object: a
permanent's 400 bytes, a stack entry's 408 bytes and its heap, and a waiting
trigger's 264 bytes and its three heap blocks. A wide board, a deep stack or a
big batch of waiting triggers therefore makes every clone pay for all of it.
The goal: a clone that copies a pointer per chunk of objects, and an object
copied only when a fork writes to it.

**A document of its own.** The build reaches into four subsystems' code (the
layer memo, the trigger queue and its keeper walks, the state-based check, the
battlefield's performers). It also owns a rule nobody owns yet: what a fork may
observe of what it shares (decision 7). `CLAUDE.md`'s architecture row gains a
name, not a line.

---

## 1. What a clone copies today

### 1.1 The stores

| Field (`state/game_state.rs`) | Today | Entry | Grows with |
|---|---|---|---|
| `objects` | `FitOnClone<IdMap<ObjectId, GameObject>>` | 48 B, no heap but a count on the card's `Arc<CardData>` | every card in the game, live tokens, every stack object |
| `battlefield` | `FitOnClone<IdMap<ObjectId, PermanentState>>` | 400 B; counters, attachments and combat lists on the heap: 0 allocations for 4,178 of 4,409 permanents read, at most 2 | permanents |
| `stack_entries` | `FitOnClone<IdMap<ObjectId, StackEntry>>` | 408 B; 4 allocations for a death trigger with one target | the stack |
| `stack` | `Vec<ObjectId>` | 8 B | the stack |
| `pending_triggers` | `Vec<PendingTrigger>` | 264 B; 3 allocations for the same trigger waiting | the triggers waiting |
| `layer_memo.frames` | `RefCell<IdMap<ObjectId, (u64, Arc<EffectiveCharacteristics>)>>` | 24 B and a count | **every object ever walked**: a stale frame is overwritten, never evicted (`state/layer_memo.rs:36`) |

`FitOnClone` (`types/ids.rs:450`) is item 183's half fix: a clone of a map
whose capacity is past twice its length is rebuilt at its length. The history
log (`state/history_log.rs:31`) is the tree's one store shared in chunks: rows in chunks
of 16 behind `Arc`s, so a clone copies one pointer and a fork's first write
copies only the open chunk.

### 1.2 Who writes and reads each store

Every access in `mtgsim/src` and `mtgsim/tests`, with comments and strings
blanked (`rust_source.code_only`), classified by what is called on the field.
"Engine" is `src` outside `#[cfg(test)]`, `test_support.rs` included.

| Store | Engine reads | Engine writes | Unit tests | Integration tests |
|---|---|---|---|---|
| `objects` | 81 `get`, 17 index, 12 `contains_key`, 4 walks | **4**: `add_object` (insert), `remove_object`, `set_object_timestamp`, `get_object_mut`, whose one engine caller is `arrive_in_zone` | 14, 4 writes | 115, 11 writes |
| `stack_entries` | 9 `get`, 2 `contains_key` | **6**: `set_stack_entry` (the one insert), `take_stack_entry` and `remove_from_zone_collection` (removes), three `get_mut` | 18, 5 inserts | 42, 10 writes |
| `battlefield` | 47 `contains_key`, 37 `get`, 8 index, 8 walks | **31**, by shape in §1.3 | 155, 38 writes | 565, 67 writes (64 field writes after `get_mut`, 3 removes) |
| `pending_triggers` | 3 walks, 3 lengths | **13**: three pushes, placement's take-partition-reassign twice and its `remove`, the two keeper walks' `&mut` loops, `end_game`'s clear | 4 | 118, 10 clears |
| `stack` | 25 | **5**: three pushes, a `retain`, a `remove` | 20, 7 pushes | 7 pushes |

The brief's counts are the same counts by another cut: its 66 battlefield
writers in `src` are the engine's 28 outside `test_support.rs` and the unit
tests' 38; `stack_entries`' 11 are 6 and 5; `objects`' 8 are 4 and 4, with
`get_object_mut`'s caller beside them.

Every engine walk of the three maps is already tagged `ORDER-FREE:`
(`engineering-practices.md` §11): each sorts by timestamp or epoch next, or
reads a set, a count or an `any`, so none observes the map's order.

### 1.3 The battlefield's 31 engine writers, by shape

1. **Membership, 2**: `insert_battlefield_entity`, `remove_from_zone_collection`.
2. **One permanent, one fact, by its performer, 15**: tap, untap and damage
   marked (`perform_action`); the deathtouch flag (`apply_deathtouch_flag`);
   `Primitive::RemoveAllDamage`; counters added and removed; `attach` (2) and
   `detach` (2); CR 613.7e's timestamp (`set_object_timestamp`); the entry's
   state (`perform_entry`, 3).
3. **Combat, 6**: attackers declared (1), blockers declared (2), removal from
   combat (3).
4. **A sweep that writes every permanent, 4**: the check's deathtouch flags
   (`engine/sba.rs:194`, `iter_mut`, at **every** state-based check), cleanup's
   damage (`engine/turns.rs:411`, `values_mut`), and the two control sweeps
   (`keep_frames_before_control_change`, `settle_announced_controllers`,
   `engine/triggers/control.rs:56` and `:98`), which run while a
   control-changing effect exists.
5. **A board a test or a scenario builds, 4**: `update_permanent` and
   `test_support`'s three combat setters.

**The stack side has shape 4 too.** `aura_put_into_graveyard_by_the_check`
(`engine/triggers/dispatch.rs:852`) and `for_each_lki_keeper` (`:1040`) take
`&mut` of every waiting trigger and every stack entry, whether or not they
write one. The second runs whenever a batch moves an object whose last known
information something may keep (`objects_lki_is_kept_for`, a read through the
same `&mut` walk).

Under copy-on-write, shapes 1, 2, 3 and 5 copy what they change. Shape 4 copies
everything a fork holds, and the check's sweep does it at every check. **The
eight walks of shape 4 are what the build rewrites** (decision 3).

---

## 2. The readings

### 2.0 How

A crate in the scratchpad depending on `mtgsim` by path, with `test-support`,
built in release, with `MTGSIM_HASH_SEED=1`. It plays `clone_bound_test`'s 24
games draw for draw (its deck builder, seeds and `GameConfig`). The two-seat
board is `close_out.py`'s 20 `performance` seeds on 60 cards and 20 life.
Allocations and bytes are exact, by the tests' counting allocator. Time is the
median of five means of N clone-and-drops. **"Every decision"** is a provider
inside `ManaWindowStop` that reads the state at each prompt with two or more
answers, the agent's decisions under the default bot stack. **Each sharing form
is built from the state's own entries and cloned in the store's place.** A
clone is its fields' clones in turn, so a form's whole clone is today's, less
the store, plus the form's store. No engine code changed.

The forms (decision 1): **A**, an `Arc` per entry in today's map; **C**, an
id-indexed table in chunks of 32, each chunk and each entry behind an `Arc`;
**B**, a persistent map, priced by its root (an `Arc` over a whole store).

### 2.1 The board's floors, re-read first

The board carried TR-5d's 41 allocations and 91.3 KB, and a 6.9 µs read by
nine single clones on 2026-10-06. Re-read before anything else and replaced in
place (`fuzz-record.md`, "Floors 2 and 3 re-read 2026-10-21"): **53
allocations** (`performance` 12353, turn 40), **90.8 KB** (`stress` 12351
under Teferi's clause, turn 80), and **8.1 µs**, the worst checkpoint of three
runs by the means method (6.9, 6.9, 8.1; two more read 7.4 and 7.5). Floor 2
has 21% room on allocations and 23% on time.

### 2.2 Field by field, between turns

`clone_bound_test`'s checkpoints, allocations / KB:

| Reading | Whole | `objects` | `battlefield` | `players` | source sets | the rest |
|---|---|---|---|---|---|---|
| 4 seats, worst allocations: `performance` 12353, turn 40 (48 permanents) | 53 / 64.6 | 1 / 28.5 | **30 / 28.9** | 10 / 3.0 | 3 / 0.4 | 9 / 3.9 |
| 4 seats, worst bytes: `stress` 12351 with the clause, turn 80 (56 permanents) | 31 / 90.8 | 1 / 28.5 | 2 / **51.3** | 12 / 3.5 | 6 / 0.6 | 10 / 6.9 |
| 2 seats, worst allocations: 12349 at its end (28 permanents) | 29 / 44.2 | 1 / 14.3 | 4 / 25.8 | 9 / 1.5 | 4 / 0.4 | 11 / 2.2 |

"The rest" holds the history (2 allocations), the registries (1–2) and the
crate-private fields, the layer memo among them. **The battlefield is most of
the worst allocations; the object store and the battlefield are most of the
bytes.**

### 2.3 At every decision

| | Decisions | Worst allocations | Worst KB | Mean allocations | Over 64 |
|---|---|---|---|---|---|
| 4 seats, today | 10,253 | **78**, `performance` 12356, turn 41, ordering 22 waiting triggers | 94.2 | 31.7 | **3** |
| 4 seats, A | | 37 | 22.6 | 29.2 | 0 |
| 4 seats, C | | 37 | 12.2 | 29.2 | 0 |
| 4 seats, the stack's stores only (`stack_entries` and the queue as C) | | 59 | 92.3 | 31.3 | 0 |
| 2 seats, today | 3,104 | 38 | 47.5 | 22.5 | 0 |
| 2 seats, A / C | | 28 / 28 | 11.1 / 5.8 | 21.3 | 0 |

At the 78-allocation ordering the waiting queue alone is 45 allocations for 22
triggers. The slowest decision clone read 9.0 µs (12353, turn 34, ordering 24
waiting, 68 allocations) by the median of five means of 200, the counting
allocator present and switched off. **`clone_bound_test`
reads none of these**: it clones between turns only, so CI passes a board that
breaks floor 2's proxy at three of its decisions.

### 2.4 The wipe, rebuilt

Item 183's board: four seats at 10,000,000 life, W death watchers ("whenever a
creature dies, target player loses 1 life and you gain 1 life") spread over the
seats, T goblin tokens, one batch destroying every creature.

The last two columns are the four stores the stack grows: `objects`,
`stack_entries`, `stack` and `pending_triggers`, under A (the order stays a
`Vec`) and under C (the order chunked too).

| Board | Today, the whole clone: allocations / KB / µs | A, the four: KB / µs | C, the four: KB / µs |
|---|---|---|---|
| W1 T99, 97 waiting | 304 / 64.7 / 15 | 2.9 / 0.6 | 0.0 / 0.06 |
| W1 T99, 97 stacked | 399 / 96.4 / 23 | 5.0 / 0.7 | 0.2 / 0.09 |
| W5 T195, 985 waiting | 2,968 / 554.5 / 182 | 12.0 / 5.6 | 0.3 / 0.13 |
| W5 T195, 985 stacked | 3,954 / 1,286.7 / 421 | 75.7 / **11.1** | 0.8 / 0.60 |
| W40 T160, 8,000 waiting | 24,013 / 4,337 / 1,800–3,000 | 66.8 / 44.3 | 2.1 / 0.97 |
| W40 T160, 8,000 stacked | 32,014 / 10,280 / 8,200–8,400 | **606.5** / **94.1** | **6.0** / **3.3** |

- **The board before the wipe, 200 permanents**, is a token board: 125.4 KB a
  clone today, 102.3 KB and 5.9–10.3 µs of it the battlefield. A: 4.3 KB and
  0.8 µs; C: 0.1 KB and 0.04 µs.
- **After all 8,000 resolve**: 13 allocations and 15.1 KB. Resolving leaves
  nothing behind on this board.
- **A shared pointer costs about 4.3 ns** counted up and down: 8,000 `Arc`s
  clone-and-drop in 33.6–34.9 µs, 8,000 integers in 1.0 µs. That is A's floor
  at depth, two pointers per stack object (its entry and its object): **a
  pointer per object does not hold floor 2's time past about 700 objects on
  the stack, nor floor 3 past about 1,800**, where both tables double.

### 2.5 Tokens made and gone: the memo

Ten rounds of 200 tokens made, walked and destroyed, on four seats with no
cards: with **no objects left in the game**, a clone grows from 1.5 KB to
**101.3 KB**, doubling as a hash table does (13.8, 26.7, 51.3, 101.3). It is the
layer memo's frames for 2,000 tokens that no longer exist. Floor 3 says "bounded
by the board's high-water mark and never by the turn count"; the memo is
bounded by every object ever walked.

### 2.6 What a fork writes between two decisions

The cost a shared fork pays instead: at each decision, the entries of each
store that differ from the previous decision's, and the chunks of 32 ids they
fall in. Four seats, 10,229 intervals (two seats in parentheses):

| Store | Entries written: mean, p90, p99, max | Chunks touched: mean, p99, max |
|---|---|---|
| `objects` | 1.16, 2, 4, 112 (0.72, 2, 3, 7) | 0.67, 3, 8 |
| `battlefield` | 1.84, 4, 18, 62 (1.40, 4, 10, 26) | 1.20, 7, 13 |
| `stack_entries` | 0.29, 1, 1, 4 (0.32) | 0.29, 1, 2 |
| `pending_triggers` | 0.07, 0, 2, 28 (0.01) | 0.01, 1, 1 |

Under C, a fork's first write to a chunk is one allocation, and to an entry
held behind its own `Arc` one more, plus the entry's own heap (none for most).
So **a fork's whole cost on the pooled board, the clone plus its writes to the
next decision, stays near today's in allocations**: about 29 for the clone and
4 to 6 for the writes (2.2 chunks, 2.2 entries behind their own `Arc`), against
today's 32 for the clone alone. In time, a 2–3 µs clone and a few tenths of a
µs of writes replace today's 5–8 µs clone. What changes is that none of it
grows with the board, the stack or the queue.

### 2.7 A read

A lookup on three boards at turns 40–50, hot cache, best of five:

| | Today | A | C |
|---|---|---|---|
| a permanent, present, reading `tapped` | 1.41–1.50 ns | 1.36–1.41 | **0.74–0.75** |
| the battlefield asked for every object id (`contains_key`) | 1.30–1.36 | 1.19–1.27 | **0.67–0.69** |
| an object, present | 1.66–1.80 | 1.45–1.53 | **0.64–0.71** |

C hashes nothing: an id is its own index. A microbenchmark with a hot cache is
not the engine's mix; the budget's arm reads the engine (§4).

**Time between turns.** Today's four stores are 4.5–5.0 µs of the slowest
checkpoints' 6.7–7.6 µs whole clone; chunked they are 0.11–0.14 µs, so the
slowest checkpoint's clone reads about 2.7 µs and the median 1.9.

---

## 3. The decisions

### Decision 1 — what is shared, and in what form

**The problem.** Each store holds its entries inline, so a clone copies each
entry and everything it owns. That is what grows with the board (§2.2,
§2.4's 200 permanents), the stack and the queue (§2.4). Every form below shares
entries between a fork and its parent and copies one on its first write after
the fork (`Arc::make_mut`). They differ in what a clone copies: one pointer per
entry, one per chunk of entries, or one per store.

| | Code | Clone: allocations / KB, between turns 4 seats · 2 seats; every decision 4 · 2 | The wipe, 8,000 stacked | Upkeep |
|---|---|---|---|---|
| **A. An `Arc` per entry in today's map** | `battlefield: FitOnClone<IdMap<ObjectId, Arc<PermanentState>>>`; a write, `Arc::make_mut(game.battlefield.get_mut(&id)?)` | 34 / 21.5 · 26 / 11.0; 37 / 22.6 · 28 / 11.1 | ~620 KB, ~95 µs: floor 3 five times over, floor 2's time ten | least code; the map keeps hashing, a table of 17 bytes a slot, and `FitOnClone` stays (decision 4) |
| **B. A persistent map** | an external crate (`imbl`'s hash array mapped trie, `rpds`), or the same shape written here as a trie keyed by id; `battlefield: imbl::HashMap<ObjectId, Arc<PermanentState>, IdHash>` | C's, less one allocation per non-empty store, and about C's KB: 32 · 24; 34 · 26 (projected) | ~9 KB, ~2 µs (projected) | a clone is one count at any size (0.003 µs measured on an `Arc` over a store); a read is the hash and 2–3 node hops; a fork's first write copies a path of 2–3 nodes. A dependency (the crate has one, `rand`; the hasher is written here on purpose, `plans/id-hasher.md`), or about 400 lines of persistent structure to own. Iterates in hash order |
| **C. Chunks, like the history log** | ids are a per-game counter, so `ObjectId(n)` lives in chunk n/32 at slot n%32: `objects: ObjectTable<GameObject>`, each chunk behind an `Arc`; a write copies the chunk if a fork shares it, then the entry | 34 / 12.1 · 26 / 5.7; 37 / 12.2 · 28 / 5.8 | ~15 KB, ~5 µs | one generic table, about 300 lines with tests. Reads hash nothing (§2.7). Iterates in id order. The outer vector spans every id minted, 8 bytes per 32 ids: 2 KB after the 8,000-trigger wipe, 250 KB after a million ids |

Not measured: B's reads and writes. No such crate is in the local cache, and
adding one is the owner's call.

**How an entry is held, under C.** A chunk is copied whole on a fork's first
write to it. So small entries with no heap of their own sit **inline** in the
chunk: an object (48 B) and a memo frame (24 B), whose chunk copy is 1.5 KB or
less. Large ones sit **behind their own `Arc`**: a permanent, a stack entry
and a waiting trigger. A chunk of 32 stack entries in a deep stack, copied
inline, would be about 13 KB and some 128 allocations for a fork that resolves
one trigger. The stack's order and the waiting queue are lists, not maps, and
take the history log's own shape: chunks of 32 in order, a fork's first write
copying the last.

**Recommended: C.**
- It is the only form measured to hold both floors on every board here.
- Its reads are cheaper than today's.
- It is the history log's idea, which the tree already owns.
- B's advantage over C, a clone of one count at any depth, shows only past
  about 15,000 objects on the stack, where C's four stores reach about 6 µs.
  It costs a dependency or a persistent structure.
- The outer vector's growth with ids minted is C's one bound past the board.
  If a board ever mints enough ids for it to matter, the step is a second level
  of chunks: 8 bytes per 1,024 ids, one more load per read.

### Decision 2 — which stores

**The problem.** A clone grows with four things the floors forbid it to grow
with: the stack (`stack_entries`, `stack`, and the stack's own objects in
`objects`), the waiting queue (`pending_triggers`), the board (`battlefield`,
`objects`), and the game's history (the memo's frames, §2.5). Item 183's
breach was the second (31 waiting triggers), its spike the first, and §2.4's
token board the third.

| Store | What it costs today, worst read | Under C | Why it is in |
|---|---|---|---|
| `stack_entries` | 32,001 allocations, 9.1 MB at 8,000 stacked | 2.0 KB, 0.94 µs | the deep stack (§2.4) |
| `stack` (the order) | 62.5 KB at 8,000 stacked | 2.0 KB | half of floor 3 on the same board |
| `pending_triggers` | 45 allocations at a 22-trigger ordering; 24,001 at 8,000 | one per non-empty list | the mid-turn breach (§2.3) |
| `objects` | 28.5 KB at both worst four-seat checkpoints; 912 KB at 8,000 stacked | 0.1 KB; 2.0 KB | a third of every clone's bytes, and every stack ability is an object |
| `battlefield` | 30 allocations at the worst checkpoint; 102 KB at 200 permanents | 0.1 KB | the worst between-turn reading, and token boards |
| the memo's frames | 101 KB with no objects left (§2.5) | bounded by the store | the only store that grows with the game's history |

**The memo takes two changes.** `remove_object` evicts the object's frame: an
id is never reused, so the frame can never be served again, and the memo is
then bounded by the object store. And its frames move into the same table,
inline, so a clone copies chunk pointers and a fork's walk copies the chunks
it stores into. Eviction alone would leave the memo's table copied whole at
every clone, at most one frame per live object.

**Not shared, and why.** After C, these are most of a clone's 29 mean
allocations:
- **The players' zones** (`library`, `hand`, `graveyard`, the counters and
  commander damage): 10–17 allocations at four seats. They are bounded by the
  seats, not by the board or the stack, so they hold no floor at risk. They are
  the next lever if floor 2 binds again.
- **The source sets** (`replacement_ability_sources` and its six siblings):
  3–6 allocations, 0.6 KB at most.
- The registries' payloads (shared since item 180), the history log (shared
  since TR-5h), the delayed triggers and the returns waiting: all inside
  §2.2's "rest", at most 11 allocations and 6.9 KB together.

**At every decision**, sharing the stack's three stores alone reads 59
allocations at four seats and 35 at two; all six, 37 and 28 (§2.3).

**Recommended:** the six rows above. Within the build the stack's stores go
first, as the brief orders, then the board's, then the memo.

### Decision 3 — how writes go through the copy

**The problem.** Copy-on-write is **correct** only if every write copies a
shared entry first. It is **cheap** only if nothing writes what it does not
change: a walk that takes `&mut` of every entry copies every entry in a fork,
whether or not it writes one. §1.3 counts 31 engine writers of the battlefield
and eight walks across three stores that are of the second kind. The options
below clone alike, (b) at form A's cost (decision 1); they differ in what
stops a wrong write and what it costs to keep.

| | Code | Correctness | Upkeep |
|---|---|---|---|
| **(a) The table is the door** | `fn get_mut(&mut self, id: &ObjectId) -> Option<&mut V>` copies the chunk, then the entry, if a fork shares them; the table offers no `iter_mut` or `values_mut`; nothing hands out an `Arc` | a raw write does not compile: there is no `&mut` to an entry but through `get_mut` | `get_mut`, `get`, `contains_key`, `insert`, `remove` and indexing keep `HashMap`'s signatures, so 27 of the 31 battlefield writers and the tests' 98 field writes keep their call sites |
| **(b) `Arc`s in today's maps, `make_mut` at each site** | form A's `Arc::make_mut(entry)` at the 54 engine write sites of §1.2 | a forgotten copy does not compile either, but `Arc::get_mut(..).unwrap()` does, and passes every unforked test: it fails only in a fork | needs a gate rule refusing `Arc::get_mut` on the stores |
| **(c) (a) with a gate** | `check_invariants.py` also refuses `make_mut` outside the table's file | as (a) | a rule refusing what the type already makes impossible |

**Recommended: (a), and no new gate rule.** The type refuses a raw write at
compile time, which is stronger than a grep. What remains is cost: a `get_mut`
that writes nothing, or a walk of shape 4. That is a copy, not a wrong answer,
and decision 6's fork-copies test reads it. The eight walks become a read, then
a write of only the entries that change:
- the check's deathtouch flags and cleanup's damage collect the ids whose
  value moves, then write each;
- the control sweeps write a permanent whose announced controller or kept
  frame changes;
- the keeper walks ask each keeper, by `&`, whether it keeps the object, then
  write its list.

### Decision 4 — `FitOnClone`'s rule after sharing

**The problem.** Item 183 left three costs on the rule: every clone of a
spiked map rebuilds it again; a rebuilt fork has no headroom; and tombstones
can hide a bloated table from `capacity`. Each is a cost of copying a hash
table's slots at a clone.

- **Under C: drop it.** None of the three stores is a hash table, so the type
  and its test go. The chunk vector holds 8 bytes per 32 ids, and a chunk
  emptied by its last removal is dropped. The id maps left on `GameState` (the
  source sets, `trigger_sources`, the zone maps, `blocker_damage_divisions`)
  hold 8–24 bytes a slot, at most 0.6 KB at every checkpoint, so none needs
  wrapping.
- **Under A: keep it.** A spiked table still copies every slot, 17 bytes each
  after sharing: 272 KB at 8,000 objects, rebuilt to about 8.5 KB at 400. The
  three costs shrink by the slot's size, from 400 bytes to 17, and stay.

**Recommended:** drop it, with C.

### Decision 5 — item 293's list of objects moved since the last check

**The problem.** CR 704.5d's sweep (a token in a zone other than the
battlefield ceases to exist) and CR 704.6d's (a commander put into a graveyard
or exile since the last check) each walk every object at every check
(`engine/sba.rs:137` and `:295`). An ability on the stack is an object, so after
a wipe stacks 8,000 death triggers, each check between resolutions walks 8,000
objects twice: 7.6 µs of a 12 µs check. Both read only what arrived in a zone
since the last check, and `arrive_in_zone` (`engine/zones.rs:250`) is the one
writer of the epoch that says when.

| | Shape | Clone | Upkeep |
|---|---|---|---|
| **(a) Every arrival** | `arrive_in_zone` pushes `ObjectRef`; the check takes the list at its top; each reader keeps an entry whose object is still that existence (its latest arrival) | empty at every grant of priority; one allocation at a decision during a cast, for the card's own move to the stack | `last_sba_check_epoch` and `moved_since` go; the scenario writer names the new field |
| **(b) The arrivals either reader can match** | as (a), pushing only a token arriving off the battlefield and a commander arriving in a graveyard or exile | as (a), and empty during a cast: a nontoken card's move pushes nothing | as (a); a new reader adds its filter |
| **(c) Read the history log since the last check** | the rows of moves newer than the check's last record | nothing new to clone | a silent move (`CAST-ROLLBACK:`) writes no row, and CR 704.5d must see a token outside the battlefield however it got there |

**Why "taken at the top" is exact.** Each check removes every token it finds
off the battlefield: no replacement pattern and no "can't" watches
`GameAction::TokenCeasesToExist`, and CR 111.7 and 704.5d offer none. So a
token there at a check's top arrived since the previous check's top. That is also
CR 704.6d's window as the code reads it today: since the start of the previous
check, so a commander CR 704.5g moves during a check is offered at the next.

**Recommended: (b).** Both readers then walk only their candidates, in epoch
order by construction, so their two `ORDER-FREE:` walks and sorts go. A debug
build keeps the old walk and asserts the list missed nothing. On the wipe the
check's 7.6 µs goes. Under C the list costs one allocation and 16 bytes per
entry, and only at a decision inside a check or a resolution that moved a token
or a commander.

### Decision 6 — the instrument

**The problem.** `clone_bound_test` reads floors 2 and 3 at item 143's
checkpoints, between turns, when the stack and the queue are empty. A search
clones at every decision, and §2.3's three over-64 decisions pass CI. A clone
reading also cannot see the cost decision 1 moves to the writes (§2.6).

| | What it reads | CI cost | Lands |
|---|---|---|---|
| (a) **Every decision** of the clone test's 24 games: a measuring provider inside `ManaWindowStop`, ≤ 64 allocations and ≤ 128 KB at each | 10,253 clones, where 188 are read today | about 1 s in release (the probe's run, cloning 15 times a decision, took 2 s) | with CL-1: it fails today |
| (b) **The fixture boards**: the wipe at 8,000 waiting and stacked, and the token churn of §2.5, under the same bounds | the depths and the history no seed reaches | about 1 s | with CL-1 |
| (c) **The fork-copies test**: at each decision of a few games, fork, answer, run to the next decision, and count the chunks the fork no longer shares with its parent (by pointer, in a test) | the write side: a walk of shape 4 shows as a copy of every chunk | about 1 s | with CL-1 |
| (d) **Time**, at the readiness pass: the median of five means of 2,000 at the 20 worst decisions by bytes and at the fixture boards | floor 2's own bound | none in CI | at A6e's readiness pass |

**Recommended: all four.** Floors 2 and 3 are then read at every decision and
on the fixture boards, not only between turns. `engineering-practices.md`
§3.1's definitions change in the build that makes them pass, not here.

### Decision 7 — determinism

**The problem.** Sharing adds things a game could observe that do not belong
to the game: whether an entry is shared, where it lives, and how many forks
hold it. None may reach a choice, a log or a count (`CLAUDE.md`, "Determinism
at the decision boundary").

1. **Order.** C iterates in id order, which is the same in every process, so
   the hasher can no longer leak through these stores. It is not the rules'
   order, though, and `ObjectId` has no `Ord` on purpose. A choice, a log or a
   count still sorts by timestamp. `check_invariants.py`'s unordered rule keeps
   reading the three fields: its type pattern gains the table's name, and the
   `ORDER-FREE:` tags stay.
2. **Sharing is never observed.** No `Arc::ptr_eq`, `Arc::strong_count` or
   address decides anything in the engine. A copy on write is not a
   `Diagnostics` row: an unforked game copies nothing and a forked one does,
   so a counter of copies would make the fixture tables differ by whether
   anyone forked. Pointer comparisons belong to tests (decision 6(c)).
3. **A fork iterates as its parent does.** Today a `FitOnClone` rebuild gives
   the fork a different hash order from its parent's. Nothing may observe it,
   but C removes the possibility.
4. **The memo stays per fork.** A fork's walk copies the chunks it stores
   into and leaves the parent's as they were, which
   `a_fork_reading_a_left_out_card_leaves_the_originals_memo_as_it_was`
   already asserts for today's map.

**Recommended:** the four as rules, the first enforced by the existing gate,
the second by review and §11's "what the gate does not see".

### Decision 8 — the builds' slot

**The problem.** The builds are a Phase 8 gate (the owner, 2026-09-26: the
floors hold a deep stack before Phase 8 prints the cards that make one).
`CLAUDE.md` places "B10's builds" after CV-7, near the end of the list Phase 8
is gated on. Three things bear on the slot:
- the mid-turn breach is on the pooled board today, and CI cannot see it until
  decision 6 lands;
- TR-6 (state triggers and the loop) and TR-7 (the persist loop) write the
  queue and the stack, the two stores the breach is in;
- each phase between now and the build adds writers to migrate.

| Slot | For | Against |
|---|---|---|
| **(a) Next, before TR-6** | CI reads every decision from then on; TR-6 and TR-7 are built on the shared stores; the census in §1.2 is today's | the triggers track waits one PR |
| (b) After TR-7, before A6e | the triggers track closes unbroken; A6e's readiness pass reads the new instrument | two phases that write the queue and the stack land without a mid-turn reading |
| (c) Where it is, after CV-7 | nothing clones in play until Phase 10's harness | about 30 steps of the route without a mid-turn reading, each adding writers |

**Recommended: (a).** It moves "B10's builds" on `CLAUDE.md`'s critical path
to beside "B10's design", and puts CL-1 after it in `roadmap-v2.md`'s A6d row.
Either edit is the review's, not this draft's.

---

## 4. The build, sized

One PR, in the band (`engineering-practices.md` §4), its commits in decision
2's order. Sizes are code plus tests.

| PR | Shape | Size | Consumer | Closes |
|---|---|---|---|---|
| **CL-1, the board and the stack shared between forks** | The chunked table, entries inline or behind their own `Arc`, and the chunked list, each with its tests. `stack_entries`, `stack` and `pending_triggers` first, then `objects`, then `battlefield`, and `FitOnClone` deleted. The eight walks of shape 4 as reads, then writes. The memo's eviction at `remove_object`, and its frames in the table. Item 293's list (decision 5(b)), with its debug walk. Decision 6's (a), (b) and (c) | table ~300 and list ~200, each half tests; the five stores' fields, constructors and the scenario writer's destructure ~150; the eight walks ~100; the memo ~80; item 293 ~110; the three instruments ~350; iteration sites whose item type changes ~50: **1,350–1,750** | the wipe, the churn and every decision of the 24 games, under floor 2's proxy and floor 3 | items 183 and 293 |

**If the first ticket's count passes 2,500**, the seam is the board: the stack's
three stores, item 293 and the instruments first, then `objects`,
`battlefield`, `FitOnClone` and the memo as a second PR. Each half carries a
consumer: the wipe for the first, the churn and the token board for the
second.

**Predictions** (the probe's model, §2.0):

| Reading | Today | After CL-1 |
|---|---|---|
| `clone_bound_test` between turns, 4 seats | 53 allocations, 90.8 KB, 8.1 µs | 34, 12.1 KB, about 2.7 µs |
| every decision, 4 seats | 78, 94.2 KB | 37, 12.2 KB |
| every decision, 2 seats | 38, 47.5 KB | 28, 5.8 KB |
| the wipe, 8,000 stacked | 32,014, 10 MB, 8.2 ms | about 15 allocations, 15 KB, 5 µs |
| the churn, no objects left | 101.3 KB | under 8 KB |
| instructions per decision, the budget's board | | within ±2.5 points. Reads lose the hasher (§2.7); each write checks a count, and each permanent, stack entry or waiting trigger made allocates its `Arc`, under one a decision (§2.6). The direction is the arm's to read |
| gameplay rows, both pools | | **IDENTICAL**: no rule reads the change |

---

## 5. Out of scope

- **The players' zones and the source sets** (decision 2): bounded by the
  seats and the board's sources, not by depth or history.
- **A6f's per-viewer knowledge record.** It is forked state that floor 3
  bounds, and its design is A6f's. It should be built on CL-1's table, and A6f
  reads this file.
- **Sharing across games.** Two games built from one registry share each
  card's `Arc<CardData>` already. Nothing here shares state between games.
- **CR 732's loop shortcuts.** C's outer vector grows with ids minted, and a
  loop mints ids. A loop the engine shortcuts mints none; until then, decision
  1's second level is the answer if a fixture shows it.

---

## 6. Found while designing

- **Today's clone touches counts every thread shares.** A clone bumps each
  object's `Arc<CardData>`, and every game built from one registry shares those
  cards: 400 atomic increments a clone at four seats, on cache lines each
  thread's games hit. In v1's parallel AI play, every clone on every thread
  contends there. Under C a clone touches only its own game's chunks; a first
  write to a chunk of objects touches its 32 cards. Not measured: CL-1's
  readiness reading can time clones on 16 threads against one.
- **The check's deathtouch sweep writes every permanent at every check**
  (`engine/sba.rs:194`), taking a flag that is almost always already clear.
  Harmless today; under any copy-on-write form it would copy the battlefield at
  every check of every fork. Decision 3 makes it a read first.
- **A pointer per object is not enough.** The brief's goal, "a clone that
  copies a pointer per object", is form A, and §2.4 measures it past both
  floors on a deep stack. The goal C meets is a pointer per chunk of objects.
- **A `StackEntry` is 408 bytes on today's tree**, where item 183 and the
  brief say 360, the size after TR-2b's shared effect (2026-09-27).
