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
use devgui::view_model::{Input, NO_EARLIER_QUESTION, Refusal};
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
    let engine = spawn_game(Play { line, audited: true, record: Some(writer), ..Play::new(start) }, Arc::new(|| {}));
    while replayed.load(Ordering::Relaxed) == 0 {
        std::thread::yield_now();
    }
    Writer::take_over(&record);
    let sent = engine.from_engine.recv_timeout(Duration::from_secs(60));
    assert!(matches!(sent, Err(RecvTimeoutError::Disconnected)), "the superseded replay sent {sent:?}");
}

/// The menu's entries as the header shows them: each label, and whether a
/// click on it does anything.
fn menu(session: &Session) -> Vec<(String, bool)> {
    session.state.tools_view().menu.into_iter().map(|entry| (entry.label, entry.live)).collect()
}

/// Savestate marks the open question, once a place, named for its turn and
/// step; the menu moves back to it, and then back to where the window was.
#[test]
fn a_savestate_marks_the_open_question_and_the_menu_moves_back_to_it() {
    let (mut session, ..) = session_with("devgui-tools-savestate", "", |_| Start::Game(dealt(6)));
    next_prompt(&mut session);
    for _ in 0..3 {
        answer_by_rule(&mut session);
    }
    let marked = question(&session);
    assert!(session.state.tools_view().savestate.live && menu(&session).is_empty());
    session.input(Input::Savestate);
    let board = session.state.board.as_ref().unwrap();
    let name = format!("Turn {} · {}", board.turn, board.phase);
    assert!(!session.state.tools_view().savestate.live, "one savestate a place");
    assert_eq!(menu(&session), [(name.clone(), false)], "listed, and the window is at it");
    for _ in 0..4 {
        answer_by_rule(&mut session);
    }
    let left = question(&session);
    assert_eq!(menu(&session), [(name.clone(), true)]);
    session.input(session.state.tools_view().menu[0].input.clone());
    assert!(!session.state.tools_view().savestate.live, "no question open while it replays");
    next_prompt(&mut session);
    assert_eq!(question(&session), marked);
    assert_eq!(menu(&session), [(name, false), ("Back to where I was".to_string(), true)]);
    session.input(session.state.tools_view().menu[1].input.clone());
    next_prompt(&mut session);
    assert_eq!(question(&session), left);
}

/// A session on the file `--load` reads, writing under the folder its own
/// test made; what it read, as a load must leave it.
fn loaded(file: &std::path::Path) -> Session {
    let folders = devgui::boards::Folders { logs: file.with_file_name("logs"), boards: file.with_file_name("boards"), ..Default::default() };
    Session::start(Start::Load(file.to_path_buf()), folders, Arc::new(|| {}))
}

/// The Bolt board played to the question after Bolt hit player 1, then
/// undone to Bolt's target and branched onto player 0, with a savestate at
/// the first question: the save holds two lines, three places the window
/// was asked on each, the savestate, and the line left. Each question
/// passed, in order.
fn branched_bolt_game(name: &str) -> (Session, [(String, String, usize); 4]) {
    let (mut session, ..) = session_with(name, BOLT_IN_HAND, scenario_game);
    next_prompt(&mut session);
    let first = question(&session);
    session.input(Input::Savestate);
    answer(&mut session, "Cast Lightning Bolt");
    let target = question(&session);
    answer(&mut session, "Player 1");
    let hit_player_1 = question(&session);
    session.input(Input::Undo);
    next_prompt(&mut session);
    answer(&mut session, "Player 0");
    let hit_player_0 = question(&session);
    (session, [first, target, hit_player_1, hit_player_0])
}

