//! Storage for replacement effects created by resolutions (CR 614.3, 615.7,
//! 701.19a).
//!
//! This is the data owner — lives on `GameState`. The pipeline that consumes
//! it lives in `engine/replacement`.
//!
//! # Why this is not `ContinuousEffectRegistry`
//!
//! What the two have in common they now *share* rather than copy:
//! [`DurationRegistry`] owns the rows, the id counter and the CR 514.2 expiry
//! hooks for both. What is left here differs in two ways, both because
//! replacement effects are not layered:
//!
//! - **No `Layer`, no timestamp ordering.** CR 616.1 orders by *player choice*,
//!   not by timestamp, so there is no analogue of `effects_in_layer`. Insertion
//!   order is preserved anyway, because it is the order the CR 616.1 prompt
//!   offers candidates in and a `DecisionProvider` picks by index — the same
//!   reason `battlefield_ids_ordered` exists.
//! - **No per-layer existence re-check.** CR 613.7a's re-check exists because a
//!   layer walk asks the same question nine times. A replacement effect is asked
//!   once, at the instant the event is proposed, which is exactly when CR 614.4
//!   wants it asked.
//!
//! # `remove_by_source` has no production caller, and battlefield-leave is not one
//!
//! The mirror call in `cleanup_zone_state` is right for
//! `ContinuousEffectRegistry` — CR 611.3b, a *static ability's* effect lasts
//! only while its source is on the battlefield — and wrong here: every row in
//! this registry was made by a resolution, and CR 611.2a gives those the
//! duration the spell or ability stated. A regeneration shield does not die
//! with the permanent whose ability made it; it expires at the CR 514.2 cleanup
//! with the rest of the turn.
//!
//! It earns a caller the day a row carries a source-scoped duration (CR 611.2b's
//! "for as long as ..."), which is a `DurationRegistry::retain` keyed on
//! duration *and* source, not on source alone — `codebase-state.md` item 17.
//!
//! # What is *not* in here
//!
//! Three of `replacement-architecture.md` §3.3's five sources never reach this
//! registry, and that is structural rather than incidental:
//!
//! - **Static abilities of permanents** are discovered by sweeping the
//!   battlefield and reading each object's *effective* ability list. Not a
//!   registry scan — that is what makes Humility and Blood Moon strip a
//!   replacement ability for free.
//! - **Counters** (CR 122.1c/d/h) come from the *counter*, not from any
//!   ability; nothing on the card says so. They are synthesized during the same
//!   sweep.
//! - **Static abilities functioning in other zones** are deferred past Phase RE.

use crate::state::duration_registry::{DurationRegistry, DurationRow, RowId};
use crate::types::effects::Duration;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::ReplacementDef;

/// Unique identifier for a registered replacement effect.
pub type ReplacementEffectId = u64;

/// One replacement effect created by a resolving spell or ability.
#[derive(Debug, Clone)]
pub struct RegisteredReplacementEffect {
    /// Unique id for this instance. Part of the CR 614.5 applied-set key, so it
    /// must never be reused — [`ReplacementEffectRegistry::add`] only ever counts up.
    pub id: ReplacementEffectId,
    /// The object whose spell or ability created this.
    pub source: ObjectId,
    /// The player who controlled that spell or ability. Resolves `PlayerRef::You`
    /// in the def's filters, and is CR 611.2c's locked-at-resolution value.
    pub controller: PlayerId,
    /// When it stops existing. Expiry runs through the same cleanup/turn-start
    /// hooks `ContinuousEffectRegistry` already uses.
    pub duration: Duration,
    /// The turn it was created on, so a player-relative duration does not
    /// expire on the turn it was made.
    pub created_on_turn: u32,
    /// What it watches for and what it does.
    pub def: ReplacementDef,
}

