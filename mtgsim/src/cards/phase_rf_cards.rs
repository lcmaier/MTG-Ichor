//! Phase RF — the gather's zone leg: static replacement abilities that
//! function off the battlefield (`replacement-architecture.md` §9, Phase RF;
//! §3.3's source 2).
//!
//! Two registered cards and three fixtures. Both oracle texts were verified on
//! Scryfall on 2026-09-16 and are quoted verbatim.
//!
//! **Why Darksteel Colossus and Nexus of Fate, and not Blightsteel Colossus.**
//! The owner named Blightsteel for this leg. Its clause is the one below plus
//! infect, and infect (CR 702.90) is not a keyword the engine has, so a
//! registered Blightsteel would be a card wearing a real name while behaving
//! differently — the thing `engineering-practices.md` §3 forbids a fixture
//! and forbids a card twice over. Darksteel Colossus prints the same clause
//! with only trample and indestructible, both of which the engine plays.
//! Nexus of Fate is the second shape §3.3 asks a mechanic for: an instant,
//! never a permanent, whose "from anywhere" is the **stack** — resolving
//! (CR 608.2n) and being countered (CR 701.6) both put it into a graveyard,
//! and both are replaced. The other two of the printed five want something
//! else first: Progenitus protection from everything, Legacy Weapon a
//! five-color activated exile.
//!
//! **Read the board, not the card.** A Colossus dying off the battlefield was
//! already found by the battlefield sweep; what this phase builds is the leg
//! that finds it in a library (milled), in a hand (discarded) and on the
//! stack (countered), and the tests are on those boards.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{
    Condition, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, Primitive,
};
use crate::types::ids::AbilityId;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use crate::types::zones::{Zone, ZoneChangeCause, ZoneSet};

/// `codebase-state.md` item 120's constructor, the copy this file owes it.
fn static_ability(effect: Effect) -> AbilityDef {
    AbilityDef {
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect,
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
    }
}

fn spell_ability(effect: Effect) -> AbilityDef {
    AbilityDef {
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Spell,
        costs: Vec::new(),
        effect,
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
    }
}

/// "If [this] would be put into a graveyard from anywhere, reveal [this] and
/// shuffle it into its owner's library instead." — one clause, and each of
/// its phrases is a different rule:
///
/// - **"from anywhere"** is CR 113.6b's statement of where the ability
///   functions, written as the condition `SourceInZone(ZoneSet::ALL)` the way
///   Wonder writes its graveyard (`layers-architecture.md` §13d decision 1b).
///   `zone_function::functioning_zones` reads it syntactically to file the
///   card as a gather candidate in every zone it enters, and the gather reads
///   it again at each proposal, where it always holds. It is also the
///   pattern's `from: None`.
/// - **"would be put into a graveyard"** is any zone change into a graveyard,
///   whatever the cause: destroyed, sacrificed, discarded, milled, countered,
///   resolved.
/// - **"shuffle it into its owner's library instead"** is a substitute event
///   plus the rest of the instruction: `ZoneChangeTo { Library }` moves it,
///   and the shuffle is the `then`, resolved after the move (CR 615.5's
///   shape, §4.1a). `Primitive::ShuffleLibrary` with an `Implicit` recipient
///   is "its owner's library" — the source's owner — and moves nothing, which
///   is CR 701.24c: a commander's CR 903.9b can send the card to the command
///   zone *instead* of the library, and its owner's library is shuffled all
///   the same while the card stays where 903.9b put it.
/// - **"reveal"** is a disclosure, and the engine has no per-viewer
///   visibility to disclose against (`backlog.md` §2.9) — the same no-op
///   Nephalia Academy's reveal is. When §2.9 lands, both are its tests.
///
/// The affected set is the card itself, so on any board the family produces
/// at most one candidate per event and CR 616.1 never asks; the ability is
/// mandatory besides.
fn shuffles_into_library_from_anywhere() -> AbilityDef {
    static_ability(Effect::Conditional(
        // CR 113.6b — the clause that says where this ability functions.
        Condition::SourceInZone(ZoneSet::ALL),
        Box::new(Effect::Replacement(Box::new(
            ReplacementDef::new(
                EventPattern::ZoneChange {
                    from: None,
                    to: Some(Zone::Graveyard),
                    cause: None,
                    object: None,
                },
                ObjectSet::SourceOnly,
                Rewrite::Instead(GameActionTemplate::ZoneChangeTo {
                    to: Zone::Library,
                    cause: ZoneChangeCause::PutIntoLibrary,
                }),
            )
            .with_then(Effect::Atom(Primitive::ShuffleLibrary, EffectRecipient::Implicit)),
        ))),
    ))
}

