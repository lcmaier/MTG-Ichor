//! `card-vocabulary-folds`' one new behavior: "target opponent" (item 345).
//!
//! `SelectionFilter::Player` holds a `PlayerSet` (item 342's), so "target
//! opponent" is `Player(PlayerSet::Opponents)`: 742 printed cards say it
//! (`o:"target opponent" game:paper -is:funny`, 2026-10-26), and v1 is
//! four-player Commander. The fixture is three seats, so the caster has two
//! opponents to choose between and is no candidate. It is cast from hand with
//! exactly its cost under `ManaWindowStop`, as a shipped client casts: the
//! cast path is what announces a target (CR 601.2c), which a staged stack
//! entry would not check.

use std::sync::Arc;

use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::oracle::legality::enumerate_legal_selections;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_in_hand, setup_game, test_dp};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, PlayerSet, Primitive, SelectionFilter, TargetCount};
use mtgsim::types::ids::{AbilityId, ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

fn target_opponent() -> SelectionFilter {
    SelectionFilter::Player(PlayerSet::Opponents)
}

/// "{B} Sorcery — Target opponent loses 2 life."
fn opponent_drain() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Opponent Drain")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .card_type(CardType::Sorcery)
        .ability(AbilityDef {
            rules_text: "Target opponent loses 2 life.".into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(Primitive::LoseLife(AmountExpr::Fixed(2)), EffectRecipient::Target(target_opponent(), TargetCount::Exactly(1))),
        })
        .build()
}

/// Cast the drain from `caster`'s hand with exactly `{B}`, its one instance
/// of "target" answered with `aimed_at`.
fn cast_at(game: &mut GameState, caster: PlayerId, aimed_at: PlayerId) -> ObjectId {
    let drain = put_in_hand(game, opponent_drain(), caster);
    game.players[caster].mana_pool.add(ManaType::Black, 1);
    let aim = ScriptedDecisionProvider::new();
    aim.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
        vec![ChoiceOption::Player(aimed_at)],
    );
    game.cast_spell(caster, drain, &ManaWindowStop::new(aim)).expect("castable from exactly its cost");
    drain
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

/// The candidates are the caster's opponents, both of them, and never the
/// caster: the enumeration a decision provider is offered and the check a
/// target is held to answer the same.
#[test]
fn target_opponent_offers_each_opponent_and_not_the_caster() {
    let game = setup_game(3);
    assert_eq!(
        enumerate_legal_selections(&game, &target_opponent(), None, 0),
        vec![ObjectOrPlayerId::Player(1), ObjectOrPlayerId::Player(2)]
    );
    assert_eq!(
        enumerate_legal_selections(&game, &target_opponent(), None, 1),
        vec![ObjectOrPlayerId::Player(0), ObjectOrPlayerId::Player(2)],
        "\"opponent\" is the caster's: player 1 casting it may target player 0"
    );
    assert_eq!(enumerate_legal_selections(&game, &SelectionFilter::Player(PlayerSet::Everyone), None, 0).len(), 3, "the control");
}

/// Cast from hand at three seats: the opponent chosen loses 2 life, and the
/// other opponent and the caster lose none.
#[test]
fn target_opponent_loses_two_life_cast_from_hand() {
    let mut game = setup_game(3);
    let before: Vec<i64> = (0..3).map(|p| life(&game, p)).collect();
    cast_at(&mut game, 0, 2);
    game.resolve_top_of_stack(&test_dp()).expect("resolving");
    assert_eq!((0..3).map(|p| life(&game, p) - before[p]).collect::<Vec<_>>(), vec![0, 0, -2]);
}

/// The caster is not offered as a target, so a provider that answers with
/// them is answering a question it was not asked.
#[test]
#[should_panic(expected = "is not offered")]
fn target_opponent_does_not_offer_the_caster() {
    let mut game = setup_game(3);
    cast_at(&mut game, 0, 0);
}
