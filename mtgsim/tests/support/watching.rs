//! A provider that notes what each prompt offered, for a test that reads who
//! was asked and among what: the scripted provider's own record
//! (`ScriptedDecisionProvider::asked_of`) keeps the question and not its
//! options.

use std::cell::RefCell;
use std::rc::Rc;

use mtgsim::decision::provider::{DecisionProvider, SeatMode};
use mtgsim::decision::question::{DecisionContext, DecisionKind, DecisionOption};
use mtgsim::state::game_state::GameState;
use mtgsim::types::ids::PlayerId;

/// What each prompt offered: who was asked, the question, and its options,
/// a number's as its least and most.
#[derive(Default)]
pub struct Asked {
    pub of_options: RefCell<Vec<(PlayerId, DecisionKind, Vec<DecisionOption>)>>,
    pub of_numbers: RefCell<Vec<(PlayerId, DecisionKind, u64, u64)>>,
}

/// A provider that notes each prompt in `asked` and answers through `inner`.
pub struct Watching<D> {
    inner: D,
    asked: Rc<Asked>,
}

impl<D> Watching<D> {
    pub fn new(inner: D) -> (Self, Rc<Asked>) {
        let asked = Rc::new(Asked::default());
        (Watching { inner, asked: Rc::clone(&asked) }, asked)
    }
}

impl<D: DecisionProvider> DecisionProvider for Watching<D> {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &DecisionContext, options: &[DecisionOption], bounds: (usize, usize)) -> Vec<usize> {
        self.asked.of_options.borrow_mut().push((player, context.kind.clone(), options.to_vec()));
        self.inner.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &DecisionContext, min: u64, max: u64) -> u64 {
        self.asked.of_numbers.borrow_mut().push((player, context.kind.clone(), min, max));
        self.inner.pick_number(game, player, context, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &DecisionContext,
        total: u64,
        buckets: &[DecisionOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.inner.allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &DecisionContext, items: &[DecisionOption]) -> Vec<usize> {
        self.inner.choose_ordering(game, player, context, items)
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.inner.seat_mode(player)
    }
}
