use crate::events::event::{GameEvent, LossReason};
use crate::oracle::characteristics::{is_creature, get_effective_toughness};
use crate::state::game_state::GameState;
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
        for i in 0..self.players.len() {
            if self.players[i].life_total <= 0 && !self.player_lost[i] {
                self.player_lost[i] = true;
                self.events.emit(GameEvent::PlayerLost {
                    player_id: i,
                    reason: LossReason::LifeReachedZero,
                });
                self.events.emit(GameEvent::StateBasedActionPerformed);
                any_performed = true;
            }
        }

        // 704.5b — Player who attempted to draw from empty library loses
        for i in 0..self.players.len() {
            if self.players[i].has_drawn_from_empty_library && !self.player_lost[i] {
                self.player_lost[i] = true;
                self.events.emit(GameEvent::PlayerLost {
                    player_id: i,
                    reason: LossReason::DrawnFromEmptyLibrary,
                });
                self.events.emit(GameEvent::StateBasedActionPerformed);
                any_performed = true;
            }
        }

        // 704.5f — Creature with toughness 0 or less is put into owner's graveyard
        let zero_toughness: Vec<ObjectId> = self.battlefield.keys()
            .filter(|id| {
                if is_creature(self, **id) {
                    let effective_t = get_effective_toughness(self, **id).unwrap_or(0);
                    return effective_t <= 0;
                }
                false
            })
            .copied()
            .collect();

        for id in zero_toughness {
            let owner = self.objects.get(&id).map(|o| o.owner).unwrap_or(0);
            self.move_object(id, Zone::Graveyard)?;
            self.events.emit(GameEvent::CreatureDied { creature_id: id, owner });
            any_performed = true;
        }

        // 704.5g — Creature with lethal damage is destroyed
        // Also handles deathtouch (rule 702.2b): any nonzero damage from a
        // deathtouch source is lethal.
        let lethal_damage: Vec<ObjectId> = self.battlefield.keys()
            .filter(|id| {
                if is_creature(self, **id) {
                    let effective_t = get_effective_toughness(self, **id).unwrap_or(0);
                    if effective_t <= 0 { return false; } // handled by 704.5f
                    let entry = self.battlefield.get(id).unwrap();
                    // Normal lethal damage OR any damage from deathtouch source
                    return entry.damage_marked >= effective_t as u32
                        || (entry.damage_marked > 0 && entry.damaged_by_deathtouch);
                }
                false
            })
            .copied()
            .collect();

        for id in lethal_damage {
            let owner = self.objects.get(&id).map(|o| o.owner).unwrap_or(0);
            // TODO: check for indestructible / regeneration
            self.move_object(id, Zone::Graveyard)?;
            self.events.emit(GameEvent::CreatureDied { creature_id: id, owner });
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
        game.place_on_battlefield(bears_id, 0).damage_marked = 2; // lethal for a 2/2

        // SBA should destroy the creature
        let performed = game.check_state_based_actions().unwrap();
        assert!(performed);
        assert!(!game.battlefield.contains_key(&bears_id));
        assert_eq!(game.players[0].graveyard.len(), 1);
        assert_eq!(game.get_object(bears_id).unwrap().zone, Zone::Graveyard);
    }

    #[test]
    fn test_sba_deathtouch_damage_destroys_creature() {
        let mut game = GameState::new(2, 20);

        // 4/5 creature with 1 damage from deathtouch source
        let data = CardDataBuilder::new("Earth Elemental")
            .card_type(CardType::Creature)
            .power_toughness(4, 5)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let bf = game.place_on_battlefield(id, 0);
        bf.damage_marked = 1; // only 1 damage
        bf.damaged_by_deathtouch = true; // but from deathtouch

        let performed = game.check_state_based_actions().unwrap();
        assert!(performed);
        assert!(!game.battlefield.contains_key(&id));
        assert_eq!(game.get_object(id).unwrap().zone, Zone::Graveyard);
    }

    #[test]
    fn test_sba_deathtouch_zero_damage_no_destroy() {
        let mut game = GameState::new(2, 20);

        // Creature with deathtouch flag but 0 damage (shouldn't happen normally,
        // but verify the guard)
        let data = CardDataBuilder::new("Earth Elemental")
            .card_type(CardType::Creature)
            .power_toughness(4, 5)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let bf = game.place_on_battlefield(id, 0);
        bf.damage_marked = 0;
        bf.damaged_by_deathtouch = true;

        let performed = game.check_state_based_actions().unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&id));
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
        game.place_on_battlefield(bears_id, 0);

        let performed = game.check_state_based_actions().unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&bears_id));
    }
}
