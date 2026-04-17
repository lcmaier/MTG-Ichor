# Ticket Implementation Prompt Template

> Paste this into a new Cascade / LLM session to implement a ticket from
> `plans/implementation-plan-final.md`. Replace `{{TICKET_ID}}` and any
> `{{...}}` placeholders. Leave the rest verbatim — it reflects the
> current project conventions.

---

```text
# Task

Implement ticket **{{TICKET_ID}}** from `plans/implementation-plan-final.md`
in the Rust MTG simulator workspace (`mtgsim_v2/mtgsim/`).

{{Optional: 1-2 sentences of specific intent or deviations from the plan.}}

# Required Reading (in this order)

1. **Always:** `plans/implementation-plan-final.md`
   - The **Overview** and **Execution Order** sections (context).
   - The **Testing Conventions** section (A/B/C/D/E taxonomy + per-ticket
     coverage rules + standing rules). Binding for every ticket.
   - The ticket body for **{{TICKET_ID}}** — scope, depends on, files, steps,
     tests, acceptance criteria, commit message.
   - The ticket bodies for any direct dependencies listed in `Depends on`.
   - The **Discrepancies & Notes** section if the ticket references `§N`.

2. **Always:** relevant source modules named in the ticket's **Files** list.

3. **Rules grounding (required for any ticket implementing MTG rules
   behavior; skip for pure tooling/infra/refactor tickets):**
   - **Comprehensive Rules** — `MTG-Rules/Chapter N - <name>.txt` (full
     chapters) or `MTG-Rules/LLM-Chapter-Splits/` (pre-split partials for
     Chapters 6/7 when the full file is unwieldy). Read the specific
     sub-rules the ticket cites (e.g., if the ticket lists ATOMs `601.2b-*`,
     read CR 601.2b and its surrounding context in Chapter 6). The
     `Glossary.txt` is the authoritative definition source — consult it
     before interpreting any ambiguous rules term.
   - **Atomic test catalog** — for any ticket whose `ATOMs:` list is
     non-empty:
     - `plans/atomic-tests/phase-index-phase-{X}.md` — index entry for each
       ATOM the ticket must cover (summary + tags + dependency refs).
     - `plans/atomic-tests/sessions/session-{N}.md` — full enriched spec
       (board state, DP script, expected result) for each ATOM. The
       session files are the source of truth for test setup; the summary
       files are navigation aids only.
     - `plans/atomic-tests/summaries/session-{N}-summary.md` — use to
       locate which session contains a given ATOM, then go to the session.
     - `plans/atomic-tests/supplemental-docs/` — architectural research
       (e.g., `state-tracking-architecture.md`, `rule-400-7-details.md`,
       `603-2f-complexity.md`). Read when the ticket or its ATOMs cite a
       supplemental doc by name.
     - `plans/atomic-tests/pass0-dependency-map.md` — cross-phase
       architectural decisions. Consult when the ticket's `Source:` line
       references a pass0 section (§N).

4. **Conditional reading** — only read these when the ticket actually
   involves the relevant subsystem:
   - **DecisionProvider / `ask_*` / `ChoiceKind` changes** →
     `plans/test-strategy-post-dp-refactor.md` §2–§3 (what the refactor
     unlocks, test classes) and `plans/special-1c-audit-notes.md` (migration
     patterns + gotchas like `CastSpell` returning `ActionTaken` immediately,
     candidate action ordering, `GenericManaAllocation` bucket count,
     `queue_empty_turn_passes()` scope).
   - **Layer system / continuous effects** → Part 2 sub-plan docs referenced
     by the ticket.
   - Any other docs explicitly cited in the ticket's `Source:` line.

{{Optional: list any extra docs or prior-art tickets the LLM should read
for this specific ticket.}}

# Project Invariants (do not violate)

General invariants that apply to every ticket:

- **Oracle module (`src/oracle/`) is read-only.** Engine mutations go through
  `engine/actions.rs::execute_action` where applicable.
- **Game state mutations emit deltas / events** per current scaffolding. Do
  not remove `events.emit(...)` calls unless the ticket explicitly says so.
- **Do not weaken, skip, or delete existing tests.** If a test needs to
  change because semantics changed, state the reason before editing it.
- **Zero warnings.** `cargo build` and `cargo test` must produce zero
  warnings at the end.
- **Minimal, focused edits.** Prefer extending existing helpers to adding
  new ones. No drive-by refactors outside the ticket's scope.
- **No silent scope expansion.** If something outside the ticket needs
  fixing, note it as a follow-up; don't fix it inline.
- **Ground rules behavior in the CR.** When implementing anything that maps
  to a Comprehensive Rules sub-rule, the CR text (plus Glossary) is
  authoritative. If the ticket, an ATOM spec, and the CR disagree, flag it
  before proceeding — don't silently pick one. Cite the specific sub-rule
  (e.g., `CR 608.2b`) in code comments and test names where useful.

Subsystem invariants — **only apply if the ticket touches that subsystem**:

- **Player decisions (DP surface):** go through the 4-primitive trait in
  `src/ui/decision.rs` (`pick_n`, `pick_number`, `allocate`, `choose_ordering`)
  via `ask_*` free functions in `src/ui/ask.rs`. `ChoiceKind` lives in
  `src/ui/choice_types.rs`. Tests use `ScriptedDecisionProvider` with
  explicit `expect_*` queue entries — no silent defaults. `ChoiceKind` stays
  exhaustive and semantic; don't add catch-all variants.
- **Effect resolution:** effects are `Primitive` atoms composed via the
  `Effect` combinator enum in `types/effects.rs`; resolution goes through
  `engine/resolve.rs`.
- **Zone changes:** use `move_object`; do not manipulate zone collections
  directly except where a ticket explicitly sanctions it.
- *(Add subsystem invariants here as later refactors land — layer system,
  delta log, replacement-effect middleware, etc.)*

# Testing Requirements

Read the **Testing Conventions** section in `implementation-plan-final.md`
for the authoritative rules. In brief:

- Tag every new test with its class (A/B/C/D/E) in the PR summary.
- The conventions specify which classes are **mandatory** based on what the
  ticket changes (e.g., adding an `ask_*` function forces Class C; new bounds
  force Class A; new `ChoiceKind` variants force Class B + fuzz smoke; etc.).
  Consult that list — do not try to infer from memory.
- Write failing tests **before** implementation for surface changes (DP,
  `ask_*`, new public API). For pure refactors or internal cleanup, tests
  may come after.

# Workflow

1. **Plan.** State your understanding of the ticket in 3–6 bullets. List the
   files you will touch, the CR sub-rules and ATOMs you'll cover, and which
   test classes you will add (by class and approximate count). Flag any
   conflicts you notice between the ticket, its ATOM specs, and the CR
   before coding. Ask for confirmation if anything is ambiguous.
2. **Write failing tests first** for surface-level behavior the ticket
   specifies. For pure refactors, tests come after.
3. **Implement** the ticket's Steps in order. Keep edits minimal.
4. **Verify.** Run `cargo test` (NOT with `2>&1` — PowerShell stderr redirect
   produces a spurious non-zero exit). Iterate until all tests pass and
   there are zero warnings.
5. **Run the fuzz harness** if the ticket touches anything
   `RandomDecisionProvider` can observe, or changes core game flow:
   `cargo run --release --bin fuzz_games -- --games 200`. Expect 200/200.
   Skip if the ticket is pure docs/tests/refactor with no runtime impact.
6. **Report.** Produce a summary with:
   - Files changed (one-line rationale each).
   - New tests, grouped by class (A/B/C/D/E).
   - Full test-count breakdown in the format
     `N new, X unit + Y integration + Z doc-test = T total`.
   - Fuzz result if applicable.
   - The exact commit message from the ticket's **Commit** field.
   - Any deviations from the ticket's Steps, with justification.
   - Any follow-ups that surfaced (candidate Discrepancy §N entries or new
     SPECIAL-# tickets) — do NOT edit the plan to add them; list them for
     me to review.

# Environment Notes (Windows PowerShell)

- Run `cargo test` **without** `2>&1` (produces false exit code 1).
- Do **not** use `cd`; set `Cwd` on tool invocations instead.
- Workspace root: `c:\Users\maier\Desktop\MTG Simulator\mtgsim_v2\`.
- Crate root: `c:\Users\maier\Desktop\MTG Simulator\mtgsim_v2\mtgsim\`.

# Out of Scope

Anything not in the ticket's Steps. If you find something that needs fixing
and is clearly outside the ticket, note it as a candidate follow-up in the
final report — do not silently fix it.
```

