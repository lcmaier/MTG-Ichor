//! Phase RE-6 — the game's end.
//!
//! CR 104's two ends as proposals: the state-based check's losses (704.5a–c,
//! 704.6c) as members of the CR 704.3 batch, `Primitive::{LoseGame, WinGame}`
//! for the effect-stated kinds, CR 704.7's collapse of two reasons into one
//! member, CR 704.5b's window closing at the check, CR 104.2a/104.4a settled
//! by the batch, CR 104.1's "immediately", and CR 800.4j at the priority
//! loop — against the four printed cards in `cards::phase_re_cards` and three
//! fixture spells for the primitives.
//!
//! **Every board here is about whether a loss or a win happens**, which is
//! the phase's one axis: replaced (Exquisite Archangel, Stunning Reversal),
//! refused (Platinum Angel), or substituted for a draw (Laboratory Maniac).

use std::sync::Arc;

use mtgsim::cards::basic_lands::forest;
use mtgsim::cards::phase_re_cards::{
    exquisite_archangel, laboratory_maniac, platinum_angel, rhox_faithmender,
    stunning_reversal, thought_reflection,
};
use mtgsim::engine::actions::ZoneChangeCause;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::events::event::{GameEvent, LossReason};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::state::battlefield::AttackTarget;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameResult, GameState, StackEntry};
use mtgsim::test_support::{
    fill_library, put_in_hand, put_on_battlefield, setup_game, test_ctx, test_dp,
    vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::effects::{
    AffectedSet, AmountExpr, Duration, Effect, EffectRecipient, PlayerSet, Primitive,
    SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::replacement::EventPattern;
use mtgsim::types::restriction::{Restriction, RestrictionDef};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A nameless card with no abilities, to be the source of a fixture
/// resolution.
fn fixture_object() -> Arc<CardData> {
    CardDataBuilder::new("Fixture").build()
}

/// Resolve `effect` for `controller` against `targets`, the way a spell would.
fn resolve_with(
    game: &mut GameState,
    controller: PlayerId,
    effect: &Effect,
    targets: Vec<ResolvedTarget>,
    dp: &dyn DecisionProvider,
) {
    let source = put_in_hand(game, fixture_object(), controller);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller,
        targets,
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(effect, &ctx, dp).expect("resolving");
}

/// Resolve a card's spell effect for `controller` with no targets; the card's
/// own id is the resolution's source, as it is for a row the card creates.
fn resolve_card(game: &mut GameState, card: Arc<CardData>, controller: PlayerId) -> ObjectId {
    let id = put_in_hand(game, card.clone(), controller);
    let ctx = ResolutionContext {
        source: id,
        ability_source: None,
        controller,
        targets: vec![],
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).expect("resolving");
    id
}

/// Put `card` on the stack as a cast spell with its real effect, so that
/// `resolve_top_of_stack` runs the card's instructions and then CR 608.2n's
/// trip — `test_support::put_spell_on_stack` stages an empty effect.
fn stage_spell(game: &mut GameState, card: Arc<CardData>, controller: PlayerId) -> ObjectId {
    let effect = card.abilities[0].effect.clone();
    let obj = GameObject::new(card, controller, Zone::Stack);
    let id = obj.id;
    game.add_object(obj);
    game.stack.push(id);
    game.set_stack_entry(StackEntry {
        object_id: id,
        controller,
        chosen_targets: Vec::new(),
        recipient: EffectRecipient::Implicit,
        chosen_modes: Vec::new(),
        x_value: None,
        effect,
        is_spell: true,
        chosen_alternative_cost: None,
        additional_costs_paid: Vec::new(),
        cast_from: Some(Zone::Hand),
        ability_identity: None,
    });
    id
}

/// "Draw a card", as a resolving spell says it.
fn draw_one(game: &mut GameState, player: PlayerId, dp: &dyn DecisionProvider) {
    let effect = Effect::Atom(
        Primitive::DrawCards(AmountExpr::Fixed(1)),
        EffectRecipient::Controller,
    );
    resolve_with(game, player, &effect, vec![], dp);
}

/// "Your life total becomes `n`", as a resolving spell says it.
fn set_life(game: &mut GameState, player: PlayerId, n: u64, dp: &dyn DecisionProvider) {
    let effect = Effect::Atom(
        Primitive::SetLifeTotal(AmountExpr::Fixed(n)),
        EffectRecipient::Controller,
    );
    resolve_with(game, player, &effect, vec![], dp);
}

/// One state-based check; reports whether it did anything (CR 704.3).
fn sba(game: &mut GameState, dp: &dyn DecisionProvider) -> bool {
    game.check_state_based_actions(dp).expect("checking state-based actions")
}

/// Every `PlayerLost` this game has recorded, in order.
fn losses(game: &GameState) -> Vec<(PlayerId, LossReason)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::PlayerLost { player_id, reason } => Some((*player_id, *reason)),
            _ => None,
        })
        .collect()
}

