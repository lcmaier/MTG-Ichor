use std::sync::Arc;

use crate::objects::card_data::CardData;
use crate::objects::object::GameObject;
use crate::state::game_config::GameConfig;
use crate::state::game_state::{GameState, PhaseType, StepType};
use crate::types::ids::PlayerId;
use crate::types::zones::Zone;
use crate::ui::decision::DecisionProvider;

/// A decklist: ordered list of card definitions that make up a player's deck.
pub type Decklist = Vec<Arc<CardData>>;

/// The outcome of a completed game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameResult {
    Winner(PlayerId),
    Draw,
}

/// Top-level game lifecycle wrapper.
///
/// `Game` owns the `GameState`, `GameConfig`, and game result. It is the
/// entry point that threads a `DecisionProvider` into the engine. Engine
/// methods on `GameState` (e.g. `cast_spell`, `run_priority_loop`) accept
/// `&dyn DecisionProvider` as a parameter for target selection, mana
/// allocation, priority actions, etc. `Game` is responsible for providing
/// the provider to those calls and for decision-requiring logic that lives
/// outside the engine (e.g. cleanup discard, mulligans).
pub struct Game {
    pub state: GameState,
    pub config: GameConfig,
    pub result: Option<GameResult>,
}

impl Game {
    /// Return a snapshot of the event log for external inspection (e.g. fuzz harness).
    ///
    /// Delegates to `ui::display::format_event_log` for human-readable output
    /// with card names resolved from object IDs.
    pub fn event_log_snapshot(&self) -> Vec<String> {
        crate::ui::display::format_event_log(&self.state)
    }

    /// Create a new game from config and decklists.
    ///
    /// Builds a `GameState` with the configured starting life and populates
    /// each player's library from their decklist. Does NOT shuffle or draw
    /// opening hands — that happens in `setup()`.
    pub fn new(config: GameConfig, decklists: Vec<Decklist>) -> Result<Self, String> {
        let num_players = decklists.len();
        if num_players < 2 {
            return Err("Game requires at least 2 players".to_string());
        }

        let mut state = GameState::new(num_players, config.starting_life);

        // Populate libraries from decklists
        for (player_id, decklist) in decklists.into_iter().enumerate() {
            for card_data in decklist {
                let obj = GameObject::in_library(card_data, player_id);
                let id = obj.id;
                state.add_object(obj);
                state.players[player_id].library.push(id);
            }
        }

        // Set max hand size from config
        for player in &mut state.players {
            player.max_hand_size = config.max_hand_size;
        }

        // Set first-player draw skip flag
        if !config.first_player_draws {
            state.skip_first_draw = true;
        }

        Ok(Game {
            state,
            config,
            result: None,
        })
    }

    /// Perform game setup: shuffle libraries and draw opening hands.
    ///
    /// Mulligan handling is stubbed — players always keep their first hand.
    /// Full London mulligan support requires multiple `DecisionProvider`
    /// calls per player and will be implemented when needed.
    pub fn setup(&mut self, _decisions: &dyn DecisionProvider) -> Result<(), String> {
        // Shuffle each player's library
        for player in &mut self.state.players {
            Self::shuffle_library(&mut player.library);
        }

        // Draw opening hands
        let hand_size = self.config.starting_hand_size;
        let num_players = self.state.num_players();
        for player_id in 0..num_players {
            for _ in 0..hand_size {
                self.state.draw_card(player_id)?;
            }
        }

        // TODO: mulligan decisions (London mulligan)
        // For each player in turn order:
        //   ask decisions.choose_mulligan(&self.state, player_id)
        //   if mulligan: shuffle hand into library, draw 7, bottom N

        Ok(())
    }

