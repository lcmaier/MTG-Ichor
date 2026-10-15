//! TR-4b — attachment's records (`triggers-architecture.md` §12, TR-4b).
//!
//! 1. CR 701.3d's four routes to becoming unattached, each announced as one
//!    `Unattached` record naming its route: an unattach, a move to another
//!    object, the attachment leaving and the host leaving.
//! 2. CR 704.5n, 704.5p and 704.5d as members of CR 704.3's one event,
//!    decided against the board before it (`codebase-state.md` item 6, and
//!    `plans/references/feedback-loops.md` §4).
//! 3. Item 228: an Aura entering attached becomes attached, in the entry's
//!    own event (CR 303.4f).

use std::sync::Arc;

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lh_cards::holy_strength;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::{ChosenTargets, TargetInstance};
use mtgsim::events::cause::GeneratedBy;
use mtgsim::engine::actions::GameAction;
use mtgsim::events::cause::{EventCause, StateBasedAction};
use mtgsim::events::event::{GameEvent, UnattachRoute};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    aura_enchanting_your_creature, card_of_type, effect_ctx, equipment, put_in_graveyard, put_on_battlefield,
    put_spell_on_stack, setup_game, setup_two_player_game, static_ability, test_ctx, test_dp,
};
use mtgsim::types::card_types::{ArtifactType, CardType, EnchantmentType, Subtype};
use mtgsim::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, Primitive, ReturnUnder, SelectionFilter, TargetCount,
    TypeChange,
};
use mtgsim::types::ids::{ObjectId, ObjectRef, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::replacement::EnterMods;
use mtgsim::types::zones::{Zone, ZoneChangeAct};

// ---------------------------------------------------------------------------
// Fixtures and helpers
// ---------------------------------------------------------------------------

fn destroy_all(game: &mut GameState, objects: &[ObjectId]) {
    let by = mtgsim::types::ids::new_object_id();
    let batch = objects.iter().map(|&object| GameAction::Destroy { object }).collect();
    game.execute_actions(batch, &effect_ctx(game, by)).unwrap();
}

fn attach(game: &mut GameState, attachment: ObjectId, host: ObjectId) {
    game.execute_action(GameAction::Attach { attachment, host }, &test_ctx()).unwrap();
}

/// Every attach and unattach the game recorded, in order.
fn attachment_records(game: &GameState) -> Vec<GameEvent> {
    game.recorded_events()
        .events()
        .filter(|e| matches!(e, GameEvent::Attached { .. } | GameEvent::Unattached { .. }))
        .cloned()
        .collect()
}

fn unattached(attachment: ObjectRef, former_host: ObjectRef, route: UnattachRoute) -> GameEvent {
    GameEvent::Unattached { attachment, former_host, route }
}

fn existence(game: &GameState, id: ObjectId) -> ObjectRef {
    game.object_ref(id).expect("in the store")
}

/// A 1/1 creature whose static ability makes each permanent of `subtype` a
/// 2/2 creature, with `keywords` — March of the Machines' shape on a
/// creature that can die in a state-based check.
fn animator(subtype: Subtype, keywords: &[KeywordFlag]) -> Arc<CardData> {
    let animated = EffectRecipient::FilteredPermanents(ObjectFilter::BySubtype(subtype));
    let creature = TypeChange { add_types: vec![CardType::Creature], ..TypeChange::NONE };
    let mut effects = vec![
        Effect::Atom(Primitive::ChangeType(creature, Duration::WhileSourceOnBattlefield), animated.clone()),
        Effect::Atom(
            Primitive::SetPowerToughness(AmountExpr::Fixed(2), AmountExpr::Fixed(2), Duration::WhileSourceOnBattlefield),
            animated.clone(),
        ),
    ];
    effects.extend(
        keywords
            .iter()
            .map(|&k| Effect::Atom(Primitive::GrantKeywordFlag(k, Duration::WhileSourceOnBattlefield), animated.clone())),
    );
    CardDataBuilder::new("Probe Animator")
        .card_type(CardType::Creature)
        .power_toughness(1, 1)
        .ability(static_ability(Effect::Sequence(effects)))
        .build()
}

fn equipment_subtype() -> Subtype {
    Subtype::Artifact(ArtifactType::Equipment)
}

fn mark_damage(game: &mut GameState, id: ObjectId, damage: u32) {
    game.battlefield.get_mut(&id).unwrap().damage_marked = damage;
}

fn host(game: &GameState, id: ObjectId) -> Option<ObjectId> {
    game.battlefield[&id].attached_to
}

/// Grizzly Bears with an Equipment on it, for player 0.
fn equipped_bears(game: &mut GameState) -> (ObjectId, ObjectId) {
    let bears = put_on_battlefield(game, grizzly_bears(), 0);
    let sword = put_on_battlefield(game, equipment("Probe Sword"), 0);
    assert!(game.attach(sword, bears));
    (sword, bears)
}

// ---------------------------------------------------------------------------
// 1. The four routes (CR 701.3d)
// ---------------------------------------------------------------------------

/// Moving an Equipment to another creature unattaches it from the first,
/// announced ahead of the attach, in the one event.
#[test]
fn reattaching_announces_the_unattach_from_the_first_host_then_the_attach() {
    let mut game = setup_two_player_game();
    let (sword, bears) = equipped_bears(&mut game);
    let other = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let (sword_ref, bears_ref) = (existence(&game, sword), existence(&game, bears));
    game.record_events();

    attach(&mut game, sword, other);

    assert_eq!(
        attachment_records(&game),
        vec![
            unattached(sword_ref, bears_ref, UnattachRoute::AttachedElsewhere),
            GameEvent::Attached { attachment: sword, host: other, former_host: Some(bears) },
        ]
    );
    let batches: Vec<_> = game.recorded_events().records().iter().map(|r| r.stamp.batch).collect();
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0], batches[1], "one event (CR 701.3c, 701.3d)");
}

