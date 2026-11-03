//! `options-named-by-what-they-are`: every option the engine offers is named
//! by what it is, so a game's log replays to the same game wherever a build
//! lists the options (`setup-architecture.md#7e-pausing-at-a-decision`).
//!
//! The board is Bridge from Below's ruling: a nontoken creature of yours and
//! one of an opponent's die in one event, both of its abilities trigger, and
//! you choose their order. The Zombie ability checks "if this card is in your
//! graveyard" again as it resolves, so the order decides whether a Zombie is
//! made.

use std::sync::{Arc, Mutex};

use mtgsim::cards::registry::CardRegistry;
use mtgsim::state::decision_log::{self, AnswerLine, BuiltStart, Chosen, GameStart};
use mtgsim::state::game_state::GameState;
use mtgsim::types::ids::ObjectId;
use mtgsim::types::mana::ManaCost;
use mtgsim::types::triggers::TriggerTier;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{AbilityPlace, ChoiceKind, ChoiceOption, TriggerOption, TriggeredAbility};
use mtgsim::ui::decision::{DispatchDecisionProvider, PriorityAction, ScriptedDecisionProvider};
use mtgsim::ui::display::format_event_log;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::replay::Replay;

const BRIDGE_AND_PYROCLASM: &str = "\
hand 0: Pyroclasm
graveyard 0: Bridge from Below
battlefield: Mountain | controller 0
battlefield: Mountain [second] | controller 0
battlefield: Grizzly Bears | controller 0
battlefield: Grizzly Bears [theirs] | controller 1
library 0: Mountain | x5
library 1: Mountain | x5
";

/// The board built, recording its events and writing its log into the
/// returned text, as a client attaches the engine's writer.
fn recorded() -> (BuiltStart, Arc<Mutex<String>>) {
    let start = GameStart::Scenario { path: "bridge".to_string(), seed: 0, text: BRIDGE_AND_PYROCLASM.to_string() };
    let mut built = start.build(&CardRegistry::default_registry()).expect("the board builds");
    let text = Arc::new(Mutex::new(decision_log::opening(&start).iter().map(|line| format!("{line}\n")).collect::<String>()));
    let (writer, answers) = (Arc::clone(&text), Arc::new(Mutex::new(0)));
    let game = built.game_mut();
    game.state.record_events();
    game.state.log_decisions(move |game, decision| {
        let mut number = answers.lock().unwrap();
        *number += 1;
        writer.lock().unwrap().push_str(&format!("{}\n", AnswerLine::of(game, *number, decision)));
    });
    (built, text)
}

/// The one card named `name` that `controller` controls, or that is in a
/// graveyard or a hand when `controller` is `None`.
fn named(game: &GameState, name: &str, controller: Option<usize>) -> ObjectId {
    // ORDER-FREE: one card of each name and controller on this board.
    let found: Vec<ObjectId> = game
        .objects
        .values()
        .filter(|object| object.card_data.name == name)
        .filter(|object| controller.is_none_or(|seat| game.battlefield.get(&object.id).is_some_and(|entry| entry.controller == seat)))
        .map(|object| object.id)
        .collect();
    let [id] = found[..] else { panic!("{} cards named {name}", found.len()) };
    id
}

/// Bridge from Below's trigger of its ability at `place`, for `died`.
fn bridge_trigger(game: &GameState, bridge: ObjectId, place: usize, died: ObjectId) -> ChoiceOption {
    let ability = game.objects[&bridge].card_data.abilities[place - 1].id;
    ChoiceOption::Trigger(TriggerOption {
        source: game.object_ref(bridge).expect("in the store"),
        ability: TriggeredAbility::OfItsSource { ability: ability.definition(), granted_by: None, place: AbilityPlace(Some(place)) },
        triggered_by: vec![died],
    })
}

