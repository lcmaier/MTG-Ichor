//! Everywhere — the registry's five-colour land — behaves as its text says.
//!
//! Three claims, each one a game can falsify: it taps for any one of the five
//! colours and only once; it enters untapped, because "tapped" is Overlord of
//! the Hauntwoods' creation instruction rather than the token's own text; and
//! CR 305.7 strips all five printed abilities under Blood Moon exactly as it
//! strips a dual's two. The last is the one that makes the land safe in the
//! pool: Blood Moon is in `PERFORMANCE_POOL`, and a five-type land is the
//! widest board CR 305.7's carve-out has had.

use mtgsim::cards::phase_ld_cards::blood_moon;
use mtgsim::cards::token_lands::everywhere;
use mtgsim::oracle::characteristics::get_effective_subtypes;
use mtgsim::oracle::mana_helpers::available_mana_sources;
use mtgsim::test_support::{put_in_hand, put_on_battlefield, setup_two_player_game, test_ctx};
use mtgsim::types::card_types::{LandType, Subtype};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;

/// The mana types Everywhere's abilities offer, sorted so the assertion does
/// not depend on ability order.
fn offered(game: &mtgsim::state::game_state::GameState, land: mtgsim::types::ids::ObjectId)
    -> Vec<(ManaType, mtgsim::types::ids::AbilityId)> {
    let mut out: Vec<_> = available_mana_sources(game, 0)
        .into_iter()
        .filter(|s| s.permanent_id == land)
        .map(|s| (s.produces, s.ability_id))
        .collect();
    out.sort_by_key(|(mt, _)| format!("{mt:?}"));
    out
}

#[test]
fn test_everywhere_offers_each_of_the_five_colours_and_taps_for_one() {
    let mut game = setup_two_player_game();
    let land = put_on_battlefield(&mut game, everywhere(), 0);

    let sources = offered(&game, land);
    let produces: Vec<ManaType> = sources.iter().map(|(mt, _)| *mt).collect();
    assert_eq!(
        produces,
        vec![
            ManaType::Black,
            ManaType::Blue,
            ManaType::Green,
            ManaType::Red,
            ManaType::White
        ],
        "one ability per basic land type, and nothing else"
    );

    // Tap it for black. One activation taps the land, so the other four
    // abilities are spent with it — it is one land, not five.
    let (_, black) = sources[0];
    game.activate_mana_ability(0, land, black, &test_ctx()).expect("black is offered");
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Black), 1);
    assert_eq!(game.players[0].mana_pool.total(), 1);
    assert!(game.battlefield.get(&land).unwrap().tapped);

    let (_, blue) = sources[1];
    assert!(
        game.activate_mana_ability(0, land, blue, &test_ctx()).is_err(),
        "a tapped land cannot pay {{T}} again"
    );
    assert!(offered(&game, land).is_empty(), "nothing left to offer once tapped");
}

/// "Create a tapped Everywhere token" is the Overlord's instruction, not the
/// token's text, so a land drop of it enters untapped (CR 110.5b).
#[test]
fn test_everywhere_enters_untapped_when_played() {
    let mut game = setup_two_player_game();
    let land = put_in_hand(&mut game, everywhere(), 0);

    game.play_land(0, land, Zone::Hand, &test_ctx()).expect("the land drop is legal");

    assert!(
        !game.battlefield.get(&land).unwrap().tapped,
        "Everywhere's own text says nothing about entering tapped"
    );
    assert_eq!(offered(&game, land).len(), 5, "and it is usable the turn it lands");
}

/// CR 305.7 on a five-type land: Blood Moon sets the subtypes to Mountain
/// alone, which strips the five printed abilities and grants the one
/// intrinsic `{T}: Add {R}`. Same rule as the dual-land test in
/// `phase_ld_integration_test.rs`; a wider board.
#[test]
fn test_blood_moon_makes_everywhere_a_mountain_that_taps_for_red_only() {
    let mut game = setup_two_player_game();
    let land = put_on_battlefield(&mut game, everywhere(), 0);
    let _moon = put_on_battlefield(&mut game, blood_moon(), 0);

    let subtypes = get_effective_subtypes(&game, land);
    assert_eq!(subtypes.len(), 1, "Mountain, and none of the other four");
    assert!(subtypes.contains(&Subtype::Land(LandType::Mountain)));

    let sources = offered(&game, land);
    assert_eq!(sources.len(), 1, "five printed abilities stripped, one intrinsic granted");
    let (produces, red) = sources[0];
    assert_eq!(produces, ManaType::Red);

    game.activate_mana_ability(0, land, red, &test_ctx())
        .expect("the intrinsic Mountain ability is activatable by the id enumerated");
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 1);
    assert_eq!(game.players[0].mana_pool.total(), 1);
}
