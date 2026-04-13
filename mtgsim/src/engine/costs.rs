use std::collections::HashMap;

use crate::types::costs::Cost;
use crate::oracle::characteristics::{has_summoning_sickness, is_creature};
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::ManaType;

/// Shared cost payment logic.
///
/// All spells and ability types (mana, activated, spell casting) that need to pay costs
/// funnel through this module. This avoids duplicating the cost payment
/// pattern across mana_abilities.rs, activated.rs, etc.
///
/// **Generic mana allocation:** When a cost includes a generic mana component,
/// the caller must provide a `generic_allocation` map specifying which mana
/// types to spend. This is a player decision (routed through `DecisionProvider`
/// in the calling code). See `ManaPool::pay()` for details.

impl GameState {
    /// Read-only check: can all costs be paid right now?
    ///
    /// Checks both resource availability AND cost restrictions.
    /// Cost restrictions (Phase 5) start as a no-op — the `check_cost_restrictions`
    /// call is a placeholder for when continuous effects populate
    /// `GameState::cost_restrictions`.
    pub fn can_pay_costs(
        &self,
        costs: &[Cost],
        player_id: PlayerId,
        source_id: ObjectId,
    ) -> Result<(), String> {
        for cost in costs {
            self.check_cost_resource(cost, player_id, source_id)?;
            // Phase 5: self.check_cost_restrictions(cost, player_id, source_id)?;
        }
        Ok(())
    }

    /// Resource check: does the player have the resources to pay this cost?
    fn check_cost_resource(
        &self,
        cost: &Cost,
        player_id: PlayerId,
        source_id: ObjectId,
    ) -> Result<(), String> {
        match cost {
            Cost::Tap => {
                let entry = self.battlefield.get(&source_id)
                    .ok_or_else(|| format!("Permanent {} not on battlefield", source_id))?;
                if entry.tapped {
                    return Err("Permanent is already tapped".to_string());
                }
                // Rule 302.6 / 702.10c: Summoning sickness prevents creatures from
                // tapping, unless they have haste.
                if is_creature(self, source_id) && has_summoning_sickness(self, source_id) {
                    return Err("Creature has summoning sickness".to_string());
                }
                Ok(())
            }
            Cost::Untap => {
                let entry = self.battlefield.get(&source_id)
                    .ok_or_else(|| format!("Permanent {} not on battlefield", source_id))?;
                if !entry.tapped {
                    return Err("Permanent is not tapped".to_string());
                }
                // Rule 302.6 / 702.10c: Summoning sickness prevents creatures from
                // paying {Q} (untap symbol), unless they have haste.
                if is_creature(self, source_id) && has_summoning_sickness(self, source_id) {
                    return Err("Creature has summoning sickness".to_string());
                }
                Ok(())
            }
            Cost::Mana(mana_cost) => {
                let player = self.get_player(player_id)?;
                if !player.mana_pool.can_pay(mana_cost) {
                    return Err("Not enough mana".to_string());
                }
                Ok(())
            }
            Cost::PayLife(amount) => {
                let player = self.get_player(player_id)?;
                if player.life_total < *amount as i64 {
                    return Err(format!(
                        "Cannot pay {} life, only {} available",
                        amount, player.life_total
                    ));
                }
                Ok(())
            }
            Cost::SacrificeSelf => {
                if !self.battlefield.contains_key(&source_id) {
                    return Err(format!("Permanent {} not on battlefield", source_id));
                }
                Ok(())
            }
            Cost::Sacrifice(_, _)
            | Cost::Discard(_, _)
            | Cost::ExileFromGraveyard(_, _)
            | Cost::RemoveCounters(_, _)
            | Cost::AddCounters(_, _) => {
                Err(format!("Cost {:?} validation not yet implemented", cost))
            }
        }
    }

    /// Pay a list of costs for a spell or permanent's ability.
    ///
    /// `generic_allocation` specifies how to pay any generic mana components.
    /// For costs with no generic mana (most ability costs), pass an empty map.
    ///
    /// Validates and pays each cost in order. If any cost can't be paid,
    /// returns an error (costs already paid are NOT rolled back — the caller
    /// should validate with `can_pay_costs` first if rollback-safety is needed).
    pub fn pay_costs(
        &mut self,
        costs: &[Cost],
        player_id: PlayerId,
        source_id: ObjectId,
        generic_allocation: &HashMap<ManaType, u64>,
    ) -> Result<(), String> {
        for cost in costs {
            self.pay_single_cost(cost, player_id, source_id, generic_allocation)?;
        }
        Ok(())
    }

