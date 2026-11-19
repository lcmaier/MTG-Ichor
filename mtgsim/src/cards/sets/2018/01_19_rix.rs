//! Rivals of Ixalan (`rix`), released 2018-01-19.

use std::sync::Arc;

use crate::cards::authoring::{
    all_their, at_beginning_of, choose_objects_with_total_power_at_most, chosen_this_way, creature, dies, each_player, gain_life,
    put_counters, sacrifice, spell_ability, they_control, this_object, triggered_ability, you, Whose,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Condition, CounterType, Effect, PlayerSet, TargetCount};
use crate::types::history::{CountIs, HistoryCount, TurnFact};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{EventRole, TriggerCondition, TriggerDef, TriggerSubject};

/// Paladin of Atonement — {1}{W}
/// Creature — Vampire Knight 1/1
///
/// > At the beginning of each upkeep, if you lost life last turn, put a +1/+1
/// > counter on this creature.
/// > When this creature dies, you gain life equal to its toughness.
///
/// "Last turn" is the game's previous turn, whoever's it was: the ruling
/// reads the history, not the Paladin's presence. "Its toughness" is read off
/// the dies record's frame, the creature as it last existed on the
/// battlefield (CR 608.2h).
///
/// It exercises "last turn" off the histories; "its toughness" off the dies
/// record's frame.
///
/// # The rulings, and where each is tested
///
/// - *"...cares only whether you lost life last turn, even if Paladin of
///   Atonement wasn't on the battlefield when that happened. It doesn't care
///   how much you lost, whether you also gained life, or even whether you
///   gained more life than you lost."*
///   → `phase_tr2a_integration_test::paladin_reads_last_turns_loss_whatever_else_happened`.
/// - *"...use Paladin of Atonement's toughness as it last existed on the
///   battlefield. If its toughness was less than 0, you won't gain life."*
///   → `phase_tr2a_integration_test::paladin_gains_its_last_toughness_and_nothing_below_zero`.
pub fn paladin_of_atonement() -> Arc<CardData> {
    let lost_life_last_turn =
        Condition::LastTurn(HistoryCount { players: PlayerSet::You, fact: TurnFact::LifeLost, of: None }, CountIs::AtLeast(1));
    CardDataBuilder::new("Paladin of Atonement")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Vampire))
        .subtype(Subtype::Creature(CreatureType::Knight))
        .power_toughness(1, 1)
        .rules_text(
            "At the beginning of each upkeep, if you lost life last turn, put a +1/+1 counter on this creature.\n\
             When this creature dies, you gain life equal to its toughness.",
        )
        .ability(triggered_ability("At the beginning of each upkeep, if you lost life last turn, put a +1/+1 counter on this creature.", TriggerDef {
            condition: TriggerCondition::Event(at_beginning_of(StepType::Upkeep, Whose::Each)),
            intervening_if: Some(lost_life_last_turn),
            limit: None,
            effect: put_counters(this_object(), CounterType::PlusOnePlusOne, 1),
        }))
        .ability(triggered_ability("When this creature dies, you gain life equal to its toughness.", TriggerDef {
            condition: TriggerCondition::Event(dies(TriggerSubject::ThisObject).into()),
            intervening_if: None,
            limit: None,
            effect: gain_life(you(), AmountExpr::TriggeringToughness(EventRole::Object)),
        }))
        .build()
}

/// Slaughter the Strong — {1}{W}{W}
/// Sorcery
///
/// > Each player chooses any number of creatures they control with total
/// > power 4 or less, then sacrifices all other creatures they control.
///
/// (Oracle text verified on Scryfall, 2026-11-18.)
///
/// A choice each player makes for themselves, the active player first (CR
/// 101.4, 608.2e), limited by what its creatures' power adds up to: the
/// legal answers are sets, so the question shows each creature's power and
/// the limit, and an answer over it is refused and asked again. Then every
/// creature no player chose is sacrificed by its controller, all at once.
///
/// # The rulings, and where each is tested
///
/// - #1, the total checked over what each player chose ("two 2/2 creatures,
///   or a 1/1 and a 3/3 creature, but not all four"), and #2, the active
///   player first and every creature not chosen sacrificed at once →
///   `phase_several_choosers_at_one_step_integration_test::slaughter_the_strong_takes_a_set_inside_the_total_from_each_player`.
/// - #3, a creature of negative power "subtracts from the total power of
///   the other creatures its controller chooses" →
///   `phase_several_choosers_at_one_step_integration_test::a_creature_of_negative_power_makes_room_under_the_total`.
pub fn slaughter_the_strong() -> Arc<CardData> {
    let text = "Each player chooses any number of creatures they control with total power 4 or less, then sacrifices all other creatures they control.";
    CardDataBuilder::new("Slaughter the Strong")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                choose_objects_with_total_power_at_most(each_player(), creature().and(they_control()), TargetCount::AnyNumber, 4),
                sacrifice(all_their(each_player(), creature().other_than(chosen_this_way()))),
            ]),
        ))
        .build()
}
