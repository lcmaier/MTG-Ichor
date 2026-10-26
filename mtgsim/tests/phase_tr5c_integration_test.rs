//! TR-5c — blocks, targeting and activation (`triggers-architecture.md` §12,
//! TR-5c).
//!
//! 1. The declaration of blockers is one record for the step (CR 509.1, 802.4),
//!    written when nobody blocks, with the unblocked creatures (CR 509.1h).
//! 2. "This turn" and "this combat": what an object did, read off the
//!    combat's history or off the permanent (CR 400.7).

use std::sync::Arc;

use mtgsim::cards::alpha::{giant_growth, lightning_bolt};
use mtgsim::cards::basic_lands::forest;
use mtgsim::cards::authoring::{
    activates_an_ability_of, attacks_and_isnt_blocked, becomes_blocked, becomes_blocked_by, becomes_the_target_of, blocks, blocks_a_creature, dies,
    player_becomes_the_target_of, triggered_ability, whenever, Whose,
};
use mtgsim::cards::phase_a4i_cards::seeds_of_strength;
use mtgsim::cards::phase_lh_cards::holy_strength;
use mtgsim::cards::phase_tr5c_cards::{cephalid_aristocrat, loyal_sentry};
use mtgsim::cards::utility_creatures::merfolk_thaumaturgist;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction};
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::ManaOutput;
use mtgsim::types::ids::AbilityId;
use mtgsim::types::card_types::{EnchantmentType, Subtype};
use mtgsim::types::mana::ManaType;
use mtgsim::ui::display::{format_record, log_lines};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::engine::actions::ZoneChangeAct;
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::get_effective_power;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::{
    fill_library, put_in_hand, put_on_battlefield, put_on_battlefield_this_turn, put_on_battlefield_under, set_active_player,
    set_attacking, setup_game, setup_two_player_game, static_ability, test_ctx, test_dp,
};
use mtgsim::cards::phase_re9_cards::deep_water;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    PlayerSet, Antecedent, AmountExpr, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::colors::Color;
use mtgsim::types::history::{CountIs, ObjectFact};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::triggers::{
    BlockOccurrence, EventRole, Multiplicity, TargetedBy, TriggerCondition, TriggerDef, TriggerEvent, TriggerLimit, TriggerSubject,
};
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

/// Every `BlockersDeclared` record, in order, each creature by id.
fn declarations(game: &GameState) -> Vec<(Vec<(ObjectId, ObjectId)>, Vec<ObjectId>)> {
    game.recorded_events()
        .events()
        .filter_map(|event| match event {
            GameEvent::BlockersDeclared { blocks, unblocked } => Some((
                blocks.iter().map(|(blocker, attacker)| (blocker.id, attacker.id)).collect(),
                unblocked.iter().map(|attacker| attacker.id).collect(),
            )),
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
            EffectRecipient::filtered_permanents(creatures),
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

/// And the converse: attacking without a declaration is not being unblocked.
/// A creature put onto the battlefield attacking before blockers are declared
/// can be blocked like any other, and then it is a blocked creature (CR
/// 509.1h), while the declared attacker no one blocked is the unblocked one.
#[test]
fn a_creature_attacking_without_a_declaration_can_be_blocked() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, life_on("Fixture Unblocked Watcher", attacks_and_isnt_blocked(TriggerSubject::Filter(ObjectFilter::All))), 0);
    put_on_battlefield(&mut game, life_on("Fixture Blocked Watcher", becomes_blocked(TriggerSubject::Filter(ObjectFilter::All))), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let entered = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let blocker = put_on_battlefield(&mut game, grizzly_bears(), 1);
    declare_attacks(&mut game, &[(bears, 1)]);
    set_attacking(&mut game, entered, 1);
    declare_blocks(&mut game, &[vec![(blocker, entered)]]);
    assert_eq!(declarations(&game), vec![(vec![(blocker, entered)], vec![bears])]);
    resolve_all(&mut game);
    assert_eq!(life(&game, 0), 21, "one unblocked creature, the declared one");
    assert_eq!(life(&game, 1), 21, "one blocked creature, the one that entered attacking");
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
    let draw = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You));
    let watcher = CardDataBuilder::new("Fixture Mourner")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(dies(entered_this_turn), draw)))
        .build();
    put_on_battlefield(&mut game, watcher, 0);
    let old = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let new = put_on_battlefield_this_turn(&mut game, grizzly_bears(), 0);
    let before = hand_size(&game, 0);

    for creature in [old, new] {
        game.change_zone(creature, Zone::Graveyard, ZoneChangeAct::Destroyed, &test_ctx()).unwrap();
    }
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&test_dp()).unwrap();
    }
    assert_eq!(hand_size(&game, 0), before + 1, "only the one that entered this turn");
}

// ---------------------------------------------------------------------------
// 3. The block arm (CR 509.3a–g)
// ---------------------------------------------------------------------------

/// Put every waiting trigger on the stack and resolve the stack.
fn resolve_all(game: &mut GameState) {
    let dp = test_dp();
    game.perform_sba_and_triggers(&dp).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&dp).unwrap();
        game.perform_sba_and_triggers(&dp).unwrap();
    }
}

/// An enchantment whose controller gains 1 life on each `arm`.
fn life_on(name: &str, arm: impl Into<TriggerEvent>) -> Arc<CardData> {
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You));
    CardDataBuilder::new(name).card_type(CardType::Enchantment).ability(triggered_ability("", whenever(arm, gain))).build()
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

/// Three attackers, one blocked by one creature, one by two, one unblocked:
/// the board each reading below counts on.
struct Board {
    game: GameState,
    attackers: [ObjectId; 3],
    /// The first attacker's blocker, then the second's two.
    blockers: [ObjectId; 3],
}

fn three_attackers_three_ways(watching: Arc<CardData>) -> Board {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, watching, 0);
    let attackers = [0, 1, 2].map(|_| put_on_battlefield(&mut game, grizzly_bears(), 0));
    let blockers = [0, 1, 2].map(|_| put_on_battlefield(&mut game, grizzly_bears(), 1));
    declare_attacks(&mut game, &attackers.map(|a| (a, 1)));
    declare_blocks(&mut game, &[vec![(blockers[0], attackers[0]), (blockers[1], attackers[1]), (blockers[2], attackers[1])]]);
    Board { game, attackers, blockers }
}

/// CR 509.3a: once for each blocking creature.
#[test]
fn blocks_triggers_once_for_each_blocking_creature() {
    let mut board = three_attackers_three_ways(life_on("Fixture Wall Watcher", blocks(TriggerSubject::Filter(ObjectFilter::All))));
    resolve_all(&mut board.game);
    assert_eq!(life(&board.game, 0), 23);
}

