//! CR 603.8's state check (`triggers-architecture.md` §4.5): the triggered
//! abilities that trigger when a game state is true rather than when an event
//! occurs.
//!
//! No record carries a state, so the dispatcher never finds one. This check
//! is their asker, at the three moments a state can have changed: after every
//! dispatch, which is what catches a state true for a moment in
//! the middle of a resolution (Emperor Crocodile's first ruling); on every
//! pass of CR 117.5's loop before placement, which sees what changed with no
//! record, a continuous effect ending among them; and at the cleanup step's
//! CR 514.3a probe.
//!
//! It finds its candidates through the dispatcher's own gate, all three legs
//! of it (printed, granted, copied), since a state trigger's source is filed
//! under `EventKind::BoardState`. Its condition is the evaluator every "as long
//! as" and intervening "if" shares, read against the board as it is.

use std::sync::Arc;

use super::dispatch::visible_to_all;
use super::audit::assert_agree;
use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::condition::{settled_holds, ConditionReferents};
use crate::engine::trace_records;
use crate::engine::zone_function::functions_in;
use crate::events::cause::{EventCause, GameRule};
use crate::events::event::GameEvent;
use crate::objects::card_data::CardData;
use crate::oracle::characteristics::controller_or_owner;
use crate::state::game_state::{AbilityIdentity, GameState};
use crate::types::effects::{Effect, EffectRecipient};
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::{
    EventIndex, EventKind, EventKindMask, PendingTrigger, Referred, TriggerBinding, TriggerCondition, TriggerDef, TriggerLimit,
    TriggerOrigin, TriggerSeq,
};
use crate::types::zones::ZoneSet;

/// A state trigger the check found true, before it is queued.
pub(super) struct StateTriggerMatch {
    identity: AbilityIdentity,
    controller: PlayerId,
    def: Arc<TriggerDef>,
    source_card: Arc<CardData>,
    instances: Vec<EffectRecipient>,
}

impl GameState {
    /// CR 603.8 — every state trigger whose condition the board matches now
    /// triggers, unless it already has and has not yet left the stack. Each
    /// is queued as a dispatch queues a trigger, and announced by
    /// `AbilityTriggered` for CR 603.3b's second tier, caused by no record.
    pub(crate) fn check_state_triggers(&mut self) -> Result<(), String> {
        // CR 104.1 — a game that has ended queues nothing.
        if self.result.is_some() {
            return Ok(());
        }
        let found: Vec<StateTriggerMatch> =
            self.state_trigger_candidates().into_iter().flat_map(|id| self.state_triggers_of(id)).collect();
        if self.dispatch_audit.is_some() {
            self.audit_state_check(&found);
        }
        let mut queued: Vec<(TriggerSeq, TriggerOrigin, PlayerId)> = Vec::new();
        for m in found {
            // "Triggers only once each turn", written as it queues, as the
            // dispatcher writes it.
            if m.def.limit == Some(TriggerLimit::TriggersOnlyOnceEachTurn) && !self.history.triggered_this_turn.insert(m.identity) {
                continue;
            }
            let seq = TriggerSeq(self.next_trigger_seq);
            self.next_trigger_seq += 1;
            let origin = TriggerOrigin::Object(m.identity);
            self.pending_triggers.push(PendingTrigger {
                seq,
                origin,
                controller: m.controller,
                source_card: m.source_card,
                instances: m.instances,
                // No event triggered it, so it binds none (`TriggerBinding::event`).
                binding: TriggerBinding {
                    def: m.def,
                    triggering_events: Vec::new(),
                    event: EventIndex(0),
                    triggered_by: None,
                    referred: Referred::default(),
                    source_found_in_graveyard: None,
                },
                kept_lki: Vec::new(),
                x_value: None,
            });
            queued.push((seq, origin, m.controller));
        }
        for (seq, origin, controller) in queued {
            self.emit_event_unbatched(
                GameEvent::AbilityTriggered { seq, origin, controller, caused_by: None },
                EventCause::GameRule(GameRule::AbilityTriggers),
            )?;
        }
        Ok(())
    }