/// CR 701.3b: attaching to the host it is already on does nothing, and
/// nothing becomes unattached either.
#[test]
fn reattaching_to_the_same_host_announces_nothing() {
    let mut game = setup_two_player_game();
    let (sword, bears) = equipped_bears(&mut game);
    game.record_events();

    attach(&mut game, sword, bears);

    assert!(attachment_records(&game).is_empty());
}

/// An Equipment leaving the battlefield becomes unattached as it leaves,
/// named as the existence that left (CR 400.7), after its own move.
#[test]
fn an_equipment_leaving_becomes_unattached_after_its_move() {
    let mut game = setup_two_player_game();
    let (sword, bears) = equipped_bears(&mut game);
    let (sword_ref, bears_ref) = (existence(&game, sword), existence(&game, bears));
    game.record_events();

    destroy_all(&mut game, &[sword]);

    assert_eq!(attachment_records(&game), vec![unattached(sword_ref, bears_ref, UnattachRoute::AttachmentLeft)]);
    let kinds: Vec<&str> = game
        .recorded_events()
        .events()
        .filter_map(|e| match e {
            GameEvent::ZoneChange { object_id, .. } if *object_id == sword => Some("move"),
            GameEvent::Unattached { .. } => Some("unattached"),
            _ => None,
        })
        .collect();
    assert_eq!(kinds, vec!["move", "unattached"]);
    assert_ne!(existence(&game, sword), sword_ref, "the record names the permanent, not the graveyard card");
}

/// A host leaving unattaches everything on it, in the order they were
/// attached, each from the host as it was.
#[test]
fn a_host_leaving_unattaches_everything_on_it() {
    let mut game = setup_two_player_game();
    let (sword, bears) = equipped_bears(&mut game);
    let aura = put_on_battlefield(&mut game, aura_enchanting_your_creature("Probe Aura"), 0);
    assert!(game.attach(aura, bears));
    let (sword_ref, aura_ref, bears_ref) = (existence(&game, sword), existence(&game, aura), existence(&game, bears));
    game.record_events();

    destroy_all(&mut game, &[bears]);

    assert_eq!(
        attachment_records(&game),
        vec![
            unattached(sword_ref, bears_ref, UnattachRoute::HostLeft),
            unattached(aura_ref, bears_ref, UnattachRoute::HostLeft),
        ]
    );
}