/// Every `PlayerWon` this game has recorded, in order.
fn wins(game: &GameState) -> Vec<PlayerId> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::PlayerWon { player_id } => Some(*player_id),
            _ => None,
        })
        .collect()
}

/// Every `LifeChanged` for `player`, as `(old, new)`.
fn life_changes(game: &GameState, player: PlayerId) -> Vec<(i64, i64)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::LifeChanged { player_id, old, new, .. } if *player_id == player => {
                Some((*old, *new))
            }
            _ => None,
        })
        .collect()
}

fn cards_drawn(game: &GameState, player: PlayerId) -> usize {
    game.events
        .events()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player_id, .. } if *player_id == player))
        .count()
}

fn zone_of(game: &GameState, id: ObjectId) -> Zone {
    game.get_object(id).expect("object exists").zone
}

/// The CR 616.1 prompt over a player-subject event names no object.
const PICK_REPLACEMENT: ChoiceKind = ChoiceKind::ChooseReplacementEffect { affected_object: None };

// ---------------------------------------------------------------------------
// The loss is a proposal — CR 704.5a–c as batch members
// ---------------------------------------------------------------------------

// COVERS: ATOM-104.3b-001
#[test]
fn a_player_at_zero_life_loses_as_a_proposal_the_pipeline_sees() {
    let mut game = setup_game(2);
    game.players[1].life_total = 0;
    let gathers = game.counters.replacement_gathers();

    assert!(sba(&mut game, &test_dp()));

    assert_eq!(losses(&game), vec![(1, LossReason::LifeReachedZero)]);
    assert!(game.player_lost[1]);
    assert_eq!(
        game.counters.replacement_gathers(),
        gathers + 1,
        "one member, one CR 614 gather: the loss went through the pipeline"
    );
    assert_eq!(game.result, Some(GameResult::Winner(0)), "CR 104.2a, settled by the batch");
}

// COVERS: ATOM-104.3c-001
#[test]
fn drawing_from_an_empty_library_loses_at_the_next_check() {
    let mut game = setup_game(2);
    draw_one(&mut game, 0, &test_dp());
    assert_eq!(cards_drawn(&game, 0), 0);
    assert!(game.players[0].has_drawn_from_empty_library);

    assert!(sba(&mut game, &test_dp()));

    assert_eq!(losses(&game), vec![(0, LossReason::DrawnFromEmptyLibrary)]);
    assert_eq!(game.result, Some(GameResult::Winner(1)));
}

// COVERS: ATOM-104.3c-002
#[test]
fn drawing_more_cards_than_the_library_holds_draws_the_rest_and_then_loses() {
    let mut game = setup_game(2);
    fill_library(&mut game, 0, 2);
    let effect = Effect::Atom(
        Primitive::DrawCards(AmountExpr::Fixed(3)),
        EffectRecipient::Controller,
    );
    resolve_with(&mut game, 0, &effect, vec![], &test_dp());

    assert_eq!(cards_drawn(&game, 0), 2, "the two that were there are drawn (CR 104.3c)");
    assert!(game.players[0].has_drawn_from_empty_library);
    assert!(sba(&mut game, &test_dp()));
    assert_eq!(losses(&game), vec![(0, LossReason::DrawnFromEmptyLibrary)]);
}

/// CR 704.7 — a player who would lose for two reasons in one check is one
/// member, carrying the first reason in CR order.
#[test]
fn two_reasons_are_one_loss_carrying_the_first_in_cr_order() {
    let mut game = setup_game(2);
    game.players[0].life_total = 0;
    game.players[0].has_drawn_from_empty_library = true;
    let gathers = game.counters.replacement_gathers();

    assert!(sba(&mut game, &test_dp()));

    assert_eq!(
        losses(&game),
        vec![(0, LossReason::LifeReachedZero)],
        "one loss, and 704.5a comes before 704.5b"
    );
    assert_eq!(game.counters.replacement_gathers(), gathers + 1, "one member, not two");
}

// COVERS: ATOM-104.2a-001
#[test]
fn the_last_player_standing_wins_when_the_batch_settles() {
    let mut game = setup_game(3);
    game.players[1].life_total = 0;
    game.players[2].life_total = -3;

    assert!(sba(&mut game, &test_dp()));

    assert_eq!(losses(&game).len(), 2);
    assert_eq!(game.result, Some(GameResult::Winner(0)));
}

/// Two members of one batch, and the settlement reads the batch: a performer
/// that asked "is anyone left" after the first loss would have crowned the
/// second player. `test_both_players_lose_is_draw` is the `Game`-level twin.
#[test]
fn everyone_losing_in_one_check_is_a_draw_not_a_win_for_the_second_to_perform() {
    let mut game = setup_game(2);
    game.players[0].life_total = 0;
    game.players[1].life_total = 0;

    assert!(sba(&mut game, &test_dp()));

    assert_eq!(losses(&game).len(), 2);
    assert_eq!(game.result, Some(GameResult::Draw));
}

// ---------------------------------------------------------------------------
// CR 104.1 — the game ends immediately
// ---------------------------------------------------------------------------

