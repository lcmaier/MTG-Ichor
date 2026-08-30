//! The CR 616.1 replacement pipeline.
//!
//! `execute_actions` proposes; this decides what actually gets performed.
//! Everything here runs *before* the mutation, which is the only instant at
//! which CR 614.4's "the replacement effect must exist before the event" can
//! honestly be asked.
//!
//! - [`instance`] names one applicable effect — CR 614.5's identity key.
//! - [`gather`] finds the applicable effects — `replacement-architecture.md`
//!   §3.3's five sources.
//! - [`apply_replacements`] is §4.1's loop: CR 616.1a–f, CR 614.5's applied
//!   set, CR 614.17c's blocked-event path.
//!
//! Riders ([`Rider`]) are queued here and resolved by the caller **after** the
//! surviving event is performed (CR 615.5, §4.1a).
//!
//! For the whole shipped type surface on one page, and one action traced from
//! `execute_actions` to a performed `GameEvent`, see
//! `plans/replacement-architecture.md` §2a.

mod gather;
mod instance;
mod pipeline;

pub(crate) use gather::{chooser_for, gather, subject_of, EventSubject};
pub use gather::CounterEffectKind;
pub(crate) use instance::{GameRuleReplacement, ReplacementInstanceId};
pub use instance::ReplacementInstance;
pub(crate) use pipeline::{apply_replacements, Rider};
