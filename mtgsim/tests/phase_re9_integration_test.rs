//! Phase RE-9 — mana: CR 106.6a's replaceable production and CR 106.12's
//! "tapped for mana".
//!
//! Every board here is one mana ability resolving, or one spell adding mana,
//! with something watching the production — the three printed cards in
//! `cards::phase_re9_cards`, and two fixtures that are Contamination's and
//! Infernal Darkness's replacement lines without the upkeep halves item 6
//! owns. Four groups:
//!
//! - **The event itself** — one performer where there were two silent
//!   writers: what it announces, that a production of nothing announces
//!   nothing, and that the tap and the production are two batches
//!   (CR 605.3b, 106.12a).
//! - **`tapped_for_mana`** — CR 106.12's definition read off the activation
//!   cost: a Forest is, Dark Ritual is not (CR 605.5b), an Ironworks
//!   sacrifice is not.
//! - **The multipliers** — CR 106.6a over plain and restricted units, and the
//!   commuting cell that makes two Reflections ask nobody.
//! - **The retype** — CR 106.12b's "specific type" through
//!   `GameActionTemplate::ProduceMana`, its filter on the permanent, its
//!   duration, and the one board it refuses.
//!
//! **What is not here**: a triggered mana ability (CR 605.1b), which is
//! critical-path item 6's; the definition already fixes what it will propose.

use std::collections::HashSet;
use std::sync::Arc;

use mtgsim::cards::phase5_pre_cards::dark_ritual;
use mtgsim::cards::phase_cm_cards::krark_clan_ironworks;
use mtgsim::cards::phase_re9_cards::{
    deep_water, doubling_cube, mana_reflection, nyxbloom_ancient, pale_moon,
};
use mtgsim::engine::actions::ActionContext;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{
    AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    forest, pass_turn, put_in_graveyard, put_in_hand, put_on_battlefield,
    setup_two_player_game, test_dp, vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::card_types::{CardType, Subtype};
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{
    AmountExpr, Effect, EffectRecipient, ManaOutput, ObjectFilter, ObjectSet, PlayerSet,
    Primitive,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::mana::{
    ManaAtom, ManaPersistence, ManaRestriction, ManaType, SpendContext, SpendPurpose,
};
use mtgsim::types::replacement::{
    EventPattern, GameActionTemplate, ReplacementDef, Rewrite, TemplateAmount,
};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

/// CR 616.1's choice between two applicable effects.
const PICK_REPLACEMENT: ChoiceKind = ChoiceKind::ChooseReplacementEffect { affected_object: None };

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A land with one mana ability, `{T}: <output>`. Invented, and named as one:
/// it wears no printed card's name (`engineering-practices.md` §3).
fn mana_land(name: &str, output: ManaOutput) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Land)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: new_ability_id(),
            ability_type: AbilityType::Mana,
            costs: vec![Cost::Tap],
            effect: Effect::Atom(Primitive::ProduceMana(output), EffectRecipient::Implicit),
        })
        .build()
}

/// One {G} spendable only on creature spells — CR 106.6's restriction on a
/// unit of mana.
fn creature_only_green() -> ManaAtom {
    ManaAtom {
        mana_type: ManaType::Green,
        source_id: None,
        restrictions: vec![ManaRestriction::OnlyForSpellTypes(vec![CardType::Creature])],
        grants: Vec::new(),
        persistence: ManaPersistence::Normal,
    }
}

/// "{T}: Add {G}. Spend this mana only to cast creature spells."
fn creature_grove() -> Arc<CardData> {
    mana_land("Creature Grove", ManaOutput { mana: Vec::new(), special: vec![creature_only_green()] })
}

/// "{T}: Add {W}{W}."
fn twin_plains() -> Arc<CardData> {
    mana_land(
        "Twin Plains",
        ManaOutput { mana: vec![(ManaType::White, AmountExpr::Fixed(2))], special: Vec::new() },
    )
}

/// "{T}: Add {W}{U}." — two types in one production, in this order.
fn tidal_meadow() -> Arc<CardData> {
    mana_land(
        "Tidal Meadow",
        ManaOutput {
            mana: vec![(ManaType::White, AmountExpr::Fixed(1)), (ManaType::Blue, AmountExpr::Fixed(1))],
            special: Vec::new(),
        },
    )
}

