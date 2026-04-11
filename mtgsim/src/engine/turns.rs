use crate::state::game_state::{GameState, Phase, PhaseType, StepType, next_step, next_phase};

/// Turn structure engine.
///
/// Handles advancing through phases and steps, processing phase/step-specific
/// actions (untap, draw, etc.), and turn transitions.

impl GameState {
    /// Advance the game state to the next step or phase.
    ///
    /// Returns the new (PhaseType, Option<StepType>) after advancing.
    pub fn advance_turn(&mut self) -> Result<(PhaseType, Option<StepType>), String> {
        // If we're in a phase with steps, try to advance to the next step
        if let Some(current_step) = self.phase.step {
            if let Some(next) = next_step(self.phase.phase_type, current_step) {
                // Execute end-of-step cleanup for the old step
                self.on_step_end(current_step)?;

                // Move to the next step within this phase
                self.phase.step = Some(next);
                self.on_step_begin(next)?;

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
            self.turn_number += 1;
            self.active_player = (self.active_player + 1) % self.num_players();
            self.priority_player = self.active_player;
        }

        self.phase = Phase::new(new_phase_type);
        self.on_phase_begin(new_phase_type)?;

        // If the new phase starts with a step, process that step's begin
        if let Some(step) = self.phase.step {
            self.on_step_begin(step)?;
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
        for player in &mut self.players {
            player.mana_pool.empty();
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

    fn on_step_begin(&mut self, step_type: StepType) -> Result<(), String> {
        match step_type {
            StepType::Untap => {
                self.process_untap_step()?;
            }
            StepType::Draw => {
                self.process_draw_step()?;
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
                // Rule 514.1: Discard to hand size — requires DecisionProvider (future)
                // TODO: wire up discard-to-hand-size once DecisionProvider is integrated

                // Rule 514.2: Remove all damage marked on permanents and end
                // "until end of turn" / "this turn" effects (simultaneous)
                for (_id, entry) in &mut self.battlefield {
                    entry.damage_marked = 0;
                    entry.damaged_by_deathtouch = false;
                }
                // Future: end "until end of turn" continuous effects here

                // Normally no priority during cleanup (rule 514.3)
                // Rule 514.3a: If SBAs would be performed or triggered abilities
                // are waiting, another cleanup step begins — handled in future phases
            }
        }
        Ok(())
    }

    fn on_step_end(&mut self, step_type: StepType) -> Result<(), String> {
        // Mana pools empty at end of each step (rule 106.4)
        for player in &mut self.players {
            player.mana_pool.empty();
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
    fn process_untap_step(&mut self) -> Result<(), String> {
        let active = self.active_player;

        // Reset land drops for the new turn
        let player = self.get_player_mut(active)?;
        player.reset_lands_played();

        // Untap permanents controlled by the active player
        for (_id, entry) in &mut self.battlefield {
            if entry.controller == active {
                entry.tapped = false;
            }
        }

        // No player gets priority during untap step
        Ok(())
    }

    /// Draw step: active player draws a card, then gets priority (rule 504)
    fn process_draw_step(&mut self) -> Result<(), String> {
        let active = self.active_player;

        // Rule 103.8a: first player skips the draw step of their first turn.
        // The skip_first_draw flag is set during Game::new() based on GameConfig.
        // This is a one-time flag — in-game "skip draw" effects use replacement
        // effects (Phase 6), not boolean flags.
        if self.skip_first_draw {
            self.skip_first_draw = false;
        } else {
            self.draw_card(active)?; // Ok(None) on empty library just flags SBA
        }

        self.priority_player = active;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::game_state::{GameState, PhaseType, StepType};
    use crate::types::card_types::*;
    use crate::types::mana::ManaType;

    /// Helper: give each player enough cards in library to not deck out during draw steps
    fn stock_libraries(game: &mut GameState, cards_per_player: usize) {
        let num_players = game.num_players();
        for pid in 0..num_players {
            for _ in 0..cards_per_player {
                let forest = CardDataBuilder::new("Forest")
                    .card_type(CardType::Land)
                    .supertype(Supertype::Basic)
                    .subtype(Subtype::Land(LandType::Forest))
                    .mana_ability_single(ManaType::Green)
                    .build();
                let obj = GameObject::in_library(forest, pid);
                let id = game.add_object(obj);
                game.players[pid].library.push(id);
            }
        }
    }

    #[test]
    fn test_advance_through_beginning_phase() {
        let mut game = GameState::new(2, 20);
        stock_libraries(&mut game, 5);

        // Starts at Beginning/Untap
        assert_eq!(game.phase.phase_type, PhaseType::Beginning);
        assert_eq!(game.phase.step, Some(StepType::Untap));

        // Advance: Untap -> Upkeep
        let (phase, step) = game.advance_turn().unwrap();
        assert_eq!(phase, PhaseType::Beginning);
        assert_eq!(step, Some(StepType::Upkeep));

        // Advance: Upkeep -> Draw
        let (phase, step) = game.advance_turn().unwrap();
        assert_eq!(phase, PhaseType::Beginning);
        assert_eq!(step, Some(StepType::Draw));

        // Player 0 should have drawn a card
        assert_eq!(game.players[0].hand.len(), 1);

        // Advance: Draw -> Precombat main (no step)
        let (phase, step) = game.advance_turn().unwrap();
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
            game.advance_turn().unwrap();
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

        // We're at Beginning/Untap already — process it by advancing to next step
        game.advance_turn().unwrap(); // Untap -> Upkeep (triggers untap processing for Upkeep's on_step_begin, but untap ran first)

        // Actually, untap processing happens when the untap step begins.
        // Since we started *at* untap, the on_step_begin already fired during game creation... 
        // Let's verify by cycling to the next turn's untap step
        // Go through the rest of the turn
        for _ in 0..12 {
            game.advance_turn().unwrap();
        }

        // Now we're at turn 2, player 1's untap step
        // Advance through player 1's full turn
        for _ in 0..13 {
            game.advance_turn().unwrap();
        }

        // Now at turn 3, player 0's untap step — this should untap the forest
        // The untap step processing runs on_step_begin when we enter it
        // We already entered it, so check:
        let entry = game.battlefield.get(&forest_id).unwrap();
        assert!(!entry.tapped, "Forest should be untapped after untap step");
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
            game.advance_turn().unwrap();
        }

        // Mana should have been emptied when we left the Beginning phase
        assert_eq!(game.players[0].mana_pool.total(), 0);
    }
}
