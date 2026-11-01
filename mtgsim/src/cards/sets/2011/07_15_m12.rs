//! Magic 2012 (`m12`), released 2011-07-15.

use std::sync::Arc;

use crate::cards::authoring::{dies, gets, host, return_to_hand, static_ability, this_object, triggered_ability, whenever};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, EnchantmentType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{SelectionFilter, TypeChange};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::TriggerSubject;

/// Angelic Destiny — {2}{W}{W}
/// Enchantment — Aura
///
/// > Enchant creature
/// > Enchanted creature gets +4/+4, has flying and first strike, and is an
/// > Angel in addition to its other types.
/// > When enchanted creature dies, return this card to its owner's hand.
///
/// The trigger is CR 603.6e's: it looks back to the creature's departure,
/// and "this card" is the Aura's new object in its owner's graveyard. CR
/// 400.7f lets it find that object when the Aura went there in the same
/// event as the creature, or by CR 704.5m's check that follows, which is
/// the usual way: the Aura stays on the battlefield, unattached, until then.
///
/// It exercises an Aura's trigger on its creature dying finds the card the Aura
/// became in its owner's graveyard (CR 603.6e, 400.7f).
///
/// # The rulings, and where each is tested
///
/// In `phase_tr4c_integration_test`.
/// - *"If the creature dies before the Angelic Destiny spell resolves,
///   Angelic Destiny will go to its owner's graveyard. It won't return to its
///   owner's hand."* (#1) →
///   `angelic_destiny_whose_creature_died_before_it_resolved_stays_in_the_graveyard`.
/// - *"If Angelic Destiny is no longer in a graveyard when its triggered
///   ability resolves, it won't be returned to its owner's hand."* (#2) →
///   `angelic_destiny_gone_from_the_graveyard_is_not_returned`.
pub fn angelic_destiny() -> Arc<CardData> {
    let anthem = "Enchanted creature gets +4/+4, has flying and first strike, and is an Angel in addition to its other types.";
    let returns = "When enchanted creature dies, return this card to its owner's hand.";
    let angel = TypeChange::adding(&[], &[Subtype::Creature(CreatureType::Angel)], &[]);
    CardDataBuilder::new("Angelic Destiny")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .enchant_filter(SelectionFilter::Creature)
        .rules_text("Enchant creature\nEnchanted creature gets +4/+4, has flying and first strike, and is an Angel in addition to its other types.\nWhen enchanted creature dies, return this card to its owner's hand.")
        .ability(static_ability(
            anthem,
            gets(host(), 4, 4).and_gains_keyword(KeywordFlag::Flying).and_gains_keyword(KeywordFlag::FirstStrike).and_becomes_types(angel),
        ))
        .ability(triggered_ability(
            returns,
            whenever(dies(TriggerSubject::Host), return_to_hand(this_object())),
        ))
        .build()
}
