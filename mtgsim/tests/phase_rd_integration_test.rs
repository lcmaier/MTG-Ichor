//! Phase RD-1 — the damage event's two subjects and its results.
//!
//! CR 614.5, 615.10, 616.1, 701.10g and 120.3a/c, against the four printed
//! cards and the one fixture, Loyalty Probe.
//!
//! The card file's own tests deal damage to a *creature*, because that path
//! existed before this phase. Everything here needs one of the two things
//! RD-1 adds to the board: an effect scoped to a **player**, or one of
//! CR 120.3's **results** — life loss and loyalty removal — that
//! `perform_action` did not decompose the damage into.

use mtgsim::cards::{fixtures::loyalty_probe, printings::{angel_of_suffering, furnace_of_rath, ghosts_of_the_innocent, gisela_blade_of_goldnight}};
use mtgsim::cards::printings;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::events::event::{BatchId, GameEvent};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    fill_library, place_vanilla_creature, put_on_battlefield, set_attacking, set_blocked_by,
    set_blocking, setup_two_player_game, test_ctx, RecordingDecisionProvider,
};
use mtgsim::types::effects::CounterType;
use mtgsim::engine::combat::resolution::assign_combat_damage;
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A 1/1 for `owner` to be a damage source. Nothing about it matters but its
/// id: `EventPattern::DealDamage` carries no source-side constraint until RD-3.
fn source_for(game: &mut GameState, owner: PlayerId) -> ObjectId {
    place_vanilla_creature(game, owner, 1, 1, &[])
}

fn bolt_player(game: &mut GameState, source: ObjectId, victim: PlayerId, amount: u64) {
    game.execute_action(
        GameAction::DealDamage {
            source,
            target: ObjectOrPlayerId::Player(victim),
            amount,
            is_combat: false,
            unpreventable: false,
            source_lki: None,
        },
        &test_ctx(),
    )
    .unwrap();
}

fn bolt_player_with(
    game: &mut GameState,
    dp: &dyn mtgsim::ui::decision::DecisionProvider,
    source: ObjectId,
    victim: PlayerId,
    amount: u64,
) {
    let ctx = ActionContext::new(dp);
    game.execute_action(
        GameAction::DealDamage {
            source,
            target: ObjectOrPlayerId::Player(victim),
            amount,
            is_combat: false,
            unpreventable: false,
            source_lki: None,
        },
        &ctx,
    )
    .unwrap();
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

// ---------------------------------------------------------------------------
// CR 120.3a — the contained life loss
// ---------------------------------------------------------------------------

/// > 120.3a Damage dealt to a player causes that player to lose that much
/// > life.
///
/// The lifelink test's twin, from the other side of the same rule: the loss is
/// a *result of* the damage, so it joins the damage's batch rather than opening
/// one, and a CR 603.2c trigger will see one event rather than two.
// COVERS: ATOM-120.3a-001
#[test]
fn damage_to_a_player_proposes_a_contained_life_loss_in_the_damages_batch() {
    let mut game = setup_two_player_game();
    let source = source_for(&mut game, 0);
    let before = game.recorded_events().len();

    bolt_player(&mut game, source, 1, 3);

    // `ATOM-120.3a-001`'s whole board and expected result: a player at 20 takes
    // 3 damage from a source without infect and is at 17. The batch assertions
    // below are this phase's; this line is the atom's.
    assert_eq!(life(&game, 1), 17);
    let recorded = game.recorded_events();
    let records = recorded.records_from(before);
    let damage: Vec<Option<BatchId>> = records
        .iter()
        .filter(|r| matches!(r.event, GameEvent::DamageDealt { .. }))
        .map(|r| r.batch())
        .collect();
    let life_changes: Vec<Option<BatchId>> = records
        .iter()
        .filter(|r| matches!(r.event, GameEvent::LifeChanged { .. }))
        .map(|r| r.batch())
        .collect();
    assert_eq!(damage.len(), 1);
    assert_eq!(life_changes.len(), 1);
    assert!(damage[0].is_some());
    assert_eq!(
        damage[0], life_changes[0],
        "CR 120.3a's loss is a result of the damage, not a second event",
    );
}

/// The loss is a **proposal**, not a subtraction, and the observable
/// difference is that it traverses the CR 616.1 loop: damage to a player costs
/// one more `gather` than damage to an object.
///
/// That is the whole of RD-1's middle-arm prediction (`§9`, "Measured"), and
/// it is what an RE-era Bloodletter of Aclazotz will hang off — a life-loss
/// replacement cannot exist unless the loss is an event.
#[test]
fn the_life_loss_is_a_proposal_and_reaches_the_pipeline() {
    let mut game = setup_two_player_game();
    let source = source_for(&mut game, 0);
    let victim = place_vanilla_creature(&mut game, 1, 9, 9, &[]);

    let before = game.diagnostics.replacement_gathers();
    game.execute_action(
        GameAction::DealDamage {
            source,
            target: ObjectOrPlayerId::Object(victim),
            amount: 1,
            is_combat: false,
            unpreventable: false,
            source_lki: None,
        },
        &test_ctx(),
    )
    .unwrap();
    let object_gathers = game.diagnostics.replacement_gathers() - before;

    let before = game.diagnostics.replacement_gathers();
    bolt_player(&mut game, source, 1, 1);
    let player_gathers = game.diagnostics.replacement_gathers() - before;

    assert_eq!(object_gathers, 1, "the damage itself");
    assert_eq!(
        player_gathers,
        object_gathers + 1,
        "the damage, and the CR 120.3a life loss it contains",
    );
}

/// The log line the decomposition must not change: `LifeChanged` still names
/// the damage's source, which is what the loss's `damage_source` carries.
#[test]
fn the_life_change_from_damage_still_names_the_damage_source() {
    let mut game = setup_two_player_game();
    let source = source_for(&mut game, 0);
    let before = game.recorded_events().len();

    bolt_player(&mut game, source, 1, 4);

    let sources: Vec<Option<ObjectId>> = game
        .recorded_events()
        .records_from(before)
        .iter()
        .filter_map(|r| match r.event {
            GameEvent::LifeChanged { source, .. } => Some(source.map(|source| source.id)),
            _ => None,
        })
        .collect();
    assert_eq!(sources, vec![Some(source)]);
}

/// CR 614.6 — a prevented event never happens, so there is no damage for
/// CR 120.3a to turn into life loss. Angel of Suffering's rider still runs
/// (§4.1a), which is what makes this test about the *loss* rather than about
/// the prevention.
#[test]
fn prevented_damage_proposes_no_life_loss() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, angel_of_suffering(), 0);
    fill_library(&mut game, 0, 20);
    let source = source_for(&mut game, 1);
    let before = game.recorded_events().len();

    bolt_player(&mut game, source, 0, 3);

    assert_eq!(life(&game, 0), 20);
    assert!(
        !game
            .recorded_events()
            .records_from(before)
            .iter()
            .any(|r| matches!(r.event, GameEvent::LifeChanged { .. })),
        "no damage happened, so nothing caused a loss of life",
    );
}

// ---------------------------------------------------------------------------
// CR 120.3c — loyalty
// ---------------------------------------------------------------------------

