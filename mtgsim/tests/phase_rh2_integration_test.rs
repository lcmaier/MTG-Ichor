//! RH-2: the batch's members, decided together (`replacement-architecture.md`
//! §2b.6, §2b.8).
//!
//! Two facts a batch decides before it performs anything, and the boards that
//! showed each read at the wrong moment:
//! - L7, which permanents a player's leaving takes (CR 800.4a): what they own
//!   leaves the game with them and what they still control is exiled, so a
//!   source on either side is looked back at (CR 603.10a);
//! - item 248, a destruction's move (CR 701.8a), decided with the destruction
//!   against the board before any member of its batch has performed (CR 704.3,
//!   614.4).

use std::sync::Arc;

use mtgsim::cards::authoring::{another, leaves_the_battlefield, triggered_ability, whenever};
use mtgsim::cards::phase_rb_cards::kalitas_traitor_of_ghet;
use mtgsim::engine::actions::GameAction;
use mtgsim::events::event::{CounterSubject, GameEvent};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    effect_ctx, put_on_battlefield, put_on_battlefield_under, setup_game, setup_two_player_game, static_ability,
    test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Condition, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerFact,
    PlayerRef, PlayerSet, Primitive,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use mtgsim::types::zones::{Zone, ZoneChangePattern};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

fn a_creature() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Creature)
}

/// The triggers `source` has waiting to be put on the stack.
fn pending_from(game: &GameState, source: ObjectId) -> usize {
    game.pending_triggers.iter().filter(|t| t.origin.source() == source).count()
}

// ---------------------------------------------------------------------------
// L7 — what a player's leaving takes (CR 800.4a), looked back at (CR 603.10a)
// ---------------------------------------------------------------------------

/// A 1/1 whose grant lasts while its controller is in the game and controls a
/// creature: "As long as you control a creature, creatures your opponents
/// control have 'Whenever another creature leaves the battlefield, you gain 1
/// life.'"
///
/// The condition is what makes the leaving visible before the move: a layer
/// condition names only players still in the game, so the grant of a source
/// the leaving player controls ends at the loss, before CR 800.4a's fourth
/// clause exiles the source.
fn granter_while_you_control_a_creature() -> Arc<CardData> {
    let mut granted = triggered_ability("", whenever(leaves_the_battlefield(another(a_creature())), gain_one()));
    granted.id = new_ability_id();
    CardDataBuilder::new("Fixture: grants while you control a creature")
        .card_type(CardType::Creature)
        .power_toughness(1, 1)
        .ability(static_ability(Effect::Conditional(
            Condition::Player { players: PlayerSet::You, fact: PlayerFact::ControlsPermanent(a_creature()) },
            Box::new(Effect::Atom(
                Primitive::GrantAbility(Arc::new(granted), Duration::WhileSourceOnBattlefield),
                EffectRecipient::FilteredPermanents(ObjectFilter::And(
                    Box::new(a_creature()),
                    Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
                )),
            )),
        )))
        .build()
}

/// L7's controlled side: a source the leaving player controls by default and
/// does not own is exiled by CR 800.4a's fourth clause, which is part of the
/// leaving, so the carrier looks back to the grant it had before the leaving
/// began, not to the lists after the loss ended the grant.
#[test]
fn a_source_the_leaving_player_controls_and_does_not_own_is_looked_back_at() {
    let mut game = setup_game(4);
    let carrier = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    // Bribery's board: player 2's card, put onto the battlefield under player 1.
    let source = put_on_battlefield_under(&mut game, granter_while_you_control_a_creature(), 2, 1);

    game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).unwrap();

    assert_eq!(game.get_object(source).unwrap().zone, Zone::Exile, "CR 800.4a: what they still control is exiled");
    assert_eq!(
        pending_from(&game, carrier),
        1,
        "the carrier had the grant immediately before the leaving, so it sees the source leave"
    );
}

/// L7's owned side: a source the leaving player owns and another player
/// controls leaves the game with its owner (CR 800.4a's first clause, 603.6c).
#[test]
fn a_source_the_leaving_player_owns_and_does_not_control_is_looked_back_at() {
    let mut game = setup_game(4);
    let carrier = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let source = put_on_battlefield_under(&mut game, granter_while_you_control_a_creature(), 1, 2);

    game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).unwrap();

    assert!(game.get_object(source).is_err(), "CR 800.4a: what they own leaves the game");
    assert_eq!(pending_from(&game, carrier), 1, "the carrier had the grant immediately before the leaving");
}

