//! Phase TR-5d — prevention and excess damage (`triggers-architecture.md`
//! §12).
//!
//! The oracle text below was verified on Scryfall on 2026-10-19 and is quoted
//! verbatim; the card's rulings were read the same day and sit in its doc
//! comment (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Selfless Squire | a prevention shield with one subject, and the arm over each application of a prevention effect | yes |

use std::sync::Arc;

use crate::cards::authoring::{damage_to_is_prevented, enters, triggered_ability, whenever};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, CounterType, Duration, Effect, EffectRecipient, ObjectSet, PatternFill, PlayerRef, PlayerSet, Primitive,
};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, ReplacementDef, Rewrite};
use crate::types::triggers::{DamageRecipient, TriggerSubject};

/// Selfless Squire — {3}{W}
/// Creature — Human Soldier 1/1
///
/// > Flash
/// >
/// > When this creature enters, prevent all damage that would be dealt to you
/// > this turn.
/// >
/// > Whenever damage that would be dealt to you is prevented, put that many
/// > +1/+1 counters on this creature.
///
/// The first ability is RD-2's prevention with one subject and no object,
/// Angel of Suffering's `NO_OBJECTS` beside `PlayerSet::You`, created for
/// the turn. The second reads CR 615.13's event, one record for each time a
/// prevention effect is applied to the damage dealt to you.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr5d_integration_test`.
/// - *The prevention lasts after the Squire leaves the battlefield* (#1) →
///   `selfless_squires_prevention_lasts_after_it_leaves`.
/// - *Any prevention effect triggers the second ability, not only the first
///   ability's* (#2) → `selfless_squire_grows_from_any_prevention_of_damage_to_you`.
pub fn selfless_squire() -> Arc<CardData> {
    let shield_text = "When this creature enters, prevent all damage that would be dealt to you this turn.";
    let growth_text = "Whenever damage that would be dealt to you is prevented, put that many +1/+1 counters on this creature.";
    let prevent_all_to_you = ReplacementDef::new(EventPattern::DealDamage { source: None, combat: None }, ObjectSet::NO_OBJECTS, Rewrite::Prevent)
        .affecting_players(PlayerSet::You);
    let shield = Effect::Atom(
        Primitive::CreateReplacement(Box::new(prevent_all_to_you), Duration::UntilEndOfTurn, PatternFill::Authored),
        EffectRecipient::Implicit,
    );
    let growth = Effect::Atom(
        Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::TriggeringAmount, by: PlayerRef::You },
        EffectRecipient::ThisObject,
    );
    CardDataBuilder::new("Selfless Squire")
        .mana_cost(ManaCost::build(&[ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Soldier))
        .power_toughness(1, 1)
        .keyword_flag(KeywordFlag::Flash)
        .rules_text(&format!("Flash\n{shield_text}\n{growth_text}"))
        .ability(triggered_ability(shield_text, whenever(enters(TriggerSubject::ThisObject), shield)))
        .ability(triggered_ability(
            growth_text,
            whenever(damage_to_is_prevented(DamageRecipient::Player(Some(PlayerRef::You))), growth),
        ))
        .build()
}
