//! TR-6 — state triggers (CR 603.8), the draw (CR 104.4) and the rule-owned
//! arm (`triggers-architecture.md` §4.5, §4.8, §4.9 and §12).

use mtgsim::cards::authoring::{enters, triggered_ability, whenever};
use mtgsim::cards::phase_re_cards::platinum_angel;
use mtgsim::engine::actions::ZoneChangeAct;
use mtgsim::events::cause::{EventCause, GameRule};
use mtgsim::events::event::GameEvent;
use mtgsim::state::game_state::{GameResult, GameState};
use mtgsim::test_support::{creature_with_ability, put_in_hand, put_on_battlefield, setup_game, test_ctx};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, ObjectFilter, Primitive};
use mtgsim::types::ids::new_ability_id;
use mtgsim::types::triggers::{TriggerEvent, TriggerSubject};
use mtgsim::types::zones::Zone;

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

/// Each draw the game recorded, by its cause: the end of the game as a
/// performed record, which a direct write of the result never leaves.
fn draws(game: &GameState) -> Vec<EventCause> {
    game.recorded_events()
        .records()
        .iter()
        .filter_map(|record| matches!(record.event, GameEvent::GameEndedInADraw).then_some(record.stamp.cause))
        .collect()
}

// ---------------------------------------------------------------------------
// The draw (CR 104.4)
// ---------------------------------------------------------------------------

/// A creature whose trigger watches a creature's ability triggering triggers
/// on its own triggering, each `AbilityTriggered` dispatched inside the one
/// that queued it. The dispatcher takes the chain at its nesting bound for one
/// with no end, and CR 104.4b makes the game a draw: performed, so the move
/// that started the chain completes. Beside Platinum Angel, whose fourth
/// ruling is that "effects that say the game is a draw ... are not affected".
#[test]
fn a_trigger_chain_at_the_nesting_bound_is_a_draw_beside_platinum_angel() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, platinum_angel(), 0);
    let mut echo = triggered_ability(
        "",
        whenever(TriggerEvent::AbilityTriggers { caused_by: None, source: Some(ObjectFilter::ByType(CardType::Creature)) }, gain_one()),
    );
    echo.id = new_ability_id();
    put_on_battlefield(&mut game, creature_with_ability("Fixture: echo", 1, 1, echo), 0);
    let mut arrives = triggered_ability("", whenever(enters(TriggerSubject::ThisObject), gain_one()));
    arrives.id = new_ability_id();
    let arriving = put_in_hand(&mut game, creature_with_ability("Fixture: arriving", 1, 1, arrives), 0);

    game.change_zone(arriving, Zone::Battlefield, ZoneChangeAct::Put, &test_ctx())
        .expect("the move is performed, and the chain it starts ends the game");

    assert_eq!(game.result, Some(GameResult::Draw));
    assert_eq!(draws(&game), vec![EventCause::GameRule(GameRule::MandatoryLoop)]);
    assert!(game.pending_triggers.is_empty(), "a finished game keeps no waiting trigger (CR 104.1)");
}
