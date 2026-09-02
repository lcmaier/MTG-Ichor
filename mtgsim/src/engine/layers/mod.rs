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
pub mod cda;
pub mod compute;
pub mod copy;
pub mod land_types;
pub mod lookahead;

pub use compute::compute_characteristics;
pub use copy::{copiable_values, CopiableValues};
pub use lookahead::compute_as_entering;
pub use types::*;
