//! TR-6 — state triggers (CR 603.8), the draw (CR 104.4) and the rule-owned
//! arm (`triggers-architecture.md` §4.5, §4.8, §4.9 and §12).

use mtgsim::cards::authoring::{dies, enters, triggered_ability, whenever};
use mtgsim::objects::card_data::CardDataBuilder;
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_re_cards::platinum_angel;
use mtgsim::cards::phase_tr6_cards::{
    destroy_your_others_then_a_token, discard_your_hand_then_draw_that_many, draws_on_an_empty_hand, emperor_crocodile,
    immortal_coils_state_trigger,
};
use mtgsim::engine::actions::ZoneChangeAct;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::{EventCause, GameRule, GeneratedBy};
use mtgsim::events::event::{EventId, GameEvent};
use mtgsim::state::game_state::{GameResult, GameState};
use mtgsim::state::trace::{RecordKind, TraceRecord};
use mtgsim::test_support::{
    creature_with_ability, fill_library, install_trace, put_in_graveyard, put_in_hand, put_on_battlefield, setup_game,
    setup_two_player_game, test_ctx, test_dp,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Condition, Effect, EffectRecipient, ObjectFilter, PlayerFact, PlayerSet, Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::triggers::{Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerSubject, TriggerTier};
use mtgsim::types::zones::{Zone, ZoneChangePattern};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::why::{why_from_trace, OpenQuestion, Why, WhyAbout};

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

/// A game whose every dispatch, and every state check, is answered twice:
/// by the gate, and by every object that may trigger (`engine::triggers::audit`).
fn audited_game() -> GameState {
    let mut game = setup_two_player_game();
    game.enable_audit();
    game
}

/// CR 117.5's loop, then the stack resolved to empty, the loop after each.
fn resolve_all(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).expect("SBAs and triggers");
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(dp).expect("it resolves");
        game.perform_sba_and_triggers(dp).expect("SBAs and triggers");
    }
}

/// How many times `source`'s abilities triggered, by the records announcing it.
fn triggered(game: &GameState, source: ObjectId) -> usize {
    game.recorded_events()
        .events()
        .filter(|event| matches!(event, GameEvent::AbilityTriggered { origin, .. } if origin.source() == source))
        .count()
}

/// Whether one of `source`'s triggered abilities is waiting or on the stack.
fn has_not_left_the_stack(game: &GameState, source: ObjectId) -> bool {
    game.pending_triggers.iter().any(|pending| pending.origin.source() == source)
        || game.stack.iter().any(|id| game.stack_entries.get(id).and_then(|entry| entry.ability_identity).is_some_and(|i| i.source.id == source))
}

fn hand_size(game: &GameState, player: PlayerId) -> usize {
    game.players[player].hand.len()
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
// RULING: Platinum Angel #4 - effects that say the game is a draw are not
//   affected by Platinum Angel.
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

/// CR 603.3b's second tier asked what caused a trigger: "the event that
/// caused it was a creature dying" is the arm's full reading of the cause's
/// record, so an artifact put into a graveyard, the same kind of record,
/// is not one.
#[test]
fn a_tier_two_trigger_reads_its_cause_as_the_arm_it_names() {
    let mut game = setup_two_player_game();
    let to_a_graveyard = TriggerEvent::ZoneChange {
        subject: TriggerSubject::Any,
        change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: None },
        by: None,
        owner: None,
        multiplicity: Multiplicity::PerOccurrence,
    };
    let mut gains = triggered_ability("", whenever(to_a_graveyard, gain_one()));
    gains.id = new_ability_id();
    put_on_battlefield(&mut game, creature_with_ability("Fixture: a permanent to a graveyard", 1, 1, gains), 0);
    let a_creature_dying = Box::new(dies(TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature))).into());
    let mut watches = triggered_ability("", whenever(TriggerEvent::AbilityTriggers { caused_by: Some(a_creature_dying), source: None }, gain_one()));
    watches.id = new_ability_id();
    let watcher = put_on_battlefield(&mut game, creature_with_ability("Fixture: caused by a creature dying", 1, 1, watches), 0);
    let trinket = put_on_battlefield(&mut game, CardDataBuilder::new("Fixture: trinket").card_type(CardType::Artifact).build(), 1);
    let bear = put_on_battlefield(&mut game, grizzly_bears(), 1);

    game.change_zone(trinket, Zone::Graveyard, ZoneChangeAct::Destroyed, &test_ctx()).unwrap();
    assert_eq!(triggered(&game, watcher), 0, "an artifact put into a graveyard is no creature dying");
    game.change_zone(bear, Zone::Graveyard, ZoneChangeAct::Destroyed, &test_ctx()).unwrap();
    assert_eq!(triggered(&game, watcher), 1);
}

