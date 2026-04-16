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

            let candidates = candidate_priority_actions(self, current_priority);
            let action = ask_choose_priority_action(decisions, self, current_priority, &candidates);

            match action {
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

                PriorityAction::CastSpell(card_id) => {
                    self.cast_spell(current_priority, card_id, decisions)?;
                    // Player who acted gets priority again (117.3c)
                    // Run SBAs before granting priority again
                    self.perform_sba_and_triggers(decisions)?;
                    return Ok(PriorityResult::ActionTaken);
                }

                PriorityAction::ActivateAbility(permanent_id, ability_id) => {
                    // Find the ability by ID and check its type
                    let card_data = self.get_object(permanent_id)?.card_data.clone();
                    let ability = card_data.abilities.iter()
                        .find(|a| a.id == ability_id)
                        .ok_or_else(|| format!("Ability {} not found on permanent {}", ability_id, permanent_id))?;

                    if ability.ability_type == crate::objects::card_data::AbilityType::Mana {
                        // Mana abilities resolve immediately (rule 605), no SBAs
                        self.activate_mana_ability(current_priority, permanent_id, ability_id)?;
                        return Ok(PriorityResult::ActionTaken);
                    }

                    let ability_index = card_data.abilities.iter()
                        .position(|a| a.id == ability_id)
                        .unwrap(); // safe: we just found it above
                    self.activate_ability(current_priority, permanent_id, ability_index, decisions)?;
                    self.perform_sba_and_triggers(decisions)?;
                    return Ok(PriorityResult::ActionTaken);
                }

                PriorityAction::PlayLand(card_id) => {
                    self.play_land(current_priority, card_id, Zone::Hand)?;
                    // Playing a land is a special action — player keeps priority (rule 116.2a)
                    // but we return so the caller can loop back
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
