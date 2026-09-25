//! Cards for Phase RE-9 — mana (`replacement-architecture.md` §9).
//!
//! **The mana production event gets its first watchers.** Until RE-9 the two
//! places that added mana wrote the pool directly and nothing could see
//! them; `GameAction::ProduceMana` is that event, and these are the printed
//! replacement effects CR 106.6a and CR 106.12b describe — plus the one
//! producer the corpus itself named as CR 106.6's integration test. Five
//! cards, on three axes:
//!
//! | Card | What it is the first of | CR |
//! |---|---|---|
//! | [`mana_reflection`] | a multiplier over a production | 106.6a |
//! | [`nyxbloom_ancient`] | the second factor the arm has seen | 106.6a |
//! | [`deep_water`] | a production retyped, with a filter on the permanent | 106.12b |
//! | [`pale_moon`] | the same retype for every player's lands, from an instant | 106.12b |
//! | [`doubling_cube`] | a mana ability with a dynamic amount, and {T} on a non-land | 106.6, 106.12 |
//!
//! Every replacement among them says "tap … for mana", which is CR 106.12's definition
//! — *"to activate a mana ability of that permanent that includes the {T}
//! symbol in its activation cost"* — and so every one of them is
//! `EventPattern::ProduceMana { tapped_for_mana: Some(true), .. }`. The
//! definition is what makes each card's second ruling true without a line of
//! code: a spell's production and a triggered mana ability's are not tapping
//! a permanent for mana, and neither proposes `true`.
//!
//! # What is not here, and what each waits for
//!
//! **Sixteen** printed cards replace "tapped for mana" (Scryfall, 2026-09-15,
//! re-run at review with the rule's phrasing rather than the card's —
//! `replacement-architecture.md` §11 item 98). Three multiply; **Virtue of
//! Strength** is the third ("if you tap a *basic land* for mana … three
//! times") and an Adventure card, which the engine has no second face for.
//! Seven retype to a constant: the two here, **Contamination** and **Infernal
//! Darkness** (each with an upkeep half that is critical-path item 6's, so
//! both are fixtures in `tests/phase_re9_integration_test.rs`, where the CR
//! is the customer), **Ritual of Subdual** (cumulative upkeep, the same),
//! **Damping Sphere** ("tapped for *two or more* mana" — an amount
//! constraint the pattern has no field for, and a second ability that needs
//! a spells-cast-this-turn count) and **Quarum Trench Gnomes** ("*target*
//! Plains … instead of *white* mana" — a chosen permanent and a type
//! constraint, on a row that lasts indefinitely). Five retype to a chosen or
//! mapped color: **Hall of Gemstone** wants a color chosen at upkeep, **Naked
//! Singularity** and **Reality Twist** a map from basic land type to color,
//! **Harvest Mage** and **Pulse of Llanowar** a choice inside the
//! substitution — facilities, none of them this phase's. **Chaos Moon**'s
//! even half is Ritual of Subdual's line under an upkeep parity check.
//! **False Dawn** — "spells and abilities you control that would add colored
//! mana instead add that much white mana" — is the one printed watcher that
//! does not say "tapped" and would be the pattern's `None`; its second
//! sentence is a payment rule (spend white as any color) that `ManaPool`
//! does not have. The eight "whenever you tap … for mana, add an additional"
//! cards, and Snowfall, are CR 605.1b's triggered mana abilities and item
//! 6's.
//!
//! # What a random deck can draw
//!
//! **Mana Reflection is the pooled card**, and it is why this PR exists: a
//! six-drop static on the hottest path in the engine, so that from the turn
//! it resolves every land tap is a gather with a match. Nyxbloom Ancient
//! stays out — the same path at seven mana with a 5/5 trample body that
//! changes combat — and Deep Water stays out as an activated `{U}` the random
//! agent would spend on nothing; its reachability is a `--require` row.

use std::sync::Arc;

use crate::objects::card_data::{
    AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder,
};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, ManaOutput, ObjectFilter, ObjectSet,
    PatternFill, PlayerRef, PlayerSet, Primitive,
};
use crate::types::ids::AbilityId;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{
    AmountRewrite, EventPattern, GameActionTemplate, ReplacementDef, Rewrite, TemplateAmount,
};

