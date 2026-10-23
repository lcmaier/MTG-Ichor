# Route codes renamed — the item 6 audit's pass 5c (2026-10-23, #282)

**A record, not a plan.** Until pass 5c the phases and route rows still to
come were cited by codes: a track's letters and a number (`RS-2`, `CP-1`), or
a roadmap row's ID (`B2`, `A6s`). Pass 5c gave each a name saying what it
delivers, cited hyphenated in backticks, and swept every live mention
(`engineering-practices.md` §4.2 has the rule). Commit messages, PR titles
and the rest of `plans/archive/` still read in the old codes; this table is
where a reader of those finds the name. Landed codes (`TR-5h`, `CV-1b`,
`RH-3` …) were never renamed, and the board's "Phases by name" lists them.

Two changes went beyond a rename, both decided at #282's review (the owner,
2026-10-23): `RS-2` split in two, and `B2` widened from Commander to the
60-card constructed formats as well.

## The phases

Each is defined where it was: its architecture doc's sizing table, or its
roadmap row.

| Was | Cited as | What it delivers | Defined in |
|---|---|---|---|
| `RS-2` | `cant-be-targeted` | The read side of "can't" at targeting: hexproof, shroud, protection's targeting half — the `BeTargeted` arm | `cant-effects-architecture.md` |
| `RS-2` | `cant-cast-play-or-activate` | The read side of "can't" at casting, playing a land and activating — the `Cast`, `PlayLand` and `ActivateAbility` arms, designed beside `permission-design` | `cant-effects-architecture.md` |
| `RS-3a` | `combat-restrictions` | CR 508.1c and 509.1b: menace, the evasion family, landwalk, protection's blocking half, "can't attack" and "can't block" per creature | `cant-effects-architecture.md` |
| `RS-3b` | `combat-requirements` | CR 508.1d and 509.1c: "attacks each combat if able", goad, "can't attack alone", and the search that obeys as many requirements as it can | `cant-effects-architecture.md` |
| `RS-4` | `cant-pay-costs` | CR 614.17b: costs that can't be paid — Yasharn, Platinum Emperion | `cant-effects-architecture.md` |
| `CV-3` | `token-copies` | "Create a token that's a copy of", and CR 707.10f's permanent-spell copy that becomes a token | `copy-effects-architecture.md` |
| `CV-4` | `spell-copies` | A copy of a spell on the stack, its new targets, and CR 707.10a's cease-to-exist — Fork, then Zada | `copy-effects-architecture.md` |
| `CV-5` | `double-faced-cards` | CR 712: a back face, transforming, modal double-faced cards cast either way | `copy-effects-architecture.md` |
| — | `split-and-adventure-style-cards` | The other characteristic sets that rode on `CV-5`'s route step: split, flip, Adventure, Prototype, Omen | `copy-effects-architecture.md` |
| `CV-6` | `face-down-and-morph` | CR 708: Layer 1b, the 2/2 with no characteristics, casting face down and turning face up | `copy-effects-architecture.md` |
| `CV-7` | `merging-and-meld` | CR 729 and 712.4: a permanent of several cards, mutate and meld | `copy-effects-architecture.md` |
| `CP-1` | `hybrid-and-phyrexian-payment` | Paying hybrid and Phyrexian mana, and {X} in an activated ability's cost | `cost-architecture.md` |
| `CP-2` | `remaining-costs` | The cost arms neither checked nor paid: discard, remove or add counters, exile from a graveyard, with amounts that are X or that the board decides | `cost-architecture.md` |
| `MA-7` | `mana-ability-costs` | Every cost a mana ability can have, read into the castability check | `mana-architecture.md` |
| `MA-3` | `any-color-plus-painlands-plus-signets` | A color chosen as the ability resolves, a mana ability that does more than add mana, a mana ability with a mana cost | `mana-architecture.md` |
| `MA-4` | `chosen-color-plus-could-produce` | A color chosen as the permanent enters, and "could produce" | `mana-architecture.md` |
| `MA-5` | `convoke-delve-improvise` | Paying with permanents or cards | `mana-architecture.md` |
| `MA-6` | `mana-payment-solver` | The solver that pays for a person's seat, and item 211's offer | `mana-architecture.md` |
| `PM-0` | `permission-design` | Who may cast, play or activate what, from where and when: the design, with a card's characteristic sets beside it | `permission-architecture.md` |
| `PM-1` | `command-zone-casting` | CR 903.8: casting a commander from the command zone, permission's first build | `permission-architecture.md` |
| "PM's later builds" | `casting-from-other-zones` | Flashback and the other casts from elsewhere, alternative costs an effect provides | `permission-architecture.md` |

## The route rows

Each is a row of `roadmap-v2.md` §3a.

| Was | Cited as | What it delivers |
|---|---|---|
| `A6s` | `route-review` | The route review after triggers: the v1 gaps the 2026-10-13 read found, each given an owner and a slot or declined |
| `A6f` | `information-model-design` | The information model's design: per-viewer knowledge, the query, the redeal, with a cost ceiling |
| `A4l` | `divided-among-targets` | CR 601.2d: an effect divided among its targets |
| `A4d` | `rust-notes` | The engine map's tour annotated for a reader coming from another language |
| `B2` | `game-formats` | Commander, Standard, Pioneer, Modern, Legacy and Vintage as data: starting life, deck limits, the commander's designation, a card's legality per format |
| `B3` | `multiplayer-rules` | What four seats still lack of CR 800 and 802: a departed player's costs and choices, the defending player |
| `B4` | `information-model` | The information model's build |
| `B7` | `breadth-features` | `roadmap-v2.md` §4's lattice: card features off the spine, planned with Phase 8's cards |
| `B8` | `as-though-effects` | CR 609.4's "as though", in a permission half and a payment half |
| `B9` | `mana-provenance` | Each unit of mana keeps its source's facts: restricted mana, {S}, "mana from a Treasure" |
| `B13` | `ironworks-loop` | The Krark-Clan Ironworks loop, an integration test across triggers, mana and permission |
| `B14` | `loop-detection` | The draw on proof (CR 104.4b), with the shortcut (CR 731.2) designed beside it |
| `C0` | `card-layer-cleanup` | The card layer in order before breadth: shared ability constructors, cards filed by first printing |
| `C1` | `rulings-for-every-card` | Every registered card's rulings, each linked to a passing test |
| `C2` | `text-changing-effects` | Layer 3, CR 612 |

## The rows that grouped a track

These had no work of their own once their phases had names, so they are
cited by subsystem in prose, and each row's first cell is its subsystem.

| Was | Now |
|---|---|
| `B1` | the cost phases, `cost-architecture.md` |
| `B5` | `combat-restrictions`, `cant-pay-costs`, `combat-requirements` |
| `B6` | the copy phases, `copy-effects-architecture.md` |
| `B11` | permission, `permission-architecture.md` |
| `B12` | the mana phases, `mana-architecture.md` |
| `RS-3` | `combat-restrictions` and `combat-requirements` |
