use crate::state::game_state::GameState;
use crate::types::card_types::CardType;
use crate::types::ids::ObjectId;
use crate::types::zones::Zone;

/// State-Based Actions (rule 704)
///
/// SBAs are checked whenever a player would receive priority. They don't use
/// the stack — they just happen. If any SBA is performed, they're all checked
/// again before a player actually gets priority.

impl GameState {
    /// Check and perform all state-based actions.
    /// Returns true if any SBA was performed (caller should re-check).
    pub fn check_state_based_actions(&mut self) -> Result<bool, String> {
        let mut any_performed = false;

        // 704.5a — Player with 0 or less life loses the game
        // (game loss not yet implemented, just log for now)
        for player in &self.players {
            if player.life_total <= 0 {
                println!("SBA: Player {} has {} life (would lose the game)", player.id, player.life_total);
                // TODO: handle game loss
            }
        }

        // 704.5b — Player who attempted to draw from empty library loses
        for player in &self.players {
            if player.has_drawn_from_empty_library {
                println!("SBA: Player {} drew from empty library (would lose the game)", player.id);
                // TODO: handle game loss
            }
        }

        // 704.5f — Creature with toughness 0 or less is put into owner's graveyard
        let zero_toughness: Vec<ObjectId> = self.battlefield.keys()
            .filter(|id| {
                if let Some(obj) = self.objects.get(id) {
                    if obj.card_data.types.contains(&CardType::Creature) {
                        let entry = self.battlefield.get(id).unwrap();
                        let base_t = obj.card_data.toughness.unwrap_or(0);
                        let effective_t = base_t + entry.toughness_modifier;
                        return effective_t <= 0;
                    }
                }
                false
            })
            .copied()
            .collect();

        for id in zero_toughness {
            let name = self.objects.get(&id).map(|o| o.card_data.name.clone()).unwrap_or_default();
            println!("SBA 704.5f: {} has 0 or less toughness, moved to graveyard", name);
            self.move_object(id, Zone::Graveyard)?;
            any_performed = true;
        }

        // 704.5g — Creature with lethal damage is destroyed
        let lethal_damage: Vec<ObjectId> = self.battlefield.keys()
            .filter(|id| {
                if let Some(obj) = self.objects.get(id) {
                    if obj.card_data.types.contains(&CardType::Creature) {
                        let entry = self.battlefield.get(id).unwrap();
                        let base_t = obj.card_data.toughness.unwrap_or(0);
                        let effective_t = base_t + entry.toughness_modifier;
                        return effective_t > 0 && entry.damage_marked >= effective_t as u32;
                    }
                }
                false
            })
            .copied()
            .collect();

        for id in lethal_damage {
            let name = self.objects.get(&id).map(|o| o.card_data.name.clone()).unwrap_or_default();
            println!("SBA 704.5g: {} has lethal damage, destroyed", name);
            // TODO: check for indestructible / regeneration
            self.move_object(id, Zone::Graveyard)?;
            any_performed = true;
        }

        // 704.5i — Planeswalker with 0 loyalty is put into owner's graveyard
        // 704.5j — Legend rule
        // 704.5k — World rule
        // (future SBAs added here as needed)

        Ok(any_performed)
    }

    /// Repeatedly check SBAs until none are performed (rule 704.3)
    pub fn check_state_based_actions_loop(&mut self) -> Result<(), String> {
        loop {
            if !self.check_state_based_actions()? {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::BattlefieldEntity;
    use crate::state::game_state::GameState;
    use crate::types::card_types::*;
    use crate::types::colors::Color;
    use crate::types::mana::ManaType;
    use crate::types::zones::Zone;

    #[test]
    fn test_sba_lethal_damage_destroys_creature() {
        let mut game = GameState::new(2, 20);

        let bears = CardDataBuilder::new("Grizzly Bears")
            .mana_cost(crate::types::mana::ManaCost::single(ManaType::Green, 1, 1))
            .color(Color::Green)
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Bear))
            .power_toughness(2, 2)
            .build();

        let obj = GameObject::new(bears, 0, Zone::Battlefield);
        let bears_id = obj.id;
        game.add_object(obj);
        let mut entry = BattlefieldEntity::new(bears_id, 0, 0);
        entry.summoning_sick = false;
        entry.damage_marked = 2; // lethal for a 2/2
        game.battlefield.insert(bears_id, entry);

        // SBA should destroy the creature
        let performed = game.check_state_based_actions().unwrap();
        assert!(performed);
        assert!(!game.battlefield.contains_key(&bears_id));
        assert_eq!(game.players[0].graveyard.len(), 1);
        assert_eq!(game.get_object(bears_id).unwrap().zone, Zone::Graveyard);
    }

    #[test]
    fn test_sba_no_action_when_healthy() {
        let mut game = GameState::new(2, 20);

        let bears = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Bear))
            .power_toughness(2, 2)
            .build();

        let obj = GameObject::new(bears, 0, Zone::Battlefield);
        let bears_id = obj.id;
        game.add_object(obj);
        let entry = BattlefieldEntity::new(bears_id, 0, 0);
        game.battlefield.insert(bears_id, entry);

        let performed = game.check_state_based_actions().unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&bears_id));
    }
}
