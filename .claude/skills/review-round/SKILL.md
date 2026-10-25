---
name: review-round
description: Handle one round of the owner's review comments on an open PR - inline code comments, numbered answers to decisions, or questions about what the PR does. Use whenever the owner leaves review notes.
---

# A review round

The round is the comments. Fix what the owner named, answer what they asked,
push, and report. Anything else you notice gets one line offering it, not the
work, unless it blocks a comment.

## New work raised in review

When a comment asks for something beyond fixing this PR's own change (a
neighboring cleanup, a new rule, another struct with the same problem), ask
"this PR or the next?" in one line before doing it, unless it blocks the
merge. The default is the next PR, so a round stays the size of its comments.

## Answering

- **Start from Magic, then the code.** For "walk me through" or "I don't
  understand": what a player would see, then where it sits on the event path,
  then `file:line`.
- **Gloss everything once.** Every item number, roadmap row, phase code and
  engine term the first time it appears in the reply; cite docs by heading
  anchor, never by section number.
- **A question back to the owner** opens with the problem the options share,
  then gives each option's code shape in a line or three, its performance and
  its upkeep, then the recommendation. Only genuine choices go to the owner; a
  defect you found is yours to fix.
- **An approval of small changes is an approval of those changes.** Do exactly
  them; anything larger is asked first.

## Fixing

- One concept per commit, so the owner can review commit by commit.
- A bug fix shows its own test failing against the pre-fix tree first; then
  run the module's tests and push. CI runs the rest.

## The report

A list in the owner's order: each comment, then the commit that answers it or
the answer itself. Then anything offered in one line, then the push.