    /// Run a single full turn for the current active player.
    ///
    /// Turn flow per step:
    /// 1. Turn-based actions (combat declarations, damage, cleanup discard)
    /// 2. Priority round (if the step grants priority)
    /// 3. Game-over check
    /// 4. Advance to next step/phase
    pub fn run_turn(&mut self, decisions: &dyn DecisionProvider) -> Result<(), String> {
        let starting_turn = self.state.turn_number;

        loop {
            if self.is_over() {
                return Ok(());
            }

            let phase_type = self.state.phase.phase_type;
            let step = self.state.phase.step;

            // 1. Turn-based actions for the current step
            self.process_turn_based_actions(phase_type, step, decisions)?;

            // 2. Priority round (most steps grant priority)
            //
            // Rule 508.8: if no creatures were declared as attackers, the
            // declare blockers and combat damage steps are skipped entirely
            // (no turn-based actions, no priority).
            let skipped_by_508_8 = !self.state.attacks_declared && matches!(
                (phase_type, step),
                (PhaseType::Combat, Some(StepType::DeclareBlockers))
                | (PhaseType::Combat, Some(StepType::FirstStrikeDamage))
                | (PhaseType::Combat, Some(StepType::CombatDamage))
            );

            let gets_priority = !skipped_by_508_8 && !matches!(
                (phase_type, step),
                (PhaseType::Beginning, Some(StepType::Untap))      // rule 502.3
                | (PhaseType::Ending, Some(StepType::Cleanup))     // rule 514.3
            );

            if gets_priority {
                self.state.run_priority_loop(decisions)?;

                // 3. Game-over check after each priority round
                if let Some(result) = self.check_game_over() {
                    self.result = Some(result);
                    return Ok(());
                }
            }

            // 4. Advance to next step/phase
            self.state.advance_turn()?;

            if self.state.turn_number > starting_turn {
                return Ok(());
            }
        }
    }

