//! Continuous effects / layer system (CR 613).
//!
//! This module implements the layer system that computes effective
//! characteristics for game objects by applying continuous effects
//! in the correct order.
//!
//! Public API:
//! - `compute_characteristics(game, id)` — the single entry point for
//!   all characteristic queries.
//! - Types: `Layer`, `ContinuousEffect`, `EffectModification`, etc.

pub mod types;
pub mod compute;

pub use compute::compute_characteristics;
pub use types::*;
