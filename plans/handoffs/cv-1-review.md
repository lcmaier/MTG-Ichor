# CV-1 review — one session, 2026-09-02

Review findings on PR #89 (`copy/cv-1-copiable-values`). Per
`engineering-practices.md` §4, one theme per session; this one is **"what the
type surface and the instruments cannot see"**, which is what most of the
comments turned out to be about. Delete this file when everything below has an
owner elsewhere.

Ordered: corrections first (something in the tree was wrong), then answers
(something in the tree was right and under-explained), then the two questions
that are genuinely open and get an owner rather than a fix.

---

## A. Corrections — fixed in this PR

### A1. `CopyRoles` could not express Mirrorform, and the reason is instructive

**Found in review.** CV-1 shipped `OthersCopyRecipient(PermanentFilter)`, with the
donor's exclusion **structural** on the argument that "each *other*" is the only
phrasing a class-scoped copy uses, so no card should have to spell it.

Mirrorform — "Each nonland permanent you control becomes a copy of target
non-Aura permanent" — prints the same shape **without the word "other"**. Its
affected set *includes* the target whenever its controller casts it. The old arm
could not say that at all.

Fixed: `FilteredCopyRecipient { filter, exclude_donor }`, and Mirrorform is
registered so the `false` leg has a consumer rather than being scaffolding.

**Why the census could not have caught it.** `copy-census.py` partitions the
corpus by *mechanism* — Tier A/B/C/D/E — and Mirrorweave and Mirrorform are the
same mechanism. A one-word difference *inside* one mechanism is precisely what a
mechanism partition cannot see, and it is the same failure mode §9 item 8 already
recorded about the clause being the wrong unit. **The census is not the
instrument for "can this type express the printed corpus"; reading three cards
in the bucket is.**

> **Practice, and it is cheap enough to be worth a line in
> `engineering-practices.md`:** before making a word in a card's text
> *structural* rather than *data*, find two more cards in the same bucket and
> check they print the same word. Structure is the expensive direction to be
> wrong in — data that is never varied costs a field, structure that has to vary
> costs an enum arm and every match on it.

### A2. `register_copied_static_effects` invented a controller

`.unwrap_or_else(|| ...owner...unwrap_or(0))` on a value that is **written into a
registry row** and resolves `PlayerRef::You` for as long as the row lives.
`gather` and `predicate` take `unwrap_or(0)` with a documented "unreachable
here" — but they spend it on one predicate evaluation and discard it. This site
persists it, so P0 would be a silently wrong board rather than a wasted walk.
Now a `debug_assert` and a skip.

### A3. `Frames/walk` 1.36 → 1.37 was attributed to a mechanism without evidence

The first draft said "which is Citanul Hierophants being copiable". **Counted
afterwards:** Cytoshape resolves **16 times in 200 games**, each row lives for
the rest of one turn, so copy rows exist during ~16 of ~6,160 turns — at most
~0.2 of the 0.7 percentage points even if every one had copied a static ability.
Arm C also plays *different games* (63-card pool, not 62; walks −0.5%, turns
+0.3%). **The shift is game content.** Corrected in `codebase-state.md`.

### A4. The Inside Out sentence was true and read as false

Audited rather than assumed: of the 70 registered cards, checked against
Scryfall, **Mirrorweave is the only one whose printed cost carries a hybrid or
Phyrexian symbol**. `inside_out` prints `{1}{U/R}`, the crate spells it `{1}{U}`,
and it **has never been registered** — that *is* the 2026-08-24 precedent, and
the entry now says so in the first clause instead of implying it.

### A5. A war story in a source comment

`compute.rs`'s `LAYER_ORDER` doc carried the "correct conclusion from a wrong
premise" paragraph. `CLAUDE.md`: *comment the why, and only where it is not
recoverable from the code plus one rule number; a war story goes in the commit
message or the architecture doc.* Trimmed to the rule plus a pointer to
`layers-architecture.md` §7.

### A6. `RecordingDp` was the third private decision-provider helper

