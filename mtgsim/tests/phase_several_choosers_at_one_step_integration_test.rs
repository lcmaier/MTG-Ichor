//! `several-choosers-at-one-step` (`choices-architecture.md`), the second of
//! `values-chosen-at-resolution`'s three: a choice each player makes, asked
//! around the table from the active player, each answer kept with its
//! player, and what the steps after it read of them.

use std::cell::RefCell;
use std::sync::Arc;

#[path = "support/watching.rs"]
mod watching;
use watching::Watching;

use mtgsim::autopilot::mana_window_stop::ManaWindowStop;
use mtgsim::cards::authoring::{
    add_mana, choose_a_second_player, choose_a_third_player, choose_color, choose_creature_type, choose_number, choose_number_at_least,
    choose_objects, choose_player, chosen_by, color, creature, creature_type, destroy, draw_cards, each, each_opponent, each_player,
    gain_life, lose_life, non, one_of_the_chosen, sacrifice, spell_ability, target_creature, target_opponent, the_chosen_color,
    the_chosen_creature_type, the_chosen_number, the_chosen_objects, the_chosen_player, the_second_chosen_player,
    the_third_chosen_player, they_control, you, you_control,
};
use mtgsim::cards::printings::{choice_of_damnations, fortunate_few, harsh_mercy, plague_spores, slaughter_the_strong};
use mtgsim::decision::provider::ScriptedDecisionProvider;
use mtgsim::decision::question::{DecisionContext, DecisionKind, DecisionOption, Rejection, TotalLimit};
use mtgsim::engine::actions::GameAction;
use mtgsim::events::event::{BatchId, EventRecord, GameEvent};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::x_values::XOffer;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{fill_library, forest, pass_turn, put_in_hand, put_on_battlefield, setup_game, test_ctx, AtEachPrompt};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    Choice, ChoiceKind, ChoiceScope, ChoiceSide, Condition, CostAnswer, CounterType, Effect, EffectRecipient, ManaOutput, ObjectValue, Pick,
    PickTotal, PlayerFact, PlayerRef, PlayerSet, Primitive, SelectionFilter, TargetCount, ValueChoice, ValueIs,
};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::zones::ZoneChangeAct;
use mtgsim::words::display::rejection;

// ---------------------------------------------------------------------------
// The boards' tools
// ---------------------------------------------------------------------------

/// A one-mana instant whose text is `text` and whose effect is `effect`: an
/// invented spell for a rule's board, cast at any player's turn.
fn instant(name: &str, text: &'static str, effect: Effect) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, effect))
        .build()
}

/// `card` in player 0's hand, cast from a pool of exactly `pool`.
fn cast_paying(game: &mut GameState, card: Arc<CardData>, pool: &[(ManaType, u64)]) -> ObjectId {
    let id = put_in_hand(game, card, 0);
    for (mana, amount) in pool {
        game.players[0].mana_pool.add(*mana, *amount);
    }
    let seat = ManaWindowStop::new(ScriptedDecisionProvider::new());
    game.cast_spell(0, id, &seat).expect("castable from exactly its cost");
    id
}

/// A game of `seats` players, each with cards to draw, at `active`'s turn.
fn at_the_turn_of(active: PlayerId, seats: usize) -> GameState {
    let mut game = setup_game(seats);
    for seat in 0..seats {
        fill_library(&mut game, seat, 20);
    }
    for _ in 0..active {
        pass_turn(&mut game);
    }
    assert_eq!(game.active_player, active, "the board's turn");
    game
}

/// A 2/2 creature of one color.
fn creature_of_color(name: &str, color: Color) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .color(color)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Bear))
        .power_toughness(2, 2)
        .build()
}

/// The script's next answer to "choose a color": `color`.
fn chooses_color(answers: &ScriptedDecisionProvider, color: Color) {
    answers.expect_choice(DecisionKind::ChooseColor { source: ObjectId::UNASSIGNED }, vec![DecisionOption::Color(color)]);
}

/// The script's next answer to a choice of objects: `chosen`.
fn picks(answers: &ScriptedDecisionProvider, chosen: Vec<DecisionOption>) {
    answers.expect_choice(DecisionKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED }, chosen);
}

/// Who each prompt of `answers` asked, in the order asked.
fn asked_in_order(answers: &ScriptedDecisionProvider) -> Vec<PlayerId> {
    answers.asked_of().into_iter().map(|(player, _)| player).collect()
}

/// "Each player chooses a color. Destroy each creature of a color chosen
/// this way."
fn each_player_condemns_a_color() -> Arc<CardData> {
    instant(
        "A Color Apiece",
        "Each player chooses a color. Destroy each creature of a color chosen this way.",
        Effect::Sequence(vec![choose_color(each_player()), destroy(each(creature().and(color(the_chosen_color()))))]),
    )
}

// ---------------------------------------------------------------------------
// The walk: who is asked, and in what order (CR 101.4, 608.2e)
// ---------------------------------------------------------------------------

/// CR 101.4 — "the active player … makes any choices required, then the next
/// player in turn order … followed by the remaining nonactive players in
/// turn order." The spell is player 0's and it is player 2's turn, so the
/// walk starts with player 2 and not with its controller. Every answer is
/// read: a creature of any chosen color is destroyed, once.
#[test]
fn each_player_is_asked_from_the_active_player_in_turn_order() {
    let mut game = at_the_turn_of(2, 4);
    let [white, blue, black, red, green] = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]
        .map(|color| put_on_battlefield(&mut game, creature_of_color("A Bear", color), 1));
    cast_paying(&mut game, each_player_condemns_a_color(), &[(ManaType::Blue, 1)]);

    let answers = ScriptedDecisionProvider::new();
    for color in [Color::Red, Color::Green, Color::Red, Color::Blue] {
        chooses_color(&answers, color);
    }
    game.resolve_top_of_stack(&answers).expect("it resolves");

    assert_eq!(asked_in_order(&answers), [2, 3, 0, 1], "the active player first, then turn order");
    let on_the_battlefield = [white, blue, black, red, green].map(|id| game.battlefield.contains_key(&id));
    assert_eq!(on_the_battlefield, [true, false, true, false, false], "red, green and blue were chosen; white and black were not");
}

/// "Each opponent chooses": the group without its controller, begun with
/// the first of them at or after the active player.
#[test]
fn each_opponent_is_asked_and_the_controller_is_not() {
    let mut game = at_the_turn_of(0, 3);
    let red = put_on_battlefield(&mut game, creature_of_color("A Red Bear", Color::Red), 0);
    let green = put_on_battlefield(&mut game, creature_of_color("A Green Bear", Color::Green), 0);
    let card = instant(
        "Their Colors",
        "Each opponent chooses a color. Destroy each creature of a color chosen this way.",
        Effect::Sequence(vec![choose_color(each_opponent()), destroy(each(creature().and(color(the_chosen_color()))))]),
    );
    cast_paying(&mut game, card, &[(ManaType::Blue, 1)]);

    let answers = ScriptedDecisionProvider::new();
    chooses_color(&answers, Color::Red);
    chooses_color(&answers, Color::Red);
    game.resolve_top_of_stack(&answers).expect("it resolves");

    assert_eq!(asked_in_order(&answers), [1, 2], "its controller, the active player, is no opponent");
    assert!(!game.battlefield.contains_key(&red), "both chose red");
    assert!(game.battlefield.contains_key(&green), "nobody chose green");
}

/// CR 104.3a, 800.4a — a player who has left the game is no longer "each
/// player", and is not asked.
#[test]
fn a_player_who_has_left_is_not_asked() {
    let mut game = at_the_turn_of(0, 3);
    let red = put_on_battlefield(&mut game, creature_of_color("A Red Bear", Color::Red), 0);
    cast_paying(&mut game, each_player_condemns_a_color(), &[(ManaType::Blue, 1)]);
    game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).expect("player 1 leaves");

    let answers = ScriptedDecisionProvider::new();
    chooses_color(&answers, Color::Green);
    chooses_color(&answers, Color::Red);
    game.resolve_top_of_stack(&answers).expect("it resolves");

    assert_eq!(asked_in_order(&answers), [0, 2]);
    assert!(!game.battlefield.contains_key(&red), "player 2's answer is read");
}

