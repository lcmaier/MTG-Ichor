//! Phase TR-3a — the delayed-trigger registry, with Final Fortune
//! (`triggers-architecture.md` §3.9, §4.6, §12).
//!
//! 1. Item 222: a copy or grant row that names its objects puts only those
//!    objects in front of the dispatcher, and only for a window of a kind
//!    their triggered abilities read.
//! 2. The turn queue names its entries: an extra turn carries the id of the
//!    entry it came from (§3.9's amendment), which "that turn" reads.
//! 3. The registry: a resolution creates a delayed trigger (CR 603.7), which
//!    is never retroactive (603.7a), triggers once or for its duration
//!    (603.7b), and finds its object by identity (603.7c, 400.7).
//! 4. Provenance (CR 603.7d–g): a spell's, an ability's, a replacement's, and
//!    a special action's. The last is a fixture, since nothing lets a player
//!    take such an action yet (`backlog.md` §2.8).
//! 5. CR 603.7h's count, 107.3n's X, and 513.2's step that does not back up.
//!
//! Most boards resolve their spell by hand (`resolve_spell`): the question is
//! the resolution's. The X spell is cast from hand from exactly its cost under
//! `ManaWindowStop`, the path a shipped client takes, and so is Final Fortune
//! in `phase_tr3a_cards`' tests.

use std::sync::Arc;

use mtgsim::cards::authoring::{at_beginning_of, dies, enters, leaves_the_battlefield, triggered_ability, whenever, Whose};
use mtgsim::cards::phase_re_cards::time_walk;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, Layer};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::events::event::CounterSubject;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::is_creature;
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::test_support::{
    forest, pass_turn, put_in_hand, put_on_battlefield, registered, setup_two_player_game, stock_libraries, test_ctx,
    test_dp, vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Condition, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerGroup,
    PlayerRef, PlayerSet, Primitive, SelectionFilter, TargetCount, TokenDef,
};
use mtgsim::types::ids::{new_ability_id, AbilityId, ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaSymbol, ManaType};
use mtgsim::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use mtgsim::types::triggers::{
    DelayedDuration, DelayedProvenance, DelayedTriggerTemplate, DelayedTurn, IdentityRef, TriggerEvent, TriggerOrigin,
    TriggerSubject, TriggerTurn,
};
use mtgsim::types::zones::{DestructionSource, Zone, ZoneChangeCause};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn gain(n: u64) -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(n)), EffectRecipient::Controller)
}

fn gain_one() -> Effect {
    gain(1)
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

fn a_creature() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Creature)
}

/// A delayed trigger as a card prints one: "[when event], [effect]", in any
/// turn.
fn template(event: impl Into<TriggerEvent>, effect: Effect, duration: DelayedDuration) -> DelayedTriggerTemplate {
    DelayedTriggerTemplate { def: Arc::new(whenever(event, effect)), duration, turn: DelayedTurn::Any }
}

/// The instruction that creates `template`.
fn create(template: DelayedTriggerTemplate) -> Effect {
    Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(template)), EffectRecipient::Controller)
}

/// "At the beginning of the next end step, [effect]."
fn at_the_next_end_step(effect: Effect) -> Effect {
    create(template(at_beginning_of(StepType::End, Whose::Each), effect, DelayedDuration::Once))
}

/// An instant whose one spell ability is `effect`.
fn instant(name: &str, cost: ManaCost, effect: Effect) -> Arc<CardData> {
    CardDataBuilder::new(name).mana_cost(cost).card_type(CardType::Instant).ability(spell(effect)).build()
}

fn spell(effect: Effect) -> AbilityDef {
    AbilityDef {
        rules_text: "".into(),
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Spell,
        costs: Vec::new(),
        effect,
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    }
}

/// "{0}: [effect]", or "{T}: [effect]".
fn activated(tap: bool, effect: Effect) -> AbilityDef {
    AbilityDef {
        rules_text: "".into(),
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Activated,
        costs: if tap { vec![mtgsim::types::costs::Cost::TapSelf] } else { Vec::new() },
        effect,
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    }
}

/// A 2/2 creature with `abilities`.
fn creature(name: &str, abilities: Vec<AbilityDef>) -> Arc<CardData> {
    let mut builder = CardDataBuilder::new(name).card_type(CardType::Creature).power_toughness(2, 2);
    for ability in abilities {
        builder = builder.ability(ability);
    }
    builder.build()
}

/// A token creature of `power`, with `abilities`.
fn token(power: i32, abilities: Vec<AbilityDef>) -> TokenDef {
    TokenDef {
        name: None,
        colors: Vec::new(),
        types: vec![CardType::Creature],
        subtypes: Vec::new(),
        supertypes: Vec::new(),
        power: Some(power),
        toughness: Some(power),
        keyword_flags: Vec::new(),
        abilities,
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
    }
}

