//! The command line, read into the game the window starts with.
//!
//! `cargo run -- [--seed N] [--pool performance|stress] [--scenario FILE]`
//!
//! The seed defaults to the scenario's own, or else to the clock. The header
//! shows it and the decision log records it, so any game can be played again.

use std::path::PathBuf;

use mtgsim::scenario::Scenario;

use crate::bridge::{GameSetup, Pool};

pub const USAGE: &str = "usage: devgui [--seed N] [--pool performance|stress] [--scenario FILE]";

/// What the window starts with.
#[derive(Clone, Debug)]
pub struct Launch {
    pub setup: GameSetup,
    /// The seed, the start and the decision log's path, for the header.
    pub setup_line: String,
}

/// `args`, the words after the program's name, read into a [`Launch`];
/// `clock` seeds a dealt game given no `--seed`. An `Err` says what to
/// change, for the terminal.
pub fn read(args: &[String], clock: impl FnOnce() -> u64) -> Result<Launch, String> {
    let (mut seed, mut pool, mut scenario) = (None, None, None);
    let mut words = args.iter();
    while let Some(flag) = words.next() {
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
            "--scenario" => scenario = Some(PathBuf::from(value()?)),
            other => return Err(format!("{other} is not a flag the dev GUI takes")),
        }
    }
    if scenario.is_some() && pool.is_some() {
        return Err("--pool picks a dealt game's cards, and a scenario names its own".to_string());
    }
    let pool = pool.unwrap_or(Pool::Performance);
    let seed = match (seed, &scenario) {
        (Some(seed), _) => seed,
        // A file that does not parse shows its refusal in the window instead.
        (None, Some(path)) => {
            std::fs::read_to_string(path).ok().and_then(|text| Scenario::parse(&text).ok()).map_or(0, |s| s.seed)
        }
        (None, None) => clock(),
    };
    let (log_name, start) = match &scenario {
        Some(path) => {
            let stem = path.file_stem().map_or("scenario".into(), |s| s.to_string_lossy());
            (format!("{stem}-seed-{seed}.log"), format!("scenario {} · seed {seed}", path.display()))
        }
        None => (format!("seed-{seed}.log"), format!("seed {seed} · {pool:?} pool")),
    };
    let log_path = PathBuf::from("logs").join(log_name);
    let setup_line = format!("{start} · decision log {}", log_path.display());
    Ok(Launch { setup: GameSetup { seed, pool, log_path: Some(log_path), scenario }, setup_line })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_string).collect()
    }

    fn launched(line: &str) -> Launch {
        read(&words(line), || 41).unwrap_or_else(|problem| panic!("{line}: {problem}"))
    }

    #[test]
    fn a_dealt_game_takes_its_seed_from_the_flag_or_else_the_clock() {
        let clocked = launched("");
        assert_eq!((clocked.setup.seed, clocked.setup.pool), (41, Pool::Performance));
        let log = PathBuf::from("logs").join("seed-41.log");
        assert_eq!(clocked.setup_line, format!("seed 41 · Performance pool · decision log {}", log.display()));
        let seeded = launched("--pool stress --seed 7");
        assert_eq!((seeded.setup.seed, seeded.setup.pool, seeded.setup.scenario), (7, Pool::Stress, None));
    }

    #[test]
    fn a_scenario_plays_at_its_own_seed_unless_the_flag_says_otherwise() {
        let path = std::env::temp_dir().join("devgui-launch-seed.scenario");
        std::fs::write(&path, "seed 5\nturn 2\n").unwrap();
        let own = launched(&format!("--scenario {}", path.display()));
        assert_eq!((own.setup.seed, own.setup.scenario.as_ref()), (5, Some(&path)));
        assert_eq!(own.setup.log_path, Some(PathBuf::from("logs").join("devgui-launch-seed-seed-5.log")));
        assert_eq!(launched(&format!("--seed 9 --scenario {}", path.display())).setup.seed, 9);
    }

    #[test]
    fn a_word_the_window_cannot_use_is_refused_with_what_to_change() {
        let refused = |line: &str| read(&words(line), || 41).err().unwrap_or_else(|| panic!("{line} was taken"));
        assert_eq!(refused("--seed seven"), "--seed takes a number, not seven");
        assert_eq!(refused("--seed"), "--seed needs a value");
        assert_eq!(refused("--pool weird"), "--pool is performance or stress, not weird");
        assert_eq!(refused("--seeds 7"), "--seeds is not a flag the dev GUI takes");
        assert_eq!(refused("--pool stress --scenario board.scenario"), "--pool picks a dealt game's cards, and a scenario names its own");
    }
}
