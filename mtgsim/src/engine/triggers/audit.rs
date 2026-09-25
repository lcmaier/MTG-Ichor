//! The dispatch audit — the dispatcher again, with its shortcuts off
//! (`triggers-architecture.md` §4.10, TR-1b).
//!
//! The dispatcher reaches its answer through shortcuts: a gate on the kinds
//! of record a window carries, three candidate sets kept as objects move
//! (printed sources on the battlefield, the zone map, the zones a granting
//! or copying row reaches), a look-back snapshot taken only when a batch
//! departs an ability list's source, and a departure's frame off its record.
//! Every wrong answer it has had was a shortcut that left something out (F1,
//! items 167 and 174), and none of them fails a random game on its own. In a
//! game that turns the audit on, every dispatch is answered twice by the same
//! matching loop (`match_candidates`): once over the dispatcher's candidates,
//! once over a set with no shortcut in it — every object that could carry a
//! triggered ability, with its list now, and with the list it had before each
//! batch of the window performed. The two answers must agree.
//!
//! **What it shares, and so cannot check**, is the loop itself: CR 113.6
//! asked per ability, `match_def`, and how many triggers one ability makes of
//! one event. Those are rules, and each has its fixtures.
//!
//! **Its reads leave no trace.** The layer memo, the diagnostics and the trace
//! handle are saved before each and restored after, so an audited game's
//! counters and trace are an unaudited one's.

use std::sync::Arc;

use super::dispatch::{LookBackSnapshot, MatchedTrigger, ObjectSnapshot, TriggerCandidate, TriggerCandidateFrame};
use super::history::TurnOrdinals;
use crate::engine::layers::compute::{compute_characteristics, no_row_reaches};
use crate::events::event::{BatchId, EventRecord, EventSeq};
use crate::objects::card_data::CardDataBuilder;
use crate::oracle::characteristics::controller_or_owner;
use crate::state::diagnostics::Diagnostics;
use crate::state::game_state::GameState;
use crate::state::layer_memo::LayerMemo;
use crate::state::trace::TraceHandle;
use crate::types::effects::Effect;
use crate::types::ids::ObjectId;
use crate::types::zones::Zone;

/// The audit's state on a game that turned it on.
#[derive(Debug, Clone, Default)]
pub struct DispatchAudit {
    /// Every object's list before each batch of the open windows performed.
    snapshots: Vec<LookBackSnapshot>,
    /// Dispatches answered twice.
    dispatches: u64,
    /// Triggers the two answers agreed on.
    triggers: u64,
}

/// What an audit read may touch and must put back.
struct Observers {
    memo: LayerMemo,
    diagnostics: Diagnostics,
    trace: Option<TraceHandle>,
}

impl GameState {
    /// Audit every dispatch for the rest of this game (`fuzz_games --audit`).
    /// The hooks it needs are inside the batch and the dispatch, so the switch
    /// is the engine's; a second call changes nothing.
    pub fn enable_dispatch_audit(&mut self) {
        if self.dispatch_audit.is_none() {
            self.dispatch_audit = Some(Box::new(DispatchAudit::default()));
        }
    }

    /// Dispatches answered twice and triggers the two answers agreed on;
    /// `None` in a game that never turned the audit on.
    pub fn dispatch_audit_counts(&self) -> Option<(u64, u64)> {
        self.dispatch_audit.as_ref().map(|audit| (audit.dispatches, audit.triggers))
    }

    /// The audit's snapshot before a batch performs: every object that could
    /// carry a triggered ability, with its list. `None` when the audit is off.
    pub(crate) fn audit_frames(&mut self) -> Option<Vec<ObjectSnapshot>> {
        self.dispatch_audit.as_ref()?;
        let saved = self.save_observers();
        let frames = self.objects_that_may_trigger().into_iter().filter_map(|id| self.object_snapshot(id)).collect();
        self.restore_observers(saved);
        Some(frames)
    }

    /// Keep a batch's snapshot for its window's dispatch.
    pub(crate) fn file_audit_snapshot(&mut self, snapshot: LookBackSnapshot) {
        if let Some(audit) = self.dispatch_audit.as_mut() {
            audit.snapshots.push(snapshot);
        }
    }

