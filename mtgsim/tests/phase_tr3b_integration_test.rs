//! TR-3b — the returns, CR 610.3's "until", and the objects a delayed
//! trigger names (`triggers-architecture.md`, TR-3b).
//!
//! 1. `codebase-state.md` item 223's row half: a resolution's registry rows
//!    name the object that has the ability, and only the rows that end with
//!    their source end as it leaves the battlefield (CR 611.2a, 611.2b,
//!    611.3b).
//! 2. What a delayed trigger refers to (CR 603.7c): `Effect::Remember`'s
//!    objects and players, each object by identity (CR 400.7).
//! 3. Flickerwisp: the return (`Primitive::ReturnToBattlefield`), its four
//!    rulings, and an Aura that returns (CR 303.4f/g).
//! 4. CR 610.3's "until": Banishing Light, its four rulings, and §13's six
//!    CR 610.3 atoms.
//! 5. Item 226.
//! 6. The review: an exile and a return in one resolution (Cloudshift's
//!    shape) and CR 111.8, and an "until" whose event is not a leaving.

use std::sync::Arc;

use mtgsim::cards::authoring::{
    at_beginning_of, dies, enters, leaves_the_battlefield, triggered_ability, whenever, Whose,
};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lh_cards::{cobbled_wings, holy_strength};
use mtgsim::cards::phase_rb_cards::rest_in_peace;
use mtgsim::cards::phase_re_cards::parallel_lives;
use mtgsim::cards::phase_tr1_cards::soul_warden;
use mtgsim::cards::phase_tr3b_cards::{banishing_light, flickerwisp};
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, Layer};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_controller, get_effective_power};
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::test_support::{
    card_of_type, put_in_hand, put_on_battlefield, registered, setup_two_player_game, test_ctx, test_dp,
    vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, ReturnUnder, SelectionFilter,
    TargetCount, TokenDef,
};
use mtgsim::types::ids::{AbilityId, ObjectId, PlayerId, UntilReturnId};
use mtgsim::types::effects::CounterType;
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::ManaType;
use mtgsim::types::triggers::{DelayedDuration, DelayedTriggerTemplate, DelayedTurn, TriggerSubject};
use mtgsim::types::zones::{DestructionSource, Zone, ZoneChangeCause};
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

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
    resolve_all(game, &RecordingDecisionProvider::picking(0));
}

/// [`place_and_resolve`], answering every prompt with `dp`.
fn resolve_all(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(dp).unwrap();
        game.perform_sba_and_triggers(dp).unwrap();
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

// ---------------------------------------------------------------------------
// 3. Flickerwisp
// ---------------------------------------------------------------------------

/// Flickerwisp enters for `player` and its trigger resolves, exiling whatever
/// `dp` targets.
fn flicker(game: &mut GameState, player: PlayerId, dp: &dyn DecisionProvider) -> ObjectId {
    let wisp = put_on_battlefield(game, flickerwisp(), player);
    resolve_all(game, dp);
    wisp
}

fn zone(game: &GameState, id: ObjectId) -> Zone {
    game.get_object(id).unwrap().zone
}

/// Flickerwisp cast from hand with exactly its cost, through the mana window
/// a shipped client pays from: it enters, exiles the opponent's creature, and
/// the card comes back under its owner's control at the next end step.
// COVERS: ATOM-603.7e-001
#[test]
fn flickerwisp_cast_from_hand_returns_the_card_it_exiled_at_the_next_end_step() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let wisp = put_in_hand(&mut game, flickerwisp(), 0);
    game.players[0].mana_pool.add(ManaType::White, 3);
    game.cast_spell(0, wisp, &ManaWindowStop::new(test_dp())).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).unwrap();
    place_and_resolve(&mut game);
    assert_eq!(zone(&game, bears), Zone::Exile);
    assert_eq!(game.delayed_triggers.len(), 1);
    assert_eq!(game.delayed_triggers[0].source, game.object_ref(wisp).unwrap(), "Flickerwisp's (CR 603.7e)");

    advance_to(&mut game, 0, StepType::End);
    place_and_resolve(&mut game);
    assert_eq!(zone(&game, bears), Zone::Battlefield);
    assert_eq!(get_effective_controller(&game, bears), Some(1), "under its owner's control");
}

