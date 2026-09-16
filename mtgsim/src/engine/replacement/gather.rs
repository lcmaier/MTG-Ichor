//! Finding the replacement effects that apply to a proposed event.
//!
//! `replacement-architecture.md` §3.3 lists five sources, and getting the list
//! wrong is the failure mode that shows up as a card silently doing nothing:
//!
//! 1. **Static abilities of permanents** — swept from `battlefield_ids_ordered`,
//!    read off each object's **effective** ability list. Not a registry scan.
//!    That is not a shortcut: it is what makes Humility and Blood Moon strip a
//!    replacement ability for free, and it asks CR 614.4's "must exist before
//!    the event" at the one instant that matters.
//! 2. **Static abilities functioning in other zones** (CR 113.6) — the same
//!    read, off a set of the objects whose printed ability functions where
//!    they are (`GameState::zone_replacement_ability_sources`) plus the zones
//!    a Layer 6 grant or a copy can reach, because a library is not a list
//!    the gather can afford to walk. Every ability is asked
//!    `zone_function::functions_in` of the zone its object is in, which is
//!    what keeps "if this would be put into a graveyard, exile it instead"
//!    on a card in hand from applying to its own discard. RF, 2026-09-16.
//! 3. **Continuous effects with a duration, from resolutions** — the registry.
//! 4. **Shields from resolutions** (CR 615.7/615.8, 701.19a) — the registry.
//! 5. **Counters** (CR 122.1c/d/h) — synthesized during the sweep, because they
//!    come from the *counter* and nothing on the card says so.
//!
//! Plus CR 614.15's self-replacement effects, which belong to the resolving
//! spell or ability rather than to any source above and would arrive through
//! `ActionContext::resolution`. `ResolutionContext` has no field to carry them
//! and gains one with the first card that needs one — §11 item 3, and the
//! reason `ReplacementClass::SelfReplacement` currently has a step and no
//! producer.

use std::collections::HashSet;

use crate::engine::actions::{ActionContext, GameAction};
use crate::engine::layers::compute_characteristics;
use crate::engine::layers::condition::settled_holds;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::zone_function::functions_in;
use crate::events::event::{CounterSubject, DamageTarget};
use crate::objects::card_data::AbilityType;
use crate::oracle::characteristics::controller_or_owner;
use crate::state::continuous_effects::puts_a_replacement_ability;
use crate::state::game_state::GameState;
use crate::types::effects::{
    ObjectSet, AmountExpr, CounterType, Effect, EffectRecipient, ObjectFilter, PlayerSet,
    Primitive, SelectionFilter, TargetCount,
};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::{
    EventPattern, GameActionTemplate, ReplacementDef, Rewrite,
};
use crate::types::zones::{Zone, ZoneChangeCause};

use crate::engine::restriction::{is_prohibited, Query};
use crate::types::restriction::ReplacementKindFilter;

use super::{EntryFrame, ReplacementInstance, ReplacementInstanceId};

/// Which of CR 122.1c's two effects a counter-derived instance is.
///
/// > 122.1c One or more shield counters on a permanent create a single
/// > replacement effect **and** a single prevention effect that protect the
/// > permanent.
///
/// Stun (122.1d) and finality (122.1h) create only the replacement.
///
/// **Why the split does not defeat the purpose of one counter:** it is what
/// gives the two halves two CR 614.5 identities, so removing a shield counter
/// to a destruction does not also spend the prevention against damage. The
/// argument is at [`ReplacementInstanceId::Counter`], which is where the key
/// that carries this variant is defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CounterEffectKind {
    Replacement,
    Prevention,
}

/// What a proposed event is *about* — CR 614.1's "whatever they're affecting"
/// and CR 616.1's "the affected object ... or the affected player".
///
/// Named for the *event*, not for the effect, because `ObjectSet` already
/// answers the other question — which objects an effect applies to — and the
/// two are asked one line apart in [`applies_to`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum EventSubject {
    Object(ObjectId),
    Player(PlayerId),
}

/// The object or player a proposed action is about.
pub(crate) fn subject_of(action: &GameAction) -> EventSubject {
    match action {
        GameAction::DealDamage { target, .. } => match target {
            DamageTarget::Object(id) => EventSubject::Object(*id),
            DamageTarget::Player(pid) => EventSubject::Player(*pid),
        },
        GameAction::DrawCards { player, .. } => EventSubject::Player(*player),
        GameAction::DrawCard { player, .. } => EventSubject::Player(*player),
        GameAction::GainLife { player, .. } => EventSubject::Player(*player),
        GameAction::LoseLife { player, .. } => EventSubject::Player(*player),
        GameAction::ZoneChange { object, .. } => EventSubject::Object(*object),
        GameAction::Untap { object } => EventSubject::Object(*object),
        GameAction::Tap { object } => EventSubject::Object(*object),
        GameAction::Destroy { object, .. } => EventSubject::Object(*object),
        GameAction::EnterBattlefield { object, .. } => EventSubject::Object(*object),
        // CR 614.16's "under your control": the tokens are created for a
        // player, and CR 616.1's chooser is that player.
        GameAction::CreateTokens { controller, .. } => EventSubject::Player(*controller),
        GameAction::CreateTokenIn { object, .. } => EventSubject::Object(*object),
        // CR 122.1's "on an object or player": CR 616.1's chooser is the
        // permanent's controller or the player getting them.
        GameAction::AddCounters { subject, .. } | GameAction::RemoveCounters { subject, .. } => {
            match subject {
                CounterSubject::Object(id) => EventSubject::Object(*id),
                CounterSubject::Player(pid) => EventSubject::Player(*pid),
            }
        }
        GameAction::Attach { attachment, .. } => EventSubject::Object(*attachment),
        // CR 614.10's three units are all about a *player* — "skip **your**
        // next turn", "**players** skip their upkeep steps" — so CR 616.1's
        // chooser is the player whose turn it is and Eon Hub on a four-player
        // table asks nobody.
        GameAction::BeginTurn { player, .. } => EventSubject::Player(*player),
        GameAction::BeginPhase { player, .. } => EventSubject::Player(*player),
        GameAction::BeginStep { player, .. } => EventSubject::Player(*player),
        // CR 701.22a's "look at the top N cards of **your** library": the
        // scrying player, who is CR 616.1's chooser for Eligeth.
        GameAction::Scry { player, .. } => EventSubject::Player(*player),
        // CR 616.1's "the affected player": the one who would lose or win.
        GameAction::PlayerLoses { player, .. } => EventSubject::Player(*player),
        GameAction::PlayerWins { player } => EventSubject::Player(*player),
        // CR 106.6a's mana enters a *player's* pool, and CR 616.1's chooser
        // is that player; a spell's production has no permanent to be about.
        GameAction::ProduceMana { player, .. } => EventSubject::Player(*player),
        // CR 701.24a — "a player shuffles their library".
        GameAction::ShuffleLibrary { player } => EventSubject::Player(*player),
    }
}