/// Both ends leaving in one event unattach once, by whichever moved first.
#[test]
fn an_equipment_and_its_host_leaving_together_unattach_once() {
    let mut game = setup_two_player_game();
    let (sword, bears) = equipped_bears(&mut game);
    let (sword_ref, bears_ref) = (existence(&game, sword), existence(&game, bears));
    game.record_events();

    destroy_all(&mut game, &[bears, sword]);

    assert_eq!(attachment_records(&game), vec![unattached(sword_ref, bears_ref, UnattachRoute::HostLeft)]);
}

/// CR 800.4a: a host leaving the game with its owner unattaches what another
/// player owns, which stays (CR 701.3d's "the object leaves the zone it was
/// in").
#[test]
fn a_host_leaving_the_game_unattaches_an_equipment_another_player_owns() {
    let mut game = setup_game(3);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let sword = put_on_battlefield(&mut game, equipment("Probe Sword"), 0);
    assert!(game.attach(sword, bears));
    let (sword_ref, bears_ref) = (existence(&game, sword), existence(&game, bears));
    game.record_events();

    game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).unwrap();

    assert!(game.battlefield.contains_key(&sword), "player 0's Equipment stays");
    assert_eq!(attachment_records(&game), vec![unattached(sword_ref, bears_ref, UnattachRoute::HostLeft)]);
}

// ---------------------------------------------------------------------------
// 2. The state-based detaches and the token sweep in CR 704.3's one event
// ---------------------------------------------------------------------------

/// The census's case (`feedback-loops.md` §4): a creature that makes an
/// attached Equipment a creature dies in the check where CR 704.5p first sees
/// the Equipment as one. The check reads the board before it, so the
/// Equipment is unattached at this check, not left on the creature.
#[test]
fn an_equipment_animated_by_a_creature_dying_in_the_check_is_unattached_at_that_check() {
    let mut game = setup_two_player_game();
    let (sword, bears) = equipped_bears(&mut game);
    let animator = put_on_battlefield(&mut game, animator(equipment_subtype(), &[]), 0);
    mark_damage(&mut game, animator, 1);
    let (sword_ref, bears_ref) = (existence(&game, sword), existence(&game, bears));
    game.record_events();

    assert!(game.check_state_based_actions(&test_dp()).unwrap());

    assert!(!game.battlefield.contains_key(&animator));
    assert_eq!(host(&game, sword), None, "CR 704.5p read the Equipment as the creature it was");
    let records = game.recorded_events();
    let record = records.records().iter().find(|r| matches!(r.event, GameEvent::Unattached { .. })).expect("unattached");
    assert_eq!(record.event, unattached(sword_ref, bears_ref, UnattachRoute::Unattach));
    assert_eq!(record.stamp.cause, EventCause::StateBasedAction(StateBasedAction::NonAttachmentAttached));
}

/// The same board with an Aura: unattached at this check, then CR 704.5m puts
/// it into its owner's graveyard at the next (CR 303.4d).
#[test]
fn an_aura_animated_by_a_creature_dying_in_the_check_is_unattached_then_put_into_the_graveyard() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let aura = put_on_battlefield(&mut game, aura_enchanting_your_creature("Probe Aura"), 0);
    assert!(game.attach(aura, bears));
    let animator = put_on_battlefield(&mut game, animator(Subtype::Enchantment(EnchantmentType::Aura), &[]), 0);
    mark_damage(&mut game, animator, 1);

    assert!(game.check_state_based_actions(&test_dp()).unwrap());
    assert_eq!(host(&game, aura), None, "unattached at the check that saw it a creature");

    assert!(game.check_state_based_actions(&test_dp()).unwrap());
    assert_eq!(game.objects[&aura].zone, Zone::Graveyard, "CR 704.5m at the next");
}

