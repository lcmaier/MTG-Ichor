use std::collections::HashSet;
use std::collections::HashMap;

use crate::types::card_types::{CardType, Supertype, Subtype};
use crate::types::colors::Color;
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

/// Costs that must be paid to activate an ability or cast a spell
#[derive(Debug, Clone, PartialEq)]
pub enum Cost {
    /// Tap the source permanent
    Tap,
    /// Pay a mana cost
    Mana(ManaCost),
    /// Sacrifice the source permanent
    SacrificeSelf,
    // Sacrifice N permanents matching criteria (future)
    // Sacrifice { count: u32, criteria: ... },
    /// Pay N life
    PayLife(u64),
    // Discard N cards (future)
    // Discard { count: u32 },
    // Add as needed...
}

/// What an ability or spell does when it resolves.
///
/// Effects are composable: `Sequence` chains multiple effects,
/// and each variant is a self-contained one-shot or continuous effect description.
///
/// **Continuous effects** (e.g. "+2/+2 until end of turn") will be modeled as
/// a one-shot `ApplyContinuousEffect` variant that *registers* a modifier in
/// the GameState. The layer system (rule 613) reads these modifiers to compute
/// effective characteristics. Modifiers carry a `Duration` (UntilEndOfTurn,
/// WhileSourceOnBattlefield, etc.) and are cleaned up when they expire.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Execute multiple effects in order
    Sequence(Vec<Effect>),

    /// Produce mana (for mana abilities)
    ProduceMana {
        mana: HashMap<ManaType, u64>,
    },

    /// Deal damage to target(s)
    DealDamage {
        amount: u64,
        // Targeting requirements will be added in Phase 2
    },

    /// Draw cards
    DrawCards {
        count: u64,
    },

    /// Gain life
    GainLife {
        amount: u64,
    },

    // Destroy target (future)
    // DestroyTarget { ... },
}

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

    pub fn ability(mut self, ability: AbilityDef) -> Self {
        self.data.abilities.push(ability);
        self
    }

    /// Shorthand: add a mana ability that taps to produce one mana of the given type.
    /// This is the standard basic land ability.
    pub fn mana_ability_single(mut self, mana_type: ManaType) -> Self {
        let mut mana_produced = HashMap::new();
        mana_produced.insert(mana_type, 1);

        self.data.abilities.push(AbilityDef {
            id: crate::types::ids::new_ability_id(),
            ability_type: AbilityType::Mana,
            costs: vec![Cost::Tap],
            effect: Effect::ProduceMana { mana: mana_produced },
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

    pub fn build(self) -> CardData {
        self.data
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
            .mana_cost(ManaCost::single(ManaType::Green, 1, 1))
            .color(Color::Green)
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(crate::types::card_types::CreatureType::Bear))
            .power_toughness(2, 2)
            .build();

        assert_eq!(bears.name, "Grizzly Bears");
        assert_eq!(bears.mana_cost.unwrap().mana_value(), 2);
        assert!(bears.types.contains(&CardType::Creature));
        assert_eq!(bears.power, Some(2));
        assert_eq!(bears.toughness, Some(2));
    }
}
