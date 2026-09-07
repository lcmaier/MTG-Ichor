//! Cards for the CM phases — cost modification (CR 601.2f, 613.11;
//! `plans/cost-architecture.md`).
//!
//! **CM-1: three printed cards, one per position in CR 601.2f's order**, because
//! the order is the whole rule and one card alone tests none of it: Thalia
//! (an increase, and in `PERFORMANCE_POOL`), Goblin Electromancer (a
//! reduction) and Trinisphere (the one card the "effects that directly affect
//! the total cost" sentence exists for). Every oracle text was read on
//! Scryfall on 2026-09-07 before it was quoted.
//!
//! **And the fixtures, registered nowhere** (`engineering-practices.md` §3's
//! rule: an invented card may not wear a real name while behaving
//! differently, and may not be cited as evidence about printed Magic). They
//! exist because the corpus atoms name boards no three printed cards build —
//! three reductions on one {1}{G} spell, a kicked spell under a tax, a spell
//! that taps its own Trinisphere for mana between the lock and the payment.
//!
//! **CM-2: two printed cards, both affinity for artifacts** (CR 702.41a) —
//! the spell's own cost ability, which is the pipeline's second gather source
//! and its first dynamic amount. Myr Enforcer and Frogmite are vanilla bodies
//! with nothing else on them, so any cost either is cast for is the
//! reduction's doing and nothing else's.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::cost_modification::{CostChange, CostModificationDef};
use crate::types::costs::{AdditionalCost, AlternativeCost, Cost};
use crate::types::effects::{
    AmountExpr, Condition, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive,
};
use crate::types::ids::new_ability_id;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};

/// "Spells you cast" — CR 109.5's "you" is the source's current controller,
/// and a spell's controller is its caster.
fn you_cast(filter: ObjectFilter) -> ObjectFilter {
    ObjectFilter::And(Box::new(filter), Box::new(ObjectFilter::ByController(PlayerRef::You)))
}

/// Thalia, Guardian of Thraben — {1}{W}
/// Legendary Creature — Human Soldier, 2/1
///
/// First strike
/// Noncreature spells cost {1} more to cast.
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// "Each spell that's not a creature spell, including your own" (her ruling
/// of 2021-11-19) — no controller clause, so the filter is the type alone.
/// The increase is CR 601.2f's first position, applied before any reduction;
/// her second ruling restates the order in full.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The pool's first cost effect, so the first card that populates
/// `cost_modification_ability_sources` and makes CR 601.2f's sweep run in a
/// measured game — on both players' spells, since she taxes everyone. A
/// two-drop any white deck casts early; with Humility already pooled, the
/// board where CR 613.11 reads a *stripped* Thalia happens in a measured
/// game too.
pub fn thalia_guardian_of_thraben() -> Arc<CardData> {
    CardDataBuilder::new("Thalia, Guardian of Thraben")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Soldier))
        .power_toughness(2, 1)
        .keyword_flag(KeywordFlag::FirstStrike)
        .rules_text("First strike\nNoncreature spells cost {1} more to cast.")
        .ability(
            CostModificationDef::spells(
                ObjectFilter::Not(Box::new(ObjectFilter::ByType(CardType::Creature))),
                CostChange::Increase(ManaCost::build(&[], 1)),
            )
            .into_ability(),
        )
        .build()
}

/// Goblin Electromancer — {U}{R}
/// Creature — Goblin Wizard, 2/2
///
/// Instant and sorcery spells you cast cost {1} less to cast.
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// The reduction position. Its rulings: the effect "reduces only generic mana
/// in the spell's total cost" (CR 118.7a), and "two Goblin Electromancers
/// will make instant and sorcery spells you cast cost {2} less to cast" —
/// which is also the board on which CR 601.2f's ordering prompt is asked at
/// all, and the reason the stress pool wants this card more than once.
pub fn goblin_electromancer() -> Arc<CardData> {
    CardDataBuilder::new("Goblin Electromancer")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Red], 0))
        .color(Color::Blue)
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Goblin))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .power_toughness(2, 2)
        .rules_text("Instant and sorcery spells you cast cost {1} less to cast.")
        .ability(
            CostModificationDef::spells(
                you_cast(ObjectFilter::Or(
                    Box::new(ObjectFilter::ByType(CardType::Instant)),
                    Box::new(ObjectFilter::ByType(CardType::Sorcery)),
                )),
                CostChange::Reduce(ManaCost::build(&[], 1)),
            )
            .into_ability(),
        )
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
        .rules_text(
            "As long as this artifact is untapped, each spell that would cost less than \
             three mana to cast costs three mana to cast.",
        )
        .ability(
            CostModificationDef::spells(ObjectFilter::All, CostChange::TotalAtLeast(3))
                .into_ability_while(Condition::SourceUntapped),
        )
        .build()
}

