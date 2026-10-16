//! TR-5a — the binding's roles (`triggers-architecture.md` §12, TR-5a).
//!
//! 1. Item 267: a permanent whose controller changes is removed from combat
//!    (CR 506.4), and an attacker whose blocker is taken stays blocked (CR
//!    509.1h).

use std::sync::Arc;

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lg_cards::act_of_treason;
use mtgsim::cards::phase_tr4d_cards::ray_of_command;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    fill_library, put_in_graveyard, put_in_hand, put_on_battlefield, set_active_player, set_attacking, set_blocked_by,
    set_blocking, setup_game, setup_two_player_game, test_dp,
};
use mtgsim::types::effects::EffectRecipient;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn stocked(players: usize) -> GameState {
    let mut game = if players == 2 { setup_two_player_game() } else { setup_game(players) };
    for p in 0..players {
        fill_library(&mut game, p, 10);
    }
    game
}

fn controller(game: &GameState, id: ObjectId) -> Option<PlayerId> {
    mtgsim::oracle::characteristics::get_effective_controller(game, id)
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

/// `card` cast by `player` from exactly `pool`, each instance of "target"
/// answered with the next of `targets`.
fn cast_at(game: &mut GameState, player: PlayerId, card: Arc<CardData>, pool: &[(ManaType, u64)], targets: &[ChoiceOption]) -> ObjectId {
    let aim = ScriptedDecisionProvider::new();
    for target in targets {
        aim.expect_choice(
            ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
            vec![target.clone()],
        );
    }
    let id = put_in_hand(game, card, player);
    for &(t, n) in pool {
        game.players[player].mana_pool.add(t, n);
    }
    game.cast_spell(player, id, &ManaWindowStop::new(aim)).expect("castable from exactly its cost");
    id
}

/// Ray of Command cast by player 0 at `creature`, the one creature an
/// opponent controls, so no prompt asks for it; resolved.
fn ray_at(game: &mut GameState, creature: ObjectId) {
    cast_at(game, 0, ray_of_command(), &[(ManaType::Blue, 4)], &[]);
    game.resolve_top_of_stack(&test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(controller(game, creature), Some(0));
}

/// Act of Treason's resolution, `thief` taking `victim` until end of turn.
fn steal(game: &mut GameState, thief: PlayerId, victim: ObjectId) {
    let card = act_of_treason();
    let source = put_in_graveyard(game, card.clone(), thief);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: thief,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(victim)]),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
}

// ---------------------------------------------------------------------------
// 1. Item 267: a change of control removes the permanent from combat
// ---------------------------------------------------------------------------

/// Player 1's Grizzly Bears attack player 0, who takes them with Ray of
/// Command. CR 506.4 removes them from combat, so they deal no combat damage,
/// and least of all to the player who now controls them.
#[test]
fn a_stolen_attacker_is_removed_from_combat() {
    let mut game = stocked(2);
    set_active_player(&mut game, 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    set_attacking(&mut game, bears, 0);

    ray_at(&mut game, bears);
    assert!(game.battlefield[&bears].attacking.is_none(), "CR 506.4: its controller changed");
    game.process_combat_damage(&test_dp(), false).unwrap();
    assert_eq!(life(&game, 0), 20, "no combat damage, and none to its own controller");
}

/// Player 0's Grizzly Bears block player 1's attacker, and player 1 takes the
/// blocker. The blocker leaves combat (CR 506.4), and the attacker stays
/// blocked with nothing blocking it (CR 509.1h), so it deals no damage.
#[test]
fn an_attacker_whose_blocker_is_taken_stays_blocked() {
    let mut game = stocked(2);
    set_active_player(&mut game, 1);
    let attacker = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let blocker = put_on_battlefield(&mut game, grizzly_bears(), 0);
    set_attacking(&mut game, attacker, 0);
    set_blocked_by(&mut game, attacker, vec![blocker]);
    set_blocking(&mut game, blocker, vec![attacker]);

    steal(&mut game, 1, blocker);
    assert_eq!(controller(&game, blocker), Some(1));
    assert!(game.battlefield[&blocker].blocking.is_none(), "CR 506.4: the blocker left combat");
    let info = game.battlefield[&attacker].attacking.as_ref().expect("the attacker is still attacking");
    assert!(info.is_blocked && info.blocked_by.is_empty(), "CR 509.1h: blocked, by nothing");
    game.process_combat_damage(&test_dp(), false).unwrap();
    assert_eq!(life(&game, 0), 20, "a blocked creature with no blockers deals no damage");
}
