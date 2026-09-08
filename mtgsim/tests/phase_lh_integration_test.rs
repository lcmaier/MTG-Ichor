//! Phase LH-1 — the Aura host becomes addressable
//! (`layers-architecture.md` §13a).
//!
//! Holy Strength is the card; `AffectedSet::Host` is what it
//! consumes. Every board below is built from the registered card rather than
//! a fixture Aura, because the claim being tested is that a *real* Aura's
//! static ability reaches its host through the layer walk — the fixtures in
//! `test_support` (Pacifism, "Test Bond") carry no ability and could not tell
//! a working row from a missing one.

use mtgsim::cards::phase_lh_cards::holy_strength;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::actions::{DestructionSource, GameAction, ZoneChangeCause};
use mtgsim::engine::resolve::ResolvedTarget;
use mtgsim::engine::targeting::spell_recipient;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::{
    get_effective_controller, get_effective_power, get_effective_toughness,
};
use mtgsim::oracle::mana_helpers::castable_spells;
use mtgsim::state::game_state::{GameState, StackEntry};
use mtgsim::test_support::{
    equipment, put_in_hand, put_on_battlefield, setup_two_player_game, test_ctx, test_dp,
    vanilla_creature,
};
use mtgsim::types::effects::{Effect, EffectRecipient, SelectionFilter, TargetCount};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pt(game: &GameState, id: ObjectId) -> (i32, i32) {
    (
        get_effective_power(game, id).expect("has power"),
        get_effective_toughness(game, id).expect("has toughness"),
    )
}

/// Holy Strength on the stack targeting `target`, as `cast_spell` leaves it
/// after CR 601.2c: a permanent spell with no spell ability carries an empty
/// `Sequence`, and the Aura's target lives in `chosen_targets`.
fn aura_on_stack_targeting(game: &mut GameState, controller: PlayerId, target: ObjectId) -> ObjectId {
    let obj = GameObject::new(holy_strength(), controller, Zone::Stack);
    let id = obj.id;
    game.add_object(obj);
    game.stack.push(id);
    game.set_stack_entry(StackEntry {
        object_id: id,
        controller,
        chosen_targets: vec![ResolvedTarget::Object(target)],
        recipient: spell_recipient(&holy_strength()),
        chosen_modes: Vec::new(),
        x_value: None,
        effect: Effect::Sequence(Vec::new()),
        is_spell: true,
        chosen_alternative_cost: None,
        additional_costs_paid: Vec::new(),
        cast_from: Some(Zone::Hand),
        ability_identity: None,
    });
    id
}

fn destroy(game: &mut GameState, id: ObjectId) {
    let source = mtgsim::types::ids::new_object_id();
    game.execute_action(
        GameAction::Destroy { object: id, source: DestructionSource::Effect(source) },
        &test_ctx(),
    )
    .expect("it is destroyed");
}

/// Every zone change of `id`, as `(from, to, cause)`.
fn moves_of(game: &GameState, id: ObjectId) -> Vec<(Zone, Zone, ZoneChangeCause)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::ZoneChange { object_id, from, to, cause, .. } if *object_id == id => {
                Some((*from, *to, *cause))
            }
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// CR 303.4m — "enchanted creature" is whatever the Aura is attached to
// ---------------------------------------------------------------------------

/// The row itself. Asked *before* the attach as well as after: the first query
/// caches a frame at the pre-attach epoch, so a writer of `attached_to` that
/// skipped its bump would serve 2/2 here in release and panic in debug.
///
/// No `COVERS`, deliberately: the corpus classifies CR 303.4m as PURE-DEF
/// ("naming convention for effect resolution … no independent testable
/// behavior beyond the attachment system", session-3), so there is no atom
/// for it. The atoms this row makes reachable are the CR 303.4 / 608.3
/// scenarios below, which claim theirs.
#[test]
fn test_holy_strength_gives_its_host_plus_one_plus_two() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let other = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let aura = put_on_battlefield(&mut game, holy_strength(), 0);

    assert_eq!(pt(&game, bears), (2, 2), "unattached, the Aura names nothing");

    game.attach(aura, bears);
    assert_eq!(pt(&game, bears), (3, 4));
    assert_eq!(pt(&game, other), (2, 2), "only the enchanted creature");
}

