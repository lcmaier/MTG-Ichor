//! `divided-among-targets`: an effect divided among its targets (CR 601.2d),
//! announced on a cast, an activation and a trigger alike (602.2b, 603.3d), and
//! acted on by each target's share
//! (`choices-architecture.md#decision-5--where-a-division-lives`).

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use mtgsim::cards::authoring::{
    at_beginning_of, create_delayed_trigger, deal_damage, deal_damage_divided, destroy, distribute_counters, divided_among, draw_cards,
    prevent_the_next_damage_divided, referred, spell_ability, that_target, whenever, you, Whose,
};
use mtgsim::cards::fixtures::{
    dragonlord_atarka_fixture, elven_rite_fixture, magma_opus_fixture, meteor_swarm_fixture, mogg_mob_fixture, remedy_fixture, rolling_thunder_fixture,
};
use mtgsim::cards::printings::{arc_lightning, forest, grizzly_bears, hill_giant, lightning_bolt, savannah_lions};
use mtgsim::cards::registry::CardRegistry;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::x_values::LegalXValues;
use mtgsim::state::decision_log::{self, AnswerLine, BuiltStart, Chosen, GameStart};
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::test_support::{put_in_hand, put_on_battlefield, setup_game, test_dp, AtEachPrompt};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{AmountExpr, CounterType, Effect, EffectRecipient, SelectionFilter, TargetCount};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::triggers::{DelayedDuration, DelayedTriggerTemplate, DelayedTurn};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption, ReplacementOption};
use mtgsim::ui::decision::{DispatchDecisionProvider, PriorityAction, ScriptedDecisionProvider};
use mtgsim::ui::display::{format_event_log, option_label};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::replay::Replay;

/// The script's next answer: `chosen` for the announcement of an instance of
/// "target". The queue matches a kind by its variant alone.
fn names_targets(answers: &ScriptedDecisionProvider, chosen: Vec<ChoiceOption>) {
    answers.expect_choice(ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED }, chosen);
}

