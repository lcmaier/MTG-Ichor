use crate::engine::actions::{ActionContext, DrawCause, GameAction};
use crate::state::game_state::{
    initial_step, next_phase, next_step, GameState, PhaseType, StepType,
};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::{ManaEmptyReason, BlanketPersistenceSet};

/// Turn structure engine — CR 500, and CR 614.10's three replaceable units.
///
/// # `advance_turn` is a drainer, not a step function
///
/// Every turn, phase and step is *proposed* before it starts (CR 614.1b makes
/// "skip" a replacement effect), so the engine cannot assume the next unit in
/// CR 500.1's sequence is the one that happens. It walks forward from a cursor,
/// proposing each unit in turn, until one of them begins — and CR 500.11's
/// "proceed past it as though it didn't exist" is what a dropped proposal
/// means: a skipped phase proposes none of its steps, a skipped turn advances
/// no turn number and expires nothing, and the sequence resumes from the unit
/// that did not happen rather than from the last one that did.
///
/// **The schedule is read and consumed here, not in a performer.** Popping
/// `turn_queue` and advancing `turn_rotation` are what *build* the proposal;
/// neither is state a CR 614 replacement effect or a CR 603 trigger can see,
/// and both have to be spent whether or not the turn begins — a skipped extra
/// turn is gone (CR 614.10a's "anything scheduled for a skipped turn won't
/// happen"), and the turn after a skipped P2 is P3's rather than P2's again.
impl GameState {
    /// Advance to the next step or phase that actually begins.
    ///
    /// Returns the position it landed on. Ends the current one first: only a
    /// unit that began ends (CR 500.5).
    pub fn advance_turn(
        &mut self,
        ctx: &ActionContext,
    ) -> Result<(PhaseType, Option<StepType>), String> {
        if let Some(step) = self.phase.step {
            self.on_step_end(step)?;
        }
        self.drain(Some(self.phase.phase_type), self.phase.step, true, true, ctx)
    }

    /// Begin the game's first turn, its beginning phase and its untap step —
    /// through the chokepoint, like every later one.
    ///
    /// Called once, by `Game::setup`, after CR 103.4's opening hands.
    /// [`GameState::new`] leaves the board *describing* turn 1 (turn number,
    /// active player, the beginning phase) so that a bare `GameState` in a unit
    /// test reads the way it always has; this is what makes those units
    /// **events**, which is what item 6's "at the beginning of your upkeep"
    /// triggers will read on turn 1 as on every other.
    ///
    /// The turn itself is unskippable here by construction rather than by
    /// exemption — CR 614.4 needs the effect to exist before the event, and
    /// before the first turn nothing has resolved and no permanent has entered
    /// — so it is proposed with the number it already has rather than through
    /// [`Self::next_turn_taker`], which would rotate past the starting player.
    /// Its untap step is proposed like any other.
    ///
    /// **CR 103.6's "begin the game with this on the battlefield" belongs
    /// between the opening hands and this call** and has no implementation;
    /// `codebase-state.md` item 119 owns the seam.
    pub fn start_first_turn(&mut self, ctx: &ActionContext) -> Result<(), String> {
        let player = self.active_player;
        let turn = self.turn_number;
        let performed =
            self.execute_actions(vec![GameAction::BeginTurn { player, turn }], ctx)?;
        if performed.is_empty() {
            return Err(
                "the game's first turn was replaced, which CR 614.4 makes impossible \
                 before anything has resolved — an engine bug rather than a rules corner"
                    .to_string(),
            );
        }
        self.on_turn_begin()?;
        self.drain(None, None, false, true, ctx)?;
        Ok(())
    }

