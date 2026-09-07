//! Which cost modifications apply to this spell, right now.
//!
//! The mirror of `replacement::gather` and `restriction::predicate`, narrower:
//! two sources, each read off an *effective* ability list — the battlefield's
//! cost-effect sources (CM-1), and the spell itself (CR 113.6d, affinity;
//! CM-2). A registry of resolution-created cost effects is later
//! (`cost-architecture.md` §4).

use crate::engine::layers::compute_characteristics;
use crate::engine::layers::condition::settled_holds;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::objects::card_data::{AbilityDef, AbilityType};
use crate::oracle::characteristics::{controller_or_owner, get_effective_abilities};
use crate::state::game_state::GameState;
use crate::types::cost_modification::{CostModificationDef, CostSubject};
use crate::types::ids::{ObjectId, PlayerId};

/// One cost modification that applies to the spell being cast: whose it is,
/// and what it does.
#[derive(Debug, Clone, PartialEq)]
pub struct CostModificationInstance {
    /// The object whose ability this is — what the ordering prompt shows, and
    /// what a dynamic amount is read against. For source 2 it is the spell.
    pub source: ObjectId,
    /// Its controller at determination, CR 109.5's "you".
    pub controller: PlayerId,
    pub def: CostModificationDef,
}

/// Every cost modification that applies to `spell`: the battlefield's sources
/// in timestamp order, then the spell's own.
///
/// The order is process-independent because CR 601.2f's ordering prompt
/// offers this list and a `DecisionProvider` answers by *index*
/// (`CLAUDE.md`, determinism). The spell's own goes last so that adding it
/// did not move an index.
///
/// **The frame every read here sees is a finished one.** CR 613.11 applies
/// cost effects after all other continuous effects, so a source's abilities
/// are its memoized top-level frame with every layer applied, and the spell's
/// frame is its stack frame — seeded from the `StackEntry` the cast wrote
/// before reaching 601.2f — or, for the castability preview, its hand frame,
/// whose controller is its owner and so the prospective caster (CR 108.4a).
/// No layer ceiling is chosen and no dependency question arises.
///
/// **Which zone licenses which read.** CR 113.6d puts a cost ability of the
/// object being cast on the *stack*, full stop — and CR 702.41a says the same
/// of affinity by name. So the hand-frame read is not a zone-function claim:
/// it is the castability preview asking what the spell *would* cost, and it
/// reads the spell's own abilities because enumeration has to agree with
/// enforcement (§3.6). The CR never has to answer that question; the engine
/// does.
pub fn cost_modifications_for(game: &GameState, spell: ObjectId) -> Vec<CostModificationInstance> {
    // The fast-path gate, the same instrument as `replacement_ability_sources`
    // and carrying the same rule: a new source of static cost abilities, or a
    // new route onto the effective list, needs a leg here or it is silently
    // dead. Three legs — printed (the set, or the spell's own card), granted
    // and copied (the two summary flags). Exact and over-approximating:
    // CR 305.7 and Humility can strip a printed ability without touching the
    // set, which costs a walk and never an answer.
    let summary = game.continuous_effects.summary();
    let widened = summary.any_granted_cost_modification || summary.any_copied_cost_modification;
    let sweep = widened || !game.cost_modification_ability_sources.is_empty();
    if !sweep && !prints_cost_ability(game, spell) {
        return Vec::new();
    }
    let Some(frame) = compute_characteristics(game, spell) else {
        return Vec::new();
    };

    let mut out = Vec::new();

    // Source 1 — the sources, sorted; not the battlefield. A grant or a copy
    // is not attributed to an object (attributing it would resolve the
    // grant's filter per permanent per cast, which is what the flag exists to
    // avoid), so either flag widens this to every permanent; that is the rare
    // board.
    if sweep {
        let candidates: Vec<ObjectId> = if widened {
            game.battlefield_ids_ordered()
        } else {
            let mut pairs: Vec<(u64, ObjectId)> = game
                .cost_modification_ability_sources
                .iter()
                .filter_map(|id| game.battlefield.get(id).map(|entry| (entry.timestamp, *id)))
                .collect();
            // Timestamps are unique (CR 613.7; `CLAUDE.md`), so the key alone
            // is a total order and no `ObjectId` tiebreak is ever consulted.
            pairs.sort_by_key(|(ts, _)| *ts);
            pairs.into_iter().map(|(_, id)| id).collect()
        };
        for id in candidates {
            let controller = controller_or_owner(game, id).unwrap_or(0);
            for ability in get_effective_abilities(game, id) {
                collect(game, &ability, id, controller, spell, &frame, &mut out);
            }
        }
    }

    // Source 2 — the spell's own cost abilities (CR 113.6d, 702.41a), off the
    // frame already computed for source 1's filter match. "You" is the
    // spell's controller, which is its caster on the stack (CR 601.2a) and
    // its owner in the preview (CR 108.4a) — the same player either way.
    for ability in frame.abilities.iter() {
        collect(game, ability, spell, frame.controller, spell, &frame, &mut out);
    }

    out
}

