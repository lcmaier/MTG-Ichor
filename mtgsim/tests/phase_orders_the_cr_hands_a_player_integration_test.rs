//! `orders-the-cr-hands-a-player`: CR 601.2h's payment order and CR 613.7m's
//! timestamps, asked only where the orders' outcomes differ
//! (`choices-architecture.md#decision-6--when-the-engine-asks-an-order-the-cr-hands-a-player`).
//!
//! **Each order's board is cast or activated, never staged**, since a staged
//! fixture cannot check a question: `engine::costs` asks it at CR 601.2h, on
//! that path. A board that reads the pool floats its mana through mana
//! abilities, since a direct write would skip the write the layers read it
//! through.

use std::cell::RefCell;

use std::sync::Arc;

use mtgsim::cards::authoring::{
    activated_ability, another, artifact, as_long_as, creature, creatures_you_control, draw_cards, each, gains_keyword, gets,
    has_base_power_and_toughness, static_ability, this_object, you,
};
use mtgsim::cards::fixtures::{
    forked_reliquary, godhead_of_awe_fixture, grudge_clerk, kudo_king_among_bears_fixture, momentous_fall_fixture,
    omnath_locus_of_mana_fixture, reckoner_of_debts, scattering_flicker, self_eating_engine, sentry_of_the_tapped_gate, tithing_engine,
};
use mtgsim::cards::printings::{grizzly_bears, myr_enforcer, savannah_lions};
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::actions::ActionContext;
use mtgsim::engine::costs::CannotPay;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_power, get_effective_toughness};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    assert_not_asked, fill_library, not_asked, place_forest, put_in_hand, put_on_battlefield, put_on_battlefield_this_turn,
    setup_two_player_game, test_dp, AtEachPrompt, RecordingDecisionProvider,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{AmountExpr, Condition, EffectRecipient};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption, CostPart, Rejection};
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::display::{option_label, rejection};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

/// `n` Forests tapped for {G} each, through the mana ability's own path.
fn float_green(game: &mut GameState, n: usize) {
    for _ in 0..n {
        let (forest, ability) = place_forest(game, 0);
        game.activate_mana_ability(0, forest, ability, &test_dp()).expect("an untapped Forest taps for mana");
    }
}

fn power_and_toughness(game: &GameState, id: ObjectId) -> (Option<i32>, Option<i32>) {
    (get_effective_power(game, id), get_effective_toughness(game, id))
}

/// The script's next answer: `creature` pays CR 601.2h's sacrifice.
fn sacrifices(answers: &ScriptedDecisionProvider, creature: ObjectId) {
    answers.expect_choice(
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id: ObjectId::UNASSIGNED, count: 1 },
        vec![ChoiceOption::Object(creature)],
    );
}

/// The script's next answer: CR 601.2h's parts, paid in this order.
fn pays_in_order(answers: &ScriptedDecisionProvider, parts: Vec<CostPart>) {
    let kind = ChoiceKind::OrderCostPayments { spell_or_ability_id: ObjectId::UNASSIGNED };
    answers.expect_order(kind, parts.into_iter().map(ChoiceOption::CostPart).collect());
}


/// A part that uses no object, as the option names it.
fn part(cost: Cost) -> CostPart {
    CostPart { cost, paid_with: Vec::new() }
}

/// Momentous Fall's mana and its sacrifice, as a payment names them.
fn the_mana() -> CostPart {
    part(Cost::Mana(ManaCost::build(&[ManaType::Green, ManaType::Green], 2)))
}

fn the_sacrifice() -> Cost {
    Cost::Sacrifice(creature(), 1)
}

fn sacrificing(creature: ObjectId) -> CostPart {
    CostPart { cost: the_sacrifice(), paid_with: vec![creature] }
}

/// `first` then `second`, or the other way round.
fn either_first(as_written: bool, first: CostPart, second: CostPart) -> Vec<CostPart> {
    if as_written { vec![first, second] } else { vec![second, first] }
}

/// `board`'s creature beside six floating green and Momentous Fall in hand,
/// cast with `parts` as the order: what the spell drew and gained, and the
/// board.
fn cast_the_fall_paying(board: impl Fn(&mut GameState) -> ObjectId, parts: impl Fn(ObjectId) -> Vec<CostPart>) -> (usize, i64, GameState) {
    let mut game = setup_two_player_game();
    game.enable_audit();
    fill_library(&mut game, 0, 10);
    let creature = board(&mut game);
    let fall = put_in_hand(&mut game, momentous_fall_fixture(), 0);
    float_green(&mut game, 6);

    let answers = ScriptedDecisionProvider::new();
    pays_in_order(&answers, parts(creature));
    game.cast_spell(0, fall, &ManaWindowStop::new(answers)).expect("six green pay {2}{G}{G}");
    let hand = game.players[0].hand.len();
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");
    (game.players[0].hand.len() - hand, game.players[0].life_total, game)
}

