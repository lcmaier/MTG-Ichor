//! What the layers' output changes, settled (`triggers-architecture.md` §12,
//! TR-5f, decision 13).
//!
//! A permanent's controller and types are the layer walk's answer (CR
//! 613.1b, 613.1d), and the answer moves at any write to the walk's inputs:
//! a continuous effect starting or ending, a static ability's "as long as"
//! condition starting or stopping to hold, an object entering, leaving or
//! gaining a counter. None of those writes says what it did to another
//! permanent's controller or types. The CR makes two such changes matter at
//! once: a change of control is an event that removes the permanent from
//! combat (CR 603.10d, 506.4), and a combatant that stops being a creature,
//! or becomes a battle, leaves combat (CR 506.4). One settlement answers
//! both, in that order, at the head of an outermost batch, at its close
//! before its window is dispatched, at each pass of the state-based check,
//! and in the cleanup step.
//!
//! At a batch's close the settlement's records join the batch's window, so
//! the batch's own triggers read the board after them (CR 603.10's "objects
//! that exist immediately after an event"); anywhere else they are a window
//! of their own.

use crate::engine::actions::ActionContext;
use crate::events::cause::{EventCause, GameRule};
use crate::oracle::characteristics::is_nonbattle_creature;
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, Timestamp};

impl GameState {
    /// Settle what the layers' output changed since the last settlement:
    /// announce each change of control, then remove from combat each
    /// combatant that can no longer be in it.
    ///
    /// Gated on the layer epoch, the number the frame cache trusts: with the
    /// epoch where the last settlement left it, every frame is the one that
    /// settlement read, so there is nothing to find.
    pub(crate) fn settle_layer_consequences(&mut self, ctx: &ActionContext) -> Result<(), String> {
        let epoch = self.layer_epoch();
        if epoch == self.settled_layer_epoch {
            return Ok(());
        }
        self.settled_layer_epoch = epoch;
        self.announce_control_changes(ctx)?;
        self.remove_combatants_that_changed_type()
    }

    /// CR 506.4: "an attacking or blocking creature that ... stops being a
    /// creature, or becomes a battle" is removed from combat, everything one
    /// change ends at once. CR 506.4d and 506.4e, an attacked planeswalker or
    /// battle that stops being one, join this check with `backlog.md` §2.23's
    /// attack path.
    fn remove_combatants_that_changed_type(&mut self) -> Result<(), String> {
        if !self.combat_had_attackers {
            return Ok(());
        }
        // ORDER-FREE: the combatants are sorted by timestamp next.
        let mut combatants: Vec<(Timestamp, ObjectId)> = self
            .battlefield
            .iter()
            .filter(|(_, entry)| entry.attacking.is_some() || entry.blocking.is_some())
            .map(|(&id, entry)| (entry.timestamp, id))
            .collect();
        combatants.sort_by_key(|&(timestamp, _)| timestamp);
        let changed: Vec<ObjectId> = combatants
            .into_iter()
            .map(|(_, id)| id)
            .filter(|&id| !is_nonbattle_creature(self, id))
            .collect();
        // NOT REPLACEABLE: CR 506.4's removal, a consequence of the type change
        // rather than a replaceable event (`replacement-architecture.md` §8a),
        // announced as one record.
        let removed = self.remove_from_combat(&changed);
        self.announce_removal_from_combat(removed, EventCause::GameRule(GameRule::CombatantTypeChanged))
    }
}
