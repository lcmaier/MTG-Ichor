//! `orders-the-cr-hands-a-player`: CR 601.2h's payment order and CR 613.7m's
//! timestamps, asked only where the orders' outcomes differ
//! (`choices-architecture.md#decision-6--when-the-engine-asks-an-order-the-cr-hands-a-player`).
//!
//! **Every board is cast from hand with mana its lands made**, since a staged
//! fixture cannot check a question (`engine::costs` asks it at CR 601.2h, on
//! the cast path), and writing the pool directly would skip the write the
//! layers read it through.

use mtgsim::cards::fixtures::{momentous_fall_fixture, omnath_locus_of_mana_fixture};
use mtgsim::cards::printings::grizzly_bears;
use mtgsim::oracle::characteristics::{get_effective_power, get_effective_toughness};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{fill_library, place_forest, put_in_hand, put_on_battlefield, setup_two_player_game, test_dp};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

/// `n` Forests tapped for {G} each, through the mana ability's own path.
fn float_green(game: &mut GameState, n: usize) {
    for _ in 0..n {
        let (forest, ability) = place_forest(game, 0);
        game.activate_mana_ability(0, forest, ability, &test_dp()).expect("an untapped Forest taps for mana");
    }
}

fn power_and_toughness(game: &GameState, id: ObjectId) -> (Option<i32>, Option<i32>) {
    (get_effective_power(game, id), get_effective_toughness(game, id))
}

/// The script's next answer: `creature` pays CR 601.2h's sacrifice.
fn sacrifices(answers: &ScriptedDecisionProvider, creature: ObjectId) {
    answers.expect_choice(
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id: ObjectId::UNASSIGNED, count: 1 },
        vec![ChoiceOption::Object(creature)],
    );
}

// ---------------------------------------------------------------------------
// What the order is an order of: a characteristic read off the pool
// ---------------------------------------------------------------------------

/// A static ability reads the pool as a walk input: every write moves what
/// the creature is, adding mana and spending it, with no frame served stale
/// (the memo's audit reads each frame again in a debug build).
#[test]
fn a_power_read_off_the_pool_moves_with_the_pool() {
    let mut game = setup_two_player_game();
    let omnath = put_on_battlefield(&mut game, omnath_locus_of_mana_fixture(), 0);
    assert_eq!(power_and_toughness(&game, omnath), (Some(1), Some(1)));

    float_green(&mut game, 2);
    assert_eq!(power_and_toughness(&game, omnath), (Some(3), Some(3)), "two unspent green mana");

    let bears = put_in_hand(&mut game, grizzly_bears(), 0);
    game.cast_spell(0, bears, &ManaWindowStop::new(test_dp())).expect("{1}{G} from exactly its cost");
    assert_eq!(game.players[0].mana_pool.unspent(ManaType::Green), 0);
    assert_eq!(power_and_toughness(&game, omnath), (Some(1), Some(1)), "spent");
}

/// Mana a player keeps is read until the step's end empties the pool (CR
/// 106.4, 500.5), which is a write like any other.
#[test]
fn a_pool_emptying_as_a_step_ends_is_read_too() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    fill_library(&mut game, 1, 5);
    let omnath = put_on_battlefield(&mut game, omnath_locus_of_mana_fixture(), 0);
    float_green(&mut game, 3);
    assert_eq!(power_and_toughness(&game, omnath), (Some(4), Some(4)));

    game.advance_turn(&test_dp()).expect("the phase ends");
    assert_eq!(power_and_toughness(&game, omnath), (Some(1), Some(1)), "the pool emptied");
}

/// A pool is its own player's: "you" is the ability's controller, so an
/// opponent's mana is no green mana you have.
#[test]
fn another_players_pool_is_not_yours() {
    let mut game = setup_two_player_game();
    let theirs = put_on_battlefield(&mut game, omnath_locus_of_mana_fixture(), 1);
    float_green(&mut game, 2);
    assert_eq!(power_and_toughness(&game, theirs), (Some(1), Some(1)));
}

// ---------------------------------------------------------------------------
// "The sacrificed creature": what a payment used, read back
// ---------------------------------------------------------------------------

/// The spell reads the creature its cost sacrificed, as it last existed: two
/// cards and 2 life for Grizzly Bears, and the record names the existence the
/// payment ended (CR 400.7), not the card now in the graveyard.
// COVERS-PARTIAL: ATOM-601.2h-003
#[test]
fn the_spell_reads_the_creature_its_cost_sacrificed() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let fall = put_in_hand(&mut game, momentous_fall_fixture(), 0);
    float_green(&mut game, 4);
    let existence = game.object_ref(bears).expect("on the battlefield");

    game.cast_spell(0, fall, &ManaWindowStop::new(test_dp())).expect("one creature, so nothing to choose");
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Graveyard, "the cost is paid");
    let paid_with = &game.stack_entries[&fall].cost_choices.paid_with;
    assert_eq!(paid_with.len(), 1);
    assert_eq!(paid_with[0].used.object(), Some(existence), "as the payment left it");

    let hand_before = game.players[0].hand.len();
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");
    assert_eq!(game.players[0].hand.len(), hand_before + 2, "its power");
    assert_eq!(game.players[0].life_total, 22, "its toughness");
}

/// Two creatures: which one pays is CR 601.2h's choice, and the spell reads
/// that one and not the other.
#[test]
fn the_creature_chosen_is_the_creature_read() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let omnath = put_on_battlefield(&mut game, omnath_locus_of_mana_fixture(), 0);
    let fall = put_in_hand(&mut game, momentous_fall_fixture(), 0);
    float_green(&mut game, 4);

    let answers = ScriptedDecisionProvider::new();
    sacrifices(&answers, bears);
    game.cast_spell(0, fall, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");

    let hand_before = game.players[0].hand.len();
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");
    assert_eq!(game.players[0].hand.len(), hand_before + 2, "the Bears' power, not Omnath's");
    assert_eq!(game.get_object(omnath).unwrap().zone, Zone::Battlefield);
}