// ---------------------------------------------------------------------------
// What the walk keeps, and what reads it
// ---------------------------------------------------------------------------

/// CR 101.4b — "a player knows the choices made by the previous players
/// when making their choice." The walk's choice is opened with no answers
/// before its first question and each answer is added as it is given, so a
/// filter naming "chosen this way" offers the first player every creature
/// and each later player only those not yet chosen.
#[test]
fn a_later_players_question_reads_the_answers_before_it() {
    let mut game = at_the_turn_of(0, 3);
    let bears = [0, 1, 2].map(|seat| put_on_battlefield(&mut game, creature_of_color("A Bear", Color::Green), seat));
    let spare = put_on_battlefield(&mut game, creature_of_color("A Spare Bear", Color::Green), 0);
    let card = instant(
        "One Each",
        "Each player chooses a creature that hasn't been chosen this way. Destroy the rest.",
        Effect::Sequence(vec![
            choose_objects(each_player(), creature().and(non(one_of_the_chosen())), TargetCount::Exactly(1)),
            destroy(each(creature().and(non(one_of_the_chosen())))),
        ]),
    );
    cast_paying(&mut game, card, &[(ManaType::Blue, 1)]);

    let answers = ScriptedDecisionProvider::new();
    picks(&answers, vec![DecisionOption::Object(bears[1])]);
    picks(&answers, vec![DecisionOption::Object(bears[2])]);
    picks(&answers, vec![DecisionOption::Object(bears[0])]);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let offered: Vec<usize> = asked.of_options.borrow().iter().map(|(_, _, options)| options.len()).collect();
    assert_eq!(offered, [4, 3, 2], "every creature, then one fewer for each answer given");
    assert_eq!(bears.map(|id| game.battlefield.contains_key(&id)), [true, true, true], "each chosen creature is spared");
    assert!(!game.battlefield.contains_key(&spare), "the rest is what no answer chose");
}

// ---------------------------------------------------------------------------
// "They": the player an answer is for (CR 608.2d)
// ---------------------------------------------------------------------------

// RULING: Divine Reckoning #7 - "Starting with the player whose turn it is,
//   each player chooses a creature in turn order. Players will know the
//   choice of each player who chose before them."
// COVERS-PARTIAL: ATOM-608.2e-001
//
/// "Each player chooses a creature they control": "they" is the player the
/// walk is on, so each is offered their own creatures and no one else's,
/// where "you control" would offer every player the controller's. A player
/// with none to choose gives an answer that chose nothing, and the rest is
/// still read against every answer, and destroyed in one batch once every
/// choice is made (CR 608.2e). Divine Reckoning's first two sentences, on
/// an instant so that the turn is not its controller's.
#[test]
fn each_player_chooses_among_what_they_control() {
    let mut game = at_the_turn_of(1, 3);
    game.record_events();
    let mine = [(); 2].map(|_| put_on_battlefield(&mut game, creature_of_color("My Bear", Color::Green), 0));
    let theirs = [(); 2].map(|_| put_on_battlefield(&mut game, creature_of_color("Their Bear", Color::Green), 1));
    let card = instant(
        "One Apiece",
        "Each player chooses a creature they control. Destroy the rest.",
        Effect::Sequence(vec![
            choose_objects(each_player(), creature().and(they_control()), TargetCount::Exactly(1)),
            destroy(each(creature().and(non(one_of_the_chosen())))),
        ]),
    );
    cast_paying(&mut game, card, &[(ManaType::Blue, 1)]);

    let answers = ScriptedDecisionProvider::new();
    picks(&answers, vec![DecisionOption::Object(theirs[1])]);
    picks(&answers, vec![DecisionOption::Object(mine[0])]);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let asked = asked.of_options.borrow();
    let offered: Vec<(PlayerId, Vec<DecisionOption>)> = asked.iter().map(|(player, _, options)| (*player, options.clone())).collect();
    assert_eq!(
        offered,
        [(1, theirs.map(DecisionOption::Object).to_vec()), (0, mine.map(DecisionOption::Object).to_vec())],
        "the active player first, each among their own; player 2 controls none and is not asked"
    );
    assert_eq!(mine.map(|id| game.battlefield.contains_key(&id)), [true, false]);
    assert_eq!(theirs.map(|id| game.battlefield.contains_key(&id)), [false, true]);
    let destroyed = moved_by(&game, ZoneChangeAct::Destroyed);
    assert_eq!(destroyed.len(), 2);
    assert_eq!(destroyed[0].1, destroyed[1].1, "one batch, after the last choice");
}

// COVERS: ATOM-101.4-001
/// CR 101.4 — "each player sacrifices a creature": the active player
/// chooses first, then the other, and both sacrifices are one event. Each
/// controls two creatures, since one apiece is no choice and asks nobody.
#[test]
fn each_player_sacrifices_a_creature_the_active_player_choosing_first() {
    let mut game = at_the_turn_of(1, 2);
    game.record_events();
    let mine = [(); 2].map(|_| put_on_battlefield(&mut game, creature_of_color("My Bear", Color::Green), 0));
    let theirs = [(); 2].map(|_| put_on_battlefield(&mut game, creature_of_color("Their Bear", Color::Green), 1));
    let a_creature_apiece =
        Choice { chooser: each_player(), among: ChoiceScope::ChoosersPermanents, picks: vec![Pick::exactly(1, creature())], acts_on: ChoiceSide::Chosen };
    let card = instant("An Edict", "Each player sacrifices a creature of their choice.", sacrifice(chosen_by(a_creature_apiece)));
    cast_paying(&mut game, card, &[(ManaType::Blue, 1)]);

    let answers = ScriptedDecisionProvider::new();
    picks(&answers, vec![DecisionOption::Object(theirs[0])]);
    picks(&answers, vec![DecisionOption::Object(mine[1])]);
    game.resolve_top_of_stack(&answers).expect("it resolves");

    assert_eq!(asked_in_order(&answers), [1, 0], "the active player, then the next in turn order");
    let gone = sacrificed(&game);
    assert_eq!(gone.iter().map(|(id, _)| *id).collect::<Vec<_>>(), [theirs[0], mine[1]]);
    assert_eq!(gone[0].1, gone[1].1, "one batch");
}

/// With one chooser and no walk, "they" is that chooser: "target opponent
/// chooses a creature they control. Destroy it."
#[test]
fn one_choosers_they_is_the_chooser() {
    let mut game = at_the_turn_of(0, 3);
    let mine = put_on_battlefield(&mut game, creature_of_color("My Bear", Color::Green), 0);
    let theirs = put_on_battlefield(&mut game, creature_of_color("Their Bear", Color::Green), 1);
    let card = instant(
        "Their Pick",
        "Target opponent chooses a creature they control. Destroy it.",
        Effect::Sequence(vec![
            choose_objects(target_opponent(), creature().and(they_control()), TargetCount::Exactly(1)),
            destroy(the_chosen_objects()),
        ]),
    );
    let id = put_in_hand(&mut game, card, 0);
    game.players[0].mana_pool.add(ManaType::Blue, 1);
    let targeting = ScriptedDecisionProvider::new();
    picks(&targeting, vec![DecisionOption::Player(1)]);
    game.cast_spell(0, id, &ManaWindowStop::new(targeting)).expect("castable");

    // One creature to choose among, so nothing is asked.
    game.resolve_top_of_stack(&ScriptedDecisionProvider::new()).expect("it resolves");
    assert!(game.battlefield.contains_key(&mine), "the controller's creature was never among the chooser's");
    assert!(!game.battlefield.contains_key(&theirs));
}

