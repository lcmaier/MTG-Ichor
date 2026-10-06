//! The window drawn offscreen at the boards the review scenarios under
//! `tests/scenarios/` reach, as pictures to review: each board is set up for
//! its prompts, so no seed is hunted for them. `UPDATE_SNAPSHOTS=true cargo
//! test --test screenshots` redraws them. A plain run compares against the
//! committed PNGs, which holds on the machine that drew them: another GPU
//! draws a few pixels differently.

#[path = "support/games.rs"]
mod games;
#[path = "support/window_by_rule.rs"]
mod window_by_rule;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use devgui::app::{SessionHeader, draw};
use devgui::boards::Folders;
use devgui::bridge::{EngineHandle, GameSetup, ToWindow};
use devgui::editor::EditorInput;
use devgui::launch::Start;
use devgui::prompt::{Answer, BoardRef, Reply};
use devgui::save::{Destination, Tools};
use devgui::session::Session;
use devgui::view_model::{Input, Mode, Progress, WindowState};
use egui_kittest::kittest::Queryable;
use egui_kittest::{Harness, SnapshotResult, SnapshotResults};
use games::{from_board, next, play, spawn};

/// Each picture: the first prompt of a kind any review board reaches, with
/// any clicks made before it is drawn. Kept to what a review needs, since
/// each redraw commits every PNG again.
const PICTURES: [(&str, &str); 6] = [
    ("PriorityAction", "priority"),
    ("ManaAbilityWindow", "mana_window"),
    ("SelectRecipients", "targets"),
    ("DeclareAttackers", "attackers"),
    ("DeclareBlockers", "blockers"),
    ("AssignCombatDamage", "combat_damage"),
];

/// The review boards, in the order their prompts are taken.
const BOARDS: [&str; 3] = ["main.scenario", "blocks.scenario", "damage.scenario"];

/// The header's line and decision log for a review board.
fn header(board: &str) -> (String, PathBuf) {
    let stem = board.trim_end_matches(".scenario");
    (format!("scenario tests/scenarios/{board} · seed 0"), PathBuf::from(format!("boards/{stem}/seed-0.log")))
}

#[test]
fn the_window_at_each_kind_of_prompt_the_review_boards_reach() {
    let mut first: BTreeMap<&str, (WindowState, &str)> = BTreeMap::new();
    for board in BOARDS {
        window_by_rule::play_by_rule(play(from_board(board)), |state| {
            let Some(prompt) = &state.prompt else { return };
            let Some((_, name)) = PICTURES.iter().find(|(kind, _)| *kind == prompt.kind) else { return };
            // A priority prompt with a spell in reach, and blocks with two
            // blockers to choose between: with every seat the window's, a lone
            // block's yes or no comes first. Since MA-1's exact check no review
            // board offers two spells at once.
            let worth_it = match prompt.kind.as_str() {
                "PriorityAction" => prompt.options.len() >= 2,
                "DeclareBlockers" => prompt.options.len() >= 2,
                _ => true,
            };
            if worth_it && !first.contains_key(name) {
                first.insert(name, (clicked_once(state), board));
            }
        });
    }
    let mut results = SnapshotResults::new();
    for (name, (state, board)) in &first {
        results.add(picture(&partway(state.clone()), &header(board), None, name));
    }
    results.add(picture(&partway(panicked()), &header("main.scenario"), None, "engine_panic"));
    results.add(picture(&refused(), &header("refused.scenario"), None, "scenario_refused"));
    // The window's first question, after the setup actions: nothing to undo.
    let sample = "../mtgsim/scenarios/bolt-into-giant-growth.scenario";
    let line = (format!("scenario {sample} · seed 0"), PathBuf::from("boards/bolt-into-giant-growth/seed-0.log"));
    results.add(picture(&setup_stack(sample), &line, None, "setup_stack"));
    results.add(picture(&partway(blocks_rejected()), &header("reask.scenario"), None, "blocks_rejected"));
    let (state, board) = &first["priority"];
    let saved = Some(Ok("saved boards/main-turn-3/main-turn-3.scenario"));
    results.add(picture(&partway(state.clone()), &header(board), saved, "board_saved"));
    results.add(picture(&replaying(), &header("main.scenario"), None, "replaying"));
    results.add(menu_picture(&with_savestates(state.clone()), &header(board), "savestates_menu"));
    let (state, board) = &first["targets"];
    let diverged = "the log diverges at answer 12: it chose player 7, which is not offered".to_string();
    results.add(picture(&WindowState { diverged: Some(diverged), ..partway(state.clone()) }, &header(board), None, "diverged"));
    let sample = "../mtgsim/scenarios/humility-opalescence.scenario";
    let line = (format!("scenario {sample} · seed 0"), PathBuf::from("boards/humility-opalescence/seed-0.log"));
    results.add(picture(&partway(why_open(sample)), &line, None, "why_panel"));
    let (blocks, mana) = (edited("blocks.scenario", "", ANGEL), edited("main.scenario", "Everywhere", "Mountain"));
    let line = |board: &str, edit: &str| (format!("scenario tests/scenarios/{board}, {edit} · seed 0"), PathBuf::new());
    let wall = why_on(&blocks, "Wall of Stone");
    results.add(picture(&partway(wall), &line("blocks.scenario", "Serra Angel edited in"), None, "why_blocks"));
    let bears = why_on(&mana, "Grizzly Bears");
    results.add(picture(&partway(bears), &line("main.scenario", "Everywhere edited to a Mountain"), None, "why_mana"));
    let sample = "../mtgsim/scenarios/bolt-into-giant-growth.scenario";
    let line = (format!("scenario {sample} · seed 0"), PathBuf::from("boards/bolt-into-giant-growth/seed-0.log"));
    results.add(picture(&partway(why_event(sample)), &line, None, "why_event"));
    results.add(editor_picture("four-seats-commander.scenario", &["Isamaru, Hound of Konda"], "editor"));
    results.add(editor_picture("holy-strength.scenario", &["precombat main", "Grizzly Bears [a]"], "editor_refused"));
    let missing: Vec<&str> = PICTURES.iter().map(|(_, name)| *name).filter(|name| !first.contains_key(name)).collect();
    assert!(missing.is_empty(), "the review boards no longer reach {missing:?}");
}

