//! Phase TR-5b — counters (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-10-17 and is
//! quoted verbatim; each card's rulings were read the same day and sit in its
//! own doc comment (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Simic Ascendancy | a put read once per creature, that many counters of a kind no rule reads, and a win behind an intervening "if" over them | yes |
//! | Protean Hydra | an entry template's X (CR 107.3m), a prevention whose rider removes that many counters, and a removal read once per counter that makes a delayed trigger | yes; castable from MA-2, the next PR, which offers an X spell at the gate |

use std::sync::Arc;

use crate::cards::authoring::{at_beginning_of, counters_put_on, counters_removed_from, triggered_ability, whenever, Whose};
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    Antecedent, AmountExpr, Condition, CounterType, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerRef, Primitive,
    SelectionFilter, TargetCount,
};
use crate::types::history::CountIs;
use crate::types::ids::AbilityId;
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::types::replacement::{EnterModsTemplate, EventPattern, ReplacementDef, Rewrite};
use crate::types::triggers::{DelayedDuration, DelayedTriggerTemplate, DelayedTurn, TriggerCondition, TriggerDef, TriggerSubject};

/// A static ability carrying one replacement effect.
fn static_replacement(rules_text: &'static str, def: ReplacementDef) -> AbilityDef {
    AbilityDef {
        rules_text: rules_text.into(),
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect: Effect::Replacement(Box::new(def)),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    }
}

fn a_creature_you_control() -> ObjectFilter {
    ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ByController(PlayerRef::You)))
}

/// Simic Ascendancy — {G}{U}
/// Enchantment
///
/// > {1}{G}{U}: Put a +1/+1 counter on target creature you control.
/// > Whenever one or more +1/+1 counters are put on a creature you control,
/// > put that many growth counters on this enchantment.
/// > At the beginning of your upkeep, if this enchantment has twenty or more
/// > growth counters on it, you win the game.
///
/// The growth counters are its own: the put is read once for each creature
/// (`once_per_object`), and "that many" is that creature's. The win is CR
/// 603.4's intervening "if" over the counters on it, asked as the upkeep
/// begins and again as the ability resolves.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr5b_integration_test`.
/// - *"An ability that triggers when counters are put on a permanent will
///   trigger if that permanent somehow enters the battlefield with those
///   counters."* (#1) →
///   `simic_ascendancy_counts_the_counters_a_creature_enters_with`.
/// - *"If Simic Ascendancy doesn't have twenty or more growth counters on it
///   as your upkeep begins, its last ability won't trigger. You can't take
///   any actions during your turn before your upkeep begins."* (#2) →
///   `simic_ascendancy_asks_for_twenty_as_the_upkeep_begins`.
/// - *"If the last ability does trigger, but counters are removed from Simic
///   Ascendancy so it has fewer than twenty remaining on it, you won't win
///   the game."* (#3) →
///   `simic_ascendancy_does_not_win_once_its_counters_fall_below_twenty`.
/// - *"If the last ability does trigger, but Simic Ascendancy leaves the
///   battlefield, use the number of counters it had on it immediately before
///   it left the battlefield to determine whether you win the game."* (#4) →
///   `simic_ascendancy_wins_off_the_count_it_had_as_it_left`.
pub fn simic_ascendancy() -> Arc<CardData> {
    let activated_text = "{1}{G}{U}: Put a +1/+1 counter on target creature you control.";
    let growth_text = "Whenever one or more +1/+1 counters are put on a creature you control, put that many growth counters on this enchantment.";
    let win_text =
        "At the beginning of your upkeep, if this enchantment has twenty or more growth counters on it, you win the game.";
    let growth = Effect::Atom(
        Primitive::AddCounters { counter: CounterType::Growth, amount: AmountExpr::TriggeringAmount, by: PlayerRef::You },
        EffectRecipient::ThisObject,
    );
    let win = TriggerDef {
        condition: TriggerCondition::Event(at_beginning_of(StepType::Upkeep, Whose::Yours)),
        intervening_if: Some(Condition::CounterCount {
            object: Antecedent::ThisObject,
            kind: Some(CounterType::Growth),
            count: CountIs::AtLeast(20),
        }),
        limit: None,
        effect: Effect::Atom(Primitive::WinGame, EffectRecipient::Controller),
    };
    CardDataBuilder::new("Simic Ascendancy")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Blue], 0))
        .color(Color::Green)
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text(&format!("{activated_text}\n{growth_text}\n{win_text}"))
        .ability(AbilityDef {
            rules_text: activated_text.into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::Mana(ManaCost::build(&[ManaType::Green, ManaType::Blue], 1))],
            effect: Effect::Atom(
                Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You },
                EffectRecipient::Target(SelectionFilter::Permanent(a_creature_you_control()), TargetCount::Exactly(1)),
            ),
        })
        .ability(triggered_ability(
            growth_text,
            whenever(counters_put_on(a_creature_you_control(), CounterType::PlusOnePlusOne).once_per_object(), growth),
        ))
        .ability(triggered_ability(win_text, win))
        .build()
}

