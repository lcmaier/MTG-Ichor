# layers-architecture.md — landed phases, evicted

**A record of finished work, not a plan.** Every section here sat under a ✅
heading in `plans/layers-architecture.md` and was moved out on 2026-09-11 once the phase had shipped,
leaving the heading, a stub and a pointer in the live doc. Nothing here is
owed and nothing here should be acted on; what each section is *for* is the
reasoning — the design as sized, what the building changed, the measurement.
`check_state_of_play.py` reads the ✅ headings that stay, and fails when a
landed section keeps more than 40 lines in the live doc
(`engineering-practices.md` §4). Later phases are appended by the PR that
lands them.

### LH-1 — the host becomes addressable (~730 additions) — ✅ 2026-09-04

*Evicted 2026-09-11 from `plans/layers-architecture.md`, where the heading and a stub remain.*


1. **`AffectedSet::Host`** plus its `EffectRecipient` lowering. One
   arm in `compute::effect_applies_to`
   (`game.battlefield.get(&effect.source).and_then(|e| e.attached_to) == Some(id)`),
   one in `static_affected_set`. **Resolved during the walk, never snapshotted**
   — the same correction §3.4 records for `ByController`, so registration
   running before the attach is not a problem.
2. **CR 608.3b, as one shared recipient helper.** `mana_helpers::spell_recipient`,
   the inline block in `cast.rs`, and `stack::extract_recipient` are three copies
   of the same fourteen lines deriving a recipient from a spell's *effect*, so
   none can see an `enchant_filter`. The helper takes the object. This closes
   `codebase-state.md` Deferred Migrations item 8.
3. **Holy Strength** — `{W}` Aura, "Enchant creature / Enchanted creature gets
   +1/+2". One Layer 7c row against the new affected set.

**Item 8 rides here rather than shipping alone, and that was measured.** Zero
registered cards carry an `enchant_filter`, so the shared helper returns exactly
what the three copies return today for every card that exists — **a fix nothing
in a fuzz game can reach**. Shipping it alone would put a new arm in front of
the performance pool that no card can open, which is the failure §3 of
`engineering-practices.md` describes. It ships with the card that makes it live.

**As built (2026-09-04, `layers/lh-1-host-addressable`).** Three departures
from the list above, each smaller than what it replaces:

1. The helper is `engine/targeting.rs::spell_recipient(&CardData)`, and the
   resolution does not call it. `StackEntry` records the recipient the targets
   were chosen against at CR 601.2c, so CR 608.2b re-checks the *same* question
   rather than a second derivation that has to agree — and the resolution stops
   reading printed abilities off an object on the stack. The fourteen lines
   survive once, as `effect_recipient`, shared with `activate_ability`, which
   had a fourth, `Atom`-only copy.
2. The epoch bumps 7g placed in LH-2 belong here, since this is the PR that
   makes the walk read `attached_to`. Rather than six bumps there is one writer:
   `GameState::attach` / `detach` write both sides of the link and bump, and
   every former site goes through them. `attach` refuses a host that is not on
   the battlefield (CR 303.4i sends such an Aura elsewhere), and a reattachment
   detaches first, so the old host's back-pointer never survives.
3. `attach_aura_on_etb` was deleted rather than made the path. It implemented
   CR 303.4f's choose-on-entry for a path no card can take, and it was the
   second attach writer. Whatever first returns an Aura to the battlefield
   brings 303.4f/g back with its consumer.
4. The variant is `Host`, not the `AttachedToSource` this section first named
   (review, 2026-09-04): on a card the source is the card itself, so "attached
   to source" read as "the things attached to me", which is the wrong
   direction. `EffectRecipient::Host` lowers to `AffectedSet::Host`, and "host"
   is what every attach site already called it.

Measured, 200 stress games at seed 12345: CR 704.5m/n's `[AuraSba]` **0 → 23**
(60 under `--require "Holy Strength"`); Holy Strength resolves 96 times
unforced and in 134–135 of 200 games forced, both pools. The CR 608.3b fizzle
is covered and **reached 0 times** — the random agent does not kill an Aura's
target in response; 3 Counterspells is the closest it came. Three serial runs
at one seed are byte-identical outside `=== Timing ===` on both pools, and the
three-arm A/B (`engineering-practices.md` §3) puts the engine's share at zero:
the pool-unchanged arm is byte-identical to `main` on `performance`. One more
composition fell out of the pool for free: Mirrorform copying a creature onto
an attached Holy Strength makes it a non-Aura permanent that is attached to
something, and the CR 704.5n catch-all unattaches it — four times in 200 games.

### LH-2 — CR 613.7e, and the field split (~900 additions) — ✅ 2026-09-05

*Evicted 2026-09-11 from `plans/layers-architecture.md`, where the heading and a stub remain.*


1. Split `PermanentState.timestamp` per the table above; update the four
   readers, each deliberately.
2. Reassign the CR 613.7 timestamp at the attach site, from `next_timestamp`.
3. Restate the contract in `CLAUDE.md` and in `battlefield_ordered`'s docs:
   determinism keys off the *entry* timestamp, which is still allocated once.
4. **Consumer: Equip (CR 702.6).** Every PR in a split carries a consumer of what
   it builds (§4), and LH-2's is a reattachment path. Equip is the right one:
   `"[Cost]: Attach this Equipment to target creature you control"` is a
   **single** target, where Aura Finesse is two and the pool has no multi-target
   spell. It also closes CR 704.5p — Equipment detach, one of the six paths the
   2026-09-01 fuzz re-audit measured at zero, and unreachable today because
   nothing can attach an Equipment.

**This is the piece to challenge first.** Equip brings an activated ability with
a timing restriction and a new `Primitive::Attach` (only `KeywordAction::Attach`
exists today). If that reads as too much for one PR, the alternative is
multi-target plus Aura Finesse — a different subsystem, not a smaller one.
**Settled 2026-09-04:** that alternative is `backlog.md` §2.20 now, a medium
PR of its own with a back-stop before RS-2 and item 6 (`roadmap-v2.md` §3a),
and LH-2 does not absorb it. Equip is the consumer. The timing restriction is
the one piece to keep minimal: CR 702.6a's "only as a sorcery" is one
activation-restriction value on `AbilityDef`, not `backlog.md` §2.8 wholesale.
CR 613.7e's reassignment lives in `GameState::attach` (LH-1's one writer), and
it closes the timestamp half of "Before Layers" 7g.

**As built (2026-09-05, `layers/lh-2-timestamp-split`; shape and card settled
in review 2026-09-06).** ~830 additions before the docs; the ~900 estimate held.

