//! Layer 2 — control-changing effects (CR 613.1b).
//!
//! Layer 2 is the second-earliest layer, so almost everything about it is a
//! statement about what happens *after* it: a filter that says "you control"
//! reads the post-Layer-2 controller (CR 109.5), and so does anything the
//! engine asks about a permanent — who untaps it, who may attack with it, whose
//! mana it makes.
//!
//! Two things make it unlike the other layers implemented so far:
//!
//! - **It is the only layer whose result carries a time.** CR 302.6 asks
//!   whether control has been continuous since the controller's most recent
//!   turn began, so `EffectiveCharacteristics` carries `control_since_turn`
//!   beside `controller`. It has to be computed rather than stored — a
//!   `Duration::UntilEndOfTurn` steal reverts at cleanup with no mutation and
//!   no event.
//! - **It reaches the stack.** CR 108.4 gives a *spell* a controller, and
//!   gaining control of a permanent spell decides who controls the permanent
//!   (CR 110.2b).

use mtgsim::cards::{phase5_pre_cards, phase_le_cards, phase_lg_cards};
use mtgsim::engine::layers::types::{
    AffectedSet, ContinuousEffect, EffectModification, EffectOrigin, Layer,
};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::{
    get_effective_controller, get_effective_power, has_keyword, has_summoning_sickness,
};
use mtgsim::oracle::legality::legal_attackers;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StackEntry, StepType};
use mtgsim::test_support::{
    attach, card_of_type, equipment, pacifism, put_in_graveyard, put_on_battlefield,
    setup_two_player_game, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    Duration, Effect, EffectRecipient, PermanentFilter, PlayerRef, Primitive, SelectionFilter,
    TargetCount,
};
use mtgsim::types::ids::{new_object_id, ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::zones::Zone;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve Act of Treason's printed effect with `thief` as its controller and
/// `victim` as its target.
///
/// The card's own `AbilityDef`, not a hand-built row: the point of most of
/// these tests is that the *card* works, and the three atoms sharing one
/// `ResolutionContext` is what makes "that creature" mean the same creature.
fn act_of_treason(game: &mut GameState, thief: PlayerId, victim: ObjectId) {
    let card = phase_lg_cards::act_of_treason();
    let source = GameObject::new(card.clone(), thief, Zone::Stack);
    let source_id = source.id;
    game.add_object(source);

    let ctx = ResolutionContext {
        source: source_id,
        controller: thief,
        targets: vec![ResolvedTarget::Object(victim)],
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp())
        .expect("Act of Treason resolves");
}

/// "Gain control of target permanent" with no untap and no haste, so a test can
/// watch CR 302.6 without Act of Treason's haste clause covering for it.
///
/// Not a card — a bare `Primitive::GainControl` resolved directly, which is
/// exactly what the shared lowering produces for the control clause of any card
/// that has one.
fn gain_control(
    game: &mut GameState,
    thief: PlayerId,
    target: ObjectId,
    duration: Duration,
) -> ObjectId {
    let source = GameObject::new(vanilla_creature(1, 1, &[]), thief, Zone::Stack);
    let source_id = source.id;
    game.add_object(source);

    let effect = Effect::Atom(
        Primitive::GainControl(duration),
        EffectRecipient::Target(
            SelectionFilter::Permanent(PermanentFilter::All),
            TargetCount::Exactly(1),
        ),
    );
    let ctx = ResolutionContext {
        source: source_id,
        controller: thief,
        targets: vec![ResolvedTarget::Object(target)],
    };
    game.resolve_effect(&effect, &ctx, &test_dp())
        .expect("GainControl resolves");
    source_id
}

/// A minimal `StackEntry` for a permanent spell with no targets and no effect.
fn stack_entry(spell_id: ObjectId, controller: PlayerId) -> StackEntry {
    StackEntry {
        object_id: spell_id,
        controller,
        chosen_targets: Vec::new(),
        chosen_modes: Vec::new(),
        x_value: None,
        effect: Effect::Sequence(Vec::new()),
        is_spell: true,
        chosen_alternative_cost: None,
        additional_costs_paid: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// CR 613.1b — Layer 2 runs before the layers that read its answer
// ---------------------------------------------------------------------------

/// The ordering claim, and the only test here where getting the layers
/// backwards produces a different number rather than a different owner.
///
/// Glorious Anthem is P0's, and its filter is "creatures you control" — a
/// `ByController(You)` node that `compute::permanent_matches_filter` evaluates
/// against `chars.controller`. That field is written in Layer 2 and read in
/// Layer 7c, so stealing P1's Bear makes the Anthem find it.
///
/// The corpus named a card called "Mind Snare" here. No such card exists —
/// Scryfall 404s on both a fuzzy lookup and an exact search — so the atom was
/// unbuildable as written and the session file now names Act of Treason. Its
/// haste clause is inert for a P/T query.
// COVERS: ATOM-613.1b-001
#[test]
fn test_stolen_creature_is_pumped_by_the_thiefs_anthem() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    assert_eq!(
        get_effective_power(&game, bear),
        Some(2),
        "P1's Bear is outside P0's anthem before the theft"
    );

    act_of_treason(&mut game, 0, bear);

    assert_eq!(get_effective_controller(&game, bear), Some(0));
    assert_eq!(
        get_effective_power(&game, bear),
        Some(3),
        "CR 613.1b: Layer 2 moves control, and Layer 7c's \"creatures you \
         control\" filter reads the moved value"
    );
}

/// The other side of the same ordering: the anthem the creature *left*.
///
/// This is the half a "controller is whatever the battlefield entry says"
/// implementation still passes by accident, so it is worth pinning separately —
/// P1's own anthem has to stop seeing the creature the moment P0 takes it.
#[test]
fn test_stolen_creature_leaves_its_owners_anthem() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 1);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    assert_eq!(get_effective_power(&game, bear), Some(3));

    act_of_treason(&mut game, 0, bear);

    assert_eq!(
        get_effective_power(&game, bear),
        Some(2),
        "CR 109.5: the anthem's \"you\" is P1, and the Bear is no longer theirs"
    );
}

