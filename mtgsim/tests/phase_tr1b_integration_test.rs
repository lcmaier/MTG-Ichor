//! Phase TR-1b — the dispatch audit, and the departure frame it was built
//! after (`triggers-architecture.md` §4.10, §12's TR-1b row).
//!
//! **Item 174 first.** CR 603.10a's frame is the permanent as it was
//! immediately before the event, and one event can take several permanents.
//! The frames are captured at the batch's seam, between deciding and
//! performing, so no member's move shows another the board after an earlier
//! member left. Each board runs in both batch orders, because the order is
//! exactly what the frame must not depend on.
//!
//! **Then the audit**, on over one board per candidate set the dispatcher
//! keeps — a printed source, a zone-map card, a granted trigger, a departed
//! frame, a survivor's snapshot — where agreeing is not panicking, so each
//! test also asserts the audit ran and saw the triggers. What it says when
//! the answers differ is `engine::triggers::audit`'s unit tests; that its
//! reads change nothing is the whole-game test at the end.

use std::sync::Arc;

use mtgsim::cards::artifacts::sol_ring;
use mtgsim::cards::authoring::{another, dies, enters, triggered_ability, whenever};
use mtgsim::cards::basic_lands::{forest, plains, swamp};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_ld_cards::march_of_the_machines;
use mtgsim::cards::phase_lf_cards::humility;
use mtgsim::cards::phase_tr1_cards::{blood_artist, soul_warden, wild_growth};
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::actions::{DestructionSource, GameAction};
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, Layer};
use mtgsim::events::event::DamageTarget;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    creature_with_ability, install_trace, put_on_battlefield, registered, setup_two_player_game, test_ctx,
    vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerRef, Primitive,
};
use mtgsim::types::ids::{new_ability_id, ObjectId};
use mtgsim::types::triggers::{
    DamageRecipient, Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerSubject,
};
use mtgsim::types::zones::Zone;
use mtgsim::ui::random::RandomDecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

fn pending(game: &GameState) -> usize {
    game.pending_triggers.len()
}

/// Destroy `objects` as one event (CR 608.2f), in the order given.
fn destroy_all(game: &mut GameState, objects: &[ObjectId], source: ObjectId) {
    let batch = objects
        .iter()
        .map(|&object| GameAction::Destroy { object, source: DestructionSource::Effect(source) })
        .collect();
    game.execute_actions(batch, &test_ctx()).expect("the wipe");
}

/// An enchantment that watches creatures die, and is not one itself.
fn mourning_shrine() -> Arc<CardData> {
    CardDataBuilder::new("Mourning Shrine")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability(whenever(dies(ObjectFilter::ByType(CardType::Creature)), gain_one())))
        .build()
}

// ---------------------------------------------------------------------------
// Item 174 — the departure frame is the board before the event (CR 603.10a)
// ---------------------------------------------------------------------------

/// Humility and Blood Artist destroyed as one event. Before it Blood Artist
/// had no abilities, so its death triggers nothing, whichever of the two
/// the batch moves first. With Humility first, a frame taken at Blood
/// Artist's own move saw Humility already gone and triggered once.
// COVERS-PARTIAL: ATOM-603.10a-001
#[test]
fn a_departure_frame_does_not_see_an_earlier_member_leave() {
    for humility_first in [true, false] {
        let mut game = setup_two_player_game();
        let artist = put_on_battlefield(&mut game, blood_artist(), 0);
        let enchantment = put_on_battlefield(&mut game, humility(), 1);
        let source = put_on_battlefield(&mut game, sol_ring(), 1);
        let order = if humility_first { [enchantment, artist] } else { [artist, enchantment] };

        destroy_all(&mut game, &order, source);

        assert_eq!(game.get_object(artist).unwrap().zone, Zone::Graveyard);
        assert_eq!(pending(&game), 0, "no ability before the event (humility_first: {humility_first})");
    }
}

