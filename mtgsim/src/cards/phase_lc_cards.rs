use std::collections::HashSet;
use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, ColorChange, Duration, Effect, EffectRecipient, PermanentFilter, PlayerRef,
    Primitive, SelectionFilter, TargetCount,
};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};

/// Cerulean Wisps — {U}
/// Instant
/// Target creature becomes blue until end of turn.
/// Draw a card.
pub fn cerulean_wisps() -> Arc<CardData> {
    let mut blue_set = HashSet::new();
    blue_set.insert(Color::Blue);
    CardDataBuilder::new("Cerulean Wisps")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::ChangeColor(
                        ColorChange::Set(blue_set),
                        Duration::UntilEndOfTurn,
                    ),
                    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                ),
                Effect::Atom(
                    Primitive::DrawCards(AmountExpr::Fixed(1)),
                    EffectRecipient::Controller,
                ),
            ]),
        })
        .build()
}

/// Moonlace — {U}
/// Instant
/// Target spell or permanent becomes colorless.
/// (Simplified: target creature becomes colorless until end of turn.)
pub fn moonlace() -> Arc<CardData> {
    CardDataBuilder::new("Moonlace")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ChangeColor(
                    ColorChange::RemoveAll,
                    Duration::UntilEndOfTurn,
                ),
                EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            ),
        })
        .build()
}

/// Crimson Wisps — {R}
/// Instant
/// Target creature gains haste and becomes red until end of turn.
/// Draw a card.
/// (Tests multi-layer effect from one spell: Layer 5 color + Layer 6 keyword)
pub fn crimson_wisps() -> Arc<CardData> {
    use crate::types::keywords::KeywordFlag;

    let mut red_set = HashSet::new();
    red_set.insert(Color::Red);
    CardDataBuilder::new("Crimson Wisps")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::GrantKeywordFlag(KeywordFlag::Haste, Duration::UntilEndOfTurn),
                    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                ),
                Effect::Atom(
                    Primitive::ChangeColor(
                        ColorChange::Set(red_set),
                        Duration::UntilEndOfTurn,
                    ),
                    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                ),
                Effect::Atom(
                    Primitive::DrawCards(AmountExpr::Fixed(1)),
                    EffectRecipient::Controller,
                ),
            ]),
        })
        .build()
}

/// Chromatic Ward (made-up) — {1}{R}
/// Enchantment
/// Creatures you control are red in addition to their other colors.
/// (Tests static ability color-adding via Layer 5 + register_static_effects)
pub fn chromatic_ward() -> Arc<CardData> {
    CardDataBuilder::new("Chromatic Ward")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Enchantment)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ChangeColor(
                    ColorChange::Add(Color::Red),
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::FilteredPermanents(PermanentFilter::And(
                    Box::new(PermanentFilter::ByType(CardType::Creature)),
                    Box::new(PermanentFilter::ByController(PlayerRef::You)),
                )),
            ),
        })
        .build()
}