/// "They" is written into a filter only where a choose step chooses. A
/// sweep that names it has no player to mean, and is refused as it
/// resolves.
#[test]
fn they_outside_a_choice_is_refused() {
    let mut game = at_the_turn_of(0, 2);
    put_on_battlefield(&mut game, creature_of_color("A Bear", Color::Green), 1);
    let card = instant("Whose", "Destroy each creature they control.", destroy(each(creature().and(they_control()))));
    cast_paying(&mut game, card, &[(ManaType::Blue, 1)]);
    let refused = game.resolve_top_of_stack(&ScriptedDecisionProvider::new()).expect_err("no choice is being made");
    assert!(refused.contains("the player a choice is for"), "{refused}");
}

// ---------------------------------------------------------------------------
// "Choose a number", with no most printed (CR 107.1b)
// ---------------------------------------------------------------------------

/// CR 107.1b — "you can't choose a negative number", and nothing bounds a
/// number above. The question says it has no most, is asked from its least
/// up to the largest number there is, and takes a number past any a board
/// would suggest.
#[test]
fn a_number_with_no_most_is_asked_from_its_least_with_no_top() {
    let mut game = at_the_turn_of(0, 2);
    let card = instant(
        "Any Number",
        "Choose a number greater than 0. You gain that much life.",
        Effect::Sequence(vec![choose_number_at_least(you(), 1), gain_life(you(), the_chosen_number())]),
    );
    cast_paying(&mut game, card, &[(ManaType::Blue, 1)]);
    let life = game.players[0].life_total;

    let answers = ScriptedDecisionProvider::new();
    let no_top = XOffer::NoTop { at_least: 1 };
    answers.expect_number(DecisionKind::ChooseNumber { source: ObjectId::UNASSIGNED, values: Box::new(no_top.clone()) }, 37);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let asked = asked.of_numbers.borrow();
    let [(0, DecisionKind::ChooseNumber { values, .. }, 1, u64::MAX)] = asked.as_slice() else { panic!("one number, from 1 up: {asked:?}") };
    assert_eq!(**values, no_top, "the question says there is no most");
    assert_eq!(game.players[0].life_total, life + 37);
}

// ---------------------------------------------------------------------------
// A step that reads a choice never made (CR 608.2b)
// ---------------------------------------------------------------------------

/// `card`, an instant whose first target is an opponent who chooses and
/// whose second is a creature, cast at player 1 and the board's one
/// creature; player 1 then leaves the game, so the choice is never made
/// while the second target keeps the spell resolving. Whether it resolved.
fn resolve_after_its_chooser_left(game: &mut GameState, card: Arc<CardData>) -> Result<(), String> {
    cast_at_player_1_who_then_leaves(game, card);
    game.resolve_top_of_stack(&ScriptedDecisionProvider::new()).map(|_| ())
}

/// [`resolve_after_its_chooser_left`] up to the resolution: the spell on
/// the stack, its choosing player gone.
fn cast_at_player_1_who_then_leaves(game: &mut GameState, card: Arc<CardData>) {
    let id = put_in_hand(game, card, 0);
    game.players[0].mana_pool.add(ManaType::Blue, 1);
    let targeting = ScriptedDecisionProvider::new();
    picks(&targeting, vec![DecisionOption::Player(1)]);
    game.cast_spell(0, id, &ManaWindowStop::new(targeting)).expect("castable");
    game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).expect("player 1 leaves");
}

/// CR 608.2b — "if part of the effect requires information about an illegal
/// target, it fails to determine any such information. Any part of the
/// effect that requires that information won't happen." The number's
/// chooser is a target who has left, so no number is chosen; "you gain that
/// much life" reads it and does not happen; and the creature, a target
/// still legal, is destroyed all the same.
#[test]
fn an_amount_that_reads_a_choice_never_made_does_not_happen() {
    let mut game = at_the_turn_of(0, 3);
    let bear = put_on_battlefield(&mut game, creature_of_color("A Bear", Color::Green), 2);
    let life = game.players[0].life_total;
    let card = instant(
        "Their Number",
        "Target opponent chooses a number between 1 and 3. You gain that much life. Destroy target creature.",
        Effect::Sequence(vec![choose_number(target_opponent(), 1, 3), gain_life(you(), the_chosen_number()), destroy(target_creature())]),
    );
    resolve_after_its_chooser_left(&mut game, card).expect("the part that reads the number does not happen, and the rest does");
    assert_eq!(game.players[0].life_total, life, "no number, so no life");
    assert!(!game.battlefield.contains_key(&bear), "the other target is still affected");
}

/// The same for a player word: "that player may draw a card", where no
/// player was chosen, is a "may" nobody is asked.
#[test]
fn a_may_whose_player_was_never_chosen_does_not_happen() {
    let mut game = at_the_turn_of(0, 3);
    let bear = put_on_battlefield(&mut game, creature_of_color("A Bear", Color::Green), 2);
    let card = instant(
        "Their Friend",
        "Target opponent chooses a player. That player may draw a card. Destroy target creature.",
        Effect::Sequence(vec![
            choose_player(target_opponent(), PlayerSet::Everyone),
            Effect::Optional {
                chooser: PlayerRef::NamedBy(Box::new(the_chosen_player())),
                effect: Box::new(draw_cards(the_chosen_player(), 1)),
            },
            destroy(target_creature()),
        ]),
    );
    resolve_after_its_chooser_left(&mut game, card).expect("nobody is asked, and the rest happens");
    assert!(!game.battlefield.contains_key(&bear));
}

/// A choose step reads too: "choose that many creatures you control" needs
/// the number before it can ask. With no number it chooses nothing and
/// keeps nothing, so "destroy the rest" does not happen either, and the
/// caster's creature stays.
#[test]
fn a_choose_step_whose_count_was_never_chosen_does_not_happen() {
    let mut game = at_the_turn_of(0, 3);
    let bear = put_on_battlefield(&mut game, creature_of_color("A Bear", Color::Green), 2);
    let my_creatures = || creature().and(you_control());
    let card = instant(
        "Keep Their Number",
        "Target opponent chooses a number between 1 and 3. Choose that many creatures you control and destroy the rest. Destroy target creature.",
        Effect::Sequence(vec![
            choose_number(target_opponent(), 1, 3),
            choose_objects(you(), my_creatures(), TargetCount::ExactlyAmount(the_chosen_number())),
            destroy(each(my_creatures().other_than(one_of_the_chosen()))),
            destroy(target_creature()),
        ]),
    );
    cast_at_player_1_who_then_leaves(&mut game, card);
    // Put there once the spell has its one creature to target.
    let mine = put_on_battlefield(&mut game, creature_of_color("My Bear", Color::Green), 0);
    game.resolve_top_of_stack(&ScriptedDecisionProvider::new()).expect("the two parts that need the number do not happen");
    assert!(game.battlefield.contains_key(&mine), "nothing was chosen, so there is no rest");
    assert!(!game.battlefield.contains_key(&bear), "the other target is still affected");
}

/// An amount inside produced mana is read like any other: "add that much
/// {R}" with no number adds none.
#[test]
fn mana_of_an_amount_never_chosen_is_not_added() {
    let mut game = at_the_turn_of(0, 3);
    let bear = put_on_battlefield(&mut game, creature_of_color("A Bear", Color::Green), 2);
    let that_much_red = ManaOutput { mana: vec![(ManaType::Red, the_chosen_number())], special: vec![] };
    let card = instant(
        "Their Fire",
        "Target opponent chooses a number between 1 and 3. Add that much {R}. Destroy target creature.",
        Effect::Sequence(vec![choose_number(target_opponent(), 1, 3), add_mana(that_much_red), destroy(target_creature())]),
    );
    resolve_after_its_chooser_left(&mut game, card).expect("no number, so no mana");
    assert_eq!(game.players[0].mana_pool.total(), 0);
    assert!(!game.battlefield.contains_key(&bear));
}