/// A static ability whose effect is a replacement effect — never a resolution,
/// so it carries no `Duration` and is re-derived off the source's *effective*
/// ability list on every gather.
fn static_replacement(def: ReplacementDef) -> AbilityDef {
    AbilityDef {
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect: Effect::Replacement(Box::new(def)),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    }
}

/// CR 106.12's "tapped for mana", optionally of a permanent matching `source`.
fn tapped_for_mana(source: Option<ObjectFilter>) -> EventPattern {
    EventPattern::ProduceMana { tapped_for_mana: Some(true), source }
}

/// Mana Reflection — {4}{G}{G}
/// Enchantment
///
/// > If you tap a permanent for mana, it produces twice as much of that mana
/// > instead.
///
/// **The pooled card, and CR 106.6a's first customer.** `Amount(Multiplier(2))`
/// over every unit the production carries — the plain ones scaled, the
/// restricted atoms repeated, which is the rule's "any restrictions … will
/// apply to all mana produced". `PlayerSet::You` is "if *you* tap".
///
/// # The rulings, and where each is tested
///
/// - *"You're 'tapping a permanent for mana' only if you're activating a mana
///   ability of that permanent that includes the {T} symbol in its cost. A
///   mana ability produces mana as part of its effect."* → CR 106.12, read off
///   the activation cost: a Forest is doubled, Dark Ritual is not
///   (`dark_ritual_is_not_tapping_a_permanent_for_mana`), and neither is
///   Krark-Clan Ironworks' sacrifice
///   (`a_mana_ability_without_a_tap_symbol_is_not_doubled`).
/// - *"If an ability triggers 'whenever you tap' something for mana and
///   produces mana, that triggered mana ability won't be affected by Mana
///   Reflection."* → CR 605.1b's triggered mana abilities are critical-path
///   item 6's; the day one exists its own production proposes
///   `tapped_for_mana: false` by the same definition, and this ruling is
///   that PR's test rather than this one's.
/// - *"Mana Reflection doesn't produce any mana itself. Rather, it causes
///   permanents you tap for mana to produce more mana. If the mana ability of
///   that permanent puts any restrictions or riders on the mana it produces,
///   that will apply to all the mana it produces this way."* →
///   `ATOM-106.6a-001`, on a fixture land whose {G} is spendable only on
///   creature spells: two restricted atoms, no free one
///   (`a_restricted_production_doubles_into_two_restricted_units`).
/// - *"The effects of multiple Mana Reflections are cumulative. For example,
///   if you have two Mana Reflections on the battlefield, you'll get four
///   times the original amount and type of mana. If you have three, you'll
///   get eight times the mana, and so on."* → four, and the prompt is the
///   first assertion: two multipliers commute, so CR 616.1 has nothing to ask
///   (`two_mana_reflections_quadruple_with_no_prompt`,
///   `three_mana_reflections_multiply_by_eight`).
pub fn mana_reflection() -> Arc<CardData> {
    CardDataBuilder::new("Mana Reflection")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 4))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text("If you tap a permanent for mana, it produces twice as much of that mana instead.")
        .ability(static_replacement(
            ReplacementDef::new(
                tapped_for_mana(None),
                ObjectSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Nyxbloom Ancient — {4}{G}{G}{G}
/// Enchantment Creature — Elemental, 5/5
///
/// > Trample
/// > If you tap a permanent for mana, it produces three times as much of
/// > that mana instead.
///
/// **The second factor the arm has seen**, and a creature, so the ability
/// exists only while the body does. Registered and unpooled: the same path
/// as Mana Reflection at seven mana, with a 5/5 trample body that would move
/// the combat rows by design.
///
/// # The rulings, and where each is tested
///
/// Its four rulings are Mana Reflection's four with the number changed, so
/// three of them are tested once, above. The fourth is its own: *"The
/// effects of multiple Nyxbloom Ancients are cumulative. For example, if you
/// have two Nyxbloom Ancients on the battlefield, you'll get nine times the
/// original amount and type of mana."* → nine, no prompt
/// (`two_nyxbloom_ancients_produce_nine_times_the_mana`) — and the two cards
/// together are six, the mixed pair the commuting cell was written for
/// (`a_reflection_and_an_ancient_produce_six_times_the_mana`).
pub fn nyxbloom_ancient() -> Arc<CardData> {
    CardDataBuilder::new("Nyxbloom Ancient")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green, ManaType::Green], 4))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elemental))
        .power_toughness(5, 5)
        .keyword_flag(KeywordFlag::Trample)
        .rules_text(
            "Trample\nIf you tap a permanent for mana, it produces three times as much of that mana instead.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                tapped_for_mana(None),
                ObjectSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Multiplier(3)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Deep Water — {U}{U}
/// Enchantment
///
/// > {U}: Until end of turn, if you tap a land you control for mana, it
/// > produces {U} instead of any other type.
///
/// **CR 106.12b's "of a specific type", and the one of its six printed cards
/// that registers whole today.** An activated ability whose effect is Fog's
/// shape — `Primitive::CreateReplacement` until end of turn — with
/// `Instead(ProduceMana { Blue, ReplacedAmount })`: every unit retyped, the
/// amount kept. "A land you control" is the pattern's `source` filter, asked
/// of the tapped permanent at the proposal, and it is on the pattern rather
/// than in `affected_objects` because the event's subject is the player.
///
/// # The rulings, and where each is tested
///
/// - *"The amount of mana produced is unchanged, but it will all be {U}."* →
///   a land producing two of one color produces two blue
///   (`deep_water_keeps_the_amount_and_changes_the_type`), and a restricted
///   unit keeps its restriction with its new color
///   (`a_retyped_restricted_unit_keeps_its_restriction`).
/// - *"Deep Waters affects lands you control when it resolves and any lands
///   you gain control of this turn."* → the filter is evaluated at each
///   proposal, not captured when the row is made: a land that arrives under
///   your control after the activation is retyped
///   (`deep_water_reaches_a_land_gained_after_the_activation`).
///
/// Beside Mana Reflection the two commute — retype-then-double and
/// double-then-retype are one event — so a Forest under both adds {U}{U} with
/// no CR 616.1 prompt (`deep_water_and_mana_reflection_commute`).
pub fn deep_water() -> Arc<CardData> {
    CardDataBuilder::new("Deep Water")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 0))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text(
            "{U}: Until end of turn, if you tap a land you control for mana, it produces {U} instead of any other type.",
        )
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::Mana(ManaCost::build(&[ManaType::Blue], 0))],
            effect: Effect::Atom(
                Primitive::CreateReplacement(
                    Box::new(
                        ReplacementDef::new(
                            tapped_for_mana(Some(ObjectFilter::And(
                                Box::new(ObjectFilter::ByType(CardType::Land)),
                                Box::new(ObjectFilter::ByController(PlayerRef::You)),
                            ))),
                            ObjectSet::NO_OBJECTS,
                            Rewrite::Instead(GameActionTemplate::ProduceMana {
                                mana_type: ManaType::Blue,
                                amount: TemplateAmount::ReplacedAmount,
                            }),
                        )
                        .affecting_players(PlayerSet::You),
                    ),
                    Duration::UntilEndOfTurn,
                    PatternFill::Authored,
                ),
                EffectRecipient::Implicit,
            ),
        })
        .build()
}

