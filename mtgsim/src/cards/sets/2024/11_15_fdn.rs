//! Foundations (`fdn`), released 2024-11-15.

use std::sync::Arc;

use crate::cards::authoring::{create_tokens, deal_damage, spell_ability, target_creature, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Effect, TokenDef};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};

/// Goblin Negotiation — {X}{R}{R}
/// Sorcery
///
/// > Goblin Negotiation deals X damage to target creature. Create a number of
/// > 1/1 red Goblin creature tokens equal to the amount of excess damage dealt
/// > to that creature this way.
///
/// The tokens read the damage record the resolution performed, which the
/// window holds until the resolution ends. It deals damage to one creature
/// only, so the excess dealt this way is the excess dealt to it.
///
/// It exercises the excess damage a resolution's own effect dealt, read later
/// in the same resolution.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr5d_integration_test`.
/// - *Excess damage is damage past lethal, the damage already marked on the
///   creature taken into account* (#1) →
///   `goblin_negotiation_counts_the_damage_already_marked`.
/// - *If Goblin Negotiation has deathtouch, 1 damage is lethal, so the rest is
///   excess* (#2) → `goblin_negotiation_with_deathtouch_makes_all_but_one_excess`.
pub fn goblin_negotiation() -> Arc<CardData> {
    let text = concat!(
        "Goblin Negotiation deals X damage to target creature. Create a number of 1/1 red Goblin creature tokens ",
        "equal to the amount of excess damage dealt to that creature this way.",
    );
    CardDataBuilder::new("Goblin Negotiation")
        .mana_cost(ManaCost::from_symbols(vec![
            ManaSymbol::X,
            ManaSymbol::Colored(ManaType::Red),
            ManaSymbol::Colored(ManaType::Red),
        ]))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                deal_damage(target_creature(), AmountExpr::X),
                create_tokens(you(), AmountExpr::ExcessDamageDealtThisWay, TokenDef::creature(1, 1, &[Color::Red], &[CreatureType::Goblin])),
            ]),
        ))
        .build()
}
