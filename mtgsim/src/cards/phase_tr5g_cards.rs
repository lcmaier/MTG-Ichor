//! Phase TR-5g — the multiplier, tokens and scry (`triggers-architecture.md`
//! §12).
//!
//! The oracle text below was verified on Scryfall on 2026-10-20 and is quoted
//! verbatim; the card's rulings were read the same day and sit in its doc
//! comment (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Panharmonicon | CR 603.2d's count, over every entering trigger of its controller's permanents | yes |

use std::sync::Arc;

use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::effects::{ObjectFilter, PlayerRef};
use crate::types::mana::ManaCost;
use crate::types::triggers::{Multiplicity, TriggerCondition, TriggerEvent, TriggerMultiplierDef, TriggerSubject};

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
/// - *Entering beside Panharmonicon, Panharmonicon included, is doubled*
///   (#7) → `a_creature_entering_beside_panharmonicon_is_doubled`.
/// - *A linked ability's additional instances are linked too* (#8, #9): not
///   yet expressible; CR 607's linked abilities are `backlog.md` §2.2's.
pub fn panharmonicon() -> Arc<CardData> {
    let text = "If an artifact or creature entering causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.";
    let artifact_or_creature = ObjectFilter::Or(
        Box::new(ObjectFilter::ByType(CardType::Artifact)),
        Box::new(ObjectFilter::ByType(CardType::Creature)),
    );
    let multiplier = TriggerMultiplierDef {
        caused_by: Some(TriggerCondition::Event(TriggerEvent::EntersBattlefield {
            subject: TriggerSubject::Filter(artifact_or_creature),
            controller: None,
            from: None,
            was_cast: None,
            multiplicity: Multiplicity::PerOccurrence,
        })),
        abilities_of: ObjectFilter::ByController(PlayerRef::You),
        additional: 1,
    };
    CardDataBuilder::new("Panharmonicon")
        .mana_cost(ManaCost::build(&[], 4))
        .card_type(CardType::Artifact)
        .rules_text(text)
        .ability(multiplier.into_ability(text))
        .build()
}
