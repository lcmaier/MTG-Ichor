//! Phase RC-4b integration tests: entering is one event, and casting is none
//! until CR 601.2i.
//!
//! RC-4's review found two places where the log recorded the first half of an
//! event the CR says never happened — the entry hop, and the cast rewind's
//! phantom — and `replacement-architecture.md` §11 item 20 is the rule both
//! broke: the replaceable event must be the outermost proposal. These are
//! log-shape tests. Each asserts what the event log says about an entry that
//! was substituted, refused or dropped, or about a cast that rewound, and each
//! that asserts a fix was run against the pre-fix tree first and failed there.
//! The two that pass on both trees are the regression guards the phase brief
//! named, and say so.

use std::sync::Arc;

use mtgsim::cards::alpha;
use mtgsim::cards::basic_lands::forest;
use mtgsim::cards::phase_rc_cards::containment_priest;
use mtgsim::engine::actions::{GameAction, ZoneChangeCause};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_graveyard, put_in_hand, put_on_battlefield, setup_two_player_game, static_ability,
    test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AffectedSet, AmountExpr, Duration, Effect, EffectRecipient, PermanentFilter, Primitive,
    SelectionFilter, TargetCount, TokenDef, TypeChange,
};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use mtgsim::types::restriction::{Restriction, RestrictionDef};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// Log readers
// ---------------------------------------------------------------------------

/// The events recorded from `start` on. Fixtures placed with
/// `put_on_battlefield` announce their own entry, so every reader here starts
/// after a test's setup and reads only what the action under test emitted.
fn log_after(game: &GameState, start: usize) -> impl Iterator<Item = &GameEvent> {
    game.events.records_from(start).iter().map(|r| &r.event)
}

/// Every `ZoneChange` since `start`, as `(object, from, to, cause)`.
fn zone_changes(game: &GameState, start: usize) -> Vec<(ObjectId, Zone, Zone, ZoneChangeCause)> {
    log_after(game, start)
        .filter_map(|e| match e {
            GameEvent::ZoneChange { object_id, from, to, cause, .. } => {
                Some((*object_id, *from, *to, *cause))
            }
            _ => None,
        })
        .collect()
}

/// The zone changes of one object, as `(from, to, cause, carries an LKI frame)`.
fn moves_of(game: &GameState, start: usize, id: ObjectId) -> Vec<(Zone, Zone, ZoneChangeCause, bool)> {
    log_after(game, start)
        .filter_map(|e| match e {
            GameEvent::ZoneChange { object_id, from, to, cause, lki, .. } if *object_id == id => {
                Some((*from, *to, *cause, lki.is_some()))
            }
            _ => None,
        })
        .collect()
}

/// Every `PermanentEnteredBattlefield` since `start`, by object.
fn entries(game: &GameState, start: usize) -> Vec<ObjectId> {
    log_after(game, start)
        .filter_map(|e| match e {
            GameEvent::PermanentEnteredBattlefield { object_id, .. } => Some(*object_id),
            _ => None,
        })
        .collect()
}

/// A one-word kind for each record since `start`, in order, with everything a
/// test here does not read left out.
fn kinds(game: &GameState, start: usize) -> Vec<&'static str> {
    log_after(game, start)
        .filter_map(|e| match e {
            GameEvent::ZoneChange { .. } => Some("zone-change"),
            GameEvent::PermanentEnteredBattlefield { .. } => Some("entered"),
            GameEvent::SpellCast { .. } => Some("spell-cast"),
            GameEvent::Tapped { .. } => Some("tapped"),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// A white enchantment whose only text is one static restriction.
fn static_restriction(name: &str, what: Restriction) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Restriction(Box::new(RestrictionDef::new(what)))))
        .build()
}

/// "Lands can't enter the battlefield." — Worms of the Earth's second sentence,
/// in the zone-change shape RC-4 registered it in.
fn lands_cant_enter() -> Restriction {
    Restriction::Event {
        pattern: EventPattern::ZoneChange {
            from: None,
            to: Some(Zone::Battlefield),
            cause: None,
            object: None,
        },
        affected: AffectedSet::Filter { filter: PermanentFilter::ByType(CardType::Land) },
        by: None,
    }
}

