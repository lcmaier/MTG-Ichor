// Read-only legality queries — can a creature attack, block, etc.

use crate::oracle::characteristics::has_keyword;
use crate::state::game_state::GameState;
use crate::types::ids::ObjectId;
use crate::types::keywords::KeywordAbility;

/// Check if a creature can attack (not summoning-sick, or has haste).
/// Rule 702.10b: Haste bypasses summoning sickness for attacking.
pub fn can_attack(game: &GameState, id: ObjectId) -> bool {
    if let Some(entry) = game.battlefield.get(&id) {
        !entry.summoning_sick || has_keyword(game, id, KeywordAbility::Haste)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::BattlefieldEntity;
    use crate::types::card_types::CardType;
    use crate::types::zones::Zone;

    #[test]
    fn test_can_attack_not_summoning_sick() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = BattlefieldEntity::new(id, 0, ts);
        entry.summoning_sick = false;
        game.battlefield.insert(id, entry);

        assert!(can_attack(&game, id));
    }

    #[test]
    fn test_cannot_attack_summoning_sick() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = BattlefieldEntity::new(id, 0, ts); // summoning_sick = true
        game.battlefield.insert(id, entry);

        assert!(!can_attack(&game, id));
    }

    #[test]
    fn test_can_attack_with_haste_while_summoning_sick() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Raging Cougar")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .keyword(KeywordAbility::Haste)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = BattlefieldEntity::new(id, 0, ts); // summoning_sick = true
        game.battlefield.insert(id, entry);

        assert!(can_attack(&game, id));
    }

    #[test]
    fn test_can_attack_not_on_battlefield() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        assert!(!can_attack(&game, fake_id));
    }
}
