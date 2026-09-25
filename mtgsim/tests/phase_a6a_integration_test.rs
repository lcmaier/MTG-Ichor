//! The bounded state (`roadmap-v2.md` A6a): no retained event log
//! (`codebase-state.md` item 42) and a history bounded by the table, not the
//! turn count (item 179).
//!
//! A performed record is kept only until the dispatches that read it have
//! returned (`triggers-architecture.md` §4.1). What the rules need from the past
//! is materialized, and these tests pin that from outside the engine: the window
//! is empty wherever a decision is made outside a batch, a trigger reads the
//! records it bound after the window that held them has flushed, and a game that
//! records nothing has no history to read.

use std::cell::Cell;
use std::sync::Arc;

use mtgsim::cards::authoring::{at_beginning_of, dies, triggered_ability, whenever, Whose};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::actions::{DestructionSource, GameAction, LifeLossCause};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::test_support::{
    creature_with_ability, put_in_hand, put_on_battlefield, setup_game, setup_two_player_game, stock_libraries,
    test_ctx, test_dp, RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Condition, Duration, Effect, EffectRecipient, ObjectFilter, PlayerSet, Primitive, SelectionFilter,
    TargetCount,
};
use mtgsim::types::history::{CountIs, HistoryCount, TurnFact};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::triggers::{TriggerCondition, TriggerDef};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::ui::random::RandomDecisionProvider;

// ---------------------------------------------------------------------------
// Item 42 — the window flushes, and nothing that outlives it reads it
// ---------------------------------------------------------------------------

/// A random provider that looks at the window whenever a player is asked for a
/// priority action.
struct WindowWatch {
    inner: RandomDecisionProvider,
    prompts: Cell<u64>,
    held: Cell<u64>,
}

impl DecisionProvider for WindowWatch {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        if matches!(context.kind, ChoiceKind::PriorityAction) {
            self.prompts.set(self.prompts.get() + 1);
            if !game.events.held().is_empty() {
                self.held.set(self.held.get() + 1);
            }
        }
        self.inner.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.inner.pick_number(game, player, context, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.inner.allocate(game, player, context, total, buckets, per_bucket_mins, per_bucket_maxs)
    }

    fn choose_ordering(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize> {
        self.inner.choose_ordering(game, player, context, items)
    }
}

/// Priority is given outside every batch and every dispatch (CR 117.3), so a
/// state forked at any priority prompt holds no record: the window flushed
/// when the last dispatch before the prompt returned. Four seats, every
/// registered card, a dozen turns.
#[test]
fn the_window_holds_nothing_at_any_priority_prompt() {
    let registry = CardRegistry::default_registry();
    let deck: Vec<Arc<CardData>> =
        registry.card_names().iter().cycle().take(60).filter_map(|name| registry.create(name).ok()).collect();
    let mut game = Game::new(GameConfig::test(), vec![deck; 4]).expect("game creation");
    game.reseed(6);
    let watch = WindowWatch { inner: RandomDecisionProvider::seeded(6), prompts: Cell::new(0), held: Cell::new(0) };
    game.setup(&watch).expect("setup");
    for _ in 0..12 {
        if game.is_over() {
            break;
        }
        game.run_turn(&watch).expect("turn");
    }

    assert!(watch.prompts.get() > 50, "only {} priority prompts: the board stopped playing", watch.prompts.get());
    assert_eq!(watch.held.get(), 0, "a priority prompt found records the window had not flushed");
    assert!(game.state.events.held().is_empty(), "nor between turns");
}

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

/// "Whenever a creature dies, you gain life equal to its power": the power is
/// the dead creature's CR 603.10a frame, a 5/5 under a pump, and the trigger
/// reads it off its own copy of the record (§3.4). The window that held the
/// death flushed when its dispatch returned, before the trigger was put on
/// the stack, and the graveyard card is a 2/2.
#[test]
fn a_trigger_reads_the_record_it_bound_after_its_window_has_flushed() {
    let mut game = setup_two_player_game();
    let gains_its_power = Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringPower), EffectRecipient::Controller);
    let mourner = put_on_battlefield(
        &mut game,
        creature_with_ability(
            "Graveside Mourner",
            1,
            1,
            triggered_ability(whenever(dies(ObjectFilter::ByType(CardType::Creature)), gains_its_power)),
        ),
        0,
    );
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    pump(&mut game, bears, 3);

    game.execute_action(GameAction::Destroy { object: bears, source: DestructionSource::Effect(mourner) }, &test_ctx())
        .expect("the destruction");
    assert!(game.events.held().is_empty(), "the death's dispatch has returned and flushed its window");
    assert_eq!(game.pending_triggers.len(), 1);

    let dp = RecordingDecisionProvider::picking(0);
    game.perform_sba_and_triggers(&dp).expect("placing");
    game.resolve_top_of_stack(&dp).expect("resolving");
    assert_eq!(game.players[0].life_total, 25, "the frame's 5 power, not the graveyard card's 2");
}

/// The engine keeps no history of its own, so a game built without a recorder
/// has none to read, and asking is an error rather than an empty answer.
#[test]
#[should_panic(expected = "no event recorder is attached")]
fn a_game_that_records_nothing_has_no_history_to_read() {
    let game = GameState::new(2, 20);
    let _ = game.recorded_events();
}

// ---------------------------------------------------------------------------
// Item 179 — the history is bounded by the table
// ---------------------------------------------------------------------------

/// Walk the turn machinery until `whose` player's `step` begins.
fn advance_to(game: &mut GameState, whose: PlayerId, step: StepType) {
    stock_libraries(game, 10);
    for _ in 0..400 {
        game.advance_turn(&test_ctx()).expect("advancing");
        if game.active_player == whose && game.phase.step == Some(step) {
            return;
        }
    }
    panic!("player {whose}'s {step:?} never began");
}

/// "At the beginning of your upkeep, if you haven't lost life since your last
/// turn, draw a card" (Marchesa, Resolute Monarch's shape) at four seats. Your
/// last turn ends as the next seat's begins, so a loss on the last of the three
/// turns between two of yours falls inside the span, and one on your own turn
/// falls before it.
#[test]
fn since_your_last_turn_spans_the_other_seats_turns() {
    let quiet = Condition::SinceYourLastTurn(HistoryCount {
        whose: PlayerSet::You,
        fact: TurnFact::LifeLost,
        is: CountIs::AtMost(0),
    });
    let vigil = || {
        CardDataBuilder::new("Steady Vigil")
            .card_type(CardType::Enchantment)
            .ability(triggered_ability(TriggerDef {
                condition: TriggerCondition::Event(at_beginning_of(StepType::Upkeep, Whose::Yours)),
                intervening_if: Some(quiet.clone()),
                limit: None,
                effect: Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller),
            }))
            .build()
    };
    let lose = |game: &mut GameState| {
        game.execute_action(GameAction::LoseLife { player: 0, amount: 1, cause: LifeLossCause::Effect }, &test_ctx())
            .expect("the loss");
    };

    let mut game = setup_game(4);
    put_on_battlefield(&mut game, vigil(), 0);
    advance_to(&mut game, 3, StepType::Upkeep);
    lose(&mut game);
    advance_to(&mut game, 0, StepType::Upkeep);
    assert_eq!(game.pending_triggers.len(), 0, "lost on the fourth seat's turn, after player 0's ended");

    let mut game = setup_game(4);
    put_on_battlefield(&mut game, vigil(), 0);
    lose(&mut game);
    advance_to(&mut game, 0, StepType::Upkeep);
    assert_eq!(game.pending_triggers.len(), 1, "lost on player 0's own turn, before the span");
}