/// A destroyed animated Equipment leaves once and is unattached once: by its
/// leaving, which the check's CR 704.5p member then finds already done.
#[test]
fn an_animated_equipment_destroyed_in_the_check_is_unattached_once() {
    let mut game = setup_two_player_game();
    let (sword, bears) = equipped_bears(&mut game);
    put_on_battlefield(&mut game, animator(equipment_subtype(), &[]), 0);
    mark_damage(&mut game, sword, 2);
    let (sword_ref, bears_ref) = (existence(&game, sword), existence(&game, bears));
    game.record_events();

    assert!(game.check_state_based_actions(&test_dp()).unwrap());

    assert_eq!(game.objects[&sword].zone, Zone::Graveyard);
    assert_eq!(attachment_records(&game), vec![unattached(sword_ref, bears_ref, UnattachRoute::AttachmentLeft)]);
}

/// CR 704.7 collapses members with the same result, and an unattach is not a
/// destruction's result: an indestructible animated Equipment with lethal
/// damage survives CR 704.5g and is still unattached by CR 704.5p in the
/// same check.
#[test]
fn an_indestructible_animated_equipment_with_lethal_damage_is_unattached_in_the_same_check() {
    let mut game = setup_two_player_game();
    let (sword, _) = equipped_bears(&mut game);
    put_on_battlefield(&mut game, animator(equipment_subtype(), &[KeywordFlag::Indestructible]), 0);
    mark_damage(&mut game, sword, 2);

    assert!(game.check_state_based_actions(&test_dp()).unwrap());

    assert!(game.battlefield.contains_key(&sword), "CR 702.12b");
    assert_eq!(host(&game, sword), None);
}

/// CR 704.5d reads the board the check began with: a token that dies in a
/// check is in its owner's graveyard until the next, and ceases to exist
/// there (CR 704.3, 117.5).
#[test]
fn a_token_that_dies_in_a_check_ceases_to_exist_at_the_next() {
    let mut game = setup_two_player_game();
    let token = put_on_battlefield(&mut game, grizzly_bears(), 0);
    game.objects.get_mut(&token).unwrap().is_token = true;
    mark_damage(&mut game, token, 2);

    assert!(game.check_state_based_actions(&test_dp()).unwrap());
    assert_eq!(game.objects.get(&token).map(|o| o.zone), Some(Zone::Graveyard), "died at the first check");

    assert!(game.check_state_based_actions(&test_dp()).unwrap());
    assert!(!game.objects.contains_key(&token), "ceased to exist at the next");
    assert!(!game.check_state_based_actions(&test_dp()).unwrap());
}

/// CR 301.6: a Fortification fortifies a land, so one on a land stays, and
/// one on a creature that is no land is unattached (CR 704.5n).
#[test]
fn a_fortification_stays_on_a_land_and_is_unattached_from_a_creature() {
    let mut game = setup_two_player_game();
    let fortification = || {
        CardDataBuilder::new("Probe Fortification")
            .card_type(CardType::Artifact)
            .subtype(Subtype::Artifact(ArtifactType::Fortification))
            .build()
    };
    let land = put_on_battlefield(&mut game, card_of_type("Probe Land", CardType::Land), 0);
    let on_land = put_on_battlefield(&mut game, fortification(), 0);
    assert!(game.attach(on_land, land));
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let on_bears = put_on_battlefield(&mut game, fortification(), 0);
    assert!(game.attach(on_bears, bears));

    game.check_state_based_actions(&test_dp()).unwrap();

    assert_eq!(host(&game, on_land), Some(land));
    assert_eq!(host(&game, on_bears), None);
}

// ---------------------------------------------------------------------------
// 3. An Aura entering attached (`codebase-state.md` item 228)
// ---------------------------------------------------------------------------

