//! One game and one board, and what Play, Reload, the editor and the saves
//! do to them. Plain Rust, so a test drives it with no window; `app` draws
//! over it and forwards each click here.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use mtgsim::cards::registry::CardRegistry;
use mtgsim::state::decision_log::GameStart;

use crate::boards::{Folders, ListedFile, start_name};
use crate::bridge::{EngineHandle, GameSetup, Play, Pool, Record, Writer, locked, spawn_game};
use crate::editor::{Editor, EditorInput, Source};
use crate::launch::{Start, start_line};
use crate::save::{self, Save};
use crate::view_model::{Input, Mode, Progress, Refusal, WindowState};

pub struct Session {
    /// How the game the window plays began, once one has; Reload begins it
    /// again.
    pub setup: Option<GameSetup>,
    /// Where it began, as read at its start.
    pub start: Option<GameStart>,
    /// That game's decision log, a file no earlier game wrote.
    pub log_path: Option<PathBuf>,
    /// The game's seed and start, for the header.
    pub start_line: String,
    /// What the window shows of the game: the engine's last messages and
    /// the answer in progress.
    pub state: WindowState,
    /// The board in the editor, kept as it is while a game is played.
    pub editor: Editor,
    pub mode: Mode,
    pub folders: Folders,
    /// The header's list of files to open, read again at each save.
    pub files: Vec<ListedFile>,
    /// What the last save or open did, for the header: where it saved, or
    /// why it could not.
    pub message: Option<Result<String, String>>,
    engine: Option<EngineHandle>,
    /// The game's record: its log, its save, and the tree of play the
    /// window's tools read.
    record: Option<Arc<Mutex<Record>>>,
    wake: Arc<dyn Fn() + Send + Sync>,
}

impl Session {
    pub fn start(start: Start, folders: Folders, wake: Arc<dyn Fn() + Send + Sync>) -> Session {
        let mut session = Session {
            setup: None,
            start: None,
            log_path: None,
            start_line: String::new(),
            state: WindowState::default(),
            editor: Editor::empty(CardRegistry::default_registry()),
            mode: Mode::Play,
            files: folders.listed(),
            folders,
            message: None,
            engine: None,
            record: None,
            wake,
        };
        match start {
            Start::Game(setup) => session.start_game(setup),
            Start::Edit(file) => {
                session.mode = Mode::Edit;
                if let Some(path) = file {
                    session.open_file(&path);
                }
            }
        }
        session
    }

    /// The window's clock, which settles each new prompt (`WindowState::settling_for`).
    pub fn tick(&mut self, now: f64) {
        self.state.tick(now);
    }

    /// Take every message the engine has sent since the last call. The
    /// engine's thread waits at the prompt it sent, or has ended, so the
    /// save is read as it stands there.
    pub fn receive(&mut self) {
        let mut received = false;
        while let Some(message) = self.engine.as_ref().and_then(|engine| engine.from_engine.try_recv().ok()) {
            self.state.receive(message);
            received = true;
        }
        if received {
            self.read_tools();
        }
    }

    /// Act on one input: a prompt's goes to the engine once it completes an
    /// answer, the editor's to the editor, and the window's own controls act
    /// here.
    pub fn input(&mut self, input: Input) {
        match input {
            // The game again from its file, read again.
            Input::Reload => {
                if let Some(setup) = self.setup.clone() {
                    self.start_game(setup);
                }
            }
            Input::Undo => self.undo(),
            Input::Savestate => self.savestate(),
            Input::MoveTo(place) => {
                if let Some(record) = self.record.clone() {
                    self.rebuild(&record, place);
                }
            }
            // From the window's thread, at any moment: the seat reads the
            // switch at its next prompt, and the log never records it.
            Input::FullControl(on) => {
                if let Some(engine) = &self.engine {
                    engine.full_control.set(on);
                }
                self.state.input(input);
            }
            Input::SaveBoard => self.save_game_board(),
            Input::Mode(mode) => self.mode = mode,
            Input::EditThisBoard => self.edit_this_board(),
            Input::EditTheScenario => self.edit_the_scenario(),
            Input::Editor(EditorInput::Play) => self.play_board(),
            Input::Editor(EditorInput::Save) => {
                self.save_board();
            }
            Input::Editor(EditorInput::Open(i)) => {
                if let Some(file) = self.files.get(i).cloned() {
                    self.open_file(&file.path);
                }
            }
            Input::Editor(edit) => self.editor.input(edit),
            input => {
                if let Some(reply) = self.state.input(input)
                    && let Some(engine) = &self.engine
                {
                    // Fails only once the engine thread has ended, and its last message said why.
                    let _ = engine.answers.send(reply);
                }
            }
        }
    }

