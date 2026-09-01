//! The CR 101.2 predicate and the sweep that feeds it.
//!
//! `gather.rs`'s counterpart, and deliberately its mirror image: the same five
//! sources in the same order, the same battlefield walk over *effective* ability
//! lists, and the same two matchers — [`pattern_watches`] and [`set_affects`],
//! imported rather than reimplemented. What is missing is the `Rewrite` half, a
//! candidate list, and any notion of order; see [`is_prohibited`] for why the
//! last one is a property of CR 101.2 rather than a simplification.

use crate::engine::actions::GameAction;
use crate::engine::replacement::{pattern_watches, set_affects, subject_of, EventSubject};
use crate::objects::card_data::AbilityType;
use crate::oracle::characteristics::{controller_or_owner, get_effective_abilities};
use crate::state::game_state::GameState;
use crate::types::effects::{AffectedSet, Effect, PlayerRef};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::EventPattern;
use crate::types::restriction::{
    ReplacementKindFilter, Restriction, RestrictionDef, SourceFilter,
};

/// What [`is_prohibited`] is being asked about.
///
/// One query type rather than one predicate per enforcement point, for §3.5's
/// reason: a second entry point is a second place for the answer to drift, and
/// CR 101.3's query side has to call the same function the chokepoints do.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Query<'a> {
    /// CR 614.17 — may this proposed event happen?
    Event {
        action: &'a GameAction,
        /// Who controls the spell or ability that proposed it — CR 101.2 scoped
        /// by cause, which is [`SourceFilter`]'s whole job.
        ///
        /// `None` for a turn-based or state-based action, which has no
        /// controller. Read off `ActionContext::resolution`, which already
        /// threads it.
        cause: Option<PlayerId>,
    },
    /// CR 701.19c / 615.12 — may this replacement effect be applied?
    ApplyReplacement {
        kind: ReplacementKindFilter,
        subject: EventSubject,
    },
}

/// CR 101.2 — is this prohibited right now?
///
/// The one reader of every restriction. Returns `true` when *any* source
/// forbids it: CR 101.2 has no tiebreak because it needs none — two
/// prohibitions agree, so this is a disjunction and never an ordering.
pub(crate) fn is_prohibited(game: &GameState, query: &Query) -> bool {
    game.counters.record_restriction_query();

    // --- Source 3: keyword abilities -------------------------------------
    //
    // Ahead of the gate, because `get_effective_abilities` is not how a keyword
    // is found and the gate would skip it (§3.5 commitment 2).
    if let Query::Event { action, .. } = query {
        if let EventSubject::Object(id) = subject_of(action) {
            if keyword_prohibits(game, id, action) {
                return true;
            }
        }
    }

    // The fast-path gate, and it is exact rather than a heuristic — the same
    // instrument as `replacement_ability_sources`, and it carries the same rule:
    // **a new source of static restriction abilities must add a leg here, or the
    // source is silently dead on every board the gate skips.** An object on the
    // battlefield can only *have* a static restriction ability if it printed one
    // (recorded at ETB) or a Layer 6 row granted it one (the registry summary).
    // Both over-approximate — CR 305.7 and Humility strip a printed ability
    // without touching the set — and over-approximating costs a layer walk,
    // never an answer.
    //
    // Sound only until Layer 1 or Layer 3 exists: each is a route to an object's
    // effective ability list that neither leg can see (`codebase-state.md` item
    // 16, and CV-1 owns the third leg).
    let has_static_source = !game.restriction_ability_sources.is_empty()
        || game.continuous_effects.summary().any_granted_restriction;
    if !has_static_source && game.restrictions.is_empty() {
        return false;
    }

    // --- Source 1: the battlefield sweep ---------------------------------
    if has_static_source {
        for id in game.battlefield_ids_ordered() {
            let controller = controller_or_owner(game, id).unwrap_or(0);
            for ability in get_effective_abilities(game, id) {
                if ability.ability_type != AbilityType::Static {
                    continue;
                }
                let Effect::Restriction(def) = &ability.effect else {
                    continue;
                };
                if matches(game, def, id, controller, query) {
                    return true;
                }
            }
        }
    }

    // --- Source 4: the registry ------------------------------------------
    for row in game.restrictions.iter() {
        if matches(game, &row.def, row.source, row.controller, query) {
            return true;
        }
    }

    false
}

