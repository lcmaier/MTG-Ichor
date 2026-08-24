# Atomic Test Generation — CR-Exhaustive Process

> **Historical process doc.** The corpus it produced is `sessions/*.md`, which is now the
> authored source of truth. Paths below are as they were: `plans/implementation-plan-final.md`
> has since moved to `plans/archive/implementation-plan-final.md`.

## Overview

This document defines a process for systematically generating atomic test specifications by walking the **entire** Comprehensive Rules at the sub-rule level. The CR text is in `MTG-Rules/` as chapter files. Each session gets the literal CR text pasted in — the LLM reads every sub-rule, not working from memory.

**Exhaustiveness guarantee:** Every sub-rule in the CR is seen by exactly one session. Nothing is skipped by omission because the input IS the CR, not a hand-picked list of rules.

## Context Files (every session)

If running in Cascade (with workspace access), add this to the prompt:
> Read these files from the workspace before starting: `design_doc.md`, `plans/roadmap.md`, `plans/implementation-plan-final.md`, and the CR chapter file(s) specified below. Do NOT work from memory — read the actual files.

If running outside Cascade (e.g., ChatGPT, Claude web), paste the contents of those files into the prompt manually.

The CR chapter files live in `MTG-Rules/` relative to the repo root.

---

## Session Plan

Chapter file sizes determine grouping. Small chapters are combined; large chapters are split.

| Session | CR Input File | ~Size | Notes |
|---------|---------------|-------|-------|
| 1 | Chapter 1 (100-199) | 135 KB | Game concepts, players, mana, colors, objects, abilities overview |
| 2 | Chapter 2 (200-299) | 44 KB | Parts of a card |
| 3 | Chapter 3 (300-399) | 37 KB | Card types |
| 4 | Chapters 4+5 (400-599) | 66 KB | Zones + Turn structure (combined) |
| 5 | Chapter 6: 600-608 | ~half of 123 KB | Spells, casting, activating, resolving |
| 6 | Chapter 6: 609-616 | ~half of 123 KB | Effects, continuous effects, layers, replacement, prevention |
| 7A | `ch7-pt-1.txt` (700.x + 701.x) | 68 KB | General additional rules + keyword actions |
| 7B | `ch7-pt-2.txt` (702.1-702.80) | 72 KB | Keyword abilities: Deathtouch through Wither |
| 8 | `ch7-pt-3.txt` (702.81-702.190) | 78 KB | Keyword abilities: Retrace through Sneak |
| 9A | `ch7-pt-4.txt` (703-712) | 75 KB | TBAs, SBAs, copying, face-down, split/flip/DFC, meld |
| 9B | `ch7-pt-5.txt` (713-732) | 49 KB | Substitute cards, tokens, player control, shortcuts, illegal actions |
| 10 | Chapters 8+9 (800-999) | 82 KB | Multiplayer + Casual (mostly OUT-OF-SCOPE or DEFERRED) |

**Splitting Ch 6:** Find the boundary between rule 608 and 609 (search for `^609.`). Paste 600-608 into session 5, 609-616+ into session 6.

**Splitting Ch 7:** Pre-split into 5 files in `MTG-Rules/LLM-Chapter-Splits/` (ch7-pt-1 through ch7-pt-5). Each session reads one file directly — no manual splitting needed.

---

## Shared Preamble (copy into every session prompt)

```
# Role
Senior Rust engineer, master Rust architect, Magic the Gathering Comprehensive Rules expert.

# Task
You are given the LITERAL TEXT of a section of the Magic: The Gathering Comprehensive Rules, plus design documents for an MTG simulator. Walk every sub-rule in the provided CR text and generate atomic test specifications for the simulator.

# Procedure
For EVERY numbered sub-rule (e.g., 613.8a, 614.12, 704.5f) in the provided CR text:

1. READ the sub-rule.
2. CLASSIFY it using the decision process below.
3. If TESTABLE, generate an atomic test (format below).
4. If BOUNDARY-DEF, generate a boundary test (format below).
5. If PURE-DEF, skip but note if it's a prerequisite for understanding a TESTABLE rule.
6. If OUT-OF-SCOPE, DEFERRED, or ALREADY-IMPLEMENTED, list the rule number and classification in a summary table at the end. Do NOT generate a test.

This ensures every sub-rule is accounted for — either it has a test, or it's explicitly classified as not needing one.

## Classification

For EVERY rule, ask: **"Can I construct a game state where implementing this rule WRONG produces a different observable outcome than implementing it RIGHT?"**

- **TESTABLE** (YES) — Observable behavior or state change. Generate an ATOM test. *If the CR text includes an Example, it is almost certainly TESTABLE.*
- **BOUNDARY-DEF** (YES, at a category boundary) — Defines set membership (e.g., "permanent card means artifact, battle, creature, enchantment, land, or planeswalker card"). Generate a test checking one in-set AND one out-of-set member.
- **PURE-DEF** (NO) — Names a concept with no independent mechanical consequence (e.g., "110.1. A permanent is a card or token on the battlefield"). Skip, but note if it's a prerequisite for a TESTABLE rule.
- **OUT-OF-SCOPE** — Mechanics the simulator will NEVER implement (Conspiracy, Planechase, Archenemy, Un-sets). Permanently excluded. (Vanguard is DEFERRED, not OUT-OF-SCOPE.)
- **DEFERRED** — Will implement in a later phase. Tag with phase (e.g., `DEFERRED — Phase 9: Commander`). Stays in the catalog as backlog.
- **ALREADY-IMPLEMENTED** — Covered by Phases 1-4.5 AND implementation is complete (no TODO markers).

**Err on the side of TESTABLE.** When in doubt, ask: "Could a reasonable engine get this wrong silently?" If yes → TESTABLE.

# Atomicity Rule
A test is atomic if it exercises exactly ONE rule-defined transformation on game state. Minimal board state (typically 1-3 permanents, 0-1 spells on stack). Expected result follows from that ONE rule — if two unrelated rules must both work, it's a composition test (COMP-).

If a test unavoidably requires another mechanism as setup/precondition, note the dependency explicitly. The test is still atomic if the "other mechanism" is precondition, not what's being verified.

# Cross-Cutting Meta-Rules
Some early rules (e.g., 101.2 "can't overrides can") are meta-rules that manifest across multiple game systems. When you encounter one:
1. Classify as **META**. Do NOT generate concrete ATOM- tests — those belong to the session covering the specific system.
2. Output: **META-[rule]:** [summary] / **Expected systems:** [list] / **Concrete tests deferred to:** [sessions]

Later sessions: when a rule implements a meta-rule behavior, generate the concrete ATOM- test AND tag it (e.g., "Tags: META-101.2"). The merge pass verifies coverage.

# Multi-Clause Sub-Rules
When a single sub-rule contains multiple independent clauses, generate separate ATOM- tests under the SAME rule number (e.g., ATOM-301.5c-001, ATOM-301.5c-002). Do NOT invent new sub-rule letters. VERIFY every rule number against the LITERAL TEXT in the file — do not rely on memory.

# Output Format

## For each TESTABLE sub-rule, produce:

**ATOM-[rule]-[N]** (e.g., ATOM-613.8a-001)
- **Rule:** [full rule number] — [one-line summary of the rule text]
- **Mechanism:** [what engine behavior this tests]
- **Minimal Board:** [minimum game state]
- **Action:** [what event/trigger occurs]
- **Expected Result:** [correct outcome per CR]
- **Phase:** [which project phase implements this, per roadmap.md]
- **Ticket:** [ticket ID from implementation-plan-final.md, or "NEW — [brief description]" if none exists]

## At the end of the section, produce:

### Composition Tests
Tests that require 2+ atomic mechanisms. Format same as above but prefix COMP-. List which ATOM- tests they compose.

### Gap Report
Any mechanisms referenced in roadmap.md or implementation-plan-final.md that SHOULD have a test in this CR section but don't. These indicate either a missing sub-rule in the CR (unlikely) or a project-specific concern not tied to a single CR rule.

# Output Strategy — CRITICAL (anti-timeout)
Do NOT attempt to produce the entire output in a single response. You WILL hit output token limits and lose work. Follow this mandatory chunked workflow:

1. **Chunk 0 (planning):** Read all required files. Count the top-level rules in scope. Divide them into chunks of **at most 15 top-level rule numbers per chunk** (e.g., 702.81-702.95 is one chunk). Announce your chunk plan (list of rule ranges per chunk) and write the session header to the output file. **STOP and wait for the user to say "continue."**
2. **Chunks 1-N (rule batches):** For each chunk, process those rules — classify and write ATOM-/BOUNDARY-/DEFERRED/OOS entries. **Append** to the output file (do NOT rewrite earlier content). At the end of each chunk, write "--- End of Chunk N ---" into the file. **STOP and wait for "continue."**
3. **Final chunk:** Write the Classification Summary Table, Composition Tests, and Gap Report. Append to the file. Done.

**Hard rules:**
- **NEVER write more than ~4000 words of test output per file-write tool call.** If a chunk would produce more, split the tool call into multiple appends.
- If the session-specific guidance below defines explicit chunk boundaries (rule ranges), use those instead of choosing your own.

Write all output to `plans/atomic-tests/sessions/session-N.md` (where N is the session number). Do NOT write to the top-level plans/ directory.
```

