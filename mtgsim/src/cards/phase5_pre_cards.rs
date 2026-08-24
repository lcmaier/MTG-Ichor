use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, Subtype, Supertype, CreatureType};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Duration, Effect, ManaOutput, PermanentFilter, PlayerRef, Primitive, TargetCount, EffectRecipient, SelectionFilter};
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
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::DrawCards(AmountExpr::Fixed(2)), 
                    EffectRecipient::Controller
                ),
                Effect::Atom(
                    Primitive::LoseLife(AmountExpr::Fixed(2)),
                    EffectRecipient::Controller
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
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::Destroy,
                EffectRecipient::Target(SelectionFilter::Permanent(
                    PermanentFilter::And(
                        Box::new(PermanentFilter::ByType(CardType::Creature)),
                        Box::new(PermanentFilter::Not(Box::new(PermanentFilter::ByColor(Color::Black))))
                    )),
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
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(7)), EffectRecipient::Controller)
        })
        .build()
}

/// Glorious Anthem - {1}{W}{W}
/// Enchantment
/// Creatures you control get +1/+1.
pub fn glorious_anthem() -> Arc<CardData> {
    CardDataBuilder::new("Glorious Anthem")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ModifyPowerToughness(
                    AmountExpr::Fixed(1),
                    AmountExpr::Fixed(1),
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

/// Zhalfirin Shapecraft - {1}{U}
/// Instant
/// Target creature has base power and toughness 4/3 until end of turn.
/// Draw a card.
pub fn zhalfirin_shapecraft() -> Arc<CardData> {
    CardDataBuilder::new("Zhalfirin Shapecraft")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::SetPowerToughness(
                        AmountExpr::Fixed(4),
                        AmountExpr::Fixed(3),
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

/// Inside Out - {1}{U} **as modeled**; the printed cost is {1}{U/R}
/// Instant
/// Switch target creature's power and toughness until end of turn.
/// Draw a card.
///
/// The hybrid symbol is the whole reason this stays a fixture and never reaches
/// `CardRegistry` — hybrid mana is unimplemented (`codebase-state.md`, CR 6),
/// and a registry that feeds `cli_play` may not quietly recost a card. Layer 7d
/// gets its registered source from `utility_creatures::merfolk_thaumaturgist`,
/// whose text needs nothing the engine lacks.
pub fn inside_out() -> Arc<CardData> {
    CardDataBuilder::new("Inside Out")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::SwitchPowerToughness(Duration::UntilEndOfTurn),
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

/// Bull Rush - {R}
/// Instant
/// Target creature gets +2/+0 until end of turn.
pub fn bull_rush() -> Arc<CardData> {
    CardDataBuilder::new("Bull Rush")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ModifyPowerToughness(
                    AmountExpr::Fixed(2),
                    AmountExpr::Fixed(0),
                    Duration::UntilEndOfTurn,
                ),
                EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            ),
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
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ProduceMana(ManaOutput {
                    mana: vec![(ManaType::Black, AmountExpr::Fixed(3))],
                    special: vec![],
                }),
                EffectRecipient::Implicit,
            ),
        })
        .build()
}