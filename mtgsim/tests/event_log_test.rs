//! The event log's lines, as `--dump-events` and the dev GUI's log read
//! them: formatted after the event, when an object a line names may be gone.
//!
//! An ability on the stack is an object that ceases to exist as it resolves,
//! doesn't resolve, or is countered (CR 608.2b, 608.2n, 701.6b), so a line
//! naming it by that object reads as a bare id. Each test reads the log once
//! the objects are gone, and asserts on its words, which every reader sees.

use std::sync::Arc;

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_cv_cards::cryptoplasm;
use mtgsim::cards::phase_tr1_cards::soul_warden;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::test_support::{
    fill_library, lightning_bolt, put_in_hand, put_on_battlefield, setup_two_player_game, test_ctx, test_dp,
};
use mtgsim::types::effects::{Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::display::format_event_log;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Walk the turn machinery until `whose` player's `step` begins. Libraries
/// are filled so a draw step on the way is not a loss.
fn advance_to(game: &mut GameState, whose: PlayerId, step: StepType) {
    for p in 0..game.num_players() {
        if game.players[p].library.len() < 5 {
            fill_library(game, p, 10);
        }
    }
    for _ in 0..200 {
        game.advance_turn(&test_ctx()).expect("advancing");
        if game.active_player == whose && game.phase.step == Some(step) {
            return;
        }
    }
    panic!("player {whose}'s {step:?} never began");
}

/// Empty `player`'s pool, fill it with exactly `pool`, and cast `card` from
/// hand under `ManaWindowStop`, as a shipped client does.
fn cast_from_pool(
    game: &mut GameState,
    player: PlayerId,
    card: Arc<CardData>,
    pool: &[(ManaType, u64)],
    dp: impl DecisionProvider,
) -> ObjectId {
    let id = put_in_hand(game, card, player);
    for t in [ManaType::White, ManaType::Blue, ManaType::Black, ManaType::Red, ManaType::Green, ManaType::Colorless] {
        let have = game.players[player].mana_pool.amount(t);
        if have > 0 {
            game.players[player].mana_pool.remove(t, have).unwrap();
        }
    }
    for &(t, n) in pool {
        game.players[player].mana_pool.add(t, n);
    }
    game.cast_spell(player, id, &ManaWindowStop::new(dp)).expect("castable from exactly its cost");
    id
}

/// Lightning Bolt cast by `player` at `target`.
fn bolt_at(game: &mut GameState, player: PlayerId, target: ObjectId) -> ObjectId {
    let aim = ScriptedDecisionProvider::new();
    aim.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
        vec![ChoiceOption::Object(target)],
    );
    cast_from_pool(game, player, lightning_bolt(), &[(ManaType::Red, 1)], aim)
}

fn resolve_top(game: &mut GameState) {
    game.resolve_top_of_stack(&test_dp()).expect("resolving");
    game.perform_sba_and_triggers(&test_dp()).expect("the state-based actions and triggers after it");
}

// ---------------------------------------------------------------------------
// An ability that doesn't resolve, and one countered
// ---------------------------------------------------------------------------

/// The dev GUI's board: Cryptoplasm's upkeep trigger targets the Bears, and
/// Lightning Bolt kills them in response. The trigger doesn't resolve (CR
/// 608.2b), and its line, read once the ability has ceased to exist, says
/// what it was, whose, and why.
#[test]
fn an_ability_that_does_not_resolve_is_named_by_its_source() {
    let mut game = setup_two_player_game();
    let crypto = put_on_battlefield(&mut game, cryptoplasm(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    advance_to(&mut game, 0, StepType::Upkeep);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    let trigger = *game.stack.last().expect("Cryptoplasm's trigger, its one target the Bears");

    bolt_at(&mut game, 1, bears);
    resolve_top(&mut game);
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Graveyard);
    resolve_top(&mut game);
    assert!(game.stack.is_empty() && !game.objects.contains_key(&trigger), "the ability ceased to exist");

    let log = format_event_log(&game);
    let line = format!("AbilityFizzled: Cryptoplasm ({crypto})'s ability [P0] doesn't resolve: every target is illegal (CR 608.2b)");
    assert!(log.contains(&line), "no {line:?} in {log:#?}");
    assert!(!log.iter().any(|l| l.starts_with("SpellFizzled")), "an ability is not a spell: {log:#?}");
}

/// A spell's line says why too, and names the card in its graveyard.
#[test]
fn a_spell_that_does_not_resolve_says_why() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let bolt = bolt_at(&mut game, 0, bears);
    game.change_zone(bears, Zone::Hand, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    resolve_top(&mut game);

    let log = format_event_log(&game);
    let line = format!("SpellFizzled: Lightning Bolt ({bolt}) doesn't resolve: every target is illegal (CR 608.2b)");
    assert!(log.contains(&line), "no {line:?} in {log:#?}");
}

/// Soul Warden's trigger countered by an effect that counters abilities (CR
/// 701.6b): the countered ability is named by its source, as the one that
/// doesn't resolve is.
#[test]
fn a_countered_ability_is_named_by_its_source() {
    let mut game = setup_two_player_game();
    let warden = put_on_battlefield(&mut game, soul_warden(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    let ability = *game.stack.last().expect("Soul Warden's trigger");
    let stifle = put_in_hand(&mut game, lightning_bolt(), 1);
    let ctx = ResolutionContext {
        source: stifle,
        ability_source: None,
        controller: 1,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(ability)]),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    let counter = Effect::Atom(Primitive::CounterAbility, EffectRecipient::Target(SelectionFilter::Spell, TargetCount::Exactly(1)));
    game.resolve_effect(&counter, &ctx, &test_dp()).unwrap();
    assert!(game.stack.is_empty());

    let log = format_event_log(&game);
    let line = format!("AbilityCountered: Soul Warden ({warden})'s ability [P0] countered by Lightning Bolt ({stifle})");
    assert!(log.contains(&line), "no {line:?} in {log:#?}");
}
