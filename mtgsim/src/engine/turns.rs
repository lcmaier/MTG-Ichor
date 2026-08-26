use crate::engine::actions::{ActionContext, GameAction};
use crate::state::game_state::{GameState, Phase, PhaseType, StepType, next_step, next_phase};
use crate::types::ids::ObjectId;
use crate::types::mana::{ManaEmptyReason, BlanketPersistenceSet};

/// Turn structure engine.
///
/// Handles advancing through phases and steps, processing phase/step-specific
/// actions (untap, draw, etc.), and turn transitions.

impl GameState {
    /// Advance the game state to the next step or phase.
    ///
    /// Returns the new (PhaseType, Option<StepType>) after advancing.
    pub fn advance_turn(
        &mut self,
        ctx: &ActionContext,
    ) -> Result<(PhaseType, Option<StepType>), String> {
        // If we're in a phase with steps, try to advance to the next step
        if let Some(current_step) = self.phase.step {
            if let Some(next) = next_step(self.phase.phase_type, current_step) {
                // Execute end-of-step cleanup for the old step
                self.on_step_end(current_step)?;

                // Move to the next step within this phase
                self.phase.step = Some(next);
                self.on_step_begin(next, ctx)?;

                return Ok((self.phase.phase_type, self.phase.step));
            }
            // No more steps in this phase — fall through to advance phase
            self.on_step_end(current_step)?;
        }

        // Advance to the next phase
        let old_phase = self.phase.phase_type;
        self.on_phase_end(old_phase)?;

        let new_phase_type = next_phase(old_phase);

        // Check for turn transition (Ending -> Beginning = new turn)
        if old_phase == PhaseType::Ending && new_phase_type == PhaseType::Beginning {
            self.on_turn_end()?;
            let next_player = (self.active_player + 1) % self.num_players();
            self.begin_turn(self.turn_number + 1, next_player);
            self.priority_player = self.active_player;
        }

        self.phase = Phase::new(new_phase_type);
        self.on_phase_begin(new_phase_type)?;

        // If the new phase starts with a step, process that step's begin
        if let Some(step) = self.phase.step {
            self.on_step_begin(step, ctx)?;
        }

        Ok((self.phase.phase_type, self.phase.step))
    }

    // --- Phase lifecycle callbacks ---

    fn on_phase_begin(&mut self, _phase_type: PhaseType) -> Result<(), String> {
        // Future: emit PhaseBegin events for triggered abilities
        Ok(())
    }

    fn on_phase_end(&mut self, phase_type: PhaseType) -> Result<(), String> {
        // Mana pools empty at end of each phase (rule 106.4)
        // TODO(T12c): build BlanketPersistenceSet from continuous effects layer
        let blanket = BlanketPersistenceSet::none();
        for player in &mut self.players {
            player.mana_pool.empty_with_reason(ManaEmptyReason::StepOrPhase, &blanket);
        }

        // Phase-specific cleanup
        match phase_type {
            PhaseType::Combat => {
                // Clear combat state from all permanents
                for (_id, entry) in &mut self.battlefield {
                    entry.clear_combat_state();
                }
                self.attacks_declared = false;
                self.blockers_declared = false;
                self.blocker_damage_divisions.clear();
                self.dealt_first_strike_damage.clear();
            }
            _ => {}
        }

        Ok(())
    }

    // --- Step lifecycle callbacks ---

    fn on_step_begin(&mut self, step_type: StepType, ctx: &ActionContext) -> Result<(), String> {
        match step_type {
            StepType::Untap => {
                // Expire "until your next turn" effects for the active player
                self.continuous_effects.remove_expired_at_turn_start(
                    self.active_player,
                    self.turn_number,
                );
                self.process_untap_step(ctx)?;
            }
            StepType::Draw => {
                self.process_draw_step(ctx)?;
            }
            StepType::Upkeep
            | StepType::BeginCombat
            | StepType::DeclareAttackers
            | StepType::DeclareBlockers
            | StepType::FirstStrikeDamage
            | StepType::CombatDamage
            | StepType::EndCombat
            | StepType::End => {
                // Active player gets priority
                self.priority_player = self.active_player;
            }
            StepType::Cleanup => {
                // Rule 514.1 (discard to hand size) needs a DecisionProvider, so it
                // lives one level up in `Game::perform_cleanup_actions`, not here.

                // Rule 514.2: Remove all damage marked on permanents and end
                // "until end of turn" / "this turn" effects (simultaneous)
                for (_id, entry) in &mut self.battlefield {
                    entry.damage_marked = 0;
                    entry.damaged_by_deathtouch = false;
                }

                // Rule 514.2: End "until end of turn" continuous effects
                self.continuous_effects.remove_expired_at_cleanup(
                    self.active_player,
                    self.turn_number,
                );

                // Normally no priority during cleanup (rule 514.3)
                // Rule 514.3a: If SBAs would be performed or triggered abilities
                // are waiting, another cleanup step begins — handled in future phases
            }
        }
        Ok(())
    }

