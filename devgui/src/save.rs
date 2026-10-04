//! The save: a session's journal of play, `<log>.save` beside each decision
//! log (`setup-architecture.md` §7.2, decision 4). The log is the one line of
//! play the window is on. The save keeps every line the session played, where
//! the window was asked, the savestates set and each move back, so Undo
//! answer, a savestate and a load can each rebuild the game at any of them.
//!
//! A **place** is the game after an answer, named by the answer's number;
//! place 0 is the start, before any answer.
//!
//! **A journal in the engine's words and the session's.** The engine's are a
//! decision log's own (`mtgsim::state::decision_log`): the format, the
//! engine, the start, and an answer a line, numbered in the order the save
//! took them, so the engine reads them as one record. Each answer follows the
//! place before it in the journal, or the place the last move named. The
//! session's own lines say where the window was asked, where a savestate is,
//! and where the window's line moved:
//!
//! ```text
//! decision log 1
//! engine 1de83d3a1b2c
//! …the start's lines…
//! window asked at 0
//! answer 1 [turn 1, precombat main] player 0 PriorityAction picks cast Lightning Bolt (#12)
//! window asked at 1
//! savestate at 1: Turn 1 · Precombat Main
//! answer 2 [turn 1, precombat main] player 0 SelectRecipients picks player 1
//! moved to 1
//! answer 3 [turn 1, precombat main] player 0 SelectRecipients picks Grizzly Bears (#7)
//! ```
//!
//! Answer 3 follows place 1, as the second answer on its line. A decision
//! log alone reads as a save with one line and no record of the window.

use std::fmt;
use std::path::{Path, PathBuf};

use mtgsim::state::decision_log::{self, AnswerLine, GameStart};
use mtgsim::state::trace::COMMIT;

/// The save beside the decision log at `log`: `<log>.save`.
pub fn path_for(log: &Path) -> PathBuf {
    let mut path = log.as_os_str().to_owned();
    path.push(".save");
    PathBuf::from(path)
}

/// A session's journal, and the tree of play it describes.
#[derive(Clone, Debug, PartialEq)]
pub struct Save {
    /// The engine whose lines open the save, which a replay's divergence
    /// names beside this one when they differ.
    pub engine: String,
    pub start: GameStart,
    entries: Vec<Entry>,
    /// Each answer's entry and the place it follows, by its number less one.
    answers: Vec<(usize, usize)>,
    /// Whether the window was asked at each place, by place.
    asked: Vec<bool>,
    /// The place the window's line is at.
    current: usize,
    /// The place the window's line most recently left.
    left: Option<usize>,
}

/// One line of the journal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    /// An answer, numbered by its order in the save.
    Answer(AnswerLine),
    /// The window was asked the question at this place.
    WindowAsked(usize),
    /// A savestate at this place, named for its turn and step.
    Savestate { at: usize, name: String },
    /// The window's line moved to this place.
    Moved(usize),
}

/// What the save lets the window's tools do, read when it last changed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tools {
    /// Undo answer has a place to go while a question is open, or a
    /// rebuild is on its way to one.
    pub undo_open: bool,
    /// And once the open question is answered.
    pub undo_answered: bool,
}

/// A place the window's menu moves to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Destination {
    Savestate { at: usize, name: String },
    /// The place the window's line most recently left, while it is not on
    /// the line the window is on: "back to where I was".
    Left(usize),
}

impl Save {
    /// A save of a game starting at `start`, as this engine writes one.
    pub fn new(start: GameStart) -> Save {
        Save::opened(COMMIT.to_string(), start)
    }

    fn opened(engine: String, start: GameStart) -> Save {
        Save { engine, start, entries: Vec::new(), answers: Vec::new(), asked: vec![false], current: 0, left: None }
    }

    /// The whole journal, opened as this engine opens a record.
    pub fn text(&self) -> String {
        let mut text = String::new();
        for line in decision_log::opening(&self.start) {
            text.push_str(&line);
            text.push('\n');
        }
        for entry in &self.entries {
            text.push_str(&entry.to_string());
            text.push('\n');
        }
        text
    }

    /// The place the window's line is at.
    pub fn current(&self) -> usize {
        self.current
    }

    /// `line` as the save's next answer, after the current place; its
    /// journal line.
    pub fn answer(&mut self, line: AnswerLine) -> String {
        self.push(Entry::Answer(AnswerLine { number: self.answers.len() + 1, ..line }))
    }