/// CR 616.1's chooser: "the affected object's controller (or its owner if it
/// has no controller) or the affected player".
///
/// A lookup, and therefore already N-player-safe — no `bool`, no "the other
/// player".
/// Takes the whole proposal rather than its [`EventSubject`], because **an
/// entry's chooser is not a property of the board**. A permanent that has not
/// entered yet has no controller, so `controller_or_owner` falls through to its
/// owner — the wrong player the moment someone casts a permanent spell they do
/// not own, or a token is created under an opponent's control. CR 110.2b's
/// answer rides on `GameAction::EnterBattlefield` instead, and CR 616.1b's
/// control-changing bucket is the same reading from the other side: the rules
/// expect an entering permanent to have a controller to ask, and it is the one
/// it is *about to* enter under.
///
/// That is why this is one function with an entry arm rather than a wrapper
/// around a subject-shaped one. A subject cannot answer for an entry, and both
/// callers — the CR 616.1 loop and `apnap_batch_order` — hold the action.
pub(crate) fn chooser_for(game: &GameState, action: &GameAction) -> Option<PlayerId> {
    match action {
        GameAction::EnterBattlefield { controller, .. } => Some(*controller),
        other => match subject_of(other) {
            EventSubject::Player(pid) => Some(pid),
            EventSubject::Object(id) => controller_or_owner(game, id),
        },
    }
}