/// The tools as a game partway through shows them: a question before this
/// one to go back to, and no savestate yet.
fn partway(state: WindowState) -> WindowState {
    WindowState { tools: Tools { undo_open: true, undo_answered: true, ..Tools::default() }, ..state }
}

/// Two savestates set, and a line left since: what the menu lists.
fn with_savestates(state: WindowState) -> WindowState {
    let destinations = vec![
        Destination::Savestate { at: 4, name: "Turn 2 · Precombat Main".to_string() },
        Destination::Savestate { at: 11, name: "Turn 3 · Declare Blockers".to_string() },
        Destination::Left(19),
    ];
    let state = partway(state);
    WindowState { tools: Tools { destinations, current: 23, ..state.tools.clone() }, ..state }
}

/// A rebuild on its way to a question: no board, the answers counted.
fn replaying() -> WindowState {
    let replaying = Progress { done: Arc::new(AtomicUsize::new(120)), of: 341 };
    partway(WindowState { replaying: Some(replaying), ..WindowState::default() })
}

/// The first click the rule would make, when it does not answer the prompt,
/// so a choice in progress shows.
fn clicked_once(state: &WindowState) -> WindowState {
    let mut after = state.clone();
    if let Some(input) = window_by_rule::inputs_by_rule(state).into_iter().find(|input| *input != Input::Done)
        && after.clone().input(input.clone()).is_none()
    {
        after.input(input);
    }
    after
}

/// What the window shows when the engine panics: an answer the validators refuse.
fn panicked() -> WindowState {
    let engine = spawn(from_board("main.scenario"));
    let mut state = WindowState::default();
    state.receive(next(&engine));
    engine.answers.send(Reply::Answer(Answer::Picks(vec![99]))).unwrap();
    state.receive(next(&engine));
    state
}

/// The window's first prompt on §5.3's sample: the stack its setup actions
/// built, player 0 holding priority over it with a Bolt left to cast.
fn setup_stack(sample: &str) -> WindowState {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let engine = spawn(GameSetup { scenario: Some(path), ..from_board("main.scenario") });
    let mut state = WindowState::default();
    state.receive(next(&engine));
    assert_eq!(state.board.as_ref().map(|board| board.stack.len()), Some(3), "the setup actions' stack");
    finish(engine);
    state
}