Moved to `test_support::RecordingDecisionProvider`. RS-1 grew a private
`CountingDp`, CV-1 grew a private `RecordingDp`; a third copy is where a helper
stops being premature. It records the `ChoiceKind` rather than a count, which is
the part worth sharing — a test asserting "one prompt" passes just as well when
the prompt was the wrong one.

---

## B. Answers — the tree is right, the reasoning was not written down

### B1. `Box<CopiableValues>`, measured

`CopiableValues` is **328 bytes** (a `String`, five `HashSet`s, a
`Vec<AbilityDef>`, two `Option<i32>`); boxed it is **8**. Inline, the arm takes
`EffectModification` from **72 → ~336** and `ContinuousEffect` from **168 →
~432** — paid by *every* row, including the thousands carrying two `i32`s,
because `effects_in_layer` hands the walk a contiguous slice it re-iterates per
layer per object. **Row size is the layer walk's memory traffic.** Numbers now in
the variant's doc comment.

### B2. `CopiableValues` does not handle spell copies, and must not

CR 707.10's copy of a spell keeps modes, targets, X and the objects paid as
costs. None of that is in `CopiableValues` — correctly. §3.3's producer table
already says a spell copy uses "a new stack object; **no row, no layer**": CR
707.10 is not a Layer 1 effect at all. A spell copy needs the *characteristics*
half (this type) **and** a `StackEntry` half (modes, targets, X, cost payments),
and the second is CV-4's to build. Adding those fields now would be the
`back_face` mistake — a field no producer writes and no reader reads. **Stated in
the type's doc comment so the next reader does not have to re-derive it.**

### B3. `copiable_values` reads `game.objects` — there is no fast path to take

Restricting the capture to the battlefield would forbid CV-1b's capture from a
graveyard card and buy nothing: `compute_to_ceiling` reads `game.objects` for the
base characteristics either way, and `FrameCache::entity` already probes
`game.battlefield` for the controller seed. **The hidden-zone fast path already
applies** — `apply_effects` early-outs when the object is not on the battlefield,
has no CDA and the registry cannot reach it (§5.1), so a capture of a card in a
graveyard is cheaper than one of a permanent, not more expensive.

### B4. Two instances of one mana ability are CR-correct, and not an optimization

A copy of Citanul Hierophants grants the *same* `AbilityDef` (same `AbilityId` —
it is cloned from one `Arc<CardData>`), so an affected creature holds
`{T}: Add {G}` twice. **CR 113.10 says an object may have the same ability
multiple times and is treated as having it that many times**, and it matters:
two instances of a *triggered* ability trigger twice. Deduplicating the effective
ability list would be exactly the "semantics-assuming shortcut" `§12` item 3
forbids — valid only while no duplicated ability is order- or count-sensitive,
with a silently wrong answer as the failure mode.

Two observations that are real and are *not* CV-1's:

- `activatable_abilities` offers both instances, so a `DecisionProvider` sees two
  identical options. CR-correct (the object has two), and it changes a random
  agent's option distribution.
- **It predates this phase.** Two Citanul Hierophants on one board already do
  it — a copy is not required. Nothing here is new; only the way to reach it is.

### B5. A re-copy adds a row; it never overwrites one

Asked directly: should a re-copy replace the stored `CopiableValues` instead of
registering a second row? **No, and the reason is durations.** CR 613.2a orders
layer 1 by timestamp and *each row carries its own `Duration`* — an
`UntilEndOfTurn` copy laid over an `Indefinite` one must expire **back to** the
indefinite values. Overwriting would delete an effect that is still running.

The Deferred Migrations entry conflated three different teardowns and now
separates them: it is **not** item 10 (CR 400.7 is about `ObjectId` surviving a
zone change; a derived row's `source` is the copying permanent, which
`remove_by_source` already reaches), **not** an in-place update (above), and
**not** the pump-spell shape (whose subject left and returned). It is one
`retain` at re-copy time over derived rows whose ability id the *new* capture
does not carry — with the care entirely in "and only those", because the subject
may also **print** the same ability, which no copy row justifies or removes.

---

## C. Open, with an owner — not fixed here