/// CR 509.3c: once for each blocked creature, however many block it.
#[test]
fn becomes_blocked_triggers_once_for_each_blocked_creature() {
    let mut board = three_attackers_three_ways(life_on("Fixture Charge Watcher", becomes_blocked(TriggerSubject::Filter(ObjectFilter::All))));
    resolve_all(&mut board.game);
    assert_eq!(life(&board.game, 0), 22, "two blocked creatures, one of them twice");
}

/// CR 509.3d: once for each creature blocking it.
#[test]
fn becomes_blocked_by_triggers_once_for_each_blocker() {
    let mut board =
        three_attackers_three_ways(life_on("Fixture Melee Watcher", becomes_blocked_by(TriggerSubject::Filter(ObjectFilter::All), TriggerSubject::Filter(ObjectFilter::All))));
    resolve_all(&mut board.game);
    assert_eq!(life(&board.game, 0), 23);
}

fn blocked_by(count: CountIs) -> TriggerEvent {
    TriggerEvent::Blocks {
        blocker: TriggerSubject::Filter(ObjectFilter::All),
        attacker: TriggerSubject::Filter(ObjectFilter::All),
        occurrence: BlockOccurrence::Blocked,
        count: Some(count),
        alone: false,
        multiplicity: Multiplicity::PerOccurrence,
    }
}

/// CR 509.3e: "becomes blocked by two or more creatures" triggers for the one
/// that was, and "by exactly one" for the other.
#[test]
fn a_count_of_blockers_is_asked_when_blockers_are_declared() {
    let mut board = three_attackers_three_ways(life_on("Fixture Crowd Watcher", blocked_by(CountIs::AtLeast(2))));
    resolve_all(&mut board.game);
    assert_eq!(life(&board.game, 0), 21);

    let mut board = three_attackers_three_ways(life_on("Fixture Duel Watcher", blocked_by(CountIs::Exactly(1))));
    resolve_all(&mut board.game);
    assert_eq!(life(&board.game, 0), 21);
}

/// "Whenever this creature becomes blocked, tap each creature blocking it":
/// a blocked creature's triggering event holds every blocker it counts, and
/// "them" is all of them.
#[test]
fn a_blocked_creatures_event_holds_every_blocker() {
    let mut game = stocked(2);
    let tap_them = Effect::Atom(Primitive::Tap, EffectRecipient::Object(Antecedent::Triggering(EventRole::Blocker)));
    let charger = CardDataBuilder::new("Fixture Charger")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(triggered_ability("", whenever(becomes_blocked(TriggerSubject::Object(Antecedent::ThisObject)), tap_them)))
        .build();
    let charger = put_on_battlefield(&mut game, charger, 0);
    let (first, second) = (put_on_battlefield(&mut game, grizzly_bears(), 1), put_on_battlefield(&mut game, grizzly_bears(), 1));
    declare_attacks(&mut game, &[(charger, 1)]);
    declare_blocks(&mut game, &[vec![(first, charger), (second, charger)]]);
    resolve_all(&mut game);
    assert!(game.battlefield[&first].tapped && game.battlefield[&second].tapped);
}

/// CR 509.3g: the unblocked creature triggers, and a blocked creature whose
/// blocker has left does not: the declaration decided it (CR 509.1h).
// COVERS-PARTIAL: ATOM-509.1h-001
#[test]
fn attacks_and_isnt_blocked_reads_the_declaration() {
    let mut board = three_attackers_three_ways(life_on("Fixture Raid Watcher", attacks_and_isnt_blocked(TriggerSubject::Filter(ObjectFilter::All))));
    board.game.change_zone(board.blockers[0], Zone::Graveyard, ZoneChangeAct::Destroyed, &test_ctx()).unwrap();
    resolve_all(&mut board.game);
    assert_eq!(life(&board.game, 0), 21, "the third attacker only");
    assert!(board.game.battlefield[&board.attackers[0]].attacking.as_ref().unwrap().is_blocked, "still blocked");
}

/// CR 509.3f: a blocked creature's characteristics as it became blocked. Of
/// two blocked creatures, the red one triggers "whenever a red creature
/// becomes blocked".
#[test]
fn a_filter_reads_the_creature_as_blockers_are_declared() {
    let red = TriggerSubject::Filter(ObjectFilter::ByColor(Color::Red));
    let mut game = stocked(2);
    put_on_battlefield(&mut game, life_on("Fixture Red Watcher", becomes_blocked(red)), 0);
    let red_bears = CardDataBuilder::new("Fixture Red Bears").card_type(CardType::Creature).color(Color::Red).power_toughness(2, 2).build();
    let (red_one, green_one) = (put_on_battlefield(&mut game, red_bears, 0), put_on_battlefield(&mut game, grizzly_bears(), 0));
    let (b1, b2) = (put_on_battlefield(&mut game, grizzly_bears(), 1), put_on_battlefield(&mut game, grizzly_bears(), 1));
    declare_attacks(&mut game, &[(red_one, 1), (green_one, 1)]);
    declare_blocks(&mut game, &[vec![(b1, red_one), (b2, green_one)]]);
    resolve_all(&mut game);
    assert_eq!(life(&game, 0), 21);
}

fn blocks_alone() -> TriggerEvent {
    TriggerEvent::Blocks {
        blocker: TriggerSubject::Filter(ObjectFilter::All),
        attacker: TriggerSubject::Filter(ObjectFilter::All),
        occurrence: BlockOccurrence::Blocker,
        count: None,
        alone: true,
        multiplicity: Multiplicity::PerOccurrence,
    }
}

/// CR 506.5: "blocks alone" is the only creature declared as a blocker, across
/// every defending player (CR 802.4).
#[test]
fn blocks_alone_is_the_only_blocker_across_every_defender() {
    let mut game = stocked(3);
    put_on_battlefield(&mut game, life_on("Fixture Lone Watcher", blocks_alone()), 0);
    let (first, second) = (put_on_battlefield(&mut game, grizzly_bears(), 0), put_on_battlefield(&mut game, grizzly_bears(), 0));
    let (b1, b2) = (put_on_battlefield(&mut game, grizzly_bears(), 1), put_on_battlefield(&mut game, grizzly_bears(), 2));
    declare_attacks(&mut game, &[(first, 1), (second, 2)]);
    declare_blocks(&mut game, &[vec![(b1, first)], vec![(b2, second)]]);
    resolve_all(&mut game);
    assert_eq!(life(&game, 0), 20, "two players blocked, one creature each");

    let mut game = stocked(3);
    put_on_battlefield(&mut game, life_on("Fixture Lone Watcher", blocks_alone()), 0);
    let (first, second) = (put_on_battlefield(&mut game, grizzly_bears(), 0), put_on_battlefield(&mut game, grizzly_bears(), 0));
    let b1 = put_on_battlefield(&mut game, grizzly_bears(), 1);
    put_on_battlefield(&mut game, grizzly_bears(), 2);
    declare_attacks(&mut game, &[(first, 1), (second, 2)]);
    declare_blocks(&mut game, &[vec![(b1, first)], vec![]]);
    resolve_all(&mut game);
    assert_eq!(life(&game, 0), 21);
}