/// The bonus goes with the Aura, not with the creature it first met.
#[test]
fn test_the_bonus_leaves_when_the_aura_does() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let aura = put_on_battlefield(&mut game, holy_strength(), 0);
    game.attach(aura, bears);
    assert_eq!(pt(&game, bears), (3, 4));

    destroy(&mut game, aura);

    assert_eq!(pt(&game, bears), (2, 2));
    assert!(
        game.battlefield[&bears].attached_by.is_empty(),
        "the host's back-pointer is cleaned up as the Aura leaves"
    );
}

// ---------------------------------------------------------------------------
// CR 303.4 / 608.3c — an Aura spell enters attached to its target
// ---------------------------------------------------------------------------

/// The target is an opponent's creature on purpose: CR 303.4e's third clause
/// — the caster controls the Aura, whoever controls what it enchants — is the
/// same board, and the bonus reaches across the table.
// COVERS: ATOM-303.4-001, ATOM-608.3c-001, ATOM-303.4e-003
#[test]
fn test_an_aura_spell_resolves_attached_to_its_target() {
    let mut game = setup_two_player_game();
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let aura = aura_on_stack_targeting(&mut game, 0, theirs);

    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    assert_eq!(game.get_object(aura).unwrap().zone, Zone::Battlefield);
    assert_eq!(game.battlefield[&aura].attached_to, Some(theirs));
    assert!(game.battlefield[&theirs].attached_by.contains(&aura));
    assert_eq!(get_effective_controller(&game, aura), Some(0), "CR 303.4e: the caster's");
    assert_eq!(get_effective_controller(&game, theirs), Some(1), "and the creature stays P1's");
    assert_eq!(pt(&game, theirs), (3, 4));
    assert_eq!(
        moves_of(&game, aura),
        vec![(Zone::Stack, Zone::Battlefield, ZoneChangeCause::Resolved)]
    );
}

// ---------------------------------------------------------------------------
// CR 303.4c / 704.5m — the Aura follows its host into the graveyard
// ---------------------------------------------------------------------------

// COVERS: ATOM-303.4c-002
#[test]
fn test_the_aura_dies_with_its_host() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let aura = aura_on_stack_targeting(&mut game, 0, bears);
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");

    destroy(&mut game, bears);
    assert_eq!(
        game.battlefield[&aura].attached_to,
        None,
        "the host's departure unattaches it before any SBA looks"
    );

    let performed = game.check_state_based_actions(&test_dp()).unwrap();
    assert!(performed, "CR 704.5m has something to do");
    assert!(!game.battlefield.contains_key(&aura));
    assert!(
        game.players[0].graveyard.contains(&aura),
        "CR 303.4c: to its *owner's* graveyard, not the host's controller's"
    );
    assert_eq!(
        moves_of(&game, aura).last().copied(),
        Some((Zone::Battlefield, Zone::Graveyard, ZoneChangeCause::AuraSba))
    );
}

// ---------------------------------------------------------------------------
// CR 608.3b — a permanent spell whose target is gone fizzles
// ---------------------------------------------------------------------------

/// `codebase-state.md` Deferred Migrations item 8. An Aura has no spell
/// ability, so a fizzle check that derives the recipient from the *effect*
/// sees `Implicit`, asks nothing, and lets the Aura enter the battlefield
/// with a target that no longer exists.
// COVERS: ATOM-608.3b-001, ATOM-303.4g-002
#[test]
fn test_an_aura_whose_target_left_fizzles() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let aura = aura_on_stack_targeting(&mut game, 0, bears);

    destroy(&mut game, bears);
    game.resolve_top_of_stack(&test_dp())
        .expect("resolution runs; the spell is countered by the rules, not an error");

    assert!(!game.battlefield.contains_key(&aura), "CR 608.3b: it never enters");
    assert!(game.players[0].graveyard.contains(&aura), "to its owner's graveyard");
    assert_eq!(
        moves_of(&game, aura),
        vec![(Zone::Stack, Zone::Graveyard, ZoneChangeCause::Fizzled)]
    );
    assert!(game
        .events
        .events()
        .any(|e| matches!(e, GameEvent::SpellFizzled { spell_id } if *spell_id == aura)));
}

// ---------------------------------------------------------------------------
// CR 303.4a / 601.2c — an Aura spell's target is defined by its enchant ability
// ---------------------------------------------------------------------------

