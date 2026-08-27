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
//! 2. **Static abilities functioning in other zones** — deferred past Phase RE.
//! 3. **Continuous effects with a duration, from resolutions** — the registry.
//! 4. **Shields from resolutions** (CR 615.7/615.8, 701.19a) — the registry.
//! 5. **Counters** (CR 122.1c/d/h) — synthesized during the sweep, because they
//!    come from the *counter* and nothing on the card says so.
//!
//! Plus CR 614.15's self-replacement effects, which belong to the resolving
//! spell or ability rather than to any source above and would arrive through
//! `ActionContext::resolution`. `ResolutionContext` has no field to carry them
//! and gains one with the first card that needs one — §11 item 3, and the
//! reason `ReplacementClass::SelfReplacement` currently has a bucket and no
//! producer.

use crate::engine::actions::{ActionContext, GameAction};
use crate::events::event::DamageTarget;
use crate::objects::card_data::AbilityType;
use crate::oracle::characteristics::{get_effective_abilities, get_effective_controller};
use crate::state::game_state::GameState;
use crate::types::effects::{
    AffectedSet, AmountExpr, CounterType, Effect, EffectRecipient, PermanentFilter, Primitive,
    SelectionFilter, TargetCount,
};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::{
    EventPattern, GameActionTemplate, ReplacementClass, ReplacementDef, Rewrite,
};
use crate::types::zones::{Zone, ZoneChangeCause};

use super::{ReplacementInstance, ReplacementInstanceId};

/// Which of CR 122.1c's two effects a counter-derived instance is.
///
/// > 122.1c One or more shield counters on a permanent create a single
/// > replacement effect **and** a single prevention effect that protect the
/// > permanent.
///
/// Stun (122.1d) and finality (122.1h) create only the replacement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CounterEffectKind {
    Replacement,
    Prevention,
}

/// What a proposed event is *about* — CR 614.1's "whatever they're affecting"
/// and CR 616.1's "the affected object ... or the affected player".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Affected {
    Object(ObjectId),
    Player(PlayerId),
}

/// The affected object or player of a proposed action.
pub(crate) fn affected_of(action: &GameAction) -> Affected {
    match action {
        GameAction::DealDamage { target, .. } => match target {
            DamageTarget::Object(id) => Affected::Object(*id),
            DamageTarget::Player(pid) => Affected::Player(*pid),
        },
        GameAction::DrawCard { player } => Affected::Player(*player),
        GameAction::GainLife { player, .. } => Affected::Player(*player),
        GameAction::LoseLife { player, .. } => Affected::Player(*player),
        GameAction::ZoneChange { object, .. } => Affected::Object(*object),
        GameAction::Untap { object } => Affected::Object(*object),
        GameAction::Tap { object } => Affected::Object(*object),
        GameAction::Destroy { object, .. } => Affected::Object(*object),
        GameAction::AddCounters { object, .. } => Affected::Object(*object),
        GameAction::RemoveCounters { object, .. } => Affected::Object(*object),
    }
}