/// "{T}: Add {G}, and {G} spendable only on creature spells." — a production
/// that mixes a free unit with a restricted one, which no printed ability does.
fn half_bound_grove() -> Arc<CardData> {
    mana_land(
        "Half-Bound Grove",
        ManaOutput {
            mana: vec![(ManaType::Green, AmountExpr::Fixed(1))],
            special: vec![creature_only_green()],
        },
    )
}

/// A mana ability whose fixed amount is zero.
fn barren_waste() -> Arc<CardData> {
    mana_land(
        "Barren Waste",
        ManaOutput { mana: vec![(ManaType::Green, AmountExpr::Fixed(0))], special: Vec::new() },
    )
}

/// An artifact with "{T}: Add {C}." — a permanent tapped for mana that is not
/// a land.
fn plain_rock() -> Arc<CardData> {
    CardDataBuilder::new("Plain Rock")
        .card_type(CardType::Artifact)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: new_ability_id(),
            ability_type: AbilityType::Mana,
            costs: vec![Cost::Tap],
            effect: Effect::Atom(
                Primitive::ProduceMana(ManaOutput {
                    mana: vec![(ManaType::Colorless, AmountExpr::Fixed(1))],
                    special: Vec::new(),
                }),
                EffectRecipient::Implicit,
            ),
        })
        .build()
}

/// A static replacement effect as an enchantment, for the two fixtures below.
fn static_enchantment(name: &str, def: ReplacementDef) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Enchantment)
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(def)),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
        })
        .build()
}

/// Contamination's second line — "If a land is tapped for mana, it produces
/// {B} instead of any other type and amount" — without its upkeep trigger,
/// which is critical-path item 6's. A fixture under its own name for that
/// reason; the CR is the customer and the printed card is the test.
fn blackened_earth() -> Arc<CardData> {
    static_enchantment(
        "Blackened Earth",
        ReplacementDef::new(
            EventPattern::ProduceMana {
                tapped_for_mana: Some(true),
                source: Some(ObjectFilter::ByType(CardType::Land)),
            },
            ObjectSet::NO_OBJECTS,
            Rewrite::Instead(GameActionTemplate::ProduceMana {
                mana_type: ManaType::Black,
                amount: TemplateAmount::Fixed(1),
            }),
        )
        .affecting_players(PlayerSet::Everyone),
    )
}

/// Infernal Darkness's second line — "If a land is tapped for mana, it
/// produces {B} instead of any other type" — without its cumulative upkeep.
fn darkened_lands() -> Arc<CardData> {
    static_enchantment(
        "Darkened Lands",
        ReplacementDef::new(
            EventPattern::ProduceMana {
                tapped_for_mana: Some(true),
                source: Some(ObjectFilter::ByType(CardType::Land)),
            },
            ObjectSet::NO_OBJECTS,
            Rewrite::Instead(GameActionTemplate::ProduceMana {
                mana_type: ManaType::Black,
                amount: TemplateAmount::ReplacedAmount,
            }),
        )
        .affecting_players(PlayerSet::Everyone),
    )
}

/// Put `card` onto the battlefield under `player` and activate its first
/// mana ability, the way the priority loop would.
fn tap_for_mana(
    game: &mut GameState,
    card: Arc<CardData>,
    player: PlayerId,
    dp: &dyn DecisionProvider,
) -> Result<ObjectId, String> {
    let ability = card.abilities[0].id;
    let id = put_on_battlefield(game, card, player);
    game.activate_mana_ability(player, id, ability, &ActionContext::new(dp)).map(|_| id)
}

