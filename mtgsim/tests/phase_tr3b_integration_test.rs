//! TR-3b — the returns, CR 610.3's "until", and the objects a delayed
//! trigger names (`triggers-architecture.md`, TR-3b).
//!
//! 1. `codebase-state.md` item 223's row half: a resolution's registry rows
//!    name the object that has the ability, and only the rows that end with
//!    their source end as it leaves the battlefield (CR 611.2a, 611.2b,
//!    611.3b).
//! 2. What a delayed trigger refers to (CR 603.7c): `Effect::Remember`'s
//!    objects and players, each object by identity (CR 400.7).

use std::sync::Arc;

use mtgsim::cards::authoring::{at_beginning_of, dies, enters, triggered_ability, whenever, Whose};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_rb_cards::rest_in_peace;
use mtgsim::cards::phase_re_cards::parallel_lives;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, Layer};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_controller, get_effective_power};
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::test_support::{
    put_on_battlefield, registered, setup_two_player_game, test_ctx, test_dp, RecordingDecisionProvider,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, SelectionFilter, TargetCount,
    TokenDef,
};
use mtgsim::types::ids::{AbilityId, ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::triggers::{DelayedDuration, DelayedTriggerTemplate, DelayedTurn, TriggerSubject};
use mtgsim::types::zones::{DestructionSource, Zone, ZoneChangeCause};
use mtgsim::ui::decision::DecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Activate `source`'s first ability for `player` and resolve it.
fn activate_and_resolve(game: &mut GameState, player: PlayerId, source: ObjectId) {
    game.activate_ability(player, source, 0, &test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
}

/// An activated ability with no cost but `{T}`.
fn tap_ability(rules_text: &'static str, effect: Effect) -> AbilityDef {
    AbilityDef {
        rules_text: rules_text.into(),
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Activated,
        costs: vec![Cost::TapSelf],
        effect,
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    }
}

fn sacrifice(game: &mut GameState, id: ObjectId) {
    game.change_zone(id, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx()).unwrap();
}

/// Activate `source`'s first ability for `player`, answering every prompt
/// with `dp`, and resolve it.
fn activate_and_resolve_with(game: &mut GameState, player: PlayerId, source: ObjectId, dp: &dyn DecisionProvider) {
    game.activate_ability(player, source, 0, dp).unwrap();
    game.resolve_top_of_stack(dp).unwrap();
}

/// Put every waiting trigger on the stack and resolve the stack, taking the
/// first option at any prompt on the way.
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

/// "At the beginning of the next end step, [effect]."
fn at_the_next_end_step(effect: Effect) -> Effect {
    let def = Arc::new(whenever(at_beginning_of(StepType::End, Whose::Each), effect));
    delayed(DelayedTriggerTemplate { def, duration: DelayedDuration::Once, turn: DelayedTurn::Any, rules_text: "".into() })
}

/// The instruction that creates `template`.
fn delayed(template: DelayedTriggerTemplate) -> Effect {
    Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(template)), EffectRecipient::Controller)
}

/// `effect` on what the delayed trigger refers to.
fn on_referred(primitive: Primitive) -> Effect {
    Effect::Atom(primitive, EffectRecipient::Referred)
}

fn remember(effect: Effect) -> Effect {
    Effect::Remember(Box::new(effect))
}

/// An artifact with one `{T}` ability, which it can activate the turn it
/// arrives.
fn totem(name: &str, rules_text: &'static str, effect: Effect) -> Arc<CardData> {
    CardDataBuilder::new(name).card_type(CardType::Artifact).ability(tap_ability(rules_text, effect)).build()
}

/// A vanilla token creature of `power`.
fn token(name: &str, power: i32) -> TokenDef {
    TokenDef {
        name: Some(name.to_string()),
        colors: Vec::new(),
        types: vec![CardType::Creature],
        subtypes: Vec::new(),
        supertypes: Vec::new(),
        power: Some(power),
        toughness: Some(power),
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
    }
}

