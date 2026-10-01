//! SU-1: the scenario loader (`plans/setup-architecture.md`). Each word of
//! §5.1 built onto a board, §4.1's refusals, and a load that is construction
//! rather than play.

use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::layers::compute::compute_characteristics;
use mtgsim::oracle::characteristics::{get_effective_power, get_effective_toughness, has_summoning_sickness};
use mtgsim::scenario::{ScenarioError, ScenarioErrorKind, Scenario};
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::state::game::{Game, RandomStreams};
use mtgsim::state::game_state::{PhaseType, StepType};
use mtgsim::types::effects::CounterType;
use mtgsim::types::history::TurnFact;
use mtgsim::types::ids::ObjectId;
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

fn load(text: &str) -> Game {
    Scenario::parse(text).and_then(|s| s.build(&CardRegistry::default_registry())).map(|built| built.game).unwrap_or_else(|r| panic!("{r}"))
}

fn refused(text: &str) -> ScenarioError {
    Scenario::parse(text).and_then(|s| s.build(&CardRegistry::default_registry())).map(|_| ()).expect_err(text)
}

/// The permanents named `name`, oldest first.
fn named(game: &Game, name: &str) -> Vec<ObjectId> {
    let state = &game.state;
    state.battlefield_ids_ordered().into_iter().filter(|id| state.objects[id].card_data.name == name).collect()
}

fn one(game: &Game, name: &str) -> ObjectId {
    named(game, name)[0]
}

#[test]
fn a_load_is_construction_and_emits_nothing() {
    let mut game = load(include_str!("../scenarios/holy-strength.scenario"));
    assert_eq!(game.state.events.next_seq().0, 0, "the event sequence is still at its start");
    game.state.record_events();
    assert_eq!(game.state.recorded_events().events().count(), 0);
    assert!(game.state.pending_triggers.is_empty());
}

#[test]
fn the_template_and_every_sample_load() {
    for (name, text) in [
        ("template", include_str!("../scenarios/template.scenario")),
        ("holy-strength", include_str!("../scenarios/holy-strength.scenario")),
        ("humility-opalescence", include_str!("../scenarios/humility-opalescence.scenario")),
        ("planeswalker", include_str!("../scenarios/planeswalker.scenario")),
        ("four-seats-commander", include_str!("../scenarios/four-seats-commander.scenario")),
    ] {
        Scenario::parse(text).and_then(|s| s.build(&CardRegistry::default_registry())).unwrap_or_else(|r| panic!("{name}: {r}"));
    }
    let listed = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/scenarios")).unwrap().count();
    assert_eq!(listed, 5, "a new file under scenarios/ joins this test");
}

/// The same file and seed, the same answers: one game.
#[test]
fn a_scenario_game_played_twice_is_one_game() {
    let play = || {
        let mut game = load(include_str!("../scenarios/holy-strength.scenario"));
        game.state.record_events();
        let agents = ManaWindowStop::new(RandomDecisionProvider::seeded(RandomStreams::from_seed(7).agents));
        let result = game.resume(&agents).unwrap();
        (result, game.event_log_snapshot())
    };
    let (first, second) = (play(), play());
    assert!(first.1.len() > 50, "a whole game: {} lines", first.1.len());
    assert_eq!(first, second);
}

#[test]
fn players_starting_life_turn_active_and_step() {
    let game = load("players 3\nstarting life 30\nturn 5\nactive 2\nstep upkeep\nplayer 1: life 12");
    let state = &game.state;
    assert_eq!(state.players.iter().map(|p| p.life_total).collect::<Vec<_>>(), [30, 12, 30]);
    assert_eq!(state.starting_life, 30);
    assert_eq!((state.turn_number, state.active_player, state.priority_player, state.turn_rotation), (5, 2, 2, 2));
    // The natural rotation up to player 2's turn 5: 1, 2, 0, 1, 2.
    assert_eq!(state.last_turn_began, [3, 4, 5]);
    assert_eq!((state.phase.phase_type, state.phase.step), (PhaseType::Beginning, Some(StepType::Upkeep)));
    assert!(!state.skip_first_draw, "a turn-5 upkeep has no first draw to skip");
}

/// CR 103.8a: the skip is set only for turn 1 before its draw step, and a
/// draw step on turn 1 of a two-player game does not exist.
#[test]
fn the_first_draw_is_derived_and_its_absent_step_refused() {
    assert!(load("step upkeep").state.skip_first_draw);
    assert!(!load("players 3\nstep upkeep").state.skip_first_draw);
    assert!(!load("step precombat main").state.skip_first_draw);
    assert!(refused("step draw").message.contains("CR 103.8a"));
    assert!(refused("step cleanup").message.contains("CR 502.4, 514.3"));
}

