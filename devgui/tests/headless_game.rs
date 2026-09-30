//! The bridge with no window: another thread plays the window's part over the
//! channels.

#[path = "support/window_by_rule.rs"]
mod window_by_rule;

use std::sync::Arc;

use devgui::bridge::{GameSetup, Outcome, Pool, ToWindow, spawn_game};
use devgui::prompt::{Answer, Primitive};
use window_by_rule::{next, play_by_rule};

#[test]
fn a_whole_game_finishes_with_a_thread_playing_the_window() {
    let log = std::env::temp_dir().join("devgui-headless-seed-7.log");
    let window = std::thread::spawn({
        let log = log.clone();
        move || play_by_rule(7, Some(log), |_| {})
    });
    let (outcome, answered) = window.join().expect("the window's thread panicked");
    assert!(matches!(outcome, Outcome::Won(_) | Outcome::Draw), "{outcome:?}");
    assert!(answered > 20, "a game of Magic asks seat 0 more than {answered} questions");

    let log = std::fs::read_to_string(&log).expect("the decision log");
    let lines: Vec<&str> = log.lines().collect();
    assert_eq!(lines[..2], ["seed 7", "pool Performance"]);
    assert!(lines[2].starts_with("deck 0 ") && lines[3].starts_with("deck 1 "));
    assert!(lines[4].starts_with("answer 1 [turn 1, "), "each answer says when: {}", lines[4]);
    let asked = lines.iter().filter(|l| l.starts_with("answer ") && !l.ends_with(" only")).count();
    assert_eq!(asked, answered, "every answer the window gave is in the log");
    assert_eq!(lines.last(), Some(&format!("outcome {outcome:?}").as_str()));
}

#[test]
fn an_illegal_answer_reaches_the_window_as_the_validators_message() {
    let engine = spawn_game(GameSetup { seed: 7, pool: Pool::Performance, log_path: None }, Arc::new(|| {}));
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