    /// The drainer's loop: propose units in CR 500.1's order until one begins.
    ///
    /// The cursor is *the last unit considered*, which is not the same as the
    /// last unit that happened — that is the whole of CR 500.11. `phase` is
    /// `None` only for the moment after a turn begins and before its beginning
    /// phase is proposed; `phase_began` is false for a phase that was skipped,
    /// and it is what stops the drainer from ending a phase that never started
    /// or from proposing a skipped phase's steps.
    ///
    /// Unbounded, as the rules are: CR 614.10 puts no cap on how many
    /// consecutive turns may be skipped, and the one board that cannot make
    /// progress — every player having left the game — is the one
    /// [`Self::next_turn_taker`] reports as `None`.
    fn drain(
        &mut self,
        mut phase: Option<PhaseType>,
        mut step: Option<StepType>,
        mut phase_began: bool,
        mut turn_began: bool,
        ctx: &ActionContext,
    ) -> Result<(PhaseType, Option<StepType>), String> {
        loop {
            let turn_unit = next_turn_unit(phase, step, phase_began);

            // CR 500.5 — a phase ends when nothing is left in it, and only a
            // phase that began ends. This is where the mana pools empty and
            // CR 511.3's combat state clears, so a skipped phase must not
            // reach it.
            if phase_began && !matches!(turn_unit, TurnUnit::Step(_)) {
                self.on_phase_end(phase.expect("phase_began implies a phase"))?;
                phase_began = false;
            }

            match turn_unit {
                TurnUnit::Step(next) => {
                    if self.begin_step(next, ctx)? {
                        let current = phase.expect("a step belongs to a phase that began");
                        // Turn-based actions, after the event and only for a
                        // step that began (CR 703.4). Outside the proposal's
                        // batch on purpose: the untap sweep is its own
                        // CR 603.2c event, not a result of the step beginning.
                        self.on_step_begin(next, ctx)?;
                        return Ok((current, Some(next)));
                    }
                    step = Some(next);
                }

                TurnUnit::Phase(next) => {
                    phase = Some(next);
                    step = None;
                    phase_began = self.begin_phase(next, ctx)?;
                    // A main phase *is* the position; a phase with steps is
                    // entered together with its first one, which the next
                    // iteration proposes.
                    if phase_began && initial_step(next).is_none() {
                        return Ok((next, None));
                    }
                }

                TurnUnit::Turn => {
                    if turn_began {
                        self.on_turn_end()?;
                        turn_began = false;
                    }
                    let Some(player) = self.next_turn_taker() else {
                        // Every player has left the game (CR 104.2a), so there
                        // is no turn to advance to. The position stays where it
                        // is and `Game::check_game_over` is what ends the game.
                        return Ok((self.phase.phase_type, self.phase.step));
                    };
                    let turn = self.turn_number + 1;
                    let performed =
                        self.execute_actions(vec![GameAction::BeginTurn { player, turn }], ctx)?;
                    if performed.is_empty() {
                        // CR 614.10a — a skipped turn advances no turn number
                        // and expires nothing. The cursor does not move, so the
                        // next iteration proposes the turn after it.
                        continue;
                    }
                    self.on_turn_begin()?;
                    turn_began = true;
                    phase = None;
                }
            }
        }
    }

    /// Who takes the next turn — CR 500.7's queue first, then the natural
    /// rotation — or `None` when no player is left to take one.
    ///
    /// **Consumes what it reads**, and both halves have to. A queued extra turn
    /// that gets skipped is spent on being skipped (CR 614.10a), and a natural
    /// turn that gets skipped still advances the rotation, or the drainer would
    /// propose the same player's turn forever. So this is called once per
    /// proposal, not once per turn that begins.
    ///
    /// CR 800.4k — "if a player who has left the game would begin a turn, that
    /// turn doesn't begin" — is checked here rather than in the pipeline,
    /// because a turn that does not begin is not an event a replacement effect
    /// could have replaced. RE-6 is what makes `player_lost` true for a reason;
    /// this is the site it will use.
    fn next_turn_taker(&mut self) -> Option<PlayerId> {
        while let Some(player) = self.turn_queue.pop() {
            if !self.player_lost[player] {
                return Some(player);
            }
        }
        let n = self.num_players();
        for _ in 0..n {
            self.turn_rotation = (self.turn_rotation + 1) % n;
            if !self.player_lost[self.turn_rotation] {
                return Some(self.turn_rotation);
            }
        }
        None
    }

    /// Propose `phase`'s beginning; report whether it happened.
    fn begin_phase(&mut self, phase: PhaseType, ctx: &ActionContext) -> Result<bool, String> {
        let player = self.active_player;
        let performed =
            self.execute_actions(vec![GameAction::BeginPhase { phase, player }], ctx)?;
        Ok(!performed.is_empty())
    }