1. **Where a row's timestamp is read — shape (b): `attach` re-stamps the rows.**
   `register_static_effects` runs inside `place_on_battlefield`, before any
   attach, so once CR 613.7e exists a registered row's stored timestamp is
   stale the moment its source attaches. Two shapes were weighed: (a) the walk
   reads the source's CR 613.7 timestamp at comparison time; (b) `attach`
   re-stamps the source's `StaticAbility` rows in place — "Before Layers" 7g's
   part (2). **(a) shipped first and was replaced in review.** Its number was
   fine — in one sitting the split alone measured −0.6%, a naive prototype of
   (a) +1.8% and the shipped (a) +2.5% per walk, inside the spread — so the
   number did not decide it. What did: under (a) `ContinuousEffect.timestamp`
   means two things by origin, exact for a resolution row and a lower bound
   for a static one, and the registry's "already in application order" stops
   being literally true — a trap for item 7's board-wide pass and for anyone
   reading a row. The "don't reconcile the registry at chokepoints" rule (a)
   leaned on is about *existence*, which depends on the walk's own output
   layer by layer; a timestamp is a discrete write with one writer, and
   CR 613.7a's third sentence describes (b) in so many words. **Built:**
   `ContinuousEffectRegistry::retime_static_rows(source, ts)`, called by
   `attach` after the entity's write, edits the source's static rows in place
   through `DurationRegistry::update_rows` — a stable re-sort on
   `(layer, timestamp, id)`, so ids survive and rows sharing the new timestamp
   keep registration order, which is 613.7a's "relative order remains the
   same". Resolution rows (613.7b) keep the timestamp of the effect that
   created them. The walk is `main`'s again, byte for byte. Measured in one
   sitting on `performance` (§3's table has the four arms): the clean engine
   +0.5% CPU/game and per 1,000 walks against `main` with an identical stream
   — round 2 had it faster than `main` — and the shipped tree +3.4% per 1,000
   walks against its `main`, where the same board under (a) had read +9.5%:
   an attached Equipment's rows are no longer re-sorted on every frame.
2. **The `len` heuristic went with it.** `ContinuousEffectRegistry::mutating`
   told a write from a no-op by comparing `len` before and after — which an
   in-place edit cannot move, and a closure that added and removed in one
   call would fool. `DurationRegistry` now keeps a generation counter that
   every add, remove, retain-that-removed and `update_rows`-that-changed
   bumps, and `mutations()` *is* that counter, the layer memo's other half. A
   CR 514.2 pass that removes nothing still costs no frame.
3. **The field split was undone, and the reader table was wrong by one.**
   Three production readers, not four; corrected above. The split into an
   `entry_timestamp` shipped in the first commit and review asked why it was
   needed at all; it was not (see "Reversed in review" above), so all three
   readers key on the one `timestamp` again, and `tests/determinism_test.rs`
   pins that a reattachment moves the attachment to the end of the sweeps and
   nothing else. The entity's `timestamp` is read at registration and by the
   sweeps, and by nothing in the walk.
4. **CR 613.7e applies to Auras as well.** Holy Strength's resolution attach
   goes through the same `attach`, so an Aura's rows are re-stamped at its
   attach from LH-2 on. Unobservable today — Layer 7c commutes — and correct.
   Attaching to the host it is already on is CR 701.3b's "does nothing": no
   write, no event, no new timestamp (701.3c says "a different object").
5. **Equip's shape.** `phase_lh_cards::equip(costs: Vec<Cost>)` builds
   `Primitive::Attach` behind
   `EffectRecipient::Target(Permanent(creature ∧ you control), Exactly(1))`,
   so CR 608.2b's re-check is CR 301.5b's "control matters when it resolves"
   and CR 701.3b's "doesn't move" with no second check. `costs` is the whole
   "[Cost]": a `u8` generic count shipped first and review widened it —
   Improvised Arsenal's equip is `{R}`, 33 printed Equipment mix generic and
   colored pips, and "Equip—[cost]" is any cost (Dark Knight's Greatsword,
   "Pay 3 life"; its once-per-turn clause is `backlog.md` §2.8, Phyrexian pips
   §2.18). It proposes `GameAction::Attach` — the proposal `codebase-state.md`
   item 6 said the pair would want — performed through `GameState::attach`,
   still the one writer, and announced as `GameEvent::Attached` on the
   transition only. `ResolutionContext::ability_source` carries the ability's
   permanent (CR 113.7a); `source` was the ephemeral stack object, which
   nothing could attach. "Activate only as a sorcery" is
   `ActivationRestriction::OnlyAsSorcery`, one value on `AbilityDef`
   (`backlog.md` §2.8 grows the enum, and names the two look-alikes that are
   not it), kept out of the window by `activatable_abilities` and refused by
   `activate_ability`; the sorcery-speed rule itself is now one function the
   cast path shares.
6. **A bug the consumer found.** `activate_ability` paid every ability with
   an empty generic-mana allocation, so any ability with a generic pip failed
   at payment and was blacklisted: Chainbreaker's `{3}, {T}` was activated 97
   times and resolved 0 in 40 `main` games, 92 and 85 after. Fixed by sharing
   the cast path's allocation step. Chainbreaker is pooled, so the protocol's
   "engine, pool unchanged" arm is not `main`'s stream; a fourth arm with the
   fix reverted and the new cards unregistered is, apart from a memo-hit count
   that moved by four per 200 games (the new `can_pay_costs` pre-check asks
   cached questions). `engineering-practices.md` §3 records the four arms.
7. **Cobbled Wings, not a fixture.** The CR 613.7e pin needs a Layer 6 grant
   on an Equipment. An invented "Skyhook Harness" shipped first; review
   pointed at the printed card with exactly that text — Cobbled Wings, `{2}`,
   "Equipped creature has flying. Equip {1}". Registered, since the engine
   plays it, and not pooled, since it opens no path Bonesplitter does not. It
   puts the reorder within a `stress` game's reach: Cobbled Wings is equipped
   in a game Humility also enters in 14 of 200.

Measured, 200 games at seed 12345 on the final tree: Bonesplitter cast 247 /
resolved 243 in 140 of 200 `performance` games, Cobbled Wings 239 / 238 in
146 of 200 `stress` games under `--require`; `Attached` 516 on `performance`
(381 of them re-equips) and 398 + 458 on `stress` (269 + 322); CR 704.5p
**0 → 15** per 200 `stress` games on Equipment subjects — 7 Bonesplitter, 8
Cobbled Wings, Mirrorform copying a non-creature onto the equipped creature —
beside 5 catch-all detaches of Mirrorform'd Holy Strengths, and once on
`performance`; three serial runs per pool byte-identical outside
`=== Timing ===`, dumps identical after the id masks, with the re-equips in
them. `specdb owed --phase LH` is clean;
ATOM-613.7e-001 is claimed partial (one reattachment against Humility, not
the atom's away-and-back against an Aura).

### LI-1 — the board-wide sequential pass (~1,250 additions) — ✅ 2026-09-06

*Evicted 2026-09-11 from `plans/layers-architecture.md`, where the heading and a stub remain.*


1. **`Board`** in `engine/layers/board.rs`: the working set in its order, the
   live frames, the CR 613.6 locked sets, and the look-ahead reference
   `FrameCache` carries today (the accessor pair moves with it — `entity` and
   the rows read stay the two places the walk touches concrete state).
2. **`applications_in_layer`**: the four kinds of decision 1, built once per
   layer per pass, sorted on the one key.
3. **`apply_layer`**: for each application in order — existence against the
   live frame; the affected members, from the locked set when the group has
   started (CR 613.6) and from the filter over W otherwise; then the
   modification, its dynamic parts resolved *before* any member is mutated
   (`CountOf` must be able to read the member being modified — a creature
   counting "creatures you control" counts itself). LI-2 replaces "in order"
   with 613.8's loop and nothing else here moves.
4. **The existence check** reads the live frame, which is the whole of item
   8 step 4 and closes 7b's Layer 6 case: `register_granted_static_effects`'s
   assert becomes `layer >= Layer6Ability`, with a test granting a Host-scoped
   "equipped creature has flying" static to an Equipment by resolution.
5. **`compute_characteristics`**: a miss for a member runs the pass and fills
   the memo for every member; a miss for a non-member runs the CDA walk. The
   debug audit compares a hit against a throwaway pass, as before.
   `compute_characteristics_uncached` (the CR 603.10a capture) and
   `compute_as_entering` are throwaway passes too.
6. **`FrameCache` survives** as the non-member walk's `(id, ceiling)` memo,
   with a reference to the live board when the walk is nested inside a pass.
7. **The cost rows.** `Layer walks` keeps its meaning — a top-level miss —
   and a new bold row, `Board walks`, says how many of them walked the whole
   board; `Layer frames` becomes board walks × members plus the non-member
   frames, so `Frames/walk` will read near the member count rather than 1.55.
   §3 says so.
8. **§5.2 rewritten** per decision 4; §5's pseudo-code and §9's interface note
   updated; `codebase-state.md` 7b's Layer 6 case and 7c's untested claim
   closed, item 8 marked "step 4 built, steps 1–3 LI-2".

**Consumer and red test.** Humility then Citanul Hierophants, both pooled:
`test_humility_before_hierophants_does_not_yet_retire_the_grant` is red
against the pass by construction (it asserts the wrong answer) and is
rewritten to assert the CR's — the Bears have no mana ability in either order.
Because both cards are pooled, the A/B's "engine, pool unchanged" arm will
legitimately differ from `main`: every game in which a creature under
Humility tapped for the Hierophants' {G} takes a different path. §3 counts
those games and says why, as LH-2 did for Chainbreaker.

**What the A/B should show, and the levers if it does not.** A pass builds
one frame per member; a walk today builds 1.55. After a write, the SBA
sweep asks six questions of every permanent, so today it is ~40 walks at 1.55
frames and under the pass one pass at ~40 frames — cheaper, and it is the
dominant case (96% of questions repeat an untouched board). Two cases cost
more: an epoch in which only one or two objects are asked before the next
write, and the look-ahead, which today walks one object with sub-frames and
under the pass builds every member — ~45 entries a game, so ~1,800 frames
against a 3,755 total today. Expected: frames up, walks down, CPU per game
flat to +10%. If it is worse, both levers are answer-preserving and each has
its proof already written: **replay** the real pass's application order
against the entering object alone for the look-ahead (§5b's asymmetry says
the other members' frames are the real ones), and **project** a layer whose
applications are pairwise statically independent — no application writes a
channel another reads, which is LI-2's static check computed from the rows'
kinds alone — through the per-object walk, which is exact for exactly that
layer. Neither is LI-1, and a finer memo key is neither (§12 "7a").

**As built (2026-09-06, `layers/li-1-board-pass`).** +1,013 / −775 in
`src` across nine files and +230 in tests before the docs: the ~685 lines the
plan counted came out as `board.rs` (new, ~560 lines) and a `compute.rs` that
keeps the entry, the memo, the non-member walk and the evaluators. The ~1,250
estimate held for the code; the docs put the PR near 1,700. Four departures
from the pieces above, each found by a test:

1. **An object in the battlefield zone with no entity is a member of the
   pass that asks about it, and of no other.** The plan said such an object
   — a token whose entry is being decided — is asked about only through its
   look-ahead. It is not: the CR 614.17 restriction predicate
   (`keyword_prohibits`) asks its keywords through the plain entry while the
   entry is decided, and passes for *other* objects run meanwhile with no
   look-ahead at all. A debug assertion that demanded the look-ahead failed
   seven token tests, and the assertion was wrong. `board::classify` names
   the three ways the entry answers — `Member`, `ZoneOnly`, `NonMember` —
   and a `ZoneOnly` object joins its own pass as `asked`, which reproduces
   its old frame exactly (the zone gate admits filter rows; the controller
   seeds from the owner). It is invisible to counts and changes no other
   member's frame, so two of them never need an order between them.
2. **`copiable_values` wanted a ceiling.** CR 613.2c's capture is the frame
   at the end of layer 1, which the old `compute_to_ceiling` gave and a full
   pass does not. `compute_board_to(.., ceiling)` stops a pass where it is
   told; `frame_at_ceiling` routes a member through that and anything else
   through its own walk.
3. **`Layer walks` still counts a query for an object that does not exist**,
   as it did: the count sits before the store probe, so that case's fixture
   rows stay comparable to `main`'s.
4. **The `Board walks` row.** Walks per game fell 2,425 → 328 on
   `performance` at 50 games (3,129 → 366 per 200) and 236 of the 328 walked
   the whole board; frames rose 3,755 → 4,289 (+14%; +21% per 200 games),
   which is board walks times members. `Frames/walk` reads 13.1 where it
   read 1.55, and the row is what makes that legible.

Two things the plan predicted and the sitting confirmed. **The A/B is
flat**: `plans/fuzz_ab.py`, one sitting, `main` at 650633f against the
branch — CPU/game 14.56 → 14.81 ms (+1.7%), ms per 1,000 questions
0.146 → 0.149 (+2.1%), both inside the sitting's spread; p99 48.3 → 46.6 ms.
**And the "engine, pool unchanged" arm legitimately differs from `main`**,
because both consumers are pooled: on 40-game `--dump-events` streams with
the id masks applied, **one game in forty differs on each pool** —
`performance` game 38, `stress` game 20 — and in both, Humility and Citanul
Hierophants had entered before the divergence; the first divergent event is
a choice made by index (a mana source tapped for a payment, a blocker) from
a list in which a creature under Humility no longer offers the Hierophants'
ability. Every other game is byte-identical. Three serial runs per pool are
identical outside `=== Timing ===`, and the sitting's rounds reproduce the
threaded counters. `engineering-practices.md` §3 has the tables.

The first `main` binary the sitting was run against was stale — built the
day before #102's last commits — and every game differed from its first
draw; a `git worktree` synced to a commit is not a binary built at it, and
the attribution script now says so on its first line. Tests: the pinned
wrong answer is flipped in place
(`test_humility_before_hierophants_retires_the_grant`), and
`tests/phase_li_integration_test.rs` pins 7b's Layer 6 case built by
resolution, a +1/+1 counter interleaving with a power-reading 7c row on
CR 613.7c's clock (the one order that stays stable under LI-2), a graveyard
Keldon Warlord counting the board through the settled path, and one pass
answering every member with the nested graveyard reads not counted as
board walks. No card and no pool change: the consumer was already in both
pools.

**Scaling, measured before the merge (2026-09-06).** §12's synthetic board —
N anthem creatures ("creatures you control get +1/+1", one 7c row each)
beside N vanilla creatures — run on one machine against `main` at 650633f
and this branch, release, µs per epoch. Two columns per arm: every member
asked once after a write (what an SBA sweep does), and one member asked.

| board (creatures + anthems) | rows | `main`, all asked | pass, all asked | `main`, one asked | pass, one asked |
|---|---:|---:|---:|---:|---:|
| 10 + 1 | 1 | 15.0 | **6.4** | **1.3** | 6.2 |
| 10 + 10 | 10 | 180.0 | **33.2** | **8.7** | 27.4 |
| 40 + 5 | 5 | 191.8 | **42.2** | **4.5** | 36.6 |
| 40 + 40 | 40 | 2,978 | **315.5** | **41.3** | 323.5 |
| 80 + 5 | 5 | 414.0 | **77.2** | **4.2** | 75.8 |
| 80 + 80 | 80 | 13,476 | **1,195** | **107.2** | 1,182 |

The pass is 2–11× cheaper whenever the board is asked about after a write,
and the gap *widens* with rows: the old walk's CR 604.2 check was a sub-walk
of the source per row per query (§12 called it superlinear), and the pass
reads a live frame instead. What the pass pays is the single-object query
between writes — the whole board for one answer, 5–10× the old cost on a
row-heavy board and bounded by members × rows. The fuzz mix nets to +1.7%
because the first case is what games do (96% of questions repeat an
unchanged board, and the sweep after a write asks about everything).
The levers if the second case ever dominates are all answer-preserving and
listed above; the first two — resolving a `Fixed` modification once per row
rather than once per target, and the look-ahead replay — are constant
factors, and the finer memo key, which the pass makes definable because it
knows which members each application touched, is the structural one.

**Review (2026-09-06).** Two names changed — `Board passes` is `Board walks`,
the entry's `Query` enum is `Membership` — and one rule number: the existence
check is CR 604.2 (611.3b says the same), not CR 613.7a, which is the
timestamp rule; the label had been wrong since 2026-08-21, and the sweep of
the older sites landed with the `PermanentState` rename (2026-09-06) —
`codebase-state.md`, "Cross-cutting". The "judge walkthrough" this section
cited for the Blood Moon boards does not exist; the rulings above replace
it, and LI-2's cards are re-planned around them. The trace page for this
PR, `plans/traces/li-1-one-pass-per-board.html`, walks the Humility +
Hierophants board through the old walk and the pass call by call, a
look-ahead entry, and a graveyard Keldon Warlord, and carries the
field-by-field account of `Board` the review asked for.

### LI-2 — CR 613.8a, 613.8b, 613.8c (~1,300–1,500 additions) — ✅ 2026-09-06

*Evicted 2026-09-11 from `plans/layers-architecture.md`, where the heading and a stub remain.*


1. **Channels.** `writes(&EffectModification) -> Channels` — types, subtypes,
   supertypes, colors, abilities, keywords, controller, power, toughness, and
   everything for `CopyFrom`; `SetSubtypes` on a land also writes abilities
   (CR 305.7). `reads` for each thing an application reads: existence reads
   abilities; `you` reads the source's controller; a filter reads its leaves;
   a dynamic amount reads through its `CountOf`. Disjoint sets are the
   static check, and they settle nearly every pair — a 7c anthem writes
   power, and nothing in 7c reads it.
2. **The hypothetical check** for the pairs the static check leaves, per
   decision 3, in three parts matching 613.8a(b)'s clauses that the engine
   can express: existence, what it applies to, what it does (the `you` of a
   `SetController(You)`, a dynamic amount). "Text" is layer 3 and is not
   modelled.
3. **The loop**: dependencies over the pending set, the transitive closure
   (a handful of applications; Floyd–Warshall is fine), `ready` = no
   dependency, `loop_ready` = in a cycle whose closure is all mutual (a source
   component), the earliest candidate by the layer's sort key, apply,
   re-evaluate — CR 613.8c is the loop re-running its own first line. That
   driver is `resolve_order_within_layer`, and it differs from §9's reserved
   signature in one way that 613.8c forces: it applies as it orders, because
   the order after the k-th application is a function of the first k.
4. **Cards, and which boards a ruling backs.** **Urborg, Tomb of
   Yawgmoth**, the printed Legendary Land ("Each land is a Swamp in addition
   to its other land types"), registered and in `PERFORMANCE_POOL` beside
   Blood Moon — the existence dependency, its ruling quoted above, and a land
   any deck drops; `phase_ld_cards::urborg_effect` stays the Enchantment
   fixture the CR 305.6 tests rest on. **The Rootpath Purifier ruling's
   board as a named fixture** ("Lands you control are basic", an invented
   name, §3's rule): the applies-to dependency with a ruling behind it. The
   printed Purifier waits on item 9 for its library clause — registering it
   without one would wear a printed name while behaving differently. **Ashaya,
   Soul of the Wild**, registered in `stress` as the printed card of the same
   shape ("Nontoken creatures you control are Forest lands in addition to
   their other types"), with its expected answer **derived from the CR, no
   ruling covering it**: Blood Moon depends on Ashaya (applying Ashaya makes
   creatures nonbasic lands), Ashaya does not depend back (Blood Moon reaches
   no creature before Ashaya applies), so Ashaya applies first and Blood Moon
   then makes the creature-lands Mountains that lose their abilities (CR
   305.7) — Ashaya's own P/T CDA included. **Opalescence** ("Each other
   non-Aura enchantment is a creature in addition to its other types and has
   base power and toughness each equal to its mana value"), registered: its
   rulings with Humility (2009-10-01, both timestamp orders, layer by layer;
   2006-02-01, two Opalescences) are 7c's CR 613.6 test with the CR's own
   answers attached, and LI-1's pass already gives them — timestamp order
   plus the locked set. It needs one filter leaf, `ObjectFilter::EachOther`
   ("each other" is `id != source`; the self-stripping fixture's doc names
   this gap), on both `object_matches_filter`s. First job of LI-2.
5. **Tests.** Urborg + Blood Moon in both orders (a basic Forest is a Forest,
   never a Forest Swamp; Urborg is a Mountain — the ruling's words);
   the Purifier fixture + Blood Moon in both orders (the ruling's words: Blood
   Moon "can no longer apply to the lands you control"); Ashaya + Blood Moon
   in both orders, marked CR-derived; Humility + Opalescence in both orders
   and the two-Opalescence board, the rulings quoted in the test — 7c's
   CR 613.6 test; a 613.8b loop on creature types — "Elves are Goblins"
   against "Goblins are Elves", `SetSubtypes` both ways so the two orders
   differ — applied in timestamp order; a 613.8c chain where C's dependency
   on B appears only after A applies; 613.8a-003 with a CDA and a non-CDA in
   one layer; the row-older-than-counter order of
   `test_a_counter_older_than_a_power_reading_row_applies_first`;
   ATOM-613.8-001 claimed partial or not at all, since its "all activated
   abilities of other creatures" is not buildable. **And the four-card board
   from the judge answer** — Opalescence, Ashaya, Blood Moon and Urborg all
   in layer 4 — asserting the sequence Opalescence → Ashaya → Blood Moon with
   Urborg never applying, which is CR 613.8c re-evaluation on a printed
   board rather than a fixture chain. `specdb.py show` each atom first.

**As built (2026-09-06, `layers/li-2-dependency`).** Code and tests came to +1,893: `src` +1,201 / −99 across twelve
files, of which `board.rs` is +769 / −84 — the rewrite of LI-1's application
half, with `Application` carrying its reads and writes and `apply_layer`
become the loop — and `phase_li_cards.rs` the next largest;
`tests/phase_li2_integration_test.rs` is +692. The ~1,300–1,500 estimate
was for the code, and the docs put the PR near 2,600 — the top of §4's
band, and the reason LI-3 stays its own PR. Seven
departures from the pieces above, each with the board that forced it:

1. **The unit of ordering is the effect, not the row.** Piece 1's
   "application" was LI-1's — one registry row. Ashaya's second ability
   lowers to two layer-4 rows (`AddType(Land)`, `AddSubtype(Forest)`), and
   ordered as two applications Blood Moon slips between them: after the
   first row the creatures are nonbasic lands, Blood Moon depends on nothing
   still pending and applies, and the second row then adds Forest to the
   locked set (CR 613.6) — Forest Mountains that tap for {G}. CR 613.8 says
   "effect" in every clause and the judge answer applies Ashaya as one step,
   so `Kind::Effect` bundles one `EffectGroup`'s rows in a layer, in id
   order, applied as one thing; its key is the first row's. A resolution's
   rows bundle the same way. The lock stays keyed on the group, which is the
   caveat `codebase-state.md` "Before Layers" item 16 records.
2. **The static check reads two frames, not one set.** Piece 1's "disjoint
   sets" would send Blood-Moon-depends-on-Urborg to the hypothetical on every
   pass: both touch abilities. What Blood Moon reads of *abilities* is its own
   source's (CR 604.2), and Urborg's targets are lands, so `Reads { source,
   members }` splits the question and a source-read is a dependency only
   when the other application's targets contain the source. That pair is
   settled statically; the reverse — Urborg's source is a nonbasic land —
   reaches the hypothetical, and `board.rs`'s unit test counts exactly one.
   `Channels` is ten bits over `EffectiveCharacteristics`' fields; `writes_of`
   is exhaustive over `EffectModification` and over-approximates where the
   target decides (`AddSubtype(Forest)` writes an ability on a land and
   nothing on a creature, and the check has no target in hand).
3. **The hypothetical applies in place under a journal.** Decision 3 said
   "clone the frame of each member B affects, apply B to the clone,
   re-evaluate A on it". Built as: apply B to the live board through
   `perform` — the function the real application uses — with a `Journal`
   saving each frame's pre-image before its first write and each CR 613.6
   lock it records; observe A; restore. The clone is the same clone at a
   different moment, and every read A makes — existence, "you", the filter,
   a count — goes through the function it always goes through rather than a
   second evaluator over an overlay. `Observation` is 613.8a(b)'s three
   questions as data — `exists`, `affected`, the resolved outcome of every
   resolving arm per target — and a dependency is two observations that
   differ. §15.2 item 3 closes on this.
4. **The loop has a fast path, and the graph is built only when it fails.**
   Piece 3's "dependencies over the pending set, the transitive closure" runs
   when the key-first pending application depends on something. When it
   depends on nothing — one row of the matrix, almost always settled by the
   channel check — it is the next application, being the earliest candidate
   by construction. So a layer of N anthems costs N² bit-ands and no
   hypothetical (`a_pairwise_independent_layer_runs_no_hypothetical`), and
   the Floyd–Warshall closure runs over a handful. **`Dependency checks`** is
   the new cost row in §3: the hypotheticals a game ran.
5. **"What it does" is the effect's own act, not the board after it.**
   613.8a(b)'s third arm — "what it does to any of the things it applies
   to" — is read as the modification the effect would write, resolved:
   `SetController`'s player, a dynamic amount's number. It is *not* read as
   the characteristic an object is left with after both effects apply, and
   the reason is a reductio: under that reading any two effects *setting*
   the same field on a shared object would depend on each other, every
   set-against-set pair would be a loop, and CR 613.8b would hand exactly
   the boards the rulings walk back to timestamps — the dependency system
   would decide nothing there. The Humility + Opalescence
   rulings do not separate the two readings (a two-effect loop is applied
   in timestamp order too, so both give the rulings' numbers); what
   separates them is the third arm's stock example, "+1/+1 for each Elf you
   control" beside "creatures are Elves", where the *number* one effect
   writes is what the other changes. So `Observation::does` holds
   `resolve_modification`'s answer for the three resolving arms and nothing
   for a modification that carries its own answer.
6. **A trace hook.** `compute_board_traced` takes a layer index and a `Vec`
   to record into, and `resolve_order_within_layer` pushes a `TraceStep` —
   the application and what it affected — per application, in the order
   applied. The four-card board's sequence — Opalescence reaching
   Blood Moon; Ashaya reaching herself, Blood Moon and the Bears; Blood Moon
   reaching all four nonbasic lands; Urborg reaching nothing — is asserted
   step by step in `board.rs`'s unit tests through it, and
   `tests/phase_li2_integration_test.rs` asserts the board it leaves in both
   entry orders.
7. **ATOM-613.8-001 claims nothing**, as piece 5 allowed: its buildable board
   (flying granted, then all abilities lost, both by resolution on one
   creature) is not a dependency, and its dependency board ("all activated
   abilities of other creatures") is not buildable. The first board looks
   like an existence dependency and is not: "the existence of the first
   effect" is the *effect's*, and a resolution's effect lasts as long as its
   text says (CR 611.2a) whatever happens to the ability it granted — losing
   all abilities after a granted flying removes the flying, and the granting
   effect still exists, which is why the later timestamp wins there (a
   creature that gains flying after Humility entered keeps it) and no
   dependency is involved. The atom's own text concedes this for its first
   board; its second board's reasoning is the "result" reading piece 5
   rejects. ATOM-613.8a-003 is claimed partial: its own board is a 7a CDA
   beside a 7c pump, two layers, which clause (a) settles before (c) is
   asked; the test builds the same-layer pair — a layer-4 subtype CDA beside
   a row reading that subtype — and shows no hypothetical ran.

The tests are piece 5's, each in both orders with the ruling quoted beside
the assertion, plus `test_a_power_reading_row_older_than_a_counter_waits_for_the_counter`
— the order LI-1 left unpinned, now 3/3 — and a sabotage check: with
`next_ready` forced to key order, seven of the fifteen fail and they
are exactly the dependency boards, while the CR 613.6 and loop boards pass
either way, as this section predicted. `ObjectFilter::EachOther` landed as
piece 4 asked, on every matcher: `compute`'s answers off
`FilterPlayers::source`, `targeting`'s refuses it (a selection has no
source), and `pipeline`'s mods-invariance table classifies it. Urborg is in
`PERFORMANCE_POOL`; Opalescence and Ashaya are `stress` only, since neither
opens a path Urborg does not and Opalescence in a random game would move
every behavioural row for a reason that is not the engine's; the Purifier
fixture is `phase_li_cards::purifier_clause`, registered nowhere. 7c's
CR 613.6 test and the real `resolve_order_within_layer` are the two exit
items this PR owed.

**The A/B (2026-09-06).** `plans/fuzz_ab.py`, one sitting, three arms:
`main` at a6f2ed8 (A), LI-2's engine with the registry and both pools
unchanged (B), and the shipped tree (C). **B reproduces A on every row of
both pools** — walks, board walks, frames, memo hits, gathers, and every
behavioural row, at 50 and at 200 games — and the three serial timing
rounds are identical line for line outside `=== Timing ===`; on 40-game
`--dump-events` streams with the id masks applied, **no game differs on
either pool**. That is what CR 613.8 predicts for a pool whose only
dependency-shaped pairs — Humility beside a creature static — already gave
the dependency's answer under timestamp order in both directions; the 8
hypotheticals B runs per game are those pairs, and each confirms a
dependency that changes nothing. C differs from A because the pool did:
Urborg in `performance` (68 → 69 cards) and three cards in `stress`
(78 → 81), and every divergent game diverges at a `Library -> Hand` draw,
since the registry's sorted name list is what `random_deck` draws from.
CPU/game median 15.42 → 15.89 (B, +3.0%) → 15.72 ms (C, +1.9%), inside the
sitting's spread — round 3 had B faster than A; ms per 1,000 questions
0.155 → 0.160 → 0.163. `engineering-practices.md` §3 has the tables.

### LI-3 — conditional statics (~750–900 additions) — ✅ 2026-09-06

*Evicted 2026-09-11 from `plans/layers-architecture.md`, where the heading and a stub remain.*


1. **Lowering.** `static_ability_atoms` gains an `Effect::Conditional(cond,
   inner)` arm that lowers `inner`'s atoms exactly as today; nothing about
   the rows changes. `card_pool_lowering_test` already puts every registered
   card on the battlefield under the assert this arm replaces.
2. **The evaluator**, `engine/layers/condition.rs`: `holds(cond, board, game,
   source)` for the eight leaves — `ControlPermanent` and
   `OpponentControlsPermanent` over W's live frames with CR 109.5's "you";
   `LifeAtLeast`/`LifeAtMost` off the players; `CardInGraveyard` off the
   graveyards; `SourceOnBattlefield` off the zone; `SpellWasKicked` and
   `ModeChosen` assert, as `evaluate_amount` does for a resolution-only
   amount. One new leaf for the Rune-of-Flight shape, a predicate on the
   host (`§15.1`'s `ObjectRef::AttachedTo` sketch, one variant, not the whole
   sketch). Item 6 adds a reader, not a language.
3. **The existence clause.** `static_ability_still_exists` finds the ability
   on the source's live frame today; if its body is `Conditional`, the
   condition is evaluated there and then. CR 613.6's locked set covers the
   later layers.
4. **Consumer: Kird Ape** — {R}, 1/1, "This creature gets +1/+2 as long as
   you control a Forest" (Scryfall-verified before it is quoted in the card
   file). Asymmetric, the cheapest printed conditional, no new leaf, and it
   reads LI-2's board: a Breeding Pool is a Forest until Blood Moon makes it
   a Mountain, and the Ape loses its bonus with no dependency involved — a
   condition at 7c reading layer 4's output. In `PERFORMANCE_POOL`: the first
   row whose existence is a condition, which is a new path in the check.
5. **The Rune-of-Flight shape as a named fixture**: an Aura enchanting a
   permanent, "as long as enchanted permanent is an Equipment, it has
   'Equipped creature has flying'" — a conditional static granting a layer-6
   static over `Host`. Three things at once, and each exists after LI-1
   (the layer-6 grant), LH-2 (Equip) and this PR (the condition). Rune of
   Flight itself is registered when item 6 gives it its draw trigger.

**As built (2026-09-06, `layers/li-3-conditional-statics`).** +1,065 / −38
across nine files before the docs: `engine/layers/condition.rs` is new (+341
with its unit tests), `board.rs` +127, `phase_li_cards.rs` +180,
`game_state.rs` +72 / −34, and `tests/phase_li3_integration_test.rs` +329.
The ~750–900 estimate was for the code and held there; the tests and the
docs put the PR near 1,300. Four departures from the pieces above.

1. **The lowering recurses over the effect *body*, not the ability.**
   Piece 1 said `static_ability_atoms` gains a `Conditional` arm; built as
   `atoms_of_static_body`, with `static_ability_atoms` a one-line wrapper
   over `&ability.effect`. The difference matters at exactly one place: a
   conditional wrapping something the lowering cannot express — a `Modal`,
   an `Optional` — must be as loud as an unconditional one, and recursion
   into the same `match` is what guarantees that rather than a second copy
   of the declining arms. `test_a_conditional_wrapping_an_unlowerable_body_is_loud`
   is the pin. Nothing in the rows changes, as decision 5 promised, and the
   card-file lowering test asserts each new card's rows are the inner
   atom's exactly.

2. **`Condition::HostMatches` reuses `ObjectFilter`.** Piece 2 asked for
   "one leaf the Rune shape needs" out of §15.1's `ObjectRef::AttachedTo`
   sketch. §15.1 spells it `ObjectHasSubtype(ObjectRef, Subtype)` — a
   predicate per characteristic, times an object reference. Built as one
   variant carrying the filter the rest of the engine already uses, so
   "is an Equipment", "is a creature" and every conjunction of them are one
   leaf rather than a family, and `condition_reads` can ask `filter_reads`
   what it reads instead of enumerating. `Host` rather than `AttachedTo`
   for §13a decision 4's reason: one word for one relationship.

3. **The Rune-of-Flight fixture is Rune of Flight's *third* line, not its
   fourth.** Piece 5 named the Equipment clause — "as long as enchanted
   permanent is an Equipment, it has 'Equipped creature has flying'" — as
   the fixture. It is not registerable, and the blocker is neither the
   condition nor the layer-6 grant: **a static ability that grants a static
   ability registers no continuous effect at all.**
   `register_static_effects` lowers `Primitive::GrantAbility` to a layer-6
   row and stops; `register_granted_static_effects`, which derives the rows
   the *granted* ability generates, has exactly one caller and it is a
   resolution (`resolve.rs`). So the ability lands on the host's frame and
   does nothing — the inert-card failure this codebase names. Verified on
   the board before the fixture was written, and recorded as
   `codebase-state.md` "Before Layers" item 7g; the printed card is three
   things away rather than two. What ships is the line above it, "as long
   as enchanted permanent is a creature, it has flying".

   **That is a different clause, not a cheaper spelling of the same one, and
   the difference is what stays untested.** The two clauses are nearly
   disjoint by construction — one fires when the host is a creature, the
   other when it is an Equipment, and an Equipment is not normally a
   creature — so the shipped fixture exercises neither the Equipment
   condition nor the thing that makes that clause interesting: a grant whose
   payload is itself a static ability, reaching a *third* object. The
   untested board is Rune of Flight on Colossus Hammer ("Equipped creature
   gets +10/+10 and loses flying"), where the Rune's granted "Equipped
   creature has flying" and the Hammer's own "loses flying" are two layer-6
   effects on one creature and CR 613.7's timestamps decide. Nothing in this
   PR says what the engine would do there, because item 7g means the Rune's
   half generates no effect at all. What the fixture *does* carry is the
   condition and the layer-6 grant over `Host` in both timestamp orders
   against Humility, which is what LI-3 built; the rest arrives with 7g.

4. **The `effect_channels` clause is pinned by a layer-4 board, not by the
   Rune.** The handoff predicted the Rune fixture would be where the
   condition's channels bite. It is not, and the reason is a layer count:
   layer 6 writes abilities and keywords, and no `ObjectFilter` leaf
   reads either, so a layer-6 condition cannot be flipped by a layer-6
   effect through any filter the engine has. What can be flipped in its own
   layer is a **layer-4** condition — `Simian Clause`, "as long as you
   control a Forest, each creature you control is an Ape", beside Blood
   Moon, which writes subtypes. Without the clause that pair is settled
   independent (the Clause's only shared read is of its *own* source's
   ability list, and Blood Moon does not reach the Clause), applies in
   timestamp order, and makes an Ape out of a creature the CR says it
   should not; with it the pair reaches the hypothetical, one check, and
   the Clause waits. Shown by sabotage: with `conditional_reads_of` removed
   exactly that one test of the six fails, and with the existence clause
   removed five of six do.

**The A/B (2026-09-06).** `plans/fuzz_ab.py`, one sitting, three arms:
`main` at 9011d42 (A), LI-3's engine with the registry and both pools
unchanged (B), and the shipped tree (C). **B is byte-identical to A** —
every counter, behavioural and cost row at 50 and 200 games on both pools,
the three serial timing rounds line for line outside `=== Timing ===`, and
the whole 40-game `--dump-events` stream on both pools. Not "identical the
new row aside", as LI-1 and LI-2 each had to say: this PR adds no counter,
and its engine change is an arm no registered ability's body reached
before Kird Ape. So the A/B's claim is narrow and exact — **the change is
inert until a conditional card is in the pool** — and every row that moves
in C is Kird Ape's, every divergent game diverging at the draw that hands
somebody a Kird Ape where `main` handed them a Keldon Warlord. CPU/game
median 15.72 → 15.89 (B, +1.1%) → 16.01 ms (C, +1.8%), inside the
sitting's spread; ms per 1,000 questions 0.163 → 0.164 → 0.162.
`engineering-practices.md` §3 has the tables and the p99 caveat.

**The trace page for the phase's close** is
`plans/traces/item-7-an-effect-waits-for-what-it-reads.html`, beside LI-1's:
the loop's two paths, the judge answer's four-card layer 4 walked through
`next_ready` and the journal, the Simian Clause board with the sabotage step
that shows what `condition_reads` buys, and Kird Ape as the same card text two
layers apart with no dependency in it.

**What the cluster still owes: a *printed* board whose order needs CR
613.8c.** The rule is tested — `test_dependencies_are_re_evaluated_after_each_application`,
three fixtures over one Idol, where C's dependency on B does not exist until A
has applied and key order would leave the Idol a non-creature — and the trace
page walks it as D. But every *registered* pair answers the same with one
ordering pass: the four-card judge board's round-1 closure already gives
Opalescence → Ashaya → Blood Moon → Urborg, and rounds 2–4 only re-confirm it.
So the loop's most distinctive line is carried by fixtures alone, which is the
gap `engineering-practices.md` §3 asks a phase to name rather than leave
implicit. The shape to search for is a three-effect chain in one layer whose
middle link is *created* by the first; the CR's own stock example — "+1/+1 for
each Elf you control" beside "creatures are Elves" — is the nearest printed
neighbourhood, and it wants a third card to make the Elves. Not a back-stop and
not scheduled: a card want, to be filled by the first Phase 8 pair that fits.

**What the phase leaves for item 6.** The `Condition` AST now has an
evaluator in a static context, which is half of what CR 603.4's intervening
"if" needs; the other half is a resolution context, where `SpellWasKicked`
and `ModeChosen` stop asserting and start reading a cast. Adding a leaf is
still a variant plus one `match` arm plus one `condition_reads` arm — the
third is the one a new leaf must not forget, since a leaf that reads a
frame and says so nowhere is a wrong *order*, not a wrong value, and no
test of the leaf itself would catch it.

### LL — hidden walks — ✅ landed 2026-09-25

*Evicted 2026-09-25 from `plans/layers-architecture.md`, where the heading and a stub remain.*

*Renamed at PR #191's review (2026-09-26): the design's "replay" is the code's notes, since a replay already meant a replayed game in this tree and a decision a player's. RowNote was ReplayStep, AffectedSet was RowDecision, `PassMembership::LeftOut` was Membership::Replayed, `WalkKind::LeftOut` was Replayed, and `HiddenCards::InPass` was Held. Identifiers below are the code's; the prose keeps the design's word.*

`codebase-state.md` item 181's fix, and the second of the two PRs `roadmap-v2.md`
A6b puts ahead of TR-2b. Lettered as §13d says a phase implementing part of a
row is. Written before code the way §13a–§13d were: the finding that sets the
scope, the decisions, the pieces, and a size measured against the tree.

**What it fixes.** Floor 1 fails on a Commander board with a row reaching the
hidden zones: 4,600–6,800 decisions per second on one thread, against 15,950
without one (`fuzz-record.md`, "Measured 2026-09-25 for item 181"). LJ made
every object in a reached zone a member of every pass (`Board::seed`,
`board.rs:175`). Libraries and hands hold 93.4% of the objects off the
battlefield, and 0.22% of the frames seeded there are read before the next bump.
Thirteen printed cards make such a row, all Commander-legal.

#### The design in one page

Written at the owner's fourth pass (2026-09-25), once three rounds of review
had grown the detail below. Everything after this subsection serves one of
these lines.

- **The idea.** Leave the cards in libraries and hands out of the layer pass.
  As the pass runs, it notes its decision about each row that reaches those
  zones: whether the row exists at that moment, who "you" is, and whether
  CR 613.6 had locked it. When something asks about one hidden card, walk that
  card alone, replaying the notes, and keep its frame in the memo. Nothing is
  walked that nothing reads.
- **Why it is exact.** The pass writes hidden cards and almost never reads
  them. It reads one in three cases, and each keeps today's pass:
  1. the card is a static ability's source (Grist, or a static that functions
     in a hand), and it stays in the pass (decision 2);
  2. two rows of one layer could depend on each other through a card there,
     which only a custom card does, and the zone stays in the pass (decision
     4, the guard);
  3. a row reaching the zone computes its effect from other objects, again
     only a custom card, and the same guard applies.
- **What rides along.** Item 182: the cast-timing check reads a hand card's
  effective frame, so Teferi's flash works. The Grist fix: a static that
  affects its own card off the battlefield now applies in every zone. A debug
  audit compares every replayed card with the old full pass.
- **Built from what exists.** The replayed walk is `compute_non_member` given
  the notes. The guard asks the pass's existing CR 613.8 pre-check
  (`Channels`). A source joins through the scan `membership` already makes for
  `Fixed` rows. Item 182's gate is `no_row_reaches`, and the audit is LJ's
  seed behind a knob. The one new type is the list of notes.
- **Size and effect.** ~1,050 lines of code and tests. Pooled games match
  `main` in every counter. On a board with one of the thirteen cards, floor 1
  is predicted at ≥ 12,800 decisions per second on one thread, from
  4,600–6,800.
- **What the guard's precision decides.** Only how rarely a board falls back
  to today's cost, never an answer. With the pre-check as it is, a fallback
  happens in about 0.5% of four-player games' decks and costs exactly `main`'s
  price while both cards are out. Sharpening the pre-check (card-type values
  would cut that to about 0.02%) is a performance lever of its own, which the
  landing PR measures before any backlog entry is written (decision 4).

#### The finding that sets the scope: the pass reads a hidden card in three ways

Item 181's lever leaves hidden cards out of the pass and replays each row's
decision on the card's own frame. That is exact for a card the pass only
*writes*. So the first question is which hidden cards the pass *reads*, and the
tree answers it by enumeration. A pass reads another object through
`Board::frame_of` and through a row's affected set:

| The read | Whose frame | A card in a library or a hand? |
|---|---|---|
| CR 604.2's existence, CR 109.5's "you", a condition's reads (`static_ability_still_exists`, `FilterPlayers::you`, `conditional_reads_of`) | a static row's **source** | **Yes.** CR 113.6b lets a static function there, and `register_static_effects` registers one on arrival (`exiled_ancestor`'s text; `zone_function.rs`'s Grist shape) |
| `ObjectSet::Fixed`, `SourceOnly`, `Host` | the named object, the source, the host | A `Fixed` target is a member wherever it is. A `SourceOnly` source, yes. A host, no |
| `AmountExpr::CountOf`, `YouControlPermanent`, `OpponentControlsPermanent` | `battlefield_ids` | No |
| `CardTypesAmong`, `CardInYourGraveyard` | graveyard cards | No: a public zone, which stays in the pass |
| `HostMatches`, a CDA's amounts | the host; as above | No |
| CR 613.8's hypothetical (`depends_on` → `observe`) | every member the observed row **affects** | **Yes**, when two rows of one layer both reach the zone |

Everything else the pass does to a hidden card is a write. A write on a card
nothing reads can be replayed: its outcome depends on the card's own frame and
on three things the pass decided about the row at that moment: whether it
exists, who "you" is, and whether CR 613.6 had locked it. None of the three
depends on the card.

**So the lever is exact for a hidden card when three things hold, and they are
this design's decisions:**
1. The card is not a static row's source (decision 2).
2. No CR 613.8 decision can turn on a card in its zone (decision 4).
3. No row writing it reads the board to resolve (decision 4 too). A replay runs
   outside the pass and cannot see mid-layer frames.

The brief named the first. The tree adds the other two.

**Is the second real? Yes, and Grist, the Hunger Tide is the printed case**
(asked at the owner's review). CR 613.8a(b) asks whether applying one effect
"would change ... what [the other] applies to", and nothing in CR 613.8 limits
that to permanents. Two additive effects can depend on each other: what makes
one depend on the other is whether it adds what the other's filter tests, not
whether either removes anything.
- Grist's first ability reads "As long as Grist isn't on the battlefield, it's
  a 1/1 Insect creature in addition to its other types". Arcane Adaptation's
  second sentence reaches "creature cards you own that aren't on the
  battlefield".
- Both apply at layer 4, and neither is a CDA: CR 604.3a(1) does not list card
  types, and Grist's is conditional besides. With Grist in a hand, applying
  Grist's effect makes it a creature card, so Arcane's effect now applies to
  it, and Arcane's depends on Grist's.
- Timestamps alone would get it wrong, and CR 613.8b is what gets it right.
  A Grist drawn after Arcane entered is the younger object (CR 613.7d), so
  timestamp order would apply Arcane's effect first, while Grist is not yet a
  creature card, and Grist would miss the chosen type. The dependency
  overrides that: Arcane's effect waits until just after Grist's, whatever
  the timestamps, and Grist is the chosen type. A pass sees the dependency
  only with the card in the hand in it.
- Grist is the row's own source, so decision 2 carries it: the source joins
  the pass, and the pass sees the dependency as it does today.

**The guard's case is the other one: a card that is not a source decides the
dependency.** One hidden-zone row would have to change *other* hidden cards in
a way another hidden-zone row's filter tests, and no printed card does:
- The printed writes into hands and libraries are Artifact, onto cards that
  are already permanent cards (Biotransference, Encroaching Mycosynth);
  creature types (six cards); Desert (Dune Chanter); and colors (Mycosynth
  Lattice, Painter's Servant, Celestial Dawn).
- No hidden-zone filter tests any of those in a way those writes can flip.
- Biotransference beside Arcane Adaptation, for instance, never depends:
  Arcane's filter tests Creature, and Biotransference adds Artifact.

So the guard is there for the CR: a custom card, or a future printing
(`engineering-practices.md` §4: the CR is the customer, and a printed card is the test).
Neither the second condition nor the third is needed by any printed card, and
decision 4 is what the guard costs the printed ones. What each would take, in
custom text (asked at the owner's review of PR #191):
- **The second, a dependency decided through a card that is not a source.**
  "Cards in your library are creature cards in addition to their other
  types", beside Arcane Adaptation. Arcane's filter tests Creature and the
  new row writes it, so Arcane's effect depends on it through every library
  card and nothing on the battlefield. A pass without the library would order
  the two by timestamp, and with Arcane the older, a library card would miss
  the chosen type.
- **The third, a row reaching the zone that reads the board to resolve.**
  "Creature cards in your hand have base power and toughness each equal to
  the greatest power among creatures you control" is a 7b amount read while
  7c can still change those creatures. The pass reads it mid-layer; a walk
  after the pass would read the settled powers instead.

**Not §12's per-object dirty tracking.** That was deferred because a fine key
must list every input, and CR 613.8 makes other objects' answers inputs. This
keeps the coarse epoch: the record is invalidated with it, and every replayed
frame is stored at it. What changes is who is in the pass, which is LJ's
question answered narrower.

#### Decisions — numbered by what the tree poses

**1. The replay: what the pass records, where it lives, what a clone pays.**

For each row reaching a left-out zone, as the pass performs it (in `perform`,
and never under a hypothetical's journal), the pass records the row, its layer
and its decision then:

```rust
pub(crate) struct RowNote {
    layer_index: u8,
    row: Arc<ContinuousEffect>,   // #190 put every row behind one
    affected: AffectedSet,
}

pub(crate) enum AffectedSet {
    /// CR 613.6: the effect started earlier. It applies to the card iff the
    /// card matched where the effect started.
    Locked,
    /// The effect exists now (CR 604.2), and this is "you" as CR 109.5 read
    /// it off the source's live frame.
    Fresh { you: PlayerId },
}
```

A row the pass found gone is not recorded, since skipping it is the replay's
answer too. The steps are in the order the pass applied them, which is CR
613.8's order as decided on the members. Decision 4 is why that order is the
whole board's.

**A replayed walk** is the card's printed seed and then, for each layer below
the ceiling, its own CDAs first (CR 613.3), then that layer's steps in order:
- the zone gate;
- the filter against the card's own frame, with `you` from the step;
- the modification, applied as `apply_resolved` applies it.

That is `compute_non_member` with one more argument. A non-member is a card
with no steps, so one function answers both, and its fast exit becomes "no CDA
and no step".

**Every layer-6 "loses all abilities" makes the per-layer decision
necessary.** An ability removed at layer 6 still made its effect at an earlier
layer: Painter's Servant under Humility, Kenrith's Transformation or Oko,
Thief of Crowns' +1 colors every card in a library at layer 5, and has no
ability by the end of layer 6. A replay that asked the memo's settled source
would find no ability and skip the row. The test board is Titania's Song on
Mycosynth Lattice, the same shape on item 181's fixtures. The decision costs
nothing the pass does not do already: the pass decides existence at every
layer (CR 604.2, `CLAUDE.md`), and the note keeps its answer. (Reworded at
review, 2026-09-26: this said Titania's Song, which read as one card's
indulgence.)

**Where.** `LayerMemo` gets `notes: RefCell<Option<(u64, Arc<[RowNote]>)>>`
beside its frames. The pass that fills the memo stores it, and it is read only
at its own epoch. A live pass reads its own record in progress: a read at
ceiling `c` needs only the layers below `c`, and those are complete.

**What it costs.** A board with no row reaching a hidden zone records nothing.
Otherwise each pass builds one list of a few steps, one per row reaching the
zone per layer. One allocation outlives the pass, the `Arc`, and a fork bumps
its count and allocates nothing. Floor 2's 64 does not move, and
`clone_bound_test.rs` reads it.

**What a replayed card reads of other objects.** Its CDAs read them as
`compute_non_member` already does: from outside a pass, the settled frames.
As a member the card read live mid-pass frames. The answers agree under
`cda.rs`'s standing premise: every CDA reads information from strictly lower
layers, which is final by its own layer. A non-member's walk already rests on
that premise, so nothing new is approximated.

**Hidden information** (§13c decision 4's constraint): strictly less exposure
than today. The record holds rows, never cards. Replayed frames sit in the
memo, which is never iterated, and the pass no longer holds a library's order
at all.

**2. A hidden card the pass reads joins the pass: a static row's source.**

The pass reads a static row's source mid-layer, and a replay reads it only at a
ceiling. So a replay misses what an earlier application in the same layer did.
That is item 181's Wonder and Yixlid Jailer argument, one zone over: a hand
card with "as long as this card is in your hand, creatures you control have
flying", beside Hollow Hands. Both apply at layer 6, and the grant waits on the
strip (CR 613.8a), but only if the pass can see the strip reach the source.

**The rule: a static row's source is a member wherever it is when some row can
write it.** Two rows can:
- **its own row, when that row is `SourceOnly`** (Grist);
- **a row reaching its zone.** A reached public zone is seeded already, so what
  this adds is a source in a left-out zone.

`seed` appends such a source after the `Fixed`-named objects, in registry
order. `membership` answers `Member` from the scan it already makes for
`Fixed` rows, since rows are few. A resolution row's source does not join (a
buyback spell back in its owner's hand, say): CR 613.7b fixes its "you" and
its existence is unconditional, so the pass never reads its frame.

**The `SourceOnly` half fixes a gap found at design, folded in at the owner's
review (2026-09-25).** A `SourceOnly` static row off the battlefield never
applies on `main`. A throwaway probe put a Grist shape into a graveyard, a hand
and a library: three rows registered, and the card was a creature in none of
the three zones. `affected_members`' `SourceOnly` arm needs the source to have
a frame, a source no row reaches is a non-member, and `compute_non_member`
applies no rows.
- **Grist is the only printed card with such a row** (Scryfall, two phrasings,
  2026-09-25), and it is played: 73,469 decks on EDHREC's card page, and it can
  be a commander.
- **Its rulings name the zones.** "Anywhere but on the battlefield, Grist is a
  Legendary Planeswalker Creature — Grist Insect", so Essence Scatter can
  counter it and Negate cannot. On the stack it is a creature spell, so
  Thalia, Guardian of Thraben (pooled) does not tax it.
- **Flat on the pools.** No pooled static row is `SourceOnly` off the
  battlefield: Wonder's is a filter row, and Darksteel Colossus's and Nexus of
  Fate's "from anywhere" clauses are replacement bodies with no row. The seed
  pays one probe of its `seen` set per `SourceOnly` static row, Kird Ape's
  included.
- **Registering the real Grist** waits on its three loyalty abilities:
  `backlog.md` §2.11 (no loyalty ability exists), the −2's reflexive trigger
  (TR-3) and the +1's repeat. Its commander eligibility, which its first
  ruling grants "during deck construction", is the Commander track's:
  `random_deck` reads printed types under `// PRE-LAYER ZONE:`.

So LL tests Grist through a clause fixture: its printed frame with its first
ability and none of its loyalty abilities, registered nowhere.

**3. The replayed walk memoizes its frame. Yes.** The cast path asks the same
card more than once (decision 5). The frame goes into the state's own memo map
and never into the shared record, so a fork that reads a hidden card leaves the
original's memo as it was. A test pins that. Only cards something reads are
stored, so floor 3 falls back toward the no-row board (item 181 measured
+0–19 KB).

**4. When a hidden zone stays in the pass: the guard, which reuses the pass's
own pre-check.** Two conditions seed a reached hidden zone as LJ seeds it
today:
- **(a) A row reaching it has a dynamic modification** (`is_dynamic`). Its
  resolution reads other objects mid-layer: a count at 7c, or "you" at layer 2
  for `SetController`. A replay outside the pass cannot see those frames.
- **(b) Two rows of one layer both reach it, and the pass's CR 613.8
  pre-check cannot rule the pair out.** The pre-check is `Channels`:
  `filter_reads` against `writes_of`, what the pass already asks before it
  runs a hypothetical. When it cannot rule a pair out, the hypothetical could
  see a change in what one row applies to *through a card there*. The order
  the pass decides on its members would then not be the whole board's, on the
  battlefield as in a hand.

The guard adds no logic of its own: it asks the question the pass already
asks, and seeds the zone wherever the pass could have looked at a card there.
Both conditions read rows and nothing else, so `seed` and `membership` call one
function, `left_out_zones`, and cannot disagree.

**What it costs the printed cards.** (a) never trips. (b) trips only as a false
positive, since no printed card makes the dependency it exists for (the
finding above). The pre-check reads card types as one characteristic. So it
cannot tell Biotransference's or Encroaching Mycosynth's added Artifact from
something a creature-card filter tests, and it sends such a pair to the exact
test, which answers "no". A tripped board runs exactly `main`'s path while both
cards are out, so `main`'s engine measures it.

The measurement was a throwaway copy of `zone_reach_cost_test` (not
committed), 2026-09-25. It gave player 0 a {2} artifact carrying
Biotransference's and Arcane Adaptation's clauses, in either registration
order. Readings are with the pair out, medians of five rounds; cells read
`performance` / `stress`:

| | µs per decision | floor 1, decisions per second on one thread |
|---|---|---|
| no row | 60.5 / 49.9 | 16,530 / 20,040 |
| a trip, Biotransference older | 219.9 / 156.9 | 4,550 / 6,370 |
| a trip, Arcane older | 265.1 / 187.2 | 3,770 / 5,340 |
| LL, untripped (predicted below) | ~70–78 / ~54–60 | ≥ 12,800 / ≥ 16,500 |

- **Arcane older is item 181's board plus one exact test per pass.** It
  applies Biotransference's Artifact to ~27 cards under a journal and re-reads
  Arcane's filter over ~370, about 50 µs. `main` pays the same on this board
  today.
- **The other floors, tripped:** the worst clone read 9.0–10.9 µs, at floor
  2's 10 µs bound, and the state 110.9 KB, under floor 3's 128. The first
  decision after a cold redeal read up to 420 µs over a warm one, against 120
  µs with no row.
- **How often:** such a pair is in about 0.5% of four-player games' decks, most
  of it Maskwood Nexus (2.6% of all decks). That is from EDHREC's card pages,
  fetched 2026-09-25, treating a game as four independent decks and ignoring
  whether both cards are out together. A run that replays one deck holding
  such a pair trips in every game.

**Settled at the owner's fourth pass: reuse the pre-check as it is, and treat
its grain as a performance lever of its own.** The owner's rule is what would
elide the false positive: an added card type can only change whether a card
matches a filter that tests that type. The rule is per value, not "additions never interact":
Grist adds Creature and Arcane tests Creature, so they depend. Encoded in
`Channels`, it is card types, supertypes and colors as value masks, a bitmask
each over enums of 15, 5 and 5. That is a change to the pass's pre-check on
every board, not a part of this lever, so it is its own PR:
- ~150 lines and ~60 of tests in `board.rs`;
- a debug audit that runs the exact test wherever the finer check says
  "independent" and the coarse one would have looked;
- `Dependency checks` falls on the pools, and every gameplay row stays
  identical;
- the guard inherits it with no change, and its printed trips shrink to
  Biotransference beside Encroaching Mycosynth, about 0.02% of games.

**Card-type values are one grain among several** (the owner, at approval).
Every hypothetical the pre-check sends on that answers "no" is work a finer
grain might have skipped:
- values of subtypes, not only of types, supertypes and colors;
- two rows whose reach shares no zone;
- two "you own" or "you control" rows under different players.

**So the landing PR measures before anything is filed.** A throwaway probe
classifies every `Dependency checks` hypothetical by its answer, and each "no"
by the grain that would have ruled it out. It reads what they cost, on both
pools at four seats and on the tripped board above. If wasted hypotheticals are
a readable share of engine time, `backlog.md` gets an entry for the
pre-check's grain as a performance lever, with that data. If they are not, the
reading goes in the `fuzz-record.md` block and nothing is filed.

**5. Item 182 rides.** The cast-timing check at `put_on_stack.rs:673` and
`oracle/mana_helpers.rs:331` asks one wrapper in `oracle/characteristics.rs`:
is the card an instant, or does it have flash (CR 117.1a, 702.8a)?
- **Gated.** When `no_row_reaches` the card, its printed types and keywords are
  exact (CR 604.3a(1): a CDA defines neither). So pooled games walk no hand
  card, and their cost rows stay identical. The price is one `membership` call
  per nonland hand card per `castable_spells`, which is one registry scan.
- **The tags.** The two sites lose their `// PRE-LAYER ZONE:` tags. The
  cast path's other reads (is it a land, its spell ability) keep theirs, for
  item 182's reason: none of the thirteen changes them.
- **The end-to-end test:** a creature card cast from hand at instant speed
  under `teferi_flash_clause`, through `cast_spell`, with exact mana under
  `ManaWindowStop`.
- **A consequence to expect:** the Teferi arm of `zone_reach_cost_test` starts
  changing games, and its "card on" timings then compare different games.

**6. The audit: the same code with the shortcut off.** A debug build checks
every replayed miss against a pass that seeds the hidden zones as LJ does (a
knob on `Board::seed`), with the counts rewound as `audit_memo_hit` rewinds
them. That makes the whole suite the replay's test on every board with such a
row, as §12's audit made it the memo's. It is the one place a wrong replay would
show before a reading.

**7. Out, as item 181 scoped them.** The seed that honors the row's owner would
narrow only the public zones once this lands. The cheaper frame leaves floor 1
failing on its own, at about ×2. A separate cache key for frames off the
battlefield is not worth one: 1.3–3.9% of passes follow nothing but writes it
could ignore.

#### The pieces

**One PR, `LL`, with no `LL-2`.**

| | Site | Size |
|---|---|---:|
| `ZoneSet::HIDDEN`, pinned to `Zone::is_public` by a test | `types/zones.rs` | ~15 |
| `left_out_zones`, the guard's (a) and (b) | `board.rs` | ~50 |
| `Board::seed`: reached public zones as LJ seeds them, left-out zones not, the sources that join, the knob | `board.rs` | ~45 |
| `pass_membership`: `PassMembership::LeftOut`, the joining sources | `board.rs` | ~30 |
| the record, in `perform` | `board.rs` | ~35 |
| `compute_non_member` walks the notes; the arms in `frame_of`, `frame_at_ceiling`, `walk_uncached` and `compute_characteristics` | `compute.rs`, `board.rs` | ~90 |
| the record in `LayerMemo` | `state/layer_memo.rs` | ~30 |
| the debug audit (decision 6) | `compute.rs` | ~30 |
| `WalkKind::LeftOut` | `trace_records.rs` | ~5 |
| item 182: the wrapper and its two sites | `oracle/characteristics.rs`, `put_on_stack.rs`, `oracle/mana_helpers.rs` | ~35 |
| Fixtures: Titania's Song's first sentence, Grist's first ability on its printed frame, Arcane Adaptation's clause, a hand-functioning Wonder, the guard's two library rows | `cards/phase_ll_cards.rs` | ~190 |
| Tests (below) | `tests/phase_ll_integration_test.rs`, unit | ~460 |
| `zone_reach_cost_test` table 4: members split public / left out | `tests/` | ~25 |
| Docs: item 181 closed and archived with 182, §3.1's floor 1 standing, this section's stub and eviction, a `fuzz-record.md` block with the pre-check's waste, a `backlog.md` entry if that supports one, A6b, `state-of-play.md` | `plans/` | ~250 |

**~1,050 lines of code and tests, ~1,300 with docs.** Item 181 sized ~500 with
tests. The difference is decision 4's guard, decision 6's audit, item 182,
decision 2's `SourceOnly` half with Grist, and the tests those owe. That is
below `engineering-practices.md` §4's band, so one PR.

#### Tests

- **The replay, per layer.** Titania's Song's first sentence and Lattice's
  clause on the battlefield: a red card in a library and one in a hand are
  colorless (the layer-5 row existed when it applied). Beside it, Teferi's
  clause under the same Song: a creature card in hand has no flash, because the
  layer-6 grant waited on the strip (CR 613.8a) and was gone.
- **Who "you" is.** Teferi's clause under a layer-2 row giving it to player 1:
  player 1's creature cards in hand have flash and player 0's do not. A replay
  that read the row's registering controller would answer the reverse.
- **A hidden source joins the pass.** The hand-functioning Wonder in hand
  grants flying, and beside Hollow Hands it does not.
- **Grist, by its rulings** (the owner's choice of card, 2026-09-25):
  - a 1/1 Insect creature card in a hand, a library, a graveyard, exile and
    the command zone, and a creature spell on the stack; on the battlefield a
    planeswalker and nothing else;
  - cast under Thalia, Guardian of Thraben through `cast_spell`, it costs
    {1}{B}{G} with exact mana, because on the stack it is a creature spell (on
    `main` the cast needs {2}{B}{G});
  - in a hand beside Arcane Adaptation's clause (Elf), Grist the younger, it is
    an Insect Elf: CR 613.8 decided through a card in a hand;
  - in a hand under Teferi's clause, it has flash.
- **`seed` and `membership` agree on every zone** (unit). On a board with an
  object in every zone, under no row, Lattice's clause, a guard trip, a hidden
  static source and a `Fixed`-named hidden card: every object is in the seed's
  members exactly when `membership` answers `Member`.
- **A fork.** A clone reads a library card. The original's memo holds no frame
  for it at the epoch, and the two share one record.
- **The guard.** Two library rows at layer 4: "creature cards in libraries are
  artifacts" and, older, "artifact creature cards in libraries are Assassins".
  A creature card in a library is an Artifact Assassin, because the second
  waited on the first (CR 613.8a). Without the guard it would not be: the pass
  would see no dependency and apply the older row first.
- **Item 182.** A creature card is cast from hand at instant speed under
  `teferi_flash_clause`, through `cast_spell`. Without the clause, the same cast
  is refused.

#### Measure: the predictions

Written here before any arm runs, and copied into the PR body.

**`zone_reach_cost_test`, card on, `performance` / `stress`:**

| | item 181 | predicted |
|---|---|---|
| members off the battlefield per pass | 333–354 | ~15–30: the public zones |
| layer frames per decision | 307.7 / 214.1 | ~60–80 / ~40–55 |
| µs per decision | 206.2 / 146.8 | ~70–78 / ~54–60 |
| **floor 1**, decisions per second on one thread | 4,850 / 6,810 | **≥ 12,800 / ≥ 16,500** |
| worst clone on `performance`: µs, KB, allocations | 10.0, 112.4, 65 | ~8, ~103, 65 |
| redeal, first decision cold − warm, µs | 76–146 / 119–179 | ~5–50 |
| games diverging from the no-row arm, Teferi / Lattice | 0 of 20 / 0 of 20 | most of 20 (item 182) / 0 of 20 |

Floor 1 is the reading that matters. Item 181 put what remains at about 8 µs
per decision on `performance`, which is 62.7 + 8 ≈ 71 µs and about 14,100
decisions per second. The range allows for the replays item 182 adds, since
every hand card the timing check asks about is now read.

**`close_out.py`, both pools, two seats and four:** every gameplay row and every
cost row `IDENTICAL`. No pooled row reaches a hidden zone, no pooled static row
has a source in one, and none is `SourceOnly` off the battlefield. Instructions
per decision: **+0.1% to +0.4%**, from item 182's gate and the seed's `seen`
probes. Any cost row that moves is a finding.

#### What the building changed

- **Size: +1,289 −128 lines of code and tests**, against ~1,050. The fixtures
  and the tests ran over, and the replay itself came in near its estimate.
  Five commits: the `SourceOnly` join with Grist, the replay, item 182, the
  cost test's table 4, and a release-build fix.
- **A locked row is noted wherever it points** when its effect started on a
  noted row, and the replay applies it exactly when the card matched that
  start. That is what the pass does (CR 613.6's set), found while writing the
  replay. The pass's lock has its own defect within a layer, which the replay
  mirrors: `codebase-state.md` item 184.
- **`HiddenCards::InPass` exists only in a debug build**, beside its audit. A
  release build of the probe warned that it was never constructed, which
  `cargo build --all-targets`, a debug build, cannot show.
- **Names:**
  - `PassMembership::LeftOut`, `RowNote`, `AffectedSet::{Locked, Fresh}`,
    `left_out_zones`, and `source_joins`, as renamed at review;
  - `oracle::characteristics::is_instant_or_has_flash` for item 182;
  - the fixtures `grist_insect_clause`, `titanias_song_clause`,
    `arcane_adaptation_elf_clause`, `pocket_griffin`, `library_artificer`
    and `library_assassins`;
  - the test helpers `put_in_exile` and `put_in_command_zone`.
- **Five sabotages each fail the test built for them**: no notes, existence
  re-read off the settled source, the row's own controller as "you", no
  hidden join, and no guard. Four of them fail at the debug audit first.

#### Measured

`close_out.py`, `main` = `3656382` against the engine arm `0a69aa4`: every
gameplay row `IDENTICAL` on both pools at two seats and four, every counter
file byte-identical outside `=== Timing ===`, and +0.36% instructions per
decision. `zone_reach_cost_test`, card on, `performance` / `stress`, against
the predictions:

| | predicted | read |
|---|---|---|
| members off the battlefield per pass | ~15–30 | 15.4 / 12.1 |
| layer frames per decision | ~60–80 / ~40–55 | 55.1–58.9 / 43.4–47.0 |
| µs per decision | ~70–78 / ~54–60 | 78.5–84.0 / 64.8–71.2 |
| **floor 1**, decisions per second on one thread | ≥ 12,800 / ≥ 16,500 | **11,907–12,734 / 14,054–15,435** |
| worst clone on `performance`: µs, KB, allocations | ~8, ~103, 65 | 7.9–9.3, 94.5–98.4, 39–40 |
| redeal, cold − warm, by turn from turn 10, µs | ~5–50 | 5.5–50.8 / 5.4–63.8 |
| games diverging, Teferi / Lattice | most / 0 of 20 | 20 / 0 of 20 |

- **Floor 1 holds, under the predicted range.** The no-row arm read 68.4 /
  55.8 µs in this sitting, against 60.5 / 49.9 in the trip sitting and 62.7 /
  49.6 in item 181's. Against its own sitting's no-row arm the lever is ×1.15
  (Teferi) and ×1.23 (Lattice), which is what the prediction's ~71 µs against
  62.7 says. The absolute fell short with the machine.
- **Layer walks per decision rose from 1.1 to 5.2–5.8**, which the prediction
  allowed for without sizing: item 182's gate asks about every nonland hand
  card at each castability check, and on these boards each is a replayed walk
  once per epoch.
- **Floor 3 breaks on one fixture game**, which nobody predicted.
  `stress` 12351 under Teferi's clause, a game item 182 changed, built a
  stack 29 deep and reads 150.4 KB at its end (`codebase-state.md` item
  183). LL's memo is about 9 KB of the breach.
- **The pre-check's grain, measured for the owner's note**: about 1% of
  engine time on `performance`, 0.7% on `stress` (`backlog.md` §2.38).

#### At the owner's review (PR #191, 2026-09-26)

- **CI failed on floor 3**, at `clone_bound_test.rs`'s fourth game, the
  one the breach above was read on. `zone_reach_cost_test` read it first,
  and the release-only gate was not run before the push. The owner's call:
  the floors hold a deep stack. A clone now rebuilds a map whose capacity is
  past twice its length (`types::ids::FitOnClone`, on `objects`,
  `battlefield` and `stack_entries`): that game reads 73.2 KB, and the worst
  of the gate's 210 readings 90.7 KB, from 150.4. A clone taken during a
  deep stack is item 183's open half, back-stopped before Phase 8
  (`roadmap-v2.md` B10).
- **Why the close-out's sitting read slow.** It ran seconds after about four
  minutes of full-core builds and a debug test suite on the native Windows
  machine, which ran no other load of this session's. Two re-reads on an idle
  machine read the no-row arm at 60.7 / 44.6 µs and floor 1 at 14,290–15,756
  / 18,092–18,922 decisions per second with the card out, above the
  prediction. The second, with `FitOnClone`, reads the worst clone at 7.8 µs
  (6.8 before it) and 91.3 KB.
- **The names**, as the note at the top of this entry says.
