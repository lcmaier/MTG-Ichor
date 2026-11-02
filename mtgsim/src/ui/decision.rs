use std::cell::RefCell;
use std::collections::VecDeque;
use std::mem::discriminant;

use crate::state::game_state::GameState;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};

use super::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption, named_by_script};

/// What the engine asks a seat's provider about the seat, rather than about a
/// decision ([`DecisionProvider::seat_mode`]). Client state, never
/// `GameState`'s, and no replay needs it: a field changes only whether a seat
/// is asked a question with one legal answer, which the engine logs either
/// way (`state::decision_log`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SeatMode {
    /// Asked at a priority point where `Pass` is all the seat can do, which
    /// the engine otherwise takes itself: a prompt with one legal answer is
    /// the engine's (`backlog.md` §2.22, rule 1). A person in full control
    /// stops there, and so does a test that watches every grant.
    pub stops_at_every_priority_point: bool,
}

/// Why a run ended before its game did (`setup-architecture.md` §7.2,
/// decision 1). A provider must return an answer and cannot say "stop", so it
/// raises one of these from inside the prompt, and [`crate::state::game::Game::until_stopped`]
/// is the one place that catches it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stop {
    /// A replay with no seat to hand over to answered its last line.
    LogSpent { answered: usize },
    /// A replayed answer is not what the engine asks: `line` is its number in
    /// the log, and `why` says what differs.
    Diverged { line: usize, why: String },
    /// A scenario's setup line cannot be played: its line in the file, the
    /// line as written, and what to change.
    SetupRefused { line: usize, written: String, why: String },
    /// A newer replay superseded this one, which stopped at its next answer.
    Superseded,
}

impl Stop {
    /// End the run here. `resume_unwind` skips the panic hook, so a stop
    /// prints nothing on its way out.
    pub fn raise(self) -> ! {
        std::panic::resume_unwind(Box::new(self))
    }
}

impl std::fmt::Display for Stop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stop::LogSpent { answered } => write!(f, "the log's {answered} answers are spent, and no seat plays on"),
            Stop::Diverged { line, why } => write!(f, "the log diverges at answer {line}: {why}"),
            Stop::SetupRefused { line, written, why } => write!(f, "line {line}, `{written}`: {why}"),
            Stop::Superseded => write!(f, "a newer replay superseded this one"),
        }
    }
}

/// What a player chooses to do when they have priority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PriorityAction {
    /// Pass priority without taking an action
    Pass,
    /// Cast a spell from a zone it can be cast from
    CastSpell(ObjectId),
    /// Activate an activated ability on a permanent
    ActivateAbility(ObjectId, AbilityId),
    /// Play a land from a zone it could be played from
    PlayLand(ObjectId),
}

// ===========================================================================
// DecisionProvider implementation for Dispatch
// ===========================================================================

/// A decision provider that dispatches to different providers per player.
///
/// Enables any combination of human/bot/network players in a single game.
/// Each player is assigned a `Box<dyn DecisionProvider>` at construction time.
/// All `DecisionProvider` methods route through `dp_for(player_id)`.
pub struct DispatchDecisionProvider {
    providers: Vec<Box<dyn DecisionProvider>>,
}

impl DispatchDecisionProvider {
    /// Create a new dispatcher from a list of generic providers, one per player.
    pub fn new(providers: Vec<Box<dyn DecisionProvider>>) -> Self {
        DispatchDecisionProvider { providers }
    }

    fn dp_for(&self, player_id: PlayerId) -> &dyn DecisionProvider {
        &*self.providers[player_id]
    }
}

impl DecisionProvider for DispatchDecisionProvider {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        self.dp_for(player).pick_n(game, player, context, options, bounds)
    }

    fn pick_number(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        min: u64,
        max: u64,
    ) -> u64 {
        self.dp_for(player).pick_number(game, player, context, min, max)
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
        self.dp_for(player).allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize> {
        self.dp_for(player).choose_ordering(game, player, context, items)
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.dp_for(player).seat_mode(player)
    }
}

// ===========================================================================
// 4-PRIMITIVE DECISION PROVIDER TRAIT
// ===========================================================================

