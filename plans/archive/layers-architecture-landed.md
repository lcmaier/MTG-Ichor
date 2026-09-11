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
