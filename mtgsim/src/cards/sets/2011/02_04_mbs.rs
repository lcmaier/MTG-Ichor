//! Mirrodin Besieged (`mbs`), released 2011-02-04.

use std::sync::Arc;

use crate::cards::authoring::{
    another, at_beginning_of, copy, creature, draws_a_card, each_opponent, has_base_power_and_toughness, lose_life, static_ability, target_permanent,
    this_object, triggered_ability, whenever, Whose,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, CharacteristicEdit, CopyException, CopyRoles, Effect, ObjectSet, PlayerRef};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{TriggerCondition, TriggerDef};

/// Cryptoplasm — {1}{U}{U}
/// Creature — Shapeshifter 2/2
///
/// > At the beginning of your upkeep, you may have this creature become a copy
/// > of another target creature, except it has this ability.
///
/// (Oracle text verified on Scryfall, 2026-10-06.)
///
/// # CV-1b's card: a copy that only a move ends
///
/// The copy is a resolution's row with no stated duration, so CR 611.2a's
/// "until the end of the game", `Duration::Indefinite`, and nothing but CR
/// 400.7 ends it: Cryptoplasm leaving the battlefield. Each upkeep it may
/// copy again (CR 707.4), a new row over the old one, and "except it has this
/// ability" keeps the trigger that does it.
///
/// Three facilities, each first needed here: `CopyRoles::ThisObjectCopiesRecipient`
/// for "this creature becomes a copy of [target]", "another target"
/// (`ObjectFilter::NotSource` in a targeting filter), and
/// `CharacteristicEdit::GainsThisAbility`.
///
/// # In `PERFORMANCE_POOL`
///
/// An indefinite copy row is an engine path no pooled card opened: a row that
/// outlives its turn, and that a move alone ends. At `{1}{U}{U}` a random game
/// casts it, and its "may" is asked every upkeep it has a target.
///
/// # The rulings, and where each is tested
///
/// All four are in `tests/phase_cv1b_integration_test.rs`: the target chosen
/// as the trigger is put on the stack and the "may" as it resolves (1); the
/// copy lasts until another overwrites it (2); an illegal target leaves it as
/// it was (3); a copy of it copies what it copies and has its ability (4).
pub fn cryptoplasm() -> Arc<CardData> {
    let text = "At the beginning of your upkeep, you may have this creature become a copy of another target creature, except it has this ability.";
    let becomes_a_copy = copy(
        target_permanent(another(creature())),
        CopyRoles::ThisObjectCopiesRecipient,
        vec![CopyException::Modifies(CharacteristicEdit::GainsThisAbility)],
    )
    // No duration printed: CR 611.2a's "until the end of the game".
    .until_end_of_game();
    CardDataBuilder::new("Cryptoplasm")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Shapeshifter))
        .power_toughness(2, 2)
        .rules_text(text)
        .ability(triggered_ability(text, TriggerDef {
            condition: TriggerCondition::Event(at_beginning_of(StepType::Upkeep, Whose::Yours)),
            intervening_if: None,
            limit: None,
            effect: Effect::Optional { chooser: PlayerRef::You, effect: Box::new(becomes_a_copy) },
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
/// The CDA counts the cards in a hand and never looks at them, which any
/// player may do to any hand (CR 402.3). Every card that enters or leaves a
/// hand is a zone move, and every zone move bumps the layer epoch, so the
/// count is never served stale.
///
/// It exercises a hand's count in the layer walk; "whenever you draw a card";
/// each opponent's life loss.
///
/// # The rulings, and where each is tested
///
/// - *"If an effect causes you to draw multiple cards, Psychosis Crawler will
///   trigger that many times."* (#1)
///   → `phase_tr2b_integration_test::psychosis_crawler_cast_from_hand_drains_once_per_card_drawn`.
pub fn psychosis_crawler() -> Arc<CardData> {
    let cards_in_your_hand = || AmountExpr::CountOf(ObjectSet::hand(PlayerRef::You));
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
        .ability(static_ability(
            "Psychosis Crawler's power and toughness are each equal to the number of cards in your hand.",
            has_base_power_and_toughness(this_object(), cards_in_your_hand(), cards_in_your_hand()),
        ).characteristic_defining())
        .ability(triggered_ability("Whenever you draw a card, each opponent loses 1 life.", whenever(
            draws_a_card(Whose::Yours),
            lose_life(each_opponent(), 1),
        )))
        .build()
}