/// CR 613.6 — one effect, parts in different layers, one affected set.
///
/// Act of Treason is the CR's own example. Control is a Layer 2 row and haste
/// is a Layer 6 row; both are created by one resolution against one
/// `ResolutionContext`, so "that creature" is the same creature in both.
// COVERS: ATOM-613.6-002
#[test]
fn test_act_of_treason_applies_control_in_layer_2_and_haste_in_layer_6() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    game.battlefield.get_mut(&creature).unwrap().tapped = true;

    act_of_treason(&mut game, 0, creature);

    assert_eq!(
        get_effective_controller(&game, creature),
        Some(0),
        "Layer 2: P0 controls it"
    );
    assert!(
        has_keyword(&game, creature, KeywordFlag::Haste),
        "Layer 6: it has haste"
    );
    assert!(
        !game.battlefield.get(&creature).unwrap().tapped,
        "the untap clause is a one-shot action, not a layer"
    );

    let layer2 = game.continuous_effects.effects_in_layer(Layer::Layer2Control);
    let layer6 = game.continuous_effects.effects_in_layer(Layer::Layer6Ability);
    assert_eq!(layer2.len(), 1);
    assert_eq!(layer6.len(), 1);
    assert_eq!(layer2[0].source, layer6[0].source, "one spell, so one source");
}

// ---------------------------------------------------------------------------
// CR 110.2b — gaining control of a permanent SPELL
// ---------------------------------------------------------------------------

