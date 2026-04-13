// Random DecisionProvider — makes random legal choices for fuzz testing.
//
// Uses an internal action queue for tap-then-cast sequences. All choices
// are random but *legal* — the DP queries oracle helpers to find valid
// options before selecting among them.

use std::collections::HashMap;

use rand::Rng;
use rand::seq::IndexedRandom;

use crate::engine::resolve::ResolvedTarget;
use crate::events::event::DamageTarget;
use crate::oracle::legality::{legal_attackers, legal_blockers, playable_lands};
use crate::oracle::mana_helpers::castable_spells;
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::GameState;
use crate::types::effects::{EffectRecipient, SelectionFilter};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaType};
use crate::ui::decision::{
    auto_allocate_generic, default_damage_assignment, default_trample_assignment,
    is_action_still_valid, queue_tap_and_cast, DecisionProvider, PriorityAction,
};

/// A decision provider that makes random legal choices.
///
/// Designed for fuzz testing: run many games of Random vs Random to surface
/// panics and edge cases in the engine.
///
/// Uses an internal `VecDeque<PriorityAction>` plan queue for tap-then-cast
/// sequences. When the queue is empty and `choose_priority_action` is called:
/// 1. **Land check**: always play a land if possible. The `playable_lands`
///    oracle query already enforces timing (active player, main phase, stack
///    empty, land drop available) — see `oracle::legality::playable_lands`.
/// 2. **Cast or pass**: phase-aware probability. During main phases (where
///    sorcery-speed spells are legal), ~80% try to cast. During non-main
///    steps (instant-only window), ~30% try to cast. This prevents the
///    bot from burning through instants in the many non-main priority
///    passes, biasing the game toward creature/sorcery action.
///
/// The queue is validated on drain: `is_action_still_valid()` catches stale
/// plans (e.g. a permanent was tapped between queue creation and execution).
/// If any action is stale the entire queue is discarded, since later actions
/// depend on earlier ones succeeding. This is intentionally not exhaustive —
/// false positives are caught by the engine's own legality checks and result
/// in errors that cause the DP to re-plan on the next priority pass.
pub struct RandomDecisionProvider {
    action_queue: std::cell::RefCell<std::collections::VecDeque<PriorityAction>>,
}

impl RandomDecisionProvider {
    pub fn new() -> Self {
        RandomDecisionProvider {
            action_queue: std::cell::RefCell::new(std::collections::VecDeque::new()),
        }
    }
}

