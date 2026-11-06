//! Fixtures for the replacement pipeline's paths (CR 614–616,
//! `replacement-architecture.md`).

use std::sync::Arc;

use crate::cards::authoring::{creature, each_in, loses_all_abilities, static_ability, static_replacement, you_control};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{ObjectFilter, ObjectSet};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{CopyDonor, EntryCopyTemplate, EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use crate::types::zones::{Zone, ZoneChangePattern, ZoneSet};

/// **Fixture.** Essence of the Wild's body and its one ability: "Creatures you
/// control enter as a copy of this creature." What every creature of an event
/// enters as, so each is a copy from the moment it is announced.
pub fn essence_of_the_wild_fixture() -> Arc<CardData> {
    CardDataBuilder::new("Essence of the Wild Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green, ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Avatar))
        .power_toughness(6, 6)
        .rules_text("Creatures you control enter as a copy of this creature.")
        .ability(static_replacement("Creatures you control enter as a copy of this creature.", ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::battlefield_filter(creature().and(you_control())),
            Rewrite::EnterAsCopy(EntryCopyTemplate { donor: CopyDonor::ThisObject, except: Vec::new() }),
        )))
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
        .ability(static_replacement("If this creature would be put into a graveyard, exile it instead.", ReplacementDef::new(
            EventPattern::ZoneChange {
                change: ZoneChangePattern { from: None, to: Some(Zone::Graveyard), act: None },
                object: None,
            },
            ObjectSet::SourceOnly,
            Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
        )))
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
        .ability(static_ability("Cards in hands lose all abilities.", loses_all_abilities(each_in(ObjectFilter::All, ZoneSet::HAND))))
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
        .ability(static_replacement("If a creature would be put into a graveyard, exile it instead.", ReplacementDef::new(
            EventPattern::ZoneChange {
                change: ZoneChangePattern { from: None, to: Some(Zone::Graveyard), act: None },
                object: None,
            },
            ObjectSet::battlefield_filter(creature()),
            Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
        )))
        .build()
}