/// Resolve Dark Ritual for `player` — a spell adding mana (CR 605.5b), with
/// the card in the graveyard as a resolved instant is (CR 608.2m).
fn resolve_dark_ritual(game: &mut GameState, player: PlayerId, dp: &dyn DecisionProvider) -> ObjectId {
    let card = dark_ritual();
    let source = put_in_graveyard(game, card.clone(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        targets: Vec::new(),
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, dp).expect("resolving Dark Ritual");
    source
}

/// Put Deep Water onto the battlefield under `player` and resolve its
/// activated ability, the way the stack would. The `{U}` is skipped: every
/// board here is about the row the activation creates, and `Cost::Mana` is
/// exercised where mana is.
fn activate_deep_water(game: &mut GameState, player: PlayerId, dp: &dyn DecisionProvider) -> ObjectId {
    let card = deep_water();
    let id = put_on_battlefield(game, card.clone(), player);
    let ctx = ResolutionContext {
        source: id,
        ability_source: None,
        controller: player,
        targets: Vec::new(),
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, dp).expect("activating Deep Water");
    id
}

/// Every `ManaAdded` the game announced, as `(source, mana, tapped_for_mana)`.
fn mana_added(game: &GameState) -> Vec<(ObjectId, Vec<(ManaType, u64)>, bool)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::ManaAdded { source_id, mana, tapped_for_mana, .. } => {
                Some((*source_id, mana.clone(), *tapped_for_mana))
            }
            _ => None,
        })
        .collect()
}

fn pool(game: &GameState, player: PlayerId, mana_type: ManaType) -> u64 {
    game.players[player].mana_pool.amount(mana_type)
}

/// How much `mana_type` the player could spend on a spell of `card_type`,
/// restricted units included.
fn spendable_on(game: &GameState, player: PlayerId, mana_type: ManaType, card_type: CardType) -> u64 {
    let types: HashSet<CardType> = [card_type].into_iter().collect();
    let subtypes: HashSet<Subtype> = HashSet::new();
    let ctx = SpendContext {
        purpose: SpendPurpose::CastSpell { card_types: &types, subtypes: &subtypes, name: "Fixture" },
    };
    game.players[player].mana_pool.amount_for(mana_type, &ctx)
}

// ---------------------------------------------------------------------------
// The event — one performer, and what it announces
// ---------------------------------------------------------------------------

/// The performer announces what it added, by type, in the proposal's order,
/// and says the permanent was tapped for it.
#[test]
fn a_forest_tapped_for_mana_announces_one_green() {
    let mut game = setup_two_player_game();
    let id = tap_for_mana(&mut game, forest(), 0, &test_dp()).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Green), 1);
    assert_eq!(mana_added(&game), vec![(id, vec![(ManaType::Green, 1)], true)]);
}

/// A production of two types is announced in the order the ability lists
/// them — the `Vec` that replaced the event's `HashMap`, and the reason: the
/// log renders this line, and a map's order is the process's.
#[test]
fn a_two_type_production_is_announced_in_the_abilitys_order() {
    let mut game = setup_two_player_game();
    let id = tap_for_mana(&mut game, tidal_meadow(), 0, &test_dp()).unwrap();

    assert_eq!(
        mana_added(&game),
        vec![(id, vec![(ManaType::White, 1), (ManaType::Blue, 1)], true)]
    );
}

/// No rule makes a zero production no event, so it reaches the pipeline;
/// the performer then adds nothing, announces nothing and counts nothing.
#[test]
fn a_production_of_nothing_performs_and_announces_nothing() {
    let mut game = setup_two_player_game();
    let id = tap_for_mana(&mut game, barren_waste(), 0, &test_dp()).unwrap();

    assert!(game.battlefield.get(&id).unwrap().tapped, "the cost was paid");
    assert_eq!(game.players[0].mana_pool.total(), 0);
    assert!(mana_added(&game).is_empty());
    assert_eq!(game.counters.mana_productions(), 0);
}

/// CR 605.3b: the resolution is a step after the activation, so the tap
/// (the cost) and the production (the effect) are two events in two batches
/// — which is what CR 106.12a's "whenever a permanent is tapped for mana"
/// needs, since it triggers on the ability *resolving and producing mana*
/// and not on the permanent becoming tapped.
#[test]
fn the_tap_and_the_production_are_two_batches() {
    let mut game = setup_two_player_game();
    let id = tap_for_mana(&mut game, forest(), 0, &test_dp()).unwrap();

    let tapped = game
        .events
        .records()
        .iter()
        .find(|r| matches!(r.event, GameEvent::Tapped { object_id } if object_id == id))
        .expect("the Forest tapped");
    let produced = game
        .events
        .records()
        .iter()
        .find(|r| matches!(r.event, GameEvent::ManaAdded { .. }))
        .expect("the Forest produced");
    assert!(tapped.batch().is_some() && produced.batch().is_some());
    assert_ne!(tapped.batch(), produced.batch(), "the cost and the effect are two events");
}

