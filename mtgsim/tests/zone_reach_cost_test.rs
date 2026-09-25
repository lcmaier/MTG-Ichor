//! What a continuous effect that reaches the hidden zones costs a Commander
//! board: `codebase-state.md` item 181's instrument, kept for the fix PR and
//! item 6's readiness pass to read again.
//!
//! **A reading, not a gate**, so it is `#[ignore]`d. Release only, for the
//! reason `clone_bound_test.rs` gives:
//!
//! ```text
//! cargo test --release --test zone_reach_cost_test -- --ignored --nocapture
//! ```
//!
//! **Three arms play every game.** Right after setup, player 0 gets a blank
//! artifact, or one of `phase_lj_cards`' two fixtures in its place: Teferi's
//! clause, whose filter matches one seat's creature cards, and Lattice's,
//! which matches every card. Nothing else differs, so a seed is one game in
//! all three arms until the row changes a choice, and table 1 says where that
//! happened. The boards are `close_out.py`'s 20 `performance` games and the
//! clone test's three `stress` seeds, at Commander scale and dealt as
//! `fuzz_games` deals them, with no event recording, as in the clone test.
//!
//! One table per question item 181 asks:
//! 1. floor 1: board walks, layer frames and engine time per decision;
//! 2. floors 2 and 3: a clone's allocations, bytes and time at item 143's
//!    checkpoints;
//! 3. `backlog.md` §2.9's ceiling: the naive redeal, and the first decision
//!    after it with the memo warm and cold;
//! 4. how many members a zone row adds to each pass, and how many of those
//!    Teferi's filter matches.

use std::cell::{Cell, RefCell};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use mtgsim::cards::phase_lj_cards::{lattice_colorless_clause, teferi_flash_clause};
use mtgsim::cards::random_deck::random_deck;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameState, PhaseType, StepType};
use mtgsim::test_support::put_on_battlefield;
use mtgsim::types::card_types::CardType;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaCost;
use mtgsim::types::zones::{Zone, ZoneSet};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

#[path = "support/counting_allocator.rs"]
mod counting_allocator;
use counting_allocator::clone_cost;

const PLAYERS: usize = 4;
const DECK_SIZE: usize = 100;
const LIFE: i64 = 40;
/// `fuzz_games`' turn limit.
const MAX_TURNS: u32 = 200;
/// Item 143's checkpoints: the turn about to be played. Table 2 reads the
/// state between two `run_turn` calls there; table 3 forks at the first
/// priority decision of that turn.
const CHECKPOINTS: [u32; 10] = [1, 10, 20, 30, 40, 50, 60, 80, 100, 150];
/// Table 3's "from mid-game on", which is where the bounded-state PR read its
/// 10–40 µs.
const MID_GAME: u32 = 10;
/// Timed rounds of every game, the arms interleaved; table 1 reads the median.
const ROUNDS: usize = 5;
/// Each fork's first decision is timed this many times, warm and cold in turn.
const FORK_REPS: usize = 7;
/// A clone's time, as the bounded-state PR read it: the median of five means.
const CLONE_MEANS: usize = 5;
const CLONES_PER_MEAN: usize = 400;
/// Floor 2's CI proxy and floor 3 (`engineering-practices.md` §3.1).
const MAX_ALLOCATIONS: u64 = 64;
const MAX_BYTES: u64 = 128 * 1024;
/// Floor 1, in decisions per second on one thread: an upper bound on the
/// loaded rate the floor is written in.
const FLOOR_1: f64 = 10_000.0;

struct Board {
    name: &'static str,
    registry: CardRegistry,
    seeds: Vec<u64>,
}

fn boards() -> Vec<Board> {
    vec![
        Board {
            name: "performance",
            registry: CardRegistry::performance_pool(),
            // `close_out.py`'s `--games 20 --seed 12345`.
            seeds: (12345..12365).collect(),
        },
        Board {
            name: "stress",
            registry: CardRegistry::default_registry(),
            seeds: vec![12345, 777, 12351],
        },
    ]
}

#[derive(Clone, Copy, PartialEq)]
enum Arm {
    Blank,
    Teferi,
    Lattice,
}

impl Arm {
    const ALL: [Arm; 3] = [Arm::Blank, Arm::Teferi, Arm::Lattice];

