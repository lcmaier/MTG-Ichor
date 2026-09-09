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
//! - **No per-layer existence re-check.** CR 604.2's re-check exists because a
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

use crate::engine::replacement::ReplacementInstanceId;
use crate::engine::resolve::ResolvedTarget;
use crate::state::duration_registry::{DurationRegistry, DurationRow, RowId};
use crate::types::effects::Duration;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::{ReplacementDef, Uses};

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
    /// The targets of the resolution that created this row, as chosen at cast
    /// (CR 601.2c) — kept because they are unrecoverable a moment later and a
    /// rider may need them.
    ///
    /// **Not the affected set.** For Mending Hands the two coincide (the row's
    /// `Fixed` *is* its target); for Divine Deflection they do not — "prevent
    /// the next X damage that would be dealt to you and/or permanents you
    /// control … Divine Deflection deals that much damage to any target" has
    /// its target chosen at cast and its affected set evaluated at the event,
    /// and its rider needs both (`plans/handoffs/rd.md`). Empty for a row no
    /// target chose: Safe Passage's, a regeneration's from an untargeted
    /// resolution.
    ///
    /// No reader yet: the `EffectRecipient` leaf that would say "the thing this
    /// effect targeted at resolution" arrives with the first rider that needs
    /// it, and threads this onto `ReplacementInstance` and `Rider` then
    /// (`codebase-state.md`, RD-2's Deferred Migrations line).
    pub targets: Vec<ResolvedTarget>,
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

impl DurationRegistry<RegisteredReplacementEffect> {
    /// Spend `prevented` of a row's CR 615.7 count, removing the row when it
    /// reaches zero.
    ///
    /// > 615.7 … Each 1 damage that would be dealt to the shielded permanent
    /// > or player is prevented. Preventing 1 damage reduces the remaining
    /// > shield by 1.
    ///
    /// In place through [`DurationRegistry::update_rows`], so the id survives
    /// and CR 614.5's applied-set key still names the same effect. Removal at
    /// zero is CR 615.3's "until they're used up"; the other end, "or their
    /// duration has expired", is the cleanup hook the registry already runs,
    /// and neither end knows about the other. Spending 0 is a no-op — CR
    /// 609.7b's "the shield isn't used up" — and is what an application that
    /// prevented nothing does here.
    ///
    /// Returns the count left, or `None` when the row was not a
    /// [`Uses::NextDamage`] row (or was already gone).
    pub fn spend_next_damage(&mut self, id: RowId, prevented: u64) -> Option<u64> {
        let mut left = None;
        self.update_rows(|row| {
            if row.id != id {
                return false;
            }
            if let Uses::NextDamage(remaining) = row.def.uses {
                let after = remaining.saturating_sub(prevented);
                row.def.uses = Uses::NextDamage(after);
                left = Some(after);
                return prevented > 0;
            }
            false
        });
        if left == Some(0) {
            self.remove(id);
        }
        left
    }
}

/// CR 614.13a/b's two exclusion sets, for one batch of simultaneous entries.
///
/// > 614.13a ... You can't choose the object that will become that permanent or
/// > any other object entering the battlefield at the same time as that object.
/// >
/// > 614.13b The same object can't be chosen to change zones more than once when
/// > applying replacement effects that modify how one or more permanents enter
/// > the battlefield.
///
/// **Two rules, two fields, and they are not the same question.** 614.13a is
/// about *this* batch's entries and is known before any replacement is applied;
/// 614.13b accumulates as they are applied and spans every entry in the batch,
/// which is what "one or more permanents" means.
///
/// **Why 614.13b is not redundant with the objects having moved.** It usually
/// is — a sacrificed creature is in a graveyard and no longer a candidate — and
/// the rule is written anyway, because the CR's own example is a single
/// Runeclaw Bear offered to devour 3 and to devour 5. Tracking it explicitly is
/// what makes the answer independent of whether a destination happens to fall
/// out of the next effect's filter.
///
/// Lives on `GameState`; `codebase-state.md` item 40 says why. Empty outside a
/// batch, and empty in every batch with no entry in it, which is nearly all of
/// them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EntrySelectionScope {
    /// CR 614.13a — every object entering the battlefield in this batch.
    ///
    /// A `Vec` and not a `HashSet` for the reason every ordered collection in
    /// this engine is one: it reaches a `DecisionProvider`'s option list by way
    /// of the candidates it removes, and a `HashSet` walk is not reproducible
    /// across processes. Batches are one or two members, so membership is a
    /// scan.
    pub entering: Vec<ObjectId>,

    /// CR 614.13b — every object already chosen to change zones while applying
    /// an entry replacement in this batch.
    pub chosen: Vec<ObjectId>,
}

