//! The CR 101.2 predicate and the sweep that feeds it.
//!
//! `gather.rs`'s counterpart, and deliberately its mirror image: the same five
//! sources in the same order, the same battlefield walk over *effective* ability
//! lists, and the same two matchers — [`pattern_watches`] and [`set_affects`],
//! imported rather than reimplemented. What is missing is the `Rewrite` half, a
//! candidate list, and any notion of order; see [`is_prohibited`] for why the
//! last one is a property of CR 101.2 rather than a simplification.

use crate::engine::actions::GameAction;
use crate::engine::audit::{assert_shortcut_agrees, Sweep};
use crate::engine::layers::condition::{settled_holds, ConditionReferents};
use crate::engine::object_filter::TextReferents;
use crate::events::cause::EventCause;
use crate::engine::replacement::{pattern_watches, set_affects, subject_of, EntryFrame};
use crate::objects::card_data::AbilityType;
use crate::oracle::characteristics::{controller_or_owner, get_effective_abilities};
use crate::state::game_state::GameState;
use crate::types::effects::{AffectedSet, ObjectSet};
use crate::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
use crate::types::keywords::KeywordFlag;
use crate::types::replacement::EventPattern;
use crate::types::restriction::{
    ReplacementKindFilter, Restriction, RestrictionDef, RuleAction, SourceFilter,
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
        /// What caused it — CR 101.2 scoped by cause, which is
        /// [`SourceFilter`]'s whole job. The proposal's `ActionContext::cause`.
        cause: EventCause,
        /// CR 614.17d — the subject's characteristics as it would exist on the
        /// battlefield, when the caller already holds them: the pipeline's
        /// per-iteration frame for an entry, or the frame of an entering
        /// object a synthetic `AddCounters` is about, whose action names no
        /// `EnterMods`. `None` means derive one from the action, which is
        /// exact for every event but that synthetic one.
        lookahead: Option<&'a EntryFrame<'a>>,
    },
    /// CR 701.19c / 615.12 — may this replacement effect be applied?
    ApplyReplacement {
        kind: ReplacementKindFilter,
        subject: Option<ObjectOrPlayerId>,
    },
    /// CR 101.1 — does this rule's own action apply to `subject`?
    ApplyRule {
        rule: RuleAction,
        subject: ObjectOrPlayerId,
    },
}