/// > 120.3c Damage dealt to a planeswalker causes that many loyalty counters
/// > to be removed from that planeswalker.
// COVERS: ATOM-120.3c-001
#[test]
fn three_damage_to_a_five_loyalty_planeswalker_leaves_two() {
    let mut game = setup_two_player_game();
    let probe = put_on_battlefield(&mut game, loyalty_probe(), 1);
    // The atom's board is five loyalty; the fixture prints three, because
    // three is what makes a Lightning Bolt lethal to it in a fuzz game.
    game.add_counters(probe, CounterType::Loyalty, 2);
    assert_eq!(game.battlefield[&probe].counter_count(CounterType::Loyalty), 5);
    let source = source_for(&mut game, 0);

    game.execute_action(
        GameAction::DealDamage {
            source,
            target: ObjectOrPlayerId::Object(probe),
            amount: 3,
            is_combat: false,
            unpreventable: false,
            source_lki: None,
        },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(game.battlefield[&probe].counter_count(CounterType::Loyalty), 2);
}

/// CR 704.5i — a planeswalker with no loyalty counters is put into its owner's
/// graveyard. Measured at **0** across 200 stress games before this phase
/// (the state-based actions' phase), because nothing could take a loyalty
/// counter off.
#[test]
fn lethal_damage_to_a_planeswalker_reaches_cr_704_5i() {
    let mut game = setup_two_player_game();
    let probe = put_on_battlefield(&mut game, loyalty_probe(), 1);
    let source = source_for(&mut game, 0);

    game.execute_action(
        GameAction::DealDamage {
            source,
            target: ObjectOrPlayerId::Object(probe),
            amount: 3,
            is_combat: false,
            unpreventable: false,
            source_lki: None,
        },
        &test_ctx(),
    )
    .unwrap();
    assert_eq!(game.battlefield[&probe].counter_count(CounterType::Loyalty), 0);

    let dp = ScriptedDecisionProvider::new();
    assert!(game.check_state_based_actions(&dp).unwrap());
    assert!(!game.battlefield.contains_key(&probe));
    assert!(game.players[1].graveyard.contains(&probe));
}

/// CR 120.3 lists "one or more of the following results", and 120.3c and
/// 120.3e are two of them: a creature planeswalker takes marked damage **and**
/// loses loyalty. `March of the Machines` cannot make one, so the fixture is
/// given both types directly.
#[test]
fn damage_to_a_creature_planeswalker_both_marks_and_removes_loyalty() {
    use mtgsim::objects::card_data::CardDataBuilder;
    use mtgsim::types::card_types::CardType;

    let mut game = setup_two_player_game();
    let data = CardDataBuilder::new("Loyalty Probe Creature")
        .card_type(CardType::Planeswalker)
        .card_type(CardType::Creature)
        .power_toughness(4, 4)
        .loyalty(5)
        .build();
    let probe = put_on_battlefield(&mut game, data, 1);
    let source = source_for(&mut game, 0);

    game.execute_action(
        GameAction::DealDamage {
            source,
            target: ObjectOrPlayerId::Object(probe),
            amount: 2,
            is_combat: false,
            unpreventable: false,
            source_lki: None,
        },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(game.battlefield[&probe].damage_marked, 2, "CR 120.3e");
    assert_eq!(
        game.battlefield[&probe].counter_count(CounterType::Loyalty),
        3,
        "CR 120.3c, and it is not the other result instead of this one",
    );
}

/// CR 120.3e is written about a *creature*, so a permanent that is neither a
/// creature nor a planeswalker takes neither result. Damage to one is
/// unreachable from the registered pool — `SelectionFilter::Any` offers only
/// creatures, planeswalkers and players — and this pins the arm anyway,
/// because a wither or infect arm lands beside it (`backlog.md` §2.6).
#[test]
fn damage_to_a_noncreature_nonplaneswalker_marks_nothing() {
    let mut game = setup_two_player_game();
    let enchantment = put_on_battlefield(&mut game, furnace_of_rath(), 1);
    let source = source_for(&mut game, 0);

    game.execute_action(
        GameAction::DealDamage {
            source,
            target: ObjectOrPlayerId::Object(enchantment),
            // Furnace doubles it; the point is that 6 lands nowhere.
            amount: 3,
            is_combat: false,
            unpreventable: false,
            source_lki: None,
        },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(game.battlefield[&enchantment].damage_marked, 0);
}

// ---------------------------------------------------------------------------
// CR 701.10g / 614.5 — doubling, on a player
// ---------------------------------------------------------------------------

/// > 701.10g To double an amount of damage that a source would deal to an
/// > object or player means to replace that damage event with a damage event
/// > that deals twice that much damage.
// COVERS: ATOM-701.10g-001
#[test]
fn furnace_of_rath_doubles_damage_to_a_player() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, furnace_of_rath(), 0);
    let source = source_for(&mut game, 0);

    bolt_player(&mut game, source, 1, 3);

    assert_eq!(life(&game, 1), 14, "3 doubled to 6");
}

/// "If you have two of these on the battlefield, the damage is multiplied
/// by 4." CR 614.5's own example, and the first time a **registered** board
/// could build it: Furnace of Rath is not legendary.
///
/// **And nobody is asked** (RD-2; `replacement-architecture.md` §11 item 29):
/// a bucket of nothing but multipliers is order-invariant — multiplication
/// commutes, and no multiplier can take another's applicability away — so
/// CR 616.1's prompt would be a choice with one outcome, which §11 item 19
/// says never to ask. Until RD-2 this test asserted the prompt was asked once.
// COVERS: ATOM-614.5-001
// COVERS: COMP-614-616-DOUBLE-REPLACEMENT-001
#[test]
fn two_furnaces_multiply_by_four_and_ask_nothing() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, furnace_of_rath(), 0);
    put_on_battlefield(&mut game, furnace_of_rath(), 0);
    let source = source_for(&mut game, 0);

    let dp = RecordingDecisionProvider::picking(0);
    bolt_player_with(&mut game, &dp, source, 1, 2);

    assert_eq!(life(&game, 1), 12, "2 -> 4 -> 8, and not more");
    assert_eq!(dp.prompts(), 0, "one outcome, no question");
}

/// The suppression's premise is that the two are *one* set of choosable
/// effects whatever else differs about them — here, their controllers — and
/// the composite atom's "either way" is the theorem rather than a second run:
/// the debug build re-gathers after the suppressed choice applied and asserts
/// the other still applies, which is the check that fires if the order could
/// ever have mattered.
#[test]
fn two_furnaces_under_different_controllers_still_ask_nothing() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, furnace_of_rath(), 0);
    put_on_battlefield(&mut game, furnace_of_rath(), 1);
    let source = source_for(&mut game, 0);

    let dp = RecordingDecisionProvider::picking(1);
    bolt_player_with(&mut game, &dp, source, 1, 1);
    assert_eq!(life(&game, 1), 16, "1 -> 2 -> 4");
    assert_eq!(dp.prompts(), 0);
}

// ---------------------------------------------------------------------------
// CR 616.1 — the order is the affected player's, and it matters
// ---------------------------------------------------------------------------

/// Ghosts of the Innocent's own ruling, verbatim:
///
/// > If both Ghosts of the Innocent and Furnace of Rath (which doubles damage)
/// > are on the battlefield, the controller of the permanent being dealt damage
/// > or the player being dealt damage can apply the effects in either order.
/// > This can matter if the original amount of damage is odd. For example, if a
/// > source would deal 3 damage, Ghosts of the Innocent would make it 1 damage,
/// > then Furnace of Rath would make it 2 damage, if the player chose to apply
/// > the effects in that order.
///
/// The first non-commuting CR 616.1 choice reachable from two **printed**
/// statics: 3 damage is either 1 then 2, or 6 then 3.
// COVERS-PARTIAL: COMP-614-DAMAGE-ORDERING-001 — the atom's non-commuting pair
// is "plus 1" and "double"; `AmountRewrite::Plus` has no printed consumer until
// Torbran in RD-3, so the pair here is halve-and-double. Everything the atom
// asserts about the *choice* — that it is presented, that both orderings are
// correct and different, and that each effect applies once — is proved.
#[test]
fn ghosts_beside_furnace_does_not_commute_and_the_damaged_player_chooses() {
    let outcomes: Vec<i64> = [true, false]
        .into_iter()
        .map(|ghosts_first| {
            let mut game = setup_two_player_game();
            let ghosts = put_on_battlefield(&mut game, ghosts_of_the_innocent(), 0);
            let furnace = put_on_battlefield(&mut game, furnace_of_rath(), 0);
            let source = source_for(&mut game, 0);
            let dp = RecordingDecisionProvider::choosing([ChoiceOption::Object(if ghosts_first { ghosts } else { furnace })]);
            bolt_player_with(&mut game, &dp, source, 1, 3);
            assert_eq!(dp.prompts(), 1, "two candidates, then one");
            20 - life(&game, 1)
        })
        .collect();

    assert_eq!(
        outcomes,
        vec![2, 3],
        "halve-then-double is 3 -> 1 -> 2; double-then-halve is 3 -> 6 -> 3",
    );
}

/// The same question against a *prevention* rather than a second `Instead`:
/// Gisela's "prevent half, rounded up" beside an opponent's Furnace of Rath,
/// on 5 damage to Gisela's controller.
///
/// Gisela's ruling: "If multiple replacement effects would modify how damage
/// would be dealt, the player being dealt damage (or the controller of the
/// permanent being dealt damage) chooses the order in which to apply those
/// effects."
#[test]
fn giselas_prevention_beside_an_opponents_furnace_does_not_commute() {
    let outcomes: Vec<i64> = [true, false]
        .into_iter()
        .map(|gisela_first| {
            let mut game = setup_two_player_game();
            let gisela = put_on_battlefield(&mut game, gisela_blade_of_goldnight(), 0);
            let furnace = put_on_battlefield(&mut game, furnace_of_rath(), 1);
            let source = source_for(&mut game, 1);
            let dp = RecordingDecisionProvider::choosing([ChoiceOption::Object(if gisela_first { gisela } else { furnace })]);
            bolt_player_with(&mut game, &dp, source, 0, 5);
            20 - life(&game, 0)
        })
        .collect();

    assert_eq!(
        outcomes,
        vec![4, 5],
        "prevent 3 of 5 then double 2 is 4; double to 10 then prevent 5 is 5",
    );
}

