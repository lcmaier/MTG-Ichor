//! The dispatch audit — a slow matcher beside the fast one
//! (`triggers-architecture.md` §4.10, TR-1b).
//!
//! The dispatcher answers "which triggered abilities does this window
//! trigger?" through a gate, five candidate sets, CR 113.6 per def and a
//! choice of list per condition. Every wrong answer it has had lived in that
//! selection, and none of them fails a random game: the fuzzer's invariants
//! are crashes, CR 117.5 and determinism. In a game that turns this on, every
//! dispatch is answered again the obvious way, and the two answers must agree.
//!
//! **The reference.** For each record of the window, every object in every
//! zone is asked with two lists: the one it had immediately before the
//! record's event, captured for every object at every batch's seam, and the
//! one it has now. A look-back condition reads the first and every other
//! condition the second (CR 603.10). An object that has changed zones since
//! the seam has only the first, and one that exists only since has only the
//! second. CR 113.6 is asked per def in the zone each list was in, and the
//! condition through the dispatcher's own `match_def` — what the audit shares
//! with the dispatcher, and so cannot check.
//!
//! **Its reads are invisible**, the memo audit's rule (`compute.rs`,
//! `audit_memo_hit`): the layer memo, the diagnostics and the trace are put
//! back as they were, so an audited game's counters are an unaudited one's.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use super::dispatch::{Asks, MatchedTrigger, TriggerCandidate, TriggerCandidateFrame};
use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::zone_function::functions_in;
use crate::events::event::{BatchId, EventSeq, GameEvent};
use crate::objects::card_data::CardData;
use crate::oracle::characteristics::controller_or_owner;
use crate::state::game_state::{AbilityIdentity, GameState};
use crate::types::effects::Effect;
use crate::types::ids::{AbilityId, IdMap, IdSet, ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::{Multiplicity, TriggerDef};
use crate::types::zones::Zone;

/// The audit's state, on a `GameState` that turned it on.
#[derive(Debug, Clone, Default)]
pub struct DispatchAudit {
    /// The open windows' seam captures, each filed once its batch performed.
    captures: Vec<AuditCapture>,
    /// Seams taken so far: an outer batch's seam comes before a nested
    /// one's, which is how the outermost capture covering a record is told.
    seams: u64,
    /// Dispatches answered twice.
    dispatches: u64,
    /// Triggers the two answers agreed on.
    triggers: u64,
}

/// Every object in every zone as one batch's seam found it, and the records
/// that batch performed.
#[derive(Debug, Clone)]
pub struct AuditCapture {
    window: Option<BatchId>,
    seam: u64,
    /// Event-log indices.
    performed: Range<usize>,
    existences: Vec<Existence>,
}

/// A seam's capture, held until its batch has performed.
pub(crate) struct AuditSeam {
    seam: u64,
    existences: Vec<Existence>,
}

/// One object as it was at one instant, with the list it had then.
#[derive(Debug, Clone)]
struct Existence {
    object: ObjectRef,
    zone: Zone,
    owner: PlayerId,
    controller: PlayerId,
    /// CR 303.4m, for `TriggerSubject::Host`.
    host: Option<ObjectId>,
    chars: Arc<EffectiveCharacteristics>,
    card: Arc<CardData>,
}

/// One trigger as the two answers are compared.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AuditKey {
    identity: AbilityIdentity,
    /// The arm of the condition that matched (`EventIndex`).
    event: usize,
    records: Vec<EventSeq>,
    subject: Option<ObjectRef>,
    controller: PlayerId,
}

impl AuditKey {
    fn of(matched: &MatchedTrigger) -> Self {
        let mut records = matched.records.clone();
        records.sort();
        AuditKey {
            identity: matched.identity,
            event: matched.event.0,
            records,
            subject: matched.object,
            controller: matched.controller,
        }
    }
}

impl GameState {
    /// Audit every dispatch for the rest of this game (`fuzz_games --audit`).
    pub fn enable_dispatch_audit(&mut self) {
        self.dispatch_audit.get_or_insert_with(Default::default);
    }

    /// Dispatches answered twice and triggers the two answers agreed on;
    /// `None` in a game that never turned the audit on.
    pub fn dispatch_audit_counts(&self) -> Option<(u64, u64)> {
        self.dispatch_audit.as_ref().map(|audit| (audit.dispatches, audit.triggers))
    }

    /// `read`, with nothing it did observable afterwards: the memo, the
    /// diagnostics and the trace are as they were before it.
    fn unobserved<R>(&mut self, read: impl FnOnce(&GameState) -> R) -> R {
        let memo = self.layer_memo.clone();
        let diagnostics = self.diagnostics.clone();
        let trace = self.trace.take();
        let result = read(self);
        self.trace = trace;
        self.diagnostics = diagnostics;
        self.layer_memo = memo;
        result
    }