/// CR 608.2c — an "if" that cannot be read is an "if" that is false, and a
/// false one leaves the answer before it alone. The caster declines the
/// "may"; the "if" on a creature type nobody chose is passed over; and "if
/// you didn't" still reads the decline, so the life is lost.
#[test]
fn an_if_passed_over_leaves_the_may_before_it_answered() {
    let mut game = at_the_turn_of(0, 3);
    put_on_battlefield(&mut game, creature_of_color("A Bear", Color::Green), 2);
    let (hand, life) = (game.players[0].hand.len(), game.players[0].life_total);
    let a_creature_of_that_type = PlayerFact::ControlsPermanent(creature().and(creature_type(the_chosen_creature_type())));
    let card = instant(
        "Their Type",
        "Target opponent chooses a creature type. You may draw a card. If you control a creature of the chosen type, you gain 1 life. \
         If you didn't draw, you lose 2 life. Destroy target creature.",
        Effect::Sequence(vec![
            choose_creature_type(target_opponent()),
            Effect::Optional { chooser: PlayerRef::You, effect: Box::new(draw_cards(you(), 1)) },
            Effect::Conditional(Condition::Player { players: PlayerSet::You, fact: a_creature_of_that_type }, Box::new(gain_life(you(), 1))),
            Effect::Conditional(Condition::CostAnswer(CostAnswer::Doesnt), Box::new(lose_life(you(), 2))),
            destroy(target_creature()),
        ]),
    );
    cast_at_player_1_who_then_leaves(&mut game, card);
    let declines = ScriptedDecisionProvider::new();
    declines.expect_choice(DecisionKind::ApplyOptionalEffect { source: ObjectId::UNASSIGNED }, vec![]);
    game.resolve_top_of_stack(&declines).expect("it resolves");

    assert_eq!(game.players[0].hand.len(), hand, "the draw was declined");
    assert_eq!(game.players[0].life_total, life - 2, "the decline is still the answer after the passed-over if");
}

/// A "may" on one instruction that won't happen is no choice to offer:
/// "you may draw that many cards" with no number asks nobody, and "if you
/// do" after it reads that it was not done.
#[test]
fn a_may_on_an_instruction_that_wont_happen_is_not_offered() {
    let mut game = at_the_turn_of(0, 3);
    let bear = put_on_battlefield(&mut game, creature_of_color("A Bear", Color::Green), 2);
    let (hand, life) = (game.players[0].hand.len(), game.players[0].life_total);
    let card = instant(
        "Their Cards",
        "Target opponent chooses a number between 1 and 3. You may draw that many cards. If you do, you gain 1 life. Destroy target creature.",
        Effect::Sequence(vec![
            choose_number(target_opponent(), 1, 3),
            Effect::Optional { chooser: PlayerRef::You, effect: Box::new(draw_cards(you(), the_chosen_number())) },
            Effect::Conditional(Condition::CostAnswer(CostAnswer::Does), Box::new(gain_life(you(), 1))),
            destroy(target_creature()),
        ]),
    );
    // The provider holds no answer, so a question asked would fail here.
    resolve_after_its_chooser_left(&mut game, card).expect("nobody is asked");
    assert_eq!((game.players[0].hand.len(), game.players[0].life_total), (hand, life), "not offered, so not done");
    assert!(!game.battlefield.contains_key(&bear));
}

// ---------------------------------------------------------------------------
// "It can't be regenerated" (CR 608.2c, 701.19c)
// ---------------------------------------------------------------------------

/// A board with player 1's Bear and Forest, the Bear regenerated this turn
/// (CR 701.19a: a shield for the next time it would be destroyed).
fn a_regenerated_bear_and_a_forest(game: &mut GameState) -> (ObjectId, ObjectId) {
    let bear = put_on_battlefield(game, creature_of_color("A Bear", Color::Green), 1);
    let forest = put_on_battlefield(game, forest(), 1);
    let shield = instant("A Shield", "Regenerate target creature.", Effect::Atom(Primitive::Regenerate, target_creature()));
    cast_paying(game, shield, &[(ManaType::Blue, 1)]);
    game.resolve_top_of_stack(&ScriptedDecisionProvider::new()).expect("the shield is made");
    (bear, forest)
}

/// Plague Spores, cast by player 0 at the board's one creature and one
/// land, and resolved.
fn resolve_plague_spores(game: &mut GameState) {
    cast_paying(game, plague_spores(), &[(ManaType::Black, 5), (ManaType::Red, 1)]);
    game.resolve_top_of_stack(&ScriptedDecisionProvider::new()).expect("it resolves");
}

/// CR 701.19c — "effects that say that a permanent can't be regenerated …
/// cause regeneration shields to not be applied." Plague Spores' "They
/// can't be regenerated" is on its destroy, so the Bear's shield is passed
/// over and it dies.
#[test]
fn a_destroy_that_cant_be_regenerated_passes_over_a_regeneration_shield() {
    let mut game = at_the_turn_of(0, 2);
    let (bear, forest) = a_regenerated_bear_and_a_forest(&mut game);
    resolve_plague_spores(&mut game);
    assert!(!game.battlefield.contains_key(&bear), "the shield is not applied");
    assert!(!game.battlefield.contains_key(&forest));
}

/// CR 608.2c — "Destroy target creature. It can't be regenerated" is one
/// instruction: the clause is about that destruction, and ends with it. The
/// Bear lives through Plague Spores on a shield counter (CR 122.1c, which
/// is no regeneration), and the next destroy that turn, which says nothing
/// about regeneration, meets the Bear's shield and is replaced by it.
#[test]
fn it_cant_be_regenerated_ends_with_the_destruction_it_follows() {
    let mut game = at_the_turn_of(0, 2);
    let (bear, _) = a_regenerated_bear_and_a_forest(&mut game);
    game.execute_action(GameAction::AddCounters { subject: ObjectOrPlayerId::Object(bear), counter: CounterType::Shield, n: 1, by: 1 }, &test_ctx())
        .expect("a shield counter");
    resolve_plague_spores(&mut game);
    assert!(game.battlefield.contains_key(&bear), "the shield counter replaced the destruction");
    assert_eq!(game.battlefield[&bear].counter_count(CounterType::Shield), 0, "and was removed for it");

    let doom = instant("A Plain Destroy", "Destroy target creature.", destroy(target_creature()));
    cast_paying(&mut game, doom, &[(ManaType::Blue, 1)]);
    game.resolve_top_of_stack(&ScriptedDecisionProvider::new()).expect("it resolves");
    assert!(game.battlefield.contains_key(&bear), "the regeneration shield applies to a later destruction");
    assert!(game.battlefield[&bear].tapped, "regenerated: tapped (CR 701.19a)");
}

/// An amount reads exactly one answer. "That many" after several players
/// each chose a number names none of them, so the card must say which, and
/// one that does not is refused as it resolves.
#[test]
fn the_chosen_number_after_several_answers_is_refused() {
    let mut game = at_the_turn_of(0, 2);
    let card = instant(
        "Whose Number",
        "Each player chooses a number between 0 and 3. You draw that many cards.",
        Effect::Sequence(vec![choose_number(each_player(), 0, 3), draw_cards(you(), the_chosen_number())]),
    );
    cast_paying(&mut game, card, &[(ManaType::Blue, 1)]);

    let answers = ScriptedDecisionProvider::new();
    for number in [1, 2] {
        let any = XOffer::Interval { at_least: 0, at_most: 3 };
        answers.expect_number(DecisionKind::ChooseNumber { source: ObjectId::UNASSIGNED, values: Box::new(any) }, number);
    }
    let refused = game.resolve_top_of_stack(&answers).expect_err("two numbers, and a reader of one");
    assert!(refused.contains("2 were"), "{refused}");
}

// ---------------------------------------------------------------------------
// A pick limited by a total (CR 608.2d, 107.1b)
// ---------------------------------------------------------------------------

/// A creature of `power`, named for a board's failure message.
fn creature_of_power(name: &str, power: i32) -> Arc<CardData> {
    CardDataBuilder::new(name).color(Color::Green).card_type(CardType::Creature).power_toughness(power, 5).build()
}

/// The script's next answer to a pick under a total: `chosen`.
fn picks_within_the_total(answers: &ScriptedDecisionProvider, chosen: &[ObjectId]) {
    let kind = DecisionKind::SelectWithinATotal { source: ObjectId::UNASSIGNED, of: ObjectValue::Power, limit: TotalLimit::AtMost(0), numbers: vec![] };
    answers.expect_choice(kind, chosen.iter().copied().map(DecisionOption::Object).collect());
}

