//! Commander Legends (`cmr`), released 2020-11-20.

use std::sync::Arc;

use crate::cards::authoring::{static_ability, static_replacement};
use crate::objects::card_data::{AbilityDef, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{Condition, Effect, ObjectFilter, ObjectSet, PlayerSet};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EnterModsTemplate, EventPattern, GameActionTemplate, ReplacementDef, Rewrite, TemplateAmount};

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

/// Eligeth, Crossroads Augur — {4}{U}{U}
/// Legendary Creature — Sphinx 5/6
///
/// > Flying
/// >
/// > If you would scry a number of cards, draw that many cards instead.
/// >
/// > Partner (You can have two commanders if both have partner.)
///
/// **The kind-changing `Instead` that reads the replaced event's own amount**,
/// and §10's acid test with a producer that is not a draw:
/// `test_opt_with_eligeth_draws_two_and_never_scrys`. Opt under Eligeth draws
/// two and the log holds no `Scried` line at all, which proves two separate
/// things at once — §4.1a's instruction split, since only the *first* of Opt's
/// two instructions is replaced, and `TemplateAmount::ReplacedAmount` reading
/// an amount off a non-draw event.
///
/// **Partner is not dead text under the name.** CR 702.124 makes it a
/// deck-construction ability with no in-game effect, so a card carrying it has
/// nothing to model in a game (`cost-architecture.md`'s Commander track owns
/// deck construction). Flying is a keyword flag.
///
/// **The other printed "would scry" card is Kenessos, Priest of Thassa** —
/// "scry that many cards plus one instead", which is `Rewrite::Amount` over
/// the same pattern rather than an `Instead`, built here and tested against a
/// fixture. The card itself waits on `backlog.md` §2.9: its second ability
/// looks at the top card of the library and acts on what it is. The cards that
/// *watch* a scry — Elrond, Master of Healing, and Goggles of Night, which
/// scries from a trigger rather than replacing one — are critical-path item 6's.
///
/// Its printed rulings are seven and every one of them is about Partner —
/// color identity, two commanders in the command zone, commander damage
/// counted separately (Scryfall, 2026-09-14). None is about the replacement,
/// and none is testable before the Commander track builds the command zone.
pub fn eligeth_crossroads_augur() -> Arc<CardData> {
    CardDataBuilder::new("Eligeth, Crossroads Augur")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 4))
        .color(Color::Blue)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Sphinx))
        .power_toughness(5, 6)
        .keyword_flag(KeywordFlag::Flying)
        .rules_text("Flying\nIf you would scry a number of cards, draw that many cards instead.\nPartner")
        .ability(static_replacement(
            "If you would scry a number of cards, draw that many cards instead.",
            ReplacementDef::new(
                EventPattern::Scry,
                PlayerSet::You,
                Rewrite::Instead(GameActionTemplate::DrawCards {
                    // "that many" — CR 615.5's amount, read off the scry.
                    n: TemplateAmount::ReplacedAmount,
                    // "draw" with no player named: the scrying player is
                    // the one who draws.
                    player: None,
                }),
            ),
        ))
        .build()
}