// ---------------------------------------------------------------------------
// Item 248 — a destruction's move, decided with the destruction
// ---------------------------------------------------------------------------

/// The tokens `player` controls.
fn tokens_of(game: &GameState, player: PlayerId) -> usize {
    game.battlefield
        .iter()
        .filter(|(id, entry)| entry.controller == player && game.objects.get(id).is_some_and(|o| o.is_token))
        .count()
}

/// Mark `id` with damage equal to its toughness, for the next state-based
/// check to destroy (CR 704.5g).
fn mark_lethal(game: &mut GameState, id: ObjectId, toughness: u32) {
    game.battlefield.get_mut(&id).unwrap().damage_marked = toughness;
}

/// Kalitas's ruling, with Kalitas `first` or second in the battlefield's
/// order: player 0's Kalitas and player 1's 2/2 die in one state-based check.
fn kalitas_dies_beside_a_creature(kalitas_first: bool) -> (GameState, ObjectId, ObjectId) {
    let mut game = setup_two_player_game();
    let (kalitas, victim) = if kalitas_first {
        let kalitas = put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0);
        (kalitas, put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1))
    } else {
        let victim = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
        (put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0), victim)
    };
    mark_lethal(&mut game, kalitas, 4);
    mark_lethal(&mut game, victim, 2);

    game.check_state_based_actions(&test_dp()).unwrap();
    (game, kalitas, victim)
}

// RULING: Kalitas, Traitor of Ghet #3 - "If Kalitas dies at the same time as creatures your opponents control, those creature cards will be exiled and you'll get that many Zombies."
#[test]
fn kalitas_placed_first_still_exiles_the_creature_dying_beside_it() {
    let (game, kalitas, victim) = kalitas_dies_beside_a_creature(true);

    assert_eq!(game.get_object(kalitas).unwrap().zone, Zone::Graveyard, "Kalitas does not replace its own death");
    assert_eq!(game.get_object(victim).unwrap().zone, Zone::Exile, "decided against the board before Kalitas left");
    assert_eq!(tokens_of(&game, 0), 1, "and Kalitas's controller gets the Zombie");
}

// RULING: Kalitas, Traitor of Ghet #3 - "If Kalitas dies at the same time as creatures your opponents control, those creature cards will be exiled and you'll get that many Zombies."
#[test]
fn kalitas_placed_second_exiles_the_creature_dying_beside_it() {
    let (game, kalitas, victim) = kalitas_dies_beside_a_creature(false);

    assert_eq!(game.get_object(kalitas).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.get_object(victim).unwrap().zone, Zone::Exile);
    assert_eq!(tokens_of(&game, 0), 1);
}

/// A destroy-all over Kalitas and two of the opponent's creatures (CR
/// 608.2f's one event): both cards are exiled and two Zombies made, and the
/// Zombies are made after every death, never between two of them (CR 615.5's
/// "immediately afterward" is after the event, and the event is the wipe).
// RULING: Kalitas, Traitor of Ghet #3 - "If Kalitas dies at the same time as creatures your opponents control, those creature cards will be exiled and you'll get that many Zombies."
#[test]
fn a_wipe_that_takes_kalitas_exiles_every_opposing_creature_and_makes_the_zombies_after() {
    let mut game = setup_two_player_game();
    let kalitas = put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0);
    let victims = [
        put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1),
        put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1),
    ];
    let wrath = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);

    let wipe = [kalitas, victims[0], victims[1]].map(|object| GameAction::Destroy { object }).to_vec();
    game.execute_actions(wipe, &effect_ctx(&game, wrath)).unwrap();

    assert_eq!(game.get_object(kalitas).unwrap().zone, Zone::Graveyard);
    for victim in victims {
        assert_eq!(game.get_object(victim).unwrap().zone, Zone::Exile, "each opposing creature card is exiled");
    }
    assert_eq!(tokens_of(&game, 0), 2, "and each makes a Zombie");

    let events: Vec<GameEvent> = game.recorded_events().events().cloned().collect();
    let last_death = events.iter().rposition(|e| matches!(e, GameEvent::ZoneChange { .. }));
    let first_zombie = events.iter().position(|e| matches!(e, GameEvent::TokenCreated { .. }));
    assert!(last_death < first_zombie, "every death before any Zombie: {last_death:?}, {first_zombie:?}");
}