fn create_one(def: TokenDef) -> Effect {
    Effect::Atom(Primitive::CreateToken(def, AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

fn tokens_named(game: &GameState, name: &str) -> Vec<ObjectId> {
    game.battlefield_ids_ordered()
        .into_iter()
        .filter(|id| game.get_object(*id).is_ok_and(|o| o.is_token && o.card_data.name == name))
        .collect()
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

/// "Target creature you control" — one instance.
fn target_your_creature() -> EffectRecipient {
    EffectRecipient::Target(
        SelectionFilter::Permanent(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Creature)),
            Box::new(ObjectFilter::ByController(PlayerRef::You)),
        )),
        TargetCount::Exactly(1),
    )
}

/// "{T}: Target creature gets +2/+2 until end of turn." An artifact, so its
/// ability can be activated the turn it arrives.
fn pump_totem() -> Arc<CardData> {
    CardDataBuilder::new("Pump Totem")
        .card_type(CardType::Artifact)
        .ability(tap_ability(
            "{T}: Target creature gets +2/+2 until end of turn.",
            Effect::Atom(
                Primitive::ModifyPowerToughness(AmountExpr::Fixed(2), AmountExpr::Fixed(2), Duration::UntilEndOfTurn),
                EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            ),
        ))
        .build()
}

/// Sower of Temptation's shape: "When this creature enters, gain control of
/// target creature an opponent controls for as long as this creature remains
/// on the battlefield."
fn tempter() -> Arc<CardData> {
    CardDataBuilder::new("Tempter")
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Faerie))
        .power_toughness(2, 2)
        .ability(triggered_ability(
            "When this creature enters, gain control of target creature an opponent controls for as long as this creature remains on the battlefield.",
            whenever(
                enters(TriggerSubject::ThisObject),
                Effect::Atom(
                    Primitive::GainControl(Duration::WhileSourceOnBattlefield),
                    EffectRecipient::Target(
                        SelectionFilter::Permanent(ObjectFilter::And(
                            Box::new(ObjectFilter::ByType(CardType::Creature)),
                            Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
                        )),
                        TargetCount::Exactly(1),
                    ),
                ),
            ),
        ))
        .build()
}

/// "{T}: Regenerate this creature." A creature put down by
/// `put_on_battlefield`, which is never summoning-sick.
fn regenerator() -> Arc<CardData> {
    CardDataBuilder::new("Regenerator")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(tap_ability(
            "{T}: Regenerate this creature.",
            Effect::Atom(Primitive::Regenerate, EffectRecipient::ThisObject),
        ))
        .build()
}

// ---------------------------------------------------------------------------
// 1. Item 223's rows
// ---------------------------------------------------------------------------

