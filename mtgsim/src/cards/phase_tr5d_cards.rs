//! Phase TR-5d — prevention and excess damage (`triggers-architecture.md`
//! §12).
//!
//! The oracle text below was verified on Scryfall on 2026-10-19 and
//! 2026-10-20 and is quoted verbatim; each card's rulings were read the same
//! day and sit in its doc comment (`engineering-practices.md#rulings`).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Selfless Squire | a prevention shield with one subject, and the arm over each application of a prevention effect | yes |
//! | Goblin Negotiation | the excess damage a resolution's own effect dealt, read later in the same resolution | yes |

use std::sync::Arc;

use crate::cards::authoring::{damage_to_is_prevented, enters, triggered_ability, whenever};
use crate::cards::phase_re_cards::goblin_token;
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{
    Antecedent, AmountExpr, CounterType, Duration, Effect, EffectRecipient, PatternFill, PlayerRef, PlayerSet, Primitive,
    SelectionFilter, TargetCount,
};
use crate::types::ids::AbilityId;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::types::replacement::{EventPattern, ReplacementDef, Rewrite};
use crate::types::triggers::{ParticipantFilter, TriggerSubject};

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
/// Angel of Suffering's `PlayerSet::You` and no object, created for
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
    let prevent_all_to_you = ReplacementDef::new(EventPattern::DealDamage { source: None, combat: None }, PlayerSet::You, Rewrite::Prevent);
    let shield = Effect::Atom(
        Primitive::CreateReplacement(Box::new(prevent_all_to_you), Duration::UntilEndOfTurn, PatternFill::Authored),
        EffectRecipient::Implicit,
    );
    let growth = Effect::Atom(
        Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::TriggeringAmount, by: PlayerRef::You },
        EffectRecipient::Object(Antecedent::ThisObject),
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
        .ability(triggered_ability(shield_text, whenever(enters(TriggerSubject::Object(Antecedent::ThisObject)), shield)))
        .ability(triggered_ability(
            growth_text,
            whenever(damage_to_is_prevented(ParticipantFilter::Player(PlayerSet::You)), growth),
        ))
        .build()
}

/// Goblin Negotiation — {X}{R}{R}
/// Sorcery
///
/// > Goblin Negotiation deals X damage to target creature. Create a number of
/// > 1/1 red Goblin creature tokens equal to the amount of excess damage dealt
/// > to that creature this way.
///
/// The tokens read the damage record the resolution performed, which the
/// window holds until the resolution ends. It deals damage to one creature
/// only, so the excess dealt this way is the excess dealt to it.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr5d_integration_test`.
/// - *Excess damage is damage past lethal, the damage already marked on the
///   creature taken into account* (#1) →
///   `goblin_negotiation_counts_the_damage_already_marked`.
/// - *If Goblin Negotiation has deathtouch, 1 damage is lethal, so the rest is
///   excess* (#2) → `goblin_negotiation_with_deathtouch_makes_all_but_one_excess`.
pub fn goblin_negotiation() -> Arc<CardData> {
    let text = concat!(
        "Goblin Negotiation deals X damage to target creature. Create a number of 1/1 red Goblin creature tokens ",
        "equal to the amount of excess damage dealt to that creature this way.",
    );
    CardDataBuilder::new("Goblin Negotiation")
        .mana_cost(ManaCost::from_symbols(vec![
            ManaSymbol::X,
            ManaSymbol::Colored(ManaType::Red),
            ManaSymbol::Colored(ManaType::Red),
        ]))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(AbilityDef {
            rules_text: text.into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::DealDamage { amount: AmountExpr::X, unpreventable: false },
                    EffectRecipient::Target(SelectionFilter::creature(), TargetCount::Exactly(1)),
                ),
                Effect::Atom(
                    Primitive::CreateToken(goblin_token(), AmountExpr::ExcessDamageDealtThisWay),
                    EffectRecipient::Player(PlayerRef::You),
                ),
            ]),
        })
        .build()
}
