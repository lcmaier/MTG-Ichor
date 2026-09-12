//! Cards for Phase RE — the remaining event kinds (`replacement-architecture.md` §9).
//!
//! # RE-1 — skips, and the turn queue (CR 614.1b, 614.10, 614.10a, 500.7)
//!
//! **Five printed cards on two axes: which unit a skip names, and where the
//! effect comes from.** The unit is what CR 614.10 replaces — a turn, a phase
//! or a step — and the source decides whether the row can be stripped, counted
//! or targeted:
//!
//! | Card | Unit | Source | Uses |
//! |---|---|---|---|
//! | [`yawgmoths_bargain`] | the draw step | a static ability, `You` | `Static` |
//! | [`eon_hub`] | the upkeep step | a static ability, `Everyone` | `Static` |
//! | [`meditate`] | a turn | a resolution, `You` | `Once`, `Indefinite` |
//! | [`time_walk`] | — (it *makes* a turn) | a resolution | — |
//! | [`moment_of_silence`] | the combat phase | a resolution, a **target** | `Once`, `UntilEndOfTurn` |
//!
//! Meditate is the first `Duration::Indefinite` row in the crate that ends by
//! *use* and never by time, which is what "your **next** turn" means; the
//! variant has existed since the registry was written with nothing to carry.
//! Time Walk is in the same PR as the skips rather than in its own because
//! the Meditate-then-Time-Walk board — a skip consuming an extra turn,
//! CR 614.10a's "the first occurrence that isn't skipped" — is what proves the
//! queue and the pipeline meet, and building them apart means rewriting
//! `advance_turn` twice.
//!
//! **Chronatog and Relentless Assault are out**, each for a facility rather
//! than for size. Chronatog's "Activate only once each turn" is an activation
//! limit `ActivationRestriction` does not have and this PR has exactly one
//! customer for; Relentless Assault's "an additional combat phase followed by
//! an additional main phase" is CR 500.8's extra *phases*, the turn queue's
//! second level, which §9 cut from this phase and RE-1's review took back in
//! as **RE-10** — where Aggravated Assault is the card, because Relentless
//! Assault also needs "creatures that attacked this turn" and nothing tracks
//! it.
//!
//! # The rulings pass (Scryfall, 2026-09-11)
//!
//! Every ruling on all five cards, with what became of it
//! (`engineering-practices.md` §3.4). Yawgmoth's Bargain and Time Walk's
//! ordering ruling are the two that carry no board of their own.
//!
//! - **Yawgmoth's Bargain** — Scryfall lists no rulings. Its tests are the
//!   rule's: the draw step's *contents* are not proposed at all
//!   (`ATOM-614.10-001`).
//! - **Eon Hub**, *"The upkeep step is skipped entirely. The turn proceeds from
//!   untap step to draw step."* → the event log, asserted as the absence of a
//!   `StepBegin { Upkeep }` between the untap and draw ones.
//! - **Eon Hub**, *"Upkeep-triggered abilities don't trigger, and 'activate
//!   only during your upkeep' abilities can't be activated."* → the first half
//!   is item 6's and falls out — a step that does not begin emits no event for
//!   a trigger to read, which is why this PR goes first. The second half has
//!   **no facility to assert against**: `ActivationRestriction` has no
//!   step-scoped arm, and no registered card carries one. Recorded, not
//!   skipped.
//! - **Eon Hub**, *"Any triggered abilities that triggered during the untap
//!   step will go onto the stack at the start of the draw step."* → item 6's,
//!   for the same reason. Nothing triggers yet. **Both Eon Hub rulings are
//!   booked as two tests item 6 owes** — `codebase-state.md` item 121, which
//!   names the board and sizes them.
//! - **Meditate**, *"You skip one turn as part of the effect."* → one row,
//!   `Uses::Once`, so one turn; and two Meditates skip two, which is
//!   CR 614.10a's own sentence and the board that needed a real queue.
//! - **Time Walk**, *"If multiple 'extra turn' effects resolve in the same
//!   turn, take them in the reverse of the order that the effects resolved."*
//!   → the queue is a stack, tested with two extra turns for two different
//!   players so that the order is observable at all.
//! - **Moment of Silence**, *"The player skips their next combat phase this
//!   turn (if any). If they manage to have two combat phases, then only their
//!   next one combat phase is skipped."* → `Uses::Once`, and the second
//!   sentence is tested against a second `BeginPhase { Combat }` proposal the
//!   fixture makes by moving the cursor, because CR 500.8's extra phases are
//!   unbuilt. **RE-10 replaces the fixture with Aggravated Assault.**
//! - **Moment of Silence**, *"It must be used before the combat phase starts or
//!   it has no effect."* → `ATOM-614.10-002`: a row created during combat meets
//!   no proposal and expires at cleanup unused.
//! - **Moment of Silence**, *"If cast on a player when it is not their turn, it
//!   has no effect."* → falls out of the subject rather than being coded: a
//!   `BeginPhase` event is about the **active** player, so a row scoped to
//!   anyone else watches nothing this turn.
//!
//! # What a random deck can draw
//!
//! Eon Hub is the pooled card: `{5}` colourless, so every deck can cast it, and
//! its effect is a *dropped* turn-structure proposal on every player's upkeep
//! for as long as it is on the battlefield — the first card in
//! `PERFORMANCE_POOL` whose cost is measured in proposals that go nowhere.
//! Yawgmoth's Bargain is registered and stays out: a random agent with one use
//! for its life total empties its library, which is the board RE-6's Laboratory
//! Maniac path wants and a distortion of every fixture until then. Time Walk
//! stays out because an extra turn in every blue deck moves `Avg turns/game` by
//! design. Meditate and Moment of Silence stay out as one-shots whose engine
//! path Eon Hub already opens.
//!
//! # RE-2 — draw (CR 614.11, 614.11a, 121.2, 121.2a, 121.6a/b, 616.1g)
//!
//! **Four printed cards on two axes: which of the two draw events the effect
//! watches, and whether the replacement keeps the draw or moves it.** CR 121.2a
//! makes the instruction an event of its own, and Alms Collector's ruling is
//! what makes the pair honest: *"count how many times the word 'draw' is
//! used."*
//!
//! | Card | Watches | Rewrite | Keeps the draw? |
//! |---|---|---|---|
//! | [`thought_reflection`] | any individual draw | `Instead(DrawCards { n: 2 })` | yes |
//! | [`teferis_ageless_insight`] | an individual draw that is not the draw step's first | `Instead(DrawCards { n: 2 })` | yes |
//! | [`alms_collector`] | an instruction of two or more | `Instead(DrawCards { n: 1 })` + a rider | yes, cut to one, plus one for you |
//! | [`notion_thief`] | an opponent's individual draw, not their draw step's first | `Instead(DrawCards { n: 1, player: You })` | yes, with a new subject |
//!
//! **Both "and" cards were mis-filed, and the same clause of CR 614.5 fixes
//! both.** A replacement gets one opportunity to affect "an event **or any
//! modified events that may replace that event**", so whatever the rewrite
//! outputs carries the applied set and whatever a rider proposes does not.
//! Notion Thief was `Prevent` plus a rider until RE's sizing; Alms Collector
//! was `Prevent` plus *two* riders until this PR's tests ran. Each card's own
//! ruling names the loop that encoding produces — two Thieves trading a draw
//! forever, Alms Collector and an opponent's Thought Reflection trading cards
//! forever — and each is fixed by putting the half that keeps the subject in
//! the rewrite. Notion Thief keeps the draw and changes its subject; Alms
//! Collector keeps the subject and changes the count. Only Alms Collector has a
//! genuinely new subject left over, and that one draw is its rider.
//!
//! # The rulings pass (Scryfall, 2026-09-11)
//!
//! Every ruling on all four cards, with what became of it, **and the test that
//! carries it** — `engineering-practices.md` §3.4 asks for the name, so the next
//! reader can go from a printed sentence to the assertion without searching.
//! Fifteen rulings; ten are tests in `tests/phase_re2_integration_test.rs`,
//! three fall out of the shape and are asserted anyway, and two have no
//! facility to assert against and say so.
//!
//! - **Thought Reflection**, *"If a spell or ability causes you to draw
//!   multiple cards, Thought Reflection's effect doubles each card draw. For
//!   example, if you cast Harmonize ('Draw three cards'), you'll draw six
//!   cards."* → `ATOM-121.2a-001` from the inner side: the instruction is not
//!   what it watches, so three individual draws each become two.
//!   → `thought_reflection_doubles_each_of_a_three_card_instruction`
//! - **Thought Reflection**, *"The effects of multiple Thought Reflections are
//!   cumulative ... two ... four times the original number ... three ... eight
//!   times."* → **the acid test**,
//!   `test_two_thought_reflections_draw_four_not_infinity`. Not legendary, so
//!   the pool can build two, and the 2ⁿ is what §3.2d's lineage rule buys.
//!   → `test_two_thought_reflections_draw_four_not_infinity`, with
//!   `three_thought_reflections_draw_eight` for the exponent.
//! - **Thought Reflection**, *"If two or more replacement effects would apply
//!   to a card-drawing event, the player who's drawing the card chooses what
//!   order to apply them."* → falls out of CR 616.1's chooser being the
//!   affected player. **Not asserted on two Reflections**: their order provably
//!   cannot change the answer, so the engine does not ask (§11 item 55). The
//!   boards that do ask are
//!   `a_draw_doubler_beside_a_notion_thief_is_a_real_choice`, where the two
//!   answers differ, and the three-Thief board, where the chooser moves with
//!   the event's subject.
//! - **Teferi's Ageless Insight**, *"If a spell or ability causes you to put a
//!   card into your hand without specifically using the word 'draw,' it's not a
//!   card drawn."* → structurally true and asserted: a `ZoneChangeCause` that
//!   is not `Drawn` proposes no draw at all, so there is no event to watch.
//!   → `a_card_put_into_hand_is_not_drawn_and_no_draw_replacement_sees_it`
//! - **Teferi's Ageless Insight**, *"If two or more replacement effects would
//!   apply to a card-drawing event, the player drawing the card chooses the
//!   order in which to apply them."* → as above.
//! - **Teferi's Ageless Insight**, *"Because [it] is legendary, it's unlikely
//!   that one player will control two. However, if that happens, each card that
//!   player would draw after the first will result in four cards being drawn."*
//!   → the legend rule makes this Thought Reflection's test, which is why the
//!   acid test is on that card. Teferi's own board is the one decision 1 named:
//!   beside a Thought Reflection in the draw step it draws **three**.
//!   → `teferi_beside_thought_reflection_draws_three_in_the_draw_step`, with
//!   `teferi_excepts_the_draw_steps_first_card` and
//!   `teferi_doubles_a_draw_that_is_not_the_draw_steps` either side of it.
//! - **Alms Collector**, *"[Its] replacement effect applies to an instruction
//!   to draw more than one card before any replacement effects apply to
//!   individual cards drawn."* → `ATOM-616.1g-001`, with Thought Reflection on
//!   the other side of the board.
//!   → `alms_collector_applies_to_the_instruction_before_thought_reflection_sees_a_draw`
//! - **Alms Collector**, *"Once a replacement effect has been applied to an
//!   event, it can't be applied again to the resulting events ... Thought
//!   Reflection can double that player's resulting card draw without Alms
//!   Collector's replacement effect applying again."* → **the ruling that
//!   changed the card's encoding**, and a test. As `Prevent` plus riders the
//!   board is an infinite loop; as an `Instead` on the count it is three cards,
//!   which is what the ruling describes.
//!   → `alms_collector_does_not_apply_again_to_the_draws_it_produced`
//! - **Alms Collector**, *"To determine whether a player is instructed to draw
//!   multiple once or instructed multiple times to draw one card, count how
//!   many times the word 'draw' is used."* → test. Ancestral Recall (pooled) is
//!   one "draw" of three and meets it; two `Primitive::DrawCards(1)` in one
//!   resolution are two instructions and do not.
//!   → `one_instruction_of_two_is_a_different_event_from_two_instructions_of_one`
//! - **Alms Collector**, *"If an effect puts cards into a player's hand without
//!   using the word 'draw' at all, [it] doesn't apply."* → the same structural
//!   fact as Teferi's first ruling, asserted once.
//! - **Alms Collector**, *"If two players each control an Alms Collector and an
//!   effect instructs them to each draw two or more cards, the replacement
//!   effect of each ... is applied and both players end up drawing two cards."*
//!   → test. Two separate instructions, one per player, each meeting the other
//!   player's Collector and neither meeting its own controller's.
//!   → `two_alms_collectors_facing_each_other_both_draw_two`
//! - **Alms Collector**, *"If two players each control [one] and a third player
//!   would draw two or more cards, the third player chooses which ... will
//!   apply, and therefore which of the first two players draws a card."* → the
//!   four-player test, and the only three-player CR 616.1 prompt reachable from
//!   two printed cards.
//!   → `a_third_player_chooses_which_alms_collector_applies`
//! - **Notion Thief**, *"If an opponent is instructed to draw a card then
//!   discard a card, and Notion Thief causes you to draw a card instead, that
//!   opponent still discards a card. The same is true of any other actions that
//!   opponent is instructed to do."* → tested on the ruling's second sentence:
//!   `Primitive::Discard` is `NotImplemented` until RE-8, so the board is a
//!   draw-then-lose-life resolution (Night's Whisper's shape, unnamed) and the
//!   assertion is that the opponent still loses the life.
//!   → `the_opponents_other_instructions_still_happen`
//! - **Notion Thief**, *"If two or more players each control a Notion Thief ...
//!   that player chooses one ... Then the player whose Notion Thief's effect
//!   was chosen repeats this process among the remaining ... Each effect can be
//!   applied to the card draw only once this way."* → the three-player test,
//!   and every sentence of it falls out rather than being coded: "that player
//!   chooses" is CR 616.1's affected player, "the player whose ... was chosen
//!   repeats" is the same rule asked of the *new* subject, and "only once" is
//!   CR 614.5's applied set travelling with the lineage.
//!   → `three_notion_thieves_pass_the_draw_once_each_in_the_rulings_order`
//! - **Notion Thief**, *"[So] if each player in a two-player game controls a
//!   Notion Thief and one would draw a card, it really will be that player who
//!   draws a card."* → test, and it is the same mechanism observed from
//!   outside: the draw is handed across the table twice and comes home.
//!   → `two_notion_thieves_hand_the_draw_across_the_table_and_back`
//!
//! # What a random deck can draw
//!
//! Thought Reflection is the pooled card, and it is `{4}{U}{U}{U}`. Seven mana
//! is the most any pooled card has cost, so its `--require` reachability is
//! read and recorded rather than assumed. It opens two paths nothing else in
//! the pool does: the gather sweep on every draw *instruction* and every
//! individual draw while it is on the battlefield, and the first `Instead`
//! whose output is decomposed. Alms Collector is `{3}{W}` and stays out —
//! it applies only to an opponent's multi-card instruction, and the pool's
//! multi-draws are Ancestral Recall and Night's Whisper, so it would sit on the
//! battlefield doing nothing in most games while paying for a creature slot.
//! Teferi's Ageless Insight and Notion Thief stay out as the same shape with a
//! narrower pattern.
//! # RE-3 — life (CR 119.3, 119.7, 119.10, 120.3a's contained loss)
//!
//! **Six printed cards over three events, and the axis is which event an effect
//! watches.** CR 119.3's gain, the loss CR 120.3a contains inside damage, and a
//! gain that CR 101.2 refuses outright:
//!
//! | Card | Watches | Does |
//! |---|---|---|
//! | [`rhox_faithmender`] | a gain, yours | `Amount(Multiplier(2))` |
//! | [`alhammarrets_archive`] | a gain, yours — and a draw | the same, plus RE-2's draw doubler |
//! | [`tainted_remedy`] | a gain, an opponent's | `Instead(LoseLife { ReplacedAmount })` |
//! | [`words_of_worship`] | a draw, yours, once | `Instead(GainLife { Fixed(5) })` |
//! | [`ali_from_cairo`] | the loss inside damage, yours | `Amount(LifeFloor(1))` |
//! | [`skullcrack`] | — (it forbids) | two `Primitive::Restrict` rows |
//!
//! **Ali from Cairo is why the loss inside damage is its own event**, and its
//! own ruling is the only thing that says so: *"this effect does not prevent
//! damage, it prevents the damage from turning into loss of life."* So it is a
//! `LoseLife { cause: Some(Damage) }` and **not** a prevention effect —
//! `ReplacementDef::is_prevention` tests the pattern for damage first, and this
//! pattern is not damage, which is what keeps Skullcrack's second sentence from
//! switching it off. `is_prevention` is untouched by this phase.
//!
//! **Skullcrack's first sentence is a "can't", not a replacement** (CR 101.2,
//! 614.17), and CR 119.7 spells out what that costs the pipeline: *"a
//! replacement effect that would replace a life gain event affecting that
//! player won't do anything."* Both directions fall out of the order of the two
//! checks rather than being coded — a gain replacement finds no event, and a
//! replacement that *produces* a gain has its substitute refused, which is
//! Leyline of Punishment's ruling about Words of Worship.
//!
//! # The rulings pass (Scryfall, 2026-09-12)
//!
//! Every ruling on all six registered cards, with what became of it **and the
//! test that carries it** (`engineering-practices.md` §3.4). Eighteen rulings;
//! thirteen are tests in `tests/phase_re3_integration_test.rs`, two are already
//! tested elsewhere, and three have no facility to assert against and say so.
//!
//! - **Rhox Faithmender**, *"If you control two Rhox Faithmenders, life you
//!   gain will be multiplied by four. Three Rhox Faithmenders will multiply any
//!   life gain by eight, and so on."* → **the acid test**, and it failed as an
//!   unexpected *prompt* before it failed as a number: the CR 616.1 suppression
//!   premise was written about damage (§11 items 19, 55).
//!   → `test_two_rhox_faithmenders_quadruple`, with
//!   `three_rhox_faithmenders_multiply_by_eight` for the exponent and
//!   `four_is_not_two_applications_of_one_faithmender` for CR 614.6's single
//!   modified event.
//! - **Rhox Faithmender**, *"If an effect sets your life total to a specific
//!   number, and that number is higher than your current life total, the effect
//!   will cause you to gain life equal to the difference ... if you have 3 life
//!   and an effect says that your life total 'becomes 10,' your life total will
//!   actually become 17."* → **no facility**: CR 119.5's "set life total" is
//!   `Primitive::SetLifeTotal`, which RE-6 builds (§9's RE-6 row). Recorded, not
//!   skipped — and it is the same board as Skullcrack's fourth ruling and
//!   Leyline's fifth, so one primitive closes three.
//! - **Rhox Faithmender**, *"In a Two-Headed Giant game, only Rhox
//!   Faithmender's controller is affected by it."* → n/a: CR 810 is `backlog`'s,
//!   and the engine has no teams (CR 102.3, `PlayerSet`'s doc).
//! - **Tainted Remedy**, *"If more than one replacement effect tries to apply
//!   to a life gain event, the player who would gain life chooses the order ...
//!   that player may choose to have the 3 life become doubled to 6 life and
//!   then lose 6 life. The player may also choose to apply Tainted Remedy
//!   first, turning 'gain 3 life' into 'lose 3 life.' Alhammarret's Archive
//!   would then not apply."* → test, with the ruling's own numbers, and the
//!   board `ordering_cannot_change_outcome` must **not** suppress.
//!   → `the_gaining_player_chooses_between_the_archive_and_tainted_remedy`
//! - **Tainted Remedy**, *"Having more than one Tainted Remedy on the
//!   battlefield doesn't have any noticeable effect on life gain. Once the
//!   effect of one Tainted Remedy applies, there is no life gain for the others
//!   to apply to."* → test, and asserted the stronger way: CR 616.1 still asks
//!   which one (both are applicable at the first iteration), and both answers
//!   are the same three life.
//!   → `a_second_tainted_remedy_has_no_gain_left_to_apply_to`
//! - **Words of Worship**, *"If multiple Words have been used prior to drawing a
//!   card, then you can choose which one to apply (and use up) each time you
//!   draw a card."* → test. Two rows from one source, one prompt, and the other
//!   row still there for the next draw — which is what "(and use up)" means.
//!   → `two_words_rows_are_a_choice_and_each_draw_uses_one_up`
//! - **Ali from Cairo**, *"This effect does not apply to effects which reduce
//!   your life without doing damage."* → test: a `Primitive::LoseLife` is
//!   `LifeLossCause::Effect`, which the pattern does not match, and the player
//!   goes to -7. → `ali_from_cairo_does_not_clamp_a_loss_that_is_not_damage`,
//!   with `ali_from_cairo_does_not_clamp_a_life_payment` for the `Cost` arm
//!   nothing had watched.
//! - **Ali from Cairo**, *"The ability works up until Ali enters the graveyard,
//!   so if he takes lethal damage or is destroyed at the same time you take
//!   damage, the ability helps you."* → test, and it falls out of CR 704.3's
//!   decide-then-perform rather than being coded: a batch decides every member
//!   against one board. → `ali_helps_on_the_earthquake_that_kills_him`
//! - **Ali from Cairo**, *"This effect does not prevent damage, it prevents the
//!   damage from turning into loss of life. So the full damage is dealt (and
//!   abilities that trigger on damage being dealt still trigger), but the full
//!   loss of life is not applied."* → **the ruling that decided the card's
//!   pattern**, and two tests: the `DamageDealt` event still carries the whole
//!   amount, and Skullcrack does not switch the clamp off.
//!   → `the_full_damage_is_still_dealt_and_only_the_loss_is_clamped` and
//!   `skullcrack_does_not_turn_off_ali_from_cairo`
//! - **Alhammarret's Archive**, *"If an effect would set your life total to a
//!   specific number that's higher ... your life total will actually become
//!   17."* → RE-6's, as Rhox Faithmender's twin above.
//! - **Alhammarret's Archive**, *"If two or more replacement effects would apply
//!   to a card-drawing event, the player drawing the card chooses the order in
//!   which to apply them."* → already tested, in RE-2:
//!   `a_draw_doubler_beside_a_notion_thief_is_a_real_choice`.
//! - **Alhammarret's Archive**, *"Because [it] is legendary ... if that happens,
//!   life gained by that player will be multiplied by four."* → the legend rule
//!   makes this Rhox Faithmender's board, which is why the acid test is on that
//!   card. This card's own board is the one that shows the two halves coexist.
//!   → `alhammarrets_archive_doubles_a_gain_and_a_draw_from_one_permanent`
//! - **Alhammarret's Archive**, *"Similarly, the effects of the last abilities
//!   of multiple Archives are cumulative."* → already tested, in RE-2:
//!   `test_two_thought_reflections_draw_four_not_infinity`.
//! - **Alhammarret's Archive**, *"In a Two-Headed Giant game ..."* → n/a, as
//!   Rhox Faithmender's.
//! - **Skullcrack**, *"Skullcrack targets only the player or planeswalker. If
//!   that player or planeswalker is an illegal target when Skullcrack tries to
//!   resolve, it won't resolve and none of its effects will happen."* → CR
//!   608.2b's fizzle, which is the stack's and predates this phase; the card
//!   adds no new claim to it. Recorded rather than re-tested.
//! - **Skullcrack**, *"Spells and abilities that would cause a player to gain
//!   life or that would prevent damage still resolve, but the life-gain and
//!   damage-prevention parts have no effect."* → test, and "still resolve" is
//!   the half that could have been got wrong: a refused proposal is not an
//!   error. → `a_life_gain_spell_still_resolves_under_skullcrack_and_gains_nothing`,
//!   with `skullcrack_stops_life_gain_for_everyone_including_its_controller`
//!   for `PlayerSet::Everyone`.
//! - **Skullcrack**, *"Effects that would replace gaining life with another
//!   effect won't apply because it's impossible for players to gain life."* →
//!   CR 119.7's own last clause, and `ATOM-119.7-004`.
//!   → `under_skullcrack_a_gain_replacement_has_no_event_to_replace`
//! - **Skullcrack**, *"If an effect says to set a player's life total to a
//!   certain number and that number is higher than the player's current life
//!   total, that part of the effect won't do anything."* → RE-6's, as above.
//!
//! **Leyline of Punishment is not registered**, so its ten rulings are not this
//! phase's pass — but one of them is a test here, because it is about a card
//! that *is* registered: *"effects that replace an event with gaining life (like
//! Words of Worship's effect does) will end up replacing the event with
//! nothing."* → `words_of_worship_under_skullcrack_replaces_the_draw_with_nothing`.
//! Its sibling — *"if a cost includes life gain (like Invigorate's alternative
//! cost does), that cost can't be paid"* — is CR 119.7's cost clause and
//! `ATOM-119.7-003`, which needs the alternative-cost model and is Phase 8's.
//!
//! # What a random deck can draw
//!
//! Rhox Faithmender is the pooled card, and it is the first RE consumer that
//! needs no second card to set it up: Knight of Meadowgrain and Vampire
//! Nighthawk are already in the pool, so lifelink's contained `GainLife` —
//! proposed in every measured game since RB with nothing watching it — is a
//! live proposal the moment this is on the battlefield. At `{3}{W}` for a 1/5
//! with lifelink it also doubles the life its own combat damage gains, so the
//! board it opens is one card wide.
//!
//! The other five stay out. Tainted Remedy and Words of Worship are enchantments
//! whose whole effect is a replacement nothing in the pool would trigger often
//! enough to pay for a slot; Ali from Cairo is a 0/1 for four mana whose clamp
//! only matters on a board that is already lethal; Alhammarret's Archive is the
//! same two engine paths as Thought Reflection and this card at five mana; and
//! Skullcrack would put a CR 101.2 restriction row on every turn it is cast,
//! which is RS-1's path rather than a new one. **Leyline of Punishment is
//! deliberately unregistered** — its opening-hand clause is §3.3 source 2's
//! zone-reaching static, which would be dead text under a real card name, and
//! the static form of its other two sentences is the fixture in
//! `tests/phase_rd4_integration_test.rs`. **Bloodletter of Aclazotz is recorded
//! as a shape and not written**: "if an opponent would lose life during your
//! turn" is a conditional static whose condition — it is your turn — the
//! `Condition` AST has no leaf for, with one customer. `EventPattern::LoseLife`
//! and `PlayerSet::Opponents` are built here for it, and its own ruling is what
//! the pattern's `cause: None` is about: *"[it] doesn't change the amount of
//! damage dealt to opponents ... they would lose 2 life, but you'd still gain
//! only 1."*
//!

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::state::game_state::{PhaseType, StepType};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    AffectedSet, AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, PatternFill,
    PlayerRef, PlayerSet, Primitive, SelectionFilter, TargetCount,
};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::keywords::KeywordFlag;
use crate::types::replacement::{
    AmountRewrite, EventPattern, GameActionTemplate, LifeLossCausePattern, ReplacementDef,
    Rewrite, TemplateAmount,
};
use crate::types::restriction::{ReplacementKindFilter, Restriction, RestrictionDef};
use crate::types::zones::DrawCause;

