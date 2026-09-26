//! Phase LL — a card in a library or a hand is walked only when something
//! reads it (`layers-architecture.md` §13e).
//!
//! Fixtures, registered nowhere. Each is a clause of a printed card, or a text
//! no card prints that the CR permits, and each is the smallest board one of
//! the phase's claims needs. Oracle texts were verified on Scryfall on
//! 2026-09-25.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, PlaneswalkerType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, Condition, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, TypeChange,
};
use crate::types::ids::AbilityId;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::zones::ZoneSet;

fn static_ability(effect: Effect) -> AbilityDef {
    AbilityDef {
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect,
    }
}

/// A type change that adds and removes nothing but `add_types` and
/// `add_subtypes`.
fn adding(add_types: Vec<CardType>, add_subtypes: Vec<Subtype>) -> TypeChange {
    TypeChange {
        add_types,
        remove_types: Vec::new(),
        set_types: None,
        add_subtypes,
        remove_subtypes: Vec::new(),
        set_subtypes: None,
        add_supertypes: Vec::new(),
        remove_supertypes: Vec::new(),
        set_supertypes: None,
    }
}

/// Grist, the Hunger Tide — {1}{B}{G}
/// Legendary Planeswalker — Grist, loyalty 3
/// "As long as Grist isn't on the battlefield, it's a 1/1 Insect creature in
/// addition to its other types."
///
/// **Its first ability on its printed frame, and none of its three loyalty
/// abilities**, which wait on `backlog.md` §2.11 (no loyalty ability exists)
/// and, for the −2's reflexive trigger, TR-3. The only printed card whose
/// static ability changes its own card everywhere but the battlefield
/// (Scryfall, two phrasings, 2026-09-25).
///
/// Its rulings are the tests: "Anywhere but on the battlefield, Grist is a
/// Legendary Planeswalker Creature — Grist Insect", so on the stack it is a
/// creature spell (Essence Scatter can counter it, and Thalia, Guardian of
/// Thraben does not tax it), and once it enters it is a planeswalker and
/// nothing else. The clause is a `SourceOnly` row, which reaches Grist only
/// because Grist is a member of every pass wherever it is
/// (`layers::board::source_joins`).
pub fn grist_insect_clause() -> Arc<CardData> {
    CardDataBuilder::new("Grist's Insect Clause")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Green], 1))
        .color(Color::Black)
        .color(Color::Green)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Planeswalker)
        .subtype(Subtype::Planeswalker(PlaneswalkerType::Grist))
        .loyalty(3)
        .rules_text("As long as Grist isn't on the battlefield, it's a 1/1 Insect creature in addition to its other types.")
        .ability(static_ability(Effect::Conditional(
            // CR 113.6c — the ability says where it does not function.
            Condition::SourceInZone(ZoneSet::EVERYWHERE_BUT_BATTLEFIELD),
            Box::new(Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::ChangeType(
                        adding(vec![CardType::Creature], vec![Subtype::Creature(CreatureType::Insect)]),
                        Duration::WhileSourceOnBattlefield,
                    ),
                    EffectRecipient::ThisObject,
                ),
                Effect::Atom(
                    Primitive::SetPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(1), Duration::WhileSourceOnBattlefield),
                    EffectRecipient::ThisObject,
                ),
            ])),
        )))
        .build()
}

/// Titania's Song — {3}{G}, Enchantment
/// "Each noncreature artifact loses all abilities and becomes an artifact
/// creature with power and toughness each equal to its mana value."
///
/// **The first sentence.** The second, "If this enchantment leaves the
/// battlefield, this effect continues until end of turn", extends a duration
/// past its source, which `Duration` cannot say.
///
/// **The test board for noting each row at its own layer** (§13e decision 1),
/// in the shape every layer-6 "loses all abilities" has: Humility on
/// Painter's Servant is the same board.
/// Beside Mycosynth Lattice's colorless line the artifact loses the ability
/// at layer 6, but the line applies at layer 5, while the ability is still
/// there — so a card in a library is colorless. A walk that asked the memo's
/// settled Lattice would find no ability and skip the row. One ability, three
/// rows, one effect: CR 613.6 locks the noncreature artifacts at layer 4 and
/// applies the strip and the P/T to the same set.
pub fn titanias_song_clause() -> Arc<CardData> {
    let noncreature_artifact = || {
        EffectRecipient::FilteredPermanents(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Artifact)),
            Box::new(ObjectFilter::Not(Box::new(ObjectFilter::ByType(CardType::Creature)))),
        ))
    };
    CardDataBuilder::new("Titania's Song's First Sentence")
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text("Each noncreature artifact loses all abilities and becomes an artifact creature with power and toughness each equal to its mana value.")
        .ability(static_ability(Effect::Sequence(vec![
            Effect::Atom(
                Primitive::ChangeType(adding(vec![CardType::Creature], vec![]), Duration::WhileSourceOnBattlefield),
                noncreature_artifact(),
            ),
            Effect::Atom(Primitive::LoseAllAbilities(Duration::WhileSourceOnBattlefield), noncreature_artifact()),
            Effect::Atom(
                Primitive::SetPowerToughness(
                    AmountExpr::AffectedManaValue,
                    AmountExpr::AffectedManaValue,
                    Duration::WhileSourceOnBattlefield,
                ),
                noncreature_artifact(),
            ),
        ])))
        .build()
}

