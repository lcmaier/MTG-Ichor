//! Phase RC-4 integration tests: the CR 614.12 look-ahead frame.
//!
//! RC-3 settled *membership* — which effects reach an entering permanent, in
//! both directions. RC-4 is the frame those effects are evaluated against: the
//! permanent "as it would exist on the battlefield", with the replacements
//! already applied to it (clause 1), its own static abilities (clause 2), and
//! the effects that already exist (clause 3). Every test here builds a board
//! where the frame and the card disagree, and asserts the frame's answer.
//!
//! The fixtures are named for what they are. Two of the three consumers
//! `replacement-architecture.md` §9 named for this phase — Grist and the
//! Theros gods — are `Effect::Conditional` statics, which the lowering cannot
//! express (Deferred Migrations item 7f), so clause (2) is exercised with an
//! anthem creature instead and §5a's count boundary with Keldon Warlord, a
//! printed card whose CDA counts the battlefield.

use std::sync::Arc;

use mtgsim::cards::artifacts::sol_ring;
use mtgsim::cards::keyword_creatures::wall_of_stone;
use mtgsim::cards::phase_ld_cards::{blood_moon, march_of_the_machines};
use mtgsim::cards::phase_lf_cards::humility;
use mtgsim::cards::phase_rc_cards::{
    adaptive_shimmerer, chainbreaker, containment_priest, dryad_arbor, keldon_warlord, root_maze,
};
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeCause};
use mtgsim::engine::layers::compute_as_entering;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{AbilityType, CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_power, get_effective_subtypes, get_effective_toughness,
    has_summoning_sickness, is_creature,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    creature_with_ability, put_in_graveyard, put_in_hand, put_on_battlefield, setup_game,
    setup_two_player_game, static_ability, test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::{CardType, CreatureType, LandType, Subtype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AffectedSet, AmountExpr, CounterType, Duration, Effect, EffectRecipient, PermanentFilter,
    PlayerRef, Primitive, TypeChange,
};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::{EnterMods, EventPattern, ReplacementDef, Rewrite};
use mtgsim::types::restriction::{Restriction, RestrictionDef};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// Helpers and fixtures
// ---------------------------------------------------------------------------

/// Every `PermanentEnteredBattlefield` in the log, as `(object, controller)`.
///
/// Fixtures placed with `put_on_battlefield` announce themselves too, so a
/// test about one card filters by id — `entries_of`.
fn entries(game: &GameState) -> Vec<(ObjectId, usize)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::PermanentEnteredBattlefield { object_id, controller } => {
                Some((*object_id, *controller))
            }
            _ => None,
        })
        .collect()
}

/// The controllers `id` was announced as entering under — one per entry.
fn entries_of(game: &GameState, id: ObjectId) -> Vec<usize> {
    entries(game).into_iter().filter(|(o, _)| *o == id).map(|(_, c)| c).collect()
}

/// Put `card` onto the battlefield from the graveyard through the chokepoint,
/// so the CR 616.1 loop runs. Reanimation's shape (`phase_rc_integration_test`).
fn reanimate(game: &mut GameState, card: Arc<CardData>, player: usize) -> ObjectId {
    let id = put_in_graveyard(game, card, player);
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx())
        .expect("it enters");
    id
}

/// "[filter] enter tapped" — Kismet's shape over an arbitrary filter, as a
/// white enchantment.
fn enter_tapped_when(name: &str, filter: PermanentFilter) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            AffectedSet::Filter { filter },
            Rewrite::EnterWith(EnterMods::tapped()),
        )))))
        .build()
}

/// "Creatures with power N or less enter tapped." No printed card says this;
/// `PermanentFilter::PowerLE` is the one leaf that reads what the frame
/// changes — counters (CR 122.1a) and the entering object's own anthem — so it
/// is the fixture that can see the frame at all.
fn small_creatures_enter_tapped(n: i32) -> Arc<CardData> {
    enter_tapped_when(
        "Small creatures enter tapped",
        PermanentFilter::And(
            Box::new(PermanentFilter::ByType(CardType::Creature)),
            Box::new(PermanentFilter::PowerLE(n)),
        ),
    )
}

/// Kismet's second clause: "Permanents your opponents control enter tapped."
fn kismet_shaped() -> Arc<CardData> {
    enter_tapped_when("Kismet-shaped", PermanentFilter::ByController(PlayerRef::Opponent))
}