### C1. The event log is not incomplete; the *instrument* is. One flag fixes it

**This does not reopen the delta-log fork**, and the distinction is worth keeping
because the two logs answer different questions:

| | Records | Its job | Is a copy in it? |
|---|---|---|---|
| **Event stream** (`GameEvent`, `--dump-events`) | performed *game actions* | CR 603.2 trigger detection — "whenever X happens" | **No, correctly.** Registering a row is not a game action and no card triggers on "became a copy" |
| **A state delta log** (dismissed 2026-08-24) | every characteristic change | nothing the CR asks for | n/a |

So the engine is **not** missing something it needs to track. What CV-1 found is
narrower and entirely about tooling: `--dump-events` is a *differential-testing*
instrument built on the trigger stream, and it inherits that stream's scope. A/B
for a phase whose whole output is a characteristic change needs a **state**
comparison, not an event comparison.

> **Proposed, and it is small: `--dump-state`.** At each priority boundary, hash
> every permanent's `EffectiveCharacteristics` (in `battlefield_ids_ordered`
> order) and emit one line per boundary. ~60 lines in `fuzz_games`, behind a
> flag, **no new tracking and nothing on `GameState`** — it is a checkpoint hash
> computed from `compute_characteristics`, which already exists. That is the
> whole difference from a delta log: a delta log records changes as they happen
> and must be maintained; this recomputes from state and cannot drift.
>
> **Owner: CV-2**, which is the first phase that would actually use it (an entry
> replacement changes characteristics *and* fires an entry event, so it is the
> first place the two logs can be checked against each other). Not built in CV-1
> because CV-1 has nothing to validate it against.

### C2. ATOM-702.131b-002 (Ascend, mid-resolution) — the mechanism is already right

Asked: *"event emission only happens after resolution, right? Are we set up to
handle this?"* Two separate things, and the atom is not about events.

**Events are not involved.** The atom turns on whether a *static ability's*
condition is re-read between clauses of one resolution. `resolve_effect` walks an
`Effect::Sequence` clause by clause, and every clause's `execute_action(s)` goes
through the pipeline, which reads `is_blocked` → `is_prohibited` →
`get_effective_abilities` → `compute_characteristics` **fresh**. Nothing is
cached across clauses. So the token is created, the board grows to ten
permanents, and the destroy clause asks a live question. **The mechanism is
right; nothing about the event stream's timing bears on it.**

What is missing is *features*, both already scheduled:

1. **The city's blessing** — a per-player designation (CR 702.131b), no
   representation today.
2. **Conditional static abilities** — "as long as you have the city's blessing,
   this creature has indestructible". `CLAUDE.md`'s critical path item 6 already
   says triggered abilities take "LKI formalization and **conditional static
   abilities**" with them.

So: no engine change owed, and the atom is correctly filed at Phase 8. Recorded
here because "is the engine set up for mid-resolution state change" is a
reasonable worry that deserves a checked answer rather than a shrug — and the
answer is yes, *because* nothing memoizes across a resolution.

### C3. `ChoiceKind` vocabulary — do not sweep, name well instead

**The cost/benefit is lopsided and it favours appending.**

- **Appending later is O(1) in the number of existing variants.** `ChoiceKind`'s
  own doc says adding a variant is "the ONLY change needed"; `cli.rs` and
  `random.rs` both have catch-alls, so nothing breaks. Measured:
  `ChooseCopySource` cost **14 lines** across two files.
- **There is no persistence, no wire format and no serialized save**, so a
  variant that turns out wrong is renamed by the compiler.
- **A sweep now would design for phases nobody has sized.** That is §9 item 10's
  named risk — "the third design pass in a row with no implementation between
  them" — and a vocabulary derived from a word search over the corpus would be
  partitioned by *English*, which is the same mistake §9 item 8 records about the
  clause being the wrong unit. "Choose" in card text covers targeting-adjacent
  selection, mode choice, division, a value, a player, a pile, and CR 700.2's
  modal menu; those are already different `ChoiceKind`s for reasons that have
  nothing to do with the shared verb.