// ---------------------------------------------------------------------------
// What the order is an order of: a characteristic read off the pool
// ---------------------------------------------------------------------------

/// A static ability reads the pool as a walk input: every write moves what
/// the creature is, adding mana and spending it, with no frame served stale
/// (the memo's audit reads each frame again in a debug build).
#[test]
fn a_power_read_off_the_pool_moves_with_the_pool() {
    let mut game = setup_two_player_game();
    let omnath = put_on_battlefield(&mut game, omnath_locus_of_mana_fixture(), 0);
    assert_eq!(power_and_toughness(&game, omnath), (Some(1), Some(1)));

    float_green(&mut game, 2);
    assert_eq!(power_and_toughness(&game, omnath), (Some(3), Some(3)), "two unspent green mana");

    let bears = put_in_hand(&mut game, grizzly_bears(), 0);
    game.cast_spell(0, bears, &ManaWindowStop::new(test_dp())).expect("{1}{G} from exactly its cost");
    assert_eq!(game.players[0].mana_pool.unspent(ManaType::Green), 0);
    assert_eq!(power_and_toughness(&game, omnath), (Some(1), Some(1)), "spent");
}

/// Mana a player keeps is read until the step's end empties the pool (CR
/// 106.4, 500.5), which is a write like any other.
#[test]
fn a_pool_emptying_as_a_step_ends_is_read_too() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    fill_library(&mut game, 1, 5);
    let omnath = put_on_battlefield(&mut game, omnath_locus_of_mana_fixture(), 0);
    float_green(&mut game, 3);
    assert_eq!(power_and_toughness(&game, omnath), (Some(4), Some(4)));

    game.advance_turn(&test_dp()).expect("the phase ends");
    assert_eq!(power_and_toughness(&game, omnath), (Some(1), Some(1)), "the pool emptied");
}

/// A pool is its own player's: "you" is the ability's controller, so an
/// opponent's mana is no green mana you have.
#[test]
fn another_players_pool_is_not_yours() {
    let mut game = setup_two_player_game();
    let theirs = put_on_battlefield(&mut game, omnath_locus_of_mana_fixture(), 1);
    float_green(&mut game, 2);
    assert_eq!(power_and_toughness(&game, theirs), (Some(1), Some(1)));
}

// ---------------------------------------------------------------------------
// "The sacrificed creature": what a payment used, read back
// ---------------------------------------------------------------------------

/// The spell reads the creature its cost sacrificed, as it last existed: two
/// cards and 2 life for Grizzly Bears, and the record names the existence the
/// payment ended (CR 400.7), not the card now in the graveyard.
// COVERS-PARTIAL: ATOM-601.2h-003
#[test]
fn the_spell_reads_the_creature_its_cost_sacrificed() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let fall = put_in_hand(&mut game, momentous_fall_fixture(), 0);
    float_green(&mut game, 4);
    let existence = game.object_ref(bears).expect("on the battlefield");

    game.cast_spell(0, fall, &ManaWindowStop::new(test_dp())).expect("one creature, so nothing to choose");
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Graveyard, "the cost is paid");
    let paid_with = &game.stack_entries[&fall].cost_choices.paid_with;
    assert_eq!(paid_with.len(), 1);
    assert_eq!(paid_with[0].used.object(), Some(existence), "as the payment left it");

    let hand_before = game.players[0].hand.len();
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");
    assert_eq!(game.players[0].hand.len(), hand_before + 2, "its power");
    assert_eq!(game.players[0].life_total, 22, "its toughness");
}

// ---------------------------------------------------------------------------
// CR 601.2h: the order, asked where it changes the game
// ---------------------------------------------------------------------------