/// "Creatures can't enter the battlefield." No printed card says this; it is
/// Worms of the Earth's shape written against the entry itself — the door
/// CR 614.17d could not use while a refused entry stranded the card.
fn creatures_cant_enter() -> Restriction {
    Restriction::Event {
        pattern: EventPattern::EnterBattlefield { cast: None },
        affected: AffectedSet::Filter { filter: PermanentFilter::ByType(CardType::Creature) },
        by: None,
    }
}

/// "Each permanent is a land in addition to its other types." Mycosynth
/// Lattice's shape with the type changed, so a creature card is a land *as it
/// would exist on the battlefield* and nowhere else.
fn everything_is_a_land() -> Arc<CardData> {
    CardDataBuilder::new("Everything is a land")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeType(
                TypeChange {
                    add_types: vec![CardType::Land],
                    remove_types: Vec::new(),
                    set_types: None,
                    add_subtypes: Vec::new(),
                    remove_subtypes: Vec::new(),
                    set_subtypes: None,
                    add_supertypes: Vec::new(),
                    remove_supertypes: Vec::new(),
                    set_supertypes: None,
                },
                Duration::WhileSourceOnBattlefield,
            ),
            EffectRecipient::FilteredPermanents(PermanentFilter::All),
        )))
        .build()
}

/// "If a creature would enter and it wasn't cast, exile it instead." —
/// Hallowed Moonlight's text as a permanent. Unlike Containment Priest it does
/// not say "nontoken", so a token entering under it is exiled instead.
fn moonlight_shaped() -> Arc<CardData> {
    CardDataBuilder::new("Moonlight-shaped")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: Some(false) },
            AffectedSet::Filter { filter: PermanentFilter::ByType(CardType::Creature) },
            Rewrite::Instead(GameActionTemplate::ZoneChangeTo {
                to: Zone::Exile,
                cause: ZoneChangeCause::Exiled,
            }),
        )))))
        .build()
}

/// A green 2/2 costing `{G}` plus `generic`.
fn bear_costing(generic: u8) -> Arc<CardData> {
    CardDataBuilder::new("Bear")
        .mana_cost(ManaCost::build(&[ManaType::Green], generic))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Bear))
        .power_toughness(2, 2)
        .build()
}

/// Resolve "create a 2/2 black Zombie creature token" for `controller`, the
/// way Kalitas's rider does.
fn create_zombie(game: &mut GameState, controller: usize, source: ObjectId) -> Result<(), String> {
    let def = TokenDef {
        name: "Zombie".to_string(),
        colors: vec![Color::Black],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Zombie)],
        power: 2,
        toughness: 2,
        keyword_flags: Vec::new(),
    };
    let effect = Effect::Atom(
        Primitive::CreateToken(def, AmountExpr::Fixed(1)),
        EffectRecipient::Controller,
    );
    let ctx = ResolutionContext { source, controller, targets: Vec::new() };
    game.resolve_effect(&effect, &ctx, &test_dp())
}

// ---------------------------------------------------------------------------
// The entry hop — Containment Priest's substitute is one move
// ---------------------------------------------------------------------------

/// The trace RC-4's review pinned: a creature card returned from the graveyard
/// under Containment Priest. The CR says it was exiled from the graveyard and
/// never entered, so the log holds one zone change, `Graveyard → Exile`, with
/// no CR 603.10a frame (nothing left the battlefield) — and the object moved
/// once, so its CR 400.7 epoch advanced once.
#[test]
fn test_containment_priest_exiles_from_the_graveyard_in_one_move() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, containment_priest(), 1);

    // A probe that moves just before, so the epoch after is measurable.
    let probe = put_in_graveyard(&mut game, vanilla_creature(1, 1, &[]), 0);
    game.change_zone(probe, Zone::Exile, ZoneChangeCause::Exiled, &test_ctx()).unwrap();
    let epoch_before = game.get_object(probe).unwrap().zone_change_epoch;

    let start = game.events.records().len();
    let bear = put_in_graveyard(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.change_zone(bear, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx())
        .expect("the entry is replaced, and that is not an error");

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Exile);
    assert!(game.exile.contains(&bear));
    assert!(!game.battlefield.contains_key(&bear));
    assert_eq!(
        moves_of(&game, start, bear),
        vec![(Zone::Graveyard, Zone::Exile, ZoneChangeCause::Exiled, false)],
        "one move from where it was, and no look-back frame for a permanent that never existed"
    );
    assert!(!entries(&game, start).contains(&bear), "it never entered");
    assert_eq!(
        game.get_object(bear).unwrap().zone_change_epoch,
        epoch_before + 1,
        "one move is one epoch (CR 400.7)"
    );
}

