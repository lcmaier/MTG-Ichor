// Read-only board state queries.

use crate::oracle::characteristics::controls;
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId, Timestamp};

/// The permanents `player_id` controls, oldest first (CR 613.7's timestamps,
/// which never tie), sorting only those (`mana-architecture.md` §3.5).
///
/// With no effect changing control, a permanent's controller is its
/// battlefield entry's, which `controls` reads without a layer walk. Only
/// then is the map walked in its own order: a layer walk writes a trace
/// record, and a record written in hash order would differ by process.
pub fn permanents_controlled_by(game: &GameState, player_id: PlayerId) -> Vec<ObjectId> {
    if game.continuous_effects.summary().any_control_changing {
        return game.battlefield_ids_ordered().into_iter().filter(|&id| controls(game, id, player_id)).collect();
    }
    let mut mine: Vec<(Timestamp, ObjectId)> = game
        .battlefield
        .iter()
        .filter(|&(&id, entry)| {
            debug_assert_eq!(entry.controller == player_id, controls(game, id, player_id));
            entry.controller == player_id
        })
        .map(|(&id, entry)| (entry.timestamp, id))
        .collect();
    mine.sort_unstable_by_key(|&(timestamp, _)| timestamp);
    mine.into_iter().map(|(_, id)| id).collect()
}

#[cfg(test)]
mod tests {
    use crate::types::replacement::EnterMods;
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
        let id0 = game.add_object(obj0);
        game.place_on_battlefield(id0, 0, &EnterMods::NONE);

        let obj1 = GameObject::new(data, 1, Zone::Battlefield);
        let id1 = game.add_object(obj1);
        game.place_on_battlefield(id1, 1, &EnterMods::NONE);

        let p0 = permanents_controlled_by(&game, 0);
        assert_eq!(p0.len(), 1);
        assert!(p0.contains(&id0));

        let p1 = permanents_controlled_by(&game, 1);
        assert_eq!(p1.len(), 1);
        assert!(p1.contains(&id1));
    }
}