/// CR 611.2a: "gets +2/+2 until end of turn" lasts until end of turn,
/// whatever becomes of the object whose ability made it. The row names the
/// totem (CR 113.7a), so the sweep that ends a static ability's rows as its
/// source leaves must leave this one.
#[test]
fn a_pump_an_activated_ability_made_outlives_its_source() {
    let mut game = setup_two_player_game();
    let totem = put_on_battlefield(&mut game, pump_totem(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    activate_and_resolve(&mut game, 0, totem);
    assert_eq!(get_effective_power(&game, bears), Some(4));
    let sources: Vec<ObjectId> = game.continuous_effects.iter().map(|row| row.source).collect();
    assert_eq!(sources, vec![totem], "the pump's row names the totem, not the ability's stack object");

    sacrifice(&mut game, totem);
    assert_eq!(get_effective_power(&game, bears), Some(4), "CR 611.2a: the pump outlives its source");
}

/// CR 611.2b: "for as long as this creature remains on the battlefield"
/// ends as it leaves, and the stolen creature goes back to its controller.
#[test]
fn a_for_as_long_as_row_a_resolution_made_ends_as_its_source_leaves() {
    let mut game = setup_two_player_game();
    let stolen = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let tempter = put_on_battlefield(&mut game, tempter(), 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(get_effective_controller(&game, stolen), Some(0));

    sacrifice(&mut game, tempter);
    assert_eq!(get_effective_controller(&game, stolen), Some(1), "CR 611.2b: the duration ended with its source");
}

/// Item 95: the shield "{T}: Regenerate this creature" makes is the
/// creature's (CR 113.7a), and its row is what CR 616.1's prompt names
/// (`replacement::gather` reads the row's source).
#[test]
fn a_regeneration_shield_from_an_activated_ability_names_the_permanent() {
    let mut game = setup_two_player_game();
    let regenerator = put_on_battlefield(&mut game, regenerator(), 0);
    activate_and_resolve(&mut game, 0, regenerator);
    let sources: Vec<ObjectId> = game.replacement_effects.iter().map(|row| row.source).collect();
    assert_eq!(sources, vec![regenerator]);
}

// ---------------------------------------------------------------------------
// 2. What a delayed trigger refers to (CR 603.7c)
// ---------------------------------------------------------------------------

/// Kiki-Jiki's and Twinflame's shape: "{T}: Create a 1/1 Soldier token.
/// Exile it at the beginning of the next end step."
fn soldier_totem() -> Arc<CardData> {
    totem(
        "Soldier Totem",
        "{T}: Create a 1/1 Soldier creature token. Exile it at the beginning of the next end step.",
        Effect::Sequence(vec![
            remember(create_one(token("Soldier", 1))),
            at_the_next_end_step(on_referred(Primitive::Exile)),
        ]),
    )
}

/// Kiki-Jiki's and Twinflame's rulings: when a doubler makes two tokens,
/// "you'll exile each of them". They are read off what the creation
/// performed, never off its count, so both are "it".
#[test]
fn a_delayed_trigger_refers_to_every_token_a_doubled_creation_made() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, parallel_lives(), 0);
    let totem = put_on_battlefield(&mut game, soldier_totem(), 0);
    activate_and_resolve(&mut game, 0, totem);
    let soldiers = tokens_named(&game, "Soldier");
    assert_eq!(soldiers.len(), 2, "Parallel Lives doubled the creation");

    advance_to(&mut game, 0, StepType::End);
    place_and_resolve(&mut game);
    assert!(tokens_named(&game, "Soldier").is_empty(), "both tokens were exiled");
}

/// The Tatsumasa board: "Create a 5/5 Dragon token. [...] when that token
/// dies", under a doubler, with both tokens dying at once. CR 603.7b: the
/// delayed trigger triggers once, its controller choosing which death
/// causes it, and the two causes give one game, so it asks nothing.
#[test]
fn a_delayed_trigger_watching_two_tokens_that_die_at_once_triggers_once() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, parallel_lives(), 0);
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(5)), EffectRecipient::Controller);
    let when_it_dies = DelayedTriggerTemplate {
        def: Arc::new(whenever(dies(TriggerSubject::Referred), gain)),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::Any,
        rules_text: "".into(),
    };
    let totem = put_on_battlefield(
        &mut game,
        totem(
            "Dragon's Fang Totem",
            "{T}: Create a 5/5 Dragon creature token. When that token dies, you gain 5 life.",
            Effect::Sequence(vec![remember(create_one(token("Dragon", 5))), delayed(when_it_dies)]),
        ),
        0,
    );
    activate_and_resolve(&mut game, 0, totem);
    let dragons = tokens_named(&game, "Dragon");
    assert_eq!(dragons.len(), 2);

    let dp = RecordingDecisionProvider::picking(0);
    let destroy = |id| GameAction::Destroy { object: id, source: DestructionSource::StateBasedAction };
    game.execute_actions(dragons.iter().map(|&id| destroy(id)).collect(), &ActionContext::new(&dp)).unwrap();
    place_and_resolve(&mut game);
    assert_eq!(life(&game, 0), 25, "one trigger for the two deaths");
    assert!(game.delayed_triggers.is_empty(), "triggered once and gone (CR 603.7b)");
    assert!(dp.kinds().iter().all(|k| k != "ChooseDelayedTriggerEvent"), "two causes, one game: nothing asked");
}

/// Sneak Attack's shape on a creature already there: "{T}: Target creature
/// you control gains haste until end of turn. Sacrifice it at the beginning
/// of the next end step."
fn haste_totem() -> Arc<CardData> {
    totem(
        "Haste Totem",
        "{T}: Target creature you control gains haste until end of turn. Sacrifice it at the beginning of the next end step.",
        Effect::Sequence(vec![
            remember(Effect::Atom(
                Primitive::GrantKeywordFlag(KeywordFlag::Haste, Duration::UntilEndOfTurn),
                target_your_creature(),
            )),
            at_the_next_end_step(on_referred(Primitive::Sacrifice)),
        ]),
    )
}

/// The instruction moves nothing, so what it remembers is its target as it
/// is: still there at the end step, it is sacrificed.
#[test]
fn a_delayed_trigger_sacrifices_the_creature_it_refers_to() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let totem = put_on_battlefield(&mut game, haste_totem(), 0);
    activate_and_resolve(&mut game, 0, totem);

    advance_to(&mut game, 0, StepType::End);
    place_and_resolve(&mut game);
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Graveyard);
}