// ---------------------------------------------------------------------------
// State triggers (CR 603.8)
// ---------------------------------------------------------------------------

/// CR 603.8's example: "whenever you have no cards in hand, draw a card"
/// triggers once and not again while it waits or is on the stack, though the
/// hand is still empty. Once it has left the stack, an empty hand triggers it
/// again.
// COVERS: ATOM-603.8-001
#[test]
fn a_state_trigger_triggers_once_until_it_has_left_the_stack() {
    let mut game = audited_game();
    fill_library(&mut game, 0, 5);
    game.players[0].hand.clear();
    let source = put_on_battlefield(&mut game, draws_on_an_empty_hand(), 0);

    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(triggered(&game, source), 1, "the empty hand triggers it");
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(triggered(&game, source), 1, "on the stack, it does not trigger again (CR 603.8)");

    game.resolve_top_of_stack(&test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(hand_size(&game, 0), 1);
    assert_eq!(triggered(&game, source), 1, "a hand with a card in it matches nothing");

    let card = game.players[0].hand[0];
    game.change_zone(card, Zone::Graveyard, ZoneChangeAct::Discarded, &test_ctx()).unwrap();
    assert_eq!(triggered(&game, source), 2, "it left the stack, and the hand is empty again");
}

/// CR 603.8, 608.2n: a state trigger resolving has not left the stack, so a
/// record its own resolution makes while its state still holds does not
/// trigger it again. "Whenever you have no cards in hand, you gain 1 life"
/// gains the life with the hand still empty.
#[test]
fn a_resolving_state_trigger_has_not_left_the_stack() {
    let mut game = audited_game();
    game.players[0].hand.clear();
    let gains = TriggerDef {
        condition: TriggerCondition::State(Condition::Player { players: PlayerSet::You, fact: PlayerFact::HandEmpty }),
        intervening_if: None,
        functions_in: None,
        limit: None,
        effect: gain_one(),
    };
    let mut ability = triggered_ability("Whenever you have no cards in hand, you gain 1 life.", gains);
    ability.id = new_ability_id();
    let source = put_on_battlefield(&mut game, creature_with_ability("Fixture: gains on an empty hand", 1, 1, ability), 0);

    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(triggered(&game, source), 1);
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.players[0].life_total, 21);
    assert_eq!(triggered(&game, source), 1, "its own gain came while it was resolving");
}

/// CR 603.8's second sentence of its example: "Discard your hand, then draw
/// that many cards" leaves the hand empty for a moment, between its two
/// instructions, and the state trigger triggers then. It goes on the stack
/// after the spell has resolved, and draws one more.
// COVERS: ATOM-603.8-002
#[test]
fn a_state_true_for_a_moment_of_a_resolution_triggers() {
    let mut game = audited_game();
    fill_library(&mut game, 0, 10);
    game.players[0].hand.clear();
    for _ in 0..3 {
        put_in_hand(&mut game, grizzly_bears(), 0);
    }
    let source = put_on_battlefield(&mut game, draws_on_an_empty_hand(), 0);
    let spell = put_in_hand(&mut game, discard_your_hand_then_draw_that_many(), 0);
    game.cast_spell(0, spell, &test_dp()).expect("it costs nothing");

    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.players[0].graveyard.len(), 4, "three discarded, and the spell");
    assert_eq!(hand_size(&game, 0), 3, "three discarded, then that many drawn");
    assert_eq!(triggered(&game, source), 1, "the hand was empty between the two");
    assert!(game.stack.is_empty() && game.pending_triggers.len() == 1, "waiting, not yet on the stack");

    resolve_all(&mut game, &test_dp());
    assert_eq!(hand_size(&game, 0), 4);
    assert_eq!(triggered(&game, source), 1);
}

