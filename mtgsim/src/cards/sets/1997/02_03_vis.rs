//! Visions (`vis`), released 1997-02-03.

use std::sync::Arc;

use crate::cards::authoring::{add_phases, creature, divided_among_targeted, each, prevent_the_next_damage_divided, spell_ability, untap, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::PhaseType;
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{Effect, ExtraPhasesAfter, ObjectFilter, SelectionFilter, TargetCount};
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

/// Remedy — {1}{W}
/// Instant
///
/// > Prevent the next 5 damage that would be dealt this turn to any number
/// > of targets, divided as you choose.
///
/// (Oracle text verified on Scryfall, 2026-11-15.)
///
/// A prevention divided among targets as it is cast: each target a shield
/// of its own share (CR 615.7).
///
/// # The rulings, and where each is tested
///
/// - *"You divide the prevention effects as you cast Remedy, not as it
///   resolves. Each target must be assigned at least 1 prevention "shield."
///   ..."* (#1) →
///   `phase_divided_among_targets_integration_test::a_divided_prevention_gives_each_target_its_own_shield`.
/// - *"If some of the targets become illegal, the original division of
///   prevention effects still applies, but the effects that would have been
///   created for illegal targets aren't created at all."* (#2) →
///   `phase_divided_among_targets_integration_test::remedy_keeps_its_division_when_a_target_becomes_illegal`.
/// - *"If multiple sources would deal damage to an affected target at once,
///   that target or that target's controller chooses which source's damage
///   to prevent for each "shield.""* (#3) →
///   `phase_rd2_integration_test::remedy_over_two_attackers_lets_the_shielded_player_choose`.
/// - *"If multiple replacement effects apply to a player or permanent being
///   dealt damage, that player or the controller of that permanent chooses
///   the order to apply them, ..."* (#4) →
///   `phase_divided_among_targets_integration_test::two_remedies_on_one_creature_are_ordered_by_its_controller`.
pub fn remedy() -> Arc<CardData> {
    let text = "Prevent the next 5 damage that would be dealt this turn to any number of targets, divided as you choose.";
    CardDataBuilder::new("Remedy")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, prevent_the_next_damage_divided(divided_among_targeted(SelectionFilter::Any, TargetCount::AnyNumber, 5.into()))))
        .build()
}
