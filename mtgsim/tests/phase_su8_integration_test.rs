//! SU-8, what happened, from the trace (`setup-architecture.md` §7c): the
//! sink's lines read back, the "can't" a CR 616.1 iteration met, and the why
//! of an event and of the triggered abilities asked about it.

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_rc_cards::master_biomancer;
use mtgsim::engine::actions::{ActionContext, ZoneChangeCause};
use mtgsim::test_support::{install_trace, put_in_graveyard, put_on_battlefield, setup_two_player_game};
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
