//! A6r — the fixes before TR-4 (`roadmap-v2.md` A6r): a draw's place among
//! its player's draws (`codebase-state.md` item 254), who caused a redirected
//! move (item 261), and an unbatched dispatch's error (item 262).

use std::sync::Arc;

use mtgsim::cards::authoring::{enters, triggered_ability, whenever};
use mtgsim::cards::phase_re8_cards::opt;
use mtgsim::cards::phase_re_cards::{notion_thief, teferis_ageless_insight, thought_reflection, words_of_worship};
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::engine::triggers::DISPATCH_NESTING_LIMIT;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{spell_effect, CardData, CardDataBuilder};
use mtgsim::scenario::Scenario;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::{
    creature_with_ability, fill_library, put_in_graveyard, put_in_hand, put_on_battlefield, setup_game, test_ctx, test_dp,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::history::TurnFact;
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::triggers::{Multiplicity, TriggerEvent, TriggerSubject};
use mtgsim::types::zones::{DrawFilter, Zone, ZoneChangeAct};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// The boards
// ---------------------------------------------------------------------------

fn stocked() -> GameState {
    let mut game = setup_game(2);
    for player in 0..2 {
        fill_library(&mut game, player, 30);
    }
    game
}

/// Park the game at `player`'s upkeep, with `player` active.
fn at_upkeep(game: &mut GameState, player: PlayerId) {
    game.active_player = player;
    game.priority_player = player;
    game.set_turn_position(Phase { phase_type: PhaseType::Beginning, step: Some(StepType::Upkeep) });
}

/// Advance until the draw step's turn-based action has happened.
fn take_the_draw_step(game: &mut GameState) {
    let dp = test_dp();
    for _ in 0..8 {
        game.advance_turn(&dp).expect("advancing");
        if game.phase.step == Some(StepType::Draw) {
            return;
        }
    }
    panic!("never reached the draw step");
}

fn resolve_as(game: &mut GameState, source: ObjectId, controller: PlayerId, effect: &Effect, dp: &dyn DecisionProvider) {
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(effect, &ctx, dp).expect("resolving");
}

/// Words of Worship's `{1}`, its row made from the permanent; the mana is
/// skipped, as in RE-3's boards.
fn activate_words(game: &mut GameState, words: ObjectId, player: PlayerId) {
    let effect = words_of_worship().abilities[0].effect.clone();
    resolve_as(game, words, player, &effect, &test_dp());
}

/// Opt for `caster`, keeping the card it looks at on top.
fn resolve_opt(game: &mut GameState, caster: PlayerId) {
    let card = opt();
    let source = put_in_graveyard(game, card.clone(), caster);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::Scry { source: None, n: 0 }, vec![]);
    resolve_as(game, source, caster, &spell_effect(&card.abilities).expect("a spell"), &dp);
}

/// "Draw `n` cards" for `player`, one instruction.
fn draw(game: &mut GameState, player: PlayerId, n: u64) {
    let source = put_in_hand(game, CardDataBuilder::new("Fixture: a draw").build(), player);
    resolve_as(game, source, player, &Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(n)), EffectRecipient::Controller), &test_dp());
}

fn drawn_by(game: &GameState, player: PlayerId) -> usize {
    game.recorded_events()
        .events()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player_id, .. } if *player_id == player))
        .count()
}

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

/// A creature whose one ability triggers "whenever `who` draws" the card at
/// `place`.
fn draw_watcher(name: &str, who: PlayerRef, place: DrawFilter) -> Arc<CardData> {
    let mut ability = triggered_ability(
        "",
        whenever(
            TriggerEvent::DrawsCard { player: Some(who), multiplicity: Multiplicity::PerOccurrence, place: Some(place) },
            gain_one(),
        ),
    );
    ability.id = new_ability_id();
    creature_with_ability(name, 2, 2, ability)
}

/// How many times `source`'s abilities have triggered.
fn triggered(game: &GameState, source: ObjectId) -> usize {
    game.recorded_events()
        .events()
        .filter(|e| matches!(e, GameEvent::AbilityTriggered { origin, .. } if origin.source() == source))
        .count()
}

// ---------------------------------------------------------------------------
// Item 254 — the replacement side counts
// ---------------------------------------------------------------------------