/// Create one token for `controller`: an entry the dispatcher sees.
fn create_token(game: &mut GameState, controller: PlayerId, def: TokenDef) -> ObjectId {
    let before: Vec<ObjectId> = game.battlefield_ids_ordered();
    game.execute_action(GameAction::CreateTokens { defs: vec![def], controller }, &test_ctx()).unwrap();
    game.battlefield_ids_ordered().into_iter().find(|id| !before.contains(id)).expect("the token entered")
}

/// Resolve `card`'s spell effect for `controller`, the way the stack would,
/// with nothing targeted; the card's own id is the resolution's source.
fn resolve_spell(game: &mut GameState, card: Arc<CardData>, controller: PlayerId) -> ObjectId {
    let id = put_in_hand(game, card.clone(), controller);
    let ctx = ResolutionContext {
        source: id,
        ability_source: None,
        controller,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
    id
}

/// Activate `source`'s ability `index` for `player` and resolve it.
fn activate_and_resolve(game: &mut GameState, player: PlayerId, source: ObjectId, index: usize) {
    game.activate_ability(player, source, index, &test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
}

/// Put every waiting trigger on the stack and resolve the stack, taking the
/// first option at any prompt on the way (CR 603.3b's order among them).
fn place_and_resolve(game: &mut GameState) {
    let dp = RecordingDecisionProvider::picking(0);
    game.perform_sba_and_triggers(&dp).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&dp).unwrap();
        game.perform_sba_and_triggers(&dp).unwrap();
    }
}

/// Walk the turn machinery until `whose` player's `step` begins. Libraries are
/// filled so a draw step on the way is not a loss.
fn advance_to(game: &mut GameState, whose: PlayerId, step: StepType) {
    for p in 0..game.num_players() {
        if game.players[p].library.len() < 5 {
            mtgsim::test_support::fill_library(game, p, 10);
        }
    }
    for _ in 0..200 {
        game.advance_turn(&test_ctx()).expect("advancing");
        if game.active_player == whose && game.phase.step == Some(step) {
            return;
        }
    }
    panic!("player {whose}'s {step:?} never began");
}

/// Give `object` to `player` with a control-changing row (CR 613.2).
fn give_control(game: &mut GameState, object: ObjectId, player: PlayerId) {
    game.continuous_effects.add(ContinuousEffect {
        duration: Duration::Indefinite,
        ..registered(object, Layer::Layer2Control, 200, EffectModification::SetController(PlayerRef::Player(player)))
    });
}

/// The dispatcher's two work counters: windows past its gate, and the
/// candidates those windows asked.
fn dispatch_work(game: &GameState) -> (u64, u64) {
    let work = game.diagnostics.trigger_dispatch();
    (work.windows, work.candidates)
}

/// The one delayed trigger waiting to go on the stack.
fn the_delayed_pending(game: &GameState) -> &mtgsim::types::triggers::PendingTrigger {
    assert_eq!(game.pending_triggers.len(), 1, "one trigger waiting");
    let pending = &game.pending_triggers[0];
    assert!(matches!(pending.origin, TriggerOrigin::Delayed(_)), "a delayed trigger's");
    pending
}

// ---------------------------------------------------------------------------
// 1. Item 222
// ---------------------------------------------------------------------------

/// Item 222: a grant to one creature of "whenever this creature becomes
/// tapped" opened the gate for every permanent, at every window. The row
/// names its object, so the dispatcher reads that object off the row, and
/// only for a window carrying a kind the granted trigger reads: an untap
/// passes no gate, and the creature's own tap asks it alone of the eight
/// permanents. The grant still triggers.
#[test]
fn a_named_grant_asks_only_its_object_and_only_for_its_kinds() {
    let mut game = setup_two_player_game();
    let carrier = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    for _ in 0..6 {
        put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    }
    let land = put_on_battlefield(&mut game, forest(), 0);
    let mut granted = triggered_ability(
        "Whenever this creature becomes tapped, you gain 1 life.",
        whenever(TriggerEvent::BecomesTapped { subject: TriggerSubject::ThisObject }, gain_one()),
    );
    granted.id = new_ability_id();
    game.continuous_effects.add(ContinuousEffect {
        duration: Duration::Indefinite,
        affected_objects: ObjectSet::Fixed(vec![carrier]),
        ..registered(carrier, Layer::Layer6Ability, 100, EffectModification::GrantAbility(Arc::new(granted)))
    });
    game.execute_action(GameAction::Tap { object: land }, &test_ctx()).unwrap();
    let before = dispatch_work(&game);

    game.execute_action(GameAction::Untap { object: land }, &test_ctx()).unwrap();
    assert_eq!(dispatch_work(&game), before, "an untap is no kind the granted trigger reads");

    game.execute_action(GameAction::Tap { object: carrier }, &test_ctx()).unwrap();
    let after = dispatch_work(&game);
    assert_eq!((after.0 - before.0, after.1 - before.1), (1, 1), "one window, one candidate: the carrier");
    assert_eq!(game.pending_triggers.len(), 1, "and the grant triggered");
    assert_eq!(game.pending_triggers[0].origin.source(), carrier);
}

// ---------------------------------------------------------------------------
// 2. The turn queue's ids
// ---------------------------------------------------------------------------

