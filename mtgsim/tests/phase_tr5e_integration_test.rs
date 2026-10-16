//! TR-5e — the attack declaration (`triggers-architecture.md` §12, TR-5e).
//!
//! 1. The declaration's record names what each attacker attacks (CR 508.1b).

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::events::event::GameEvent;
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::{fill_library, put_on_battlefield, setup_game, setup_two_player_game};
use mtgsim::types::ids::ObjectId;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::display::format_event_log;

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

/// Player 0 declares `attacks`, each attacker against its player. The
/// declaration's options are each attacker against each opponent, the
/// attackers in the order they entered and the opponents in seat order.
fn declare(game: &mut GameState, attackers: &[ObjectId], attacks: &[(ObjectId, usize)]) {
    let opponents = game.players.len() - 1;
    let picks = attacks
        .iter()
        .map(|&(attacker, player)| {
            let at = attackers.iter().position(|&a| a == attacker).unwrap();
            at * opponents + (player - 1)
        })
        .collect();
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(StepType::DeclareAttackers) });
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::DeclareAttackers, picks);
    game.process_declare_attackers(&dp).unwrap();
}

/// The one declaration's record.
fn declared(game: &GameState) -> Vec<(ObjectId, AttackTarget)> {
    let records = game.recorded_events();
    let mut declarations = records.events().filter_map(|event| match event {
        GameEvent::AttackersDeclared { attackers } => Some(attackers.clone()),
        _ => None,
    });
    let attacks = declarations.next().expect("a declaration");
    assert!(declarations.next().is_none(), "one declaration");
    attacks
}

// ---------------------------------------------------------------------------
// 1. The record
// ---------------------------------------------------------------------------

/// Two creatures attacking two of three opponents: the record pairs each with
/// its own, and the log line says so.
#[test]
fn the_declaration_names_what_each_attacker_attacks() {
    let mut game = stocked(4);
    let first = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let second = put_on_battlefield(&mut game, grizzly_bears(), 0);
    declare(&mut game, &[first, second], &[(first, 1), (second, 3)]);

    assert_eq!(declared(&game), vec![(first, AttackTarget::Player(1)), (second, AttackTarget::Player(3))]);
    let log = format_event_log(&game);
    let line = format!("AttackersDeclared: [Grizzly Bears ({first}) -> P1, Grizzly Bears ({second}) -> P3]");
    assert!(log.contains(&line), "no {line:?} in {log:#?}");
}
