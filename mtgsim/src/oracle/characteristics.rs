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
/// battlefield or the stack.** Printed control stopped equalling effective
/// control when Layer 2 landed, and the failure is silent in exactly the way
/// the printed-characteristics one was: a stolen creature keeps answering to
/// the player who lost it, so it untaps on the wrong turn, attacks for the wrong
/// side and taps for the wrong player's mana. This is the `get_effective_*`
/// wrapper for the field, and CLAUDE.md's layer-system invariant covers it.
///
/// `None` only when `id` is not in the object store at all.
///
/// Includes the stack: CR 108.4 gives a *spell* a controller, and gaining
/// control of a permanent spell means the permanent enters under the new
/// controller (CR 110.2). See `compute::base_controller`.
///
/// # Cost
///
/// Gated exactly the way `compute::effective_controller` is, and for the same
/// reason: Layer 2 is the only channel that writes `chars.controller`, so while
/// no `SetController` row is registered, `base_controller` — the value the walk
/// would seed the frame with — *is* the walk's answer. On every board with no
/// control-changing effect this stays the one `HashMap` probe the field read
/// was, which is what makes migrating the per-permanent sweeps
/// (`legal_attackers`, `available_mana_sources`, `format_battlefield`) free.
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
/// Both halves come from the layer frame, not from `BattlefieldEntity`.
/// `control_since_turn` is seeded from `controller_since_turn` and then
/// overwritten by any Layer 2 effect that actually moves control, so a stolen
/// creature is sick under its new controller without anything having been
/// written to the battlefield. See `EffectiveCharacteristics::control_since_turn`
/// for why it has to be derived rather than stored.
///
/// Convention: `control_since_turn = 0` is a pregame sentinel (CR 103.6
/// Leylines), so `0 >= 1` is false → not sick.
///
/// # What `>= game.turn_number` is and is not
///
/// It is not literally CR 302.6's "since their most recent turn began" — it is
/// "control was gained during the current turn", which is the same answer
/// whenever turns alternate. Gaining control during an opponent's turn N leaves
/// `control_since_turn = N < N + 1` by the time your turn comes, which is
/// correct: you have had it since before your turn began. This predates Layer 2
/// and Layer 2 does not change it.
///
/// # Reversion
///
/// A `Duration::UntilEndOfTurn` steal expires in the cleanup step, and control
/// returns to the original controller with no event and no mutation. Strictly,
/// CR 302.6 makes the creature sick for its original controller at that instant:
/// control was interrupted during the turn. We report it as not sick, and that
/// is unobservable — the only window between the expiry and the original
/// controller's next turn beginning is the cleanup step itself, where no player
/// receives priority (CR 514.3) and no ability can be activated. Modelling it
/// would need a record of an interruption that nothing can ever read.
pub fn has_summoning_sickness(game: &GameState, id: ObjectId) -> bool {
    if !game.battlefield.contains_key(&id) {
        return false;
    }
    let Some(chars) = compute_characteristics(game, id) else {
        return false;
    };
    if chars.control_since_turn >= game.turn_number {
        !chars.keywords.contains(&KeywordFlag::Haste)
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
