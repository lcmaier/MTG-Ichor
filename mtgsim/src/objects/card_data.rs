use std::collections::HashSet;
use std::sync::Arc;

use crate::types::card_types::{CardType, Supertype, Subtype};
use crate::types::colors::Color;
use crate::types::costs::{AdditionalCost, AlternativeCost, Cost};
use crate::types::cost_modification::{CostChange, CostModificationDef};
use crate::types::effects::{AmountExpr, Effect, ManaOutput, ObjectFilter, PlayerRef, Primitive, EffectRecipient, SelectionFilter, Selector, TargetCount};
use crate::types::keywords::KeywordFlag;
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
    /// Shared with every frame the layer walk seeds from this card, and
    /// written only through `Arc::make_mut`, so a walk is a refcount bump
    /// rather than a clone of the ability tree (`layers-architecture.md` §12).
    pub abilities: Arc<Vec<AbilityDef>>,
    pub keyword_flags: HashSet<KeywordFlag>,
    /// Color indicator (rule 204) — used for cards with no mana cost that have
    /// an intrinsic color (e.g., back faces of DFCs, Ancestral Vision suspend).
    /// None means no color indicator; color is derived from mana cost instead.
    pub color_indicator: Option<Vec<Color>>,
    /// What this Aura can legally enchant (rule 303.4).
    /// None for non-Aura cards.
    pub enchant_filter: Option<SelectionFilter>,
    /// CR 601.2c's instances of "target" for this card cast as a spell, in
    /// printed order: the spell ability's [`AbilityDef::instances`] — except for
    /// an Aura, whose one instance is its enchant ability (CR 303.4a) and sits
    /// in no effect tree. One field, one rule: the castability pre-check, the
    /// announcement and CR 608.2b's re-check all read this, so all three see
    /// `enchant_filter`.
    ///
    /// Written by `CardDataBuilder::build` and nothing else. A `CardData` is
    /// built once and shared behind an `Arc`, so this is computed once per card
    /// rather than once per card in hand per priority pass, which is what it
    /// replaced (`roadmap-v2.md` row A4n).
    pub spell_instances: Vec<EffectRecipient>,
    /// Alternative costs this card can be cast for (rule 118.9).
    /// A player may choose at most one when casting.
    pub alternative_costs: Vec<AlternativeCost>,
    /// Additional costs this card can optionally pay (rule 118.8).
    /// Multiple may be paid in a single cast (e.g. kicker + buyback).
    pub additional_costs: Vec<AdditionalCost>,
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

