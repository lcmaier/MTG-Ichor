use std::collections::HashSet;
use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, LandType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, PermanentFilter, Primitive, SelectionFilter,
    TargetCount, TypeChange,
};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};

// ===========================================================================
// Part A test cards: basic type-changing operations (no 305.7 semantics)
// ===========================================================================

/// Liquimetal Coating (simplified) — {2}
/// Artifact
/// {T}: Target permanent becomes an artifact in addition to its other types
/// until end of turn.
///
/// Modeled as an instant spell for testing (avoids activated ability infrastructure).
/// Tests: AddType (adding Artifact to any permanent).
pub fn liquimetal_coating_spell() -> Arc<CardData> {
    CardDataBuilder::new("Liquimetal Torque")
        .mana_cost(ManaCost::build(&[ManaType::Colorless], 1))
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ChangeType(
                    TypeChange {
                        add_types: vec![CardType::Artifact],
                        remove_types: Vec::new(),
                        set_types: None,
                        add_subtypes: Vec::new(),
                        remove_subtypes: Vec::new(),
                        set_subtypes: None,
                        add_supertypes: Vec::new(),
                        remove_supertypes: Vec::new(),
                        set_supertypes: None,
                    },
                    Duration::UntilEndOfTurn,
                ),
                EffectRecipient::Target(
                    SelectionFilter::Permanent(PermanentFilter::All),
                    TargetCount::Exactly(1),
                ),
            ),
        })
        .build()
}

/// Ensoul Artifact (simplified) — {1}{U}
/// Instant
/// Target artifact becomes an artifact creature with base power and toughness
/// 5/5 until end of turn.
///
/// Both atoms share `ctx.targets` — a Sequence targeting the same object.
/// Tests: AddType(Creature) + SetPowerToughness on the same target.
pub fn ensoul_artifact_spell() -> Arc<CardData> {
    CardDataBuilder::new("Ensoul Artifact")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::ChangeType(
                        TypeChange {
                            add_types: vec![CardType::Creature],
                            remove_types: Vec::new(),
                            set_types: None,
                            add_subtypes: Vec::new(),
                            remove_subtypes: Vec::new(),
                            set_subtypes: None,
                            add_supertypes: Vec::new(),
                            remove_supertypes: Vec::new(),
                            set_supertypes: None,
                        },
                        Duration::UntilEndOfTurn,
                    ),
                    // Both atoms use the same recipient — targets are shared via ctx.targets
                    EffectRecipient::Target(
                        SelectionFilter::Permanent(PermanentFilter::ByType(CardType::Artifact)),
                        TargetCount::Exactly(1),
                    ),
                ),
                Effect::Atom(
                    Primitive::SetPowerToughness(
                        AmountExpr::Fixed(5),
                        AmountExpr::Fixed(5),
                        Duration::UntilEndOfTurn,
                    ),
                    EffectRecipient::Target(
                        SelectionFilter::Permanent(PermanentFilter::ByType(CardType::Artifact)),
                        TargetCount::Exactly(1),
                    ),
                ),
            ]),
        })
        .build()
}

/// Call to Serve (simplified) — {1}{W}
/// Instant (simplified from Aura for testing)
/// Target creature gets +1/+2 and is an Angel in addition to its other
/// types until end of turn.
///
/// Tests: AddSubtype (creature subtype — Angel) combined with P/T pump.
/// Note: GrantKeyword(Flying) omitted — Layer 6 resolution not yet implemented.
pub fn call_to_serve_spell() -> Arc<CardData> {
    CardDataBuilder::new("Call to Serve")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::ChangeType(
                        TypeChange {
                            add_types: Vec::new(),
                            remove_types: Vec::new(),
                            set_types: None,
                            add_subtypes: vec![Subtype::Creature(CreatureType::Angel)],
                            remove_subtypes: Vec::new(),
                            set_subtypes: None,
                            add_supertypes: Vec::new(),
                            remove_supertypes: Vec::new(),
                            set_supertypes: None,
                        },
                        Duration::UntilEndOfTurn,
                    ),
                    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                ),
                Effect::Atom(
                    Primitive::ModifyPowerToughness(
                        AmountExpr::Fixed(1),
                        AmountExpr::Fixed(2),
                        Duration::UntilEndOfTurn,
                    ),
                    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                ),
            ]),
        })
        .build()
}

