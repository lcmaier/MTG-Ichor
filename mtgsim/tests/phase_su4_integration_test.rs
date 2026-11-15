//! SU-4, the replay (`setup-architecture.md` §7.1–§7.3): the stop that ends a
//! run from inside a prompt, the decision log's text, and the replay that
//! answers from it.

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use mtgsim::cards::registry::CardRegistry;
use mtgsim::scenario::Scenario;
use mtgsim::state::decision_log::{self, AnswerLine, BuiltStart, Chosen, GameStart, Log};
use mtgsim::state::game::{Game, Halt};
use mtgsim::state::game_state::{GameResult, GameState};
use mtgsim::types::effects::{EffectRecipient, SelectionFilter, TargetCount};
use mtgsim::types::ids::PlayerId;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, DispatchDecisionProvider, PriorityAction, ScriptedDecisionProvider, Stop};
use mtgsim::ui::display::format_event_log;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;
use mtgsim::ui::replay::Replay;

/// A Lightning Bolt castable at the first priority prompt, so that prompt is
/// asked rather than passed for the seat.
const BOLT_IN_HAND: &str = "hand 0: Lightning Bolt\nbattlefield: Mountain | controller 0\nbattlefield: Grizzly Bears | controller 1\n";

fn board(text: &str) -> Game {
    Scenario::parse(text).and_then(|s| s.build(&CardRegistry::default_registry())).unwrap_or_else(|r| panic!("{r}")).game
}

/// Raises its stop at the first prompt, having written the board as that
/// prompt shows it.
struct Stopper {
    stop: Stop,
    board_at_prompt: RefCell<Option<String>>,
}

impl Stopper {
    fn new(stop: Stop) -> Stopper {
        Stopper { stop, board_at_prompt: RefCell::new(None) }
    }

    fn stop_here(&self, game: &GameState) -> ! {
        *self.board_at_prompt.borrow_mut() = Some(Scenario::write(game).to_string());
        self.stop.clone().raise()
    }
}

impl DecisionProvider for Stopper {
    fn pick_n(&self, game: &GameState, _: PlayerId, _: &ChoiceContext, _: &[ChoiceOption], _: (usize, usize)) -> Vec<usize> {
        self.stop_here(game)
    }
    fn pick_number(&self, game: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, _: u64) -> u64 {
        self.stop_here(game)
    }
    fn allocate(&self, game: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, _: &[ChoiceOption], _: &[u64], _: Option<&[u64]>) -> Vec<u64> {
        self.stop_here(game)
    }
    fn choose_ordering(&self, game: &GameState, _: PlayerId, _: &ChoiceContext, _: &[ChoiceOption]) -> Vec<usize> {
        self.stop_here(game)
    }
}

/// A stop ends the run inside its prompt, and `until_stopped` returns it.
/// The board is the one the prompt showed, and every run entry refuses the
/// game after, since the frames unwound held work in flight.
#[test]
fn a_stopped_game_is_read_and_never_continued() {
    let mut game = board(BOLT_IN_HAND);
    let stopper = Stopper::new(Stop::Superseded);
    assert_eq!(game.until_stopped(|game| game.resume(&stopper)), Err(Halt::Stopped(Stop::Superseded)));
    assert_eq!(Some(Scenario::write(&game.state).to_string()), stopper.board_at_prompt.take(), "the board its prompt showed");

    let seats = ScriptedDecisionProvider::new();
    let entries = [game.run_turn(&seats), game.setup(&seats), game.state.run_priority_round(&seats).map(|_| ())];
    for refused in entries {
        assert!(refused.is_err_and(|why| why.contains("a stopped game is read, never continued")));
    }
}

/// Only a stop is caught: an engine bug's panic passes through
/// `until_stopped` as it would have, payload and all.
#[test]
fn a_panic_that_is_not_a_stop_passes_through() {
    struct Bug;
    impl DecisionProvider for Bug {
        fn pick_n(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: &[ChoiceOption], _: (usize, usize)) -> Vec<usize> {
            panic!("an engine bug")
        }
        fn pick_number(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, _: u64) -> u64 {
            panic!("an engine bug")
        }
        fn allocate(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, _: &[ChoiceOption], _: &[u64], _: Option<&[u64]>) -> Vec<u64> {
            panic!("an engine bug")
        }
        fn choose_ordering(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: &[ChoiceOption]) -> Vec<usize> {
            panic!("an engine bug")
        }
    }
    let mut game = board(BOLT_IN_HAND);
    let payload = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| game.until_stopped(|game| game.resume(&Bug)))).unwrap_err();
    assert_eq!(payload.downcast_ref::<&str>(), Some(&"an engine bug"));
}

