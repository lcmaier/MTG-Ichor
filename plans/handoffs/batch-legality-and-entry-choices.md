# Handoff — `batch-legality-and-entry-choices`, the design note

**Opened 2026-10-25**, A6e's cleanup PR 5b (`item-6-audit.md`'s cleanup
plan, row 5b, and its entry under "What each PR takes"). This file is the
design note, reviewed before the build. It holds only what #293 left open:
the decisions #293 settled for this PR are in
`plans/handoffs/batches-durations-and-control.md` (its decisions 2 and 3,
and item 330's answer), and the landing deletes both files. Every size below
was read against `384f2243` (#293's merge). **Revised at #294's review
rounds (2026-10-26)**: decisions 1 and 3 are rewritten (3 twice, the second
time from a Scryfall spot check), 2 is folded into 1, and 4, 5, 6 and 7 are
settled.

## Where the work sits

All of it is one batch's path: a proposal, the CR 616.1 loop that decides
every member against one board (phase 1), the performance (phase 2), then the
triggers. The words "batch", "phase 1" and "phase 2" are the #293 note's.

- **Phase 1**: the entry replacements' choices. Devour (items 61, 62, 63), a
  shockland's 2 life (item 134), tribute's opponent and its "may" (item 332).
  Each is decided here, and nothing is performed.
- **Phase 2**: what phase 1 decided. Devour's sacrifices and the life paid
  come first, as an event of their own, then the entries (decision 3).
- **Triggers**: item 330's filter reads the object as its event found it.

No card is registered. Each item's card is a fixture in its test file
(Thromok, Dragon Broodmother's token, Corpsejack Menace, a shockland, a
tribute creature, Cloudsculpt Armorer), which is how both pools stay
identical.

## The re-count

Code and tests apart, since the plan carries code at 1.5–2×. Item 26 leaves
this PR either way decision 1 goes.

| Item | What it builds | Code | Tests |
|---|---|---:|---:|
| 61 | Devour decides its moves and counts them; the moves ride on their entry and perform as a window of their own before it; phase 2's per-window work as one function; `execute_actions_new_batch` goes; the check | 130–190 | 120–160 |
| 62 | `apply_auxiliary_move`'s unused entering-controller parameter goes; `grants_devour` becomes a real Layer 6 grant, its chooser asserted | 0–15 | 50–70 |
| 63 | `PerChosen::Fixed(n)` and `PerChosen::Count`, at four literals and two readers | 20–35 | 30–50 |
| 134 | `EnterModsTemplate.unless_paid`, CR 614.12b's running total, the payment riding on its entry, and its prompt | 110–160 | 120–160 |
| 330 | `named_subject_matches` reads the record's existence as its event found it | 10–20 | 60–90 |
| 332 | Tribute's opponent asked as the creature enters, the counters' "may", and their prompts, Xantcha's folded in | 60–95 | 70–100 |
| **Total** | | **330–515** | **450–630** |

That's **780–1,145 sized** and **945–1,660 carried**, against the brief's
900–1,270 and 1,100–1,900. Item 26 leaving and decision 3 holding no
batch are the difference. Most of the range is under the band's
floor, which the owner accepted for the split.

## Decisions

1. **Item 26: no producer reaches it, even with item 281.** Seeds of
   Strength is registered (A4i), but it's three instructions, "Target creature
   gets +1/+1 until end of turn" three times, each its own effect: one creature
   chosen for all three gets three effects, and no batch carries two members.
   Item 26 is one instruction acting on two objects at once, the same one
   twice. Item 281 (`Group` takes a target) wouldn't make that batch either:
   `Group` collects what its members find and drops a repeat
   (`resolve.rs`, the `EffectRecipient::Group` arm of `resolve_effect`), so
   "exile target creature and target creature" with one creature chosen twice
   proposes one exile. CR 608.2b and 609.3 are met where each producer builds
   its batch. One instance can't repeat a target (CR 115.3), `Group` drops
   repeats, two instructions are two batches, the state-based check orders
   its members (the counters off before the death, the unattach after). The
   one `Rewrite` arm that points an event at another object is `Retarget`,
   a damage recipient (Pariah), and no producer puts a move beside damage in
   one batch.
   - (a) **Close item 26 as met at the producers**, and add to `Rewrite`'s
     growth contract that an arm that can point an event at an object an
     earlier member of the batch moves must say what happens then.
   - (b) **Move it to PR 6 with item 281**, re-filed with this reachability
     and built there with a fixture batch, since item 281's spell doesn't
     reach it either. The first draft's decision 1 goes with it: compare the
     existence each member was decided against (CR 400.7), not the zone.

   **Recommended: (a).**

2. **Folded into 1** at the first review round.

3. **Item 61: devour's moves are their own event, right before the entry,
   inside the entering batch.** Revised at both review rounds. Nothing
   moves in phase 1 after this PR. Today `apply_auxiliary_move` performs the
   sacrifices in a nested batch while phase 1 is still deciding (`pipeline.rs`,
   its `execute_actions_new_batch` call). RC-5 did that because the entering
   creature's counters depend on how many were sacrificed, a later
   replacement in the same loop reads them (Corpsejack Menace doubles them,
   Hardened Scales adds one), and RC-5 counted what was performed. #293's
   decision 2 counts what was decided instead: a replacement that drops a
   move (CR 614.6) has already dropped it when devour counts. What's left is
   when the moves happen relative to the entry.

   **The spot check (Scryfall rulings, 2026-10-26)**, on the cards whose "as
   this enters" moves other objects. Wood Elemental's, Overlaid Terrain's
   and Shimatsu the Bloodcloaked's rulings each say the sacrifice happens
   "right before" the permanent enters (Shimatsu's: before it enters, so
   nothing entering beside it can be chosen). Thunder-Thrash Elder's says all
   creatures devoured across simultaneous entries are sacrificed at the same
   time. Devouring Hellion's says a sacrificed permanent's "whenever a
   creature enters" never sees the Hellion enter, and that no player acts
   between the choice and the entry. So the moves are one event across the
   batch's entries, and it comes before them: two events, in that order. The
   first round's one event contradicts the first three rulings.
   - (a) **Two events in one batch.** The moves ride on their entry as
     decided members, as the first round had them: each is decided as devour
     applies, through the CR 616.1 loop with a fresh applied set (a finality
     counter still exiles a devoured creature), recorded on
     `GameState.entry_selection` while the loop runs (a fork at the next devour
     prompt needs it, `codebase-state.md` item 40), and moved onto the
     entering member when its group's loop ends, beside `BatchMember::inner_move`.
     Phase 2 performs every entry's moves first, under a window of their own
     that is dispatched at its close, then the entries in the batch's own
     window. That is the shape a replacement's riders already have on the
     other side (CR 615.5): a window of their own, right after the event.
     Nothing is held across batches and no batch is split. What phase 2 does
     for a window (each departing object's last known information, the
     look-back snapshot, the dispatch) runs once per window, so that part
     becomes one function phase 2 calls twice. `execute_actions_new_batch`
     goes, and `// AUXILIARY-MOVE:` marks the moves' window instead, so
     CLAUDE.md's chokepoint bullet keeps the tag with a new place to point.
     A shockland's payment rides in the same window (#293's decision 3).
   - (b) **One event**, the first round's: the moves performed with the
     entry under the batch's own id. It contradicts the rulings above.
   - (c) **The first draft's**: a decided batch held on `entry_selection`
     and performed between the entering batch's phases by a split
     `execute_batch_inner`. The same two events as (a).

   No other "as this enters" family changes. Copies, the choices of a value
   or a player, and the reveal lands move nothing, so they have nothing to
   time. Thunder-Thrash Elder is in the performance pool, and a single devour
   entry's sacrifices stay a window dispatched before its entry performs, as
   today, so both pools are predicted identical. **Recommended: (a).**

4. **Item 134: the cost's type.** **Settled: `Cost`** (the owner, at #294's
   review, 2026-10-26), with only `PayLife` built here. Any other arm is an
   `Err` as it applies, the convention `Cost` already keeps for its unbuilt
   arms. The shockland is `EnterModsTemplate { status: Tapped, unless_paid:
   Some(Cost::PayLife(2)) }`. CR 614.12b is checked as each payment is chosen,
   against what the event has chosen so far: life at least the sum (CR 119.4),
   and nothing that says the player can't lose life (CR 119.8). Any part of an
   affordable set is affordable, so choosing one at a time reaches every set
   the player could pay for together. The payment is a `LoseLife` caused by
   `EventCause::Cost { player, paying_for }`, riding on its entry
   (decision 3).

