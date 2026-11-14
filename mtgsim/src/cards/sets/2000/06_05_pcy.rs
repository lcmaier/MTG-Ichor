//! Prophecy (`pcy`), released 2000-06-05.

use std::sync::Arc;

use crate::cards::authoring::{
    any_player, draw_cards, gets, spell_ability, target_creature, that_player, the_target, triggered_ability, whenever, you,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{Effect, PlayerRef};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::TriggerEvent;

/// Rhystic Study — {2}{U}
/// Enchantment
///
/// > Whenever an opponent casts a spell, you may draw a card unless that
/// > player pays {1}.
///
/// CR 118.12a's sentence around a "you may": "that player may pay {1}. If
/// they don't, you may draw a card." Both are asked as the trigger resolves,
/// the payment first (CR 608.2c), of the player who cast the spell, who may
/// activate mana abilities to pay (CR 605.3a).
///
/// It exercises a payment asked of a trigger's player, ATOM-605.3a-002's
/// board.
///
/// # The rulings, and where each is tested
///
/// In `phase_pay_as_an_effect_resolves_integration_test`.
/// - *"Rhystic Study's triggered ability resolves before the spell that
///   caused it to trigger. It resolves even if that spell is countered. The
///   player gets the option to pay when this triggered ability resolves."*
///   (#1) → `rhystic_study_resolves_after_its_spell_is_countered`.
/// - *"You don't have to decide whether or not to draw a card until after
///   the player decides whether or not to pay."* (#2) →
///   `rhystic_study_draws_when_that_player_declines`.
pub fn rhystic_study() -> Arc<CardData> {
    let text = "Whenever an opponent casts a spell, you may draw a card unless that player pays {1}.";
    let an_opponent_casts = TriggerEvent::CastsSpell { caster: Some(PlayerRef::Opponent), spell: None };
    let may_draw = Effect::Optional { chooser: PlayerRef::You, effect: Box::new(draw_cards(you(), 1)) };
    CardDataBuilder::new("Rhystic Study")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(an_opponent_casts, may_draw.unless(that_player(), vec![Cost::Mana(ManaCost::build(&[], 1))])),
        ))
        .build()
}

/// Wild Might — {1}{G}
/// Instant
///
/// > Target creature gets +1/+1 until end of turn. That creature gets an
/// > additional +4/+4 until end of turn unless any player pays {2}.
///
/// CR 118.12a's "unless" asked of any player: each in APNAP order (CR 101.4),
/// at most once, until one pays; if none does, the +4/+4.
///
/// It exercises `PlayerRef::AnyPlayer`, the payment's one chooser that names
/// more than one player.
///
/// # The ruling, and where it is tested
///
/// In `phase_pay_as_an_effect_resolves_integration_test`.
/// - *"Each player gets the option to pay when the spell resolves."* (#1) →
///   `any_player_asks_each_player_at_most_once_in_apnap_order`.
pub fn wild_might() -> Arc<CardData> {
    let text = "Target creature gets +1/+1 until end of turn. That creature gets an additional +4/+4 until end of turn unless any player pays {2}.";
    let more = gets(the_target(), 4, 4).until_end_of_turn();
    CardDataBuilder::new("Wild Might")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                gets(target_creature(), 1, 1).until_end_of_turn(),
                more.unless(any_player(), vec![Cost::Mana(ManaCost::build(&[], 2))]),
            ]),
        ))
        .build()
}
