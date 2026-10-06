//! The bridge with no window: another thread plays the window's part over the
//! channels.

#[path = "support/games.rs"]
mod games;
#[path = "support/sessions.rs"]
mod sessions;
#[path = "support/window_by_rule.rs"]
mod window_by_rule;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use std::time::Duration;

use devgui::boards::Folders;
use devgui::bridge::{EngineHandle, GameSetup, Outcome, Play, Record, ToWindow, Writer};
use devgui::editor::{BoardNumber, EditorInput, Flag, Source, Zone};
use devgui::launch::{self, Start};
use devgui::save::{self, Save};
use devgui::session::Session;
use devgui::view_model::{Input, Mode, Refusal, WindowState};
use mtgsim::cards::registry::CardRegistry;
use mtgsim::scenario::Scenario;
use mtgsim::state::decision_log::{self, GameStart, Log};
use devgui::prompt::{Answer, Primitive, Reply};
use games::{dealt, from_board, next, play, spawn};
use sessions::{BOLT_IN_HAND, next_prompt, scenario_game, session_with};
use window_by_rule::{inputs_by_rule, play_by_rule};

/// `setup`'s game, recorded in a decision log at `log` and a save beside it.
fn recorded(setup: GameSetup, log: PathBuf) -> Play {
    let start = setup.start().unwrap_or_else(|refusal| panic!("{refusal}"));
    let record = Record::create(Save::new(start.clone()), log).unwrap_or_else(|refusal| panic!("{refusal}"));
    Play { record: Some(Writer::take_over(&Arc::new(Mutex::new(record)))), ..Play::new(start) }
}

/// The save beside `log` holds the log's line, the window asked once at
/// each of `answered` places, and nothing else.
fn saved_as_logged(log: &Path, answered: usize) -> Log {
    let logged = decision_log::read(&std::fs::read_to_string(log).expect("the decision log")).expect("the engine reads the log");
    let saved = Save::read(&std::fs::read_to_string(save::path_for(log)).expect("the save")).expect("the save reads");
    assert_eq!((&saved.start, saved.line_to(saved.current())), (&logged.start, logged.answers.clone()));
    let asked = saved.text().lines().filter(|line| line.starts_with("window asked at ")).count();
    assert_eq!(asked, answered, "a place each prompt the window answered");
    logged
}

/// Seed 6's game, played by rule at both seats, ends in 33 turns. Seed 7's,
/// this test's until the window played every seat, ran 94 turns, 14 s in
/// debug: the rule spends its mana on activated abilities before creatures.
#[test]
fn a_whole_game_finishes_with_a_thread_playing_the_window() {
    let log = std::env::temp_dir().join("devgui-headless-seed-6.log");
    let window = std::thread::spawn({
        let log = log.clone();
        move || {
            let mut asked = Vec::new();
            let played = play_by_rule(recorded(dealt(6), log), |state| asked.extend(state.prompt.as_ref().map(|p| p.player)));
            (played, asked)
        }
    });
    let ((outcome, answered), asked) = window.join().expect("the window's thread panicked");
    assert!(matches!(outcome, Outcome::Won(_) | Outcome::Draw), "{outcome:?}");
    assert!(answered > 20, "a game of Magic asks more than {answered} questions");
    assert!(asked.contains(&0) && asked.contains(&1), "the window plays every seat");

    let log = saved_as_logged(&log, answered);
    assert!(matches!(&log.start, GameStart::Dealt { seed: 6, decks, .. } if decks.len() == 2), "{:?}", log.start);
    assert_eq!(log.answers[0].turn, 1, "each answer says when");
    assert!(log.answers.len() > answered, "the log holds the window's answers, its decorators' and the engine's passes");
    assert!(log.answers.iter().any(|a| a.player == 0) && log.answers.iter().any(|a| a.player == 1), "for each seat");
    assert!(log.answers.iter().any(|a| a.forced), "a question with one legal answer is marked");
    let ended = match outcome {
        Outcome::Won(player) => decision_log::Outcome::Won(player),
        Outcome::Draw => decision_log::Outcome::Draw,
        Outcome::Error(error) => decision_log::Outcome::Error(error),
    };
    assert_eq!(log.outcome, Some(ended));
}

