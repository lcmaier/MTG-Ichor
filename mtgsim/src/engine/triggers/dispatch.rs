//! Detection — the dispatch (`triggers-architecture.md` §4).
//!
//! A dispatch is the matcher's run over one **window**: every record one
//! batch stamped, run when the outermost `execute_actions` that opened it is
//! about to return, after its riders; or one unbatched record, run inside
//! `emit_event` before it returns (§4.1). The instant is still synchronous
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
//! it had before ([`LookBackSnapshot`]). On the pools as they stood before
//! this phase every set is empty and a dispatch is the probes and nothing
//! else.

use std::sync::Arc;

use crate::engine::actions::{ActionContext, GameAction};
use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::condition::settled_holds;
use crate::engine::layers::types::{EffectiveCharacteristics, Timestamp};
use crate::engine::resolve::ResolutionContext;
use crate::engine::trace_records;
use crate::engine::zone_function::{condition_functions_in, functions_in};
use crate::events::event::{BatchId, DamageTarget, EventRecord, EventSeq, GameEvent};
use crate::objects::card_data::{AbilityDef, CardData};
use crate::oracle::characteristics::controller_or_owner;
use crate::state::continuous_effects::triggered_ability_kinds;
use crate::state::game_state::{AbilityIdentity, GameState};
use crate::types::effects::{Effect, EffectRecipient, ObjectSet, PlayerRef, Primitive};
use crate::types::ids::{DelayedTriggerId, IdMap, IdSet, ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::{
    DamageRecipient, DepartedFrame, EventIndex, EventKind, EventKindMask, IdentityRef, Multiplicity, PendingTrigger,
    Referred, RememberedObject, TriggerBinding, TriggerCondition, TriggerDef, TriggerEvent, TriggerLimit,
    TriggerOrigin, TriggerSeq, TriggerSubject,
};

use super::history::TurnOrdinals;
use crate::types::zones::{Zone, ZoneSet};

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

/// One object's ability list as a snapshot holds it, with where it was and
/// whose: an object that has left since answers from these (the audit's).
#[derive(Debug, Clone)]
pub struct ObjectSnapshot {
    pub(crate) object: ObjectRef,
    pub(crate) zone: Zone,
    pub(crate) owner: PlayerId,
    pub(crate) chars: Arc<EffectiveCharacteristics>,
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
/// whichever member of the event moves first (item 174).
#[derive(Debug, Clone)]
pub struct DepartureFrame {
    object: ObjectRef,
    /// `None` once the move has taken it.
    frame: Option<Arc<EffectiveCharacteristics>>,
}

/// CR 605.1b's three criteria, derived from the def and never a tag: no
/// target, triggers from mana being added, and could add mana. CR 605.5a is
/// the same sentence from the other side — a target or any other event
/// makes it an ordinary triggered ability.
pub fn is_mana_ability(def: &TriggerDef) -> bool {
    let arms = def.condition.events();
    !arms.is_empty()
        && arms.iter().all(|arm| matches!(arm, TriggerEvent::ManaAdded { .. }))
        && def.effect.instances().is_empty()
        && could_add_mana(&def.effect)
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
/// now, a departed object's CR 603.10a frame, or a survivor's list from
/// before a batch. An object with lists from before the event is several
/// candidates, one per list, side by side.
pub(super) struct TriggerCandidate<'a> {
    pub(super) id: ObjectId,
    /// CR 603.3a's "you": the player who controls the source now, or the
    /// frame's controller for a departed one.
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
    fn is(self, game: &GameState, id: ObjectId, seq: EventSeq) -> bool {
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
    /// A departed object's list from before it left. `snapshot: None` is the
    /// CR 603.10a frame its record carries, which answers the look-back arms
    /// of every record of the window; `Some(k)` is the list snapshot `k`
    /// holds, which answers those of the records its batch performed (the
    /// audit's).
    Departed { chars: &'a EffectiveCharacteristics, snapshot: Option<usize> },
    /// A surviving object's list from before a batch, off `LookBackSnapshot`
    /// number `snapshot`.
    Before { chars: &'a EffectiveCharacteristics, snapshot: usize },
}

impl TriggerCandidateFrame<'_> {
    fn chars(&self) -> &EffectiveCharacteristics {
        match self {
            TriggerCandidateFrame::Live { chars, .. } => chars,
            TriggerCandidateFrame::Departed { chars, .. } | TriggerCandidateFrame::Before { chars, .. } => chars,
        }
    }

    fn is_departed(&self) -> bool {
        matches!(self, TriggerCandidateFrame::Departed { .. })
    }

    /// The arms this list answers for a record that looks back through
    /// snapshot `looks_back_through`, or `None` when it answers none. A
    /// departed object has only its list from before; a survivor's list now
    /// answers everything unless the record's snapshot holds the one from
    /// before, which then answers the look-back arms in its place.
    fn asks(&self, looks_back_through: Option<usize>) -> Option<Asks> {
        match self {
            TriggerCandidateFrame::Live { snapshots, .. }
                if looks_back_through.is_some_and(|k| snapshots.contains(&k)) =>
            {
                Some(Asks::NotLookBack)
            }
            TriggerCandidateFrame::Live { .. } => Some(Asks::Every),
            TriggerCandidateFrame::Departed { snapshot: None, .. } => Some(Asks::LookBack),
            TriggerCandidateFrame::Departed { snapshot: Some(k), .. } | TriggerCandidateFrame::Before { snapshot: k, .. }
                if looks_back_through == Some(*k) =>
            {
                Some(Asks::LookBack)
            }
            TriggerCandidateFrame::Departed { .. } | TriggerCandidateFrame::Before { .. } => None,
        }
    }
}

/// One ability's match against one record, before "one or more" folds it.
struct ArmMatch<'r, 'a> {
    row: &'r TriggerCandidateDef<'a>,
    /// Which of the condition's events matched.
    arm: EventIndex,
    subjects: Vec<Option<ObjectId>>,
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

/// Which of a def's arms one record asks (`TriggerEvent::looks_back`).
#[derive(Clone, Copy)]
enum Asks {
    Every,
    LookBack,
    NotLookBack,
}

/// A match the dispatcher will queue or resolve.
pub(super) struct MatchedTrigger {
    pub(super) identity: AbilityIdentity,
    pub(super) controller: PlayerId,
    pub(super) def: Arc<TriggerDef>,
    pub(super) source_card: Arc<CardData>,
    pub(super) instances: Vec<EffectRecipient>,
    pub(super) event: EventIndex,
    pub(super) records: Vec<EventRecord>,
    pub(super) subject: Option<ObjectRef>,
    pub(super) mana: bool,
}

/// One registry entry's matches in a window (`delayed.rs`): every occurrence
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
        mark: EventSeq,
        batch: Option<BatchId>,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        let window: Vec<EventSeq> = self
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
    /// the dispatch may have to ask (`emit_event_with_provider`); a mana
    /// trigger's event (`ManaAdded`) is performed inside a batch, so the one
    /// path that resolves at dispatch never runs here.
    pub(crate) fn dispatch_unbatched(&mut self, seq: EventSeq, ctx: Option<&ActionContext>) {
        let audit = self.dispatch_audit.is_some().then(Vec::new);
        let _ = self.dispatch(&[seq], ctx, &[], audit.as_deref());
        self.flush_window_unless_nested();
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
        window: &[EventSeq],
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
        let ordinals = self.advance_history(window);
        self.nesting.dispatch_depth += 1;
        let result = self.dispatch_inner(window, ctx, snapshots, audit, &ordinals);
        self.nesting.dispatch_depth -= 1;
        result
    }

    /// One dispatch, in five steps.
    ///
    /// 1. **The gate** — five probes (§4.2): the permanents whose printed
    ///    triggers read a kind this window carries (§11's mask, source
    ///    first), the zone map, the unattributed zone set off the registry
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
        window: &[EventSeq],
        ctx: Option<&ActionContext>,
        snapshots: &[LookBackSnapshot],
        audit: Option<&[LookBackSnapshot]>,
        ordinals: &TurnOrdinals,
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
        let matches = self.detect(window, window_kinds, snapshots, ordinals);
        if let Some(audit) = audit {
            self.audit_dispatch(window, audit, &matches, ordinals);
        }
        // The registry's leg (§4.6), which has no shortcut for the audit to
        // check: every entry reading a kind of the window is asked.
        let delayed = self.detect_delayed(window, window_kinds, ordinals);
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
        window: &[EventSeq],
        window_kinds: EventKindMask,
        snapshots: &[LookBackSnapshot],
        ordinals: &TurnOrdinals,
    ) -> Vec<MatchedTrigger> {
        // --- The gate: five probes, and on the old pools nothing else -------
        if window_kinds.is_empty() {
            return Vec::new();
        }
        let unattributed = self.continuous_effects.summary().unattributed_trigger_zones;
        let readers = self.battlefield_readers(window_kinds);
        let named = self.named_trigger_carriers(window_kinds);
        let any_frame_source = window.iter().any(|seq| {
            self.events.record(*seq).is_some_and(|r| {
                frame_of(&r.event).is_some_and(|f| f.abilities.iter().any(is_triggered))
            })
        });
        // The battlefield probe and the named rows read the mask. The zone
        // map is keyed by ability, and a `Filter` row's zones and the departed
        // leg read lists no registration saw, so they keep their whole walk.
        if readers.is_empty()
            && named.is_empty()
            && self.zone_trigger_sources.is_empty()
            && unattributed.is_empty()
            && !any_frame_source
            && snapshots.is_empty()
        {
            return Vec::new();
        }
        self.find_matches(window, readers, named, unattributed, snapshots, ordinals)
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
        let mut queued: Vec<(TriggerSeq, TriggerOrigin, PlayerId, EventSeq)> = Vec::new();
        for m in matches {
            let caused_by = m.records[0].seq;
            // "Triggers only once each turn" is written as the ability
            // queues, so a second match in this same window finds it taken.
            if m.def.limit == Some(TriggerLimit::TriggersOnlyOnceEachTurn) && !self.triggered_this_turn.insert(m.identity) {
                continue;
            }
            let seq = TriggerSeq(self.next_trigger_seq);
            self.next_trigger_seq += 1;
            let binding = TriggerBinding {
                def: Arc::clone(&m.def),
                records: m.records,
                event: m.event,
                subject: m.subject,
                triggered_by: None,
                referred: Referred::default(),
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
                departed: Vec::new(),
                x_value: None,
            };
            if m.mana {
                match ctx {
                    Some(ctx) => self.resolve_triggered_mana(&pending, ctx)?,
                    // Unreachable: `ManaAdded` is always performed inside a
                    // batch. Loud in debug; queued in release, so the mana is
                    // late rather than lost.
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
            self.emit_event_unstamped(GameEvent::AbilityTriggered { seq, origin, controller, caused_by });
        }
        Ok(())
    }

    /// The permanents whose printed triggers, or the copy's they entered as,
    /// read a kind `window_kinds` carries, selected before they are ordered,
    /// in CR 613.7 order: sorted on
    /// the key `battlefield_ids_ordered` sorts on, so the candidate order —
    /// which the `OrderTriggers` prompt offers — is the whole-battlefield
    /// walk's.
    fn battlefield_readers(&self, window_kinds: EventKindMask) -> Vec<ObjectId> {
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
                ObjectSet::Host => named.extend(self.battlefield.get(&row.source).and_then(|e| e.attached_to)),
                ObjectSet::Filter { .. } => {}
            }
        }
        named
    }

    /// Whether performing these decided actions can change which triggered
    /// abilities a *surviving* object has: true when one of them takes off
    /// the battlefield a permanent that is the source of a copy effect, or of
    /// an effect that grants or removes abilities. A player losing counts for
    /// each such permanent they own, since CR 800.4a takes those with them.
    /// When it is true the batch snapshots the lists before performing
    /// (`LookBackSnapshot`).
    pub(crate) fn departs_an_ability_list_source(&self, decided: &[Option<GameAction>]) -> bool {
        let sources = &self.continuous_effects.summary().ability_list_sources;
        !sources.is_empty()
            && decided.iter().flatten().any(|action| match action {
                GameAction::ZoneChange { object, from: Zone::Battlefield, .. }
                | GameAction::Destroy { object, .. } => sources.contains(object),
                GameAction::PlayerLoses { player, .. } => sources
                    .iter()
                    .any(|s| self.objects.get(s).is_some_and(|o| o.owner == *player)),
                _ => false,
            })
    }

    /// CR 603.10a before the batch performs: frame each permanent `decided`
    /// takes off the battlefield, in batch order. A player leaving takes
    /// every permanent, since CR 800.4a's fourth clause decides what it exiles
    /// only after its first two have run. The frames are memo hits: nothing
    /// has changed since the batch began deciding.
    ///
    /// **And every other mover an entry names, from whatever zone** (CR
    /// 113.7a, 608.2h; `triggers-architecture.md` §6.1): a spell a trigger's
    /// subject is, a card a draw trigger named. The move hands the frame to
    /// each entry naming the mover (`hand_over_departed_frame`).
    pub(crate) fn capture_departure_frames(&mut self, decided: &[Option<GameAction>]) {
        let mut named: Option<Vec<ObjectRef>> = None;
        for action in decided.iter().flatten() {
            match action {
                GameAction::ZoneChange { object, from: Zone::Battlefield, .. }
                | GameAction::Destroy { object, .. } => self.capture_departure_frame(*object),
                GameAction::ZoneChange { object, .. } => {
                    let named = named.get_or_insert_with(|| self.objects_entries_name());
                    if self.object_ref(*object).is_some_and(|r| named.contains(&r)) {
                        self.capture_named_frame(*object);
                    }
                }
                GameAction::PlayerLoses { player, .. } => {
                    for id in self.battlefield_ids_ordered() {
                        self.capture_departure_frame(id);
                    }
                    let named = named.get_or_insert_with(|| self.objects_entries_name());
                    for object in named.iter() {
                        if self.objects.get(&object.id).is_some_and(|o| o.owner == *player) {
                            self.capture_named_frame(object.id);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// The objects a queued, stacked or resolving entry names: its source,
    /// and its trigger's subject; and each waiting delayed trigger's source.
    /// Empty or short on the common board.
    fn objects_entries_name(&self) -> Vec<ObjectRef> {
        let mut named: Vec<ObjectRef> = Vec::new();
        let mut name = |object: Option<ObjectRef>| {
            if let Some(object) = object
                && !named.contains(&object)
            {
                named.push(object);
            }
        };
        for pending in &self.pending_triggers {
            name(Some(pending.origin.source_ref()));
            name(pending.binding.subject);
        }
        for entry in self.stack_entries.values() {
            name(entry.ability_identity.map(|identity| identity.source));
            name(entry.trigger.as_ref().and_then(|binding| binding.subject));
        }
        if let Some(resolving) = &self.resolving {
            name(resolving.identity.map(|identity| identity.source));
            name(resolving.subject);
        }
        for delayed in &self.delayed_triggers {
            name(Some(delayed.source));
        }
        named
    }

    /// A named mover's frame, from any zone, unless its batch framed it.
    fn capture_named_frame(&mut self, id: ObjectId) {
        let Some(object) = self.object_ref(id) else { return };
        if self.departure_frames.iter().any(|d| d.object == object) {
            return;
        }
        if let Some(frame) = compute_characteristics(self, id) {
            self.departure_frames.push(DepartureFrame { object, frame: Some(frame) });
        }
    }

    /// CR 113.7a, 608.2h — `id` is leaving the zone an entry expected it in:
    /// each queued, stacked or resolving entry that names it, and each
    /// delayed trigger waiting with it as source, keeps the frame its batch
    /// took. That is `lki`, the record's own, for a permanent, and
    /// for any other mover the one `capture_departure_frames` took because an
    /// entry named it. One writer; its callers are the two performers that
    /// move an object out of a zone, before the move.
    pub(crate) fn hand_over_departed_frame(&mut self, id: ObjectId, lki: Option<&Arc<EffectiveCharacteristics>>) {
        let frame = match lki {
            Some(frame) => Arc::clone(frame),
            None => match self.take_named_frame(id) {
                Some(frame) => frame,
                None => return,
            },
        };
        let Some(object) = self.object_ref(id) else { return };
        let departed = || DepartedFrame { object, frame: Arc::clone(&frame) };
        for delayed in &mut self.delayed_triggers {
            if delayed.source == object {
                delayed.source_frame = Some(Arc::clone(&frame));
            }
        }
        for pending in &mut self.pending_triggers {
            if pending.origin.source_ref() == object || pending.binding.subject == Some(object) {
                pending.departed.push(departed());
            }
        }
        for entry in self.stack_entries.values_mut() {
            let source = entry.ability_identity.map(|identity| identity.source);
            let subject = entry.trigger.as_ref().and_then(|binding| binding.subject);
            if source == Some(object) || subject == Some(object) {
                entry.departed.push(departed());
            }
        }
        if let Some(resolving) = &mut self.resolving
            && (resolving.identity.map(|identity| identity.source) == Some(object) || resolving.subject == Some(object))
        {
            resolving.departed.push(departed());
        }
    }

    /// The frame a named mover's batch took for it, if any: the non-battlefield
    /// counterpart of `take_departure_frame`, which asserts every battlefield
    /// departure had one.
    fn take_named_frame(&mut self, id: ObjectId) -> Option<Arc<EffectiveCharacteristics>> {
        let object = self.object_ref(id)?;
        self.departure_frames.iter_mut().find(|d| d.object == object).and_then(|d| d.frame.take())
    }

    /// One permanent's frame. A permanent an enclosing batch already framed
    /// keeps that frame, since a destruction's move is a nested batch and
    /// the event it belongs to is the outer one.
    fn capture_departure_frame(&mut self, id: ObjectId) {
        if !self.battlefield.contains_key(&id) {
            return;
        }
        let Some(object) = self.object_ref(id) else { return };
        if self.departure_frames.iter().any(|d| d.object == object) {
            return;
        }
        if let Some(frame) = compute_characteristics(self, id) {
            self.departure_frames.push(DepartureFrame { object, frame: Some(frame) });
        }
    }

    /// The frame the move of `id` off the battlefield carries, taken before
    /// its batch performed.
    pub(crate) fn take_departure_frame(&mut self, id: ObjectId) -> Option<Arc<EffectiveCharacteristics>> {
        let object = self.object_ref(id)?;
        let taken = self
            .departure_frames
            .iter_mut()
            .find(|d| d.object == object)
            .and_then(|d| d.frame.take());
        // Every departure is decided by a batch, which framed it first.
        debug_assert!(taken.is_some(), "{id} left the battlefield with no frame from its batch");
        taken.or_else(|| crate::engine::layers::compute::compute_characteristics_uncached(self, id).map(Arc::new))
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
        self.candidates_now(self.battlefield_readers(EventKindMask::ALL), named, unattributed)
            .into_iter()
            .filter_map(|id| self.object_snapshot(id))
            .collect()
    }

    /// `id`'s list now, and where it is and whose.
    pub(crate) fn object_snapshot(&self, id: ObjectId) -> Option<ObjectSnapshot> {
        let object = self.objects.get(&id)?;
        Some(ObjectSnapshot {
            object: ObjectRef { id, zone_change_epoch: object.zone_change_epoch },
            zone: object.zone,
            owner: object.owner,
            chars: compute_characteristics(self, id)?,
        })
    }

    /// The objects a dispatch asks with their ability lists as they are now,
    /// as against the lists from before an event (a departed object's CR
    /// 603.10a frame, a survivor's `LookBackSnapshot`), in CR 613.7 order.
    ///
    /// Every zone, by three routes: on the battlefield, `readers` (printed
    /// triggers) and the `named` objects there, or every permanent while a
    /// `Filter` row grants or copies a triggered ability onto the
    /// battlefield; elsewhere, every object whose printed triggered ability
    /// works in the zone it is in (CR 113.6k: a graveyard's, a hand's, exile's
    /// — a record's own subject in a graveyard is one), the `named` objects
    /// there, and every object in the zones a `Filter` row reaches.
    fn candidates_now(&self, readers: Vec<ObjectId>, named: Vec<ObjectId>, unattributed: ZoneSet) -> Vec<ObjectId> {
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
        let mut elsewhere: Vec<(Timestamp, ObjectId)> = self
            .zone_trigger_sources
            .keys()
            .map(|&id| (self.object_timestamp(id), id))
            .collect();
        let mut seen: IdSet<ObjectId> = elsewhere.iter().map(|&(_, id)| id).collect();
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
        window: &[EventSeq],
        readers: Vec<ObjectId>,
        named: Vec<ObjectId>,
        unattributed: ZoneSet,
        snapshots: &[LookBackSnapshot],
        ordinals: &TurnOrdinals,
    ) -> Vec<MatchedTrigger> {
        let mut live = self.candidates_now(readers, named, unattributed);

        // Each survivor's lists from before the window's snapshotting batches.
        // A survivor no live set reaches now, like a grant's carrier after
        // the granter left, is still asked, after the rest.
        let mut before: IdMap<ObjectId, Vec<(usize, &EffectiveCharacteristics)>> = IdMap::default();
        for (k, snapshot) in snapshots.iter().enumerate() {
            for frame in &snapshot.frames {
                if self.object_ref(frame.object.id) == Some(frame.object) {
                    before.entry(frame.object.id).or_default().push((k, frame.chars.as_ref()));
                }
            }
        }
        if !before.is_empty() {
            let mut reached: IdSet<ObjectId> = live.iter().copied().collect();
            for snapshot in snapshots {
                for frame in &snapshot.frames {
                    if before.contains_key(&frame.object.id) && reached.insert(frame.object.id) {
                        live.push(frame.object.id);
                    }
                }
            }
        }

        let records: Vec<(EventSeq, &EventRecord)> = window
            .iter()
            .filter_map(|seq| self.events.record(*seq).map(|r| (*seq, r)))
            .collect();

        // Leg 2: the frames the window carries (CR 603.10a) — each departed
        // object's list as it was, asked look-back conditions only.
        let frames: Vec<(ObjectId, PlayerId, Zone, &EffectiveCharacteristics)> = records
            .iter()
            .filter_map(|(_, r)| match &r.event {
                GameEvent::ZoneChange { object_id, owner, from, lki: Some(frame), .. }
                | GameEvent::LeftTheGame { object_id, owner, from, lki: Some(frame) } => {
                    Some((*object_id, *owner, *from, frame.as_ref()))
                }
                _ => None,
            })
            .collect();

        let mut candidates: Vec<TriggerCandidate<'_>> = Vec::new();
        for id in live {
            let Some(object) = self.objects.get(&id) else { continue };
            let Some(chars) = compute_characteristics(self, id) else { continue };
            let earlier = before.remove(&id).unwrap_or_default();
            let controller = controller_or_owner(self, id).unwrap_or(object.owner);
            let host = self.battlefield.get(&id).and_then(|e| e.attached_to);
            let snapshots = earlier.iter().map(|&(k, _)| k).collect();
            // The lists from before sit right behind the list now, so an
            // object's triggers keep the order a single candidate gave them.
            let before_lists = earlier
                .into_iter()
                .map(|(snapshot, chars)| TriggerCandidateFrame::Before { chars, snapshot });
            let lists = std::iter::once(TriggerCandidateFrame::Live { chars, snapshots }).chain(before_lists);
            for frame in lists {
                candidates.push(TriggerCandidate {
                    id,
                    controller,
                    owner: object.owner,
                    zone: object.zone,
                    host,
                    frame,
                    card: Arc::clone(&object.card_data),
                });
            }
        }
        for (id, owner, from, frame) in frames {
            // A departed object's card: still in the store for a zone change,
            // gone for a `LeftTheGame` — then the frame's own list is all
            // there is, and the stack object is built from a card that names
            // what the frame does.
            let card = match self.objects.get(&id) {
                Some(object) => Arc::clone(&object.card_data),
                None => Arc::new(frame_card(frame)),
            };
            candidates.push(TriggerCandidate {
                id,
                controller: frame.controller,
                owner,
                zone: from,
                host: None,
                frame: TriggerCandidateFrame::Departed { chars: frame, snapshot: None },
                card,
            });
        }

        let matches = self.match_candidates(&records, &candidates, snapshots, ordinals);
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
        records: &[(EventSeq, &EventRecord)],
        candidates: &[TriggerCandidate<'_>],
        snapshots: &[LookBackSnapshot],
        ordinals: &TurnOrdinals,
    ) -> Vec<MatchedTrigger> {
        let mut matches: Vec<MatchedTrigger> = Vec::new();
        // "One or more" accumulates across the window: (identity, event) -> index into `matches`.
        let mut once: Vec<((AbilityIdentity, EventIndex), usize)> = Vec::new();

        // One row per triggered ability, so the records loop below pays
        // only for what depends on the record (§4.2).
        let mut defs: Vec<TriggerCandidateDef<'_>> = Vec::new();
        for (index, candidate) in candidates.iter().enumerate() {
            let chars = candidate.frame.chars();
            let epoch = self.objects.get(&candidate.id).map(|o| o.zone_change_epoch).unwrap_or(0);
            for ability in chars.abilities.iter() {
                let Effect::Triggered(def) = &ability.effect else { continue };
                // CR 113.6 is asked of each ability: the object is here
                // because *some* ability of its functions here (§4.2).
                if !functions_in(ability, &chars.types, candidate.zone) {
                    continue;
                }
                defs.push(TriggerCandidateDef {
                    candidate: index,
                    identity: AbilityIdentity {
                        source: ObjectRef { id: candidate.id, zone_change_epoch: epoch },
                        ability: ability.id,
                    },
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
                let Some(asks) = candidate.frame.asks(looks_back_through) else { continue };
                let identity = row.identity;
                let def = row.def;
                let outcome = self.match_def(def, candidate, asks, *seq, &record.event, identity, ordinals);
                let (matched, subjects, refusal) = match outcome {
                    Ok((event, subjects)) => (Some(event), subjects, None),
                    Err(refusal) => (None, Vec::new(), Some(refusal)),
                };
                let mana = matched.is_some() && is_mana_ability(def);
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
                let this = ArmMatch { row, arm, subjects };
                match hits.iter_mut().find(|hit| hit.row.identity == identity) {
                    Some(earlier) if this.arm < earlier.arm => *earlier = this,
                    Some(_) => {}
                    None => hits.push(this),
                }
            }
            for ArmMatch { row, arm: matched, subjects } in hits {
                let candidate = &candidates[row.candidate];
                let (identity, def) = (row.identity, row.def);
                let mana = is_mana_ability(def);
                let arm = &def.condition.events()[matched.0];
                // The ability's own def, shared: nothing writes it once the card
                // is built, so a Humility landing before placement still cannot
                // un-trigger it (CR 113.7a).
                let def_arc: Arc<TriggerDef> = Arc::clone(def);
                match arm.multiplicity() {
                    Multiplicity::PerOccurrence => {
                        for subject in subjects {
                            matches.push(MatchedTrigger {
                                identity,
                                controller: candidate.controller,
                                def: Arc::clone(&def_arc),
                                source_card: Arc::clone(&candidate.card),
                                instances: row.instances.to_vec(),
                                event: matched,
                                records: vec![(*record).clone()],
                                subject: subject.and_then(|id| self.object_ref(id)),
                                mana,
                            });
                        }
                    }
                    // CR 603.2c's boundary is the window: one trigger, every
                    // matching record in its binding, no one object.
                    Multiplicity::OncePerEvent => {
                        match once.iter().find(|((i, e), _)| *i == identity && *e == matched) {
                            Some((_, at)) => matches[*at].records.push((*record).clone()),
                            None => {
                                once.push(((identity, matched), matches.len()));
                                matches.push(MatchedTrigger {
                                    identity,
                                    controller: candidate.controller,
                                    def: Arc::clone(&def_arc),
                                    source_card: Arc::clone(&candidate.card),
                                    instances: row.instances.to_vec(),
                                    event: matched,
                                    records: vec![(*record).clone()],
                                    subject: None,
                                    mana,
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
    /// subjects of its occurrences (one per trigger). `Err` names the
    /// predicate that refused.
    fn match_def(
        &self,
        def: &TriggerDef,
        candidate: &TriggerCandidate<'_>,
        asks: Asks,
        seq: EventSeq,
        event: &GameEvent,
        identity: AbilityIdentity,
        ordinals: &TurnOrdinals,
    ) -> Result<(EventIndex, Vec<Option<ObjectId>>), Refusal> {
        if matches!(def.condition, TriggerCondition::State(_)) {
            return Err(Refusal::StateTrigger);
        }
        // CR 603.2f, per candidate, of the object as the event left it. A frame
        // candidate was a permanent, which is visible.
        if !candidate.frame.is_departed() && !visible_to_all(self, candidate.id) {
            return Err(Refusal::Visibility);
        }
        let referents = TriggerReferents {
            this: ThisObject::Candidate(candidate.id),
            controller: candidate.controller,
            owner: candidate.owner,
            host: candidate.host,
            this_ability: Some(identity),
            referred: &[],
        };
        let mut matched: Option<(EventIndex, Vec<Option<ObjectId>>)> = None;
        for (index, arm) in def.condition.events().iter().enumerate() {
            match asks {
                Asks::LookBack if !arm.looks_back() => continue,
                Asks::NotLookBack if arm.looks_back() => continue,
                _ => {}
            }
            if !arm.reads(event) {
                continue;
            }
            // CR 113.6k asks each trigger condition where it functions; the
            // pre-pass asked the ability, which is their union.
            if !condition_functions_in(arm, &candidate.frame.chars().types, candidate.zone) {
                continue;
            }
            let subjects = self.occurrences_matching_arm(arm, &referents, seq, event);
            if !subjects.is_empty() {
                matched = Some((EventIndex(index), subjects));
                break;
            }
        }
        let (event_index, subjects) = matched.ok_or(Refusal::TriggerCondition)?;
        if !self.within_once_per_turn_limit(def, identity, candidate.controller, seq, ordinals) {
            return Err(Refusal::Limit);
        }
        // CR 603.4 at the trigger. "You" is the candidate's controller (CR
        // 109.5): the source's now, or its frame's for a look-back candidate.
        if let Some(condition) = &def.intervening_if
            && !settled_holds(condition, self, candidate.id, Some(candidate.controller))
        {
            return Err(Refusal::InterveningIf);
        }
        Ok((event_index, subjects))
    }

    /// The once-per-turn limits (§3.5), each read at the trigger. CR
    /// 603.2h's gate is its source's controller's; "only once each turn" is
    /// the ability's; "the first time" is this record's place in its turn.
    pub(super) fn within_once_per_turn_limit(
        &self,
        def: &TriggerDef,
        identity: AbilityIdentity,
        controller: PlayerId,
        seq: EventSeq,
        ordinals: &TurnOrdinals,
    ) -> bool {
        match def.limit {
            None => true,
            Some(TriggerLimit::DoThisOnlyOnceEachTurn) => !self.action_taken_this_turn.contains(&(identity, controller)),
            Some(TriggerLimit::TriggersOnlyOnceEachTurn) => !self.triggered_this_turn.contains(&identity),
            Some(TriggerLimit::FirstTimeEachTurn) => ordinals.place_in_turn(seq) == Some(1),
        }
    }

    /// Each occurrence of `arm`'s event that the record `event` holds, as that
    /// occurrence's subject ("that object", if any), read for the trigger's
    /// `referents`: each is one trigger (CR 603.2c). A record holds one
    /// occurrence for every arm but `Attacks`, which holds one per matching
    /// attacker. Empty when the record does not match the arm.
    ///
    /// One arm against one record, and every case has the same shape: the
    /// `match` pairs the arm with the record kind it reads, and the arm's own
    /// fields are the predicates over that record. **A field left `None` is
    /// not asked** — every one of them is an `is_none_or` — which is how "any"
    /// is spelled at the type, the convention the replacement patterns share.
    /// `one` wraps a boolean as an occurrence: true is the single subject the
    /// arm's `subject_of` projection names, false is no occurrence at all. The
    /// only arm that does not go through it is `Attacks`, where CR 508.3a
    /// makes each matching attacker an occurrence of its own.
    pub(super) fn occurrences_matching_arm(
        &self,
        arm: &TriggerEvent,
        referents: &TriggerReferents<'_>,
        seq: EventSeq,
        event: &GameEvent,
    ) -> Vec<Option<ObjectId>> {
        let one = |ok: bool| if ok { vec![arm.subject_of(event)] } else { Vec::new() };
        match (arm, event) {
            (
                TriggerEvent::ZoneChange { subject, from, to, cause, owner, .. },
                GameEvent::ZoneChange { object_id, owner: moved_owner, from: rf, to: rt, cause: rc, lki },
            ) => one(
                from.is_none_or(|z| z == *rf)
                    && to.is_none_or(|z| z == *rt)
                    && cause.is_none_or(|c| c == *rc)
                    && owner.as_ref().is_none_or(|p| self.player_ref_is(p, *moved_owner, referents))
                    && self.subject_matches(subject, Some(*object_id), referents, seq, lki.as_deref()),
            ),
            // CR 603.6c names this event: a leaves-the-battlefield ability
            // triggers on it and nothing narrower — no `to`, no cause.
            (
                TriggerEvent::ZoneChange { subject, from, to, cause, owner, .. },
                GameEvent::LeftTheGame { object_id, owner: moved_owner, from: rf, lki },
            ) => one(
                from.is_none_or(|z| z == *rf)
                    && to.is_none()
                    && cause.is_none()
                    && owner.as_ref().is_none_or(|p| self.player_ref_is(p, *moved_owner, referents))
                    && self.subject_matches(subject, Some(*object_id), referents, seq, lki.as_deref()),
            ),
            (TriggerEvent::BecomesTapped { subject }, GameEvent::Tapped { object_id })
            | (TriggerEvent::BecomesUntapped { subject }, GameEvent::Untapped { object_id }) => {
                one(self.subject_matches(subject, Some(*object_id), referents, seq, None))
            }
            (TriggerEvent::DrawsCard { player: who, .. }, GameEvent::CardDrawn { player_id, .. })
            | (TriggerEvent::ShufflesLibrary { player: who }, GameEvent::LibraryShuffled { player_id }) => {
                one(who.as_ref().is_none_or(|p| self.player_ref_is(p, *player_id, referents)))
            }
            (
                TriggerEvent::ManaAdded { source, tapped_for_mana, mana },
                GameEvent::ManaAdded { source_id, mana: added, tapped_for_mana: tapped, .. },
            ) => one(
                tapped_for_mana.is_none_or(|t| t == *tapped)
                    && mana.is_none_or(|m| added.iter().any(|(t, n)| *t == m && *n > 0))
                    && self.subject_matches(source, Some(*source_id), referents, seq, None),
            ),
            (
                TriggerEvent::DamageDealt { source, recipient, combat, .. },
                GameEvent::DamageDealt { source_id, source_frame, target, is_combat, .. },
            ) => {
                let to = match (recipient, target) {
                    (DamageRecipient::Any, _) => true,
                    (DamageRecipient::Player(who), DamageTarget::Player(pid)) => {
                        who.as_ref().is_none_or(|p| self.player_ref_is(p, *pid, referents))
                    }
                    (DamageRecipient::Object(filter), DamageTarget::Object(id)) => match filter {
                        None => true,
                        Some(filter) => self.subject_matches(
                            &TriggerSubject::Filter(filter.clone()),
                            Some(*id),
                            referents,
                            seq,
                            None,
                        ),
                    },
                    (DamageRecipient::Player(_), DamageTarget::Object(_))
                    | (DamageRecipient::Object(_), DamageTarget::Player(_)) => false,
                };
                one(
                    to && combat.is_none_or(|c| c == *is_combat)
                        && self.subject_matches(source, Some(*source_id), referents, seq, source_frame.as_deref()),
                )
            }
            (TriggerEvent::PhaseBegins { phase, whose }, GameEvent::PhaseBegin { phase: rp, player }) => {
                one(phase == rp && whose.as_ref().is_none_or(|p| self.player_ref_is(p, *player, referents)))
            }
            (TriggerEvent::StepBegins { step, whose }, GameEvent::StepBegin { step: rs, player }) => {
                one(step == rs && whose.as_ref().is_none_or(|p| self.player_ref_is(p, *player, referents)))
            }
            (TriggerEvent::TurnBegins { whose }, GameEvent::TurnBegin { player, .. }) => {
                one(whose.as_ref().is_none_or(|p| self.player_ref_is(p, *player, referents)))
            }
            // The sign is the split. A 0 gain never reaches the log (CR
            // 119.10, `replacement::never_happens`).
            (TriggerEvent::GainsLife { player: who, .. }, GameEvent::LifeChanged { player_id, old, new, .. }) => {
                one(new > old && who.as_ref().is_none_or(|p| self.player_ref_is(p, *player_id, referents)))
            }
            (TriggerEvent::LosesLife { player: who, .. }, GameEvent::LifeChanged { player_id, old, new, .. }) => {
                one(new < old && who.as_ref().is_none_or(|p| self.player_ref_is(p, *player_id, referents)))
            }
            (TriggerEvent::CastsSpell { caster, spell }, GameEvent::SpellCast { spell_id, caster: who }) => {
                let spell_ok = match spell {
                    None => true,
                    Some(filter) => self.subject_matches(
                        &TriggerSubject::Filter(filter.clone()),
                        Some(*spell_id),
                        referents,
                        seq,
                        None,
                    ),
                };
                one(spell_ok && caster.as_ref().is_none_or(|p| self.player_ref_is(p, *who, referents)))
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
                one(
                    from_ok
                        && cast_ok
                        && controller.as_ref().is_none_or(|p| self.player_ref_is(p, *rc, referents))
                        && self.subject_matches(subject, Some(*object_id), referents, seq, None),
                )
            }
            (TriggerEvent::Attacks { attacker, .. }, GameEvent::AttackersDeclared { attackers }) => attackers
                .iter()
                .filter(|id| self.subject_matches(attacker, Some(**id), referents, seq, None))
                .map(|id| Some(*id))
                .collect(),
            (TriggerEvent::AbilityTriggers { caused_by, source }, GameEvent::AbilityTriggered { origin, caused_by: cause, .. }) => {
                let source_ok = match source {
                    None => true,
                    Some(filter) => self.subject_matches(
                        &TriggerSubject::Filter(filter.clone()),
                        Some(origin.source()),
                        referents,
                        seq,
                        None,
                    ),
                };
                let cause_ok = match caused_by {
                    None => true,
                    Some(inner) => self.events.record(*cause).is_some_and(|r| inner.reads(&r.event)),
                };
                one(source_ok && cause_ok)
            }
            // The same ability of the same object, whichever instance (§6.5).
            (
                TriggerEvent::AbilityResolves { identity: IdentityRef::ThisAbility },
                GameEvent::AbilityResolved { identity: resolved, .. },
            ) => one(referents.this_ability.is_some_and(|this| {
                this.source == resolved.source && this.ability.definition() == resolved.ability.definition()
            })),
            _ => Vec::new(),
        }
    }

    /// "Which object" (§3.3): `This` is the source itself, `Host` what it is
    /// attached to, a filter is read against the subject — off the record's
    /// frame for a look-back arm, since the object has left the zone the
    /// frame describes, and off the live board otherwise. `seq` is the
    /// record's, which a remembered source's own departure is.
    fn subject_matches(
        &self,
        subject: &TriggerSubject,
        id: Option<ObjectId>,
        referents: &TriggerReferents<'_>,
        seq: EventSeq,
        frame: Option<&EffectiveCharacteristics>,
    ) -> bool {
        match (subject, id) {
            (TriggerSubject::Any, _) => true,
            (TriggerSubject::ThisObject, Some(id)) => referents.this.is(self, id, seq),
            (TriggerSubject::Host, Some(id)) => referents.host == Some(id),
            (TriggerSubject::Referred, Some(id)) => referents.referred.iter().any(|r| r.is(self, id, seq)),
            (TriggerSubject::Filter(filter), Some(id)) => self
                .object_matches_filter_of_source(id, filter, referents.controller, referents.this.id(), frame)
                .unwrap_or(false),
            (
                TriggerSubject::ThisObject | TriggerSubject::Host | TriggerSubject::Filter(_) | TriggerSubject::Referred,
                None,
            ) => false,
        }
    }

    /// "Whose" — a `PlayerRef` against a record's player, read for the
    /// trigger's referents (CR 109.5's "you" is its controller).
    fn player_ref_is(&self, who: &PlayerRef, player: PlayerId, referents: &TriggerReferents<'_>) -> bool {
        match who {
            PlayerRef::You => player == referents.controller,
            PlayerRef::Opponent => player != referents.controller,
            PlayerRef::Owner => player == referents.owner,
            PlayerRef::Player(pid) => player == *pid,
        }
    }

    /// The zone the permanent at `id` entered from, off its own zone change
    /// earlier in the same window — `None` for a token, which came from
    /// nowhere (CR 111.2). The entry's performer announces both, so the
    /// window holds the move whenever it holds the entry.
    fn entry_origin(&self, id: ObjectId, entered_at: EventSeq) -> Option<Zone> {
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
                let player = match recipient {
                    EffectRecipient::Host => self
                        .battlefield
                        .get(&source)
                        .and_then(|e| e.attached_to)
                        .and_then(|host| controller_or_owner(self, host))
                        .unwrap_or(pending.controller),
                    _ => pending.controller,
                };
                let mut resolution = ResolutionContext::untargeted(source, player);
                resolution.ability_source = self.object_ref(source);
                resolution.trigger = Some(pending.binding.clone());
                let mut mana = Vec::with_capacity(output.mana.len());
                for (mana_type, amount) in &output.mana {
                    mana.push((*mana_type, self.evaluate_amount(amount, &resolution)?));
                }
                self.execute_action(
                    GameAction::ProduceMana {
                        player,
                        source,
                        mana,
                        special: output.special.clone(),
                        tapped_for_mana: false,
                    },
                    &ActionContext::resolving(ctx.dp, &resolution),
                )
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

/// The CR 603.10a frame a record carries, if it carries one.
fn frame_of(event: &GameEvent) -> Option<&EffectiveCharacteristics> {
    match event {
        GameEvent::ZoneChange { lki, .. } | GameEvent::LeftTheGame { lki, .. } => lki.as_deref(),
        _ => None,
    }
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

