//! TR-5c — blocks, targeting and activation (`triggers-architecture.md` §12,
//! TR-5c).
//!
//! 1. The declaration of blockers is one record for the step (CR 509.1, 802.4),
//!    written when nobody blocks, with the unblocked creatures (CR 509.1h).
//! 2. "This turn" and "this combat": what an object did, read off the
//!    combat's history or off the permanent (CR 400.7).

use std::sync::Arc;

use mtgsim::cards::authoring::{dies, triggered_ability, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::get_effective_power;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::{
    fill_library, put_on_battlefield, put_on_battlefield_this_turn, set_attacking, setup_game, setup_two_player_game,
    static_ability, test_dp,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, Primitive};
use mtgsim::types::history::ObjectFact;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::triggers::TriggerSubject;
use mtgsim::types::zones::Zone;
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn stocked(players: usize) -> GameState {
    let mut game = if players == 2 { setup_two_player_game() } else { setup_game(players) };
    for p in 0..players {
        fill_library(&mut game, p, 10);
    }
    game
}

/// The active player declares `attacks`, each attacker against its player.
fn declare_attacks(game: &mut GameState, attacks: &[(ObjectId, PlayerId)]) {
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(StepType::DeclareAttackers) });
    let dp = ScriptedDecisionProvider::new();
    let chosen = attacks.iter().map(|&(id, p)| ChoiceOption::AttackerTarget(id, AttackTarget::Player(p))).collect();
    dp.expect_choice(ChoiceKind::DeclareAttackers, chosen);
    game.process_declare_attackers(&dp).unwrap();
}

/// Each defending player who has a creature that could block declares its
/// list of (blocker, attacker), in seat order.
fn declare_blocks(game: &mut GameState, per_defender: &[Vec<(ObjectId, ObjectId)>]) {
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(StepType::DeclareBlockers) });
    let dp = ScriptedDecisionProvider::new();
    for blocks in per_defender {
        let chosen = blocks.iter().map(|&(b, a)| ChoiceOption::BlockerAttacker(b, a)).collect();
        dp.expect_choice(ChoiceKind::DeclareBlockers, chosen);
    }
    game.process_declare_blockers(&dp).unwrap();
    assert!(dp.is_empty(), "every scripted declaration was asked");
}

/// Every `BlockersDeclared` record, in order.
fn declarations(game: &GameState) -> Vec<(Vec<(ObjectId, ObjectId)>, Vec<ObjectId>)> {
    game.recorded_events()
        .events()
        .filter_map(|event| match event {
            GameEvent::BlockersDeclared { blocks, unblocked } => Some((blocks.clone(), unblocked.clone())),
            _ => None,
        })
        .collect()
}

/// An enchantment whose creatures matching `filter` get +1/+0.
fn banner(name: &str, filter: ObjectFilter) -> Arc<CardData> {
    let creatures = ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(filter));
    CardDataBuilder::new(name)
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(0), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(creatures),
        )))
        .build()
}

fn power(game: &GameState, id: ObjectId) -> Option<i32> {
    get_effective_power(game, id)
}

fn hand_size(game: &GameState, player: PlayerId) -> usize {
    game.players[player].hand.len()
}

// ---------------------------------------------------------------------------
// 1. The declaration's record
// ---------------------------------------------------------------------------

/// Nobody blocks, and the declaration is still written, naming the attacker
/// as unblocked: CR 509.3g's "attacks and isn't blocked" reads it.
#[test]
fn a_declaration_with_no_blocks_is_still_written() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    declare_attacks(&mut game, &[(bears, 1)]);
    declare_blocks(&mut game, &[vec![]]);
    assert_eq!(declarations(&game), vec![(vec![], vec![bears])]);
}

/// With no creature to block with, the defending player is asked nothing,
/// and the record still says the attacker is unblocked.
#[test]
fn a_defender_with_nothing_to_block_with_still_declares() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    declare_attacks(&mut game, &[(bears, 1)]);
    declare_blocks(&mut game, &[]);
    assert_eq!(declarations(&game), vec![(vec![], vec![bears])]);
}