**What *is* worth having is one naming rule**, because the only defect a late
variant can carry that is expensive to fix is being named for a *card* instead of
for the *question*:

> A `ChoiceKind` is named for the question the player is answering, never for the
> card or the mechanic that asks it. `ChooseCopySource`, not `Cytoshape`;
> `ChooseEnteringController`, not `CR616_1b`. A DP heuristic keyed on the variant
> should be right for every card that ever produces it.

And the one substantive lesson CV-1 does carry: **reusing a variant is the
mistake, not adding one.** `SelectRecipients` would have fit Cytoshape's prompt
mechanically and lied about it semantically — there the chosen object is what the
effect acts on, here it is the one permanent the effect does *not* change.

### C4. Reachability is thin — `--require` built here; the pool land is next

Counted: Cytoshape resolved **16 times in 200 `performance` games**. A 63-card
pool, a 60-card deck built from color-appropriate subsets, and a three-mana
two-color instant: any one card is rare, and it gets rarer with every phase
that adds one. **The second time a phase has had to say "the path is open but
barely"** — the first was RS-1.

**Built in this PR: `fuzz_games --require "A,B"`.** Forces one copy of each named
card into every deck, **seeds the deck's colors from theirs** so the cards are
actually castable, and prints casts, resolutions and the share of games each
reached. It fixes the confusion rather than the pool: `PERFORMANCE_POOL` is a
*cost* instrument that had been asked to double as a *coverage* one, and these
are now two flags answering two questions.

**Measured, 200 games / seed 12345 / `--threads 1`:**

| | without `--require` | with |
|---|---:|---:|
| Cytoshape resolutions (`performance`) | **16** | **401**, in 90% of games |
| Cytoshape (`stress`) | 0 in 40 games | 206, in 58% |
| Mirrorweave (`stress`) | 2 in 40 games | 211, in 61% |
| Mirrorform (`stress`) | — | 220, in 58% |

**An empty `--require` changes nothing**, and that is the property that makes it
safe to ship in the same binary as the timing arm: every RNG draw it adds is
guarded, and the default run reproduces the recorded fixture table to the digit
(93,914 walks / 1.45 frames-per-walk / 28.8 turns / 531 gathers).