/// `ATOM-601.2h-003`'s board, from hand. Sacrificed before the mana is paid,
/// Omnath last existed with six green unspent, a 7/7; after, with two, a 3/3.
/// One board, two games, and the choice was the player's.
// COVERS: ATOM-601.2h-003
#[test]
fn sacrificing_omnath_before_or_after_the_mana_draws_a_different_number() {
    let omnath = |game: &mut GameState| put_on_battlefield(game, omnath_locus_of_mana_fixture(), 0);
    let (drawn, life, _) = cast_the_fall_paying(omnath, |omnath| vec![sacrificing(omnath), the_mana()]);
    assert_eq!((drawn, life), (7, 27), "the sacrifice first: 7 power and toughness");

    let (drawn, life, game) = cast_the_fall_paying(omnath, |omnath| vec![the_mana(), sacrificing(omnath)]);
    assert_eq!((drawn, life), (3, 23), "the mana first: 3 power and toughness");
    assert_eq!(game.players[0].mana_pool.unspent(ManaType::Green), 2, "the same mana is spent either way");
}

/// A CDA reads the pool too (CR 604.3): Serra Avatar's shape, over mana. It
/// applies to its own object with no row, so only the object's own ability
/// list can say so.
#[test]
fn a_cda_over_the_pool_makes_the_order_a_question() {
    let pool_sized = || {
        let green = || AmountExpr::UnspentMana(ManaType::Green);
        let text = "This creature's power and toughness are each equal to the amount of unspent green mana you have.";
        CardDataBuilder::new("Pool Sized")
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Elemental))
            .power_toughness(0, 0)
            .rules_text(text)
            .ability(static_ability(text, has_base_power_and_toughness(this_object(), green(), green())).characteristic_defining())
            .build()
    };
    let board = |game: &mut GameState| put_on_battlefield(game, pool_sized(), 0);
    assert_eq!(cast_the_fall_paying(board, |pooled| vec![sacrificing(pooled), the_mana()]).0, 6);
    assert_eq!(cast_the_fall_paying(board, |pooled| vec![the_mana(), sacrificing(pooled)]).0, 2);
}

/// A row over a filter reaches the creature sacrificed though it is printed
/// on no creature: the Bears read the pool through an anthem.
#[test]
fn a_row_over_a_filter_reading_the_pool_makes_the_order_a_question() {
    let anthem = || {
        let green = || AmountExpr::UnspentMana(ManaType::Green);
        let text = "Creatures you control get +1/+1 for each unspent green mana you have.";
        CardDataBuilder::new("Pool Anthem")
            .card_type(CardType::Enchantment)
            .rules_text(text)
            .ability(static_ability(text, gets(each(creatures_you_control()), green(), green())))
            .build()
    };
    let board = |game: &mut GameState| {
        put_on_battlefield(game, anthem(), 0);
        put_on_battlefield(game, grizzly_bears(), 0)
    };
    assert_eq!(cast_the_fall_paying(board, |bears| vec![sacrificing(bears), the_mana()]).0, 8);
    assert_eq!(cast_the_fall_paying(board, |bears| vec![the_mana(), sacrificing(bears)]).0, 4);
}

/// What a person reads at the question: each part by what it costs and what
/// pays it, so two sacrifices read apart, and why an order was refused.
#[test]
fn a_part_says_what_it_costs_and_what_pays_it() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(option_label(&game, &ChoiceOption::CostPart(the_mana())), "pay {2}{G}{G}");
    assert_eq!(option_label(&game, &ChoiceOption::CostPart(sacrificing(bears))), format!("sacrifice 1: Grizzly Bears ({bears})"));
    assert_eq!(
        rejection(&game, &Rejection::OrderLeavesACostUnpaid(Cost::TapSelf)),
        "That order leaves \"tap it\" unpaid, and unpayable costs can't be paid (CR 601.2h)",
    );
}

/// Two parts alike in every word and using nothing are one cost twice: either
/// order the same game, so nobody is asked.
#[test]
fn one_cost_twice_is_not_asked() {
    let mut game = setup_two_player_game();
    game.enable_audit();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let costs = [Cost::PayLife(2), Cost::PayLife(2)];
    let answers = test_dp();
    game.plan_payment(&costs, 0, bears, &ActionContext::paying_cost(&answers, 0, bears)).expect("4 life of 20");
    assert_not_asked(&game, 0, "OrderCostPayments", "every order leaves one game (CR 601.2h)");
}