/// CR 500.7, with the id §3.9's amendment puts on the proposal: Time Walk's
/// extra turn is the queue entry it made, the turn that begins from it says
/// which, and the natural turn after it says it is none.
#[test]
fn an_extra_turn_carries_the_queue_entry_it_came_from() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 10);
    resolve_spell(&mut game, time_walk(), 0);
    assert_eq!(game.turn_queue.len(), 1);
    let queued = game.turn_queue[0];
    assert_eq!((queued.player, game.extra_turn), (0, None), "a natural turn names no entry");

    pass_turn(&mut game);
    assert_eq!((game.active_player, game.turn_number), (0, 2));
    assert_eq!(game.extra_turn, Some(queued.id), "the extra turn is the entry it came from");

    pass_turn(&mut game);
    assert_eq!((game.active_player, game.turn_number), (1, 3));
    assert_eq!(game.extra_turn, None, "and the natural turn after it is none");
}

// ---------------------------------------------------------------------------
// 3. The registry
// ---------------------------------------------------------------------------

/// CR 603.7: a resolving spell creates "at the beginning of the next end step,
/// destroy target creature". The spell targets nothing: the delayed trigger's
/// target is its own, chosen as it goes on the stack (CR 603.3d). The
/// registry holds it until the end step begins, it triggers then and leaves,
/// and it destroys the creature.
// COVERS: ATOM-603.7-001
#[test]
fn a_spell_creates_a_delayed_trigger_that_fires_at_the_next_end_step() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let doom = instant(
        "Delayed Doom",
        ManaCost::build(&[ManaType::Black], 0),
        at_the_next_end_step(Effect::Atom(
            Primitive::Destroy,
            EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
        )),
    );
    assert!(doom.spell_instances.is_empty(), "the delayed trigger's target is not the spell's");
    resolve_spell(&mut game, doom, 0);
    assert_eq!(game.delayed_triggers.len(), 1, "registered as the spell resolved");
    assert!(game.pending_triggers.is_empty(), "and nothing has happened");

    advance_to(&mut game, 0, StepType::End);
    the_delayed_pending(&game);
    assert!(game.delayed_triggers.is_empty(), "it triggered, once, and is gone");
    place_and_resolve(&mut game);
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Graveyard);
}

/// "{T}: When this creature becomes untapped, you gain 2 life" and "{0}: When
/// this creature leaves the battlefield, you gain 3 life": CR 603.7a's two
/// examples, as one creature's abilities.
fn watchful_sentinel() -> Arc<CardData> {
    creature(
        "Watchful Sentinel",
        vec![
            activated(
                true,
                create(template(
                    TriggerEvent::BecomesUntapped { subject: TriggerSubject::ThisObject },
                    gain(2),
                    DelayedDuration::Once,
                )),
            ),
            activated(false, create(template(leaves_the_battlefield(TriggerSubject::ThisObject), gain(3), DelayedDuration::Once))),
        ],
    )
}

/// CR 603.7a: a delayed trigger "won't trigger until it has actually been
/// created, even if its trigger event occurred just beforehand". The rule's
/// second example: the creature untaps before the ability resolves, and the
/// trigger waits for the next untap. Its first: the creature leaves the
/// battlefield before the ability resolves, so "this creature" is gone
/// (CR 400.7), and the trigger never triggers, not even when the creature
/// comes back and leaves again.
// COVERS: ATOM-603.7a-001
#[test]
fn a_delayed_trigger_is_never_retroactive() {
    let mut game = setup_two_player_game();
    let sentinel = put_on_battlefield(&mut game, watchful_sentinel(), 0);

    game.activate_ability(0, sentinel, 0, &test_dp()).unwrap();
    game.execute_action(GameAction::Untap { object: sentinel }, &test_ctx()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.delayed_triggers.len(), 1);
    assert!(game.pending_triggers.is_empty(), "the untap before it existed triggers nothing");
    game.execute_action(GameAction::Tap { object: sentinel }, &test_ctx()).unwrap();
    game.execute_action(GameAction::Untap { object: sentinel }, &test_ctx()).unwrap();
    the_delayed_pending(&game);
    place_and_resolve(&mut game);
    assert_eq!(life(&game, 0), 22, "the next untap does");

    game.activate_ability(0, sentinel, 1, &test_dp()).unwrap();
    game.change_zone(sentinel, Zone::Hand, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    game.change_zone(sentinel, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.delayed_triggers.len(), 1, "created, about a creature already gone");
    game.change_zone(sentinel, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx()).unwrap();
    assert!(game.pending_triggers.is_empty(), "the creature that left now is a new object");
}

/// CR 603.7a in the one window that holds a record from before the entry was
/// created: a rider runs inside the batch it rides on (CR 615.5), after its
/// members are performed. The Archivist's rider creates "the next time a
/// creature dies, you gain 1 life" as player 0's creature dies beside the
/// exiled one; that death came first and triggers nothing, and the next one
/// does.
// COVERS: ATOM-603.7a-001
#[test]
fn a_delayed_trigger_a_rider_creates_does_not_see_its_own_batch() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        archivist_whose_rider(create(template(dies(a_creature()), gain_one(), DelayedDuration::Once))),
        0,
    );
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let mine = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let later = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    destroy_together(&mut game, &[theirs, mine], &test_dp());
    assert_eq!(game.get_object(mine).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.delayed_triggers.len(), 1, "the rider created it");
    assert!(game.pending_triggers.is_empty(), "the death performed before it existed triggers nothing");

    destroy_together(&mut game, &[later], &test_dp());
    the_delayed_pending(&game);
}

