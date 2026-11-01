//! Phase TR-6 — state triggers (`triggers-architecture.md` §4.5, §12).
//!
//! The oracle text below was verified on Scryfall on 2026-10-21 and is quoted
//! verbatim; the card's rulings were read the same day and sit in its doc
//! comment (`engineering-practices.md#rulings`).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Emperor Crocodile | CR 603.8's state check, after a resolution's own batches and before placement | yes |
//!
//! The fixtures below are registered nowhere. Each is a board one of the
//! phase's rules or rulings needs that no registered card makes.

use std::sync::Arc;

use crate::cards::authoring::{create_tokens, destroy, discard, draw_cards, each, lose_the_game, sacrifice, spell_ability, this_object, you};
use crate::cards::authoring::triggered_ability;
use crate::cards::phase_re_cards::soldier_token;
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{
    ObjectSet, AmountExpr, Condition, DiscardChooser, Effect, ObjectFilter, PlayerFact, PlayerRef, PlayerSet, };
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{TriggerCondition, TriggerDef};

/// "When [state], [effect]": a state trigger (CR 603.8). It has no
/// intervening "if", since it checks its state as it triggers and never again.
fn when_state(condition: Condition, effect: Effect) -> TriggerDef {
    TriggerDef { condition: TriggerCondition::State(condition), intervening_if: None, limit: None, effect }
}

/// "You [fact]": the fact of the ability's controller, now.
fn you_now(fact: PlayerFact) -> Condition {
    Condition::Player { players: PlayerSet::You, fact }
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
/// # The rulings, and where each is tested
///
/// In `phase_tr6_integration_test`.
/// - *It triggers if you don't control another creature even for a brief
///   moment during a resolution* (#1) →
///   `emperor_crocodile_triggers_for_a_moment_in_the_middle_of_a_resolution`.
/// - *It checks only as it triggers, so a creature gained before it resolves
///   doesn't save it* (#2) → `emperor_crocodile_is_sacrificed_whatever_arrives_before_it_resolves`.
pub fn emperor_crocodile() -> Arc<CardData> {
    let other_creature = ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::NotSource));
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

/// Immortal Coil's third ability, alone: "When there are no cards in your
/// graveyard, you lose the game." A fixture, not the card. The Coil's first
/// ability costs "exile two cards from your graveyard", and
/// `Cost::ExileFromGraveyard` is neither checked nor paid
/// (`engine::costs`). A card under its real name may not carry a dead
/// ability (`engineering-practices.md#cards-and-pools`).
pub fn immortal_coils_state_trigger() -> Arc<CardData> {
    CardDataBuilder::new("Fixture: Immortal Coil's state trigger")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Artifact)
        .rules_text("When there are no cards in your graveyard, you lose the game.")
        .ability(triggered_ability(
            "When there are no cards in your graveyard, you lose the game.",
            when_state(
                Condition::Not(Box::new(you_now(PlayerFact::CardInGraveyard(ObjectFilter::All)))),
                lose_the_game(you()),
            ),
        ))
        .build()
}

/// CR 603.8's example permanent: "Whenever you have no cards in hand, draw
/// a card."
pub fn draws_on_an_empty_hand() -> Arc<CardData> {
    CardDataBuilder::new("Fixture: whenever you have no cards in hand, draw a card")
        .card_type(CardType::Enchantment)
        .rules_text("Whenever you have no cards in hand, draw a card.")
        .ability(triggered_ability(
            "Whenever you have no cards in hand, draw a card.",
            when_state(you_now(PlayerFact::HandEmpty), draw_cards(you(), 1)),
        ))
        .build()
}

/// CR 603.8's example spell: "Discard your hand, then draw that many cards."
/// Its hand is empty for a moment between the two instructions.
pub fn discard_your_hand_then_draw_that_many() -> Arc<CardData> {
    spell(
        "Fixture: discard your hand, then draw that many cards",
        "Discard your hand, then draw that many cards.",
        Effect::Sequence(vec![
            discard(you(), AmountExpr::CountOf(ObjectSet::hand(PlayerRef::You)), DiscardChooser::Affected),
            draw_cards(you(), AmountExpr::DiscardedThisWay),
        ]),
    )
}

/// Emperor Crocodile's first ruling's board: a resolution after which its
/// controller has another creature, though not between its two instructions.
/// The ruling names Brago, King Eternal's flicker; a creature destroyed and a
/// token made is the same moment.
pub fn destroy_your_others_then_a_token() -> Arc<CardData> {
    let your_non_crocodiles = ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::And(
            Box::new(ObjectFilter::ByController(PlayerRef::You)),
            Box::new(ObjectFilter::Not(Box::new(ObjectFilter::BySubtype(Subtype::Creature(CreatureType::Crocodile))))),
        )),
    );
    spell(
        "Fixture: destroy your other creatures, then make a Soldier",
        "Destroy each non-Crocodile creature you control, then create a 1/1 white Soldier creature token.",
        Effect::Sequence(vec![
            destroy(each(your_non_crocodiles)),
            create_tokens(you(), 1, soldier_token()),
        ]),
    )
}

/// A sorcery with `effect` and nothing to pay or target.
fn spell(name: &str, text: &'static str, effect: Effect) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            effect,
        ))
        .build()
}