/// Every replacement effect that applies to `action` right now.
///
/// Returned in a deterministic order — battlefield timestamp order for the
/// sweep, registration order for the registry — because CR 616.1's prompt
/// offers this list and a `DecisionProvider` picks from it by *index*.
///
/// `blocked` is CR 614.17c: "if an event can't happen, it can only be replaced
/// by a self-replacement effect". When set, everything outside
/// `ReplacementClass::SelfReplacement` is discarded — which today means the
/// list is empty, since nothing produces one yet.
///
/// `frame` is CR 614.12's look-ahead for this event's subject, built by the
/// caller once per pipeline iteration and computed only if a filter-scoped
/// `affected` asks (`EntryFrame`). For an event that is not an entry it is
/// empty and every read falls through to the finished board.
pub(crate) fn gather(
    game: &GameState,
    action: &GameAction,
    ctx: &ActionContext,
    blocked: bool,
    frame: &EntryFrame<'_>,
) -> Vec<ReplacementInstance> {
    game.counters.record_replacement_gather();

    // What *caused* this event, for [`ReplacementDef::by`]: the controller of the
    // resolving spell or ability (CR 608.2; CR 109.5's "you"), or `None` for a
    // turn-based or state-based action — the same expression the pipeline hands
    // `Query::Event::cause`, since a "can't" and a replacement print one predicate.
    let cause = ctx.resolution.map(|r| r.controller);

    // CR 614.17c's filter is applied at the door rather than at the end: with
    // no self-replacement producer, a blocked event has no candidates at all
    // and the whole sweep below is dead work.
    if blocked {
        return Vec::new();
    }

    // The board-level fast path, and not an optimization: `get_effective_abilities`
    // is a full layer walk, and an ungated sweep runs one per permanent per
    // proposed action — ~6,000 extra walks a game against the untap step alone
    // (2026-09-01) — while skipping the per-permanent gate below cost 10.3% of
    // game time (RC-2's A/B, `replacement-architecture.md` §9). Exact, not a
    // heuristic: a battlefield object has a static replacement ability only if it
    // printed one (`replacement_ability_sources`, at ETB) or a Layer 6 row granted
    // one (the registry summary); both over-approximate, which costs a walk and
    // never an answer.
    let subject = subject_of(action);
    let proposal = EventProposal { action, subject, cause, frame: Some(frame) };
    let mut candidates = Vec::new();

    // --- Game rules that behave as replacement effects (CR 903.9b) ---------
    // Ahead of the gate, which would skip it: neither a sweep nor a registry row,
    // one `HashMap` lookup, and exact — 903.9b applies only to the object the
    // event is already about.
    if let Some(instance) = commander_zone_replacement(game, action) {
        push_if_applicable(game, &mut candidates, instance, &proposal);
    }

    // --- Source 1a: the permanent that is entering (CR 614.12) -------------
    // The sweep walks `battlefield_ids_ordered`, and the entering permanent is
    // not on it — its entry is what this pipeline is deciding — so this is the
    // gate leg `CLAUDE.md` demands; without it every "enters tapped" is dead
    // text. Ahead of the fast-path gate for `commander_zone_replacement`'s
    // reason (exact, one walk). Gathered here and spliced in after the sweep:
    // CR 613.7's oldest-first puts the entering permanent last, it has no
    // timestamp until `place_on_battlefield`, and CR 613.7e re-timestamps an
    // attaching Aura, never its host; simultaneous entries (CR 613.7m) are
    // `codebase-state.md` item 4.
    let mut entering: Vec<ReplacementInstance> = Vec::new();
    if let GameAction::EnterBattlefield { object, controller, .. } = action {
        // Its abilities as it would exist on the battlefield — the frame, not a
        // plain walk: the card is still in its source zone while the entry is
        // decided, and CR 614.12 clause (3) is what lets Humility strip an entering
        // "enters with" before it applies. Computed once per iteration, shared with
        // `set_affects`.
        // CR 113.6h: asked as on the battlefield, which is the zone the frame
        // computes it in.
        if let Some(chars) = frame.frame_of(*object) {
            let asked = AskedForReplacements {
                id: *object,
                controller: *controller,
                chars,
                zone: Zone::Battlefield,
                scope: SelfScope::EnteringSelf,
            };
            push_static_ability_replacements(game, &mut entering, &asked, &proposal);
        }
    }

    // "Unattributed": a static replacement ability an object has without
    // having printed it — a Layer 6 grant or a copy row put it there, so
    // neither printed set can name the object. The summary says where such an
    // object can be: a `Filter` row names zones, a named row names objects,
    // and each sweep reads the shape it is given.
    let summary = game.continuous_effects.summary();
    let unattributed_zones = summary.unattributed_replacement_zones;
    let any_named_unattributed = summary.any_named_unattributed_replacement;
    let has_static_source = !game.replacement_ability_sources.is_empty()
        || !game.zone_replacement_ability_sources.is_empty()
        || !unattributed_zones.is_empty()
        || any_named_unattributed;
    if !has_static_source
        && game.replacement_effects.is_empty()
        && !any_replacement_counter(game)
    {
        candidates.extend(entering);
        return candidates;
    }

    // --- Sources 1 and 5: the battlefield sweep ----------------------------
    // The fast path per *permanent*: `has_static_source` decides whether the
    // sweep runs, this decides which permanents are worth a walk — the same
    // predicate one object at a time, exact with the same over-approximations.
    // No under-approximation: an unattributed ability is on the effective list
    // and in neither ETB set, so a row reaching the battlefield by zone or by
    // name opens every permanent to a walk — registry-wide, not per object,
    // since narrowing it would resolve a filter per permanent per check.
    let any_unattributed = unattributed_zones.contains(Zone::Battlefield) || any_named_unattributed;
    for id in game.battlefield_ids_ordered() {
        if any_unattributed || game.replacement_ability_sources.contains(&id) {
            let controller = controller_or_owner(game, id).unwrap_or(0);
            if let Some(chars) = compute_characteristics(game, id) {
                let asked = AskedForReplacements {
                    id,
                    controller,
                    chars: &chars,
                    zone: Zone::Battlefield,
                    scope: SelfScope::Existing,
                };
                push_static_ability_replacements(game, &mut candidates, &asked, &proposal);
            }
        }

        for (counter, kind, def) in counter_replacements(game, id) {
            let controller = controller_or_owner(game, id).unwrap_or(0);
            push_if_applicable(
                game,
                &mut candidates,
                ReplacementInstance {
                    id: ReplacementInstanceId::Counter(id, counter, kind),
                    source: id,
                    controller,
                    def,
                },
                &proposal,
            );
        }
    }

    // --- Source 2: static abilities functioning off the battlefield --------
    // (CR 113.6.) The same read as source 1 over a candidate list that is
    // never a zone — a library is ~60 objects a seat and a gather runs ~2,300
    // times a game. Three ways onto the list, one per gate leg:
    //
    // - printed: `zone_replacement_ability_sources`, kept by the registration
    //   doors, holds each object's printed defs, and the frame is read only
    //   when one of them could apply to *this* proposal (`printed_could_apply`)
    //   — so a Colossus in a library costs a frame on the events that would
    //   put it into a graveyard and a def check on every other;
    // - named: a grant or copy row over `SourceOnly` or `Fixed` names the
    //   objects it reaches, and they are read by name wherever they are;
    // - zoned: a grant or copy row over a `Filter` names zones, and those are
    //   walked whole — the one walk of a zone this leg makes, only while such
    //   a row exists, and no registered card makes one.
    //
    // In CR 613.7d timestamp order, the battlefield's own key, so the list
    // CR 616.1 offers is one order rather than two. The entering object is
    // source 1a's: it is read off CR 614.12's frame with that rule's narrower
    // scope, and reading it again here off its source zone would offer its
    // filter-scoped rows to its own entry, which 614.12's parenthesis forbids.
    let mut elsewhere: Vec<(u64, ObjectId)> = Vec::new();
    for (&id, printed) in game.zone_replacement_ability_sources.iter() {
        if printed.iter().any(|def| printed_could_apply(game, id, def, &proposal)) {
            elsewhere.push((game.object_timestamp(id), id));
        }
    }
    let zone_sweep = unattributed_zones.beyond_battlefield();
    if any_named_unattributed || !zone_sweep.is_empty() {
        let mut named: Vec<ObjectId> = Vec::new();
        if any_named_unattributed {
            for row in game.continuous_effects.iter().filter(|r| puts_a_replacement_ability(r)) {
                match &row.affected_objects {
                    ObjectSet::SourceOnly => named.push(row.source),
                    ObjectSet::Fixed(ids) => named.extend(ids.iter().copied()),
                    // A host is a permanent (CR 301.5, 303.4): the battlefield sweep's.
                    ObjectSet::Host | ObjectSet::Filter { .. } => {}
                }
            }
        }
        for zone in zone_sweep.iter() {
            named.extend(game.zone_ids_ordered(zone));
        }
        let mut seen: HashSet<ObjectId> = elsewhere.iter().map(|&(_, id)| id).collect();
        for id in named {
            let off_battlefield =
                matches!(game.objects.get(&id), Some(obj) if obj.zone != Zone::Battlefield);
            if off_battlefield && seen.insert(id) {
                elsewhere.push((game.object_timestamp(id), id));
            }
        }
    }
    // Every arrival is stamped from one counter (CR 613.7d), so the keys are
    // distinct and the order is process-independent; the map's own order
    // never reaches CR 616.1's list. A tie would be that order leaking.
    elsewhere.sort_unstable_by_key(|&(timestamp, _)| timestamp);
    debug_assert!(
        elsewhere.windows(2).all(|pair| pair[0].0 < pair[1].0),
        "two zone-leg candidates share a CR 613.7d timestamp"
    );
    for (_, id) in elsewhere {
        if frame.is_entering(id) {
            continue;
        }
        let Some(obj) = game.objects.get(&id) else { continue };
        let Some(chars) = compute_characteristics(game, id) else { continue };
        let asked = AskedForReplacements {
            id,
            // CR 108.4 — no controller off the battlefield, so its owner.
            controller: controller_or_owner(game, id).unwrap_or(obj.owner),
            chars: &chars,
            zone: obj.zone,
            scope: SelfScope::Existing,
        };
        push_static_ability_replacements(game, &mut candidates, &asked, &proposal);
    }

    candidates.extend(entering);

    // --- Sources 3 and 4: the registry -------------------------------------
    for row in game.replacement_effects.iter() {
        push_if_applicable(
            game,
            &mut candidates,
            ReplacementInstance {
                id: ReplacementInstanceId::Registered(row.id),
                source: row.source,
                controller: row.controller,
                def: row.def.clone(),
            },
            &proposal,
        );
    }

    candidates
}

/// The proposed event as every leg asks about it — one value carried through
/// the sweeps rather than four parameters, so no leg can hand a candidate a
/// different event from its neighbor's.
struct EventProposal<'a, 'g> {
    action: &'a GameAction,
    subject: EventSubject,
    /// Who caused it, for [`ReplacementDef::by`].
    cause: Option<PlayerId>,
    /// CR 614.12's look-ahead for the subject, when the caller holds one; the
    /// CR 616.1f re-ask of a later batch member does not.
    frame: Option<&'a EntryFrame<'g>>,
}

