//! Phase LI — the CR 613.8 cluster (`layers-architecture.md` §13b).
//!
//! LI-2's cards: the boards the rulings walk, so that the dependency
//! algorithm is tested against printed text with a published answer, and
//! one fixture for the ruling whose printed card is out of reach. Every
//! oracle text below was verified on Scryfall on 2026-09-06 and is quoted
//! verbatim; the rulings the tests assert are quoted in
//! `tests/phase_li2_integration_test.rs` beside the assertions.
//!
//! `phase_ld_cards::urborg_effect` — the Enchantment stand-in — stays what
//! the CR 305.6 tests rest on. It was an Enchantment precisely so that those
//! tests would not depend on this phase; now that the real Land exists, the
//! two are different fixtures for different rules, not two spellings of one.
//!
//! LI-3's are the conditional statics: Kird Ape, the cheapest printed "as
//! long as" there is, and two fixtures for shapes whose printed cards are out
//! of reach — one Aura clause of Rune of Flight, and a layer-4 condition that
//! another effect in the same layer flips.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, EnchantmentType, LandType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, Condition, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef,
    Primitive, SelectionFilter, Selector, TypeChange,
};
use crate::types::keywords::KeywordFlag;
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};

/// A `TypeChange` that adds and nothing else.
fn adds(types: &[CardType], subtypes: &[Subtype], supertypes: &[Supertype]) -> TypeChange {
    TypeChange {
        add_types: types.to_vec(),
        remove_types: Vec::new(),
        set_types: None,
        add_subtypes: subtypes.to_vec(),
        remove_subtypes: Vec::new(),
        set_subtypes: None,
        add_supertypes: supertypes.to_vec(),
        remove_supertypes: Vec::new(),
        set_supertypes: None,
    }
}

fn static_ability(effect: Effect) -> AbilityDef {
    AbilityDef {
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
        id: new_ability_id(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect,
    }
}

/// Urborg, Tomb of Yawgmoth
/// Legendary Land
/// Each land is a Swamp in addition to its other land types.
///
/// (Oracle text verified on Scryfall, 2026-09-06.)
///
/// The existence dependency (CR 613.8a(b)), on the card whose ruling states
/// it: "If an effect such as that of Magus of the Moon causes Urborg to lose
/// its abilities by setting it to a basic land type not in addition to its
/// other types, it won't turn lands into Swamps, no matter in what order
/// those effects started to apply" (2021-03-19). Urborg is a Legendary Land
/// and so nonbasic (CR 305.8 — the supertype alone decides), so Blood Moon
/// makes it a Mountain and CR 305.7 strips the very ability that generates
/// this effect. Applying Blood Moon changes whether Urborg's effect *exists*,
/// so Urborg waits for Blood Moon whatever their timestamps, and by then it
/// has nothing to apply.
///
/// No printed mana ability: Urborg taps for {B} only because it is itself a
/// land and so a Swamp under its own effect, which is CR 305.6's intrinsic
/// ability arriving through the additive clause of CR 305.7.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The pool's first card whose effect can depend on another's — Blood Moon
/// has been in the pool since Phase LD — and a land, so every deck that
/// draws it drops it. `random_deck` fills nonbasic land slots from the
/// registry's nonbasic lands, which is how it reaches a game at all. That
/// makes the dependency loop's slow path (`board::next_ready`) live in
/// a measured game, where before this card every layer in the pool was
/// pairwise independent under the static check.
pub fn urborg_tomb_of_yawgmoth() -> Arc<CardData> {
    CardDataBuilder::new("Urborg, Tomb of Yawgmoth")
        .supertype(Supertype::Legendary)
        .card_type(CardType::Land)
        .rules_text("Each land is a Swamp in addition to its other land types.")
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeType(
                adds(&[], &[Subtype::Land(LandType::Swamp)], &[]),
                Duration::WhileSourceOnBattlefield,
            ),
            EffectRecipient::FilteredPermanents(ObjectFilter::ByType(CardType::Land)),
        )))
        .build()
}