    /// The audit's capture at a batch's seam, between deciding and
    /// performing; `None` when the audit is off.
    pub(crate) fn audit_seam(&mut self) -> Option<AuditSeam> {
        let audit = self.dispatch_audit.as_mut()?;
        audit.seams += 1;
        let seam = audit.seams;
        let existences = self.unobserved(|game| game.existences());
        Some(AuditSeam { seam, existences })
    }

    /// File a seam's capture now that its batch has performed `performed`.
    pub(crate) fn audit_performed(&mut self, seam: AuditSeam, performed: Range<usize>) {
        let window = self.events.current_stamp().batch;
        if let Some(audit) = self.dispatch_audit.as_mut() {
            audit.captures.push(AuditCapture { window, seam: seam.seam, performed, existences: seam.existences });
        }
    }

    /// The captures of `window`'s batches, taken by its dispatch.
    pub(super) fn take_audit_captures(&mut self, window: Option<BatchId>) -> Vec<AuditCapture> {
        let Some(audit) = self.dispatch_audit.as_mut() else { return Vec::new() };
        let (mine, others) = std::mem::take(&mut audit.captures).into_iter().partition(|c| c.window == window);
        audit.captures = others;
        mine
    }

    /// Answer `window` again, the reference's way, and panic unless the
    /// answer is the dispatcher's.
    pub(super) fn audit_dispatch(&mut self, window: &[EventSeq], captures: &[AuditCapture], engine: &[MatchedTrigger]) {
        let engine: Vec<AuditKey> = engine.iter().map(AuditKey::of).collect();
        let reference = self.unobserved(|game| game.reference_matches(window, captures));
        assert_agree(self, window, &engine, &reference);
        if let Some(audit) = self.dispatch_audit.as_mut() {
            audit.dispatches += 1;
            audit.triggers += engine.len() as u64;
        }
    }

    /// Every object in every zone, with its list now. An ability on the
    /// stack is an object (CR 113.1c) but not one with its source's
    /// abilities, so it is not one of these.
    fn existences(&self) -> Vec<Existence> {
        const ZONES: [Zone; 7] = [
            Zone::Battlefield,
            Zone::Stack,
            Zone::Exile,
            Zone::Command,
            Zone::Graveyard,
            Zone::Hand,
            Zone::Library,
        ];
        let mut all = Vec::new();
        for zone in ZONES {
            for id in self.zone_ids_ordered(zone) {
                if zone == Zone::Stack && self.stack_entries.get(&id).is_some_and(|entry| !entry.is_spell) {
                    continue;
                }
                let Some(object) = self.objects.get(&id) else { continue };
                let Some(chars) = compute_characteristics(self, id) else { continue };
                all.push(Existence {
                    object: ObjectRef { id, zone_change_epoch: object.zone_change_epoch },
                    zone: object.zone,
                    owner: object.owner,
                    controller: controller_or_owner(self, id).unwrap_or(object.owner),
                    host: self.battlefield.get(&id).and_then(|entry| entry.attached_to),
                    chars,
                    card: Arc::clone(&object.card_data),
                });
            }
        }
        all
    }

    /// The reference's answer for `window`.
    fn reference_matches(&self, window: &[EventSeq], captures: &[AuditCapture]) -> Vec<AuditKey> {
        let now = self.existences();
        let now_at: IdMap<ObjectId, usize> = now.iter().enumerate().map(|(k, e)| (e.object.id, k)).collect();
        let mut keys: Vec<AuditKey> = Vec::new();
        // "One or more" across the window: (identity, arm) -> index into `keys`.
        let mut once: HashMap<(AbilityIdentity, usize), usize> = HashMap::new();
        for &seq in window {
            let Some(record) = self.events.record(seq) else { continue };
            // Immediately before this record's event: the seam of the
            // outermost batch that performed it, the earliest seam covering
            // it. An unbatched record has none, and nothing changed a list
            // between its emission and now.
            let before = captures
                .iter()
                .filter(|c| c.performed.contains(&seq.0))
                .min_by_key(|c| c.seam)
                .map(|c| c.existences.as_slice());
            for (then, current) in pair(before, &now, &now_at) {
                self.reference_ask(seq, &record.event, then, current, &mut keys, &mut once);
            }
        }
        keys
    }

