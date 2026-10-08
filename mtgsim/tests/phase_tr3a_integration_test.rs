//! Phase TR-3a — the delayed-trigger registry, with Final Fortune
//! (`triggers-architecture.md` §3.9, §4.6, §12).
//!
//! 1. Item 222: a copy or grant row that names its objects puts only those
//!    objects in front of the dispatcher, and only for a window of a kind
//!    their triggered abilities read.

use std::sync::Arc;

use mtgsim::cards::authoring::{triggered_ability, whenever};
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, Layer};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{forest, put_on_battlefield, registered, setup_two_player_game, test_ctx, vanilla_creature};
use mtgsim::types::effects::{AmountExpr, Duration, Effect, EffectRecipient, ObjectSet, Primitive};
use mtgsim::types::ids::new_ability_id;
use mtgsim::types::triggers::{TriggerEvent, TriggerSubject};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

/// The dispatcher's two work counters: windows past its gate, and the
/// candidates those windows asked.
fn dispatch_work(game: &GameState) -> (u64, u64) {
    let work = game.diagnostics.trigger_dispatch();
    (work.windows, work.candidates)
}

// ---------------------------------------------------------------------------
// 1. Item 222
// ---------------------------------------------------------------------------

/// Item 222: a grant to one creature of "whenever this creature becomes
/// tapped" opened the gate for every permanent, at every window. The row
/// names its object, so the dispatcher reads that object off the row, and
/// only for a window carrying a kind the granted trigger reads: an untap
/// passes no gate, and the creature's own tap asks it alone of the eight
/// permanents. The grant still triggers.
#[test]
fn a_named_grant_asks_only_its_object_and_only_for_its_kinds() {
    let mut game = setup_two_player_game();
    let carrier = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    for _ in 0..6 {
        put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    }
    let land = put_on_battlefield(&mut game, forest(), 0);
    let mut granted = triggered_ability(
        "Whenever this creature becomes tapped, you gain 1 life.",
        whenever(TriggerEvent::BecomesTapped { subject: TriggerSubject::ThisObject }, gain_one()),
    );
    granted.id = new_ability_id();
    game.continuous_effects.add(ContinuousEffect {
        duration: Duration::Indefinite,
        affected_objects: ObjectSet::Fixed(vec![carrier]),
        ..registered(carrier, Layer::Layer6Ability, 100, EffectModification::GrantAbility(Arc::new(granted)))
    });
    game.execute_action(GameAction::Tap { object: land }, &test_ctx()).unwrap();
    let before = dispatch_work(&game);

    game.execute_action(GameAction::Untap { object: land }, &test_ctx()).unwrap();
    assert_eq!(dispatch_work(&game), before, "an untap is no kind the granted trigger reads");

    game.execute_action(GameAction::Tap { object: carrier }, &test_ctx()).unwrap();
    let after = dispatch_work(&game);
    assert_eq!((after.0 - before.0, after.1 - before.1), (1, 1), "one window, one candidate: the carrier");
    assert_eq!(game.pending_triggers.len(), 1, "and the grant triggered");
    assert_eq!(game.pending_triggers[0].origin.source(), carrier);
}