/// A 2/2 with "Creatures you control get +1/+1" — self-including, unlike
/// every printed lord since Lorwyn, which says "other". CR 614.12 clause (2)
/// with no `Effect::Conditional` in the way.
fn anthem_bear() -> Arc<CardData> {
    creature_with_ability(
        "Anthem Bear",
        2,
        2,
        static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(
                AmountExpr::Fixed(1),
                AmountExpr::Fixed(1),
                Duration::WhileSourceOnBattlefield,
            ),
            EffectRecipient::FilteredPermanents(PermanentFilter::And(
                Box::new(PermanentFilter::ByType(CardType::Creature)),
                Box::new(PermanentFilter::ByController(PlayerRef::You)),
            )),
        )),
    )
}

/// A castable 2/2: one colored symbol and no generic, so `cast_spell` asks
/// nothing about mana.
fn castable_bear(name: &str, color: ManaType) -> CardDataBuilder {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[color], 0))
        .color(match color {
            ManaType::Black => Color::Black,
            ManaType::White => Color::White,
            _ => Color::Green,
        })
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Bear))
        .power_toughness(2, 2)
}

/// "This creature enters under an opponent's control." — Xantcha's first
/// sentence alone, as a black 2/2.
fn defector() -> Arc<CardData> {
    castable_bear("Defector", ManaType::Black)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            AffectedSet::SourceOnly,
            Rewrite::EnterUnderControlOf(PlayerRef::Opponent),
        )))))
        .build()
}

/// The Defector with a second entry replacement, "enters with two +1/+1
/// counters", so CR 616.1b has something to be ordered ahead of.
fn defector_with_counters() -> Arc<CardData> {
    castable_bear("Defector with counters", ManaType::Black)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            AffectedSet::SourceOnly,
            Rewrite::EnterUnderControlOf(PlayerRef::Opponent),
        )))))
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            AffectedSet::SourceOnly,
            Rewrite::EnterWith(EnterMods::with_counters(CounterType::PlusOnePlusOne, 2)),
        )))))
        .build()
}

/// An enchantment carrying one static "can't".
fn static_restriction(name: &str, what: Restriction) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Restriction(Box::new(RestrictionDef::new(what)))))
        .build()
}

/// "Lands can't enter the battlefield." — Worms of the Earth's second sentence,
/// in the zone-change shape. The entry is the zone change (RC-4b), so this is
/// asked of the entry proposal, before anything moves;
/// `phase_rc4b_integration_test` writes the same restriction against
/// `EventPattern::EnterBattlefield` directly.
fn lands_cant_enter() -> Restriction {
    Restriction::Event {
        pattern: EventPattern::ZoneChange {
            from: None,
            to: Some(Zone::Battlefield),
            cause: None,
            object: None,
        },
        affected: AffectedSet::Filter { filter: PermanentFilter::ByType(CardType::Land) },
        by: None,
    }
}

/// "Creatures you control can't have -1/-1 counters put on them." — Melira,
/// Sylvok Outcast's second sentence and Darksteel Angel's third, alone.
fn no_minus_counters_on_your_creatures() -> Restriction {
    Restriction::Event {
        pattern: EventPattern::CounterChange {
            counter: Some(CounterType::MinusOneMinusOne),
            adding: true,
        },
        affected: AffectedSet::Filter {
            filter: PermanentFilter::And(
                Box::new(PermanentFilter::ByType(CardType::Creature)),
                Box::new(PermanentFilter::ByController(PlayerRef::You)),
            ),
        },
        by: None,
    }
}

/// A Layer 4 effect over every permanent.
fn every_permanent(name: &str, change: TypeChange) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeType(change, Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(PermanentFilter::All),
        )))
        .build()
}

fn type_change(add: Vec<CardType>, remove: Vec<CardType>) -> TypeChange {
    TypeChange {
        add_types: add,
        remove_types: remove,
        set_types: None,
        add_subtypes: Vec::new(),
        remove_subtypes: Vec::new(),
        set_subtypes: None,
        add_supertypes: Vec::new(),
        remove_supertypes: Vec::new(),
        set_supertypes: None,
    }
}

