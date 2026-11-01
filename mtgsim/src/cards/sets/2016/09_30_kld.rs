//! Kaladesh (`kld`), released 2016-09-30.

use std::sync::Arc;

use crate::cards::authoring::{artifact, creature, draw_cards, get_counters, lose_life, spell_ability, you, you_cast, you_control};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::cost_modification::{CostChange, CostModificationDef};
use crate::types::effects::{CounterType, Effect};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{Multiplicity, TriggerCondition, TriggerEvent, TriggerMultiplierDef, TriggerSubject};

/// Foundry Inspector — {3}
/// Artifact Creature — Construct, 3/2
///
/// Artifact spells you cast cost {1} less to cast.
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// The reducer for the Ironworks board: it makes the artifact being cast
/// cheaper *before* the window opens, so the total the window's sacrifices
/// are paid against is one the board no longer justifies by the time they are
/// made. That is the lock-in, seen from the payment side.
pub fn foundry_inspector() -> Arc<CardData> {
    CardDataBuilder::new("Foundry Inspector")
        .mana_cost(ManaCost::build(&[], 3))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Construct))
        .power_toughness(3, 2)
        .rules_text("Artifact spells you cast cost {1} less to cast.")
        .ability(
            CostModificationDef::spells(
                you_cast(artifact()),
                CostChange::Reduce(ManaCost::build(&[], 1)),
            )
            .into_ability("Artifact spells you cast cost {1} less to cast."),
        )
        .build()
}

/// Live Fast — {2}{B}
/// Sorcery
///
/// > You draw two cards, lose 2 life, and get {E}{E} (two energy counters).
///
/// **The producer of a player's counters, with nothing else in it**: RE-2's
/// draw instruction, RA's life loss, and `Primitive::GetCounters` — the one
/// new primitive, resolved to its controller.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"Energy counters are a kind of counter that a player may have. They're
///   not associated with any specific permanents."* → `PlayerState::counters`
///   has the kind and no permanent does — the engine's own test,
///   `a_player_gets_counters_through_the_same_event`; the card as printed
///   is card breadth's to test, when fixtures move to set folders.
/// - *"Any effects that interact with counters a player gets, has, or loses
///   can interact with energy counters."* → Vorinclex's and Winding
///   Constrictor's player halves, on this card.
/// - *"To pay one or more {E}, you lose that many energy counters. You can't
///   pay more energy counters than you have."* → a cost, which waits for its
///   first card (`backlog.md` §2.16's close).
///
/// **Registered and not pooled**: a three-mana cantrip whose counters nothing
/// pooled reads. Its job in RE-5 was to be Vorinclex's and Winding
/// Constrictor's producer.
pub fn live_fast() -> Arc<CardData> {
    CardDataBuilder::new("Live Fast")
        .mana_cost(ManaCost::build(&[ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Sorcery)
        .rules_text("You draw two cards, lose 2 life, and get {E}{E}.")
        .ability(spell_ability(
            "You draw two cards, lose 2 life, and get {E}{E}.",
            Effect::Sequence(vec![
                draw_cards(you(), 2),
                lose_life(you(), 2),
                get_counters(you(), CounterType::Energy, 2),
            ]),
        ))
        .build()
}

/// Panharmonicon — {4}
/// Artifact
///
/// > If an artifact or creature entering causes a triggered ability of a
/// > permanent you control to trigger, that ability triggers an additional
/// > time.
///
/// The event is asked of each record that triggered an ability, with
/// Panharmonicon's own referents, and the ability is any of a permanent its
/// controller controls, whatever its own trigger event says.
///
/// It exercises CR 603.2d's count, over every entering trigger of its
/// controller's permanents.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr5g_integration_test`.
/// - *A permanent's own entering trigger is doubled, and so is another's
///   trigger on it entering* (#1) → `panharmonicon_doubles_a_permanents_own_entering_trigger`
///   and `panharmonicon_doubles_soul_warden_on_a_creature_entering`.
/// - *Replacement effects are unaffected* (#2) →
///   `panharmonicon_adds_no_counter_a_creature_enters_with`.
/// - *"As [this] enters" is unaffected* (#3): an "as enters" choice is a
///   replacement in this engine (CR 614.12), so #2's test is its board.
/// - *You need not control the entering permanent, only the triggered
///   ability's* (#4) → `panharmonicon_doubles_on_an_opponents_creature_entering`, and
///   an opponent's Panharmonicon doubles nothing of yours:
///   `an_opponents_panharmonicon_doubles_nothing_of_yours`.
/// - *Each instance is its own trigger, with its own choices* (#5) →
///   `each_doubled_instance_chooses_its_own_target`.
/// - *Two make three, not four* (#6) → `two_panharmonicons_make_three`.
/// - *A trigger on entering beside Panharmonicon, Panharmonicon included,
///   triggers an additional time* (#7) →
///   `a_trigger_on_entering_beside_panharmonicon_triggers_twice`.
/// - *A linked ability's additional instances are linked too* (#8, #9): not
///   yet expressible; CR 607's linked abilities are `backlog.md` §2.2's.
pub fn panharmonicon() -> Arc<CardData> {
    let text = "If an artifact or creature entering causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.";
    let artifact_or_creature = artifact().or(creature());
    let multiplier = TriggerMultiplierDef {
        caused_by: Some(TriggerCondition::Event(TriggerEvent::EntersBattlefield {
            subject: TriggerSubject::Filter(artifact_or_creature),
            controller: None,
            from: None,
            was_cast: None,
            multiplicity: Multiplicity::PerOccurrence,
        })),
        abilities_of: you_control(),
        additional: 1,
    };
    CardDataBuilder::new("Panharmonicon")
        .mana_cost(ManaCost::build(&[], 4))
        .card_type(CardType::Artifact)
        .rules_text(text)
        .ability(multiplier.into_ability(text))
        .build()
}
