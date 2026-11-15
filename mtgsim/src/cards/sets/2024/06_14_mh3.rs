//! Modern Horizons 3 (`mh3`), released 2024-06-14.

use std::sync::Arc;

use crate::cards::authoring::{activated_ability, deal_damage_divided, divided_among_targeted};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{SelectionFilter, TargetCount};
use crate::types::mana::{ManaCost, ManaType};

/// Mogg Mob — {R}{R}{R}
/// Creature — Goblin 3/3
///
/// > Sacrifice this creature: It deals 3 damage divided as you choose among
/// > one, two, or three targets.
///
/// (Oracle text verified on Scryfall, 2026-11-15.)
///
/// A division on an activation (CR 602.2b), announced ahead of the cost, so
/// the shares are fixed before the sacrifice, and the source deals them as
/// it last existed (CR 113.7a).
///
/// # The rulings, and where each is tested
///
/// - *"You choose how many targets Mogg Mob's ability has and how the damage
///   is divided as you activate the ability. Each target must receive at
///   least 1 damage."* (#1) →
///   `phase_divided_among_targets_integration_test::an_activation_divides_ahead_of_its_cost`.
/// - *"If some of the targets are illegal as Mogg Mob's activated ability
///   tries to resolve, the original division of damage still applies and the
///   damage that would have been dealt to the illegal targets is lost. ..."*
///   (#2) → `phase_divided_among_targets_integration_test::mogg_mobs_share_for_an_illegal_target_is_lost`.
pub fn mogg_mob() -> Arc<CardData> {
    let text = "Sacrifice this creature: It deals 3 damage divided as you choose among one, two, or three targets.";
    CardDataBuilder::new("Mogg Mob")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red, ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Goblin))
        .power_toughness(3, 3)
        .rules_text(text)
        .ability(activated_ability(
            text,
            vec![Cost::SacrificeSelf],
            deal_damage_divided(divided_among_targeted(SelectionFilter::Any, TargetCount::Between(1, 3), 3.into())),
        ))
        .build()
}