/// The diagnostic row counts productions performed — one per tap, none for a
/// production of nothing.
#[test]
fn mana_productions_counts_performed_productions() {
    let mut game = setup_two_player_game();
    tap_for_mana(&mut game, forest(), 0, &test_dp()).unwrap();
    tap_for_mana(&mut game, forest(), 0, &test_dp()).unwrap();
    tap_for_mana(&mut game, barren_waste(), 0, &test_dp()).unwrap();

    assert_eq!(game.counters.mana_productions(), 2);
}

// ---------------------------------------------------------------------------
// CR 106.12 — "tapped for mana" is a definition, and the event carries it
// ---------------------------------------------------------------------------

/// CR 106.12: a permanent is "tapped for mana" only by a mana ability whose
/// activation cost includes {T}. The event carries that fact for the
/// triggers CR 106.12a describes: a Forest's production says `true`, Dark
/// Ritual's says `false` (CR 605.5b — a spell is never a mana ability).
///
/// Partial: the atom's *query* — "was this permanent tapped for mana this
/// turn" — is the trigger's, critical-path item 6; what this proves is that
/// the fact the query would read is on the performed event.
// COVERS-PARTIAL: ATOM-106.12a-001
#[test]
fn the_event_says_whether_the_permanent_was_tapped_for_mana() {
    let mut game = setup_two_player_game();
    let forest_id = tap_for_mana(&mut game, forest(), 0, &test_dp()).unwrap();
    let ritual = resolve_dark_ritual(&mut game, 0, &test_dp());

    assert_eq!(
        mana_added(&game),
        vec![
            (forest_id, vec![(ManaType::Green, 1)], true),
            (ritual, vec![(ManaType::Black, 3)], false),
        ]
    );
}

/// Mana Reflection's first ruling: "you're 'tapping a permanent for mana'
/// only if you're activating a mana ability of that permanent that includes
/// the {T} symbol in its cost". Dark Ritual is a spell, and adds three.
#[test]
fn dark_ritual_is_not_tapping_a_permanent_for_mana() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, mana_reflection(), 0);
    resolve_dark_ritual(&mut game, 0, &test_dp());

    assert_eq!(pool(&game, 0, ManaType::Black), 3, "a spell's production is not doubled");
}

/// The same ruling from the other side: Krark-Clan Ironworks' mana ability is
/// "Sacrifice an artifact: Add {C}{C}", with no {T} in its cost, so its
/// production is not "tapping a permanent for mana" either.
#[test]
fn a_mana_ability_without_a_tap_symbol_is_not_doubled() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, mana_reflection(), 0);
    // The only artifact is the Ironworks itself, so the sacrifice is forced
    // and the scripted provider is asked nothing.
    tap_for_mana(&mut game, krark_clan_ironworks(), 0, &ScriptedDecisionProvider::new()).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Colorless), 2, "no tap symbol, no doubling");
    assert_eq!(mana_added(&game).last().map(|(_, _, tapped)| *tapped), Some(false));
}

// ---------------------------------------------------------------------------
// CR 106.6a — the multipliers
// ---------------------------------------------------------------------------

/// Mana Reflection's text, on the pooled card: a Forest adds {G}{G}, in one
/// production (CR 614.6 — the modified event happens in place of the
/// original), with nobody asked anything.
#[test]
fn a_forest_under_mana_reflection_adds_two_green() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, mana_reflection(), 0);
    let dp = ScriptedDecisionProvider::new();
    let id = tap_for_mana(&mut game, forest(), 0, &dp).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Green), 2);
    assert_eq!(mana_added(&game), vec![(id, vec![(ManaType::Green, 2)], true)], "one event of two");
    assert!(dp.is_empty(), "one applicable effect asks nothing");
}

