//! Phase RE-1 — skips, and the turn queue.
//!
//! CR 614.1b, 614.10, 614.10a, 500.7, 500.11 and 508.8, against the five
//! printed cards in `cards::phase_re_cards`.
//!
//! **Every board here is a turn sequence**, because that is the only thing a
//! skip is observable in: a step that is skipped emits nothing, runs no
//! turn-based action and grants no priority, so the assertion is always about
//! what the event log does *not* contain and where the drainer landed instead.
//!
//! `ATOM-614.10b-001` — "skip …, then take another action" — has **no test and
//! no card**. The corpus's own audit note says the Scryfall regex
//! `o:/skip.*then/` returns empty, and RE's census (2026-09-11) confirmed it at
//! zero: there is nothing to write the follow-up action onto, and a fixture
//! wearing the rule would assert the engine's guess rather than a printed
//! card's behaviour. Recorded here rather than annotated anywhere
//! (`replacement-architecture.md` §9, RE decision 6).
//!
//! CR 500.7 likewise has **no atom in the corpus** — `backlog.md` §2.17 said it
//! was thin and RE-1's sizing confirmed it — so
//! [`two_extra_turns_are_taken_most_recently_created_first`] claims none and
//! tests the rule's own sentence instead.

use std::sync::Arc;

use mtgsim::cards::phase_re_cards::{
    eon_hub, meditate, moment_of_silence, time_walk, yawgmoths_bargain,
};
use mtgsim::engine::actions::ActionContext;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, CardData};
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::{
    fill_library, place_vanilla_creature, put_in_hand, put_on_battlefield, setup_game,
    setup_two_player_game, test_ctx, test_dp,
};
use mtgsim::types::effects::{AmountExpr, Duration, Effect, EffectRecipient, Primitive};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve `card`'s spell effect for `controller` against `targets`, the way
/// the stack would; the card's own id is the resolution's source.
fn resolve_spell(
    game: &mut GameState,
    card: Arc<CardData>,
    controller: PlayerId,
    targets: Vec<ResolvedTarget>,
) -> ObjectId {
    let id = put_in_hand(game, card.clone(), controller);
    let ctx = ResolutionContext {
        source: id,
        ability_source: None,
        controller,
        targets,
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
    id
}

/// Walk the turn machinery `steps` times with `dp` answering.
fn advance(game: &mut GameState, dp: &dyn DecisionProvider, steps: usize) {
    let ctx = ActionContext::new(dp);
    for _ in 0..steps {
        game.advance_turn(&ctx).expect("advancing");
    }
}

/// Walk until a turn **begins**, and report which one.
///
/// Counted by the event rather than by positions, which is the point of the
/// whole PR: a skipped turn produces no event and no position, so a test that
/// advanced a fixed number of steps would be asserting the step arithmetic of
/// whatever it happened to skip.
fn to_next_turn(game: &mut GameState, dp: &dyn DecisionProvider) -> (PlayerId, u32) {
    let ctx = ActionContext::new(dp);
    let before = game.events.len();
    for _ in 0..200 {
        game.advance_turn(&ctx).expect("advancing");
        if let Some(begun) = game.events.records_from(before).iter().find_map(|r| match &r.event {
            GameEvent::TurnBegin { player, turn_number } => Some((*player, *turn_number)),
            _ => None,
        }) {
            return begun;
        }
    }
    panic!("no turn began within 200 positions");
}

/// The next `n` turns that begin, in order.
fn next_turns(game: &mut GameState, dp: &dyn DecisionProvider, n: usize) -> Vec<(PlayerId, u32)> {
    (0..n).map(|_| to_next_turn(game, dp)).collect()
}

/// Every turn that actually began, in order, as `(player, turn number)`.
fn turns_begun(game: &GameState) -> Vec<(PlayerId, u32)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::TurnBegin { player, turn_number } => Some((*player, *turn_number)),
            _ => None,
        })
        .collect()
}

/// Every step that actually began, in order.
fn steps_begun(game: &GameState) -> Vec<StepType> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::StepBegin { step } => Some(*step),
            _ => None,
        })
        .collect()
}

/// Every phase that actually began, in order.
fn phases_begun(game: &GameState) -> Vec<PhaseType> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::PhaseBegin { phase } => Some(*phase),
            _ => None,
        })
        .collect()
}

