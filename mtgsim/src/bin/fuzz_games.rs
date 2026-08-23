// Fuzz harness — runs N games of Random vs Random to surface panics and edge cases.
//
// Usage: cargo run --bin fuzz_games -- --games 100 --max-turns 200 --verbose
//        cargo run --bin fuzz_games -- --games 10 --dump-events events.log
//        cargo run --bin fuzz_games -- --seed 12345 --games 1 --verbose
//
// `--seed N` is a full reproducibility contract: the same seed replays the same
// games, turn for turn, in any process. Each game's three RNG streams — decks,
// shuffle, AI — are derived from `master_seed + game_num` and nothing else, and
// game `k` does not depend on games before it, so a run that panicked at game
// `k` can be reproduced from its printed per-game seed alone. Anything that
// leaks process state into a decision breaks this; see
// `GameState::battlefield_ordered` for the one that did.

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
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    let mut result = Args {
        games: 100,
        max_turns: 200,
        verbose: false,
        dump_events: None,
        seed: None,
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
fn extract_stats(events: &[GameEvent]) -> GameStats {
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

    for game_num in 0..args.games {
        // Derive per-game seed from master seed for reproducibility
        let game_seed = master_seed.wrapping_add(game_num as u64);
        let mut deck_rng = StdRng::seed_from_u64(game_seed);

        // Three independent streams, all a pure function of `game_seed`: deck
        // construction, the in-game shuffle, and the AI's choices. Distinct
        // sub-seeds rather than one — seeding three `StdRng`s identically would
        // correlate the shuffle with the deck it shuffles.
        let shuffle_seed = game_seed ^ 0x9E37_79B9_7F4A_7C15;
        let dp_seed = game_seed ^ 0xD1B5_4A32_D192_ED03;

        let deck1 = random_deck(&registry, &mut deck_rng);
        let deck2 = random_deck(&registry, &mut deck_rng);

        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let config = GameConfig::test();
            let mut game =
                Game::new(config, vec![deck1, deck2]).expect("Failed to create game");
            game.reseed(shuffle_seed);
            let dp = RandomDecisionProvider::seeded(dp_seed);
            game.setup(&dp).expect("Failed to setup game");

            let max = args.max_turns;
            let mut turns = 0u32;

            while !game.is_over() && turns < max {
                if let Err(e) = game.run_turn(&dp) {
                    return Err((
                        format!("Turn {} error: {}", turns, e),
                        game.event_log_snapshot(),
                        game.state.events.events().to_vec(),
                    ));
                }
                turns += 1;
            }

            Ok((
                game.result.clone(),
                turns,
                game.event_log_snapshot(),
                game.state.events.events().to_vec(),
            ))
        }));

        match result {
            Ok(Ok((game_result, turns, event_log, raw_events))) => {
                completed += 1;
                total_turns += turns as u64;
                if turns > max_turns_seen {
                    max_turns_seen = turns;
                }
                if turns >= args.max_turns {
                    hit_turn_limit += 1;
                }

                let stats = extract_stats(&raw_events);
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

                if let Some(ref path) = args.dump_events {
                    dump_event_log(path, game_num, &event_log);
                }
            }
            Ok(Err((e, event_log, _raw_events))) => {
                errors += 1;
                println!("Game {:>4} (seed {:>12}): ERROR — {}", game_num + 1, game_seed, e);
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
                println!("Game {:>4} (seed {:>12}): PANIC — {}", game_num + 1, game_seed, msg);
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
