use std::collections::HashSet;
use std::sync::Arc;

use crate::types::card_types::{CardType, Supertype, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Effect, ManaOutput, Primitive, EffectRecipient, SelectionFilter};
use crate::types::keywords::KeywordAbility;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::ids::AbilityId;

/// The immutable "printed card" definition.
///
/// This is what's printed on the physical card — it never changes at runtime.
/// Game objects reference a CardData, and the layer system computes effective
/// characteristics on top of it.
#[derive(Debug, Clone, PartialEq)]
pub struct CardData {
    pub name: String,
    pub mana_cost: Option<ManaCost>,
    pub colors: HashSet<Color>,
    pub types: HashSet<CardType>,
    pub supertypes: HashSet<Supertype>,
    pub subtypes: HashSet<Subtype>,
    pub rules_text: String,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub loyalty: Option<i32>,
    pub defense: Option<i32>,
    pub abilities: Vec<AbilityDef>,
    pub keywords: HashSet<KeywordAbility>,
    /// Color indicator (rule 204) — used for cards with no mana cost that have
    /// an intrinsic color (e.g., back faces of DFCs, Ancestral Vision suspend).
    /// None means no color indicator; color is derived from mana cost instead.
    pub color_indicator: Option<Vec<Color>>,
    /// What this Aura can legally enchant (rule 303.4).
    /// None for non-Aura cards.
    pub enchant_filter: Option<SelectionFilter>,
}

/// The type of an ability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityType {
    /// Mana abilities (special — don't use the stack)
    Mana,
    /// Activated abilities (cost: effect)
    Activated,
    /// Triggered abilities (when/whenever/at)
    Triggered,
    /// Static abilities (continuous effect)
    Static,
    /// Spell ability (the effect of an instant/sorcery)
    Spell,
}

/// Definition of a single ability on a card.
///
/// This is the printed ability — at runtime, activated/triggered abilities
/// become objects on the stack with their own identity.
#[derive(Debug, Clone, PartialEq)]
pub struct AbilityDef {
    pub id: AbilityId,
    pub ability_type: AbilityType,
    pub costs: Vec<Cost>,
    pub effect: Effect,
}

/// Costs that must be paid to activate an ability or cast a spell.
///
/// Only `Tap`, `Mana`, `SacrificeSelf`, and `PayLife` are fully implemented.
/// Other variants exist for forward-compatibility; `can_pay_costs` and
/// `pay_single_cost` return `Err("not yet implemented")` for them.
#[derive(Debug, Clone, PartialEq)]
pub enum Cost {
    /// Tap the source permanent
    Tap,
    /// Untap the source permanent (Devoted Druid)
    Untap,
    /// Pay a mana cost
    Mana(ManaCost),
    /// Pay N life
    PayLife(u64),
    /// Sacrifice the source permanent
    SacrificeSelf,
    /// Sacrifice N permanents matching a filter ("Sacrifice a creature")
    Sacrifice(crate::types::effects::PermanentFilter, u32),
    /// Discard N cards matching a filter ("Discard a card")
    Discard(crate::types::effects::CardFilter, u32),
    /// Exile N cards from your graveyard matching a filter
    ExileFromGraveyard(crate::types::effects::CardFilter, u32),
    /// Remove N counters of a type from the source
    RemoveCounters(crate::types::effects::CounterType, u32),
    /// Add N counters of a type to the source (e.g. blight counters)
    AddCounters(crate::types::effects::CounterType, u32),
}

// Effect and Primitive types are defined in types::effects and re-exported here
// for convenience. See effect_system_plan.md for the full design.

// --- Builder Pattern ---

/// Builder for constructing CardData with a fluent API.
///
/// # Example
/// ```
/// use mtgsim::objects::card_data::CardDataBuilder;
/// use mtgsim::types::card_types::{CardType, Supertype, Subtype, LandType};
/// use mtgsim::types::mana::ManaType;
///
/// let forest = CardDataBuilder::new("Forest")
///     .card_type(CardType::Land)
///     .supertype(Supertype::Basic)
///     .subtype(Subtype::Land(LandType::Forest))
///     .mana_ability_single(ManaType::Green)
///     .build();
/// ```
pub struct CardDataBuilder {
    data: CardData,
}

impl CardDataBuilder {
    pub fn new(name: &str) -> Self {
        CardDataBuilder {
            data: CardData {
                name: name.to_string(),
                mana_cost: None,
                colors: HashSet::new(),
                types: HashSet::new(),
                supertypes: HashSet::new(),
                subtypes: HashSet::new(),
                rules_text: String::new(),
                power: None,
                toughness: None,
                loyalty: None,
                defense: None,
                abilities: Vec::new(),
                keywords: HashSet::new(),
                color_indicator: None,
                enchant_filter: None,
            },
        }
    }

