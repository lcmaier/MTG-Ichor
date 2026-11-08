//! Outlaws of Thunder Junction (`otj`), released 2024-04-19.

use std::sync::Arc;

use crate::cards::authoring::{artifact, deal_damage, destroy, spell_ability, spree, target_creature, target_permanent};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::mana::{ManaCost, ManaType};

/// Explosive Derailment — {R}
/// Instant
/// Spree (Choose one or more additional costs.)
/// + {2} — Explosive Derailment deals 4 damage to target creature.
/// + {2} — Destroy target artifact.
///
/// (Oracle text verified on Scryfall, 2026-11-07.)
///
/// CR 702.172a's spree: "choose one or more modes", each paid for as an
/// additional cost when chosen (CR 700.2h), Abrade's two modes behind {2}
/// each. The modes are asked before the mana is known, and the spell is
/// offered while some set of its modes is payable. Spree is the modal node
/// that writes it, a count of one or more and a cost a mode; nothing reads a
/// spree keyword yet, which a card asking "a spell with spree" would, with
/// `breadth-features`.
///
/// # The rulings, and where each is tested
///
/// - #1 and #2, the + signs mean nothing: no board.
/// - #3, at least one mode, its cost paid, and #4, the modes chosen as it is
///   cast: `explosive_derailment_needs_a_mode_and_pays_each_chosen_modes_cost`.
/// - #5, a mode with a target only while it has one, and #7, no mode twice:
///   `explosive_derailment_offers_only_the_modes_with_a_legal_target`.
/// - #6, the instructions in printed order, and #9, nothing between them:
///   `explosive_derailment_resolves_its_modes_in_printed_order`.
/// - #8, its mana value is its mana cost's:
///   `explosive_derailment_has_mana_value_one_whatever_was_paid`.
/// - #10, a copy keeps its modes: not yet expressible, a copy of a spell
///   (CR 707.10) is `spell-copies`'.
/// - #11, the spell resolves while one target is legal:
///   `explosive_derailment_resolves_against_whichever_target_is_left`.
/// - #12, cast without paying its mana cost, a mode is still chosen and paid
///   for: not yet expressible, a free cast is `casting-from-other-zones`'.
pub fn explosive_derailment() -> Arc<CardData> {
    let text = "Spree\n+ {2} — Explosive Derailment deals 4 damage to target creature.\n+ {2} — Destroy target artifact.";
    let two = || vec![Cost::Mana(ManaCost::build(&[], 2))];
    CardDataBuilder::new("Explosive Derailment")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, spree([(two(), deal_damage(target_creature(), 4)), (two(), destroy(target_permanent(artifact())))])))
        .build()
}
