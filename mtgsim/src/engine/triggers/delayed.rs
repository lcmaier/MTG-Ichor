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
//!
//! **A reflexive trigger reads the other side** (CR 603.12): what the
//! resolution creating it performed before it, checked as it is created,
//! through the same door and the same matcher. It never waits.

use std::sync::Arc;

use super::dispatch::{TriggerReferents, DelayedMatch, MatchedTrigger, Refusal, ThisObject};
use super::history::TurnOrdinals;
use crate::types::triggers::ReflexiveForm;
use crate::engine::actions::ActionContext;
use crate::engine::layers::condition::settled_holds;
use crate::engine::trace_records;
use crate::events::event::{EventRecord, EventSeq, GameEvent};
use crate::state::game_state::GameState;
use crate::types::ids::{DelayedTriggerId, ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::{
    DelayedDuration, DelayedProvenance, DelayedTrigger, DelayedTriggerTemplate, DepartedFrame, EventIndex, EventKindMask,
    Multiplicity, PendingTrigger, Referred, RememberedObject, TriggerBinding, TriggerLimit, TriggerOrigin, TriggerSeq,
    TriggerTurn,
};
use crate::types::zones::UntilReturn;
use crate::ui::ask::ask_choose_delayed_trigger_event;
use crate::ui::choice_types::ChoiceOption;

impl GameState {
    /// The registry's one door (CR 603.7a). The caller supplies the
    /// provenance its rule gives: a resolution reads CR 603.7d–f's off its
    /// context, and a special action a static ability allows would read
    /// 603.7g's off that ability's object (`backlog.md` §2.8). `referred` is
    /// what its "that card" means (CR 603.7c), the creating resolution's
    /// `Effect::Remember`. A reflexive one (CR 603.12) is checked here against
    /// what its resolution has performed so far, and never waits. Returns the
    /// new entry's id. Announced first, so its creation is no record it reads.
    pub fn register_delayed_trigger(
        &mut self,
        template: &DelayedTriggerTemplate,
        provenance: DelayedProvenance,
        referred: Referred,
    ) -> Result<DelayedTriggerId, String> {
        let earlier: Vec<EventSeq> = match template.duration {
            DelayedDuration::Reflexive(_) => {
                let resolution = provenance.resolution.ok_or_else(|| {
                    format!(
                        "a reflexive trigger on {} has no resolution creating it, and CR 603.12 makes one only as a spell or ability resolves",
                        provenance.source.id
                    )
                })?;
                self.events.resolution_records(resolution.stamp, resolution.began_at).map(|r| r.seq).collect()
            }
            DelayedDuration::Once | DelayedDuration::ThisTurn => Vec::new(),
        };
        let id = DelayedTriggerId(self.next_delayed_trigger_id);
        self.next_delayed_trigger_id += 1;
        let owner = self.owner_now_or(provenance.source.id, provenance.controller);
        self.emit_event(GameEvent::DelayedTriggerCreated {
            id,
            source: provenance.source.id,
            controller: provenance.controller,
            rules_text: template.rules_text,
            duration: template.duration,
        });
        let delayed = DelayedTrigger {
            id,
            def: Arc::clone(&template.def),
            source: provenance.source,
            source_left_at: None,
            source_frame: None,
            source_card: provenance.source_card,
            controller: provenance.controller,
            owner,
            created_at: self.events.next_seq(),
            created_in: self.events.current_stamp().batch,
            created_by: provenance.created_by,
            duration: template.duration,
            x_value: provenance.x_value,
            turn: provenance.turn,
            instances: template.def.effect.instances(),
            rules_text: template.rules_text,
            referred,
        };
        match template.duration {
            DelayedDuration::Reflexive(form) => self.trigger_reflexive(&delayed, form, &earlier),
            DelayedDuration::Once | DelayedDuration::ThisTurn => self.delayed_triggers.push(delayed),
        }
        Ok(id)
    }

    /// CR 603.12 — a reflexive trigger, checked as it is created against the
    /// records its resolution performed before it (`earlier`): one trigger
    /// for each that matches (603.12a), one for them all where its event is
    /// "one or more", or for "when you don't" one if none does. Queued and
    /// announced as a dispatch queues a delayed trigger.
    fn trigger_reflexive(&mut self, reflexive: &DelayedTrigger, form: ReflexiveForm, earlier: &[EventSeq]) {
        // A record's place in its turn is a dispatch's to count, and no
        // reflexive trigger reads "the first time each turn".
        let occurrences = self.occurrences_among(reflexive, earlier, &TurnOrdinals::none(), |_| true);
        let triggered: Vec<PendingTrigger> = match form {
            ReflexiveForm::Does => occurrences.into_iter().map(|m| pending_of(m, reflexive)).collect(),
            ReflexiveForm::Doesnt => self.when_not_done(reflexive, occurrences.is_empty()).into_iter().collect(),
        };
        let mut queued = Vec::new();
        for pending in triggered {
            self.queue_pending(pending, &mut queued);
        }
        for (seq, origin, controller, caused_by) in queued {
            self.emit_event_unstamped(GameEvent::AbilityTriggered { seq, origin, controller, caused_by });
        }
    }

    /// "When you don't": the one trigger, bound to no record, when the action
    /// was not taken and its intervening "if" holds (CR 603.4).
    fn when_not_done(&self, reflexive: &DelayedTrigger, not_done: bool) -> Option<PendingTrigger> {
        if !not_done {
            return None;
        }
        if let Some(condition) = &reflexive.def.intervening_if
            && !settled_holds(condition, self, reflexive.source.id, Some(reflexive.controller))
        {
            return None;
        }
        let unbound = MatchedTrigger {
            identity: reflexive.identity(),
            controller: reflexive.controller,
            def: Arc::clone(&reflexive.def),
            source_card: Arc::clone(&reflexive.source_card),
            instances: reflexive.instances.clone(),
            event: EventIndex(0),
            records: Vec::new(),
            subject: None,
            mana: false,
        };
        Some(pending_of(unbound, reflexive))
    }

    /// The source's owner as an entry naming it is made (CR 108.3), read now
    /// because the store can lose the source before the entry reads it (item
    /// 235). `fallback`, the entry's controller, stands in for a source
    /// already gone: a token that ceased to exist before its ability resolved
    /// (CR 704.5d), whose controller is its owner unless control changed, or
    /// an object whose owner has left the game (CR 800.4a). Neither is exact
    /// until last known information carries the owner
    /// (`triggers-architecture.md` §3.11, TR-4a).
    pub(crate) fn owner_now_or(&self, id: ObjectId, fallback: PlayerId) -> PlayerId {
        self.objects.get(&id).map_or(fallback, |o| o.owner)
    }

    /// CR 400.7: object `id` is about to move, and the next record is that
    /// move. A delayed trigger whose source it is, or that refers to it,
    /// keeps that record's number, which is how "when this creature leaves
    /// the battlefield" and "when that token dies" know their own object's
    /// move. The callers are the movers whose record follows the move at
    /// once: `perform_zone_change` and a player leaving (CR 800.4a).
    pub(crate) fn note_delayed_source_moving(&mut self, id: ObjectId) {
        if self.delayed_triggers.is_empty() && self.until_returns.is_empty() {
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
            for referred in delayed.referred.objects.iter_mut() {
                if referred.object == object && referred.left_at.is_none() {
                    referred.left_at = Some(at);
                }
            }
        }
        // And the objects CR 610.3's "until" events name.
        for until in self.until_returns.iter_mut() {
            for remembered in std::iter::once(&mut until.source).chain(until.referred.iter_mut()) {
                if remembered.object == object && remembered.left_at.is_none() {
                    remembered.left_at = Some(at);
                }
            }
        }
    }

    /// CR 610.3 — take off `until_returns` every return whose event happened
    /// in `window`, oldest first.
    pub(crate) fn take_returns_due(&mut self, window: &[EventSeq]) -> Vec<UntilReturn> {
        if self.until_returns.is_empty() {
            return Vec::new();
        }
        let (due, waiting): (Vec<UntilReturn>, Vec<UntilReturn>) =
            std::mem::take(&mut self.until_returns).into_iter().partition(|until| self.is_due(until, window));
        self.until_returns = waiting;
        due
    }

    /// Whether `window` holds the event `until` waits for. The event is
    /// matched the way a trigger condition is: "this" is the source of the
    /// exiling ability, and "that" is the target the "until" names (Calix's
    /// enchantment), both by identity. An event recorded before the return
    /// was made does not count.
    fn is_due(&self, until: &UntilReturn, window: &[EventSeq]) -> bool {
        let referents = TriggerReferents {
            this: ThisObject::Remembered(until.source),
            controller: until.controller,
            owner: until.owner,
            host: None,
            this_ability: None,
            referred: &until.referred,
        };
        window.iter().filter_map(|seq| self.events.record(*seq)).any(|record| {
            record.seq >= until.created_at
                && until.until.reads(&record.event)
                && !self.occurrences_matching_arm(&until.until, &referents, record.seq, &record.event).is_empty()
        })
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
    /// window order, of the records it existed before (CR 603.7a).
    fn delayed_occurrences(
        &self,
        delayed: &DelayedTrigger,
        window: &[EventSeq],
        ordinals: &TurnOrdinals,
    ) -> Vec<MatchedTrigger> {
        self.occurrences_among(delayed, window, ordinals, |record| existed_before(delayed, record))
    }

    /// Every occurrence that triggers an entry among the records of `window`
    /// it `reads`, in window order. A record counts if one of the entry's
    /// arms reads its kind; it is matched the way an object's ability is
    /// (`match_delayed`), the verdict is traced, and a match adds its
    /// occurrences (`add_occurrences`).
    fn occurrences_among(
        &self,
        delayed: &DelayedTrigger,
        window: &[EventSeq],
        ordinals: &TurnOrdinals,
        reads: impl Fn(&EventRecord) -> bool,
    ) -> Vec<MatchedTrigger> {
        let referents = self.delayed_referents(delayed);
        let arms = delayed.def.condition.events();
        let mut occurrences: Vec<MatchedTrigger> = Vec::new();
        for seq in window {
            let Some(record) = self.events.record(*seq) else { continue };
            if !reads(record) || !arms.iter().any(|arm| arm.reads(&record.event)) {
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
    /// source's owner as it was made, "this ability" the one that created it
    /// (CR 603.7h), and "that token" what it refers to.
    fn delayed_referents<'a>(&self, delayed: &'a DelayedTrigger) -> TriggerReferents<'a> {
        TriggerReferents {
            this: ThisObject::Remembered(RememberedObject { object: delayed.source, left_at: delayed.source_left_at }),
            controller: delayed.controller,
            owner: delayed.owner,
            host: None,
            this_ability: delayed.created_by,
            referred: &delayed.referred.objects,
        }
    }

    /// The entry against one record, as `match_def` is for an object's
    /// ability: the first arm that matches and its occurrences' subjects,
    /// then the def's once-per-turn limit and its intervening "if". `Err`
    /// names the predicate that refused.
    fn match_delayed(
        &self,
        delayed: &DelayedTrigger,
        referents: &TriggerReferents<'_>,
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
                // Never in the registry: checked as it was created.
                DelayedDuration::Reflexive(_) => Vec::new(),
            };
            for pending in triggered {
                self.queue_pending(pending, queued);
            }
        }
    }

    /// One delayed trigger onto the queue, its sequence number given, and
    /// its `AbilityTriggered` to announce added to `queued` — unless "triggers
    /// only once each turn" has already had its turn.
    fn queue_pending(
        &mut self,
        mut pending: PendingTrigger,
        queued: &mut Vec<(TriggerSeq, TriggerOrigin, PlayerId, EventSeq)>,
    ) {
        let identity = pending.origin.identity();
        if pending.binding.def.limit == Some(TriggerLimit::TriggersOnlyOnceEachTurn)
            && !self.triggered_this_turn.insert(identity)
        {
            return;
        }
        pending.seq = TriggerSeq(self.next_trigger_seq);
        self.next_trigger_seq += 1;
        // "When you don't" is caused by no record.
        let caused_by = pending.binding.records.first().map_or(EventSeq(0), |r| r.seq);
        queued.push((pending.seq, pending.origin, pending.controller, caused_by));
        self.pending_triggers.push(pending);
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
            referred: delayed.referred.clone(),
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::cards::authoring::whenever;
    use crate::cards::creatures::grizzly_bears;
    use crate::engine::actions::GameAction;
    use crate::test_support::{put_on_battlefield_under, setup_two_player_game, test_ctx};
    use crate::types::effects::{Effect, PlayerRef};
    use crate::types::triggers::{
        DelayedDuration, DelayedProvenance, DelayedTriggerTemplate, DelayedTurn, Multiplicity, Referred,
        TriggerEvent, TriggerTurn,
    };

    /// CR 108.3: a delayed trigger's "its owner" is its source's owner, fixed
    /// as the trigger is made, and stays so once the store has lost the source
    /// (item 235). The source is P0's under P1's control, so a fallback to the
    /// controller watches the wrong player.
    #[test]
    fn its_owner_is_the_sources_after_the_source_is_gone() {
        let mut game = setup_two_player_game();
        let card = grizzly_bears();
        let source = put_on_battlefield_under(&mut game, Arc::clone(&card), 0, 1);
        let owner_gains = TriggerEvent::GainsLife { player: Some(PlayerRef::Owner), multiplicity: Multiplicity::PerOccurrence };
        let template = DelayedTriggerTemplate {
            def: Arc::new(whenever(owner_gains, Effect::Sequence(Vec::new()))),
            duration: DelayedDuration::Once,
            turn: DelayedTurn::Any,
            rules_text: "When its owner next gains life, nothing.".into(),
        };
        let provenance = DelayedProvenance {
            source: game.object_ref(source).unwrap(),
            source_card: card,
            controller: 1,
            created_by: None,
            x_value: None,
            turn: TriggerTurn::Any,
            resolution: None,
        };
        game.register_delayed_trigger(&template, provenance, Referred::default()).unwrap();
        game.remove_from_game(source).unwrap();

        game.execute_action(GameAction::GainLife { player: 0, amount: 1, source }, &test_ctx()).unwrap();
        assert_eq!(game.pending_triggers.len(), 1, "the owner, P0, gained life");
        assert!(game.delayed_triggers.is_empty());
    }
}
