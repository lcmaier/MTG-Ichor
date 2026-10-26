//! Phase TR-4d — control, countering, loss, and the records' missing facts
//! (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-10-15 and is
//! quoted verbatim; each card's rulings were read the same day and sit in its
//! own doc comment (`engineering-practices.md#rulings`).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Multani's Presence | a trigger on a spell being countered, looking back at who cast it (CR 603.10e) | no |
//! | Ray of Command | a delayed trigger on losing control, which a steal ending at cleanup makes, in a second cleanup step (CR 603.10d, 514.3a) | yes |

use std::sync::Arc;

use crate::cards::authoring::{spell_is_countered, triggered_ability, whenever, you_lose_control_of, Whose};
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{
    Antecedent, AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, SelectionFilter, TargetCount,
};
use crate::types::ids::AbilityId;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{DelayedDuration, DelayedTriggerTemplate, DelayedTurn, TriggerSubject};

/// Multani's Presence — {G}
/// Enchantment
///
/// > Whenever a spell you've cast is countered, draw a card.
///
/// "A spell you've cast" is who cast it, which an effect that took control of
/// the spell does not change (`StackEntry::caster`).
///
/// # The ruling, and where it is tested
///
/// In `phase_tr4d_integration_test`.
/// - *"A spell that doesn't resolve because its targets are illegal isn't
///   countered, so this ability won't trigger. This is a change from previous
///   rules."* (#1) → `a_spell_whose_targets_are_illegal_is_not_countered`.
pub fn multanis_presence() -> Arc<CardData> {
    let text = "Whenever a spell you've cast is countered, draw a card.";
    CardDataBuilder::new("Multani's Presence")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(
                spell_is_countered(Whose::Yours),
                Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You)),
            ),
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
    let creature_an_opponent_controls = ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
    );
    let tap_it = DelayedTriggerTemplate {
        def: Arc::new(whenever(
            you_lose_control_of(TriggerSubject::Object(Antecedent::Referred)),
            Effect::Atom(Primitive::Tap, EffectRecipient::Object(Antecedent::Referred)),
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
        .ability(AbilityDef {
            rules_text: text.into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            // One instance of "target" (CR 115.3): "it" and "that creature"
            // are back-references to it.
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::Untap,
                    EffectRecipient::Target(SelectionFilter::Permanent(creature_an_opponent_controls), TargetCount::Exactly(1)),
                ),
                Effect::Remember(Box::new(Effect::Atom(
                    Primitive::GainControl { to: PlayerRef::You, duration: Duration::UntilEndOfTurn },
                    EffectRecipient::SameInstanceAs(0),
                ))),
                Effect::Atom(
                    Primitive::GrantKeywordFlag(KeywordFlag::Haste, Duration::UntilEndOfTurn),
                    EffectRecipient::SameInstanceAs(0),
                ),
                Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(tap_it)), EffectRecipient::Player(PlayerRef::You)),
            ]),
        })
        .build()
}
