//! `value-chosen-as-an-effect-resolves` (`choices-architecture.md`), the
//! first of `values-chosen-at-resolution`'s three: a number read off an
//! object and compared, a player verb whose player is gone, and a value or
//! objects chosen as an effect resolves, kept, and used.

use mtgsim::cards::authoring::{power_at_least, power_at_most, toughness_at_least, toughness_at_most};
use mtgsim::engine::layers::types::{ContinuousEffect, EffectId, EffectModification, EffectOrigin, Layer};
use mtgsim::oracle::characteristics::get_effective_colors;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{forest, put_on_battlefield, setup_two_player_game, vanilla_creature};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{AmountExpr, Duration, ObjectFilter, ObjectSet, ObjectValue, ValueIs};
use mtgsim::types::ids::ObjectId;

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