---

## Per-Session Instructions

Each session prompt is assembled in this order:
1. **Context instruction** (from "Context Files" section above — tells Cascade to read files)
2. **Shared Preamble** (the big block above — Role, Task, Procedure, Atomicity Rule, Output Format)
3. **Session-specific block** (below — CR section + guidance)

### Session 1: Chapter 1 — Game Concepts (CR 100-199)
```
[CONTEXT INSTRUCTION + SHARED PREAMBLE]

# CR Section
Read `MTG-Rules/Chapter 1 - Game Concepts.txt` from the workspace.

# Session-Specific Guidance
This chapter covers foundational concepts (players, game objects, mana, colors, life, drawing, damage, etc.).

**Classification reminders for this chapter:**
- Do NOT default to PURE-DEF just because a rule "defines a concept." Apply the classification decision process from the preamble to every rule. Many rules in this chapter define categories, boundaries, or behaviors that are TESTABLE or BOUNDARY-DEF.
- Rules with Examples in the CR text are almost certainly TESTABLE — the example IS a test scenario.
- Meta-rules that apply across multiple game systems (e.g., 101.2 "can't overrides can", 101.1 "card text overrides rules") should be classified as META with deferred concrete tests per the preamble instructions.

**Key areas to watch:**
- Mana rules (106.x): spending restrictions, color identity, mana abilities — these define engine behavior, not just vocabulary.
- Object type boundaries (110.x, 111.x): "permanent card", "spell", "token" — these are BOUNDARY-DEF if the engine needs a filter/predicate for the category.
- Damage rules (120.x): assignment, prevention, lethal — observable state changes.
- Life total rules (119.x): life gain/loss, starting life, life as a cost — observable state changes.
- Drawing rules (121.x): replacement effects on draws, empty library — observable state changes.

Cross-reference against Phase 5-Pre tickets (T01-T22) for data model rules, and the existing engine implementation for turn/priority rules.
```

### Session 2: Chapter 2 — Parts of a Card (CR 200-299)
```
[CONTEXT INSTRUCTION + SHARED PREAMBLE]

# CR Section
Read `MTG-Rules/Chapter 2 - Parts of a Card.txt` from the workspace.

# Session-Specific Guidance
This chapter defines card characteristics (name, mana cost, color, type, etc.).

**Classification reminders for this chapter:**
- Do NOT default to PURE-DEF just because a rule "defines a characteristic." Many characteristic rules define computed behavior or boundaries that are TESTABLE or BOUNDARY-DEF.
- If a rule defines HOW a characteristic is determined (e.g., CMC calculation, color from mana cost), it is TESTABLE — the computation can be wrong.
- If a rule defines WHAT belongs to a category (e.g., which types are supertypes vs subtypes), it is BOUNDARY-DEF.
- Rules with Examples in the CR text are almost certainly TESTABLE.

**Key areas to watch:**
- CMC/mana value calculation rules: computed values, edge cases (X spells, split cards, MDFCs) — TESTABLE.
- Color determination from mana cost and color indicator — TESTABLE (T05).
- Color identity rules (for Commander — Phase 9) — TESTABLE.
- How characteristics change in different zones — TESTABLE.
- Type/subtype/supertype boundaries — BOUNDARY-DEF.

Cross-reference against T05 (color_indicator) and Phase 5 layer system tickets (L01-L21) for characteristic computation.
```

### Session 3: Chapter 3 — Card Types (CR 300-399)
```
[CONTEXT INSTRUCTION + SHARED PREAMBLE]

# CR Section
Read `MTG-Rules/Chapter 3 - Card Types.txt` from the workspace.

# Session-Specific Guidance
This chapter defines card types and their rules.

**CRITICAL — Multi-clause sub-rules:**
Many sub-rules in this chapter pack MULTIPLE independent clauses into a single rule number (e.g., 301.5c contains 6 distinct behavioral clauses in one paragraph). You MUST:
- Read each sub-rule's ACTUAL TEXT from the file. Do NOT reconstruct rule text from memory.
- When a single sub-rule (e.g., 301.5c) contains multiple behavioral clauses, generate separate ATOM- tests for each clause, all under the SAME rule number (e.g., ATOM-301.5c-001, ATOM-301.5c-002, ...). Do NOT invent new sub-rule letters — the clauses belong to the rule number they appear under.
- Before writing any test, VERIFY the rule number by checking the literal text in the file. If you believe rule 301.5d says something about "equipping itself," re-read the file — that clause is in 301.5c.

**Classification reminders for this chapter:**
- Do NOT default to PURE-DEF just because a rule "defines a card type." Apply the classification decision process from the preamble to every rule. Many rules here define attachment legality, zone behavior, or state-based actions that are TESTABLE.
- Rules with Examples in the CR text are almost certainly TESTABLE — the example IS a test scenario.
- Boundary definitions are common here (e.g., what can/can't be equipped, what counts as a basic land type). These are BOUNDARY-DEF.

**Scope boundaries — what NOT to test here:**
- Combat mechanics (attacking, blocking, damage assignment) → tested in Sessions 4-5 via Chapter 5 rules. Only test combat-adjacent rules that are DEFINED in Chapter 3 (e.g., 302.7 "creature can attack and block" is Chapter 3; 508/509 declare attackers/blockers is Chapter 5).
- Casting/activation pipeline → tested in Session 5 via Chapter 6 rules. Only test casting-adjacent rules defined here (e.g., 304.4 "instants can be cast during other players' turns").
- Layer system type changes → tested in Session 6. Only test the definition of what types/subtypes exist, not how continuous effects change them.

**Key testable areas by rule range:**
- 300.1-300.2: Card type enumeration — BOUNDARY-DEF (which types exist, shared/exclusive)
- 301.x Artifacts: Equipment attachment rules (301.5a-f, many clauses per sub-rule), Fortification (301.6), Vehicles (301.7) — mostly TESTABLE
- 302.x Creatures: summoning sickness (302.6), P/T (302.8 — cross-ref 208), creature types — mix of TESTABLE and ALREADY-IMPLEMENTED
- 303.x Enchantments: Aura attachment (303.4a-j — T15, T15b), Aura legality (303.4b on resolution), Aura falling off (303.4c — SBA), Saga counters (303.4d-e) — heavily TESTABLE
- 304.x Instants: timing rules (304.4-5) — some ALREADY-IMPLEMENTED
- 305.x Lands: land play rules (305.1-4), basic land type mana abilities (305.7 — critical for Blood Moon/L17) — mix of TESTABLE and ALREADY-IMPLEMENTED
- 306.x Planeswalkers: loyalty activation (306.5-7 — T14, T19), uniqueness rule removed (306.4 is obsolete), damage redirection — TESTABLE
- 307.x Sorceries: timing (307.4-5) — some ALREADY-IMPLEMENTED
- 308.x Tribals: DEFERRED — Phase 8 (tribal is deprecated but legacy cards exist)
- 309.x Planes, 310.x Phenomena: OUT-OF-SCOPE (Planechase-only)
- 311.x Vanguard: DEFERRED (Vanguard support is a stretch goal)
- 312.x Schemes: OUT-OF-SCOPE (Archenemy-only)
- 313.x Conspiracies: OUT-OF-SCOPE (Conspiracy draft-only)
- 314.x Dungeons: DEFERRED — Phase 8-9
- 315.x Battles: DEFERRED — Phase 8-9

Cross-reference against T14 (planeswalker counters/SBAs), T15/T15b (Aura attachment/legality), T21a-d (combat — but most combat tests belong in Sessions 4-5), and Phase 5 layer tickets for 305.7 (L17 Blood Moon).
```