/// CR 802.4: each defending player declares in APNAP order, and the step has
/// one record holding every block, in that order.
#[test]
fn every_defending_players_blocks_are_one_record() {
    let mut game = stocked(3);
    let (first, second) = (put_on_battlefield(&mut game, grizzly_bears(), 0), put_on_battlefield(&mut game, grizzly_bears(), 0));
    let (b1, b2) = (put_on_battlefield(&mut game, grizzly_bears(), 1), put_on_battlefield(&mut game, grizzly_bears(), 2));
    declare_attacks(&mut game, &[(first, 1), (second, 2)]);
    declare_blocks(&mut game, &[vec![(b1, first)], vec![(b2, second)]]);
    assert_eq!(declarations(&game), vec![(vec![(b1, first), (b2, second)], vec![])]);
}

/// CR 509.3g's second sentence: a creature attacking that was never declared
/// an attacker, as one put onto the battlefield attacking, is unblocked too.
/// And a line in the log names it.
#[test]
fn a_creature_attacking_without_a_declaration_is_unblocked() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let entered = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let blocker = put_on_battlefield(&mut game, grizzly_bears(), 1);
    declare_attacks(&mut game, &[(bears, 1)]);
    set_attacking(&mut game, entered, 1);
    declare_blocks(&mut game, &[vec![(blocker, bears)]]);
    assert_eq!(declarations(&game), vec![(vec![(blocker, bears)], vec![entered])]);
    let line = format!("BlockersDeclared: [Grizzly Bears ({blocker}) blocks Grizzly Bears ({bears}), Grizzly Bears ({entered}) unblocked]");
    assert!(mtgsim::ui::display::format_event_log(&game).contains(&line), "no {line:?}");
}

// ---------------------------------------------------------------------------
// 2. "This turn" and "this combat"
// ---------------------------------------------------------------------------

/// "Each creature that blocked or was blocked this turn": the blocker and
/// the attacker it blocked, each by its own fact, and the creature that
/// attacked unblocked by neither.
#[test]
fn blocked_and_was_blocked_are_the_declarations_two_sides() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, banner("Fixture Blocker's Banner", ObjectFilter::ThisTurn(ObjectFact::Blocked)), 0);
    put_on_battlefield(&mut game, banner("Fixture Blocked Banner", ObjectFilter::ThisTurn(ObjectFact::WasBlocked)), 1);
    let (bears, unblocked) = (put_on_battlefield(&mut game, grizzly_bears(), 0), put_on_battlefield(&mut game, grizzly_bears(), 0));
    let blocker = put_on_battlefield(&mut game, grizzly_bears(), 1);
    declare_attacks(&mut game, &[(bears, 1), (unblocked, 1)]);
    assert_eq!([power(&game, bears), power(&game, blocker)], [Some(2), Some(2)]);

    declare_blocks(&mut game, &[vec![(blocker, bears)]]);
    assert_eq!(power(&game, blocker), Some(3), "it blocked");
    assert_eq!(power(&game, bears), Some(3), "it was blocked");
    assert_eq!(power(&game, unblocked), Some(2), "it did neither");
}

/// A creature that entered this turn, against one that was there before.
#[test]
fn entered_this_turn_reads_the_permanent() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, banner("Fixture Newcomer's Banner", ObjectFilter::ThisTurn(ObjectFact::Entered)), 0);
    let old = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let new = put_on_battlefield_this_turn(&mut game, grizzly_bears(), 0);
    assert_eq!(power(&game, old), Some(2));
    assert_eq!(power(&game, new), Some(3));
}

/// A creature that left is asked as it last was: "whenever a creature that
/// entered this turn dies" reads the frame's entry turn (CR 603.10a).
#[test]
fn entered_this_turn_reads_a_departed_creatures_frame() {
    let mut game = stocked(2);
    let entered_this_turn =
        TriggerSubject::Filter(ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ThisTurn(ObjectFact::Entered))));
    let draw = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    let watcher = CardDataBuilder::new("Fixture Mourner")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(dies(entered_this_turn), draw)))
        .build();
    put_on_battlefield(&mut game, watcher, 0);
    let old = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let new = put_on_battlefield_this_turn(&mut game, grizzly_bears(), 0);
    let before = hand_size(&game, 0);

    for creature in [old, new] {
        game.change_zone(creature, Zone::Graveyard, mtgsim::engine::actions::ZoneChangeAct::Destroyed, &mtgsim::test_support::test_ctx()).unwrap();
    }
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&test_dp()).unwrap();
    }
    assert_eq!(hand_size(&game, 0), before + 1, "only the one that entered this turn");
}