/// A restriction that forbids what was asked, named: whose it is, and which.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Prohibition {
    /// The object the restriction is an ability or an effect of.
    pub source: ObjectId,
    pub by: ProhibitedBy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProhibitedBy {
    /// A keyword the source has, as indestructible is (CR 702.12b).
    Keyword(KeywordFlag),
    /// One of the source's static abilities, by its text.
    StaticAbility(&'static str),
    /// An effect a resolving spell or ability registered.
    RegisteredEffect,
}

impl ProhibitedBy {
    /// How the trace's `pipeline` record names it: which kind of restriction,
    /// and its words, which a registered effect leaves to its source.
    pub(crate) fn as_recorded(self) -> (&'static str, Option<&'static str>) {
        match self {
            ProhibitedBy::Keyword(keyword) => ("keyword", Some(crate::ui::display::keyword_name(keyword))),
            ProhibitedBy::StaticAbility(words) => ("static_ability", Some(words)),
            ProhibitedBy::RegisteredEffect => ("registered_effect", None),
        }
    }
}

/// CR 101.2 — is this prohibited right now?
///
/// The one reader of every restriction. Returns `true` when *any* source
/// forbids it: CR 101.2 has no tiebreak because it needs none — two
/// prohibitions agree, so this is a disjunction and never an ordering.
pub(crate) fn is_prohibited(game: &GameState, query: &Query) -> bool {
    prohibition(game, query).is_some()
}

/// CR 101.1 — whether `rule`'s own action applies to `subject`: whether no
/// card says it does not (`Restriction::RuleDoesNotApply`). The one ask of
/// axis 4, for every site where a rule acts.
pub(crate) fn rule_applies(game: &GameState, rule: RuleAction, subject: ObjectOrPlayerId) -> bool {
    !is_prohibited(game, &Query::ApplyRule { rule, subject })
}

/// [`is_prohibited`], naming the restriction it found first, in the order its
/// sources are swept. Which one is named is a reader's question, never a
/// rule's: any one forbids it (CR 101.2).
pub(crate) fn prohibition(game: &GameState, query: &Query) -> Option<Prohibition> {
    let found = prohibition_sweeping(game, query, Sweep::Gated);
    if game.audit_on() {
        let everything = game.unobserved(|| match *query {
            Query::Event { action, cause, lookahead } => {
                let fresh = lookahead.map(EntryFrame::uncomputed);
                prohibition_sweeping(game, &Query::Event { action, cause, lookahead: fresh.as_ref() }, Sweep::Everything)
            }
            Query::ApplyReplacement { .. } | Query::ApplyRule { .. } => prohibition_sweeping(game, query, Sweep::Everything),
        });
        let named = |found: Option<Prohibition>| found.map(|p| format!("{p:?}")).into_iter().collect();
        assert_shortcut_agrees("the restriction sweep's gate", || format!("{query:?}"), named(found), named(everything));
    }
    found
}

/// [`prohibition`]'s sweep, past its gate as the game runs it, or over every
/// permanent, as its audit does.
fn prohibition_sweeping(game: &GameState, query: &Query, sweep: Sweep) -> Option<Prohibition> {
    game.diagnostics.record_restriction_query();

    // --- Source 3: keyword abilities -------------------------------------
    //
    // Ahead of the gate, because `get_effective_abilities` is not how a keyword
    // is found and the gate would skip it (§3.5 commitment 2).
    if let Query::Event { action, cause, .. } = query
        && let Some(ObjectOrPlayerId::Object(id)) = subject_of(action)
        && let Some(keyword) = keyword_prohibits(game, id, action, *cause) {
        return Some(Prohibition { source: id, by: ProhibitedBy::Keyword(keyword) });
    }

    // The fast-path gate, exact, with `replacement_ability_sources`' rule: **a new
    // source of static restriction abilities needs a leg here, or it is dead on
    // every board the gate skips.** Printed (at ETB, an entry copy's included),
    // granted and copied (the summary's flags; CR 707.2a). Each over-approximates,
    // costing a walk, never an answer; Layer 3 is the route with no leg.
    let has_static_source = sweep == Sweep::Everything
        || !game.restriction_ability_sources.is_empty()
        || game.continuous_effects.summary().any_granted_restriction
        || game.continuous_effects.summary().any_copied_restriction;
    if !has_static_source && game.restrictions.is_empty() {
        return None;
    }

    // CR 614.17d's frame for an entry. Held here so the sweep and the registry
    // share one, and computed only if a `Filter` reaches `set_affects`.
    let derived;
    let frame: Option<&EntryFrame<'_>> = match query {
        Query::Event { lookahead: Some(frame), .. } => Some(*frame),
        Query::Event { action, lookahead: None, .. } => {
            derived = EntryFrame::new(game, action);
            Some(&derived)
        }
        Query::ApplyReplacement { .. } | Query::ApplyRule { .. } => None,
    };

    // --- Source 1: the battlefield sweep ---------------------------------
    if has_static_source {
        for id in game.battlefield_ids_ordered() {
            let controller = controller_or_owner(game, id).unwrap_or(0);
            for ability in get_effective_abilities(game, id).iter() {
                if ability.ability_type != AbilityType::Static {
                    continue;
                }
                // Through a CR 604.2 "as long as" wrapper too, asked at the
                // proposal against the settled board — the same leg, with the
                // same evaluator, that `replacement::gather` gives a
                // conditional replacement.
                let Some((condition, def)) = ability.effect.as_restriction() else { continue };
                if condition.is_some_and(|condition| !settled_holds(condition, game, id, None, ConditionReferents::NONE)) {
                    continue;
                }
                if matches(game, def, id, controller, query, frame) {
                    return Some(Prohibition { source: id, by: ProhibitedBy::StaticAbility(ability.rules_text.words) });
                }
            }
        }
    }

    // --- Source 4: the registry ------------------------------------------
    for row in game.restrictions.iter() {
        if matches(game, &row.def, row.source, row.controller, query, frame) {
            return Some(Prohibition { source: row.source, by: ProhibitedBy::RegisteredEffect });
        }
    }

    None
}