### Session 4: Chapters 4+5 — Zones + Turn Structure (CR 400-599)
```
[CONTEXT INSTRUCTION + SHARED PREAMBLE]

# CR Section
Read both files from the workspace:
1. `MTG-Rules/Chapter 4 - Zones.txt`
2. `MTG-Rules/Chapter 5 - Turn Structure.txt`

# Session-Specific Guidance

**Classification reminders for this session:**
- Do NOT default to PURE-DEF for zone rules. Many zone rules define observable transitions (object moves, identity changes, visibility) that are TESTABLE.
- Do NOT default to ALREADY-IMPLEMENTED for turn structure rules without verifying the implementation is complete. Many combat sub-steps and cleanup details are NOT yet implemented.
- Rules with Examples in the CR text are almost certainly TESTABLE.

**Scope boundaries — what NOT to test here:**
- Casting/activation pipeline details (601.2 steps) → Session 5. Only test timing rules defined in Chapter 5 (e.g., 505.5 "priority during main phase").
- Layer system and continuous effects → Session 6. Only test zone-related state changes.
- Specific keyword interactions during combat → Sessions 7-8. Only test the combat step structure itself.

**Chapter 4 — Zones (400-408):**
- 400.1-400.6: Zone fundamentals — mix of PURE-DEF and TESTABLE. 400.6 (zone-change replacement effects) and 400.7 (new object identity on zone change) are critical TESTABLE rules.
- 400.7a-d: Exceptions to new-identity rule (Auras, Equipment, counters, prevention effects) — all TESTABLE.
- 401.x Library: face-down, ordering, searching — some ALREADY-IMPLEMENTED, some TESTABLE.
- 402.x Hand: visibility, hand size — some ALREADY-IMPLEMENTED.
- 403.x Battlefield: phasing (403.3-403.6 — DEFERRED Phase 8), ETB/LTB events — TESTABLE.
- 404.x Graveyard: face-up, ordering, owner-specific — some ALREADY-IMPLEMENTED.
- 405.x Stack: LIFO, object removal on resolution — some ALREADY-IMPLEMENTED, verify completeness.
- 406.x Exile: face-up default, face-down exile — TESTABLE for face-down mechanics.
- 407.x Ante: OUT-OF-SCOPE (permanently excluded, banned in all sanctioned play).
- 408.x Command zone: DEFERRED — Phase 9 (Commander), emblems.

**Chapter 5 — Turn Structure (500-514):**
- 500.x-501.x: Phase/step structure — mostly PURE-DEF or ALREADY-IMPLEMENTED. Verify turn order is correct.
- 502.x Untap step: untap all, no priority — ALREADY-IMPLEMENTED, verify.
- 503.x Upkeep: trigger timing, priority — partially ALREADY-IMPLEMENTED (Phase 7 triggers are NOT).
- 504.x Draw step: draw TBA, trigger timing — partially ALREADY-IMPLEMENTED.
- 505.x Main phase: land play timing, sorcery-speed timing — partially ALREADY-IMPLEMENTED.
- 506.x-507.x Combat phase begin: attack requirements, restrictions — TESTABLE. 506.4 (skip combat if no creatures) is a common source of bugs.
- 508.x Declare attackers: step-by-step procedure, 508.8 (skip sub-steps if no attacks) — partially ALREADY-IMPLEMENTED (508.8 done, but verify sub-step details).
- 509.x Declare blockers: step-by-step procedure, multiple blockers — partially ALREADY-IMPLEMENTED.
- 510.x Combat damage: assignment rules, 2025 rule changes (no ordering), first strike — partially ALREADY-IMPLEMENTED.
- 511.x End of combat: trigger timing, removal from combat — TESTABLE for edge cases.
- 512.x-513.x End step: trigger timing, "at the beginning of the next end step" — Phase 7.
- 514.x Cleanup: discard to hand size, damage removal, cleanup re-loop (514.3a — T16) — partially ALREADY-IMPLEMENTED.

Cross-reference against existing implementation (Phases 1-4.5), T16 (cleanup re-loop), T21a-T21d (combat sub-steps), and Phase 7 (trigger timing at step boundaries).
```

### Session 5: Chapter 6, Part 1 — Spells, Casting, Resolving (CR 600-608)
```
[CONTEXT INSTRUCTION + SHARED PREAMBLE]

# CR Section
Read `MTG-Rules/LLM-Chapter-Splits/ch6-pt-1.txt` from the workspace. This file contains ONLY rules 600 through 608 (the casting/activation/resolution pipeline). Do NOT read the full Chapter 6 file — Session 6 covers the remainder.

# Session-Specific Guidance
This covers the casting pipeline and resolution — the most mechanically dense part of the rules.

**Classification reminders for this session:**
- Nearly every sub-rule in 601.2 (a through i) is TESTABLE — each step of the casting pipeline can fail independently and has distinct observable effects.
- Do NOT skip 601.2 sub-rules as PURE-DEF. Each defines a mandatory step in a sequential procedure. Getting the ORDER wrong (e.g., choosing targets before choosing modes) is a testable bug.
- Triggered ability rules (603.x) are mostly Phase 7 but are still TESTABLE — classify them with their target phase.
- Rules with Examples in the CR text are almost certainly TESTABLE.

**Scope boundaries — what NOT to test here:**
- Continuous effects, layers, replacement effects (609-616) → Session 6. Only test how spells/abilities RESOLVE, not how ongoing effects are applied.
- Specific keyword abilities (702.x) → Sessions 7-8. Only test the general framework for abilities.
- SBA checks during resolution → Session 9 (704.x). Only test resolution outcome, not SBA processing.

**Key testable areas by rule range:**
- 600.x: General spell/ability framework — mostly PURE-DEF with some TESTABLE (600.2 stack interaction basics).
- 601.x Casting spells: THE most test-dense section. 601.2a-i is a 9-step procedure — each step needs at least one ATOM. 601.3 timing/flash (T18). 601.4-5 alternative/additional costs. Key edge cases: 601.2e cost locking, 601.2f mana abilities during casting, 601.2g paying costs.
- 602.x Activating abilities: parallel to 601 for abilities (T19, T20). 602.2 activation restrictions (sorcery-speed, once-per-turn). 602.3 activation procedure steps.
- 603.x Triggered abilities: trigger conditions (603.2), trigger timing (603.3-603.4), "when/whenever/at" distinctions (603.1), intervening-if (603.4), leaves-the-battlefield triggers (603.6c-d), Phase 7 scope but generate tests now.
- 604.x Static abilities: CDAs (604.3 — L01), characteristic-setting (604.3a-b), prevention/restriction static abilities. CDAs are critical for layers.
- 605.x Mana abilities: definition (605.1a-b), special timing (605.3), triggered mana abilities (605.5). Some ALREADY-IMPLEMENTED — verify.
- 606.x Loyalty abilities: activation procedure, one-per-turn — TESTABLE (T14, T19).
- 607.x Linked abilities: linked pairs (607.1-607.2), ETB/activated links, imprint — TESTABLE (T20).
- 608.x Resolving: resolution procedure (608.2a-c), fizzle on all-illegal-targets (608.2b), partial resolution, modal spell resolution. Several sub-rules ALREADY-IMPLEMENTED — verify.

Cross-reference against T17-T20 (casting/activation), Phase 7 (triggers), L01 (CDAs), and existing cast.rs/stack.rs implementation.
```

### Session 6: Chapter 6, Part 2 — Effects, Layers, Replacement (CR 609-616+)
```
[CONTEXT INSTRUCTION + SHARED PREAMBLE]

# CR Section
Read `MTG-Rules/LLM-Chapter-Splits/ch6-pt-2.txt` from the workspace. This file contains ONLY rules 609 onward (effects, layers, replacement/prevention). Do NOT read the full Chapter 6 file — Session 5 already covered 600-608.

# Session-Specific Guidance
This covers the layer system, continuous effects, and replacement effects — the architecturally hardest part of the engine.

**Classification reminders for this session:**
- Nearly EVERY sub-rule in 613.x (layers) is TESTABLE — each layer/sublayer defines a specific computation order, and getting the order wrong produces different observable results.
- Replacement effect rules (614.x) and prevention effect rules (615.x) define behavioral transformations — they are TESTABLE, not PURE-DEF.
- Rules with Examples in the CR text are almost certainly TESTABLE — layer/replacement examples are particularly valuable as test cases because the interactions are complex.
- This is the session where META-101.2 ("can't beats can") gets many concrete tests. Tag them with META-101.2.

**Scope boundaries — what NOT to test here:**
- Casting/activation procedures (601-602) → Session 5. Only test how effects are APPLIED after resolution.
- Specific keyword abilities that create continuous effects (702.x) → Sessions 7-8. Test the general layer/effect framework, not specific keyword implementations.
- SBA processing (704.x) → Session 9. Only test the effect system, not SBA checks that trigger afterward.

**Key testable areas by rule range:**
- 609.x Effects: one-shot vs continuous distinction (609.2-609.3) — TESTABLE for classification. 609.4 linked effects.
- 610.x One-shot effects: creation of continuous effects from one-shots (610.2-610.3) — TESTABLE.
- 611.x Continuous effects: duration tracking (611.2a-d — "until end of turn", "for as long as", etc.), interaction with zone changes (611.2c — effect ends when source leaves), timestamp ordering basics (611.3) — all TESTABLE.
- 612.x Text-changing effects: Layer 3 — TESTABLE (L03).
- 613.x Layer system: THIS IS THE MOST CRITICAL SECTION. Every layer (613.1a-g = Layers 1-7) and sublayer (613.3a-f = 7a-7f) needs tests. Key rules: 613.7 (timestamp ordering within a layer), 613.8 (dependency), 613.9 (continuous effect interaction with characteristics). Generate tests for EACH layer individually AND for inter-layer ordering.
  - Layer 1 (613.1a): Copy effects — L01
  - Layer 2 (613.1b): Control-changing effects — L03
  - Layer 3 (613.1c): Text-changing effects — L03
  - Layer 4 (613.1d): Type-changing effects — L10
  - Layer 5 (613.1e): Color-changing effects — L10
  - Layer 6 (613.1f): Ability-adding/removing effects — L06
  - Layer 7 (613.1g): P/T effects — sublayers 7a-7f (L04, L07, L08)
  - Dependency (613.8): circular dependency, non-circular dependency — L09
- 614.x Replacement effects: "instead" keyword (614.1), self-replacement (614.6), one-replacement-per-event (614.5), ETB replacement (614.12 — look-ahead), "as [this] enters" (614.12a) — all Phase 6, heavily TESTABLE.
- 615.x Prevention effects: damage prevention (615.1-615.5), "prevent the next N damage" (615.3), prevention shields — Phase 6, TESTABLE.
- 616.x Multiple replacement/prevention: ordering when multiple apply (616.1), controller chooses (616.1a-b) — Phase 6, TESTABLE.

Cross-reference against L01-L21 (Phase 5 layers), Phase 6 (replacement/prevention) in roadmap.md, and META-101.2 (can't beats can).
```

