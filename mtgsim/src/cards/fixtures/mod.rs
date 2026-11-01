//! The invented cards: nothing here is a printing. Each makes an engine path
//! testable, and is filed by the architecture doc whose path it tests, or by
//! the CR subject it exists for when no doc owns one. A fixture must not wear
//! a real card's name while behaving differently
//! (`engineering-practices.md#cards-and-pools`); eight here still do, until
//! `creature-types-plus-fixture-names` settles each.

mod layers;
mod replacement;
mod cost;
mod triggers;
mod state_based_actions;

pub use layers::*;
pub use replacement::*;
pub use cost::*;
pub use triggers::*;
pub use state_based_actions::*;
