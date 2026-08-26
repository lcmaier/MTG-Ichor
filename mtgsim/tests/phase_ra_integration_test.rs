//! Phase RA integration tests: the event spine.
//!
//! RA carries no replacement behavior. What it owes is that every mutation a
//! CR 614 replacement or a CR 603 trigger could care about is proposed through
//! one chokepoint and recorded once, with enough on the record to tell the
//! cases apart — drawn from tutored, destroyed from sacrificed, countered from
//! resolved. These tests pin the *record*, which is the part that has no other
//! consumer until Phase RB and would otherwise rot unwatched.

use std::sync::Arc;

use mtgsim::cards::{alpha, basic_lands, phase5_pre_cards};
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeCause};
use mtgsim::engine::layers::types::EffectiveCharacteristics;
use mtgsim::events::event::{BatchId, GameEvent, ResolutionStamp};
use mtgsim::objects::card_data::{
    AbilityDef, AbilityType, CardData, CardDataBuilder,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    fill_library, put_in_hand, put_on_battlefield, setup_two_player_game, test_ctx,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, PermanentFilter, PlayerRef, Primitive,
    SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{new_ability_id, ObjectId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// A 2/2 creature carrying Glorious Anthem's text, so it pumps *itself*.
///
/// The point of the self-reference: `register_static_effects` puts the row in
/// the registry at ETB with this creature as the source, and
/// `cleanup_zone_state` retires it the instant the creature leaves (CR 611.2a).
/// So its effective P/T on the battlefield is unreachable a moment later — which
/// is what makes it the right probe for *when* the LKI frame is captured.
fn self_anthem_creature() -> Arc<CardData> {
    CardDataBuilder::new("Standard Bearer")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::ModifyPowerToughness(
                    AmountExpr::Fixed(1),
                    AmountExpr::Fixed(1),
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::FilteredPermanents(PermanentFilter::And(
                    Box::new(PermanentFilter::ByType(CardType::Creature)),
                    Box::new(PermanentFilter::ByController(PlayerRef::You)),
                )),
            ),
        })
        .build()
}

/// Every `ZoneChange` in the log, as `(object, from, to, cause)`.
fn zone_changes(game: &GameState) -> Vec<(ObjectId, Zone, Zone, ZoneChangeCause)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::ZoneChange { object_id, from, to, cause, .. } => {
                Some((*object_id, *from, *to, *cause))
            }
            _ => None,
        })
        .collect()
}

/// The LKI frame on the single `ZoneChange` that moved `id`.
fn lki_for(game: &GameState, id: ObjectId) -> Option<&EffectiveCharacteristics> {
    let mut found = None;
    for e in game.events.events() {
        if let GameEvent::ZoneChange { object_id, lki, .. } = e {
            if *object_id == id {
                assert!(found.is_none(), "expected exactly one zone change for {}", id);
                found = Some(lki.as_deref());
            }
        }
    }
    found.expect("no zone change for that object")
}

// ---------------------------------------------------------------------------
// CR 603.10a — the look-back frame
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-603.10a-002
// Builds the look-back half: the record of a permanent leaving the battlefield
// carries its characteristics as of an instant before it left, which is what
// lets a death trigger be found on an object that is no longer a permanent. The
// atom's other half — the trigger actually firing — is CR 603 and Phase 7.
#[test]
fn test_the_lki_frame_is_the_permanent_as_it_last_existed() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, self_anthem_creature(), 0);

    // 2/2 printed, 3/3 on the battlefield under its own anthem.
    assert_eq!(
        mtgsim::oracle::characteristics::get_effective_power(&game, id),
        Some(3),
    );

    game.change_zone(id, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx())
        .unwrap();

    let lki = lki_for(&game, id).expect("a permanent leaving the battlefield has a frame");
    assert_eq!(
        (lki.power, lki.toughness),
        (Some(3), Some(3)),
        "CR 603.10a reads the object as it existed before the event, not as printed",
    );

    // And the point of capturing it *there*: the answer is gone a moment later.
    // CR 611.2a retired the row the creature's own static ability generated the
    // instant it left, so nothing downstream could reconstruct this.
    assert_eq!(
        mtgsim::oracle::characteristics::get_effective_power(&game, id),
        Some(2),
        "after the move the layer walk answers about a graveyard card",
    );
}

