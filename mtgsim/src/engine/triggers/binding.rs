//! The bound facts, read back at resolution (`triggers-architecture.md` §3.4, §6.3).
//!
//! A [`TriggerBinding`] holds its triggering events, each with the record it
//! matched, copied at dispatch, and the objects that event is about. Every
//! read here is of the binding's own records through the matched arm's
//! projection, and each instruction names the event role it reads (TR-5a).

use std::sync::Arc;

use crate::engine::layers::condition::ConditionReferents;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::resolve::{ResolutionContext, ResolvedTarget};
use crate::events::event::EventRecord;
use crate::oracle::characteristics::CurrentOrLastKnown;
use crate::state::game_state::{GameState, ResolvingObject, StackEntry};
use crate::types::effects::EffectRecipient;
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::{EventRole, Lki, PendingTrigger, TriggerBinding, TriggerEvent, TriggeringEventObjects};

/// The frame `record` carries for its object `id`, as `arm` reads it: a
/// departure's CR 603.10a frame for an arm that looks back, and a source
/// that had left before the event, as it last existed
/// (`GameEvent::source_lki`). A "from anywhere" arm's object is the object
/// after the move (CR 603.6c), read where it went.
pub(crate) fn record_frame<'r>(arm: &TriggerEvent, record: &'r EventRecord, id: ObjectId) -> Option<&'r Lki> {
    if let Some(lki) = record.event.source_lki()
        && lki.object().id == id
    {
        return Some(lki);
    }
    let (lki, ..) = record.event.lki()?;
    (arm.looks_back() && lki.object().id == id).then_some(lki)
}

/// An object of a triggering event, as a condition reads it when the ability
/// triggers (CR 603.4's first check): as its record framed it
/// (`record_frame`), and on the board otherwise.
pub(crate) fn read_at_trigger<'r>(arm: &TriggerEvent, record: &'r EventRecord, id: ObjectId) -> CurrentOrLastKnown<'r> {
    CurrentOrLastKnown { id, lki: record_frame(arm, record, id) }
}

/// Each object of the binding's one triggering event that its record carries
/// a frame for, with that frame (`record_frame`). Nothing for a "one or more"
/// binding, which has no single triggering event.
pub(crate) fn record_frames(binding: &TriggerBinding) -> impl Iterator<Item = (ObjectRef, &Lki)> {
    let arm = binding.event();
    binding.triggering_event().into_iter().flat_map(move |event| {
        event.objects.objects().filter_map(move |(_, object)| Some((object, record_frame(arm, &event.record, object.id)?)))
    })
}

/// The last known information a stack object carries for `object` (CR
/// 113.7a, 608.2h), once `object` has left: the frame its trigger's record
/// carries for it (`record_frame`), else what it kept as `object` left later.
/// `None` while `object` has not left. The one reading of the rule, which an
/// object on the stack and the object resolving both ask through `lki_of`.
fn carried_lki<'a>(object: ObjectRef, trigger: Option<&'a TriggerBinding>, kept: &'a [Lki]) -> Option<&'a Lki> {
    let mut frames = trigger.into_iter().flat_map(record_frames);
    match frames.find(|(named, _)| *named == object) {
        Some((_, lki)) => Some(lki),
        None => kept.iter().find(|frame| frame.object() == object),
    }
}

impl StackEntry {
    /// The last known information it carries for `object` (`carried_lki`).
    pub fn lki_of(&self, object: ObjectRef) -> Option<&Lki> {
        carried_lki(object, self.trigger.as_ref(), &self.kept_lki)
    }
}

impl ResolvingObject {
    /// The last known information it carries for `object` (`carried_lki`).
    pub fn lki_of(&self, object: ObjectRef) -> Option<&Lki> {
        carried_lki(object, self.trigger.as_ref(), &self.kept_lki)
    }
}

impl PendingTrigger {
    /// The last known information it carries for `object` (`carried_lki`).
    pub fn lki_of(&self, object: ObjectRef) -> Option<&Lki> {
        carried_lki(object, Some(&self.binding), &self.kept_lki)
    }
}

/// The objects of a trigger's triggering event, as a condition reads them
/// (CR 113.7a, 608.2h; `ConditionReferents`).
#[derive(Clone, Copy)]
pub enum TriggeringEventView<'a> {
    /// Outside a trigger, and a "one or more" trigger, which has no single
    /// triggering event.
    None,
    /// When the ability triggers: one triggering event's objects, each as
    /// `read_at_trigger` reads it.
    AtTrigger { arm: &'a TriggerEvent, record: &'a EventRecord, objects: &'a TriggeringEventObjects },
    /// As the ability resolves: the binding's one triggering event, as
    /// `GameState::triggering_object` reads it.
    Resolving { game: &'a GameState, binding: &'a TriggerBinding },
}

impl<'a> TriggeringEventView<'a> {
    /// The object that played `role`, as it is read here.
    pub fn object(self, role: EventRole) -> Option<CurrentOrLastKnown<'a>> {
        match self {
            TriggeringEventView::None => None,
            TriggeringEventView::AtTrigger { arm, record, objects } => {
                objects.get(arm.role_as_recorded(role)).map(|object| read_at_trigger(arm, record, object.id))
            }
            TriggeringEventView::Resolving { game, binding } => game.triggering_object(binding, role),
        }
    }

    /// Whether any record it holds passes `test`: the one being matched, as
    /// the ability triggers, or one of the binding's, as it resolves.
    pub fn any_record(self, test: impl FnMut(&EventRecord) -> bool) -> bool {
        match self {
            TriggeringEventView::None => false,
            TriggeringEventView::AtTrigger { record, .. } => std::iter::once(record).any(test),
            TriggeringEventView::Resolving { binding, .. } => binding.records().any(test),
        }
    }

    /// The one of its objects that is `id`, as `object` reads it.
    pub fn object_with_id(self, id: ObjectId) -> Option<CurrentOrLastKnown<'a>> {
        EventRole::ALL.into_iter().filter_map(|role| self.object(role)).find(|object| object.id == id)
    }
}

