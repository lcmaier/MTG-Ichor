//! A board described at rest, and its text (`plans/setup-architecture.md`).
//!
//! A scenario is the state at the start of a step's priority round, with the
//! stack empty and nothing waiting to trigger. [`Scenario::parse`] reads a
//! file, [`Scenario::write`] writes a game's board, and [`Scenario::build`]
//! makes a `Game` of either through the engine's construction doors, emitting
//! nothing. The grammar, every word with its default, is the design's §5.1
//! table; `mtgsim/scenarios/template.scenario` shows each word in use.

mod board;
mod build;
mod error;
mod text;
mod write;

pub use board::{
    Arrival, Attacked, CardLine, CardWord, LineKind, LineNumbered, NamedCard, PlayerWord, Scenario, SetupAction, SetupVerb, Targeted,
};
pub use error::{ScenarioError, ScenarioErrorKind};
pub use write::WrittenBoard;