    fn name(self) -> &'static str {
        match self {
            Arm::Blank => "no row",
            Arm::Teferi => "Teferi's clause",
            Arm::Lattice => "Lattice's clause",
        }
    }

    /// The card player 0 gets. The blank is the fixtures' own shape, a {2}
    /// colorless artifact, with no ability.
    fn card(self) -> Arc<CardData> {
        match self {
            Arm::Blank => CardDataBuilder::new("Blank Artifact")
                .mana_cost(ManaCost::build(&[], 2))
                .card_type(CardType::Artifact)
                .build(),
            Arm::Teferi => teferi_flash_clause(),
            Arm::Lattice => lattice_colorless_clause(),
        }
    }
}

fn config() -> GameConfig {
    let mut config = GameConfig::test();
    config.starting_life = LIFE;
    config
}

/// `fuzz_games`' game `seed` at Commander scale, set up, with `arm`'s card
/// on the battlefield under player 0.
fn deal(board: &Board, seed: u64, arm: Arm) -> (Game, ManaWindowStop<RandomDecisionProvider>, ObjectId) {
    let mut deck_rng = StdRng::seed_from_u64(seed);
    let decks: Vec<Vec<Arc<CardData>>> = (0..PLAYERS)
        .map(|_| random_deck(&board.registry, &mut deck_rng, &[], 1, DECK_SIZE))
        .collect();
    let mut game = Game::new(config(), decks).expect("game creation");
    game.reseed(seed ^ 0x9E37_79B9_7F4A_7C15);
    let dp = ManaWindowStop::new(RandomDecisionProvider::seeded(seed ^ 0xD1B5_4A32_D192_ED03));
    game.setup(&dp).expect("setup");
    let card = put_on_battlefield(&mut game.state, arm.card(), 0);
    (game, dp, card)
}

/// The diagnostics rows table 1 reads, summed over a board's games.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
struct Counts {
    decisions: u64,
    board_walks: u64,
    layer_walks: u64,
    frames: u64,
    memo_hits: u64,
}

impl Counts {
    fn of(state: &GameState) -> Self {
        let d = &state.diagnostics;
        Counts {
            decisions: d.decisions(),
            board_walks: d.board_walks(),
            layer_walks: d.layer_walks(),
            frames: d.layer_frames(),
            memo_hits: d.memo_hits(),
        }
    }

    fn add(&mut self, other: Counts) {
        self.decisions += other.decisions;
        self.board_walks += other.board_walks;
        self.layer_walks += other.layer_walks;
        self.frames += other.frames;
        self.memo_hits += other.memo_hits;
    }

    fn per_decision(self, n: u64) -> f64 {
        n as f64 / self.decisions as f64
    }
}

/// One game's time: the whole turn loop, and the turns the arm's card was on
/// the battlefield from start to end, with their decisions.
#[derive(Clone, Copy, Default)]
struct Timed {
    total: Duration,
    card_on: Duration,
    card_on_decisions: u64,
}

/// One game to its end, timed turn by turn.
fn play_timed(board: &Board, seed: u64, arm: Arm) -> (Timed, Counts) {
    let (mut game, dp, card) = deal(board, seed, arm);
    let mut timed = Timed::default();
    let mut turns = 0;
    while !game.is_over() && turns < MAX_TURNS {
        let on_before = game.state.battlefield.contains_key(&card);
        let decisions = game.state.diagnostics.decisions();
        let started = Instant::now();
        game.run_turn(&dp).expect("turn");
        let elapsed = started.elapsed();
        timed.total += elapsed;
        if on_before && game.state.battlefield.contains_key(&card) {
            timed.card_on += elapsed;
            timed.card_on_decisions += game.state.diagnostics.decisions() - decisions;
        }
        turns += 1;
    }
    (timed, Counts::of(&game.state))
}

// ---------------------------------------------------------------------------
// The watched pass: divergence, the row's presence, forks
// ---------------------------------------------------------------------------

/// A branch point for table 3: the game at a priority decision, and the
/// provider that was about to answer it.
struct Fork {
    turn: u32,
    state: GameState,
    dp: RandomDecisionProvider,
}

/// The game's own provider stack, watched: every prompt's fingerprint, whether
/// the arm's card was on the battlefield when it was asked, and a fork at the
/// first priority decision of each checkpoint turn.
struct Watcher {
    stack: ManaWindowStop<RandomDecisionProvider>,
    card: ObjectId,
    prints: RefCell<Vec<(u64, u64)>>,
    prompts_with_card: Cell<u64>,
    forks: RefCell<Vec<Fork>>,
}

