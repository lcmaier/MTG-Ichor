# MA-2's dev GUI notes — note 2 waits on the owner

`codebase-state.md` item 276, branch `devgui/ma-2-notes`, its draft PR.
**Delete this file in the commit that closes item 276.**

## Done on the branch

- **Note 1, the why panel's width.** The title row wraps, and a drag widens
  the panel only to half the room the log leaves (`why_long_title.png`).
- **Note 3, sources not offered yet.** `ui::why::not_offered_yet`, drawn
  greyed in the payment step (`mana_not_yet.png`).

## Open: note 2, a prevented event leaves nothing in the log

A design question, put to the owner in the PR body with the options and
their tradeoffs: (A) build TR-5d's `DamagePrevented` record now, the record
and its log line only, as #231 brought `Targeted` forward
(`triggers-architecture.md` §3.12 item 16); (B) the log reads the rider's
cause; (C) leave it to TR-5d. Recommended: (A).

## Left once the owner decides

- Build the chosen option, its tests, and its doc rows: item 16 and TR-5d's
  row in `triggers-architecture.md`, `setup-architecture.md` §7c's "Why
  didn't Grizzly Bears die?", and the prevention leg in
  `replacement-architecture.md`. Under (A) or (B), redraw the pictures, and
  read the fuzz dumps: the counters should match, and only the log lines
  should differ.
- Close item 276: stub it, archive its body, and slot what is left of it.
- Mark the PR ready.
