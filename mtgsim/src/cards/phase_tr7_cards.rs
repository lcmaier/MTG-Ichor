//! Phase TR-7 — the persist loop (`triggers-architecture.md` §12, TR-7).
//!
//! The oracle text below was verified on Scryfall on 2026-10-21 and is quoted
//! verbatim; each card's rulings were read the same day and sit in its doc
//! comment (`engineering-practices.md#rulings`). With Blood Artist the three
//! make a loop: sacrifice the Scarecrow to the Seer, persist returns it with
//! a -1/-1 counter beside Grumgully's +1/+1, CR 704.5q removes the pair, and
//! it can persist again.
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Grumgully, the Generous | an entry counter over "not a Human", read after CR 616.1c's copy step | yes |
//! | Wingrattle Scarecrow | a triggered ability granted under a condition, asked off the frame of the object that left | yes |
//! | Viscera Seer | an activated ability whose cost sacrifices a chosen creature | no: a random agent sacrifices its own board with it |

use std::sync::Arc;

use crate::cards::authoring::persist;
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    Antecedent, AmountExpr, Condition, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerFact, PlayerRef, PlayerSet, Primitive,
};
use crate::types::ids::AbilityId;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EnterModsTemplate, EventPattern, ReplacementDef, Rewrite};

/// "You control a [color] creature."
fn you_control_a_creature_of(color: Color) -> Condition {
    let creature_of_color = ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ByColor(color)));
    Condition::Player { players: PlayerSet::You, fact: PlayerFact::ControlsPermanent(creature_of_color) }
}

/// Grumgully, the Generous — {1}{R}{G}
/// Legendary Creature — Goblin Shaman, 3/3
///
/// > Each other non-Human creature you control enters with an additional
/// > +1/+1 counter on it.
///
/// Master Biomancer's shape with a fixed amount: an entry replacement over the
/// creatures you control that are not Humans. "Other" needs no filter leaf.
/// The gather sweeps the battlefield, where an entering Grumgully is not, so
/// it neither counts itself nor a creature entering beside it.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr7_integration_test`.
/// - *A non-Human creature you control that would enter with no +1/+1
///   counters enters with one* (#1) → `grumgully_gives_each_non_human_you_control_its_counter`.
/// - *Other non-Humans entering at the same time as Grumgully get none* (#2)
///   → `a_creature_entering_beside_grumgully_gets_no_counter`.
/// - *A copy of a Human gets none, and a Human entering as a non-Human gets
///   one* (#3) → `grumgully_reads_the_creature_a_copy_enters_as`: CR 614.12's
///   look-ahead, after CR 616.1c's copy step.
pub fn grumgully_the_generous() -> Arc<CardData> {
    let text = "Each other non-Human creature you control enters with an additional +1/+1 counter on it.";
    let non_human_creature_you_control = ObjectFilter::And(
        Box::new(ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ByController(PlayerSet::You)))),
        Box::new(ObjectFilter::Not(Box::new(ObjectFilter::BySubtype(Subtype::Creature(CreatureType::Human))))),
    );
    CardDataBuilder::new("Grumgully, the Generous")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Green], 1))
        .color(Color::Red)
        .color(Color::Green)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Goblin))
        .subtype(Subtype::Creature(CreatureType::Shaman))
        .power_toughness(3, 3)
        .rules_text(text)
        .ability(AbilityDef {
            rules_text: text.into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                ObjectSet::battlefield_filter(non_human_creature_you_control),
                Rewrite::EnterWith(EnterModsTemplate::with_counters(CounterType::PlusOnePlusOne, 1)),
            ))),
        })
        .build()
}