fn cards_drawn(game: &GameState) -> usize {
    game.events
        .events()
        .filter(|e| matches!(e, GameEvent::CardDrawn { .. }))
        .count()
}

/// A board mid-turn-1: player 0 active, everyone stocked so a draw step does
/// not deck anyone, and the position parked at the ending phase's End step so
/// the very next `advance_turn` reaches the turn boundary.
fn at_the_turn_boundary(num_players: usize) -> GameState {
    let mut game = setup_game(num_players);
    for pid in 0..num_players {
        fill_library(&mut game, pid, 60);
    }
    game.phase = Phase { phase_type: PhaseType::Ending, step: Some(StepType::End) };
    game
}

// ---------------------------------------------------------------------------
// CR 614.10a — two skip effects consume two occurrences
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-614.10a-001
//
// The atom's board is two "skip your next draw step" effects; every printed
// draw-step skip is a **static** (Yawgmoth's Bargain, Necropotence), which is
// `Uses::Static` and can never be consumed, so the rule's "one effect will be
// satisfied in skipping the first occurrence, while the other will remain"
// cannot be built on the draw step from the printed pool at all. Two Meditates
// are the same sentence on the unit that does print consumably, and what this
// proves is the rule and not the atom's step-shaped instance.
#[test]
fn two_meditates_make_a_player_skip_their_next_two_turns() {
    let mut game = at_the_turn_boundary(2);
    resolve_spell(&mut game, meditate(), 0, vec![]);
    resolve_spell(&mut game, meditate(), 0, vec![]);
    assert_eq!(game.replacement_effects.len(), 2, "one row per Meditate");

    // Two applicable rows on one event is a real CR 616.1 choice, and the
    // affected player makes it — here player 0, whose turn is being skipped.
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![0]);

    // Player 0's next two turns do not happen and advance no turn number
    // (CR 614.10a, 500.11): player 1 takes turns 2, 3 and 4 in a row, and
    // player 0's next turn is number 5.
    assert_eq!(
        next_turns(&mut game, &dp, 4),
        vec![(1, 2), (1, 3), (1, 4), (0, 5)],
        "two rows, two skipped turns, then a turn that happens"
    );
    assert!(game.replacement_effects.is_empty(), "both rows spent, one per occurrence");
}

// ---------------------------------------------------------------------------
// CR 614.10a + CR 500.7 — a skip consumes the extra turn
// ---------------------------------------------------------------------------

/// §9's board for putting the skips and the queue in one PR: the extra turn is
/// the "next occurrence" the skip consumes, and the natural turn after it
/// begins.
#[test]
fn a_skip_consumes_the_extra_turn_and_the_natural_turn_still_arrives() {
    let mut game = at_the_turn_boundary(2);
    resolve_spell(&mut game, meditate(), 0, vec![]);
    resolve_spell(&mut game, time_walk(), 0, vec![]);
    assert_eq!(game.turn_queue.len(), 1, "Time Walk queued one extra turn");

    // The extra turn is proposed first (CR 500.7's "directly after the
    // specified turn") and Meditate's row replaces it with nothing. Player 1's
    // natural turn follows, and player 0's own natural turn after that — the
    // extra turn was skipped, not the natural one.
    assert_eq!(next_turns(&mut game, &test_dp(), 2), vec![(1, 2), (0, 3)]);
    assert!(game.turn_queue.is_empty(), "a skipped extra turn is spent on being skipped");
    assert!(game.replacement_effects.is_empty(), "and so is the row that skipped it");
}

/// The same board without the skip, so the reader can see which half each card
/// is doing.
#[test]
fn an_extra_turn_is_taken_by_its_controller_before_the_rotation_resumes() {
    let mut game = at_the_turn_boundary(2);
    resolve_spell(&mut game, time_walk(), 0, vec![]);

    assert_eq!(
        next_turns(&mut game, &test_dp(), 3),
        vec![(0, 2), (1, 3), (0, 4)]
    );
}

