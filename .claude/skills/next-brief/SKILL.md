---
name: next-brief
description: Write the brief for the next PR after the owner merges one. Use when the owner says a PR merged and asks for the next brief, the next leg, or where things stand.
---

# The brief for the next PR

The owner pastes the brief into a fresh session, so it is that session's
whole contract. Its top must be something the owner can approve at a glance;
everything below it is for the session.

## First, from the merged tree

- `git fetch`, then read `origin/main`'s SHA and `gh pr list --state open`.
- Read where the route stands: `plans/state-of-play.md`'s critical path, the
  `roadmap-v2.md` row the next work sits on, and any `plans/handoffs/` file
  for a detour still running.
- Every count, size and SHA in the brief is read off the tree today, never
  carried from memory or an older brief.

## Where we stand, in chat, before the brief

A short ordered list of the next few PRs by plain name, one line each saying
what it delivers, with the one about to be briefed marked. Gloss every item
number, roadmap row and phase code the first time it appears.

## The brief

**The owner's five lines**, at the top, in plain words with no unglossed codes:

1. **What changes in a game of Magic**, or "nothing a player sees", and why it
   still matters.
2. **Why now**: what it unblocks, or what it must come before.
3. **Done means**: the observable result, not the steps.
4. **Size**: code plus tests against the 1,500–2,500 band
   (`plans/engineering-practices.md#sizing-a-phase`), and how it was counted.
5. **Detour budget**, when the work is an audit, a cleanup or a rename off the
   route: how many PRs it may take, and that going past them needs the
   owner's yes. On the route, say which row instead.

**For the session**, below the five lines:

- the branch (`<subsystem>/<name>`, `plans/engineering-practices.md#naming-a-phase`)
  and its base SHA;
- what to read first, in order, by file and heading anchor;
- the intake: `python plans/check_state_of_play.py --slotted <name>`;
- whether there is a design stop, run as `/design-note` (the note first, a
  draft PR, stop for the owner's review);
- the gates that can fail for this work, and the close-out's deliverables;
- that it ends with `/pr-handover`, and that merging is the owner's call.

A design's decisions are numbered from 1, and no heading states a count.

## Then stop

Give the brief in chat for the owner to read, edit and paste. Do not start the
work in this session unless the owner says to.
