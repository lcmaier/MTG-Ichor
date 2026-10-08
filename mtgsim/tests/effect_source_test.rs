//! `codebase-state.md` item 223's event half: an ability's effects belong to
//! the object that has the ability (CR 113.7, 120.2b, 608.2h), not to the
//! stack object CR 608.2n removes.
//!
//! 1. The life an ability gains is the permanent's, and the log says so.
//! 2. Lifelink and Circle of Protection: Red read the permanent that deals an
//!    ability's damage.
//! 3. A source that left before its ability's damage is dealt deals it as it
//!    last existed (CR 608.2h, 702.15c): the owner's Dragonhawk board, with
//!    Loxodon Warhammer, over a fixture with only Dragonhawk's end-step
//!    damage. Its "for each of those cards that are still exiled" names the
//!    cards the creating ability exiled, which is TR-3b's `refs`.
//!
//! The boards resolve real activations and real triggers, so the stack
//! object exists and is gone by the time anything reads the source.

use std::sync::Arc;

use mtgsim::cards::authoring::{at_beginning_of, enters, triggered_ability, whenever, Whose};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lh_cards::loxodon_warhammer;
use mtgsim::cards::phase_rd_cards::circle_of_protection_red;
use mtgsim::cards::phase_tr1_cards::soul_warden;
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::layers::types::{EffectModification, Layer};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::state::replacement_effects::RegisteredReplacementEffect;
use mtgsim::test_support::{
    creature_with_ability, fill_library, put_on_battlefield, put_on_battlefield_under, registered, setup_two_player_game,
    test_ctx, test_dp, vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerGroup, PlayerRef, PlayerSet, Primitive,
};
use mtgsim::types::ids::{AbilityId, ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::replacement::{EventPattern, ReplacementDef, RetargetSpec, Rewrite, SourcePattern};
use mtgsim::types::triggers::{
    DamageRecipient, DelayedDuration, DelayedTriggerTemplate, DelayedTurn, Multiplicity, TriggerEvent, TriggerSubject,
};
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::display::format_event_log;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

/// "This creature deals `n` damage to each opponent."
fn damage_each_opponent(n: u64) -> Effect {
    Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::Fixed(n), unpreventable: false },
        EffectRecipient::EachOf(PlayerGroup::set(PlayerSet::Opponents)),
    )
}

/// A red 1/1: "{T}: This creature deals 1 damage to each opponent." Red, so
/// Circle of Protection: Red may choose it; no lifelink of its own, so
/// whatever lifelink it has comes from the Warhammer.
fn pinger() -> Arc<CardData> {
    CardDataBuilder::new("Ember Pinger")
        .card_type(CardType::Creature)
        .color(Color::Red)
        .power_toughness(1, 1)
        .ability(AbilityDef {
            rules_text: "{T}: This creature deals 1 damage to each opponent.".into(),
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::TapSelf],
            effect: damage_each_opponent(1),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
        })
        .build()
}

fn equip(game: &mut GameState, equipment: ObjectId, host: ObjectId) {
    game.execute_action(GameAction::Attach { attachment: equipment, host }, &test_ctx()).unwrap();
}