/// CR 500.7's ordering sentence, on the only board that makes it observable:
/// two *different* players' extra turns, created in one turn.
///
/// **No atom** — CR 500's corpus entries cover 500.1–500.5 and 500.7 has none
/// (`backlog.md` §2.17 said so; RE-1's sizing confirmed it), so this claims
/// nothing and tests the rule's own words.
#[test]
fn two_extra_turns_are_taken_most_recently_created_first() {
    let mut game = at_the_turn_boundary(4);
    resolve_spell(&mut game, time_walk(), 1, vec![]);
    resolve_spell(&mut game, time_walk(), 2, vec![]);

    // Player 2's Time Walk resolved second, so player 2's extra turn is taken
    // first. Then player 1's. Then the natural rotation resumes from player 0,
    // whose turn the two extra ones were added after — so player 1, not
    // player 3.
    assert_eq!(
        next_turns(&mut game, &test_dp(), 3),
        vec![(2, 2), (1, 3), (1, 4)]
    );
}

/// **Timesifter's own bookkeeping, which is what `turn_rotation` is.**
///
/// > *Remember which player would have taken the next turn if Timesifter's
/// > ability hadn't triggered the first time. After Timesifter leaves the
/// > battlefield and all extra turns have been taken, that player takes the
/// > next turn.* (Scryfall, fetched 2026-09-11.)
///
/// Timesifter needs item 6's triggers to be registered, so the board is built
/// from `Primitive::ExtraTurn` directly — but the shape is its, and it is the
/// one the card is infamous for: *"With two Timesifters on the battlefield, two
/// extra turns are created for each turn taken"*, in a four-player game, which
/// is a queue that never empties. Two things are asserted and neither is
/// reachable on two players: extra turns drain **most recently created first**
/// across several players (CR 500.7), and when the queue finally empties the
/// **natural** rotation resumes at the player it had reached — not at whoever
/// took the last extra turn.
#[test]
fn a_deep_queue_drains_most_recent_first_and_leaves_the_rotation_where_it_was() {
    let mut game = at_the_turn_boundary(4);

    // Eight extra turns, created during player 0's turn, two per player, in
    // the order a pair of Timesifters would hand them out.
    for _ in 0..2 {
        for player in 0..4 {
            resolve_spell(&mut game, time_walk(), player, vec![]);
        }
    }
    assert_eq!(game.turn_queue.len(), 8);

    // A stack: the last created is the first taken, so the eight come back in
    // reverse — P3 P2 P1 P0, twice.
    let extra = next_turns(&mut game, &test_dp(), 8);
    assert_eq!(
        extra,
        vec![(3, 2), (2, 3), (1, 4), (0, 5), (3, 6), (2, 7), (1, 8), (0, 9)],
        "CR 500.7 — the most recently created turn is taken first"
    );
    assert!(game.turn_queue.is_empty());

    // And now the ruling's last sentence: the rotation is still at player 0,
    // whose natural turn the eight were added after, so the next natural turn
    // is player 1's and the three after it are one per player in order —
    // eight extra turns moved the rotation not at all.
    //
    // Player 0 also happens to have taken the *last* extra turn, so this one
    // board cannot tell `turn_rotation` from `active_player`.
    // `an_extra_turn_on_someone_elses_turn_does_not_consume_the_taker_s_own`
    // is the board that can.
    assert_eq!(
        next_turns(&mut game, &test_dp(), 3),
        vec![(1, 10), (2, 11), (3, 12)],
        "the natural rotation resumes where it left off, once per player"
    );
}

/// The distinguishing half of the board above: an extra turn for somebody who
/// is **not** the player whose natural turn it is.
///
/// Final Fortune is the printed card — an *instant*, so it resolves on another
/// player's turn — and this is the case `(active_player + 1) % n` gets wrong.
/// Player 1 takes an extra turn during player 0's, and then still takes their
/// own natural turn, because CR 500.7 inserts the extra turn after player 0's
/// rather than in place of player 1's.
#[test]
fn an_extra_turn_on_someone_elses_turn_does_not_consume_the_taker_s_own() {
    let mut game = at_the_turn_boundary(4);
    // Player 0 is active; the extra turn is player 1's.
    resolve_spell(&mut game, time_walk(), 1, vec![]);

    assert_eq!(
        next_turns(&mut game, &test_dp(), 4),
        vec![(1, 2), (1, 3), (2, 4), (3, 5)],
        "player 1's extra turn, then player 1's own, then the rotation"
    );
}

// ---------------------------------------------------------------------------
// CR 614.10 — a skipped step's contents are not proposed
// ---------------------------------------------------------------------------

