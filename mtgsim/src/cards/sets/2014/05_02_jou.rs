//! Journey into Nyx (`jou`), released 2014-05-02.

use std::sync::Arc;

use crate::cards::authoring::{
    an_opponent_controls, enters, exile_until, land, leaves_the_battlefield, non, target_permanent, triggered_ability, whenever,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::ReturnUnder;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::TriggerSubject;

/// Banishing Light — {2}{W}
/// Enchantment
///
/// > When this enchantment enters, exile target nonland permanent an
/// > opponent controls until this enchantment leaves the battlefield.
///
/// CR 610.3's "until": the exile, and the return it waits to make
/// (`GameState::until_returns`), which is performed as Banishing Light's
/// departure is dispatched, without the stack, under the card's owner's
/// control (610.3c). The exile's object is the card in exile by identity, so
/// a token that ceased to exist there returns nothing.
///
/// It exercises CR 610.3's "until" exile, and its return made at a dispatch
/// without the stack.
///
/// # The rulings, and where each is tested
///
/// All in `phase_tr3b_integration_test`.
/// - *"If an Aura is exiled this way, its owner chooses what it will enchant
///   as it returns to the battlefield. An Aura put onto the battlefield this
///   way doesn't target anything (so it could be attached to a permanent an
///   opponent controls with hexproof, for example), but the Aura's enchant
///   ability restricts what it can be attached to. If the Aura can't legally
///   be attached to anything, it remains in exile for the rest of the
///   game."* (#1) → `banishing_lights_aura_returns_enchanting_what_its_owner_chooses`
///   and `banishing_lights_aura_with_nothing_to_enchant_stays_in_exile`.
/// - *"Auras attached to the exiled permanent will be put into their owners'
///   graveyards. Any Equipment will become unattached and remain on the
///   battlefield. Any counters on the exiled permanent will cease to exist.
///   When the card returns to the battlefield, it will be a new object with
///   no connection to the card that was exiled."* (#2) →
///   `banishing_lights_card_returns_as_a_new_object`.
/// - *"If Banishing Light leaves the battlefield before its triggered ability
///   resolves, the target permanent won't be exiled."* (#3, CR 610.3b) →
///   `banishing_light_gone_before_its_trigger_resolves_exiles_nothing`.
/// - *"If a token is exiled this way, it will cease to exist and won't
///   return to the battlefield."* (#4) → `a_token_banishing_light_exiles_does_not_return`.
pub fn banishing_light() -> Arc<CardData> {
    let text = "When this enchantment enters, exile target nonland permanent an opponent controls until this enchantment leaves the battlefield.";
    let nonland_an_opponent_controls = non(land()).and(an_opponent_controls());
    CardDataBuilder::new("Banishing Light")
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(
                enters(TriggerSubject::ThisObject),
                exile_until(target_permanent(nonland_an_opponent_controls), leaves_the_battlefield(TriggerSubject::ThisObject), ReturnUnder::Owner),
            ),
        ))
        .build()
}
