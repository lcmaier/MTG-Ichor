# RE-4 review — findings, triaged (2026-09-13)

Owner's review of PR #132, captured before fixing anything
(`engineering-practices.md` §4). Verdicts are mine; each names what it changes.
Delete this file when the last theme lands.

**Status (2026-09-13):** themes A (`fcf161b`), C (`c74ff47`) and B
(`d98c9be` code, docs in the commit after) are on the branch. **Theme D is
its own PR, immediately after #132 merges** — the owner's call, since it is
a large mechanical diff about nothing to do with tokens. Themes E and F and
the re-measurement (the engine moved in A, B and C, so the A/B, the
determinism runs and the §3 tables are stale) **wait for the owner to review
the three commits.** The ledger lines the code replaced are items 126–128
now; R2/R6/R8/R14 below refer to the numbering they had at the review.

## Theme A — the loop was the engine's, not the rules'

**R1. §11 item 77 claims a CR 104.4b loop; the owner reads CR 614.5 and Alms
Collector's ruling the other way.** The owner is right. Alms Collector's second
ruling — *"once a replacement effect has been applied to an event, it can't be
applied again to the resulting events"* — covers the collector's "you draw a
card" half, which is a resulting event of the same replacement. Two
Reflections and two Collectors across two seats therefore terminate: P0's draw
doubled, halved by P1's Collector with P1 drawing one; P1's draw doubled by
P1's Reflection (first opportunity), halved by P0's Collector with P0 drawing
one; and that draw meets four effects that have all applied. P0 draws two, P1
one. The engine looped because a **rider's proposal starts a fresh applied
set** — §4.1a's "fresh lineage" bullet, RE-2's design — which is exactly the
re-application CR 614.5 forbids. **Fix (design, this PR):** a rider carries the
replaced event's applied set; `Rider.lineage` filled with the group's final set,
`GameState::rider_lineage` handed to the rider's proposals. Kalitas + Doubling
Season still makes two Zombies (the Season never applied to the death). The
cap is then a guard against *engine* loops — a lost lineage — and returns an
error rather than a draw; its bound is a fuzz row, not a magic number (R12).
Corrects: §11 item 77, §4.1a, §3.2d's Alms note, `CLAUDE.md`'s rider bullet,
`glossary.md`, `resolve_rider`'s doc, the test, §3's block, the archive.

**R12. `MANDATORY_LOOP_DEPTH = 48` is a magic number; pull the loop detector
forward?** CR 731's loop handling is about *optional* actions and shortcuts
(731.1–731.2), and CR 104.4b's mandatory loop is undecidable in general — every
engine bounds it, as `check_state_based_actions_loop` already does at 500.
After R1, no replacement-only chain can loop by construction (CR 614.5 plus a
finite instance set), so the bound is an *engine* invariant: nesting deeper
than any legitimate chain means a lineage was lost. **Fix:** `Max batch depth`
becomes a fuzz row so the bound is measured every run; the guard errors,
loudly, with the measured maximum in its doc; no rules claim attaches to it.

**R13. Explain `decomposition_depth`.** Answered in the reply; the field's doc
is rewritten with R1 (it is a debug invariant per lineage, never a cap).

## Theme B — what this PR should have brought in (the owner's rule)

**R2, R6, R8, R14. "Why not bring the cards in?" / "if so small, do now" /
"why leave mutation half done?" / "is the template scheduled?"** Adopted as a
rule in `engineering-practices.md` §4: **an arm the PR's own type opens, with
a printed customer and sized under ~80 lines, ships in that PR** — the ledger
is for facilities, not for arms the type is already open for. Applied here:

- `EventPattern::CreateTokens { kind }` (item 126) — Divine Visitation's
  "creature tokens", Xorn's "Treasure tokens". Built; Divine Visitation
  registered.
- `GameActionTemplate::CreateTokens` (item 128) — **not `Plus`** (item 127):
  Xorn's text adds "an additional *Treasure* token", a named def, and
  Chatterfang's "those tokens plus that many Squirrels" is the same shape with
  a count, Divine Visitation's "that many Angels instead" the replacing form.
  One template, three printed customers; Divine Visitation registered; Xorn
  and Chatterfang wait on a Treasure def (§2.19) and a variable sacrifice
  cost. `Plus` over a creation stays refused, now with the right reason.