impl Watcher {
    /// Note the prompt about to be answered, and fork at it if table 3 wants it.
    fn watch(&self, game: &GameState, context: &ChoiceContext, options: usize) {
        if game.battlefield.contains_key(&self.card) {
            self.prompts_with_card.set(self.prompts_with_card.get() + 1);
        }
        let turn = game.turn_number;
        let forked = self.forks.borrow().last().is_some_and(|f| f.turn == turn);
        if CHECKPOINTS.contains(&turn) && !forked && options >= 2 && resumable(game, context) {
            self.forks.borrow_mut().push(Fork {
                turn,
                state: game.clone(),
                dp: self.stack.inner().clone(),
            });
        }
    }

    /// The prompt and its answer, keyed by the decisions made before it.
    fn print(&self, game: &GameState, prompt: std::fmt::Arguments) {
        let mut hasher = DefaultHasher::new();
        without_grant_rows(&std::fmt::format(prompt)).hash(&mut hasher);
        self.prints.borrow_mut().push((game.diagnostics.decisions(), hasher.finish()));
    }
}

/// `text` with every `AbilityId`'s `grant` blanked. A granted ability's id
/// carries its registry row's number, and a fixture's own row moves every
/// later row's number by one, where the blank arm registers none: the same
/// game, told apart by a counter.
fn without_grant_rows(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("grant: ") {
        let (head, tail) = rest.split_at(at + "grant: ".len());
        out.push_str(head);
        out.push('_');
        rest = tail.trim_start_matches(|c: char| c.is_ascii_digit());
    }
    out.push_str(rest);
    out
}

/// A prompt `Game::resume_turn_at_priority` can take over from — the fork
/// test's definition (`priority_fork_test.rs`, `is_resumable`).
fn resumable(game: &GameState, context: &ChoiceContext) -> bool {
    matches!(context.kind, ChoiceKind::PriorityAction)
        && game.priority_player == game.active_player
        && game.in_game(game.active_player)
        && !matches!(
            (game.phase.phase_type, game.phase.step),
            (PhaseType::Ending, Some(StepType::Cleanup))
        )
}

impl DecisionProvider for Watcher {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        self.watch(game, context, options.len());
        let answer = self.stack.pick_n(game, player, context, options, bounds);
        self.print(game, format_args!("{:?}{options:?}{bounds:?}{answer:?}", context.kind));
        answer
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.watch(game, context, 0);
        let answer = self.stack.pick_number(game, player, context, min, max);
        self.print(game, format_args!("{:?}{min}{max}{answer}", context.kind));
        answer
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.watch(game, context, 0);
        let answer = self.stack.allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs);
        self.print(game, format_args!("{:?}{total}{buckets:?}{per_bucket_mins:?}{answer:?}", context.kind));
        answer
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.watch(game, context, 0);
        let answer = self.stack.choose_ordering(game, player, context, items);
        self.print(game, format_args!("{:?}{items:?}{answer:?}", context.kind));
        answer
    }
}

/// A clone at one checkpoint: allocations, bytes, and µs.
struct CloneReading {
    allocations: u64,
    bytes: u64,
    micros: f64,
}

fn read_clone(state: &GameState) -> CloneReading {
    let (allocations, bytes) = clone_cost(state);
    let means: Vec<f64> = (0..CLONE_MEANS)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..CLONES_PER_MEAN {
                drop(std::hint::black_box(state.clone()));
            }
            started.elapsed().as_secs_f64() * 1e6 / CLONES_PER_MEAN as f64
        })
        .collect();
    CloneReading {
        allocations,
        bytes,
        micros: median(means),
    }
}

/// The objects a row reaching every zone but the battlefield adds to each
/// pass, and how many of them Teferi's filter matches: player 0's creature
/// cards. Printed types, since no row on these boards changes a type off the
/// battlefield, and reading them walks nothing.
fn off_battlefield(state: &GameState) -> (u64, u64) {
    let mut members = 0;
    let mut matched = 0;
    for zone in ZoneSet::EVERYWHERE_BUT_BATTLEFIELD.iter() {
        for id in state.zone_ids_ordered(zone) {
            let Some(obj) = state.objects.get(&id) else { continue };
            members += 1;
            if obj.owner == 0 && obj.card_data.types.contains(&CardType::Creature) {
                matched += 1;
            }
        }
    }
    (members, matched)
}