/// The script's next answer: `shares` for CR 601.2d's division.
fn divides(answers: &ScriptedDecisionProvider, shares: Vec<(ChoiceOption, u64)>) {
    answers.expect_allocation(ChoiceKind::DivideAmong { source: ObjectId::UNASSIGNED }, shares);
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
    names_targets(&answers, vec![ChoiceOption::Object(bears), ChoiceOption::Player(1)]);
    divides(&answers, vec![(ChoiceOption::Player(1), 2), (ChoiceOption::Object(bears), 1)]);
    let asked = RefCell::new(None);
    let seat = AtEachPrompt::new(answers, |game: &GameState, context: &ChoiceContext| {
        if let ChoiceKind::DivideAmong { source } = context.kind {
            *asked.borrow_mut() = Some((source, game.objects[&mob].zone));
        }
    });

    game.activate_ability(0, mob, 0, &seat).expect("sacrificing it pays");

    let ability = *game.stack.last().expect("the ability is on the stack");
    assert_eq!(asked.into_inner(), Some((ability, Zone::Battlefield)), "asked of the ability on the stack, before the sacrifice");
    assert_eq!(division(&game, ability), vec![(ObjectOrPlayerId::Object(bears), 1), (ObjectOrPlayerId::Player(1), 2)]);
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
    names_targets(&answers, vec![ChoiceOption::Object(bears), ChoiceOption::Object(lions)]);
    divides(&answers, vec![(ChoiceOption::Object(bears), 1), (ChoiceOption::Object(lions), 4)]);
    game.place_pending_triggers(&answers).expect("the trigger is placed");

    let trigger = *game.stack.last().expect("the trigger is on the stack");
    let asked_of = |kind: &ChoiceKind| matches!(kind, ChoiceKind::DivideAmong { source } if *source == trigger);
    assert!(answers.asked().iter().any(asked_of), "asked of the trigger on the stack");
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
    names_targets(&answers, vec![ChoiceOption::Object(bears), ChoiceOption::Player(1)]);
    names_targets(&answers, vec![ChoiceOption::Object(one), ChoiceOption::Object(other)]);
    divides(&answers, vec![(ChoiceOption::Object(bears), 1), (ChoiceOption::Player(1), 3)]);

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
    names_targets(&answers, vec![ChoiceOption::Object(bears), ChoiceOption::Object(lions)]);

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
    names_targets(&answers, vec![ChoiceOption::Object(bears)]);
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
    names_targets(&answers, vec![ChoiceOption::Object(bears), ChoiceOption::Player(1)]);
    divides(&answers, vec![(ChoiceOption::Object(bears), 3), (ChoiceOption::Player(1), 2)]);
    game.cast_spell(0, remedy, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).expect("the shields are made");

    let either = ChoiceOption::Replacement(ReplacementOption::FromResolution { source: remedy, resolution: remedy });
    let label = option_label(&game, &either);
    assert!(label.ends_with(&format!("divided as you choose, targeting Grizzly Bears ({bears}) and Player 1")), "{label}");

    for target in [ChoiceOption::Object(bears), ChoiceOption::Player(1)] {
        let bolt = put_in_hand(&mut game, lightning_bolt(), 0);
        game.players[0].mana_pool.add(ManaType::Red, 1);
        let answers = ScriptedDecisionProvider::new();
        names_targets(&answers, vec![target]);
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
    names_targets(&answers, vec![ChoiceOption::Object(bears), ChoiceOption::Player(1)]);
    divides(&answers, vec![(ChoiceOption::Object(bears), 1), (ChoiceOption::Player(1), 2)]);

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

/// CR 601.2c: X = 2 with one creature to choose has no legal announcement, so
/// the cast rewinds (CR 732.1) rather than asking for two out of one.
#[test]
fn a_count_the_board_cannot_meet_rewinds_the_cast() {
    let mut game = setup_game(2);
    let swarm = put_in_hand(&mut game, meteor_swarm_fixture(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.players[0].mana_pool.add(ManaType::Red, 3);
    let answers = ScriptedDecisionProvider::new();
    answers.expect_number(ChoiceKind::ChooseXValue { spell_id: swarm, values: Box::new(LegalXValues::zero_through(2)) }, 2);

    game.cast_spell(0, swarm, &ManaWindowStop::new(answers)).expect_err("two targets, one creature");

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
            Effect::Sequence(vec![deal_damage_divided(SelectionFilter::Any, TargetCount::Between(1, 2), 3), deal_damage(that_target(0), 1)]),
        ))
        .build();
    let spell = put_in_hand(&mut game, card, 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.players[0].mana_pool.add(ManaType::Red, 1);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Object(bears), ChoiceOption::Player(1)]);
    divides(&answers, vec![(ChoiceOption::Object(bears), 1), (ChoiceOption::Player(1), 2)]);

    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");

    assert_eq!(damage_on(&game, bears), 2, "its share of 1, then 1");
    assert_eq!(game.players[1].life_total, 17, "its share of 2, then 1");
}

/// `Effect::Remember` around a divided shield remembers each target once: each
/// shield's row is made by the instruction undivided, which notes its own.
#[test]
fn a_divided_shield_is_remembered_once_a_target() {
    let mut game = setup_game(2);
    let text = "Prevent the next 2 damage that would be dealt this turn to any number of targets, divided as you choose. Draw a card at the next end step for each.";
    let later = DelayedTriggerTemplate {
        def: Arc::new(whenever(at_beginning_of(StepType::End, Whose::Each), draw_cards(referred(), 1))),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::NoneNamed,
        rules_text: "Draw a card.".into(),
    };
    let shield = prevent_the_next_damage_divided(SelectionFilter::Any, TargetCount::AnyNumber, 2);
    let card: Arc<CardData> = CardDataBuilder::new("Remembered Shields")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, Effect::Sequence(vec![Effect::Remember(Box::new(shield)), create_delayed_trigger(you(), later)])))
        .build();
    let spell = put_in_hand(&mut game, card, 0);
    game.players[0].mana_pool.add(ManaType::White, 1);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Player(0), ChoiceOption::Player(1)]);

    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");

    assert_eq!(game.delayed_triggers[0].referred.players, vec![0, 1]);
}