/// Activate `source`'s first ability for `player` and resolve it.
fn activate_and_resolve(game: &mut GameState, player: PlayerId, source: ObjectId) {
    game.activate_ability(player, source, 0, &test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
}

fn life_changes(game: &GameState) -> Vec<(PlayerId, i64, Option<ObjectId>)> {
    game.recorded_events()
        .records()
        .iter()
        .filter_map(|r| match r.event {
            GameEvent::LifeChanged { player_id, old, new, source, .. } => Some((player_id, new - old, source)),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 1. The life an ability gains
// ---------------------------------------------------------------------------

/// #231's `fizzle_log.png`: Blood Artist's gain was logged as `(source: #22)`,
/// its trigger's stack object, gone by the time the log read it. Soul
/// Warden's gain is Soul Warden's (CR 113.7), and its line names it.
#[test]
fn an_abilitys_life_gain_is_logged_as_the_permanents() {
    let mut game = setup_two_player_game();
    let warden = put_on_battlefield(&mut game, soul_warden(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(life_changes(&game), vec![(0, 1, Some(warden))]);
    let line = format!("LifeChanged: P0 20 -> 21 (source: Soul Warden ({warden}))");
    let log = format_event_log(&game);
    assert!(log.contains(&line), "no {line:?} in {log:#?}");
}

// ---------------------------------------------------------------------------
// 2. The permanent deals an ability's damage
// ---------------------------------------------------------------------------

/// CR 120.2b, 608.2h: "This creature deals 1 damage" is the creature's
/// damage, so the Warhammer's lifelink on it gains its controller that much
/// (CR 702.15b). The ability's stack object never had lifelink: the grant is
/// on the creature.
#[test]
fn lifelink_reads_the_permanent_whose_ability_deals_the_damage() {
    let mut game = setup_two_player_game();
    let pinger = put_on_battlefield(&mut game, pinger(), 0);
    let warhammer = put_on_battlefield(&mut game, loxodon_warhammer(), 0);
    equip(&mut game, warhammer, pinger);

    activate_and_resolve(&mut game, 0, pinger);

    assert_eq!(life(&game, 1), 19);
    assert_eq!(life(&game, 0), 21, "the pinger has lifelink, and it dealt the damage");
    assert_eq!(life_changes(&game), vec![(1, -1, Some(pinger)), (0, 1, Some(pinger))], "the loss and the gain both the pinger's");
}

/// CR 609.7a: "If the player chooses a permanent, the effect will apply to
/// the next damage dealt by that permanent, regardless of whether it's
/// combat damage or damage dealt as the result of a spell or ability." The
/// Circle chooses the pinger, and the pinger's ability's damage is the
/// pinger's.
#[test]
fn circle_of_protection_red_prevents_an_abilitys_damage_from_the_chosen_permanent() {
    let mut game = setup_two_player_game();
    // Oldest, so the CR 609.7a prompt lists it first.
    let pinger = put_on_battlefield(&mut game, pinger(), 1);
    let circle_card = circle_of_protection_red();
    let circle = put_on_battlefield(&mut game, circle_card.clone(), 0);
    let ctx = ResolutionContext { ability_source: game.object_ref(circle), ..ResolutionContext::untargeted(circle, 0) };
    game.resolve_effect(&circle_card.abilities[0].effect, &ctx, &RecordingDecisionProvider::picking(0)).unwrap();
    let chosen = game.replacement_effects.iter().find_map(|row| match &row.def.pattern {
        EventPattern::DealDamage { source: Some(pattern), .. } => pattern.object,
        _ => None,
    });
    assert_eq!(chosen, Some(pinger), "CR 609.7a's choice, made as the effect was created");

    activate_and_resolve(&mut game, 1, pinger);

    assert_eq!(life(&game, 0), 20, "the chosen permanent's damage, prevented");
    assert_eq!(game.replacement_effects.iter().count(), 0, "and the shield is spent (CR 615.8)");
}

// ---------------------------------------------------------------------------
// 3. A source that left deals the damage as it last existed
// ---------------------------------------------------------------------------

/// A 4/4 with only Dragonhawk, Fate's Tempest's end-step damage: "When this
/// creature enters, at the beginning of your next end step, this creature
/// deals 2 damage to each opponent."
fn dragonhawk_shaped() -> Arc<CardData> {
    tempest_fixture(damage_each_opponent(2))
}

/// The same creature, whose end-step damage is `damage`.
fn tempest_fixture(damage: Effect) -> Arc<CardData> {
    let end_step_damage = DelayedTriggerTemplate {
        def: Arc::new(whenever(at_beginning_of(StepType::End, Whose::Yours), damage)),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::Any,
    };
    let create = Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(end_step_damage)), EffectRecipient::Controller);
    CardDataBuilder::new("Tempest Fixture")
        .card_type(CardType::Creature)
        .power_toughness(4, 4)
        .ability(triggered_ability(
            "When this creature enters, at the beginning of your next end step, this creature deals 2 damage to each opponent.",
            whenever(enters(TriggerSubject::ThisObject), create),
        ))
        .build()
}

/// `card` entered under player 0's control, owned by `owner`, and its
/// enters trigger resolved, so the delayed trigger is waiting.
fn enter_and_wait(game: &mut GameState, card: Arc<CardData>, owner: PlayerId) -> ObjectId {
    let hawk = put_on_battlefield_under(game, card, owner, 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.delayed_triggers.len(), 1, "the end-step damage is waiting");
    hawk
}

/// The fixture and a Warhammer on player 0's side, the delayed trigger
/// waiting and the Warhammer attached.
fn dragonhawk_board(game: &mut GameState) -> (ObjectId, ObjectId) {
    let hawk = enter_and_wait(game, dragonhawk_shaped(), 0);
    let warhammer = put_on_battlefield(game, loxodon_warhammer(), 0);
    equip(game, warhammer, hawk);
    (hawk, warhammer)
}

fn dies(game: &mut GameState, creature: ObjectId) {
    game.change_zone(creature, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx()).unwrap();
}

/// Walk the turn machinery to player 0's end step; libraries are filled so a
/// draw step on the way is not a loss.
fn to_the_end_step(game: &mut GameState) {
    for p in 0..game.num_players() {
        fill_library(game, p, 10);
    }
    for _ in 0..20 {
        game.advance_turn(&test_ctx()).expect("advancing");
        if game.active_player == 0 && game.phase.step == Some(StepType::End) {
            game.perform_sba_and_triggers(&test_dp()).unwrap();
            return;
        }
    }
    panic!("player 0's end step never began");
}

/// Equipped as the trigger resolves: the creature deals the damage, and has
/// lifelink then (CR 702.15b).
#[test]
fn an_ability_of_a_creature_equipped_as_it_resolves_has_lifelink() {
    let mut game = setup_two_player_game();
    dragonhawk_board(&mut game);

    to_the_end_step(&mut game);
    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(life(&game, 1), 18);
    assert_eq!(life(&game, 0), 22);
}

/// Died equipped before its delayed trigger triggered: it deals the damage
/// from the graveyard as it last existed on the battlefield, Warhammer and
/// all (CR 608.2h, 702.15c, 113.7a's "the source can still perform the
/// action even though it no longer exists"). The frame is the one the
/// delayed trigger kept as its source left (`DelayedTrigger::source_frame`).
#[test]
fn a_creature_that_died_equipped_before_its_trigger_deals_the_damage_with_lifelink() {
    let mut game = setup_two_player_game();
    let (hawk, _) = dragonhawk_board(&mut game);
    dies(&mut game, hawk);
    assert!(!game.battlefield.contains_key(&hawk));

    to_the_end_step(&mut game);
    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(life(&game, 1), 18);
    assert_eq!(life(&game, 0), 22, "lifelink, off the frame it left with");
}

/// The same death after the trigger is on the stack: the frame is the one
/// its stack entry kept (`triggers-architecture.md` §6.1).
#[test]
fn a_creature_that_died_equipped_in_response_deals_the_damage_with_lifelink() {
    let mut game = setup_two_player_game();
    let (hawk, _) = dragonhawk_board(&mut game);

    to_the_end_step(&mut game);
    assert_eq!(game.stack.len(), 1, "the delayed trigger, waiting to resolve");
    dies(&mut game, hawk);
    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(life(&game, 1), 18);
    assert_eq!(life(&game, 0), 22);
}

/// Equipped when the trigger was created, but the Warhammer moved off before
/// the creature died: it had no lifelink as it left, which is the moment
/// that counts, so the damage gains nothing.
#[test]
fn a_creature_unequipped_before_it_died_has_no_lifelink() {
    let mut game = setup_two_player_game();
    let (hawk, warhammer) = dragonhawk_board(&mut game);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    equip(&mut game, warhammer, bears);
    assert!(!mtgsim::oracle::characteristics::has_keyword(&game, hawk, KeywordFlag::Lifelink));
    dies(&mut game, hawk);

    to_the_end_step(&mut game);
    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(life(&game, 1), 18, "it still deals the damage (CR 113.7a)");
    assert_eq!(life(&game, 0), 20, "and had no lifelink as it left");
}

// ---------------------------------------------------------------------------
// The other readers of a source that left: deathtouch, a shield's property,
// "that source's controller", and a trigger on the damage
// ---------------------------------------------------------------------------

/// `object` has `modification` until end of turn, from a row about it alone,
/// which ends as it leaves (CR 400.7). Once it has died, only the frame it
/// left with still has what the row gave it.
fn give(game: &mut GameState, object: ObjectId, layer: Layer, modification: EffectModification) {
    let ts = game.allocate_timestamp();
    game.continuous_effects.add(registered(object, layer, ts, modification));
}

fn make_red(game: &mut GameState, object: ObjectId) {
    give(game, object, Layer::Layer5Color, EffectModification::SetColors([Color::Red].into_iter().collect()));
}

/// A resolution's replacement row around `player` that outlives the
/// damage's source, since it names no chosen source.
fn shield_around(game: &mut GameState, player: PlayerId, def: ReplacementDef) {
    let anchor = put_on_battlefield(game, vanilla_creature(1, 1, &[]), player);
    let turn = game.turn_number;
    game.replacement_effects.add(RegisteredReplacementEffect {
        id: 0,
        source: anchor,
        controller: player,
        duration: Duration::UntilEndOfTurn,
        created_on_turn: turn,
        targets: Vec::new(),
        def: def.affecting_players(PlayerSet::You),
    });
}

/// CR 702.2e: deathtouch is read off the last known information too. The
/// creature had deathtouch from a row about it alone, and died; its 2 damage
/// to a 5/5 is still lethal (CR 702.2b).
#[test]
fn a_creature_that_died_with_deathtouch_deals_its_ability_damage_with_deathtouch() {
    let mut game = setup_two_player_game();
    let their_creatures = ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
    );
    let damage = Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::Fixed(2), unpreventable: false },
        EffectRecipient::FilteredPermanents(their_creatures),
    );
    let hawk = enter_and_wait(&mut game, tempest_fixture(damage), 0);
    let giant = put_on_battlefield(&mut game, vanilla_creature(5, 5, &[]), 1);
    give(&mut game, hawk, Layer::Layer6Ability, EffectModification::GrantKeywordFlag(KeywordFlag::Deathtouch));
    dies(&mut game, hawk);

    to_the_end_step(&mut game);
    game.resolve_top_of_stack(&test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();

    assert_eq!(game.get_object(giant).unwrap().zone, Zone::Graveyard, "2 damage from a deathtouch source");
}

/// CR 609.7b: a shield rechecks the source's properties when it would deal
/// damage, and a source that left is what it last was. The creature was red
/// by a row about it alone, which ended as it died; it deals the damage red.
#[test]
fn a_shield_against_red_sources_reads_a_source_that_left_as_it_last_was() {
    let mut game = setup_two_player_game();
    let hawk = enter_and_wait(&mut game, dragonhawk_shaped(), 0);
    make_red(&mut game, hawk);
    shield_around(
        &mut game,
        1,
        ReplacementDef::new(
            EventPattern::DealDamage {
                source: Some(SourcePattern::matching(ObjectFilter::ByColor(Color::Red))),
                combat: None,
            },
            ObjectSet::NO_OBJECTS,
            Rewrite::Prevent,
        ),
    );
    dies(&mut game, hawk);

    to_the_end_step(&mut game);
    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(life(&game, 1), 20, "a red source's damage, prevented");
}

/// Reflect Damage's "that source's controller": player 1 owns the creature
/// and player 0 controlled it as it died, so the damage goes to player 0
/// (CR 608.2h). Its card in player 1's graveyard has no controller, and the
/// answer was its owner before.
#[test]
fn that_sources_controller_is_the_controller_it_last_had() {
    let mut game = setup_two_player_game();
    let hawk = enter_and_wait(&mut game, dragonhawk_shaped(), 1);
    shield_around(
        &mut game,
        1,
        ReplacementDef::new(
            EventPattern::DealDamage { source: Some(SourcePattern::chosen()), combat: None },
            ObjectSet::NO_OBJECTS,
            Rewrite::Retarget(RetargetSpec::ToDamageSourceController),
        ),
    );
    dies(&mut game, hawk);
    assert_eq!(game.get_object(hawk).unwrap().owner, 1);

    to_the_end_step(&mut game);
    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(life(&game, 1), 20);
    assert_eq!(life(&game, 0), 18, "dealt to the player who controlled it");
}

/// "Whenever a red source deals damage to you, you gain 1 life", over the
/// damage of a creature that was red as it died: the record carries what
/// dealt the damage, so the trigger reads what the damage's results read.
#[test]
fn a_trigger_on_the_damage_reads_a_source_that_left_as_it_last_was() {
    let mut game = setup_two_player_game();
    let hawk = enter_and_wait(&mut game, dragonhawk_shaped(), 0);
    make_red(&mut game, hawk);
    let red_damage_to_you = TriggerEvent::DamageDealt {
        source: TriggerSubject::Filter(ObjectFilter::ByColor(Color::Red)),
        recipient: DamageRecipient::Player(Some(PlayerRef::You)),
        combat: None,
        multiplicity: Multiplicity::PerOccurrence,
    };
    let gain_one = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller);
    let ward = creature_with_ability("Ember Ward", 1, 1, triggered_ability("", whenever(red_damage_to_you, gain_one)));
    put_on_battlefield(&mut game, ward, 1);
    dies(&mut game, hawk);

    to_the_end_step(&mut game);
    game.resolve_top_of_stack(&test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();

    assert_eq!(game.stack.len(), 1, "the ward triggered on a red source's damage");
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(life(&game, 1), 19);
}