- `TokenDef::enters_tapped` (item 129's cheap half) — 132 printed "create a
  tapped" effects, every one a trigger or a Treasure, so a fixture proves it;
  "attacking" is CR 508.4's and combat's.
- Item 130 is deleted: it recorded nothing to build.
- **Bard, King of Dale** — R5 found it is a draw doubler *and* a token
  doubler, both halves built; registered as RE-4's fifth card.

**R5. Bard, King of Dale does not replace draws with tokens.** Correct — it
doubles them; Hullbreacher is the draw-to-Treasure card, and it waits on
§2.19. Item 128's text corrected; Bard registered (above).

**R16. Hallowed Moonlight's "cast from any zone" ruling owes a test when
§2.3 lands.** Recorded in `backlog.md` §2.3's atoms and in the test's doc.

## Theme C — the suppression premise, and the compile-time hooks

**R15. Master Biomancer beside Hallowed Moonlight need not be asked: the
token ceases to exist either way.** Right, and it is §4.1's standing question
applied. A single `Instead(ZoneChangeTo)` beside `EnterWith`s has one outcome
whichever applies first — the substitute discards the entry's mods, and after
it nothing entry-shaped matches — under the common clauses (static, rider-less,
not optional). **Fix:** a fifth shape in `ordering_cannot_change_outcome`, its
debug check, and the test flipped to zero prompts. The per-token fresh-set
claim moves to a board that *is* observable per token: two devour tokens
created together, each asked once, neither offered the other (CR 614.13a from a
producer).

**R10. "The rule for whoever adds (a) or (c): revisit the predicate" is
invisible to a fresh agent; make it a compile error.** Three of the four
already are: (c) `filter_is_mods_invariant` matches `ObjectFilter`
exhaustively, (d) `reads_the_amount` matches `EventPattern` exhaustively, (b)
`pattern_watches`' entry arm destructures every field of the pattern. (a) was
not — `EnterModsTemplate::is_fixed` read one field by name. **Fix:** it
destructures the struct, so a new field breaks it; the predicate's comment
names the four hooks instead of the rule.

## Theme D — the record moves out of the practices doc

**R4. Move the statistics out of `engineering-practices.md`.** Directed. Every
`**Re-recorded …**` block and §3.1a — lines 243–1636, about 1,400 of the
file's 2,400 — move to `plans/fuzz-record.md`, newest first, with a two-line
redirect where they were; the rules (§3's list, "the five bold rows", 3.1–3.5)
stay. Pointers that name "§3's table" as the *rule* still resolve; the redirect
covers the rest.

## Theme E — measurement framing

**R3. The fourth arm looks like a workaround for the byte-identical check.**
Half right. The check is the strongest instrument the A/B has — a whole-game
trace equality — and it is not the problem; the *middle arm's definition* was:
"registered, old pool" is an engine reading only on `performance`, because on
`stress` registration *is* the pool change. The arm that reads the engine on
both pools is "cards unregistered", and it should be the standing middle arm.
**Fix (doc):** §3's recipe names the three arms as engine / registered /
pooled with that meaning; §11 item 80 reframed.

## Theme F — answers that change a sentence

**R7. `CreateTokens` reported as decided: nothing reads it?** Correct — nothing
reads a `CreateTokens` member of `performed` today; the precedent is for the
log and for whoever reads `performed` later. The archive sentence says so.

**R9. "Unreachable rather than wrong" — it *is* wrong.** Yes. The ledger's two
rows split on reachability, and every open item is a wrong answer or a
missing facility; the phrase conflated the axes. One sentence at the ledger's
head fixes the vocabulary; the archived item is a record and stays.

**R11. `subject_of`'s `CreateTokenIn` arm looks bespoke.** It is one line the
compiler forces (three exhaustive matches, one line each); the bespoke thing is
the variant, which decision 3 argued for over an `Option` on `ZoneChange.from`.
No compaction available short of the `Option`, which was the rejected shape.

## General

**G1. Predefined tokens.** §2.27's library half, now ordered: Walker, Clue,
Food, Blood, Map, Junk, Lander, Mutagen, Shard and Powerstone need nothing
but `Primitive::Sacrifice` as a cost, which CM-3 shipped — one small PR with
`Primitive::Investigate` (138 printed "investigate"); Treasure and Gold land
with §2.19, which the Commander mana base also needs, so §2.19 is the next
mana-side PR; the Roles need attach-on-creation; Wicked Role needs CR 603;
Incubator needs CV-5. Written into §2.27.

**G2. Deferral anxiety, and the unsized.** The retrofit risk is in *facts*, not
*features* — a fact is unrecoverable if not captured when it happens (an event,
a cause, a frame); a feature is a normal diff later. Every RE-4 line was a
feature, and the rule in Theme B is the answer to the ones that were cheap.
For the unsized: the one unsized item on the route is critical-path item 6,
and its rule is already "write the doc first". The ledger's audit row to watch
is "reachable, wrong today" (4), which is the real bug list.

## Second pass (2026-09-13) — questions on themes A, B, C; nothing changed yet

**R17. Is the nesting guard a loop detector, or a guard against another
two-Collectors board?** Neither. It cannot fire on a rules loop and it is not
meant to catch one: after theme A a replacement-only chain is bounded by
CR 614.5 by construction, so the only way past `BATCH_NESTING_LIMIT` is an
engine defect of the shape theme A fixed — a nested batch that starts without
the lineage it should carry, or a rider encoding that re-proposes its own
event. It turns that stack overflow into an error naming the invariant.
A *legitimate* mandatory loop through triggers (item 6's) is CR 104.4b's and
is R21's business, not this constant's. **Doc:** the constant's comment says
"a loop of its own making", which is right, and should say in one more
sentence that it is not the CR 104.4b detector. The 32 is a placeholder until
the re-measurement reads the `Max batch depth` row; the doc should carry that
number when it exists.

**R18. Performance of carrying lineage.** Bounded by the number of instances
that applied to one event — single digits on any printed board — and no walks
or frames: a rider's set is one clone per rider queued (riders are rare), the
`Option<HashSet>` handed to `execute_actions` is a move, and an empty set
allocates nothing. The one growth is a doubled-draw chain, where each level's
set is one entry larger and cloned once per level — quadratic in a chain
length that is the number of draw instances on the board (four, on the widest
registered one). The re-measurement is the check; the two-seat `performance`
identity is what should hold.

**R19. Does "attacking" go in the entry seed too?** Yes, the same door —
CR 508.4 is "put onto the battlefield attacking", a fact about how it enters
— but not a `bool`: it names what is attacked, and it means nothing outside
the combat steps 508.4 allows. So `EnterMods` gains `attacking:
Option<AttackTarget>` and `place_on_battlefield`'s wake writes the combat
state. On growth: CR 614.1c's vocabulary is bounded — tapped, counters,
attacking, face-down (Layer 1, the one that "changes everything"),
transformed, as a copy, attached to — six or seven fields ever, and every
reader that must decide about a new one is a full destructure that the
compiler breaks (R10). The risk is semantic, not arity: a field that feeds a
*characteristic* (face-down, copy) breaks the entry-shape premise, and that
is the case the predicate's expiry list already names.

**R20. "the def" is undefined in isolation.** Agreed. The crate says *def* for
every card-authored definition — `TokenDef`, `ReplacementDef`,
`RestrictionDef`, `AbilityDef` — as distinct from an *instance* (a def on a
board, with a source and a controller) and an *object*. **Fix:** a glossary
entry beside **instance**, `def` on the watchlist, and the test doc line
reworded to "the token's def, never the entry's frame". Small; with E/F.

**R21. The loop watcher.** Recorded where the owner remembers it: `roadmap.md`
D11 (mandatory loop detection, "post-v1 / stretch") and D26 (divergent loop
shortcutting — `GameNumber` with `Finite`/`Shortcut`/`Relative`,
`LoopDeclaration` with `Concrete`/`MatchPlusN`, `ask_declare_loop_count`
through `pick_number`; the rule it cites as 727 is 731 in `tmnt`), with
"Loop detection Tiers 1–3 and D26 survive, re-based on performed-action
transcripts" in the archive's item 3, and `replacement-architecture.md` §12's
one-line pointer. The tiers doc it names, `state-tracking-architecture.md`, is
not in the tree under that name. **What theme A changes about the premise:**
the mandatory half cannot be a replacement-only loop any more, so it is a
*trigger* question (item 6) and the buffer-of-N-batches-with-state-hashes
watcher belongs to the trigger phase's design, reading the performed stream
RA built; the optional half — a player *declaring* a loop as a choice
sequence plus an expected per-iteration delta, the engine running one
iteration to check the delta and then applying it N times as one batch — is
CR 731.2's procedure and the piece no simulator has, and it is the AI
harness's infinite-mana question. **Proposed:** a `backlog.md` entry, "Loops
(CR 104.4b, 731)", holding the sketch, the two halves, what each depends on
(item 6; item 40's discipline for a state hash; the `DecisionProvider`
surface), and D11/D26 struck as graduated into it — a capture, not a design;
the design is its own doc when it is scheduled.

**R22. How many suppression shapes, and organize now?** Five, one per phase
since RC-4, and the sixth candidate is already on the board: Divine
Visitation beside Parallel Lives is asked and has one outcome (a multiplier
and a replace-by-"that many" commute; the test asks both ways). The shapes are
ad hoc proofs over a whole bucket; the organization they want is pairwise —
classify each `Rewrite` on an event kind into a commutation class
(multiplicative, additive, absorbing exit, mods-adding, idempotent
substitute) and a table of which classes commute, with the common clauses
(static, rider-less, not optional, not a counter) factored out and one debug
check per class rather than per shape. **Not in this PR**: the five each have
a printed board and a check, and the table is a ~150-line refactor of a
predicate the standing question has corrected three times, which wants a
session of its own. **The trigger is the sixth shape**, which RE-5's counter
doublers beside Hardened Scales will force (a multiplier and an additive do
not commute, which the table states and a sixth predicate would have to
re-derive). Recorded as a backlog entry with that trigger.
