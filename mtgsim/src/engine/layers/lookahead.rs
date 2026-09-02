//! CR 614.12 / 614.17d — one object computed as it *would exist* on the
//! battlefield.
//!
//! > 614.12. … To determine which replacement effects apply and how they
//! > apply, check the characteristics of the permanent as it would exist on
//! > the battlefield, taking into account replacement effects that have
//! > already modified how it enters the battlefield (see rule 616.1),
//! > continuous effects from the permanent's own static abilities that would
//! > apply to it once it's on the battlefield, and continuous effects that
//! > already exist and would apply to the permanent.
//!
//! # A read-side overlay, not a clone
//!
//! `replacement-architecture.md` §11 item 5 decided the shape: a `GameState`
//! clone duplicates `GameState.rng` against the determinism doctrine and
//! produces a second live copy of every v4 `ObjectId`, aliased rather than
//! distinguishable. So the hypothetical is expressed as the *reads* the layer
//! walk makes of concrete state, behind the accessor pair on `FrameCache`
//! (`entity` and `rows_in_layer` in `compute.rs`): each answers from this
//! struct when the object being computed is the entering one, and from the
//! real board for every other object.
//!
//! # One object is hypothetical; nothing else is (§5b)
//!
//! Each of CR 614.12's three clauses perturbs one read, and every perturbation
//! is scoped to the entering object alone:
//!
//! | Clause | Perturbed read |
//! |---|---|
//! | (1) replacements that already modified how it enters | the counters layers 6 and 7c read come from the pending `EnterMods` |
//! | (2) its own static abilities | `rows_in_layer` appends the rows `register_static_effects` *would* create, timestamped as the entry would timestamp them |
//! | (3) effects that already exist | RC-3 admitted the battlefield *zone* to filter matching; the overlay only widens that to an object not yet in the zone — a CR 614.17d "can't enter" asked at the zone change |
//!
//! Plus the seed. The frame's controller is the proposed one — CR 110.2b's
//! default, or a CR 616.1b rewrite of it — and CR 302.6's clock starts this
//! turn. `base_controller`'s battlefield probe never finds an entity for an
//! entering object, so without the seed a filter's "you control" would read
//! the owner, which is wrong for every permanent spell cast by a non-owner.
//!
//! What is **not** perturbed is anything about another object. The entering
//! permanent's anthem is in *its* frame and reaches no other object's, because
//! the would-be rows are appended only when computing the entering id; and an
//! `AmountExpr::CountOf` enumerates `battlefield_ids_ordered`, which the
//! entering object is not on. That asymmetry is §5a's Thassa boundary —
//! visible to filters, invisible to counts — and it falls out of the structure
//! rather than being special-cased anywhere.
//!
//! # Where it lives, and the decision-site invariant
//!
//! On the stack, threaded through `FrameCache`. `codebase-state.md` item 40
//! asks of any state a decision is taken against whether it is
//! *outcome-bearing*: drop it and re-derive, and does the game reach the same
//! outcome? This is a pure function of `GameState` and the proposal being
//! decided, so it does — it is bookkeeping, and bookkeeping may live off
//! `GameState`. The proposal itself, `apply_replacements`' `event`, is the
//! outcome-bearing thing, and it is already item 40's first violator.

use std::collections::{HashMap, HashSet};

use crate::engine::layers::compute::{compute_to_ceiling, FrameCache, LAYER_ORDER};
use crate::engine::layers::types::{
    ContinuousEffect, EffectGroup, EffectModification, EffectOrigin, EffectiveCharacteristics,
};
use crate::objects::card_data::AbilityType;
use crate::state::battlefield::CounterStack;
use crate::state::game_state::GameState;
use crate::types::effects::{CounterType, Duration};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::EnterMods;

/// The look-ahead for one entering object — see the module docs.
///
/// Built once per frame request from the proposal being decided, and dropped
/// with it. Nothing here is written back anywhere.
pub struct Lookahead {
    pub object: ObjectId,
    /// The controller it would enter under. `chars.controller`'s seed, and
    /// `base_controller`'s answer for this object for the walk's duration.
    pub controller: PlayerId,
    /// CR 302.6's clock, as the entry would set it.
    pub(super) controller_since_turn: u32,
    /// CR 122.6a — the counters it would enter with, as the performer would
    /// put them on: one stack per kind, timestamped after the entity's.
    pub(super) counters: HashMap<CounterType, CounterStack>,
    /// CR 614.12 clause (2): the registry rows its own static abilities would
    /// generate, exactly as `register_static_effects` would write them.
    pub(super) rows: Vec<ContinuousEffect>,
    /// `rows` holds an `EffectGroup` with more than one row, so the walk's
    /// CR 613.6 bookkeeping has to be on for this object's frame.
    pub(super) any_multi_row_group: bool,
    /// `rows` holds a `SetController`, so `effective_controller`'s gate may not
    /// short-circuit for this object's frame.
    pub(super) any_control_changing: bool,
}

impl Lookahead {
    /// `object` as it would exist on the battlefield under `controller`, having
    /// entered with `mods`.
    pub fn new(game: &GameState, object: ObjectId, controller: PlayerId, mods: &EnterMods) -> Self {
        // Timestamps as `place_on_battlefield` would allocate them: the
        // entity's first, then one per counter kind in `mods` order. The exact
        // values matter less than the order — both are later than every
        // registered row's, which is what CR 613.7a asks of an object's own
        // static-ability effects and CR 613.7c of its counters. Nothing is
        // allocated; `next_timestamp` is read, not advanced.
        let entity_timestamp = game.next_timestamp;
        let mut counters: HashMap<CounterType, CounterStack> = HashMap::new();
        let mut next = entity_timestamp + 1;
        for &(kind, n) in &mods.counters {
            let stack = counters
                .entry(kind)
                .or_insert(CounterStack { count: 0, timestamp: next });
            stack.count += n;
            stack.timestamp = next;
            next += 1;
        }

        let rows = would_be_rows(game, object, controller, entity_timestamp);
        let mut seen: HashSet<EffectGroup> = HashSet::with_capacity(rows.len());
        let any_multi_row_group = rows.iter().any(|row| !seen.insert(row.group()));
        let any_control_changing = rows
            .iter()
            .any(|row| matches!(row.modification, EffectModification::SetController(_)));

        Lookahead {
            object,
            controller,
            controller_since_turn: game.turn_number,
            counters,
            rows,
            any_multi_row_group,
            any_control_changing,
        }
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
/// that the three reads CR 614.12 names answer for the would-be permanent and
/// every read about any other object answers off the real board. Counted as a
/// layer walk, like every other top-level frame.
///
/// `id` may be anywhere: in the battlefield zone with no entity yet (the
/// replacement pipeline's case), or still in a hand, graveyard or on the stack
/// (a CR 614.17d "can't enter" asked at the zone change). The overlay makes it
/// a permanent for the walk's duration either way. It also wins over a real
/// entity, because a hypothetical about an object is about that object.
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
