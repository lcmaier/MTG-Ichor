//! Darksteel (`dst`), released 2004-02-06.

use std::sync::Arc;

use crate::cards::authoring::shuffles_into_library_from_anywhere;
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::cost_modification::{CostChange, CostModificationDef};
use crate::types::effects::{Condition, ObjectFilter};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::ManaCost;

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
/// **This card and not Blightsteel Colossus**, which the owner named for the
/// leg: Blightsteel's clause is this one plus infect, and infect (CR 702.90)
/// is not a keyword the engine has, so a registered Blightsteel would wear a
/// real name while behaving differently, which
/// `engineering-practices.md#cards-and-pools` forbids. This card prints the
/// same clause with only trample and indestructible, both of which the
/// engine plays.
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
        .rules_text("Trample\nIndestructible\nIf Darksteel Colossus would be put into a graveyard from anywhere, reveal Darksteel Colossus and shuffle it into its owner's library instead.")
        .ability(shuffles_into_library_from_anywhere("If Darksteel Colossus would be put into a graveyard from anywhere, reveal Darksteel Colossus and shuffle it into its owner's library instead.", ))
        .build()
}

/// Trinisphere — {3}
/// Artifact
///
/// As long as this artifact is untapped, each spell that would cost less than
/// three mana to cast costs three mana to cast. (Additional mana in the cost
/// may be paid with any color of mana or colorless mana. For example, a spell
/// that would cost {1}{B} to cast costs {2}{B} to cast instead.)
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// The third position — CR 601.2f's "any effects that directly affect the
/// total cost", a sentence that exists for this card and no other printed one
/// (`cost-architecture.md` §1). Its rulings: "apply Trinisphere's effect if
/// the mana component of the spell's cost is less than three mana" after
/// every increase and reduction, and "if Trinisphere leaves the battlefield
/// or becomes tapped or untapped as a cost to cast a spell, this cost is paid
/// after you've locked in the total cost" — which is [`locked_sphere`]'s
/// board.
///
/// "As long as this artifact is untapped" is a conditional static (LI-3's
/// shape) over a *status*, `Condition::SourceUntapped`, read against the
/// settled board at CR 601.2f.
pub fn trinisphere() -> Arc<CardData> {
    CardDataBuilder::new("Trinisphere")
        .mana_cost(ManaCost::build(&[], 3))
        .card_type(CardType::Artifact)
        .rules_text("As long as this artifact is untapped, each spell that would cost less than three mana to cast costs three mana to cast.")
        .ability(
            CostModificationDef::spells(ObjectFilter::All, CostChange::TotalAtLeast(3))
                .into_ability_while("As long as this artifact is untapped, each spell that would cost less than three mana to cast costs three mana to cast.", Condition::SourceUntapped),
        )
        .build()
}