/// CR 103.8a: seat 0 plays first in a two-player game and skips its first draw
/// step, so the window's first prompt, seat 0's in turn 1's main phase, shows
/// the seven cards it kept.
#[test]
fn the_window_starts_holding_seven_since_its_first_draw_is_skipped() {
    let engine = spawn(dealt(7));
    let ToWindow::Prompt { snapshot, prompt, .. } = next(&engine) else {
        panic!("expected a prompt first");
    };
    assert_eq!(prompt.player, 0);
    assert_eq!((snapshot.turn, snapshot.active_player, snapshot.phase.as_str()), (1, 0, "Precombat Main"));
    assert_eq!(snapshot.players[0].hand.len(), 7);
    assert_eq!(snapshot.players[0].library.len(), 53);
}

/// Four seats dealt as `fuzz_games --players 4` deals them. In a game of
/// more than two nobody skips the first draw (CR 103.8c), so the first prompt
/// shows seat 0 holding eight. Played by rule until the window has been asked
/// at every seat, not to the end: by rule, four seats play a long game.
#[test]
fn a_four_seat_game_deals_four_hands_and_asks_the_window_at_every_seat() {
    let engine = spawn(GameSetup { players: 4, ..dealt(6) });
    let mut state = WindowState::default();
    state.receive(next(&engine));
    let board = state.board.as_ref().expect("a board at the first prompt");
    let held: Vec<(usize, usize)> = board.players.iter().map(|p| (p.hand.len(), p.library.len())).collect();
    assert_eq!(held, [(8, 52), (7, 53), (7, 53), (7, 53)]);
    let mut asked = BTreeSet::new();
    while let Some(prompt) = &state.prompt {
        asked.insert(prompt.player);
        if asked.len() == 4 {
            break;
        }
        let reply = inputs_by_rule(&state).into_iter().find_map(|input| state.input(input)).expect("the rule answers");
        engine.answers.send(reply).expect("the engine hung up with a prompt open");
        state.receive(next(&engine));
    }
    assert_eq!(asked, BTreeSet::from([0, 1, 2, 3]), "{:?}", state.status());
    let EngineHandle { answers, thread, .. } = engine;
    drop(answers);
    thread.join().expect("a game whose window has gone ends without panicking");
}

/// The four-seat Commander sample, played by rule to its end. Player 1 has
/// left the game (CR 800.4a), so the window is never asked at that seat.
#[test]
fn the_four_seat_sample_plays_to_its_end_and_never_asks_the_seat_that_left() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../mtgsim/scenarios/four-seats-commander.scenario");
    let mut asked = BTreeSet::new();
    let setup = GameSetup { scenario: Some(path), ..dealt(0) };
    let (outcome, _) = play_by_rule(play(setup), |state| asked.extend(state.prompt.as_ref().map(|p| p.player)));
    assert!(matches!(outcome, Outcome::Won(_) | Outcome::Draw), "{outcome:?}");
    assert_eq!(asked, BTreeSet::from([0, 2, 3]));
}