/// Sneak Attack's ruling: "If that creature has left the battlefield, even if
/// it came back, you don't sacrifice it." The one that came back is a new
/// object (CR 400.7).
#[test]
fn a_creature_that_left_and_came_back_is_not_the_one_referred_to() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let totem = put_on_battlefield(&mut game, haste_totem(), 0);
    activate_and_resolve(&mut game, 0, totem);
    game.change_zone(bears, Zone::Exile, ZoneChangeCause::Exiled, &test_ctx()).unwrap();
    game.change_zone(bears, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx()).unwrap();

    advance_to(&mut game, 0, StepType::End);
    place_and_resolve(&mut game);
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Battlefield, "the creature that came back stays");
}

/// Sneak Attack's ruling: "You sacrifice the creature only if you still
/// control it."
#[test]
fn a_creature_referred_to_that_another_player_controls_is_not_sacrificed() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let totem = put_on_battlefield(&mut game, haste_totem(), 0);
    activate_and_resolve(&mut game, 0, totem);
    game.continuous_effects.add(ContinuousEffect {
        duration: Duration::Indefinite,
        ..registered(bears, Layer::Layer2Control, 200, EffectModification::SetController(PlayerRef::Player(1)))
    });

    advance_to(&mut game, 0, StepType::End);
    place_and_resolve(&mut game);
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Battlefield);
}

/// "That player": a player the remembered instruction named. "{T}: Target
/// player gains 1 life. At the beginning of the next end step, that player
/// gains 3 life."
#[test]
fn a_delayed_trigger_refers_to_the_player_its_creator_named() {
    let mut game = setup_two_player_game();
    let totem = put_on_battlefield(
        &mut game,
        totem(
            "Kindness Totem",
            "{T}: Target player gains 1 life. At the beginning of the next end step, that player gains 3 life.",
            Effect::Sequence(vec![
                remember(Effect::Atom(
                    Primitive::GainLife(AmountExpr::Fixed(1)),
                    EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
                )),
                at_the_next_end_step(on_referred(Primitive::GainLife(AmountExpr::Fixed(3)))),
            ]),
        ),
        0,
    );
    activate_and_resolve_with(&mut game, 0, totem, &RecordingDecisionProvider::picking(1));
    assert_eq!((life(&game, 0), life(&game, 1)), (20, 21), "player 1 was targeted");

    advance_to(&mut game, 0, StepType::End);
    place_and_resolve(&mut game);
    assert_eq!((life(&game, 0), life(&game, 1)), (20, 24));
}

/// A move a replacement sent elsewhere is not "that card": Rest in Peace
/// exiles the creature a destruction would put into a graveyard, so the
/// delayed trigger refers to nothing.
#[test]
fn an_object_a_replacement_sent_elsewhere_is_not_referred_to() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, rest_in_peace(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let totem = put_on_battlefield(
        &mut game,
        totem(
            "Grave Totem",
            "{T}: Destroy target creature you control. At the beginning of the next end step, exile it.",
            Effect::Sequence(vec![
                remember(Effect::Atom(Primitive::Destroy, target_your_creature())),
                at_the_next_end_step(on_referred(Primitive::Exile)),
            ]),
        ),
        0,
    );
    activate_and_resolve(&mut game, 0, totem);
    assert_eq!(game.get_object(bears).unwrap().zone, Zone::Exile, "Rest in Peace exiled it instead");
    assert_eq!(game.delayed_triggers.len(), 1);
    assert!(game.delayed_triggers[0].referred.objects.is_empty(), "it never reached the graveyard");
}

/// The window holds a resolution's records until the resolution ends, and
/// not after it (`EventWindow::resolution_records`).
#[test]
fn the_window_holds_a_resolutions_records_until_it_ends() {
    let mut game = setup_two_player_game();
    let totem = put_on_battlefield(&mut game, soldier_totem(), 0);
    activate_and_resolve(&mut game, 0, totem);
    assert!(game.events.held().is_empty(), "flushed as the resolution ended");
    assert_eq!(game.delayed_triggers[0].referred.objects.len(), 1, "read before the flush");
}
