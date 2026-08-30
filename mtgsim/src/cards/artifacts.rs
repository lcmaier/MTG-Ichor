//! Artifacts.
//!
//! The crate had **no artifact card at all** outside the `phase_l*` fixtures
//! until this module, and that absence was load-bearing in the wrong direction:
//! March of the Machines ("Each noncreature artifact is an artifact creature
//! with power and toughness each equal to its mana value") is registered, draws
//! into fuzz decks, and had nothing to animate — so Layer 7b, the sublayer it
//! exists to exercise, had zero random-play coverage. Registering an artifact
//! is what turns that card on. Same shape as `dual_lands.rs`, which was written
//! to give Blood Moon a mana base to hit.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::keywords::KeywordFlag;
use crate::types::costs::Cost;
use crate::types::effects::{AmountExpr, Effect, EffectRecipient, ManaOutput, Primitive};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};

/// Sol Ring — {1}
/// Artifact
/// `{T}: Add {C}{C}.`
///
/// Verbatim, and chosen for three reasons beyond being the most-played card in
/// Commander: it is colorless, so `random_deck` puts it in *every* deck rather
/// than only the ones sharing its colors; its mana value is 1, so under March
/// of the Machines it animates into a 1/1 that survives its own SBA check; and
/// the animated form is a creature with a mana ability, which puts CR 302.6 on
/// a permanent that was not a creature when the turn began.
pub fn sol_ring() -> Arc<CardData> {
    CardDataBuilder::new("Sol Ring")
        .card_type(CardType::Artifact)
        .mana_cost(ManaCost::build(&[], 1))
        .rules_text("{T}: Add {C}{C}.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Mana,
            costs: vec![Cost::Tap],
            effect: Effect::Atom(
                Primitive::ProduceMana(ManaOutput {
                    // One ability producing two, not two abilities producing one:
                    // `{C}{C}` is a single activation, and a mana ability that
                    // could be activated twice per tap would be a different card.
                    mana: vec![(ManaType::Colorless, AmountExpr::Fixed(2))],
                    special: vec![],
                }),
                EffectRecipient::Implicit,
            ),
        })
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
///   against the CR 613.7a existence check.
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
        .keyword(KeywordFlag::Indestructible)
        .build()
}