/// "Each permanent is a land in addition to its other types." No printed card;
/// the shape is Mycosynth Lattice's with the type changed.
fn everything_is_a_land() -> Arc<CardData> {
    every_permanent("Everything is a land", type_change(vec![CardType::Land], Vec::new()))
}

/// "Permanents aren't planeswalkers." No printed card — the direction RC-3's
/// membership gate changed for CR 306.5b, and the only one a test can reach.
fn nothing_is_a_planeswalker() -> Arc<CardData> {
    every_permanent(
        "Nothing is a planeswalker",
        type_change(Vec::new(), vec![CardType::Planeswalker]),
    )
}

/// A blue planeswalker with printed loyalty 4, castable for {U}.
fn planeswalker() -> Arc<CardData> {
    CardDataBuilder::new("Jace, the Mind Sculptor")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .color(Color::Blue)
        .card_type(CardType::Planeswalker)
        .loyalty(4)
        .build()
}

fn cast_and_resolve(game: &mut GameState, player: usize, card: Arc<CardData>, mana: ManaType) -> ObjectId {
    let id = put_in_hand(game, card, player);
    game.players[player].mana_pool.add(mana, 1);
    game.cast_spell(player, id, &test_dp()).expect("it is castable");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    id
}

// ---------------------------------------------------------------------------
// RC-3's slip, corrected: CR 306.5b reads the frame
// ---------------------------------------------------------------------------

/// The direction the membership gate changed for CR 306.5b — the
/// `codebase-state.md` ledger had it backwards. A planeswalker card whose type
/// a filter-scoped Layer 4 effect removes is not a planeswalker as it would
/// exist on the battlefield, so it has no intrinsic loyalty ability and enters
/// with no loyalty counters. (The other direction reads *printed* loyalty,
/// which a non-planeswalker has none of, and changed nothing.)
#[test]
fn test_a_planeswalker_whose_type_a_filter_effect_removes_enters_with_no_loyalty() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, nothing_is_a_planeswalker(), 1);

    let jace = cast_and_resolve(&mut game, 0, planeswalker(), ManaType::Blue);

    assert_eq!(
        game.battlefield.get(&jace).unwrap().counter_count(CounterType::Loyalty),
        0,
        "CR 306.5b gives the ability to a planeswalker, and on this battlefield it is not one"
    );
}

// ---------------------------------------------------------------------------
// CR 614.12 clause (1) — the replacements already applied are in the frame
// ---------------------------------------------------------------------------

/// Chainbreaker is a 3/3 that enters with two -1/-1 counters. "Creatures with
/// power 1 or less enter tapped" does not match a 3/3, so on the first
/// iteration only the counters apply; on the next, CR 616.1f re-gathers
/// against a proposal that carries them, the frame reads 1/1, and the tapped
/// half applies. No prompt at any point, and the order is forced by the rule
/// rather than chosen.
#[test]
fn test_a_replacement_already_applied_is_read_by_the_next_ones_filter() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, small_creatures_enter_tapped(1), 1);

    let scarecrow = reanimate(&mut game, chainbreaker(), 0);

    let entry = game.battlefield.get(&scarecrow).unwrap();
    assert_eq!(entry.counter_count(CounterType::MinusOneMinusOne), 2);
    assert!(entry.tapped, "1/1 in the frame once its own counters are on the proposal");

    // A 3/3 that enters with nothing is 3/3 in the frame and enters untapped.
    let bears = reanimate(&mut game, vanilla_creature(3, 3, &[]), 0);
    assert!(!game.battlefield.get(&bears).unwrap().tapped);
}

// ---------------------------------------------------------------------------
// CR 614.12 clause (2) — the permanent's own static abilities
// ---------------------------------------------------------------------------

/// A 2/2 whose own anthem buffs creatures it controls, itself included, is 3/3
/// as it would exist on the battlefield — so "creatures with power 2 or less
/// enter tapped" does not apply to it, though it applies to a vanilla 2/2.
#[test]
fn test_an_entering_creatures_own_anthem_is_in_its_own_frame() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, small_creatures_enter_tapped(2), 1);

    // The control first, while no anthem is on the board: a plain 2/2 is 2/2
    // in its frame and enters tapped.
    let bears = reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert!(game.battlefield.get(&bears).unwrap().tapped);

    let anthem = reanimate(&mut game, anthem_bear(), 0);
    assert!(
        !game.battlefield.get(&anthem).unwrap().tapped,
        "CR 614.12 clause (2): its own +1/+1 is in the frame, so it is 3/3 to the filter"
    );
    assert_eq!(get_effective_power(&game, anthem), Some(3), "and 3/3 once it is there");
    assert_eq!(get_effective_power(&game, bears), Some(3), "the row is registered now and reaches the bear too");
}

