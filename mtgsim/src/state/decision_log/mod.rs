//! The decision log: every answer a game's choices get, in the order it gets
//! them, written by the engine so that the record is the game's and not a
//! seat's (A6g's playable PR, the owner's call at its design review,
//! 2026-10-02).
//!
//! A seat's provider sees only the prompts that reach it, and that depends on
//! how the seat is set up: auto-yield and auto-pay answer some prompts above
//! it, and full control makes the engine ask at a priority point where passing
//! is all the player can do, where it otherwise passes itself. So the log is
//! written at the two places every answer passes: the four `ui::ask`
//! validators, whoever answered, and the priority loop's own pass. A pass is
//! one line whoever made it, and the same game writes the same log whatever
//! its seats were set to. A replay that has every seat stop at every priority
//! point is asked each line's question in turn.
//!
//! **An observer, never a participant**, as the trace sink is: writing a line
//! draws from no rng, asks no provider and changes no control flow, and a
//! writer reads the turn and the step and never characteristics, since a read
//! through the layers counts a walk. Off by default: a game nobody logs pays
//! one branch on an `Option` per answer.
//!
//! **A fork logs nothing.** `GameState` derives `Clone` and a search forks it
//! at every decision; a log is one line of play, so a clone has none until it
//! is given its own.
//!
//! **The text is the engine's** (`setup-architecture.md` §7.2, decision 6),
//! so every client's records say the same words and one reader reads them: a
//! client attaches a writer and keeps the lines wherever it keeps records. A
//! record opens with its format and the engine that wrote it, then where the
//! game began ([`GameStart`]), then an answer a line ([`AnswerLine`]), each
//! naming what it chose rather than where the option sat, so a later build
//! replays it until the game itself differs; it ends with the outcome once
//! there is one. The text is a function of the game, with no clock in it, so
//! two records of one game differ only in their engine line and in the path
//! a scenario was read from.

mod hook;
mod start;
mod text;

pub use hook::{DecisionLogHandle, LoggedAnswer, LoggedDecision};
pub use start::{BuiltStart, GameStart};
pub use text::{AnswerLine, Chosen, FORMAT, Log, LogError, Outcome, opening, read};
