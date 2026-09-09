//! Phase RD-1 — the damage event's two subjects and its results.
//!
//! CR 614.5, 615.10, 616.1, 701.10g and 120.3a/c, against the four printed
//! cards and the one fixture in `cards::phase_rd_cards`.
//!
//! The card file's own tests deal damage to a *creature*, because that path
//! existed before this phase. Everything here needs one of the two things
//! RD-1 adds to the board: an effect scoped to a **player**, or one of
//! CR 120.3's **results** — life loss and loyalty removal — that
//! `perform_action` did not decompose the damage into.

use mtgsim::cards::phase_rd_cards::{
    angel_of_suffering, furnace_of_rath, ghosts_of_the_innocent, gisela_blade_of_goldnight,
    loyalty_probe,
};
use mtgsim::cards::{keyword_creatures, phase_rb_cards};
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::events::event::{BatchId, DamageTarget, GameEvent};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    fill_library, place_vanilla_creature, put_on_battlefield, set_attacking, set_blocked_by,
    set_blocking, setup_two_player_game, test_ctx, RecordingDecisionProvider,
};
use mtgsim::types::effects::CounterType;
use mtgsim::engine::combat::resolution::assign_combat_damage;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::ui::choice_types::ChoiceKind;
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
            target: DamageTarget::Player(victim),
            amount,
            is_combat: false,
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
            target: DamageTarget::Player(victim),
            amount,
            is_combat: false,
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
    let before = game.events.len();

    bolt_player(&mut game, source, 1, 3);

    // `ATOM-120.3a-001`'s whole board and expected result: a player at 20 takes
    // 3 damage from a source without infect and is at 17. The batch assertions
    // below are this phase's; this line is the atom's.
    assert_eq!(life(&game, 1), 17);
    let records = game.events.records_from(before);
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

    let before = game.counters.replacement_gathers();
    game.execute_action(
        GameAction::DealDamage {
            source,
            target: DamageTarget::Object(victim),
            amount: 1,
            is_combat: false,
        },
        &test_ctx(),
    )
    .unwrap();
    let object_gathers = game.counters.replacement_gathers() - before;

    let before = game.counters.replacement_gathers();
    bolt_player(&mut game, source, 1, 1);
    let player_gathers = game.counters.replacement_gathers() - before;

    assert_eq!(object_gathers, 1, "the damage itself");
    assert_eq!(
        player_gathers,
        object_gathers + 1,
        "the damage, and the CR 120.3a life loss it contains",
    );
}

/// The log line the decomposition must not change: `LifeChanged` still names
/// the damage's source, which is what `LifeLossCause::Damage` carries.
#[test]
fn the_life_change_from_damage_still_names_the_damage_source() {
    let mut game = setup_two_player_game();
    let source = source_for(&mut game, 0);
    let before = game.events.len();

    bolt_player(&mut game, source, 1, 4);

    let sources: Vec<Option<ObjectId>> = game
        .events
        .records_from(before)
        .iter()
        .filter_map(|r| match r.event {
            GameEvent::LifeChanged { source, .. } => Some(source),
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
    let before = game.events.len();

    bolt_player(&mut game, source, 0, 3);

    assert_eq!(life(&game, 0), 20);
    assert!(
        !game
            .events
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
            target: DamageTarget::Object(probe),
            amount: 3,
            is_combat: false,
        },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(game.battlefield[&probe].counter_count(CounterType::Loyalty), 2);
}

/// CR 704.5i — a planeswalker with no loyalty counters is put into its owner's
/// graveyard. Measured at **0** across 200 stress games before this phase
/// (`cards::phase_sba_cards`), because nothing could take a loyalty counter
/// off.
#[test]
fn lethal_damage_to_a_planeswalker_reaches_cr_704_5i() {
    let mut game = setup_two_player_game();
    let probe = put_on_battlefield(&mut game, loyalty_probe(), 1);
    let source = source_for(&mut game, 0);

    game.execute_action(
        GameAction::DealDamage {
            source,
            target: DamageTarget::Object(probe),
            amount: 3,
            is_combat: false,
        },
        &test_ctx(),
    )
    .unwrap();
    assert_eq!(game.battlefield[&probe].counter_count(CounterType::Loyalty), 0);

    let decisions = ScriptedDecisionProvider::new();
    assert!(game.check_state_based_actions(&decisions).unwrap());
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
            target: DamageTarget::Object(probe),
            amount: 2,
            is_combat: false,
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
            target: DamageTarget::Object(enchantment),
            // Furnace doubles it; the point is that 6 lands nowhere.
            amount: 3,
            is_combat: false,
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
    // `battlefield_ids_ordered` is CR 613.7 timestamp order, so the Ghosts
    // entering first is candidate 0 and the Furnace candidate 1.
    let outcomes: Vec<i64> = [0usize, 1]
        .into_iter()
        .map(|pick| {
            let mut game = setup_two_player_game();
            put_on_battlefield(&mut game, ghosts_of_the_innocent(), 0);
            put_on_battlefield(&mut game, furnace_of_rath(), 0);
            let source = source_for(&mut game, 0);
            let dp = RecordingDecisionProvider::picking(pick);
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
    let outcomes: Vec<i64> = [0usize, 1]
        .into_iter()
        .map(|pick| {
            let mut game = setup_two_player_game();
            put_on_battlefield(&mut game, gisela_blade_of_goldnight(), 0);
            put_on_battlefield(&mut game, furnace_of_rath(), 1);
            let source = source_for(&mut game, 1);
            let dp = RecordingDecisionProvider::picking(pick);
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
                target: DamageTarget::Object(mine),
                amount: 3,
                is_combat: false,
            },
            GameAction::DealDamage {
                source,
                target: DamageTarget::Object(theirs),
                amount: 3,
                is_combat: false,
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
    let mammoth = put_on_battlefield(&mut game, keyword_creatures::war_mammoth(), 0);
    let blocker = place_vanilla_creature(&mut game, 1, 2, 2, &[]);

    set_attacking(&mut game, mammoth, 1);
    set_blocked_by(&mut game, mammoth, vec![blocker]);
    set_blocking(&mut game, blocker, vec![mammoth]);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_allocation(
        ChoiceKind::AssignTrampleDamage {
            attacker_id: mammoth,
            defending_target: DamageTarget::Player(1),
        },
        vec![2, 1],
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
/// `ResolvedTarget::Player` — and reads the event's amount through
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
    let before = game.events.len();

    bolt_player(&mut game, source, 0, 3);

    let batches: Vec<Option<BatchId>> = game
        .events
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
            target: DamageTarget::Object(mine),
            amount: 3,
            is_combat: false,
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
    put_on_battlefield(&mut game, phase_rb_cards::leyline_of_the_void(), 1);
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
