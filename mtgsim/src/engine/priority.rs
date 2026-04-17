use crate::oracle::legality::candidate_priority_actions;
use crate::state::game_state::GameState;
use crate::types::zones::Zone;
use crate::ui::ask::ask_choose_priority_action;
use crate::ui::decision::{DecisionProvider, PriorityAction};

/// Result of a single priority round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityResult {
    /// The stack was resolved (top item resolved). Another round should follow.
    StackResolved,
    /// The stack is empty and all players passed — the current step/phase ends.
    PhaseEnds,
    /// A player took an action (cast/activate/play land). Continue the round.
    ActionTaken,
}

impl GameState {
    /// Run a complete priority round (rule 117).
    ///
    /// This is the main game loop for a single "priority passing" cycle:
    /// 1. Before granting priority, perform SBAs until stable (rule 117.5).
    /// 2. Active player gets priority.
    /// 3. If they act → they get priority again (117.3c).
    /// 4. If they pass → next player gets priority (117.3d).
    /// 5. If all pass in succession:
    ///    - Stack non-empty → resolve top (117.4 / 405.5), return StackResolved.
    ///    - Stack empty → return PhaseEnds.
    pub fn run_priority_round(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<PriorityResult, String> {
        // --- Rule 117.5: SBAs before granting priority ---
        self.perform_sba_and_triggers(decisions)?;

        let num_players = self.num_players();
        let mut consecutive_passes = 0;
        let mut current_priority = self.active_player;

        loop {
            self.priority_player = current_priority;

            // `candidate_priority_actions` is an overapproximation: it includes
            // e.g. `CastSpell(id)` when affordability is heuristically met but
            // the current mana pool can't actually cover the cost (see
            // `plans/atomic-tests/supplemental-docs/dp-middleware-and-candidate-enumeration.md`).
            // Per that design (§2.2), the engine is responsible for retrying
            // when execution rejects a DP-chosen action.
            //
            // Retry loop (D26 / SPECIAL-2):
            //   - On execution failure, blacklist the specific action for this
            //     priority window and re-prompt with the filtered list.
            //   - Bound retries at `3 × candidates.len()` (minimum 6) and fall
            //     back to `Pass` when the budget is exhausted. This prevents
            //     infinite loops and gives RandomDP / buggy candidate filters
            //     a diagnostic signal via stderr.
            let all_candidates = candidate_priority_actions(self, current_priority);
            let max_retries = all_candidates.len().saturating_mul(3).max(6);
            let mut blacklist: Vec<PriorityAction> = Vec::new();
            let mut retries: usize = 0;

            // Choose an action and attempt execution; retry on failure.
            // On success, `executed` holds the action that ran and (for
            // ActivateAbility) whether it was a mana ability (which bypasses
            // the post-action SBA pass, per rule 605).
            let executed: (PriorityAction, bool) = loop {
                let available: Vec<PriorityAction> = all_candidates
                    .iter()
                    .filter(|a| !blacklist.contains(a))
                    .cloned()
                    .collect();

                // If every non-Pass candidate has been blacklisted (or the
                // retry budget is exhausted), force a Pass. Pass is always
                // safe — it has no execution path that can fail.
                if available.is_empty() || retries >= max_retries {
                    if retries >= max_retries {
                        eprintln!(
                            "WARN: priority retry budget ({}) exhausted for player {} — forcing Pass. \
                             Blacklist size: {}. This is a diagnostic signal that `castable_spells` / \
                             `activatable_abilities` may be too loose.",
                            max_retries, current_priority, blacklist.len()
                        );
                    }
                    break (PriorityAction::Pass, false);
                }

                let action = ask_choose_priority_action(
                    decisions, self, current_priority, &available,
                );

                // Pass doesn't execute anything — accept it immediately.
                if matches!(action, PriorityAction::Pass) {
                    break (action, false);
                }

                // Attempt execution. On Err, the callee is required to leave
                // game state clean (see `cast_spell` rollback via move_object,
                // `activate_ability` via `rollback_ability_activation`).
                let (exec_result, was_mana_ability) = match &action {
                    PriorityAction::Pass => unreachable!(),
                    PriorityAction::CastSpell(card_id) => (
                        self.cast_spell(current_priority, *card_id, decisions),
                        false,
                    ),
                    PriorityAction::PlayLand(card_id) => (
                        self.play_land(current_priority, *card_id, Zone::Hand),
                        false,
                    ),
                    PriorityAction::ActivateAbility(permanent_id, ability_id) => {
                        // Dispatch mana-vs-non-mana. Mana abilities resolve
                        // immediately (rule 605) and don't trigger SBAs.
                        let card_data = match self.get_object(*permanent_id) {
                            Ok(obj) => obj.card_data.clone(),
                            Err(e) => {
                                // Source disappeared — blacklist and retry.
                                blacklist.push(action.clone());
                                retries = retries.saturating_add(1);
                                eprintln!(
                                    "WARN: activate_ability source {} missing: {}",
                                    permanent_id, e
                                );
                                continue;
                            }
                        };
                        let is_mana = card_data.abilities.iter()
                            .find(|a| a.id == *ability_id)
                            .map(|a| a.ability_type == crate::objects::card_data::AbilityType::Mana)
                            .unwrap_or(false);
                        let result = if is_mana {
                            self.activate_mana_ability(current_priority, *permanent_id, *ability_id)
                        } else {
                            let idx = card_data.abilities.iter()
                                .position(|a| a.id == *ability_id);
                            match idx {
                                Some(i) => self.activate_ability(
                                    current_priority, *permanent_id, i, decisions,
                                ),
                                None => Err(format!(
                                    "Ability {} not found on permanent {}",
                                    ability_id, permanent_id
                                )),
                            }
                        };
                        (result, is_mana)
                    }
                };

                match exec_result {
                    Ok(()) => break (action, was_mana_ability),
                    Err(_e) => {
                        blacklist.push(action);
                        retries = retries.saturating_add(1);
                        // Loop again with tighter candidate list.
                    }
                }
            };

            match executed.0 {
                PriorityAction::Pass => {
                    consecutive_passes += 1;
                    if consecutive_passes >= num_players {
                        // All players passed in succession (rule 117.4)
                        if self.stack.is_empty() {
                            return Ok(PriorityResult::PhaseEnds);
                        } else {
                            self.resolve_top_of_stack(decisions)?;
                            // After resolution, active player gets priority (117.3b)
                            // Run SBAs again before granting (117.5)
                            self.perform_sba_and_triggers(decisions)?;
                            return Ok(PriorityResult::StackResolved);
                        }
                    }
                    // Next player gets priority (117.3d)
                    current_priority = (current_priority + 1) % num_players;
                }

                PriorityAction::CastSpell(_) => {
                    // Player who acted gets priority again (117.3c) — we
                    // return and let the caller start a fresh round.
                    self.perform_sba_and_triggers(decisions)?;
                    return Ok(PriorityResult::ActionTaken);
                }

                PriorityAction::ActivateAbility(_, _) => {
                    // Mana abilities resolve immediately (rule 605) with no
                    // SBA pass; other activated abilities go on the stack and
                    // get the normal SBA sweep.
                    if !executed.1 {
                        self.perform_sba_and_triggers(decisions)?;
                    }
                    return Ok(PriorityResult::ActionTaken);
                }

                PriorityAction::PlayLand(_) => {
                    // Playing a land is a special action (rule 116.2a) —
                    // player keeps priority, caller loops back.
                    return Ok(PriorityResult::ActionTaken);
                }
            }
        }
    }

    /// Run priority rounds until the phase/step ends or the game ends.
    ///
    /// This loops `run_priority_round` until all players pass with an empty
    /// stack. After each stack resolution, another round begins.
    pub fn run_priority_loop(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        loop {
            match self.run_priority_round(decisions)? {
                PriorityResult::PhaseEnds => return Ok(()),
                PriorityResult::StackResolved | PriorityResult::ActionTaken => {
                    // Continue looping — more priority passing needed
                }
            }
        }
    }

    /// Perform state-based actions and put triggered abilities on stack (rule 117.5).
    ///
    /// 117.5 procedure:
    /// 1. Repeat SBAs until none are performed (704.3).
    /// 2. Put triggered abilities on the stack (603.3).
    /// 3. If any triggers were placed, go back to step 1.
    /// 4. Otherwise, the player who would receive priority does so.
    fn perform_sba_and_triggers(&mut self, decisions: &dyn DecisionProvider) -> Result<(), String> {
        loop {
            // Step 1: Exhaust all SBAs (rule 704.3)
            self.check_state_based_actions_loop(decisions)?;

            // Step 2: Place triggered abilities on the stack (rule 603.3)
            let triggers_placed = false; // Phase 7 stub

            // Step 3: If no triggers were placed, we're stable
            if !triggers_placed {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::{AbilityDef, AbilityType, CardDataBuilder};
    use crate::objects::object::GameObject;
    use crate::state::game_state::PhaseType;
    use crate::types::card_types::CardType;
    use crate::types::effects::{AmountExpr, Effect, Primitive, EffectRecipient, SelectionFilter, TargetCount};
    use crate::types::mana::{ManaCost, ManaType};
    use crate::ui::choice_types::ChoiceKind;
    use crate::ui::decision::ScriptedDecisionProvider;

    #[test]
    fn test_all_pass_empty_stack_ends_phase() {
        let mut game = GameState::new(2, 20);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        let decisions = ScriptedDecisionProvider::new();
        // Both players pass (index 0 = Pass)
        decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
        decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);

        let result = game.run_priority_round(&decisions).unwrap();
        assert_eq!(result, PriorityResult::PhaseEnds);
    }

    #[test]
    fn test_cast_and_resolve_via_priority() {
        let mut game = GameState::new(2, 20);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        // Give player 0 a bolt in hand and red mana
        let bolt_data = CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::build(&[ManaType::Red], 0))
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage(AmountExpr::Fixed(3)),
                    EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
                ),
            })
            .build();
        let obj = GameObject::new(bolt_data, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        game.players[0].mana_pool.add(ManaType::Red, 1);

        let decisions = ScriptedDecisionProvider::new();
        // Player 0 casts bolt (index 1 in [Pass, CastSpell(card_id)])
        decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
        // Target: Player(1) is at index 1 in [Player(0), Player(1)]
        decisions.expect_pick_n(ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
            spell_id: card_id,
        }, vec![1]);

        // First round: player 0 casts bolt (returns ActionTaken immediately)
        let result = game.run_priority_round(&decisions).unwrap();
        assert_eq!(result, PriorityResult::ActionTaken);
        assert!(game.stack.contains(&card_id));

        // Second round: both pass, stack resolves
        decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
        decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
        let result = game.run_priority_round(&decisions).unwrap();
        assert_eq!(result, PriorityResult::StackResolved);

        // Bolt resolved — player 1 lost 3 life
        assert_eq!(game.players[1].life_total, 17);

        // Third round: empty stack, both pass -> phase ends
        decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
        decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
        let result = game.run_priority_round(&decisions).unwrap();
        assert_eq!(result, PriorityResult::PhaseEnds);
    }

    #[test]
    fn test_run_priority_loop_no_actions() {
        let mut game = GameState::new(2, 20);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        let decisions = ScriptedDecisionProvider::new();
        // Both players pass — phase ends
        decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
        decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);

        game.run_priority_loop(&decisions).unwrap();
        // Should complete without error — phase ended
    }
}
