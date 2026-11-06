# The card census — what every Commander card needs from the engine

> **Status:** run 2026-10-26 against that day's tree, Forge at `0dc8c669`
> (2026-10-26), for `route-review`. Moved here from that session's
> scratchpad on 2026-10-27. **Authority:** the counts in `results/` are these
> scripts' output, and the verdicts are `mapping.py`'s table, a reading of the
> engine's enums and a grep. On what exists, `codebase-state.md` wins.

## What it measures, and how far to trust it

Forge's card scripts name each card's effects, triggers, statics, costs and
keywords. `parse_forge.py` turns each script into a set of needs
(`api:Pump`, `trig:Attacks`, `kw:Kicker`, …), and `mapping.py` gives each need a
verdict: built, missing with the roadmap step that builds it, or unclear. A
card is "authorable" when every need is built.

**A "missing" verdict is reliable; an "authorable" one is not.** Calibration
held on the registered cards (210 of 213 authorable, the three misses real).
But the census matches a need's name, not its parameters, and an authoring
check that wrote 20 random "authorable" cards against the engine passed 7
(a throwaway crate, 2026-10-26, not kept). The census's 48% authorable over 32,080 Commander-legal
cards is closer to 17% (somewhere in 9–27% at that sample size). The blocker
ranking, which reads only the missing verdicts, stands.

What the census cannot see, from the authoring check: a need's parameters (the
sign of a pump, an amount such as "its power", a counter kind, the damage
source), target qualifiers ("attacking creature"), a type that exists but does
nothing at resolution (Buyback, `AmountExpr::TargetPower`), and ruling-level
meaning ("whenever … while …" is not an intervening "if").

## Forge's license

Forge is GPL-3.0. **Its scripts are fetched at run time and never committed**:
nothing in this folder contains Forge text, only need names and counts.

## Re-running it

From an empty scratch directory, with the scripts run by path:

```bash
git clone --depth 1 --filter=blob:none --sparse https://github.com/Card-Forge/forge.git forge
git -C forge sparse-checkout set forge-gui/res/cardsfolder forge-gui/res/tokenscripts
python -I parse_forge.py forge/forge-gui/res/cardsfolder cards_needs.json
python -I fetch_scryfall.py all_cards.json
python -I fetch_first_prints.py first_prints.json
python -I census.py cards_needs.json all_cards.json registered_map.tsv all
python -I by_set.py cards_needs.json first_prints.json
python -I precons.py cards_needs.json precons.json
python -I residue.py cards_needs.json all_cards.json 80 --examples
```

`registered_map.tsv` maps each registered card to its Forge script, for the
calibration: `match_registered.py` writes it from `git -C forge ls-files` (paths
relative to `cardsfolder`) and a list of the registered names, one per line,
from `mtgsim/src/cards/registry.rs`. The Scryfall fetches take several minutes
each and sleep between pages.

`residue.py`'s `80` is how many unclear needs it lists. **When the engine
moves, `mapping.py` is what goes stale.** Re-read a verdict against the tree
before quoting a count it produced. Its owners for modes, "unless pays" and
costs to attack were renamed to their rows after the run (`route-review`,
2026-10-29), so `results/` still reads "route-review: modal" and the like.

## The results

- `results/census-2026-10-26.txt`: the calibration; Commander-legal cards by
  verdict; each blocker by the cards it blocks and blocks alone; the unlock
  curve, building the step that frees the most cards next; the same for the
  5,000 most-played (EDHREC rank); and the unclear residue.
- `results/by-set-2026-10-26.txt`: each card by its first printing, in release
  order. Of the 37 steps and backlog entries the census names, 22 are first
  needed by a set printed in 1993–1994, so authoring in release order meets
  most of the route in its first year of sets. Its first line's 136 first
  printings with no Forge script are 95 unreleased cards (Star Trek, one
  Marvel card), 4 tokens and dungeons, 7 meld results Forge scripts under their
  front halves, 29 Secret Lair commanders and one Unfinity card: 37 legal in
  Commander, one of them in `tdc` (Baldin, Century Herdmaster), which the
  decks' gap census reads against the tree like every other.
- `results/precons-2026-10-26.txt`: each Commander product since 2018, ranked by
  how many distinct steps and mechanics its decks need, with Starter Commander
  Decks' (`scd`) and Tarkir: Dragonstorm Commander's (`tdc`) needs by step.
  `precons.py` re-ranks the products when the card work after v0.5 picks its
  next ones.

## The gap census

**Every unregistered card of the five lists, written against the tree at #314's
merge and played, 2026-11-06.** It replaced `pilot-2026-11-06.md`. Where the census
above reads Forge's scripts and matches a need's name, this one writes each card, so
it sees parameters and refusals. `gap_census.py`'s docstring says what each record
holds and what proves a card writable.

- `results/gap-census.md`: the rating, each deck's cards by step, each row's
  cards, each gap and each card, generated.
- `results/gap-census-records.jsonl`: one line a card, which `report` reads.
- `results/gap-census-composition.txt`: each card's written tree, for a reader
  to hold against its text.

**How far to trust it.**

- **It sees a card's blockers only up to its first row not built.** A card waiting on
  `information-model` may hold another blocker behind the search, so each row's
  card-by-card hunt finishes the list. No card's list is complete past its first row.
- **What is proved is that writable cards are writable.** Nothing proves a blocked
  card needs only what it lists: a row's name on a card with no clause to write is
  a reading of its text. Writing corrected the reading both ways: a check land read
  as blocked is writable, and a graveyard count read as an amount is a panic.
- **A board asserts the author's reading of the card.** Removing each piece shows the
  board notices the piece, never that the piece means what the card says.
- **Rulings are unread,** games ran at two seats, and random play catches a refusal
  but not a clause that silently does nothing. Those are `rulings-for-every-card`'s,
  `multiplayer-rules`' and a row's own tests.
- **Loops:** none of the 378 can close a mandatory loop alone (CR 104.4b), judged by
  reading and not by play. The nearest is Colossal Grave-Reaver with Stitcher's
  Supplier, which ends when the library does. Two cards from two decks at one table
  were not read for this.

**Re-running it.** `report` needs only the records, so a row landing clears its
cards with `python plans/references/card-census/gap_census.py report`. `ingest` needs
a crate: `pilot_fetch.py` fetches the cards, `gap_skeletons.py` writes each printed
face, and each card is written beside its board and its needs. The crate is not
kept, since no card lands in the tree: each card's written tree stays in
`gap-census-composition.txt`.
