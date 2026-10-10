//! The bound facts, read back at resolution (`triggers-architecture.md` §3.4, §6.3).
//!
//! A [`TriggerBinding`] holds the records it matched, copied at dispatch, so
//! every read here is of the binding's own records through the matched arm's
//! projection, and the frame comes with the record.

use std::sync::Arc;

use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::resolve::{ResolutionContext, ResolvedTarget};
use crate::events::event::GameEvent;
use crate::state::game_state::{GameState, ResolvingObject, StackEntry};
use crate::types::effects::EffectRecipient;
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::{DepartedFrame, TriggerBinding};

/// The subject as it was in the zone it left, when the matched event was its
/// own departure: the record's CR 603.10a frame, naming the existence that
/// left.
pub(crate) fn departure_frame(binding: &TriggerBinding) -> Option<&DepartedFrame> {
    let subject = binding.subject?;
    match &binding.records.first()?.event {
        GameEvent::ZoneChange { object_id, lki: Some(frame), .. }
        | GameEvent::LeftTheGame { object_id, lki: Some(frame), .. }
            if *object_id == subject.id =>
        {
            Some(frame)
        }
        _ => None,
    }
}

/// The last known information a stack object carries for `object` (CR
/// 113.7a, 608.2h), once `object` has left: its subject's departure record's,
/// when its trigger's event was that departure (CR 603.10a) and `object` is
/// that subject — which it names by the object it became — else what it kept
/// as `object` left later. `None` while `object` has not left. The one
/// reading of the rule, which an object on the stack and the object
/// resolving both ask through `lki_of`.
fn carried_lki<'a>(
    object: ObjectRef,
    subject: Option<ObjectRef>,
    subject_lki: Option<&'a DepartedFrame>,
    kept: &'a [DepartedFrame],
) -> Option<&'a DepartedFrame> {
    match subject_lki {
        Some(lki) if subject == Some(object) => Some(lki),
        _ => kept.iter().find(|frame| frame.object == object),
    }
}

impl StackEntry {
    /// The last known information it carries for `object` (`carried_lki`).
    pub fn lki_of(&self, object: ObjectRef) -> Option<&DepartedFrame> {
        let binding = self.trigger.as_ref();
        carried_lki(object, binding.and_then(|b| b.subject), binding.and_then(departure_frame), &self.departed)
    }
}

impl ResolvingObject {
    /// The last known information it carries for `object` (`carried_lki`).
    pub fn lki_of(&self, object: ObjectRef) -> Option<&DepartedFrame> {
        carried_lki(object, self.subject, self.subject_lki.as_ref(), &self.departed)
    }
}

impl GameState {
    /// "That object" — the binding's subject, if it is still the object the
    /// event was about. CR 603.6's "unable to be found in the zone it went
    /// to" and CR 400.7's new object are one comparison: the id is in the
    /// store and its `zone_change_epoch` is the one the dispatcher saw.
    pub fn bound_object(&self, binding: &TriggerBinding) -> Option<ObjectId> {
        let reference = binding.subject?;
        let object = self.objects.get(&reference.id)?;
        (object.zone_change_epoch == reference.zone_change_epoch).then_some(reference.id)
    }

    /// "That player" — the matched event's `player_of` on the first record.
    pub fn bound_player(&self, binding: &TriggerBinding) -> Option<PlayerId> {
        binding.event().player_of(&binding.records.first()?.event)
    }

    /// "That many" — the arm's `amount_of`, summed over the matched records
    /// (Simic Ascendancy's "that many" across a batch). `None` when the arm
    /// carries no quantity, so a card that asks is refused rather than told 0.
    pub fn bound_amount(&self, binding: &TriggerBinding) -> Option<u64> {
        let event = binding.event();
        let mut total: Option<u64> = None;
        for record in &binding.records {
            let n = event.amount_of(&record.event)?;
            total = Some(total.unwrap_or(0) + n);
        }
        total
    }

    /// "Its power", "its toughness": the bound object's characteristics as
    /// CR 608.2h reads them. When the matched event was the object's
    /// departure, it is expected where it was, so the record's CR 603.10a
    /// frame answers (Paladin of Atonement's ruling: its toughness "as it last
    /// existed on the battlefield"). Otherwise it is the object now, while it
    /// is still where the event left it, and once it has left, the last known
    /// information the resolving entry kept as it went (§6.1). `None` there is
    /// a missed capture.
    pub fn bound_characteristics(&self, binding: &TriggerBinding) -> Option<Arc<EffectiveCharacteristics>> {
        let subject = binding.subject?;
        match departure_frame(binding) {
            Some(departed) => Some(Arc::clone(&departed.frame)),
            None => match self.bound_object(binding) {
                Some(id) => compute_characteristics(self, id),
                None => self.resolving.as_ref()?.lki_of(subject).map(|lki| Arc::clone(&lki.frame)),
            },
        }
    }

    /// The bound fact a `TriggeringObject`, `TriggeringPlayer` or `Referred`
    /// atom acts on, as the target slice every primitive already reads —
    /// empty when the object can no longer be found (CR 603.6, 603.7c), which
    /// is the primitive doing nothing. `Err` outside a trigger's resolution:
    /// a spell has no event to refer back to.
    pub(crate) fn bound_targets(
        &self,
        recipient: &EffectRecipient,
        ctx: &ResolutionContext,
    ) -> Result<Vec<ResolvedTarget>, String> {
        let binding = ctx.trigger.as_ref().ok_or_else(|| {
            format!(
                "{:?} on {:?} refers to a trigger's event, and this resolution is not a triggered ability's",
                recipient, ctx.source
            )
        })?;
        Ok(match recipient {
            EffectRecipient::TriggeringObject => {
                self.bound_object(binding).map(ResolvedTarget::Object).into_iter().collect()
            }
            EffectRecipient::TriggeringPlayer => {
                self.bound_player(binding).map(ResolvedTarget::Player).into_iter().collect()
            }
            EffectRecipient::Referred => {
                let objects = binding.referred.objects.iter().filter_map(|r| r.found(self)).map(ResolvedTarget::Object);
                objects.chain(binding.referred.players.iter().map(|&p| ResolvedTarget::Player(p))).collect()
            }
            other => {
                return Err(format!("{:?} is not a bound fact of a trigger", other));
            }
        })
    }
}
