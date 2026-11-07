use std::sync::Arc;

use crate::engine::actions::ActionContext;
use crate::events::cause::GameRule;
use crate::objects::card_data::CardData;
use crate::objects::object::GameObject;
use crate::state::game_config::GameConfig;
use crate::state::game_state::{GameResult, GameState, PhaseType, StepType};
use crate::ui::decision::{DecisionProvider, Stop};

/// A player's deck, the cards CR 103.3 makes their library, in order. A
/// [`crate::scenario::Decklist`] is the names it is built from.
pub type Deck = Vec<Arc<CardData>>;

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

    /// Create a new game from config and decks.
    ///
    /// Builds a `GameState` with the configured starting life and populates
    /// each player's library from their deck. Does NOT shuffle or draw
    /// opening hands — that happens in `setup()`.
    pub fn new(config: GameConfig, decks: Vec<Deck>) -> Result<Self, String> {
        let num_players = decks.len();
        if num_players < 2 {
            return Err("Game requires at least 2 players".to_string());
        }

        let mut state = GameState::new(num_players, config.starting_life);

        for (player_id, deck) in decks.into_iter().enumerate() {
            for card_data in deck {
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


    /// `run` on this game, ended early by a [`Stop`] a provider raised inside
    /// it; any other panic passes through. A stopped game is the board at the
    /// prompt it stopped at, which reads and writes as that prompt showed it,
    /// and is never continued: the frames unwound held work in flight (a
    /// batch's decided members, a cast's remaining steps), so every run entry
    /// refuses it (`setup-architecture.md` §7.2, decision 1).
    pub fn until_stopped<T>(&mut self, run: impl FnOnce(&mut Game) -> Result<T, String>) -> Result<T, Halt> {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(self))) {
            Ok(ran) => ran.map_err(Halt::Error),
            Err(payload) => match payload.downcast::<Stop>() {
                Ok(stop) => {
                    self.state.stopped = true;
                    Err(Halt::Stopped(*stop))
                }
                Err(other) => std::panic::resume_unwind(other),
            },
        }
    }

    /// Perform game setup: shuffle libraries and draw opening hands.
    ///
    /// Mulligan handling is stubbed — players always keep their first hand
    /// (CR 103.5; `backlog.md` §2.32).
    pub fn setup(&mut self, dp: &dyn DecisionProvider) -> Result<(), String> {
        self.state.refuse_if_stopped()?;
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
        let actx = ActionContext::game_rule(dp, GameRule::OpeningHand);
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
        self.state.start_first_turn(dp)?;

        Ok(())
    }

    /// Run a single full turn for the current active player.
    ///
    /// Turn flow per step:
    /// 1. Turn-based actions (combat declarations, damage, cleanup discard)
    /// 2. Priority round (if the step grants priority)
    /// 3. Game-over check
    /// 4. Advance to next step/phase
    pub fn run_turn(&mut self, dp: &dyn DecisionProvider) -> Result<(), String> {
        let starting_turn = self.state.turn_number;
        self.run_turn_steps(dp, starting_turn, false)
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
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        let starting_turn = self.state.turn_number;
        self.run_turn_steps(dp, starting_turn, true)
    }

    /// Play a game built at the start of a priority round, as a scenario
    /// builds one (`setup-architecture.md` §1): finish this turn from that
    /// round, then run it to its end.
    pub fn resume(&mut self, dp: &dyn DecisionProvider) -> Result<GameResult, String> {
        self.resume_turn_at_priority(dp)?;
        self.run(dp)
    }

    /// The step drainer behind [`Game::run_turn`] and
    /// [`Game::resume_turn_at_priority`]. `starting_turn` is the turn number
    /// this call is finishing; `resuming` skips the current step's turn-based
    /// actions, for a caller that re-enters after them.
    fn run_turn_steps(
        &mut self,
        dp: &dyn DecisionProvider,
        starting_turn: u32,
        mut resuming: bool,
    ) -> Result<(), String> {
        self.state.refuse_if_stopped()?;
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
                self.process_turn_based_actions(phase_type, step, dp)?;
            }

            // 2. Priority round (most steps grant priority)
            //
            // CR 508.8 refuses steps with no attackers at the proposal
            // (`begin_step`), so this loop never sees one. CR 514.3a: cleanup grants
            // priority only if something happens in it, and then a new cleanup begins.
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
                loop {
                    let performed = self.state.check_state_based_actions(dp)?;
                    // A state trigger is waiting too once the check finds it
                    // true (CR 603.8), as "at the beginning of the next
                    // cleanup step" is once its step begins.
                    self.state.check_state_triggers()?;
                    if !performed && self.state.pending_triggers.is_empty() {
                        break;
                    }
                    self.state.check_state_based_actions_loop(dp)?;
                    self.state.run_priority_loop(dp)?;

                    if self.is_over() {
                        return Ok(());
                    }

                    // "Another cleanup step begins": proposed like the first, so
                    // CR 614.10 can skip it, and one that begins does what the
                    // first did (CR 514.2, 800.4c, and the discard with 800.4j's gate).
                    if !self.state.begin_step(StepType::Cleanup, dp)? {
                        break;
                    }
                }
            } else {
                let gets_priority = !matches!(
                    (phase_type, step),
                    (PhaseType::Beginning, Some(StepType::Untap))  // rule 502.3
                );

                if gets_priority {
                    self.state.run_priority_loop(dp)?;

                    // 3. Game-over check after each priority round
                    if self.is_over() {
                        return Ok(());
                    }
                }
            }

            // 4. Advance to next step/phase
            self.state.advance_turn(dp)?;

            // **A skipped turn advances no turn number** (CR 614.10a), so this
            // is not "the number went up" — it is "the drainer produced a
            // turn". It still reads as `>` all the same: the number is
            // incremented by exactly the turns that begin, and the drainer does
            // not return until one has.
            if self.state.turn_number > starting_turn {
                return Ok(());
            }

            // ...except when every player has left (CR 104.4a): `next_turn_taker`
            // finds nobody, and with no priority in the untap step nothing else
            // would notice, so the loop would re-enter forever. The losses' batch
            // settled the result this reads.
            if self.is_over() {
                return Ok(());
            }
        }
    }

    /// Execute turn-based actions for the current step (rule 703.4).
    ///
    /// These happen BEFORE the priority round for each step:
    /// - Combat: declare attackers/blockers, deal damage
    fn process_turn_based_actions(
        &mut self,
        phase_type: PhaseType,
        step: Option<StepType>,
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        // CR 800.4j — a turn whose active player has left "continues to its
        // completion without an active player". Declaring attackers (508.1)
        // has nobody to perform it; the untap, draw and cleanup steps' actions
        // are `engine::turns`'s and gate the same way there. Their permanents
        // have left with them (CR 800.4a).
        let no_active_player = !self.state.in_game(self.state.active_player);
        match (phase_type, step) {
            // --- Combat phase ---
            (PhaseType::Combat, Some(StepType::DeclareAttackers)) if no_active_player => {}
            (PhaseType::Combat, Some(StepType::DeclareAttackers)) => {
                self.state.process_declare_attackers(dp)?;
            }
            // No `combat_had_attackers` guard on these three: CR 508.8 refuses
            // the *step* when nothing attacked (`GameState::begin_step`), so
            // reaching them at all means attackers were declared.
            (PhaseType::Combat, Some(StepType::DeclareBlockers)) => {
                self.state.process_declare_blockers(dp)?;
            }
            (PhaseType::Combat, Some(StepType::FirstStrikeDamage)) => {
                self.state.process_combat_damage(dp, true)?;
            }
            (PhaseType::Combat, Some(StepType::CombatDamage)) => {
                self.state.process_combat_damage(dp, false)?;
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
        dp: &dyn DecisionProvider,
    ) -> Result<GameResult, String> {
        while !self.is_over() {
            self.run_turn(dp)?;
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

}

/// How a run [`Game::until_stopped`] watched ended without finishing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Halt {
    /// The run's own error, as it returns one.
    Error(String),
    /// A provider stopped it.
    Stopped(Stop),
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

    fn make_test_decklist(count: usize) -> Deck {
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

        let dp = ScriptedDecisionProvider::new();
        game.setup(&dp).unwrap();

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
        let dp = ScriptedDecisionProvider::new();
        game.setup(&dp).unwrap();
        (game, dp)
    }

    fn draw_steps_begun(game: &Game) -> usize {
        game.state
            .recorded_events()
            .events()
            .filter(|event| matches!(event, GameEvent::StepBegin { step: StepType::Draw, .. }))
            .count()
    }

    /// Plays the next turn, passing in its main phases: did `player` draw in it?
    fn drew_in_next_turn(game: &mut Game, dp: &ScriptedDecisionProvider, player: usize) -> bool {
        let library = game.state.players[player].library.len();
        dp.queue_main_phase_passes();
        game.run_turn(dp).unwrap();
        game.state.players[player].library.len() < library
    }

    // COVERS: ATOM-103.8a-001, ATOM-504.1-002
    #[test]
    fn test_two_seats_skip_the_first_draw() {
        for config in configs_without_an_override() {
            let (mut game, dp) = first_draw_game(config, 2);
            assert!(!drew_in_next_turn(&mut game, &dp, 0), "CR 103.8a: the starting player skips it");
            assert_eq!(draw_steps_begun(&game), 0, "CR 500.11: as though it didn't exist");
            assert!(!game.state.skip_first_draw);
            assert!(drew_in_next_turn(&mut game, &dp, 1));
            assert!(drew_in_next_turn(&mut game, &dp, 0), "their second turn draws");
            assert_eq!(draw_steps_begun(&game), 2);
        }
    }

    // COVERS: ATOM-103.8c-001
    #[test]
    fn test_three_and_four_seats_do_not_skip_it() {
        for seats in [3, 4] {
            for config in configs_without_an_override() {
                let (mut game, dp) = first_draw_game(config, seats);
                assert!(drew_in_next_turn(&mut game, &dp, 0), "CR 103.8c at {seats} seats");
            }
        }
    }

    #[test]
    fn test_the_test_configs_override_still_draws() {
        let (mut game, dp) = first_draw_game(GameConfig::test(), 2);
        assert!(drew_in_next_turn(&mut game, &dp, 0));
    }

    /// A loss proposed the way `Primitive::LoseGame` proposes one, in its own
    /// batch, so the settlement is the batch's (CR 104.2a / 104.4a).
    fn lose(game: &mut Game, player: usize) {
        use crate::engine::actions::GameAction;
        let dp = ScriptedDecisionProvider::new();
        game.state
            .execute_action(
                GameAction::PlayerLoses { player },
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

        let dp = ScriptedDecisionProvider::new();
        game.setup(&dp).unwrap();
        game.state.skip_first_draw = true; // avoid discard-to-hand-size noise

        // Turn completes normally with no SBAs during cleanup
        let starting_turn = game.state.turn_number;
        dp.queue_main_phase_passes();
        game.run_turn(&dp).unwrap();
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

        let dp = ScriptedDecisionProvider::new();
        game.setup(&dp).unwrap();
        game.state.skip_first_draw = true; // avoid discard-to-hand-size noise

        let starting_turn = game.state.turn_number;
        dp.queue_main_phase_passes();
        game.run_turn(&dp).unwrap();

        assert_eq!(game.state.turn_number, starting_turn + 1);
        assert!(!game.is_over());
    }
}