/// An object a sweep is asking for its replacement abilities.
struct AskedForReplacements<'a> {
    id: ObjectId,
    controller: PlayerId,
    /// Its **effective** frame.
    chars: &'a EffectiveCharacteristics,
    /// The zone it is asked *as* in (CR 113.6).
    zone: Zone,
    scope: SelfScope,
}

/// A gate on the frame read, never an answer: could the def `id` *printed*
/// apply to this proposal at all?
///
/// The engine's answers come from effective characteristics, always
/// (`CLAUDE.md`'s layer-system invariant), and this function decides
/// nothing — it decides whether computing them is worth it. Computing a
/// frame is a layer walk, and the zone leg would otherwise do one per gather
/// for every Colossus in a library, on damage events, taps and draws its
/// clause cannot touch. So the leg first asks the cheaper question of the
/// printed def, and only a "yes" reads the frame, where the effective def
/// decides.
///
/// Why a "no" here is safe: an object's effective replacement defs are its
/// printed ones or fewer. Layer 6 can strip one (Hollow Hands) or grant one,
/// and a granted or copied def reaches this leg by name or by zone through
/// the other two legs, never through this map; nothing rewrites a printed def
/// in place — Layer 3, text, is the route every gate leg leaves open. So a
/// printed def that does not apply has no effective def that could, and
/// skipping the frame changes no answer. What the printed def cannot say is
/// whether its "as long as" clause holds, which the frame read asks. Asked
/// through [`def_applies`], the same function the effective def is asked
/// through, so the pairing of patterns and events lives once.
fn printed_could_apply(
    game: &GameState,
    id: ObjectId,
    def: &ReplacementDef,
    p: &EventProposal<'_, '_>,
) -> bool {
    let controller = controller_or_owner(game, id).unwrap_or(0);
    def_applies(game, def, id, controller, p)
}

/// Whether the object being asked for replacement abilities is already on the
/// battlefield, or is the one entering.
///
/// **CR 614.12's parenthesis, and it is a membership rule rather than a frame
/// question**:
///
/// > 614.12. … Such effects may come from the permanent itself if they affect
/// > only that permanent (as opposed to a general subset of permanents that
/// > includes it) …
///
/// So an entering permanent contributes its `ObjectSet::SourceOnly`
/// replacements ("this land enters tapped") and **not** its filter-scoped ones.
/// Orb of Dreams says "Permanents enter tapped" and enters untapped; without
/// this the entering Orb finds its own row through `set_affects`, which matches
/// a `Filter` against any object in any zone, and taps itself.
///
/// Nothing already in a zone is excluded from anything — an object there is
/// one of the "existing" effects clause (3) means, on the battlefield or off
/// it — so the two sweeps pass [`SelfScope::Existing`] and the check costs
/// them one integer comparison they always win.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelfScope {
    /// The battlefield sweep and the zone leg. Every replacement ability the
    /// object has counts.
    Existing,
    /// Source 1a. Only `ObjectSet::SourceOnly` counts (CR 614.12).
    EnteringSelf,
}

/// Every static replacement ability on `id` that functions in `zone`, as
/// candidate instances.
///
/// **The sweeps' loop body, lifted so all three callers share it** — nothing
/// more. It computes nothing: the caller decides which object to ask and
/// hands over that object's frame, read off the board for the two sweeps and
/// off CR 614.12's frame for the entering permanent.
///
/// `chars` is the **effective** frame either way, which is source 1's whole
/// point: Humility and Blood Moon strip a replacement ability for free, and
/// CR 614.4's "must exist before the event" is asked at the one instant that
/// matters. Its types are what CR 113.6 asks (`zone_function`).
///
/// `zone` is where the object is asked *as*: its own zone for the sweeps, the
/// battlefield for the entering permanent (CR 113.6h — "functions as that
/// object is entering the battlefield"). AskedForReplacements of every ability, on the
/// battlefield too, so the rule has one home: a "from your graveyard" clause
/// on a permanent is skipped here for the same reason it is skipped in a
/// library.
///
/// `controller` is a parameter because the callers know it differently: on
/// the battlefield it is `controller_or_owner`, off it CR 108.4's owner, and
/// for an entering permanent CR 110.2b's default off the proposal — the same
/// reason `chooser_for` takes the action.
///
/// `scope` is CR 614.12's parenthesis, and it is the only thing the callers
/// ask differently about the *effects* rather than about the object — see
/// [`SelfScope`].
fn push_static_ability_replacements(
    game: &GameState,
    out: &mut Vec<ReplacementInstance>,
    asked: &AskedForReplacements<'_>,
    p: &EventProposal<'_, '_>,
) {
    let AskedForReplacements { id, controller, chars, zone, scope } = *asked;
    for ability in &chars.abilities {
        if ability.ability_type != AbilityType::Static {
            continue;
        }
        if !functions_in(ability, &chars.types, zone) {
            continue;
        }
        // CR 604.2 through the "as long as" wrapper: the effect exists while its
        // condition holds, and CR 614.4 asks that *before the event* — so the
        // condition is asked here against the settled board, with the evaluator
        // the layer pass and CR 613.11 use. Laboratory Maniac is a candidate exactly
        // when the draw would fail, never a prompt beside Thought Reflection.
        let def = match &ability.effect {
            Effect::Replacement(def) => def,
            Effect::Conditional(condition, inner) => {
                let Effect::Replacement(def) = &**inner else { continue };
                if !settled_holds(condition, game, id) {
                    continue;
                }
                def
            }
            _ => continue,
        };
        if scope == SelfScope::EnteringSelf && !matches!(def.affected_objects, ObjectSet::SourceOnly) {
            continue;
        }
        push_if_applicable(
            game,
            out,
            ReplacementInstance {
                id: ReplacementInstanceId::StaticAbility(id, ability.id),
                source: id,
                controller,
                def: (**def).clone(),
            },
            p,
        );
    }
}

fn push_if_applicable(
    game: &GameState,
    out: &mut Vec<ReplacementInstance>,
    instance: ReplacementInstance,
    p: &EventProposal<'_, '_>,
) {
    // CR 701.19c — "can't be regenerated" causes shields "to not be applied":
    // withheld at the door, not spent, so the shield stays for a later
    // destruction. The one place an effect is applied to an event, which is why
    // `Restriction::ApplyReplacement` is closed at one arm
    // (`cant-effects-architecture.md` §3.3); recognizing a shield is what
    // `is_regeneration`'s authored bit is for.
    if instance.def.is_regeneration
        && is_prohibited(
            game,
            &Query::ApplyReplacement {
                kind: ReplacementKindFilter::Regeneration,
                subject: p.subject,
            },
        )
    {
        return;
    }
    if def_applies(game, &instance.def, instance.source, instance.controller, p) {
        out.push(instance);
    }
}