/// CR 108.4's first half, with owner and controller deliberately different.
///
/// The frame seeds a spell's controller from its `StackEntry`, and this is the
/// only test that can tell: everywhere else the caster owns the card, so
/// falling back to `GameObject.owner` gives the same answer and the seed is
/// untested. A player casting a card they do not own is real — CR 108.4 is
/// worded for it, and every "cast target card from an opponent's graveyard or
/// library" effect produces it.
#[test]
fn test_a_spell_reports_its_caster_not_its_owner() {
    let mut game = setup_two_player_game();

    // P0 owns the card; P1 is casting it.
    let spell = GameObject::new(vanilla_creature(2, 2, &[]), 0, Zone::Stack);
    let spell_id = spell.id;
    game.add_object(spell);
    game.stack.push(spell_id);
    game.stack_entries.insert(spell_id, stack_entry(spell_id, 1));

    assert_eq!(
        get_effective_controller(&game, spell_id),
        Some(1),
        "CR 108.4: a spell's controller is who cast it, not who owns the card"
    );

    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(
        get_effective_controller(&game, spell_id),
        Some(1),
        "CR 110.2: and the permanent enters under that player's control"
    );
    assert_eq!(game.objects.get(&spell_id).unwrap().owner, 0);
}

/// CR 108.4 gives a spell a controller, so Layer 2 can move it, and CR 110.2b
/// makes the resulting permanent enter under whoever controlled the spell as it
/// resolved.
///
/// Three pieces had to line up. `compute::base_controller` seeds the frame from
/// `StackEntry` as well as `BattlefieldEntity`; `collect_controllable_targets`
/// lets a `GainControl` name a target that is on the stack rather than the
/// battlefield; and `resolve_top_of_stack` reads the controller *before* it
/// pops, because the pop destroys the `StackEntry` the seed comes from.
// COVERS: ATOM-110.2b-001
#[test]
fn test_gaining_control_of_a_permanent_spell_moves_the_permanent() {
    let mut game = setup_two_player_game();

    // P0 casts a creature: object on the stack with a StackEntry naming P0.
    let spell = GameObject::new(vanilla_creature(2, 2, &[]), 0, Zone::Stack);
    let spell_id = spell.id;
    game.add_object(spell);
    game.stack.push(spell_id);
    game.stack_entries.insert(spell_id, stack_entry(spell_id, 0));

    assert_eq!(
        get_effective_controller(&game, spell_id),
        Some(0),
        "CR 108.4: the spell's controller before anyone interferes"
    );

    // P1 gains control of the spell. Indefinite, not UEOT: the effect lasts as
    // long as there is anything to control, and the permanent keeps the
    // controller it entered under.
    gain_control(&mut game, 1, spell_id, Duration::Indefinite);
    assert_eq!(get_effective_controller(&game, spell_id), Some(1));

    game.resolve_top_of_stack(&test_dp())
        .expect("the creature spell resolves");

    assert!(
        game.battlefield.contains_key(&spell_id),
        "it is a permanent now"
    );
    // The *base* controller, not the effective one, and the difference is the
    // whole rule. `Duration::Indefinite` leaves the Layer 2 row in the registry
    // after the spell becomes a permanent, so `get_effective_controller` would
    // answer 1 even if the permanent had entered under P0's control and the row
    // were merely still applying on top. CR 110.2b is a claim about what
    // `init_zone_state_with_controller` was handed.
    assert_eq!(
        game.battlefield.get(&spell_id).unwrap().controller,
        1,
        "CR 110.2b: it *enters* under the control of whoever controlled the spell"
    );
    assert_eq!(get_effective_controller(&game, spell_id), Some(1));
    assert_eq!(
        game.objects.get(&spell_id).unwrap().owner,
        0,
        "CR 108.3: ownership does not move"
    );
}

// ---------------------------------------------------------------------------
// CR 302.6 — summoning sickness follows control
// ---------------------------------------------------------------------------