    /// `window`'s snapshots, taken by its dispatch; `None` when the audit is off.
    pub(super) fn take_audit_snapshots(&mut self, window: Option<BatchId>) -> Option<Vec<LookBackSnapshot>> {
        let audit = self.dispatch_audit.as_mut()?;
        let (mine, others) = std::mem::take(&mut audit.snapshots).into_iter().partition(|s| s.window == window);
        audit.snapshots = others;
        Some(mine)
    }

    /// Answer `window` again with every shortcut off, and panic unless the
    /// answer is the dispatcher's.
    pub(super) fn audit_dispatch(
        &mut self,
        window: &[EventSeq],
        snapshots: &[LookBackSnapshot],
        engine: &[MatchedTrigger],
        ordinals: &TurnOrdinals,
    ) {
        let saved = self.save_observers();
        let reference = self.matches_without_shortcuts(window, snapshots, ordinals);
        self.restore_observers(saved);
        let describe = |matches: &[MatchedTrigger]| matches.iter().map(|m| self.describe(m)).collect::<Vec<_>>();
        assert_agree(&self.describe_window(window), describe(engine), describe(&reference));
        if let Some(audit) = self.dispatch_audit.as_mut() {
            audit.dispatches += 1;
            audit.triggers += engine.len() as u64;
        }
    }

    fn save_observers(&mut self) -> Observers {
        Observers {
            memo: self.layer_memo.clone(),
            diagnostics: self.diagnostics.clone(),
            trace: self.trace.take(),
        }
    }

    fn restore_observers(&mut self, saved: Observers) {
        self.layer_memo = saved.memo;
        self.diagnostics = saved.diagnostics;
        self.trace = saved.trace;
    }

    /// Every object in every zone that could carry a triggered ability: all
    /// of them on the battlefield and the stack, and elsewhere every object a
    /// continuous effect could reach or that printed one — an object no row
    /// reaches has its printed abilities and nothing else. An ability on the
    /// stack is an object (CR 113.1c) without its source's abilities.
    fn objects_that_may_trigger(&self) -> Vec<ObjectId> {
        let printed_trigger =
            |id: &ObjectId| self.objects.get(id).is_some_and(|o| o.card_data.abilities.iter().any(|a| matches!(a.effect, Effect::Triggered(_))));
        let mut ids = self.battlefield_ids_ordered();
        ids.extend(self.stack.iter().copied().filter(|id| self.stack_entries.get(id).is_none_or(|e| e.is_spell)));
        for zone in [Zone::Exile, Zone::Command, Zone::Graveyard, Zone::Hand, Zone::Library] {
            ids.extend(self.zone_ids_ordered(zone).into_iter().filter(|id| !no_row_reaches(self, *id) || printed_trigger(id)));
        }
        ids
    }

