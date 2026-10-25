//! Item 185, built (`plans/elisions.md`): a record at each question the
//! engine answers without asking, and an audit at each shortcut that had only
//! its argument.
//!
//! **The record.** Every forced prompt is decided in `ui::ask`, so each ask
//! is called here as an engine site calls it, with a provider that panics on
//! any prompt: the answer comes back unasked, and the game's log says so. The
//! three elisions are shown where their boards already are: CR 616.1's in
//! `phase_rg_integration_test.rs`, CR 603.3b's in
//! `phase_tr5a_integration_test.rs`, CR 603.7b's in
//! `phase_tr3b_integration_test.rs`.
//!
//! **The audits.** Each gate is shown agreeing on a board its printed leg
//! covers, then panicking when the leg is taken away: the board a missing leg
//! would leave silently wrong.

use mtgsim::cards::alpha::giant_growth;
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_cm_cards::thalia_guardian_of_thraben;
use mtgsim::cards::phase_rb_cards::rest_in_peace;
use mtgsim::cards::phase_rs_cards::sigarda_host_of_herons;
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::cost_determination::cost_modifications_for;
use mtgsim::types::ids::ObjectOrPlayerId;
use mtgsim::events::event::DamageTarget;
use mtgsim::oracle::x_values::LegalXValues;
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    assert_not_asked, card_of_type, effect_ctx, forest, not_asked, place_forest, put_in_hand, put_on_battlefield,
    setup_two_player_game, test_ctx,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{EffectRecipient, SelectionFilter, TargetCount};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::zones::{Zone, ZoneChangeAct};
use mtgsim::ui::ask::{
    ask_allocate_next_damage, ask_choose_copy_source, ask_choose_damage_source, ask_choose_entering_controller,
    ask_choose_entering_defender, ask_choose_generic_mana_allocation, ask_choose_sacrifice_for_cost,
    ask_choose_trample_damage_assignment, ask_choose_x_value, ask_discard, ask_select_recipients,
};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::display::{format_event_log, format_record, log_lines};

const ONE_LEGAL_ANSWER: &str = "its one legal answer";

/// A provider that panics on any prompt: an answer it lets through was not asked.
fn asks_nobody() -> ScriptedDecisionProvider {
    ScriptedDecisionProvider::new()
}

// ---------------------------------------------------------------------------
// The record: in the game's log, and not in the dev GUI's
// ---------------------------------------------------------------------------

/// A discard that takes the whole hand, between two taps: the full log puts
/// the unasked question where it was taken, and the dev GUI's log leaves it
/// out, as it leaves out a mana ability's activation.
#[test]
fn the_game_log_says_what_was_not_asked_and_the_dev_gui_log_does_not() {
    let mut game = setup_two_player_game();
    let land = put_on_battlefield(&mut game, forest(), 0);
    let card = put_in_hand(&mut game, forest(), 1);
    let before = game.recorded_events().len();
    game.execute_action(GameAction::Tap { object: land }, &test_ctx()).unwrap();

    let discarded = ask_discard(&asks_nobody(), &game, 1, &[card], 1, None);
    assert_eq!(discarded, vec![card]);

    let full = format_event_log(&game);
    let unasked = format!("NotAsked: P1 Discard: Forest ({card}), {ONE_LEGAL_ANSWER}");
    assert_eq!(full.last(), Some(&unasked), "after the tap it followed: {full:#?}");

    let recorded = game.recorded_events();
    let drawn: Vec<String> =
        log_lines(&game, recorded.records_from(before)).into_iter().map(|record| format_record(&game, record)).collect();
    assert!(!drawn.is_empty() && drawn.iter().all(|line| !line.starts_with("NotAsked")), "{drawn:#?}");
}

