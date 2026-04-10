# Workflow Prompts for Phased Implementation

Patterns for driving implementation work in **Windsurf (Cascade)**.

### How Cascade differs from a blank-chat LLM

- **Autonomous file reading.** Cascade reads files on its own via code search,
  grep, and file reads. You can't (and don't need to) manually control what it
  sees. It will pull in context it thinks it needs.
- **Persistent memories.** Architectural decisions, phase history, and file
  structures carry across sessions automatically. You don't need to re-paste
  context each time.
- **The prompt IS the chat message.** There's no separate template to fill out —
  your message to Cascade is the prompt. Keep it short and directive.
- **It reads the plans directory.** The `plans/` files act as a knowledge base.
  Cascade can and will read `implementation-plan-final.md`, `roadmap.md`,
  `alchemy-mechanics-audit.md`, etc. when it needs architectural context.
  Point it at the right file rather than quoting contents.

**Implication:** Your job is to tell Cascade *what* to do and *where to look*,
not to hand-feed it context. Be specific about the ticket/deliverable, name
the relevant plan files, and state constraints. Let it do the reading.

---

## (A) Implement Ticket

A short chat message to kick off work on a specific ticket.

### Message pattern

> Implement **T04** (counter infrastructure) from `implementation-plan-final.md`.
> Dependencies T01 and T02 are done. Read the ticket description and acceptance
> criteria from the plan. Check `phase-index-phase-5-pre.md` for relevant ATOM
> IDs. Run `cargo test` after changes.

That's it. ~3 sentences. Cascade will:

1. Read the ticket from `implementation-plan-final.md`
2. Read the relevant source files it identifies from the ticket description
3. Grep the phase index for ATOM IDs matching the ticket
4. Implement, write tests, run `cargo test`

### What to include in your message

| Always include | Why |
|---|---|
| **Ticket ID** (e.g., T04, L07) | Unambiguous scope — Cascade reads the plan for details |
| **Which dependencies are done** | Prevents reimplementing existing work |
| **"Run cargo test"** | Explicit verification instruction |

| Include if relevant | Why |
|---|---|
| **Specific constraint** ("don't modify zones.rs") | Override Cascade's autonomous scoping when needed |
| **Design question** ("should X use pattern Y?") | Flag ambiguities upfront rather than discovering mid-implementation |
| **ATOM IDs** if you know them | Speeds up the behavioral target lookup |
| **"Check [file] for context"** | Direct Cascade toward a specific plan doc if the answer isn't obvious from code alone |

### What NOT to include

- **Full ticket descriptions** — Cascade reads `implementation-plan-final.md` itself.
- **Architecture summaries** — Cascade has memories + can read `roadmap.md`, `alchemy-mechanics-audit.md`, etc.
- **File lists** — Cascade identifies files to modify from the ticket description and code search. Only override if you want to constrain it.

### Variations

- **Multi-session ticket:** At session start, say "Continue T04. Last session
  we finished sub-steps 1-3. Pick up at sub-step 4." Cascade's checkpoint
  summaries carry the detailed state.
- **Bug fix:** "There's a bug in [behavior]. Expected X, got Y. Reproduce with
  [test or steps]. Diagnose root cause before fixing."
- **Refactor:** "Refactor [module] from [current pattern] to [target pattern].
  Don't change behavior — all existing tests must pass."

---

## (B) Phase Kickoff — Flesh Out Deferred Items

A chat message to generate the ticket breakdown for a new phase.

### Message pattern

> Phase 7 kickoff. Read the Phase 7 section in `roadmap.md` and the deferred
> items targeting Phase 7 in `implementation-plan-final.md` (Part 2 Deferred
> Items table). Also read the zone-agnostic trigger scanner constraint in
> `roadmap.md` and `alchemy-mechanics-audit.md` Q4/Q7.
>
> For each deliverable: propose concrete data structures, break into S/M
> tickets (matching the plan's format), show the dependency graph, and flag
> risks. Don't implement any code.

### What to include in your message

| Always include | Why |
|---|---|
| **Phase number** | Scopes the work |
| **"Read [specific plan files]"** | Directs Cascade to the right knowledge base docs |
| **"Don't implement code"** | Prevents Cascade from jumping to implementation |
| **Output format expectation** | "Matching the plan's format" / "pipe-delimited tables" / etc. |

| Include if relevant | Why |
|---|---|
| **Specific deferred items** to focus on | If the phase is large, you can scope to a subset |
| **Specific architectural constraints** | E.g., "the trigger scanner must be zone-agnostic" |
| **"Check [source file] for current state"** | If the design depends on current struct layouts |

### What the output should contain

1. **Design** — concrete structs, fields, function signatures
2. **Tickets** — ID, title, size, dependencies, acceptance criteria, files touched
3. **Dependency graph** — which tickets depend on which
4. **Risk flags** — design spikes, merge conflict risks, missing test coverage

### When to use (B) vs skip to (A)

- **Phase 5-Pre, Phase 5-Layers:** Already have full ticket breakdowns → skip (B), go straight to (A)
- **Phases 6, 7, 8, 9:** Roadmap bullets but no tickets → (B) first, then (A)
- **Deferred item lands mid-phase:** Mini (B) for just that item
- **Mid-implementation design gap:** Targeted (B) for just the gap

---

## Workflow Summary

```
Phase N start
    │
    ├─ Has full ticket breakdown? ──yes──► (A) per ticket, Tier 1 first
    │
    └─ No / partial ──► (B) to generate ticket breakdown
                              │
                              ▼
                        Review, adjust, merge into plan
                              │
                              ▼
                        (A) per ticket, Tier 1 first
                              │
                              ▼
                        After each ticket: cargo test, update plan status
                              │
                              ▼
                        Verification gate (if defined in plan)
                              │
                              ▼
                        Next tier / next phase
```

---

## Windsurf-Specific Tips

1. **Point, don't paste.** Say "read T04 from `implementation-plan-final.md`"
   instead of copying the ticket into your message. Cascade reads the file
   directly and gets the full context including surrounding tickets.

2. **Name files, not concepts.** "Check the alchemy audit" is vague.
   "Read `alchemy-mechanics-audit.md` Q4" is precise. Cascade's file search
   works best with exact filenames.

3. **Use follow-up messages for steering.** If Cascade is going off-track
   mid-ticket, a short "Stop — that approach conflicts with [X]. Use [Y]
   instead" is more effective than a long upfront prompt trying to anticipate
   every failure mode.

4. **Let memories do the work.** Architectural decisions (delta log design,
   D20a/D20b split, GameAction middleware, etc.) are stored in Cascade's
   memory. You don't need to remind it. If it makes a decision that contradicts
   a prior architectural choice, correct it and it will update its memory.

5. **One ticket per conversation when possible.** Cascade's context window is
   finite. A fresh conversation per ticket (or per 2-3 small tickets) keeps
   context focused. The checkpoint system carries state across conversations.

6. **Verify incrementally.** "Run `cargo test`" after each ticket. Don't batch
   3 tickets then test — compound failures are harder to diagnose.

7. **Escape hatch.** If Cascade hits an ambiguity, it should stop and ask.
   If it doesn't, and you see it guessing, say "Stop. What assumption did
   you just make about [X]?" This is more reliable than upfront "don't guess"
   instructions, because Cascade doesn't always know it's guessing.