// ---------------------------------------------------------------------------
// The replay
// ---------------------------------------------------------------------------

/// `start` built, recording its events and writing its record into the
/// returned text, as a client attaches the engine's writer.
fn recorded(start: &GameStart) -> (BuiltStart, Arc<Mutex<String>>) {
    let mut built = start.build(&CardRegistry::default_registry()).expect("the start builds");
    let text = Arc::new(Mutex::new(decision_log::opening(start).iter().map(|line| format!("{line}\n")).collect::<String>()));
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

/// A record's start built again, recording its events, to replay into.
fn rebuilt(log: &Log) -> BuiltStart {
    let mut built = log.start.build(&CardRegistry::default_registry()).expect("the record's start builds");
    built.game_mut().state.record_events();
    built
}

/// Seeded random agents, each behind `ManaWindowStop` as `fuzz_games` seats
/// them.
fn agents(players: u64, seed: u64) -> DispatchDecisionProvider {
    DispatchDecisionProvider::new(
        (0..players).map(|seat| Box::new(ManaWindowStop::new(RandomDecisionProvider::seeded(seed + seat))) as Box<dyn DecisionProvider>).collect(),
    )
}

fn events(built: &BuiltStart) -> Vec<String> {
    format_event_log(&built.game().state)
}

/// `setup-architecture.md` §5.3's stack, built by setup actions and played
/// to its end by random seats: the game, and its record read back.
fn bolt_into_giant_growth() -> (BuiltStart, Result<GameResult, Halt>, Log) {
    let path = "scenarios/bolt-into-giant-growth.scenario";
    let text = std::fs::read_to_string(path).expect("the sample");
    let (mut built, record) = recorded(&GameStart::Scenario { path: path.to_string(), seed: 7, text });
    let ended = built.play(&agents(2, 7));
    let log = decision_log::read(&record.lock().unwrap()).expect("the record reads");
    (built, ended, log)
}

/// A scenario's setup actions are answers its record holds, so a replay
/// plays them with the rest, with no setup driver, to the same game.
#[test]
fn a_scenario_game_and_its_setup_actions_replay_to_the_same_game() {
    let (original, ended, log) = bolt_into_giant_growth();
    assert!(ended.is_ok(), "{ended:?}");
    let cast = |line: &AnswerLine| matches!(&line.chosen, Chosen::Picks(picks) if picks.iter().any(|p| p.starts_with("cast Lightning Bolt (")));
    assert!(log.answers.iter().any(cast), "the setup driver's cast is a line of the record");
    let mut replayed = rebuilt(&log);
    let replay = Replay::of(log);
    assert_eq!(replayed.replay(&replay), ended);
    assert_eq!(replay.remaining(), 0, "every answer the record holds was asked for");
    assert_eq!(events(&replayed), events(&original));
}

/// A cast from the hand paid with exactly its mana, under `ManaWindowStop`
/// as a seat is: the target, the window and the window's close are lines
/// like any other, and the replay casts the same spell.
#[test]
fn a_cast_from_hand_with_exact_mana_replays() {
    let start = GameStart::Scenario { path: "bolt".to_string(), seed: 0, text: BOLT_IN_HAND.to_string() };
    let (mut original, record) = recorded(&start);
    let state = &original.game().state;
    let named = |name: &str| state.objects.values().find(|object| object.card_data.name == name).map(|object| object.id).expect(name);
    let (bolt, mountain, bears) = (named("Lightning Bolt"), named("Mountain"), named("Grizzly Bears"));
    let tap = state.objects[&mountain].card_data.abilities[0].id;
    let caster = ScriptedDecisionProvider::new();
    caster.expect_choice(ChoiceKind::PriorityAction, vec![ChoiceOption::Action(PriorityAction::CastSpell(bolt))]);
    let target = EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1));
    caster.expect_choice(ChoiceKind::SelectRecipients { recipient: target, spell_id: bolt }, vec![ChoiceOption::Object(bears)]);
    let window = ChoiceKind::ManaAbilityWindow { spell_or_ability_id: bolt, remaining_cost: ManaCost::build(&[ManaType::Red], 0) };
    caster.expect_choice(window, vec![ChoiceOption::Action(PriorityAction::ActivateAbility(mountain, tap))]);
    let seats = DispatchDecisionProvider::new(vec![Box::new(ManaWindowStop::new(caster)), Box::new(ScriptedDecisionProvider::new())]);
    original.game_mut().until_stopped(|game| game.resume_turn_at_priority(&seats)).expect("the turn plays");
    assert_eq!(original.game().state.objects[&bears].zone, Zone::Graveyard, "the Bolt killed the Bears");

    let log = decision_log::read(&record.lock().unwrap()).expect("the record reads");
    let mut replayed = rebuilt(&log);
    let replay = Replay::of(log);
    replayed.game_mut().until_stopped(|game| game.resume_turn_at_priority(&replay)).expect("the replay plays");
    assert_eq!(replay.remaining(), 0);
    assert_eq!(events(&replayed), events(&original));
}