/// CR 732.1: a cast reversed after its one legal target was answered
/// unasked takes the note back, since the game's log shows nothing of a
/// reversed action.
#[test]
fn a_reversed_cast_takes_back_what_it_answered_unasked() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    let growth = put_in_hand(&mut game, giant_growth(), 0);
    place_forest(&mut game, 0);
    let dp = ScriptedDecisionProvider::new();
    let window = ChoiceKind::ManaAbilityWindow { spell_or_ability_id: growth, remaining_cost: ManaCost::zero() };
    dp.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    dp.expect_pick_n(window, vec![]);
    dp.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);

    game.run_priority_round(&dp).unwrap();
    assert!(game.players[0].hand.contains(&growth), "the cast was reversed");
    assert!(not_asked(&game).is_empty(), "{:#?}", not_asked(&game));
}

/// A game nobody records writes nothing, and the answer is the same.
#[test]
fn a_game_that_records_nothing_notes_nothing() {
    let mut game = GameState::new(2, 20);
    let card = put_in_hand(&mut game, forest(), 1);
    assert_eq!(ask_discard(&asks_nobody(), &game, 1, &[card], 1, None), vec![card]);
}

// ---------------------------------------------------------------------------
// Each forced prompt, answered unasked and recorded
// ---------------------------------------------------------------------------

/// "Target creature" with one creature: the common forced prompt.
#[test]
fn a_fixed_count_taking_every_legal_recipient_is_answered_unasked() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let spell = put_in_hand(&mut game, card_of_type("Fixture Bolt", CardType::Instant), 0);
    let target_creature = EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1));
    let legal = [ObjectOrPlayerId::Object(bears)];

    let chosen = ask_select_recipients(&asks_nobody(), &game, 0, &target_creature, spell, &legal, 1, 1);
    assert_eq!(chosen, legal);
    assert_eq!(
        not_asked(&game),
        vec![format!("NotAsked: P0 SelectRecipients for Fixture Bolt ({spell}): Grizzly Bears ({bears}), {ONE_LEGAL_ANSWER}")]
    );
}

#[test]
fn an_x_with_one_payable_value_is_announced_unasked() {
    let mut game = setup_two_player_game();
    let spell = put_in_hand(&mut game, card_of_type("Fixture Blaze", CardType::Sorcery), 0);
    assert_eq!(ask_choose_x_value(&asks_nobody(), &game, 0, spell, LegalXValues::zero_through(0)), 0);
    assert_eq!(not_asked(&game), vec![format!("NotAsked: P0 ChooseXValue for Fixture Blaze ({spell}): 0, {ONE_LEGAL_ANSWER}")]);
}

/// One type of mana in the pool for the generic part: one split.
#[test]
fn a_generic_split_with_one_type_of_mana_is_answered_unasked() {
    let mut game = setup_two_player_game();
    let spell = put_in_hand(&mut game, card_of_type("Fixture Sorcery", CardType::Sorcery), 0);
    let cost = ManaCost::build(&[ManaType::Red], 1);
    let split = ask_choose_generic_mana_allocation(&asks_nobody(), &game, 0, spell, &cost, &[(ManaType::Red, 2)], 1);
    assert_eq!(split.get(&ManaType::Red), Some(&1));
    assert_not_asked(&game, 0, "GenericManaAllocation", ONE_LEGAL_ANSWER);
}

/// CR 702.19b: two power against a blocker owed two leaves nothing to trample over.
#[test]
fn a_trample_split_with_nothing_left_over_is_answered_unasked() {
    let mut game = setup_two_player_game();
    let attacker = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let blocker = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let (blocked, over) =
        ask_choose_trample_damage_assignment(&asks_nobody(), &game, 0, attacker, &[blocker], DamageTarget::Player(1), 2, &[2], None);
    assert_eq!((blocked, over), (vec![(blocker, 2)], 0));
    assert_eq!(
        not_asked(&game),
        vec![format!(
            "NotAsked: P0 AssignTrampleDamage for Grizzly Bears ({attacker}): 2 to Grizzly Bears ({blocker}), 0 to P1, {ONE_LEGAL_ANSWER}"
        )]
    );
}

