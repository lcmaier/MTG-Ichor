//! Fixtures for cost determination and payment (CR 601.2f–h,
//! `cost-architecture.md`).

use std::sync::Arc;

use crate::cards::authoring::{activated_ability, artifact, colored, creature, draw_cards, spell_ability, you, you_cast};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::cost_modification::{CostChange, CostModificationDef};
use crate::types::costs::{AdditionalCost, AlternativeCost, Cost};
use crate::types::effects::{AmountExpr, Condition, ObjectFilter};
use crate::types::mana::{ManaCost, ManaType};

/// Locked Sphere — {3}
/// Artifact
///
/// **A fixture, and an invented name.** Trinisphere's line with a mana
/// ability under it: "{T}: Add {C}." It is the lock-in test CM-1 can run
/// without sacrifice-as-cost (CM-3's): the total is locked at three at
/// CR 601.2f, the sphere is tapped for mana in 601.2g's window, and three is
/// what 601.2h pays — Trinisphere's own ruling ("becomes tapped … as a cost
/// to cast a spell"), reached through the mana window rather than improvise.
pub fn locked_sphere() -> Arc<CardData> {
    CardDataBuilder::new("Locked Sphere")
        .mana_cost(ManaCost::build(&[], 3))
        .card_type(CardType::Artifact)
        .mana_ability_single(ManaType::Colorless)
        .rules_text(
            "As long as this artifact is untapped, each spell that would cost less than \
             three mana to cast costs three mana to cast.\n{T}: Add {C}.",
        )
        .ability(
            CostModificationDef::spells(ObjectFilter::All, CostChange::TotalAtLeast(3))
                .into_ability_while("As long as this artifact is untapped, each spell that would cost less than three mana to cast costs three mana to cast.", Condition::SourceUntapped),
        )
        .build()
}

/// A reducer fixture: an enchantment whose one ability is a reduction on the
/// spells its controller casts. `filter` narrows "spells you cast".
fn reducer(name: &str, color: Color, mana: ManaType, filter: ObjectFilter, less: ManaCost, text: &'static str) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[mana], 0))
        .color(color)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(CostModificationDef::spells(you_cast(filter), CostChange::Reduce(less)).into_ability(text))
        .build()
}

/// Generic Reducer — {G} Enchantment: "Spells you cast cost {1} less to cast."
/// One of the three reducers `ATOM-601.2f-002`'s floor board needs.
pub fn generic_reducer() -> Arc<CardData> {
    reducer(
        "Generic Reducer",
        Color::Green,
        ManaType::Green,
        ObjectFilter::All,
        ManaCost::build(&[], 1),
        "Spells you cast cost {1} less to cast.",
    )
}

/// Green Reducer — {G} Enchantment: "Green spells you cast cost {G} less to
/// cast." A *colored* reduction, CR 118.7b–c's shape.
pub fn green_reducer() -> Arc<CardData> {
    reducer(
        "Green Reducer",
        Color::Green,
        ManaType::Green,
        colored(Color::Green),
        ManaCost::build(&[ManaType::Green], 0),
        "Green spells you cast cost {G} less to cast.",
    )
}

/// Red Reducer — {R} Enchantment: "Red spells you cast cost {R} less to
/// cast." With Goblin Electromancer it is `ATOM-601.2f-004`'s board: two
/// *different* reductions on one red instant, ordered by the caster.
pub fn red_reducer() -> Arc<CardData> {
    reducer(
        "Red Reducer",
        Color::Red,
        ManaType::Red,
        colored(Color::Red),
        ManaCost::build(&[ManaType::Red], 0),
        "Red spells you cast cost {R} less to cast.",
    )
}

/// Power Reducer — {2}{W}
/// Creature — Human Soldier, 2/2
///
/// **A fixture, and an invented name.** "Spells you cast cost {X} less to cast,
/// where X is this creature's power." Golden-Tail Trainer prints that shape and
/// cannot be registered yet — its second ability is an attack trigger
/// (critical-path item 6), and a card wearing its name with half its text is
/// what `engineering-practices.md#cards-and-pools` forbids. The *arm* has no
/// such problem, so this fixture is its consumer.
///
/// It is a creature because the amount reads a power, and the reduction is
/// read off the **effective** frame: an anthem on this makes it reduce more,
/// which is the whole of `cost-architecture.md` §3.7's entitlement argument
/// stated as a board.
pub fn power_reducer() -> Arc<CardData> {
    CardDataBuilder::new("Power Reducer")
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Soldier))
        .power_toughness(2, 2)
        .rules_text("Spells you cast cost {X} less to cast, where X is this creature's power.")
        .ability(
            CostModificationDef::spells(
                you_cast(ObjectFilter::All),
                CostChange::ReduceGeneric(AmountExpr::SourcePower),
            )
            .into_ability("Spells you cast cost {X} less to cast, where X is this creature's power."),
        )
        .build()
}

