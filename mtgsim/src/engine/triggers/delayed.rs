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
//! **A reflexive trigger reads the other side** (CR 603.12): what "that
//! action", taken by the resolution creating it, performed before it,
//! checked as it is created, through the same door and the same matcher. It
//! never waits.

use std::sync::Arc;

use super::dispatch::{fold_trigger_occurrences, TriggerReferents, TriggerSource, DelayedMatch, MatchedTrigger, Refusal, ThisObject};
use crate::types::triggers::{CreatingResolution, ReflexiveForm};
use crate::engine::actions::ActionContext;
use super::binding::TriggeringEventView;
use crate::engine::trace_records;
use crate::events::event::{CreatedAt, EventRecord, EventId, GameEvent};
use crate::events::unasked::{Elision, Unasked, UnaskedAnswer};
use crate::state::game_state::GameState;
use crate::types::ids::{DelayedTriggerId, ExtraTurnId, ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::{
    DelayedDuration, DelayedProvenance, DelayedTrigger, DelayedTriggerTemplate, EventIndex, EventKindMask,
    PendingTrigger, Referred, RememberedObject, TriggerBinding, TriggerOrigin, TriggerId,
    TriggeringEvent, TriggeringEventObjects,
    TriggerTurn,
};
use crate::types::zones::UntilReturn;
use crate::ui::ask::ask_choose_delayed_trigger_event;
use crate::ui::choice_types::{ChoiceKind, ChoiceOption};

impl GameState {
    /// CR 603.7a — a delayed triggered ability is created, through the one
    /// door every rule that creates one uses. The caller supplies the
    /// provenance its rule gives: a resolution reads CR 603.7d–f's off its
        /// context, and an action a static ability allows (CR 103.6's
    /// opening-hand reveal) would read 603.7g's off that ability's object. `referred` is
    /// what its "that card" means (CR 603.7c), the creating resolution's
    /// `Effect::Remember`. Announced first, so its creation is no record it
    /// reads. Then one of two things, by its duration:
    /// - **Once or this turn:** it waits on the registry for an event after
    ///   this one (603.7a), which a dispatch asks it about.
    /// - **Reflexive** (603.12): it asks about "that action", which its
    ///   resolution has already taken, now, and never waits
    ///   (`trigger_reflexive`).
    ///
    /// Returns its number.
    pub fn create_delayed_trigger(
        &mut self,
        template: &DelayedTriggerTemplate,
        provenance: DelayedProvenance,
        referred: Referred,
    ) -> Result<DelayedTriggerId, String> {
        let reflexive = match template.duration {
            DelayedDuration::Reflexive(form) => Some((
                form,
                provenance.resolution.ok_or_else(|| {
                    format!(
                        "a reflexive trigger on {} has no resolution creating it, and CR 603.12 makes one only as a spell or ability resolves",
                        provenance.source.id
                    )
                })?,
            )),
            DelayedDuration::Once | DelayedDuration::ThisTurn => None,
        };
        let id = DelayedTriggerId(self.next_delayed_trigger_id);
        self.next_delayed_trigger_id += 1;
        let owner = self.owner_of_source(provenance.source)?;
        let created = GameEvent::DelayedTriggerCreated {
            id,
            source: provenance.source,
            controller: provenance.controller,
            rules_text: template.rules_text,
            duration: template.duration,
        };
        self.emit_event(created, provenance.cause)?;
        let delayed = DelayedTrigger {
            id,
            def: Arc::clone(&template.def),
            source: provenance.source,
            source_card: provenance.source_card,
            controller: provenance.controller,
            owner,
            created_at: CreatedAt::now(&self.events),
            created_by: provenance.created_by,
            duration: template.duration,
            x_value: provenance.x_value,
            turn: provenance.turn,
            instances: template.def.effect.instances(),
            rules_text: template.rules_text,
            referred,
        };
        match reflexive {
            Some((form, resolution)) => self.trigger_reflexive(&delayed, form, resolution)?,
            None => self.delayed_triggers.push(delayed),
        }
        Ok(id)
    }

    /// CR 603.12 — a reflexive trigger, checked as it is created against
    /// "that action": the records its resolution performed taking it, which
    /// came before the trigger, so they are what it reads and no later event
    /// ever is. One trigger for each record that matches (603.12a), one for
    /// them all where its event is "one or more", or for "when you don't" one
    /// if none does. Queued and announced as a dispatch queues a delayed
    /// trigger.
    fn trigger_reflexive(&mut self, reflexive: &DelayedTrigger, form: ReflexiveForm, resolution: CreatingResolution) -> Result<(), String> {
        let that_action: Vec<EventId> = match resolution.that_action {
            Some((from, to)) => {
                self.events.resolution_records(resolution.stamp, from).filter(|r| r.seq < to).map(|r| r.seq).collect()
            }
            None => Vec::new(),
        };
        let occurrences = self.occurrences_among(reflexive, &that_action, |_| true);
        let triggered: Vec<PendingTrigger> = match form {
            ReflexiveForm::Does => occurrences.into_iter().map(|m| pending_of(m, reflexive)).collect(),
            ReflexiveForm::Doesnt => self.when_not_done(reflexive, occurrences.is_empty()).into_iter().collect(),
        };
        let mut announce = Vec::new();
        for pending in triggered {
            self.queue_pending(pending, &mut announce);
        }
        self.announce_triggered(announce)
    }

    /// "When you don't": the one trigger, bound to no record, when the action
    /// was not taken and its intervening "if" holds (CR 603.4).
    fn when_not_done(&self, reflexive: &DelayedTrigger, not_done: bool) -> Option<PendingTrigger> {
        if !not_done {
            return None;
        }
        let source = TriggerSource { object: reflexive.source.id, controller: reflexive.controller, lki: self.lki(reflexive.source) };
        if reflexive.def.intervening_if.as_ref().is_some_and(|condition| !self.intervening_if_holds(condition, source, TriggeringEventView::None)) {
            return None;
        }
        Some(pending_of(matched_of(reflexive, EventIndex(0), Vec::new()), reflexive))
    }

    /// The source's owner as an entry naming it is made (CR 108.3), kept on
    /// the entry because the store can lose the source unframed before the
    /// entry reads it: a return is no keeper, and a card that leaves the game
    /// from a graveyard with its owner leaves no last known information. No
    /// owner now is a missed capture.
    pub(crate) fn owner_of_source(&self, source: ObjectRef) -> Result<PlayerId, String> {
        self.owner_of(source.id).ok_or_else(|| format!("{} is gone, and nothing kept its last known information to say who owned it", source.id))
    }


    /// CR 610.3 — take off `until_returns` every return whose event happened
    /// in `window`, oldest first.
    pub(crate) fn take_returns_due(&mut self, window: &[EventId]) -> Vec<UntilReturn> {
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
    fn is_due(&self, until: &UntilReturn, window: &[EventId]) -> bool {
        let referents = TriggerReferents {
            this: ThisObject::Remembered(until.source),
            controller: until.controller,
            owner: until.owner,
            host: None,
            this_ability: None,
            referred: &until.referred,
        };
        window.iter().filter_map(|seq| self.events.record(*seq)).any(|record| {
            until.created_at.is_before(record) && !self.triggering_events_in(&until.until, &referents, record).is_empty()
        })
    }

    /// CR 514.2: the cleanup step ends "this turn" effects, and a delayed
    /// triggered ability with that stated duration ends with them.
    pub(crate) fn end_this_turn_delayed_triggers(&mut self) {
        self.delayed_triggers.retain(|d| d.duration != DelayedDuration::ThisTurn);
    }

    /// A turn has begun. An entry waiting for an extra turn that is neither
    /// this turn nor still coming can never trigger: that turn has ended, or
    /// it was skipped, and CR 614.10a says anything scheduled for a skipped
    /// turn won't happen.
    pub(crate) fn drop_delayed_triggers_of_gone_turns(&mut self) {
        let gone: Vec<ExtraTurnId> = self
            .delayed_triggers
            .iter()
            .filter_map(|d| match d.turn {
                TriggerTurn::Extra(turn) if !self.extra_turn_current_or_queued(turn) => Some(turn),
                TriggerTurn::Extra(_) | TriggerTurn::Any | TriggerTurn::LaterThan(_) => None,
            })
            .collect();
        if !gone.is_empty() {
            self.delayed_triggers.retain(|d| !matches!(d.turn, TriggerTurn::Extra(turn) if gone.contains(&turn)));
        }
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
        window: &[EventId],
        window_kinds: EventKindMask,
    ) -> Vec<DelayedMatch> {
        if self.delayed_triggers.is_empty() {
            return Vec::new();
        }
        let mut found = Vec::new();
        for delayed in &self.delayed_triggers {
            if !delayed.def.record_kinds().intersects(window_kinds) || !self.in_turn(delayed.turn) {
                continue;
            }
            let occurrences = self.delayed_occurrences(delayed, window);
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
        window: &[EventId],
    ) -> Vec<MatchedTrigger> {
        self.occurrences_among(delayed, window, |record| delayed.created_at.is_before(record))
    }

    /// Every occurrence that triggers an entry among the records of `window`
    /// it `reads`, in window order. A record counts if one of the entry's
    /// arms reads its kind; it is matched the way an object's ability is
    /// (`match_delayed`), the verdict is traced, and a match is folded as an
    /// object's is (CR 603.2c).
    fn occurrences_among(
        &self,
        delayed: &DelayedTrigger,
        window: &[EventId],
        reads: impl Fn(&EventRecord) -> bool,
    ) -> Vec<MatchedTrigger> {
        let referents = self.delayed_referents(delayed);
        let arms = delayed.def.condition.events();
        let mut matches: Vec<MatchedTrigger> = Vec::new();
        for seq in window {
            let Some(record) = self.events.record(*seq) else { continue };
            if !reads(record) || !arms.iter().any(|arm| arm.reads(&record.event)) {
                continue;
            }
            let verdict = self.match_delayed(delayed, &referents, record);
            self.trace_delayed_verdict(delayed, record.seq, verdict.as_ref().err().copied());
            if let Ok((event, triggering_events)) = verdict {
                let arm = &arms[event.0];
                fold_trigger_occurrences(&mut matches, delayed.identity(), arm, event, record, triggering_events, |triggering_events| {
                    matched_of(delayed, event, triggering_events)
                });
            }
        }
        matches
    }

    /// What an entry's words for itself mean (CR 603.7c–g): "this object" is
    /// its source as remembered, "you" its controller, "its owner" the
    /// source's owner as it was made, "this ability" the one that created it
    /// (CR 603.7h), and "that token" what it refers to.
    fn delayed_referents<'a>(&self, delayed: &'a DelayedTrigger) -> TriggerReferents<'a> {
        TriggerReferents {
            this: ThisObject::Remembered(RememberedObject::now(delayed.source)),
            controller: delayed.controller,
            owner: delayed.owner,
            host: None,
            this_ability: delayed.created_by,
            referred: &delayed.referred.objects,
        }
    }

    /// The entry against one record, as `match_def` is for an object's
    /// ability, every arm asked. The intervening "if"'s "you" is its
    /// controller (CR 109.5), and its source is read as it last existed once
    /// it has left (CR 113.7a, 400.7).
    fn match_delayed(
        &self,
        delayed: &DelayedTrigger,
        referents: &TriggerReferents<'_>,
        record: &EventRecord,
    ) -> Result<(EventIndex, Vec<TriggeringEventObjects>), Refusal> {
        let source = TriggerSource { object: delayed.source.id, controller: delayed.controller, lki: self.lki(delayed.source) };
        // A delayed trigger states no zone: it has no source to function in.
        self.first_arm_that_triggers(&delayed.def, None, delayed.identity(), referents, record, source, |_| true)
    }

    /// The trace's `trigger` record of an entry's verdict on record `seq`,
    /// under the zone its source is in; none once the store has no source.
    fn trace_delayed_verdict(&self, delayed: &DelayedTrigger, seq: EventId, refusal: Option<Refusal>) {
        let Some(zone) = self.objects.get(&delayed.source.id).map(|o| o.zone) else { return };
        self.trace(|| {
            trace_records::trigger(self, seq, &delayed.identity(), zone, refusal.is_none(), refusal.map(Refusal::name), false)
        });
    }


    /// Queue the registry's matches, after the objects' (§4.6). A `ThisTurn`
    /// entry triggers on every occurrence and stays. A `Once` entry triggers
    /// once and leaves as it queues (CR 603.7b), its controller choosing
    /// among simultaneous occurrences.
    pub(super) fn queue_delayed(&mut self, matched: Vec<DelayedMatch>, ctx: Option<&ActionContext>, announce: &mut Vec<GameEvent>) {
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
                self.queue_pending(pending, announce);
            }
        }
    }

    /// One delayed trigger onto the queue, numbered, unless "triggers only
    /// once each turn" has already had its turn.
    fn queue_pending(&mut self, pending: PendingTrigger, announce: &mut Vec<GameEvent>) {
        if !self.may_trigger_this_turn(&pending.binding.def, pending.origin.identity()) {
            return;
        }
        let caused_by = pending.binding.first_record().map(|r| r.seq);
        let pending = PendingTrigger { id: self.take_next_trigger_id(), ..pending };
        self.queue_trigger(pending, caused_by, announce);
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
        let reads = entries.first()?.binding.def.trigger_binding_reads();
        let several = entries.len() >= 2;
        let mut distinct: Vec<PendingTrigger> = Vec::new();
        for entry in entries {
            if !distinct.iter().any(|kept| self.entries_agree_on(reads, kept, &entry)) {
                distinct.push(entry);
            }
        }
        if distinct.len() < 2 {
            let kept = distinct.pop()?;
            if several {
                self.note_unasked(kept.controller, Unasked::OneGame(Elision::DelayedTriggerCause), || {
                    let cause = match kept.binding.triggering_event().and_then(|t| t.objects.objects().next()) {
                        Some((_, object)) => ChoiceOption::Object(object.id),
                        None => ChoiceOption::Number(kept.binding.first_record().map_or(0, |r| r.seq.0 as u64)),
                    };
                    (ChoiceKind::ChooseDelayedTriggerEvent { source }, UnaskedAnswer::Picks(vec![cause]))
                });
            }
            return Some(kept);
        }
        let Some(ctx) = ctx else {
            // Two occurrences of one arm come only in a batch's window, a
            // declaration's record or one spell's targeting window, and each
            // dispatches with a provider.
            debug_assert!(false, "CR 603.7b's choice reached a dispatch with no provider to ask");
            return distinct.into_iter().next();
        };
        // Each cause by its object, or by its record where objects do not
        // tell the causes apart.
        let by_object: Vec<Option<ObjectId>> =
            distinct.iter().map(|e| e.binding.triggering_event().and_then(|t| t.objects.objects().next()).map(|(_, object)| object.id)).collect();
        let objects_tell = by_object.iter().all(Option::is_some)
            && by_object.iter().enumerate().all(|(i, a)| by_object[..i].iter().all(|b| b != a));
        let options: Vec<ChoiceOption> = distinct
            .iter()
            .zip(&by_object)
            .map(|(entry, object)| match object {
                Some(id) if objects_tell => ChoiceOption::Object(*id),
                _ => ChoiceOption::Number(entry.binding.first_record().map_or(0, |r| r.seq.0 as u64)),
            })
            .collect();
        let controller = distinct[0].controller;
        let chosen = ask_choose_delayed_trigger_event(ctx.dp, self, controller, source, &options);
        distinct.into_iter().nth(chosen)
    }
}