impl Default for RandomDecisionProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DecisionProvider for RandomDecisionProvider {
    fn choose_priority_action(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> PriorityAction {
        // Drain queue first, validating each action is still legal
        {
            let mut queue = self.action_queue.borrow_mut();
            if let Some(action) = queue.pop_front() {
                if is_action_still_valid(game, player_id, &action) {
                    return action;
                }
                // Stale plan — discard remaining queued actions
                queue.clear();
            }
        }

        let mut rng = rand::rng();

        // 1. Land check: always play a land if we can
        let lands = playable_lands(game, player_id);
        if !lands.is_empty() {
            let &land_id = lands.choose(&mut rng).unwrap();
            return PriorityAction::PlayLand(land_id);
        }

        // 2. Cast or pass: phase-aware probability
        //    Main phase = sorcery-speed window → 80% (exercise creatures/sorceries)
        //    Non-main steps = instant-only → 30% (conserve instants)
        let is_main = matches!(
            game.phase.phase_type,
            crate::state::game_state::PhaseType::Precombat | crate::state::game_state::PhaseType::Postcombat
        );
        let cast_prob = if is_main { 0.80 } else { 0.30 };
        let castable = castable_spells(game, player_id);
        if !castable.is_empty() && rng.random_bool(cast_prob) {
            let idx = rng.random_range(0..castable.len());
            let (card_id, ref sources) = castable[idx];
            return queue_tap_and_cast(&self.action_queue, sources, card_id);
        }

        PriorityAction::Pass
    }

    fn choose_attackers(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> Vec<(ObjectId, AttackTarget)> {
        let available = legal_attackers(game, player_id);
        if available.is_empty() {
            return Vec::new();
        }

        let mut rng = rand::rng();

        // Find an opponent to attack
        let defending = (0..game.num_players())
            .find(|&pid| pid != player_id)
            .unwrap_or(1);

        // Random subset of attackers
        let mut attackers = Vec::new();
        for &id in &available {
            if rng.random_bool(0.5) {
                attackers.push((id, AttackTarget::Player(defending)));
            }
        }
        attackers
    }

    fn choose_blockers(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> Vec<(ObjectId, ObjectId)> {
        let available = legal_blockers(game, player_id);
        if available.is_empty() {
            return Vec::new();
        }

        // Find attacking creatures
        let attackers: Vec<ObjectId> = game
            .battlefield
            .iter()
            .filter(|(_, e)| e.attacking.is_some())
            .map(|(id, _)| *id)
            .collect();

        if attackers.is_empty() {
            return Vec::new();
        }

        let mut rng = rand::rng();
        let mut blocks = Vec::new();

        // Each available blocker has a ~40% chance to block a random attacker
        for &blocker_id in &available {
            if rng.random_bool(0.40) {
                if let Some(&attacker_id) = attackers.choose(&mut rng) {
                    // Check flying/reach: only block flyers if we have flying or reach
                    let attacker_flies = crate::oracle::characteristics::has_keyword(
                        game,
                        attacker_id,
                        crate::types::keywords::KeywordAbility::Flying,
                    );
                    if attacker_flies {
                        let can_block_flyer =
                            crate::oracle::characteristics::has_keyword(
                                game,
                                blocker_id,
                                crate::types::keywords::KeywordAbility::Flying,
                            ) || crate::oracle::characteristics::has_keyword(
                                game,
                                blocker_id,
                                crate::types::keywords::KeywordAbility::Reach,
                            );
                        if !can_block_flyer {
                            continue;
                        }
                    }
                    blocks.push((blocker_id, attacker_id));
                }
            }
        }

        blocks
    }

    fn choose_discard(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> Option<ObjectId> {
        let player = game.players.get(player_id)?;
        if player.hand.is_empty() {
            return None;
        }
        let mut rng = rand::rng();
        player.hand.choose(&mut rng).copied()
    }

    fn choose_targets(
        &self,
        game: &GameState,
        _player_id: PlayerId,
        recipient: &EffectRecipient,
    ) -> Vec<ResolvedTarget> {
        let mut rng = rand::rng();

        // Extract the filter — Target and Choose select identically here.
        let filter = match recipient {
            EffectRecipient::Implicit | EffectRecipient::Controller => return Vec::new(),
            EffectRecipient::Target(f, _) | EffectRecipient::Choose(f, _) => f,
        };

        match filter {
            SelectionFilter::Player => {
                // Random player
                let pid = rng.random_range(0..game.num_players());
                vec![ResolvedTarget::Player(pid)]
            }
            SelectionFilter::Creature => {
                // Random creature on the battlefield
                let creatures: Vec<ObjectId> = game
                    .battlefield
                    .keys()
                    .copied()
                    .filter(|&id| {
                        crate::oracle::characteristics::is_creature(game, id)
                    })
                    .collect();
                if let Some(&id) = creatures.choose(&mut rng) {
                    vec![ResolvedTarget::Object(id)]
                } else {
                    Vec::new()
                }
            }
            SelectionFilter::Any => {
                // Randomly pick player or creature
                let creatures: Vec<ObjectId> = game
                    .battlefield
                    .keys()
                    .copied()
                    .filter(|&id| {
                        crate::oracle::characteristics::is_creature(game, id)
                    })
                    .collect();

                let num_targets = game.num_players() + creatures.len();
                if num_targets == 0 {
                    return Vec::new();
                }

                let choice = rng.random_range(0..num_targets);
                if choice < game.num_players() {
                    vec![ResolvedTarget::Player(choice)]
                } else {
                    let idx = choice - game.num_players();
                    vec![ResolvedTarget::Object(creatures[idx])]
                }
            }
            SelectionFilter::Permanent(_) => {
                // Random permanent that matches the filter
                let perms: Vec<ObjectId> = game
                    .battlefield
                    .keys()
                    .copied()
                    .filter(|&id| {
                        game.validate_selection(filter, &ResolvedTarget::Object(id)).is_ok()
                    })
                    .collect();
                if let Some(&id) = perms.choose(&mut rng) {
                    vec![ResolvedTarget::Object(id)]
                } else {
                    Vec::new()
                }
            }
            SelectionFilter::Spell => {
                // Random spell on the stack, excluding the top entry
                // (the spell being cast — it's already on the stack when
                // choose_targets is called, and a spell can't target itself,
                // rule 114.5).
                let top = game.stack.last().copied();
                let candidates: Vec<ObjectId> = game.stack.iter()
                    .copied()
                    .filter(|&id| Some(id) != top)
                    .collect();
                if let Some(&id) = candidates.choose(&mut rng) {
                    vec![ResolvedTarget::Object(id)]
                } else {
                    Vec::new()
                }
            }
        }
    }

    fn choose_attacker_damage_assignment(
        &self,
        game: &GameState,
        _player_id: PlayerId,
        _attacker_id: ObjectId,
        blockers: &[ObjectId],
        power: u64,
    ) -> Vec<(ObjectId, u64)> {
        default_damage_assignment(game, blockers, power)
    }

    fn choose_trample_damage_assignment(
        &self,
        game: &GameState,
        _player_id: PlayerId,
        _attacker_id: ObjectId,
        blockers: &[ObjectId],
        _defending_target: DamageTarget,
        power: u64,
        has_deathtouch: bool,
    ) -> (Vec<(ObjectId, u64)>, u64) {
        default_trample_assignment(game, blockers, power, has_deathtouch)
    }

    fn choose_legend_to_keep(
        &self,
        _game: &GameState,
        _player_id: PlayerId,
        legendaries: &[ObjectId],
    ) -> ObjectId {
        // Keep the first one (arbitrary — random bot doesn't care)
        legendaries[0]
    }

    fn choose_generic_mana_allocation(
        &self,
        game: &GameState,
        player_id: PlayerId,
        mana_cost: &ManaCost,
    ) -> HashMap<ManaType, u64> {
        auto_allocate_generic(game, player_id, mana_cost).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::BattlefieldEntity;
    use crate::state::game_state::{GameState, Phase, PhaseType};
    use crate::types::card_types::*;
    use crate::types::zones::Zone;

    fn setup_basic_game() -> GameState {
        let mut game = GameState::new(2, 20);
        game.phase = Phase::new(PhaseType::Precombat);
        game.active_player = 0;
        game
    }

    #[allow(dead_code)]
    fn place_forest(game: &mut GameState, player_id: PlayerId) -> ObjectId {
        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Forest))
            .mana_ability_single(ManaType::Green)
            .build();
        let obj = GameObject::new(forest, player_id, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = BattlefieldEntity::new(id, player_id, ts, 0);
        game.battlefield.insert(id, entry);
        id
    }

    #[test]
    fn test_random_dp_passes_when_nothing_to_do() {
        let game = setup_basic_game();
        let dp = RandomDecisionProvider::new();

        // With no cards in hand and no lands, should pass
        let action = dp.choose_priority_action(&game, 0);
        assert_eq!(action, PriorityAction::Pass);
    }

    #[test]
    fn test_random_dp_choose_discard() {
        let mut game = setup_basic_game();
        let data = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(data, 0, Zone::Hand);
        let id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(id);

        let dp = RandomDecisionProvider::new();
        let discard = dp.choose_discard(&game, 0);
        assert_eq!(discard, Some(id));
    }

    #[test]
    fn test_random_dp_choose_targets_player() {
        let game = setup_basic_game();
        let dp = RandomDecisionProvider::new();
        let targets = dp.choose_targets(
            &game,
            0,
            &EffectRecipient::Target(SelectionFilter::Player, crate::types::effects::TargetCount::Exactly(1)),
        );
        assert_eq!(targets.len(), 1);
        match targets[0] {
            ResolvedTarget::Player(pid) => assert!(pid < 2),
            _ => panic!("Expected player target"),
        }
    }

    #[test]
    fn test_random_dp_attackers_empty_battlefield() {
        let game = setup_basic_game();
        let dp = RandomDecisionProvider::new();
        let attackers = dp.choose_attackers(&game, 0);
        assert!(attackers.is_empty());
    }

    #[test]
    fn test_random_dp_blockers_no_attackers() {
        let mut game = setup_basic_game();
        // Place a creature for player 1 but nobody is attacking
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 1, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = BattlefieldEntity::new(id, 1, ts, 0);
        game.battlefield.insert(id, entry);

        let dp = RandomDecisionProvider::new();
        let blockers = dp.choose_blockers(&game, 1);
        assert!(blockers.is_empty());
    }
}
