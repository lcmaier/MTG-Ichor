//! `divided-among-targets`: an effect divided among its targets (CR 601.2d),
//! announced on a cast, an activation and a trigger alike (602.2b, 603.3d), and
//! acted on by each target's share
//! (`choices-architecture.md#decision-5--where-a-division-lives`).

use std::sync::Arc;

use mtgsim::cards::authoring::{deal_damage, deal_damage_divided, destroy, divided_among, spell_ability, that_target};
use mtgsim::cards::fixtures::{
    dragonlord_atarka_fixture, elven_rite_fixture, magma_opus_fixture, meteor_swarm_fixture, mogg_mob_fixture, remedy_fixture, rolling_thunder_fixture,
};
use mtgsim::cards::printings::{forest, grizzly_bears, lightning_bolt, savannah_lions};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::x_values::LegalXValues;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_in_hand, put_on_battlefield, setup_game, test_dp};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{AmountExpr, CounterType, DividedAmong, Effect, EffectRecipient, SelectionFilter};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

/// The script's next answer: `chosen` for the announcement of an instance of
/// "target". The queue matches a kind by its variant alone.
fn names_targets(answers: &ScriptedDecisionProvider, of: ObjectId, chosen: Vec<ChoiceOption>) {
    answers.expect_choice(ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: of }, chosen);
}

/// The script's next answer: `shares` for CR 601.2d's division.
fn divides(answers: &ScriptedDecisionProvider, of: ObjectId, shares: Vec<(ChoiceOption, u64)>) {
    answers.expect_allocation(ChoiceKind::DivideAmong { source: of }, shares);
}

/// What `on_stack`'s first instance announced, each target with its share.
fn division(game: &GameState, on_stack: ObjectId) -> Vec<(ObjectOrPlayerId, u64)> {
    let instance = &game.stack_entries[&on_stack].chosen_targets[0];
    instance.division().expect("the instance divides").iter().map(|(target, share)| (target.id(), *share)).collect()
}

fn damage_on(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].damage_marked
}

fn the_asked(answers: &ScriptedDecisionProvider) -> Vec<&'static str> {
    answers.asked().iter().map(ChoiceKind::as_str).collect()
}