    /// One object against one record. Each of its triggered abilities is
    /// asked once: its look-back conditions off the list from before, the
    /// rest off the list now, and the first arm in the def's order that
    /// matches is the trigger — an ability triggers once per event (CR
    /// 603.2c) whichever of its conditions saw it.
    fn reference_ask(
        &self,
        seq: EventSeq,
        event: &GameEvent,
        then: Option<&Existence>,
        now: Option<&Existence>,
        keys: &mut Vec<AuditKey>,
        once: &mut HashMap<(AbilityIdentity, usize), usize>,
    ) {
        let Some(id) = then.or(now).map(|e| e.object.id) else { return };
        let departed = now.is_none();
        let then_defs = then.map(triggered).unwrap_or_default();
        let now_defs = now.map(triggered).unwrap_or_default();
        let mut abilities: Vec<(AbilityId, usize)> = then_defs.iter().map(|&(key, _)| key).collect();
        for &(key, _) in &now_defs {
            if !abilities.contains(&key) {
                abilities.push(key);
            }
        }
        // Item 168's convention, which the dispatcher follows: the identity
        // carries the epoch the object has now, or 0 once it has left the game.
        let epoch = self.objects.get(&id).map(|o| o.zone_change_epoch).unwrap_or(0);
        for key in abilities {
            let looked_back = self.reference_ask_list(key, &then_defs, then, Asks::LookBack, departed, seq, event);
            let current = self.reference_ask_list(key, &now_defs, now, Asks::NotLookBack, false, seq, event);
            let chosen = match (looked_back, current) {
                (Some(a), Some(b)) => Some(if b.0 < a.0 { b } else { a }),
                (a, b) => a.or(b),
            };
            let Some((index, subjects, def, controller)) = chosen else { continue };
            let identity = AbilityIdentity { source: ObjectRef { id, zone_change_epoch: epoch }, ability: key.0 };
            match def.condition.events()[index].multiplicity() {
                Multiplicity::PerOccurrence => {
                    for subject in subjects {
                        keys.push(AuditKey {
                            identity,
                            event: index,
                            records: vec![seq],
                            subject: subject.and_then(|s| self.object_ref(s)),
                            controller,
                        });
                    }
                }
                Multiplicity::OncePerEvent => match once.get(&(identity, index)) {
                    Some(&at) => keys[at].records.push(seq),
                    None => {
                        once.insert((identity, index), keys.len());
                        keys.push(AuditKey { identity, event: index, records: vec![seq], subject: None, controller });
                    }
                },
            }
        }
    }
}

impl GameState {
    /// One list's instance of ability `key`, asked `asks` of one record: the
    /// arm that matched, its subjects, the def and the list's controller.
    fn reference_ask_list<'a>(
        &self,
        key: (AbilityId, usize),
        defs: &[((AbilityId, usize), &'a TriggerDef)],
        existence: Option<&Existence>,
        asks: Asks,
        departed: bool,
        seq: EventSeq,
        event: &GameEvent,
    ) -> Option<(usize, Vec<Option<ObjectId>>, &'a TriggerDef, PlayerId)> {
        let existence = existence?;
        let def = defs.iter().find(|(k, _)| *k == key)?.1;
        let candidate = candidate(existence, departed);
        self.match_def(def, &candidate, asks, seq, event)
            .ok()
            .map(|(index, subjects)| (index.0, subjects, def, existence.controller))
    }
}

/// Each object's list from before a record's event beside its list now. With
/// no seam, every object has one list, which is both.
fn pair<'a>(
    before: Option<&'a [Existence]>,
    now: &'a [Existence],
    now_at: &IdMap<ObjectId, usize>,
) -> Vec<(Option<&'a Existence>, Option<&'a Existence>)> {
    let Some(before) = before else {
        return now.iter().map(|e| (Some(e), Some(e))).collect();
    };
    let mut pairs = Vec::with_capacity(before.len() + 8);
    let mut survivors: IdSet<ObjectId> = IdSet::default();
    for then in before {
        // The same object only while it has not changed zones (CR 400.7).
        let current = now_at.get(&then.object.id).map(|&k| &now[k]).filter(|e| e.object == then.object);
        if current.is_some() {
            survivors.insert(then.object.id);
        }
        pairs.push((Some(then), current));
    }
    pairs.extend(now.iter().filter(|e| !survivors.contains(&e.object.id)).map(|e| (None, Some(e))));
    pairs
}

/// The triggered abilities of one list that function in its zone (CR
/// 113.6), keyed by id and occurrence so two lists' instances of one
/// ability pair up.
fn triggered(existence: &Existence) -> Vec<((AbilityId, usize), &TriggerDef)> {
    let mut defs: Vec<((AbilityId, usize), &TriggerDef)> = Vec::new();
    for ability in existence.chars.abilities.iter() {
        let Effect::Triggered(def) = &ability.effect else { continue };
        if !functions_in(ability, &existence.chars.types, existence.zone) {
            continue;
        }
        let occurrence = defs.iter().filter(|((id, _), _)| *id == ability.id).count();
        defs.push(((ability.id, occurrence), def));
    }
    defs
}

