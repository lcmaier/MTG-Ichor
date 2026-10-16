//! CR 603.10d's event — a permanent's controller changing
//! (`triggers-architecture.md` §12, TR-4d).
//!
//! Control is the layer walk's output (CR 613.1b): a control-changing effect
//! starting or ending changes it, and no performer proposes or announces
//! that. So a sweep compares each permanent's controller with the one the
//! stream last announced (`PermanentState::announced_controller`) and
//! announces each difference as a `GameEvent::ControlChanged`, the changes
//! one sweep finds as one event. Its points are the head of an outermost
//! batch, each pass of the state-based check, and the cleanup step after
//! CR 514.2's expiry.
//!
//! A trigger on the change looks back (CR 603.10d), so each permanent keeps
//! the frame the last sweep read while a control-changing effect exists, when
//! the comparison walks it anyway. Control moves only when the board does: in
//! a batch, whose head is a sweep; or in a row a resolution writes, a
//! static's registration, or an expiry, which keep the frames again just
//! before they write. Before the first control-changing row there is no
//! frame to keep, so writing that row keeps them first.

use std::sync::Arc;

use super::dispatch::{LookBackSnapshot, ObjectSnapshot};
use crate::engine::actions::ActionContext;
use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::types::{EffectiveCharacteristics, Layer};
use crate::events::cause::{EventCause, GameRule};
use crate::events::event::GameEvent;
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::zones::Zone;

/// One permanent whose controller changed: from whom, to whom, and its frame
/// from before.
struct ControlChange {
    object: ObjectId,
    from: PlayerId,
    to: PlayerId,
    before: Option<Arc<EffectiveCharacteristics>>,
}

/// A permanent whose controller did not change, with its frame from before
/// the sweep and its frame now: the change may still have altered its
/// ability list, as a "you control" grant moving does.
struct Unchanged {
    object: ObjectId,
    before: Arc<EffectiveCharacteristics>,
    now: Arc<EffectiveCharacteristics>,
}

impl GameState {
    /// Keep each permanent's frame as it is now, as the one the next change
    /// of its controller looks back to. Only while a control-changing effect
    /// exists, or when `sets_controller`: the write about to happen is the
    /// first such effect, and nothing has kept a frame before it.
    pub(crate) fn keep_frames_before_control_change(&mut self, sets_controller: bool) {
        if !sets_controller && !self.continuous_effects.summary().any_control_changing {
            return;
        }
        for id in self.battlefield_ids_ordered() {
            let frame = compute_characteristics(self, id);
            if let Some(entry) = self.battlefield.get_mut(&id) {
                entry.frame_before_control_change = frame;
            }
        }
    }

    /// Before a static ability's `layer` row is registered, in the batch its
    /// object enters or gains the ability in: that batch's head kept the
    /// frames while a control-changing effect existed, so only the first such
    /// effect keeps them here.
    pub(crate) fn keep_frames_before_the_first_control_row(&mut self, layer: Layer) {
        if layer == Layer::Layer2Control && !self.continuous_effects.summary().any_control_changing {
            self.keep_frames_before_control_change(true);
        }
    }

    /// Announce every permanent whose controller is no longer the one the
    /// stream last announced, as one event, and dispatch it. A no-op unless a
    /// control-changing effect exists or a permanent is announced away from
    /// the controller it entered under, which is the change back as the last
    /// such effect ends.
    pub(crate) fn announce_control_changes(&mut self, ctx: &ActionContext) -> Result<(), String> {
        let (changes, kept) = self.sweep_controllers();
        if changes.is_empty() {
            return Ok(());
        }
        self.announce(changes, kept, ctx)
    }

    /// Take every permanent's controller as announced, and announce nothing:
    /// a board built outside play, as if every change of control on it had
    /// been announced as it happened. The scenario loader's, at the priority
    /// point it builds; and a test's that writes a control-changing row
    /// straight into the registry.
    pub fn settle_announced_controllers(&mut self) {
        self.sweep_controllers();
    }

    /// The sweep: each permanent's controller against the one announced,
    /// written as announced, with the frame from before for each that
    /// changed, and the pair of frames for each that did not.
    fn sweep_controllers(&mut self) -> (Vec<ControlChange>, Vec<Unchanged>) {
        let rows = self.continuous_effects.summary().any_control_changing;
        let mut changes: Vec<ControlChange> = Vec::new();
        let mut kept: Vec<Unchanged> = Vec::new();
        if !rows && !self.control_announced_away {
            return (changes, kept);
        }
        let mut away = false;
        for id in self.battlefield_ids_ordered() {
            // With no control-changing effect, a permanent's controller is the
            // one it entered under, and there is nothing to walk.
            let frame = if rows { compute_characteristics(self, id) } else { None };
            let Some(entry) = self.battlefield.get_mut(&id) else { continue };
            let now = frame.as_ref().map_or(entry.controller, |frame| frame.controller);
            let before = std::mem::replace(&mut entry.frame_before_control_change, frame.clone());
            if now != entry.announced_controller {
                changes.push(ControlChange { object: id, from: entry.announced_controller, to: now, before });
                entry.announced_controller = now;
            } else if let (Some(before), Some(now)) = (before, frame) {
                kept.push(Unchanged { object: id, before, now });
            }
            away |= now != entry.controller;
        }
        self.control_announced_away = away;
        (changes, kept)
    }

    /// The changes one sweep found, as one window, with the lists from
    /// before it (`LookBackSnapshot`): each changed permanent's, and each
    /// other permanent's whose ability list the change altered, as a "you
    /// control" grant moving does.
    fn announce(
        &mut self,
        changes: Vec<ControlChange>,
        kept: Vec<Unchanged>,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        let mut frames: Vec<ObjectSnapshot> = Vec::new();
        let mut records: Vec<GameEvent> = Vec::new();
        for change in changes {
            let before = match change.before {
                Some(before) => before,
                None => {
                    debug_assert!(false, "{}'s controller changed with no frame kept from before it", change.object);
                    compute_characteristics(self, change.object)
                        .ok_or_else(|| format!("{}'s controller changed and it has no characteristics", change.object))?
                }
            };
            let lki = self
                .last_known_information(change.object, before)
                .ok_or_else(|| format!("{}'s controller changed and it is not in the store", change.object))?;
            frames.push(ObjectSnapshot { zone: Zone::Battlefield, lki: lki.clone() });
            records.push(GameEvent::ControlChanged { object_id: change.object, from: change.from, to: change.to, lki });
        }
        for Unchanged { object, before, now } in kept {
            if before.abilities != now.abilities
                && let Some(lki) = self.last_known_information(object, before)
            {
                frames.push(ObjectSnapshot { zone: Zone::Battlefield, lki });
            }
        }
        let audit = self.audit_frames_before_control_change(&frames);

        let previous = self.events.open_batch();
        let (mark, window) = (self.events.next_seq(), self.events.current_batch());
        for record in records {
            self.emit_event(record, EventCause::GameRule(GameRule::ControlChanges))?;
        }
        let performed = mark.0..self.events.next_seq().0;
        self.events.close_batch(previous);
        self.look_back_snapshots.push(LookBackSnapshot { window, performed: performed.clone(), frames });
        if let Some(frames) = audit {
            self.file_audit_snapshot(LookBackSnapshot { window, performed, frames });
        }
        self.dispatch_batch(mark, window, ctx)
    }
}
