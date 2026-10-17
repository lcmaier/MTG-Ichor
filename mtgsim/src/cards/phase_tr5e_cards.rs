//! Phase TR-5e — the attack declaration (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-10-16 and is
//! quoted verbatim; each card's rulings were read the same day and sit in its
//! own doc comment (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Hellrider | an attack trigger for each creature its controller attacks with, and damage to what that creature is attacking (CR 506.3, 508.3a) | yes |

use std::sync::Arc;

use crate::cards::authoring::{attacks, triggered_ability, whenever};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Antecedent, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::EventRole;

/// Hellrider — {2}{R}{R}
/// Creature — Devil 3/3
///
/// > Haste
/// > Whenever a creature you control attacks, this creature deals 1 damage
/// > to the player or planeswalker it's attacking.
///
/// "It" is the creature that attacked, the trigger's attacker, and what it is
/// attacking is read as the ability resolves: nothing once it has left combat
/// and stayed on the battlefield, and what it was attacking once it has left
/// (CR 608.2h; Fathom Fleet Swordjack's ruling on the same words).
///
/// # The ruling, and where it is tested
///
/// In `phase_tr5e_integration_test`.
/// - *"Creatures you control may attack multiple players and/or
///   planeswalkers. For each attacking creature, Hellrider will deal damage to
///   the corresponding player or planeswalker."* (#1) →
///   `hellrider_deals_each_attackers_damage_to_what_it_attacks`, at four
///   seats. Nothing attacks a planeswalker until `backlog.md` §2.23's attack
///   path, so the test's creatures attack players.
pub fn hellrider() -> Arc<CardData> {
    let text = "Whenever a creature you control attacks, this creature deals 1 damage to the player or planeswalker it's attacking.";
    let creature_you_control = ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    );
    let one_damage = Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::Fixed(1), unpreventable: false },
        EffectRecipient::AttackedBy(Antecedent::InEvent(EventRole::Attacker)),
    );
    CardDataBuilder::new("Hellrider")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Devil))
        .power_toughness(3, 3)
        .keyword_flag(KeywordFlag::Haste)
        .rules_text(&format!("Haste\n{text}"))
        .ability(triggered_ability(text, whenever(attacks(creature_you_control), one_damage)))
        .build()
}