/// A share of counters on a player has no event to go to, and is refused
/// rather than lost.
#[test]
fn a_share_of_counters_on_a_player_is_refused() {
    let mut game = setup_game(2);
    let text = "Distribute two +1/+1 counters among one or two targets.";
    let card: Arc<CardData> = CardDataBuilder::new("Counters At Anyone")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(text, distribute_counters(SelectionFilter::Any, TargetCount::Between(1, 2), CounterType::PlusOnePlusOne, 2)))
        .build();
    let spell = put_in_hand(&mut game, card, 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.players[0].mana_pool.add(ManaType::Green, 1);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Object(bears), ChoiceOption::Player(1)]);

    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");
    let refused = game.resolve_top_of_stack(&test_dp()).expect_err("a player takes no `AddCounters`");

    assert!(refused.contains("onto a player"), "{refused}");
}

/// A verb no text divides is refused at resolution, loudly, rather than acting
/// on each target as if it had the whole.
#[test]
fn a_verb_that_cannot_divide_is_refused() {
    let mut game = setup_game(2);
    let text = "Destroy one or two target creatures, divided as you choose.";
    let clause = divided_among(SelectionFilter::Creature, TargetCount::Between(1, 2), AmountExpr::Fixed(2));
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
    names_targets(&answers, vec![ChoiceOption::Object(bears)]);

    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");
    let refused = game.resolve_top_of_stack(&test_dp()).expect_err("destruction has no share to read");

    assert!(refused.contains("601.2d"), "{refused}");
    assert!(game.battlefield.contains_key(&bears));
}

/// Arc Lightning in player 0's hand with exactly its cost in the pool.
fn arc_lightning_in_hand(game: &mut GameState) -> ObjectId {
    game.players[0].mana_pool.add(ManaType::Red, 3);
    put_in_hand(game, arc_lightning(), 0)
}

/// A Lightning Bolt from player 0 at `creature`, resolved, and CR 704's check.
fn bolted(game: &mut GameState, creature: ObjectId) {
    let bolt = put_in_hand(game, lightning_bolt(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Object(creature)]);
    game.cast_spell(0, bolt, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).expect("the Bolt resolves");
    game.check_state_based_actions(&test_dp()).expect("checking state-based actions");
}

// RULING: Arc Lightning #1 - "You choose how many targets Arc Lightning has and
//   how the damage is divided as you cast the spell."
// COVERS: ATOM-601.2d-001
#[test]
fn arc_lightning_keeps_the_division_it_was_cast_with() {
    let mut game = setup_game(2);
    let giant = put_on_battlefield(&mut game, hill_giant(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let lions = put_on_battlefield(&mut game, savannah_lions(), 1);
    let arc = arc_lightning_in_hand(&mut game);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Object(giant), ChoiceOption::Object(bears)]);
    divides(&answers, vec![(ChoiceOption::Object(giant), 2), (ChoiceOption::Object(bears), 1)]);

    game.cast_spell(0, arc, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");

    assert_eq!(division(&game, arc), vec![(ObjectOrPlayerId::Object(giant), 2), (ObjectOrPlayerId::Object(bears), 1)]);
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");
    assert_eq!([damage_on(&game, giant), damage_on(&game, bears), damage_on(&game, lions)], [2, 1, 0]);
}

/// Two chosen of three on the board, and nothing left for the second: the one
/// way a real prompt can be answered with a target left out, since all three
/// chosen is a split [`arc_lightning_asks_nothing_of_one_split`] never asks.
// RULING: Arc Lightning #1 - "Each target must receive at least 1 damage."
// COVERS-PARTIAL: ATOM-601.2d-002
#[test]
#[should_panic(expected = "allocated 0 to bucket 1 but minimum is 1")]
fn arc_lightning_refuses_a_target_with_no_share() {
    let mut game = setup_game(2);
    let giant = put_on_battlefield(&mut game, hill_giant(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    put_on_battlefield(&mut game, savannah_lions(), 1);
    let arc = arc_lightning_in_hand(&mut game);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Object(giant), ChoiceOption::Object(bears)]);
    divides(&answers, vec![(ChoiceOption::Object(giant), 3), (ChoiceOption::Object(bears), 0)]);

    let _ = game.cast_spell(0, arc, &ManaWindowStop::new(answers));
}

/// One target takes the whole, and three take one each: a split with one
/// answer is not asked.
#[test]
fn arc_lightning_asks_nothing_of_one_split() {
    let mut game = setup_game(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let lions = put_on_battlefield(&mut game, savannah_lions(), 1);
    let arc = arc_lightning_in_hand(&mut game);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Object(bears), ChoiceOption::Object(lions), ChoiceOption::Player(1)]);
    let seat = ManaWindowStop::new(answers);
    game.cast_spell(0, arc, &seat).expect("castable from exactly its cost");
    assert_eq!(the_asked(seat.inner()), ["SelectRecipients"]);
    assert_eq!(division(&game, arc).iter().map(|(_, share)| *share).collect::<Vec<_>>(), [1, 1, 1]);
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");

    let again = arc_lightning_in_hand(&mut game);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Player(1)]);
    let seat = ManaWindowStop::new(answers);
    game.cast_spell(0, again, &seat).expect("castable from exactly its cost");
    assert_eq!(the_asked(seat.inner()), ["SelectRecipients"]);
    assert_eq!(division(&game, again), vec![(ObjectOrPlayerId::Player(1), 3)]);
}

