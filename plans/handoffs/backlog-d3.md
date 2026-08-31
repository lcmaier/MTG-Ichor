# Handoff — the D3b triage

**Resume here.** D3a landed: `plans/backlog.md` exists, with its structure, the
exclusion decision, and four seeded entries. **Read `backlog.md` first** — it is
now authoritative for everything this file used to hold about the three known
mechanics, and it is not restated here. Then `cr-coverage-audit.md` §2 (method),
§5 (findings), §6 (the queries). Delete this file when D3b lands.

---

## 0. What is left

D3b was split three ways. **Slice 1 landed** — the 38 sections a plan doc
already discusses, 197 atoms, now `backlog.md` §3. Five became entries
(§2.5–§2.8); the other 33 are dispositioned in §3.1 and §3.2.

**Slice 2 — the 25 sections no plan doc mentions, 100 atoms.** This is where
the remaining real entries come from. `backlog.md` §4 lists the clusters and
warns what to expect: several look plainly implemented from their summaries
alone and will resolve like §3.2 rather than becoming entries.

**Slice 3 — the long tail**, 24 sections at ≤2 atoms. One line each.

```bash
python plans/specdb.py orphaned --bucket unbuilt --all
```

Still reports 297/63 and will keep doing so: **re-filing is not how a section
leaves that list, and §3's dispositions are prose the query cannot read.** Use
it as the source, not as a progress bar.

**Also still open**: the 25 CR 601 casting-procedure atoms — modal
announcement, kicker-conditional targets, divide-or-distribute, the 601.2g mana
window, 601.4's look-ahead. `backlog.md` §4 says why they were left out.

---

## 1. Decided at D3a, do not re-litigate

- **`backlog.md` is excluded from `orphaned`'s ownership set** — it is that
  query's *output*, so leaving it in would make the query converge to zero by
  being written. The full argument, and why it is **not** the audit doc's
  argument, is `backlog.md` §1 and `_scan_citations`'s docstring. The burn-down
  is driven by graduation to an architecture doc instead.
- **Atoms re-file to a `Backlog` phase**, a real phase name in `PHASE_ORDER`
  that `normalize_phase` recognises. The line keeps its old filing visible:
  `- **Phase:** Backlog — cost pipeline (was Phase 5 Layers)`.
- **Do not design the mechanics here.** A line each. Three phase designs is a
  different project.
- **D2b is deferred, deliberately.** 54 atoms are cited only in `mtgsim/src/`:
  production code encodes the rule and no test mentions it. Those need a test
  *written*, not annotated. It does not block D3b — but slice 1 found **56 more
  atoms of the same shape** (`backlog.md` §3.2), so D2b is larger than its
  original 54 and is now the biggest single thing this workstream has deferred.
- **Slice 1 did not re-file anything, and left a policy question open**
  (`backlog.md` §3.3): **27 of `owed`'s 31 atoms sit in slice-1 sections, 14 in
  CR 701 alone.** Re-filing them would take the gate from 31 to 4. Whether a
  `NEW`-ticketed atom belongs to `owed` or to `backlog.md` is the owner's call,
  and it was not taken in passing.

---

## 2. Corrections — claims made and then disproved

Recorded so they are not re-derived. Each was believed, acted on, and killed by
a measurement. The first four are from the D2/D3a sessions; the last two are
D3a's own.

1. **"~24% of the corpus points at a dead `L##`/`T##` ticket vocabulary."**
   False. Every atom normalizes to a real phase; `ticket` is a secondary label
   no query gates on. There is no tombstone problem in the data.
2. **"D2 shrinks the backlog."** False. D2 operates on the **cited** bucket, the
   backlog is the **uncited** one. **D2 never gated D3.**
3. **"Item 30's back-stop is before RC."** False — `ATOM-702.44a-001` is
   ticketed *"DEFERRED — Phase 8"* and named the dependency itself. Fixed in
   `77bda5e`; the back-stop is **CV**.
4. **"The 75 `cited` atoms are a cheap annotation errand."** False twice over —
   18 in `tests/` of which only **3** were real, and 54 in `src/` needing tests.
5. **"CR 601/607 is casting from a non-hand zone, 43 atoms."** False, and it is
   the one that changed D3a's shape. **No** shipped-phase CR 601 or 607 atom
   concerns a non-hand zone — those are at Phase 8, correctly filed. The 43 are
   casting-procedure depth (25), linked abilities (10), cost pipeline (7) and one
   covered. **Linked abilities (CR 607) is a whole mechanic of its own**, and it
   had been travelling under CR 601's section number. It is *named* in three plan
   docs and designed by none; no mention is rule-level, which is why its atoms
   survived the ownership filter. Do not repeat "no doc mentions it" — that was
   checked at D3a and is false.
6. **"Cost modification is CR 107/118/202, 58 atoms."** Over-collected. That
   bucket is 54 shipped-phase atoms and at least four mechanics; **28 stayed
   put** because their `Mechanism` field names a function that exists and works
   (`ManaPool::spend()`, `pay_life()`, `check_cost_resource`). They are missing a
   test, not missing behavior.

**The method that fixed 5 and 6: triage from the atom's `Mechanism` field, not
its rule number.** `Mechanism` names the function; the rule number names a CR
section, and a CR section is a loose proxy for a mechanic. This needs no code
read and it is the single most useful thing carried into D3b.

7. **"Slice 1 is bookkeeping — mostly pointers."** Broadly right, and worth the
   number: **two thirds of it (132 of 197 atoms) needed no entry.** But it also
   produced four real entries and one verified Deferred Migrations find —
   `StackEntry.chosen_modes`, dead scaffolding with twelve empty writers and no
   reader (`codebase-state.md` item 31). Cheap slices still turn things up.

**Three estimates, all low**: D1 called ~20 lines, landed +58/−17; D2a called 18
annotations, produced 3 plus a query bug; D3a inherited "~101 atoms to re-file"
and re-filed 43. **Re-size against live output before committing to scope.**

---

## 3. Gates

`python plans/specdb.py build` / `gaps` / `orphans` / `owed` ·
`python plans/check_claude_md.py`.

`owed` moved **38 → 31** at D3a and should not move again at D3b unless a
re-filed atom carries a `NEW` ticket — check, don't assume. `CLAUDE.md` is at
**198/200**; a new authority row needs one removed. If D3b touches Rust — it
should not — `cargo build --all-targets` must print zero warnings and
`cargo test` must stay green.
