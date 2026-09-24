//! Phase TR-2a — the histories, the gates and each player
//! (`triggers-architecture.md` §12, TR-2a; §13's TR-2a row).
//!
//! **What a trigger remembers across a turn.** TR-1 matched one window at a
//! time. TR-2a gives the dispatcher a memory: every player's turn,
//! materialized record by record (`TurnSummary`), and the two once-per-turn
//! gates, each written by the instant its rule names. The tests read that
//! memory where the rules read it: at the trigger, at resolution, and a turn
//! later.
//!
//! The fixtures are invented and carry their own names (`engineering-
//! practices.md` §3). The printed cards were verified on Scryfall on
//! 2026-09-24.

use std::sync::Arc;

use mtgsim::cards::authoring::{enters, triggered_ability, whenever};
use mtgsim::cards::basic_lands::forest;
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lg_cards::act_of_treason;
use mtgsim::engine::actions::{DestructionSource, DrawCause, GameAction, LifeLossCause};
use mtgsim::engine::layers::condition::settled_holds;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::DamageTarget;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameState, PhaseType};
use mtgsim::state::history::TurnSummary;
use mtgsim::test_support::{
    creature_with_ability, pass_turn, put_in_hand, put_on_battlefield, setup_two_player_game, stock_libraries, test_ctx,
    test_dp,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Condition, CounterType, Effect, EffectRecipient, ObjectFilter, PlayerRef, PlayerSet, Primitive,
};
use mtgsim::types::history::{CountIs, HistoryCount, TurnFact};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::triggers::{Multiplicity, TriggerEvent, TriggerSubject};
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A 1/1 creature fixture carrying one triggered ability.
fn watcher(name: &str, event: impl Into<TriggerEvent>, effect: Effect) -> Arc<CardData> {
    creature_with_ability(name, 1, 1, triggered_ability(whenever(event, effect)))
}

fn counter_on(recipient: EffectRecipient) -> Effect {
    Effect::Atom(
        Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You },
        recipient,
    )
}

fn place(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).expect("placing");
}

fn resolve_top(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.resolve_top_of_stack(dp).expect("resolving");
}

fn plus_ones(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield.get(&id).map_or(0, |e| e.counter_count(CounterType::PlusOnePlusOne))
}

// ---------------------------------------------------------------------------
// CR 113.7a, 400.7 — "this creature" is the ability's source, by identity
// ---------------------------------------------------------------------------

/// "Put a +1/+1 counter on this creature": the counter goes on the permanent
/// whose ability resolved, not on the ephemeral stack object.
#[test]
fn this_object_is_the_permanent_whose_ability_resolves() {
    let mut game = setup_two_player_game();
    let sapling =
        put_on_battlefield(&mut game, watcher("Self-Tending Sapling", enters(TriggerSubject::ThisObject), counter_on(EffectRecipient::ThisObject)), 0);
    place(&mut game, &test_dp());
    resolve_top(&mut game, &test_dp());
    assert_eq!(plus_ones(&game, sapling), 1);
}

/// CR 400.7 — flickered between its trigger and its resolution, the source
/// is a new object, and "this creature" finds nothing: the returned
/// permanent gets no counter from the first trigger.
#[test]
fn this_object_finds_nothing_once_its_source_has_left_and_returned() {
    let mut game = setup_two_player_game();
    let sapling =
        put_on_battlefield(&mut game, watcher("Self-Tending Sapling", enters(TriggerSubject::ThisObject), counter_on(EffectRecipient::ThisObject)), 0);
    place(&mut game, &test_dp());
    assert_eq!(game.stack.len(), 1);
    game.change_zone(sapling, Zone::Exile, ZoneChangeCause::Exiled, &test_ctx()).unwrap();
    game.change_zone(sapling, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    assert_eq!(game.get_object(sapling).unwrap().zone, Zone::Battlefield, "it came back");

    // The first trigger resolves; the return's own trigger is still pending.
    resolve_top(&mut game, &test_dp());
    assert_eq!(plus_ones(&game, sapling), 0, "the returned permanent is not the object the ability is of");
}

// ---------------------------------------------------------------------------
// §3.10 — the turn summaries: one row per player per turn, one writer
// ---------------------------------------------------------------------------

/// This turn's row for `player`, all zeros when nothing was counted on it.
fn row(game: &GameState, player: PlayerId) -> TurnSummary {
    game.history[player].turn(game.turn_number).cloned().unwrap_or_default()
}

fn this_turn(whose: PlayerSet, fact: TurnFact, is: CountIs) -> Condition {
    Condition::ThisTurn(HistoryCount { whose, fact, is })
}

/// `player` gains control of `victim` the way Act of Treason's resolution
/// gives it (a Layer 2 row), so the thief controls it and the owner does not.
fn steal(game: &mut GameState, victim: ObjectId, player: PlayerId) {
    let treason = put_in_hand(game, act_of_treason(), player);
    let ctx = ResolutionContext {
        source: treason,
        ability_source: None,
        controller: player,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(victim)]),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&act_of_treason().abilities[0].effect, &ctx, &test_dp()).expect("the steal");
}

