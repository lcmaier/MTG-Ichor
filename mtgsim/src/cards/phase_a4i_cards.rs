//! Cards for A4i — several instances of the word "target" on one spell
//! (CR 115.3, 601.2c, 608.2b).
//!
//! **Five cards, three shapes, and the shapes are the point.** CR 601.2c draws
//! three lines and a phase that shipped only one of them would have defined the
//! instance model under no pressure:
//!
//! | Shape | Card here | What it presses on |
//! |---|---|---|
//! | One instance, several choices | Jagged Lightning | the choices must differ, and the spell needs two creatures to be castable |
//! | Several instances, free to share | Seeds of Strength, Plague Spores | one object chosen once for each instance |
//! | Several instances, excluding each other | Incremental Growth | "another target", which is a criterion rather than a rule |
//!
//! Seat of the Synod is the fifth and is here for one board: ATOM-608.2b-002
//! wants the *same* permanent to be a legal land and an illegal nonblack
//! creature at once, which needs an artifact land, March of the Machines to
//! make it a creature and Moonlace to make it black. The last two are already
//! in the pool; the land was the missing piece.
//!
//! **What a random deck can draw.** Seeds of Strength is the only one in
//! `PERFORMANCE_POOL` (`engineering-practices.md` §3.1 — one card per new
//! engine path, deliberately, with a re-record attached). It is the cheapest of
//! the five, it is castable with a single creature on the board because
//! CR 601.2c lets all three clauses name it, and at that board it asks nothing
//! at all: one legal choice for a fixed count of one is not a choice (CR 102.2).
//! The other four are registered and unpooled — Plague Spores and Incremental
//! Growth want boards a random game will not build, and Jagged Lightning's
//! `TargetCount::Exactly(2)` is the same engine path Seeds of Strength already
//! walks.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerRef,
    PlayerSet, Primitive, SelectionFilter, TargetCount,
};
use crate::types::ids::AbilityId;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::restriction::{ReplacementKindFilter, Restriction, RestrictionDef};

fn spell(effect: Effect) -> AbilityDef {
    AbilityDef {
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
        id: AbilityId::UNASSIGNED,
        ability_type: AbilityType::Spell,
        costs: Vec::new(),
        effect,
    }
}

fn target_creature() -> EffectRecipient {
    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1))
}

/// Seeds of Strength — {G}{W}
/// Instant
/// Target creature gets +1/+1 until end of turn.
/// Target creature gets +1/+1 until end of turn.
/// Target creature gets +1/+1 until end of turn.
///
/// **Three instances of "target" with identical clauses**, which is the case
/// no structural rule can decide. Written the obvious way — each atom carrying
/// the clause it acts on — this card and Ensoul Artifact are both a `Sequence`
/// whose atoms carry pairwise-equal recipients, and their instance counts are
/// three and one. Ensoul Artifact prints "target artifact" once and acts on it
/// twice; this prints "target creature" three times. Nothing in the shape
/// separates them, so the card says which it means
/// (`EffectRecipient::SameInstanceAs`).
///
/// In `PERFORMANCE_POOL`, and why: several-clause casting is a new engine path
/// (`engineering-practices.md` §3.1), and this is the cheapest card that walks
/// it — two mana, both colors reachable from the pool's duals, and castable
/// with one creature on the board.
///
/// # The rulings, and where each is tested
///
/// - *"You may choose the same creature as a target multiple times since the
///   card says 'target creature' multiple times. You may give three different
///   creatures +1/+1 each, one creature +2/+2 and another creature +1/+1, or a
///   single creature +3/+3."* — **testable now**, and it is CR 601.2c's second
///   sentence with this card's numbers. All three distributions are asserted:
///   `seeds_of_strength_can_name_one_creature_for_all_three_instances`,
///   `seeds_of_strength_can_split_its_three_instances_across_creatures`.
pub fn seeds_of_strength() -> Arc<CardData> {
    let pump = |recipient| {
        Effect::Atom(
            Primitive::ModifyPowerToughness(
                AmountExpr::Fixed(1),
                AmountExpr::Fixed(1),
                Duration::UntilEndOfTurn,
            ),
            recipient,
        )
    };
    CardDataBuilder::new("Seeds of Strength")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::White], 0))
        .color(Color::Green)
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(
            "Target creature gets +1/+1 until end of turn. \
             Target creature gets +1/+1 until end of turn. \
             Target creature gets +1/+1 until end of turn.",
        )
        .ability(spell(Effect::Sequence(vec![
            pump(target_creature()),
            pump(target_creature()),
            pump(target_creature()),
        ])))
        .build()
}

