//! The three types that name one applicable replacement effect.
//!
//! Separated from the pipeline that consumes them because CR 614.5's identity
//! question is answered per *gather source* (`gather.rs`) while the loop that
//! keys on the answer lives in `pipeline.rs` — both need these and neither owns
//! them.

use crate::state::replacement_effects::ReplacementEffectId;
use crate::types::effects::CounterType;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::replacement::ReplacementDef;

use super::gather::CounterEffectKind;

/// Identifies one replacement *effect instance*, for CR 614.5.
///
/// > 614.5. A replacement effect doesn't invoke itself repeatedly ... it
/// > affects an event only once.
///
/// "Once" is per *effect*, so the key has to name the effect — not the object
/// that generated it and not the card that object is. Each variant is the
/// identity its source already uses elsewhere in the engine, which is what
/// stops this becoming a fourth notion of identity:
///
/// - a registry row is its `ReplacementEffectId`, never reused;
/// - a static ability is `(ObjectId, AbilityId)` — the same pair
///   `EffectOrigin::StaticAbility` and `activatable_abilities` use, because
///   `AbilityId` names a *definition* that two objects sharing an
///   `Arc<CardData>` also share;
/// - a counter-derived effect is the permanent plus the counter kind plus
///   which of CR 122.1c's two effects it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplacementInstanceId {
    /// A row in the replacement registry (CR 614.3, 615.7, 701.19a).
    Registered(ReplacementEffectId),
    /// A static ability of an object (CR 614.1a).
    StaticAbility(ObjectId, AbilityId),
    /// Synthesized from counters on a permanent (CR 122.1c/d/h).
    ///
    /// Three components rather than two because **CR 122.1c makes two effects
    /// from one counter**: "One or more shield counters on a permanent create a
    /// single replacement effect *and* a single prevention effect". They are
    /// separate effects and CR 614.5 tracks them separately.
    ///
    /// Note what the *count* does not do: 122.1c/d/h all say "one **or more**
    /// counters ... create **a single** replacement effect", so two stun
    /// counters do not give two applications to one event. Keying on the kind
    /// rather than on a counter gives that structurally.
    Counter(ObjectId, CounterType, CounterEffectKind),
    /// A replacement effect that belongs to no object's text — a *rule* that
    /// behaves as one. CR 903.9b is the only member. The `ObjectId` is the
    /// commander the rule is about, so two commanders leaving at once are two
    /// instances and CR 614.5 (or its 903.9b exception) applies to each.
    GameRule(ObjectId, GameRuleReplacement),
}

/// Which rule-shaped replacement effect a [`ReplacementInstanceId::GameRule`]
/// names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameRuleReplacement {
    /// CR 903.9b — a commander that would be put into its owner's hand or
    /// library may go to the command zone instead.
    CommanderZone,
}

/// One applicable replacement effect, resolved against the object that has it.
///
/// A gathered snapshot rather than a borrow: the loop mutates game state
/// between iterations (`consume_use`, and the rewrite itself), so a candidate
/// list holding `&GameState` could not survive one pass.
#[derive(Debug, Clone)]
pub struct ReplacementInstance {
    pub id: ReplacementInstanceId,
    /// The object generating it. Resolves `AffectedSet::SourceOnly` and is what
    /// a CR 616.1 prompt names.
    pub source: ObjectId,
    /// CR 109.5's "you" for this effect's filters — the source's *effective*
    /// controller for a static ability, the locked-in controller for a
    /// registry row.
    pub controller: PlayerId,
    pub def: ReplacementDef,
}