/// What the watched pass of one game read.
struct Watched {
    counts: Counts,
    prints: Vec<(u64, u64)>,
    prompts: u64,
    prompts_with_card: u64,
    clones: Vec<(String, CloneReading)>,
    members: Vec<(u64, u64)>,
    forks: Vec<Fork>,
}

fn play_watched(board: &Board, seed: u64, arm: Arm) -> Watched {
    let (mut game, stack, card) = deal(board, seed, arm);
    let watcher = Watcher {
        stack,
        card,
        prints: RefCell::new(Vec::new()),
        prompts_with_card: Cell::new(0),
        forks: RefCell::new(Vec::new()),
    };
    let mut clones = Vec::new();
    let mut members = Vec::new();
    let mut turns = 0;
    while !game.is_over() && turns < MAX_TURNS {
        if CHECKPOINTS.contains(&game.state.turn_number) {
            clones.push((format!("turn {}", game.state.turn_number), read_clone(&game.state)));
            members.push(off_battlefield(&game.state));
        }
        game.run_turn(&watcher).expect("turn");
        turns += 1;
    }
    clones.push((format!("end, {turns} turns"), read_clone(&game.state)));
    let prints = watcher.prints.into_inner();
    Watched {
        counts: Counts::of(&game.state),
        prompts: prints.len() as u64,
        prints,
        prompts_with_card: watcher.prompts_with_card.get(),
        clones,
        members,
        forks: watcher.forks.into_inner(),
    }
}

// ---------------------------------------------------------------------------
// Table 3: the naive redeal, and the first decision after it
// ---------------------------------------------------------------------------

/// `backlog.md` §2.34's naive redeal, as the bounded-state PR's probe made it:
/// the viewer's library shuffled, and each opponent's hand dealt again from
/// that hand and that library together, off `GameState.rng`. The zones of the
/// cards that moved are walk inputs, so the memo goes cold.
fn redeal(state: &mut GameState, viewer: PlayerId) {
    state.shuffle_library(viewer);
    let GameState { players, rng, objects, .. } = state;
    for (seat, player) in players.iter_mut().enumerate() {
        if seat == viewer {
            continue;
        }
        let dealt = player.hand.len();
        let mut cards: Vec<ObjectId> = player.hand.drain(..).chain(player.library.drain(..)).collect();
        cards.shuffle(rng);
        player.library = cards.split_off(dealt);
        player.hand = cards;
        for (ids, zone) in [(&player.hand, Zone::Hand), (&player.library, Zone::Library)] {
            for id in ids {
                objects.get_mut(id).expect("a dealt card exists").zone = zone;
            }
        }
    }
    state.bump_layer_epoch();
}

/// The game's own provider, noting the board walks, layer frames and time
/// spent before it is first asked.
struct FirstAsk {
    stack: ManaWindowStop<RandomDecisionProvider>,
    started: Instant,
    first: Cell<Option<(Duration, u64, u64)>>,
}

impl FirstAsk {
    fn ask(&self, game: &GameState) {
        if self.first.get().is_none() {
            let d = &game.diagnostics;
            self.first.set(Some((self.started.elapsed(), d.board_walks(), d.layer_frames())));
        }
    }
}

impl DecisionProvider for FirstAsk {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        self.ask(game);
        self.stack.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.ask(game);
        self.stack.pick_number(game, player, context, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.ask(game);
        self.stack.allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.ask(game);
        self.stack.choose_ordering(game, player, context, items)
    }
}

/// One timing of a fork's first decision: µs, board walks and layer frames
/// spent before it. The rest of the turn is played out and not timed.
fn first_decision(fork: &Fork, cold: bool) -> (f64, u64, u64) {
    let mut state = fork.state.clone();
    if cold {
        redeal(&mut state, fork.state.active_player);
    }
    let (walks, frames) = (state.diagnostics.board_walks(), state.diagnostics.layer_frames());
    let mut game = Game { state, config: config() };
    let dp = FirstAsk {
        stack: ManaWindowStop::new(fork.dp.clone()),
        started: Instant::now(),
        first: Cell::new(None),
    };
    game.resume_turn_at_priority(&dp).expect("resumed turn");
    let (elapsed, walks_then, frames_then) = dp.first.get().expect("the fork's own prompt is asked again");
    (elapsed.as_secs_f64() * 1e6, walks_then - walks, frames_then - frames)
}