    /// The window's triggers over the candidate set with no shortcut in it.
    /// A record some batch performed reads look-back arms off that batch's
    /// snapshot — a survivor's list from before, or a departed object's —
    /// and every other arm off the list now; a record no batch performed
    /// reads everything off the list now.
    fn matches_without_shortcuts(
        &self,
        window: &[EventSeq],
        snapshots: &[LookBackSnapshot],
        ordinals: &TurnOrdinals,
    ) -> Vec<MatchedTrigger> {
        let records: Vec<(EventSeq, &EventRecord)> =
            window.iter().filter_map(|seq| self.events.record(*seq).map(|r| (*seq, r))).collect();
        let every_snapshot: Vec<usize> = (0..snapshots.len()).collect();
        let mut candidates: Vec<TriggerCandidate<'_>> = Vec::new();
        for id in self.objects_that_may_trigger() {
            let Some(object) = self.objects.get(&id) else { continue };
            let Some(chars) = compute_characteristics(self, id) else { continue };
            candidates.push(TriggerCandidate {
                id,
                controller: controller_or_owner(self, id).unwrap_or(object.owner),
                owner: object.owner,
                zone: object.zone,
                host: self.battlefield.get(&id).and_then(|e| e.attached_to),
                frame: TriggerCandidateFrame::Live { chars, snapshots: every_snapshot.clone() },
                card: Arc::clone(&object.card_data),
            });
        }
        for (k, snapshot) in snapshots.iter().enumerate() {
            for frame in &snapshot.frames {
                let id = frame.object.id;
                let candidate = match self.objects.get(&id) {
                    Some(object) if object.zone_change_epoch == frame.object.zone_change_epoch => TriggerCandidate {
                        id,
                        controller: controller_or_owner(self, id).unwrap_or(object.owner),
                        owner: object.owner,
                        zone: object.zone,
                        host: self.battlefield.get(&id).and_then(|e| e.attached_to),
                        frame: TriggerCandidateFrame::Before { chars: &frame.chars, snapshot: k },
                        card: Arc::clone(&object.card_data),
                    },
                    // Moved or gone since the snapshot: its list from then,
                    // under its controller then (CR 603.3a, 603.10a).
                    departed => TriggerCandidate {
                        id,
                        controller: frame.chars.controller,
                        owner: frame.owner,
                        zone: frame.zone,
                        host: None,
                        frame: TriggerCandidateFrame::Departed { chars: &frame.chars, snapshot: Some(k) },
                        card: match departed {
                            Some(object) => Arc::clone(&object.card_data),
                            None => CardDataBuilder::new(&frame.chars.name).build(),
                        },
                    },
                };
                candidates.push(candidate);
            }
        }
        self.match_candidates(&records, &candidates, snapshots, ordinals)
    }

    /// A trigger as the comparison and its report read it.
    fn describe(&self, matched: &MatchedTrigger) -> String {
        let source = matched.identity.source;
        let mut records: Vec<usize> = matched.records.iter().map(|r| r.seq.0).collect();
        records.sort_unstable();
        format!(
            "{} {}@{} ability {} arm {} records {records:?} subject {:?} controller {}",
            crate::ui::display::card_name(self, source.id),
            source.id,
            source.zone_change_epoch,
            matched.identity.ability,
            matched.event.0,
            matched.subject,
            matched.controller
        )
    }

    fn describe_window(&self, window: &[EventSeq]) -> String {
        let records: Vec<String> = window
            .iter()
            .filter_map(|seq| {
                self.events.record(*seq).map(|r| format!("#{} {}", seq.0, crate::ui::display::format_event(self, &r.event)))
            })
            .collect();
        records.join("; ")
    }
}

/// The two answers as multisets. A disagreement panics with the window and
/// what each side had that the other did not; `fuzz_games` reports it with the
/// game's seed.
pub(crate) fn assert_agree(window: &str, mut dispatcher: Vec<String>, mut reference: Vec<String>) {
    dispatcher.sort();
    reference.sort();
    if dispatcher == reference {
        return;
    }
    let without = |from: &[String], other: &[String]| {
        let mut rest = other.to_vec();
        let mut only = Vec::new();
        for item in from {
            match rest.iter().position(|r| r == item) {
                Some(at) => {
                    rest.remove(at);
                }
                None => only.push(item.clone()),
            }
        }
        only
    };
    panic!(
        "dispatch audit: the dispatcher and the audit disagree over [{window}]\n  dispatcher only: {:?}\n  audit only: {:?}",
        without(&dispatcher, &reference),
        without(&reference, &dispatcher)
    );
}

#[cfg(test)]
mod tests {
    use super::assert_agree;

    fn triggers(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    /// The comparison fed two different answers.
    #[test]
    #[should_panic(expected = "dispatch audit: the dispatcher and the audit disagree")]
    fn two_different_answers_are_a_disagreement() {
        assert_agree("", triggers(&["A"]), triggers(&["B"]));
    }

    /// A multiset, not a set: one trigger where the audit has it twice.
    #[test]
    #[should_panic(expected = "audit only: [\"A\"]")]
    fn a_trigger_missing_once_is_a_disagreement() {
        assert_agree("", triggers(&["A"]), triggers(&["A", "A"]));
    }

    /// Order is not compared: the `OrderTriggers` prompt owns it.
    #[test]
    fn the_same_triggers_in_another_order_agree() {
        assert_agree("", triggers(&["A", "B", "C"]), triggers(&["C", "A", "B"]));
    }
}
