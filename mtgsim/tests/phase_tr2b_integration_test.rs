//! Phase TR-2b — "may", CR 118.12's answer, and the `departed` frames
//! (`triggers-architecture.md` §12, TR-2b; §13's TR-2b row).
//!
//! **What a trigger reads after the event.** TR-2a gave a trigger the turn it
//! happened in. TR-2b gives its resolution the rest: a choice made as it
//! resolves ("you may", CR 603.5), the answer that choice or a mandatory action
//! leaves for the clause after it (CR 118.12), and the last known information
//! of an object that left after it triggered (CR 113.7a, 608.2h).
//!
//! The fixtures are invented and carry their own names (`engineering-
//! practices.md` §3). The printed cards were verified on Scryfall on
//! 2026-09-26.

use std::sync::Arc;

use mtgsim::cards::authoring::{draws_a_card, triggered_ability, whenever, Whose};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    creature_with_ability, fill_library, put_on_battlefield, setup_two_player_game, test_ctx, test_dp,
};
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::triggers::TriggerEvent;
use mtgsim::types::zones::Zone;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A 1/1 creature fixture carrying one triggered ability.
fn watcher(name: &str, event: impl Into<TriggerEvent>, effect: Effect) -> Arc<CardData> {
    creature_with_ability(name, 1, 1, triggered_ability(whenever(event, effect)))
}

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

fn pending(game: &GameState) -> usize {
    game.pending_triggers.len()
}

/// Resolve `effect` as if `source`'s controller had cast it: no targets.
fn resolve_as(game: &mut GameState, source: ObjectId, effect: Effect) {
    let ctx = ResolutionContext::untargeted(source, 0);
    game.resolve_effect(&effect, &ctx, &test_dp()).expect("resolving");
}

fn put_top_cards_into_hand(n: u64) -> Effect {
    Effect::Atom(Primitive::PutTopCardsIntoHand(AmountExpr::Fixed(n)), EffectRecipient::Controller)
}

// ---------------------------------------------------------------------------
// CR 121.1, 121.5 — a draw, and a move to the hand that is not one
// ---------------------------------------------------------------------------

/// "Put the top card of your library into your hand" is not a draw (CR
/// 121.5): the card moves, no draw is announced, and "whenever you draw a
/// card" does not trigger. On an empty library it does nothing, and the player
/// does not lose, since CR 704.5b reads only an attempted draw. The draw beside
/// it is the control: one card, one record, one trigger.
// COVERS: ATOM-121.5-001
#[test]
fn a_card_put_into_the_hand_without_drawing_fires_no_draw_trigger() {
    let mut game = setup_two_player_game();
    game.record_events();
    let scribe = put_on_battlefield(&mut game, watcher("Studious Scribe", draws_a_card(Whose::Yours), gain_one()), 0);
    fill_library(&mut game, 0, 2);

    resolve_as(&mut game, scribe, put_top_cards_into_hand(1));
    assert_eq!(game.players[0].hand.len(), 1, "the card moved");
    assert_eq!(pending(&game), 0, "not a draw, so nothing triggers (CR 121.5)");
    let drawn = game.recorded_events().events().filter(|e| matches!(e, GameEvent::CardDrawn { .. })).count();
    assert_eq!(drawn, 0, "and no draw is announced");

    game.draw_card(0, &test_ctx()).unwrap();
    assert_eq!(pending(&game), 1, "the control: a draw triggers once");

    game.pending_triggers.clear();
    assert!(game.players[0].library.is_empty());
    resolve_as(&mut game, scribe, put_top_cards_into_hand(1));
    assert!(!game.players[0].has_drawn_from_empty_library, "no draw was attempted");
    game.check_state_based_actions_loop(&test_dp()).unwrap();
    assert!(game.in_game(0), "an empty library put into nothing loses nothing");
    assert_eq!(game.get_object(scribe).unwrap().zone, Zone::Battlefield);
}
