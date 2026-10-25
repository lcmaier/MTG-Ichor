//! Phase TR-4c — the other two look-back classes and the zone statement
//! (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-10-15 and is
//! quoted verbatim; each card's rulings were read the same day and sit in its
//! own doc comment (`engineering-practices.md#rulings`).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Bridge from Below | an intervening "if" that states the zone its ability works in (CR 113.6b), asked from a graveyard; a card that reaches the graveyard in the event answers none of its look-back triggers (CR 603.10a) | yes |
//! | Guerrilla Tactics | a trigger on this card leaving a hidden zone (its discard), asked where the card went (CR 603.10, 113.6k, 702.29c) | no |
//! | Angelic Destiny | an Aura's trigger on its creature dying finds the card the Aura became in its owner's graveyard (CR 603.6e, 400.7f) | no |
//! | Rancor | an Aura's trigger on itself going to the graveyard, finding the card it became (CR 400.7e) | no |

use std::sync::Arc;

use crate::cards::authoring::{dies, triggered_ability, whenever};
use crate::cards::phase_rb_cards::zombie_token;
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, EnchantmentType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, SelectionFilter,
    TargetCount, TypeChange,
};
use crate::types::ids::AbilityId;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::restriction::SourceFilter;
use crate::types::triggers::{EventRole, Multiplicity, TriggerDef, TriggerEvent, TriggerSubject};
use crate::types::zones::{Zone, ZoneChangeAct, ZoneChangePattern, ZoneSet};

/// "Whenever [subject] is put into [whose] graveyard from the battlefield":
/// the graveyard named by its owner, whoever controlled the creature
/// (Bridge from Below's second ruling).
fn put_into_a_graveyard_from_the_battlefield(subject: ObjectFilter, whose: PlayerRef) -> TriggerEvent {
    TriggerEvent::ZoneChange {
        subject: TriggerSubject::Filter(subject),
        change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: None },
        by: None,
        owner: Some(whose),
        multiplicity: Multiplicity::PerOccurrence,
    }
}

/// "[It] deals `n` damage to any target."
fn deals_damage_to_any_target(n: u64) -> Effect {
    Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::Fixed(n), unpreventable: false },
        EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
    )
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
        .ability(AbilityDef {
            rules_text: spell.into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: deals_damage_to_any_target(2),
        })
        .ability(triggered_ability(trigger, whenever(discarded, deals_damage_to_any_target(4))))
        .build()
}

/// A static ability of an Aura, on the creature it enchants.
fn enchanted_creature(rules_text: &'static str, primitives: Vec<Primitive>) -> AbilityDef {
    AbilityDef {
        rules_text: rules_text.into(),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect: Effect::Sequence(primitives.into_iter().map(|p| Effect::Atom(p, EffectRecipient::Host)).collect()),
    }
}

/// Angelic Destiny — {2}{W}{W}
/// Enchantment — Aura
///
/// > Enchant creature
/// > Enchanted creature gets +4/+4, has flying and first strike, and is an
/// > Angel in addition to its other types.
/// > When enchanted creature dies, return this card to its owner's hand.
///
/// The trigger is CR 603.6e's: it looks back to the creature's departure,
/// and "this card" is the Aura's new object in its owner's graveyard. CR
/// 400.7f lets it find that object when the Aura went there in the same
/// event as the creature, or by CR 704.5m's check that follows, which is
/// the usual way: the Aura stays on the battlefield, unattached, until then.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr4c_integration_test`.
/// - *"If the creature dies before the Angelic Destiny spell resolves,
///   Angelic Destiny will go to its owner's graveyard. It won't return to its
///   owner's hand."* (#1) →
///   `angelic_destiny_whose_creature_died_before_it_resolved_stays_in_the_graveyard`.
/// - *"If Angelic Destiny is no longer in a graveyard when its triggered
///   ability resolves, it won't be returned to its owner's hand."* (#2) →
///   `angelic_destiny_gone_from_the_graveyard_is_not_returned`.
pub fn angelic_destiny() -> Arc<CardData> {
    let anthem = "Enchanted creature gets +4/+4, has flying and first strike, and is an Angel in addition to its other types.";
    let returns = "When enchanted creature dies, return this card to its owner's hand.";
    let angel = TypeChange {
        add_types: Vec::new(),
        remove_types: Vec::new(),
        set_types: None,
        add_subtypes: vec![Subtype::Creature(CreatureType::Angel)],
        remove_subtypes: Vec::new(),
        set_subtypes: None,
        add_supertypes: Vec::new(),
        remove_supertypes: Vec::new(),
        set_supertypes: None,
    };
    CardDataBuilder::new("Angelic Destiny")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .enchant_filter(SelectionFilter::Creature)
        .rules_text("Enchant creature\nEnchanted creature gets +4/+4, has flying and first strike, and is an Angel in addition to its other types.\nWhen enchanted creature dies, return this card to its owner's hand.")
        .ability(enchanted_creature(
            anthem,
            vec![
                Primitive::ModifyPowerToughness(AmountExpr::Fixed(4), AmountExpr::Fixed(4), Duration::WhileSourceOnBattlefield),
                Primitive::GrantKeywordFlag(KeywordFlag::Flying, Duration::WhileSourceOnBattlefield),
                Primitive::GrantKeywordFlag(KeywordFlag::FirstStrike, Duration::WhileSourceOnBattlefield),
                Primitive::ChangeType(angel, Duration::WhileSourceOnBattlefield),
            ],
        ))
        .ability(triggered_ability(
            returns,
            whenever(dies(TriggerSubject::Host), Effect::Atom(Primitive::ReturnToHand, EffectRecipient::ThisObject)),
        ))
        .build()
}

