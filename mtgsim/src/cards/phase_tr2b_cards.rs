//! Phase TR-2b — "may", CR 118.12's answer, and the `departed` frames
//! (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-09-26 and is
//! quoted verbatim; each card's rulings were read the same day and sit in its
//! own doc comment (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Nykthos Paragon | "you may" behind CR 603.2h's gate, which only a yes closes; "that many" counters on each creature you control | no |
//! | Psychosis Crawler | a hand's count in the layer walk; "whenever you draw a card"; each opponent's life loss | yes |
//! | Cosi's Trickster | "whenever an opponent shuffles their library"; a "may" on this creature | yes |

use std::sync::Arc;

use crate::cards::authoring::{draws_a_card, shuffles_their_library, triggered_ability, whenever, Whose};
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, PlayerGroup, PlayerRef, PlayerSet,
    Primitive, Selector,
};
use crate::types::ids::AbilityId;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerLimit};

/// "You may put [amount] +1/+1 counters on [recipient]."
fn you_may_put_counters(amount: AmountExpr, recipient: EffectRecipient) -> Effect {
    Effect::Optional {
        chooser: PlayerRef::You,
        effect: Box::new(Effect::Atom(
            Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount, by: PlayerRef::You },
            recipient,
        )),
    }
}

/// Nykthos Paragon — {4}{W}{W}
/// Enchantment Creature — Human Soldier 4/6
///
/// > Whenever you gain life, you may put that many +1/+1 counters on each
/// > creature you control. Do this only once each turn.
///
/// "Do this" is the counters, so only a yes closes CR 603.2h's gate (§6.4):
/// the walk answers `Does` for a yes and the resolution writes the gate on
/// that answer alone.
///
/// # The rulings, and where each is tested
///
/// All in `phase_tr2b_integration_test`.
/// - *"...As long as you haven't yet chosen to put +1/+1 counters on your
///   creatures with it, all instances of gaining life will cause Nykthos
///   Paragon's ability to trigger."* (#1) and *"Once you have chosen to put
///   +1/+1 counters on your creatures, further instances of gaining life will
///   not cause the ability to trigger."* (#3)
///   → `paragon_triggers_until_you_choose_and_then_does_not`.
/// - *"...if you control multiple Nykthos Paragons, you will be able to do
///   this once for each of them."* (#2) → `two_paragons_each_put_their_counters_once`.
/// - *"If multiple instances of the ability are on the stack, you will be able
///   to put +1/+1 counters for only one of those instances..."* (#4) and the
///   two lifelink creatures of #6
///   → `two_lifelink_creatures_trigger_paragon_twice_and_only_the_first_acts`.
/// - *"If a creature you control is dealt lethal damage at the same time that
///   you gain life, it won't receive +1/+1 counters..."* (#5)
///   → `a_creature_dealt_lethal_damage_as_you_gain_life_gets_no_counters`.
/// - *"...if a single creature you control with lifelink deals combat damage
///   to multiple creatures, players, and/or planeswalkers at the same time
///   ..., the ability will trigger only once."* (#6)
///   → `one_lifelink_creature_dealing_damage_twice_at_once_is_one_gain`.
pub fn nykthos_paragon() -> Arc<CardData> {
    let each_creature_you_control = EffectRecipient::FilteredPermanents(ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    ));
    CardDataBuilder::new("Nykthos Paragon")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 4))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Soldier))
        .power_toughness(4, 6)
        .rules_text(
            "Whenever you gain life, you may put that many +1/+1 counters on each creature you control. \
             Do this only once each turn.",
        )
        .ability(triggered_ability(TriggerDef {
            condition: TriggerCondition::Event(TriggerEvent::GainsLife {
                player: Some(PlayerRef::You),
                multiplicity: Multiplicity::PerOccurrence,
            }),
            intervening_if: None,
            limit: Some(TriggerLimit::DoThisOnlyOnceEachTurn),
            effect: you_may_put_counters(AmountExpr::TriggeringAmount, each_creature_you_control),
        }))
        .build()
}

/// Psychosis Crawler — {5}
/// Artifact Creature — Phyrexian Horror */*
///
/// > Psychosis Crawler's power and toughness are each equal to the number of
/// > cards in your hand.
/// > Whenever you draw a card, each opponent loses 1 life.
///
/// The CDA counts a hand whose size is public (CR 402.3). Every card that
/// enters or leaves a hand is a zone move, and every zone move bumps the layer
/// epoch, so the count is never served stale.
///
/// # The rulings, and where each is tested
///
/// - *"If an effect causes you to draw multiple cards, Psychosis Crawler will
///   trigger that many times."* (#1)
///   → `phase_tr2b_integration_test::psychosis_crawler_cast_from_hand_drains_once_per_card_drawn`.
pub fn psychosis_crawler() -> Arc<CardData> {
    let cards_in_your_hand = || AmountExpr::CountOf(Selector::CardsInHand(PlayerRef::You));
    CardDataBuilder::new("Psychosis Crawler")
        .mana_cost(ManaCost::build(&[], 5))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Phyrexian))
        .subtype(Subtype::Creature(CreatureType::Horror))
        .power_toughness(0, 0)
        .rules_text(
            "Psychosis Crawler's power and toughness are each equal to the number of cards in your hand.\n\
             Whenever you draw a card, each opponent loses 1 life.",
        )
        .ability(AbilityDef {
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::SetPowerToughness(cards_in_your_hand(), cards_in_your_hand(), Duration::WhileSourceOnBattlefield),
                EffectRecipient::ThisObject,
            ),
            is_characteristic_defining: true,
            activation_restriction: ActivationRestriction::None,
        })
        .ability(triggered_ability(whenever(
            draws_a_card(Whose::Yours),
            Effect::Atom(
                Primitive::LoseLife(AmountExpr::Fixed(1)),
                EffectRecipient::EachOf(PlayerGroup::set(PlayerSet::Opponents)),
            ),
        )))
        .build()
}

/// Cosi's Trickster — {U}
/// Creature — Merfolk Wizard 1/1
///
/// > Whenever an opponent shuffles their library, you may put a +1/+1 counter
/// > on this creature.
///
/// # The rulings, and where each is tested
///
/// - *"...triggers when an opponent shuffles their library because that player
///   was instructed to do so by a spell or ability that specifically contains
///   the word 'shuffle'..."* (#1) and *"If an opponent's library is empty or
///   has just a single card in it ..., Cosi's Trickster's ability will still
///   trigger."* (#3)
///   → `phase_tr2b_integration_test::cosis_trickster_sees_an_opponents_shuffle_even_of_an_empty_library`.
/// - *"The cascade ability doesn't cause a player to shuffle their library..."*
///   (#2) → a `no-registered-card` disposition in the ledger: nothing
///   registered cascades, and no primitive puts cards on the bottom in a
///   random order.
pub fn cosis_trickster() -> Arc<CardData> {
    CardDataBuilder::new("Cosi's Trickster")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Merfolk))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .power_toughness(1, 1)
        .rules_text("Whenever an opponent shuffles their library, you may put a +1/+1 counter on this creature.")
        .ability(triggered_ability(whenever(
            shuffles_their_library(Whose::AnOpponents),
            you_may_put_counters(AmountExpr::Fixed(1), EffectRecipient::ThisObject),
        )))
        .build()
}
