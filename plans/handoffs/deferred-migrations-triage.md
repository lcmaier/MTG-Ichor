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

- **Item ids are not unique**: 103 items use ids 1–65 twice, a main run plus
  per-section runs in the four "Before X" sections. Every cross-reference in the
  tree is hand-qualified and correct today. Decide whether to renumber into one
  space (and fix every citation — grep `item \d` across `plans/` and
  `mtgsim/src`) or to leave it and say in the section header that ids are
  section-scoped. Either is defensible; silence is not.
- Two branches sit behind `origin/main`, unmerged and unscheduled —
  `replacement/rc-2-enter-battlefield` and `phase-ld-part-b`. Say what they are
  or delete them.

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
