//! Phase TR-5c — blocks, targeting and activation (`triggers-architecture.md`
//! §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-10-18 and is
//! quoted verbatim; each card's rulings were read the same day and sit in its
//! own doc comment (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Loyal Sentry | a block read once for each attacker it blocks, and one instruction on two objects | yes |
//! | Cephalid Aristocrat | becoming the target of a spell or ability, once for each, whoever controls it | yes |

use std::sync::Arc;

use crate::cards::authoring::{becomes_the_target_of, blocks_a_creature, triggered_ability, whenever};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{EventRole, TargetedBy, TriggerSubject};

/// Loyal Sentry — {W}
/// Creature — Human Soldier 1/1
///
/// > When this creature blocks a creature, destroy that creature and this
/// > creature.
///
/// One trigger for each creature it blocks (CR 509.3b), and one instruction
/// on both (CR 608.2f): the two are destroyed in one event.
///
/// # The ruling, and where it is tested
///
/// In `phase_tr5c_integration_test`.
/// - *"Loyal Sentry and the creature it blocks are destroyed before combat
///   damage is dealt. The blocked creature is destroyed even if Loyal Sentry
///   leaves the battlefield before its triggered ability resolves."* (#1) →
///   `loyal_sentry_destroys_both_before_combat_damage` and
///   `loyal_sentry_destroys_the_attacker_after_it_has_left`.
pub fn loyal_sentry() -> Arc<CardData> {
    let text = "When this creature blocks a creature, destroy that creature and this creature.";
    let both = EffectRecipient::Group(vec![EffectRecipient::Triggering(EventRole::Attacker), EffectRecipient::ThisObject]);
    CardDataBuilder::new("Loyal Sentry")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Soldier))
        .power_toughness(1, 1)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(blocks_a_creature(TriggerSubject::ThisObject, TriggerSubject::Any), Effect::Atom(Primitive::Destroy, both)),
        ))
        .build()
}

/// Cephalid Aristocrat — {4}{U}
/// Creature — Octopus Noble 3/3
///
/// > Whenever this creature becomes the target of a spell or ability, mill
/// > two cards.
///
/// Any spell or ability, its controller's own included, and once for each,
/// however many of its instances chose it (CR 115.9a counts instances only
/// for "with N targets").
///
/// # The rulings
///
/// None (Scryfall, 2026-10-18). Its tests are in `phase_tr5c_integration_test`.
pub fn cephalid_aristocrat() -> Arc<CardData> {
    let text = "Whenever this creature becomes the target of a spell or ability, mill two cards.";
    CardDataBuilder::new("Cephalid Aristocrat")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 4))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Octopus))
        .subtype(Subtype::Creature(CreatureType::Noble))
        .power_toughness(3, 3)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(
                becomes_the_target_of(TriggerSubject::ThisObject, TargetedBy::SpellOrAbility, None),
                Effect::Atom(Primitive::Mill(AmountExpr::Fixed(2)), EffectRecipient::Controller),
            ),
        ))
        .build()
}
