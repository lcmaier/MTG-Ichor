//! Phase LL — a card in a library or a hand is walked only when something
//! reads it (`layers-architecture.md` §13e).
//!
//! **Grist, the Hunger Tide first**, by its rulings: a static ability that
//! changes its own card everywhere but the battlefield, which applies only
//! because its card is a member of every pass wherever it is (§13e decision
//! 2). Every Grist here is `phase_ll_cards::grist_insect_clause`, its first
//! ability on its printed frame. Then the notes a card left out of the pass
//! is walked with, and item 182.
//!
//! **Two casts go through `cast_spell` from hand**, with exactly the spell's
//! cost in the pool and the provider under `ManaWindowStop`, as a shipped
//! client runs it: Grist under Thalia, and a creature card given flash.

use std::sync::Arc;

use mtgsim::cards::{alpha, phase_cm_cards, phase_lj_cards, phase_ll_cards, phase_rf_cards};
use mtgsim::engine::layers::types::{EffectModification, Layer};
use mtgsim::objects::card_data::CardData;
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_colors, get_effective_controller, get_effective_power,
    get_effective_toughness, has_keyword, has_subtype, has_type, is_creature,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_command_zone, put_in_exile, put_in_graveyard, put_in_hand, put_in_library, put_on_battlefield,
    put_spell_on_stack, registered, set_active_player, setup_two_player_game, vanilla_creature,
    RecordingDecisionProvider,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::effects::PlayerRef;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::ManaType;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Put exactly `pool` into the player's mana pool, cast `card` from hand, and
/// report whether it was cast — `phase_cm_integration_test`'s exact-pool
/// discipline, so a success means the locked total was `pool` and nothing else.
/// The provider is a shipped client's stack: `ManaWindowStop` over it.
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
    game.cast_spell(player, id, &ManaWindowStop::new(RecordingDecisionProvider::picking(0))).map(|_| id)
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

// ---------------------------------------------------------------------------
// The notes: a card in a library or a hand, walked alone (§13e decision 1)
// ---------------------------------------------------------------------------

/// Titania's Song strips Mycosynth Lattice's clause at layer 6, and the
/// clause colors every card off the battlefield at layer 5, while its ability
/// is still there. The note keeps the row as the pass had it at layer 5: a red
/// card in a library and one in a hand are colorless. Beside it, Teferi's
/// clause loses its layer-6 grant to the same strip, which applies first
/// (CR 613.8a), so a creature card in hand has no flash.
#[test]
fn test_titanias_song_leaves_lattices_colorless_line_standing_at_layer_5() {
    let mut game = setup_two_player_game();
    let lattice = put_on_battlefield(&mut game, phase_lj_cards::lattice_colorless_clause(), 0);
    put_on_battlefield(&mut game, phase_ll_cards::titanias_song_clause(), 1);
    let in_library = put_in_library(&mut game, alpha::lightning_bolt(), 0);
    let in_hand = put_in_hand(&mut game, alpha::lightning_bolt(), 1);
    assert!(get_effective_abilities(&game, lattice).is_empty(), "the Song strips the clause by the end of layer 6");
    assert!(get_effective_colors(&game, in_library).is_empty(), "the Bolt in a library is colorless");
    assert!(get_effective_colors(&game, in_hand).is_empty(), "the Bolt in a hand is colorless");

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase_lj_cards::teferi_flash_clause(), 0);
    let bear = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert!(has_keyword(&game, bear, KeywordFlag::Flash), "Teferi's clause grants flash");
    put_on_battlefield(&mut game, phase_ll_cards::titanias_song_clause(), 1);
    assert!(!has_keyword(&game, bear, KeywordFlag::Flash), "the Song strips the clause before its grant applies");
}

/// CR 109.5: a static ability's "you" is its source's controller when the row
/// applies. Teferi's clause under a layer-2 effect giving it to player 1
/// grants flash to player 1's creature cards and not player 0's, though
/// player 0 put it onto the battlefield and registered its row. The note
/// keeps "you" as the pass read it, and the row's own controller is the
/// answer that would be wrong.
#[test]
fn test_a_noted_row_reads_you_as_the_pass_did() {
    let mut game = setup_two_player_game();
    let clause = put_on_battlefield(&mut game, phase_lj_cards::teferi_flash_clause(), 0);
    let ours = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
    let theirs = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 1);
    assert!(has_keyword(&game, ours, KeywordFlag::Flash));
    assert!(!has_keyword(&game, theirs, KeywordFlag::Flash));

    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(registered(
        clause,
        Layer::Layer2Control,
        timestamp,
        EffectModification::SetController(PlayerRef::Player(1)),
    ));
    assert_eq!(get_effective_controller(&game, clause), Some(1));
    assert!(has_keyword(&game, theirs, KeywordFlag::Flash), "player 1 controls the clause now");
    assert!(!has_keyword(&game, ours, KeywordFlag::Flash), "so player 0's creature cards lose flash");
}

