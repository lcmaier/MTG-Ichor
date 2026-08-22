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
///
/// **An Enchantment on purpose, and not interchangeable with the real card.**
/// Urborg, Tomb of Yawgmoth is a Legendary *Land*, so it is nonbasic and Blood
/// Moon turns it into a Mountain, stripping the very ability that generates this
/// effect (CR 305.7). That makes Urborg's effect dependent on Blood Moon under
/// CR 613.8a(b), so Blood Moon wins regardless of timestamps. Modeling it as an
/// enchantment keeps that dependency out of the 305.6 tests, which are about the
/// additive clause rather than about effect ordering. Do not use this card to
/// reason about the real Blood Moon / Urborg interaction.
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

/// Underground Sea (simplified) — nonbasic dual land
/// Land — Island Swamp
/// {T}: Add {U}.
/// {T}: Add {B}.
///
/// The board setup for ATOM-305.7-002 and COMP-305.7+305.6-001. The atoms
/// specify a single "{T}: Add {U} or {B}" — modal mana abilities aren't
/// supported yet, so it's modeled as two separate mana abilities. CR 305.7
/// strips both either way, so the distinction doesn't affect what's under test.
///
/// Deliberately has NO Basic supertype, so Blood Moon's "nonbasic lands" filter
/// matches it.
pub fn dual_land_ub() -> Arc<CardData> {
    fn mana_ability(mana_type: ManaType) -> AbilityDef {
        AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Mana,
            costs: vec![crate::types::costs::Cost::Tap],
            effect: Effect::Atom(
                Primitive::ProduceMana(crate::types::effects::ManaOutput {
                    mana: vec![(mana_type, AmountExpr::Fixed(1))],
                    special: vec![],
                }),
                EffectRecipient::Implicit,
            ),
        }
    }

    CardDataBuilder::new("Underground Sea")
        .card_type(CardType::Land)
        .subtype(Subtype::Land(LandType::Island))
        .subtype(Subtype::Land(LandType::Swamp))
        .rules_text("{T}: Add {U}. {T}: Add {B}.")
        .ability(mana_ability(ManaType::Blue))
        .ability(mana_ability(ManaType::Black))
        .build()
}

/// Windswept Heights (invented) — {1}{W}
/// Enchantment
/// Lands have flying.
///
/// Not "lands you control": the filter is a bare `ByType(Land)`. Same as
/// [`land_creatures_have_flying`] — the text matches the filter.
///
/// Nonsense as a Magic card, but it is the only ability-granting channel that
/// exists today: `Primitive::GrantKeyword` registers a real Layer 6 effect
/// through `register_static_effects`. Used by ATOM-305.7-003 to show that an
/// ability granted by another effect survives Blood Moon — CR 305.7's "this
/// doesn't remove any abilities that were granted to the land by other effects".
pub fn lands_have_flying() -> Arc<CardData> {
    CardDataBuilder::new("Windswept Heights")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("Lands have flying.")
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::GrantKeyword(
                    crate::types::keywords::KeywordAbility::Flying,
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::FilteredPermanents(PermanentFilter::ByType(CardType::Land)),
            ),
        })
        .build()
}

// ===========================================================================
// CR 613.6 test cards — an effect that applies in two layers, where the first
// layer's part breaks the filter the second layer's part reads.
// ===========================================================================

/// March of the Machines (simplified) — {4}{U}
/// Enchantment
/// Each noncreature artifact is an artifact creature with power and toughness
/// each equal to its mana value.
///
/// The CR's own worked example for rule 613.6, and a real card rather than an
/// invented one. The static ability applies in two layers over one filter:
/// `AddType(Creature)` in Layer 4, `SetPowerToughness` in Layer 7b, both over
/// "artifact that is not a creature". After Layer 4 runs, nothing matches that
/// filter any more — which is exactly why 613.6 says the effect "will continue
/// to be applied to the same set of objects in each other applicable layer".
///
/// **Simplified P/T.** The printed card sets power and toughness to mana value;
/// `register_static_effects` only understands `AmountExpr::Fixed`, so this is a
/// flat 2/2. The simplification is orthogonal to what the card is here to test.
pub fn march_of_the_machines() -> Arc<CardData> {
    // Artifact AND NOT Creature
    let noncreature_artifact = PermanentFilter::And(
        Box::new(PermanentFilter::ByType(CardType::Artifact)),
        Box::new(PermanentFilter::Not(Box::new(PermanentFilter::ByType(
            CardType::Creature,
        )))),
    );

    CardDataBuilder::new("March of the Machines")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 4))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text("Each noncreature artifact is an artifact creature with power and toughness each equal to its mana value.")
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Static,
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
                        Duration::WhileSourceOnBattlefield,
                    ),
                    EffectRecipient::FilteredPermanents(noncreature_artifact.clone()),
                ),
                Effect::Atom(
                    Primitive::SetPowerToughness(
                        AmountExpr::Fixed(2),
                        AmountExpr::Fixed(2),
                        Duration::WhileSourceOnBattlefield,
                    ),
                    EffectRecipient::FilteredPermanents(noncreature_artifact),
                ),
            ]),
        })
        .build()
}