/// The departure note's other half: a delayed trigger whose source is still
/// there sees that source leave, on the record of that move, once.
#[test]
fn a_delayed_trigger_sees_its_own_source_leave() {
    let mut game = setup_two_player_game();
    let sentinel = put_on_battlefield(&mut game, watchful_sentinel(), 0);
    activate_and_resolve(&mut game, 0, sentinel, 1);
    assert!(game.pending_triggers.is_empty());

    game.change_zone(sentinel, Zone::Hand, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    assert_eq!(the_delayed_pending(&game).origin.source(), sentinel);
    place_and_resolve(&mut game);
    assert_eq!(life(&game, 0), 23);
}

/// CR 113.7a, 608.2h: a delayed trigger whose source has left reads the
/// source as it last existed. The creature leaves as a 4/4 before its end
/// step trigger fires: the registry keeps the frame it left with, and the
/// trigger carries it onto the stack, where the source's characteristics are
/// read from it.
#[test]
fn a_delayed_trigger_keeps_the_frame_its_source_left_with() {
    let mut game = setup_two_player_game();
    let herald = creature("Lingering Herald", vec![activated(false, at_the_next_end_step(gain_one()))]);
    let herald = put_on_battlefield(&mut game, herald, 0);
    let herald_then = game.object_ref(herald).unwrap();
    activate_and_resolve(&mut game, 0, herald, 0);
    let counters = GameAction::AddCounters {
        subject: CounterSubject::Object(herald),
        counter: CounterType::PlusOnePlusOne,
        n: 2,
        by: 0,
    };
    game.execute_action(counters, &test_ctx()).unwrap();
    assert!(game.delayed_triggers[0].source_frame.is_none(), "its source is still there");

    game.change_zone(herald, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx()).unwrap();
    let kept = game.delayed_triggers[0].source_frame.as_ref().map(|frame| frame.power);
    assert_eq!(kept, Some(Some(4)), "the 4/4 that left");

    advance_to(&mut game, 0, StepType::End);
    let departed = &the_delayed_pending(&game).departed;
    assert_eq!(departed.iter().map(|d| (d.object, d.frame.power)).collect::<Vec<_>>(), vec![(herald_then, Some(4))]);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    let on_stack = &game.stack_entries[game.stack.last().unwrap()];
    assert_eq!(on_stack.departed.iter().map(|d| d.object).collect::<Vec<_>>(), vec![herald_then], "onto the stack");
    place_and_resolve(&mut game);
    assert_eq!(life(&game, 0), 21);
}

/// The same for a spell's delayed trigger, whose source is the spell (CR
/// 603.7d): the spell is framed as it leaves the stack, where it last
/// existed as that object.
#[test]
fn a_spells_delayed_trigger_keeps_the_spell_as_it_left_the_stack() {
    let mut game = setup_two_player_game();
    let mend = instant("Delayed Mend", ManaCost::build(&[ManaType::White], 0), at_the_next_end_step(gain_one()));
    let mend = put_in_hand(&mut game, mend, 0);
    game.players[0].mana_pool.add(ManaType::White, 1);
    game.cast_spell(0, mend, &ManaWindowStop::new(test_dp())).expect("castable from exactly its cost");
    let cast = game.object_ref(mend).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
    let delayed = &game.delayed_triggers[0];
    assert_eq!(delayed.source, cast);
    let frame = delayed.source_frame.as_ref().map(|frame| frame.types.contains(&CardType::Instant));
    assert_eq!(frame, Some(true), "the instant spell, framed as it left the stack");
}

/// CR 603.7b: "a delayed triggered ability will trigger only once — the next
/// time its trigger event occurs — unless it has a stated duration". The
/// next creature to enter draws a card and the second does not; a "this
/// turn" one gains for each, and ends with the turn (CR 514.2).
// COVERS: ATOM-603.7b-001
#[test]
fn a_delayed_trigger_fires_once_unless_it_states_a_duration() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 10);
    let draw = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    resolve_spell(
        &mut game,
        instant("Next Arrival", ManaCost::build(&[ManaType::Blue], 0), create(template(enters(a_creature()), draw, DelayedDuration::Once))),
        0,
    );
    resolve_spell(
        &mut game,
        instant("Each Arrival", ManaCost::build(&[ManaType::White], 0), create(template(enters(a_creature()), gain_one(), DelayedDuration::ThisTurn))),
        0,
    );
    let hand = game.players[0].hand.len();

    create_token(&mut game, 1, token(1, Vec::new()));
    assert_eq!(game.pending_triggers.len(), 2, "the next time, and this turn");
    place_and_resolve(&mut game);
    create_token(&mut game, 1, token(1, Vec::new()));
    place_and_resolve(&mut game);
    assert_eq!(game.players[0].hand.len(), hand + 1, "the once trigger drew once");
    assert_eq!(life(&game, 0), 22, "the this-turn trigger gained twice");
    assert_eq!(game.delayed_triggers.len(), 1, "and only it is left");

    advance_to(&mut game, 1, StepType::Upkeep);
    assert!(game.delayed_triggers.is_empty(), "cleanup ended this turn's");
    create_token(&mut game, 1, token(1, Vec::new()));
    assert!(game.pending_triggers.is_empty());
}

