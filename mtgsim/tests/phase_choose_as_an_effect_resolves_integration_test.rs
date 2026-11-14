//! `choose-as-an-effect-resolves`: an object chosen without targeting it is
//! chosen as the effect applies (CR 608.2d, 115.10), and a division among
//! objects not targeted is made then too, each chosen receiving at least one
//! (`choices-architecture.md`, decision 8 and decision 5's untargeted half).

use std::sync::Arc;

use mtgsim::cards::authoring::{creatures_you_control, spell_ability};
use mtgsim::cards::printings::{feast_of_the_victorious_dead, final_showdown, grizzly_bears, savannah_lions};
use mtgsim::engine::actions::{GameAction, ZoneChangeAct};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::has_keyword;
use mtgsim::oracle::mana_helpers::castable_spells;
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::test_support::{fill_library, put_in_hand, put_on_battlefield, setup_game, test_ctx, test_dp, vanilla_creature};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, CounterType, DividedCount, Duration, Effect, EffectRecipient, PlayerRef, Primitive, SelectionFilter, TargetCount, TokenDef,
};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

/// **Fixture.** Final Showdown's second mode alone: "Choose a creature you
/// control. It gains indestructible until end of turn."
fn showdown_fixture() -> Arc<CardData> {
    let text = "Choose a creature you control. It gains indestructible until end of turn.";
    CardDataBuilder::new("Fixture Showdown")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Atom(
                Primitive::GrantKeywordFlag(KeywordFlag::Indestructible, Duration::UntilEndOfTurn),
                EffectRecipient::Choose(SelectionFilter::Permanent(creatures_you_control()), TargetCount::Exactly(1)),
            ),
        ))
        .build()
}

/// **Fixture.** Blessing of Frost's first sentence with its X at three, the
/// snow mana spent being no count the engine keeps: "Distribute three +1/+1
/// counters among any number of creatures you control."
fn frost_blessing_fixture() -> Arc<CardData> {
    let text = "Distribute three +1/+1 counters among any number of creatures you control.";
    let among = TargetCount::Divided(Box::new(DividedCount { among: TargetCount::AnyNumber, total: AmountExpr::Fixed(3) }));
    CardDataBuilder::new("Fixture Frost Blessing")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Atom(
                Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(3), by: PlayerRef::You },
                EffectRecipient::Choose(SelectionFilter::Permanent(creatures_you_control()), among),
            ),
        ))
        .build()
}

/// `card` in player 0's hand, cast from a pool of exactly its {W} or {G},
/// with `answers` answering the cast. The spell's id.
fn cast(game: &mut GameState, card: Arc<CardData>, mana: ManaType, answers: ScriptedDecisionProvider) -> (ObjectId, ScriptedDecisionProvider) {
    let id = put_in_hand(game, card, 0);
    game.players[0].mana_pool.add(mana, 1);
    let seat = ManaWindowStop::new(answers);
    game.cast_spell(0, id, &seat).expect("castable from exactly its cost");
    (id, seat.into_inner())
}

/// The script's next answer: `chosen`, picked from what a choice offers.
fn picks(answers: &ScriptedDecisionProvider, chosen: Vec<ChoiceOption>) {
    answers.expect_choice(ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED }, chosen);
}

/// The script's next answer: `shares`, the division among what was chosen.
fn divides(answers: &ScriptedDecisionProvider, shares: Vec<(ChoiceOption, u64)>) {
    answers.expect_allocation(ChoiceKind::DivideAmong { source: ObjectId::UNASSIGNED }, shares);
}

fn the_asked(answers: &ScriptedDecisionProvider) -> Vec<&'static str> {
    answers.asked().iter().map(ChoiceKind::as_str).collect()
}

fn plus_ones(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].counter_count(CounterType::PlusOnePlusOne)
}