/// Does this restriction forbid what the query asks about?
///
/// The outer `match` is on a **pair**: the restriction's axis on the left, the
/// question's axis on the right. Only each same-axis pair can match, which
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
///                                   act: Sacrificed, object: None },
///            affected: objects(Filter { ByController(You) }),
///            by:       Some(ControlledBy(Opponent)),
///        }
/// source     = <Sigarda>          controller = P0   (CR 109.5: hers, now)
/// query      = Query::Event {
///                  action: ZoneChange { object: <Bear>, Battlefield ->
///                                       Graveyard, act: Sacrificed },
///                  cause:  Effect { <the edict>, P1, Spell },
///              }
/// ```
///
/// Three questions, all of which must say yes:
///
/// 1. **`pattern_watches`** — is this the *kind* of event Sigarda names? The
///    proposal is a battlefield→graveyard move whose act is `Sacrificed`, and the
///    pattern's three `Some(..)` fields all agree. **Yes.** (A *destruction*
///    would carry `act: Destroyed` and fail here, which is CR 701.21b: a
///    sacrifice is not a destruction.)
/// 2. **`set_affects`** — is the event's subject inside her affected set? The
///    subject is the Bear, the set is "permanents you control", and `you` is
///    resolved against `controller`, which is P0. **Yes.** (Had P1's own
///    creature been the subject, this would be where it failed.)
/// 3. **`cause_matches`** — was it caused by the right sort of source? `by`
///    says "a player who is not the restriction's controller"; `cause` is P1's
///    spell and `controller` is P0. **Yes.** (P0's *own* edict would name P0
///    and fail here — the field exists for exactly this.)
///
/// So the Bear is dropped from the candidate list. Sigarda is asked the same
/// three questions about herself and answers the same way, the list comes out
/// empty, and CR 101.3 ignores the instruction — no prompt.
///
/// # And the other axis
///
/// "Destroy target creature. It can't be regenerated" registers a
/// `Restriction::ApplyReplacement { kind: Regeneration, affected: objects(Fixed([creature])) }`.
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
    frame: Option<&EntryFrame<'_>>,
) -> bool {
    match (&def.restriction, query) {
        (
            Restriction::Event { pattern, affected, by },
            Query::Event { action, cause, .. },
        ) => {
            pattern_watches(game, pattern, action, cause, source, controller)
                // The test `ReplacementDef` is asked: `set_affects` asks the object half
                // or the player half by what the event is *about*, never both.
                && set_affects(game, affected, source, controller, subject_of(action), frame)
                && cause_matches(by.as_ref(), cause, &mut TextReferents::for_effect(game, controller, source))
        }

        (
            Restriction::ApplyReplacement { kind, affected },
            Query::ApplyReplacement { kind: asked, subject },
        ) => {
            kind == asked && set_affects(game, affected, source, controller, *subject, None)
        }

        (
            Restriction::RuleDoesNotApply { rule, affected },
            Query::ApplyRule { rule: asked, subject },
        ) => rule == asked && set_affects(game, affected, source, controller, Some(*subject), None),

        (Restriction::Event { .. }, Query::ApplyReplacement { .. } | Query::ApplyRule { .. })
        | (Restriction::ApplyReplacement { .. }, Query::Event { .. } | Query::ApplyRule { .. })
        | (Restriction::RuleDoesNotApply { .. }, Query::Event { .. } | Query::ApplyReplacement { .. }) => false,
    }
}

