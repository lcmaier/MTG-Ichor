//! Cost modification — CR 601.2f's order, and CR 613.11's cost half
//! (`plans/cost-architecture.md`).
//!
//! `gather.rs` finds the cost effects that apply to one spell; `total.rs` is
//! CR 601.2f's arithmetic and its order, and the one entry point the cast
//! pipeline calls. Nothing here is a registry row: a cost effect is read off
//! its source's *effective* ability list at the moment a cost is determined,
//! exactly as `engine::replacement::gather` and
//! `engine::restriction::predicate` read theirs, and for the same reason —
//! that read *is* CR 604.2's existence check, so a Thalia under Humility
//! stops taxing with nothing to reconcile.

mod gather;
mod total;

pub use gather::{gather, CostModificationInstance};
pub use total::{determine_total_cost, preview_mana_cost};
