use std::collections::HashMap;

use crate::types::costs::{AdditionalCost, AlternativeCost, Cost};
use crate::oracle::characteristics::{has_summoning_sickness, is_creature};
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};

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

/// Assemble the total cost to cast a spell (rule 601.2f).
///
/// Starts with the base cost (either the card's mana cost or an alternative
/// cost), adds X mana (x_value * x_count generic symbols), appends any
/// additional costs chosen by the player, then passes through the cost
/// modification pipeline.
///
/// Returns the assembled `Vec<Cost>` ready for payment.
pub fn assemble_total_cost(
    base_mana_cost: &ManaCost,
    chosen_alt_cost: Option<&AlternativeCost>,
    chosen_additional_costs: &[&AdditionalCost],
    x_value: u64,
) -> Vec<Cost> {
    // Step 1: Determine the base cost (normal mana cost or alternative cost).
    // Alternative costs replace the mana cost entirely (rule 118.9a).
    let mut base_costs: Vec<Cost> = if let Some(alt) = chosen_alt_cost {
        alt.costs().to_vec()
    } else {
        // Normal path: start with the card's mana cost, expanding X symbols
        // into concrete generic mana (x_value * x_count).
        let x_count = base_mana_cost.x_count();
        let mut symbols: Vec<ManaSymbol> = base_mana_cost.symbols
            .iter()
            .filter(|s| !matches!(s, ManaSymbol::X))
            .copied()
            .collect();

        let x_generic_total = x_value * (x_count as u64);
        for _ in 0..x_generic_total {
            symbols.push(ManaSymbol::Generic);
        }

        vec![Cost::Mana(ManaCost::from_symbols(symbols))]
    };

    // Step 2: Append additional costs unconditionally (rule 118.8).
    // Additional costs layer on top of whichever base was chosen.
    for additional in chosen_additional_costs {
        base_costs.extend(additional.costs().iter().cloned());
    }

    // Step 3: Cost modification pipeline (increases, reductions, floors).
    apply_cost_modifications(base_costs)
}

