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
//! 4. Item 178: an equip ability attaches its own Equipment, by identity
//!    (CR 400.7).
//! 5. The two trigger events: "becomes attached" (CR 603.2e, read after the
//!    event) and "becomes unattached" (CR 603.10c, looking back), with
//!    Siona, Captain of the Pyleas' rulings on a fixture shaped like her.
//! 6. Grafted Wargear's three rulings, the first once per route.
//! 7. Bramble Elemental, "becomes attached"'s benign consumer.

use std::sync::Arc;

use mtgsim::cards::authoring::{becomes_attached, becomes_unattached, triggered_ability, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lh_cards::{bonesplitter, equip, holy_strength};
use mtgsim::cards::phase_tr4b_cards::{bramble_elemental, grafted_wargear};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::{ChosenTargets, TargetInstance};
use mtgsim::events::cause::GeneratedBy;
use mtgsim::engine::actions::GameAction;
use mtgsim::events::cause::{EventCause, StateBasedAction};
use mtgsim::events::event::{GameEvent, UnattachRoute};
use mtgsim::objects::card_data::{AbilityType, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_abilities, get_effective_controller};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    aura_enchanting_your_creature, card_of_type, effect_ctx, equipment, put_in_graveyard, put_on_battlefield,
    put_spell_on_stack, setup_game, setup_two_player_game, static_ability, test_ctx, test_dp, RecordingDecisionProvider,
};
use mtgsim::types::card_types::{ArtifactType, CardType, EnchantmentType, Subtype};
use mtgsim::types::effects::{
    PlayerSet, Antecedent, AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, ReturnUnder, SelectionFilter,
    TargetCount, TypeChange,
};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, ObjectRef, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::ManaType;
use mtgsim::types::replacement::EnterMods;
use mtgsim::types::triggers::{EventRole, TriggerSubject};
use mtgsim::types::zones::{Zone, ZoneChangeAct};
use mtgsim::ui::decision::DecisionProvider;

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

/// Every attach and unattach the game recorded, in order, each unattach
/// without the frame of an end that left, which TR-5d's tests read.
fn attachment_records(game: &GameState) -> Vec<GameEvent> {
    game.recorded_events()
        .events()
        .filter(|e| matches!(e, GameEvent::Attached { .. } | GameEvent::Unattached { .. }))
        .map(|e| match e {
            GameEvent::Unattached { attachment, former_host, route, .. } => unattached(*attachment, *former_host, *route),
            other => other.clone(),
        })
        .collect()
}

fn unattached(attachment: ObjectRef, former_host: ObjectRef, route: UnattachRoute) -> GameEvent {
    GameEvent::Unattached { attachment, former_host, route, attachment_left: false, former_host_left: false }
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
            GameEvent::Attached { attachment: sword_ref, host: game.object_ref(other).unwrap(), former_host: Some(bears_ref) },
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
            GameEvent::ZoneChange { departed, .. } if departed.object().id == sword => Some("move"),
            GameEvent::Unattached { .. } => Some("unattached"),
            _ => None,
        })
        .collect();
    assert_eq!(kinds, vec!["move", "unattached"]);
    assert_ne!(existence(&game, sword), sword_ref, "the record names the permanent, not the graveyard card");
}

/// A host leaving unattaches everything on it, each from the host as it
/// was. The records are one event, so the CR gives them no order; this pins
/// the engine's (`attached_by`'s, the order they were attached), since an
/// order that reaches a prompt must be the same in every process.
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

