//! Phase TR-4a — the frame's facts and their readers (`triggers-architecture.md`
//! §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-10-14 and is
//! quoted verbatim; each card's rulings were read the same day and sit in its
//! own doc comment (`engineering-practices.md#rulings`).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Strangleroot Geist | undying: an intervening "if" about the counters it had, read off its last known information at both instants, and a return with a counter | yes |
//! | Demonic Vigor | an Aura's "when enchanted creature dies", asked of its host's departure, which framed what was attached to it | no |
//! | Compost | "from anywhere", which reads the card where it went and not as it left | no |

use std::sync::Arc;

use crate::cards::authoring::{dies, triggered_ability, undying, whenever};
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, EnchantmentType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{
    PlayerSet, Antecedent, AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, SelectionFilter,
};
use crate::types::ids::AbilityId;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{EventRole, Multiplicity, TriggerEvent, TriggerSubject};
use crate::types::zones::{Zone, ZoneChangePattern};

/// Strangleroot Geist — {G}{G}
/// Creature — Spirit 2/1
///
/// > Haste
/// > Undying (When this creature dies, if it had no +1/+1 counters on it,
/// > return it to the battlefield under its owner's control with a +1/+1
/// > counter on it.)
///
/// Undying's "if it had" is a fact about the permanent as it left (CR
/// 603.10a), read off the counters its last known information kept, at the
/// trigger and again at resolution (CR 603.4). "It" is the card in the
/// graveyard (CR 400.7e), which returns as a new object (CR 400.7) with its
/// +1/+1 counter as part of the entry (CR 122.6).
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The first pooled ability whose intervening "if" reads the triggering
/// object's last known information, and the first return with counters: a new
/// engine path, which §3.4 measures rather than assumes. It returns once, so
/// it lengthens a stress game by a creature rather than a loop (the
/// distortion `fuzz-record.md`'s TR-4a block measured).
///
/// # The rulings
///
/// Scryfall lists none. Kitchen Finks' seven persist rulings are tested as
/// undying's mirror in `phase_tr4a_integration_test`, each on both keywords.
pub fn strangleroot_geist() -> Arc<CardData> {
    CardDataBuilder::new("Strangleroot Geist")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Spirit))
        .power_toughness(2, 1)
        .rules_text("Haste\nUndying")
        .keyword_flag(KeywordFlag::Haste)
        .ability(undying())
        .build()
}

/// Demonic Vigor — {B}
/// Enchantment — Aura
///
/// > Enchant creature
/// > Enchanted creature gets +1/+1.
/// > When enchanted creature dies, return that card to its owner's hand.
///
/// A leaves-the-battlefield ability (CR 603.10a), so it looks back: the Aura
/// stays on the battlefield, unattached, until CR 704.5m puts it into the
/// graveyard, and what it enchanted is read off the creature's departure,
/// which framed what was attached to it. "That card" is the one the creature
/// became in the graveyard (CR 603.6e, 400.7).
///
/// # The ruling, and where it is tested
///
/// In `phase_tr4a_integration_test`.
/// - *"Demonic Vigor can enchant a token, but its last ability won't return
///   the token to your hand."* (#1) →
///   `demonic_vigor_triggers_for_a_token_and_returns_nothing`.
pub fn demonic_vigor() -> Arc<CardData> {
    let text = "When enchanted creature dies, return that card to its owner's hand.";
    CardDataBuilder::new("Demonic Vigor")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .rules_text("Enchant creature\nEnchanted creature gets +1/+1.\nWhen enchanted creature dies, return that card to its owner's hand.")
        .enchant_filter(SelectionFilter::Creature)
        .ability(AbilityDef {
            rules_text: "Enchanted creature gets +1/+1.".into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ModifyPowerToughness(
                    AmountExpr::Fixed(1),
                    AmountExpr::Fixed(1),
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::Object(Antecedent::ThisObjectsHost),
            ),
        })
        .ability(triggered_ability(
            text,
            whenever(
                dies(TriggerSubject::Object(Antecedent::ThisObjectsHost)),
                Effect::Atom(Primitive::ReturnToHand, EffectRecipient::Object(Antecedent::Triggering(EventRole::Object))),
            ),
        ))
        .build()
}

/// Compost — {1}{G}
/// Enchantment
///
/// > Whenever a black card is put into an opponent's graveyard from anywhere,
/// > you may draw a card.
///
/// "From anywhere" is never a leaves-the-battlefield ability (CR 603.6c), so
/// it does not look back: "black card" is asked of the card in the graveyard
/// (CR 603.10). A Clone that copied a black creature dies blue, and a
/// creature Darkest Hour made black dies its own color. A token is not a
/// card (CR 108.2), and the graveyard is its owner's (CR 400.3).
///
/// # The ruling, and where it is tested
///
/// In `phase_tr4a_integration_test`.
/// - *"The ability triggers on a black card being discarded, milled, or
///   countered, as well as a black card going to the graveyard from the
///   battlefield."* (#1) → `compost_sees_a_black_card_reach_a_graveyard_from_every_zone`.
pub fn compost() -> Arc<CardData> {
    let text = "Whenever a black card is put into an opponent's graveyard from anywhere, you may draw a card.";
    let black_card = ObjectFilter::And(
        Box::new(ObjectFilter::ByColor(Color::Black)),
        Box::new(ObjectFilter::Not(Box::new(ObjectFilter::Token))),
    );
    let put_into_an_opponents_graveyard = TriggerEvent::ZoneChange {
        subject: TriggerSubject::Filter(black_card),
        change: ZoneChangePattern { from: None, to: Some(Zone::Graveyard), act: None },
        by: None,
        owner: PlayerSet::Opponents,
        multiplicity: Multiplicity::PerOccurrence,
    };
    let may_draw = Effect::Optional {
        chooser: PlayerRef::You,
        effect: Box::new(Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You))),
    };
    CardDataBuilder::new("Compost")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(text, whenever(put_into_an_opponents_graveyard, may_draw)))
        .build()
}