/// Does this restriction forbid what the query asks about?
///
/// The outer `match` is on a **pair**: the restriction's axis on the left, the
/// question's axis on the right. Only the two same-axis pairs can match, which
/// is why the cross pairs return `false` rather than being an error — a
/// `Restriction::ApplyReplacement` simply has nothing to say about a proposed
/// event, the way a lock has nothing to say about a door it does not fit.
///
/// # Worked example: Sigarda vs. an opponent's Diabolic Edict
///
/// P0 controls Sigarda and a Bear. P1's edict resolves, and P0 chooses the
/// Bear to sacrifice — so before offering that candidate, the filter asks:
///
/// ```text
/// def  = Restriction::Event {
///            pattern:  ZoneChange { from: Battlefield, to: Graveyard,
///                                   cause: Sacrificed, object: None },
///            affected: Filter { ByController(You) },
///            by:       Some(ControlledBy(Opponent)),
///        }
/// source     = <Sigarda>          controller = P0   (CR 109.5: hers, now)
/// query      = Query::Event {
///                  action: ZoneChange { object: <Bear>, Battlefield ->
///                                       Graveyard, cause: Sacrificed },
///                  cause:  Some(P1),      // who controls the edict
///              }
/// ```
///
/// Three questions, all of which must say yes:
///
/// 1. **`pattern_watches`** — is this the *kind* of event Sigarda names? The
///    proposal is a battlefield→graveyard move caused by `Sacrificed`, and the
///    pattern's three `Some(..)` fields all agree. **Yes.** (A *destruction*
///    would carry `cause: Destroyed` and fail here, which is CR 701.21b: a
///    sacrifice is not a destruction.)
/// 2. **`set_affects`** — is the event's subject inside her affected set? The
///    subject is the Bear, the set is "permanents you control", and `you` is
///    resolved against `controller`, which is P0. **Yes.** (Had P1's own
///    creature been the subject, this would be where it failed.)
/// 3. **`cause_matches`** — was it caused by the right sort of source? `by`
///    says "a player who is not the restriction's controller"; `cause` is P1
///    and `controller` is P0. **Yes.** (P0's *own* edict would give
///    `cause == Some(P0)` and fail here — the field exists for exactly this.)
///
/// So the Bear is dropped from the candidate list. Sigarda is asked the same
/// three questions about herself and answers the same way, the list comes out
/// empty, and CR 101.3 ignores the instruction — no prompt.
///
/// # And the other axis
///
/// "Destroy target creature. It can't be regenerated" registers a
/// `Restriction::ApplyReplacement { kind: Regeneration, to: Fixed([creature]) }`.
/// When `gather` later finds a regeneration shield around that creature it asks
/// `Query::ApplyReplacement { kind: Regeneration, subject: <creature> }`, and
/// the second arm answers with the *same* `set_affects` call — the kinds agree
/// and the creature is in the set, so the shield is withheld. The destruction
/// still happens, which is the whole reason this cannot be an `Event` arm.
fn matches(
    game: &GameState,
    def: &RestrictionDef,
    source: ObjectId,
    controller: PlayerId,
    query: &Query,
) -> bool {
    match (&def.what, query) {
        (Restriction::Event { pattern, affected, by }, Query::Event { action, cause }) => {
            pattern_watches(game, pattern, action, controller)
                && set_affects(game, affected, source, controller, subject_of(action))
                && cause_matches(by.as_ref(), *cause, controller)
        }

        (
            Restriction::ApplyReplacement { kind, to },
            Query::ApplyReplacement { kind: asked, subject },
        ) => kind == asked && set_affects(game, to, source, controller, *subject),

        (Restriction::Event { .. }, Query::ApplyReplacement { .. })
        | (Restriction::ApplyReplacement { .. }, Query::Event { .. }) => false,
    }
}