/// "If *you* tap a permanent for mana": an opponent's Reflection watches an
/// opponent's taps, and yours are yours.
#[test]
fn an_opponents_mana_reflection_does_not_double_your_lands() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, mana_reflection(), 1);
    tap_for_mana(&mut game, forest(), 0, &test_dp()).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Green), 1);
}

/// Mana Reflection's fourth ruling: "if you have two Mana Reflections on the
/// battlefield, you'll get four times the original amount". And the prompt
/// is the first assertion: two multipliers over one event commute, so
/// CR 616.1 has one outcome and asks nobody.
#[test]
fn two_mana_reflections_quadruple_with_no_prompt() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, mana_reflection(), 0);
    put_on_battlefield(&mut game, mana_reflection(), 0);
    let dp = ScriptedDecisionProvider::new();
    tap_for_mana(&mut game, forest(), 0, &dp).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Green), 4);
    assert!(dp.is_empty(), "two multipliers commute, so CR 616.1 has nothing to ask");
}

/// "If you have three, you'll get eight times the mana" — 2ⁿ, which two
/// copies alone cannot separate from "doubled once per copy after the first".
#[test]
fn three_mana_reflections_multiply_by_eight() {
    let mut game = setup_two_player_game();
    for _ in 0..3 {
        put_on_battlefield(&mut game, mana_reflection(), 0);
    }
    let dp = ScriptedDecisionProvider::new();
    tap_for_mana(&mut game, forest(), 0, &dp).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Green), 8);
    assert!(dp.is_empty());
}

/// Nyxbloom Ancient's own ruling: "if you have two Nyxbloom Ancients on the
/// battlefield, you'll get nine times the original amount".
#[test]
fn two_nyxbloom_ancients_produce_nine_times_the_mana() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, nyxbloom_ancient(), 0);
    put_on_battlefield(&mut game, nyxbloom_ancient(), 0);
    let dp = ScriptedDecisionProvider::new();
    tap_for_mana(&mut game, forest(), 0, &dp).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Green), 9);
    assert!(dp.is_empty());
}

/// The mixed pair the commuting cell was written for: a doubler beside a
/// tripler is six either way round, and still no prompt.
#[test]
fn a_reflection_and_an_ancient_produce_six_times_the_mana() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, mana_reflection(), 0);
    put_on_battlefield(&mut game, nyxbloom_ancient(), 0);
    let dp = ScriptedDecisionProvider::new();
    tap_for_mana(&mut game, twin_plains(), 0, &dp).unwrap();

    assert_eq!(pool(&game, 0, ManaType::White), 12, "two white, times six");
    assert!(dp.is_empty());
}

/// CR 106.6a: "any restrictions or additional effects created by the spell
/// or ability will apply to all mana produced". The atom's board — a land
/// whose {G} is spendable only on creature spells, under Mana Reflection —
/// produces two restricted units and no free one; and CR 106.6's own
/// sentence, that the restriction "doesn't affect the mana's type": both are
/// still green, and both pay a creature spell's {G}.
// COVERS: ATOM-106.6a-001, ATOM-106.6-001
#[test]
fn a_restricted_production_doubles_into_two_restricted_units() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, mana_reflection(), 0);
    let id = tap_for_mana(&mut game, creature_grove(), 0, &test_dp()).unwrap();

    let atoms = game.players[0].mana_pool.special_atoms();
    assert_eq!(atoms, &[(creature_only_green(), 2)], "two units, both restricted");
    assert_eq!(pool(&game, 0, ManaType::Green), 0, "and no free green");
    assert_eq!(spendable_on(&game, 0, ManaType::Green, CardType::Creature), 2);
    assert_eq!(spendable_on(&game, 0, ManaType::Green, CardType::Instant), 0);
    assert_eq!(
        mana_added(&game),
        vec![(id, vec![(ManaType::Green, 2)], true)],
        "the log reports the type, the pool keeps the restriction"
    );
}

