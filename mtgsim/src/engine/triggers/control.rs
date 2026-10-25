//! CR 603.10d's event — a permanent's controller changing
//! (`triggers-architecture.md` §12, TR-4d).
//!
//! Control is the layer walk's output (CR 613.1b): a control-changing effect
//! starting or ending changes it, and no performer proposes or announces
//! that. So a sweep compares each permanent's controller with the one the
//! stream last announced (`PermanentState::announced_control`) and
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
//! before they write, unless a change is still waiting to be announced.
//! Before the first control-changing row there is no frame to keep, so
//! writing that row keeps them first.

use std::sync::Arc;

use super::dispatch::{LookBackSnapshot, ObjectSnapshot};
use crate::engine::actions::ActionContext;
use crate::engine::audit::assert_shortcut_agrees;
use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::types::{EffectiveCharacteristics, Layer};
use crate::events::cause::{EventCause, GameRule};
use crate::events::event::GameEvent;
use crate::state::battlefield::AnnouncedControl;
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::zones::Zone;

/// One permanent whose controller changed, from whom and to whom.
pub struct ControlChange {
    object: ObjectId,
    from: PlayerId,
    to: PlayerId,
}

/// What a sweep found: each permanent whose controller changed, and every
/// permanent's frame as it was kept before the change, which the change
/// looks back to (CR 603.10d). Every permanent's and not only the ones that
/// changed hands, because one permanent's controller can change another's
/// abilities: "creatures you control have ..." follows its source to the
/// thief, and the creatures left behind lose what it gave them.
pub struct ControlSweep {
    changes: Vec<ControlChange>,
    frames_before: Vec<(ObjectId, Arc<EffectiveCharacteristics>)>,
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
        let frames: Vec<(ObjectId, Option<Arc<EffectiveCharacteristics>>)> =
            self.battlefield_ids_ordered().into_iter().map(|id| (id, compute_characteristics(self, id))).collect();
        // A change no sweep has announced yet looks back to the frames kept
        // before it, so a row written after it keeps nothing: Ray of
        // Command's haste, written after its steal and before any batch.
        let unannounced = frames.iter().any(|(id, frame)| {
            frame.as_ref().zip(self.battlefield.get(id)).is_some_and(|(frame, entry)| frame.controller != entry.announced_control.controller())
        });
        if unannounced {
            return;
        }
        for (id, frame) in frames {
            // Write only a permanent whose kept frame changes: a write copies
            // a permanent a fork shares (`state::shared_stores`), and this runs
            // before every row a resolution or an expiry writes, so rewriting
            // each would copy the battlefield into every fork. Equal by value
            // is enough, since a change of control looks back to the contents.
            let kept = self.battlefield.get(&id).map(|entry| entry.announced_control.frame().map(|kept| &**kept));
            if kept.is_none_or(|kept| kept == frame.as_deref()) {
                continue;
            }
            // Every frame's controller is the announced one here, so a frame
            // replaces the record and none keeps its controller.
            if let Some(entry) = self.battlefield.get_mut(&id) {
                entry.announced_control = match frame {
                    Some(frame) => AnnouncedControl::Frame(frame),
                    None => AnnouncedControl::Controller(entry.announced_control.controller()),
                };
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
    /// stream last announced, as one event, and dispatch it.
    pub(crate) fn announce_control_changes(&mut self, ctx: &ActionContext) -> Result<(), String> {
        let sweep = self.settle_announced_controllers();
        if self.audit_on() {
            self.audit_announced_controllers();
        }
        if sweep.changes.is_empty() {
            return Ok(());
        }
        self.announce(sweep, ctx)
    }

    /// The sweep: write each permanent's controller as the one announced, and
    /// return what changed since the last sweep, with the frames from before.
    /// Announcing it is the caller's: `announce_control_changes` in play, and
    /// nobody for a board built outside play, which takes its controllers as
    /// if each change had been announced as it happened: the scenario
    /// loader's, and a test's that writes a control-changing row straight
    /// into the registry.
    ///
    /// A no-op unless a control-changing effect exists now or did at the last
    /// sweep: that one announces control going back as the last effect ends.
    pub fn settle_announced_controllers(&mut self) -> ControlSweep {
        let rows = self.continuous_effects.summary().any_control_changing;
        let mut sweep = ControlSweep { changes: Vec::new(), frames_before: Vec::new() };
        if !rows && !self.control_effects_at_last_sweep {
            return sweep;
        }
        for id in self.battlefield_ids_ordered() {
            // With no control-changing effect, a permanent's controller is the
            // one it entered under, and there is nothing to walk.
            let frame = if rows { compute_characteristics(self, id) } else { None };
            let Some(entry) = self.battlefield.get(&id) else { continue };
            let now = frame.as_ref().map_or(entry.controller, |frame| frame.controller);
            let announced = entry.announced_control.controller();
            // Write only a permanent whose record would change, for the
            // reason above; an unchanged one hands back its kept frame
            // unwritten, equal in value to its replacement.
            if now == announced && entry.announced_control.frame().map(|kept| &**kept) == frame.as_deref() {
                if let Some(kept) = entry.announced_control.frame() {
                    sweep.frames_before.push((id, Arc::clone(kept)));
                }
                continue;
            }
            let Some(entry) = self.battlefield.get_mut(&id) else { continue };
            let read = match frame {
                Some(frame) => AnnouncedControl::Frame(frame),
                None => AnnouncedControl::Controller(now),
            };
            if let AnnouncedControl::Frame(before) = std::mem::replace(&mut entry.announced_control, read) {
                sweep.frames_before.push((id, before));
            }
            if now != announced {
                sweep.changes.push(ControlChange { object: id, from: announced, to: now });
            }
        }
        self.control_effects_at_last_sweep = rows;
        // CR 611.2b: "for as long as you control" it ends as it changes hands,
        // and a later change back does not begin it again (Dragonlord
        // Silumgar's ruling). What ending a steal returns is the next sweep's.
        for change in &sweep.changes {
            self.continuous_effects.remove_rows_ending_with_control_of(change.object, change.to);
            self.replacement_effects.remove_rows_ending_with_control_of(change.object, change.to);
            self.restrictions.remove_rows_ending_with_control_of(change.object, change.to);
        }
        sweep
    }

    /// The sweep's audit (`engine::audit`): after a sweep, every permanent's
    /// controller, walked, is the one it announced, whether or not its gate
    /// let it walk.
    fn audit_announced_controllers(&self) {
        let ids = self.battlefield_ids_ordered();
        let announced =
            ids.iter().filter_map(|id| Some(format!("{id} {}", self.battlefield.get(id)?.announced_control.controller()))).collect();
        let walked = self.unobserved(|| {
            ids.iter().filter_map(|id| Some(format!("{id} {}", compute_characteristics(self, *id)?.controller))).collect()
        });
        assert_shortcut_agrees("the control sweep's gate", || "every permanent's controller".to_string(), announced, walked);
    }

    /// The changes one sweep found, as one window, with every permanent's
    /// list from before it (`LookBackSnapshot`).
    fn announce(&mut self, sweep: ControlSweep, ctx: &ActionContext) -> Result<(), String> {
        let ControlSweep { changes, frames_before } = sweep;
        let mut frames: Vec<ObjectSnapshot> = Vec::new();
        for (object, before) in frames_before {
            if let Some(lki) = self.lki_with_frame(object, before) {
                frames.push(ObjectSnapshot { zone: Zone::Battlefield, lki });
            }
        }
        let changed: Vec<ObjectId> = changes.iter().map(|change| change.object).collect();
        // NOT REPLACEABLE: CR 506.4 removes a permanent whose controller changed
        // from combat, as part of the change, so the change's triggers read
        // the board without it. An attacker it blocked stays blocked (CR
        // 509.1h), which `remove_from_combat` keeps. Announced in the change's
        // window, after the change.
        let removed = self.remove_from_combat(&changed);
        let mut records: Vec<GameEvent> = Vec::new();
        for change in changes {
            let lki = match frames.iter().find(|frame| frame.lki.object().id == change.object) {
                Some(frame) => frame.lki.clone(),
                None => {
                    debug_assert!(false, "{}'s controller changed with no frame kept from before it", change.object);
                    self.lki_now(change.object)
                        .ok_or_else(|| format!("{}'s controller changed and it is not in the store", change.object))?
                }
            };
            debug_assert_eq!(lki.characteristics().controller, change.from, "{}'s frame from before names another controller", change.object);
            records.push(GameEvent::ControlChanged { to: change.to, lki });
        }
        let audit = self.audit_frames_before_control_change(&frames);

        // Inside an outermost batch's close the records join its window, and
        // its close dispatches them.
        let previous = self.events.open_batch();
        let joined = previous.is_some();
        let (mark, window) = (self.events.next_seq(), self.events.current_batch());
        for record in records {
            self.emit_event(record, EventCause::GameRule(GameRule::ControlChanges))?;
        }
        self.announce_removal_from_combat(removed, EventCause::GameRule(GameRule::ControlChanges))?;
        let performed = mark.0..self.events.next_seq().0;
        self.events.close_batch(previous);
        self.look_back_snapshots.push(LookBackSnapshot { window, performed: performed.clone(), frames });
        if let Some(frames) = audit {
            self.file_audit_snapshot(LookBackSnapshot { window, performed, frames });
        }
        if joined {
            return Ok(());
        }
        self.dispatch_batch(mark, window, ctx)
    }
}