/// The why panel open on Serra Angel at the sample's first question: what
/// each layer did to it under Humility, and what reached it and did not
/// apply (`setup-architecture.md` §7c).
fn why_open(sample: &str) -> WindowState {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let engine = spawn(GameSetup { scenario: Some(path), ..from_board("main.scenario") });
    let mut state = WindowState::default();
    state.receive(next(&engine));
    let board = state.board.as_ref().expect("a board at the first question");
    let permanents = board.players.iter().flat_map(|player| &player.battlefield);
    let angel = permanents.map(|p| &p.card).find(|card| card.name == "Serra Angel").map(|card| card.id).expect("the Angel");
    engine.answers.send(state.input(Input::Why(BoardRef::Object(angel))).expect("a why at an open question")).unwrap();
    state.receive(next(&engine));
    assert!(state.why_view().is_some(), "the panel shows");
    finish(engine);
    state
}

/// The line SU-7's click script adds to `blocks.scenario`.
const ANGEL: &str = "battlefield: Serra Angel | controller 1, attacking player 0\n";

/// A review board with `from` replaced by `to`, or `to` added at its end when
/// `from` is empty: the edit a click script makes in the editor, written to a
/// file of its own.
fn edited(board: &str, from: &str, to: &str) -> PathBuf {
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/scenarios").join(board)).unwrap();
    let text = if from.is_empty() { format!("{text}{to}") } else { text.replace(from, to) };
    let file = std::env::temp_dir().join(format!("devgui-picture-{}", board.replace(".scenario", "-edited.scenario")));
    std::fs::write(&file, text).unwrap();
    file
}

/// The why panel open on the card called `name` at the board's first
/// question: SU-7's section first, then what the layers did.
fn why_on(board: &Path, name: &str) -> WindowState {
    let engine = spawn(GameSetup { scenario: Some(board.to_path_buf()), ..from_board("main.scenario") });
    let mut state = WindowState::default();
    state.receive(next(&engine));
    let board = state.board.as_ref().expect("a board at the first question");
    let cards = board.players.iter().flat_map(|player| player.battlefield.iter().map(|p| &p.card).chain(&player.hand));
    let id = cards.into_iter().find(|card| card.name == name).map(|card| card.id).unwrap_or_else(|| panic!("no {name}"));
    engine.answers.send(state.input(Input::Why(BoardRef::Object(id))).expect("a why at an open question")).unwrap();
    state.receive(next(&engine));
    assert!(state.why_view().is_some_and(|view| view.sections[0].heading == "At this question"), "the panel shows");
    finish(engine);
    state
}

/// The why panel open on the event where Lightning Bolt deals its damage, on
/// §5.3's sample once its stack has resolved: read from a replay's trace,
/// stopped at the question then open (`setup-architecture.md` §7c, SU-8).
fn why_event(sample: &str) -> WindowState {
    let root = std::env::temp_dir().join("devgui-picture-why-event");
    let _ = std::fs::remove_dir_all(&root);
    let folders = Folders { logs: root.join("logs"), boards: root.join("boards"), ..Folders::default() };
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let start = Start::Game(GameSetup { scenario: Some(path), ..from_board("main.scenario") });
    let mut session = Session::start(start, folders, Arc::new(|| {}));
    let until = |session: &mut Session, done: &dyn Fn(&Session) -> bool| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        while !done(session) {
            assert!(std::time::Instant::now() < deadline, "{}", session.state.status());
            std::thread::sleep(std::time::Duration::from_millis(5));
            session.receive();
        }
    };
    let damage = loop {
        until(&mut session, &|session| session.state.prompt.is_some());
        if let Some(line) = session.state.log.iter().find(|line| line.text.starts_with("DamageDealt: Lightning Bolt")) {
            break line.event;
        }
        let pass = session.state.prompt.as_ref().and_then(|prompt| prompt.pass).expect("a priority question");
        session.input(Input::OptionButton(pass));
    };
    session.input(Input::WhyEvent(damage));
    until(&mut session, &|session| session.state.why_view().is_some_and(|view| view.title.starts_with("DamageDealt")));
    session.state.clone()
}

