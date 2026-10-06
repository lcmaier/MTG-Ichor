# setup-architecture.md — landed phases, evicted

**A record of finished work, not a plan.** Every section here sat under a ✅
heading in `plans/setup-architecture.md` and was moved out when its phase
shipped, leaving the heading, a stub and a pointer in the live doc. Nothing
here is owed and nothing here should be acted on; what each section is *for*
is the reasoning — the build as sized, what the building changed, the
measurement. `check_state_of_play.py` reads the ✅ headings that stay, and
fails when a landed section keeps more than 40 lines in the live doc
(`engineering-practices.md` §4). Later phases are appended by the PR that
lands them.

#### The editor's advanced settings, first part — ✅ landed 2026-10-06

*Evicted 2026-10-06 from `plans/setup-architecture.md` §8, where the heading and a stub remain.*

### The build as sized (2026-10-03, at SU-3's design; split 2026-10-06, at the brief)

§7b.2's decision 2 sized the ten rows its A leaves as text at ~600–900
lines, code and tests together, row by row (§8's paragraph): the typeable
field ~40–60, player counters ~45–65, lands played ~20–30, left the game
~20–30, commander damage ~50–80, history ~110–160, `this turn:` ~90–130,
`counters:` lines ~60–90, setup actions ~210–310. SU-3's editor had run 1.5–2.1
times its sizing, so the brief split the settings into up to three PRs. The
first is the switch, the field, and the four player rows that
`four-seats-commander.scenario` and the template use, ~175–265.

**The two decisions the brief left open**, each with a recommendation the
build took: one Advanced toggle in the editor's header, off by default, each
control in place of the text it replaces; and a refused typed line leaving
the board as it was, the field keeping the line with the refusal under it,
in words (`engineering-practices.md` §10.1, question 6). The second was
sharpened at the build (below, 1).

### Sized against built

Lines added, `devgui/` with a `src` file's `#[cfg(test)]` module and
`examples/` counted as tests, read off the PR's last code commit
(`3e8b67b`). The split between the two parts is by hand, to the nearest five.

| Part | Where | Sized, code and tests | Code, built | Tests, built |
|---|---|---:|---:|---:|
| The switch and the typed field, kept across opens | `editor.rs`, `app.rs`, `session.rs` | 40–60 | ~100 | ~85: two unit tests, the random clicks' switch and field, the session's assertion, the picture, `prompt_cost` |
| The four player rows, with the write rule and the reading they share | `editor.rs`, `app.rs` | 135–205 | ~140 | ~125: four unit tests and their helpers, the random clicks' rows and reach check, the typed line past what a count holds |
| A counter count kept at the most a count holds (below, 7) | `mtgsim/src/state/player.rs`, `battlefield.rs` | — | 6 | 19 |
| **The first part** | | **175–265** | **244** | **230** |

Code and tests came to 474 lines, 1.8–2.7 times the sizing; the dev GUI's
code alone, 238, is 0.9–1.4 times it. What the row-by-row sizing left out:
the field's own state (the line, its refusal, Add live only for a line not
yet refused, and Enter keeping the focus); the switch kept across opens; the
reading each count shows, which differs by row; one write rule for every
player word, which replaced life's own; which words a control shows, an
exhaustive match; the refusal mark on a player's name; the engine's fix; and
the tests, which ran about twice the code's share. The band stayed far under
2,500 at every commit: 367, 420, 433 and 444, then 469 and 474 with the fix,
the docs beside.

### What the build changed in the design

1. **Only the parser refuses a typed line.** The brief's recommendation
   said "the loader's message"; the build reads that as the parser's. A
   line the loader refuses goes in, as any edit the parser reads does, and
   the loader's refusal shows under the board and marks it. The editor
   judges no line itself (§7b.1), and boards are built through refused
   states: a commander's damage typed before the card is made a commander.
   A line that says nothing new, a comment or a stated default, stays in the
   field too, since it would change nothing. A game's fact the board
   already states is the parser's "stated twice", and its control sits
   above the field.
2. **Each count is the loader's reading of the words**, not the first word.
   The loader sets lands played and commander damage, so the last word
   stands, and adds counters (`PlayerState::add_counters`), so two
   `poison 2` words count 4. A click leaves one word at the first one's
   place, or none at zero, which is the default.
3. **A refusal of a word a control shows marks the player's name**, since
   no text shows the word. A refused life word, which nothing marked
   before, marks it too.
4. **The counter kinds a player has** are the two `CounterType` groups as a
   player's, poison and energy (CR 122.1). Any other kind a player line
   states stays text, as does commander damage naming no one commander.
5. **Commander damage is counted for each commander on the board**, each
   named by its card line, so a tag the commander takes later (§7b.1's
   rule) keeps the count with its commander.
6. **The typed line goes last in the board's text**, so it renumbers nothing
   already there, and the board reads it back as it reads any edit.
