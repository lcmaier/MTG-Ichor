//! SU-8, what happened, from the trace (`setup-architecture.md` §7c): the
//! sink's lines read back, the "can't" a CR 616.1 iteration met, the why of an
//! event and of the triggered abilities asked about it, and every question
//! kind's line.

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_rc_cards::{master_biomancer, thunder_thrash_elder};
use mtgsim::cards::phase_tr1_cards::{felidar_sovereign, soul_warden};
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeCause};
use mtgsim::engine::priority::PriorityResult;
use mtgsim::events::event::{DamageTarget, EventSeq};
use mtgsim::scenario::{BuiltScenario, Scenario, SetupDriver};
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::state::trace::{FieldValue, RecordKind, TraceRecord};
use mtgsim::test_support::{
    fill_library, install_trace, put_in_graveyard, put_in_hand, put_on_battlefield, setup_game, setup_two_player_game,
    test_ctx, vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::ManaCost;
use mtgsim::types::triggers::TriggerTier;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::random::RandomDecisionProvider;
use mtgsim::ui::why::{why, why_from_trace, OpenQuestion, Why, WhyAbout, WhyLine};

/// Every line of `answer`'s section headed `heading`, as its text and rule.
fn lines(answer: &Why, heading: &str) -> Vec<(String, Option<&'static str>)> {
    let section = answer.sections.iter().find(|s| s.heading == heading);
    let section = section.unwrap_or_else(|| panic!("no section {heading:?} in {answer:#?}"));
    section.lines.iter().map(|WhyLine { text, rule, .. }| (text.clone(), *rule)).collect()
}

fn has(answer: &Why, heading: &str, text: &str, rule: Option<&'static str>) -> bool {
    lines(answer, heading).contains(&(text.to_string(), rule))
}

/// The performed event whose words contain `words`, by its place in the stream.
fn event(records: &[TraceRecord], words: &str) -> EventSeq {
    let found = records.iter().find(|r| r.kind == RecordKind::Event && r.str("text").is_some_and(|t| t.contains(words)));
    EventSeq(found.and_then(|r| r.u64("index")).unwrap_or_else(|| panic!("no event says {words:?}")) as usize)
}

fn named(game: &GameState, id: ObjectId) -> String {
    mtgsim::ui::display::named(game, id)
}

/// Item 210: a debug build's trace is a release build's. Two Biomancers'
/// entry counters commute, so CR 616.1's prompt is suppressed, and the debug
/// build's check of that (`check_order_invariance`) gathers again over a
/// probe of the entry. The probe's CR 614.12 look-ahead wrote a `layer_walk`
/// the release build never writes: a release build writes four, and the
/// debug build wrote a fifth.
#[test]
fn a_debug_builds_order_check_writes_nothing_to_the_trace() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, master_biomancer(), 0);
    put_on_battlefield(&mut game, master_biomancer(), 0);
    let bears = put_in_graveyard(&mut game, grizzly_bears(), 0);
    let trace = install_trace(&mut game, "item 210");
    // No answer queued: a CR 616.1 prompt here fails the test.
    let dp = ScriptedDecisionProvider::new();
    game.change_zone(bears, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("the Bears return");

    let entering = trace.of_kind("layer_walk").into_iter().filter(|line| line.contains("\"membership\":\"entering\"")).count();
    assert_eq!(entering, 4, "the look-ahead walks a release build writes");
}

/// CR 614.17: a "can't" is checked ahead of every replacement effect and wins
/// (CR 101.2). An indestructible creature with lethal damage is not destroyed
/// (CR 702.12b), and the CR 616.1 iteration that met the "can't" names it:
/// which member, whose restriction, and which.
#[test]
fn the_iteration_a_cant_stopped_names_it() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Indestructible]), 0);
    game.battlefield.get_mut(&creature).unwrap().damage_marked = 3;
    let trace = install_trace(&mut game, "a can't, named");
    game.perform_sba_and_triggers(&ScriptedDecisionProvider::new()).unwrap();
    assert!(game.battlefield.contains_key(&creature), "indestructible: lethal damage destroys nothing");

    let records = trace.records();
    let stopped: Vec<&FieldValue> = records.iter().filter(|r| r.kind == RecordKind::Pipeline).flat_map(|r| r.items("prohibited")).collect();
    let [named] = stopped.as_slice() else { panic!("one member stopped, found {stopped:?}") };
    let field = |key: &str| named.get(key).cloned();
    assert_eq!(field("i"), Some(FieldValue::Number(0)), "the batch's one proposal, the destruction");
    assert_eq!(field("source"), Some(FieldValue::Number(creature.raw().into())));
    assert_eq!(field("by"), Some(FieldValue::Text("keyword".into())));
    assert_eq!(field("words"), Some(FieldValue::Text("indestructible".into())));
}

