//! Phase RF integration tests: the gather's zone leg — static replacement
//! abilities functioning off the battlefield (CR 113.6;
//! `replacement-architecture.md` §9 Phase RF and §3.3 source 2).
//!
//! **Read the board, not the card.** A Colossus dying off the battlefield was
//! found by the battlefield sweep before this phase, so the tests that prove
//! the leg are the ones where the card is in a **hand**, a **library** or on
//! the **stack** when it would be put into a graveyard. The battlefield board
//! is here as the control, and the two negatives — an ability that states no
//! zone (CR 113.6's default) and an ability the frame has stripped — are what
//! separate "the set was filed" from "the rule was asked".
//!
//! The last two tests are the affected side: the zone check `set_affects`
//! now asks of a `Filter` row, which the LJ-era `debug_assert` stood in for.

use mtgsim::cards::phase_rb_cards::rest_in_peace;
use mtgsim::cards::phase_rf_cards::{
    darksteel_colossus, hollow_hands, nexus_of_fate, sealing_ward, timid_golem,
};
use mtgsim::engine::actions::{ActionContext, ZoneChangeCause};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::events::event::GameEvent;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_graveyard, put_in_hand, put_in_library, put_on_battlefield, put_spell_on_stack,
    setup_two_player_game, test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::engine::targeting::{ChosenTargets};

fn zone_of(game: &GameState, id: ObjectId) -> Zone {
    game.get_object(id).unwrap().zone
}

/// How many times `player`'s library was shuffled — CR 701.24a's event, which
/// is the only line a shuffle writes.
fn shuffles(game: &GameState, player: PlayerId) -> usize {
    game.events
        .events()
        .filter(|e| matches!(e, GameEvent::LibraryShuffled { player_id } if *player_id == player))
        .count()
}

/// Three vanilla cards in `player`'s library, so a shuffle has something to
/// randomize and a mill has a library to read.
fn stock(game: &mut GameState, player: PlayerId) -> Vec<ObjectId> {
    (0..3).map(|_| put_in_library(game, vanilla_creature(1, 1, &[]), player)).collect()
}

// ---------------------------------------------------------------------------
// The leg — a source in a hand, a library, on the stack
// ---------------------------------------------------------------------------

/// The facility, on the board that needs it: a card in **hand** whose static
/// replacement ability functions there (CR 113.6b, "from anywhere").
///
/// Before RF the gather swept `battlefield_ids_ordered` alone, so a discarded
/// Colossus went to the graveyard like any other card. The rider is the other
/// half of the instruction: the library it was put into is shuffled.
#[test]
fn test_a_colossus_discarded_from_hand_is_shuffled_into_its_library_instead() {
    let mut game = setup_two_player_game();
    let library = stock(&mut game, 0);
    let colossus = put_in_hand(&mut game, darksteel_colossus(), 0);

    game.change_zone(colossus, Zone::Graveyard, ZoneChangeCause::Discarded, &test_ctx())
        .unwrap();

    assert_eq!(zone_of(&game, colossus), Zone::Library, "shuffled into its owner's library");
    assert!(game.players[0].graveyard.is_empty(), "it never reached the graveyard");
    assert!(!game.players[0].hand.contains(&colossus));
    assert_eq!(game.players[0].library.len(), library.len() + 1);
    assert_eq!(shuffles(&game, 0), 1, "CR 701.24a — the library was shuffled once");
}

/// A card in a **library** — the zone with no entity, no controller and, at
/// four seats, ~400 objects the gather cannot afford to walk. Milled, it is
/// "put into a graveyard from anywhere" and the substitute puts it back where
/// it is: no zone change, and the library is shuffled (CR 701.24c).
///
/// This is also the board `Game::new`'s registration door exists for — a card
/// that starts the game in a library and never moves before the mill.
#[test]
fn test_a_colossus_milled_from_a_library_stays_in_it_and_the_library_is_shuffled() {
    let mut game = setup_two_player_game();
    stock(&mut game, 0);
    let colossus = put_in_library(&mut game, darksteel_colossus(), 0);
    assert_eq!(*game.players[0].library.last().unwrap(), colossus, "on top");
    let before = game.events.len();

    game.change_zone(colossus, Zone::Graveyard, ZoneChangeCause::Milled, &test_ctx())
        .unwrap();

    assert_eq!(zone_of(&game, colossus), Zone::Library);
    assert!(game.players[0].graveyard.is_empty());
    assert_eq!(game.players[0].library.len(), 4);
    assert_eq!(shuffles(&game, 0), 1);
    assert!(
        !game
            .events
            .records_from(before)
            .iter()
            .any(|r| matches!(r.event, GameEvent::ZoneChange { .. })),
        "a card put into the zone it is in changes no zone, so no ZoneChange is announced"
    );
}