/// The frame's appearance, the same fault's other half: an artifact March of
/// the Machines animates dies as a creature, even when March leaves first in
/// the same event. A frame taken after March left framed it as a noncreature
/// artifact, and "whenever a creature dies" missed it.
// COVERS-PARTIAL: ATOM-603.10a-001
#[test]
fn a_departure_frame_keeps_the_type_an_earlier_member_gave_it() {
    for march_first in [true, false] {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, mourning_shrine(), 0);
        let march = put_on_battlefield(&mut game, march_of_the_machines(), 1);
        let ring = put_on_battlefield(&mut game, sol_ring(), 1);
        let source = put_on_battlefield(&mut game, sol_ring(), 0);
        let order = if march_first { [march, ring] } else { [ring, march] };

        destroy_all(&mut game, &order, source);

        assert_eq!(pending(&game), 1, "the ring died a creature (march_first: {march_first})");
    }
}

// ---------------------------------------------------------------------------
// CR 603.2c — one ability, one trigger per event, across a survivor's two lists
// ---------------------------------------------------------------------------

/// A survivor of a batch that takes a look-back snapshot is asked through two
/// lists: its list from before for look-back arms, its list now for the rest.
/// One ability whose look-back arm ("dies") and other arm ("put into a
/// graveyard from anywhere") both match one death triggers once for it, not
/// once per list; the first arm in the ability's order is the one bound.
#[test]
fn one_ability_triggers_once_per_death_across_a_survivors_two_lists() {
    let mut game = setup_two_player_game();
    let two_arms = TriggerDef {
        condition: TriggerCondition::AnyOf(vec![
            dies(a_creature()).into(),
            TriggerEvent::ZoneChange {
                subject: TriggerSubject::Filter(a_creature()),
                from: None,
                to: Some(Zone::Graveyard),
                cause: None,
                owner: None,
                multiplicity: Multiplicity::PerOccurrence,
            },
        ]),
        intervening_if: None,
        limit: None,
        effect: gain_one(),
    };
    let watcher = put_on_battlefield(&mut game, creature_with_ability("Twofold Mourner", 1, 1, triggered_ability(two_arms)), 0);
    // The granter is the source of an ability list's row, so the wipe that
    // takes it takes a look-back snapshot of the watcher.
    let granter = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    let bystander = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    grant(&mut game, granter, bystander, enters(another(a_creature())));
    let source = put_on_battlefield(&mut game, sol_ring(), 1);

    destroy_all(&mut game, &[granter, bear], source);

    assert_eq!(pending(&game), 2, "the granter's death and the bear's, once each");
    assert!(game.pending_triggers.iter().all(|t| t.origin.source() == watcher));
    assert!(game.pending_triggers.iter().all(|t| t.binding.event.0 == 0), "the first arm, dies");
}

// ---------------------------------------------------------------------------
// The audit, on over each candidate set (triggers-architecture.md §4.10)
// ---------------------------------------------------------------------------

/// A two-player board with the audit on from its first dispatch.
fn audited_game() -> GameState {
    let mut game = setup_two_player_game();
    game.enable_dispatch_audit();
    game
}

/// The audit ran, and agreed with the dispatcher on `triggers` triggers in
/// all. Agreeing is not panicking; this is the guard that it was asked.
fn assert_audited(game: &GameState, triggers: u64) {
    let (dispatches, agreed) = game.dispatch_audit_counts().expect("the audit is on");
    assert!(dispatches > 0, "the audit answered no dispatch");
    assert_eq!(agreed, triggers, "the triggers both answers held");
}

fn deal(game: &mut GameState, source: ObjectId, target: DamageTarget, amount: u64) {
    game.execute_action(
        GameAction::DealDamage { source, target, amount, is_combat: true, unpreventable: false },
        &test_ctx(),
    )
    .expect("dealing damage");
}

fn a_creature() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Creature)
}

