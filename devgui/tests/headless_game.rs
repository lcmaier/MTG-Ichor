//! The bridge with no window: another thread plays the window's part over the
//! channels.

#[path = "support/window_by_rule.rs"]
mod window_by_rule;

use std::sync::Arc;

use devgui::bridge::{Outcome, ToWindow, spawn_game};
use devgui::prompt::{Answer, Primitive};
use window_by_rule::{dealt, from_board, next, play_by_rule};

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
    let asked = lines.iter().filter(|l| l.starts_with("answer ")).count();
    assert_eq!(asked, answered, "every answer the window gave is in the log");
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
            engine.answers.send(Answer::Picks(vec![99])).unwrap();
        }
        other => panic!("expected a pick as the first prompt, got {other:?}"),
    }
    match next(&engine) {
        ToWindow::Panicked { message } => assert!(message.contains("DP returned index 99"), "{message}"),
        other => panic!("expected the validator's panic, got {other:?}"),
    }
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
    assert_eq!(lines.iter().filter(|l| l.starts_with("answer ")).count(), answered);
}

/// A file the loader refuses reaches the window as its line and its fix.
#[test]
fn a_refused_scenario_reaches_the_window_with_its_line() {
    let path = std::env::temp_dir().join("devgui-refused.scenario");
    std::fs::write(&path, "turn 2
hand 0: Grizly Bears
").unwrap();
    let engine = spawn_game(devgui::bridge::GameSetup { scenario: Some(path), ..dealt(0, None) }, Arc::new(|| {}));
    match next(&engine) {
        ToWindow::Refused { message } => assert!(message.starts_with("line 2: Grizly Bears is not registered"), "{message}"),
        other => panic!("expected the refusal, got {other:?}"),
    }
}