/// Words of Worship takes the draw step's draw, so Opt's card is the first one
/// drawn in that step, and Teferi's Ageless Insight excepts it: one card, as
/// miracle counts "the first card you've drawn this turn" (CR 702.94a).
#[test]
fn teferi_excepts_opts_card_after_words_of_worship_took_the_draw_steps() {
    let mut game = stocked();
    put_on_battlefield(&mut game, teferis_ageless_insight(), 0);
    let words = put_on_battlefield(&mut game, words_of_worship(), 0);
    at_upkeep(&mut game, 0);
    activate_words(&mut game, words, 0);
    let life = game.players[0].life_total;

    take_the_draw_step(&mut game);
    assert_eq!((drawn_by(&game, 0), game.players[0].life_total), (0, life + 5), "the step's draw became 5 life");
    resolve_opt(&mut game, 0);

    assert_eq!(drawn_by(&game, 0), 1, "the first card drawn in the draw step is not doubled");
}

/// The same board under an opponent's Notion Thief: Opt's card is the first
/// its player draws in their draw step, so the Thief leaves it alone.
#[test]
fn notion_thief_leaves_opts_card_after_words_of_worship_took_the_draw_steps() {
    let mut game = stocked();
    put_on_battlefield(&mut game, notion_thief(), 1);
    let words = put_on_battlefield(&mut game, words_of_worship(), 0);
    at_upkeep(&mut game, 0);
    activate_words(&mut game, words, 0);

    take_the_draw_step(&mut game);
    resolve_opt(&mut game, 0);

    assert_eq!((drawn_by(&game, 0), drawn_by(&game, 1)), (1, 0));
}

// ---------------------------------------------------------------------------
// Item 254 — the trigger side asks
// ---------------------------------------------------------------------------

/// Gavi, Nest Warden's "whenever you draw your second card each turn", on a
/// fixture. Its ruling (2020-04-17): the count is the turn's whether or not it
/// was on the battlefield for the first card, and it never triggers on the
/// third or fourth.
#[test]
fn the_turns_second_card_triggers_once_whenever_the_watcher_arrived() {
    let mut game = stocked();
    draw(&mut game, 0, 1);
    let gavi = put_on_battlefield(&mut game, draw_watcher("Fixture: Gavi", PlayerRef::You, DrawFilter::NthThisTurn(2)), 0);

    draw(&mut game, 0, 1);
    assert_eq!(triggered(&game, gavi), 1, "the turn's second card");
    draw(&mut game, 0, 2);

    assert_eq!(triggered(&game, gavi), 1, "not the third or the fourth");
}

/// Two cards drawn in one window — a draw Thought Reflection doubled — are the
/// turn's first and second, each counted as it was drawn.
#[test]
fn a_doubled_draws_second_card_is_the_turns_second() {
    let mut game = stocked();
    put_on_battlefield(&mut game, thought_reflection(), 0);
    let gavi = put_on_battlefield(&mut game, draw_watcher("Fixture: Gavi", PlayerRef::You, DrawFilter::NthThisTurn(2)), 0);

    draw(&mut game, 0, 1);

    assert_eq!((drawn_by(&game, 0), triggered(&game, gavi)), (2, 1));
}

/// Orcish Bowmasters' "whenever an opponent draws a card except the first one
/// they draw in each of their draw steps", on a fixture, under the opponent's
/// registered Thought Reflection: the step's draw becomes two cards, and the
/// second is not the first.
#[test]
fn a_doubled_draw_steps_second_card_is_not_its_first() {
    let mut game = stocked();
    put_on_battlefield(&mut game, thought_reflection(), 0);
    let bowmasters = put_on_battlefield(&mut game, draw_watcher("Fixture: Bowmasters", PlayerRef::Opponent, DrawFilter::NotFirstInDrawStep), 1);
    at_upkeep(&mut game, 0);

    take_the_draw_step(&mut game);
    assert_eq!((drawn_by(&game, 0), triggered(&game, bowmasters)), (2, 1));
    draw(&mut game, 0, 1);

    assert_eq!((drawn_by(&game, 0), triggered(&game, bowmasters)), (4, 3), "a later draw in the step, doubled");
}

/// When the step's own draw is replaced, the next card drawn in it is its
/// first: Wiretapping's "your first card during each of your draw steps"
/// triggers on it, and the Bowmasters fixture waits for the card after.
#[test]
fn the_draw_steps_first_card_is_the_first_drawn_in_it() {
    let mut game = stocked();
    let words = put_on_battlefield(&mut game, words_of_worship(), 0);
    let wiretapping = put_on_battlefield(&mut game, draw_watcher("Fixture: Wiretapping", PlayerRef::You, DrawFilter::FirstInDrawStep), 0);
    let bowmasters = put_on_battlefield(&mut game, draw_watcher("Fixture: Bowmasters", PlayerRef::Opponent, DrawFilter::NotFirstInDrawStep), 1);
    at_upkeep(&mut game, 0);
    activate_words(&mut game, words, 0);

    take_the_draw_step(&mut game);
    resolve_opt(&mut game, 0);
    assert_eq!((triggered(&game, wiretapping), triggered(&game, bowmasters)), (1, 0));
    draw(&mut game, 0, 1);

    assert_eq!((triggered(&game, wiretapping), triggered(&game, bowmasters)), (1, 1));
}

