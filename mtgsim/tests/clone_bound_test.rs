//! Floors 2 and 3 in CI: what one clone of a Commander game allocates, at item
//! 143's checkpoints through the game's end and **at every decision**
//! (`engineering-practices.md#the-floors`; `clone-architecture.md` decision 6).
//! A search clones mid-turn too, where the stack and the triggers waiting hold
//! what between turns they do not: TR-5g's games reached 78 allocations at a
//! 22-trigger ordering prompt while every checkpoint read under 64.
//!
//! A search that stores states clones one at every iteration, so a clone has
//! to stay cheap and small however long the game runs
//! (`plans/references/ai-performance-floors.md`). Time is machine-bound and
//! read at the readiness pass; this test asserts the portable proxies:
//! **at most 64 allocations** (floor 2's CI form) and **at most 128 KB** (floor
//! 3), counted exactly by `support/counting_allocator.rs`.
//!
//! **Its own binary, with one test.** The global allocator counts only while
//! the measuring thread has switched it on, so nothing else in the process
//! could reach the count, and one test means one thread to switch it.
//!
//! **Release only**, as CI's own step runs it (`cargo test --release --test
//! clone_bound_test`): about two seconds there, and 675 s in a debug build
//! (three games), where the layer memo's audit re-walks every memo hit
//! (`compute_characteristics`).
//!
//! **The games are `fuzz_games`'**: its deck builder and its three seed
//! derivations, four seats of 100 cards at 40 life. Three are `stress` seeds.
//! 12345 and 777 are item 143's; 12351 is game 7 of the 100-game sweep at
//! 12345, which ran 184 turns on the tree this test was written on, and a long
//! game is what shows a size bounded by the board rather than the turn count.
//! The `stress` pool grows with every registered card, so its games change
//! with it; the test reads whatever checkpoints they reach. **The rest are
//! `close_out.py`'s 20 `performance` games**, the board the budget is read on,
//! so the floors are checked where the budget is. `fuzz_games --pool <pool>
//! --players 4 --deck-size 100 --life 40 --seed <seed> --games 1` plays any of
//! them.
//!
//! **The fourth game is 12351 again, with Teferi's clause under player 0 from
//! setup on** (`phase_lj_cards::teferi_flash_clause`, `codebase-state.md` item
//! 181). A row reaching every zone but the battlefield makes the memo hold a
//! frame for every card in the game, and its `And` filter's two boxes are cloned
//! with the registry, so it is the heavier clone of item 181's two fixtures.

use std::cell::RefCell;
use std::sync::Arc;

use rand::rngs::StdRng;
use rand::SeedableRng;

use mtgsim::cards::phase_lj_cards::teferi_flash_clause;
use mtgsim::cards::random_deck::random_deck;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::put_on_battlefield;
use mtgsim::types::ids::PlayerId;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, SeatMode};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

#[path = "support/counting_allocator.rs"]
mod counting_allocator;
use counting_allocator::clone_cost;

/// Floor 2's CI proxy.
const MAX_ALLOCATIONS: u64 = 64;
/// Floor 3.
const MAX_BYTES: u64 = 128 * 1024;

/// Item 143's checkpoints: the state between two `run_turn` calls, named by
/// the turn about to be played, and the game's end.
const CHECKPOINTS: [u32; 10] = [1, 10, 20, 30, 40, 50, 60, 80, 100, 150];

/// `fuzz_games`' turn limit.
const MAX_TURNS: u32 = 200;

/// A clone's cost at each prompt with two or more answers: what reaches the
/// agent under `ManaWindowStop`, so it sits inside it. The game's answers are
/// the inner provider's, so the games are `fuzz_games`' still.
struct CloneAtEachDecision<D> {
    inner: D,
    /// The decisions read, and the worst: (allocations, bytes, where).
    decisions: RefCell<usize>,
    worst: RefCell<(u64, u64, String)>,
    over: RefCell<Vec<String>>,
}

impl<D> CloneAtEachDecision<D> {
    fn at(&self, game: &GameState, answers: usize, prompt: &str) {
        if answers < 2 {
            return;
        }
        let (allocations, bytes) = clone_cost(game);
        *self.decisions.borrow_mut() += 1;
        let at = || format!("turn {} {prompt}: {allocations} allocations, {:.1} KB", game.turn_number, bytes as f64 / 1024.0);
        if allocations > MAX_ALLOCATIONS || bytes > MAX_BYTES {
            self.over.borrow_mut().push(at());
        }
        let mut worst = self.worst.borrow_mut();
        if allocations > worst.0 || (allocations == worst.0 && bytes > worst.1) {
            *worst = (allocations, bytes, at());
        }
    }
}

/// The prompt's kind, short.
fn named(context: &ChoiceContext) -> String {
    format!("{:?}", context.kind).chars().take(48).collect()
}

