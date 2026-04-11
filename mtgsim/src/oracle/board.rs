// Read-only board state queries.

use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};

/// Get all object IDs on the battlefield controlled by a player.
pub fn permanents_controlled_by(game: &GameState, player_id: PlayerId) -> Vec<ObjectId> {
    game.battlefield.iter()
        .filter(|(_, entry)| entry.controller == player_id)
        .map(|(id, _)| *id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::types::card_types::CardType;
    use crate::types::zones::Zone;

    #[test]
    fn test_permanents_controlled_by_empty() {
        let game = GameState::new(2, 20);
        assert!(permanents_controlled_by(&game, 0).is_empty());
    }

    #[test]
    fn test_permanents_controlled_by_filters_by_controller() {
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Forest").card_type(CardType::Land).build();
        let obj0 = GameObject::new(data.clone(), 0, Zone::Battlefield);
        let id0 = obj0.id;
        game.add_object(obj0);
        game.place_on_battlefield(id0, 0);

        let obj1 = GameObject::new(data, 1, Zone::Battlefield);
        let id1 = obj1.id;
        game.add_object(obj1);
        game.place_on_battlefield(id1, 1);

        let p0 = permanents_controlled_by(&game, 0);
        assert_eq!(p0.len(), 1);
        assert!(p0.contains(&id0));

        let p1 = permanents_controlled_by(&game, 1);
        assert_eq!(p1.len(), 1);
        assert!(p1.contains(&id1));
    }
}
