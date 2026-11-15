//! The invented cards: nothing here is a printing. Each makes an engine path
//! testable, and is filed by the architecture doc whose path it tests, or by
//! the CR subject it exists for when no doc owns one. A fixture is a
//! half-card or an invented card, never a printed card's whole text: that is
//! the card, registered in its first printing's set with its rulings
//! (`engineering-practices.md#cards-and-pools`). A half-card takes the
//! card's name with "Fixture" after it, or "Spell Fixture" when it is an
//! instant standing in for a permanent's effect, and must not behave
//! differently from the part it keeps; an invented one takes a name no card
//! prints.

mod choices;
mod layers;
mod replacement;
mod cost;
mod mana;
mod triggers;
mod state_based_actions;

pub use choices::*;
pub use layers::*;
pub use replacement::*;
pub use cost::*;
pub use mana::*;
pub use triggers::*;
pub use state_based_actions::*;