/// Casts Pyroclasm, which kills both Bears at once, and orders Bridge from
/// Below's two triggers: the Zombie's on top when `token_first`, so it
/// resolves first. Returns the game and its log, read back.
fn played(token_first: bool) -> (BuiltStart, decision_log::Log) {
    let (mut built, record) = recorded();
    let state = &built.game().state;
    let pyroclasm = named(state, "Pyroclasm", None);
    let bridge = named(state, "Bridge from Below", None);
    let (yours, theirs) = (named(state, "Grizzly Bears", Some(0)), named(state, "Grizzly Bears", Some(1)));
    // ORDER-FREE: the two Mountains are alike; either pays.
    let mountains: Vec<ObjectId> = state.objects.values().filter(|o| o.card_data.name == "Mountain" && state.battlefield.contains_key(&o.id)).map(|o| o.id).collect();
    let tap = |id: ObjectId| ChoiceOption::Action(PriorityAction::ActivateAbility(id, state.objects[&id].card_data.abilities[0].id));
    let makes_a_zombie = bridge_trigger(state, bridge, 1, yours);
    let exiles_itself = bridge_trigger(state, bridge, 2, theirs);

    let caster = ScriptedDecisionProvider::new();
    caster.expect_choice(ChoiceKind::PriorityAction, vec![ChoiceOption::Action(PriorityAction::CastSpell(pyroclasm))]);
    let window = ChoiceKind::ManaAbilityWindow { spell_or_ability_id: pyroclasm, remaining_cost: ManaCost::build(&[], 0) };
    caster.expect_choice(window.clone(), vec![tap(mountains[0])]);
    caster.expect_choice(window, vec![tap(mountains[1])]);
    // The first item goes on the stack lowest, so the last resolves first.
    let order = if token_first { vec![exiles_itself, makes_a_zombie] } else { vec![makes_a_zombie, exiles_itself] };
    caster.expect_order(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First }, order);
    let seats = DispatchDecisionProvider::new(vec![Box::new(ManaWindowStop::new(caster)), Box::new(ScriptedDecisionProvider::new())]);
    built.game_mut().until_stopped(|game| game.resume_turn_at_priority(&seats)).expect("the turn plays");

    let log = decision_log::read(&record.lock().unwrap()).expect("the log reads");
    (built, log)
}

fn zombies(built: &BuiltStart) -> usize {
    // ORDER-FREE: a count.
    built.game().state.battlefield.keys().filter(|id| built.game().state.objects[id].card_data.name.starts_with("Zombie")).count()
}

/// The log names each of the Bridge's two triggers by its ability and the
/// creature it is for, and a replay of each order plays it again: a Zombie
/// when its trigger resolved first, none when the exile did.
#[test]
fn bridge_from_below_replays_its_trigger_order_both_ways() {
    for token_first in [true, false] {
        let (original, log) = played(token_first);
        let state = &original.game().state;
        let bridge = named(state, "Bridge from Below", None);
        assert_eq!(state.objects[&bridge].zone, Zone::Exile, "the exile resolved either way");
        assert_eq!(zombies(&original), usize::from(token_first), "token_first: {token_first}");

        let ordered = log
            .answers
            .iter()
            .find_map(|line| match &line.chosen {
                Chosen::Order(items) if line.kind == "OrderTriggers" => Some(items.clone()),
                _ => None,
            })
            .expect("the order is a line of the log");
        assert!(
            ordered.iter().all(|item| item.starts_with(&format!("Bridge from Below ({bridge}), ability "))),
            "each trigger names its ability: {ordered:?}"
        );
        assert_ne!(ordered[0], ordered[1], "and the two read apart");

        let mut replayed = log.start.build(&CardRegistry::default_registry()).expect("the log's start builds");
        replayed.game_mut().state.record_events();
        let replay = Replay::of(log);
        replayed.game_mut().until_stopped(|game| game.resume_turn_at_priority(&replay)).expect("the replay plays");
        assert_eq!(replay.remaining(), 0, "every answer the log holds was asked for");
        assert_eq!(format_event_log(&replayed.game().state), format_event_log(&original.game().state));
        assert_eq!(zombies(&replayed), zombies(&original));
    }
}