/// CR 616.1's chooser: "the affected object's controller (or its owner if it
/// has no controller) or the affected player".
///
/// A lookup, and therefore already N-player-safe — no `bool`, no "the other
/// player".
pub(crate) fn chooser_for(game: &GameState, affected: Affected) -> Option<PlayerId> {
    match affected {
        Affected::Player(pid) => Some(pid),
        Affected::Object(id) => get_effective_controller(game, id)
            .or_else(|| game.objects.get(&id).map(|obj| obj.owner)),
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
pub(crate) fn gather(
    game: &GameState,
    action: &GameAction,
    _ctx: &ActionContext,
    blocked: bool,
) -> Vec<ReplacementInstance> {
    // CR 614.17c's filter is applied at the door rather than at the end: with
    // no self-replacement producer, a blocked event has no candidates at all
    // and the whole sweep below is dead work.
    if blocked {
        return Vec::new();
    }

    // The fast path, and it is not an optimization — it is the difference
    // between the pipeline being free and the pipeline tripling the engine's
    // cost. `get_effective_abilities` is a full `compute_characteristics` walk,
    // and an ungated sweep would run one per permanent per proposed action:
    // measured against the untap step alone that is ~6,000 extra layer walks
    // per `fuzz_games` game.
    //
    // The gate is *exact*, not a heuristic. An object on the battlefield can
    // only have a static replacement ability if it printed one — recorded in
    // `replacement_ability_sources` at ETB — or if a Layer 6 row granted it
    // one, which the registry summary reports. Both over-approximate (CR 305.7
    // and Humility can strip a printed ability without touching the set), and
    // over-approximating costs a walk, never an answer.
    let affected = affected_of(action);
    let mut candidates = Vec::new();

    // --- Game rules that behave as replacement effects (CR 903.9b) ---------
    //
    // Ahead of the fast-path gate, because this source is neither a battlefield
    // sweep nor a registry row and the gate would skip it. It costs one
    // `HashMap` lookup and is exact: 903.9b only ever applies to the object the
    // event is already about.
    if let Some(instance) = commander_zone_replacement(game, action) {
        push_if_applicable(game, &mut candidates, instance, action, affected);
    }

    let has_static_source = !game.replacement_ability_sources.is_empty()
        || game.continuous_effects.summary().any_granted_replacement;
    if !has_static_source
        && game.replacement_effects.is_empty()
        && !any_replacement_counter(game)
    {
        return candidates;
    }

    // --- Sources 1 and 5: the battlefield sweep ----------------------------
    for id in game.battlefield_ids_ordered() {
        if has_static_source {
            let controller = get_effective_controller(game, id).unwrap_or_else(|| {
                game.objects.get(&id).map(|o| o.owner).unwrap_or(0)
            });
            for ability in get_effective_abilities(game, id) {
                if ability.ability_type != AbilityType::Static {
                    continue;
                }
                let Effect::Replacement(def) = &ability.effect else {
                    continue;
                };
                push_if_applicable(
                    game,
                    &mut candidates,
                    ReplacementInstance {
                        id: ReplacementInstanceId::StaticAbility(id, ability.id),
                        source: id,
                        controller,
                        def: (**def).clone(),
                    },
                    action,
                    affected,
                );
            }
        }

        for (counter, kind, def) in counter_replacements(game, id) {
            let controller = get_effective_controller(game, id).unwrap_or_else(|| {
                game.objects.get(&id).map(|o| o.owner).unwrap_or(0)
            });
            push_if_applicable(
                game,
                &mut candidates,
                ReplacementInstance {
                    id: ReplacementInstanceId::Counter(id, counter, kind),
                    source: id,
                    controller,
                    def,
                },
                action,
                affected,
            );
        }
    }

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
            action,
            affected,
        );
    }

    candidates
}

fn push_if_applicable(
    game: &GameState,
    out: &mut Vec<ReplacementInstance>,
    instance: ReplacementInstance,
    action: &GameAction,
    affected: Affected,
) {
    // CR 701.19c — "effects that say that a permanent can't be regenerated ...
    // cause regeneration shields to **not be applied**". Withheld at the door,
    // which is what makes the rule's distinction real: the shield still exists,
    // is still not spent, and is still there for a later turn.
    if instance.def.is_regeneration {
        if let Affected::Object(id) = affected {
            if game.cant_be_regenerated.contains(&id) {
                return;
            }
        }
    }
    if applies_to(game, &instance, action, affected) {
        out.push(instance);
    }
}

/// Does this effect apply to this event — CR 614.1's pattern *and* shield?
fn applies_to(
    game: &GameState,
    instance: &ReplacementInstance,
    action: &GameAction,
    affected: Affected,
) -> bool {
    pattern_matches(game, &instance.def, action, instance.controller)
        && shield_contains(game, instance, affected)
}

/// CR 614.1's "shields around whatever they're affecting".
fn shield_contains(
    game: &GameState,
    instance: &ReplacementInstance,
    affected: Affected,
) -> bool {
    let id = match affected {
        Affected::Object(id) => id,
        // No `AffectedSet` variant names a player, so a player-affecting event
        // falls outside every shield the type can express. That is not a
        // silently wrong answer, it is the reason `EventPattern` has no
        // `DrawCard`/`GainLife`/`LoseLife` arm: a replacement effect that
        // shields a *player* needs a player-scoping mechanism, and it lands in
        // Phase RE with the cards that want it.
        Affected::Player(_) => return false,
    };
    match &instance.def.affected {
        AffectedSet::SourceOnly => instance.source == id,
        AffectedSet::Fixed(ids) => ids.contains(&id),
        AffectedSet::Filter { filter } => game
            .permanent_matches_filter(id, filter, instance.controller)
            .unwrap_or(false),
    }
}