/// The census's case (`feedback-loops.md` §4). Before the check, Probe
/// Animator makes the Equipment a 2/2 creature, the Equipment is on Grizzly
/// Bears, and the Animator has lethal damage. One check sees two conditions
/// on that board (CR 704.3): the Animator dies (704.5g), and a creature is
/// attached to something, so the Equipment becomes unattached (704.5p). Both
/// happen. Read after the death instead, the Equipment is no creature, and
/// nothing would ever unattach it from the Bears.
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
    assert_eq!(record.event, unattached(sword_ref, bears_ref, UnattachRoute::AttachedToNothing));
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
    let chosen = TargetInstance::from_announcement(game, enchant, &[ObjectOrPlayerId::Object(target)]).unwrap();
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
                GameEvent::ZoneChange { departed, .. } if departed.object().id == id => "move",
                GameEvent::PermanentEnteredBattlefield { object, .. } if object.id == id => "entered",
                GameEvent::Attached { attachment, .. } if attachment.id == id => "attached",
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
    assert!(game.recorded_events().events().any(|e| *e == GameEvent::Attached { attachment: game.object_ref(aura).unwrap(), host: game.object_ref(bears).unwrap(), former_host: None }));
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
        targets: ChosenTargets::one(vec![ObjectOrPlayerId::Object(aura)]),
        ..ResolutionContext::untargeted(source, 0, GeneratedBy::Spell)
    };
    let back = Effect::Atom(
        Primitive::ReturnToBattlefield { under: ReturnUnder::Owner, counters: Vec::new(), tapped: false, attacking: None },
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

// ---------------------------------------------------------------------------
// 4. An equip ability attaches the Equipment it belongs to (item 178)
// ---------------------------------------------------------------------------

/// CR 400.7: an Equipment flickered in response to its equip ability is a
/// new object, which the old ability does not attach (CR 301.5b: nothing
/// happens when the Equipment has left).
#[test]
fn an_equip_ability_does_not_attach_its_equipment_once_it_has_left_and_returned() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let splitter = put_on_battlefield(&mut game, bonesplitter(), 0);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);
    let equip = get_effective_abilities(&game, splitter)
        .iter()
        .position(|a| a.ability_type == AbilityType::Activated)
        .expect("Bonesplitter's equip");
    game.activate_ability(0, splitter, equip, &test_dp()).unwrap();
    assert_eq!(game.stack.len(), 1, "one creature, so the target was forced");

    let before = existence(&game, splitter);
    game.change_zone(splitter, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).unwrap();
    game.change_zone(splitter, Zone::Battlefield, ZoneChangeAct::Put, &test_ctx()).unwrap();
    assert_ne!(existence(&game, splitter), before, "a new object (CR 400.7)");
    game.record_events();

    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(host(&game, splitter), None);
    assert_eq!(game.battlefield[&bears].attached_by, Vec::<ObjectId>::new());
    assert!(attachment_records(&game).is_empty());
}

// ---------------------------------------------------------------------------
// 5. "Becomes attached" and "becomes unattached" (CR 603.2e, 603.10c)
// ---------------------------------------------------------------------------

fn gain(n: u64) -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(n)), EffectRecipient::Player(PlayerRef::You))
}

fn a_creature() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Creature)
}

/// An Equipment with Equip {0} that watches itself both ways, with two
/// amounts so one life total says what fired: 1 per "becomes attached to a
/// creature", 10 per "becomes unattached from a creature".
fn watching_equipment() -> Arc<CardData> {
    CardDataBuilder::new("Probe Watching Equipment")
        .card_type(CardType::Artifact)
        .subtype(equipment_subtype())
        .ability(triggered_ability(
            "Whenever this Equipment becomes attached to a creature, you gain 1 life.",
            whenever(becomes_attached(TriggerSubject::Object(Antecedent::ThisObject), a_creature()), gain(1)),
        ))
        .ability(triggered_ability(
            "Whenever this Equipment becomes unattached from a creature, you gain 10 life.",
            whenever(becomes_unattached(TriggerSubject::Object(Antecedent::ThisObject), a_creature()), gain(10)),
        ))
        .ability(equip("Equip {0}", Vec::new()))
        .build()
}

/// Put every waiting trigger on the stack and resolve the stack.
fn resolve_all(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(dp).unwrap();
        game.perform_sba_and_triggers(dp).unwrap();
    }
}

/// Activate `equipment`'s equip at the `pick`th creature and resolve it.
fn equip_to(game: &mut GameState, equipment: ObjectId, pick: usize) {
    let index = get_effective_abilities(game, equipment)
        .iter()
        .position(|a| a.ability_type == AbilityType::Activated)
        .expect("an equip ability");
    let dp = RecordingDecisionProvider::picking(pick);
    game.activate_ability(0, equipment, index, &dp).unwrap();
    game.resolve_top_of_stack(&dp).unwrap();
}