    /// Propose `step`'s beginning; report whether it happened.
    fn begin_step(&mut self, step: StepType, ctx: &ActionContext) -> Result<bool, String> {
        // CR 508.8 — "if no creatures are declared as attackers ... skip the
        // declare blockers and combat damage steps". A **rule**, checked ahead
        // of the pipeline like CR 101.2's "can't"s: there is no event here for
        // a replacement effect to see, and a step that does not begin runs no
        // turn-based action and grants no priority.
        if !self.attacks_declared
            && matches!(
                step,
                StepType::DeclareBlockers | StepType::FirstStrikeDamage | StepType::CombatDamage
            )
        {
            return Ok(false);
        }
        let player = self.active_player;
        let performed = self.execute_actions(vec![GameAction::BeginStep { step, player }], ctx)?;
        Ok(!performed.is_empty())
    }

    // --- Turn lifecycle callbacks ---

    /// CR 500.4's expiries, for a turn that **began**.
    ///
    /// > 611.2b ... "until your next turn" ... it lasts until that player's
    /// > next turn begins.
    ///
    /// Here and not in the untap step's begin hook, and the difference bites in
    /// two directions. Eight printed cards skip the untap step, and an expiry
    /// hung off a step that may not happen is an effect that never ends; and
    /// CR 614.10a's "a skipped turn expires nothing" is the same sentence from
    /// the other side — this hook runs only for a turn whose proposal
    /// survived.
    fn on_turn_begin(&mut self) -> Result<(), String> {
        let player = self.active_player;
        let turn = self.turn_number;
        self.continuous_effects.remove_expired_at_turn_start(player, turn);
        // CR 611.2b applies to replacement effects with a duration the same
        // way — a regeneration shield or "prevent all damage this turn" ends
        // when its duration does.
        self.replacement_effects.remove_expired_at_turn_start(player, turn);
        self.restrictions.remove_expired_at_turn_start(player, turn);
        Ok(())
    }

    // --- Phase lifecycle callbacks ---

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
                // "Until your next turn" expires at `on_turn_begin`, not here:
                // this step can be skipped and a turn cannot un-begin.
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
                // ... and the replacement effects with the same duration. This
                // is what makes CR 701.19a's "the next time [permanent] would be
                // destroyed **this turn**" end at end of turn rather than
                // lingering as an unused shield forever.
                self.replacement_effects.remove_expired_at_cleanup(
                    self.active_player,
                    self.turn_number,
                );
                // ... and the CR 101.2 "can't"s, whose durations are the
                // card's rather than the engine's. This replaces a hardcoded
                // `cant_be_regenerated.clear()` that asserted every "can't be
                // regenerated" was a this-turn fact with no rule cited; the
                // scope is now a `Duration` argument on `Primitive::Restrict`,
                // which is where CR 608.2c can be applied per card.
                self.restrictions.remove_expired_at_cleanup(
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

    /// The counterpart of [`Self::on_turn_begin`], and empty for a reason:
    /// per-turn resets (land drops) happen in `process_untap_step`, where
    /// CR 502 puts them, and CR 514.2's cleanup is the cleanup step's.
    ///
    /// Runs only for a turn that began — a skipped turn has no end.
    fn on_turn_end(&mut self) -> Result<(), String> {
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
        // One batch: CR 502.1 says the permanents untap *simultaneously*, and
        // CR 603.2c's "whenever one or more permanents untap" reads the batch,
        // not its members.
        let batch = to_untap.into_iter().map(|object| GameAction::Untap { object }).collect();
        self.execute_actions(batch, ctx)?;

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
            //
            // The **instruction** rather than the draw (CR 121.2a). CR 504.1's
            // turn-based action is "draw a card", which is one "draw" of one
            // card, and every draw instruction proposes the outer so that
            // Divination and a pair of cantrips are told apart by `n` rather
            // than by which event the producer happened to build. This is the
            // engine's one `DrawCause::TurnBased` site (CR 121.1).
            self.execute_action(
                GameAction::DrawCards { player: active, n: 1, cause: DrawCause::TurnBased },
                ctx,
            )?;
        }

        self.priority_player = active;
        Ok(())
    }

}