    /// `setup`'s game on a thread of its own, its start read now and its
    /// record a log and a save no earlier game wrote, the log named for the
    /// seed the start plays.
    fn start_game(&mut self, setup: GameSetup) {
        self.leave_game();
        let begun = setup.start();
        self.setup = Some(setup.clone());
        let start = match begun {
            Ok(start) => start,
            Err(message) => {
                self.start_line = setup.scenario.map_or_else(String::new, |path| format!("scenario {}", path.display()));
                self.state.refused = Some((Refusal::Scenario, message));
                return;
            }
        };
        self.start_line = start_line(&setup, &start);
        let log_path = own_log(&self.folders.game_log(&start));
        let record = match Record::create(Save::new(start.clone()), log_path.clone()) {
            Ok(record) => Arc::new(Mutex::new(record)),
            Err(message) => {
                self.state.refused = Some((Refusal::Record, message));
                return;
            }
        };
        self.spawn(Play { record: Some(Writer::take_over(&record)), ..Play::new(start.clone()) });
        (self.start, self.log_path, self.record) = (Some(start), Some(log_path), Some(record));
    }

    /// Undo answer: the window's previous question asked again, on a game
    /// built again from its start and replayed to it (`setup-architecture.md`
    /// §7.2, decision 2). While a replay runs, each press moves its target
    /// back one more question.
    fn undo(&mut self) {
        let Some(record) = self.record.clone() else { return };
        let open = self.state.prompt.is_some() || self.state.replaying.is_some();
        let target = locked(&record).save.undo_target(open);
        if let Some(place) = target {
            self.rebuild(&record, place);
        }
    }

    /// Savestate: the open question's place kept in the save, named for its
    /// turn and step, for the menu to come back to.
    fn savestate(&mut self) {
        let (Some(record), Some(name)) = (&self.record, self.state.savestate_name()) else { return };
        if !self.state.tools.savestate_here {
            locked(record).savestate(name);
            self.read_tools();
        }
    }

    /// The game built again from its start and replayed to `place`, where
    /// the save's line moves; the window counts the replay until the engine
    /// asks there. The game it replaces writes nothing more, its replay
    /// stopping at its next answer, and its thread unwinds when its channel
    /// closes.
    fn rebuild(&mut self, record: &Arc<Mutex<Record>>, place: usize) {
        let Some(start) = self.start.clone() else { return };
        let writer = Writer::take_over(record);
        let line = {
            let mut record = locked(record);
            record.move_to(place);
            record.save.line_to(place)
        };
        let replaying = Progress { done: writer.replayed(), of: line.len() };
        let full_control = self.state.full_control;
        self.state = WindowState { full_control, now: self.state.now, replaying: Some(replaying), ..WindowState::default() };
        self.spawn(Play { start, line, audited: false, record: Some(writer) });
        self.read_tools();
    }

    /// What the save lets the tools do, read again after it changed.
    fn read_tools(&mut self) {
        self.state.tools = self.record.as_ref().map(|record| locked(record).save.tools()).unwrap_or_default();
    }

    /// The game left for another: its record shut, so a replay on its way
    /// stops and nothing more is written, and the window cleared but for
    /// full control, which is the window's. Its thread unwinds when its
    /// channel closes, as a closed window ends it.
    fn leave_game(&mut self) {
        if let Some(record) = self.record.take() {
            locked(&record).shut();
        }
        let full_control = self.state.full_control;
        self.state = WindowState { full_control, now: self.state.now, ..WindowState::default() };
        (self.engine, self.start, self.log_path, self.message) = (None, None, None, None);
    }

    /// `play` on a thread of its own, under the window's full control.
    fn spawn(&mut self, play: Play) {
        let engine = spawn_game(play, Arc::clone(&self.wake));
        engine.full_control.set(self.state.full_control);
        self.engine = Some(engine);
    }