#[test]
fn the_state_based_check_performs_nothing_in_a_game_that_has_ended() {
    let mut game = setup_game(3);
    game.players[1].life_total = 0;
    game.players[2].life_total = 0;
    assert!(sba(&mut game, &test_dp()));
    assert_eq!(game.result, Some(GameResult::Winner(0)));

    // The winner's own lethal board, one check later: nothing is proposed.
    game.players[0].life_total = 0;
    assert!(!sba(&mut game, &test_dp()));
    assert_eq!(losses(&game).len(), 2);
    assert!(!game.player_lost[0]);
}

#[test]
fn nobody_receives_priority_in_a_game_that_has_ended() {
    let mut game = setup_game(2);
    game.players[1].life_total = 0;
    let dp = RecordingDecisionProvider::picking(0);

    let result = game.run_priority_round(&dp).expect("a round");

    assert_eq!(result, PriorityResult::PhaseEnds);
    assert_eq!(dp.prompts(), 0, "the loss is performed ahead of the first grant, and that is the end");
    assert_eq!(game.result, Some(GameResult::Winner(0)));
}

// ---------------------------------------------------------------------------
// CR 800.4j — the priority rotation passes over a player who has left
// ---------------------------------------------------------------------------

#[test]
fn a_lost_player_is_passed_over_in_the_priority_rotation() {
    let mut game = setup_game(4);
    game.players[2].life_total = 0;
    let dp = RecordingDecisionProvider::picking(0);

    let result = game.run_priority_round(&dp).expect("a round");

    assert_eq!(result, PriorityResult::PhaseEnds);
    assert!(game.player_lost[2]);
    assert_eq!(game.result, None, "three players remain");
    assert_eq!(dp.prompts(), 3, "P0, P1 and P3 pass; P2 is never asked");
}

// COVERS-PARTIAL: ATOM-800.4j-001
//
// The atom's departure is a concession, which no harness offers (CR 104.3a
// is a *leave*, not a proposed loss); the active player here loses to a
// state-based action instead, which is the same departure for CR 800.4j's
// purposes. What is covered is the priority half — "the next player in turn
// order receives priority" — and the turn continuing; the departed player's
// objects leaving the game is RE-7's, and their next turn not beginning is
// RE-1's `a_lost_players_turn_does_not_begin`.
#[test]
fn a_departed_active_players_priority_passes_to_the_next_player_in_turn_order() {
    let mut game = setup_game(4);
    game.players[0].life_total = 0;
    let dp = RecordingDecisionProvider::picking(0);

    let result = game.run_priority_round(&dp).expect("a round");

    assert_eq!(result, PriorityResult::PhaseEnds);
    assert!(game.player_lost[0]);
    assert_eq!(dp.prompts(), 3, "P1, P2, P3 — and the round ends when the three have passed");
    assert_eq!(game.priority_player, 3, "the last player asked was the last in turn order");
    assert_eq!(game.active_player, 0, "the turn is still P0's, continuing without them");
}

/// CR 506.2 — the defending players are the active player's *opponents*, and
/// a player who has left the game is nobody's opponent: a departed seat is not
/// offered as an attack target. Found by the four-player run, where the random
/// agent had been attacking empty seats for a hundred turns.
#[test]
fn a_departed_player_is_not_offered_as_an_attack_target() {
    let mut game = setup_game(4);
    let attacker = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.players[1].life_total = 0;
    game.players[2].life_total = 0;
    assert!(sba(&mut game, &test_dp()));
    assert_eq!(game.result, None, "two seats remain");

    // Takes the first pair offered; with P1 and P2 gone, that is P3.
    let dp = RecordingDecisionProvider::picking(0);
    game.process_declare_attackers(&dp).unwrap();

    let target = game.battlefield[&attacker].attacking.as_ref().map(|a| a.target.clone());
    assert!(
        matches!(target, Some(AttackTarget::Player(3))),
        "the only opponent still in the game, not {target:?}"
    );
}

/// The whole-game path for CR 800.4j: the active player leaves in their own
/// upkeep and the turn runs to the end without them — no draw, no attackers
/// declared for the creature they still have, no cleanup discard — and the
/// next turn is the next player's (CR 800.4k).
#[test]
fn a_departed_active_players_turn_continues_without_them() {
    let deck: Vec<Arc<CardData>> = (0..40).map(|_| forest()).collect();
    let mut g = Game::new(GameConfig::test(), vec![deck.clone(), deck.clone(), deck.clone(), deck])
        .unwrap();
    g.reseed(11);
    g.setup(&test_dp()).unwrap();
    put_on_battlefield(&mut g.state, vanilla_creature(2, 2, &[]), 0);
    let drawn_before = cards_drawn(&g.state, 0);
    let hand_before = g.state.players[0].hand.len();

    // Lost before the first priority grant of the turn.
    g.state.players[0].life_total = 0;
    let dp = RecordingDecisionProvider::picking(0);
    g.run_turn(&dp).unwrap();

    assert!(g.state.player_lost[0]);
    assert_eq!(g.result(), None);
    assert_eq!(cards_drawn(&g.state, 0), drawn_before, "no draw step for a player who has left");
    assert!(
        !dp.kinds().iter().any(|k| k.starts_with("DeclareAttackers")),
        "nobody declares attackers in a turn with no active player"
    );
    assert_eq!(g.state.players[0].hand.len(), hand_before, "and no cleanup discard");
    assert_eq!(g.state.turn_number, 2);
    assert_eq!(g.state.active_player, 1, "CR 800.4k: the next turn is the next player's");
}