/// The likeliest place for RE-9 to surprise, named in its brief: a mana
/// ability activated inside CR 601.2g's payment window proposes through the
/// same performer with the same replacement. One Forest under Mana
/// Reflection pays a {1}{G} creature; one Forest alone cannot.
#[test]
fn a_mana_ability_activated_in_the_payment_window_is_doubled() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, mana_reflection(), 0);
    put_on_battlefield(&mut game, forest(), 0);
    let bear = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
    let dp = RecordingDecisionProvider::picking(0);
    game.cast_spell(0, bear, &dp).expect("one Forest under Mana Reflection pays {1}{G}");
    assert!(
        dp.kinds().iter().any(|k| k.starts_with("ManaAbilityWindow")),
        "the Forest was tapped in the window: {:?}",
        dp.kinds()
    );
    assert_eq!(game.players[0].mana_pool.total(), 0, "both green paid the cost");

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, forest(), 0);
    let bear = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert!(
        game.cast_spell(0, bear, &RecordingDecisionProvider::picking(0)).is_err(),
        "one Forest alone is one mana short"
    );
}

// ---------------------------------------------------------------------------
// CR 106.12b — "of a specific type": the retype
// ---------------------------------------------------------------------------

/// Deep Water's first ruling: "the amount of mana produced is unchanged, but
/// it will all be {U}".
#[test]
fn deep_water_keeps_the_amount_and_changes_the_type() {
    let mut game = setup_two_player_game();
    activate_deep_water(&mut game, 0, &test_dp());
    let id = tap_for_mana(&mut game, twin_plains(), 0, &test_dp()).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Blue), 2);
    assert_eq!(pool(&game, 0, ManaType::White), 0);
    assert_eq!(mana_added(&game), vec![(id, vec![(ManaType::Blue, 2)], true)]);
}

/// Infernal Darkness's ruling, on its line as a fixture: "if a land that's
/// tapped for mana would add {W}{W}, it adds {B}{B} instead" — and "a land"
/// is any player's, so the opponent's Twin Plains is black too.
#[test]
fn infernal_darkness_line_turns_two_white_into_two_black() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, darkened_lands(), 0);
    tap_for_mana(&mut game, twin_plains(), 1, &test_dp()).unwrap();

    assert_eq!(pool(&game, 1, ManaType::Black), 2);
    assert_eq!(pool(&game, 1, ManaType::White), 0);
}

/// CR 106.6: a restriction "doesn't affect the mana's type", and the converse
/// — retyping a unit does not drop the restriction the ability put on it.
#[test]
fn a_retyped_restricted_unit_keeps_its_restriction() {
    let mut game = setup_two_player_game();
    activate_deep_water(&mut game, 0, &test_dp());
    tap_for_mana(&mut game, creature_grove(), 0, &test_dp()).unwrap();

    let expected = ManaAtom { mana_type: ManaType::Blue, ..creature_only_green() };
    assert_eq!(game.players[0].mana_pool.special_atoms(), &[(expected, 1)]);
    assert_eq!(spendable_on(&game, 0, ManaType::Blue, CardType::Creature), 1);
    assert_eq!(spendable_on(&game, 0, ManaType::Blue, CardType::Instant), 0);
}

/// "A land you control": the pattern's filter is asked of the permanent, so
/// an artifact tapped for mana is left alone.
#[test]
fn deep_water_leaves_a_permanent_that_is_not_a_land_alone() {
    let mut game = setup_two_player_game();
    activate_deep_water(&mut game, 0, &test_dp());
    tap_for_mana(&mut game, plain_rock(), 0, &test_dp()).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Colorless), 1);
    assert_eq!(pool(&game, 0, ManaType::Blue), 0);
}

/// "A land *you* control", and "if *you* tap": an opponent's Forest, tapped
/// by the opponent, is neither.
#[test]
fn deep_water_leaves_an_opponents_land_alone() {
    let mut game = setup_two_player_game();
    activate_deep_water(&mut game, 0, &test_dp());
    tap_for_mana(&mut game, forest(), 1, &test_dp()).unwrap();

    assert_eq!(pool(&game, 1, ManaType::Green), 1);
}

/// Deep Water's second ruling: "affects lands you control when it resolves
/// and any lands you gain control of this turn". The filter is evaluated at
/// each production, not captured when the row is made, so a land that
/// arrives after the activation is retyped too.
#[test]
fn deep_water_reaches_a_land_gained_after_the_activation() {
    let mut game = setup_two_player_game();
    activate_deep_water(&mut game, 0, &test_dp());
    tap_for_mana(&mut game, forest(), 0, &test_dp()).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Blue), 1);
    assert_eq!(pool(&game, 0, ManaType::Green), 0);
}

