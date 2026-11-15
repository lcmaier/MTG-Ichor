//! `value-chosen-as-an-effect-resolves` (`choices-architecture.md`), the
//! first of `values-chosen-at-resolution`'s three: a number read off an
//! object and compared, a player verb whose player is gone, and a value or
//! objects chosen as an effect resolves, kept, and used.

use std::sync::Arc;

use mtgsim::cards::authoring::{
    creatures_you_control, draw_cards, power_at_least, power_at_most, put_counters, spell_ability, target_player, toughness_at_least,
    toughness_at_most,
};
use mtgsim::cards::printings::grizzly_bears;
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::layers::types::{ContinuousEffect, EffectId, EffectModification, EffectOrigin, Layer};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::get_effective_colors;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    fill_library, forest, put_in_hand, put_on_battlefield, setup_game, setup_two_player_game, test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, ObjectValue, SelectionFilter, TargetCount, ValueIs,
};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// The boards' tools
// ---------------------------------------------------------------------------

/// A one-mana sorcery of `color` whose text is `text` and whose effect is
/// `effect`: what each fixture below is.
fn sorcery(name: &str, color: (ManaType, Color), text: &'static str, effect: Effect) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[color.0], 0))
        .color(color.1)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(text, effect))
        .build()
}

/// `card` in player 0's hand, cast from a pool of exactly its one mana, with
/// `answers` answering the cast. The spell's id, and the answers back.
fn cast(game: &mut GameState, card: Arc<CardData>, mana: ManaType, answers: ScriptedDecisionProvider) -> (ObjectId, ScriptedDecisionProvider) {
    let id = put_in_hand(game, card, 0);
    game.players[0].mana_pool.add(mana, 1);
    let seat = ManaWindowStop::new(answers);
    game.cast_spell(0, id, &seat).expect("castable from exactly its cost");
    (id, seat.into_inner())
}

/// The script's next answer to a choice of objects or players: `chosen`.
fn picks(answers: &ScriptedDecisionProvider, chosen: Vec<ChoiceOption>) {
    answers.expect_choice(ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED }, chosen);
}

// ---------------------------------------------------------------------------
// A number read off an object, and compared (`ObjectFilter::WithValue`)
// ---------------------------------------------------------------------------

/// Every permanent `filter` describes becomes red, so a test reads the
/// filter's answer for an object off its colors: the layer walk's asker.
fn redden_what_matches(game: &mut GameState, source: ObjectId, filter: ObjectFilter) {
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(ContinuousEffect {
        id: EffectId::UNASSIGNED,
        source,
        origin: EffectOrigin::Resolution,
        layer: Layer::Layer5Color,
        duration: Duration::UntilEndOfTurn,
        controller: 0,
        created_on_turn: 1,
        timestamp,
        affected_objects: ObjectSet::battlefield_filter(filter),
        modification: EffectModification::AddColor(Color::Red),
    });
}

/// Which of `objects` `filter` describes, on a fresh board each time so one
/// filter's answer is never read through another's.
fn matched(filter: ObjectFilter) -> [bool; 4] {
    let mut game = setup_two_player_game();
    let small = put_on_battlefield(&mut game, vanilla_creature(1, 4, &[]), 0);
    let even = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    let big = put_on_battlefield(&mut game, vanilla_creature(5, 2, &[]), 0);
    let land = put_on_battlefield(&mut game, forest(), 0);
    redden_what_matches(&mut game, even, filter);
    [small, even, big, land].map(|id| get_effective_colors(&game, id).contains(&Color::Red))
}

/// Dusk's "power 3 or greater" and Dawn's "power 2 or less", Citywide Bust's
/// "toughness 4 or greater": one filter, the number and the comparison its
/// two parts. Over a 1/4, a 3/3, a 5/2 and a land.
#[test]
fn a_power_or_a_toughness_is_compared_at_least_and_at_most() {
    assert_eq!(matched(power_at_least(3)), [false, true, true, false], "Dusk: power 3 or greater");
    assert_eq!(matched(power_at_most(2)), [true, false, false, false], "Dawn: power 2 or less");
    assert_eq!(matched(toughness_at_least(4)), [true, false, false, false], "Citywide Bust: toughness 4 or greater");
    assert_eq!(matched(toughness_at_most(2)), [false, false, true, false], "toughness 2 or less");
}

/// "Equal to" (Void's shape, over a power) and "less than" (mentor's, CR
/// 702.134a), which an at-least and an at-most pair could not say.
#[test]
fn a_number_is_compared_exactly_and_strictly_less() {
    let power = |is: ValueIs| ObjectFilter::WithValue { of: ObjectValue::Power, is: Box::new(is) };
    assert_eq!(matched(power(ValueIs::Exactly(AmountExpr::Fixed(3)))), [false, true, false, false]);
    assert_eq!(matched(power(ValueIs::LessThan(AmountExpr::Fixed(3)))), [true, false, false, false], "3 is not less than 3");
    assert_eq!(matched(power(ValueIs::LessThan(AmountExpr::Fixed(6)))), [true, true, true, false]);
}

