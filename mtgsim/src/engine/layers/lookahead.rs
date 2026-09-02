//! CR 614.12 / 614.17d — one object computed as it *would exist* on the
//! battlefield, as a read-side overlay on the layer walk.
//!
//! The walk reads concrete state in two places: the `BattlefieldEntity` it
//! seeds from (controller, CR 302.6's clock, counters) and the registry slice
//! it applies per layer. Both go through an accessor on `FrameCache`
//! (`entity` and `rows_in_layer` in `compute.rs`), and when the object being
//! computed is the entering one the accessor answers from this struct — the
//! entity the performer *would* build and the rows `register_static_effects`
//! *would* write. For every other object it answers from the real board.
//!
//! That one-object asymmetry is the whole design. `replacement-architecture.md`
//! §5d has the rest: CR 614.12's three clauses against the reads they perturb,
//! why this is not a `GameState` clone, why the entering object is invisible
//! to a count over the battlefield, and why the frame may live on the stack.

use crate::engine::layers::compute::{compute_to_ceiling, FrameCache, LAYER_ORDER};
use crate::engine::layers::types::{ContinuousEffect, EffectOrigin, EffectiveCharacteristics};
use crate::objects::card_data::AbilityType;
use crate::state::battlefield::BattlefieldEntity;
use crate::state::continuous_effects::RegistryScopeSummary;
use crate::state::game_state::GameState;
use crate::types::effects::Duration;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::EnterMods;

/// The look-ahead for one entering object — see the module docs.
///
/// Built once per frame request from the proposal being decided, and dropped
/// with it. Nothing here is written back anywhere.
pub struct Lookahead {
    pub object: ObjectId,
    /// The entity `place_on_battlefield` would build: the proposed controller,
    /// CR 302.6's clock started this turn, and CR 122.6a's counters from the
    /// pending `EnterMods` — CR 614.12 clause (1).
    pub(super) entity: BattlefieldEntity,
    /// CR 614.12 clause (2): the registry rows its own static abilities would
    /// generate, exactly as `register_static_effects` would write them.
    pub(super) rows: Vec<ContinuousEffect>,
    /// `rows` summarised as the registry summarises its own, so the walk's
    /// fast-path gates stay exact for rows the registry does not hold.
    pub(super) summary: RegistryScopeSummary,
}

impl Lookahead {
    /// `object` as it would exist on the battlefield under `controller`, having
    /// entered with `mods`.
    pub fn new(game: &GameState, object: ObjectId, controller: PlayerId, mods: &EnterMods) -> Self {
        // Timestamps as `place_on_battlefield` would allocate them — the
        // entity's, then one per counter kind in `mods` order — read off
        // `next_timestamp` without advancing it. Only the order matters: later
        // than every registered row, which is where CR 613.7a puts an object's
        // own static-ability effects and CR 613.7c its counters.
        let entity_timestamp = game.next_timestamp;
        let mut entity =
            BattlefieldEntity::new(object, controller, entity_timestamp, game.turn_number);
        entity.tapped = mods.tapped;
        let mut next = entity_timestamp + 1;
        for &(kind, n) in &mods.counters {
            entity.add_counters(kind, n, next);
            next += 1;
        }

        let rows = would_be_rows(game, object, controller, entity_timestamp);
        let summary = RegistryScopeSummary::of(&rows);

        Lookahead { object, entity, rows, summary }
    }
}

/// The rows `register_static_effects` would write for `object` entering under
/// `controller` — the same lowering, with no side effects and no assertions.
///
/// Printed abilities on purpose, as the real registration reads them: whether
/// the object still *has* each ability on the battlefield is CR 613.7a's
/// question, and `static_ability_still_exists` re-asks it at every layer
/// against this object's own frame — which is how Humility or Blood Moon strip
/// an entering permanent's static ability before it can apply to itself.
///
/// The loud arms of the lowering are left to the performer, which runs the
/// real registration a moment later on the same card and asserts there. A
/// card that lowers to nothing contributes nothing here, which is also what it
/// will contribute once it has entered.
fn would_be_rows(
    game: &GameState,
    object: ObjectId,
    controller: PlayerId,
    timestamp: u64,
) -> Vec<ContinuousEffect> {
    let Some(obj) = game.objects.get(&object) else {
        return Vec::new();
    };
    let card = &obj.card_data;
    let mut rows = Vec::new();

    for ability in &card.abilities {
        if ability.ability_type != AbilityType::Static {
            continue;
        }
        // CR 604.3a(3) — a CDA is never a registry row (`layers::cda`).
        if ability.is_characteristic_defining {
            continue;
        }
        // `Effect::Replacement` and `Effect::Restriction` lower to no atoms:
        // they are discovered off the effective ability list, not registered.
        for (primitive, recipient) in GameState::static_ability_atoms(ability, &card.name) {
            let Some(affected) = GameState::static_affected_set(recipient, &card.name) else {
                continue;
            };
            for (layer, modification) in GameState::static_primitive_rows(primitive) {
                rows.push(ContinuousEffect {
                    id: 0,
                    source: object,
                    origin: EffectOrigin::StaticAbility { ability: ability.id },
                    layer,
                    duration: Duration::WhileSourceOnBattlefield,
                    controller,
                    created_on_turn: game.turn_number,
                    timestamp,
                    affected: affected.clone(),
                    modification,
                });
            }
        }
    }

    rows
}

/// CR 614.12 / 614.17d — `id`'s characteristics as it *would exist* on the
/// battlefield under `controller`, with `pending` already applied.
///
/// The read-side counterpart of `compute_characteristics`: one full layer walk
/// of `id` with the [`Lookahead`] threaded through the walk's `FrameCache`, so
/// that the reads CR 614.12 names answer for the would-be permanent and every
/// read about any other object answers off the real board. Counted as a layer
/// walk, like every other top-level frame.
///
/// `id` may be anywhere: in the battlefield zone with no entity yet (the
/// replacement pipeline's case), or still in a hand, graveyard or on the stack
/// (a CR 614.17d "can't enter" asked at the zone change). The overlay makes it
/// a permanent for the walk's duration either way, and answers ahead of a real
/// entity if one exists, because the caller asked about the proposal.
pub fn compute_as_entering(
    game: &GameState,
    id: ObjectId,
    controller: PlayerId,
    pending: &EnterMods,
) -> Option<EffectiveCharacteristics> {
    game.counters.record_layer_walk();
    let lookahead = Lookahead::new(game, id, controller, pending);
    let mut cache = FrameCache::new(Some(&lookahead));
    compute_to_ceiling(game, id, LAYER_ORDER.len(), &mut cache)
}
