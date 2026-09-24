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
use mtgsim::cards::phase_re_cards::alms_collector;
use mtgsim::engine::actions::{DestructionSource, DrawCause, GameAction, LifeLossCause};
use mtgsim::engine::layers::condition::settled_holds;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::{DamageTarget, GameEvent, LossReason};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_power, has_keyword};
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameState, PhaseType};
use mtgsim::state::history::TurnSummary;
use mtgsim::test_support::{
    creature_with_ability, pass_turn, put_in_hand, put_on_battlefield, set_active_player, setup_game,
    setup_two_player_game, stock_libraries, test_ctx, test_dp, RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Condition, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, PlayerSet,
    Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::history::{CountIs, HistoryCount, TurnFact};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::triggers::{
    Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerLimit, TriggerSubject,
};
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::choice_types::ChoiceKind;
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

// ---------------------------------------------------------------------------
// CR 603.2h, §3.5 — the once-per-turn limits, each read where its rule reads it
// ---------------------------------------------------------------------------

fn draw_one() -> Effect {
    Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

fn a_creature() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Creature)
}

/// A triggered ability with a once-per-turn limit.
fn limited(event: impl Into<TriggerEvent>, limit: TriggerLimit, effect: Effect) -> AbilityDef {
    triggered_ability(TriggerDef {
        condition: TriggerCondition::Event(event.into()),
        intervening_if: None,
        limit: Some(limit),
        effect,
    })
}

fn enchantment_with(name: &str, ability: AbilityDef) -> Arc<CardData> {
    CardDataBuilder::new(name).card_type(CardType::Enchantment).ability(ability).build()
}

/// "Whenever a creature enters, draw a card. Do this only once each turn."
fn tollkeepers_ledger() -> Arc<CardData> {
    enchantment_with(
        "Tollkeeper's Ledger",
        limited(enters(a_creature()), TriggerLimit::DoThisOnlyOnceEachTurn, draw_one()),
    )
}

fn hand(game: &GameState, player: PlayerId) -> usize {
    game.players[player].hand.len()
}

/// Resolve everything on the stack and everything that triggers on the way,
/// taking the first option of any prompt and each player's triggers in the
/// order they triggered.
fn drain(game: &mut GameState) {
    let dp = RecordingDecisionProvider::picking(0);
    place(game, &dp);
    while !game.stack.is_empty() {
        resolve_top(game, &dp);
        place(game, &dp);
    }
}