### Session 7A: Chapter 7 — General Additional Rules + Keyword Actions (CR 700.x-701.x)
```
[CONTEXT INSTRUCTION + SHARED PREAMBLE]

# CR Section
Read `MTG-Rules/LLM-Chapter-Splits/ch7-pt-1.txt` from the workspace. This file contains rules 700 through 701.68 (general additional rules and keyword actions). Sessions 7B, 8, 9A, 9B cover the rest of Chapter 7.

# Session-Specific Guidance
**Do NOT auto-generate a summary.** Output ONLY ATOM-/COMP- test blocks, classification table, and gap report. Summary is a separate step.

**Scope:** 700.x general rules + 701.x keyword actions. No keyword abilities (702.x) — those are Session 7B/8.

**Cross-session rules (700.x):**
700.x is the CR's catch-all. Several rules architecturally belong to earlier sessions' systems. Do NOT defer them — classify by what they DO:
- **700.2 (modal spells)** → Part of 601.2b casting pipeline. `choose_modes` DP method, mode-conditional targeting (700.2c), opponent mode choice (700.2e), mode-locked copies (700.2g) = Phase 5-Pre T18. The effect *resolver* for `Effect::Modal` = Phase 8. Classify each sub-rule individually.
- **700.3 (piles)** → Resolution-time behavior (Fact or Fiction). Phase 8.
- **700.7 ("this [something]")** → Part of 603.x triggered ability object identity. Phase 7.
- **700.x batch predicates** (devotion, historic, party, modified, etc.) → Genuinely Phase 8. DEFERRED.

**701.x Keyword Actions — Tiered classification:**
The file defines ~68 keyword actions. Do NOT mass-defer. Read each one, look up its actual rule number (numbering has gaps), and classify:

1. **ALREADY-IMPLEMENTED:** Activate (→602), Cast (→601), Counter, basic Destroy, basic Discard, Exile, Play (land), Shuffle, Tap/Untap. For these, verify ALL sub-rules — some add behavior beyond the basic action (e.g., Destroy's distinction from other graveyard moves needs Phase 7 delta flag; Discard has random variant for Phase 8).
2. **TESTABLE Phase 5-Pre/5/6/7:** Attach/Unattach (T15/T15b), Sacrifice (bypasses regen/indestructible), Double/Triple (layer 7c), Regenerate (replacement effect, Phase 6), Exchange (control/life/P/T swaps). Full ATOM tests.
3. **TESTABLE Phase 8:** Create (tokens), Fight, Mill, Scry, Search, Surveil, Reveal, and similar. Full ATOM tests tagged Phase 8.
4. **DEFERRED Phase 9:** Goad, Transform, Convert, Meld, Face a Villainous Choice. One-line entries.
5. **DEFERRED Phase 8:** Niche actions (fateseal, clash, proliferate, detain, populate, monstrosity, bolster, manifest, exert, explore, amass, connive, incubate, discover, cloak, etc.). One-line entries.
6. **OUT-OF-SCOPE:** Planeswalk, Set in Motion/Abandon, Assemble, Open an Attraction/Roll to Visit.

**Scope boundaries:**
- Layer interactions → Session 6. Test only the action's own behavior.
- SBA processing → Session 9A. Test only the action's direct effect.
- 601.2 casting procedure → Session 5. But 700.x rules that *define* casting-time behavior ARE in scope here.

Cross-reference: T15/T15b (attach), T17 (cost modification), T18 (flash/modal), Phase 7 (delta log), Phase 8 keyword list in roadmap.md.

Write output to `plans/atomic-tests/sessions/session-7a.md`.
```

### Session 7B: Chapter 7 — Keyword Abilities: Deathtouch through Wither (CR 702.1-702.80)
```
[CONTEXT INSTRUCTION + SHARED PREAMBLE]

# CR Section
Read `MTG-Rules/LLM-Chapter-Splits/ch7-pt-2.txt` from the workspace. This file contains rules 702.1 through 702.80 (keyword abilities Deathtouch through Wither). Session 7A covered 700-701; Session 8 covers 702.81+.

# Session-Specific Guidance
**Do NOT auto-generate a summary.** Output ONLY ATOM-/COMP- test blocks, classification table, and gap report. Summary is a separate step.

**10 ALREADY-IMPLEMENTED keywords (Phases 1-4) — audit each sub-rule:**
The engine already implements these keywords. For each, check EVERY sub-rule — some have unimplemented sub-rules that still need ATOM tests:

- **Deathtouch (702.2):** 702.2a-c ALREADY-IMPLEMENTED. 702.2d (functions from any zone) — DEFERRED Phase 6, note dependency. 702.2e (last known info on zone change) — DEFERRED Phase 5 epoch model. 702.2f (redundancy) — ALREADY-IMPLEMENTED.
- **Defender (702.3):** All sub-rules ALREADY-IMPLEMENTED. No gaps.
- **Double Strike (702.4):** 702.4a-b, 702.4e ALREADY-IMPLEMENTED. 702.4c (removing DS stops second step) — DEFERRED Phase 5 continuous effects. 702.4d (giving DS after first step) — DEFERRED Phase 5.
- **First Strike (702.7):** 702.7a-b, 702.7d ALREADY-IMPLEMENTED. 702.7c (gain/remove FS mid-combat) — DEFERRED Phase 5.
- **Flying (702.9):** All sub-rules ALREADY-IMPLEMENTED. No gaps.
- **Haste (702.10):** All sub-rules ALREADY-IMPLEMENTED. No gaps.
- **Lifelink (702.15):** 702.15a-b, 702.15e-f ALREADY-IMPLEMENTED. 702.15c (LKI on zone change) — DEFERRED Phase 5 epoch. 702.15d (functions from any zone) — DEFERRED Phase 6.
- **Reach (702.17):** All sub-rules ALREADY-IMPLEMENTED. No gaps.
- **Trample (702.19):** 702.19a-b, 702.19g ALREADY-IMPLEMENTED. 702.19c (trample over planeswalkers) — DEFERRED Phase 8. 702.19d (all blockers removed before damage → all to defender) — generate ATOM test, engine probably handles this but needs explicit coverage. 702.19e-f (PW removed from combat) — DEFERRED Phase 8.
- **Vigilance (702.20):** All sub-rules ALREADY-IMPLEMENTED. No gaps.

For unimplemented sub-rules, generate ATOM tests tagged with the appropriate phase. For ALREADY-IMPLEMENTED sub-rules, list them in the classification table but do NOT generate tests.

**Other keywords in 702.1-702.80 — classification guidance:**
- Phase 5-Pre (existing tickets): Enchant → T15, Equip → T15b, Flash → T18, Hexproof → T22, Indestructible → T09, Protection → T22, Shroud → T22
- Phase 7: Ward (triggered ability "when targeted, counter unless pay [cost]") — NEW ticket
- Phase 8 (commonly played — full ATOM tests): Kicker, Flashback, Morph, Storm, Cycling, Phasing, Suspend, Persist, Wither, Split Second, Convoke, Delve, Affinity, Evoke, Changeling
- Phase 8 (niche — DEFERRED one-liner): Banding (extremely complex, low priority), Rampage, Cumulative Upkeep, Flanking, Fading, Echo, Horsemanship, Shadow, Buyback, Madness
- Cost-modification keywords (Kicker, Affinity, Convoke, Delve, Buyback) → cross-reference T17

**Scope boundaries:**
- Layer interactions → Session 6. Test only the keyword's own behavior.
- SBA processing → Session 9A.
- Keyword actions (destroy, sacrifice, etc.) → Session 7A.
- Keywords 702.81+ → Session 8.

Cross-reference: T09, T15/T15b, T17, T18, T22, Phase 7 (ward), Phase 8 keyword list in roadmap.md.

Write output to `plans/atomic-tests/sessions/session-7b.md`.
```

