# Sizing calibration: how far estimates ran from what landed

**Generated** by `sizing-calibration.py`, read at #313's merge (2026-11-06); a
dated record, not a live count. These are the estimates a doc or PR body states
from RH-1 (#243) on, the first phases sized the way the later ones were. Measured
from git: code plus tests, lines added across the merge, review rounds included.
A PR body's own figure can differ, and #307's and #308's are added plus removed:
#307's "2.2×" is 1.70× by lines added.

**How `roadmap-v2.md`'s Sizing section uses each kind:**

- **Set at a design stop:** the top is the expected size, and about 0.5–1.7 of it
  the range. Most of these already carry code at ×1.5–2.
- **A plan's sum carried at code ×1.5–2 plus tests:** about 1.1× its top.
- **A plain sum of `**Sized:**` lines:** about 3× its top, 2.1–3.7, on only four
  rows, and only for the items it lists, so add the feature itself.
- **A route's PR count before any design:** about 3× its top, 3.5–4× for a whole
  subsystem and 1.5–2.5× for a smaller row.
- **A PR count set where the row was designed:** it holds. TR-5 is the one that
  doubled, and TR-3 and the item 6 audit grew by about half.

## Lines

### Set at a design stop

| PR | Phase | Estimate | Measured | × its top | Set by |
|---|---|---:|---:|---:|---|
| #243 | RH-1 | 1,900–2,300 | 2,814 | 1.22 | `replacement-architecture.md`, at the multiplier |
| #244 | RH-2 | 1,040–1,235 | 655 | 0.53 | `replacement-architecture.md`, at the multiplier |
| #245 | RH-3 | 1,390–1,620 | 1,279 | 0.79 | `replacement-architecture.md`, at the multiplier |
| #251 | TR-4a | 2,120–2,440 | 2,104 | 0.86 | A6p's re-count |
| #252 | TR-4b | 1,150–1,850 | 1,650 | 0.89 | A6p's re-count |
| #253 | TR-4c | 1,560–1,750 | 1,579 | 0.90 | A6p's re-count |
| #254 | TR-4d | 1,175–2,040 | 1,690 | 0.83 | A6p's re-count |
| #255 | TR-5a | 1,380–1,600 | 1,622 | 1.01 | its design |
| #256 | TR-5e | 1,465–1,680 | 1,551 | 0.92 | its design |
| #257 | TR-5f | 1,770–1,995 | 2,544 | 1.28 | its re-count |
| #258 | TR-5b | 1,630–1,850 | 2,209 | 1.19 | its design |
| #259 | MA-2 | 1,320–1,360 | 1,908 | 1.40 | its design |
| #261 | TR-5c | 2,045–2,370 | 2,630 | 1.11 | its design |
| #262 | TR-5h | 1,630–1,920 | 2,145 | 1.12 | its design |
| #263 | TR-5d | 880–1,000 | 1,644 | 1.64 | TR-5h's three designs |
| #264 | TR-5g | 1,330–1,510 | 2,081 | 1.38 | TR-5h's three designs |
| #267 | CL-1 | 1,350–1,750 | 1,340 | 0.77 | #266's design |
| #268 | TR-6 | 1,305–1,720 | 1,190 | 0.69 | its design |
| #269 | TR-7 | 975–1,300 | 1,110 | 0.85 | its design |
| #307 | `options-named-by-what-they-are` | 440–760 | 1,294 | 1.70 | #306's decision 4 |

20 rows: mean 1.06, median 0.97, range 0.53–1.70.

### A sum with the ratio already on it

| PRs | Row | Carried | Measured | × its top | Set by |
|---|---|---:|---:|---:|---|
| #250 | A6m's second PR | 650–1,150 | 1,125 | 0.98 | its split at #240 |
| #284 | `history-and-frame-records` | 1,700–2,000 | 1,823 | 0.91 | #283's plan |
| #288 | `resolution-context-rider` | 130–160 | 202 | 1.26 | #283's plan |
| #289, #290 | `trigger-matcher-merges` | 1,350–1,800 | 2,517 | 1.40 | #283's plan |
| #291 | `one-fact-one-type` | 1,700–2,100 | 2,004 | 0.95 | #283's plan |
| #293, #294 | `batches-durations-and-control` | 1,750–2,100 | 2,445 | 1.16 | #283's plan |
| #308 | `divided-among-targets` | 1,200–1,650 | 1,383 | 0.84 | `choices-architecture.md`, the backlog's 500–700 at #307's ratio |

7 rows: mean 1.07, median 0.98, range 0.84–1.40.

### A plain sum of `**Sized:**` lines

| Part | Estimate | Measured | × its top | Set by |
|---|---:|---:|---:|---|
| #310, the hunt's six fixes | 360 | 765 | 2.12 | item sizes of about 155 and 205 |
| #305, `scripted-answers-by-identity` | 700–900 | 2,353 | 2.61 | its brief, in changed lines |
| #313's ladder | 100–150 | 450 | 3.00 | its brief, item 377; its three commits |
| #313's item 378 | 30–50 | 186 | 3.72 | item 378; its two commits |

4 rows: mean 2.86, median 2.81, range 2.12–3.72.

## PR counts

### A route's PR count, before any design

| Row | Guessed | Landed | × its top | What landed |
|---|---:|---:|---:|---|
| Item 5's remainder | 3–5 | 20 | 4.0 | RC 6, RD 4, RE 10 |
| Item 6, triggers | 4–6 | 21 | 3.5 | TR-1 to TR-7 |
| `card-layer-cleanup` | 1–2 | 5 | 2.5 | #301–#305 |
| Item 7, CR 613.8 | 1–2 | 3 | 1.5 | LI-1 to LI-3 |

4 rows: mean 2.88, median 3.00, range 1.50–4.00.

### A PR count set where the row was designed

| Row | Set | Landed | × | What landed |
|---|---:|---:|---:|---|
| RD | 4 | 4 | 1.00 | RD-1 to RD-4 |
| RE | 9 | 10 | 1.11 | RE-1 to RE-10 |
| CM | 5 | 5 | 1.00 | CM-0 to CM-4 |
| RH | 3 | 3 | 1.00 | RH-1 to RH-3 |
| TR-3 | 2 | 3 | 1.50 | TR-3a to TR-3c |
| TR-4 | 4 | 4 | 1.00 | TR-4a to TR-4d |
| TR-5 | 4 | 8 | 2.00 | TR-5a to TR-5h |
| `readiness` | 2 | 2 | 1.00 | #299, #300 |
| The item 6 audit | 20 | 28 | 1.40 | #270–#300; #270's plan said 13–20 |

9 rows: mean 1.22, median 1.00, range 1.00–2.00.

## Overhead, the eight weeks to #313

### What was merged

By branch prefix, with `ROUTE_PRS` on the route: a route PR is one that builds a row of the Sizing table or the cleanup plan whose last rows joined it, so work is counted the way the rows left are.

| Fortnight from | route | dev GUI | audit | process and docs | detour | All |
|---|---:|---:|---:|---:|---:|---:|
| 2026-09-12 | 33 | 1 | 13 | 5 | 10 | 62 |
| 2026-09-26 | 10 | 24 | 3 | 5 | 1 | 43 |
| 2026-10-10 | 25 | 1 | 17 | 0 | 1 | 44 |
| 2026-10-24 | 19 | 1 | 3 | 3 | 2 | 28 |
| **All** | **87** | **27** | **36** | **13** | **14** | **177** |

87 of 177 merge commits on the route: 2.0 merged PRs for each route PR, and 10.9 route PRs a week. A PR rebased in has no merge commit and is not counted. `setup/` is the dev GUI's own track; `state/` is the clone's.

A phase PR's size over the design-stop rows: median 1,647, mean 1,752.