/// The seed draws a shuffled library from the game's stream: the same seed
/// is the same order, a listed library is the file's order, top first.
#[test]
fn seed_and_the_libraries() {
    let shuffled = |seed: u64| {
        let game = load(&format!("seed {seed}\nlibrary 1 shuffled: Forest | x6\nlibrary 1 shuffled: Mountain | x6"));
        game.state.players[1].library.iter().map(|id| game.state.objects[id].card_data.name.clone()).collect::<Vec<_>>()
    };
    assert_eq!(shuffled(3), shuffled(3));
    assert_ne!(shuffled(3), shuffled(4), "another seed, another order");
    let game = load("library 0: Mountain\nlibrary 0: Forest | x2\nlibrary 0: Plains");
    let top_first: Vec<&str> = game.state.players[0].library.iter().rev().map(|id| game.state.objects[id].card_data.name.as_str()).collect();
    assert_eq!(top_first, ["Mountain", "Forest", "Forest", "Plains"]);
    assert!(refused("library 0: Forest\nlibrary 0 shuffled: Forest").message.contains("shuffle orders the whole library"));
}

#[test]
fn hand_graveyard_exile_and_command() {
    let game = load(
        "hand 1: Lightning Bolt | x2\ngraveyard 0: Grizzly Bears\ngraveyard 0: Savannah Lions\n\
         exile: Lightning Bolt | owner 1\ncommand: Isamaru, Hound of Konda | owner 0, commander",
    );
    let state = &game.state;
    let name = |id: &ObjectId| state.objects[id].card_data.name.as_str();
    assert_eq!(state.players[1].hand.len(), 2);
    assert_eq!(state.players[0].graveyard.last().map(name), Some("Savannah Lions"), "each card on top of the last");
    assert_eq!(state.objects[&state.exile[0]].owner, 1);
    assert!(state.objects[&state.command[0]].is_commander);
    assert!(refused("exile: Lightning Bolt").message.contains("needs `owner p`"));
}

#[test]
fn player_counters_lands_played_and_commander_damage() {
    let game = load(
        "player 1: poison 3, energy 2, lands played 1\n\
         battlefield: Isamaru, Hound of Konda | controller 0, commander\n\
         player 1: commander damage 12 from Isamaru, Hound of Konda",
    );
    let player = &game.state.players[1];
    assert_eq!((player.counter_count(CounterType::Poison), player.counter_count(CounterType::Energy)), (3, 2));
    assert_eq!(player.lands_played_this_turn, 1);
    assert_eq!(player.commander_damage_taken.get(&one(&game, "Isamaru, Hound of Konda")), Some(&12));
}

/// CR 800.4a: a player who has left owns nothing, is not active, and leaves
/// two players in the game; the rotation passes over them.
#[test]
fn a_player_who_has_left() {
    let game = load("players 3\nturn 4\nactive 2\nplayer 1: left the game");
    assert!(!game.state.in_game(1));
    assert_eq!(game.state.last_turn_began, [3, 0, 4], "turns 1 to 4 went 0, 2, 0, 2");
    for (text, says) in [
        ("players 3\nplayer 1: left the game\nhand 1: Forest", "owns or controls"),
        ("players 3\nplayer 0: left the game", "the active player"),
        ("player 1: left the game", "fewer than two"),
    ] {
        assert!(refused(text).message.contains(says), "{text}");
    }
}

