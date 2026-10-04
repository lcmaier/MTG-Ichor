//! The command line, read into how the window starts.
//!
//! `cargo run -- [--seed N] [--pool performance|stress] [--players N] [--scenario FILE]`,
//! or `cargo run -- --edit [FILE]` for the board editor.
//!
//! The seed defaults to the scenario's own, read at each start, or else to
//! the clock. The header shows it and the decision log records it, so any
//! game can be played again.

use std::path::PathBuf;

use mtgsim::state::decision_log::GameStart;

use crate::bridge::{GameSetup, Pool};

pub const USAGE: &str =
    "usage: devgui [--seed N] [--pool performance|stress] [--players N] [--scenario FILE], or devgui --edit [FILE]";

/// What the window starts with.
#[derive(Clone, Debug)]
pub enum Start {
    /// A game, dealt or from a scenario's file.
    Game(GameSetup),
    /// The board editor, on a file's board or an empty one.
    Edit(Option<PathBuf>),
}

/// The game's seed and start, as the header says them: the seed `start`
/// played, which a scenario's file may have given.
pub fn start_line(setup: &GameSetup, start: &GameStart) -> String {
    match start {
        GameStart::Scenario { path, seed, .. } => format!("scenario {path} · seed {seed}"),
        GameStart::Dealt { seed, decks, .. } if decks.len() == 2 => format!("seed {seed} · {:?} pool", setup.pool),
        GameStart::Dealt { seed, decks, .. } => format!("seed {seed} · {:?} pool · {} players", setup.pool, decks.len()),
    }
}

/// `args`, the words after the program's name, read into a [`Start`];
/// `clock` seeds a dealt game given no `--seed`. An `Err` says what to
/// change, for the terminal.
pub fn read(args: &[String], clock: impl FnOnce() -> u64) -> Result<Start, String> {
    let (mut seed, mut pool, mut players, mut scenario, mut edit) = (None, None, None, None, None);
    let mut words = args.iter().peekable();
    while let Some(flag) = words.next() {
        if flag == "--edit" {
            edit = Some(words.next_if(|word| !word.starts_with("--")).map(PathBuf::from));
            continue;
        }
        let mut value = || words.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--seed" => {
                let text = value()?;
                seed = Some(text.parse::<u64>().map_err(|_| format!("--seed takes a number, not {text}"))?);
            }
            "--pool" => {
                pool = Some(match value()?.as_str() {
                    "performance" => Pool::Performance,
                    "stress" => Pool::Stress,
                    other => return Err(format!("--pool is performance or stress, not {other}")),
                });
            }
            "--players" => {
                let text = value()?;
                players = Some(text.parse::<usize>().ok().filter(|n| *n >= 2).ok_or_else(|| {
                    format!("--players takes the number of seats, 2 or more, not {text}")
                })?);
            }
            "--scenario" => scenario = Some(PathBuf::from(value()?)),
            other => return Err(format!("{other} is not a flag the dev GUI takes")),
        }
    }
    if let Some(file) = edit {
        if seed.is_some() || pool.is_some() || players.is_some() || scenario.is_some() {
            return Err("--edit opens a board in the editor, whose own seed and seats it shows; it takes no other flag".to_string());
        }
        return Ok(Start::Edit(file));
    }
    if scenario.is_some() && pool.is_some() {
        return Err("--pool picks a dealt game's cards, and a scenario names its own".to_string());
    }
    if scenario.is_some() && players.is_some() {
        return Err("--players deals a game's seats, and a scenario states its own".to_string());
    }
    let pool = pool.unwrap_or(Pool::Performance);
    let players = players.unwrap_or(2);
    // A scenario's own seed is its file's, read as each game starts.
    let seed = if scenario.is_none() { Some(seed.unwrap_or_else(clock)) } else { seed };
    Ok(Start::Game(GameSetup { seed, pool, players, scenario }))
}

#[cfg(test)]
mod tests {
    use crate::boards::Folders;

    use super::*;