/// CR 101.2 scoped by what caused the event — §2.6's Sigarda family.
///
/// `None` means "however caused" and matches everything, including the causeless
/// events (turn-based and state-based actions) that no [`SourceFilter`] matches.
fn cause_matches(
    by: Option<&SourceFilter>,
    cause: Option<PlayerId>,
    controller: PlayerId,
) -> bool {
    let Some(by) = by else {
        return true;
    };
    // A turn-based or state-based action has no controller, so it satisfies no
    // source filter. The right answer rather than an omission: Sigarda does not
    // stop CR 704.5's sacrifices, and there are none to stop.
    let Some(cause) = cause else {
        return false;
    };
    match by {
        // Relative to the *restriction's* controller, which is CR 109.5's "you"
        // for a static ability: Sigarda's "your opponents" is an opponent of
        // whoever currently controls Sigarda.
        SourceFilter::ControlledBy(player_ref) => match player_ref {
            PlayerRef::You | PlayerRef::Owner => cause == controller,
            PlayerRef::Opponent => cause != controller,
            PlayerRef::Player(pid) => cause == *pid,
        },
    }
}

/// Source 3 — the restrictions an object's *keywords* create.
///
/// Structurally `gather`'s `counter_replacements`: derived from something the
/// object **has** rather than from text it prints, so the CR's words are quoted
/// rather than paraphrased.
///
/// **Asked only of the event's own subject, and that is exact rather than a
/// shortcut.** Every keyword-derived restriction below is
/// `AffectedSet::SourceOnly`, so the only object whose synthesized restriction
/// can match an event about X is X itself. Sweeping the battlefield instead
/// would cost one full `compute_characteristics` walk per permanent per proposed
/// action, where this costs the one walk `is_blocked` already paid. The
/// `debug_assert` is what keeps the argument true as the list grows: a keyword
/// restriction that is *not* `SourceOnly` needs the sweep, and it must not be
/// able to arrive here quietly.
fn keyword_prohibits(game: &GameState, id: ObjectId, action: &GameAction) -> bool {
    let defs = keyword_restrictions(game, id);
    if defs.is_empty() {
        return false;
    }
    // Resolved after the early-out, not before it: `controller_of` is a full
    // `compute_characteristics` walk and almost no object has a keyword
    // restriction, so paying for it up front would have doubled what
    // `is_blocked` used to cost on every proposed action about an object.
    let controller = controller_or_owner(game, id).unwrap_or(0);
    for def in defs {
        debug_assert!(
            matches!(
                def.what,
                Restriction::Event { affected: AffectedSet::SourceOnly, .. }
            ),
            "a keyword-derived restriction that is not `AffectedSet::SourceOnly` \
             cannot be found by asking the event's subject about its own \
             keywords — it has to join the battlefield sweep, and until it does \
             it silently forbids nothing. See `keyword_prohibits`."
        );
        if matches(
            game,
            &def,
            id,
            controller,
            // A keyword's prohibition is unscoped by cause: CR 702.12b does not
            // care who is destroying the permanent.
            &Query::Event { action, cause: None },
        ) {
            return true;
        }
    }
    false
}

/// The whole of source 3 today.
///
/// One entry, and the census makes it the largest single consumer in the
/// document: **524 printed cards** have indestructible. Hexproof, shroud, menace
/// and intimidate are the next four (`codebase-state.md` item 15) and are Tier
/// 1a/1d — RS-2's and RS-3a's, because they forbid a *choice* rather than an
/// event, so none of them belongs in this function.
fn keyword_restrictions(game: &GameState, id: ObjectId) -> Vec<RestrictionDef> {
    use crate::oracle::characteristics::has_keyword;
    use crate::types::keywords::KeywordFlag;

    let mut out = Vec::new();

    // > 702.12b. A permanent with indestructible can't be destroyed. Such
    // > permanents aren't destroyed by lethal damage, and they ignore the
    // > state-based action that checks for lethal damage (see rule 704.5g).
    //
    // No `DestructionSourcePattern`: 702.12b names both of CR 701.8b's ways.
    if has_keyword(game, id, KeywordFlag::Indestructible) {
        out.push(RestrictionDef::new(Restriction::Event {
            pattern: EventPattern::Destroy { source: None },
            affected: AffectedSet::SourceOnly,
            by: None,
        }));
    }

    out
}
