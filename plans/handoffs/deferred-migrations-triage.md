# Prompt — triage Deferred Migrations

Paste the block below to open the session. Everything above it is context for
whoever is deciding *whether* to run it.

**Why now (2026-09-03).** RC-5 added 13 items in one PR, the largest single
addition the section has taken, and the review asked whether the deferrals are
still safe. Measured: 103 items, 92% of `codebase-state.md`, **62 with no
reachability note** and 83 with no size. The audit and the four-step order are
at the head of the Deferred Migrations section itself; this is step 1 and step 2
of that order. Steps 3 and 4 are tidying and are explicitly *not* in scope here.

**Size.** ~a day. It is a reading pass with a small write per item, and it
touches one file. Expect the diff to be large and boring, and expect two or
three items to turn out to be live bugs — that is the point of running it.

---

```
Triage `codebase-state.md`'s Deferred Migrations section. Branch:
docs/deferred-migrations-triage, from main.

The audit at the head of that section has the numbers and the four-step order.
This session is **steps 1 and 2 only** — reachability and size. Do not collapse
the closed items and do not split the file; both are cheap later and both would
bury the diff that matters.

## Step 1 — reachability, for the 62 items that do not state it

For each open item with no reachability note, decide which it is and write one
sentence saying so:

- **Unreachable** — name the thing that does not exist yet. "No registered card
  takes this road", "`Primitive::X` is a stub", "no caller produces a
  multi-entry batch". Items 46, 49, 52 and 61 are the shape to copy.
- **Reachable** — name the card, pool or code path that gets there, and say
  whether the behaviour is wrong *today*. **A reachable item is a bug report,
  not a deferral**: if the pool can build the board, say what the engine does
  and what the CR says, and expect to open a ticket rather than a note.
- **Closed** — the work landed and nobody struck the item. Mark it, with the
  commit.

**The claim to be most suspicious of is "unreachable" written years of phases
ago.** Reachability is monotone: every phase adds cards and paths, and nothing
removes them. An item that was unreachable when written may not be now — item 49
is the worked example, and RC-5 is what made its board exist. So re-derive each
verdict against the tree as it is; do not carry the old one forward.

**Verify, do not assume.** `--pool stress` plays every registered card, so
"reachable" usually means "a fuzz game can build this". Where a card decides it,
check the card on Scryfall — oracle text *and* its rulings (RC-5's item 61 is
what happens when only the CR is read). Where code decides it, read the code.

## Step 2 — size the 83 unsized

One `**Sized:**` line each, in the section's existing idiom: what changes,
roughly how many lines, and what it lands with. A range is fine; "unknown until
X exists" is fine and is itself a size. An item nobody can schedule is not
deferred, it is forgotten — that is the rule this step enforces.

## While you are in there

- **"item N" is three namespaces, not one, and that is bigger than it looked
  when this prompt was written** (measured 2026-09-03). Within this section, 108
  items use ids 1–65 twice — a main run plus per-section runs in the four
  "Before X" sections. Across the tree it is worse: `CLAUDE.md`'s critical path
  numbers items 1–7 and is cited as "item 7" too, so **`item 6` and `item 7` each
  name three different things**. Counted across `plans/*.md` and `CLAUDE.md`:
  **355 "item N" citations, 261 of them (73%) without naming which namespace.**
  Most read correctly from context; the bare ones in prose are where a reader
  loses. Decide the fix — renumber into one space and fix every citation, or
  adopt a prefix (`DM-46`, `CP-7`) and sweep, or leave it and say in each
  section header that ids are section-scoped. **Prefixing is the cheapest of the
  three** and is the only one that makes a bare citation impossible rather than
  merely discouraged. Either way this is the pass to do it in: it already
  touches every item and every citation of them.
- Two branches sit behind `origin/main`, unmerged and unscheduled —
  `replacement/rc-2-enter-battlefield` and `phase-ld-part-b`. Say what they are
  or delete them.
- **`plans/handoffs/cv-1-review.md` holds six items marked "open, with an
  owner"** (C1–C6) and CV-1 shipped. `CLAUDE.md`'s authority table says a
  handoff is deleted when the work lands, so those six are debt sitting in a
  file whose contract says it should not exist. They belong in this section or
  in `copy-effects-architecture.md`. **Absorb them before or during this pass** —
  otherwise the triage's output is "the complete list of what is owed" and is
  missing six.

## Exit

Suite green and zero warnings (this should touch no code; if it does, that is a
finding and it gets its own commit with a test). Both `check_*.py`. Every open
item states reachability and carries a size. The audit block at the head of the
section is re-recorded with the new counts, and says what the triage found —
including how many "unreachable" claims had gone stale, which is the number
that says whether this pass needs repeating and how often.

Report the reachable-and-wrong items separately at the end. Those are the
session's actual output; the rest is bookkeeping.
```