    fn on_step_end(&mut self, step_type: StepType) -> Result<(), String> {
        // Mana pools empty at end of each step (rule 106.4)
        // TODO(T12c): build BlanketPersistenceSet from continuous effects layer
        let blanket = BlanketPersistenceSet::none();
        for player in &mut self.players {
            player.mana_pool.empty_with_reason(ManaEmptyReason::StepOrPhase, &blanket);
        }

        match step_type {
            _ => {} // Future: step-specific cleanup
        }
        Ok(())
    }

    fn on_turn_end(&mut self) -> Result<(), String> {
        // Per-turn resets (land drops, etc.) happen in process_untap_step,
        // which is the canonical location per rules (rule 502).
        Ok(())
    }

    // --- Step processors ---

    /// Untap step: untap all permanents controlled by the active player,
    /// reset land drops (rule 502)
    fn process_untap_step(&mut self, ctx: &ActionContext) -> Result<(), String> {
        let active = self.active_player;

        // Reset land drops for the new turn
        let player = self.get_player_mut(active)?;
        player.reset_lands_played();

        // Untap permanents the active player *effectively* controls (CR 502.1).
        //
        // Two passes because the predicate is a `&self` layer query and the
        // untap is a `&mut self` write.
        //
        // **Ordered, and that is not cosmetic.** This sweep used to iterate
        // `battlefield.keys()` under a comment saying it reached no decision.
        // True while each untap was a direct write; false now that each is a
        // replaceable `GameAction::Untap`. CR 616.1 prompts the affected
        // permanent's controller when two effects want one untap (stun counters,
        // CR 122.1d), so the order the proposals are made in is observable and
        // `HashMap` order differs per process.
        let to_untap: Vec<ObjectId> = self
            .battlefield_ids_ordered()
            .into_iter()
            .filter(|&id| crate::oracle::characteristics::controls(self, id, active))
            .collect();
        for id in to_untap {
            self.execute_action(GameAction::Untap { object: id }, ctx)?;
        }

        // No player gets priority during untap step
        Ok(())
    }

