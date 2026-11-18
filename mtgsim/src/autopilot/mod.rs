//! What answers for a seat, chosen by its client: the random agent answers
//! every question, and the decorators stacked over a provider answer some of
//! them, with full control's switch to hand them back to the person.
pub mod random;
pub mod mana_window_stop;
pub mod full_control;
pub mod auto_yield;
pub mod auto_order_triggers;