// ---------------------------------------------------------------------------
// The primitives — CR 104.3e, 104.2b, 119.5
// ---------------------------------------------------------------------------

// COVERS: ATOM-104.3e-001
#[test]
fn an_effect_may_state_that_a_player_loses_the_game() {
    let mut game = setup_game(2);
    let effect = Effect::Atom(
        Primitive::LoseGame,
        EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
    );
    resolve_with(&mut game, 0, &effect, vec![ResolvedTarget::Player(1)], &test_dp());

    assert!(game.player_lost[1]);
    assert_eq!(losses(&game), vec![(1, LossReason::Effect)]);
    assert_eq!(game.result, Some(GameResult::Winner(0)));
}

// COVERS: ATOM-104.2b-001
#[test]
fn an_effect_may_state_that_a_player_wins_the_game() {
    let mut game = setup_game(2);
    let effect = Effect::Atom(Primitive::WinGame, EffectRecipient::Controller);
    resolve_with(&mut game, 0, &effect, vec![], &test_dp());

    assert_eq!(wins(&game), vec![0]);
    assert_eq!(game.result, Some(GameResult::Winner(0)));
    assert!(!game.player_lost[1], "the opponent did not lose; the game simply ended");
}

// COVERS: ATOM-119.5-001
#[test]
fn setting_a_life_total_lower_is_a_loss_of_the_difference() {
    let mut game = setup_game(2);
    game.players[0].life_total = 15;
    set_life(&mut game, 0, 10, &test_dp());

    assert_eq!(life_changes(&game, 0), vec![(15, 10)]);
    // An effect's loss carries no source; a gain carries the effect's.
    let source = game.events.events().find_map(|e| match e {
        GameEvent::LifeChanged { player_id: 0, source, .. } => Some(*source),
        _ => None,
    });
    assert_eq!(source, Some(None), "it was a `LoseLife`, not a gain");
}

// COVERS: ATOM-119.5-002
#[test]
fn setting_a_life_total_higher_is_a_gain_of_the_difference() {
    let mut game = setup_game(2);
    game.players[0].life_total = 10;
    set_life(&mut game, 0, 20, &test_dp());

    assert_eq!(life_changes(&game, 0), vec![(10, 20)]);
    let source = game.events.events().find_map(|e| match e {
        GameEvent::LifeChanged { player_id: 0, source, .. } => Some(*source),
        _ => None,
    });
    assert!(matches!(source, Some(Some(_))), "it was a `GainLife`, with the effect as its source");
}

#[test]
fn setting_a_life_total_to_what_it_already_is_proposes_nothing() {
    let mut game = setup_game(2);
    game.players[0].life_total = 12;
    let gathers = game.counters.replacement_gathers();
    set_life(&mut game, 0, 12, &test_dp());

    assert!(life_changes(&game, 0).is_empty());
    assert_eq!(game.counters.replacement_gathers(), gathers, "no event, so nothing to replace");
}

/// Rhox Faithmender's ruling: *"becomes 10" from 3 becomes 17* — the gain of
/// 7 is doubled, because CR 119.5 makes it a gain.
#[test]
fn rhox_faithmender_makes_becomes_ten_from_three_seventeen() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, rhox_faithmender(), 0);
    game.players[0].life_total = 3;
    set_life(&mut game, 0, 10, &test_dp());

    assert_eq!(game.players[0].life_total, 17);
}

/// Skullcrack's ruling: *"if an effect says to set a player's life total to a
/// certain number and that number is higher than the player's current life
/// total, that part of the effect won't do anything"* — CR 119.7 refusing the
/// gain CR 119.5 proposes.
#[test]
fn setting_a_life_total_higher_under_a_cant_gain_does_nothing() {
    let mut game = setup_game(2);
    let cant_gain = Effect::Atom(
        Primitive::Restrict(
            RestrictionDef::new(Restriction::Event {
                pattern: EventPattern::GainLife,
                affected_objects: AffectedSet::NO_OBJECTS,
                affected_players: PlayerSet::Everyone,
                by: None,
            }),
            Duration::UntilEndOfTurn,
        ),
        EffectRecipient::Controller,
    );
    resolve_with(&mut game, 1, &cant_gain, vec![], &test_dp());
    game.players[0].life_total = 10;

    set_life(&mut game, 0, 20, &test_dp());
    assert_eq!(game.players[0].life_total, 10, "upward: refused");
    set_life(&mut game, 0, 5, &test_dp());
    assert_eq!(game.players[0].life_total, 5, "downward: a loss, which nothing forbids");
}