/// Holy Strength in `player`'s hand with the {W} to cast it floating.
fn holy_strength_in_hand(game: &mut GameState, player: PlayerId) -> ObjectId {
    let id = put_in_hand(game, holy_strength(), player);
    game.players[player].mana_pool.add(ManaType::White, 1);
    id
}

/// Cast `aura` at index 0 of the legal targets, which each caller makes
/// unambiguous by leaving exactly one creature on the battlefield.
fn cast_at_the_only_creature(game: &mut GameState, player: PlayerId, aura: ObjectId) {
    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id: aura,
        },
        vec![0],
    );
    game.cast_spell(player, aura, &decisions).expect("it is castable");
    assert!(decisions.is_empty(), "CR 601.2c asked for the target");
    game.resolve_top_of_stack(&decisions).expect("it resolves");
}

/// The whole cast path: the target is chosen at CR 601.2c from the enchant
/// ability — the card has no spell ability to take one from — and CR 608.3c
/// attaches to it.
// COVERS: ATOM-303.4-001
#[test]
fn test_an_aura_cast_from_hand_targets_at_601_2c_and_enters_attached() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let aura = holy_strength_in_hand(&mut game, 0);

    cast_at_the_only_creature(&mut game, 0, aura);

    assert_eq!(game.battlefield[&aura].attached_to, Some(bears));
    assert_eq!(pt(&game, bears), (3, 4));
}

/// No creature, no cast — and the castability pre-check the fuzz agent reads
/// agrees, which is what keeps a random game from trying.
// COVERS: ATOM-303.4a-001
#[test]
fn test_an_aura_with_no_legal_target_cannot_be_cast() {
    let mut game = setup_two_player_game();
    let aura = holy_strength_in_hand(&mut game, 0);

    assert!(
        castable_spells(&game, 0).iter().all(|(id, _)| *id != aura),
        "CR 601.2c's pre-check: a spell with no legal target is not castable"
    );
    assert!(game.cast_spell(0, aura, &test_dp()).is_err());
    assert_eq!(game.get_object(aura).unwrap().zone, Zone::Hand, "the cast rewound");
    assert!(game.stack.is_empty());
}

/// CR 702.5a — "Enchant creature" is the restriction. With only an artifact
/// to enchant the Aura cannot be cast at all; once a creature is there, that
/// is what it enchants and the artifact is never offered.
// COVERS: ATOM-702.5a-001
#[test]
fn test_enchant_creature_admits_only_creatures() {
    let mut game = setup_two_player_game();
    let trinket = put_on_battlefield(&mut game, equipment("Trinket"), 1);
    let aura = holy_strength_in_hand(&mut game, 0);

    assert!(castable_spells(&game, 0).iter().all(|(id, _)| *id != aura));
    assert!(game.cast_spell(0, aura, &test_dp()).is_err(), "an artifact is not a creature");

    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    cast_at_the_only_creature(&mut game, 0, aura);

    assert_eq!(game.battlefield[&aura].attached_to, Some(bears));
    assert!(game.battlefield[&trinket].attached_by.is_empty());
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// In the registry, so `fuzz_games --pool stress` can draw it, and in the
/// performance pool, so the new arm is measured rather than assumed
/// (`engineering-practices.md` §3).
#[test]
fn test_holy_strength_is_registered_and_in_the_performance_pool() {
    assert!(CardRegistry::default_registry().create("Holy Strength").is_ok());
    assert!(CardRegistry::performance_pool().create("Holy Strength").is_ok());
}

// ===========================================================================
// LH-2 — Equip (CR 702.6a), and the timestamp an attach gives (CR 613.7e)
// ===========================================================================

use mtgsim::cards::phase_lh_cards::bonesplitter;
use mtgsim::objects::card_data::AbilityType;
use mtgsim::oracle::characteristics::get_effective_abilities;
use mtgsim::oracle::mana_helpers::activatable_abilities;
use mtgsim::state::game_state::{Phase, PhaseType};
use mtgsim::cards::phase_lf_cards::humility;
use mtgsim::cards::phase_lh_cards::cobbled_wings;
use mtgsim::oracle::characteristics::has_keyword;
use mtgsim::types::keywords::KeywordFlag;

fn equip_ability_index(game: &GameState, equipment: ObjectId) -> usize {
    get_effective_abilities(game, equipment)
        .iter()
        .position(|a| a.ability_type == AbilityType::Activated)
        .expect("an Equipment with an equip ability")
}

/// Activate `equipment`'s equip with its {1} floating, pick the `pick`th legal
/// target, and resolve. The error is the activation's — a resolution failure
/// is a test bug.
fn equip(game: &mut GameState, player: PlayerId, equipment: ObjectId, pick: usize) -> Result<(), String> {
    game.players[player].mana_pool.add(ManaType::White, 1);
    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            spell_id: equipment,
        },
        vec![pick],
    );
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
        vec![1],
    );
    let idx = equip_ability_index(game, equipment);
    if let Err(e) = game.activate_ability(player, equipment, idx, &decisions) {
        // Refused before any question was asked; the provider asserts on drop
        // that every expectation was consumed, and here none should be.
        std::mem::forget(decisions);
        return Err(e);
    }
    game.resolve_top_of_stack(&decisions).expect("an activated equip resolves");
    Ok(())
}

