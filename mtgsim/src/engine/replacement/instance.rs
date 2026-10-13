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

/// The replacement effects that have already had their one chance at an event
/// (CR 614.5: "it gets only one opportunity to affect an event or any modified
/// events that may replace that event"), in the order they had it. The CR 616.1
/// loop never offers one of these again, and the event's record keeps the ones
/// that applied. Zero to three long, so a list read linearly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedReplacements(Vec<Opportunity>);

/// How an effect used its one opportunity: it applied, or it was offered and
/// declined ("you may ... instead"), which uses it all the same.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Opportunity {
    Applied(AppliedReplacement),
    Declined(ReplacementInstanceId),
}

impl Opportunity {
    fn instance(&self) -> &ReplacementInstanceId {
        match self {
            Opportunity::Applied(applied) => &applied.instance,
            Opportunity::Declined(id) => id,
        }
    }
}

impl AppliedReplacements {
    /// An event nothing has replaced yet.
    pub const EMPTY: AppliedReplacements = AppliedReplacements(Vec::new());

    pub fn contains(&self, id: &ReplacementInstanceId) -> bool {
        self.0.iter().any(|opportunity| opportunity.instance() == id)
    }

    /// Adds an effect that applied, last, unless it is already here.
    pub fn apply(&mut self, applied: AppliedReplacement) {
        if !self.contains(&applied.instance) {
            self.0.push(Opportunity::Applied(applied));
        }
    }

    /// Adds an effect that was offered and declined, last, unless it is
    /// already here.
    pub fn decline(&mut self, id: ReplacementInstanceId) {
        if !self.contains(&id) {
            self.0.push(Opportunity::Declined(id));
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The effects that applied, in the order they did: what the record of
    /// the event keeps. `None` when none did.
    pub fn applied(&self) -> Option<Arc<[AppliedReplacement]>> {
        let applied: Vec<AppliedReplacement> = self
            .0
            .iter()
            .filter_map(|opportunity| match opportunity {
                Opportunity::Applied(applied) => Some(*applied),
                Opportunity::Declined(_) => None,
            })
            .collect();
        (!applied.is_empty()).then(|| applied.into())
    }

    /// The same effects, with none marked as having redirected: what a
    /// rider's events start from. They had their chance at the event the rider
    /// is the rest of (CR 614.5), and none of them moved the rider's own
    /// events anywhere.
    pub fn without_redirects(&self) -> AppliedReplacements {
        AppliedReplacements(
            self.0
                .iter()
                .map(|opportunity| match opportunity {
                    Opportunity::Applied(applied) => {
                        Opportunity::Applied(AppliedReplacement { redirected: false, ..*applied })
                    }
                    Opportunity::Declined(id) => Opportunity::Declined(*id),
                })
                .collect(),
        )
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