/// Table 3's reading of one fork.
struct ForkReading {
    turn: u32,
    redeal_micros: f64,
    warm_micros: f64,
    cold_micros: f64,
    cold_walks: u64,
    cold_frames: u64,
}

fn read_fork(fork: &Fork) -> ForkReading {
    let mut warm = Vec::new();
    let mut cold = Vec::new();
    let mut redeals = Vec::new();
    let mut cold_work = (0, 0);
    for _ in 0..FORK_REPS {
        warm.push(first_decision(fork, false).0);
        let (micros, walks, frames) = first_decision(fork, true);
        cold.push(micros);
        cold_work = (walks, frames);
        let mut state = fork.state.clone();
        let started = Instant::now();
        redeal(&mut state, fork.state.active_player);
        redeals.push(started.elapsed().as_secs_f64() * 1e6);
    }
    ForkReading {
        turn: fork.turn,
        redeal_micros: median(redeals),
        warm_micros: median(warm),
        cold_micros: median(cold),
        cold_walks: cold_work.0,
        cold_frames: cold_work.1,
    }
}

// ---------------------------------------------------------------------------
// The reading
// ---------------------------------------------------------------------------

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// Everything one arm read on one board.
#[derive(Default)]
struct Reading {
    rounds: Vec<f64>,
    /// Per round: the time of the turns the card stayed on the battlefield.
    rounds_card_on: Vec<f64>,
    card_on_decisions: u64,
    counts: Counts,
    prompts: u64,
    prompts_with_card: u64,
    /// Per game: the decision at which the prompts first differ from the no-row
    /// arm's, and the game's decisions; `None` where they never do.
    divergence: Vec<(Option<u64>, u64)>,
    /// Per checkpoint: the game's seed, where in it, and the clone.
    clones: Vec<(u64, String, CloneReading)>,
    members: Vec<(u64, u64)>,
    forks: Vec<ForkReading>,
}

/// The decision, counted from 1, at which two games' prompts first differ.
fn first_difference(ours: &[(u64, u64)], theirs: &[(u64, u64)]) -> Option<u64> {
    let longer = if ours.len() > theirs.len() { ours } else { theirs };
    let at = ours.iter().zip(theirs).position(|(a, b)| a != b);
    match at {
        Some(i) => Some(ours[i].0 + 1),
        None if ours.len() != theirs.len() => Some(longer[ours.len().min(theirs.len())].0 + 1),
        None => None,
    }
}

#[test]
#[ignore = "a reading, not a gate: run with --release -- --ignored --nocapture"]
fn zone_reaching_row_cost_on_the_commander_board() {
    // As in the clone test: bytes vary with the id hasher's seed, which
    // `types::ids` reads once, when the first id map is built.
    // SAFETY: this binary's one test sets it before any other thread exists
    // that could read the environment.
    unsafe { std::env::set_var("MTGSIM_HASH_SEED", "1") };

    for board in boards() {
        let mut readings: Vec<Reading> = Arm::ALL.iter().map(|_| Reading::default()).collect();

        // Table 1's time: every game of the board per round, the arms taking
        // turns so a drift on the machine lands on all three.
        for _ in 0..ROUNDS {
            for (a, &arm) in Arm::ALL.iter().enumerate() {
                let mut total = Duration::ZERO;
                let mut card_on = Duration::ZERO;
                let mut card_on_decisions = 0;
                let mut counts = Counts::default();
                for &seed in &board.seeds {
                    let (timed, game_counts) = play_timed(&board, seed, arm);
                    total += timed.total;
                    card_on += timed.card_on;
                    card_on_decisions += timed.card_on_decisions;
                    counts.add(game_counts);
                }
                let reading = &mut readings[a];
                assert!(
                    reading.rounds.is_empty() || reading.counts == counts,
                    "{} / {}: a round read other counts — the games are not deterministic",
                    board.name,
                    arm.name()
                );
                reading.counts = counts;
                reading.card_on_decisions = card_on_decisions;
                reading.rounds.push(total.as_secs_f64());
                reading.rounds_card_on.push(card_on.as_secs_f64());
            }
        }

        // The watched pass, the no-row arm first so the others compare to it.
        let mut reference: Vec<Vec<(u64, u64)>> = Vec::new();
        for (a, &arm) in Arm::ALL.iter().enumerate() {
            let mut counts = Counts::default();
            for (g, &seed) in board.seeds.iter().enumerate() {
                let watched = play_watched(&board, seed, arm);
                counts.add(watched.counts);
                let reading = &mut readings[a];
                reading.prompts += watched.prompts;
                reading.prompts_with_card += watched.prompts_with_card;
                if arm == Arm::Blank {
                    reading.divergence.push((None, watched.counts.decisions));
                    reference.push(watched.prints);
                } else {
                    let at = first_difference(&watched.prints, &reference[g]);
                    reading.divergence.push((at, watched.counts.decisions));
                }
                reading.clones.extend(watched.clones.into_iter().map(|(at, r)| (seed, at, r)));
                reading.members.extend(watched.members);
                reading.forks.extend(watched.forks.iter().map(read_fork));
            }
            assert_eq!(
                counts,
                readings[a].counts,
                "{} / {}: watching the game changed it",
                board.name,
                arm.name()
            );
        }

        report(&board, &readings);
    }
}