/// When an activated ability may be activated (CR 602.5d), beyond having
/// priority.
///
/// One value, not `backlog.md` §2.8's activation-restriction surface: Equip
/// (CR 702.6a) reads "Activate only as a sorcery", and that is the whole of
/// what this enum can say. §2.8 owns the rest — "activate only once each
/// turn", "only during combat", the functioning zone — and grows this enum
/// when a card needs it, rather than this enum guessing at their shape.
///
/// Honored at all three ability-index sites CLAUDE.md names:
/// `activatable_abilities` does not offer a restricted ability out of its
/// window, `activate_ability` refuses it (the enforcement), and
/// `priority.rs` reaches the second through the first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationRestriction {
    /// Any time the player has priority (CR 602.1, the default).
    None,
    /// CR 602.5d — "the player must follow the timing rules for casting a
    /// sorcery spell, though the ability isn't actually a sorcery": active
    /// player, main phase, empty stack.
    OnlyAsSorcery,
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
    /// CR 601.2c's instances of "target" that `effect` declares, in printed
    /// order — [`Effect::instances`], stored.
    ///
    /// **A struct literal writes `Vec::new()` here, and `CardDataBuilder::build`
    /// overwrites it** on every def it can reach from the card, the way it
    /// stamps `id`: the printed list, then every def nested in an effect. A
    /// def that never meets the builder keeps the empty list, which is the
    /// right answer for every one that exists — CR 305.6's synthesized mana
    /// ability and the static abilities the test helpers build announce
    /// nothing. A runtime-built def with a targeting effect would announce
    /// nothing too, silently: `cards::registry`'s test is the gate for every
    /// def a card carries, and a new runtime birth site owes the same check.
    pub instances: Vec<EffectRecipient>,
    /// CR 602.5d. Meaningful only when `ability_type` is `Activated`; every
    /// other kind carries `None`.
    pub activation_restriction: ActivationRestriction,
    /// CR 604.3 — this ability is a characteristic-defining ability.
    ///
    /// CR 604.3a lists five criteria. Four of them are properties of the
    /// ability's *text* and are what the card author asserts by setting this:
    /// (1) it defines colors, subtypes, power, or toughness; (3) it doesn't
    /// directly affect any other object's characteristics; (4) it isn't an
    /// ability the object grants itself; (5) it doesn't set those values only
    /// under a condition.
    ///
    /// Criterion (2) is *provenance* — printed on the object it affects,
    /// granted to a token by the effect that created it, or acquired by a copy
    /// or text-changing effect — and no bool on a definition can state it,
    /// because the same `AbilityDef` can reach an object by any route. It is
    /// maintained instead by whoever writes the ability onto an object:
    ///
    /// - Printed abilities keep whatever the card author wrote. ✅
    /// - A copy effect (Layer 1) or a text-changing effect (Layer 3) hands the
    ///   whole `AbilityDef` over, so the flag rides along — which is exactly
    ///   what 604.3a(2) asks for. ✅
    /// - **A Layer 6 `GrantAbility` clears this on the def it grants**, in
    ///   `layers::compute`'s grant arm: a Layer 6 grant is none of 604.3a(2)'s
    ///   routes, so the ability is never a CDA there, however its text reads.
    ///   A token's abilities are its `CardData`'s, 604.3a(2)'s second route,
    ///   and keep the flag as printed ones do.
    ///
    /// Read by `engine::layers::cda`, which applies CDAs off the object's own
    /// effective ability list. CDAs are never registered as continuous effects
    /// — see `Layer::Layer7aCdaPT`.
    pub is_characteristic_defining: bool,
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
                abilities: Arc::new(Vec::new()),
                keyword_flags: HashSet::new(),
                color_indicator: None,
                enchant_filter: None,
                spell_instances: Vec::new(),
                alternative_costs: Vec::new(),
                additional_costs: Vec::new(),
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

    pub fn keyword_flag(mut self, keyword: KeywordFlag) -> Self {
        self.data.keyword_flags.insert(keyword);
        self
    }

    /// **Affinity for [text]** — CR 702.41a: "This spell costs {1} less to
    /// cast for each [text] you control."
    ///
    /// A keyword that is not a [`KeywordFlag`]: the rule *defines* affinity as
    /// that sentence, so the card carries the sentence and nothing in the
    /// engine knows the word — a static ability whose subject is the spell
    /// itself (CR 113.6d) and whose change is a generic reduction of a count.
    ///
    /// `filter` is the "[text]" — `ObjectFilter::ByType(CardType::Artifact)`
    /// for affinity for artifacts. "You control" is added here, so a card
    /// writes the noun and no more. CR 702.41b — "if a spell has multiple
    /// instances of affinity, each of them applies" — is calling this twice.
    pub fn affinity_for(self, filter: ObjectFilter) -> Self {
        let you_control = ObjectFilter::And(
            Box::new(filter),
            Box::new(ObjectFilter::ByController(PlayerRef::You)),
        );
        self.ability(
            CostModificationDef::itself(CostChange::ReduceGeneric(AmountExpr::CountOf(
                Selector::PermanentsMatching(you_control),
            )))
            .into_ability(),
        )
    }

    pub fn color_indicator(mut self, colors: Vec<Color>) -> Self {
        self.data.color_indicator = Some(colors);
        self
    }

    pub fn ability(mut self, ability: AbilityDef) -> Self {
        Arc::make_mut(&mut self.data.abilities).push(ability);
        self
    }

    /// Shorthand: add a mana ability that taps to produce one mana of the given type.
    /// This is the standard basic land ability.
    pub fn mana_ability_single(mut self, mana_type: ManaType) -> Self {
        Arc::make_mut(&mut self.data.abilities).push(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Mana,
            costs: vec![Cost::TapSelf],
            effect: Effect::Atom(
                Primitive::ProduceMana(ManaOutput {
                    mana: vec![(mana_type, AmountExpr::Fixed(1))],
                    special: vec![],
                }),
                EffectRecipient::Implicit,
            ),
        });

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

    pub fn alternative_cost(mut self, cost: AlternativeCost) -> Self {
        self.data.alternative_costs.push(cost);
        self
    }

    pub fn additional_cost(mut self, cost: AdditionalCost) -> Self {
        self.data.additional_costs.push(cost);
        self
    }

    /// Finish the card, giving every ability def reachable from it an id and
    /// its instances of "target".
    ///
    /// A def still carrying `AbilityId::UNASSIGNED` — what every card file
    /// writes — gets `AbilityId::printed(name, ordinal)`. The printed list
    /// takes ordinals `0..n` in order, so a printed ability's ordinal is its
    /// index in `abilities`, the index `activatable_abilities` hands out; the
    /// defs nested in the printed effects (a granted ability, a token's
    /// abilities) follow from `n`, in `Effect::for_each_ability_def_mut`'s
    /// order. A def that already has an id keeps it, which is what lets a
    /// test author one and read it back through the card.
    ///
    /// `AbilityDef::instances` is overwritten on every def reached, id or no
    /// id, and `CardData::spell_instances` is filled last: from the enchant
    /// ability for an Aura (CR 303.4a), otherwise from the printed spell
    /// ability, otherwise empty — a permanent spell announces nothing.
    pub fn build(mut self) -> Arc<CardData> {
        let name = self.data.name.clone();
        let mut ordinal = 0u32;
        let mut stamp = |def: &mut AbilityDef| {
            if def.id == AbilityId::UNASSIGNED {
                def.id = AbilityId::printed(&name, ordinal);
            }
            ordinal += 1;
            def.instances = def.effect.instances();
        };
        let abilities = Arc::make_mut(&mut self.data.abilities);
        for def in abilities.iter_mut() {
            stamp(def);
        }
        for def in abilities.iter_mut() {
            def.effect.for_each_ability_def_mut(&mut stamp);
        }
        // CR 702.5a — only an Aura carries an enchant ability, and CR 303.4a
        // makes it the spell's one instance of "target".
        self.data.spell_instances = match &self.data.enchant_filter {
            Some(filter) => vec![EffectRecipient::Target(filter.clone(), TargetCount::Exactly(1))],
            None => abilities
                .iter()
                .find(|a| a.ability_type == AbilityType::Spell)
                .map(|spell| spell.instances.clone())
                .unwrap_or_default(),
        };
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

    #[test]
    fn test_card_data_default_no_costs() {
        let card = CardDataBuilder::new("Vanilla Creature").build();
        assert!(card.alternative_costs.is_empty());
        assert!(card.additional_costs.is_empty());
    }

    #[test]
    fn test_card_data_with_kicker() {
        let card = CardDataBuilder::new("Goblin Bushwhacker")
            .card_type(CardType::Creature)
            .mana_cost(ManaCost::build(&[ManaType::Red], 0))
            .additional_cost(AdditionalCost::Kicker(vec![Cost::Mana(
                ManaCost::build(&[ManaType::Red], 0),
            )]))
            .build();

        assert_eq!(card.additional_costs.len(), 1);
        assert!(matches!(&card.additional_costs[0], AdditionalCost::Kicker(_)));
        assert!(card.alternative_costs.is_empty());
    }

    #[test]
    fn test_card_data_with_alternative_cost() {
        let card = CardDataBuilder::new("Force of Will")
            .card_type(CardType::Instant)
            .alternative_cost(AlternativeCost::Custom(
                "Exile a blue card and pay 1 life".to_string(),
                vec![Cost::PayLife(1)],
            ))
            .build();

        assert_eq!(card.alternative_costs.len(), 1);
        assert!(matches!(&card.alternative_costs[0], AlternativeCost::Custom(_, _)));
        assert!(card.additional_costs.is_empty());
    }
}
