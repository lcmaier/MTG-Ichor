//! The bridge with no window: another thread plays the window's part over the
//! channels.

#[path = "support/games.rs"]
mod games;
#[path = "support/window_by_rule.rs"]
mod window_by_rule;

use std::sync::Arc;

use std::time::{Duration, Instant};

use devgui::bridge::{GameSetup, Outcome, ToWindow, spawn_game};
use devgui::session::Session;
use devgui::view_model::{Input, WindowState};
use mtgsim::cards::registry::CardRegistry;
use mtgsim::scenario::Scenario;
use devgui::prompt::{Answer, Primitive, Reply};
use games::{dealt, from_board, next};
use window_by_rule::play_by_rule;

#[test]
fn a_whole_game_finishes_with_a_thread_playing_the_window() {
    let log = std::env::temp_dir().join("devgui-headless-seed-7.log");
    let window = std::thread::spawn({
        let log = log.clone();
        move || play_by_rule(dealt(7, Some(log)), |_| {})
    });
    let (outcome, answered) = window.join().expect("the window's thread panicked");
    assert!(matches!(outcome, Outcome::Won(_) | Outcome::Draw), "{outcome:?}");
    assert!(answered > 20, "a game of Magic asks seat 0 more than {answered} questions");

    let log = std::fs::read_to_string(&log).expect("the decision log");
    let lines: Vec<&str> = log.lines().collect();
    assert_eq!(lines[..2], ["seed 7", "pool Performance"]);
    assert!(lines[2].starts_with("deck 0 ") && lines[3].starts_with("deck 1 "));
    assert!(lines[4].starts_with("answer 1 [turn 1, "), "each answer says when: {}", lines[4]);
    let answers: Vec<&&str> = lines.iter().filter(|l| l.starts_with("answer ")).collect();
    let seat_0 = answers.iter().filter(|l| l.contains("] player 0 ")).count();
    assert!(seat_0 > answered, "seat 0's lines hold the window's answers, its decorators' and the engine's passes");
    assert!(answers.iter().any(|l| l.contains("] player 1 ")), "and the other seat's answers");
    assert!(answers.iter().any(|l| l.ends_with(" forced")), "a question with one legal answer is marked");
    assert_eq!(lines.last(), Some(&format!("outcome {outcome:?}").as_str()));
}

/// CR 103.8a: seat 0 plays first in a two-player game and skips its first draw
/// step, so the window's first prompt, in turn 1's main phase, shows the seven
/// cards it kept.
#[test]
fn the_window_starts_holding_seven_since_its_first_draw_is_skipped() {
    let engine = spawn_game(dealt(7, None), Arc::new(|| {}));
    let ToWindow::Prompt { snapshot, .. } = next(&engine) else {
        panic!("expected a prompt first");
    };
    assert_eq!((snapshot.turn, snapshot.active_player, snapshot.phase.as_str()), (1, 0, "Precombat Main"));
    assert_eq!(snapshot.players[0].hand.len(), 7);
    assert_eq!(snapshot.players[0].library.len(), 53);
}

#[test]
fn an_illegal_answer_reaches_the_window_as_the_validators_message() {
    let engine = spawn_game(dealt(7, None), Arc::new(|| {}));
    match next(&engine) {
        ToWindow::Prompt { prompt, .. } if matches!(prompt.primitive, Primitive::PickN { .. }) => {
            engine.answers.send(Reply::Answer(Answer::Picks(vec![99]))).unwrap();
        }
        other => panic!("expected a pick as the first prompt, got {other:?}"),
    }
    match next(&engine) {
        ToWindow::Panicked { message } => assert!(message.contains("DP returned index 99"), "{message}"),
        other => panic!("expected the validator's panic, got {other:?}"),
    }
}