// ---------------------------------------------------------------------------
// Exquisite Archangel
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-704.7-001
//
// The atom's board names Lich's Mirror, whose rider shuffles three zones into
// the library and needs a `Primitive::ShuffleIntoLibrary` nothing has built;
// Exquisite Archangel is the printed card with the same shape of effect — one
// "if you would lose the game" static — and what the atom is about is that a
// player at 0 life who also drew from an empty library is *one* loss, replaced
// once. That is proved here with a different rider.
#[test]
fn a_loss_for_two_reasons_is_replaced_once() {
    let mut game = setup_game(2);
    let archangel = put_on_battlefield(&mut game, exquisite_archangel(), 0);
    game.players[0].life_total = 0;
    game.players[0].has_drawn_from_empty_library = true;
    let gathers = game.counters.replacement_gathers();

    assert!(sba(&mut game, &test_dp()), "the check changed the game — the rider ran");

    assert!(losses(&game).is_empty(), "the loss was replaced");
    assert!(!game.player_lost[0]);
    assert_eq!(game.result, None);
    assert_eq!(life_changes(&game, 0), vec![(0, 20)], "one rider, so one life change");
    assert_eq!(zone_of(&game, archangel), Zone::Exile, "and one exile");
    // One member: the gather ran once for the loss and then once for the
    // rider's two proposals (the exile and the gain), never twice for a loss.
    assert_eq!(game.counters.replacement_gathers(), gathers + 3);
}

/// Item 112 — CR 704.5b's window closes at the check that read it. The
/// Archangel's ruling: *"you won't lose again until you try to draw again and
/// still can't do so"*.
#[test]
fn a_replaced_empty_library_loss_is_not_proposed_again_until_the_next_draw() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, exquisite_archangel(), 0);
    game.players[0].life_total = 5;
    draw_one(&mut game, 0, &test_dp());

    assert!(sba(&mut game, &test_dp()));
    assert_eq!(life_changes(&game, 0), vec![(5, 20)], "replaced: life becomes the starting total");
    assert!(!game.players[0].has_drawn_from_empty_library, "the window closed");

    // The next check proposes nothing: the flag was consumed, not left set.
    assert!(!sba(&mut game, &test_dp()));
    assert!(losses(&game).is_empty());
    assert_eq!(life_changes(&game, 0).len(), 1);

    // Trying again, with the Archangel now in exile, loses.
    draw_one(&mut game, 0, &test_dp());
    assert!(sba(&mut game, &test_dp()));
    assert_eq!(losses(&game), vec![(0, LossReason::DrawnFromEmptyLibrary)]);
}

/// The Archangel's first ruling, first half: the loss and the Archangel's own
/// death are two members of one CR 704.3 batch, decided against one board, so
/// the Archangel replaces the loss while it is still there to do so. The
/// second half — *"you choose whether Exquisite Archangel is moved to exile or
/// to your graveyard"* — is not offered: riders run after the batch performs
/// (CR 615.5), the death has happened by then, and the card in the graveyard is
/// a new object (CR 400.7) the rider's exile does not find. The graveyard
/// outcome is taken without the choice — `codebase-state.md` item 125.
#[test]
fn exquisite_archangel_replaces_the_loss_while_dying_in_the_same_check() {
    let mut game = setup_game(2);
    let archangel = put_on_battlefield(&mut game, exquisite_archangel(), 0);
    game.battlefield.get_mut(&archangel).unwrap().damage_marked = 5;
    game.players[0].life_total = -3;

    assert!(sba(&mut game, &test_dp()));

    assert!(losses(&game).is_empty(), "the loss was replaced");
    assert_eq!(life_changes(&game, 0), vec![(-3, 20)]);
    assert_eq!(zone_of(&game, archangel), Zone::Graveyard, "dead to CR 704.5g in the same event");
    assert!(game.exile.is_empty(), "the rider's exile found no such creature");
    assert_eq!(game.result, None);
}

/// The Archangel's second ruling — CR 101.2 ahead of the pipeline, and
/// CR 614.17c leaving a refused event to self-replacements only.
#[test]
fn under_platinum_angel_exquisite_archangel_does_not_apply() {
    let mut game = setup_game(2);
    let archangel = put_on_battlefield(&mut game, exquisite_archangel(), 0);
    put_on_battlefield(&mut game, platinum_angel(), 0);
    game.players[0].life_total = 0;
    let dp = RecordingDecisionProvider::picking(0);

    assert!(!sba(&mut game, &dp), "refused, and a refusal changes nothing");

    assert_eq!(dp.prompts(), 0);
    assert!(life_changes(&game, 0).is_empty(), "the rider never queued");
    assert_eq!(zone_of(&game, archangel), Zone::Battlefield);
    assert_eq!(game.players[0].life_total, 0, "you keep playing, at 0");
    assert!(losses(&game).is_empty());
    assert_eq!(game.result, None);
}