/// Incremental Growth — {3}{G}{G}
/// Sorcery
/// Put a +1/+1 counter on target creature, two +1/+1 counters on another
/// target creature, and three +1/+1 counters on a third target creature.
///
/// **The "another target" shape**, which CR 601.2c reaches through its
/// parenthesis rather than through a rule of its own: one object may be chosen
/// once for each instance "as long as it fits the targeting criteria", and
/// "another" is how a card writes the exclusion *into* those criteria. Each
/// later clause carries `ObjectFilter::OtherThanInstance` for every earlier one.
///
/// Registered and unpooled: three distinct creatures is a board a random game
/// reaches rarely, and the castability rule it presses on is the same one
/// Jagged Lightning presses on more cheaply.
///
/// # The rulings, and where each is tested
///
/// - *"You must choose three different targets in order to cast Incremental
///   Growth."* — **testable now**. Two creatures is not enough, and the spell
///   is not offered at all rather than being offered and rewound:
///   `incremental_growth_needs_a_third_creature_to_be_castable`.
/// - *"If some of the creatures are illegal targets as Incremental Growth tries
///   to resolve, the remaining legal targets still get the appropriate number
///   of +1/+1 counters. If all targets are illegal, Incremental Growth doesn't
///   resolve."* — **testable now**, and it is both halves of CR 608.2b in one
///   sentence: `incremental_growth_still_counters_the_creatures_that_are_left`
///   and `incremental_growth_does_not_resolve_with_every_creature_gone`.
///
/// *"You decide how many +1/+1 counters each creature will get as part of
/// casting the spell"* is not a third ruling here — the counts are printed
/// (one, two, three) and belong to the clauses, not to CR 601.2d's division.
pub fn incremental_growth() -> Arc<CardData> {
    // "another target creature", "a third target creature": a creature, and not
    // one an earlier instance took.
    let other_than = |earlier_targets: &[usize]| {
        let mut filter = ObjectFilter::ByType(CardType::Creature);
        for &ix in earlier_targets {
            filter = ObjectFilter::And(
                Box::new(filter),
                Box::new(ObjectFilter::OtherThanInstance(ix)),
            );
        }
        EffectRecipient::Target(SelectionFilter::Permanent(filter), TargetCount::Exactly(1))
    };
    let grow = |n: u64, recipient| {
        Effect::Atom(
            Primitive::AddCounters {
                counter: CounterType::PlusOnePlusOne,
                amount: AmountExpr::Fixed(n),
                by: PlayerRef::You,
            },
            recipient,
        )
    };
    CardDataBuilder::new("Incremental Growth")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Sorcery)
        .rules_text(
            "Put a +1/+1 counter on target creature, two +1/+1 counters on another target \
             creature, and three +1/+1 counters on a third target creature.",
        )
        .ability(spell(Effect::Sequence(vec![
            grow(1, target_creature()),
            grow(2, other_than(&[0])),
            grow(3, other_than(&[0, 1])),
        ])))
        .build()
}

/// Jagged Lightning — {3}{R}{R}
/// Sorcery
/// Jagged Lightning deals 3 damage to each of two target creatures.
///
/// **One instance of "target" holding two choices**, which is the other half of
/// CR 601.2c's first sentence and the half no registered card had: the two
/// creatures must differ, and a board with one creature does not make the spell
/// castable. `ATOM-608.2b-005`'s card, and the cheapest witness that CR 608.2b
/// is per target — a creature that gains protection in response is skipped
/// while the other still takes its three.
///
/// Registered and unpooled: `TargetCount::Exactly(2)` walks the same
/// announcement loop Seeds of Strength already walks, so pooling it would move
/// the stream for no new path (`engineering-practices.md` §3.1).
///
/// # The rulings, and where each is tested
///
/// Scryfall lists none for this card.
pub fn jagged_lightning() -> Arc<CardData> {
    CardDataBuilder::new("Jagged Lightning")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 3))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text("Jagged Lightning deals 3 damage to each of two target creatures.")
        .ability(spell(Effect::Atom(
            Primitive::DealDamage { amount: AmountExpr::Fixed(3), unpreventable: false },
            EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(2)),
        )))
        .build()
}

