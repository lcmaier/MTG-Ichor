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

use super::dispatch::{visible_to_all, OffBattlefieldLegs};
use super::audit::assert_agree;
use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::condition::{settled_holds, ConditionReferents};
use crate::engine::trace_records;
use crate::engine::zone_function::functions_in;
use crate::objects::card_data::CardData;
use crate::oracle::characteristics::controller_or_owner;
use crate::state::game_state::{AbilityIdentity, GameState};
use crate::types::effects::EffectRecipient;
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::{
    EventIndex, EventKind, EventKindMask, PendingTrigger, Referred, TriggerBinding, TriggerCondition, TriggerDef, TriggerLimit,
    TriggerOrigin,
};

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
        let mut announce = Vec::new();
        for m in found {
            if !self.may_trigger_this_turn(&m.def, m.identity) {
                continue;
            }
            let pending = PendingTrigger {
                seq: self.take_next_trigger_id(),
                origin: TriggerOrigin::Object(m.identity),
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
                x_value: None,
            };
            self.queue_trigger(pending, None, &mut announce);
        }
        self.announce_triggered(announce)
    }

    /// The objects that may carry a state trigger, off the dispatcher's gate:
    /// the printed sources filed under `BoardState`, on the battlefield and
    /// off it, the objects a row naming them puts one on, and the zones a
    /// `Filter` row does. In CR 613.7 order, as the dispatcher's candidates.
    fn state_trigger_candidates(&self) -> Vec<ObjectId> {
        let gate = self.trigger_gate(EventKindMask::of(EventKind::BoardState), Vec::new(), OffBattlefieldLegs::FilteredByKind);
        if gate.is_empty() {
            return Vec::new();
        }
        gate.candidates()
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
            let Some(def) = ability.effect.as_trigger() else { continue };
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
    /// stack" — so until then it is waiting to be put on the stack, on it, or
    /// resolving (CR 608.2n's last step is the leaving). Read off the keepers,
    /// which say exactly that, so nothing re-arms it: its leaving, however it
    /// leaves, is the answer changing.
    fn state_trigger_has_not_left_the_stack(&self, identity: AbilityIdentity) -> bool {
        self.lki_keepers().any(|keeper| keeper.is_triggered(identity))
    }

    /// The state check asked again of every object that may trigger, with no
    /// gate (`audit.rs`), whose answer must be the gated one's.
    fn audit_state_check(&mut self, gated: &[StateTriggerMatch]) {
        let reference: Vec<StateTriggerMatch> =
            self.unobserved(|| self.objects_that_may_trigger().into_iter().flat_map(|id| self.state_triggers_of(id)).collect());
        let describe = |matches: &[StateTriggerMatch]| -> Vec<String> {
            matches
                .iter()
                .map(|m| format!("{} {}", crate::ui::display::printed_name(self, m.identity.source.id), m.identity.ability))
                .collect()
        };
        assert_agree("the state check", describe(gated), describe(&reference));
    }
}
