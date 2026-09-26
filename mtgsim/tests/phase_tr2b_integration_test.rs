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
use mtgsim::oracle::characteristics::get_effective_controller;
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::{CounterSubject, GameEvent};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::test_support::{
    card_of_type, creature_with_ability, fill_library, put_in_hand, put_in_library, put_on_battlefield,
    put_on_battlefield_under, set_active_player, setup_game, setup_two_player_game, stock_libraries, test_ctx,
    test_dp, vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Choice, ChoiceScope, ChoiceSide, Condition, CostAnswer, CounterType, Duration, Effect,
    EffectRecipient, ObjectFilter, Pick, PlayerFact, PlayerGroup, PlayerRef, PlayerSet, Primitive, SelectionFilter,
    TargetCount,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, ObjectRef, PlayerId};
use mtgsim::types::mana::ManaCost;
use mtgsim::types::triggers::{TriggerCondition, TriggerDef, TriggerEvent, TriggerSubject};
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
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

/// "[Chooser] sacrifices [n] creature(s) of their choice", chosen as it
/// resolves from the chooser's own.
fn sacrifices(chooser: EffectRecipient, n: u64) -> Effect {
    Effect::Atom(
        Primitive::Sacrifice,
        EffectRecipient::ChosenBy(Box::new(Choice {
            chooser,
            among: ChoiceScope::ChoosersPermanents,
            picks: vec![Pick::exactly(n, ObjectFilter::ByType(CardType::Creature))],
            acts_on: ChoiceSide::Chosen,
        })),
    )
}

/// Cast `card`, which costs {0}, from `player`'s hand.
fn cast(game: &mut GameState, player: PlayerId, card: Arc<CardData>) -> ObjectId {
    let id = put_in_hand(game, card, player);
    let dp = ManaWindowStop::new(RecordingDecisionProvider::picking(0));
    game.cast_spell(player, id, &dp).expect("castable");
    id
}

/// Cast a {0} instant that does nothing, from `player`'s hand.
fn cast_a_spell(game: &mut GameState, player: PlayerId) {
    let spell = CardDataBuilder::new("Idle Thought")
        .mana_cost(ManaCost::build(&[], 0))
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            id: new_ability_id(),
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(Vec::new()),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
        })
        .build();
    cast(game, player, spell);
}

fn sacrificed(game: &GameState) -> Vec<(ObjectId, Option<mtgsim::events::event::BatchId>)> {
    game.recorded_events()
        .records()
        .iter()
        .filter_map(|r| match r.event {
            GameEvent::ZoneChange { object_id, cause: ZoneChangeCause::Sacrificed, .. } => Some((object_id, r.stamp.batch)),
            _ => None,
        })
        .collect()
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

// ---------------------------------------------------------------------------
// CR 701.21a, 608.2d — sacrifice, and a choice made as the effect applies
// ---------------------------------------------------------------------------

/// Standstill's board (ATOM-118.12-001): "When a player casts a spell,
/// sacrifice this enchantment. If you do, each player draws three cards",
/// with the enchantment exiled before its triggers resolve. It is not there
/// to sacrifice, so CR 118.12's answer is `Cant`, "if you do" fails, and
/// nobody draws, for either trigger. The fixture draws for each player where
/// Standstill draws for "that player's opponents": no `PlayerGroup` can say
/// the second yet, and the atom's claim is that nobody draws.
// COVERS: ATOM-118.12-001
#[test]
fn a_sacrifice_whose_permanent_has_gone_answers_cant_and_draws_nothing() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 10);
    let draw_three = Effect::Atom(
        Primitive::DrawCards(AmountExpr::Fixed(3)),
        EffectRecipient::EachOf(PlayerGroup::set(PlayerSet::Everyone)),
    );
    let standing = CardDataBuilder::new("Standing Stillness")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability(whenever(
            TriggerEvent::CastsSpell { caster: None, spell: None },
            Effect::Sequence(vec![Effect::Atom(Primitive::Sacrifice, EffectRecipient::ThisObject), if_you(CostAnswer::Does, draw_three)]),
        )))
        .build();
    let stillness = put_on_battlefield(&mut game, standing, 0);
    cast_a_spell(&mut game, 1);
    cast_a_spell(&mut game, 1);
    place(&mut game, &RecordingDecisionProvider::picking(0));
    assert_eq!(game.stack.len(), 4, "two spells, and the two triggers above them");

    game.execute_action(
        GameAction::ZoneChange { object: stillness, from: Zone::Battlefield, to: Zone::Exile, cause: ZoneChangeCause::Exiled },
        &test_ctx(),
    )
    .unwrap();
    let hands = (game.players[0].hand.len(), game.players[1].hand.len());
    for _ in 0..2 {
        game.resolve_top_of_stack(&test_dp()).unwrap();
    }
    assert_eq!((game.players[0].hand.len(), game.players[1].hand.len()), hands, "nobody draws");
}

