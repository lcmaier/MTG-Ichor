//! SU-2: setup actions (`plans/setup-architecture.md` §5.3). Each verb and
//! each answer played from a board by `SetupDriver`, the refusals at load and
//! in play, and a stack built this way played on as one game.

use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::engine::resolve::ResolvedTarget;
use mtgsim::scenario::{BuiltScenario, Scenario, ScenarioError, ScenarioErrorKind, SetupDriver};
use mtgsim::state::game::RandomStreams;
use mtgsim::state::game_state::GameState;
use mtgsim::types::ids::ObjectId;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{ScriptedDecisionProvider, SeatMode};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

fn build_with(registry: &CardRegistry, text: &str) -> BuiltScenario {
    Scenario::parse(text).and_then(|s| s.build(registry)).unwrap_or_else(|r| panic!("{r}"))
}

fn refused(text: &str) -> ScenarioError {
    Scenario::parse(text).and_then(|s| s.build(&CardRegistry::default_registry())).map(|_| ()).expect_err(text)
}

/// The board once every setup action is taken. Each line's action ends the
/// priority round it is taken in, so a round a line; the seats' own provider
/// is a script that expects nothing, so nobody else is asked.
fn played_with(registry: &CardRegistry, text: &str) -> GameState {
    let BuiltScenario { mut game, setup } = build_with(registry, text);
    let seats = ScriptedDecisionProvider::new();
    let lines = setup.len();
    let driver = SetupDriver::new(setup, &seats);
    for _ in 0..lines {
        assert_eq!(game.state.run_priority_round(&driver), Ok(PriorityResult::ActionTaken), "{text}");
    }
    game.state
}

fn played(text: &str) -> GameState {
    played_with(&CardRegistry::default_registry(), text)
}

/// The refusal a line meets in play: the driver's panic, which names it.
fn refused_in_play(text: &str) -> String {
    let BuiltScenario { mut game, setup } = build_with(&CardRegistry::default_registry(), text);
    let seats = ScriptedDecisionProvider::new();
    let driver = SetupDriver::new(setup, &seats);
    let refusal = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        for _ in 0..4 {
            let _ = game.state.run_priority_round(&driver);
        }
    }))
    .expect_err(text);
    refusal.downcast_ref::<String>().cloned().unwrap_or_default()
}

/// The object named `name` in `zone`, the only one there.
fn object(state: &GameState, zone: Zone, name: &str) -> ObjectId {
    let mut found = state.objects.values().filter(|o| o.zone == zone && o.card_data.name == name).map(|o| o.id);
    let one = found.next().unwrap_or_else(|| panic!("no {name} in {zone:?}"));
    assert!(found.next().is_none(), "two of {name} in {zone:?}");
    one
}

fn permanent(state: &GameState, name: &str) -> ObjectId {
    object(state, Zone::Battlefield, name)
}

/// Each stack object's targets, bottom first, an instance of "target" each.
fn targets_on_the_stack(state: &GameState) -> Vec<Vec<Vec<ResolvedTarget>>> {
    let entry = |id: &ObjectId| state.stack_entries[id].chosen_targets.iter().map(|instance| instance.chosen.clone()).collect();
    state.stack.iter().map(entry).collect()
}

/// §5.3's stack: Bolt at the Bears, Giant Growth in response, and the
/// Thaumaturgist on top, with a second Bolt in player 0's hand.
const SECTION_5_3: &str = include_str!("../scenarios/bolt-into-giant-growth.scenario");

