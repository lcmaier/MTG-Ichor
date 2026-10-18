//! Phase MA-2 — the legal values of X (`mana-architecture.md` §3.7, §10).
//!
//! The oracle text below was verified on Scryfall on 2026-10-17 and is
//! quoted verbatim, and the card's rulings were read the same day: it has
//! none (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Blaze | the plain X spell: the gate at the least X, CR 601.2b's legal values, 601.2f's expansion, and the announced X read at resolution | yes |

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount};
use crate::types::ids::AbilityId;
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};

/// Blaze — {X}{R}
/// Sorcery
/// "Blaze deals X damage to any target."
///
/// No rulings (Scryfall, 2026-10-17).
pub fn blaze() -> Arc<CardData> {
    CardDataBuilder::new("Blaze")
        .card_type(CardType::Sorcery)
        .color(Color::Red)
        .mana_cost(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Red)]))
        .rules_text("Blaze deals X damage to any target.")
        .ability(AbilityDef {
            rules_text: "Blaze deals X damage to any target.".into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::DealDamage { amount: AmountExpr::X, unpreventable: false },
                EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
            ),
        })
        .build()
}