/// Does this effect apply to this event?
///
/// Three halves of one CR 614.1 question, not three unrelated checks: an
/// effect applies when it *watches* this kind of event, **and** *affects* the
/// object the event is about, **and** admits what *caused* the event
/// ([`ReplacementDef::by`]).
///
/// `pub(super)` for one caller beyond the sweep: the CR 616.1f loop re-asks it
/// of an effect it has just applied, which is how an exempt effect's
/// termination is checked rather than assumed.
pub(super) fn applies_to(
    game: &GameState,
    instance: &ReplacementInstance,
    action: &GameAction,
    subject: EventSubject,
    cause: Option<PlayerId>,
    frame: Option<&EntryFrame<'_>>,
) -> bool {
    let p = EventProposal { action, subject, cause, frame };
    def_applies(game, &instance.def, instance.source, instance.controller, &p)
}

/// [`applies_to`] of a def that has no instance yet — the zone leg's
/// `printed_could_apply` asks it of a printed def before deciding whether the
/// frame is worth reading, and every instance asks it through the same
/// function, so a printed def and its effective twin are judged alike.
fn def_applies(
    game: &GameState,
    def: &ReplacementDef,
    source: ObjectId,
    controller: PlayerId,
    p: &EventProposal<'_, '_>,
) -> bool {
    // The cause, asked of the *effect* (`def.by`), not the pattern. `None` is
    // "however caused"; a `Some` against a turn-based or state-based action's
    // `None` is `false`, so Nephalia Academy leaves the cleanup discard alone.
    def.by.as_ref().is_none_or(|by| by.matches(p.cause, controller))
        && pattern_watches(game, &def.pattern, p.action, controller)
        && set_affects(
            game,
            &def.affected_objects,
            &def.affected_players,
            source,
            controller,
            p.subject,
            p.frame,
        )
}

/// Is the event's subject inside this effect's two affected sets — CR 614.1's
/// "whatever they're affecting"?
///
/// **Two sets, unioned, because the CR names two kinds of subject.** An event
/// about an object asks `ObjectSet`; an event about a player asks
/// [`PlayerSet`]. Furnace of Rath's "a permanent or player" is `Filter { All }`
/// plus `Everyone` and is one effect either way — which is why this is one
/// function with two parameters rather than two functions
/// (`replacement-architecture.md` §9, RD decision 0).
///
/// Takes the sets and their owner's two ids rather than a
/// `ReplacementInstance`, because a "can't" asks the identical question of
/// identical sets and has no instance to offer
/// (`cant-effects-architecture.md` §3.1: a restriction is discovered exactly
/// the way a replacement effect is, and differs only in what it is asked at).
///
/// `frame` is where CR 614.12 and 614.17d land on the object side: a `Filter`
/// about an entering permanent is matched against the permanent *as it would
/// exist on the battlefield*, not against the card. `SourceOnly` and `Fixed`
/// match by id and never look. A player subject reaches no frame at all — a
/// player is not an object and no layer computes one.
pub(crate) fn set_affects(
    game: &GameState,
    affected: &ObjectSet,
    affected_players: &PlayerSet,
    source: ObjectId,
    controller: PlayerId,
    subject: EventSubject,
    frame: Option<&EntryFrame<'_>>,
) -> bool {
    let id = match subject {
        EventSubject::Object(id) => id,
        // CR 109.5 resolves "you" against the effect's *current* controller,
        // which is exactly what `controller` is here — the sweep reads it off
        // the board on every gather rather than snapshotting it at ETB.
        EventSubject::Player(pid) => return affected_players.contains(controller, pid),
    };
    match affected {
        ObjectSet::SourceOnly => source == id,
        ObjectSet::Fixed(ids) => ids.contains(&id),
        // CR 303.4m, by id like the two above: an entering permanent is
        // attached to by nothing, so the frame has nothing to say.
        ObjectSet::Host => {
            game.battlefield.get(&source).and_then(|e| e.attached_to) == Some(id)
        }
        // On behalf of the effect, not a selection: `ObjectFilter::EachOther` is
        // "other than the effect's `source`", which a selection has no source for —
        // Palisade Giant's "other permanents you control" (`codebase-state.md` item 103).
        ObjectSet::Filter { filter, zones } => {
            // The zone half, ahead of the filter, as the layer walk's
            // `in_zones_or_entering` asks it: the entering object counts as on the
            // battlefield — CR 614.12 asks what it *would be* there, and its source
            // zone is not the question — and anything else must be where the row
            // reaches. Rest in Peace's "from anywhere" is `ZoneSet::ALL`; "if a
            // creature would be put into a graveyard" is the battlefield, and a
            // milled creature *card* is not a creature (CR 109.2).
            let in_zone = match frame {
                Some(f) if f.is_entering(id) => zones.contains(Zone::Battlefield),
                _ => matches!(game.objects.get(&id), Some(obj) if zones.contains(obj.zone)),
            };
            if !in_zone {
                return false;
            }
            game.object_matches_filter_of_source(
                id,
                filter,
                controller,
                source,
                frame.and_then(|f| f.frame_of(id)),
            )
            .unwrap_or(false)
        }
    }
}