/// CR 608.2d: "choose a creature you control" asks nothing as the spell is
/// cast, and asks its controller as it resolves, which is when the creature
/// is chosen. Final Showdown's first ruling: "You don't choose which creature
/// will gain indestructible until the spell is resolving."
// COVERS-PARTIAL: ATOM-608.2d-001
#[test]
fn a_choice_is_asked_as_the_spell_resolves_and_not_as_it_is_cast() {
    let mut game = setup_game(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let lions = put_on_battlefield(&mut game, savannah_lions(), 0);

    let (spell, at_cast) = cast(&mut game, showdown_fixture(), ManaType::White, ScriptedDecisionProvider::new());
    assert!(the_asked(&at_cast).is_empty(), "casting it asks nothing: {:?}", the_asked(&at_cast));
    assert!(game.stack_entries[&spell].announced.targets().next().is_none(), "no instance of \"target\"");

    let at_resolution = ScriptedDecisionProvider::new();
    picks(&at_resolution, vec![ChoiceOption::Object(lions)]);
    game.resolve_top_of_stack(&at_resolution).expect("the spell resolves");

    assert_eq!(the_asked(&at_resolution), ["SelectRecipients"]);
    assert!(has_keyword(&game, lions, KeywordFlag::Indestructible));
    assert!(!has_keyword(&game, bears, KeywordFlag::Indestructible));
}

/// A choice is no target, so CR 601.2c's "you can't cast this spell unless
/// you have legal choices for all its targets" asks nothing of it: with no
/// creature to choose the spell is offered and cast, and does nothing as it
/// resolves (Crush Underfoot's ruling: "the spell will finish resolving with
/// no effect").
#[test]
fn a_spell_that_chooses_is_castable_with_nothing_to_choose() {
    let mut game = setup_game(2);
    let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let showdown = put_in_hand(&mut game, showdown_fixture(), 0);
    game.players[0].mana_pool.add(ManaType::White, 1);
    assert!(castable_spells(&game, 0).contains(&showdown), "offered with no creature of its controller's");

    game.cast_spell(0, showdown, &ManaWindowStop::new(test_dp())).expect("cast with no creature to choose");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves, choosing nothing");
    assert!(!has_keyword(&game, theirs, KeywordFlag::Indestructible), "another player's creature is no choice");
    assert!(game.continuous_effects.is_empty());
}

/// Shroud stops targeting by its own controller too (CR 702.18a), and a
/// choice is no targeting (CR 115.10a): the creature with shroud is chosen.
/// Blessing of Frost's ruling says the same of protection: "Because none of
/// the creatures are targeted, you can choose to put +1/+1 counters on a
/// creature with protection from green."
#[test]
fn shroud_does_not_stop_a_choice() {
    let mut game = setup_game(2);
    let shrouded = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Shroud]), 0);
    let (_, _) = cast(&mut game, showdown_fixture(), ManaType::White, ScriptedDecisionProvider::new());
    game.resolve_top_of_stack(&test_dp()).expect("the spell resolves");
    assert!(has_keyword(&game, shrouded, KeywordFlag::Indestructible), "the one creature, chosen unasked");
}

