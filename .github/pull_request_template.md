<!--
The /pr-handover skill fills this in. `gh pr create --body-file` does not
read this file, so the skill reads it instead; the GitHub web form shows it
as the starting body.
-->

## What changes in a game of Magic

<!-- Two or three plain sentences: what a player would see differently, or
"nothing a player sees" and why it still matters. No item numbers or phase
codes here. -->

## Changes by concept

<!-- One entry per concept, in commit order. Each: what was wrong, with a
concrete card or board; what changed; why this way. Item and row numbers go
in brackets at the end, never as the description. -->

- **Concept** (`commit`): what was wrong, what changed, why. [items]

## Tests to read first

<!-- The tests that state the new behavior: file, test name, and the rule or
ruling each proves. Say so when a test stages its target instead of casting
from hand. -->

## How to read this PR

| Read closely | Skim | Mechanical |
|---|---|---|
| New types and signatures, decisions, rules logic | Bodies that follow from those | Renames, sweeps, generated files |

## Size

<!-- Code plus tests (`mtgsim/src`, `mtgsim/tests`), the sizing band's
measure, and the total with docs, both against the base. -->

## Gates

<!-- Each gate run and its result; each gate skipped and why it cannot fail
here. The A/B, the budget and the floors when the engine changed. -->

## Review before handover

<!-- What the owner-review agent found, and what was done about each finding;
anything left for the owner to decide. -->