/// Cockatrice's shape: "whenever this creature blocks or becomes blocked by a
/// creature, destroy that creature".
fn basilisk() -> Arc<CardData> {
    let destroy = Effect::Atom(Primitive::Destroy, EffectRecipient::Object(Antecedent::Triggering(EventRole::OtherCombatant)));
    let def = TriggerDef {
        condition: TriggerCondition::AnyOf(vec![
            blocks_a_creature(TriggerSubject::Object(Antecedent::ThisObject), TriggerSubject::Filter(ObjectFilter::All)).into(),
            becomes_blocked_by(TriggerSubject::Object(Antecedent::ThisObject), TriggerSubject::Filter(ObjectFilter::All)).into(),
        ]),
        intervening_if: None,
        limit: None,
        effect: destroy,
    };
    CardDataBuilder::new("Fixture Basilisk").card_type(CardType::Creature).power_toughness(1, 3).ability(triggered_ability("", def)).build()
}

/// "That creature" is the attacker when it blocks and the blocker when it is
/// blocked, and never the creature itself.
#[test]
fn the_other_creature_in_a_block_is_read_through_the_arm_that_matched() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let blocking = put_on_battlefield(&mut game, basilisk(), 1);
    declare_attacks(&mut game, &[(bears, 1)]);
    declare_blocks(&mut game, &[vec![(blocking, bears)]]);
    resolve_all(&mut game);
    assert!(!game.battlefield.contains_key(&bears), "it blocked the bears");
    assert!(game.battlefield.contains_key(&blocking));

    let mut game = stocked(2);
    let attacking = put_on_battlefield(&mut game, basilisk(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    declare_attacks(&mut game, &[(attacking, 1)]);
    declare_blocks(&mut game, &[vec![(bears, attacking)]]);
    resolve_all(&mut game);
    assert!(!game.battlefield.contains_key(&bears), "the bears blocked it");
    assert!(game.battlefield.contains_key(&attacking));
}

/// The registry's refusals for the block arm: a reading no CR 509.3 rule
/// combines, a side of a block named by event role where the arm fixes it to
/// the source, and the other creature on an arm with no side of its own.
#[test]
fn the_registry_refuses_block_readings_no_rule_gives() {
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You));
    let unblocked_by = TriggerEvent::Blocks {
        blocker: TriggerSubject::Object(Antecedent::ThisObject),
        attacker: TriggerSubject::Filter(ObjectFilter::All),
        occurrence: BlockOccurrence::Unblocked,
        count: None,
        alone: false,
        multiplicity: Multiplicity::PerOccurrence,
    };
    assert_eq!(whenever(unblocked_by, gain.clone()).readings_its_arms_cannot_give().len(), 1);
    let counted_block = TriggerEvent::Blocks {
        blocker: TriggerSubject::Filter(ObjectFilter::All),
        attacker: TriggerSubject::Filter(ObjectFilter::All),
        occurrence: BlockOccurrence::Block,
        count: Some(CountIs::AtLeast(2)),
        alone: false,
        multiplicity: Multiplicity::PerOccurrence,
    };
    assert_eq!(whenever(counted_block, gain).readings_its_arms_cannot_give().len(), 1);
    let destroy = |role| Effect::Atom(Primitive::Destroy, EffectRecipient::Object(Antecedent::Triggering(role)));
    let own_side = whenever(becomes_blocked_by(TriggerSubject::Object(Antecedent::ThisObject), TriggerSubject::Filter(ObjectFilter::All)), destroy(EventRole::Attacker));
    assert_eq!(own_side.readings_its_arms_cannot_give().len(), 1, "its own side, by event role");
    let no_side = whenever(becomes_blocked_by(TriggerSubject::Filter(ObjectFilter::All), TriggerSubject::Filter(ObjectFilter::All)), destroy(EventRole::OtherCombatant));
    assert_eq!(no_side.readings_its_arms_cannot_give().len(), 1);
    let fine = whenever(becomes_blocked_by(TriggerSubject::Object(Antecedent::ThisObject), TriggerSubject::Filter(ObjectFilter::All)), destroy(EventRole::OtherCombatant));
    assert!(fine.readings_its_arms_cannot_give().is_empty());
}

/// A block arm is about the side its text names: "when that creature becomes
/// blocked by a creature" watches the creature its delayed trigger refers to
/// (CR 603.7c), on the attacker's side, as "when that creature blocks a
/// creature" does on the blocker's. An unblocked creature has no other
/// creature to name.
#[test]
fn a_block_arm_is_about_the_side_it_names() {
    let blocked: TriggerEvent = becomes_blocked_by(TriggerSubject::Object(Antecedent::Referred), TriggerSubject::Filter(ObjectFilter::All)).into();
    assert!(blocked.watches_referred());
    assert_eq!(blocked.subject(), Some(&TriggerSubject::Object(Antecedent::Referred)));
    let blocking: TriggerEvent = blocks_a_creature(TriggerSubject::Object(Antecedent::Referred), TriggerSubject::Filter(ObjectFilter::All)).into();
    assert!(blocking.watches_referred());
    let destroy = Effect::Atom(Primitive::Destroy, EffectRecipient::Object(Antecedent::Triggering(EventRole::OtherCombatant)));
    let unblocked = whenever(attacks_and_isnt_blocked(TriggerSubject::Object(Antecedent::ThisObject)), destroy);
    assert_eq!(unblocked.readings_its_arms_cannot_give().len(), 1, "no creature across an unblocked one");
}

// ---------------------------------------------------------------------------
// 4. Loyal Sentry, and one instruction on two objects
// ---------------------------------------------------------------------------

/// Its ruling's first sentence: both are destroyed before combat damage, in
/// one event, which a "one or more creatures die" trigger sees once.
// RULING: Loyal Sentry #1 - "Loyal Sentry and the creature it blocks are destroyed before combat damage is dealt."
#[test]
fn loyal_sentry_destroys_both_before_combat_damage() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, life_on("Fixture Mourning Bell", dies(TriggerSubject::Filter(ObjectFilter::All)).once_per_event()), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let sentry = put_on_battlefield(&mut game, loyal_sentry(), 1);
    declare_attacks(&mut game, &[(bears, 1)]);
    declare_blocks(&mut game, &[vec![(sentry, bears)]]);
    resolve_all(&mut game);
    assert!(!game.battlefield.contains_key(&bears) && !game.battlefield.contains_key(&sentry));
    assert_eq!(life(&game, 1), 21, "one event, so the bell rings once");
    game.process_combat_damage(&test_dp(), false).unwrap();
    assert_eq!(life(&game, 1), 21, "nothing left to deal combat damage");
}

