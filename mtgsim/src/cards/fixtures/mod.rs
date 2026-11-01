//! The invented cards: nothing here is a printing. Each makes an engine path
//! testable, and is filed by the architecture doc whose path it tests, or by
//! the CR subject it exists for when no doc owns one. A fixture must not wear
//! a real card's name while behaving differently
//! (`engineering-practices.md#cards-and-pools`). One shaped like a printed
//! card takes that card's name with "Fixture" after it, or "Spell Fixture"
//! when it is an instant standing in for a permanent's effect; an invented
//! one takes a name no card prints.

mod layers;
mod replacement;
mod cost;
mod mana;
mod triggers;
mod state_based_actions;

pub use layers::*;
pub use replacement::*;
pub use cost::*;
pub use mana::*;
pub use triggers::*;
pub use state_based_actions::*;
