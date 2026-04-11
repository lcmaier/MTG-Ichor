// Read-only game state queries for object characteristics.
//
// These functions inspect GameState and return derived information about
// an object's effective characteristics. Phase 5 will replace the
// implementations with layer-system-aware lookups; all consumers call
// these functions so the transition is a single-point change.

use crate::state::game_state::GameState;
use crate::types::card_types::CardType;
use crate::types::ids::ObjectId;
use crate::types::keywords::KeywordAbility;

/// Check if a permanent has an effective keyword ability.
/// Phase 4: reads printed keywords from card_data.
/// Phase 5: layer-system-aware (granted/removed keywords from continuous effects).
pub fn has_keyword(game: &GameState, id: ObjectId, keyword: KeywordAbility) -> bool {
    game.objects.get(&id)
        .map(|obj| obj.card_data.keywords.contains(&keyword))
        .unwrap_or(false)
}

/// Check if an object on the battlefield is currently a creature.
/// Phase 3: reads printed types. Phase 5: reads effective types from layer system.
pub fn is_creature(game: &GameState, id: ObjectId) -> bool {
    game.objects.get(&id)
        .map(|obj| obj.card_data.types.contains(&CardType::Creature))
        .unwrap_or(false)
}

/// Check if a permanent has summoning sickness.
/// A permanent has summoning sickness if its controller gained control of it
/// on the current turn (controller_since_turn >= turn_number) and it doesn't
/// have haste. Convention: controller_since_turn = 0 is a pregame sentinel
/// (rule 103.6), so 0 >= 1 is false → not sick.
pub fn has_summoning_sickness(game: &GameState, id: ObjectId) -> bool {
    if let Some(entry) = game.battlefield.get(&id) {
        if entry.controller_since_turn >= game.turn_number {
            !has_keyword(game, id, KeywordAbility::Haste)
        } else {
            false
        }
    } else {
        false
    }
}

/// Get effective power for a creature on the battlefield.
/// Phase 3: base + modifier. Phase 5: computed through layer system.
pub fn get_effective_power(game: &GameState, id: ObjectId) -> Option<i32> {
    let obj = game.objects.get(&id)?;
    let entry = game.battlefield.get(&id)?;
    let base = obj.card_data.power?;
    Some(base + entry.power_modifier)
}

/// Get effective toughness for a creature on the battlefield.
/// Phase 3: base + modifier. Phase 5: computed through layer system.
pub fn get_effective_toughness(game: &GameState, id: ObjectId) -> Option<i32> {
    let obj = game.objects.get(&id)?;
    let entry = game.battlefield.get(&id)?;
    let base = obj.card_data.toughness?;
    Some(base + entry.toughness_modifier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::types::colors::Color;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;

    #[test]
    fn test_has_keyword_true() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Serra Angel")
            .card_type(CardType::Creature)
            .color(Color::White)
            .mana_cost(ManaCost::single(ManaType::White, 2, 3))
            .power_toughness(4, 4)
            .keyword(KeywordAbility::Flying)
            .keyword(KeywordAbility::Vigilance)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);

        assert!(has_keyword(&game, id, KeywordAbility::Flying));
        assert!(has_keyword(&game, id, KeywordAbility::Vigilance));
    }

    #[test]
    fn test_has_keyword_false() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .mana_cost(ManaCost::single(ManaType::Green, 1, 1))
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);

        assert!(!has_keyword(&game, id, KeywordAbility::Flying));
        assert!(!has_keyword(&game, id, KeywordAbility::Haste));
        assert!(!has_keyword(&game, id, KeywordAbility::Trample));
    }

    #[test]
    fn test_has_keyword_nonexistent_object() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        assert!(!has_keyword(&game, fake_id, KeywordAbility::Flying));
    }

    #[test]
    fn test_is_creature_true() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);

        assert!(is_creature(&game, id));
    }

    #[test]
    fn test_is_creature_false_for_land() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);

        assert!(!is_creature(&game, id));
    }

    #[test]
    fn test_get_effective_power_base() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        assert_eq!(get_effective_power(&game, id), Some(2));
    }

    #[test]
    fn test_get_effective_power_with_modifier() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0).power_modifier = 3;

        assert_eq!(get_effective_power(&game, id), Some(5));
    }

    #[test]
    fn test_get_effective_toughness_base() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        assert_eq!(get_effective_toughness(&game, id), Some(2));
    }

    #[test]
    fn test_get_effective_toughness_nonexistent() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        assert_eq!(get_effective_toughness(&game, fake_id), None);
    }
}