// RULING: Emperor Crocodile #1 - it triggers if you control no other creature
//   even for a brief moment during a resolution.
#[test]
fn emperor_crocodile_triggers_for_a_moment_in_the_middle_of_a_resolution() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    let crocodile = put_on_battlefield(&mut game, emperor_crocodile(), 0);
    resolve_all(&mut game, &test_dp());
    assert_eq!(triggered(&game, crocodile), 0, "the Bears are another creature");

    let spell = put_in_hand(&mut game, destroy_your_others_then_a_token(), 0);
    game.cast_spell(0, spell, &test_dp()).expect("it costs nothing");
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(triggered(&game, crocodile), 1, "between the destruction and the Soldier");
    assert_eq!(game.battlefield.len(), 2, "the Crocodile and the Soldier");

    resolve_all(&mut game, &test_dp());
    assert_eq!(zone_of(&game, crocodile), Some(Zone::Graveyard), "sacrificed beside the Soldier");
}

// RULING: Emperor Crocodile #2 - it checks only as it triggers, so gaining a
//   creature before it resolves does not save it.
#[test]
fn emperor_crocodile_is_sacrificed_whatever_arrives_before_it_resolves() {
    let mut game = audited_game();
    let crocodile = put_on_battlefield(&mut game, emperor_crocodile(), 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(game.stack.len(), 1, "alone, it triggers");

    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone_of(&game, crocodile), Some(Zone::Graveyard));
    assert_eq!(zone_of(&game, bears), Some(Zone::Battlefield));
    assert_eq!(triggered(&game, crocodile), 1);
}

/// Cast from hand, paid with exactly its cost: alone on its controller's side
/// of the battlefield, it sacrifices itself.
#[test]
fn emperor_crocodile_cast_from_hand_alone_sacrifices_itself() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.players[0].mana_pool.add(ManaType::Green, 4);
    let card = put_in_hand(&mut game, emperor_crocodile(), 0);
    let seat = ManaWindowStop::new(test_dp());

    game.cast_spell(0, card, &seat).expect("castable with {3}{G}");
    resolve_all(&mut game, &seat);

    assert_eq!(game.players[0].graveyard.len(), 1, "it entered, triggered and was sacrificed");
    // ORDER-FREE: a predicate over every entry, whose answer no order changes.
    assert!(game.battlefield.values().all(|entry| entry.controller == 1), "the opponent's Bears are not another creature of yours");
}