### Session 8: Chapter 7 — Keyword Abilities: Retrace through Sneak (CR 702.81-702.190)
```
[CONTEXT INSTRUCTION + SHARED PREAMBLE]

# CR Section
Read `MTG-Rules/LLM-Chapter-Splits/ch7-pt-3.txt` from the workspace. This file contains rules 702.81 through 702.190 (keyword abilities Retrace through Sneak). Session 7B covered 702.1-702.80; Sessions 9A-9B cover 703+.

# Session-Specific Guidance
**Do NOT auto-generate a summary.** Output ONLY ATOM-/COMP- test blocks, classification table, and gap report.

No keywords in this range are ALREADY-IMPLEMENTED. Many are niche or format-specific.

**Classification guidance:**
- **OUT-OF-SCOPE:** Keywords only in supplemental products (Planechase, Archenemy, Un-sets, Conspiracy).
- **DEFERRED Phase 9:** DFC-related keywords (Daybound/Nightbound, Disturb, More Than Meets the Eye, Prototype).
- **Cost-modification keywords** (Convoke, Delve, Improvise, Emerge, Assist, Undaunted, Spectacle, Escape, Foretell) → These modify 601.2f total cost. Classify as Phase 5-Pre T17, not Phase 8.
- **Triggered-ability keywords** (Cascade, Storm, Encore, Afterlife, Afflict, Fabricate, Exploit, Melee, Myriad, Backup) → Phase 7 dependency. Note it in the test.

**Keywords to prioritize (full ATOM tests):**
Cascade, Overload, Bestow, Dash, Crew (→301.7), Menace, Prowess, Escape, Mutate, Casualty, Blitz, Bargain, Reconfigure (→301.5c)

**Scope boundaries:**
- Layer interactions → Session 6. Keyword actions → Session 7A.
- 601.2 casting procedure → Session 5. But keywords defining casting/cost behavior ARE in scope.

**Chunked output plan:** ~110 keywords. Use ≤15 keywords per chunk.
- **Chunk 0:** Read file, count keywords, announce chunk plan. STOP.
- **Chunks 1-N:** Process ≤15 keywords each. STOP after each.
- **Final chunk:** Classification Summary, Composition Tests, Gap Report. STOP.

Cross-reference: Phase 8 keyword list in roadmap.md, T17 (cost modification).

Write output to `plans/atomic-tests/sessions/session-8.md`.
```

### Session 9A: Chapter 7 — TBAs, SBAs, Copying, Card Variants (CR 703-712)
```
[CONTEXT INSTRUCTION + SHARED PREAMBLE]

# CR Section
Read `MTG-Rules/LLM-Chapter-Splits/ch7-pt-4.txt` from the workspace. This file contains rules 703 through 712 (turn-based actions, state-based actions, coin flipping, die rolling, copying, face-down spells, split/flip/leveler/DFC/saga/adventure/class/case cards, meld). Session 9B covers 713-732.

# Session-Specific Guidance
**Do NOT auto-generate a summary.** Output ONLY ATOM-/COMP- test blocks, classification table, and gap report.

**CRITICAL: Rule numbers have shifted in recent CR updates.** New sections (coin flipping, die rolling, Saga, Adventure, Class, Case, etc.) were inserted. You MUST read actual rule numbers from the file. Do NOT assume any number.

**Cross-session rules — classify by what they DO, not where they "belong":**
- **703.x TBAs** → Session 4 tested step structure; this session tests the TBA *definitions* (703.4 list).
- **704.x SBAs** → Session 1 tested the *concept*; this session tests each individual 704.5x check. The authoritative SBA list is HERE.
- **Copying rules** → Session 6 tested how copy effects apply in layers; this session tests the copy *procedure* (copiable values, copy+modifications).

**State-Based Actions (704.x) — most critical section:**
Each 704.5x sub-rule is TESTABLE. Use these facts:
- ALREADY-IMPLEMENTED: 0 life (704.5a), empty library draw (704.5b), toughness ≤ 0 (704.5f), lethal damage (704.5g), deathtouch damage (704.5g)
- Phase 5-Pre: poison counters (T16), PW 0 loyalty (T14/T16), legend rule (T14), Aura SBA (T15), Equipment SBA (T15b), +1/+1 and -1/-1 annihilation (T16)
- Phase 6: copy not on battlefield/stack → cease to exist
- Phase 8: token not on battlefield → cease to exist, world rule
- Classify remaining 704.5x sub-rules case-by-case

**Card type variant rules:**
- Face-down (morph/manifest/disguise/cloak) — DEFERRED Phase 8-9
- Split cards — DEFERRED Phase 9 (D4)
- Flip cards — OUT-OF-SCOPE (Kamigawa-only)
- Double-faced cards — DEFERRED Phase 9 (D3)
- Saga, Adventure, Class, Case, Omen, Prototype, Station — DEFERRED Phase 8-9 (classify individually)
- Meld — DEFERRED Phase 9
- Leveler — DEFERRED Phase 9

**Scope boundaries:**
- Layer system application of copy effects → Session 6 (Layer 1).
- Keyword definitions → Sessions 7A/7B/8.
- 601.2 casting procedure → Session 5. Resolution-time behavior defined here IS in scope.

**Chunked output plan:**
- **Chunk 0:** Read file, count rule sections, announce plan. STOP.
- **Chunk 1:** 703.x TBAs + 704.x SBAs (most critical — take the space needed). STOP.
- **Chunk 2:** Copying rules + coin/die rolling. STOP.
- **Chunk 3:** Card type variant rules (face-down through meld). Many DEFERRED one-liners. STOP.
- **Chunk 4 (final):** Classification Summary, Composition Tests, Gap Report. STOP.

Cross-reference: T13-T16 (SBAs), Phase 6 (copy), Phase 7 (triggers), Phase 9 (DFCs).

Write output to `plans/atomic-tests/sessions/session-9a.md`.
```