/// What each pick under a total showed its player, and why it was asked
/// again where it was: the limit, each option's number, and the refusal.
type TotalsAsked = Vec<(TotalLimit, Vec<(ObjectId, i64)>, Option<Rejection>)>;

/// Slaughter the Strong, cast by player 0 at their own turn and resolved
/// with `answers`.
fn resolve_slaughter_the_strong(game: &mut GameState, answers: ScriptedDecisionProvider) -> TotalsAsked {
    game.record_events();
    cast_paying(game, slaughter_the_strong(), &[(ManaType::White, 3)]);
    resolve_noting_each_total(game, answers)
}

/// The spell on the stack resolved with `answers`, and what each pick under
/// a total showed.
fn resolve_noting_each_total(game: &mut GameState, answers: ScriptedDecisionProvider) -> TotalsAsked {
    let asked = RefCell::new(Vec::new());
    let seat = AtEachPrompt::new(answers, |_: &GameState, context: &DecisionContext| {
        if let DecisionKind::SelectWithinATotal { limit, numbers, .. } = &context.kind {
            asked.borrow_mut().push((*limit, numbers.clone(), context.rejected.clone()));
        }
    });
    game.resolve_top_of_stack(&seat).expect("it resolves");
    asked.into_inner()
}

/// Each sacrifice performed, with the batch it was performed in.
fn sacrificed(game: &GameState) -> Vec<(ObjectId, Option<BatchId>)> {
    moved_by(game, ZoneChangeAct::Sacrificed)
}

/// Each object `act` moved, with the batch it was moved in.
fn moved_by(game: &GameState, act: ZoneChangeAct) -> Vec<(ObjectId, Option<BatchId>)> {
    let moved = |record: &EventRecord| match record.event {
        GameEvent::ZoneChange { ref departed, act: by, .. } if by == act => Some((departed.object().id, record.stamp.batch)),
        _ => None,
    };
    game.recorded_events().records().iter().filter_map(moved).collect()
}

// RULING: Slaughter the Strong #1 - "... checks that the total power of
//   creatures each player chose this way is 4 or less. For example, you
//   could save two 2/2 creatures, or a 1/1 and a 3/3 creature, but not all
//   four of those creatures."
// RULING: Slaughter the Strong #2 - "... first the player whose turn it is
//   chooses which creatures will be spared, then each other player in turn
//   order does the same knowing the choices made before them. Then all the
//   creatures not chosen are sacrificed simultaneously."
//
/// The legal answers are sets: player 0's question shows each creature's
/// power and the limit, all four (8) is refused with the total it came to
/// and asked again, and the 1/1 with the 3/3 (4) is taken. Player 1 keeps
/// the 4/4 and not the 5/5. Player 2's lone 7/7 fits no answer but the
/// empty one, so they are not asked. Every creature not chosen is then
/// sacrificed by its own controller (CR 701.21a), in one batch.
#[test]
fn slaughter_the_strong_takes_a_set_inside_the_total_from_each_player() {
    let mut game = at_the_turn_of(0, 3);
    let [two, other_two, one, three] = [2, 2, 1, 3].map(|power| put_on_battlefield(&mut game, creature_of_power("Player 0's", power), 0));
    let [five, four] = [5, 4].map(|power| put_on_battlefield(&mut game, creature_of_power("Player 1's", power), 1));
    let seven = put_on_battlefield(&mut game, creature_of_power("Player 2's", 7), 2);

    let answers = ScriptedDecisionProvider::new();
    picks_within_the_total(&answers, &[two, other_two, one, three]);
    picks_within_the_total(&answers, &[one, three]);
    picks_within_the_total(&answers, &[four]);
    let asked = resolve_slaughter_the_strong(&mut game, answers);

    let at_most_four = TotalLimit::AtMost(4);
    let player_0s = vec![(two, 2), (other_two, 2), (one, 1), (three, 3)];
    let too_much = Rejection::TotalOutsideTheLimit { of: ObjectValue::Power, total: 8, limit: at_most_four };
    assert_eq!(
        asked,
        [
            (at_most_four, player_0s.clone(), None),
            (at_most_four, player_0s, Some(too_much.clone())),
            (at_most_four, vec![(five, 5), (four, 4)], None),
        ],
        "player 0 twice, then player 1; player 2 has one legal answer and is not asked"
    );
    assert_eq!(rejection(&game, &too_much), "Those add up to 8, and the choice is of total power 4 or less (CR 608.2d)");

    let kept = [one, three, four].map(|id| game.battlefield.contains_key(&id));
    assert_eq!(kept, [true; 3], "what each player chose is spared");
    let gone = sacrificed(&game);
    let mut ids: Vec<ObjectId> = gone.iter().map(|(id, _)| *id).collect();
    ids.sort();
    let mut the_rest = vec![two, other_two, five, seven];
    the_rest.sort();
    assert_eq!(ids, the_rest, "each player's own, sacrificed and not destroyed");
    assert!(gone.windows(2).all(|pair| pair[0].1 == pair[1].1), "one batch");
}

// RULING: Slaughter the Strong #3 - "If a creature's power is somehow less
//   than 0, it subtracts from the total power of the other creatures its
//   controller chooses. This can cause creatures with power 5 or greater to
//   survive."
//
/// CR 107.1b — a calculation uses a negative number as it is. The 6/5
/// alone is over the limit, and beside the creature of power −2 the pick
/// adds up to 4: the question shows the −2, and both are spared.
#[test]
fn a_creature_of_negative_power_makes_room_under_the_total() {
    let mut game = at_the_turn_of(0, 2);
    let six = put_on_battlefield(&mut game, creature_of_power("A Giant", 6), 0);
    let minus_two = put_on_battlefield(&mut game, creature_of_power("A Weakling", -2), 0);

    let answers = ScriptedDecisionProvider::new();
    picks_within_the_total(&answers, &[six]);
    picks_within_the_total(&answers, &[six, minus_two]);
    let asked = resolve_slaughter_the_strong(&mut game, answers);

    let alone = Rejection::TotalOutsideTheLimit { of: ObjectValue::Power, total: 6, limit: TotalLimit::AtMost(4) };
    let shown = vec![(six, 6), (minus_two, -2)];
    assert_eq!(asked, [(TotalLimit::AtMost(4), shown.clone(), None), (TotalLimit::AtMost(4), shown, Some(alone))]);
    assert!(game.battlefield.contains_key(&six) && game.battlefield.contains_key(&minus_two), "6 and -2 are 4");
    assert!(sacrificed(&game).is_empty());
}

/// An invented instant: "Choose any number of creatures you control with
/// total power [`is`]. Destroy the rest."
fn keep_some_whose_total_power(is: ValueIs, text: &'static str) -> Arc<CardData> {
    let mine = || creature().and(you_control());
    let what = ValueChoice::Objects {
        among: SelectionFilter::Permanent(mine()),
        count: TargetCount::AnyNumber,
        total: Some(PickTotal { of: ObjectValue::Power, is }),
    };
    let keep = Effect::Atom(Primitive::ChooseValue { kind: ChoiceKind::Objects, what, how: you().into() }, EffectRecipient::Implicit);
    instant("Keep Some", text, Effect::Sequence(vec![keep, destroy(each(mine().other_than(one_of_the_chosen())))]))
}