    /// The window asked at the current place; its journal line, unless the
    /// save already has it.
    pub fn window_asked(&mut self) -> Option<String> {
        (!self.asked[self.current]).then(|| self.push(Entry::WindowAsked(self.current)))
    }

    /// A savestate at the current place, named `name`; its journal line.
    pub fn savestate(&mut self, name: String) -> String {
        self.push(Entry::Savestate { at: self.current, name })
    }

    /// The window's line moved to `place`; its journal line.
    pub fn move_to(&mut self, place: usize) -> String {
        self.push(Entry::Moved(place))
    }

    /// Takes `entry` into the tree and the journal. Every place it names is
    /// one the save has: the live writers name the current place, and the
    /// reader checks.
    fn push(&mut self, entry: Entry) -> String {
        match &entry {
            Entry::Answer(_) => {
                self.answers.push((self.entries.len(), self.current));
                self.asked.push(false);
                self.current = self.answers.len();
            }
            Entry::WindowAsked(place) => self.asked[*place] = true,
            Entry::Savestate { .. } => {}
            Entry::Moved(place) => {
                // Undo after undo walks back along one line, so the place it
                // left stays where that line got to.
                if !self.left.is_some_and(|left| self.passes(self.current, left)) {
                    self.left = Some(self.current);
                }
                self.current = *place;
            }
        }
        let line = entry.to_string();
        self.entries.push(entry);
        line
    }

    /// The place before `place` on its line, or none for the start.
    fn before(&self, place: usize) -> Option<usize> {
        place.checked_sub(1).map(|at| self.answers[at].1)
    }

    /// Whether the line to `to` passes through `place`, or ends there.
    fn passes(&self, place: usize, to: usize) -> bool {
        std::iter::successors(Some(to), |at| self.before(*at)).any(|at| at == place)
    }

    /// The answers from the start to `place`, numbered as its own line.
    pub fn line_to(&self, place: usize) -> Vec<AnswerLine> {
        let mut places: Vec<usize> = std::iter::successors(Some(place), |at| self.before(*at)).filter(|at| *at > 0).collect();
        places.reverse();
        places
            .into_iter()
            .enumerate()
            .filter_map(|(i, at)| match &self.entries[self.answers[at - 1].0] {
                Entry::Answer(line) => Some(AnswerLine { number: i + 1, ..line.clone() }),
                _ => None,
            })
            .collect()
    }

    /// The place `answers` answers along the line to `place`.
    pub fn along(&self, place: usize, answers: usize) -> usize {
        let mut line: Vec<usize> = std::iter::successors(Some(place), |at| self.before(*at)).collect();
        line.reverse();
        line.get(answers).copied().unwrap_or(place)
    }

    /// Where Undo answer goes: the last place on the line to the current one
    /// where the window was asked, before it while its question is `open`
    /// (or a rebuild is on its way to it), or at it once that is answered.
    pub fn undo_target(&self, open: bool) -> Option<usize> {
        let from = if open { self.before(self.current)? } else { self.current };
        std::iter::successors(Some(from), |at| self.before(*at)).find(|at| self.asked[*at])
    }

    pub fn tools(&self) -> Tools {
        Tools { undo_open: self.undo_target(true).is_some(), undo_answered: self.undo_target(false).is_some() }
    }

    /// Whether a savestate is at `place`.
    pub fn has_savestate(&self, place: usize) -> bool {
        self.entries.iter().any(|entry| matches!(entry, Entry::Savestate { at, .. } if *at == place))
    }

    /// What the window's menu lists: every savestate, in the order set, then
    /// the place the window's line most recently left, while the line the
    /// window is on does not pass through it. The lines played and left
    /// before it stay in the journal and out of the menu (the owner, at
    /// #216's review).
    pub fn destinations(&self) -> Vec<Destination> {
        let savestates = self.entries.iter().filter_map(|entry| match entry {
            Entry::Savestate { at, name } => Some(Destination::Savestate { at: *at, name: name.clone() }),
            _ => None,
        });
        let left = self.left.filter(|left| !self.passes(*left, self.current)).map(Destination::Left);
        savestates.chain(left).collect()
    }

