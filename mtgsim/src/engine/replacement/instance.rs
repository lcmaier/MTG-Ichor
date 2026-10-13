//! The three types that name one applicable replacement effect.
//!
//! Separated from the pipeline that consumes them because CR 614.5's identity
//! question is answered per *gather source* (`gather.rs`) while the loop that
//! keys on the answer lives in `pipeline.rs` — both need these and neither owns
//! them.

use std::sync::Arc;

use crate::events::event::AppliedReplacement;
use crate::state::replacement_effects::ReplacementEffectId;
use crate::types::effects::CounterType;
use crate::types::ids::{AbilityId, ObjectId, ObjectRef, PlayerId};
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
/// - a static ability is `(ObjectRef, AbilityId)` — the pair
///   `EffectOrigin::StaticAbility` and `activatable_abilities` use, because
///   `AbilityId` names a *definition* that two objects sharing an
///   `Arc<CardData>` also share;
/// - a counter-derived effect is the permanent plus the counter kind plus
///   which of CR 122.1c's two effects it is.
///
/// **An object by identity, not by id** (CR 400.7). A CR 615.5 rider carries
/// the replaced event's applied set into its own events, and an object those
/// events have moved out and back is a new object whose abilities are new
/// effects. Keyed by id, its ability would read as already applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReplacementInstanceId {
    /// A row in the replacement registry (CR 614.3, 615.7, 701.19a).
    Registered(ReplacementEffectId),
    /// A static ability of an object (CR 614.1a).
    StaticAbility(ObjectRef, AbilityId),
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
    Counter(ObjectRef, CounterType, CounterEffectKind),
    /// A replacement effect that belongs to no object's text — a *rule* that
    /// behaves as one. CR 903.9b is the only member. The object is the
    /// commander the rule is about, so two commanders leaving at once are two
    /// instances and CR 614.5 (or its 903.9b exception) applies to each.
    GameRule(ObjectRef, GameRuleReplacement),
}

/// What CR 616.1's loop has done with replacement effects for one event and
/// the events that replace it. CR 614.5 gives each effect "only one
/// opportunity to affect an event or any modified events that may replace that
/// event", and an effect uses it by applying or, for a "you may ... instead",
/// by being declined. The loop offers neither kind again, and the event's
/// record keeps the ones that applied, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedReplacements {
    /// In the order they applied.
    applied: Vec<AppliedReplacement>,
    /// Optional effects their chooser declined.
    declined: Vec<ReplacementInstanceId>,
}

impl AppliedReplacements {
    /// An event nothing has replaced yet.
    pub const EMPTY: AppliedReplacements = AppliedReplacements { applied: Vec::new(), declined: Vec::new() };

    /// Whether `id` has had its one opportunity: it applied, or was declined.
    pub fn contains(&self, id: &ReplacementInstanceId) -> bool {
        self.applied.iter().any(|a| a.instance == *id) || self.declined.contains(id)
    }

    /// Adds an effect that applied, last, unless it has had its opportunity.
    pub fn apply(&mut self, applied: AppliedReplacement) {
        if !self.contains(&applied.instance) {
            self.applied.push(applied);
        }
    }

    /// Adds an optional effect its chooser declined, unless it has had its
    /// opportunity.
    pub fn decline(&mut self, id: ReplacementInstanceId) {
        if !self.contains(&id) {
            self.declined.push(id);
        }
    }

    pub fn len(&self) -> usize {
        self.applied.len() + self.declined.len()
    }

    /// The effects that applied, in the order they did: what the record of
    /// the event keeps. `None` when none did.
    pub fn applied(&self) -> Option<Arc<[AppliedReplacement]>> {
        (!self.applied.is_empty()).then(|| self.applied.as_slice().into())
    }

    /// The same effects, with none marked as having redirected: what a
    /// rider's events start from. They had their chance at the event the rider
    /// is the rest of (CR 614.5), and none of them moved the rider's own
    /// events anywhere.
    pub fn without_redirects(&self) -> AppliedReplacements {
        AppliedReplacements {
            applied: self.applied.iter().map(|a| AppliedReplacement { redirected: false, ..*a }).collect(),
            declined: self.declined.clone(),
        }
    }
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
    /// The object generating it. Resolves `ObjectSet::SourceOnly` and is what
    /// a CR 616.1 prompt names.
    pub source: ObjectId,
    /// CR 109.5's "you" for this effect's filters — the source's *effective*
    /// controller for a static ability, the locked-in controller for a
    /// registry row.
    pub controller: PlayerId,
    pub def: ReplacementDef,
}