/// Does this pattern watch for the proposed event's kind (CR 614.1)?
///
/// `pub(crate)` for the same reason [`set_affects`] is: `Restriction::Event`
/// reuses `EventPattern` verbatim, so a "can't be destroyed" and an "if it
/// would be destroyed, instead …" ask this one function the same question.
pub(crate) fn pattern_watches(
    game: &GameState,
    pattern: &EventPattern,
    action: &GameAction,
    you: PlayerId,
) -> bool {
    match (pattern, action) {
        // CR 609.7's source predicate and CR 510.2's combat flag, asked **now**
        // rather than captured: 609.7b's recheck is free because the gather already
        // happens at the proposal, and a source that stopped matching yields no
        // candidate, so `consume_use` never runs — 609.7b's "the shield isn't used
        // up" for free. 609.7c is the same read for a source off the battlefield.
        // The `unwrap_or(true)`s are the fields' meaning ("this effect does not
        // ask"); the `unwrap_or(false)` swallows `object_matches_filter`'s three
        // authoring-error `Err`s, unreached in 600 fuzz games (2026-09-09) and
        // shared with `set_affects` — `codebase-state.md` item 103.
        (
            EventPattern::DealDamage { source, combat },
            GameAction::DealDamage { source: dealt_by, is_combat, .. },
        ) => {
            combat.map(|c| c == *is_combat).unwrap_or(true)
                && source
                    .as_ref()
                    .map(|p| {
                        p.object.map(|chosen| chosen == *dealt_by).unwrap_or(true)
                            && p.filter
                                .as_ref()
                                .map(|f| {
                                    game.object_matches_filter(*dealt_by, f, you)
                                        .unwrap_or(false)
                                })
                                .unwrap_or(true)
                    })
                    .unwrap_or(true)
        }

        (
            EventPattern::ZoneChange { from, to, cause, object },
            GameAction::ZoneChange {
                object: moving,
                from: actual_from,
                to: actual_to,
                cause: actual_cause,
            },
        ) => {
            from.map(|z| z == *actual_from).unwrap_or(true)
                && to.map(|z| z == *actual_to).unwrap_or(true)
                && cause.map(|c| c == *actual_cause).unwrap_or(true)
                && object
                    .as_ref()
                    .map(|f| game.object_matches_filter(*moving, f, you).unwrap_or(false))
                    .unwrap_or(true)
        }

        // Entering is the zone change onto the battlefield (CR 614.1c), so a
        // zone-change pattern admitting that destination watches an entry too
        // (Worms of the Earth, Grafdigger's Cage), `from` and `cause` compared as
        // above and `object` read in its source zone (the Cage's ruling). A token's
        // entry has neither `from` nor `cause`.
        (
            EventPattern::ZoneChange { from, to, cause, object },
            GameAction::EnterBattlefield {
                object: moving,
                from: actual_from,
                cause: actual_cause,
                ..
            },
        ) => {
            to.map(|z| z == Zone::Battlefield).unwrap_or(true)
                && from.map(|z| Some(z) == *actual_from).unwrap_or(true)
                && cause.map(|c| Some(c) == *actual_cause).unwrap_or(true)
                && object
                    .as_ref()
                    .map(|f| game.object_matches_filter(*moving, f, you).unwrap_or(false))
                    .unwrap_or(true)
        }

        (EventPattern::Untap, GameAction::Untap { .. }) => true,
        (EventPattern::Tap, GameAction::Tap { .. }) => true,

        // CR 121.2a's instruction: Alms Collector's "two or more" is `at_least:
        // Some(2)`, `None` asks nothing. The two draw arms do not cross-match — the
        // printed ruling counts "how many times the word 'draw' is used" — so
        // Divination and a pair of cantrips stay distinct.
        (EventPattern::DrawCards { at_least }, GameAction::DrawCards { n, .. }) => {
            at_least.map(|k| *n >= k).unwrap_or(true)
        }

        // CR 121.1's individual draw. `cause` is the field the ten "except the
        // first one you draw in each of your draw steps" cards read, and it is
        // this draw's own rather than its instruction's — see [`DrawCause`].
        (EventPattern::DrawCard { cause }, GameAction::DrawCard { cause: actual, .. }) => {
            cause.map(|c| c == *actual).unwrap_or(true)
        }

        // CR 119.3 / 119.10's gain and CR 120.3a's loss. Which *player* each is
        // around is `set_affects`'s question one function below; these ask only
        // about the event, and a gain has nothing to ask.
        (EventPattern::GainLife, GameAction::GainLife { .. }) => true,
        (EventPattern::LoseLife { cause }, GameAction::LoseLife { cause: actual, .. }) => {
            cause.map(|c| c.matches(*actual)).unwrap_or(true)
        }

        // CR 601's fact off the entry's cause: `Resolved` is a permanent spell
        // that was cast, everything else was not.
        (
            EventPattern::EnterBattlefield { cast },
            GameAction::EnterBattlefield { cause, .. },
        ) => cast
            .map(|required| (*cause == Some(ZoneChangeCause::Resolved)) == required)
            .unwrap_or(true),

        (EventPattern::Destroy { source }, GameAction::Destroy { source: actual, .. }) => {
            source.map(|p| p.matches(*actual)).unwrap_or(true)
        }

        // CR 614.16's "one or more" is the one count read (as `CreateTokens` reads
        // "one or more tokens"); `by` is Vorinclex's "if *you* would put". Which
        // permanent or player the effect is around is `set_affects`'s question.
        (
            EventPattern::AddCounters { counter, by },
            GameAction::AddCounters { counter: actual, n, by: putter, .. },
        ) => {
            *n >= 1
                && counter.map(|c| c == *actual).unwrap_or(true)
                && by.as_ref().map(|set| set.contains(you, *putter)).unwrap_or(true)
        }
        (
            EventPattern::RemoveCounters { counter },
            GameAction::RemoveCounters { counter: actual, .. },
        ) => counter.map(|c| c == *actual).unwrap_or(true),

        // CR 122.6's second door: counters a permanent enters with are "put on" it,
        // so a counter pattern watches the entry — "one or more" of each row, and
        // the row's putter (CR 122.6a's named player, else the controller CR 616.1b
        // settled). Nothing is removed as a permanent enters.
        (
            EventPattern::AddCounters { counter, by },
            GameAction::EnterBattlefield { mods, controller, .. },
        ) => mods.counters.iter().any(|row| {
            row.n >= 1
                && counter.map(|c| c == row.counter).unwrap_or(true)
                && by.as_ref().map(|set| set.contains(you, row.putter(*controller))).unwrap_or(true)
        }),

        // CR 614.1b's skips. Which *player* the effect is around is
        // `set_affects`'s question, one function below — these ask only which
        // unit the effect names, and `None` asks nothing.
        (EventPattern::BeginTurn, GameAction::BeginTurn { .. }) => true,
        (EventPattern::BeginPhase { phase }, GameAction::BeginPhase { phase: actual, .. }) => {
            phase.map(|p| p == *actual).unwrap_or(true)
        }
        (EventPattern::BeginStep { step }, GameAction::BeginStep { step: actual, .. }) => {
            step.map(|s| s == *actual).unwrap_or(true)
        }

        // CR 701.22's scry, no field: both printed "would scry" clauses say "a
        // number of cards", and the one count the rules single out — 0 — is
        // `never_happens`'s. Which player is `set_affects`'s question.
        (EventPattern::Scry, GameAction::Scry { .. }) => true,

        // CR 104's two ends. Which *player* is `set_affects`'s question; the
        // loss's reason is asked by nothing printed (RE decision 5).
        (EventPattern::PlayerLoses, GameAction::PlayerLoses { .. }) => true,
        (EventPattern::PlayerWins, GameAction::PlayerWins { .. }) => true,

        // CR 614.16's "one or more tokens" — the rule's own phrase is the one
        // count this arm reads, and no multiplier the pipeline admits crosses
        // it — of the kind the pattern names, asked of each def. Which
        // *player* the effect is around is `set_affects`'s question.
        (EventPattern::CreateTokens { kind }, GameAction::CreateTokens { defs, .. }) => {
            defs.iter().any(|d| kind.as_ref().is_none_or(|k| k.matches(d)))
        }

        // CR 106.12b's two constraints. The event knows both facts; the pattern's
        // `Option`s let an effect decline to ask, and `None` is satisfied by every
        // production (Mana Reflection asks `Some(true)` of tapping and nothing of the
        // permanent; Deep Water asks both). The inner `unwrap_or(false)` swallows
        // `object_matches_filter`'s three authoring-error `Err`s, as `DealDamage`'s
        // arm does (`codebase-state.md` item 103).
        (
            EventPattern::ProduceMana { tapped_for_mana, source: filter },
            GameAction::ProduceMana { tapped_for_mana: actual, source: producer, .. },
        ) => {
            tapped_for_mana.map(|t| t == *actual).unwrap_or(true)
                && filter
                    .as_ref()
                    .map(|f| game.object_matches_filter(*producer, f, you).unwrap_or(false))
                    .unwrap_or(true)
        }

        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Source 5 — counters (CR 122.1c/d/h)
// ---------------------------------------------------------------------------

/// The three counter kinds that generate a replacement effect.
///
/// **Three is the whole of CR 122.1, audited rather than assumed** (RB's
/// review). Of that rule's nine kinds only 122.1c (shield), 122.1d (stun) and
/// 122.1h (finality) create a replacement effect: 122.1a is Layer 7c, 122.1e/f/g
/// are SBA inputs, 122.1i is a trigger, and 122.1b's fifteen keyword counters
/// grant a keyword. **None of those fifteen is CR 614-shaped**, and the two that
/// come closest are the two the rules deliberately put elsewhere — indestructible
/// is a "can't" (CR 702.12b, so `is_prohibited` rather than a `ReplacementDef`) and
/// lifelink is a further result of the damage event (CR 120.3f), not a
/// replacement of it. The rest are evasion, targeting, blocking,
/// combat-damage-step, damage-assignment or turn-based rules; vigilance's
/// "attacking doesn't cause it to tap" (702.20b) modifies the CR 508.1f
/// turn-based action and proposes no event to replace.
const REPLACEMENT_COUNTERS: [CounterType; 3] =
    [CounterType::Shield, CounterType::Stun, CounterType::Finality];

/// Is any permanent carrying a counter that generates a replacement effect?
///
/// Part of `gather`'s fast path, and *computed* rather than cached: counters
/// have three chokepoints — `GameState::add_counters`, `perform_action`'s
/// `RemoveCounters` arm, and CR 704.5q's annihilation, which writes
/// `PermanentState` directly (`sba.rs`, `codebase-state.md` Deferred
/// Migrations item 6) — so a set maintained at two of them drifts, and drift
/// reads as a card that silently does nothing. The scan skips immediately on
/// the empty-counters case, which is almost every permanent.
fn any_replacement_counter(game: &GameState) -> bool {
    game.battlefield.values().any(|entry| {
        !entry.counters.is_empty()
            && REPLACEMENT_COUNTERS.iter().any(|c| entry.counters.contains_key(c))
    })
}

/// The replacement effects the counters on `id` create.
///
/// **Their text is the CR's, verbatim.** Nothing on the card says any of this
/// (CR 122.1's counters are the source, not an ability), so the rule is quoted
/// on each one rather than paraphrased.
fn counter_replacements(
    game: &GameState,
    id: ObjectId,
) -> Vec<(CounterType, CounterEffectKind, ReplacementDef)> {
    let Some(entry) = game.battlefield.get(&id) else {
        return Vec::new();
    };
    if entry.counters.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();

    if entry.counter_count(CounterType::Shield) > 0 {
        // > 122.1c … "If this permanent would be destroyed as the result of an
        // > effect, instead remove a shield counter from it"
        // "As the result of an effect" is CR 701.8b way 1 only: lethal damage is
        // stopped by the prevention half below, before 704.5g asks.
        out.push((
            CounterType::Shield,
            CounterEffectKind::Replacement,
            ReplacementDef::new(
                EventPattern::Destroy {
                    source: Some(crate::types::replacement::DestructionSourcePattern::Effect),
                },
                ObjectSet::SourceOnly,
                Rewrite::Instead(GameActionTemplate::RemoveCountersFromAffected {
                    counter: CounterType::Shield,
                    n: 1,
                }),
            ),
        ));

        // > 122.1c … "If damage would be dealt to this permanent, prevent that
        // > damage and remove a shield counter from it"
        // `Prevent` plus a rider, not an `Instead`: CR 615.13 lets triggers fire on
        // damage *being prevented*.
        out.push((
            CounterType::Shield,
            CounterEffectKind::Prevention,
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                ObjectSet::SourceOnly,
                Rewrite::Prevent,
            )
            .with_then(remove_one_counter(CounterType::Shield)),
        ));
    }

    if entry.counter_count(CounterType::Stun) > 0 {
        // > 122.1d ... "If a permanent with a stun counter on it would become
        // > untapped, instead remove a stun counter from it."
        out.push((
            CounterType::Stun,
            CounterEffectKind::Replacement,
            ReplacementDef::new(
                EventPattern::Untap,
                ObjectSet::SourceOnly,
                Rewrite::Instead(GameActionTemplate::RemoveCountersFromAffected {
                    counter: CounterType::Stun,
                    n: 1,
                }),
            ),
        ));
    }

    if entry.counter_count(CounterType::Finality) > 0 {
        // > 122.1h … "If this permanent would be put into a graveyard from the
        // > battlefield, exile it instead."
        // Any cause, and the counter is not removed: 122.1h does not say to, and
        // CR 122.2 ends its counters as it leaves anyway.
        out.push((
            CounterType::Finality,
            CounterEffectKind::Replacement,
            ReplacementDef::new(
                EventPattern::ZoneChange {
                    from: Some(Zone::Battlefield),
                    to: Some(Zone::Graveyard),
                    cause: None,
                    object: None,
                },
                ObjectSet::SourceOnly,
                Rewrite::Instead(GameActionTemplate::ZoneChangeTo {
                    to: Zone::Exile,
                    cause: ZoneChangeCause::Exiled,
                }),
            ),
        ));
    }

    out
}

/// The CR 615.5 rider that removes one counter from the shielded permanent.
///
/// `EffectRecipient::Target` because a rider resolves against a
/// `ResolutionContext` whose single target is the affected object; the
/// recipient's filter is never re-validated, since nothing was targeted in the
/// CR 115 sense.
fn remove_one_counter(counter: CounterType) -> Effect {
    Effect::Atom(
        Primitive::RemoveCounters(counter, AmountExpr::Fixed(1)),
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)),
    )
}

