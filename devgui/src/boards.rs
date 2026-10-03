//! Where a board and the records of its games live (`setup-architecture.md`
//! §7b, decision 3): `boards/<board>/`, a folder a board, holding its file
//! and the decision log of every game played from it. A dealt game starts
//! from no board, so its logs stay in `logs/`. Nothing here writes a file
//! outside `boards/`, so a committed board becomes a board of its own at its
//! first save, and a sample by a copy its PR reviews.

use std::path::{Path, PathBuf};

use crate::bridge::GameSetup;
use crate::editor::Source;

/// The folders the dev GUI writes into, and the committed ones it lists.
#[derive(Clone, Debug)]
pub struct Folders {
    /// Dealt games' decision logs.
    pub logs: PathBuf,
    /// A folder a board.
    pub boards: PathBuf,
    /// The committed boards' folders, each with the name the list shows.
    pub committed: Vec<(String, PathBuf)>,
}

impl Default for Folders {
    /// Where the window runs, `devgui/` as `cargo run` runs it: `logs/` and
    /// `boards/`, and the tree's samples and review boards.
    fn default() -> Folders {
        Folders {
            logs: PathBuf::from("logs"),
            boards: PathBuf::from("boards"),
            committed: vec![
                ("mtgsim/scenarios".to_string(), PathBuf::from("../mtgsim/scenarios")),
                ("devgui/tests/scenarios".to_string(), PathBuf::from("tests/scenarios")),
            ],
        }
    }
}

/// A file the header's list opens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListedFile {
    pub label: String,
    pub path: PathBuf,
}

impl Folders {
    /// A game's decision log: in its board's folder, named for the seed; a
    /// dealt game's in `logs/`, named for the seed and the seats.
    pub fn game_log(&self, setup: &GameSetup) -> PathBuf {
        match &setup.scenario {
            Some(path) => self.boards.join(stem(path)).join(format!("seed-{}.log", setup.seed)),
            None => self.logs.join(format!("{}.log", start_name(setup))),
        }
    }

    /// Where a board from `source` saves: its own file, or a new board's.
    pub fn save_path(&self, source: &Source) -> PathBuf {
        match source {
            Source::Board(path) => path.clone(),
            Source::Empty => self.first_unused((1..).map(|n| format!("board-{n}"))),
            Source::Game { start, turn } => self.new_board(&format!("{start}-turn-{turn}")),
            Source::File(path) => self.new_board(&stem(path)),
        }
    }

    /// The first of `stem`, `stem-2`, `stem-3`, … with no board file, so a
    /// folder holding only a committed board's games takes the first save.
    pub fn new_board(&self, stem: &str) -> PathBuf {
        self.first_unused(std::iter::once(stem.to_string()).chain((2..).map(|n| format!("{stem}-{n}"))))
    }

    fn first_unused(&self, names: impl Iterator<Item = String>) -> PathBuf {
        names.map(|name| self.boards.join(&name).join(format!("{name}.scenario"))).find(|path| !path.exists()).unwrap_or_default()
    }

    /// Is `path` in `boards/`, a board's own file?
    pub fn holds(&self, path: &Path) -> bool {
        match (path.canonicalize(), self.boards.canonicalize()) {
            (Ok(path), Ok(boards)) => path.starts_with(boards),
            _ => false,
        }
    }

    /// The boards in `boards/`, then the committed scenarios, each by name.
    pub fn listed(&self) -> Vec<ListedFile> {
        let boards = folder(&self.boards).into_iter().filter_map(|dir| {
            let name = dir.file_name()?.to_str()?.to_string();
            let path = dir.join(format!("{name}.scenario"));
            path.exists().then(|| ListedFile { label: format!("boards/{name}"), path })
        });
        let committed = self.committed.iter().flat_map(|(label, dir)| {
            let scenarios = folder(dir).into_iter().filter(|path| path.extension().is_some_and(|e| e == "scenario"));
            scenarios.map(move |path| ListedFile { label: format!("{label}/{}", stem(&path)), path })
        });
        boards.chain(committed).collect()
    }
}

/// What `dir` holds, by name; nothing when it cannot be read.
fn folder(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir).into_iter().flatten().flatten().map(|entry| entry.path()).collect();
    paths.sort();
    paths
}

/// The name of a game's start: its board's, or a dealt game's seed and seats.
pub fn start_name(setup: &GameSetup) -> String {
    match &setup.scenario {
        Some(path) => stem(path),
        None if setup.players == 2 => format!("seed-{}", setup.seed),
        None => format!("seed-{}-players-{}", setup.seed, setup.players),
    }
}

fn stem(path: &Path) -> String {
    path.file_stem().map_or("board".to_string(), |stem| stem.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::Pool;

    fn temp_folders(name: &str) -> Folders {
        let root = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&root);
        let committed = vec![("samples".to_string(), root.join("samples"))];
        Folders { logs: root.join("logs"), boards: root.join("boards"), committed }
    }

    /// A board is named once and keeps its file; a committed board's games
    /// share its folder, which its first save then takes; a dealt game's log
    /// stays in `logs/`.
    #[test]
    fn a_board_gets_a_folder_of_its_own_beside_its_games() {
        let folders = temp_folders("devgui-boards");
        let sample = folders.committed[0].1.join("holy-strength.scenario");
        let dealt = GameSetup { seed: 41, pool: Pool::Performance, players: 4, log_path: None, scenario: None };
        assert_eq!(folders.game_log(&dealt), folders.logs.join("seed-41-players-4.log"));
        let from_sample = GameSetup { seed: 7, scenario: Some(sample.clone()), ..dealt };
        assert_eq!(folders.game_log(&from_sample), folders.boards.join("holy-strength").join("seed-7.log"));
        let first = folders.save_path(&Source::File(sample.clone()));
        assert_eq!(first, folders.boards.join("holy-strength").join("holy-strength.scenario"));
        for path in [&first, &sample] {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "turn 2\n").unwrap();
        }
        assert_eq!(folders.save_path(&Source::File(sample)), folders.boards.join("holy-strength-2").join("holy-strength-2.scenario"));
        assert_eq!(folders.save_path(&Source::Board(first.clone())), first);
        assert_eq!(folders.save_path(&Source::Empty), folders.boards.join("board-1").join("board-1.scenario"));
        let game = Source::Game { start: "seed-41".to_string(), turn: 3 };
        assert_eq!(folders.save_path(&game), folders.boards.join("seed-41-turn-3").join("seed-41-turn-3.scenario"));
        assert!(folders.holds(&first) && !folders.holds(&folders.committed[0].1.join("holy-strength.scenario")));
        let labels: Vec<String> = folders.listed().into_iter().map(|file| file.label).collect();
        assert_eq!(labels, ["boards/holy-strength", "samples/holy-strength"]);
    }
}