/// One mill is one event: `Primitive::Mill` proposes its N zone changes as
/// one batch (CR 701.17a's "put the top N cards"), and CR 616.1 decides each
/// member against the same board. So a Colossus second from the top does not
/// stop the mill — the card above it and the card below it reach the
/// graveyard in mill order, its own member is the one replaced — and the
/// rider runs after the whole batch (§4.1a), so the shuffle finds the library
/// the mill left rather than the one it started from.
#[test]
fn test_a_mill_is_one_event_and_the_colossus_replaces_only_its_own_member() {
    let mut game = setup_two_player_game();
    let below = put_in_library(&mut game, vanilla_creature(1, 1, &[]), 0);
    let colossus = put_in_library(&mut game, darksteel_colossus(), 0);
    let above = put_in_library(&mut game, vanilla_creature(2, 2, &[]), 0);
    // Where the milling spell would be as it resolves.
    let source = put_in_graveyard(&mut game, vanilla_creature(1, 1, &[]), 0);
    let before = game.events.len();

    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 0,
        targets: ChosenTargets::EMPTY,
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(
        &Effect::Atom(Primitive::Mill(AmountExpr::Fixed(3)), EffectRecipient::Controller),
        &ctx,
        &test_dp(),
    )
    .unwrap();

    assert_eq!(
        game.players[0].graveyard,
        vec![source, above, below],
        "the cards above and below the Colossus were milled, top first"
    );
    assert_eq!(game.players[0].library, vec![colossus], "its own member was replaced");
    assert_eq!(shuffles(&game, 0), 1);
    let order: Vec<&str> = game
        .events
        .records_from(before)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::ZoneChange { cause: ZoneChangeCause::Milled, .. } => Some("mill"),
            GameEvent::LibraryShuffled { .. } => Some("shuffle"),
            _ => None,
        })
        .collect();
    assert_eq!(
        order,
        vec!["mill", "mill", "shuffle"],
        "the whole mill first, then the rider — nothing stops in the middle of one event"
    );
}

/// A source on the **stack**: an instant countered on the way to its
/// graveyard (CR 701.6). Nexus of Fate is never a permanent, so nothing
/// before this phase could have found its ability at all.
#[test]
fn test_a_nexus_of_fate_countered_on_the_stack_goes_back_into_the_library() {
    let mut game = setup_two_player_game();
    stock(&mut game, 0);
    let nexus = put_spell_on_stack(&mut game, nexus_of_fate(), 0);

    game.change_zone(nexus, Zone::Graveyard, ZoneChangeCause::Countered, &test_ctx())
        .unwrap();

    assert_eq!(zone_of(&game, nexus), Zone::Library);
    assert!(game.stack.is_empty());
    assert!(game.players[0].graveyard.is_empty());
    assert_eq!(shuffles(&game, 0), 1);
}

/// The famous case: Nexus of Fate **resolves**, its owner takes an extra turn
/// (CR 500.7), and CR 608.2n's move to the graveyard is replaced — the card
/// goes back into the library it will be drawn from again.
#[test]
fn test_a_nexus_of_fate_that_resolves_takes_an_extra_turn_and_returns_to_the_library() {
    let mut game = setup_two_player_game();
    stock(&mut game, 0);
    let nexus = put_spell_on_stack(&mut game, nexus_of_fate(), 0);
    // `put_spell_on_stack` stages an entry with no effect; give it the card's.
    let mut entry = game.stack_entries[&nexus].clone();
    entry.effect = Effect::Atom(Primitive::ExtraTurn, EffectRecipient::Controller);
    game.set_stack_entry(entry);

    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(game.turn_queue, vec![0], "an extra turn for its controller");
    assert_eq!(zone_of(&game, nexus), Zone::Library, "CR 608.2n's move was replaced");
    assert!(game.players[0].graveyard.is_empty());
    assert_eq!(shuffles(&game, 0), 1);
}