fn life(game: &GameState) -> i64 {
    game.players[0].life_total
}

/// The watching Equipment on the first of two Grizzly Bears, its trigger for
/// that attach already resolved.
fn watched_on_bears(game: &mut GameState) -> (ObjectId, ObjectId, ObjectId) {
    let first = put_on_battlefield(game, grizzly_bears(), 0);
    let second = put_on_battlefield(game, grizzly_bears(), 0);
    let gear = put_on_battlefield(game, watching_equipment(), 0);
    equip_to(game, gear, 0);
    resolve_all(game, &test_dp());
    assert_eq!(host(game, gear), Some(first));
    (gear, first, second)
}

// COVERS: ATOM-603.2e-002
#[test]
fn equipping_the_creature_it_is_already_attached_to_triggers_nothing() {
    let mut game = setup_two_player_game();
    let (gear, first, _) = watched_on_bears(&mut game);
    let before = life(&game);

    equip_to(&mut game, gear, 0);
    resolve_all(&mut game, &test_dp());

    assert_eq!(host(&game, gear), Some(first));
    assert_eq!(life(&game), before, "neither \"becomes attached\" nor \"becomes unattached\" (CR 603.2e, 701.3b)");
}

/// Moving the Equipment unattaches it from the first creature, and "that
/// creature" is the first one (CR 701.3d).
// COVERS: ATOM-603.10c-002
#[test]
fn moving_an_equipment_to_another_creature_triggers_its_unattach_from_the_first() {
    let mut game = setup_two_player_game();
    let (gear, first, _) = watched_on_bears(&mut game);
    let first_ref = existence(&game, first);

    equip_to(&mut game, gear, 1);

    let unattached: Vec<_> = game
        .pending_triggers
        .iter()
        .filter(|p| p.origin.source() == gear && p.binding.records().any(|r| matches!(r.event, GameEvent::Unattached { .. })))
        .collect();
    assert_eq!(unattached.len(), 1);
    assert_eq!(unattached[0].binding.triggering_object(EventRole::Host), Some(first_ref), "\"that creature\" is the one it left");
}

/// One equip, two triggers: unattached from the first, attached to the
/// second.
// COVERS: ATOM-603.10c-003
#[test]
fn moving_an_equipment_to_another_creature_triggers_both_ways() {
    let mut game = setup_two_player_game();
    let (gear, _, second) = watched_on_bears(&mut game);
    let before = life(&game);

    equip_to(&mut game, gear, 1);
    resolve_all(&mut game, &RecordingDecisionProvider::picking(0));

    assert_eq!(host(&game, gear), Some(second));
    assert_eq!(life(&game), before + 11);
}

/// The host leaving unattaches the Equipment, and the trigger reads the
/// creature as it was before it left (CR 603.10c). Here it was a creature
/// only while an effect animated it on the battlefield, so "from a creature"
/// holds of how it was and not of the artifact card in the graveyard.
// COVERS: ATOM-603.10c-001
#[test]
fn an_equipment_whose_creature_is_destroyed_triggers_looking_back() {
    let mut game = setup_two_player_game();
    let clue = Subtype::Artifact(ArtifactType::Clue);
    put_on_battlefield(&mut game, animator(clue.clone(), &[]), 0);
    let construct = put_on_battlefield(
        &mut game,
        CardDataBuilder::new("Probe Clue").card_type(CardType::Artifact).subtype(clue).build(),
        0,
    );
    let gear = put_on_battlefield(&mut game, watching_equipment(), 0);
    equip_to(&mut game, gear, 1);
    resolve_all(&mut game, &test_dp());
    assert_eq!(host(&game, gear), Some(construct), "the animated Clue, the second creature");
    let before = life(&game);

    destroy_all(&mut game, &[construct]);
    resolve_all(&mut game, &test_dp());

    assert_eq!(host(&game, gear), None);
    assert_eq!(life(&game), before + 10);
}