/// The 4-primitive decision provider trait.
///
/// Every MtG decision decomposes into one of these four shapes:
/// - `pick_n`: select N items from a list (attackers, targets, discard, etc.)
/// - `pick_number`: choose a number in a range (X value, loop count, etc.)
/// - `allocate`: distribute a total across buckets (damage, mana, etc.)
/// - `choose_ordering`: reorder a list (scry, stack ordering, etc.)
///
/// `ChoiceContext` carries the semantic kind so impls can specialize.
pub trait DecisionProvider {
    /// Pick N items from a list of options. Bounds: (min, max) selections.
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize>;

    /// Pick a number in an inclusive range.
    fn pick_number(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        min: u64,
        max: u64,
    ) -> u64;

    /// Distribute a total across buckets. Sum of returned vec must equal total.
    /// `per_bucket_mins` has one entry per bucket: bucket[i] must receive >= per_bucket_mins[i].
    /// `per_bucket_maxs`, if Some, has one entry per bucket: bucket[i] must receive <= per_bucket_maxs[i].
    /// For uniform minimums, pass `&vec![min; buckets.len()]`.
    /// Pass `None` for maxs when no upper bound is needed.
    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64>;

    /// Order a list of items. Returns indices in desired order.
    fn choose_ordering(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize>;

    /// The seat's mode. The default, every field off, is an agent's; a
    /// provider that is not an agent says so, and anything standing between
    /// the engine and that provider — a decorator,
    /// [`DispatchDecisionProvider`] — forwards the question.
    fn seat_mode(&self, _player: PlayerId) -> SeatMode {
        SeatMode::default()
    }
}

/// The provider for an action a rule performs that asks nobody anything: the
/// draw at the dispatcher's nesting bound (CR 104.4b), proposed from a
/// dispatch that may have no provider of its own. Nothing reaches it, since no
/// replacement reads a draw and the game it ends triggers nothing. Every
/// answer it would give is one its validator refuses, so a prompt that did
/// reach it stops the game instead of being answered.
pub(crate) struct AsksNobody;

impl DecisionProvider for AsksNobody {
    fn pick_n(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, options: &[ChoiceOption], _: (usize, usize)) -> Vec<usize> {
        vec![options.len()]
    }

    fn pick_number(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: u64, max: u64) -> u64 {
        max.saturating_add(1)
    }

    fn allocate(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, total: u64, buckets: &[ChoiceOption], _: &[u64], _: Option<&[u64]>) -> Vec<u64> {
        vec![total.saturating_add(1); buckets.len().max(1)]
    }

    fn choose_ordering(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        vec![items.len()]
    }
}

// ---------------------------------------------------------------------------
// Scripted DecisionProvider for tests
// ---------------------------------------------------------------------------

/// A single scripted expectation: what kind of decision we expect, and what to return.
#[derive(Debug)]
pub struct ScriptedExpectation {
    /// The ChoiceKind we expect the engine to ask for (discriminant match, fields ignored).
    /// No `Any` fallback — every scripted decision must declare what it expects.
    pub expected_kind: ChoiceKind,
    pub response: ScriptedResponse,
}

/// An option as a script names it: what it is, and, when other options
/// offered read the same, its place among them counted from 1, as the
/// decision log writes `(2 of 3)`.
pub type NamedOption = (ChoiceOption, Option<usize>);

/// The response shape for a scripted expectation. Every answer names what it
/// chooses, found in whatever order the engine lists the options, by the
/// matcher a replay uses (`position_named`).
#[derive(Debug)]
pub enum ScriptedResponse {
    /// The options to pick.
    Choose(Vec<NamedOption>),
    Number(u64),
    /// Each bucket with its amount: the decision log's allocation.
    Allocation(Vec<(ChoiceOption, u64)>),
    /// The items, first first.
    Order(Vec<NamedOption>),
    /// A take-or-leave question's one option, taken.
    Take,
}

/// The positions in `options` of the options `chosen` names, by the one
/// matcher. A script is stricter than a replayed line: words alone that more
/// than one offered option reads are refused, since they would mean whichever
/// the engine lists first, and so is a place where no other option reads the
/// same, which would mean nothing.
fn found_indices<'a>(options: &[ChoiceOption], game: &GameState, chosen: impl IntoIterator<Item = &'a NamedOption>) -> Vec<usize> {
    let offered = || options.iter().map(|o| o.as_logged(game)).collect::<Vec<_>>();
    let mut found = Vec::new();
    for (option, place) in chosen {
        let words = option.as_logged(game);
        let named = named_by_script(options, game, option);
        match (place, named.len()) {
            (None, alike) if alike > 1 => panic!(
                "ScriptedDecisionProvider: {alike} options offered read {words}, and its words alone cannot say which: {:?}",
                offered()
            ),
            (Some(_), 1) => panic!("ScriptedDecisionProvider: no other option offered reads {words}, so name it without a place"),
            _ => {}
        }
        let at = match place {
            Some(place) => place.checked_sub(1).and_then(|place| named.get(place)).copied().filter(|i| !found.contains(i)),
            None => named.into_iter().find(|i| !found.contains(i)),
        };
        match at {
            Some(at) => found.push(at),
            None => panic!("ScriptedDecisionProvider: {words} is not offered: {:?}", offered()),
        }
    }
    found
}