/// Each record is counted once, on the row of the player it names: the
/// drawer, the player whose life total moved, the player dealt the damage,
/// the controller of the creature that died. The other player's row is
/// untouched, and the leaves sum the rows "you" names.
#[test]
fn each_record_is_counted_on_the_row_of_the_player_it_names() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 5);
    let mine = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let ctx = test_ctx();
    game.execute_action(GameAction::DrawCards { player: 0, n: 2, cause: DrawCause::Effect }, &ctx).unwrap();
    game.execute_action(GameAction::GainLife { player: 0, amount: 2, source: mine }, &ctx).unwrap();
    game.execute_action(GameAction::GainLife { player: 0, amount: 1, source: mine }, &ctx).unwrap();
    game.execute_action(GameAction::LoseLife { player: 0, amount: 2, cause: LifeLossCause::Effect }, &ctx).unwrap();
    let damage = GameAction::DealDamage {
        source: theirs,
        target: DamageTarget::Player(0),
        amount: 3,
        is_combat: false,
        unpreventable: false,
    };
    game.execute_action(damage, &ctx).unwrap();
    game.execute_action(GameAction::Destroy { object: mine, source: DestructionSource::Effect(theirs) }, &ctx).unwrap();

    let expected = TurnSummary {
        cards_drawn: 2,
        life_gained: 3,
        life_gain_events: 2,
        life_lost: 5,
        life_loss_events: 2,
        damage_taken: 3,
        controlled_creatures_died: 1,
        ..TurnSummary::default()
    };
    assert_eq!(row(&game, 0), expected);
    assert_eq!(row(&game, 1), TurnSummary::default());

    // "You" is the source's controller, so P1's creature reads P0 as an opponent.
    let lost = |whose, n| this_turn(whose, TurnFact::LifeLost, CountIs::AtLeast(n));
    assert!(!settled_holds(&lost(PlayerSet::You, 5), &game, theirs));
    assert!(settled_holds(&lost(PlayerSet::Opponents, 5), &game, theirs));
    assert!(!settled_holds(&lost(PlayerSet::Opponents, 6), &game, theirs));
    let untouched = this_turn(PlayerSet::You, TurnFact::DamageTaken, CountIs::AtMost(0));
    assert!(settled_holds(&untouched, &game, theirs));
}

/// A creature stolen and then destroyed died under its thief's control, so
/// the death is on the thief's row and not its owner's.
#[test]
fn a_stolen_creatures_death_is_counted_on_its_controllers_row() {
    let mut game = setup_two_player_game();
    let bear = put_on_battlefield(&mut game, grizzly_bears(), 1);
    steal(&mut game, bear, 0);
    let destroy = GameAction::Destroy { object: bear, source: DestructionSource::Effect(bear) };
    game.execute_action(destroy, &test_ctx()).unwrap();
    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Graveyard);
    assert_eq!(row(&game, 0).controlled_creatures_died, 1, "the thief controlled it as it died");
    assert_eq!(row(&game, 1).controlled_creatures_died, 0, "its owner did not");
}

/// Damage dealt to a permanent is not dealt to its controller (CR 120.3):
/// neither damage nor life on the controller's row.
#[test]
fn damage_to_a_permanent_is_not_damage_to_its_controller() {
    let mut game = setup_two_player_game();
    let bear = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let source = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let damage = GameAction::DealDamage {
        source,
        target: DamageTarget::Object(bear),
        amount: 1,
        is_combat: false,
        unpreventable: false,
    };
    game.execute_action(damage, &test_ctx()).unwrap();
    assert_eq!(row(&game, 0), TurnSummary::default());
}

/// A {1} 1/1 artifact creature fixture.
fn clockwork_mite() -> Arc<CardData> {
    CardDataBuilder::new("Clockwork Mite")
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .mana_cost(ManaCost::build(&[], 1))
        .power_toughness(1, 1)
        .build()
}

/// Cast from hand out of exactly its cost (CR 601.2i): one spell on the
/// caster's row, counted under each of the types it was cast with.
#[test]
fn a_spell_cast_from_hand_counts_once_and_under_each_of_its_types() {
    let mut game = setup_two_player_game();
    let mite = put_in_hand(&mut game, clockwork_mite(), 0);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);
    let dp = ManaWindowStop::new(ScriptedDecisionProvider::new());
    game.cast_spell(0, mite, &dp).expect("cast from an exact pool");
    assert_eq!(game.players[0].mana_pool.total(), 0, "the pool was exactly the cost");

    let row = row(&game, 0);
    assert_eq!(row.spells_cast, 1);
    assert_eq!(row.count(TurnFact::SpellsCastOfType(CardType::Artifact)), 1);
    assert_eq!(row.count(TurnFact::SpellsCastOfType(CardType::Creature)), 1);
    assert_eq!(row.count(TurnFact::SpellsCastOfType(CardType::Instant)), 0);
}