/// Dread's shape, as `phase_tr1_integration_test.rs` builds it: a damage
/// trigger that functions on the battlefield, and a "from anywhere" one that
/// functions everywhere (CR 113.6k).
fn dread_shaped() -> Arc<CardData> {
    let damage = TriggerEvent::DamageDealt {
        source: a_creature().into(),
        recipient: DamageRecipient::Player(Some(PlayerRef::You)),
        combat: None,
        multiplicity: Multiplicity::PerOccurrence,
    };
    let from_anywhere = TriggerEvent::ZoneChange {
        subject: TriggerSubject::This,
        from: None,
        to: Some(Zone::Graveyard),
        cause: None,
        owner: None,
        multiplicity: Multiplicity::PerOccurrence,
    };
    CardDataBuilder::new("Incarnate Menace")
        .card_type(CardType::Creature)
        .power_toughness(6, 6)
        .ability(triggered_ability(whenever(damage, gain_one())))
        .ability(triggered_ability(whenever(from_anywhere, gain_one())))
        .build()
}

/// A Layer 6 grant of `event`'s trigger from `granter` to `carrier`, for as
/// long as the granter is on the battlefield.
fn grant(game: &mut GameState, granter: ObjectId, carrier: ObjectId, event: impl Into<TriggerEvent>) {
    let mut granted = triggered_ability(whenever(event, gain_one()));
    granted.id = new_ability_id();
    game.continuous_effects.add(ContinuousEffect {
        duration: Duration::WhileSourceOnBattlefield,
        affected_objects: ObjectSet::Fixed(vec![carrier]),
        ..registered(granter, Layer::Layer6Ability, 100, EffectModification::GrantAbility(Box::new(granted)))
    });
}

/// A printed source on the battlefield (`trigger_sources`): Soul Warden sees a
/// creature enter.
#[test]
fn the_audit_agrees_over_a_printed_source() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, soul_warden(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);

    assert_eq!(pending(&game), 1);
    assert_audited(&game, 1);
}

/// A card in a graveyard whose trigger functions there (`zone_trigger_sources`)
/// — Dread's two boards: its "from anywhere" trigger fires after the move, and
/// its damage trigger is not asked from the graveyard (F1).
#[test]
fn the_audit_agrees_over_a_zone_map_card() {
    let mut game = audited_game();
    let dread = put_on_battlefield(&mut game, dread_shaped(), 0);
    let attacker = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);

    destroy_all(&mut game, &[dread], source);
    deal(&mut game, attacker, DamageTarget::Player(0), 2);

    assert_eq!(game.get_object(dread).unwrap().zone, Zone::Graveyard);
    assert_eq!(pending(&game), 1, "the from-anywhere trigger, and not the damage one");
    assert_audited(&game, 1);
}

/// A trigger a Layer 6 row grants (`RegistryScopeSummary`'s granted leg): the
/// carrier printed nothing, and triggers.
#[test]
fn the_audit_agrees_over_a_granted_trigger() {
    let mut game = audited_game();
    let carrier = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let granter = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    grant(&mut game, granter, carrier, enters(another(a_creature())));

    put_on_battlefield(&mut game, grizzly_bears(), 1);

    assert_eq!(pending(&game), 1);
    assert!(game.pending_triggers.iter().all(|t| t.origin.source() == carrier));
    assert_audited(&game, 1);
}

/// The frames a window's departures carry (CR 603.10a): Blood Artist dying
/// beside two creatures, and item 174's board in both orders.
#[test]
fn the_audit_agrees_over_a_departed_frame() {
    let mut game = audited_game();
    let artist = put_on_battlefield(&mut game, blood_artist(), 0);
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);
    destroy_all(&mut game, &[artist, a, b], source);
    assert_eq!(pending(&game), 3);
    assert_audited(&game, 3);

    for humility_first in [true, false] {
        let mut game = audited_game();
        let artist = put_on_battlefield(&mut game, blood_artist(), 0);
        let enchantment = put_on_battlefield(&mut game, humility(), 1);
        let source = put_on_battlefield(&mut game, sol_ring(), 1);
        let order = if humility_first { [enchantment, artist] } else { [artist, enchantment] };
        destroy_all(&mut game, &order, source);
        assert_eq!(pending(&game), 0, "humility_first: {humility_first}");
        assert_audited(&game, 0);
    }
}