/// Each verb, each seat's, and each seat passing until the next line's seat
/// holds priority: the stack is the lines' in order, each with its target,
/// the costs paid from the untapped lands, and the next prompt is the seats'.
#[test]
fn each_verb_builds_the_stack_its_lines_describe() {
    let BuiltScenario { mut game, setup } = build_with(&CardRegistry::default_registry(), SECTION_5_3);
    let stops = SeatMode { stops_at_every_priority_point: true, ..SeatMode::default() };
    let seats = ScriptedDecisionProvider::new().with_seat_mode(stops);
    let driver = SetupDriver::new(setup, &seats);
    for _ in 0..3 {
        assert_eq!(game.state.run_priority_round(&driver), Ok(PriorityResult::ActionTaken));
    }
    let state = &game.state;
    let bears = ResolvedTarget::Object(permanent(state, "Grizzly Bears"));
    assert_eq!(state.stack[..2], [object(state, Zone::Stack, "Lightning Bolt"), object(state, Zone::Stack, "Giant Growth")]);
    let ability = state.stack_entries[&state.stack[2]].ability_identity.expect("an activated ability");
    assert_eq!(ability.source.id, permanent(state, "Merfolk Thaumaturgist"));
    assert_eq!(targets_on_the_stack(state), [[[bears]], [[bears]], [[bears]]]);
    for paid in ["Forest", "Merfolk Thaumaturgist"] {
        assert!(state.battlefield[&permanent(state, paid)].tapped, "{paid}");
    }
    let mountains = state.battlefield_ordered().into_iter().filter(|(id, _)| state.objects[id].card_data.name == "Mountain").map(|(_, entry)| entry.tapped);
    assert_eq!(mountains.collect::<Vec<_>>(), [true, false], "one Mountain pays the first Bolt");
    // The seats' own from here: each is asked, and passing resolves the top.
    seats.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    seats.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    assert_eq!(game.state.run_priority_round(&driver), Ok(PriorityResult::StackResolved));
    assert_eq!(game.state.stack.len(), 2);
}

/// The `targeting` segments in the line's order: each of Seeds of
/// Strength's three targets takes the next one it offers, the same creature
/// twice (CR 115.3).
#[test]
fn each_target_takes_the_next_segment_it_offers() {
    let state = played(
        "hand 0: Seeds of Strength\nbattlefield: Forest | controller 0\nbattlefield: Plains | controller 0\n\
         battlefield: Grizzly Bears | controller 0\nbattlefield: Savannah Lions | controller 0\n\
         then: player 0 casts Seeds of Strength | targeting Grizzly Bears | targeting Grizzly Bears | targeting Savannah Lions",
    );
    let [bears, lions] = ["Grizzly Bears", "Savannah Lions"].map(|name| ResolvedTarget::Object(permanent(&state, name)));
    assert_eq!(targets_on_the_stack(&state), [[[bears], [bears], [lions]]]);
}

#[test]
fn a_target_can_be_a_player() {
    let state = played(
        "hand 0: Lightning Bolt\nbattlefield: Mountain | controller 0\nbattlefield: Grizzly Bears | controller 1\n\
         then: player 0 casts Lightning Bolt | targeting player 1",
    );
    assert_eq!(targets_on_the_stack(&state), [[[ResolvedTarget::Player(1)]]]);
}

/// `ability N` is the ability's place among the permanent's abilities: Mind
/// Stone's first is a mana ability and its second draws, which is also its
/// one activated ability. Its {1} is paid from the Plains, though the stone
/// is listed first, since the driver taps the line's own permanent last.
#[test]
fn ability_n_names_the_activated_ability() {
    let board = "battlefield: Mind Stone | controller 0\nbattlefield: Plains | controller 0\n";
    for line in ["then: player 0 activates Mind Stone | ability 2", "then: player 0 activates Mind Stone"] {
        let state = played(&format!("{board}{line}"));
        let ability = state.stack_entries[&state.stack[0]].ability_identity.expect("an activated ability");
        assert_eq!(ability.source.id, object(&state, Zone::Graveyard, "Mind Stone"), "{line}: sacrificed for its cost");
        assert!(state.battlefield[&permanent(&state, "Plains")].tapped, "{line}");
    }
    assert!(refused(&format!("{board}then: player 0 activates Mind Stone | ability 1")).message.contains("is a mana ability"));
    assert!(refused(&format!("{board}then: player 0 activates Mind Stone | ability 3")).message.contains("has no ability 3"));
    assert!(refused("battlefield: Grizzly Bears | controller 0\nthen: player 0 activates Grizzly Bears").message.contains("no activated ability"));
}