/// The unreplaced case, for contrast: a land drop is one zone change and one
/// entry in one batch, the move announced first — the after-picture of the
/// review's Trace A.
#[test]
fn test_an_entry_from_hand_is_one_move_and_one_entry_in_one_batch() {
    let mut game = setup_two_player_game();
    let land = put_in_hand(&mut game, forest(), 0);

    let start = game.events.records().len();
    game.play_land(0, land, Zone::Hand, &test_ctx()).unwrap();

    assert_eq!(
        zone_changes(&game, start),
        vec![(land, Zone::Hand, Zone::Battlefield, ZoneChangeCause::PlayedAsLand)],
    );
    assert_eq!(entries(&game, start), vec![land]);
    assert_eq!(kinds(&game, start), vec!["zone-change", "entered"]);
    let batches: Vec<_> = game.events.records_from(start).iter().map(|r| r.stamp.batch).collect();
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], batches[1], "one event, one batch");
}

// ---------------------------------------------------------------------------
// CR 614.6 — a dropped entry leaves the card where it was
// ---------------------------------------------------------------------------

/// A "can't enter" watching the entry itself. Refusing the event leaves the
/// card in the graveyard with nothing announced, and the caller is told
/// nothing went wrong, because nothing did.
// COVERS-PARTIAL: ATOM-614.17d-001
#[test]
fn test_a_dropped_entry_leaves_the_card_where_it_was() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, static_restriction("Creatures can't enter", creatures_cant_enter()), 1);
    let bear = put_in_graveyard(&mut game, vanilla_creature(2, 2, &[]), 0);

    let start = game.events.records().len();
    game.change_zone(bear, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx())
        .expect("a refused entry is not an error");

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Graveyard);
    assert!(game.players[0].graveyard.contains(&bear));
    assert!(!game.battlefield.contains_key(&bear));
    assert!(zone_changes(&game, start).is_empty(), "nothing moved, so nothing was announced");
    assert!(entries(&game, start).is_empty());
}

/// Regression guard, and it passes on both trees: Worms of the Earth's
/// zone-change-shaped restriction still refuses a returned Forest, now at the
/// entry proposal, which is the only proposal an entry has.
#[test]
fn test_worms_of_the_earth_still_refuses_a_returned_forest() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, static_restriction("Lands can't enter", lands_cant_enter()), 1);
    let land = put_in_graveyard(&mut game, forest(), 0);

    let start = game.events.records().len();
    game.change_zone(land, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx()).unwrap();

    assert_eq!(game.get_object(land).unwrap().zone, Zone::Graveyard);
    assert!(!game.battlefield.contains_key(&land));
    assert!(zone_changes(&game, start).is_empty());
}

// ---------------------------------------------------------------------------
// CR 608.3e — a resolved permanent spell whose entry does not happen
// ---------------------------------------------------------------------------

/// "If a permanent spell resolves but its controller can't put it onto the
/// battlefield, that player puts it into its owner's graveyard." The spell
/// resolved, so the graveyard trip carries `Resolved`.
// COVERS-PARTIAL: ATOM-608.3e-001
#[test]
fn test_a_resolved_spell_whose_entry_is_refused_goes_to_the_graveyard() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, static_restriction("Creatures can't enter", creatures_cant_enter()), 1);
    let bear = put_in_hand(&mut game, bear_costing(0), 0);
    game.players[0].mana_pool.add(ManaType::Green, 1);

    let start = game.events.records().len();
    game.cast_spell(0, bear, &test_dp()).expect("nothing stops the cast (CR 601.3)");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves; it just cannot arrive");

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Graveyard);
    assert!(game.stack.is_empty(), "CR 608.3e takes it off the stack");
    assert!(!game.battlefield.contains_key(&bear));
    assert_eq!(
        zone_changes(&game, start),
        vec![
            (bear, Zone::Hand, Zone::Stack, ZoneChangeCause::Cast),
            (bear, Zone::Stack, Zone::Graveyard, ZoneChangeCause::Resolved),
        ],
    );
    assert!(entries(&game, start).is_empty());
}