/// CR 509.1a's re-ask, end to end: the window declares its one Wall of Stone
/// blocking both Bears, the engine asks again and says why, and a legal
/// declaration goes through.
#[test]
fn an_illegal_block_is_asked_again_with_the_rule_it_broke() {
    let engine = spawn_game(from_board("reask.scenario", None), Arc::new(|| {}));
    let mut state = WindowState::default();
    state.receive(next(&engine));
    let rejected = |state: &WindowState| state.prompt.as_ref().and_then(|prompt| prompt.rejected.clone());
    assert_eq!(state.prompt.as_ref().map(|prompt| prompt.kind.as_str()), Some("DeclareBlockers"));
    assert_eq!(rejected(&state), None, "a first ask rejects nothing");
    state.input(Input::OptionButton(0));
    state.input(Input::OptionButton(1));
    engine.answers.send(state.input(Input::Done).expect("both blocks declared")).unwrap();

    state.receive(next(&engine));
    let why = rejected(&state).expect("the re-ask says why");
    assert!(why.starts_with("Those blocks are illegal: Wall of Stone ("), "{why}");
    assert!(why.ends_with(") can block only one attacker (CR 509.1a)"), "{why}");
    state.input(Input::OptionButton(0));
    engine.answers.send(state.input(Input::Done).expect("one block declared")).unwrap();
    let message = next(&engine);
    assert!(!matches!(message, ToWindow::Panicked { .. } | ToWindow::Finished { .. }), "{message:?}");
}

/// A game from a scenario: the log opens with the file, its seed and its
/// text verbatim, so it is a save even after the file changes, and the game
/// plays from the board to its end.
#[test]
fn a_whole_game_from_a_scenario_logs_the_file_it_began_from() {
    let log = std::env::temp_dir().join("devgui-headless-scenario.log");
    let setup = from_board("main.scenario", Some(log.clone()));
    let board = std::fs::read_to_string(setup.scenario.as_ref().unwrap()).unwrap();
    let (outcome, answered) = play_by_rule(setup, |_| {});
    assert!(matches!(outcome, Outcome::Won(_) | Outcome::Draw), "{outcome:?}");
    let log = std::fs::read_to_string(&log).expect("the decision log");
    let lines: Vec<&str> = log.lines().collect();
    assert!(lines[0].starts_with("scenario ") && lines[0].ends_with("main.scenario"), "{}", lines[0]);
    assert_eq!(lines[1..3], ["seed 0", "begin scenario text"]);
    let text_lines = board.lines().count();
    assert_eq!(lines[3..3 + text_lines], board.lines().collect::<Vec<_>>()[..]);
    assert_eq!(lines[3 + text_lines], "end scenario text");
    let seat_0 = lines.iter().filter(|l| l.starts_with("answer ") && l.contains("] player 0 ")).count();
    assert!(seat_0 > answered, "every answer the window gave is among seat 0's lines");
}

