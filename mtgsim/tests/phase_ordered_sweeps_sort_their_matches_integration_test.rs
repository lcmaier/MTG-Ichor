//! `ordered-sweeps-sort-their-matches`: a battlefield sweep that keeps some
//! permanents asks each one in the map's order and sorts only what it keeps
//! (`GameState::battlefield_ordered_where`), where it sorted the whole
//! battlefield and then asked. The output is what filtering the sorted list
//! gives, so every decision list, batch and graveyard order is unchanged.
//!
//! The board puts six of P0's Grizzly Bears onto the battlefield in the
//! reverse of the order their objects were made, so their CR 613.7 timestamp
//! order is not their id order, and marks lethal damage on two.

use std::cell::{Cell, RefCell};

use mtgsim::cards::printings::grizzly_bears;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{install_trace, put_in_hand, setup_two_player_game, test_ctx, test_dp};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::zones::{Zone, ZoneChangeAct};

/// The board, and its bears in timestamp order.
fn six_bears_entered_newest_object_first() -> (GameState, Vec<ObjectId>) {
    let mut game = setup_two_player_game();
    let made: Vec<ObjectId> = (0..6).map(|_| put_in_hand(&mut game, grizzly_bears(), 0)).collect();
    for &bears in made.iter().rev() {
        game.change_zone(bears, Zone::Battlefield, ZoneChangeAct::Put, &test_ctx()).unwrap();
    }
    let entered: Vec<ObjectId> = made.into_iter().rev().collect();
    for &bears in [entered[1], entered[4]].iter() {
        game.battlefield.get_mut(&bears).unwrap().damage_marked = 2;
    }
    assert_eq!(game.battlefield_ids_ordered(), entered, "oldest permanent first, which is not id order");
    (game, entered)
}

fn ids(entries: Vec<(ObjectId, &mtgsim::state::battlefield::PermanentState)>) -> Vec<ObjectId> {
    entries.into_iter().map(|(id, _)| id).collect()
}

#[test]
fn an_ordered_sweep_keeps_what_the_sorted_sweep_keeps_in_its_order() {
    let (game, entered) = six_bears_entered_newest_object_first();
    let damaged = |_: ObjectId, entry: &mtgsim::state::battlefield::PermanentState| entry.damage_marked > 0;
    let swept: Vec<ObjectId> = game.battlefield_ordered().into_iter().filter(|&(id, entry)| damaged(id, entry)).map(|(id, _)| id).collect();
    assert_eq!(ids(game.battlefield_ordered_where(damaged)), swept);
    assert_eq!(swept, vec![entered[1], entered[4]]);
    assert_eq!(ids(game.battlefield_ordered_where(|_, _| true)), entered);
    assert!(game.battlefield_ordered_where(|_, _| false).is_empty());
}

/// CR 704.3's one event puts the two into the graveyard, an ordered zone, in
/// the batch's order, oldest first; the audit asks each sweep again and agrees.
#[test]
fn the_state_based_check_destroys_in_timestamp_order_and_its_audit_agrees() {
    let (mut game, entered) = six_bears_entered_newest_object_first();
    game.enable_audit();
    assert!(game.check_state_based_actions(&test_dp()).unwrap());
    assert_eq!(game.players[0].graveyard, vec![entered[1], entered[4]]);
}

/// A filter whose answer depends on what it was asked before: it keeps the
/// first six it is asked, so the audit's second asking keeps none.
#[test]
#[should_panic(expected = "an ordered sweep")]
fn the_ordered_sweeps_audit_panics_when_its_filter_remembers_what_it_asked() {
    let (mut game, _) = six_bears_entered_newest_object_first();
    game.enable_audit();
    let asked = Cell::new(0);
    game.battlefield_ordered_where(|_, _| {
        asked.set(asked.get() + 1);
        asked.get() <= 6
    });
}

/// A layer walk writes a trace record, so a traced game asks in timestamp order.
#[test]
fn a_traced_game_asks_an_ordered_sweep_oldest_first() {
    let (mut game, entered) = six_bears_entered_newest_object_first();
    let _trace = install_trace(&mut game, "an ordered sweep, traced");
    let asked = RefCell::new(Vec::new());
    game.battlefield_ordered_where(|id, _| {
        asked.borrow_mut().push(id);
        true
    });
    assert_eq!(asked.into_inner(), entered);
}