/// A delayed trigger's match on its arm `event`: never a mana ability, and
/// never counted again (CR 603.2d's last sentence).
fn matched_of(delayed: &DelayedTrigger, event: EventIndex, triggering_events: Vec<TriggeringEvent>) -> MatchedTrigger {
    MatchedTrigger {
        identity: delayed.identity(),
        controller: delayed.controller,
        def: Arc::clone(&delayed.def),
        stated_zones: None,
        source_card: Arc::clone(&delayed.source_card),
        instances: delayed.instances.clone(),
        event,
        triggering_events,
        mana: false,
        additional: 0,
    }
}

/// A delayed trigger's match as the queue's entry, its sequence number still
/// to be given. It carries the entry's X (CR 107.3n) and, once the source has
/// left, the frame the source left with (CR 113.7a).
fn pending_of(m: MatchedTrigger, delayed: &DelayedTrigger) -> PendingTrigger {
    PendingTrigger {
        id: TriggerId(0),
        origin: TriggerOrigin::Delayed(m.identity),
        controller: m.controller,
        source_card: m.source_card,
        instances: m.instances,
        binding: TriggerBinding {
            def: m.def,
            triggering_events: m.triggering_events,
            event: m.event,
            triggered_by: None,
            referred: delayed.referred.clone(),
            source_found_in_graveyard: None,
            stated_zones: None,
        },
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
    use crate::types::effects::{PlayerSet, Antecedent, Effect, PlayerRef};
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
        let owner_gains = TriggerEvent::GainsLife { player: PlayerSet::Player(PlayerRef::OwnerOf(Antecedent::ThisObject)), multiplicity: Multiplicity::PerOccurrence };
        let template = DelayedTriggerTemplate {
            def: Arc::new(whenever(owner_gains, Effect::Sequence(Vec::new()))),
            duration: DelayedDuration::Once,
            turn: DelayedTurn::NoneNamed,
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
            cause: crate::events::cause::EventCause::Harness,
        };
        game.create_delayed_trigger(&template, provenance, Referred::default()).unwrap();
        game.remove_from_game(source).unwrap();

        game.execute_action(GameAction::GainLife { player: 0, amount: 1, source }, &test_ctx()).unwrap();
        assert_eq!(game.pending_triggers.len(), 1, "the owner, P0, gained life");
        assert!(game.delayed_triggers.is_empty());
    }
}