/// Gisela's doubler and her prevention are two rows on one object, and at most
/// one of them applies to **one damage proposal** — one source, one subject —
/// because CR 102.1's "an opponent" and CR 109.5's "you" are disjoint. So a
/// single damage event is never asked a CR 616.1 question by her alone.
///
/// **Not a claim about a whole board wipe.** CR 120.4d calls all of a
/// Pyroclasm's simultaneous damage "the damage event", and the CR uses the word
/// both ways; the engine's unit is the batch *member*, which is what CR 614.5's
/// applied set and CR 615.10's "separately to … events that would happen at the
/// same time" are keyed on. A Pyroclasm across both players' creatures is one
/// batch in which Gisela's doubler applies to the opponent's members and her
/// prevention to her controller's — both halves live, on different members.
/// The second board below is that case.
#[test]
fn giselas_two_halves_never_apply_to_one_damage_proposal() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, gisela_blade_of_goldnight(), 0);
    let source = source_for(&mut game, 0);

    let dp = RecordingDecisionProvider::picking(0);
    bolt_player_with(&mut game, &dp, source, 0, 5);
    assert_eq!(life(&game, 0), 18, "prevent half of 5 rounded up leaves 2");

    bolt_player_with(&mut game, &dp, source, 1, 5);
    assert_eq!(life(&game, 1), 10, "and the opponent's is doubled");

    assert_eq!(dp.prompts(), 0, "one candidate per proposal is no choice at all");

    // The other half of the doc comment: one *batch* whose members are about
    // different subjects sees both of Gisela's rows, each on its own member.
    let mine = place_vanilla_creature(&mut game, 0, 9, 9, &[]);
    let theirs = place_vanilla_creature(&mut game, 1, 9, 9, &[]);
    let ctx = ActionContext::new(&dp);
    game.execute_actions(
        vec![
            GameAction::DealDamage {
                source,
                target: ObjectOrPlayerId::Object(mine),
                amount: 3,
                is_combat: false,
                unpreventable: false,
                source_lki: None,
            },
            GameAction::DealDamage {
                source,
                target: ObjectOrPlayerId::Object(theirs),
                amount: 3,
                is_combat: false,
                unpreventable: false,
                source_lki: None,
            },
        ],
        &ctx,
    )
    .unwrap();
    assert_eq!(game.battlefield[&mine].damage_marked, 1, "prevent half of 3, up");
    assert_eq!(game.battlefield[&theirs].damage_marked, 6, "doubled");
    assert_eq!(dp.prompts(), 0, "still one candidate per member");
}

// ---------------------------------------------------------------------------
// Trample — divide before doubling
// ---------------------------------------------------------------------------

/// Furnace of Rath's ruling: "The trample rules cause damage to be divided
/// before it is doubled."
///
/// Structural rather than coded: `assign_combat_damage` divides the attacker's
/// power between the blocker and the defending player and proposes two events,
/// so the pipeline never sees the undivided number. War Mammoth (3/3, trample)
/// is the pool's trampler.
///
/// **The blocker is a 2/2, and that is the whole test.** With a 1/1 the two
/// readings of CR 702.19b agree by accident — 1 is lethal before doubling and
/// after — so the board says nothing. Against a 2/2 they disagree: lethal
/// judged on the *printed* 3 forces 2 to the blocker and leaves 1 to trample
/// through, while lethal judged on a doubled 6 would let the attacker assign 1
/// (which Furnace would make 2, "lethal") and send 2 through. The assertion on
/// the minimum the prompt carried is the direct form of that claim; the
/// assertions on the board are what it produces.
#[test]
fn trample_assigns_lethal_before_furnace_doubles() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, furnace_of_rath(), 0);
    let mammoth = put_on_battlefield(&mut game, printings::war_mammoth(), 0);
    let blocker = place_vanilla_creature(&mut game, 1, 2, 2, &[]);

    set_attacking(&mut game, mammoth, 1);
    set_blocked_by(&mut game, mammoth, vec![blocker]);
    set_blocking(&mut game, blocker, vec![mammoth]);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_allocation(
        ChoiceKind::AssignTrampleDamage {
            attacker_id: mammoth,
            defending_target: ObjectOrPlayerId::Player(1),
        },
        vec![(ChoiceOption::Object(blocker), 2), (ChoiceOption::Player(1), 1)],
    );
    let assignments = assign_combat_damage(&game, &dp, 0, false);
    // The floor the prompt carried, which is this test's claim: CR 702.19b's
    // lethal requirement is the blocker's toughness, judged on the Mammoth's
    // printed 3 — not the 1 a doubled assignment would make lethal.
    assert_eq!(dp.allocation_mins(), vec![vec![2u64, 0]]);
    assert_eq!(assignments.len(), 3, "two halves of the Mammoth, plus the block");

    let ctx = ActionContext::new(&dp);
    game.apply_combat_damage(assignments, &ctx).unwrap();

    // Each half is doubled on its own: 2 -> 4 on the blocker, 1 -> 2 through.
    // Doubling first and dividing after would have been 6 to split.
    assert_eq!(game.battlefield[&blocker].damage_marked, 4);
    assert_eq!(life(&game, 1), 18);
    // The blocker's own 2 power, doubled back at the Mammoth.
    assert_eq!(game.battlefield[&mammoth].damage_marked, 4);
}

// ---------------------------------------------------------------------------
// Angel of Suffering — a rider on a player subject
// ---------------------------------------------------------------------------

/// > If damage would be dealt to you, prevent that damage and mill twice that
/// > many cards.
///
/// The rider names the player the damage was aimed at — RD-1's
/// `ObjectOrPlayerId::Player` — and reads the event's amount through
/// `AmountExpr::Multiply(ReplacedAmount, 2)`.
#[test]
fn angel_of_suffering_prevents_the_damage_and_mills_twice_that_many() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, angel_of_suffering(), 0);
    fill_library(&mut game, 0, 20);
    let source = source_for(&mut game, 1);

    bolt_player(&mut game, source, 0, 3);

    assert_eq!(life(&game, 0), 20);
    assert_eq!(game.players[0].library.len(), 14);
    assert_eq!(game.players[0].graveyard.len(), 6);
}

/// Two rulings on one claim, so one test with two boards:
///
/// - *"If you would mill more cards than are in your library, you mill all
///   cards in your library"* — CR 701.17b's "mill as many as possible", which
///   is not a failure.
/// - *"Damage that would be dealt to you will be prevented even if you can't
///   mill twice that many"* — the prevention is decided in the CR 616.1 loop
///   and the rider runs afterwards (§4.1a), so a short library cannot
///   un-prevent it.
///
/// **The empty board is the sharper of the two and is why both are here.** A
/// short library still proposes moves; an empty one proposes *none at all*, so
/// it is the board that would fail if the prevention were ever made conditional
/// on the rider having done something. They were two tests until review pointed
/// out that they make one claim.
///
/// A genuine "can't mill" — a CR 101.2 restriction over the library→graveyard
/// move — would be a third, distinct board, and nothing prints one; the
/// fixture-without-a-card shape is RD-4's, where the "damage can't be
/// prevented" family already needs it.
#[test]
fn angel_of_suffering_prevents_however_little_it_can_mill() {
    for (library, milled) in [(3usize, 3usize), (0, 0)] {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, angel_of_suffering(), 0);
        fill_library(&mut game, 0, library);
        let source = source_for(&mut game, 1);

        bolt_player(&mut game, source, 0, 5);

        assert_eq!(life(&game, 0), 20, "prevented with {library} cards to mill");
        assert!(game.players[0].library.is_empty());
        assert_eq!(game.players[0].graveyard.len(), milled);
    }
}

/// CR 701.17a — "that player puts **that many cards** from the top of their
/// library into their graveyard" — is one simultaneous move, and the CR says
/// nothing here like CR 121.2's "cards may only be drawn one at a time".
///
/// So a mill of six is **one batch of six members**, not six batches. The batch
/// is what a CR 603.2c trigger will read: "whenever one or more cards are put
/// into your graveyard" has to fire once. Each member is still its own event
/// for CR 614.5, which is what lets Leyline of the Void apply to every card
/// rather than to the first — the test below this one.
#[test]
fn a_mill_is_one_batch_of_many_moves() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, angel_of_suffering(), 0);
    fill_library(&mut game, 0, 20);
    let source = source_for(&mut game, 1);
    let before = game.recorded_events().len();

    bolt_player(&mut game, source, 0, 3);

    let batches: Vec<Option<BatchId>> = game
        .recorded_events()
        .records_from(before)
        .iter()
        .filter(|r| matches!(r.event, GameEvent::ZoneChange { .. }))
        .map(|r| r.batch())
        .collect();
    assert_eq!(batches.len(), 6, "twice 3");
    assert!(batches[0].is_some());
    assert!(
        batches.iter().all(|b| *b == batches[0]),
        "CR 701.17a puts that many cards into the graveyard, once",
    );
}

/// The Angel is scoped by `PlayerSet::You`, so it does nothing about damage to
/// anyone else — the half a `PlayerSet::Everyone` would silently get wrong.
#[test]
fn angel_of_suffering_does_not_protect_the_opponent() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, angel_of_suffering(), 0);
    fill_library(&mut game, 0, 20);
    let source = source_for(&mut game, 0);

    bolt_player(&mut game, source, 1, 3);

    assert_eq!(life(&game, 1), 17);
    assert_eq!(game.players[0].library.len(), 20, "and nothing was milled");
}

/// The Angel's set names no object, so damage to a *permanent* its controller
/// controls is untouched — "if damage would be dealt to **you**".
#[test]
fn angel_of_suffering_does_not_protect_its_controllers_permanents() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, angel_of_suffering(), 0);
    fill_library(&mut game, 0, 20);
    let mine = place_vanilla_creature(&mut game, 0, 9, 9, &[]);
    let source = source_for(&mut game, 1);

    game.execute_action(
        GameAction::DealDamage {
            source,
            target: ObjectOrPlayerId::Object(mine),
            amount: 3,
            is_combat: false,
            unpreventable: false,
            source_lki: None,
        },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(game.battlefield[&mine].damage_marked, 3);
    assert_eq!(game.players[0].library.len(), 20);
}

