//! A4o integration tests: "counter target spell" may not name an activated
//! ability (CR 112.1, 113.7a, 701.6a).
//!
//! **The board is the one the audit reproduced**, and it is a board a random
//! game reaches: Counterspell and Merfolk Thaumaturgist are both in
//! `PERFORMANCE_POOL`, so every measured game that lines the two up could cast
//! the one at the other's ability. `SelectionFilter::Spell` asked only whether
//! the object was on the stack, and an activated ability on the stack is a
//! `GameObject` carrying a clone of its source's `CardData` — so
//! `Primitive::CounterSpell`'s move to the graveyard put a *second* Merfolk
//! Thaumaturgist in a graveyard while the first stood on the battlefield.
//! Nothing panicked, which is why no fuzz run noticed.
//!
//! The two tests are a pair, on one board, because either alone proves half of
//! it. The first says the ability is never offered; the second says a real
//! spell beside it still is, and is chosen without a prompt — a filter that
//! refused everything would pass the first test and fail the second.
//!
//! Both cast Counterspell from hand through `cast_spell`: the claim is about
//! what CR 601.2c announces and what `castable_spells` offers, and a staged
//! `StackEntry` would answer neither.

use mtgsim::cards::alpha::{counterspell, giant_growth};
use mtgsim::cards::utility_creatures::merfolk_thaumaturgist;
use mtgsim::engine::resolve::ResolvedTarget;
use mtgsim::objects::card_data::AbilityType;
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_power, get_effective_toughness,
};
use mtgsim::oracle::mana_helpers::castable_spells;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_in_hand, put_on_battlefield, setup_two_player_game, test_dp};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Merfolk Thaumaturgist under player 1, with Counterspell and its {U}{U} in
/// player 0's hand and pool.
///
/// Exactly {U}{U} and no lands: the cost is forced (CR 102.2) and the CR 601.2g
/// window has no mana ability to offer, so every prompt these tests see would
/// be a CR 601.2c one — which is the assertion.
fn thaumaturgist_and_a_counterspell(game: &mut GameState) -> (ObjectId, ObjectId) {
    let thaum = put_on_battlefield(game, merfolk_thaumaturgist(), 1);
    let counter = put_in_hand(game, counterspell(), 0);
    game.players[0].mana_pool.add(ManaType::Blue, 2);
    (thaum, counter)
}

/// Activate the Thaumaturgist's `{T}:` ability, by its index in the
/// **effective** ability list (CLAUDE.md's layer-system invariant).
///
/// Its "target creature" is forced: the Thaumaturgist is the only creature on
/// the battlefield in both fixtures, so nothing prompts.
fn activate_the_switch(
    game: &mut GameState,
    controller: PlayerId,
    thaum: ObjectId,
    dp: &ScriptedDecisionProvider,
) -> ObjectId {
    let abilities = get_effective_abilities(game, thaum);
    let idx = abilities
        .iter()
        .position(|a| a.ability_type == AbilityType::Activated)
        .expect("the Thaumaturgist has one activated ability");
    game.activate_ability(controller, thaum, idx, dp)
        .expect("{T} is payable and the target is forced");
    *game.stack.last().expect("the ability is on the stack")
}

/// `(power, toughness)` as the layer system computes it.
fn pt(game: &GameState, id: ObjectId) -> (i32, i32) {
    (
        get_effective_power(game, id).expect("has power"),
        get_effective_toughness(game, id).expect("has toughness"),
    )
}

/// Every object in any graveyard whose card is named `name`.
fn graveyard_copies_of(game: &GameState, name: &str) -> usize {
    game.players
        .iter()
        .flat_map(|p| p.graveyard.iter())
        .filter(|id| {
            game.get_object(**id)
                .map(|o| o.card_data.name == name)
                .unwrap_or(false)
        })
        .count()
}

// ---------------------------------------------------------------------------
// The defect
// ---------------------------------------------------------------------------

