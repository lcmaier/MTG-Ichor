//! Phase TR-3a — the delayed-trigger registry (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-10-07 and is
//! quoted verbatim; each card's rulings were read the same day and sit in its
//! own doc comment (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Final Fortune | a spell's delayed trigger (CR 603.7d) in "that turn", the extra turn it made (CR 500.7) | no |
//! | Blessed Wine | a spell's delayed trigger in the next turn, which ends no game | no |
//!
//! **Two cards where the brief named one** (the owner, 2026-10-07). Every
//! printed "that turn" card makes its caster lose (Final Fortune, Last
//! Chance, Warrior's Oath, Alchemist's Gambit), and a random agent casts one
//! whenever it can: Final Fortune ended 29 of 200 two-seat stress games at its
//! extra turn, at seed 12345, and was cast 3 times in 200 four-seat games.
//! Blessed Wine puts the registry in every stress sitting with no game cut
//! short; Final Fortune stays for the shapes stress exists to hunt, its
//! caster leaving mid-turn at four seats (CR 800.4) among them.

use std::sync::Arc;

use crate::cards::authoring::{at_beginning_of, whenever, Whose};
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive};
use crate::types::ids::AbilityId;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{DelayedDuration, DelayedTriggerTemplate, DelayedTurn};

/// One paragraph of an instant: a spell ability (CR 113.2c).
fn spell_ability(rules_text: &'static str, effect: Effect) -> AbilityDef {
    AbilityDef {
        rules_text: rules_text.into(),
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Spell,
        costs: Vec::new(),
        effect,
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    }
}

/// Final Fortune — {R}{R}
/// Instant
///
/// > Take an extra turn after this one. At the beginning of that turn's end
/// > step, you lose the game.
///
/// One spell ability: the extra turn, then the delayed trigger in "that
/// turn", which names the queue entry the first instruction made
/// (`DelayedTurn::ThatExtraTurn`), so "that turn's end step" is whichever end
/// step that turn has. The trigger is the spell's and its controller's as it
/// resolved (CR 603.7d), so "you" is that player wherever the card has gone.
///
/// **Registered, not pooled**, for Time Walk's reason: an extra turn moves
/// `Avg turns/game` by design, and a loss at its end moves the outcomes.
///
/// # The rulings, and where each is tested
///
/// Both below.
/// - *"If multiple 'extra turn' effects resolve in the same turn, take them in
///   the reverse of the order that the effects resolved. In other words, the
///   most recently created extra turn is taken first."* (#1)
///   → `final_fortunes_turn_is_its_own_among_several`: a Time Walk resolving
///   after it is taken first, and the loss waits for Final Fortune's own turn.
/// - *"If you end up skipping the extra turn that is gained, you do not lose
///   the game."* (#2) → `a_skipped_final_fortune_turn_loses_nothing`.
pub fn final_fortune() -> Arc<CardData> {
    let text = "Take an extra turn after this one. At the beginning of that turn's end step, you lose the game.";
    let lose = DelayedTriggerTemplate {
        def: Arc::new(whenever(
            at_beginning_of(StepType::End, Whose::Each),
            Effect::Atom(Primitive::LoseGame, EffectRecipient::Controller),
        )),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::ThatExtraTurn,
    };
    CardDataBuilder::new("Final Fortune")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                Effect::Atom(Primitive::ExtraTurn, EffectRecipient::Controller),
                Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(lose)), EffectRecipient::Controller),
            ]),
        ))
        .build()
}