/// The first mill in the crate meets a replacement effect for free: Leyline of
/// the Void has watched "a card would be put into a graveyard from anywhere"
/// since Phase RB, and `Primitive::Mill` proposes exactly that move.
#[test]
fn a_milled_card_meets_leyline_of_the_void() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, angel_of_suffering(), 0);
    put_on_battlefield(&mut game, printings::leyline_of_the_void(), 1);
    fill_library(&mut game, 0, 20);
    let source = source_for(&mut game, 1);

    bolt_player(&mut game, source, 0, 2);

    assert_eq!(game.players[0].library.len(), 16);
    assert!(game.players[0].graveyard.is_empty(), "exiled instead");
    assert_eq!(
        game.exile.len(),
        4,
        "Leyline of the Void replaced each mill's move, one card at a time",
    );
}

/// The damage cards' rows and rulings on a board built by hand, each
/// damage event proposed directly.
mod the_damage_cards {
    use std::sync::Arc;

    use mtgsim::cards::authoring::any_target;
    use mtgsim::cards::fixtures::loyalty_probe;
    use mtgsim::cards::printings::{
        circle_of_protection_red, dark_sphere, daunting_defender, fog, furnace_of_rath, ghosts_of_the_innocent,
        gisela_blade_of_goldnight, guardian_seraph, mending_hands, palisade_giant, pariah, reverse_damage,
        safe_passage, samite_censer_bearer, samite_healer, torbran_thane_of_red_fell,
    };
    use mtgsim::engine::actions::{ActionContext, GameAction};
    use mtgsim::engine::combat::resolution::assign_combat_damage;
    use mtgsim::engine::targeting::ChosenTargets;
    use mtgsim::objects::card_data::AbilityType;
    use mtgsim::objects::card_data::{CardData, CardDataBuilder};
    use mtgsim::state::game_state::GameState;
    use mtgsim::state::replacement_effects::ReplacementEffectId;
    use mtgsim::test_support::{
        place_vanilla_creature, put_on_battlefield, set_attacking, setup_two_player_game,
        test_ctx, RecordingDecisionProvider,
    };
    use mtgsim::types::card_types::CardType;
    use mtgsim::types::colors::Color;
    use mtgsim::types::costs::Cost;
    use mtgsim::types::effects::{CounterType, Duration, Effect, EffectRecipient, PatternFill, Primitive};
    use mtgsim::types::effects::{ObjectSet, PlayerSet};
    use mtgsim::types::ids::ObjectOrPlayerId;
    use mtgsim::types::mana::{ManaCost, ManaType};
    use mtgsim::types::replacement::{EventPattern, ReplacementDef, Rewrite};

    /// A 2/2 for P0 and a board with no replacement source but the one under
    /// test. Damage to a *creature* throughout: RD-1's tests of damage to a
    /// **player** need CR 120.3a's decomposition, which is two commits away,
    /// and they live in `tests/phase_rd_integration_test.rs`.
    fn board() -> (GameState, ObjectIdPair) {
        let mut game = setup_two_player_game();
        let bolt_source = place_vanilla_creature(&mut game, 0, 1, 1, &[]);
        let victim = place_vanilla_creature(&mut game, 1, 5, 5, &[]);
        (game, ObjectIdPair { source: bolt_source, victim })
    }

    struct ObjectIdPair {
        source: mtgsim::types::ids::ObjectId,
        victim: mtgsim::types::ids::ObjectId,
    }

    fn deal(game: &mut GameState, ids: &ObjectIdPair, amount: u64) {
        game.execute_action(
            GameAction::DealDamage {
                source: ids.source,
                target: ObjectOrPlayerId::Object(ids.victim),
                amount,
                is_combat: false,
                unpreventable: false,
                source_lki: None,
            },
            &test_ctx(),
        )
        .unwrap();
    }

    fn marked(game: &GameState, ids: &ObjectIdPair) -> u32 {
        game.battlefield[&ids.victim].damage_marked
    }

    // CR 701.10g on a real card: 3 becomes 6.
    #[test]
    fn furnace_of_rath_doubles_damage_to_a_permanent() {
        let (mut game, ids) = board();
        put_on_battlefield(&mut game, furnace_of_rath(), 0);
        deal(&mut game, &ids, 3);
        assert_eq!(marked(&game, &ids), 6);
    }

    // "The multiplied damage counts in all ways as if it came from the
    // original source. Furnace of Rath is not the source."
    #[test]
    fn furnace_of_rath_is_not_the_source_of_the_doubled_damage() {
        use mtgsim::events::event::GameEvent;
        let (mut game, ids) = board();
        let furnace = put_on_battlefield(&mut game, furnace_of_rath(), 0);
        deal(&mut game, &ids, 3);
        let dealt: Vec<_> = game
            .recorded_events()
            .events()
            .filter_map(|e| match e {
                GameEvent::DamageDealt { source, amount, .. } => Some((source.id, *amount)),
                _ => None,
            })
            .collect();
        assert_eq!(dealt, vec![(ids.source, 6)]);
        assert_ne!(dealt[0].0, furnace);
    }

    // "Half of 1 rounded down is 0. A source that would deal 1 damage won't
    // deal damage at all" — CR 614.7a drops the emptied event, so nothing is
    // marked and no `DamageDealt` is emitted.
    //
    // **Three boards, because "nothing happened" is not evidence on its own.**
    // The absence of a `DamageDealt` event cannot, by itself, distinguish
    // "Ghosts halved 1 to 0 and CR 614.7a dropped it" from "the damage never
    // reached the pipeline at all" — so the same 1 damage without Ghosts is the
    // control, and 2 damage with Ghosts shows the instance applying rather than
    // being skipped. What is still not observable here is *which rule* dropped
    // the event, which is a question for the trace sink (`roadmap-v2.md` A4c).
    #[test]
    fn ghosts_of_the_innocent_halves_one_damage_to_nothing() {
        use mtgsim::events::event::GameEvent;

        let (mut control, ids) = board();
        deal(&mut control, &ids, 1);
        assert_eq!(marked(&control, &ids), 1, "1 damage lands when nothing halves it");

        let (mut game, ids) = board();
        put_on_battlefield(&mut game, ghosts_of_the_innocent(), 0);
        deal(&mut game, &ids, 2);
        assert_eq!(marked(&game, &ids), 1, "and Ghosts is applying on this board");

        let (mut game, ids) = board();
        put_on_battlefield(&mut game, ghosts_of_the_innocent(), 0);
        deal(&mut game, &ids, 1);
        assert_eq!(marked(&game, &ids), 0);
        assert!(!game
            .recorded_events()
            .events()
            .any(|e| matches!(e, GameEvent::DamageDealt { .. })));
    }

    // "Multiple Ghosts of the Innocent effects are cumulative … with three on
    // the battlefield, 14 damage becomes 7, then 3, then finally 1."
    //
    // Three instances is three CR 616.1 candidates, so the affected side is
    // asked twice — and the answer cannot matter, because halving commutes with
    // itself. `picking_all` is the provider that would notice a candidate the
    // list should not have offered.
    #[test]
    fn three_ghosts_take_fourteen_damage_to_one() {
        let (mut game, ids) = board();
        for _ in 0..3 {
            put_on_battlefield(&mut game, ghosts_of_the_innocent(), 0);
        }
        let dp = RecordingDecisionProvider::picking_all();
        let ctx = ActionContext::new(&dp);
        game.execute_action(
            GameAction::DealDamage {
                source: ids.source,
                target: ObjectOrPlayerId::Object(ids.victim),
                amount: 14,
                is_combat: false,
                unpreventable: false,
                source_lki: None,
            },
            &ctx,
        )
        .unwrap();
        assert_eq!(marked(&game, &ids), 1);
        assert_eq!(dp.prompts(), 2, "three candidates, then two, then one");
    }

