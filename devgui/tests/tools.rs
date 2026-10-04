//! The window's tools with no window (`setup-architecture.md` §7.1–§7.3):
//! Undo answer, savestates and their menu, the save beside each log, and
//! `--load`. Each plays a short board through the session, as clicks do.

#[path = "support/games.rs"]
#[allow(dead_code, reason = "these tests drive a session; the bridge-level helpers are the other files'")]
mod games;
#[path = "support/sessions.rs"]
mod sessions;
#[path = "support/window_by_rule.rs"]
#[allow(dead_code, reason = "these tests answer by the rule one question at a time, through a session")]
mod window_by_rule;

use std::sync::atomic::Ordering;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use devgui::bridge::{Play, Record, Writer, locked, spawn_game};
use devgui::launch::Start;
use devgui::save::{self, Save};
use devgui::session::Session;
use devgui::view_model::{Input, NO_EARLIER_QUESTION};
use games::{dealt, play};
use sessions::{BOLT_IN_HAND, next_prompt, scenario_game, session_with};
use window_by_rule::{inputs_by_rule, play_by_rule};

/// The open question as a test compares it: the board as `Scenario::write`
/// writes it, the question's kind, and the seat it asks.
fn question(session: &Session) -> (String, String, usize) {
    let (Some(board), Some(prompt)) = (&session.state.board, &session.state.prompt) else {
        panic!("no open question: {}", session.state.status());
    };
    (board.board_text.clone(), prompt.kind.clone(), prompt.player)
}

/// The open question answered with the option whose label holds `label`,
/// and the next one waited for.
fn answer(session: &mut Session, label: &str) {
    let prompt = session.state.prompt.clone().expect("an open question");
    let at = prompt.options.iter().position(|option| option.label.contains(label));
    let at = at.unwrap_or_else(|| panic!("no {label} among {:?}", prompt.options));
    session.input(Input::OptionButton(at));
    assert!(session.state.prompt.is_none(), "{label} answered {}", prompt.kind);
    next_prompt(session);
}

/// The open question answered by `window_by_rule`'s rule, and the next one
/// waited for.
fn answer_by_rule(session: &mut Session) {
    for input in inputs_by_rule(&session.state) {
        session.input(input);
        if session.state.prompt.is_none() {
            return next_prompt(session);
        }
    }
    panic!("the rule built no answer for {:?}", session.state.prompt);
}

/// Undo answer asks the window's previous question again, on the board a
/// game played straight to it shows, across a target choice; at the first
/// question there is nothing to undo, and the button says so.
#[test]
fn undo_asks_the_previous_question_again_on_the_board_a_straight_game_shows() {
    let (mut straight, ..) = session_with("devgui-tools-straight", BOLT_IN_HAND, scenario_game);
    next_prompt(&mut straight);
    let cast = question(&straight);
    answer(&mut straight, "Cast Lightning Bolt");
    let target = question(&straight);
    assert_eq!(target.1, "SelectRecipients");

    let (mut session, ..) = session_with("devgui-tools-undo", BOLT_IN_HAND, scenario_game);
    next_prompt(&mut session);
    let off = session.state.tools_view();
    assert_eq!((off.undo.live, off.undo_off), (false, Some(NO_EARLIER_QUESTION)), "the first question");
    answer(&mut session, "Cast Lightning Bolt");
    answer(&mut session, "Player 1");
    let played_on = question(&session);
    assert!(session.state.tools_view().undo.live);
    session.input(Input::Undo);
    assert!(session.state.board.is_none() && session.state.replaying.is_some(), "a replay, counted");
    next_prompt(&mut session);
    assert_eq!(question(&session), target, "the target asked again, on the straight game's board");
    session.input(Input::Undo);
    next_prompt(&mut session);
    assert_eq!(question(&session), cast);
    assert!(!session.state.tools_view().undo.live, "back at the first question");
    answer(&mut session, "Cast Lightning Bolt");
    answer(&mut session, "Player 1");
    assert_eq!(question(&session), played_on, "the same answers play to the same board");
}

/// While a replay runs Undo answer stays live, each press moving its target
/// back one more question: three presses go back three questions and wait
/// for one replay, the last; the two it supersedes stop at their next
/// answer, which `a_superseded_replay_stops_and_asks_the_window_nothing`
/// shows. The save records each move.
#[test]
fn three_undo_presses_during_a_replay_wait_for_one_replay() {
    let (mut session, ..) = session_with("devgui-tools-three", "", |_| Start::Game(dealt(6)));
    let mut questions = Vec::new();
    next_prompt(&mut session);
    for _ in 0..30 {
        questions.push(question(&session));
        answer_by_rule(&mut session);
    }
    let replays: Vec<_> = (0..3)
        .map(|_| {
            session.input(Input::Undo);
            session.state.replaying.clone().expect("a replay, counted")
        })
        .collect();
    next_prompt(&mut session);
    assert_eq!(question(&session), questions[27], "three questions back");
    let lines: Vec<usize> = replays.iter().map(|replay| replay.of).collect();
    assert!(lines[0] > lines[1] && lines[1] > lines[2], "each press a question further back: {lines:?}");
    let done = (replays[2].done.load(Ordering::Relaxed), replays[2].of);
    assert_eq!(done.0, done.1, "the last replayed its whole line");
    let log = session.log_path.clone().expect("the game's log");
    let saved = std::fs::read_to_string(save::path_for(&log)).unwrap();
    assert_eq!(saved.lines().filter(|line| line.starts_with("moved to ")).count(), 3);
    let read = Save::read(&saved).unwrap();
    assert_eq!(read.line_to(read.current()).len(), done.1, "the save's line is the one replayed");
}

/// A replay superseded on its way stops at its next answer: its thread ends
/// having sent the window nothing, where a replay left to run would reach
/// the end of its line and send the window what the game came to. It is
/// superseded once it has begun, which its writer's count shows, and it
/// keeps the memo's audits on, about 1.5 ms an answer in debug, so it is
/// still on its way.
#[test]
fn a_superseded_replay_stops_and_asks_the_window_nothing() {
    let start = dealt(6).start().unwrap();
    let log = std::env::temp_dir().join("devgui-tools-superseded").join("seed-6.log");
    let record = Arc::new(Mutex::new(Record::create(Save::new(start.clone()), log).unwrap()));
    play_by_rule(Play { record: Some(Writer::take_over(&record)), ..play(dealt(6)) }, |_| {});
    let line = {
        let record = locked(&record);
        record.save.line_to(record.save.current())
    };
    assert!(line.len() > 300, "a whole game's line: {}", line.len());
    let writer = Writer::take_over(&record);
    let replayed = writer.replayed();
    let engine = spawn_game(Play { start, line, audited: true, record: Some(writer) }, Arc::new(|| {}));
    while replayed.load(Ordering::Relaxed) == 0 {
        std::thread::yield_now();
    }
    Writer::take_over(&record);
    let sent = engine.from_engine.recv_timeout(Duration::from_secs(60));
    assert!(matches!(sent, Err(RecvTimeoutError::Disconnected)), "the superseded replay sent {sent:?}");
}