/// An Equipment that leaves triggers off its own ability as it was (CR
/// 603.10c), the way a leaves-the-battlefield ability does.
#[test]
fn an_equipment_that_leaves_triggers_its_own_unattach() {
    let mut game = setup_two_player_game();
    let (gear, _, _) = watched_on_bears(&mut game);
    let before = life(&game);

    destroy_all(&mut game, &[gear]);
    resolve_all(&mut game, &test_dp());

    assert_eq!(game.objects[&gear].zone, Zone::Graveyard);
    assert_eq!(life(&game), before + 10);
}

/// Siona, Captain of the Pyleas' last ability on a vanilla 2/2, gaining 1
/// life where she makes a Soldier: "Whenever an Aura you control becomes
/// attached to a creature you control".
fn siona_shape() -> Arc<CardData> {
    let yours = |filter: ObjectFilter| ObjectFilter::And(Box::new(filter), Box::new(ObjectFilter::ByController(PlayerSet::You)));
    let aura = ObjectFilter::BySubtype(Subtype::Enchantment(EnchantmentType::Aura));
    CardDataBuilder::new("Probe Siona")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(triggered_ability(
            "Whenever an Aura you control becomes attached to a creature you control, you gain 1 life.",
            whenever(becomes_attached(yours(aura), yours(a_creature())), gain(1)),
        ))
        .build()
}

/// Siona's first ruling, its three cases: an Aura entering attached
/// triggers her, one moving from another object triggers her, and an
/// attach to the creature it already enchants does not.
#[test]
fn an_aura_entering_attached_or_moving_triggers_siona_and_one_left_in_place_does_not() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, siona_shape(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let other = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let start = life(&game);

    let aura = aura_spell_targeting(&mut game, 0, bears);
    game.resolve_top_of_stack(&test_dp()).unwrap();
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game), start + 1, "entering attached is becoming attached");

    attach(&mut game, aura, other);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game), start + 2, "moving from another object is too");

    attach(&mut game, aura, other);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game), start + 2, "attaching it where it already is is not");
}

/// An Aura that reads "You control enchanted creature" and enchants a
/// creature or artifact, for Siona's fourth ruling.
fn control_aura() -> Arc<CardData> {
    CardDataBuilder::new("Probe Control Aura")
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .enchant_filter(SelectionFilter::Creature)
        .ability(static_ability(Effect::Atom(
            Primitive::GainControl { to: PlayerRef::You, duration: Duration::WhileSourceOnBattlefield },
            EffectRecipient::Object(Antecedent::ThisObjectsHost),
        )))
        .build()
}

/// Siona's fourth ruling: the trigger reads the board after the attach. An
/// Aura that makes an opponent's creature yours triggers her.
#[test]
fn an_aura_that_takes_a_creature_triggers_siona_as_the_creature_it_made_yours() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, siona_shape(), 0);
    let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let start = life(&game);

    let aura = put_spell_on_stack(&mut game, control_aura(), 0);
    let enchant = control_aura().spell_instances.last().cloned().expect("an Aura announces its enchant clause");
    let chosen = TargetInstance::from_announcement(&game, enchant, &[ObjectOrPlayerId::Object(theirs)]).unwrap();
    game.stack_entries.get_mut(&aura).unwrap().chosen_targets = vec![chosen];
    game.resolve_top_of_stack(&test_dp()).unwrap();
    resolve_all(&mut game, &test_dp());

    assert_eq!(get_effective_controller(&game, theirs), Some(0));
    assert_eq!(life(&game), start + 1);
}

/// An Equipment whose creature has "Whenever an Equipment becomes unattached
/// from this creature, you gain 5 life", with Equip {0}.
fn granting_equipment() -> Arc<CardData> {
    let granted = triggered_ability(
        "Whenever an Equipment becomes unattached from this creature, you gain 5 life.",
        whenever(becomes_unattached(ObjectFilter::BySubtype(equipment_subtype()), TriggerSubject::Object(Antecedent::ThisObject)), gain(5)),
    );
    CardDataBuilder::new("Probe Granting Equipment")
        .card_type(CardType::Artifact)
        .subtype(equipment_subtype())
        .ability(static_ability(Effect::Atom(
            Primitive::GrantAbility(Arc::new(granted), Duration::WhileSourceOnBattlefield),
            EffectRecipient::Object(Antecedent::ThisObjectsHost),
        )))
        .ability(equip("Equip {0}", Vec::new()))
        .build()
}