/// "Total power 5 or greater" is a limit from below: one 3/5 is refused
/// with the 3 it comes to, and both are taken. Where no pick reaches the
/// limit the choice cannot be made (CR 608.2d): nobody is asked, nothing is
/// kept, and "destroy the rest", which reads it, does not happen.
#[test]
fn a_total_or_greater_is_asked_until_reached_and_never_made_out_of_reach() {
    let or_greater = || {
        let text = "Choose any number of creatures you control with total power 5 or greater. Destroy the rest.";
        keep_some_whose_total_power(ValueIs::AtLeast(5.into()), text)
    };
    let mut game = at_the_turn_of(0, 2);
    let [three, other_three, one] = [3, 3, 1].map(|power| put_on_battlefield(&mut game, creature_of_power("Mine", power), 0));
    cast_paying(&mut game, or_greater(), &[(ManaType::Blue, 1)]);
    let answers = ScriptedDecisionProvider::new();
    picks_within_the_total(&answers, &[three]);
    picks_within_the_total(&answers, &[three, other_three]);
    let asked = resolve_noting_each_total(&mut game, answers);

    let at_least_five = TotalLimit::AtLeast(5);
    let shown = vec![(three, 3), (other_three, 3), (one, 1)];
    let short = Rejection::TotalOutsideTheLimit { of: ObjectValue::Power, total: 3, limit: at_least_five };
    assert_eq!(asked, [(at_least_five, shown.clone(), None), (at_least_five, shown, Some(short.clone()))]);
    assert_eq!(rejection(&game, &short), "Those add up to 3, and the choice is of total power 5 or greater (CR 608.2d)");
    assert_eq!([three, other_three, one].map(|id| game.battlefield.contains_key(&id)), [true, true, false]);

    let mut game = at_the_turn_of(0, 2);
    let twos = [2, 2].map(|power| put_on_battlefield(&mut game, creature_of_power("Mine", power), 0));
    cast_paying(&mut game, or_greater(), &[(ManaType::Blue, 1)]);
    let asked = resolve_noting_each_total(&mut game, ScriptedDecisionProvider::new());
    assert!(asked.is_empty(), "4 is the most there is, so there is no choice to make");
    assert!(twos.iter().all(|id| game.battlefield.contains_key(id)), "nothing chosen is not everything unchosen");
}

/// "Total power less than 3" is 2 or less, a total being a whole number:
/// the 2/5 beside the 1/5 comes to 3 and is refused, and the 2/5 alone is
/// taken.
#[test]
fn a_total_less_than_a_number_is_at_most_one_fewer() {
    let mut game = at_the_turn_of(0, 2);
    let [two, one, three] = [2, 1, 3].map(|power| put_on_battlefield(&mut game, creature_of_power("Mine", power), 0));
    let text = "Choose any number of creatures you control with total power less than 3. Destroy the rest.";
    cast_paying(&mut game, keep_some_whose_total_power(ValueIs::LessThan(3.into()), text), &[(ManaType::Blue, 1)]);
    let answers = ScriptedDecisionProvider::new();
    picks_within_the_total(&answers, &[two, one]);
    picks_within_the_total(&answers, &[two]);
    let asked = resolve_noting_each_total(&mut game, answers);

    let at_most_two = TotalLimit::AtMost(2);
    let over = Rejection::TotalOutsideTheLimit { of: ObjectValue::Power, total: 3, limit: at_most_two };
    let limits_and_refusals: Vec<(TotalLimit, Option<Rejection>)> = asked.into_iter().map(|(limit, _, rejected)| (limit, rejected)).collect();
    assert_eq!(limits_and_refusals, [(at_most_two, None), (at_most_two, Some(over))]);
    assert_eq!([two, one, three].map(|id| game.battlefield.contains_key(&id)), [true, false, false]);
}

// ---------------------------------------------------------------------------
// A verb on what several players chose
// ---------------------------------------------------------------------------

/// "Destroy each creature chosen this way": the verb acts on every answer's
/// objects, each once. Two players choose the same creature, and it is
/// destroyed in one event beside the third player's.
#[test]
fn a_verb_acts_once_on_what_two_players_both_chose() {
    let mut game = at_the_turn_of(0, 3);
    game.record_events();
    let [shared, other, spared] = [(); 3].map(|_| put_on_battlefield(&mut game, creature_of_color("A Bear", Color::Green), 2));
    let card = instant(
        "Marked Apiece",
        "Each player chooses a creature. Destroy each creature chosen this way.",
        Effect::Sequence(vec![choose_objects(each_player(), creature(), TargetCount::Exactly(1)), destroy(the_chosen_objects())]),
    );
    cast_paying(&mut game, card, &[(ManaType::Blue, 1)]);

    let answers = ScriptedDecisionProvider::new();
    for chosen in [shared, shared, other] {
        picks(&answers, vec![DecisionOption::Object(chosen)]);
    }
    game.resolve_top_of_stack(&answers).expect("it resolves");

    let destroyed: Vec<ObjectId> = moved_by(&game, ZoneChangeAct::Destroyed).into_iter().map(|(id, _)| id).collect();
    assert_eq!(destroyed, [shared, other], "the creature two players chose is destroyed once");
    assert!(game.battlefield.contains_key(&spared));
}

/// The same for players: "each player chosen this way draws a card" is one
/// card for a player two of them chose.
#[test]
fn each_chosen_player_acts_once_however_many_chose_them() {
    let mut game = at_the_turn_of(0, 3);
    let card = instant(
        "Named Apiece",
        "Each player chooses a player. Each player chosen this way draws a card.",
        Effect::Sequence(vec![choose_player(each_player(), PlayerSet::Everyone), draw_cards(the_chosen_player(), 1)]),
    );
    cast_paying(&mut game, card, &[(ManaType::Blue, 1)]);
    let hands = [0, 1, 2].map(|seat: usize| game.players[seat].hand.len());

    let answers = ScriptedDecisionProvider::new();
    for chosen in [1, 1, 0] {
        answers.expect_choice(DecisionKind::ChoosePlayer { source: ObjectId::UNASSIGNED }, vec![DecisionOption::Player(chosen)]);
    }
    game.resolve_top_of_stack(&answers).expect("it resolves");

    let drawn = [0, 1, 2].map(|seat: usize| game.players[seat].hand.len() - hands[seat]);
    assert_eq!(drawn, [1, 1, 0], "player 1 was chosen twice and draws once");
}

// ---------------------------------------------------------------------------
// The cards
// ---------------------------------------------------------------------------

/// A 2/2 of one creature type.
fn creature_of_type(creature_type: CreatureType) -> Arc<CardData> {
    CardDataBuilder::new("A Creature")
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(creature_type))
        .power_toughness(2, 2)
        .build()
}

// COVERS-PARTIAL: ATOM-101.4c-001
/// Harsh Mercy. Each player chooses a type, the active player first, and
/// every type is chosen before anything is destroyed (CR 101.4c: the
/// choosing is one step and the destroying the next). A creature of any
/// chosen type is spared, whoever chose it: two players naming Goblin spare
/// the one Goblin. The Wall's regeneration shield is passed over, since the
/// destroy carries "they can't be regenerated".
#[test]
fn harsh_mercy_spares_a_creature_of_any_type_a_player_chose() {
    let mut game = at_the_turn_of(0, 3);
    // Regenerated while it is the board's one creature, so the shield has
    // one target and asks nothing.
    let wall = put_on_battlefield(&mut game, creature_of_type(CreatureType::Wall), 1);
    let shield = instant("A Shield", "Regenerate target creature.", Effect::Atom(Primitive::Regenerate, target_creature()));
    cast_paying(&mut game, shield, &[(ManaType::Blue, 1)]);
    game.resolve_top_of_stack(&ScriptedDecisionProvider::new()).expect("the shield is made");
    let goblin = put_on_battlefield(&mut game, creature_of_type(CreatureType::Goblin), 0);
    let elf = put_on_battlefield(&mut game, creature_of_type(CreatureType::Elf), 1);
    let zombie = put_on_battlefield(&mut game, creature_of_type(CreatureType::Zombie), 2);
    cast_paying(&mut game, harsh_mercy(), &[(ManaType::White, 3)]);

    let answers = ScriptedDecisionProvider::new();
    for creature_type in [CreatureType::Goblin, CreatureType::Elf, CreatureType::Goblin] {
        answers.expect_choice(DecisionKind::ChooseCreatureType { source: ObjectId::UNASSIGNED }, vec![DecisionOption::CreatureType(creature_type)]);
    }
    let creatures_at_each_question = RefCell::new(Vec::new());
    let seat = AtEachPrompt::new(answers, |game: &GameState, _: &DecisionContext| {
        creatures_at_each_question.borrow_mut().push(game.battlefield.len());
    });
    game.resolve_top_of_stack(&seat).expect("it resolves");

    assert_eq!(creatures_at_each_question.into_inner(), [4, 4, 4], "every type is chosen before any creature is destroyed");
    let on_the_battlefield = [goblin, elf, zombie, wall].map(|id| game.battlefield.contains_key(&id));
    assert_eq!(on_the_battlefield, [true, true, false, false], "Goblin and Elf were chosen; the Wall's shield is not applied");
}

