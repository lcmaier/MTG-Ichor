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

use mtgsim::cards::authoring::{at_beginning_of, draws_a_card, enters, triggered_ability, whenever, Whose};
use mtgsim::cards::phase_rd_cards::safe_passage;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::test_support::{
    creature_with_ability, fill_library, put_on_battlefield, setup_two_player_game, stock_libraries,
    test_ctx, test_dp,
};
use mtgsim::types::effects::{AmountExpr, Condition, CostAnswer, Effect, EffectRecipient, PlayerRef, Primitive};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::triggers::{TriggerEvent, TriggerSubject};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

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

fn you_may(effect: Effect) -> Effect {
    Effect::Optional { chooser: PlayerRef::You, effect: Box::new(effect) }
}

fn if_you(answer: CostAnswer, effect: Effect) -> Effect {
    Effect::Conditional(Condition::CostAnswer(answer), Box::new(effect))
}

fn draw_one() -> Effect {
    Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

fn lose_one() -> Effect {
    Effect::Atom(Primitive::LoseLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

/// A provider that answers the one "you may" `source` asks.
fn answering_may(source: ObjectId, yes: bool) -> ScriptedDecisionProvider {
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::OptionalEffect { source }, if yes { vec![0] } else { Vec::new() });
    dp
}

fn place(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).expect("placing");
}

/// The top of the stack's object and `resolve_top_of_stack`, so a test can
/// script the prompt its resolution asks.
fn top_of_stack(game: &GameState) -> ObjectId {
    *game.stack.last().expect("something on the stack")
}

/// Walk the turn machinery until `whose` player's `step` begins.
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

// ---------------------------------------------------------------------------
// CR 603.5, 118.12 — "may", and the answer the clause after it reads
// ---------------------------------------------------------------------------

/// "At the beginning of your upkeep, you may draw a card": the ability
/// triggers and goes on the stack whatever its controller means to do, and
/// the choice is asked as it resolves — yes draws, no does not.
// COVERS: ATOM-603.5-001
#[test]
fn a_may_trigger_goes_on_the_stack_and_is_asked_as_it_resolves() {
    for yes in [true, false] {
        let mut game = setup_two_player_game();
        put_on_battlefield(
            &mut game,
            watcher("Dawn Reader", at_beginning_of(StepType::Upkeep, Whose::Yours), you_may(draw_one())),
            0,
        );
        advance_to(&mut game, 0, StepType::Upkeep);
        assert_eq!(pending(&game), 1, "CR 603.5 — it triggers regardless");
        place(&mut game, &test_dp());
        let hand = game.players[0].hand.len();

        let ability = top_of_stack(&game);
        game.resolve_top_of_stack(&answering_may(ability, yes)).unwrap();
        assert_eq!(game.players[0].hand.len(), hand + usize::from(yes), "yes draws, no does not ({yes})");
    }
}

/// Wicked Guardian's third ruling: "if the damage ... is prevented, you still
/// draw a card". CR 118.12's clause reads the choice, not the stream, so a
/// "may" whose damage Safe Passage prevents still answers `Does`.
// COVERS-PARTIAL: ATOM-118.12-002
#[test]
fn a_may_whose_damage_is_prevented_still_answers_does() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 3);
    let shield_source = put_on_battlefield(&mut game, watcher("Warden of the Way", draws_a_card(Whose::Yours), lose_one()), 0);
    resolve_as(&mut game, shield_source, safe_passage().abilities[0].effect.clone());

    let hurt_me = Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::Fixed(2), unpreventable: false },
        EffectRecipient::Controller,
    );
    let guardian = watcher(
        "Grudging Guardian",
        enters(TriggerSubject::ThisObject),
        Effect::Sequence(vec![you_may(hurt_me), if_you(CostAnswer::Does, draw_one())]),
    );
    put_on_battlefield(&mut game, guardian, 0);
    place(&mut game, &test_dp());
    let (life, hand) = (game.players[0].life_total, game.players[0].hand.len());

    let ability = top_of_stack(&game);
    game.resolve_top_of_stack(&answering_may(ability, true)).unwrap();
    assert_eq!(game.players[0].life_total, life, "Safe Passage prevented the 2");
    assert_eq!(game.players[0].hand.len(), hand + 1, "and the card is drawn: the answer is the choice");
}

/// "If you do … if you don't …" read one answer. A clause's own atoms take
/// actions of their own, and the walk puts the answer back after each clause,
/// so a second "if you don't" still sees the declined "may".
#[test]
fn two_clauses_after_one_may_read_its_one_answer() {
    for yes in [true, false] {
        let mut game = setup_two_player_game();
        fill_library(&mut game, 0, 3);
        let source = put_on_battlefield(&mut game, watcher("Idle Scholar", draws_a_card(Whose::Each), draw_one()), 1);
        let effect = Effect::Sequence(vec![
            you_may(gain_one()),
            if_you(CostAnswer::Doesnt, lose_one()),
            if_you(CostAnswer::Doesnt, draw_one()),
        ]);
        let hand = game.players[0].hand.len();
        let ctx = ResolutionContext::untargeted(source, 0);
        game.resolve_effect(&effect, &ctx, &answering_may(source, yes)).unwrap();
        if yes {
            assert_eq!((game.players[0].life_total, game.players[0].hand.len()), (21, hand));
        } else {
            assert_eq!((game.players[0].life_total, game.players[0].hand.len()), (19, hand + 1), "both clauses ran");
        }
    }
}