/// The order is asked after the sacrifice is chosen, since which creature
/// pays decides whether it matters. The Bears read no pool, so nobody is
/// asked, and the log says so.
#[test]
fn sacrificing_a_creature_that_reads_no_pool_asks_no_order() {
    let mut game = setup_two_player_game();
    game.enable_audit();
    fill_library(&mut game, 0, 5);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    put_on_battlefield(&mut game, omnath_locus_of_mana_fixture(), 0);
    let fall = put_in_hand(&mut game, momentous_fall_fixture(), 0);
    float_green(&mut game, 4);

    let answers = ScriptedDecisionProvider::new();
    sacrifices(&answers, bears);
    game.cast_spell(0, fall, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");
    assert_not_asked(&game, 0, "OrderCostPayments", "every order leaves one game (CR 601.2h)");
}

/// A cost's {T} ahead of the sacrifice of its own source is the one order that
/// completes (Self-Eating Engine paying with itself), so it is no question,
/// and the log calls it what it is. Paying with another artifact, the two
/// parts commute.
#[test]
fn the_one_order_that_completes_is_not_asked() {
    let mut game = setup_two_player_game();
    game.enable_audit();
    fill_library(&mut game, 0, 5);
    let engine = put_on_battlefield(&mut game, self_eating_engine(), 0);
    game.activate_ability(0, engine, 0, &test_dp()).expect("it sacrifices itself, having tapped");
    assert_eq!(game.get_object(engine).unwrap().zone, Zone::Graveyard);
    assert_not_asked(&game, 0, "OrderCostPayments", "its one legal answer");

    let mut game = setup_two_player_game();
    game.enable_audit();
    fill_library(&mut game, 0, 5);
    let engine = put_on_battlefield(&mut game, self_eating_engine(), 0);
    let other = put_on_battlefield(&mut game, self_eating_engine(), 0);
    let answers = ScriptedDecisionProvider::new();
    sacrifices(&answers, other);
    game.activate_ability(0, engine, 0, &answers).expect("the other artifact pays");
    assert_not_asked(&game, 0, "OrderCostPayments", "every order leaves one game (CR 601.2h)");
}

/// CR 603.4 reads the intervening "if" as the event happens, so the part
/// ahead of the sacrifice decides whether Reckoner of Debts triggers: the
/// player's order, though neither part reads the other.
#[test]
fn a_watcher_reading_what_another_part_changes_makes_the_order_a_question() {
    for life_first in [true, false] {
        let mut game = setup_two_player_game();
        game.enable_audit();
        fill_library(&mut game, 0, 5);
        game.players[0].life_total = 11;
        let engine = put_on_battlefield(&mut game, tithing_engine(), 0);
        put_on_battlefield(&mut game, reckoner_of_debts(), 0);
        let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);

        let answers = ScriptedDecisionProvider::new();
        pays_in_order(&answers, either_first(life_first, part(Cost::PayLife(2)), sacrificing(bears)));
        game.activate_ability(0, engine, 0, &answers).expect("one creature, and life to pay");
        assert_eq!(game.pending_triggers.len(), usize::from(life_first), "at 9 life the Reckoner triggers, at 11 it does not");
    }
}

