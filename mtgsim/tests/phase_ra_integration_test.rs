//! Phase RA integration tests: the event spine.
//!
//! RA carries no replacement behavior. What it owes is that every mutation a
//! CR 614 replacement or a CR 603 trigger could care about is proposed through
//! one chokepoint and recorded once, with enough on the record to tell the
//! cases apart — drawn from tutored, destroyed from sacrificed, countered from
//! resolved. These tests pin the *record*, which is the part that has no other
//! consumer until Phase RB and would otherwise rot unwatched.

use std::sync::Arc;

use mtgsim::cards::{alpha, basic_lands, creatures, phase5_pre_cards};
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
    // The proposal describes where the object *is*, and performing it against a
    // different board is a caller bug: RB matches replacement effects on
    // `from`, so a stale value is a wrong match rather than a cosmetic
    // mismatch.
    //
    // **This checks the zone and nothing else.** It is not a general staleness
    // guard — a proposal built when a permanent was untapped and performed
    // after it was tapped sails through, and should, because no field of
    // `ZoneChange` claims anything about tapped. Each arm asserts its own
    // preconditions: `Tap`/`Untap` error for an object that is not on the
    // battlefield, `DealDamage` for a target that is not.
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

// COVERS-PARTIAL: ATOM-601.2h-002
//
// Partial: the atom leaves a *partial* pool ({2}{R} against a {3}{R} cost) and
// asserts the rewind spends none of it. This player has no mana at all, so the
// "payment fails, the cast rewinds" half is proven and the "and the mana you
// did have is still there" half is not.
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

    // And neither is the forward half (RC-4b). CR 601.2a really does put the
    // card on the stack before costs are paid, but the spell is not cast until
    // 601.2i, so the move is announced there — and a cast that rewinds never
    // gets there. A replay of this log sees no move at all, which is what
    // CR 601.2 says happened.
    assert!(
        zone_changes(&game).is_empty(),
        "the 601.2a move is announced at 601.2i, which a rewound cast never reaches",
    );
}

/// Grizzly Bears {1}{G} against a pool of {R}{G} is payable exactly one way,
/// and the prompt now offers only that way — this board is
/// `codebase-state.md` 16c's reproducer, and it used to rewind the cast.
///
/// The prompt lists the pool's types in `ManaType` order, Red then Green, and
/// each bucket's maximum is what its own pips leave over: Red 1, Green 0. So
/// the generic {1} goes on the Red, the {G} pip keeps its Green, and the cast
/// completes. See its sibling below for the answer that is no longer offered.
///
/// The rewind is still there and still right — `phase_rc4b`'s two rewind tests
/// pin it, and `fuzz_games` fails any run in which a spell resolves without a
/// `SpellCast` — it is just no longer reachable this way.
#[test]
fn test_the_generic_split_is_offered_clamped_and_the_only_answer_pays() {
    let mut game = setup_two_player_game();
    let bear = put_in_hand(&mut game, creatures::grizzly_bears(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);
    game.players[0].mana_pool.add(ManaType::Green, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: ManaCost::zero() },
        vec![1, 0], // [Red, Green]: the generic {1} on the Red
    );
    game.cast_spell(0, bear, &decisions).expect("the only legal split pays");
    assert!(decisions.is_empty(), "the split prompt was asked");

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Stack);
    assert!(game.stack.contains(&bear));
    assert_eq!(game.players[0].mana_pool.total(), 0, "both mana paid the cost");
    assert!(
        game.events.events().any(|e| matches!(
            e,
            GameEvent::SpellCast { spell_id, .. } if *spell_id == bear
        )),
        "announced as cast at 601.2i",
    );
}

/// The same board, and the split that used to cost the whole cast: the generic
/// {1} on the Green the {G} pip still needs. Before the clamp it passed the
/// prompt's own validation, `ManaPool::pay` refused it, and CR 601.2 rewound
/// everything — for a choice the engine should never have offered. The Green
/// bucket's maximum is 0 now, and the panic names it.
#[test]
#[should_panic(expected = "DP allocated 1 to bucket 1 but maximum is 0")]
fn test_the_generic_split_cannot_be_put_on_a_color_its_pip_needs() {
    let mut game = setup_two_player_game();
    let bear = put_in_hand(&mut game, creatures::grizzly_bears(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);
    game.players[0].mana_pool.add(ManaType::Green, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: ManaCost::zero() },
        vec![0, 1], // [Red, Green]: bucket 1 is the Green, and the {G} pip has it
    );
    let _ = game.cast_spell(0, bear, &decisions);
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
        "the stamp is provenance: which resolution proposed this mutation. \
         CR 614.15 is what makes the *proposal* side of it load-bearing — a \
         self-replacement effect belongs to the resolving spell rather than to \
         any registry, so `apply_replacements` finds it through \
         `ActionContext::resolution`. Carrying it onto the record as well is a \
         separate, weaker call: nothing matches on it yet",
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
// CR 608.2n / 608.3 — leaving the stack is a zone change like any other
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
    // The third bypass. CR 608.2b's counter-by-game-rules is a real zone
    // change, and 39 printed cards replace "a card that would be put into a
    // graveyard from anywhere" (Leyline of the Void, Rest in Peace) — none of
    // which could see this while the site wrote `obj.zone` by hand.
    //
    // **An earlier version of this comment said a commander creature spell
    // fizzles here. It cannot, twice over.** A creature spell has no targets, so
    // CR 608.3a resolves it unconditionally; only an Aura or a mutating creature
    // spell can fail CR 608.3b, and that is a different code path from this one.
    // And a fizzled spell goes to the *graveyard*, so the Commander rule with an
    // interest is CR 903.9a — a state-based action (CR 704.6d) that is not
    // blocked on this phase at all — not CR 903.9b, which is hand or library.
    //
    // What is true is narrower and newer: **Ashaya's Enduring Bond** (Legendary
    // Sorcery, "Ashaya's Enduring Bond can be your commander"; Scryfall
    // 2026-08-26) makes a *fizzling commander spell* reachable for the first
    // time. Before it, every commander was a permanent, and permanents that
    // resolve do not come through here.
    //
    // The spell under test is an instant, which is what CR 608.2b is written
    // for.
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
    // An activated ability is not a card and has no destination zone: CR 608.2n
    // removes it from the stack and it ceases to exist. Routing the *spell*
    // paths through the chokepoint must not sweep this one along with them.
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
    // The reason there is no `CreatureDied` or `PlaneswalkerDied`. A Gideon is
    // both, and any event that named one type would make a reader miss every
    // trigger keyed on the other. One zone change, and the CR 603.10a frame
    // answers for every type it had.
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
        "the frame carries the whole type set, which is what a single \
         type-specific event never could",
    );
}

