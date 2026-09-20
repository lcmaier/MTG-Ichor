//! Detection — the dispatch (`triggers-architecture.md` §4).
//!
//! A dispatch is the matcher's run over one **window**: every record one
//! batch stamped, run when the outermost `execute_actions` that opened it is
//! about to return, after its riders; or one unbatched record, run inside
//! `emit_event` before it returns (§4.1). The instant is still synchronous
//! with the mutation — before the next proposal, before the state-based
//! check, before anyone receives priority — so "that instant is now" holds
//! and the layer walk answers live.
//!
//! **Who is asked** (§4.2) mirrors `replacement::gather` because a new reader
//! of the effective ability list is dead on every board a gate skips: the
//! battlefield sweep behind `trigger_sources`, the zone sweep behind
//! `zone_trigger_sources`, the granted and copied legs on
//! `RegistryScopeSummary`, and the frames the window's departures carry
//! (CR 603.10a). On the pools as they stood before this phase every set is
//! empty and a dispatch is the probes and nothing else.

use std::sync::Arc;

use crate::engine::actions::{ActionContext, GameAction};
use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::condition::settled_holds;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::resolve::ResolutionContext;
use crate::engine::trace_records;
use crate::events::event::{BatchId, DamageTarget, EventRecord, EventSeq, GameEvent};
use crate::objects::card_data::{AbilityDef, AbilityType, CardData};
use crate::oracle::characteristics::controller_or_owner;
use crate::state::game_state::{AbilityIdentity, GameState};
use crate::types::effects::{Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive};
use crate::types::ids::{IdSet, ObjectId, PlayerId};
use crate::types::triggers::{
    DamageRecipient, EventIndex, Multiplicity, ObjectRef, PendingTrigger, TriggerBinding,
    TriggerCondition, TriggerDef, TriggerEvent, TriggerOrigin, TriggerSeq, TriggerSubject,
    TriggerTier,
};
use crate::types::zones::{Zone, ZoneSet};

/// Dispatches nested inside dispatches — a tier-2 trigger's `AbilityTriggered`
/// dispatched from the dispatch that queued it, and so on. The recursion is
/// bounded by the abilities present (no printed ability watches its own
/// kind); a depth past this is the engine's mistake (§4.8).
pub const DISPATCH_NESTING_LIMIT: usize = 16;

/// CR 605.1b's three criteria, derived from the def and never a tag: no
/// target, triggers from mana being added, and could add mana. CR 605.5a is
/// the same sentence from the other side — a target or any other event
/// makes it an ordinary triggered ability.
pub fn is_mana_ability(def: &TriggerDef) -> bool {
    let arms = def.condition.events();
    !arms.is_empty()
        && arms.iter().all(|arm| matches!(arm, TriggerEvent::ManaAdded { .. }))
        && def.effect.instances().is_empty()
        && could_add_mana(&def.effect)
}

fn could_add_mana(effect: &Effect) -> bool {
    match effect {
        Effect::Atom(Primitive::ProduceMana(_), _) => true,
        Effect::Atom(..) => false,
        Effect::Sequence(effects) | Effect::Modal { modes: effects, .. } => {
            effects.iter().any(could_add_mana)
        }
        Effect::Conditional(_, inner)
        | Effect::Optional(inner)
        | Effect::ForEach(_, inner)
        | Effect::Repeat(_, inner) => could_add_mana(inner),
        Effect::Replacement(_)
        | Effect::Restriction(_)
        | Effect::CostModification(_)
        | Effect::Triggered(_) => false,
    }
}

/// CR 603.2f's predicate (S1): the object as the event left it is visible to
/// all players. Its body is `backlog.md` §2.9's to replace with a per-viewer
/// query; today a public zone and not face down is exact.
pub fn visible_to_all(game: &GameState, id: ObjectId) -> bool {
    let Some(object) = game.objects.get(&id) else { return false };
    if !object.zone.is_public() {
        return false;
    }
    !game.battlefield.get(&id).is_some_and(|entry| entry.face_down)
}

