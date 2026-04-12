use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, Subtype, Supertype, CreatureType};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Effect, ManaOutput, PermanentFilter, Primitive, TargetCount, TargetSpec};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};

/// Isamaru, Hound of Konda - {W}
/// Legendary Creature - Dog
/// 2/2
pub fn isamaru_hound_of_konda() -> Arc<CardData> {
    CardDataBuilder::new("Isamaru, Hound of Konda")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Dog))
        .power_toughness(2, 2)
        .build()
}

/// Night's Whisper - {1}{B}
/// Sorcery
/// You draw two cards and lose 2 life
pub fn nights_whisper() -> Arc<CardData> {
    CardDataBuilder::new("Night's Whisper")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Sorcery)
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::DrawCards(AmountExpr::Fixed(2)), 
                    TargetSpec::You
                ),
                Effect::Atom(
                    Primitive::LoseLife(AmountExpr::Fixed(2)),
                    TargetSpec::You
                )
            ])
        })
        .build()
}

/// Doom Blade - {1}{B}
/// Instant
/// Destroy target nonblack creature.
pub fn doom_blade() -> Arc<CardData> {
    CardDataBuilder::new("Doom Blade")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::Destroy,
                TargetSpec::Permanent(
                    PermanentFilter::And(
                        Box::new(PermanentFilter::ByType(CardType::Creature)),
                        Box::new(PermanentFilter::Not(Box::new(PermanentFilter::ByColor(Color::Black))))
                    ),
                    TargetCount::Exactly(1)
                )
            )
        })
        .build()
}

pub fn angels_mercy() -> Arc<CardData> {
    CardDataBuilder::new("Angel's Mercy")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(7)), TargetSpec::You)
        })
        .build()
}

/// Dark Ritual - {B}
/// Instant
/// Add {B}{B}{B}.
pub fn dark_ritual() -> Arc<CardData> {
    CardDataBuilder::new("Dark Ritual")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ProduceMana(ManaOutput {
                    mana: vec![(ManaType::Black, AmountExpr::Fixed(3))],
                    special: vec![],
                }),
                TargetSpec::None,
            ),
        })
        .build()
}