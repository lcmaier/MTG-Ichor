// Combat step processing — turn-based actions for declare attackers,
// declare blockers, and combat damage steps.
// See rules 508, 509, 510.

use std::collections::HashSet;

use crate::engine::combat::resolution::assign_combat_damage;
use crate::engine::combat::validation::{
    validate_attackers, validate_blockers,
    AttackConstraints, BlockConstraints,
};
use crate::events::event::GameEvent;
use crate::oracle::characteristics::has_keyword;
use crate::state::battlefield::{AttackTarget, AttackingInfo, BlockingInfo};
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::keywords::KeywordAbility;
use crate::ui::decision::DecisionProvider;

impl GameState {
    /// Declare attackers turn-based action (rule 508.1).
    ///
    /// Asks the active player to choose attackers via `DecisionProvider`,
    /// validates, taps them, and updates battlefield state.
    /// Returns `true` if any attackers were declared.
    pub fn process_declare_attackers(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<bool, String> {
        let active = self.active_player;
        let proposed = decisions.choose_attackers(self, active);

        if proposed.is_empty() {
            return Ok(false);
        }

        // Validate the proposed attackers
        // Phase 3: no constraints
        validate_attackers(self, active, &proposed, &AttackConstraints::none())
            .map_err(|e| format!("Invalid attackers: {}", e))?;

        // Pre-collect vigilance set to avoid borrow-checker conflict
        // (has_keyword borrows self.objects, battlefield.get_mut borrows self.battlefield)
        let vigilance_set: HashSet<ObjectId> = proposed.iter()
            .filter(|(id, _)| has_keyword(self, *id, KeywordAbility::Vigilance))
            .map(|(id, _)| *id)
            .collect();

        // Apply: tap each attacker and set attacking info (rule 508.1f)
        for (creature_id, target) in &proposed {
            if let Some(entry) = self.battlefield.get_mut(creature_id) {
                // Rule 702.20b: Vigilance prevents tapping from attacking
                if !vigilance_set.contains(creature_id) {
                    entry.tapped = true;
                }
                entry.attacking = Some(AttackingInfo {
                    target: target.clone(),
                    is_blocked: false,
                    blocked_by: Vec::new(),
                });
            }
        }

        self.attacks_declared = true;

        let attacker_ids: Vec<ObjectId> = proposed.iter().map(|(id, _)| *id).collect();
        self.events.emit(GameEvent::AttackersDeclared {
            attackers: attacker_ids,
        });

        Ok(true)
    }

    /// Declare blockers turn-based action (rule 509.1).
    ///
    /// For each defending player, asks them to choose blockers via
    /// `DecisionProvider`, validates, and updates battlefield state.
    pub fn process_declare_blockers(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        // Find defending players — each player being attacked
        let defending_players: Vec<PlayerId> = self.get_defending_players();

        for defender in defending_players {
            let proposed = decisions.choose_blockers(self, defender);

            if proposed.is_empty() {
                continue;
            }

            // Validate
            validate_blockers(self, defender, &proposed, &BlockConstraints::none())
                .map_err(|e| format!("Invalid blockers: {}", e))?;

            // Apply: set blocking info and update attacker's blocked_by
            for (blocker_id, attacker_id) in &proposed {
                // Mark blocker
                if let Some(entry) = self.battlefield.get_mut(blocker_id) {
                    if let Some(ref mut info) = entry.blocking {
                        info.blocking.push(*attacker_id);
                    } else {
                        entry.blocking = Some(BlockingInfo {
                            blocking: vec![*attacker_id],
                        });
                    }
                }

                // Mark attacker as blocked
                if let Some(entry) = self.battlefield.get_mut(attacker_id) {
                    if let Some(ref mut info) = entry.attacking {
                        info.is_blocked = true;
                        info.blocked_by.push(*blocker_id);
                    }
                }
            }

            let blocker_pairs: Vec<(ObjectId, ObjectId)> = proposed.clone();
            self.events.emit(GameEvent::BlockersDeclared {
                blockers: blocker_pairs,
            });
        }

        self.blockers_declared = true;

        Ok(())
    }

    /// Combat damage turn-based action (rule 510).
    ///
    /// `first_strike_only`: if true, only first/double strike creatures deal damage.
    /// If no creature in combat has first strike or double strike, the first-strike
    /// step is skipped entirely (returns Ok immediately).
    pub fn process_combat_damage(
        &mut self,
        decisions: &dyn DecisionProvider,
        first_strike_only: bool,
    ) -> Result<(), String> {
        if first_strike_only {
            // Check if any creature in combat has first strike or double strike
            let any_first_strike = self.battlefield.values().any(|e| {
                (e.attacking.is_some() || e.blocking.is_some())
                && (has_keyword(self, e.object_id, KeywordAbility::FirstStrike)
                    || has_keyword(self, e.object_id, KeywordAbility::DoubleStrike))
            });

            if !any_first_strike {
                // No first/double strike creatures → skip this step entirely
                return Ok(());
            }
        }

        let active = self.active_player;

        // Compute damage assignments (read-only, delegates to DecisionProvider
        // for multi-blocker damage division)
        let assignments = assign_combat_damage(
            self,
            decisions,
            active,
            first_strike_only,
        );

        // Apply damage (mutating)
        self.apply_combat_damage(assignments)?;

        // Track who dealt damage in the first-strike step
        if first_strike_only {
            // Collect IDs first to avoid borrow conflict
            let fs_ids: Vec<ObjectId> = self.battlefield.values()
                .filter(|e| {
                    (e.attacking.is_some() || e.blocking.is_some())
                    && (has_keyword(self, e.object_id, KeywordAbility::FirstStrike)
                        || has_keyword(self, e.object_id, KeywordAbility::DoubleStrike))
                })
                .map(|e| e.object_id)
                .collect();
            for id in fs_ids {
                self.dealt_first_strike_damage.insert(id);
            }
        }

        Ok(())
    }

    /// Get the list of defending players (players being attacked).
    fn get_defending_players(&self) -> Vec<PlayerId> {
        let mut defenders = Vec::new();
        for (_id, entry) in &self.battlefield {
            if let Some(ref info) = entry.attacking {
                if let AttackTarget::Player(pid) = info.target {
                    if !defenders.contains(&pid) {
                        defenders.push(pid);
                    }
                }
            }
        }
        defenders
    }

}