### Session 9B: Chapter 7 — Card Type Variants, Game-Altering Mechanics, Shortcuts, Illegal Actions (CR 713-732)
```
[CONTEXT INSTRUCTION + SHARED PREAMBLE]

# CR Section
Read `MTG-Rules/LLM-Chapter-Splits/ch7-pt-5.txt` from the workspace. This file contains rules 713 through 732 (substitute cards, Saga cards, Adventurer cards, Class cards, Attraction cards, Prototype cards, Case cards, Omen cards, Station cards, controlling another player, ending turns/phases, restarting game, Monarch, Initiative, rad counters, subgames, merging with permanents, day/night, shortcuts, handling illegal actions). Session 9A covered 703-712.

# Session-Specific Guidance
**Do NOT auto-generate a summary.** Output ONLY ATOM-/COMP- test blocks, classification table, and gap report.

**CRITICAL: Read actual rule numbers from the file.** Do NOT assume numbering.

**CRITICAL — DEFERRED ≠ skip.** A rule classified as DEFERRED still gets a full ATOM test specification if it is TESTABLE. "DEFERRED — Phase N" means the *implementation* is in Phase N; the *test spec* is written NOW. The only rules that get one-liner treatment are PURE-DEF (no observable mechanical consequence) and OUT-OF-SCOPE (permanently excluded). If a rule defines an observable behavior, state change, trigger condition, SBA, zone-dependent characteristic, or engine decision point, it is TESTABLE — write the full ATOM block with the appropriate Phase tag.

**Classification reminders for this session:**
This file is dominated by card type variant rules (Saga, Adventure, Class, Prototype, Case, Omen, Station) and game-altering mechanics (Monarch, Initiative, end-the-turn, Day/Night). Do NOT blanket-defer entire rule sections. Read each sub-rule individually. Many sub-rules within these systems define specific, independently testable behaviors:
- **Trigger conditions** — a wrong trigger template produces different game state
- **State-based actions** — a missing SBA leaves a permanent alive that should be gone
- **Zone-dependent characteristics** — wrong characteristics on stack/battlefield affect legality, targeting, and resolution
- **Alternative casting semantics** — wrong cast-mode choice breaks the casting pipeline
- **Designation tracking** — wrong designation state produces incorrect ability grants
- **Copiable values** — wrong copy behavior propagates incorrect characteristics
- **Resolution-time zone destinations** — wrong zone after resolution (exile vs graveyard vs library)
- **Multi-step procedures** — "end the turn" is a sequential procedure where each step is independently testable

Rules with Examples in the CR text are almost certainly TESTABLE.

**Scope notes:**
- **713.x (Substitute Cards):** OUT-OF-SCOPE — physical play aid with no digital equivalent.
- **717.x (Attraction Cards):** OUT-OF-SCOPE — Un-set mechanic (Unfinity).
- **726.x (Restarting the Game):** Post-v1 — extremely niche (Karn Liberated only). One-liner entries acceptable.
- **728.x (Subgames):** Post-v1 — extremely niche (Shahrazad banned everywhere). One-liner entries acceptable.
- **Everything else (714-716, 718-725, 727, 729-732):** Classify sub-rules individually per the preamble. Most will be TESTABLE with a DEFERRED phase tag.

**Phase mapping context (consult roadmap.md for full details):**
- Sagas are unblocked by Phase 7 (triggered abilities) + Phase 8 (card breadth)
- Adventure/Omen/Split are Phase 9 (D4 CardLayout restructuring)
- Prototype is Phase 9 (CharacteristicOverrides on zone sidecars)
- Monarch/Initiative are Phase 7 (D15 — triggered abilities with no source)
- Day/Night is Phase 9 (D14 — DFC system)
- Merging/Mutate is Phase 9
- End-the-turn / End-combat-phase is Phase 8
- Controlling another player is Phase 8
- Shortcuts/loops are Phase 7 (D26 — GameNumber stub) / Phase 9 (full GameNumber)
- Handling illegal actions is Phase 8

**Scope boundaries — what NOT to test here:**
- Layer system interactions → Session 6. Test the card type's own zone/characteristic behavior.
- SBA processing pipeline → Session 9A. Test specific new SBAs defined here (e.g., Saga sacrifice).
- Casting pipeline steps (601.2) → Session 5. Test alternative casting MODE CHOICE defined here.
- Keyword abilities (702.x) → Sessions 7B/8. Test card type FRAME RULES defined here.

**Chunked output plan:**
- **Chunk 0 (planning):** Read file, count sections, announce plan. STOP.
- **Chunk 1:** Rules 713–721 (Substitute, Saga, Adventurer, Class, Attraction, Prototype, Case, Omen, Station). STOP.
- **Chunk 2:** Rules 722–727 (Controlling Another Player, Ending Turns/Phases, Monarch, Initiative, Restarting Game, Rad Counters). STOP.
- **Chunk 3:** Rules 728–732 (Subgames, Merging, Day/Night, Shortcuts, Illegal Actions). STOP.
- **Chunk 4 (final):** Classification Summary, Composition Tests, Gap Report. STOP.

Cross-reference: Phase 7 (triggered abilities, loops/D26, Monarch/Initiative D15), Phase 8 (card breadth, tokens), Phase 9 (DFCs, day/night D14, Adventure D4, Prototype, Mutate), Session 1 (win/lose 104.x), Session 9A (TBAs 703.x, SBAs 704.x, copying 707.x).

Write output to `plans/atomic-tests/sessions/session-9b.md`.
```

### Session 10: Chapters 8+9 — Multiplayer + Casual (CR 800-999)
```
[CONTEXT INSTRUCTION + SHARED PREAMBLE]

# CR Section
Read both files from the workspace:
1. `MTG-Rules/Chapter 8 - Multiplayer Rules.txt`
2. `MTG-Rules/Chapter 9 - Casual Variants.txt`

# Session-Specific Guidance
Most of this is either OUT-OF-SCOPE (permanently excluded supplemental formats) or DEFERRED (future phases). This session should be the shortest in terms of ATOM output.

**Do NOT auto-generate a summary.** Output ONLY ATOM-/COMP- test blocks, classification table, and gap report.

**VERIFY every rule number against the literal file text.** Do NOT guess from memory.

**Classification guidance:**
- Use the two-label distinction carefully here:
  - **OUT-OF-SCOPE** for formats the simulator will NEVER support: Planechase, Archenemy, Conspiracy draft, Un-set variants.
  - **DEFERRED — Phase N** for formats/rules planned for later phases: Commander (Phase 9), general multiplayer foundations (future), Two-Headed Giant (future).
- Commander (903.x) is the ONE major exception — it's Phase 9 scope. Classify Commander rules as **DEFERRED — Phase 9** and generate TESTABLE tests for them (they'll be implemented in Phase 9).
- General multiplayer rules (800.x) that would apply to Commander games should be classified as **DEFERRED — Phase 9** rather than OUT-OF-SCOPE, since Commander IS multiplayer.
- Rules with Examples in the CR text are almost certainly TESTABLE if they're in-scope.

**Scope boundaries:**
- Only generate ATOM tests for rules that are in Phase 9 Commander scope or that affect 2-player games.
- Do NOT generate tests for Planechase, Archenemy, Conspiracy, or Un-set variants (OUT-OF-SCOPE). Vanguard is DEFERRED (stretch goal) — classify as DEFERRED but don't generate tests now.
- Two-Headed Giant: DEFERRED (future multiplayer support) — classify but don't generate tests now.

**Key testable areas (Commander — 903.x) — verify numbers from the file:**
- Commander designation (command zone placement) — TESTABLE
- Color identity computation (mana cost + color indicators + mana symbols in rules text) — TESTABLE
- Deck construction (100-card singleton, color identity restriction) — TESTABLE
- Commander in command zone at game start — TESTABLE
- Starting life = 40 — TESTABLE
- Commander can be cast from command zone — TESTABLE
- Commander tax (+{2} per previous cast from command zone) — TESTABLE
- Commander tax applies to zone transitions through command zone — TESTABLE
- Commander dies/exiled → may return to command zone (replacement effect) — TESTABLE
- Commander damage tracking (21+ → lose) — TESTABLE (T16 cross-ref)

**Other in-scope rules from Chapter 8 — verify numbers from the file:**
- Player-leaves-game cleanup (800.4x) — DEFERRED Phase 9 (Commander is multiplayer, players can leave)
- Attack Multiple Players option (802.x) — DEFERRED Phase 9
- APNAP order is defined in 101.4 (Chapter 1, Session 1) — cross-ref META only, do not re-test here

**Chunked output plan (mandatory — see shared preamble):**
This session is the smallest, but still chunk to avoid timeouts:
- **Chunk 0:** Read files, count top-level rule sections, confirm chunk plan. Write session header. STOP.
- **Chunk 1:** Chapter 8 — Multiplayer rules (800.x-802.x). Most will be DEFERRED one-liners. STOP.
- **Chunk 2:** Chapter 9 — Commander (903.x) + other casual variants (900-902, 904+). Commander rules get full ATOM tests; others are mostly OUT-OF-SCOPE or DEFERRED. STOP.
- **Chunk 3 (final):** Classification Summary Table, Composition Tests, Gap Report. STOP.

Cross-reference against Phase 9 in roadmap.md and T16 (commander damage SBA).
```

---

## Post-Session Condensation Step

After each session is complete and audited, produce a **condensed summary** in `plans/atomic-tests/summaries/session-N-summary.md`. This is what the merge sessions consume (the full session files are too large to merge in one context window).

The condensed summary contains ONLY:

1. **ATOM index** — One line per test: `ATOM-XXX.X-NNN | Rule summary | Phase | Ticket | Tags`
2. **BOUNDARY-DEF index** — Same format for boundary tests
3. **COMP index** — Same format for composition tests, plus which ATOMs they compose
4. **META entries** — Full META blocks (these are small)
5. **Classification summary table** — The full table from the session
6. **NEW tickets list** — The full new-tickets table
7. **Gap report** — The full gap report
8. **ALREADY-IMPLEMENTED list** — Rule numbers only (no descriptions needed)
9. **OUT-OF-SCOPE list** — Rule numbers and one-line reasons (permanently excluded)
10. **DEFERRED list** — Rule numbers, target phase, and one-line reasons (planned for later phases)

Do NOT include: full board states, full expected results, full action descriptions, architectural notes, or audit response text. Those stay in the full `session-N.md` file. The summary should be ~2-5k words, not ~30k.

Format for the ATOM index:
```
| ID | Rule | Summary | Phase | Ticket | Tags |
|----|------|---------|-------|--------|------|
| ATOM-100.2a-001 | 100.2a | Constructed deck min 60 cards | Phase 5-Pre | GameConfig | |
| ATOM-100.2a-002 | 100.2a | No more than 4 copies non-basic | Phase 5-Pre | GameConfig | |
```

---

## Merge Process

### Design Principles