/// A static ability whose effect is a replacement effect — never a resolution,
/// so it carries no `Duration` and is re-derived off the source's *effective*
/// ability list on every gather.
fn static_replacement(def: ReplacementDef) -> AbilityDef {
    AbilityDef {
        id: new_ability_id(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect: Effect::Replacement(Box::new(def)),
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
    }
}

/// One ability with no costs beyond the ones given.
fn one_shot(ability_type: AbilityType, costs: Vec<Cost>, effect: Effect) -> AbilityDef {
    AbilityDef {
        id: new_ability_id(),
        ability_type,
        costs,
        effect,
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
    }
}

/// "Skip [unit]" as CR 614.10 defines it: *"instead of doing [something], do
/// nothing"*, which CR 614.1b says is a replacement effect and CR 614.6 says is
/// a [`Rewrite::Prevent`].
///
/// No new `Rewrite` arm, for that reason — "replaced with nothing" **is** 614.6,
/// and a `Rewrite::Skip` would be a second spelling of one algebra element
/// (`replacement-architecture.md` §9, RE decision 0).
///
/// The object set is empty on every one of these: CR 614.10's three units are
/// about a *player*, so the scope is a [`PlayerSet`] and nothing else.
fn skip(pattern: EventPattern, players: PlayerSet) -> ReplacementDef {
    ReplacementDef::new(pattern, AffectedSet::NO_OBJECTS, Rewrite::Prevent)
        .affecting_players(players)
}

/// Yawgmoth's Bargain — {4}{B}{B}
/// Enchantment
///
/// > Skip your draw step.
/// > Pay 1 life: Draw a card.
///
/// The plainest static skip there is, and the one that shows what a skipped
/// step costs: not the draw alone but the whole step — no `StepBegin`, no
/// turn-based action, no priority round (`ATOM-614.10-001`). Its second
/// ability is here because a card that only subtracted would never be cast by
/// anything, and both halves — `Cost::PayLife` and `Primitive::DrawCards` —
/// already exist.
///
/// Scryfall lists no rulings (2026-09-11).
///
/// **Registered and not pooled.** A random agent with one use for its life
/// total will empty its library, which is the board RE-6's Laboratory Maniac
/// path wants and a distortion of every fixture until it lands.
pub fn yawgmoths_bargain() -> Arc<CardData> {
    CardDataBuilder::new("Yawgmoth's Bargain")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black], 4))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("Skip your draw step.\nPay 1 life: Draw a card.")
        .ability(static_replacement(skip(
            EventPattern::BeginStep { step: Some(StepType::Draw) },
            PlayerSet::You,
        )))
        .ability(one_shot(
            AbilityType::Activated,
            vec![Cost::PayLife(1)],
            Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller),
        ))
        .build()
}