// RULING: Flickerwisp #1 - "The exiled card will return to the battlefield at
//   the beginning of the end step even if Flickerwisp is no longer on the
//   battlefield."
#[test]
fn flickerwisps_card_returns_though_flickerwisp_has_left() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let wisp = flicker(&mut game, 0, &test_dp());
    sacrifice(&mut game, wisp);

    advance_to(&mut game, 0, StepType::End);
    place_and_resolve(&mut game);
    assert_eq!(zone(&game, bears), Zone::Battlefield);
}

/// "At the beginning of the end step, you gain 1 life."
fn end_step_creature() -> Arc<CardData> {
    CardDataBuilder::new("Dusk Keeper")
        .card_type(CardType::Creature)
        .power_toughness(1, 1)
        .ability(triggered_ability(
            "At the beginning of the end step, you gain 1 life.",
            whenever(
                at_beginning_of(StepType::End, Whose::Each),
                Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
            ),
        ))
        .build()
}

// RULING: Flickerwisp #2 - "If the permanent that returns to the battlefield
//   has any abilities that trigger at the beginning of the end step, those
//   abilities won't trigger that turn."
#[test]
fn a_returned_permanents_end_step_ability_waits_for_the_next_end_step() {
    let mut game = setup_two_player_game();
    let keeper = put_on_battlefield(&mut game, end_step_creature(), 1);
    flicker(&mut game, 0, &test_dp());
    assert_eq!(zone(&game, keeper), Zone::Exile);

    advance_to(&mut game, 0, StepType::End);
    place_and_resolve(&mut game);
    assert_eq!(zone(&game, keeper), Zone::Battlefield);
    assert_eq!(life(&game, 1), 20, "it entered after this end step began");

    advance_to(&mut game, 1, StepType::End);
    place_and_resolve(&mut game);
    assert_eq!(life(&game, 1), 21, "and triggers at the next");
}

// RULING: Flickerwisp #3 - "Auras attached to the exiled permanent will be put
//   into their owners' graveyards. Equipment attached to the exiled permanent
//   will become unattached and remain on the battlefield. Any counters on the
//   exiled permanent will cease to exist. Once the exiled permanent returns,
//   it's considered a new object with no relation to the object that it was."
#[test]
fn flickerwisps_card_returns_as_a_new_object() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let strength = put_on_battlefield(&mut game, holy_strength(), 1);
    let wings = put_on_battlefield(&mut game, cobbled_wings(), 1);
    game.attach(strength, bears);
    game.attach(wings, bears);
    game.add_counters(bears, CounterType::PlusOnePlusOne, 1);
    let before = game.object_ref(bears).unwrap();

    flicker(&mut game, 0, &RecordingDecisionProvider::picking(0));
    assert_eq!(zone(&game, bears), Zone::Exile);
    assert_eq!(zone(&game, strength), Zone::Graveyard, "the Aura went to its owner's graveyard (CR 704.5m)");
    assert_eq!(zone(&game, wings), Zone::Battlefield);
    assert_eq!(game.battlefield[&wings].attached_to, None, "the Equipment stayed, unattached");

    advance_to(&mut game, 0, StepType::End);
    place_and_resolve(&mut game);
    assert_eq!(zone(&game, bears), Zone::Battlefield);
    assert_ne!(game.object_ref(bears), Some(before), "a new object (CR 400.7)");
    assert_eq!(game.battlefield[&bears].counter_count(CounterType::PlusOnePlusOne), 0, "its counters ceased to exist");
    assert_eq!(game.battlefield[&wings].attached_to, None);
}

// RULING: Flickerwisp #4 - "If a token is exiled this way, it will cease to
//   exist and won't return to the battlefield."
#[test]
fn a_token_flickerwisp_exiles_does_not_return() {
    let mut game = setup_two_player_game();
    game.execute_action(GameAction::CreateTokens { defs: vec![token("Soldier", 1)], controller: 1 }, &test_ctx()).unwrap();
    let soldier = tokens_named(&game, "Soldier")[0];
    flicker(&mut game, 0, &test_dp());
    assert!(game.get_object(soldier).is_err(), "it ceased to exist in exile (CR 704.5d)");

    advance_to(&mut game, 0, StepType::End);
    place_and_resolve(&mut game);
    assert!(tokens_named(&game, "Soldier").is_empty(), "nothing came back");
}