1. **The merge output must be self-contained per phase.** During implementation, you load the phase catalog for the phase you're working on + the CR text for rule details. You should NOT need to open any session file or session summary.
2. **Full session files are reference-of-last-resort.** They contain the LLM's reasoning about edge cases. If a test spec in the phase catalog seems incomplete and the CR text doesn't clarify, check the session file — but this should be rare.
3. **Session summaries are intermediate artifacts.** They exist to make the merge feasible. After the merge is complete, they can be archived alongside the full session files.
4. **The CR is the canonical source for rule details.** Every ATOM carries a rule number. When implementing, read the rule from the CR text, not from the session's paraphrase.

### Pre-Merge: Compress Summaries

The 12 session summaries total ~380KB — borderline for a single context window that also needs to do real reasoning. Before running the merge passes, produce a compressed input file.

Run once with all summaries loaded. This is a **mechanical extraction**, not an analytical task, so context pressure is manageable.

```
# Role
Senior Rust engineer, MTG rules expert.

# Task
Read ALL session summary files: plans/atomic-tests/summaries/session-{1,2,3,4,5,6,7a,7b,8,9a,9b,10}-summary.md.

Produce a single compressed file that retains ALL test IDs and their metadata but strips everything else. This is a MECHANICAL task — do not analyze, de-duplicate, or reorganize.

## Output Format

### Section 1: Master ATOM Table
Combine all ATOM index tables from all summaries into ONE table. Keep every row exactly as-is. Add a "Session" column.

| ID | Rule | Summary | Phase | Ticket | Tags | Session |

### Section 2: Master BOUNDARY-DEF Table
Same format, all BOUNDARY-DEF entries.

### Section 3: Master COMP Table
Same format, all COMP entries. Keep the "Composes" column from each summary.

### Section 4: All META Entries
Copy every META block verbatim from every summary.

### Section 5: All NEW Tickets
Combine all NEW ticket tables. Add a "Session" column.

### Section 6: All Gap Reports
Concatenate all gap report sections. Prefix each with its session number.

### Section 7: Classification Totals
For each session, copy ONLY the totals row from the classification summary table:
| Session | TESTABLE | BOUNDARY-DEF | PURE-DEF | DEFERRED | OOS | IMPL |

### Section 8: DEFERRED Master List
Combine all DEFERRED lists (rule + phase + one-line reason). De-duplicate by rule number (keep the version with the most detail).

### Section 9: OUT-OF-SCOPE Master List
Combine all OUT-OF-SCOPE lists. De-duplicate by rule number.

### Section 10: ALREADY-IMPLEMENTED Master List
Combine all ALREADY-IMPLEMENTED rule number lists. De-duplicate.

**Hard rules:**
- Do NOT summarize or paraphrase any test. Copy the one-line summary exactly.
- Do NOT remove any test, even if it looks like a duplicate — that's Pass 1's job.
- Do NOT add analysis, commentary, or recommendations.
- Write output in chunks ≤4000 words per file-write call.

Output as `plans/atomic-tests/merge-input-compressed.md`.
```

### Pass 0: Cross-Session Dependency Map ✅ COMPLETED

**Status:** Generated, audited, and corrected. Output: `plans/atomic-tests/pass0-dependency-map.md`.

Key results:
- **25 true META entries** (session summaries over-tagged ~15 items as META that were actually cluster notes or implementation details — these were reclassified)
- **14 shared mechanism clusters** (A–N): Alt Cost, Additional Cost, Cost Reduction by Resource (3 sub-clusters), ETB+Store+Read, Dies+Return, Face-Down, Combat Attack Triggers, Evasion, GY-Activated, DFC/Transform, Replacement Effects, Text-Changing, Combat Swap, ETB Token+Attach
- **Merge-half split confirmed:** Sessions 1–6 (Half A, ~1050 ATOMs, foundation + layers) / Sessions 7A–10 (Half B, ~650 ATOMs, keywords + formats)
- ~17 cross-session duplicate groups identified with resolutions

### Pass 1: De-duplicate + Phase-Organize (two halves)

Run TWICE — once per half.
- **Half A:** Sessions 1, 2, 3, 4, 5, 6 (~1050 ATOMs). Foundation rules 100–616.
- **Half B:** Sessions 7a, 7b, 8, 9a, 9b, 10 (~650 ATOMs). Keywords 700–732, formats 800–903.

**Pre-split input files** (generated by `merge-sessions.py --half both`):
- Half A: `plans/atomic-tests/merge-input-half-A.md` (169KB, ~925 ATOMs)
- Half B: `plans/atomic-tests/merge-input-half-B.md` (158KB, ~763 ATOMs)

Each half file contains only its own sessions' ATOMs/BOUNDARY-DEFs/COMPs/META/tickets, plus the shared DEFERRED/OUT-OF-SCOPE/ALREADY-IMPLEMENTED lists.

**Each run reads:**
1. `plans/atomic-tests/merge-input-half-[A or B].md` — the pre-split input for your half
2. `plans/atomic-tests/pass0-dependency-map.md` — the audited dependency map (Sections 1-3 are most relevant: META table, duplicates, clusters)
3. `plans/roadmap.md`
4. `plans/implementation-plan-final.md`

Do NOT read the full `merge-input-compressed.md` — use your half file. Do NOT read session files or summaries.

```
# Role
Senior Rust engineer, MTG rules expert.

# Task — Pass 1, Half [A or B]

You are processing [Half A: sessions 1–6 | Half B: sessions 7a–10].

Read these files:
1. plans/atomic-tests/merge-input-half-[A or B].md (your pre-split input — contains ONLY your half's ATOMs/BOUNDARY-DEFs/COMPs/META/tickets, plus shared DEFERRED/OUT-OF-SCOPE/ALREADY-IMPLEMENTED)
2. plans/atomic-tests/pass0-dependency-map.md
3. plans/roadmap.md
4. plans/implementation-plan-final.md

Do NOT read merge-input-compressed.md or any session/summary files.

## Step 1: De-duplicate within this half
Using pass0's Section 2 (Cross-Session Duplicates) as a starting point, resolve duplicates within your sessions. Keep the more detailed/specific version. Produce a short table of removed duplicates with one-line rationale.

## Step 2: Organize by phase
Group all surviving ATOM/BOUNDARY/COMP entries into:
- Phase 5-Pre (T01–T22, T15b, T21a–T21d)
- Phase 5-Layers (L01–L21)
- Phase 6 (Replacement Effects — 614, 615, 616)
- Phase 7 (Triggered Abilities — 603, delayed triggers, state triggers)
- Phase 8 (Keywords & Breadth — 702.x, 701.x, tokens, emblems)
- Phase 9 (DFC, split cards, sagas, commander, multiplayer)
- Post-v1 (restart, subgames, niche)
- ALREADY-IMPLEMENTED (completeness tracking only — list IDs, no enrichment needed)

## Step 3: Enrich each test entry
For every non-ALREADY-IMPLEMENTED ATOM/BOUNDARY/COMP, expand the one-line summary into a self-contained test spec. Format (use this exact structure, one per test):

    ### ATOM-XXX.Xa-NNN
    - **Rule:** [number] — [one-line summary from CR]
    - **Board:** [1-2 sentence minimal game state]
    - **Action:** [what happens]
    - **Expected:** [correct outcome per CR]
    - **Ticket:** [T##/L##/NEW-###]
    - **Deps:** [ATOM IDs or mechanisms required as preconditions, if any]
    - **Cluster:** [pass0 cluster letter if applicable, e.g. "Cluster A (Alt Cost)"]

Reconstruct Board/Action/Expected from the rule number + your CR knowledge. The CR is the source of truth — do NOT try to recover session file wording. If a rule is ambiguous enough that you can't reconstruct the spec from rule number + summary alone, flag with `[NEEDS-REVIEW]`.

For BOUNDARY-DEF entries: use the same format but the Expected field should state the boundary condition being defined (e.g., "Engine must reject X" or "Enum must contain Y").

For COMP entries: list which ATOMs are composed and what cross-mechanism interaction is tested.

## Step 4: Cross-half annotations
Using pass0's clusters (Section 3) and META table (Section 1), produce a short appendix:
- Tests in THIS half that share a cluster with the OTHER half (list cluster letter + relevant ATOM IDs)
- META entries from THIS half whose concrete tests are in the OTHER half (or vice versa)
- Any cross-half duplicates not caught by pass0

## Step 5: NEW ticket consolidation
De-duplicate NEW tickets within this half. Produce a table:
| Ticket ID | Rule(s) | Summary | Phase | Cluster |

## Output
Write to `plans/atomic-tests/merge-pass1-half-[A or B].md`.

Write in chunks of ≤4000 words. Group by phase — one phase per chunk (split large phases across multiple chunks). Start each chunk with a phase header so the file is navigable.

Expected output size: Half A ~30-40k words (many foundation tests need enrichment), Half B ~20-30k words (keyword tests are more formulaic).
```

### Pass 2: Final Phase Catalogs (Multi-Stage)