**It found a live rules defect on its first run.** Resolutions exceeded casts,
which is impossible for a card that resolves once. Diagnosed the same session:
`cast.rs`'s `pay_costs(...)?` is the one fallible step after CR 601.2a with no
`rollback_cast_to_hand`, so a failed payment strands the card **on the stack**,
where it later resolves — unpaid, ~5 times per game. `codebase-state.md` item
**16c** has the evidence (206 in 40 games at 103acf1; every ghost card has a
generic cost component and no zero-generic card is ever one, which is the
allocation prompt's own gate) and the one-line fix. **Its own PR, and it should
precede everything** — and it did: `fix/cast-rollback-on-payment-failure`
(2026-09-02) closed it with the rollback, a regression test shown failing on the
pre-fix tree, and a harness invariant — `fuzz_games` now fails a run in which
anything leaves the stack by resolving without a `SpellCast` behind it, and
prints `Uncast resolved: 0` in every clean one. 16c's closing entry carries what
the free spells were worth in game content and answers the prompt-shape question.

**Known limitation, and it is the price of the color seeding.** `--require
Mirrorweave` makes every deck W/U, so the card is exercised against one color
pair and never meets a black, red or green card. The mode answers *"can the path
be walked"*, not *"what does it meet"*, and a phase must not read interaction
coverage off it. The seeding is a workaround for the missing five-color land,
not a design choice — with the land, forced insertion alone suffices and the
seeding can go. That is deliverable 2 of the land's handoff. **Done,
2026-09-03 (`pool/everywhere-land`):** the seeding is gone, every deck's mana
base taps for any color, and `--require` prints board diversity beside the
resolution count — `engineering-practices.md` §3 has the three numbers.

**~~Still owed, and it has its own handoff:~~ landed 2026-09-03 as its own PR,
not inside a phase, for the reason the handoff gave (it moves game content).**
The land is Everywhere rather than City of Brass — "add one mana of any color"
is not expressible, `backlog.md` §2.19 — and the handoff file is deleted. Note
what it was *not* asked to do: raise the unforced number. A card competes with
the whole pool for 36 slots and that ratio worsens with every card added;
`--require` answers "was the path walked", and the pool measures cost.

**Rejected: weighting new cards up in `random_deck`.** It makes the pool
unrepresentative in a way that silently distorts the timing arm, which is the
one thing `PERFORMANCE_POOL` exists to protect.

### C5. `apply_to`'s clone is the cost `layers-architecture.md` §12 already sized

`CopiableValues::apply_to` clones a `String`, five `HashSet`s and a
`Vec<AbilityDef>` into the frame, per affected object per walk — and
`Vec<AbilityDef>` is a **deep** clone, since `AbilityDef` owns a boxed `Effect`
tree. On a Commander board where Mirrorform has copied a permanent onto twenty
others, every walk of every one of them pays it twice: once seeding the frame
from `CardData`, once overwriting it from the capture.

**This is not a new lever, and that is the useful part.** §12 already measured
"eliding the per-frame `Vec<AbilityDef>` deep clone" at a uniform **~30%** and
already names the fix: make `CardData::abilities` an `Arc<Vec<AbilityDef>>` and
have the frame clone the `Arc`, with `Arc::make_mut` in the layer-4 and layer-6
arms that mutate it. `CopiableValues.abilities` takes the identical treatment and
`apply_to` becomes an `Arc` clone. **Answer-preserving, bounded, and it is
already next in §12's ordering** — CV-1 raises its value, it does not create it.

Explicitly *not* measured here: CV-1's reachability is too thin for a fuzz run to
see this, which is C4's point restated. The board that would show it is
synthetic, and `layers-architecture.md` §12 is where synthetic boards live.

### C6. Should CR 400.7 (item 10) be promoted on the critical path? No — it should be CV-1b's first commit

Asked directly, because CV-1b is blocked on it. **The answer is no, and the
reason is what the critical path is for.**

`CLAUDE.md`'s critical path orders **systems by what they unblock**. Item 10 is
not a system; it is one bounded fix — a teardown keyed on the *affected* object
alongside the existing `remove_by_source`. Check what it actually gates:

| Critical-path item | Blocked on item 10? |
|---|---|
| 5 (RC-5), 5b (RS-2), 5c (CV-2), 6, 6b, 7 | **No.** Every one is startable today |
| **CV-1b** (the indefinite 25) | **Yes**, and only this |

Promoting it would add a row to a file at its 200-line budget to schedule work
that exactly one sub-phase needs, ahead of five that do not.

**But it should stop being an unsized Deferred Migrations entry**, because two
things have changed since it was written:

1. **It has a second customer.** It was recorded as a pump-spell bug; copy rows
   are now a second class with the same shape, and CV-2's Tier B rows are *not*
   (those are torn down by source, structurally — `copy-effects-architecture.md`
   §5.3). So the customer list is closed and short: pumps, and Tier C copies.
2. **It is a live wrong answer, not a future one.** Giant Growth is in
   `PERFORMANCE_POOL`. A creature that dies and returns in the same turn still
   wears the pump. That is reachable in a fuzz game today and nothing detects it,
   because the harness compares event streams and this is a *state* error — C1's
   `--dump-state` is what would catch it, which is a second customer for that
   flag.

**The scheduling that follows: item 10 is CV-1b's first commit, not a phase of
its own.** CV-1b is "the indefinite 25", it cannot ship without item 10, and
item 10 is too small to be a phase. Writing it that way puts the fix behind a
consumer that tests it — which is the same rule every other phase follows, and
the reason `Duration::Indefinite` was held back from CV-1 in the first place.

**One thing that sizing must not skip**, and item 10 already says it: CR
400.7a—c's *exception* list currently works **by accident**, because the engine
never breaks the object relation at all. Implementing the default without the
exceptions would regress the control case and Xu-Ifit's rider. So "item 10" is
two pieces of work, and only the first is small.
