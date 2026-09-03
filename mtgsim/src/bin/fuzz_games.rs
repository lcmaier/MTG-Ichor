// Fuzz harness — runs N games of Random vs Random to surface panics and edge cases.
//
// Usage: cargo run --bin fuzz_games -- --games 100 --max-turns 200 --verbose
//        cargo run --bin fuzz_games -- --games 10 --dump-events events.log
//        cargo run --bin fuzz_games -- --seed 12345 --games 1 --verbose
//        cargo run --bin fuzz_games -- --games 200 --threads 1     (serial)
//        cargo run --bin fuzz_games -- --pool stress                (every card)
//        cargo run --bin fuzz_games -- --require "Cytoshape,Mirrorweave"
//
// `--require` is the *coverage* instrument, as `--pool` is the *cost* one. It
// forces one copy of each named card into every deck and reports casts,
// resolutions, the share of games each reached, and whether the board it met
// was a random one. Nothing about the deck is bent toward the required card:
// every deck's mana base can pay for anything (`random_deck`), which is what
// let the mode stop seeding a deck's colours from the card's (2026-09-03). An
// empty list changes nothing: every RNG draw it adds is guarded, so a
// `--require` run and a timing run come from the same binary without the
// first perturbing the second.
//
// **Read the `resolved` column, not `cast`.** `cast` counts `SpellCast` events
// and `resolved` counts departures from the stack, and a countered spell has
// the first without the second. The other direction is impossible — a spell
// cannot resolve without having been cast — and the harness checks it in
// every game, flag or no flag: see `uncast_resolutions`. `resolved > cast` in
// this table was how `codebase-state.md` item 16c was found.
//
// `--pool` picks the card pool. `performance` is the frozen 55 every recorded
// baseline was measured on and is the default, because an A/B against a pool
// that moved is not an A/B; `stress` is every registered card, which is what
// hunts panics and exercises effect interactions. See
// `plans/engineering-practices.md` § "Two card pools".
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
// **Everything outside the `=== Timing ===` block is byte-identical across runs
// at one seed.** That is what makes the three-run determinism check in
// `CLAUDE.md` a plain `diff` of two regions rather than a hunt for scattered
// timing lines, and it is why every duration-derived number — including the
// slowest game's seed, which is chosen *by* a duration — lives in that block.
//
// Timing is reported as a mean and a tail. `Time/game` is wall-clock divided by
// games, so it falls as workers are added; `CPU/game` is the mean of each game's
// own measured duration, so it *rises* — 89.8ms alone against 191.5ms with 16
// games in flight, because the layer walk is allocation-heavy and the workers
// contend for memory bandwidth rather than for cores.
//
// The tail is there because **the mean is the one statistic guaranteed to hide a
// performance cliff.** A card shape that makes the layer walk fall over moves
// the slowest game by orders of magnitude and a 50-game mean by about two
// percent. `CPU/turn` is the sharper of the two tails: the slowest *game* is
// usually just the longest game, whereas a slow *turn* is an anomaly whatever
// the game's length.
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
use mtgsim::types::ids::ObjectId;
use mtgsim::types::zones::{Zone, ZoneChangeCause};
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
    /// Which card pool to play.
    pool: CardPool,
    /// Card names every generated deck must contain, and whose casts and
    /// resolutions are counted and reported. Empty by default, and an empty
    /// list must change nothing — see `random_deck`.
    require: Vec<String>,
}

/// The two pools, and the reason there are two.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CardPool {
    /// The frozen 55. Comparable across phases; the default.
    Performance,
    /// Every registered card. Grows, so its numbers are only comparable to
    /// another run of the same tree.
    Stress,
}