fn candidate(existence: &Existence, departed: bool) -> TriggerCandidate<'_> {
    let frame = if departed {
        TriggerCandidateFrame::Departed(&existence.chars)
    } else {
        TriggerCandidateFrame::Live { chars: Arc::clone(&existence.chars), snapshots: Vec::new() }
    };
    TriggerCandidate {
        id: existence.object.id,
        controller: existence.controller,
        owner: existence.owner,
        zone: existence.zone,
        host: existence.host,
        frame,
        card: Arc::clone(&existence.card),
    }
}

/// The two answers as multisets of triggers. A disagreement panics with the
/// window and both sides, which `fuzz_games` reports with the game's seed.
pub(crate) fn assert_agree(game: &GameState, window: &[EventSeq], engine: &[AuditKey], reference: &[AuditKey]) {
    let mut count: HashMap<&AuditKey, i64> = HashMap::new();
    for key in engine {
        *count.entry(key).or_default() += 1;
    }
    for key in reference {
        *count.entry(key).or_default() -= 1;
    }
    if count.values().all(|&n| n == 0) {
        return;
    }
    let (mut dispatcher_only, mut reference_only) = (Vec::new(), Vec::new());
    for (key, n) in count {
        let side = if n > 0 { &mut dispatcher_only } else { &mut reference_only };
        for _ in 0..n.unsigned_abs() {
            side.push(render(game, key));
        }
    }
    dispatcher_only.sort();
    reference_only.sort();
    let records: Vec<String> = window
        .iter()
        .filter_map(|seq| {
            game.events
                .record(*seq)
                .map(|r| format!("#{} {}", seq.0, crate::ui::display::format_event(game, &r.event)))
        })
        .collect();
    panic!(
        "dispatch audit: the dispatcher and the reference disagree over [{}]\n  dispatcher only: {:?}\n  reference only: {:?}",
        records.join("; "),
        dispatcher_only,
        reference_only
    );
}

fn render(game: &GameState, key: &AuditKey) -> String {
    let source = key.identity.source;
    let name = game.objects.get(&source.id).map_or("(left the game)", |o| o.card_data.name.as_str());
    let records: Vec<usize> = key.records.iter().map(|s| s.0).collect();
    format!(
        "{name} {}@{} {:?} arm {} records {records:?} subject {:?} controller {}",
        source.id, source.zone_change_epoch, key.identity.ability, key.event, key.subject, key.controller
    )
}

#[cfg(test)]
mod tests {
    use super::{assert_agree, AuditKey};
    use crate::objects::object::GameObject;
    use crate::state::game_state::{AbilityIdentity, GameState};
    use crate::test_support::vanilla_creature;
    use crate::types::ids::{new_ability_id, ObjectRef};
    use crate::types::zones::Zone;

    fn key(game: &mut GameState, subject_count: usize) -> Vec<AuditKey> {
        let id = game.add_object(GameObject::new(vanilla_creature(1, 1, &[]), 0, Zone::Graveyard));
        let identity = AbilityIdentity { source: ObjectRef { id, zone_change_epoch: 0 }, ability: new_ability_id() };
        (0..subject_count)
            .map(|k| AuditKey { identity, event: 0, records: vec![crate::events::event::EventSeq(k)], subject: None, controller: 0 })
            .collect()
    }

    /// The comparison fed two different answers: one trigger each side, of
    /// different abilities.
    #[test]
    #[should_panic(expected = "dispatch audit: the dispatcher and the reference disagree")]
    fn two_different_answers_are_a_disagreement() {
        let mut game = GameState::new(2, 20);
        let engine = key(&mut game, 1);
        let reference = key(&mut game, 1);
        assert_agree(&game, &[], &engine, &reference);
    }

    /// A multiset, not a set: one trigger where the reference has it twice.
    #[test]
    #[should_panic(expected = "reference only")]
    fn a_trigger_missing_once_is_a_disagreement() {
        let mut game = GameState::new(2, 20);
        let reference = key(&mut game, 1);
        let engine = reference.clone();
        let reference = [reference.clone(), reference].concat();
        assert_agree(&game, &[], &engine, &reference);
    }

    /// Order is not compared: the `OrderTriggers` prompt owns it.
    #[test]
    fn the_same_triggers_in_another_order_agree() {
        let mut game = GameState::new(2, 20);
        let engine = key(&mut game, 3);
        let reference: Vec<AuditKey> = engine.iter().rev().cloned().collect();
        assert_agree(&game, &[], &engine, &reference);
    }
}
