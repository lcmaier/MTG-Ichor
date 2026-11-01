//! Judgment (`jud`), released 2002-05-27.

use std::sync::Arc;

use crate::cards::authoring::{
    as_long_as, creature, creatures_you_control, each, functioning_in, gains_keyword, static_ability, static_replacement, with_subtype,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, LandType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{Condition, ObjectSet, PlayerFact, PlayerSet};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AuxiliaryMove, EventPattern, ReplacementDef, Rewrite};
use crate::types::zones::{Zone, ZoneChangeAct, ZoneSet};

/// Sutured Ghoul — {4}{B}{B}{B}
/// Creature — Zombie, */*
///
/// Trample
/// As this creature enters, exile any number of creature cards from your
/// graveyard.
/// Sutured Ghoul's power is equal to the total power of the exiled cards and
/// its toughness is equal to their total toughness.
///
/// (Oracle text verified on Scryfall, 2026-09-03.)
///
/// # The second shape of CR 614.13, and it is a different code path
///
/// Devour chooses permanents on the battlefield; this chooses **cards in a
/// player's graveyard**, which `auxiliary_candidates` enumerates from the
/// player's own zone rather than from the shared one. Registering only the
/// first would leave the second exactly as reachable as CR 616.1's
/// multi-candidate branch was after RB shipped Kalitas alone
/// (`engineering-practices.md#how-many-cards-a-mechanic-owes`).
///
/// It is also the card CR 614.13a is written about — "if Sutured Ghoul and
/// Runeclaw Bear enter the battlefield from your graveyard at the same time,
/// you can't choose to exile either of them" — and the *first* clause of that
/// rule is reachable here with one entry and no help: a Ghoul returned from a
/// graveyard is still in the graveyard while its own replacement is applied
/// (RC-4b decides the entry before the move), so without CR 614.13a it exiles
/// itself.
///
/// # What is missing, and it is the third sentence
///
/// The P/T clause reads "**the exiled cards**", which CR 614.14 links to the
/// exiling ability — CR 607's linked abilities, ticket T20, the same thing
/// Painter's Servant waits on. So the characteristic-defining ability CR 208.2
/// gives the `*/*` box is absent, and the card is registered with what that box
/// is worth without it: **0/0**, which is also the right answer whenever no
/// card is exiled. A Ghoul that exiles something should live and dies instead.
/// Recorded in `codebase-state.md`; it is a missing CDA, not a wrong one, and
/// the entry mechanism this card is registered for is complete.
///
/// **Not in `PERFORMANCE_POOL`**: `{4}{B}{B}{B}` is barely castable in a random
/// game, and the pool measures engine cost rather than card coverage
/// (`engineering-practices.md#cards-and-pools`). It is in the default registry,
/// so `--pool stress` plays it.
pub fn sutured_ghoul() -> Arc<CardData> {
    CardDataBuilder::new("Sutured Ghoul")
        .mana_cost(ManaCost::build(
            &[ManaType::Black, ManaType::Black, ManaType::Black],
            4,
        ))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Zombie))
        // `*/*` with its CDA unimplemented — see the doc comment. CR 607.
        .power_toughness(0, 0)
        .keyword_flag(KeywordFlag::Trample)
        .rules_text(
            "Trample\n\
             As this creature enters, exile any number of creature cards from your \
             graveyard.\n\
             Sutured Ghoul's power is equal to the total power of the exiled cards and \
             its toughness is equal to their total toughness.",
        )
        .ability(static_replacement(
            "As this creature enters, exile any number of creature cards from your graveyard.",
            ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                ObjectSet::SourceOnly,
                Rewrite::EnterAfterMoving(AuxiliaryMove {
                    // "Your graveyard" is the zone itself, so the scope needs no
                    // `ByController` leaf — a player's graveyard is enumerated
                    // in its own order.
                    from: Zone::Graveyard,
                    filter: creature(),
                    to: Zone::Exile,
                    act: ZoneChangeAct::Put,
                    up_to: None,
                    // The counters are not how this card grows; CR 614.14's
                    // linked P/T ability is, and it is T20's.
                    per_chosen: None,
                }),
            ),
        ))
        .build()
}

/// Wonder — {3}{U}
/// Creature — Incarnation, 2/2
/// "Flying
///  As long as this card is in your graveyard and you control an Island,
///  creatures you control have flying."
///
/// **The first static ability in the engine that functions off the
/// battlefield**, and the card §13c named for the job in one line: Wonder is
/// precisely the card LJ does not unlock. Its *affected* set is the
/// battlefield, which LJ already reaches; its *source* is in a graveyard,
/// which is CR 113.6b and this row.
///
/// # The ability, clause by clause
///
/// The card prints two clauses, and each does a different job:
///
/// - "This card is in your graveyard" is CR 113.6b's statement, the
///   `Effect::FunctionsIn` around the body. `zone_function::functioning_zones`
///   reads it, which is what puts the row in the registry when the card
///   reaches a graveyard and keeps it out when the card is on the battlefield
///   — where Wonder is a 2/2 flier that grants nothing. The existence check
///   asks it *again* at every layer, which is what retires the row when the
///   card is exiled (§13d decision 3).
/// - "You control an Island" is an ordinary "as long as", the shape LI-3
///   landed for Kird Ape. It names no zone and does not place the ability.
///
/// # Its printed flying is a `KeywordFlag`, not an ability
///
/// So CR 113.6's default arm never sees it: quadrant ① keywords are
/// characteristics on the frame, seeded from the card in every zone. The
/// four-quadrant map is `plans/glossary.md`, "quadrant", with the per-keyword
/// detail in `types::keywords`. A Wonder in a graveyard therefore still *reports*
/// flying if something asks, which CR 113.6 says it should not — recorded as
/// `codebase-state.md`'s LK finding rather than fixed here, because nothing
/// reads a non-battlefield object's keyword flags for a rules decision and the
/// fix belongs with whatever first does.
pub fn wonder() -> Arc<CardData> {
    CardDataBuilder::new("Wonder")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 3))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Incarnation))
        .power_toughness(2, 2)
        .keyword_flag(KeywordFlag::Flying)
        .rules_text(
            "Flying\nAs long as this card is in your graveyard and you control an Island, \
             creatures you control have flying.",
        )
        .ability(functioning_in(
            // CR 113.6b — the clause that says where this ability functions.
            ZoneSet::GRAVEYARD,
            static_ability(
                "As long as this card is in your graveyard and you control an Island, creatures you control have flying.",
                as_long_as(
                    Condition::Player {
                        players: PlayerSet::You,
                        fact: PlayerFact::ControlsPermanent(with_subtype(Subtype::Land(LandType::Island))),
                    },
                    gains_keyword(each(creatures_you_control()), KeywordFlag::Flying),
                ),
            ),
        ))
        .build()
}