/// Opalescence — {2}{W}{W}
/// Enchantment
/// Each other non-Aura enchantment is a creature in addition to its other
/// types and has base power and base toughness each equal to its mana value.
///
/// (Oracle text verified on Scryfall, 2026-09-06.)
///
/// One ability, two atoms — a layer 4 part and a layer 7b part sharing one
/// `EffectGroup` — which is the shape CR 613.6 is written about, and its
/// Humility rulings (2009-10-01, 2006-02-01) walk layers 4, 6 and 7b with
/// timestamps and are this engine's CR 613.6 test with the CR's own answers
/// attached (`codebase-state.md` "Before Layers" 7c). Under Humility an
/// Opalescence animated by another loses this ability in layer 6, and its
/// 7b part still applies to the set it locked in layer 4.
///
/// "Each other" is `ObjectFilter::EachOther`, the leaf this card is the first
/// consumer of; its 2004-10-04 ruling is the leaf's test: "Does not animate
/// itself. But can be animated by another Opalescence."
///
/// Registered, not pooled: it opens no engine path Urborg does not (its
/// order against Humility is the timestamp's, which LI-1 already gave), and
/// animating every enchantment in a random game would move every
/// behavioural row of §3's table for a reason that is not the engine's.
pub fn opalescence() -> Arc<CardData> {
    let other_non_aura_enchantments = EffectRecipient::FilteredPermanents(ObjectFilter::And(
        Box::new(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Enchantment)),
            Box::new(ObjectFilter::Not(Box::new(ObjectFilter::BySubtype(
                Subtype::Enchantment(EnchantmentType::Aura),
            )))),
        )),
        Box::new(ObjectFilter::EachOther),
    ));

    CardDataBuilder::new("Opalescence")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text(
            "Each other non-Aura enchantment is a creature in addition to its other types and \
             has base power and base toughness each equal to its mana value.",
        )
        .ability(static_ability(Effect::Sequence(vec![
            Effect::Atom(
                Primitive::ChangeType(
                    adds(&[CardType::Creature], &[], &[]),
                    Duration::WhileSourceOnBattlefield,
                ),
                other_non_aura_enchantments.clone(),
            ),
            Effect::Atom(
                Primitive::SetPowerToughness(
                    AmountExpr::AffectedManaValue,
                    AmountExpr::AffectedManaValue,
                    Duration::WhileSourceOnBattlefield,
                ),
                other_non_aura_enchantments,
            ),
        ])))
        .build()
}