/// Eon Hub — {5}
/// Artifact
///
/// > Players skip their upkeep steps.
///
/// The static whose scope is **everyone**, which is what a per-player counter
/// could never have expressed: the row is one effect that applies to each
/// player's upkeep in turn, gathered off the artifact's *effective* ability
/// list, so Humility or CR 305.7 taking the ability away takes the skip away
/// with it. CR 616.1's chooser is the affected player — the one whose upkeep it
/// is — so on a four-player table this asks nobody, four times a round.
///
/// Its rulings are in this module's doc comment; two of the three are item 6's.
///
/// **The pooled card of the PR.** Colourless at five, so every deck can cast
/// it, and every player's upkeep for the rest of the game is then a proposal
/// that goes nowhere — the first measured card whose cost is a *dropped*
/// turn-structure event.
pub fn eon_hub() -> Arc<CardData> {
    CardDataBuilder::new("Eon Hub")
        .mana_cost(ManaCost::build(&[], 5))
        .card_type(CardType::Artifact)
        .rules_text("Players skip their upkeep steps.")
        .ability(static_replacement(skip(
            EventPattern::BeginStep { step: Some(StepType::Upkeep) },
            PlayerSet::Everyone,
        )))
        .build()
}

/// Meditate — {2}{U}
/// Instant
///
/// > Draw four cards. You skip your next turn.
///
/// The **consumable** skip, and the first row in the crate whose duration is
/// `Indefinite` and whose end is a use: "your next turn" is not a length of
/// time, so nothing but `Uses::Once` can retire it. CR 614.10a's second
/// sentence is the board two of these build — *"if two effects each cause a
/// player to skip their next occurrence, that player must skip the next
/// two"* — and it is the test that could not be written against a per-player
/// counter, which has no way to be offered to CR 616.1 twice.
///
/// Ruling (2026-09-11): *"You skip one turn as part of the effect."* → one row,
/// one turn.
pub fn meditate() -> Arc<CardData> {
    CardDataBuilder::new("Meditate")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text("Draw four cards. You skip your next turn.")
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::DrawCards(AmountExpr::Fixed(4)),
                    EffectRecipient::Controller,
                ),
                Effect::Atom(
                    // `PlayerSet::You` and a `Controller` recipient: the row is
                    // the def as authored, and CR 109.5 resolves "you" against
                    // the row's controller at each event rather than at
                    // resolution.
                    Primitive::CreateReplacement(
                        Box::new(skip(EventPattern::BeginTurn, PlayerSet::You).once()),
                        Duration::Indefinite,
                        PatternFill::Authored,
                    ),
                    EffectRecipient::Controller,
                ),
            ]),
        ))
        .build()
}

