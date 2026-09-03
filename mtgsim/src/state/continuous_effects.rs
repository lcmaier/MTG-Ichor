//! Storage for active continuous effects (CR 613).
//!
//! This is the data owner — lives on GameState. The computation logic
//! lives in `engine/layers/compute.rs`.

use crate::engine::layers::types::{ContinuousEffect, EffectId, Layer, Timestamp};
use crate::state::duration_registry::{DurationRegistry, DurationRow, RowId};
use crate::types::effects::Duration;
use crate::types::ids::{ObjectId, PlayerId};

/// CR 613.7's storage order: layer first, then timestamp, with the registry's
/// own id breaking ties. `effects_in_layer` binary-searches on it.
impl DurationRow for ContinuousEffect {
    type SortKey = (Layer, Timestamp);

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
    fn sort_key(&self) -> Self::SortKey {
        (self.layer, self.timestamp)
    }
}

/// Cheap, registry-wide facts that let `compute_characteristics` skip work it
/// would otherwise have to do per object per layer.
///
/// `layers-architecture.md` §5.1 describes this struct with three more fields
/// (`touches_hidden_zones`, `touches_stack`, `has_active_cdas`) for the
/// hidden-zone fast path and CDA handling. Those land with the systems that
/// need them; the struct is introduced here with the one field that has a
/// caller, so the later work extends it rather than growing a parallel counter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryScopeSummary {
    /// True iff some `EffectGroup` has more than one row in the registry.
    ///
    /// CR 613.6's "started applying" bookkeeping only changes an answer for a
    /// CR-level effect the registry split across several rows — a single-row
    /// group is marked as started immediately after its only row applies, and
    /// nothing ever reads the mark. When this is false, `compute.rs` skips the
    /// bookkeeping entirely rather than hashing an `EffectGroup` twice per
    /// effect per layer. Worth 3.3x on a static-heavy board; see the comment
    /// on the `started` set in `apply_effects`.
    pub any_multi_row_group: bool,

    /// True iff some row can change an object's controller — i.e. some
    /// `EffectModification::SetController` is registered.
    ///
    /// `compute::effective_controller` needs this because CR 109.5 resolves a
    /// static ability's "you" against the source's *effective* controller, and
    /// asking for that means a `compute_to_ceiling` walk of the source. When
    /// this is false, `chars.controller` provably cannot differ from the value
    /// the walk seeds it with — `BattlefieldEntity.controller`, or the owner
    /// off the battlefield — because Layer 2 is the only channel that writes
    /// it: no CDA lives in Layer 2 (CR 613.4a lists only 7a), and no counter
    /// touches controller. So the field read is the walk's answer, not an
    /// approximation of it.
    ///
    /// This matters because `effect_applies_to` runs *before* the CR 613.7a
    /// existence check and therefore for objects the filter rejects, unlike the
    /// existence check, which only runs for effects that already matched.
    ///
    /// **Load-bearing, not a trim.** Forcing it off costs 83.7 → 107.2 ms/game
    /// on `fuzz_games --games 200 --seed 12345` with a control-changing card in
    /// the pool. It was worth ~4% when `FilterPlayers::you()` was its only
    /// caller; the Layer 2 phase put 20 more behind it, several inside
    /// per-permanent sweeps.
    ///
    /// A sharper per-object version was built and measured and is not faster.
    /// `codebase-state.md` Deferred Migrations item 13 has the numbers and the
    /// reason, so it does not get rebuilt on the same reasoning.
    pub any_control_changing: bool,

    /// True iff some row grants an ability whose body is an
    /// `Effect::Replacement` — i.e. some object on the battlefield may have a
    /// replacement ability that it did not print.
    ///
    /// The other half of `engine::replacement::gather`'s fast-path gate.
    /// `GameState::replacement_ability_sources` records printed replacement
    /// abilities at ETB; a Layer 6 `GrantAbility` can put one on an object that
    /// never printed it, and nothing at ETB could have known. Between the two
    /// the gate is **sound**: an object can only *have* a static replacement
    /// ability if it printed one or a registry row granted it one.
    ///
    /// Narrower than "any grant at all" on purpose. Granting keywords and
    /// abilities is common — the gate would be permanently on, and the fast
    /// path would buy nothing on exactly the boards that are most expensive to
    /// walk.
    pub any_granted_replacement: bool,

    /// True iff some row grants an ability whose body is an
    /// `Effect::Restriction` — i.e. some object on the battlefield may have a
    /// restriction ability that it did not print.
    ///
    /// The other half of `engine::restriction::is_prohibited`'s gate, and a
    /// separate flag rather than a widening of `any_granted_replacement` for
    /// that flag's own stated reason: the two sweeps read different ability
    /// bodies, so a shared flag would turn each one's fast path on for the
    /// other's cards.
    pub any_granted_restriction: bool,

    /// True iff some `EffectModification::CopyFrom` row's captured values carry
    /// a static replacement ability — i.e. some object on the battlefield may
    /// have a replacement ability it neither printed nor was granted.
    ///
    /// The **third** leg of `engine::replacement::gather`'s gate, and the one
    /// `copy-effects-architecture.md` §4.7 was written to predict: `gather`
    /// reads the *effective* ability list, and CR 707.2a puts a copied ability
    /// on that list through neither of the other two legs. Without it a copied
    /// replacement effect is silently dead on every board the gate skips.
    ///
    /// A summary flag rather than an insert into
    /// `GameState::replacement_ability_sources`, and the reason is that set's
    /// own doc comment: it is "a set rather than a count, so it cannot drift",
    /// and it is cleared only at `cleanup_zone_state`. A copy row can **expire**
    /// with no zone change, so the set would drift high. Drifting high costs a
    /// wasted walk and never an answer — but quietly falsifying that sentence
    /// is worse than one more field, and the summary is recomputed from the rows
    /// on every mutation, so it cannot drift at all.
    pub any_copied_replacement: bool,

    /// True iff some `EffectModification::CopyFrom` row's captured values carry
    /// a static restriction ability.
    ///
    /// The same leg on the *other* gate. `copy-effects-architecture.md` §4.7
    /// counted one; RS-1 had already built a second to the same recipe
    /// (`engine::restriction::predicate`), whose own comment names this phase as
    /// the owner. Split from `any_copied_replacement` for the reason
    /// `any_granted_restriction` is split from `any_granted_replacement`: the two
    /// sweeps read different ability bodies, so a shared flag would turn each
    /// one's fast path on for the other's cards.
    pub any_copied_restriction: bool,
}

