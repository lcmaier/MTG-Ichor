//! The policies a seat chooses: the random agent, and the decorators stacked
//! over a provider to answer some of its questions for it.
pub mod random;
pub mod mana_window_stop;
pub mod full_control;
pub mod auto_yield;
pub mod auto_order_triggers;
