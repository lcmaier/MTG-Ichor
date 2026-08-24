//! Creatures whose text is an activated ability.
//!
//! Distinct from `creatures.rs` (vanilla — printed P/T and nothing else) and
//! `keyword_creatures.rs` (CR 702 keywords, which are a `KeywordFlag` rather
//! than an `AbilityDef`). These are the ones that put something on the stack.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    Duration, Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount,
};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};

/// Merfolk Thaumaturgist — {2}{U}
/// Creature — Merfolk Wizard (1/2)
/// `{T}: Switch target creature's power and toughness until end of turn.`
///
/// The pool's only Layer 7d source, and the reason it is a creature rather than
/// the instant that already existed as a fixture: `phase5_pre_cards::inside_out`
/// simplifies a hybrid cost the engine cannot express, so it cannot be
/// registered without lying about the card (see its doc comment).
///
/// It is also the first `AbilityType::Activated` ability in the registry —
/// every other registered ability is a spell, a mana ability, or a static — so
/// it is what gives `activate_ability`'s stack path, its target selection, and
/// its rollback arms their first random-play exposure. The tap cost drags CR
/// 302.6 in with it.
pub fn merfolk_thaumaturgist() -> Arc<CardData> {
    CardDataBuilder::new("Merfolk Thaumaturgist")
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Merfolk))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .color(Color::Blue)
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .power_toughness(1, 2)
        .rules_text("{T}: Switch target creature's power and toughness until end of turn.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::Tap],
            effect: Effect::Atom(
                Primitive::SwitchPowerToughness(Duration::UntilEndOfTurn),
                EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            ),
        })
        .build()
}