/// The rows land on their turns, so "since your last turn" reads what a
/// played game would: each player's last turn ended as the next began.
#[test]
fn history_rows_and_since_your_last_turn() {
    let game = load(
        "turn 4\nactive 1\n\
         player 0 this turn: life gained 2\nplayer 0 last turn: life gained 3, cards drawn 1\n\
         player 0 this game: life gained 10\nplayer 1 this turn: creature spells cast 1, spells cast 1",
    );
    let (state, now) = (&game.state, 4);
    let history = |p: usize| &state.players[p].history;
    assert_eq!(history(0).this_turn(now).count(TurnFact::LifeGained), 2);
    assert_eq!(history(0).last_turn(now).count(TurnFact::LifeGained), 3);
    assert_eq!(history(0).this_game().count(TurnFact::LifeGained), 10);
    assert_eq!(history(0).this_game().count(TurnFact::CardsDrawn), 1, "this game defaults to the two rows' sum");
    assert_eq!(history(1).this_turn(now).count(TurnFact::SpellsCastOfType(mtgsim::types::card_types::CardType::Creature)), 1);
    // Player 0 took turn 3, which ended as turn 4 began; player 1's last
    // turn, turn 2, ended as turn 3 began.
    assert_eq!(history(0).since_your_last_turn(0, history(0)).count(TurnFact::LifeGained), 2);
    assert_eq!(history(1).since_your_last_turn(0, history(0)).count(TurnFact::LifeGained), 5);
    assert!(refused("player 0 last turn: cards drawn 1").message.contains("turn 1 has no last turn"));
    assert!(refused("turn 2\nplayer 0 this game: cards drawn 1").message.contains("no turn before last"));
}

#[test]
fn this_turns_abilities() {
    let game = load(
        "battlefield: Elvish Warmaster | controller 0\n\
         this turn: Elvish Warmaster | triggered, ability 2 resolved 2, took its once-each-turn action",
    );
    let state = &game.state;
    assert_eq!(state.triggered_this_turn.len(), 1);
    assert_eq!(state.resolutions_this_turn.values().copied().collect::<Vec<_>>(), [2]);
    assert_eq!(state.action_taken_this_turn.iter().map(|(_, player)| *player).collect::<Vec<_>>(), [0]);
    assert!(refused("battlefield: Elvish Warmaster | controller 0\nthis turn: Elvish Warmaster | resolved 1").message.contains("ability N"));
}

/// The door: rows registered, the order the file's, the controller and
/// owner each the other's default.
#[test]
fn battlefield_controller_owner_and_order() {
    let game = load(
        "battlefield: Grizzly Bears | owner 1\nbattlefield: Glorious Anthem | controller 0\n\
         battlefield: Grizzly Bears | owner 0, controller 1",
    );
    let bears = named(&game, "Grizzly Bears");
    let state = &game.state;
    assert_eq!(state.battlefield_ids_ordered()[1], one(&game, "Glorious Anthem"), "timestamps in file order");
    assert_eq!((state.objects[&bears[0]].owner, state.battlefield[&bears[0]].controller), (1, 1));
    assert_eq!((state.objects[&bears[1]].owner, state.battlefield[&bears[1]].controller), (0, 1));
    assert_eq!(get_effective_power(state, bears[0]), Some(2), "the Anthem is player 0's");
    assert!(refused("battlefield: Grizzly Bears").message.contains("controller p"));
}

/// CR 302.6: a permanent arrived before this turn unless the file says when.
#[test]
fn tapped_and_arrived() {
    let game = load(
        "players 3\nturn 5\nactive 1\nbattlefield: Grizzly Bears | controller 1, tapped\n\
         battlefield: Savannah Lions | controller 1, arrived this turn\nbattlefield: Wall of Stone | controller 0, arrived turn 4\n\
         battlefield: Hill Giant | controller 2, arrived turn 2",
    );
    let state = &game.state;
    assert!(state.battlefield[&one(&game, "Grizzly Bears")].tapped);
    assert!(!has_summoning_sickness(state, one(&game, "Grizzly Bears")));
    assert!(has_summoning_sickness(state, one(&game, "Savannah Lions")));
    assert!(has_summoning_sickness(state, one(&game, "Wall of Stone")), "player 0's turn 4 is their most recent");
    assert!(!has_summoning_sickness(state, one(&game, "Hill Giant")), "player 2's turn 3 began after it arrived");
    assert!(refused("battlefield: Grizzly Bears | controller 0, arrived turn 2").message.contains("not a turn before"));
}

/// A stated kind replaces the intrinsic count (CR 306.5b), and a `counters:`
/// line stamps a kind after the lines above it, so a flying counter put on
/// after Humility outlasts it (CR 613.7c, 613.5).
#[test]
fn counters_on_the_line_and_after_it() {
    let game = load(
        "battlefield: Loyalty Probe | controller 0\nbattlefield: Loyalty Probe | controller 1, loyalty 1\n\
         battlefield: Grizzly Bears [a] | controller 0, flying 1, +1/+1 2\nbattlefield: Grizzly Bears [b] | controller 0\n\
         battlefield: Humility | controller 1\ncounters: Grizzly Bears [b] | flying 1",
    );
    let probes = named(&game, "Loyalty Probe");
    let state = &game.state;
    assert_eq!(probes.iter().map(|id| state.battlefield[id].counter_count(CounterType::Loyalty)).collect::<Vec<_>>(), [3, 1]);
    let flying = |tag: usize| compute_characteristics(state, named(&game, "Grizzly Bears")[tag]).unwrap().keyword_flags.contains(&KeywordFlag::Flying);
    assert!(!flying(0), "Humility arrived after the counter");
    assert!(flying(1), "the counter arrived after Humility");
    assert_eq!(get_effective_power(state, named(&game, "Grizzly Bears")[0]), Some(3), "1/1, then +1/+1 counters");
}