/// Holy Strength on the stack targeting `target`, as `cast_spell` leaves it
/// after CR 601.2c: its enchant clause (CR 303.4a) is its one instance.
fn aura_spell_targeting(game: &mut GameState, controller: PlayerId, target: ObjectId) -> ObjectId {
    let aura = put_spell_on_stack(game, holy_strength(), controller);
    let enchant = holy_strength().spell_instances.last().cloned().expect("an Aura announces its enchant clause");
    let chosen = TargetInstance::from_announcement(game, enchant, &[ResolvedTarget::Object(target)]).unwrap();
    game.stack_entries.get_mut(&aura).unwrap().chosen_targets = vec![chosen];
    aura
}

/// The batch each of `id`'s records is in, in order, with the record's kind.
fn records_of(game: &GameState, id: ObjectId) -> Vec<(&'static str, Option<mtgsim::events::event::BatchId>)> {
    game.recorded_events()
        .records()
        .iter()
        .filter_map(|r| {
            let kind = match &r.event {
                GameEvent::ZoneChange { object_id, .. } if *object_id == id => "move",
                GameEvent::PermanentEnteredBattlefield { object_id, .. } if *object_id == id => "entered",
                GameEvent::Attached { attachment, .. } if *attachment == id => "attached",
                _ => return None,
            };
            Some((kind, r.stamp.batch))
        })
        .collect()
}

/// CR 303.4f: an Aura spell enters attached to its target, and entering
/// attached is "becoming attached" (Siona, Captain of the Pyleas' first
/// ruling). Announced inside the entry's event, after the arrival.
#[test]
fn an_aura_spell_resolving_announces_that_it_became_attached() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let aura = aura_spell_targeting(&mut game, 0, bears);
    game.record_events();

    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(host(&game, aura), Some(bears));
    assert!(game.recorded_events().events().any(|e| *e == GameEvent::Attached { attachment: aura, host: bears, former_host: None }));
    let records = records_of(&game, aura);
    assert_eq!(records.iter().map(|(kind, _)| *kind).collect::<Vec<_>>(), vec!["move", "entered", "attached"]);
    assert!(records.iter().all(|(_, batch)| *batch == records[0].1), "one event (CR 303.4f's \"as it enters\")");
}

/// CR 303.4f's other door: an Aura returned to the battlefield enchants what
/// its controller chooses as it enters, and becomes attached in that event.
#[test]
fn an_aura_returned_to_the_battlefield_announces_that_it_became_attached() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let aura = put_in_graveyard(&mut game, holy_strength(), 0);
    let source = mtgsim::types::ids::new_object_id();
    let ctx = ResolutionContext {
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(aura)]),
        ..ResolutionContext::untargeted(source, 0, GeneratedBy::Spell)
    };
    let back = Effect::Atom(
        Primitive::ReturnToBattlefield { under: ReturnUnder::Owner, counters: Vec::new() },
        EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
    );
    game.record_events();

    game.resolve_effect(&back, &ctx, &test_dp()).unwrap();

    assert_eq!(host(&game, aura), Some(bears));
    let records = records_of(&game, aura);
    assert_eq!(records.iter().map(|(kind, _)| *kind).collect::<Vec<_>>(), vec!["move", "entered", "attached"]);
    assert!(records.iter().all(|(_, batch)| *batch == records[0].1));
}

/// CR 303.4h: a permanent that is no Aura, Equipment or Fortification enters
/// unattached, whatever the effect putting it there names.
#[test]
fn a_creature_put_onto_the_battlefield_attached_enters_unattached() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let card = put_in_graveyard(&mut game, grizzly_bears(), 0);
    let mods = EnterMods { attached_to: Some(bears), ..EnterMods::NONE };
    let entry = GameAction::EnterBattlefield { object: card, from: Some(Zone::Graveyard), controller: 0, mods, act: Some(ZoneChangeAct::Put) };
    game.record_events();

    game.execute_action(entry, &test_ctx()).unwrap();

    assert_eq!(host(&game, card), None);
    assert!(attachment_records(&game).is_empty());
}