/// The next thing CR 500.1's sequence offers the drainer.
///
/// Three arms because CR 614.10 replaces three units and each is proposed
/// separately: a phase that begins does not drag its first step in with it, or
/// a card that skips the upkeep step would have to skip the beginning phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TurnUnit {
    /// Another step inside the phase that is happening.
    Step(StepType),
    /// The phase after the cursor's — CR 500.1's order, wrapping to the next
    /// turn's beginning phase.
    Phase(PhaseType),
    /// The turn boundary: the ending phase is behind the cursor.
    Turn,
}

/// CR 500.1's sequence, read off the drainer's cursor.
///
/// `phase` is `None` for the instant after a turn begins, when no phase of it
/// has been proposed yet. `phase_began` false is CR 500.11's "as though it
/// didn't exist": a skipped phase offers no steps, so the sequence goes
/// straight to the phase after it.
///
/// A free function rather than a method because it reads nothing but the
/// cursor — which is what makes the drainer's termination argument checkable:
/// the cursor advances through a fixed, finite sequence on every iteration
/// except the one where a turn is skipped, and that one consumes a schedule
/// entry or the rotation.
fn next_turn_unit(
    phase: Option<PhaseType>,
    step: Option<StepType>,
    phase_began: bool,
) -> TurnUnit {
    let Some(current) = phase else {
        return TurnUnit::Phase(PhaseType::Beginning);
    };
    if phase_began {
        let next = match step {
            Some(last) => next_step(current, last),
            None => initial_step(current),
        };
        if let Some(next) = next {
            return TurnUnit::Step(next);
        }
    }
    if current == PhaseType::Ending {
        return TurnUnit::Turn;
    }
    TurnUnit::Phase(next_phase(current))
}

#[cfg(test)]
mod tests {
    use crate::types::replacement::EnterMods;
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
        // Combat: BeginCombat, DeclareAttackers, EndCombat = 3 advances
        // Postcombat: 1 advance (no steps)
        // Ending: End, Cleanup = 2 advances
        // Total: 10 advances to complete one turn — CR 508.8 refuses the
        // declare-blockers, first-strike and combat-damage steps when nothing
        // attacked, so they are not positions `advance_turn` stops on.

        for _ in 0..10 {
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
        game.place_on_battlefield(forest_id, 0, &EnterMods::NONE).tapped = true;

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
        game.place_on_battlefield(tapped_id, 0, &EnterMods::NONE).tapped = true;
        let untapped_id = game.add_object(GameObject::new(
            land("Untapped Forest"), 0, crate::types::zones::Zone::Battlefield));
        game.place_on_battlefield(untapped_id, 0, &EnterMods::NONE).tapped = false;

        let before = game.events.len();
        // Walk to player 0's next untap step.
        for _ in 0..26 {
            game.advance_turn(&test_ctx()).unwrap();
        }

        let untapped: Vec<crate::types::ids::ObjectId> = game.events.records_from(before).iter()
            .filter_map(|r| match &r.event {
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
    fn test_the_untap_step_is_one_batch() {
        let mut game = GameState::new(2, 20);
        stock_libraries(&mut game, 5);

        let land = |name: &str| CardDataBuilder::new(name)
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .mana_ability_single(ManaType::Green)
            .build();

        for name in ["Forest A", "Forest B"] {
            let id = game.add_object(GameObject::new(
                land(name), 0, crate::types::zones::Zone::Battlefield));
            game.place_on_battlefield(id, 0, &EnterMods::NONE).tapped = true;
        }

        let before = game.events.len();
        for _ in 0..26 {
            game.advance_turn(&test_ctx()).unwrap();
        }

        // CR 502.1: "all the permanents untap simultaneously." One batch id is
        // what lets CR 603.2c's "whenever one or more permanents untap" fire
        // once for the step instead of once per permanent.
        let batches: Vec<_> = game.events.records_from(before).iter()
            .filter(|r| matches!(r.event, crate::events::event::GameEvent::Untapped { .. }))
            .map(|r| r.batch())
            .collect();
        assert_eq!(batches.len(), 2, "both lands untapped");
        assert!(batches[0].is_some(), "the untap step is batched");
        assert_eq!(batches[0], batches[1], "one step, one event");
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
