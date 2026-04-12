use std::collections::HashMap;

use crate::events::event::{GameEvent, LossReason};
use crate::oracle::characteristics::{get_effective_name, is_creature, get_effective_toughness};
use crate::state::game_state::GameState;
use crate::types::card_types::{CardType, Supertype};
use crate::types::effects::CounterType;
use crate::types::ids::ObjectId;
use crate::types::zones::Zone;
use crate::ui::decision::DecisionProvider;

/// State-Based Actions (rule 704)
///
/// SBAs are checked whenever a player would receive priority. They don't use
/// the stack — they just happen. If any SBA is performed, they're all checked
/// again before a player actually gets priority.

impl GameState {
    /// Check and perform all state-based actions.
    /// Returns true if any SBA was performed (caller should re-check).
    pub fn check_state_based_actions(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<bool, String> {
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
        let pw_zero_loyalty: Vec<ObjectId> = self.battlefield.keys()
            .filter(|id| {
                if let Some(obj) = self.objects.get(id) {
                    if obj.card_data.types.contains(&CardType::Planeswalker) {
                        let entry = self.battlefield.get(id).unwrap();
                        return entry.counter_count(CounterType::Loyalty) == 0;
                    }
                }
                false
            })
            .copied()
            .collect();

        for id in pw_zero_loyalty {
            let owner = self.objects.get(&id).map(|o| o.owner).unwrap_or(0);
            self.move_object(id, Zone::Graveyard)?;
            self.events.emit(GameEvent::PlaneswalkerDied { object_id: id, owner });
            self.events.emit(GameEvent::StateBasedActionPerformed);
            any_performed = true;
        }

        // 704.5j — Legend rule: if a player controls two or more legendary
        // permanents with the same name, they choose one to keep and the
        // rest are put into their owners' graveyards.
        {
            // Group legendary permanents by (controller, effective_name)
            let mut legend_groups: HashMap<(usize, String), Vec<ObjectId>> = HashMap::new();
            for (&id, entry) in &self.battlefield {
                if let Some(obj) = self.objects.get(&id) {
                    if obj.card_data.supertypes.contains(&Supertype::Legendary) {
                        let name = get_effective_name(self, id);
                        legend_groups
                            .entry((entry.controller, name))
                            .or_default()
                            .push(id);
                    }
                }
            }

            // For each group with more than one, the controller chooses one to keep
            let mut to_remove: Vec<ObjectId> = Vec::new();
            for ((_controller, _name), ids) in &legend_groups {
                if ids.len() > 1 {
                    let controller = self.battlefield.get(&ids[0]).unwrap().controller;
                    let keep = decisions.choose_legend_to_keep(self, controller, ids);
                    for &id in ids {
                        if id != keep {
                            to_remove.push(id);
                        }
                    }
                }
            }

            for id in to_remove {
                let owner = self.objects.get(&id).map(|o| o.owner).unwrap_or(0);
                self.move_object(id, Zone::Graveyard)?;
                self.events.emit(GameEvent::LegendRuleSacrificed { object_id: id, owner });
                self.events.emit(GameEvent::StateBasedActionPerformed);
                any_performed = true;
            }
        }

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
    pub fn check_state_based_actions_loop(
        &mut self,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        loop {
            if !self.check_state_based_actions(decisions)? {
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
    use crate::ui::decision::PassiveDecisionProvider;

    #[test]
    fn test_sba_lethal_damage_destroys_creature() {
        let mut game = GameState::new(2, 20);

        let bears = CardDataBuilder::new("Grizzly Bears")
            .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::Green], 1))
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
        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
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

        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
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

        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
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

        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
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

        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
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

        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
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

        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
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

        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&bears_id));
    }

    // -----------------------------------------------------------------------
    // T14: Legend rule tests (704.5j)
    // -----------------------------------------------------------------------

    #[test]
    fn test_sba_legend_rule_two_same_name() {
        // Two legendary permanents with the same name controlled by the same player.
        // SBA should remove one (the default keeps the first).
        let mut game = GameState::new(2, 20);

        let legend1_data = CardDataBuilder::new("Thalia, Guardian of Thraben")
            .card_type(CardType::Creature)
            .supertype(Supertype::Legendary)
            .power_toughness(2, 1)
            .build();
        let legend2_data = CardDataBuilder::new("Thalia, Guardian of Thraben")
            .card_type(CardType::Creature)
            .supertype(Supertype::Legendary)
            .power_toughness(2, 1)
            .build();

        let obj1 = GameObject::new(legend1_data, 0, Zone::Battlefield);
        let id1 = obj1.id;
        game.add_object(obj1);
        game.place_on_battlefield(id1, 0);

        let obj2 = GameObject::new(legend2_data, 0, Zone::Battlefield);
        let id2 = obj2.id;
        game.add_object(obj2);
        game.place_on_battlefield(id2, 0);

        // Both on the battlefield
        assert!(game.battlefield.contains_key(&id1));
        assert!(game.battlefield.contains_key(&id2));

        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
        assert!(performed);

        // Exactly one should remain, one should be in graveyard
        let on_bf = game.battlefield.contains_key(&id1) as usize
            + game.battlefield.contains_key(&id2) as usize;
        assert_eq!(on_bf, 1);
        assert_eq!(game.players[0].graveyard.len(), 1);
    }