    /// The editor's board saved, then started from its file as Reload
    /// starts one, its log beside it: only while the loader accepts it.
    fn play_board(&mut self) {
        if self.editor.refusal().is_some() {
            return;
        }
        let Some(path) = self.save_board() else { return };
        let players = self.editor.board().players;
        self.start_game(GameSetup { seed: None, pool: Pool::Performance, players, scenario: Some(path) });
        self.mode = Mode::Play;
    }

    /// The editor's board written to its file, which a board not yet in
    /// `boards/` gets now; that file, or `None` and why not in the header.
    fn save_board(&mut self) -> Option<PathBuf> {
        let path = self.folders.save_path(&self.editor.source);
        let saved = self.write(&path, &self.editor.file_text());
        if saved.is_some() {
            self.editor.source = Source::Board(path.clone());
        }
        saved
    }

    /// "Save board as scenario": the board the game is at, a new board
    /// named for the game's start and turn, its first line naming the log.
    fn save_game_board(&mut self) {
        let (Some(board), Some(start)) = (&self.state.board, &self.start) else { return };
        let path = self.folders.new_board(&format!("{}-turn-{}", start_name(start), board.turn));
        let text = format!("# Saved from {}, turn {}.\n{}", self.game_named(), board.turn, board.board_text);
        self.write(&path, &text);
    }

    /// "Edit this board": the board the game is at, as `Scenario::write`
    /// writes it at each prompt, the writer's report kept above it.
    fn edit_this_board(&mut self) {
        let (Some(board), Some(start)) = (&self.state.board, &self.start) else { return };
        let text = format!("# From {}, turn {}.\n{}", self.game_named(), board.turn, board.board_text);
        let source = Source::Game { start: start_name(start), turn: board.turn };
        self.open(&text, source, "the game's board");
    }

    /// "Edit the scenario": the editor's own board if Play started this
    /// game, or else its file, read again.
    fn edit_the_scenario(&mut self) {
        let Some(path) = self.setup.as_ref().and_then(|setup| setup.scenario.clone()) else { return };
        if self.editor.source == Source::Board(path.clone()) {
            self.mode = Mode::Edit;
        } else {
            self.open_file(&path);
        }
    }

    fn open_file(&mut self, path: &Path) {
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let source = if self.folders.holds(path) { Source::Board(path.to_path_buf()) } else { Source::File(path.to_path_buf()) };
                self.open(&text, source, &path.display().to_string());
            }
            Err(e) => self.message = Some(Err(format!("cannot read {}: {e}", path.display()))),
        }
    }

    /// `text` in the editor, in place of its board, which a person keeps
    /// by saving it first.
    fn open(&mut self, text: &str, source: Source, what: &str) {
        match Editor::open(text, source, CardRegistry::default_registry()) {
            Ok(editor) => {
                (self.editor, self.mode, self.message) = (editor, Mode::Edit, None);
            }
            Err(refusal) => self.message = Some(Err(format!("cannot open {what}: {refusal}"))),
        }
    }

    /// The game for a board's first line: its decision log.
    fn game_named(&self) -> String {
        self.log_path.as_ref().map_or_else(|| "a game".to_string(), |log| log.display().to_string())
    }

    /// `text` written to `path`, its folder made; the header says where, or
    /// why not, and the list is read again.
    fn write(&mut self, path: &Path, text: &str) -> Option<PathBuf> {
        let made = path.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|()| std::fs::write(path, text));
        self.files = self.folders.listed();
        self.message = Some(match &made {
            Ok(()) => Ok(format!("saved {}", path.display())),
            Err(e) => Err(format!("cannot save {}: {e}", path.display())),
        });
        made.ok().map(|()| path.to_path_buf())
    }
}

/// `path`, or the first of `stem-2.log`, `stem-3.log`, … beside it that is
/// not on disk, nor its save: Reload keeps the record of the game it
/// replaces, and a thread still finishing that game writes only its own.
fn own_log(path: &Path) -> PathBuf {
    let stem = path.file_stem().map_or("game".to_string(), |s| s.to_string_lossy().into_owned());
    let extension = path.extension().map_or(String::new(), |e| format!(".{}", e.to_string_lossy()));
    let named = |n: u32| match n {
        1 => path.to_path_buf(),
        n => path.with_file_name(format!("{stem}-{n}{extension}")),
    };
    (1..).map(named).find(|path| !path.exists() && !save::path_for(path).exists()).unwrap_or_else(|| path.to_path_buf())
}
