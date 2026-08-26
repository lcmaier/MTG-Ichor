// Fuzz harness — runs N games of Random vs Random to surface panics and edge cases.
//
// Usage: cargo run --bin fuzz_games -- --games 100 --max-turns 200 --verbose
//        cargo run --bin fuzz_games -- --games 10 --dump-events events.log
//        cargo run --bin fuzz_games -- --seed 12345 --games 1 --verbose
//        cargo run --bin fuzz_games -- --games 200 --threads 1     (serial)
//
// `--seed N` is a full reproducibility contract: the same seed replays the same
// games, turn for turn, in any process. Each game's three RNG streams — decks,
// shuffle, AI — are derived from `master_seed + game_num` and nothing else, and
// game `k` does not depend on games before it, so a run that panicked at game
// `k` can be reproduced from its printed per-game seed alone. Anything that
// leaks process state into a decision breaks this; see
// `GameState::battlefield_ordered` for the one that did.
//
// **That independence is also what makes the worker pool safe.** Games are
// distributed across threads by index, but every game's inputs are a pure
// function of `master_seed + game_num`, so which worker runs which game cannot
// change an outcome. Results are collected with their index and sorted back
// into game order before anything is printed or aggregated, so stdout is
// byte-identical at any `--threads` value — that is the acceptance test for
// this harness, and `--threads 1` is kept as the serial reference.
//
// Timing is reported twice. `Time/game` is wall-clock divided by games, so it
// falls as workers are added; `CPU/game` is the mean of each game's own
// measured duration, so it *rises* — 89.8ms alone against 191.5ms with 16
// games in flight, because the layer walk is allocation-heavy and the workers
// contend for memory bandwidth rather than for cores.
//
// **Which mode to use, measured (200 games / seed 12345, 10 runs each):**
//
// - **Coverage — hunting panics and errors — wants threads.** 17.8s -> 2.66s
//   at 16 workers, a 6.7x speedup, and precision does not matter for a
//   pass/fail sweep.
// - **Benchmarking wants `--threads 1`.** Threading inflates run-to-run CV from
//   2.4% to 6.1%, and matching a serial median-of-five would take roughly 32
//   threaded runs — which costs *more* wall time than the five serial ones.
//   Contention noise is not a fixed offset that cancels in an A/B.
//
// Scaling is sublinear and worth knowing before sizing a worker pool: on an
// 8-core/16-thread machine, 8 workers return 5.3x and 16 return 6.7x, and
// per-game cost inflates from two workers onward. Default to physical cores
// rather than logical ones. `codebase-state.md` carries the measured curve and
// what is and is not established about the cause.

use std::collections::HashMap;
use std::panic;
use std::sync::Arc;
use std::time::Instant;

use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::{Rng, SeedableRng};

use mtgsim::cards::registry::CardRegistry;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::types::card_types::{CardType, Supertype};
use mtgsim::types::colors::Color;
use mtgsim::types::mana::ManaSymbol;
use mtgsim::ui::random::RandomDecisionProvider;