/// One candidate ability, as the matcher sees it.
struct Candidate<'a> {
    id: ObjectId,
    /// CR 603.3a's "you": the player who controls the source now, or the
    /// frame's controller for a departed one.
    controller: PlayerId,
    owner: PlayerId,
    zone: Zone,
    /// The source's host (CR 303.4m), for `TriggerSubject::Host`.
    host: Option<ObjectId>,
    /// `None` for a live object — its list is read fresh — and the CR 603.10a
    /// frame for a departed one, which is asked look-back conditions only.
    frame: Option<&'a EffectiveCharacteristics>,
}

/// A match the dispatcher will queue or resolve.
struct Match {
    identity: AbilityIdentity,
    controller: PlayerId,
    def: Arc<TriggerDef>,
    source_card: Arc<CardData>,
    instances: Vec<EffectRecipient>,
    event: EventIndex,
    records: Vec<EventSeq>,
    object: Option<ObjectRef>,
    mana: bool,
}

/// Why a candidate did not trigger — the `trigger` record's field.
#[derive(Clone, Copy)]
enum Refusal {
    /// The condition is a state trigger, which TR-6 checks.
    State,
    /// CR 603.2f.
    Visibility,
    /// The arm does not read this record, or its predicates said no.
    Condition,
    /// CR 603.4 at the trigger.
    InterveningIf,
}

impl Refusal {
    fn name(self) -> &'static str {
        match self {
            Refusal::State => "state",
            Refusal::Visibility => "visibility",
            Refusal::Condition => "condition",
            Refusal::InterveningIf => "intervening_if",
        }
    }
}

impl GameState {
    /// The close of a batch: every record since `mark` that carries `batch`.
    /// Records inside the window with another id are an auxiliary batch's
    /// (`// AUXILIARY-MOVE:`), dispatched at their own close; unstamped ones
    /// are `AbilityTriggered`s, dispatched as they were emitted.
    pub(crate) fn dispatch_batch(
        &mut self,
        mark: usize,
        batch: Option<BatchId>,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        let window: Vec<EventSeq> = (mark..self.events.len())
            .map(EventSeq)
            .filter(|seq| self.events.record(*seq).is_some_and(|r| r.stamp.batch == batch))
            .collect();
        self.dispatch(&window, Some(ctx))
    }

    /// One record emitted outside any batch — a phase beginning, a cast, an
    /// activation, a trigger triggering — dispatched inside `emit_event`.
    pub(crate) fn dispatch_unbatched(&mut self, seq: EventSeq) {
        // No `ActionContext` reaches an emission, and none is needed: a mana
        // trigger's event (`ManaAdded`) is performed inside a batch, so the
        // only path that resolves at dispatch never runs here.
        let _ = self.dispatch(&[seq], None);
    }

    fn dispatch(&mut self, window: &[EventSeq], ctx: Option<&ActionContext>) -> Result<(), String> {
        // CR 104.1 — a game that has ended queues nothing; nobody would
        // receive priority to place it.
        if window.is_empty() || self.result.is_some() {
            return Ok(());
        }
        if self.dispatch_depth >= DISPATCH_NESTING_LIMIT {
            return Err(format!(
                "trigger dispatches nested {} deep: an ability's triggering keeps triggering \
                 another, which no printed ability does",
                self.dispatch_depth
            ));
        }
        self.dispatch_depth += 1;
        let result = self.dispatch_inner(window, ctx);
        self.dispatch_depth -= 1;
        result
    }

