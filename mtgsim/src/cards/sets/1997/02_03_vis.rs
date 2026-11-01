//! Visions (`vis`), released 1997-02-03.

use std::sync::Arc;

use crate::cards::authoring::{add_phases, creature, each, spell_ability, untap, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::PhaseType;
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{Effect, ExtraPhasesAfter, ObjectFilter};
use crate::types::history::ObjectFact;
use crate::types::mana::{ManaCost, ManaType};

/// Relentless Assault — {2}{R}{R}
/// Sorcery
///
/// > Untap all creatures that attacked this turn. After this main phase,
/// > there is an additional combat phase followed by an additional main
/// > phase.
///
/// "Creatures that attacked this turn" are those declared as attackers this
/// turn (CR 508.1), so a creature put onto the battlefield attacking is not
/// one (CR 508.4). The extra phases are Aggravated Assault's, anchored to
/// this main phase.
///
/// It exercises "creatures that attacked this turn", the combat history's
/// `Attacked` rows, and a second combat (CR 500.8).
///
/// # The rulings, and where each is tested
///
/// In `phase_tr5f_integration_test`.
/// - *Only if it resolves during a main phase* (#1) →
///   `relentless_assault_adds_no_phase_outside_a_main_phase`.
/// - *A creature that attacks each turn if able must attack only once* (#2):
///   attack requirements are `cant-effects-architecture.md`'s `combat-restrictions`, a
///   disposition.
/// - *Cast on an opponent's turn, that opponent's creatures untap and that
///   opponent attacks again* (#3) →
///   `relentless_assault_on_an_opponents_turn_is_that_players_combat`.
/// - *Attack triggers trigger during each combat* (#4) →
///   `relentless_assault_untaps_what_attacked_and_attack_triggers_fire_again`.
pub fn relentless_assault() -> Arc<CardData> {
    let text = "Untap all creatures that attacked this turn. After this main phase, there is an additional combat phase followed by an additional main phase.";
    let attacked_this_turn =
        creature().and(ObjectFilter::ThisTurn(ObjectFact::Attacked));
    CardDataBuilder::new("Relentless Assault")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                untap(each(attacked_this_turn)),
                add_phases(you(), vec![PhaseType::Combat, PhaseType::Postcombat], ExtraPhasesAfter::ThisMainPhase),
            ]),
        ))
        .build()
}
