//! Wilds of Eldraine (`woe`), released 2023-09-08.

use std::sync::Arc;

use crate::cards::authoring::{choose_number, chosen, creature, destroy, each, power_at_least, spell_ability, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{ChoiceName, Effect};
use crate::types::mana::{ManaCost, ManaType};

/// Expel the Interlopers — {3}{W}{W}
/// Sorcery
///
/// > Choose a number between 0 and 10. Destroy all creatures with power
/// > greater than or equal to the chosen number.
///
/// (Oracle text verified on Scryfall, 2026-11-15.)
///
/// A value chosen as the spell resolves (CR 608.2d), kept, and read: two
/// steps, the number and the destruction. The second step's filter compares
/// each creature's power with the number kept, read once as the step runs
/// (CR 608.2h), so a creature whose power changes afterwards is not
/// reconsidered. A creature with power below 0 survives even a 0.
///
/// # The rulings, and where each is tested
///
/// - *"The numbers you may choose include 0 and 10."* (#1)
///   → `phase_value_chosen_as_an_effect_resolves_integration_test::expel_the_interlopers_offers_0_through_10_and_destroys_from_the_number_up`.
pub fn expel_the_interlopers() -> Arc<CardData> {
    let text = "Choose a number between 0 and 10. Destroy all creatures with power greater than or equal to the chosen number.";
    CardDataBuilder::new("Expel the Interlopers")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                choose_number(you(), 0, 10),
                destroy(each(creature().and(power_at_least(chosen(ChoiceName::Number))))),
            ]),
        ))
        .build()
}
