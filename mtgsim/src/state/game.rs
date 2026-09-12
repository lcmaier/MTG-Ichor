use std::sync::Arc;

use crate::engine::actions::{ActionContext, ZoneChangeCause};
use crate::objects::card_data::CardData;
use crate::objects::object::GameObject;
use crate::state::game_config::GameConfig;
use crate::state::game_state::{GameResult, GameState, PhaseType, StepType};
use crate::types::zones::Zone;
use crate::ui::ask::ask_choose_discard;
use crate::ui::decision::DecisionProvider;

/// A decklist: ordered list of card definitions that make up a player's deck.
pub type Decklist = Vec<Arc<CardData>>;

/// Top-level game lifecycle wrapper.
///
/// `Game` owns the `GameState` and `GameConfig`. It is the entry point that
/// threads a `DecisionProvider` into the engine. Engine methods on `GameState`
/// (e.g. `cast_spell`, `run_priority_loop`) accept `&dyn DecisionProvider` as a
/// parameter for target selection, mana allocation, priority actions, etc.
/// `Game` is responsible for providing the provider to those calls and for
/// decision-requiring logic that lives outside the engine (e.g. cleanup
/// discard, mulligans).
///
/// **The result is not this wrapper's** — it is [`GameState::result`], written
/// where CR 104.1's "immediately" happens, inside the chokepoint. `Game` only
/// reads it, which is why nothing here can end a game that the engine has not.
pub struct Game {
    pub state: GameState,
    pub config: GameConfig,
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