/// A test-oriented generic decision provider with a single queue of
/// `ScriptedExpectation`s. Each expectation pairs a mandatory `ChoiceKind`
/// (discriminant-matched) with a `ScriptedResponse`.
///
/// Every test must state what decision it expects — this is the entire point
/// of the self-documenting design.
pub struct ScriptedDecisionProvider {
    queue: RefCell<VecDeque<ScriptedExpectation>>,
    /// The `per_bucket_mins` of every [`DecisionProvider::allocate`] this
    /// provider has answered, in prompt order — see [`Self::allocation_mins`].
    allocation_mins: RefCell<Vec<Vec<u64>>>,
    /// Every prompt's player and kind, in prompt order — see [`Self::asked`]
    /// and [`Self::asked_of`].
    asked: RefCell<Vec<(PlayerId, ChoiceKind)>>,
    mode: SeatMode,
}

impl ScriptedDecisionProvider {
    pub fn new() -> Self {
        ScriptedDecisionProvider {
            queue: RefCell::new(VecDeque::new()),
            allocation_mins: RefCell::new(Vec::new()),
            asked: RefCell::new(Vec::new()),
            mode: SeatMode::default(),
        }
    }

    /// Report `mode` as the seat's, for a test that scripts what a seat which
    /// is not an agent's is asked.
    pub fn with_seat_mode(mut self, mode: SeatMode) -> Self {
        self.mode = mode;
        self
    }

    /// The per-bucket **minimums** each `allocate` was offered, in prompt
    /// order.
    ///
    /// **A constraint is not an outcome, and some rules are only about the
    /// constraint.** CR 702.19b's trample requirement — "assign lethal damage
    /// to all those blocking creatures" — is a floor the engine computes and
    /// hands to the player; what the player then does with the rest is a
    /// different claim. A test that only reads the board cannot tell a correct
    /// floor from a permissive one that the scripted answer happened to
    /// respect, so a Furnace of Rath doubling the result would look identical
    /// whether lethal was judged before or after the doubling.
    ///
    /// Added to this provider rather than to a bespoke one: a fourth
    /// `DecisionProvider` in a test file has to implement
    /// all four methods, and the three it does not care about end up answering
    /// prompts the test never expected — which is the exact failure this
    /// provider's "every decision must be declared" design exists to prevent.
    pub fn allocation_mins(&self) -> Vec<Vec<u64>> {
        self.allocation_mins.borrow().clone()
    }

    /// The kind of every prompt answered, in prompt order, with what each
    /// carried: a `ChooseXValue`'s values, a window's remaining cost. The
    /// queue matches a kind by its variant alone, so what a question offered
    /// is read here, as [`Self::allocation_mins`] reads a split's floors.
    pub fn asked(&self) -> Vec<ChoiceKind> {
        self.asked.borrow().iter().map(|(_, kind)| kind.clone()).collect()
    }

    /// [`Self::asked`] with the player each prompt asked, so a test can say
    /// who answered as well as what.
    pub fn asked_of(&self) -> Vec<(PlayerId, ChoiceKind)> {
        self.asked.borrow().clone()
    }