/// "Until end of turn": the row is gone with the cleanup step.
#[test]
fn deep_water_expires_at_end_of_turn() {
    let mut game = setup_two_player_game();
    activate_deep_water(&mut game, 0, &test_dp());
    pass_turn(&mut game);
    assert!(game.replacement_effects.is_empty(), "the turn is over");
    tap_for_mana(&mut game, forest(), 0, &test_dp()).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Green), 1);
}

/// Retype-then-double and double-then-retype are one event, so beside Mana
/// Reflection the affected player is asked nothing and gets {U}{U}.
#[test]
fn deep_water_and_mana_reflection_commute() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, mana_reflection(), 0);
    activate_deep_water(&mut game, 0, &test_dp());
    let dp = ScriptedDecisionProvider::new();
    tap_for_mana(&mut game, forest(), 0, &dp).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Blue), 2);
    assert!(dp.is_empty(), "a retype at the replaced amount commutes with a multiplier");
}

/// Contamination's line — "{B} instead of any other type *and amount*" —
/// beside Mana Reflection is CR 616.1's real question: doubled first is one
/// black, fixed first is two. The tapping player chooses, and the
/// candidates are offered in timestamp order, the Reflection's first.
#[test]
fn contamination_line_beside_mana_reflection_is_a_real_choice() {
    let doubled_first = {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, mana_reflection(), 0);
        put_on_battlefield(&mut game, blackened_earth(), 0);
        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(PICK_REPLACEMENT, vec![0]);
        tap_for_mana(&mut game, forest(), 0, &dp).unwrap();
        assert!(dp.is_empty(), "one prompt");
        pool(&game, 0, ManaType::Black)
    };
    let fixed_first = {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, mana_reflection(), 0);
        put_on_battlefield(&mut game, blackened_earth(), 0);
        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(PICK_REPLACEMENT, vec![1]);
        tap_for_mana(&mut game, forest(), 0, &dp).unwrap();
        assert!(dp.is_empty(), "one prompt");
        pool(&game, 0, ManaType::Black)
    };

    assert_eq!(doubled_first, 1, "{{G}}{{G}}, then one black instead of any type and amount");
    assert_eq!(fixed_first, 2, "one black, then doubled");
}

/// The one board the retype refuses rather than guesses: a production that
/// mixes a free unit with a restricted one, set to a fixed amount, has no
/// rule saying which restriction the new mana carries — and no printed
/// mana ability produces such a mix. Loud, over a silently dropped
/// restriction.
#[test]
fn a_mixed_production_under_a_fixed_retype_is_refused() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, blackened_earth(), 0);
    let err = tap_for_mana(&mut game, half_bound_grove(), 0, &test_dp()).unwrap_err();

    assert!(err.contains("units disagree about their restrictions"), "{err}");
}

/// The same mixed production under a retype that keeps the amount is not a
/// question at all: every unit keeps what it carried, with its type changed.
#[test]
fn a_mixed_production_under_a_replaced_amount_retype_keeps_every_unit() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, darkened_lands(), 0);
    tap_for_mana(&mut game, half_bound_grove(), 0, &test_dp()).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Black), 1, "the free unit, retyped");
    let expected = ManaAtom { mana_type: ManaType::Black, ..creature_only_green() };
    assert_eq!(game.players[0].mana_pool.special_atoms(), &[(expected, 1)]);
}

/// Pale Moon's ruling — "does not change the amount of mana produced, only
/// the color" — on "a player" (any player) and "a nonbasic land" (the
/// filter): the opponent's two-mana nonbasic land produces two colorless,
/// and a basic Forest is left alone.
#[test]
fn pale_moon_retypes_any_players_nonbasic_land_and_leaves_a_basic_alone() {
    let mut game = setup_two_player_game();
    let card = pale_moon();
    let source = put_in_graveyard(&mut game, card.clone(), 0);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 0,
        targets: Vec::new(),
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).expect("resolving Pale Moon");

    tap_for_mana(&mut game, twin_plains(), 1, &test_dp()).unwrap();
    assert_eq!(pool(&game, 1, ManaType::Colorless), 2, "the opponent's nonbasic land, retyped");
    assert_eq!(pool(&game, 1, ManaType::White), 0);

    tap_for_mana(&mut game, forest(), 0, &test_dp()).unwrap();
    assert_eq!(pool(&game, 0, ManaType::Green), 1, "a basic land is not nonbasic");
}

