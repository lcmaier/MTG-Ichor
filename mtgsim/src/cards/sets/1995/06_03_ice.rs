//! Ice Age (`ice`), released 1995-06-03.

use std::sync::Arc;

use crate::cards::authoring::{
    an_opponent_controls, at_beginning_of, create_delayed_trigger, creature, deal_damage, draw_cards, each, gain_control_of, gain_life, gains_keyword,
    referred, spell_ability, tap, target_permanent, the_target, untap, whenever, you, you_lose_control_of, Whose,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::Effect;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{DelayedDuration, DelayedTriggerTemplate, DelayedTurn, TriggerSubject};

/// Blessed Wine — {1}{W}
/// Instant
///
/// > You gain 1 life.
/// > Draw a card at the beginning of the next turn's upkeep.
///
/// Two spell abilities, one per paragraph, followed in the order written (CR
/// 113.2c, 608.2c): the gain, then a delayed trigger that is the spell's and
/// its controller's as it resolved (CR 603.7d), so "draw" is that player's in
/// whoever's upkeep comes next. "The next turn's" binds it to a later turn
/// than this one (`DelayedTurn::NextTurn`), so a second upkeep this turn (CR
/// 500.10) is not it.
///
/// **Registered, not pooled**: §12 pools the registry with TR-3b's two cards,
/// so TR-3a's close-out read the performance pool unmoved.
///
/// It exercises a spell's delayed trigger in the next turn, which ends no game.
///
/// # The rulings, and where each is tested
///
/// Scryfall lists no rulings (2026-10-07). The tests are
/// `phase_tr3a_integration_test`'s `final_fortune_and_blessed_wine`: cast from
/// hand, and the second upkeep it waits past.
pub fn blessed_wine() -> Arc<CardData> {
    let draw_next_upkeep = DelayedTriggerTemplate {
        def: Arc::new(whenever(
            at_beginning_of(StepType::Upkeep, Whose::Each),
            draw_cards(you(), 1),
        )),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::NextTurn,
        rules_text: "Draw a card at the beginning of the next turn's upkeep.".into(),
    };
    CardDataBuilder::new("Blessed Wine")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text("You gain 1 life.\nDraw a card at the beginning of the next turn's upkeep.")
        .ability(spell_ability(
            "You gain 1 life.",
            gain_life(you(), 1),
        ))
        .ability(spell_ability(
            "Draw a card at the beginning of the next turn's upkeep.",
            create_delayed_trigger(you(), draw_next_upkeep),
        ))
        .build()
}

/// Pyroclasm — {1}{R}
///
/// > Pyroclasm deals 2 damage to each creature.
///
/// CR 615.10's example needs the other half of it, and this is the pool's
/// first "each creature" damage: one source, every creature, **one event**.
/// `EffectRecipient::FilteredPermanents` is what names them, and the batch is
/// what makes CR 615.7's allocation and CR 704.3's simultaneity reachable at
/// all — a loop of one-member batches would be a different game.
///
/// Scryfall lists no rulings (2026-09-08).
pub fn pyroclasm() -> Arc<CardData> {
    CardDataBuilder::new("Pyroclasm")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text("Pyroclasm deals 2 damage to each creature.")
        .ability(spell_ability("Pyroclasm deals 2 damage to each creature.",
            deal_damage(each(creature()), 2),
        ))
        .build()
}

/// Ray of Command — {3}{U}
/// Instant
///
/// > Untap target creature an opponent controls and gain control of it until
/// > end of turn. That creature gains haste until end of turn. When you lose
/// > control of the creature, tap it.
///
/// The steal's creature is what the instruction acted on (`Effect::Remember`,
/// CR 603.7c), and the delayed trigger's "you" is the spell's controller (CR
/// 603.7d). Losing control is whatever ends it: the steal wearing off, a
/// third player taking the creature, or its leaving the battlefield.
///
/// It exercises a delayed trigger on losing control, which a steal ending at
/// cleanup makes, in a second cleanup step (CR 603.10d, 514.3a).
///
/// # The rulings, and where each is tested
///
/// In `phase_tr4d_integration_test`.
/// - *"You tap the creature when you lose control of it for any reason —
///   because Ray of Command's effect ends, or because a spell or ability
///   causes another player to gain control of it."* (#1) →
///   `ray_of_command_taps_the_creature_another_player_takes`.
/// - *"You may target an untapped creature with Ray of Command."* (#2) →
///   `ray_of_command_may_target_an_untapped_creature`.
/// - *"The creature returns to the opponent when the 'until end of turn'
///   effect wears off during the cleanup step. It taps during the Cleanup
///   step (if it is not already tapped), so any abilities triggered off it
///   tapping happen at that time."* (#3) →
///   `ray_of_command_taps_the_creature_in_the_cleanup_step`.
pub fn ray_of_command() -> Arc<CardData> {
    let text = "Untap target creature an opponent controls and gain control of it until end of turn. That creature gains haste until end of turn. When you lose control of the creature, tap it.";
    let creature_an_opponent_controls = creature().and(an_opponent_controls());
    let tap_it = DelayedTriggerTemplate {
        def: Arc::new(whenever(
            you_lose_control_of(TriggerSubject::Referred),
            tap(referred()),
        )),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::NoneNamed,
        rules_text: "When you lose control of the creature, tap it.".into(),
    };
    CardDataBuilder::new("Ray of Command")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 3))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(
            text,
            // One instance of "target" (CR 115.3): "it" and "that creature"
            // are back-references to it.
            Effect::Sequence(vec![
                untap(target_permanent(creature_an_opponent_controls)),
                Effect::Remember(Box::new(gain_control_of(the_target()).until_end_of_turn())),
                gains_keyword(the_target(), KeywordFlag::Haste).until_end_of_turn(),
                create_delayed_trigger(you(), tap_it),
            ]),
        ))
        .build()
}