/// A savestate and a branch survive a save and a load: the load replays the
/// line the save ended on and asks its next question, Undo walks back into
/// the save, and the menu moves to the savestate and back. Play goes on in a
/// new pair beside the loaded one, which is never written.
#[test]
fn a_savestate_and_a_branch_survive_a_save_and_a_load() {
    let (session, [first, target, _, hit_player_0]) = branched_bolt_game("devgui-tools-load");
    let log = session.log_path.clone().unwrap();
    let saved = save::path_for(&log);
    let before = (std::fs::read(&log).unwrap(), std::fs::read(&saved).unwrap());
    drop(session);

    let mut session = loaded(&saved);
    assert!(session.state.replaying.is_some(), "the load counts its replay");
    assert!(session.start_line.starts_with(&format!("loaded {} · scenario ", saved.display())), "{}", session.start_line);
    next_prompt(&mut session);
    assert_eq!(question(&session), hit_player_0, "the line the save ended on");
    let next_log = log.with_file_name(format!("{}-2.log", log.file_stem().unwrap().to_string_lossy()));
    assert_eq!(session.log_path.as_ref(), Some(&next_log), "a new pair beside the loaded one");
    assert_eq!(menu(&session), [("Turn 1 · Precombat Main".to_string(), true), ("Back to where I was".to_string(), true)]);
    session.input(Input::Undo);
    next_prompt(&mut session);
    assert_eq!(question(&session), target, "Undo walks back into the save");
    session.input(session.state.tools_view().menu[0].input.clone());
    next_prompt(&mut session);
    assert_eq!(question(&session), first, "the savestate");
    session.input(session.state.tools_view().menu[1].input.clone());
    next_prompt(&mut session);
    assert_eq!(question(&session), hit_player_0, "back to where the load began");
    drop(session);
    assert_eq!((std::fs::read(&log).unwrap(), std::fs::read(&saved).unwrap()), before, "the loaded files, untouched");
    let resaved = Save::read(&std::fs::read_to_string(save::path_for(&next_log)).unwrap()).unwrap();
    let reread = Save::read(&String::from_utf8(before.1).unwrap()).unwrap();
    assert_eq!(resaved.line_to(resaved.current()), reread.line_to(reread.current()), "the new save, holding the loaded one, back at its end");
}

/// A decision log loads as a save of one line with no record of where the
/// window was asked: it replays to its end, and its undo starts with the
/// first answer given after loading.
#[test]
fn a_decision_log_loads_as_one_line_and_its_undo_starts_after_it() {
    let (session, [.., hit_player_0]) = branched_bolt_game("devgui-tools-load-log");
    let log = session.log_path.clone().unwrap();
    drop(session);
    let mut session = loaded(&log);
    next_prompt(&mut session);
    assert_eq!(question(&session), hit_player_0, "the log is the line the window was on");
    assert!(menu(&session).is_empty() && !session.state.tools_view().undo.live, "nothing recorded before it");
    answer_by_rule(&mut session);
    session.input(Input::Undo);
    next_prompt(&mut session);
    assert_eq!(question(&session), hit_player_0, "the first question asked after loading");
}

/// A loaded line this build no longer takes stops at that answer and says
/// where; the game plays on from the answer before it, asking its question.
#[test]
fn a_load_that_diverges_says_where_and_plays_on_from_the_answer_before() {
    let (session, [_, target, ..]) = branched_bolt_game("devgui-tools-diverge");
    let log = session.log_path.clone().unwrap();
    drop(session);
    let text = std::fs::read_to_string(&log).unwrap();
    let changed = text.replace("SelectRecipients picks player 0", "SelectRecipients picks player 7");
    assert_ne!(changed, text, "the log names Bolt's target");
    let altered = log.with_file_name("altered.log");
    std::fs::write(&altered, changed).unwrap();
    let mut session = loaded(&altered);
    next_prompt(&mut session);
    let diverged = session.state.diverged.clone().expect("the divergence, said");
    assert!(diverged.contains("it chose player 7, which is not offered"), "{diverged}");
    assert_eq!(question(&session), target, "Bolt's target asked again");
}

/// A file `--load` cannot read as a record is refused in the window, naming
/// its line, as a scenario that does not load is.
#[test]
fn a_load_that_does_not_read_is_refused_in_the_window() {
    let (_, _, file) = session_with("devgui-tools-unreadable", "turn 2
", |_| Start::Edit(None));
    let session = loaded(&file);
    let Some((Refusal::Load, message)) = &session.state.refused else { panic!("{}", session.state.status()) };
    assert!(message.starts_with(&format!("{}, line 1: ", file.display())), "{message}");
    assert_eq!(session.state.status(), "The save did not load");
}