/// CR 603.2h — once the action is taken, the ability no longer triggers that
/// turn; the next turn it does again.
// COVERS: ATOM-603.2h-001
#[test]
fn do_this_only_once_stops_triggering_once_the_action_is_taken() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 10);
    put_on_battlefield(&mut game, tollkeepers_ledger(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(pending(&game), 1);
    drain(&mut game);
    assert_eq!(hand(&game, 0), 1, "the first entry draws");

    put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(pending(&game), 0, "the action was taken this turn: no trigger");

    pass_turn(&mut game);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(pending(&game), 1, "a new turn, a new action");
}

/// CR 603.2h read at resolution: two instances triggered before either
/// resolved, and the second resolves and does nothing. ATOM-603.2h-002's own
/// board is Nykthos Paragon's, whose "may" is TR-2b's.
// COVERS-PARTIAL: ATOM-603.2h-002
#[test]
fn a_second_instance_resolves_and_does_nothing_once_the_action_is_taken() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 10);
    put_on_battlefield(&mut game, tollkeepers_ledger(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(pending(&game), 2, "nothing had resolved, so both triggered");
    drain(&mut game);
    assert_eq!(hand(&game, 0), 1, "only one of the two drew");
}

/// CR 603.2h reads "its source's controller": after P0 took the action, P1
/// steals the source, and P1 has not taken it.
#[test]
fn do_this_only_once_is_the_controllers_gate() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 10);
    let ledger = put_on_battlefield(&mut game, tollkeepers_ledger(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    drain(&mut game);
    assert_eq!(hand(&game, 0), 1);

    steal(&mut game, ledger, 1);
    let before = hand(&game, 1);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    assert_eq!(pending(&game), 1, "P1 has not taken the action this turn");
    drain(&mut game);
    assert_eq!(hand(&game, 1), before + 1, "and P1 draws");
}

/// "Whenever a creature enters, you gain 1 life. This ability triggers only
/// once each turn." The limit counts triggering, not resolving: a second
/// entry while the first trigger waits does not trigger.
fn once_bitten_totem() -> Arc<CardData> {
    enchantment_with(
        "Once-Bitten Totem",
        limited(enters(a_creature()), TriggerLimit::TriggersOnlyOnceEachTurn, gain_one()),
    )
}

#[test]
fn triggers_only_once_each_turn_counts_the_trigger() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 10);
    put_on_battlefield(&mut game, once_bitten_totem(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(pending(&game), 1, "the second entry finds it triggered");
    drain(&mut game);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(pending(&game), 0);

    pass_turn(&mut game);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(pending(&game), 1, "a new turn");
}

/// "For the first time each turn" is the record's place in its turn: two
/// losses in one batch are the first and the second, and a later loss is not
/// the first.
#[test]
fn the_first_time_each_turn_is_the_records_place_in_its_turn() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 10);
    let first_loss =
        TriggerEvent::LosesLife { player: Some(PlayerRef::You), multiplicity: Multiplicity::PerOccurrence };
    let brooder = put_on_battlefield(
        &mut game,
        creature_with_ability(
            "Grudge Brooder",
            1,
            1,
            limited(first_loss, TriggerLimit::FirstTimeEachTurn, counter_on(EffectRecipient::ThisObject)),
        ),
        0,
    );
    let a = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let b = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let hit = |source| GameAction::DealDamage {
        source,
        target: DamageTarget::Player(0),
        amount: 1,
        is_combat: false,
        unpreventable: false,
    };
    game.execute_actions(vec![hit(a), hit(b)], &test_ctx()).unwrap();
    assert_eq!(pending(&game), 1, "two losses, and only the first is the first");
    drain(&mut game);
    assert_eq!(plus_ones(&game, brooder), 1);

    game.execute_action(hit(a), &test_ctx()).unwrap();
    assert_eq!(pending(&game), 0, "the third loss this turn");

    pass_turn(&mut game);
    game.execute_action(hit(a), &test_ctx()).unwrap();
    assert_eq!(pending(&game), 1, "the first loss of a new turn");
}

// ---------------------------------------------------------------------------
// CR 603.7h — how many times this ability has resolved this turn (§6.5)
// ---------------------------------------------------------------------------

/// Ashling the Pilgrim's shape, with a stand-in for its third-time effect
/// (the card needs two amount leaves no phase has built): "{0}: Put a +1/+1
/// counter on this creature. If this is the third time this ability has
/// resolved this turn, you gain 3 life."
fn kindling_pilgrim() -> Arc<CardData> {
    let ability = AbilityDef {
        id: new_ability_id(),
        instances: Vec::new(),
        ability_type: AbilityType::Activated,
        costs: Vec::new(),
        effect: Effect::Sequence(vec![
            counter_on(EffectRecipient::ThisObject),
            Effect::Conditional(
                Condition::ResolvedThisTurn(3),
                Box::new(Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(3)), EffectRecipient::Controller)),
            ),
        ]),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    };
    creature_with_ability("Kindling Pilgrim", 1, 1, ability)
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