// RULING: Arc Lightning #2 - "If some of the creatures are illegal targets as
//   Arc Lightning tries to resolve, the original division of damage still
//   applies but no damage is dealt to the illegal targets."
#[test]
fn an_illegal_target_takes_no_share_and_the_rest_stand() {
    let mut game = setup_game(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let arc = arc_lightning_in_hand(&mut game);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Object(bears), ChoiceOption::Player(1)]);
    divides(&answers, vec![(ChoiceOption::Object(bears), 2), (ChoiceOption::Player(1), 1)]);
    game.cast_spell(0, arc, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");

    bolted(&mut game, bears);
    assert_eq!(game.objects[&bears].zone, Zone::Graveyard, "a target no longer in its zone is illegal");
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");

    assert_eq!(game.players[1].life_total, 19, "the player's 1, and nothing of the Bears' 2");
}

// RULING: Arc Lightning #2 - "If all targets are illegal, Arc Lightning doesn't
//   resolve."
#[test]
fn arc_lightning_with_every_target_illegal_does_not_resolve() {
    let mut game = setup_game(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let lions = put_on_battlefield(&mut game, savannah_lions(), 1);
    let arc = arc_lightning_in_hand(&mut game);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Object(bears), ChoiceOption::Object(lions)]);
    divides(&answers, vec![(ChoiceOption::Object(bears), 2), (ChoiceOption::Object(lions), 1)]);
    game.cast_spell(0, arc, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");

    bolted(&mut game, bears);
    bolted(&mut game, lions);
    game.resolve_top_of_stack(&test_dp()).expect("the rule removes it");

    assert_eq!(game.objects[&arc].zone, Zone::Graveyard);
    assert!(game.stack.is_empty());
}

/// CR 800.4a at four seats: a player who left is CR 608.2b's illegal target,
/// and the seat still in the game takes the share announced for it.
#[test]
fn a_seat_that_left_takes_no_share() {
    let mut game = setup_game(4);
    let arc = arc_lightning_in_hand(&mut game);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Player(2), ChoiceOption::Player(3)]);
    divides(&answers, vec![(ChoiceOption::Player(2), 1), (ChoiceOption::Player(3), 2)]);
    game.cast_spell(0, arc, &ManaWindowStop::new(answers)).expect("castable from exactly its cost");

    game.players[3].has_drawn_from_empty_library = true;
    game.check_state_based_actions(&test_dp()).expect("checking state-based actions");
    assert!(!game.in_game(3), "CR 704.5b, then CR 800.4a");
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");

    assert_eq!(game.players[2].life_total, 19);
    assert_eq!(game.players[3].life_total, 20);
}

const ARC_LIGHTNING_AT_THE_BEARS: &str = "\
hand 0: Arc Lightning
battlefield: Mountain | controller 0
battlefield: Mountain [second] | controller 0
battlefield: Mountain [third] | controller 0
battlefield: Grizzly Bears | controller 1
library 0: Mountain | x5
library 1: Mountain | x5
";

