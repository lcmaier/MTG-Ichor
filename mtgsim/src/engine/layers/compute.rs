//! Core computation: `compute_characteristics(game, id)`.
//!
//! Walks the continuous effect registry in layer order and produces
//! `EffectiveCharacteristics` for a given object. All oracle queries
//! route through this function.
//!
//! Reads base characteristics from CardData, then applies all continuous
//! effects in layer order (1→2→3→4→5→6→7b→7c→7d).

use crate::engine::layers::types::*;
use crate::state::game_state::GameState;
use crate::types::effects::CounterType;
use crate::types::ids::ObjectId;

/// Compute the effective characteristics of a game object after applying
/// all active continuous effects in layer order.
///
/// Returns `None` if the object doesn't exist.
pub fn compute_characteristics(game: &GameState, id: ObjectId) -> Option<EffectiveCharacteristics> {
    let obj = game.objects.get(&id)?;
    let card = &obj.card_data;

    // Start from printed (base) characteristics
    let mut chars = EffectiveCharacteristics {
        name: card.name.clone(),
        mana_cost: card.mana_cost.clone(),
        colors: card.colors.clone(),
        types: card.types.clone(),
        subtypes: card.subtypes.clone(),
        supertypes: card.supertypes.clone(),
        keywords: card.keywords.clone(),
        abilities: card.abilities.clone(),
        power: card.power,
        toughness: card.toughness,
        controller: obj.owner, // default; overridden by battlefield entry or L2 effects
    };

    // If on the battlefield, use the actual controller from BattlefieldEntity
    if let Some(entry) = game.battlefield.get(&id) {
        chars.controller = entry.controller;
    }

    // Walk layers in order, applying all effects (registered + counters)
    apply_effects(game, id, &mut chars);

    Some(chars)
}

/// Apply all continuous effects in layer order (rule 613).
///
/// Walks registered effects by layer, and also applies counter-derived
/// P/T modifications in layer 7c (rule 613.4c) alongside other modifiers.
fn apply_effects(game: &GameState, id: ObjectId, chars: &mut EffectiveCharacteristics) {
    let has_registered = !game.continuous_effects.is_empty();
    let on_battlefield = game.battlefield.contains_key(&id);

    // Fast path: nothing to apply
    if !has_registered && !on_battlefield {
        return;
    }

    let layers = [
        Layer::Layer1Copy,
        Layer::Layer2Control,
        Layer::Layer3Text,
        Layer::Layer4Type,
        Layer::Layer5Color,
        Layer::Layer6Ability,
        Layer::Layer7bSetPT,
        Layer::Layer7cModifyPT,
        Layer::Layer7dSwitchPT,
    ];

    for &layer in &layers {
        // Apply registered effects in this layer
        if has_registered {
            let effects = game.continuous_effects.effects_in_layer(layer);
            for effect in effects {
                if !effect_applies_to(effect, id, chars, game) {
                    continue;
                }
                apply_modification(&effect.modification, chars, id);
            }
        }

        // Apply counter P/T in layer 7c (rule 613.4c)
        if layer == Layer::Layer7cModifyPT {
            if let Some(entry) = game.battlefield.get(&id) {
                let plus = entry.counter_count(CounterType::PlusOnePlusOne) as i32;
                if plus != 0 {
                    if let Some(ref mut p) = chars.power { *p += plus; }
                    if let Some(ref mut t) = chars.toughness { *t += plus; }
                }
                let minus = entry.counter_count(CounterType::MinusOneMinusOne) as i32;
                if minus != 0 {
                    if let Some(ref mut p) = chars.power { *p -= minus; }
                    if let Some(ref mut t) = chars.toughness { *t -= minus; }
                }
                // TODO: handle other P/T-modifying counter types (+2/+2, +0/+1, etc.)
                // when they are added to CounterType.
            }
        }
    }
}

/// Check whether a continuous effect applies to the given object.
fn effect_applies_to(
    effect: &ContinuousEffect,
    id: ObjectId,
    chars: &EffectiveCharacteristics,
    game: &GameState,
) -> bool {
    match &effect.affected {
        AffectedSet::SourceOnly => effect.source == id,
        AffectedSet::Fixed(ids) => ids.contains(&id),
        AffectedSet::Filter { filter, controller } => {
            // Object must be on the battlefield for filter-based effects
            if !game.battlefield.contains_key(&id) {
                return false;
            }
            // Check controller constraint
            if let Some(ctrl) = controller {
                if chars.controller != *ctrl {
                    return false;
                }
            }
            // Check the permanent filter against current characteristics
            permanent_matches_filter(filter, chars)
        }
    }
}

