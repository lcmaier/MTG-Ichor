//! A question the engine answered for a player without asking: a forced
//! prompt, whose one legal answer any provider would give, or an elision,
//! whose answers all leave one game (`plans/elisions.md`). Each is recorded
//! where it is taken, so a game's log says what was not asked and why; the
//! full event log draws it and the dev GUI's log does not
//! (`ui::display::log_lines`), as with a mana ability's activation.
//!
//! **An observer, never a participant**, as the decision log and the trace
//! sink are: kept by the event recorder beside its records and written
//! through `&GameState`, it draws no randomness, asks no provider and takes
//! no `EventId`, so a game performs the same events whether anything records.

use crate::events::event::EventId;
use crate::types::ids::PlayerId;
use crate::ui::choice_types::{ChoiceKind, ChoiceOption};

/// One question the engine did not ask.
#[derive(Debug, Clone)]
pub struct UnaskedChoice {
    /// The record it comes before in the stream: the next record's id when
    /// it was taken.
    pub before: EventId,
    /// The player who would have been asked.
    pub player: PlayerId,
    pub kind: ChoiceKind,
    pub answer: UnaskedAnswer,
    pub why: Unasked,
}

/// The answer given, in the shape of the primitive that would have asked,
/// each option by what it is rather than where it sat.
#[derive(Debug, Clone)]
pub enum UnaskedAnswer {
    Picks(Vec<ChoiceOption>),
    Number(u64),
    /// Each part, and how much went to it.
    Allocation(Vec<(ChoiceOption, u64)>),
    /// The items, first first.
    Order(Vec<ChoiceOption>),
}

/// Why the question was not asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unasked {
    /// A forced prompt: the question has one legal answer.
    OneLegalAnswer,
    /// An elision: every legal answer leaves one game.
    OneGame(Elision),
}

/// Which elision (`plans/elisions.md` §3), each with the premise written there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elision {
    /// CR 616.1: which of several replacement effects applies first.
    ReplacementOrder,
    /// CR 603.3b: the order one player's triggered abilities go on the stack.
    TriggerOrder,
    /// CR 603.7b: which simultaneous event triggers a delayed triggered ability.
    DelayedTriggerCause,
}