#[test]
fn test_the_zone_change_carries_everything_the_death_events_did() {
    // Field-for-field against the four events RA-3 deleted. Each was
    // `(id, owner)` plus one type and an implied rule number, and every one of
    // those facts is on the zone change or its frame.
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
    assert_eq!(owner, 1, "the `owner` the deleted events carried");
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
    // There is no `SpellResolved`. `resolve_top_of_stack` emitted it
    // unconditionally, so it fired for activated abilities too and always had —
    // a matcher keying "whenever a spell resolves" on it would have been wrong
    // about every ability in the game. What an ability leaves behind instead is
    // `AbilityResolved` and no zone change at all (CR 608.2n).
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
        game.events.events().any(|e| matches!(e, GameEvent::AbilityResolved { .. })),
        "an ability announces itself by its durable identity, which is what a \
         CR 603.7h counter reads",
    );
    assert!(
        zone_changes(&game).is_empty(),
        "an ability is not a card: it ceases to exist rather than going anywhere",
    );
}

#[test]
fn test_the_log_line_still_says_what_died() {
    // The four deleted events were also four log lines, and deleting them is
    // only safe if the one that replaced them says as much. Object, owner, the
    // rule that moved it, and what it was — the last of which the old lines
    // could give one type at a time.
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, creature_planeswalker(), 0);
    game.battlefield.get_mut(&id).unwrap().damage_marked = 2;

    let decisions = ScriptedDecisionProvider::new();
    assert!(game.check_state_based_actions(&decisions).unwrap());

    let line = mtgsim::ui::display::format_event_log(&game)
        .into_iter()
        .find(|l| l.starts_with("ZoneChange:"))
        .expect("the death is in the log");

    for expected in ["Gideon, Test Subject", "P0", "Battlefield", "Graveyard",
                     "DestroyedBySba", "Creature", "Planeswalker"] {
        assert!(line.contains(expected), "log line {:?} is missing {:?}", line, expected);
    }
}

#[test]
fn test_a_multi_target_destroy_is_one_event() {
    // CR 608.2f: "Some spells and abilities include actions taken on multiple
    // players and/or objects. In most cases, each such action is processed
    // simultaneously." A board wipe is one event, not N.
    //
    // This is what a CR 614 replacement reads. Kalitas, Traitor of Ghet ("If a
    // nontoken creature an opponent controls would die, instead exile that card
    // and create a 2/2 black Zombie") applies once *per death* -- CR 614.5 is
    // per event and batch members are separate events -- but the pipeline only
    // gets to make that judgement if the deaths arrive together. A loop of
    // `execute_action` would hand it N unrelated events instead.
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, self_anthem_creature(), 1);
    let b = put_on_battlefield(&mut game, self_anthem_creature(), 1);
    let c = put_on_battlefield(&mut game, self_anthem_creature(), 1);

    let ctx = mtgsim::engine::resolve::ResolutionContext {
        source: a,
        controller: 0,
        targets: vec![
            mtgsim::engine::resolve::ResolvedTarget::Object(a),
            mtgsim::engine::resolve::ResolvedTarget::Object(b),
            mtgsim::engine::resolve::ResolvedTarget::Object(c),
        ],
    };
    let dp = ScriptedDecisionProvider::new();
    game.resolve_effect(
        &Effect::Atom(Primitive::Destroy, EffectRecipient::Implicit),
        &ctx,
        &dp,
    ).unwrap();

    let batches: Vec<_> = game.events.records().iter()
        .filter(|r| matches!(r.event, GameEvent::ZoneChange { .. }))
        .map(|r| r.batch())
        .collect();
    assert_eq!(batches.len(), 3, "all three were destroyed");
    assert!(batches[0].is_some());
    assert!(
        batches.iter().all(|x| *x == batches[0]),
        "one spell, one event -- a replacement effect gets to apply once per          death within it, which it cannot judge if they arrive separately",
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