/// CR 303.4f: an Aura returning enchants what the player it enters under
/// chooses, among what it could enchant, and not as a target: an opponent's
/// hexproof creature is a legal choice (Banisher Priest's and Banishing
/// Light's Aura ruling). Two candidates, so it asks.
#[test]
fn an_aura_flickerwisp_returns_enchants_what_its_owner_chooses() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let strength = put_on_battlefield(&mut game, holy_strength(), 0);
    game.attach(strength, bears);
    let warded = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Hexproof]), 1);
    flicker(&mut game, 0, &RecordingDecisionProvider::picking(1));
    assert_eq!(zone(&game, strength), Zone::Exile, "Flickerwisp exiled the Aura");

    advance_to(&mut game, 0, StepType::End);
    let dp = RecordingDecisionProvider::picking(1);
    game.perform_sba_and_triggers(&dp).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&dp).unwrap();
        game.perform_sba_and_triggers(&dp).unwrap();
    }
    assert_eq!(zone(&game, strength), Zone::Battlefield);
    assert!(dp.kinds().iter().any(|k| k == "SelectRecipients"), "the owner was asked");
    assert_eq!(game.battlefield[&strength].attached_to, Some(warded), "the second option, the hexproof creature");
    assert_eq!(zone(&game, bears), Zone::Battlefield);
}

/// CR 303.4g: an Aura returning with nothing it could enchant stays where it
/// is, in exile.
#[test]
fn an_aura_with_nothing_to_enchant_stays_in_exile() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let strength = put_on_battlefield(&mut game, holy_strength(), 0);
    game.attach(strength, bears);
    let wisp = flicker(&mut game, 0, &RecordingDecisionProvider::picking(1));
    assert_eq!(zone(&game, strength), Zone::Exile);
    sacrifice(&mut game, bears);
    sacrifice(&mut game, wisp);

    advance_to(&mut game, 0, StepType::End);
    place_and_resolve(&mut game);
    assert_eq!(zone(&game, strength), Zone::Exile, "no creature to enchant (CR 303.4g)");
}

// ---------------------------------------------------------------------------
// 4. CR 610.3's "until"
// ---------------------------------------------------------------------------

/// Banishing Light enters for `player` and its trigger resolves, exiling
/// whatever `dp` targets.
fn banish(game: &mut GameState, player: PlayerId, dp: &dyn DecisionProvider) -> ObjectId {
    let light = put_on_battlefield(game, banishing_light(), player);
    resolve_all(game, dp);
    light
}

/// Banisher Priest's shape with no controller in its filter: "When this
/// creature enters, exile target creature until this creature leaves the
/// battlefield", returning under `under`'s control.
fn banisher(under: ReturnUnder) -> Arc<CardData> {
    CardDataBuilder::new("Banisher")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(triggered_ability(
            "When this creature enters, exile target creature until this creature leaves the battlefield.",
            whenever(
                enters(TriggerSubject::ThisObject),
                Effect::Atom(
                    Primitive::ExileUntil {
                        until: Box::new(leaves_the_battlefield(TriggerSubject::ThisObject).into()),
                        refers_to: None,
                        under,
                    },
                    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                ),
            ),
        ))
        .build()
}

/// Calix, Destiny's Hand's shape as an instant: "Exile target creature until
/// target enchantment you control leaves the battlefield."
fn calix_instant() -> Arc<CardData> {
    let yours = ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Enchantment)),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    );
    let text = "Exile target creature until target enchantment you control leaves the battlefield.";
    CardDataBuilder::new("Calix's Edict")
        .mana_cost(mtgsim::types::mana::ManaCost::build(&[ManaType::White], 0))
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            rules_text: text.into(),
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ExileUntil {
                    until: Box::new(leaves_the_battlefield(TriggerSubject::Referred).into()),
                    refers_to: Some(EffectRecipient::Target(SelectionFilter::Permanent(yours), TargetCount::Exactly(1))),
                    under: ReturnUnder::Owner,
                },
                EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            ),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
        })
        .build()
}

