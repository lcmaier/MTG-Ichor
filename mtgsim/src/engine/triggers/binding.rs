//! The bound facts, read back at resolution (`triggers-architecture.md` §3.4, §6.3).
//!
//! A [`TriggerBinding`] holds the records it matched, copied at dispatch, so
//! every read here is of the binding's own records through the matched arm's
//! projection, and the frame comes with the record.

use std::sync::Arc;

use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::condition::ConditionObjects;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::resolve::{ResolutionContext, ResolvedTarget};
use crate::state::game_state::{GameState, ResolvingObject, StackEntry};
use crate::types::effects::EffectRecipient;
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::{Lki, TriggerBinding};

/// The subject as it was in the zone it left, when the matched event was its
/// own departure and the arm that matched looks back: the record's CR
/// 603.10a frame, naming the existence that left. A "from anywhere" arm's
/// subject is the object after the move (CR 603.6c), read where it went.
pub(crate) fn subject_lki(binding: &TriggerBinding) -> Option<&Lki> {
    let subject = binding.subject?;
    if !binding.event().looks_back() {
        return None;
    }
    let (lki, ..) = binding.records.first()?.event.lki()?;
    (lki.object().id == subject.id).then_some(lki)
}

/// What "them" means in a trigger like Hedge Shredder's "Whenever one or more
/// land cards are put into your graveyard from your library, put them onto the
/// battlefield": every card that event moved. One trigger matched all of the
/// event's records (CR 603.2c's "one or more"), and each record says where its
/// card went (`GameEvent::new_object`), so a card that has moved again since
/// is not found. `Err` for a record that is not a zone change: it says only
/// the object's id, not which existence of it the event was about.
pub(crate) fn triggering_objects(binding: &TriggerBinding) -> Result<Vec<ObjectRef>, String> {
    binding
        .records
        .iter()
        .map(|record| {
            record.event.new_object().ok_or_else(|| {
                format!("\"them\" names the objects zone changes made, and record {} is {:?}", record.seq.0, record.event)
            })
        })
        .collect()
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
    subject_lki: Option<&'a Lki>,
    kept: &'a [Lki],
) -> Option<&'a Lki> {
    match subject_lki {
        Some(lki) if subject == Some(object) => Some(lki),
        _ => kept.iter().find(|frame| frame.object() == object),
    }
}

impl StackEntry {
    /// The last known information it carries for `object` (`carried_lki`).
    pub fn lki_of(&self, object: ObjectRef) -> Option<&Lki> {
        let binding = self.trigger.as_ref();
        carried_lki(object, binding.and_then(|b| b.subject), binding.and_then(subject_lki), &self.kept_lki)
    }
}

impl ResolvingObject {
    /// The last known information it carries for `object` (`carried_lki`).
    pub fn lki_of(&self, object: ObjectRef) -> Option<&Lki> {
        carried_lki(object, self.subject, self.subject_lki.as_ref(), &self.kept_lki)
    }
}

impl GameState {
    /// "That object" — the binding's subject, if it is still the object the
    /// event was about. CR 603.6's "unable to be found in the zone it went
    /// to" and CR 400.7's new object are one comparison: the id is in the
    /// store and its `zone_change_epoch` is the one the dispatcher saw.
    pub fn bound_object(&self, binding: &TriggerBinding) -> Option<ObjectId> {
        let reference = binding.subject?;
        self.find(reference).map(|_| reference.id)
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
        match self.bound_as_it_was(binding)? {
            (_, Some(lki)) => Some(Arc::clone(lki.frame())),
            (id, None) => compute_characteristics(self, id),
        }
    }

    /// The bound object as CR 608.2h reads it, `bound_characteristics`'
    /// order: its id with the record's frame when the event was its
    /// departure, alone while it is still where the event left it, or with
    /// the last known information the resolving entry kept as it left later.
    /// `None` for a binding with no subject, and for a missed capture.
    fn bound_as_it_was<'a>(&'a self, binding: &'a TriggerBinding) -> Option<(ObjectId, Option<&'a Lki>)> {
        let subject = binding.subject?;
        match subject_lki(binding) {
            Some(lki) => Some((subject.id, Some(lki))),
            None => match self.bound_object(binding) {
                Some(id) => Some((id, None)),
                None => self.resolving.as_ref()?.lki_of(subject).map(|lki| (subject.id, Some(lki))),
            },
        }
    }

    /// What a resolving condition reads about the objects its text names
    /// (CR 113.7a, 608.2h): the source's last known information once
    /// `source`, the existence that put the ability on the stack, has left,
    /// and "it", the trigger's subject, as `bound_characteristics` reads it.
    pub(crate) fn condition_objects<'a>(
        &'a self,
        source: Option<ObjectRef>,
        binding: Option<&'a TriggerBinding>,
    ) -> ConditionObjects<'a> {
        let resolving = self.resolving.as_ref();
        ConditionObjects {
            source_lki: source.and_then(|source| resolving?.lki_of(source)),
            triggering: binding.and_then(|binding| self.bound_as_it_was(binding)),
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
            EffectRecipient::TriggeringObjects => triggering_objects(binding)?
                .into_iter()
                .filter(|&object| self.find(object).is_some())
                .map(|object| ResolvedTarget::Object(object.id))
                .collect(),
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