    /// The objects that may carry a state trigger, off the dispatcher's gate:
    /// the printed sources filed under `BoardState`, on the battlefield and
    /// off it, the objects a row naming them puts one on, and the zones a
    /// `Filter` row does. In CR 613.7 order, as the dispatcher's candidates.
    fn state_trigger_candidates(&self) -> Vec<ObjectId> {
        let board_state = EventKindMask::of(EventKind::BoardState);
        let summary = self.continuous_effects.summary();
        let readers = self.battlefield_readers(board_state);
        let named = self.named_trigger_carriers(board_state);
        let zones = match summary.unattributed_trigger_kinds.contains(EventKind::BoardState) {
            true => summary.unattributed_trigger_zones,
            false => ZoneSet::EMPTY,
        };
        // ORDER-FREE: `candidates_now` sorts the zone map's objects by timestamp.
        let mut elsewhere = self.zone_trigger_sources.iter().filter(|(_, kinds)| kinds.contains(EventKind::BoardState)).map(|(&id, _)| id).peekable();
        if readers.is_empty() && named.is_empty() && zones.is_empty() && elsewhere.peek().is_none() {
            return Vec::new();
        }
        self.candidates_now(readers, named, Vec::new(), zones, elsewhere)
    }

    /// Each of `id`'s state triggers that triggers now: one that functions
    /// where the object is (CR 113.6), is not waiting or on the stack already,
    /// and whose condition holds. Read-only, so the audit can ask it of every
    /// object. Empty, and nothing allocated, for an object with none.
    pub(super) fn state_triggers_of(&self, id: ObjectId) -> Vec<StateTriggerMatch> {
        let mut found = Vec::new();
        let Some(object) = self.objects.get(&id) else { return found };
        // CR 603.2f — a hidden object's abilities trigger on nothing.
        if !visible_to_all(self, id) {
            return found;
        }
        let Some(chars) = compute_characteristics(self, id) else { return found };
        let existence = ObjectRef { id, zone_change_epoch: object.zone_change_epoch };
        let controller = controller_or_owner(self, id).unwrap_or(object.owner);
        for ability in chars.abilities.iter() {
            let Effect::Triggered(def) = &ability.effect else { continue };
            let TriggerCondition::State(condition) = &def.condition else { continue };
            if !functions_in(ability, &chars.types, object.zone) {
                continue;
            }
            let identity = AbilityIdentity { source: existence, ability: ability.id };
            let refused_by = if def.intervening_if.is_some() {
                // A state trigger checks its state at the trigger only
                // (Emperor Crocodile's second ruling), so it has no "if".
                Some("intervening_if")
            } else if def.limit == Some(TriggerLimit::FirstTimeEachTurn) {
                // "For the first time each turn" is about an event.
                Some("limit")
            } else if self.state_trigger_has_not_left_the_stack(identity) {
                Some("not_left_the_stack")
            } else if !settled_holds(condition, self, id, None, ConditionReferents::NONE) {
                Some("condition")
            } else {
                None
            };
            self.trace(|| trace_records::state_trigger(self, &identity, object.zone, refused_by));
            if refused_by.is_none() {
                found.push(StateTriggerMatch {
                    identity,
                    controller,
                    def: Arc::clone(def),
                    source_card: Arc::clone(&object.card_data),
                    instances: ability.instances.clone(),
                });
            }
        }
        found
    }

    /// CR 603.8: "A state-triggered ability doesn't trigger again until the
    /// ability has resolved, has been countered, or has otherwise left the
    /// stack" — so until then it is waiting to be put on the stack, or on it.
    /// Read off the queue and the stack, which say exactly that, so nothing
    /// re-arms it: its leaving, however it leaves, is the answer changing.
    fn state_trigger_has_not_left_the_stack(&self, identity: AbilityIdentity) -> bool {
        let this = TriggerOrigin::Object(identity);
        self.pending_triggers.iter().any(|pending| pending.origin == this)
            || self.stack.iter().any(|id| {
                self.stack_entries.get(id).is_some_and(|entry| entry.trigger.is_some() && entry.ability_identity == Some(identity))
            })
    }

    /// The state check asked again of every object that may trigger, with no
    /// gate (`audit.rs`), whose answer must be the gated one's.
    fn audit_state_check(&mut self, gated: &[StateTriggerMatch]) {
        let saved = self.save_observers();
        let reference: Vec<StateTriggerMatch> =
            self.objects_that_may_trigger().into_iter().flat_map(|id| self.state_triggers_of(id)).collect();
        self.restore_observers(saved);
        let describe = |matches: &[StateTriggerMatch]| -> Vec<String> {
            matches
                .iter()
                .map(|m| format!("{} {}", crate::ui::display::printed_name(self, m.identity.source.id), m.identity.ability))
                .collect()
        };
        assert_agree("the state check", describe(gated), describe(&reference));
    }
}