/// CR 615.7: "the next 3 damage" against sources of 1 and 2 prevents all of it.
#[test]
fn a_next_damage_split_with_nothing_spare_is_answered_unasked() {
    let mut game = setup_two_player_game();
    let shield = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let a = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let b = put_on_battlefield(&mut game, grizzly_bears(), 1);
    assert_eq!(ask_allocate_next_damage(&asks_nobody(), &game, 0, shield, 3, &[(a, 1), (b, 2)]), vec![1, 2]);
    assert_not_asked(&game, 0, "AllocateNextDamage", ONE_LEGAL_ANSWER);
}

#[test]
fn a_sacrifice_taking_every_candidate_is_answered_unasked() {
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let b = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let spell = put_in_hand(&mut game, card_of_type("Fixture Offering", CardType::Sorcery), 0);
    assert_eq!(ask_choose_sacrifice_for_cost(&asks_nobody(), &game, 0, spell, 2, &[a, b]), vec![a, b]);
    assert_not_asked(&game, 0, "ChooseSacrificeForCost", ONE_LEGAL_ANSWER);
}

#[test]
fn a_copy_with_one_creature_to_copy_is_answered_unasked() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let spell = put_in_hand(&mut game, card_of_type("Fixture Shape", CardType::Instant), 0);
    assert_eq!(ask_choose_copy_source(&asks_nobody(), &game, 0, spell, &[bears]), bears);
    assert_not_asked(&game, 0, "ChooseCopySource", ONE_LEGAL_ANSWER);
}

#[test]
fn a_source_of_damage_with_one_candidate_is_answered_unasked() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let spell = put_in_hand(&mut game, card_of_type("Fixture Circle", CardType::Instant), 0);
    let candidate = game.object_ref(bears).unwrap();
    assert_eq!(ask_choose_damage_source(&asks_nobody(), &game, 0, spell, &[candidate]), candidate);
    assert_not_asked(&game, 0, "ChooseDamageSource", ONE_LEGAL_ANSWER);
}

/// "An opponent of your choice" in a two-player game.
#[test]
fn an_opponent_of_your_choice_with_one_opponent_is_answered_unasked() {
    let mut game = setup_two_player_game();
    let entering = put_in_hand(&mut game, grizzly_bears(), 0);
    assert_eq!(ask_choose_entering_controller(&asks_nobody(), &game, 0, entering, &[1]), 1);
    assert_not_asked(&game, 0, "ChooseEnteringController", ONE_LEGAL_ANSWER);
}

/// CR 508.4 with one player to attack.
#[test]
fn an_entering_attacker_with_one_thing_to_attack_is_answered_unasked() {
    let mut game = setup_two_player_game();
    let entering = put_in_hand(&mut game, grizzly_bears(), 0);
    let attacked = ask_choose_entering_defender(&asks_nobody(), &game, 0, entering, &[AttackTarget::Player(1)]);
    assert_eq!(attacked, AttackTarget::Player(1));
    assert_not_asked(&game, 0, "ChooseEnteringDefender", ONE_LEGAL_ANSWER);
}

/// Two candidates are a question: asked, and not recorded.
#[test]
fn a_choice_with_two_answers_is_asked_and_not_recorded() {
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let b = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let spell = put_in_hand(&mut game, card_of_type("Fixture Shape", CardType::Instant), 0);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::ChooseCopySource { source: spell }, vec![1]);
    assert_eq!(ask_choose_copy_source(&dp, &game, 0, spell, &[a, b]), b);
    assert!(not_asked(&game).is_empty());
}

// ---------------------------------------------------------------------------
// The audits: agreeing where the gate's leg is there, panicking where it is not
// ---------------------------------------------------------------------------