/// Ashaya, Soul of the Wild — {3}{G}{G}
/// Legendary Creature — Elemental
/// */*
/// Ashaya's power and toughness are each equal to the number of lands you
/// control.
/// Nontoken creatures you control are Forest lands in addition to their
/// other types. (They're still affected by summoning sickness.)
///
/// (Oracle text verified on Scryfall, 2026-09-06.)
///
/// The applies-to dependency (CR 613.8a(b)) on a printed card: applying
/// Ashaya makes creatures nonbasic lands, which changes what Blood Moon
/// applies to, so Blood Moon waits for Ashaya — "if there are nontoken
/// creatures", as the judge answer in `plans/references/` puts it, because
/// the dependency is decided against the board being built. **No ruling
/// covers Ashaya beside Blood Moon**; the expected answer in the tests is
/// derived from the CR and says so: Ashaya first, then Blood Moon makes the
/// creature-lands Mountains that lose their abilities (CR 305.7), Ashaya's
/// own CDA included, so Ashaya is 0/0.
///
/// Two abilities. The first is a CDA (CR 604.3a) and counts lands *as of
/// layer 7a* — after its own second ability has made it a land — which is
/// its 2020-09-25 ruling: "it's affected by its second ability and thus its
/// first ability counts itself". The second lowers to two layer-4 rows,
/// `AddType(Land)` and `AddSubtype(Forest)`, and it is why the pass orders
/// *effects* rather than rows (`board::Kind::Effect`).
///
/// In `stress` only: the shape is Urborg's, measured there, and five mana
/// is a rare cast in a random game.
pub fn ashaya_soul_of_the_wild() -> Arc<CardData> {
    let lands_you_control = AmountExpr::CountOf(Selector::PermanentsMatching(ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Land)),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    )));
    let nontoken_creatures_you_control = ObjectFilter::And(
        Box::new(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Creature)),
            Box::new(ObjectFilter::ByController(PlayerRef::You)),
        )),
        Box::new(ObjectFilter::Not(Box::new(ObjectFilter::Token))),
    );

    CardDataBuilder::new("Ashaya, Soul of the Wild")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 3))
        .color(Color::Green)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elemental))
        // `*/*` — the CDA supplies both numbers, in every zone (CR 208.2a).
        .power_toughness(0, 0)
        .rules_text(
            "Ashaya's power and toughness are each equal to the number of lands you control.\n\
             Nontoken creatures you control are Forest lands in addition to their other types. \
             (They're still affected by summoning sickness.)",
        )
        .ability(AbilityDef {
            is_characteristic_defining: true,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::SetPowerToughness(
                    lands_you_control.clone(),
                    lands_you_control,
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::Implicit,
            ),
        })
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeType(
                adds(&[CardType::Land], &[Subtype::Land(LandType::Forest)], &[]),
                Duration::WhileSourceOnBattlefield,
            ),
            EffectRecipient::FilteredPermanents(nontoken_creatures_you_control),
        )))
        .build()
}

/// Purifier Clause — {3}{G}
/// Creature — Elf Druid, 3/4
/// Lands you control are basic.
///
/// **A fixture, and an invented name** (`engineering-practices.md` §3's
/// rule): the battlefield half of Rootpath Purifier's "Lands you control and
/// land cards in your library are basic", without the library clause, which
/// waits on `codebase-state.md` layers item 9. Registering the printed card
/// without that clause would wear its name while behaving differently, so
/// this one wears its own and is registered in no pool.
///
/// It is the board of the Purifier's 2022-10-14 ruling: "Lands that become
/// basic are no longer nonbasic lands. This may change what effects can
/// apply to them. For example, if an opponent controls Blood Moon, an
/// enchantment which says 'Nonbasic lands are Mountains,' and you play
/// Rootpath Purifier, Blood Moon can no longer apply to the lands you
/// control because they are all basic." — the applies-to dependency with a
/// ruling behind it, in both timestamp orders.
pub fn purifier_clause() -> Arc<CardData> {
    CardDataBuilder::new("Purifier Clause")
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elf))
        .subtype(Subtype::Creature(CreatureType::Druid))
        .power_toughness(3, 4)
        .rules_text("Lands you control are basic.")
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeType(
                adds(&[], &[], &[Supertype::Basic]),
                Duration::WhileSourceOnBattlefield,
            ),
            EffectRecipient::FilteredPermanents(ObjectFilter::And(
                Box::new(ObjectFilter::ByType(CardType::Land)),
                Box::new(ObjectFilter::ByController(PlayerRef::You)),
            )),
        )))
        .build()
}