    /// Enqueue a pick_n expectation naming the options to pick, so an engine
    /// change that lists them in another order breaks nothing. An empty list
    /// picks none.
    pub fn expect_choice(&self, kind: ChoiceKind, chosen: Vec<ChoiceOption>) {
        self.queue.borrow_mut().push_back(ScriptedExpectation {
            expected_kind: kind,
            response: ScriptedResponse::Choose(chosen.into_iter().map(|option| (option, None)).collect()),
        });
    }

    /// [`Self::expect_choice`] for options that read the same as others
    /// offered, each with its place among them counted from 1, as the log
    /// writes `(2 of 2)`. Only a question that offers two effects as their one
    /// source needs it: CR 616.1's replacements, a creature's printed one
    /// beside its finality counter's, where place is all that tells them
    /// apart.
    pub fn expect_choice_among_alike(&self, kind: ChoiceKind, chosen: Vec<(ChoiceOption, usize)>) {
        self.queue.borrow_mut().push_back(ScriptedExpectation {
            expected_kind: kind,
            response: ScriptedResponse::Choose(chosen.into_iter().map(|(option, place)| (option, Some(place))).collect()),
        });
    }

    /// Enqueue a take-or-leave answer that takes the one option offered: a
    /// "you may" whose option is an object the call that asks creates, a
    /// trigger's stack object or a token entering, which no script can name
    /// beforehand. Refused when more than one option is offered. Leaving it
    /// is `expect_choice` of nothing.
    pub fn expect_take(&self, kind: ChoiceKind) {
        self.queue.borrow_mut().push_back(ScriptedExpectation {
            expected_kind: kind,
            response: ScriptedResponse::Take,
        });
    }

    /// Enqueue a pick_number expectation.
    pub fn expect_number(&self, kind: ChoiceKind, n: u64) {
        self.queue.borrow_mut().push_back(ScriptedExpectation {
            expected_kind: kind,
            response: ScriptedResponse::Number(n),
        });
    }

    /// Enqueue an allocate expectation that names every bucket the engine
    /// offers, in any order, with what it gets.
    pub fn expect_allocation(&self, kind: ChoiceKind, amounts: Vec<(ChoiceOption, u64)>) {
        self.queue.borrow_mut().push_back(ScriptedExpectation {
            expected_kind: kind,
            response: ScriptedResponse::Allocation(amounts),
        });
    }

    /// Enqueue a choose_ordering expectation that names the items, first
    /// first.
    pub fn expect_order(&self, kind: ChoiceKind, order: Vec<ChoiceOption>) {
        self.queue.borrow_mut().push_back(ScriptedExpectation {
            expected_kind: kind,
            response: ScriptedResponse::Order(order.into_iter().map(|option| (option, None)).collect()),
        });
    }

    /// [`Self::expect_order`] for items that read the same as others, each
    /// with its place among them counted from 1: CR 603.3b's triggered
    /// abilities of one source, which the order offers as that source.
    pub fn expect_order_among_alike(&self, kind: ChoiceKind, order: Vec<(ChoiceOption, usize)>) {
        self.queue.borrow_mut().push_back(ScriptedExpectation {
            expected_kind: kind,
            response: ScriptedResponse::Order(order.into_iter().map(|(option, place)| (option, Some(place))).collect()),
        });
    }

    /// Pop the front expectation and validate. Panics if queue is empty or
    /// kind doesn't match — every DP call must have a corresponding expectation.
    fn pop_and_validate(&self, player: PlayerId, actual_kind: &ChoiceKind, method_name: &str) -> ScriptedResponse {
        self.asked.borrow_mut().push((player, actual_kind.clone()));
        let mut queue = self.queue.borrow_mut();
        let expectation = match queue.pop_front() {
            Some(e) => e,
            None => panic!(
                "ScriptedDecisionProvider: unexpected {} call (kind: {:?}), no scripted response in queue",
                method_name, actual_kind
            ),
        };

        assert_eq!(
            discriminant(&expectation.expected_kind),
            discriminant(actual_kind),
            "ScriptedDecisionProvider: kind mismatch — expected {:?}, got {:?}",
            expectation.expected_kind,
            actual_kind,
        );

        expectation.response
    }