fn report(board: &Board, readings: &[Reading]) {
    let blank = &readings[0];
    let blank_micros = median(blank.rounds.clone()) * 1e6 / blank.counts.decisions as f64;
    let blank_frames = blank.counts.per_decision(blank.counts.frames);

    println!("\n=== {}: {} games at Commander scale ===", board.name, board.seeds.len());

    println!("\n1. floor 1 — per decision, medians of {ROUNDS} rounds");
    let blank_on = median(blank.rounds_card_on.clone()) * 1e6 / blank.card_on_decisions as f64;
    println!(
        "{:<17} {:>9} {:>8} {:>8} {:>9} {:>9} {:>8} {:>7} {:>11} {:>8} {:>14} {:>9} {:>7}",
        "arm", "decisions", "b.walks", "l.walks", "frames", "hits", "µs", "×", "decisions/s", "row on", "diverged", "µs, on", "×, on"
    );
    for (arm, r) in Arm::ALL.iter().zip(readings) {
        let c = r.counts;
        let micros = median(r.rounds.clone()) * 1e6 / c.decisions as f64;
        let diverged: Vec<u64> = r.divergence.iter().filter_map(|(at, _)| *at).collect();
        let row_on = if *arm == Arm::Blank {
            "—".to_string()
        } else {
            format!("{:.1}%", 100.0 * r.prompts_with_card as f64 / r.prompts as f64)
        };
        let diverged = if *arm == Arm::Blank {
            "—".to_string()
        } else {
            format!("{} of {}", diverged.len(), r.divergence.len())
        };
        let micros_on = median(r.rounds_card_on.clone()) * 1e6 / r.card_on_decisions as f64;
        println!(
            "{:<17} {:>9} {:>8.2} {:>8.2} {:>9.1} {:>9.1} {:>8.1} {:>7.2} {:>11.0} {:>8} {:>14} {:>9.1} {:>7.2}",
            arm.name(),
            c.decisions,
            c.per_decision(c.board_walks),
            c.per_decision(c.layer_walks),
            c.per_decision(c.frames),
            c.per_decision(c.memo_hits),
            micros,
            micros / blank_micros,
            1e6 / micros,
            row_on,
            diverged,
            micros_on,
            micros_on / blank_on,
        );
    }
    for (arm, r) in Arm::ALL.iter().zip(readings).skip(1) {
        let frames = r.counts.per_decision(r.counts.frames);
        let micros = median(r.rounds_card_on.clone()) * 1e6 / r.card_on_decisions as f64;
        println!(
            "  {}: frames ×{:.1}; with the row on, floor 1 {} at {:.0} decisions/s on one thread ({} of {} decisions)",
            arm.name(),
            frames / blank_frames,
            if 1e6 / micros >= FLOOR_1 { "holds" } else { "fails" },
            1e6 / micros,
            r.card_on_decisions,
            r.counts.decisions
        );
        let mut firsts: Vec<String> = r
            .divergence
            .iter()
            .zip(&board.seeds)
            .filter_map(|((at, of), seed)| at.map(|at| format!("{seed}: {at} of {of}")))
            .collect();
        if firsts.is_empty() {
            firsts.push("no game diverged".to_string());
        }
        println!("    first differing decision — {}", firsts.join(", "));
    }

    println!("\n2. floors 2 and 3 — one clone, at item 143's checkpoints and each game's end");
    println!(
        "{:<17} {:>11} {:>10} {:>9} {:>14} {:>8}",
        "arm", "checkpoints", "worst KB", "allocs", "worst µs", "over"
    );
    for (arm, r) in Arm::ALL.iter().zip(readings) {
        let worst_kb = r.clones.iter().map(|(_, _, c)| c.bytes).max().unwrap_or(0) as f64 / 1024.0;
        let worst_allocs = r.clones.iter().map(|(_, _, c)| c.allocations).max().unwrap_or(0);
        let worst_micros = r.clones.iter().map(|(_, _, c)| c.micros).fold(0.0, f64::max);
        let over: Vec<String> = r
            .clones
            .iter()
            .filter(|(_, _, c)| c.allocations > MAX_ALLOCATIONS || c.bytes > MAX_BYTES)
            .map(|(seed, at, c)| format!("{seed} {at}: {:.1} KB, {} allocations", c.bytes as f64 / 1024.0, c.allocations))
            .collect();
        println!(
            "{:<17} {:>11} {:>10.1} {:>9} {:>14.1} {:>8}",
            arm.name(),
            r.clones.len(),
            worst_kb,
            worst_allocs,
            worst_micros,
            over.len()
        );
        for at in over {
            println!("    over a floor: {at}");
        }
    }
    println!("  each game's end, the three arms: KB / allocations / µs, turns");
    for seed in &board.seeds {
        let cells: Vec<String> = readings
            .iter()
            .map(|r| {
                let (_, at, c) = r
                    .clones
                    .iter()
                    .find(|(s, at, _)| s == seed && at.starts_with("end"))
                    .expect("every game reads its end");
                let turns = at.trim_start_matches("end, ");
                format!("{:.1} / {} / {:.1}, {turns}", c.bytes as f64 / 1024.0, c.allocations, c.micros)
            })
            .collect();
        println!("    {seed}: {}", cells.join("  |  "));
    }

    println!("\n3. the redeal — µs, medians of {FORK_REPS} per fork; mid-game is turn {MID_GAME} on");
    println!(
        "{:<17} {:>6} {:>13} {:>13} {:>13} {:>17} {:>15}",
        "arm", "forks", "redeal", "warm", "cold", "cold − warm", "cold walks/frames"
    );
    for (arm, r) in Arm::ALL.iter().zip(readings) {
        let mid: Vec<&ForkReading> = r.forks.iter().filter(|f| f.turn >= MID_GAME).collect();
        let range = |read: fn(&ForkReading) -> f64| {
            let values: Vec<f64> = mid.iter().map(|f| read(f)).collect();
            if values.is_empty() {
                return "—".to_string();
            }
            let lo = values.iter().copied().fold(f64::INFINITY, f64::min);
            let hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            format!("{lo:.1}–{hi:.1}")
        };
        let walks = mid.iter().map(|f| f.cold_walks).max().unwrap_or(0);
        let frames = if mid.is_empty() { 0.0 } else { median(mid.iter().map(|f| f.cold_frames as f64).collect()) };
        println!(
            "{:<17} {:>6} {:>13} {:>13} {:>13} {:>17} {:>15}",
            arm.name(),
            mid.len(),
            range(|f| f.redeal_micros),
            range(|f| f.warm_micros),
            range(|f| f.cold_micros),
            range(|f| f.cold_micros - f.warm_micros),
            format!("≤{walks} / {frames:.0}"),
        );
    }
    println!("  cold − warm by turn, the median over the board's forks, per arm:");
    let mut turns: Vec<u32> = blank.forks.iter().map(|f| f.turn).collect();
    turns.sort();
    turns.dedup();
    for turn in turns {
        let cells: Vec<String> = readings
            .iter()
            .map(|r| {
                let at: Vec<f64> = r.forks.iter().filter(|f| f.turn == turn).map(|f| f.cold_micros - f.warm_micros).collect();
                if at.is_empty() { "—".to_string() } else { format!("{:.1} ({})", median(at.clone()), at.len()) }
            })
            .collect();
        println!("    turn {turn:>3}: {}", cells.join("  |  "));
    }

    println!("\n4. what a zone row adds to each pass, at the checkpoints (the no-row arm's games)");
    let (members, matched): (u64, u64) = blank.members.iter().fold((0, 0), |(m, t), (a, b)| (m + a, t + b));
    let n = blank.members.len().max(1) as f64;
    println!(
        "  {:.0} objects off the battlefield per checkpoint; Teferi's filter matches {:.1} of them ({:.1}%)",
        members as f64 / n,
        matched as f64 / n,
        100.0 * matched as f64 / members.max(1) as f64
    );
}