    /// Draw step: active player draws a card, then gets priority (rule 504)
    fn process_draw_step(&mut self, ctx: &ActionContext) -> Result<(), String> {
        let active = self.active_player;

        // Rule 103.8a: first player skips the draw step of their first turn.
        // The skip_first_draw flag is set during Game::new() based on GameConfig.
        // This is a one-time flag — in-game "skip draw" effects use replacement
        // effects (Phase 6), not boolean flags.
        if self.skip_first_draw {
            self.skip_first_draw = false;
        } else {
            // Through the chokepoint, not straight to `draw_card`: CR 614.11
            // draw replacements and CR 614.10 skips both act on the *proposal*,
            // and the turn-based action is where the proposal is born.
            self.execute_action(GameAction::DrawCard { player: active }, ctx)?;
        }

        self.priority_player = active;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use crate::test_support::test_ctx;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::game_state::{GameState, PhaseType, StepType};
    use crate::types::card_types::*;
    use crate::types::mana::ManaType;
    use crate::test_support::stock_libraries;

    #[test]
    fn test_advance_through_beginning_phase() {
        let mut game = GameState::new(2, 20);
        stock_libraries(&mut game, 5);

        // Starts at Beginning/Untap
        assert_eq!(game.phase.phase_type, PhaseType::Beginning);
        assert_eq!(game.phase.step, Some(StepType::Untap));

        // Advance: Untap -> Upkeep
        let (phase, step) = game.advance_turn(&test_ctx()).unwrap();
        assert_eq!(phase, PhaseType::Beginning);
        assert_eq!(step, Some(StepType::Upkeep));

        // Advance: Upkeep -> Draw
        let (phase, step) = game.advance_turn(&test_ctx()).unwrap();
        assert_eq!(phase, PhaseType::Beginning);
        assert_eq!(step, Some(StepType::Draw));

        // Player 0 should have drawn a card
        assert_eq!(game.players[0].hand.len(), 1);

        // Advance: Draw -> Precombat main (no step)
        let (phase, step) = game.advance_turn(&test_ctx()).unwrap();
        assert_eq!(phase, PhaseType::Precombat);
        assert_eq!(step, None);
    }

    #[test]
    fn test_full_turn_cycle() {
        let mut game = GameState::new(2, 20);
        stock_libraries(&mut game, 10);

        assert_eq!(game.turn_number, 1);
        assert_eq!(game.active_player, 0);

        // Advance through all phases/steps of turn 1
        // Beginning: Untap, Upkeep, Draw = 3 advances
        // Precombat: 1 advance (no steps)
        // Combat: BeginCombat, DeclareAttackers, DeclareBlockers, FirstStrikeDamage, CombatDamage, EndCombat = 6 advances
        // Postcombat: 1 advance (no steps)
        // Ending: End, Cleanup = 2 advances
        // Total: 13 advances to complete one turn

        for _ in 0..13 {
            game.advance_turn(&test_ctx()).unwrap();
        }

        assert_eq!(game.turn_number, 2);
        assert_eq!(game.active_player, 1);
        assert_eq!(game.phase.phase_type, PhaseType::Beginning);
        assert_eq!(game.phase.step, Some(StepType::Untap));
    }

    #[test]
    fn test_untap_step_clears_tapped() {
        let mut game = GameState::new(2, 20);
        stock_libraries(&mut game, 5);

        // Put a tapped permanent on the battlefield for player 0
        let forest_data = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .mana_ability_single(ManaType::Green)
            .build();
        let forest = GameObject::new(forest_data, 0, crate::types::zones::Zone::Battlefield);
        let forest_id = game.add_object(forest);
        game.place_on_battlefield(forest_id, 0).tapped = true;

        // Advance past turn 1 (on_step_begin already fired for current untap)
        game.advance_turn(&test_ctx()).unwrap();
        for _ in 0..12 {
            game.advance_turn(&test_ctx()).unwrap();
        }

        // Advance through player 1's full turn
        for _ in 0..13 {
            game.advance_turn(&test_ctx()).unwrap();
        }

        // Turn 3, player 0's untap step — forest should be untapped
        let entry = game.battlefield.get(&forest_id).unwrap();
        assert!(!entry.tapped, "Forest should be untapped after untap step");
    }

    #[test]
    fn test_untap_step_announces_only_the_permanents_it_actually_untapped() {
        let mut game = GameState::new(2, 20);
        stock_libraries(&mut game, 5);

        let land = |name: &str| CardDataBuilder::new(name)
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .mana_ability_single(ManaType::Green)
            .build();

        // One tapped, one already untapped, both controlled by player 0.
        let tapped_id = game.add_object(GameObject::new(
            land("Tapped Forest"), 0, crate::types::zones::Zone::Battlefield));
        game.place_on_battlefield(tapped_id, 0).tapped = true;
        let untapped_id = game.add_object(GameObject::new(
            land("Untapped Forest"), 0, crate::types::zones::Zone::Battlefield));
        game.place_on_battlefield(untapped_id, 0).tapped = false;

        let before = game.events.len();
        // Walk to player 0's next untap step.
        for _ in 0..26 {
            game.advance_turn(&test_ctx()).unwrap();
        }

        let untapped: Vec<crate::types::ids::ObjectId> = game.events.events()[before..].iter()
            .filter_map(|e| match e {
                crate::events::event::GameEvent::Untapped { object_id } => Some(*object_id),
                _ => None,
            })
            .collect();

        // CR 502.1 untaps every permanent the active player controls, but
        // CR 603.2e only *announces* the ones that changed state. A sweep that
        // emitted per-permanent rather than per-transition would report both.
        assert_eq!(untapped, vec![tapped_id]);
        assert!(!game.battlefield.get(&tapped_id).unwrap().tapped);
    }

    #[test]
    fn test_mana_empties_at_phase_end() {
        let mut game = GameState::new(2, 20);
        stock_libraries(&mut game, 5);

        // Add some mana to player 0
        game.players[0].mana_pool.add(ManaType::Green, 3);
        assert_eq!(game.players[0].mana_pool.total(), 3);

        // Advance through Beginning phase (3 steps) to Precombat main
        for _ in 0..3 {
            game.advance_turn(&test_ctx()).unwrap();
        }

        // Mana should have been emptied when we left the Beginning phase
        assert_eq!(game.players[0].mana_pool.total(), 0);
    }
}