impl<D: DecisionProvider> DecisionProvider for CloneAtEachDecision<D> {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        self.at(game, options.len(), &named(context));
        self.inner.pick_n(game, player, context, options, bounds)
    }
    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.at(game, (max - min + 1) as usize, &named(context));
        self.inner.pick_number(game, player, context, min, max)
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
        self.at(game, buckets.len(), &named(context));
        self.inner.allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }
    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.at(game, items.len(), &named(context));
        self.inner.choose_ordering(game, player, context, items)
    }
    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.inner.seat_mode(player)
    }
}

/// `fuzz_games`' game `game_seed` on the Commander board, set up, with `row`'s
/// card on the battlefield under player 0 if there is one.
fn commander_game(
    registry: &CardRegistry,
    game_seed: u64,
    row: Option<Arc<CardData>>,
) -> (Game, ManaWindowStop<CloneAtEachDecision<RandomDecisionProvider>>) {
    let mut deck_rng = StdRng::seed_from_u64(game_seed);
    let shuffle_seed = game_seed ^ 0x9E37_79B9_7F4A_7C15;
    let dp_seed = game_seed ^ 0xD1B5_4A32_D192_ED03;
    let decks: Vec<Vec<Arc<CardData>>> = (0..4).map(|_| random_deck(registry, &mut deck_rng, &[], 1, 100)).collect();
    let mut config = GameConfig::test();
    config.starting_life = 40;
    let mut game = Game::new(config, decks).expect("game creation");
    game.reseed(shuffle_seed);
    let dp = ManaWindowStop::new(CloneAtEachDecision {
        inner: RandomDecisionProvider::seeded(dp_seed),
        decisions: RefCell::new(0),
        worst: RefCell::new((0, 0, String::new())),
        over: RefCell::new(Vec::new()),
    });
    game.setup(&dp).expect("setup");
    if let Some(card) = row {
        put_on_battlefield(&mut game.state, card, 0);
    }
    (game, dp)
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release only: CI runs it with --release")]
fn a_commander_clone_stays_under_the_allocation_and_byte_floors() {
    // Bytes vary with the id hasher's seed: a map doubles one checkpoint
    // earlier under some seeds (item 143's re-take). `types::ids` reads the
    // variable once, when the first id map is built, which is below.
    // SAFETY: this binary's one test sets it before any other thread exists
    // that could read the environment.
    unsafe { std::env::set_var("MTGSIM_HASH_SEED", "1") };

    let stress = CardRegistry::default_registry();
    let performance = CardRegistry::performance_pool();
    let mut readings = Vec::new();
    let mut over = Vec::new();
    let mut decision_count = 0;
    let mut games: Vec<(&str, &CardRegistry, u64, Option<fn() -> Arc<CardData>>)> = vec![
        ("stress", &stress, 12345, None),
        ("stress", &stress, 777, None),
        ("stress", &stress, 12351, None),
        ("stress", &stress, 12351, Some(teferi_flash_clause)),
    ];
    // `close_out.py`'s `--games 20 --seed 12345 --pool performance`.
    games.extend((12345..12365).map(|seed| ("performance", &performance, seed, None)));
    for (pool, registry, seed, row) in games {
        let (mut game, dp) = commander_game(registry, seed, row.map(|card| card()));
        let with = if row.is_some() { " with Teferi's clause" } else { "" };
        let mut turns = 0;
        let mut read = |game: &Game, at: String| {
            let (allocations, bytes) = clone_cost(&game.state);
            let line = format!("{pool} {seed}{with}, {at}: {allocations} allocations, {:.1} KB", bytes as f64 / 1024.0);
            if allocations > MAX_ALLOCATIONS || bytes > MAX_BYTES {
                over.push(line.clone());
            }
            readings.push(line);
        };
        while !game.is_over() && turns < MAX_TURNS {
            if CHECKPOINTS.contains(&game.state.turn_number) {
                read(&game, format!("turn {}", game.state.turn_number));
            }
            game.run_turn(&dp).expect("turn");
            turns += 1;
        }
        read(&game, format!("end, {turns} turns"));
        let decisions = dp.inner();
        let (allocations, bytes, at) = decisions.worst.borrow().clone();
        decision_count += *decisions.decisions.borrow();
        if allocations > 0 {
            readings.push(format!("{pool} {seed}{with}, worst of its decisions, {at}"));
        }
        let _ = bytes;
        over.extend(decisions.over.borrow().iter().map(|line| format!("{pool} {seed}{with}, {line}")));
    }
    readings.push(format!("{decision_count} decisions read"));
    println!("{}", readings.join("\n"));
    assert!(
        over.is_empty(),
        "a clone passed {MAX_ALLOCATIONS} allocations or {} KB:\n{}\n\nevery reading:\n{}",
        MAX_BYTES / 1024,
        over.join("\n"),
        readings.join("\n"),
    );
}