The combined Pass 1 outputs (~200KB) + roadmap (~49KB) + implementation plan (~128KB) exceed the LLM context window. Pass 2 is split into four stages with script-assisted pre-filtering.

**Pre-requisite:** Run `extract-pass2-index.py` to generate input files:
```bash
cd plans/atomic-tests
python extract-pass2-index.py --index          # → pass2-global-index.md (~20KB)
python extract-pass2-index.py --phase all      # → pass2-phase-*.md (one per phase)
python extract-pass2-index.py --tickets        # → pass2-ticket-index.md (~6KB)
```

#### Stage 2A: Cross-Half De-duplication

Input: `pass2-global-index.md` (~20KB) + `pass0-dependency-map.md` (~27KB)
Output: `pass2-dedup-decisions.md`

```
# Role
Senior Rust engineer, MTG rules expert.

# Task
Read: plans/atomic-tests/pass2-global-index.md, plans/atomic-tests/pass0-dependency-map.md.

## Step 1: Cross-half de-duplication
Using the Cross-Half Annotations (Section 6 of global index) and pass0 Section 2 (Cross-Session Duplicates), resolve remaining cross-half overlaps. For each overlap:
- State which entry to KEEP (with half label) and which to DROP
- One-line rationale
- If complementary (not duplicate), state "KEEP BOTH — complementary" with reason

## Step 2: Cross-half COMP detection
Using pass0 Section 3 (Shared Mechanism Clusters), identify ATOM tests from different halves that should become COMP tests because they test a cross-half interaction. For each:
- List the ATOM IDs being composed
- State the interaction being tested
- Assign a COMP ID
- Assign to the later phase of the two ATOMs

## Step 3: Known issues from Pass 1 review
Resolve these specific items flagged during review:
- ATOM-701.19a-001 and ATOM-701.8c-001 (Half B Phase 6): near-identical regen tests → merge or differentiate
- ATOM-601.2a-002: appears in both Phase 5-Pre and Phase 5-Layers in Half A → assign to one phase
- Vanguard (902): listed as OUT-OF-SCOPE but project scope says "stretch goal" → reclassify

Output as plans/atomic-tests/pass2-dedup-decisions.md.
Format: markdown tables + one-line rationales. Keep concise — this is a decision log, not a catalog.
```

#### Stage 2B: Per-Phase Catalog (run once per phase)

Run for each phase: Phase 5-Pre, Phase 5-Layers, Phase 6, Phase 7, Phase 8, Phase 9.

> **Note:** `pass2-ticket-index.md` only covers Phase 5-Pre (T01–T22) and Phase 5-Layers (L01–L21). For Phases 6–9, there are no detailed ticket breakdowns yet. For those phases, use NEW tickets from the atomic sessions as the ticket list, and note that formal ticket planning is pending.

Input per run: `pass2-phase-{name}.md` (12-58KB) + `pass2-dedup-decisions.md` + `pass2-ticket-index.md` (6KB) + relevant section of `roadmap.md`
Output per run: `catalog-phase-{name}.md`

```
# Role
Senior Rust engineer, MTG rules expert.

# Task — Phase Catalog: [PHASE NAME]

Read:
1. plans/atomic-tests/pass2-phase-[name].md (enriched test specs for this phase from both halves)
2. plans/atomic-tests/pass2-dedup-decisions.md (cross-half dedup decisions — apply DROP/MERGE decisions)
3. plans/atomic-tests/pass2-ticket-index.md (compressed ticket list — ONLY covers Phase 5-Pre and Phase 5-Layers; for Phases 6–9 this file has no relevant tickets)
4. plans/roadmap.md — read ONLY the section for [PHASE NAME]

Produce a self-contained catalog section for this phase:

## 1. Phase Summary
2-3 sentences on what this phase delivers (from roadmap.md).

## 2. Ticket Index
- **Phase 5-Pre / Phase 5-Layers:** List every ticket ID from pass2-ticket-index.md with a one-line description.
- **Phases 6–9:** No formal tickets exist yet. Instead, list the NEW tickets surfaced by the atomic sessions (from the phase file and dedup decisions). Note: "Formal ticket planning pending — using atomic-session NEW tickets as provisional ticket list."

## 3. Test Catalog
All ATOM/BOUNDARY/COMP tests for this phase, with enriched specs from Pass 1.
- Apply dedup decisions: drop entries marked DROP, merge entries marked MERGE, add new COMPs.
- **Phase 5-Pre / Phase 5-Layers:** Sort by ticket, then by rule number within each ticket.
- **Phases 6–9:** Sort by rule number (no ticket grouping available). Preserve the existing Ticket field from Pass 1 if present, otherwise leave as TBD.
- Preserve the ### ATOM-xxx / Rule / Board / Action / Expected / Ticket / Deps / Cluster format.

## 4. COMP Tests
Composition tests exercising cross-ticket interactions WITHIN this phase.
Include both existing COMPs from Pass 1 and any new cross-half COMPs from dedup decisions.

## 5. Cross-Phase COMP Tests
Tests requiring infrastructure from a prior phase. List: test ID, precondition phase, mechanism.

## 6. Verification Gates
Map tests to Gate/Milestone criteria from roadmap.md for this phase.

## 7. NEW Tickets
- **Phase 5-Pre / Phase 5-Layers:** Tickets surfaced by atomic sessions not in implementation-plan-final.md. For each: proposed ticket ID, rule(s), scope estimate (S/M/L), recommended insertion point.
- **Phases 6–9:** All tickets are effectively "new" since no formal plan exists. Group tests into logical ticket-sized clusters (by mechanism/rule area) and propose provisional ticket IDs. This becomes the starting point for future implementation planning.

Output as plans/atomic-tests/catalog-phase-[name].md.
Write in chunks ≤4000 words. Split large phases across chunks.
```

#### Stage 2C: Gap Analysis & Appendices

Input: `pass2-global-index.md` (~20KB) + `pass2-ticket-index.md` (~6KB)
Output: `catalog-appendices.md`

```
# Role
Senior Rust engineer, MTG rules expert.

# Task
Read: plans/atomic-tests/pass2-global-index.md, plans/atomic-tests/pass2-ticket-index.md.

Produce appendices for the master catalog:

## 1. Gap Analysis
Cross-reference EVERY ticket in pass2-ticket-index.md against the test ID index (Section 1 of global index). For each ticket:
- If it has ≥1 test: mark COVERED
- If it has 0 tests: mark [GAP] and generate a stub test entry: ATOM-[rule]-GAP / Rule / Board / Action / Expected / [GAP-STUB]
List all GAP tickets explicitly.

## 2. META Verification
For each META entry (Section 4 of global index), verify it maps to concrete test IDs.
Flag any META with no concrete tests. Generate stubs if needed.

## 3. Master Statistics
| Phase | ATOM | BOUNDARY | COMP | GAP-STUB | NEW tickets |
Counts from global index Section 9, plus GAP-STUBs from Step 1.

## 4. DEFERRED Backlog
From global index Section 7, produce a clean appendix grouped by target phase.
Format: | Rule | Summary | Target Phase |

## 5. OUT-OF-SCOPE Registry
From global index Section 8, produce clean appendix.
Format: | Rule/Feature | Reason |

## 6. ALREADY-IMPLEMENTED Registry
From global index Section 1 (ALREADY-IMPLEMENTED), list all IDs.

Output as plans/atomic-tests/catalog-appendices.md.
```

#### Stage 2D: Assembly

Concatenate the per-phase catalogs and appendices into the final file:

```bash
cd plans/atomic-tests
python extract-pass2-index.py --assemble
```

This reads `catalog-phase-*.md` and `catalog-appendices.md`, concatenates them in implementation order, and writes `atomic-test-catalog.md`.

### Post-Merge Cleanup

After the merge is complete:
1. **Archive intermediate artifacts:** Move `merge-input-compressed.md`, `merge-input-half-A.md`, `merge-input-half-B.md`, `pass0-dependency-map.md`, `merge-pass1-half-A.md`, `merge-pass1-half-B.md`, `pass2-global-index.md`, `pass2-phase-*.md`, `pass2-dedup-decisions.md`, `catalog-phase-*.md`, `catalog-appendices.md` into `plans/atomic-tests/archive/merge-artifacts/`.
2. **Archive session files:** Move `plans/atomic-tests/sessions/` and `plans/atomic-tests/summaries/` into `plans/atomic-tests/archive/`. These are reference-of-last-resort only.
3. **Keep active in `plans/atomic-tests/`:** `atomic-test-catalog.md` (the final merge output), `extract-pass2-index.py` and `merge-sessions.py` (tooling). Keep `supplemental-docs/state-tracking-architecture.md` (architectural decision record).
4. **The catalog + CR text + roadmap + implementation plan** are the four documents needed for implementation. Everything else is archived history.