/// "If you can't" reads an action that could not be started: an edict on
/// yourself with no creature, and "sacrifice this" once someone else controls
/// it (CR 701.21a — a player can't sacrifice what they don't control).
#[test]
fn if_you_cant_reads_a_sacrifice_that_could_not_start() {
    let lose_five = Effect::Atom(Primitive::LoseLife(AmountExpr::Fixed(5)), EffectRecipient::Controller);

    let mut game = setup_two_player_game();
    let source = put_on_battlefield(&mut game, card_of_type("Hungry Idol", CardType::Artifact), 0);
    let effect = Effect::Sequence(vec![sacrifices(EffectRecipient::Controller, 1), if_you(CostAnswer::Cant, lose_five.clone())]);
    resolve_as(&mut game, source, effect.clone());
    assert_eq!(game.players[0].life_total, 15, "no creature to sacrifice: 5 life");
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    resolve_as(&mut game, source, effect);
    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.players[0].life_total, 15, "a creature sacrificed: no loss");

    // The trigger's controller is 0; the permanent is now 1's.
    let stolen = put_on_battlefield(&mut game, card_of_type("Wandering Idol", CardType::Artifact), 1);
    let ctx = ResolutionContext {
        ability_source: Some(ObjectRef { id: stolen, zone_change_epoch: game.get_object(stolen).unwrap().zone_change_epoch }),
        ..ResolutionContext::untargeted(stolen, 0)
    };
    let sacrifice_this = Effect::Sequence(vec![
        Effect::Atom(Primitive::Sacrifice, EffectRecipient::ThisObject),
        if_you(CostAnswer::Cant, lose_five),
    ]);
    game.resolve_effect(&sacrifice_this, &ctx, &test_dp()).unwrap();
    assert_eq!(game.get_object(stolen).unwrap().zone, Zone::Battlefield, "not theirs to sacrifice");
    assert_eq!(game.players[0].life_total, 10);
}

/// "Sacrifice that creature": the bound object, found by identity.
#[test]
fn a_trigger_sacrifices_the_object_its_event_named() {
    let mut game = setup_two_player_game();
    let sacrifice_it = Effect::Atom(Primitive::Sacrifice, EffectRecipient::TriggeringObject);
    put_on_battlefield(
        &mut game,
        watcher("Ravenous Gate", enters(ObjectFilter::ByType(CardType::Creature)), sacrifice_it),
        0,
    );
    game.pending_triggers.clear();
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    place(&mut game, &test_dp());
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Graveyard);
}

/// "Each opponent sacrifices a creature" at four seats: each chooses in turn,
/// the active player first (CR 101.4, Soul Shatter's ruling), and all three
/// are sacrificed at once, in one batch.
#[test]
fn each_opponent_chooses_in_turn_and_sacrifices_at_once() {
    let mut game = setup_game(4);
    game.record_events();
    set_active_player(&mut game, 2);
    let source = put_on_battlefield(&mut game, card_of_type("Grim Decree", CardType::Artifact), 0);
    let mut first = Vec::new();
    for seat in 1..4 {
        first.push(put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), seat));
        put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), seat);
    }
    let dp = RecordingDecisionProvider::picking(0);
    let each_opponent = EffectRecipient::EachOf(PlayerGroup::set(PlayerSet::Opponents));
    let ctx = ResolutionContext::untargeted(source, 0);
    game.resolve_effect(&sacrifices(each_opponent, 1), &ctx, &dp).unwrap();

    assert_eq!(dp.prompts(), 3, "each opponent chose one of two");
    let gone = sacrificed(&game);
    let order: Vec<ObjectId> = gone.iter().map(|(id, _)| *id).collect();
    assert_eq!(order, vec![first[1], first[2], first[0]], "seat 2, the active player, first");
    assert!(gone.windows(2).all(|pair| pair[0].1 == pair[1].1), "one batch");
}

/// The choice is every object verb's, not sacrifice's: "target player exiles
/// a creature of their choice" through the same recipient.
#[test]
fn an_exile_edict_chooses_through_the_same_recipient() {
    let mut game = setup_two_player_game();
    let source = put_on_battlefield(&mut game, card_of_type("Banishing Decree", CardType::Artifact), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let exiles = Effect::Atom(
        Primitive::Exile,
        EffectRecipient::ChosenBy(Box::new(Choice {
            chooser: EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
            among: ChoiceScope::ChoosersPermanents,
            picks: vec![Pick::exactly(1, ObjectFilter::ByType(CardType::Creature))],
            acts_on: ChoiceSide::Chosen,
        })),
    );
    let ctx = ResolutionContext {
        targets: ChosenTargets::one(vec![ResolvedTarget::Player(1)]),
        ..ResolutionContext::untargeted(source, 0)
    };
    game.resolve_effect(&exiles, &ctx, &test_dp()).unwrap();
    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Exile);
}

// ---------------------------------------------------------------------------
// CR 113.7a, 608.2h, 109.5 — what a trigger reads once what it names has gone
// ---------------------------------------------------------------------------