/// With nothing reading life as the creature dies, the same two parts
/// commute, so the question goes.
#[test]
fn the_same_parts_with_no_watcher_are_not_asked() {
    let mut game = setup_two_player_game();
    game.enable_audit();
    fill_library(&mut game, 0, 5);
    let engine = put_on_battlefield(&mut game, tithing_engine(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    game.activate_ability(0, engine, 0, &test_dp()).expect("one creature, and life to pay");
    assert_not_asked(&game, 0, "OrderCostPayments", "every order leaves one game (CR 601.2h)");
}

/// An ability of the creature sacrificed sees the life payment only if the
/// creature was still there.
#[test]
fn a_moved_object_watching_another_part_makes_the_order_a_question() {
    for life_first in [true, false] {
        let mut game = setup_two_player_game();
        game.enable_audit();
        fill_library(&mut game, 0, 5);
        let engine = put_on_battlefield(&mut game, tithing_engine(), 0);
        let clerk = put_on_battlefield(&mut game, grudge_clerk(), 0);

        let answers = ScriptedDecisionProvider::new();
        pays_in_order(&answers, either_first(life_first, part(Cost::PayLife(2)), sacrificing(clerk)));
        game.activate_ability(0, engine, 0, &answers).expect("one creature, and life to pay");
        assert_eq!(game.pending_triggers.len(), usize::from(life_first), "the trigger needs the creature there for the payment");
    }
}

/// Only orders that complete: each pair of parts can change the game, the
/// Bears last existing as a 3/3 or a 2/2 by the {T}, so all three are ordered,
/// and an answer paying the {T} after its own source has gone is refused,
/// naming the part it would leave unpaid. Nothing was paid.
// COVERS-PARTIAL: ATOM-601.2h-002
#[test]
fn an_order_leaving_a_part_unpaid_is_refused_and_asked_again() {
    let mut game = setup_two_player_game();
    game.enable_audit();
    fill_library(&mut game, 0, 5);
    let reliquary = put_on_battlefield(&mut game, forked_reliquary(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let another_creature = || CostPart { cost: Cost::Sacrifice(another(creature()), 1), paid_with: vec![bears] };

    let answers = ScriptedDecisionProvider::new();
    pays_in_order(&answers, vec![part(Cost::SacrificeSelf), part(Cost::TapSelf), another_creature()]);
    pays_in_order(&answers, vec![part(Cost::TapSelf), part(Cost::SacrificeSelf), another_creature()]);
    let refusals = RefCell::new(Vec::new());
    let seat = AtEachPrompt::new(answers, |game: &GameState, context: &ChoiceContext| {
        if matches!(context.kind, ChoiceKind::OrderCostPayments { .. }) {
            refusals.borrow_mut().push((context.rejected.clone(), game.battlefield.contains_key(&reliquary)));
        }
    });
    game.activate_ability(0, reliquary, 1, &seat).expect("the second order completes");

    let refused = Rejection::OrderLeavesACostUnpaid(Cost::TapSelf);
    assert_eq!(refusals.into_inner(), vec![(None, true), (Some(refused), true)], "asked again before anything is paid");
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Graveyard);
}

/// The summoning sickness rule forbids beginning to activate (CR 602.5,
/// 602.5a) and nothing after: once begun, {T} needs only an untapped source
/// (CR 118.3). So a creature hasty only by the creature its cost sacrifices
/// pays the two in either order, as one hasty of its own does, and where the
/// order shows, under its anthem, the player chooses it. With no haste at all
/// it is refused before anything is asked or paid.
#[test]
fn haste_is_read_as_the_activation_begins_and_never_at_the_payment() {
    let altar = |own_haste: bool| {
        let anthem = "As long as this creature is untapped, other creatures you control get +1/+1.";
        let activated = "{T}, Sacrifice another creature: Draw a card.";
        let builder = CardDataBuilder::new("Hasty Altar")
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Elemental))
            .power_toughness(1, 1)
            .rules_text("Hasty Altar's text")
            .ability(static_ability(anthem, as_long_as(Condition::SourceUntapped, gets(each(another(creatures_you_control())), 1, 1))))
            .ability(activated_ability(activated, vec![Cost::TapSelf, Cost::Sacrifice(another(creature()), 1)], draw_cards(you(), 1)));
        if own_haste { builder.keyword_flag(KeywordFlag::Haste).build() } else { builder.build() }
    };
    let haste_giver = || -> Arc<CardData> {
        let text = "Other creatures you control have haste.";
        CardDataBuilder::new("Haste Giver")
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Elemental))
            .power_toughness(1, 1)
            .rules_text(text)
            .ability(static_ability(text, gains_keyword(each(another(creatures_you_control())), KeywordFlag::Haste)))
            .build()
    };
    let another_creature = |paid_by| CostPart { cost: Cost::Sacrifice(another(creature()), 1), paid_with: vec![paid_by] };

    let mut game = setup_two_player_game();
    game.enable_audit();
    game.begin_turn(1, 0);
    fill_library(&mut game, 0, 5);
    let source = put_on_battlefield_this_turn(&mut game, altar(true), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let answers = ScriptedDecisionProvider::new();
    pays_in_order(&answers, vec![another_creature(bears), part(Cost::TapSelf)]);
    game.activate_ability(0, source, 1, &answers).expect("its own haste survives the sacrifice");
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Graveyard);

    let mut game = setup_two_player_game();
    game.enable_audit();
    game.begin_turn(1, 0);
    fill_library(&mut game, 0, 5);
    let source = put_on_battlefield_this_turn(&mut game, altar(false), 0);
    let giver = put_on_battlefield(&mut game, haste_giver(), 0);
    let answers = ScriptedDecisionProvider::new();
    pays_in_order(&answers, vec![another_creature(giver), part(Cost::TapSelf)]);
    game.activate_ability(0, source, 1, &answers).expect("the haste is read as the activation begins");
    assert_eq!(answers.remaining(), 0, "the player chose to sacrifice the haste's source first");
    assert_eq!(game.get_object(giver).unwrap().zone, Zone::Graveyard);
    assert!(game.battlefield[&source].tapped);

    let mut game = setup_two_player_game();
    game.begin_turn(1, 0);
    let source = put_on_battlefield_this_turn(&mut game, altar(false), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert!(game.activate_ability(0, source, 1, &test_dp()).is_err(), "no haste, so it cannot begin");
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Battlefield, "nothing paid");
}

