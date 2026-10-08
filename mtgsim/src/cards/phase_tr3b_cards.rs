//! Phase TR-3b — the returns, and what a delayed trigger refers to
//! (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-10-08 and is
//! quoted verbatim; each card's rulings were read the same day and sit in
//! its own doc comment (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Flickerwisp | a triggered ability's delayed trigger (CR 603.7e) naming the card it exiled (603.7c), `ReturnToBattlefield` | yes |

use std::sync::Arc;

use crate::cards::authoring::{another, at_beginning_of, enters, triggered_ability, whenever, Whose};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{
    Effect, EffectRecipient, ObjectFilter, Primitive, ReturnUnder, SelectionFilter, TargetCount,
};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::{DelayedDuration, DelayedTriggerTemplate, DelayedTurn, TriggerSubject};

/// Flickerwisp — {1}{W}{W}
/// Creature — Elemental 3/1
///
/// > Flying
/// >
/// > When this creature enters, exile another target permanent. Return that
/// > card to the battlefield under its owner's control at the beginning of
/// > the next end step.
///
/// The exile is remembered (`Effect::Remember`), so "that card" is the card
/// in exile, by identity: a token ceases to exist there (CR 704.5d) and the
/// trigger finds nothing, and so does a card that left exile in the
/// meantime (CR 603.7c, 400.7). The delayed trigger is Flickerwisp's and its
/// controller's as the enters trigger resolved (CR 603.7e).
///
/// # The rulings, and where each is tested
///
/// All in `phase_tr3b_integration_test`.
/// - *"The exiled card will return to the battlefield at the beginning of
///   the end step even if Flickerwisp is no longer on the battlefield."*
///   (#1) → `flickerwisps_card_returns_though_flickerwisp_has_left`.
/// - *"If the permanent that returns to the battlefield has any abilities
///   that trigger at the beginning of the end step, those abilities won't
///   trigger that turn."* (#2) → `a_returned_permanents_end_step_ability_waits_for_the_next_end_step`.
/// - *"Auras attached to the exiled permanent will be put into their owners'
///   graveyards. Equipment attached to the exiled permanent will become
///   unattached and remain on the battlefield. Any counters on the exiled
///   permanent will cease to exist. Once the exiled permanent returns, it's
///   considered a new object with no relation to the object that it was."*
///   (#3) → `flickerwisps_card_returns_as_a_new_object`.
/// - *"If a token is exiled this way, it will cease to exist and won't return
///   to the battlefield."* (#4) → `a_token_flickerwisp_exiles_does_not_return`.
pub fn flickerwisp() -> Arc<CardData> {
    let text = "When this creature enters, exile another target permanent. Return that card to the battlefield under its owner's control at the beginning of the next end step.";
    let return_it = DelayedTriggerTemplate {
        def: Arc::new(whenever(
            at_beginning_of(StepType::End, Whose::Each),
            Effect::Atom(Primitive::ReturnToBattlefield(ReturnUnder::Owner), EffectRecipient::Referred),
        )),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::Any,
        rules_text: "Return that card to the battlefield under its owner's control at the beginning of the next end step.".into(),
    };
    CardDataBuilder::new("Flickerwisp")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elemental))
        .power_toughness(3, 1)
        .keyword_flag(KeywordFlag::Flying)
        .rules_text(&format!("Flying
{text}"))
        .ability(triggered_ability(
            text,
            whenever(
                enters(TriggerSubject::ThisObject),
                Effect::Sequence(vec![
                    Effect::Remember(Box::new(Effect::Atom(
                        Primitive::Exile,
                        EffectRecipient::Target(
                            SelectionFilter::Permanent(another(ObjectFilter::All)),
                            TargetCount::Exactly(1),
                        ),
                    ))),
                    Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(return_it)), EffectRecipient::Controller),
                ]),
            ),
        ))
        .build()
}