/// The same rule through the printed shape: Worms of the Earth beside "each
/// permanent is a land", so a creature spell is a land as it would exist on
/// the battlefield (CR 614.17d) and can't enter.
// COVERS-PARTIAL: ATOM-608.3e-001
#[test]
fn test_cr_608_3e_through_worms_of_the_earth_and_the_frame() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, static_restriction("Lands can't enter", lands_cant_enter()), 1);
    put_on_battlefield(&mut game, everything_is_a_land(), 1);
    let bear = put_in_hand(&mut game, bear_costing(0), 0);
    game.players[0].mana_pool.add(ManaType::Green, 1);

    game.cast_spell(0, bear, &test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Graveyard, "a land can't enter, so CR 608.3e");
    assert!(game.stack.is_empty());
}

// ---------------------------------------------------------------------------
// Tokens — created in exile instead, and not created at all
// ---------------------------------------------------------------------------

/// Hallowed Moonlight's ruling: a creature token that would enter is created
/// in exile instead and ceases to exist as a state-based action (CR 704.5d).
/// No entry is announced, and the one zone change carries no look-back frame,
/// because there was never a permanent to look back at.
#[test]
fn test_an_exiled_instead_token_never_had_a_battlefield_to_leave() {
    let mut game = setup_two_player_game();
    let moonlight = put_on_battlefield(&mut game, moonlight_shaped(), 0);

    let start = game.events.records().len();
    create_zombie(&mut game, 0, moonlight).expect("the token is created — in exile");

    let token = *game.exile.last().expect("it is in exile");
    assert!(game.get_object(token).unwrap().is_token);
    assert!(!game.battlefield.contains_key(&token));
    assert!(entries(&game, start).is_empty(), "it never entered");
    let moves = moves_of(&game, start, token);
    assert_eq!(moves.len(), 1, "one zone change, into exile");
    assert_eq!(moves[0].1, Zone::Exile);
    assert!(!moves[0].3, "no CR 603.10a frame: it was never a permanent");

    game.check_state_based_actions(&test_dp()).unwrap();
    assert!(!game.objects.contains_key(&token), "CR 704.5d");
}

/// CR 111.5 — "if a spell or ability would create a token, but a rule or
/// effect states that a permanent with one or more of that token's
/// characteristics can't enter the battlefield, the token is not created."
#[test]
fn test_a_dropped_token_entry_creates_nothing() {
    let mut game = setup_two_player_game();
    let wall = put_on_battlefield(&mut game, static_restriction("Creatures can't enter", creatures_cant_enter()), 0);
    let objects_before = game.objects.len();

    let start = game.events.records().len();
    create_zombie(&mut game, 0, wall).expect("CR 111.5 is not an error");

    assert_eq!(game.objects.len(), objects_before, "not created");
    assert!(game.exile.is_empty());
    assert!(log_after(&game, start).next().is_none(), "and nothing was announced");
}

// ---------------------------------------------------------------------------
// The proposal vocabulary — entering is not a zone change proposal
// ---------------------------------------------------------------------------

/// A `ZoneChange` onto the battlefield is no longer a proposal the engine
/// makes; `change_zone` routes it to the entry, and a caller that builds one
/// by hand has bypassed that routing. Debug builds say so.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "is not a proposal")]
fn test_a_zone_change_onto_the_battlefield_is_not_a_proposal() {
    let mut game = setup_two_player_game();
    let bear = put_in_graveyard(&mut game, vanilla_creature(2, 2, &[]), 0);
    let _ = game.execute_action(
        GameAction::ZoneChange {
            object: bear,
            from: Zone::Graveyard,
            to: Zone::Battlefield,
            cause: ZoneChangeCause::Returned,
        },
        &test_ctx(),
    );
}