/// **Fixture.** "As long as this card is in your hand, creatures you control
/// have flying." — Wonder's clause, one zone over, and a text no card prints
/// (`phase_lk_cards::exiled_ancestor` says why none does).
///
/// **A hidden static source** (§13e decision 2). Beside Hollow Hands the
/// grant and the strip both apply at layer 6, and the grant waits on the strip
/// (CR 613.8a) only if the pass sees the strip reach the card, so the card
/// joins the pass. Walked alone it would be read as of the end of layer 5,
/// still granting.
pub fn pocket_griffin() -> Arc<CardData> {
    CardDataBuilder::new("Pocket Griffin")
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Griffin))
        .power_toughness(2, 2)
        .rules_text("As long as this card is in your hand, creatures you control have flying.")
        .ability(static_ability(Effect::Conditional(
            Condition::SourceInZone(ZoneSet::HAND),
            Box::new(Effect::Atom(
                Primitive::GrantKeywordFlag(KeywordFlag::Flying, Duration::WhileSourceOnBattlefield),
                EffectRecipient::FilteredPermanents(ObjectFilter::And(
                    Box::new(ObjectFilter::ByType(CardType::Creature)),
                    Box::new(ObjectFilter::ByController(PlayerRef::You)),
                )),
            )),
        )))
        .build()
}

/// Arcane Adaptation — {2}{U}, Enchantment
/// "As this enchantment enters, choose a creature type. Creatures you control
/// are the chosen type in addition to their other types. The same is true for
/// creature spells you control and creature cards you own that aren't on the
/// battlefield."
///
/// **The last clause, with Elf chosen**: the half that reaches a library and a
/// hand. Beside Grist in a hand it is the printed case of a CR 613.8
/// dependency decided through a hidden card — Grist's own effect makes it a
/// creature card, so this one depends on it — and Grist joins the pass as its
/// row's source, which is how the pass sees it (§13e).
pub fn arcane_adaptation_elf_clause() -> Arc<CardData> {
    CardDataBuilder::new("Arcane Adaptation's Elf Clause")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text("Creature cards you own that aren't on the battlefield are Elves in addition to their other types.")
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeType(
                adding(vec![], vec![Subtype::Creature(CreatureType::Elf)]),
                Duration::WhileSourceOnBattlefield,
            ),
            EffectRecipient::FilteredObjectsIn(
                ObjectFilter::And(
                    Box::new(ObjectFilter::ByType(CardType::Creature)),
                    Box::new(ObjectFilter::ByOwner(PlayerRef::You)),
                ),
                ZoneSet::EVERYWHERE_BUT_BATTLEFIELD,
            ),
        )))
        .build()
}

/// **Fixture.** "Creature cards in libraries are artifacts in addition to
/// their other types." — Biotransference's shape, reaching libraries alone.
///
/// Half of the guard's pair (§13e decision 4, (b)); `library_assassins` is
/// the other.
pub fn library_artificer() -> Arc<CardData> {
    CardDataBuilder::new("Library Artificer")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text("Creature cards in libraries are artifacts in addition to their other types.")
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeType(adding(vec![CardType::Artifact], vec![]), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredObjectsIn(ObjectFilter::ByType(CardType::Creature), ZoneSet::LIBRARY),
        )))
        .build()
}

/// **Fixture.** "Artifact creature cards in libraries are Assassins in
/// addition to their other types."
///
/// **A dependency no printed card makes, decided through cards in a
/// library** (§13e decision 4, (b)). Beside `library_artificer` it depends on
/// the artificer's effect, which makes the creature cards artifacts. The
/// dependency is real only through a library card, so the guard keeps the
/// libraries in the pass: left out, the pass would see no dependency and
/// apply the older effect first.
pub fn library_assassins() -> Arc<CardData> {
    CardDataBuilder::new("Library Assassins")
        .mana_cost(ManaCost::build(&[ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("Artifact creature cards in libraries are Assassins in addition to their other types.")
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeType(
                adding(vec![], vec![Subtype::Creature(CreatureType::Assassin)]),
                Duration::WhileSourceOnBattlefield,
            ),
            EffectRecipient::FilteredObjectsIn(
                ObjectFilter::And(
                    Box::new(ObjectFilter::ByType(CardType::Artifact)),
                    Box::new(ObjectFilter::ByType(CardType::Creature)),
                ),
                ZoneSet::LIBRARY,
            ),
        )))
        .build()
}
