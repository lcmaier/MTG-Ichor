# Trace pages

A **trace page** is a hand-authored HTML file under `plans/traces/` that walks
two or three real board states through the engine call by call, every read
labeled by what it consulted (the board, a hypothetical frame, a registry, a
player), and ends in a table of where those reads differ. A design doc says
what the engine should do and a test asserts one outcome; the page shows the
**path between them**, which a diff cannot show and a reviewer cannot
reconstruct. When a phase owes one is `engineering-practices.md#trace-pages`;
this doc is how to write one, and the list of those written.

## Writing one

- **Its examples are the phase's findings, not its feature list.** A page that
  walks the happy path explains the feature; one that walks the board the
  review argued about explains the phase.
- **It may walk a design the phase rejected**, beside the one that shipped,
  where the finding is "this other reading is wrong" and a test can only
  assert that the wrong thing does not happen.
- **Budget two to three hours.** That is the right cost at a phase's close and
  the wrong one mid-debugging, which is what the trace sink is for.
- **Do not generate a page from every test**, and do not diagram code that is
  about to change.

## Naming

`<phase>-<claim>.html`, kebab-case, where the claim is the sentence the page
proves: `rc-4b-entering-is-one-event`, not `rc-4b-traces`. A phase still to
come is named as `engineering-practices.md#naming-a-phase` says.

## Structure

In this order:

1. **The map**: the boards, the cards, and which trace answers which question,
   with a mermaid flowchart of the path and the phase's new nodes colored.
2. **Lettered traces**, A, B, C…, each a numbered walk; every step names the
   function and what it read.
3. **Where the reads differ**: the comparison table, which is the payload.
4. **A closing section that ties it to the phase**: what changed, trace by
   trace, or what is still open.

## Pinned, self-contained, linked

- **Pinned, never maintained.** The first comment in the file, in its `<head>`
  right after the charset and viewport `<meta>` lines, names the commit and the
  frozen CR version. A later change to the path gets its own page, and the old
  one is left alone; that is what makes the detail safe to write.
- **Self-contained**, using only the CDNs the artifact sandbox allows: fonts
  from Google Fonts, mermaid from jsDelivr.
- **Linked from the owning architecture doc's phase entry and the phase's
  `codebase-state.md` entry**, and given a row below. The `plans/` markdown
  stays the authority; a page nothing links is a page nobody finds.

## The pages