// RULING: Fortunate Few #1 - "After you choose a nonland permanent, each
//   other player does so in turn order starting from the player whose turn
//   it is. Each player will know the choice made by previous players."
// RULING: Fortunate Few #2 - "Once Fortunate Few begins resolving, no
//   player may take any actions other than those specified by Fortunate Few
//   until it's done resolving. ..."
// RULING: Fortunate Few #3 - "A permanent with indestructible can be chosen
//   this way even though it wouldn't be destroyed otherwise."
// RULING: Fortunate Few #4 - "Lands can't be chosen and won't be destroyed,
//   even if they have other types (such as an artifact land)."
//
/// You choose among what you don't control, then each other player among
/// what they don't control and no one has chosen: three questions and
/// nothing else, each offering one permanent fewer for each pick made
/// before it. The artifact land is offered to nobody and is not destroyed;
/// the indestructible creature is offered, and chosen.
#[test]
fn fortunate_few_asks_you_then_each_other_player_among_what_is_not_chosen() {
    let mut game = at_the_turn_of(0, 3);
    let permanent = |name: &str, types: &[CardType], flags: &[KeywordFlag]| {
        let builder = types.iter().fold(CardDataBuilder::new(name), |builder, card_type| builder.card_type(*card_type));
        flags.iter().fold(builder, |builder, flag| builder.keyword_flag(*flag)).power_toughness(2, 2).build()
    };
    let my_artifact = put_on_battlefield(&mut game, permanent("My Artifact", &[CardType::Artifact], &[]), 0);
    let my_creature = put_on_battlefield(&mut game, permanent("My Creature", &[CardType::Creature], &[]), 0);
    let artifact_land = put_on_battlefield(&mut game, permanent("An Artifact Land", &[CardType::Artifact, CardType::Land], &[]), 0);
    let their_creature = put_on_battlefield(&mut game, permanent("Player 1's Creature", &[CardType::Creature], &[]), 1);
    let indestructible = put_on_battlefield(&mut game, permanent("Player 1's Rock", &[CardType::Creature], &[KeywordFlag::Indestructible]), 1);
    let last_creature = put_on_battlefield(&mut game, permanent("Player 2's Creature", &[CardType::Creature], &[]), 2);
    let last_enchantment = put_on_battlefield(&mut game, permanent("Player 2's Enchantment", &[CardType::Enchantment], &[]), 2);
    cast_paying(&mut game, fortunate_few(), &[(ManaType::White, 5)]);

    let answers = ScriptedDecisionProvider::new();
    picks(&answers, vec![DecisionOption::Object(indestructible)]);
    picks(&answers, vec![DecisionOption::Object(my_creature)]);
    picks(&answers, vec![DecisionOption::Object(their_creature)]);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let asked = asked.of_options.borrow();
    let offered: Vec<(PlayerId, Vec<DecisionOption>)> = asked.iter().map(|(player, _, options)| (*player, options.clone())).collect();
    let objects = |ids: &[ObjectId]| ids.iter().copied().map(DecisionOption::Object).collect::<Vec<_>>();
    assert_eq!(
        offered,
        [
            (0, objects(&[their_creature, indestructible, last_creature, last_enchantment])),
            (1, objects(&[my_artifact, my_creature, last_creature, last_enchantment])),
            (2, objects(&[my_artifact, their_creature])),
        ],
        "no land, nothing of the chooser's own, and nothing chosen before"
    );
    let on_the_battlefield =
        [indestructible, my_creature, their_creature, artifact_land, my_artifact, last_creature, last_enchantment].map(|id| game.battlefield.contains_key(&id));
    assert_eq!(on_the_battlefield, [true, true, true, true, false, false, false], "the three chosen and the land stay; the rest is destroyed");
}

/// Choice of Damnations at two seats, where player 1 is its one legal
/// target: player 1 chooses `number`, player 0 answers the "may", and player
/// 1 keeps `kept` where a keep is asked. Who was asked what, in order.
fn resolve_choice_of_damnations(game: &mut GameState, number: u64, lose_the_life: bool, kept: Option<ObjectId>) -> Vec<(PlayerId, &'static str)> {
    game.record_events();
    let spell = cast_paying(game, choice_of_damnations(), &[(ManaType::Black, 6)]);
    let answers = ScriptedDecisionProvider::new();
    let any_number = XOffer::NoTop { at_least: 0 };
    answers.expect_number(DecisionKind::ChooseNumber { source: ObjectId::UNASSIGNED, values: Box::new(any_number) }, number);
    let taken = if lose_the_life { vec![DecisionOption::Object(spell)] } else { vec![] };
    answers.expect_choice(DecisionKind::ApplyOptionalEffect { source: ObjectId::UNASSIGNED }, taken);
    if let Some(kept) = kept {
        picks(&answers, vec![DecisionOption::Object(kept)]);
    }
    game.resolve_top_of_stack(&answers).expect("it resolves");
    answers.asked_of().into_iter().map(|(player, kind)| (player, kind.as_str())).collect()
}

// RULING: Choice of Damnations #1 - "After the opponent chooses a number,
//   Choice of Damnations' controller may choose to make that opponent
//   either lose that much life or make that opponent sacrifice permanents."
//
/// The opponent's number comes first and is any number at all (CR 107.1b:
/// no most printed). Its controller then takes the life: 13 of it, and no
/// permanent is touched.
#[test]
fn choice_of_damnations_has_the_opponent_lose_the_number_they_chose() {
    let mut game = at_the_turn_of(0, 2);
    let theirs = [(); 3].map(|_| put_on_battlefield(&mut game, creature_of_color("Their Bear", Color::Green), 1));
    let life = game.players[1].life_total;

    let asked = resolve_choice_of_damnations(&mut game, 13, true, None);

    assert_eq!(asked, [(1, "ChooseNumber"), (0, "ApplyOptionalEffect")], "the opponent's number, then its controller's may");
    assert_eq!(game.players[1].life_total, life - 13);
    assert!(theirs.iter().all(|id| game.battlefield.contains_key(id)));
}

/// CR 107.1b — a number with no most printed is any number, and the
/// greatest one there is to answer with is past a life total's range. The
/// opponent who names it loses that much: their life total goes to its
/// least, and never back up through the top.
#[test]
fn the_greatest_number_there_is_loses_life_and_gains_none() {
    let mut game = at_the_turn_of(0, 2);
    let life = game.players[1].life_total;

    resolve_choice_of_damnations(&mut game, u64::MAX, true, None);

    assert_eq!(game.players[1].life_total, life.saturating_sub(i64::MAX), "a loss, to the least a life total holds");
}

/// Zero is a number (CR 107.1b). Taken as life it is no loss; declined, the
/// opponent keeps no permanent, with nothing to choose, and sacrifices
/// every one they control.
#[test]
fn choice_of_damnations_at_zero_keeps_nothing() {
    let mut game = at_the_turn_of(0, 2);
    let theirs = [(); 3].map(|_| put_on_battlefield(&mut game, creature_of_color("Their Bear", Color::Green), 1));
    let life = game.players[1].life_total;
    let asked = resolve_choice_of_damnations(&mut game, 0, true, None);
    assert_eq!(asked, [(1, "ChooseNumber"), (0, "ApplyOptionalEffect")]);
    assert_eq!(game.players[1].life_total, life, "no life is lost");
    assert!(theirs.iter().all(|id| game.battlefield.contains_key(id)));

    let mut game = at_the_turn_of(0, 2);
    let mine = put_on_battlefield(&mut game, creature_of_color("My Bear", Color::Green), 0);
    let theirs = [(); 3].map(|_| put_on_battlefield(&mut game, creature_of_color("Their Bear", Color::Green), 1));
    let asked = resolve_choice_of_damnations(&mut game, 0, false, None);
    assert_eq!(asked, [(1, "ChooseNumber"), (0, "ApplyOptionalEffect")], "none of three is no choice");
    assert!(theirs.iter().all(|id| !game.battlefield.contains_key(id)), "all but none");
    assert_eq!(sacrificed(&game).len(), 3);
    assert!(game.battlefield.contains_key(&mine));
}