/// Its ruling's second sentence: the blocked creature is destroyed even if
/// the Sentry has left before its trigger resolves.
// RULING: Loyal Sentry #1 - "The blocked creature is destroyed even if Loyal Sentry leaves the battlefield before its triggered ability resolves."
#[test]
fn loyal_sentry_destroys_the_attacker_after_it_has_left() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let sentry = put_on_battlefield(&mut game, loyal_sentry(), 1);
    declare_attacks(&mut game, &[(bears, 1)]);
    declare_blocks(&mut game, &[vec![(sentry, bears)]]);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(game.stack.len(), 1);
    game.change_zone(sentry, Zone::Hand, ZoneChangeAct::Put, &test_ctx()).unwrap();
    resolve_all(&mut game);
    assert!(!game.battlefield.contains_key(&bears));
    assert_eq!(game.objects[&sentry].zone, Zone::Hand, "a new object in hand, which the trigger does not know");
}

/// An indestructible attacker survives, and the Sentry still goes: each is
/// destroyed or not on its own terms (CR 702.12b).
#[test]
fn loyal_sentry_still_dies_beside_an_indestructible_attacker() {
    let mut game = stocked(2);
    let darksteel = CardDataBuilder::new("Fixture Darksteel Bear")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .keyword_flag(KeywordFlag::Indestructible)
        .build();
    let bear = put_on_battlefield(&mut game, darksteel, 0);
    let sentry = put_on_battlefield(&mut game, loyal_sentry(), 1);
    declare_attacks(&mut game, &[(bear, 1)]);
    declare_blocks(&mut game, &[vec![(sentry, bear)]]);
    resolve_all(&mut game);
    assert!(game.battlefield.contains_key(&bear) && !game.battlefield.contains_key(&sentry));
}

// ---------------------------------------------------------------------------
// 5. Becoming the target (CR 115.1)
// ---------------------------------------------------------------------------

/// Empty `player`'s pool, fill it with exactly `pool`, and cast `card` from
/// hand, each instance of "target" answered with the next of `targets`.
fn cast_at(game: &mut GameState, player: PlayerId, card: Arc<CardData>, pool: &[(ManaType, u64)], targets: &[ObjectId]) -> ObjectId {
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
    let aim = ScriptedDecisionProvider::new();
    for &target in targets {
        aim.expect_choice(
            ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
            vec![ChoiceOption::Object(target)],
        );
    }
    game.cast_spell(player, id, &ManaWindowStop::new(aim)).expect("castable from exactly its cost");
    id
}

fn bolt_at(game: &mut GameState, player: PlayerId, target: ObjectId) -> ObjectId {
    cast_at(game, player, lightning_bolt(), &[(ManaType::Red, 1)], &[target])
}

/// `player` casts Lightning Bolt at a player.
fn bolt_at_player(game: &mut GameState, player: PlayerId, at: PlayerId) -> ObjectId {
    let id = put_in_hand(game, lightning_bolt(), player);
    game.players[player].mana_pool.add(ManaType::Red, 1);
    let aim = ScriptedDecisionProvider::new();
    aim.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
        vec![ChoiceOption::Player(at)],
    );
    game.cast_spell(player, id, &ManaWindowStop::new(aim)).expect("castable");
    id
}

/// `player` activates Merfolk Thaumaturgist's `{T}` ability at `target`.
fn thaumaturgist_at(game: &mut GameState, player: PlayerId, thaumaturgist: ObjectId, target: ObjectId) {
    let abilities = mtgsim::oracle::characteristics::get_effective_abilities(game, thaumaturgist);
    let index = abilities.iter().position(|a| a.ability_type == AbilityType::Activated).expect("an activated ability");
    let aim = ScriptedDecisionProvider::new();
    aim.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
        vec![ChoiceOption::Object(target)],
    );
    game.activate_ability(player, thaumaturgist, index, &ManaWindowStop::new(aim)).expect("activated");
}

/// A creature beside the one aimed at, so the target is a choice: a spell
/// with one legal target takes it without a prompt.
fn bystander(game: &mut GameState) -> ObjectId {
    put_on_battlefield(game, grizzly_bears(), 1)
}

fn library_size(game: &GameState, player: PlayerId) -> usize {
    game.players[player].library.len()
}

/// Every waiting trigger onto the stack, its targets answered by `dp`.
fn place_triggers(game: &mut GameState, dp: &ScriptedDecisionProvider) {
    game.perform_sba_and_triggers(dp).unwrap();
}

/// **Fixture.** Frost Titan's first ability without its payment, which is
/// `hybrid-and-phyrexian-payment`'s: "Whenever this creature becomes the target of a spell or ability
/// an opponent controls, counter that spell or ability."
fn frost_titan_fixture() -> Arc<CardData> {
    let counter = Effect::Atom(Primitive::Counter, EffectRecipient::Object(Antecedent::Triggering(EventRole::SpellOrAbility)));
    CardDataBuilder::new("Fixture Frost Titan")
        .card_type(CardType::Creature)
        .color(Color::Blue)
        .power_toughness(6, 6)
        .ability(triggered_ability(
            "",
            whenever(becomes_the_target_of(TriggerSubject::Object(Antecedent::ThisObject), TargetedBy::SpellOrAbility, PlayerSet::Opponents), counter),
        ))
        .build()
}

/// Cephalid Aristocrat mills two for its controller's own spell, once however
/// many of the spell's instances chose it (CR 115.9a counts them only for
/// "with N targets"), and the spell resolves after it.
#[test]
fn cephalid_aristocrat_mills_once_for_each_spell_its_own_included() {
    let mut game = stocked(2);
    let aristocrat = put_on_battlefield(&mut game, cephalid_aristocrat(), 0);
    bystander(&mut game);
    let before = library_size(&game, 0);
    cast_at(&mut game, 0, seeds_of_strength(), &[(ManaType::Green, 1), (ManaType::White, 1)], &[aristocrat, aristocrat, aristocrat]);
    let targeted: Vec<u32> = game
        .recorded_events()
        .events()
        .filter_map(|event| match event {
            GameEvent::Targeted { instances, .. } => Some(*instances),
            _ => None,
        })
        .collect();
    assert_eq!(targeted, vec![3], "one record for one spell, three instances");
    resolve_all(&mut game);
    assert_eq!(library_size(&game, 0), before - 2);
    assert_eq!(power(&game, aristocrat), Some(6), "Seeds of Strength resolved after it");
}

/// An opponent's spell and an opponent's activated ability each trigger it.
#[test]
fn cephalid_aristocrat_mills_for_an_opponents_spell_and_ability() {
    let mut game = stocked(2);
    let aristocrat = put_on_battlefield(&mut game, cephalid_aristocrat(), 0);
    let thaumaturgist = put_on_battlefield(&mut game, merfolk_thaumaturgist(), 1);
    let before = library_size(&game, 0);
    set_active_player(&mut game, 1);
    thaumaturgist_at(&mut game, 1, thaumaturgist, aristocrat);
    resolve_all(&mut game);
    assert_eq!(library_size(&game, 0), before - 2, "an activated ability");
    bolt_at(&mut game, 1, aristocrat);
    resolve_all(&mut game);
    assert_eq!(library_size(&game, 0), before - 4, "a spell");
}