    /// A save read back from its journal, or a decision log read as a save
    /// with one line; what does not read is refused naming its line.
    pub fn read(text: &str) -> Result<Save, String> {
        let mut record = String::new();
        // Each engine line's line in `text`, for a refusal; and the session's
        // own lines, each with the answers before it.
        let mut numbered: Vec<usize> = Vec::new();
        let mut ours: Vec<(usize, usize, Entry)> = Vec::new();
        let (mut answers, mut scenario_text) = (0, false);
        for (at, line) in text.lines().enumerate() {
            let entry = if scenario_text { None } else { Entry::read(line).transpose().map_err(|why| format!("line {}: {why}", at + 1))? };
            match entry {
                Some(entry) => ours.push((at + 1, answers, entry)),
                None => {
                    // A scenario's text is the engine's, whatever its lines
                    // say (`state::decision_log`'s start).
                    match line {
                        "begin scenario text" => scenario_text = true,
                        "end scenario text" => scenario_text = false,
                        _ => answers += usize::from(!scenario_text && line.starts_with("answer ")),
                    }
                    record.push_str(line);
                    record.push('\n');
                    numbered.push(at + 1);
                }
            }
        }
        let log = decision_log::read(&record).map_err(|refusal| {
            let line = numbered.get(refusal.line - 1).copied().unwrap_or(text.lines().count() + 1);
            format!("line {line}: {}", refusal.why)
        })?;
        let mut save = Save::opened(log.engine, log.start);
        let mut answer_lines = log.answers.into_iter();
        for (line, before, entry) in ours {
            while save.answers.len() < before {
                let Some(answer) = answer_lines.next() else { break };
                save.push(Entry::Answer(answer));
            }
            let named = match &entry {
                Entry::WindowAsked(at) | Entry::Savestate { at, .. } | Entry::Moved(at) => *at,
                Entry::Answer(_) => 0,
            };
            if named > save.answers.len() {
                return Err(format!("line {line}: place {named}, after {} answers", save.answers.len()));
            }
            save.push(entry);
        }
        for answer in answer_lines {
            save.push(Entry::Answer(answer));
        }
        Ok(save)
    }
}