/// Tamiyo, Inquisitive Student's "when you draw your third card in a turn",
/// on a fixture, with Brainstorm's "draw three cards" after the draw step's
/// card: the instruction draws the turn's second, third and fourth, and the
/// trigger is the third's. It is queued at the instruction's close and goes on
/// the stack the next time a player would receive priority, after the spell
/// has finished resolving (CR 603.3).
#[test]
fn a_draw_of_three_after_the_draw_steps_card_holds_the_turns_third() {
    let mut game = stocked();
    let tamiyo = put_on_battlefield(&mut game, draw_watcher("Fixture: Tamiyo", PlayerRef::You, DrawFilter::NthThisTurn(3)), 0);
    at_upkeep(&mut game, 0);
    take_the_draw_step(&mut game);

    draw(&mut game, 0, 3);

    assert_eq!(triggered(&game, tamiyo), 1, "the third card, not the second or fourth");
    let binding = &game.pending_triggers.last().expect("queued, not yet on the stack").binding;
    assert!(matches!(
        binding.triggering_events[..],
        [ref event] if matches!(event.record.event, GameEvent::CardDrawn { .. })
            && event.record.place_in_turn() == Some(3)
    ));
}

// ---------------------------------------------------------------------------
// Item 254 — the save
// ---------------------------------------------------------------------------

/// A board saved in the draw step keeps how many cards were drawn in it, and
/// only the active player's draw step has one.
#[test]
fn the_draw_steps_count_is_a_word_of_the_save() {
    let load = |text: &str| Scenario::parse(text).and_then(|s| s.build(&CardRegistry::default_registry()));
    let game = load("turn 3\nstep draw\nplayer 0: cards drawn this draw step 1\nplayer 0 this turn: cards drawn 2")
        .map(|built| built.game)
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let history = &game.state.players[0].history;
    assert_eq!((history.this_turn(3).count(TurnFact::CardsDrawn), history.this_draw_step()), (2, 1));
    let written = Scenario::write(&game.state).to_string();
    assert!(written.contains("cards drawn this draw step 1"), "{written}");

    for (text, says) in [
        ("turn 3\nstep draw\nplayer 1: cards drawn this draw step 1\nplayer 1 this turn: cards drawn 1", "only during their own"),
        ("turn 3\nplayer 0: cards drawn this draw step 1\nplayer 0 this turn: cards drawn 1", "only during their own"),
        ("turn 3\nstep draw\nplayer 0: cards drawn this draw step 2\nplayer 0 this turn: cards drawn 1", "more cards drawn"),
    ] {
        let refusal = load(text).map(|_| ()).expect_err(text);
        assert!(refusal.message.contains(says), "{text}: {refusal}");
    }
}

// ---------------------------------------------------------------------------
// Item 262 — an unbatched dispatch's error reaches its emitter
// ---------------------------------------------------------------------------

/// A creature whose trigger watches a creature's ability triggering triggers
/// on its own triggering, each `AbilityTriggered` dispatched inside the one
/// that queued it. No printed ability does (`triggers-architecture.md` §4.8),
/// and until TR-6 settles CR 104.4b's draw there, the limit is an `Err`: the
/// move that started the chain is told.
#[test]
fn a_dispatch_past_the_nesting_limit_fails_the_move_that_started_it() {
    let mut game = setup_game(2);
    let mut echo = triggered_ability(
        "",
        whenever(TriggerEvent::AbilityTriggers { caused_by: None, source: Some(ObjectFilter::ByType(CardType::Creature)) }, gain_one()),
    );
    echo.id = new_ability_id();
    put_on_battlefield(&mut game, creature_with_ability("Fixture: echo", 1, 1, echo), 0);
    let mut arrives = triggered_ability("", whenever(enters(TriggerSubject::ThisObject), gain_one()));
    arrives.id = new_ability_id();
    let arriving = put_in_hand(&mut game, creature_with_ability("Fixture: arriving", 1, 1, arrives), 0);

    let entered = game.change_zone(arriving, Zone::Battlefield, ZoneChangeAct::Put, &test_ctx());

    let err = entered.expect_err("the chain reaches the limit");
    assert!(err.contains(&format!("nested {DISPATCH_NESTING_LIMIT} deep")), "{err}");
}
