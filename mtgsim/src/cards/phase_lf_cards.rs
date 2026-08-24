//! Cards for the Layer 6 phase — ability adding and removing (CR 613.1f).

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, ManaOutput, PermanentFilter, PlayerRef,
    Primitive,
};
use crate::types::costs::Cost;
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};

/// Humility — {2}{W}{W}
/// Enchantment
/// All creatures lose all abilities and have base power and toughness 1/1.
///
/// The Layer 6 card, and the one the corpus names in ATOM-613.1f-001 and
/// COMP-613-TARMOGOYF-HUMILITY-001. Real text, verbatim; nothing here is
/// simplified.
///
/// **One ability, two atoms, and that matters.** The card is a single static
/// `AbilityDef` whose effect is a `Sequence`: `LoseAllAbilities` in Layer 6 and
/// `SetPowerToughness(1, 1)` in Layer 7b, both over "each creature". They
/// therefore share an `EffectGroup` and a timestamp, which is what CR 613.6
/// wants — "if an effect starts to apply in one layer, it will continue to be
/// applied to the same set of objects in each other applicable layer". Split
/// into two `AbilityDef`s they would be two groups, and the 7b half would
/// re-filter against a board the 6 half had already changed.
///
/// **It does not strip itself.** Humility is an Enchantment and its filter is
/// creatures, so CR 613.7a's existence check finds its ability intact at every
/// layer. That is why this card, unlike `phase_ld_cards::moonlit_steppe`, needs
/// nothing from the frame-cache termination argument.
///
/// **What it cannot do yet.** Humility + Opalescence is the famous case, and it
/// is out of reach: it needs a second Layer 7b effect whose order against
/// Humility's is decided by CR 613.8 dependency plus intra-7b timestamp. See
/// the Scryfall rulings and `codebase-state.md` item 8. Humility against a
/// single opponent's creatures — including a Tarmogoyf, whose CDA dies in
/// Layer 6 before Layer 7a can read it — is fully in scope and is what the
/// tests exercise.
pub fn humility() -> Arc<CardData> {
    let creatures = EffectRecipient::FilteredPermanents(PermanentFilter::ByType(CardType::Creature));

    CardDataBuilder::new("Humility")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("All creatures lose all abilities and have base power and toughness 1/1.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::LoseAllAbilities(Duration::WhileSourceOnBattlefield),
                    creatures.clone(),
                ),
                Effect::Atom(
                    Primitive::SetPowerToughness(
                        AmountExpr::Fixed(1),
                        AmountExpr::Fixed(1),
                        Duration::WhileSourceOnBattlefield,
                    ),
                    creatures,
                ),
            ]),
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humility_is_one_ability_with_two_atoms() {
        let card = humility();
        assert_eq!(
            card.abilities.len(),
            1,
            "one AbilityDef, so both halves share an EffectGroup (CR 613.6)"
        );
        match &card.abilities[0].effect {
            Effect::Sequence(atoms) => assert_eq!(atoms.len(), 2),
            other => panic!("expected a Sequence, got {other:?}"),
        }
        assert!(!card.abilities[0].is_characteristic_defining);
    }
}

/// Citanul Hierophants — {3}{G}
/// Creature — Human Druid, 3/2
/// Creatures you control have "{T}: Add {G}."
///
/// Real card, verbatim, and here because it is the **only** production card
/// exercising `Primitive::GrantAbility` on a *static* ability. That arm has
/// been in `static_primitive_rows` since the Layer 6 phase with nothing
/// reaching it: every `GrantAbility` in the pool arrives through a resolution
/// (`resolve::register_granted_static_effects`), which is a different code
/// path with a different `AffectedSet` shape — `Fixed`, locked to the spell's
/// targets, rather than a live `Filter`.
///
/// **Why this one works when "a static ability granting a static ability over a
/// filter" does not** (`codebase-state.md` item 7, the open filter half): the
/// granted ability here is a *mana* ability, so it generates no continuous
/// effect of its own. There is nothing to derive, nothing for CR 613.7a to
/// re-check, and the whole card is one Layer 6 row over a filter. Swap the
/// granted body for a static one and it lands in the open case immediately.
///
/// It also crosses the layer system into mana enumeration, which is the part
/// that broke last time: `oracle::mana_helpers::activatable_abilities`,
/// `engine::priority`'s id→index re-derivation, and `engine::cast::
/// activate_ability` must all index the *effective* ability list, or a creature
/// under this card taps for the wrong thing. That coupling is called out in
/// CLAUDE.md and had no card able to test it end to end until now.
pub fn citanul_hierophants() -> Arc<CardData> {
    // The granted body: `{T}: Add {G}`, exactly what a Forest carries.
    let granted = AbilityDef {
        is_characteristic_defining: false,
        id: new_ability_id(),
        ability_type: AbilityType::Mana,
        costs: vec![Cost::Tap],
        effect: Effect::Atom(
            Primitive::ProduceMana(ManaOutput {
                mana: vec![(ManaType::Green, AmountExpr::Fixed(1))],
                special: vec![],
            }),
            EffectRecipient::Implicit,
        ),
    };

    CardDataBuilder::new("Citanul Hierophants")
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .power_toughness(3, 2)
        .rules_text("Creatures you control have \"{T}: Add {G}.\"")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::GrantAbility(
                    Box::new(granted),
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