/// Time Walk — {1}{U}
/// Sorcery
///
/// > Take an extra turn after this one.
///
/// The turn queue's only producer, and the card that makes a skip's "next
/// occurrence" mean something: with Meditate's row already in the registry the
/// extra turn is the occurrence that gets skipped, and the natural turn after
/// it begins (CR 614.10a).
///
/// Ruling (2026-09-11): *"If multiple 'extra turn' effects resolve in the same
/// turn, take them in the reverse of the order that the effects resolved."* →
/// CR 500.7's "most recently created turn will be taken first", which is the
/// queue being a stack.
///
/// **Registered and not pooled**: an extra turn in every blue deck moves
/// `Avg turns/game` by design, which is a worse baseline rather than a wider
/// one.
pub fn time_walk() -> Arc<CardData> {
    CardDataBuilder::new("Time Walk")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Sorcery)
        .rules_text("Take an extra turn after this one.")
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Atom(Primitive::ExtraTurn, EffectRecipient::Controller),
        ))
        .build()
}

/// Moment of Silence — {W}
/// Instant
///
/// > Target player skips their next combat phase this turn.
///
/// The **targeted** skip and the only phase-scoped one here. Its three rulings
/// are all consequences of the shape rather than special cases: `Uses::Once`
/// makes it one phase, `Duration::UntilEndOfTurn` makes a row that meets no
/// proposal expire unused, and the event's subject being the *active* player
/// makes a row on anybody else watch nothing.
///
/// The affected set is authored empty in both halves, which
/// `Primitive::CreateReplacement`'s `Target` arm requires: the shape is the
/// card's and the player is the resolution's.
pub fn moment_of_silence() -> Arc<CardData> {
    CardDataBuilder::new("Moment of Silence")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text("Target player skips their next combat phase this turn.")
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Atom(
                Primitive::CreateReplacement(
                    Box::new(
                        skip(
                            EventPattern::BeginPhase { phase: Some(PhaseType::Combat) },
                            PlayerSet::Nobody,
                        )
                        .once(),
                    ),
                    Duration::UntilEndOfTurn,
                    PatternFill::Authored,
                ),
                EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
            ),
        ))
        .build()
}

