//! Phase RE-10 — extra phases, and the turn plan.
//!
//! CR 500.8, 500.5, 505.1 and 511.3, against Aggravated Assault — CR 500.8's
//! first producer in this engine.
//!
//! **Every board here is a turn sequence**, as RE-1's are, and for a sharper
//! reason: the whole PR is a cursor change, so the only thing worth asserting
//! is which phases a turn actually *had* and in what order. The event log
//! answers that directly — `PhaseBegin` is emitted by the performer and by
//! nothing else — and a spliced phase is indistinguishable from a natural one
//! in it, which is the point.
//!
//! **Two boards here belong to RE-1 and could not be built then.** Moment of
//! Silence is a card RE-1 registered whose first two rulings both need a turn
//! with two combat phases; RE-1 tested them against a second
//! `GameAction::BeginPhase { Combat }` proposal its fixture made by moving the
//! drainer's cursor by hand, because CR 500.8's extra phases were unbuilt.
//! [`moment_of_silence_skips_only_the_next_of_two_combat_phases`] and
//! [`a_skip_cast_during_a_combat_phase_is_spent_on_the_next_one`] are those
//! claims against a board the engine produces, and the cursor move is deleted
//! with this file's arrival.

use std::sync::Arc;

use mtgsim::cards::phase_re10_cards::aggravated_assault;
use mtgsim::cards::phase_re_cards::moment_of_silence;
use mtgsim::engine::actions::ActionContext;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::{
    fill_library, place_vanilla_creature, put_in_hand, put_on_battlefield, set_attacking,
    setup_game, setup_two_player_game, test_ctx, test_dp,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::engine::targeting::{ChosenTargets};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Put an Aggravated Assault onto the battlefield under `controller` and
/// resolve its activated ability, the way the stack would.
///
/// The activation's own legality — `ActivationRestriction::OnlyAsSorcery` and
/// the {3}{R}{R} — is `cast.rs`'s and is tested by Bonesplitter's equip; what
/// is under test here is what the *resolution* does to the turn.
fn activate_assault(game: &mut GameState, controller: PlayerId) -> ObjectId {
    let card = aggravated_assault();
    let id = put_on_battlefield(game, card.clone(), controller);
    resolve_ability(game, &card, id, controller);
    id
}

/// Resolve `source`'s first activated ability for `controller`.
fn resolve_ability(
    game: &mut GameState,
    card: &Arc<CardData>,
    source: ObjectId,
    controller: PlayerId,
) {
    let ctx = ResolutionContext {
        source,
        ability_source: Some(source),
        controller,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp())
        .expect("resolving Aggravated Assault");
}

/// Resolve `card`'s spell effect for `controller`, the way the stack would.
fn resolve_spell(
    game: &mut GameState,
    card: Arc<CardData>,
    controller: PlayerId,
    targets: Vec<ResolvedTarget>,
) {
    let id = put_in_hand(game, card.clone(), controller);
    let ctx = ResolutionContext {
        source: id,
        ability_source: None,
        controller,
        targets: ChosenTargets::one(targets),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
}

/// Every phase that began after `mark` and **before the next turn did**.
///
/// `advance_turn` reports the position it lands on, and the position after a
/// turn's last one belongs to the next turn — so a raw range carries one phase
/// too many. Cut at the `TurnBegin`.
fn phases_begun_this_turn(game: &GameState, mark: usize) -> Vec<PhaseType> {
    game.events
        .records_from(mark)
        .iter()
        .take_while(|r| !matches!(r.event, GameEvent::TurnBegin { .. }))
        .filter_map(|r| match &r.event {
            GameEvent::PhaseBegin { phase, .. } => Some(*phase),
            _ => None,
        })
        .collect()
}

/// Walk until the turn ends, counting the positions `advance_turn` reports.
///
/// Counted by turn boundary rather than by a fixed step count, for RE-1's
/// reason: the number of positions is exactly what an extra phase changes.
fn positions_to_end_of_turn(game: &mut GameState, dp: &dyn DecisionProvider) -> usize {
    let ctx = ActionContext::new(dp);
    let turn = game.turn_number;
    for count in 1..500 {
        game.advance_turn(&ctx).expect("advancing");
        if game.turn_number != turn {
            return count;
        }
    }
    panic!("the turn did not end within 500 positions");
}

/// A board in player 0's precombat main phase, everyone stocked.
fn in_precombat_main(num_players: usize) -> GameState {
    let mut game = setup_game(num_players);
    for pid in 0..num_players {
        fill_library(&mut game, pid, 60);
    }
    game.set_turn_position(Phase { phase_type: PhaseType::Precombat, step: None });
    game
}

// ---------------------------------------------------------------------------
// CR 500.8 — extra phases, and where they go
// ---------------------------------------------------------------------------

// COVERS: ATOM-500.8-001
//
// The rule's first sentence, and the card's: two phases, directly after the
// one the ability resolved in, in the order the card prints them.
#[test]
fn two_extra_phases_are_spliced_directly_after_the_phase_that_made_them() {
    let mut game = in_precombat_main(2);
    activate_assault(&mut game, 0);

    // The plan, before a single position is walked: the splice is what the
    // resolution did, and the drainer has not been asked anything yet.
    assert_eq!(
        game.turn_plan.phases.iter().map(|p| p.phase_type).collect::<Vec<_>>(),
        vec![
            PhaseType::Beginning,
            PhaseType::Precombat,
            // The pair, directly after the precombat main the ability
            // resolved in — not after the natural combat phase.
            PhaseType::Combat,
            PhaseType::Postcombat,
            PhaseType::Combat,
            PhaseType::Postcombat,
            PhaseType::Ending,
        ],
        "CR 500.8 — directly after the specified phase, in printed order"
    );

    // And the turn walks them: two combats and two postcombat mains.
    let mark = game.events.len();
    positions_to_end_of_turn(&mut game, &test_dp());
    assert_eq!(
        phases_begun_this_turn(&game, mark),
        vec![
            PhaseType::Combat,
            PhaseType::Postcombat,
            PhaseType::Combat,
            PhaseType::Postcombat,
            PhaseType::Ending,
        ],
        "a spliced phase begins exactly like a natural one"
    );
}

/// CR 500.8's last sentence, and the card's second ruling — *"if you have
/// enough mana, the ability may be activated more than once in a turn."*
///
/// > 500.8. ... If multiple extra phases are created after the same phase, the
/// > most recently created phase will occur first.
///
/// There is no comparator anywhere that says this. The second activation
/// splices at the same index and pushes the first activation's pair behind it,
/// which *is* the rule — the same shape CR 500.7's "most recently created
/// turn" turned out to be `Vec::pop` for.
#[test]
fn a_second_activation_puts_its_phases_ahead_of_the_firsts() {
    let mut game = in_precombat_main(2);
    let assault = activate_assault(&mut game, 0);

    // A second activation of the *same* permanent, in the same main phase.
    let card = aggravated_assault();
    resolve_ability(&mut game, &card, assault, 0);

    assert_eq!(game.turn_plan.phases.len(), 9, "five natural plus two pairs");

    // Both pairs sit between the precombat main and the natural combat phase,
    // and the rule is only observable in that there are four of them: what the
    // second activation bought is that *its* pair is walked first. The
    // permanents are identical, so the assertion that carries the rule is the
    // count and the block's shape.
    assert_eq!(
        game.turn_plan.phases[2..6].iter().map(|p| p.phase_type).collect::<Vec<_>>(),
        vec![
            PhaseType::Combat,
            PhaseType::Postcombat,
            PhaseType::Combat,
            PhaseType::Postcombat,
        ],
        "the second splice went in ahead of the first, not after it"
    );

    let mark = game.events.len();
    positions_to_end_of_turn(&mut game, &test_dp());
    assert_eq!(
        phases_begun_this_turn(&game, mark)
            .iter()
            .filter(|p| **p == PhaseType::Combat)
            .count(),
        3,
        "two extra combat phases and the natural one"
    );
}

/// The plan is this turn's, and CR 500.8's "after this main phase" names a
/// phase of the turn the effect resolved in.
#[test]
fn extra_phases_do_not_survive_into_the_next_turn() {
    let mut game = in_precombat_main(2);
    activate_assault(&mut game, 0);
    positions_to_end_of_turn(&mut game, &test_dp());

    assert_eq!(
        game.turn_plan.phases.len(),
        5,
        "the next turn is CR 500.1's five phases again"
    );
    let mark = game.events.len();
    positions_to_end_of_turn(&mut game, &test_dp());
    assert_eq!(
        phases_begun_this_turn(&game, mark)
            .iter()
            .filter(|p| **p == PhaseType::Combat)
            .count(),
        1,
        "one combat phase, like any turn nobody spent eight mana on"
    );
}

/// A turn's position count is one of the two counters an extra phase may
/// legitimately move, and this is the test that says by how much.
///
/// **Four, not seven, and CR 508.8 is why.** A combat phase has six steps, but
/// nobody attacks on this board, so *"if no creatures are declared as
/// attackers ... skip the declare blockers and combat damage steps"* drops
/// three of them before the pipeline ever sees them — leaving begin combat,
/// declare attackers and end combat, plus the stepless main phase. The number
/// this asserts is therefore a fact about two rules meeting, which is why it is
/// asserted rather than reasoned about.
#[test]
fn an_extra_combat_and_main_add_four_positions_to_an_unattacked_turn() {
    let without = {
        let mut game = in_precombat_main(2);
        positions_to_end_of_turn(&mut game, &test_dp())
    };
    let with = {
        let mut game = in_precombat_main(2);
        activate_assault(&mut game, 0);
        positions_to_end_of_turn(&mut game, &test_dp())
    };
    assert_eq!(
        with - without,
        4,
        "begin combat, declare attackers, end combat (CR 508.8), and a main phase"
    );
}

// ---------------------------------------------------------------------------
// What a second combat phase has to be clean for
// ---------------------------------------------------------------------------

/// CR 511.3 and CR 500.5 — the combat state clears when a combat phase ends,
/// and *this* is the board that makes that observable for the first time.
///
/// `on_phase_end(Combat)` has cleared all five fields since long before RE, but
/// no turn had two combat phases to notice with. This test is what keeps it
/// true: a creature that attacked in the first combat phase is not still
/// attacking in the second.
#[test]
fn combat_state_is_clear_at_the_start_of_the_second_combat_phase() {
    let mut game = in_precombat_main(2);
    let attacker = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
    activate_assault(&mut game, 0);

    // Walk into the first combat phase's declare-attackers step and attack.
    let dp = test_dp();
    let ctx = ActionContext::new(&dp);
    while !(game.phase.phase_type == PhaseType::Combat
        && game.phase.step == Some(StepType::DeclareAttackers))
    {
        game.advance_turn(&ctx).expect("advancing");
    }
    game.attacks_declared = true;
    set_attacking(&mut game, attacker, 1);

    // Walk into the second combat phase.
    let mut seen_combats = 1;
    while seen_combats < 2 {
        game.advance_turn(&ctx).expect("advancing");
        if game.phase.phase_type == PhaseType::Combat
            && game.phase.step == Some(StepType::BeginCombat)
        {
            seen_combats += 1;
        }
    }

    assert!(!game.attacks_declared, "CR 511.3 — the first combat's state is gone");
    assert!(
        game.battlefield[&attacker].attacking.is_none(),
        "and the creature is not still attacking"
    );
}

/// CR 500.5 — mana pools empty at the end of *each* phase, spliced ones
/// included.
#[test]
fn mana_empties_at_the_end_of_each_inserted_phase() {
    let mut game = in_precombat_main(2);
    activate_assault(&mut game, 0);

    let dp = test_dp();
    let ctx = ActionContext::new(&dp);
    // Into the first inserted combat phase, then float mana inside it.
    while game.phase.phase_type != PhaseType::Combat {
        game.advance_turn(&ctx).expect("advancing");
    }
    game.players[0].mana_pool.add(ManaType::Red, 3);
    assert_eq!(game.players[0].mana_pool.total(), 3);

    // Out the other side of it.
    while game.phase.phase_type == PhaseType::Combat {
        game.advance_turn(&ctx).expect("advancing");
    }
    assert_eq!(
        game.players[0].mana_pool.total(),
        0,
        "an inserted phase ends like any other (CR 500.5)"
    );
}

/// CR 505.1 / 701.26b — the untap half of the card, and the arm that rides
/// with it. "Untap all creatures you control" is **one batch**, because
/// CR 603.2c's "whenever one or more permanents untap" reads the batch rather
/// than its members.
#[test]
fn untap_all_creatures_you_control_is_one_batch_and_spares_the_opponents() {
    let mut game = in_precombat_main(2);
    let mine_a = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
    let mine_b = place_vanilla_creature(&mut game, 0, 1, 1, &[]);
    let theirs = place_vanilla_creature(&mut game, 1, 3, 3, &[]);
    for id in [mine_a, mine_b, theirs] {
        game.battlefield.get_mut(&id).unwrap().tapped = true;
    }

    let mark = game.events.len();
    activate_assault(&mut game, 0);

    assert!(!game.battlefield[&mine_a].tapped);
    assert!(!game.battlefield[&mine_b].tapped);
    assert!(
        game.battlefield[&theirs].tapped,
        "\"you control\" — the opponent's creature is untouched"
    );

    let batches: Vec<_> = game
        .events
        .records_from(mark)
        .iter()
        .filter(|r| matches!(r.event, GameEvent::Untapped { .. }))
        .map(|r| r.stamp.batch)
        .collect();
    assert_eq!(batches.len(), 2, "two untaps");
    assert!(batches[0].is_some(), "performed inside a batch");
    assert_eq!(batches[0], batches[1], "and one batch (CR 603.2c)");
}

// ---------------------------------------------------------------------------
// RE-1's card, on a board RE-1 could not build
// ---------------------------------------------------------------------------

/// Moment of Silence's first ruling, without the fixture that used to carry it.
///
/// > "If they manage to have two combat phases, then only their next one combat
/// > phase is skipped."
///
/// RE-1 asserted this by moving the drainer's cursor back to the precombat
/// main phase by hand, because nothing in the engine could create a second
/// combat phase. This is the same claim against a board the engine produced:
/// Aggravated Assault makes two, the skip eats the first, and the second
/// happens.
#[test]
fn moment_of_silence_skips_only_the_next_of_two_combat_phases() {
    let mut game = in_precombat_main(2);
    activate_assault(&mut game, 0);
    resolve_spell(&mut game, moment_of_silence(), 0, vec![ResolvedTarget::Player(0)]);

    // Asserted before the walk: `on_turn_begin` rebuilds the plan, so after
    // the turn boundary this reads the *next* turn's five phases.
    assert_eq!(
        game.turn_plan.phases.iter().filter(|p| p.phase_type == PhaseType::Combat).count(),
        2,
        "the extra combat phase and the natural one"
    );

    let mark = game.events.len();
    positions_to_end_of_turn(&mut game, &test_dp());
    assert_eq!(
        phases_begun_this_turn(&game, mark)
            .iter()
            .filter(|p| **p == PhaseType::Combat)
            .count(),
        1,
        "the *next* one was skipped, and the one after it still happened"
    );
    assert!(
        game.replacement_effects.is_empty(),
        "one use, one phase — and it was spent on the first of them"
    );
}

/// Moment of Silence's other ruling, on the same board and legal throughout:
/// *"it must be used before the combat phase starts or it has no effect."*
///
/// > 614.10. ... once a step or phase has started, it can no longer be skipped.
///
/// **This is the test RE-1 wrote by moving the cursor**, and the move is what
/// RE-10 deletes. The board is now one the engine produces and one a player
/// could sit at: Aggravated Assault makes an extra combat phase at sorcery
/// speed, the instant is cast *inside* that phase, and the row it creates meets
/// no proposal there — the phase has already started — so it survives to be
/// spent on the natural combat phase later in the same turn.
///
/// COVERS-PARTIAL: ATOM-614.10-002 — the half `phase_re1_integration_test`'s
/// `a_skip_created_during_the_combat_phase_meets_no_proposal_and_expires`
/// records as having no board on this unit: an `UntilEndOfTurn` row waiting
/// for a *next* occurrence of the phase it was created in. The atom's own
/// board is the draw step and is claimed there; this is the same rule on the
/// unit CR 500.8 made repeatable.
#[test]
fn a_skip_cast_during_a_combat_phase_is_spent_on_the_next_one() {
    let mut game = in_precombat_main(2);
    activate_assault(&mut game, 0);

    // Into the first (spliced) combat phase.
    let dp = test_dp();
    let ctx = ActionContext::new(&dp);
    while game.phase.phase_type != PhaseType::Combat {
        game.advance_turn(&ctx).expect("advancing");
    }

    // Cast inside it. CR 614.10's last sentence: the phase has started, so
    // this row watches nothing here.
    resolve_spell(&mut game, moment_of_silence(), 0, vec![ResolvedTarget::Player(0)]);
    let mark = game.events.len();
    while game.phase.phase_type == PhaseType::Combat {
        game.advance_turn(&ctx).expect("advancing");
    }
    assert_eq!(
        game.replacement_effects.len(),
        1,
        "the combat phase that had started finished, and the row is unspent"
    );

    // And it is spent on the natural combat phase, which never begins.
    positions_to_end_of_turn(&mut game, &dp);
    assert_eq!(
        phases_begun_this_turn(&game, mark)
            .iter()
            .filter(|p| **p == PhaseType::Combat)
            .count(),
        0,
        "the second combat phase is the one that gets skipped"
    );
    assert!(game.replacement_effects.is_empty(), "one use, one phase");
}

// ---------------------------------------------------------------------------
// Four players
// ---------------------------------------------------------------------------

/// The plan is the active player's turn, and a four-player table changes
/// nothing about that — but it is the board where "whose turn's plan is it"
/// could go wrong, so it is asserted rather than assumed (`CLAUDE.md`:
/// write new systems N-player-shaped).
#[test]
fn an_extra_phase_belongs_to_the_turn_that_made_it_on_a_four_player_table() {
    let mut game = in_precombat_main(4);
    activate_assault(&mut game, 0);
    assert_eq!(game.turn_plan.phases.len(), 7);

    // Player 0's turn has the extra pair.
    let mark = game.events.len();
    positions_to_end_of_turn(&mut game, &test_dp());
    assert_eq!(
        phases_begun_this_turn(&game, mark)
            .iter()
            .filter(|p| **p == PhaseType::Combat)
            .count(),
        2
    );

    // The next three players' turns are ordinary, and the Aggravated Assault
    // is still on the battlefield throughout — the plan is per turn, not per
    // permanent.
    for _ in 0..3 {
        let mark = game.events.len();
        assert_eq!(game.turn_plan.phases.len(), 5);
        positions_to_end_of_turn(&mut game, &test_dp());
        assert_eq!(
            phases_begun_this_turn(&game, mark)
                .iter()
                .filter(|p| **p == PhaseType::Combat)
                .count(),
            1,
            "an opponent's turn gets no phases from a permanent they do not control"
        );
    }
}

/// The plan's cursor and the position are two facts, and
/// `GameState::set_turn_position` is the only seam that writes both by hand. A
/// fixture that writes `phase` alone drains from wherever the cursor was —
/// which is the bug this seam exists to make unwriteable, and the
/// `debug_assert` in `advance_turn` is what catches it.
#[test]
fn the_seam_moves_the_cursor_with_the_position() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 20);
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(StepType::BeginCombat) });

    assert_eq!(game.turn_plan.cursor, Some(2), "combat is the third of the five");
    assert_eq!(game.phase.phase_type, PhaseType::Combat);

    // And the drainer advances from there rather than from turn 1's beginning.
    let mark = game.events.len();
    game.advance_turn(&test_ctx()).expect("advancing");
    assert_eq!(
        phases_begun_this_turn(&game, mark),
        Vec::new(),
        "still inside the combat phase it was placed in"
    );
    assert_eq!(game.phase.phase_type, PhaseType::Combat);
}
