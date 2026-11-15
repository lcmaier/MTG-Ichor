//! Innistrad: Crimson Vow (`vow`), released 2021-11-19.

use std::sync::Arc;

use crate::cards::authoring::{choose_number, chosen_by, creature, each_player, sacrifice, spell_ability, the_chosen_number, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{Choice, ChoiceScope, ChoiceSide, Effect, Pick};
use crate::types::mana::{ManaCost, ManaType};

/// By Invitation Only — {3}{W}{W}
/// Sorcery
///
/// > Choose a number between 0 and 13. Each player sacrifices that many
/// > creatures of their choice.
///
/// (Oracle text verified on Scryfall, 2026-11-15.)
///
/// A number chosen as the spell resolves (CR 608.2d), read as an edict's
/// count: each player picks that many of their creatures, or as many as
/// they have.
///
/// # The rulings, and where each is tested
///
/// All three → `phase_value_chosen_as_an_effect_resolves_integration_test::by_invitation_only_has_each_player_sacrifice_the_number_chosen`:
/// - *"The numbers you may choose include 0 and 13."* (#1)
/// - *"If a player controls fewer than the chosen number of creatures, they
///   will sacrifice all their creatures."* (#2)
/// - *"Starting with the player whose turn it is, each player in turn order
///   chooses which creatures they will sacrifice, then all the creatures
///   chosen by all players are sacrificed at the same time. ..."* (#3)
pub fn by_invitation_only() -> Arc<CardData> {
    let text = "Choose a number between 0 and 13. Each player sacrifices that many creatures of their choice.";
    let each_sacrifices_that_many = sacrifice(chosen_by(Choice {
        chooser: each_player(),
        among: ChoiceScope::ChoosersPermanents,
        picks: vec![Pick::exactly(the_chosen_number(), creature())],
        acts_on: ChoiceSide::Chosen,
    }));
    CardDataBuilder::new("By Invitation Only")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(text, Effect::Sequence(vec![choose_number(you(), 0, 13), each_sacrifices_that_many])))
        .build()
}
