// First, so every module declared after it has `counted_enum!` in scope.
#[macro_use]
mod counted_enum;
pub mod ids;
pub mod colors;
pub mod mana;
pub mod card_types;
pub mod zones;
pub mod keywords;
pub mod keyword_actions;
pub mod costs;
pub mod effects;
pub mod replacement;
pub mod restriction;
pub mod cost_modification;
pub mod triggers;
pub mod history;