// ---------------------------------------------------------------------------
// CM-2 — the spell's own cost abilities (CR 113.6d, 702.41a)
// ---------------------------------------------------------------------------

/// Myr Enforcer — {7}
/// Artifact Creature — Myr, 4/4
///
/// Affinity for artifacts (This spell costs {1} less to cast for each artifact
/// you control.)
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// That is the whole card: a body and the keyword. CR 702.41a *defines*
/// affinity as "This spell costs {1} less to cast for each [text] you
/// control", so [`CardDataBuilder::affinity_for`] writes that sentence and
/// nothing in the engine knows the word.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The pool's first cost ability that is the *spell's own*, so the first
/// measured game in which CR 601.2f's gather has a second source, the
/// castability preview reads a hand card's own ability list, and an
/// `AmountExpr::CountOf` runs at cast time rather than inside a layer walk.
/// Colorless, so every deck in the pool can cast it; an artifact itself, so
/// a second copy in a deck counts the first. Its `{7}` is why it is the one
/// pooled and Frogmite is not: the reduction has room to be large enough to
/// see, where a `{4}` body is castable without ever asking.
///
/// [`CardDataBuilder::affinity_for`]: crate::objects::card_data::CardDataBuilder::affinity_for
pub fn myr_enforcer() -> Arc<CardData> {
    CardDataBuilder::new("Myr Enforcer")
        .mana_cost(ManaCost::build(&[], 7))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Myr))
        .power_toughness(4, 4)
        .rules_text(
            "Affinity for artifacts (This spell costs {1} less to cast for each artifact \
             you control.)",
        )
        .affinity_for(ObjectFilter::ByType(CardType::Artifact))
        .build()
}

/// Frogmite — {4}
/// Artifact Creature — Frog, 2/2
///
/// Affinity for artifacts (This spell costs {1} less to cast for each artifact
/// you control.)
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// The same ability on a smaller body, registered and not pooled. Two
/// affinity cards is what lets a test put one on the battlefield and count it
/// for the other — the board on which a self-reduction reads a board the
/// first copy is standing on.
pub fn frogmite() -> Arc<CardData> {
    CardDataBuilder::new("Frogmite")
        .mana_cost(ManaCost::build(&[], 4))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Frog))
        .power_toughness(2, 2)
        .rules_text(
            "Affinity for artifacts (This spell costs {1} less to cast for each artifact \
             you control.)",
        )
        .affinity_for(ObjectFilter::ByType(CardType::Artifact))
        .build()
}

// ---------------------------------------------------------------------------
// Fixtures — invented names, registered nowhere
// ---------------------------------------------------------------------------

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
                .into_ability_while(Condition::SourceUntapped),
        )
        .build()
}

/// A reducer fixture: an enchantment whose one ability is a reduction on the
/// spells its controller casts. `filter` narrows "spells you cast".
fn reducer(name: &str, color: Color, mana: ManaType, filter: ObjectFilter, less: ManaCost, text: &str) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[mana], 0))
        .color(color)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(CostModificationDef::spells(you_cast(filter), CostChange::Reduce(less)).into_ability())
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
        ObjectFilter::ByColor(Color::Green),
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
        ObjectFilter::ByColor(Color::Red),
        ManaCost::build(&[ManaType::Red], 0),
        "Red spells you cast cost {R} less to cast.",
    )
}

/// A spell fixture: "Draw a card." with the given cost, color and type. The
/// effect is the least a spell can do; the fixture is its cost.
fn lesson(name: &str, cost: ManaCost, color: Color, card_type: CardType) -> CardDataBuilder {
    CardDataBuilder::new(name)
        .mana_cost(cost)
        .color(color)
        .card_type(card_type)
        .rules_text("Draw a card.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::DrawCards(AmountExpr::Fixed(1)),
                EffectRecipient::Controller,
            ),
        })
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
