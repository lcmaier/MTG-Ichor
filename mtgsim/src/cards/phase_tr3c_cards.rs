//! Phase TR-3c — the reflexive trigger (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-10-09 and is
//! quoted verbatim; each card's rulings were read the same day and sit in
//! its own doc comment (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Cornered Crook | an optional sacrifice, then CR 603.12's "when you do", whose damage its source deals as it last existed if it has left | no |

use std::sync::Arc;

use crate::cards::authoring::{enters, is_sacrificed, triggered_ability, when_you_do, whenever};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, Choice, ChoiceScope, ChoiceSide, Effect, EffectRecipient, ObjectFilter, Pick, PlayerRef, Primitive,
    SelectionFilter, TargetCount,
};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::TriggerSubject;

/// Cornered Crook — {4}{R}
/// Creature — Lizard Warrior 5/4
///
/// > When this creature enters, you may sacrifice an artifact. When you do,
/// > this creature deals 3 damage to any target.
///
/// The enters trigger has no target; "when you do" is a reflexive trigger
/// (CR 603.12), created as the trigger resolves and checked at once against
/// what it performed: a sacrifice by its controller makes it trigger, and its
/// target is chosen as it is put on the stack (CR 603.3d). Its source is the
/// Crook (CR 603.7e), which deals the damage as it last existed if it has
/// left by then (CR 113.7a, 608.2h).
///
/// Registered, not pooled: the performance pool has no "when you do", and
/// its A/B stays IDENTICAL.
///
/// # The ruling, and where it is tested
///
/// In `phase_tr3c_integration_test`.
/// - *"You don't choose a target for Cornered Crook's ability at the time it
///   triggers. Rather, a second "reflexive" ability triggers when you
///   sacrifice an artifact this way. You choose a target for that ability as
///   it goes on the stack. Each player may respond to this triggered ability
///   as normal."* (#1) → `cornered_crooks_target_is_chosen_for_its_second_ability`.
pub fn cornered_crook() -> Arc<CardData> {
    let text = "When this creature enters, you may sacrifice an artifact. When you do, this creature deals 3 damage to any target.";
    // Checked once, against the sacrifice the line before it made; it never
    // waits for another.
    let deals_3 = when_you_do(
        is_sacrificed(ObjectFilter::ByController(PlayerRef::You)),
        Effect::Atom(
            Primitive::DealDamage { amount: AmountExpr::Fixed(3), unpreventable: false },
            EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
        ),
        "When you do, this creature deals 3 damage to any target.",
    );
    CardDataBuilder::new("Cornered Crook")
        .mana_cost(ManaCost::build(&[ManaType::Red], 4))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Lizard))
        .subtype(Subtype::Creature(CreatureType::Warrior))
        .power_toughness(5, 4)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(
                enters(TriggerSubject::ThisObject),
                Effect::Sequence(vec![
                    Effect::Optional {
                        chooser: PlayerRef::You,
                        effect: Box::new(Effect::Atom(
                            Primitive::Sacrifice,
                            EffectRecipient::ChosenBy(Box::new(Choice {
                                chooser: EffectRecipient::Controller,
                                among: ChoiceScope::ChoosersPermanents,
                                picks: vec![Pick::exactly(1, ObjectFilter::ByType(CardType::Artifact))],
                                acts_on: ChoiceSide::Chosen,
                            })),
                        )),
                    },
                    Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(deals_3)), EffectRecipient::Controller),
                ]),
            ),
        ))
        .build()
}