/// The count is of resolutions, not activations: two activations on the
/// stack at once are two resolutions as they resolve, the third resolution
/// is the one that gains, and the fourth is past it (Ashling the Pilgrim's
/// second and fourth rulings).
#[test]
fn the_nth_resolution_counts_resolutions_not_activations() {
    let mut game = setup_two_player_game();
    let pilgrim = put_on_battlefield(&mut game, kindling_pilgrim(), 0);
    let dp = RecordingDecisionProvider::picking(0);
    game.activate_ability(0, pilgrim, 0, &dp).expect("first activation");
    game.activate_ability(0, pilgrim, 0, &dp).expect("second, in response");
    drain(&mut game);
    assert_eq!((plus_ones(&game, pilgrim), life(&game, 0)), (2, 20), "two resolutions, no bonus");

    game.activate_ability(0, pilgrim, 0, &dp).expect("third");
    drain(&mut game);
    assert_eq!((plus_ones(&game, pilgrim), life(&game, 0)), (3, 23), "the third resolution");

    game.activate_ability(0, pilgrim, 0, &dp).expect("fourth");
    drain(&mut game);
    assert_eq!((plus_ones(&game, pilgrim), life(&game, 0)), (4, 23), "only the third");
}

/// "It doesn't matter who controlled the creature or the previous abilities
/// when they resolved" (Ashling the Pilgrim's third ruling): two resolutions
/// under P0 and one under the thief make the thief's the third. The count is
/// the ability's, not a row of either player's.
#[test]
fn a_control_change_does_not_restart_the_count() {
    let mut game = setup_two_player_game();
    let pilgrim = put_on_battlefield(&mut game, kindling_pilgrim(), 0);
    let dp = RecordingDecisionProvider::picking(0);
    for _ in 0..2 {
        game.activate_ability(0, pilgrim, 0, &dp).expect("P0 activates");
        drain(&mut game);
    }
    steal(&mut game, pilgrim, 1);
    game.activate_ability(1, pilgrim, 0, &dp).expect("the thief activates");
    drain(&mut game);
    assert_eq!(life(&game, 1), 23, "the third resolution this turn, whoever controlled the first two");
}

// ---------------------------------------------------------------------------
// CR 608.2h — "its power": determined as the effect applies
// ---------------------------------------------------------------------------

/// +`n`/+`n` until end of turn on `id`, the way a resolving pump spell gives it.
fn pump(game: &mut GameState, id: ObjectId, n: u64) {
    let giant_growth = Effect::Atom(
        Primitive::ModifyPowerToughness(AmountExpr::Fixed(n), AmountExpr::Fixed(n), Duration::UntilEndOfTurn),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    let source = put_in_hand(game, grizzly_bears(), 0);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 0,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(id)]),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&giant_growth, &ctx, &test_dp()).expect("the pump");
}

/// "When this creature enters, it deals damage equal to its power to target
/// player": 3 power as it triggers, 6 after a pump in response. The power is
/// the creature's as the effect applies, since it is still on the
/// battlefield (CR 608.2h).
// COVERS: ATOM-608.2h-001
#[test]
fn its_power_is_read_as_the_effect_applies() {
    let mut game = setup_two_player_game();
    let bolt_of_self = Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::TriggeringPower, unpreventable: false },
        EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
    );
    let brute = put_on_battlefield(
        &mut game,
        creature_with_ability(
            "Proud Brute",
            3,
            3,
            triggered_ability(whenever(enters(TriggerSubject::ThisObject), bolt_of_self)),
        ),
        0,
    );
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
            spell_id: ObjectId::UNASSIGNED,
        },
        vec![1],
    );
    place(&mut game, &dp);
    pump(&mut game, brute, 3);
    resolve_top(&mut game, &test_dp());
    assert_eq!(life(&game, 1), 14, "6 damage, not 3");
}

// ---------------------------------------------------------------------------
// CR 611.2c — "creatures you control get ..." fixes its set as it resolves
// ---------------------------------------------------------------------------