/// CR 103.5's opening hands are drawn before the first turn begins, so no
/// turn holds them; a draw step's draw is its own turn's.
#[test]
fn the_opening_hands_are_drawn_in_no_turn() {
    let deck: Vec<Arc<CardData>> = (0..20).map(|_| forest()).collect();
    let mut game = Game::new(GameConfig::test(), vec![deck.clone(), deck]).unwrap();
    game.setup(&test_dp()).unwrap();
    for player in 0..2 {
        assert_eq!(game.state.history[player].sum(TurnFact::CardsDrawn, 1, game.state.turn_number), 0);
    }
    pass_turn(&mut game.state);
    while game.state.phase.phase_type != PhaseType::Precombat {
        game.state.advance_turn(&test_ctx()).expect("advancing");
    }
    let turn = game.state.turn_number;
    assert_eq!(game.state.history[1].turn(turn).map_or(0, |r| r.cards_drawn), 1, "the second turn's draw step");
}

// ---------------------------------------------------------------------------
// §3.3 — the two arms TR-2a adds: a loss, and a cast
// ---------------------------------------------------------------------------

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

fn pending(game: &GameState) -> usize {
    game.pending_triggers.len()
}

/// "Whenever you lose life": one trigger per loss record (§14 question 3),
/// so two sources' damage in one batch is two; a gain is the other arm's,
/// and another player's loss is not "you".
#[test]
fn loses_life_triggers_once_per_loss_and_never_on_a_gain() {
    let mut game = setup_two_player_game();
    let grudge = TriggerEvent::LosesLife { player: Some(PlayerRef::You), multiplicity: Multiplicity::PerOccurrence };
    let keeper = put_on_battlefield(&mut game, watcher("Grudge Keeper", grudge, counter_on(EffectRecipient::ThisObject)), 0);
    let ctx = test_ctx();
    game.execute_action(GameAction::LoseLife { player: 0, amount: 2, cause: LifeLossCause::Effect }, &ctx).unwrap();
    assert_eq!(pending(&game), 1);
    game.execute_action(GameAction::GainLife { player: 0, amount: 2, source: keeper }, &ctx).unwrap();
    game.execute_action(GameAction::LoseLife { player: 1, amount: 2, cause: LifeLossCause::Effect }, &ctx).unwrap();
    assert_eq!(pending(&game), 1, "a gain, and another player's loss, trigger nothing");

    let a = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let b = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let hit = |source| GameAction::DealDamage {
        source,
        target: DamageTarget::Player(0),
        amount: 1,
        is_combat: true,
        unpreventable: false,
    };
    game.execute_actions(vec![hit(a), hit(b)], &ctx).unwrap();
    assert_eq!(pending(&game), 3, "two sources, two losses (CR 120.3a)");
}

/// "Whenever you cast a creature spell": the filter reads the spell on the
/// stack as its record is dispatched (CR 601.2i), so a creature spell
/// triggers it and a noncreature spell does not.
#[test]
fn casts_spell_reads_the_spell_as_it_is_cast() {
    let mut game = setup_two_player_game();
    let cast = TriggerEvent::CastsSpell { caster: Some(PlayerRef::You), spell: Some(ObjectFilter::ByType(CardType::Creature)) };
    put_on_battlefield(&mut game, watcher("Beast Caller's Drum", cast, gain_one()), 0);
    let dp = ManaWindowStop::new(ScriptedDecisionProvider::new());

    let mite = put_in_hand(&mut game, clockwork_mite(), 0);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);
    game.cast_spell(0, mite, &dp).expect("the artifact creature");
    assert_eq!(pending(&game), 1, "a creature spell");
    place(&mut game, &test_dp());
    while !game.stack.is_empty() {
        resolve_top(&mut game, &test_dp());
    }

    let trinket = put_in_hand(&mut game, clockwork_trinket(), 0);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);
    game.cast_spell(0, trinket, &dp).expect("the artifact");
    assert_eq!(pending(&game), 0, "a noncreature spell triggers nothing");
}

/// A {1} artifact fixture with no abilities.
fn clockwork_trinket() -> Arc<CardData> {
    CardDataBuilder::new("Clockwork Trinket")
        .card_type(CardType::Artifact)
        .mana_cost(ManaCost::build(&[], 1))
        .build()
}