/// The applicable effects CR 616.1 lets the affected player choose among, out
/// of everything that applies.
///
/// **616.1a–e is a ladder, not a partition**, which is what the name is trying
/// to say: each step reads "if any of the … effects are [kind], one of them
/// must be chosen. If not, proceed to [the next step]". So the first step with
/// any candidate at all decides the whole question, and everything at a lower
/// step is not a choice the player has right now — it will be offered again on
/// 616.1f's next pass, once the chosen one has applied.
///
/// Named for the rule's own sentence (`codebase-state.md` item 65: the CR
/// never says "bucket").
///
/// Generic over what carries the instance, because the pipeline's candidates
/// carry the members each applies to beside it.
pub(crate) fn must_choose_among<T>(
    candidates: Vec<T>,
    class: impl Fn(&T) -> crate::types::replacement::ReplacementClass,
) -> Vec<T> {
    // `ReplacementClass` derives `Ord` in CR order, so the minimum class present
    // is the ladder's first non-empty step and 616.1e's `Other` the fallthrough;
    // `top` came out of the candidates, so the filter keeps at least one.
    let Some(top) = candidates.iter().map(&class).min() else {
        return candidates;
    };
    candidates.into_iter().filter(|c| class(c) == top).collect()
}

// ---------------------------------------------------------------------------
// CR 903.9b — the commander zone replacement
// ---------------------------------------------------------------------------

