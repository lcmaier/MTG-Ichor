//! The engine's question boundary: what a seat is asked, the kinds and options
//! it is asked in, the `DecisionProvider` trait every client answers through,
//! and the providers built beside it: a seat router, the engine's own two for
//! a rule's draw and an audit's clone, and a scripted provider for tests.
pub mod decision_types;
pub mod ask;
pub mod provider;
