//! Phase LL — a card in a library or a hand is walked only when something
//! reads it (`layers-architecture.md` §13e).
//!
//! **Grist, the Hunger Tide first**, by its rulings: a static ability that
//! changes its own card everywhere but the battlefield, which applies only
//! because its card is a member of every pass wherever it is (§13e decision
//! 2). Every Grist here is `phase_ll_cards::grist_insect_clause`, its first
//! ability on its printed frame.

use std::sync::Arc;

use mtgsim::cards::{phase_cm_cards, phase_ll_cards};
use mtgsim::objects::card_data::CardData;
use mtgsim::oracle::characteristics::{get_effective_power, get_effective_toughness, has_subtype, has_type, is_creature};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_command_zone, put_in_exile, put_in_graveyard, put_in_hand, put_in_library, put_on_battlefield,
    put_spell_on_stack, setup_two_player_game, RecordingDecisionProvider,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaType;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Put exactly `pool` into the player's mana pool, cast `card` from hand, and
/// report whether it was cast — `phase_cm_integration_test`'s exact-pool
/// discipline, so a success means the locked total was `pool` and nothing else.
fn cast_from_pool(game: &mut GameState, player: PlayerId, card: Arc<CardData>, pool: &[(ManaType, u64)]) -> Result<ObjectId, String> {
    let id = put_in_hand(game, card, player);
    for t in [ManaType::White, ManaType::Blue, ManaType::Black, ManaType::Red, ManaType::Green, ManaType::Colorless] {
        let have = game.players[player].mana_pool.amount(t);
        if have > 0 {
            game.players[player].mana_pool.remove(t, have).unwrap();
        }
    }
    for &(t, n) in pool {
        game.players[player].mana_pool.add(t, n);
    }
    game.cast_spell(player, id, &RecordingDecisionProvider::picking(0)).map(|_| id)
}

/// Grist's ruling, as the oracle answers it: a 1/1 Insect creature, still a
/// planeswalker.
fn is_a_one_one_insect_creature(game: &GameState, grist: ObjectId) -> bool {
    is_creature(game, grist)
        && has_type(game, grist, CardType::Planeswalker)
        && has_subtype(game, grist, &Subtype::Creature(CreatureType::Insect))
        && get_effective_power(game, grist) == Some(1)
        && get_effective_toughness(game, grist) == Some(1)
}

// ---------------------------------------------------------------------------
// Grist, the Hunger Tide
// ---------------------------------------------------------------------------

/// "Anywhere but on the battlefield, Grist is a Legendary Planeswalker
/// Creature — Grist Insect. Once it enters the battlefield, it is no longer a
/// creature and is just a planeswalker." Six zones and the seventh, each its
/// own card, since a card in each would otherwise read one pass's answer.
///
/// On `main` the card was a creature in none of them: its row is `SourceOnly`,
/// which reaches a source only through its frame in the pass, and a card off
/// the battlefield was walked alone.
#[test]
fn test_grist_is_a_creature_card_everywhere_but_the_battlefield() {
    let mut game = setup_two_player_game();
    let zones: Vec<(&str, ObjectId)> = vec![
        ("hand", put_in_hand(&mut game, phase_ll_cards::grist_insect_clause(), 0)),
        ("library", put_in_library(&mut game, phase_ll_cards::grist_insect_clause(), 0)),
        ("graveyard", put_in_graveyard(&mut game, phase_ll_cards::grist_insect_clause(), 0)),
        ("exile", put_in_exile(&mut game, phase_ll_cards::grist_insect_clause(), 0)),
        ("command zone", put_in_command_zone(&mut game, phase_ll_cards::grist_insect_clause(), 0)),
        ("stack", put_spell_on_stack(&mut game, phase_ll_cards::grist_insect_clause(), 0)),
    ];
    for (zone, grist) in &zones {
        assert!(is_a_one_one_insect_creature(&game, *grist), "Grist in the {zone} is a 1/1 Insect creature");
    }

    let on_battlefield = put_on_battlefield(&mut game, phase_ll_cards::grist_insect_clause(), 0);
    assert!(!is_creature(&game, on_battlefield), "on the battlefield Grist is a planeswalker and nothing else");
    assert!(!has_subtype(&game, on_battlefield, &Subtype::Creature(CreatureType::Insect)));
    assert_eq!(get_effective_power(&game, on_battlefield), None);
}

/// Thalia's "noncreature spells cost {1} more to cast", and Grist's ruling
/// that it can be countered by Essence Scatter but not by Negate: on the stack
/// Grist is a creature spell, so Thalia does not tax it. Through `cast_spell`
/// from an exact pool, so the locked total is exactly {1}{B}{G}.
#[test]
fn test_grist_cast_under_thalia_is_a_creature_spell_and_pays_no_tax() {
    let board = || {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, phase_cm_cards::thalia_guardian_of_thraben(), 1);
        game
    };

    let mut game = board();
    let cast = cast_from_pool(&mut game, 0, phase_ll_cards::grist_insect_clause(), &[(ManaType::Black, 2), (ManaType::Green, 1)]);
    assert!(cast.is_ok(), "Grist is castable for {{1}}{{B}}{{G}} under Thalia: {cast:?}");
    assert_eq!(game.players[0].mana_pool.total(), 0, "the whole pool is the cost");
    assert!(is_creature(&game, cast.unwrap()), "a creature spell on the stack");

    let mut game = board();
    let short = cast_from_pool(&mut game, 0, phase_ll_cards::grist_insect_clause(), &[(ManaType::Black, 1), (ManaType::Green, 1)]);
    assert!(short.is_err(), "one mana short is still short");
}