7. **A counter count stays at the most a count holds** (the owner, at
   #227). A player's counter words add, so two could sum past `u32::MAX`
   and panic the loader in a debug build ("attempt to add with overflow"),
   and the field reached that by typing, a panic on the window's thread.
   `PlayerState::add_counters` and `PermanentState::add_counters` saturate
   now, the permanent's reachable only in play, after a board set its count
   near the most. Each test failed first on the tree before the fix.
   `close_out.py`: every row `IDENTICAL`, instructions per decision +0.01%
   (`fuzz-record.md`).

#### SU-8 — what happened, from the trace — ✅ landed 2026-10-06

*Evicted 2026-10-06 from `plans/setup-architecture.md` §8, where the heading and a stub remain.*

### The build as sized (2026-10-05, at #220; amended 2026-10-06)

**The engine, ~190–295.**
- the reader of the sink's lines, ~90–130;
- `is_prohibited`'s form that names the restriction, and the `pipeline`
  record's field, ~25–45;
- the why's sections for an event and for an object's triggers, ~70–110;
- item 210's fix, ~5–10;
- **every question kind's line, ~80–140** (`codebase-state.md` item 212's
  census, the owner, 2026-10-05). `ui::why::refusals` matches every
  `ChoiceKind` without a wildcard. Four kinds already answer: the priority
  question and the two declarations (SU-7), and CR 601.2g's mana window, which
  MA-1 built (`mana-architecture.md` §3.13; amended 2026-10-06, since this
  line had it landing with item 162's build). Each kind says what it ranges
  over, so a why asked about anything else says the question does not range
  over it. Two kinds are answered from the trace: `ChooseReplacementEffect`'s
  candidates, from the `pipeline` record, and `OrderTriggers`', from the
  `trigger` records. Three get a reason from their own filter:
  `ChooseEnteringController` (CR 800.4a), `ChooseAuxiliaryZoneChange` (CR
  614.13a and 101.2) and `ChooseCopySource` (the copy effect's filter). The
  other filtered kinds' reasons land with their owners: `SelectRecipients`
  with RS-2; `ChooseXValue` and `GenericManaAllocation` with MA-2
  (`mana-architecture.md` §3.13); `ChooseSacrificeForCost` with
  `cost-architecture.md` CP-2. The rest say only what they range over.
  `plans/references/cast-census.md` §9 has the table, read off the enum and
  asserted against it. `refusals`' doc comment still cites item 212, now
  closed, and this PR rewrites it.

**The dev GUI, ~110–180.**
- the replay thread for a question about then, with its stop and its
  supersession, ~70–110;
- each log line's event number in the snapshot, and the right-click on it,
  ~30–50;
- the sections drawn, ~10–20.

**Tests, ~150–230.**
- the reader's round trip against every record kind;
- an event's batch read back on `bolt-into-giant-growth.scenario`;
- a "can't" named: an indestructible creature with lethal damage;
- a trigger refused by its intervening "if";
- in the headless tests, the replay stopping at the open question;
- every `ChoiceKind` answers a why without panicking, and a departed
  player is named as the reason at `ChooseEnteringController`, ~30–50.

**A/B.** No record is written in a fuzz game unless `--trace` asks for it.
Predicted `IDENTICAL` on every counter, and instructions within ±0.3%. With
`--trace`, the `pipeline` record's new field is the only change in its text.

**In all:** ~1,050–1,615 lines of code and ~500–750 of tests, as sized at
#220. SU-6 landed at 1,062 and 622, and SU-7 at 1,042 and 606. SU-8, as
sized above, is ~380–615 more and ~180–280 of tests; item 212's census added
the question kinds' lines (~80–140 and ~30–50).

### Sized against built

Lines added, a `src` file's `#[cfg(test)]` module and `devgui/examples/`
counted as tests, read off the PR's last code commit (`8bfeb01`): each file's
additions in the whole diff, split by part where a file holds two.

| Part | Where | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---|---:|---:|---:|---:|
| The reader of the sink's lines, and `RecordKind` | `state/trace.rs`, `engine/trace_records.rs` | 90–130 | 368 | | 47, the round trip |
| `prohibition`, and the `pipeline` record's field | `restriction/predicate.rs`, `restriction/mod.rs`, `replacement/pipeline.rs` | 25–45 | about 87 | | |
| The why's sections for an event and the triggers asked about it | `ui/what_happened.rs`, `ui/why.rs`, `types/ids.rs`, `ui/mod.rs` | 70–110 | about 323 | | |
| Every question kind's line, and the three filters made shared checks | `ui/why.rs`, `replacement/pipeline.rs`, `replacement/mod.rs` | 80–140 | about 284 | | |
| Item 210's fix | `replacement/pipeline.rs` | 5–10 | 7 | | |
| A replay's stop handed to it | `ui/replay.rs` | | 6 | | |
| The replay thread, its stop and its supersession | `devgui/src/why_replay.rs`, `session.rs` | 70–110 | 212 | | |
| Each log line's event number, the right-click, and who answers which why | `snapshot.rs`, `prompt.rs`, `bridge.rs`, `view_model.rs`, `lib.rs` | 30–50 | 174 | | 54, `view_model.rs`' tests |
| The sections drawn | `app.rs` | 10–20 | 7 | | |
| The engine's tests | `phase_su8_integration_test.rs`, `test_support.rs`, SU-7's updated | | | 150–230, with the dev GUI's | 400 |
| The dev GUI's tests: the replay's headless tests, random clicks, the pictures | `devgui/tests/`, `devgui/examples/` | | | | 150 |
| **SU-8, the whole diff** | | **380–615** | **1,468** | **180–280** | **651** |

Code ran at 2.4–3.9 times its sizing, past SU-1 to SU-7's 1.0–3.4, and tests
at 2.3–3.6. The engine ran over most, 1,075 against 190–295, and the dev GUI
393 against 110–180. What the sizing left out:
- **The reader is a parser of its own.** No `serde` (main item 141), so it reads
  the writer's JSON itself, with a value type, typed reads and a kind list the
  writer shares, where the sizing counted a reader of fields.
- **Every record a batch writes is worded**: its proposals, each iteration's
  candidates, "can't"s, choice and results, a batch after it that performed
  nothing, and each trigger's verdict with where it went. And 27 range lines,
  three filters made checks the enumeration shares, and two trace readers.
- **Who answers which why**, in the dev GUI: the seat or a replay, a request
  number so a newer request supersedes an older one, the panel's waiting
  state, and a finished game answering through its whole line.

The band stayed under 2,500 at every commit: 2,119 in all.

### What the build changed in the design

1. **A seat stands behind the replay's line** (§7c.1's decision 3, amended
   where it stands). The why needs the open question's context and options,
   which only the asking seat holds, so the seat behind the line reads it
   there and stops the run with `Stop::LogSpent`. `Replay::with_control` takes
   the stop the window holds before the replay exists.
2. **A why at CR 616.1's choice or CR 603.3b's order is a replay's too**
   (decision 3, amended). Their reasons are in the trace, and the seat follows
   the object there without answering, so each why has one answerer. At CR
   616.1's choice the trace holds the batch's earlier iterations only, since
   the open one is written at its exit: a why says what applied already (CR
   614.5), and the range line says the rest.
3. **A batch decided after an event that performed nothing is told with it**
   (decision 4, amended). A destruction a "can't" stopped has no event of its
   own, so "why didn't it die" is the damage's why.
4. **`prohibition` names which restriction**, a keyword, a static ability's
   text or an effect a resolution registered (decision 2's item 4, amended).
5. **The window marks the request it waits on** (§7c.2's bridge, amended):
   `ToWindow::WhyFromTrace` carries the request's number, and an answer to any
   other request is dropped.
6. **A game that has ended answers through its whole line**, its right-clicks
   and links live, as §7c.2 said SU-8 would.
7. **A range line names the player asked**: "the lands Player 0 can play".
8. **`ChooseCopySource`'s reason is the effect's own words.** Its filter is a
   `SelectionFilter` the question does not carry, and `validate_selection`
   answers in engine text; both are RS-2's to type.
9. **The log is a shade brighter.** egui draws a label that senses a click in
   its interactive color.
10. **Found:** the review pictures had not been redrawn since MA-1, whose
    exact check changed the games they show, and the screenshot test failed on
    `main` (`codebase-state.md` item 216).

#### SU-7 — why an option is not offered — ✅ landed 2026-10-05

*Evicted 2026-10-05 from `plans/setup-architecture.md` §8, where the heading and a stub remain, with SU-8's sizing beside it.*

### The build as sized (2026-10-05, at #220)

**The engine, ~290–450.**
- the reasons, typed and worded, ~80–120;
- the enumeration's checks as functions that return them (`can_cast`,
  `can_play_land`, `can_activate` and `can_attack`), ~120–180, much of it
  moved rather than written;
- `check_cast_legality` on the same reasons, ~20–40;
- the why's section for the open question, ~60–90, and a player as what a
  why can be about, ~10–20 (moved from SU-6, §7c.1's decision 4).

**The dev GUI, ~20–35**: the section's heading, and the right-click on a
player's line.

**Tests, ~150–220.** One per family of reasons, each reached on a board, and
each shown to fail first where it moves a check.

**A/B.** The priority question's candidates are built by the same checks, in
the same order. Predicted `IDENTICAL` on every counter, and instructions within
±0.3%.

### Sized against built

Lines added, a `src` file's `#[cfg(test)]` module and `devgui/examples/`
counted as tests, read off the PR's last code commit (`4cabaab`): each
file's additions in the whole diff, split by part where a file holds two.

| Part | Where | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---|---:|---:|---:|---:|
| The reasons, typed, with the engine's own words | `mana_helpers.rs`, `legality.rs`, `costs.rs`, `put_on_stack.rs` | 80–120, with the words | about 180 | | |
| The words a client shows, each with its rule | `ui/display.rs` | (above) | 196 | | |
| The checks, and the enumerations keeping their `Ok`s | `mana_helpers.rs`, `legality.rs` | 120–180 | about 240 | | 6, `legality.rs`' tests |
| The enforcement on the same checks | `put_on_stack.rs`, `zones.rs`, `validation.rs`, `costs.rs`, `priority.rs` | 20–40 | about 50 | | 6 |
| The question's section, and a player as what a why is about | `ui/why.rs` | 70–110 | 322 | | 439, `phase_su7_integration_test.rs` and SU-6's updated |
| The right-click on a player's line, the seat's question | `devgui/src/` | 20–35 | 56 | | 12 |
| The panel's tests, random clicks on players, two pictures, `prompt_cost` | `devgui/tests/`, `devgui/examples/` | | | | 121 |
| Other tests: `play_land` loses its zone | `mtgsim/tests/` | | | | 22 |
| **SU-7, the whole diff** | | **310–485** | **1,042** | **150–220** | **606** |

Code ran at 2.1–3.4 times its sizing, past the 1.0–2.5 SU-1 to SU-6 ran at,
and tests at 2.8–4.0. The engine ran over and the dev GUI did not much: 986
against 290–450, and 56 against 20–35. Three things the sizing left out:
- **Two more enforcement points and a cost check.** `play_land` and
  `activate_ability` ask the shared checks too (item 188 names `play_land`
  beside `check_cast_legality`), and `can_pay_costs` returns `CannotPay`, so a
  {T} on a tapped permanent says so.
- **Each reason worded twice, on purpose.** The engine's `Display` keeps the
  old error text, which the trace's `priority_rejected` record carries;
  `ui::display` words it for a person, with the rule apart.
- **The section answers at every question.** Whether the question offers the
  object or the player is matched over every kind of option with no wildcard,
  and the declare-blockers question answers both ways, for a blocker and for
  an attacker, with a reason about the blocker alone said once.

The band stayed under 2,500 at every commit: 1,648 in all.

### What the build changed in the design

1. **`play_land` and `activate_ability` share the checks too** (decision 2's
   item 2, amended where it stands), and `play_land` loses its `from` zone:
   the check reads the hand, as CR 305.1 does, and playing from elsewhere is
   RS-2's permission (`cant-effects-architecture.md` §4.3).
2. **The cast refuses a land (CR 305.9).** Only the enumeration did, so a
   `CastSpell` naming a land went through `cast_spell` as a spell; a probe on
   the tree before `7463c38` cast a Forest and left it on the stack.
3. **`can_cast` returns the mana sources it would tap**, which
   `castable_spells` returns (§7c.2's sketch, amended).
4. **A target's reasons are RS-2's** (the owner, 2026-10-05). At every
   question but the priority question and the two declarations, the section
   says only whether the thing is offered.
5. **No "can't" arm yet.** No restriction on a choice exists until RS-2 and
   RS-3a build them; each family's check is one site for them now, and the
   RS rows say so.
6. **The click script edits Everywhere to a Mountain** (§7c.2, amended). One
   Everywhere counts as five mana sources, so on `main.scenario` the Bears are
   offered: `codebase-state.md` item 162, whose design doc the owner made the
   PR after SU-7. A Forest would leave Player 0 nothing to cast, so the
   window would not stop at the priority question.
7. **The A/B in two steps** (the owner, 2026-10-05). Merging the attack check
   dropped a second `is_creature` the old bool asked, which moves `Memo hits`.
   The merge kept it, the last code commit (`4cabaab`) drops it, and the
   close-out's third arm sits between them.
8. **Every moved check was broken once, and a test caught it.** 29 checks:
   two caught only by the new tests (the cast refusing a land, the controller
   check on activation), and the life check by an older test outside the SU-7
   files (`pre_phase3_integration_test.rs`).
9. **Found:** the priority question never offers a mana ability, which CR
   605.3a allows (`codebase-state.md` item 211, slotted with item 162's
   design).

#### SU-6 — the why panel, and what the layers did — ✅ landed 2026-10-05

*Evicted 2026-10-05 from `plans/setup-architecture.md` §8, where the heading and a stub remain, with SU-7's and SU-8's sizing beside it.*

### The build as sized (2026-10-05, at #220)

**The engine, ~280–420.**
- `layers::explain`: the recorder in `run_pass`, `perform` and
  `compute_non_member`, ~110–160;
- its types and its entry, ~50–80;
- `ui::why`'s value, and the words of the layer section: a frame's changes
  field by field, destructured with no `..` so that a new field must answer,
  ~120–180.

**The dev GUI, ~180–270.**
- `Reply::Why` and the seat following its object, ~40–60;
- the view model's panel, with Back, ~80–120;
- the drawing, ~50–80, and the right-click on each item, ~10.

**Tests, ~200–300.**
- the explanation on these boards: Humility and Opalescence in both orders;
  Blood Moon and Urborg's dependency; an anthem that does not apply; a CR
  613.6 lock; a counter; a CDA; and a card in a library that a row reaches;
- its equality with the memo's frame;
- a game with every why asked, against the same game with none;
- the bridge's round trip, the view model, random clicks with right-clicks,
  and a review picture.

**A/B.** `run_pass` takes a recorder, which every game path passes as `None`.
Predicted `IDENTICAL` on every counter, and instructions within ±0.3%.
`prompt_cost` reads a why on the large board's busiest object
(`engineering-practices.md` §10.4).

### Sized against built

Lines added, a `src` file's `#[cfg(test)]` module and `devgui/examples/`
counted as tests, read off the PR's last code commit (`92a3334`): each
file's additions in the whole diff.

| Part | Where | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---|---:|---:|---:|---:|
| The recorder in the pass, `perform` and the non-member walk | `board.rs`, `compute.rs` | 110–160 | 268 | | 19, the layer tests' hook |
| The explanation's types and its entry | `explain.rs`, `layers/mod.rs` | 50–80 | 191 | | |
| The why's value, and the layer section's words | `ui/why.rs`, `ui/mod.rs` | 120–180 | 361 | | 299, `phase_su6_integration_test.rs` |
| `Reply::Why`, and the seat following its object | `prompt.rs`, `bridge.rs`, `session.rs` | 40–60 | 68 | | |
| The view model's panel, with Back, and the hover's hint | `view_model.rs` | 80–120 | 123 | | 57 |
| The drawing, and the right-click on each item | `app.rs` | 60–90 | 51 | | |
| The panel's tests, random clicks with right-clicks, the picture, `prompt_cost` | `devgui/tests/`, `devgui/examples/` | | | | 247 |
| **SU-6, the whole diff** | | **460–690** | **1,062** | **200–300** | **622** |

CI's test step adds six lines. Code ran at 1.5–2.3 times its sizing, inside
the 1.0–2.5 SU-1 to SU-5 ran at, and tests at 2.1–3.1. The engine ran over
and the dev GUI did not: 820 against 280–420, and 242 against 180–270. The
sizing left out most of what an explanation has to say. `ui::why` words
every characteristic a frame has, each a field's change, where the sizing
counted the change list alone; it summarizes a frame for the printed card
and the result, and names a layer with its rule. `explain.rs` carries six
kinds of application and four ways to miss, each with its doc, and an entry
with a route for each of the four pass memberships. The pass gained
`OwnApplication`, which names a member's own applications at the four
sites that push one, and `could_name` and `missed`, which decide before an
application applies what it would do to the object. The band stayed under
2,500 at every commit: 1,684 in all.

### What the build changed in the design

1. **A step keeps every object it affected** (`LayerStep::affected`), which
   §7c.2's sketch did not have. The panel names them under a miss ("It
   applied to"), and LI-2's one-layer test hook reads them:
   `compute_board_traced` and its `TraceStep` became
   `compute_board_recorded` with a `Recorder`, so there is one record of a
   layer's order, not two.
2. **`OwnApplication` names which of a member's own applications applied**: a CDA,
   a keyword counter, P/T counters, the copy it entered as, or what it
   entered with. It held an optional CDA id, and an explanation names each.
   CR 604.2's check moved onto it (`OwnApplication::stripped`). It was `Own`
   until the review, which found the name said nothing at its uses.
3. **CR 306.5b's loyalty ability is a step of its own**
   (`AppliedBy::IntrinsicLoyalty`). It is added at the end of layer 4, outside
   any application, so the recorder records it there, in the pass and in the
   non-member walk alike.
4. **The recorder's checks move nothing on the game's path.** The first
   close-out read +0.55%. Per function, a tuple match built `(recorder,
   before)`, which moved an `Option` of a frame, about two hundred bytes, on
   every walk the game runs, where it is `None`. Let-chains test each part
   in place (`0195717`), and the reading fell to +0.50%; the rest is placed
   code (`fuzz-record.md`, the SU-6 block).
5. **A change reads "from … to …"**, not "… → …": the window's font has no
   arrow, and drew a box.
6. **`ToWindow::Prompt`'s why is boxed**, for clippy's `large_enum_variant`:
   the variant was 457 bytes against `Finished`'s 240.
7. **The memo's audit is not paused in debug** (decision 3, amended where it
   stands): a why at a question costs 1.9 ms there, audit and all, beside the
   snapshot's 33 ms.
8. **The panel is kept across Undo answer, the savestates and an unchanged
   board's Reload.** The rebuilt game's seat is told the object
   (`Play::watching`) and answers about it at its first question, since
   the same start numbers its objects alike. Any other start closes it, a
   load among them: a board edited since, or a dealt game's new seed, can
   give the id another card. As first built, Play after swapping Humility
   and Opalescence in the editor followed #22 onto Opalescence (`92a3334`,
   with a test that fails on the tree before it).
9. **A player's line moved to SU-7** (decision 4, amended): the layers say
   nothing about a player, and SU-7's section is the first that does.
10. **With no question open, an object's hover says a right-click asks
    nothing** (`5d900c1`), as decision 3's "a line saying why" asked; the
    open panel already said so.
11. **The layers doc's line went to §9**, beside LI-2's loop, not §13b, LI's
    landed plan (decision 5, amended).

#### SU-5 — the tools — ✅ landed 2026-10-04

*Evicted 2026-10-04 from `plans/setup-architecture.md` §8, where the heading and a stub remain, with the split into SU-4 and SU-5 that was sized beside it.*

### The build as sized (2026-10-03, at #216)

**The tools (§7.1–§7.3), sized 2026-10-02 with doc comments and messages
counted, revised over the review rounds of 2026-10-03, and re-sized at SU-4's
design after SU-3 (2026-10-03).** The replay, undo, the save and savestates,
whose consumer is the window. The exported test they were split from is
dropped (§7.2, decision 5).

**The order** (the owner, 2026-10-03): SU-3, then the tools. Nothing in the
tools needs the editor, and nothing in the editor needs the tools, and the
editor first makes the boards the tools are tried on quicker to build, a
four-seat Commander board above all. The seats change (§7, the window playing
every seat) came before both, as a PR of its own, the seats PR, with four
seats in it (§7b's decision 4, the owner, 2026-10-03).

**Kept true after SU-3.** §7.1–§7.3 named the dev GUI as it stood at their
design. SU-3 and the seats PR moved some of it, which §7b.3 lists as built,
and §7.1's "What the tree has" was read again against `fb1767a` before this
re-size. The decisions themselves (the typed unwind, undo to the window's last
prompt, no agent, the save a journal) rest on the decision boundary and the
engine's log, which SU-3 did not touch.

**What the re-size found**, against the first sizing's one PR of 1,085–1,585
of code and 590–880 of tests:
- eight readers of `Debug` text to replace, not three (§7.1);
- the header and a dealt start's `GameConfig` (decision 6), ~40–60 more in the
  text;
- the answer's fit checked through `ui::ask`'s own predicates, which assert
  today, so the replay and the validators share one check and keep no second
  copy (~30–50 in `ui::ask`);
- the stop's catcher over six entries, and its two consumers outside the
  engine, `fuzz_games`' scenario path and the dev GUI's refused setup line,
  ~25–35;
- item 200 (§7.1), ~30–50, and the tools' controls beside the editor's switch,
  ~10–20, in the window;
- the dev GUI's log written in the engine's text, which the first sizing
  counted in the bridge's row, moved to the engine's half with the refused
  setup line's display (~35–50), so the text has its writer in the PR that
  makes it and the window shows a stopped setup line as it did a panic;
- answers recorded by what was chosen (decision 6's D, the owner at #216's
  review): an option's identity written and compared by one matcher, which
  the replay, the setup driver and `ById` use, and a forced line skipped or
  answered where one build asks it and the other does not, and the scripted
  provider's answer by identity for new tests, ~260–410 with tests. The 299
  scripted answers already written move onto it in a PR of their own beside
  C0, the test cleanup before Phase 8's breadth (`codebase-state.md` item
  209, the owner at #216's review).

The last three phases ran 1.5–2.1 times their sizing on code (SU-1 ~1.9,
SU-2 ~2.0, SU-3 1.5–2.1; the archive's tables), each from what its sizing left
out, so each total below carries that range beside the count.

**Decided: two PRs, split along the engine's line** (the owner, 2026-10-03,
at #216's review). The owner reads engine changes closely and the dev GUI
loosely, so the engine's half goes up as a PR of its own, ahead of the
window's.

| SU-5, the tools: the window's half | Where | Code | Tests |
|---|---|---|---|
| The save: its journal (the engine's lines, the window's prompts, savestates, moves back), the tree, undo's place, the savestates and the line last left, written and read | devgui `save.rs` | 250–350 | 150–220 |
| The bridge: a start from a save, the save's writer on the engine thread, the window's prompts marked, the savestate reply, a superseded game's writers shut, a rebuild's replayed lines buffered and flushed at the hand-over | devgui `bridge.rs` | 150–220 | — |
| Session, launch and view model: Undo answer, Savestate, the menu, `--load`, the replaying count, a load playing on in a new pair | devgui `session.rs`, `launch.rs`, `view_model.rs` | 190–280 | 130–190 |
| Item 200: a scenario's own seed kept until its file is read; a log that cannot be opened refused in the window | devgui `launch.rs`, `session.rs`, `bridge.rs` | 30–50 | 20–30 |
| The drawing: the three controls beside Reload, the menu, the count | devgui `app.rs` | 50–80 | the pictures |
| **SU-5** | | **670–980** | **300–440** |

At the last phases' rate SU-5's code is 1,000–2,060, so 1,300–2,500 in all,
the dev GUI's, which may run over, reported per commit (the owner,
2026-10-03). Its tests are the rest of the proofs the tools owe: undo gives
the board a game played to that answer gives, through `Scenario::write`; a
savestate and a branch survive a save and a load; three presses of Undo
during a replay wait for one replay; item 200's two fixes. Each plays a short
board and is timed, as #214's and #215's were, and the dev GUI's CI step is
read against its ~24 s. The view model changes, so `prompt_cost` is read
before and after (`engineering-practices.md` §10.4).

| | **A. One PR, as first planned** | **B. Two: SU-4 the engine's, SU-5 the window's** | **C. Three: B with SU-5 split again, the savestates and their menu last** |
|---|---|---|---|
| Size, code and tests | 2,300–3,410; at the last phases' rate 3,070–5,890 | SU-4 1,330–1,990, SU-5 970–1,420; at that rate to 3,390 and 2,500 | SU-4 as B; SU-5 ~750–1,100 and a third ~220–320 |
| Review | the engine's ~1,300–2,000 lines in one PR with the dev GUI's ~1,000–1,400 | a PR that is the engine's, but for ~40 lines the window needs to keep working, read closely; then a dev GUI PR read by its click script and pictures | as B, and a third small window PR |
| What lands first | everything at once | the replay, with every dev GUI log readable by it, before any button | as B; undo and the save before the savestates |
| Risk | past the band, by a lot at the last phases' rate | the engine's text is designed before its second reader, the save's journal, exists: the journal's reader hands SU-4's its engine lines and reads its own (SU-4's ✅ section), and SU-5 may still find a gap to fix in the engine (`engineering-practices.md` §4: a split moves risk, it does not remove it) | as B |

**Decided: B.** Named for what each delivers: **SU-4, the replay** (the
engine's text, the replay and the stop) and **SU-5, the tools** (undo, the save
and savestates), the next code rather than a letter.

**A/B, predicted.** SU-5 is planned to touch no engine file, so its arms would
be one engine; if it does touch one, its A/B runs as SU-4's did
(`fuzz-record.md`, the SU-4 block).

### Sized against built

Lines added, `devgui/` with a `src` file's `#[cfg(test)]` module and
`examples/` counted as tests, read off the PR's last code commit
(`4139468`): each file's additions in the whole diff.

| Part | Where | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---|---:|---:|---:|---:|
| The save: the journal, the tree, undo's place, the savestates and the line left | `save.rs` | 250–350 | 331 | 150–220 | 141 |
| The bridge: the record and its writers, the hand-over, the replay and a divergence, a start from a setup | `bridge.rs` | 150–220 | 394 | — | 38 |
| Session, launch and view model: Undo answer, Savestate, the menu, `--load`, the count, the refusals | `session.rs`, `launch.rs`, `view_model.rs` | 190–280 | 364 | 130–190 | 31, and `tests/tools.rs`' 302 with its support's 50 |
| Item 200: the seed read at each start, the log named for it; a log that cannot be made refused | across the above, `boards.rs` | 30–50 | 189 in its two commits | 20–30 | 132 in its two commits |
| The drawing, and the pictures | `app.rs`, `tests/screenshots.rs` | 50–80 | 54 | the pictures | 70 |
| **SU-5, the whole diff** | | **670–980** | **1,162** | **300–440** | **784** |

The rest of the tests are the headless tests' moves onto the new start
(`headless_game.rs`, +103), the random clicks' (+9), the other support
files (+22), `boards.rs`' test (+4), and `prompt_cost`'s reading of the
tools (+14). Code ran at 1.2–1.7 times its sizing, inside
the 1.5–2.1 the last phases ran at, and tests at 1.8–2.6. What the sizing
left out: item 200's first half moved the start's build to the session and
named the log from it, which the save beside the log needed anyway (its 189
lines are in the bridge's and the session's counts); the bridge's own
forwarding wrapper for the hand-over, five methods; a load's divergence,
played on from the answer before it; the refusals' kinds, which the view
model words; and a writer's count for the window. The band stayed under
2,500 at every commit: 1,946 in all.

### What the build changed in the design

1. **The save's answers are numbered in the order taken**, not by their
   place on a line, so its engine lines are one record `decision_log::read`
   reads, and the session's lines name places by those numbers. A line's own
   numbering is given back when it is replayed (`Save::line_to`).
2. **A savestate is written at the click**, under the record's lock, not
   relayed to the engine's thread as a reply (decision 4, amended where it
   stands): the thread waits at the open prompt, so it writes nothing then.
3. **The game's buttons come first** after the Play | Edit switch. Beside
   Reload at the header's end, they moved as the status changed width, which
   a replay's count does at once, and a double click's second half landed
   on Reload (`engineering-practices.md` §10.1, question 8).
4. **A divergence plays on by building again.** Decision 6's "the window
   plays on from that board" meets decision 1's "a stopped game is never
   continued", so the bridge builds the game again and replays to the answer
   before the one that diverged, its audits paused, as the first pass has
   just checked those answers. The save's line moves there, and the place it
   left is the menu's "Back to where I was", which diverges again if taken.
5. **What a load does with a record that ended inside its setup actions**,
   which SU-4 left to SU-5: it replays the answers the record holds and asks
   the question the setup stopped at, on the board as far as it got. Such a
   record is a refused setup line's, or a panic's during setup, and its
   scenario's Reload is the way to play the setup again.
6. **The undo target is one rule**: the last place the window was asked on
   the line to the save's current place, strictly before it while a question
   is open or a rebuild is on its way to one, else at it. So Undo after a
   panic asks again the question whose answer the validator refused.
7. **The line left survives undo after undo**: the place a move leaves is
   kept while the moves walk back along one line, so three undos leave it
   once, and "Back to where I was" returns to where they began.
8. **`ToWindow::Refused` names what it refuses**, so a load's start that no
   longer builds says the save did not load, and a decision log that cannot
   be made says so, each with its own hint, where the window's one heading
   said the scenario did not load.

#### SU-4 — the replay — ✅ landed 2026-10-03

*Evicted 2026-10-03 from `plans/setup-architecture.md` §8, where the heading and a stub remain; SU-5's sizing stays there.*

### The build as sized (2026-10-03, at #216)

| SU-4, the replay: the engine's half | Where | Code | Tests |
|---|---|---|---|
| `ChoiceKind::as_str`, replacing the eight readers of `Debug` text | `ui::choice_types`; `scenario::setup`, `test_support`, devgui `prompt.rs`, four test files | 40–60 | 15–25 |
| The text: a record's header (the format, the engine), a start (a dealt game's seed, its `GameConfig` destructured with no `..`, and its decks by name; or a scenario's path, seed and text), an answer line and the outcome; written and read, each refusal naming its line; a start built into a game | `state::decision_log` | 330–470 | 120–180 |
| An option's identity, what was chosen: written, and compared by one matcher, which the replay, the setup driver and `ById` use; the scripted provider's answer by identity, `expect_choice`, which new tests use | `ui::choice_types`, `scenario::setup`, `ui::decision` | 150–230 | 80–130 |
| The replay: each line's player, kind, turn and step checked, its choice found by identity among the options this build offers and its fit through `ui::ask`'s predicates, which the validators then assert; a forced line skipped or answered where one build asks it and the other does not; the seats after the last line, or a stop naming where it diverged; a superseded replay stopped | `ui::replay`, `ui::ask` | 200–290 | 170–260 |
| The stop: why a run ended, the catcher over any run, a stopped game refused at each of the six entries; the setup driver's refusal as a stop, which `fuzz_games` reports as the game's error | `ui::decision`, `state::game`, `scenario::setup`, `bin/fuzz_games` | 100–145 | 60–100 |
| No layer audit while replaying within a session (§7.1) | `engine::layers::compute`, `state::diagnostics` | 15–25 | 15–25 |
| The dev GUI on the engine's text and the stop: its log written in the engine's words, and a refused setup line shown as a refusal rather than an engine panic | devgui `bridge.rs` | 35–50 | — |
| **SU-4** | | **870–1,270** | **460–720** |

At the last phases' rate SU-4's code is 1,310–2,670, so 1,770–3,390 in all.
Its upper half crosses 2,500, so the build measures at each commit, code and
tests apart, and stops to report when it crosses. An option's identity and
its matcher are the seam that would move into a PR of their own, ahead of the
replay, since they have consumers without it. **Its consumers**, each a test or a
client of what it builds (`engineering-practices.md` §4: every PR in a split
carries one):
- the replay test at `phase_a6g_integration_test.rs:180` moves onto the
  engine's replay, over the engine's text written and read back, at two seats
  and four;
- the setup driver and `phase_cv2a`'s `ById` find an answer through the one
  matcher, where each keeps a copy of its own today;
- SU-2's refusal test (`phase_su2_integration_test.rs:49`) catches a stop
  where it catches a panic today;
- `fuzz_games --scenario` reports a refused line as the game's error, where its
  `catch_unwind` counts a panic today;
- the dev GUI writes its log in the engine's text, so every game the window
  plays from SU-4 on is a record SU-5's `--load` reads, and it shows a refused
  setup line as the refusal it is.

Its tests beside them: a reordered option list replaying to the same game,
and a choice no longer offered stopping at its line; a scenario game with
setup actions replayed; a cast
from hand with exact mana under `ManaWindowStop`, replayed; each disagreement
(the player, the kind, the turn or step, an answer that does not fit) stopping
at its line; the stop's four reasons, and a panic that is not a stop passing
through; a stopped game refused; the audit's switch; the text's refusals and
a pinned text. Each replay plays a game capped at a few turns, as the a6g
test's eight are, timed in debug: `cargo test` runs the engine at opt-level 0
with the audit on, where a whole Commander game replays in ~35 s (§7.1).

**A/B, predicted before any arm runs.** SU-4 changes nothing a fuzz game's
outcome depends on: `fuzz_games` attaches no log, uses no replay, and meets
the stop only on a scenario's refused line, and `ui::ask`'s validators assert
the predicates they assert today, from one function each. So every gameplay
and cost row is predicted `IDENTICAL` on both pools at two seats and four,
and instructions per decision within ±0.3%, for code the compiler places
differently.

### Sized against built

Lines added, `mtgsim/` and `devgui/` with a `src` file's `#[cfg(test)]`
module counted as tests, read off #216's tip (`c3425c9`): each row its
commits' additions, so a line two commits touched counts in both, and the
totals the whole diff's.

| Part | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---:|---:|---:|---:|
| `ChoiceKind::as_str` and the eight readers it replaced | 40–60 | 44 | 15–25 | 46 |
| An option as what it is, the one matcher, `expect_choice` | 150–230 | 120 | 80–130 | 108 |
| The text: the format, the engine, the start written, read and built, the answer lines, the outcome | 330–470 | 575 | 120–180 | 142 |
| The replay, with `ui::ask`'s checks as predicates | 200–290 | 390 | 170–260 | 245 |
| The stop, and the setup driver's refusal as one | 100–145 | 106 | 60–100 | 122 |
| The audits paused within a session | 15–25 | 40 | 15–25 | 41 |
| The dev GUI's log in the engine's text, its start built through `BuiltStart` | 35–50 | 55 | — | 21 |
| **SU-4** | **870–1,270** | **1,322** | **460–720** | **718** |

The engine's own code is +1,259 against 835–1,220, its tests +680; the dev
GUI +101 with its bridge 39 lines shorter net. What the sizing left out: the
text's module doc, 24 lines of which moved from the old file into `mod.rs`
and count as added; the reader's refusals, which say what was expected where;
the predicates' error text, which keeps the validators' messages word for
word; `GameConfig` destructured and read a line a field. The text row ran
1.2–1.7 times its sizing and the replay row 1.3–2.0 with the predicates in it;
the rest came in near or under their rows.

### What the build changed in the design

1. **`as_str` moved, not added.** The trace sink kept an exhaustive match of
   the names (`trace_records::choice_kind_name`); `ChoiceKind::as_str` is that
   match moved onto the kind, so the trace, the log and a refusal spell a kind
   one way.
2. **Exact names exposed stale tests.** `"ChooseReplacement"` matched only as
   a prefix of `ChooseReplacementEffect`; `"DiscardToHandSize"` named a kind
   RE-8 renamed `Discard`, so `phase_re6`'s "no cleanup discard is asked"
   could not fail; `prompt_subject_test`'s list of every variant held 26 of
   27, its count hard-coded, with `OrderTriggers` missing from its list of
   kinds asked without a subject. All three fixed in the first commit.
3. **A scenario's replay runs no setup driver.** Its record holds the answers
   the driver gave, so `BuiltStart::replay` answers them from the log with the
   rest, and `BuiltStart::play` runs the driver. A record that ends inside its
   setup actions, as one refused there does, replays to its last line, and
   SU-5's load decides what a person sees then.
4. **A start is read within a record.** §8's risk said the reader would read a
   start apart from a record. It reads a line apart (`AnswerLine::read`) and a
   start only within `decision_log::read`, so SU-5's journal hands the reader
   its engine lines and reads its own.
5. **The audits' pause is the caller's.** The replay resumes the audits at the
   hand-over; a run that stops leaves them as its caller set them, since a
   stopped game is read and never continued.
6. **The dev GUI builds its start through the engine.** The bridge's own
   dealing and scenario build went with its log text: it makes a `GameStart`,
   builds it and plays it through `BuiltStart`, so the start a record names is
   the one the window played.
7. **`ReplayControl`** carries the lines answered and the supersede flag
   across threads, for SU-5's replaying count and an undo during a replay.

#### SU-3 — the board editor — ✅ landed 2026-10-03

*Evicted 2026-10-03 from `plans/setup-architecture.md` §8, where the heading and a stub remain, with the seats PR's sizing, which was made beside it.*

### The build as sized (2026-10-03, at the design)

**SU-3, the board editor, re-sized at its design (2026-10-03)** with doc
comments and messages counted, for decision 2's A and decisions 1 and 3 as
recommended (§7b.2). §7a's ~500–900 and the A6g row's ~350–600 came before the
design. SU-1's code came in at about 1.9 times its sizing and SU-2's at about
2.0, each from what its sizing left out (the archive's tables), while SU-1's
devgui part came in inside its range (198 against 150–220). Most of SU-3 is
devgui code like that part, and its refusals are the loader's.

| SU-3 piece | Where | Code | Tests |
|---|---|---|---|
| The names in development listed; `CardWord`'s `Display`; the step words and the tag spelling public | `cards::registry`, `scenario::text`, `scenario::write` | 25–40 | 15–25 |
| The editor: the board, undo, the renumbering, the check and its mark; each input; tags, order, attaching, leaving the battlefield; the search | devgui `editor.rs` | 400–550 | 260–380 |
| The editor's view: the facts, the seats and zones, the card buttons, the card's controls, the words shown as text, the search's results | devgui `editor.rs` | 250–330 | 100–150 |
| Session and launch: the switch, Play from the board's file, "Edit this board", "Edit the scenario", Save, Copy as text, the list, `--edit`; a folder per board | devgui `session.rs`, `bridge.rs`, `launch.rs` | 120–190 | 90–160 |
| The drawing | devgui `app.rs` | 220–320 | two pictures, 30–50 |
| **SU-3** | | **1,015–1,430** | **495–765** |

The editor's tests include its own random clicks (§10.3's shape): random
inputs from what its view offers, checking after each that the board is its
own text read back, that each offered click changes the board or the
selection, and that undo walks back to the start.

**Decision 2's B** adds, row by row: player counters ~45–65, lands played
~20–30, left the game ~20–30, commander damage ~50–80, history ~110–160,
`this turn:` ~90–130, `counters:` lines ~60–90, setup actions ~210–310: ~600–900
in all, so ~2,100–3,100 for SU-3. Decided A, these rows are the editor's
advanced settings, slotted on A6g's row after the "why" panel (§7b.2,
decision 2).

