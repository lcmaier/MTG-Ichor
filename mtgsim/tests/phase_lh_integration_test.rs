//! Phase LH-1 — the Aura host becomes addressable
//! (`layers-architecture.md` §13a).
//!
//! Holy Strength is the card; `AffectedSet::AttachedToSource` is what it
//! consumes. Every board below is built from the registered card rather than
//! a fixture Aura, because the claim being tested is that a *real* Aura's
//! static ability reaches its host through the layer walk — the fixtures in
//! `test_support` (Pacifism, "Test Bond") carry no ability and could not tell
//! a working row from a missing one.

use mtgsim::cards::phase_lh_cards::holy_strength;
use mtgsim::engine::actions::{DestructionSource, GameAction, ZoneChangeCause};
use mtgsim::engine::resolve::ResolvedTarget;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::{
    get_effective_controller, get_effective_power, get_effective_toughness,
};
use mtgsim::state::game_state::{GameState, StackEntry};
use mtgsim::test_support::{
    attach, put_on_battlefield, setup_two_player_game, test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::effects::Effect;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::zones::Zone;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pt(game: &GameState, id: ObjectId) -> (i32, i32) {
    (
        get_effective_power(game, id).expect("has power"),
        get_effective_toughness(game, id).expect("has toughness"),
    )
}

/// Holy Strength on the stack targeting `target`, as `cast_spell` leaves it
/// after CR 601.2c: a permanent spell with no spell ability carries an empty
/// `Sequence`, and the Aura's target lives in `chosen_targets`.
fn aura_on_stack_targeting(game: &mut GameState, controller: PlayerId, target: ObjectId) -> ObjectId {
    let obj = GameObject::new(holy_strength(), controller, Zone::Stack);
    let id = obj.id;
    game.add_object(obj);
    game.stack.push(id);
    game.set_stack_entry(StackEntry {
        object_id: id,
        controller,
        chosen_targets: vec![ResolvedTarget::Object(target)],
        chosen_modes: Vec::new(),
        x_value: None,
        effect: Effect::Sequence(Vec::new()),
        is_spell: true,
        chosen_alternative_cost: None,
        additional_costs_paid: Vec::new(),
        cast_from: Some(Zone::Hand),
        ability_identity: None,
    });
    id
}

fn destroy(game: &mut GameState, id: ObjectId) {
    let source = mtgsim::types::ids::new_object_id();
    game.execute_action(
        GameAction::Destroy { object: id, source: DestructionSource::Effect(source) },
        &test_ctx(),
    )
    .expect("it is destroyed");
}

/// Every zone change of `id`, as `(from, to, cause)`.
fn moves_of(game: &GameState, id: ObjectId) -> Vec<(Zone, Zone, ZoneChangeCause)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::ZoneChange { object_id, from, to, cause, .. } if *object_id == id => {
                Some((*from, *to, *cause))
            }
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// CR 303.4m — "enchanted creature" is whatever the Aura is attached to
// ---------------------------------------------------------------------------

/// The row itself. Asked *before* the attach as well as after: the first query
/// caches a frame at the pre-attach epoch, so a writer of `attached_to` that
/// skipped its bump would serve 2/2 here in release and panic in debug.
#[test]
fn test_holy_strength_gives_its_host_plus_one_plus_two() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let other = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let aura = put_on_battlefield(&mut game, holy_strength(), 0);

    assert_eq!(pt(&game, bears), (2, 2), "unattached, the Aura names nothing");

    attach(&mut game, aura, bears);
    assert_eq!(pt(&game, bears), (3, 4));
    assert_eq!(pt(&game, other), (2, 2), "only the enchanted creature");
}

/// The bonus goes with the Aura, not with the creature it first met.
#[test]
fn test_the_bonus_leaves_when_the_aura_does() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let aura = put_on_battlefield(&mut game, holy_strength(), 0);
    attach(&mut game, aura, bears);
    assert_eq!(pt(&game, bears), (3, 4));

    destroy(&mut game, aura);

    assert_eq!(pt(&game, bears), (2, 2));
    assert!(
        game.battlefield[&bears].attached_by.is_empty(),
        "the host's back-pointer is cleaned up as the Aura leaves"
    );
}

// ---------------------------------------------------------------------------
// CR 303.4 / 608.3c — an Aura spell enters attached to its target
// ---------------------------------------------------------------------------

/// The target is an opponent's creature on purpose: CR 303.4e's third clause
/// — the caster controls the Aura, whoever controls what it enchants — is the
/// same board, and the bonus reaches across the table.
// COVERS: ATOM-303.4-001, ATOM-608.3c-001, ATOM-303.4e-003
#[test]
fn test_an_aura_spell_resolves_attached_to_its_target() {
    let mut game = setup_two_player_game();
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let aura = aura_on_stack_targeting(&mut game, 0, theirs);

    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    assert_eq!(game.get_object(aura).unwrap().zone, Zone::Battlefield);
    assert_eq!(game.battlefield[&aura].attached_to, Some(theirs));
    assert!(game.battlefield[&theirs].attached_by.contains(&aura));
    assert_eq!(get_effective_controller(&game, aura), Some(0), "CR 303.4e: the caster's");
    assert_eq!(get_effective_controller(&game, theirs), Some(1), "and the creature stays P1's");
    assert_eq!(pt(&game, theirs), (3, 4));
    assert_eq!(
        moves_of(&game, aura),
        vec![(Zone::Stack, Zone::Battlefield, ZoneChangeCause::Resolved)]
    );
}

// ---------------------------------------------------------------------------
// CR 303.4c / 704.5m — the Aura follows its host into the graveyard
// ---------------------------------------------------------------------------

// COVERS: ATOM-303.4c-002
#[test]
fn test_the_aura_dies_with_its_host() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let aura = aura_on_stack_targeting(&mut game, 0, bears);
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    destroy(&mut game, bears);
    assert_eq!(
        game.battlefield[&aura].attached_to,
        None,
        "the host's departure unattaches it before any SBA looks"
    );

    let performed = game.check_state_based_actions(&test_dp()).unwrap();
    assert!(performed, "CR 704.5m has something to do");
    assert!(!game.battlefield.contains_key(&aura));
    assert!(
        game.players[0].graveyard.contains(&aura),
        "CR 303.4c: to its *owner's* graveyard, not the host's controller's"
    );
    assert_eq!(
        moves_of(&game, aura).last().copied(),
        Some((Zone::Battlefield, Zone::Graveyard, ZoneChangeCause::AuraSba))
    );
}
