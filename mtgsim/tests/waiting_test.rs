//! The Waiting view (`mtgsim::ui::waiting`): what the game is holding for
//! later, as a client reads it. Each board checks a row appears when the
//! thing it describes is made, says what the CR says it is, and is gone the
//! moment it fires, expires, or its turn is skipped or will never come.

use std::sync::Arc;

use mtgsim::cards::authoring::{at_beginning_of, whenever, Whose};
use mtgsim::cards::phase_re_cards::{meditate, time_walk};
use mtgsim::cards::phase_tr3a_cards::{blessed_wine, final_fortune};
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::test_support::{put_in_hand, setup_game, setup_two_player_game, stock_libraries, test_ctx, test_dp};
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::triggers::{DelayedDuration, DelayedTriggerTemplate, DelayedTurn};
use mtgsim::events::event::LossReason;
use mtgsim::ui::waiting::{what_is_waiting, TriggersIn, WaitingTurn};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve `card`'s first spell ability for `controller` by hand, the card
/// its own source.
fn resolve_spell(game: &mut GameState, card: Arc<CardData>, controller: PlayerId) -> ObjectId {
    let id = put_in_hand(game, card.clone(), controller);
    game.resolve_effect(&card.abilities[0].effect, &ResolutionContext::untargeted(id, controller), &test_dp()).unwrap();
    id
}

/// Blessed Wine's second paragraph, resolved for player 0: the delayed draw.
fn resolve_wines_draw(game: &mut GameState) -> ObjectId {
    let wine = blessed_wine();
    let id = put_in_hand(game, wine.clone(), 0);
    game.resolve_effect(&wine.abilities[1].effect, &ResolutionContext::untargeted(id, 0), &test_dp()).unwrap();
    id
}

fn advance_to(game: &mut GameState, whose: PlayerId, step: StepType) {
    stock_libraries(game, 10);
    for _ in 0..200 {
        game.advance_turn(&test_ctx()).expect("advancing");
        if game.active_player == whose && game.phase.step == Some(step) {
            return;
        }
    }
    panic!("player {whose}'s {step:?} never began");
}

// ---------------------------------------------------------------------------
// Delayed triggers
// ---------------------------------------------------------------------------

/// The click script's board: Blessed Wine's draw waits, says what it does,
/// whose it is and which turn it can trigger in, and goes as it triggers in
/// the next turn's upkeep.
#[test]
fn blessed_wines_draw_waits_until_the_next_turns_upkeep() {
    let mut game = setup_two_player_game();
    let wine = resolve_wines_draw(&mut game);

    let rows = what_is_waiting(&game).delayed_triggers;
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.text, "Draw a card at the beginning of the next turn's upkeep.");
    assert_eq!(row.source, format!("Blessed Wine ({wine})"));
    assert_eq!(row.controller, 0);
    assert_eq!(row.turn, TriggersIn::TurnAfter(game.turn_number), "not a second upkeep this turn (CR 500.10)");
    assert_eq!(row.duration, DelayedDuration::Once);

    advance_to(&mut game, 1, StepType::Upkeep);
    assert!(what_is_waiting(&game).delayed_triggers.is_empty(), "it triggered, once, and is gone");
}

/// "Each time this turn" until CR 514.2's cleanup ends it, and then gone
/// without ever triggering.
#[test]
fn a_this_turn_delayed_trigger_is_gone_after_the_cleanup() {
    let mut game = setup_two_player_game();
    let template = DelayedTriggerTemplate {
        def: Arc::new(whenever(
            at_beginning_of(StepType::Upkeep, Whose::Each),
            Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
        )),
        duration: DelayedDuration::ThisTurn,
        turn: DelayedTurn::Any,
        rules_text: "This turn, whenever an upkeep begins, you gain 1 life.".into(),
    };
    let create = Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(template)), EffectRecipient::Controller);
    let source = put_in_hand(&mut game, blessed_wine(), 0);
    game.resolve_effect(&create, &ResolutionContext::untargeted(source, 0), &test_dp()).unwrap();
    let rows = what_is_waiting(&game).delayed_triggers;
    assert_eq!((rows[0].duration, rows[0].turn), (DelayedDuration::ThisTurn, TriggersIn::AnyTurn));

    advance_to(&mut game, 1, StepType::Untap);
    assert!(what_is_waiting(&game).delayed_triggers.is_empty(), "the cleanup ended it");
}

// ---------------------------------------------------------------------------
// Extra turns, and a delayed trigger bound to one
// ---------------------------------------------------------------------------

/// Final Fortune and then Time Walk: two extra turns, Time Walk's taken
/// first (CR 500.7), and Final Fortune's loss bound to its own. Each row
/// goes as its turn begins, and the loss's row says when its turn is the one
/// in progress.
#[test]
fn extra_turns_are_listed_in_the_order_they_will_be_taken() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 10);
    resolve_spell(&mut game, final_fortune(), 0);
    resolve_spell(&mut game, time_walk(), 0);
    let fortune = game.turn_queue[0].id;
    let walk = game.turn_queue[1].id;

    let waiting = what_is_waiting(&game);
    assert_eq!(
        waiting.extra_turns,
        vec![WaitingTurn { id: walk, player: 0 }, WaitingTurn { id: fortune, player: 0 }],
        "the most recently created first"
    );
    assert_eq!(waiting.delayed_triggers[0].turn, TriggersIn::ExtraTurn { player: 0, in_progress: false });

    advance_to(&mut game, 0, StepType::Upkeep);
    assert_eq!(game.extra_turn, Some(walk));
    assert_eq!(what_is_waiting(&game).extra_turns, vec![WaitingTurn { id: fortune, player: 0 }]);

    advance_to(&mut game, 0, StepType::Upkeep);
    assert_eq!(game.extra_turn, Some(fortune));
    let waiting = what_is_waiting(&game);
    assert!(waiting.extra_turns.is_empty());
    assert_eq!(waiting.delayed_triggers[0].turn, TriggersIn::ExtraTurn { player: 0, in_progress: true });
}

/// Meditate's "skip your next turn" skips Final Fortune's (CR 614.10a): the
/// turn's row and the loss bound to it both go.
#[test]
fn a_skipped_extra_turn_takes_its_row_and_its_trigger_with_it() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 10);
    resolve_spell(&mut game, meditate(), 0);
    resolve_spell(&mut game, final_fortune(), 0);
    assert_eq!(what_is_waiting(&game).extra_turns.len(), 1);

    advance_to(&mut game, 1, StepType::Upkeep);
    let waiting = what_is_waiting(&game);
    assert!(waiting.extra_turns.is_empty() && waiting.delayed_triggers.is_empty(), "{waiting:?}");
}

/// A player who has left takes no turn (CR 800.4), so their queued extra
/// turn is not waiting for anything.
#[test]
fn a_departed_players_extra_turn_is_not_waiting() {
    let mut game = setup_game(3);
    stock_libraries(&mut game, 10);
    resolve_spell(&mut game, time_walk(), 2);
    assert_eq!(what_is_waiting(&game).extra_turns.len(), 1);

    game.execute_action(GameAction::PlayerLoses { player: 2, reason: LossReason::Effect }, &test_ctx()).unwrap();
    assert!(what_is_waiting(&game).extra_turns.is_empty());
}
