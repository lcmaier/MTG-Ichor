//! A6e's cleanup, `record-existences`: a record names each object it is
//! about by the existence the event was about (CR 400.7), so a trigger on it
//! remembers that existence and not the one its object became later in the
//! same event.

use std::sync::Arc;

use mtgsim::cards::authoring::triggers::{counters_removed_from, triggered_ability, whenever};
use mtgsim::engine::actions::GameAction;
use mtgsim::events::event::CounterSubject;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_on_battlefield, setup_two_player_game, test_ctx, test_dp, vanilla_creature};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{CounterType, Effect, EffectRecipient, ObjectFilter, Primitive};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::triggers::{EventRole, TriggerSubject};
use mtgsim::types::zones::Zone;

/// "Whenever a +1/+1 counter is removed from a creature, exile that
/// creature."
fn exiles_what_loses_a_counter() -> Arc<CardData> {
    let arm = counters_removed_from(TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature)), CounterType::PlusOnePlusOne);
    CardDataBuilder::new("Fixture Shedding Ward")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(arm, Effect::Atom(Primitive::Exile, EffectRecipient::Triggering(EventRole::Object)))))
        .build()
}

fn add_counters(game: &mut GameState, object: ObjectId, counter: CounterType, n: u32) {
    game.execute_action(GameAction::AddCounters { subject: CounterSubject::Object(object), counter, n, by: 0 }, &test_ctx())
        .expect("on the battlefield");
}

/// One state-based check removes a 2/2's +1/+1 and -1/-1 counters in pairs
/// (CR 704.5q) and puts it into its graveyard at 0 toughness (CR 704.5f), as
/// one event (CR 704.3). The removal is about the creature on the
/// battlefield, so "that creature" is that existence, which the move ended:
/// the trigger finds no creature to exile, and the card the move made stays
/// in the graveyard (CR 400.7).
#[test]
fn a_trigger_on_a_removal_remembers_the_creature_its_death_ended() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, exiles_what_loses_a_counter(), 0);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    add_counters(&mut game, bears, CounterType::PlusOnePlusOne, 1);
    add_counters(&mut game, bears, CounterType::MinusOneMinusOne, 3);

    game.perform_sba_and_triggers(&test_dp()).expect("the check and its triggers");
    assert_eq!(game.stack.len(), 1, "the removal triggered the ward");
    assert_eq!(game.get_object(bears).expect("its card").zone, Zone::Graveyard);
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&test_dp()).expect("the trigger resolves");
    }

    assert_eq!(game.get_object(bears).expect("its card").zone, Zone::Graveyard, "a new object, which the trigger never names");
}