/// "The next time a creature you control dies, you gain life equal to its
/// power." A wipe takes a 1/1 and a 3/3 at once.
fn next_death() -> Arc<CardData> {
    let you_control = ObjectFilter::And(Box::new(a_creature()), Box::new(ObjectFilter::ByController(PlayerRef::You)));
    instant(
        "Next Death",
        ManaCost::build(&[ManaType::Black], 0),
        create(template(
            dies(you_control),
            Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringPower), EffectRecipient::Controller),
            DelayedDuration::Once,
        )),
    )
}

fn destroy_together(game: &mut GameState, objects: &[ObjectId], dp: &dyn mtgsim::ui::decision::DecisionProvider) {
    let by = mtgsim::types::ids::new_object_id();
    let batch = objects.iter().map(|&object| GameAction::Destroy { object, source: DestructionSource::Effect(by) }).collect();
    game.execute_actions(batch, &ActionContext::new(dp)).unwrap();
}

/// CR 603.7b's second sentence: the event occurs twice at once and the
/// ability has no stated duration, so its controller chooses which event
/// causes it to trigger, and it triggers once. The atom's board is
/// Tatsumasa's "when that token dies" under a doubler; a trigger naming
/// objects its creator made is TR-3b's (`refs`, with Flickerwisp's "that
/// card"), and the choice is the same choice: two of a delayed trigger's
/// events, together, one trigger.
// COVERS: ATOM-603.7b-002
#[test]
fn simultaneous_events_trigger_a_once_delayed_trigger_once_by_its_controllers_choice() {
    let mut game = setup_two_player_game();
    let small = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let big = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    let spell = resolve_spell(&mut game, next_death(), 0);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::ChooseDelayedTriggerEvent { source: spell }, vec![ChoiceOption::Object(big)]);
    destroy_together(&mut game, &[small, big], &dp);
    assert!(dp.is_empty(), "the controller chose, at the trigger");
    let pending = the_delayed_pending(&game);
    assert_eq!(pending.binding.subject.map(|s| s.id), Some(big), "the 3/3's death caused it");
    assert!(game.delayed_triggers.is_empty());
    place_and_resolve(&mut game);
    assert_eq!(life(&game, 0), 23);
}

/// Item 224: attackers are declared outside any batch, so CR 603.7b's choice
/// had no provider when one `Once` delayed trigger matched two attackers.
/// The declaration's dispatch carries its step's provider: the controller
/// chooses which attack causes it, and it triggers once.
#[test]
fn two_attackers_declared_at_once_ask_a_once_delayed_trigger_s_controller() {
    use mtgsim::state::game_state::{Phase, PhaseType};
    let mut game = setup_two_player_game();
    let small = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let big = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    let you_control = ObjectFilter::And(Box::new(a_creature()), Box::new(ObjectFilter::ByController(PlayerRef::You)));
    let spell = resolve_spell(
        &mut game,
        instant(
            "Next Charge",
            ManaCost::build(&[ManaType::Red], 0),
            create(template(
                TriggerEvent::Attacks { attacker: TriggerSubject::Filter(you_control), multiplicity: mtgsim::types::triggers::Multiplicity::PerOccurrence },
                Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringPower), EffectRecipient::Controller),
                DelayedDuration::Once,
            )),
        ),
        0,
    );
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(StepType::DeclareAttackers) });

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::DeclareAttackers, vec![0, 1]);
    dp.expect_choice(ChoiceKind::ChooseDelayedTriggerEvent { source: spell }, vec![ChoiceOption::Object(big)]);
    game.process_declare_attackers(&dp).unwrap();
    assert!(dp.is_empty(), "both attacked, and the controller chose");
    assert_eq!(the_delayed_pending(&game).binding.subject.map(|s| s.id), Some(big));
    let _ = small;
    place_and_resolve(&mut game);
    assert_eq!(life(&game, 0), 23);
}

/// The choice is asked only between events that give different games: a
/// "draw a card" reads nothing of its event, so two deaths at once are one
/// answer, and nobody is asked (CR 102.2; item 163's reading).
#[test]
fn simultaneous_events_that_agree_on_everything_read_ask_nothing() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 10);
    let one = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let two = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    let draw = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    resolve_spell(&mut game, instant("Next Loss", ManaCost::zero(), create(template(dies(a_creature()), draw, DelayedDuration::Once))), 0);

    let dp = RecordingDecisionProvider::picking(0);
    destroy_together(&mut game, &[one, two], &dp);
    assert_eq!(dp.prompts(), 0);
    the_delayed_pending(&game);
}