/// > 903.9b If a commander would be put into its owner's hand or library from
/// > anywhere, its owner may put it into the command zone instead. **This
/// > replacement effect may apply more than once to the same event. This is an
/// > exception to rule 614.5.**
///
/// The rules' *only* stated exception to CR 614.5, which is why
/// `ReplacementDef::exempt_from_614_5` exists and why it must not grow a second
/// user without a CR cite.
///
/// Synthesized per event rather than registered, for the same reason counters
/// are: it comes from a rule, and nothing on the card says so. "From anywhere"
/// means no `from` constraint; the destination is read off the event so that
/// one arm covers both hand and library without `EventPattern` growing an
/// or-of-zones axis it has no other use for.
///
/// Note the chooser falls out of CR 616.1 without a special case: a card in a
/// graveyard, library or hand has no controller, so `chooser_for` answers with
/// its owner — which is exactly whom 903.9b asks.
fn commander_zone_replacement(
    game: &GameState,
    action: &GameAction,
) -> Option<ReplacementInstance> {
    let GameAction::ZoneChange { object, to, .. } = action else {
        return None;
    };
    if !matches!(to, Zone::Hand | Zone::Library) {
        return None;
    }
    let obj = game.objects.get(object)?;
    if !obj.is_commander {
        return None;
    }
    // "Its owner's hand or library" is CR 400.3's own guarantee — any hand or
    // library destination *is* the owner's (`add_to_zone_collection` files by
    // `obj.owner`), and `ZoneChange` carries no destination player — so there is
    // no check to make here.
    let mut def = ReplacementDef::new(
        EventPattern::ZoneChange {
            from: None,
            to: Some(*to),
            cause: None,
            object: None,
        },
        ObjectSet::Fixed(vec![*object]),
        Rewrite::Instead(GameActionTemplate::ZoneChangeTo {
            to: Zone::Command,
            cause: ZoneChangeCause::CommanderZoneReplacement,
        }),
    )
    .optional();
    def.exempt_from_614_5 = true;
    Some(ReplacementInstance {
        id: ReplacementInstanceId::GameRule(
            *object,
            super::GameRuleReplacement::CommanderZone,
        ),
        source: *object,
        controller: obj.owner,
        def,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::test_support::{put_spell_on_stack, setup_two_player_game, static_ability, test_dp};
    use crate::types::card_types::CardType;
    use crate::types::effects::Condition;
    use crate::types::replacement::{EnterMods, EnterModsTemplate};
    use crate::types::zones::ZoneSet;

    /// The zone leg skips the entering object, which source 1a owns with
    /// CR 614.12's narrower scope.
    ///
    /// The board that shows why: a permanent card whose "permanents enter
    /// tapped" functions from anywhere (CR 113.6b), on the stack, entering. Its
    /// row is filter-scoped, so CR 614.12's parenthesis says it must not apply
    /// to its own entry — source 1a admits `SourceOnly` alone. The zone leg
    /// finds the same card on the stack, where the ability functions, and
    /// reading it there with the sweeps' scope would offer that row to the
    /// entry it is about. Zero candidates is the CR's answer.
    #[test]
    fn the_zone_leg_does_not_read_the_entering_object() {
        let mut game = setup_two_player_game();
        let card = CardDataBuilder::new("Orb From Anywhere")
            .card_type(CardType::Artifact)
            .ability(static_ability(Effect::Conditional(
                Condition::SourceInZone(ZoneSet::ALL),
                Box::new(Effect::Replacement(Box::new(ReplacementDef::new(
                    EventPattern::EnterBattlefield { cast: None },
                    ObjectSet::battlefield_filter(ObjectFilter::All),
                    Rewrite::EnterWith(EnterModsTemplate::tapped()),
                )))),
            )))
            .build();
        let orb = put_spell_on_stack(&mut game, card, 0);
        assert!(
            game.zone_replacement_ability_sources.contains_key(&orb),
            "filed on the stack, where the ability functions"
        );

        let action = GameAction::EnterBattlefield {
            object: orb,
            from: Some(Zone::Stack),
            controller: 0,
            mods: EnterMods::NONE,
            cause: Some(ZoneChangeCause::Resolved),
        };
        let dp = test_dp();
        let ctx = ActionContext::new(&dp);
        let frame = EntryFrame::new(&game, &action);
        let found = gather(&game, &action, &ctx, false, &frame);
        assert!(
            found.is_empty(),
            "CR 614.12 — a filter-scoped row on the entering object reaches its own entry \
             through no source: {:?}",
            found.iter().map(|i| i.id).collect::<Vec<_>>()
        );
    }
}