// ---------------------------------------------------------------------------
// CR 601.2 — the cast's move is announced at 601.2i, and a rewind announces
// nothing
// ---------------------------------------------------------------------------

/// A cast that pays with a mana ability. Two orders, and they differ on
/// purpose. The *state* order is the CR's: the card is on the stack from
/// 601.2a, the land is tapped at 601.2g against a spell already there, and the
/// spell becomes cast at 601.2i. The *log* order is tap, then the move, then
/// `SpellCast`, because the move is not an event until 601.2i — a cast that
/// rewinds must leave no trace of it (CR 732.1) — while the tap is an event
/// when it happens. No trigger cares which came first: CR 603.3 places both
/// after the cast, in their controller's order.
#[test]
fn test_a_cast_is_announced_at_601_2i_after_its_mana_abilities() {
    let mut game = setup_two_player_game();
    let land = put_on_battlefield(&mut game, forest(), 0);
    let bear = put_in_hand(&mut game, bear_costing(0), 0);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::ManaAbilityWindow {
            spell_or_ability_id: bear,
            remaining_cost: ManaCost::build(&[ManaType::Green], 0),
        },
        vec![0],
    );
    let start = game.events.records().len();
    game.cast_spell(0, bear, &dp).expect("the Forest pays for it");
    assert!(dp.is_empty());

    assert!(game.battlefield.get(&land).unwrap().tapped);
    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Stack);
    assert_eq!(kinds(&game, start), vec!["tapped", "zone-change", "spell-cast"]);
    assert_eq!(
        zone_changes(&game, start),
        vec![(bear, Zone::Hand, Zone::Stack, ZoneChangeCause::Cast)],
    );
}

/// A cast that taps a land at 601.2g and then cannot pay at 601.2h. CR 732.1:
/// the action is reversed and the spell returns to the zone it came from, and
/// the player *may* also reverse the mana abilities activated along the way.
/// The engine takes the "may not" branch unasked — the tap stays performed,
/// stays in the log, and the mana stays in the pool — and the other branch is
/// a `DecisionProvider` question it does not ask yet (`backlog.md` §2.18). The
/// move that never legally happened is not in the log at all.
// COVERS-PARTIAL: ATOM-601.2h-002
#[test]
fn test_a_rewound_cast_keeps_its_mana_abilities_and_leaves_no_zone_change() {
    let mut game = setup_two_player_game();
    let land = put_on_battlefield(&mut game, forest(), 0);
    let bear = put_in_hand(&mut game, bear_costing(1), 0);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::ManaAbilityWindow {
            spell_or_ability_id: bear,
            remaining_cost: ManaCost::build(&[ManaType::Green], 1),
        },
        vec![0],
    );
    let start = game.events.records().len();
    assert!(game.cast_spell(0, bear, &dp).is_err(), "{{1}}{{G}} against one Forest");
    assert!(dp.is_empty());

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Hand, "rewound");
    assert!(game.players[0].hand.contains(&bear));
    assert!(game.stack.is_empty());
    assert!(game.battlefield.get(&land).unwrap().tapped, "the tap was a legal action and stays");
    assert!(
        game.players[0].mana_pool.available().iter().any(|(mt, n)| *mt == ManaType::Green && *n == 1),
        "and its mana is still in the pool (CR 732.1)"
    );
    assert_eq!(kinds(&game, start), vec!["tapped"], "the tap is in the log; the move that rewound is not");
    assert!(zone_changes(&game, start).is_empty());
}

/// The rewind site before any mana is involved: a Lightning Bolt with no mana
/// and a target. It rewinds at 601.2h and leaves nothing in the log.
#[test]
fn test_a_rewound_cast_leaves_no_zone_change() {
    let mut game = setup_two_player_game();
    let bolt = put_in_hand(&mut game, alpha::lightning_bolt(), 0);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
            spell_id: bolt,
        },
        vec![1],
    );
    let start = game.events.records().len();
    assert!(game.cast_spell(0, bolt, &dp).is_err());

    assert_eq!(game.get_object(bolt).unwrap().zone, Zone::Hand);
    assert!(log_after(&game, start).next().is_none(), "a rewound cast is not an event, in either direction");
}