/// Pale Moon — {1}{U}
/// Instant
///
/// > Until end of turn, if a player taps a nonbasic land for mana, it
/// > produces colorless mana instead of any other type.
///
/// **The second registrable type-changer, found at review** — the census
/// regex read the card's phrase "tap … for mana" and this card says "taps a
/// nonbasic land for mana" (`replacement-architecture.md` §11 item 98). Deep
/// Water's shape on an instant: Fog's `CreateReplacement` until end of turn,
/// `PlayerSet::Everyone` for "a player", and "nonbasic land" as the filter on
/// the tapped permanent. Registered and unpooled: a one-shot whose engine path
/// Deep Water already opens.
///
/// # The rulings, and where each is tested
///
/// - *"The ability does not change the amount of mana produced, only the
///   color."* → `ReplacedAmount`: an opponent's two-mana nonbasic land
///   produces two colorless, and a basic Forest is left alone
///   (`pale_moon_retypes_any_players_nonbasic_land_and_leaves_a_basic_alone`).
pub fn pale_moon() -> Arc<CardData> {
    CardDataBuilder::new("Pale Moon")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(
            "Until end of turn, if a player taps a nonbasic land for mana, it produces colorless mana instead of any other type.",
        )
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::CreateReplacement(
                    Box::new(
                        ReplacementDef::new(
                            tapped_for_mana(Some(ObjectFilter::And(
                                Box::new(ObjectFilter::ByType(CardType::Land)),
                                Box::new(ObjectFilter::Not(Box::new(ObjectFilter::BySupertype(
                                    Supertype::Basic,
                                )))),
                            ))),
                            ObjectSet::NO_OBJECTS,
                            Rewrite::Instead(GameActionTemplate::ProduceMana {
                                mana_type: ManaType::Colorless,
                                amount: TemplateAmount::ReplacedAmount,
                            }),
                        )
                        .affecting_players(PlayerSet::Everyone),
                    ),
                    Duration::UntilEndOfTurn,
                    PatternFill::Authored,
                ),
                EffectRecipient::Implicit,
            ),
        })
        .build()
}

