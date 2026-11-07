//! `mode-rules-and-costs`: the rules of CR 700.2 a plain "choose one" leaves
//! off — "another target" over any target (CR 115.4), each mode targeting a
//! different player, a mode chosen more than once (700.2d), another player
//! choosing (700.2e), pawprints (700.2i), a mode chosen before not offered,
//! and a mode's own cost (700.2h, spree) — `choices-architecture.md`,
//! decisions 2 and 3.
//!
//! **Every board is cast, activated or triggered, never staged**: the modes
//! and targets are questions asked on that path, and a staged entry cannot
//! check them.

use mtgsim::cards::authoring::{any_other_target, any_target, deal_damage, lose_life, spell_ability, target_opponent};
use mtgsim::engine::object_filter::EarlierTargets;
use mtgsim::engine::targeting::{ChosenTargets, FilterIdentity};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::legality::enumerate_legal_selections_excluding;
use mtgsim::cards::printings::grizzly_bears;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_in_hand, put_on_battlefield, setup_game, setup_two_player_game, test_dp};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{Effect, PlayerSet, SelectionFilter};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

use std::sync::Arc;

/// An instant costing {R} with `text`, which `effect` does.
fn invented_instant(name: &'static str, text: &'static str, effect: Effect) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, effect))
        .build()
}

/// The script's next answer: `chosen` for an instance of "target".
fn names_targets(answers: &ScriptedDecisionProvider, chosen: Vec<ChoiceOption>) {
    let kind = ChoiceKind::SelectRecipients { recipient: mtgsim::types::effects::EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED };
    answers.expect_choice(kind, chosen);
}

fn damage_on(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].damage_marked
}

// ---------------------------------------------------------------------------
// "Another target" over any target (item 154's first half)
// ---------------------------------------------------------------------------

/// "Any other target" excludes a *player* the earlier instance took, which
/// the exclusion could not say while it was a leaf of a permanent's filter:
/// with player 1 named first, the second instance offers player 0 and the
/// creature, never player 1 again.
#[test]
fn any_other_target_is_not_the_player_the_first_instance_took() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let first = ChosenTargets::one(vec![ObjectOrPlayerId::Player(1)]);
    let identity = FilterIdentity::for_text_of(ObjectId::UNASSIGNED, EarlierTargets::Chosen(&first));
    let other = SelectionFilter::other_than(SelectionFilter::Any, &[0]);

    let offered = enumerate_legal_selections_excluding(&game, &other, None, 0, identity);

    assert_eq!(offered, [ObjectOrPlayerId::Player(0), ObjectOrPlayerId::Object(bears)]);
}

/// Cast from hand, "2 damage to any target and 1 damage to any other
/// target": player 1 first, the creature second, each dealt its own amount.
#[test]
fn any_other_target_takes_its_own_damage_after_the_first_names_a_player() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let text = "Invented Fork deals 2 damage to any target and 1 damage to any other target.";
    let fork = invented_instant(
        "Invented Fork",
        text,
        Effect::Sequence(vec![deal_damage(any_target(), 2), deal_damage(any_other_target(0), 1)]),
    );
    let spell = put_in_hand(&mut game, fork, 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Player(1)]);
    names_targets(&answers, vec![ChoiceOption::Object(bears)]);
    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    assert_eq!(game.players[1].life_total, 18);
    assert_eq!(damage_on(&game, bears), 1);
}

// ---------------------------------------------------------------------------
// "Target opponent" (item 345)
// ---------------------------------------------------------------------------

/// At three seats "target opponent" offers both opponents and never the
/// caster, and cast from hand it takes the opponent named: player 2 loses
/// the life, player 1 and the caster none.
#[test]
fn target_opponent_offers_each_opponent_and_never_the_caster() {
    let mut game = setup_game(3);
    let opponents = SelectionFilter::Player(PlayerSet::Opponents);
    assert_eq!(
        enumerate_legal_selections_excluding(&game, &opponents, None, 0, FilterIdentity::NONE),
        [ObjectOrPlayerId::Player(1), ObjectOrPlayerId::Player(2)],
    );

    let text = "Target opponent loses 2 life.";
    let drain = invented_instant("Invented Drain", text, lose_life(target_opponent(), 2));
    let spell = put_in_hand(&mut game, drain, 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);
    let answers = ScriptedDecisionProvider::new();
    names_targets(&answers, vec![ChoiceOption::Player(2)]);
    game.cast_spell(0, spell, &ManaWindowStop::new(answers)).expect("castable");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    let life: Vec<i64> = game.players.iter().map(|player| player.life_total).collect();
    assert_eq!(life, [20, 20, 18]);
}
