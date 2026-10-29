# Engineering practices — the rules

Every process rule the project keeps: how a PR is sized, measured, commented,
named and gated. Each is a bold sentence, a line or two, and a pointer to where
its reasoning lives. The engine's invariants are `CLAUDE.md`'s, and each
subsystem's design is its architecture doc's. The history behind these rules,
and the doc this page replaced, are `archive/engineering-practices-records.md`.

**By concept:** writing code is [comments](#comments), [names](#names),
[records and concepts](#records-and-concepts) and [module layout](#module-layout);
writing docs is [line budgets](#line-budgets) and [counts in docs](#counts-in-docs);
designing is [the rules pass](#the-rules-pass) and
[the standing review question](#the-standing-review-question); planning is
[sizing a phase](#sizing-a-phase); cards are [cards and pools](#cards-and-pools)
and [rulings](#rulings); measuring is [measuring](#measuring); closing work is
[the spec database](#the-spec-database), [trace pages](#trace-pages) and
[the spine-phase audit](#the-spine-phase-audit); the GUI is
[the dev GUI](#the-dev-gui); and the checked invariants are
[invariant gates](#invariant-gates).

## How to cite this page

- **Cite a section by its heading's anchor:** `engineering-practices.md#the-close-out`,
  GitHub's anchor for the heading, so the citation says where it leads, a grep
  finds it, and on GitHub it is a link. Never by number.
- **A heading here is a name other files use.** Rename one only with a sweep of
  its citations; `python plans/check_practices_page.py` fails a citation that
  resolves to no heading, and any citation of this page by section number.
- **A rule that needs more than about three lines is two rules, or its
  reasoning belongs in the doc it points to.**

## Line budgets

- **`CLAUDE.md` is 200 lines, hard**, checked by `check_claude_md.py`; this page
  has its own budget, checked by `check_practices_page.py`. A raise is typed
  (`--budget N`), so it is a visible act with a reason attached.
- **An invariant in `CLAUDE.md` is at most three lines plus a pointer**; the
  reasoning, the war story and the rule numbers live in the architecture doc.
- **Adding a section requires removing one, chosen against the whole file**,
  not against whatever sits next to the edit.
- **A war story is not an invariant**, and **status is not content**: landed
  status is derived onto `state-of-play.md` from ✅ headings, never written.
- Why, and the cases: `archive/engineering-practices-records.md#1-claudemd-has-a-line-budget-and-the-budget-is-checked`.

## Comments

- **Comment the *why*, and only where it is not recoverable from the code plus
  one rule number.** `// CR 701.26a — only untapped permanents can be tapped`
  earns its place; six lines restating the next four do not.
- **One rule cite beats a paragraph.** A comment past about four lines is
  reasoning that belongs in `plans/`, with a one-line pointer from the source.
- **Say the rule, never a plans doc's number for it.** A `codebase-state.md`
  item or a doc's private label means nothing in the code; plans docs may cite
  items (the owner, #244).
- **A war story goes in the commit message or the architecture doc.**
- **Doc comments on types and public functions are exempt from brevity only**;
  the why-not-what half still applies.
- **A rule for new and edited code**, never a license to delete comments
  nobody reread; existing density comes down by the sweep below.

### The comment sweep

- **The guard is a sweep, not a length budget**: the stale comments it finds are
  short, true when written, and outgrown by an unrelated change. Run it every
  15–20 PRs and at the close of any phase that moved a measurable.
- **`plans/references/comment-census.py` at both ends, with `--rev`**: the wide
  tier is the count to record, the tight tier the list to read, and the block
  census with its history words is what cuts lines.
- **Re-derive or date every number; then delete what the code now says**, as a
  separate read. A shorter comment that is wrong is not an improvement, so read
  the doc comments of the subsystem the window built for a count or a tense.
- **Run the history-word grep over every touched file before opening the PR.**
- **`CLAUDE.md` carries no comment-length rule**; this sweep is the guard.
- The four sweeps, their tables and what each found:
  `archive/engineering-practices-records.md#21-the-comment-audit--why-the-guard-is-a-sweep-and-not-a-budget`.

## Names

A name is read where it is used: a card file, a match arm, a struct literal,
with no doc comment and often no type prefix. If the use alone does not say what
it is, rename it. Every name a PR adds is read against this list:

1. **Name the noun**: `ThisObject`, not `This`; a flag says it is one,
   `is_state_trigger`.
2. **Say what it does**: `SameInstanceAs(0)`, not `Instance(0)`.
3. **Put the subsystem on a generic word in a shared module**: `TriggerTier`, not `Tier`.
4. **Don't borrow a word someone else owns**: the CR owns counters and targets,
   Phase 10 owns "agent", and "Vehicle" is a subtype.
5. **Don't name a thing by what it isn't, or by a state it can't have**:
   `EachOther`, not `Other`; `NONE`, not `EMPTY`.
6. **Name a type by its contents**: a set of objects alone is `ObjectSet`, not `AffectedSet`, which names the objects and players an effect affects.
7. **Where the card's text has a phrase, use it**: `DoThisOnlyOnceEachTurn`.
8. **Name the players**: a condition about a player says which ones.
9. **A constructor says what it builds**: `IdMap::default()`, not
   `Default::default()`; an empty value that is a claim is a named constant
   (`ObjectSet::NO_OBJECTS`). Clippy's `default_trait_access` gates the first half.
10. **A return names its parts**: a tuple only for a pair that is one thing;
    otherwise a struct with named fields, or two functions.

- **One spelling per concept**: a second type naming the same object takes the
  first's name (`TriggerSubject::ThisObject`, `EffectRecipient::ThisObject`).
- **Retrofit where a PR reaches**: a phase that extends an enum reads its
  existing names against the list and renames the failures in a mechanical
  commit of their own, counted first. Live docs follow; the archive does not.
- The review that asked for each check: `archive/engineering-practices-records.md#2b-names-are-read-at-the-call-site`.

### One-line wrappers

- **"It reads better at N call sites" is not a reason.** A wrapper earns its
  place when it names a question the docs already ask in prose, pins a
  comparison callers would get subtly wrong (`zone_function::functions_in`), or
  is the narrow half of a pair whose wide half must stay reachable.
- **Otherwise inline it; when one applies, the reason goes in its doc comment.**
- **A chokepoint is a different question**, justified by what it makes impossible.
- The worked cases: `archive/engineering-practices-records.md#2a-when-a-one-line-wrapper-earns-its-place`.

## Records and concepts

### One record, never two kept in step

- **Two collections whose entries must stay paired, by position or by key, are
  one record split in two.** Every writer has to remember the other half, and a
  missed half is silent until a reader pairs the wrong two. Put the fields on
  one record. A collection derived and dropped at once is not a second copy.
- **Mirror the CR's shape, so each of its readings is one place in the code.**
  CR 113.7a's last known information is one store keyed by existence
  (`GameState::lki`), and `departure_read` in `engine/triggers/dispatch.rs`
  reads a departing object the two ways CR 603.10 and 603.6c give it: as its
  record framed it for an arm that looks back, and as it is now for any other.
  Two match arms, one per reading. → `triggers-architecture.md` §3.11.

### Key a concept before storing it

- **Ask what the CR keys a concept by, and who owns it, before deciding where it
  lives or how long.** TR-2b chose where last known information lived by how
  long each keeper needed it, and twenty fields followed; the key CR 400.7 gives
  it, the existence, came only with #285's store.
- **A concept with two owners is a finding, and so is a key narrower than the
  CR's.** Each concept's key, owner and holders are the register,
  `cr-coverage-audit.md#4c-concepts-and-their-owners`; `check_type_surface.py`
  fails a new holder of a registered concept until the register has its line.
- **A word a card's text spells has one spelling.** Before a new arm or enum in
  the types a card is written in, ask whether another role already spells its
  word: hold the shared word type, or keep the arm with its reason.
  `cr-coverage-audit.md#4d-words-and-their-spellings` lines every arm, and
  `check_type_surface.py` fails one without its line.

## Counts in docs

- **A count or `file:line` in a doc is generated, or it drives a decision**, and
  the doc says which: the board's counts are regenerated; a sizing is rounded
  and dated.
- **Anything else is described, not counted**: "every `GameAction` variant has
  an arm", not "the 26 variants"; name the function, not its line. A dated
  record ("at TR-2b, 34 variants") is history and stays.
- **It bites in contract sections, `codebase-state.md` item bodies outside
  `**Sized:**`, and module doc comments**; a work-list or handoff may carry
  exact counts, since it is consumed and deleted.

## The rules pass

- **Before deferring a rules question, search the CR for the rule that watches
  the one you are implementing.** Three greps: the chapter that owns the thing
  you change; the chapter that watches it (CR 603 and 614 for an event, 613 for
  a characteristic, 400.7 and 603.10 for a zone change); and the thing's name,
  in the CR's words, across the whole file.
- **The corpus too**: an atom written against one rule without the rule that
  overrides it cannot reach its own board.
- The case it comes from (CR 603.6c, found after RE-7 deferred it):
  `archive/engineering-practices-records.md#8-the-rules-pass--read-the-rules-that-name-the-rule-not-just-the-site`.

### What a new type must express

For a new type, or one a phase grows by an arm or a field (a prompt kind, a
`Rewrite` arm, a template's field), three questions, answered in the design
note ahead of its options, with the count's table and query (`/design-note`).
A shape drawn from the cards the brief names, uncounted, is not ready for review:

1. **What must it express at v1?** List what the CR lets it be, then count the
   cards with a census script over Scryfall's data, calibrated on the
   registered cards first (`plans/references/cast-census.md` §2 is the example).
2. **Which rules let a card override it?** "Unless … says otherwise" (602.2),
   "usually" (601.2), "by default" (701.9b); one that refuses an override "for
   any reason" (305.2b) may be encoded as fixed.
3. **Which card families override it, and how many cards?** A default the engine
   hard-codes is a family with no owner until its first card arrives.

## The standing review question

- **"What does this actually check, and what happens if it runs twice?"** Ask it
  of any claim the engine makes about itself: name the values compared and
  their types in a sentence, and say what a predicate that suppresses, memoizes
  or short-circuits assumes about repeating.
- **When:** any change to `ordering_cannot_change_outcome` or its leaf tables;
  any new `EventPattern`, `Rewrite` or `GameActionTemplate` arm; any
  `debug_assert` standing in for a proof; any "safe because …" whose because is
  a list of cases.
- **A premise that enumerates shapes breaks when the list grows**, so put a
  property on the type whose arms grow (`EventPattern::reads_the_amount`),
  beside the enum. A fact relating two types stays at its caller.
- The three corrections that made it a rule: `archive/engineering-practices-records.md#41-the-standing-review-question`.

## Sizing a phase

- **Size a phase before writing it, and split it in the doc, not in the
  moment.** Implementation PRs run 1,500–2,500 lines.
- **The band counts code plus tests** (`mtgsim/src`, `mtgsim/tests`), with docs
  reported beside it, net of generated files (`state-of-play.md`,
  `rulings-ledger.json`, what `specdb.py build` writes). Track it at every
  commit and flag a crossing at once.
- **Hunt the gaps card by card before counting**: read every clause of every
  card the phase names against the tree, and list each facility its rows do not
  name.
- **Read the intake at the first ticket**: `check_state_of_play.py --slotted
  <name>` lists every open item slotted to the phase; take each or re-slot it.
- **Every PR in a split carries at least one consumer of what it builds.**
- **A brief opens with five plain lines the owner approves**: what changes in a
  game, why now, what done means, the size, and a detour's budget. → `/next-brief`.
- **A detour states its PR budget when it starts.** An audit, cleanup or rename
  off the route names how many PRs it may take; going past needs the owner's yes.
- **Review findings go to `plans/handoffs/<phase>-review.md`**, captured before
  anything is fixed, triaged, and closed one theme per session.
- **`git log main..HEAD` lies after a squash merge**; check content with
  `git diff --stat origin/main HEAD`.
- The phase that made it a rule (RB, at 2.2× the largest before it):
  `archive/engineering-practices-records.md#4-sizing-a-phase-and-splitting-it`.

### Gaps a phase finds

- **An arm the PR's own type opens, with a printed customer and under about
  eighty lines, ships in that PR.**
- **The CR is the customer; a printed card is the test.** A facility the CR
  states is owed whether or not a card prints it: it gets a fixture test and a
  reachability line saying "no printed producer", never "nothing owed".
- **A gap in the phase's own subsystem ships in the phase while the band
  holds.** What is still filed is slotted to a named phase or roadmap row,
  a mechanical fix as much as one that needs a design decision: a gap with
  printed customers waits on the route, never on a card's registration.

### Landing a phase

- **A shipped phase marks its own heading** — `#### <name> — ✅ landed <date>`
  in the owning architecture doc, in the commit that ships it; the board reads it.
- **A landed phase's section is a stub of at most 40 lines**; its body moves to
  `plans/archive/<doc>-landed.md` under the same heading in the landing PR, and
  `check_state_of_play.py --check` fails a longer one.

### Naming a phase

- **A phase has a plain name of a few words**, first wherever it is named; a
  landed phase's old code follows it.
- **A phase still to come is cited by its name, hyphenated in backticks**
  (`combat-restrictions`): two to four words in the CR's words, "plus" joining
  different work. A board-read heading puts it first; a branch is
  `<subsystem>/<name>`.
- **No new codes**, and a split names each part for what it delivers. A roadmap
  row's ID orders work and never names it. Retired codes:
  `archive/route-codes-renamed.md`.
- **The index is generated**: the board's "Phases by name".

## Cards and pools

`cards/registry.rs` builds three pools:

| Pool | Contents | For | Flag |
|---|---|---|---|
| **performance** | a representative board (`PERFORMANCE_POOL`), grown one card per new engine path | A/B-ing an engine change, interleaved in one sitting | `--pool performance` (default) |
| **stress** | every registered card | panics, errors, effect interactions | `--pool stress` |
| **fixtures** | every registered card and the castable fixtures | the debug checks fixtures carry, in CI's checked sitting only; keeps no table | `--pool fixtures` |

- **Register the card**: writing a card and registering it are one act, and
  registration puts it before every check the project has (once the engine
  can play it).
- **`PERFORMANCE_POOL` is representative, not frozen.** A phase that opens a new
  engine path adds one card, deliberately, and re-records both tables in
  `fuzz-record.md`; `performance_pool()` panics on a name no longer registered.
- **Before a word in a card's text becomes structural, find two more cards in
  the bucket that print it.** Structure is the expensive direction to be wrong in.
- **A fixture may be invented. It must not wear a real card's name while
  behaving differently, nor be cited in the corpus or plans as evidence about
  printed Magic.**
- **The pool measures cost; `--require` measures coverage.** The unforced count
  is not a target. Run `--require` once per phase that adds a card and record
  its resolutions beside its copies per deck, since a count compares only per
  copy. An empty `--require` changes nothing.
- **A phase whose consumer is an activated ability measures the activation, not
  the cast**, and the random agent gets no behavioral cap: a diagnostic counter
  instead. Repeatable activations are ordinary Magic and are not kept out of the pool.
- **Say which pool a number came from**; the pools are not comparable.
- **The tables are fixtures, not benchmarks**: seed-deterministic, comparable
  across machines, and `ms/game` is never in them.
- Why two pools, and the readings behind each rule:
  `archive/engineering-practices-records.md#3-two-card-pools`.

### How many cards a mechanic owes

- **Ask whether the rule is defined over one object or several, not for a
  quota.** Tier 1, the rule itself needs two (CR 616.1, 613.7, 614.5, 704.7),
  and one card leaves a branch unreachable. Tier 2, two of different shape,
  varied on the axes the CR names. Tier 3, one is plenty.
- **When a rule needs two objects, check two *registered* cards can produce
  them**: a bespoke fixture can cover an atom the pool cannot build.
- **A second card of the same shape buys nothing**; never "N cards per phase".
- **A gap register is re-measured, not inherited**: a gap named by its cause
  outlives the cause.

## Rulings

- **Every card a phase registers gets a rulings pass, written down.** Each ruling
  is testable now (a test named for its claim), already tested (say which), or
  not yet expressible (name the facility and its owner).
- **It goes in the card's doc comment, under `# The rulings, and where each is
  tested`**, never a module block.
- **Format-variant rulings are dropped, not answered.**
- **After the pass, ask what the card's filter has that no test varies.**
- Fetch with `curl` and a UA header: `/cards/named?exact=…` carries a
  `rulings_uri`.

### The ledger and its gate

- **Every ruling of a registered card has a linked test** (`// RULING: <Card> #<n>`)
  or one of five dispositions, and `check_rulings.py --check` fails one with
  neither. A ruling's `#n` is stored and never reused; drift fails, found by
  `--fetch`; the gate is scoped to cards read or registered after the ledger.
- **A fixture shaped like a printed card cites its ruling the same way**, and
  `--fetch` takes cited cards too.
- **The end state is every official card tested against each of its rulings**,
  pooled cards first (`roadmap-v2.md`, `rulings-for-every-card`); a
  `not-expressible` disposition is interim.
- The decisions' detail is `check_rulings.py`'s docstring; the census and
  why the ledger comes before the parser:
  `archive/engineering-practices-records.md#34a-how-big-is-the-retroactive-half-and-is-the-tool-worth-building`.

### When a ruling and the CR disagree

Adopted at RG's review (the owner, 2026-09-29). Also where two CR rules
disagree on a board their text did not foresee.

1. **A ruling is the default for the card it names.**
2. **The CR's text wins over a contradicting ruling only once Wizards has
   acknowledged the contradiction or its staff sided with the text**; until
   then the ruling is followed, and the conflict is a row here marked open.
3. **Where the CR is silent**, the latest official statement decides; failing
   one, the reading that fits the neighboring rules.
4. **A deviation is one register row, one switch and a test, never an
   exemption**: a test on the ruling's board, annotated
   `// RULING-DEVIATION: <card> #<n> (<id>)`, which `check_rulings.py` accepts
   only when `<id>` is a row below.
5. **A gray area never holds up a PR**: decided by these rules in the PR that
   meets it, recorded here, pinned by a test, and re-read at every new CR version.

| Id | In tension | The engine follows | Switch | Reasoning | Tests |
|---|---|---|---|---|---|
| `lookahead-entry-counters` | CR 614.12, whose look-ahead counts "replacement effects that have already modified how it enters", against Arixmethes, Slumbering Isle's 2020 ruling, under which an entering permanent's "enters with" counters do not change what the look-ahead sees; and, since CV-2b, against Spark Double's 2019 ruling #7, "printed on the card plus one", on a planeswalker whose type hangs on its counters | the CR's text: Wizards called it a contradiction in 2024, the latest staff statement on Arixmethes (2025) reads it by the text, and the rules architect still counted 614.12 among his open issues in 2026 | whether the look-ahead's condition checks read the would-be permanent's counters: `Condition::CounterCount` on the entering object | `replacement-architecture.md` §5e; `copy-effects-architecture.md` §4.1a, "The Kaito board" | `kaito_cast_beside_oath_of_gideon_is_its_controllers_order`, `kaito_enters_tapped_under_an_opponents_creatures_enter_tapped`, `spark_double_copying_kaito_gets_one_loyalty_counter_and_a_plus_one` |
| `aura-found-in-graveyard` | CR 400.7f and 603.6e, under which an Aura's trigger on its enchanted permanent leaving finds the card the Aura became "in its owner's graveyard", against CR 400.7e and 603.6c, which find a moved object in the first public zone it went to. They part on a board where a replacement exiles the Aura and not its creature: Angelic Destiny on an opponent's creature under that opponent's Leyline of the Void, the creature dying into its own graveyard and the Aura exiled | the CR's text: no official statement on the board was found (2026-10-15). The other reading, the owner's at #253's review, is that "graveyard" names the default path CR 704.5m sends an Aura down rather than an exclusion; it is the switch's other position, and the one a custom card's "exile it instead" would want | the destination CR 400.7f asks for, in its two places: `to == Zone::Graveyard` in `perform_zone_change`'s CR 704.5m hook, and the `to: Zone::Graveyard` pattern in `aura_put_into_graveyard_beside_its_host`. "The first public zone it went to" is the flip | `triggers-architecture.md` §4.3, "An Aura's new object" | `angelic_destiny_exiled_instead_of_its_state_based_move_is_not_returned`, `angelic_destiny_exiled_beside_its_creature_is_not_returned` |
| `copy-exception-conditions` | CR 707.9f checks a conditional copy exception "taking into account any other exceptions that effect includes", and is silent on two conditional exceptions whose effects decide each other's conditions: Spark Double's two counters, on a planeswalker whose type hangs on its loyalty counters, where a look-ahead that sees an entry's counters (`lookahead-entry-counters`) makes each exception's counter matter to the other | the words (the third rule; no official statement addresses it): each conditional exception is checked against the copy without it and with every other exception that applies there, a conditional one checked the same way without both. The only reading that passes 707.9f's own test on that board | whether 707.9f's frame includes the other conditional exceptions that apply, or only the unconditional ones: one line in the copy applier | `copy-effects-architecture.md` §4.1a, "The Kaito board" | `spark_double_copying_kaito_gets_one_loyalty_counter_and_a_plus_one` |

## Measuring

### The close-out

Each pool answers a question the other cannot, so a PR runs both:
**performance** asks *did my change make the engine slower?*, read as a delta
between two builds in one run; **stress** asks *is there a card shape that makes
the engine fall over?*, read against a threshold (0 errors, 0 panics, 0
turn-limit hits). **Never A/B any number across a pool change.**

While building: **predictions** in the PR body before any arm runs, one line per
arm; **the band** tracked at every commit (`git diff --numstat <merge-base> HEAD
-- mtgsim/src mtgsim/tests`); **docs in the commit that changes what they
describe**. After the last code commit, each once:

1. `cargo build --all-targets` (zero warnings), `cargo clippy --all-targets --
   -D warnings` and `cargo test`, never again after a docs-only commit; then the
   check scripts on `CLAUDE.md`'s line.
2. `python plans/close_out.py --arm main=origin/main --arm engine=<sha>
   [--arm shipped=HEAD] [--require "<newly pooled cards>"]`, in the background
   while the docs are written: the audited counters on both pools at two seats
   and four, the budget, the reachability read, the record block's table, and
   the copies the branch made. The model pass over the changed files runs beside it.
3. **Game-by-game dump attribution only when a gameplay row moves that the
   predictions did not**; a moved cost row is a reading.
4. **The big audited smoke only when the PR touches the dispatcher or the
   audit**: `fuzz_games --audit`, 1,000 games at seed 777, both pools and seats.
5. The close-out writes the phase's stub and archive eviction, a `fuzz-record.md`
   block of about 50 lines (the table and only the moves nobody predicted), the
   roadmap row if the route changed, and the PR body, each copy the branch made
   merged or kept with its reason.

Not run: separate determinism runs (CI's step), status prose in the roadmap, a
timing sitting (the budget reads instructions).

### The budget

- **A phase's engine arm may cost at most 2.5 points of callgrind instructions
  per decision, at identical counters, against `main`** on `close_out.py`'s
  board (Commander scale, 20 games, the hasher's seed pinned), or the PR says
  why in its `fuzz-record.md` block and the reviewer decides. A decision is a
  prompt with two or more legal answers.
- **It is the per-PR half of a ratchet**; the other half is each spine close's
  readiness reading of decisions per core-second, never worse without a reason.
- **Instructions, not time**: a timing sitting spreads 2–6%, wider than the
  budget, while instructions repeat to ~0.01%. Stalls are read at readiness.
- Adopted 2026-09-15, in instructions since 2026-09-25; the calibration is
  `fuzz-record.md`, "Calibrated 2026-09-25".

### The floors

The ratchet stops the engine getting worse; the floors say where it stops being
fit for v1's AI use case at all (derived in `plans/references/ai-performance-floors.md`).
`check_state_of_play.py` joins the limits below with the latest readings in
`fuzz-record.md#the-latest-floor-readings` and draws both on the board.

<!-- floors: begin -->
| Floor | Limit | Definition | Instrument |
|---|---|---|---|
| 1. Decisions per loaded physical core-second | ≥ 10000 | at Commander scale (`--deck-size 100 --life 40 --players 4`) under the default bot stack; a decision counted where it reaches the agent; loaded = the one-thread rate × (16-thread speed-up ÷ physical cores) | the readiness pass, counted by a provider innermost in the stack, with callgrind instructions per decision beside it |
| 2. Allocations per clone | ≤ 64 | CI's proxy for a full-state clone within 10 µs, at item 143's checkpoints, at every decision and on the fixture boards | `clone_bound_test` and `clone_fixture_test` in CI's release step, with `fork_copies_test` beside them |
| 2. Microseconds per clone | ≤ 10 | a full-state clone, portably ≤ 1/6 of one decision's engine CPU, at the same readings | the readiness pass |
| 3. Kilobytes per clone | ≤ 128 | deep clone size, bounded by the board's high-water mark and never by the turn count | the same tests, exact bytes with the hash seed pinned |
<!-- floors: end -->

- **Floor 1 is read under a named stack** (`backlog.md` §2.22's default:
  `ManaWindowStop` on, the tap-solver decorator off), as it is read on a named
  board. A reading under another stack is another operating point, never a
  regression, and is not compared across a stack change.
- **A reading is replaced in place when it is taken again**; every reading's
  dated record stays in `fuzz-record.md`.

### Reading the tail

- **A mean is the one statistic guaranteed to hide a cliff**: read p50 / p99 /
  max, and `CPU/turn` beside `CPU/game`, since a long game is not a slow one.
  The slowest game's seed replays alone with `--seed N --games 1`.
- **A timing compares only within one sitting**, interleaved against a binary
  built from the other tree, never against a figure in a doc; a suspicious
  cross-sitting delta gets an interleaved A/B before a diagnosis.
- **A turn tail that climbs while the median holds is the signal**; a game tail
  climbing with it is probably a longer game. Run 200 games when the tail is
  read: below 100, p99 is the max.
- The sitting that showed it: `archive/engineering-practices-records.md#32-reading-the-tail-and-why-the-mean-cannot-do-this-job`.

### The cost rows

- **The bold rows of a record block are engine cost, and they are fixtures**:
  a pure function of the seed and the pool, so a change means the cost model
  moved, as a change to "creatures died" means behavior did. How to read each:
  `fuzz-record.md#how-to-read-the-cost-rows`.
- **So a count is part of determinism**: an `any` over a hash map with a layer
  query inside is a violation, even when its answer cannot vary.

### Determinism

- **Everything outside `=== Timing ===` is byte-identical at one seed, any
  `--threads` and any `MTGSIM_HASH_SEED`.** CI checks it on every push, and
  `prof_arm.sh` checks its native run against valgrind's. `fuzz_ab.py` stays the
  tool for timing rounds and `--copies` on a heavy board.

## The spec database

- **Annotate at write time**, above `#[test]`: `// COVERS:` when the test builds
  the atom's whole scenario, `// COVERS-PARTIAL:` otherwise.
- **Never claim an atom a test doesn't prove**; a false link is worse than a
  blank. `suspicious` is a smell test, and silence proves nothing.
- **A phase does not close until `owed` is clean for it**, every atom covered or
  deferred with a reason; adding the phase to `SHIPPED_PHASES` arms the gate.
  CI does not run `specdb` (the owner, 2026-08-31).
- **Triage what `owed` reports as a fact or a feature.** A fact (object
  identity, who cast it, characteristics an instant ago) is unrecoverable if not
  captured when it exists, so record it on the first customer; a feature is a
  normal diff, deferred freely. Count cards to decide when to build a feature,
  never whether to record a fact.
- **A defect with no atom is invisible to both `owed` and Deferred Migrations**;
  that is the review's to find.
- The command reference is `specdb.py`'s docstring; why it is a gate:
  `archive/engineering-practices-records.md#5-the-spec-database-as-a-gate`.

## Module layout

- **A `mod.rs` declares and re-exports; it does not define.** Module docs, `mod`
  declarations, `use`, attributes, and `macro_rules!` a module's children share
  (metacode, in scope only after it is defined). `lib.rs` and `main.rs` are
  exempt. `check_module_layout.py` fails the rest.
- **The implementation goes in a sibling named for what it does**:
  `engine/replacement/`'s `gather.rs`, `pipeline.rs`, `instance.rs`. A one-file
  module is fine; that file is not called `mod.rs`.

## Trace pages

- **Write one at the close of a phase that changes *how* a read is answered**,
  not what the answer is: not for a card, an enum arm or a pool entry. Its
  examples are the phase's findings, and it may walk a design the phase rejected.
  The decision is recorded on the phase's heading.
- **Pinned, never maintained**: a page records the engine at the commit its
  `<head>` names, and a later change gets its own page.
- **Linked from the owning architecture doc's phase entry and the phase's
  `codebase-state.md` entry**, and listed in `trace-pages.md`.
- **The trace sink makes a page's spine, never its argument**: the boards, the
  questions and the "where the reads differ" table are authored.
- The format, the naming, the index and the trace sink: `trace-pages.md`. The
  engine map, one event through every subsystem, is `engine-map.md`, kept true
  by `check_engine_map.py`.

## The spine-phase audit

- **When a critical-path item closes, audit it before the next starts**, as
  passes with named instruments, each its own PR, planned in a `plans/handoffs/`
  file the last pass's PR deletes. A closing PR is the wrong place: the person
  closing believes the claims.
- **Findings land where the project records them**: a numbered
  `codebase-state.md` item under a dated "Found by" heading, a `backlog.md`
  entry, a stale claim rewritten in place, a rule here.
- **An audit ships docs and small fixes only**; anything larger is a scheduled
  item with an owner. A pass re-reads what it touches, and a number in its brief
  is a starting point.
- **A critical-path item longer than about eight PRs gets the hygiene pass at
  its midpoint**, with a shape review of the code the rest will extend.
- The passes and their instruments: `spine-phase-audit.md`.

## The dev GUI

- **Logic is plain Rust under tests, and egui only draws.** The bridge, the
  snapshot, the view model and the session are reviewed as the engine is; the
  drawing files lay out what the view model built and report clicks.
  `check_egui_only_draws.py` fails a window crate outside the drawing files, or
  the engine inside one. A new drawing file is a line in its list.
- **A drawing file is reviewed against the egui checklist**, which a review
  agent is given: `devgui-map.md#the-egui-checklist`.
- **Every GUI PR body carries** its files sorted by how to review them, a click
  script in Magic terms, and every picture it changed, old beside new.
- **The decision log is the engine's, not the window's**: the engine writes
  every answer and its own passes; a seat's mode is never in it.
- The parts and their tests, random clicks among them: `devgui-map.md`.

### What the engine says and what a client draws

- **A fact only the rules compute is the engine's**; **wording every reader sees
  alike is `ui::display`'s**; **how it looks is the client's.**
- **The review's question**: would a second display client compute the same
  thing from the same facts? Then it is the engine's (`ui::display::type_line_now`
  is the worked case).

### What one prompt costs the window

- **A GUI PR that touches the snapshot or the view model quotes `prompt_cost`'s
  before and after** (`cargo run --release --example prompt_cost`, in `devgui/`).
  A number to read beside a change, not a gate.
- **Per-frame work is built where its input changes unless the reading says it
  is too small to matter**, which is a measurement, not an argument.
- The readings so far: `archive/engineering-practices-records.md#104-what-one-prompt-costs-the-window`.

## Invariant gates

`check_invariants.py`, in CI and on `CLAUDE.md`'s line, reads three of
`CLAUDE.md`'s invariants:

- **printed**: `card_data.` and one of `types`, `subtypes`, `supertypes`,
  `colors`, `keyword_flags` or `abilities`, or the same through a local bound to
  a whole card; allowed in `oracle/` and `cards/`.
- **unordered**: an iteration, `drain` or `retain` over a hash-keyed field of
  `GameState`, read off its declaration, tests included.
- **write**: a write to a field whose change is an event, or a call to the
  store's writer of one, outside the file `PERFORMERS` names for it.

`#[cfg(test)]` items and `test_support.rs` are not read by printed and write.

### The tags

A site the rule does not mean carries a tag that says why, starting a comment
line in the block directly above its statement (covering the whole statement),
or trailing on its line.

| Tag | Check | Says |
|---|---|---|
| `PRE-LAYER ZONE:` | printed | asked before the layer system can answer for the object: legality from a hand, an entering permanent's lookahead (CR 614.12), `register_static_effects` |
| `AS PRINTED:` | printed | the card as printed, for a display, a statistic or the scenario format; no rule reads it |
| `NO ROW REACHES:` | printed | an object no continuous effect reaches, or with a leg that asks the effective list when one may |
| `ORDER-FREE:` | unordered | nothing observes the order: a sort by a total key next, a set, a count, an `any`, each entry written alone |
| `CAST-ROLLBACK:` | write | CR 601.2a's move onto the stack and its rewind, silent both ways and announced at 601.2i |
| `NOT REPLACEABLE:` | write | a write no replacement effect can watch for (CR 614.1): damage wearing off, removal from combat, the deathtouch flag |

- **`AUXILIARY-MOVE:` is not read by the gate**: it marks a batch, which is proposed.
- **A count is not order-free if its closure walks the layers**, since a walk
  writes a trace record.
- **On a hot path, order is paid for only while tracing**: sort under
  `trace_on()` alone, in one block (the owner, #247).

### Performers and known violations

- **A new performer is a row in `PERFORMERS`**, with its reason, read in the diff.
- **`KNOWN` lists violations an item owns, by item number; an entry matching
  nothing fails**, so the PR that closes the item takes its entry off.

### What the gate does not see

A `&CardData` handed to a function; iteration through a local bound to a map,
or a write through a `&mut` a helper returned; the `std` hash maps off
`GameState` (CI's three-process determinism step catches a leak there);
`devgui/`; a swallowed `Result` and registry membership. Review owns them.
Making event fields private is `names-and-hygiene-sweeps`' question.

### Randomness

- **`clippy.toml` refuses every way rand offers to draw from the thread's RNG or
  the OS**, for both crates; the opt-outs `CLAUDE.md` names carry
  `#[expect(clippy::disallowed_methods, reason = …)]`, never `allow`.

### The checked sitting

- **`[profile.release-checked]`, release with debug assertions and overflow
  checks, plays 50 stress games and 50 of the `fixtures` pool in CI's
  `fuzz-checked` job**, so the audits a release build compiles out are played.
- **A test asserting a `debug_assert!` fires is `#[cfg(debug_assertions)]`**;
  none needs to be loud in release, since the checked sitting plays every
  registered card with them on.