impl EntrySelectionScope {
    /// May `id` be chosen to change zones (CR 614.13a, 614.13b)?
    pub fn admits(&self, id: ObjectId) -> bool {
        !self.entering.contains(&id) && !self.chosen.contains(&id)
    }
}

/// CR 615.7's allocation answers, for one batch of simultaneous damage.
///
/// > 615.7 … If damage would be dealt to the shielded permanent or player by
/// > two or more applicable sources at the same time, the player or the
/// > controller of the permanent chooses which damage the shield prevents.
///
/// **Per instance, not per subject group.** One "prevent the next N damage"
/// effect is one instance — one registry row — and it owns one count, so it is
/// asked once across every batch member it applies to, whatever their subjects:
/// Mending Hands' members share one subject, Divine Deflection's "you and/or
/// permanents you control" span several. The CR 616.1 loop decides one subject
/// group at a time (`replacement-architecture.md` §9, RD decision 3), so the
/// answer is taken the first time the instance is chosen in any group's loop
/// and read by the groups decided after it.
///
/// Lives on `GameState` for `codebase-state.md` item 40's reason: the shares
/// are consulted across later groups' CR 616.1 prompts, and a fork at one of
/// those has to see them. Saved and restored by `execute_batch_inner` beside
/// [`EntrySelectionScope`]; empty outside a batch, and empty in every batch no
/// CR 615.7 count is chosen in, which is nearly all of them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PreventionAllocationScope {
    /// Each chosen instance's answer: how much of its count each batch member
    /// (by index into the batch) is to be given. Members without an entry get
    /// nothing from that instance.
    ///
    /// **A `Vec` scanned linearly, not a map**, for [`EntrySelectionScope`]'s
    /// reasons and one more. The outer list holds one entry per CR 615.7 count
    /// *chosen in this batch* — zero in nearly every batch, one in the rest —
    /// and the inner one a share per bucket, which is the batch's own width:
    /// two or three. Hashing a `ReplacementInstanceId` to find one of one costs
    /// more than the comparison does. The third reason is the standing one: a
    /// `HashMap` walk is not reproducible across processes, and everything in
    /// this struct is downstream of a `DecisionProvider` answer.
    pub allocations: Vec<(ReplacementInstanceId, Vec<(usize, u64)>)>,
}

impl PreventionAllocationScope {
    /// The shares already decided for `instance`, if it has been asked.
    pub fn shares_for(&self, instance: ReplacementInstanceId) -> Option<&[(usize, u64)]> {
        self.allocations
            .iter()
            .find(|(id, _)| *id == instance)
            .map(|(_, shares)| shares.as_slice())
    }
}

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
            targets: Vec::new(),
            def: ReplacementDef::new(
                EventPattern::Destroy { source: None },
                AffectedSet::Fixed(vec![source]),
                Rewrite::Prevent,
            ),
        }
    }

    fn next_damage_row(source: ObjectId, n: u64) -> RegisteredReplacementEffect {
        use crate::types::replacement::AmountRewrite;
        RegisteredReplacementEffect {
            id: 0,
            source,
            controller: 0,
            duration: Duration::UntilEndOfTurn,
            created_on_turn: 1,
            targets: Vec::new(),
            def: ReplacementDef::new(
                EventPattern::DealDamage,
                AffectedSet::Fixed(vec![source]),
                Rewrite::Amount(AmountRewrite::PreventRemaining),
            )
            .next_damage(n),
        }
    }

    // CR 615.7 — "preventing 1 damage reduces the remaining shield by 1", in
    // place, with the id kept; CR 615.3's "until they're used up" at zero.
    #[test]
    fn spending_a_next_damage_count_keeps_the_row_until_it_reaches_zero() {
        let mut reg = ReplacementEffectRegistry::new();
        let src = Uuid::new_v4();
        let id = reg.add(next_damage_row(src, 4));

        assert_eq!(reg.spend_next_damage(id, 3), Some(1));
        assert_eq!(
            reg.iter().next().map(|r| (r.id, r.def.uses)),
            Some((id, Uses::NextDamage(1)))
        );

        // CR 609.7b — an application that prevented nothing spends nothing.
        assert_eq!(reg.spend_next_damage(id, 0), Some(1));
        assert_eq!(reg.len(), 1);

        assert_eq!(reg.spend_next_damage(id, 1), Some(0));
        assert!(reg.is_empty(), "used up");
        assert_eq!(reg.spend_next_damage(id, 1), None, "and gone");
    }

    // A `Once` row has no count to spend; the caller removes it whole.
    #[test]
    fn spending_is_only_for_next_damage_rows() {
        let mut reg = ReplacementEffectRegistry::new();
        let id = reg.add(row(Uuid::new_v4(), Duration::UntilEndOfTurn));
        assert_eq!(reg.spend_next_damage(id, 2), None);
        assert_eq!(reg.len(), 1);
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