    // Gisela's doubler is scoped by CR 102.1's "an opponent", resolved against
    // her controller: P0's Gisela doubles damage to P1's creature and leaves
    // damage to P0's own alone.
    #[test]
    fn gisela_doubles_only_damage_to_an_opponents_permanent() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, gisela_blade_of_goldnight(), 0);
        let source = place_vanilla_creature(&mut game, 0, 1, 1, &[]);
        let theirs = place_vanilla_creature(&mut game, 1, 9, 9, &[]);
        let mine = place_vanilla_creature(&mut game, 0, 9, 9, &[]);

        for victim in [theirs, mine] {
            game.execute_action(
                GameAction::DealDamage {
                    source,
                    target: ObjectOrPlayerId::Object(victim),
                    amount: 3,
                    is_combat: false,
                    unpreventable: false,
                    source_lki: None,
                },
                &test_ctx(),
            )
            .unwrap();
        }
        assert_eq!(game.battlefield[&theirs].damage_marked, 6);
        // Her *other* half applies here: prevent half of 3, rounded up, is 2.
        assert_eq!(game.battlefield[&mine].damage_marked, 1);
    }

    // "Gisela doubles damage … from any source, including sources controlled
    // by those opponents." The pattern carries no source-side constraint, and
    // RD-3 is where one could be added by mistake.
    #[test]
    fn gisela_doubles_an_opponents_own_source() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, gisela_blade_of_goldnight(), 0);
        let their_source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        let their_victim = place_vanilla_creature(&mut game, 1, 9, 9, &[]);
        game.execute_action(
            GameAction::DealDamage {
                source: their_source,
                target: ObjectOrPlayerId::Object(their_victim),
                amount: 2,
                is_combat: false,
                unpreventable: false,
                source_lki: None,
            },
            &test_ctx(),
        )
        .unwrap();
        assert_eq!(game.battlefield[&their_victim].damage_marked, 4);
    }

    // CR 107.1a, on the board rather than on the arithmetic: Gisela rounds up,
    // so 5 damage to something she protects leaves 2.
    #[test]
    fn gisela_prevents_half_rounded_up() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, gisela_blade_of_goldnight(), 0);
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        let mine = place_vanilla_creature(&mut game, 0, 9, 9, &[]);
        game.execute_action(
            GameAction::DealDamage {
                source,
                target: ObjectOrPlayerId::Object(mine),
                amount: 5,
                is_combat: false,
                unpreventable: false,
                source_lki: None,
            },
            &test_ctx(),
        )
        .unwrap();
        assert_eq!(game.battlefield[&mine].damage_marked, 2);
    }

    // The fixture is a planeswalker that enters with its printed loyalty
    // (CR 306.5b), which is what CR 120.3c will take counters off.
    #[test]
    fn loyalty_probe_enters_with_three_loyalty() {
        let mut game = setup_two_player_game();
        let probe = put_on_battlefield(&mut game, loyalty_probe(), 0);
        assert_eq!(
            game.battlefield[&probe].counter_count(CounterType::Loyalty),
            3
        );
    }

    // -----------------------------------------------------------------------
    // RD-2
    // -----------------------------------------------------------------------

    use mtgsim::engine::resolve::ResolutionContext;
    use mtgsim::events::event::GameEvent;
    use mtgsim::state::replacement_effects::RegisteredReplacementEffect;
    use mtgsim::test_support::put_in_hand;
    use mtgsim::types::replacement::Uses;

    /// Resolve `card`'s spell effect for `controller` against `targets`, the
    /// way the stack would — the card's own id is the resolution's source.
    fn resolve_spell(
        game: &mut GameState,
        card: Arc<CardData>,
        controller: mtgsim::types::ids::PlayerId,
        targets: Vec<ObjectOrPlayerId>,
    ) -> mtgsim::types::ids::ObjectId {
        let id = put_in_hand(game, card.clone(), controller);
        let ctx = ResolutionContext {
            source: id,
            ability_source: None,
            controller,
            generated_by: mtgsim::events::cause::GeneratedBy::Spell,
            targets: ChosenTargets::one(targets),
            replaced_event: None,
            trigger: None,
        };
        let dp = mtgsim::test_support::test_dp();
        game.resolve_effect(&card.abilities[0].effect, &ctx, &dp).unwrap();
        id
    }

    fn rows(game: &GameState) -> Vec<&RegisteredReplacementEffect> {
        game.replacement_effects.iter().collect()
    }

    fn bolt(game: &mut GameState, source: mtgsim::types::ids::ObjectId, target: ObjectOrPlayerId, amount: u64) {
        game.execute_action(
            GameAction::DealDamage { source, target, amount, is_combat: false, unpreventable: false, source_lki: None },
            &test_ctx(),
        )
        .unwrap();
    }

    fn damage_dealt(game: &GameState) -> usize {
        game.recorded_events().events().filter(|e| matches!(e, GameEvent::DamageDealt { .. })).count()
    }

    // The plain shield: one row, on the player it targeted, counting four,
    // for this turn, with the resolution's target kept on it.
    #[test]
    fn mending_hands_makes_one_row_counting_four_on_its_target() {
        let mut game = setup_two_player_game();
        let spell = resolve_spell(&mut game, mending_hands(), 0, vec![ObjectOrPlayerId::Player(1)]);
        let rows = rows(&game);
        assert_eq!(rows.len(), 1);
        let row = rows[0];
        assert_eq!(row.def.uses, Uses::NextDamage(4));
        assert_eq!(row.def.affected.objects, ObjectSet::NO_OBJECTS);
        assert_eq!(row.def.affected.players, PlayerSet::Fixed(vec![1]));
        assert_eq!(row.duration, Duration::UntilEndOfTurn);
        assert_eq!(row.source, spell);
        assert_eq!(row.controller, 0);
        assert_eq!(row.targets, vec![mtgsim::types::ids::ObjectOrPlayer::Player(1)]);
        assert!(row.def.is_prevention());
    }

    // Targeting a creature fills the object half instead, and leaves the
    // player half empty.
    #[test]
    fn mending_hands_on_a_creature_fills_the_object_set() {
        let mut game = setup_two_player_game();
        let bear = place_vanilla_creature(&mut game, 1, 2, 2, &[]);
        resolve_spell(&mut game, mending_hands(), 0, vec![ObjectOrPlayerId::Object(bear)]);
        let row = rows(&game)[0];
        assert_eq!(row.def.affected.objects, ObjectSet::Fixed(vec![bear]));
        assert_eq!(row.def.affected.players, PlayerSet::Nobody);
    }

    // "Safe Passage prevents all damage, not just combat damage, that would be
    // dealt to you and creatures you control this turn."
    #[test]
    fn safe_passage_prevents_noncombat_damage_to_its_caster() {
        let mut game = setup_two_player_game();
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        resolve_spell(&mut game, safe_passage(), 0, Vec::new());
        bolt(&mut game, source, ObjectOrPlayerId::Player(0), 3);
        assert_eq!(game.players[0].life_total, 20);
        assert_eq!(damage_dealt(&game), 0, "CR 615.6 — the damage never happened");
    }

    // "Safe Passage will prevent damage dealt to creatures that weren't on
    // the battlefield at the time it resolved." The row is a `Filter`, asked
    // at the event — the opposite of CR 615.11's fixed-at-resolution rows.
    #[test]
    fn safe_passage_covers_a_creature_that_entered_after_it_resolved() {
        let mut game = setup_two_player_game();
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        resolve_spell(&mut game, safe_passage(), 0, Vec::new());
        let latecomer = place_vanilla_creature(&mut game, 0, 4, 4, &[]);
        let theirs = place_vanilla_creature(&mut game, 1, 4, 4, &[]);
        bolt(&mut game, source, ObjectOrPlayerId::Object(latecomer), 3);
        bolt(&mut game, source, ObjectOrPlayerId::Object(theirs), 3);
        assert_eq!(game.battlefield[&latecomer].damage_marked, 0);
        assert_eq!(game.battlefield[&theirs].damage_marked, 3, "an opponent's creature is not covered");
    }

    // "Safe Passage doesn't prevent damage that would be dealt to
    // planeswalkers you control." The filter is `ByType(Creature)`.
    #[test]
    fn safe_passage_does_not_cover_a_planeswalker_you_control() {
        let mut game = setup_two_player_game();
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        let probe = put_on_battlefield(&mut game, loyalty_probe(), 0);
        resolve_spell(&mut game, safe_passage(), 0, Vec::new());
        bolt(&mut game, source, ObjectOrPlayerId::Object(probe), 2);
        assert_eq!(game.battlefield[&probe].counter_count(CounterType::Loyalty), 1);
    }

    // "Safe Passage has no effect on damage that's already been dealt."
    #[test]
    fn safe_passage_has_no_effect_on_damage_already_dealt() {
        let mut game = setup_two_player_game();
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        let mine = place_vanilla_creature(&mut game, 0, 4, 4, &[]);
        bolt(&mut game, source, ObjectOrPlayerId::Object(mine), 2);
        resolve_spell(&mut game, safe_passage(), 0, Vec::new());
        assert_eq!(game.battlefield[&mine].damage_marked, 2, "still marked");
        bolt(&mut game, source, ObjectOrPlayerId::Object(mine), 2);
        assert_eq!(game.battlefield[&mine].damage_marked, 2, "and the next 2 are prevented");
    }

    // CR 113.7a — the row an ability makes names the permanent, which outlives
    // the stack object CR 608.2n deletes when the ability finishes resolving.
    // A real activation: `{T}` opens no mana window (CR 601.2g), so the one
    // prompt is the target.
    #[test]
    fn samite_healer_taps_for_a_count_of_one_and_the_row_names_the_healer() {
        use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
        use mtgsim::ui::decision::ScriptedDecisionProvider;

        let mut game = setup_two_player_game();
        let healer = put_on_battlefield(&mut game, samite_healer(), 0);
        let dp = ScriptedDecisionProvider::new();
        dp.expect_choice(
            ChoiceKind::SelectRecipients { recipient: any_target(), spell_id: healer },
            vec![ChoiceOption::Player(1)],
        );
        game.activate_ability(0, healer, 0, &dp).expect("{T} is payable");
        assert!(game.battlefield[&healer].tapped);
        game.resolve_top_of_stack(&dp).unwrap();

        let rows = rows(&game);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].source, healer);
        assert!(game.objects.contains_key(&rows[0].source), "and it still exists");
        assert_eq!(rows[0].def.uses, Uses::NextDamage(1));
        assert_eq!(rows[0].def.affected.players, PlayerSet::Fixed(vec![1]));
    }

    // The cost is the card's: {W} and itself, in that order.
    #[test]
    fn samite_censer_bearer_costs_white_and_itself() {
        let card = samite_censer_bearer();
        let ability = &card.abilities[0];
        assert!(matches!(ability.ability_type, AbilityType::Activated(_)));
        assert_eq!(
            ability.costs(),
            vec![Cost::Mana(ManaCost::build(&[ManaType::White], 0)), Cost::SacrificeSelf]
        );
        assert!(matches!(
            ability.effect,
            Effect::Atom(_, EffectRecipient::FilteredPermanents(_))
        ));
    }

    // -----------------------------------------------------------------------
    // RD-3 — the rulings pass
    // -----------------------------------------------------------------------

    use mtgsim::engine::layers::types::{EffectModification, Layer};
    use mtgsim::test_support::{lightning_bolt, put_spell_on_stack, registered};
    use mtgsim::types::colors::Color as TestColor;
    use mtgsim::types::replacement::SourcePattern as TestSourcePattern;

    /// Resolve `card`'s **activated** ability for `controller`, the way the
    /// stack would — CR 113.7a's source is the permanent, so the row an
    /// ability makes names it rather than the ephemeral stack object.
    fn activate(
        game: &mut GameState,
        card: Arc<CardData>,
        controller: mtgsim::types::ids::PlayerId,
        dp: &dyn mtgsim::ui::decision::DecisionProvider,
    ) -> mtgsim::types::ids::ObjectId {
        let permanent = put_on_battlefield(game, card.clone(), controller);
        let ctx = ResolutionContext {
            ability_source: game.object_ref(permanent),
            ..ResolutionContext::untargeted(permanent, controller, mtgsim::events::cause::GeneratedBy::Spell)
        };
        game.resolve_effect(&card.abilities[0].effect, &ctx, dp).unwrap();
        permanent
    }

    /// Take the **first** option at every prompt, for a board whose answer
    /// cannot change what it asserts: a CR 616.1 prompt the ruling says either
    /// order answers the same way, which each test asserts rather than
    /// assumes.
    fn first() -> mtgsim::test_support::RecordingDecisionProvider {
        mtgsim::test_support::RecordingDecisionProvider::picking(0)
    }

    /// Name `options` at every prompt of two or more: CR 609.7a's chosen
    /// source, wherever the candidates list it.
    fn naming(options: impl IntoIterator<Item = mtgsim::types::ids::ObjectId>) -> mtgsim::test_support::RecordingDecisionProvider {
        mtgsim::test_support::RecordingDecisionProvider::choosing(
            options.into_iter().map(mtgsim::ui::choice_types::ChoiceOption::Object),
        )
    }

    /// The chosen source on the one row in the registry.
    fn chosen_source(game: &GameState) -> Option<mtgsim::types::ids::ObjectId> {
        match &rows(game)[0].def.pattern {
            EventPattern::DealDamage { source: Some(p), .. } => p.object.map(|chosen| chosen.id),
            _ => None,
        }
    }

    fn life(game: &GameState, player: mtgsim::types::ids::PlayerId) -> i64 {
        game.players[player].life_total
    }

    // CR 615.8 — "these effects prevent the next instance of damage from that
    // source, regardless of how much damage that is". Seven is prevented
    // whole, and the next instance from the same source is dealt in full.
    //
    // COVERS: ATOM-615.8-001
    #[test]
    fn circle_of_protection_red_prevents_the_whole_next_instance_however_big() {
        let mut game = setup_two_player_game();
        // A red 1/1 for the opponent beside the Circle: the choice names it.
        let red_source = put_on_battlefield(&mut game, lightning_bolt_creature(), 1);
        activate(&mut game, circle_of_protection_red(), 0, &naming([red_source]));
        assert_eq!(chosen_source(&game), Some(red_source), "CR 609.7a, at creation");

        bolt(&mut game, red_source, ObjectOrPlayerId::Player(0), 7);
        assert_eq!(life(&game, 0), 20, "the whole instance, whatever its size");
        assert!(rows(&game).is_empty(), "and the row is spent (CR 615.8)");

        bolt(&mut game, red_source, ObjectOrPlayerId::Player(0), 3);
        assert_eq!(life(&game, 0), 17, "a later instance is dealt normally");
    }

    // CR 615.9 / 609.7b — "when the source would deal damage, the shield
    // rechecks the source's properties. If the properties no longer match, the
    // damage isn't prevented or replaced ... the shield isn't used up."
    //
    // The id still matches; the color does not, and that is what makes this
    // two fields rather than one.
    //
    // COVERS: ATOM-615.9-001
    // COVERS: ATOM-609.7b-001
    #[test]
    fn circle_of_protection_red_does_not_apply_and_is_not_spent_once_the_source_turns_blue() {
        let mut game = setup_two_player_game();
        let source = put_on_battlefield(&mut game, lightning_bolt_creature(), 1);
        activate(&mut game, circle_of_protection_red(), 0, &naming([source]));
        assert_eq!(chosen_source(&game), Some(source));

        let ts = game.allocate_timestamp();
        let mut blue = std::collections::HashSet::new();
        blue.insert(TestColor::Blue);
        game.continuous_effects.add(registered(
            source,
            Layer::Layer5Color,
            ts,
            EffectModification::SetColors(blue),
        ));

        bolt(&mut game, source, ObjectOrPlayerId::Player(0), 3);
        assert_eq!(life(&game, 0), 17, "no longer red, so nothing is prevented");
        assert_eq!(rows(&game).len(), 1, "and the shield is not used up");
    }

    // The ruling: "Can be used even when there is no damage to prevent. It
    // prevents the next damage (if any) from the source this turn." So the row
    // is real, sits unused, and goes at the cleanup step rather than at zero.
    #[test]
    fn circle_of_protection_red_can_be_activated_with_no_damage_to_prevent() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, lightning_bolt_creature(), 1);
        activate(&mut game, circle_of_protection_red(), 0, &first());
        assert_eq!(rows(&game).len(), 1);
        assert_eq!(rows(&game)[0].def.uses, Uses::Once);

        mtgsim::test_support::pass_turn(&mut game);
        assert!(rows(&game).is_empty(), "CR 615.3 — the duration expired");
    }

    // The ruling: "It only affects damage dealt by the source one time. If the
    // source damages you a second time this turn, the damage will not be
    // reversed." And the rider is the printed reader of RD-2's prevented
    // channel: 4 prevented is 4 life gained, once.
    #[test]
    fn reverse_damage_reverses_the_first_instance_only() {
        let mut game = setup_two_player_game();
        let source = put_on_battlefield(&mut game, lightning_bolt_creature(), 1);
        resolve_spell(&mut game, reverse_damage(), 0, Vec::new());
        assert_eq!(chosen_source(&game), Some(source));

        bolt(&mut game, source, ObjectOrPlayerId::Player(0), 4);
        assert_eq!(life(&game, 0), 24, "prevented, and gained that much");

        bolt(&mut game, source, ObjectOrPlayerId::Player(0), 4);
        assert_eq!(life(&game, 0), 20, "the second instance is not reversed");
    }

    // The ruling: "if the source would deal 5 damage after two of these
    // abilities have resolved, the first one prevents 2 damage, reducing it to
    // 3 damage, then the second one prevents a further 1 damage, reducing the
    // total damage dealt to 2." Two rows, each applied once (CR 614.5), and
    // the order does not matter because both halve the same way.
    #[test]
    fn two_dark_spheres_take_five_to_three_to_two() {
        let mut game = setup_two_player_game();
        let source = put_on_battlefield(&mut game, lightning_bolt_creature(), 1);
        let sphere = activate(&mut game, dark_sphere(), 0, &naming([source]));
        activate(&mut game, dark_sphere(), 0, &naming([source]));
        assert_eq!(rows(&game).len(), 2);

        // Two candidates, so CR 616.1 asks — and either answer is this one,
        // because both rows halve the same way.
        let dp = naming([sphere]);
        game.execute_action(
            GameAction::DealDamage {
                source,
                target: ObjectOrPlayerId::Player(0),
                amount: 5,
                is_combat: false,
                unpreventable: false,
                source_lki: None,
            },
            &ActionContext::new(&dp),
        )
        .unwrap();
        assert_eq!(life(&game, 0), 18, "5 - 2 - 1");
        assert!(rows(&game).is_empty(), "both spent");
    }

    // The ruling: "This ability prevents 1 damage from each source an opponent
    // controls each time that source would deal damage to you. It prevents 1
    // of any damage, not just combat damage." Two sources, one point off each.
    #[test]
    fn guardian_seraph_prevents_one_from_each_opposing_source() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, guardian_seraph(), 0);
        let first = place_vanilla_creature(&mut game, 1, 3, 3, &[]);
        let second = place_vanilla_creature(&mut game, 1, 3, 3, &[]);

        bolt(&mut game, first, ObjectOrPlayerId::Player(0), 3);
        bolt(&mut game, second, ObjectOrPlayerId::Player(0), 3);
        assert_eq!(life(&game, 0), 20 - 2 - 2, "1 off each event, not 1 in total");
    }

    // The other half of the source predicate: your own creature hitting you is
    // not "a source an opponent controls".
    #[test]
    fn guardian_seraph_does_not_prevent_damage_from_your_own_source() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, guardian_seraph(), 0);
        let yours = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
        bolt(&mut game, yours, ObjectOrPlayerId::Player(0), 3);
        assert_eq!(life(&game, 0), 17);
    }

    // The ruling: "The effects from multiple Guardian Seraphs are cumulative."
    // Two instances, each applied once (CR 614.5), so 3 becomes 1.
    #[test]
    fn two_guardian_seraphs_are_cumulative() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, guardian_seraph(), 0);
        put_on_battlefield(&mut game, guardian_seraph(), 0);
        let source = place_vanilla_creature(&mut game, 1, 3, 3, &[]);

        // Two candidates on one event: CR 616.1 asks, and both answers agree.
        let dp = first();
        game.execute_action(
            GameAction::DealDamage {
                source,
                target: ObjectOrPlayerId::Player(0),
                amount: 3,
                is_combat: false,
                unpreventable: false,
                source_lki: None,
            },
            &ActionContext::new(&dp),
        )
        .unwrap();
        assert_eq!(life(&game, 0), 19, "3 - 1 - 1");
    }

    // CR 609.7c — "the prevention or replacement applies to sources that are
    // permanents with that property **and to any sources that aren't on the
    // battlefield** that have that property." Lightning Bolt is a spell on the
    // stack: not a permanent, and inside Guardian Seraph's predicate because
    // an opponent controls it.
    //
    // COVERS: ATOM-609.7c-001
    #[test]
    fn guardian_seraph_reaches_a_spell_on_the_stack() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, guardian_seraph(), 0);
        let spell = put_spell_on_stack(&mut game, lightning_bolt(), 1);
        bolt(&mut game, spell, ObjectOrPlayerId::Player(0), 3);
        assert_eq!(life(&game, 0), 18, "a non-battlefield source, 3 - 1");
    }

    // Any source at all, and the target side does the work: a Cleric creature
    // you control takes one less. The CR's own Pyroclasm board is
    // `tests/phase_rd3_integration_test.rs`.
    #[test]
    fn daunting_defender_prevents_one_to_a_cleric_you_control() {
        let mut game = setup_two_player_game();
        let defender = put_on_battlefield(&mut game, daunting_defender(), 0);
        let not_a_cleric = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);

        bolt(&mut game, source, ObjectOrPlayerId::Object(defender), 2);
        bolt(&mut game, source, ObjectOrPlayerId::Object(not_a_cleric), 2);
        assert_eq!(game.battlefield[&defender].damage_marked, 1, "it is its own Cleric");
        assert_eq!(game.battlefield[&not_a_cleric].damage_marked, 2);
    }

    // The `combat` field's whole job, from both sides: combat damage is
    // prevented and non-combat damage is not.
    #[test]
    fn fog_prevents_combat_damage_and_lets_a_bolt_through() {
        let mut game = setup_two_player_game();
        resolve_spell(&mut game, fog(), 0, Vec::new());
        let source = place_vanilla_creature(&mut game, 1, 3, 3, &[]);

        game.execute_action(
            GameAction::DealDamage {
                source,
                target: ObjectOrPlayerId::Player(0),
                amount: 3,
                is_combat: true,
                unpreventable: false,
                source_lki: None,
            },
            &test_ctx(),
        )
        .unwrap();
        assert_eq!(life(&game, 0), 20, "combat damage");

        bolt(&mut game, source, ObjectOrPlayerId::Player(0), 3);
        assert_eq!(life(&game, 0), 17, "and non-combat damage goes through");
    }

    // Fog says "all", and its own caster's opponents are inside it: a creature
    // an opponent controls takes no combat damage either.
    #[test]
    fn fog_covers_every_permanent_and_every_player() {
        let mut game = setup_two_player_game();
        resolve_spell(&mut game, fog(), 0, Vec::new());
        let mine = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
        let theirs = place_vanilla_creature(&mut game, 1, 2, 2, &[]);

        for (source, target) in [(mine, theirs), (theirs, mine)] {
            game.execute_action(
                GameAction::DealDamage {
                    source,
                    target: ObjectOrPlayerId::Object(target),
                    amount: 2,
                    is_combat: true,
                    unpreventable: false,
                    source_lki: None,
                },
                &test_ctx(),
            )
            .unwrap();
        }
        assert_eq!(game.battlefield[&mine].damage_marked, 0);
        assert_eq!(game.battlefield[&theirs].damage_marked, 0);
    }

    // The ruling: "The additional 2 damage is dealt by the same source as the
    // original source of damage. The damage isn't dealt by Torbran unless
    // Torbran is the original source of damage."
    #[test]
    fn torbran_adds_two_and_the_damage_is_still_the_original_sources() {
        let mut game = setup_two_player_game();
        let torbran = put_on_battlefield(&mut game, torbran_thane_of_red_fell(), 0);
        let red_source = put_on_battlefield(&mut game, lightning_bolt_creature(), 0);

        bolt(&mut game, red_source, ObjectOrPlayerId::Player(1), 3);
        assert_eq!(life(&game, 1), 15, "3 + 2");
        let dealt: Vec<_> = game
            .recorded_events()
            .events()
            .filter_map(|e| match e {
                GameEvent::DamageDealt { source, amount, .. } => Some((source.id, *amount)),
                _ => None,
            })
            .collect();
        assert_eq!(dealt, vec![(red_source, 5)]);
        assert!(!dealt.iter().any(|(s, _)| *s == torbran), "Torbran deals none of it");
    }

    // Both sides of the predicate, each tested by the board that fails it: a
    // source you control that is not red, and a red source you do not control.
    #[test]
    fn torbran_asks_both_halves_of_its_source_predicate() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, torbran_thane_of_red_fell(), 0);
        let yours_not_red = place_vanilla_creature(&mut game, 0, 1, 1, &[]);
        let red_not_yours = put_on_battlefield(&mut game, lightning_bolt_creature(), 1);

        bolt(&mut game, yours_not_red, ObjectOrPlayerId::Player(1), 3);
        assert_eq!(life(&game, 1), 17, "colorless: not a red source");
        bolt(&mut game, red_not_yours, ObjectOrPlayerId::Player(1), 3);
        assert_eq!(life(&game, 1), 14, "red, but not yours");
    }

    // The ruling: "If all of the damage is prevented, Torbran's effect no
    // longer applies." A whole-event prevention leaves a 0-damage proposal,
    // CR 614.7a drops it on the next iteration, and Torbran is never gathered
    // — so nothing is added to nothing.
    #[test]
    fn torbran_does_not_apply_once_all_of_the_damage_is_prevented() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, torbran_thane_of_red_fell(), 0);
        let red_source = put_on_battlefield(&mut game, lightning_bolt_creature(), 0);
        // Fog, cast by the player about to be hit, over combat damage.
        resolve_spell(&mut game, fog(), 1, Vec::new());

        // Fog and Torbran both apply, so CR 616.1 asks the damaged player for
        // an order — the ruling's own sentence. Either answer is 0: Fog first
        // empties the event and CR 614.7a drops it before Torbran is gathered;
        // Torbran first makes it 5 and Fog prevents all 5.
        let before = damage_dealt(&game);
        let dp = first();
        game.execute_action(
            GameAction::DealDamage {
                source: red_source,
                target: ObjectOrPlayerId::Player(1),
                amount: 3,
                is_combat: true,
                unpreventable: false,
                source_lki: None,
            },
            &ActionContext::new(&dp),
        )
        .unwrap();
        assert_eq!(life(&game, 1), 20, "prevented, and nothing added to it");
        assert_eq!(damage_dealt(&game), before, "no damage event at all");
    }

    // A red creature, so a `SourcePattern`'s color clause has a printed board
    // to be asked on. Not a registered card: it exists to *be* a source.
    fn lightning_bolt_creature() -> Arc<CardData> {
        CardDataBuilder::new("Red Probe")
            .mana_cost(ManaCost::build(&[ManaType::Red], 0))
            .color(Color::Red)
            .card_type(CardType::Creature)
            .power_toughness(1, 1)
            .rules_text("")
            .build()
    }

    // -----------------------------------------------------------------------
    // RD-4 — the rulings pass
    // -----------------------------------------------------------------------

    /// A registry row inserted directly, for the shapes whose printed producer
    /// is a later PR.
    fn fixture_row(
        game: &mut GameState,
        source: mtgsim::types::ids::ObjectId,
        controller: mtgsim::types::ids::PlayerId,
        def: ReplacementDef,
    ) {
        let turn = game.turn_number;
        game.replacement_effects.add(RegisteredReplacementEffect {
            id: ReplacementEffectId::UNASSIGNED,
            source,
            resolution: mtgsim::events::event::ResolutionStamp { source, controller },
            made_by_text: None,
            controller,
            generated_by: mtgsim::events::cause::GeneratedBy::Spell,
            duration: Duration::UntilEndOfTurn,
            created_on_turn: turn,
            targets: Vec::new(),
            def,
        });
    }

    /// [`bolt`] under a caller's own decision provider — CR 616.1's prompt is
    /// the point of the boards below.
    fn bolt_with(
        game: &mut GameState,
        source: mtgsim::types::ids::ObjectId,
        target: ObjectOrPlayerId,
        amount: u64,
        ctx: &ActionContext,
    ) {
        game.execute_action(
            GameAction::DealDamage { source, target, amount, is_combat: false, unpreventable: false, source_lki: None },
            ctx,
        )
        .unwrap();
    }

    /// Damage marked on one permanent — the file's other `marked` takes the
    /// two-id board these tests do not use.
    fn marked_on(game: &GameState, id: mtgsim::types::ids::ObjectId) -> u32 {
        game.battlefield[&id].damage_marked
    }

    /// The ruling, verbatim: *"If you would be dealt combat damage, the damage dealt
    /// to the enchanted creature instead is still combat damage."*
    ///
    /// A fact about the *rewrite* rather than about the pattern —
    /// `Rewrite::Retarget` touches `target` and nothing else — and it matters
    /// beyond bookkeeping, because CR 510.2 and lifelink both read the flag.
    ///
    /// `GameEvent::DamageDealt` does not carry the flag, so the assertion is made
    /// through a reader of it: a `combat: Some(true)` row scoped to the host, which
    /// applies to the **redirected** event and prevents it. A rewrite that dropped
    /// the flag would leave that row unapplied and the host with 4 damage on it.
    #[test]
    fn pariah_redirected_combat_damage_is_still_combat_damage() {
        let mut game = setup_two_player_game();
        game.active_player = 1;
        let host = place_vanilla_creature(&mut game, 0, 2, 6, &[]);
        let aura = put_on_battlefield(&mut game, pariah(), 0);
        assert!(game.attach(aura, host));
        let attacker = place_vanilla_creature(&mut game, 1, 4, 4, &[]);
        set_attacking(&mut game, attacker, 0);
        let watcher = place_vanilla_creature(&mut game, 0, 1, 1, &[]);
        fixture_row(
            &mut game,
            watcher,
            0,
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: Some(true) },
                ObjectSet::Fixed(vec![host]),
                Rewrite::Prevent,
            ),
        );

        let from = game.recorded_events().len();
        let dp = RecordingDecisionProvider::picking(0);
        let assignments = assign_combat_damage(&game, &dp, 1, false);
        game.apply_combat_damage(assignments, &ActionContext::new(&dp)).unwrap();

        let dealt: Vec<mtgsim::types::ids::ObjectOrPlayer> = game
            .recorded_events()
            .records_from(from)
            .iter()
            .filter_map(|r| match &r.event {
                GameEvent::DamageDealt { recipient, .. } => Some(*recipient),
                _ => None,
            })
            .collect();
        assert!(dealt.is_empty(), "the combat-only row saw it: {:?}", dealt);
        assert_eq!(life(&game, 0), 20);
        assert_eq!(marked_on(&game, host), 0, "redirected, and then prevented as combat damage");
    }

    /// The control for the board above: the same row watching **non**-combat damage
    /// does not apply, so the redirect is all that happens.
    #[test]
    fn pariah_redirected_noncombat_damage_is_not_combat_damage() {
        let mut game = setup_two_player_game();
        let host = place_vanilla_creature(&mut game, 0, 2, 6, &[]);
        let aura = put_on_battlefield(&mut game, pariah(), 0);
        assert!(game.attach(aura, host));
        let watcher = place_vanilla_creature(&mut game, 0, 1, 1, &[]);
        fixture_row(
            &mut game,
            watcher,
            0,
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: Some(true) },
                ObjectSet::Fixed(vec![host]),
                Rewrite::Prevent,
            ),
        );
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        
        bolt(&mut game, source, ObjectOrPlayerId::Player(0), 3);

        assert_eq!(life(&game, 0), 20);
        assert_eq!(marked_on(&game, host), 3);
    }

    /// The other ruling: *"if a spell would deal 6 damage to you and you control
    /// multiple Pariahs, you may have that 6 damage dealt to either of the enchanted
    /// creatures instead, but you can't choose to have 3 damage dealt to each one."*
    ///
    /// One CR 616.1 prompt, and the whole amount on one host. The "can't divide"
    /// half falls out of the loop rather than being coded: the first redirect moves
    /// the event's subject off the player, so the second Pariah's `PlayerSet::You`
    /// stops matching and it is never offered again.
    #[test]
    fn two_pariahs_ask_which_and_the_whole_amount_lands_on_one_host() {
        let mut game = setup_two_player_game();
        let first = place_vanilla_creature(&mut game, 0, 2, 9, &[]);
        let second = place_vanilla_creature(&mut game, 0, 2, 9, &[]);
        let aura_one = put_on_battlefield(&mut game, pariah(), 0);
        let aura_two = put_on_battlefield(&mut game, pariah(), 0);
        assert!(game.attach(aura_one, first));
        assert!(game.attach(aura_two, second));
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);

        let dp = RecordingDecisionProvider::picking(1);
        let ctx = ActionContext::new(&dp);
        bolt_with(&mut game, source, ObjectOrPlayerId::Player(0), 6, &ctx);

        assert_eq!(dp.prompts(), 1, "CR 616.1 asks once, and only once");
        assert!(dp.kinds()[0] == "ChooseReplacementEffect", "{:?}", dp.kinds());
        assert_eq!(marked_on(&game, first) + marked_on(&game, second), 6, "nothing was lost");
        assert!(
            (marked_on(&game, first), marked_on(&game, second)) == (0, 6)
                || (marked_on(&game, first), marked_on(&game, second)) == (6, 0),
            "the whole 6 lands on one host, never 3 and 3: {:?}",
            (marked_on(&game, first), marked_on(&game, second))
        );
        assert_eq!(life(&game, 0), 20);
    }

    /// *"If you control more than one Palisade Giant, you choose which redirection
    /// effect to apply … You can't have 4 damage dealt to each Giant or choose to
    /// have the 8 damage dealt to you."*
    ///
    /// The last clause is what makes the effect mandatory: there is no decline, so
    /// the 8 never stays on the player.
    #[test]
    fn two_palisade_giants_ask_which_and_the_whole_amount_lands_on_one() {
        let mut game = setup_two_player_game();
        let first = put_on_battlefield(&mut game, palisade_giant(), 0);
        let second = put_on_battlefield(&mut game, palisade_giant(), 0);
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);

        let dp = RecordingDecisionProvider::picking(1);
        let ctx = ActionContext::new(&dp);
        bolt_with(&mut game, source, ObjectOrPlayerId::Player(0), 8, &ctx);

        assert_eq!(dp.prompts(), 1);
        assert_eq!(life(&game, 0), 20, "and never dealt to you");
        assert_eq!(marked_on(&game, first) + marked_on(&game, second), 8);
        assert!(
            marked_on(&game, first) == 8 || marked_on(&game, second) == 8,
            "the whole 8 on one Giant: {:?}",
            (marked_on(&game, first), marked_on(&game, second))
        );
    }

    /// The word "other", which is the whole of `NotSource`: damage aimed at the
    /// Giant itself is not redirected onto the Giant.
    ///
    /// Vacuous on a tree where the filter matches nothing, which is why it is here
    /// beside the test above rather than instead of it — together they pin the leaf
    /// from both sides.
    #[test]
    fn palisade_giant_does_not_redirect_the_damage_aimed_at_itself() {
        let mut game = setup_two_player_game();
        let giant = put_on_battlefield(&mut game, palisade_giant(), 0);
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        let dp = RecordingDecisionProvider::picking(0);
        let ctx = ActionContext::new(&dp);

        bolt_with(&mut game, source, ObjectOrPlayerId::Object(giant), 4, &ctx);

        assert_eq!(marked_on(&game, giant), 4, "dealt as proposed");
        assert_eq!(dp.prompts(), 0, "and nothing was offered to replace it");
    }

    // The `SourcePattern` a card writes before a resolution touches it: the
    // object empty, the filter the card's. `PatternFill::ChosenDamageSource`
    // asserts on the first half.
    #[test]
    fn a_chosen_source_card_authors_no_object() {
        for card in [circle_of_protection_red(), reverse_damage(), dark_sphere()] {
            let Effect::Atom(Primitive::CreateReplacement(def, _, fill), _) =
                &card.abilities[0].effect
            else {
                panic!("{} is a CreateReplacement", card.name);
            };
            assert_eq!(*fill, PatternFill::ChosenDamageSource);
            match &def.pattern {
                EventPattern::DealDamage { source: Some(p), .. } => {
                    assert_eq!(p.object, None, "{} authors no source id", card.name)
                }
                other => panic!("{} watches {:?}", card.name, other),
            }
        }
        // And the two that ask nothing say so.
        assert_eq!(
            TestSourcePattern::chosen(),
            TestSourcePattern { object: None, filter: None }
        );
    }
}
