//! A6e's cleanup, `batch-legality-and-entry-choices`: what one event does to
//! the objects it names, and the choices a permanent's entry makes.

use std::sync::Arc;

use mtgsim::cards::authoring::triggers::{counters_removed_from, triggered_ability, whenever};
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeAct};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_graveyard, put_on_battlefield, put_on_battlefield_under, setup_two_player_game, static_ability, test_ctx, test_dp,
    vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerRef, Primitive,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, ObjectOrPlayerId};
use mtgsim::types::replacement::{AuxiliaryMove, EventPattern, PerChosen, ReplacementDef, Rewrite};
use mtgsim::types::triggers::TriggerSubject;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;

fn add_counters(game: &mut GameState, object: ObjectId, counter: CounterType, n: u32) {
    game.execute_action(GameAction::AddCounters { subject: ObjectOrPlayerId::Object(object), counter, n, by: 0 }, &test_ctx())
        .expect("on the battlefield");
}

fn counters(game: &GameState, id: ObjectId, kind: CounterType) -> u32 {
    game.battlefield.get(&id).map_or(0, |entry| entry.counter_count(kind))
}

/// Devour, "as this enters, you may sacrifice any number of creatures. It
/// enters with `per` +1/+1 counters for each" (CR 702.82a).
fn devour(per: PerChosen) -> Effect {
    Effect::Replacement(Box::new(ReplacementDef::new(
        EventPattern::EnterBattlefield { cast: None },
        ObjectSet::SourceOnly,
        Rewrite::EnterAfterMoving(AuxiliaryMove {
            from: Zone::Battlefield,
            filter: ObjectFilter::And(
                Box::new(ObjectFilter::ByType(CardType::Creature)),
                Box::new(ObjectFilter::ByController(PlayerRef::You)),
            ),
            to: Zone::Graveyard,
            act: ZoneChangeAct::Sacrificed,
            up_to: None,
            per_chosen: Some((CounterType::PlusOnePlusOne, per)),
        }),
    )))
}

fn devourer(name: &str, per: PerChosen) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .power_toughness(1, 1)
        .ability(static_ability(devour(per)))
        .build()
}

/// Enter `entering` from its graveyard, answering devour's prompt with `picks`.
fn reanimate_devouring(game: &mut GameState, entering: ObjectId, picks: Vec<usize>) {
    let dp = test_dp();
    dp.expect_pick_n(ChoiceKind::ChooseAuxiliaryZoneChange { entering, source: entering, to: Zone::Graveyard }, picks);
    game.change_zone(entering, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&dp)).expect("it enters");
    assert!(dp.is_empty(), "devour asked once");
}

/// "Creatures have devour `n`", a Layer 6 grant.
fn grants_devour_to_creatures(n: u32) -> Arc<CardData> {
    let mut granted = static_ability(devour(PerChosen::Fixed(n)));
    granted.id = new_ability_id();
    CardDataBuilder::new("Fixture Devour Banner")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::GrantAbility(Arc::new(granted), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(ObjectFilter::ByType(CardType::Creature)),
        )))
        .build()
}

// ---------------------------------------------------------------------------
// A granted devour is the entering creature's
// ---------------------------------------------------------------------------

/// Devour granted by player 1's enchantment is still the entering creature's
/// own ability, so CR 614.13a's "you" is its controller, player 0 (CR 109.5):
/// player 0 chooses among player 0's creatures, and player 1's is no
/// candidate.
#[test]
fn a_granted_devour_is_chosen_by_the_entering_creatures_controller() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grants_devour_to_creatures(1), 1);
    let mine = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let entering = put_in_graveyard(&mut game, vanilla_creature(1, 1, &[]), 0);

    reanimate_devouring(&mut game, entering, vec![0]);

    assert_eq!(game.get_object(mine).expect("its card").zone, Zone::Graveyard, "player 0 sacrificed their own creature");
    assert!(game.battlefield.contains_key(&theirs), "player 1's creature was never a candidate");
    assert_eq!(counters(&game, entering, CounterType::PlusOnePlusOne), 1);
}

// ---------------------------------------------------------------------------
// Devour by a count
// ---------------------------------------------------------------------------

/// Thromok the Insatiable's "devour X, where X is the number of creatures
/// devoured this way": its ruling's three creatures give nine counters.
#[test]
fn devour_by_the_count_gives_the_count_squared() {
    let mut game = setup_two_player_game();
    for _ in 0..3 {
        put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    }
    let gorger = put_in_graveyard(&mut game, devourer("Fixture Squaring Gorger", PerChosen::Count), 0);

    reanimate_devouring(&mut game, gorger, vec![0, 1, 2]);

    assert_eq!(counters(&game, gorger, CounterType::PlusOnePlusOne), 9);
    assert_eq!(game.players[0].graveyard.len(), 3, "the three it devoured");
}

/// Two creatures give four (the ruling's second example), so the count is
/// read per entry rather than fixed when the ability is written.
#[test]
fn devour_by_the_count_reads_this_entrys_count() {
    let mut game = setup_two_player_game();
    for _ in 0..3 {
        put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    }
    let gorger = put_in_graveyard(&mut game, devourer("Fixture Squaring Gorger", PerChosen::Count), 0);

    reanimate_devouring(&mut game, gorger, vec![0, 2]);

    assert_eq!(counters(&game, gorger, CounterType::PlusOnePlusOne), 4);
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