// RULING: Choice of Damnations #2 - "If the opponent must sacrifice all but
//   a number of permanents, that opponent chooses that many permanents and
//   then sacrifices the rest. If the number chosen is greater than the
//   number of permanents the opponent controls, the player sacrifices
//   nothing."
//
/// "If you don't": the opponent keeps one permanent of their choice and
/// sacrifices their other two, and player 0's own permanent is no part of
/// it. With a number above what they control, every permanent is kept
/// without a question and nothing is sacrificed.
#[test]
fn choice_of_damnations_has_the_opponent_keep_that_many_and_sacrifice_the_rest() {
    let mut game = at_the_turn_of(0, 2);
    let mine = put_on_battlefield(&mut game, creature_of_color("My Bear", Color::Green), 0);
    let theirs = [(); 3].map(|_| put_on_battlefield(&mut game, creature_of_color("Their Bear", Color::Green), 1));
    let life = game.players[1].life_total;

    let asked = resolve_choice_of_damnations(&mut game, 1, false, Some(theirs[1]));

    assert_eq!(asked, [(1, "ChooseNumber"), (0, "ApplyOptionalEffect"), (1, "SelectRecipients")]);
    assert_eq!(game.players[1].life_total, life, "the life was not taken");
    assert_eq!(theirs.map(|id| game.battlefield.contains_key(&id)), [false, true, false], "one kept, by its controller's choice");
    assert!(game.battlefield.contains_key(&mine), "the caster's permanents are not that player's");
    let mut gone: Vec<ObjectId> = sacrificed(&game).into_iter().map(|(id, _)| id).collect();
    gone.sort();
    assert_eq!(gone, [theirs[0], theirs[2]], "sacrificed, and by the player who controls them");

    let mut game = at_the_turn_of(0, 2);
    let theirs = [(); 3].map(|_| put_on_battlefield(&mut game, creature_of_color("Their Bear", Color::Green), 1));
    let asked = resolve_choice_of_damnations(&mut game, 7, false, None);
    assert_eq!(asked, [(1, "ChooseNumber"), (0, "ApplyOptionalEffect")], "seven of three keeps all three, with nothing to ask");
    assert!(theirs.iter().all(|id| game.battlefield.contains_key(id)) && sacrificed(&game).is_empty());
}

// ---------------------------------------------------------------------------
// Three players chosen one at a time, each a different one
// ---------------------------------------------------------------------------

/// Gluntch, the Bestower's three choices on an instant, with a life gain
/// where its third player creates Treasure tokens: "Choose a player. They
/// put two +1/+1 counters on a creature they control. Choose a second
/// player to draw a card. Then choose a third player to gain 2 life."
fn three_favors() -> Arc<CardData> {
    let they_put_two_counters = Primitive::AddCounters {
        counter: CounterType::PlusOnePlusOne,
        amount: 2.into(),
        by: PlayerRef::NamedBy(Box::new(the_chosen_player())),
    };
    instant(
        "Three Favors",
        "Choose a player. They put two +1/+1 counters on a creature they control. Choose a second player to draw a card. \
         Then choose a third player to gain 2 life.",
        Effect::Sequence(vec![
            choose_player(you(), PlayerSet::Everyone),
            choose_objects(the_chosen_player(), creature().and(they_control()), TargetCount::Exactly(1)),
            Effect::Atom(they_put_two_counters, the_chosen_objects()),
            choose_a_second_player(you(), PlayerSet::Everyone),
            draw_cards(the_second_chosen_player(), 1),
            choose_a_third_player(you(), PlayerSet::Everyone),
            gain_life(the_third_chosen_player(), 2),
        ]),
    )
}

/// The script's next answer to "choose a player": `player`.
fn chooses_player(answers: &ScriptedDecisionProvider, player: PlayerId) {
    answers.expect_choice(DecisionKind::ChoosePlayer { source: ObjectId::UNASSIGNED }, vec![DecisionOption::Player(player)]);
}

// RULING: Gluntch, the Bestower #1 - "The first chosen player gets to
//   choose which creature they will place the counters on."
// RULING: Gluntch, the Bestower #3 - "All three players must be different
//   players, although any one of them can be you. ..."
//
/// The first player chosen picks the creature among their own. The second
/// question leaves the first player out, and the third has one player left
/// and is not asked: three different players, one of them the controller.
#[test]
fn three_players_chosen_in_turn_are_three_different_players() {
    let mut game = at_the_turn_of(0, 3);
    let theirs = [(); 2].map(|_| put_on_battlefield(&mut game, creature_of_color("Their Bear", Color::Green), 1));
    put_on_battlefield(&mut game, creature_of_color("My Bear", Color::Green), 0);
    cast_paying(&mut game, three_favors(), &[(ManaType::Blue, 1)]);
    let (hand, life) = (game.players[0].hand.len(), game.players[2].life_total);

    let answers = ScriptedDecisionProvider::new();
    chooses_player(&answers, 1);
    picks(&answers, vec![DecisionOption::Object(theirs[1])]);
    chooses_player(&answers, 0);
    let (seat, asked) = Watching::new(answers);
    game.resolve_top_of_stack(&seat).expect("it resolves");

    let asked = asked.of_options.borrow();
    let offered: Vec<(PlayerId, Vec<DecisionOption>)> = asked.iter().map(|(player, _, options)| (*player, options.clone())).collect();
    let players = |seats: &[PlayerId]| seats.iter().copied().map(DecisionOption::Player).collect::<Vec<_>>();
    assert_eq!(
        offered,
        [(0, players(&[0, 1, 2])), (1, theirs.map(DecisionOption::Object).to_vec()), (0, players(&[0, 2]))],
        "any player; the chosen player's own creatures; then any other player"
    );
    assert_eq!(game.battlefield[&theirs[1]].counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(game.players[0].hand.len(), hand + 1, "the second player drew");
    assert_eq!(game.players[2].life_total, life + 2, "the one player left is the third");
}

// RULING: Gluntch, the Bestower #2 - "You may choose a player who doesn't
//   control any creatures as the first chosen player."
// RULING: Gluntch, the Bestower #3 - "... If there are only two players in
//   the game, you won't choose a third player, and no one will create
//   Treasure tokens."
//
/// At two seats: a player with no creature is chosen first and puts no
/// counters; the second is the one player left; and no third player is
/// chosen, so the step that reads one does not happen (CR 608.2b).
#[test]
fn with_two_players_no_third_player_is_chosen() {
    let mut game = at_the_turn_of(0, 2);
    let mine = put_on_battlefield(&mut game, creature_of_color("My Bear", Color::Green), 0);
    cast_paying(&mut game, three_favors(), &[(ManaType::Blue, 1)]);
    let (hand, lives) = (game.players[0].hand.len(), [0, 1].map(|seat: usize| game.players[seat].life_total));

    let answers = ScriptedDecisionProvider::new();
    chooses_player(&answers, 1);
    game.resolve_top_of_stack(&answers).expect("it resolves");

    assert_eq!(asked_in_order(&answers), [0], "the first player is the one question");
    assert_eq!(game.battlefield[&mine].counter_count(CounterType::PlusOnePlusOne), 0, "player 1 controls no creature to put them on");
    assert_eq!(game.players[0].hand.len(), hand + 1, "the second player is the one left: the controller");
    assert_eq!([0, 1].map(|seat: usize| game.players[seat].life_total), lives, "nobody is the third player");
}