/// Frost Titan's first ruling: a spell, an Aura spell, an activated ability
/// and a triggered ability, each controlled by an opponent, are countered.
// RULING: Frost Titan #1 - "Frost Titan's first ability affects each spell (including Aura spells), activated ability, and triggered ability that's controlled by an opponent and that Frost Titan becomes a target of."
#[test]
fn frost_titan_counters_an_opponents_spell_and_activated_ability() {
    let mut game = stocked(2);
    let titan = put_on_battlefield(&mut game, frost_titan_fixture(), 0);
    let thaumaturgist = put_on_battlefield(&mut game, merfolk_thaumaturgist(), 1);
    set_active_player(&mut game, 1);
    let bolt = bolt_at(&mut game, 1, titan);
    resolve_all(&mut game);
    assert_eq!(game.objects[&bolt].zone, Zone::Graveyard, "countered");
    assert_eq!(game.battlefield[&titan].damage_marked, 0);

    thaumaturgist_at(&mut game, 1, thaumaturgist, titan);
    resolve_all(&mut game);
    assert_eq!(power(&game, titan), Some(6), "the switch was countered");
    let countered = game.recorded_events().events().filter(|e| matches!(e, GameEvent::AbilityCountered { .. })).count();
    assert_eq!(countered, 1);
}

// RULING: Frost Titan #1 - "Frost Titan's first ability affects each spell (including Aura spells), activated ability, and triggered ability that's controlled by an opponent and that Frost Titan becomes a target of."
#[test]
fn frost_titan_counters_an_opponents_aura_spell() {
    let mut game = stocked(2);
    let titan = put_on_battlefield(&mut game, frost_titan_fixture(), 0);
    bystander(&mut game);
    set_active_player(&mut game, 1);
    let aura = cast_at(&mut game, 1, holy_strength(), &[(ManaType::White, 1)], &[titan]);
    resolve_all(&mut game);
    assert_eq!(game.objects[&aura].zone, Zone::Graveyard, "the Aura spell is countered");
    assert_eq!(power(&game, titan), Some(6));
}

// RULING: Frost Titan #1 - "Frost Titan's first ability affects each spell (including Aura spells), activated ability, and triggered ability that's controlled by an opponent and that Frost Titan becomes a target of."
#[test]
fn frost_titan_counters_an_opponents_triggered_ability() {
    let mut game = stocked(2);
    let titan = put_on_battlefield(&mut game, frost_titan_fixture(), 0);
    let tap_one = Effect::Atom(Primitive::Tap, EffectRecipient::Target(SelectionFilter::creature(), TargetCount::Exactly(1)));
    let watcher = CardDataBuilder::new("Fixture Grim Watcher")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(dies(TriggerSubject::Filter(ObjectFilter::All)), tap_one)))
        .build();
    put_on_battlefield(&mut game, watcher, 1);
    bystander(&mut game);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.change_zone(bears, Zone::Graveyard, ZoneChangeAct::Destroyed, &test_ctx()).unwrap();
    let aim = ScriptedDecisionProvider::new();
    aim.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
        vec![ChoiceOption::Object(titan)],
    );
    place_triggers(&mut game, &aim);
    resolve_all(&mut game);
    assert!(!game.battlefield[&titan].tapped, "the tap trigger was countered");
}

/// ATOM-702.21a-002's trigger half: its controller's own spell does not
/// trigger "an opponent controls", and resolves.
// COVERS-PARTIAL: ATOM-702.21a-002
#[test]
fn frost_titan_ignores_its_controllers_own_spell() {
    let mut game = stocked(2);
    let titan = put_on_battlefield(&mut game, frost_titan_fixture(), 0);
    bystander(&mut game);
    cast_at(&mut game, 0, giant_growth(), &[(ManaType::Green, 1)], &[titan]);
    resolve_all(&mut game);
    assert_eq!(power(&game, titan), Some(9));
}

/// "That spell or ability" is the one that targeted it: of two spells on the
/// stack, Frost Titan counters the one aimed at it.
#[test]
fn that_spell_or_ability_is_the_one_that_targeted_it() {
    let mut game = stocked(2);
    let titan = put_on_battlefield(&mut game, frost_titan_fixture(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    set_active_player(&mut game, 1);
    let at_bears = bolt_at(&mut game, 1, bears);
    let at_titan = bolt_at(&mut game, 1, titan);
    resolve_all(&mut game);
    assert_eq!(game.objects[&at_titan].zone, Zone::Graveyard);
    assert!(!game.battlefield.contains_key(&bears), "the other bolt resolved");
    let countered: Vec<ObjectId> = game
        .recorded_events()
        .events()
        .filter_map(|e| match e {
            GameEvent::SpellCountered { lki, .. } => Some(lki.object().id),
            _ => None,
        })
        .collect();
    assert_eq!(countered, vec![at_titan]);
    let _ = at_bears;
}

/// The arm's player leg: "whenever you become the target of a spell or
/// ability an opponent controls" triggers on an opponent's Bolt at you, and
/// not on your own.
#[test]
fn you_becoming_the_target_of_an_opponents_spell_triggers_it() {
    let mut game = stocked(2);
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(2)), EffectRecipient::Player(PlayerRef::You));
    let amulet = CardDataBuilder::new("Fixture Amulet")
        .card_type(CardType::Artifact)
        .ability(triggered_ability("", whenever(player_becomes_the_target_of(Whose::Yours, TargetedBy::SpellOrAbility, PlayerSet::Opponents), gain)))
        .build();
    put_on_battlefield(&mut game, amulet, 0);
    set_active_player(&mut game, 1);
    bolt_at_player(&mut game, 1, 0);
    resolve_all(&mut game);
    assert_eq!(life(&game, 0), 19, "2 gained, then 3 dealt");
    bolt_at_player(&mut game, 0, 0);
    resolve_all(&mut game);
    assert_eq!(life(&game, 0), 16, "its own spell gains nothing");
}

/// CR 601.2c announces a spell's targets at once, so "one or more creatures
/// you control become the target" is once for a spell that targets two, and
/// "a creature you control" twice.
#[test]
fn one_spells_targets_are_one_event() {
    let yours = TriggerSubject::Filter(ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerSet::You)),
    ));
    for (arm, gained) in [
        (becomes_the_target_of(yours.clone(), TargetedBy::SpellOrAbility, PlayerSet::Everyone).once_per_event(), 1),
        (becomes_the_target_of(yours.clone(), TargetedBy::SpellOrAbility, PlayerSet::Everyone).into(), 2),
    ] {
        let mut game = stocked(2);
        put_on_battlefield(&mut game, life_on("Fixture Guardian", arm), 0);
        let (first, second) = (put_on_battlefield(&mut game, grizzly_bears(), 0), put_on_battlefield(&mut game, grizzly_bears(), 0));
        cast_at(&mut game, 0, seeds_of_strength(), &[(ManaType::Green, 1), (ManaType::White, 1)], &[first, second, first]);
        resolve_all(&mut game);
        assert_eq!(life(&game, 0), 20 + gained);
    }
}