#[test]
fn test_the_lki_frame_carries_the_abilities_the_object_had() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, self_anthem_creature(), 0);

    game.change_zone(id, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx())
        .unwrap();

    let lki = lki_for(&game, id).unwrap();
    assert_eq!(
        lki.abilities.len(),
        1,
        "a leaves-the-battlefield trigger is looked for on the frame, so the \
         frame has to carry the ability list",
    );
    assert_eq!(lki.controller, 0, "and who controlled it, which the graveyard does not record");
}

#[test]
fn test_a_move_that_never_touched_the_battlefield_has_no_frame() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 3);

    let drawn = game.draw_card(0, &test_ctx()).unwrap().unwrap();

    assert!(
        lki_for(&game, drawn).is_none(),
        "nothing left the battlefield, so there is nothing to look back at",
    );
}

// ---------------------------------------------------------------------------
// The cause reaches the record
// ---------------------------------------------------------------------------

#[test]
fn test_the_zone_change_carries_the_reason_the_engine_moved_it() {
    // Same `(from, to)`, different facts. 1,287 printed cards trigger on "dies"
    // and 278 want "sacrifices" specifically, and no pair of zones can tell them
    // apart.
    let mut game = setup_two_player_game();
    let destroyed = put_on_battlefield(&mut game, self_anthem_creature(), 0);
    let sacrificed = put_on_battlefield(&mut game, self_anthem_creature(), 0);

    game.change_zone(destroyed, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx())
        .unwrap();
    game.change_zone(sacrificed, Zone::Graveyard, ZoneChangeCause::Sacrificed, &test_ctx())
        .unwrap();

    assert_eq!(
        zone_changes(&game),
        vec![
            (destroyed, Zone::Battlefield, Zone::Graveyard, ZoneChangeCause::Destroyed),
            (sacrificed, Zone::Battlefield, Zone::Graveyard, ZoneChangeCause::Sacrificed),
        ],
    );
}

#[test]
fn test_a_land_play_is_a_zone_change_with_a_reason() {
    // `play_land` wrote straight to `move_object` until RA-3 — a fourth,
    // undocumented bypass of the chokepoint, and the single most frequent zone
    // change in the game. `ZoneChangeCause::PlayedAsLand` had no call site at
    // all, which is how it went unnoticed.
    let mut game = setup_two_player_game();
    let land = put_in_hand(&mut game, basic_lands::forest(), 0);
    game.phase.phase_type = mtgsim::state::game_state::PhaseType::Precombat;

    game.play_land(0, land, Zone::Hand, &test_ctx()).unwrap();

    assert_eq!(
        zone_changes(&game),
        vec![(land, Zone::Hand, Zone::Battlefield, ZoneChangeCause::PlayedAsLand)],
    );
}

#[test]
fn test_a_stale_zone_change_proposal_is_loud() {
    // The proposal describes a board. Performing it against a different one is
    // a caller bug, and RB will match replacement effects on `from` — so a
    // stale value is a wrong match rather than a cosmetic mismatch.
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, self_anthem_creature(), 0);
    game.change_zone(id, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx())
        .unwrap();

    let stale = game.execute_action(
        GameAction::ZoneChange {
            object: id,
            from: Zone::Battlefield,
            to: Zone::Exile,
            cause: ZoneChangeCause::Exiled,
        },
        &test_ctx(),
    );
    assert!(stale.is_err(), "it is in the graveyard, not on the battlefield");
    assert_eq!(zone_changes(&game).len(), 1, "and nothing was announced");
}

