//! Phase TR-2a — the histories, the gates and each player (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-09-24 and is
//! quoted verbatim; each card's rulings were read the same day and sit in its
//! own doc comment (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Paladin of Atonement | "last turn" off the histories; "its toughness" off the dies record's frame | no |
//! | Vengeful Warchief | "for the first time each turn": a record's place in its turn | yes |
//! | Elvish Warmaster | "one or more" with "triggers only once each turn"; a pump over a filter, fixed as it resolves | yes |
//! | Temple Bell | "each player draws", in APNAP order (CR 121.2c) | no |

use std::sync::Arc;

use crate::cards::authoring::{another, at_beginning_of, dies, triggered_ability, Whose};
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    AmountExpr, Condition, CounterType, Duration, Effect, EffectRecipient, ObjectFilter,
    PlayerGroup, PlayerRef, PlayerSet, Primitive, TokenDef,
};
use crate::types::history::{CountIs, HistoryCount, TurnFact};
use crate::types::ids::AbilityId;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerLimit, TriggerSubject};

/// "Put a +1/+1 counter on this creature."
fn counter_on_this() -> Effect {
    Effect::Atom(
        Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You },
        EffectRecipient::ThisObject,
    )
}

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
        Condition::LastTurn(HistoryCount { whose: PlayerSet::You, fact: TurnFact::LifeLost, is: CountIs::AtLeast(1) });
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
        .ability(triggered_ability(TriggerDef {
            condition: TriggerCondition::Event(at_beginning_of(StepType::Upkeep, Whose::Each)),
            intervening_if: Some(lost_life_last_turn),
            limit: None,
            effect: counter_on_this(),
        }))
        .ability(triggered_ability(TriggerDef {
            condition: TriggerCondition::Event(dies(TriggerSubject::ThisObject).into()),
            intervening_if: None,
            limit: None,
            effect: Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringToughness), EffectRecipient::Controller),
        }))
        .build()
}

/// Vengeful Warchief — {4}{B}
/// Creature — Orc Warrior 4/4
///
/// > Whenever you lose life for the first time each turn, put a +1/+1 counter
/// > on this creature. (Damage causes loss of life.)
///
/// "The first time each turn" is the loss record's place among this turn's
/// losses (§3.5), so two losses in one batch are the first and the second.
///
/// # The rulings, and where each is tested
///
/// - *"A player loses life if they pay life."*
///   → `phase_tr2a_integration_test::warchief_counts_paid_life_as_lost_life`.
/// - *"You put only one +1/+1 counter on Vengeful Warchief, no matter how much
///   life you lost."* → the same test.
/// - *"If you pay life to cast a spell or activate an ability, you don't put a
///   +1/+1 counter on Vengeful Warchief until after you've finished casting
///   that spell or activating that ability. You put the counter on Vengeful
///   Warchief before that spell or ability resolves."*
///   → `phase_tr2a_integration_test::warchiefs_counter_goes_on_after_the_activation_and_before_it_resolves`.
/// - *"If Vengeful Warchief comes under your control after you've already lost
///   life in a turn, its ability can't trigger during that turn."*
///   → `phase_tr2a_integration_test::a_warchief_that_arrives_after_the_first_loss_waits_for_next_turn`.
pub fn vengeful_warchief() -> Arc<CardData> {
    CardDataBuilder::new("Vengeful Warchief")
        .mana_cost(ManaCost::build(&[ManaType::Black], 4))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Orc))
        .subtype(Subtype::Creature(CreatureType::Warrior))
        .power_toughness(4, 4)
        .rules_text(
            "Whenever you lose life for the first time each turn, put a +1/+1 counter on this creature. \
             (Damage causes loss of life.)",
        )
        .ability(triggered_ability(TriggerDef {
            condition: TriggerCondition::Event(TriggerEvent::LosesLife {
                player: Some(PlayerRef::You),
                multiplicity: Multiplicity::PerOccurrence,
            }),
            intervening_if: None,
            limit: Some(TriggerLimit::FirstTimeEachTurn),
            effect: counter_on_this(),
        }))
        .build()
}

/// "a 1/1 green Elf Warrior creature token" (CR 111.4).
pub fn elf_warrior_token() -> TokenDef {
    TokenDef {
        name: None,
        colors: vec![Color::Green],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Elf), Subtype::Creature(CreatureType::Warrior)],
        supertypes: Vec::new(),
        power: Some(1),
        toughness: Some(1),
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
    }
}