/// Wingrattle Scarecrow — {3}
/// Artifact Creature — Scarecrow, 2/2
///
/// > This creature has flying as long as you control a blue creature.
/// > This creature has persist as long as you control a black creature.
///
/// Two conditional statics over the Scarecrow itself, Kird Ape's shape: one
/// grants a keyword, the other persist, a triggered ability. Persist is
/// asked off the frame of the Scarecrow that died (CR 603.10a), so it
/// triggers if a black creature was there as it left, the black creature
/// dying beside it included, and once on the stack it no longer needs one.
///
/// # The rulings, and where each is tested
///
/// Kitchen Finks' seven, word for word. `phase_tr4a_integration_test` proves
/// them on printed persist; each is asked again in
/// `phase_tr7_integration_test` on the Scarecrow, beside a black creature.
/// - *A creature with persist that stops being a creature still persists*
///   (#1) → `the_scarecrow_persists_after_it_stops_being_a_creature`.
/// - *Whether it had a -1/-1 counter is read off how it last existed on the
///   battlefield* (#2) → `the_scarecrow_returns_once_and_then_stays_dead` and
///   `a_scarecrow_that_died_with_a_minus_counter_stays_dead`.
/// - *Several instances each trigger, and the next to resolve does nothing*
///   (#3) → `two_instances_of_persist_return_the_scarecrow_once`.
/// - *A token with persist triggers and cannot return* (#4) →
///   `a_scarecrow_token_persists_into_nothing`.
/// - *It returns as a new object with no memory of the last* (#5) →
///   `the_scarecrow_returns_as_a_new_object`.
/// - *Simultaneous persists go on in APNAP order and return one at a time,
///   the nonactive player's first* (#6) →
///   `two_players_scarecrows_return_the_nonactive_players_first`.
/// - *A creature with +1/+1 counters killed by -1/-1 counters dies with both
///   on it, and persist does not trigger* (#7) →
///   `a_scarecrow_killed_through_its_plus_counter_stays_dead`.
pub fn wingrattle_scarecrow() -> Arc<CardData> {
    let flying = "This creature has flying as long as you control a blue creature.";
    let persists = "This creature has persist as long as you control a black creature.";
    let has_while_you_control_a = |text: &'static str, color: Color, grant: Primitive| AbilityDef {
        rules_text: text.into(),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect: Effect::Conditional(you_control_a_creature_of(color), Box::new(Effect::Atom(grant, EffectRecipient::Object(Antecedent::ThisObject)))),
    };
    CardDataBuilder::new("Wingrattle Scarecrow")
        .mana_cost(ManaCost::build(&[], 3))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Scarecrow))
        .power_toughness(2, 2)
        .rules_text("This creature has flying as long as you control a blue creature.\nThis creature has persist as long as you control a black creature.")
        .ability(has_while_you_control_a(flying, Color::Blue, Primitive::GrantKeywordFlag(KeywordFlag::Flying, Duration::WhileSourceOnBattlefield)))
        .ability(has_while_you_control_a(persists, Color::Black, Primitive::GrantAbility(Arc::new(persist()), Duration::WhileSourceOnBattlefield)))
        .build()
}

/// Viscera Seer — {B}
/// Creature — Vampire Wizard, 1/1
///
/// > Sacrifice a creature: Scry 1.
///
/// The sacrifice is a cost (CR 602.2b), so the creature is gone before the
/// ability is on the stack and its death triggers go above it.
///
/// # Not in `PERFORMANCE_POOL`
///
/// A random agent takes the activation at nearly every priority and
/// sacrifices its own creatures with it, so the pool's boards would thin out
/// (the owner, at #269's review). `stress`, which drives the engine every way
/// it can be driven, plays it.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr7_integration_test`.
/// - *It can sacrifice itself* (#1) → `viscera_seer_can_sacrifice_itself`.
/// - *A creature sacrificed after blockers are declared deals no combat
///   damage* (#2) → `a_blocked_creature_sacrificed_to_the_seer_deals_no_combat_damage`.
pub fn viscera_seer() -> Arc<CardData> {
    let text = "Sacrifice a creature: Scry 1.";
    CardDataBuilder::new("Viscera Seer")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Vampire))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .power_toughness(1, 1)
        .rules_text(text)
        .ability(AbilityDef {
            rules_text: text.into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::Sacrifice(ObjectFilter::ByType(CardType::Creature), 1)],
            effect: Effect::Atom(Primitive::Scry(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You)),
        })
        .build()
}