/// The would-be row is subject to CR 613.7a like a registered one: Humility
/// strips the Anthem Bear's ability at layer 6, so its anthem does not exist by
/// the time layer 7c would apply it, and the frame is Humility's 1/1.
#[test]
fn test_humility_strips_the_entering_creatures_own_anthem_before_it_applies() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, humility(), 1);
    put_on_battlefield(&mut game, small_creatures_enter_tapped(2), 1);

    let anthem = reanimate(&mut game, anthem_bear(), 0);
    assert!(
        game.battlefield.get(&anthem).unwrap().tapped,
        "1/1 in the frame: Humility took the anthem away at layer 6 (CR 613.7a)"
    );
}

/// Clause (2) via the API: an object anywhere, computed as entering.
#[test]
fn test_compute_as_entering_seeds_the_proposed_controller() {
    let mut game = setup_two_player_game();
    let bears = put_in_graveyard(&mut game, vanilla_creature(2, 2, &[]), 0);

    let as_p1 = compute_as_entering(&game, bears, 1, &EnterMods::NONE).unwrap();
    assert_eq!(as_p1.controller, 1, "the proposed controller, not the owner");
    assert_eq!(
        mtgsim::engine::layers::compute_characteristics(&game, bears).unwrap().controller,
        0,
        "and the real object is untouched"
    );
}

// ---------------------------------------------------------------------------
// §5a — visible to filters, invisible to counts
// ---------------------------------------------------------------------------

/// Keldon Warlord counts the non-Wall creatures its controller controls,
/// itself included, and the count is a CDA that Humility removes.
#[test]
fn test_keldon_warlord_counts_non_wall_creatures_you_control() {
    let mut game = setup_two_player_game();
    let (you, opponent) = (0, 1);
    let warlord = put_on_battlefield(&mut game, keldon_warlord(), you);
    assert_eq!(get_effective_power(&game, warlord), Some(1), "it counts itself");

    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), you);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), you);
    put_on_battlefield(&mut game, wall_of_stone(), you);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), opponent);

    assert_eq!(
        get_effective_power(&game, warlord),
        Some(3),
        "itself and your two bears; not your Wall, not the opponent's bear"
    );
    assert_eq!(get_effective_toughness(&game, warlord), Some(3));

    put_on_battlefield(&mut game, humility(), opponent);
    assert_eq!(
        get_effective_power(&game, warlord),
        Some(1),
        "Humility strips the CDA at layer 6 and sets 1/1 at 7b"
    );
}

/// The count half of the Thassa boundary. As the Warlord enters beside two
/// other creatures it is 2/2 — the count runs over the battlefield, which it
/// is not on — so "creatures with power 2 or less enter tapped" applies. A
/// moment later it is one of the creatures it counts and is 3/3.
#[test]
fn test_an_entering_count_does_not_include_the_entering_object() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, small_creatures_enter_tapped(2), 1);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    let warlord = reanimate(&mut game, keldon_warlord(), 0);

    assert!(
        game.battlefield.get(&warlord).unwrap().tapped,
        "2/2 in the look-ahead frame: it counted the two bears and not itself"
    );
    assert_eq!(get_effective_power(&game, warlord), Some(3), "3/3 once it is there");
}

// ---------------------------------------------------------------------------
// CR 614.12 clause (3) through a filter-scoped replacement — Containment Priest
// ---------------------------------------------------------------------------