/// "{0}: Exile this creature at the beginning of the next end step", and
/// "{0}: Destroy this creature at the beginning of the next end step".
fn doomed_construct() -> Arc<CardData> {
    creature(
        "Doomed Construct",
        vec![
            activated(false, at_the_next_end_step(Effect::Atom(Primitive::Exile, EffectRecipient::ThisObject))),
            activated(false, at_the_next_end_step(Effect::Atom(Primitive::Destroy, EffectRecipient::ThisObject))),
        ],
    )
}

/// CR 603.7c: a delayed trigger "that refers to a particular object still
/// affects it even if the object changes characteristics". The rule's own
/// example: "exile this creature at the beginning of the next end step"
/// exiles the permanent though it is no longer a creature.
// COVERS: ATOM-603.7c-001
#[test]
fn a_delayed_trigger_affects_its_object_whatever_it_has_become() {
    let mut game = setup_two_player_game();
    let construct = put_on_battlefield(&mut game, doomed_construct(), 0);
    activate_and_resolve(&mut game, 0, construct, 0);
    game.continuous_effects.add(registered(construct, Layer::Layer4Type, 100, EffectModification::RemoveType(CardType::Creature)));
    assert!(!is_creature(&game, construct), "no longer a creature");

    advance_to(&mut game, 0, StepType::End);
    place_and_resolve(&mut game);
    assert_eq!(game.get_object(construct).unwrap().zone, Zone::Exile);
}

/// CR 400.7: the delayed trigger remembers its object by identity, so the
/// creature bounced and replayed before the end step is a new object, and
/// "destroy this creature" finds nothing (CR 603.7c's second sentence). The
/// trigger still triggers and resolves; it does nothing.
// COVERS: ATOM-400.7-001
#[test]
fn a_delayed_trigger_does_not_follow_its_object_to_a_new_existence() {
    let mut game = setup_two_player_game();
    let construct = put_on_battlefield(&mut game, doomed_construct(), 0);
    activate_and_resolve(&mut game, 0, construct, 1);
    game.change_zone(construct, Zone::Hand, ZoneChangeCause::Returned, &test_ctx()).unwrap();
    game.change_zone(construct, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx()).unwrap();

    advance_to(&mut game, 0, StepType::End);
    the_delayed_pending(&game);
    place_and_resolve(&mut game);
    assert_eq!(game.get_object(construct).unwrap().zone, Zone::Battlefield, "the new object is not destroyed");
}

// ---------------------------------------------------------------------------
// 4. Provenance
// ---------------------------------------------------------------------------

/// CR 603.7d: a spell's delayed trigger is the spell's, and its controller is
/// the player who controlled the spell as it resolved. "At the beginning of
/// the next upkeep, each player draws a card": the next upkeep is player 1's,
/// and the trigger is still player 0's.
// COVERS: ATOM-603.7d-001
#[test]
fn a_spells_delayed_trigger_is_the_spells_and_its_controllers() {
    let mut game = setup_two_player_game();
    let each_draws = Effect::Atom(
        Primitive::DrawCards(AmountExpr::Fixed(1)),
        EffectRecipient::EachOf(PlayerGroup::set(PlayerSet::Everyone)),
    );
    let gift = resolve_spell(
        &mut game,
        instant(
            "Upkeep Gift",
            ManaCost::build(&[ManaType::Green], 0),
            create(template(at_beginning_of(StepType::Upkeep, Whose::Each), each_draws, DelayedDuration::Once)),
        ),
        0,
    );
    advance_to(&mut game, 1, StepType::Upkeep);
    let pending = the_delayed_pending(&game);
    assert_eq!(pending.controller, 0, "its spell's controller, in player 1's upkeep");
    assert_eq!(pending.origin.source_ref(), game.object_ref(gift).unwrap(), "its source is the spell");
    let hands = (game.players[0].hand.len(), game.players[1].hand.len());
    place_and_resolve(&mut game);
    assert_eq!((game.players[0].hand.len(), game.players[1].hand.len()), (hands.0 + 1, hands.1 + 1));
}

/// "{0}: At the beginning of the next end step, put a +1/+1 counter on this
/// creature", and "When this creature enters, at the beginning of the next
/// end step, you gain 1 life" (Flickerwisp's shape, on a token).
fn mirror_warden_ability() -> AbilityDef {
    activated(
        false,
        at_the_next_end_step(Effect::Atom(
            Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You },
            EffectRecipient::ThisObject,
        )),
    )
}

