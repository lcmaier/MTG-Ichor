//! TR-4b — attachment's records (`triggers-architecture.md` §12, TR-4b).
//!
//! 1. CR 701.3d's four routes to becoming unattached, each announced as one
//!    `Unattached` record naming its route: an unattach, a move to another
//!    object, the attachment leaving and the host leaving.

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::engine::actions::GameAction;
use mtgsim::events::event::{GameEvent, UnattachRoute};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    aura_enchanting_your_creature, effect_ctx, equipment, put_on_battlefield, setup_game, setup_two_player_game, test_ctx,
};
use mtgsim::types::ids::{ObjectId, ObjectRef};

// ---------------------------------------------------------------------------
// Fixtures and helpers
// ---------------------------------------------------------------------------

fn destroy_all(game: &mut GameState, objects: &[ObjectId]) {
    let by = mtgsim::types::ids::new_object_id();
    let batch = objects.iter().map(|&object| GameAction::Destroy { object }).collect();
    game.execute_actions(batch, &effect_ctx(game, by)).unwrap();
}

fn attach(game: &mut GameState, attachment: ObjectId, host: ObjectId) {
    game.execute_action(GameAction::Attach { attachment, host }, &test_ctx()).unwrap();
}

/// Every attach and unattach the game recorded, in order.
fn attachment_records(game: &GameState) -> Vec<GameEvent> {
    game.recorded_events()
        .events()
        .filter(|e| matches!(e, GameEvent::Attached { .. } | GameEvent::Unattached { .. }))
        .cloned()
        .collect()
}

fn unattached(attachment: ObjectRef, former_host: ObjectRef, route: UnattachRoute) -> GameEvent {
    GameEvent::Unattached { attachment, former_host, route }
}

fn existence(game: &GameState, id: ObjectId) -> ObjectRef {
    game.object_ref(id).expect("in the store")
}

/// Grizzly Bears with an Equipment on it, for player 0.
fn equipped_bears(game: &mut GameState) -> (ObjectId, ObjectId) {
    let bears = put_on_battlefield(game, grizzly_bears(), 0);
    let sword = put_on_battlefield(game, equipment("Probe Sword"), 0);
    assert!(game.attach(sword, bears));
    (sword, bears)
}

// ---------------------------------------------------------------------------
// 1. The four routes (CR 701.3d)
// ---------------------------------------------------------------------------

/// Moving an Equipment to another creature unattaches it from the first,
/// announced ahead of the attach, in the one event.
#[test]
fn reattaching_announces_the_unattach_from_the_first_host_then_the_attach() {
    let mut game = setup_two_player_game();
    let (sword, bears) = equipped_bears(&mut game);
    let other = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let (sword_ref, bears_ref) = (existence(&game, sword), existence(&game, bears));
    game.record_events();

    attach(&mut game, sword, other);

    assert_eq!(
        attachment_records(&game),
        vec![
            unattached(sword_ref, bears_ref, UnattachRoute::AttachedElsewhere),
            GameEvent::Attached { attachment: sword, host: other, former_host: Some(bears) },
        ]
    );
    let batches: Vec<_> = game.recorded_events().records().iter().map(|r| r.stamp.batch).collect();
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], batches[1], "one event (CR 701.3c, 701.3d)");
}

/// CR 701.3b: attaching to the host it is already on does nothing, and
/// nothing becomes unattached either.
#[test]
fn reattaching_to_the_same_host_announces_nothing() {
    let mut game = setup_two_player_game();
    let (sword, bears) = equipped_bears(&mut game);
    game.record_events();

    attach(&mut game, sword, bears);

    assert!(attachment_records(&game).is_empty());
}

/// An Equipment leaving the battlefield becomes unattached as it leaves,
/// named as the existence that left (CR 400.7), after its own move.
#[test]
fn an_equipment_leaving_becomes_unattached_after_its_move() {
    let mut game = setup_two_player_game();
    let (sword, bears) = equipped_bears(&mut game);
    let (sword_ref, bears_ref) = (existence(&game, sword), existence(&game, bears));
    game.record_events();

    destroy_all(&mut game, &[sword]);

    assert_eq!(attachment_records(&game), vec![unattached(sword_ref, bears_ref, UnattachRoute::AttachmentLeft)]);
    let kinds: Vec<&str> = game
        .recorded_events()
        .events()
        .filter_map(|e| match e {
            GameEvent::ZoneChange { object_id, .. } if *object_id == sword => Some("move"),
            GameEvent::Unattached { .. } => Some("unattached"),
            _ => None,
        })
        .collect();
    assert_eq!(kinds, vec!["move", "unattached"]);
    assert_ne!(existence(&game, sword), sword_ref, "the record names the permanent, not the graveyard card");
}

/// A host leaving unattaches everything on it, in the order they were
/// attached, each from the host as it was.
#[test]
fn a_host_leaving_unattaches_everything_on_it() {
    let mut game = setup_two_player_game();
    let (sword, bears) = equipped_bears(&mut game);
    let aura = put_on_battlefield(&mut game, aura_enchanting_your_creature("Probe Aura"), 0);
    assert!(game.attach(aura, bears));
    let (sword_ref, aura_ref, bears_ref) = (existence(&game, sword), existence(&game, aura), existence(&game, bears));
    game.record_events();

    destroy_all(&mut game, &[bears]);

    assert_eq!(
        attachment_records(&game),
        vec![
            unattached(sword_ref, bears_ref, UnattachRoute::HostLeft),
            unattached(aura_ref, bears_ref, UnattachRoute::HostLeft),
        ]
    );
}

/// Both ends leaving in one event unattach once, by whichever moved first.
#[test]
fn an_equipment_and_its_host_leaving_together_unattach_once() {
    let mut game = setup_two_player_game();
    let (sword, bears) = equipped_bears(&mut game);
    let (sword_ref, bears_ref) = (existence(&game, sword), existence(&game, bears));
    game.record_events();

    destroy_all(&mut game, &[bears, sword]);

    assert_eq!(attachment_records(&game), vec![unattached(sword_ref, bears_ref, UnattachRoute::HostLeft)]);
}

/// CR 800.4a: a host leaving the game with its owner unattaches what another
/// player owns, which stays (CR 701.3d's "the object leaves the zone it was
/// in").
#[test]
fn a_host_leaving_the_game_unattaches_an_equipment_another_player_owns() {
    let mut game = setup_game(3);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let sword = put_on_battlefield(&mut game, equipment("Probe Sword"), 0);
    assert!(game.attach(sword, bears));
    let (sword_ref, bears_ref) = (existence(&game, sword), existence(&game, bears));
    game.record_events();

    game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).unwrap();

    assert!(game.battlefield.contains_key(&sword), "player 0's Equipment stays");
    assert_eq!(attachment_records(&game), vec![unattached(sword_ref, bears_ref, UnattachRoute::HostLeft)]);
}