fn elves_you_control() -> ObjectFilter {
    ObjectFilter::And(
        Box::new(ObjectFilter::BySubtype(Subtype::Creature(CreatureType::Elf))),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    )
}

/// Elvish Warmaster — {1}{G}
/// Creature — Elf Warrior 2/2
///
/// > Whenever one or more other Elves you control enter, create a 1/1 green
/// > Elf Warrior creature token. This ability triggers only once each turn.
/// > {5}{G}{G}: Elves you control get +2/+2 and gain deathtouch until end of turn.
///
/// The earliest printing of "this ability triggers only once each turn", so
/// its ruling is the phrase's definition (§3.5): the gate is the trigger's,
/// written as the dispatcher queues it.
///
/// # The rulings, and where each is tested
///
/// - *"It doesn't matter how many Elves enter the battlefield under your
///   control. The ability creates only one Elf Warrior creature token."*
///   → `phase_tr2a_integration_test::warmaster_makes_one_token_however_many_elves_enter`.
/// - *"Once the triggered ability has triggered once during a turn, it can't
///   trigger again, even if the triggered ability is still on the stack, has
///   been countered, or has otherwise left the stack."*
///   → `phase_tr2a_integration_test::warmaster_triggers_once_a_turn_even_while_its_first_trigger_waits`.
/// - *"The activated ability affects only Elves you control as the ability
///   resolves..."* → `phase_tr2a_integration_test::warmasters_pump_is_fixed_as_it_resolves`.
pub fn elvish_warmaster() -> Arc<CardData> {
    let other_elves_you_control = TriggerEvent::EntersBattlefield {
        subject: TriggerSubject::Filter(another(ObjectFilter::BySubtype(Subtype::Creature(CreatureType::Elf)))),
        controller: Some(PlayerRef::You),
        from: None,
        was_cast: None,
        multiplicity: Multiplicity::OncePerEvent,
    };
    let pump = AbilityDef {
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Activated,
        costs: vec![Cost::Mana(ManaCost::build(&[ManaType::Green, ManaType::Green], 5))],
        effect: Effect::Sequence(vec![
            Effect::Atom(
                Primitive::ModifyPowerToughness(AmountExpr::Fixed(2), AmountExpr::Fixed(2), Duration::UntilEndOfTurn),
                EffectRecipient::FilteredPermanents(elves_you_control()),
            ),
            Effect::Atom(
                Primitive::GrantKeywordFlag(KeywordFlag::Deathtouch, Duration::UntilEndOfTurn),
                EffectRecipient::FilteredPermanents(elves_you_control()),
            ),
        ]),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    };
    CardDataBuilder::new("Elvish Warmaster")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elf))
        .subtype(Subtype::Creature(CreatureType::Warrior))
        .power_toughness(2, 2)
        .rules_text(
            "Whenever one or more other Elves you control enter, create a 1/1 green Elf Warrior creature token. \
             This ability triggers only once each turn.\n\
             {5}{G}{G}: Elves you control get +2/+2 and gain deathtouch until end of turn.",
        )
        .ability(triggered_ability(TriggerDef {
            condition: TriggerCondition::Event(other_elves_you_control),
            intervening_if: None,
            limit: Some(TriggerLimit::TriggersOnlyOnceEachTurn),
            effect: Effect::Atom(
                Primitive::CreateToken(elf_warrior_token(), AmountExpr::Fixed(1)),
                EffectRecipient::Controller,
            ),
        }))
        .ability(pump)
        .build()
}

/// Temple Bell — {3}
/// Artifact
///
/// > {T}: Each player draws a card.
///
/// One instruction to every player, which CR 121.2c performs in APNAP order:
/// the active player draws first. No rulings.
pub fn temple_bell() -> Arc<CardData> {
    CardDataBuilder::new("Temple Bell")
        .mana_cost(ManaCost::build(&[], 3))
        .card_type(CardType::Artifact)
        .rules_text("{T}: Each player draws a card.")
        .ability(AbilityDef {
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::TapSelf],
            effect: Effect::Atom(
                Primitive::DrawCards(AmountExpr::Fixed(1)),
                EffectRecipient::EachOf(PlayerGroup::set(PlayerSet::Everyone)),
            ),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
        })
        .build()
}