/// The Archangel's third ruling: two printed statics, one CR 616.1 choice —
/// a `Prevent` with a rider is no suppression shape — and the other's effect
/// is not applicable once the loss is gone.
#[test]
fn two_exquisite_archangels_you_choose_which_applies() {
    let mut game = setup_game(2);
    let first = put_on_battlefield(&mut game, exquisite_archangel(), 0);
    let second = put_on_battlefield(&mut game, exquisite_archangel(), 0);
    game.players[0].life_total = -2;
    let dp = ScriptedDecisionProvider::new();
    // Offered in battlefield order; the second is chosen.
    dp.expect_pick_n(PICK_REPLACEMENT, vec![1]);

    assert!(sba(&mut game, &dp));

    assert!(dp.is_empty());
    assert_eq!(zone_of(&game, second), Zone::Exile);
    assert_eq!(zone_of(&game, first), Zone::Battlefield, "the other did not apply");
    assert_eq!(life_changes(&game, 0), vec![(-2, 20)], "one rider");
}

/// The Archangel's fourth ruling, first sentence, and its sixth's loss leg:
/// a poison loss is replaced like any other — `EventPattern::PlayerLoses` has
/// no reason — with 40 life "becoming" 20 as a loss of 20. Then the check
/// repeats (the game changed), the poison is still there, and with the
/// Archangel in exile the player loses.
#[test]
fn exquisite_archangel_applies_to_a_poison_loss_and_then_the_poison_still_loses() {
    let mut game = setup_game(2);
    let archangel = put_on_battlefield(&mut game, exquisite_archangel(), 0);
    game.players[0].life_total = 40;
    game.players[0].poison_counters = 10;

    game.check_state_based_actions_loop(&test_dp()).unwrap();

    assert_eq!(life_changes(&game, 0), vec![(40, 20)]);
    assert_eq!(zone_of(&game, archangel), Zone::Exile);
    assert_eq!(losses(&game), vec![(0, LossReason::PoisonCounters)]);
    assert_eq!(game.result, Some(GameResult::Winner(1)));
}

/// The Archangel's sixth ruling, gain leg: -4 becoming 20 is a gain of 24, and
/// "other cards that interact with life gain ... will interact with this
/// effect accordingly" — Rhox Faithmender makes it 48.
#[test]
fn exquisite_archangel_from_minus_four_is_a_gain_of_twenty_four_that_rhox_faithmender_doubles() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, exquisite_archangel(), 0);
    put_on_battlefield(&mut game, rhox_faithmender(), 0);
    game.players[0].life_total = -4;

    assert!(sba(&mut game, &test_dp()));

    assert_eq!(life_changes(&game, 0), vec![(-4, 44)]);
    assert_eq!(game.result, None);
}

/// "Your starting life total" is the game's, not 20: in a game that started at
/// 40 the Archangel sets 40.
#[test]
fn exquisite_archangel_reads_the_games_starting_life_total() {
    let mut game = GameState::new(2, 40);
    put_on_battlefield(&mut game, exquisite_archangel(), 0);
    game.players[0].life_total = 0;

    assert!(sba(&mut game, &test_dp()));

    assert_eq!(game.players[0].life_total, 40);
}

/// The Archangel's seventh ruling and Stunning Reversal's fourth: an
/// opponent's win is not a loss event. The game ends, the Archangel is
/// untouched, and nothing was proposed for it to see.
#[test]
fn an_opponents_win_is_not_a_loss_exquisite_archangel_can_replace() {
    let mut game = setup_game(2);
    let archangel = put_on_battlefield(&mut game, exquisite_archangel(), 0);
    let effect = Effect::Atom(Primitive::WinGame, EffectRecipient::Controller);
    resolve_with(&mut game, 1, &effect, vec![], &test_dp());

    assert_eq!(game.result, Some(GameResult::Winner(1)));
    assert_eq!(zone_of(&game, archangel), Zone::Battlefield);
    assert!(life_changes(&game, 0).is_empty());
    assert!(losses(&game).is_empty());
}

// ---------------------------------------------------------------------------
// Stunning Reversal
// ---------------------------------------------------------------------------

/// Stunning Reversal's second ruling: the row outlives the card. CR 608.2c's
/// second instruction exiles the spell from the stack, and CR 608.2m lets it
/// finish resolving from there rather than making the graveyard trip.
#[test]
fn stunning_reversal_is_exiled_as_it_resolves_and_its_row_survives_it() {
    let mut game = setup_game(2);
    let spell = stage_spell(&mut game, stunning_reversal(), 0);

    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(zone_of(&game, spell), Zone::Exile);
    assert!(game.players[0].graveyard.is_empty());
    assert!(game.stack.is_empty());
    assert_eq!(game.replacement_effects.iter().count(), 1, "the row is registered");
}

