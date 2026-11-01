//! Dark Ascension (`dka`), released 2012-02-03.

use std::sync::Arc;

use crate::cards::authoring::{attacked_by, attacks, creature, deal_damage, non, triggered_ability, undying, whenever, you_control};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::cost_modification::{CostChange, CostModificationDef};
use crate::types::effects::Antecedent;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::EventRole;

/// Hellrider — {2}{R}{R}
/// Creature — Devil 3/3
///
/// > Haste
/// > Whenever a creature you control attacks, this creature deals 1 damage
/// > to the player or planeswalker it's attacking.
///
/// "It" is the creature that attacked, the trigger's attacker, and what it is
/// attacking is read as the ability resolves: nothing once it has left combat
/// and stayed on the battlefield, and what it was attacking once it has left
/// (CR 608.2h; Fathom Fleet Swordjack's ruling on the same words).
///
/// It exercises an attack trigger for each creature its controller attacks
/// with, and damage to what that creature is attacking (CR 506.3, 508.3a).
///
/// # The ruling, and where it is tested
///
/// In `phase_tr5e_integration_test`.
/// - *"Creatures you control may attack multiple players and/or
///   planeswalkers. For each attacking creature, Hellrider will deal damage to
///   the corresponding player or planeswalker."* (#1) →
///   `hellrider_deals_each_attackers_damage_to_what_it_attacks`, at four
///   seats. Nothing attacks a planeswalker until `backlog.md` §2.23's attack
///   path, so the test's creatures attack players.
pub fn hellrider() -> Arc<CardData> {
    let text = "Whenever a creature you control attacks, this creature deals 1 damage to the player or planeswalker it's attacking.";
    let creature_you_control = creature().and(you_control());
    let one_damage = deal_damage(attacked_by(Antecedent::Triggering(EventRole::Attacker)), 1);
    CardDataBuilder::new("Hellrider")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Devil))
        .power_toughness(3, 3)
        .keyword_flag(KeywordFlag::Haste)
        .rules_text(&format!("Haste\n{text}"))
        .ability(triggered_ability(text, whenever(attacks(creature_you_control), one_damage)))
        .build()
}

/// Strangleroot Geist — {G}{G}
/// Creature — Spirit 2/1
///
/// > Haste
/// > Undying (When this creature dies, if it had no +1/+1 counters on it,
/// > return it to the battlefield under its owner's control with a +1/+1
/// > counter on it.)
///
/// Undying's "if it had" is a fact about the permanent as it left (CR
/// 603.10a), read off the counters its last known information kept, at the
/// trigger and again at resolution (CR 603.4). "It" is the card in the
/// graveyard (CR 400.7e), which returns as a new object (CR 400.7) with its
/// +1/+1 counter as part of the entry (CR 122.6).
///
/// It exercises undying: an intervening "if" about the counters it had, read
/// off its last known information at both instants, and a return with a
/// counter.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The first pooled ability whose intervening "if" reads the triggering
/// object's last known information, and the first return with counters: a new
/// engine path, which §3.4 measures rather than assumes. It returns once, so
/// it lengthens a stress game by a creature rather than a loop (the
/// distortion `fuzz-record.md`'s TR-4a block measured).
///
/// # The rulings
///
/// Scryfall lists none. Kitchen Finks' seven persist rulings are tested as
/// undying's mirror in `phase_tr4a_integration_test`, each on both keywords.
pub fn strangleroot_geist() -> Arc<CardData> {
    CardDataBuilder::new("Strangleroot Geist")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Spirit))
        .power_toughness(2, 1)
        .rules_text("Haste\nUndying")
        .keyword_flag(KeywordFlag::Haste)
        .ability(undying())
        .build()
}

/// Thalia, Guardian of Thraben — {1}{W}
/// Legendary Creature — Human Soldier, 2/1
///
/// First strike
/// Noncreature spells cost {1} more to cast.
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// "Each spell that's not a creature spell, including your own" (her ruling
/// of 2021-11-19) — no controller clause, so the filter is the type alone.
/// The increase is CR 601.2f's first position, applied before any reduction;
/// her second ruling restates the order in full.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The pool's first cost effect, so the first card that populates
/// `cost_modification_ability_sources` and makes CR 601.2f's sweep run in a
/// measured game — on both players' spells, since she taxes everyone. A
/// two-drop any white deck casts early; with Humility already pooled, the
/// board where CR 613.11 reads a *stripped* Thalia happens in a measured
/// game too.
pub fn thalia_guardian_of_thraben() -> Arc<CardData> {
    CardDataBuilder::new("Thalia, Guardian of Thraben")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Soldier))
        .power_toughness(2, 1)
        .keyword_flag(KeywordFlag::FirstStrike)
        .rules_text("First strike\nNoncreature spells cost {1} more to cast.")
        .ability(
            CostModificationDef::spells(
                non(creature()),
                CostChange::Increase(ManaCost::build(&[], 1)),
            )
            .into_ability("Noncreature spells cost {1} more to cast."),
        )
        .build()
}