/// Plague Spores — {4}{B}{R}
/// Sorcery
/// Destroy target nonblack creature and target land. They can't be regenerated.
///
/// **Two instances with different criteria**, and `ATOM-608.2b-002`'s card: the
/// CR's own example of one object satisfying both clauses is an artifact
/// creature land, and when it turns black the creature clause goes illegal
/// while the land clause does not. That board is what proves the two rules of
/// CR 608.2b are different rules.
///
/// **"They" is two atoms, not one.** An atom reads its own instance, so the
/// regeneration restriction is written once per clause. That is not a
/// workaround: CR 701.19c's effect is about each permanent, and a single atom
/// spanning two instances would need a recipient the CR never asks for.
///
/// Registered and unpooled: the board needs March of the Machines and Moonlace
/// together, which a random game does not assemble.
///
/// # The rulings, and where each is tested
///
/// Scryfall lists none for this card.
pub fn plague_spores() -> Arc<CardData> {
    let cant_regenerate = |recipient| {
        Effect::Atom(
            Primitive::Restrict(
                RestrictionDef::new(Restriction::ApplyReplacement {
                    kind: ReplacementKindFilter::Regeneration,
                    // `Primitive::Restrict` overwrites the object set with the
                    // resolution's targets — this atom's instance.
                    to_objects: ObjectSet::NO_OBJECTS,
                    to_players: PlayerSet::Nobody,
                }),
                Duration::UntilEndOfTurn,
            ),
            recipient,
        )
    };
    CardDataBuilder::new("Plague Spores")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Red], 4))
        .color(Color::Black)
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text("Destroy target nonblack creature and target land. They can't be regenerated.")
        .ability(spell(Effect::Sequence(vec![
            // The shields are stripped before the destruction, or CR 701.19a's
            // replacement would have already applied by the time the
            // restriction existed.
            cant_regenerate(EffectRecipient::Target(
                SelectionFilter::Permanent(ObjectFilter::And(
                    Box::new(ObjectFilter::ByType(CardType::Creature)),
                    Box::new(ObjectFilter::Not(Box::new(ObjectFilter::ByColor(Color::Black)))),
                )),
                TargetCount::Exactly(1),
            )),
            cant_regenerate(EffectRecipient::Target(
                SelectionFilter::Permanent(ObjectFilter::ByType(CardType::Land)),
                TargetCount::Exactly(1),
            )),
            Effect::Atom(Primitive::Destroy, EffectRecipient::SameInstanceAs(0)),
            Effect::Atom(Primitive::Destroy, EffectRecipient::SameInstanceAs(1)),
        ])))
        .build()
}

/// Seat of the Synod
/// Artifact Land
/// {T}: Add {U}.
///
/// Here for one board and not for a rule: `ATOM-608.2b-002` needs a permanent
/// that can be both a land and a creature, and the engine's route to that is
/// March of the Machines animating an artifact land. An artifact land is also
/// CR 601.2c's own example ("Destroy target artifact and target land … can
/// target the same artifact land twice"), so it earns the registration twice
/// over.
///
/// Registered and unpooled: a land that taps for one blue adds nothing a basic
/// Island does not, and pooling it would move the stream to measure nothing.
///
/// # The rulings, and where each is tested
///
/// Scryfall lists none for this card.
pub fn seat_of_the_synod() -> Arc<CardData> {
    CardDataBuilder::new("Seat of the Synod")
        .card_type(CardType::Artifact)
        .card_type(CardType::Land)
        .rules_text("{T}: Add {U}.")
        .mana_ability_single(ManaType::Blue)
        .build()
}