fn each_opponent_loses_its_power() -> Effect {
    Effect::Atom(
        Primitive::LoseLife(AmountExpr::TriggeringPower),
        EffectRecipient::EachOf(PlayerGroup::set(PlayerSet::Opponents)),
    )
}

/// A 2/2: "When this creature enters, if you have 10 or more life, each
/// opponent loses life equal to its power."
fn reckoner() -> Arc<CardData> {
    let def = TriggerDef {
        condition: TriggerCondition::Event(enters(TriggerSubject::ThisObject).into()),
        intervening_if: Some(Condition::Player {
            whose: PlayerSet::You,
            fact: PlayerFact::LifeAtLeast(AmountExpr::Fixed(10)),
        }),
        limit: None,
        effect: each_opponent_loses_its_power(),
    };
    creature_with_ability("Borrowed Reckoner", 2, 2, triggered_ability(def))
}

/// Item 169's board: the reckoner enters under player 0's control though
/// player 1 owns it, is grown to 4/4 and sacrificed in response. The recheck's
/// "you" is still player 0 (CR 109.5, 603.3a), not the owner whose graveyard it
/// is in, and "its power" is the 4 it last had.
#[test]
fn an_enters_trigger_reads_its_controller_and_power_once_its_source_is_sacrificed() {
    let mut game = setup_two_player_game();
    game.players[1].life_total = 5;
    let reckoner = put_on_battlefield_under(&mut game, reckoner(), 1, 0);
    place(&mut game, &test_dp());
    let grow = GameAction::AddCounters {
        subject: CounterSubject::Object(reckoner),
        counter: CounterType::PlusOnePlusOne,
        n: 2,
        by: 0,
    };
    game.execute_action(grow, &test_ctx()).unwrap();
    game.change_zone(reckoner, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx()).unwrap();

    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.players[1].life_total, 1, "player 0's 20 life met the \"if\", and it was last a 4/4");
}

/// The recheck of a stolen source: player 1 takes the reckoner in response,
/// and "you" is still player 0, who controlled it as it triggered.
#[test]
fn the_recheck_of_a_stolen_source_reads_the_player_it_triggered_for() {
    let mut game = setup_two_player_game();
    game.players[1].life_total = 5;
    let reckoner = put_on_battlefield(&mut game, reckoner(), 0);
    place(&mut game, &test_dp());
    let thief = put_on_battlefield(&mut game, card_of_type("Thieving Idol", CardType::Artifact), 1);
    let steal = Effect::Atom(
        Primitive::GainControl(Duration::Indefinite),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    let ctx = ResolutionContext {
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(reckoner)]),
        ..ResolutionContext::untargeted(thief, 1)
    };
    game.resolve_effect(&steal, &ctx, &test_dp()).unwrap();
    assert_eq!(get_effective_controller(&game, reckoner), Some(1));

    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.players[1].life_total, 3, "player 0's 20 life met the \"if\", and player 1 is its opponent");
}

/// Item 169's other zones: "its power" of a spell countered in response, and
/// of a drawn card discarded in response, each off the frame it left with.
#[test]
fn a_countered_spell_and_a_discarded_card_leave_their_power_behind() {
    let mut game = setup_two_player_game();
    let casts = TriggerEvent::CastsSpell { caster: None, spell: None };
    put_on_battlefield(&mut game, watcher("Spiteful Critic", casts, each_opponent_loses_its_power()), 0);
    put_on_battlefield(&mut game, watcher("Spiteful Reader", draws_a_card(Whose::Yours), each_opponent_loses_its_power()), 0);

    let ogre = CardDataBuilder::new("Hasty Ogre")
        .mana_cost(ManaCost::build(&[], 0))
        .card_type(CardType::Creature)
        .power_toughness(3, 3)
        .build();
    let spell = cast(&mut game, 0, ogre);
    place(&mut game, &test_dp());
    game.change_zone(spell, Zone::Graveyard, ZoneChangeCause::Countered, &test_ctx()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.players[1].life_total, 17, "the countered spell's 3");

    let card = put_in_library(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.draw_card(0, &test_ctx()).unwrap();
    place(&mut game, &test_dp());
    game.change_zone(card, Zone::Graveyard, ZoneChangeCause::Discarded, &test_ctx()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.players[1].life_total, 15, "the discarded card's 2");
}

/// "When this creature enters, sacrifice it, then each opponent loses life
/// equal to its power": the effect moved it itself, and the resolving ability
/// keeps the frame it left with.
#[test]
fn an_ability_that_sacrifices_its_own_source_reads_the_power_it_left_with() {
    let mut game = setup_two_player_game();
    let effect = Effect::Sequence(vec![
        Effect::Atom(Primitive::Sacrifice, EffectRecipient::ThisObject),
        each_opponent_loses_its_power(),
    ]);
    let martyr = creature_with_ability("Brief Martyr", 3, 3, triggered_ability(whenever(enters(TriggerSubject::ThisObject), effect)));
    let martyr = put_on_battlefield(&mut game, martyr, 0);
    place(&mut game, &test_dp());
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.get_object(martyr).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.players[1].life_total, 17);
}