/// Check if a permanent's current characteristics match a filter.
fn permanent_matches_filter(
    filter: &crate::types::effects::PermanentFilter,
    chars: &EffectiveCharacteristics,
) -> bool {
    use crate::types::effects::PermanentFilter;
    match filter {
        PermanentFilter::All => true,
        PermanentFilter::ByType(t) => chars.types.contains(t),
        PermanentFilter::BySubtype(s) => chars.subtypes.contains(s),
        PermanentFilter::BySupertype(s) => chars.supertypes.contains(s),
        PermanentFilter::ByColor(c) => chars.colors.contains(c),
        PermanentFilter::ByController(_) => {
            // Controller filtering is handled by the AffectedSet::Filter.controller field
            true
        }
        PermanentFilter::PowerLE(n) => {
            chars.power.map(|p| p <= *n).unwrap_or(false)
        }
        PermanentFilter::And(a, b) => {
            permanent_matches_filter(a, chars) && permanent_matches_filter(b, chars)
        }
        PermanentFilter::Not(inner) => !permanent_matches_filter(inner, chars),
    }
}

/// Apply a single effect modification to the characteristics frame.
///
/// `object_id` is the object being computed. Layer 4's subtype arms need it to
/// derive stable ids for intrinsic mana abilities (CR 305.6) — see
/// `land_types::intrinsic_mana_ability`.
fn apply_modification(
    modification: &EffectModification,
    chars: &mut EffectiveCharacteristics,
    object_id: ObjectId,
) {
    match modification {
        // Layer 2
        EffectModification::SetController(pid) => {
            chars.controller = *pid;
        }

        // Layer 4
        EffectModification::AddType(t) => { chars.types.insert(*t); }
        EffectModification::RemoveType(t) => { chars.types.remove(t); }
        EffectModification::SetTypes(types) => { chars.types = types.clone(); }
        // CR 305.6/305.7 land semantics live in `land_types` — see the module
        // docs there for why this is not a Layer 6 concern.
        EffectModification::AddSubtype(s) => {
            crate::engine::layers::land_types::apply_add_subtype(chars, s, object_id);
        }
        EffectModification::RemoveSubtype(s) => { chars.subtypes.remove(s); }
        EffectModification::SetSubtypes(subtypes) => {
            crate::engine::layers::land_types::apply_set_subtypes(chars, subtypes, object_id);
        }
        EffectModification::AddSupertype(s) => { chars.supertypes.insert(*s); }
        EffectModification::RemoveSupertype(s) => { chars.supertypes.remove(s); }
        EffectModification::SetSupertypes(supertypes) => { chars.supertypes = supertypes.clone(); }

        // Layer 5
        EffectModification::AddColor(c) => { chars.colors.insert(*c); }
        EffectModification::SetColors(colors) => { chars.colors = colors.clone(); }
        EffectModification::RemoveAllColors => { chars.colors.clear(); }

        // Layer 6
        EffectModification::GrantKeyword(kw) => { chars.keywords.insert(*kw); }
        EffectModification::RemoveKeyword(kw) => { chars.keywords.remove(kw); }
        EffectModification::LoseAllAbilities => {
            chars.keywords.clear();
            chars.abilities.clear();
        }

        // Layer 7b
        EffectModification::SetPowerToughness { power, toughness } => {
            chars.power = Some(*power);
            chars.toughness = Some(*toughness);
        }

        // Layer 7c
        EffectModification::ModifyPowerToughness { power, toughness } => {
            if let Some(ref mut p) = chars.power {
                *p += power;
            }
            if let Some(ref mut t) = chars.toughness {
                *t += toughness;
            }
        }

        // Layer 7d
        EffectModification::SwitchPowerToughness => {
            let old_power = chars.power;
            chars.power = chars.toughness;
            chars.toughness = old_power;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::types::card_types::CardType;
    use crate::types::colors::Color;
    use crate::types::keywords::KeywordAbility;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;

    #[test]
    fn test_base_characteristics_from_card_data() {
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
        game.place_on_battlefield(id, 0);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.name, "Grizzly Bears");
        assert_eq!(chars.power, Some(2));
        assert_eq!(chars.toughness, Some(2));
        assert!(chars.types.contains(&CardType::Creature));
        assert!(chars.colors.contains(&Color::Green));
        assert_eq!(chars.controller, 0);
    }

    // COVERS-PARTIAL: ATOM-613.4c-001
    #[test]
    fn test_registered_effect_pump_power_only() {
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

        // Register a +3/+0 effect
        let effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer7cModifyPT,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: 1,
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::ModifyPowerToughness { power: 3, toughness: 0 },
        };
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.power, Some(5));
        assert_eq!(chars.toughness, Some(2));
    }

    // COVERS: ATOM-122.1a-001
    #[test]
    fn test_counters_modify_pt() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = game.place_on_battlefield(id, 0);
        entry.add_counters(CounterType::PlusOnePlusOne, 2);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.power, Some(4));
        assert_eq!(chars.toughness, Some(4));
    }

    // COVERS-PARTIAL: ATOM-122.1a-002
    #[test]
    fn test_counters_plus_and_minus() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Big Creature")
            .card_type(CardType::Creature)
            .power_toughness(5, 5)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = game.place_on_battlefield(id, 0);
        entry.add_counters(CounterType::PlusOnePlusOne, 3);
        entry.add_counters(CounterType::MinusOneMinusOne, 1);

        let chars = compute_characteristics(&game, id).unwrap();
        // Net: +3 -1 = +2
        assert_eq!(chars.power, Some(7));
        assert_eq!(chars.toughness, Some(7));
    }

    #[test]
    fn test_nonexistent_object_returns_none() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        assert!(compute_characteristics(&game, fake_id).is_none());
    }

    #[test]
    fn test_non_battlefield_object_no_modifiers() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .color(Color::Red)
            .build();
        let obj = GameObject::new(data, 0, Zone::Hand);
        let id = obj.id;
        game.add_object(obj);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.name, "Lightning Bolt");
        assert!(chars.types.contains(&CardType::Instant));
        assert!(chars.colors.contains(&Color::Red));
        // Not on battlefield, so controller defaults to owner
        assert_eq!(chars.controller, 0);
    }

    #[test]
    fn test_keywords_from_card_data() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Serra Angel")
            .card_type(CardType::Creature)
            .power_toughness(4, 4)
            .keyword(KeywordAbility::Flying)
            .keyword(KeywordAbility::Vigilance)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.keywords.contains(&KeywordAbility::Flying));
        assert!(chars.keywords.contains(&KeywordAbility::Vigilance));
        assert!(!chars.keywords.contains(&KeywordAbility::Trample));
    }

    // COVERS-PARTIAL: ATOM-613.4c-001
    #[test]
    fn test_registered_effect_modifies_pt() {
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

        // Register a +3/+3 effect targeting this creature
        let effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer7cModifyPT,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: 1,
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::ModifyPowerToughness { power: 3, toughness: 3 },
        };
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.power, Some(5));
        assert_eq!(chars.toughness, Some(5));
    }

    #[test]
    fn test_registered_effect_grants_keyword() {
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

        // Register a "gains flying" effect
        let effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer6Ability,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: 1,
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::GrantKeyword(KeywordAbility::Flying),
        };
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.keywords.contains(&KeywordAbility::Flying));
    }

    #[test]
    fn test_filter_based_effect() {
        use crate::types::effects::{Duration, PermanentFilter};

        let mut game = GameState::new(2, 20);

        // Two creatures controlled by player 0
        let bears_data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let bears = GameObject::new(bears_data, 0, Zone::Battlefield);
        let bears_id = bears.id;
        game.add_object(bears);
        game.place_on_battlefield(bears_id, 0);

        let giant_data = CardDataBuilder::new("Hill Giant")
            .card_type(CardType::Creature)
            .power_toughness(3, 3)
            .build();
        let giant = GameObject::new(giant_data, 0, Zone::Battlefield);
        let giant_id = giant.id;
        game.add_object(giant);
        game.place_on_battlefield(giant_id, 0);

        // Register an anthem: "Creatures you control get +1/+1"
        let anthem_source = crate::types::ids::new_object_id();
        let effect = ContinuousEffect {
            id: 0,
            source: anthem_source,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer7cModifyPT,
            duration: Duration::WhileSourceOnBattlefield,
            controller: 0,
            created_on_turn: 1,
            timestamp: 1,
            affected: AffectedSet::Filter {
                filter: PermanentFilter::ByType(CardType::Creature),
                controller: Some(0),
            },
            modification: EffectModification::ModifyPowerToughness { power: 1, toughness: 1 },
        };
        game.continuous_effects.add(effect);

        let bears_chars = compute_characteristics(&game, bears_id).unwrap();
        assert_eq!(bears_chars.power, Some(3));
        assert_eq!(bears_chars.toughness, Some(3));

        let giant_chars = compute_characteristics(&game, giant_id).unwrap();
        assert_eq!(giant_chars.power, Some(4));
        assert_eq!(giant_chars.toughness, Some(4));
    }

    #[test]
    fn test_set_colors_replaces_base_colors() {
        use crate::types::effects::Duration;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register a "becomes blue" effect (SetColors)
        let mut blue = std::collections::HashSet::new();
        blue.insert(Color::Blue);
        let effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer5Color,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::SetColors(blue),
        };
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.colors.contains(&Color::Blue));
        assert!(!chars.colors.contains(&Color::Green));
        assert_eq!(chars.colors.len(), 1);
    }

    #[test]
    fn test_add_color_preserves_existing() {
        use crate::types::effects::Duration;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register an "also red" effect (AddColor)
        let effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer5Color,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::AddColor(Color::Red),
        };
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.colors.contains(&Color::Green));
        assert!(chars.colors.contains(&Color::Red));
        assert_eq!(chars.colors.len(), 2);
    }

    #[test]
    fn test_remove_all_colors_makes_colorless() {
        use crate::types::effects::Duration;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register a "becomes colorless" effect
        let effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer5Color,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::RemoveAllColors,
        };
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.colors.is_empty());
    }

    #[test]
    fn test_color_change_independent_of_pt() {
        use crate::types::effects::Duration;

        // Color change (L5) should not affect P/T (L7) and vice versa
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // L5: becomes blue
        let mut blue = std::collections::HashSet::new();
        blue.insert(Color::Blue);
        let color_effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer5Color,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::SetColors(blue),
        };
        game.continuous_effects.add(color_effect);

        // L7c: +3/+3
        let pt_effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer7cModifyPT,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::ModifyPowerToughness { power: 3, toughness: 3 },
        };
        game.continuous_effects.add(pt_effect);

        let chars = compute_characteristics(&game, id).unwrap();
        // Color should be blue (not green)
        assert!(chars.colors.contains(&Color::Blue));
        assert!(!chars.colors.contains(&Color::Green));
        // P/T should be 5/5 (2+3)
        assert_eq!(chars.power, Some(5));
        assert_eq!(chars.toughness, Some(5));
    }

    #[test]
    fn test_filter_based_color_effect() {
        use crate::types::effects::{Duration, PermanentFilter};

        // Static ability: "Creatures you control are also red"
        let mut game = GameState::new(2, 20);

        let bears_data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let bears = GameObject::new(bears_data, 0, Zone::Battlefield);
        let bears_id = bears.id;
        game.add_object(bears);
        game.place_on_battlefield(bears_id, 0);

        // Opponent's creature should NOT be affected
        let opp_data = CardDataBuilder::new("Savannah Lions")
            .card_type(CardType::Creature)
            .color(Color::White)
            .power_toughness(2, 1)
            .build();
        let opp = GameObject::new(opp_data, 1, Zone::Battlefield);
        let opp_id = opp.id;
        game.add_object(opp);
        game.place_on_battlefield(opp_id, 1);

        let source_id = crate::types::ids::new_object_id();
        let effect = ContinuousEffect {
            id: 0,
            source: source_id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer5Color,
            duration: Duration::WhileSourceOnBattlefield,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Filter {
                filter: PermanentFilter::ByType(CardType::Creature),
                controller: Some(0),
            },
            modification: EffectModification::AddColor(Color::Red),
        };
        game.continuous_effects.add(effect);

        let bears_chars = compute_characteristics(&game, bears_id).unwrap();
        assert!(bears_chars.colors.contains(&Color::Green));
        assert!(bears_chars.colors.contains(&Color::Red));

        let opp_chars = compute_characteristics(&game, opp_id).unwrap();
        assert!(opp_chars.colors.contains(&Color::White));
        assert!(!opp_chars.colors.contains(&Color::Red));
    }

    // COVERS-PARTIAL: ATOM-613.4d-004
    #[test]
    fn test_counters_applied_before_switch_pt() {
        use crate::types::effects::Duration;

        // Regression test: counters are in 7c, switch is 7d.
        // A 1/4 creature with two +1/+1 counters and a switch effect:
        // 7c: 1+2=3 / 4+2=6, then 7d: swap → 6/3
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Wall")
            .card_type(CardType::Creature)
            .power_toughness(1, 4)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = game.place_on_battlefield(id, 0);
        entry.add_counters(CounterType::PlusOnePlusOne, 2);

        // Register a switch P/T effect (layer 7d)
        let effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer7dSwitchPT,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: 1,
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::SwitchPowerToughness,
        };
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        // 7c: 1+2=3, 4+2=6; 7d: swap → 6/3
        assert_eq!(chars.power, Some(6));
        assert_eq!(chars.toughness, Some(3));
    }

    // === Layer 4 type-changing tests ===

    // COVERS-PARTIAL: ATOM-205.1b-004
    #[test]
    fn test_add_type_preserves_existing() {
        use crate::types::effects::Duration;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Darksteel Ingot")
            .card_type(CardType::Artifact)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register "becomes also a creature" effect
        let effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer4Type,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::AddType(CardType::Creature),
        };
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.types.contains(&CardType::Artifact));
        assert!(chars.types.contains(&CardType::Creature));
    }

    #[test]
    fn test_remove_type() {
        use crate::types::effects::Duration;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Mycosynth Lattice")
            .card_type(CardType::Artifact)
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Remove Creature type
        let effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer4Type,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::RemoveType(CardType::Creature),
        };
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.types.contains(&CardType::Artifact));
        assert!(!chars.types.contains(&CardType::Creature));
    }

    // COVERS-PARTIAL: ATOM-205.1a-003
    #[test]
    fn test_set_subtypes_replaces_all() {
        use crate::types::card_types::{LandType, Subtype};
        use crate::types::effects::Duration;
        use std::collections::HashSet;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Steam Vents")
            .card_type(CardType::Land)
            .subtype(Subtype::Land(LandType::Island))
            .subtype(Subtype::Land(LandType::Mountain))
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // SetSubtypes to just Forest
        let mut forest_set = HashSet::new();
        forest_set.insert(Subtype::Land(LandType::Forest));
        let effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer4Type,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::SetSubtypes(forest_set),
        };
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.subtypes.contains(&Subtype::Land(LandType::Forest)));
        assert!(!chars.subtypes.contains(&Subtype::Land(LandType::Island)));
        assert!(!chars.subtypes.contains(&Subtype::Land(LandType::Mountain)));
        assert_eq!(chars.subtypes.len(), 1);
    }

    #[test]
    fn test_add_subtype_preserves_existing() {
        use crate::types::card_types::{LandType, Subtype};
        use crate::types::effects::Duration;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Mountain")
            .card_type(CardType::Land)
            .subtype(Subtype::Land(LandType::Mountain))
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Add Swamp subtype ("in addition to")
        let effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer4Type,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::AddSubtype(Subtype::Land(LandType::Swamp)),
        };
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.subtypes.contains(&Subtype::Land(LandType::Mountain)));
        assert!(chars.subtypes.contains(&Subtype::Land(LandType::Swamp)));
        assert_eq!(chars.subtypes.len(), 2);
    }

    #[test]
    fn test_add_supertype() {
        use crate::types::card_types::Supertype;
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

        // Add Legendary supertype
        let effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer4Type,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::AddSupertype(Supertype::Legendary),
        };
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.supertypes.contains(&Supertype::Legendary));
    }

    // COVERS-PARTIAL: ATOM-613.1d-001
    #[test]
    fn test_type_change_before_color_change() {
        use crate::types::effects::{Duration, PermanentFilter};

        // Test layer ordering: L4 (type) applies before L5 (color)
        // A filter-based color effect that checks types should see the
        // type as it stands after L4.
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Darksteel Ingot")
            .card_type(CardType::Artifact)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // L4: Add Creature type
        let l4_effect = ContinuousEffect {
            id: 0,
            source: id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer4Type,
            duration: Duration::UntilEndOfTurn,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Fixed(vec![id]),
            modification: EffectModification::AddType(CardType::Creature),
        };
        game.continuous_effects.add(l4_effect);

        // L5: "Creatures are also red" (filter-based)
        let l5_source = crate::types::ids::new_object_id();
        let l5_effect = ContinuousEffect {
            id: 0,
            source: l5_source,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer5Color,
            duration: Duration::WhileSourceOnBattlefield,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Filter {
                filter: PermanentFilter::ByType(CardType::Creature),
                controller: None,
            },
            modification: EffectModification::AddColor(Color::Red),
        };
        game.continuous_effects.add(l5_effect);

        let chars = compute_characteristics(&game, id).unwrap();
        // Should be both Artifact and Creature (L4 applied)
        assert!(chars.types.contains(&CardType::Artifact));
        assert!(chars.types.contains(&CardType::Creature));
        // Should be Red (L5 filter sees the L4-modified type = Creature)
        assert!(chars.colors.contains(&Color::Red));
    }
}
