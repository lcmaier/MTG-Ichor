// Read-only game state queries for object characteristics.
//
// All functions route through `compute_characteristics` from the layer system.
// This module is the single-point interface that the rest of the engine calls.

use std::collections::HashSet;

use crate::engine::layers::compute::compute_characteristics;
use crate::objects::card_data::AbilityDef;
use crate::state::game_state::GameState;
use crate::types::card_types::{CardType, Subtype, Supertype};
use crate::types::ids::ObjectId;
use crate::types::keywords::KeywordAbility;

/// Check if a permanent has an effective keyword ability.
/// Routes through the layer system — accounts for granted/removed keywords.
pub fn has_keyword(game: &GameState, id: ObjectId, keyword: KeywordAbility) -> bool {
    compute_characteristics(game, id)
        .map(|chars| chars.keywords.contains(&keyword))
        .unwrap_or(false)
}

/// Get the effective name of a game object.
pub fn get_effective_name(game: &GameState, id: ObjectId) -> String {
    compute_characteristics(game, id)
        .map(|chars| chars.name)
        .unwrap_or_default()
}

/// Check if an object on the battlefield is currently a creature.
/// Routes through the layer system — accounts for type-changing effects.
pub fn is_creature(game: &GameState, id: ObjectId) -> bool {
    compute_characteristics(game, id)
        .map(|chars| chars.types.contains(&CardType::Creature))
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

/// Get the effective colors of a game object after applying Layer 5 effects.
/// Routes through the layer system — accounts for color-changing effects
/// (AddColor, SetColors, RemoveAllColors).
pub fn get_effective_colors(game: &GameState, id: ObjectId) -> std::collections::HashSet<crate::types::colors::Color> {
    compute_characteristics(game, id)
        .map(|chars| chars.colors)
        .unwrap_or_default()
}

/// Get the effective card types of a game object after applying Layer 4 effects.
/// Routes through the layer system — accounts for type-changing effects.
pub fn get_effective_types(game: &GameState, id: ObjectId) -> HashSet<CardType> {
    compute_characteristics(game, id)
        .map(|chars| chars.types)
        .unwrap_or_default()
}

/// Get the effective subtypes of a game object after applying Layer 4 effects.
/// Routes through the layer system — accounts for type-changing effects.
pub fn get_effective_subtypes(game: &GameState, id: ObjectId) -> HashSet<Subtype> {
    compute_characteristics(game, id)
        .map(|chars| chars.subtypes)
        .unwrap_or_default()
}

/// Get the effective supertypes of a game object after applying Layer 4 effects.
/// Routes through the layer system — accounts for type-changing effects.
/// Does this object have `card_type` after Layer 4 effects?
///
/// Prefer this over `obj.card_data.types.contains(..)` for anything on the
/// battlefield or the stack: an artifact animated by Ensoul Artifact *is* a
/// creature even though its printed types say otherwise.
pub fn has_type(game: &GameState, id: ObjectId, card_type: CardType) -> bool {
    compute_characteristics(game, id)
        .map(|chars| chars.types.contains(&card_type))
        .unwrap_or(false)
}

/// Does this object have `subtype` after Layer 4 effects?
pub fn has_subtype(game: &GameState, id: ObjectId, subtype: &Subtype) -> bool {
    compute_characteristics(game, id)
        .map(|chars| chars.subtypes.contains(subtype))
        .unwrap_or(false)
}

/// Does this object have `supertype` after Layer 4 effects?
pub fn has_supertype(game: &GameState, id: ObjectId, supertype: Supertype) -> bool {
    compute_characteristics(game, id)
        .map(|chars| chars.supertypes.contains(&supertype))
        .unwrap_or(false)
}

/// Is this object a permanent type (artifact, creature, enchantment, land,
/// planeswalker, or battle) after Layer 4 effects? Used to decide whether a
/// resolving spell goes to the battlefield.
pub fn has_permanent_type(game: &GameState, id: ObjectId) -> bool {
    compute_characteristics(game, id)
        .map(|chars| chars.types.iter().any(|t| t.is_permanent()))
        .unwrap_or(false)
}

pub fn get_effective_supertypes(game: &GameState, id: ObjectId) -> HashSet<Supertype> {
    compute_characteristics(game, id)
        .map(|chars| chars.supertypes)
        .unwrap_or_default()
}

/// Get the effective abilities of a game object after all layers.
///
/// Prefer this over `obj.card_data.abilities` for anything on the battlefield:
/// a Blood-Mooned dual land has lost its printed mana abilities and gained an
/// intrinsic `{T}: Add {R}` that exists nowhere in its `CardData` (CR 305.7).
///
/// The returned `AbilityDef`s carry stable ids, so `ability.id` remains a valid
/// activation handle across calls — including for synthesized intrinsics.
pub fn get_effective_abilities(game: &GameState, id: ObjectId) -> Vec<AbilityDef> {
    compute_characteristics(game, id)
        .map(|chars| chars.abilities)
        .unwrap_or_default()
}

/// Get effective power for a creature on the battlefield.
/// Routes through the layer system — accounts for P/T modifications,
/// counters, and set-P/T effects.
pub fn get_effective_power(game: &GameState, id: ObjectId) -> Option<i32> {
    // Only return power for battlefield objects (maintains existing behavior)
    game.battlefield.get(&id)?;
    compute_characteristics(game, id)?.power
}

/// Get effective toughness for a creature on the battlefield.
/// Routes through the layer system — accounts for P/T modifications,
/// counters, and set-P/T effects.
pub fn get_effective_toughness(game: &GameState, id: ObjectId) -> Option<i32> {
    // Only return toughness for battlefield objects (maintains existing behavior)
    game.battlefield.get(&id)?;
    compute_characteristics(game, id)?.toughness
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
            .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 3))
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
            .mana_cost(ManaCost::build(&[ManaType::Green], 1))
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
        use crate::engine::layers::types::{
            AffectedSet, ContinuousEffect, EffectModification, Layer,
        };
        use crate::types::effects::Duration;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register a +3/+0 effect via the layer system
        let effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: crate::engine::layers::types::EffectOrigin::Resolution,
            layer: Layer::Layer7cModifyPT,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: 1,
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::ModifyPowerToughness { power: 3, toughness: 0 },
        };
        game.continuous_effects.add(effect);

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