// ---------------------------------------------------------------------------
// CR 601.2 — a rewind is not an event
// ---------------------------------------------------------------------------

#[test]
fn test_a_failed_cast_announces_nothing() {
    // RA-1 took cast rollbacks off the chokepoint and tagged them
    // `// CAST-ROLLBACK:`, but `move_object` still emitted a `ZoneChange` for
    // them, so the log recorded a Stack→Hand move that never happened. Moving
    // emission into `perform_action`'s arm is what finally makes the tag true.
    let mut game = setup_two_player_game();
    let bolt = put_in_hand(&mut game, alpha::lightning_bolt(), 0);
    // No mana, so CR 601.2h cannot be completed and the whole proposal rewinds.

    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
            spell_id: bolt,
        },
        vec![1],
    );
    assert!(game.cast_spell(0, bolt, &decisions).is_err());

    assert_eq!(game.get_object(bolt).unwrap().zone, Zone::Hand, "rewound");
    assert!(
        !zone_changes(&game)
            .iter()
            .any(|(_, from, to, _)| *from == Zone::Stack && *to == Zone::Hand),
        "the rewind itself is not a zone change and may not be announced",
    );

    // **The forward half is still announced, and that is a known gap.** CR
    // 601.2a really does put the card on the stack before costs are paid, so
    // the state change is right; what is wrong is that its `ZoneChange` is
    // emitted at 601.2a, when whether the cast rewinds is not yet knowable. A
    // replay of this log therefore sees a Hand -> Stack move that CR 601.2 says
    // never happened. Fixing it means deferring the announcement to 601.2i
    // without deferring the move, which is its own piece of work -- recorded in
    // codebase-state.md's Deferred Migrations.
    assert_eq!(
        zone_changes(&game)
            .iter()
            .filter(|(_, from, to, _)| *from == Zone::Hand && *to == Zone::Stack)
            .count(),
        1,
        "documenting the gap, not endorsing it",
    );
}

// ---------------------------------------------------------------------------
// Which resolution emitted it
// ---------------------------------------------------------------------------

#[test]
fn test_an_event_names_the_resolution_that_proposed_it() {
    let mut game = setup_two_player_game();
    let bolt = put_in_hand(&mut game, alpha::lightning_bolt(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
            spell_id: bolt,
        },
        vec![1],
    );
    game.cast_spell(0, bolt, &decisions).unwrap();
    game.resolve_top_of_stack(&decisions).unwrap();

    let damage = game
        .events
        .records()
        .iter()
        .find(|r| matches!(r.event, GameEvent::DamageDealt { .. }))
        .expect("the bolt dealt damage");
    assert_eq!(
        damage.resolution(),
        Some(ResolutionStamp { source: bolt, controller: 0 }),
        "CR 614.15 self-replacement effects belong to the resolving object, so \
         the record has to say which resolution a mutation came from",
    );
}

#[test]
fn test_a_turn_based_action_belongs_to_no_resolution() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, self_anthem_creature(), 0);
    game.battlefield.get_mut(&id).unwrap().tapped = true;

    let before = game.events.len();
    game.execute_action(GameAction::Untap { object: id }, &test_ctx()).unwrap();

    let untap = &game.events.records_from(before)[0];
    assert!(matches!(untap.event, GameEvent::Untapped { .. }));
    assert_eq!(
        untap.resolution(),
        None,
        "an untap step, a state-based action, cost payment and combat damage \
         belong to no resolution",
    );
    assert!(untap.batch().is_some(), "but every performed action is in a batch");
}

// ---------------------------------------------------------------------------
// The batch, end to end
// ---------------------------------------------------------------------------