#[test]
fn counterspell_is_not_offered_an_activated_ability_on_the_stack() {
    // CR 112.1 — a spell is a *card* on the stack, and CR 113.7a makes an
    // activated ability on the stack something else entirely: an object that
    // is not a card and goes to no zone. Before the filter asked `is_spell`,
    // the ability was the only other object on the stack, so CR 102.2 made the
    // choice forced and the cast went through with no prompt at all.
    let mut game = setup_two_player_game();
    let (thaum, counter) = thaumaturgist_and_a_counterspell(&mut game);
    let dp = test_dp();

    let ability = activate_the_switch(&mut game, 1, thaum, &dp);
    assert!(
        !game.stack_entries[&ability].is_spell,
        "the object on the stack is an ability, not a spell",
    );

    // CR 601.2c, asked by the oracle: with no legal target, the spell is not
    // castable, so the harness is never offered the cast in the first place.
    assert!(
        !castable_spells(&game, 0).iter().any(|(id, _)| *id == counter),
        "an ability on the stack is not a spell to counter, so Counterspell has \
         no legal target and CR 601.2c forbids the cast",
    );

    // And the engine agrees with its own oracle: the cast is refused rather
    // than rewound after a target is chosen (`codebase-state.md` item 139's
    // class is the disagreement, and this is not one).
    let refused = game
        .cast_spell(0, counter, &dp)
        .expect_err("no legal target (CR 601.2c)");
    assert!(
        refused.contains("target"),
        "the refusal names the target rule, got: {}",
        refused,
    );
    assert_eq!(
        game.get_object(counter).expect("still an object").zone,
        Zone::Hand,
        "CR 601.2's rewind puts the card back where it was",
    );
    assert_eq!(game.stack, vec![ability], "the stack is untouched");

    // The ability resolves as it was always going to: 1/2 switched to 2/1, and
    // no card anywhere. The phantom was a second Merfolk Thaumaturgist in
    // player 1's graveyard, put there by `Primitive::CounterSpell`'s
    // `change_zone` on an object whose `CardData` is a clone of its source's.
    game.resolve_top_of_stack(&dp).expect("the ability resolves");
    assert_eq!(pt(&game, thaum), (2, 1), "Layer 7d switched the two (CR 613.4d)");
    assert_eq!(
        graveyard_copies_of(&game, "Merfolk Thaumaturgist"),
        0,
        "the permanent is on the battlefield and nowhere else — an ability is \
         not a card (CR 113.7a)",
    );
    assert!(game.stack.is_empty(), "nothing is left on the stack");
}

// ---------------------------------------------------------------------------
// The half that says the filter narrows rather than refuses
// ---------------------------------------------------------------------------

#[test]
fn counterspell_takes_the_spell_beside_the_ability_without_a_prompt() {
    // The same board, plus a real spell: player 1 activates the Thaumaturgist
    // and then casts Giant Growth at it, so the stack holds one ability and
    // one spell. CR 102.2 makes Counterspell's target forced *again* — but for
    // the opposite reason. One candidate, and it is the spell.
    //
    // A `ScriptedDecisionProvider` with an empty queue panics on the first
    // prompt, which is what makes "no prompt" an assertion rather than a
    // comment: before the fix this board offered two candidates and asked.
    let mut game = setup_two_player_game();
    let (thaum, counter) = thaumaturgist_and_a_counterspell(&mut game);
    let growth = put_in_hand(&mut game, giant_growth(), 1);
    game.players[1].mana_pool.add(ManaType::Green, 1);
    let dp = test_dp();

    let ability = activate_the_switch(&mut game, 1, thaum, &dp);
    game.cast_spell(1, growth, &dp)
        .expect("an instant, and the Thaumaturgist is its only legal target");
    assert_eq!(game.stack, vec![ability, growth], "ability, then spell");

    game.cast_spell(0, counter, &dp)
        .expect("one legal target on the stack, so the choice is forced");
    let chosen = &game.stack_entries[&counter].chosen_targets;
    assert_eq!(chosen.len(), 1, "one instance of \"target\" (CR 601.2c)");
    assert_eq!(
        chosen[0].chosen,
        vec![ResolvedTarget::Object(growth)],
        "the spell, not the ability",
    );

    game.resolve_top_of_stack(&dp).expect("Counterspell resolves");
    assert_eq!(
        game.get_object(growth).expect("still a card").zone,
        Zone::Graveyard,
        "CR 701.6a — a countered spell goes to its owner's graveyard",
    );
    assert_eq!(game.stack, vec![ability], "the ability is untouched");

    game.resolve_top_of_stack(&dp).expect("the ability resolves");
    assert_eq!(
        pt(&game, thaum),
        (2, 1),
        "+3/+3 never happened, and the switch did",
    );
    assert_eq!(
        graveyard_copies_of(&game, "Merfolk Thaumaturgist"),
        0,
        "still no phantom",
    );
}