    fn dispatch_inner(&mut self, window: &[EventSeq], ctx: Option<&ActionContext>) -> Result<(), String> {
        // --- The gate: five probes, and on the old pools nothing else -------
        let summary = self.continuous_effects.summary();
        let unattributed = summary.granted_trigger_zones | summary.copied_trigger_zones;
        let any_frame_source = window.iter().any(|seq| {
            self.events.record(*seq).is_some_and(|r| {
                frame_of(&r.event).is_some_and(|f| f.abilities.iter().any(is_triggered))
            })
        });
        if self.trigger_sources.is_empty()
            && self.zone_trigger_sources.is_empty()
            && unattributed.is_empty()
            && !any_frame_source
        {
            return Ok(());
        }

        let matches = self.find_matches(window, unattributed);
        if matches.is_empty() {
            return Ok(());
        }

        // --- Queue, or resolve a mana trigger at once (CR 605.4a) ----------
        let mut queued: Vec<(TriggerSeq, TriggerOrigin, PlayerId, EventSeq)> = Vec::new();
        for m in matches {
            let seq = TriggerSeq(self.next_trigger_seq);
            self.next_trigger_seq += 1;
            let binding = TriggerBinding {
                def: Arc::clone(&m.def),
                records: m.records.clone(),
                event: m.event,
                object: m.object,
                triggered: None,
            };
            let caused_by = m.records[0];
            let origin = TriggerOrigin::Object(m.identity);
            let pending = PendingTrigger {
                seq,
                origin,
                controller: m.controller,
                def: Arc::clone(&m.def),
                source_card: Arc::clone(&m.source_card),
                instances: m.instances.clone(),
                binding,
                tier: m.def.condition.tier(),
                mana: m.mana,
                state: false,
            };
            if m.mana {
                match ctx {
                    Some(ctx) => self.resolve_triggered_mana(&pending, ctx)?,
                    // Unreachable: `ManaAdded` is always performed inside a
                    // batch. Loud in debug; queued in release, so the mana is
                    // late rather than lost.
                    None => {
                        debug_assert!(false, "a triggered mana ability reached an unbatched dispatch");
                        self.pending_triggers.push(pending);
                    }
                }
                continue;
            }
            self.pending_triggers.push(pending);
            queued.push((seq, origin, m.controller, caused_by));
        }

        // --- CR 603.3b's second tier watches this record (§4.8) -------------
        // Emitted after the window has closed, unstamped, each dispatched as
        // it is emitted like any unbatched record.
        for (seq, origin, controller, caused_by) in queued {
            self.emit_event_unstamped(GameEvent::AbilityTriggered { seq, origin, controller, caused_by });
        }
        Ok(())
    }

