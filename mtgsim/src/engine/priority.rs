use crate::state::game_state::GameState;
use crate::ui::decision::{DecisionProvider, PriorityAction};
use crate::types::zones::Zone;

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
        self.perform_sba_and_triggers()?;

        let num_players = self.num_players();
        let mut consecutive_passes = 0;
        let mut current_priority = self.active_player;

        loop {
            self.priority_player = current_priority;

            let action = decisions.choose_priority_action(self, current_priority);

            match action {
                PriorityAction::Pass => {
                    consecutive_passes += 1;
                    if consecutive_passes >= num_players {
                        // All players passed in succession (rule 117.4)
                        if self.stack.is_empty() {
                            return Ok(PriorityResult::PhaseEnds);
                        } else {
                            self.resolve_top_of_stack()?;
                            // After resolution, active player gets priority (117.3b)
                            // Run SBAs again before granting (117.5)
                            self.perform_sba_and_triggers()?;
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
                    self.perform_sba_and_triggers()?;
                    return Ok(PriorityResult::ActionTaken);
                }

                PriorityAction::ActivateAbility(permanent_id, ability_id) => {
                    // Find the ability index by ID
                    let card_data = self.get_object(permanent_id)?.card_data.clone();
                    let ability_index = card_data.abilities.iter()
                        .position(|a| a.id == ability_id)
                        .ok_or_else(|| format!("Ability {} not found on permanent {}", ability_id, permanent_id))?;
                    self.activate_ability(current_priority, permanent_id, ability_index, decisions)?;
                    self.perform_sba_and_triggers()?;
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
    /// Loops until no more SBAs are performed and no abilities trigger.
    /// Triggered abilities are stubbed for Phase 6.
    fn perform_sba_and_triggers(&mut self) -> Result<(), String> {
        loop {
            let sba_performed = self.check_state_based_actions()?;
            // Phase 6: put triggered abilities on the stack here
            let triggers_placed = false; // stub
            if !sba_performed && !triggers_placed {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::resolve::ResolvedTarget;
    use crate::objects::card_data::{AbilityDef, AbilityType, CardDataBuilder};
    use crate::objects::object::GameObject;
    use crate::state::game_state::PhaseType;
    use crate::types::card_types::CardType;
    use crate::types::effects::{AmountExpr, Effect, Primitive, TargetSpec, TargetCount};
    use crate::types::mana::{ManaCost, ManaType};
    use crate::ui::decision::ScriptedDecisionProvider;

    #[test]
    fn test_all_pass_empty_stack_ends_phase() {
        let mut game = GameState::new(2, 20);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        let decisions = ScriptedDecisionProvider::new();
        // Both players will pass (default ScriptedDecisionProvider behavior)

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
            .mana_cost(ManaCost::single(ManaType::Red, 1, 0))
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage(AmountExpr::Fixed(3)),
                    TargetSpec::Player(TargetCount::Exactly(1)),
                ),
            })
            .build();
        let obj = GameObject::new(bolt_data, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        game.players[0].mana_pool.add(ManaType::Red, 1);

        let decisions = ScriptedDecisionProvider::new();
        // Player 0 casts bolt, then both pass to let it resolve
        decisions.priority_decisions.borrow_mut().push(
            PriorityAction::CastSpell(card_id),
        );
        decisions.target_decisions.borrow_mut().push(
            vec![ResolvedTarget::Player(1)],
        );

        // First round: player 0 casts bolt
        let result = game.run_priority_round(&decisions).unwrap();
        assert_eq!(result, PriorityResult::ActionTaken);
        assert!(game.stack.contains(&card_id));

        // Second round: both pass (default), stack resolves
        let result = game.run_priority_round(&decisions).unwrap();
        assert_eq!(result, PriorityResult::StackResolved);

        // Bolt resolved — player 1 lost 3 life
        assert_eq!(game.players[1].life_total, 17);

        // Third round: empty stack, both pass -> phase ends
        let result = game.run_priority_round(&decisions).unwrap();
        assert_eq!(result, PriorityResult::PhaseEnds);
    }

    #[test]
    fn test_run_priority_loop_no_actions() {
        let mut game = GameState::new(2, 20);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        let decisions = ScriptedDecisionProvider::new();

        game.run_priority_loop(&decisions).unwrap();
        // Should complete without error — phase ended
    }
}