#[test]
fn damage_and_attached_to() {
    let game = load(
        "battlefield: Grizzly Bears | controller 0, damage 1\nbattlefield: Holy Strength | controller 0, attached to Grizzly Bears",
    );
    let (bears, aura) = (one(&game, "Grizzly Bears"), one(&game, "Holy Strength"));
    let state = &game.state;
    assert_eq!(state.battlefield[&bears].damage_marked, 1);
    assert_eq!(state.battlefield[&aura].attached_to, Some(bears));
    assert_eq!((get_effective_power(state, bears), get_effective_toughness(state, bears)), (Some(3), Some(4)));
    assert!(state.object_timestamp(aura) > state.object_timestamp(bears));
    let host_after = refused("battlefield: Holy Strength | controller 0, attached to Grizzly Bears\nbattlefield: Grizzly Bears | controller 0");
    assert_eq!((host_after.kind, host_after.line), (ScenarioErrorKind::Reference, Some(1)));
}

#[test]
fn attacking_blocked_and_blocking() {
    let game = load(include_str!("../scenarios/holy-strength.scenario"));
    let (bears, wall) = (named(&game, "Grizzly Bears")[0], one(&game, "Wall of Stone"));
    let attack = game.state.battlefield[&bears].attacking.clone().unwrap();
    assert!(matches!(attack.target, AttackTarget::Player(1)));
    assert_eq!((attack.is_blocked, attack.blocked_by), (true, vec![wall]));
    assert_eq!(game.state.battlefield[&wall].blocking.as_ref().map(|b| b.blocking.clone()), Some(vec![bears]));
    assert!(game.state.attacks_declared && game.state.blockers_declared);

    let at_a_walker = load(include_str!("../scenarios/planeswalker.scenario"));
    let lions = at_a_walker.state.battlefield[&one(&at_a_walker, "Savannah Lions")].attacking.clone().unwrap();
    assert!(matches!(lions.target, AttackTarget::Planeswalker(id) if id == one(&at_a_walker, "Loyalty Probe")));

    let blocked = load("turn 2\nstep combat damage\nbattlefield: Grizzly Bears | controller 0, attacking player 1, blocked");
    let attack = blocked.state.battlefield[&one(&blocked, "Grizzly Bears")].attacking.clone().unwrap();
    assert!(attack.is_blocked && attack.blocked_by.is_empty(), "CR 509.1h: blocked, its blocker gone");
}

#[test]
fn dealt_first_strike_damage() {
    let game = load("step first strike damage\nbattlefield: Isamaru, Hound of Konda | controller 0, attacking player 1, dealt first-strike damage");
    assert!(game.state.dealt_first_strike_damage.contains(&one(&game, "Isamaru, Hound of Konda")));
    assert!(refused("step declare blockers\nbattlefield: Grizzly Bears | controller 0, attacking player 1, dealt first-strike damage")
        .message.contains("CR 510.4"));
}

