//! What a fork copies between two decisions (`clone-architecture.md` decision
//! 6(c)): the write side of a clone, which a clone's own reading cannot see.
//!
//! A clone shares every chunk of the stores in `state::shared_stores`, and a
//! fork copies a chunk the first time it writes into it. So the chunks a game
//! holds at one decision that it did not hold at the last are what a fork
//! taken at the last would have copied on its way here. A walk that takes
//! `&mut` of every entry copies every chunk of its store at every pass, which
//! is what this catches: the state-based check's deathtouch sweep did, at
//! every check, before CL-1.
//!
//! **Release only**, as CI's step runs it: four Commander games, cloned at
//! every decision.

use std::cell::{Cell, RefCell};
use std::sync::Arc;

use rand::SeedableRng;
use rand::rngs::StdRng;

use mtgsim::cards::random_deck::random_deck;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::GameState;
use mtgsim::types::ids::PlayerId;
use mtgsim::ui::ask::{forced_allocation, only_number, only_order, only_pick};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, SeatMode};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

/// The mean chunks a fork may copy between two decisions, over every store.
/// The pooled board read 2.2 at the design (`clone-architecture.md` §2.6),
/// and a sweep of every permanent alone adds the battlefield's every chunk.
const MAX_MEAN_CHUNKS: f64 = 4.0;

/// Keeps the state as the last decision left it, and at each decision counts
/// the chunks the game has copied since: what reaches the agent under
/// `ManaWindowStop`, so it sits inside it. A decision is a prompt with two or
/// more legal answers by the engine's own predicates.
struct ChunksSinceTheLastDecision<D> {
    inner: D,
    last: RefCell<Option<GameState>>,
    intervals: Cell<usize>,
    chunks: Cell<usize>,
    most: Cell<usize>,
}

impl<D> ChunksSinceTheLastDecision<D> {
    fn at(&self, game: &GameState, forced: bool) {
        if forced {
            return;
        }
        if let Some(last) = self.last.borrow().as_ref() {
            let copied = game.objects.chunks_not_shared_with(&last.objects)
                + game.battlefield.chunks_not_shared_with(&last.battlefield)
                + game.stack_entries.chunks_not_shared_with(&last.stack_entries)
                + game.stack.chunks_not_shared_with(&last.stack)
                + game.pending_triggers.chunks_not_shared_with(&last.pending_triggers);
            self.intervals.set(self.intervals.get() + 1);
            self.chunks.set(self.chunks.get() + copied);
            self.most.set(self.most.get().max(copied));
        }
        *self.last.borrow_mut() = Some(game.clone());
    }
}

impl<D: DecisionProvider> DecisionProvider for ChunksSinceTheLastDecision<D> {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        self.at(game, only_pick(options.len(), bounds).is_some());
        self.inner.pick_n(game, player, context, options, bounds)
    }
    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.at(game, only_number(min, max).is_some());
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
        self.at(game, forced_allocation(total, per_bucket_mins, per_bucket_maxs).is_some());
        self.inner.allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }
    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.at(game, only_order(items.len()).is_some());
        self.inner.choose_ordering(game, player, context, items)
    }
    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.inner.seat_mode(player)
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release only: CI runs it with --release")]
fn a_fork_copies_only_the_chunks_it_writes() {
    let registry = CardRegistry::performance_pool();
    let (mut intervals, mut chunks, mut most) = (0, 0, 0);
    // `close_out.py`'s first four `performance` games, as `clone_bound_test`
    // builds them.
    for game_seed in 12345..12349u64 {
        let mut deck_rng = StdRng::seed_from_u64(game_seed);
        let decks: Vec<Vec<Arc<CardData>>> = (0..4).map(|_| random_deck(&registry, &mut deck_rng, &[], 1, 100)).collect();
        let mut config = GameConfig::test();
        config.starting_life = 40;
        let mut game = Game::new(config, decks).expect("game creation");
        game.reseed(game_seed ^ 0x9E37_79B9_7F4A_7C15);
        let dp = ManaWindowStop::new(ChunksSinceTheLastDecision {
            inner: RandomDecisionProvider::seeded(game_seed ^ 0xD1B5_4A32_D192_ED03),
            last: RefCell::new(None),
            intervals: Cell::new(0),
            chunks: Cell::new(0),
            most: Cell::new(0),
        });
        game.setup(&dp).expect("setup");
        let mut turns = 0;
        while !game.is_over() && turns < 200 {
            game.run_turn(&dp).expect("turn");
            turns += 1;
        }
        let reading = dp.inner();
        intervals += reading.intervals.get();
        chunks += reading.chunks.get();
        most = most.max(reading.most.get());
    }
    let mean = chunks as f64 / intervals.max(1) as f64;
    println!("{intervals} intervals between decisions: {mean:.2} chunks copied on average, {most} at most");
    assert!(intervals > 1000, "the games reached {intervals} decisions");
    assert!(mean <= MAX_MEAN_CHUNKS, "a fork copies {mean:.2} chunks between decisions on average, past {MAX_MEAN_CHUNKS}");
}