// ---------------------------------------------------------------------------
// RE-2 — draw
// ---------------------------------------------------------------------------

/// "If you would draw a card, draw two cards instead" — the shape Thought
/// Reflection and Teferi's Ageless Insight share, differing only in the
/// [`DrawCause`] they except.
///
/// [`GameActionTemplate::DrawCards`]'s `player` is `None` on both: the draw
/// stays with the player who would have drawn it. Notion Thief is the arm's
/// other customer and the one that moves it.
fn draw_two_instead(cause: Option<DrawCause>) -> ReplacementDef {
    ReplacementDef::new(
        EventPattern::DrawCard { cause },
        AffectedSet::NO_OBJECTS,
        Rewrite::Instead(GameActionTemplate::DrawCards { n: 2, player: None }),
    )
    .affecting_players(PlayerSet::You)
}

/// Thought Reflection — {4}{U}{U}{U}
/// Enchantment
///
/// > If you would draw a card, draw two cards instead.
///
/// **The acid test's card.** It is not legendary, so a board can hold two, and
/// its ruling gives the arithmetic verbatim: two draw four times the original
/// number, three draw eight. CR 616.1 asks nothing between them — the order
/// provably cannot change the total, which is `ordering_cannot_change_outcome`'s
/// third shape (§11 item 55). That 2ⁿ is what §3.2d's lineage rule buys — each
/// doubled draw inherits the applied set of the draw it came from, so a
/// Reflection that has applied cannot apply to its own output. Without the
/// inheritance the game does not answer wrongly; it hangs.
///
/// Its rulings are in this module's doc comment.
///
/// **The pooled card of the PR**, at seven mana, which is the most any pooled
/// card has cost — so its reachability is measured with `--require` rather than
/// assumed.
pub fn thought_reflection() -> Arc<CardData> {
    CardDataBuilder::new("Thought Reflection")
        .mana_cost(ManaCost::build(
            &[ManaType::Blue, ManaType::Blue, ManaType::Blue],
            4,
        ))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text("If you would draw a card, draw two cards instead.")
        .ability(static_replacement(draw_two_instead(None)))
        .build()
}