/// `setup-architecture.md` §5.3's stack, from its review board: the
/// Thaumaturgist's switch, then Giant Growth, then Lightning Bolt resolve,
/// and the why of the Bolt's damage reads its batch back from the trace.
#[test]
fn the_bolts_damage_reads_back_its_batch() {
    let text = include_str!("../scenarios/bolt-into-giant-growth.scenario");
    let BuiltScenario { mut game, setup } =
        Scenario::parse(text).and_then(|s| s.build(&CardRegistry::default_registry())).unwrap_or_else(|r| panic!("{r}"));
    let seats = RecordingDecisionProvider::picking(0).stopping_at_every_priority_point();
    let driver = SetupDriver::new(setup, &seats);
    for _ in 0..3 {
        assert_eq!(game.state.run_priority_round(&driver), Ok(PriorityResult::ActionTaken), "a setup line");
    }
    let trace = install_trace(&mut game.state, "the Bolt's damage");
    while !game.state.stack.is_empty() {
        assert_eq!(game.state.run_priority_round(&driver), Ok(PriorityResult::StackResolved));
    }

    let records = trace.records();
    let answer = why_from_trace(&game.state, WhyAbout::Event(event(&records, "DamageDealt")), None, &records);
    assert!(answer.title.starts_with("DamageDealt: Lightning Bolt") && answer.title.contains("Grizzly Bears"), "{}", answer.title);
    let happened = lines(&answer, "What happened");
    assert_eq!(happened[0].0, "Read from a replay of the whole game.");
    assert!(happened[1].0.starts_with("It was performed in batch "), "{happened:#?}");
    let proposed = answer.sections[0].lines.iter().find(|line| line.text.starts_with("Proposed: DealDamage"));
    let proposed = proposed.unwrap_or_else(|| panic!("{happened:#?}"));
    assert!(proposed.names.iter().any(|(_, label)| label.starts_with("Grizzly Bears")), "the Bears are linked");
    assert!(has(&answer, "Triggered abilities asked about it", "No triggered ability was asked about it.", None));
}

/// The why of the damage before a "can't": an indestructible creature takes
/// lethal damage, the state-based destruction is proposed in a batch of its
/// own and stopped (CR 614.17, 702.12b), and having no event of its own, it
/// is told after the damage.
#[test]
fn a_destruction_a_cant_stopped_is_told_after_the_damage() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Indestructible]), 0);
    let source = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    let trace = install_trace(&mut game, "a can't, told");
    let damage = GameAction::DealDamage { source, target: DamageTarget::Object(creature), amount: 3, is_combat: false, unpreventable: false };
    game.execute_action(damage, &test_ctx()).unwrap();
    game.perform_sba_and_triggers(&ScriptedDecisionProvider::new()).unwrap();

    let records = trace.records();
    let answer = why_from_trace(&game, WhyAbout::Event(event(&records, "DamageDealt")), None, &records);
    let happened = lines(&answer, "What happened");
    assert!(happened.iter().any(|(text, _)| text.starts_with("Then batch ") && text.ends_with("performed nothing:")), "{happened:#?}");
    let stopped = format!("A “can't” stops proposal 0: {} has indestructible", named(&game, creature));
    assert!(has(&answer, "What happened", &stopped, Some("614.17")), "{happened:#?}");
}