fn offered_to(game: &GameState, player: PlayerId, equipment: ObjectId) -> bool {
    activatable_abilities(game, player).iter().any(|(id, _, _)| *id == equipment)
}

fn attaches_of(game: &GameState, attachment: ObjectId) -> Vec<(ObjectId, Option<ObjectId>)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::Attached { attachment: a, host, former_host } if *a == attachment => {
                Some((*host, *former_host))
            }
            _ => None,
        })
        .collect()
}

/// CR 702.6a in full: the ability is offered and legal only at sorcery speed
/// — active player, main phase, empty stack — and when it resolves the
/// Equipment is attached, both ends of the link written, and the bonus is on
/// the host. Each timing leg is checked against the window the random agent
/// reads as well as against the activation itself.
// COVERS: ATOM-702.6a-001, ATOM-702.6a-003, ATOM-301.5b-002, ATOM-301.5-001
#[test]
fn test_equip_is_offered_and_legal_only_at_sorcery_speed_and_then_attaches() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let splitter = put_on_battlefield(&mut game, bonesplitter(), 0);
    assert_eq!(pt(&game, bears), (2, 2), "unattached, the row names nothing");

    // Not a main phase.
    game.phase = Phase::new(PhaseType::Combat);
    assert!(!offered_to(&game, 0, splitter));
    assert!(equip(&mut game, 0, splitter, 0).is_err());

    // A main phase, but not the active player's.
    game.phase = Phase::new(PhaseType::Precombat);
    game.active_player = 1;
    assert!(!offered_to(&game, 0, splitter));
    assert!(equip(&mut game, 0, splitter, 0).is_err());

    // Active player, main phase, but the stack is not empty.
    game.active_player = 0;
    let on_stack = aura_on_stack_targeting(&mut game, 1, bears);
    assert!(!offered_to(&game, 0, splitter));
    assert!(equip(&mut game, 0, splitter, 0).is_err());
    assert_eq!(game.battlefield[&splitter].attached_to, None, "nothing attached on the way");
    let decisions = ScriptedDecisionProvider::new();
    game.resolve_top_of_stack(&decisions).expect("the Aura resolves");
    assert_eq!(game.battlefield[&on_stack].attached_to, Some(bears));

    // Sorcery speed.
    assert!(offered_to(&game, 0, splitter));
    equip(&mut game, 0, splitter, 0).expect("legal at sorcery speed");

    assert_eq!(game.battlefield[&splitter].attached_to, Some(bears));
    assert!(game.battlefield[&bears].attached_by.contains(&splitter));
    // +1/+2 from Holy Strength, +2/+0 from Bonesplitter.
    assert_eq!(pt(&game, bears), (5, 4));
    assert_eq!(attaches_of(&game, splitter), vec![(bears, None)]);
}

/// "Target creature you control" — an opponent's creature is not a legal
/// target, so with only one on the board the activation has nowhere to go,
/// and with one of each the first legal pick is yours even though the
/// opponent's entered first.
// COVERS: ATOM-702.6a-002
#[test]
fn test_equip_targets_only_creatures_you_control() {
    let mut game = setup_two_player_game();
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let splitter = put_on_battlefield(&mut game, bonesplitter(), 0);

    assert!(equip(&mut game, 0, splitter, 0).is_err(), "no legal target");
    assert_eq!(game.battlefield[&splitter].attached_to, None);
    assert!(game.stack.is_empty(), "the activation rolled back");

    let mine = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    equip(&mut game, 0, splitter, 0).expect("your creature is a legal target");
    assert_eq!(game.battlefield[&splitter].attached_to, Some(mine));
    assert!(!game.battlefield[&theirs].attached_by.contains(&splitter));
}