/// Cost modification pipeline stub (rule 601.2f).
///
/// In the full implementation (L15, Phase 5 Layers), this applies:
/// 1. **Increases** — Thalia, Guardian of Thraben; Sphere of Resistance
/// 2. **Reductions** — Goblin Electromancer; Helm of Awakening
/// 3. **Trinisphere floor** — total mana cost cannot be less than 3
/// 4. **Lock** — final cost is locked, no further modifications
///
/// For now, this is a passthrough: returns costs unmodified.
fn apply_cost_modifications(costs: Vec<Cost>) -> Vec<Cost> {
    // TODO(L15): Wire continuous effects layer here.
    // The pipeline should:
    //   1. Collect all CostModification effects from the continuous effects layer
    //   2. Sort by dependency (increases before reductions before floors)
    //   3. Apply each to the mana component of `costs`
    //   4. Return modified costs
    costs
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

    // --- assemble_total_cost tests (T18a) ---

    #[test]
    fn test_assemble_normal_mana_cost() {
        // {1}{R} with no alt/additional/X → [Cost::Mana({1}{R})]
        let base = ManaCost::build(&[ManaType::Red], 1);
        let result = assemble_total_cost(&base, None, &[], 0);
        assert_eq!(result.len(), 1);
        if let Cost::Mana(mc) = &result[0] {
            assert_eq!(mc.mana_value(), 2);
            assert_eq!(mc.colored_count(ManaType::Red), 1);
            assert_eq!(mc.generic_count(), 1);
        } else {
            panic!("Expected Cost::Mana");
        }
    }

    #[test]
    fn test_assemble_x_cost_single() {
        // {X}{R} with X=3 → Cost::Mana with {R} + 3 generic = MV 4
        let base = ManaCost::from_symbols(vec![
            crate::types::mana::ManaSymbol::X,
            crate::types::mana::ManaSymbol::Colored(ManaType::Red),
        ]);
        let result = assemble_total_cost(&base, None, &[], 3);
        assert_eq!(result.len(), 1);
        if let Cost::Mana(mc) = &result[0] {
            assert_eq!(mc.colored_count(ManaType::Red), 1);
            assert_eq!(mc.generic_count(), 3);
            assert_eq!(mc.x_count(), 0); // X symbols are removed
            assert_eq!(mc.mana_value(), 4);
        } else {
            panic!("Expected Cost::Mana");
        }
    }

    #[test]
    fn test_assemble_x_cost_zero() {
        // {X}{R} with X=0 → Cost::Mana({R}) only
        let base = ManaCost::from_symbols(vec![
            crate::types::mana::ManaSymbol::X,
            crate::types::mana::ManaSymbol::Colored(ManaType::Red),
        ]);
        let result = assemble_total_cost(&base, None, &[], 0);
        assert_eq!(result.len(), 1);
        if let Cost::Mana(mc) = &result[0] {
            assert_eq!(mc.colored_count(ManaType::Red), 1);
            assert_eq!(mc.generic_count(), 0);
            assert_eq!(mc.mana_value(), 1);
        } else {
            panic!("Expected Cost::Mana");
        }
    }

    #[test]
    fn test_assemble_double_x_cost() {
        // {X}{X} with X=2 → Cost::Mana with 4 generic symbols
        let base = ManaCost::from_symbols(vec![
            crate::types::mana::ManaSymbol::X,
            crate::types::mana::ManaSymbol::X,
        ]);
        let result = assemble_total_cost(&base, None, &[], 2);
        assert_eq!(result.len(), 1);
        if let Cost::Mana(mc) = &result[0] {
            assert_eq!(mc.generic_count(), 4); // 2 * 2 = 4
            assert_eq!(mc.x_count(), 0);
        } else {
            panic!("Expected Cost::Mana");
        }
    }

    #[test]
    fn test_assemble_alternative_cost() {
        use crate::types::costs::AlternativeCost;
        // Alternative cost: pay 1 life instead of mana
        let base = ManaCost::build(&[ManaType::Red], 2);
        let alt = AlternativeCost::Custom(
            "Pay 1 life".to_string(),
            vec![Cost::PayLife(1)],
        );
        let result = assemble_total_cost(&base, Some(&alt), &[], 0);
        // Should contain just PayLife(1), not the original mana cost
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Cost::PayLife(1));
    }

    #[test]
    fn test_assemble_additional_cost_kicker() {
        use crate::types::costs::AdditionalCost;
        // {1}{R} + kicker {R} → [Cost::Mana({1}{R}), Cost::Mana({R})]
        let base = ManaCost::build(&[ManaType::Red], 1);
        let kicker = AdditionalCost::Kicker(vec![
            Cost::Mana(ManaCost::build(&[ManaType::Red], 0)),
        ]);
        let result = assemble_total_cost(&base, None, &[&kicker], 0);
        assert_eq!(result.len(), 2);
        // First: base mana cost
        if let Cost::Mana(mc) = &result[0] {
            assert_eq!(mc.mana_value(), 2);
        } else {
            panic!("Expected Cost::Mana for base");
        }
        // Second: kicker mana cost
        if let Cost::Mana(mc) = &result[1] {
            assert_eq!(mc.colored_count(ManaType::Red), 1);
        } else {
            panic!("Expected Cost::Mana for kicker");
        }
    }

    #[test]
    fn test_assemble_alt_plus_additional() {
        use crate::types::costs::{AlternativeCost, AdditionalCost};
        // Alt cost: pay 2 life. Additional: kicker {G}.
        // Total = [PayLife(2), Cost::Mana({G})]
        let base = ManaCost::build(&[ManaType::Red], 3);
        let alt = AlternativeCost::Custom(
            "Pay 2 life".to_string(),
            vec![Cost::PayLife(2)],
        );
        let kicker = AdditionalCost::Kicker(vec![
            Cost::Mana(ManaCost::build(&[ManaType::Green], 0)),
        ]);
        let result = assemble_total_cost(&base, Some(&alt), &[&kicker], 0);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Cost::PayLife(2));
        if let Cost::Mana(mc) = &result[1] {
            assert_eq!(mc.colored_count(ManaType::Green), 1);
        } else {
            panic!("Expected Cost::Mana for kicker");
        }
    }

    #[test]
    fn test_assemble_cost_modification_passthrough() {
        // Verify that apply_cost_modifications is a passthrough (stub)
        let base = ManaCost::build(&[ManaType::Red], 1);
        let result = assemble_total_cost(&base, None, &[], 0);
        // Result should be identical to what we'd get without modification
        assert_eq!(result.len(), 1);
        if let Cost::Mana(mc) = &result[0] {
            assert_eq!(mc.mana_value(), 2);
        } else {
            panic!("Expected Cost::Mana");
        }
    }
}