/// "An artifact" and "a creature" over one artifact creature, the overlap no
/// count per filter sees: each part moves what the other needs, so no order
/// completes, and it is refused as it is planned, with nothing paid.
// COVERS-PARTIAL: ATOM-601.2h-002
#[test]
fn two_parts_using_one_object_are_refused_before_anything_moves() {
    let mut game = setup_two_player_game();
    game.enable_audit();
    let enforcer = put_on_battlefield(&mut game, myr_enforcer(), 0);
    let costs = [Cost::Sacrifice(artifact(), 1), the_sacrifice()];
    assert!(game.can_pay_costs(&costs, 0, enforcer).is_ok(), "each part alone is payable");

    let answers = test_dp();
    let refused = game.plan_payment(&costs, 0, enforcer, &ActionContext::paying_cost(&answers, 0, enforcer));
    assert!(refused.is_err_and(|why| why.contains("two parts use one object")));
    assert_eq!(game.get_object(enforcer).unwrap().zone, Zone::Battlefield, "nothing moved");
}

/// A player below 0 life, as Platinum Angel keeps one, still pays what costs no
/// life: the check for life payments reads what the payment pays, so a total of
/// nothing is never more than the player has.
#[test]
fn a_payment_of_no_life_is_payable_below_zero_life() {
    let mut game = setup_two_player_game();
    game.enable_audit();
    game.players[0].life_total = -3;
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    game.players[0].mana_pool.add(ManaType::White, 1);
    let costs = [Cost::Mana(ManaCost::build(&[ManaType::White], 0)), Cost::SacrificeSelf];

    let answers = test_dp();
    game.plan_payment(&costs, 0, bears, &ActionContext::paying_cost(&answers, 0, bears)).expect("no life in it");
}

/// Two life payments each pass CR 118.3's check and together are more than
/// the player has, so no order pays them. Refused as it is planned, since
/// partial payments are not allowed: no question, no life lost, nothing moved.
// COVERS-PARTIAL: ATOM-601.2h-002
#[test]
fn life_no_order_can_pay_is_refused_with_nothing_paid() {
    let mut game = setup_two_player_game();
    game.enable_audit();
    game.players[0].life_total = 4;
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let costs = [the_sacrifice(), Cost::PayLife(3), Cost::PayLife(2)];
    assert!(game.can_pay_costs(&costs, 0, bears).is_ok(), "each part alone is payable");

    let answers = test_dp();
    let refused = game.plan_payment(&costs, 0, bears, &ActionContext::paying_cost(&answers, 0, bears));
    assert!(refused.is_err_and(|why| why.contains("5 life in all")));
    assert_eq!(game.players[0].life_total, 4, "none of it paid");
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Battlefield, "nothing moved");
}

/// Two creatures: which one pays is CR 601.2h's choice, and the spell reads
/// that one and not the other.
#[test]
fn the_creature_chosen_is_the_creature_read() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let omnath = put_on_battlefield(&mut game, omnath_locus_of_mana_fixture(), 0);
    let fall = put_in_hand(&mut game, momentous_fall_fixture(), 0);
    float_green(&mut game, 4);

    let answers = ScriptedDecisionProvider::new();
    sacrifices(&answers, bears);
    game.cast_spell(0, fall, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");

    let hand_before = game.players[0].hand.len();
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");
    assert_eq!(game.players[0].hand.len(), hand_before + 2, "the Bears' power, not Omnath's");
    assert_eq!(game.get_object(omnath).unwrap().zone, Zone::Battlefield);
}