/// Rest in Peace on the battlefield, so a destroyed creature is exiled.
fn rest_in_peace_board() -> (GameState, mtgsim::types::ids::ObjectId, mtgsim::types::ids::ObjectId) {
    let mut game = setup_two_player_game();
    let peace = put_on_battlefield(&mut game, rest_in_peace(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.enable_audit();
    (game, peace, bears)
}

#[test]
fn the_replacement_gates_audit_agrees_over_a_printed_source() {
    let (mut game, _, bears) = rest_in_peace_board();
    game.execute_action(GameAction::Destroy { object: bears }, &test_ctx()).unwrap();
    assert_eq!(game.objects[&bears].zone, Zone::Exile);
}

#[test]
#[should_panic(expected = "the replacement gather's gate")]
fn the_replacement_gates_audit_panics_when_a_printed_leg_is_missing() {
    let (mut game, peace, bears) = rest_in_peace_board();
    game.replacement_ability_sources.remove(&peace);
    let _ = game.execute_action(GameAction::Destroy { object: bears }, &test_ctx());
}

/// Sigarda beside P0's creature, and P1's effect sacrificing it.
fn sigarda_board() -> (GameState, mtgsim::types::ids::ObjectId, mtgsim::types::ids::ObjectId, mtgsim::types::ids::ObjectId) {
    let mut game = setup_two_player_game();
    let sigarda = put_on_battlefield(&mut game, sigarda_host_of_herons(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let edict = put_in_hand(&mut game, card_of_type("Fixture Edict", CardType::Sorcery), 1);
    game.enable_audit();
    (game, sigarda, bears, edict)
}

fn sacrificed(object: mtgsim::types::ids::ObjectId) -> GameAction {
    GameAction::ZoneChange { object, from: Zone::Battlefield, to: Zone::Graveyard, act: ZoneChangeAct::Sacrificed }
}

#[test]
fn the_restriction_gates_audit_agrees_over_a_printed_source() {
    let (mut game, _, bears, edict) = sigarda_board();
    let ctx = effect_ctx(&game, edict);
    let _ = game.execute_action(sacrificed(bears), &ctx);
    assert_eq!(game.objects[&bears].zone, Zone::Battlefield, "Sigarda: an opponent's effect can't make P0 sacrifice it");
}

#[test]
#[should_panic(expected = "the restriction sweep's gate")]
fn the_restriction_gates_audit_panics_when_a_printed_leg_is_missing() {
    let (mut game, sigarda, bears, edict) = sigarda_board();
    game.restriction_ability_sources.remove(&sigarda);
    let ctx = effect_ctx(&game, edict);
    let _ = game.execute_action(sacrificed(bears), &ctx);
}

/// Thalia, and a noncreature spell in hand whose cost she raises.
fn thalia_board() -> (GameState, mtgsim::types::ids::ObjectId, mtgsim::types::ids::ObjectId) {
    let mut game = setup_two_player_game();
    let thalia = put_on_battlefield(&mut game, thalia_guardian_of_thraben(), 1);
    let spell = put_in_hand(&mut game, card_of_type("Fixture Sorcery", CardType::Sorcery), 0);
    game.enable_audit();
    (game, thalia, spell)
}

#[test]
fn the_cost_gates_audit_agrees_over_a_printed_source() {
    let (game, thalia, spell) = thalia_board();
    let found = cost_modifications_for(&game, spell);
    assert_eq!(found.iter().map(|m| m.source).collect::<Vec<_>>(), vec![thalia]);
}

#[test]
#[should_panic(expected = "the cost sweep's gate")]
fn the_cost_gates_audit_panics_when_a_printed_leg_is_missing() {
    let (mut game, thalia, spell) = thalia_board();
    game.cost_modification_ability_sources.remove(&thalia);
    cost_modifications_for(&game, spell);
}

#[test]
fn the_control_sweeps_audit_agrees_with_no_control_changing_effect() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    game.enable_audit();
    game.execute_action(GameAction::Tap { object: bears }, &test_ctx()).unwrap();
}

/// A controller that changed by some route other than layer 2: the sweep's
/// gate skips the walk, and the audit walks it.
#[test]
#[should_panic(expected = "the control sweep's gate")]
fn the_control_sweeps_audit_panics_when_a_controller_changed_unseen() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    game.battlefield.get_mut(&bears).unwrap().announced_controller = 1;
    game.enable_audit();
    let _ = game.execute_action(GameAction::Tap { object: bears }, &test_ctx());
}