/// CR 509.1a's re-ask: the window declared its one Wall of Stone blocking
/// both Bears, and is asked again, told why.
fn blocks_rejected() -> WindowState {
    let engine = spawn(from_board("reask.scenario"));
    let mut state = WindowState::default();
    state.receive(next(&engine));
    state.input(Input::OptionButton(0));
    state.input(Input::OptionButton(1));
    engine.answers.send(state.input(Input::Done).expect("both blocks declared")).unwrap();
    state.receive(next(&engine));
    assert!(state.prompt.as_ref().is_some_and(|prompt| prompt.rejected.is_some()), "the re-ask says why");
    finish(engine);
    state
}

/// Close a game waiting on the window and let its thread end, so none is
/// still unwinding when the test process exits.
fn finish(engine: EngineHandle) {
    let EngineHandle { answers, thread, .. } = engine;
    drop(answers);
    thread.join().expect("a superseded game's thread ends without panicking");
}

/// What the window shows when the file does not load.
fn refused() -> WindowState {
    let path = std::env::temp_dir().join("devgui-picture-refused.scenario");
    std::fs::write(&path, "turn 3\nstep precombat main\nbattlefield: Grizzly Bears | controller 0, attacking player 1\n").unwrap();
    let engine = spawn(GameSetup { scenario: Some(path), ..from_board("main.scenario") });
    let message = next(&engine);
    assert!(matches!(message, ToWindow::Refused { .. }), "{message:?}");
    let mut state = WindowState::default();
    state.receive(message);
    state
}

fn picture(state: &WindowState, line: &(String, PathBuf), saved: Option<Result<&str, &str>>, name: &str) -> SnapshotResult {
    let mut harness = game_window(state, line, saved);
    if state.replaying.is_some() {
        // A replay's count repaints on a timer, so its window never settles.
        harness.run_steps(4);
    } else {
        harness.run();
    }
    harness.try_snapshot(name)
}

/// The game's window with the header's menu of savestates opened.
fn menu_picture(state: &WindowState, line: &(String, PathBuf), name: &str) -> SnapshotResult {
    let mut harness = game_window(state, line, None);
    harness.run();
    harness.get_by_label("Savestates").click();
    harness.run();
    harness.try_snapshot(name)
}

fn game_window<'a>(state: &'a WindowState, (line, log): &'a (String, PathBuf), saved: Option<Result<&'a str, &'a str>>) -> Harness<'a> {
    let tools = Some(state.tools_view());
    let header = SessionHeader { mode: Mode::Play, line, log: Some(log), playing: true, reloadable: true, message: saved, files: &[], tools };
    Harness::builder().with_size([1280.0, 800.0]).build_ui(move |ui| {
        draw(ui, state, &header, None);
    })
}

/// The editor on a sample, after a click on each named step or card, and
/// "bear" typed into the search with Grizzly Bears chosen. The session reads
/// the file and writes nothing.
fn editor_picture(sample: &str, clicks: &[&str], name: &str) -> SnapshotResult {
    // As `cargo run -- --edit ../mtgsim/scenarios/<sample>` names it, from `devgui/`.
    let path = Path::new("../mtgsim/scenarios").join(sample);
    let mut session = Session::start(Start::Edit(Some(path)), Folders::default(), Arc::new(|| {}));
    for clicked in clicks {
        let view = session.editor.view();
        let step = view.steps.iter().find(|step| step.label == *clicked).map(|step| step.input.clone());
        let cards = view.seats.iter().flat_map(|seat| seat.zones.iter().flat_map(|zone| &zone.cards));
        let card = cards.filter(|card| card.title.ends_with(*clicked)).map(|card| card.input.clone()).next();
        let input = step.or(card).unwrap_or_else(|| panic!("{sample} shows no {clicked}"));
        session.input(Input::Editor(input));
    }
    session.input(Input::Editor(EditorInput::Search("bear".to_string())));
    let bears = session.editor.view().search.results.into_iter().find(|result| result.label == "Grizzly Bears").map(|result| result.input);
    session.input(Input::Editor(bears.unwrap_or_else(|| panic!("no Grizzly Bears to choose"))));
    let view = session.editor.view();
    let header = SessionHeader { mode: Mode::Edit, line: "", log: None, playing: false, reloadable: false, message: None, files: &session.files, tools: None };
    let mut harness = Harness::builder().with_size([1280.0, 800.0]).build_ui(|ui| {
        draw(ui, &WindowState::default(), &header, Some(&view));
    });
    harness.run();
    harness.try_snapshot(name)
}
