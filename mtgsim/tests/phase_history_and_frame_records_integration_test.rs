//! A6e's cleanup, `history-and-frame-records`: what a record keeps of an
//! object that is gone, and the history log's rows as every "this turn"
//! fact's one store (`plans/handoffs/item-6-audit.md`, pass 3's cleanup).

use mtgsim::cards::authoring::triggers::{leaves_the_battlefield, triggered_ability, whenever};
use mtgsim::cards::phase_lh_cards::holy_strength;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::events::event::{GameEvent, UnattachRoute};
use mtgsim::oracle::characteristics::get_effective_power;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_on_battlefield, setup_game, setup_two_player_game, test_dp, vanilla_creature};
use mtgsim::objects::card_data::CardDataBuilder;
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, ObjectFilter, Primitive};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::triggers::{EventRole, TriggerSubject};

// ---------------------------------------------------------------------------
// An unattach, each end as it left (CR 603.10c)
// ---------------------------------------------------------------------------

/// A 2/2 of `player`'s enchanted with Holy Strength, a 3/4 while it is.
fn enchanted_creature(game: &mut GameState, player: PlayerId) -> (ObjectId, ObjectId) {
    let host = put_on_battlefield(game, vanilla_creature(2, 2, &[]), player);
    let aura = put_on_battlefield(game, holy_strength(), player);
    game.execute_action(GameAction::Attach { attachment: aura, host }, &ActionContext::new(&test_dp())).unwrap();
    assert_eq!(get_effective_power(game, host), Some(3));
    (host, aura)
}

/// The one unattach recorded since record `since`.
fn the_unattach(game: &GameState, since: usize) -> GameEvent {
    let recorded = game.recorded_events();
    let mut unattaches = recorded.events().skip(since).filter(|e| matches!(e, GameEvent::Unattached { .. }));
    let unattach = unattaches.next().expect("an unattach").clone();
    assert!(unattaches.next().is_none(), "one unattach for one pair");
    unattach
}

/// The creature and its Aura destroyed in one event: the unattach keeps
/// each end as it left, the host a 3/4 under its Aura, though the Aura's
/// own departure is the one that unattached them.
#[test]
fn an_unattach_keeps_both_ends_when_both_leave_in_one_event() {
    let mut game = setup_two_player_game();
    let (host, aura) = enchanted_creature(&mut game, 0);
    let since = game.recorded_events().len();
    let destroy = |object| GameAction::Destroy { object };
    game.execute_actions(vec![destroy(aura), destroy(host)], &ActionContext::new(&test_dp())).unwrap();

    let unattach = the_unattach(&game, since);
    assert!(matches!(unattach, GameEvent::Unattached { route: UnattachRoute::AttachmentLeft, .. }), "{unattach:?}");
    assert!(unattach.frame_of(aura).is_some(), "the end whose departure unattached them");
    let host_as_it_left = unattach.frame_of(host).expect("the host, which left in the same event");
    assert_eq!(host_as_it_left.chars().power, Some(3), "as it was, enchanted");
}

/// CR 800.4a: a player leaves a three-seat game with an enchanted creature.
/// Both leave in one event, and the unattach keeps both.
#[test]
fn an_unattach_keeps_both_ends_when_their_owner_leaves_the_game() {
    let mut game = setup_game(3);
    let (host, aura) = enchanted_creature(&mut game, 1);
    let since = game.recorded_events().len();
    game.execute_action(GameAction::PlayerLoses { player: 1 }, &ActionContext::new(&test_dp())).unwrap();

    let unattach = the_unattach(&game, since);
    assert!(unattach.frame_of(aura).is_some() && unattach.frame_of(host).is_some(), "{unattach:?}");
}

/// Only the host leaves: the Aura stays on the battlefield, so the unattach
/// keeps the host alone, and the Aura is read as it is.
#[test]
fn an_unattach_keeps_no_frame_of_an_end_that_stays() {
    let mut game = setup_two_player_game();
    let (host, aura) = enchanted_creature(&mut game, 0);
    let since = game.recorded_events().len();
    game.execute_action(GameAction::Destroy { object: host }, &ActionContext::new(&test_dp())).unwrap();

    let unattach = the_unattach(&game, since);
    assert!(unattach.frame_of(host).is_some());
    assert!(unattach.frame_of(aura).is_none(), "{unattach:?}");
}

// ---------------------------------------------------------------------------
// The object a departure from the game names (CR 800.4a, 603.6c)
// ---------------------------------------------------------------------------

/// "Whenever a creature leaves the battlefield, you gain life equal to its
/// power": its power is the creature's as it last existed (CR 608.2h).
fn fixture_mourners_tithe() -> std::sync::Arc<mtgsim::objects::card_data::CardData> {
    let creature = TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature));
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringPower(EventRole::Object)), EffectRecipient::Controller);
    CardDataBuilder::new("Fixture Mourner's Tithe")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(leaves_the_battlefield(creature), gain)))
        .build()
}

/// Player 1 leaves a three-seat game with a 3/3, which leaves the
/// battlefield as it leaves the game (CR 603.6c). Player 0's watcher binds
/// that creature as it left, and gains its power.
#[test]
fn a_creature_that_left_the_game_is_its_triggers_object() {
    let mut game = setup_game(3);
    put_on_battlefield(&mut game, fixture_mourners_tithe(), 0);
    put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    game.execute_action(GameAction::PlayerLoses { player: 1 }, &ActionContext::new(&test_dp())).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&test_dp()).unwrap();
        game.perform_sba_and_triggers(&test_dp()).unwrap();
    }
    assert_eq!(game.players[0].life_total, 23);
}