/// A survivor's list from before the batch (`LookBackSnapshot`, item 167):
/// Humility leaving beside a creature while Blood Artist lives, and a grant
/// whose source dies in the wipe.
#[test]
fn the_audit_agrees_over_a_survivors_snapshot() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, blood_artist(), 0);
    let enchantment = put_on_battlefield(&mut game, humility(), 1);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let source = put_on_battlefield(&mut game, sol_ring(), 1);
    destroy_all(&mut game, &[enchantment, bear], source);
    assert_eq!(pending(&game), 0, "no ability before the wipe");
    assert_audited(&game, 0);

    let mut game = audited_game();
    let carrier = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let granter = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    grant(&mut game, granter, carrier, dies(another(a_creature())));
    let source = put_on_battlefield(&mut game, sol_ring(), 1);
    destroy_all(&mut game, &[granter, bear], source);
    assert_eq!(pending(&game), 2, "the granter and the bear, off the list from before");
    assert_audited(&game, 2);
}

/// Its reads are invisible: a whole game with the audit on plays, counts and
/// traces exactly what the same game with it off does — every diagnostics
/// row, every event, every trace line. The deck carries the three pooled
/// trigger cards so the audit has triggers to agree on.
#[test]
fn an_audited_game_counts_and_traces_what_an_unaudited_one_does() {
    let play = |audit: bool| {
        let registry = CardRegistry::performance_pool();
        let mut deck: Vec<Arc<CardData>> = registry
            .card_names()
            .iter()
            .cycle()
            .take(30)
            .filter_map(|n| registry.create(n).ok())
            .collect();
        deck.extend([soul_warden(), soul_warden(), blood_artist(), blood_artist(), wild_growth(), wild_growth()]);
        for _ in 0..8 {
            deck.extend([plains(), swamp(), forest()]);
        }
        let mut g = Game::new(GameConfig::test(), vec![deck; 2]).unwrap();
        if audit {
            g.state.enable_dispatch_audit();
        }
        g.reseed(174);
        let trace = install_trace(&mut g.state, "tr-1b");
        let dp = RandomDecisionProvider::seeded(175);
        g.setup(&dp).unwrap();
        for _ in 0..12 {
            if g.is_over() {
                break;
            }
            g.run_turn(&dp).unwrap();
        }
        let counts = format!("{:?}", g.state.diagnostics);
        (counts, g.event_log_snapshot(), trace.lines(), g.state.dispatch_audit_counts())
    };

    let (counts, events, trace, _) = play(false);
    let (audited_counts, audited_events, audited_trace, audit) = play(true);

    let (dispatches, triggers) = audit.expect("the audit was on");
    assert!(dispatches > 0 && triggers > 0, "{triggers} triggers over {dispatches} dispatches");
    assert_eq!(counts, audited_counts, "every diagnostics row");
    assert_eq!(events, audited_events, "every event");
    assert_eq!(trace, audited_trace, "every trace line");
}

// ---------------------------------------------------------------------------
// The dispatcher's counts (§4.10, decision 4)
// ---------------------------------------------------------------------------

/// The three rows count what they say. Soul Warden entering is a window its
/// own mask reads, so it passes the gate, asks one candidate and matches
/// nothing ("another creature"); a bear entering does the same and matches
/// once; a tap is a kind no source reads and never passes the gate.
#[test]
fn the_dispatchers_counts_are_windows_past_the_gate_candidates_and_matches() {
    let mut game = setup_two_player_game();
    let counts = |game: &GameState| {
        let d = &game.diagnostics;
        (d.trigger_windows(), d.trigger_candidates(), d.trigger_matches())
    };
    assert_eq!(counts(&game), (0, 0, 0));

    put_on_battlefield(&mut game, soul_warden(), 0);
    assert_eq!(counts(&game), (1, 1, 0), "its own entry: asked, and refused");

    let bear = put_on_battlefield(&mut game, grizzly_bears(), 1);
    assert_eq!(counts(&game), (2, 2, 1), "the bear's entry: asked, and matched");

    game.execute_action(GameAction::Tap { object: bear }, &test_ctx()).unwrap();
    assert_eq!(counts(&game), (2, 2, 1), "a tap no source reads stops at the gate");
    assert_eq!(pending(&game), 1);
}