**The seats PR** (§7's "Seats", decision 4), with decision 5's lever:

| Seats PR piece | Where | Code | Tests |
|---|---|---|---|
| A stack of decorators and a yield per seat; the prompt names its seat; N decks; the agent and the two-seat refusal gone | devgui `bridge.rs`, `prompt.rs` | 50–80 | — |
| `--players N` | devgui `launch.rs` | 15–25 | 10–15 |
| The asked seat named and marked; seats in a fixed order; the outcome by seat | devgui `view_model.rs`, `app.rs` | 30–50 | 25–40 |
| The tests answering every seat; a dealt four-seat game and the four-seat sample played by rule | devgui tests | — | 60–100 |
| The engine at opt-level 1 in debug | `devgui/Cargo.toml` | 3–5 | — |
| **The seats PR** | | **~100–160** | **~95–155** |

Its review pictures are drawn again, since a prompt names its seat. Of its
~200–320, four seats are ~50–80: `--players`, and the four-seat games.

**A/B, predicted before any arm runs.** Neither PR changes what a fuzz game
runs. The seats PR touches no engine file. SU-3's engine changes are a new
listing and spellings made public, read by nothing `fuzz_games` reaches. So
every gameplay and cost row is predicted `IDENTICAL` on both pools at two
seats and four, and SU-3's instructions per decision within ±0.3%, for code
the compiler places differently.

### Sized against built

Lines added, `mtgsim/` and `devgui/` with a `src` file's `#[cfg(test)]`
module counted as tests, read off `main` (`7832f22`).

| Part | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---:|---:|---:|---:|
| The names in development, the spellings public, a card line written straight into its formatter | 25–40 | 50 | 15–25 | 27 |
| The editor: its model and its view (`editor.rs`), and the search (`search.rs`) | 650–880 | 1,379 | 360–530 | 328 |
| Session and launch: the switch, Play, "Edit this board", "Edit the scenario", Save, the list, `--edit`, the folders (`boards.rs`) | 120–190 | 376 | 90–160 | 225 |
| The drawing (`app.rs`), and `prompt_cost`'s editor readings | 220–320 | 359 | 30–50 | 41 |
| **SU-3, against `main`** | **1,015–1,430** | **2,164** | **495–765** | **621** |

The editor's random clicks (`tests/random_clicks.rs`) are its tests' 120,
in the second row. What the sizing left out: a type for each kind of
control in the view, each with its doc comment; thirty inputs, each a
variant with its line; the rules' helpers for references, tags and order; the
counter controls, one per kind; the folders as a module of their own; and
the read-back check. As SU-1's and SU-2's code ran at about twice its sizing,
SU-3's ran at 1.5–2.1 times, all but 50 lines of it in the dev GUI. The band
crossed 2,500 at the session's commit; the owner had said a dev GUI overage
may run (2026-10-03).

### What the build changed in the design

1. **The battlefield is one zone in the editor's order.** §7b.1 said a card
   moves up or down in its zone; across the seats that left Humility unable
   to arrive before Opalescence when each was its seat's only permanent, so
   the battlefield's order is one, its cards numbered across the seats, and
   the other zones are each seat's own.
2. **A line of several moves one copy.** Moving a card out of `library 0:
   Forest | x10` takes one Forest, so a line on the battlefield is always one
   permanent, which a tag can name.
3. **The random clicks found two offered clicks that changed nothing.** A
   move past only identical lines (two `Forest | x5` lines, one put back by
   hand) and a second copy of a tagged card, which the grammar refuses. A move
   is offered only when it passes a line that says something else, and a
   tagged card's copies have no "+".
4. **The read-back compares the text.** A name holding `#` reads back as
   another board, the comment cut off, with no parse error, so an edit is
   made only when the board read back writes the same text.
5. **`commander` off drops the commander damage that named only it**, as
   leaving the battlefield drops the words naming only a permanent: an edit
   that takes a card out of a reference's reach takes that reference.
6. **A fourth spelling made public**: `PlayerWord`'s Display, the player
   words the editor shows as text. And `CardLine`'s Display writes each word
   into its formatter: the snapshot's board text read 838 allocations on
   `main`, 904 with `CardWord`'s Display through `to_string`, and 601 now.
7. **The build's choices** the design left to it: a committed file's first
   save in a session makes a board, the first of `<stem>`, `<stem>-2`, …
   with no board file, and later saves write that board; "Edit this board"
   and "Save board as scenario" name theirs for the game's start and turn, a
   dealt game's `seed-N-turn-T`; a file's leading comment block, the comment
   and blank lines above its first line, is written back above the board.

#### SU-1 — the scenario loader — ✅ landed 2026-10-01

*Evicted 2026-10-01 from `plans/setup-architecture.md` §8, where the heading and a stub remain.*

### The build as sized (2026-10-01, before the build)

Each commit is measured as code and tests apart.

1. **The doors.** The performer's state half and the construction door; the
   arrival turn; `intrinsic_entry_mods` moved from `test_support` into the
   engine, both calling one copy; counter-kind names moved into the engine,
   devgui's labels reading them. Tests: the door emits nothing, registers rows
   and gives loyalty.
2. **`mtgsim::scenario`'s types, parser and errors**, names and tags included.
   Tests: each error class.
3. **The loader**, its checks, the resume entry, the streams function, and the
   list of cards in development (§7a). Tests: one per word in §5.1; load emits
   nothing; a scenario game played twice is one game; the template and each
   sample under `mtgsim/scenarios/` load.
4. **The writer**, its destructure and its report, and the round-trip test over
   played boards (§4.3).
5. **`fuzz_games --scenario`**, droppable (~40 lines): random games from a
   board, and the three-hash-seed check from one.
6. **devgui:** the start, the argument, Reload, Save, load errors and the log
   header. Tests: the headless game from a scenario; the review pictures drawn
   from scenarios, with no seed hunted.
7. **Docs:** this file's ✅ section, `codebase-state.md` items for §10's
   findings with their slots, `roadmap-v2.md` A6g's row as built (it names
   SU-1 to SU-3 in their slots since this design's review), the
   `fuzz-record.md` block.

**A/B.** The performer's split and the arrival parameter carry the same values
in play, so `close_out.py` runs once and predicts `IDENTICAL` on both pools,
with item 194 merged first since it moves every two-seat game. **Review path:**
the PR body sorts files by how to review them, carries a click script in Magic
terms, and shows the pictures.

### Sized against built

Measured at each part's commit against the one before it; the parts' sums
count a line edited twice twice, so the whole is read off `main`.

| Part | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---:|---:|---:|---:|
| 1. Doors | 60–90 | 166 | 60–90 | 38 |
| 2. Parser | 250–310 | 603 | 130–170 | 83 |
| 3. Loader, checks, resume, streams, cards in development | 340–440 | 594 | 320–400 | 339 |
| 4. Writer and round trip | 180–250 | 707 | 140–220 | 265 |
| 5. `fuzz_games` | 30–50 | 49 | — | 0 |
| 6. devgui | 150–220 | 198 | 60–100 | 149 |
| **SU-1, against `main`** | **~1,010–1,360** | **2,307** | **~710–980** | **874** |

What the sizing left out, by part: the grammar table in the module doc (moved to
the design's §5.1 at review, 2026-10-01) and
the refusal messages that say what to change (2 and 3); the writer's
destructures, which name 97 fields a line each, and the report's ~30 checks
(4); `CounterType::name` and the move of `intrinsic_entry_mods`, counted
again where it landed (1). The tests came in near the sizing. The total
crossed the band's 2,500 at the writer's commit, before its tests; the
owner kept the PR whole there (2,515, with ~3,000–3,300 projected).

### What the build changed in the design

- **`arrived turn N`** beside `arrived this turn`: CR 302.6 measures from the
  controller's own most recent turn, so a non-active player's creature that
  arrived on their last turn is still sick, and "this turn" cannot say so.
  The writer writes an arrival only where that comparison still reads it.
- **One attacker per blocker.** A word that names a card takes the rest of
  its line, since a name may hold a comma; `blocking <card>, …` could not
  parse. No registered card blocks two.
- **`this turn: <card> | …`**, the bar like every other card line, and
  `ability N` by printed place for a card with two abilities it could mean.
- **A graveyard is listed bottom first.** §4.3's rule stamps lines in file
  order and each card goes on top; the example's "top first" contradicted it.
- **`dealt first-strike damage`** is refused before the first-strike damage
  step, not off it: the engine keeps the set to the end of combat.
- **Two refusals the design did not list:** turn 1's draw step at two seats
  (CR 103.8a, 500.11, from the brief), and a library that mixes shuffled and
  listed lines.
- **`cast` and `x_value` are named, not reported:** nothing reads them once the
  entry has been dispatched. `cost_choices` is reported: a condition reads it
  at rest.
- **The round trip's first run found a loader bug:** CR 508.8 skips declare
  blockers and the damage steps, and the loader also refused the end of
  combat step without attackers. Fixed before the writer's commit.

#### SU-2 — setup actions — ✅ landed 2026-10-01

*Evicted 2026-10-01 from `plans/setup-architecture.md` §8, where the heading and a stub remain.*

### The build as sized (2026-10-01, before the build)

§8 sized SU-2 at the driver ~180–260 lines, the action words ~80–120 and tests
~150–270, ~400–650 in all. The brief asked for a re-sizing before the build,
counting the arms its six open questions implied; it came to code ~570–735 and
tests ~370–530 in the band, and ~60–90 more in devgui, with SU-1's factor of
1.9 in mind.

### Sized against built

Measured at each part's commit, `mtgsim/src` and `mtgsim/tests` with a
`src` file's `#[cfg(test)]` module counted as tests; the whole is read off
`main`.

| Part | Code, sized | Code, built | Tests, sized | Tests, built |
|---|---:|---:|---:|---:|
| 1. The words (`board.rs`, `text.rs`, `mod.rs`) | 80–120 | 121 | — | 31 |
| 2. The loader's resolution and refusals (`build.rs`) | in the driver's | 129 | — | — |
| 3. The driver (`setup.rs`), `fuzz_games` | 180–260 | 374 | 150–270 | 229 |
| 4. devgui | — | 16 | — | 31 |
| 5. Item 198 | 10–20 | 16 | a test | 54 |
| **SU-2, against `main`** | **~260–380** | **640** | **~150–270** | **314** |

The band's figure is +954 −47; devgui adds +47 −9 and one picture. What the
sizing left out: the names resolved in the loader rather than the driver (the
brief's one road for names), with their refusals; the match over all 22
`ChoiceKind`s, which a new question has to pass; the tap and the generic
split; and the refusal messages and doc comments, as SU-1's did. The
re-sizing came within its range, with the six open questions' arms counted.

### What the build changed in the design

- **`mode N` and `x N` wait.** Nothing asks for a mode: `Effect::Modal`
  cannot resolve (`backlog.md` §2.7). No X spell is offered at priority:
  `ManaPool::can_pay` and `find_mana_sources` read no X (`codebase-state.md`'s
  CR 107 row), so no game reaches CR 107.3a's question, and `AmountExpr::X`
  errors at resolution besides. `x N` was built in the first commit and left
  in the second, when its test found the window never offered the spell. The
  driver names `ChooseXValue` among the questions no line answers.
- **`AutoPayer` taps nothing.** It orders cost reductions; the window's pick
  is the driver's, the random agent's preference taking its first source, and
  the line's own permanent last: Mind Stone's ability otherwise taps the stone
  for its own {1} and rewinds.
- **The refusal of a line no seat reaches comes one prompt early.** The driver
  stops every seat at every priority point while a line is left, so a seat
  with `Pass` alone is asked rather than passed over, and the line is refused
  when its seat holds priority without its action.
- **Setup actions resolve nothing.** §0, §2, §5.2 and §7a had said they build
  a resolved effect; with two verbs and that refusal, they build a stack, and
  the seats' passes resolve it. A line that resolves the stack is open.
- **Names resolve at the load.** The design had the driver resolve each name;
  the loader does, through the table it built the board with, and hands the
  ids over beside the game, so its checks are refusals at load rather than in
  play. A target is a permanent, a card in a graveyard or exile, or a spell
  an earlier line casts, so a library's twenty Forests need no tags.