/// Rancor — {G}
/// Enchantment — Aura
///
/// > Enchant creature
/// > Enchanted creature gets +2/+0 and has trample.
/// > When this Aura is put into a graveyard from the battlefield, return it
/// > to its owner's hand.
///
/// A trigger on the Aura itself going to the graveyard: "it" is the card it
/// became there (CR 400.7e), which the binding names since RH-3.
///
/// # The ruling, and where it is tested
///
/// In `phase_tr4c_integration_test`.
/// - *"If the creature this Aura would enchant is an illegal target by the
///   time Rancor tries to resolve, the Aura spell doesn't resolve. It won't
///   enter the battlefield, so it won't be put into a graveyard from the
///   battlefield and its ability won't trigger."* (#1) →
///   `rancor_whose_creature_left_before_it_resolved_does_not_trigger`.
pub fn rancor() -> Arc<CardData> {
    let anthem = "Enchanted creature gets +2/+0 and has trample.";
    let returns = "When this Aura is put into a graveyard from the battlefield, return it to its owner's hand.";
    CardDataBuilder::new("Rancor")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .enchant_filter(SelectionFilter::Creature)
        .rules_text("Enchant creature\nEnchanted creature gets +2/+0 and has trample.\nWhen this Aura is put into a graveyard from the battlefield, return it to its owner's hand.")
        .ability(enchanted_creature(
            anthem,
            vec![
                Primitive::ModifyPowerToughness(AmountExpr::Fixed(2), AmountExpr::Fixed(0), Duration::WhileSourceOnBattlefield),
                Primitive::GrantKeywordFlag(KeywordFlag::Trample, Duration::WhileSourceOnBattlefield),
            ],
        ))
        .ability(triggered_ability(
            returns,
            whenever(dies(TriggerSubject::ThisObject), Effect::Atom(Primitive::ReturnToHand, EffectRecipient::Triggering(EventRole::Object))),
        ))
        .build()
}

/// Bridge from Below — {B}{B}{B}
/// Enchantment
///
/// > Whenever a nontoken creature is put into your graveyard from the
/// > battlefield, if this card is in your graveyard, create a 2/2 black Zombie
/// > creature token.
/// > When a creature is put into an opponent's graveyard from the
/// > battlefield, if this card is in your graveyard, exile this card.
///
/// Each intervening "if" states where its ability works (CR 113.6b): in a
/// graveyard and nowhere else, so on the battlefield the card does nothing.
/// Both abilities are leaves-the-battlefield abilities and look back (CR
/// 603.10a), so the card has to be in the graveyard before the creature dies.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The phase's new dispatcher path: while the card sits in a graveyard,
/// every death is matched against a trigger there. Forced into every `stress`
/// deck, 200 games at seed 777 and two seats, it reached a graveyard 23 times
/// (18 discarded) and triggered 24 times, and the average game ran 27.1 turns
/// against 26.7. It does nothing on the battlefield, so it ends no game.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr4c_integration_test`.
/// - *"While Bridge from Below is on the battlefield, it has no effect. It
///   does something only if it's in a graveyard. Yes, this is a bit weird."*
///   (#1) → `bridge_from_below_on_the_battlefield_does_nothing`.
/// - *"Neither ability cares who controlled the creature that died, only
///   which graveyard it was put into. ..."* (#2) →
///   `bridge_from_below_reads_whose_graveyard_not_who_controlled_the_creature`.
/// - *"If Bridge from Below isn't in your graveyard immediately before a
///   creature dies, its abilities won't trigger at all. If Bridge from Below
///   isn't still in your graveyard as its first ability resolves, you won't
///   create a token."* (#3) →
///   `bridge_from_below_put_into_the_graveyard_by_the_same_wipe_does_not_trigger`
///   and `bridge_from_below_gone_from_the_graveyard_makes_no_token`.
/// - *"If a nontoken creature you control and a creature an opponent controls
///   die at the same time, you choose the order in which Bridge from Below's
///   triggered abilities resolve. You can create a token before you exile
///   Bridge from Below."* (#4) →
///   `bridge_from_below_can_make_its_token_before_it_exiles_itself`.
pub fn bridge_from_below() -> Arc<CardData> {
    let zombie = "Whenever a nontoken creature is put into your graveyard from the battlefield, if this card is in your graveyard, create a 2/2 black Zombie creature token.";
    let exile = "When a creature is put into an opponent's graveyard from the battlefield, if this card is in your graveyard, exile this card.";
    let nontoken_creature =
        ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::Not(Box::new(ObjectFilter::Token))));
    CardDataBuilder::new("Bridge from Below")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black, ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("Whenever a nontoken creature is put into your graveyard from the battlefield, if this card is in your graveyard, create a 2/2 black Zombie creature token.\nWhen a creature is put into an opponent's graveyard from the battlefield, if this card is in your graveyard, exile this card.")
        .ability(triggered_ability(
            zombie,
            TriggerDef {
                // CR 113.6b's statement, printed as the intervening "if" (CR 603.4).
                functions_in: Some(ZoneSet::GRAVEYARD),
                ..whenever(
                    put_into_a_graveyard_from_the_battlefield(nontoken_creature, PlayerRef::You),
                    Effect::Atom(Primitive::CreateToken(zombie_token(), AmountExpr::Fixed(1)), EffectRecipient::Controller),
                )
            },
        ))
        .ability(triggered_ability(
            exile,
            TriggerDef {
                // CR 113.6b's statement, printed as the intervening "if" (CR 603.4).
                functions_in: Some(ZoneSet::GRAVEYARD),
                ..whenever(
                    put_into_a_graveyard_from_the_battlefield(ObjectFilter::ByType(CardType::Creature), PlayerRef::Opponent),
                    Effect::Atom(Primitive::Exile, EffectRecipient::ThisObject),
                )
            },
        ))
        .build()
}