/// A spell fixture: "Draw a card." with the given cost, color and type. The
/// effect is the least a spell can do; the fixture is its cost.
fn lesson(name: &str, cost: ManaCost, color: Color, card_type: CardType) -> CardDataBuilder {
    CardDataBuilder::new(name)
        .mana_cost(cost)
        .color(color)
        .card_type(card_type)
        .rules_text("Draw a card.")
        .ability(spell_ability(
            "Draw a card.",
            draw_cards(you(), 1),
        ))
}

/// Ember Lesson — {1}{R} Instant: "Draw a card." `ATOM-118.7-001`'s {1}{R}.
pub fn ember_lesson() -> Arc<CardData> {
    lesson("Ember Lesson", ManaCost::build(&[ManaType::Red], 1), Color::Red, CardType::Instant).build()
}

/// Crimson Lesson — {1}{R}{R} Instant: "Draw a card." `ATOM-601.2f-004`'s
/// {1}{R}{R} red instant.
pub fn crimson_lesson() -> Arc<CardData> {
    lesson(
        "Crimson Lesson",
        ManaCost::build(&[ManaType::Red, ManaType::Red], 1),
        Color::Red,
        CardType::Instant,
    )
    .build()
}

/// Verdant Lesson — {1}{G} Sorcery: "Draw a card." `ATOM-601.2f-002`'s
/// {1}{G} spell, reduced by {1}, {1} and {G} to {0}.
pub fn verdant_lesson() -> Arc<CardData> {
    lesson("Verdant Lesson", ManaCost::build(&[ManaType::Green], 1), Color::Green, CardType::Sorcery).build()
}

/// Kicked Lesson — {3}{R} Sorcery, kicker {2}: "Draw a card." The kicker buys
/// nothing; `ATOM-601.2f-001`'s board is its *cost* — base, plus the kicker,
/// plus an increase, as one mana component.
pub fn kicked_lesson() -> Arc<CardData> {
    lesson("Kicked Lesson", ManaCost::build(&[ManaType::Red], 3), Color::Red, CardType::Sorcery)
        .additional_cost(AdditionalCost::Kicker(vec![Cost::Mana(ManaCost::build(&[], 2))]))
        .build()
}

/// Bargain Lesson — {2}{R} Sorcery: "You may pay {R} rather than pay this
/// spell's mana cost. Draw a card." `ATOM-118.9d-001`: a modification applies
/// to the alternative cost that was chosen (CR 118.9d).
pub fn bargain_lesson() -> Arc<CardData> {
    lesson("Bargain Lesson", ManaCost::build(&[ManaType::Red], 2), Color::Red, CardType::Sorcery)
        .alternative_cost(AlternativeCost::Custom(
            "Pay {R} rather than pay this spell's mana cost".to_string(),
            vec![Cost::Mana(ManaCost::build(&[ManaType::Red], 0))],
        ))
        .build()
}

/// **A fixture, and an invented name.** "As an additional cost to cast this
/// spell, sacrifice two creatures. Draw a card." — {2}{B} Sorcery.
///
/// No printed card sacrifices two permanents for one cost *and* fits the
/// pool's vocabulary, and the count is worth building: CR 601.2h pays one
/// cost as one thing, so both creatures leave the battlefield together and a
/// "whenever one or more creatures die" trigger will see one event. A loop
/// could not say that (`CLAUDE.md`'s simultaneous rule).
pub fn twin_offering() -> Arc<CardData> {
    lesson("Twin Offering", ManaCost::build(&[ManaType::Black], 2), Color::Black, CardType::Sorcery)
        .additional_cost(AdditionalCost::Mandatory(vec![Cost::Sacrifice(
            creature(),
            2,
        )]))
        .build()
}

/// **A fixture, and an invented name.** "Sacrifice an artifact, {T}: Draw a
/// card." — a {2} Artifact whose ability prints its costs in the order that
/// breaks if the engine pays them as printed.
///
/// Every printed card puts the mana and the tap ahead of the sacrifice, so
/// `payment_order_rank`'s reason is invisible on real cards: paid as printed,
/// this fixture's sacrifice takes its own source and the `{T}` then fails
/// with a payment already made — CR 732.1's cancellation, reached by a cost
/// list rather than by anything a player did. The engine pays object-moving
/// costs last, so the tap happens first and the board stays consistent.
/// Registered nowhere: it exists to make the ordering claim falsifiable.
pub fn self_eating_engine() -> Arc<CardData> {
    CardDataBuilder::new("Self-Eating Engine")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Artifact)
        .rules_text("Sacrifice an artifact, {T}: Draw a card.")
        .ability(activated_ability(
            "Sacrifice an artifact, {T}: Draw a card.",
            vec![
                Cost::Sacrifice(artifact(), 1),
                Cost::TapSelf,
            ],
            draw_cards(you(), 1),
        ))
        .build()
}
