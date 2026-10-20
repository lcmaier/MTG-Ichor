//! Detection — the dispatch (`triggers-architecture.md` §4).
//!
//! A dispatch is the matcher's run over one **window**: every record one
//! batch stamped, run when the outermost `execute_actions` that opened it is
//! about to return, its riders' records then forming a batch of their own; or
//! one unbatched record, run inside `emit_event` before it returns (§4.1). The instant is still synchronous
//! with the mutation — before the next proposal, before the state-based
//! check, before anyone receives priority — so "that instant is now" holds
//! and the layer walk answers live.
//!
//! **Who is asked** (§4.2) mirrors `replacement::gather` because a new reader
//! of the effective ability list is dead on every board a gate skips: the
//! battlefield sweep behind `trigger_sources`, the zone sweep behind
//! `zone_trigger_sources`, the granted and copied legs on
//! `RegistryScopeSummary`, and the frames the window's departures carry
//! (CR 603.10a) — plus, for an object that survives a batch which removed
//! the source of an effect copying, granting or removing abilities, the list
//! it had before ([`LookBackSnapshot`]). When every set is empty, a dispatch
//! is the probes and nothing else.

use std::sync::Arc;

use crate::engine::mana::ManaProducer;
use crate::oracle::characteristics::CurrentOrLastKnown;
use crate::types::effects::Antecedent;
use crate::engine::actions::{ActionContext, GameAction};
use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::condition::{settled_holds, ConditionReferents};
use crate::engine::layers::types::{EffectiveCharacteristics, Timestamp};
use crate::engine::mana::triggered_mana_recipient;
use crate::engine::object_filter::{player_ref_names, EarlierTargets, FilterReferents};
use crate::engine::resolve::ResolutionContext;
use crate::engine::targeting::HeldFrame;
use crate::engine::trace_records;
use crate::engine::zone_function::{condition_functions_in, functions_in};
use crate::events::cause::{EventCause, GameRule, GeneratedBy};
use crate::events::event::{BatchId, CounterSubject, DamageTarget, EventRecord, EventId, GameEvent};
use crate::objects::card_data::{AbilityDef, CardData};
use crate::oracle::characteristics::controller_or_owner;
use crate::state::battlefield::{AttackTarget, CostChoices};
use crate::state::continuous_effects::triggered_ability_kinds;
use crate::state::game_state::{AbilityIdentity, GameState, StackEntry};
use crate::types::effects::{Effect, EffectRecipient, ObjectSet, PlayerRef, Primitive};
use crate::engine::targeting::TargetRef;
use crate::types::history::CountIs;
use crate::types::ids::{DelayedTriggerId, IdMap, IdSet, ObjectId, ObjectRef, PlayerId};
use crate::types::mana::ManaType;
use crate::types::triggers::{
    zone_change_looks_back, AttackOccurrence, BlockOccurrence, CountersOn, TargetSubject, TargetedBy, DamageRecipient, DefenderFilter, EventIndex, EventKind, EventKindMask, IdentityRef, Lki, Multiplicity,
    ObjectOrPlayer, PendingTrigger, Referred, RememberedObject, TriggerBinding, TriggerCondition, TriggerDef, TriggerEvent,
    TriggerLimit, TriggerOrigin, TriggerSeq, TriggerSubject, TriggeringEvent, TriggeringEventObjects,
};
use super::binding::TriggeringEventView;

use crate::types::zones::{DrawOrdinals, Zone, ZoneSet};

/// Dispatches nested inside dispatches — a tier-2 trigger's `AbilityTriggered`
/// dispatched from the dispatch that queued it, and so on. The recursion is
/// bounded by the abilities present (no printed ability watches its own
/// kind); a depth past this is the engine's mistake (§4.8).
pub const DISPATCH_NESTING_LIMIT: usize = 16;

/// The ability lists surviving objects had just before a batch was
/// performed: the survivor's counterpart of the CR 603.10a frame a departure
/// record carries.
///
/// CR 603.10 decides a leaves-the-battlefield trigger from "the existence of
/// those abilities ... immediately prior to the event". An object that left
/// has that list on its zone change. An object that stayed has no record, and
/// its list after the event differs from the one before only when the batch
/// removed the source of an effect that copies, grants or removes abilities:
/// Humility dying gives a surviving Blood Artist its ability back, and a
/// granter dying takes the granted one away. Once that source is gone its
/// effect is gone from the registry and the old list cannot be recomputed, so
/// the batch saves the lists first, between deciding and performing, and the
/// dispatch at the window's close answers look-back arms from them.
///
/// A nested batch that joins the window takes its own. A record reads the
/// outermost one whose batch performed it, since a nested batch inside a
/// performer is the enclosing event at finer grain (CR 704.3's one event);
/// failing that the first one taken after it, since no list changed between;
/// failing that the live list, which is then also the list before it.
#[derive(Debug, Clone)]
pub struct LookBackSnapshot {
    pub(crate) window: Option<BatchId>,
    /// The records the batch performed, by sequence number.
    pub(crate) performed: std::ops::Range<usize>,
    pub(crate) frames: Vec<ObjectSnapshot>,
}

/// One object as a snapshot holds it, with where it was: its last known
/// information, which answers for an object that has left since (the
/// audit's) as a departure record's answers for the dispatcher — its ability
/// list, its host and its counters.
#[derive(Debug, Clone)]
pub struct ObjectSnapshot {
    pub(crate) zone: Zone,
    pub(crate) lki: Lki,
}

impl LookBackSnapshot {
    /// The snapshot `seq` looks back through, by the rule above.
    fn for_record(snapshots: &[LookBackSnapshot], seq: usize) -> Option<usize> {
        let covering = snapshots.iter().enumerate().filter(|(_, s)| s.performed.contains(&seq));
        let later = snapshots.iter().enumerate().filter(|(_, s)| s.performed.start > seq);
        covering
            .min_by_key(|(_, s)| s.performed.start)
            .or_else(|| later.min_by_key(|(_, s)| s.performed.start))
            .map(|(k, _)| k)
    }
}

/// One permanent's CR 603.10a frame, taken before the batch that decided its
/// departure performs: the permanent as it was immediately before the event,
/// whichever member of the event moves first.
#[derive(Debug, Clone)]
pub struct LkiCapture {
    object: ObjectRef,
    /// `None` once the move has taken it.
    lki: Option<Lki>,
}

/// A holder of a `kept_lki` (CR 113.7a, 608.2h): a triggered ability waiting
/// to be put on the stack, a stack entry, the resolving object, or a delayed
/// trigger waiting for its event. Each reads its source later, and a trigger
/// the objects of its one triggering event too, so when one of them leaves,
/// the keeper keeps its last known information.
#[derive(Clone, Copy)]
struct LkiKeeper<'a> {
    source: Option<ObjectRef>,
    trigger: Option<&'a TriggerBinding>,
}

impl LkiKeeper<'_> {
    /// The objects it keeps last known information for: its source, then its
    /// triggering event's objects.
    fn objects(self) -> impl Iterator<Item = ObjectRef> {
        let event_objects = self.trigger.and_then(TriggerBinding::triggering_event).into_iter().flat_map(|event| event.objects.objects());
        self.source.into_iter().chain(event_objects.map(|(_, object)| object))
    }
}

/// Where a keeper keeps the last known information of an object it reads
/// once that object leaves (CR 113.7a, 608.2h): the list of a waiting
/// triggered ability, a stack entry or the resolving object, one per object,
/// or a delayed trigger's one of its source.
enum KeptLkiSlot<'a> {
    List(&'a mut Vec<Lki>),
    Source(&'a mut Option<Lki>),
}

/// CR 605.1b's three criteria, derived from the def and never a tag: no
/// target, could add mana, and triggered from mana being added or a mana
/// ability's activation, which is the triggering record's to say. "Whenever
/// you activate an ability, add {C}" is a mana ability when a mana ability's
/// activation triggers it, and an ordinary triggered ability when another
/// activation does. CR 605.5a is the same sentence from the other side.
pub fn is_mana_ability(def: &TriggerDef, triggered_by: &GameEvent) -> bool {
    could_be_a_mana_ability(def)
        && matches!(triggered_by, GameEvent::ManaAdded { .. } | GameEvent::AbilityActivated { mana_ability: true, .. })
}

/// Whether some event could make `def` a mana ability (`is_mana_ability`):
/// no target, could add mana, and an arm that can trigger on mana being added
/// or on a mana ability's activation. What a mana payment counts as mana its
/// own activations may still produce.
pub fn could_be_a_mana_ability(def: &TriggerDef) -> bool {
    let from_mana = |arm: &TriggerEvent| {
        matches!(arm, TriggerEvent::ManaAdded { .. } | TriggerEvent::ActivatesAbility { mana_ability: None | Some(true), .. })
    };
    def.condition.events().iter().any(from_mana) && def.effect.instances().is_empty() && could_add_mana(&def.effect)
}

fn could_add_mana(effect: &Effect) -> bool {
    match effect {
        Effect::Atom(Primitive::ProduceMana(_), _) => true,
        Effect::Atom(..) => false,
        Effect::Sequence(effects) | Effect::Modal { modes: effects, .. } => {
            effects.iter().any(could_add_mana)
        }
        Effect::Conditional(_, inner)
        | Effect::Optional { effect: inner, .. }
        | Effect::Remember(inner)
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

/// One ability list of one object the matcher asks: a live object's list
/// now, the CR 603.10a frame of an object that left, or a survivor's list from
/// before a batch. An object with lists from before the event is several
/// candidates, one per list, side by side.
pub(super) struct TriggerCandidate<'a> {
    /// The existence whose list this is: the object now, or the one that left,
    /// which is what triggered (CR 400.7).
    pub(super) object: ObjectRef,
    /// CR 603.3a's "you": the player who controls the source now, or the
    /// frame's controller for one that left.
    pub(super) controller: PlayerId,
    pub(super) owner: PlayerId,
    pub(super) zone: Zone,
    /// The source's host (CR 303.4m), for `TriggerSubject::Host`.
    pub(super) host: Option<ObjectId>,
    pub(super) frame: TriggerCandidateFrame<'a>,
    /// What a stack object is built from (CR 603.3's "text of the ability").
    pub(super) card: Arc<CardData>,
}

/// What a triggered ability's text means by its words for itself, which its
/// trigger event is read against (§3.3): "this object", "you", "its owner",
/// "enchanted" or "equipped", and "this ability". An object's own ability
/// takes them from the candidate; a delayed trigger from its registry entry
/// (CR 603.7d–g), whose "this object" is its source remembered by identity.
#[derive(Clone, Copy)]
pub(super) struct TriggerReferents<'a> {
    /// "This object".
    pub(super) this: ThisObject,
    /// "You" (CR 109.5): the controller.
    pub(super) controller: PlayerId,
    /// "Its owner", of "this object".
    pub(super) owner: PlayerId,
    /// The source's host (CR 303.4m), for `TriggerSubject::Host`.
    pub(super) host: Option<ObjectId>,
    /// CR 603.7h's "this ability": the ability whose arm this is, or the one
    /// that created a delayed trigger; `None` for a spell's delayed trigger.
    pub(super) this_ability: Option<AbilityIdentity>,
    /// CR 603.7c's "that token": the objects a delayed trigger refers to;
    /// none for an object's own ability.
    pub(super) referred: &'a [RememberedObject],
}

/// A trigger's text names no instance of "target" for "another target" to
/// be other than.
impl FilterReferents for TriggerReferents<'_> {
    fn you(&mut self) -> PlayerId {
        self.controller
    }

    fn source(&self) -> Option<ObjectId> {
        Some(self.this.id())
    }

    fn source_owner(&mut self) -> Option<PlayerId> {
        Some(self.owner)
    }

    fn earlier_targets(&self) -> EarlierTargets<'_> {
        EarlierTargets::None
    }

    /// The candidate, now: an arm's own words have no triggering event yet.
    fn defending_player(&self, game: &GameState, creature: Antecedent) -> Option<PlayerId> {
        match creature {
            Antecedent::ThisObject => CurrentOrLastKnown { id: self.this.id(), lki: None }.defending_player(game, &|_| {}),
            Antecedent::Triggering(_) => None,
        }
    }
}

/// What "this object" is to an arm.
#[derive(Clone, Copy)]
pub(super) enum ThisObject {
    /// An object's own ability: the candidate is that object, now or as its
    /// frame, so its id is the whole question.
    Candidate(ObjectId),
    /// A delayed trigger's source (CR 603.7d–g), remembered by identity (CR
    /// 400.7) with the record of the move that ended it, if one has.
    Remembered(RememberedObject),
}

impl ThisObject {
    fn id(self) -> ObjectId {
        match self {
            ThisObject::Candidate(id) => id,
            ThisObject::Remembered(remembered) => remembered.object.id,
        }
    }

    /// Whether record `seq`'s object `id` is this object: the candidate, or
    /// the remembered existence (CR 400.7).
    fn is(self, game: &GameState, id: ObjectId, seq: EventId) -> bool {
        match self {
            ThisObject::Candidate(this) => id == this,
            ThisObject::Remembered(remembered) => remembered.is(game, id, seq),
        }
    }
}