/// Stunning Reversal's seventh ruling, gain leg: from -5, "becomes 1" is a
/// gain of 6 — and the row is `Uses::Once`, so a second loss this turn is a
/// loss.
#[test]
fn stunning_reversal_from_minus_five_is_a_gain_of_six() {
    let mut game = setup_game(2);
    fill_library(&mut game, 0, 10);
    resolve_card(&mut game, stunning_reversal(), 0);
    game.players[0].life_total = -5;

    assert!(sba(&mut game, &test_dp()));

    assert!(losses(&game).is_empty());
    assert_eq!(cards_drawn(&game, 0), 7);
    assert_eq!(life_changes(&game, 0), vec![(-5, 1)]);
    assert_eq!(game.result, None);

    // Once.
    game.players[0].life_total = 0;
    assert!(sba(&mut game, &test_dp()));
    assert_eq!(losses(&game), vec![(0, LossReason::LifeReachedZero)]);
}

/// Stunning Reversal's eighth ruling: *"you'll lose the game immediately
/// after"*. The rider's draws re-arm CR 704.5b, the check repeats because the
/// game changed, and the spent row cannot see the second proposal — no
/// priority window between the two.
#[test]
fn stunning_reversal_with_a_short_library_loses_immediately_after() {
    let mut game = setup_game(2);
    fill_library(&mut game, 0, 3);
    resolve_card(&mut game, stunning_reversal(), 0);
    game.players[0].life_total = 0;

    // The first check performs no member — the loss was replaced — and still
    // reports that it did something, because the rider did.
    assert!(sba(&mut game, &test_dp()));
    assert!(losses(&game).is_empty());
    assert_eq!(cards_drawn(&game, 0), 3);
    assert_eq!(game.players[0].life_total, 1);

    // The repeat is the loss.
    assert!(sba(&mut game, &test_dp()));
    assert_eq!(losses(&game), vec![(0, LossReason::DrawnFromEmptyLibrary)]);
    assert_eq!(game.result, Some(GameResult::Winner(1)));
}

/// Stunning Reversal's third ruling — and the unspent row is still there for
/// a loss the Angel no longer refuses.
#[test]
fn under_platinum_angel_stunning_reversal_neither_applies_nor_is_spent() {
    let mut game = setup_game(2);
    fill_library(&mut game, 0, 10);
    let angel = put_on_battlefield(&mut game, platinum_angel(), 0);
    resolve_card(&mut game, stunning_reversal(), 0);
    game.players[0].life_total = 0;

    assert!(!sba(&mut game, &test_dp()));
    assert_eq!(cards_drawn(&game, 0), 0);
    assert_eq!(game.replacement_effects.iter().count(), 1, "not spent");

    game.change_zone(angel, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx()).unwrap();
    assert!(sba(&mut game, &test_dp()));
    assert_eq!(cards_drawn(&game, 0), 7, "the row applied once the refusal was gone");
    assert_eq!(game.players[0].life_total, 1);
    assert!(losses(&game).is_empty());
}

/// Stunning Reversal's first ruling — the board that decides CR 104.2a is
/// checked per batch. Four `PlayerLoses` members, one replaced: three
/// perform, the batch settles the survivor's win before the rider runs, and
/// the rider's seven draws are the game continuing to be over (CR 104.1).
#[test]
fn stunning_reversal_when_everyone_would_lose_at_once_its_controller_wins() {
    let mut game = setup_game(4);
    fill_library(&mut game, 2, 10);
    resolve_card(&mut game, stunning_reversal(), 2);
    for p in 0..4 {
        game.players[p].life_total = 0;
    }

    assert!(sba(&mut game, &test_dp()));

    assert_eq!(
        losses(&game).iter().map(|(p, _)| *p).collect::<Vec<_>>(),
        vec![0, 1, 3],
        "everyone but the Reversal's controller lost, in batch order"
    );
    assert_eq!(game.result, Some(GameResult::Winner(2)));
    assert_eq!(cards_drawn(&game, 2), 0, "nothing performs after the game has ended");
    assert_eq!(game.players[2].life_total, 0, "not even the rider's life total");
}

// ---------------------------------------------------------------------------
// Platinum Angel
// ---------------------------------------------------------------------------

/// Platinum Angel's first ruling: every state-based reason at once, refused at
/// every check, and the game goes on.
#[test]
fn platinum_angel_refuses_every_state_based_loss_and_the_game_goes_on() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, platinum_angel(), 0);
    game.players[0].life_total = -10;
    game.players[0].poison_counters = 10;
    draw_one(&mut game, 0, &test_dp());
    let queries = game.counters.restriction_queries();

    assert!(!sba(&mut game, &test_dp()));
    assert!(!sba(&mut game, &test_dp()), "and again — you keep playing");

    assert!(losses(&game).is_empty());
    assert!(!game.player_lost[0]);
    assert_eq!(game.result, None);
    assert!(game.counters.restriction_queries() > queries, "the proposal was asked, and refused");
}

