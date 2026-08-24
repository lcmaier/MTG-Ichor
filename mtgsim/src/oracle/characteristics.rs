// Read-only game state queries for object characteristics.
//
// All functions route through `compute_characteristics` from the layer system.
// This module is the single-point interface that the rest of the engine calls.

use std::collections::HashSet;

use crate::engine::layers::compute::compute_characteristics;
use crate::objects::card_data::AbilityDef;
use crate::state::game_state::GameState;
use crate::types::card_types::{CardType, Subtype, Supertype};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::keywords::KeywordFlag;

/// Check if a permanent has an effective keyword ability.
/// Routes through the layer system — accounts for granted/removed keywords.
pub fn has_keyword(game: &GameState, id: ObjectId, keyword: KeywordFlag) -> bool {
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

/// The effective controller of a game object after Layer 2 (CR 613.1b).
///
/// **Prefer this over `BattlefieldEntity.controller` for anything on the
/// battlefield or the stack** — a stolen permanent otherwise keeps answering to
/// the player who lost it, silently. Spells included: CR 108.4 gives one a
/// controller too.
///
/// `None` only when `id` is not in the object store at all.
///
/// The gate makes this a single `HashMap` probe while no `SetController` row
/// exists, which is what keeps the migrated per-permanent sweeps affordable;
/// see `RegistryScopeSummary::any_control_changing` for why it is exact.
pub fn get_effective_controller(game: &GameState, id: ObjectId) -> Option<PlayerId> {
    if !game.continuous_effects.summary().any_control_changing {
        return crate::engine::layers::compute::base_controller(game, id);
    }
    compute_characteristics(game, id).map(|chars| chars.controller)
}

/// Does `player` control `id` right now? The predicate form of
/// [`get_effective_controller`], which is how most call sites want it.
pub fn controls(game: &GameState, id: ObjectId, player: PlayerId) -> bool {
    get_effective_controller(game, id) == Some(player)
}

/// Check if a permanent has summoning sickness (CR 302.6).
///
/// > a creature's activated ability with the tap symbol [...] can't be
/// > activated unless the creature has been under its controller's control
/// > continuously since their most recent turn began.
///
/// Both halves come from the layer frame, so a stolen creature is sick under its
/// new controller with nothing written to the battlefield — see
/// `EffectiveCharacteristics::control_since_turn` for why that has to be
/// derived.
///
/// **The comparison is against the controller's own last turn, not the turn
/// being played.** Those agree on your turn and disagree on everyone else's:
/// a creature you cast on your turn is still sick throughout each opponent's
/// turn, and only your *next* turn beginning frees it. Comparing against
/// `game.turn_number` freed it one turn early, which a creature holding a
/// granted `{T}` ability could observe at instant speed (Citanul Hierophants).
/// `turn_number - 1` would be the two-player shortcut for the same thing and is
/// equally wrong at four; `last_turn_began` is per player for that reason.
///
/// Two boundary answers, both carried by the `None` arm:
/// `control_since_turn = 0` is the pregame sentinel (CR 103.6), and a player
/// who has not had a turn has the start of the game as their earliest reference
/// point — so an opening-hand Leyline is not sick, while anything that arrived
/// after the game began is, until its controller's first turn.
///
/// **Known imprecision, unobservable.** When an `UntilEndOfTurn` steal expires,
/// control was interrupted during the turn, so CR 302.6 makes the creature sick
/// again for its original controller; we report it as not sick. The only window
/// before that player's next turn begins is the cleanup step, where nobody gets
/// priority (CR 514.3), so nothing can ever ask.
pub fn has_summoning_sickness(game: &GameState, id: ObjectId) -> bool {
    if !game.battlefield.contains_key(&id) {
        return false;
    }
    let Some(chars) = compute_characteristics(game, id) else {
        return false;
    };
    let sick = match game.most_recent_turn_began(chars.controller) {
        // Control gained *during* that turn is not control since it began, so
        // the same turn number is still sick.
        Some(began) => chars.control_since_turn >= began,
        None => chars.control_since_turn > 0,
    };
    sick && !chars.keywords.contains(&KeywordFlag::Haste)
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

/// Get the effective supertypes of a game object after applying Layer 4 effects.
/// Routes through the layer system — accounts for type-changing effects.
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

/// Get effective power. Routes through the layer system — accounts for P/T
/// modifications, counters, and set-P/T effects.
///
/// **Not restricted to the battlefield.** CR 208.2a: a P/T-defining
/// characteristic-defining ability "functions everywhere, even outside the
/// game", and CR 208.3 gives a card off the battlefield the power and toughness
/// printed on it. A Tarmogoyf in a graveyard has a power, and it is the one its
/// CDA computes. This gate used to return `None` for anything not on the
/// battlefield, which cost nothing while no CDA existed and is wrong now.
///
/// `None` still means "no power at all" — an enchantment, a land, an artifact
/// with no printed P/T box.
///
/// Still unimplemented, and unrelated: CR 208.3's other half, that a noncreature
/// *permanent* has no P/T even with one printed (an unanimated Vehicle reports
/// its printed numbers here).
pub fn get_effective_power(game: &GameState, id: ObjectId) -> Option<i32> {
    compute_characteristics(game, id)?.power
}

/// Get effective toughness. See `get_effective_power` for why this is not
/// gated on the battlefield.
pub fn get_effective_toughness(game: &GameState, id: ObjectId) -> Option<i32> {
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
            .keyword(KeywordFlag::Flying)
            .keyword(KeywordFlag::Vigilance)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);

        assert!(has_keyword(&game, id, KeywordFlag::Flying));
        assert!(has_keyword(&game, id, KeywordFlag::Vigilance));
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

        assert!(!has_keyword(&game, id, KeywordFlag::Flying));
        assert!(!has_keyword(&game, id, KeywordFlag::Haste));
        assert!(!has_keyword(&game, id, KeywordFlag::Trample));
    }

    #[test]
    fn test_has_keyword_nonexistent_object() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        assert!(!has_keyword(&game, fake_id, KeywordFlag::Flying));
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
            AffectedSet, ContinuousEffect, EffectModification, Layer, PtValue,
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
            modification: EffectModification::ModifyPowerToughness { power: PtValue::Fixed(3), toughness: PtValue::Fixed(0) },
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