#[test]
fn test_a_board_wipe_of_state_based_deaths_is_one_batch() {
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, phase5_pre_cards::isamaru_hound_of_konda(), 0);
    let b = put_on_battlefield(&mut game, self_anthem_creature(), 1);
    game.battlefield.get_mut(&a).unwrap().damage_marked = 9;
    game.battlefield.get_mut(&b).unwrap().damage_marked = 9;

    let decisions = ScriptedDecisionProvider::new();
    assert!(game.check_state_based_actions(&decisions).unwrap());

    let batches: Vec<Option<BatchId>> = game
        .events
        .records()
        .iter()
        .filter(|r| matches!(r.event, GameEvent::ZoneChange { .. }))
        .map(|r| r.batch())
        .collect();
    assert_eq!(batches.len(), 2);
    assert!(batches[0].is_some());
    assert_eq!(
        batches[0], batches[1],
        "CR 704.3 performs them simultaneously as a single event, across players",
    );
}

#[test]
fn test_a_land_play_and_a_later_one_are_separate_batches() {
    let mut game = setup_two_player_game();
    game.phase.phase_type = mtgsim::state::game_state::PhaseType::Precombat;
    let first = put_in_hand(&mut game, basic_lands::forest(), 0);
    let second = put_in_hand(&mut game, basic_lands::forest(), 0);

    let dp = ScriptedDecisionProvider::new();
    let ctx = ActionContext::new(&dp);
    game.play_land(0, first, Zone::Hand, &ctx).unwrap();
    game.players[0].lands_played_this_turn = 0;
    game.play_land(0, second, Zone::Hand, &ctx).unwrap();

    let batches: Vec<Option<BatchId>> = game
        .events
        .records()
        .iter()
        .filter(|r| matches!(r.event, GameEvent::ZoneChange { .. }))
        .map(|r| r.batch())
        .collect();
    assert_eq!(batches.len(), 2);
    assert_ne!(batches[0], batches[1], "two events, not one");
}

// ---------------------------------------------------------------------------
// CR 608.2m — leaving the stack is a zone change like any other
// ---------------------------------------------------------------------------

#[test]
fn test_a_resolving_permanent_spell_moves_through_the_chokepoint() {
    // One of the three `// REPLACEMENT-BYPASS:` sites. CR 903.9b has to be able
    // to offer the command zone instead of the battlefield here, and could not
    // while the site wrote `obj.zone` by hand.
    let mut game = setup_two_player_game();
    let id = put_in_hand(&mut game, self_anthem_creature(), 0);
    game.players[0].mana_pool.add(ManaType::White, 1);

    let decisions = ScriptedDecisionProvider::new();
    game.cast_spell(0, id, &decisions).unwrap();
    game.resolve_top_of_stack(&decisions).unwrap();

    assert!(game.battlefield.contains_key(&id));
    assert_eq!(
        zone_changes(&game),
        vec![
            (id, Zone::Hand, Zone::Stack, ZoneChangeCause::Cast),
            (id, Zone::Stack, Zone::Battlefield, ZoneChangeCause::Resolved),
        ],
    );
}

#[test]
fn test_a_resolving_instant_moves_through_the_chokepoint() {
    let mut game = setup_two_player_game();
    let bolt = put_in_hand(&mut game, alpha::lightning_bolt(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
            spell_id: bolt,
        },
        vec![1],
    );
    game.cast_spell(0, bolt, &decisions).unwrap();
    game.resolve_top_of_stack(&decisions).unwrap();

    assert_eq!(
        zone_changes(&game).last().copied(),
        Some((bolt, Zone::Stack, Zone::Graveyard, ZoneChangeCause::Resolved)),
    );
}