/// CR 603.10c looks back to "the existence of those abilities ... immediately
/// prior to the event": moving the Equipment takes the granted ability off
/// the first creature in the event that triggers it, and it still triggers.
#[test]
fn a_creature_whose_equipment_granted_its_unattach_trigger_triggers_as_the_equipment_moves() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 0);
    let gear = put_on_battlefield(&mut game, granting_equipment(), 0);
    equip_to(&mut game, gear, 0);
    resolve_all(&mut game, &test_dp());
    let before = life(&game);

    equip_to(&mut game, gear, 1);
    resolve_all(&mut game, &test_dp());

    assert_eq!(life(&game), before + 5, "the first creature's, as it was; the second's is about itself");
}

// ---------------------------------------------------------------------------
// 6. Grafted Wargear
// ---------------------------------------------------------------------------

/// Grafted Wargear on the first of two Grizzly Bears, both player 0's.
fn wargear_on_bears(game: &mut GameState) -> (ObjectId, ObjectId, ObjectId) {
    let first = put_on_battlefield(game, grizzly_bears(), 0);
    let second = put_on_battlefield(game, grizzly_bears(), 0);
    let wargear = put_on_battlefield(game, grafted_wargear(), 0);
    equip_to(game, wargear, 0);
    resolve_all(game, &test_dp());
    assert_eq!(host(game, wargear), Some(first));
    (wargear, first, second)
}

fn zone_of(game: &GameState, id: ObjectId) -> Option<Zone> {
    game.objects.get(&id).map(|o| o.zone)
}

// RULING: Grafted Wargear #1 - equipping it to a new creature unattaches it.
#[test]
fn grafted_wargear_moved_to_another_creature_sacrifices_the_first() {
    let mut game = setup_two_player_game();
    let (wargear, first, second) = wargear_on_bears(&mut game);
    game.record_events();

    equip_to(&mut game, wargear, 1);
    resolve_all(&mut game, &test_dp());

    assert_eq!(host(&game, wargear), Some(second));
    assert!(game.recorded_events().events().any(|e| {
        matches!(e, GameEvent::ZoneChange { departed, act: ZoneChangeAct::Sacrificed, .. } if departed.object().id == first)
    }));
}

// RULING: Grafted Wargear #1 - Grafted Wargear leaving the battlefield
//   unattaches it, and its own ability triggers.
#[test]
fn grafted_wargear_leaving_sacrifices_its_creature() {
    let mut game = setup_two_player_game();
    let (wargear, first, _) = wargear_on_bears(&mut game);

    destroy_all(&mut game, &[wargear]);
    resolve_all(&mut game, &test_dp());

    assert_eq!(zone_of(&game, first), Some(Zone::Graveyard));
}

// RULING: Grafted Wargear #1 - the equipped creature ceasing to be a
//   creature unattaches it (CR 704.5n), and the permanent it was on is
//   sacrificed.
#[test]
fn grafted_wargear_on_a_creature_that_stops_being_one_sacrifices_it() {
    let mut game = setup_two_player_game();
    let clue = Subtype::Artifact(ArtifactType::Clue);
    let animator = put_on_battlefield(&mut game, animator(clue.clone(), &[]), 0);
    let construct = put_on_battlefield(
        &mut game,
        CardDataBuilder::new("Probe Clue").card_type(CardType::Artifact).subtype(clue).build(),
        0,
    );
    let wargear = put_on_battlefield(&mut game, grafted_wargear(), 0);
    equip_to(&mut game, wargear, 1);
    resolve_all(&mut game, &test_dp());
    assert_eq!(host(&game, wargear), Some(construct));

    destroy_all(&mut game, &[animator]);
    resolve_all(&mut game, &test_dp());

    assert_eq!(host(&game, wargear), None);
    assert_eq!(zone_of(&game, construct), Some(Zone::Graveyard), "the artifact it was on");
}