/// CR 603.4: Felidar Sovereign's upkeep trigger is asked about its
/// controller's upkeep and refused by its intervening "if" below 40 life; at
/// 40 it triggers, and goes on the stack at the next priority.
#[test]
fn a_trigger_refused_by_its_intervening_if_says_so() {
    for (life, verdict, after) in [
        (38, "did not trigger", ("Its intervening “if” was false when the event happened", Some("603.4"))),
        (40, "triggered", ("Put on the stack as", Some("603.3"))),
    ] {
        let mut game = setup_two_player_game();
        let felidar = put_on_battlefield(&mut game, felidar_sovereign(), 0);
        game.players[0].life_total = life;
        let trace = install_trace(&mut game, "Felidar Sovereign");
        upkeep_of(&mut game, 0);
        game.perform_sba_and_triggers(&ScriptedDecisionProvider::new()).unwrap();

        let records = trace.records();
        // Asked about every event since it entered; the upkeep's is the one its
        // condition matched.
        let asked = records.iter().find(|r| {
            r.kind == RecordKind::Trigger && r.u64("source") == Some(felidar.raw()) && r.str("refused_by") != Some("condition")
        });
        let upkeep = EventSeq(asked.and_then(|r| r.u64("record")).expect("the upkeep trigger was asked") as usize);
        let answer = why_from_trace(&game, WhyAbout::Event(upkeep), None, &records);
        let triggers = lines(&answer, "Triggered abilities asked about it");
        let head = (format!("{}: {verdict}", named(&game, felidar)), Some("603.2"));
        let at = triggers.iter().position(|line| *line == head).unwrap_or_else(|| panic!("{triggers:#?}"));
        assert_eq!(triggers[at + 1].0, "“At the beginning of your upkeep, if you have 40 or more life, you win the game.”");
        assert!(triggers[at + 2].0.starts_with(after.0) && triggers[at + 2].1 == after.1, "{triggers:#?}");
    }
}

/// `player`'s next upkeep, the libraries filled so the draws on the way do
/// not end the game.
fn upkeep_of(game: &mut GameState, player: PlayerId) {
    for p in 0..game.num_players() {
        fill_library(game, p, 10);
    }
    for _ in 0..20 {
        game.advance_turn(&test_ctx()).unwrap();
        if game.active_player == player && game.phase.step == Some(StepType::Upkeep) {
            return;
        }
    }
    panic!("player {player}'s upkeep never began");
}

/// Every question kind, built here in `ChoiceKind`'s order. A kind added to
/// the enum does not compile in `place` until it is listed, and a kind left
/// out of the list fails the order's assertion.
fn every_kind(object: ObjectId) -> Vec<ChoiceKind> {
    let kinds = vec![
        ChoiceKind::PriorityAction,
        ChoiceKind::DeclareAttackers,
        ChoiceKind::DeclareBlockers,
        ChoiceKind::AssignCombatDamage { attacker_id: object },
        ChoiceKind::AssignTrampleDamage { attacker_id: object, defending_target: DamageTarget::Player(1) },
        ChoiceKind::ChooseXValue { spell_id: object, x_count: 1 },
        ChoiceKind::ChooseAlternativeCost { spell_id: object },
        ChoiceKind::ChooseAdditionalCosts { spell_id: object },
        ChoiceKind::SelectRecipients { recipient: mtgsim::types::effects::EffectRecipient::Controller, spell_id: object },
        ChoiceKind::GenericManaAllocation { spell_or_ability_id: object, mana_cost: ManaCost::build(&[], 2) },
        ChoiceKind::OrderCostReductions { spell_id: object },
        ChoiceKind::ManaAbilityWindow { spell_or_ability_id: object, remaining_cost: ManaCost::build(&[], 1) },
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id: object, count: 1 },
        ChoiceKind::ChooseReplacementEffect { affected_object: Some(object) },
        ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First },
        ChoiceKind::ApplyOptionalReplacement { affected_object: Some(object), source: object },
        ChoiceKind::ApplyOptionalEffect { source: object },
        ChoiceKind::AllocateNextDamage { source: object, remaining: 2 },
        ChoiceKind::ChooseDamageSource { source: object },
        ChoiceKind::ChooseEnteringController { object },
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: object, source: object, to: Zone::Graveyard },
        ChoiceKind::ChooseCopySource { source: object },
        ChoiceKind::CommanderToCommandZoneSba { commander: object },
        ChoiceKind::Discard { source: None },
        ChoiceKind::Scry { source: None, n: 1 },
        ChoiceKind::ScryOrder { source: None, bottom: true },
        ChoiceKind::LegendRule { legend_name: "Grizzly Bears".to_string() },
    ];
    let place = |kind: &ChoiceKind| match kind {
        ChoiceKind::PriorityAction => 0,
        ChoiceKind::DeclareAttackers => 1,
        ChoiceKind::DeclareBlockers => 2,
        ChoiceKind::AssignCombatDamage { .. } => 3,
        ChoiceKind::AssignTrampleDamage { .. } => 4,
        ChoiceKind::ChooseXValue { .. } => 5,
        ChoiceKind::ChooseAlternativeCost { .. } => 6,
        ChoiceKind::ChooseAdditionalCosts { .. } => 7,
        ChoiceKind::SelectRecipients { .. } => 8,
        ChoiceKind::GenericManaAllocation { .. } => 9,
        ChoiceKind::OrderCostReductions { .. } => 10,
        ChoiceKind::ManaAbilityWindow { .. } => 11,
        ChoiceKind::ChooseSacrificeForCost { .. } => 12,
        ChoiceKind::ChooseReplacementEffect { .. } => 13,
        ChoiceKind::OrderTriggers { .. } => 14,
        ChoiceKind::ApplyOptionalReplacement { .. } => 15,
        ChoiceKind::ApplyOptionalEffect { .. } => 16,
        ChoiceKind::AllocateNextDamage { .. } => 17,
        ChoiceKind::ChooseDamageSource { .. } => 18,
        ChoiceKind::ChooseEnteringController { .. } => 19,
        ChoiceKind::ChooseAuxiliaryZoneChange { .. } => 20,
        ChoiceKind::ChooseCopySource { .. } => 21,
        ChoiceKind::CommanderToCommandZoneSba { .. } => 22,
        ChoiceKind::Discard { .. } => 23,
        ChoiceKind::Scry { .. } => 24,
        ChoiceKind::ScryOrder { .. } => 25,
        ChoiceKind::LegendRule { .. } => 26,
    };
    let places: Vec<usize> = kinds.iter().map(place).collect();
    assert_eq!(places, (0..27).collect::<Vec<_>>(), "every kind, once, in order");
    kinds
}