/// Teferi's Ageless Insight — {2}{U}{U}
/// Legendary Enchantment
///
/// > If you would draw a card except the first one you draw in each of your
/// > draw steps, draw two cards instead.
///
/// **The card [`DrawCause`] exists for**, and the one that shows the stamping
/// rule doing work: `Some(DrawCause::Effect)` watches every draw but the first
/// of a draw step, and "the first" is not a count kept anywhere — it is the
/// first inner of the draw step's instruction, with every later inner stamped
/// `Effect` at every level of decomposition. So beside a Thought Reflection
/// this draws **three** in the draw step: the Reflection doubles the
/// instruction's one draw, the doubled instruction keeps its `TurnBased` cause
/// (CR 614.6), its first inner is the card this excepts, and its second is the
/// card this doubles.
///
/// Legendary, so the two-copy board is Thought Reflection's.
pub fn teferis_ageless_insight() -> Arc<CardData> {
    CardDataBuilder::new("Teferi's Ageless Insight")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .supertype(Supertype::Legendary)
        .rules_text(
            "If you would draw a card except the first one you draw in each of your draw steps, draw two cards instead.",
        )
        .ability(static_replacement(draw_two_instead(Some(DrawCause::Effect))))
        .build()
}

/// Alms Collector — {3}{W}
/// Creature — Cat Cleric 3/3
///
/// > Flash
/// > If an opponent would draw two or more cards, instead you and that player
/// > each draw a card.
///
/// **The only printed customer for [`EventPattern::DrawCards`]**, and the card
/// that makes the instruction event necessary rather than tidy: its own ruling
/// says to count how many times the word "draw" is used, so "draw two cards" is
/// one event this watches and two cantrips are two events it does not.
///
/// **Only half of it is a rider, and the other half is the modified event.**
/// §9 filed the whole of "you and that player each draw a card" under `then` on
/// §3.2d's heterogeneous rule, and its own second ruling refuses that: *"once
/// Alms Collector's replacement effect has modified the effect of a player's
/// Divination, Thought Reflection can double that player's resulting card draw
/// **without Alms Collector's replacement effect applying again**."* CR 614.5
/// gives an effect one opportunity to affect "an event **or any modified events
/// that may replace that event**", and the affected player's one draw is such a
/// modified event — so it has to carry this effect's applied set, which only the
/// rewrite's own output does. As `Prevent` plus two riders it does not, and the
/// board is an infinite loop rather than a wrong number: the rider's draw is
/// doubled back to two, this applies again, and the two effects trade cards
/// until the game is a draw (CR 104.4b) or the engine's stack runs out.
///
/// So the split follows §3.2d's rule read one clause further in. The affected
/// player's half is the **same event with a smaller count** — homogeneous
/// multiplicity, which is a count field — and only the controller's draw is a
/// genuinely new subject, which is what `then` is for. One rider, not two.
///
/// **Flash is not modelled** (`codebase-state.md`'s timing item), and it costs
/// this card's tests nothing: every board here puts it on the battlefield
/// before the draw, which is the only state its replacement reads.
///
/// The two draws come out affected-player-first, because a rider resolves after
/// the event it rides on (§4.1a) — neither the card's text order nor CR 121.2c's
/// turn order, which `codebase-state.md` item 122 owns and sizes.
pub fn alms_collector() -> Arc<CardData> {
    CardDataBuilder::new("Alms Collector")
        .mana_cost(ManaCost::build(&[ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Cat))
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(3, 3)
        .rules_text(
            "Flash\nIf an opponent would draw two or more cards, instead you and that player each draw a card.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DrawCards { at_least: Some(2) },
                AffectedSet::NO_OBJECTS,
                // "That player draws a card": the same instruction with `n`
                // rewritten to 1, so it keeps this effect's applied set and
                // whatever doubles it afterwards cannot hand it back.
                Rewrite::Instead(GameActionTemplate::DrawCards { n: 1, player: None }),
            )
            .affecting_players(PlayerSet::Opponents)
            // "And you draw a card": the half that is a different player's
            // draw, which nothing about the replaced event can carry.
            .with_then(Effect::Atom(
                Primitive::DrawCards(AmountExpr::Fixed(1)),
                EffectRecipient::Controller,
            )),
        ))
        .build()
}