/// CR 602.2b: an activation announces the division as a cast does, ahead of
/// its cost. The sacrifice that pays leaves the shares standing, and the
/// source deals them as it last existed.
#[test]
fn an_activation_divides_ahead_of_its_cost() {
    let mut game = setup_game(2);
    let mob = put_on_battlefield(&mut game, mogg_mob_fixture(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, mob, vec![ChoiceOption::Object(bears), ChoiceOption::Player(1)]);
    divides(&answers, mob, vec![(ChoiceOption::Player(1), 2), (ChoiceOption::Object(bears), 1)]);

    game.activate_ability(0, mob, 0, &answers).expect("sacrificing it pays");

    let ability = *game.stack.last().expect("the ability is on the stack");
    assert_eq!(division(&game, ability), vec![(ObjectOrPlayerId::Object(bears), 1), (ObjectOrPlayerId::Player(1), 2)]);
    assert_eq!(the_asked(&answers), ["SelectRecipients", "DivideAmong"]);
    assert_eq!(game.objects[&mob].zone, Zone::Graveyard, "the cost is paid");

    game.resolve_top_of_stack(&test_dp()).expect("the ability resolves");
    assert_eq!(damage_on(&game, bears), 1);
    assert_eq!(game.players[1].life_total, 18);
}

/// CR 603.3d: a trigger divides as it goes on the stack, here among two
/// opponents' creatures, each player's seat its own.
#[test]
fn a_trigger_divides_as_it_goes_on_the_stack() {
    let mut game = setup_game(3);
    let atarka = put_in_hand(&mut game, dragonlord_atarka_fixture(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let lions = put_on_battlefield(&mut game, savannah_lions(), 2);
    game.players[0].mana_pool.add(ManaType::Red, 6);
    game.players[0].mana_pool.add(ManaType::Green, 1);
    game.cast_spell(0, atarka, &ManaWindowStop::new(test_dp())).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).expect("the creature enters");

    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, atarka, vec![ChoiceOption::Object(bears), ChoiceOption::Object(lions)]);
    divides(&answers, atarka, vec![(ChoiceOption::Object(bears), 1), (ChoiceOption::Object(lions), 4)]);
    game.place_pending_triggers(&answers).expect("the trigger is placed");

    let trigger = *game.stack.last().expect("the trigger is on the stack");
    assert_eq!(division(&game, trigger), vec![(ObjectOrPlayerId::Object(bears), 1), (ObjectOrPlayerId::Object(lions), 4)]);
    game.resolve_top_of_stack(&test_dp()).expect("the trigger resolves");
    assert_eq!(damage_on(&game, bears), 1);
    assert_eq!(damage_on(&game, lions), 4);
}

/// CR 115.6: "any number" may be none. The trigger stays on the stack with no
/// targets and resolves to nothing, where a clause of one or more would leave
/// it (CR 603.3d).
#[test]
fn any_number_may_be_none() {
    let mut game = setup_game(2);
    let atarka = put_in_hand(&mut game, dragonlord_atarka_fixture(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 6);
    game.players[0].mana_pool.add(ManaType::Green, 1);
    game.cast_spell(0, atarka, &ManaWindowStop::new(test_dp())).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).expect("the creature enters");

    let answers = ScriptedDecisionProvider::new();
    game.place_pending_triggers(&answers).expect("the trigger is placed");

    let trigger = *game.stack.last().expect("nothing is required of the trigger, so it stays");
    assert_eq!(game.stack_entries[&trigger].chosen_targets[0].division(), None, "nothing to divide among");
    game.resolve_top_of_stack(&test_dp()).expect("the trigger resolves");
    assert!(game.stack.is_empty());
}

/// CR 601.2c then 601.2d: every target is announced before any division, so
/// the second instance is named before the first divides.
#[test]
fn the_division_follows_every_instances_targets() {
    let mut game = setup_game(2);
    let opus = put_in_hand(&mut game, magma_opus_fixture(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let (one, other) = (put_on_battlefield(&mut game, forest(), 1), put_on_battlefield(&mut game, forest(), 1));
    game.players[0].mana_pool.add(ManaType::Red, 5);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, opus, vec![ChoiceOption::Object(bears), ChoiceOption::Player(1)]);
    names_targets(&answers, opus, vec![ChoiceOption::Object(one), ChoiceOption::Object(other)]);
    divides(&answers, opus, vec![(ChoiceOption::Object(bears), 1), (ChoiceOption::Player(1), 3)]);

    let seat = ManaWindowStop::new(answers);
    game.cast_spell(0, opus, &seat).expect("castable from exactly its cost");

    assert_eq!(the_asked(seat.inner()), ["SelectRecipients", "SelectRecipients", "DivideAmong"]);
    assert_eq!(game.stack_entries[&opus].chosen_targets[1].division(), None, "tapping divides nothing");
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");
    assert_eq!(game.players[1].life_total, 17);
    assert!(game.battlefield[&one].tapped && game.battlefield[&other].tapped);
}

/// CR 601.2d's "or counters": two among two is one split, and two on one
/// target is the other. Neither asks.
#[test]
fn counters_are_distributed_by_each_targets_share() {
    let mut game = setup_game(2);
    let rite = put_in_hand(&mut game, elven_rite_fixture(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let lions = put_on_battlefield(&mut game, savannah_lions(), 0);
    game.players[0].mana_pool.add(ManaType::Green, 2);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, rite, vec![ChoiceOption::Object(bears), ChoiceOption::Object(lions)]);

    let seat = ManaWindowStop::new(answers);
    game.cast_spell(0, rite, &seat).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");

    assert_eq!(the_asked(seat.inner()), ["SelectRecipients"], "one split is left, so it is not asked");
    for creature in [bears, lions] {
        assert_eq!(game.battlefield[&creature].counter_count(CounterType::PlusOnePlusOne), 1);
    }

    let again = put_in_hand(&mut game, elven_rite_fixture(), 0);
    game.players[0].mana_pool.add(ManaType::Green, 2);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, again, vec![ChoiceOption::Object(bears)]);
    game.cast_spell(0, again, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");
    assert_eq!(game.battlefield[&bears].counter_count(CounterType::PlusOnePlusOne), 3);
}

/// CR 615.7: each target a shield of its own share, so three on the Bears
/// stops a Bolt there, and two on the player leaves one through.
#[test]
fn a_divided_prevention_gives_each_target_its_own_shield() {
    let mut game = setup_game(2);
    let remedy = put_in_hand(&mut game, remedy_fixture(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.players[0].mana_pool.add(ManaType::White, 2);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, remedy, vec![ChoiceOption::Object(bears), ChoiceOption::Player(1)]);
    divides(&answers, remedy, vec![(ChoiceOption::Object(bears), 3), (ChoiceOption::Player(1), 2)]);
    game.cast_spell(0, remedy, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).expect("the shields are made");

    for target in [ChoiceOption::Object(bears), ChoiceOption::Player(1)] {
        let bolt = put_in_hand(&mut game, lightning_bolt(), 0);
        game.players[0].mana_pool.add(ManaType::Red, 1);
        let answers = ScriptedDecisionProvider::new();
        names_targets(&answers, bolt, vec![target]);
        game.cast_spell(0, bolt, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");
        game.resolve_top_of_stack(&test_dp()).expect("the Bolt resolves");
    }

    assert_eq!(damage_on(&game, bears), 0, "three on the Bears, three prevented");
    assert_eq!(game.players[1].life_total, 19, "two on the player, one through");
}

/// CR 601.2d's total may be the X announced before the targets (601.2b), and
/// each target's share is read off the announcement, not the resolving X.
#[test]
fn a_total_of_x_is_divided_as_the_targets_are_announced() {
    let mut game = setup_game(2);
    let thunder = put_in_hand(&mut game, rolling_thunder_fixture(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.players[0].mana_pool.add(ManaType::Red, 4);
    let answers = ScriptedDecisionProvider::new();
    answers.expect_number(ChoiceKind::ChooseXValue { spell_id: thunder, values: Box::new(LegalXValues::zero_through(3)) }, 3);
    names_targets(&answers, thunder, vec![ChoiceOption::Object(bears), ChoiceOption::Player(1)]);
    divides(&answers, thunder, vec![(ChoiceOption::Object(bears), 1), (ChoiceOption::Player(1), 2)]);

    game.cast_spell(0, thunder, &ManaWindowStop::new(answers)).expect("{X}{R} with X = 3");
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");

    assert_eq!(damage_on(&game, bears), 1);
    assert_eq!(game.players[1].life_total, 18);
}

/// CR 601.2d: nine targets can't each receive one of eight, so the
/// announcement fails and the cast rewinds (CR 732.1).
#[test]
fn a_count_past_the_total_rewinds_the_cast() {
    let mut game = setup_game(2);
    let swarm = put_in_hand(&mut game, meteor_swarm_fixture(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 10);
    let answers = ScriptedDecisionProvider::new();
    answers.expect_number(ChoiceKind::ChooseXValue { spell_id: swarm, values: Box::new(LegalXValues::zero_through(9)) }, 9);

    let refused = game.cast_spell(0, swarm, &ManaWindowStop::new(answers)).expect_err("nine targets, eight damage");

    assert!(refused.contains("601.2d"), "{refused}");
    assert_eq!(game.objects[&swarm].zone, Zone::Hand, "the cast rewinds");
}

/// A later atom naming a divided instance reads its targets and prints its own
/// amount: only the atom that declared the division acts by the shares.
#[test]
fn a_later_atom_naming_the_instance_reads_no_share() {
    let mut game = setup_game(2);
    let text = "This spell deals 3 damage divided as you choose among one or two targets, then 1 damage to each of them.";
    let card: Arc<CardData> = CardDataBuilder::new("Divided Then Each")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![deal_damage_divided(SelectionFilter::Any, DividedAmong::one_to(2), 3), deal_damage(that_target(0), 1)]),
        ))
        .build();
    let spell = put_in_hand(&mut game, card, 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.players[0].mana_pool.add(ManaType::Red, 1);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, spell, vec![ChoiceOption::Object(bears), ChoiceOption::Player(1)]);
    divides(&answers, spell, vec![(ChoiceOption::Object(bears), 1), (ChoiceOption::Player(1), 2)]);

    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");

    assert_eq!(damage_on(&game, bears), 2, "its share of 1, then 1");
    assert_eq!(game.players[1].life_total, 17, "its share of 2, then 1");
}

/// A verb no text divides is refused at resolution, loudly, rather than acting
/// on each target as if it had the whole.
#[test]
fn a_verb_that_cannot_divide_is_refused() {
    let mut game = setup_game(2);
    let text = "Destroy one or two target creatures, divided as you choose.";
    let clause = divided_among(SelectionFilter::Creature, DividedAmong::one_to(2), AmountExpr::Fixed(2));
    let card: Arc<CardData> = CardDataBuilder::new("Undividable")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(text, destroy(clause)))
        .build();
    let spell = put_in_hand(&mut game, card, 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.players[0].mana_pool.add(ManaType::Red, 1);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, spell, vec![ChoiceOption::Object(bears)]);

    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");
    let refused = game.resolve_top_of_stack(&test_dp()).expect_err("destruction has no share to read");

    assert!(refused.contains("601.2d"), "{refused}");
    assert!(game.battlefield.contains_key(&bears));
}
