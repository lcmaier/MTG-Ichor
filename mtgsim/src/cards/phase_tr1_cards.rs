//! Phase TR-1 — the trigger spine's five cards (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-09-19 and is
//! quoted verbatim; each card's rulings were read the same day and sit in
//! its own doc comment (`engineering-practices.md` §3.4). One card per
//! engine path the phase opens, and the three that measure a path are in
//! `PERFORMANCE_POOL`:
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Soul Warden | the matcher at a batch's close: CR 603.6a's "another", two entries in one window | yes |
//! | Blood Artist | CR 603.10a's look-back off the frame, a target chosen at placement (603.3d) | yes |
//! | Verdant Force | "each upkeep" — a step beginning, a token from a trigger | no |
//! | Wild Growth | CR 605.1b's stackless mana trigger inside the mana window | yes |
//! | Felidar Sovereign | CR 603.4's intervening "if" at both instants, `WinGame` | no |
//!
//! A triggered ability is `AbilityType::Triggered` with `Effect::Triggered`,
//! and a card touches nothing but this file: the arms it reads are
//! `TriggerEvent`'s, the predicates are the filters the rest of the engine
//! shares, and "another" is `NotSource` beside the type leaf.

use std::sync::Arc;

use crate::cards::authoring::{another, at_beginning_of, dies, enters, triggered_ability, whenever, Whose};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, EnchantmentType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, Condition, Effect, EffectRecipient, ManaOutput, ObjectFilter, PlayerFact, PlayerSet,
    Primitive, SelectionFilter, TargetCount, TokenDef,
};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{TriggerCondition, TriggerDef, TriggerEvent, TriggerSubject};
use crate::state::game_state::StepType;

/// Soul Warden — {W}
/// Creature — Human Cleric 1/1
///
/// > Whenever another creature enters, you gain 1 life.
///
/// The matcher's card: CR 603.6a's "another" is the `NotSource` leaf, and
/// the batch close is what makes two creatures entering together two
/// triggers rather than one and none.
///
/// # The rulings, and where each is tested
///
/// - *"If this creature enters at the same time as one or more other
///   creatures, its ability will trigger for each of those other creatures."*
///   → `phase_tr1_integration_test::soul_warden_entering_beside_two_creatures_triggers_for_each_of_them`.
pub fn soul_warden() -> Arc<CardData> {
    CardDataBuilder::new("Soul Warden")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(1, 1)
        .rules_text("Whenever another creature enters, you gain 1 life.")
        .ability(triggered_ability(whenever(
            enters(another(ObjectFilter::ByType(CardType::Creature))),
            Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
        )))
        .build()
}

/// Blood Artist — {1}{B}
/// Creature — Vampire 0/1
///
/// > Whenever this creature or another creature dies, target player loses 1
/// > life and you gain 1 life.
///
/// "This creature or another creature" is any creature, so the subject is
/// the plain type filter; its own death is found on the CR 603.10a frame the
/// record carries, and the target is chosen as the ability goes on the
/// stack (CR 603.3d), which is the placement prompt the pool measures.
///
/// # The rulings, and where each is tested
///
/// - *"If Blood Artist and one or more other creatures die at the same time,
///   its ability will trigger for each of those creatures."*
///   → `phase_tr1_integration_test::blood_artist_dying_beside_two_creatures_triggers_three_times`.
pub fn blood_artist() -> Arc<CardData> {
    CardDataBuilder::new("Blood Artist")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Vampire))
        .power_toughness(0, 1)
        .rules_text("Whenever this creature or another creature dies, target player loses 1 life and you gain 1 life.")
        .ability(triggered_ability(whenever(
            dies(ObjectFilter::ByType(CardType::Creature)),
            Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::LoseLife(AmountExpr::Fixed(1)),
                    EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
                ),
                Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
            ]),
        )))
        .build()
}

/// "a 1/1 green Saproling creature token" — named "Saproling Token" (CR 111.4).
pub fn saproling_token() -> TokenDef {
    TokenDef {
        name: None,
        colors: vec![Color::Green],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Saproling)],
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

