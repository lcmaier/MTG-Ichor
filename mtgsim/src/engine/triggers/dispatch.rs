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
use crate::oracle::characteristics::{FrameContext, ObjectInformation};
use crate::types::effects::Antecedent;
use crate::engine::actions::{ActionContext, GameAction, ZoneChangeAct};
use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::condition::{settled_holds, ConditionReferents};
use crate::engine::layers::types::{EffectiveCharacteristics, Timestamp};
use crate::engine::mana::triggered_mana_recipient;
use crate::engine::object_filter::{player_ref_names, EarlierTargets, FilterReferents};
use crate::engine::resolve::ResolutionContext;
use crate::engine::trace_records;
use crate::engine::zone_function::{condition_functions_in, functions_as_static, functions_in};
use crate::events::cause::{EventCause, GameRule, GeneratedBy};
use crate::ui::decision::AsksNobody;
use crate::events::event::{BatchId, DepartedObject, EventRecord, EventId, GameEvent, LifeChange};
use crate::objects::card_data::CardData;
use crate::objects::object::GameObject;
use crate::oracle::characteristics::controller_or_owner;
use crate::state::battlefield::{AttackTarget, CostChoices};
use crate::state::continuous_effects::triggered_ability_kinds;
use crate::state::game_state::{AbilityIdentity, GameState, StackEntry};
use crate::state::history_log::RowFact;
use crate::types::effects::{ObjectFilter, Condition, Effect, EffectRecipient, PlayerRef, Primitive};
use crate::types::history::CountIs;
use crate::types::ids::{DelayedTriggerId, IdMap, IdSet, ObjectId, ObjectOrPlayer, ObjectRef, PlayerId};
use crate::types::mana::ManaType;
use crate::types::triggers::{zone_change_looks_back, AttackOccurrence, BlockOccurrence, ObjectOrPlayerSubject, TargetedBy, DamageRecipient, DefenderFilter, EventIndex, EventKind, EventKindMask, IdentityRef, Lki, Multiplicity, PendingTrigger, Referred, RememberedObject, TriggerBinding, TriggerCondition, TriggerDef, TriggerEvent, TriggerLimit, TriggerOrigin, TriggerId, TriggerSubject, TriggeringEvent, TriggeringEventObjects};
use super::binding::TriggeringEventView;

use crate::types::zones::{DrawOrdinals, Zone, ZoneSet};

/// Dispatches nested inside dispatches — a tier-2 trigger's `AbilityTriggered`
/// dispatched from the dispatch that queued it, and so on. No printed ability
/// watches its own kind, so a chain this deep is taken for one with no end,
/// each ability triggering on the last one's triggering, and the game is a
/// draw (CR 104.4b): a count standing in for a proof that the chain cannot end
/// (`triggers-architecture.md` §4.8, §4.9).
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

/// A keeper (CR 113.7a, 608.2h): a triggered ability waiting to be put on the
/// stack, a stack entry, the resolving object, or a delayed trigger waiting for
/// its event. Each reads its source later, and a trigger the objects of its one
/// triggering event too, so a batch frames one of them that leaves, for the
/// LKI store.
#[derive(Clone, Copy)]
pub(super) struct LkiKeeper<'a> {
    /// The ability it is; `None` for a spell.
    identity: Option<AbilityIdentity>,
    /// Its trigger, for a triggered ability that has triggered and not yet
    /// left the stack (CR 603.8, 608.2n).
    trigger: Option<&'a TriggerBinding>,
    place: KeeperPlace,
}

/// Where a keeper is, so a write can reach it. A trigger's binding is fixed
/// once it triggers but for one field, `source_found_in_graveyard` (CR
/// 400.7f), which a state-based action can fill while the trigger waits or
/// sits on the stack (`aura_put_into_graveyard_by_sba`).
#[derive(Clone, Copy)]
enum KeeperPlace {
    Waiting(usize),
    Stacked(ObjectId),
    Resolving,
    Delayed,
}

impl LkiKeeper<'_> {
    /// The objects it reads: its source, then its triggering event's objects.
    fn objects(self) -> impl Iterator<Item = ObjectRef> {
        let event_objects = self.trigger.and_then(TriggerBinding::triggering_event).into_iter().flat_map(|event| event.objects.objects());
        self.identity.map(|identity| identity.source).into_iter().chain(event_objects.map(|(_, object)| object))
    }

    /// Whether it is ability `identity` triggered and not yet left the stack.
    pub(super) fn is_triggered(self, identity: AbilityIdentity) -> bool {
        self.trigger.is_some() && self.identity == Some(identity)
    }
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
        | Effect::FunctionsIn(_, inner)
        | Effect::Optional { effect: inner, .. }
        | Effect::Remember(inner)
        | Effect::ForEach(_, inner)
        | Effect::Repeat(_, inner) => could_add_mana(inner),
        Effect::Replacement(_)
        | Effect::Restriction(_)
        | Effect::CostModification(_)
        | Effect::TriggerMultiplier(_)
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

/// The objects that may carry a triggered ability reading some kinds of
/// record, as the dispatch gate's legs find them: the permanents whose
/// printed triggers read one, the objects a row naming them puts one on, the
/// window's newcomers, the zones a `Filter` row reaches, and the cards off
/// the battlefield on the zone map. Built by one constructor
/// (`GameState::trigger_gate`) for the dispatch, the look-back snapshot and
/// the state check.
pub(super) struct TriggerGate<'a> {
    game: &'a GameState,
    kinds: EventKindMask,
    off_battlefield_legs: OffBattlefieldLegs,
    readers: Vec<ObjectId>,
    named: Vec<ObjectId>,
    newcomers: Vec<ObjectId>,
    unattributed: ZoneSet,
}

/// Which of the cards off the battlefield the gate hands the matcher. Two
/// legs find them: the zone map, every card whose triggered ability works
/// where it is (a graveyard's, a hand's), filed with the kinds of record its
/// abilities read; and the zones a row granting or copying a triggered
/// ability reaches. `Unfiltered` takes every card on both, whatever the
/// window holds: the dispatch and the look-back snapshot ask that today.
/// `FilteredByKind` takes only those whose abilities read a kind the window
/// carries: the state check asks that, for its one kind. Whether the dispatch
/// can afford to filter too is the readiness pass's question.
#[derive(Clone, Copy)]
pub(super) enum OffBattlefieldLegs {
    Unfiltered,
    FilteredByKind,
}

impl<'a> TriggerGate<'a> {
    /// Whether no leg holds anything, so the candidates are none.
    pub(super) fn is_empty(&self) -> bool {
        let no_zone_sources = match self.off_battlefield_legs {
            OffBattlefieldLegs::Unfiltered => self.game.zone_trigger_sources.is_empty(),
            OffBattlefieldLegs::FilteredByKind => zone_sources(self.game, self.kinds, self.off_battlefield_legs).next().is_none(),
        };
        self.readers.is_empty() && self.named.is_empty() && self.newcomers.is_empty() && self.unattributed.is_empty() && no_zone_sources
    }

    /// The candidates its legs reach, in timestamp order.
    pub(super) fn candidates(self) -> Vec<ObjectId> {
        let TriggerGate { game, kinds, off_battlefield_legs, readers, named, newcomers, unattributed } = self;
        game.candidates_now(readers, named, newcomers, unattributed, zone_sources(game, kinds, off_battlefield_legs))
    }
}

/// The zone map's objects, only those filed under `kinds` where the legs off
/// the battlefield ask them.
fn zone_sources(game: &GameState, kinds: EventKindMask, off_battlefield_legs: OffBattlefieldLegs) -> impl Iterator<Item = ObjectId> + '_ {
    // ORDER-FREE: `candidates_now` sorts the zone map's objects by timestamp.
    game.zone_trigger_sources
        .iter()
        .filter(move |(_, filed)| matches!(off_battlefield_legs, OffBattlefieldLegs::Unfiltered) || filed.intersects(kinds))
        .map(|(&id, _)| id)
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
    /// The source's host (CR 303.4m), for `TriggerSubject::Object(Antecedent::ThisObjectsHost)`.
    pub(super) host: Option<ObjectId>,
    pub(super) frame: TriggerCandidateFrame<'a>,
    /// What a stack object is built from (CR 603.3's "text of the ability").
    pub(super) card: Arc<CardData>,
}

impl<'a> TriggerCandidate<'a> {
    /// A live object read through `frame`, under its controller `now` (CR
    /// 603.3a). A list from before an event is its controller's then (CR
    /// 603.10, 603.3a), which a control change made someone else.
    pub(super) fn live(
        object: &'a GameObject,
        existence: ObjectRef,
        now: PlayerId,
        host: Option<ObjectId>,
        frame: TriggerCandidateFrame<'a>,
    ) -> Self {
        let controller = match &frame {
            TriggerCandidateFrame::Before { lki, .. } => lki.characteristics().controller,
            TriggerCandidateFrame::Live { .. } | TriggerCandidateFrame::LastKnown { .. } => now,
        };
        TriggerCandidate { object: existence, controller, owner: object.owner, zone: object.zone, host, frame, card: Arc::clone(&object.card_data) }
    }

    /// An existence that has left `zone`, read through its last known
    /// information, under its controller then (CR 603.3a, 603.10a):
    /// `snapshot: None` is the frame its record carries, `Some(k)` snapshot
    /// `k`'s. The card is the store's after a zone change; for an object gone
    /// from the store with its owner (CR 800.4a) the frame's own list is all
    /// there is, and the stack object is built from a card that names what
    /// the frame does.
    pub(super) fn from_lki(game: &GameState, lki: &'a Lki, zone: Zone, snapshot: Option<usize>) -> Self {
        let card = match game.objects.get(&lki.object().id) {
            Some(object) => Arc::clone(&object.card_data),
            None => Arc::new(frame_card(lki.characteristics())),
        };
        TriggerCandidate {
            object: lki.object(),
            controller: lki.characteristics().controller,
            owner: lki.owner(),
            zone,
            host: lki.host(),
            frame: TriggerCandidateFrame::LastKnown { lki, snapshot },
            card,
        }
    }

    /// What the candidate's ability means by "this object", "you", "its
    /// owner" and "enchanted creature", "this ability" being `this_ability`
    /// (CR 603.7h).
    fn referents(&self, this_ability: Option<AbilityIdentity>) -> TriggerReferents<'static> {
        TriggerReferents::of_object(self.object.id, self.controller, self.owner, self.host, this_ability)
    }
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
    /// The source's host (CR 303.4m), for `TriggerSubject::Object(Antecedent::ThisObjectsHost)`.
    pub(super) host: Option<ObjectId>,
    /// CR 603.7h's "this ability": the ability whose arm this is, or the one
    /// that created a delayed trigger; `None` for a spell's delayed trigger.
    pub(super) this_ability: Option<AbilityIdentity>,
    /// CR 603.7c's "that token": the objects a delayed trigger refers to;
    /// none for an object's own ability.
    pub(super) referred: &'a [RememberedObject],
}