// COVERS: ATOM-614.10-001
#[test]
fn yawgmoths_bargain_skips_the_whole_draw_step_and_not_just_the_draw() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 20);
    put_on_battlefield(&mut game, yawgmoths_bargain(), 0);
    game.phase = Phase { phase_type: PhaseType::Beginning, step: Some(StepType::Upkeep) };

    let before = game.events.len();
    let hand_before = game.players[0].hand.len();
    let (phase, step) = game.advance_turn(&test_ctx()).unwrap();

    // CR 500.11: proceed past it as though it didn't exist. The draw step's
    // *contents* are never proposed — no draw, no turn-based action — and the
    // one that did not happen announced nothing, so the turn goes straight
    // from the upkeep step to the precombat main phase.
    assert_eq!((phase, step), (PhaseType::Precombat, None));
    let records = game.events.records_from(before);
    assert!(
        !records.iter().any(|r| matches!(
            r.event,
            GameEvent::StepBegin { step: StepType::Draw }
        )),
        "the draw step did not begin, so it announced nothing"
    );
    assert!(
        !records.iter().any(|r| matches!(r.event, GameEvent::CardDrawn { .. })),
        "and nothing scheduled for it happened (CR 614.10a)"
    );
    assert_eq!(game.players[0].hand.len(), hand_before);
}

/// Eon Hub's first ruling, word for word: *"The upkeep step is skipped
/// entirely. The turn proceeds from untap step to draw step."*
#[test]
fn eon_hub_takes_the_turn_from_the_untap_step_to_the_draw_step() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 20);
    put_on_battlefield(&mut game, eon_hub(), 0);
    game.phase = Phase { phase_type: PhaseType::Beginning, step: Some(StepType::Untap) };

    let before = game.events.len();
    let (phase, step) = game.advance_turn(&test_ctx()).unwrap();
    assert_eq!((phase, step), (PhaseType::Beginning, Some(StepType::Draw)));

    let steps: Vec<StepType> = game
        .events
        .records_from(before)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::StepBegin { step } => Some(*step),
            _ => None,
        })
        .collect();
    assert_eq!(steps, vec![StepType::Draw], "the upkeep step announced nothing");
}

/// Eon Hub says "**players**", and CR 616.1's chooser is the affected player —
/// the one whose upkeep it is — so a four-player table asks nobody, four times
/// a round. `test_ctx`'s provider panics on an unexpected prompt, which is the
/// assertion.
#[test]
fn eon_hub_skips_every_players_upkeep_on_a_four_player_table() {
    let mut game = at_the_turn_boundary(4);
    put_on_battlefield(&mut game, eon_hub(), 0);

    let before = game.events.len();
    assert_eq!(
        next_turns(&mut game, &test_dp(), 4),
        vec![(1, 2), (2, 3), (3, 4), (0, 5)],
        "the rotation is untouched; only the upkeep steps are gone"
    );

    let records = game.events.records_from(before);
    assert!(
        !records.iter().any(|r| matches!(
            r.event,
            GameEvent::StepBegin { step: StepType::Upkeep }
        )),
        "no player's upkeep step began"
    );
    // One static ability, every player: the row is re-gathered per event off
    // the artifact's effective ability list rather than being per-player state.
    assert_eq!(game.replacement_effects.len(), 0, "a static needs no registry row");
}

// ---------------------------------------------------------------------------
// CR 614.10 — "once a step, phase, or turn has started, it can no longer be
// skipped"
// ---------------------------------------------------------------------------