    #[test]
    fn test_sba_legend_rule_different_names_ok() {
        // Two legendary permanents with DIFFERENT names — no SBA.
        let mut game = GameState::new(2, 20);

        let legend1 = CardDataBuilder::new("Thalia, Guardian of Thraben")
            .card_type(CardType::Creature)
            .supertype(Supertype::Legendary)
            .power_toughness(2, 1)
            .build();
        let legend2 = CardDataBuilder::new("Isamaru, Hound of Konda")
            .card_type(CardType::Creature)
            .supertype(Supertype::Legendary)
            .power_toughness(2, 1)
            .build();

        let obj1 = GameObject::new(legend1, 0, Zone::Battlefield);
        let id1 = obj1.id;
        game.add_object(obj1);
        game.place_on_battlefield(id1, 0);

        let obj2 = GameObject::new(legend2, 0, Zone::Battlefield);
        let id2 = obj2.id;
        game.add_object(obj2);
        game.place_on_battlefield(id2, 0);

        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&id1));
        assert!(game.battlefield.contains_key(&id2));
    }

    #[test]
    fn test_sba_legend_rule_different_controllers_ok() {
        // Two legendary permanents with the SAME name but different controllers — no SBA.
        let mut game = GameState::new(2, 20);

        let data1 = CardDataBuilder::new("Thalia, Guardian of Thraben")
            .card_type(CardType::Creature)
            .supertype(Supertype::Legendary)
            .power_toughness(2, 1)
            .build();
        let data2 = CardDataBuilder::new("Thalia, Guardian of Thraben")
            .card_type(CardType::Creature)
            .supertype(Supertype::Legendary)
            .power_toughness(2, 1)
            .build();

        let obj1 = GameObject::new(data1, 0, Zone::Battlefield);
        let id1 = obj1.id;
        game.add_object(obj1);
        game.place_on_battlefield(id1, 0); // controller = player 0

        let obj2 = GameObject::new(data2, 1, Zone::Battlefield);
        let id2 = obj2.id;
        game.add_object(obj2);
        game.place_on_battlefield(id2, 1); // controller = player 1

        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&id1));
        assert!(game.battlefield.contains_key(&id2));
    }

    // -----------------------------------------------------------------------
    // T14: Planeswalker loyalty tests (704.5i)
    // -----------------------------------------------------------------------

    #[test]
    fn test_sba_planeswalker_zero_loyalty_dies() {
        // A planeswalker with 0 loyalty counters should be put into graveyard by SBA.
        let mut game = GameState::new(2, 20);

        let pw_data = CardDataBuilder::new("Jace, the Mind Sculptor")
            .card_type(CardType::Planeswalker)
            .loyalty(3)
            .build();

        let obj = GameObject::new(pw_data, 0, Zone::Battlefield);
        let pw_id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(pw_id, 0);

        // Verify ETB set loyalty counters
        assert_eq!(
            game.battlefield.get(&pw_id).unwrap()
                .counter_count(crate::types::effects::CounterType::Loyalty),
            3
        );

        // Remove all loyalty counters to simulate damage
        game.battlefield.get_mut(&pw_id).unwrap()
            .remove_counters(crate::types::effects::CounterType::Loyalty, 3);
        assert_eq!(
            game.battlefield.get(&pw_id).unwrap()
                .counter_count(crate::types::effects::CounterType::Loyalty),
            0
        );

        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
        assert!(performed);
        assert!(!game.battlefield.contains_key(&pw_id));
        assert_eq!(game.get_object(pw_id).unwrap().zone, Zone::Graveyard);
    }

    #[test]
    fn test_sba_planeswalker_with_loyalty_stays() {
        // A planeswalker with positive loyalty should NOT be affected by SBA.
        let mut game = GameState::new(2, 20);

        let pw_data = CardDataBuilder::new("Jace, the Mind Sculptor")
            .card_type(CardType::Planeswalker)
            .loyalty(3)
            .build();

        let obj = GameObject::new(pw_data, 0, Zone::Battlefield);
        let pw_id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(pw_id, 0);

        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
        assert!(!performed);
        assert!(game.battlefield.contains_key(&pw_id));
        assert_eq!(
            game.battlefield.get(&pw_id).unwrap()
                .counter_count(crate::types::effects::CounterType::Loyalty),
            3
        );
    }

    #[test]
    fn test_planeswalker_etb_sets_loyalty_counters() {
        // When a planeswalker enters the battlefield, it should get loyalty counters
        // equal to its printed loyalty (rule 306.5b / ATOM-209.1-001).
        let mut game = GameState::new(2, 20);

        let pw_data = CardDataBuilder::new("Liliana of the Veil")
            .card_type(CardType::Planeswalker)
            .loyalty(3)
            .build();

        let obj = GameObject::new(pw_data, 0, Zone::Battlefield);
        let pw_id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(pw_id, 0);

        let entry = game.battlefield.get(&pw_id).unwrap();
        assert_eq!(entry.counter_count(crate::types::effects::CounterType::Loyalty), 3);
    }

    #[test]
    fn test_planeswalker_zero_printed_loyalty_dies_immediately() {
        // A planeswalker with 0 printed loyalty enters with 0 loyalty counters.
        // The SBA should immediately put it into the graveyard.
        let mut game = GameState::new(2, 20);

        let pw_data = CardDataBuilder::new("Tibalt, the Zero")
            .card_type(CardType::Planeswalker)
            .loyalty(0)
            .build();

        let obj = GameObject::new(pw_data, 0, Zone::Battlefield);
        let pw_id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(pw_id, 0);

        // Should have 0 loyalty counters (loyalty(0) → guard skips adding)
        assert_eq!(
            game.battlefield.get(&pw_id).unwrap()
                .counter_count(crate::types::effects::CounterType::Loyalty),
            0
        );

        let performed = game.check_state_based_actions(&PassiveDecisionProvider).unwrap();
        assert!(performed);
        assert!(!game.battlefield.contains_key(&pw_id));
        assert_eq!(game.get_object(pw_id).unwrap().zone, Zone::Graveyard);
    }
}