    /// The matcher's read-only half: every candidate ability against every
    /// record of the window, in window order then candidate order — the
    /// order the `OrderTriggers` prompt will offer, which has to be
    /// process-stable end to end (§15 item 1).
    fn find_matches(&self, window: &[EventSeq], unattributed: ZoneSet) -> Vec<Match> {
        // Leg 1: the battlefield, in CR 613.7 order, gated per permanent.
        let on_battlefield = unattributed.contains(Zone::Battlefield);
        let mut live: Vec<ObjectId> = self
            .battlefield_ids_ordered()
            .into_iter()
            .filter(|id| on_battlefield || self.trigger_sources.contains(id))
            .collect();

        // Legs 3 and 4: objects off the battlefield whose ability functions
        // where they are (CR 113.6k, derived) — the record's own subject in a
        // graveyard is one of them — plus the zones a grant or copy reaches,
        // walked whole while such a row exists. In CR 613.7d order.
        let mut elsewhere: Vec<(u64, ObjectId)> = self
            .zone_trigger_sources
            .keys()
            .map(|&id| (self.object_timestamp(id), id))
            .collect();
        let mut seen: IdSet<ObjectId> = elsewhere.iter().map(|&(_, id)| id).collect();
        for zone in unattributed.beyond_battlefield().iter() {
            for id in self.zone_ids_ordered(zone) {
                if seen.insert(id) {
                    elsewhere.push((self.object_timestamp(id), id));
                }
            }
        }
        elsewhere.sort_unstable_by_key(|&(timestamp, _)| timestamp);
        live.extend(elsewhere.into_iter().map(|(_, id)| id));

        let records: Vec<(EventSeq, &EventRecord)> = window
            .iter()
            .filter_map(|seq| self.events.record(*seq).map(|r| (*seq, r)))
            .collect();

        let mut matches: Vec<Match> = Vec::new();
        // "One or more" accumulates across the window: (identity, arm) -> index into `matches`.
        let mut once: Vec<((AbilityIdentity, EventIndex), usize)> = Vec::new();

        // Leg 2: the frames the window carries (CR 603.10a) — each departed
        // object's list as it was, asked look-back conditions only.
        let frames: Vec<(ObjectId, PlayerId, Zone, &EffectiveCharacteristics)> = records
            .iter()
            .filter_map(|(_, r)| match &r.event {
                GameEvent::ZoneChange { object_id, owner, from, lki: Some(frame), .. }
                | GameEvent::LeftTheGame { object_id, owner, from, lki: Some(frame) } => {
                    Some((*object_id, *owner, *from, frame.as_ref()))
                }
                _ => None,
            })
            .collect();

        let mut candidates: Vec<(Candidate<'_>, Arc<Vec<AbilityDef>>, Arc<CardData>)> = Vec::new();
        for id in live {
            let Some(object) = self.objects.get(&id) else { continue };
            let Some(chars) = compute_characteristics(self, id) else { continue };
            candidates.push((
                Candidate {
                    id,
                    controller: controller_or_owner(self, id).unwrap_or(object.owner),
                    owner: object.owner,
                    zone: object.zone,
                    host: self.battlefield.get(&id).and_then(|e| e.attached_to),
                    frame: None,
                },
                Arc::clone(&chars.abilities),
                Arc::clone(&object.card_data),
            ));
        }
        for (id, owner, from, frame) in frames {
            // A departed object's card: still in the store for a zone change,
            // gone for a `LeftTheGame` — then the frame's own list is all
            // there is, and the stack object is built from a card that names
            // what the frame does.
            let card = match self.objects.get(&id) {
                Some(object) => Arc::clone(&object.card_data),
                None => Arc::new(frame_card(frame)),
            };
            candidates.push((
                Candidate { id, controller: frame.controller, owner, zone: from, host: None, frame: Some(frame) },
                Arc::clone(&frame.abilities),
                card,
            ));
        }

        for (seq, record) in &records {
            for (candidate, abilities, card) in &candidates {
                for (index, ability) in abilities.iter().enumerate() {
                    let Effect::Triggered(def) = &ability.effect else { continue };
                    if ability.ability_type != AbilityType::Triggered {
                        continue;
                    }
                    let instance = abilities[..index].iter().filter(|a| a.id == ability.id).count() as u32;
                    let identity = AbilityIdentity {
                        source: candidate.id,
                        zone_change_epoch: self
                            .objects
                            .get(&candidate.id)
                            .map(|o| o.zone_change_epoch)
                            .unwrap_or(0),
                        ability: ability.id,
                        instance,
                    };
                    let outcome = self.match_def(def, candidate, *seq, &record.event);
                    let (arm, subjects, refusal) = match outcome {
                        Ok((arm, subjects)) => (Some(arm), subjects, None),
                        Err(refusal) => (None, Vec::new(), Some(refusal)),
                    };
                    let mana = arm.is_some() && is_mana_ability(def);
                    self.trace(|| {
                        trace_records::trigger(
                            self,
                            *seq,
                            &identity,
                            candidate.zone,
                            arm.is_some(),
                            refusal.map(Refusal::name),
                            mana,
                        )
                    });
                    let Some(arm) = arm else { continue };
                    let arm_event = &def.condition.events()[arm.0];
                    let def_arc: Arc<TriggerDef> = Arc::new((**def).clone());
                    match arm_event.multiplicity() {
                        Multiplicity::PerOccurrence => {
                            for subject in subjects {
                                matches.push(Match {
                                    identity,
                                    controller: candidate.controller,
                                    def: Arc::clone(&def_arc),
                                    source_card: Arc::clone(card),
                                    instances: ability.instances.clone(),
                                    event: arm,
                                    records: vec![*seq],
                                    object: subject.and_then(|id| self.object_ref(id)),
                                    mana,
                                });
                            }
                        }
                        // CR 603.2c's boundary is the window: one trigger, every
                        // matching record in its binding, no one object.
                        Multiplicity::OncePerEvent => {
                            match once.iter().find(|((i, a), _)| *i == identity && *a == arm) {
                                Some((_, at)) => matches[*at].records.push(*seq),
                                None => {
                                    once.push(((identity, arm), matches.len()));
                                    matches.push(Match {
                                        identity,
                                        controller: candidate.controller,
                                        def: Arc::clone(&def_arc),
                                        source_card: Arc::clone(card),
                                        instances: ability.instances.clone(),
                                        event: arm,
                                        records: vec![*seq],
                                        object: None,
                                        mana,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        matches
    }

    /// One def against one record: the first arm that matches, and the
    /// subjects of its occurrences (one per trigger). `Err` names the
    /// predicate that refused.
    fn match_def(
        &self,
        def: &TriggerDef,
        candidate: &Candidate<'_>,
        seq: EventSeq,
        event: &GameEvent,
    ) -> Result<(EventIndex, Vec<Option<ObjectId>>), Refusal> {
        if matches!(def.condition, TriggerCondition::State(_)) {
            return Err(Refusal::State);
        }
        // CR 603.2f, per candidate, of the object as the event left it. A frame
        // candidate was a permanent, which is visible.
        if candidate.frame.is_none() && !visible_to_all(self, candidate.id) {
            return Err(Refusal::Visibility);
        }
        let mut matched: Option<(EventIndex, Vec<Option<ObjectId>>)> = None;
        for (index, arm) in def.condition.events().iter().enumerate() {
            // A frame is asked look-back conditions only (§4.2 leg 2); a live
            // object is asked everything, its look-back arms against the
            // list it has now.
            if candidate.frame.is_some() && !arm.looks_back() {
                continue;
            }
            if !arm.reads(event) {
                continue;
            }
            let subjects = self.arm_occurrences(arm, candidate, seq, event);
            if !subjects.is_empty() {
                matched = Some((EventIndex(index), subjects));
                break;
            }
        }
        let (arm, subjects) = matched.ok_or(Refusal::Condition)?;
        // CR 603.4 at the trigger. "You" is the source's controller, read off
        // the source; a condition about the bound facts is TR-2's reader.
        if let Some(condition) = &def.intervening_if
            && !settled_holds(condition, self, candidate.id)
        {
            return Err(Refusal::InterveningIf);
        }
        Ok((arm, subjects))
    }

    /// The occurrences of `arm` in `event`, as the subject of each — one for
    /// every kind this phase ships, one per matching attacker for the attack
    /// shape. Empty when the arm's predicates refuse the record.
    fn arm_occurrences(
        &self,
        arm: &TriggerEvent,
        candidate: &Candidate<'_>,
        seq: EventSeq,
        event: &GameEvent,
    ) -> Vec<Option<ObjectId>> {
        let one = |ok: bool| if ok { vec![arm.subject_of(event)] } else { Vec::new() };
        match (arm, event) {
            (
                TriggerEvent::ZoneChange { subject, from, to, cause, owner, .. },
                GameEvent::ZoneChange { object_id, owner: moved_owner, from: rf, to: rt, cause: rc, lki },
            ) => one(
                from.is_none_or(|z| z == *rf)
                    && to.is_none_or(|z| z == *rt)
                    && cause.is_none_or(|c| c == *rc)
                    && owner.as_ref().is_none_or(|p| self.player_ref_is(p, *moved_owner, candidate))
                    && self.subject_matches(subject, Some(*object_id), candidate, lki.as_deref()),
            ),
            // CR 603.6c names this event: a leaves-the-battlefield ability
            // triggers on it and nothing narrower — no `to`, no cause.
            (
                TriggerEvent::ZoneChange { subject, from, to, cause, owner, .. },
                GameEvent::LeftTheGame { object_id, owner: moved_owner, from: rf, lki },
            ) => one(
                from.is_none_or(|z| z == *rf)
                    && to.is_none()
                    && cause.is_none()
                    && owner.as_ref().is_none_or(|p| self.player_ref_is(p, *moved_owner, candidate))
                    && self.subject_matches(subject, Some(*object_id), candidate, lki.as_deref()),
            ),
            (TriggerEvent::BecomesTapped { subject }, GameEvent::Tapped { object_id })
            | (TriggerEvent::BecomesUntapped { subject }, GameEvent::Untapped { object_id }) => {
                one(self.subject_matches(subject, Some(*object_id), candidate, None))
            }
            (
                TriggerEvent::ManaAdded { source, tapped_for_mana, mana },
                GameEvent::ManaAdded { source_id, mana: added, tapped_for_mana: tapped, .. },
            ) => one(
                tapped_for_mana.is_none_or(|t| t == *tapped)
                    && mana.is_none_or(|m| added.iter().any(|(t, n)| *t == m && *n > 0))
                    && self.subject_matches(source, Some(*source_id), candidate, None),
            ),
            (
                TriggerEvent::DamageDealt { source, recipient, combat, .. },
                GameEvent::DamageDealt { source_id, target, is_combat, .. },
            ) => {
                let to = match (recipient, target) {
                    (DamageRecipient::Any, _) => true,
                    (DamageRecipient::Player(who), DamageTarget::Player(pid)) => {
                        who.as_ref().is_none_or(|p| self.player_ref_is(p, *pid, candidate))
                    }
                    (DamageRecipient::Object(filter), DamageTarget::Object(id)) => match filter {
                        None => true,
                        Some(filter) => self.subject_matches(
                            &TriggerSubject::Filter(filter.clone()),
                            Some(*id),
                            candidate,
                            None,
                        ),
                    },
                    (DamageRecipient::Player(_), DamageTarget::Object(_))
                    | (DamageRecipient::Object(_), DamageTarget::Player(_)) => false,
                };
                one(
                    to && combat.is_none_or(|c| c == *is_combat)
                        && self.subject_matches(source, Some(*source_id), candidate, None),
                )
            }
            (TriggerEvent::PhaseBegins { phase, whose }, GameEvent::PhaseBegin { phase: rp, player }) => {
                one(phase == rp && whose.as_ref().is_none_or(|p| self.player_ref_is(p, *player, candidate)))
            }
            (TriggerEvent::StepBegins { step, whose }, GameEvent::StepBegin { step: rs, player }) => {
                one(step == rs && whose.as_ref().is_none_or(|p| self.player_ref_is(p, *player, candidate)))
            }
            (TriggerEvent::TurnBegins { whose }, GameEvent::TurnBegin { player, .. }) => {
                one(whose.as_ref().is_none_or(|p| self.player_ref_is(p, *player, candidate)))
            }
            // The sign is the split: a loss is TR-2's `LosesLife`. A 0 gain
            // never reaches the log (CR 119.10, `replacement::never_happens`).
            (TriggerEvent::GainsLife { player: who, .. }, GameEvent::LifeChanged { player_id, old, new, .. }) => {
                one(new > old && who.as_ref().is_none_or(|p| self.player_ref_is(p, *player_id, candidate)))
            }
            (
                TriggerEvent::EntersBattlefield { subject, controller, from, cast, .. },
                GameEvent::PermanentEnteredBattlefield { object_id, controller: rc },
            ) => {
                // The join (§4.4): `from` off the same object's zone change in
                // this window — a token has none — and "cast" off the facts the
                // permanent keeps (CR 400.7d).
                let from_ok = match from {
                    None => true,
                    Some(zone) => self.entry_origin(*object_id, seq) == Some(*zone),
                };
                let cast_ok = cast.is_none_or(|expected| {
                    self.battlefield.get(object_id).is_some_and(|e| e.cast.is_some()) == expected
                });
                one(
                    from_ok
                        && cast_ok
                        && controller.as_ref().is_none_or(|p| self.player_ref_is(p, *rc, candidate))
                        && self.subject_matches(subject, Some(*object_id), candidate, None),
                )
            }
            (TriggerEvent::Attacks { attacker, .. }, GameEvent::AttackersDeclared { attackers }) => attackers
                .iter()
                .filter(|id| self.subject_matches(attacker, Some(**id), candidate, None))
                .map(|id| Some(*id))
                .collect(),
            (TriggerEvent::AbilityTriggers { caused_by, of }, GameEvent::AbilityTriggered { origin, caused_by: cause, .. }) => {
                let of_ok = match of {
                    None => true,
                    Some(filter) => self.subject_matches(
                        &TriggerSubject::Filter(filter.clone()),
                        Some(origin.source()),
                        candidate,
                        None,
                    ),
                };
                let cause_ok = match caused_by {
                    None => true,
                    Some(inner) => self.events.record(*cause).is_some_and(|r| inner.reads(&r.event)),
                };
                one(of_ok && cause_ok)
            }
            _ => Vec::new(),
        }
    }

    /// "Which object" (§3.3): `This` is the source itself, `Host` what it is
    /// attached to, a filter is read against the subject — off the record's
    /// frame for a look-back arm, since the object has left the zone the
    /// frame describes, and off the live board otherwise.
    fn subject_matches(
        &self,
        subject: &TriggerSubject,
        id: Option<ObjectId>,
        candidate: &Candidate<'_>,
        frame: Option<&EffectiveCharacteristics>,
    ) -> bool {
        match (subject, id) {
            (TriggerSubject::Any, _) => true,
            (TriggerSubject::This, Some(id)) => id == candidate.id,
            (TriggerSubject::Host, Some(id)) => candidate.host == Some(id),
            (TriggerSubject::Filter(filter), Some(id)) => self
                .object_matches_filter_of_source(id, filter, candidate.controller, candidate.id, frame)
                .unwrap_or(false),
            (TriggerSubject::This | TriggerSubject::Host | TriggerSubject::Filter(_), None) => false,
        }
    }

    /// "Whose" — a `PlayerRef` against a record's player, read for the
    /// candidate (CR 109.5's "you" is its controller).
    fn player_ref_is(&self, who: &PlayerRef, player: PlayerId, candidate: &Candidate<'_>) -> bool {
        match who {
            PlayerRef::You => player == candidate.controller,
            PlayerRef::Opponent => player != candidate.controller,
            PlayerRef::Owner => player == candidate.owner,
            PlayerRef::Player(pid) => player == *pid,
        }
    }

    /// The zone the permanent at `id` entered from, off its own zone change
    /// earlier in the same window — `None` for a token, which came from
    /// nowhere (CR 111.2).
    fn entry_origin(&self, id: ObjectId, entered_at: EventSeq) -> Option<Zone> {
        let mut seq = entered_at.0;
        while seq > 0 {
            seq -= 1;
            let record = self.events.record(EventSeq(seq))?;
            match &record.event {
                GameEvent::ZoneChange { object_id, to: Zone::Battlefield, from, .. } if *object_id == id => {
                    return Some(*from)
                }
                GameEvent::TokenCreated { object_id, .. } if *object_id == id => return None,
                _ => {}
            }
        }
        None
    }

    fn object_ref(&self, id: ObjectId) -> Option<ObjectRef> {
        self.objects.get(&id).map(|o| ObjectRef { id, zone_change_epoch: o.zone_change_epoch })
    }

    /// CR 605.4a — a triggered mana ability resolves immediately after the
    /// mana ability that triggered it, with no stack object, no priority and
    /// no target. Through the same proposal an activated one makes, so a
    /// replacement or a "can't" sees it; `tapped_for_mana` is false (CR 106.12
    /// — the Aura was not tapped). "Its controller adds" is the `Host`
    /// recipient: the enchanted land's controller, whoever controls the Aura
    /// (Wild Growth's ruling — the mana is not the land's ability).
    fn resolve_triggered_mana(&mut self, pending: &PendingTrigger, ctx: &ActionContext) -> Result<(), String> {
        let source = pending.origin.source();
        let effect = pending.def.effect.clone();
        self.resolve_mana_trigger_effect(&effect, pending, source, ctx)
    }

    fn resolve_mana_trigger_effect(
        &mut self,
        effect: &Effect,
        pending: &PendingTrigger,
        source: ObjectId,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        match effect {
            Effect::Atom(Primitive::ProduceMana(output), recipient) => {
                let player = match recipient {
                    EffectRecipient::Host => self
                        .battlefield
                        .get(&source)
                        .and_then(|e| e.attached_to)
                        .and_then(|host| controller_or_owner(self, host))
                        .unwrap_or(pending.controller),
                    _ => pending.controller,
                };
                let mut resolution = ResolutionContext::untargeted(source, player);
                resolution.ability_source = Some(source);
                resolution.trigger = Some(pending.binding.clone());
                let mut mana = Vec::with_capacity(output.mana.len());
                for (mana_type, amount) in &output.mana {
                    mana.push((*mana_type, self.evaluate_amount(amount, &resolution)?));
                }
                self.execute_action(
                    GameAction::ProduceMana {
                        player,
                        source,
                        mana,
                        special: output.special.clone(),
                        tapped_for_mana: false,
                    },
                    &ActionContext::resolving(ctx.dp, &resolution),
                )
            }
            Effect::Sequence(effects) => {
                for sub in effects {
                    self.resolve_mana_trigger_effect(sub, pending, source, ctx)?;
                }
                Ok(())
            }
            other => Err(format!("unsupported effect in a triggered mana ability: {:?}", other)),
        }
    }
}

fn is_triggered(def: &AbilityDef) -> bool {
    def.ability_type == AbilityType::Triggered && matches!(def.effect, Effect::Triggered(_))
}

/// The CR 603.10a frame a record carries, if it carries one.
fn frame_of(event: &GameEvent) -> Option<&EffectiveCharacteristics> {
    match event {
        GameEvent::ZoneChange { lki, .. } | GameEvent::LeftTheGame { lki, .. } => lki.as_deref(),
        _ => None,
    }
}

/// A card for a frame whose object has left the game (CR 800.4a): the name
/// the frame carried and its abilities, which is what CR 603.3's "text of
/// the ability that created it" needs and nothing more.
fn frame_card(frame: &EffectiveCharacteristics) -> CardData {
    let mut builder = crate::objects::card_data::CardDataBuilder::new(&frame.name);
    for ability in frame.abilities.iter() {
        builder = builder.ability(ability.clone());
    }
    Arc::try_unwrap(builder.build()).unwrap_or_else(|arc| (*arc).clone())
}

/// A filter's `NotSource` is what "another" excludes; exposed for the
/// card files so "another creature" reads as one expression.
pub fn another(filter: ObjectFilter) -> ObjectFilter {
    ObjectFilter::And(Box::new(filter), Box::new(ObjectFilter::NotSource))
}

/// A tier-2 condition's tier, re-exported for the placement's drain.
pub fn tier_of(def: &TriggerDef) -> TriggerTier {
    def.condition.tier()
}