/// Two players' Kalitases die in one check: each is an opponent's creature to
/// the other, so each exiles the other and each player gets a Zombie. No
/// order of the batch can answer this one, which is why the move is decided
/// with the destruction rather than the batch reordered.
// RULING: Kalitas, Traitor of Ghet #3 - "If Kalitas dies at the same time as creatures your opponents control, those creature cards will be exiled and you'll get that many Zombies."
#[test]
fn two_players_kalitases_dying_together_exile_each_other() {
    let mut game = setup_two_player_game();
    let first = put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0);
    let second = put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 1);
    mark_lethal(&mut game, first, 4);
    mark_lethal(&mut game, second, 4);

    game.check_state_based_actions(&test_dp()).unwrap();

    assert_eq!(game.get_object(first).unwrap().zone, Zone::Exile);
    assert_eq!(game.get_object(second).unwrap().zone, Zone::Exile);
    assert_eq!((tokens_of(&game, 0), tokens_of(&game, 1)), (1, 1), "one Zombie each");
}

/// A destruction replaced has no move to decide: the shield counter (CR
/// 122.1c) replaces the destruction, so the creature stays and Kalitas, which
/// watches the move, never sees one.
#[test]
fn a_destruction_a_shield_counter_replaced_proposes_no_move() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, kalitas_traitor_of_ghet(), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let source = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    game.execute_action(
        GameAction::AddCounters { subject: CounterSubject::Object(bear), counter: CounterType::Shield, n: 1, by: 1 },
        &test_ctx(),
    )
    .unwrap();

    game.execute_action(GameAction::Destroy { object: bear }, &effect_ctx(&game, source)).unwrap();

    assert!(game.battlefield.contains_key(&bear));
    assert_eq!(game.battlefield[&bear].counter_count(CounterType::Shield), 0);
    assert_eq!(tokens_of(&game, 0), 0);
    assert!(!game.recorded_events().events().any(|e| matches!(e, GameEvent::ZoneChange { .. })));
}

// ---------------------------------------------------------------------------
// §2b.11's other outer event: CR 800.4a's exiles, one event
// ---------------------------------------------------------------------------

/// "If a permanent would be put into exile from the battlefield, put it into
/// its owner's graveyard instead." A fixture: the replacement that watches
/// CR 800.4a's fourth clause.
fn exile_to_graveyard_watcher() -> Arc<CardData> {
    CardDataBuilder::new("Fixture: exiles go to the graveyard")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::ZoneChange {
                change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Exile), act: None },
                object: None,
            },
            ObjectSet::battlefield_filter(ObjectFilter::All),
            Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Graveyard }),
        )))))
        .build()
}

/// Player 1 leaves controlling two permanents they do not own, the watcher
/// and a bystander, `watcher_first` or second in the battlefield's order.
fn the_leaving_player_controls_a_watcher_and_a_bystander(watcher_first: bool) -> (GameState, ObjectId, ObjectId) {
    let mut game = setup_game(4);
    let (watcher, bystander) = if watcher_first {
        let watcher = put_on_battlefield_under(&mut game, exile_to_graveyard_watcher(), 2, 1);
        (watcher, put_on_battlefield_under(&mut game, vanilla_creature(2, 2, &[]), 3, 1))
    } else {
        let bystander = put_on_battlefield_under(&mut game, vanilla_creature(2, 2, &[]), 3, 1);
        (put_on_battlefield_under(&mut game, exile_to_graveyard_watcher(), 2, 1), bystander)
    };

    game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).unwrap();
    (game, watcher, bystander)
}

/// CR 800.4a's fourth clause exiles everything the leaving player still
/// controls as one event, so the watcher, there before the event (CR 614.4),
/// replaces every exile in it, its own included, whichever goes first.
#[test]
fn the_leaving_players_exiles_are_one_event_with_the_watcher_first() {
    let (game, watcher, bystander) = the_leaving_player_controls_a_watcher_and_a_bystander(true);

    assert_eq!(game.get_object(watcher).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.get_object(bystander).unwrap().zone, Zone::Graveyard, "decided against the board with the watcher");
}

#[test]
fn the_leaving_players_exiles_are_one_event_with_the_watcher_second() {
    let (game, watcher, bystander) = the_leaving_player_controls_a_watcher_and_a_bystander(false);

    assert_eq!(game.get_object(watcher).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.get_object(bystander).unwrap().zone, Zone::Graveyard);
}
