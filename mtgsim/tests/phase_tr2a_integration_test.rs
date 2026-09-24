//! Phase TR-2a — the histories, the gates and each player
//! (`triggers-architecture.md` §12, TR-2a; §13's TR-2a row).
//!
//! **What a trigger remembers across a turn.** TR-1 matched one window at a
//! time. TR-2a gives the dispatcher a memory: every player's turn,
//! materialized record by record (`TurnSummary`), and the two once-per-turn
//! gates, each written by the instant its rule names. The tests read that
//! memory where the rules read it: at the trigger, at resolution, and a turn
//! later.
//!
//! The fixtures are invented and carry their own names (`engineering-
//! practices.md` §3). The printed cards were verified on Scryfall on
//! 2026-09-24.

use std::sync::Arc;

use mtgsim::cards::authoring::{enters, triggered_ability, whenever};
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{creature_with_ability, put_on_battlefield, setup_two_player_game, test_ctx, test_dp};
use mtgsim::types::effects::{AmountExpr, CounterType, Effect, EffectRecipient, PlayerRef, Primitive};
use mtgsim::types::triggers::{TriggerEvent, TriggerSubject};
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::decision::DecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A 1/1 creature fixture carrying one triggered ability.
fn watcher(name: &str, event: impl Into<TriggerEvent>, effect: Effect) -> Arc<CardData> {
    creature_with_ability(name, 1, 1, triggered_ability(whenever(event, effect)))
}

fn counter_on(recipient: EffectRecipient) -> Effect {
    Effect::Atom(
        Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You },
        recipient,
    )
}

fn place(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).expect("placing");
}

fn resolve_top(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.resolve_top_of_stack(dp).expect("resolving");
}

fn plus_ones(game: &GameState, id: mtgsim::types::ids::ObjectId) -> u32 {
    game.battlefield.get(&id).map_or(0, |e| e.counter_count(CounterType::PlusOnePlusOne))
}

// ---------------------------------------------------------------------------
// CR 113.7a, 400.7 — "this creature" is the ability's source, by identity
// ---------------------------------------------------------------------------

/// "Put a +1/+1 counter on this creature": the counter goes on the permanent
/// whose ability resolved, not on the ephemeral stack object.
#[test]
fn this_object_is_the_permanent_whose_ability_resolves() {
    let mut game = setup_two_player_game();
    let sapling =
        put_on_battlefield(&mut game, watcher("Self-Tending Sapling", enters(TriggerSubject::ThisObject), counter_on(EffectRecipient::ThisObject)), 0);
    place(&mut game, &test_dp());
    resolve_top(&mut game, &test_dp());
    assert_eq!(plus_ones(&game, sapling), 1);
}

/// CR 400.7 — flickered between its trigger and its resolution, the source
/// is a new object, and "this creature" finds nothing: the returned
/// permanent gets no counter from the first trigger.
#[test]
fn this_object_finds_nothing_once_its_source_has_left_and_returned() {
    let mut game = setup_two_player_game();
    let sapling =
        put_on_battlefield(&mut game, watcher("Self-Tending Sapling", enters(TriggerSubject::ThisObject), counter_on(EffectRecipient::ThisObject)), 0);
    place(&mut game, &test_dp());
    assert_eq!(game.stack.len(), 1);
    game.change_zone(sapling, Zone::Exile, ZoneChangeCause::Exiled, &test_ctx()).unwrap();
    game.change_zone(sapling, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    assert_eq!(game.get_object(sapling).unwrap().zone, Zone::Battlefield, "it came back");

    // The first trigger resolves; the return's own trigger is still pending.
    resolve_top(&mut game, &test_dp());
    assert_eq!(plus_ones(&game, sapling), 0, "the returned permanent is not the object the ability is of");
}