/// Darksteel Colossus — {11}
/// Artifact Creature — Golem, 11/11
/// "Trample
///  Indestructible
///  If Darksteel Colossus would be put into a graveyard from anywhere, reveal
///  Darksteel Colossus and shuffle it into its owner's library instead."
///
/// The leg's first consumer, and pooled: at eleven mana it is almost never
/// cast, so what it puts in front of every measured game is a card in a
/// library or a hand that the zone leg reads on every gather — the cost this
/// phase opens, and the one its A/B is read for.
///
/// Its own ruling (Scryfall, 2013-07-01) is the board the battlefield half is
/// tested on: indestructible stops destruction, "however, a creature with
/// indestructible can be put into the graveyard for a number of reasons …
/// sacrificed or if its toughness is 0 or less. (In these cases, of course,
/// Darksteel Colossus would be shuffled into its owner's library instead.)"
pub fn darksteel_colossus() -> Arc<CardData> {
    CardDataBuilder::new("Darksteel Colossus")
        .mana_cost(ManaCost::build(&[], 11))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Golem))
        .power_toughness(11, 11)
        .keyword_flag(KeywordFlag::Trample)
        .keyword_flag(KeywordFlag::Indestructible)
        .rules_text(
            "Trample\nIndestructible\nIf Darksteel Colossus would be put into a graveyard \
             from anywhere, reveal Darksteel Colossus and shuffle it into its owner's \
             library instead.",
        )
        .ability(shuffles_into_library_from_anywhere())
        .build()
}

/// Nexus of Fate — {5}{U}{U}
/// Instant
/// "Take an extra turn after this one.
///  If Nexus of Fate would be put into a graveyard from anywhere, reveal Nexus
///  of Fate and shuffle it into its owner's library instead."
///
/// The card that is never on the battlefield, which is the sentence that
/// makes it the right second shape: its static ability functions on the
/// stack by CR 113.6b's statement, and CR 113.6's first sentence would have
/// put an instant's abilities there anyway. Resolving is the famous case —
/// CR 608.2n's move to the graveyard is replaced and the card goes back into
/// the library it will be drawn from again — and being countered (CR 701.6)
/// or discarded is the same clause from a different zone.
///
/// **Registered, not pooled**, for the reason Time Walk is not: an extra
/// turn moves `Avg turns/game` by design, which is a worse baseline rather
/// than a wider one. `Primitive::ExtraTurn` is that card's, unchanged.
///
/// "Reveal Nexus of Fate" is not modeled: the engine has no per-viewer
/// visibility to reveal against (`backlog.md` §2.9), so the reveal is the
/// same no-op it is on Darksteel Colossus and Nephalia Academy, and this
/// card is one of §2.9's tests when it lands.
pub fn nexus_of_fate() -> Arc<CardData> {
    CardDataBuilder::new("Nexus of Fate")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 5))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(
            "Take an extra turn after this one.\nIf Nexus of Fate would be put into a \
             graveyard from anywhere, reveal Nexus of Fate and shuffle it into its owner's \
             library instead.",
        )
        .ability(spell_ability(Effect::Atom(
            Primitive::ExtraTurn,
            EffectRecipient::Controller,
        )))
        .ability(shuffles_into_library_from_anywhere())
        .build()
}