/// Heartfire Hero's valiant: "whenever this creature becomes the target of a
/// spell or ability you control for the first time each turn, put a +1/+1
/// counter on it".
fn valiant_fixture() -> Arc<CardData> {
    let counter = Effect::Atom(Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You }, EffectRecipient::Object(Antecedent::ThisObject));
    let def = TriggerDef {
        limit: Some(TriggerLimit::FirstTimeEachTurn),
        ..whenever(becomes_the_target_of(TriggerSubject::Object(Antecedent::ThisObject), TargetedBy::SpellOrAbility, PlayerSet::You), counter)
    };
    CardDataBuilder::new("Fixture Valiant Mouse").card_type(CardType::Creature).power_toughness(1, 1).ability(triggered_ability("", def)).build()
}

/// Valiant counts its controller's targetings only: an opponent's first does
/// not use it up, its controller's first triggers, and its second does not.
/// Its trigger resolves before the spell (Heartfire Hero #2).
// RULING: Heartfire Hero #2 - "Valiant abilities will resolve before the spell or ability that caused them to trigger."
#[test]
fn valiant_triggers_the_first_time_its_controller_targets_it_each_turn() {
    let mut game = stocked(2);
    let mouse = put_on_battlefield(&mut game, valiant_fixture(), 0);
    let thaumaturgist = put_on_battlefield(&mut game, merfolk_thaumaturgist(), 1);
    let counters = |game: &GameState| game.battlefield[&mouse].counters.get(&CounterType::PlusOnePlusOne).map_or(0, |c| c.count);

    thaumaturgist_at(&mut game, 1, thaumaturgist, mouse);
    resolve_all(&mut game);
    assert_eq!(counters(&game), 0, "an opponent's");

    let growth = cast_at(&mut game, 0, giant_growth(), &[(ManaType::Green, 1)], &[mouse]);
    place_triggers(&mut game, &ScriptedDecisionProvider::new());
    assert_eq!(game.stack.len(), 2);
    assert_ne!(game.stack.last(), Some(&growth), "the trigger is on top of the spell");
    resolve_all(&mut game);
    assert_eq!(counters(&game), 1, "its controller's first");

    cast_at(&mut game, 0, giant_growth(), &[(ManaType::Green, 1)], &[mouse]);
    resolve_all(&mut game);
    assert_eq!(counters(&game), 1, "not the second");
}

/// A new turn starts the count over: the rows are the turn's.
#[test]
fn valiant_triggers_again_next_turn() {
    let mut game = stocked(2);
    let mouse = put_on_battlefield(&mut game, valiant_fixture(), 0);
    bystander(&mut game);
    cast_at(&mut game, 0, giant_growth(), &[(ManaType::Green, 1)], &[mouse]);
    resolve_all(&mut game);
    game.begin_turn(2, 1);
    game.begin_turn(3, 0);
    cast_at(&mut game, 0, giant_growth(), &[(ManaType::Green, 1)], &[mouse]);
    resolve_all(&mut game);
    assert_eq!(game.battlefield[&mouse].counters.get(&CounterType::PlusOnePlusOne).map_or(0, |c| c.count), 2);
}

/// Kira's "for the first time each turn" names no controller: the first
/// spell is countered whoever cast it, and the second resolves.
#[test]
fn a_first_time_for_any_controller_counts_every_targeting() {
    let mut game = stocked(2);
    let counter = Effect::Atom(Primitive::Counter, EffectRecipient::Object(Antecedent::Triggering(EventRole::SpellOrAbility)));
    let def = TriggerDef {
        limit: Some(TriggerLimit::FirstTimeEachTurn),
        ..whenever(becomes_the_target_of(TriggerSubject::Object(Antecedent::ThisObject), TargetedBy::SpellOrAbility, PlayerSet::Everyone), counter)
    };
    let spinner = CardDataBuilder::new("Fixture Glass Spinner").card_type(CardType::Creature).power_toughness(2, 2).ability(triggered_ability("", def)).build();
    let spinner = put_on_battlefield(&mut game, spinner, 0);
    bystander(&mut game);
    let first = cast_at(&mut game, 0, giant_growth(), &[(ManaType::Green, 1)], &[spinner]);
    resolve_all(&mut game);
    assert_eq!(game.objects[&first].zone, Zone::Graveyard);
    assert_eq!(power(&game, spinner), Some(2), "the first was countered");
    cast_at(&mut game, 0, giant_growth(), &[(ManaType::Green, 1)], &[spinner]);
    resolve_all(&mut game);
    assert_eq!(power(&game, spinner), Some(5), "the second resolved");
}

/// The registry refuses "for the first time each turn" on an arm the engine
/// cannot answer it for. A death has no place in the turn on its record; a
/// targeting's first time is read off the turn's targeting rows, which keep
/// no spell's characteristics, so "an Aura spell" is refused there while "a
/// spell" is accepted.
#[test]
fn first_time_each_turn_is_refused_where_nothing_can_tell_a_first() {
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You));
    let first = |arm: TriggerEvent| TriggerDef { limit: Some(TriggerLimit::FirstTimeEachTurn), ..whenever(arm, gain.clone()) };
    assert_eq!(first(dies(TriggerSubject::Filter(ObjectFilter::All)).into()).readings_its_arms_cannot_give().len(), 1);
    let aura = TargetedBy::Spell(Some(ObjectFilter::BySubtype(Subtype::Enchantment(EnchantmentType::Aura))));
    assert_eq!(first(becomes_the_target_of(TriggerSubject::Object(Antecedent::ThisObject), aura, PlayerSet::Everyone).into()).readings_its_arms_cannot_give().len(), 1);
    assert!(first(becomes_the_target_of(TriggerSubject::Object(Antecedent::ThisObject), TargetedBy::Spell(None), PlayerSet::Everyone).into())
        .readings_its_arms_cannot_give()
        .is_empty());
}

// ---------------------------------------------------------------------------
// 6. Activating an ability (CR 602.2, 605.3a)
// ---------------------------------------------------------------------------

/// "A [creature/artifact/land] or [...]", for an activation's source.
fn of_type(types: &[CardType]) -> TriggerSubject {
    let mut filters = types.iter().map(|&t| ObjectFilter::ByType(t));
    let first = filters.next().unwrap();
    TriggerSubject::Filter(filters.fold(first, |either, or| ObjectFilter::Or(Box::new(either), Box::new(or))))
}