/// On Serra's Wings (simplified) — {3}{W}
/// Instant (simplified from Legendary Enchantment — Aura)
/// Target creature is legendary and gets +1/+1 until end of turn.
///
/// Tests: AddSupertype (Legendary) combined with P/T pump.
/// Note: GrantKeyword atoms omitted — Layer 6 resolution not yet implemented.
pub fn on_serras_wings_spell() -> Arc<CardData> {
    CardDataBuilder::new("On Serra's Wings")
        .mana_cost(ManaCost::build(&[ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::ChangeType(
                        TypeChange {
                            add_types: Vec::new(),
                            remove_types: Vec::new(),
                            set_types: None,
                            add_subtypes: Vec::new(),
                            remove_subtypes: Vec::new(),
                            set_subtypes: None,
                            add_supertypes: vec![Supertype::Legendary],
                            remove_supertypes: Vec::new(),
                            set_supertypes: None,
                        },
                        Duration::UntilEndOfTurn,
                    ),
                    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                ),
                Effect::Atom(
                    Primitive::ModifyPowerToughness(
                        AmountExpr::Fixed(1),
                        AmountExpr::Fixed(1),
                        Duration::UntilEndOfTurn,
                    ),
                    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                ),
            ]),
        })
        .build()
}

// ===========================================================================
// Part B test cards: CR 305.7 semantics (Blood Moon, Urborg)
// These will be fully testable after AbilityOrigin infrastructure is added.
// ===========================================================================

/// Blood Moon — {2}{R}
/// Enchantment
/// Nonbasic lands are Mountains.
/// (Static ability: filter = lands without Basic supertype, SetSubtypes({Mountain}))
pub fn blood_moon() -> Arc<CardData> {
    let mut mountain_set = HashSet::new();
    mountain_set.insert(Subtype::Land(LandType::Mountain));

    // Filter: Land AND NOT Basic
    let nonbasic_land_filter = PermanentFilter::And(
        Box::new(PermanentFilter::ByType(CardType::Land)),
        Box::new(PermanentFilter::Not(
            Box::new(PermanentFilter::BySupertype(Supertype::Basic)),
        )),
    );

    CardDataBuilder::new("Blood Moon")
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Enchantment)
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ChangeType(
                    TypeChange {
                        add_types: Vec::new(),
                        remove_types: Vec::new(),
                        set_types: None,
                        add_subtypes: Vec::new(),
                        remove_subtypes: Vec::new(),
                        set_subtypes: Some(mountain_set),
                        add_supertypes: Vec::new(),
                        remove_supertypes: Vec::new(),
                        set_supertypes: None,
                    },
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::FilteredPermanents(nonbasic_land_filter),
            ),
        })
        .build()
}

/// Urborg-style effect (simplified) — {B}
/// Enchantment
/// Each land is a Swamp in addition to its other land types.
/// (Static ability: filter = all lands, AddSubtype(Swamp))
pub fn urborg_effect() -> Arc<CardData> {
    CardDataBuilder::new("Urborg Effect")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ChangeType(
                    TypeChange {
                        add_types: Vec::new(),
                        remove_types: Vec::new(),
                        set_types: None,
                        add_subtypes: vec![Subtype::Land(LandType::Swamp)],
                        remove_subtypes: Vec::new(),
                        set_subtypes: None,
                        add_supertypes: Vec::new(),
                        remove_supertypes: Vec::new(),
                        set_supertypes: None,
                    },
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::FilteredPermanents(PermanentFilter::ByType(CardType::Land)),
            ),
        })
        .build()
}