/// Cloudspire Mesa (invented) — nonbasic land
/// Land
/// Creatures have flying.
///
/// Not "creatures you control" — the filter is a bare `ByType(Creature)` with
/// no controller constraint, and the text says what the filter does. Adding
/// `ByController(PlayerRef::You)` would work (`extract_controller_from_filter`
/// reads it), but controller scoping is not what this card is here to exercise.
///
/// The Land-typed sibling of [`lands_have_flying`], and it exists because Blood
/// Moon's filter is "nonbasic **land**". Testing that a stripped static ability
/// retires the continuous effect it generated needs a permanent that (a) Blood
/// Moon can strip — so, a land without the `Basic` supertype — and (b) has a
/// static ability generating a registered effect. Nothing in the card pool is
/// both, which is why item 7 of Deferred Migrations went unnoticed: the bug is
/// live, but no card reaches it.
///
/// Nonsense as a Magic card, in the same way and for the same reason as
/// `lands_have_flying`. `Primitive::GrantKeyword` is used because it is the one
/// Layer 6 channel wired end to end today.
///
/// Deliberately has NO `Basic` supertype, so Blood Moon's filter matches it.
pub fn land_creatures_have_flying() -> Arc<CardData> {
    CardDataBuilder::new("Cloudspire Mesa")
        .card_type(CardType::Land)
        .rules_text("Creatures have flying.")
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::GrantKeyword(
                    crate::types::keywords::KeywordAbility::Flying,
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::FilteredPermanents(PermanentFilter::ByType(CardType::Creature)),
            ),
        })
        .build()
}

/// Moonlit Steppe (invented) — nonbasic land
/// Land
/// Nonbasic lands are Mountains.
///
/// Blood Moon's ability, printed on a land that its own effect then matches. A
/// deliberately compressed stand-in for a board the CR permits and real cards
/// reach: Opalescence makes Blood Moon a creature, Ashaya, Soul of the Wild
/// makes each nontoken creature you control a Forest land, and Blood Moon —
/// now a nonbasic land — strips its own ability under CR 305.7.
///
/// Compressed because our `PermanentFilter` has no way to say "each **other**
/// non-Aura enchantment", so a modelled Opalescence would also enchant itself
/// and put three layers of noise between the fixture and what it is testing.
/// One card reaches the same self-reference.
///
/// What it is for: proving the CR 613.7a existence check terminates. The check
/// asks whether this land still has its static ability, which is a
/// characteristics query on the land, which runs the check again. It resolves
/// because the question is asked at a strictly lower layer ceiling each time
/// (`layers-architecture.md` §5.2) — at Layer 4 it reads the frame as of the
/// end of Layer 3, where the strip has not run. Get that wrong and this card
/// overflows the stack.
pub fn self_stripping_land() -> Arc<CardData> {
    let mut mountain_set = HashSet::new();
    mountain_set.insert(Subtype::Land(LandType::Mountain));

    let nonbasic_land_filter = PermanentFilter::And(
        Box::new(PermanentFilter::ByType(CardType::Land)),
        Box::new(PermanentFilter::Not(Box::new(
            PermanentFilter::BySupertype(Supertype::Basic),
        ))),
    );

    CardDataBuilder::new("Moonlit Steppe")
        .card_type(CardType::Land)
        .rules_text("Nonbasic lands are Mountains.")
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