/// "If a nontoken creature would enter and it wasn't cast, exile it instead."
/// A returned creature card never becomes a permanent: no entry is announced
/// and the card is in exile.
///
/// Chainbreaker brings its own "enters with two -1/-1 counters", so the bucket
/// is an `Instead` beside an `EnterWith` — the shape that keeps CR 616.1
/// asking (`test_a_non_commuting_entry_replacement_keeps_cr_616_1_asking`) —
/// and the choice is scripted. Both answers exile it.
#[test]
fn test_containment_priest_exiles_a_creature_that_was_not_cast() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, containment_priest(), 1);

    let dp = ScriptedDecisionProvider::new();
    let scarecrow = put_in_graveyard(&mut game, chainbreaker(), 0);
    dp.expect_pick_n(
        ChoiceKind::ChooseReplacementEffect { affected_object: Some(scarecrow) },
        vec![0],
    );
    game.change_zone(scarecrow, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("the zone change happens; the entry is what gets replaced");
    assert!(dp.is_empty());

    assert!(game.exile.contains(&scarecrow));
    assert!(!game.battlefield.contains_key(&scarecrow));
    assert_eq!(game.get_object(scarecrow).unwrap().zone, Zone::Exile);
    assert!(
        entries(&game).iter().all(|(id, _)| *id != scarecrow),
        "it never entered, so nothing announced an entry"
    );
}

/// A creature spell resolving was cast, and the Priest does not match it.
#[test]
fn test_containment_priest_ignores_a_creature_that_was_cast() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, containment_priest(), 1);

    let bears = cast_and_resolve(&mut game, 0, castable_bear("Cast Bear", ManaType::Green).build(), ManaType::Green);

    assert!(game.battlefield.contains_key(&bears));
    assert_eq!(entries_of(&game, bears), vec![0]);
}

/// "Nontoken": a token entering under the Priest is not exiled.
#[test]
fn test_containment_priest_ignores_tokens() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, containment_priest(), 1);

    let mut token = GameObject::new(vanilla_creature(1, 1, &[]), 0, Zone::Battlefield);
    token.is_token = true;
    let id = token.id;
    game.add_object(token);
    game.execute_action(
        GameAction::EnterBattlefield {
            object: id,
            from: None,
            controller: 0,
            mods: EnterMods::NONE,
            cause: None,
        },
        &ActionContext::new(&test_dp()),
    )
    .unwrap();

    assert!(game.battlefield.contains_key(&id));
}

/// The frame, on the Priest's own filter. Sol Ring is not a creature card, but
/// under March of the Machines it is an artifact creature *as it would exist on
/// the battlefield* — and CR 614.12 says that is what the Priest checks.
#[test]
fn test_containment_priest_reads_the_look_ahead_frame() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, containment_priest(), 1);
    put_on_battlefield(&mut game, march_of_the_machines(), 1);

    let ring = reanimate(&mut game, sol_ring(), 0);
    assert!(game.exile.contains(&ring), "a creature on the battlefield it would be, so exiled");

    // Without March it is what its card says, and the Priest lets it through.
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, containment_priest(), 1);
    let ring = reanimate(&mut game, sol_ring(), 0);
    assert!(game.battlefield.contains_key(&ring));
}

/// The road a fuzz game has to a creature that "wasn't cast": a land drop.
/// Dryad Arbor played as a land is exiled, and the land drop was still used.
#[test]
fn test_dryad_arbor_played_as_a_land_is_exiled_by_containment_priest() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, containment_priest(), 1);

    let arbor = put_in_hand(&mut game, dryad_arbor(), 0);
    game.play_land(0, arbor, Zone::Hand, &test_ctx()).expect("the land drop is legal");

    assert!(game.exile.contains(&arbor));
    assert!(!game.battlefield.contains_key(&arbor));
    assert_eq!(game.players[0].lands_played_this_turn, 1, "the land was played; it just never arrived");
}

// ---------------------------------------------------------------------------
// §11 item 19 — the prompt that is real, beside the one that is not
// ---------------------------------------------------------------------------

/// Root Maze taps an entering land; Containment Priest exiles an entering
/// creature that wasn't cast; Dryad Arbor played is both. Two candidates, one
/// an `EnterWith` and one an `Instead`, so `order_invariant_entry_bucket` does
/// not apply and CR 616.1 asks the land's controller. Either answer exiles
/// it — but which CR 614.5 slot is spent first is the event log's business,
/// and the branch this keeps alive is the one RB shipped dead with Kalitas.
// COVERS-PARTIAL: ATOM-616.1-001
#[test]
fn test_a_non_commuting_entry_replacement_keeps_cr_616_1_asking() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, root_maze(), 1);
    put_on_battlefield(&mut game, containment_priest(), 1);

    let dp = ScriptedDecisionProvider::new();
    let arbor = put_in_hand(&mut game, dryad_arbor(), 0);
    dp.expect_pick_n(
        ChoiceKind::ChooseReplacementEffect { affected_object: Some(arbor) },
        vec![0],
    );
    game.play_land(0, arbor, Zone::Hand, &ActionContext::new(&dp)).unwrap();

    assert!(dp.is_empty(), "CR 616.1 asked, and the answer was consumed");
    assert!(game.exile.contains(&arbor));
}

