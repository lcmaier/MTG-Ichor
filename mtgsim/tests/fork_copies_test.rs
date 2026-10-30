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

use mtgsim::cards::registry::CardRegistry;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::deal_commander_game;
use mtgsim::ui::each_decision::OnEachDecision;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

/// The mean chunks a fork may copy between two decisions, over every store.
/// The pooled board read 2.2 at the design (`clone-architecture.md` §2.6),
/// and a sweep of every permanent alone adds the battlefield's every chunk.
const MAX_MEAN_CHUNKS: f64 = 4.0;

/// Keeps the state as the last decision left it, and at each decision counts
/// the chunks the game has copied since.
#[derive(Default)]
struct ChunksSinceTheLastDecision {
    last: RefCell<Option<GameState>>,
    intervals: Cell<usize>,
    chunks: Cell<usize>,
    most: Cell<usize>,
}

impl ChunksSinceTheLastDecision {
    fn at(&self, game: &GameState) {
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

#[test]
#[cfg_attr(debug_assertions, ignore = "release only: CI runs it with --release")]
fn a_fork_copies_only_the_chunks_it_writes() {
    let registry = CardRegistry::performance_pool();
    let (mut intervals, mut chunks, mut most) = (0, 0, 0);
    // `close_out.py`'s first four `performance` games, as `clone_bound_test`
    // builds them.
    for game_seed in 12345..12349u64 {
        let (mut game, agents) = deal_commander_game(&registry, game_seed);
        let reading = ChunksSinceTheLastDecision::default();
        // Inside `ManaWindowStop`, so it sees what reaches the agent.
        let dp = ManaWindowStop::new(OnEachDecision::new(RandomDecisionProvider::seeded(agents), |game, _| reading.at(game)));
        game.setup(&dp).expect("setup");
        let mut turns = 0;
        while !game.is_over() && turns < 200 {
            game.run_turn(&dp).expect("turn");
            turns += 1;
        }
        intervals += reading.intervals.get();
        chunks += reading.chunks.get();
        most = most.max(reading.most.get());
    }
    let mean = chunks as f64 / intervals.max(1) as f64;
    println!("{intervals} intervals between decisions: {mean:.2} chunks copied on average, {most} at most");
    assert!(intervals > 1000, "the games reached {intervals} decisions");
    assert!(mean <= MAX_MEAN_CHUNKS, "a fork copies {mean:.2} chunks between decisions on average, past {MAX_MEAN_CHUNKS}");
}