impl TriggerReferents<'static> {
    /// An object's own ability's: "this object" is the object, and it refers
    /// to nothing, since what is referred to is a delayed trigger's (CR
    /// 603.7c).
    pub(super) fn of_object(
        this: ObjectId,
        controller: PlayerId,
        owner: PlayerId,
        host: Option<ObjectId>,
        this_ability: Option<AbilityIdentity>,
    ) -> Self {
        TriggerReferents { this: ThisObject::Candidate(this), controller, owner, host, this_ability, referred: &[] }
    }
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

    /// The candidate, or its host, now: an arm's own words have no
    /// triggering event yet, and name no one remembered object.
    fn defending_player(&self, game: &GameState, creature: Antecedent) -> Option<PlayerId> {
        match creature {
            Antecedent::ThisObject => ObjectInformation::Current(self.this.id()).defending_player(game, &|_| {}),
            Antecedent::ThisObjectsHost => ObjectInformation::Current(self.host?).defending_player(game, &|_| {}),
            Antecedent::Triggering(_) | Antecedent::Referred => None,
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
    /// 400.7).
    Remembered(RememberedObject),
}

impl ThisObject {
    fn id(self) -> ObjectId {
        match self {
            ThisObject::Candidate(id) => id,
            ThisObject::Remembered(remembered) => remembered.object.id,
        }
    }

    /// Whether a record's object `id`, the existence `named`, is this
    /// object: the candidate, or the remembered existence (CR 400.7).
    fn is(self, id: ObjectId, named: Option<ObjectRef>) -> bool {
        match self {
            ThisObject::Candidate(this) => id == this,
            ThisObject::Remembered(remembered) => named == Some(remembered.object),
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
    /// A surviving object's list from before a batch: `LookBackSnapshot`
    /// number `snapshot`'s frame of it, which keeps its counters, attachments
    /// and combat as they were then too.
    Before { lki: &'a Lki, snapshot: usize },
}

impl TriggerCandidateFrame<'_> {
    fn characteristics(&self) -> &EffectiveCharacteristics {
        match self {
            TriggerCandidateFrame::Live { chars, .. } => chars,
            TriggerCandidateFrame::LastKnown { lki, .. } => lki.characteristics(),
            TriggerCandidateFrame::Before { lki, .. } => lki.characteristics(),
        }
    }

    /// Object `id` as a filter reads it through this frame: by its frame from
    /// before the event for an object that left or a survivor's list from
    /// before a batch, which say who controlled it then and what was on it,
    /// and by its list now otherwise.
    fn information(&self, id: ObjectId) -> ObjectInformation<'_> {
        match self {
            TriggerCandidateFrame::LastKnown { lki, .. } | TriggerCandidateFrame::Before { lki, .. } => {
                ObjectInformation::LastKnown(lki)
            }
            TriggerCandidateFrame::Live { chars, .. } => ObjectInformation::Characteristics(id, chars, FrameContext::Now),
        }
    }

    /// Whether this list is on the board `arm` reads for a record that looks
    /// back through snapshot `looks_back_through` (CR 603.10): after the
    /// event, or before it for an arm that looks back.
    fn on_the_board_read_by(&self, arm: &TriggerEvent, looks_back_through: Option<usize>, newcomer: bool) -> bool {
        self.answered_arms(looks_back_through, newcomer).is_some_and(|answered| answered.admits(arm))
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
    mana: bool,
}

/// One triggered ability of one candidate, with the facts that do not depend
/// on the record answered once: CR 113.6 and the identity.
struct TriggerCandidateDef<'a> {
    /// Index into the candidate list.
    candidate: usize,
    identity: AbilityIdentity,
    def: &'a Arc<TriggerDef>,
    /// The zones its text states it functions in (CR 113.6b).
    stated_zones: Option<ZoneSet>,
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

impl AnsweredArms {
    /// Whether a list of this kind answers `arm` (CR 603.10): an arm that
    /// looks back reads the list from before the event, and every other arm
    /// the list after it.
    fn admits(self, arm: &TriggerEvent) -> bool {
        match self {
            AnsweredArms::All => true,
            AnsweredArms::LookBackOnly => arm.looks_back(),
            AnsweredArms::AllButLookBack => !arm.looks_back(),
        }
    }
}

/// A departure's subject `id` as an arm reads it: as its record framed it for
/// an arm that looks back, and as it is now for any other, "from anywhere"
/// among them (CR 603.6c, 603.10).
fn departure_read<'a>(arm: &TriggerEvent, departed: &'a DepartedObject) -> ObjectInformation<'a> {
    match departed.lki() {
        Some(lki) if arm.looks_back() => ObjectInformation::LastKnown(lki),
        _ => ObjectInformation::Current(departed.object().id),
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
    /// The zones its text states it functions in, which its binding takes
    /// (CR 113.6b).
    pub(super) stated_zones: Option<ZoneSet>,
    pub(super) source_card: Arc<CardData>,
    pub(super) instances: Vec<EffectRecipient>,
    pub(super) event: EventIndex,
    pub(super) triggering_events: Vec<TriggeringEvent>,
    pub(super) mana: bool,
    /// How many more times it triggers (CR 603.2d): the multipliers' count,
    /// which a delayed or reflexive trigger never has.
    pub(super) additional: u32,
}

impl MatchedTrigger {
    /// The arm of its condition that matched.
    pub(super) fn arm(&self) -> &TriggerEvent {
        &self.def.condition.events()[self.event.0]
    }
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
    /// The condition is a state trigger, which the state check asks of the
    /// board instead (`triggers::state`, CR 603.8).
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
            // The draw asks nothing, and the dispatch may have no provider.
            let rule = ActionContext::game_rule(&AsksNobody, GameRule::MandatoryLoop);
            return self.execute_actions(vec![GameAction::GameEndsInADraw], &rule).map(|_| ());
        }
        // Every record, before the gate: what happened this turn is read by
        // cards that are not on the battlefield yet (§3.10).
        self.advance_history(window);
        self.nesting.dispatch_depth += 1;
        let result = self.dispatch_inner(window, ctx, snapshots, audit);
        self.nesting.dispatch_depth -= 1;
        // CR 603.8 — after every dispatch, so a state true only for a moment
        // of a resolution, between two of its instructions, triggers (Emperor
        // Crocodile's first ruling).
        if result.is_ok() {
            self.check_state_triggers()?;
        }
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
    /// 2. **The candidates** — `find_trigger_matches`' four legs: every object that
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
        let gate = self.trigger_gate(window_kinds, self.newcomer_trigger_carriers(window), OffBattlefieldLegs::Unfiltered);
        // The departed leg reads lists no registration saw, so it keeps its
        // whole walk.
        let any_frame_source = window.iter().any(|seq| {
            self.events.record(*seq).is_some_and(|r| {
                r.event.lki().is_some_and(|(lki, ..)| lki.characteristics().abilities.iter().any(|ability| ability.effect.as_trigger().is_some()))
            })
        });
        if gate.is_empty() && !any_frame_source && snapshots.is_empty() {
            return Vec::new();
        }
        self.find_trigger_matches(window, gate, snapshots)
    }

    /// The dispatch gate's legs for a window carrying `kinds`, with the
    /// window's `newcomers`, and the cards off the battlefield filtered or
    /// not (`OffBattlefieldLegs`).
    pub(super) fn trigger_gate(&self, kinds: EventKindMask, newcomers: Vec<ObjectId>, off_battlefield_legs: OffBattlefieldLegs) -> TriggerGate<'_> {
        let summary = self.continuous_effects.summary();
        let unattributed = match off_battlefield_legs {
            OffBattlefieldLegs::Unfiltered => summary.unattributed_trigger_zones,
            OffBattlefieldLegs::FilteredByKind if summary.unattributed_trigger_kinds.intersects(kinds) => summary.unattributed_trigger_zones,
            OffBattlefieldLegs::FilteredByKind => ZoneSet::EMPTY,
        };
        TriggerGate {
            game: self,
            kinds,
            off_battlefield_legs,
            readers: self.battlefield_readers(kinds),
            named: self.named_trigger_carriers(kinds),
            newcomers,
            unattributed,
        }
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
        let mut announce: Vec<GameEvent> = Vec::new();
        for m in matches {
            let caused_by = m.triggering_events[0].record.seq;
            // Read before CR 603.2d's count: once, whatever the count.
            if !self.may_trigger_this_turn(&m.def, m.identity) {
                continue;
            }
            let times = match m.def.limit {
                Some(TriggerLimit::TriggersOnlyOnceEachTurn) => 1,
                _ => 1 + m.additional,
            };
            let source_found_in_graveyard = self.aura_put_into_graveyard_beside_its_host(&m);
            let binding = TriggerBinding {
                def: Arc::clone(&m.def),
                triggering_events: m.triggering_events,
                event: m.event,
                triggered_by: None,
                referred: Referred::default(),
                source_found_in_graveyard,
                stated_zones: m.stated_zones,
            };
            let origin = TriggerOrigin::Object(m.identity);
            // Each time it triggers is its own trigger, with its own choices
            // and targets (Panharmonicon's fifth ruling). The last time takes
            // the binding, so a trigger counted once copies nothing.
            for binding in std::iter::repeat_n(binding, times as usize) {
                let pending = PendingTrigger {
                    id: self.take_next_trigger_id(),
                    origin,
                    controller: m.controller,
                    source_card: Arc::clone(&m.source_card),
                    instances: m.instances.clone(),
                    binding,
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
                self.queue_trigger(pending, Some(caused_by), &mut announce);
            }
        }
        self.queue_delayed(delayed, ctx, &mut announce);
        self.announce_triggered(announce)
    }

    /// "This ability triggers only once each turn", taken as the ability
    /// queues, so a second match in the same window finds it taken.
    /// False once this turn's has been taken.
    pub(super) fn may_trigger_this_turn(&mut self, def: &TriggerDef, identity: AbilityIdentity) -> bool {
        def.limit != Some(TriggerLimit::TriggersOnlyOnceEachTurn) || self.history.triggered_this_turn.insert(identity)
    }

    /// The next trigger's number, in the order triggers are queued.
    pub(super) fn take_next_trigger_id(&mut self) -> TriggerId {
        let id = TriggerId(self.next_trigger_id);
        self.next_trigger_id += 1;
        id
    }

    /// `pending` onto the queue, and its `AbilityTriggered` onto `announce`:
    /// caused by the record that matched (the first, for a "one or more"
    /// trigger), or by none for a state trigger (CR 603.8) and a "when you
    /// don't" (CR 603.12).
    pub(super) fn queue_trigger(&mut self, pending: PendingTrigger, caused_by: Option<EventId>, announce: &mut Vec<GameEvent>) {
        announce.push(GameEvent::AbilityTriggered { trigger: pending.id, origin: pending.origin, controller: pending.controller, caused_by });
        self.pending_triggers.push(pending);
    }

    /// The records CR 603.3b's second tier ("whenever an ability triggers")
    /// watches: emitted after the window has closed, unstamped, each
    /// dispatched as it is emitted like any unbatched record.
    pub(super) fn announce_triggered(&mut self, announce: Vec<GameEvent>) -> Result<(), String> {
        for triggered in announce {
            self.emit_event_unbatched(triggered, EventCause::GameRule(GameRule::AbilityTriggers))?;
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
        if !matched.arm().watches_its_host_leave() || self.find(aura).is_some() {
            return None;
        }
        // `aura-found-in-graveyard`'s switch
        // (`engineering-practices.md#when-a-ruling-and-the-cr-disagree`), with
        // the CR 704.5m hook's: the graveyard only.
        self.events.held().iter().find_map(|record| match &record.event {
            GameEvent::ZoneChange { to: Zone::Graveyard, departed: DepartedObject::WithLki(lki), .. } if lki.object() == aura => record.event.new_object(),
            _ => None,
        })
    }

    /// CR 400.7f's second sentence: CR 704.5m has put `aura`, the existence
    /// that was on the battlefield, into its owner's graveyard (and not, by a
    /// replacement, anywhere else), and each of its
    /// waiting triggers on its enchanted permanent leaving finds it there. They
    /// are pending, or on the stack at the latest: the check follows the
    /// leaving before anyone receives priority.
    ///
    /// The triggers that find it are read first and only those are written:
    /// a write copies the trigger if a fork shares it (`state::shared_stores`).
    pub(crate) fn aura_put_into_graveyard_by_sba(&mut self, aura: ObjectRef) {
        let Some(found) = self.object_ref(aura.id) else { return };
        let triggers_finding_the_aura: Vec<KeeperPlace> = self
            .lki_keepers()
            .filter(|keeper| {
                keeper.identity.is_some_and(|identity| identity.source == aura)
                    && keeper.trigger.is_some_and(|binding| {
                        binding.event().is_some_and(TriggerEvent::watches_its_host_leave) && binding.source_found_in_graveyard.is_none()
                    })
            })
            .map(|keeper| keeper.place)
            .collect();
        for place in triggers_finding_the_aura {
            let binding = match place {
                KeeperPlace::Waiting(queue_index) => self.pending_triggers.get_mut(queue_index).map(|pending| &mut pending.binding),
                KeeperPlace::Stacked(id) => self.stack_entries.get_mut(&id).and_then(|entry| entry.trigger.as_mut()),
                KeeperPlace::Resolving => self.resolving.as_mut().and_then(|resolving| resolving.trigger.as_mut()),
                KeeperPlace::Delayed => None,
            };
            if let Some(binding) = binding {
                binding.source_found_in_graveyard = Some(found);
            }
        }
    }

    /// The permanents whose printed triggers, or the copy's they entered as,
    /// read a kind `window_kinds` carries, selected before they are ordered,
    /// in CR 613.7 order: sorted on
    /// the key `battlefield_ids_ordered` sorts on, so the candidate order —
    /// which the `OrderTriggers` prompt offers — is the whole-battlefield
    /// walk's.
    pub(super) fn battlefield_readers(&self, window_kinds: EventKindMask) -> Vec<ObjectId> {
        // ORDER-FREE: put in timestamp order.
        self.in_battlefield_order(self.trigger_sources.iter().filter(|(_, kinds)| kinds.intersects(window_kinds)).map(|(&id, _)| id))
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
    pub(super) fn named_trigger_carriers(&self, window_kinds: EventKindMask) -> Vec<ObjectId> {
        if !self.continuous_effects.summary().named_unattributed_trigger_kinds.intersects(window_kinds) {
            return Vec::new();
        }
        let mut named: Vec<ObjectId> = Vec::new();
        for row in self.continuous_effects.iter() {
            if !triggered_ability_kinds(row).intersects(window_kinds) {
                continue;
            }
            named.extend(row.affected_objects.named_objects(self, row.source).into_iter().flatten());
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
    /// **And every other mover something reads later, from whatever zone**
    /// (CR 113.7a, 608.2h; `triggers-architecture.md` §3.11, decision 1): a
    /// spell countered, whose record looks back at it (CR 603.10e); the source
    /// an activation's own cost moves, which the activation's records name as
    /// it was; and any mover a keeper reads, a spell a trigger's triggering
    /// event is about, a card a draw trigger named, a card put onto the
    /// battlefield from exile. The move files the frame in the LKI store
    /// (`depart`), where its readers find it.
    ///
    /// Each decided action comes with whether a replacement applied to it,
    /// since a move one kept in its zone is announced with a frame and an
    /// untouched one performs nothing, and with its cause, which names the
    /// object a cost is paid for.
    pub(crate) fn capture_lki<'a>(&mut self, decided: impl Iterator<Item = (&'a GameAction, bool, EventCause)>) {
        let mut kept_for: Option<Vec<ObjectRef>> = None;
        for (action, replaced, cause) in decided {
            let (object, from, to) = match action {
                GameAction::Destroy { object, .. } => {
                    if self.battlefield.contains_key(object) {
                        self.capture_frame(*object);
                    }
                    continue;
                }
                // A replacement keeping the object where it was (CR 614.6): the
                // record of the act it replaced carries the object as it was.
                GameAction::ZoneChange { object, from, to, .. } if from == to => {
                    if replaced {
                        self.capture_frame(*object);
                    }
                    continue;
                }
                GameAction::ZoneChange { object, from, to, .. } => (*object, *from, *to),
                GameAction::EnterBattlefield { object, from: Some(from), .. } => (*object, *from, Zone::Battlefield),
                GameAction::PlayerLoses { player } => {
                    for id in self.battlefield_ids_ordered() {
                        if self.departs_with(id, *player) && self.zone_change_looks_back_at(id, Zone::Battlefield, None) {
                            self.capture_frame(id);
                        }
                    }
                    let kept_for = kept_for.get_or_insert_with(|| self.objects_lki_is_kept_for());
                    for object in kept_for.iter() {
                        if self.departs_with(object.id, *player) {
                            self.capture_frame(object.id);
                        }
                    }
                    continue;
                }
                _ => continue,
            };
            let countered = matches!(action, GameAction::ZoneChange { act: ZoneChangeAct::Countered, .. });
            let paid_for = matches!(cause, EventCause::Cost { paying_for, .. } if paying_for == object);
            if self.zone_change_looks_back_at(object, from, Some(to)) || countered || paid_for {
                self.capture_frame(object);
                continue;
            }
            let kept_for = kept_for.get_or_insert_with(|| self.objects_lki_is_kept_for());
            if self.object_ref(object).is_some_and(|r| kept_for.contains(&r)) {
                self.capture_frame(object);
            }
        }
    }

    /// Whether the move of `id` out of `from` for `to` (`None`: the game)
    /// looks back, so its record carries the frame its batch took: CR
    /// 603.10a's classes for the zones (`zone_change_looks_back`), asked of an
    /// object that was there to leave. A token whose entry is still being decided has no entry to look
    /// back at (`create_tokens`), and a phased-out permanent is treated as
    /// though it does not exist (CR 702.26b), so it leaves the game with its
    /// owner unframed and triggers nothing (CR 603.6c's "phased-in").
    pub(crate) fn zone_change_looks_back_at(&self, id: ObjectId, from: Zone, to: Option<Zone>) -> bool {
        zone_change_looks_back(from, to)
            && (from != Zone::Battlefield || self.battlefield.get(&id).is_some_and(|entry| !entry.phased_out))
    }

    /// The objects some keeper reads (`lki_keepers`), each once. Empty or
    /// short on the common board.
    fn objects_lki_is_kept_for(&self) -> Vec<ObjectRef> {
        let mut kept_for: Vec<ObjectRef> = Vec::new();
        for keeper in self.lki_keepers() {
            for object in keeper.objects() {
                if !kept_for.contains(&object) {
                    kept_for.push(object);
                }
            }
        }
        kept_for
    }

    /// Every keeper in the game, with what it reads.
    ///
    /// A keeper is anything that will read an object later, and so needs that
    /// object's last known information if the object leaves first (CR 113.7a,
    /// 608.2h): a triggered ability waiting to be put on the stack, a spell or
    /// ability on the stack, the one resolving, and a delayed trigger waiting
    /// for its event. Each reads its source; a trigger also reads the objects
    /// of the event that triggered it. When one of those objects is about to
    /// leave, its batch frames it (`objects_lki_is_kept_for` asks which
    /// objects anyone reads) for the LKI store. The walk only reads: the
    /// waiting triggers and the stack entries live in stores a fork shares
    /// (`state::shared_stores`), where a write copies what it touches.
    pub(super) fn lki_keepers(&self) -> impl Iterator<Item = LkiKeeper<'_>> + '_ {
        let waiting = self.pending_triggers.iter().enumerate().map(|(queue_index, pending)| LkiKeeper {
            identity: Some(pending.origin.identity()),
            trigger: Some(&pending.binding),
            place: KeeperPlace::Waiting(queue_index),
        });
        // In stack order, never the map's: a capture is a layer walk, which
        // writes a trace record.
        let stacked = self.stack.iter().filter_map(|&id| {
            let entry = self.stack_entries.get(&id)?;
            Some(LkiKeeper { identity: entry.ability_identity, trigger: entry.trigger.as_ref(), place: KeeperPlace::Stacked(id) })
        });
        let delayed = self
            .delayed_triggers
            .iter()
            .map(|delayed| LkiKeeper { identity: Some(delayed.identity()), trigger: None, place: KeeperPlace::Delayed });
        waiting.chain(stacked).chain(self.resolving_keeper()).chain(delayed)
    }

    /// The resolving object, as a keeper.
    fn resolving_keeper(&self) -> Option<LkiKeeper<'_>> {
        let resolving = self.resolving.as_ref()?;
        Some(LkiKeeper { identity: resolving.identity, trigger: resolving.trigger.as_ref(), place: KeeperPlace::Resolving })
    }

    /// How `object` last existed, as a keeper that reads it does (CR 113.7a):
    /// asked of the resolving object first, then of every keeper in turn, each
    /// through its own trigger's records before the LKI store
    /// (`lki_as_read_by`). `None` while `object` is still there, or once
    /// nothing reads it.
    pub(crate) fn lki_as_a_keeper_reads(&self, object: ObjectRef) -> Option<&Lki> {
        self.resolving_keeper()
            .into_iter()
            .chain(self.lki_keepers())
            .filter(|keeper| keeper.objects().any(|read| read == object))
            .find_map(|keeper| self.lki_as_read_by(keeper.trigger, object))
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
    pub(crate) fn lki_with_frame(&self, id: ObjectId, frame: Arc<EffectiveCharacteristics>) -> Option<Lki> {
        let object = self.objects.get(&id)?;
        let cost_choices = self.cost_choices_of(id).unwrap_or(CostChoices::NONE);
        let existence = ObjectRef { id, zone_change_epoch: object.zone_change_epoch };
        let (permanent, entry) = (self.battlefield.get(&id), self.stack_entries.get(&id));
        Some(Lki::new(existence, frame, object.owner, object.is_token, cost_choices, permanent, entry))
    }

    /// `id`'s last known information with its frame off the layer memo, for
    /// every reader but a performing batch's fallback.
    pub(crate) fn lki_now(&self, id: ObjectId) -> Option<Lki> {
        self.lki_with_frame(id, compute_characteristics(self, id)?)
    }

    /// Whether a batch in progress framed `object` and has yet to move it:
    /// `object` leaves later in the event being performed.
    pub(crate) fn leaves_later_in_this_batch(&self, object: ObjectRef) -> bool {
        self.lki_captures.iter().find(|capture| capture.object == object).is_some_and(|capture| capture.lki.is_some())
    }

    /// The capture its batch took of `object`, if one did.
    fn capture_of(&mut self, object: ObjectRef) -> Option<&mut LkiCapture> {
        self.lki_captures.iter_mut().find(|d| d.object == object)
    }

    /// The last known information a kept-for mover's batch took for it, if any:
    /// the counterpart of `take_captured_lki` for a move in no CR 603.10a
    /// class, which no batch is bound to frame.
    pub(crate) fn take_named_lki(&mut self, id: ObjectId) -> Option<Lki> {
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
            self.lki_with_frame(id, Arc::new(frame))
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
        // Every printed source, not the ones whose kinds a look-back arm
        // reads: which arms look back is `TriggerEvent::looks_back`'s to say,
        // and a kind filter here would be a second table of it. No newcomer's
        // trigger looks back but off its own move's frame.
        let mut ids = self.trigger_gate(EventKindMask::ALL, Vec::new(), OffBattlefieldLegs::Unfiltered).candidates();
        // And the multipliers, which CR 603.2d's count reads on the same board.
        for id in self.trigger_multiplier_sources_now() {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        ids.into_iter().filter_map(|id| self.object_snapshot(id)).collect()
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
    pub(super) fn candidates_now(
        &self,
        readers: Vec<ObjectId>,
        named: Vec<ObjectId>,
        newcomers: Vec<ObjectId>,
        unattributed: ZoneSet,
        zone_sources: impl Iterator<Item = ObjectId>,
    ) -> Vec<ObjectId> {
        let mut live: Vec<ObjectId> = if unattributed.contains(Zone::Battlefield) {
            self.battlefield_ids_ordered()
        } else if named.is_empty() {
            readers
        } else {
            // A permanent both printed and named is one candidate.
            let mut once: IdSet<ObjectId> = IdSet::default();
            self.in_battlefield_order(readers.iter().chain(&named).copied().filter(|&id| once.insert(id)))
        };
        let mut elsewhere: Vec<(Timestamp, ObjectId)> = zone_sources.map(|id| (self.object_timestamp(id), id)).collect();
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
    fn find_trigger_matches(&self, window: &[EventId], gate: TriggerGate<'_>, snapshots: &[LookBackSnapshot]) -> Vec<MatchedTrigger> {
        let mut live = gate.candidates();

        // Each survivor's lists from before the window's snapshotting batches.
        // A survivor no live set reaches now, like a grant's carrier after
        // the granter left, is still asked, after the rest.
        let mut before = self.lists_before(snapshots);
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

        // The frames the window's departures carry (CR 603.10a): the list each
        // object that left had, asked the arms that look back only.
        let frames: Vec<(&Lki, Zone)> = records.iter().filter_map(|(_, r)| r.event.lki()).map(|(lki, from, _)| (lki, from)).collect();

        let mut candidates: Vec<TriggerCandidate<'_>> = Vec::new();
        self.push_live_candidates(live, &mut before, &mut candidates);
        candidates.extend(frames.into_iter().map(|(lki, from)| TriggerCandidate::from_lki(self, lki, from, None)));

        let (mut matches, matched_candidates) = self.match_candidates(&records, &candidates, snapshots);
        // CR 603.2d, asked only of a window that triggered something.
        if !matches.is_empty() {
            let multipliers = self.trigger_multiplier_candidates(&records, snapshots);
            self.count_additional_triggers(&mut matches, &matched_candidates, &candidates, &multipliers, &records, snapshots);
        }
        self.diagnostics.record_trigger_dispatch(candidates.len() as u64, matches.len() as u64);
        matches
    }

    /// Each survivor's lists from before the window's snapshotting batches,
    /// by the snapshot that holds each.
    fn lists_before<'a>(&self, snapshots: &'a [LookBackSnapshot]) -> IdMap<ObjectId, Vec<(usize, &'a Lki)>> {
        let mut before: IdMap<ObjectId, Vec<(usize, &Lki)>> = IdMap::default();
        for (k, snapshot) in snapshots.iter().enumerate() {
            for frame in &snapshot.frames {
                let object = frame.lki.object();
                if self.find(object).is_some() {
                    before.entry(object.id).or_default().push((k, &frame.lki));
                }
            }
        }
        before
    }

    /// The live objects `ids` as candidates: each one's list now, then the
    /// lists `before` holds for it.
    fn push_live_candidates<'a>(
        &'a self,
        ids: Vec<ObjectId>,
        before: &mut IdMap<ObjectId, Vec<(usize, &'a Lki)>>,
        candidates: &mut Vec<TriggerCandidate<'a>>,
    ) {
        for id in ids {
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
                .map(|(snapshot, lki)| TriggerCandidateFrame::Before { lki, snapshot });
            let lists = std::iter::once(TriggerCandidateFrame::Live { chars, snapshots }).chain(before_lists);
            candidates.extend(lists.map(|frame| TriggerCandidate::live(object, existence, controller, host, frame)));
        }
    }

    /// The permanents CR 603.2d's count asks, each read as the dispatch reads
    /// a trigger's source: on the battlefield now, with its lists from before
    /// the window's snapshotting batches, then each that left the
    /// battlefield in the window, off its departure's frame. Empty, and
    /// nothing walked, on a board with no printed multiplier, no row granting
    /// or copying one, and no departure carrying one.
    fn trigger_multiplier_candidates<'a>(
        &'a self,
        records: &[(EventId, &'a EventRecord)],
        snapshots: &'a [LookBackSnapshot],
    ) -> Vec<TriggerCandidate<'a>> {
        let mut candidates: Vec<TriggerCandidate<'_>> = Vec::new();
        let mut sources = self.trigger_multiplier_sources_now();
        // A survivor whose multiplier only a list from before holds: one a
        // granter gave it, the granter gone in the window.
        for frame in snapshots.iter().flat_map(|snapshot| &snapshot.frames) {
            let object = frame.lki.object();
            if carries_a_trigger_multiplier(frame.lki.characteristics()) && self.find(object).is_some() && !sources.contains(&object.id) {
                sources.push(object.id);
            }
        }
        if !sources.is_empty() {
            self.push_live_candidates(sources, &mut self.lists_before(snapshots), &mut candidates);
        }
        for (lki, from, _) in records.iter().filter_map(|(_, record)| record.event.lki()) {
            if from == Zone::Battlefield && carries_a_trigger_multiplier(lki.characteristics()) {
                candidates.push(TriggerCandidate::from_lki(self, lki, from, None));
            }
        }
        candidates
    }

    /// The permanents that may carry a trigger multiplier now, in timestamp
    /// order: every one while a row grants or copies one (the summary's two
    /// flags), else the printed sources.
    pub(crate) fn trigger_multiplier_sources_now(&self) -> Vec<ObjectId> {
        let summary = self.continuous_effects.summary();
        if summary.any_granted_trigger_multiplier || summary.any_copied_trigger_multiplier {
            return self.battlefield_ids_ordered();
        }
        // ORDER-FREE: put in timestamp order.
        self.in_battlefield_order(self.trigger_multiplier_sources.iter().copied())
    }

    /// CR 603.2d: how many more times each match triggers, every multiplier
    /// that applies to it adding its `additional` (Panharmonicon's sixth
    /// ruling). `matched` names each match's candidate, which says how its
    /// source was read; `multipliers` are the permanents that may carry one,
    /// read the same way.
    pub(super) fn count_additional_triggers(
        &self,
        matches: &mut [MatchedTrigger],
        matched: &[usize],
        candidates: &[TriggerCandidate<'_>],
        multipliers: &[TriggerCandidate<'_>],
        records: &[(EventId, &EventRecord)],
        snapshots: &[LookBackSnapshot],
    ) {
        if multipliers.is_empty() {
            return;
        }
        let newcomers = self.window_newcomers(records);
        for (each, &candidate) in matches.iter_mut().zip(matched) {
            let source = &candidates[candidate];
            each.additional = multipliers
                .iter()
                .map(|multiplier| {
                    let newcomer = newcomers.contains(&multiplier.object);
                    self.additional_times_from(multiplier, newcomer, each, source, snapshots)
                })
                .sum();
        }
    }

    /// What one permanent's multipliers add to `matched`, whose source the
    /// dispatch read as `source`. Each adds when it is on the board the
    /// matched arm reads (CR 603.10), the source is a permanent it names, and
    /// a triggering event is the event it names.
    fn additional_times_from(
        &self,
        multiplier: &TriggerCandidate<'_>,
        newcomer: bool,
        matched: &MatchedTrigger,
        source: &TriggerCandidate<'_>,
        snapshots: &[LookBackSnapshot],
    ) -> u32 {
        // A card's ability in another zone is no permanent's: on the
        // battlefield, or off its frame from it (Teysa Karlov's first and
        // seventh rulings).
        if source.zone != Zone::Battlefield || multiplier.zone != Zone::Battlefield {
            return 0;
        }
        let arm = matched.arm();
        let chars = multiplier.frame.characteristics();
        let referents = multiplier.referents(None);
        let mut additional = 0;
        for ability in chars.abilities.iter() {
            let Some((condition, def)) = ability.effect.as_trigger_multiplier() else { continue };
            if !functions_as_static(ability, &chars.types, Zone::Battlefield) {
                continue;
            }
            let as_long_as = ConditionReferents { source_lki: multiplier.frame.lki(), triggering_event: TriggeringEventView::None };
            if condition.is_some_and(|condition| !settled_holds(condition, self, multiplier.object.id, Some(multiplier.controller), as_long_as)) {
                continue;
            }
            if !self.object_matches_filter(source.frame.information(source.object.id), &def.abilities_of, &mut { referents }) {
                continue;
            }
            let caused = matched.triggering_events.iter().any(|event| {
                let looks_back_through = LookBackSnapshot::for_record(snapshots, event.record.seq.0);
                multiplier.frame.on_the_board_read_by(arm, looks_back_through, newcomer)
                    && def.caused_by.as_ref().is_none_or(|caused_by| self.is_caused_by(caused_by, arm, &referents, &event.record))
            });
            if caused {
                additional += def.additional;
            }
        }
        additional
    }

    /// Whether `record`, which triggered `arm`, is an event a multiplier's
    /// `caused_by` names: asked of the record with the multiplier's own
    /// referents and the frames its own arm reads, whatever the doubled arm
    /// says (Gandalf the White's ninth and tenth rulings). An arm naming an
    /// act the multiplier's does not watches another event, which the
    /// record shares (Teysa Karlov's fifth).
    fn is_caused_by(&self, caused_by: &TriggerCondition, arm: &TriggerEvent, referents: &TriggerReferents<'_>, record: &EventRecord) -> bool {
        caused_by.events().iter().any(|by| {
            arm.named_act().is_none_or(|act| by.named_act() == Some(act)) && !self.triggering_events_in(by, referents, record).is_empty()
        })
    }

    /// Every candidate ability against every record of the window, in window
    /// order then candidate order — the order the `OrderTriggers` prompt will
    /// offer, which has to be process-stable end to end (§15 item 1). Which
    /// candidates there are is the caller's: the dispatcher's legs, or the
    /// audit's every object (`audit.rs`). Beside each match, the candidate it
    /// is of, which CR 603.2d's count reads its source through.
    pub(super) fn match_candidates(
        &self,
        records: &[(EventId, &EventRecord)],
        candidates: &[TriggerCandidate<'_>],
        snapshots: &[LookBackSnapshot],
    ) -> (Vec<MatchedTrigger>, Vec<usize>) {
        let mut matches: Vec<MatchedTrigger> = Vec::new();
        let mut matched_candidates: Vec<usize> = Vec::new();

        let newcomers = self.window_newcomers(records);
        let is_newcomer: Vec<bool> = candidates.iter().map(|candidate| newcomers.contains(&candidate.object)).collect();

        // One row per triggered ability, so the records loop below pays
        // only for what depends on the record (§4.2).
        let mut defs: Vec<TriggerCandidateDef<'_>> = Vec::new();
        for (index, candidate) in candidates.iter().enumerate() {
            let chars = candidate.frame.characteristics();
            for ability in chars.abilities.iter() {
                let Some(def) = ability.effect.as_trigger() else { continue };
                // CR 113.6 is asked of each ability: the object is here
                // because *some* ability of its functions here (§4.2).
                if !functions_in(ability, &chars.types, candidate.zone) {
                    continue;
                }
                defs.push(TriggerCandidateDef {
                    candidate: index,
                    identity: AbilityIdentity { source: candidate.object, ability: ability.id },
                    def,
                    stated_zones: ability.effect.stated_zones(),
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
                let outcome = self.match_def(def, row.stated_zones, candidate, answered, record, identity);
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
                let this = ArmMatch { row, arm, triggering_events, mana };
                match hits.iter_mut().find(|hit| hit.row.identity == identity) {
                    Some(earlier) if this.arm < earlier.arm => *earlier = this,
                    Some(_) => {}
                    None => hits.push(this),
                }
            }
            for ArmMatch { row, arm: matched, triggering_events, mana } in hits {
                let candidate = &candidates[row.candidate];
                let arm = &row.def.condition.events()[matched.0];
                // The ability's own def, shared: nothing writes it once the card
                // is built, so a Humility landing before placement still cannot
                // un-trigger it (CR 113.7a).
                let triggers_added = fold_trigger_occurrences(&mut matches, row.identity, arm, matched, record, triggering_events, |triggering_events| {
                    MatchedTrigger {
                        identity: row.identity,
                        controller: candidate.controller,
                        def: Arc::clone(row.def),
                        stated_zones: row.stated_zones,
                        source_card: Arc::clone(&candidate.card),
                        instances: row.instances.to_vec(),
                        event: matched,
                        triggering_events,
                        mana,
                        additional: 0,
                    }
                });
                matched_candidates.extend(std::iter::repeat_n(row.candidate, triggers_added));
            }
        }
        (matches, matched_candidates)
    }

    /// One def against one record: the first arm that matches, and the
    /// objects of its triggering events (one per trigger). `Err` names the
    /// predicate that refused.
    fn match_def(
        &self,
        def: &TriggerDef,
        stated_zones: Option<ZoneSet>,
        candidate: &TriggerCandidate<'_>,
        answered: AnsweredArms,
        record: &EventRecord,
        identity: AbilityIdentity,
    ) -> Result<(EventIndex, Vec<TriggeringEventObjects>), Refusal> {
        if matches!(def.condition, TriggerCondition::State(_)) {
            return Err(Refusal::StateTrigger);
        }
        if !self.visible_as_read(candidate) {
            return Err(Refusal::Visibility);
        }
        let referents = candidate.referents(Some(identity));
        // The intervening "if"'s "you" is the candidate's controller (CR
        // 109.5): the source's now, or its frame's for a look-back candidate,
        // and "this" is read as it left for one.
        let source = TriggerSource { object: candidate.object.id, controller: candidate.controller, lki: candidate.frame.lki() };
        self.first_arm_that_triggers(def, stated_zones, identity, &referents, record, source, |arm| {
            // CR 113.6k asks each trigger condition where it functions; the
            // pre-pass asked the ability, which is their union.
            answered.admits(arm) && condition_functions_in(stated_zones, arm, &candidate.frame.characteristics().types, candidate.zone)
        })
    }

    /// Whether `record` triggers the ability `def`, and how. The last three
    /// steps of a match, the same for an object's ability (`match_def`) and a
    /// delayed trigger (`match_delayed`), each of which asks its own
    /// questions first:
    /// 1. the first of the def's trigger events ("whenever X or Y" has two)
    ///    that reads `record` and that `asks` admits, with the objects of
    ///    each event in `record` it matches;
    /// 2. the once-per-turn limits ("only once each turn", "the first time
    ///    each turn"), checked against that event;
    /// 3. CR 603.4's intervening "if", which a trigger must pass as it
    ///    triggers.
    ///
    /// `Err` names the step that refused, which the trace records.
    pub(super) fn first_arm_that_triggers(
        &self,
        def: &TriggerDef,
        stated_zones: Option<ZoneSet>,
        identity: AbilityIdentity,
        referents: &TriggerReferents<'_>,
        record: &EventRecord,
        source: TriggerSource<'_>,
        asks: impl Fn(&TriggerEvent) -> bool,
    ) -> Result<(EventIndex, Vec<TriggeringEventObjects>), Refusal> {
        let (event_index, triggering_events) = def
            .condition
            .events()
            .iter()
            .enumerate()
            .filter(|(_, arm)| arm.reads(&record.event) && asks(arm))
            .find_map(|(index, arm)| {
                let triggering_events = self.triggering_events_in(arm, referents, record);
                (!triggering_events.is_empty()).then_some((EventIndex(index), triggering_events))
            })
            .ok_or(Refusal::TriggerCondition)?;
        let arm = &def.condition.events()[event_index.0];
        if !self.within_once_per_turn_limit(def, identity, source.controller, arm, record, referents) {
            return Err(Refusal::Limit);
        }
        let triggering_events = self.triggers_whose_intervening_if_holds(def, stated_zones, arm, record, triggering_events, source)?;
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
        stated_zones: Option<ZoneSet>,
        arm: &TriggerEvent,
        record: &EventRecord,
        triggering_events: Vec<TriggeringEventObjects>,
        source: TriggerSource<'_>,
    ) -> Result<Vec<TriggeringEventObjects>, Refusal> {
        // CR 113.6b's statement is printed as the intervening "if" and asked
        // with it, of the source where it is now, as the resolution asks it.
        if stated_zones.is_some_and(|zones| !self.objects.get(&source.object).is_some_and(|object| zones.contains(object.zone))) {
            return Err(Refusal::InterveningIf);
        }
        let Some(condition) = &def.intervening_if else { return Ok(triggering_events) };
        let holding: Vec<TriggeringEventObjects> = triggering_events
            .into_iter()
            .filter(|objects| {
                let triggering_event = match arm.multiplicity().has_one_triggering_event() {
                    true => TriggeringEventView::AtTrigger { game: self, arm, record, objects },
                    false => TriggeringEventView::None,
                };
                self.intervening_if_holds(condition, source, triggering_event)
            })
            .collect();
        if holding.is_empty() { Err(Refusal::InterveningIf) } else { Ok(holding) }
    }

    /// CR 603.4's intervening "if" of `source`'s ability, read about
    /// `triggering_event`: "you" is the source's controller (CR 109.5), and a
    /// source that has left is read as it last existed.
    pub(super) fn intervening_if_holds(&self, condition: &Condition, source: TriggerSource<'_>, triggering_event: TriggeringEventView<'_>) -> bool {
        settled_holds(condition, self, source.object, Some(source.controller), ConditionReferents { source_lki: source.lki, triggering_event })
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
                // "Your first creature spell each turn": the first of the
                // caster's spells this turn the arm admits, whatever came
                // before it (Kalamax's and Plan for All Outcomes' rulings),
                // read off the turn's cast rows, each the spell as it was cast.
                (TriggerEvent::CastsSpell { spell: Some(filter), .. }, GameEvent::SpellCast { caster, .. }) => {
                    !self.history.log.rows_of_turn(self.turn_number).any(|earlier| {
                        earlier.seq < record.seq
                            && earlier.player == Some(*caster)
                            && matches!(earlier.fact, RowFact::Cast { .. })
                            && earlier.frame_matches(|object| self.object_matches_filter(object, filter, &mut { *referents }))
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
    /// arm's record kinds (`TriggerEvent::reads`) are the gate, the `match`
    /// pairs the arm with each kind it reads, and the arm's own fields are
    /// the predicates over that record. **A field left `None` is not asked**
    /// — every one of them is an `is_none_or` — which is how "any" is spelled
    /// at the type, the convention the replacement patterns share.
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
        let events = self.triggering_events_of_pair(arm, referents, record);
        // What an instruction may name is the arm's declaration
        // (`TriggerEvent::event_roles`), so an event carries no other part.
        debug_assert!(
            events.iter().flat_map(TriggeringEventObjects::objects).all(|(role, _)| arm.event_roles().contains(&role)),
            "{arm:?} declares fewer event roles than {:?} carries",
            record.event
        );
        events
    }

    /// [`Self::triggering_events_in`]'s pairing of an arm with a record.
    fn triggering_events_of_pair(
        &self,
        arm: &TriggerEvent,
        referents: &TriggerReferents<'_>,
        record: &EventRecord,
    ) -> Vec<TriggeringEventObjects> {
        let (seq, event) = (record.seq, &record.event);
        if !arm.reads(event) {
            return Vec::new();
        }
        let triggering_event_if = |ok: bool| if ok { vec![self.triggering_event_objects(arm, record)] } else { Vec::new() };
        match (arm, event) {
            (
                TriggerEvent::ZoneChange { subject, change, by, owner, .. },
                GameEvent::ZoneChange { .. } | GameEvent::LeftTheGame { .. } | GameEvent::SameZoneMove { .. },
            ) => {
                // The resolving spell or ability caused the move, also where a
                // replacement redirected it (`replacement-architecture.md` §2b.4).
                let caused = || by.as_ref().is_none_or(|by| by.matches(self, &record.stamp.cause, &mut { *referents }));
                let (moved, left, moved_owner, moved_as_named) = match event {
                    // The subject is read as the record framed it only for an
                    // arm that looks back (CR 603.10a); "from anywhere" reads the
                    // object after the move (CR 603.6c's last sentence, 603.10).
                    GameEvent::ZoneChange { departed, from, to, act, .. } => {
                        let named = change.matches(Some(*from), *to, Some(*act)) && caused();
                        (departure_read(arm, departed), departed.object(), departed.owner(), named)
                    }
                    // CR 603.6c names this event for a leaves-the-battlefield
                    // ability, and for nothing else: a card leaving the game
                    // from a graveyard leaves no graveyard (Erebos's Titan's
                    // third ruling). Nothing narrower either — no `to`, no act,
                    // no cause. Only a phased-in permanent's, which is one its
                    // departure framed (`zone_change_looks_back_at`).
                    GameEvent::LeftTheGame { departed, from } => {
                        let leaves_the_battlefield =
                            change.from == Some(Zone::Battlefield) && change.to.is_none() && change.act.is_none() && by.is_none();
                        let framed = departed.lki().is_some();
                        let named = *from == Zone::Battlefield && framed && leaves_the_battlefield;
                        (departure_read(arm, departed), departed.object(), departed.owner(), named)
                    }
                    // The act a replacement kept in its zone (CR 614.6), which
                    // only an arm that names an act reads: a mill, not "leaves
                    // your library". The object is asked as it was, which is
                    // how it still is.
                    GameEvent::SameZoneMove { zone, act, lki, .. } => {
                        (ObjectInformation::LastKnown(lki), lki.object(), lki.owner(), change.matches(Some(*zone), *zone, Some(*act)) && caused())
                    }
                    _ => return Vec::new(),
                };
                triggering_event_if(
                    moved_as_named
                        && owner.as_ref().is_none_or(|p| self.player_ref_is(p, moved_owner, referents))
                        && self.subject_of_a_move_matches(subject, moved, left, referents, seq),
                )
            }
            (TriggerEvent::BecomesTapped { subject }, GameEvent::Tapped { object })
            | (TriggerEvent::BecomesUntapped { subject }, GameEvent::Untapped { object }) => {
                triggering_event_if(self.named_subject_matches(subject, *object, referents, seq))
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
                GameEvent::ManaAdded { source, mana: added, tapped_for_mana: tapped, .. },
            ) => {
                let added = |m: ManaType| added.iter().any(|(t, n)| *t == m && *n > 0);
                let producer = ManaProducer { id: source.id, tapped_for_mana: *tapped, lki: event.source_lki() };
                triggering_event_if(self.mana_added_arm_matches(arm, producer, Some(*source), added, referents, seq))
            }
            (
                TriggerEvent::DamageDealt { source, recipient, combat, excess, .. },
                GameEvent::DamageDealt { source: dealt_by, recipient: dealt_to, is_combat, excess: dealt_excess, .. },
            ) => {
                // The source as it dealt the damage (CR 608.2h): as it last
                // existed once the existence that dealt it has ended, before
                // the damage or later in this event, and else as it is now.
                let as_it_dealt = record.event.source_lki().or_else(|| self.lki(*dealt_by));
                let source_read = ObjectInformation::of(dealt_by.id, as_it_dealt);
                triggering_event_if(
                    self.damage_recipient_matches(recipient, *dealt_to, referents, seq)
                        && combat.is_none_or(|c| c == *is_combat)
                        && (!*excess || *dealt_excess > 0)
                        && self.subject_matches_existence(source, source_read, Some(*dealt_by), referents, seq),
                )
            }
            // Announced as the batch decided, and read at its close, so the
            // recipient is asked as the damage left it (CR 603.10).
            (TriggerEvent::DamageIsPrevented { recipient, .. }, GameEvent::DamagePrevented { recipient: prevented_to, .. }) => {
                triggering_event_if(self.damage_recipient_matches(recipient, *prevented_to, referents, seq))
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
            // The direction is the split.
            (TriggerEvent::GainsLife { player: who, .. }, GameEvent::LifeChanged { player_id, .. }) => triggering_event_if(
                matches!(event.life_change(), Some(LifeChange::Gained(_)))
                    && who.as_ref().is_none_or(|p| self.player_ref_is(p, *player_id, referents)),
            ),
            (TriggerEvent::LosesLife { player: who, .. }, GameEvent::LifeChanged { player_id, .. }) => triggering_event_if(
                matches!(event.life_change(), Some(LifeChange::Lost(_)))
                    && who.as_ref().is_none_or(|p| self.player_ref_is(p, *player_id, referents)),
            ),
            (TriggerEvent::CastsSpell { caster, spell }, GameEvent::SpellCast { spell: cast, caster: who }) => {
                let spell_ok = match spell {
                    None => true,
                    Some(filter) => self.subject_matches(
                        &TriggerSubject::Filter(filter.clone()),
                        ObjectInformation::Current(cast.id),
                        referents,
                        seq,
                    ),
                };
                triggering_event_if(spell_ok && caster.as_ref().is_none_or(|p| self.player_ref_is(p, *who, referents)))
            }
            // CR 603.10e: the spell as it was on the stack, and whoever cast it.
            (TriggerEvent::IsCountered { spell, caster }, GameEvent::SpellCountered { lki, .. }) => {
                let cast_by = lki.entry().and_then(StackEntry::caster);
                triggering_event_if(
                    caster.as_ref().is_none_or(|p| cast_by.is_some_and(|who| self.player_ref_is(p, who, referents)))
                        && spell.as_ref().is_none_or(|filter| {
                            let filter = TriggerSubject::Filter(filter.clone());
                            self.subject_matches(&filter, ObjectInformation::LastKnown(lki), referents, seq)
                        }),
                )
            }
            (
                TriggerEvent::EntersBattlefield { subject, controller, from, was_cast, .. },
                GameEvent::PermanentEnteredBattlefield { object, controller: rc },
            ) => {
                let object_id = &object.id;
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
                        && self.named_subject_matches(subject, *object, referents, seq),
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
                    .filter(|&&(declared, target)| {
                        self.named_subject_matches(attacker, declared, referents, seq)
                            && defender.as_ref().is_none_or(|asked| self.defender_matches(asked, target, referents, seq))
                    })
                    .filter_map(|&(declared, target)| Some((declared, self.attacked(target)?)));
                match occurrence {
                    AttackOccurrence::Attacker => {
                        attacks.map(|(attacker, defender)| TriggeringEventObjects::Attack { attacker, defender }).collect()
                    }
                    // Each player, planeswalker or battle once, in the order its
                    // first matching attacker was declared. The CR fixes no
                    // order among one event's occurrences; a block reads each
                    // side the same way (`block_events`).
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
            (TriggerEvent::Blocks { .. }, GameEvent::BlockersDeclared { .. } | GameEvent::BecameBlocked { .. }) => {
                self.block_events(arm, record, referents)
            }
            // The source as it was activated, on the battlefield: off its frame
            // when a cost moved it (Harsh Mentor's second ruling).
            (
                TriggerEvent::ActivatesAbility { player, source, mana_ability },
                GameEvent::AbilityActivated { identity, controller, mana_ability: was_mana, from, .. },
            ) => {
                let read = ObjectInformation::of(identity.source.id, record.event.source_lki());
                let matched = *from == Zone::Battlefield
                    && mana_ability.is_none_or(|asked| asked == *was_mana)
                    && player.as_ref().is_none_or(|p| self.player_ref_is(p, *controller, referents))
                    && self.subject_matches(source, read, referents, seq);
                if matched { vec![TriggeringEventObjects::Object(identity.source)] } else { Vec::new() }
            }
            // Read after the event: the target as it is now, and the spell or
            // ability on the stack, a spell by its characteristics there.
            (
                TriggerEvent::BecomesTarget { target: watched, by: kind, controller: whose, .. },
                GameEvent::Targeted { target, by, ability_source, controller, .. },
            ) => {
                let target_ok = self.object_or_player_matches(watched, *target, referents, seq);
                let kind_ok = kind.admits(ability_source.is_none())
                    && match kind {
                        TargetedBy::Spell(Some(filter)) => {
                            self.subject_matches(&TriggerSubject::Filter(filter.clone()), ObjectInformation::Current(by.id), referents, seq)
                        }
                        TargetedBy::Spell(None) | TargetedBy::SpellOrAbility | TargetedBy::Ability => true,
                    };
                let whose_ok = whose.as_ref().is_none_or(|p| self.player_ref_is(p, *controller, referents));
                if target_ok && kind_ok && whose_ok {
                    vec![TriggeringEventObjects::Target { target: *target, by: *by }]
                } else {
                    Vec::new()
                }
            }
            (TriggerEvent::AbilityTriggers { caused_by, source }, GameEvent::AbilityTriggered { origin, caused_by: cause, .. }) => {
                let source_ok = match source {
                    None => true,
                    Some(filter) => self.subject_matches(
                        &TriggerSubject::Filter(filter.clone()),
                        ObjectInformation::Current(origin.source()),
                        referents,
                        seq,
                    ),
                };
                // The arm it names, read in full of the record that caused the
                // trigger, as a multiplier's "caused by" is (`is_caused_by`).
                let cause_ok = match caused_by {
                    None => true,
                    Some(inner) => cause
                        .and_then(|cause| self.events.record(cause))
                        .is_some_and(|caused| !self.triggering_events_in(inner, referents, caused).is_empty()),
                };
                triggering_event_if(source_ok && cause_ok)
            }
            // The same ability of the same object, whichever instance (§6.5).
            (
                TriggerEvent::AbilityResolves { identity: IdentityRef::ThisAbility },
                GameEvent::AbilityResolved { identity: resolved, .. },
            ) => triggering_event_if(referents.this_ability.is_some_and(|this| this.ignoring_ability_grant() == resolved.ignoring_ability_grant())),
            // Read after the event: the attach is done when the dispatch asks
            // (Siona, Captain of the Pyleas' fourth ruling).
            (
                TriggerEvent::BecomesAttached { attachment: watched, host: on },
                GameEvent::Attached { attachment, host, .. },
            ) => triggering_event_if(
                self.named_subject_matches(watched, *attachment, referents, seq)
                    && self.named_subject_matches(on, *host, referents, seq),
            ),
            // CR 603.10c: each end as it was just before, which for one that
            // left in this event is its departure's frame.
            (
                TriggerEvent::BecomesUnattached { attachment: watched, former_host: from },
                GameEvent::Unattached { attachment, former_host, .. },
            ) => triggering_event_if(
                self.subject_matches(
                    watched, self.subject_read_looking_back(*attachment), referents, seq,
                ) && self.subject_matches(
                    from, self.subject_read_looking_back(*former_host), referents, seq,
                ),
            ),
            // CR 603.10d: the subject as it was before the change, "you" its
            // controller then (the candidate's, off its frame from before).
            (TriggerEvent::ControlChanges { subject, from, to }, GameEvent::ControlChanged { to: gained, lki }) => {
                triggering_event_if(
                    from.as_ref().is_none_or(|p| self.player_ref_is(p, lki.characteristics().controller, referents))
                        && to.as_ref().is_none_or(|p| self.player_ref_is(p, *gained, referents))
                        && self.subject_matches(subject, ObjectInformation::LastKnown(lki), referents, seq),
                )
            }
            // Leaving the battlefield is losing control of it (Khârn the
            // Betrayer's first ruling), to nobody: so only an arm that names no
            // one gaining it, and its controller as it left.
            (
                TriggerEvent::ControlChanges { subject, from, to: None },
                GameEvent::ZoneChange { from: left, departed, .. } | GameEvent::LeftTheGame { from: left, departed },
            ) => match departed.lki() {
                Some(lki) if *left == Zone::Battlefield => triggering_event_if(
                    from.as_ref().is_none_or(|p| self.player_ref_is(p, lki.characteristics().controller, referents))
                        && self.subject_matches(subject, ObjectInformation::LastKnown(lki), referents, seq),
                ),
                _ => Vec::new(),
            },
            (TriggerEvent::CountersPutOn { .. }, GameEvent::CountersPutOn { .. })
            | (TriggerEvent::CountersRemovedFrom { .. }, GameEvent::CountersRemovedFrom { .. }) => {
                self.counter_events(arm, record, referents)
            }
            // The definition the creation made the token from, after any
            // replacement (Ajani, Nacatl Avenger's eighth ruling).
            (
                TriggerEvent::CreatesToken { owner, definition, .. },
                GameEvent::TokenCreated { owner: creator, definition: made_from, .. },
            ) => triggering_event_if(
                owner.as_ref().is_none_or(|p| self.player_ref_is(p, *creator, referents))
                    && definition.as_deref().is_none_or(|definition| definition == made_from.as_ref()),
            ),
            (TriggerEvent::Scries { player }, GameEvent::Scried { player_id, .. }) => {
                triggering_event_if(player.as_ref().is_none_or(|p| self.player_ref_is(p, *player_id, referents)))
            }
            // Every arm, by name, so a new arm does not compile until it is matched
            // here. The gate admits only the kinds an arm reads, and each such
            // pair has its case above (`each_admitted_pair_has_its_case`).
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
                | TriggerEvent::AbilityResolves { .. }
                | TriggerEvent::CreatesToken { .. }
                | TriggerEvent::Scries { .. },
                _,
            ) => {
                debug_assert!(false, "{arm:?} reads {event:?}, and the matcher has no case for the pair");
                Vec::new()
            }
        }
    }

    /// The declaration's occurrences of a block arm (CR 509.3a–g), in the
    /// record's order, each side asked as it is now (CR 509.3f). Read once
    /// for each blocking creature or each blocked one, the other side's
    /// creatures that match are the ones it counts, and the occurrence needs
    /// one of them, or `count`'s many (CR 509.3e).
    fn block_events(&self, arm: &TriggerEvent, record: &EventRecord, referents: &TriggerReferents<'_>) -> Vec<TriggeringEventObjects> {
        // CR 509.3c: an effect's block is a "becomes blocked" with no creature
        // blocking, so no arm that names a blocker, counts them or asks for
        // the only one reads it (509.3d, 509.3e, 506.5).
        if let (
            TriggerEvent::Blocks { blocker: TriggerSubject::Filter(ObjectFilter::All), attacker, occurrence: BlockOccurrence::Blocked, count: None, alone: false, .. },
            GameEvent::BecameBlocked { attacker: blocked },
        ) = (arm, &record.event)
        {
            let watched = self.named_subject_matches(attacker, *blocked, referents, record.seq);
            return watched.then_some(TriggeringEventObjects::BlockedBy { attacker: *blocked, blockers: Vec::new() }).into_iter().collect();
        }
        let (
            TriggerEvent::Blocks { blocker, attacker, occurrence, count, alone, .. },
            GameEvent::BlockersDeclared { blocks, unblocked },
        ) = (arm, &record.event)
        else {
            return Vec::new();
        };
        let seq = record.seq;
        // Each creature on one side of the declaration once, in the order it
        // first appears there. The CR fixes no order among one event's
        // occurrences; an attack's defenders are read the same way.
        let each_on = |side: fn(&(ObjectRef, ObjectRef)) -> ObjectRef| {
            let mut each: Vec<ObjectRef> = Vec::new();
            for pair in blocks {
                if !each.contains(&side(pair)) {
                    each.push(side(pair));
                }
            }
            each
        };
        let blocker_of = |&(b, _): &(ObjectRef, ObjectRef)| b;
        let attacker_of = |&(_, a): &(ObjectRef, ObjectRef)| a;
        // CR 506.5: the only creature declared as a blocker.
        if *alone && each_on(blocker_of).len() != 1 {
            return Vec::new();
        }
        let is = |subject: &TriggerSubject, named: ObjectRef| self.named_subject_matches(subject, named, referents, seq);
        let enough = |n: usize| count.unwrap_or(CountIs::AtLeast(1)).satisfied_by(n as u64);
        match *occurrence {
            BlockOccurrence::Block => blocks
                .iter()
                .filter(|&&(b, a)| is(blocker, b) && is(attacker, a))
                .map(|&(b, a)| TriggeringEventObjects::Block { blocker: b, attacker: a })
                .collect(),
            BlockOccurrence::Blocker => each_on(blocker_of)
                .into_iter()
                    .filter(|&b| is(blocker, b))
                    .filter_map(|b| {
                        let attackers: Vec<ObjectRef> =
                            blocks.iter().filter(|&&(of, a)| of == b && is(attacker, a)).map(|&(_, a)| a).collect();
                        enough(attackers.len()).then_some(TriggeringEventObjects::Blocking { blocker: b, attackers })
                    })
                    .collect(),
            BlockOccurrence::Blocked => each_on(attacker_of)
                .into_iter()
                    .filter(|&a| is(attacker, a))
                    .filter_map(|a| {
                        let blockers: Vec<ObjectRef> =
                            blocks.iter().filter(|&&(b, of)| of == a && is(blocker, b)).map(|&(b, _)| b).collect();
                        enough(blockers.len()).then_some(TriggeringEventObjects::BlockedBy { attacker: a, blockers })
                    })
                    .collect(),
            BlockOccurrence::Unblocked => unblocked
                .iter()
                .filter(|&&a| is(attacker, a))
                .map(|&a| TriggeringEventObjects::Unblocked { attacker: a })
                .collect(),
        }
    }

    /// A counter record's events for a counter arm that reads it. It matches
    /// when it passes the arm's three questions, each of which the arm's
    /// `None` leaves unasked: the counter's kind ("a +1/+1 counter"), the
    /// player who put or removed them ("you put"), and what they are on. A
    /// removal by a rule names no player (CR 704.5q, 120.3c), so no `by`
    /// matches it. Then a counter is an occurrence of a per-counter arm, so a
    /// record of three holds three events (Protean Hydra's fourth ruling),
    /// and one event otherwise, which the "one or more" readings fold.
    fn counter_events(&self, arm: &TriggerEvent, record: &EventRecord, referents: &TriggerReferents<'_>) -> Vec<TriggeringEventObjects> {
        let (TriggerEvent::CountersPutOn { subject: watched, counter_type, by, multiplicity }
        | TriggerEvent::CountersRemovedFrom { subject: watched, counter_type, by, multiplicity }) = arm
        else {
            return Vec::new();
        };
        let (on, kind, n, who) = match &record.event {
            GameEvent::CountersPutOn { subject, counter_type, n, by, .. } => (*subject, *counter_type, *n, Some(*by)),
            GameEvent::CountersRemovedFrom { subject, counter_type, n, by, .. } => (*subject, *counter_type, *n, *by),
            _ => return Vec::new(),
        };
        let matched = counter_type.is_none_or(|k| k == kind)
            && by.as_ref().is_none_or(|p| who.is_some_and(|who| self.player_ref_is(p, who, referents)))
            && self.object_or_player_matches(watched, on, referents, record.seq);
        if !matched {
            return Vec::new();
        }
        vec![self.triggering_event_objects(arm, record); multiplicity.counter_occurrences(n).0]
    }

    /// Whether the object or player a record names is the one an arm's
    /// [`ObjectOrPlayerSubject`] watches, for both arms that have one: a
    /// counter arm's (what the counters went on or came off) and a targeting
    /// arm's (what became the target). `watched` is the card's words, `it`
    /// the record's fact. "This creature"
    /// or "a creature you control" (`ObjectOrPlayerSubject::Object`) asks the
    /// object `it` names the question every object arm asks, of the object as
    /// the event found it (CR 603.10). "You" or "a player"
    /// (`ObjectOrPlayerSubject::Player`) asks the player `it` names, any
    /// player for `None`. A player never matches an object arm, nor the
    /// reverse.
    fn object_or_player_matches(
        &self,
        watched: &ObjectOrPlayerSubject,
        it: ObjectOrPlayer,
        referents: &TriggerReferents<'_>,
        seq: EventId,
    ) -> bool {
        match (watched, it) {
            (ObjectOrPlayerSubject::Object(which), ObjectOrPlayer::Object(object)) => self.named_subject_matches(which, object, referents, seq),
            (ObjectOrPlayerSubject::Player(who), ObjectOrPlayer::Player(player)) => {
                who.as_ref().is_none_or(|p| self.player_ref_is(p, player, referents))
            }
            (ObjectOrPlayerSubject::Object(_), ObjectOrPlayer::Player(_)) | (ObjectOrPlayerSubject::Player(_), ObjectOrPlayer::Object(_)) => false,
        }
    }

    /// Whether `arm`, one event of a trigger condition, is a `ManaAdded` arm —
    /// "whenever [something] is tapped for mana" or "adds mana" (CR 106.12a) —
    /// that the mana `producer` just made sets off: tapped for mana or not,
    /// with the types `added` admits, and made by the object the arm names
    /// for `referents`' ability (Wild Growth's enchanted land). `made_by` is
    /// the existence the record names, and `None` before any record.
    fn mana_added_arm_matches(
        &self,
        arm: &TriggerEvent,
        producer: ManaProducer<'_>,
        made_by: Option<ObjectRef>,
        added: impl Fn(ManaType) -> bool,
        referents: &TriggerReferents<'_>,
        seq: EventId,
    ) -> bool {
        let TriggerEvent::ManaAdded { source, tapped_for_mana, mana } = arm else { return false };
        // A producer its own cost moved is read as it last existed (CR 608.2h).
        let read = ObjectInformation::of(producer.id, producer.lki);
        tapped_for_mana.is_none_or(|t| t == producer.tapped_for_mana)
            && mana.is_none_or(added)
            && self.subject_matches_existence(source, read, made_by, referents, seq)
    }

    /// [`Self::mana_added_arm_matches`], asked before the mana is made.
    ///
    /// `oracle::mana_supply` decides whether a player could pay a cost
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
        let referents = TriggerReferents::of_object(permanent, controller, object.owner, crate::oracle::board::host_of(self, permanent), None);
        let producer = ManaProducer { id: producer, tapped_for_mana: tapped, lki: None };
        self.mana_added_arm_matches(arm, producer, None, added, &referents, self.events.next_seq())
    }

    /// Whether damage dealt, or prevented, to `target` is to the player or
    /// permanent `recipient` names; a permanent is read now.
    fn damage_recipient_matches(
        &self,
        recipient: &DamageRecipient,
        target: ObjectOrPlayer,
        referents: &TriggerReferents<'_>,
        seq: EventId,
    ) -> bool {
        match (recipient, target) {
            (DamageRecipient::Any, _) => true,
            (DamageRecipient::Player(who), ObjectOrPlayer::Player(pid)) => {
                who.as_ref().is_none_or(|p| self.player_ref_is(p, pid, referents))
            }
            (DamageRecipient::Object(filter), ObjectOrPlayer::Object(object)) => match filter {
                None => true,
                Some(filter) => self.named_subject_matches(&TriggerSubject::Filter(filter.clone()), object, referents, seq),
            },
            (DamageRecipient::Player(_), ObjectOrPlayer::Object(_)) | (DamageRecipient::Object(_), ObjectOrPlayer::Player(_)) => false,
        }
    }

    /// "Which object" (§3.3): this object is the source itself, its host what it is
    /// attached to, a filter is read against the subject — as `read` says.
    /// `event_id` names the matched record.
    fn subject_matches(&self, subject: &TriggerSubject, object: ObjectInformation<'_>, referents: &TriggerReferents<'_>, event_id: EventId) -> bool {
        self.subject_matches_existence(subject, object, None, referents, event_id)
    }

    /// [`Self::subject_matches`] for an object a record names, `named` being
    /// the existence the event was about (CR 400.7): "this object" and "that
    /// creature" compare it, though the object may have moved since in the
    /// same event, and a filter reads the object as the event found it. That
    /// is the object as it is now while the existence lasts, and its last
    /// known information once it has ended: counters a state-based check
    /// removes come off the creature the same check kills (CR 704.3).
    fn named_subject_matches(&self, subject: &TriggerSubject, named: ObjectRef, referents: &TriggerReferents<'_>, event_id: EventId) -> bool {
        self.subject_matches_existence(subject, ObjectInformation::of(named.id, self.lki(named)), Some(named), referents, event_id)
    }

    /// [`Self::subject_matches`] for the object a move moved, `left` being
    /// the existence that left. A delayed trigger's own source and the
    /// objects it refers to ("when that creature dies") are each one
    /// existence: once an object moves it is a new object (CR 400.7), so
    /// asking whether a record is about one needs the existence the record
    /// is about. Every other record leaves that to the read, where it is the
    /// existence a frame keeps or the object as it is now. A move read as it
    /// is now (an arm that does not look back) would give the existence the
    /// move made, so the move passes the one that left.
    fn subject_of_a_move_matches(
        &self,
        subject: &TriggerSubject,
        object: ObjectInformation<'_>,
        left: ObjectRef,
        referents: &TriggerReferents<'_>,
        event_id: EventId,
    ) -> bool {
        self.subject_matches_existence(subject, object, Some(left), referents, event_id)
    }

    /// The one body of the two above: `existence`, where given, is which
    /// existence of its object the record is about.
    fn subject_matches_existence(
        &self,
        subject: &TriggerSubject,
        object: ObjectInformation<'_>,
        existence: Option<ObjectRef>,
        referents: &TriggerReferents<'_>,
        event_id: EventId,
    ) -> bool {
        let id = object.id();
        // Read only for "this object" and "that creature", which compare
        // existences: the given one, else the frame's, else the object's now.
        let record_existence = || {
            existence.or_else(|| match &object {
                ObjectInformation::LastKnown(lki) => Some(lki.object()),
                ObjectInformation::Current(id) | ObjectInformation::Characteristics(id, ..) => self.object_ref(*id),
            })
        };
        match subject {
            // Asks nothing of the object, so one the store has lost still matches.
            TriggerSubject::Filter(ObjectFilter::All) => true,
            TriggerSubject::Object(Antecedent::ThisObject) => referents.this.is(id, record_existence()),
            // An arm has no triggering event to name yet; the registry refuses it.
            TriggerSubject::Object(Antecedent::Triggering(_)) => false,
            // Only a permanent has anything attached to it (CR 301.5, 303.4).
            // One read as it was just before this event is asked as that frame
            // has it (CR 603.10a), since the Aura or Equipment that stays has
            // been unattached since. A source gone before the event was no
            // permanent at it, and "whenever enchanted creature deals damage"
            // never matches it, even an Aura on a new object that shares its id
            // after a flicker (CR 400.7).
            // Nothing is attached to a permanent not yet on the battlefield,
            // and a history row's frame keeps no attachments.
            TriggerSubject::Object(Antecedent::ThisObjectsHost) => match object {
                ObjectInformation::Current(id) | ObjectInformation::Characteristics(id, _, FrameContext::Now) => referents.host == Some(id),
                ObjectInformation::LastKnown(lki) => !self.gone_before(event_id, lki) && lki.had_attached(referents.this.id()),
                ObjectInformation::Characteristics(_, _, FrameContext::Entering | FrameContext::HistoryRow(_)) => false,
            },
            TriggerSubject::Object(Antecedent::Referred) => {
                let existence = record_existence();
                referents.referred.iter().any(|referred| existence == Some(referred.object))
            }
            TriggerSubject::Filter(filter) => self.object_matches_filter(object, filter, &mut { *referents }),
        }
    }

    /// Whether record `event_id` reads `lki` as its source's from before the
    /// event: a source that left before dealing damage, activating or making
    /// mana (CR 608.2h, `GameEvent::source_lki`).
    fn gone_before(&self, event_id: EventId, lki: &Lki) -> bool {
        self.events.record(event_id).and_then(|record| record.event.source_lki()).is_some_and(|source| source.object() == lki.object())
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
            self.subject_matches(&TriggerSubject::Filter(filter.clone()), ObjectInformation::Current(id), referents, seq)
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
    fn subject_read_looking_back(&self, object: ObjectRef) -> ObjectInformation<'_> {
        if self.find(object).is_some() && self.battlefield.contains_key(&object.id) {
            return ObjectInformation::Current(object.id);
        }
        self.events
            .held()
            .iter()
            .find_map(|record| record.event.lki().map(|(lki, ..)| lki).filter(|lki| lki.object() == object))
            .map_or(ObjectInformation::Current(object.id), ObjectInformation::LastKnown)
    }

    /// The zone the permanent at `id` entered from, off its own zone change
    /// earlier in the same window — `None` for a token, which came from
    /// nowhere (CR 111.2). The entry's performer announces both, so the
    /// window holds the move whenever it holds the entry.
    pub(super) fn entry_origin(&self, id: ObjectId, entered_at: EventId) -> Option<Zone> {
        let held = self.events.held();
        let before = held.partition_point(|r| r.seq < entered_at);
        for record in held[..before].iter().rev() {
            match &record.event {
                GameEvent::ZoneChange { departed, to: Zone::Battlefield, from, .. } if departed.object().id == id => {
                    return Some(*from)
                }
                GameEvent::TokenCreated { token, .. } if token.id == id => return None,
                _ => {}
            }
        }
        None
    }

    /// The objects `record`'s one triggering event is about (§3.4), each as
    /// the existence its record names, which the trigger remembers, grouped
    /// as the arm's `event_roles()` groups them; a part this event fills with
    /// a player names no object. Its record is one `triggering_events_in`
    /// matched to it. An attack record holds one event per attacker, which
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
            | TriggerEvent::ControlChanges { .. }
            | TriggerEvent::CreatesToken { .. } => self.single_object(&record.event),
            TriggerEvent::DamageDealt { .. } => self.damage_objects(&record.event),
            TriggerEvent::DamageIsPrevented { .. } => self.prevention_objects(&record.event),
            TriggerEvent::CountersPutOn { .. } | TriggerEvent::CountersRemovedFrom { .. } => {
                match &record.event {
                    GameEvent::CountersPutOn { subject: ObjectOrPlayer::Object(object), .. }
                    | GameEvent::CountersRemovedFrom { subject: ObjectOrPlayer::Object(object), .. } => {
                        TriggeringEventObjects::Object(*object)
                    }
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
            | TriggerEvent::AbilityResolves { .. }
            | TriggerEvent::Scries { .. } => TriggeringEventObjects::NoObject,
        }
    }

    /// The one object a one-object event is about: the card or permanent
    /// that moved, was tapped, untapped, entered or changed control, the
    /// token created, the mana's source, the card drawn, or the spell.
    fn single_object(&self, event: &GameEvent) -> TriggeringEventObjects {
        let object = match event {
            // The existence that left the game, which the store no longer
            // holds by the time the record is read (CR 800.4a).
            GameEvent::LeftTheGame { departed, .. } => departed.object(),
            // The new object the move made, so "return it" after a dies
            // trigger finds the card only in the graveyard it went to (CR
            // 603.6c, 400.7e).
            GameEvent::ZoneChange { departed, new_epoch, .. } => {
                ObjectRef { id: departed.object().id, zone_change_epoch: *new_epoch }
            }
            GameEvent::SameZoneMove { lki, .. } | GameEvent::ControlChanged { lki, .. } => lki.object(),
            // The card the counter put into its owner's graveyard, as the
            // dispatch finds it: the countering's move is a record of its own.
            GameEvent::SpellCountered { lki, .. } => match self.object_ref(lki.object().id) {
                Some(card) => card,
                None => return TriggeringEventObjects::NoObject,
            },
            GameEvent::Tapped { object }
            | GameEvent::Untapped { object }
            | GameEvent::PermanentEnteredBattlefield { object, .. }
            | GameEvent::TokenCreated { token: object, .. }
            | GameEvent::ManaAdded { source: object, .. }
            | GameEvent::CardDrawn { card: object, .. }
            | GameEvent::SpellCast { spell: object, .. } => *object,
            _ => return TriggeringEventObjects::NoObject,
        };
        TriggeringEventObjects::Object(object)
    }

    /// A damage event's source and recipient. A source that left before
    /// dealing the damage is the existence that dealt it (CR 608.2h).
    fn damage_objects(&self, event: &GameEvent) -> TriggeringEventObjects {
        match event {
            GameEvent::DamageDealt { source, recipient, .. } => TriggeringEventObjects::Damage { source: *source, recipient: *recipient },
            _ => TriggeringEventObjects::NoObject,
        }
    }

    /// The object a prevention effect prevented damage to; a player is
    /// none.
    fn prevention_objects(&self, event: &GameEvent) -> TriggeringEventObjects {
        match event {
            GameEvent::DamagePrevented { recipient: ObjectOrPlayer::Object(recipient), .. } => {
                TriggeringEventObjects::Prevention { recipient: *recipient }
            }
            _ => TriggeringEventObjects::NoObject,
        }
    }

    /// An attachment and its host, or its former host. An unattach names
    /// each end as the existence the record names (Grafted Wargear's first
    /// ruling).
    fn attachment_objects(&self, event: &GameEvent) -> TriggeringEventObjects {
        match event {
            GameEvent::Attached { attachment, host, .. } => TriggeringEventObjects::Attach { attachment: *attachment, host: *host },
            GameEvent::Unattached { attachment, former_host, .. } => {
                TriggeringEventObjects::Attach { attachment: *attachment, host: *former_host }
            }
            _ => TriggeringEventObjects::NoObject,
        }
    }

    /// The window's newcomers: a window is one event, so each was somewhere
    /// else immediately before every record of it.
    fn window_newcomers(&self, records: &[(EventId, &EventRecord)]) -> Vec<ObjectRef> {
        records.iter().filter_map(|(_, record)| self.newcomer_from(&record.event)).collect()
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
            GameEvent::TokenCreated { token, .. } => Some(*token),
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
    /// — the Aura was not tapped). "Its controller adds" is the host as
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

/// Adds one record's matches of one ability to the window's triggers (CR
/// 603.2c): `triggering_events`, the occurrences of `arm` (the ability
/// `identity`'s trigger event number `event`) that `record` holds. An arm
/// read per occurrence makes a trigger of each. A "one or more" arm's
/// occurrences join the window's one trigger of that arm, or, for an arm
/// read once per subject, the window's one trigger about the record's
/// object or player. `trigger` builds a new trigger from its events.
/// Returns how many new triggers it added, which the dispatch counts to keep
/// its list of each trigger's candidate in step.
pub(super) fn fold_trigger_occurrences(
    matches: &mut Vec<MatchedTrigger>,
    identity: AbilityIdentity,
    arm: &TriggerEvent,
    event: EventIndex,
    record: &EventRecord,
    triggering_events: Vec<TriggeringEventObjects>,
    trigger: impl Fn(Vec<TriggeringEvent>) -> MatchedTrigger,
) -> usize {
    let these = triggering_events.into_iter().map(|objects| TriggeringEvent { record: record.clone(), objects });
    let subject = match arm.multiplicity() {
        Multiplicity::PerOccurrence => {
            let before = matches.len();
            matches.extend(these.map(|triggering_event| trigger(vec![triggering_event])));
            return matches.len() - before;
        }
        Multiplicity::OncePerEvent => None,
        Multiplicity::OncePerSubject => Some(arm.occurrence_subject(&record.event)),
    };
    let joins = |m: &MatchedTrigger| {
        m.identity == identity
            && m.event == event
            && subject.is_none_or(|subject| m.triggering_events.first().and_then(|e| arm.occurrence_subject(&e.record.event)) == subject)
    };
    match matches.iter_mut().find(|m| joins(m)) {
        Some(folded) => {
            folded.triggering_events.extend(these);
            0
        }
        None => {
            matches.push(trigger(these.collect()));
            1
        }
    }
}

fn carries_a_trigger_multiplier(chars: &EffectiveCharacteristics) -> bool {
    chars.abilities.iter().any(|ability| ability.effect.as_trigger_multiplier().is_some())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event::{EventStamp, RecordPlaces};
    use crate::types::triggers::samples;

    /// Every pair of arm and record kind the gate admits reaches its own case
    /// of `triggering_events_in`, whose catch-all is a debug check: one arm
    /// of each kind against one record of each.
    #[test]
    fn each_admitted_pair_has_its_case() {
        let game = crate::test_support::setup_two_player_game();
        let referents = TriggerReferents::of_object(ObjectId::UNASSIGNED, 0, 0, None, None);
        let mut admitted = 0;
        for (arm, _) in samples::arms() {
            for (_, event) in samples::records(samples::lki()) {
                if !arm.reads(&event) {
                    continue;
                }
                let stamp = EventStamp { batch: None, cause: EventCause::Harness };
                let record = EventRecord { seq: EventId(0), event, stamp, names: None, places: RecordPlaces::NOT_COUNTED };
                game.triggering_events_in(&arm, &referents, &record);
                admitted += 1;
            }
        }
        assert_eq!(admitted, 38);
    }
}
