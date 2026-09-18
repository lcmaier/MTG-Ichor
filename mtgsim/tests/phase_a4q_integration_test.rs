//! A4q integration test: identical instances of "target" are one question
//! (CR 601.2c).
//!
//! **What CR 601.2c asks is per instance; what the *oracle* has to scan for is
//! per distinct question.** Seeds of Strength prints "target creature" three
//! times with identical criteria and nothing reading an earlier instance, so
//! the three clauses ask the battlefield the same thing three times — and each
//! ask is a `validate_selection` per candidate, which is a layer query. The
//! rule's answer is unchanged; the cost of reaching it is not.
//!
//! **The control is a one-clause spell with the same cost and the same
//! primitive**, so the two castability checks differ in nothing but how many
//! instances they walk. An absolute query count would have to know what the
//! rest of `castable_spells` spends on a board — the mana sources, the timing
//! check — and would then be a number nobody could read; a difference of zero
//! against a one-clause twin is the claim itself. Against the tree before this
//! row the control reads one query and Seeds reads three.
//!
//! The cast that follows is not decoration. `castable_spells` only *offers*,
//! and the fold changes what it scans on the way to offering, so the test
//! closes by casting Seeds from hand through `cast_spell` with exactly its
//! cost in the pool — the announcement loop reached for real, all three
//! instances filled, and the creature 3 points larger for it.

use mtgsim::cards::creatures;
use mtgsim::cards::phase_a4i_cards::seeds_of_strength;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData,
    CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_power, get_effective_toughness};
use mtgsim::oracle::mana_helpers::castable_spells;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_in_hand, put_on_battlefield, setup_two_player_game, test_dp};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pump(recipient: EffectRecipient) -> Effect {
    Effect::Atom(
        Primitive::ModifyPowerToughness(
            AmountExpr::Fixed(1),
            AmountExpr::Fixed(1),
            Duration::UntilEndOfTurn,
        ),
        recipient,
    )
}

fn target_creature() -> EffectRecipient {
    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1))
}

/// Seeds of Strength with two of its three clauses taken away — same cost,
/// same colors, same primitive, one instance of "target".
///
/// A fixture with its own name rather than a printed card: what the control has
/// to match is Seeds' *cost*, since everything `castable_spells` does besides
/// CR 601.2c's loop is a function of that, and no registered one-clause instant
/// costs `{G}{W}`.
fn one_clause_pump() -> Arc<CardData> {
    CardDataBuilder::new("A4q One-Clause Pump")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::White], 0))
        .color(Color::Green)
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text("Target creature gets +1/+1 until end of turn.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: mtgsim::types::ids::AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: pump(target_creature()),
        })
        .build()
}

/// One creature, one card in hand, exactly `{G}{W}` in the pool and no lands.
///
/// No lands on purpose: `find_mana_sources` walks the battlefield, so a land
/// would put layer queries into the delta that have nothing to do with
/// CR 601.2c. With the cost already in the pool there is nothing to look for.
fn board_with(card: Arc<CardData>) -> (GameState, ObjectId, ObjectId) {
    let mut game = setup_two_player_game();
    let bear = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let spell = put_in_hand(&mut game, card, 0);
    game.players[0].mana_pool.add(ManaType::Green, 1);
    game.players[0].mana_pool.add(ManaType::White, 1);
    (game, bear, spell)
}

/// Layer questions the oracle was asked — walks plus memo hits, which is
/// `engineering-practices.md` §3's reading of those two rows together.
fn layer_queries(game: &GameState) -> u64 {
    game.diagnostics.layer_walks() + game.diagnostics.memo_hits()
}

/// What one `castable_spells` costs in layer queries, and what it offered.
fn cost_of_the_castability_check(game: &mut GameState, player: PlayerId) -> (u64, Vec<ObjectId>) {
    let before = layer_queries(game);
    let offered: Vec<ObjectId> = castable_spells(game, player).into_iter().map(|(id, _)| id).collect();
    (layer_queries(game) - before, offered)
}

/// `(power, toughness)` as the layer system computes it.
fn pt(game: &GameState, id: ObjectId) -> (i32, i32) {
    (
        get_effective_power(game, id).expect("has power"),
        get_effective_toughness(game, id).expect("has toughness"),
    )
}

// ---------------------------------------------------------------------------
// CR 601.2c — the same question, asked once
// ---------------------------------------------------------------------------

/// Three identical instances of "target" cost one instance's worth of scanning,
/// and the spell is offered either way.
#[test]
fn seeds_of_strength_scans_the_board_once_for_its_three_identical_clauses() {
    let (mut control, _, one_clause) = board_with(one_clause_pump());
    let (one_clause_queries, control_offered) = cost_of_the_castability_check(&mut control, 0);
    assert_eq!(control_offered, vec![one_clause], "the control is castable");
    assert!(one_clause_queries > 0, "one clause scans the board at least once");

    let (mut game, bear, seeds) = board_with(seeds_of_strength());
    let (seeds_queries, offered) = cost_of_the_castability_check(&mut game, 0);
    assert_eq!(offered, vec![seeds], "three clauses, one creature: still castable");
    assert_eq!(
        seeds_queries, one_clause_queries,
        "three identical clauses are one question (CR 601.2c)"
    );

    // And the announcement loop the check was standing in for still fills all
    // three instances when the spell is actually cast.
    let dp = ManaWindowStop::new(test_dp());
    game.cast_spell(0, seeds, &dp).expect("castable");
    game.resolve_top_of_stack(&dp).expect("resolves");
    assert_eq!(pt(&game, bear), (5, 5), "2/2 plus +1/+1 three times");
}