// COVERS: ATOM-614.10-002
//
// The atom's own board, on the atom's own unit: the skip arrives *during* a
// draw step, and CR 614.10's last sentence — "once a step, phase, or turn has
// started, it can no longer be skipped" — is the proposal site. The current
// draw step is not retroactively ended; the next one never begins.
#[test]
fn a_skip_that_arrives_mid_step_waits_for_the_next_occurrence_of_it() {
    let mut game = at_the_turn_boundary(2);

    // Walk to player 1's draw step, which is where the skip will arrive.
    to_next_turn(&mut game, &test_dp());
    while game.phase.step != Some(StepType::Draw) {
        game.advance_turn(&ActionContext::new(&test_dp())).unwrap();
    }
    let drawn_before = cards_drawn(&game);
    assert_eq!(game.active_player, 1);

    // Yawgmoth's Bargain enters mid-draw-step. Its controller has already
    // drawn: the step is under way and is not un-started.
    put_on_battlefield(&mut game, yawgmoths_bargain(), 1);
    let before = game.events.len();
    game.advance_turn(&ActionContext::new(&test_dp())).unwrap();
    assert_eq!(
        game.phase,
        Phase { phase_type: PhaseType::Precombat, step: None },
        "the draw step ended the way it would have anyway"
    );
    assert!(
        !game.events.records_from(before).iter().any(|r| matches!(
            r.event,
            GameEvent::StepBegin { step: StepType::Draw }
        )),
        "and it was not re-proposed on its way out"
    );
    assert_eq!(cards_drawn(&game), drawn_before, "the draw had already happened");

    // Player 1's next draw step is the first occurrence the skip can meet, and
    // it does not happen.
    to_next_turn(&mut game, &test_dp());
    assert_eq!(to_next_turn(&mut game, &test_dp()), (1, 4));
    let before = game.events.len();
    let drawn_before = cards_drawn(&game);
    while game.phase.phase_type == PhaseType::Beginning {
        game.advance_turn(&ActionContext::new(&test_dp())).unwrap();
    }
    assert!(
        !game.events.records_from(before).iter().any(|r| matches!(
            r.event,
            GameEvent::StepBegin { step: StepType::Draw }
        )),
        "the draw step of the next occurrence did not begin (CR 614.10)"
    );
    assert_eq!(cards_drawn(&game), drawn_before, "and nothing scheduled for it happened");
}

// COVERS-PARTIAL: ATOM-614.10-002
//
// Moment of Silence's own ruling — "it must be used before the combat phase
// starts or it has no effect" — which is the atom's first half on a different
// unit. The second half has no board here: the row is `UntilEndOfTurn`, so
// there is no next occurrence for it to wait for, and the test above is the
// one that builds both.
#[test]
fn a_skip_created_during_the_combat_phase_meets_no_proposal_and_expires() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 20);
    game.phase = Phase { phase_type: PhaseType::Combat, step: Some(StepType::BeginCombat) };

    // Moment of Silence's second ruling: "It must be used before the combat
    // phase starts or it has no effect."
    resolve_spell(&mut game, moment_of_silence(), 0, vec![ResolvedTarget::Player(0)]);
    assert_eq!(game.replacement_effects.len(), 1);

    let before = game.events.len();
    // Out of combat and through the ending phase's cleanup step.
    advance(&mut game, &test_dp(), 8);

    // The current combat phase is not retroactively ended: its remaining steps
    // all begin.
    let steps: Vec<StepType> = game
        .events
        .records_from(before)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::StepBegin { step } => Some(*step),
            _ => None,
        })
        .collect();
    assert!(steps.contains(&StepType::EndCombat), "the combat phase finished normally");
    assert!(
        game.replacement_effects.is_empty(),
        "the row met no proposal and expired at cleanup (CR 615.3's duration half)"
    );
}

/// Moment of Silence's third ruling: *"If cast on a player when it is not their
/// turn, it has no effect."* Falls out of the event's subject rather than being
/// coded — a `BeginPhase` event is about the **active** player.
#[test]
fn a_skip_on_a_player_whose_turn_it_is_not_watches_nothing() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 20);
    game.phase = Phase { phase_type: PhaseType::Precombat, step: None };

    // Player 0 is the active player; the row is around player 1.
    resolve_spell(&mut game, moment_of_silence(), 0, vec![ResolvedTarget::Player(1)]);

    let before = game.events.len();
    let (phase, step) = game.advance_turn(&test_ctx()).unwrap();
    assert_eq!((phase, step), (PhaseType::Combat, Some(StepType::BeginCombat)));
    assert!(
        game.events.records_from(before).iter().any(|r| matches!(
            r.event,
            GameEvent::PhaseBegin { phase: PhaseType::Combat }
        )),
        "player 0's combat phase began"
    );
    assert_eq!(game.replacement_effects.len(), 1, "and the row is unspent");
}