    fn words(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_string).collect()
    }

    fn launched(line: &str) -> GameSetup {
        match read(&words(line), || 41) {
            Ok(Start::Game(setup)) => setup,
            other => panic!("{line}: {other:?}"),
        }
    }

    /// The header's line and the decision log's name, as a game from `setup`
    /// begins.
    fn begun(setup: &GameSetup) -> (String, PathBuf) {
        let start = setup.start().unwrap_or_else(|refusal| panic!("{refusal}"));
        (start_line(setup, &start), Folders::default().game_log(&start))
    }

    #[test]
    fn a_dealt_game_takes_its_seed_from_the_flag_or_else_the_clock() {
        let clocked = launched("");
        assert_eq!((clocked.seed, clocked.pool), (Some(41), Pool::Performance));
        assert_eq!(begun(&clocked), ("seed 41 · Performance pool".to_string(), PathBuf::from("logs").join("seed-41.log")));
        let seeded = launched("--pool stress --seed 7");
        assert_eq!((seeded.seed, seeded.pool, seeded.scenario), (Some(7), Pool::Stress, None));
        assert_eq!(seeded.players, 2, "two seats unless asked");
    }

    /// `fuzz_games --players N` deals N decks from the seed, and so does the
    /// window, so a fuzz game's printed seed deals that table here.
    #[test]
    fn a_dealt_game_takes_its_seats_from_the_flag() {
        let four = launched("--players 4 --seed 7");
        assert_eq!(four.players, 4);
        let line = "seed 7 · Performance pool · 4 players".to_string();
        assert_eq!(begun(&four), (line, PathBuf::from("logs").join("seed-7-players-4.log")));
    }

    /// A scenario's games are recorded in its board's folder
    /// (`setup-architecture.md` §7b, decision 3), each named for the seed it
    /// played: the flag's, or else the file's as the game starts, which
    /// launch leaves unread (`codebase-state.md` item 200).
    #[test]
    fn a_scenario_plays_at_its_own_seed_unless_the_flag_says_otherwise() {
        let path = std::env::temp_dir().join("devgui-launch-seed.scenario");
        std::fs::write(&path, "seed 5\nturn 2\n").unwrap();
        let own = launched(&format!("--scenario {}", path.display()));
        assert_eq!((own.seed, own.scenario.as_ref()), (None, Some(&path)));
        let (line, log) = begun(&own);
        assert_eq!(line, format!("scenario {} · seed 5", path.display()));
        assert_eq!(log, PathBuf::from("boards").join("devgui-launch-seed").join("seed-5.log"));
        assert_eq!(launched(&format!("--seed 9 --scenario {}", path.display())).seed, Some(9));
    }

    #[test]
    fn the_editor_opens_a_file_or_an_empty_board() {
        let edit = |line: &str| match read(&words(line), || 41) {
            Ok(Start::Edit(file)) => file,
            other => panic!("{line}: {other:?}"),
        };
        assert_eq!(edit("--edit"), None);
        assert_eq!(edit("--edit board.scenario"), Some(PathBuf::from("board.scenario")));
    }

    #[test]
    fn a_word_the_window_cannot_use_is_refused_with_what_to_change() {
        let refused = |line: &str| read(&words(line), || 41).err().unwrap_or_else(|| panic!("{line} was taken"));
        assert_eq!(refused("--seed seven"), "--seed takes a number, not seven");
        assert_eq!(refused("--seed"), "--seed needs a value");
        assert_eq!(refused("--pool weird"), "--pool is performance or stress, not weird");
        assert_eq!(refused("--seeds 7"), "--seeds is not a flag the dev GUI takes");
        assert_eq!(refused("--pool stress --scenario board.scenario"), "--pool picks a dealt game's cards, and a scenario names its own");
        assert_eq!(refused("--players 1"), "--players takes the number of seats, 2 or more, not 1");
        assert_eq!(refused("--players four"), "--players takes the number of seats, 2 or more, not four");
        assert_eq!(refused("--players 4 --scenario board.scenario"), "--players deals a game's seats, and a scenario states its own");
        assert!(refused("--edit --seed 3").starts_with("--edit opens a board in the editor"));
    }
}