/// Which list a candidate reads, and so which trigger arms it answers: CR
/// 603.10 gives a look-back arm the list from before the event and every
/// other arm the list now.
pub(super) enum TriggerCandidateFrame<'a> {
    /// A live object's list now, off the layer memo. `snapshots` are the
    /// ones holding its list from before, by index into the window's.
    Live { chars: Arc<EffectiveCharacteristics>, snapshots: Vec<usize> },
    /// An object's list from before it left: its last known information
    /// (CR 113.7a). `snapshot: None` is the CR 603.10a frame its record
    /// carries, which answers the look-back arms of every record of the
    /// window; `Some(k)` is the one snapshot `k` holds, which answers those
    /// of the records its batch performed (the audit's).
    LastKnown { lki: &'a Lki, snapshot: Option<usize> },
    /// A surviving object's list from before a batch, off `LookBackSnapshot`
    /// number `snapshot`.
    Before { chars: &'a EffectiveCharacteristics, snapshot: usize },
}

impl TriggerCandidateFrame<'_> {
    fn chars(&self) -> &EffectiveCharacteristics {
        match self {
            TriggerCandidateFrame::Live { chars, .. } => chars,
            TriggerCandidateFrame::LastKnown { lki, .. } => lki.chars(),
            TriggerCandidateFrame::Before { chars, .. } => chars,
        }
    }

    /// The candidate's last known information, for an object that left: what
    /// its own condition reads "this" off (CR 113.7a).
    fn lki(&self) -> Option<&Lki> {
        match self {
            TriggerCandidateFrame::LastKnown { lki, .. } => Some(lki),
            TriggerCandidateFrame::Live { .. } | TriggerCandidateFrame::Before { .. } => None,
        }
    }

    /// The arms this list may answer for a record that looks back through
    /// snapshot `looks_back_through`, or `None` when it answers none. An
    /// object that left has only its list from before; a survivor's list now
    /// answers every arm unless the record's snapshot holds the one from
    /// before, which then answers the look-back arms in its place. A
    /// `newcomer` was somewhere else immediately before the event, so its
    /// list now answers no look-back arm (CR 603.10a; Bridge from Below's
    /// third ruling).
    fn answered_arms(&self, looks_back_through: Option<usize>, newcomer: bool) -> Option<AnsweredArms> {
        match self {
            TriggerCandidateFrame::Live { snapshots, .. }
                if newcomer || looks_back_through.is_some_and(|k| snapshots.contains(&k)) =>
            {
                Some(AnsweredArms::AllButLookBack)
            }
            TriggerCandidateFrame::Live { .. } => Some(AnsweredArms::All),
            TriggerCandidateFrame::LastKnown { snapshot: None, .. } => Some(AnsweredArms::LookBackOnly),
            TriggerCandidateFrame::LastKnown { snapshot: Some(k), .. } | TriggerCandidateFrame::Before { snapshot: k, .. }
                if looks_back_through == Some(*k) =>
            {
                Some(AnsweredArms::LookBackOnly)
            }
            TriggerCandidateFrame::LastKnown { .. } | TriggerCandidateFrame::Before { .. } => None,
        }
    }
}

/// One ability's match against one record, before "one or more" folds it.
struct ArmMatch<'r, 'a> {
    row: &'r TriggerCandidateDef<'a>,
    /// Which of the condition's events matched.
    arm: EventIndex,
    triggering_events: Vec<TriggeringEventObjects>,
}

/// One triggered ability of one candidate, with the facts that do not depend
/// on the record answered once: CR 113.6 and the identity.
struct TriggerCandidateDef<'a> {
    /// Index into the candidate list.
    candidate: usize,
    identity: AbilityIdentity,
    def: &'a Arc<TriggerDef>,
    instances: &'a [EffectRecipient],
}

/// Which of a triggered ability's arms one candidate's ability list may
/// answer for one record (CR 603.10): a list from before the event answers
/// the look-back arms (`TriggerEvent::looks_back`), a list after it every
/// other arm, and a live object's list both, unless the event changed the
/// list or brought the object.
#[derive(Clone, Copy)]
enum AnsweredArms {
    /// A live object's list, which the event left as it was.
    All,
    /// A list from before the event: an object that left, or a survivor's
    /// list a snapshot kept.
    LookBackOnly,
    /// A live object's list after an event that changed it, or of an object
    /// the event brought.
    AllButLookBack,
}