/// A second equip moves the Equipment: the old host's back-pointer is gone,
/// the bonus went with it, and the event names where it came from.
// COVERS: ATOM-301.5c-005
#[test]
fn test_re_equipping_moves_the_equipment_and_cleans_the_old_host() {
    let mut game = setup_two_player_game();
    let first = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let second = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    let splitter = put_on_battlefield(&mut game, bonesplitter(), 0);

    equip(&mut game, 0, splitter, 0).unwrap();
    assert_eq!(pt(&game, first), (4, 2));

    equip(&mut game, 0, splitter, 1).unwrap();
    assert_eq!(game.battlefield[&splitter].attached_to, Some(second));
    assert!(game.battlefield[&second].attached_by.contains(&splitter));
    assert!(!game.battlefield[&first].attached_by.contains(&splitter));
    assert_eq!(pt(&game, first), (2, 2));
    assert_eq!(pt(&game, second), (5, 3));
    assert_eq!(attaches_of(&game, splitter), vec![(first, None), (second, Some(first))]);
}

/// CR 701.3b — attaching to the object it is already attached to does
/// nothing: no transition, no event, and no new timestamp (CR 701.3c says
/// "a different object").
#[test]
fn test_equipping_the_host_it_is_already_on_does_nothing() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let splitter = put_on_battlefield(&mut game, bonesplitter(), 0);

    equip(&mut game, 0, splitter, 0).unwrap();
    let stamped = game.battlefield[&splitter].timestamp;
    equip(&mut game, 0, splitter, 0).unwrap();

    assert_eq!(game.battlefield[&splitter].attached_to, Some(bears));
    assert_eq!(game.battlefield[&bears].attached_by, vec![splitter]);
    assert_eq!(attaches_of(&game, splitter).len(), 1, "the second activation announced nothing");
    assert_eq!(game.battlefield[&splitter].timestamp, stamped);
}

/// CR 613.7e, pinned where Layer 7c cannot see it. The Equipment enters
/// first (T1), Humility second (T2). By registration order Humility's "lose
/// all abilities" is the later Layer 6 effect and the grant is gone; the
/// timestamp the equip gives the Equipment (T3 > T2) is the only thing that
/// puts its grant after Humility — and, since the ordered sweeps key on the
/// same timestamp, Cobbled Wings now sorts last among the three.
///
/// Partial: the atom moves an Equipment away and back against an Aura that
/// grants flying; this is one reattachment against Humility, which the pool
/// registers.
// COVERS-PARTIAL: ATOM-613.7e-001
#[test]
fn test_a_reattached_equipment_gets_a_timestamp_later_than_humility() {
    let mut game = setup_two_player_game();
    let wings = put_on_battlefield(&mut game, cobbled_wings(), 0);
    let humility_id = put_on_battlefield(&mut game, humility(), 1);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert!(!has_keyword(&game, bears, KeywordFlag::Flying));
    assert_eq!(game.battlefield_ids_ordered(), vec![wings, humility_id, bears]);

    equip(&mut game, 0, wings, 0).unwrap();

    assert!(
        game.battlefield[&wings].timestamp > game.battlefield[&humility_id].timestamp,
        "CR 613.7e: the attach gave the Equipment a new timestamp"
    );
    assert!(
        has_keyword(&game, bears, KeywordFlag::Flying),
        "the grant now applies after Humility's Layer 6 strip"
    );
    assert_eq!(
        game.battlefield_ids_ordered(),
        vec![humility_id, bears, wings],
        "the sweep order is CR 613.7's, so the equipped Wings sorts last"
    );
}