/// A static ability functioning from a hand (CR 113.6b) joins the pass
/// (§13e decision 2), so the pass sees Hollow Hands' strip reach it. Both
/// apply at layer 6, the grant waits on the strip (CR 613.8a), and the
/// creature loses the flying the card in hand gave it. Walked alone, read as
/// of the end of layer 5, the card would still grant.
#[test]
fn test_a_static_ability_in_a_hand_joins_the_pass() {
    let mut game = setup_two_player_game();
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    put_in_hand(&mut game, phase_ll_cards::pocket_griffin(), 0);
    assert!(has_keyword(&game, bear, KeywordFlag::Flying), "the Griffin grants flying from the hand");
    put_on_battlefield(&mut game, phase_rf_cards::hollow_hands(), 1);
    assert!(!has_keyword(&game, bear, KeywordFlag::Flying), "Hollow Hands strips the Griffin first");
}

/// CR 613.8 decided through a card in a hand, the printed case: Arcane
/// Adaptation's clause reaches creature cards off the battlefield, and Grist's
/// own effect makes Grist one, so the clause depends on it. Grist drawn after
/// the clause entered is the younger (CR 613.7d), and timestamp order alone
/// would apply the clause first and leave Grist without the type. Grist
/// joins the pass as its row's source, which is how the pass sees the
/// dependency.
#[test]
fn test_grist_in_a_hand_is_an_elf_under_arcane_adaptations_clause() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase_ll_cards::arcane_adaptation_elf_clause(), 0);
    let grist = put_in_hand(&mut game, phase_ll_cards::grist_insect_clause(), 0);
    assert!(has_subtype(&game, grist, &Subtype::Creature(CreatureType::Elf)), "the clause applied after Grist's own");
    assert!(has_subtype(&game, grist, &Subtype::Creature(CreatureType::Insect)));
}

/// "Anything that could search for or affect a creature or planeswalker card
/// in zones other than the battlefield could affect Grist": Teferi's clause
/// gives Grist in a hand flash.
#[test]
fn test_grist_in_a_hand_has_flash_under_teferis_clause() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase_lj_cards::teferi_flash_clause(), 0);
    let grist = put_in_hand(&mut game, phase_ll_cards::grist_insect_clause(), 0);
    assert!(has_keyword(&game, grist, KeywordFlag::Flash));
}

/// A dependency decided through cards in a library, which no printed card
/// makes: the Assassins' effect reaches artifact creature cards, and the
/// artificer's makes creature cards artifacts, so the Assassins' depends on
/// it. The guard keeps the libraries in the pass (§13e decision 4, (b)), and
/// a creature card there is an Artifact Assassin though the Assassins' effect
/// is the older. Left out, the pass would see no dependency and apply the
/// older first.
#[test]
fn test_the_guard_holds_a_library_two_effects_could_depend_through() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase_ll_cards::library_assassins(), 0);
    put_on_battlefield(&mut game, phase_ll_cards::library_artificer(), 1);
    let bear = put_in_library(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert!(has_type(&game, bear, CardType::Artifact));
    assert!(
        has_subtype(&game, bear, &Subtype::Creature(CreatureType::Assassin)),
        "the Assassins' effect waited on the artificer's"
    );
}

// ---------------------------------------------------------------------------
// Item 182: the cast-timing check reads the card, through the layers
// ---------------------------------------------------------------------------

/// CR 702.8a's flash, read through the layers: Teferi's clause gives a
/// creature card in its owner's hand flash, and the owner casts it on the
/// other player's turn through `cast_spell`. Without the clause the same cast
/// is refused at sorcery timing (CR 117.1a). On `main` the check read printed
/// flash, and the clause changed no game.
// COVERS: ATOM-702.8a-001
#[test]
fn test_a_creature_card_given_flash_is_cast_on_the_other_players_turn() {
    let board = |clause: bool| {
        let mut game = setup_two_player_game();
        if clause {
            put_on_battlefield(&mut game, phase_lj_cards::teferi_flash_clause(), 0);
        }
        set_active_player(&mut game, 1);
        game
    };

    let mut game = board(true);
    let cast = cast_from_pool(&mut game, 0, vanilla_creature(2, 2, &[]), &[(ManaType::Green, 2)]);
    assert!(cast.is_ok(), "flash lets player 0 cast on player 1's turn: {cast:?}");
    assert_eq!(game.players[0].mana_pool.total(), 0, "the whole pool is the cost");

    let mut game = board(false);
    let refused = cast_from_pool(&mut game, 0, vanilla_creature(2, 2, &[]), &[(ManaType::Green, 2)]);
    assert!(refused.is_err(), "without flash it waits for sorcery timing");
}