    pub fn mana_cost(mut self, cost: ManaCost) -> Self {
        self.data.mana_cost = Some(cost);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.data.colors.insert(color);
        self
    }

    pub fn card_type(mut self, card_type: CardType) -> Self {
        self.data.types.insert(card_type);
        self
    }

    pub fn supertype(mut self, supertype: Supertype) -> Self {
        self.data.supertypes.insert(supertype);
        self
    }

    pub fn subtype(mut self, subtype: Subtype) -> Self {
        self.data.subtypes.insert(subtype);
        self
    }

    pub fn rules_text(mut self, text: &str) -> Self {
        self.data.rules_text = text.to_string();
        self
    }

    pub fn power_toughness(mut self, power: i32, toughness: i32) -> Self {
        self.data.power = Some(power);
        self.data.toughness = Some(toughness);
        self
    }

    pub fn loyalty(mut self, loyalty: i32) -> Self {
        self.data.loyalty = Some(loyalty);
        self
    }

    pub fn defense(mut self, defense: i32) -> Self {
        self.data.defense = Some(defense);
        self
    }

    pub fn keyword(mut self, keyword: KeywordAbility) -> Self {
        self.data.keywords.insert(keyword);
        self
    }

    pub fn color_indicator(mut self, colors: Vec<Color>) -> Self {
        self.data.color_indicator = Some(colors);
        self
    }

    pub fn ability(mut self, ability: AbilityDef) -> Self {
        self.data.abilities.push(ability);
        self
    }

    /// Shorthand: add a mana ability that taps to produce one mana of the given type.
    /// This is the standard basic land ability.
    pub fn mana_ability_single(mut self, mana_type: ManaType) -> Self {
        self.data.abilities.push(AbilityDef {
            id: crate::types::ids::new_ability_id(),
            ability_type: AbilityType::Mana,
            costs: vec![Cost::Tap],
            effect: Effect::Atom(
                Primitive::ProduceMana(ManaOutput {
                    mana: vec![(mana_type, AmountExpr::Fixed(1))],
                    special: vec![],
                }),
                EffectRecipient::Implicit,
            ),
        });

        // Set rules text if empty
        if self.data.rules_text.is_empty() {
            let mana_symbol = match mana_type {
                ManaType::White => "{W}",
                ManaType::Blue => "{U}",
                ManaType::Black => "{B}",
                ManaType::Red => "{R}",
                ManaType::Green => "{G}",
                ManaType::Colorless => "{C}",
            };
            self.data.rules_text = format!("{{T}}: Add {}.", mana_symbol);
        }

        self
    }

    pub fn enchant_filter(mut self, filter: SelectionFilter) -> Self {
        self.data.enchant_filter = Some(filter);
        self
    }

    pub fn build(self) -> Arc<CardData> {
        Arc::new(self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::card_types::LandType;

    #[test]
    fn test_build_basic_land() {
        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Forest))
            .mana_ability_single(ManaType::Green)
            .build();

        assert_eq!(forest.name, "Forest");
        assert!(forest.types.contains(&CardType::Land));
        assert!(forest.supertypes.contains(&Supertype::Basic));
        assert!(forest.mana_cost.is_none());
        assert_eq!(forest.abilities.len(), 1);
        assert_eq!(forest.abilities[0].ability_type, AbilityType::Mana);
        assert_eq!(forest.rules_text, "{T}: Add {G}.");
    }

    #[test]
    fn test_build_creature() {
        let bears = CardDataBuilder::new("Grizzly Bears")
            .mana_cost(ManaCost::build(&[ManaType::Green], 1))
            .color(Color::Green)
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(crate::types::card_types::CreatureType::Bear))
            .power_toughness(2, 2)
            .build();

        assert_eq!(bears.name, "Grizzly Bears");
        assert_eq!(bears.mana_cost.as_ref().unwrap().mana_value(), 2);
        assert!(bears.types.contains(&CardType::Creature));
        assert_eq!(bears.power, Some(2));
        assert_eq!(bears.toughness, Some(2));
    }

    #[test]
    fn test_card_data_color_indicator_none_default() {
        let card = CardDataBuilder::new("Test Card").build();
        assert!(card.color_indicator.is_none());
    }

    #[test]
    fn test_card_data_color_indicator_set() {
        let card = CardDataBuilder::new("Archangel Avacyn")
            .color_indicator(vec![Color::Red])
            .build();
        let indicator = card.color_indicator.as_ref().unwrap();
        assert_eq!(indicator.len(), 1);
        assert_eq!(indicator[0], Color::Red);

        // Multi-color indicator
        let card2 = CardDataBuilder::new("Nicol Bolas Back")
            .color_indicator(vec![Color::Blue, Color::Black, Color::Red])
            .build();
        let indicator2 = card2.color_indicator.as_ref().unwrap();
        assert_eq!(indicator2.len(), 3);
    }
}
