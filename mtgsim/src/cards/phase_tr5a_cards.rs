//! Phase TR-5a — the binding's roles (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-10-16 and is
//! quoted verbatim; each card's rulings were read the same day and sit in its
//! own doc comment (`engineering-practices.md#rulings`).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Sphere Grid | a damage trigger whose "that creature" is the source, which its instruction names (CR 608.2c), and a grant to the creatures with a counter on them | yes |

use std::sync::Arc;

use crate::cards::authoring::{triggered_ability, whenever};
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{
    PlayerSet, Antecedent, AmountExpr, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive,
};
use crate::types::history::CountIs;
use crate::types::ids::AbilityId;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{ParticipantFilter, EventRole, Multiplicity, TriggerEvent, TriggerSubject};

/// Sphere Grid — {1}{G}
/// Enchantment
///
/// > Whenever a creature you control deals combat damage to a player, put a
/// > +1/+1 counter on that creature.
/// > Unlock Ability — Creatures you control with +1/+1 counters on them have
/// > reach and trample.
///
/// "That creature" is the creature the condition names, the damage's source
/// (`EventRole::DamageSource`). "Unlock Ability" is a flavor word (CR 207.2d).
///
/// # The ruling, and where it is tested
///
/// In `phase_tr5a_integration_test`.
/// - *"Once a creature that has reach because of Sphere Grid's last ability
///   has blocked a creature, removing all +1/+1 counters from that creature
///   or causing Sphere Grid to leave the battlefield won't cause that creature
///   to stop blocking."* (#1) →
///   `a_sphere_grid_blocker_keeps_blocking_once_its_counters_go` and
///   `a_sphere_grid_blocker_keeps_blocking_once_the_grid_leaves`.
pub fn sphere_grid() -> Arc<CardData> {
    let trigger_text =
        "Whenever a creature you control deals combat damage to a player, put a +1/+1 counter on that creature.";
    let unlock_text = "Unlock Ability — Creatures you control with +1/+1 counters on them have reach and trample.";
    let creature_you_control = ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerSet::You)),
    );
    let with_a_counter = ObjectFilter::And(
        Box::new(creature_you_control.clone()),
        Box::new(ObjectFilter::WithCounters { kind: Some(CounterType::PlusOnePlusOne), count: CountIs::AtLeast(1) }),
    );
    let deals_combat_damage_to_a_player = TriggerEvent::DamageDealt {
        source: TriggerSubject::Filter(creature_you_control),
        recipient: ParticipantFilter::Player(PlayerSet::Everyone),
        combat: Some(true),
        excess: false,
        multiplicity: Multiplicity::PerOccurrence,
    };
    let counter_on_that_creature = Effect::Atom(
        Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You },
        EffectRecipient::Object(Antecedent::Triggering(EventRole::DamageSource)),
    );
    let grant = |keyword: KeywordFlag| {
        Effect::Atom(
            Primitive::GrantKeywordFlag(keyword, Duration::WhileSourceOnBattlefield),
            EffectRecipient::filtered_permanents(with_a_counter.clone()),
        )
    };
    CardDataBuilder::new("Sphere Grid")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(&format!("{trigger_text}\n{unlock_text}"))
        .ability(triggered_ability(trigger_text, whenever(deals_combat_damage_to_a_player, counter_on_that_creature)))
        .ability(AbilityDef {
            rules_text: unlock_text.into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![grant(KeywordFlag::Reach), grant(KeywordFlag::Trample)]),
        })
        .build()
}
