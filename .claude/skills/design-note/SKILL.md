---
name: design-note
description: Write a phase's design note at its design stop - re-count the items, count the printed card families before shaping any new type, arm, field or prompt kind, put each decision to the owner with its tradeoffs, have the owner-review agent read the note, and open a draft PR. Use when a brief asks for a design stop, and before building anything that adds or reshapes a type.
---

# The design note

The owner reviews a design before any code is written. A note is good when
every decision in it is a real choice, shaped for everything the rules and
the printed cards ask of it, not only for the cards the brief names. Do these
in order, then stop.

## 1. Intake and re-count

- `python plans/check_state_of_play.py --slotted <name>`, then each item's
  body; take each item or re-slot it.
- Re-count each item's **Sized:** line against the tree, code and tests apart
  (`plans/engineering-practices.md#sizing-a-phase`).

## 2. Count before you shape

Every decision that adds or reshapes a type counts what the type must hold
before any option is written (`plans/engineering-practices.md#what-a-new-type-must-express`).
A type here is a struct or enum, an arm of one (a `ChoiceKind`, a `Rewrite`
or `EventPattern` arm), or a field of either.

- **The rules**: the rule that states the thing, and the rules that watch it
  (`plans/engineering-practices.md#the-rules-pass`).
- **The printed cards**: one Scryfall query per family, through curl with a
  user-agent header, `total_cards` read off page one, under
  `game:paper -is:funny`. `o:` skips reminder text and `kw:` counts a keyword.
  Search every axis the rule names, and try two phrasings before saying none
  exists: since 2024 the text says "this creature", not the card's name.
- **The rulings** of two or three cards in each family, read for timing and
  for which object is which.
- **The table**, at the head of the decision, ahead of the options: the
  families grouped by what the shape must do (never by card), each with its
  count, an example, and what holds it today. The query and the date go
  under it.

A shape drawn from the cards the brief happens to name is the miss this step
exists for. A decision with no printed family, such as a rename or a type no
card reaches, says so in one line, with why.

## 3. The decisions

- Each in `/review-round`'s format: the problem the options share, each
  option's code shape, performance and upkeep, then the recommendation.
  Number them from 1, ask only genuine choices, and fix a defect you found
  instead of asking about it.
- Lead with where the work sits on the event path. Gloss every item number,
  row and phase code once, and cite docs by heading anchor.
- The note is `plans/handoffs/<name>.md`; regenerate the board with
  `python plans/check_state_of_play.py --write`.

## 4. A fresh read

Launch the `owner-review` agent at its design stop: the note's path, and
nothing about why the decisions are what they are. Fix what it finds, and put
a genuine choice it raises to the owner.

## 5. The draft PR, then stop

- `gh pr create --draft`, its body mapping each decision to the code it
  changes, with `file:line`.
- In chat: each decision in a line with its recommendation, and the size.
  Then stop. The build starts on the owner's answers, on the same branch.
- Each review round on the note runs as `/review-round`, and a decision it
  reopens is counted again as in step 2.