/// Immortal Coil's state trigger checks as it triggers: a card put into the
/// graveyard after that does not save its controller.
// RULING: Immortal Coil #4 - the third ability checks whether your graveyard
//   is empty only at the time it triggers.
#[test]
fn immortal_coil_checks_its_graveyard_only_as_it_triggers() {
    let mut game = audited_game();
    let coil = put_on_battlefield(&mut game, immortal_coils_state_trigger(), 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(triggered(&game, coil), 1);

    put_in_graveyard(&mut game, grizzly_bears(), 0);
    resolve_all(&mut game, &test_dp());
    assert_eq!(game.result, Some(GameResult::Winner(1)));
}

// RULING: Immortal Coil #5 - if the third ability is countered but your
//   graveyard is still empty, it immediately triggers again.
#[test]
fn immortal_coil_countered_triggers_again_at_once() {
    let mut game = audited_game();
    let coil = put_on_battlefield(&mut game, immortal_coils_state_trigger(), 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    let ability = *game.stack.last().unwrap();

    counter(&mut game, ability);

    assert!(game.stack.is_empty() || !game.stack.contains(&ability), "countered (CR 701.6a)");
    assert_eq!(triggered(&game, coil), 2, "its leaving the stack lets it trigger, and it does");
    assert!(has_not_left_the_stack(&game, coil));
    assert_eq!(game.result, None);
}

// RULING: Immortal Coil #6 - if the third ability resolves but you don't lose
//   (Platinum Angel), it immediately triggers again.
#[test]
fn immortal_coil_resolved_without_the_loss_triggers_again() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, platinum_angel(), 0);
    let coil = put_on_battlefield(&mut game, immortal_coils_state_trigger(), 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();

    game.resolve_top_of_stack(&test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();

    assert_eq!(game.result, None, "Platinum Angel: you can't lose the game");
    assert_eq!(triggered(&game, coil), 2);
    assert_eq!(game.stack.len(), 1, "on the stack again, and its controller has priority");
}

/// Counter the ability `ability` on the stack, as Stifle does.
fn counter(game: &mut GameState, ability: ObjectId) {
    let source = put_in_hand(game, grizzly_bears(), 1);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 1,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ObjectOrPlayerId::Object(ability)]),
        replaced_event: None,
        trigger: None,
    };
    let stifle = Effect::Atom(Primitive::CounterAbility, EffectRecipient::Target(SelectionFilter::Spell, TargetCount::Exactly(1)));
    game.resolve_effect(&stifle, &ctx, &test_dp()).expect("countering");
}

fn zone_of(game: &GameState, id: ObjectId) -> Option<Zone> {
    game.objects.get(&id).map(|object| object.zone)
}

/// Every line of `answer`'s section headed `heading`, as its text and rule.
fn why_lines(answer: &Why, heading: &str) -> Vec<(String, Option<&'static str>)> {
    let section = answer.sections.iter().find(|s| s.heading == heading);
    let section = section.unwrap_or_else(|| panic!("no section {heading:?} in {answer:#?}"));
    section.lines.iter().map(|line| (line.text.clone(), line.rule)).collect()
}

/// The newest performed event whose words contain `words`.
fn newest_event(records: &[TraceRecord], words: &str) -> EventId {
    let found = records.iter().rev().find(|r| r.kind == RecordKind::Event && r.str("text").is_some_and(|t| t.contains(words)));
    EventId(found.and_then(|r| r.u64("index")).unwrap_or_else(|| panic!("no event says {words:?}")) as usize)
}