    /// Returns true if the queue is empty (all expectations consumed).
    pub fn is_empty(&self) -> bool {
        self.queue.borrow().is_empty()
    }

    /// Returns the number of remaining expectations.
    pub fn remaining(&self) -> usize {
        self.queue.borrow().len()
    }

    /// Enqueue the passes of a turn in which the active player holds a land
    /// and does nothing else: one in each main phase, where playing it is
    /// offered. Every other priority point offers `Pass` alone, which the
    /// engine takes without asking (`engine::priority`).
    pub fn queue_main_phase_passes(&self) {
        for _ in 0..2 {
            self.expect_choice(ChoiceKind::PriorityAction, vec![ChoiceOption::Action(PriorityAction::Pass)]);
        }
    }
}

impl Drop for ScriptedDecisionProvider {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            let remaining = self.queue.borrow().len();
            assert_eq!(
                remaining, 0,
                "ScriptedDecisionProvider dropped with {} unconsumed expectation(s) — \
                 test bug: scripted expectations were enqueued but never consumed by DP calls",
                remaining,
            );
        }
    }
}

impl DecisionProvider for ScriptedDecisionProvider {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        _bounds: (usize, usize),
    ) -> Vec<usize> {
        match self.pop_and_validate(player, &context.kind, "pick_n") {
            ScriptedResponse::Choose(chosen) => found_indices(options, game, &chosen),
            ScriptedResponse::Take => {
                assert_eq!(options.len(), 1, "ScriptedDecisionProvider: `expect_take` takes the one option offered, and the engine offers {options:?}");
                vec![0]
            }
            // A number answers a pick over numbers by value: CR 601.2b's X as
            // a set with a gap (`ChoiceKind::ChooseXValue`).
            ScriptedResponse::Number(n) => match options.iter().position(|option| matches!(option, ChoiceOption::Number(offered) if *offered == n)) {
                Some(index) => vec![index],
                None => panic!("ScriptedDecisionProvider: number {n} is not offered: {options:?}"),
            },
            other => panic!(
                "ScriptedDecisionProvider: pick_n called but scripted response is {:?}, expected a pick",
                other
            ),
        }
    }

    fn pick_number(
        &self,
        _game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        _min: u64,
        _max: u64,
    ) -> u64 {
        match self.pop_and_validate(player, &context.kind, "pick_number") {
            ScriptedResponse::Number(n) => n,
            other => panic!(
                "ScriptedDecisionProvider: pick_number called but scripted response is {:?}, expected Number",
                other
            ),
        }
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        _total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        _per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.allocation_mins.borrow_mut().push(per_bucket_mins.to_vec());
        match self.pop_and_validate(player, &context.kind, "allocate") {
            ScriptedResponse::Allocation(amounts) => {
                let named: Vec<NamedOption> = amounts.iter().map(|(bucket, _)| (bucket.clone(), None)).collect();
                let at = found_indices(buckets, game, &named);
                assert_eq!(
                    at.len(),
                    buckets.len(),
                    "ScriptedDecisionProvider: the allocation names {} buckets, and the engine offers {:?}",
                    at.len(),
                    buckets.iter().map(|bucket| bucket.as_logged(game)).collect::<Vec<_>>()
                );
                let mut allocation = vec![0; buckets.len()];
                for (&bucket, &(_, amount)) in at.iter().zip(&amounts) {
                    allocation[bucket] = amount;
                }
                allocation
            }
            other => panic!(
                "ScriptedDecisionProvider: allocate called but scripted response is {:?}, expected an allocation",
                other
            ),
        }
    }

    fn choose_ordering(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize> {
        match self.pop_and_validate(player, &context.kind, "choose_ordering") {
            ScriptedResponse::Order(order) => found_indices(items, game, &order),
            other => panic!(
                "ScriptedDecisionProvider: choose_ordering called but scripted response is {:?}, expected an order",
                other
            ),
        }
    }

    fn seat_mode(&self, _player: PlayerId) -> SeatMode {
        self.mode
    }
}