/// Doubling Cube — {2}
/// Artifact
///
/// > {3}, {T}: Double the amount of each type of unspent mana you have.
///
/// **The corpus's own integration test for CR 106.6, and a mana ability with
/// {T} on a permanent that is not a land.** The atomic-test session that
/// wrote `ATOM-106.6-001` named this card as the integration test for
/// restricted mana under doubling and deferred it to the suite; it lands
/// here because RE-9 is where a production became an event. Its first ruling
/// is what makes it this phase's: *"Doubling Cube's ability is a mana
/// ability"* — CR 605.1a, no target, adds mana — and its cost includes {T},
/// so by CR 106.12 tapping it is "tapping a permanent for mana" and Mana
/// Reflection doubles what it produces.
///
/// The amount is the first dynamic one a mana ability has carried:
/// `AmountExpr::UnspentMana(type)` per type, read off the activating
/// player's pool when the ability resolves — after the {3} is paid, since an
/// activation pays its costs before the ability resolves (CR 602.2, 605.3b).
/// The output lists all six types; the performer adds nothing for a type at
/// zero. Registered and unpooled: `available_mana_sources` enumerates fixed
/// amounts only, so the random agent never reaches for it, and a card that
/// doubles a pool would move the gameplay rows by design.
///
/// # The rulings, and where each is tested
///
/// - *"Doubling Cube's ability is a mana ability."* → activated through
///   `activate_mana_ability`, and its production says `tapped_for_mana`, so
///   Mana Reflection doubles the doubling
///   (`doubling_cube_is_tapped_for_mana_so_mana_reflection_doubles_its_doubling`).
/// - *"The 'type' of mana is its color, or lack thereof."* → `UnspentMana`
///   is asked per [`ManaType`], colorless included.
/// - *"Any restrictions on the unspent mana aren't copied. For example, if you
///   have {C}{W}{W}{B} with no restrictions on it and {U}{U}{U} that can be
///   used only to cast artifact spells, you'll end up with
///   {C}{C}{W}{W}{W}{W}{B}{B}, {U}{U}{U} that can be used only to cast
///   artifact spells, and {U}{U}{U} that can be used for anything."* → the
///   ruling's own board: restricted units counted by their type, the copies
///   free (`doubling_cube_counts_restricted_mana_and_copies_it_unrestricted`).
///   CR 106.6's sentence — a restriction "doesn't affect the mana's type" —
///   is what makes the restricted {U}{U}{U} count as three blue.
pub fn doubling_cube() -> Arc<CardData> {
    CardDataBuilder::new("Doubling Cube")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Artifact)
        .rules_text("{3}, {T}: Double the amount of each type of unspent mana you have.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Mana,
            costs: vec![Cost::Mana(ManaCost::build(&[], 3)), Cost::TapSelf],
            effect: Effect::Atom(
                Primitive::ProduceMana(ManaOutput {
                    mana: [
                        ManaType::White,
                        ManaType::Blue,
                        ManaType::Black,
                        ManaType::Red,
                        ManaType::Green,
                        ManaType::Colorless,
                    ]
                    .into_iter()
                    .map(|t| (t, AmountExpr::UnspentMana(t)))
                    .collect(),
                    special: Vec::new(),
                }),
                EffectRecipient::Implicit,
            ),
        })
        .build()
}
