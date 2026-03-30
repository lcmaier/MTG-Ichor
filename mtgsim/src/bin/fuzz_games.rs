// Fuzz harness — runs N games of Random vs Random to surface panics and edge cases.
//
// Usage: cargo run --bin fuzz_games -- --games 100 --max-turns 200 --verbose
//        cargo run --bin fuzz_games -- --games 10 --dump-events events.log

use std::panic;
use std::sync::Arc;
use std::time::Instant;

use rand::seq::IndexedRandom;

use mtgsim::cards::registry::CardRegistry;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::types::card_types::CardType;
use mtgsim::ui::random::RandomDecisionProvider;

/// CLI arguments (simple manual parsing, no external deps).
struct Args {
    games: usize,
    max_turns: u32,
    verbose: bool,
    /// If set, dump event logs for every game to this file path.
    dump_events: Option<String>,
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    let mut result = Args {
        games: 100,
        max_turns: 200,
        verbose: false,
        dump_events: None,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--games" | "-n" => {
                i += 1;
                if i < args.len() {
                    result.games = args[i].parse().unwrap_or(100);
                }
            }
            "--max-turns" | "-t" => {
                i += 1;
                if i < args.len() {
                    result.max_turns = args[i].parse().unwrap_or(200);
                }
            }
            "--verbose" | "-v" => {
                result.verbose = true;
            }
            "--dump-events" | "-d" => {
                i += 1;
                if i < args.len() {
                    result.dump_events = Some(args[i].clone());
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
            }
        }
        i += 1;
    }

    result
}

/// Generate a random deck from the registry.
///
/// Mix of basic lands and spells/creatures. Aims for a roughly playable
/// distribution: ~40% lands, ~60% nonlands.
fn random_deck(registry: &CardRegistry) -> Vec<Arc<CardData>> {
    let mut rng = rand::rng();

    let lands = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
    let nonlands: Vec<&str> = registry
        .card_names()
        .into_iter()
        .filter(|name| {
            if let Ok(card) = registry.create(name) {
                !card.types.contains(&CardType::Land)
            } else {
                false
            }
        })
        .collect();

    let mut deck: Vec<Arc<CardData>> = Vec::with_capacity(40);

    // ~17 lands
    for _ in 0..17 {
        let land_name = lands.choose(&mut rng).unwrap();
        if let Ok(card) = registry.create(land_name) {
            deck.push(card);
        }
    }

    // ~23 nonlands (or more lands if no nonlands available)
    for _ in 0..23 {
        if nonlands.is_empty() {
            let land_name = lands.choose(&mut rng).unwrap();
            if let Ok(card) = registry.create(land_name) {
                deck.push(card);
            }
        } else {
            let name = nonlands.choose(&mut rng).unwrap();
            if let Ok(card) = registry.create(name) {
                deck.push(card);
            }
        }
    }

    deck
}

/// Append a game's event log to the dump file.
fn dump_event_log(path: &str, game_num: usize, events: &[String]) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("Failed to open event log dump file");

    writeln!(file, "=== Game {} ({} events) ===", game_num + 1, events.len()).ok();
    for (i, event) in events.iter().enumerate() {
        writeln!(file, "  {:>4}: {}", i, event).ok();
    }
    writeln!(file).ok();
}

fn main() {
    let args = parse_args();

    println!("=== MTG Simulator Fuzz Harness ===");
    println!(
        "Running {} games, max {} turns each",
        args.games, args.max_turns
    );
    println!();

    let registry = CardRegistry::default_registry();
    let start = Instant::now();

    let mut completed = 0u64;
    let mut panics = 0u64;
    let mut errors = 0u64;
    let mut total_turns = 0u64;
    let mut max_turns_seen = 0u32;
    let mut hit_turn_limit = 0u64;

    for game_num in 0..args.games {
        let deck1 = random_deck(&registry);
        let deck2 = random_deck(&registry);

        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let config = GameConfig::test();
            let mut game =
                Game::new(config, vec![deck1, deck2]).expect("Failed to create game");
            let dp = RandomDecisionProvider::new();
            game.setup(&dp).expect("Failed to setup game");

            let max = args.max_turns;
            let mut turns = 0u32;

            while !game.is_over() && turns < max {
                if let Err(e) = game.run_turn(&dp) {
                    return Err((format!("Turn {} error: {}", turns, e), game.event_log_snapshot()));
                }
                turns += 1;
            }

            Ok((game.result.clone(), turns, game.event_log_snapshot()))
        }));

        match result {
            Ok(Ok((game_result, turns, event_log))) => {
                completed += 1;
                total_turns += turns as u64;
                if turns > max_turns_seen {
                    max_turns_seen = turns;
                }
                if turns >= args.max_turns {
                    hit_turn_limit += 1;
                }

                if args.verbose {
                    let result_str = match game_result {
                        Some(mtgsim::state::game::GameResult::Winner(pid)) => {
                            format!("P{} wins", pid)
                        }
                        Some(mtgsim::state::game::GameResult::Draw) => "Draw".to_string(),
                        None => "No result (turn limit)".to_string(),
                    };
                    println!(
                        "Game {:>4}: {} in {} turns",
                        game_num + 1,
                        result_str,
                        turns
                    );
                }

                if let Some(ref path) = args.dump_events {
                    dump_event_log(path, game_num, &event_log);
                }
            }
            Ok(Err((e, event_log))) => {
                errors += 1;
                println!("Game {:>4}: ERROR — {}", game_num + 1, e);
                if let Some(ref path) = args.dump_events {
                    dump_event_log(path, game_num, &event_log);
                }
            }
            Err(panic_info) => {
                panics += 1;
                let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown panic".to_string()
                };
                println!("Game {:>4}: PANIC — {}", game_num + 1, msg);
            }
        }
    }

    let elapsed = start.elapsed();
    let avg_turns = if completed > 0 {
        total_turns as f64 / completed as f64
    } else {
        0.0
    };

    println!();
    println!("=== Results ===");
    println!("Games run:       {}", args.games);
    println!("Completed:       {}", completed);
    println!("Errors:          {}", errors);
    println!("Panics:          {}", panics);
    println!("Hit turn limit:  {}", hit_turn_limit);
    println!("Avg turns/game:  {:.1}", avg_turns);
    println!("Max turns seen:  {}", max_turns_seen);
    println!("Total time:      {:.2}s", elapsed.as_secs_f64());
    println!(
        "Time/game:       {:.2}ms",
        elapsed.as_millis() as f64 / args.games as f64
    );

    if panics > 0 || errors > 0 {
        std::process::exit(1);
    }
}