/// Verdant Force — {5}{G}{G}{G}
/// Creature — Elemental 7/7
///
/// > At the beginning of each upkeep, create a 1/1 green Saproling creature token.
///
/// "Each upkeep" is `Whose::Each`: the step's record carries whose it is
/// (item 10) and this ability does not ask.
///
/// # The rulings, and where each is tested
///
/// - *"Verdant Force's ability triggers at the beginning of each upkeep, not
///   just each of your upkeeps."*
///   → `phase_tr1_integration_test::verdant_force_triggers_at_an_opponents_upkeep_too`.
/// - The Two-Headed Giant ruling is a format-variant disposition in the
///   ledger: CR 810 is not built.
pub fn verdant_force() -> Arc<CardData> {
    CardDataBuilder::new("Verdant Force")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green, ManaType::Green], 5))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elemental))
        .power_toughness(7, 7)
        .rules_text("At the beginning of each upkeep, create a 1/1 green Saproling creature token.")
        .ability(triggered_ability(whenever(
            at_beginning_of(StepType::Upkeep, Whose::Each),
            Effect::Atom(
                Primitive::CreateToken(saproling_token(), AmountExpr::Fixed(1)),
                EffectRecipient::Controller,
            ),
        )))
        .build()
}

/// Wild Growth — {G}
/// Enchantment — Aura
///
/// > Enchant land
/// > Whenever enchanted land is tapped for mana, its controller adds an
/// > additional {G}.
///
/// CR 605.1b's triggered mana ability, derived and never tagged: no target,
/// triggers from mana being added, could add mana — so the dispatcher
/// resolves it at once (CR 605.4a), inside the CR 601.2g window when the
/// land was tapped there, and the stack never sees it. "Its controller adds"
/// is the `Host` recipient: the land's controller, whoever controls the Aura.
///
/// # The rulings, and where each is tested
///
/// - *"The additional mana is not an ability of the land and is not
///   something the land can produce."*
///   → `phase_tr1_integration_test::wild_growths_mana_is_the_auras_and_does_not_tap_the_land_for_mana_again`:
///   the record's source is the Aura and `tapped_for_mana` is false, so a
///   second Wild Growth on the same land adds one more {G}, not two.
pub fn wild_growth() -> Arc<CardData> {
    CardDataBuilder::new("Wild Growth")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .rules_text("Enchant land\nWhenever enchanted land is tapped for mana, its controller adds an additional {G}.")
        .enchant_filter(SelectionFilter::Permanent(ObjectFilter::ByType(CardType::Land)))
        .ability(triggered_ability(whenever(
            TriggerEvent::ManaAdded { source: TriggerSubject::Host, tapped_for_mana: Some(true), mana: None },
            Effect::Atom(
                Primitive::ProduceMana(ManaOutput {
                    mana: vec![(ManaType::Green, AmountExpr::Fixed(1))],
                    special: vec![],
                }),
                EffectRecipient::Host,
            ),
        )))
        .build()
}

/// Felidar Sovereign — {4}{W}{W}
/// Creature — Cat Beast 4/6
///
/// > Vigilance (Attacking doesn't cause this creature to tap.)
/// > Lifelink (Damage dealt by this creature also causes you to gain that much life.)
/// > At the beginning of your upkeep, if you have 40 or more life, you win the game.
///
/// CR 603.4's intervening "if": checked as the upkeep begins, and again as
/// the ability resolves (608.2a). "Your upkeep" is `Whose::Yours`.
///
/// # The rulings, and where each is tested
///
/// - *"Felidar Sovereign's triggered ability checks to see if you have 40 or
///   more life as your upkeep begins. If you don't, the ability won't trigger
///   at all. If you do, the ability will check again as it tries to resolve.
///   If you don't have 40 or more life at that time, the ability won't do
///   anything."* → the three tests
///   `phase_tr1_integration_test::felidar_sovereign_wins_when_the_condition_holds_at_both_instants`,
///   `felidar_sovereign_does_not_trigger_below_forty_life` and
///   `felidar_sovereign_does_nothing_when_life_drops_before_it_resolves`.
/// - The Two-Headed Giant ruling is a format-variant disposition in the
///   ledger: CR 810 is not built.
pub fn felidar_sovereign() -> Arc<CardData> {
    CardDataBuilder::new("Felidar Sovereign")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 4))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Cat))
        .subtype(Subtype::Creature(CreatureType::Beast))
        .power_toughness(4, 6)
        .keyword_flag(KeywordFlag::Vigilance)
        .keyword_flag(KeywordFlag::Lifelink)
        .rules_text("Vigilance\nLifelink\nAt the beginning of your upkeep, if you have 40 or more life, you win the game.")
        .ability(triggered_ability(TriggerDef {
            condition: TriggerCondition::Event(at_beginning_of(StepType::Upkeep, Whose::Yours)),
            intervening_if: Some(Condition::Player {
                whose: PlayerSet::You,
                fact: PlayerFact::LifeAtLeast(AmountExpr::Fixed(40)),
            }),
            limit: None,
            effect: Effect::Atom(Primitive::WinGame, EffectRecipient::Controller),
        }))
        .build()
}