/// Every total a registered card can be paid with, of two or more parts: an
/// activation's cost, a spell's mana with each additional cost, and each
/// alternative cost. Paid on a board with something for each part to take,
/// each way its sacrifice can go: no order is asked, and the audit pays every
/// order on a clone and finds one game. A question here would be a new
/// question in every game that pays it. Its time grows with the registry;
/// `codebase-state.md` files what to do once that binds.
#[test]
fn no_registered_cost_asks_its_order() {
    let registry = CardRegistry::default_registry();
    let mut paid = 0;
    for name in registry.card_names() {
        let card = registry.create(name).expect("a registered card");
        let mana = card.mana_cost.clone().map(|mana| Cost::Mana(mana.with_x_as(0)));
        let additional = card.additional_costs.iter().map(|cost| mana.iter().chain(cost.costs()).cloned().collect::<Vec<_>>());
        let alternative = card.alternative_costs.iter().map(|cost| cost.costs().to_vec());
        let activated = card.abilities.iter().map(|ability| ability.costs().to_vec());
        for costs in additional.chain(alternative).chain(activated).filter(|costs| costs.len() >= 2) {
            for picking in 0..2 {
                let mut game = setup_two_player_game();
                game.enable_audit();
                fill_library(&mut game, 0, 5);
                let source = put_on_battlefield(&mut game, card.clone(), 0);
                put_on_battlefield(&mut game, grizzly_bears(), 0);
                put_on_battlefield(&mut game, savannah_lions(), 0);
                put_on_battlefield(&mut game, self_eating_engine(), 0);
                place_forest(&mut game, 0);
                for mana_type in [ManaType::White, ManaType::Blue, ManaType::Black, ManaType::Red, ManaType::Green, ManaType::Colorless] {
                    game.players[0].mana_pool.add(mana_type, 8);
                }
                // Only a part `remaining-costs` has yet to build may go unpaid
                // here: anything else is a cost this board failed to test.
                match game.can_pay_costs(&costs, 0, source) {
                    Err(CannotPay::Unchecked) => continue,
                    Err(refused) => panic!("{name}'s {costs:?}: {refused}"),
                    Ok(()) => {}
                }
                let answers = RecordingDecisionProvider::picking(picking);
                let ctx = ActionContext::paying_cost(&answers, 0, source);
                game.plan_payment(&costs, 0, source, &ctx).unwrap_or_else(|refused| panic!("{name}: {refused}"));
                assert!(!answers.kinds().iter().any(|kind| kind == "OrderCostPayments"), "{name} asked its order: {costs:?}");
                paid += 1;
            }
        }
    }
    assert!(paid > 0, "no registered cost reached the plan");
}

// ---------------------------------------------------------------------------
// CR 613.7m: timestamps received at once
// ---------------------------------------------------------------------------

/// `objects` exiled and returned as one event, by a spell cast from hand.
/// `chosen` answers each CR 613.7m question it raises, in APNAP order.
fn flickers(game: &mut GameState, objects: &[ObjectId], chosen: Vec<(PlayerId, Vec<ObjectId>)>) {
    let flicker = put_in_hand(game, scattering_flicker(), 0);
    float_blue(game);
    let answers = ScriptedDecisionProvider::new();
    let targets = objects.iter().copied().map(ChoiceOption::Object).collect();
    answers.expect_choice(ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED }, targets);
    for (player, order) in chosen {
        answers.expect_order(ChoiceKind::OrderSimultaneousEntries { player }, order.into_iter().map(ChoiceOption::Object).collect());
    }
    let seat = ManaWindowStop::new(answers);
    game.cast_spell(0, flicker, &seat).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&seat).expect("the spell resolves");
}

/// {2}{U}, floated: a test's own write, ahead of any frame that reads a pool.
fn float_blue(game: &mut GameState) {
    game.players[0].mana_pool.add(ManaType::Blue, 3);
}

/// CR 613.7m's choice, where it decides a layer: two of one player's
/// creatures arrive together, each making every other creature a fixed size,
/// and the later wins. The player chooses which that is.
#[test]
fn a_players_objects_arriving_together_are_stamped_in_the_order_they_choose() {
    for (later_is_godhead, lions_are) in [(true, 1), (false, 2)] {
        let mut game = setup_two_player_game();
        game.enable_audit();
        let kudo = put_on_battlefield(&mut game, kudo_king_among_bears_fixture(), 0);
        let godhead = put_on_battlefield(&mut game, godhead_of_awe_fixture(), 0);
        let lions = put_on_battlefield(&mut game, savannah_lions(), 0);

        let order = if later_is_godhead { vec![kudo, godhead] } else { vec![godhead, kudo] };
        flickers(&mut game, &[kudo, godhead], vec![(0, order)]);

        assert_eq!(power_and_toughness(&game, lions), (Some(lions_are), Some(lions_are)));
    }
}

