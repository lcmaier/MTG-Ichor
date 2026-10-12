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
use mtgsim::engine::actions::GameAction;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_on_battlefield, put_on_battlefield_under, setup_game, static_ability, test_ctx, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Condition, Duration, Effect, EffectRecipient, ObjectFilter, PlayerFact, PlayerRef, PlayerSet,
    Primitive,
};
use mtgsim::types::ids::{new_ability_id, ObjectId};
use mtgsim::types::zones::Zone;

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