/// Notion Thief — {2}{U}{B}
/// Creature — Human Rogue 3/1
///
/// > Flash
/// > If an opponent would draw a card except the first one they draw in each of
/// > their draw steps, instead that player skips that draw and you draw a card.
///
/// **The same event with a new subject**, and its own ruling is the only reason
/// to know that. Read as English it is Alms Collector's shape — a skip and a
/// draw joined by "and" — and §3.2d filed it as `Prevent` plus a rider until
/// the rulings pass. The ruling walks two Thieves, says each is *"applied to
/// the card draw only once"*, and concludes that in a two-player game *"it
/// really will be that player who draws a card"*. A rider's draw is a fresh
/// proposal with a fresh applied set, so two Thieves as riders would hand the
/// draw back and forth forever. As `Instead(DrawCards { n: 1, player: You })`
/// the draw keeps its lineage: each Thief applies once, and the draw comes home.
///
/// `n: 1` rather than a `DrawCard`, because the substitute for a draw is always
/// the instruction (CR 121.2a) — which also means the Thief's own draw is
/// `DrawCause::Effect` and a second Thief can take it.
pub fn notion_thief() -> Arc<CardData> {
    CardDataBuilder::new("Notion Thief")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Black], 2))
        .color(Color::Blue)
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Rogue))
        .power_toughness(3, 1)
        .rules_text(
            "Flash\nIf an opponent would draw a card except the first one they draw in each of their draw steps, instead that player skips that draw and you draw a card.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DrawCard { cause: Some(DrawCause::Effect) },
                AffectedSet::NO_OBJECTS,
                Rewrite::Instead(GameActionTemplate::DrawCards {
                    n: 1,
                    player: Some(PlayerRef::You),
                }),
            )
            .affecting_players(PlayerSet::Opponents),
        ))
        .build()
}

// ---------------------------------------------------------------------------
// RE-3 — life
// ---------------------------------------------------------------------------