/// CLI arguments (simple manual parsing, no external deps).
struct Args {
    games: usize,
    max_turns: u32,
    verbose: bool,
    /// If set, dump event logs for every game to this file path.
    dump_events: Option<String>,
    /// If set, use this seed for reproducibility.
    seed: Option<u64>,
    /// Worker threads. Defaults to the machine's parallelism; 1 is serial.
    threads: usize,
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    let mut result = Args {
        games: 100,
        max_turns: 200,
        verbose: false,
        dump_events: None,
        seed: None,
        threads: std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
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
            "--seed" | "-s" => {
                i += 1;
                if i < args.len() {
                    result.seed = Some(args[i].parse().unwrap_or(0));
                }
            }
            "--threads" | "-j" => {
                i += 1;
                if i < args.len() {
                    result.threads = args[i].parse().unwrap_or(1).max(1);
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

/// Map a Color to its corresponding basic land name.
fn color_to_land(color: Color) -> &'static str {
    match color {
        Color::White => "Plains",
        Color::Blue => "Island",
        Color::Black => "Swamp",
        Color::Red => "Mountain",
        Color::Green => "Forest",
    }
}

/// Extract the set of colors required by a card's mana cost.
fn card_colors(card: &CardData) -> Vec<Color> {
    let mut colors = Vec::new();
    if let Some(ref cost) = card.mana_cost {
        for sym in &cost.symbols {
            match sym {
                ManaSymbol::Colored(mt) => {
                    if let Some(c) = mana_type_to_color(*mt) {
                        if !colors.contains(&c) {
                            colors.push(c);
                        }
                    }
                }
                ManaSymbol::Hybrid(a, b) => {
                    for mt in [a, b] {
                        if let Some(c) = mana_type_to_color(*mt) {
                            if !colors.contains(&c) {
                                colors.push(c);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    // Also use card's color identity
    for c in &card.colors {
        if !colors.contains(c) {
            colors.push(*c);
        }
    }
    colors
}

fn mana_type_to_color(mt: mtgsim::types::mana::ManaType) -> Option<Color> {
    match mt {
        mtgsim::types::mana::ManaType::White => Some(Color::White),
        mtgsim::types::mana::ManaType::Blue => Some(Color::Blue),
        mtgsim::types::mana::ManaType::Black => Some(Color::Black),
        mtgsim::types::mana::ManaType::Red => Some(Color::Red),
        mtgsim::types::mana::ManaType::Green => Some(Color::Green),
        mtgsim::types::mana::ManaType::Colorless => None,
    }
}

/// How many of a deck's land slots are filled with nonbasic lands.
///
/// A flat constant over a static pool, not a mana-base model. Replace it with a
/// real picker when card breadth (Phase 8) gives it something to choose between;
/// today the entire nonbasic pool is the ten original duals.
const NONBASIC_LANDS_PER_DECK: usize = 5;

/// Generate a color-coherent 60-card deck.
///
/// 1. Pick 1-2 colors randomly.
/// 2. Select all nonland cards whose color requirements are a subset of the
///    chosen colors.
/// 3. Fill 36 nonland slots from that pool (with repeats).
/// 4. Fill up to `NONBASIC_LANDS_PER_DECK` land slots from the registry's
///    nonbasic lands, then the rest with basics.
///
/// Deck construction is deliberately crude — this is a fuzz harness, not a
/// deckbuilder. It exists to produce a legal 60 that casts spells and attacks.
fn random_deck(registry: &CardRegistry, rng: &mut StdRng) -> Vec<Arc<CardData>> {
    let all_colors = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];

    // Pick 1-2 colors
    let num_colors = if rng.random_bool(0.6) { 2 } else { 1 };
    let mut deck_colors: Vec<Color> = Vec::new();
    while deck_colors.len() < num_colors {
        let &c = all_colors.choose(rng).unwrap();
        if !deck_colors.contains(&c) {
            deck_colors.push(c);
        }
    }

    // Find nonland cards castable in these colors
    let nonland_names: Vec<String> = registry
        .card_names()
        .into_iter()
        .filter(|name| {
            if let Ok(card) = registry.create(name) {
                if card.types.contains(&CardType::Land) {
                    return false;
                }
                let required = card_colors(&card);
                // All required colors must be in our deck colors
                required.iter().all(|c| deck_colors.contains(c))
            } else {
                false
            }
        })
        .map(|s| s.to_string())
        .collect();

    let mut deck: Vec<Arc<CardData>> = Vec::with_capacity(60);

    // 36 nonlands
    for _ in 0..36 {
        if nonland_names.is_empty() {
            break;
        }
        let name = nonland_names.choose(rng).unwrap();
        if let Ok(card) = registry.create(name) {
            deck.push(card);
        }
    }

    // Pad remaining nonland slots with lands if card pool is too small
    let nonland_count = deck.len();
    let land_count = 60 - nonland_count;

    // Nonbasic lands first, then basics for whatever is left.
    //
    // Without these every land in every deck was a basic, which made Blood Moon
    // inert and left CR 305.7 with no random-play coverage at all. A land
    // qualifies if it produces at least one of the deck's colours — not a subset
    // test, deliberately: a two-colour deck's duals are all off by one colour
    // under a subset rule, and a mono-colour deck would get none at all. An
    // unusable second colour on a land costs nothing here.
    let nonbasic_names: Vec<String> = registry
        .card_names()
        .into_iter()
        .filter(|name| {
            let Ok(card) = registry.create(name) else { return false };
            if !card.types.contains(&CardType::Land) {
                return false;
            }
            if card.supertypes.contains(&Supertype::Basic) {
                return false;
            }
            let produced = land_mana_colors(&card);
            produced.iter().any(|c| deck_colors.contains(c))
        })
        .map(|s| s.to_string())
        .collect();

    let nonbasic_slots = NONBASIC_LANDS_PER_DECK.min(land_count);
    let mut nonbasics_added = 0;
    if !nonbasic_names.is_empty() {
        for _ in 0..nonbasic_slots {
            let name = nonbasic_names.choose(rng).unwrap();
            if let Ok(card) = registry.create(name) {
                deck.push(card);
                nonbasics_added += 1;
            }
        }
    }

    // Basics fill the rest, split evenly among colors.
    let basic_names: Vec<&str> = deck_colors.iter().map(|c| color_to_land(*c)).collect();
    for i in 0..(land_count - nonbasics_added) {
        let land_name = basic_names[i % basic_names.len()];
        if let Ok(card) = registry.create(land_name) {
            deck.push(card);
        }
    }

    deck
}

/// The colors of mana a land's abilities can produce.
///
/// Reads printed abilities on purpose — this runs before the game exists, so
/// there is no object to compute effective characteristics for. Deck
/// construction is a `// PRE-LAYER ZONE:` concern in the same sense as
/// `oracle::legality`'s hand queries.
fn land_mana_colors(card: &CardData) -> Vec<Color> {
    use mtgsim::types::effects::{Effect, Primitive};

    let mut colors = Vec::new();
    for ability in &card.abilities {
        let Effect::Atom(Primitive::ProduceMana(output), _) = &ability.effect else {
            continue;
        };
        for (mana_type, _) in &output.mana {
            if let Some(color) = mana_type_to_color(*mana_type) {
                if !colors.contains(&color) {
                    colors.push(color);
                }
            }
        }
    }
    colors
}

/// Per-game statistics extracted from the event log.
#[derive(Debug, Default)]
struct GameStats {
    spells_cast: u32,
    creatures_died: u32,
    damage_events: u32,
    total_damage: u64,
    lands_played: u32,
    combat_phases_with_attackers: u32,
    life_changes: u32,
}

/// Extract action statistics from raw GameEvents.
fn extract_stats<'a>(events: impl Iterator<Item = &'a GameEvent>) -> GameStats {
    let mut stats = GameStats::default();
    for event in events {
        match event {
            GameEvent::SpellCast { .. } => stats.spells_cast += 1,
            GameEvent::CreatureDied { .. } => stats.creatures_died += 1,
            GameEvent::DamageDealt { amount, .. } => {
                stats.damage_events += 1;
                stats.total_damage += amount;
            }
            GameEvent::LifeChanged { .. } => stats.life_changes += 1,
            GameEvent::AttackersDeclared { attackers } if !attackers.is_empty() => {
                stats.combat_phases_with_attackers += 1;
            }
            GameEvent::ZoneChange { from, to, .. } => {
                if *from == mtgsim::types::zones::Zone::Hand
                    && *to == mtgsim::types::zones::Zone::Battlefield
                {
                    // This counts land plays (spells go Hand→Stack→Battlefield)
                    stats.lands_played += 1;
                }
            }
            _ => {}
        }
    }
    stats
}

/// Aggregate statistics across all games.
#[derive(Debug, Default)]
struct AggregateStats {
    total_spells_cast: u64,
    total_creatures_died: u64,
    total_damage_events: u64,
    total_damage: u64,
    total_lands_played: u64,
    total_combat_with_attackers: u64,
    total_life_changes: u64,
    games_counted: u64,
}

impl AggregateStats {
    fn add(&mut self, game: &GameStats) {
        self.total_spells_cast += game.spells_cast as u64;
        self.total_creatures_died += game.creatures_died as u64;
        self.total_damage_events += game.damage_events as u64;
        self.total_damage += game.total_damage;
        self.total_lands_played += game.lands_played as u64;
        self.total_combat_with_attackers += game.combat_phases_with_attackers as u64;
        self.total_life_changes += game.life_changes as u64;
        self.games_counted += 1;
    }

    fn avg(&self, total: u64) -> f64 {
        if self.games_counted == 0 { 0.0 } else { total as f64 / self.games_counted as f64 }
    }
}

/// Append a game's event log to the dump file.
/// Append one game's formatted event log to `path`.
///
/// Called from the serial reporting pass, so blocks land in game order however
/// many workers produced them. Note when diffing two dumps: event lines carry
/// `ObjectId`s, which are v4 UUIDs and therefore differ between *processes*
/// regardless of seed or thread count. Mask them before comparing, or every
/// line looks changed and `diff` cannot align anything.
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

/// Everything one game produces that the reporting pass needs.
///
/// The point of collecting rather than printing is ordering: workers finish out
/// of order, so nothing is printed or aggregated until every game is back in
/// game order. `event_log` is `None` unless `--dump-events` asked for it — it is
/// the only large field, and formatting one per game and dropping it was
/// wasted work in the serial harness too.
enum GameOutcome {
    Completed {
        result: Option<mtgsim::state::game::GameResult>,
        turns: u32,
        stats: GameStats,
        event_log: Option<Vec<String>>,
    },
    Error {
        message: String,
        event_log: Option<Vec<String>>,
    },
    Panic {
        message: String,
    },
}

/// Run game `game_num`, catching a panic as a result rather than unwinding out.
///
/// Depends on nothing but its arguments — that is what lets the pool hand games
/// out in any order.
fn run_one_game(
    registry: &CardRegistry,
    master_seed: u64,
    game_num: usize,
    max_turns: u32,
    keep_event_log: bool,
) -> (GameOutcome, std::time::Duration) {
    // Derive per-game seed from master seed for reproducibility
    let game_seed = master_seed.wrapping_add(game_num as u64);
    let mut deck_rng = StdRng::seed_from_u64(game_seed);

    // Three independent streams, all a pure function of `game_seed`: deck
    // construction, the in-game shuffle, and the AI's choices. Distinct
    // sub-seeds rather than one — seeding three `StdRng`s identically would
    // correlate the shuffle with the deck it shuffles.
    let shuffle_seed = game_seed ^ 0x9E37_79B9_7F4A_7C15;
    let dp_seed = game_seed ^ 0xD1B5_4A32_D192_ED03;

    let deck1 = random_deck(registry, &mut deck_rng);
    let deck2 = random_deck(registry, &mut deck_rng);

    let started = Instant::now();
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let config = GameConfig::test();
        let mut game = Game::new(config, vec![deck1, deck2]).expect("Failed to create game");
        game.reseed(shuffle_seed);
        let dp = RandomDecisionProvider::seeded(dp_seed);
        game.setup(&dp).expect("Failed to setup game");

        let mut turns = 0u32;

        while !game.is_over() && turns < max_turns {
            if let Err(e) = game.run_turn(&dp) {
                return Err((
                    format!("Turn {} error: {}", turns, e),
                    if keep_event_log { Some(game.event_log_snapshot()) } else { None },
                ));
            }
            turns += 1;
        }

        Ok((
            game.result.clone(),
            turns,
            if keep_event_log { Some(game.event_log_snapshot()) } else { None },
            extract_stats(game.state.events.events()),
        ))
    }));
    let elapsed = started.elapsed();

    let outcome = match result {
        Ok(Ok((result, turns, event_log, stats))) => GameOutcome::Completed {
            result,
            turns,
            stats,
            event_log,
        },
        Ok(Err((message, event_log))) => GameOutcome::Error { message, event_log },
        Err(panic_info) => GameOutcome::Panic {
            message: if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown panic".to_string()
            },
        },
    };
    (outcome, elapsed)
}

/// Run `games` games across `threads` workers and return them in game order.
///
/// A shared index rather than a fixed split, so one long game does not leave a
/// worker idle while another still has a queue. Each worker keeps its own
/// results and they are merged and sorted at the end — no locking on the hot
/// path, and the sort is what guarantees the output is thread-count
/// independent.
fn run_games(
    registry: &CardRegistry,
    master_seed: u64,
    games: usize,
    max_turns: u32,
    keep_event_log: bool,
    threads: usize,
) -> Vec<(GameOutcome, std::time::Duration)> {
    if threads <= 1 || games <= 1 {
        return (0..games)
            .map(|n| run_one_game(registry, master_seed, n, max_turns, keep_event_log))
            .collect();
    }

    let next = std::sync::atomic::AtomicUsize::new(0);
    let mut collected: Vec<(usize, (GameOutcome, std::time::Duration))> =
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..threads.min(games))
                .map(|_| {
                    let next = &next;
                    scope.spawn(move || {
                        let mut mine = Vec::new();
                        loop {
                            let n = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if n >= games {
                                break;
                            }
                            mine.push((
                                n,
                                run_one_game(registry, master_seed, n, max_turns, keep_event_log),
                            ));
                        }
                        mine
                    })
                })
                .collect();

            handles
                .into_iter()
                .flat_map(|h| h.join().expect("worker thread panicked outside a game"))
                .collect()
        });

    collected.sort_by_key(|(n, _)| *n);
    collected.into_iter().map(|(_, outcome)| outcome).collect()
}

fn main() {
    let args = parse_args();

    // Determine master seed: explicit or random
    let master_seed = args.seed.unwrap_or_else(|| {
        rand::rng().random::<u64>()
    });

    println!("=== MTG Simulator Fuzz Harness ===");
    println!(
        "Running {} games, max {} turns each",
        args.games, args.max_turns
    );
    println!("Master seed: {} (reproduce with --seed {})", master_seed, master_seed);
    if args.threads > 1 {
        println!("Threads: {}", args.threads);
    }
    println!();

    let registry = CardRegistry::default_registry();
    let start = Instant::now();

    let mut completed = 0u64;
    let mut panics = 0u64;
    let mut errors = 0u64;
    let mut total_turns = 0u64;
    let mut max_turns_seen = 0u32;
    let mut hit_turn_limit = 0u64;
    let mut agg_stats = AggregateStats::default();
    let mut winner_counts: HashMap<String, u64> = HashMap::new();

    let outcomes = run_games(
        &registry,
        master_seed,
        args.games,
        args.max_turns,
        args.dump_events.is_some(),
        args.threads,
    );

    // Reporting is a serial pass over the games in order, so every line printed
    // and every number accumulated is what the serial harness produced.
    let mut cpu_total = std::time::Duration::ZERO;
    for (game_num, (outcome, game_time)) in outcomes.into_iter().enumerate() {
        cpu_total += game_time;
        let game_seed = master_seed.wrapping_add(game_num as u64);

        match outcome {
            GameOutcome::Completed { result: game_result, turns, stats, event_log } => {
                completed += 1;
                total_turns += turns as u64;
                if turns > max_turns_seen {
                    max_turns_seen = turns;
                }
                if turns >= args.max_turns {
                    hit_turn_limit += 1;
                }

                agg_stats.add(&stats);

                let result_str = match game_result {
                    Some(mtgsim::state::game::GameResult::Winner(pid)) => {
                        let key = format!("P{} wins", pid);
                        *winner_counts.entry(key.clone()).or_insert(0) += 1;
                        key
                    }
                    Some(mtgsim::state::game::GameResult::Draw) => {
                        *winner_counts.entry("Draw".to_string()).or_insert(0) += 1;
                        "Draw".to_string()
                    }
                    None => {
                        *winner_counts.entry("Turn limit".to_string()).or_insert(0) += 1;
                        "No result (turn limit)".to_string()
                    }
                };

                if args.verbose {
                    println!(
                        "Game {:>4} (seed {:>12}): {} in {:>3} turns | spells:{:>3} lands:{:>3} attacks:{:>3} deaths:{:>3} dmg:{:>4}",
                        game_num + 1,
                        game_seed,
                        result_str,
                        turns,
                        stats.spells_cast,
                        stats.lands_played,
                        stats.combat_phases_with_attackers,
                        stats.creatures_died,
                        stats.total_damage,
                    );
                }

                if let (Some(path), Some(log)) = (args.dump_events.as_ref(), event_log.as_ref()) {
                    dump_event_log(path, game_num, log);
                }
            }
            GameOutcome::Error { message, event_log } => {
                errors += 1;
                println!("Game {:>4} (seed {:>12}): ERROR — {}", game_num + 1, game_seed, message);
                if let (Some(path), Some(log)) = (args.dump_events.as_ref(), event_log.as_ref()) {
                    dump_event_log(path, game_num, log);
                }
            }
            GameOutcome::Panic { message } => {
                panics += 1;
                println!("Game {:>4} (seed {:>12}): PANIC — {}", game_num + 1, game_seed, message);
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
    println!("Master seed:     {}", master_seed);
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
    // Wall-clock/game falls as workers are added, so it measures the harness
    // rather than the engine. This is the mean of each game's own measured
    // duration: comparable across `--threads` values, and the number to
    // benchmark on.
    println!(
        "CPU/game:        {:.2}ms",
        cpu_total.as_secs_f64() * 1000.0 / args.games as f64
    );

    println!();
    println!("=== Outcomes ===");
    let mut outcomes: Vec<_> = winner_counts.iter().collect();
    // Name breaks ties: `winner_counts` is a `HashMap`, so equal counts would
    // otherwise print in whatever order this process's hasher produced.
    outcomes.sort_by_key(|(k, v)| (std::cmp::Reverse(**v), (*k).clone()));
    for (outcome, count) in &outcomes {
        println!("  {:<20} {:>5} ({:.1}%)", outcome, count, **count as f64 / args.games as f64 * 100.0);
    }

    if agg_stats.games_counted > 0 {
        println!();
        println!("=== Action Stats (avg per game) ===");
        println!("  Spells cast:      {:>6.1}", agg_stats.avg(agg_stats.total_spells_cast));
        println!("  Lands played:     {:>6.1}", agg_stats.avg(agg_stats.total_lands_played));
        println!("  Combat w/ atk:    {:>6.1}", agg_stats.avg(agg_stats.total_combat_with_attackers));
        println!("  Creatures died:   {:>6.1}", agg_stats.avg(agg_stats.total_creatures_died));
        println!("  Damage events:    {:>6.1}", agg_stats.avg(agg_stats.total_damage_events));
        println!("  Total damage:     {:>6.1}", agg_stats.avg(agg_stats.total_damage));
        println!("  Life changes:     {:>6.1}", agg_stats.avg(agg_stats.total_life_changes));
    }

    if panics > 0 || errors > 0 {
        std::process::exit(1);
    }
}