/// "Creatures you control get +1/+1 and gain flying until end of turn": the
/// set is the controller's creatures as the effect resolves. The opponent's
/// creature is not in it, and neither is a creature that arrives afterwards
/// (CR 611.2c).
#[test]
fn a_filtered_one_shot_applies_to_the_permanents_it_matched_as_it_resolved() {
    let mut game = setup_two_player_game();
    let mine = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let yours = EffectRecipient::FilteredPermanents(ObjectFilter::And(
        Box::new(a_creature()),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    ));
    let rally = Effect::Sequence(vec![
        Effect::Atom(
            Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(1), Duration::UntilEndOfTurn),
            yours.clone(),
        ),
        Effect::Atom(Primitive::GrantKeywordFlag(KeywordFlag::Flying, Duration::UntilEndOfTurn), yours),
    ]);
    let source = put_in_hand(&mut game, grizzly_bears(), 0);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 0,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&rally, &ctx, &test_dp()).expect("the rally");
    let late = put_on_battlefield(&mut game, grizzly_bears(), 0);

    assert_eq!(get_effective_power(&game, mine), Some(3));
    assert!(has_keyword(&game, mine, KeywordFlag::Flying));
    assert_eq!(get_effective_power(&game, theirs), Some(2), "not the controller's");
    assert!(!has_keyword(&game, theirs, KeywordFlag::Flying));
    assert_eq!(get_effective_power(&game, late), Some(2), "arrived after the set was fixed");
    assert!(!has_keyword(&game, late, KeywordFlag::Flying));
}

// ---------------------------------------------------------------------------
// CR 121.2c — several players drawing: the active player first (item 122)
// ---------------------------------------------------------------------------

/// Who drew, in the order the log recorded it.
fn draw_order(game: &GameState) -> Vec<PlayerId> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::CardDrawn { player_id, .. } => Some(*player_id),
            _ => None,
        })
        .collect()
}

/// `player` resolves "draw `n` cards" as one instruction.
fn draw_instruction(game: &mut GameState, player: PlayerId, n: u64) {
    let draw = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(n)), EffectRecipient::Controller);
    let source = put_in_hand(game, grizzly_bears(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&draw, &ctx, &test_dp()).expect("the draw");
}

/// Alms Collector's "instead you and that player each draw a card" is one
/// instruction to two players, and CR 121.2c has the active player draw
/// first, whichever of the two that is.
#[test]
fn alms_collectors_two_draws_come_out_active_player_first() {
    // The Collector's controller is active.
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 5);
    put_on_battlefield(&mut game, alms_collector(), 0);
    draw_instruction(&mut game, 1, 2);
    assert_eq!(draw_order(&game), vec![0, 1]);

    // The affected player is active.
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 5);
    set_active_player(&mut game, 1);
    put_on_battlefield(&mut game, alms_collector(), 0);
    draw_instruction(&mut game, 1, 2);
    assert_eq!(draw_order(&game), vec![1, 0]);
}

/// "Each player draws a card" at four seats: the active player, then the
/// rest in turn order, and a player who has left the game draws nothing
/// (CR 101.4, 121.2c, 800.4a).
#[test]
fn each_player_draws_in_apnap_order_over_the_seats_still_in_the_game() {
    let mut game = setup_game(4);
    stock_libraries(&mut game, 5);
    set_active_player(&mut game, 2);
    game.execute_action(GameAction::PlayerLoses { player: 3, reason: LossReason::Effect }, &test_ctx())
        .unwrap();
    let each_draws = Effect::Atom(
        Primitive::DrawCards(AmountExpr::Fixed(1)),
        EffectRecipient::EachPlayer(PlayerSet::Everyone),
    );
    let source = put_in_hand(&mut game, grizzly_bears(), 0);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 0,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&each_draws, &ctx, &test_dp()).expect("the draws");
    assert_eq!(draw_order(&game), vec![2, 0, 1]);
}

/// The atom's own board: "each player draws 3 cards" with P0 active. P0
/// performs all three draws, then P1 (CR 121.2c).
// COVERS: ATOM-121.2c-001
#[test]
fn each_player_draws_three_and_the_active_player_draws_all_of_theirs_first() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 5);
    let each_draws_three = Effect::Atom(
        Primitive::DrawCards(AmountExpr::Fixed(3)),
        EffectRecipient::EachPlayer(PlayerSet::Everyone),
    );
    let source = put_in_hand(&mut game, grizzly_bears(), 1);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 1,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&each_draws_three, &ctx, &test_dp()).expect("the draws");
    assert_eq!(draw_order(&game), vec![0, 0, 0, 1, 1, 1]);
}