/// Kird Ape — {R}
/// Creature — Ape, 1/1
/// This creature gets +1/+2 as long as you control a Forest.
///
/// (Oracle text verified on Scryfall, 2026-09-06.)
///
/// **The cheapest printed conditional static there is**, and the one that
/// needs no new condition leaf: `ControlPermanent(BySubtype(Forest))` over an
/// `Implicit` recipient, which is one layer-7c row whose *existence* is a
/// question asked every pass (CR 604.2, `board::static_ability_still_exists`).
/// Asymmetric, so a row applied twice or transposed fails an assertion.
///
/// It reads LI-2's board without depending on anything. A Taiga is a Forest
/// until Blood Moon sets it to Mountain in layer 4, and the Ape's condition
/// is read at 7c against a board where that has already happened — so the
/// bonus is simply gone, two layers later, with no dependency involved
/// (CR 613.8a(a) confines dependency to one layer).
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The pool's first row whose existence is a *condition* rather than an
/// ability lookup, which is a new path in the check and one that runs for
/// every application in every layer of every pass the Ape is on the board
/// for. A one-mana creature any red deck casts on turn one, and the pool
/// already has five basic Forests and four nonbasic lands with the subtype,
/// so both answers happen in a measured game.
pub fn kird_ape() -> Arc<CardData> {
    CardDataBuilder::new("Kird Ape")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Ape))
        .power_toughness(1, 1)
        .rules_text("This creature gets +1/+2 as long as you control a Forest.")
        .ability(static_ability(Effect::Conditional(
            Condition::ControlPermanent(ObjectFilter::BySubtype(Subtype::Land(
                LandType::Forest,
            ))),
            Box::new(Effect::Atom(
                Primitive::ModifyPowerToughness(
                    AmountExpr::Fixed(1),
                    AmountExpr::Fixed(2),
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::Implicit,
            )),
        )))
        .build()
}

/// Flight Clause — {1}{U}
/// Enchantment — Aura
/// Enchant permanent
/// As long as enchanted permanent is a creature, it has flying.
///
/// **A fixture, and an invented name** (`engineering-practices.md` §3's
/// rule): the third line of Rune of Flight, whose printed text is
/// "Enchant permanent / When this Aura enters, draw a card. / As long as
/// enchanted permanent is a creature, it has flying. / As long as enchanted
/// permanent is an Equipment, it has 'Equipped creature has flying.'"
/// (Scryfall, 2026-09-06). Two of those four lines are out of reach — the
/// draw is a trigger (critical-path item 6), and the Equipment clause grants
/// a *static* ability from a static ability, which registers no continuous
/// effect (`codebase-state.md` "Before Layers" item 7g). Registering the
/// printed card with either missing would wear its name while behaving
/// differently, so this one wears its own.
///
/// It is CR 613.7a's own worked example one line short, and three phases
/// meet on it: the layer-6 grant over `Host` (LH-1), an Aura whose host can
/// be reattached (LH-2), and the condition (LI-3). Against Humility in either
/// timestamp order it is the one board where all three are observable at
/// once — Humility strips at layer 6 and this grants at layer 6, so the
/// timestamps decide, and the condition decides whether there is anything to
/// order at all.
pub fn flight_clause() -> Arc<CardData> {
    CardDataBuilder::new("Flight Clause")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .rules_text(
            "Enchant permanent\nAs long as enchanted permanent is a creature, it has flying.",
        )
        // CR 702.5a — "Enchant permanent" is this Aura's targeting
        // restriction, and CR 303.4a makes it the spell's target. Permanent,
        // not creature: the condition is what decides whether the grant does
        // anything, and an Aura that could only enchant creatures could not
        // have a condition worth reading.
        .enchant_filter(SelectionFilter::Permanent(ObjectFilter::All))
        .ability(static_ability(Effect::Conditional(
            Condition::HostMatches(ObjectFilter::ByType(CardType::Creature)),
            Box::new(Effect::Atom(
                Primitive::GrantKeywordFlag(KeywordFlag::Flying, Duration::WhileSourceOnBattlefield),
                EffectRecipient::Host,
            )),
        )))
        .build()
}