/// Blessed Wine — {1}{W}
/// Instant
///
/// > You gain 1 life.
/// > Draw a card at the beginning of the next turn's upkeep.
///
/// Two spell abilities, one per paragraph, followed in the order written (CR
/// 113.2c, 608.2c): the gain, then a delayed trigger that is the spell's and
/// its controller's as it resolved (CR 603.7d), so "draw" is that player's in
/// whoever's upkeep comes next. "The next turn's" binds it to a later turn
/// than this one (`DelayedTurn::NextTurn`), so a second upkeep this turn (CR
/// 500.10) is not it.
///
/// **Registered, not pooled**: §12 pools the registry with TR-3b's two cards,
/// so this PR's close-out reads the performance pool unmoved.
///
/// # The rulings, and where each is tested
///
/// Scryfall lists no rulings (2026-10-07). The tests are below: cast from
/// hand, and the second upkeep it waits past.
pub fn blessed_wine() -> Arc<CardData> {
    let draw_next_upkeep = DelayedTriggerTemplate {
        def: Arc::new(whenever(
            at_beginning_of(StepType::Upkeep, Whose::Each),
            Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller),
        )),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::NextTurn,
    };
    CardDataBuilder::new("Blessed Wine")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text("You gain 1 life.\nDraw a card at the beginning of the next turn's upkeep.")
        .ability(spell_ability(
            "You gain 1 life.",
            Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
        ))
        .ability(spell_ability(
            "Draw a card at the beginning of the next turn's upkeep.",
            Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(draw_next_upkeep)), EffectRecipient::Controller),
        ))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::phase_re_cards::{meditate, time_walk};
    use crate::engine::actions::GameAction;
    use crate::engine::resolve::ResolutionContext;
    use crate::engine::targeting::ChosenTargets;
    use crate::state::game_state::{GameResult, GameState};
    use crate::test_support::{put_in_hand, setup_two_player_game, stock_libraries, test_ctx, test_dp, RecordingDecisionProvider};
    use crate::types::ids::{ObjectId, PlayerId};
    use crate::types::triggers::{TriggerOrigin, TriggerTurn};
    use crate::types::zones::Zone;
    use crate::ui::mana_window_stop::ManaWindowStop;

    /// Resolve `card`'s spell effect for `controller` by hand, the card its
    /// own source.
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

    /// Place every waiting trigger and resolve the stack, as the priority
    /// loop would, unless the game has ended.
    fn place_and_resolve(game: &mut GameState) {
        let dp = RecordingDecisionProvider::picking(0);
        game.perform_sba_and_triggers(&dp).unwrap();
        while !game.stack.is_empty() && game.result.is_none() {
            game.resolve_top_of_stack(&dp).unwrap();
            game.perform_sba_and_triggers(&dp).unwrap();
        }
    }

    /// Play through turn `last`, resolving what triggers at each step, until
    /// the next turn begins or the game ends.
    fn play_through(game: &mut GameState, last: u32) {
        for _ in 0..400 {
            if game.result.is_some() || game.turn_number > last {
                return;
            }
            game.advance_turn(&test_ctx()).expect("advancing");
            place_and_resolve(game);
        }
    }

    /// Walk the turn machinery until `whose` player's `step` begins,
    /// resolving what triggers on the way.
    fn play_to(game: &mut GameState, whose: PlayerId, step: StepType) {
        for _ in 0..200 {
            game.advance_turn(&test_ctx()).expect("advancing");
            if game.active_player == whose && game.phase.step == Some(step) {
                return;
            }
            place_and_resolve(game);
        }
        panic!("player {whose}'s {step:?} never began");
    }

    /// Cast `card` for player 0 from hand, with exactly `pool` in their pool,
    /// under `ManaWindowStop`, and resolve it.
    fn cast_and_resolve(game: &mut GameState, card: Arc<CardData>, pool: &[(ManaType, u64)]) -> ObjectId {
        let id = put_in_hand(game, card, 0);
        for &(mana, n) in pool {
            game.players[0].mana_pool.add(mana, n);
        }
        game.cast_spell(0, id, &ManaWindowStop::new(test_dp())).expect("castable from exactly its cost");
        game.resolve_top_of_stack(&test_dp()).unwrap();
        id
    }

    /// The extra turn a registered "that turn" trigger waits for.
    fn its_extra_turn(game: &GameState) -> crate::types::ids::ExtraTurnId {
        match game.delayed_triggers[0].turn {
            TriggerTurn::Extra(turn) => turn,
            other => panic!("not bound to an extra turn: {other:?}"),
        }
    }

    /// Cast from hand from exactly {R}{R} under `ManaWindowStop`: the extra
    /// turn is queued, and the delayed trigger is the spell's, its
    /// controller's (CR 603.7d) and that turn's. Player 0 takes the extra
    /// turn and loses at its end step, so player 1 wins (CR 104.2a).
    #[test]
    fn final_fortune_cast_from_hand_loses_at_its_turns_end_step() {
        let mut game = setup_two_player_game();
        stock_libraries(&mut game, 10);
        let fortune = put_in_hand(&mut game, final_fortune(), 0);
        game.players[0].mana_pool.add(ManaType::Red, 2);
        game.cast_spell(0, fortune, &ManaWindowStop::new(test_dp())).expect("castable from exactly its cost");
        let cast = game.object_ref(fortune).unwrap();
        game.resolve_top_of_stack(&test_dp()).unwrap();
        assert_eq!(game.turn_queue.len(), 1);
        let delayed = &game.delayed_triggers[0];
        assert_eq!((delayed.source, delayed.controller), (cast, 0), "the spell, as it resolved, and its controller");
        assert_eq!(its_extra_turn(&game), game.turn_queue[0].id, "that turn");
        assert_eq!(game.get_object(fortune).unwrap().zone, Zone::Graveyard);

        play_through(&mut game, 2);
        assert_eq!((game.active_player, game.turn_number), (0, 2), "the extra turn");
        assert_eq!(game.result, Some(GameResult::Winner(1)));
        assert!(game.delayed_triggers.is_empty());
    }

    // RULING: Final Fortune #2 - "If you end up skipping the extra turn that is
    //   gained, you do not lose the game."
    /// Meditate's "you skip your next turn" skips the extra turn (CR 614.10a's
    /// next occurrence), so "that turn" never comes: player 0 plays their
    /// next natural turn and loses nothing, and the trigger is gone.
    #[test]
    fn a_skipped_final_fortune_turn_loses_nothing() {
        let mut game = setup_two_player_game();
        stock_libraries(&mut game, 10);
        resolve_spell(&mut game, meditate(), 0);
        resolve_spell(&mut game, final_fortune(), 0);
        assert_eq!(game.delayed_triggers.len(), 1);

        play_through(&mut game, 2);
        assert_eq!(game.result, None, "no loss");
        assert_eq!((game.active_player, game.turn_number), (0, 3), "player 1's turn 2, then player 0's turn 3");
        assert!(game.delayed_triggers.is_empty(), "its turn was skipped, so it went");
    }

    // RULING: Final Fortune #1 - "If multiple 'extra turn' effects resolve in the
    //   same turn, take them in the reverse of the order that the effects
    //   resolved. In other words, the most recently created extra turn is taken
    //   first."
    /// Final Fortune resolves, then Time Walk: Time Walk's turn is the most
    /// recently created, so it comes first, turn 2, and player 0 survives its
    /// end step. Final Fortune's is turn 3, and its end step is the loss.
    #[test]
    fn final_fortunes_turn_is_its_own_among_several() {
        let mut game = setup_two_player_game();
        stock_libraries(&mut game, 10);
        resolve_spell(&mut game, final_fortune(), 0);
        resolve_spell(&mut game, time_walk(), 0);
        let fortunes_turn = its_extra_turn(&game);
        assert_eq!(fortunes_turn, game.turn_queue[0].id, "the older entry, taken last");

        play_through(&mut game, 2);
        assert_eq!(game.result, None, "Time Walk's turn, turn 2, ends with no loss");
        assert_eq!((game.active_player, game.turn_number), (0, 3));
        assert_eq!(game.extra_turn, Some(fortunes_turn), "and turn 3 is Final Fortune's");

        play_through(&mut game, 3);
        assert_eq!(game.turn_number, 3);
        assert_eq!(game.result, Some(GameResult::Winner(1)));
    }

    /// Cast from hand from exactly {1}{W} under `ManaWindowStop`: one life
    /// now, and a delayed trigger that is the spell's and player 0's (CR
    /// 603.7d), bound to a later turn. Player 1's upkeep is the next turn's,
    /// and player 0 draws there.
    #[test]
    fn blessed_wine_cast_from_hand_draws_in_the_next_turns_upkeep() {
        let mut game = setup_two_player_game();
        stock_libraries(&mut game, 10);
        let wine = put_in_hand(&mut game, blessed_wine(), 0);
        game.players[0].mana_pool.add(ManaType::White, 1);
        game.players[0].mana_pool.add(ManaType::Colorless, 1);
        game.cast_spell(0, wine, &ManaWindowStop::new(test_dp())).expect("castable from exactly its cost");
        let cast = game.object_ref(wine).unwrap();
        game.resolve_top_of_stack(&test_dp()).unwrap();
        assert_eq!(game.players[0].life_total, 21);
        let delayed = &game.delayed_triggers[0];
        assert_eq!((delayed.source, delayed.controller, delayed.turn), (cast, 0, TriggerTurn::LaterThan(1)));
        let hand = game.players[0].hand.len();

        play_to(&mut game, 1, StepType::Upkeep);
        let pending = &game.pending_triggers[0];
        assert!(matches!(pending.origin, TriggerOrigin::Delayed(_)));
        assert_eq!(pending.controller, 0, "player 0's, in player 1's upkeep");
        place_and_resolve(&mut game);
        assert_eq!(game.players[0].hand.len(), hand + 1);
        assert!(game.delayed_triggers.is_empty());
    }

    /// "The next turn's upkeep" is not a second upkeep this turn (CR
    /// 500.10's extra step, Paradox Haze's shape, proposed here by hand since
    /// no registered card makes one): the trigger waits for the next turn's.
    #[test]
    fn blessed_wine_waits_past_a_second_upkeep_this_turn() {
        let mut game = setup_two_player_game();
        stock_libraries(&mut game, 10);
        cast_and_resolve(&mut game, blessed_wine(), &[(ManaType::White, 1), (ManaType::Colorless, 1)]);

        game.execute_action(GameAction::BeginStep { step: StepType::Upkeep, player: 0 }, &test_ctx()).unwrap();
        assert!(game.pending_triggers.is_empty(), "an upkeep of this turn");
        play_to(&mut game, 1, StepType::Upkeep);
        assert_eq!(game.pending_triggers.len(), 1, "the next turn's");
    }
}
