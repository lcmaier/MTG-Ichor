//! Phase TR-4b — attachment's records (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-10-14 and is
//! quoted verbatim; each card's rulings were read the same day and sit in its
//! own doc comment (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Grafted Wargear | "becomes unattached" on every route CR 701.3d counts, looking back (CR 603.10c), and "that permanent" as the existence it was | no |
//! | Bramble Elemental | "becomes attached", an Aura entering attached included (CR 303.4f) | yes |

use std::sync::Arc;

use crate::cards::authoring::{becomes_attached, becomes_unattached, triggered_ability, whenever};
use crate::cards::phase_lh_cards::equip;
use crate::cards::phase_tr1_cards::saproling_token;
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use crate::types::card_types::{ArtifactType, CardType, CreatureType, EnchantmentType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, Primitive};
use crate::types::ids::AbilityId;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{Participant, TriggerSubject};

/// Grafted Wargear — {3}
/// Artifact — Equipment
///
/// > Equipped creature gets +3/+2.
/// > Whenever this Equipment becomes unattached from a permanent, sacrifice
/// > that permanent.
/// > Equip {0} ({0}: Attach to target creature you control. Equip only as a
/// > sorcery.)
///
/// "Becomes unattached" is one record on every route (CR 701.3d), and it
/// looks back (CR 603.10c), so the Wargear's own trigger fires when it is
/// the one that left. "That permanent" is the object it was attached to, as
/// the existence it was (CR 400.7): a creature that has left is not
/// sacrificed, nor the new object it became.
///
/// # Not pooled, and why
///
/// Equip {0} at sorcery speed is offered to the random agent every main
/// phase, and every move to another creature sacrifices the one it left:
/// forced into every `stress` deck, 200 games at seed 12345 and two seats,
/// it moved 262 times, and the average game ran 26.9 turns against 25.4
/// (`fuzz-record.md`, the TR-4b block). It ends no game, so it is registered
/// for `stress` to hunt with, and Bramble Elemental, pooled, carries
/// "becomes attached" beside it.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr4b_integration_test`.
/// - *"Grafted Wargear becomes unattached from the creature it's equipping if
///   you equip it to a new creature, if Grafted Wargear leaves the
///   battlefield, if the equipped creature ceases to be a creature, or if
///   Grafted Wargear ceases to be an Equipment. (It also becomes unattached
///   if the equipped creature leaves the battlefield, but the triggered
///   ability won't do anything in that case.)"* (#1) → one test per route.
/// - *"You may activate this card's equip ability targeting the creature
///   that it is currently attached to, even if it's the only creature you
///   control. ..."* (#2) →
///   `grafted_wargear_equipped_again_to_its_creature_sacrifices_nothing`.
/// - *"If your Grafted Wargear somehow ends up on an opponent's creature and
///   it then becomes unattached from that creature, you won't be able to
///   sacrifice that creature because you don't control it."* (#3) →
///   `grafted_wargear_does_not_sacrifice_an_opponents_creature`.
pub fn grafted_wargear() -> Arc<CardData> {
    let text = "Whenever this Equipment becomes unattached from a permanent, sacrifice that permanent.";
    CardDataBuilder::new("Grafted Wargear")
        .mana_cost(ManaCost::build(&[], 3))
        .card_type(CardType::Artifact)
        .subtype(Subtype::Artifact(ArtifactType::Equipment))
        .rules_text("Equipped creature gets +3/+2.\nWhenever this Equipment becomes unattached from a permanent, sacrifice that permanent.\nEquip {0}")
        .ability(AbilityDef {
            rules_text: "Equipped creature gets +3/+2.".into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ModifyPowerToughness(AmountExpr::Fixed(3), AmountExpr::Fixed(2), Duration::WhileSourceOnBattlefield),
                EffectRecipient::Host,
            ),
        })
        .ability(triggered_ability(
            text,
            whenever(
                becomes_unattached(TriggerSubject::ThisObject, TriggerSubject::Any),
                Effect::Atom(Primitive::Sacrifice, EffectRecipient::Triggering(Participant::Host)),
            ),
        ))
        .ability(equip("Equip {0}", Vec::new()))
        .build()
}

/// Bramble Elemental — {3}{G}{G}
/// Creature — Elemental 4/4
///
/// > Whenever an Aura becomes attached to this creature, create two 1/1
/// > green Saproling creature tokens.
///
/// "Becomes attached" on the transition only (CR 603.2e): an Aura spell
/// resolving onto it and one entering attached by any other means each
/// trigger it (CR 303.4f), as does one moved onto it from another object,
/// and an attach to it while the Aura is already there does not. Brood
/// Keeper's first ruling says so of the same text.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The benign consumer of the same records: it harms nobody a random agent
/// plays it against, so it carries the phase's new path into the
/// `performance` tables, which TR-4b re-recorded with it. A random Aura
/// seldom lands on it; what it measures is the dispatch's cost, since while
/// it is on the battlefield every attach a pooled Aura or Equipment makes is
/// matched against it.
///
/// # The rulings
///
/// Scryfall lists none.
pub fn bramble_elemental() -> Arc<CardData> {
    let text = "Whenever an Aura becomes attached to this creature, create two 1/1 green Saproling creature tokens.";
    CardDataBuilder::new("Bramble Elemental")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elemental))
        .power_toughness(4, 4)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(
                becomes_attached(
                    ObjectFilter::BySubtype(Subtype::Enchantment(EnchantmentType::Aura)),
                    TriggerSubject::ThisObject,
                ),
                Effect::Atom(Primitive::CreateToken(saproling_token(), AmountExpr::Fixed(2)), EffectRecipient::Controller),
            ),
        ))
        .build()
}