| Page | Phase | What it proves |
|---|---|---|
| Page | Phase | What it proves |
|---|---|---|
| `rc-4b-entering-is-one-event.html` | RC-4b | four entries through the CR 614.12 look-ahead frame, each read labelled board or frame, and what RC-4b changed trace by trace |
| `cv-1-a-copy-is-a-snapshot.html` | CV-1 | a Cytoshape resolution from the choice to the copy row and back, and CR 707.4's re-copy tearing that row down through the existence check |
| `rc-5-applying-an-entry-can-move-the-board.html` | RC-5 | devour's selection and its nested batch, the zone chain that makes CR 614.13b bite, `frame_of(source)` and §5b's asymmetry, and two entries decided against one board |
| `item-7-an-effect-waits-for-what-it-reads.html` | LI-2 + LI-3, closing item 7 | the CR 613.8 loop's fast and slow paths; the judge answer's four-card layer 4 step by step through `next_ready` and the journal; a condition that *is* a dependency (Simian Clause under Blood Moon, with the sabotage step that shows what `condition_reads` buys) and one that is not (Kird Ape, two layers apart); the read-by-read table |
| `li-1-one-pass-per-board.html` | LI-1 | the `Board` struct field by field, the entry's three routes, and Humility + Citanul Hierophants through the old walk and the pass — the one read that produced the wrong answer, and where it reads from now; a look-ahead entry; a graveyard Keldon Warlord |
| `re-2-a-draw-carries-its-lineage.html` | RE-2 | the first decomposed event: two Thought Reflections through the applied set that travels with it, and the same board without it — a stack overflow at depth two rather than a wrong number; Teferi's exception living in the shape of the event tree instead of a counter; **Alms Collector in both encodings**, the shipped one and the one §9 sized, which is a CR 104.4b loop; three Notion Thieves moving the event's subject and CR 616.1's chooser with it; the read-by-read table |
| `rd-2-a-decision-is-per-subject.html` | RD-2 | the CR 616.1 loop's new unit: two shield counters under two blockers through the per-member loop and the per-subject one, and the first-strike twin that shows the key is the batch; Furnace beside Mending Hands in both orders; a `NextDamage(3)` under sources of 2 and 4 with the allocation asked once; the two boards where nothing is consumed — Safe Passage beside Mending Hands, and a `Once` half chosen against 1 — and the consume-after-apply order that makes them right |
| `rf-a-source-off-the-battlefield.html` | RF | the gather's zone leg read by read: a Colossus in a library while a Bolt resolves, which is why neither library is ever walked (the map, the printed-def precheck, and the two things that *would* walk a zone); the same Colossus second of three in a mill — one batch, one member replaced, the same-zone no-op, the rider after the batch; a Colossus commander sacrificed, which is CR 616.1 twice on one card and CR 701.24c's shuffle of a library the card never reached; the read-by-read table |
| `a4i-a-target-belongs-to-an-instance.html` | A4i | CR 115.3's instance of "target" as the unit a chosen object is filed under: three equal atoms as three instances, "another target" and how its exclusion stacks, CR 608.2b as two rules; the read-by-read table |
| `tr-1-a-trigger-is-matched-at-the-close.html` | TR-1 | the window is the event, matched against one batch's records at its close: Humility and an ETB creature entering as one event, an artifact dying in the wipe that kills the creatures it watches, Wild Growth inside the mana window; the read-by-read table |
| `rg-an-entry-write-changes-what-applies-next.html` | RG | the CR 616.1 loop over an entry, from traced runs: a Bears under Master Biomancer, where the Mutant lives as state and why Humility leaves it; Adaptive Shimmerer under Biomancer beside Doubling Season, premise (d)'s extra gather and the four orders; Kaito beside Oath of Gideon, CR 306.5b on the frame and the feeds table's last row, in both orders, with the gray area the judges split on; the read-by-read table |
| `cv-2b-an-exception-is-checked-without-itself.html` | CV-2b | CR 707.9f's check, from traced runs: Spark Double copying a legend, where each conditional exception is checked on a look-ahead of the copy without it; the Kaito board, where the two exceptions read each other's counters and both apply, so neither CR 306.5b nor Oath of Gideon is gathered after; and a later copy in the same entry taking the exception's counter back |
| `ma-1-a-payment-is-a-matching-not-a-count.html` | MA-1 | "can this player pay this cost" answered as a matching, not a count, on six boards: Everywhere against `{1}{G}`, a Tundra and a Plains under Hall's condition, Mana Reflection on Everywhere, Wild Growth on a Badlands under Mana Reflection, Chainbreaker paying its own `{3}`, eight Plains with Krark-Clan Ironworks and Doubling Cube; the read-by-read table |
| `tr-5g-a-multiplier-is-read-where-its-trigger-looks.html` | TR-5g | CR 603.2d's count read through the dispatcher's kinds of candidate list: Panharmonicon beside Soul Warden (the live list), Gandalf dying beside a Golem (a departure's frame), a granted multiplier whose granter dies (a snapshot); which list answers which arm |
| `tr-4d-control-is-announced-by-a-sweep.html` | TR-4d, written at the item 6 audit | CR 603.10d's look-back with no performer to watch: a steal ending at cleanup, which the gate's second half announces; the first control row, kept before it writes, at both sites; Ray of Command's rows with no batch between, where a change not yet announced keeps no newer frame; each fix taken out once; which frame each look-back reads |
| `tr-5h-every-history-read-is-a-scan-of-one-log.html` | TR-5h, written at the item 6 audit | the history log's three readers: Vengeful Warchief's "first time each turn" and Paladin of Atonement's "last turn", the damage facts with a flicker, and the layer memo's keys beside each write that moves them, with the note and the reframe each taken out once; what each read scans, and what it read before |

## The trace sink and the engine map

- **The trace sink** (`mtgsim/src/state/trace.rs`) records what a page records
  by hand, off by default at one branch per emit point: each batch, each CR
  616.1 iteration, each top-level layer walk, the decision boundary (the
  prompt, and a rejection between a prompt and its re-ask), and each
  performed event. It is reachable as `fuzz_games --trace DIR`,
  `cli_play --trace PATH` and `test_support::install_trace`, and
  `plans/trace_spine.py` renders one game's lines through
  `plans/traces/viewer.html`. **It makes a page's spine, never its
  argument**: the boards, the question each answers, the "where the reads
  differ" table and the closing section are authored, and a generated page
  marks their places. It is for the question asked mid-debugging, and for
  regenerating a page's rows after a refactor, which shows the diff and is no
  reason to touch the pinned page. `TraceRecord::read` is the writer's inverse,
  and the dev GUI's why panel answers questions about the past from a replay's
  trace (`setup-architecture.md` §7c).
- **The engine map** is `plans/engine-map.md`: the modules and what each owns,
  the chokepoint's arms, the gate legs a new replacement source must extend,
  and one event's path from proposal to triggers, each seam linked to its
  architecture doc and trace page. It is markdown because it is refreshed where
  a page is pinned, and `plans/check_engine_map.py` keeps it true; its
  `--drift` is a refresh's to-do list.