    /// Pay a single cost. Internal helper.
    fn pay_single_cost(
        &mut self,
        cost: &Cost,
        player_id: PlayerId,
        source_id: ObjectId,
        generic_allocation: &HashMap<ManaType, u64>,
    ) -> Result<(), String> {
        match cost {
            Cost::Tap => {
                let entry = self.battlefield.get(&source_id)
                    .ok_or_else(|| format!("Permanent {} not on battlefield", source_id))?;
                if entry.tapped {
                    return Err("Permanent is already tapped".to_string());
                }
                // Rule 302.6 / 702.10c: Summoning sickness prevents creatures from
                // tapping, unless they have haste.
                if is_creature(self, source_id) && has_summoning_sickness(self, source_id) {
                    return Err("Creature has summoning sickness".to_string());
                }
                let entry = self.battlefield.get_mut(&source_id).unwrap();
                entry.tapped = true;
                Ok(())
            }
            Cost::Untap => {
                let entry = self.battlefield.get(&source_id)
                    .ok_or_else(|| format!("Permanent {} not on battlefield", source_id))?;
                if !entry.tapped {
                    return Err("Permanent is not tapped".to_string());
                }
                // Rule 302.6 / 702.10c: Summoning sickness prevents creatures from
                // paying {Q} (untap symbol), unless they have haste.
                if is_creature(self, source_id) && has_summoning_sickness(self, source_id) {
                    return Err("Creature has summoning sickness".to_string());
                }
                let entry = self.battlefield.get_mut(&source_id).unwrap();
                entry.tapped = false;
                Ok(())
            }
            Cost::Mana(mana_cost) => {
                let player = self.get_player_mut(player_id)?;
                if mana_cost.generic_count() == 0 {
                    player.mana_pool.pay_specific_only(mana_cost)
                } else {
                    player.mana_pool.pay(mana_cost, generic_allocation)
                }
            }
            Cost::PayLife(amount) => {
                let player = self.get_player_mut(player_id)?;
                if player.life_total < *amount as i64 {
                    return Err(format!(
                        "Cannot pay {} life, only {} available",
                        amount, player.life_total
                    ));
                }
                player.life_total -= *amount as i64;
                Ok(())
            }
            Cost::SacrificeSelf => {
                self.move_object(source_id, crate::types::zones::Zone::Graveyard)
            }
            Cost::Sacrifice(_, _)
            | Cost::Discard(_, _)
            | Cost::ExileFromGraveyard(_, _)
            | Cost::RemoveCounters(_, _)
            | Cost::AddCounters(_, _) => {
                Err(format!("Cost {:?} payment not yet implemented", cost))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::types::costs::Cost;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::BattlefieldEntity;
    use crate::state::game_state::GameState;
    use crate::types::card_types::*;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;

    fn setup_with_forest() -> (GameState, crate::types::ids::ObjectId) {
        let mut game = GameState::new(2, 20);
        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Forest))
            .mana_ability_single(ManaType::Green)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = BattlefieldEntity::new(id, 0, 0, 0);
        game.battlefield.insert(id, entry);
        (game, id)
    }

    #[test]
    fn test_pay_tap_cost() {
        let (mut game, forest_id) = setup_with_forest();
        let no_alloc = HashMap::new();
        game.pay_costs(&[Cost::Tap], 0, forest_id, &no_alloc).unwrap();
        assert!(game.battlefield.get(&forest_id).unwrap().tapped);
    }

    #[test]
    fn test_pay_tap_cost_already_tapped() {
        let (mut game, forest_id) = setup_with_forest();
        let no_alloc = HashMap::new();
        game.pay_costs(&[Cost::Tap], 0, forest_id, &no_alloc).unwrap();
        assert!(game.pay_costs(&[Cost::Tap], 0, forest_id, &no_alloc).is_err());
    }

    #[test]
    fn test_pay_mana_cost_specific() {
        let (mut game, _) = setup_with_forest();
        game.players[0].mana_pool.add(ManaType::Green, 2);
        let cost = ManaCost::build(&[ManaType::Green], 0);
        let no_alloc = HashMap::new();
        game.pay_costs(&[Cost::Mana(cost)], 0, crate::types::ids::new_object_id(), &no_alloc).unwrap();
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 1);
    }

