# The authoring check — 20 "authorable" cards written against the engine

> **Status:** run 2026-10-26, the census's second step (`README.md`). Twenty
> cards drawn at random from the census's "authorable" bin
> (`results/authoring-check-sample.json`) were written in a throwaway crate
> outside the tree and played through the engine's real paths: cast from hand
> with exact mana, `activate_ability`, `process_declare_attackers`,
> `advance_turn` and `execute_action(Destroy)`. The crate is not kept: it was
> written against the vocabulary of #297, which closed unmerged, so it no
> longer builds. This table is its record.

**7 of 20 pass.** A works; B is a type that exists and does the wrong thing;
C cannot be written. The three C\* compile and are refused or wrong at run
time, so a stricter reading is A 7, B 5, C 8. Either way the census was wrong
on 13 of 20.

| Card | Class | What happened |
|---|---|---|
| Enormous Baloth | A | Cast for exactly {6}{G}, enters as a 7/7 |
| Cavalcade of Calamity | A | The attack trigger, the damage following the attacked seat at three players, and the ruling that it still deals damage after the attacker is destroyed |
| Cloudshredder Sliver | A | Two keyword grants; the opponent's Sliver gets neither |
| Dogged Pursuit | A | At four players, lives end 21, 19, 19, 19 |
| Shell Shield | A | Kicker and "if this spell was kicked" |
| Horror of Horrors | A | Sacrifice a Swamp to regenerate; a Forest can't pay |
| Pirate Hat | A | A granted triggered ability and "Equip Pirate {1}" |
| Recruit the Worthy | B | Buyback is offered and paid, and the card still goes to the graveyard: nothing reads it at resolution (CR 702.27a) |
| Rune of Might | B | On an Equipment, the granted "Equipped creature gets +1/+1 and has trample" does nothing: a static ability granted by a static ability derives no effect ("Before Layers" item 7's open half) |
| Mr. Orfeo, the Boulder | C\* | `AmountExpr::TargetPower` is refused at resolution; doubling a negative power (CR 701.10c) can't be written, since amounts are unsigned |
| Kitsune Palliator | C\* | The players' half of "each creature and each player" is refused (item 94) |
| Wargling | C\* | "Whenever … while …" written as an intervening "if" is re-checked at resolution, which its ruling says it is not |
| Venom Blast | C | "It deals damage": the damage instruction names no source, and "its power" is `TargetPower` |
| Smashing Spree | C | No filter means "is attacking" |
| Whisk Away | C | No filter means "attacking or blocking" |
| Bounty Hunter | C | No bounty counter |
| Hell Swarm | C | "All creatures get −1/−0": an amount can't be negative |
| Afterburner Expert | C | No "activate only once" restriction, and nothing names an exhaust ability |
| Killer Service | C | No amount counts opponents, and nothing pays mana during a resolution ("you may pay {2} … If you do") |
| Hapatra, the Desert Fang | C | No "greatest mana value among", and no "up to one target" per opponent; read off the types, not compiled |

## What the misses have in common

- **Parameters inside a need**: the sign of a pump, a derived amount ("its
  power", greatest mana value, number of opponents), the counter kind, and
  the damage source.
- **Target qualifiers**: "attacking", "attacking or blocking", one target per
  opponent.
- **A type that exists but does nothing**: Buyback, `TargetPower`, `EachOf` on
  a prevention shield, a static granted by a static. 4 of the 13 misses; only
  running the card shows them.
- **Ruling-level meaning**: "while" is not CR 603.4's intervening "if".
- **Optional costs paid during a resolution**, which the census read as the
  card's own costs.