/// Every question kind answers a why about an object it does not offer, an
/// object it does, a player and an event, now and from an empty trace,
/// without panicking; and about a card it does not offer, each says what it
/// ranges over or why the card is never offered.
#[test]
fn every_question_kind_answers_a_why() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let card = put_in_hand(&mut game, grizzly_bears(), 1);
    let options = [ChoiceOption::Object(bears)];
    for kind in every_kind(bears) {
        let context = ChoiceContext::new(kind.clone());
        let question = OpenQuestion { player: 0, context: &context, options: &options };
        for about in [WhyAbout::Object(card), WhyAbout::Object(bears), WhyAbout::Player(1), WhyAbout::Event(EventSeq(0))] {
            for answer in [why(&game, about, Some(&question)), why_from_trace(&game, about, Some(&question), &[])] {
                assert!(!answer.sections.is_empty(), "{kind:?}, {about:?}");
            }
        }
        let said = lines(&why(&game, WhyAbout::Object(card), Some(&question)), "At this question");
        let ranges = said.iter().any(|(text, _)| text.starts_with("Not among the options: the question ranges over "));
        let never = said.iter().any(|(text, _)| text == "Never offered to Player 0:");
        assert!(ranges || never, "{kind:?} says nothing of a card it does not offer: {said:#?}");
    }
}

/// CR 800.4a: a player who has left the game is not an opponent to choose,
/// and the why says so, as it says the chooser is not their own opponent
/// (CR 102.2). Four seats, the fourth gone.
#[test]
fn a_departed_player_is_named_at_choose_entering_controller() {
    let mut game = setup_game(4);
    let entering = put_in_hand(&mut game, grizzly_bears(), 0);
    game.player_lost[3] = true;
    let context = ChoiceContext::new(ChoiceKind::ChooseEnteringController { object: entering });
    let options = [ChoiceOption::Player(1), ChoiceOption::Player(2)];
    let question = OpenQuestion { player: 0, context: &context, options: &options };

    let departed = why(&game, WhyAbout::Player(3), Some(&question));
    assert!(has(&departed, "At this question", "To control it: Player 3 has left the game.", Some("800.4a")), "{departed:#?}");
    let chooser = why(&game, WhyAbout::Player(0), Some(&question));
    let yourself = "To control it: Player 0 is choosing, and an opponent is another player.";
    assert!(has(&chooser, "At this question", yourself, Some("102.2")), "{chooser:#?}");
    let offered = why(&game, WhyAbout::Player(1), Some(&question));
    assert!(has(&offered, "At this question", "Offered to Player 0:", None), "{offered:#?}");
}