impl Entry {
    /// A session's line read back: `None` for the engine's.
    fn read(line: &str) -> Option<Result<Entry, String>> {
        let place = |text: &str| text.parse::<usize>().map_err(|_| format!("place `{text}` is not a number"));
        if let Some(rest) = line.strip_prefix("window asked at ") {
            return Some(place(rest).map(Entry::WindowAsked));
        }
        if let Some(rest) = line.strip_prefix("savestate at ") {
            let Some((at, name)) = rest.split_once(": ") else {
                return Some(Err(format!("`{line}`: a savestate is `savestate at N: name`")));
            };
            return Some(place(at).map(|at| Entry::Savestate { at, name: name.to_string() }));
        }
        line.strip_prefix("moved to ").map(|rest| place(rest).map(Entry::Moved))
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entry::Answer(line) => write!(f, "{line}"),
            Entry::WindowAsked(at) => write!(f, "window asked at {at}"),
            Entry::Savestate { at, name } => write!(f, "savestate at {at}: {name}"),
            Entry::Moved(to) => write!(f, "moved to {to}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use mtgsim::state::decision_log::Chosen;
    use mtgsim::state::game_config::GameConfig;

    use super::*;

    fn start() -> GameStart {
        GameStart::Dealt { seed: 41, config: GameConfig::unrestricted(), decks: vec![vec!["Mountain".to_string()]; 2] }
    }

    /// An answer as the engine would log it, telling lines apart by `picked`.
    fn answer(picked: &str) -> AnswerLine {
        let chosen = Chosen::Picks(vec![picked.to_string()]);
        AnswerLine { number: 0, turn: 1, step: "precombat main".to_string(), player: 0, kind: "PriorityAction".to_string(), forced: false, chosen }
    }

    fn picked(line: &[AnswerLine]) -> Vec<String> {
        line.iter().map(|answer| match &answer.chosen {
            Chosen::Picks(picks) => picks.join(""),
            other => format!("{other:?}"),
        }).collect()
    }

    /// Three answers with the window asked before each, a savestate at the
    /// second, Undo answer back to it, and a branch: the save of the
    /// module's example, without its start.
    fn branched() -> Save {
        let mut save = Save::new(start());
        for picked in ["a", "b", "c"] {
            save.window_asked();
            save.answer(answer(picked));
        }
        save.window_asked();
        save.move_to(1);
        save.savestate("Turn 1 · Precombat Main".to_string());
        save.answer(answer("d"));
        save
    }

    /// The journal's text, pinned after the start: a change here is a change
    /// to every save on disk, which a new reader must still read.
    #[test]
    fn a_save_is_written_and_read_back_as_its_journal() {
        let save = branched();
        let text = save.text();
        let session: Vec<&str> = text.lines().skip_while(|line| !line.starts_with("window")).collect();
        assert_eq!(session, [
            "window asked at 0",
            "answer 1 [turn 1, precombat main] player 0 PriorityAction picks a",
            "window asked at 1",
            "answer 2 [turn 1, precombat main] player 0 PriorityAction picks b",
            "window asked at 2",
            "answer 3 [turn 1, precombat main] player 0 PriorityAction picks c",
            "window asked at 3",
            "moved to 1",
            "savestate at 1: Turn 1 · Precombat Main",
            "answer 4 [turn 1, precombat main] player 0 PriorityAction picks d",
        ]);
        assert_eq!(Save::read(&text), Ok(save));
    }

    /// Each answer follows the place before it, so a branch's line runs
    /// from the start through the place it left from, numbered as its own.
    #[test]
    fn a_line_runs_from_the_start_through_its_branch() {
        let save = branched();
        assert_eq!(save.current(), 4);
        assert_eq!(picked(&save.line_to(4)), ["a", "d"]);
        assert_eq!(save.line_to(4).iter().map(|answer| answer.number).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(picked(&save.line_to(3)), ["a", "b", "c"]);
        assert!(save.line_to(0).is_empty());
        assert_eq!((save.along(4, 1), save.along(3, 2), save.along(4, 0)), (1, 2, 0));
    }

    /// Undo answer goes to the last place the window was asked: before the
    /// open question, or at the question just answered.
    #[test]
    fn undo_goes_back_to_where_the_window_was_last_asked() {
        let mut save = branched();
        assert_eq!(save.undo_target(false), Some(1), "after answer 4, the question it answered");
        save.window_asked();
        assert_eq!(save.undo_target(true), Some(1), "a question open at 4: the one before it, on its own line");
        save.move_to(1);
        assert_eq!(save.undo_target(true), Some(0));
        save.move_to(0);
        assert_eq!(save.undo_target(true), None, "nothing before the first question");
    }

    /// The menu lists the savestates and the place the window's line most
    /// recently left, while the window's line does not pass through it.
    #[test]
    fn the_menu_lists_the_savestates_and_where_the_line_was_left() {
        let mut save = branched();
        let savestate = Destination::Savestate { at: 1, name: "Turn 1 · Precombat Main".to_string() };
        assert_eq!(save.destinations(), [savestate.clone(), Destination::Left(3)], "left at 3 by the undo to 1");
        save.move_to(3);
        assert_eq!(save.destinations(), [savestate.clone(), Destination::Left(4)], "back to 3, leaving the branch");
        save.move_to(2);
        save.move_to(1);
        assert_eq!(save.destinations(), [savestate.clone(), Destination::Left(3)], "two undos along one line leave it once");
        save.move_to(3);
        assert_eq!(save.destinations(), [savestate], "back where it was");
    }

    /// A decision log is a save with one line, and nothing says where the
    /// window was asked.
    #[test]
    fn a_decision_log_reads_as_a_save_of_one_line() {
        let mut lines: Vec<String> = decision_log::opening(&start());
        lines.extend(["answer 1 [turn 1, upkeep] player 0 PriorityAction forced picks pass", "outcome draw"].map(String::from));
        let save = Save::read(&(lines.join("\n") + "\n")).unwrap();
        assert_eq!((save.current(), save.line_to(1).len(), save.undo_target(false)), (1, 1, None));
    }

    /// A line of a scenario's text stays the scenario's, whatever it reads
    /// as; a place no answer has reached, and an engine line that does not
    /// read, are refused naming their line in the save.
    #[test]
    fn a_save_that_does_not_read_names_its_line() {
        let text = "turn 2\nmoved to 7\n".to_string();
        let scenario = Save::new(GameStart::Scenario { path: "x.scenario".to_string(), seed: 0, text });
        assert_eq!(Save::read(&scenario.text()), Ok(scenario.clone()));
        let journal = branched().text();
        let next = journal.lines().count() + 1;
        let after = |line: &str| Save::read(&format!("{journal}{line}\n"));
        assert_eq!(after("moved to 9"), Err(format!("line {next}: place 9, after 4 answers")));
        assert_eq!(after("window asked at nine"), Err(format!("line {next}: place `nine` is not a number")));
        assert_eq!(after("answer 9 [turn 1, upkeep] player 0 PriorityAction picks a"), Err(format!("line {next}: answer 9 follows answer 4")));
    }
}