#[test]
fn platinum_angel_leaving_the_battlefield_lets_the_next_check_lose() {
    let mut game = setup_game(2);
    let angel = put_on_battlefield(&mut game, platinum_angel(), 0);
    game.players[0].life_total = 0;
    game.players[0].poison_counters = 10;
    assert!(!sba(&mut game, &test_dp()));

    game.change_zone(angel, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx()).unwrap();

    assert!(sba(&mut game, &test_dp()));
    assert_eq!(
        losses(&game),
        vec![(0, LossReason::LifeReachedZero)],
        "one loss, first reason in CR order"
    );
    assert_eq!(game.result, Some(GameResult::Winner(1)));
}

/// Its controller may still win: `PlayerSet::Opponents` is the second row's
/// whole scope.
#[test]
fn platinum_angels_controller_can_still_win() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, platinum_angel(), 0);
    let effect = Effect::Atom(Primitive::WinGame, EffectRecipient::Controller);
    resolve_with(&mut game, 0, &effect, vec![], &test_dp());

    assert_eq!(game.result, Some(GameResult::Winner(0)));
}

// ---------------------------------------------------------------------------
// Laboratory Maniac
// ---------------------------------------------------------------------------

// COVERS: ATOM-614.11-002, ATOM-121.6a-001
#[test]
fn laboratory_maniac_wins_instead_of_drawing_from_an_empty_library() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, laboratory_maniac(), 0);
    draw_one(&mut game, 0, &test_dp());

    assert_eq!(wins(&game), vec![0]);
    assert_eq!(game.result, Some(GameResult::Winner(0)));
    assert_eq!(cards_drawn(&game, 0), 0, "the draw was replaced, not attempted");
    assert!(!game.players[0].has_drawn_from_empty_library, "so CR 704.5b never armed");
    assert!(losses(&game).is_empty());
}

/// The condition is asked at gather (CR 604.2, 614.4): with a card left the
/// Maniac is not a candidate at all, so a Thought Reflection beside it is
/// never a prompt with one live option. The doubled draw's first inner takes
/// the last card and its second is the win.
#[test]
fn thought_reflection_beside_laboratory_maniac_asks_nothing_and_the_second_card_is_the_win() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, laboratory_maniac(), 0);
    put_on_battlefield(&mut game, thought_reflection(), 0);
    fill_library(&mut game, 0, 1);
    let dp = RecordingDecisionProvider::picking(0);

    draw_one(&mut game, 0, &dp);

    assert_eq!(dp.prompts(), 0);
    assert_eq!(cards_drawn(&game, 0), 1);
    assert_eq!(wins(&game), vec![0]);
    assert_eq!(game.result, Some(GameResult::Winner(0)));
}

#[test]
fn laboratory_maniac_does_nothing_while_the_library_has_cards() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, laboratory_maniac(), 0);
    fill_library(&mut game, 0, 2);

    draw_one(&mut game, 0, &test_dp());
    draw_one(&mut game, 0, &test_dp());

    assert_eq!(cards_drawn(&game, 0), 2);
    assert!(wins(&game).is_empty());
    assert_eq!(game.result, None);
}

/// Laboratory Maniac's first ruling: an opponent's Platinum Angel is the
/// printed "can't win". The draw is replaced, the substituted win is refused
/// at the next iteration (CR 101.2), and with nothing left to replace it
/// (CR 614.17c) the event does not happen — no card, no flag, no loss.
#[test]
fn laboratory_maniac_under_an_opponents_platinum_angel_neither_wins_nor_loses() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, laboratory_maniac(), 0);
    put_on_battlefield(&mut game, platinum_angel(), 1);

    draw_one(&mut game, 0, &test_dp());

    assert!(wins(&game).is_empty());
    assert_eq!(cards_drawn(&game, 0), 0);
    assert!(!game.players[0].has_drawn_from_empty_library, "the draw was still replaced");
    assert!(!sba(&mut game, &test_dp()));
    assert!(losses(&game).is_empty());
    assert_eq!(game.result, None);
}

/// Two Maniacs on one draw carry the same rewrite, and `PlayerWins` is
/// instance-invariant and idempotent — the fourth suppression shape, so one
/// outcome and no prompt.
#[test]
fn two_laboratory_maniacs_are_one_outcome_and_no_prompt() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, laboratory_maniac(), 0);
    put_on_battlefield(&mut game, laboratory_maniac(), 0);
    let dp = RecordingDecisionProvider::picking(0);

    draw_one(&mut game, 0, &dp);

    assert_eq!(dp.prompts(), 0);
    assert_eq!(wins(&game), vec![0]);
}

/// CR 104.1 from the winner's side: a three-card instruction with an empty
/// library ends the game at its first inner, and the other two never
/// propose — one `PlayerWon`, not three.
#[test]
fn a_win_ends_the_game_immediately_and_nothing_after_it_performs() {
    let mut game = setup_game(2);
    put_on_battlefield(&mut game, laboratory_maniac(), 0);
    let effect = Effect::Atom(
        Primitive::DrawCards(AmountExpr::Fixed(3)),
        EffectRecipient::Controller,
    );
    resolve_with(&mut game, 0, &effect, vec![], &test_dp());

    assert_eq!(wins(&game), vec![0]);
    assert_eq!(game.result, Some(GameResult::Winner(0)));
}