/// The other index is a real candidate: Root Maze's row is gathered by the
/// battlefield sweep ahead of the Priest's, and choosing the Priest first
/// exiles the Arbor before Root Maze's slot is spent.
#[test]
fn test_the_other_cr_616_1_order_is_available_where_the_order_matters() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, root_maze(), 1);
    put_on_battlefield(&mut game, containment_priest(), 1);

    let dp = ScriptedDecisionProvider::new();
    let arbor = put_in_hand(&mut game, dryad_arbor(), 0);
    dp.expect_pick_n(
        ChoiceKind::ChooseReplacementEffect { affected_object: Some(arbor) },
        vec![1],
    );
    game.play_land(0, arbor, Zone::Hand, &ActionContext::new(&dp)).unwrap();

    assert!(dp.is_empty(), "the second candidate is a real one");
    assert!(game.exile.contains(&arbor));
}

/// The prompt the frame makes real. Adaptive Shimmerer is a 0/0 that enters
/// with three +1/+1 counters; "creatures with power 1 or less enter tapped"
/// matches it at 0/0 and not at 3/3. Both apply at the first iteration, and
/// the order decides whether it enters tapped — so `PowerLE` is not an
/// invariant leaf, and CR 616.1 asks.
#[test]
fn test_a_power_filter_beside_counters_is_a_real_choice() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, small_creatures_enter_tapped(1), 1);

    // Counters first: 3/3 by the next iteration, the tapped half never applies.
    let dp = ScriptedDecisionProvider::new();
    let shimmerer = put_in_graveyard(&mut game, adaptive_shimmerer(), 0);
    dp.expect_pick_n(
        ChoiceKind::ChooseReplacementEffect { affected_object: Some(shimmerer) },
        vec![1],
    );
    game.change_zone(shimmerer, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .unwrap();
    assert!(dp.is_empty());
    let entry = game.battlefield.get(&shimmerer).unwrap();
    assert_eq!(entry.counter_count(CounterType::PlusOnePlusOne), 3);
    assert!(!entry.tapped, "counters first: it was 3/3 when the tapped half looked");

    // Tapped first: it applies to the 0/0, then the counters.
    let dp = ScriptedDecisionProvider::new();
    let shimmerer = put_in_graveyard(&mut game, adaptive_shimmerer(), 0);
    dp.expect_pick_n(
        ChoiceKind::ChooseReplacementEffect { affected_object: Some(shimmerer) },
        vec![0],
    );
    game.change_zone(shimmerer, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .unwrap();
    assert!(dp.is_empty());
    let entry = game.battlefield.get(&shimmerer).unwrap();
    assert_eq!(entry.counter_count(CounterType::PlusOnePlusOne), 3);
    assert!(entry.tapped, "tapped first: it was 0/0 when the tapped half looked");
}

// ---------------------------------------------------------------------------
// CR 616.1b — control-changing entry replacements go first
// ---------------------------------------------------------------------------

/// Two entry replacements on one card, one of them control-changing. CR 616.1b
/// forces the control-changing one first, so there is one candidate in each
/// bucket and nothing is asked; then the counters. It enters under the
/// opponent with two counters, and the entry is announced under the opponent.
// COVERS: ATOM-616.1b-001
#[test]
fn test_a_control_changing_entry_replacement_applies_first_and_asks_nothing() {
    let mut game = setup_two_player_game();

    let id = cast_and_resolve(&mut game, 0, defector_with_counters(), ManaType::Black);

    let entry = game.battlefield.get(&id).unwrap();
    assert_eq!(entry.controller, 1, "CR 616.1b: under an opponent's control");
    assert_eq!(entry.counter_count(CounterType::PlusOnePlusOne), 2, "and the counters still came");
    assert_eq!(entries(&game), vec![(id, 1)]);
}

/// Why 616.1b goes first: the controller is what the other replacements read.
/// P1's Kismet-shaped enchantment taps permanents P1's opponents control. P0
/// casts the Defector; it enters under P1 — not P1's opponent — and untapped.
/// With the enchantment on P0's side the same cast enters tapped, because
/// under P1 it is P0's opponent's permanent.
#[test]
fn test_the_controller_settled_by_cr_616_1b_is_what_later_filters_read() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, kismet_shaped(), 1);
    let id = cast_and_resolve(&mut game, 0, defector(), ManaType::Black);
    let entry = game.battlefield.get(&id).unwrap();
    assert_eq!(entry.controller, 1);
    assert!(!entry.tapped, "P1's own permanent now, so P1's Kismet does not tap it");

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, kismet_shaped(), 0);
    let id = cast_and_resolve(&mut game, 0, defector(), ManaType::Black);
    let entry = game.battlefield.get(&id).unwrap();
    assert_eq!(entry.controller, 1);
    assert!(entry.tapped, "an opponent's permanent to P0's Kismet, which read the settled controller");
}