// RULING: Grafted Wargear #1 - Grafted Wargear ceasing to be an Equipment
//   unattaches it (CR 704.5p).
#[test]
fn grafted_wargear_that_stops_being_an_equipment_sacrifices_its_creature() {
    let mut game = setup_two_player_game();
    let (wargear, first, _) = wargear_on_bears(&mut game);
    let not_equipment = TypeChange { remove_subtypes: vec![equipment_subtype()], ..TypeChange::NONE };
    let source = mtgsim::types::ids::new_object_id();
    let ctx = ResolutionContext {
        targets: ChosenTargets::one(vec![ObjectOrPlayerId::Object(wargear)]),
        ..ResolutionContext::untargeted(source, 0, GeneratedBy::Spell)
    };
    let strip = Effect::Atom(
        Primitive::ChangeType(not_equipment, Duration::UntilEndOfTurn),
        EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
    );
    game.resolve_effect(&strip, &ctx, &test_dp()).unwrap();

    resolve_all(&mut game, &test_dp());

    assert_eq!(host(&game, wargear), None);
    assert_eq!(zone_of(&game, first), Some(Zone::Graveyard));
}

// RULING: Grafted Wargear #1 - the equipped creature leaving unattaches it,
//   "but the triggered ability won't do anything in that case."
#[test]
fn grafted_wargear_whose_creature_left_triggers_and_sacrifices_nothing() {
    let mut game = setup_two_player_game();
    let (wargear, first, second) = wargear_on_bears(&mut game);

    destroy_all(&mut game, &[first]);
    assert_eq!(game.pending_triggers.iter().filter(|p| p.origin.source() == wargear).count(), 1, "it triggered");
    resolve_all(&mut game, &test_dp());

    assert_eq!(zone_of(&game, wargear), Some(Zone::Battlefield));
    assert_eq!(zone_of(&game, second), Some(Zone::Battlefield));
    assert_eq!(zone_of(&game, first), Some(Zone::Graveyard));
}

/// CR 400.7: a creature that left and came back is a new object, which
/// "that permanent" does not name.
#[test]
fn grafted_wargear_does_not_sacrifice_its_creature_after_it_returned() {
    let mut game = setup_two_player_game();
    let (wargear, first, _) = wargear_on_bears(&mut game);
    let exile_it = EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1));
    let flicker = Effect::Sequence(vec![
        Effect::Atom(Primitive::Exile, exile_it.clone()),
        Effect::Atom(
            Primitive::ReturnToBattlefield { under: ReturnUnder::Owner, counters: Vec::new(), tapped: false, attacking: None },
            EffectRecipient::SameInstanceAs(0),
        ),
    ]);
    let spell = put_spell_on_stack(&mut game, card_of_type("Probe Flicker", CardType::Instant), 0);
    let chosen = TargetInstance::from_announcement(&game, exile_it, &[ObjectOrPlayerId::Object(first)]).unwrap();
    let entry = game.stack_entries.get_mut(&spell).unwrap();
    entry.effect = Arc::new(flicker);
    entry.chosen_targets = vec![chosen];

    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.pending_triggers.iter().filter(|p| p.origin.source() == wargear).count(), 1, "it triggered");
    resolve_all(&mut game, &test_dp());

    assert_eq!(zone_of(&game, first), Some(Zone::Battlefield));
}

/// An Equipment that sends "that permanent" back to its owner's hand: a
/// reader that, unlike a sacrifice, would act on a card in a graveyard.
fn bouncing_equipment() -> Arc<CardData> {
    CardDataBuilder::new("Probe Bouncing Equipment")
        .card_type(CardType::Artifact)
        .subtype(equipment_subtype())
        .ability(triggered_ability(
            "Whenever this Equipment becomes unattached from a permanent, return that permanent to its owner's hand.",
            whenever(
                becomes_unattached(TriggerSubject::Object(Antecedent::ThisObject), TriggerSubject::Filter(ObjectFilter::All)),
                Effect::Atom(Primitive::ReturnToHand, EffectRecipient::Object(Antecedent::Triggering(EventRole::Host))),
            ),
        ))
        .ability(equip("Equip {0}", Vec::new()))
        .build()
}