/// CR 608.2d: a division among objects not targeted is chosen and divided as
/// the effect applies, each chosen object receiving at least one. Nothing is
/// asked as the spell is cast.
// COVERS: ATOM-608.2d-002
#[test]
fn a_division_among_creatures_not_targeted_is_made_as_the_spell_resolves() {
    let mut game = setup_game(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let lions = put_on_battlefield(&mut game, savannah_lions(), 0);

    let (spell, at_cast) = cast(&mut game, frost_blessing_fixture(), ManaType::Green, ScriptedDecisionProvider::new());
    assert!(the_asked(&at_cast).is_empty(), "casting it asks nothing: {:?}", the_asked(&at_cast));
    assert!(game.stack_entries[&spell].announced.targets().next().is_none());

    let at_resolution = ScriptedDecisionProvider::new();
    picks(&at_resolution, vec![ChoiceOption::Object(bears), ChoiceOption::Object(lions)]);
    divides(&at_resolution, vec![(ChoiceOption::Object(bears), 2), (ChoiceOption::Object(lions), 1)]);
    game.resolve_top_of_stack(&at_resolution).expect("the spell resolves");

    assert_eq!(the_asked(&at_resolution), ["SelectRecipients", "DivideAmong"]);
    assert_eq!((plus_ones(&game, bears), plus_ones(&game, lions)), (2, 1));
}

/// CR 608.2d: fewer objects than the most may be chosen, each still receiving
/// at least one; the creature not chosen gets nothing.
// COVERS: ATOM-608.2d-003
#[test]
fn a_division_may_be_among_fewer_objects_than_it_could_be() {
    let mut game = setup_game(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let lions = put_on_battlefield(&mut game, savannah_lions(), 0);
    let third = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let (_, _) = cast(&mut game, frost_blessing_fixture(), ManaType::Green, ScriptedDecisionProvider::new());

    let at_resolution = ScriptedDecisionProvider::new();
    picks(&at_resolution, vec![ChoiceOption::Object(bears), ChoiceOption::Object(lions)]);
    divides(&at_resolution, vec![(ChoiceOption::Object(bears), 2), (ChoiceOption::Object(lions), 1)]);
    game.resolve_top_of_stack(&at_resolution).expect("the spell resolves");

    assert_eq!((plus_ones(&game, bears), plus_ones(&game, lions), plus_ones(&game, third)), (2, 1, 0));
}

/// CR 608.2d's "each chosen ... receives at least one": three counters
/// divided 3/0 between two chosen creatures is no answer.
#[test]
#[should_panic(expected = "ask_divide_among: DP")]
fn a_chosen_object_receives_at_least_one() {
    let mut game = setup_game(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let lions = put_on_battlefield(&mut game, savannah_lions(), 0);
    let (_, _) = cast(&mut game, frost_blessing_fixture(), ManaType::Green, ScriptedDecisionProvider::new());

    let at_resolution = ScriptedDecisionProvider::new();
    picks(&at_resolution, vec![ChoiceOption::Object(bears), ChoiceOption::Object(lions)]);
    divides(&at_resolution, vec![(ChoiceOption::Object(bears), 3), (ChoiceOption::Object(lions), 0)]);
    let _ = game.resolve_top_of_stack(&at_resolution);
}

/// "Distribute three counters" places all three: with one creature to choose
/// it is chosen unasked and takes the three, nothing divided.
#[test]
fn a_division_with_one_object_to_choose_puts_everything_on_it() {
    let mut game = setup_game(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let (_, _) = cast(&mut game, frost_blessing_fixture(), ManaType::Green, ScriptedDecisionProvider::new());

    let at_resolution = ScriptedDecisionProvider::new();
    game.resolve_top_of_stack(&at_resolution).expect("the spell resolves");
    assert!(the_asked(&at_resolution).is_empty(), "one choice and one split are no question");
    assert_eq!(plus_ones(&game, bears), 3);
}

// ---------------------------------------------------------------------------
// The printed cards
// ---------------------------------------------------------------------------

/// The script's next answer: the spree modes at `places`.
fn chooses_modes(answers: &ScriptedDecisionProvider, places: &[u8]) {
    answers.expect_choice(ChoiceKind::ChooseModes { source: ObjectId::UNASSIGNED, words: Vec::new() }, vec![ChoiceOption::modes(places)]);
}

/// Final Showdown cast by player 0 with the modes at `places`, from enough
/// white mana for every mode. What casting it asked.
fn cast_final_showdown(game: &mut GameState, places: &[u8]) -> Vec<&'static str> {
    let showdown = put_in_hand(game, final_showdown(), 0);
    game.players[0].mana_pool.add(ManaType::White, 8);
    let answers = ScriptedDecisionProvider::new();
    chooses_modes(&answers, places);
    let seat = ManaWindowStop::new(answers);
    game.cast_spell(0, showdown, &seat).expect("castable");
    the_asked(&seat.into_inner())
}

// RULING: Final Showdown #1 - "The second mode of Final Showdown doesn't target the creature. You don't choose which creature will gain indestructible until the spell is resolving. At that point, it's too late for anyone to respond to the spell."
#[test]
fn final_showdown_chooses_its_creature_as_it_resolves() {
    let mut game = setup_game(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let lions = put_on_battlefield(&mut game, savannah_lions(), 0);

    assert_eq!(cast_final_showdown(&mut game, &[2]), ["ChooseModes"], "no creature is named as it is cast");
    let at_resolution = ScriptedDecisionProvider::new();
    picks(&at_resolution, vec![ChoiceOption::Object(lions)]);
    game.resolve_top_of_stack(&at_resolution).expect("it resolves");

    assert_eq!(the_asked(&at_resolution), ["SelectRecipients"]);
    assert!(has_keyword(&game, lions, KeywordFlag::Indestructible));
    assert!(!has_keyword(&game, bears, KeywordFlag::Indestructible));
}

/// All three modes, in printed order (CR 700.2): every creature loses its
/// abilities, then the creature chosen gains indestructible, later than the
/// loss (CR 613.7), then every creature is destroyed. The chosen one stays;
/// player 0's other creature and player 1's printed-indestructible one die.
// RULING: Final Showdown #2 - "If an effect grants a creature an ability after Final Showdown causes all creatures to lose all abilities, that creature won't lose that ability."
// RULING: Final Showdown #8 - "No matter which modes you choose, you always follow the instructions in the order they are written."
#[test]
fn final_showdown_spares_the_creature_it_chose() {
    let mut game = setup_game(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let lions = put_on_battlefield(&mut game, savannah_lions(), 0);
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Indestructible]), 1);

    cast_final_showdown(&mut game, &[1, 2, 3]);
    let at_resolution = ScriptedDecisionProvider::new();
    picks(&at_resolution, vec![ChoiceOption::Object(bears)]);
    game.resolve_top_of_stack(&at_resolution).expect("it resolves");
    game.perform_sba_and_triggers(&test_dp()).unwrap();

    assert!(game.battlefield.contains_key(&bears), "the creature chosen survives");
    assert!(!game.battlefield.contains_key(&lions));
    assert!(!game.battlefield.contains_key(&theirs), "its indestructible was lost before the wipe");
}

/// `creature` dies: put into its owner's graveyard from the battlefield (CR
/// 700.4).
fn dies(game: &mut GameState, creature: ObjectId) {
    game.change_zone(creature, Zone::Graveyard, ZoneChangeAct::Sacrificed, &test_ctx()).unwrap();
}

/// Walk the turn to player 0's end step, its triggers placed; libraries are
/// filled so a draw on the way is not a loss.
fn to_the_end_step(game: &mut GameState) {
    for p in 0..game.num_players() {
        fill_library(game, p, 10);
    }
    for _ in 0..20 {
        game.advance_turn(&test_dp()).expect("advancing");
        if game.active_player == 0 && game.phase.step == Some(StepType::End) {
            game.perform_sba_and_triggers(&test_dp()).unwrap();
            return;
        }
    }
    panic!("player 0's end step never began");
}

/// Feast of the Victorious Dead on player 0's side with the Bears and the
/// Lions, and two creatures of player 1's: the board and the creatures.
fn feast_board() -> (GameState, ObjectId, ObjectId, [ObjectId; 2]) {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, feast_of_the_victorious_dead(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let lions = put_on_battlefield(&mut game, savannah_lions(), 0);
    let theirs = [put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1), put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1)];
    (game, bears, lions, theirs)
}

/// The trigger announces nothing, and its controller divides the counters
/// as it resolves: three deaths, three life and three counters split two
/// and one.
// RULING: Feast of the Victorious Dead #3 - "The ability doesn't target any creatures. You choose how the counters are distributed as the ability resolves."
#[test]
fn feast_of_the_victorious_dead_divides_its_counters_as_it_resolves() {
    let (mut game, bears, lions, theirs) = feast_board();
    let third = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    theirs.into_iter().chain([third]).for_each(|creature| dies(&mut game, creature));
    to_the_end_step(&mut game);
    let feast = *game.stack.last().expect("Feast triggered");
    assert!(game.stack_entries[&feast].announced.targets().next().is_none(), "no creature is named as it is put on the stack");

    let at_resolution = ScriptedDecisionProvider::new();
    picks(&at_resolution, vec![ChoiceOption::Object(bears), ChoiceOption::Object(lions)]);
    divides(&at_resolution, vec![(ChoiceOption::Object(bears), 1), (ChoiceOption::Object(lions), 2)]);
    game.resolve_top_of_stack(&at_resolution).expect("it resolves");

    assert_eq!(the_asked(&at_resolution), ["SelectRecipients", "DivideAmong"]);
    assert_eq!(game.players[0].life_total, 23);
    assert_eq!((plus_ones(&game, bears), plus_ones(&game, lions)), (1, 2));
}

/// One death before the end step makes it trigger; a second, in response,
/// counts too, since the deaths are counted as it resolves.
// RULING: Feast of the Victorious Dead #1 - "If no creatures have died by the time your end step begins, the ability of Feast of the Victorious Dead won't trigger at all. Provided the ability does trigger, it will count the number of creatures that have died as the ability resolves. In other words, creatures that die in response to the ability will count as long as at least one died previously to have the ability trigger in the first place."
#[test]
fn feast_of_the_victorious_dead_counts_the_deaths_as_it_resolves() {
    let (mut game, bears, lions, theirs) = feast_board();
    dies(&mut game, theirs[0]);
    to_the_end_step(&mut game);
    assert_eq!(game.stack.len(), 1, "one death made it trigger");
    dies(&mut game, theirs[1]);

    let at_resolution = ScriptedDecisionProvider::new();
    picks(&at_resolution, vec![ChoiceOption::Object(lions)]);
    game.resolve_top_of_stack(&at_resolution).expect("it resolves");

    assert_eq!(game.players[0].life_total, 22);
    assert_eq!((plus_ones(&game, bears), plus_ones(&game, lions)), (0, 2), "both counters on the one creature chosen");
}

/// No death, no trigger: the intervening "if" (CR 603.4).
#[test]
fn feast_of_the_victorious_dead_does_not_trigger_without_a_death() {
    let (mut game, _, _, _) = feast_board();
    to_the_end_step(&mut game);
    assert!(game.stack.is_empty());
    assert_eq!(game.players[0].life_total, 20);
}

/// A token that died counts, and so does a card exiled from the graveyard
/// since it died.
// RULING: Feast of the Victorious Dead #2 - "The ability will count any creature that died, including token creatures. For nontoken creatures that died, it doesn't matter if the cards are still in the graveyard."
#[test]
fn feast_of_the_victorious_dead_counts_a_token_and_a_card_no_longer_in_the_graveyard() {
    let (mut game, bears, _, theirs) = feast_board();
    let before = game.battlefield_ids_ordered();
    let token = TokenDef::creature(1, 1, &[Color::White], &[]);
    game.execute_action(GameAction::CreateTokens { defs: vec![token], controller: 1, attacking: None }, &test_ctx()).unwrap();
    let token = game.battlefield_ids_ordered().into_iter().find(|id| !before.contains(id)).expect("the token entered");
    dies(&mut game, token);
    dies(&mut game, theirs[0]);
    game.change_zone(theirs[0], Zone::Exile, ZoneChangeAct::Put, &test_ctx()).unwrap();
    to_the_end_step(&mut game);

    let at_resolution = ScriptedDecisionProvider::new();
    picks(&at_resolution, vec![ChoiceOption::Object(bears)]);
    game.resolve_top_of_stack(&at_resolution).expect("it resolves");
    assert_eq!(game.players[0].life_total, 22, "the token and the exiled card both count");
    assert_eq!(plus_ones(&game, bears), 2);
}