/// Which source caused the event — §2.6's Sigarda family. Not a rule of
/// CR 101.2's; see [`SourceFilter`].
///
/// `None` means "however caused" and matches everything, including the events
/// no [`SourceFilter`] matches (a cost, a turn-based or state-based action).
fn cause_matches(by: Option<&SourceFilter>, cause: &EventCause, referents: &mut TextReferents<'_, '_>) -> bool {
    by.is_none_or(|by| by.matches(referents.game, cause, referents))
}

/// Source 3 — the restrictions an object's *keywords* create.
///
/// Structurally `gather`'s `counter_replacements`: derived from something the
/// object **has** rather than from text it prints, so the CR's words are quoted
/// rather than paraphrased.
///
/// **Asked only of the event's own subject, and that is exact rather than a
/// shortcut.** Every keyword-derived restriction below is
/// `ObjectSet::SourceOnly`, so the only object whose synthesized restriction
/// can match an event about X is X itself. Sweeping the battlefield instead
/// would cost one full `compute_characteristics` walk per permanent per proposed
/// action, where this costs the one walk `is_blocked` already paid. The
/// `debug_assert` is what keeps the argument true as the list grows: a keyword
/// restriction that is *not* `SourceOnly` needs the sweep, and it must not be
/// able to arrive here quietly.
fn keyword_prohibits(game: &GameState, id: ObjectId, action: &GameAction, cause: EventCause) -> Option<KeywordFlag> {
    let defs = keyword_restrictions(game, id);
    if defs.is_empty() {
        return None;
    }
    // Resolved after the early-out, not before it: `controller_of` is a full
    // `compute_characteristics` walk and almost no object has a keyword
    // restriction, so paying for it up front would double the cost of every
    // proposed action about an object.
    let controller = controller_or_owner(game, id).unwrap_or(0);
    for (keyword, def) in defs {
        debug_assert!(
            matches!(
                def.restriction,
                Restriction::Event { affected: AffectedSet { objects: ObjectSet::SourceOnly, .. }, .. }
            ),
            "a keyword-derived restriction that is not `ObjectSet::SourceOnly` \
             cannot be found by asking the event's subject about its own \
             keywords — it has to join the battlefield sweep, and until it does \
             it silently forbids nothing. See `keyword_prohibits`."
        );
        if matches(
            game,
            &def,
            id,
            controller,
            // A keyword's prohibition is unscoped by cause (its `by` is `None`):
            // CR 702.12b does not care who is destroying the permanent.
            &Query::Event { action, cause, lookahead: None },
            // `SourceOnly`, per the assertion above, so no frame is ever read.
            None,
        ) {
            return Some(keyword);
        }
    }
    None
}

/// The whole of source 3 today.
///
/// One entry, and the census makes it the largest single consumer in the
/// document: **524 printed cards** have indestructible (2026-08-31). Hexproof, shroud, menace
/// and intimidate are the next four (`codebase-state.md` item 15) and are Tier
/// 1a/1d — `cant-be-targeted`'s and `combat-restrictions`'s, because they forbid a *choice* rather than an
/// event, so none of them belongs in this function.
fn keyword_restrictions(game: &GameState, id: ObjectId) -> Vec<(KeywordFlag, RestrictionDef)> {
    use crate::oracle::characteristics::has_keyword;

    let mut out = Vec::new();

    // > 702.12b. A permanent with indestructible can't be destroyed. Such
    // > permanents aren't destroyed by lethal damage, and they ignore the
    // > state-based action that checks for lethal damage (see rule 704.5g).
    //
    // No `DestructionSourcePattern`: 702.12b names both of CR 701.8b's ways.
    if has_keyword(game, id, KeywordFlag::Indestructible) {
        out.push((
            KeywordFlag::Indestructible,
            RestrictionDef::new(Restriction::Event {
                pattern: EventPattern::Destroy { source: None },
                affected: AffectedSet::objects(ObjectSet::SourceOnly),
                by: None,
            }),
        ));
    }

    out
}