/// The control: the battlefield sweep's half, which worked before RF. Its own
/// ruling's board — indestructible does not stop a sacrifice, "in these cases,
/// of course, Darksteel Colossus would be shuffled into its owner's library
/// instead".
#[test]
fn test_a_colossus_sacrificed_from_the_battlefield_is_shuffled_into_its_library() {
    let mut game = setup_two_player_game();
    stock(&mut game, 0);
    let colossus = put_on_battlefield(&mut game, darksteel_colossus(), 0);

    game.change_zone(colossus, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();

    assert_eq!(zone_of(&game, colossus), Zone::Library);
    assert!(!game.battlefield.contains_key(&colossus));
    assert_eq!(shuffles(&game, 0), 1);
}

/// "Its owner's library", not its controller's: a stolen Colossus sacrificed
/// by the thief goes back into the library of the player who owns it, and
/// that is the library that is shuffled.
#[test]
fn test_a_stolen_colossus_returns_to_its_owners_library_and_shuffles_that_one() {
    let mut game = setup_two_player_game();
    stock(&mut game, 0);
    stock(&mut game, 1);
    let colossus = put_on_battlefield(&mut game, darksteel_colossus(), 0);
    game.battlefield.get_mut(&colossus).unwrap().controller = 1;
    game.bump_layer_epoch();

    game.change_zone(colossus, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();

    assert!(game.players[0].library.contains(&colossus), "its owner's library");
    assert_eq!(shuffles(&game, 0), 1);
    assert_eq!(shuffles(&game, 1), 0, "the thief's library is untouched");
}

// ---------------------------------------------------------------------------
// The two negatives — filed is not functioning, printed is not effective
// ---------------------------------------------------------------------------

/// CR 113.6's first sentence: an ability that states no zone functions only
/// on the battlefield. The same pattern as the Colossus's, minus "from
/// anywhere" — exiled when it dies, and an ordinary card when discarded.
///
/// The "only" in CR 113.6b's "only from those zones", asked of a replacement
/// rather than of Wonder's grant, and the test that the zone leg asks
/// `functions_in` of every ability rather than trusting its candidate set.
// COVERS-PARTIAL: ATOM-113.6-001
#[test]
fn test_an_ability_that_states_no_zone_functions_only_on_the_battlefield() {
    let mut game = setup_two_player_game();
    let in_hand = put_in_hand(&mut game, timid_golem(), 0);
    game.change_zone(in_hand, Zone::Graveyard, ZoneChangeCause::Discarded, &test_ctx())
        .unwrap();
    assert_eq!(zone_of(&game, in_hand), Zone::Graveyard, "in hand, the ability does not function");

    let in_library = put_in_library(&mut game, timid_golem(), 0);
    game.change_zone(in_library, Zone::Graveyard, ZoneChangeCause::Milled, &test_ctx())
        .unwrap();
    assert_eq!(zone_of(&game, in_library), Zone::Graveyard, "nor in a library");

    let on_battlefield = put_on_battlefield(&mut game, timid_golem(), 0);
    game.change_zone(on_battlefield, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();
    assert_eq!(zone_of(&game, on_battlefield), Zone::Exile, "on the battlefield it does");
}

/// The zone leg reads the **effective** ability list (`CLAUDE.md`'s
/// layer-system invariant), which is what makes a Layer 6 strip into a hand
/// reach a replacement ability there. "Cards in hands lose all abilities" is
/// Yixlid Jailer one zone over; with it on the battlefield a discarded
/// Colossus has no clause to replace its own discard, and goes to the
/// graveyard. The gate still filed the card — its *printed* ability functions
/// in a hand — which is the over-approximation the gate is allowed and the
/// answer it is not.
#[test]
fn test_the_zone_leg_reads_the_effective_ability_list() {
    let mut game = setup_two_player_game();
    stock(&mut game, 0);
    put_on_battlefield(&mut game, hollow_hands(), 1);
    let colossus = put_in_hand(&mut game, darksteel_colossus(), 0);

    game.change_zone(colossus, Zone::Graveyard, ZoneChangeCause::Discarded, &test_ctx())
        .unwrap();

    assert_eq!(zone_of(&game, colossus), Zone::Graveyard, "stripped in hand, so nothing replaced");
    assert_eq!(shuffles(&game, 0), 0);
}

/// A candidate found off the battlefield still leaves the zone it stops
/// functioning in: the set is per zone (CR 113.6), retired on leaving and
/// refiled on arrival, so a card bouncing between zones is asked exactly where
/// it is. Here the hand's registration is what the discard reads, and the
/// library's — where it lands — is what the next mill reads.
#[test]
fn test_the_candidate_set_follows_the_card_from_zone_to_zone() {
    let mut game = setup_two_player_game();
    stock(&mut game, 0);
    let colossus = put_in_hand(&mut game, darksteel_colossus(), 0);
    assert!(game.zone_replacement_ability_sources.contains_key(&colossus));
    assert!(!game.replacement_ability_sources.contains(&colossus));

    game.change_zone(colossus, Zone::Graveyard, ZoneChangeCause::Discarded, &test_ctx())
        .unwrap();
    assert_eq!(zone_of(&game, colossus), Zone::Library);
    assert!(game.zone_replacement_ability_sources.contains_key(&colossus), "refiled in the library");

    // From the library, a second time, through the mill door.
    game.change_zone(colossus, Zone::Graveyard, ZoneChangeCause::Milled, &test_ctx())
        .unwrap();
    assert_eq!(zone_of(&game, colossus), Zone::Library);
    assert_eq!(shuffles(&game, 0), 2);

    // Into exile, where the clause functions too (`ZoneSet::ALL`) and the map
    // follows it. The move *out* of exile into a graveyard is the one board
    // this file does not have: no registered card makes it and no
    // `ZoneChangeCause` names it — Pull from Eternity brings both
    // (`codebase-state.md` main item 148).
    game.change_zone(colossus, Zone::Exile, ZoneChangeCause::Exiled, &test_ctx()).unwrap();
    assert_eq!(zone_of(&game, colossus), Zone::Exile);
    assert!(game.zone_replacement_ability_sources.contains_key(&colossus), "refiled in exile");

    // And onto the battlefield, where the other set takes over.
    let on_bf = put_on_battlefield(&mut game, darksteel_colossus(), 0);
    assert!(game.replacement_ability_sources.contains(&on_bf));
    assert!(!game.zone_replacement_ability_sources.contains_key(&on_bf));
}

// ---------------------------------------------------------------------------
// Two replacements on one card's journey — CR 616.1 twice, and CR 701.24c
// ---------------------------------------------------------------------------

/// A Colossus that is a **commander**. Sacrificed, its own clause replaces
/// the graveyard with the library; CR 903.9b then replaces *that* with the
/// command zone, if its owner says so. The rider still shuffles the library
/// the card never reached — CR 701.24c's "even if … an effect causes all of
/// those objects to be moved to another zone" — and the card stays where
/// 903.9b put it, which is why the rider moves nothing.
#[test]
fn test_a_colossus_commander_goes_to_the_command_zone_and_its_owners_library_is_still_shuffled() {
    let mut game = setup_two_player_game();
    stock(&mut game, 0);
    let colossus = put_on_battlefield(&mut game, darksteel_colossus(), 0);
    game.objects.get_mut(&colossus).unwrap().is_commander = true;

    let dp = ScriptedDecisionProvider::new();
    // Only 903.9b asks: the Colossus's own clause is mandatory.
    dp.expect_pick_n(
        ChoiceKind::ApplyOptionalReplacement { affected_object: Some(colossus), source: colossus },
        vec![0],
    );
    let ctx = ActionContext::new(&dp);
    game.change_zone(colossus, Zone::Graveyard, ZoneChangeCause::Sacrificed, &ctx).unwrap();

    assert_eq!(zone_of(&game, colossus), Zone::Command, "903.9b applied to the substitute");
    assert!(!game.players[0].library.contains(&colossus));
    assert_eq!(shuffles(&game, 0), 1, "CR 701.24c — shuffled although the card went elsewhere");
    assert!(dp.is_empty(), "asked exactly once");
}

// ---------------------------------------------------------------------------
// The affected side — a Filter row reaches what its zones say
// ---------------------------------------------------------------------------

/// "If a creature would be put into a graveyard, exile it instead" is a row
/// over the battlefield: CR 109.2's "creature" is a creature permanent, and a
/// milled creature *card* is not one. Rest in Peace's "a card or token … from
/// anywhere" is `ZoneSet::ALL` and reaches the same milled card.
///
/// The behavior the `debug_assert` in `set_affects` stood in for. Before RF
/// the zone half of a `Filter` was not asked, and the battlefield row would
/// have exiled the milled card too.
#[test]
fn test_a_battlefield_scoped_filter_row_does_not_reach_a_card_in_a_library() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, sealing_ward(), 0);

    let milled = put_in_library(&mut game, vanilla_creature(1, 1, &[]), 0);
    game.change_zone(milled, Zone::Graveyard, ZoneChangeCause::Milled, &test_ctx()).unwrap();
    assert_eq!(zone_of(&game, milled), Zone::Graveyard, "a creature card is not a creature");

    let died = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    game.change_zone(died, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx()).unwrap();
    assert_eq!(zone_of(&game, died), Zone::Exile, "a creature is");
}

/// The same mill under Rest in Peace, whose row says "from anywhere".
#[test]
fn test_a_from_anywhere_filter_row_reaches_a_card_in_a_library() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, rest_in_peace(), 0);

    let milled = put_in_library(&mut game, vanilla_creature(1, 1, &[]), 1);
    game.change_zone(milled, Zone::Graveyard, ZoneChangeCause::Milled, &test_ctx()).unwrap();
    assert_eq!(zone_of(&game, milled), Zone::Exile, "ZoneSet::ALL reaches a library");
}