/// CR 603.7e: an activated or triggered ability's delayed trigger has that
/// ability's source as its own, and its controller is the player who
/// controlled that ability as it resolved: player 0's, though player 1
/// controls the creature by the end step, and its "this creature" is the
/// creature. A triggered ability's, by the same rule, is the token's.
// COVERS: ATOM-603.7e-001
#[test]
fn an_abilitys_delayed_trigger_has_that_abilitys_source() {
    let mut game = setup_two_player_game();
    let warden = put_on_battlefield(&mut game, creature("Mirror Warden", vec![mirror_warden_ability()]), 0);
    let warden_then = game.object_ref(warden).unwrap();
    activate_and_resolve(&mut game, 0, warden, 0);
    let etb = triggered_ability("", whenever(enters(TriggerSubject::ThisObject), at_the_next_end_step(gain_one())));
    let token = create_token(&mut game, 0, token(1, vec![etb]));
    place_and_resolve(&mut game);
    assert_eq!(game.delayed_triggers.len(), 2);
    give_control(&mut game, warden, 1);

    advance_to(&mut game, 0, StepType::End);
    assert_eq!(game.pending_triggers.len(), 2);
    let sources: Vec<(ObjectId, PlayerId)> =
        game.pending_triggers.iter().map(|t| (t.origin.source(), t.controller)).collect();
    assert_eq!(sources, vec![(warden, 0), (token, 0)]);
    assert_eq!(game.pending_triggers[0].origin.source_ref(), warden_then, "the existence that activated it");
    place_and_resolve(&mut game);
    assert_eq!(game.battlefield[&warden].counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(life(&game, 0), 21);
}

/// "If a nontoken creature an opponent controls would die, exile it instead.
/// At the beginning of the next end step, you gain 2 life": a static
/// replacement whose rider creates a delayed trigger (Kalitas's shape).
fn grim_archivist() -> Arc<CardData> {
    archivist_whose_rider(at_the_next_end_step(gain(2)))
}

/// Kalitas's replacement, with `rider` as its CR 615.5 rider.
fn archivist_whose_rider(rider: Effect) -> Arc<CardData> {
    let opponents_creature = ObjectFilter::And(
        Box::new(a_creature()),
        Box::new(ObjectFilter::And(
            Box::new(ObjectFilter::Not(Box::new(ObjectFilter::Token))),
            Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
        )),
    );
    let replacement = ReplacementDef::new(
        EventPattern::ZoneChange { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), cause: None, object: None },
        ObjectSet::battlefield_filter(opponents_creature),
        Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
    )
    .with_then(rider);
    CardDataBuilder::new("Grim Archivist")
        .card_type(CardType::Enchantment)
        .ability(AbilityDef {
            rules_text: "".into(),
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(replacement)),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
        })
        .build()
}

/// CR 603.7f: a replacement effect a static ability generates creates the
/// delayed trigger, so its source is the object with the static ability and
/// its controller is that object's controller as the replacement applied:
/// player 0, though player 1 controls the Archivist by the end step.
// COVERS: ATOM-603.7f-001
#[test]
fn a_replacements_delayed_trigger_is_its_objects_as_it_applied() {
    let mut game = setup_two_player_game();
    let archivist = put_on_battlefield(&mut game, grim_archivist(), 0);
    let victim = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    destroy_together(&mut game, &[victim], &test_dp());
    assert_eq!(game.get_object(victim).unwrap().zone, Zone::Exile, "the replacement applied");
    assert_eq!(game.delayed_triggers.len(), 1, "and its rider created the delayed trigger");
    assert_eq!(game.delayed_triggers[0].source, game.object_ref(archivist).unwrap());
    give_control(&mut game, archivist, 1);

    advance_to(&mut game, 0, StepType::End);
    let pending = the_delayed_pending(&game);
    assert_eq!((pending.origin.source(), pending.controller), (archivist, 0));
    place_and_resolve(&mut game);
    assert_eq!((life(&game, 0), life(&game, 1)), (22, 20));
}

/// CR 603.7g, by fixture: nothing lets a player take a special action a
/// static ability allows yet (`backlog.md` §2.8), so the registry is handed
/// the provenance that action would give it, the object with the static
/// ability under its controller as the action was taken. The trigger keeps
/// both when control of the object changes.
// COVERS: ATOM-603.7g-001
#[test]
fn a_special_actions_delayed_trigger_is_its_static_abilitys_objects() {
    let mut game = setup_two_player_game();
    let altar = put_on_battlefield(&mut game, CardDataBuilder::new("Waiting Altar").card_type(CardType::Artifact).build(), 0);
    let card = Arc::clone(&game.get_object(altar).unwrap().card_data);
    game.register_delayed_trigger(
        &template(at_beginning_of(StepType::End, Whose::Each), gain(2), DelayedDuration::Once),
        DelayedProvenance {
            source: game.object_ref(altar).unwrap(),
            source_card: card,
            controller: 0,
            created_by: None,
            x_value: None,
            turn: TriggerTurn::Any,
        },
    );
    give_control(&mut game, altar, 1);

    advance_to(&mut game, 0, StepType::End);
    let pending = the_delayed_pending(&game);
    assert_eq!((pending.origin.source(), pending.controller), (altar, 0));
    place_and_resolve(&mut game);
    assert_eq!(life(&game, 0), 22);
}

// ---------------------------------------------------------------------------
// 5. CR 603.7h, 107.3n, 513.2
// ---------------------------------------------------------------------------