/// The rule Act of Treason's haste clause is paying for.
///
/// `gain_control` deliberately has no haste, so this is the naked CR 302.6
/// answer: a creature that changed hands this turn has not been under its new
/// controller's control continuously since their most recent turn began.
#[test]
fn test_gaining_control_gives_the_creature_summoning_sickness() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    assert!(
        !has_summoning_sickness(&game, creature),
        "it has been P1's since before the turn"
    );

    gain_control(&mut game, 0, creature, Duration::UntilEndOfTurn);

    assert!(
        has_summoning_sickness(&game, creature),
        "CR 302.6: P0 gained control this turn"
    );
}

/// And why the Layer 2 arm compares before it assigns.
///
/// "Target creature" carries no controller restriction, so aiming a
/// control-gaining spell at your own creature is legal. Control never changes,
/// so the CR 302.6 clock never restarts. An implementation that writes
/// `control_since_turn` unconditionally passes every other test in this file
/// and fails this one.
#[test]
fn test_gaining_control_of_your_own_creature_does_not_make_it_sick() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    gain_control(&mut game, 0, creature, Duration::UntilEndOfTurn);

    assert_eq!(get_effective_controller(&game, creature), Some(0));
    assert!(
        !has_summoning_sickness(&game, creature),
        "CR 302.6 asks whether control was *continuous*, and it was"
    );
}

/// Act of Treason buys its way out of the rule above, which is the whole point
/// of the card.
#[test]
fn test_act_of_treason_beats_summoning_sickness_with_haste() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    act_of_treason(&mut game, 0, creature);

    assert!(
        !has_summoning_sickness(&game, creature),
        "sick under CR 302.6, but the Layer 6 haste clause answers it"
    );
    assert_eq!(
        legal_attackers(&game, 0),
        vec![creature],
        "so P0 can attack with it, which is what the card is for"
    );
    assert!(legal_attackers(&game, 1).is_empty(), "and P1 cannot");
}

// ---------------------------------------------------------------------------
// The engine reads the moved controller everywhere it matters
// ---------------------------------------------------------------------------

/// CR 502.1 — "the active player untaps all permanents they control". The
/// stolen creature untaps on the thief's turn, not its owner's.
#[test]
fn test_untap_step_untaps_the_permanents_you_effectively_control() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let own = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    gain_control(&mut game, 0, creature, Duration::Indefinite);
    for id in [creature, own] {
        game.battlefield.get_mut(&id).unwrap().tapped = true;
    }

    // Walk into P0's untap step. P1 is active and in their ending phase, so the
    // next turn transition makes P0 active and `on_step_begin(Untap)` runs the
    // sweep as part of `advance_turn`.
    game.active_player = 1;
    game.phase = Phase::new(PhaseType::Ending);
    while game.phase.step != Some(StepType::Untap) {
        game.advance_turn().unwrap();
    }
    assert_eq!(game.active_player, 0);

    assert!(
        !game.battlefield.get(&creature).unwrap().tapped,
        "P0 controls it, so P0's untap step untaps it"
    );
    assert!(
        game.battlefield.get(&own).unwrap().tapped,
        "P1 still controls this one"
    );
}

/// CR 514.2 — an "until end of turn" effect ends in the cleanup step, and
/// control goes back with nothing having to notice.
#[test]
fn test_control_reverts_when_the_effect_expires() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    act_of_treason(&mut game, 0, creature);
    assert_eq!(get_effective_controller(&game, creature), Some(0));

    game.continuous_effects
        .remove_expired_at_cleanup(game.active_player, game.turn_number);

    assert_eq!(
        get_effective_controller(&game, creature),
        Some(1),
        "the row is gone, so the frame seeds from BattlefieldEntity again"
    );
    assert!(
        !game.continuous_effects.summary().any_control_changing,
        "and the gate goes back off, so control-free boards cost what they did"
    );
}

