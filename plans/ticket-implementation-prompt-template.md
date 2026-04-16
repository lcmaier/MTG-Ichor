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

1. `plans/implementation-plan-final.md`
   - The **Overview** and **Execution Order** sections (context).
   - The **Testing Conventions** section (taxonomy + per-ticket coverage rules
     + standing rules). These are binding for this work.
   - The ticket body for **{{TICKET_ID}}** — scope, depends on, files, steps,
     tests, acceptance criteria, commit message.
   - The ticket bodies for any direct dependencies listed in `Depends on`.
   - The **Discrepancies & Notes** section, if the ticket references a
     discrepancy number (§N).

2. `plans/test-strategy-post-dp-refactor.md`
   - §2 (what the DP refactor unlocks), §3 (test classes A–E), §6 (landed
     tickets).

3. `plans/special-1c-audit-notes.md`
   - DP test-migration patterns and gotchas (CastSpell returns ActionTaken
     immediately; candidate ordering Pass → PlayLand → CastSpell →
     ActivateAbility; `GenericManaAllocation` bucket count; `DiscardToHandSize`
     one call per card; `LegendRule` fires inside `perform_sba_and_triggers`;
     `queue_empty_turn_passes()` excludes DeclareAttackers and
     DiscardToHandSize).

4. Relevant source modules named in the ticket's **Files** list.

# Project Invariants (do not violate)

- **DP surface is the 4-primitive trait** in `src/ui/decision.rs`:
  `pick_n`, `pick_number`, `allocate`, `choose_ordering`. Engine call sites go
  through `ask_*` free functions in `src/ui/ask.rs`. `ChoiceKind` lives in
  `src/ui/choice_types.rs`.
- **Tests use `ScriptedDecisionProvider`** with explicit
  `expect_pick_n(kind, indices)` / `expect_number(kind, n)` /
  `expect_allocation(kind, alloc)` / `expect_ordering(kind, order)`.
  No silent defaults — `PassiveDecisionProvider` is deleted.
- **`ChoiceKind` is exhaustive and semantic.** Only add variants that the
  engine actively needs. Each variant should carry enough context for
  `ask_*` + `validate_*` to be written against it.
- **Oracle module (`src/oracle/`) is read-only.** Engine mutations go through
  `engine/actions.rs::execute_action` where applicable.
- **Game state mutations emit deltas (Phase 6+ design).** For now,
  `events.emit(...)` scaffolding remains; do not remove it unless the ticket
  explicitly says so.
- **Do not weaken, skip, or delete existing tests.** If a test needs to
  change because semantics changed, say so and justify it before editing.
- **Zero warnings.** `cargo build` and `cargo test` must produce zero
  warnings when done.

# Testing Requirements (from Testing Conventions)

Tag every new test with its class (A/B/C/D/E) in the PR summary.
Mandatory coverage for this ticket if it does any of the following:

- Modifies or adds an `ask_*` function → **Class C** test(s).
- Adds a `ChoiceKind` variant → **Class B** test; **Class A** if it
  introduces new bounds/validation.
- Adds a `SelectionFilter` or `PermanentFilter` variant → **Class C** test(s).
- Adds a new multi-step engine flow → **Class B** test asserting DP
  sequence.
- Adds a new primitive input constraint → **Class A** negative test.
- Adds a `ChoiceKind` visible to `RandomDecisionProvider` → fuzz harness
  200/200 in acceptance.

Write failing tests **before** the implementation where feasible,
especially for DP-surface changes.

# Workflow

1. **Plan.** State your understanding of the ticket in 3–6 bullets. List the
   files you will touch and which test classes you will add (by class and
   approximate count). Ask for confirmation if anything is ambiguous.
2. **Write failing tests first** for the behaviors the ticket specifies, at
   least for DP-surface and validation changes. For pure refactors, tests
   come after.
3. **Implement** the steps in the ticket in order. Keep edits minimal and
   focused. Prefer extending existing helpers to adding new ones.
4. **Verify.** Run `cargo test` (NOT with `2>&1` — PowerShell stderr redirect
   produces a spurious non-zero exit). Iterate until all tests pass and there
   are zero warnings.
5. **Run the fuzz harness** if the ticket touches the DP surface or a
   `ChoiceKind` visible to `RandomDecisionProvider`:
   `cargo run --release --bin fuzz_games -- --games 200`. Expect 200/200.
6. **Report.** Produce a summary with:
   - Files changed (with one-line rationale each).
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
- When a ticket has a **discovery discrepancy** (§15a, §15b, etc.), paste
  the discrepancy text into the prompt explicitly — it's usually
  load-bearing context.
- If the ticket depends on in-flight work not yet merged, note the branch
  or commit it's based on and paste the dependency's current status.