#[test]
fn test_a_fizzling_spell_moves_through_the_chokepoint() {
    // The third bypass, and the one v1 Commander needs most: a commander
    // creature spell whose target vanished goes stack -> graveyard here, and
    // CR 903.9b must get a say.
    let mut game = setup_two_player_game();
    let victim = put_on_battlefield(&mut game, self_anthem_creature(), 1);
    let bolt = put_in_hand(&mut game, alpha::lightning_bolt(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);

    let decisions = ScriptedDecisionProvider::new();
    // Candidates for `Any` are [Player(0), Player(1), <the one creature>].
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
            spell_id: bolt,
        },
        vec![2],
    );
    game.cast_spell(0, bolt, &decisions).unwrap();

    // The target leaves before the spell resolves, so all its targets are
    // illegal and CR 608.2b counters it by game rules.
    game.change_zone(victim, Zone::Exile, ZoneChangeCause::Exiled, &test_ctx()).unwrap();
    game.resolve_top_of_stack(&decisions).unwrap();

    assert!(game.events.events().any(|e| matches!(e, GameEvent::SpellFizzled { .. })));
    assert_eq!(
        zone_changes(&game).last().copied(),
        Some((bolt, Zone::Stack, Zone::Graveyard, ZoneChangeCause::Fizzled)),
        "countered by game rules is not the same fact as resolved",
    );
}

#[test]
fn test_a_resolving_ability_leaves_no_zone_change() {
    // An activated ability is not a card and has no destination zone: CR 608.2m
    // says it simply ceases to exist. Routing the *spell* paths through the
    // chokepoint must not sweep this one along with them.
    let mut game = setup_two_player_game();
    let (land, ability) = mtgsim::test_support::place_forest(&mut game, 0);

    let before = game.events.len();
    game.activate_mana_ability(0, land, ability, &test_ctx()).unwrap();

    assert!(
        !game.events.records_from(before).iter().any(|r| matches!(
            r.event,
            GameEvent::ZoneChange { to: Zone::Graveyard, .. }
        )),
        "an ability ceases to exist; it does not go anywhere",
    );
}

// ---------------------------------------------------------------------------
// The death events are display sugar
// ---------------------------------------------------------------------------

/// A 2/2 that is a Creature *and* a Planeswalker — the Gideon shape.
fn creature_planeswalker() -> Arc<CardData> {
    CardDataBuilder::new("Gideon, Test Subject")
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Creature)
        .card_type(CardType::Planeswalker)
        .power_toughness(2, 2)
        .loyalty(4)
        .build()
}

#[test]
fn test_one_death_carries_every_type_that_died() {
    // The reason the type-specific death events cannot be matched on. A Gideon
    // dying to lethal damage emits `CreatureDied` and not `PlaneswalkerDied`,
    // so a matcher reading those events would miss every "whenever a
    // planeswalker dies" trigger on the board. The `ZoneChange` plus its LKI
    // frame answers both from one event, which is also what CR 704.3 says
    // happened: one event, not one per type.
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, creature_planeswalker(), 0);
    game.battlefield.get_mut(&id).unwrap().damage_marked = 2;

    let decisions = ScriptedDecisionProvider::new();
    assert!(game.check_state_based_actions(&decisions).unwrap());

    assert_eq!(
        zone_changes(&game),
        vec![(id, Zone::Battlefield, Zone::Graveyard, ZoneChangeCause::DestroyedBySba)],
        "one permanent left the battlefield once",
    );
    let lki = lki_for(&game, id).expect("it was a permanent");
    assert!(lki.types.contains(&CardType::Creature));
    assert!(
        lki.types.contains(&CardType::Planeswalker),
        "the frame carries the whole type set; the death events carry one each",
    );

    // And what the sugar actually said: one of the two applicable types.
    let announced_pw = game.events.events().any(|e| matches!(e, GameEvent::PlaneswalkerDied { .. }));
    assert!(
        !announced_pw,
        "704.5g claimed it, so `PlaneswalkerDied` never fired — which is exactly \
         why nothing may match on these",
    );
}

