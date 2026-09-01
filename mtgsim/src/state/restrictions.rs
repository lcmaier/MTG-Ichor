//! Storage for "can't" effects created by resolutions (CR 101.2, 611.2a).
//!
//! The fourth customer `duration_registry.rs` predicted, and it needs a
//! [`DurationRow`] impl and nothing else. `SortKey = ()`: no read of this
//! registry wants an order, because CR 101.2 has no tiebreak among
//! prohibitions — two "can't"s agree, so there is nothing for an order to
//! decide. `add` is therefore a push and this type is a **type alias**, on
//! `codebase-state.md` item 34's rule: compose where the wrapper has its own
//! surface, alias where it does not.
//!
//! # This registry is one of five sources, and the smallest
//!
//! `cant-effects-architecture.md` §3.4 lists where restrictions come from, and
//! four of the five never reach here — for the same structural reason
//! `replacement_effects.rs` gives:
//!
//! - **Static abilities of permanents** are swept off each object's *effective*
//!   ability list, which is what makes Humility strip a restriction for free.
//! - **Keyword abilities** (indestructible, and the largest source by card
//!   count) are synthesized during that sweep, because nothing on the card
//!   prints the CR 702.12b text.
//! - **Static abilities functioning in other zones** are deferred on the same
//!   terms as `gather`'s source 2.
//! - **Rules of the game** — CR 614.17b's derived cost prohibition, RS-4's.
//!
//! What lands here is source 4 alone: a row a resolution created, with the
//! duration that resolution's text gives it. Which is why
//! [`Primitive::Restrict`](crate::types::effects::Primitive::Restrict) carries
//! a `Duration` argument rather than inferring one — CR 608.2c hands scope
//! determination to a human reader ("apply the rules of English to the text"),
//! so it has to be authored per card (§9 finding 1).

use crate::state::duration_registry::{DurationRegistry, DurationRow, RowId};
use crate::types::effects::Duration;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::restriction::RestrictionDef;

/// Unique identifier for a registered restriction.
pub type RestrictionId = u64;

/// One "can't" created by a resolving spell or ability.
#[derive(Debug, Clone)]
pub struct RegisteredRestriction {
    /// Unique id for this row. Never reused, on the shared allocation
    /// discipline — see [`DurationRow::id`].
    pub id: RestrictionId,
    /// The object whose spell or ability created this.
    pub source: ObjectId,
    /// The player who controlled that spell or ability. Resolves `PlayerRef::You`
    /// in the def's filters, and is CR 611.2c's locked-at-resolution value.
    pub controller: PlayerId,
    /// When it stops existing. **Authored, never inferred** — see the module
    /// docs and §9 finding 1.
    pub duration: Duration,
    /// The turn it was created on, so a player-relative duration does not
    /// expire on the turn it was made.
    pub created_on_turn: u32,
    /// What it forbids.
    pub def: RestrictionDef,
}

/// `SortKey = ()` — see the module docs. Nothing reads these rows in an order,
/// so the id tiebreak alone decides placement and `add` is an append.
impl DurationRow for RegisteredRestriction {
    type SortKey = ();

    fn id(&self) -> RowId {
        self.id
    }
    fn set_id(&mut self, id: RowId) {
        self.id = id;
    }
    fn source(&self) -> ObjectId {
        self.source
    }
    fn duration(&self) -> Duration {
        self.duration
    }
    fn controller(&self) -> PlayerId {
        self.controller
    }
    fn created_on_turn(&self) -> u32 {
        self.created_on_turn
    }
    fn sort_key(&self) -> Self::SortKey {}
}

/// Every "can't" a resolution has created.
pub type RestrictionRegistry = DurationRegistry<RegisteredRestriction>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::effects::AffectedSet;
    use crate::types::replacement::EventPattern;
    use crate::types::restriction::{Restriction, RestrictionDef};
    use uuid::Uuid;

    fn row(source: ObjectId, duration: Duration) -> RegisteredRestriction {
        RegisteredRestriction {
            id: 0,
            source,
            controller: 0,
            duration,
            created_on_turn: 1,
            def: RestrictionDef::new(Restriction::Event {
                pattern: EventPattern::Destroy { source: None },
                affected: AffectedSet::Fixed(vec![source]),
                by: None,
            }),
        }
    }

    /// The CR 514.2 expiry the generic owns, asserted once here so that a
    /// duration bug fixed in one registry and not another cannot hide — which
    /// is the failure mode `duration_registry.rs` exists to remove.
    #[test]
    fn test_until_end_of_turn_expires_at_cleanup() {
        let mut reg = RestrictionRegistry::new();
        let src = Uuid::new_v4();
        reg.add(row(src, Duration::UntilEndOfTurn));
        reg.add(row(src, Duration::Indefinite));

        assert_eq!(reg.remove_expired_at_cleanup(0, 1).len(), 1);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.iter().next().unwrap().duration, Duration::Indefinite);
    }

    #[test]
    fn test_registration_order_survives_removal() {
        let mut reg = RestrictionRegistry::new();
        let sources: Vec<ObjectId> = (0..3).map(|_| Uuid::new_v4()).collect();
        for s in &sources {
            reg.add(row(*s, Duration::UntilEndOfTurn));
        }
        reg.remove_by_source(sources[1]);
        let order: Vec<ObjectId> = reg.iter().map(|r| r.source).collect();
        assert_eq!(order, vec![sources[0], sources[2]]);
    }
}