/// Simian Clause — {1}{G}
/// Enchantment
/// As long as you control a Forest, each creature you control is an Ape in
/// addition to its other types.
///
/// **A fixture, and an invented name** (`engineering-practices.md` §3's
/// rule), for a shape no printed card in reach has: a *layer-4* effect whose
/// condition another layer-4 effect flips. Blood Moon sets a Taiga to
/// Mountain, the Forest goes away, and this effect stops existing — so it
/// depends on Blood Moon (CR 613.8a(b), the existence clause) and waits for
/// it whatever the timestamps say.
///
/// **It is the board `board::condition_reads` exists for.** Without the
/// condition's channels the pair is settled independent by the static check
/// and never reaches the hypothetical: this card's own reads would be its
/// filter's (types, controller), Blood Moon writes subtypes and abilities,
/// and the one read they share — the ability list, for CR 604.2 — is of
/// *this* card's source, which Blood Moon does not reach. The condition is
/// what reads a subtype off another object.
///
/// Kird Ape's board is the same shape two layers apart and needs none of
/// that, which is why this one is not the Ape.
pub fn simian_clause() -> Arc<CardData> {
    CardDataBuilder::new("Simian Clause")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(
            "As long as you control a Forest, each creature you control is an Ape in addition \
             to its other types.",
        )
        .ability(static_ability(Effect::Conditional(
            Condition::ControlPermanent(ObjectFilter::BySubtype(Subtype::Land(
                LandType::Forest,
            ))),
            Box::new(Effect::Atom(
                Primitive::ChangeType(
                    adds(&[], &[Subtype::Creature(CreatureType::Ape)], &[]),
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::FilteredPermanents(ObjectFilter::And(
                    Box::new(ObjectFilter::ByType(CardType::Creature)),
                    Box::new(ObjectFilter::ByController(PlayerRef::You)),
                )),
            )),
        )))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A sanity check on the lowering of each card: the rows a static ability
    /// registers are what the tests reason about.
    #[test]
    fn each_card_lowers_to_the_rows_its_text_describes() {
        use crate::engine::layers::types::{EffectModification, Layer, PtValue};
        use crate::state::game_state::GameState;

        let rows = |card: Arc<CardData>| -> Vec<(Layer, EffectModification)> {
            card.abilities
                .iter()
                .filter(|a| a.ability_type == AbilityType::Static && !a.is_characteristic_defining)
                .flat_map(|a| {
                    GameState::static_ability_atoms(a, &card.name)
                        .into_iter()
                        .flat_map(|(p, _)| GameState::static_primitive_rows(p))
                        .collect::<Vec<_>>()
                })
                .collect()
        };

        assert_eq!(
            rows(urborg_tomb_of_yawgmoth()),
            vec![(Layer::Layer4Type, EffectModification::AddSubtype(Subtype::Land(LandType::Swamp)))]
        );
        assert_eq!(rows(opalescence()).iter().map(|(l, _)| *l).collect::<Vec<_>>(), vec![
            Layer::Layer4Type,
            Layer::Layer7bSetPT
        ]);
        // One ability, two layer-4 rows — one effect for CR 613.8's purposes.
        assert_eq!(rows(ashaya_soul_of_the_wild()), vec![
            (Layer::Layer4Type, EffectModification::AddType(CardType::Land)),
            (Layer::Layer4Type, EffectModification::AddSubtype(Subtype::Land(LandType::Forest))),
        ]);
        assert_eq!(rows(purifier_clause()), vec![(
            Layer::Layer4Type,
            EffectModification::AddSupertype(Supertype::Basic)
        )]);

        // LI-3 — a conditional static lowers to exactly the rows its inner
        // atom does. The condition is nowhere in this list, which is the
        // whole of §13b decision 5: it stays on the ability, and CR 604.2's
        // existence check reads it there every layer.
        assert_eq!(rows(kird_ape()), vec![(
            Layer::Layer7cModifyPT,
            EffectModification::ModifyPowerToughness {
                power: PtValue::Fixed(1),
                toughness: PtValue::Fixed(2),
            }
        )]);
        assert_eq!(rows(flight_clause()), vec![(
            Layer::Layer6Ability,
            EffectModification::GrantKeywordFlag(KeywordFlag::Flying)
        )]);
        assert_eq!(rows(simian_clause()), vec![(
            Layer::Layer4Type,
            EffectModification::AddSubtype(Subtype::Creature(CreatureType::Ape))
        )]);

        let ashaya = ashaya_soul_of_the_wild();
        assert!(ashaya.abilities[0].is_characteristic_defining, "the P/T ability is a CDA");
        assert!(!ashaya.abilities[1].is_characteristic_defining);
    }
}