/// Rhox Faithmender — {3}{W}
/// Creature — Rhino Monk 1/5
///
/// > Lifelink
/// > If you would gain life, you gain twice that much life instead.
///
/// **The first consumer in Phase RE that meets a live proposal without a
/// fixture.** Lifelink's contained `GainLife` has been proposed in every
/// measured game since RB, with nothing watching it; this card is on the
/// battlefield with lifelink of its own, so it doubles the life its own combat
/// damage gains.
///
/// Its rulings are in this module's doc comment.
pub fn rhox_faithmender() -> Arc<CardData> {
    CardDataBuilder::new("Rhox Faithmender")
        .mana_cost(ManaCost::build(&[ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Rhino))
        .subtype(Subtype::Creature(CreatureType::Monk))
        .power_toughness(1, 5)
        .keyword_flag(KeywordFlag::Lifelink)
        .rules_text("Lifelink\nIf you would gain life, you gain twice that much life instead.")
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::GainLife,
                AffectedSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Tainted Remedy — {2}{B}
/// Enchantment
///
/// > If an opponent would gain life, that player loses that much life instead.
///
/// **The kind-changing substitution, and the first customer of
/// [`TemplateAmount::ReplacedAmount`].** "That much" is the gain's own number
/// read at the moment this applies, which is what makes its ordering ruling
/// arithmetic rather than a coin flip: beside Alhammarret's Archive the gaining
/// player picks double-then-lose-6, or lose-3-then-nothing.
///
/// Its rulings are in this module's doc comment.
///
/// Four-player: `PlayerSet::Opponents` is three opponents against one static
/// row, which is the shape CR 109.5 resolves per event rather than per
/// registration.
pub fn tainted_remedy() -> Arc<CardData> {
    CardDataBuilder::new("Tainted Remedy")
        .mana_cost(ManaCost::build(&[ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("If an opponent would gain life, that player loses that much life instead.")
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::GainLife,
                AffectedSet::NO_OBJECTS,
                Rewrite::Instead(GameActionTemplate::LoseLife {
                    amount: TemplateAmount::ReplacedAmount,
                }),
            )
            .affecting_players(PlayerSet::Opponents),
        ))
        .build()
}

/// Words of Worship — {2}{W}
/// Enchantment
///
/// > {1}: The next time you would draw a card this turn, you gain 5 life
/// > instead.
///
/// **A draw replaced by life — RE-2's pattern and RE-3's template on one
/// row**, and the first `Uses::Once` draw replacement in the crate. It is a
/// resolution's row rather than a static ability, so `Duration::UntilEndOfTurn`
/// is the card's "this turn" and `Uses::Once` is its "the next time"; neither
/// is derived, for CR 608.2c's reason.
///
/// [`EventPattern::DrawCard`] with `cause: None` — "the next time you would
/// draw a card" excepts nothing, so the draw step's own draw is a candidate.
///
/// Its rulings are in this module's doc comment. **Leyline of Punishment's
/// ruling about this card is the CR 101.2 ordering test**: under a "players
/// can't gain life", the substituted gain is proposed, refused, and the draw
/// has been replaced with nothing.
pub fn words_of_worship() -> Arc<CardData> {
    CardDataBuilder::new("Words of Worship")
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("{1}: The next time you would draw a card this turn, you gain 5 life instead.")
        .ability(one_shot(
            AbilityType::Activated,
            vec![Cost::Mana(ManaCost::build(&[], 1))],
            Effect::Atom(
                Primitive::CreateReplacement(
                    Box::new(
                        ReplacementDef::new(
                            EventPattern::DrawCard { cause: None },
                            AffectedSet::NO_OBJECTS,
                            Rewrite::Instead(GameActionTemplate::GainLife {
                                amount: TemplateAmount::Fixed(5),
                            }),
                        )
                        .affecting_players(PlayerSet::You)
                        .once(),
                    ),
                    Duration::UntilEndOfTurn,
                    PatternFill::Authored,
                ),
                EffectRecipient::Controller,
            ),
        ))
        .build()
}

/// Ali from Cairo — {2}{R}{R}
/// Creature — Human 0/1
///
/// > Damage that would reduce your life total to less than 1 reduces it to 1
/// > instead.
///
/// **It watches the loss, not the damage**, and its own ruling is the only
/// reason to know that: *"this effect does not prevent damage, it prevents the
/// damage from turning into loss of life. So the full damage is dealt (and
/// abilities that trigger on damage being dealt still trigger), but the full
/// loss of life is not applied."* CR 120.3a's contained `LoseLife` is that
/// loss, built in RD-1 for this card.
///
/// So the def is `LoseLife { cause: Some(Damage) }` and **not** a prevention
/// effect: `is_prevention` tests the pattern for damage first, and this pattern
/// is not damage, which is why Skullcrack's "damage can't be prevented" does
/// not switch it off.
///
/// `cause: Some(Damage)` is also the whole answer to whether it clamps a life
/// *payment*. It does not, twice over: CR 119.4 refuses a payment larger than
/// the life total before any replacement is asked, and a payment's cause is
/// [`LifeLossCause::Cost`], which this pattern does not match. The card agrees
/// — "damage that would reduce" — and so does its first ruling, *"this effect
/// does not apply to effects which reduce your life without doing damage."*
///
/// Its rulings are in this module's doc comment.
pub fn ali_from_cairo() -> Arc<CardData> {
    CardDataBuilder::new("Ali from Cairo")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .power_toughness(0, 1)
        .rules_text("Damage that would reduce your life total to less than 1 reduces it to 1 instead.")
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::LoseLife { cause: Some(LifeLossCausePattern::Damage) },
                AffectedSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::LifeFloor(1)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Alhammarret's Archive — {5}
/// Legendary Artifact
///
/// > If you would gain life, you gain twice that much life instead.
/// > If you would draw a card except the first one you draw in each of your
/// > draw steps, draw two cards instead.
///
/// **Two statics on one permanent, one from each of the two RE phases** —
/// Rhox Faithmender's gain doubler and Teferi's Ageless Insight's draw doubler,
/// written as the same two defs because they *are* the same two defs. Gisela's
/// shape, and the reason RE-2 → RE-3 is a hard order in §9.
///
/// Its rulings are Rhox Faithmender's and Teferi's, already tests; the module
/// doc says which.
pub fn alhammarrets_archive() -> Arc<CardData> {
    CardDataBuilder::new("Alhammarret's Archive")
        .mana_cost(ManaCost::build(&[], 5))
        .card_type(CardType::Artifact)
        .supertype(Supertype::Legendary)
        .rules_text(
            "If you would gain life, you gain twice that much life instead.\nIf you would draw a card except the first one you draw in each of your draw steps, draw two cards instead.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::GainLife,
                AffectedSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .ability(static_replacement(draw_two_instead(Some(DrawCause::Effect))))
        .build()
}

/// Skullcrack — {1}{R}
/// Instant
///
/// > Players can't gain life this turn. Damage can't be prevented this turn.
/// > Skullcrack deals 3 damage to target player or planeswalker.
///
/// **The card that lands RD-4's restriction row in a game** (§11 item 26). Its
/// second sentence is the `ApplyReplacement { Prevention }` row RD-4 could only
/// build as a fixture, because every printed carrier of it needed a facility
/// the engine lacked; its first is the `Event { GainLife }` row RE-3 gave
/// `Restriction::Event` the player set for (item 45).
///
/// **Three atoms in text order, and the order is the card's**: CR 608.2c reads
/// a spell's instructions in the order printed, and both restrictions are in
/// place before the damage is dealt — which is what makes a lifelinker's damage
/// gain nothing this turn.
///
/// Its rulings are in this module's doc comment.
///
/// **Leyline of Punishment is deliberately not registered.** It is the static
/// form of the same two rows — an `Effect::Restriction` on a permanent, which
/// RS-1's sweep already reads — and the RD-4 fixture extended with the life arm
/// is what proves that form. What keeps it out is its first line: "if this card
/// is in your opening hand, you may begin the game with it on the battlefield"
/// is §3.3 source 2's zone-reaching static, which would be dead text under a
/// real card name. Recorded so the omission reads as the rule and not as an
/// oversight.
pub fn skullcrack() -> Arc<CardData> {
    CardDataBuilder::new("Skullcrack")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .rules_text(
            "Players can't gain life this turn. Damage can't be prevented this turn. Skullcrack deals 3 damage to target player or planeswalker.",
        )
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Sequence(vec![
                // "Players can't gain life this turn." No object and every
                // player, so the row is complete as authored and the
                // resolution's target — the player it then damages — is not
                // what it is about.
                Effect::Atom(
                    Primitive::Restrict(
                        RestrictionDef::new(Restriction::Event {
                            pattern: EventPattern::GainLife,
                            affected_objects: AffectedSet::NO_OBJECTS,
                            affected_players: PlayerSet::Everyone,
                            by: None,
                        }),
                        Duration::UntilEndOfTurn,
                    ),
                    EffectRecipient::Controller,
                ),
                // "Damage can't be prevented this turn." CR 615.12's row, whose
                // two halves are a union: every object and every player.
                Effect::Atom(
                    Primitive::Restrict(
                        RestrictionDef::new(Restriction::ApplyReplacement {
                            kind: ReplacementKindFilter::Prevention,
                            to_objects: AffectedSet::Filter { filter: ObjectFilter::All },
                            to_players: PlayerSet::Everyone,
                        }),
                        Duration::UntilEndOfTurn,
                    ),
                    EffectRecipient::Controller,
                ),
                Effect::Atom(
                    Primitive::DealDamage { amount: AmountExpr::Fixed(3), unpreventable: false },
                    EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
                ),
            ]),
        ))
        .build()
}