/// `RegistryScopeSummary::any_control_changing` is the gate that makes 20
/// migrated call sites free on boards with no control-changing effect. It has
/// never been true before this phase; pin that it is now.
#[test]
fn test_a_registered_control_effect_turns_the_gate_on() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    assert!(!game.continuous_effects.summary().any_control_changing);

    act_of_treason(&mut game, 0, creature);

    assert!(
        game.continuous_effects.summary().any_control_changing,
        "the gate has to come on, or the walk short-circuits to the pre-Layer-2 \
         value and every control change is invisible"
    );
}

// ---------------------------------------------------------------------------
// CR 301.5d / 303.4e — an attachment's controller is its own
// ---------------------------------------------------------------------------

/// CR 301.5d: "The controller of an Equipment is not necessarily the controller
/// of the creature it's attached to." Stealing the creature does not steal the
/// Equipment, and does not detach it.
// COVERS: ATOM-301.5d-001
#[test]
fn test_stealing_the_equipped_creature_leaves_the_equipment_behind() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let sword = put_on_battlefield(&mut game, equipment("Test Sword"), 0);
    attach(&mut game, sword, creature);

    gain_control(&mut game, 1, creature, Duration::Indefinite);

    assert_eq!(get_effective_controller(&game, creature), Some(1));
    assert_eq!(
        get_effective_controller(&game, sword),
        Some(0),
        "CR 301.5d: the Equipment's controller is independent"
    );
    assert_eq!(
        game.battlefield.get(&sword).unwrap().attached_to,
        Some(creature),
        "and it stays attached"
    );
}

/// CR 303.4e, the same sentence for Auras. Pacifism carries `enchant_filter =
/// Creature`, which the stolen creature still matches, so SBA 704.5n leaves it
/// alone — the "you control" version is a separate test.
// COVERS: ATOM-303.4e-001
#[test]
fn test_stealing_the_enchanted_creature_leaves_the_aura_behind() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let aura = put_on_battlefield(&mut game, pacifism(), 0);
    attach(&mut game, aura, creature);

    gain_control(&mut game, 1, creature, Duration::Indefinite);
    game.check_state_based_actions(&test_dp()).unwrap();

    assert_eq!(get_effective_controller(&game, creature), Some(1));
    assert_eq!(
        get_effective_controller(&game, aura),
        Some(0),
        "CR 303.4e: the Aura's controller is independent"
    );
    assert_eq!(
        game.battlefield.get(&aura).unwrap().attached_to,
        Some(creature),
        "\"enchant creature\" is still satisfied, so 704.5n does not fire"
    );
}

/// Both at once, on one creature. Composed rather than assumed: the two
/// attachments live in the same `attached_by` vector, and a control change that
/// walked that vector would take both with it.
// COVERS: COMP-301.5d+303.4e-001
#[test]
fn test_one_control_change_moves_neither_the_equipment_nor_the_aura() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let sword = put_on_battlefield(&mut game, equipment("Test Sword"), 0);
    let aura = put_on_battlefield(&mut game, pacifism(), 0);
    attach(&mut game, sword, creature);
    attach(&mut game, aura, creature);

    gain_control(&mut game, 1, creature, Duration::Indefinite);
    game.check_state_based_actions(&test_dp()).unwrap();

    assert_eq!(get_effective_controller(&game, creature), Some(1));
    assert_eq!(get_effective_controller(&game, sword), Some(0));
    assert_eq!(get_effective_controller(&game, aura), Some(0));

    let host = game.battlefield.get(&creature).unwrap();
    assert!(host.attached_by.contains(&sword) && host.attached_by.contains(&aura));
}

// ---------------------------------------------------------------------------
// The row itself
// ---------------------------------------------------------------------------

