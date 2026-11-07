//! Scars of Mirrodin (`som`), released 2010-10-01.

use std::sync::Arc;

use crate::cards::authoring::{any_other_target, any_target, deal_damage, spell_ability};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::Effect;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};

/// Arc Trail — {1}{R}
/// Sorcery
/// Arc Trail deals 2 damage to any target and 1 damage to any other target.
///
/// (Oracle text verified on Scryfall, 2026-11-07.)
///
/// CR 115.4's "any other target": the second instance is any target but the
/// first's, a player as much as a creature, which an exclusion on a
/// permanent's filter could not say (item 154).
///
/// # The rulings, and where each is tested
///
/// Scryfall lists none for this card (2026-11-07). Cast from hand in
/// `arc_trail_deals_its_two_amounts_to_two_different_targets`.
pub fn arc_trail() -> Arc<CardData> {
    let text = "Arc Trail deals 2 damage to any target and 1 damage to any other target.";
    CardDataBuilder::new("Arc Trail")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(text, Effect::Sequence(vec![deal_damage(any_target(), 2), deal_damage(any_other_target(0), 1)])))
        .build()
}

/// Darksteel Myr — {3}
/// Artifact Creature — Myr
/// 0/1 Indestructible
///
/// **The pool's only indestructible permanent**, added 2026-08-26 because it was
/// the one gap card selection could actually close. `KeywordFlag::Indestructible`
/// is read in two places — SBA 704.5g and `Primitive::Destroy` — and no card in
/// any card file had it, so neither branch was ever taken by `fuzz_games`.
///
/// Three things it exercises at once, which is why this card and not another:
///
/// - **RB item 4 moves that check.** Indestructible stops being a filter inside
///   `Destroy` and becomes a CR 702.12b/614.17 "can't", checked ahead of the
///   replacement pipeline. Fuzz coverage before the move is worth more than after.
/// - **Humility strips it** (all creatures lose all abilities), so the pool now
///   contains a Layer 6 effect that changes an SBA outcome — a live
///   continuous-effect × state-based-action interaction on every fuzz run,
///   against the CR 604.2 existence check.
/// - It is the registry's **second artifact**, which gives March of the Machines
///   a subject other than Sol Ring. (March does not animate it — it is already a
///   creature — which is itself the correct answer to check.)
pub fn darksteel_myr() -> Arc<CardData> {
    CardDataBuilder::new("Darksteel Myr")
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Myr))
        .mana_cost(ManaCost::build(&[], 3))
        .power_toughness(0, 1)
        .keyword_flag(KeywordFlag::Indestructible)
        .rules_text("Indestructible")
        .build()
}