/// Protean Hydra — {X}{G}
/// Creature — Hydra 0/0
///
/// > This creature enters with X +1/+1 counters on it.
/// > If damage would be dealt to this creature, prevent that damage and
/// > remove that many +1/+1 counters from it.
/// > Whenever a +1/+1 counter is removed from this creature, put two +1/+1
/// > counters on it at the beginning of the next end step.
///
/// X is the X of the spell becoming it (CR 107.3m), 0 for any other way onto
/// the battlefield. The prevention is Angel of Suffering's shape: `Prevent`,
/// and a CR 615.5 rider reading "that many", the damage that would have been
/// dealt. The removal is read once for each counter, and each makes a delayed
/// trigger whose "it" is this creature, found by identity (CR 400.7).
///
/// # The rulings, and where each is tested
///
/// In `phase_tr5b_integration_test`.
/// - *"All damage that would be dealt to Protean Hydra is prevented, even if
///   it doesn't have that many +1/+1 counters on it. This is true even if
///   Protean Hydra has no +1/+1 counters on it at all (and is able to remain
///   on the battlefield because some other effect is boosting its
///   toughness). If the amount of damage that would be dealt to Protean
///   Hydra is greater than the number of +1/+1 counters on it, all the +1/+1
///   counters on Protean Hydra are removed from it."* (#1) →
///   `protean_hydra_prevents_all_damage_and_loses_the_counters_it_has`.
/// - *"If unpreventable damage is dealt to Protean Hydra, the Hydra's second
///   ability will try to prevent it and fail (meaning that damage has its
///   normal results), and it will also remove that many +1/+1 counters from
///   Protean Hydra."* (#2) →
///   `protean_hydra_loses_counters_to_damage_that_cannot_be_prevented`.
/// - *"Protean Hydra's last ability triggers whenever a +1/+1 counter is
///   removed from it for any reason, not just when a +1/+1 counter is
///   removed by its second ability."* (#3) →
///   `protean_hydra_grows_back_from_any_removal`.
/// - *"If multiple +1/+1 counters are removed from Protean Hydra at once, its
///   last ability will trigger that many times."* (#4) →
///   `protean_hydra_triggers_once_for_each_counter_removed`.
/// - *"If a -1/-1 counter is put on Protean Hydra while it has +1/+1 counters
///   on it, that -1/-1 counter and a +1/+1 counter will be removed from it as
///   a state-based action. This will cause Protean Hydra's last ability to
///   trigger."* (#5) → `protean_hydra_triggers_on_counter_annihilation`.
/// - *"If any +1/+1 counters are removed from Protean Hydra during the end
///   step, its last ability will trigger and set up a delayed triggered
///   ability that will trigger at the beginning of the next end step (which
///   is near the end of the following turn)."* (#6) →
///   `protean_hydra_removed_during_an_end_step_grows_at_the_next_turns`.
pub fn protean_hydra() -> Arc<CardData> {
    let enters_text = "This creature enters with X +1/+1 counters on it.";
    let prevent_text =
        "If damage would be dealt to this creature, prevent that damage and remove that many +1/+1 counters from it.";
    let grow_text = "Whenever a +1/+1 counter is removed from this creature, put two +1/+1 counters on it at the beginning of the next end step.";
    let two_more = DelayedTriggerTemplate {
        def: Arc::new(whenever(
            at_beginning_of(StepType::End, Whose::Each),
            Effect::Atom(
                Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(2), by: PlayerRef::You },
                EffectRecipient::ThisObject,
            ),
        )),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::NoneNamed,
        rules_text: "Put two +1/+1 counters on it at the beginning of the next end step.".into(),
    };
    let prevent = ReplacementDef::new(EventPattern::DealDamage { source: None, combat: None }, ObjectSet::SourceOnly, Rewrite::Prevent)
        .with_then(Effect::Atom(
            Primitive::RemoveCounters(CounterType::PlusOnePlusOne, AmountExpr::ReplacedAmount),
            EffectRecipient::ThisObject,
        ));
    CardDataBuilder::new("Protean Hydra")
        .mana_cost(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Green)]))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Hydra))
        .power_toughness(0, 0)
        .rules_text(&format!("{enters_text}\n{prevent_text}\n{grow_text}"))
        .ability(static_replacement(
            enters_text,
            ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                ObjectSet::SourceOnly,
                Rewrite::EnterWith(EnterModsTemplate::with_counter_amount(CounterType::PlusOnePlusOne, AmountExpr::X)),
            ),
        ))
        .ability(static_replacement(prevent_text, prevent))
        .ability(triggered_ability(
            grow_text,
            whenever(
                counters_removed_from(TriggerSubject::ThisObject, CounterType::PlusOnePlusOne),
                Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(two_more)), EffectRecipient::Controller),
            ),
        ))
        .build()
}