/// The active player's objects get the earlier timestamps, whoever that is
/// and whichever order the spell named them in, so the nonactive player's
/// effect wins the layer. One object each, so nobody orders anything.
// COVERS: ATOM-613.7m-001
#[test]
fn apnap_order_stamps_the_active_players_objects_first() {
    for (active, lions_are) in [(0, 1), (1, 2)] {
        for kudo_named_first in [true, false] {
            let mut game = setup_two_player_game();
            game.enable_audit();
            game.active_player = active;
            let kudo = put_on_battlefield(&mut game, kudo_king_among_bears_fixture(), 0);
            let godhead = put_on_battlefield(&mut game, godhead_of_awe_fixture(), 1);
            let lions = put_on_battlefield(&mut game, savannah_lions(), 0);
            let named = if kudo_named_first { [kudo, godhead] } else { [godhead, kudo] };
            flickers(&mut game, &named, Vec::new());

            let (earlier, later) = if active == 0 { (kudo, godhead) } else { (godhead, kudo) };
            assert!(game.object_timestamp(earlier) < game.object_timestamp(later), "the active player's first");
            assert_eq!(power_and_toughness(&game, lions), (Some(lions_are), Some(lions_are)), "the later effect wins");
        }
    }
}

/// Each player orders their own, in APNAP order, and what generates no effect
/// keeps its place among them: the Bears hold the middle of the active
/// player's three, and the nonactive player's two come after all of them.
#[test]
fn each_player_orders_their_own_and_the_rest_keep_their_places() {
    let mut game = setup_two_player_game();
    game.enable_audit();
    let (kudo, bears, godhead) = (
        put_on_battlefield(&mut game, kudo_king_among_bears_fixture(), 0),
        put_on_battlefield(&mut game, grizzly_bears(), 0),
        put_on_battlefield(&mut game, godhead_of_awe_fixture(), 0),
    );
    let (their_kudo, their_godhead) = (put_on_battlefield(&mut game, kudo_king_among_bears_fixture(), 1), put_on_battlefield(&mut game, godhead_of_awe_fixture(), 1));
    let lions = put_on_battlefield(&mut game, savannah_lions(), 0);

    flickers(&mut game, &[kudo, bears, godhead, their_kudo, their_godhead], vec![(0, vec![godhead, kudo]), (1, vec![their_godhead, their_kudo])]);

    let stamps: Vec<_> = [godhead, bears, kudo, their_godhead, their_kudo].iter().map(|id| game.object_timestamp(*id)).collect();
    assert!(stamps.windows(2).all(|pair| pair[0] < pair[1]), "{stamps:?}");
    assert_eq!(power_and_toughness(&game, lions), (Some(2), Some(2)), "their Kudo is the latest");
}

/// Alike objects never reach the question: two Kudos say one thing.
#[test]
fn alike_objects_arriving_together_are_not_asked() {
    let mut game = setup_two_player_game();
    game.enable_audit();
    let first = put_on_battlefield(&mut game, kudo_king_among_bears_fixture(), 0);
    let second = put_on_battlefield(&mut game, kudo_king_among_bears_fixture(), 0);
    flickers(&mut game, &[first, second], Vec::new());
    assert_not_asked(&game, 0, "OrderSimultaneousEntries", "every order leaves one game (CR 613.7m)");
}

/// Effects that reach only their own object never meet, so no order of them
/// can show: each Omnath's power is its own.
#[test]
fn objects_whose_effects_reach_only_themselves_are_not_asked() {
    let mut game = setup_two_player_game();
    game.enable_audit();
    let first = put_on_battlefield(&mut game, omnath_locus_of_mana_fixture(), 0);
    let second = put_on_battlefield(&mut game, sentry_of_the_tapped_gate(), 0);
    flickers(&mut game, &[first, second], Vec::new());
    assert_not_asked(&game, 0, "OrderSimultaneousEntries", "every order leaves one game (CR 613.7m)");
}

/// An object generating no continuous effect reads no timestamp, so beside
/// one that does there is nothing to order and nothing claimed.
#[test]
fn one_generating_object_asks_nothing() {
    let mut game = setup_two_player_game();
    game.enable_audit();
    let kudo = put_on_battlefield(&mut game, kudo_king_among_bears_fixture(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    flickers(&mut game, &[kudo, bears], Vec::new());
    assert!(!not_asked(&game).iter().any(|line| line.contains("OrderSimultaneousEntries")), "{:#?}", not_asked(&game));
    assert_eq!(power_and_toughness(&game, bears), (Some(2), Some(2)));
}
