//! Phase TR-5b — counters (CR 122.6, 122.7, 704.5q).
//!
//! A put and a removal are two records, each naming the player who did it
//! where the rules name one, and the count it left (`now`). Counters a
//! permanent enters with are announced as put on it (CR 122.6). The trigger
//! arms read the two records once per counter, once per object, or once per
//! event, and CR 122.7's threshold is the ability's own intervening "if". CR
//! 704.5q's two removals are members of the state-based batch. An entry
//! template reads the X of the spell becoming the object (CR 107.3m).

use std::sync::Arc;

use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeAct};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::{CounterSubject, DamageTarget, GameEvent};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_graveyard, put_in_hand, put_on_battlefield, setup_two_player_game, test_ctx, test_dp,
    vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, CounterType, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, SelectionFilter,
    TargetCount,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::zones::Zone;
use mtgsim::ui::decision::DecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A nameless card with no abilities, to be the source of a fixture
/// resolution.
fn fixture_object() -> Arc<CardData> {
    CardDataBuilder::new("Fixture").build()
}

/// Resolve `effect` for `player` with `targets` already chosen, the way a
/// spell would.
fn resolve_targeting(
    game: &mut GameState,
    player: PlayerId,
    targets: Vec<ResolvedTarget>,
    effect: &Effect,
    dp: &dyn DecisionProvider,
) {
    let source = put_in_hand(game, fixture_object(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        generated_by: mtgsim::events::cause::GeneratedBy::Spell,
        targets: ChosenTargets::one(targets),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(effect, &ctx, dp).expect("resolving");
}

/// "Put `n` `kind` counters on target permanent", or with `Remove`, "remove
/// `n`", resolved by `player` on `target`.
fn counters_on_target(game: &mut GameState, player: PlayerId, target: ObjectId, primitive: Primitive) {
    let effect = Effect::Atom(
        primitive,
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)),
    );
    resolve_targeting(game, player, vec![ResolvedTarget::Object(target)], &effect, &test_dp());
}

fn put(kind: CounterType, n: u64) -> Primitive {
    Primitive::AddCounters { counter: kind, amount: AmountExpr::Fixed(n), by: PlayerRef::You }
}

fn remove(kind: CounterType, n: u64) -> Primitive {
    Primitive::RemoveCounters(kind, AmountExpr::Fixed(n))
}

/// Put a card from a graveyard onto the battlefield, through the chokepoint.
fn reanimate(game: &mut GameState, card: Arc<CardData>, player: PlayerId) -> ObjectId {
    let id = put_in_graveyard(game, card, player);
    game.change_zone(id, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&test_dp()))
        .expect("it enters");
    id
}

/// The counter records performed since `start`, in order.
fn counter_records(game: &GameState, start: usize) -> Vec<GameEvent> {
    game.recorded_events()
        .records_from(start)
        .iter()
        .map(|record| record.event.clone())
        .filter(|event| matches!(event, GameEvent::CountersPutOn { .. } | GameEvent::CountersRemovedFrom { .. }))
        .collect()
}

/// A planeswalker with printed loyalty 3 and nothing else.
fn loyalty_three() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Walker").card_type(CardType::Planeswalker).loyalty(3).build()
}

// ---------------------------------------------------------------------------
// The two records, their players and their counts
// ---------------------------------------------------------------------------

/// CR 608.2c: an instruction's controller puts the counters on, whoever
/// controls the permanent, and the record keeps the count they leave.
#[test]
fn a_put_names_the_instructions_controller_and_the_count_it_leaves() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    counters_on_target(&mut game, 0, bears, put(CounterType::PlusOnePlusOne, 1));
    let start = game.recorded_events().records().len();

    counters_on_target(&mut game, 1, bears, put(CounterType::PlusOnePlusOne, 2));

    assert_eq!(
        counter_records(&game, start),
        vec![GameEvent::CountersPutOn {
            subject: CounterSubject::Object(bears),
            counter_type: CounterType::PlusOnePlusOne,
            n: 2,
            by: 1,
            now: 3,
        }],
    );
}

/// CR 608.2c again, for a removal: Vampire Hexmage's "remove all counters"
/// is its controller's, as a put is. CR 701.2: as many as there are.
#[test]
fn a_removal_by_an_instruction_names_its_controller_and_takes_what_is_there() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    counters_on_target(&mut game, 0, bears, put(CounterType::PlusOnePlusOne, 3));
    let start = game.recorded_events().records().len();

    counters_on_target(&mut game, 1, bears, remove(CounterType::PlusOnePlusOne, 2));
    counters_on_target(&mut game, 1, bears, remove(CounterType::PlusOnePlusOne, 5));

    let removed = |n, now| GameEvent::CountersRemovedFrom {
        subject: CounterSubject::Object(bears),
        counter_type: CounterType::PlusOnePlusOne,
        n,
        by: Some(1),
        now,
    };
    assert_eq!(counter_records(&game, start), vec![removed(2, 1), removed(1, 0)]);
}

/// CR 120.3c words damage's loyalty removal with no player, where 120.3d
/// names wither's putter, so the removal has none.
#[test]
fn damage_to_a_planeswalker_removes_loyalty_that_no_player_removed() {
    let mut game = setup_two_player_game();
    let walker = reanimate(&mut game, loyalty_three(), 1);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let start = game.recorded_events().records().len();

    game.execute_action(
        GameAction::DealDamage {
            source: bears,
            target: DamageTarget::Object(walker),
            amount: 2,
            is_combat: false,
            unpreventable: false,
            source_lki: None,
        },
        &test_ctx(),
    )
    .expect("the damage is dealt");

    assert_eq!(
        counter_records(&game, start),
        vec![GameEvent::CountersRemovedFrom {
            subject: CounterSubject::Object(walker),
            counter_type: CounterType::Loyalty,
            n: 2,
            by: None,
            now: 1,
        }],
    );
}

/// CR 122.1d's replacement removes a stun counter in place of an untap, and
/// neither the rule nor the untap it replaces names a player.
#[test]
fn a_stun_counters_own_replacement_removes_it_with_no_player() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    counters_on_target(&mut game, 0, bears, put(CounterType::Stun, 1));
    game.battlefield.get_mut(&bears).expect("on the battlefield").tapped = true;
    let start = game.recorded_events().records().len();

    game.execute_action(GameAction::Untap { object: bears }, &test_ctx()).expect("the untap is replaced");

    assert_eq!(
        counter_records(&game, start),
        vec![GameEvent::CountersRemovedFrom {
            subject: CounterSubject::Object(bears),
            counter_type: CounterType::Stun,
            n: 1,
            by: None,
            now: 0,
        }],
    );
}