impl CardPool {
    fn name(self) -> &'static str {
        match self {
            CardPool::Performance => "performance",
            CardPool::Stress => "stress",
        }
    }

    fn registry(self) -> CardRegistry {
        match self {
            CardPool::Performance => CardRegistry::performance_pool(),
            CardPool::Stress => CardRegistry::default_registry(),
        }
    }
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
        pool: CardPool::Performance,
        require: Vec::new(),
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
            "--require" | "-r" => {
                i += 1;
                if i < args.len() {
                    // Deduplicated, order preserved. A repeated name used to
                    // produce two report rows counting the same events, which
                    // double-counts anything summed off them, and forced two
                    // copies into the deck while the flag's own docs promise
                    // one. Names are matched exactly — `registry.create` is
                    // case-sensitive, and a near-miss is fatal below rather
                    // than silently required-nothing.
                    let mut names: Vec<String> = Vec::new();
                    for n in args[i].split(',').map(|n| n.trim()) {
                        if !n.is_empty() && !names.iter().any(|s| s == n) {
                            names.push(n.to_string());
                        }
                    }
                    result.require = names;
                }
            }
            "--pool" | "-p" => {
                i += 1;
                if i < args.len() {
                    result.pool = match args[i].as_str() {
                        "performance" => CardPool::Performance,
                        "stress" => CardPool::Stress,
                        other => {
                            eprintln!("Unknown pool {other:?} — want performance or stress");
                            std::process::exit(2);
                        }
                    };
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

/// How many of a deck's land slots are basic lands — one of each type.
///
/// Basics are what CR 305.7's "nonbasic" leaves alone, so a deck keeps one of
/// each for the contrast: under Blood Moon they are the only lands still making
/// anything but red. Every other land slot is nonbasic, below.
const BASIC_LANDS_PER_DECK: usize = 5;

/// How many of a deck's land slots are drawn from the registry's nonbasic
/// lands — the ten original duals and Everywhere, uniformly.
///
/// A flat constant over a static pool, not a mana-base model. Replace it with a
/// real picker when card breadth (Phase 8) gives it something to choose between.
const NONBASIC_LANDS_PER_DECK: usize = 5;

/// Build one 60-card deck.
///
/// 1. 36 nonland slots, drawn uniformly with repeats from **every** nonland the
///    registry holds. There is no colour filter (dropped 2026-09-03): the mana
///    base below can pay for anything, so a filter would only decide which
///    slice of the pool a card gets to meet.
/// 2. `NONBASIC_LANDS_PER_DECK` slots from the registry's nonbasic lands.
/// 3. `BASIC_LANDS_PER_DECK` basics, one of each type.
/// 4. Every remaining land slot is a land that taps for all five colours —
///    Everywhere, today. A pool with no such land falls back to basics.
///
/// Until 2026-09-03 a deck rolled one or two colours and drew only the nonlands
/// castable in them, because every land in the registry made one or two
/// colours and a random 36 over random basics would have cast nothing. That is
/// also why `--require` seeded a deck's colours from the required card's — which
/// put a forced `{1}{G}{U}` against a G/U board in every game and never against
/// a black, red or green card. An any-colour mana base removes both.
///
/// Deck construction is deliberately crude — this is a fuzz harness, not a
/// deckbuilder. It exists to produce a legal 60 that casts spells and attacks.
///
/// `required` is `--require`'s list, and **an empty list must leave this
/// function exactly as it was**: every RNG draw below is guarded so that the
/// stream, the deck and therefore every recorded number are unchanged when the
/// flag is absent. That is the property that lets a reachability run and a
/// timing run come from the same binary without the first contaminating the
/// second.
fn random_deck(
    registry: &CardRegistry,
    rng: &mut StdRng,
    required: &[Arc<CardData>],
) -> Vec<Arc<CardData>> {
    let build = |name: &str| registry.create(name).ok();
    let is_land = |card: &CardData| card.types.contains(&CardType::Land);

    let nonland_names: Vec<&str> = registry
        .card_names()
        .into_iter()
        .filter(|name| build(name).is_some_and(|card| !is_land(&card)))
        .collect();

    let mut deck: Vec<Arc<CardData>> = Vec::with_capacity(60);

    // 36 nonlands
    for _ in 0..36 {
        if nonland_names.is_empty() {
            break;
        }
        let name = nonland_names.choose(rng).unwrap();
        if let Some(card) = build(name) {
            deck.push(card);
        }
    }

    // One copy of each required **nonland** card, replacing a nonland slot
    // rather than adding to the deck — 60 cards stays 60, so mana density and
    // the draw curve are untouched. Deterministic (the first slots, no RNG
    // draw) because the library is shuffled in-game anyway, and because a draw
    // here would move the stream for every later card.
    //
    // A required *land* is held back to the land section below, for the same
    // reason: spending a nonland slot on it would quietly make the deck
    // 35/25 and move every number the pool exists to measure.
    let mut required_lands: Vec<&Arc<CardData>> = Vec::new();
    let mut slot = 0usize;
    for card in required {
        if is_land(card) {
            required_lands.push(card);
            continue;
        }
        if slot < deck.len() {
            deck[slot] = card.clone();
        } else {
            deck.push(card.clone());
        }
        slot += 1;
    }

    // Pad remaining nonland slots with lands if card pool is too small
    let nonland_count = deck.len();
    let land_count = 60 - nonland_count;

    let nonbasic_names: Vec<&str> = registry
        .card_names()
        .into_iter()
        .filter(|name| {
            build(name)
                .is_some_and(|card| is_land(&card) && !card.supertypes.contains(&Supertype::Basic))
        })
        .collect();

    // Nonbasic lands first: `NONBASIC_LANDS_PER_DECK` draws whatever `required`
    // holds, so the RNG stream does not depend on the flag. A required land goes
    // in ahead of them and takes its slot from the any-colour fill at the end,
    // so the land count does not depend on it either.
    let mut lands_added = 0usize;
    for card in &required_lands {
        if lands_added < land_count {
            deck.push((*card).clone());
            lands_added += 1;
        }
    }
    if !nonbasic_names.is_empty() {
        for _ in 0..NONBASIC_LANDS_PER_DECK.min(land_count) {
            let name = nonbasic_names.choose(rng).unwrap();
            if let Some(card) = build(name) {
                deck.push(card);
                lands_added += 1;
            }
        }
    }

    // One basic of each type.
    let basics = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
    for i in 0..BASIC_LANDS_PER_DECK.min(land_count.saturating_sub(lands_added)) {
        if let Some(card) = build(color_to_land(basics[i % basics.len()])) {
            deck.push(card);
            lands_added += 1;
        }
    }

    // Everything else taps for any colour.
    let any_colour: Vec<&str> = nonbasic_names
        .iter()
        .copied()
        .filter(|name| build(name).is_some_and(|card| land_mana_colors(&card).len() == 5))
        .collect();
    let mut i = 0usize;
    while lands_added < land_count {
        let name = if any_colour.is_empty() {
            color_to_land(basics[i % basics.len()])
        } else {
            any_colour[i % any_colour.len()]
        };
        if let Some(card) = build(name) {
            deck.push(card);
        }
        lands_added += 1;
        i += 1;
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
    // --- engine work (state/diagnostics.rs), not read off the event log ---
    layer_walks: u64,
    layer_frames: u64,
    replacement_gathers: u64,
    restriction_queries: u64,
    /// `--require` reachability: `(name, cast, resolved)`, in the order the
    /// flag listed them. Empty unless the flag is set.
    ///
    /// **Resolved, not cast, is the number that answers the question.** A cast
    /// spell that is countered never runs the engine path the phase built; the
    /// point of this mode is "was the path walked", so both are reported and
    /// the gap between them is itself information. A required land has no
    /// cast; its land drops fill both columns.
    reach: Vec<(String, u32, u32)>,
    /// `--require` board diversity: a permanent of a colour no required card
    /// has entered the battlefield this game. False unless the flag is set.
    ///
    /// **This is what dropping the colour seeding bought, and the resolution
    /// count cannot see it.** A forced card that resolves 390 times against
    /// the same colour pair has walked its path and met nothing; this says
    /// whether the board it met was the pool's or a hand-picked slice of it.
    met_another_colour: bool,
}

/// Extract action statistics from raw GameEvents.
///
/// `game`, `watch` and `required_colours` are only read when `--require` named
/// cards: resolving an `ObjectId` to a card name needs the object store, and
/// doing it per event on every run would be a `HashMap` probe the default path
/// does not owe.
fn extract_stats<'a>(
    events: impl Iterator<Item = &'a GameEvent>,
    game: &mtgsim::state::game_state::GameState,
    watch: &[String],
    required_colours: &std::collections::HashSet<Color>,
) -> GameStats {
    let mut stats = GameStats::default();
    stats.reach = watch.iter().map(|n| (n.clone(), 0, 0)).collect();

    // Printed name, not the effective one: this asks "did the card the flag
    // named get cast", which is a question about the card, and CV-1 is the
    // phase that makes a permanent's effective name something else entirely.
    let named = |id: mtgsim::types::ids::ObjectId| -> Option<&str> {
        game.objects.get(&id).map(|o| o.card_data.name.as_str())
    };
    let bump = |stats: &mut GameStats, name: Option<&str>, cast: bool| {
        let Some(name) = name else { return };
        for row in stats.reach.iter_mut() {
            if row.0 == name {
                if cast { row.1 += 1 } else { row.2 += 1 }
            }
        }
    };

    for event in events {
        match event {
            GameEvent::SpellCast { spell_id, .. } => {
                stats.spells_cast += 1;
                if !watch.is_empty() {
                    bump(&mut stats, named(*spell_id), true);
                }
            }

            GameEvent::DamageDealt { amount, .. } => {
                stats.damage_events += 1;
                stats.total_damage += amount;
            }
            GameEvent::LifeChanged { .. } => stats.life_changes += 1,
            GameEvent::AttackersDeclared { attackers } if !attackers.is_empty() => {
                stats.combat_phases_with_attackers += 1;
            }
            GameEvent::PermanentEnteredBattlefield { object_id, .. } => {
                // Printed colours, deliberately, for the reason `named` reads
                // the printed name: the question is which *cards* the required
                // one met, and a copy's effective colour is its donor's.
                if !watch.is_empty() && !stats.met_another_colour {
                    let printed = game.objects.get(object_id).map(|o| &o.card_data.colors);
                    if printed.is_some_and(|cs| cs.iter().any(|c| !required_colours.contains(c))) {
                        stats.met_another_colour = true;
                    }
                }
            }
            GameEvent::ZoneChange { object_id, from, to, cause, lki, .. } => {
                use mtgsim::types::zones::Zone;
                // CR 608 — a spell finishes resolving by leaving the stack.
                // An instant or sorcery goes to the graveyard; a permanent
                // spell goes to the battlefield. Both are the path having run.
                if !watch.is_empty()
                    && *from == Zone::Stack
                    && matches!(
                        cause,
                        mtgsim::types::zones::ZoneChangeCause::Resolved
                    )
                {
                    bump(&mut stats, named(*object_id), false);
                }
                if *from == Zone::Hand && *to == Zone::Battlefield {
                    // This counts land plays (spells go Hand→Stack→Battlefield)
                    stats.lands_played += 1;
                    // A required *land* is played, not cast, so its play is
                    // its `cast` and its arrival its `resolved` — otherwise a
                    // land the flag named reads as never resolved.
                    if !watch.is_empty() {
                        bump(&mut stats, named(*object_id), true);
                        bump(&mut stats, named(*object_id), false);
                    }
                }
                // Deaths, read off the zone change rather than a type-specific
                // event. **This is why the number moved** (5.3 → 6.2 at
                // --games 50 --seed 12345): `CreatureDied` was emitted only by
                // the state-based-action sweep, so a creature killed by a spell
                // never counted. The CR 603.10a frame says what it was.
                if *from == Zone::Battlefield && *to == Zone::Graveyard {
                    let was_creature = lki.as_ref().is_some_and(|f| {
                        f.types.contains(&mtgsim::types::card_types::CardType::Creature)
                    });
                    if was_creature {
                        stats.creatures_died += 1;
                    }
                }
            }
            _ => {}
        }
    }
    stats
}

/// CR 601.2i — nothing leaves the stack by resolving unless it was cast.
///
/// A spell reaches the stack at CR 601.2a and is announced at 601.2i, so every
/// object leaving the stack with `ZoneChangeCause::Resolved` has a `SpellCast`
/// behind it; an ability ceases to exist instead and emits no zone change. An
/// object with none was stranded on the stack by a cast that failed after
/// 601.2a and did not rewind — item 16c, which resolved about five spells per
/// game unpaid, in every fuzz run, because nothing asserted this. Reported the
/// way a panic is, since it is a wrong answer rather than a slow one.
///
/// Counted per object, not remembered per object: a card cast twice (bounced
/// and recast) owes two announcements, and `ObjectId` survives the round trip.
/// The event index rather than the id in the report, because ids are v4 UUIDs
/// and this line sits in the region that must be byte-identical across runs.
fn uncast_resolutions<'a>(
    events: impl Iterator<Item = &'a GameEvent>,
    game: &mtgsim::state::game_state::GameState,
) -> Vec<String> {
    let mut casts: HashMap<ObjectId, u32> = HashMap::new();
    let mut violations = Vec::new();
    for (index, event) in events.enumerate() {
        match event {
            GameEvent::SpellCast { spell_id, .. } => {
                *casts.entry(*spell_id).or_insert(0) += 1;
            }
            GameEvent::ZoneChange {
                object_id,
                from: Zone::Stack,
                cause: ZoneChangeCause::Resolved,
                ..
            } => {
                let owed = casts.entry(*object_id).or_insert(0);
                if *owed == 0 {
                    let name = game
                        .objects
                        .get(object_id)
                        .map_or("<unknown>", |o| o.card_data.name.as_str());
                    violations.push(format!("{name} at event {index}"));
                } else {
                    *owed -= 1;
                }
            }
            _ => {}
        }
    }
    violations
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
    total_layer_walks: u64,
    total_layer_frames: u64,
    total_replacement_gathers: u64,
    total_restriction_queries: u64,
    games_counted: u64,
    /// `(name, cast, resolved, games_in_which_it_resolved)`.
    reach: Vec<(String, u64, u64, u64)>,
    /// Games in which `GameStats::met_another_colour` was set.
    games_met_another_colour: u64,
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
        self.total_layer_walks += game.layer_walks;
        self.total_layer_frames += game.layer_frames;
        self.total_replacement_gathers += game.replacement_gathers;
        self.total_restriction_queries += game.restriction_queries;
        if self.reach.is_empty() {
            self.reach = game.reach.iter().map(|(n, _, _)| (n.clone(), 0, 0, 0)).collect();
        }
        for (i, (_, cast, resolved)) in game.reach.iter().enumerate() {
            if let Some(row) = self.reach.get_mut(i) {
                row.1 += *cast as u64;
                row.2 += *resolved as u64;
                if *resolved > 0 {
                    row.3 += 1;
                }
            }
        }
        if game.met_another_colour {
            self.games_met_another_colour += 1;
        }
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
/// One game's timing, kept rather than summed away so the tail can be reported.
///
/// `turns` is `None` for a game that errored or panicked: there is no turn count
/// to divide by, and a crashed game's "cost per turn" would be noise in the one
/// distribution that exists to surface anomalies.
struct GameTiming {
    seed: u64,
    ms: f64,
    turns: Option<u32>,
}

/// Nearest-rank percentile over an ascending slice; `p` is a fraction in `0..=1`.
///
/// Nearest rank rather than interpolation, because at the game counts this
/// harness runs an interpolated p99 would invent precision the sample does not
/// have. The visible consequence: **p99 equals max below 100 games.** At
/// `--games 50`, rank `ceil(0.99 * 50)` is 50 — the last element. Two equal
/// numbers there are not a bug, they are the sample saying it is too small to
/// separate them; `--games 200` is where p99 starts carrying its own signal.
fn percentile(ascending: &[f64], p: f64) -> f64 {
    if ascending.is_empty() {
        return 0.0;
    }
    let rank = ((p * ascending.len() as f64).ceil() as usize).clamp(1, ascending.len());
    ascending[rank - 1]
}

enum GameOutcome {
    Completed {
        result: Option<mtgsim::state::game::GameResult>,
        turns: u32,
        stats: GameStats,
        event_log: Option<Vec<String>>,
        /// `uncast_resolutions`' findings. The game still completed and still
        /// counts in every fixture row; this is reported beside it, as an
        /// error is, and fails the run the same way.
        violations: Vec<String>,
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
    required: &[Arc<CardData>],
    require_names: &[String],
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

    let deck1 = random_deck(registry, &mut deck_rng, required);
    let deck2 = random_deck(registry, &mut deck_rng, required);

    // The colours the required cards have between them, for the diversity
    // line: a permanent outside this set is one the seeding used to exclude.
    let required_colours: std::collections::HashSet<Color> =
        required.iter().flat_map(|c| c.colors.iter().copied()).collect();

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
            uncast_resolutions(game.state.events.events(), &game.state),
            {
                let mut s = extract_stats(
                    game.state.events.events(),
                    &game.state,
                    require_names,
                    &required_colours,
                );
                // Read once at the end of the game rather than accumulated per
                // turn: the counters are monotonic for the game's lifetime and
                // nothing resets them, so the final value *is* the total.
                let c = &game.state.counters;
                s.layer_walks = c.layer_walks();
                s.layer_frames = c.layer_frames();
                s.replacement_gathers = c.replacement_gathers();
                s.restriction_queries = c.restriction_queries();
                s
            },
        ))
    }));
    let elapsed = started.elapsed();

    let outcome = match result {
        Ok(Ok((result, turns, event_log, violations, stats))) => GameOutcome::Completed {
            result,
            turns,
            stats,
            event_log,
            violations,
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
#[allow(clippy::too_many_arguments)]
fn run_games(
    registry: &CardRegistry,
    master_seed: u64,
    games: usize,
    max_turns: u32,
    keep_event_log: bool,
    threads: usize,
    required: &[Arc<CardData>],
    require_names: &[String],
) -> Vec<(GameOutcome, std::time::Duration)> {
    if threads <= 1 || games <= 1 {
        return (0..games)
            .map(|n| {
                run_one_game(
                    registry, master_seed, n, max_turns, keep_event_log, required, require_names,
                )
            })
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
                                run_one_game(
                                    registry,
                                    master_seed,
                                    n,
                                    max_turns,
                                    keep_event_log,
                                    required,
                                    require_names,
                                ),
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

    // Named in the header and again in the results, because the stats below
    // mean nothing without it — the two pools are not comparable to each other.
    let registry = args.pool.registry();
    println!("Card pool: {} ({} cards)", args.pool.name(), registry.card_names().len());

    // Resolved once, up front, and **fatal on a miss**. A typo that silently
    // required nothing would report the same thin reachability the mode exists
    // to fix, which is the failure this whole flag is a response to.
    let required: Vec<Arc<CardData>> = args
        .require
        .iter()
        .map(|name| {
            registry.create(name).unwrap_or_else(|_| {
                // Two different mistakes want two different fixes: a typo
                // wants spelling, an unpooled card wants `--pool stress`.
                // One message for both sends the typo the wrong way.
                if CardRegistry::default_registry().create(name).is_ok() {
                    eprintln!(
                        "--require: '{}' is registered but not in the {} pool. \
                         Use --pool stress.",
                        name,
                        args.pool.name()
                    );
                } else {
                    eprintln!(
                        "--require: no registered card is named '{}' (matching is \
                         exact and case-sensitive).",
                        name
                    );
                }
                std::process::exit(2);
            })
        })
        .collect();
    if !required.is_empty() {
        println!(
            "Required in every deck: {}  (one copy each, in a nonland slot; the deck is otherwise random)",
            args.require.join(", ")
        );
    }
    println!();
    let start = Instant::now();

    let mut completed = 0u64;
    let mut panics = 0u64;
    let mut errors = 0u64;
    let mut uncast = 0u64;
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
        &required,
        &args.require,
    );

    // Reporting is a serial pass over the games in order, so every line printed
    // and every number accumulated is what the serial harness produced.
    let mut cpu_total = std::time::Duration::ZERO;
    let mut timings: Vec<GameTiming> = Vec::with_capacity(args.games);
    for (game_num, (outcome, game_time)) in outcomes.into_iter().enumerate() {
        cpu_total += game_time;
        let game_seed = master_seed.wrapping_add(game_num as u64);
        timings.push(GameTiming {
            seed: game_seed,
            ms: game_time.as_secs_f64() * 1000.0,
            turns: match &outcome {
                GameOutcome::Completed { turns, .. } => Some(*turns),
                _ => None,
            },
        });

        match outcome {
            GameOutcome::Completed { result: game_result, turns, stats, event_log, violations } => {
                completed += 1;
                if !violations.is_empty() {
                    uncast += violations.len() as u64;
                    println!(
                        "Game {:>4} (seed {:>12}): INVARIANT — {} resolved without a SpellCast: {}",
                        game_num + 1,
                        game_seed,
                        violations.len(),
                        violations.join(", ")
                    );
                }
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
    println!("Card pool:       {}", args.pool.name());
    println!("Games run:       {}", args.games);
    println!("Completed:       {}", completed);
    println!("Errors:          {}", errors);
    println!("Panics:          {}", panics);
    println!("Uncast resolved: {}", uncast);
    println!("Hit turn limit:  {}", hit_turn_limit);
    println!("Avg turns/game:  {:.1}", avg_turns);
    println!("Max turns seen:  {}", max_turns_seen);

    // Everything above this point is byte-identical across runs at one seed;
    // everything in this block is not. Keeping the boundary sharp is what lets
    // the three-run determinism check diff two regions instead of filtering
    // lines out of one.
    println!();
    println!("=== Timing ===");
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

    let mut game_ms: Vec<f64> = timings.iter().map(|t| t.ms).collect();
    game_ms.sort_by(|a, b| a.total_cmp(b));
    println!(
        "CPU/game tail:   {:.2} p50 / {:.2} p99 / {:.2} max  (ms)",
        percentile(&game_ms, 0.50),
        percentile(&game_ms, 0.99),
        percentile(&game_ms, 1.00),
    );

    // Completed games only, and only those that reached a turn: a game that
    // errored on turn 0 has no rate, and dividing by zero would put an infinity
    // at the top of the distribution this line exists to read.
    let mut turn_ms: Vec<f64> = timings
        .iter()
        .filter_map(|t| match t.turns {
            Some(turns) if turns > 0 => Some(t.ms / turns as f64),
            _ => None,
        })
        .collect();
    turn_ms.sort_by(|a, b| a.total_cmp(b));
    println!(
        "CPU/turn tail:   {:.2} p50 / {:.2} p99 / {:.2} max  (ms)",
        percentile(&turn_ms, 0.50),
        percentile(&turn_ms, 0.99),
        percentile(&turn_ms, 1.00),
    );

    // The handle that makes a tail number actionable: every game is a pure
    // function of its own seed, so the outlier replays alone.
    if let Some(slowest) = timings.iter().max_by(|a, b| a.ms.total_cmp(&b.ms)) {
        let turns = slowest
            .turns
            .map_or_else(|| "no turn count".to_string(), |t| format!("{t} turns"));
        println!(
            "Slowest game:    {:.2}ms over {} — repro: --seed {} --games 1",
            slowest.ms, turns, slowest.seed
        );
    }

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

        // Engine work, and it is a *fixture* like the block above rather than a
        // benchmark like `=== Timing ===`. Every number here is a pure function
        // of the seed and the card pool, so it is comparable across machines and
        // across months — which is exactly what `ms/game` is not, and why
        // `engineering-practices.md` §3 stores these and refuses to store that.
        //
        // **Read `Frames/walk` first.** It is the CR 613.7a existence re-check's
        // cost per layer walk, and it is the number that moves when the layer
        // system starts asking about other objects.
        println!();
        println!("=== Engine Work (avg per game) ===");
        println!("  Layer walks:      {:>8.0}", agg_stats.avg(agg_stats.total_layer_walks));
        println!("  Layer frames:     {:>8.0}", agg_stats.avg(agg_stats.total_layer_frames));
        println!(
            "  Frames/walk:      {:>8.2}",
            if agg_stats.total_layer_walks == 0 {
                0.0
            } else {
                agg_stats.total_layer_frames as f64 / agg_stats.total_layer_walks as f64
            }
        );
        println!("  Replacement gathers: {:>5.0}", agg_stats.avg(agg_stats.total_replacement_gathers));
        println!("  Restriction queries: {:>5.0}", agg_stats.avg(agg_stats.total_restriction_queries));
    }

    // `--require`'s answer, and the reason the mode exists: `PERFORMANCE_POOL`
    // is a *timing* instrument that had been asked to double as a coverage one.
    // These numbers say whether a path was walked; the ones above say what it
    // cost. Reading either off the other is how "the path is open" and "the
    // path is exercised" got confused in the first place.
    if !agg_stats.reach.is_empty() {
        println!();
        println!("=== Reachability (--require, {} games) ===", agg_stats.games_counted);
        for (name, cast, resolved, games) in &agg_stats.reach {
            let pct = if agg_stats.games_counted == 0 {
                0.0
            } else {
                *games as f64 * 100.0 / agg_stats.games_counted as f64
            };
            println!(
                "  {:<26} cast {:>4}  resolved {:>4}  in {:>4} games ({:.0}%)",
                name, cast, resolved, games, pct
            );
        }
        // Cast-but-not-resolved is a countered or fizzled spell, so the path the
        // phase built never ran. Called out rather than left to arithmetic.
        let dead: Vec<&str> = agg_stats
            .reach
            .iter()
            .filter(|(_, _, resolved, _)| *resolved == 0)
            .map(|(n, _, _, _)| n.as_str())
            .collect();
        if !dead.is_empty() {
            println!("  NEVER RESOLVED: {}", dead.join(", "));
        }
        // What the required cards met. Their colours are what the deck used to
        // be seeded with, so this is the share of games that would have had no
        // such permanent under the old mode — and the number the resolution
        // count cannot see.
        let required_colours: std::collections::HashSet<Color> =
            required.iter().flat_map(|c| c.colors.iter().copied()).collect();
        let mut letters: Vec<&str> = Vec::new();
        for (colour, letter) in [
            (Color::White, "W"),
            (Color::Blue, "U"),
            (Color::Black, "B"),
            (Color::Red, "R"),
            (Color::Green, "G"),
        ] {
            if required_colours.contains(&colour) {
                letters.push(letter);
            }
        }
        let pct = if agg_stats.games_counted == 0 {
            0.0
        } else {
            agg_stats.games_met_another_colour as f64 * 100.0 / agg_stats.games_counted as f64
        };
        println!(
            "  Board diversity: {} of {} games ({:.0}%) saw a permanent of a colour outside {{{}}}",
            agg_stats.games_met_another_colour,
            agg_stats.games_counted,
            pct,
            letters.join(""),
        );
    }

    if panics > 0 || errors > 0 || uncast > 0 {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        land_mana_colors, percentile, random_deck, BASIC_LANDS_PER_DECK,
        NONBASIC_LANDS_PER_DECK,
    };
    use mtgsim::cards::registry::CardRegistry;
    use mtgsim::objects::card_data::CardData;
    use mtgsim::types::card_types::{CardType, Supertype};
    use rand::{rngs::StdRng, SeedableRng};

    fn is_land(card: &CardData) -> bool {
        card.types.contains(&CardType::Land)
    }

    /// **The property the whole `--require` design rests on.** A reachability run
    /// and a timing run come from one binary, so an empty list has to leave the
    /// RNG stream and therefore the deck exactly as it was — otherwise every
    /// recorded number silently depends on whether the flag exists.
    #[test]
    fn an_empty_require_list_changes_no_deck() {
        let registry = CardRegistry::performance_pool();
        for seed in [1u64, 12345, 999] {
            let mut a = StdRng::seed_from_u64(seed);
            let mut b = StdRng::seed_from_u64(seed);
            // Two decks per game, so the check has to survive the second draw:
            // a stray RNG call in the first deck moves the second one too.
            for _ in 0..2 {
                let deck = random_deck(&registry, &mut a, &[]);
                let same = random_deck(&registry, &mut b, &[]);
                let names: Vec<&str> = deck.iter().map(|c| c.name.as_str()).collect();
                let same_names: Vec<&str> = same.iter().map(|c| c.name.as_str()).collect();
                assert_eq!(names, same_names, "seed {seed}");
                assert_eq!(deck.len(), 60);
            }
        }
    }

    /// A required **land** comes out of the nonbasic-land budget, not a nonland
    /// slot — otherwise requiring one would quietly make the deck 35/25 and move
    /// every number `PERFORMANCE_POOL` exists to measure.
    #[test]
    fn a_required_land_does_not_change_the_land_count() {
        use mtgsim::types::card_types::CardType;
        let registry = CardRegistry::performance_pool();
        let tundra = registry.create("Tundra").expect("in the performance pool");
        let mut with = StdRng::seed_from_u64(3);
        let mut without = StdRng::seed_from_u64(3);
        for _ in 0..10 {
            let a = random_deck(&registry, &mut with, std::slice::from_ref(&tundra));
            let b = random_deck(&registry, &mut without, &[]);
            let lands = |d: &[std::sync::Arc<mtgsim::objects::card_data::CardData>]| {
                d.iter().filter(|c| c.types.contains(&CardType::Land)).count()
            };
            assert_eq!(a.len(), 60);
            assert_eq!(lands(&a), lands(&b), "the land count is untouched");
            assert!(a.iter().any(|c| c.name == "Tundra"));
        }
    }

    /// A required card is in every deck and the deck is still 60 cards — it
    /// replaces a nonland slot rather than adding one, so mana density is
    /// untouched. Nothing else about the deck bends toward it: the mana base
    /// below is what makes inclusion mean *castable*.
    #[test]
    fn a_required_card_is_in_every_deck_and_the_deck_stays_sixty() {
        let registry = CardRegistry::performance_pool();
        let cytoshape = registry.create("Cytoshape").expect("in the performance pool");
        let mut rng = StdRng::seed_from_u64(7);
        for _ in 0..25 {
            let deck = random_deck(&registry, &mut rng, std::slice::from_ref(&cytoshape));
            assert_eq!(deck.len(), 60);
            assert!(
                deck.iter().any(|c| c.name == "Cytoshape"),
                "every deck contains the required card"
            );
        }
    }

    /// The mana base: one basic of each type, `NONBASIC_LANDS_PER_DECK` draws,
    /// and every other land slot taps for all five colours. That last tier is
    /// what lets a forced `{1}{G}{U}` be cast in a deck that never rolled green
    /// or blue — there is no roll any more — and it holds in both pools.
    #[test]
    fn every_deck_can_make_every_colour_and_keeps_a_basic_of_each_type() {
        for registry in [CardRegistry::performance_pool(), CardRegistry::default_registry()] {
            let mut rng = StdRng::seed_from_u64(11);
            for _ in 0..10 {
                let deck = random_deck(&registry, &mut rng, &[]);
                let lands: Vec<&CardData> =
                    deck.iter().map(|c| c.as_ref()).filter(|c| is_land(c)).collect();
                assert_eq!(lands.len(), 24, "36 nonlands, 24 lands");

                let basics: Vec<&str> = lands
                    .iter()
                    .filter(|c| c.supertypes.contains(&Supertype::Basic))
                    .map(|c| c.name.as_str())
                    .collect();
                assert_eq!(basics.len(), BASIC_LANDS_PER_DECK);
                for basic in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
                    assert!(basics.contains(&basic), "one {basic} in every deck");
                }

                let any_colour = lands.iter().filter(|c| land_mana_colors(c).len() == 5).count();
                assert!(
                    any_colour >= 24 - BASIC_LANDS_PER_DECK - NONBASIC_LANDS_PER_DECK,
                    "the fill tier taps for any colour; got {any_colour}"
                );
            }
        }
    }

    /// Nonlands are drawn from the whole pool, not a colour-filtered slice of
    /// it: a deck holds cards of more colours than any two-colour roll allowed.
    #[test]
    fn nonlands_are_drawn_from_the_whole_pool() {
        let registry = CardRegistry::performance_pool();
        let mut rng = StdRng::seed_from_u64(5);
        for _ in 0..10 {
            let deck = random_deck(&registry, &mut rng, &[]);
            let colours: std::collections::HashSet<_> = deck
                .iter()
                .filter(|c| !is_land(c))
                .flat_map(|c| c.colors.iter().copied())
                .collect();
            assert!(colours.len() >= 3, "36 draws from the whole pool span colours; got {colours:?}");
        }
    }

    #[test]
    fn percentile_uses_nearest_rank_and_never_indexes_past_the_end() {
        let xs: Vec<f64> = (1..=100).map(|n| n as f64).collect();
        assert_eq!(percentile(&xs, 0.50), 50.0);
        assert_eq!(percentile(&xs, 0.99), 99.0);
        assert_eq!(percentile(&xs, 1.00), 100.0, "p100 is the max, not an overrun");
    }

    #[test]
    fn p99_collapses_onto_max_below_a_hundred_samples() {
        // Documented in `percentile`'s comment and asserted here so nobody
        // "fixes" the duplicate reading at --games 50 by interpolating.
        let xs: Vec<f64> = (1..=50).map(|n| n as f64).collect();
        assert_eq!(percentile(&xs, 0.99), percentile(&xs, 1.00));
    }

    #[test]
    fn percentile_handles_the_degenerate_inputs() {
        assert_eq!(percentile(&[], 0.99), 0.0, "no games run");
        assert_eq!(percentile(&[7.0], 0.50), 7.0, "one game is every percentile");
        // p0 would round rank to 0; the clamp keeps it on the first element.
        assert_eq!(percentile(&[3.0, 9.0], 0.0), 3.0);
    }
}
