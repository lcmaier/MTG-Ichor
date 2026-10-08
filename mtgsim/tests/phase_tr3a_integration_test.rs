//! Phase TR-3a — the delayed-trigger registry, with Final Fortune
//! (`triggers-architecture.md` §3.9, §4.6, §12).
//!
//! 1. Item 222: a copy or grant row that names its objects puts only those
//!    objects in front of the dispatcher, and only for a window of a kind
//!    their triggered abilities read.
//! 2. The turn queue names its entries: an extra turn carries the id of the
//!    entry it came from (§3.9's amendment), which "that turn" reads.

use std::sync::Arc;

use mtgsim::cards::authoring::{triggered_ability, whenever};
use mtgsim::cards::phase_re_cards::time_walk;
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, Layer};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    forest, pass_turn, put_in_hand, put_on_battlefield, registered, setup_two_player_game, stock_libraries, test_ctx,
    test_dp, vanilla_creature,
};
use mtgsim::types::effects::{AmountExpr, Duration, Effect, EffectRecipient, ObjectSet, Primitive};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::triggers::{TriggerEvent, TriggerSubject};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

/// Resolve `card`'s spell effect for `controller`, the way the stack would,
/// with nothing targeted; the card's own id is the resolution's source.
fn resolve_spell(game: &mut GameState, card: Arc<CardData>, controller: PlayerId) -> ObjectId {
    let id = put_in_hand(game, card.clone(), controller);
    let ctx = ResolutionContext {
        source: id,
        ability_source: None,
        controller,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
    id
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

// ---------------------------------------------------------------------------
// 2. The turn queue's ids
// ---------------------------------------------------------------------------

/// CR 500.7, with the id §3.9's amendment puts on the proposal: Time Walk's
/// extra turn is the queue entry it made, the turn that begins from it says
/// which, and the natural turn after it says it is none.
#[test]
fn an_extra_turn_carries_the_queue_entry_it_came_from() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 10);
    resolve_spell(&mut game, time_walk(), 0);
    assert_eq!(game.turn_queue.len(), 1);
    let queued = game.turn_queue[0];
    assert_eq!((queued.player, game.extra_turn), (0, None), "a natural turn names no entry");

    pass_turn(&mut game);
    assert_eq!((game.active_player, game.turn_number), (0, 2));
    assert_eq!(game.extra_turn, Some(queued.id), "the extra turn is the entry it came from");

    pass_turn(&mut game);
    assert_eq!((game.active_player, game.turn_number), (1, 3));
    assert_eq!(game.extra_turn, None, "and the natural turn after it is none");
}