/// What a replay cannot follow stops it at the line, saying why: another
/// question than the line's, a choice no longer offered, an answer that does
/// not fit its question.
#[test]
fn a_record_that_disagrees_stops_at_its_line_and_says_why() {
    let (_, _, log) = bolt_into_giant_growth();
    let choice = log.answers.iter().position(|line| !line.forced).expect("a line that chose");
    let diverged = |log: Log| -> (usize, String) {
        let mut built = rebuilt(&log);
        match built.replay(&Replay::new(log.answers)) {
            Err(Halt::Stopped(Stop::Diverged { line, why })) => (line, why),
            other => panic!("no divergence: {other:?}"),
        }
    };
    let mut asked = log.clone();
    asked.answers[choice].kind = "DeclareAttackers".to_string();
    let (line, why) = diverged(asked);
    assert_eq!(line, choice + 1);
    assert!(why.contains("answering DeclareAttackers") && why.contains("the engine asks player 0 PriorityAction"), "{why}");

    let mut gone = log.clone();
    gone.answers[choice].chosen = Chosen::Picks(vec!["cast Shock (#99)".to_string()]);
    assert_eq!(diverged(gone), (choice + 1, "it chose cast Shock (#99), which is not offered".to_string()));

    let mut two = log.clone();
    if let Chosen::Picks(picks) = &mut two.answers[choice].chosen {
        picks.push("pass".to_string());
    }
    let (_, why) = diverged(two);
    assert_eq!(why, "its answer does not fit: 2 selections, expected 1-1");
}

/// A question with one legal answer may be asked by one build and taken by
/// another: a forced line no question asks is passed over, and a forced
/// question no line has is answered, and neither changes the game.
#[test]
fn a_one_answer_question_two_builds_disagree_on_asking_changes_nothing() {
    let (original, ended, log) = bolt_into_giant_growth();
    let forced = log.answers.iter().position(|line| line.forced).expect("a forced line");
    let mut dropped = log.answers.clone();
    dropped.remove(forced);
    let mut doubled = log.answers.clone();
    doubled.insert(forced, log.answers[forced].clone());
    for answers in [dropped, doubled] {
        let mut replayed = rebuilt(&log);
        let replay = Replay::new(answers);
        assert_eq!(replayed.replay(&replay), ended);
        assert_eq!(events(&replayed), events(&original));
    }
}

/// Spent, a replay hands the game to the seats, or with none stops the run;
/// a replay superseded stops at its next answer.
#[test]
fn a_spent_replay_hands_over_to_the_seats_or_stops() {
    let (_, _, log) = bolt_into_giant_growth();
    let half = log.answers.len() / 2;
    let mut built = rebuilt(&log);
    assert_eq!(built.replay(&Replay::new(log.answers[..half].to_vec())), Err(Halt::Stopped(Stop::LogSpent { answered: half })));

    let seats = agents(2, 99);
    let replay = Replay::new(log.answers[..half].to_vec()).then(&seats);
    let mut built = rebuilt(&log);
    let ended = built.replay(&replay);
    assert!(ended.is_ok(), "the seats played the game to its end: {ended:?}");
    assert_eq!(replay.control().answered(), half);

    let replay = Replay::new(log.answers.clone());
    replay.control().supersede();
    let mut built = rebuilt(&log);
    assert_eq!(built.replay(&replay), Err(Halt::Stopped(Stop::Superseded)));
}