/// Moment of Silence's first ruling, second sentence: *"If they manage to have
/// two combat phases, then only their next one combat phase is skipped."*
///
/// **The board is built by moving the cursor, because CR 500.8's extra phases
/// are unbuilt** — the turn queue RE-1 landed holds extra *turns* only, and the
/// drainer's cursor holds a phase *type*, so it could not tell two combat
/// phases apart even if something produced one. **RE-10 is the PR that deletes
/// this fixture** (`replacement-architecture.md` §9): once the cursor indexes a
/// turn plan, Aggravated Assault makes the second combat phase for real. What the fixture produces is a genuine second
/// `GameAction::BeginPhase { Combat }` proposal, which is the only thing the
/// claim is about: a `Uses::Once` row created during the first combat phase
/// meets no proposal there (CR 614.10's "once a phase has started, it can no
/// longer be skipped") and is spent on the next one. Relentless Assault
/// replaces the cursor move on the day its class lands.
#[test]
fn a_phase_skip_cast_during_combat_is_spent_on_the_next_combat_phase() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 20);

    // The first combat phase, already under way.
    game.phase = Phase { phase_type: PhaseType::Combat, step: Some(StepType::BeginCombat) };
    resolve_spell(&mut game, moment_of_silence(), 0, vec![ResolvedTarget::Player(0)]);

    // It finishes: CR 614.10's last sentence, and the row is unspent.
    let before = game.events.len();
    while game.phase.phase_type == PhaseType::Combat {
        game.advance_turn(&ActionContext::new(&test_dp())).unwrap();
    }
    assert!(
        steps_begun_from(&game, before).contains(&StepType::EndCombat),
        "the combat phase that had started finished"
    );
    assert_eq!(game.replacement_effects.len(), 1, "and the row is unspent");

    // A second combat phase, as CR 500.8 would insert one.
    game.phase = Phase { phase_type: PhaseType::Precombat, step: None };
    let before = game.events.len();
    let (phase, step) = game.advance_turn(&ActionContext::new(&test_dp())).unwrap();

    assert_eq!(
        (phase, step),
        (PhaseType::Postcombat, None),
        "the second combat phase is the one that gets skipped"
    );
    let records = game.events.records_from(before);
    assert!(
        !records.iter().any(|r| matches!(
            r.event,
            GameEvent::PhaseBegin { phase: PhaseType::Combat }
        )),
        "and it announced nothing"
    );
    assert!(game.replacement_effects.is_empty(), "one use, one phase");
}

/// The phase-level skip, and the rule a step-level one cannot show: a skipped
/// phase proposes **none** of its steps (CR 500.11).
#[test]
fn a_skipped_phase_proposes_none_of_its_steps() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 20);
    game.phase = Phase { phase_type: PhaseType::Precombat, step: None };
    resolve_spell(&mut game, moment_of_silence(), 0, vec![ResolvedTarget::Player(0)]);

    let before = game.events.len();
    let (phase, step) = game.advance_turn(&test_ctx()).unwrap();

    // Straight from the precombat main phase to the postcombat one.
    assert_eq!((phase, step), (PhaseType::Postcombat, None));
    let records = game.events.records_from(before);
    assert!(
        !records.iter().any(|r| matches!(r.event, GameEvent::StepBegin { .. })),
        "the combat phase's six steps were never proposed"
    );
    assert!(
        !records.iter().any(|r| matches!(
            r.event,
            GameEvent::PhaseBegin { phase: PhaseType::Combat }
        )),
        "and the phase announced nothing"
    );
    assert!(game.replacement_effects.is_empty(), "one use, one phase");
}

// ---------------------------------------------------------------------------
// CR 614.1b — "skip" identifies a replacement effect
// ---------------------------------------------------------------------------

// COVERS: BOUNDARY-DEF-614.1b-001
#[test]
fn skip_is_a_replacement_effect_and_an_at_the_beginning_of_ability_is_not() {
    // In-set: Yawgmoth's Bargain's "Skip your draw step" is a static ability
    // whose effect is a `ReplacementDef`, discovered by the replacement sweep
    // off the permanent's effective ability list.
    let bargain = yawgmoths_bargain();
    let skip_ability = &bargain.abilities[0];
    assert_eq!(skip_ability.ability_type, AbilityType::Static);
    assert!(matches!(skip_ability.effect, Effect::Replacement(_)));

    // Out-of-set: "At the beginning of your draw step, draw an additional
    // card" is a **triggered** ability. It is not a replacement effect, and the
    // engine's proof of that is the one that matters — the replacement sweep
    // does not find it, so the draw step still happens.
    let trigger = Arc::new(
        mtgsim::objects::card_data::CardDataBuilder::new("Draw-Step Trigger Fixture")
            .card_type(mtgsim::types::card_types::CardType::Enchantment)
            .rules_text("At the beginning of your draw step, draw an additional card.")
            .ability(AbilityDef {
                id: new_ability_id(),
                ability_type: AbilityType::Triggered,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DrawCards(AmountExpr::Fixed(1)),
                    EffectRecipient::Controller,
                ),
                is_characteristic_defining: false,
                activation_restriction:
                    mtgsim::objects::card_data::ActivationRestriction::None,
            })
            .build()
            .as_ref()
            .clone(),
    );

    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 20);
    put_on_battlefield(&mut game, trigger, 0);
    game.phase = Phase { phase_type: PhaseType::Beginning, step: Some(StepType::Upkeep) };

    let before = game.events.len();
    let (phase, step) = game.advance_turn(&test_ctx()).unwrap();
    assert_eq!(
        (phase, step),
        (PhaseType::Beginning, Some(StepType::Draw)),
        "a triggered ability replaces nothing: the draw step begins"
    );
    assert_eq!(
        game.events
            .records_from(before)
            .iter()
            .filter(|r| matches!(r.event, GameEvent::CardDrawn { .. }))
            .count(),
        1,
        "the turn-based draw happened; the trigger's own draw is item 6's"
    );
}

