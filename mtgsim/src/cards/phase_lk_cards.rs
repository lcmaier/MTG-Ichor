//! Phase LK — CR 113.6, which abilities function in which zone
//! (`layers-architecture.md` §13d).
//!
//! One registered card and one fixture. Both oracle texts were verified on
//! Scryfall on 2026-09-14 and the registered one is quoted verbatim.
//!
//! **Why Wonder and not a graveyard replacement or a flashback card.** Every
//! zone-reaching effect asks two zone questions — where its *source* is, and
//! where the objects it *affects* are. LJ answered the second (§13c); this row
//! answers the first. Wonder is the smallest card that asks only the first: a
//! static ability functioning **from a graveyard**, granting to permanents on
//! the battlefield, which is the half LJ already does. Madness needs CR 702.35
//! and a hand-functioning replacement, and every flashback card needs
//! `backlog.md` §2.3 before CR 113.6 is any use to it.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, LandType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{
    Condition, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive,
};
use crate::types::ids::new_ability_id;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::zones::ZoneSet;

fn static_ability(effect: Effect) -> AbilityDef {
    AbilityDef {
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
        id: new_ability_id(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect,
    }
}

/// "creatures you control" — CR 109.5's "you" is the source's controller, and
/// off the battlefield CR 108.4 makes that its owner, which is what the layer
/// walk resolves `PlayerRef::You` to for a card in a graveyard.
fn creatures_you_control() -> ObjectFilter {
    ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    )
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
/// The condition is written in the order the card prints it, and each clause
/// does a different job:
///
/// - `SourceInZone(GRAVEYARD)` is CR 113.6b's statement. Read *syntactically*
///   by `zone_function::functioning_zones`, which is what puts the row in the
///   registry when the card reaches a graveyard and keeps it out when the card
///   is on the battlefield — where Wonder is a 2/2 flier that grants nothing.
///   Read *again* at every layer by the existence check, which is what retires
///   the row when the card is exiled (§13d decision 3).
/// - `ControlPermanent(Island)` is an ordinary "as long as", the shape LI-3
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
        .ability(static_ability(Effect::Conditional(
            Condition::All(vec![
                // CR 113.6b — the clause that says where this ability functions.
                Condition::SourceInZone(ZoneSet::GRAVEYARD),
                Condition::ControlPermanent(ObjectFilter::BySubtype(Subtype::Land(
                    LandType::Island,
                ))),
            ]),
            Box::new(Effect::Atom(
                Primitive::GrantKeywordFlag(
                    KeywordFlag::Flying,
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::FilteredPermanents(creatures_you_control()),
            )),
        )))
        .build()
}

/// **Fixture.** "As long as this card isn't on the battlefield, creatures you
/// control get +1/+1."
///
/// CR 113.6c — "an ability that states which zones it *doesn't* function in
/// functions everywhere except for the specified zones" — and the point of the
/// fixture is that it needs no new arm: it is Wonder's clause holding
/// `ZoneSet::EVERYWHERE_BUT_BATTLEFIELD`, the complement LJ already spelled as
/// a constant for Grist and Mycosynth Lattice.
///
/// # This text is a shape no real card has, and the reason is information
///
/// Raised at the LK review, and it is a sharper observation than "the printed
/// population is thin". Every printed card in this family affects **only
/// itself** — Grist, the Hunger Tide is "as long as Grist isn't on the
/// battlefield, **it's** a 1/1 Insect creature" (Scryfall, verified
/// 2026-09-14) — and that is not a coincidence about design taste. A card in
/// a **hidden** zone is `ZoneSet::EVERYWHERE_BUT_BATTLEFIELD`'s problem: an
/// anthem functioning from a library or a hand would change the board while
/// nobody at the table could see why, and there is no good way to tell them.
/// Self-affecting clauses have no such problem, because the object whose
/// characteristics changed is the object nobody can see either.
///
/// **So the fixture deliberately overstates what a card may say**, and keeps
/// doing it: what it is testing is that the *predicate* reads a complement off
/// the same field as a plain zone, and an anthem is what makes that observable
/// from outside the card (`get_effective_power` on another permanent) rather
/// than by querying the fixture's own frame. A self-scoped version would test
/// the CDA path instead, which is `layers::cda`'s and not this predicate's.
///
/// The constraint the card list implies is real and already has an owner:
/// §13c decision 4 filed "a zone-reaching row over a hidden zone must not make
/// that zone's order or contents observable" against `backlog.md` §2.9, the
/// information model. This fixture is a source in a *graveyard*, which CR 400.2
/// makes public, so it does not reach that constraint — but a card genuinely
/// printing this text would, and §2.9 is where it gets answered.
pub fn exiled_ancestor() -> Arc<CardData> {
    CardDataBuilder::new("Exiled Ancestor")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Spirit))
        .power_toughness(1, 1)
        .rules_text("As long as this card isn't on the battlefield, creatures you control get +1/+1.")
        .ability(static_ability(Effect::Conditional(
            Condition::SourceInZone(ZoneSet::EVERYWHERE_BUT_BATTLEFIELD),
            Box::new(Effect::Atom(
                Primitive::ModifyPowerToughness(
                    crate::types::effects::AmountExpr::Fixed(1),
                    crate::types::effects::AmountExpr::Fixed(1),
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::FilteredPermanents(creatures_you_control()),
            )),
        )))
        .build()
}