impl RegistryScopeSummary {
    /// The flags for one list of rows, in one pass.
    ///
    /// The registry's own rows, or the rows a `layers::lookahead::Lookahead`
    /// holds *outside* it for an entering object — the walk's gates have to
    /// answer the same way for both, so one function computes both.
    pub fn of<'a>(effects: impl IntoIterator<Item = &'a ContinuousEffect>) -> Self {
        use crate::engine::layers::types::{EffectGroup, EffectModification};
        use crate::types::effects::Effect;

        let mut seen: std::collections::HashSet<EffectGroup> = std::collections::HashSet::new();
        let mut summary = RegistryScopeSummary::default();
        for effect in effects {
            if !seen.insert(effect.group()) {
                summary.any_multi_row_group = true;
            }
            match &effect.modification {
                EffectModification::SetController(_) => summary.any_control_changing = true,
                EffectModification::GrantAbility(def) => match def.effect {
                    Effect::Replacement(_) => summary.any_granted_replacement = true,
                    Effect::Restriction(_) => summary.any_granted_restriction = true,
                    _ => {}
                },
                // CR 707.2a — the captured list is scanned rather than counted,
                // because both gates ask about a *body*, not about a copy. A
                // copy of a vanilla creature must not turn either fast path on.
                EffectModification::CopyFrom(values) => {
                    for ability in &values.abilities {
                        match ability.effect {
                            Effect::Replacement(_) => summary.any_copied_replacement = true,
                            Effect::Restriction(_) => summary.any_copied_restriction = true,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        summary
    }
}

/// Owns all active continuous effects in the game.
///
/// The `Vec`, the id counter and the CR 514.2 expiry hooks live in
/// [`DurationRegistry`], which this type owns and delegates to; what stays here
/// is what is specific to *layered* effects — the CR 613.7 layer slice and the
/// summary flags.
#[derive(Debug, Clone)]
pub struct ContinuousEffectRegistry {
    effects: DurationRegistry<ContinuousEffect>,
    summary: RegistryScopeSummary,
    /// How many times the rows have changed. One half of
    /// `GameState::layer_epoch`, kept here because `mutating` is the one route
    /// to the rows and cannot reach the state that owns the other half.
    mutations: u64,
}

impl ContinuousEffectRegistry {
    pub fn new() -> Self {
        ContinuousEffectRegistry {
            effects: DurationRegistry::new(),
            summary: RegistryScopeSummary::default(),
            mutations: 0,
        }
    }

    /// Registry-wide summary flags. See `RegistryScopeSummary`.
    pub fn summary(&self) -> &RegistryScopeSummary {
        &self.summary
    }

    /// How many times the rows have changed — see `GameState::layer_epoch`.
    pub fn mutations(&self) -> u64 {
        self.mutations
    }

    /// Run a mutation against the rows, then rebuild the summary.
    ///
    /// Every mutating method funnels through here. The summary is derived
    /// state, and a path that skipped the rebuild would show up as a silently
    /// skipped existence check — the exact class of bug the Layer 2 phase
    /// existed to remove.
    fn mutating<R>(&mut self, f: impl FnOnce(&mut DurationRegistry<ContinuousEffect>) -> R) -> R {
        let before = self.effects.len();
        let out = f(&mut self.effects);
        // The rows are a layer-walk input, and this is their bump. `len` tells
        // a write from a no-op exactly: every `DurationRegistry` mutator adds
        // or removes rows and none edits one in place, and each closure here
        // makes one call. Exactness matters because both CR 514.2 expiry paths
        // run every turn and usually remove nothing — a bump for nothing would
        // cost every memoized frame at each turn boundary.
        if self.effects.len() != before {
            self.mutations += 1;
        }
        self.recompute_summary();
        out
    }

    /// Recompute `summary` from scratch.
    ///
    /// A full walk on every mutation rather than incremental counters: adds and
    /// removes are rare next to reads, and a counter that drifts would show up
    /// as a silently skipped existence check.
    fn recompute_summary(&mut self) {
        self.summary = RegistryScopeSummary::of(self.effects.iter());

        debug_assert!(
            self.effects.is_sorted(),
            "registry order invariant violated after mutation"
        );
    }

    /// Register a new continuous effect. Returns its unique ID.
    ///
    /// Placement is `DurationRegistry::add`'s: `(layer, timestamp)` from
    /// `sort_key`, then the id it just allocated. `EffectId` as the sub-order is
    /// doing real work rather than breaking an accidental tie — see
    /// `effects_in_layer`.
    pub fn add(&mut self, effect: ContinuousEffect) -> EffectId {
        // CR 604.3a(3) — a CDA affects only the object that has it, so it needs
        // no `AffectedSet` and never becomes a registry row. `layers::cda`
        // applies them off the object's own ability list instead. Layer 7a is
        // the CDA-only sublayer, so a row landing here means someone routed a
        // CDA through registration; catch it at the door rather than letting it
        // apply twice.
        debug_assert!(
            effect.layer != Layer::Layer7aCdaPT,
            "Layer 7a is applied intrinsically, never from the registry              (CR 604.3a(3)); effect from source {:?} tried to register there",
            effect.source
        );

        self.mutating(|rows| rows.add(effect))
    }

    /// Remove a specific effect by its ID. Returns the removed effect if found.
    pub fn remove(&mut self, id: EffectId) -> Option<ContinuousEffect> {
        self.mutating(|rows| rows.remove(id))
    }

    /// Remove all effects generated by a given source object.
    /// Used when a permanent leaves the battlefield (CR 611.3b).
    pub fn remove_by_source(&mut self, source: ObjectId) -> Vec<ContinuousEffect> {
        self.mutating(|rows| rows.remove_by_source(source))
    }

    /// All effects in a layer, already in application order (CR 613.7).
    ///
    /// No sorting happens here. The rows are *maintained* in
    /// `(layer, timestamp, id)` order by `add`, so a layer is a contiguous
    /// range and this is two binary searches and a slice.
    ///
    /// `EffectId` is the sub-order within a timestamp, and it is doing real
    /// work rather than breaking an accidental tie: CR 613.7a gives every
    /// continuous effect from a static ability of one object *the object's*
    /// timestamp, so all of an object's static-ability effects share one by
    /// design. Ids are assigned in registration order and never reused, which
    /// is what 613.7a's "the relative order of those timestamps remains the
    /// same" asks for.
    ///
    /// Removals preserve order for the same reason — see
    /// `DurationRegistry::retain`.
    ///
    /// (Not yet CR 613.8: dependency ordering is unimplemented, so this is
    /// timestamp order only. See Deferred Migrations item 8.)
    pub fn effects_in_layer(&self, layer: Layer) -> &[ContinuousEffect] {
        debug_assert!(self.effects.is_sorted(), "registry order invariant violated");
        let all = self.effects.as_slice();
        let lo = all.partition_point(|e| e.layer < layer);
        let hi = all.partition_point(|e| e.layer <= layer);
        &all[lo..hi]
    }

    /// Iterate over all registered effects.
    pub fn iter(&self) -> impl Iterator<Item = &ContinuousEffect> {
        self.effects.iter()
    }

    /// Returns true if no effects are registered.
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    /// Number of registered effects.
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Remove effects that expire during the cleanup step (rule 514.2).
    pub fn remove_expired_at_cleanup(
        &mut self,
        active_player: PlayerId,
        current_turn: u32,
    ) -> Vec<ContinuousEffect> {
        self.mutating(|rows| rows.remove_expired_at_cleanup(active_player, current_turn))
    }

    /// Remove effects that expire at the start of a player's turn.
    pub fn remove_expired_at_turn_start(
        &mut self,
        active_player: PlayerId,
        current_turn: u32,
    ) -> Vec<ContinuousEffect> {
        self.mutating(|rows| rows.remove_expired_at_turn_start(active_player, current_turn))
    }

    /// Allocate the next timestamp value.
    pub fn next_timestamp(counter: &mut Timestamp) -> Timestamp {
        let ts = *counter;
        *counter += 1;
        ts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::layers::types::*;
    use crate::types::effects::Duration;
    use uuid::Uuid;
    use crate::test_support::registered_source_only;

    fn make_effect(source: ObjectId, layer: Layer, timestamp: Timestamp) -> ContinuousEffect {
        registered_source_only(
            source,
            layer,
            timestamp,
            EffectModification::ModifyPowerToughness {
                power: PtValue::Fixed(1),
                toughness: PtValue::Fixed(1),
            },
        )
    }

    #[test]
    fn test_add_and_iter() {
        let mut reg = ContinuousEffectRegistry::new();
        let src = Uuid::new_v4();
        let id = reg.add(make_effect(src, Layer::Layer7cModifyPT, 1));
        assert_eq!(id, 1);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.iter().next().unwrap().id, 1);
        assert_eq!(reg.add(make_effect(src, Layer::Layer7cModifyPT, 2)), 2, "ids ascend");
    }

    #[test]
    fn test_remove_by_id() {
        let mut reg = ContinuousEffectRegistry::new();
        let src = Uuid::new_v4();
        let id1 = reg.add(make_effect(src, Layer::Layer7cModifyPT, 1));
        let _id2 = reg.add(make_effect(src, Layer::Layer7cModifyPT, 2));
        assert_eq!(reg.len(), 2);

        let removed = reg.remove(id1);
        assert!(removed.is_some());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_remove_by_source() {
        let mut reg = ContinuousEffectRegistry::new();
        let src_a = Uuid::new_v4();
        let src_b = Uuid::new_v4();
        reg.add(make_effect(src_a, Layer::Layer7cModifyPT, 1));
        reg.add(make_effect(src_a, Layer::Layer6Ability, 2));
        reg.add(make_effect(src_b, Layer::Layer7cModifyPT, 3));

        let removed = reg.remove_by_source(src_a);
        assert_eq!(removed.len(), 2);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.iter().next().unwrap().source, src_b);
    }

    // COVERS-PARTIAL: ATOM-613.7-001
    #[test]
    fn test_effects_in_layer_sorted_by_timestamp() {
        let mut reg = ContinuousEffectRegistry::new();
        let src = Uuid::new_v4();
        reg.add(make_effect(src, Layer::Layer7cModifyPT, 5));
        reg.add(make_effect(src, Layer::Layer7cModifyPT, 2));
        reg.add(make_effect(src, Layer::Layer6Ability, 3));
        reg.add(make_effect(src, Layer::Layer7cModifyPT, 8));

        let layer7c = reg.effects_in_layer(Layer::Layer7cModifyPT);
        assert_eq!(layer7c.len(), 3);
        assert_eq!(layer7c[0].timestamp, 2);
        assert_eq!(layer7c[1].timestamp, 5);
        assert_eq!(layer7c[2].timestamp, 8);
    }

    #[test]
    fn test_is_empty() {
        let mut reg = ContinuousEffectRegistry::new();
        assert!(reg.is_empty());
        let src = Uuid::new_v4();
        reg.add(make_effect(src, Layer::Layer7cModifyPT, 1));
        assert!(!reg.is_empty());
    }

    // COVERS-PARTIAL: ATOM-611.2a-001
    #[test]
    fn test_remove_expired_at_cleanup_removes_until_end_of_turn() {
        let mut reg = ContinuousEffectRegistry::new();
        let src = Uuid::new_v4();
        // UntilEndOfTurn effect
        reg.add(make_effect(src, Layer::Layer7cModifyPT, 1));
        // WhileSourceOnBattlefield effect (should NOT be removed)
        let mut permanent_effect = make_effect(src, Layer::Layer7cModifyPT, 2);
        permanent_effect.duration = Duration::WhileSourceOnBattlefield;
        reg.add(permanent_effect);

        assert_eq!(reg.len(), 2);
        let removed = reg.remove_expired_at_cleanup(0, 1);
        assert_eq!(removed.len(), 1);
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.iter().next().unwrap().duration, Duration::WhileSourceOnBattlefield);
    }

    #[test]
    fn test_remove_expired_at_turn_start_until_your_next_turn() {
        let mut reg = ContinuousEffectRegistry::new();
        let src = Uuid::new_v4();

        // Effect created by player 0 on turn 1, duration UntilYourNextTurn
        let mut effect = make_effect(src, Layer::Layer7cModifyPT, 1);
        effect.duration = Duration::UntilYourNextTurn;
        effect.controller = 0;
        effect.created_on_turn = 1;
        reg.add(effect);

        // Turn 1, player 0's turn — should NOT expire (same turn it was created)
        let removed = reg.remove_expired_at_turn_start(0, 1);
        assert_eq!(removed.len(), 0);
        assert_eq!(reg.len(), 1);

        // Turn 2, player 1's turn — should NOT expire (wrong player)
        let removed = reg.remove_expired_at_turn_start(1, 2);
        assert_eq!(removed.len(), 0);
        assert_eq!(reg.len(), 1);

        // Turn 3, player 0's next turn — SHOULD expire
        let removed = reg.remove_expired_at_turn_start(0, 3);
        assert_eq!(removed.len(), 1);
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn test_until_your_next_turn_expires_on_extra_turn() {
        // Extra turns advance turn_number, so an extra turn for the controller
        // IS their "next turn" and the effect correctly expires at its start.
        let mut reg = ContinuousEffectRegistry::new();
        let src = Uuid::new_v4();

        let mut effect = make_effect(src, Layer::Layer7cModifyPT, 1);
        effect.duration = Duration::UntilYourNextTurn;
        effect.controller = 0;
        effect.created_on_turn = 1;
        reg.add(effect);

        // Turn 2 is an extra turn for player 0 — effect expires (turn 2 > 1)
        let removed = reg.remove_expired_at_turn_start(0, 2);
        assert_eq!(removed.len(), 1);
    }
}
