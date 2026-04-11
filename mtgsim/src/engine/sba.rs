use crate::events::event::{GameEvent, LossReason};
use crate::oracle::characteristics::{is_creature, get_effective_toughness};
use crate::state::game_state::GameState;
use crate::types::effects::CounterType;
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

        // 704.5q — +1/+1 and -1/-1 counter annihilation
        // If a permanent has both +1/+1 and -1/-1 counters, remove pairs
        // until only one type remains.
        let annihilation_targets: Vec<(ObjectId, u32)> = self.battlefield.iter()
            .filter_map(|(&id, entry)| {
                let plus = entry.counter_count(CounterType::PlusOnePlusOne);
                let minus = entry.counter_count(CounterType::MinusOneMinusOne);
                if plus > 0 && minus > 0 {
                    Some((id, plus.min(minus)))
                } else {
                    None
                }
            })
            .collect();

        for (id, pairs) in annihilation_targets {
            if let Some(entry) = self.battlefield.get_mut(&id) {
                entry.remove_counters(CounterType::PlusOnePlusOne, pairs);
                entry.remove_counters(CounterType::MinusOneMinusOne, pairs);
            }
            self.events.emit(GameEvent::CountersAnnihilated { object_id: id, pairs_removed: pairs });
            self.events.emit(GameEvent::StateBasedActionPerformed);
            any_performed = true;
        }

        // 704.5d — Token in a non-battlefield zone ceases to exist
        // Tokens cease to exist — they are removed from the game entirely.
        // This is NOT a zone change (no death trigger, no ZoneChange event).
        let tokens_to_remove: Vec<(ObjectId, Zone)> = self.objects.iter()
            .filter(|(_, obj)| obj.is_token && obj.zone != Zone::Battlefield)
            .map(|(&id, obj)| (id, obj.zone))
            .collect();

        for (id, zone) in tokens_to_remove {
            // Remove from zone collection (reuse the centralized helper;
            // stack_entries cleanup is handled internally)
            self.remove_from_zone_collection(id, zone)?;
            // Remove from central object store
            self.objects.remove(&id);
            self.events.emit(GameEvent::TokenCeasedToExist { object_id: id });
            self.events.emit(GameEvent::StateBasedActionPerformed);
            any_performed = true;
        }

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
    fn test_sba_counter_annihilation() {
        // Permanent with 3 +1/+1 and 2 -1/-1 → ends with 1 +1/+1 and 0 -1/-1
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Bear))
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = game.place_on_battlefield(id, 0);
        entry.add_counters(crate::types::effects::CounterType::PlusOnePlusOne, 3);
        entry.add_counters(crate::types::effects::CounterType::MinusOneMinusOne, 2);

        let performed = game.check_state_based_actions().unwrap();
        assert!(performed);

        let entry = game.battlefield.get(&id).unwrap();
        assert_eq!(entry.counter_count(crate::types::effects::CounterType::PlusOnePlusOne), 1);
        assert_eq!(entry.counter_count(crate::types::effects::CounterType::MinusOneMinusOne), 0);
    }

    #[test]
    fn test_sba_counter_annihilation_equal() {
        // Equal counts → both zeroed
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Bear))
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = game.place_on_battlefield(id, 0);
        entry.add_counters(crate::types::effects::CounterType::PlusOnePlusOne, 4);
        entry.add_counters(crate::types::effects::CounterType::MinusOneMinusOne, 4);

        let performed = game.check_state_based_actions().unwrap();
        assert!(performed);

        let entry = game.battlefield.get(&id).unwrap();
        assert_eq!(entry.counter_count(crate::types::effects::CounterType::PlusOnePlusOne), 0);
        assert_eq!(entry.counter_count(crate::types::effects::CounterType::MinusOneMinusOne), 0);
    }

    #[test]
    fn test_sba_token_ceases_to_exist_in_graveyard() {
        // Token in graveyard is removed from the game entirely
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Goblin Token")
            .card_type(CardType::Creature)
            .power_toughness(1, 1)
            .build();
        let mut obj = GameObject::new(data, 0, Zone::Graveyard);
        obj.is_token = true;
        let id = obj.id;
        game.add_object(obj);
        game.players[0].graveyard.push(id);

        let performed = game.check_state_based_actions().unwrap();
        assert!(performed);

        // Token should be completely removed from the game
        assert!(game.objects.get(&id).is_none());
        assert!(!game.players[0].graveyard.contains(&id));

        // Should have emitted TokenCeasedToExist event
        let has_event = game.events.events().iter().any(|e| {
            matches!(e, crate::events::event::GameEvent::TokenCeasedToExist { object_id } if *object_id == id)
        });
        assert!(has_event);
    }

    #[test]
    fn test_sba_token_on_battlefield_stays() {
        // Token on battlefield should NOT be removed
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Goblin Token")
            .card_type(CardType::Creature)
            .power_toughness(1, 1)
            .build();
        let mut obj = GameObject::new(data, 0, Zone::Battlefield);
        obj.is_token = true;
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        let performed = game.check_state_based_actions().unwrap();
        assert!(!performed);

        // Token should still exist
        assert!(game.objects.get(&id).is_some());
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
