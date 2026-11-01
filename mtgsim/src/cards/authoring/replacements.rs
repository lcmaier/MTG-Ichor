//! CR 614's replacement effects as the words a card writes inside
//! [`super::static_replacement`] or a resolution's
//! [`super::create_replacement`]: the event a card's text watches, who it
//! affects, and what happens instead, each a phrase more than one card
//! prints. In the CR's order: the events (CR 106.12, 609.7a), the scope
//! (CR 614.1), then the kinds of replacement by their rules (CR 614.9 to
//! 614.11), then a whole static ability.

use super::abilities::static_ability;
use super::instructions::shuffle_library;
use super::recipients::this_object;
use crate::objects::card_data::AbilityDef;
use crate::types::effects::{AffectedSet, Effect, ObjectFilter, ObjectSet, PlayerSet};
use crate::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, RetargetSpec, Rewrite, SourcePattern, TemplateAmount};
use crate::types::zones::{DrawFilter, Zone, ZoneChangePattern, ZoneSet};

/// CR 106.12's "tapped for mana", optionally of a permanent matching `source`.
pub fn tapped_for_mana(source: Option<ObjectFilter>) -> EventPattern {
    EventPattern::ProduceMana { tapped_for_mana: Some(true), source }
}

/// "The next time a source of your choice would deal damage to you this turn"
/// — the def the three CR 609.7a cards share, differing only in the property
/// the chosen source must still have and in what the effect then does.
///
/// The `SourcePattern`'s `object` is `None` here and stays `None` on the card:
/// CR 609.7a chooses the source *when the effect is created*, so the
/// resolution writes it (`PatternFill::ChosenDamageSource`). The `filter` is
/// the card's own and is rechecked at every damage event, which is CR 609.7b.
pub fn the_next_damage_from_a_chosen_source(
    filter: Option<ObjectFilter>,
    rewrite: Rewrite,
) -> ReplacementDef {
    let source = match filter {
        Some(f) => SourcePattern::matching(f),
        None => SourcePattern::chosen(),
    };
    ReplacementDef::new(
        EventPattern::DealDamage { source: Some(source), combat: None },
        PlayerSet::You,
        rewrite,
    )
    .once()
}

/// Every permanent and every player: CR 614.1's two kinds of subject, the
/// affected set a card means when it prints "a permanent or player" (any
/// one of them, so all of both).
///
/// Furnace of Rath and Ghosts of the Innocent print the same scope word for
/// word, so they share this rather than each spelling it: a divergence between
/// two cards whose oracle text is identical would be an authoring bug nothing
/// could catch.
pub fn every_permanent_and_player() -> AffectedSet {
    AffectedSet { objects: ObjectSet::battlefield_filter(ObjectFilter::All), players: PlayerSet::Everyone }
}

/// "All damage that would be dealt to you … is dealt to [something] instead" —
/// the pattern and the two sets Pariah and Palisade Giant share.
///
/// `combat: None` is the whole of "all damage": CR 614.9 is indifferent to
/// where the damage came from, and both cards' first ruling is that redirected
/// combat damage is still combat damage — which is a fact about the *rewrite*
/// (it does not touch `is_combat`) rather than about the pattern.
pub fn all_damage_to_you(objects: ObjectSet, spec: RetargetSpec) -> ReplacementDef {
    ReplacementDef::new(
        EventPattern::DealDamage { source: None, combat: None },
        AffectedSet { objects, players: PlayerSet::You },
        Rewrite::Retarget(spec),
    )
}

/// "Skip [unit]" as CR 614.10 defines it: *"instead of doing [something], do
/// nothing"*, which CR 614.1b says is a replacement effect and CR 614.6 says is
/// a [`Rewrite::Prevent`].
///
/// No new `Rewrite` arm, for that reason — "replaced with nothing" **is** 614.6,
/// and a `Rewrite::Skip` would be a second spelling of one algebra element.
///
/// The object set is empty on every one of these: CR 614.10's three units are
/// about a *player*, so the scope is a [`PlayerSet`] and nothing else.
pub fn skip(pattern: EventPattern, players: PlayerSet) -> ReplacementDef {
    ReplacementDef::new(pattern, players, Rewrite::Prevent)
}

/// "If you would draw a card, draw two cards instead" — the shape Thought
/// Reflection and Teferi's Ageless Insight share, differing only in the
/// [`DrawFilter`] they ask for.
///
/// [`GameActionTemplate::DrawCards`]'s `player` is `None` on both: the draw
/// stays with the player who would have drawn it. Notion Thief is the arm's
/// other customer and the one that moves it.
pub fn draw_two_instead(place: Option<DrawFilter>) -> ReplacementDef {
    ReplacementDef::new(
        EventPattern::DrawCard { place },
        PlayerSet::You,
        Rewrite::Instead(GameActionTemplate::DrawCards {
            n: TemplateAmount::Fixed(2),
            player: None,
        }),
    )
}

/// "If [this] would be put into a graveyard from anywhere, reveal [this] and
/// shuffle it into its owner's library instead." — one clause, and each of
/// its phrases is a different rule:
///
/// - **"from anywhere"** is CR 113.6b's statement of where the ability
///   functions, `Effect::FunctionsIn(ZoneSet::ALL, ..)` as Wonder states its
///   graveyard. `zone_function::functioning_zones` reads it to file the card
///   as a gather candidate in every zone it enters. It is also the pattern's
///   `from: None`.
/// - **"would be put into a graveyard"** is any zone change into a graveyard,
///   whatever the cause: destroyed, sacrificed, discarded, milled, countered,
///   resolved.
/// - **"shuffle it into its owner's library instead"** is a substitute event
///   plus the rest of the instruction: `ZoneChangeTo { Library }` moves it,
///   and the shuffle is the `then`, resolved after the move (CR 615.5's
///   shape). `Primitive::ShuffleLibrary` with an `Implicit` recipient
///   is "its owner's library" — the source's owner — and moves nothing, which
///   is CR 701.24c: a commander's CR 903.9b can send the card to the command
///   zone *instead* of the library, and its owner's library is shuffled all
///   the same while the card stays where 903.9b put it.
/// - **"reveal"** is a disclosure to every player, which the information
///   model owns (`information-model`); until a viewer can be told, it is the
///   same no-op Nephalia Academy's reveal is, and both are that phase's
///   tests.
///
/// The affected set is the card itself, so on any board the family produces
/// at most one candidate per event and CR 616.1 never asks; the ability is
/// mandatory besides.
pub fn shuffles_into_library_from_anywhere(rules_text: &'static str) -> AbilityDef {
    static_ability(rules_text, Effect::FunctionsIn(
        // CR 113.6b — the clause that says where this ability functions.
        ZoneSet::ALL,
        Box::new(Effect::Replacement(Box::new(
            ReplacementDef::new(
                EventPattern::ZoneChange {
                    change: ZoneChangePattern { from: None, to: Some(Zone::Graveyard), act: None },
                    object: None,
                },
                ObjectSet::SourceOnly,
                Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Library }),
            )
            .with_then(shuffle_library(this_object())),
        ))),
    ))
}
