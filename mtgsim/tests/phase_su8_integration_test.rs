//! SU-8, what happened, from the trace (`setup-architecture.md` §7c): the
//! sink's lines read back, the "can't" a CR 616.1 iteration met, and the why
//! of an event and of the triggered abilities asked about it.

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_rc_cards::master_biomancer;
use mtgsim::engine::actions::{ActionContext, ZoneChangeCause};
use mtgsim::state::trace::{FieldValue, RecordKind};
use mtgsim::test_support::{
    install_trace, put_in_graveyard, put_on_battlefield, setup_two_player_game, vanilla_creature,
};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::zones::Zone;
use mtgsim::ui::decision::ScriptedDecisionProvider;

/// Item 210: a debug build's trace is a release build's. Two Biomancers'
/// entry counters commute, so CR 616.1's prompt is suppressed, and the debug
/// build's check of that (`check_order_invariance`) gathers again over a
/// probe of the entry. The probe's CR 614.12 look-ahead wrote a `layer_walk`
/// the release build never writes: a release build writes four, and the
/// debug build wrote a fifth.
#[test]
fn a_debug_builds_order_check_writes_nothing_to_the_trace() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, master_biomancer(), 0);
    put_on_battlefield(&mut game, master_biomancer(), 0);
    let bears = put_in_graveyard(&mut game, grizzly_bears(), 0);
    let trace = install_trace(&mut game, "item 210");
    // No answer queued: a CR 616.1 prompt here fails the test.
    let dp = ScriptedDecisionProvider::new();
    game.change_zone(bears, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("the Bears return");

    let entering = trace.of_kind("layer_walk").into_iter().filter(|line| line.contains("\"membership\":\"entering\"")).count();
    assert_eq!(entering, 4, "the look-ahead walks a release build writes");
}

/// CR 614.17: a "can't" is checked ahead of every replacement effect and wins
/// (CR 101.2). An indestructible creature with lethal damage is not destroyed
/// (CR 702.12b), and the CR 616.1 iteration that met the "can't" names it:
/// which member, whose restriction, and which.
#[test]
fn the_iteration_a_cant_stopped_names_it() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Indestructible]), 0);
    game.battlefield.get_mut(&creature).unwrap().damage_marked = 3;
    let trace = install_trace(&mut game, "a can't, named");
    game.perform_sba_and_triggers(&ScriptedDecisionProvider::new()).unwrap();
    assert!(game.battlefield.contains_key(&creature), "indestructible: lethal damage destroys nothing");

    let records = trace.records();
    let stopped: Vec<&FieldValue> = records.iter().filter(|r| r.kind == RecordKind::Pipeline).flat_map(|r| r.items("prohibited")).collect();
    let [named] = stopped.as_slice() else { panic!("one member stopped, found {stopped:?}") };
    let field = |key: &str| named.get(key).cloned();
    assert_eq!(field("i"), Some(FieldValue::Number(0)), "the batch's one proposal, the destruction");
    assert_eq!(field("source"), Some(FieldValue::Number(creature.raw().into())));
    assert_eq!(field("by"), Some(FieldValue::Text("keyword".into())));
    assert_eq!(field("words"), Some(FieldValue::Text("indestructible".into())));
}