#[test]
fn an_illegal_answer_reaches_the_window_as_the_validators_message() {
    let engine = spawn(dealt(7));
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
    let engine = spawn(from_board("reask.scenario"));
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

/// A game from a scenario: the log's start is the file, its seed and its
/// text verbatim, so it is a save even after the file changes, and the game
/// plays from the board to its end.
#[test]
fn a_whole_game_from_a_scenario_logs_the_file_it_began_from() {
    let log = std::env::temp_dir().join("devgui-headless-scenario.log");
    let setup = from_board("main.scenario");
    let board = std::fs::read_to_string(setup.scenario.as_ref().unwrap()).unwrap();
    let (outcome, answered) = play_by_rule(recorded(setup, log.clone()), |_| {});
    assert!(matches!(outcome, Outcome::Won(_) | Outcome::Draw), "{outcome:?}");
    let log = saved_as_logged(&log, answered);
    let GameStart::Scenario { path, seed: 0, text } = &log.start else { panic!("{:?}", log.start) };
    assert!(path.ends_with("main.scenario"), "{path}");
    assert!(text.lines().eq(board.lines()), "the board's text, verbatim");
    assert!(log.answers.len() > answered, "every answer the window gave is in the log, beside the engine's own passes");
}

/// A file the loader refuses reaches the window as its line and its fix.
#[test]
fn a_refused_scenario_reaches_the_window_with_its_line() {
    let path = std::env::temp_dir().join("devgui-refused.scenario");
    std::fs::write(&path, "turn 2
hand 0: Grizly Bears
").unwrap();
    let engine = spawn(GameSetup { scenario: Some(path), ..dealt(0) });
    match next(&engine) {
        ToWindow::Refused { message, .. } => assert!(message.starts_with("line 2: Grizly Bears is not registered"), "{message}"),
        other => panic!("expected the refusal, got {other:?}"),
    }
}

/// A setup line the engine cannot play reaches the window as a refusal
/// naming its line, as a line the loader refuses does: the setup driver
/// stops the run, where it once panicked.
#[test]
fn a_setup_line_refused_in_play_reaches_the_window_with_its_line() {
    let path = std::env::temp_dir().join("devgui-refused-in-play.scenario");
    std::fs::write(&path, "hand 0: Lightning Bolt
battlefield: Grizzly Bears | controller 1
then: player 0 casts Lightning Bolt | targeting Grizzly Bears
").unwrap();
    let engine = spawn(GameSetup { scenario: Some(path), ..dealt(0) });
    match next(&engine) {
        ToWindow::Refused { message, .. } => assert!(message.starts_with("line 3, `"), "{message}"),
        other => panic!("expected the refusal, got {other:?}"),
    }
}

/// Reload reads the file again, so an edit shows with no relaunch.
#[test]
fn reload_builds_the_game_again_from_the_file_as_it_now_reads() {
    let (mut session, _, file) = session_with("devgui-session-reload", BOLT_IN_HAND, scenario_game);
    next_prompt(&mut session);
    assert_eq!(session.state.board.as_ref().map(|b| b.players[0].life), Some(13));
    std::fs::write(&file, BOLT_IN_HAND.replace("life 13", "life 7")).unwrap();
    session.input(Input::Reload);
    assert!(session.state.board.is_none(), "the old game's board is gone");
    next_prompt(&mut session);
    assert_eq!(session.state.board.as_ref().map(|b| b.players[0].life), Some(7));
}

/// Item 200: a scenario launched with no `--seed` plays at its file's seed
/// as each game starts, so a file fixed after it was refused, and a `seed`
/// line edited, play at the seed they now say, each log named for it.
#[test]
fn reload_plays_at_the_seed_the_file_now_says() {
    let launched = |file: &Path| launch::read(&["--scenario".to_string(), file.display().to_string()], || 0).unwrap();
    let (mut session, _, file) = session_with("devgui-session-seed", "seed 5\nturn two\n", launched);
    while session.state.refused.is_none() {
        std::thread::sleep(Duration::from_millis(10));
        session.receive();
    }
    let seed_played = |session: &Session| {
        let log = session.log_path.as_ref().expect("a game's decision log");
        match decision_log::read(&std::fs::read_to_string(log).unwrap()).unwrap().start {
            GameStart::Scenario { seed, .. } => (seed, log.file_name().unwrap().to_string_lossy().into_owned()),
            other => panic!("{other:?}"),
        }
    };
    for seed in [5, 9] {
        std::fs::write(&file, format!("seed {seed}\n{BOLT_IN_HAND}")).unwrap();
        session.input(Input::Reload);
        next_prompt(&mut session);
        assert_eq!(seed_played(&session), (seed, format!("seed-{seed}.log")));
    }
}

/// A scenario's setup actions play before the window is asked anything:
/// its first prompt is over the stack §5.3's sample builds, and Reload
/// builds it again.
#[test]
fn setup_actions_play_before_the_first_prompt_and_again_on_reload() {
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("../mtgsim/scenarios/bolt-into-giant-growth.scenario");
    let (mut session, ..) = session_with("devgui-session-setup", "", |_| scenario_game(&sample));
    for _ in 0..2 {
        next_prompt(&mut session);
        let stack: Vec<&str> = session.state.board.iter().flat_map(|board| &board.stack).map(|item| item.name.as_str()).collect();
        assert_eq!(stack, ["Merfolk Thaumaturgist", "Giant Growth", "Lightning Bolt"], "top first");
        assert_eq!(session.state.prompt.as_ref().map(|prompt| prompt.kind.as_str()), Some("PriorityAction"));
        session.input(Input::Reload);
        assert!(session.state.board.is_none(), "the old game's board is gone");
    }
}

/// Item 200: a decision log whose folder cannot be made is refused in the
/// window, not shown as an engine panic.
#[test]
fn a_decision_log_that_cannot_be_made_is_refused_in_the_window() {
    let root = std::env::temp_dir().join("devgui-session-unwritable");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    // A file where the logs' folder would go, so no folder can be made there.
    std::fs::write(root.join("logs"), "").unwrap();
    let folders = Folders { logs: root.join("logs").join("logs"), boards: root.join("boards"), ..Folders::default() };
    let session = Session::start(Start::Game(dealt(7)), folders, Arc::new(|| {}));
    let Some((Refusal::Record, message)) = &session.state.refused else { panic!("{}", session.state.status()) };
    let log = root.join("logs").join("logs").join("seed-7.log");
    assert!(message.starts_with(&format!("cannot create the decision log {}: ", log.display())), "{message}");
    assert_eq!((session.state.panic.as_ref(), session.log_path.as_ref()), (None, None), "no game started");
    assert_eq!(session.state.no_board(), "No board: the decision log could not be made.");
}

/// Reload starts a decision log of its own beside the first, which keeps the
/// record of the game it replaced.
#[test]
fn reload_keeps_the_replaced_games_log_and_starts_its_own() {
    let (mut session, folders, _) = session_with("devgui-session-reload-log", BOLT_IN_HAND, scenario_game);
    let dir = folders.boards.join("devgui-session-reload-log");
    next_prompt(&mut session);
    session.input(Input::Reload);
    next_prompt(&mut session);
    assert_eq!(session.log_path.as_ref(), Some(&dir.join("seed-0-2.log")));
    for log in [dir.join("seed-0.log"), dir.join("seed-0-2.log")] {
        let read = decision_log::read(&std::fs::read_to_string(&log).unwrap());
        assert!(matches!(read, Ok(Log { start: GameStart::Scenario { .. }, .. })), "{}: {read:?}", log.display());
    }
}

/// "Save board as scenario" makes a board of its own, named for the game's
/// start and turn, its first line naming the log; it loads, and a second
/// save never overwrites the first.
#[test]
fn save_board_as_scenario_makes_a_board_folder_named_for_the_start_and_turn() {
    let (mut session, folders, _) = session_with("devgui-session-save", BOLT_IN_HAND, scenario_game);
    next_prompt(&mut session);
    session.input(Input::SaveBoard);
    let saved = folders.boards.join("devgui-session-save-turn-1").join("devgui-session-save-turn-1.scenario");
    assert_eq!(session.message, Some(Ok(format!("saved {}", saved.display()))));
    let text = std::fs::read_to_string(&saved).unwrap();
    let log = folders.boards.join("devgui-session-save").join("seed-0.log");
    assert!(text.starts_with(&format!("# Saved from {}, turn 1.\n", log.display())), "{text}");
    let game = Scenario::parse(&text).and_then(|s| s.build(&CardRegistry::default_registry())).unwrap().game;
    assert_eq!(game.state.players[0].life_total, 13);
    session.input(Input::SaveBoard);
    assert!(folders.boards.join("devgui-session-save-turn-1-2").exists());
}

/// The editor's Play saves the board into a folder of its own and starts it
/// from that file, its decision log beside it; the next Play saves over the
/// same file and logs beside the first. The file it was opened from is never
/// written.
#[test]
fn play_saves_the_board_in_its_folder_and_logs_its_games_beside_it() {
    let (mut session, folders, file) = session_with("devgui-session-play", BOLT_IN_HAND, |file| Start::Edit(Some(file.to_path_buf())));
    assert_eq!(session.mode, Mode::Edit);
    session.input(Input::Editor(EditorInput::Number(BoardNumber::Life(0), 9)));
    session.input(Input::Editor(EditorInput::Play));
    let board = folders.boards.join("devgui-session-play");
    assert_eq!(session.setup.as_ref().and_then(|setup| setup.scenario.clone()), Some(board.join("devgui-session-play.scenario")));
    assert_eq!((session.mode, session.log_path.clone()), (Mode::Play, Some(board.join("seed-0.log"))));
    next_prompt(&mut session);
    assert_eq!(session.state.board.as_ref().map(|b| b.players[0].life), Some(9));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), BOLT_IN_HAND, "the file opened is never written");
    session.input(Input::Mode(Mode::Edit));
    session.input(Input::Editor(EditorInput::Number(BoardNumber::Life(0), 8)));
    session.input(Input::Editor(EditorInput::Play));
    assert_eq!(session.log_path, Some(board.join("seed-0-2.log")));
    assert!(std::fs::read_to_string(board.join("devgui-session-play.scenario")).unwrap().contains("player 0: life 8"));
    assert!(session.files.iter().any(|listed| listed.label == "boards/devgui-session-play"));
}

/// "Edit this board" opens the board the game is at, the writer's report
/// above it, the advanced settings as they were, and its first save names
/// it for the game's start and turn.
#[test]
fn edit_this_board_opens_the_board_the_game_is_at() {
    let (mut session, folders, _) = session_with("devgui-session-edit-board", BOLT_IN_HAND, scenario_game);
    next_prompt(&mut session);
    session.input(Input::Editor(EditorInput::Advanced(true)));
    session.input(Input::EditThisBoard);
    assert!(session.editor.advanced, "the advanced settings stay on");
    assert_eq!(session.mode, Mode::Edit);
    assert_eq!(session.editor.source, Source::Game { start: "devgui-session-edit-board".to_string(), turn: 1 });
    assert!(session.editor.comments[0].starts_with("# From "), "{:?}", session.editor.comments);
    assert!(session.editor.text().contains("hand 0: Lightning Bolt\n") && session.editor.text().contains("player 0: life 13\n"));
    session.input(Input::Editor(EditorInput::Save));
    let saved = folders.boards.join("devgui-session-edit-board-turn-1").join("devgui-session-edit-board-turn-1.scenario");
    assert_eq!(session.editor.source, Source::Board(saved));
}

/// "Edit the scenario" reads the file again; once the editor's Play has
/// started the game, it goes back to the editor's board, undo and all.
#[test]
fn edit_the_scenario_reads_the_file_again_or_returns_to_the_board_play_started() {
    let (mut session, _, file) = session_with("devgui-session-edit-scenario", BOLT_IN_HAND, scenario_game);
    next_prompt(&mut session);
    std::fs::write(&file, BOLT_IN_HAND.replace("life 13", "life 6")).unwrap();
    session.input(Input::EditTheScenario);
    assert_eq!(session.editor.source, Source::File(file.clone()));
    assert!(session.editor.text().contains("player 0: life 6\n"), "read again: {}", session.editor.text());
    session.input(Input::Editor(EditorInput::Number(BoardNumber::Life(0), 5)));
    session.input(Input::Editor(EditorInput::Play));
    next_prompt(&mut session);
    session.input(Input::EditTheScenario);
    assert_eq!(session.mode, Mode::Edit);
    assert!(session.editor.view().undo.live, "the editor's own board, undo and all");
}

/// The PR's click script: a four-seat Commander board built in the editor
/// from an empty one, and played to its first question. Player 2's Isamaru
/// attacks Player 3, whose Wall of Stone can block it; Player 0's commander
/// shares Isamaru's name, so each takes a tag.
#[test]
fn a_four_seat_commander_board_built_in_the_editor_plays() {
    let (mut session, ..) = session_with("devgui-session-commander", "", |_| Start::Edit(None));
    for fact in [(BoardNumber::Players, 4), (BoardNumber::StartingLife, 40), (BoardNumber::Turn, 6)] {
        click(&mut session, EditorInput::Number(fact.0, fact.1));
    }
    click(&mut session, EditorInput::Active(2));
    let steps = session.editor.view().steps;
    let attackers = steps.into_iter().find(|step| step.label == "declare attackers").unwrap();
    click(&mut session, attackers.input);
    let put = |session: &mut Session, name: &str, seat, zone| {
        click(session, EditorInput::Search(name.to_string()));
        let named = (0..).find(|&i| session.editor.search().name(i) == Some(name)).unwrap();
        click(session, EditorInput::Choose(named));
        click(session, EditorInput::Put(seat, zone));
        session.editor.editing.unwrap()
    };
    let isamaru = "Isamaru, Hound of Konda";
    for (seat, commander) in [(0, isamaru), (1, "Thalia, Guardian of Thraben")] {
        let card = put(&mut session, commander, seat, Zone::Command);
        click(&mut session, EditorInput::Flag(card, Flag::Commander, true));
    }
    let attacker = put(&mut session, isamaru, 2, Zone::Battlefield);
    for edit in [EditorInput::Flag(attacker, Flag::Commander, true), EditorInput::Flag(attacker, Flag::Tapped, true), EditorInput::AttackPlayer(attacker, 3)] {
        click(&mut session, edit);
    }
    put(&mut session, "Wall of Stone", 3, Zone::Battlefield);
    for seat in 0..4 {
        put(&mut session, "Plains", seat, Zone::Library);
    }
    let text = session.editor.text().to_string();
    assert!(text.contains("command: Isamaru, Hound of Konda [a] | owner 0, commander\n"), "{text}");
    assert!(text.contains("battlefield: Isamaru, Hound of Konda [b] | controller 2, commander, tapped, attacking player 3\n"), "{text}");
    assert!(session.editor.refusal().is_none(), "{:?}", session.editor.refusal());
    click(&mut session, EditorInput::Play);
    next_prompt(&mut session);
    let prompt = session.state.prompt.as_ref().unwrap();
    assert_eq!((prompt.player, prompt.kind.as_str()), (3, "DeclareBlockers"));
    let board = session.state.board.as_ref().unwrap();
    assert_eq!(board.players.iter().map(|player| player.life).collect::<Vec<_>>(), [40; 4]);
}

fn click(session: &mut Session, input: EditorInput) {
    session.input(Input::Editor(input));
}