/// What can be refused before play is refused at load, naming the line.
#[test]
fn a_setup_action_is_refused_at_load_naming_its_line() {
    use ScenarioErrorKind::{Reference, Unreachable};
    for (text, kind, says) in [
        ("hand 1: Lightning Bolt\nthen: player 0 casts Lightning Bolt", Reference, "is not in player 0's hand"),
        ("hand 0: Lightning Bolt\nhand 0: Lightning Bolt\nthen: player 0 casts Lightning Bolt", Reference, "lines 1 and 2"),
        ("hand 0: Lightning Bolt [a]\nthen: player 0 casts Lightning Bolt [a]\nthen: player 0 casts Lightning Bolt [a]", Reference, "line 2 already"),
        ("hand 0: Forest\nthen: player 0 casts Forest", Unreachable, "CR 305.1"),
        ("hand 0: Lightning Bolt\nthen: player 2 casts Lightning Bolt", Reference, "not in a 2-player game"),
        ("battlefield: Merfolk Thaumaturgist | controller 1\nthen: player 0 activates Merfolk Thaumaturgist", Unreachable, "CR 602.2"),
        ("hand 0: Lightning Bolt\nthen: player 0 casts Lightning Bolt | targeting Grizly Bears", Reference, "names no permanent"),
        // Hands and libraries are no targets, so the library's Bears is not one.
        ("hand 0: Lightning Bolt\nlibrary 1: Grizzly Bears\nthen: player 0 casts Lightning Bolt | targeting Grizzly Bears", Reference, "names no permanent"),
        ("players 3\nplayer 2: left the game\nhand 0: Lightning Bolt\nthen: player 0 casts Lightning Bolt | targeting player 2", Unreachable, "no target"),
    ] {
        let refusal = refused(text);
        assert_eq!((refusal.kind, refusal.line), (kind, Some(text.lines().count())), "{text}: {refusal}");
        assert!(refusal.message.contains(says), "{text}: {refusal}");
    }
}

/// What only play shows is refused when it shows, naming the line: an action
/// its seat is not offered when it holds priority, rather than taken a turn
/// later; one the engine rewinds once picked; a target the choice does not
/// offer, one too few, or one too many; and a question no line answers.
#[test]
fn a_setup_action_is_refused_in_play_naming_its_line() {
    let bolt = "hand 0: Lightning Bolt\nbattlefield: Mountain | controller 0\nbattlefield: Grizzly Bears | controller 1\n";
    for (text, says) in [
        (
            "hand 1: Mind Rot\nbattlefield: Swamp | controller 1, x3\nthen: player 1 casts Mind Rot | targeting player 0".to_string(),
            "player 1 holds priority, and the engine does not offer this",
        ),
        (format!("{bolt}then: player 0 activates Mind Stone"), "Mind Stone is not on the battlefield"),
        ("battlefield: Mind Stone | controller 0\nthen: player 0 activates Mind Stone".to_string(), "rewound it"),
        (
            "hand 0: Giant Growth\nbattlefield: Forest | controller 0\nbattlefield: Grizzly Bears | controller 0\n\
             battlefield: Savannah Lions | controller 0\nthen: player 0 casts Giant Growth | targeting player 1"
                .to_string(),
            "no `targeting` segment left",
        ),
        (format!("{bolt}then: player 0 casts Lightning Bolt"), "no `targeting` segment left"),
        (format!("{bolt}then: player 0 casts Lightning Bolt | targeting player 1 | targeting Grizzly Bears"), "names Grizzly Bears as a target"),
        (
            "hand 0: Bone Splinters\nbattlefield: Swamp | controller 0\nbattlefield: Grizzly Bears | controller 0\n\
             battlefield: Savannah Lions | controller 0\nbattlefield: Hill Giant | controller 1\n\
             then: player 0 casts Bone Splinters | targeting Hill Giant"
                .to_string(),
            "is asked ChooseSacrificeForCost, which no line answers",
        ),
    ] {
        let line = text.lines().count();
        let refusal = if says.ends_with("on the battlefield") { refused(&text).to_string() } else { refused_in_play(&text) };
        assert!(refusal.starts_with(&format!("line {line}")), "{text}: {refusal}");
        assert!(refusal.contains(says), "{text}: {refusal}");
    }
}

/// The stack setup actions build, played on by the seats' agents: the same
/// file and seed are one game, the lines played before anyone else is asked.
#[test]
fn a_stack_built_by_setup_actions_played_twice_is_one_game() {
    let play = || {
        let BuiltScenario { mut game, setup } = build_with(&CardRegistry::default_registry(), SECTION_5_3);
        game.state.record_events();
        let agents = ManaWindowStop::new(RandomDecisionProvider::seeded(RandomStreams::from_seed(7).agents));
        let result = game.resume(&SetupDriver::new(setup, &agents)).unwrap();
        (result, game.event_log_snapshot())
    };
    let (first, second) = (play(), play());
    assert!(first.1.len() > 20, "a whole game: {} lines", first.1.len());
    assert_eq!(first, second);
}