/// Cast `card` from `player`'s hand with exactly one white mana, its targets
/// each the one legal choice.
fn cast_for_white(game: &mut GameState, player: PlayerId, card: Arc<CardData>) -> ObjectId {
    let id = put_in_hand(game, card, player);
    game.players[player].mana_pool.add(ManaType::White, 1);
    game.cast_spell(player, id, &ManaWindowStop::new(test_dp())).expect("castable from exactly its cost");
    id
}

/// Banishing Light cast from hand with exactly its cost: it exiles the
/// opponent's creature until it leaves, and its leaving returns the card at
/// once, with no stack (CR 610.3).
// COVERS: ATOM-610.3-001
#[test]
fn banishing_light_returns_the_card_as_it_leaves() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let light = put_in_hand(&mut game, banishing_light(), 0);
    game.players[0].mana_pool.add(ManaType::White, 3);
    game.cast_spell(0, light, &ManaWindowStop::new(test_dp())).expect("castable from exactly its cost");
    game.resolve_top_of_stack(&test_dp()).unwrap();
    place_and_resolve(&mut game);
    assert_eq!(zone(&game, bears), Zone::Exile);
    assert_eq!(game.until_returns.len(), 1);

    sacrifice(&mut game, light);
    assert_eq!(zone(&game, bears), Zone::Battlefield, "back as Banishing Light's departure was dispatched");
    assert!(game.stack.is_empty() && game.pending_triggers.is_empty(), "no triggered ability, no stack");
    assert!(game.until_returns.is_empty());
    assert_eq!(get_effective_controller(&game, bears), Some(1));
}

// RULING: Banishing Light #3 - "If Banishing Light leaves the battlefield
//   before its triggered ability resolves, the target permanent won't be
//   exiled."
// COVERS: ATOM-610.3b-001
#[test]
fn banishing_light_gone_before_its_trigger_resolves_exiles_nothing() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let light = put_on_battlefield(&mut game, banishing_light(), 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(game.stack.len(), 1, "the enters trigger, targeting the creature");
    game.change_zone(light, Zone::Hand, ZoneChangeCause::Returned, &test_ctx()).unwrap();

    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(zone(&game, bears), Zone::Battlefield, "CR 610.3b: the event came after it triggered");
    assert!(game.until_returns.is_empty());
}

/// CR 610.3a's spell: "exile target creature until target enchantment you
/// control leaves the battlefield", with the enchantment destroyed in
/// response. The event has happened since the spell was cast, so nothing
/// moves.
// COVERS: ATOM-610.3a-001
#[test]
fn an_until_whose_event_happened_before_the_spell_resolved_moves_nothing() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let charm = put_on_battlefield(&mut game, card_of_type("Lucky Charm", CardType::Enchantment), 0);
    cast_for_white(&mut game, 0, calix_instant());
    sacrifice(&mut game, charm);

    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(zone(&game, bears), Zone::Battlefield);
    assert!(game.until_returns.is_empty());
}

/// Calix's shape with its enchantment still there: the exile waits on the
/// targeted enchantment, not on the spell's source.
#[test]
fn an_until_watching_a_target_returns_as_the_target_leaves() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let charm = put_on_battlefield(&mut game, card_of_type("Lucky Charm", CardType::Enchantment), 0);
    cast_for_white(&mut game, 0, calix_instant());
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(zone(&game, bears), Zone::Exile);

    sacrifice(&mut game, charm);
    assert_eq!(zone(&game, bears), Zone::Battlefield);
}

/// CR 610.3c: a creature its owner's opponent controlled when it was exiled
/// returns under its owner's control.
// COVERS: ATOM-610.3c-001
#[test]
fn an_until_return_is_under_its_owners_control() {
    let mut game = setup_two_player_game();
    let stolen = put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.continuous_effects.add(ContinuousEffect {
        duration: Duration::Indefinite,
        ..registered(stolen, Layer::Layer2Control, 200, EffectModification::SetController(PlayerRef::Player(0)))
    });
    assert_eq!(get_effective_controller(&game, stolen), Some(0));
    let priest = put_on_battlefield(&mut game, banisher(ReturnUnder::Owner), 0);
    place_and_resolve(&mut game);
    assert_eq!(zone(&game, stolen), Zone::Exile);

    sacrifice(&mut game, priest);
    assert_eq!(zone(&game, stolen), Zone::Battlefield);
    assert_eq!(get_effective_controller(&game, stolen), Some(1), "its owner's");
}