/// **Fixture.** "If this creature would be put into a graveyard, exile it
/// instead." — the same pattern as the Colossus's with **no zone statement**.
///
/// CR 113.6's first sentence: an ability that states no zone functions only
/// on the battlefield, so this one exiles the Golem when it is sacrificed and
/// does nothing when it is discarded from a hand or milled from a library.
/// The negative half of the leg — the "only" that makes CR 113.6b a rule —
/// and the test that `zone_function::functions_in` is asked of every source
/// the zone leg reads, not only that the set was filed correctly.
pub fn timid_golem() -> Arc<CardData> {
    CardDataBuilder::new("Timid Golem")
        .mana_cost(ManaCost::build(&[], 3))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Golem))
        .power_toughness(2, 2)
        .rules_text("If this creature would be put into a graveyard, exile it instead.")
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::ZoneChange {
                from: None,
                to: Some(Zone::Graveyard),
                cause: None,
                object: None,
            },
            ObjectSet::SourceOnly,
            Rewrite::Instead(GameActionTemplate::ZoneChangeTo {
                to: Zone::Exile,
                cause: ZoneChangeCause::Exiled,
            }),
        )))))
        .build()
}

/// **Fixture.** "Cards in hands lose all abilities." — Yixlid Jailer's row one
/// zone over.
///
/// What it tests is that the zone leg reads the **effective** ability list
/// and not the printed one: with this on the battlefield a Darksteel Colossus
/// in hand has no abilities, so a discard puts it into the graveyard. The
/// gate says the card is a candidate (its *printed* ability functions in a
/// hand) and the sweep, reading the frame LJ made computable, finds nothing —
/// which is the over-approximation the gate is allowed and the answer it is
/// not.
///
/// A hidden-zone reach, and the constraint `layers-architecture.md` §13c
/// decision 4 filed against `backlog.md` §2.9 is why this is a fixture: an
/// anthem into hands is exactly the shape information hides.
///
/// **It owes tests it cannot have yet.** The abilities that function from a
/// hand are the ones this strip must turn off: cycling and channel
/// (CR 113.6j, activated from hand), and the evoke and madness families
/// once `backlog.md` §2.3 lets a card be cast from anywhere but a hand's
/// ordinary door. Each lands with its keyword and this fixture is its
/// negative — `codebase-state.md` main item 147.
pub fn hollow_hands() -> Arc<CardData> {
    CardDataBuilder::new("Hollow Hands")
        .mana_cost(ManaCost::build(&[ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("Cards in hands lose all abilities.")
        .ability(static_ability(Effect::Atom(
            Primitive::LoseAllAbilities(Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredObjectsIn(ObjectFilter::All, ZoneSet::HAND),
        )))
        .build()
}

/// **Fixture.** "If a creature would be put into a graveyard, exile it
/// instead." — a filter row over the **battlefield**, beside Rest in Peace's
/// over `ZoneSet::ALL`.
///
/// CR 109.2: "creature" alone means a creature permanent, and a creature
/// *card* in a library is not one — so this exiles a creature that dies and
/// leaves a milled creature card to reach the graveyard. The affected side's
/// zone check, which the LJ-era `debug_assert` in `set_affects` stood in for:
/// before RF the zone half of a `Filter` was not asked, and this row would
/// have exiled the milled card.
pub fn sealing_ward() -> Arc<CardData> {
    CardDataBuilder::new("Sealing Ward")
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("If a creature would be put into a graveyard, exile it instead.")
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::ZoneChange {
                from: None,
                to: Some(Zone::Graveyard),
                cause: None,
                object: None,
            },
            ObjectSet::battlefield_filter(ObjectFilter::ByType(CardType::Creature)),
            Rewrite::Instead(GameActionTemplate::ZoneChangeTo {
                to: Zone::Exile,
                cause: ZoneChangeCause::Exiled,
            }),
        )))))
        .build()
}
