//! Which cost modifications apply to this spell, right now.
//!
//! The mirror of `replacement::gather` and `restriction::predicate`, narrower:
//! one source today, the battlefield's cost-effect sources, read off their
//! effective ability lists. The spell's *own* cost abilities (CR 113.6d,
//! affinity) are CM-2's second source; a registry of resolution-created cost
//! effects is later (`cost-architecture.md` §4).

use crate::engine::layers::compute_characteristics;
use crate::engine::layers::condition::settled_holds;
use crate::objects::card_data::AbilityType;
use crate::oracle::characteristics::{controller_or_owner, get_effective_abilities};
use crate::state::game_state::GameState;
use crate::types::cost_modification::{CostModificationDef, CostSubject};
use crate::types::ids::{ObjectId, PlayerId};

/// One cost modification that applies to the spell being cast: whose it is,
/// and what it does.
#[derive(Debug, Clone, PartialEq)]
pub struct CostModificationInstance {
    /// The object whose ability this is — what the ordering prompt shows.
    pub source: ObjectId,
    /// Its controller at determination, CR 109.5's "you".
    pub controller: PlayerId,
    pub def: CostModificationDef,
}

/// Every cost modification that applies to `spell`, in battlefield timestamp
/// order.
///
/// The order is process-independent because CR 601.2f's ordering prompt
/// offers this list and a `DecisionProvider` answers by *index*
/// (`CLAUDE.md`, determinism).
///
/// **The frame every read here sees is a finished one.** CR 613.11 applies
/// cost effects after all other continuous effects, so a source's abilities
/// are its memoized top-level frame with every layer applied, and the spell's
/// frame is its stack frame — seeded from the `StackEntry` the cast wrote
/// before reaching 601.2f — or, for the castability preview, its hand frame,
/// whose controller is its owner and so the prospective caster (CR 108.4a).
/// No layer ceiling is chosen and no dependency question arises.
pub fn cost_modifications_for(game: &GameState, spell: ObjectId) -> Vec<CostModificationInstance> {
    // The fast-path gate, the same instrument as `replacement_ability_sources`
    // and carrying the same rule: a new source of static cost abilities, or a
    // new route onto the effective list, needs a leg here or it is silently
    // dead. Three legs — printed (the set), granted and copied (the two
    // summary flags). Exact and over-approximating: CR 305.7 and Humility can
    // strip a printed ability without touching the set, which costs a walk
    // and never an answer.
    let summary = game.continuous_effects.summary();
    let widened = summary.any_granted_cost_modification || summary.any_copied_cost_modification;
    if !widened && game.cost_modification_ability_sources.is_empty() {
        return Vec::new();
    }
    let Some(frame) = compute_characteristics(game, spell) else {
        return Vec::new();
    };

    // The sources, sorted — not the battlefield. A grant or a copy is not
    // attributed to an object (attributing it would resolve the grant's
    // filter per permanent per cast, which is what the flag exists to avoid),
    // so either flag widens this to every permanent; that is the rare board.
    let candidates: Vec<ObjectId> = if widened {
        game.battlefield_ids_ordered()
    } else {
        let mut pairs: Vec<(u64, ObjectId)> = game
            .cost_modification_ability_sources
            .iter()
            .filter_map(|id| game.battlefield.get(id).map(|entry| (entry.timestamp, *id)))
            .collect();
        // Timestamps are unique (CR 613.7; `CLAUDE.md`), so the key alone is
        // a total order and no `ObjectId` tiebreak is ever consulted.
        pairs.sort_by_key(|(ts, _)| *ts);
        pairs.into_iter().map(|(_, id)| id).collect()
    };

    let mut out = Vec::new();
    for id in candidates {
        let controller = controller_or_owner(game, id).unwrap_or(0);
        for ability in get_effective_abilities(game, id) {
            if ability.ability_type != AbilityType::Static {
                continue;
            }
            let Some((condition, def)) = ability.effect.as_cost_modification() else {
                continue;
            };
            // "As long as [X]" — LI-3's shape, read against the settled board,
            // which for a status leaf such as `SourceUntapped` is the entity
            // and for a characteristic leaf is the finished frame.
            if let Some(condition) = condition {
                if !settled_holds(condition, game, id) {
                    continue;
                }
            }
            if applies_to(game, def, controller, spell, &frame) {
                out.push(CostModificationInstance { source: id, controller, def: def.clone() });
            }
        }
    }
    out
}

/// Does this modification's subject include the spell?
fn applies_to(
    game: &GameState,
    def: &CostModificationDef,
    source_controller: PlayerId,
    spell: ObjectId,
    frame: &crate::engine::layers::types::EffectiveCharacteristics,
) -> bool {
    match &def.applies_to {
        // The frame-side matcher, with "you" the *source's* controller
        // (CR 109.5) and the spell's controller its caster — so "spells you
        // cast" is `ByController(You)` and "spells your opponents cast" is
        // `ByController(Opponent)`, with nothing spell-specific to add.
        CostSubject::Spells(filter) => game
            .object_matches_filter_in_frame(spell, filter, source_controller, frame)
            .unwrap_or(false),
    }
}