/// CR 610.3c's "unless otherwise specified": "return it under your control".
// COVERS: ATOM-610.3c-002
#[test]
fn an_until_return_that_says_your_control_is_under_yours() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let priest = put_on_battlefield(&mut game, banisher(ReturnUnder::You), 0);
    place_and_resolve(&mut game);
    assert_eq!(zone(&game, bears), Zone::Exile);

    sacrifice(&mut game, priest);
    assert_eq!(zone(&game, bears), Zone::Battlefield);
    assert_eq!(get_effective_controller(&game, bears), Some(0), "the exile's controller's");
}

/// CR 610.3d: two Banishing Lights leaving at once return their cards at
/// once, so each returned Soul Warden sees the other enter.
// COVERS: ATOM-610.3d-001
#[test]
fn two_until_returns_from_one_event_are_one_event() {
    let mut game = setup_two_player_game();
    let first = put_on_battlefield(&mut game, soul_warden(), 1);
    let second = put_on_battlefield(&mut game, soul_warden(), 1);
    place_and_resolve(&mut game);
    let before = life(&game, 1);
    let dp = RecordingDecisionProvider::picking(0);
    let lights = [banish(&mut game, 0, &dp), banish(&mut game, 0, &dp)];
    assert_eq!((zone(&game, first), zone(&game, second)), (Zone::Exile, Zone::Exile));

    let destroy = |id| GameAction::Destroy { object: id, source: DestructionSource::StateBasedAction };
    game.execute_actions(lights.iter().map(|&id| destroy(id)).collect(), &ActionContext::new(&dp)).unwrap();
    assert_eq!((zone(&game, first), zone(&game, second)), (Zone::Battlefield, Zone::Battlefield));
    place_and_resolve(&mut game);
    assert_eq!(life(&game, 1), before + 2, "each Warden saw the other enter");
}

// RULING: Banishing Light #1 - "If an Aura is exiled this way, its owner
//   chooses what it will enchant as it returns to the battlefield. An Aura put
//   onto the battlefield this way doesn't target anything (so it could be
//   attached to a permanent an opponent controls with hexproof, for example),
//   but the Aura's enchant ability restricts what it can be attached to."
#[test]
fn banishing_lights_aura_returns_enchanting_what_its_owner_chooses() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let strength = put_on_battlefield(&mut game, holy_strength(), 1);
    game.attach(strength, bears);
    let warded = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Hexproof]), 0);
    let light = banish(&mut game, 0, &RecordingDecisionProvider::picking(1));
    assert_eq!(zone(&game, strength), Zone::Exile, "the second option, the Aura");

    let dp = RecordingDecisionProvider::picking(1);
    game.execute_actions(
        vec![GameAction::Destroy { object: light, source: DestructionSource::StateBasedAction }],
        &ActionContext::new(&dp),
    )
    .unwrap();
    assert!(dp.kinds().iter().any(|k| k == "SelectRecipients"), "its owner chose");
    assert_eq!(game.battlefield[&strength].attached_to, Some(warded), "the opponent's hexproof creature");
}

// RULING: Banishing Light #1 - "If the Aura can't legally be attached to
//   anything, it remains in exile for the rest of the game."
#[test]
fn banishing_lights_aura_with_nothing_to_enchant_stays_in_exile() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let strength = put_on_battlefield(&mut game, holy_strength(), 1);
    game.attach(strength, bears);
    let light = banish(&mut game, 0, &RecordingDecisionProvider::picking(1));
    sacrifice(&mut game, bears);

    sacrifice(&mut game, light);
    assert_eq!(zone(&game, strength), Zone::Exile, "CR 303.4g");
    assert!(game.until_returns.is_empty(), "and no return is left waiting");
}

