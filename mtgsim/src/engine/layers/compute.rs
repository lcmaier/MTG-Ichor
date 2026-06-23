//! Core computation: `compute_characteristics(game, id)`.
//!
//! Walks the continuous effect registry in layer order and produces
//! `EffectiveCharacteristics` for a given object. All oracle queries
//! route through this function.
//!
//! Reads base characteristics from CardData, then applies all continuous
//! effects in layer order (1→2→3→4→5→6→7b→7c→7d).
//! Counter-derived P/T is applied in layer 7c alongside other modifiers.

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
                apply_modification(&effect.modification, chars);
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
fn apply_modification(modification: &EffectModification, chars: &mut EffectiveCharacteristics) {
    match modification {
        // Layer 2
        EffectModification::SetController(pid) => {
            chars.controller = *pid;
        }

        // Layer 4
        EffectModification::AddType(t) => { chars.types.insert(*t); }
        EffectModification::RemoveType(t) => { chars.types.remove(t); }
        EffectModification::SetTypes(types) => { chars.types = types.clone(); }
        EffectModification::AddSubtype(s) => { chars.subtypes.insert(s.clone()); }
        EffectModification::RemoveSubtype(s) => { chars.subtypes.remove(s); }
        EffectModification::SetSubtypes(subtypes) => { chars.subtypes = subtypes.clone(); }
        EffectModification::AddSupertype(s) => { chars.supertypes.insert(*s); }
        EffectModification::RemoveSupertype(s) => { chars.supertypes.remove(s); }

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
}
