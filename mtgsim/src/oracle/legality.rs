// Read-only legality queries — can a creature attack, block, etc.

use crate::oracle::characteristics::{has_keyword, has_summoning_sickness, is_creature};
use crate::state::game_state::{GameState, PhaseType};
use crate::types::card_types::CardType;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::keywords::KeywordAbility;

/// Check if a creature can attack (not summoning-sick, or has haste).
/// Rule 702.10b: Haste bypasses summoning sickness for attacking.
pub fn can_attack(game: &GameState, id: ObjectId) -> bool {
    if game.battlefield.contains_key(&id) {
        !has_summoning_sickness(game, id)
    } else {
        false
    }
}

/// Get all lands in a player's hand that they can legally play this turn.
///
/// Checks:
/// - Card is a land
/// - Player hasn't exceeded their lands-per-turn limit
/// - It's a main phase and the stack is empty (sorcery-speed timing)
/// - Player is the active player
pub fn playable_lands(game: &GameState, player_id: PlayerId) -> Vec<ObjectId> {
    let player = match game.players.get(player_id) {
        Some(p) => p,
        None => return Vec::new(),
    };

    // Timing: active player, main phase, empty stack
    if player_id != game.active_player {
        return Vec::new();
    }
    let is_main = matches!(
        game.phase.phase_type,
        PhaseType::Precombat | PhaseType::Postcombat
    );
    if !is_main || !game.stack.is_empty() {
        return Vec::new();
    }

    if !player.can_play_land() {
        return Vec::new();
    }

    player.hand.iter()
        .copied()
        .filter(|&id| {
            game.objects.get(&id)
                .map(|obj| obj.card_data.types.contains(&CardType::Land))
                .unwrap_or(false)
        })
        .collect()
}

/// Get all creatures controlled by a player that can legally be declared as attackers.
///
/// Checks per-creature legality (rule 508.1a): on battlefield, is a creature,
/// controlled by player, untapped, not summoning-sick (or has haste), no defender.
pub fn legal_attackers(game: &GameState, player_id: PlayerId) -> Vec<ObjectId> {
    game.battlefield.iter()
        .filter_map(|(id, entry)| {
            if entry.controller != player_id {
                return None;
            }
            if !is_creature(game, *id) {
                return None;
            }
            if entry.tapped {
                return None;
            }
            if !can_attack(game, *id) {
                return None;
            }
            if has_keyword(game, *id, KeywordAbility::Defender) {
                return None;
            }
            Some(*id)
        })
        .collect()
}

/// Get all creatures controlled by a player that can legally block.
///
/// A creature can block if it's on the battlefield, is a creature, untapped,
/// and controlled by the defending player. Specific attacker legality
/// (flying/reach checks) is handled during actual block declarations.
pub fn legal_blockers(game: &GameState, player_id: PlayerId) -> Vec<ObjectId> {
    game.battlefield.iter()
        .filter_map(|(id, entry)| {
            if entry.controller != player_id {
                return None;
            }
            if !is_creature(game, *id) {
                return None;
            }
            if entry.tapped {
                return None;
            }
            Some(*id)
        })
        .collect()
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
        let entry = BattlefieldEntity::new(id, 0, ts, 0);
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
        game.place_on_battlefield(id, 0); // entered this turn = summoning sick

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
        game.place_on_battlefield(id, 0); // entered this turn = summoning sick

        assert!(can_attack(&game, id));
    }

    #[test]
    fn test_can_attack_not_on_battlefield() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        assert!(!can_attack(&game, fake_id));
    }

    // --- playable_lands tests ---

    #[test]
    fn test_playable_lands_main_phase() {
        let mut game = GameState::new(2, 20);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Hand);
        let id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(id);

        let lands = playable_lands(&game, 0);
        assert_eq!(lands.len(), 1);
        assert_eq!(lands[0], id);
    }

    #[test]
    fn test_playable_lands_wrong_phase() {
        let mut game = GameState::new(2, 20);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Combat);
        game.active_player = 0;

        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Hand);
        let id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(id);

        assert!(playable_lands(&game, 0).is_empty());
    }

    #[test]
    fn test_playable_lands_already_played() {
        let mut game = GameState::new(2, 20);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;
        game.players[0].lands_played_this_turn = 1;

        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(forest, 0, Zone::Hand);
        let id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(id);

        assert!(playable_lands(&game, 0).is_empty());
    }

    // --- legal_attackers tests ---

    #[test]
    fn test_legal_attackers_basic() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = BattlefieldEntity::new(id, 0, ts, 0);
        game.battlefield.insert(id, entry);

        let attackers = legal_attackers(&game, 0);
        assert_eq!(attackers.len(), 1);
        assert_eq!(attackers[0], id);
    }

    #[test]
    fn test_legal_attackers_excludes_defender() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Wall of Stone")
            .card_type(CardType::Creature)
            .power_toughness(0, 8)
            .keyword(KeywordAbility::Defender)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = BattlefieldEntity::new(id, 0, ts, 0);
        game.battlefield.insert(id, entry);

        assert!(legal_attackers(&game, 0).is_empty());
    }

    #[test]
    fn test_legal_attackers_excludes_tapped() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = BattlefieldEntity::new(id, 0, ts, 0);
        entry.tapped = true;
        game.battlefield.insert(id, entry);

        assert!(legal_attackers(&game, 0).is_empty());
    }

    // --- legal_blockers tests ---

    #[test]
    fn test_legal_blockers_basic() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = BattlefieldEntity::new(id, 0, ts, 0);
        game.battlefield.insert(id, entry);

        let blockers = legal_blockers(&game, 0);
        assert_eq!(blockers.len(), 1);
    }

    #[test]
    fn test_legal_blockers_excludes_tapped() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = BattlefieldEntity::new(id, 0, ts, 0);
        entry.tapped = true;
        game.battlefield.insert(id, entry);

        assert!(legal_blockers(&game, 0).is_empty());
    }
}
