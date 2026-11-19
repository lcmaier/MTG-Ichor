//! `several-choosers-at-one-step` (`choices-architecture.md`), the second of
//! `values-chosen-at-resolution`'s three: a choice each player makes, asked
//! around the table from the active player, each answer kept with its
//! player, and what the steps after it read of them.

use std::sync::Arc;

#[path = "support/watching.rs"]
mod watching;
use watching::Watching;

use mtgsim::autopilot::mana_window_stop::ManaWindowStop;
use mtgsim::cards::authoring::{
    choose_color, choose_number, choose_number_at_least, choose_objects, choose_player, color, creature, destroy, draw_cards, each,
    each_opponent, each_player, gain_life, non, one_of_the_chosen, spell_ability, target_creature, target_opponent, the_chosen_color,
    the_chosen_number, the_chosen_objects, the_chosen_player, they_control, you,
};
use mtgsim::decision::provider::ScriptedDecisionProvider;
use mtgsim::decision::question::{DecisionKind, DecisionOption};
use mtgsim::engine::actions::GameAction;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::x_values::XOffer;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{fill_library, pass_turn, put_in_hand, put_on_battlefield, setup_game, test_ctx};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{Effect, EffectRecipient, PlayerRef, PlayerSet, TargetCount};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};

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

/// "Each player chooses a creature they control": "they" is the player the
/// walk is on, so each is offered their own creatures and no one else's,
/// where "you control" would offer every player the controller's. A player
/// with none to choose gives an answer that chose nothing, and the rest is
/// still read against every answer.
#[test]
fn each_player_chooses_among_what_they_control() {
    let mut game = at_the_turn_of(1, 3);
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
    let id = put_in_hand(game, card, 0);
    game.players[0].mana_pool.add(ManaType::Blue, 1);
    let targeting = ScriptedDecisionProvider::new();
    picks(&targeting, vec![DecisionOption::Player(1)]);
    game.cast_spell(0, id, &ManaWindowStop::new(targeting)).expect("castable");
    game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).expect("player 1 leaves");
    game.resolve_top_of_stack(&ScriptedDecisionProvider::new()).map(|_| ())
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