    /// Execute turn-based actions for the current step (rule 703.4).
    ///
    /// These happen BEFORE the priority round for each step:
    /// - Combat: declare attackers/blockers, deal damage
    /// - Cleanup: discard to hand size
    fn process_turn_based_actions(
        &mut self,
        phase_type: PhaseType,
        step: Option<StepType>,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        match (phase_type, step) {
            // --- Combat phase ---
            (PhaseType::Combat, Some(StepType::DeclareAttackers)) => {
                self.state.process_declare_attackers(decisions)?;
            }
            (PhaseType::Combat, Some(StepType::DeclareBlockers)) => {
                if self.state.attacks_declared {
                    self.state.process_declare_blockers(decisions)?;
                }
            }
            (PhaseType::Combat, Some(StepType::FirstStrikeDamage)) => {
                if self.state.attacks_declared {
                    self.state.process_combat_damage(decisions, true)?;
                }
            }
            (PhaseType::Combat, Some(StepType::CombatDamage)) => {
                if self.state.attacks_declared {
                    self.state.process_combat_damage(decisions, false)?;
                }
            }
            // --- Cleanup step ---
            (PhaseType::Ending, Some(StepType::Cleanup)) => {
                self.handle_cleanup_discard(decisions)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Run the complete game until a result is determined.
    ///
    /// Takes a single `DecisionProvider` that handles decisions for ALL
    /// players. Each trait method receives `player_id` as an argument, so
    /// the implementation can dispatch to the correct player (human UI,
    /// AI, network client, etc.) based on who is being asked.
    pub fn run(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<GameResult, String> {
        while !self.is_over() {
            self.run_turn(decisions)?;
        }
        self.result.clone().ok_or_else(|| "Game ended without a result".to_string())
    }

    pub fn is_over(&self) -> bool {
        self.result.is_some()
    }

    /// Check if the game should end based on player loss flags.
    ///
    /// Examines `GameState::player_lost` flags (set by SBAs) and determines
    /// the game result. In a 2-player game:
    /// - One player lost → other player wins
    /// - Both lost simultaneously → draw
    pub fn check_game_over(&self) -> Option<GameResult> {
        let losers: Vec<PlayerId> = self.state.player_lost.iter()
            .copied()
            .enumerate()
            .filter(|&(_, lost)| lost)
            .map(|(id, _)| id)
            .collect();

        if losers.is_empty() {
            return None;
        }

        let num_players = self.state.num_players();
        if losers.len() >= num_players {
            // All players lost simultaneously → draw
            return Some(GameResult::Draw);
        }

        if num_players == 2 {
            // Two-player game: the other player wins
            let winner = if losers[0] == 0 { 1 } else { 0 };
            return Some(GameResult::Winner(winner));
        }

        // Multiplayer: last player standing wins
        let survivors: Vec<PlayerId> = (0..num_players)
            .filter(|id| !self.state.player_lost[*id])
            .collect();
        if survivors.len() == 1 {
            return Some(GameResult::Winner(survivors[0]));
        }

        // Multiple survivors remain — game continues
        None
    }

    /// Handle cleanup step discard to hand size (rule 514.1).
    fn handle_cleanup_discard(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        let active = self.state.active_player;
        let max = self.state.players[active].max_hand_size as usize;

        while self.state.players[active].hand.len() > max {
            let card_id = decisions.choose_discard(&self.state, active)
                .ok_or("Player must choose a card to discard")?;

            // Verify the chosen card is actually in hand
            if !self.state.players[active].hand.contains(&card_id) {
                return Err("Chosen card is not in hand".to_string());
            }

            self.state.move_object(card_id, Zone::Graveyard)?;
        }

        Ok(())
    }

    /// Shuffle a library in place using rand.
    fn shuffle_library(library: &mut Vec<crate::types::ids::ObjectId>) {
        use rand::seq::SliceRandom;
        let rng = &mut rand::rng();
        library.shuffle(rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::types::card_types::{CardType, Supertype, Subtype, LandType};
    use crate::types::mana::ManaType;
    use crate::ui::decision::PassiveDecisionProvider;

    fn make_test_decklist(count: usize) -> Decklist {
        (0..count)
            .map(|_| {
                CardDataBuilder::new("Forest")
                    .card_type(CardType::Land)
                    .supertype(Supertype::Basic)
                    .subtype(Subtype::Land(LandType::Forest))
                    .mana_ability_single(ManaType::Green)
                    .build()
            })
            .collect()
    }

    #[test]
    fn test_game_creation() {
        let config = GameConfig::test();
        let game = Game::new(
            config,
            vec![make_test_decklist(20), make_test_decklist(20)],
        ).unwrap();

        assert_eq!(game.state.num_players(), 2);
        assert_eq!(game.state.players[0].life_total, 20);
        assert_eq!(game.state.players[1].life_total, 20);
        assert_eq!(game.state.players[0].library.len(), 20);
        assert_eq!(game.state.players[1].library.len(), 20);
        assert!(!game.is_over());
    }

    #[test]
    fn test_game_creation_too_few_players() {
        let config = GameConfig::test();
        assert!(Game::new(config, vec![make_test_decklist(20)]).is_err());
    }

    #[test]
    fn test_game_setup_draws_hands() {
        let config = GameConfig::test();
        let mut game = Game::new(
            config,
            vec![make_test_decklist(20), make_test_decklist(20)],
        ).unwrap();

        let decisions = PassiveDecisionProvider;
        game.setup(&decisions).unwrap();

        assert_eq!(game.state.players[0].hand.len(), 7);
        assert_eq!(game.state.players[1].hand.len(), 7);
        assert_eq!(game.state.players[0].library.len(), 13);
        assert_eq!(game.state.players[1].library.len(), 13);
    }

    #[test]
    fn test_standard_config_skips_first_draw() {
        let config = GameConfig::standard();
        let game = Game::new(
            config,
            vec![make_test_decklist(60), make_test_decklist(60)],
        ).unwrap();
        assert!(game.state.skip_first_draw);
    }

    #[test]
    fn test_check_game_over_no_losers() {
        let config = GameConfig::test();
        let game = Game::new(
            config,
            vec![make_test_decklist(20), make_test_decklist(20)],
        ).unwrap();

        assert!(game.check_game_over().is_none());
    }

    #[test]
    fn test_check_game_over_one_loser() {
        let config = GameConfig::test();
        let mut game = Game::new(
            config,
            vec![make_test_decklist(20), make_test_decklist(20)],
        ).unwrap();

        game.state.player_lost[1] = true;
        assert_eq!(game.check_game_over(), Some(GameResult::Winner(0)));
    }

    #[test]
    fn test_check_game_over_both_lose_is_draw() {
        let config = GameConfig::test();
        let mut game = Game::new(
            config,
            vec![make_test_decklist(20), make_test_decklist(20)],
        ).unwrap();

        game.state.player_lost[0] = true;
        game.state.player_lost[1] = true;
        assert_eq!(game.check_game_over(), Some(GameResult::Draw));
    }

    #[test]
    fn test_run_single_turn() {
        let config = GameConfig::test();
        let mut game = Game::new(
            config,
            vec![make_test_decklist(20), make_test_decklist(20)],
        ).unwrap();

        let decisions = PassiveDecisionProvider;
        game.setup(&decisions).unwrap();

        let starting_turn = game.state.turn_number;
        game.run_turn(&decisions).unwrap();

        assert_eq!(game.state.turn_number, starting_turn + 1);
        assert!(!game.is_over());
    }
}