/// Does the proposed event match the pattern?
fn pattern_matches(
    game: &GameState,
    def: &ReplacementDef,
    action: &GameAction,
    you: PlayerId,
) -> bool {
    match (&def.pattern, action) {
        (EventPattern::DealDamage, GameAction::DealDamage { .. }) => true,

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
                    .map(|f| game.permanent_matches_filter(*moving, f, you).unwrap_or(false))
                    .unwrap_or(true)
        }

        (EventPattern::Untap, GameAction::Untap { .. }) => true,
        (EventPattern::Tap, GameAction::Tap { .. }) => true,

        (EventPattern::Destroy { source }, GameAction::Destroy { source: actual, .. }) => {
            source.map(|p| p.matches(*actual)).unwrap_or(true)
        }

        (
            EventPattern::CounterChange { counter, adding },
            GameAction::AddCounters { counter: actual, .. },
        ) => *adding && counter.map(|c| c == *actual).unwrap_or(true),
        (
            EventPattern::CounterChange { counter, adding },
            GameAction::RemoveCounters { counter: actual, .. },
        ) => !*adding && counter.map(|c| c == *actual).unwrap_or(true),

        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Source 5 — counters (CR 122.1c/d/h)
// ---------------------------------------------------------------------------

/// The three counter kinds that generate a replacement effect.
const REPLACEMENT_COUNTERS: [CounterType; 3] =
    [CounterType::Shield, CounterType::Stun, CounterType::Finality];

/// Is any permanent carrying a counter that generates a replacement effect?
///
/// Part of `gather`'s fast path, and deliberately *computed* rather than
/// cached: a cached count would have to be maintained at every site that
/// touches `BattlefieldEntity::counters`, and a drifting count reads as a card
/// that silently does nothing — which is the exact failure this phase exists to
/// remove. The scan is a `HashMap` walk over the battlefield that skips
/// immediately on the empty-counters case, which is almost every permanent.
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
        // > 122.1c ... "If this permanent would be destroyed as the result of
        // > an effect, instead remove a shield counter from it"
        //
        // "As the result of an effect" is CR 701.8b way 1 only. A shield
        // counter does not answer CR 704.5g's lethal-damage destruction through
        // this half at all — the prevention half below stops the damage before
        // 704.5g ever asks.
        out.push((
            CounterType::Shield,
            CounterEffectKind::Replacement,
            ReplacementDef::new(
                EventPattern::Destroy {
                    source: Some(crate::types::replacement::DestructionSourcePattern::Effect),
                },
                AffectedSet::SourceOnly,
                Rewrite::Instead(GameActionTemplate::RemoveCountersFromAffected {
                    counter: CounterType::Shield,
                    n: 1,
                }),
            ),
        ));

        // > 122.1c ... "If damage would be dealt to this permanent, prevent
        // > that damage and remove a shield counter from it"
        //
        // `Prevent` plus a rider rather than an `Instead`: CR 615.13 lets
        // triggers fire on damage *being prevented*, so the engine has to know
        // a prevention happened rather than seeing a substituted event.
        out.push((
            CounterType::Shield,
            CounterEffectKind::Prevention,
            ReplacementDef::new(
                EventPattern::DealDamage,
                AffectedSet::SourceOnly,
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
                AffectedSet::SourceOnly,
                Rewrite::Instead(GameActionTemplate::RemoveCountersFromAffected {
                    counter: CounterType::Stun,
                    n: 1,
                }),
            ),
        ));
    }

    if entry.counter_count(CounterType::Finality) > 0 {
        // > 122.1h ... "If this permanent would be put into a graveyard from
        // > the battlefield, exile it instead."
        //
        // Any cause — this is not restricted to destruction — and the counter
        // is *not* removed: 122.1h does not say to, and the permanent is
        // leaving anyway, which CR 122.2 makes the end of its counters.
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
                AffectedSet::SourceOnly,
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
        EffectRecipient::Target(SelectionFilter::Permanent(PermanentFilter::All), TargetCount::Exactly(1)),
    )
}

/// CR 616.1a–e's forced buckets. Exposed for the pipeline's use.
pub(crate) fn forced_bucket(candidates: Vec<ReplacementInstance>) -> Vec<ReplacementInstance> {
    // `ReplacementClass` derives `Ord` in CR order, so the minimum present
    // class is 616.1's highest-priority non-empty one and 616.1e's `Other` is
    // the fallthrough by construction.
    let Some(top) = candidates.iter().map(|c| c.def.class).min() else {
        return candidates;
    };
    debug_assert!(
        top == ReplacementClass::Other || candidates.iter().any(|c| c.def.class == top),
        "forced_bucket picked a class nothing is in"
    );
    candidates.into_iter().filter(|c| c.def.class == top).collect()
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
    // "Its owner's hand or library" — a commander headed for someone else's
    // hand is not this rule's business.
    if obj.owner != owner_of_destination(game, *object) {
        return None;
    }
    let mut def = ReplacementDef::new(
        EventPattern::ZoneChange {
            from: None,
            to: Some(*to),
            cause: None,
            object: None,
        },
        AffectedSet::Fixed(vec![*object]),
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

/// Whose hand or library a zone change is headed for.
///
/// `add_to_zone_collection` puts a card into its **owner's** hand or library
/// unconditionally, so the destination's player is the owner and 903.9b's
/// "its owner's" clause is satisfied by construction. Written out anyway,
/// because the day an effect puts a card into a *different* player's library
/// this becomes the check that stops 903.9b firing on it.
fn owner_of_destination(game: &GameState, object: ObjectId) -> PlayerId {
    game.objects.get(&object).map(|o| o.owner).unwrap_or(0)
}
