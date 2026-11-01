//! Dominaria United (`dmu`), released 2022-09-09.

use std::sync::Arc;

use crate::cards::authoring::static_replacement;
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, LandType, Subtype};
use crate::types::effects::ObjectSet;
use crate::types::mana::ManaType;
use crate::types::replacement::{EnterModsTemplate, EventPattern, ReplacementDef, Rewrite};

/// Idyllic Beachfront — Land — Plains Island
///
/// ({T}: Add {W} or {U}.)
/// This land enters tapped.
///
/// (Oracle text verified on Scryfall, 2026-09-01. Dominaria United; the
/// parenthesis is the whole of the mana half, because CR 305.6 makes it
/// intrinsic to the land types.)
///
/// # Why this one out of 773
///
/// CR 110.5b's shape — "permanents enter the battlefield untapped … unless a
/// spell or ability says otherwise" — is the most-printed replacement effect in
/// Magic, and nearly every card carrying it carries something else too: an ETB
/// trigger, a sacrifice ability, cycling, a storage counter. This cycle is the
/// clause and nothing else, and its mana is the dual lands' shape exactly —
/// two basic land types, two intrinsic abilities, and the same modeling
/// shortcut documented there. So the card adds one rules line and no machinery.
///
/// **It is also a land, and that is the point of picking a land.** A land drop
/// is the highest-frequency entry in the game and it reaches the battlefield by
/// a different road than a resolving permanent spell — `play_land` rather than
/// `resolve_top_of_stack`, with `GameState::resolving` empty, so CR 110.2b's
/// default controller comes from the owner rather than from the stack entry.
///
/// **`ObjectSet::SourceOnly`, which is CR 614.12's first sentence** — "such
/// effects may come from the permanent itself if they affect only that
/// permanent (as opposed to a general subset of permanents that includes it)".
/// That is what lets RC-2 ship without the look-ahead frame: the effect is found
/// on the entering object, applies to the entering object, and asks nothing
/// about the board.
///
/// # The interaction that makes it worth registering rather than fixturing —
/// and the one it exposes
///
/// Blood Moon is in `PERFORMANCE_POOL`, and CR 305.7 clears a nonbasic land's
/// abilities wholesale, "enters tapped" included. The real ruling is that a
/// tapland under Blood Moon enters **untapped**, and `gather` reading the
/// *effective* ability list is meant to deliver that for free
/// (`replacement-architecture.md` §3.3).
///
/// **It does not, and this card is what found out.** Blood Moon's row is a
/// `ContinuousEffect` whose `ObjectSet` is a `Filter`, and
/// the layer gate (then `compute.rs::effect_applies_to`, now
/// `board.rs::Board::in_zones_or_entering`) returns `false` for a filter effect against
/// an object that is not on the battlefield — so the *layer registry* reaches
/// no entering permanent. That gate is Phase **RC-3**'s one line, and until it
/// moves the strip is real everywhere except at the instant it matters here.
///
/// **The gate is not "no filter reaches an entry", which is what RC-2 wrote
/// (corrected 2026-09-02).** `ReplacementDef.affected`'s object half is matched by
/// `gather::set_affects` through a different and ungated function, so
/// `root_maze` taps this land on the way in without any help from RC-3.
pub fn idyllic_beachfront() -> Arc<CardData> {
    CardDataBuilder::new("Idyllic Beachfront")
        .card_type(CardType::Land)
        .subtype(Subtype::Land(LandType::Plains))
        .subtype(Subtype::Land(LandType::Island))
        .rules_text("This land enters tapped.")
        .ability(static_replacement(
            "This land enters tapped.",
            ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                ObjectSet::SourceOnly,
                Rewrite::EnterWith(EnterModsTemplate::tapped()),
            ),
        ))
        // CR 305.6 — intrinsic to the land types, modeled as two explicit mana
        // abilities for the dual lands' documented reason.
        .mana_ability_single(ManaType::White)
        .mana_ability_single(ManaType::Blue)
        .build()
}