impl GameState {
    /// "That creature", "them": the object that played `role` in each
    /// triggering event that is still the object the event was about. CR
    /// 603.6's "unable to be found in the zone it went to" and CR 400.7's new
    /// object are one comparison: the id is in the store and its
    /// `zone_change_epoch` is the one the dispatcher saw. A zone change's
    /// trigger finds the object it moved only in a public zone (CR 400.7e), so
    /// not in the hand or library it went to.
    pub fn bound_objects(&self, binding: &TriggerBinding, role: EventRole) -> Vec<ObjectId> {
        let zone_change = matches!(binding.event(), TriggerEvent::ZoneChange { .. });
        binding
            .triggering_objects(role)
            .into_iter()
            .filter(|&reference| self.find(reference).is_some_and(|object| !zone_change || object.zone.is_public()))
            .map(|reference| reference.id)
            .collect()
    }

    /// "That player" — the matched event's `player_of` on the first record.
    pub fn bound_player(&self, binding: &TriggerBinding) -> Option<PlayerId> {
        binding.event().player_of(&binding.first_record()?.event)
    }

    /// "That many" — the arm's `amount_of`, summed over the matched records
    /// (Simic Ascendancy's "that many" across a batch). `None` when the arm
    /// carries no quantity, so a card that asks is refused rather than told 0.
    pub fn bound_amount(&self, binding: &TriggerBinding) -> Option<u64> {
        let event = binding.event();
        let mut total: Option<u64> = None;
        for record in binding.records() {
            let n = event.amount_of(&record.event)?;
            total = Some(total.unwrap_or(0) + n);
        }
        total
    }

    /// "Its power", "its toughness": the characteristics of the object that
    /// played `role`, as CR 608.2h reads them. When its record framed
    /// it (`record_frame`), it is expected where it was, so the frame answers
    /// (Paladin of Atonement's ruling: its toughness "as it last existed on the
    /// battlefield"). Otherwise it is the object now, while it is still where
    /// the event left it, and once it has left, the last known information the
    /// resolving entry kept as it went (§6.1). `None` there is a missed
    /// capture.
    pub fn bound_characteristics(
        &self,
        binding: &TriggerBinding,
        role: EventRole,
    ) -> Option<Arc<EffectiveCharacteristics>> {
        self.triggering_object(binding, role)?.characteristics(self)
    }

    /// "It", the object that played `role` in a resolving trigger's
    /// event, in the form CR 608.2h reads it: the object itself while it is
    /// still where the event left it, and otherwise how it last existed.
    ///
    /// "How it last existed" comes from one of two places. When its record
    /// framed it — a dies trigger's creature, a source that left before
    /// dealing damage — that frame. When it left later, say sacrificed in
    /// response, the resolving trigger kept its frame as it went
    /// ([`Self::lki_kept_by_resolving`]). `None` when the trigger has no
    /// single triggering event, or when the object left and nothing was kept,
    /// which is a missed capture.
    pub(crate) fn triggering_object<'a>(
        &'a self,
        binding: &'a TriggerBinding,
        role: EventRole,
    ) -> Option<CurrentOrLastKnown<'a>> {
        let object = binding.triggering_object(role)?;
        let lki = match record_frame(binding.event(), &binding.triggering_event()?.record, object.id) {
            Some(lki) => Some(lki),
            None if self.find(object).is_some() => None,
            None => Some(self.lki_kept_by_resolving(object)?),
        };
        Some(CurrentOrLastKnown { id: object.id, lki })
    }

    /// How `object` last existed, as the spell or ability now resolving
    /// remembers it — for an object its text names that has left the zone
    /// it expected (CR 113.7a, 608.2h): a dies trigger's own creature, or a
    /// creature sacrificed to pay the ability's cost. A stack object keeps
    /// these as they leave because the board can no longer say what they
    /// were. `None` while `object` is still there.
    pub(crate) fn lki_kept_by_resolving(&self, object: ObjectRef) -> Option<&Lki> {
        self.resolving.as_ref()?.lki_of(object)
    }

    /// What a resolving condition reads about the objects its text names
    /// (CR 113.7a, 608.2h): the source's last known information once
    /// `source`, the existence that put the ability on the stack, has left,
    /// and the triggering event's objects, as `triggering_object` reads them.
    pub(crate) fn condition_objects<'a>(
        &'a self,
        source: Option<ObjectRef>,
        binding: Option<&'a TriggerBinding>,
    ) -> ConditionReferents<'a> {
        ConditionReferents {
            source_lki: source.and_then(|source| self.lki_kept_by_resolving(source)),
            triggering_event: match binding {
                Some(binding) => TriggeringEventView::Resolving { game: self, binding },
                None => TriggeringEventView::None,
            },
        }
    }

    /// The bound fact a `Triggering`, `TriggeringPlayer` or `Referred` atom
    /// acts on, as the target slice every primitive already reads — empty when
    /// the object can no longer be found (CR 603.6, 603.7c), which is the
    /// primitive doing nothing. `Err` outside a trigger's resolution: a spell
    /// has no event to refer back to.
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
            EffectRecipient::Triggering(role) => {
                self.bound_objects(binding, *role).into_iter().map(ResolvedTarget::Object).collect()
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
