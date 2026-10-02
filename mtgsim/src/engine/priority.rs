use crate::oracle::legality::candidate_priority_actions;
use crate::engine::actions::ActionContext;
use crate::state::decision_log::{LoggedAnswer, LoggedDecision};
use crate::state::game_state::GameState;
use crate::types::zones::Zone;
use crate::ui::ask::ask_choose_priority_action;
use crate::ui::choice_types::{ChoiceKind, Rejection};
use crate::ui::decision::{DecisionProvider, PriorityAction};

/// How many answers to one question the engine rejects before it ends the
/// game with an error. The CR sets no limit (CR 732.2 lets a player redo a
/// reversed action), and this one is far past anything a person does, so only
/// an agent that ignores `ChoiceContext::rejected` reaches it.
pub(crate) const REJECTION_LIMIT: usize = 1_000;

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

        // CR 104.1 — the game ends immediately, and nobody receives priority
        // in a game that has ended.
        if self.result.is_some() {
            return Ok(PriorityResult::PhaseEnds);
        }

        // CR 800.4j — players who have left the game are passed over, and a
        // round ends when everyone *still in the game* has passed in
        // succession. The active player receives priority first if they are
        // still here; "if the active player would receive priority, instead
        // the next player in turn order receives priority" otherwise.
        //
        // Counted once per round: a player can only leave between rounds,
        // because a loss is a state-based action and those run at the top of
        // this function and after each action, each of which starts a new
        // round.
        let players_in_game = (0..self.num_players()).filter(|&p| self.in_game(p)).count();
        let mut consecutive_passes = 0;
        let Some(mut current_priority) = (if self.in_game(self.active_player) {
            Some(self.active_player)
        } else {
            self.next_player_in_game(self.active_player)
        }) else {
            return Ok(PriorityResult::PhaseEnds);
        };

        loop {
            self.priority_player = current_priority;

            // `candidate_priority_actions` is an overapproximation: it includes
            // e.g. `CastSpell(id)` when affordability is heuristically met but
            // the current mana pool can't actually cover the cost, and the engine
            // asks again when execution rejects a DP-chosen action (§2.2 of
            // `plans/atomic-tests/supplemental-docs/dp-middleware-and-candidate-enumeration.md`).
            //
            // The re-ask offers the board's list as it stands, the rejected
            // action included, and says which action was rejected: the player
            // may take it again (CR 732.2), and skipping it is an agent's policy,
            // which lives on the agent's seat (`codebase-state.md` item 193).
            let mut rejected: Option<Rejection> = None;
            let mut rejections: usize = 0;

            // Choose an action and attempt execution; ask again on failure.
            // On success, `executed` holds the action that ran and (for
            // ActivateAbility) whether it was a mana ability (which bypasses
            // the post-action SBA pass, per rule 605).
            let executed: (PriorityAction, bool) = loop {
                // Enumerated per prompt, not per window. A rejected cast's mana
                // abilities stay activated (CR 732.1 — the reversal is the
                // player's option and the engine never offers it), so the board a
                // re-ask is offered from is not the board the last list was built
                // from, and re-offering that list offers casts no enumeration of
                // *this* board would. The prompt has to be a function of
                // `GameState` or a clone taken at one cannot rebuild it —
                // `codebase-state.md` items 139 and 41.
                let available: Vec<PriorityAction> = candidate_priority_actions(self, current_priority);

                // `Pass` is always offered, so a list of one is `[Pass]` alone:
                // one legal answer, the engine's (`backlog.md` §2.22, rule 1),
                // unless the seat stops at every priority point. The engine's
                // pass is logged as the seat's would be, so the decision log
                // does not depend on whether the seat stops.
                if available.len() == 1
                    && !decisions.seat_mode(current_priority).stops_at_every_priority_point
                {
                    self.log_decision(LoggedDecision {
                        player: current_priority,
                        kind: &ChoiceKind::PriorityAction,
                        answer: LoggedAnswer::Picks(&[0]),
                        forced: true,
                    });
                    break (PriorityAction::Pass, false);
                }

                let action = ask_choose_priority_action(
                    decisions, self, current_priority, &available, rejected.take(),
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
                        self.play_land(
                            current_priority, *card_id, Zone::Hand,
                            &ActionContext::new(decisions),
                        ),
                        false,
                    ),
                    PriorityAction::ActivateAbility(permanent_id, ability_id) => 'activation: {
                        // Dispatch mana-vs-non-mana. Mana abilities resolve
                        // immediately (rule 605) and don't trigger SBAs.
                        if let Err(e) = self.get_object(*permanent_id) {
                            // Source disappeared — rejected like any other.
                            eprintln!(
                                "WARN: activate_ability source {} missing: {}",
                                permanent_id, e
                            );
                            break 'activation (Err(e), false);
                        }
                        // Effective abilities: intrinsic land mana abilities
                        // (CR 305.6) are absent from CardData, and the index
                        // handed to `activate_ability` must match the list
                        // `activatable_abilities` enumerated.
                        let abilities =
                            crate::oracle::characteristics::get_effective_abilities(
                                self, *permanent_id,
                            );
                        let is_mana = abilities.iter()
                            .find(|a| a.id == *ability_id)
                            .map(|a| a.ability_type == crate::objects::card_data::AbilityType::Mana)
                            .unwrap_or(false);
                        let result = if is_mana {
                            self.activate_mana_ability(
                                current_priority, *permanent_id, *ability_id,
                                &ActionContext::new(decisions),
                            )
                        } else {
                            let idx = abilities.iter()
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
                    Err(e) => {
                        rejections += 1;
                        // What `--dump-events` cannot show: a cast the enumeration
                        // offered and the engine rejected performs nothing, so the
                        // re-ask that follows is only explicable from here.
                        self.trace(|| {
                            crate::engine::trace_records::priority_rejected(current_priority, &action, &e, rejections)
                        });
                        if rejections == REJECTION_LIMIT {
                            return Err(format!(
                                "player {current_priority} chose an action the engine rejected {REJECTION_LIMIT} times in \
                                 one priority window, the last {action:?}: {e}. A provider that keeps choosing what it was \
                                 told failed does not read `ChoiceContext::rejected`."
                            ));
                        }
                        rejected = Some(Rejection::Reversed(action));
                    }
                }
            };

            match executed.0 {
                PriorityAction::Pass => {
                    consecutive_passes += 1;
                    if consecutive_passes >= players_in_game {
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
                    // Next player still in the game gets priority (117.3d,
                    // 800.4j). `players_in_game >= 1` here, so there is one.
                    current_priority = self
                        .next_player_in_game(current_priority)
                        .expect("a round with a passer has a player in the game");
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
    ///
    /// Step 2 is where CR 603.8's state check will run first (TR-6); until
    /// then the queue holds only what a dispatch put there.
    pub fn perform_sba_and_triggers(&mut self, decisions: &dyn DecisionProvider) -> Result<(), String> {
        loop {
            // Step 1: Exhaust all SBAs (rule 704.3)
            self.check_state_based_actions_loop(decisions)?;

            // CR 104.1 — a game that has ended places nothing.
            if self.result.is_some() {
                break;
            }

            // Step 2: Place triggered abilities on the stack (rule 603.3)
            let triggers_placed = self.place_pending_triggers(decisions)?;

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
    use crate::ui::decision::{ScriptedDecisionProvider, SeatMode};

    /// Each player has `Pass` alone, which the engine takes: an empty script
    /// panics on any prompt, so the round ending is the assertion that
    /// nobody was asked.
    #[test]
    fn test_all_pass_empty_stack_ends_phase() {
        let mut game = GameState::new(2, 20);
        game.set_turn_position(crate::state::game_state::Phase::new(PhaseType::Precombat));
        let decisions = ScriptedDecisionProvider::new();

        let result = game.run_priority_round(&decisions).unwrap();
        assert_eq!(result, PriorityResult::PhaseEnds);
    }

    /// The same round with both seats stopping at every priority point, as a
    /// person in full control does: each is asked, though `Pass` is all
    /// either can do.
    #[test]
    fn a_seat_that_stops_at_every_priority_point_is_asked_to_pass() {
        let mut game = GameState::new(2, 20);
        game.set_turn_position(crate::state::game_state::Phase::new(PhaseType::Precombat));
        let decisions = ScriptedDecisionProvider::new()
            .with_seat_mode(SeatMode { stops_at_every_priority_point: true });
        decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
        decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);

        let result = game.run_priority_round(&decisions).unwrap();
        assert_eq!(result, PriorityResult::PhaseEnds);
    }

    #[test]
    fn test_cast_and_resolve_via_priority() {
        let mut game = GameState::new(2, 20);
        game.set_turn_position(crate::state::game_state::Phase::new(PhaseType::Precombat));
        game.active_player = 0;

        // Give player 0 a bolt in hand and red mana
        let bolt_data = CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::build(&[ManaType::Red], 0))
            .ability(AbilityDef {
                rules_text: "".into(),
                is_characteristic_defining: false,
                activation_restriction: crate::objects::card_data::ActivationRestriction::None,
                id: crate::types::ids::new_ability_id(),
                instances: Vec::new(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage { amount: AmountExpr::Fixed(3), unpreventable: false },
                    EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
                ),
            })
            .build();
        let obj = GameObject::new(bolt_data, 0, Zone::Hand);
        let card_id = game.add_object(obj);
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

        // Second round: both pass, stack resolves — each has `Pass` alone
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
        game.set_turn_position(crate::state::game_state::Phase::new(PhaseType::Precombat));
        let decisions = ScriptedDecisionProvider::new();
        // Both players pass — phase ends

        game.run_priority_loop(&decisions).unwrap();
        // Should complete without error — phase ended
    }
}