// ---------------------------------------------------------------------------
// CR 614.10a — "anything scheduled for the next occurrence waits for the first
// occurrence that isn't skipped"
// ---------------------------------------------------------------------------

/// A duration is scheduled for "your next turn", and a skipped turn is not one:
/// the effect lasts across it to the turn that actually begins (CR 611.2b,
/// 614.10a).
#[test]
fn until_your_next_turn_waits_for_the_first_turn_that_is_not_skipped() {
    let mut game = at_the_turn_boundary(2);
    let creature = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
    pump_until_your_next_turn(&mut game, creature, 0);
    resolve_spell(&mut game, meditate(), 0, vec![]);
    assert_eq!(game.continuous_effects.len(), 1);

    // Player 1's turn: the wrong player, so nothing expires.
    assert_eq!(to_next_turn(&mut game, &test_dp()), (1, 2));
    assert_eq!(game.continuous_effects.len(), 1, "not this player's turn");

    // Player 0's next turn is skipped, so it expires nothing — and the turn
    // that does begin is player 1's again.
    assert_eq!(to_next_turn(&mut game, &test_dp()), (1, 3));
    assert_eq!(
        game.continuous_effects.len(),
        1,
        "a turn that did not begin is not 'your next turn' (CR 614.10a)"
    );

    // The first turn of player 0's that is not skipped is the one it ends on.
    assert_eq!(to_next_turn(&mut game, &test_dp()), (0, 4));
    assert!(game.continuous_effects.is_empty());
}

/// Its twin, and the one the tree used to simulate by handing the registry a
/// turn number: an **extra** turn for the controller is their next turn, so the
/// effect expires at its start.
#[test]
fn until_your_next_turn_expires_on_a_real_extra_turn() {
    let mut game = at_the_turn_boundary(2);
    let creature = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
    pump_until_your_next_turn(&mut game, creature, 0);
    resolve_spell(&mut game, time_walk(), 0, vec![]);

    assert_eq!(
        to_next_turn(&mut game, &test_dp()),
        (0, 2),
        "the extra turn is player 0's, and it is the next turn there is"
    );
    assert!(
        game.continuous_effects.is_empty(),
        "an extra turn for the controller IS their next turn (CR 611.2b)"
    );
}