// ---------------------------------------------------------------------------
// Doubling Cube — CR 106.6's integration test, and {T} on a non-land
// ---------------------------------------------------------------------------

/// Doubling Cube's third ruling, on its own board: "{C}{W}{W}{B} with no
/// restrictions and {U}{U}{U} that can be used only to cast artifact spells"
/// becomes "{C}{C}{W}{W}{W}{W}{B}{B}, {U}{U}{U} … only to cast artifact
/// spells, and {U}{U}{U} that can be used for anything". The restricted
/// units are counted by their type (CR 106.6) and the copies carry no
/// restriction.
///
/// The {3} is paid from the free mana first, and which three the planner
/// takes is its choice, so the free types are asserted against what the
/// event says was added — each type doubled from what remained — and the
/// blue against the ruling's numbers, since restricted blue cannot pay an
/// ability's cost and so is never spent.
#[test]
fn doubling_cube_counts_restricted_mana_and_copies_it_unrestricted() {
    let mut game = setup_two_player_game();
    let artifact_only_blue = ManaAtom {
        mana_type: ManaType::Blue,
        source_id: None,
        restrictions: vec![ManaRestriction::OnlyForSpellTypes(vec![CardType::Artifact])],
        grants: Vec::new(),
        persistence: ManaPersistence::Normal,
    };
    {
        let p = &mut game.players[0].mana_pool;
        p.add(ManaType::Colorless, 1 + 3);
        p.add(ManaType::White, 2);
        p.add(ManaType::Black, 1);
        for _ in 0..3 {
            p.add_special(artifact_only_blue.clone());
        }
    }
    let dp = RecordingDecisionProvider::picking(0);
    let cube = tap_for_mana(&mut game, doubling_cube(), 0, &dp).unwrap();

    let (source, added, tapped) = mana_added(&game).pop().expect("the Cube produced");
    assert_eq!(source, cube);
    assert!(tapped, "a mana ability with {{T}} in its cost");
    let added_of = |t: ManaType| added.iter().find(|(x, _)| *x == t).map(|(_, n)| *n).unwrap_or(0);
    for t in [ManaType::Colorless, ManaType::White, ManaType::Black] {
        assert_eq!(pool(&game, 0, t), 2 * added_of(t), "{t:?}: what remained after {{3}}, doubled");
    }
    assert_eq!(
        added_of(ManaType::Colorless) + added_of(ManaType::White) + added_of(ManaType::Black),
        4,
        "seven free mana, three paid, four doubled"
    );
    assert_eq!(added_of(ManaType::Blue), 3, "three restricted blue count as three blue");
    assert_eq!(pool(&game, 0, ManaType::Blue), 3, "and the copies are free");
    assert_eq!(
        game.players[0].mana_pool.special_atoms(),
        &[(artifact_only_blue, 3)],
        "the restricted three are untouched"
    );
}

/// Doubling Cube's first ruling — "Doubling Cube's ability is a mana
/// ability" — with {T} in its cost, so by CR 106.12 it is tapped for mana and
/// Mana Reflection doubles the doubling: five green, three paid, the Cube
/// produces the two that remained, Mana Reflection makes that production
/// four, and the pool ends at six.
#[test]
fn doubling_cube_is_tapped_for_mana_so_mana_reflection_doubles_its_doubling() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, mana_reflection(), 0);
    game.players[0].mana_pool.add(ManaType::Green, 5);
    let dp = RecordingDecisionProvider::picking(0);
    let cube = tap_for_mana(&mut game, doubling_cube(), 0, &dp).unwrap();

    assert_eq!(pool(&game, 0, ManaType::Green), 6, "two remained, produced twice over");
    assert_eq!(
        mana_added(&game).pop(),
        Some((cube, vec![(ManaType::Green, 4)], true)),
        "one production of four, tapped for mana"
    );
}
