//! Urza's Destiny (`uds`), released 1999-06-07.

use std::sync::Arc;

use crate::cards::authoring::{
    activated_ability, becomes_types, color, creature, draw_cards, each, non, other_than_this, sacrifice, skip, static_ability, static_replacement,
    this_object, token, triggered_ability, when_state, whenever, with_subtype, you, you_now,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::{CardType, CreatureType, EnchantmentType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{AmountExpr, Condition, Effect, ObjectFilter, PlayerFact, PlayerRef, PlayerSet, TypeChange};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::EventPattern;
use crate::types::triggers::{Multiplicity, TriggerEvent, TriggerSubject};
use crate::types::zones::{Zone, ZoneChangePattern};

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
/// It exercises "from anywhere", which reads the card where it went and not as
/// it left.
///
/// # The ruling, and where it is tested
///
/// In `phase_tr4a_integration_test`.
/// - *"The ability triggers on a black card being discarded, milled, or
///   countered, as well as a black card going to the graveyard from the
///   battlefield."* (#1) → `compost_sees_a_black_card_reach_a_graveyard_from_every_zone`.
pub fn compost() -> Arc<CardData> {
    let text = "Whenever a black card is put into an opponent's graveyard from anywhere, you may draw a card.";
    let black_card = color(Color::Black).and(non(token()));
    let put_into_an_opponents_graveyard = TriggerEvent::ZoneChange {
        subject: TriggerSubject::Filter(black_card),
        change: ZoneChangePattern { from: None, to: Some(Zone::Graveyard), act: None },
        by: None,
        owner: Some(PlayerRef::Opponent),
        multiplicity: Multiplicity::PerOccurrence,
    };
    let may_draw = Effect::Optional {
        chooser: PlayerRef::You,
        effect: Box::new(draw_cards(you(), 1)),
    };
    CardDataBuilder::new("Compost")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(text, whenever(put_into_an_opponents_graveyard, may_draw)))
        .build()
}

/// Emperor Crocodile — {3}{G}
/// Creature — Crocodile, 5/5
///
/// > When you control no other creatures, sacrifice this creature.
///
/// A state trigger: the board is asked, not an event, at each of the state
/// check's three moments. "No other creatures" is `Condition::Not` over the
/// positive fact, and "other" is relative to the Crocodile itself.
///
/// It exercises CR 603.8's state check, after a resolution's own batches and
/// before placement.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr6_integration_test`.
/// - *It triggers if you don't control another creature even for a brief
///   moment during a resolution* (#1) →
///   `emperor_crocodile_triggers_for_a_moment_in_the_middle_of_a_resolution`.
/// - *It checks only as it triggers, so a creature gained before it resolves
///   doesn't save it* (#2) → `emperor_crocodile_is_sacrificed_whatever_arrives_before_it_resolves`.
pub fn emperor_crocodile() -> Arc<CardData> {
    let other_creature = creature().and(other_than_this());
    CardDataBuilder::new("Emperor Crocodile")
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Crocodile))
        .power_toughness(5, 5)
        .rules_text("When you control no other creatures, sacrifice this creature.")
        .ability(triggered_ability(
            "When you control no other creatures, sacrifice this creature.",
            when_state(
                Condition::Not(Box::new(you_now(PlayerFact::ControlsPermanent(other_creature)))),
                sacrifice(this_object()),
            ),
        ))
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
/// "Each other" is `ObjectFilter::NotSource`, the leaf this card is the first
/// consumer of; its 2004-10-04 ruling is the leaf's test: "Does not animate
/// itself. But can be animated by another Opalescence."
///
/// Registered, not pooled: it opens no engine path Urborg does not (its
/// order against Humility is the timestamp's, which LI-1 already gave), and
/// animating every enchantment in a random game would move every
/// behavioral row of §3's table for a reason that is not the engine's.
pub fn opalescence() -> Arc<CardData> {
    let other_non_aura_enchantments = each(
        ObjectFilter::ByType(CardType::Enchantment)
            .and(non(with_subtype(Subtype::Enchantment(EnchantmentType::Aura))))
            .and(other_than_this()),
    );

    CardDataBuilder::new("Opalescence")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text(
            "Each other non-Aura enchantment is a creature in addition to its other types and \
             has base power and base toughness each equal to its mana value.",
        )
        .ability(static_ability(
            "Each other non-Aura enchantment is a creature in addition to its other types and has base power and base toughness each equal to its mana value.",
            becomes_types(other_non_aura_enchantments, TypeChange::adding(&[], &[CardType::Creature], &[]))
                .and_has_base_power_and_toughness(AmountExpr::AffectedManaValue, AmountExpr::AffectedManaValue),
        ))
        .build()
}

/// Yawgmoth's Bargain — {4}{B}{B}
/// Enchantment
///
/// > Skip your draw step.
/// > Pay 1 life: Draw a card.
///
/// The plainest static skip there is, and the one that shows what a skipped
/// step costs: not the draw alone but the whole step — no `StepBegin`, no
/// turn-based action, no priority round (`ATOM-614.10-001`). Its second
/// ability is here because a card that only subtracted would never be cast by
/// anything, and both halves — `Cost::PayLife` and `Primitive::DrawCards` —
/// already exist.
///
/// Scryfall lists no rulings (2026-09-11).
///
/// **Registered and not pooled.** A random agent with one use for its life
/// total will empty its library, which is the board RE-6's Laboratory Maniac
/// path wants and a distortion of every fixture until it lands.
pub fn yawgmoths_bargain() -> Arc<CardData> {
    CardDataBuilder::new("Yawgmoth's Bargain")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black], 4))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("Skip your draw step.\nPay 1 life: Draw a card.")
        .ability(static_replacement("Skip your draw step.", skip(
            EventPattern::BeginStep { step: Some(StepType::Draw) },
            PlayerSet::You,
        )))
        .ability(activated_ability("Pay 1 life: Draw a card.",
            vec![Cost::PayLife(1)],
            draw_cards(you(), 1),
        ))
        .build()
}