// RULING: Banishing Light #2 - "Auras attached to the exiled permanent will
//   be put into their owners' graveyards. Any Equipment will become unattached
//   and remain on the battlefield. Any counters on the exiled permanent will
//   cease to exist. When the card returns to the battlefield, it will be a new
//   object with no connection to the card that was exiled."
#[test]
fn banishing_lights_card_returns_as_a_new_object() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let strength = put_on_battlefield(&mut game, holy_strength(), 1);
    let wings = put_on_battlefield(&mut game, cobbled_wings(), 1);
    game.attach(strength, bears);
    game.attach(wings, bears);
    game.add_counters(bears, CounterType::PlusOnePlusOne, 1);
    let before = game.object_ref(bears).unwrap();
    let light = banish(&mut game, 0, &RecordingDecisionProvider::picking(0));
    assert_eq!(zone(&game, bears), Zone::Exile);
    assert_eq!(zone(&game, strength), Zone::Graveyard);
    assert_eq!(game.battlefield[&wings].attached_to, None);

    sacrifice(&mut game, light);
    assert_eq!(zone(&game, bears), Zone::Battlefield);
    assert_ne!(game.object_ref(bears), Some(before), "a new object (CR 400.7)");
    assert_eq!(game.battlefield[&bears].counter_count(CounterType::PlusOnePlusOne), 0);
}

// RULING: Banishing Light #4 - "If a token is exiled this way, it will cease
//   to exist and won't return to the battlefield."
#[test]
fn a_token_banishing_light_exiles_does_not_return() {
    let mut game = setup_two_player_game();
    game.execute_action(GameAction::CreateTokens { defs: vec![token("Soldier", 1)], controller: 1 }, &test_ctx()).unwrap();
    let soldier = tokens_named(&game, "Soldier")[0];
    let light = banish(&mut game, 0, &test_dp());
    assert!(game.get_object(soldier).is_err(), "it ceased to exist in exile (CR 704.5d)");

    sacrifice(&mut game, light);
    assert!(tokens_named(&game, "Soldier").is_empty());
    assert!(game.until_returns.is_empty());
}

/// The Waiting view lists an "until" return by what it watches and what it
/// would return.
#[test]
fn the_waiting_view_shows_an_until_return() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    banish(&mut game, 0, &test_dp());
    let waiting = mtgsim::ui::waiting::what_is_waiting(&game);
    assert_eq!(waiting.until_returns.len(), 1);
    assert!(waiting.until_returns[0].watched.as_deref().is_some_and(|w| w.contains("Banishing Light")));
    assert!(waiting.until_returns[0].returns[0].contains("Grizzly Bears"));
    assert!(waiting.until_returns[0].source.contains("Banishing Light"));
}

/// Item 234: a waiting return keeps the number its making gave it, not its
/// place in the list. The first of two Lights leaves, so the second's return
/// moves up a place, and is still number 2; a third is number 3, since a
/// number is never reused.
#[test]
fn a_waiting_return_keeps_its_number_as_the_list_changes() {
    let mut game = setup_two_player_game();
    let bears = [put_on_battlefield(&mut game, grizzly_bears(), 1), put_on_battlefield(&mut game, grizzly_bears(), 1)];
    let dp = RecordingDecisionProvider::picking(0);
    let lights = [banish(&mut game, 0, &dp), banish(&mut game, 0, &dp)];
    let rows = |game: &GameState| -> Vec<(UntilReturnId, Vec<String>)> {
        mtgsim::ui::waiting::what_is_waiting(game).until_returns.into_iter().map(|row| (row.id, row.returns)).collect()
    };
    let waiting = rows(&game);
    assert_eq!(waiting.iter().map(|(id, _)| *id).collect::<Vec<_>>(), [UntilReturnId(1), UntilReturnId(2)]);
    let second = waiting[1].1.clone();

    sacrifice(&mut game, lights[0]);
    let mut zones = bears.map(|b| zone(&game, b));
    zones.sort_by_key(|z| *z == Zone::Exile);
    assert_eq!(zones, [Zone::Battlefield, Zone::Exile], "the first Light's card back, the second's still away");
    assert_eq!(rows(&game), [(UntilReturnId(2), second)], "the second Light's, first in the list now");

    banish(&mut game, 0, &dp);
    let ids: Vec<UntilReturnId> = rows(&game).into_iter().map(|(id, _)| id).collect();
    assert_eq!(ids, [UntilReturnId(2), UntilReturnId(3)]);
}