/// How an arm reads its subject off a record: where the subject was at the
/// moment the trigger asks about.
///
/// The last two both read the subject's last known information, and read its
/// characteristics alike, including what an Aura or Equipment gave it
/// (Loxodon Warhammer's lifelink on a Dragonhawk that has died). What sets
/// them apart is whether the subject was a permanent at the moment the
/// trigger asks about, which decides one question: whether the trigger's own
/// Aura or Equipment was attached to it (`TriggerSubject::Host`). Only a
/// permanent has anything attached to it (CR 301.5, 303.4).
#[derive(Clone, Copy)]
enum SubjectRead<'a> {
    /// Where it is now.
    Now,
    /// This event is the subject leaving where it was, and the trigger looks
    /// back to just before it (CR 603.10): a permanent leaving the
    /// battlefield, as its record framed it, which "when enchanted creature
    /// dies" matches the Aura that was on; or a spell countered on the stack.
    LeftInThisEvent(&'a Lki),
    /// It had left the battlefield at some earlier event, and deals damage
    /// now as it last existed, through an ability that resolved after it had
    /// gone (CR 608.2h): Dragonhawk's end-step damage. It is no permanent at
    /// this event, so no Aura or Equipment is on it, and "whenever enchanted
    /// creature deals damage" never matches it, even an Aura on a new object
    /// that shares its id after a flicker (CR 400.7).
    GoneBeforeThisEvent(&'a Lki),
}

impl<'a> SubjectRead<'a> {
    /// A departure's subject: as its record framed it for an arm that looks
    /// back, and now for any other, "from anywhere" among them (CR 603.6c,
    /// 603.10).
    fn of_departure(arm: &TriggerEvent, lki: &'a Option<Lki>) -> SubjectRead<'a> {
        match lki {
            Some(lki) if arm.looks_back() => SubjectRead::LeftInThisEvent(lki),
            _ => SubjectRead::Now,
        }
    }
}

/// The object a triggered ability belongs to, as its intervening "if" reads
/// it at the trigger: the object, its controller (the "if"'s "you", CR
/// 109.5), and how it last existed once it has left — persist's creature,
/// which has just died.
#[derive(Clone, Copy)]
pub(super) struct TriggerSource<'a> {
    pub(super) object: ObjectId,
    pub(super) controller: PlayerId,
    pub(super) lki: Option<&'a Lki>,
}

/// A match the dispatcher will queue or resolve.
pub(super) struct MatchedTrigger {
    pub(super) identity: AbilityIdentity,
    pub(super) controller: PlayerId,
    pub(super) def: Arc<TriggerDef>,
    pub(super) source_card: Arc<CardData>,
    pub(super) instances: Vec<EffectRecipient>,
    pub(super) event: EventIndex,
    pub(super) triggering_events: Vec<TriggeringEvent>,
    pub(super) mana: bool,
    /// The source's last known information when the existence that
    /// triggered had already left, which its entry keeps from the start (CR
    /// 113.7a): no later hand-over will reach it.
    pub(super) source_lki: Option<Lki>,
}

/// One registry entry's matches in a window (`delayed.rs`): every event
/// that triggers it, which its duration then queues all of or chooses one of.
pub(super) struct DelayedMatch {
    pub(super) id: DelayedTriggerId,
    pub(super) occurrences: Vec<MatchedTrigger>,
}

/// Why a candidate did not trigger — the `trigger` record's field.
#[derive(Clone, Copy)]
pub(super) enum Refusal {
    /// The condition is a state trigger, which TR-6 checks.
    StateTrigger,
    /// CR 603.2f.
    Visibility,
    /// The arm does not read this record, or its predicates said no.
    TriggerCondition,
    /// CR 603.4 at the trigger.
    InterveningIf,
    /// A once-per-turn limit (§3.5): the action was taken, the ability has
    /// triggered, or the record is not the turn's first of its kind.
    Limit,
}

impl Refusal {
    pub(super) fn name(self) -> &'static str {
        match self {
            Refusal::StateTrigger => "state",
            Refusal::Visibility => "visibility",
            Refusal::TriggerCondition => "condition",
            Refusal::InterveningIf => "intervening_if",
            Refusal::Limit => "limit",
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
        mark: EventId,
        batch: Option<BatchId>,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        let window: Vec<EventId> = self
            .events
            .records_since(mark)
            .iter()
            .filter(|r| r.stamp.batch == batch)
            .map(|r| r.seq)
            .collect();
        let (snapshots, others): (Vec<_>, Vec<_>) = std::mem::take(&mut self.look_back_snapshots)
            .into_iter()
            .partition(|s| s.window == batch);
        self.look_back_snapshots = others;
        let audit = self.take_audit_snapshots(batch);
        let result = self.dispatch(&window, Some(ctx), &snapshots, audit.as_deref());
        self.flush_window_unless_nested();
        result
    }

    /// One record emitted outside any batch — a cast, an activation, an
    /// ability's resolution, a trigger triggering, attackers declared —
    /// dispatched inside `emit_event`. `ctx` is the emitter's provider, where
    /// the dispatch may have to ask (`emit_event_with_provider`). A
    /// triggered mana ability resolves here only on a mana ability's
    /// activation, which `announce_activation` emits with its provider.
    pub(crate) fn dispatch_unbatched(&mut self, seq: EventId, ctx: Option<&ActionContext>) -> Result<(), String> {
        let audit = self.dispatch_audit.is_some().then(Vec::new);
        let result = self.dispatch(&[seq], ctx, &[], audit.as_deref());
        self.flush_window_unless_nested();
        result
    }

    /// Flush the window when nothing can still read it: the dispatch that just
    /// returned was the outermost one, and no batch is open (§4.1).
    ///
    /// Anywhere else a record in the window still has a reader to come:
    /// - **Inside another dispatch.** A dispatch that queues two triggers
    ///   emits two `AbilityTriggered` records, and each one's own dispatch
    ///   reads the record that caused it. A flush after the first would take
    ///   away the cause the second reads.
    /// - **Inside an open batch.** An auxiliary batch (CR 614.13's devour
    ///   sacrifice) is dispatched while the batch it interrupts is still open,
    ///   and that batch's records are dispatched only when it closes. A flush
    ///   there would take them out of the window before their dispatch.
    /// - **Inside a resolution.** A later instruction reads what the earlier
    ///   ones performed (`EventWindow::resolution_records`): what
    ///   `Effect::Remember` remembers (CR 603.7c). `resolve_top_of_stack`
    ///   flushes as the resolution ends.
    ///
    /// A trigger keeps copies of the records it binds, so nothing reads the
    /// window after this.
    pub(crate) fn flush_window_unless_nested(&mut self) {
        if self.nesting.dispatch_depth == 0 && self.nesting.batch_depth == 0 && self.resolving.is_none() {
            self.events.flush();
        }
    }

    fn dispatch(
        &mut self,
        window: &[EventId],
        ctx: Option<&ActionContext>,
        snapshots: &[LookBackSnapshot],
        audit: Option<&[LookBackSnapshot]>,
    ) -> Result<(), String> {
        // CR 104.1 — a game that has ended queues nothing; nobody would
        // receive priority to place it.
        if window.is_empty() || self.result.is_some() {
            return Ok(());
        }
        if self.nesting.dispatch_depth >= DISPATCH_NESTING_LIMIT {
            return Err(format!(
                "trigger dispatches nested {} deep: an ability's triggering keeps triggering \
                 another, which no printed ability does",
                self.nesting.dispatch_depth
            ));
        }
        // Every record, before the gate: what happened this turn is read by
        // cards that are not on the battlefield yet (§3.10).
        self.advance_history(window);
        self.nesting.dispatch_depth += 1;
        let result = self.dispatch_inner(window, ctx, snapshots, audit);
        self.nesting.dispatch_depth -= 1;
        result
    }

    /// One dispatch, in five steps.
    ///
    /// 1. **The gate** — six probes (§4.2): the permanents whose printed
    ///    triggers read a kind this window carries (§11's mask, source
    ///    first), the zone map, the window's newcomers whose every trigger is
    ///    on themselves moving, the unattributed zone set off the registry
    ///    summary, whether any record of the window carries a CR 603.10a
    ///    frame with a triggered ability in it, and whether a batch of the
    ///    window took a look-back snapshot. All empty, and a dispatch is the
    ///    probes and nothing else.
    /// 2. **The candidates** — `find_matches`' four legs: every object that
    ///    may carry a triggered ability functioning where it is, in CR 613.7
    ///    order, each with the effective ability list read once and the card a
    ///    stack object would be built from.
    /// 3. **The match** — every candidate's every triggered def against every
    ///    record of the window. `match_def` answers with the event that
    ///    matched and the subject of each occurrence, or with the predicate
    ///    that refused; either way one `trigger` trace record is written.
    /// 4. **Queue, or resolve** — a `PendingTrigger` per match onto
    ///    `GameState` and nothing else (CR 603.2, 117.2a: nothing happens when
    ///    an ability triggers), with the one exception the CR makes — a
    ///    triggered mana ability, which CR 605.4a resolves here and never
    ///    queues.
    /// 5. **The tier-2 emission** — one unstamped `AbilityTriggered` per
    ///    queued trigger, after the window has closed, each dispatched as it
    ///    is emitted (§4.8). CR 603.3b's second tier is what watches that
    ///    record, and the recursion it opens is what `DISPATCH_NESTING_LIMIT`
    ///    bounds.
    ///
    /// In an audited game `audit` is the window's snapshots of every object
    /// (§4.10), and the audit answers the same window between steps 3 and 4,
    /// whatever the gate said.
    fn dispatch_inner(
        &mut self,
        window: &[EventId],
        ctx: Option<&ActionContext>,
        snapshots: &[LookBackSnapshot],
        audit: Option<&[LookBackSnapshot]>,
    ) -> Result<(), String> {
        // The window's kinds, OR-ed once (§11). A window no arm can read — a
        // spell cast, an activation, a shuffle — would be refused by every
        // arm of every candidate on every leg; this is that answer without
        // the walk.
        let window_kinds = window.iter().fold(EventKindMask::EMPTY, |mask, seq| {
            match self.events.record(*seq).and_then(|r| EventKind::from_record(&r.event)) {
                Some(kind) => mask.with(kind),
                None => mask,
            }
        });
        let matches = self.detect(window, window_kinds, snapshots);
        if let Some(audit) = audit {
            self.audit_dispatch(window, audit, &matches);
        }
        // The registry's leg (§4.6), which has no shortcut for the audit to
        // check: every entry reading a kind of the window is asked.
        let delayed = self.detect_delayed(window, window_kinds);
        let due = self.take_returns_due(window);
        if matches.is_empty() && delayed.is_empty() && due.is_empty() {
            return Ok(());
        }
        self.queue_matches(matches, delayed, ctx)?;
        // CR 610.3 — "immediately after the specified event", and no stack.
        self.return_until(due, ctx)
    }

    /// Steps 1 to 3 of `dispatch_inner`'s five, for the objects' abilities:
    /// the gate and the match, read-only. The registry's leg is
    /// `detect_delayed`.
    fn detect(
        &self,
        window: &[EventId],
        window_kinds: EventKindMask,
        snapshots: &[LookBackSnapshot],
    ) -> Vec<MatchedTrigger> {
        // --- The gate: six probes -------------------------------------------
        if window_kinds.is_empty() {
            return Vec::new();
        }
        let unattributed = self.continuous_effects.summary().unattributed_trigger_zones;
        let readers = self.battlefield_readers(window_kinds);
        let named = self.named_trigger_carriers(window_kinds);
        let newcomers = self.newcomer_trigger_carriers(window);
        let any_frame_source = window.iter().any(|seq| {
            self.events.record(*seq).is_some_and(|r| {
                r.event.lki().is_some_and(|(lki, ..)| lki.chars().abilities.iter().any(is_triggered))
            })
        });
        // The battlefield probe and the named rows read the mask. The zone
        // map is keyed by ability, and a `Filter` row's zones and the departed
        // leg read lists no registration saw, so they keep their whole walk.
        if readers.is_empty()
            && named.is_empty()
            && self.zone_trigger_sources.is_empty()
            && newcomers.is_empty()
            && unattributed.is_empty()
            && !any_frame_source
            && snapshots.is_empty()
        {
            return Vec::new();
        }
        self.find_matches(window, readers, named, newcomers, unattributed, snapshots)
    }

    /// Steps 4 and 5, for the matches the two legs found: the objects', then
    /// the registry's.
    fn queue_matches(
        &mut self,
        matches: Vec<MatchedTrigger>,
        delayed: Vec<DelayedMatch>,
        ctx: Option<&ActionContext>,
    ) -> Result<(), String> {
        // --- Queue, or resolve a mana trigger at once (CR 605.4a) ----------
        let mut queued: Vec<(TriggerSeq, TriggerOrigin, PlayerId, EventId)> = Vec::new();
        for m in matches {
            let caused_by = m.triggering_events[0].record.seq;
            // "Triggers only once each turn" is written as the ability
            // queues, so a second match in this same window finds it taken.
            if m.def.limit == Some(TriggerLimit::TriggersOnlyOnceEachTurn) && !self.history.triggered_this_turn.insert(m.identity) {
                continue;
            }
            let seq = TriggerSeq(self.next_trigger_seq);
            self.next_trigger_seq += 1;
            let source_found_in_graveyard = self.aura_put_into_graveyard_beside_its_host(&m);
            let binding = TriggerBinding {
                def: Arc::clone(&m.def),
                triggering_events: m.triggering_events,
                event: m.event,
                triggered_by: None,
                referred: Referred::default(),
                source_found_in_graveyard,
            };
            let origin = TriggerOrigin::Object(m.identity);
            let pending = PendingTrigger {
                seq,
                origin,
                controller: m.controller,
                source_card: Arc::clone(&m.source_card),
                instances: m.instances.clone(),
                binding,
                is_state_trigger: false,
                kept_lki: m.source_lki.into_iter().collect(),
                x_value: None,
            };
            if m.mana {
                match ctx {
                    Some(ctx) => self.resolve_triggered_mana(&pending, ctx)?,
                    // Unreachable: `ManaAdded` is always performed inside a
                    // batch, and every activation is announced with its
                    // provider. Loud in debug; queued in release, so the mana
                    // is late rather than lost.
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
        self.queue_delayed(delayed, ctx, &mut queued);

        // --- CR 603.3b's second tier watches this record (§4.8) -------------
        // Emitted after the window has closed, unstamped, each dispatched as
        // it is emitted like any unbatched record.
        for (seq, origin, controller, caused_by) in queued {
            self.emit_event_unbatched(
                GameEvent::AbilityTriggered { seq, origin, controller, caused_by },
                EventCause::GameRule(GameRule::AbilityTriggers),
            )?;
        }
        Ok(())
    }

    /// CR 400.7f's first sentence: an Aura put into its owner's graveyard in
    /// the event its enchanted permanent left the battlefield in, which its
    /// trigger on that leaving (CR 603.6e) finds there. Read off the window,
    /// which still holds the record of the Aura going to the graveyard as the
    /// trigger is queued.
    fn aura_put_into_graveyard_beside_its_host(&self, matched: &MatchedTrigger) -> Option<ObjectRef> {
        let aura = matched.identity.source;
        if !matched.def.condition.events()[matched.event.0].watches_its_host_leave() || self.find(aura).is_some() {
            return None;
        }
        // `aura-found-in-graveyard`'s switch (`engineering-practices.md`
        // §3.4b), with the CR 704.5m hook's: the graveyard only.
        self.events.held().iter().find_map(|record| match &record.event {
            GameEvent::ZoneChange { to: Zone::Graveyard, lki: Some(lki), .. } if lki.object() == aura => record.event.new_object(),
            _ => None,
        })
    }

    /// CR 400.7f's second sentence: CR 704.5m has put `aura`, the existence
    /// that was on the battlefield, into its owner's graveyard (and not, by a
    /// replacement, anywhere else), and each of its
    /// waiting triggers on its enchanted permanent leaving finds it there. They
    /// are pending, or on the stack at the latest: the check follows the
    /// leaving before anyone receives priority.
    pub(crate) fn aura_put_into_graveyard_by_the_check(&mut self, aura: ObjectRef) {
        let Some(found) = self.object_ref(aura.id) else { return };
        let finds = |source: ObjectRef, binding: &mut TriggerBinding| {
            if source == aura && binding.event().watches_its_host_leave() && binding.source_found_in_graveyard.is_none() {
                binding.source_found_in_graveyard = Some(found);
            }
        };
        for pending in &mut self.pending_triggers {
            finds(pending.origin.source_ref(), &mut pending.binding);
        }
        for id in &self.stack {
            let Some(entry) = self.stack_entries.get_mut(id) else { continue };
            if let (Some(identity), Some(binding)) = (entry.ability_identity, entry.trigger.as_mut()) {
                finds(identity.source, binding);
            }
        }
    }

    /// The permanents whose printed triggers, or the copy's they entered as,
    /// read a kind `window_kinds` carries, selected before they are ordered,
    /// in CR 613.7 order: sorted on
    /// the key `battlefield_ids_ordered` sorts on, so the candidate order —
    /// which the `OrderTriggers` prompt offers — is the whole-battlefield
    /// walk's.
    fn battlefield_readers(&self, window_kinds: EventKindMask) -> Vec<ObjectId> {
        // ORDER-FREE: sorted by timestamp next.
        let mut readers: Vec<(Timestamp, ObjectId)> = self
            .trigger_sources
            .iter()
            .filter(|(_, kinds)| kinds.intersects(window_kinds))
            .filter_map(|(&id, _)| self.battlefield.get(&id).map(|entry| (entry.timestamp, id)))
            .collect();
        readers.sort_unstable_by_key(|&(timestamp, _)| timestamp);
        readers.into_iter().map(|(_, id)| id).collect()
    }

    /// The objects that have a triggered ability only because a continuous
    /// effect names them, and whose ability could read this window.
    ///
    /// A permanent with a *printed* triggered ability is filed in
    /// `trigger_sources` as it enters, under the kinds of event it reads, and
    /// `battlefield_readers` asks that file. Two effects give an object a
    /// triggered ability it did not print, and no filing sees them: a copy
    /// (Cryptoplasm becoming a copy of a creature and keeping its own
    /// upkeep trigger) and a grant (an Aura's "enchanted creature has
    /// 'whenever this creature deals damage, ...'"). Each such effect is a
    /// registry row. A row that names its objects — its own source, a fixed
    /// list, or its source's host — is read here: its objects are
    /// candidates, wherever they are, when a triggered ability it puts on
    /// them reads a kind of event the window carries (item 222). A row over
    /// a filter names zones instead, and `candidates_now` walks them.
    ///
    /// Unordered: `candidates_now` puts every candidate in CR 613.7 order.
    fn named_trigger_carriers(&self, window_kinds: EventKindMask) -> Vec<ObjectId> {
        if !self.continuous_effects.summary().named_unattributed_trigger_kinds.intersects(window_kinds) {
            return Vec::new();
        }
        let mut named: Vec<ObjectId> = Vec::new();
        for row in self.continuous_effects.iter() {
            if !triggered_ability_kinds(row).intersects(window_kinds) {
                continue;
            }
            match &row.affected_objects {
                ObjectSet::SourceOnly => named.push(row.source),
                ObjectSet::Fixed(ids) => named.extend(ids.iter().copied()),
                // A host is a permanent (CR 301.5, 303.4).
                ObjectSet::Host => named.extend(crate::oracle::board::host_of(self, row.source)),
                ObjectSet::Filter { .. } => {}
            }
        }
        named
    }

    /// The window's newcomers on `newcomer_trigger_sources`: objects off the
    /// battlefield whose every functioning trigger is on themselves moving,
    /// which no window but the one that brought them where they are can
    /// trigger. Empty, and no record read, while the set is.
    fn newcomer_trigger_carriers(&self, window: &[EventId]) -> Vec<ObjectId> {
        if self.newcomer_trigger_sources.is_empty() {
            return Vec::new();
        }
        window
            .iter()
            .filter_map(|seq| self.events.record(*seq))
            .filter_map(|record| self.newcomer_from(&record.event))
            .filter(|newcomer| self.find(*newcomer).is_some() && self.newcomer_trigger_sources.contains(&newcomer.id))
            .map(|newcomer| newcomer.id)
            .collect()
    }

    /// Whether performing these decided actions can change which triggered
    /// abilities a *surviving* object has: true when one of them takes off
    /// the battlefield a permanent that is the source of a copy effect, or of
    /// an effect that grants or removes abilities. A player losing counts for
    /// each such permanent that goes with them (`departs_with`, CR 800.4a).
    /// So does an attachment that is such a source leaving the object it is
    /// on, which takes what it gave that object (CR 603.10c: "becomes
    /// unattached" looks back). When it is true the batch snapshots the
    /// lists before performing (`LookBackSnapshot`).
    pub(crate) fn departs_an_ability_list_source<'a>(&self, mut decided: impl Iterator<Item = &'a GameAction>) -> bool {
        let sources = &self.continuous_effects.summary().ability_list_sources;
        let attached = |id: &ObjectId| self.battlefield.get(id).is_some_and(|entry| entry.attached_to.is_some());
        !sources.is_empty()
            && decided.any(|action| match action {
                GameAction::ZoneChange { object, from: Zone::Battlefield, .. }
                | GameAction::Destroy { object, .. } => sources.contains(object),
                GameAction::PlayerLoses { player } => sources.iter().any(|&s| self.departs_with(s, *player)),
                GameAction::Unattach { attachment } | GameAction::Attach { attachment, .. } => {
                    sources.contains(attachment) && attached(attachment)
                }
                _ => false,
            })
    }

    /// CR 603.10a before the batch performs: frame each object `decided`
    /// moves in one of its three classes (`zone_change_looks_back`), in batch
    /// order: a permanent leaving the battlefield, a card leaving a graveyard,
    /// a visible object put into a hand or library. A player leaving takes
    /// the permanents that go with them (`departs_with`): CR 800.4a's fourth
    /// clause exiles only after its first two have run, so the set is
    /// everything it may exile. A permanent's frame is a memo hit, since
    /// nothing has changed since the batch began deciding.
    ///
    /// **And every other mover a keeper reads, from whatever zone** (CR
    /// 113.7a, 608.2h; `triggers-architecture.md` §6.1): a spell a trigger's
    /// triggering event is about, a card a draw trigger named. The move hands
    /// the frame to each keeper that reads the mover (`hand_over_lki`).
    pub(crate) fn capture_lki<'a>(&mut self, decided: impl Iterator<Item = &'a GameAction>) {
        let mut kept_for: Option<Vec<ObjectRef>> = None;
        for action in decided {
            match action {
                GameAction::ZoneChange { object, from: Zone::Battlefield, .. }
                | GameAction::Destroy { object, .. } => {
                    if self.battlefield.contains_key(object) {
                        self.capture_frame(*object);
                    }
                }
                // A replacement keeping the object where it was (CR 614.6): the
                // record of the act it replaced carries the object as it was.
                GameAction::ZoneChange { object, from, to, .. } if zone_change_looks_back(*from, Some(*to)) || from == to => {
                    self.capture_frame(*object);
                }
                GameAction::EnterBattlefield { object, from: Some(from), .. }
                    if zone_change_looks_back(*from, Some(Zone::Battlefield)) =>
                {
                    self.capture_frame(*object);
                }
                GameAction::ZoneChange { object, .. } => {
                    let kept_for = kept_for.get_or_insert_with(|| self.objects_lki_is_kept_for());
                    if self.object_ref(*object).is_some_and(|r| kept_for.contains(&r)) {
                        self.capture_frame(*object);
                    }
                }
                GameAction::PlayerLoses { player } => {
                    for id in self.battlefield_ids_ordered() {
                        if self.departs_with(id, *player) {
                            self.capture_frame(id);
                        }
                    }
                    let kept_for = kept_for.get_or_insert_with(|| self.objects_lki_is_kept_for());
                    for object in kept_for.iter() {
                        if self.departs_with(object.id, *player) {
                            self.capture_frame(object.id);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// The objects some keeper keeps last known information for
    /// (`for_each_lki_keeper`), each once. Empty or short on the common board.
    fn objects_lki_is_kept_for(&mut self) -> Vec<ObjectRef> {
        let mut kept_for: Vec<ObjectRef> = Vec::new();
        self.for_each_lki_keeper(|keeper, _| {
            for object in keeper.objects() {
                if !kept_for.contains(&object) {
                    kept_for.push(object);
                }
            }
        });
        kept_for
    }

    /// Every keeper, with where it keeps the frame of an object it reads that
    /// leaves. The one list of keepers, so the capture
    /// (`objects_lki_is_kept_for`) and the hand-over cannot disagree.
    fn for_each_lki_keeper(&mut self, mut each: impl FnMut(LkiKeeper<'_>, KeptLkiSlot<'_>)) {
        for pending in &mut self.pending_triggers {
            let keeper = LkiKeeper { source: Some(pending.origin.source_ref()), trigger: Some(&pending.binding) };
            each(keeper, KeptLkiSlot::List(&mut pending.kept_lki));
        }
        // In stack order, never the map's: a capture is a layer walk, which
        // writes a trace record.
        for id in &self.stack {
            let Some(entry) = self.stack_entries.get_mut(id) else { continue };
            let keeper = LkiKeeper { source: entry.ability_identity.map(|identity| identity.source), trigger: entry.trigger.as_ref() };
            each(keeper, KeptLkiSlot::List(&mut entry.kept_lki));
        }
        if let Some(resolving) = &mut self.resolving {
            let keeper = LkiKeeper { source: resolving.identity.map(|identity| identity.source), trigger: resolving.trigger.as_ref() };
            each(keeper, KeptLkiSlot::List(&mut resolving.kept_lki));
        }
        for delayed in &mut self.delayed_triggers {
            each(LkiKeeper { source: Some(delayed.source), trigger: None }, KeptLkiSlot::Source(&mut delayed.source_lki));
        }
    }

    /// A mover's frame, taken before its batch performs (CR 603.10a for a
    /// permanent, 113.7a for anything a keeper reads). One an enclosing batch
    /// already framed keeps that frame, since a destruction's move is a
    /// nested batch and the event it belongs to is the outer one.
    fn capture_frame(&mut self, id: ObjectId) {
        let Some(object) = self.object_ref(id) else { return };
        if self.capture_of(object).is_some() {
            return;
        }
        if let Some(lki) = self.lki_now(id) {
            self.lki_captures.push(LkiCapture { object, lki: Some(lki) });
        }
    }

    /// `id`'s last known information, taken now, before it leaves (CR
    /// 113.7a): `frame`, its characteristics, with the facts about it no layer
    /// computes. There is one answer; the caller passes the frame only
    /// because where it may be read from depends on when. A batch frames its
    /// movers off the layer memo before it performs, and the fallback in
    /// `take_captured_lki`, reached while one performs, may not read the memo.
    pub(crate) fn last_known_information(&self, id: ObjectId, frame: Arc<EffectiveCharacteristics>) -> Option<Lki> {
        let object = self.objects.get(&id)?;
        let cost_choices = self.cost_choices_of(id).unwrap_or(CostChoices::NONE);
        let existence = ObjectRef { id, zone_change_epoch: object.zone_change_epoch };
        Some(Lki::new(existence, frame, object.owner, cost_choices, self.battlefield.get(&id), self.stack_entries.get(&id)))
    }

    /// `id`'s last known information with its frame off the layer memo, for
    /// every reader but a performing batch's fallback.
    pub(crate) fn lki_now(&self, id: ObjectId) -> Option<Lki> {
        self.last_known_information(id, compute_characteristics(self, id)?)
    }

    /// The capture its batch took of `object`, if one did.
    fn capture_of(&mut self, object: ObjectRef) -> Option<&mut LkiCapture> {
        self.lki_captures.iter_mut().find(|d| d.object == object)
    }

    /// CR 603.10a, 113.7a — what `id` hands over as it leaves `from` for `to`
    /// (`None`: the game), before it moves: the frame its batch took, to every
    /// keeper that reads it.
    /// Returns the frame its record carries, for a move in one of CR 603.10a's
    /// classes; a token whose entry is still being decided has no entity and
    /// nothing to look back at (`create_tokens`).
    pub(crate) fn hand_over_before_leaving(&mut self, id: ObjectId, from: Zone, to: Option<Zone>) -> Option<Lki> {
        let entering_token = from == Zone::Battlefield && !self.battlefield.contains_key(&id);
        let lki = if zone_change_looks_back(from, to) && !entering_token { self.take_captured_lki(id, from) } else { None };
        self.hand_over_lki(id, lki.as_ref());
        self.note_delayed_source_moving(id);
        lki
    }

    /// CR 113.7a, 608.2h — `id` is leaving the zone a keeper expected it in:
    /// each keeper that reads it keeps the frame its batch took. That is
    /// `lki`, the record's own, for a permanent, and for any other mover the
    /// one `capture_lki` took because a keeper reads it. One writer; its
    /// callers are the two performers that
    /// move an object out of a zone, before the move.
    fn hand_over_lki(&mut self, id: ObjectId, lki: Option<&Lki>) {
        let lki = match lki {
            Some(lki) => lki.clone(),
            None => match self.take_named_lki(id) {
                Some(lki) => lki,
                None => return,
            },
        };
        let object = lki.object();
        self.for_each_lki_keeper(|keeper, slot| {
            if !keeper.objects().any(|kept_for| kept_for == object) {
                return;
            }
            match slot {
                KeptLkiSlot::List(list) => list.push(lki.clone()),
                KeptLkiSlot::Source(source_lki) => *source_lki = Some(lki.clone()),
            }
        });
    }

    /// The last known information a kept-for mover's batch took for it, if any:
    /// the counterpart of `take_captured_lki` for a move in no CR 603.10a
    /// class, which no batch is bound to frame.
    fn take_named_lki(&mut self, id: ObjectId) -> Option<Lki> {
        let object = self.object_ref(id)?;
        self.capture_of(object)?.lki.take()
    }

    /// The last known information the move of `id` out of `from` carries,
    /// taken before its batch performed: a move in one of CR 603.10a's
    /// classes, or one a replacement kept in `from` (`GameEvent::SameZoneMove`).
    pub(crate) fn take_captured_lki(&mut self, id: ObjectId, from: Zone) -> Option<Lki> {
        let taken = self.take_named_lki(id);
        // Every such move is decided by a batch, which framed it first.
        debug_assert!(taken.is_some(), "{id} moved from the {from:?} with no frame from its batch");
        taken.or_else(|| {
            let frame = crate::engine::layers::compute::compute_characteristics_uncached(self, id)?;
            self.last_known_information(id, Arc::new(frame))
        })
    }

    /// The objects the dispatch at the window's close could ask a look-back
    /// trigger of, each with the existence it has now and its frame: every
    /// permanent that printed a triggered ability, the cards off the
    /// battlefield whose triggered ability works in the zone they are in,
    /// and, while an effect grants or copies a triggered ability, the objects
    /// it names or every object in the zones it reaches. Read before the
    /// batch performs, since the departure about to happen may end that very
    /// effect. The frames are memo hits: nothing has changed since the batch
    /// began deciding.
    pub(crate) fn look_back_frames(&self) -> Vec<ObjectSnapshot> {
        let unattributed = self.continuous_effects.summary().unattributed_trigger_zones;
        // Every printed source, not the ones whose kinds a look-back arm
        // reads: which arms look back is `TriggerEvent::looks_back`'s to say,
        // and a kind filter here would be a second table of it.
        let named = self.named_trigger_carriers(EventKindMask::ALL);
        // No newcomer's trigger looks back but off its own move's frame.
        self.candidates_now(self.battlefield_readers(EventKindMask::ALL), named, Vec::new(), unattributed)
            .into_iter()
            .filter_map(|id| self.object_snapshot(id))
            .collect()
    }

    /// `id` as it is now, and where it is.
    pub(crate) fn object_snapshot(&self, id: ObjectId) -> Option<ObjectSnapshot> {
        let lki = self.lki_now(id)?;
        Some(ObjectSnapshot { zone: self.objects.get(&id)?.zone, lki })
    }

    /// The objects a dispatch asks with their ability lists as they are now,
    /// as against the lists from before an event (the CR 603.10a frame of an
    /// object that left, a survivor's `LookBackSnapshot`), in CR 613.7 order.
    ///
    /// Every zone, by three routes: on the battlefield, `readers` (printed
    /// triggers) and the `named` objects there, or every permanent while a
    /// `Filter` row grants or copies a triggered ability onto the
    /// battlefield; elsewhere, every object whose printed triggered ability
    /// works in the zone it is in (CR 113.6k: a graveyard's, a hand's, exile's
    /// — a record's own subject in a graveyard is one), the window's
    /// `newcomers` whose every such trigger is on themselves moving, the
    /// `named` objects there, and every object in the zones a `Filter` row
    /// reaches.
    fn candidates_now(&self, readers: Vec<ObjectId>, named: Vec<ObjectId>, newcomers: Vec<ObjectId>, unattributed: ZoneSet) -> Vec<ObjectId> {
        let mut live: Vec<ObjectId> = if unattributed.contains(Zone::Battlefield) {
            self.battlefield_ids_ordered()
        } else if named.is_empty() {
            readers
        } else {
            let mut on_battlefield: Vec<(Timestamp, ObjectId)> = readers
                .iter()
                .chain(&named)
                .filter_map(|&id| self.battlefield.get(&id).map(|entry| (entry.timestamp, id)))
                .collect();
            on_battlefield.sort_unstable_by_key(|&(timestamp, _)| timestamp);
            on_battlefield.dedup_by_key(|&mut (_, id)| id);
            on_battlefield.into_iter().map(|(_, id)| id).collect()
        };
        // ORDER-FREE: sorted by timestamp below.
        let mut elsewhere: Vec<(Timestamp, ObjectId)> = self
            .zone_trigger_sources
            .keys()
            .map(|&id| (self.object_timestamp(id), id))
            .collect();
        let mut seen: IdSet<ObjectId> = elsewhere.iter().map(|&(_, id)| id).collect();
        for id in newcomers {
            if seen.insert(id) {
                elsewhere.push((self.object_timestamp(id), id));
            }
        }
        let named_elsewhere = named.iter().copied().filter(|id| !self.battlefield.contains_key(id) && self.objects.contains_key(id));
        for id in named_elsewhere {
            if seen.insert(id) {
                elsewhere.push((self.object_timestamp(id), id));
            }
        }
        for zone in unattributed.beyond_battlefield().iter() {
            for id in self.zone_ids_ordered(zone) {
                if seen.insert(id) {
                    elsewhere.push((self.object_timestamp(id), id));
                }
            }
        }
        elsewhere.sort_unstable_by_key(|&(timestamp, _)| timestamp);
        live.extend(elsewhere.into_iter().map(|(_, id)| id));
        live
    }

    /// The matcher's read-only half: the dispatcher's candidates for the
    /// window — its four legs and the survivors' lists from before — asked
    /// by [`Self::match_candidates`].
    fn find_matches(
        &self,
        window: &[EventId],
        readers: Vec<ObjectId>,
        named: Vec<ObjectId>,
        newcomers: Vec<ObjectId>,
        unattributed: ZoneSet,
        snapshots: &[LookBackSnapshot],
    ) -> Vec<MatchedTrigger> {
        let mut live = self.candidates_now(readers, named, newcomers, unattributed);

        // Each survivor's lists from before the window's snapshotting batches.
        // A survivor no live set reaches now, like a grant's carrier after
        // the granter left, is still asked, after the rest.
        let mut before: IdMap<ObjectId, Vec<(usize, &EffectiveCharacteristics)>> = IdMap::default();
        for (k, snapshot) in snapshots.iter().enumerate() {
            for frame in &snapshot.frames {
                let object = frame.lki.object();
                if self.find(object).is_some() {
                    before.entry(object.id).or_default().push((k, frame.lki.chars()));
                }
            }
        }
        if !before.is_empty() {
            let mut reached: IdSet<ObjectId> = live.iter().copied().collect();
            for snapshot in snapshots {
                for frame in &snapshot.frames {
                    let id = frame.lki.object().id;
                    if before.contains_key(&id) && reached.insert(id) {
                        live.push(id);
                    }
                }
            }
        }

        let records: Vec<(EventId, &EventRecord)> = window
            .iter()
            .filter_map(|seq| self.events.record(*seq).map(|r| (*seq, r)))
            .collect();

        // Leg 2: the frames the window carries (CR 603.10a) — the list each
        // object that left had, asked look-back conditions only.
        let frames: Vec<(&Lki, Zone)> = records.iter().filter_map(|(_, r)| r.event.lki()).map(|(lki, from, _)| (lki, from)).collect();

        let mut candidates: Vec<TriggerCandidate<'_>> = Vec::new();
        for id in live {
            let Some(object) = self.objects.get(&id) else { continue };
            let Some(chars) = compute_characteristics(self, id) else { continue };
            let existence = ObjectRef { id, zone_change_epoch: object.zone_change_epoch };
            let earlier = before.remove(&id).unwrap_or_default();
            let controller = controller_or_owner(self, id).unwrap_or(object.owner);
            let host = crate::oracle::board::host_of(self, id);
            let snapshots = earlier.iter().map(|&(k, _)| k).collect();
            // The lists from before sit right behind the list now, so an
            // object's triggers keep the order a single candidate gave them.
            let before_lists = earlier
                .into_iter()
                .map(|(snapshot, chars)| TriggerCandidateFrame::Before { chars, snapshot });
            let lists = std::iter::once(TriggerCandidateFrame::Live { chars, snapshots }).chain(before_lists);
            for frame in lists {
                // A list from before the event is its controller's then (CR
                // 603.10, 603.3a), which a control change made someone else.
                let controller = match &frame {
                    TriggerCandidateFrame::Before { chars, .. } => chars.controller,
                    TriggerCandidateFrame::Live { .. } | TriggerCandidateFrame::LastKnown { .. } => controller,
                };
                candidates.push(TriggerCandidate {
                    object: existence,
                    controller,
                    owner: object.owner,
                    zone: object.zone,
                    host,
                    frame,
                    card: Arc::clone(&object.card_data),
                });
            }
        }
        for (lki, from) in frames {
            // The card of an object that left: in the store for a zone change,
            // gone for a `LeftTheGame` — then the frame's own list is all
            // there is, and the stack object is built from a card that names
            // what the frame does.
            let card = match self.objects.get(&lki.object().id) {
                Some(object) => Arc::clone(&object.card_data),
                None => Arc::new(frame_card(lki.chars())),
            };
            candidates.push(TriggerCandidate {
                object: lki.object(),
                controller: lki.chars().controller,
                owner: lki.owner(),
                zone: from,
                host: lki.host(),
                frame: TriggerCandidateFrame::LastKnown { lki, snapshot: None },
                card,
            });
        }

        let matches = self.match_candidates(&records, &candidates, snapshots);
        self.diagnostics.record_trigger_dispatch(candidates.len() as u64, matches.len() as u64);
        matches
    }

    /// Every candidate ability against every record of the window, in window
    /// order then candidate order — the order the `OrderTriggers` prompt will
    /// offer, which has to be process-stable end to end (§15 item 1). Which
    /// candidates there are is the caller's: the dispatcher's legs, or the
    /// audit's every object (`audit.rs`).
    pub(super) fn match_candidates(
        &self,
        records: &[(EventId, &EventRecord)],
        candidates: &[TriggerCandidate<'_>],
        snapshots: &[LookBackSnapshot],
    ) -> Vec<MatchedTrigger> {
        let mut matches: Vec<MatchedTrigger> = Vec::new();
        // "One or more" accumulates across the window: (identity, event, and
        // for a once-per-subject arm the subject) -> index into `matches`.
        let mut once: Vec<((AbilityIdentity, EventIndex, Option<CounterSubject>), usize)> = Vec::new();

        // The window's newcomers: a window is one event (§4.1), so each was
        // somewhere else immediately before every record of it.
        let newcomers: Vec<ObjectRef> = records.iter().filter_map(|(_, record)| self.newcomer_from(&record.event)).collect();
        let is_newcomer: Vec<bool> = candidates.iter().map(|candidate| newcomers.contains(&candidate.object)).collect();

        // One row per triggered ability, so the records loop below pays
        // only for what depends on the record (§4.2).
        let mut defs: Vec<TriggerCandidateDef<'_>> = Vec::new();
        for (index, candidate) in candidates.iter().enumerate() {
            let chars = candidate.frame.chars();
            for ability in chars.abilities.iter() {
                let Effect::Triggered(def) = &ability.effect else { continue };
                // CR 113.6 is asked of each ability: the object is here
                // because *some* ability of its functions here (§4.2).
                if !functions_in(ability, &chars.types, candidate.zone) {
                    continue;
                }
                defs.push(TriggerCandidateDef {
                    candidate: index,
                    identity: AbilityIdentity { source: candidate.object, ability: ability.id },
                    def,
                    instances: &ability.instances,
                });
            }
        }

        for (seq, record) in records {
            let looks_back_through = LookBackSnapshot::for_record(snapshots, seq.0);
            // CR 603.2c: an ability triggers once each time its trigger event
            // occurs. A survivor is asked through two lists and can match one
            // record through each; the first arm in the ability's order wins.
            let mut hits: Vec<ArmMatch<'_, '_>> = Vec::new();
            for row in &defs {
                let candidate = &candidates[row.candidate];
                let Some(answered) = candidate.frame.answered_arms(looks_back_through, is_newcomer[row.candidate]) else { continue };
                let identity = row.identity;
                let def = row.def;
                let outcome = self.match_def(def, candidate, answered, record, identity);
                let (matched, triggering_events, refusal) = match outcome {
                    Ok((event, triggering_events)) => (Some(event), triggering_events, None),
                    Err(refusal) => (None, Vec::new(), Some(refusal)),
                };
                let mana = matched.is_some() && is_mana_ability(def, &record.event);
                self.trace(|| {
                    trace_records::trigger(
                        self,
                        *seq,
                        &identity,
                        candidate.zone,
                        matched.is_some(),
                        refusal.map(Refusal::name),
                        mana,
                    )
                });
                let Some(arm) = matched else { continue };
                let this = ArmMatch { row, arm, triggering_events };
                match hits.iter_mut().find(|hit| hit.row.identity == identity) {
                    Some(earlier) if this.arm < earlier.arm => *earlier = this,
                    Some(_) => {}
                    None => hits.push(this),
                }
            }
            for ArmMatch { row, arm: matched, triggering_events } in hits {
                let candidate = &candidates[row.candidate];
                let (identity, def) = (row.identity, row.def);
                let mana = is_mana_ability(def, &record.event);
                let arm = &def.condition.events()[matched.0];
                // The ability's own def, shared: nothing writes it once the card
                // is built, so a Humility landing before placement still cannot
                // un-trigger it (CR 113.7a).
                let def_arc: Arc<TriggerDef> = Arc::clone(def);
                match arm.multiplicity() {
                    Multiplicity::PerOccurrence => {
                        for objects in triggering_events {
                            matches.push(MatchedTrigger {
                                identity,
                                controller: candidate.controller,
                                def: Arc::clone(&def_arc),
                                source_card: Arc::clone(&candidate.card),
                                instances: row.instances.to_vec(),
                                event: matched,
                                triggering_events: vec![TriggeringEvent { record: (*record).clone(), objects }],
                                mana,
                                source_lki: candidate.frame.lki().cloned(),
                            });
                        }
                    }
                    // CR 603.2c's boundary is the window: one trigger, every
                    // matching event in its binding, no one object; or one for
                    // each subject the window's records are about (§4.4).
                    how @ (Multiplicity::OncePerEvent | Multiplicity::OncePerSubject) => {
                        let subject = match how {
                            Multiplicity::OncePerSubject => arm.occurrence_subject(&record.event),
                            Multiplicity::PerOccurrence | Multiplicity::OncePerEvent => None,
                        };
                        let these = triggering_events.into_iter().map(|objects| TriggeringEvent { record: (*record).clone(), objects });
                        match once.iter().find(|((i, e, s), _)| *i == identity && *e == matched && *s == subject) {
                            Some((_, at)) => matches[*at].triggering_events.extend(these),
                            None => {
                                once.push(((identity, matched, subject), matches.len()));
                                matches.push(MatchedTrigger {
                                    identity,
                                    controller: candidate.controller,
                                    def: Arc::clone(&def_arc),
                                    source_card: Arc::clone(&candidate.card),
                                    instances: row.instances.to_vec(),
                                    event: matched,
                                    triggering_events: these.collect(),
                                    mana,
                                    source_lki: candidate.frame.lki().cloned(),
                                });
                            }
                        }
                    }
                }
            }
        }
        matches
    }

    /// One def against one record: the first arm that matches, and the
    /// objects of its triggering events (one per trigger). `Err` names the
    /// predicate that refused.
    fn match_def(
        &self,
        def: &TriggerDef,
        candidate: &TriggerCandidate<'_>,
        answered: AnsweredArms,
        record: &EventRecord,
        identity: AbilityIdentity,
    ) -> Result<(EventIndex, Vec<TriggeringEventObjects>), Refusal> {
        let event = &record.event;
        if matches!(def.condition, TriggerCondition::State(_)) {
            return Err(Refusal::StateTrigger);
        }
        if !self.visible_as_read(candidate) {
            return Err(Refusal::Visibility);
        }
        let referents = TriggerReferents {
            this: ThisObject::Candidate(candidate.object.id),
            controller: candidate.controller,
            owner: candidate.owner,
            host: candidate.host,
            this_ability: Some(identity),
            referred: &[],
        };
        let mut matched: Option<(EventIndex, Vec<TriggeringEventObjects>)> = None;
        for (index, arm) in def.condition.events().iter().enumerate() {
            match answered {
                AnsweredArms::LookBackOnly if !arm.looks_back() => continue,
                AnsweredArms::AllButLookBack if arm.looks_back() => continue,
                AnsweredArms::All | AnsweredArms::LookBackOnly | AnsweredArms::AllButLookBack => {}
            }
            if !arm.reads(event) {
                continue;
            }
            // CR 113.6k asks each trigger condition where it functions; the
            // pre-pass asked the ability, which is their union.
            if !condition_functions_in(def, arm, &candidate.frame.chars().types, candidate.zone) {
                continue;
            }
            let triggering_events = self.triggering_events_in(arm, &referents, record);
            if !triggering_events.is_empty() {
                matched = Some((EventIndex(index), triggering_events));
                break;
            }
        }
        let (event_index, triggering_events) = matched.ok_or(Refusal::TriggerCondition)?;
        let arm = &def.condition.events()[event_index.0];
        if !self.within_once_per_turn_limit(def, identity, candidate.controller, arm, record, &referents) {
            return Err(Refusal::Limit);
        }
        // CR 603.4 at the trigger. "You" is the candidate's controller (CR
        // 109.5): the source's now, or its frame's for a look-back candidate,
        // and "this" is read as it left for one.
        let source = TriggerSource { object: candidate.object.id, controller: candidate.controller, lki: candidate.frame.lki() };
        let triggering_events = self.triggers_whose_intervening_if_holds(def, arm, record, triggering_events, source)?;
        Ok((event_index, triggering_events))
    }

    /// CR 603.2f, asked at the instant the candidate's list is read: a
    /// frame's immediately before the event (CR 603.10), in the zone the
    /// object left, and any other list after it, where the object is. Only a
    /// move in one of CR 603.10a's classes carries a frame, and each leaves
    /// a public zone, so a frame always answers yes: a creature returned to
    /// its owner's hidden hand still triggers its leaves-the-battlefield
    /// ability.
    fn visible_as_read(&self, candidate: &TriggerCandidate<'_>) -> bool {
        match candidate.frame {
            TriggerCandidateFrame::LastKnown { .. } => candidate.zone.is_public(),
            TriggerCandidateFrame::Live { .. } | TriggerCandidateFrame::Before { .. } => {
                visible_to_all(self, candidate.object.id)
            }
        }
    }

    /// CR 603.4's first check, when the event happens: of the triggers one
    /// event makes — one per triggering event, as two attackers make two — the ones
    /// whose intervening "if" holds, each asked about its own objects exactly
    /// as they will be again at resolution. A "one or more" trigger is one
    /// trigger with no single triggering event. Refused when none holds.
    pub(super) fn triggers_whose_intervening_if_holds(
        &self,
        def: &TriggerDef,
        arm: &TriggerEvent,
        record: &EventRecord,
        triggering_events: Vec<TriggeringEventObjects>,
        source: TriggerSource<'_>,
    ) -> Result<Vec<TriggeringEventObjects>, Refusal> {
        let Some(condition) = &def.intervening_if else { return Ok(triggering_events) };
        let holding: Vec<TriggeringEventObjects> = triggering_events
            .into_iter()
            .filter(|objects| {
                let triggering_event = match arm.multiplicity() {
                    Multiplicity::PerOccurrence | Multiplicity::OncePerSubject => {
                        TriggeringEventView::AtTrigger { arm, record, objects }
                    }
                    Multiplicity::OncePerEvent => TriggeringEventView::None,
                };
                let referents = ConditionReferents { source_lki: source.lki, triggering_event };
                settled_holds(condition, self, source.object, Some(source.controller), referents)
            })
            .collect();
        if holding.is_empty() { Err(Refusal::InterveningIf) } else { Ok(holding) }
    }

    /// The once-per-turn limits (§3.5), each read at the trigger. CR
    /// 603.2h's gate is its source's controller's; "only once each turn" is
    /// the ability's; "the first time" is the record's place in its turn
    /// (`EventRecord::place_in_turn`), or for a targeting, no earlier row of
    /// the turn's that `arm` would also have matched: Heartfire Hero's "if
    /// that creature hasn't yet been the target of a spell or ability you
    /// control this turn".
    pub(super) fn within_once_per_turn_limit(
        &self,
        def: &TriggerDef,
        identity: AbilityIdentity,
        controller: PlayerId,
        arm: &TriggerEvent,
        record: &EventRecord,
        referents: &TriggerReferents<'_>,
    ) -> bool {
        match def.limit {
            None => true,
            Some(TriggerLimit::DoThisOnlyOnceEachTurn) => !self.history.action_taken_this_turn.contains(&(identity, controller)),
            Some(TriggerLimit::TriggersOnlyOnceEachTurn) => !self.history.triggered_this_turn.contains(&identity),
            Some(TriggerLimit::FirstTimeEachTurn) => match (arm, &record.event) {
                (TriggerEvent::BecomesTarget { by, controller: whose, .. }, GameEvent::Targeted { target, .. }) => {
                    !self.history.log.targetings_of_turn(self.turn_number).any(|earlier| {
                        earlier.seq < record.seq
                            && earlier.target == *target
                            && by.admits(earlier.by_spell)
                            && whose.as_ref().is_none_or(|p| self.player_ref_is(p, earlier.controller, referents))
                    })
                }
                _ => record.place_in_turn() == Some(1),
            },
        }
    }

    /// Each event `record` holds that triggers `arm`, as the objects that
    /// event is about, read for the trigger's `referents`: each is one trigger
    /// (CR 603.2c's occurrences). A record holds one for every arm but
    /// `Attacks`, which holds one per matching attacker. Empty when the record
    /// does not match the arm.
    ///
    /// One arm against one record, and every case has the same shape: the
    /// `match` pairs the arm with the record kind it reads, and the arm's own
    /// fields are the predicates over that record. **A field left `None` is
    /// not asked** — every one of them is an `is_none_or` — which is how "any"
    /// is spelled at the type, the convention the replacement patterns share.
    /// `triggering_event_if` wraps a boolean as an event: true is the one
    /// event `triggering_event_objects` reads off the record, false is none.
    /// The only arm that does not go through it is `Attacks`, where CR 508.3a
    /// makes each matching attacker an event of its own.
    pub(super) fn triggering_events_in(
        &self,
        arm: &TriggerEvent,
        referents: &TriggerReferents<'_>,
        record: &EventRecord,
    ) -> Vec<TriggeringEventObjects> {
        let (seq, event) = (record.seq, &record.event);
        let triggering_event_if = |ok: bool| if ok { vec![self.triggering_event_objects(arm, record)] } else { Vec::new() };
        match (arm, event) {
            // The subject is read as the record framed it only for an arm that
            // looks back (CR 603.10a); "from anywhere" reads the object after
            // the move (CR 603.6c's last sentence, 603.10).
            (
                TriggerEvent::ZoneChange { subject, change, by, owner, .. },
                GameEvent::ZoneChange { object_id, owner: moved_owner, from, to, act, lki, applied: _, new_epoch: _ },
            ) => triggering_event_if(
                change.matches(Some(*from), *to, Some(*act))
                    // The resolving spell or ability caused the move, also where a
                    // replacement redirected it (`replacement-architecture.md` §2b.4).
                    && by.as_ref().is_none_or(|by| by.matches(self, &record.stamp.cause, &mut { *referents }))
                    && owner.as_ref().is_none_or(|p| self.player_ref_is(p, *moved_owner, referents))
                    && self.subject_matches(subject, Some(*object_id), referents, seq, SubjectRead::of_departure(arm, lki)),
            ),
            // CR 603.6c names this event for a leaves-the-battlefield ability,
            // and for nothing else: a card leaving the game from a graveyard
            // leaves no graveyard (Erebos's Titan's third ruling). Nothing
            // narrower either — no `to`, no act, no cause.
            (
                TriggerEvent::ZoneChange { subject, change, by, owner, .. },
                GameEvent::LeftTheGame { object_id, owner: moved_owner, from, lki },
            ) => triggering_event_if(
                *from == Zone::Battlefield
                    && change.from == Some(Zone::Battlefield)
                    && change.to.is_none()
                    && change.act.is_none()
                    && by.is_none()
                    && owner.as_ref().is_none_or(|p| self.player_ref_is(p, *moved_owner, referents))
                    && self.subject_matches(subject, Some(*object_id), referents, seq, SubjectRead::of_departure(arm, lki)),
            ),
            // The act a replacement kept in its zone (CR 614.6), read only by an
            // arm that names an act: a mill, not "leaves your library". The
            // object is asked as it was, which is how it still is.
            (
                TriggerEvent::ZoneChange { subject, change, by, owner, .. },
                GameEvent::SameZoneMove { object_id, owner: moved_owner, zone, act, lki, applied: _ },
            ) => triggering_event_if(
                change.act.is_some()
                    && change.matches(Some(*zone), *zone, Some(*act))
                    && by.as_ref().is_none_or(|by| by.matches(self, &record.stamp.cause, &mut { *referents }))
                    && owner.as_ref().is_none_or(|p| self.player_ref_is(p, *moved_owner, referents))
                    && self.subject_matches(subject, Some(*object_id), referents, seq, SubjectRead::LeftInThisEvent(lki)),
            ),
            (TriggerEvent::BecomesTapped { subject }, GameEvent::Tapped { object_id })
            | (TriggerEvent::BecomesUntapped { subject }, GameEvent::Untapped { object_id }) => {
                triggering_event_if(self.subject_matches(subject, Some(*object_id), referents, seq, SubjectRead::Now))
            }
            (TriggerEvent::DrawsCard { player: who, place, .. }, GameEvent::CardDrawn { player_id, .. }) => triggering_event_if(
                who.as_ref().is_none_or(|p| self.player_ref_is(p, *player_id, referents))
                    && place.is_none_or(|place| {
                        place.matches(DrawOrdinals { in_turn: record.place_in_turn(), in_draw_step: record.place_in_draw_step() })
                    }),
            ),
            (TriggerEvent::ShufflesLibrary { player: who }, GameEvent::LibraryShuffled { player_id })
            | (TriggerEvent::PlayerLoses { player: who }, GameEvent::PlayerLost { player_id }) => {
                triggering_event_if(who.as_ref().is_none_or(|p| self.player_ref_is(p, *player_id, referents)))
            }
            (
                TriggerEvent::ManaAdded { .. },
                GameEvent::ManaAdded { source_id, mana: added, tapped_for_mana: tapped, .. },
            ) => {
                let added = |m: ManaType| added.iter().any(|(t, n)| *t == m && *n > 0);
                let producer = ManaProducer { id: *source_id, tapped_for_mana: *tapped, lki: event.source_lki() };
                triggering_event_if(self.mana_added_arm_matches(arm, producer, added, referents, seq))
            }
            (
                TriggerEvent::DamageDealt { source, recipient, combat, .. },
                GameEvent::DamageDealt { source_id, target, is_combat, .. },
            ) => {
                // A source that left before dealing damage deals it as it last
                // existed (CR 608.2h); one still there is read now.
                let source_read = record.event.source_lki().map_or(SubjectRead::Now, SubjectRead::GoneBeforeThisEvent);
                triggering_event_if(
                    self.damage_recipient_matches(recipient, *target, referents, seq)
                        && combat.is_none_or(|c| c == *is_combat)
                        && self.subject_matches(source, Some(*source_id), referents, seq, source_read),
                )
            }
            // Announced as the batch decided, and read at its close, so the
            // recipient is asked as the damage left it (CR 603.10).
            (TriggerEvent::DamageIsPrevented { recipient, .. }, GameEvent::DamagePrevented { target, .. }) => {
                triggering_event_if(self.damage_recipient_matches(recipient, *target, referents, seq))
            }
            (TriggerEvent::PhaseBegins { phase, whose }, GameEvent::PhaseBegin { phase: rp, player }) => {
                triggering_event_if(phase == rp && whose.as_ref().is_none_or(|p| self.player_ref_is(p, *player, referents)))
            }
            (TriggerEvent::StepBegins { step, whose }, GameEvent::StepBegin { step: rs, player }) => {
                triggering_event_if(step == rs && whose.as_ref().is_none_or(|p| self.player_ref_is(p, *player, referents)))
            }
            (TriggerEvent::TurnBegins { whose }, GameEvent::TurnBegin { player, .. }) => {
                triggering_event_if(whose.as_ref().is_none_or(|p| self.player_ref_is(p, *player, referents)))
            }
            // The sign is the split. A 0 gain never reaches the log (CR
            // 119.10, `replacement::never_happens`).
            (TriggerEvent::GainsLife { player: who, .. }, GameEvent::LifeChanged { player_id, old, new, .. }) => {
                triggering_event_if(new > old && who.as_ref().is_none_or(|p| self.player_ref_is(p, *player_id, referents)))
            }
            (TriggerEvent::LosesLife { player: who, .. }, GameEvent::LifeChanged { player_id, old, new, .. }) => {
                triggering_event_if(new < old && who.as_ref().is_none_or(|p| self.player_ref_is(p, *player_id, referents)))
            }
            (TriggerEvent::CastsSpell { caster, spell }, GameEvent::SpellCast { spell_id, caster: who }) => {
                let spell_ok = match spell {
                    None => true,
                    Some(filter) => self.subject_matches(
                        &TriggerSubject::Filter(filter.clone()),
                        Some(*spell_id),
                        referents,
                        seq,
                        SubjectRead::Now,
                    ),
                };
                triggering_event_if(spell_ok && caster.as_ref().is_none_or(|p| self.player_ref_is(p, *who, referents)))
            }
            // CR 603.10e: the spell as it was on the stack, and whoever cast it.
            (TriggerEvent::IsCountered { spell, caster }, GameEvent::SpellCountered { spell_id, lki, .. }) => {
                let cast_by = lki.entry().and_then(StackEntry::caster);
                triggering_event_if(
                    caster.as_ref().is_none_or(|p| cast_by.is_some_and(|who| self.player_ref_is(p, who, referents)))
                        && spell.as_ref().is_none_or(|filter| {
                            let filter = TriggerSubject::Filter(filter.clone());
                            self.subject_matches(&filter, Some(*spell_id), referents, seq, SubjectRead::LeftInThisEvent(lki))
                        }),
                )
            }
            (
                TriggerEvent::EntersBattlefield { subject, controller, from, was_cast, .. },
                GameEvent::PermanentEnteredBattlefield { object_id, controller: rc },
            ) => {
                // The join (§4.4): `from` off the same object's zone change in
                // this window — a token has none — and "cast" off the facts the
                // permanent keeps (CR 400.7d).
                let from_ok = match from {
                    None => true,
                    Some(zone) => self.entry_origin(*object_id, seq) == Some(*zone),
                };
                let cast_ok = was_cast.is_none_or(|expected| {
                    self.battlefield.get(object_id).is_some_and(|e| e.cast.is_some()) == expected
                });
                triggering_event_if(
                    from_ok
                        && cast_ok
                        && controller.as_ref().is_none_or(|p| self.player_ref_is(p, *rc, referents))
                        && self.subject_matches(subject, Some(*object_id), referents, seq, SubjectRead::Now),
                )
            }
            // The declaration's attackers that match, each with what it
            // attacks, in the record's order: a triggering event each, or one
            // for each player, planeswalker or battle they attack.
            (
                TriggerEvent::Attacks { attacker, defender, occurrence, alone, .. },
                GameEvent::AttackersDeclared { attackers },
            ) => {
                // CR 506.5: the only creature declared as an attacker.
                if *alone && attackers.len() != 1 {
                    return Vec::new();
                }
                let attacks = attackers
                    .iter()
                    .filter(|&&(id, target)| {
                        self.subject_matches(attacker, Some(id), referents, seq, SubjectRead::Now)
                            && defender.as_ref().is_none_or(|asked| self.defender_matches(asked, target, referents, seq))
                    })
                    .filter_map(|&(id, target)| Some((self.subject_after(event, id)?, self.attacked(target)?)));
                match occurrence {
                    AttackOccurrence::Attacker => {
                        attacks.map(|(attacker, defender)| TriggeringEventObjects::Attack { attacker, defender }).collect()
                    }
                    AttackOccurrence::Defender => {
                        let mut defenses: Vec<(ObjectOrPlayer, Vec<ObjectRef>)> = Vec::new();
                        for (attacker, defender) in attacks {
                            match defenses.iter_mut().find(|(attacked, _)| *attacked == defender) {
                                Some((_, its_attackers)) => its_attackers.push(attacker),
                                None => defenses.push((defender, vec![attacker])),
                            }
                        }
                        defenses
                            .into_iter()
                            .map(|(defender, attackers)| TriggeringEventObjects::Defense { defender, attackers })
                            .collect()
                    }
                }
            }
            (TriggerEvent::Blocks { .. }, GameEvent::BlockersDeclared { .. }) => self.block_events(arm, record, referents),
            // The source as it was activated, on the battlefield: off its frame
            // when a cost moved it (Harsh Mentor's second ruling).
            (
                TriggerEvent::ActivatesAbility { player, source, mana_ability },
                GameEvent::AbilityActivated { identity, controller, mana_ability: was_mana, from, .. },
            ) => {
                let read = record.event.source_lki().map_or(SubjectRead::Now, SubjectRead::GoneBeforeThisEvent);
                let matched = *from == Zone::Battlefield
                    && mana_ability.is_none_or(|asked| asked == *was_mana)
                    && player.as_ref().is_none_or(|p| self.player_ref_is(p, *controller, referents))
                    && self.subject_matches(source, Some(identity.source.id), referents, seq, read);
                if matched { vec![TriggeringEventObjects::Object(identity.source)] } else { Vec::new() }
            }
            // Read after the event: the target as it is now, and the spell or
            // ability on the stack, a spell by its characteristics there.
            (
                TriggerEvent::BecomesTarget { target: watched, by: kind, controller: whose, .. },
                GameEvent::Targeted { target, by, ability_source, controller, .. },
            ) => {
                let target_ok = match (watched, target) {
                    (TargetSubject::Object(subject), TargetRef::Object(object)) => {
                        self.subject_matches(subject, Some(object.id), referents, seq, SubjectRead::Now)
                    }
                    (TargetSubject::Player(who), TargetRef::Player(player)) => {
                        who.as_ref().is_none_or(|p| self.player_ref_is(p, *player, referents))
                    }
                    (TargetSubject::Object(_), TargetRef::Player(_)) | (TargetSubject::Player(_), TargetRef::Object(_)) => false,
                };
                let kind_ok = kind.admits(ability_source.is_none())
                    && match kind {
                        TargetedBy::Spell(Some(filter)) => {
                            self.subject_matches(&TriggerSubject::Filter(filter.clone()), Some(*by), referents, seq, SubjectRead::Now)
                        }
                        TargetedBy::Spell(None) | TargetedBy::SpellOrAbility | TargetedBy::Ability => true,
                    };
                let whose_ok = whose.as_ref().is_none_or(|p| self.player_ref_is(p, *controller, referents));
                let objects = || {
                    let target = match target {
                        TargetRef::Object(object) => ObjectOrPlayer::Object(*object),
                        TargetRef::Player(player) => ObjectOrPlayer::Player(*player),
                    };
                    Some(TriggeringEventObjects::Target { target, by: self.object_ref(*by)? })
                };
                if target_ok && kind_ok && whose_ok { objects().into_iter().collect() } else { Vec::new() }
            }
            (TriggerEvent::AbilityTriggers { caused_by, source }, GameEvent::AbilityTriggered { origin, caused_by: cause, .. }) => {
                let source_ok = match source {
                    None => true,
                    Some(filter) => self.subject_matches(
                        &TriggerSubject::Filter(filter.clone()),
                        Some(origin.source()),
                        referents,
                        seq,
                        SubjectRead::Now,
                    ),
                };
                let cause_ok = match caused_by {
                    None => true,
                    Some(inner) => self.events.record(*cause).is_some_and(|r| inner.reads(&r.event)),
                };
                triggering_event_if(source_ok && cause_ok)
            }
            // The same ability of the same object, whichever instance (§6.5).
            (
                TriggerEvent::AbilityResolves { identity: IdentityRef::ThisAbility },
                GameEvent::AbilityResolved { identity: resolved, .. },
            ) => triggering_event_if(referents.this_ability.is_some_and(|this| {
                this.source == resolved.source && this.ability.definition() == resolved.ability.definition()
            })),
            // Read after the event: the attach is done when the dispatch asks
            // (Siona, Captain of the Pyleas' fourth ruling).
            (
                TriggerEvent::BecomesAttached { attachment: watched, host: on },
                GameEvent::Attached { attachment, host, .. },
            ) => triggering_event_if(
                self.subject_matches(watched, Some(*attachment), referents, seq, SubjectRead::Now)
                    && self.subject_matches(on, Some(*host), referents, seq, SubjectRead::Now),
            ),
            // CR 603.10c: each end as it was just before, which for one that
            // left in this event is its departure's frame.
            (
                TriggerEvent::BecomesUnattached { attachment: watched, former_host: from },
                GameEvent::Unattached { attachment, former_host, .. },
            ) => triggering_event_if(
                self.subject_matches(
                    watched, Some(attachment.id), referents, seq, self.subject_read_looking_back(*attachment),
                ) && self.subject_matches(
                    from, Some(former_host.id), referents, seq, self.subject_read_looking_back(*former_host),
                ),
            ),
            // CR 603.10d: the subject as it was before the change, "you" its
            // controller then (the candidate's, off its frame from before).
            (TriggerEvent::ControlChanges { subject, from, to }, GameEvent::ControlChanged { object_id, from: lost, to: gained, lki }) => {
                triggering_event_if(
                    from.as_ref().is_none_or(|p| self.player_ref_is(p, *lost, referents))
                        && to.as_ref().is_none_or(|p| self.player_ref_is(p, *gained, referents))
                        && self.subject_matches(subject, Some(*object_id), referents, seq, SubjectRead::LeftInThisEvent(lki)),
                )
            }
            // Leaving the battlefield is losing control of it (Khârn the
            // Betrayer's first ruling), to nobody: so only an arm that names no
            // one gaining it, and its controller as it left.
            (
                TriggerEvent::ControlChanges { subject, from, to: None },
                GameEvent::ZoneChange { object_id, from: Zone::Battlefield, lki: Some(lki), .. }
                | GameEvent::LeftTheGame { object_id, from: Zone::Battlefield, lki: Some(lki), .. },
            ) => triggering_event_if(
                from.as_ref().is_none_or(|p| self.player_ref_is(p, lki.chars().controller, referents))
                    && self.subject_matches(subject, Some(*object_id), referents, seq, SubjectRead::LeftInThisEvent(lki)),
            ),
            // A record matches when it passes the arm's three questions, each
            // of which the arm's `None` leaves unasked: the counter's kind
            // ("a +1/+1 counter"), the player who put or removed them ("you
            // put"), and what they are on. A removal by a rule names no player
            // (CR 704.5q, 120.3c), so no `by` matches it. A counter is an
            // occurrence of a per-counter arm, so a record of three holds three
            // events (Protean Hydra's fourth ruling).
            (
                TriggerEvent::CountersPutOn { subject: watched, counter_type, by, multiplicity },
                GameEvent::CountersPutOn { subject: on, counter_type: kind, n, by: putter, .. },
            ) => {
                let kind_matches = counter_type.is_none_or(|k| k == *kind);
                let putter_matches = by.as_ref().is_none_or(|p| self.player_ref_is(p, *putter, referents));
                if kind_matches && putter_matches && self.counters_on_matches(watched, *on, referents, seq) {
                    self.counter_events(arm, record, *multiplicity, *n)
                } else {
                    Vec::new()
                }
            }
            (
                TriggerEvent::CountersRemovedFrom { subject: watched, counter_type, by, multiplicity },
                GameEvent::CountersRemovedFrom { subject: on, counter_type: kind, n, by: remover, .. },
            ) => {
                let kind_matches = counter_type.is_none_or(|k| k == *kind);
                let remover_matches =
                    by.as_ref().is_none_or(|p| remover.is_some_and(|r| self.player_ref_is(p, r, referents)));
                if kind_matches && remover_matches && self.counters_on_matches(watched, *on, referents, seq) {
                    self.counter_events(arm, record, *multiplicity, *n)
                } else {
                    Vec::new()
                }
            }
            // Every arm, by name, so a new arm does not compile until it is matched
            // here; a record of another kind holds no event of it.
            (
                TriggerEvent::ZoneChange { .. }
                | TriggerEvent::BecomesTapped { .. }
                | TriggerEvent::BecomesUntapped { .. }
                | TriggerEvent::DrawsCard { .. }
                | TriggerEvent::ManaAdded { .. }
                | TriggerEvent::DamageDealt { .. }
                | TriggerEvent::DamageIsPrevented { .. }
                | TriggerEvent::PhaseBegins { .. }
                | TriggerEvent::StepBegins { .. }
                | TriggerEvent::TurnBegins { .. }
                | TriggerEvent::GainsLife { .. }
                | TriggerEvent::LosesLife { .. }
                | TriggerEvent::EntersBattlefield { .. }
                | TriggerEvent::Attacks { .. }
                | TriggerEvent::Blocks { .. }
                | TriggerEvent::BecomesTarget { .. }
                | TriggerEvent::ActivatesAbility { .. }
                | TriggerEvent::CastsSpell { .. }
                | TriggerEvent::IsCountered { .. }
                | TriggerEvent::PlayerLoses { .. }
                | TriggerEvent::ShufflesLibrary { .. }
                | TriggerEvent::CountersPutOn { .. }
                | TriggerEvent::CountersRemovedFrom { .. }
                | TriggerEvent::BecomesAttached { .. }
                | TriggerEvent::BecomesUnattached { .. }
                | TriggerEvent::ControlChanges { .. }
                | TriggerEvent::AbilityTriggers { .. }
                | TriggerEvent::AbilityResolves { .. },
                _,
            ) => Vec::new(),
        }
    }

    /// The declaration's occurrences of a block arm (CR 509.3a–g), in the
    /// record's order, each side asked as it is now (CR 509.3f). Read once
    /// for each blocking creature or each blocked one, the other side's
    /// creatures that match are the ones it counts, and the occurrence needs
    /// one of them, or `count`'s many (CR 509.3e).
    fn block_events(&self, arm: &TriggerEvent, record: &EventRecord, referents: &TriggerReferents<'_>) -> Vec<TriggeringEventObjects> {
        let (
            TriggerEvent::Blocks { blocker, attacker, occurrence, count, alone, .. },
            GameEvent::BlockersDeclared { blocks, unblocked },
        ) = (arm, &record.event)
        else {
            return Vec::new();
        };
        let (event, seq) = (&record.event, record.seq);
        // Each creature on one side of the declaration once, in its order.
        let each_on = |side: fn(&(ObjectId, ObjectId)) -> ObjectId| {
            let mut each: Vec<ObjectId> = Vec::new();
            for pair in blocks {
                if !each.contains(&side(pair)) {
                    each.push(side(pair));
                }
            }
            each
        };
        let blocker_of = |&(b, _): &(ObjectId, ObjectId)| b;
        let attacker_of = |&(_, a): &(ObjectId, ObjectId)| a;
        // CR 506.5: the only creature declared as a blocker.
        if *alone && each_on(blocker_of).len() != 1 {
            return Vec::new();
        }
        let is = |subject: &TriggerSubject, id: ObjectId| self.subject_matches(subject, Some(id), referents, seq, SubjectRead::Now);
        let now = |id: ObjectId| self.subject_after(event, id);
        let enough = |n: usize| count.unwrap_or(CountIs::AtLeast(1)).satisfied_by(n as u64);
        match *occurrence {
            BlockOccurrence::Block => blocks
                .iter()
                .filter(|&&(b, a)| is(blocker, b) && is(attacker, a))
                .filter_map(|&(b, a)| Some(TriggeringEventObjects::Block { blocker: now(b)?, attacker: now(a)? }))
                .collect(),
            BlockOccurrence::Blocker => each_on(blocker_of)
                .into_iter()
                    .filter(|&b| is(blocker, b))
                    .filter_map(|b| {
                        let attackers: Vec<ObjectRef> =
                            blocks.iter().filter(|&&(of, a)| of == b && is(attacker, a)).filter_map(|&(_, a)| now(a)).collect();
                        enough(attackers.len()).then_some(())?;
                        Some(TriggeringEventObjects::Blocking { blocker: now(b)?, attackers })
                    })
                    .collect(),
            BlockOccurrence::Blocked => each_on(attacker_of)
                .into_iter()
                    .filter(|&a| is(attacker, a))
                    .filter_map(|a| {
                        let blockers: Vec<ObjectRef> =
                            blocks.iter().filter(|&&(b, of)| of == a && is(blocker, b)).filter_map(|&(b, _)| now(b)).collect();
                        enough(blockers.len()).then_some(())?;
                        Some(TriggeringEventObjects::BlockedBy { attacker: now(a)?, blockers })
                    })
                    .collect(),
            BlockOccurrence::Unblocked => unblocked
                .iter()
                .filter(|&&a| is(attacker, a))
                .filter_map(|&a| Some(TriggeringEventObjects::Unblocked { attacker: now(a)? }))
                .collect(),
        }
    }

    /// A counter record's events for `arm`, or none when `matched` is false:
    /// one per counter for a per-counter arm, and one for the record
    /// otherwise, which the "one or more" readings fold.
    fn counter_events(
        &self,
        arm: &TriggerEvent,
        record: &EventRecord,
        multiplicity: Multiplicity,
        n: u32,
    ) -> Vec<TriggeringEventObjects> {
        let objects = self.triggering_event_objects(arm, record);
        match multiplicity {
            Multiplicity::PerOccurrence => vec![objects; n as usize],
            Multiplicity::OncePerEvent | Multiplicity::OncePerSubject => vec![objects],
        }
    }

    /// Whether the counters a record put or removed are where the arm watches.
    /// `watched` is the card's words, `on` the record's fact. "On this
    /// creature" or "on a creature you control" (`CountersOn::Object`) asks
    /// the object `on` names the question every object arm asks, of the object
    /// as it is after the event (CR 603.10). "On you" or "on a player"
    /// (`CountersOn::Player`) asks the player `on` names, any player for
    /// `None`. Counters on a player never match an object arm, nor the reverse.
    fn counters_on_matches(&self, watched: &CountersOn, on: CounterSubject, referents: &TriggerReferents<'_>, seq: EventId) -> bool {
        match (watched, on) {
            (CountersOn::Object(which), CounterSubject::Object(id)) => {
                self.subject_matches(which, Some(id), referents, seq, SubjectRead::Now)
            }
            (CountersOn::Player(who), CounterSubject::Player(player)) => {
                who.as_ref().is_none_or(|p| self.player_ref_is(p, player, referents))
            }
            (CountersOn::Object(_), CounterSubject::Player(_)) | (CountersOn::Player(_), CounterSubject::Object(_)) => false,
        }
    }

    /// Whether `arm`, one event of a trigger condition, is a `ManaAdded` arm —
    /// "whenever [something] is tapped for mana" or "adds mana" (CR 106.12a) —
    /// that the mana `producer` just made sets off: tapped for mana or not,
    /// with the types `added` admits, and made by the object the arm names
    /// for `referents`' ability (Wild Growth's enchanted land).
    fn mana_added_arm_matches(
        &self,
        arm: &TriggerEvent,
        producer: ManaProducer<'_>,
        added: impl Fn(ManaType) -> bool,
        referents: &TriggerReferents<'_>,
        seq: EventId,
    ) -> bool {
        let TriggerEvent::ManaAdded { source, tapped_for_mana, mana } = arm else { return false };
        // A producer its own cost moved is read as it last existed (CR 608.2h).
        let read = producer.lki.map_or(SubjectRead::Now, SubjectRead::GoneBeforeThisEvent);
        tapped_for_mana.is_none_or(|t| t == producer.tapped_for_mana)
            && mana.is_none_or(added)
            && self.subject_matches(source, Some(producer.id), referents, seq, read)
    }

    /// [`Self::mana_added_arm_matches`], asked before the mana is made.
    ///
    /// `oracle::mana_supply` (MA-1) decides whether a player could pay a cost
    /// before the engine offers the spell or ability at priority. For
    /// that it works out what tapping each source would add, Wild Growth's
    /// extra {G} included, so it asks this of a production that has not
    /// happened: would `permanent`'s own triggered mana ability trigger if
    /// `producer` made this mana? With no event yet, the record the production
    /// would be is the window's next. The trigger's limits, CR 113.6's zones
    /// and CR 603.2f's visibility stay the dispatcher's; the inventory does
    /// not ask them.
    pub(crate) fn mana_added_arm_would_match(
        &self,
        arm: &TriggerEvent,
        permanent: ObjectId,
        controller: PlayerId,
        producer: ObjectId,
        tapped: bool,
        added: impl Fn(ManaType) -> bool,
    ) -> bool {
        let Some(object) = self.objects.get(&permanent) else { return false };
        let referents = TriggerReferents {
            this: ThisObject::Candidate(permanent),
            controller,
            owner: object.owner,
            host: crate::oracle::board::host_of(self, permanent),
            this_ability: None,
            referred: &[],
        };
        let producer = ManaProducer { id: producer, tapped_for_mana: tapped, lki: None };
        self.mana_added_arm_matches(arm, producer, added, &referents, self.events.next_seq())
    }

    /// Whether damage dealt, or prevented, to `target` is to the player or
    /// permanent `recipient` names; a permanent is read now.
    fn damage_recipient_matches(
        &self,
        recipient: &DamageRecipient,
        target: DamageTarget,
        referents: &TriggerReferents<'_>,
        seq: EventId,
    ) -> bool {
        match (recipient, target) {
            (DamageRecipient::Any, _) => true,
            (DamageRecipient::Player(who), DamageTarget::Player(pid)) => {
                who.as_ref().is_none_or(|p| self.player_ref_is(p, pid, referents))
            }
            (DamageRecipient::Object(filter), DamageTarget::Object(id)) => match filter {
                None => true,
                Some(filter) => {
                    self.subject_matches(&TriggerSubject::Filter(filter.clone()), Some(id), referents, seq, SubjectRead::Now)
                }
            },
            (DamageRecipient::Player(_), DamageTarget::Object(_)) | (DamageRecipient::Object(_), DamageTarget::Player(_)) => false,
        }
    }

    /// "Which object" (§3.3): `This` is the source itself, `Host` what it is
    /// attached to, a filter is read against the subject — as `read` says.
    /// `seq` is the record's, which a remembered source's own departure is.
    fn subject_matches(
        &self,
        subject: &TriggerSubject,
        id: Option<ObjectId>,
        referents: &TriggerReferents<'_>,
        seq: EventId,
        read: SubjectRead<'_>,
    ) -> bool {
        match (subject, id) {
            (TriggerSubject::Any, _) => true,
            (TriggerSubject::ThisObject, Some(id)) => referents.this.is(self, id, seq),
            // Only a permanent has anything attached to it. One that left in
            // this event is asked as its departure framed it (CR 603.10a), since
            // the Aura or Equipment that stays has been unattached since.
            (TriggerSubject::Host, Some(id)) => match read {
                SubjectRead::LeftInThisEvent(lki) => lki.had_attached(referents.this.id()),
                SubjectRead::Now => referents.host == Some(id),
                SubjectRead::GoneBeforeThisEvent(_) => false,
            },
            (TriggerSubject::Referred, Some(id)) => referents.referred.iter().any(|r| r.is(self, id, seq)),
            (TriggerSubject::Filter(filter), Some(id)) => {
                let lki = match read {
                    SubjectRead::LeftInThisEvent(lki) | SubjectRead::GoneBeforeThisEvent(lki) => Some(lki),
                    SubjectRead::Now => None,
                };
                self.object_matches_filter(id, filter, &mut { *referents }, lki.map(HeldFrame::LastKnown))
            }
            (
                TriggerSubject::ThisObject | TriggerSubject::Host | TriggerSubject::Filter(_) | TriggerSubject::Referred,
                None,
            ) => false,
        }
    }

    /// "Whose" — a `PlayerRef` against a record's player, read for the
    /// trigger's referents (CR 109.5's "you" is its controller).
    fn player_ref_is(&self, who: &PlayerRef, player: PlayerId, referents: &TriggerReferents<'_>) -> bool {
        player_ref_names(self, who, player, &mut { *referents }) == Some(true)
    }

    /// Whether `target`, what a creature was declared attacking, is what
    /// `asked` names (CR 508.3a, 508.3b, 508.3e).
    fn defender_matches(&self, asked: &DefenderFilter, target: AttackTarget, referents: &TriggerReferents<'_>, seq: EventId) -> bool {
        asked.admits(target, &|who, player| self.player_ref_is(who, player, referents), &|filter, id| {
            self.subject_matches(&TriggerSubject::Filter(filter.clone()), Some(id), referents, seq, SubjectRead::Now)
        })
    }

    /// What `target` names, a permanent as the existence it is now (CR
    /// 400.7). `None` for a permanent the store no longer holds.
    fn attacked(&self, target: AttackTarget) -> Option<ObjectOrPlayer> {
        Some(match target {
            AttackTarget::Player(player) => ObjectOrPlayer::Player(player),
            AttackTarget::Planeswalker(id) | AttackTarget::Battle(id) => ObjectOrPlayer::Object(self.object_ref(id)?),
        })
    }

    /// `object` as it was just before the window's event (CR 603.10): where it
    /// is now while it is still that existence on the battlefield, and else
    /// as its departure in this window framed it — the join `entry_origin`
    /// makes for an entry's `from`.
    fn subject_read_looking_back(&self, object: ObjectRef) -> SubjectRead<'_> {
        if self.find(object).is_some() && self.battlefield.contains_key(&object.id) {
            return SubjectRead::Now;
        }
        self.events
            .held()
            .iter()
            .find_map(|record| record.event.lki().map(|(lki, ..)| lki).filter(|lki| lki.object() == object))
            .map_or(SubjectRead::Now, SubjectRead::LeftInThisEvent)
    }

    /// The zone the permanent at `id` entered from, off its own zone change
    /// earlier in the same window — `None` for a token, which came from
    /// nowhere (CR 111.2). The entry's performer announces both, so the
    /// window holds the move whenever it holds the entry.
    fn entry_origin(&self, id: ObjectId, entered_at: EventId) -> Option<Zone> {
        let held = self.events.held();
        let before = held.partition_point(|r| r.seq < entered_at);
        for record in held[..before].iter().rev() {
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

    /// The objects `record`'s one triggering event is about (§3.4), each as
    /// the existence the trigger remembers (`subject_after`). The arm says
    /// which event roles the event has, grouped as `event_roles()` groups
    /// them; its record is one `triggering_events_in` matched to it. An
    /// object the store no longer holds leaves the event about none. An
    /// attack record holds one event per attacker, which
    /// `triggering_events_in` makes, so it answers none here.
    fn triggering_event_objects(&self, arm: &TriggerEvent, record: &EventRecord) -> TriggeringEventObjects {
        match arm {
            TriggerEvent::ZoneChange { .. }
            | TriggerEvent::BecomesTapped { .. }
            | TriggerEvent::BecomesUntapped { .. }
            | TriggerEvent::DrawsCard { .. }
            | TriggerEvent::ManaAdded { .. }
            | TriggerEvent::EntersBattlefield { .. }
            | TriggerEvent::CastsSpell { .. }
            | TriggerEvent::IsCountered { .. }
            | TriggerEvent::ControlChanges { .. } => self.single_object(&record.event),
            TriggerEvent::DamageDealt { .. } => self.damage_objects(&record.event),
            TriggerEvent::DamageIsPrevented { .. } => self.prevention_objects(&record.event),
            TriggerEvent::CountersPutOn { .. } | TriggerEvent::CountersRemovedFrom { .. } => {
                match &record.event {
                    GameEvent::CountersPutOn { subject: CounterSubject::Object(id), .. }
                    | GameEvent::CountersRemovedFrom { subject: CounterSubject::Object(id), .. } => self
                        .subject_after(&record.event, *id)
                        .map_or(TriggeringEventObjects::NoObject, TriggeringEventObjects::Object),
                    _ => TriggeringEventObjects::NoObject,
                }
            }
            TriggerEvent::BecomesAttached { .. } | TriggerEvent::BecomesUnattached { .. } => {
                self.attachment_objects(&record.event)
            }
            TriggerEvent::Attacks { .. }
            | TriggerEvent::Blocks { .. }
            | TriggerEvent::BecomesTarget { .. }
            | TriggerEvent::ActivatesAbility { .. }
            | TriggerEvent::PhaseBegins { .. }
            | TriggerEvent::StepBegins { .. }
            | TriggerEvent::TurnBegins { .. }
            | TriggerEvent::GainsLife { .. }
            | TriggerEvent::LosesLife { .. }
            | TriggerEvent::PlayerLoses { .. }
            | TriggerEvent::ShufflesLibrary { .. }
            | TriggerEvent::AbilityTriggers { .. }
            | TriggerEvent::AbilityResolves { .. } => TriggeringEventObjects::NoObject,
        }
    }

    /// The one object a one-object event is about: the card or permanent
    /// that moved, was tapped, untapped, entered or changed control, the
    /// mana's source, the card drawn, or the spell.
    fn single_object(&self, event: &GameEvent) -> TriggeringEventObjects {
        let id = match event {
            GameEvent::ZoneChange { object_id, .. }
            | GameEvent::LeftTheGame { object_id, .. }
            | GameEvent::SameZoneMove { object_id, .. }
            | GameEvent::Tapped { object_id }
            | GameEvent::Untapped { object_id }
            | GameEvent::PermanentEnteredBattlefield { object_id, .. }
            | GameEvent::ControlChanged { object_id, .. } => *object_id,
            // The existence that made the mana, where its own cost moved it.
            GameEvent::ManaAdded { source_lki: Some(lki), .. } => return TriggeringEventObjects::Object(lki.object()),
            GameEvent::ManaAdded { source_id, .. } => *source_id,
            GameEvent::CardDrawn { card_id, .. } => *card_id,
            GameEvent::SpellCast { spell_id, .. } | GameEvent::SpellCountered { spell_id, .. } => *spell_id,
            _ => return TriggeringEventObjects::NoObject,
        };
        self.subject_after(event, id).map_or(TriggeringEventObjects::NoObject, TriggeringEventObjects::Object)
    }

    /// A damage event's source and recipient. A source that left before
    /// dealing the damage is the existence that dealt it (CR 608.2h).
    fn damage_objects(&self, event: &GameEvent) -> TriggeringEventObjects {
        let GameEvent::DamageDealt { source_id, target, .. } = event else {
            return TriggeringEventObjects::NoObject;
        };
        let source = event.source_lki().map(Lki::object).or_else(|| self.subject_after(event, *source_id));
        let recipient = match target {
            DamageTarget::Object(id) => self.subject_after(event, *id).map(ObjectOrPlayer::Object),
            DamageTarget::Player(player) => Some(ObjectOrPlayer::Player(*player)),
        };
        match (source, recipient) {
            (Some(source), Some(recipient)) => TriggeringEventObjects::Damage { source, recipient },
            _ => TriggeringEventObjects::NoObject,
        }
    }

    /// The object a prevention effect prevented damage to; a player is
    /// none.
    fn prevention_objects(&self, event: &GameEvent) -> TriggeringEventObjects {
        match event {
            GameEvent::DamagePrevented { target: DamageTarget::Object(id), .. } => self
                .subject_after(event, *id)
                .map_or(TriggeringEventObjects::NoObject, |recipient| TriggeringEventObjects::Prevention { recipient }),
            _ => TriggeringEventObjects::NoObject,
        }
    }

    /// An attachment and its host, or its former host. An unattach names
    /// each end as the existence the record names (Grafted Wargear's first
    /// ruling).
    fn attachment_objects(&self, event: &GameEvent) -> TriggeringEventObjects {
        match event {
            GameEvent::Attached { attachment, host, .. } => {
                match (self.subject_after(event, *attachment), self.subject_after(event, *host)) {
                    (Some(attachment), Some(host)) => TriggeringEventObjects::Attach { attachment, host },
                    _ => TriggeringEventObjects::NoObject,
                }
            }
            GameEvent::Unattached { attachment, former_host, .. } => {
                TriggeringEventObjects::Attach { attachment: *attachment, host: *former_host }
            }
            _ => TriggeringEventObjects::NoObject,
        }
    }

    /// "That object" as a trigger remembers it: `id` together with the
    /// zone-change epoch that tells this object from what it becomes if it
    /// moves again (CR 400.7). For a zone change of `id`, the epoch is the one
    /// the move gave it, which the record carries, so "return it" after a dies
    /// trigger looks for the card only in the graveyard it went to (CR
    /// 603.6c), even if something moved it before the dispatch. Any other
    /// record names `id` alone, so the object is taken as the dispatch finds
    /// it, which is right after the event.
    pub(super) fn subject_after(&self, event: &GameEvent, id: ObjectId) -> Option<ObjectRef> {
        // An unattach names each end as the existence it was, so a host that
        // has left since is not "that permanent" (Grafted Wargear's first
        // ruling).
        if let GameEvent::Unattached { attachment, former_host, .. } = event {
            return [*attachment, *former_host].into_iter().find(|end| end.id == id);
        }
        match event.new_object() {
            Some(made) if made.id == id => Some(made),
            _ => self.object_ref(id),
        }
    }

    /// The newcomer `event` brought, if any: the object it put where it is
    /// now, which was somewhere else just before the event. That is a zone
    /// change's moved card, as the new object the move made it (CR 400.7), or
    /// a token the event created. `match_candidates` asks a newcomer none of
    /// the event's look-back arms (CR 603.10a), so a Bridge from Below put
    /// into the graveyard by a wipe does not trigger on the creatures dying
    /// beside it.
    fn newcomer_from(&self, event: &GameEvent) -> Option<ObjectRef> {
        match event {
            GameEvent::TokenCreated { object_id, .. } => self.object_ref(*object_id),
            event => event.new_object(),
        }
    }

    /// `id` as the object it is now: its id and its current existence (CR
    /// 400.7). `None` for an id no longer in the store.
    pub fn object_ref(&self, id: ObjectId) -> Option<ObjectRef> {
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
        let effect = pending.binding.def.effect.clone();
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
                let player = triggered_mana_recipient(self, recipient, source, pending.controller);
                let mut resolution = ResolutionContext::for_mana_ability(self, source, player, GeneratedBy::TriggeredAbility);
                resolution.trigger = Some(pending.binding.clone());
                let producer = ManaProducer { id: source, tapped_for_mana: false, lki: None };
                self.produce_mana(output, player, producer, &resolution, &ActionContext::resolving(ctx.dp, &resolution))
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
    matches!(def.effect, Effect::Triggered(_))
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

