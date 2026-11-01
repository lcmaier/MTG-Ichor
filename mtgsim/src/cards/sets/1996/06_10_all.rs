//! Alliances (`all`), released 1996-06-10.

use std::sync::Arc;

use crate::cards::authoring::{any_target, deal_damage, spell_ability, triggered_ability, whenever};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{Effect, PlayerRef};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::restriction::SourceFilter;
use crate::types::triggers::{Multiplicity, TriggerEvent, TriggerSubject};
use crate::types::zones::{Zone, ZoneChangeAct, ZoneChangePattern};

/// "[It] deals `n` damage to any target."
fn deals_damage_to_any_target(n: u64) -> Effect {
    deal_damage(any_target(), n)
}

/// Guerrilla Tactics — {1}{R}
/// Instant
///
/// > Guerrilla Tactics deals 2 damage to any target.
/// > When a spell or ability an opponent controls causes you to discard this
/// > card, it deals 4 damage to any target.
///
/// The discard is not one of CR 603.10a's look-back classes, so the trigger
/// is asked after the event, where the card went (CR 603.10): the graveyard,
/// or exile under Rest in Peace. CR 113.6k makes it work in every zone it can
/// trigger from, and CR 702.29c says what those are for a trigger on a
/// discard: "whatever zone the card winds up in". So Yixlid Jailer stops it,
/// as its second ruling says of Krosan Tusker's cycling trigger.
///
/// It exercises a trigger on this card leaving a hidden zone (its discard),
/// asked where the card went (CR 603.10, 113.6k, 702.29c).
///
/// # The rulings, and where each is tested
///
/// In `phase_tr4c_integration_test`.
/// - *"If it is discarded, the damage done by it is done by this card due to
///   an ability and not by a spell."* (#1) →
///   `guerrilla_tactics_discarded_by_an_opponents_spell_deals_four_damage`.
/// - *"The second ability will trigger even on an optional discard caused by
///   an opponent."* (#2) →
///   `guerrilla_tactics_triggers_on_an_optional_discard_an_opponent_causes`.
/// - *"Discarding as a cost to cast a spell will not trigger the ability.
///   Only discarding as an effect will trigger the ability."* (#3) →
///   `guerrilla_tactics_discarded_as_a_cost_does_not_trigger`.
pub fn guerrilla_tactics() -> Arc<CardData> {
    let spell = "Guerrilla Tactics deals 2 damage to any target.";
    let trigger = "When a spell or ability an opponent controls causes you to discard this card, it deals 4 damage to any target.";
    let discarded = TriggerEvent::ZoneChange {
        subject: TriggerSubject::ThisObject,
        change: ZoneChangePattern { from: Some(Zone::Hand), to: None, act: Some(ZoneChangeAct::Discarded) },
        by: Some(SourceFilter::ControlledBy(PlayerRef::Opponent)),
        owner: None,
        multiplicity: Multiplicity::PerOccurrence,
    };
    CardDataBuilder::new("Guerrilla Tactics")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .rules_text("Guerrilla Tactics deals 2 damage to any target.\nWhen a spell or ability an opponent controls causes you to discard this card, it deals 4 damage to any target.")
        .ability(spell_ability(
            spell,
            deals_damage_to_any_target(2),
        ))
        .ability(triggered_ability(trigger, whenever(discarded, deals_damage_to_any_target(4))))
        .build()
}