/// `SortKey = ()` — CR 616.1 orders by player choice, not by any stored key, so
/// the id tiebreak alone decides placement and `add` is an append. That is the
/// registration order the CR 616.1 prompt offers candidates in, and a
/// `DecisionProvider` picks by index.
impl DurationRow for RegisteredReplacementEffect {
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

/// Every replacement effect a resolution has created.
///
/// A [`DurationRegistry`] and nothing more. `cant-effects-architecture.md` §9
/// finding 7 expected a wrapper here keeping "`Uses::Once` removal, gather-order
/// iteration" — but `Uses::Once` removal *is* `remove(id)` (CR 701.19a: one
/// shield, one destruction replaced) and gather-order iteration *is* `iter()`,
/// so every method the wrapper had was a one-line delegation that made a reader
/// ask what it was adding.
///
/// If a replacement-specific method does earn its place — item 17's
/// source-scoped expiry is the candidate — it goes on an inherent
/// `impl DurationRegistry<RegisteredReplacementEffect>`, which is legal because
/// the generic is local to this crate. This does not have to become a struct
/// again to grow one.
pub type ReplacementEffectRegistry = DurationRegistry<RegisteredReplacementEffect>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::effects::AffectedSet;
    use crate::types::replacement::{EventPattern, ReplacementDef, Rewrite};
    use uuid::Uuid;

    fn row(source: ObjectId, duration: Duration) -> RegisteredReplacementEffect {
        RegisteredReplacementEffect {
            id: 0,
            source,
            controller: 0,
            duration,
            created_on_turn: 1,
            def: ReplacementDef::new(
                EventPattern::Destroy { source: None },
                AffectedSet::Fixed(vec![source]),
                Rewrite::Prevent,
            ),
        }
    }

    #[test]
    fn test_ids_are_never_reused() {
        // The id is part of the CR 614.5 applied-set key: a reused one would
        // let a fresh effect inherit an earlier one's "already applied" mark.
        let mut reg = ReplacementEffectRegistry::new();
        let src = Uuid::new_v4();
        let a = reg.add(row(src, Duration::UntilEndOfTurn));
        reg.remove(a);
        let b = reg.add(row(src, Duration::UntilEndOfTurn));
        assert_ne!(a, b);
    }

    #[test]
    fn test_remove_by_source() {
        let mut reg = ReplacementEffectRegistry::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        reg.add(row(a, Duration::UntilEndOfTurn));
        reg.add(row(a, Duration::Indefinite));
        reg.add(row(b, Duration::UntilEndOfTurn));

        assert_eq!(reg.remove_by_source(a).len(), 2);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.iter().next().unwrap().source, b);
    }

    #[test]
    fn test_registration_order_survives_removal() {
        // Order is decision order: a `DecisionProvider` picks a CR 616.1
        // candidate by index.
        let mut reg = ReplacementEffectRegistry::new();
        let sources: Vec<ObjectId> = (0..4).map(|_| Uuid::new_v4()).collect();
        for s in &sources {
            reg.add(row(*s, Duration::UntilEndOfTurn));
        }
        reg.remove_by_source(sources[1]);
        let order: Vec<ObjectId> = reg.iter().map(|e| e.source).collect();
        assert_eq!(order, vec![sources[0], sources[2], sources[3]]);
    }

    #[test]
    fn test_until_end_of_turn_expires_at_cleanup() {
        let mut reg = ReplacementEffectRegistry::new();
        let src = Uuid::new_v4();
        reg.add(row(src, Duration::UntilEndOfTurn));
        reg.add(row(src, Duration::WhileSourceOnBattlefield));

        assert_eq!(reg.remove_expired_at_cleanup(0, 1).len(), 1);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.iter().next().unwrap().duration, Duration::WhileSourceOnBattlefield);
    }

    #[test]
    fn test_until_your_next_turn_does_not_expire_on_the_turn_it_was_made() {
        let mut reg = ReplacementEffectRegistry::new();
        let src = Uuid::new_v4();
        reg.add(row(src, Duration::UntilYourNextTurn));

        assert_eq!(reg.remove_expired_at_turn_start(0, 1).len(), 0, "same turn");
        assert_eq!(reg.remove_expired_at_turn_start(1, 2).len(), 0, "wrong player");
        assert_eq!(reg.remove_expired_at_turn_start(0, 3).len(), 1);
        assert!(reg.is_empty());
    }
}