/// A rules fixture, not a card: no printed Equipment both strips and grants
/// (Scryfall, 2026-09-06 — Barrow-Blade is the only "loses all abilities"
/// Equipment, and that is a trigger). Two Layer 6 static abilities in printed
/// order, the strip first and the grant second, so their relative order *is*
/// the observable answer.
fn strip_then_grant_equipment() -> std::sync::Arc<mtgsim::objects::card_data::CardData> {
    use mtgsim::types::card_types::{ArtifactType, CardType, Subtype};
    use mtgsim::types::effects::{Duration, Primitive};
    mtgsim::objects::card_data::CardDataBuilder::new("Muzzle and Wings (fixture)")
        .card_type(CardType::Artifact)
        .subtype(Subtype::Artifact(ArtifactType::Equipment))
        .mana_cost(mtgsim::types::mana::ManaCost::build(&[], 1))
        .ability(mtgsim::test_support::static_ability(Effect::Atom(
            Primitive::LoseAllAbilities(Duration::WhileSourceOnBattlefield),
            EffectRecipient::Host,
        )))
        .ability(mtgsim::test_support::static_ability(Effect::Atom(
            Primitive::GrantKeywordFlag(KeywordFlag::Flying, Duration::WhileSourceOnBattlefield),
            EffectRecipient::Host,
        )))
        .build()
}

/// `(id, timestamp)` of every registry row `source` generated, in storage
/// order — which is application order within a layer.
fn rows_of(game: &GameState, source: ObjectId) -> Vec<(mtgsim::engine::layers::types::EffectId, u64)> {
    game.continuous_effects.iter().filter(|e| e.source == source).map(|e| (e.id, e.timestamp)).collect()
}

/// CR 613.7a, third sentence: "if the object the ability is on receives a new
/// timestamp, each continuous effect generated by static abilities of that
/// object receives a new timestamp as well, but the relative order of those
/// timestamps remains the same." Strip then grant on one Equipment: a
/// creature with printed flying loses everything and then has flying, and so
/// does one without. Each attach re-stamps both rows to one new timestamp
/// (CR 613.7e); a re-stamp that reversed them, gave them distinct fresh
/// timestamps in the wrong order, or replaced the rows with new ids would
/// leave the host flightless or break the id check.
///
/// No `COVERS`: the corpus states the third sentence in §613.7a's summary and
/// files no atom for it (ATOM-613.7a-001 is the later-of clause).
#[test]
fn test_a_re_stamp_keeps_the_relative_order_of_one_objects_effects() {
    let mut game = setup_two_player_game();
    let flyer = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Flying]), 0);
    let walker = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    let harness = put_on_battlefield(&mut game, strip_then_grant_equipment(), 0);
    let registered = rows_of(&game, harness);
    assert_eq!(registered.len(), 2, "two static abilities, two rows");
    assert_eq!(registered[0].1, registered[1].1, "CR 613.7a: one timestamp for both");
    let attach = |game: &mut GameState, host| {
        game.execute_action(GameAction::Attach { attachment: harness, host }, &test_ctx())
            .expect("both on the battlefield");
    };

    attach(&mut game, flyer);
    assert!(has_keyword(&game, flyer, KeywordFlag::Flying), "strip, then grant: the grant is the later row");
    let after_first = rows_of(&game, harness);
    assert_eq!(after_first.iter().map(|r| r.0).collect::<Vec<_>>(), registered.iter().map(|r| r.0).collect::<Vec<_>>(), "same rows, same order");
    assert!(after_first[0].1 > registered[0].1, "and a new timestamp");
    assert_eq!(after_first[0].1, after_first[1].1);
    assert_eq!(after_first[0].1, game.battlefield[&harness].timestamp);

    attach(&mut game, walker);
    assert!(has_keyword(&game, walker, KeywordFlag::Flying), "the same order on the second host");
    assert!(has_keyword(&game, flyer, KeywordFlag::Flying), "the first host is back to its printed flying");
    let after_second = rows_of(&game, harness);
    assert_eq!(after_second.iter().map(|r| r.0).collect::<Vec<_>>(), registered.iter().map(|r| r.0).collect::<Vec<_>>());
    assert!(after_second[0].1 > after_first[0].1);
    assert_eq!(after_second[0].1, after_second[1].1);
}

/// CR 301.5b — an Equipment spell resolves like any artifact and enters
/// unattached; there is no target to choose at CR 601.2c.
// COVERS: ATOM-301.5b-001
#[test]
fn test_an_equipment_spell_enters_unattached() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let splitter = put_in_hand(&mut game, bonesplitter(), 0);
    game.players[0].mana_pool.add(ManaType::White, 1);

    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
        vec![1],
    );
    game.cast_spell(0, splitter, &decisions).expect("castable");
    game.resolve_top_of_stack(&decisions).expect("resolves");

    assert!(game.battlefield.contains_key(&splitter));
    assert_eq!(game.battlefield[&splitter].attached_to, None);
    assert!(decisions.is_empty());
}