/// §4.1's three classes, each naming the line and what to change; a state
/// the rules correct is built, and CR 117.5 performs it.
#[test]
fn the_classes_of_refusal() {
    let not_a_card = refused("hand 0: Grizly Bears");
    assert_eq!((not_a_card.kind, not_a_card.line), (ScenarioErrorKind::NotACard, Some(1)));
    assert!(not_a_card.message.contains("Grizly Bears is not registered"));

    let two = refused("battlefield: Grizzly Bears | controller 0\nbattlefield: Grizzly Bears | controller 0\nbattlefield: Holy Strength | controller 0, attached to Grizzly Bears");
    assert_eq!((two.kind, two.line), (ScenarioErrorKind::Reference, Some(3)));
    assert!(two.message.contains("lines 1 and 2"), "{two}");

    for (text, says) in [
        ("battlefield: Grizzly Bears | controller 0, attacking player 1", "attackers exist from the declare attackers step"),
        ("step declare attackers\nbattlefield: Grizzly Bears | controller 1, attacking player 0", "CR 508.1a"),
        ("step declare attackers\nbattlefield: Grizzly Bears | controller 0, attacking player 0", "CR 508.1b"),
        ("step declare blockers\nbattlefield: Grizzly Bears | controller 0\nbattlefield: Wall of Stone | controller 1, blocking Grizzly Bears", "not attacking"),
        ("step declare blockers", "CR 508.8"),
        ("battlefield: Lightning Bolt | controller 0", "CR 304.4"),
        ("players 3
step declare attackers
player 2: left the game
battlefield: Grizzly Bears | controller 0, attacking player 2", "who has left the game"),
    ] {
        let refusal = refused(text);
        assert_eq!(refusal.kind, ScenarioErrorKind::Unreachable, "{text}");
        assert!(refusal.message.contains(says), "{text}: {refusal}");
    }

    let mut corrected = load("player 1: poison 10\nbattlefield: Grizzly Bears | controller 0, damage 2");
    corrected.state.record_events();
    let agents = ManaWindowStop::new(RandomDecisionProvider::seeded(1));
    assert_eq!(corrected.resume(&agents), Ok(mtgsim::state::game_state::GameResult::Winner(0)), "CR 704.5c, before any priority");
    assert!(corrected.state.battlefield.is_empty(), "CR 704.5g in the same check");
}

/// A card in development is a name a scenario can use and a pool cannot.
#[test]
fn a_card_in_development_loads_and_stays_out_of_the_pools() {
    let mut registry = CardRegistry::default_registry();
    registry.register_in_development("Dev Bear", || {
        mtgsim::objects::card_data::CardDataBuilder::new("Dev Bear")
            .card_type(mtgsim::types::card_types::CardType::Creature)
            .power_toughness(2, 2)
            .build()
    });
    let scenario = Scenario::parse("battlefield: Dev Bear | controller 0").unwrap();
    let game = scenario.build(&registry).unwrap().game;
    assert_eq!(named(&game, "Dev Bear").len(), 1);
    assert!(!registry.card_names().contains(&"Dev Bear"));
    assert!(scenario.build(&CardRegistry::default_registry()).is_err());
}

/// The writer spells what the loader built: the design's board comes back in
/// the file's words, timestamp order kept, and reloads to the same text.
#[test]
fn a_loaded_board_is_written_back_in_its_own_words() {
    let game = load(include_str!("../scenarios/holy-strength.scenario"));
    let written = Scenario::write(&game.state);
    assert!(written.unwritten.is_empty(), "{:?}", written.unwritten);
    let text = written.to_string();
    for line in [
        "turn 3\nstep declare blockers\nplayer 0: life 18\nplayer 0 this turn: attackers declared 1\nplayer 1: poison 2\n",
        "graveyard 0: Savannah Lions\nbattlefield: Glorious Anthem | controller 0\n",
        "battlefield: Grizzly Bears [a] | controller 0, tapped, attacking player 1\n",
        "battlefield: Holy Strength | controller 0, attached to Grizzly Bears [a]\n",
        "battlefield: Loyalty Probe | controller 1, loyalty 1\nbattlefield: Humility | controller 1, arrived this turn\n",
        "command: Isamaru, Hound of Konda | owner 0, commander\n",
    ] {
        assert!(text.contains(line), "missing {line:?} in\n{text}");
    }
    assert_eq!(Scenario::write(&load(&text).state).to_string(), text);
}

/// A life total below zero is at rest under Platinum Angel, so it reads
/// and writes back like any other.
#[test]
fn a_life_total_below_zero_reads_and_writes_back() {
    let game = load("player 0: life -3
battlefield: Platinum Angel | controller 0");
    assert_eq!(game.state.players[0].life_total, -3);
    assert!(Scenario::write(&game.state).to_string().contains("player 0: life -3
"));
}

/// What §2 plays rather than writes is reported, a line each, at the top.
#[test]
fn the_writer_reports_what_it_cannot_write() {
    let mut game = load("battlefield: Grizzly Bears | controller 0");
    game.state.players[0].mana_pool.add(mtgsim::types::mana::ManaType::Red, 1);
    game.state.turn_queue.push(1);
    let written = Scenario::write(&game.state);
    assert_eq!(written.unwritten.len(), 2, "{:?}", written.unwritten);
    assert!(written.to_string().starts_with("# not written: an extra turn (CR 500.7)\n# not written: player 0's mana pool"));
}