/// "Target creature gets +1/+1 until your next turn", as a resolution.
fn pump_until_your_next_turn(game: &mut GameState, creature: ObjectId, controller: PlayerId) {
    let effect = Effect::Atom(
        Primitive::ModifyPowerToughness(
            AmountExpr::Fixed(1),
            AmountExpr::Fixed(1),
            Duration::UntilYourNextTurn,
        ),
        EffectRecipient::Implicit,
    );
    let ctx = ResolutionContext {
        source: creature,
        ability_source: None,
        controller,
        targets: vec![ResolvedTarget::Object(creature)],
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(&effect, &ctx, &test_dp()).unwrap();
}

// ---------------------------------------------------------------------------
// The units the drainer proposes, and the ones it does not
// ---------------------------------------------------------------------------

/// Every unit of an ordinary turn is proposed, and the three events that had
/// existed since the log was written with nothing emitting them now carry it.
#[test]
fn an_ordinary_turn_announces_its_turn_its_five_phases_and_its_steps() {
    let mut game = at_the_turn_boundary(2);

    // One whole turn, cut at its own boundaries: everything the log records
    // between this turn beginning and the next one.
    let before = game.events.len();
    to_next_turn(&mut game, &test_dp());
    to_next_turn(&mut game, &test_dp());
    let all = game.events.records_from(before);
    let records: Vec<_> = all
        .iter()
        .skip_while(|r| !matches!(r.event, GameEvent::TurnBegin { .. }))
        .skip(1)
        .take_while(|r| !matches!(r.event, GameEvent::TurnBegin { .. }))
        .collect();

    let phases: Vec<PhaseType> = records
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::PhaseBegin { phase } => Some(*phase),
            _ => None,
        })
        .collect();
    assert_eq!(
        phases,
        vec![
            PhaseType::Beginning,
            PhaseType::Precombat,
            PhaseType::Combat,
            PhaseType::Postcombat,
            PhaseType::Ending,
        ],
        "CR 500.1 — each phase takes place every turn"
    );
    let steps: Vec<StepType> = records
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::StepBegin { step } => Some(*step),
            _ => None,
        })
        .collect();
    // CR 508.8 — with no attackers, the declare blockers and combat damage
    // steps are skipped, and a skipped step announces nothing. The first-strike
    // step is the engine's own split of CR 510.4's, and 508.8 names it too.
    assert_eq!(
        steps,
        vec![
            StepType::Untap,
            StepType::Upkeep,
            StepType::Draw,
            StepType::BeginCombat,
            StepType::DeclareAttackers,
            StepType::EndCombat,
            StepType::End,
            StepType::Cleanup,
        ]
    );
}

/// CR 508.8 is a rule at the proposal site, not a replacement effect: with no
/// attackers there is no event for CR 614 to see at all.
#[test]
fn with_no_attackers_the_blocker_and_damage_steps_never_begin() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 20);
    game.phase = Phase { phase_type: PhaseType::Combat, step: Some(StepType::DeclareAttackers) };
    assert!(!game.attacks_declared);

    let before = game.events.len();
    let (phase, step) = game.advance_turn(&test_ctx()).unwrap();
    assert_eq!((phase, step), (PhaseType::Combat, Some(StepType::EndCombat)));
    assert_eq!(
        steps_begun_from(&game, before),
        vec![StepType::EndCombat],
        "three steps skipped, none of them announced"
    );
}

fn steps_begun_from(game: &GameState, from: usize) -> Vec<StepType> {
    game.events
        .records_from(from)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::StepBegin { step } => Some(*step),
            _ => None,
        })
        .collect()
}

/// CR 800.4k — "if a player who has left the game would begin a turn, that turn
/// doesn't begin". A rule at the same site, ahead of the pipeline: RE-6 is what
/// makes `player_lost` true for a reason, and this is the site it will use.
#[test]
fn a_lost_players_turn_does_not_begin() {
    let mut game = at_the_turn_boundary(4);
    game.player_lost[1] = true;

    assert_eq!(
        next_turns(&mut game, &test_dp(), 2),
        vec![(2, 2), (3, 3)],
        "player 1 is passed over entirely, and the turn numbers are the two that began"
    );
}

// ---------------------------------------------------------------------------
// The whole-game path
// ---------------------------------------------------------------------------

/// The first turn's units are proposed like any other's, which is what makes
/// item 6's "at the beginning of your upkeep" readable on turn 1 — and what
/// makes turn 1's untap step run its turn-based action at all.
#[test]
fn the_first_turn_and_its_untap_step_are_proposed() {
    use mtgsim::state::game::Game;
    use mtgsim::state::game_config::GameConfig;

    let deck: Vec<Arc<CardData>> =
        (0..40).map(|_| mtgsim::cards::basic_lands::forest()).collect();
    let mut g = Game::new(GameConfig::standard(), vec![deck.clone(), deck]).unwrap();
    g.reseed(7);
    g.setup(&test_dp()).unwrap();

    assert_eq!(turns_begun(&g.state), vec![(0, 1)]);
    assert_eq!(phases_begun(&g.state), vec![PhaseType::Beginning]);
    assert_eq!(steps_begun(&g.state), vec![StepType::Untap]);
    assert_eq!(g.state.phase.step, Some(StepType::Untap));
    // CR 103.4's opening hands are draws and nothing else is: the untap step's
    // own turn-based action drew nothing.
    assert_eq!(cards_drawn(&g.state), 14);
}