5. **How an entry's choices are asked.** **Settled: (a), by answer shape**
   (the owner, at #294's second review round, 2026-10-26). The first draft
   designed three prompts for the three cards this PR has. Every printed "as this enters"
   choice is CR 614.12a's, made before the permanent enters, and the cards
   group by what the player answers:

   | The answer | Printed families | Cards | Asked today by |
   |---|---|---:|---|
   | A value | a color 53, a creature type 57, a card name 15, a land type 8, a number 5, a card type 3, one of two words 20 (anchor words, odd or even), riot 13 | ~175 | nothing: CR 607's linked choices, which `route-review` decides |
   | A player | "choose a player" or "an opponent" 17, enters under an opponent's control 4 (Xantcha), tribute 11 | 32 | `ChooseEnteringController` |
   | Objects | devour 23 and Sutured Ghoul's shape ~10 (CR 614.13), enter as a copy ~60, enter attacking (CR 508.4), a card to reveal 23 (the reveal lands), amplify 9, another permanent to remember ~15 | ~140 | `ChooseAuxiliaryZoneChange`, `ChooseCopySource`, `ChooseEnteringDefender` |
   | Yes or no | pay a cost 26 (the shocklands, the double-faced lands' 3 life), unleash 14, tribute's opponent putting the counters 11 | 51 | nothing |

   Scryfall, 2026-10-26: `game:paper -is:funny` with `o:"as ~ enters, <phrase>"`
   or `o:/as this [a-z]+ enters, <phrase>/` (287 cards print the phrase at
   all), keywords by `kw:`. This PR's customers are a yes or no (pay, put
   counters) and a player (tribute's opponent).
   - (a) **By answer shape.** One arm per shape, with what is asked carried
     as data, the way `ChooseAuxiliaryZoneChange` already carries its
     destination. This PR adds two arms.
     `ChoiceKind::AcceptAsItEnters { object, source, offer: EntryOffer }` is
     the yes or no, with `EntryOffer::Pay(Cost)` for the shockland and
     `EntryOffer::PutCounters { counter, n }` for tribute's opponent (and
     unleash's, later). `ChoiceKind::ChoosePlayerAsItEnters { object, source,
     role: EntryRole }` is the player, with `EntryRole::Controller` for
     Xantcha's `ChooseEnteringController`, folded in (17 lines in 4 test
     files), and `EntryRole::CounterPutter` for tribute. A later family adds
     an offer or a role, with its display and "why" text. The objects shape
     keeps its three arms, which could fold the same way later. The values
     shape is CR 607's.
   - (b) **By question**, as the four entry prompts are today: `PayAsItEnters`,
     `PutCountersAsItEnters`, and tribute's opponent as `ChooseOpponent`.
     Each later family adds its own arm with its first card, about 40 lines
     each. Two arms would already ask "which opponent" as a permanent enters.

   Both bring in the dev GUI gate, and neither costs anything at run time:
   a prompt is asked only when a card asks it.

### #293's two open questions

6. **"With the first registered card" as a Slotted line.** **Settled:
   retired** (the owner, at #294's review, 2026-10-26), in `dd89d789`. The
   board script no longer reads it as a home, the practices page and
   `codebase-state.md`'s header drop it, the two lines that named it beside a
   phase name the phase only, and item 284 closes as moot.
7. **The three `Query` names.** **Settled: (a)** (the owner, at #294's
   third round, 2026-10-26, on the condition that only the "can't" check
   reads a `Query`). It holds by design: `Query` is the one question CR
   101.2's reader takes, and its three readers are that module's own,
   `is_prohibited`, `prohibition` (which "can't" forbids it, for the trace and
   devour's candidate filter) and `rule_applies`, the first negated. Every
   site reads as a question. `is_prohibited` asks one question three
   ways: may this event happen (`Event`, CR 614.17's "can't be destroyed"),
   may this replacement effect be applied (`TryApplyReplacement`, CR
   701.19c's "can't be regenerated"), and does this rule's action apply
   (`TryApplyRule`, CR 101.1's "you don't lose the game for having 0 or less
   life"). `Try` went on the second two at #293's review because each can
   come back "no", so the name says the application is asked about, not
   done. `Event` can come back "no" just as well (the owner, at #294's second
   round), so the first draft's reason for leaving it alone doesn't hold. The
   three should read alike.
   - (a) **Each named for what might be prohibited, said as a clause:**
     `EventHappens`, `ReplacementApplies`, `RuleApplies`.
     `is_prohibited(game, &Query::RuleApplies { .. })` reads "is it
     prohibited that the rule applies", a question, so nothing reads as done.
   - (b) **`Try` on all three:** `TryEvent`, `TryApplyReplacement`,
     `TryApplyRule`. `TryEvent` names no verb.
   - (c) **`Check` on all three**, your `CheckEventHappens`: it repeats what
     `is_prohibited` already says.

   15 sites for `Event` and 11 for the other two, in the build's first
   commit.

## What the build checks

- Item 61, in debug builds: a batch's entry moves perform as one window
  ahead of its entries, and they're exactly the members devour counted.
- `cargo build --all-targets` with zero warnings, `cargo test`,
  `cargo clippy --all-targets -- -D warnings`, and the `check_*.py` family.
- The close-out's A/B on both pools, predicted identical: no registered card
  reaches a new path, and Thunder-Thrash Elder's window keeps its records'
  order (decision 3).
- The CPU budget on phase 2's per-member read of the moves riding on it, the
  floors, and `specdb owed --phase "Phase 7"` still clean.
- The audited smoke, since phase 2 changes.
- The dev GUI gate, for decision 5's new arms.

Each fix is shown failing first against the tree without it.
