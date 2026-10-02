use std::sync::Arc;

use crate::engine::actions::{ActionContext, ZoneChangeCause};
use crate::objects::card_data::CardData;
use crate::objects::object::GameObject;
use crate::state::game_config::GameConfig;
use crate::state::game_state::{GameResult, GameState, PhaseType, StepType};
use crate::types::zones::Zone;
use crate::engine::actions::GameAction;
use crate::ui::ask::ask_discard;
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

        for (player_id, decklist) in decklists.into_iter().enumerate() {
            for card_data in decklist {
                // Through the door that registers what functions in a library
                // (CR 113.6), so a card whose static ability works "from
                // anywhere" is a gather candidate from the first mill.
                state.create_in_zone(GameObject::in_library(card_data, player_id))?;
            }
        }

        for player in &mut state.players {
            player.max_hand_size = config.max_hand_size;
        }

        state.skip_first_draw = match config.first_player_draws {
            Some(draws) => !draws,
            None => starting_player_skips_first_draw(&state),
        };

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
    /// Mulligan handling is stubbed — players always keep their first hand
    /// (CR 103.5; `backlog.md` §2.32).
    pub fn setup(&mut self, decisions: &dyn DecisionProvider) -> Result<(), String> {
        for player_id in 0..self.state.num_players() {
            self.state.shuffle_library(player_id);
        }

        let hand_size = self.config.starting_hand_size;
        let num_players = self.state.num_players();
        // CR 103.5 calls this drawing, and it is the one draw in the engine
        // that calls the *performer* instead of proposing a `DrawCards`
        // instruction. Safe by construction rather than by exemption: the
        // battlefield is empty and the registry has no rows, so no replacement
        // can be gathered and no choice can arise, and Leylines (CR 103.6)
        // arrive after this point.
        let actx = ActionContext::new(decisions);
        // No turn has begun while the game is set up (CR 103), so the opening
        // hands are drawn in turn 0, which has no history row.
        self.state.turn_number = 0;
        for player_id in 0..num_players {
            for _ in 0..hand_size {
                self.state.draw_card(player_id, &actx)?;
            }
        }

        // CR 103.5's mulligans are not asked: every player keeps their first
        // hand, whatever `GameConfig::mulligan_rule` says. `backlog.md` §2.32.

        // CR 103.8 — the first turn begins, and it begins the way every later
        // one does: a `BeginTurn` proposal, its beginning phase, its untap step,
        // so the first untap step runs its turn-based action like any other.
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
        self.run_turn_steps(decisions, starting_turn, false)
    }

    /// Finish the turn from a priority prompt a fork was taken at.
    ///
    /// The entry point `codebase-state.md` item 41's test needs and item 140
    /// extends: a clone taken at a priority prompt is resumed by re-entering
    /// the step's priority loop, **not** by re-running the step from the top —
    /// its turn-based actions (CR 703.4) have already happened, and performing
    /// them twice is a different game. Everything the resumed loop reads is on
    /// `GameState`, which is what makes the clone a complete description of the
    /// game at that prompt.
    ///
    /// **Round starts only, and not the cleanup step's.** `run_priority_round`
    /// begins every round at the active player, and a re-ask's rejection lives
    /// only until the next ask, so a clone taken anywhere else in a round
    /// resumes as a different round; CR 514.3a's re-loop lives in this method's
    /// own cleanup branch and is not re-enterable at all. Item 140 owns both.
    pub fn resume_turn_at_priority(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        let starting_turn = self.state.turn_number;
        self.run_turn_steps(decisions, starting_turn, true)
    }

    /// Play a game built at the start of a priority round, as a scenario
    /// builds one (`setup-architecture.md` §1): finish this turn from that
    /// round, then run it to its end.
    pub fn resume(&mut self, decisions: &dyn DecisionProvider) -> Result<GameResult, String> {
        self.resume_turn_at_priority(decisions)?;
        self.run(decisions)
    }

    /// The step drainer behind [`Game::run_turn`] and
    /// [`Game::resume_turn_at_priority`]. `starting_turn` is the turn number
    /// this call is finishing; `resuming` skips the current step's turn-based
    /// actions, for a caller that re-enters after them.
    fn run_turn_steps(
        &mut self,
        decisions: &dyn DecisionProvider,
        starting_turn: u32,
        mut resuming: bool,
    ) -> Result<(), String> {
        loop {
            if self.is_over() {
                return Ok(());
            }

            let phase_type = self.state.phase.phase_type;
            let step = self.state.phase.step;

            // 1. Turn-based actions for the current step
            if resuming {
                resuming = false;
            } else {
                self.process_turn_based_actions(phase_type, step, decisions)?;
            }

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
                // Rule 514.3a: repeat while SBAs fire during cleanup — or
                // while triggered abilities are waiting, which the rule names
                // in the same breath ("and/or any triggered abilities are
                // waiting to be put onto the stack"). A trigger from the
                // step's own discard or damage removal is the reachable case.
                while self.state.check_state_based_actions(decisions)?
                    || !self.state.pending_triggers.is_empty()
                {
                    self.state.check_state_based_actions_loop(decisions)?;
                    self.state.run_priority_loop(decisions)?;

                    if self.is_over() {
                        return Ok(());
                    }

                    // "another cleanup step begins" — a second occurrence of
                    // the step, proposed like the first: CR 614.10's skips
                    // are per occurrence, so this one is skippable too, and a
                    // refused one runs no turn-based action. One that begins
                    // does what the first did: `begin_step` runs CR 514.2 and
                    // CR 800.4c, and the discard is the first's, CR 800.4j's
                    // gate included.
                    let actx = ActionContext::new(decisions);
                    if !self.state.begin_step(StepType::Cleanup, &actx)? {
                        break;
                    }
                    self.process_turn_based_actions(PhaseType::Ending, Some(StepType::Cleanup), decisions)?;
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
        // way there. Their permanents have left with them (CR 800.4a).
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

    /// The outcome, once there is one. A read of [`GameState::result`], which
    /// the engine records at the batch that ended the game.
    pub fn result(&self) -> Option<GameResult> {
        self.state.result.clone()
    }

    /// Handle cleanup step discard to hand size (rule 514.1).
    ///
    /// > 514.1. First, if the active player's hand contains more cards than
    /// > their maximum hand size (normally seven), they discard **enough
    /// > cards** to reduce their hand size to that number.
    ///
    /// **One discard of N, not N discards of one.** The rule names a single
    /// turn-based action over "enough cards", so the choice is made once and
    /// the cards move as one batch — which is what CR 603.2c's "whenever one
    /// or more cards are discarded" will read, and the same shape
    /// `Primitive::Discard` builds for Mind Rot.
    fn handle_cleanup_discard(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        let active = self.state.active_player;
        let max = self.state.players[active].max_hand_size as usize;
        let hand: Vec<_> = self.state.players[active].hand.clone();
        if hand.len() <= max {
            return Ok(());
        }
        // No `source`: CR 514.1 is a turn-based action and no spell or ability
        // caused it, which is also why a `ReplacementDef::by` never matches it.
        let chosen = ask_discard(decisions, &self.state, active, &hand, hand.len() - max, None);
        let batch: Vec<GameAction> = chosen
            .into_iter()
            .map(|object| GameAction::ZoneChange {
                object,
                from: Zone::Hand,
                to: Zone::Graveyard,
                cause: ZoneChangeCause::Discarded,
            })
            .collect();
        let actx = ActionContext::new(decisions);
        self.state.execute_actions(batch, &actx)?;
        Ok(())
    }

}

/// CR 103.8a and 103.8c — whether the starting player skips the draw step of
/// their first turn: in a two-player game they do, and in a game that begins
/// with more than two players (CR 800.1) nobody does.
pub fn starting_player_skips_first_draw(state: &GameState) -> bool {
    !state.is_multiplayer()
}

/// The two random streams a game draws from besides its decks, derived from
/// one seed (`setup-architecture.md` §6): the game's, which `GameState::rng`
/// shuffles from, and the agents'. Distinct sub-seeds, since two `StdRng`s
/// seeded alike would correlate a shuffle with the choices made over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomStreams {
    pub game: u64,
    pub agents: u64,
}

impl RandomStreams {
    pub fn from_seed(seed: u64) -> RandomStreams {
        RandomStreams { game: seed ^ 0x9E37_79B9_7F4A_7C15, agents: seed ^ 0xD1B5_4A32_D192_ED03 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event::GameEvent;
    use crate::objects::card_data::CardDataBuilder;
    use crate::types::card_types::{CardType, Supertype, Subtype, LandType};
    use crate::types::mana::ManaType;
    use crate::ui::decision::{ScriptedDecisionProvider, SeatMode};

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

    /// Every constructor but `test`'s, each leaving CR 103.8 to the seat count.
    fn configs_without_an_override() -> [GameConfig; 3] {
        [GameConfig::standard(), GameConfig::limited(), GameConfig::unrestricted()]
    }

    /// A game of Forests at `seats` seats, set up. An eighth card fits in hand,
    /// so cleanup asks no discard whichever answer CR 103.8 gives.
    fn first_draw_game(mut config: GameConfig, seats: usize) -> (Game, ScriptedDecisionProvider) {
        config.max_hand_size = 8;
        let mut game = Game::new(config, vec![make_test_decklist(20); seats]).unwrap();
        game.state.record_events();
        let decisions = ScriptedDecisionProvider::new();
        game.setup(&decisions).unwrap();
        (game, decisions)
    }

    fn draw_steps_begun(game: &Game) -> usize {
        game.state
            .recorded_events()
            .events()
            .filter(|event| matches!(event, GameEvent::StepBegin { step: StepType::Draw, .. }))
            .count()
    }

    /// Plays the next turn, passing in its main phases: did `player` draw in it?
    fn drew_in_next_turn(game: &mut Game, decisions: &ScriptedDecisionProvider, player: usize) -> bool {
        let library = game.state.players[player].library.len();
        decisions.queue_main_phase_passes();
        game.run_turn(decisions).unwrap();
        game.state.players[player].library.len() < library
    }

    // COVERS: ATOM-103.8a-001, ATOM-504.1-002
    #[test]
    fn test_two_seats_skip_the_first_draw() {
        for config in configs_without_an_override() {
            let (mut game, decisions) = first_draw_game(config, 2);
            assert!(!drew_in_next_turn(&mut game, &decisions, 0), "CR 103.8a: the starting player skips it");
            assert_eq!(draw_steps_begun(&game), 0, "CR 500.11: as though it didn't exist");
            assert!(!game.state.skip_first_draw);
            assert!(drew_in_next_turn(&mut game, &decisions, 1));
            assert!(drew_in_next_turn(&mut game, &decisions, 0), "their second turn draws");
            assert_eq!(draw_steps_begun(&game), 2);
        }
    }

    // COVERS: ATOM-103.8c-001
    #[test]
    fn test_three_and_four_seats_do_not_skip_it() {
        for seats in [3, 4] {
            for config in configs_without_an_override() {
                let (mut game, decisions) = first_draw_game(config, seats);
                assert!(drew_in_next_turn(&mut game, &decisions, 0), "CR 103.8c at {seats} seats");
            }
        }
    }

    #[test]
    fn test_the_test_configs_override_still_draws() {
        let (mut game, decisions) = first_draw_game(GameConfig::test(), 2);
        assert!(drew_in_next_turn(&mut game, &decisions, 0));
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
        decisions.queue_main_phase_passes();
        game.run_turn(&decisions).unwrap();
        assert_eq!(game.state.turn_number, starting_turn + 1);

        // Set poison to 10; the SBA check ahead of the upkeep's first priority
        // grant performs player 1's loss, and CR 104.1 ends the game there —
        // nobody is asked to pass in a game that has ended, though both seats
        // now stop at every priority point.
        game.state.players[1].add_counters(crate::types::effects::CounterType::Poison, 10);
        let after_loss = ScriptedDecisionProvider::new()
            .with_seat_mode(SeatMode { stops_at_every_priority_point: true });
        game.run_turn(&after_loss).unwrap();
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
        decisions.queue_main_phase_passes();
        game.run_turn(&decisions).unwrap();

        assert_eq!(game.state.turn_number, starting_turn + 1);
        assert!(!game.is_over());
    }
}