/// A file the loader refuses reaches the window as its line and its fix.
#[test]
fn a_refused_scenario_reaches_the_window_with_its_line() {
    let path = std::env::temp_dir().join("devgui-refused.scenario");
    std::fs::write(&path, "turn 2
hand 0: Grizly Bears
").unwrap();
    let engine = spawn_game(GameSetup { scenario: Some(path), ..dealt(0, None) }, Arc::new(|| {}));
    match next(&engine) {
        ToWindow::Refused { message } => assert!(message.starts_with("line 2: Grizly Bears is not registered"), "{message}"),
        other => panic!("expected the refusal, got {other:?}"),
    }
}

/// A session's first prompt, waited for: the engine thread sends it.
fn first_prompt(session: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(60);
    while session.state.prompt.is_none() {
        assert!(Instant::now() < deadline, "no prompt in a minute");
        std::thread::sleep(Duration::from_millis(10));
        session.receive();
    }
}

/// Seat 0 holds a spell it can cast, so its first prompt is turn 1's.
const BOLT_IN_HAND: &str = "hand 0: Lightning Bolt
library 0: Mountain | x5
library 1: Mountain | x5
                            battlefield: Mountain | controller 0
player 0: life 13
";

/// Reload reads the file again, so an edit shows with no relaunch.
#[test]
fn reload_builds_the_game_again_from_the_file_as_it_now_reads() {
    let path = std::env::temp_dir().join("devgui-session-reload.scenario");
    std::fs::write(&path, BOLT_IN_HAND).unwrap();
    let mut session = Session::start(GameSetup { scenario: Some(path.clone()), ..dealt(0, None) }, Arc::new(|| {}));
    first_prompt(&mut session);
    assert_eq!(session.state.board.as_ref().map(|b| b.players[0].life), Some(13));
    std::fs::write(&path, BOLT_IN_HAND.replace("life 13", "life 7")).unwrap();
    session.input(Input::Reload);
    assert!(session.state.board.is_none(), "the old game's board is gone");
    first_prompt(&mut session);
    assert_eq!(session.state.board.as_ref().map(|b| b.players[0].life), Some(7));
}

/// A scenario's setup actions play before the window is asked anything:
/// its first prompt is over the stack §5.3's sample builds, and Reload
/// builds it again.
#[test]
fn setup_actions_play_before_the_first_prompt_and_again_on_reload() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../mtgsim/scenarios/bolt-into-giant-growth.scenario");
    let mut session = Session::start(GameSetup { scenario: Some(path), ..dealt(0, None) }, Arc::new(|| {}));
    for _ in 0..2 {
        first_prompt(&mut session);
        let stack: Vec<&str> = session.state.board.iter().flat_map(|board| &board.stack).map(|item| item.name.as_str()).collect();
        assert_eq!(stack, ["Merfolk Thaumaturgist", "Giant Growth", "Lightning Bolt"], "top first");
        assert_eq!(session.state.prompt.as_ref().map(|prompt| prompt.kind.as_str()), Some("PriorityAction"));
        session.input(Input::Reload);
        assert!(session.state.board.is_none(), "the old game's board is gone");
    }
}

/// Reload starts a decision log of its own beside the first, which keeps the
/// record of the game it replaced.
#[test]
fn reload_keeps_the_replaced_games_log_and_starts_its_own() {
    let dir = std::env::temp_dir().join("devgui-session-reload-log");
    let _ = std::fs::remove_dir_all(&dir);
    let path = std::env::temp_dir().join("devgui-session-reload-log.scenario");
    std::fs::write(&path, BOLT_IN_HAND).unwrap();
    let first = dir.join("bolt-seed-0.log");
    let setup = GameSetup { scenario: Some(path), ..dealt(0, Some(first.clone())) };
    let mut session = Session::start(setup, Arc::new(|| {}));
    first_prompt(&mut session);
    session.input(Input::Reload);
    first_prompt(&mut session);
    let second = dir.join("bolt-seed-0-2.log");
    assert_eq!(session.log_path.as_ref(), Some(&second));
    for log in [&first, &second] {
        let text = std::fs::read_to_string(log).unwrap();
        assert!(text.starts_with("scenario ") && text.contains("end scenario text"), "{}: {text}", log.display());
    }
}

/// "Save board as scenario" writes beside the decision log, a file that
/// loads, and a second save never overwrites the first.
#[test]
fn save_board_writes_a_file_that_loads_beside_the_log() {
    let dir = std::env::temp_dir().join("devgui-session-save");
    let _ = std::fs::remove_dir_all(&dir);
    let path = std::env::temp_dir().join("devgui-session-save.scenario");
    std::fs::write(&path, BOLT_IN_HAND).unwrap();
    let setup = GameSetup { scenario: Some(path), ..dealt(0, Some(dir.join("bolt-seed-0.log"))) };
    let mut session = Session::start(setup, Arc::new(|| {}));
    first_prompt(&mut session);
    session.input(Input::SaveBoard);
    let saved = dir.join("bolt-seed-0-turn-1.scenario");
    assert_eq!(session.saved, Some(Ok(format!("saved {}", saved.display()))));
    let text = std::fs::read_to_string(&saved).unwrap();
    let game = Scenario::parse(&text).and_then(|s| s.build(&CardRegistry::default_registry())).unwrap().game;
    assert_eq!(game.state.players[0].life_total, 13);
    session.input(Input::SaveBoard);
    assert!(dir.join("bolt-seed-0-turn-1-2.scenario").exists());
}
