//! A4p integration tests: a seat that has left the game is not a target
//! (CR 800.4a, 601.2c, 608.2b).
//!
//! **Four seats, because the defect cannot exist at two.** A two-player
//! departure ends the game (CR 104.2a), so the window in which a player is not
//! in the game and the game continues opens only above two — which is where v1
//! lives. `num_players()` is the player vector's length and CR 800.4a never
//! shrinks it, so every arm that enumerated `0..num_players()` offered the
//! departed seat and every arm that counted `players.len()` counted it.
//!
//! The two tests are the two halves the arms disagreed about. `Player` was
//! caught at the end of CR 601.2c — offered, then refused, so the cast the
//! oracle had promised was rewound. `Any` was not caught at all:
//! `validate_any_target` never asked `in_game`, so "any target" damage
//! resolved against a player the game no longer has.
//!
//! Both cast from hand through `cast_spell` with exactly the spell's cost in
//! the pool and no lands: the claim is about what CR 601.2c *offers*, and a
//! staged `StackEntry` would answer none of it.

use mtgsim::cards::alpha::{ancestral_recall, lightning_bolt};
use mtgsim::engine::resolve::ResolvedTarget;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_hand, setup_game, stock_libraries, test_dp, RecordingDecisionProvider,
};
use mtgsim::types::ids::PlayerId;
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Player 3 loses to CR 704.5b — a draw from an empty library — and leaves.
///
/// **That reason rather than zero life**, because it is the one that leaves the
/// life total alone: player 3 stays on 20, so three damage landing on a seat
/// that is not in the game is visible as a life total rather than as a number
/// that was already wrong.
fn player_three_leaves(game: &mut GameState) {
    game.players[3].has_drawn_from_empty_library = true;
    game.check_state_based_actions(&test_dp())
        .expect("checking state-based actions");
    assert!(!game.in_game(3), "CR 704.5b, then CR 800.4a");
    assert!(game.result.is_none(), "four seats less one is still a game");
}

/// How many cards are in `player`'s hand.
fn hand_size(game: &GameState, player: PlayerId) -> usize {
    game.players[player].hand.len()
}

// ---------------------------------------------------------------------------
// "Any target" — the half nothing caught
// ---------------------------------------------------------------------------

#[test]
fn any_target_damage_does_not_resolve_against_a_seat_that_left_the_game() {
    // Lightning Bolt is cast while all four seats are in the game, so the
    // target is legal when it is chosen (CR 601.2c) and the question is what
    // CR 608.2b makes of it afterwards: "if all its targets... are now illegal,
    // the spell doesn't resolve". A player who has left the game is not a
    // player (CR 800.4a), so the target is illegal and the spell is countered
    // by game rules — but `validate_any_target` asked only that the index be in
    // range, so the three damage landed on a seat the game no longer had.
    let mut game = setup_game(4);
    let bolt = put_in_hand(&mut game, lightning_bolt(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    // The battlefield is empty, so "any target" offers exactly the four seats
    // and index 3 is player 3.
    let dp = RecordingDecisionProvider::picking(3);
    game.cast_spell(0, bolt, &dp).expect("{R} is in the pool and the seats are legal");
    assert_eq!(
        game.stack_entries[&bolt].chosen_targets[0].chosen,
        vec![ResolvedTarget::Player(3)],
        "the fourth seat, chosen while it was still a player",
    );

    player_three_leaves(&mut game);

    game.resolve_top_of_stack(&dp).expect("resolving the top of the stack");
    assert_eq!(
        game.players[3].life_total, 20,
        "CR 608.2b — the only target is illegal, so the spell does not resolve \
         and deals no damage to a player who is not in the game",
    );
    assert_eq!(
        game.get_object(bolt).expect("still a card").zone,
        Zone::Graveyard,
        "a spell countered by game rules goes to its owner's graveyard",
    );
    assert!(game.stack.is_empty(), "nothing is left on the stack");
}

// ---------------------------------------------------------------------------
// "Target player" — the half that was offered and then refused
// ---------------------------------------------------------------------------

#[test]
fn a_seat_that_left_the_game_is_not_among_the_players_a_cast_offers() {
    // `RecordingDecisionProvider::picking(3)` asks for the *fourth* option and
    // clamps to the last one offered, so what this test reads off the chosen
    // target is how many seats CR 601.2c put in front of the provider. Before
    // the fix the fourth option was player 3, and `validate_targets` then
    // refused it under CR 800.4a — a cast the oracle had offered, rewound at
    // the end of the announcement (`codebase-state.md` item 139's class).
    let mut game = setup_game(4);
    stock_libraries(&mut game, 5);
    player_three_leaves(&mut game);

    let recall = put_in_hand(&mut game, ancestral_recall(), 0);
    game.players[0].mana_pool.add(ManaType::Blue, 1);
    let before = hand_size(&game, 2);

    let dp = RecordingDecisionProvider::picking(3);
    game.cast_spell(0, recall, &dp)
        .expect("three seats are in the game, so CR 601.2c has a legal target");
    assert_eq!(
        game.stack_entries[&recall].chosen_targets[0].chosen,
        vec![ResolvedTarget::Player(2)],
        "the last seat offered is the last seat still in the game",
    );

    game.resolve_top_of_stack(&dp).expect("Ancestral Recall resolves");
    assert_eq!(hand_size(&game, 2), before + 3, "target player draws three cards");
    assert!(
        !game.in_game(3),
        "and the departed seat was never a candidate to draw them",
    );
}
