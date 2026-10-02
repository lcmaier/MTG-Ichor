//! A clone of the state shares each stacked effect rather than copying its
//! tree: `codebase-state.md` item 183's TR-2b note. Its own binary because
//! `support/counting_allocator.rs` is a `#[global_allocator]`, which belongs to
//! one binary.

use std::sync::Arc;

use mtgsim::cards::phase_rs_cards::diabolic_edict;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_in_hand, setup_two_player_game, RecordingDecisionProvider};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount};
use mtgsim::types::ids::AbilityId;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

#[path = "support/counting_allocator.rs"]
mod counting_allocator;
use counting_allocator::clone_cost;

const DEPTH: usize = 8;

/// Diabolic Edict's cost and its one "target player", with an effect that
/// holds nothing on the heap where the edict's `ChosenBy` holds a box and a
/// pick list.
fn heap_free_instant() -> Arc<CardData> {
    CardDataBuilder::new("Heap-Free Instant")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Instant)
        .rules_text("Target player loses 1 life.")
        .ability(AbilityDef {
            rules_text: "",
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::LoseLife(AmountExpr::Fixed(1)),
                EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
            ),
        })
        .build()
}

/// `DEPTH` of `card`, each cast from player 0's hand with exactly its cost.
fn stacked(card: fn() -> Arc<CardData>) -> GameState {
    let mut game = setup_two_player_game();
    let dp = ManaWindowStop::new(RecordingDecisionProvider::picking(0));
    for _ in 0..DEPTH {
        let id = put_in_hand(&mut game, card(), 0);
        game.players[0].mana_pool.add(ManaType::Black, 1);
        game.players[0].mana_pool.add(ManaType::Colorless, 1);
        game.cast_spell(0, id, &dp).expect("castable from exactly its cost");
    }
    assert_eq!(game.stack.len(), DEPTH);
    game
}

#[test]
fn a_clone_shares_each_stacked_effect() {
    let edicts = clone_cost(&stacked(diabolic_edict));
    let heap_free = clone_cost(&stacked(heap_free_instant));
    assert_eq!(edicts, heap_free, "(allocations, bytes): a clone copied the edicts' effect trees");
}
