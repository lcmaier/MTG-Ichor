//! Fixtures for triggered abilities (CR 603, `triggers-architecture.md`): the
//! history records and state triggers, and the ones the fixtures pool casts.

use std::sync::Arc;

use crate::cards::authoring::{
    becomes_a_blocked_creature, create_tokens, creature, destroy, discard, draw_cards, each, each_in, gain_life, is_also_color,
    leaves_the_battlefield, lose_the_game, non, spell_ability, static_ability, target_creature, token, triggered_ability, when_state, whenever,
    with_subtype, you, you_control, you_have,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Condition, DiscardChooser, Effect, ObjectFilter, ObjectSet, PlayerFact, PlayerRef, PlayerSet, TokenDef};
use crate::types::history::{HistoryCount, HistorySpan, ObjectFact, TurnFact};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{EventRole, TriggerCondition, TriggerDef, TriggerEvent, TriggerLimit, TriggerSubject};
use crate::types::zones::ZoneSet;

/// Every castable fixture, by name, as `CardRegistry::fixture_pool` registers
/// them.
pub const CASTABLE: [(&str, fn() -> Arc<CardData>); 22] = [
    ("Fixture Curtain", fixture_curtain),
    ("Fixture Token Tally", fixture_token_tally),
    ("Fixture Mourning Ink", fixture_mourning_ink),
    ("Fixture Mourner's Tithe", fixture_mourners_tithe),
    ("Fixture First Creature's Tithe", fixture_first_creature_tithe),
    ("Mogg Mob Fixture", super::mogg_mob_fixture),
    ("Dragonlord Atarka Fixture", super::dragonlord_atarka_fixture),
    ("Magma Opus Fixture", super::magma_opus_fixture),
    ("Elven Rite Fixture", super::elven_rite_fixture),
    ("Remedy Fixture", super::remedy_fixture),
    ("Rolling Thunder Fixture", super::rolling_thunder_fixture),
    ("Meteor Swarm Fixture", super::meteor_swarm_fixture),
    ("Omnath, Locus of Mana Fixture", super::omnath_locus_of_mana_fixture),
    ("Momentous Fall Fixture", super::momentous_fall_fixture),
    ("Tithing Engine", super::tithing_engine),
    ("Reckoner of Debts", super::reckoner_of_debts),
    ("Grudge Clerk", super::grudge_clerk),
    ("Forked Reliquary", super::forked_reliquary),
    ("Sentry of the Tapped Gate", super::sentry_of_the_tapped_gate),
    ("Kudo, King Among Bears Fixture", super::kudo_king_among_bears_fixture),
    ("Godhead of Awe Fixture", super::godhead_of_awe_fixture),
    ("Scattering Flicker", super::scattering_flicker),
];

/// Fixture Curtain — {W}, instant: "Target creature becomes blocked." CR
/// 509.1h's status, which only an unblocked attacking creature takes.
pub fn fixture_curtain() -> Arc<CardData> {
    let text = "Target creature becomes blocked.";
    CardDataBuilder::new("Fixture Curtain")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, becomes_a_blocked_creature(target_creature())))
        .build()
}

/// Fixture Token Tally — {1}, instant: "You gain 1 life for each token
/// creature you controlled that died this turn." The tokens are known only
/// by their departures' rows once CR 704.5d has removed them.
pub fn fixture_token_tally() -> Arc<CardData> {
    let text = "You gain 1 life for each token creature you controlled that died this turn.";
    let died = HistoryCount { players: PlayerSet::You, fact: TurnFact::ControlledCreaturesDied, of: Some(Box::new(token())) };
    CardDataBuilder::new("Fixture Token Tally")
        .mana_cost(ManaCost::build(&[], 1))
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, gain_life(you(), AmountExpr::History(HistorySpan::ThisTurn, Box::new(died)))))
        .build()
}

/// Fixture Mourning Ink — {1}{B}, enchantment: "Cards in graveyards that died
/// this turn are black." Each card its death made (CR 700.4, 400.7).
pub fn fixture_mourning_ink() -> Arc<CardData> {
    let text = "Cards in graveyards that died this turn are black.";
    CardDataBuilder::new("Fixture Mourning Ink")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(static_ability(text, is_also_color(each_in(ObjectFilter::ThisTurn(ObjectFact::Died), ZoneSet::GRAVEYARD), Color::Black)))
        .build()
}

/// Fixture Mourner's Tithe — {2}, enchantment: "Whenever a creature leaves
/// the battlefield, you gain life equal to its power." Its power as it last
/// existed (CR 608.2h), a creature leaving the game with its owner included
/// (CR 603.6c).
pub fn fixture_mourners_tithe() -> Arc<CardData> {
    let text = "Whenever a creature leaves the battlefield, you gain life equal to its power.";
    let creature = TriggerSubject::Filter(creature());
    let gain = gain_life(you(), AmountExpr::TriggeringPower(EventRole::Object));
    CardDataBuilder::new("Fixture Mourner's Tithe")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(text, whenever(leaves_the_battlefield(creature), gain)))
        .build()
}

/// Fixture First Creature's Tithe — {1}{G}, enchantment: "Whenever you cast
/// your first creature spell each turn, you gain 1 life." The first of the
/// kind it names, over the whole turn (Kalamax, the Stormsire's ruling).
pub fn fixture_first_creature_tithe() -> Arc<CardData> {
    let text = "Whenever you cast your first creature spell each turn, you gain 1 life.";
    let first_creature = TriggerDef {
        condition: TriggerCondition::Event(TriggerEvent::CastsSpell {
            caster: Some(PlayerRef::You),
            spell: Some(creature()),
        }),
        intervening_if: None,
        limit: Some(TriggerLimit::FirstTimeEachTurn),
        effect: gain_life(you(), 1),
    };
    CardDataBuilder::new("Fixture First Creature's Tithe")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(text, first_creature))
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
                Condition::Not(Box::new(you_have(PlayerFact::CardInGraveyard(ObjectFilter::All)))),
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
            when_state(you_have(PlayerFact::HandEmpty), draw_cards(you(), 1)),
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
    let your_non_crocodiles = creature().and(you_control().and(non(with_subtype(Subtype::Creature(CreatureType::Crocodile)))));
    spell(
        "Fixture: destroy your other creatures, then make a Soldier",
        "Destroy each non-Crocodile creature you control, then create a 1/1 white Soldier creature token.",
        Effect::Sequence(vec![
            destroy(each(your_non_crocodiles)),
            create_tokens(you(), 1, TokenDef::creature(1, 1, &[Color::White], &[CreatureType::Soldier])),
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
