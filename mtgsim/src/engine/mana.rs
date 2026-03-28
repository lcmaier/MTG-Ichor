use std::collections::HashMap;

use crate::objects::card_data::{AbilityType, Effect};
use crate::state::game_state::GameState;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};

/// Mana ability engine (rule 605).
///
/// Mana abilities are special: they don't use the stack and resolve immediately.
/// This module handles activating mana abilities on permanents. Cost payment
/// is delegated to the shared `engine::costs::pay_costs` system.
///
/// Note: Complex mana abilities (e.g. Metalworker) that involve choices or
/// non-mana effects will eventually go through the general effect resolution
/// pipeline, just without using the stack.

impl GameState {
    /// Activate a mana ability on a permanent.
    ///
    /// Mana abilities resolve immediately (they don't go on the stack).
    /// Cost payment is handled by the shared `pay_costs` system.
    pub fn activate_mana_ability(
        &mut self,
        player_id: PlayerId,
        permanent_id: ObjectId,
        ability_id: AbilityId,
    ) -> Result<(), String> {
        // Snapshot the ability definition (clone to release borrow)
        let obj = self.get_object(permanent_id)?;
        let card_data = obj.card_data.clone();

        let ability = card_data.abilities.iter()
            .find(|a| a.id == ability_id)
            .ok_or_else(|| format!("Ability {} not found on permanent {}", ability_id, permanent_id))?;

        if ability.ability_type != AbilityType::Mana {
            return Err("This is not a mana ability".to_string());
        }

        // Verify controller
        let entry = self.battlefield.get(&permanent_id)
            .ok_or_else(|| format!("Permanent {} not on battlefield", permanent_id))?;
        if entry.controller != player_id {
            return Err("You don't control this permanent".to_string());
        }

        // Pay costs via shared cost payment system.
        // Mana ability costs are always specific (tap, etc.) — no generic allocation needed.
        let no_generic = HashMap::new();
        self.pay_costs(&ability.costs, player_id, permanent_id, &no_generic)?;

        // Resolve effect immediately (mana abilities don't use the stack)
        self.resolve_mana_effect(&ability.effect, player_id)?;

        Ok(())
    }

    /// Resolve the effect of a mana ability.
    ///
    /// Currently handles ProduceMana directly. As the effect system grows,
    /// complex mana abilities (Metalworker, Selvala) will route through the
    /// general effect resolution pipeline instead.
    fn resolve_mana_effect(
        &mut self,
        effect: &Effect,
        player_id: PlayerId,
    ) -> Result<(), String> {
        match effect {
            Effect::ProduceMana { mana } => {
                let player = self.get_player_mut(player_id)?;
                for (mana_type, amount) in mana {
                    player.mana_pool.add(*mana_type, *amount);
                }
                Ok(())
            }
            Effect::Sequence(effects) => {
                for sub_effect in effects {
                    self.resolve_mana_effect(sub_effect, player_id)?;
                }
                Ok(())
            }
            _ => Err(format!("Unsupported effect in mana ability: {:?}", effect)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::BattlefieldEntity;
    use crate::state::game_state::GameState;
    use crate::types::card_types::*;
    use crate::types::mana::ManaType;
    use crate::types::zones::Zone;

    fn setup_with_forest() -> (GameState, crate::types::ids::ObjectId, crate::types::ids::AbilityId) {
        let mut game = GameState::new(2, 20);

        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Forest))
            .mana_ability_single(ManaType::Green)
            .build();

        let ability_id = forest.abilities[0].id;

        let obj = GameObject::new(forest, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let mut entry = BattlefieldEntity::new(id, 0);
        entry.summoning_sick = false;
        game.battlefield.insert(id, entry);

        (game, id, ability_id)
    }

    #[test]
    fn test_activate_forest_mana_ability() {
        let (mut game, forest_id, ability_id) = setup_with_forest();

        assert_eq!(game.players[0].mana_pool.total(), 0);

        game.activate_mana_ability(0, forest_id, ability_id).unwrap();

        assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 1);
        assert!(game.battlefield.get(&forest_id).unwrap().tapped);
    }

    #[test]
    fn test_cannot_activate_already_tapped() {
        let (mut game, forest_id, ability_id) = setup_with_forest();

        game.activate_mana_ability(0, forest_id, ability_id).unwrap();
        let result = game.activate_mana_ability(0, forest_id, ability_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_activate_opponents_permanent() {
        let (mut game, forest_id, ability_id) = setup_with_forest();

        let result = game.activate_mana_ability(1, forest_id, ability_id);
        assert!(result.is_err());
    }
}
