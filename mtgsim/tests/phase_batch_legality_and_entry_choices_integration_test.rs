//! A6e's cleanup, `batch-legality-and-entry-choices`: what one event does to
//! the objects it names, and the choices a permanent's entry makes.

use std::sync::Arc;

use mtgsim::cards::authoring::triggers::{counters_removed_from, triggered_ability, whenever};
use mtgsim::engine::actions::GameAction;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_on_battlefield, put_on_battlefield_under, setup_two_player_game, test_ctx, test_dp, vanilla_creature};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{AmountExpr, CounterType, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId};
use mtgsim::types::triggers::TriggerSubject;
use mtgsim::types::zones::Zone;

fn add_counters(game: &mut GameState, object: ObjectId, counter: CounterType, n: u32) {
    game.execute_action(GameAction::AddCounters { subject: ObjectOrPlayerId::Object(object), counter, n, by: 0 }, &test_ctx())
        .expect("on the battlefield");
}

// ---------------------------------------------------------------------------
// A trigger's filter reads an object its own event moved as the event found it
// ---------------------------------------------------------------------------

/// "Whenever one or more counters are removed from a permanent you control,
/// you gain 1 life", Cloudsculpt Armorer's trigger with a gain for its seek.
fn gains_when_counters_leave_your_permanent() -> Arc<CardData> {
    let arm = counters_removed_from(TriggerSubject::Filter(ObjectFilter::ByController(PlayerRef::You)), None).once_per_object();
    CardDataBuilder::new("Fixture Counter Warden")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(arm, Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller))))
        .build()
}

/// A 1/1 with a +1/+1 counter and two -1/-1 counters is 0/0. One state-based
/// check removes a counter of each kind (CR 704.5q) and puts it into its
/// graveyard (CR 704.5f) as one event (CR 704.3), so the counters come off a
/// permanent its controller controls, and the trigger reads it as the check
/// found it. The creature is player 1's, controlled by player 0, so the card
/// in player 1's graveyard is player 1's to ask about (CR 108.4a) and the two
/// reads disagree.
#[test]
fn a_counter_trigger_reads_a_creature_its_check_also_killed_as_the_check_found_it() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, gains_when_counters_leave_your_permanent(), 0);
    let soldier = put_on_battlefield_under(&mut game, vanilla_creature(1, 1, &[]), 1, 0);
    add_counters(&mut game, soldier, CounterType::PlusOnePlusOne, 1);
    add_counters(&mut game, soldier, CounterType::MinusOneMinusOne, 2);

    game.perform_sba_and_triggers(&test_dp()).expect("the check and its triggers");
    assert_eq!(game.get_object(soldier).expect("its card").zone, Zone::Graveyard, "the check killed it");
    assert_eq!(game.stack.len(), 1, "one permanent lost counters, so the warden triggered once");
    let life = game.players[0].life_total;
    game.resolve_top_of_stack(&test_dp()).expect("the trigger resolves");
    assert_eq!(game.players[0].life_total, life + 1);
}