/// **Fixture.** Harsh Mentor's shape: "Whenever an opponent activates an
/// ability of an artifact, creature, or land on the battlefield, if it isn't a
/// mana ability, this creature deals 2 damage to that player."
fn mentor_fixture() -> Arc<CardData> {
    let two = Effect::Atom(Primitive::DealDamage { amount: AmountExpr::Fixed(2), unpreventable: false }, EffectRecipient::Player(PlayerRef::TheEventsPlayer));
    let arm = activates_an_ability_of(Whose::AnOpponents, of_type(&[CardType::Artifact, CardType::Creature, CardType::Land]), Some(false));
    CardDataBuilder::new("Fixture Mentor").card_type(CardType::Creature).power_toughness(2, 2).ability(triggered_ability("", whenever(arm, two))).build()
}

/// `player` taps `permanent` for its mana ability.
fn tap_for_mana(game: &mut GameState, player: PlayerId, permanent: ObjectId) {
    let abilities = mtgsim::oracle::characteristics::get_effective_abilities(game, permanent);
    let ability = abilities.iter().find(|a| a.ability_type == AbilityType::Mana).expect("a mana ability").id;
    game.activate_mana_ability(player, permanent, ability, &test_dp()).unwrap();
}

/// An opponent's activation triggers it, and its controller's does not; the
/// trigger goes on the stack above the ability, so it resolves first.
// RULING: Harsh Mentor #3 - "Harsh Mentor's ability resolves before the ability that caused it to trigger."
#[test]
fn an_opponents_activation_triggers_and_resolves_first() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, mentor_fixture(), 0);
    let theirs = put_on_battlefield(&mut game, merfolk_thaumaturgist(), 1);
    let mine = put_on_battlefield(&mut game, merfolk_thaumaturgist(), 0);
    set_active_player(&mut game, 1);
    thaumaturgist_at(&mut game, 1, theirs, mine);
    place_triggers(&mut game, &ScriptedDecisionProvider::new());
    assert_eq!(game.stack.len(), 2, "the ability, and the trigger above it");
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(life(&game, 1), 18, "the trigger resolved first");
    resolve_all(&mut game);

    set_active_player(&mut game, 0);
    thaumaturgist_at(&mut game, 0, mine, theirs);
    resolve_all(&mut game);
    assert_eq!(life(&game, 1), 18, "its controller's own activation");
}

/// **Fixture.** Deathrite Shaman's first ability, with a player for its
/// target: "{T}: Target player gains 1 life. Add {C}."
fn shaman_fixture() -> Arc<CardData> {
    let gain = Effect::Atom(
        Primitive::GainLife(AmountExpr::Fixed(1)),
        EffectRecipient::Target(SelectionFilter::Player(PlayerSet::Everyone), TargetCount::Exactly(1)),
    );
    let add = Effect::Atom(
        Primitive::ProduceMana(ManaOutput { mana: vec![(ManaType::Colorless, AmountExpr::Fixed(1))], special: vec![] }),
        EffectRecipient::Player(PlayerRef::You),
    );
    CardDataBuilder::new("Fixture Shaman")
        .card_type(CardType::Creature)
        .power_toughness(1, 2)
        .ability(AbilityDef {
            rules_text: "{T}: Target player gains 1 life. Add {C}.".into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::TapSelf],
            effect: Effect::Sequence(vec![gain, add]),
        })
        .build()
}

/// An activated ability that adds mana but has a target is no mana ability
/// (CR 605.1a; Deathrite Shaman's first ruling: "it uses the stack and can be
/// responded to"). Its record says so, so Harsh Mentor's "if it isn't a mana
/// ability" triggers on it, and the mana comes as it resolves.
#[test]
fn an_ability_that_adds_mana_with_a_target_is_no_mana_ability() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, mentor_fixture(), 0);
    let shaman = put_on_battlefield(&mut game, shaman_fixture(), 1);
    set_active_player(&mut game, 1);
    let abilities = mtgsim::oracle::characteristics::get_effective_abilities(&game, shaman);
    let index = abilities.iter().position(|a| a.ability_type == AbilityType::Activated).expect("an activated ability");
    let aim = ScriptedDecisionProvider::new();
    aim.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
        vec![ChoiceOption::Player(1)],
    );
    game.activate_ability(1, shaman, index, &ManaWindowStop::new(aim)).expect("activated");
    let recorded = game.recorded_events();
    let kind = recorded.events().find_map(|event| match event {
        GameEvent::AbilityActivated { mana_ability, .. } => Some(*mana_ability),
        _ => None,
    });
    assert_eq!(kind, Some(false), "its record: not a mana ability");
    place_triggers(&mut game, &ScriptedDecisionProvider::new());
    assert_eq!(game.stack.len(), 2, "the ability, and the trigger above it");
    resolve_all(&mut game);
    assert_eq!(life(&game, 1), 19, "2 dealt by the trigger, then 1 gained");
    assert_eq!(game.players[1].mana_pool.amount(ManaType::Colorless), 1, "added as it resolved");
}

/// A mana ability's activation is an activation (CR 605.3a): "an ability of a
/// land" triggers on it, and Harsh Mentor's "if it isn't a mana ability"
/// leaves it out.
#[test]
fn a_mana_ability_is_activated_too() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, life_on("Fixture Land Watcher", activates_an_ability_of(Whose::Each, of_type(&[CardType::Land]), None)), 0);
    put_on_battlefield(&mut game, mentor_fixture(), 0);
    let forest = put_on_battlefield(&mut game, forest(), 1);
    tap_for_mana(&mut game, 1, forest);
    assert_eq!(game.players[1].mana_pool.amount(ManaType::Green), 1);
    resolve_all(&mut game);
    assert_eq!(life(&game, 0), 21, "the land's ability was activated");
    assert_eq!(life(&game, 1), 20, "and it is a mana ability");
}

/// A source sacrificed to pay the cost is read as it was on the battlefield:
/// "an ability of an artifact" still triggers (CR 602.2b pays before it
/// announces).
#[test]
fn a_source_sacrificed_as_a_cost_is_read_as_it_was() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, life_on("Fixture Artificer", activates_an_ability_of(Whose::Each, of_type(&[CardType::Artifact]), None)), 0);
    let petal = with_a_sacrifice_for_mana(CardDataBuilder::new("Fixture Petal").card_type(CardType::Artifact), &[Cost::TapSelf]);
    let petal = put_on_battlefield(&mut game, petal, 0);
    tap_for_mana(&mut game, 0, petal);
    assert_eq!(game.objects[&petal].zone, Zone::Graveyard);
    resolve_all(&mut game);
    assert_eq!(life(&game, 0), 21);
}

/// `card` with "[{T},] Sacrifice this artifact: Add {C}."
fn with_a_sacrifice_for_mana(card: CardDataBuilder, costs: &[Cost]) -> Arc<CardData> {
    let text = if costs.contains(&Cost::TapSelf) { "{T}, Sacrifice this artifact: Add {C}." } else { "Sacrifice this artifact: Add {C}." };
    card.ability(AbilityDef {
        rules_text: text.into(),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Mana,
        costs: costs.iter().cloned().chain([Cost::SacrificeSelf]).collect(),
        effect: Effect::Atom(
            Primitive::ProduceMana(ManaOutput { mana: vec![(ManaType::Colorless, AmountExpr::Fixed(1))], special: vec![] }),
            EffectRecipient::Implicit,
        ),
    })
    .build()
}