---

## Usage Notes

- For **small** tickets (SPECIAL-*, most T## Small-scope), this prompt is
  sufficient on its own. Point the LLM at `{{TICKET_ID}}` and go.
- For **medium/large** tickets that span many files (T18a, L04, etc.),
  prepend 1–2 paragraphs of architectural context pulled from the ticket's
  `Source:` / `Design decisions:` bullets — don't rely on the LLM to
  re-derive them from `Steps`.
- For **test-only tickets** (SPECIAL-5, SPECIAL-6, future Class-C
  extensions), add a sentence calling out that no game logic changes — this
  prevents the LLM from "helpfully" refactoring nearby code.
- For **rules-implementation tickets** (anything with a non-empty `ATOMs:`
  list or explicit CR sub-rule references), paste the ATOM IDs and cited
  CR sub-rule numbers into the `Optional:` extras slot. The LLM can then
  locate the full specs via `plans/atomic-tests/phase-index-phase-{X}.md`
  → `plans/atomic-tests/sessions/session-{N}.md` without having to grep.
  Attach the specific `MTG-Rules/Chapter {N} - *.txt` file path when a
  single chapter is dominant (e.g., Chapter 6 for the casting pipeline).
- For **DP / `ask_*` tickets**, explicitly list
  `plans/test-strategy-post-dp-refactor.md` and
  `plans/special-1c-audit-notes.md` in the `Optional:` extras slot so the
  LLM promotes them from "conditional" to "required" reading.
- When a ticket has a **discovery discrepancy** (§15a, §15b, etc.), paste
  the discrepancy text into the prompt explicitly — it's usually
  load-bearing context.
- If the ticket depends on in-flight work not yet merged, note the branch
  or commit it's based on and paste the dependency's current status.
- **Keep this template subsystem-agnostic.** When a new major refactor lands
  (layer system, delta log, etc.) and introduces invariants future tickets
  must respect, add them to the "Subsystem invariants" list — one bullet
  each, conditionally phrased ("only apply if the ticket touches X").
