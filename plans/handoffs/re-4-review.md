# RE-4 review — findings, triaged (2026-09-13)

Owner's review of PR #132, captured before fixing anything
(`engineering-practices.md` §4). Verdicts are mine; each names what it changes.
Delete this file when the last theme lands.

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