/// "An opponent of your choice" with two opponents is a choice, made by the
/// effect's controller before the permanent enters (CR 614.12a): the entry the
/// performer announces already carries its result. Three players, because
/// with one opponent CR 102.2 leaves nothing to choose.
// COVERS-PARTIAL: ATOM-614.12a-001
#[test]
fn test_three_player_opponent_of_your_choice_is_asked_before_the_entry() {
    let mut game = setup_game(3);

    let dp = ScriptedDecisionProvider::new();
    let id = put_in_hand(&mut game, defector(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.cast_spell(0, id, &test_dp()).expect("it is castable");
    // Candidates are the opponents in player order, [1, 2]; index 1 is player 2.
    dp.expect_pick_n(ChoiceKind::ChooseEnteringController { object: id }, vec![1]);
    game.resolve_top_of_stack(&dp).expect("it resolves");

    assert!(dp.is_empty(), "asked once, before the entry");
    assert_eq!(game.battlefield.get(&id).unwrap().controller, 2);
    assert_eq!(entries(&game), vec![(id, 2)], "the announced entry carries the choice");
}

// ---------------------------------------------------------------------------
// CR 614.17d — "can't" effects on the frame
// ---------------------------------------------------------------------------

/// Worms of the Earth's shape without the frame: a Forest returned from the
/// graveyard stays there. Asked at the entry, before anything moves, so the
/// card never leaves.
#[test]
fn test_lands_cant_enter_keeps_a_returned_forest_in_the_graveyard() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, static_restriction("Lands can't enter", lands_cant_enter()), 1);

    let forest = put_in_graveyard(&mut game, mtgsim::cards::basic_lands::forest(), 0);
    game.change_zone(forest, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx()).unwrap();

    assert_eq!(game.get_object(forest).unwrap().zone, Zone::Graveyard);
    assert!(game.players[0].graveyard.contains(&forest));
    assert!(!game.battlefield.contains_key(&forest));
}

/// And with the frame: Grizzly Bears is not a land card, but under "each
/// permanent is a land in addition to its other types" it would be a land on
/// the battlefield — and CR 614.17d says that is the object the "can't" is
/// checked against. It stays in the graveyard; without the land-maker it enters.
// COVERS-PARTIAL: ATOM-614.17d-001
#[test]
fn test_a_cant_enter_is_decided_on_the_frame_at_the_entry() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, static_restriction("Lands can't enter", lands_cant_enter()), 1);
    put_on_battlefield(&mut game, everything_is_a_land(), 1);

    let bears = put_in_graveyard(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.change_zone(bears, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    assert_eq!(
        game.get_object(bears).unwrap().zone,
        Zone::Graveyard,
        "as it would exist on the battlefield it is a land, and lands can't enter"
    );

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, static_restriction("Lands can't enter", lands_cant_enter()), 1);
    let bears = reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert!(game.battlefield.contains_key(&bears), "a creature card that would stay one enters");
}