/// CR 208.3: a noncreature has no power or toughness, so it is neither side
/// of a comparison. The land is no "toughness 0 or greater", and no "not
/// toughness 0 or greater" either.
// COVERS-PARTIAL: ATOM-208.3-001
#[test]
fn an_object_with_no_such_number_is_neither_side_of_the_comparison() {
    assert_eq!(matched(toughness_at_least(0)), [true, true, true, false]);
    assert_eq!(matched(ObjectFilter::Not(Box::new(toughness_at_least(0)))), [false, false, false, false]);
}

/// The table compares plain numbers. An amount nobody read first, here X on
/// a board with no X, is a card asking where its site cannot answer: loud,
/// never a silent "no".
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "compares a number with an amount nothing read first")]
fn an_amount_nothing_read_is_refused_loudly() {
    matched(power_at_least(AmountExpr::X));
}

// ---------------------------------------------------------------------------
// A player verb whose player is gone (CR 608.2b, 800.4a)
// ---------------------------------------------------------------------------

/// **Fixture, modeled on Combat Tutorial**: "Target player draws two cards.
/// Put a +1/+1 counter on up to one target creature you control." Two
/// instances of "target" in one spell, the first a player a verb acts on.
fn combat_tutorial_fixture() -> Arc<CardData> {
    let text = "Target player draws two cards. Put a +1/+1 counter on up to one target creature you control.";
    let up_to_one_creature = EffectRecipient::Target(SelectionFilter::Permanent(creatures_you_control()), TargetCount::UpTo(1));
    sorcery(
        "Fixture Combat Tutorial",
        (ManaType::Blue, Color::Blue),
        text,
        Effect::Sequence(vec![draw_cards(target_player(), 2), put_counters(up_to_one_creature, CounterType::PlusOnePlusOne, 1)]),
    )
}

/// CR 608.2b — the spell resolves for the target still legal and "won't
/// affect" the one that is not: with its target player gone from the game,
/// nobody draws. The draw fell to the spell's controller before.
#[test]
fn a_draw_whose_target_player_has_left_is_no_ones() {
    let mut game = setup_game(3);
    for seat in 0..3 {
        fill_library(&mut game, seat, 5);
    }
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let answers = ScriptedDecisionProvider::new();
    picks(&answers, vec![ChoiceOption::Player(1)]);
    picks(&answers, vec![ChoiceOption::Object(bears)]);
    cast(&mut game, combat_tutorial_fixture(), ManaType::Blue, answers);
    game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).expect("player 1 leaves");

    game.resolve_top_of_stack(&test_dp()).expect("it resolves: one target is still legal");
    assert_eq!(game.players[0].hand.len(), 0, "the departed player's draw is no one's");
    assert_eq!(game.battlefield[&bears].counter_count(CounterType::PlusOnePlusOne), 1, "the creature is still affected");
}

// ---------------------------------------------------------------------------
// What a choice of a kind offers
// ---------------------------------------------------------------------------

/// CR 205.3m — the creature types are the rule's list, word for word as the
/// rules file prints it: Human is one, the two-word Time Lord is one, and
/// Forest, a land type, is not a `CreatureType` at all.
#[test]
fn the_creature_types_are_the_ones_the_rule_lists() {
    let rules = include_str!("../../MTG-Rules/versions/tmnt.txt");
    let rule = rules.lines().find(|line| line.starts_with("205.3m ")).expect("the rules file has CR 205.3m");
    let one_word = rule.split("one word long: ").nth(1).expect("the rule lists its one-word types").trim_end_matches('.');
    let mut printed: Vec<String> =
        one_word.split(", ").map(|word| word.trim_start_matches("and ").replace('\u{2019}', "'")).chain(["Time Lord".to_string()]).collect();
    let mut ours: Vec<String> = CreatureType::ALL.iter().map(|creature_type| Subtype::Creature(*creature_type).word()).collect();
    printed.sort();
    ours.sort();
    assert_eq!(ours, printed);
    assert!(ours.contains(&"Human".to_string()) && ours.contains(&"Time Lord".to_string()));
    assert!(!ours.contains(&"Forest".to_string()));
}

/// CR 105.4 — "choose a color" is one of the five; "multicolored" and
/// "colorless" are not colors, and `Color` has no arm to say either.
#[test]
fn the_colors_are_the_five() {
    assert_eq!(Color::ALL, [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]);
}

/// The same spell with its player still here: that player draws, and its
/// controller does not.
#[test]
fn a_draw_goes_to_its_target_player() {
    let mut game = setup_game(3);
    for seat in 0..3 {
        fill_library(&mut game, seat, 5);
    }
    // No creature to target, so "up to one" is none and is not asked.
    let answers = ScriptedDecisionProvider::new();
    picks(&answers, vec![ChoiceOption::Player(1)]);
    cast(&mut game, combat_tutorial_fixture(), ManaType::Blue, answers);

    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(game.players[1].hand.len(), 2);
    assert_eq!(game.players[0].hand.len(), 0);
}