#[test]
fn test_the_zone_change_carries_everything_the_death_events_did() {
    // Field-for-field: `CreatureDied { creature_id, owner }`,
    // `PlaneswalkerDied { object_id, owner }` and
    // `LegendRuleSacrificed { object_id, owner }` are all
    // `(id, owner)` plus a type and a rule number, and every one of those five
    // facts is on the zone change or its frame.
    let mut game = setup_two_player_game();
    let doomed = put_on_battlefield(&mut game, phase5_pre_cards::isamaru_hound_of_konda(), 1);
    let other = put_on_battlefield(&mut game, phase5_pre_cards::isamaru_hound_of_konda(), 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_pick_n(
        ChoiceKind::LegendRule { legend_name: "Isamaru, Hound of Konda".to_string() },
        vec![0],
    );
    assert!(game.check_state_based_actions(&decisions).unwrap());

    let kept = game.battlefield_ids_ordered()[0];
    let died = if kept == doomed { other } else { doomed };

    let record = game
        .events
        .records()
        .iter()
        .find_map(|r| match &r.event {
            GameEvent::ZoneChange { object_id, owner, from, to, cause, lki }
                if *object_id == died =>
            {
                Some((*owner, *from, *to, *cause, lki.as_deref()))
            }
            _ => None,
        })
        .expect("the legend rule moved it");

    let (owner, from, to, cause, lki) = record;
    assert_eq!(owner, 1, "the `owner` the death events carried");
    assert_eq!((from, to), (Zone::Battlefield, Zone::Graveyard), "\"dies\"");
    assert_eq!(cause, ZoneChangeCause::LegendRule, "the rule number, as a cause");
    let lki = lki.expect("a permanent leaving the battlefield has a frame");
    assert!(lki.types.contains(&CardType::Creature), "the type the event named");
    assert!(
        lki.supertypes.contains(&mtgsim::types::card_types::Supertype::Legendary),
        "and more besides — the frame is the whole object, not one field of it",
    );
}

#[test]
fn test_an_ability_resolving_is_not_a_spell_resolving() {
    // The old name was `SpellResolved`, and `resolve_top_of_stack` emits it
    // unconditionally — so it fired for activated abilities too, and always
    // had. A matcher keying "whenever a spell resolves" on it would have been
    // wrong about every ability in the game. Renamed, and the durable fact an
    // ability actually offers is `AbilityResolved` (CR 603.7h).
    let mut game = setup_two_player_game();
    let thaum = put_on_battlefield(
        &mut game,
        mtgsim::cards::utility_creatures::merfolk_thaumaturgist(),
        0,
    );
    game.battlefield.get_mut(&thaum).unwrap().controller_since_turn = 0;

    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id: thaum,
        },
        vec![0],
    );
    let abilities = mtgsim::oracle::characteristics::get_effective_abilities(&game, thaum);
    let idx = abilities
        .iter()
        .position(|a| a.ability_type == mtgsim::objects::card_data::AbilityType::Activated)
        .unwrap();
    game.activate_ability(0, thaum, idx, &decisions).unwrap();
    game.resolve_top_of_stack(&decisions).unwrap();

    assert!(
        game.events.events().any(|e| matches!(e, GameEvent::StackObjectResolved { .. })),
        "it fires for abilities, which is what made the old name a lie",
    );
    assert!(
        game.events.events().any(|e| matches!(e, GameEvent::AbilityResolved { .. })),
        "and the durable identity is what a CR 603.7h counter reads",
    );
    assert!(
        zone_changes(&game).is_empty(),
        "an ability is not a card: it ceases to exist rather than going anywhere",
    );
}

// ---------------------------------------------------------------------------
// The chokepoint holds
// ---------------------------------------------------------------------------

#[test]
fn test_tapping_for_mana_is_recorded() {
    // A regression guard for RA-2's exit criterion, kept here because RA-3
    // rewired the emission path underneath it: `perform_action`'s arms are the
    // only production writers of `entry.tapped`.
    let mut game = setup_two_player_game();
    let (land, ability) = mtgsim::test_support::place_forest(&mut game, 0);

    game.activate_mana_ability(0, land, ability, &test_ctx()).unwrap();

    assert!(game.battlefield.get(&land).unwrap().tapped);
    assert!(
        game.events.events().any(|e| matches!(e, GameEvent::Tapped { object_id } if *object_id == land)),
        "CR 603.2e: becoming tapped is a transition, and it is announced",
    );
}