/// "That permanent" is the permanent (CR 400.7). A creature that died is a
/// card in a graveyard, which an unattach trigger does not find: no CR 400.7
/// exception reaches it, unlike an Aura's leaves-the-battlefield trigger (CR
/// 603.6e).
#[test]
fn an_unattach_trigger_does_not_find_the_card_its_creature_became() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let gear = put_on_battlefield(&mut game, bouncing_equipment(), 0);
    equip_to(&mut game, gear, 0);
    resolve_all(&mut game, &test_dp());

    destroy_all(&mut game, &[bears]);
    resolve_all(&mut game, &test_dp());

    assert_eq!(zone_of(&game, bears), Some(Zone::Graveyard));
}

// RULING: Grafted Wargear #2 - equipping it to the creature it is attached to
//   is legal, and "generally won't do anything".
#[test]
fn grafted_wargear_equipped_again_to_its_creature_sacrifices_nothing() {
    let mut game = setup_two_player_game();
    let wargear = put_on_battlefield(&mut game, grafted_wargear(), 0);
    let only = put_on_battlefield(&mut game, grizzly_bears(), 0);
    equip_to(&mut game, wargear, 0);
    resolve_all(&mut game, &test_dp());

    equip_to(&mut game, wargear, 0);
    resolve_all(&mut game, &test_dp());

    assert_eq!(host(&game, wargear), Some(only));
    assert_eq!(zone_of(&game, only), Some(Zone::Battlefield));
}

// RULING: Grafted Wargear #3 - its controller cannot sacrifice an opponent's
//   creature.
#[test]
fn grafted_wargear_does_not_sacrifice_an_opponents_creature() {
    let mut game = setup_two_player_game();
    let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let mine = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let wargear = put_on_battlefield(&mut game, grafted_wargear(), 0);
    attach(&mut game, wargear, theirs);
    resolve_all(&mut game, &test_dp());

    equip_to(&mut game, wargear, 0);
    resolve_all(&mut game, &test_dp());

    assert_eq!(host(&game, wargear), Some(mine));
    assert_eq!(zone_of(&game, theirs), Some(Zone::Battlefield), "player 0 does not control it");
}

// ---------------------------------------------------------------------------
// 7. Bramble Elemental
// ---------------------------------------------------------------------------

fn saprolings(game: &GameState) -> usize {
    game.battlefield_ids_ordered()
        .into_iter()
        .filter(|id| game.objects.get(id).is_some_and(|o| o.is_token))
        .count()
}

/// An Aura spell resolving onto it is an Aura becoming attached to it (CR
/// 303.4f), and one moved onto it from another creature is too.
#[test]
fn bramble_elemental_makes_saprolings_for_an_aura_entering_onto_it_or_moved_onto_it() {
    let mut game = setup_two_player_game();
    let bramble = put_on_battlefield(&mut game, bramble_elemental(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);

    let aura = aura_spell_targeting(&mut game, 0, bramble);
    game.resolve_top_of_stack(&test_dp()).unwrap();
    resolve_all(&mut game, &test_dp());
    assert_eq!(saprolings(&game), 2);

    attach(&mut game, aura, bears);
    resolve_all(&mut game, &test_dp());
    assert_eq!(saprolings(&game), 2, "an Aura leaving it is no Aura becoming attached to it");

    attach(&mut game, aura, bramble);
    resolve_all(&mut game, &test_dp());
    assert_eq!(saprolings(&game), 4);
}

/// CR 603.2e: an Aura attached to it again where it already is, or onto
/// another creature, makes nothing.
#[test]
fn bramble_elemental_makes_nothing_for_an_aura_left_in_place_or_put_on_another_creature() {
    let mut game = setup_two_player_game();
    let bramble = put_on_battlefield(&mut game, bramble_elemental(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);

    aura_spell_targeting(&mut game, 0, bears);
    game.resolve_top_of_stack(&test_dp()).unwrap();
    resolve_all(&mut game, &test_dp());
    assert_eq!(saprolings(&game), 0);

    let aura = put_on_battlefield(&mut game, holy_strength(), 0);
    assert!(game.attach(aura, bramble));
    attach(&mut game, aura, bramble);
    resolve_all(&mut game, &test_dp());
    assert_eq!(saprolings(&game), 0);
}