/// And as the trigger resolves: "that artifact's power" is its power on the
/// battlefield, off the frame its record kept (CR 608.2h).
#[test]
fn a_source_sacrificed_as_a_cost_is_read_as_it_was_at_resolution() {
    let mut game = stocked(2);
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringPower(EventRole::Object)), EffectRecipient::Player(PlayerRef::You));
    let reader = CardDataBuilder::new("Fixture Artifact Reader")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(activates_an_ability_of(Whose::Yours, of_type(&[CardType::Artifact]), None), gain)))
        .build();
    put_on_battlefield(&mut game, reader, 0);
    let golem = CardDataBuilder::new("Fixture Golem").card_type(CardType::Artifact).card_type(CardType::Creature).power_toughness(3, 3);
    let golem = put_on_battlefield(&mut game, with_a_sacrifice_for_mana(golem, &[]), 0);
    tap_for_mana(&mut game, 0, golem);
    assert_eq!(game.objects[&golem].zone, Zone::Graveyard);
    resolve_all(&mut game);
    assert_eq!(life(&game, 0), 23);
}

/// A land player 1 owns and player 0 controls, which sacrifices itself for
/// {C}, as Crystal Vein's second ability does ("{T}, Sacrifice this land: Add
/// {C}{C}").
fn a_stolen_vein(game: &mut GameState) -> ObjectId {
    let vein = with_a_sacrifice_for_mana(CardDataBuilder::new("Fixture Vein").card_type(CardType::Land), &[Cost::TapSelf]);
    put_on_battlefield_under(game, vein, 1, 0)
}

/// The mana a source makes after its own cost moved it is still the mana of
/// the permanent it was (CR 608.2h): "whenever a land you control is tapped
/// for mana" triggers for a stolen land sacrificed for its mana, which in its
/// owner's graveyard is no land its thief controls.
#[test]
fn a_source_its_cost_moved_is_read_as_it_was_when_its_mana_is_added() {
    let mut game = stocked(2);
    let yours = ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Land)), Box::new(ObjectFilter::ByController(PlayerSet::You)));
    let watcher = life_on("Fixture Land Watcher", TriggerEvent::ManaAdded { source: TriggerSubject::Filter(yours), tapped_for_mana: Some(true), mana: None });
    put_on_battlefield(&mut game, watcher, 0);
    let vein = a_stolen_vein(&mut game);
    tap_for_mana(&mut game, 0, vein);
    assert_eq!(game.objects[&vein].zone, Zone::Graveyard);
    resolve_all(&mut game);
    assert_eq!(life(&game, 0), 21, "a land its controller tapped for mana");
}

/// And a replacement reads it the same way: under Deep Water ("if you tap a
/// land you control for mana, it produces {U} instead"), the stolen land's
/// {C} is {U}.
#[test]
fn a_replacement_reads_a_source_its_cost_moved_as_it_was() {
    let mut game = stocked(2);
    let card = deep_water();
    let water = put_on_battlefield(&mut game, card.clone(), 0);
    let ctx = ResolutionContext::untargeted(water, 0, mtgsim::events::cause::GeneratedBy::Spell);
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).expect("Deep Water's ability");
    let vein = a_stolen_vein(&mut game);
    tap_for_mana(&mut game, 0, vein);
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Blue), 1, "Deep Water's blue");
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Colorless), 0);
}

/// CR 605.1b: a triggered ability that triggers on a mana ability's
/// activation and could add mana is a mana ability, and resolves at once,
/// with no stack (CR 605.4a).
#[test]
fn a_trigger_on_a_mana_abilitys_activation_that_adds_mana_resolves_at_once() {
    let mut game = stocked(2);
    let add_one = Effect::Atom(
        Primitive::ProduceMana(ManaOutput { mana: vec![(ManaType::Colorless, AmountExpr::Fixed(1))], special: vec![] }),
        EffectRecipient::Player(PlayerRef::You),
    );
    let echo = CardDataBuilder::new("Fixture Mana Echo")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(activates_an_ability_of(Whose::Yours, TriggerSubject::Filter(ObjectFilter::All), Some(true)), add_one)))
        .build();
    put_on_battlefield(&mut game, echo, 0);
    let forest = put_on_battlefield(&mut game, forest(), 0);
    tap_for_mana(&mut game, 0, forest);
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 1);
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Colorless), 1, "added at once");
    assert!(game.stack.is_empty() && game.pending_triggers.is_empty());
}

/// CR 605.1b asks what the trigger triggered from, not what its text names:
/// "whenever you activate an ability, add {C}" is a mana ability when a
/// mana ability's activation triggers it, and uses the stack when another
/// activation does.
#[test]
fn a_trigger_is_a_mana_ability_by_the_activation_that_triggered_it() {
    let mut game = stocked(2);
    let add_one = Effect::Atom(
        Primitive::ProduceMana(ManaOutput { mana: vec![(ManaType::Colorless, AmountExpr::Fixed(1))], special: vec![] }),
        EffectRecipient::Player(PlayerRef::You),
    );
    let echo = CardDataBuilder::new("Fixture Any Echo")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(activates_an_ability_of(Whose::Yours, TriggerSubject::Filter(ObjectFilter::All), None), add_one)))
        .build();
    put_on_battlefield(&mut game, echo, 0);
    let forest = put_on_battlefield(&mut game, forest(), 0);
    tap_for_mana(&mut game, 0, forest);
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Colorless), 1, "a mana ability's activation: added at once");
    assert!(game.stack.is_empty() && game.pending_triggers.is_empty());

    let mine = put_on_battlefield(&mut game, merfolk_thaumaturgist(), 0);
    let theirs = put_on_battlefield(&mut game, merfolk_thaumaturgist(), 1);
    thaumaturgist_at(&mut game, 0, mine, theirs);
    place_triggers(&mut game, &ScriptedDecisionProvider::new());
    assert_eq!(game.stack.len(), 2, "another activation: the trigger on the stack above it");
}

/// The log leaves a mana ability's activation implicit, since its tap and
/// its mana say it, and the full event log keeps it.
#[test]
fn a_mana_abilitys_activation_is_left_implicit_in_the_log() {
    let mut game = stocked(2);
    let forest = put_on_battlefield(&mut game, forest(), 0);
    tap_for_mana(&mut game, 0, forest);
    let recorded = game.recorded_events();
    let lines: Vec<String> = log_lines(&game, recorded.records()).into_iter().map(|record| format_record(&game, record)).collect();
    assert!(lines.iter().any(|line| line.starts_with("ManaAdded")), "{lines:#?}");
    assert!(!lines.iter().any(|line| line.starts_with("AbilityActivated")), "{lines:#?}");
    assert!(mtgsim::ui::display::format_event_log(&game).iter().any(|line| line.starts_with("AbilityActivated")));
}