/// Registered, so `fuzz_games --pool stress` can draw it, and pooled, so the
/// new primitive, the new action and the first activation restriction are
/// measured rather than assumed (`engineering-practices.md` §3).
#[test]
fn test_bonesplitter_is_registered_and_in_the_performance_pool() {
    assert!(CardRegistry::default_registry().create("Bonesplitter").is_ok());
    assert!(CardRegistry::performance_pool().create("Bonesplitter").is_ok());
}

/// Registered so a `stress` game can put it beside Humility (CR 613.7e in
/// the wild), and deliberately not pooled: it opens no engine path
/// Bonesplitter does not, and the pool grows one card per path.
#[test]
fn test_cobbled_wings_is_registered_and_not_pooled() {
    assert!(CardRegistry::default_registry().create("Cobbled Wings").is_ok());
    assert!(CardRegistry::performance_pool().create("Cobbled Wings").is_err());
}

// ===========================================================================
// CR 704.5p — a creature is never attached to anything
// ===========================================================================

/// CR 704.5p, first sentence: "If a battle or creature is attached to an
/// object or player, it becomes unattached and remains on the battlefield."
///
/// March of the Machines makes Bonesplitter a creature while it is equipping
/// something. It must come off, and stop granting +2/+0 — which is the whole
/// of March's own ruling: "if an Equipment becomes a creature, it can no
/// longer equip a creature. If it's currently attached to a creature, it
/// becomes unattached (but remains on the battlefield)."
///
/// Both cards are in `PERFORMANCE_POOL`, so this board is reachable in any
/// measured game where they meet.
// COVERS: ATOM-704.5p-001
#[test]
fn test_an_equipment_that_becomes_a_creature_unattaches() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let splitter = put_on_battlefield(&mut game, bonesplitter(), 0);
    assert!(game.attach(splitter, bears));
    assert_eq!(pt(&game, bears), (4, 2), "equipped, before anything animates it");

    put_on_battlefield(&mut game, mtgsim::cards::phase_ld_cards::march_of_the_machines(), 0);
    game.check_state_based_actions_loop(&test_dp()).unwrap();

    assert!(
        mtgsim::oracle::characteristics::is_creature(&game, splitter),
        "March animated it, which is the premise",
    );
    assert_eq!(
        game.battlefield[&splitter].attached_to, None,
        "CR 704.5p: a creature is not attached to anything",
    );
    assert!(game.battlefield.contains_key(&splitter), "and remains on the battlefield");
    assert_eq!(pt(&game, bears), (2, 2), "so it stops granting +2/+0");
}

/// The other half of CR 704.5p's first sentence, and the composition that
/// closes a two-year-old TODO: an **Aura** that is also a creature.
///
/// It is unattached by the same predicate — one rule, not two — and then
/// CR 704.5m puts it into its owner's graveyard on the loop's next pass,
/// because by then it is an Aura attached to nothing. No registered card
/// animates an enchantment, so the row is placed by hand; the rule is not
/// hypothetical, only its carrier is.
// COVERS-PARTIAL: ATOM-704.5p-001
#[test]
fn test_an_aura_that_becomes_a_creature_unattaches_then_dies() {
    use mtgsim::engine::layers::types::{EffectModification, Layer};
    use mtgsim::test_support::registered;
    use mtgsim::types::card_types::CardType;

    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let aura = put_on_battlefield(&mut game, holy_strength(), 0);
    assert!(game.attach(aura, bears));
    assert_eq!(pt(&game, bears), (3, 4), "enchanted");

    let ts = game.allocate_timestamp();
    let row = registered(aura, Layer::Layer4Type, ts, EffectModification::AddType(CardType::Creature));
    game.continuous_effects.add(row);

    game.check_state_based_actions_loop(&test_dp()).unwrap();

    assert!(!game.battlefield.contains_key(&aura), "704.5p unattached it, then 704.5m took it");
    assert!(game.players[0].graveyard.contains(&aura));
    assert_eq!(pt(&game, bears), (2, 2), "and its +1/+2 went with it");
}