/// Devour's choice (CR 614.13a) reads its own effect's filter: a land is not
/// a creature, so it is not what Thunder-Thrash Elder's devour chooses, and
/// the Elder itself is entering.
#[test]
fn devours_filter_says_why_a_permanent_is_not_offered() {
    let mut game = setup_two_player_game();
    let elder = put_in_hand(&mut game, thunder_thrash_elder(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let land = put_on_battlefield(&mut game, CardRegistry::default_registry().create("Mountain").unwrap(), 0);
    let context = ChoiceContext::new(ChoiceKind::ChooseAuxiliaryZoneChange { entering: elder, source: elder, to: Zone::Graveyard });
    let options = [ChoiceOption::Object(bears)];
    let question = OpenQuestion { player: 0, context: &context, options: &options };

    let answer = why(&game, WhyAbout::Object(land), Some(&question));
    let not_chosen = format!("To be moved: it is not what {}'s effect chooses.", named(&game, elder));
    assert!(has(&answer, "At this question", &not_chosen, None), "{answer:#?}");
    let answer = why(&game, WhyAbout::Object(elder), Some(&question));
    let entering = "To be moved: it is entering the battlefield in this event.";
    assert!(has(&answer, "At this question", entering, Some("614.13a")), "{answer:#?}");
}

/// CR 603.3b's ordering reads the trace: a second Soul Warden enters, the
/// first's ability triggers and waits to be ordered, and the second's, asked
/// about its own arrival, was refused by its condition, "another creature"
/// (CR 603.2). The seat, with no trace, says only what the question ranges
/// over.
#[test]
fn an_ordering_says_what_another_ability_answered() {
    let mut game = setup_two_player_game();
    let first = put_on_battlefield(&mut game, soul_warden(), 0);
    let second = put_in_graveyard(&mut game, soul_warden(), 0);
    let trace = install_trace(&mut game, "an ordering");
    let dp = ScriptedDecisionProvider::new();
    game.change_zone(second, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp)).unwrap();
    assert_eq!(game.pending_triggers.len(), 1, "the first Warden's, waiting");

    let context = ChoiceContext::new(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First });
    let options = [ChoiceOption::Object(first)];
    let question = OpenQuestion { player: 0, context: &context, options: &options };
    let records = trace.records();
    let said = lines(&why_from_trace(&game, WhyAbout::Object(second), Some(&question), &records), "At this question");
    assert!(said.iter().any(|(text, _)| text.starts_with("Asked about: ") && text.contains("Soul Warden")), "{said:#?}");
    let refused = ("Its trigger condition does not match this event".to_string(), Some("603.2"));
    assert!(said.contains(&refused), "{said:#?}");
    let now = lines(&why(&game, WhyAbout::Object(second), Some(&question)), "At this question");
    assert!(!now.contains(&refused), "the seat has no trace to read: {now:#?}");
}

/// Breadth: a seeded random game over decks of every registered card, traced
/// from its setup, and every event it performed answers a why read from its
/// trace, each with both sections: every record shape a whole game writes.
#[test]
fn every_event_of_a_random_game_answers_from_its_trace() {
    let registry = CardRegistry::default_registry();
    let deck: Vec<_> = registry.card_names().iter().cycle().take(60).filter_map(|name| registry.create(name).ok()).collect();
    let mut game = Game::new(GameConfig::test(), vec![deck; 2]).expect("game creation");
    let trace = install_trace(&mut game.state, "a random game");
    game.reseed(7);
    let dp = RandomDecisionProvider::seeded(7);
    game.setup(&dp).expect("setup");
    for _ in 0..6 {
        if game.is_over() {
            break;
        }
        game.run_turn(&dp).expect("turn");
    }

    let records = trace.records();
    let events: Vec<EventSeq> =
        records.iter().filter(|r| r.kind == RecordKind::Event).filter_map(|r| r.u64("index")).map(|i| EventSeq(i as usize)).collect();
    assert!(events.len() > 50, "a game's worth of events, {}", events.len());
    for event in events {
        let answer = why_from_trace(&game.state, WhyAbout::Event(event), None, &records);
        assert!(!answer.title.is_empty() && answer.sections.len() == 2, "{event:?}: {answer:#?}");
    }
}