/// The why panel, from the trace. A state trigger names no event, so its
/// record says when the board was asked. On the Crocodile's first-ruling
/// board, the Bears' destruction shows the Crocodile that triggered after it,
/// the one place the panel can say why a Crocodile beside a Soldier is
/// sacrificed; once the ability is on the stack, it says so. At the ordering,
/// the other Crocodile, beside its own Bears, says what it answered when the
/// board was last asked, once, though it was asked after every event.
#[test]
fn the_why_panel_says_when_the_board_was_asked() {
    let mut game = audited_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let crocodile = put_on_battlefield(&mut game, emperor_crocodile(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    let theirs = put_on_battlefield(&mut game, emperor_crocodile(), 1);
    resolve_all(&mut game, &test_dp());
    let trace = install_trace(&mut game, "the board, asked");
    let spell = put_in_hand(&mut game, destroy_your_others_then_a_token(), 0);
    game.cast_spell(0, spell, &test_dp()).expect("it costs nothing");
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(triggered(&game, crocodile), 1);
    assert_eq!(zone_of(&game, bears), Some(Zone::Graveyard));

    let records = trace.records();
    let destroyed = newest_event(&records, "Grizzly Bears");
    let after_it = "State triggers that triggered after it";
    let said = why_lines(&why_from_trace(&game, WhyAbout::Event(destroyed), None, &records), after_it);
    assert!(said.contains(&(format!("{}: triggered", named(&game, crocodile)), Some("603.8"))), "{said:#?}");
    let waits = "It waits to be put on the stack the next time a player would receive priority";
    assert!(said.contains(&(waits.to_string(), Some("603.3"))), "{said:#?}");

    let context = ChoiceContext::new(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First });
    let options = [ChoiceOption::Object(crocodile)];
    let question = OpenQuestion { player: 0, context: &context, options: &options };
    let said = why_lines(&why_from_trace(&game, WhyAbout::Object(theirs), Some(&question), &records), "At this question");
    let asked = said.iter().filter(|(text, _)| text.starts_with("Asked of the board after: ")).count();
    assert_eq!(asked, 1, "its newest answer only: {said:#?}");
    assert!(said.contains(&(format!("{}: did not trigger", named(&game, theirs)), Some("603.8"))), "{said:#?}");
    assert!(said.contains(&("Its state was not true of the board".to_string(), Some("603.8"))), "{said:#?}");

    game.perform_sba_and_triggers(&test_dp()).unwrap();
    let records = trace.records();
    let said = why_lines(&why_from_trace(&game, WhyAbout::Event(destroyed), None, &records), after_it);
    assert!(said.iter().any(|(text, rule)| text.starts_with("Put on the stack as ") && *rule == Some("603.3")), "{said:#?}");
}

fn named(game: &GameState, id: ObjectId) -> String {
    mtgsim::ui::display::named(game, id)
}

// ---------------------------------------------------------------------------
// "Each [permanent]" reaches every primitive
// ---------------------------------------------------------------------------

/// Resolve `primitive` over "each creature you control" for player 0, beside
/// two creatures of player 0's and one of player 1's. Returns `(yours, theirs)`.
fn each_creature_you_control(game: &mut GameState, primitive: Primitive) -> (Vec<ObjectId>, ObjectId) {
    let yours = vec![put_on_battlefield(game, grizzly_bears(), 0), put_on_battlefield(game, grizzly_bears(), 0)];
    let theirs = put_on_battlefield(game, grizzly_bears(), 1);
    let yours_filter = ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(mtgsim::types::effects::PlayerRef::You)),
    );
    let source = put_in_hand(game, grizzly_bears(), 0);
    let ctx = ResolutionContext::untargeted(source, 0, GeneratedBy::Spell);
    let each = Effect::Atom(primitive, EffectRecipient::FilteredPermanents(yours_filter));
    game.resolve_effect(&each, &ctx, &test_dp()).expect("resolving");
    (yours, theirs)
}

/// "Exile each creature you control" — Merciless Eviction's shape.
#[test]
fn exile_reaches_each_permanent_a_filter_names() {
    let mut game = setup_two_player_game();
    let (yours, theirs) = each_creature_you_control(&mut game, Primitive::Exile);
    assert!(yours.iter().all(|&id| zone_of(&game, id) == Some(Zone::Exile)));
    assert_eq!(zone_of(&game, theirs), Some(Zone::Battlefield));
}

/// "Tap all creatures ..." — Sleep's shape.
#[test]
fn tap_reaches_each_permanent_a_filter_names() {
    let mut game = setup_two_player_game();
    let (yours, theirs) = each_creature_you_control(&mut game, Primitive::Tap);
    assert!(yours.iter().all(|id| game.battlefield[id].tapped));
    assert!(!game.battlefield[&theirs].tapped);
}

/// "Return all creatures ... to their owners' hands" — Evacuation's shape.
#[test]
fn return_to_hand_reaches_each_permanent_a_filter_names() {
    let mut game = setup_two_player_game();
    let (yours, theirs) = each_creature_you_control(&mut game, Primitive::ReturnToHand);
    assert!(yours.iter().all(|&id| zone_of(&game, id) == Some(Zone::Hand)));
    assert_eq!(zone_of(&game, theirs), Some(Zone::Battlefield));
}

/// "Shuffle each creature you control into its owner's library."
#[test]
fn shuffle_into_library_reaches_each_permanent_a_filter_names() {
    let mut game = setup_two_player_game();
    let (yours, theirs) = each_creature_you_control(&mut game, Primitive::ShuffleIntoLibrary);
    assert!(yours.iter().all(|&id| zone_of(&game, id) == Some(Zone::Library)));
    assert_eq!(zone_of(&game, theirs), Some(Zone::Battlefield));
}