/// "{0}: Put a +1/+1 counter on this creature. When this ability has resolved
/// for the third time this turn, sacrifice this creature."
fn overcharged_golem() -> Arc<CardData> {
    let sacrifice = create(template(
        TriggerEvent::AbilityResolves { identity: IdentityRef::ThisAbility },
        Effect::Atom(Primitive::Sacrifice, EffectRecipient::ThisObject),
        DelayedDuration::Once,
    ));
    let mut ability = activated(
        false,
        Effect::Sequence(vec![
            Effect::Atom(
                Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You },
                EffectRecipient::ThisObject,
            ),
            Effect::Conditional(Condition::ResolvedThisTurn(3), Box::new(sacrifice)),
        ]),
    );
    ability.id = new_ability_id();
    creature("Overcharged Golem", vec![ability])
}

/// CR 603.7h: the delayed trigger "is created only once, during the
/// appropriate resolution of that ability". The first two resolutions create
/// nothing; the third creates it, and it triggers on that resolution's own
/// end (CR 608.2n's `AbilityResolved`) and sacrifices the golem.
// COVERS: ATOM-603.7h-001
#[test]
fn a_delayed_trigger_on_the_nth_resolution_is_created_by_that_resolution_alone() {
    let mut game = setup_two_player_game();
    let golem = put_on_battlefield(&mut game, overcharged_golem(), 0);
    for _ in 0..2 {
        activate_and_resolve(&mut game, 0, golem, 0);
        assert!(game.delayed_triggers.is_empty() && game.pending_triggers.is_empty());
    }
    activate_and_resolve(&mut game, 0, golem, 0);
    assert!(game.delayed_triggers.is_empty(), "created by the third resolution, and triggered by its end");
    the_delayed_pending(&game);
    place_and_resolve(&mut game);
    assert_eq!(game.get_object(golem).unwrap().zone, Zone::Graveyard);
}

/// "{X}{G}: At the beginning of the next end step, you gain X life."
fn deferred_bounty() -> Arc<CardData> {
    let gain_x = Effect::Atom(Primitive::GainLife(AmountExpr::X), EffectRecipient::Controller);
    instant(
        "Deferred Bounty",
        ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Green)]),
        at_the_next_end_step(gain_x),
    )
}

/// CR 107.3n: the delayed trigger's text refers to an X it does not define,
/// and the spell that created it had X chosen, so its X is the spell's. Cast
/// from hand with X = 3 from exactly {3}{G}.
// COVERS: ATOM-107.3n-001
#[test]
fn a_delayed_trigger_takes_the_x_of_the_spell_that_created_it() {
    let mut game = setup_two_player_game();
    let bounty = put_in_hand(&mut game, deferred_bounty(), 0);
    game.players[0].mana_pool.add(ManaType::Green, 1);
    game.players[0].mana_pool.add(ManaType::Colorless, 3);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_number(ChoiceKind::ChooseXValue { spell_id: bounty, x_count: 1 }, 3);
    game.cast_spell(0, bounty, &ManaWindowStop::new(dp)).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.delayed_triggers[0].x_value, Some(3));

    advance_to(&mut game, 0, StepType::End);
    place_and_resolve(&mut game);
    assert_eq!(life(&game, 0), 23);
}

/// CR 513.2's first sentence: a permanent with an "at the beginning of the end
/// step" ability that enters during the end step does not trigger until the
/// next turn's, since the step has begun and does not back up.
// COVERS: ATOM-513.2-001
#[test]
fn a_permanent_entering_during_the_end_step_waits_for_the_next() {
    let mut game = setup_two_player_game();
    advance_to(&mut game, 0, StepType::End);
    let sacrifice_itself = triggered_ability(
        "",
        whenever(at_beginning_of(StepType::End, Whose::Each), Effect::Atom(Primitive::Sacrifice, EffectRecipient::ThisObject)),
    );
    let ephemeral = create_token(&mut game, 0, token(1, vec![sacrifice_itself]));
    assert!(game.pending_triggers.is_empty(), "the end step has begun");

    advance_to(&mut game, 1, StepType::End);
    assert_eq!(game.pending_triggers.len(), 1, "the next turn's end step");
    place_and_resolve(&mut game);
    assert!(!game.battlefield.contains_key(&ephemeral));
}

/// CR 513.2's second sentence: a delayed "at the beginning of the next end
/// step" created during the end step waits for the next turn's.
// COVERS: ATOM-513.2-002
#[test]
fn a_delayed_trigger_created_during_the_end_step_waits_for_the_next() {
    let mut game = setup_two_player_game();
    advance_to(&mut game, 0, StepType::End);
    resolve_spell(&mut game, instant("Late Gift", ManaCost::build(&[ManaType::White], 0), at_the_next_end_step(gain(2))), 0);
    assert!(game.pending_triggers.is_empty(), "the end step does not back up");

    advance_to(&mut game, 1, StepType::End);
    the_delayed_pending(&game);
    place_and_resolve(&mut game);
    assert_eq!(life(&game, 0), 22);
}
