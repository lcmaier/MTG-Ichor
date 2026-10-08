//! The delayed-trigger registry (CR 603.7; `triggers-architecture.md` §3.9,
//! §4.6).
//!
//! An effect creates a delayed triggered ability: a resolution's instruction,
//! a replacement's rider, or a special action a static ability allows (CR
//! 603.7a). It then waits on `GameState::delayed_triggers` for its event. It
//! is no object's ability, so the dispatcher asks the registry as a leg of its
//! own, beside the objects' leg, at the same windows and through the same arm
//! matcher ([`TriggerReferents`]). Its "this object" is its CR 603.7d–g source,
//! remembered by identity.
//!
//! **CR 603.7a is one filter, and 513.2 falls out of it.** An entry reads
//! only the records performed after it was created, and none of the event it
//! was created during. Most windows hold nothing earlier anyway; the batch a
//! rider creates one in does, on both sides of the rider. So an event just
//! before the entry's creation never reaches it, and a delayed "at the
//! beginning of the next end step" created during the end step waits for the
//! next turn's.

use std::sync::Arc;

use super::dispatch::{TriggerReferents, DelayedMatch, MatchedTrigger, Refusal, ThisObject};
use super::history::TurnOrdinals;
use crate::engine::actions::ActionContext;
use crate::engine::layers::condition::settled_holds;
use crate::engine::trace_records;
use crate::events::event::{EventRecord, EventSeq, GameEvent};
use crate::state::game_state::GameState;
use crate::types::ids::{DelayedTriggerId, ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::{
    DelayedDuration, DelayedProvenance, DelayedTrigger, DelayedTriggerTemplate, DepartedFrame, EventIndex, EventKindMask,
    Multiplicity, PendingTrigger, TriggerBinding, TriggerLimit, TriggerOrigin, TriggerSeq, TriggerTurn,
};
use crate::ui::ask::ask_choose_delayed_trigger_event;
use crate::ui::choice_types::ChoiceOption;

impl GameState {
    /// The registry's one door (CR 603.7a). The caller supplies the
    /// provenance its rule gives: a resolution reads CR 603.7d–f's off its
    /// context, and a special action a static ability allows would read
    /// 603.7g's off that ability's object (`backlog.md` §2.8). Returns the
    /// new entry's id. Announced first, so its creation is no record it reads.
    pub fn register_delayed_trigger(
        &mut self,
        template: &DelayedTriggerTemplate,
        provenance: DelayedProvenance,
    ) -> DelayedTriggerId {
        let id = DelayedTriggerId(self.next_delayed_trigger_id);
        self.next_delayed_trigger_id += 1;
        self.emit_event(GameEvent::DelayedTriggerCreated {
            id,
            source: provenance.source.id,
            controller: provenance.controller,
            rules_text: template.rules_text,
            duration: template.duration,
        });
        self.delayed_triggers.push(DelayedTrigger {
            id,
            def: Arc::clone(&template.def),
            source: provenance.source,
            source_left_at: None,
            source_frame: None,
            source_card: provenance.source_card,
            controller: provenance.controller,
            created_at: self.events.next_seq(),
            created_in: self.events.current_stamp().batch,
            created_by: provenance.created_by,
            duration: template.duration,
            x_value: provenance.x_value,
            turn: provenance.turn,
            instances: template.def.effect.instances(),
            rules_text: template.rules_text,
        });
        id
    }

    /// CR 400.7: object `id` is about to move, and the next record is that
    /// move. A delayed trigger whose source it is keeps that record's number,
    /// which is how "when this creature leaves the battlefield" knows its own
    /// departure. The callers are the movers whose record follows the move
    /// at once: `perform_zone_change` and a player leaving (CR 800.4a).
    pub(crate) fn note_delayed_source_moving(&mut self, id: ObjectId) {
        if self.delayed_triggers.is_empty() {
            return;
        }
        if let Some(object) = self.object_ref(id) {
            self.note_delayed_source_left(object);
        }
    }

    /// [`Self::note_delayed_source_moving`] for an existence that has
    /// already moved silently: CR 601.2a's cast, which is announced as the
    /// next record only once the spell becomes cast (601.2i).
    pub(crate) fn note_delayed_source_left(&mut self, object: ObjectRef) {
        let at = self.events.next_seq();
        for delayed in self.delayed_triggers.iter_mut() {
            if delayed.source == object && delayed.source_left_at.is_none() {
                delayed.source_left_at = Some(at);
            }
        }
    }

    /// CR 514.2: the cleanup step ends "this turn" effects, and a delayed
    /// triggered ability with that stated duration ends with them.
    pub(crate) fn end_this_turn_delayed_triggers(&mut self) {
        self.delayed_triggers.retain(|d| d.duration != DelayedDuration::ThisTurn);
    }

    /// A turn has begun. An entry waiting for an extra turn that is neither
    /// this turn nor still in the queue can never trigger: that turn has
    /// ended, or it was skipped, and CR 614.10a says anything scheduled for a
    /// skipped turn won't happen.
    pub(crate) fn drop_delayed_triggers_of_gone_turns(&mut self) {
        let current = self.extra_turn;
        let queue = &self.turn_queue;
        self.delayed_triggers.retain(|d| match d.turn {
            TriggerTurn::Extra(turn) => current == Some(turn) || queue.iter().any(|e| e.id == turn),
            TriggerTurn::Any | TriggerTurn::LaterThan(_) => true,
        });
    }

    /// Whether the turn in progress is one a delayed trigger bound to `turn`
    /// may trigger in.
    fn in_turn(&self, turn: TriggerTurn) -> bool {
        match turn {
            TriggerTurn::Any => true,
            TriggerTurn::LaterThan(created) => self.turn_number > created,
            TriggerTurn::Extra(extra) => self.extra_turn == Some(extra),
        }
    }

    /// The registry's leg of a dispatch (§4.6): each entry whose def reads a
    /// kind the window carries, in its turn, against every record of the
    /// window. Read-only, like the objects' leg, and in creation order.
    pub(super) fn detect_delayed(
        &self,
        window: &[EventSeq],
        window_kinds: EventKindMask,
        ordinals: &TurnOrdinals,
    ) -> Vec<DelayedMatch> {
        if self.delayed_triggers.is_empty() {
            return Vec::new();
        }
        let mut found = Vec::new();
        for delayed in &self.delayed_triggers {
            if !delayed.def.record_kinds().intersects(window_kinds) || !self.in_turn(delayed.turn) {
                continue;
            }
            let occurrences = self.delayed_occurrences(delayed, window, ordinals);
            if !occurrences.is_empty() {
                found.push(DelayedMatch { id: delayed.id, occurrences });
            }
        }
        found
    }

    /// One entry against the window: every occurrence that triggers it, in
    /// window order. A record counts if the entry existed before it (CR
    /// 603.7a) and one of its arms reads the record's kind; it is matched the
    /// way an object's ability is (`match_delayed`), the verdict is traced,
    /// and a match adds its occurrences (`add_occurrences`).
    fn delayed_occurrences(
        &self,
        delayed: &DelayedTrigger,
        window: &[EventSeq],
        ordinals: &TurnOrdinals,
    ) -> Vec<MatchedTrigger> {
        let referents = self.delayed_referents(delayed);
        let arms = delayed.def.condition.events();
        let mut occurrences: Vec<MatchedTrigger> = Vec::new();
        for seq in window {
            let Some(record) = self.events.record(*seq) else { continue };
            if !existed_before(delayed, record) || !arms.iter().any(|arm| arm.reads(&record.event)) {
                continue;
            }
            let verdict = self.match_delayed(delayed, &referents, record, ordinals);
            self.trace_delayed_verdict(delayed, record.seq, verdict.as_ref().err().copied());
            if let Ok((event, subjects)) = verdict {
                self.add_occurrences(&mut occurrences, delayed, record, event, subjects);
            }
        }
        occurrences
    }

    /// What an entry's words for itself mean (CR 603.7c–g): "this object" is
    /// its source as remembered, "you" its controller, "its owner" the
    /// source's owner while the store holds it, and "this ability" the one
    /// that created it (CR 603.7h).
    fn delayed_referents(&self, delayed: &DelayedTrigger) -> TriggerReferents {
        TriggerReferents {
            this: ThisObject::Remembered { object: delayed.source, left_at: delayed.source_left_at },
            controller: delayed.controller,
            owner: self.objects.get(&delayed.source.id).map_or(delayed.controller, |o| o.owner),
            host: None,
            this_ability: delayed.created_by,
        }
    }

    /// The entry against one record, as `match_def` is for an object's
    /// ability: the first arm that matches and its occurrences' subjects,
    /// then the def's once-per-turn limit and its intervening "if". `Err`
    /// names the predicate that refused.
    fn match_delayed(
        &self,
        delayed: &DelayedTrigger,
        referents: &TriggerReferents,
        record: &EventRecord,
        ordinals: &TurnOrdinals,
    ) -> Result<(EventIndex, Vec<Option<ObjectId>>), Refusal> {
        let matched = delayed
            .def
            .condition
            .events()
            .iter()
            .enumerate()
            .filter(|(_, arm)| arm.reads(&record.event))
            .find_map(|(index, arm)| {
                let subjects = self.occurrences_matching_arm(arm, referents, record.seq, &record.event);
                (!subjects.is_empty()).then_some((EventIndex(index), subjects))
            })
            .ok_or(Refusal::TriggerCondition)?;
        if !self.within_once_per_turn_limit(&delayed.def, delayed.identity(), delayed.controller, record.seq, ordinals) {
            return Err(Refusal::Limit);
        }
        // CR 603.4 at the trigger, "you" its controller (CR 109.5).
        if let Some(condition) = &delayed.def.intervening_if
            && !settled_holds(condition, self, delayed.source.id, Some(delayed.controller))
        {
            return Err(Refusal::InterveningIf);
        }
        Ok(matched)
    }

    /// The trace's `trigger` record of an entry's verdict on record `seq`,
    /// under the zone its source is in; none once the store has no source.
    fn trace_delayed_verdict(&self, delayed: &DelayedTrigger, seq: EventSeq, refusal: Option<Refusal>) {
        let Some(zone) = self.objects.get(&delayed.source.id).map(|o| o.zone) else { return };
        self.trace(|| {
            trace_records::trigger(self, seq, &delayed.identity(), zone, refusal.is_none(), refusal.map(Refusal::name), false)
        });
    }

    /// A record's match on arm `event`, added to the entry's occurrences: one
    /// per subject, or, for a "one or more" arm (CR 603.2c), the record joins
    /// that arm's single occurrence across the window. An arm is one kind or
    /// the other, so an occurrence of a "one or more" arm is its fold.
    fn add_occurrences(
        &self,
        occurrences: &mut Vec<MatchedTrigger>,
        delayed: &DelayedTrigger,
        record: &EventRecord,
        event: EventIndex,
        subjects: Vec<Option<ObjectId>>,
    ) {
        let occurrence = |subject: Option<ObjectId>| MatchedTrigger {
            identity: delayed.identity(),
            controller: delayed.controller,
            def: Arc::clone(&delayed.def),
            source_card: Arc::clone(&delayed.source_card),
            instances: delayed.instances.clone(),
            event,
            records: vec![record.clone()],
            subject: subject.and_then(|id| self.object_ref(id)),
            mana: false,
        };
        match delayed.def.condition.events()[event.0].multiplicity() {
            Multiplicity::PerOccurrence => occurrences.extend(subjects.into_iter().map(occurrence)),
            Multiplicity::OncePerEvent => match occurrences.iter_mut().find(|o| o.event == event) {
                Some(folded) => folded.records.push(record.clone()),
                None => occurrences.push(occurrence(None)),
            },
        }
    }

    /// Queue the registry's matches, after the objects' (§4.6). A `ThisTurn`
    /// entry triggers on every occurrence and stays. A `Once` entry triggers
    /// once and leaves as it queues (CR 603.7b), its controller choosing
    /// among simultaneous occurrences.
    pub(super) fn queue_delayed(
        &mut self,
        matched: Vec<DelayedMatch>,
        ctx: Option<&ActionContext>,
        queued: &mut Vec<(TriggerSeq, TriggerOrigin, PlayerId, EventSeq)>,
    ) {
        for DelayedMatch { id, occurrences } in matched {
            let Some(at) = self.delayed_triggers.iter().position(|d| d.id == id) else { continue };
            let delayed = &self.delayed_triggers[at];
            let duration = delayed.duration;
            let entries: Vec<PendingTrigger> = occurrences.into_iter().map(|m| pending_of(m, delayed)).collect();
            let triggered = match duration {
                DelayedDuration::ThisTurn => entries,
                DelayedDuration::Once => {
                    let source = self.delayed_triggers.remove(at).source.id;
                    self.choose_delayed_cause(entries, source, ctx).into_iter().collect()
                }
            };
            for mut pending in triggered {
                let identity = pending.origin.identity();
                if pending.binding.def.limit == Some(TriggerLimit::TriggersOnlyOnceEachTurn)
                    && !self.triggered_this_turn.insert(identity)
                {
                    continue;
                }
                pending.seq = TriggerSeq(self.next_trigger_seq);
                self.next_trigger_seq += 1;
                let caused_by = pending.binding.records.first().map_or(EventSeq(0), |r| r.seq);
                queued.push((pending.seq, pending.origin, pending.controller, caused_by));
                self.pending_triggers.push(pending);
            }
        }
    }

    /// CR 603.7b: "if its trigger event occurs more than once simultaneously
    /// ..., the controller of the delayed triggered ability chooses which
    /// event causes the ability to trigger". Causes that agree on every fact
    /// the def reads give one game (item 163's reading, §5.2), so the
    /// controller is asked only between causes that differ, and only when
    /// there are two or more of them.
    fn choose_delayed_cause(
        &self,
        entries: Vec<PendingTrigger>,
        source: ObjectId,
        ctx: Option<&ActionContext>,
    ) -> Option<PendingTrigger> {
        let reads = entries.first()?.binding.def.bound_reads();
        let mut distinct: Vec<PendingTrigger> = Vec::new();
        for entry in entries {
            if !distinct.iter().any(|kept| self.entries_agree_on(reads, kept, &entry)) {
                distinct.push(entry);
            }
        }
        if distinct.len() < 2 {
            return distinct.pop();
        }
        let Some(ctx) = ctx else {
            // Two occurrences of one arm come only in a batch's window or in
            // `AttackersDeclared`, and both dispatch with a provider.
            debug_assert!(false, "CR 603.7b's choice reached a dispatch with no provider to ask");
            return distinct.into_iter().next();
        };
        // Each cause by its object, or by its record where objects do not
        // tell the causes apart.
        let by_object: Vec<Option<ObjectId>> = distinct.iter().map(|e| e.binding.subject.map(|s| s.id)).collect();
        let objects_tell = by_object.iter().all(Option::is_some)
            && by_object.iter().enumerate().all(|(i, a)| by_object[..i].iter().all(|b| b != a));
        let options: Vec<ChoiceOption> = distinct
            .iter()
            .zip(&by_object)
            .map(|(entry, object)| match object {
                Some(id) if objects_tell => ChoiceOption::Object(*id),
                _ => ChoiceOption::Number(entry.binding.records.first().map_or(0, |r| r.seq.0 as u64)),
            })
            .collect();
        let controller = distinct[0].controller;
        let chosen = ask_choose_delayed_trigger_event(ctx.dp, self, controller, source, &options);
        distinct.into_iter().nth(chosen)
    }
}

/// CR 603.7a: whether `delayed` existed before the event `record` is part of
/// — created ahead of the record, and not during that event, which a rider's
/// batch performs on both sides of the rider.
fn existed_before(delayed: &DelayedTrigger, record: &EventRecord) -> bool {
    record.seq >= delayed.created_at && !delayed.created_in.is_some_and(|batch| record.stamp.batch == Some(batch))
}

/// A delayed trigger's match as the queue's entry, its sequence number still
/// to be given. It carries the entry's X (CR 107.3n) and, once the source has
/// left, the frame the source left with (CR 113.7a).
fn pending_of(m: MatchedTrigger, delayed: &DelayedTrigger) -> PendingTrigger {
    PendingTrigger {
        seq: TriggerSeq(0),
        origin: TriggerOrigin::Delayed(m.identity),
        controller: m.controller,
        source_card: m.source_card,
        instances: m.instances,
        binding: TriggerBinding {
            def: m.def,
            records: m.records,
            event: m.event,
            subject: m.subject,
            triggered_by: None,
        },
        is_state_trigger: false,
        departed: delayed
            .source_frame
            .iter()
            .map(|frame| DepartedFrame { object: delayed.source, frame: Arc::clone(frame) })
            .collect(),
        x_value: delayed.x_value,
    }
}
