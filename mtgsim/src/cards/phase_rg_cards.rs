//! Phase RG — the entry state (`replacement-architecture.md` §3.5).
//!
//! One card, for "enters untapped": a status the last applied effect sets,
//! with CR 616.1's order between opposite statuses (D5, D6). Its oracle text
//! and its rulings were read on Scryfall on 2026-09-28.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{Condition, Effect, ObjectFilter, ObjectSet};
use crate::types::ids::AbilityId;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EnterModsTemplate, EventPattern, ReplacementDef, Rewrite};

fn static_ability(rules_text: &'static str, effect: Effect) -> AbilityDef {
    AbilityDef {
        rules_text,
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect,
    }
}

/// "Other permanents enter [with a status]" while its source is in the state
/// `condition` names. Over every permanent, because an entering Archelos is
/// never one of its own: source 1a admits only `ObjectSet::SourceOnly`
/// (CR 614.12's parenthesis), as Master Biomancer's "other" needs no leaf.
fn others_enter(rules_text: &'static str, condition: Condition, template: EnterModsTemplate) -> AbilityDef {
    static_ability(rules_text, Effect::Conditional(
        condition,
        Box::new(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::battlefield_filter(ObjectFilter::All),
            Rewrite::EnterWith(template),
        )))),
    ))
}

/// Archelos, Lagoon Mystic — {1}{B}{G}{U}
/// Legendary Creature — Turtle Shaman, 2/4
///
/// Two conditional static replacements, one per status. Each condition is
/// asked at the proposal against the board as it stands (CR 614.4), so the
/// status Archelos has when a permanent would enter decides which applies.
///
/// # The rulings, and where each is tested (`phase_rg_integration_test.rs`)
///
/// - **#1**: neither ability applies to Archelos's own entry, nor to anything
///   entering at the same time as it. Its own: source 1a's scope. At the same
///   time: the one-board decision, since it is not on the battlefield while
///   the batch it enters with is decided.
/// - **#2**: beside another effect that says a permanent enters tapped, an
///   untapped Archelos makes it the entering permanent's controller's choice
///   (CR 616.1, the last status applied wins); and a permanent an instruction
///   puts onto the battlefield tapped, with no replacement effect, enters
///   untapped under it, since the instruction's word is the proposal's seed.
/// - **#3**: with more than one Archelos, the entering permanent's controller
///   orders them.
///
/// # In `PERFORMANCE_POOL`
///
/// "Enters untapped" and CR 616.1's order between opposite statuses are the
/// engine path RG opens; beside the pooled Idyllic Beachfront and Root Maze a
/// random game reaches that prompt. With `Everywhere` in the pool a
/// `{1}{B}{G}{U}` card is castable in every deck.
pub fn archelos_lagoon_mystic() -> Arc<CardData> {
    CardDataBuilder::new("Archelos, Lagoon Mystic")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Green, ManaType::Blue], 1))
        .color(Color::Black)
        .color(Color::Green)
        .color(Color::Blue)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Turtle))
        .subtype(Subtype::Creature(CreatureType::Shaman))
        .power_toughness(2, 4)
        .rules_text(
            "As long as Archelos is tapped, other permanents enter tapped.\n\
             As long as Archelos is untapped, other permanents enter untapped.",
        )
        .ability(others_enter("As long as Archelos is tapped, other permanents enter tapped.", Condition::SourceTapped, EnterModsTemplate::tapped()))
        .ability(others_enter("As long as Archelos is untapped, other permanents enter untapped.", Condition::SourceUntapped, EnterModsTemplate::untapped()))
        .build()
}