    #[test]
    fn test_pay_mana_cost_with_generic() {
        let (mut game, _) = setup_with_forest();
        game.players[0].mana_pool.add(ManaType::Green, 2);
        game.players[0].mana_pool.add(ManaType::Red, 1);
        // Cost: {1}{G} — player chooses to spend Red for generic
        let cost = ManaCost::build(&[ManaType::Green], 1);
        let mut alloc = HashMap::new();
        alloc.insert(ManaType::Red, 1);
        game.pay_costs(&[Cost::Mana(cost)], 0, crate::types::ids::new_object_id(), &alloc).unwrap();
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 1);
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 0);
    }

    #[test]
    fn test_pay_life_cost() {
        let (mut game, forest_id) = setup_with_forest();
        let no_alloc = HashMap::new();
        game.pay_costs(&[Cost::PayLife(3)], 0, forest_id, &no_alloc).unwrap();
        assert_eq!(game.players[0].life_total, 17);
    }

    #[test]
    fn test_pay_life_cost_insufficient() {
        let (mut game, forest_id) = setup_with_forest();
        let no_alloc = HashMap::new();
        assert!(game.pay_costs(&[Cost::PayLife(21)], 0, forest_id, &no_alloc).is_err());
    }

    // --- Cost::Untap ({Q}) summoning sickness tests (T10 / E13) ---

    fn setup_creature_on_turn(turn: u32, keywords: Vec<crate::types::keywords::KeywordAbility>) -> (GameState, crate::types::ids::ObjectId) {
        let mut game = GameState::new(2, 20);
        game.turn_number = turn;
        let mut builder = CardDataBuilder::new("Test Creature")
            .card_type(CardType::Creature)
            .power_toughness(2, 2);
        for kw in keywords {
            builder = builder.keyword(kw);
        }
        let data = builder.build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let mut entry = BattlefieldEntity::new(id, 0, 0, turn);
        // Start tapped so {Q} (untap) is payable resource-wise
        entry.tapped = true;
        game.battlefield.insert(id, entry);
        (game, id)
    }

    #[test]
    fn test_untap_cost_blocked_by_summoning_sickness() {
        // Creature enters on turn 1, game is on turn 1 → summoning sick → can't pay {Q}
        let (mut game, creature_id) = setup_creature_on_turn(1, vec![]);
        let no_alloc = HashMap::new();
        let result = game.pay_costs(&[Cost::Untap], 0, creature_id, &no_alloc);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("summoning sickness"));
    }

    #[test]
    fn test_untap_cost_allowed_with_haste() {
        // Creature with haste enters on turn 1, game is on turn 1 → haste bypasses sickness
        let (mut game, creature_id) = setup_creature_on_turn(1, vec![crate::types::keywords::KeywordAbility::Haste]);
        let no_alloc = HashMap::new();
        game.pay_costs(&[Cost::Untap], 0, creature_id, &no_alloc).unwrap();
        assert!(!game.battlefield.get(&creature_id).unwrap().tapped);
    }

    #[test]
    fn test_untap_cost_allowed_on_noncreature() {
        // Artifact (non-creature) with {Q} cost — no summoning sickness restriction
        let mut game = GameState::new(2, 20);
        game.turn_number = 1;
        let data = CardDataBuilder::new("Test Artifact")
            .card_type(CardType::Artifact)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let mut entry = BattlefieldEntity::new(id, 0, 0, 1);
        entry.tapped = true;
        game.battlefield.insert(id, entry);

        let no_alloc = HashMap::new();
        game.pay_costs(&[Cost::Untap], 0, id, &no_alloc).unwrap();
        assert!(!game.battlefield.get(&id).unwrap().tapped);
    }

    #[test]
    fn test_untap_cost_blocked_by_control_change() {
        // Creature entered on turn 1, control changes on turn 3 → sick again on turn 3
        let (mut game, creature_id) = setup_creature_on_turn(1, vec![]);
        // Advance to turn 3 so creature is no longer sick from ETB
        game.turn_number = 3;
        // Simulate control change on turn 3
        game.battlefield.get_mut(&creature_id).unwrap().controller_since_turn = 3;
        game.battlefield.get_mut(&creature_id).unwrap().tapped = true;

        let no_alloc = HashMap::new();
        let result = game.pay_costs(&[Cost::Untap], 0, creature_id, &no_alloc);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("summoning sickness"));
    }

    #[test]
    fn test_untap_cost_allowed_control_change_haste() {
        // Creature with haste, control changes on turn 3 → haste bypasses
        let (mut game, creature_id) = setup_creature_on_turn(1, vec![crate::types::keywords::KeywordAbility::Haste]);
        game.turn_number = 3;
        game.battlefield.get_mut(&creature_id).unwrap().controller_since_turn = 3;
        game.battlefield.get_mut(&creature_id).unwrap().tapped = true;

        let no_alloc = HashMap::new();
        game.pay_costs(&[Cost::Untap], 0, creature_id, &no_alloc).unwrap();
        assert!(!game.battlefield.get(&creature_id).unwrap().tapped);
    }
}