/// "Creatures you control can't have -1/-1 counters put on them." Chainbreaker
/// would enter with two; CR 101.2 refuses them at the door and the entry goes
/// on without them — Melira's ruling. The "can't" is decided on the frame:
/// "you control" reads the controller it would enter under.
// COVERS-PARTIAL: ATOM-614.17d-001
#[test]
fn test_a_cant_have_counters_put_on_refuses_them_at_the_entry() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        static_restriction("No -1/-1 counters on your creatures", no_minus_counters_on_your_creatures()),
        0,
    );

    let scarecrow = reanimate(&mut game, chainbreaker(), 0);

    let entry = game.battlefield.get(&scarecrow).unwrap();
    assert_eq!(entry.counter_count(CounterType::MinusOneMinusOne), 0, "refused, not applied and undone");
    assert_eq!(get_effective_power(&game, scarecrow), Some(3));
    assert_eq!(entries_of(&game, scarecrow), vec![0], "the entry itself happened");
}

/// The same "can't" on the opponent's side reaches nothing of P0's, so the
/// counters arrive.
#[test]
fn test_an_opponents_cant_does_not_reach_your_entering_creature() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        static_restriction("No -1/-1 counters on your creatures", no_minus_counters_on_your_creatures()),
        1,
    );

    let scarecrow = reanimate(&mut game, chainbreaker(), 0);
    assert_eq!(game.battlefield.get(&scarecrow).unwrap().counter_count(CounterType::MinusOneMinusOne), 2);
}

/// CR 306.5b's loyalty is a rule's counters, and it goes through the same
/// door: a "can't have loyalty counters put on it" over every permanent leaves
/// a planeswalker entering with none. No printed card scopes it that way — the
/// shape is Melira's Keepers' "can't have counters put on it" over all.
#[test]
fn test_the_rules_own_entry_counters_go_through_the_same_door() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        static_restriction(
            "No loyalty counters",
            Restriction::Event {
                pattern: EventPattern::CounterChange { counter: Some(CounterType::Loyalty), adding: true },
                affected: AffectedSet::Filter { filter: PermanentFilter::All },
                by: None,
            },
        ),
        1,
    );

    let jace = cast_and_resolve(&mut game, 0, planeswalker(), ManaType::Blue);
    assert_eq!(game.battlefield.get(&jace).unwrap().counter_count(CounterType::Loyalty), 0);
}

// ---------------------------------------------------------------------------
// Dryad Arbor — the land creature itself
// ---------------------------------------------------------------------------

/// A land that is a creature: played as a land, summoning-sick the turn it
/// arrives (CR 302.6, which its reminder text says), a creature to a filter.
#[test]
fn test_dryad_arbor_is_a_summoning_sick_land_creature() {
    let mut game = setup_two_player_game();
    let arbor = put_in_hand(&mut game, dryad_arbor(), 0);
    game.play_land(0, arbor, Zone::Hand, &test_ctx()).unwrap();

    assert!(game.battlefield.contains_key(&arbor));
    assert!(is_creature(&game, arbor));
    assert!(has_summoning_sickness(&game, arbor), "its {{T}} mana ability waits a turn");
    assert_eq!(get_effective_power(&game, arbor), Some(1));
}

/// CR 305.7 makes a nonbasic land a Mountain and strips its abilities; CR
/// 205.1a says setting land types replaces *land* types. Dryad Arbor under
/// Blood Moon is a Mountain Dryad that is still a 1/1 creature with one
/// intrinsic mana ability — not a Mountain that stopped being a Dryad.
#[test]
fn test_blood_moon_makes_dryad_arbor_a_mountain_dryad() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, blood_moon(), 1);

    let arbor = put_in_hand(&mut game, dryad_arbor(), 0);
    game.play_land(0, arbor, Zone::Hand, &test_ctx()).unwrap();

    let subtypes = get_effective_subtypes(&game, arbor);
    assert!(subtypes.contains(&Subtype::Land(LandType::Mountain)));
    assert!(!subtypes.contains(&Subtype::Land(LandType::Forest)));
    assert!(
        subtypes.contains(&Subtype::Creature(CreatureType::Dryad)),
        "CR 205.1a: the new land type replaces land types, and only those"
    );
    assert!(is_creature(&game, arbor));
    let abilities = get_effective_abilities(&game, arbor);
    assert_eq!(abilities.len(), 1, "CR 305.7: its own abilities are gone, the intrinsic one is here");
    assert_eq!(abilities[0].ability_type, AbilityType::Mana);
}