/// Take one ability's cost modification, if it has one that applies.
fn collect(
    game: &GameState,
    ability: &AbilityDef,
    source: ObjectId,
    controller: PlayerId,
    spell: ObjectId,
    frame: &EffectiveCharacteristics,
    out: &mut Vec<CostModificationInstance>,
) {
    if ability.ability_type != AbilityType::Static {
        return;
    }
    let Some((condition, def)) = ability.effect.as_cost_modification() else {
        return;
    };
    // "As long as [X]" — LI-3's shape, read against the settled board, which
    // for a status leaf such as `SourceUntapped` is the entity and for a
    // characteristic leaf is the finished frame. A spell on the stack has no
    // entity, so a status clause on one is simply false.
    if let Some(condition) = condition {
        if !settled_holds(condition, game, source) {
            return;
        }
    }
    if applies_to(game, def, source, controller, spell, frame) {
        out.push(CostModificationInstance { source, controller, def: def.clone() });
    }
}

/// Does this modification's subject include the spell?
///
/// The two arms partition the two sources by construction, which is why
/// neither caller passes a flag saying which one it is: `source == spell`
/// exactly when the ability came off the spell's own list.
fn applies_to(
    game: &GameState,
    def: &CostModificationDef,
    source: ObjectId,
    source_controller: PlayerId,
    spell: ObjectId,
    frame: &EffectiveCharacteristics,
) -> bool {
    match &def.applies_to {
        // The frame-side matcher, with "you" the *source's* controller
        // (CR 109.5) and the spell's controller its caster — so "spells you
        // cast" is `ByController(You)` and "spells your opponents cast" is
        // `ByController(Opponent)`, with nothing spell-specific to add.
        //
        // **Source 1's alone.** CR 113.6d licenses an object's own ability to
        // modify what *that particular object* costs and says nothing wider,
        // so a `Spells` subject printed on the object being cast is not
        // consulted; source 2 is `Itself` only (`cost-architecture.md` §4).
        CostSubject::Spells(filter) => {
            source != spell
                && game
                    .object_matches_filter_in_frame(spell, filter, source_controller, frame)
                    .unwrap_or(false)
        }
        // CR 113.6d's "that particular object" — an identity test. Only
        // source 2 can satisfy it, since a permanent on the battlefield is
        // never the object being cast.
        CostSubject::Itself => source == spell,
    }
}

/// The gate's printed leg for source 2: does this object's *card* carry a
/// static cost ability at all?
///
/// **This reads `card_data` for an object on the stack**, which the
/// layer-system invariant otherwise forbids — and it is a gate, not an
/// answer, in exactly the sense `cost_modification_ability_sources` is one.
/// The answer still comes from the effective list above; this only decides
/// whether the frame is worth computing. It has to: without it, source 2
/// would put one `compute_characteristics` on every card in hand at every
/// castability preview, on every board, for a mechanic 364 printed cards
/// have — which is a layer walk per hand card per epoch bought for nothing.
///
/// **Exact today, and the two flags are the legs that keep it honest.**
/// `compute_non_member` applies no registry rows, so a non-member's effective
/// ability list *is* its printed list: neither a Layer 6 grant nor a Layer 1
/// copy can reach a spell on the stack or a card in hand. The moment one can
/// — CR 113.6e's second sentence is the rule that would license it — the
/// caller's `widened` leg is what catches it, which is why this is OR'd with
/// the summary flags rather than consulted alone.
fn prints_cost_ability(game: &GameState, spell: ObjectId) -> bool {
    game.objects.get(&spell).is_some_and(|obj| {
        obj.card_data.abilities.iter().any(|ability| {
            ability.ability_type == AbilityType::Static
                && ability.effect.as_cost_modification().is_some()
        })
    })
}