/// The one card named `name` on this board.
fn the_one(game: &GameState, name: &str) -> ObjectId {
    // ORDER-FREE: one card of this name on this board.
    let found: Vec<ObjectId> = game.objects.values().filter(|object| object.card_data.name == name).map(|object| object.id).collect();
    let [id] = found[..] else { panic!("{} cards named {name}", found.len()) };
    id
}

/// Arc Lightning cast from hand through a game's own priority, 1 at the Bears
/// and 2 at the opponent, writing its log as a client attaches the writer.
fn played() -> (BuiltStart, decision_log::Log) {
    let start = GameStart::Scenario { path: "arc".to_string(), seed: 0, text: ARC_LIGHTNING_AT_THE_BEARS.to_string() };
    let mut built = start.build(&CardRegistry::default_registry()).expect("the board builds");
    let record = Arc::new(Mutex::new(decision_log::opening(&start).iter().map(|line| format!("{line}\n")).collect::<String>()));
    let (writer, answered) = (Arc::clone(&record), Arc::new(Mutex::new(0)));
    let game = built.game_mut();
    game.state.record_events();
    game.state.log_decisions(move |game, decision| {
        let mut number = answered.lock().unwrap();
        *number += 1;
        writer.lock().unwrap().push_str(&format!("{}\n", AnswerLine::of(game, *number, decision)));
    });

    let state = &built.game().state;
    let (arc, bears) = (the_one(state, "Arc Lightning"), the_one(state, "Grizzly Bears"));
    // ORDER-FREE: the three Mountains are alike; any pays.
    let mountains: Vec<ObjectId> = state.battlefield.keys().filter(|id| state.objects[id].card_data.name == "Mountain").collect();
    let tap = |id: ObjectId| ChoiceOption::Action(PriorityAction::ActivateAbility(id, state.objects[&id].card_data.abilities[0].id));

    let caster = ScriptedDecisionProvider::new();
    caster.expect_choice(ChoiceKind::PriorityAction, vec![ChoiceOption::Action(PriorityAction::CastSpell(arc))]);
    names_targets(&caster, vec![ChoiceOption::Object(bears), ChoiceOption::Player(1)]);
    divides(&caster, vec![(ChoiceOption::Object(bears), 1), (ChoiceOption::Player(1), 2)]);
    let window = ChoiceKind::ManaAbilityWindow { spell_or_ability_id: arc, remaining_cost: ManaCost::build(&[], 0) };
    for mountain in mountains {
        caster.expect_choice(window.clone(), vec![tap(mountain)]);
    }
    let seats = DispatchDecisionProvider::new(vec![Box::new(ManaWindowStop::new(caster)), Box::new(ScriptedDecisionProvider::new())]);
    built.game_mut().until_stopped(|game| game.resume_turn_at_priority(&seats)).expect("the turn plays");

    let log = decision_log::read(&record.lock().unwrap()).expect("the log reads");
    (built, log)
}

/// The log writes each target with its share, and a replay plays the same game
/// from those words.
#[test]
fn a_logged_division_replays_to_the_same_game() {
    let (original, log) = played();
    let state = &original.game().state;
    let bears = the_one(state, "Grizzly Bears");
    assert_eq!(state.players[1].life_total, 18, "the opponent's share");

    let shares = log
        .answers
        .iter()
        .find_map(|line| match &line.chosen {
            Chosen::Allocation(amounts) if line.kind == "DivideAmong" => Some(amounts.clone()),
            _ => None,
        })
        .expect("the division is a line of the log");
    assert_eq!(shares, vec![(1, format!("Grizzly Bears ({bears})")), (2, "player 1".to_string())]);

    let mut replayed = log.start.build(&CardRegistry::default_registry()).expect("the log's start builds");
    replayed.game_mut().state.record_events();
    let replay = Replay::of(log);
    replayed.game_mut().until_stopped(|game| game.resume_turn_at_priority(&replay)).expect("the replay plays");

    assert_eq!(replay.remaining(), 0, "every answer the log holds was asked for");
    assert_eq!(format_event_log(&replayed.game().state), format_event_log(&original.game().state));
    assert_eq!(replayed.game().state.players[1].life_total, 18);
}
