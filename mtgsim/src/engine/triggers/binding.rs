//! The bound facts, read back at resolution (`triggers-architecture.md` §3.4, §6.3).
//!
//! A [`TriggerBinding`] points at the records and copies nothing they hold,
//! so every read here goes back to the log through the matched arm's
//! projection — one copy of each fact, and the frame comes with the record.

use std::sync::Arc;

use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::resolve::{ResolutionContext, ResolvedTarget};
use crate::events::event::GameEvent;
use crate::state::game_state::GameState;
use crate::types::effects::EffectRecipient;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::triggers::TriggerBinding;

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
        let first = binding.records.first()?;
        let record = self.events.record(*first)?;
        binding.event().player_of(&record.event)
    }

    /// "That many" — the arm's `amount_of`, summed over the matched records
    /// (Simic Ascendancy's "that many" across a batch). `None` when the arm
    /// carries no quantity, so a card that asks is refused rather than told 0.
    pub fn bound_amount(&self, binding: &TriggerBinding) -> Option<u64> {
        let event = binding.event();
        let mut total: Option<u64> = None;
        for seq in &binding.records {
            let record = self.events.record(*seq)?;
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
    /// is still where the event left it. `None` when it is neither, which is
    /// the last known information TR-2b's `departed` frames hold.
    pub fn bound_characteristics(&self, binding: &TriggerBinding) -> Option<Arc<EffectiveCharacteristics>> {
        let subject = binding.subject?;
        let first = self.events.record(*binding.records.first()?)?;
        match &first.event {
            GameEvent::ZoneChange { object_id, lki: Some(frame), .. }
            | GameEvent::LeftTheGame { object_id, lki: Some(frame), .. }
                if *object_id == subject.id =>
            {
                Some(Arc::clone(frame))
            }
            _ => compute_characteristics(self, self.bound_object(binding)?),
        }
    }

    /// The bound fact a `TriggeringObject` or `TriggeringPlayer` atom acts
    /// on, as the target slice every primitive already reads — empty when
    /// the object can no longer be found (CR 603.6), which is the primitive
    /// doing nothing. `Err` outside a trigger's resolution: a spell has no
    /// event to refer back to.
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
            other => {
                return Err(format!("{:?} is not a bound fact of a trigger", other));
            }
        })
    }
}
