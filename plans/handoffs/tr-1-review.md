# TR-1 review — findings, triaged, and the order that closes them

The owner's review of [PR #173](https://github.com/lcmaier/MTG-Ichor/pull/173),
2026-09-20, captured before anything was fixed (`engineering-practices.md` §4).
Thirty-eight comments, two defects found while answering them, five themes in
the order below. **Close one theme per session, starting cold from this file**;
delete the file in the PR that lands the last one, redirecting every pointer the
way `138eeb8` did for A4i's.

Nothing here is in the PR. The brief said findings go here and not into the PR,
so every theme is a follow-up PR off `main` after #173 merges; the owner may pull
theme A forward if one rename before the merge is preferred to a second sweep
over the same lines. Comments are numbered in the order they were asked
(#1–#38) and indexed at the bottom; the two defects are F1 and F2.

---

## The order, and why

| | Theme | Closes | Why in this position | Gate |
|---|---|---|---|---|
| 1 | **A — names, shapes, comments** | #1 #4 #11 #12 #13 #14 #17 #22 #24 #26 #27 #28 #29 #30 #33 #34, F2 | Mechanical and agreed; everything after it — and every site TR-2 adds — is written against the final names, so it happens first and once | `cargo test`, zero warnings, the six gates, three hasher seeds; `fuzz_ab.py` engine vs `main` `IDENTICAL` on every row (a rename cannot move a counter; the fold and the field cuts are real edits) |
| 2 | **B — the authoring surface** | #5 #6 #7 #36 | Small, and TR-2's seven cards are written through it; after A so the constructors are born under the final names | the same, plus `check_rulings --check` (five cards re-expressed, texts unchanged) |
| 3 | **C — the matcher** | F1 #16 #38, §11's kind mask | Three edits to one function and one test; F1 is a wrong answer the day TR-4 registers a mixed-zone card, and the mask is the lever §11 pre-approved, whose reading was taken 2026-09-20 (the note under C) | the fixture red before the fix and green after; `fuzz_ab.py` `IDENTICAL` (no pooled card has a mixed-zone trigger, the flatten is a refactor, a mask skips only visits that would not have matched); the trigger-heavy probe re-run and recorded |
| 4 | **D — the docs, and three decisions** | #2 #3 #8 #10 #21 #23 #31 #35 #37, the keeps | No code. Item 167 restated, the band said, three owner's calls written down with a recommendation each; TR-2's brief reads this theme before it is written | the six gates (`check_state_of_play --write`) |
| 5 | **E — the decided engine work before TR-2** | what D decides of #2 and #23 | The two edits TR-2's gates and TR-4's frame would otherwise build on top of | `fuzz_ab.py` with the cost measured — the snapshot adds work at a batch's open on boards with rows, and Humility is pooled |

Then TR-2, whose brief re-counts its rows against the tree the way every brief
does; theme D says what to re-count with.

---

## A — names, shapes, comments · *closed 2026-09-20, [PR #174](https://github.com/lcmaier/MTG-Ichor/pull/174)*

Rename sweeps anchored on the type's own methods, never on a bare word
(`sizing-and-doing-a-rename-sweep`); the site counts below are `grep -rn` over
`src` and `tests` on 2026-09-20.

| # | Site | Action | Size |
|---|---|---|---|
| 4 | `ObjectFilter::EachOther` | → **`NotSource`**. Neither is the CR's word ("another"); `NotSource` says what the leaf reads — the ability's source, CR 113.7's word. Its doc comment must say that for a granted ability the source is the *carrier*, not the granter (the matcher passes the candidate). | 31 sites in 14 files, 11 doc mentions, no glossary anchor |
| 11 | `dispatch.rs` `struct Candidate` | → **`TriggerCandidate`** — the replacement pipeline has a private `Candidate` and a `CandidateRow` (`pipeline.rs:135`, `:642`); no compile clash, grep-hostile | 1 file |
| 12 | `dispatch.rs` `struct Match` | → **`MatchedTrigger`** | 1 file |
| 13, 17 | `dispatch_inner`, `arm_occurrences` | The walkthroughs given in review become the functions' doc comments: the five steps (gate · candidates · match · queue or resolve · the tier-2 emission) on the first, and the one paragraph ("one arm against one record; a field left `None` is not asked; `one` wraps a boolean as an occurrence") on the second. A reader was lost; the code did not say its shape | ~25 lines of comment |
| 14 | `Vec<(u64, ObjectId)>` at `dispatch.rs:275` | Use the existing **`Timestamp`** alias (`layers/types.rs:35`), and take it to the three sites that still say `u64` — `GameObject::timestamp`, `object_timestamp`, `next_timestamp`/`allocate_timestamp`. `zone_change_epoch` has no alias anywhere; coin one (`ZoneChangeEpoch`) for `ObjectRef`, `AbilityIdentity`, `GameObject` and the counter, for the same reason | ~12 sites |
| 22 | `RegistryScopeSummary::{granted,copied}_trigger_zones` | **Fold into one `unattributed_trigger_zones`**: the two are only ever read ORed (`dispatch.rs:187`), and the replacement twin already folded its granted and copied legs into `unattributed_replacement_zones` for that reason (`continuous_effects.rs:222` and the comment above it). Two writers (`:242`, `:257`), one reader, the `game_state.rs:534` comment | ~10 lines |
| 24 | `GameState::dispatch_depth` | Kept on the state — item 40 is why: the re-entry goes through the emit path, which has no parameter channel, and a clone at a prompt must carry it. **The objection is surface, so fold the three guards** — `batch_depth`, `decomposition_depth`, `dispatch_depth` — **into one `NestingGuards` field**, which removes two fields rather than adding a third. `actions.rs`' three wrappers, `diagnostics.record_batch_depth`, the dispatcher | ~40 lines |
| 26 | `TriggerCondition` beside `Condition` | **Keep both.** CR 603.1 names the when-clause the "trigger condition"; the clash is with the engine's generic predicate type at 102 sites, and renaming that to `Predicate` is the sweep that would fix it. A glossary line: *trigger condition* is the when-clause, *condition* a predicate over state | glossary |
| 27 | `TriggerCondition::arms()`, `ArmIndex`, `TriggerBinding::arm` | → **`events()`**, **`EventIndex`**, **`event`**. "Arm" is also the docs' word for an enum variant | 6 + 8 sites |
| 28 | `Tier` | → **`TriggerTier`** | 31 sites, no collision |
| 29 | `Occurrence` | → **`Multiplicity`** — reads as "the arm's multiplicity" at the field; the variants keep CR 603.2c's own noun (`PerOccurrence`, `OncePerEvent`). `TriggerFires` reads as a verb | 19 sites |
| 30 | `Subject` | → **`TriggerSubject`** | 32 sites |
| 33 | `ObjectRef` in `types/triggers.rs` | Its sibling is not `PlayerRef` (authoring vocabulary) but `AbilityIdentity`'s `(source, zone_change_epoch)` pair, which is the same two fields spelled out. **Move it to `types/ids.rs` beside `ObjectId` and reuse it: `AbilityIdentity { source: ObjectRef, ability, instance }`.** TR-3's delayed registry is its second consumer | 6 + 12 sites |
| 34 | `PendingTrigger` | Ten fields, three dead: **`def` duplicates `binding.def`; `tier` is `def.condition.tier()`; `mana` exists only for the trace** and the trace site can call `is_mana_ability`. Seven left. (The queue is the registry: a `Vec` because dispatch order is the default order the prompt offers and it is drained whole; the row exists because the `StackEntry` cannot be built until targets and the object id are chosen at placement.) | ~15 sites |
| 1 | `fuzz-record.md:72` "identical counters" | The struct became `Diagnostics` in `9486ccf` because the CR owns "counters" *in code*; that commit kept the printed row names and the record's prose on purpose, and the record says "every counter" in three earlier blocks' tables. **A glossary line**: prose about the struct says *diagnostics rows*; the printed row names and the tables' "every counter" cells stay, because `fuzz-record.md` is keyed on them | glossary |
| F2 | `placement.rs:118` and `triggers-architecture.md` §5.4 | **The comment is wrong about the code.** It says CR 603.3d's removal "never creates the object"; `place_one` creates it, pushes it, announces targets against it, and removes both on failure — which is the CR's literal order ("removed from the stack"), and the object has to exist because targeting legality reads the source. Rewrite both to say so, and say what the equivalence rests on: nothing between creation and removal is proposed or emitted; the writes are an id, a timestamp and two epoch bumps, none of which reaches an outcome; 603.3a's controller was fixed at dispatch and 603.3b's order was taken before targets in the CR's own sequence, so a removed trigger consumed its slot exactly as in paper | 2 comments |

**Recorded keeps** (answered in review; here so they are not re-asked): #19
`retain` keeps the stack's order where `swap_remove` would move the last object
into the hole, and matches the three sibling removals; at 608.2a the object is
the top by construction, so a `pop` with a debug assertion would also be exact.
#20 the three `EventLog` wrappers stay — `records` is private, so `record` is
the newtype's only reader (14 callers); `next_seq` (one caller) keeps the
length-to-sequence conversion inside the log; `emit_unstamped` is the second
door and is named so the exemption is greppable. #32 `TriggerSeq` pairs with
`EventSeq`, the other monotone counter; the `*Id` family is hashed ids, so
`TriggerId` would put it in the wrong family by name.

---

## B — the authoring surface · *small; TR-2's cards use it*

**#5, #6, #7, #36.** A card file spells `TriggerEvent::ZoneChange { subject,
from: Some(Battlefield), to: Some(Graveyard), cause: None, owner: None,
occurrence }` for "dies", and `whose: None` for "each upkeep". The reader's
objection is right on both counts: a defined CR term deserves a constructor,
and `None` reads as "nothing" where the card means "any". The test file already
has `dies`, `at_upkeep`, `creature_enters`; `phase_tr1_cards.rs` has
`triggered_ability`, `whenever`, `another`. There is no shared card-helper
module today — helpers live per phase file and in `test_support`.

**Action.** One module (`src/cards/authoring.rs`, or the owner's name) holding
`triggered_ability`, `whenever`, `at_beginning_of`, `dies`, `enters`,
`leaves_the_battlefield`, `another`; the five TR-1 cards and the TR-1 fixtures
re-expressed through it; `phase_tr1_cards::another` and the test file's copies
deleted. The constructors take the "any" reading as a word, never as `None`:
`at_beginning_of(Step::Upkeep, Each)` / `(…, You)`, `dies(a_creature())` with
cause and owner unasked, and a builder (`.caused_by(Sacrificed)`,
`.owned_by(Opponent)`) when a card asks. **The `Option` fields stay at the type
level**: the replacement patterns share the convention ("`None` on a field means
any", `types/replacement.rs:237`), so changing the type forks two surfaces or
sweeps both, and a GUI card builder presents "Any" itself whatever the Rust
type says.

**`dies` for a land (#36).** `tmnt.txt` 700.4 is "The term dies means 'is put
into a graveyard from the battlefield.'" with no creature restriction — the
sentence limiting it to creatures is not in the version the engine targets. So
`dies(filter)` is the constructor for any permanent, and the test at
`phase_tr1_integration_test.rs:457` is not wrong. Titania, Protector of
Argoth's Oracle text still prints the long phrase for lands (Scryfall,
2026-09-20), so Phase 8's parser maps both spellings to the one constructor.

Size ~120 lines. Gate: the A gate plus `check_rulings --check`.

---

## C — the matcher · *three edits to one function, one test, and §11's lever*

**F1 — the dispatcher does not apply CR 113.6 per ability for an off-battlefield
source.** `register_static_effects` puts into `zone_trigger_sources` only the
ability ids that function in the object's zone (`game_state.rs:1566`), but
`find_matches` reads the map's *keys* (`dispatch.rs:275`) and then asks every
triggered def on the object's effective list; `match_def` has no zone check.
The replacement gather makes exactly this check per def
(`gather.rs:535`, `functions_in(ability, &chars.types, zone)`); the dispatcher
has no such line.

*Proved 2026-09-20 with a throwaway fixture (deleted):* an Ichorid-shaped
creature card — "when this card is put into a graveyard from anywhere" beside
"whenever a creature dies" — in a graveyard, and a creature dies: **pending 1,
where the answer is 0**; the same card on the battlefield: 1, correct.

*Reachability:* no registered card has a trigger that functions in one zone
beside one that functions in another, so unreachable today; wrong the day TR-4
registers Bloodghast or Ichorid, and TR-4's own row names neither, so this file
is the only thing holding it. **If C has not landed when this file is next
touched, file it as a `codebase-state.md` item under "Found by TR-1" with this
text.**

*Fix (~10 lines):* for a leg-3/4 candidate, skip a def unless
`functions_in(ability, &chars.types, candidate.zone)` — the gather's line is the
template — or read the map's registered ids for the object. The fixture above
becomes the test, red first (`git stash push mtgsim/src`).

**#16 — the triple loop.** Records × candidates × abilities, and each candidate's
layer walk is already outside the loops; but two things inside are paid per
record that are facts per def: the instance ordinal is recounted by scanning the
list prefix, and the identity is rebuilt. **Build `(candidate, identity, def)`
once for every triggered def before the records loop, then two loops.** Bucket by
event kind only if a profile asks — windows are a few records and the sets a
few objects, and the A/B put the whole dispatch at +1.0/+1.5% per decision at
zero triggers. F1's check goes in that same pre-pass.

**#38 — `the_pooled_cards_play_whole_games`.** The fuzz harness is not run by
`cargo test`, but the fork test already plays `performance_pool` boards at two
and four seats and `determinism_test` plays `default_registry`, so the pooled
cards run whole games in CI already. What this test adds is forcing two copies
of each of the three into every deck so triggers fire under the random agent;
what it asserts is weak — `pending_triggers.is_empty() || !g.is_over()` holds
trivially until the game ends. **Keep it and tighten it**: a provider that asserts
the queue is empty at every priority prompt (CR 117.5's claim, the thing the
test's doc comment says it checks), and a name that says so. Or drop it and
lean on the fork test; the owner decides, keep-and-tighten recommended.

**The reading §11 was waiting for — the dispatcher on a trigger-heavy board
(2026-09-20, post-merge).** §11 named one lever and said it would not be built
until a reading asked: a per-source mask of the event kinds its triggers read,
so a window visits only the sources that read it. The reading was taken on the
merged tree with a probe build of `fuzz_games` — `--stuff N` forces N copies
each of Soul Warden, Blood Artist and Wild Growth into every deck (24 of 36
nonland slots at 8), and an `Instant` around `GameState::dispatch` at the outer
depth. The unflagged probe read `IDENTICAL` to the shipped binary on every
counter, so the knob is inert when unset. Raw outputs: the session scratchpad,
`ab2/`.

*The A/B* — `fuzz_ab.py`, performance pool, 200 games, seed 12345, medians of
three interleaved rounds, `--threads 1`:

| copies each | seats | triggers placed / game | CPU / game | turns / game | CPU / turn, p50 | µs / decision |
|---|---|---|---|---|---|---|
| 0 (shipped) | 2 | 1.5 | 7.8 ms | 31 | 0.22 ms | 33 |
| 1 | 2 | 4.8 | 9.1 ms | 32 | 0.23 ms | 38 |
| 4 | 2 | 16.5 | 9.8 ms | 34 | 0.25 ms | 41 |
| 8 | 2 | 47.5 | 15.4 ms | 43 | 0.30 ms | 52 |
| 0 (shipped) | 4 | 4.2 | 27.4 ms | 62 | 0.41 ms | 58 |
| 1 | 4 | 13.5 | 31.5 ms | 65 | 0.44 ms | 65 |
| 4 | 4 | 52.4 | 41.3 ms | 73 | 0.52 ms | 78 |
| 8 | 4 | 149.6 | 63.9 ms | 91 | 0.65 ms | 99 |

*The probe's split* — the same boards, 200 games, `--threads 1`:

| copies each | seats | dispatch ms / game | share of CPU | past the gate / game | candidates / game | matches / game |
|---|---|---|---|---|---|---|
| 0 | 2 | 0.51 | 6% | 243 | 297 | 2.7 |
| 1 | 2 | 1.26 | 13% | 565 | 994 | 8.7 |
| 4 | 2 | 1.69 | 18% | 877 | 3,267 | 26.5 |
| 8 | 2 | 3.10 | 20% | 1,252 | 10,131 | 66.1 |
| 0 | 4 | 1.90 | 7% | 660 | 888 | 6.1 |
| 1 | 4 | 4.25 | 14% | 1,418 | 3,052 | 20.2 |
| 4 | 4 | 7.05 | 18% | 2,102 | 11,358 | 72.2 |
| 8 | 4 | 12.39 | 21% | 2,913 | 32,311 | 183.2 |

*What it says.* On the heaviest board the dispatcher is a fifth of all CPU and
40–50% of the added cost per turn; the rest is the triggers' placement, prompts
and resolutions, and wider boards (frames per walk 13 → 21 at two seats, 21 →
33 at four). The driver is one ratio: a dispatch past the gate visits 8
candidates at two seats and 11 at four, and **fewer than 1% of the visits
match**. The gate asks "is any source present", so once one Soul Warden is out
every batch close builds the ordered battlefield list, looks every source up in
the memo and asks each def whether it reads the record — about 0.3 µs a visit,
10,000 visits a game at two seats and 32,000 at four. On the shipped pool 27%
of batches already pass the gate for 1.2 candidates, which is the 6–7%.
Healthy: memo hits scale ×2.6 while layer walks rise 64%, and board walks per
gather move 0.22 → 0.27, so the dispatch forces no walk the state-based check
would not have paid; deterministic under three hasher seeds; no errors, no
turn-limit hits. Heavy in count, not in kind — three trigger shapes; a
Commander table's cast, attack, end-step and counter watchers raise the mask's
payoff, not the per-visit cost.

*Action, in this theme's PR.* `trigger_sources: IdSet<ObjectId>` becomes a map
from source to the mask of event kinds its printed triggered defs read
(`TriggerEvent::reads` is the per-record test; the mask is its per-kind union,
written where the set is written today). The window's kinds are OR-ed once; a
source is a candidate only if the masks intersect; a dispatch whose window no
source reads returns at the gate, before the battlefield list is built.
Over-approximate in one direction only: the granted and copied legs and the
departure frames keep their whole-list walk. About 60 lines, in the same
pre-pass as the flatten. **Gate:** `fuzz_ab.py` engine vs `main` `IDENTICAL` on
every counter — a mask skips only a visit that would not have matched — and
the probe re-run on the eight-copy boards, with the sitting above and the
after-arm recorded as a post-merge reading in `fuzz-record.md`. If the
trigger-heavy fixture is to be repeatable, `--require` gains a `--copies N`
companion in the same PR (the probe's `--stuff` hard-codes the three names);
the owner's call. Expected: most of the dispatcher's share on the eight-copy
board and nearly all of it on the shipped pool — an estimate until the arm runs.

Gate: the fixture red then green; `fuzz_ab.py` engine vs `main` `IDENTICAL`;
the probe re-run and the record.

---

## D — the docs, and three decisions · *no code; TR-2's brief reads this*

### D.1 Item 167, restated (#2) — *decision: snapshot now, or TR-4's frame*

The item as filed sizes the fix as "when the outermost batch is a battlefield
departure and `trigger_sources` is non-empty, snapshot each source's look-back
defs". Too narrow on two axes:

- **Sources.** The surviving source can be off the battlefield. Bridge from Below
  in a graveyard under Yixlid Jailer, and one wipe takes Jailer and an opponent's
  creature: before the event Bridge had no abilities and should not trigger;
  after it the engine reads its restored list and does. The snapshot must cover
  the zone map's sources, not only the battlefield set. And the sign runs both
  ways: a look-back ability *granted* by a row whose source leaves in the same
  batch existed before the event and not after, and the engine misses that one.
- **Events.** CR 603.10a has three classes; TR-4 adds cards leaving a graveyard
  and visible cards going to hand or library, so "a battlefield departure" is
  today's one class, not the rule.

What stays true: the pre-event and post-event lists differ only when a registry
row (Layer 1, 3 or 6) arrived or left in the same batch, so the cheap trigger
for the snapshot is **"the batch departs a permanent that carries a registry
row"**, decided at the batch's open against the registry, on every board
without such a row a probe and nothing else.

**The two designs, for the owner:** (a) snapshot at the batch's open — for every
source in both sets, an `Arc` clone of its look-back defs when the registry
holds a row that can reach a departing permanent, matched at the close in place
of the live list; ~80 lines, two fixtures (Humility beside Blood Artist in one
wipe; Bridge under Jailer), an A/B because it adds work at the open on boards
with rows and Humility is pooled. (b) TR-4's `LastKnownInformation` grows a
per-window frame for every source the batch touched, and 167 waits for it.
**Recommendation: (a), in theme E** — Humility and Blood Artist are both pooled,
so the item is reachable-wrong today, and (b) makes the frame carry a list for
objects that did not move, which is not what a frame is.

Paste the two axes and the trigger into item 167's **Sized** paragraph; leave
its reachability line as is.

### D.2 The instance ordinal (#23) — *decision: provenance ids, before TR-2's gates*

**Why the field exists.** A4g derives a printed ability's id from the card name
and its ordinal, so every Diffusion Sliver's granted def has the same id on
every Sliver, and two Diffusion Slivers put the same def on each Sliver twice
under one id; the rulings say the abilities are cumulative, so both instances
trigger. In TR-1 the ordinal is load-bearing in exactly one place — the "one or
more" fold keys on `(identity, arm)`, and without it two instances of such a
trigger fold into one — and it keys the elision and the trace record.

**Why it reads as a hack, and where that bites.** An ordinal among same-id defs
is stable under a later grant but not under an earlier grant *ending*: if the
first Diffusion Sliver leaves mid-turn, the survivor's instance 1 becomes
instance 0, and a TR-2 once-per-turn gate keyed on instance 1 is orphaned while
instance 0's gate reads as unused.

**The alternative: provenance.** Mint the granted def's id from the granting
row's source and epoch at the Layer 6 grant site (`compute.rs:953`, where
`is_characteristic_defining` is already cleared — the id module has a
per-object role, `AbilityId::derived_on`, to build beside). Two grants carry two
ids with no ordinal, stable while the grant exists and gone with it. Costs: that
site, an id constructor, the elision re-keyed on def equality instead of id
equality (`placement.rs:99`), and item 149's closure amended. "Loses all
abilities" is by list, so CR 113.10b is unaffected. Diffusion Sliver itself
waits for TR-5's target event, so the field has no printed consumer yet.
**Recommendation: provenance, decided in this theme and built in E, before TR-2
keys a gate on the identity.**

### D.3 The nesting bound (#8) — *doc amendment now, code in TR-6*

"No printed ability watches its own kind" is necessary and not sufficient once
custom cards exist. What the bound guards is a trigger watching abilities
triggering whose own triggering it then watches; that recursion is synchronous
inside one dispatch, so TR-6's loop detector (which counts decisions) cannot see
it, and this bound is the only guard for that loop. **The correct answer at the
bound is CR 104.4b's draw, not an `Err`.** Amend §4.8: TR-6 settles `Draw`
through the same settlement the loop detector uses, and the constant becomes
the detector's threshold knob (§4.9's, default 15) rather than a second number.
`BATCH_NESTING_LIMIT` (32, `actions.rs:38`) is the same pattern but a genuine
engine invariant — CR 614.5 bounds every replacement chain — so it stays an
`Err`.

### D.4 Sizing (#3) — *what the rows got wrong, and what to change*

| Row | Sized | Landed | |
|---|---|---|---|
| the dispatcher | ~520 | ~970 (`dispatch.rs` 754, `binding.rs` 73, `zone_function` 31, summary 31, `game_state` 85) | 1.9× — the one code row that missed |
| types, placement, resolution, cards | ~1,160 | ~1,210 | on |
| tests | ~700 | 1,871 (50 tests, ~36 lines each) | **§13 owes TR-1 46 atoms** — half the CR 603 corpus — plus seven rulings and the named fixtures; the row counted none of that |
| docs and ledger | ~250 | 940 (trace page 203, archive 302, codebase-state 183, fuzz-record 98, ledger 38) | 3.8× — and never in the band in practice |
| **whole** | ~2,750 (rows; the heading said 2,300) | 5,166 | |

The three prior rules phases ran 3,000–3,600 whole and 1,800–2,500 on code plus
tests (RE-8 1,254 + 971 + 832; RE-9 954 + 843 + 1,212; A4i 1,573 + 890 +
1,178), so **the band has only ever described code plus tests**; TR-1 is over it
either way at 4,226. The later rows, read against §13's counts (TR-2 owes 8
atoms, TR-3 20, TR-4 14, TR-5 4, TR-6 2) and their own named ruling tests, at
36 lines a test:

| Phase | code rows | with the largest row ×1.9 | tests, sized | tests, re-counted | code + tests |
|---|---|---|---|---|---|
| TR-2 | 1,100 | 1,550 | 700 | ~900 (8 atoms; Paragon's six, Trickster's three, Warmaster, Ashling, 603.1b, 121.2c) | 2,000–2,450 |
| TR-3 | 960 | 1,430 | 650 | **~1,100** (20 atoms; Heart-Piercer's four, Tatsumasa, Sneak Attack, 513.2 both ways) | 2,060–2,530 |
| TR-4 | 900 | 1,330 | 700 | **~1,100** (14 atoms; Kitchen Finks' eight, Wargear's three, Guile's two, 122.8/9) | 2,000–2,430 |
| TR-5 | 1,000 | 1,500 | 750 | ~900 (4 atoms; 509.3's seven, Panharmonicon's ten, Frost Titan, 508.4) | 1,900–2,400 |
| TR-6 | 420 | 570 | 450 | ~300 | 720–870 |

**So: no re-plan.** Three edits to make before TR-2's brief:

1. `engineering-practices.md` §4: one sentence saying the band counts code plus
   tests, with docs reported beside it — that is what the earlier phases were
   held to, and the docs figure is a function of the trace page and the
   eviction rule, not of the design.
2. §12: TR-3's and TR-4's test rows to ~1,100; every test row states its atom
   count and its ruling tests, which is the number the flat 700 missed.
3. §12: a **candidate seam** named for the two phases whose largest row could
   put them past 2,500 on code plus tests — TR-3: the registry with Final
   Fortune and Flickerwisp, then "until" with Banishing Light; TR-5: the combat
   shapes and targeting, then counters, prevention and the multiplier. Used only
   if the brief's re-count says so.

### D.5 Lines for other docs

- **#21 `CastFacts`** — `tmnt.txt` 400.7d is the rule ("an ability of a permanent
  can reference information about the spell that became that permanent as it
  resolved, including what costs were paid to cast that spell or what mana was
  spent"); the letter moved across CR versions. Main item 9's landed paragraph
  gains: the struct is 400.7d's home, built at one site
  (`game_state.rs:1209`) off the resolving object, and `x_value`, kicked and
  mana spent join it when a card reads them, each one constructor site plus the
  carry off the `StackEntry`, whose alternative cost, additional costs and X
  already exist.
- **#31 sharing with `EventPattern`** — §3.3 gains: they watch different streams
  (a proposal before it happens, rewritable by the closed `Rewrite` algebra; a
  performed record after, with frames, look-back and occurrences); ten of the
  fourteen arms have a sibling in shape, and the vocabulary is shared at the leaf
  (`ObjectFilter`, `PlayerRef`, `Condition`, `ZoneChangeCause`). What is real is
  shared *field structs* — a `ZoneChangePattern { from, to, cause }` and
  `SourcePattern` for damage — and TR-4's widening of `ZoneChange` is the moment.
  Never a shared enum. Layers watch nothing.
- **#35 `StackWatcher`** — a line in TR-2's row: it moves to `test_support` in
  TR-2's first commit (five uses in one file today; every trigger phase claims
  "before priority").
- **#37 the elision and Tireless Provisioner** — a line in TR-2's row, beside its
  binding reader: the elision compares only the bound facts the effect *reads*
  (walk it for the three `Triggering*` leaves), so two landfall triggers from two
  lands stop prompting when the effect ignores the land. Provisioner itself is
  correct today: its Food-or-Treasure choice is CR 608.2d's, made as the effect
  applies (no bulleted modes, CR 700.2), so two identical stack objects give the
  same game in either order.
- **#10 `visible_to_all`** — no action; the pointer is right. Scheduled as
  `roadmap-v2.md` row B4 (1–2 PRs, hard back-stop before Phase 8's face-down and
  reveal cards and before Phase 10); the owner's pass-4 decision (2026-09-15)
  was that the triggers doc consumes `Zone::is_public()` and does not pull §2.9
  forward. The predicate is exact today — the only hidden state on the board is
  `face_down` — and the CR's own counterexample board needs Future Sight and
  Telepathy, which the model does not yet express.
- **The keeps**, one line each in the TR-1 archive's notes so they are not
  re-asked: **#9** a modal trigger with one mana mode is a mana ability by CR
  605.1b's "could add mana" (605.2 keeps the class when the state cannot
  produce it; the class must be knowable before modes are chosen because a mana
  ability never reaches the stack) — modal triggers are `backlog.md` §2.7's and
  the mana resolver refuses a `Modal` root with an `Err`, so nothing misresolves
  quietly; **#15** the timestamp sort has no determinism problem — every object
  in the store carries a unique timestamp from one monotonic counter, so the
  keys never tie; **#18** 603.3d's equivalence holds (F2's rewrite says why);
  **#25** the two source sets mirror the replacement pair for its stated reason
  (leg 1 probes a set while walking the ordered battlefield, leg 3 iterates a map
  whose value is the functioning-ability list — the value F1 says the dispatcher
  never read).

---

## E — the decided engine work before TR-2

Whatever D decides: **provenance ids** (~60 lines: the id constructor, the Layer
6 grant site, the elision's key, item 149's closure; a test with two grants of
one def under one controller, the first ending mid-turn) and **item 167's
snapshot** (~80 lines, the two fixtures in D.1, item 167 closed by the eviction
rule). One PR each or one PR both, sized in D. Gate: `fuzz_ab.py` three arms
with the cost recorded in `fuzz-record.md` — the snapshot's probe runs on every
board with a registry row, and the pools have Humility.

---

## Where each theme lands when this file goes

A → the code, the glossary, F2's two comments. B → the module and one line in
the TR-1 archive's notes. C → the fixture and a tenth note in the archive (or a
`codebase-state.md` item, if C slipped). D → the docs it names. E → the code,
the `fuzz-record.md` block, item 167 closed, item 149's closure amended. The
deleting PR redirects the pointers: `codebase-state.md`'s "Found by TR-1"
heading, PR #173's body, `roadmap-v2.md` row A6's landed note.

---

## Index — every comment, its theme

| # | Site | One line | Theme |
|---|---|---|---|
| 1 | `fuzz-record.md:72` | "counters" vs `Diagnostics` — glossary line; printed rows stay | A |
| 2 | `codebase-state.md` item 167 | too narrow: zone-map sources, TR-4's two classes, both signs | D.1 → E |
| 3 | size | 5,166 whole; no re-plan; three edits before TR-2's brief | D.4 |
| 4 | `EachOther` | → `NotSource` | A |
| 5 | `dies` abstraction | the authoring module | B |
| 6, 7 | `None` for "any" | constructors take the word; `Option` stays at the type | B |
| 8 | `DISPATCH_NESTING_LIMIT` | CR 104.4b's draw at the bound, the detector's knob; TR-6 | D.3 |
| 9 | `could_add_mana` on `Modal` | correct by 605.1b's "could"; keep | D.5 |
| 10 | `visible_to_all` | scheduled, row B4; exact today | D.5 |
| 11 | `Candidate` | → `TriggerCandidate` | A |
| 12 | `Match` | → `MatchedTrigger` | A |
| 13 | `dispatch_inner` | the five steps become its doc comment | A |
| 14 | `u64` timestamp | `Timestamp` alias, and one for the epoch | A |
| 15 | `sort_unstable_by_key` | no problem: unique keys | D.5 |
| 16 | the triple loop | flatten once before the records loop | C |
| 17 | `arm_occurrences` | the paragraph becomes its doc comment | A |
| 18 | 603.3d equivalence | holds; the comment was wrong about the code (F2) | A |
| 19 | `retain` | order matters; keep | A (keep) |
| 20 | three wrappers | keep all three | A (keep) |
| 21 | 400.7d, `CastFacts` | rule text is right; growth is one site per fact | D.5 |
| 22 | two trigger zone sets | fold to one | A |
| 23 | `instance` | provenance ids, before TR-2's gates | D.2 → E |
| 24 | `dispatch_depth` | stays on the state; fold the three guards | A |
| 25 | two source sets | mirror the replacement pair; the map's value is F1's | D.5 |
| 26 | `TriggerCondition` / `Condition` | keep; glossary line | A |
| 27 | `arms()` | → `events()`, `EventIndex` | A |
| 28 | `Tier` | → `TriggerTier` | A |
| 29 | `Occurrence` | → `Multiplicity` | A |
| 30 | `Subject` | → `TriggerSubject` | A |
| 31 | `TriggerEvent` vs `EventPattern` | shared field structs at TR-4; never a shared enum | D.5 |
| 32 | `TriggerSeq` | keep; pairs with `EventSeq` | A (keep) |
| 33 | `ObjectRef` | to `types/ids.rs`; reused in `AbilityIdentity` | A |
| 34 | `PendingTrigger` | three dead fields | A |
| 35 | `StackWatcher` | to `test_support` at TR-2's first commit | D.5 |
| 36 | `dies` on a land | correct under `tmnt.txt` 700.4; one constructor | B |
| 37 | elision vs Provisioner | correct; the binding-read refinement at TR-2 | D.5 |
| 38 | whole-games test | keep and tighten the assertion | C |
| F1 | `find_matches` | no CR 113.6 check per def off the battlefield — proved | C |
| F2 | `place_one`'s comment | says "never creates"; the code creates and removes | A |
| — | the dispatcher on a trigger-heavy board | §11's kind mask; the reading taken 2026-09-20, a fifth of CPU at eight sources | C |
