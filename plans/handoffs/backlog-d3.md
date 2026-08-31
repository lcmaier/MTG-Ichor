# Handoff — `plans/backlog.md` and the D3 triage

**Resume here.** Everything below is state from the 2026-08-31 session that is
*not* recoverable from the repo. What *is* in the repo is not restated: read
`cr-coverage-audit.md` §2 (the method), §5 (findings), §6 (the queries) first.
Delete this file when D3 lands.

---

## 0. Where the work stopped

PR #70 is open and carries the whole CR-audit rewrite plus the tooling. It is
**not** a prerequisite for D3 in the code sense, but D3's numbers come from
`orphaned`, which #70 introduces — so either merge #70 first or branch off it.

Done: the type-surface audit, `specdb orphaned`, D1 (the cited/unbuilt
pre-sort), D2a (hand-confirming the 18 test-cited candidates).

Not done: **D2b** and **D3**, below.

---

## 1. The one number that matters

```bash
python plans/specdb.py orphaned --bucket unbuilt
```

**332 atoms across 63 sections.** That is `backlog.md`'s upper bound, and
**nothing pending reduces it** — see §4's correction 2. Regenerate it rather
than trusting this file; it is a query, and a number in prose rots.

---

## 2. What D3 is, and the split

**D3a — structure plus the two known clusters.**
Write `backlog.md`; seed it with the two mechanics whose verdicts are already
established, and re-file their atoms off the shipped phases they are wrongly
filed under.

- **Cost modification** (CR 107/118/202/601.2f) — 58 atoms, sessions 1, 2, 5.
  `apply_cost_modifications` is a passthrough stub with a test asserting so.
  `replacement-architecture.md` §9 already says it "needs a phase marker of its
  own … and it is not small."
- **Casting from a non-hand zone** (CR 601/607) — 43 atoms, session 5.
  `check_cast_legality` hard-codes `Zone::Hand`. CR 607 linked abilities is 20
  uncovered atoms and no doc mentions it.

**No code read is needed for these two** — the type-surface audit already did
it, and both are documented stubs. That is what makes them the cheap first
slice.

> **D3a moves a gate: `owed` 38 → 31.** Seven of the 101 carry a NEW ticket.
> That is expected and is the reason D3a needs its own branch rather than
> joining PR #70, whose stated exit criterion is `owed` = 38. **Verify the
> arithmetic before assuming it still holds.**

**D3b — the remaining ~61 sections.** One judgment each: real backlog item,
rough size, what it blocks. This is the bulk and the part most likely to go
thin if rushed; consider splitting again rather than doing 61 in one pass.

**Third mechanic, no atoms: voting (CR 701.38)** and CR 201.4 with it —
`DecisionProvider` is four index-shaped methods with no vote and no "choose a
card name". It gets a `backlog.md` entry but stays invisible to `owed` and
`orphaned` either way, because it has **zero atoms**. Authoring them is Pass B,
which is explicitly unscheduled.

---

## 3. Decisions taken, not yet written anywhere

- **`plans/backlog.md` is the agreed home** for everything off the critical
  path — owner's call, and the motivation was wanting a roadmap with everything
  yet to do rather than only the critical components. It needs a row in
  `CLAUDE.md`'s authority table; the file is at **197/200**, so there is room
  for one line and not much more.
- **Do not design the three mechanics here.** A line each — rough size, what it
  blocks — is the deliverable. Sizing before writing is the project's own rule,
  and three phase designs is a different project.
- **D2b is deferred, deliberately.** 54 atoms are cited only in `mtgsim/src/`:
  production code encodes the rule and *no test mentions it at all*. Those need
  a test **written**, not annotated — larger than D2a and closer to Pass B than
  to planning. It does not block D3.

---

## 4. Corrections — claims made and then disproved this session

Recorded so they are not re-derived. Each was believed, acted on, and killed by
a measurement.

1. **"~24% of the corpus points at a dead `L##`/`T##` ticket vocabulary."**
   False. Every atom normalizes to a real phase; `ticket` is a secondary label
   that no query gates on. There is no tombstone problem in the data.
2. **"D2 shrinks the backlog."** False, and it is the important one: D2 operates
   on the **cited** bucket, the backlog is the **uncited** one. **D2 never
   gated D3.**
3. **"Item 30's back-stop is before RC."** False. `ATOM-702.44a-001` is ticketed
   *"DEFERRED — Phase 8. Requires mana-color-spent tracking"* — the corpus
   scheduled sunburst at Phase 8 and named the dependency itself. Fixed in
   `77bda5e`; the back-stop is **CV**.
4. **"The 75 `cited` atoms are a cheap annotation errand."** False twice over —
   18 in `tests/` of which only **3** were real, and 54 in `src/` needing tests
   written.

**Two estimates, both low**: D1 was called ~20 lines and landed +58/−17; D2a
was called 18 annotations and produced 3 plus a query bug. **Re-size D3 against
live `orphaned` output before committing to scope.**

---

## 5. A trap to know about

`orphaned`'s third filter reads *any* plan-doc mention as ownership. Writing
about a rule in `plans/*.md` therefore removes its atoms from the worklist.
`cr-coverage-audit.md` is excluded from that set (`_scan_citations`'s
`exclude_docs`) because an instrument must not satisfy its own filter — it had
been claiming 19 atoms, CR 117.1a among them, purely by citing them as
examples.

**`backlog.md` will have the same problem, and worse**: a backlog exists to
name rules, so every entry written will delete its own atoms from the query
that found them. **Decide up front** whether `backlog.md` joins the exclusion
list. The argument for excluding it is identical to the audit doc's; the
argument against is that a backlog entry genuinely *is* a design claiming the
rule. This is a real design question and it should not be discovered halfway
through writing entries.

---

## 6. Gates

`python plans/specdb.py build` / `gaps` / `orphans` / `owed` ·
`python plans/check_claude_md.py`. Expect `owed` **38 → 31** at D3a and **only**
there. If D3 touches Rust — it should not — `cargo build --all-targets` must
print zero warnings and `cargo test` must stay green.