/// `SetController` carries a `PlayerRef`, and for a resolution row CR 611.2c
/// fixes what it means when the effect begins. Moving the *source* afterwards
/// must not move the effect's allegiance.
#[test]
fn test_a_resolution_control_effect_does_not_follow_its_source() {
    let mut game = setup_two_player_game();
    let victim = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let source = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);

    game.continuous_effects.add(ContinuousEffect {
        id: 0,
        source,
        origin: EffectOrigin::Resolution,
        layer: Layer::Layer2Control,
        duration: Duration::Indefinite,
        controller: 0,
        created_on_turn: 1,
        timestamp: 100,
        affected: AffectedSet::Fixed(vec![victim]),
        modification: EffectModification::SetController(PlayerRef::You),
    });
    assert_eq!(get_effective_controller(&game, victim), Some(0));

    // Someone takes the source. The effect keeps pointing at P0.
    game.continuous_effects.add(ContinuousEffect {
        id: 0,
        source: victim,
        origin: EffectOrigin::Resolution,
        layer: Layer::Layer2Control,
        duration: Duration::Indefinite,
        controller: 1,
        created_on_turn: 1,
        timestamp: 200,
        affected: AffectedSet::Fixed(vec![source]),
        modification: EffectModification::SetController(PlayerRef::You),
    });

    assert_eq!(get_effective_controller(&game, source), Some(1));
    assert_eq!(
        get_effective_controller(&game, victim),
        Some(0),
        "CR 611.2c: a resolution effect's \"you\" was fixed when it began"
    );
}

/// Two control effects on one permanent: CR 613.7 orders them by timestamp
/// within the layer, and the later one wins.
#[test]
fn test_the_later_control_effect_wins() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    gain_control(&mut game, 1, creature, Duration::Indefinite);
    assert_eq!(get_effective_controller(&game, creature), Some(1));

    gain_control(&mut game, 0, creature, Duration::Indefinite);
    assert_eq!(
        get_effective_controller(&game, creature),
        Some(0),
        "CR 613.7: later timestamp, applied later, wins"
    );
}

/// A permanent nobody has stolen still answers, and off the battlefield the
/// answer is the owner (CR 108.4 gives a card in a hand no controller at all,
/// and owner is the only defensible value for a non-`Option` field).
///
/// Also pins that the gated and ungated paths agree, which is the whole basis
/// for `any_control_changing` being exact rather than a heuristic.
#[test]
fn test_effective_controller_falls_back_the_way_the_frame_seeds() {
    let mut game = setup_two_player_game();
    let onboard = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let in_hand = GameObject::new(vanilla_creature(2, 2, &[]), 1, Zone::Hand);
    let in_hand_id = in_hand.id;
    game.add_object(in_hand);

    assert_eq!(get_effective_controller(&game, onboard), Some(1));
    assert_eq!(get_effective_controller(&game, in_hand_id), Some(1));
    assert_eq!(
        get_effective_controller(&game, new_object_id()),
        None,
        "no such object"
    );

    // Registering an unrelated control effect flips `any_control_changing`,
    // which is the only difference between the two paths.
    let other = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    gain_control(&mut game, 1, other, Duration::Indefinite);
    assert!(game.continuous_effects.summary().any_control_changing);
    assert_eq!(get_effective_controller(&game, onboard), Some(1));
    assert_eq!(get_effective_controller(&game, in_hand_id), Some(1));
}

/// A CDA is not a registry row (CR 604.3a(3)) and CR 613.4a admits none in
/// Layer 2, so the CDA pass never produces a `SetController`. This pins that a
/// board carrying both a CDA and a control effect computes without tripping
/// `apply_modification`'s `origin: None` assertion.
#[test]
fn test_a_cda_and_a_control_effect_coexist() {
    let mut game = setup_two_player_game();
    let goyf = put_on_battlefield(&mut game, phase_le_cards::tarmogoyf(), 1);
    put_in_graveyard(
        &mut game,
        card_of_type("Ancestral Recall", CardType::Instant),
        0,
    );

    gain_control(&mut game, 0, goyf, Duration::Indefinite);

    assert_eq!(get_effective_controller(&game, goyf), Some(0));
    assert_eq!(
        get_effective_power(&game, goyf),
        Some(1),
        "the CDA still runs at Layer 7a: one card type in graveyards"
    );
}