        Ok(Game { state, config })
    }

    /// Make this game replayable from `seed` — same seed, same shuffle.
    ///
    /// Call before [`Game::setup`]; after it the libraries are already
    /// shuffled. The decision provider carries its own RNG, so a fully
    /// reproducible run seeds both (see `RandomDecisionProvider::seeded`).
    pub fn reseed(&mut self, seed: u64) {
        self.state.reseed(seed);
    }

    /// Take this game's randomness from the OS instead — for interactive play,
    /// where the same opening hand every session would be the bug.
    pub fn reseed_from_entropy(&mut self) {
        self.state.reseed_from_entropy();
    }

    /// Perform game setup: shuffle libraries and draw opening hands.
    ///
    /// Mulligan handling is stubbed — players always keep their first hand.
    /// Full London mulligan support requires multiple `DecisionProvider`
    /// calls per player and will be implemented when needed.
    pub fn setup(&mut self, decisions: &dyn DecisionProvider) -> Result<(), String> {
        // Shuffle each player's library
        for player_id in 0..self.state.num_players() {
            self.state.shuffle_library(player_id);
        }

        // Draw opening hands
        let hand_size = self.config.starting_hand_size;
        let num_players = self.state.num_players();
        // CR 103.4 calls this drawing, and it is the one draw in the engine
        // that calls the *performer* instead of proposing a `DrawCards`
        // instruction. Safe by construction rather than by exemption: the
        // battlefield is empty and the registry has no rows, so no replacement
        // can be gathered and no choice can arise, and Leylines (CR 103.6)
        // arrive after this point.
        let actx = ActionContext::new(decisions);
        for player_id in 0..num_players {
            for _ in 0..hand_size {
                self.state.draw_card(player_id, &actx)?;
            }
        }

        // TODO: mulligan decisions (London mulligan)
        // For each player in turn order:
        //   ask decisions.choose_mulligan(&self.state, player_id)
        //   if mulligan: shuffle hand into library, draw 7, bottom N

        // CR 103.7 — the first turn begins, and it begins the way every later
        // one does: a `BeginTurn` proposal, its beginning phase, its untap
        // step. Before RE-1 the first turn was a state `GameState::new` had
        // already written and its untap step ran no turn-based action at all,
        // because nothing called `on_step_begin` for the step the constructor
        // had parked on.
        self.state.start_first_turn(&actx)?;

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
            // Rule 508.8 is not read here: it refuses the step at the proposal
            // site (`GameState::begin_step`), so a declare-blockers or
            // combat-damage step with no attackers never begins and this loop
            // never sees one.
            //
            // Rule 514.3a: Cleanup normally doesn't grant priority, but if
            // SBAs are performed during cleanup, players get priority and then
            // a new cleanup step begins (re-remove damage, re-discard, re-check).
            let is_cleanup = matches!(
                (phase_type, step),
                (PhaseType::Ending, Some(StepType::Cleanup))
            );

            if is_cleanup {
                // Rule 514.3a: repeat while SBAs fire during cleanup
                while self.state.check_state_based_actions(decisions)? {
                    self.state.check_state_based_actions_loop(decisions)?;
                    self.state.run_priority_loop(decisions)?;

                    if self.is_over() {
                        return Ok(());
                    }

                    self.perform_cleanup_actions(decisions)?;
                }
            } else {
                let gets_priority = !matches!(
                    (phase_type, step),
                    (PhaseType::Beginning, Some(StepType::Untap))  // rule 502.3
                );

                if gets_priority {
                    self.state.run_priority_loop(decisions)?;

                    // 3. Game-over check after each priority round
                    if self.is_over() {
                        return Ok(());
                    }
                }
            }

            // 4. Advance to next step/phase
            self.state.advance_turn(&ActionContext::new(decisions))?;

            // **A skipped turn advances no turn number** (CR 614.10a), so this
            // is not "the number went up" — it is "the drainer produced a
            // turn". It still reads as `>` all the same: the number is
            // incremented by exactly the turns that begin, and the drainer does
            // not return until one has.
            if self.state.turn_number > starting_turn {
                return Ok(());
            }

            // ...with one board where it returns without producing one, and
            // that board is the reason this check is here rather than only
            // after a priority round. CR 104.4a: every player has left the
            // game, so `GameState::next_turn_taker` finds nobody to propose a
            // turn for and the position stays put. The untap step grants no
            // priority, so nothing else in this loop would notice, and it
            // would re-enter forever. The batch that performed those losses
            // settled the result, which is what this reads.
            if self.is_over() {
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
        // CR 800.4j — a turn whose active player has left "continues to its
        // completion without an active player". The two turn-based actions
        // *that player* performs — declaring attackers (508.1) and the
        // cleanup discard (514.1) — have nobody to perform them; the untap
        // step's and the draw step's are `engine::turns`'s and gate the same
        // way there. Their permanents staying to be untapped is RE-7's.
        let no_active_player = !self.state.in_game(self.state.active_player);
        match (phase_type, step) {
            // --- Combat phase ---
            (PhaseType::Combat, Some(StepType::DeclareAttackers)) if no_active_player => {}
            (PhaseType::Combat, Some(StepType::DeclareAttackers)) => {
                self.state.process_declare_attackers(decisions)?;
            }
            // No `attacks_declared` guard on these three: CR 508.8 refuses
            // the *step* when nothing attacked (`GameState::begin_step`), so
            // reaching them at all means attackers were declared.
            (PhaseType::Combat, Some(StepType::DeclareBlockers)) => {
                self.state.process_declare_blockers(decisions)?;
            }
            (PhaseType::Combat, Some(StepType::FirstStrikeDamage)) => {
                self.state.process_combat_damage(decisions, true)?;
            }
            (PhaseType::Combat, Some(StepType::CombatDamage)) => {
                self.state.process_combat_damage(decisions, false)?;
            }
            // --- Cleanup step ---
            (PhaseType::Ending, Some(StepType::Cleanup)) if no_active_player => {}
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
        self.result().ok_or_else(|| "Game ended without a result".to_string())
    }

    /// CR 104.1 — has the game ended? A read of [`GameState::result`].
    pub fn is_over(&self) -> bool {
        self.state.result.is_some()
    }

    /// The outcome, once there is one. A read of [`GameState::result`]: the
    /// engine records it at the batch that ended the game, and this wrapper
    /// derives nothing from the loss flags any more.
    pub fn result(&self) -> Option<GameResult> {
        self.state.result.clone()
    }

    /// Perform cleanup step actions: remove damage from all permanents,
    /// clear deathtouch flags, and discard to hand size (rules 514.1 + 514.2).
    /// Extracted for reuse in cleanup SBA re-loop (rule 514.3a).
    fn perform_cleanup_actions(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        // Rule 514.2: Remove all damage and end "until end of turn" effects
        for (_id, entry) in &mut self.state.battlefield {
            entry.damage_marked = 0;
            entry.damaged_by_deathtouch = false;
        }
        // Rule 514.1: Discard to hand size
        self.handle_cleanup_discard(decisions)?;
        Ok(())
    }

    /// Handle cleanup step discard to hand size (rule 514.1).
    fn handle_cleanup_discard(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        let active = self.state.active_player;
        let max = self.state.players[active].max_hand_size as usize;

        while self.state.players[active].hand.len() > max {
            let hand: Vec<_> = self.state.players[active].hand.clone();
            let card_id = ask_choose_discard(decisions, &self.state, active, &hand)
                .ok_or("Player must choose a card to discard")?;

            // Verify the chosen card is in hand
            if !self.state.players[active].hand.contains(&card_id) {
                return Err("Chosen card is not in hand".to_string());
            }

            // CR 514.1 cleanup discard — a turn-based action, no resolution.
            let actx = ActionContext::new(decisions);
            self.state.change_zone(card_id, Zone::Graveyard, ZoneChangeCause::Discarded, &actx)?;
        }

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::types::card_types::{CardType, Supertype, Subtype, LandType};
    use crate::types::mana::ManaType;
    use crate::ui::decision::ScriptedDecisionProvider;

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

        let decisions = ScriptedDecisionProvider::new();
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

    /// A loss proposed the way `Primitive::LoseGame` proposes one, in its own
    /// batch, so the settlement is the batch's (CR 104.2a / 104.4a).
    fn lose(game: &mut Game, player: usize) {
        use crate::engine::actions::GameAction;
        use crate::events::event::LossReason;
        let dp = ScriptedDecisionProvider::new();
        game.state
            .execute_action(
                GameAction::PlayerLoses { player, reason: LossReason::Effect },
                &ActionContext::new(&dp),
            )
            .unwrap();
    }

    #[test]
    fn test_no_losers_no_result() {
        let config = GameConfig::test();
        let game = Game::new(
            config,
            vec![make_test_decklist(20), make_test_decklist(20)],
        ).unwrap();

        assert!(game.result().is_none());
        assert!(!game.is_over());
    }

    // COVERS: ATOM-104.2a-001
    #[test]
    fn test_one_loser_and_the_other_player_wins() {
        let config = GameConfig::test();
        let mut game = Game::new(
            config,
            vec![make_test_decklist(20), make_test_decklist(20)],
        ).unwrap();

        lose(&mut game, 1);
        assert_eq!(game.result(), Some(GameResult::Winner(0)));
        assert!(game.is_over());
    }

    #[test]
    fn test_two_losses_in_two_batches_are_not_a_draw() {
        // CR 104.1 ended the game at the first batch; a loss performed after
        // it is the game continuing to be over, not a second result.
        let config = GameConfig::test();
        let mut game = Game::new(
            config,
            vec![make_test_decklist(20), make_test_decklist(20)],
        ).unwrap();

        lose(&mut game, 0);
        lose(&mut game, 1);
        assert_eq!(game.result(), Some(GameResult::Winner(1)));
    }

    #[test]
    fn test_cleanup_sba_reloop() {
        // Rule 514.3a: cleanup SBA re-loop path exercises without panic.
        // Poison SBA fires during first priority round, ending the game.
        let config = GameConfig::test();
        let mut game = Game::new(
            config,
            vec![make_test_decklist(20), make_test_decklist(20)],
        ).unwrap();

        let decisions = ScriptedDecisionProvider::new();
        game.setup(&decisions).unwrap();
        game.state.skip_first_draw = true; // avoid discard-to-hand-size noise

        // Turn completes normally with no SBAs during cleanup
        let starting_turn = game.state.turn_number;
        decisions.queue_empty_turn_passes();
        game.run_turn(&decisions).unwrap();
        assert_eq!(game.state.turn_number, starting_turn + 1);

        // Set poison to 10; the SBA check ahead of the upkeep's first priority
        // grant performs player 1's loss, and CR 104.1 ends the game there —
        // nobody is asked to pass in a game that has ended.
        game.state.players[1].poison_counters = 10;
        game.run_turn(&decisions).unwrap();
        assert!(game.is_over());
        assert_eq!(game.result(), Some(GameResult::Winner(0)));
    }

    #[test]
    fn test_run_single_turn() {
        let config = GameConfig::test();
        let mut game = Game::new(
            config,
            vec![make_test_decklist(20), make_test_decklist(20)],
        ).unwrap();

        let decisions = ScriptedDecisionProvider::new();
        game.setup(&decisions).unwrap();
        game.state.skip_first_draw = true; // avoid discard-to-hand-size noise

        let starting_turn = game.state.turn_number;
        decisions.queue_empty_turn_passes();
        game.run_turn(&decisions).unwrap();

        assert_eq!(game.state.turn_number, starting_turn + 1);
        assert!(!game.is_over());
    }
}