// ---------------------------------------------------------------------------
// 5. Item 226
// ---------------------------------------------------------------------------

/// Cobbled Wings prints {2} (Scryfall, 2026-10-08); it was built at {1}.
#[test]
fn cobbled_wings_costs_two() {
    let cost = cobbled_wings().mana_cost.as_ref().map(|cost| cost.mana_value());
    assert_eq!(cost, Some(2));
}

// ---------------------------------------------------------------------------
// 6. The review: Cloudshift's shape, CR 111.8, and "until" any event
// ---------------------------------------------------------------------------

/// Cloudshift's shape as an instant: "Exile target creature you control,
/// then return that card to the battlefield under your control." No delayed
/// trigger: the return finds the card the exile moved (CR 400.7j).
fn cloudshift_shape() -> Arc<CardData> {
    let text = "Exile target creature you control, then return that card to the battlefield under your control.";
    CardDataBuilder::new("Cloud Step")
        .mana_cost(mtgsim::types::mana::ManaCost::build(&[ManaType::White], 0))
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            rules_text: text.into(),
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(Primitive::Exile, target_your_creature()),
                Effect::Atom(Primitive::ReturnToBattlefield(ReturnUnder::You), EffectRecipient::SameInstanceAs(0)),
            ]),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
        })
        .build()
}

/// The card comes back under its caster's control as a new object, which
/// Cloudshift's rulings say of the creature it flickers.
#[test]
fn an_exile_then_a_return_in_one_resolution_brings_the_card_back_under_your_control() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let before = game.object_ref(bears).unwrap();
    cast_for_white(&mut game, 0, cloudshift_shape());
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(zone(&game, bears), Zone::Battlefield);
    assert_ne!(game.object_ref(bears), Some(before), "a new object (CR 400.7)");
    assert_eq!(get_effective_controller(&game, bears), Some(0));
}

/// CR 111.8: "A token that has left the battlefield can't move to another
/// zone or come back onto the battlefield." Cloudshift's ruling: "If a token
/// is exiled this way, it will cease to exist and won't return to the
/// battlefield."
// COVERS: ATOM-111.8-001
#[test]
fn a_token_that_has_left_the_battlefield_does_not_come_back() {
    let mut game = setup_two_player_game();
    game.execute_action(GameAction::CreateTokens { defs: vec![token("Soldier", 1)], controller: 0 }, &test_ctx()).unwrap();
    let soldier = tokens_named(&game, "Soldier")[0];
    cast_for_white(&mut game, 0, cloudshift_shape());
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(zone(&game, soldier), Zone::Exile, "it stayed where it went");

    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert!(game.get_object(soldier).is_err(), "and ceased to exist (CR 111.7)");
}

/// An "until" whose event is not a leaving cannot yet say whether its event
/// happened since the ability triggered (CR 610.3b), so its resolution is
/// refused rather than guessed: "When this creature enters, exile target
/// creature until you gain life."
#[test]
fn an_until_that_cannot_answer_610_3b_is_refused() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    let gains = mtgsim::types::triggers::TriggerEvent::GainsLife {
        player: Some(PlayerRef::You),
        multiplicity: mtgsim::types::triggers::Multiplicity::PerOccurrence,
    };
    let jailer = CardDataBuilder::new("Life Jailer")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(triggered_ability(
            "When this creature enters, exile target creature until you gain life.",
            whenever(
                enters(TriggerSubject::ThisObject),
                Effect::Atom(
                    Primitive::ExileUntil { until: Box::new(gains), refers_to: None, under: ReturnUnder::Owner },
                    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                ),
            ),
        ))
        .build();
    put_on_battlefield(&mut game, jailer, 0);
    let dp = RecordingDecisionProvider::picking(0);
    game.perform_sba_and_triggers(&dp).unwrap();
    let refused = game.resolve_top_of_stack(&dp).expect_err("refused");
    assert!(refused.contains("610.3a/b"), "{refused}");
}
