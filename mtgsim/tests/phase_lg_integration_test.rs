//! Layer 2 — control-changing effects (CR 613.1b).
//!
//! Two things make it unlike the layers implemented so far:
//!
//! - **Its result carries a time.** CR 302.6 asks whether control has been
//!   continuous, so the frame carries `control_since_turn` beside `controller`.
//! - **It reaches the stack.** CR 108.4 gives a *spell* a controller, and
//!   gaining control of a permanent spell decides who controls the permanent
//!   (CR 110.2b).

use mtgsim::cards::{phase5_pre_cards, phase_le_cards, phase_lf_cards, phase_lg_cards};
use mtgsim::engine::layers::types::{
    AffectedSet, ContinuousEffect, EffectModification, EffectOrigin, Layer,
};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_controller, get_effective_power, has_keyword,
    has_summoning_sickness,
};
use mtgsim::oracle::legality::legal_attackers;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StackEntry, StepType};
use mtgsim::test_support::test_ctx;
use mtgsim::test_support::{
    attach, aura_enchanting_your_creature, card_of_type, equipment, fill_library, pacifism,
    pass_turn, put_in_graveyard, put_on_battlefield, put_on_battlefield_this_turn, setup_game,
    setup_two_player_game, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, PermanentFilter, PlayerRef, Primitive,
    SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{new_object_id, ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::zones::Zone;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve Act of Treason's printed effect — the card's own `AbilityDef`, not a
/// hand-built row, so the three atoms share one `ResolutionContext`.
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

/// A bare `Primitive::GainControl` — no untap and no haste — so a test can watch
/// CR 302.6 without Act of Treason's haste clause covering for it.
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

/// A minimal `StackEntry` for a spell with no targets.
fn stack_entry(spell_id: ObjectId, controller: PlayerId, effect: Effect) -> StackEntry {
    StackEntry {
        object_id: spell_id,
        controller,
        chosen_targets: Vec::new(),
        chosen_modes: Vec::new(),
        x_value: None,
        effect,
        is_spell: true,
        chosen_alternative_cost: None,
        additional_costs_paid: Vec::new(),
        cast_from: Some(Zone::Hand),
    }
}

// ---------------------------------------------------------------------------
// CR 613.1b — Layer 2 runs before the layers that read its answer
// ---------------------------------------------------------------------------

/// The ordering claim: `chars.controller` is written in Layer 2 and read by
/// Glorious Anthem's "creatures you control" filter in Layer 7c, so stealing
/// P1's Bear makes P0's anthem find it.
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

/// The other side of the same ordering, which a "controller is whatever the
/// battlefield entry says" implementation still passes by accident: P1's own
/// anthem has to stop seeing the creature the moment P0 takes it.
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

/// CR 613.6 — one effect, parts in different layers, one affected set. Control
/// is a Layer 2 row and haste a Layer 6 row, both from one resolution, so "that
/// creature" is the same creature in both.
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
    game.stack_entries.insert(spell_id, stack_entry(spell_id, 1, Effect::Sequence(Vec::new())));

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

/// CR 110.2b — the permanent enters under whoever controlled the spell as it
/// resolved.
// COVERS: ATOM-110.2b-001
#[test]
fn test_gaining_control_of_a_permanent_spell_moves_the_permanent() {
    let mut game = setup_two_player_game();

    // P0 casts a creature: object on the stack with a StackEntry naming P0.
    let spell = GameObject::new(vanilla_creature(2, 2, &[]), 0, Zone::Stack);
    let spell_id = spell.id;
    game.add_object(spell);
    game.stack.push(spell_id);
    game.stack_entries.insert(spell_id, stack_entry(spell_id, 0, Effect::Sequence(Vec::new())));

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

/// Gaining control of an instant does **not** redirect it to the thief's
/// graveyard. Two rules pull opposite ways and both hold: the spell's controller
/// resolves it, so the thief draws, but CR 608.2m sends the finished spell to
/// its *owner's* graveyard and CR 108.3 never moves ownership.
///
/// Worth pinning because this phase edited this function — the controller is now
/// read ahead of the pop, and the graveyard line sits four lines away reading
/// `owner`. No permanent-spell test can catch a conflation, since a permanent
/// never goes to a graveyard on resolution.
#[test]
fn test_gaining_control_of_an_instant_does_not_move_its_graveyard() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 1, 3);

    // P0 owns and casts a "draw a card" instant.
    let spell = GameObject::new(card_of_type("Test Instant", CardType::Instant), 0, Zone::Stack);
    let spell_id = spell.id;
    game.add_object(spell);
    game.stack.push(spell_id);
    game.stack_entries.insert(
        spell_id,
        stack_entry(
            spell_id,
            0,
            Effect::Atom(
                Primitive::DrawCards(AmountExpr::Fixed(1)),
                EffectRecipient::Controller,
            ),
        ),
    );

    gain_control(&mut game, 1, spell_id, Duration::Indefinite);
    assert_eq!(get_effective_controller(&game, spell_id), Some(1));

    let p1_hand_before = game.players[1].hand.len();
    game.resolve_top_of_stack(&test_dp()).expect("the instant resolves");

    assert_eq!(
        game.players[1].hand.len(),
        p1_hand_before + 1,
        "CR 609: the spell's controller resolves it, so P1 draws"
    );
    assert!(
        game.players[0].graveyard.contains(&spell_id),
        "CR 608.2m + 108.3: but it goes to its *owner's* graveyard"
    );
    assert!(!game.players[1].graveyard.contains(&spell_id));
    assert!(
        !game.battlefield.contains_key(&spell_id),
        "an instant is not a permanent"
    );
}

// ---------------------------------------------------------------------------
// CR 302.6 — summoning sickness follows control
// ---------------------------------------------------------------------------

/// The naked CR 302.6 answer, with no haste clause covering for it: a creature
/// that changed hands this turn has not been under its new controller's control
/// since their most recent turn began.
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

/// And why the Layer 2 arm compares before it assigns. "Target creature" has no
/// controller restriction, so this is legal, and control never changes — so the
/// CR 302.6 clock never restarts. An implementation that writes
/// `control_since_turn` unconditionally passes every other test here and fails
/// this one.
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
        game.advance_turn(&test_ctx()).unwrap();
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

/// CR 303.4c + 303.4e + Layer 2, composed: the Aura does *not* move with the
/// creature, and that is exactly why it dies. Its "you" is its own controller
/// (CR 109.5), who no longer controls what it enchants, so SBA 704.5n puts it
/// into its owner's graveyard.
///
/// Only one of the three moving parts is Layer 2 — `targeting`'s filter also
/// had to learn to resolve `PlayerRef::You` at all, and to read the *effective*
/// controller.
// COVERS: COMP-303.4c+303.4e+L11-001
#[test]
fn test_an_enchant_creature_you_control_aura_falls_off_when_the_creature_is_stolen() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let aura = put_on_battlefield(
        &mut game,
        aura_enchanting_your_creature("Test Bond"),
        0,
    );
    attach(&mut game, aura, creature);

    game.check_state_based_actions(&test_dp()).unwrap();
    assert!(
        game.battlefield.contains_key(&aura),
        "P0 controls the creature, so the restriction is satisfied"
    );

    gain_control(&mut game, 1, creature, Duration::Indefinite);
    game.check_state_based_actions(&test_dp()).unwrap();

    assert_eq!(
        get_effective_controller(&game, creature),
        Some(1),
        "the creature moved"
    );
    assert!(
        !game.battlefield.contains_key(&aura),
        "CR 704.5n: the Aura's controller no longer controls its host"
    );
    assert!(
        game.players[0].graveyard.contains(&aura),
        "CR 303.4c: to its owner's graveyard"
    );
    assert!(
        !game.battlefield.get(&creature).unwrap().attached_by.contains(&aura),
        "and the host's back-pointer is cleaned up"
    );
}

/// The control-independent half of the same rule, stated as a filter question
/// rather than an SBA one: an "Enchant creature you control" Aura controlled by
/// P1 may legally enchant P1's creatures and not P0's, and stealing a creature
/// moves it between those two sets.
#[test]
fn test_you_control_in_an_enchant_filter_reads_the_effective_controller() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let filter = SelectionFilter::Permanent(PermanentFilter::And(
        Box::new(PermanentFilter::ByType(CardType::Creature)),
        Box::new(PermanentFilter::ByController(PlayerRef::You)),
    ));

    assert!(
        game.validate_targets(
            &EffectRecipient::Target(filter.clone(), TargetCount::Exactly(1)),
            &[ResolvedTarget::Object(creature)],
            0,
        )
        .is_ok(),
        "P0 controls it"
    );
    assert!(
        game.validate_targets(
            &EffectRecipient::Target(filter.clone(), TargetCount::Exactly(1)),
            &[ResolvedTarget::Object(creature)],
            1,
        )
        .is_err(),
        "P1 does not"
    );

    gain_control(&mut game, 1, creature, Duration::Indefinite);

    assert!(
        game.validate_targets(
            &EffectRecipient::Target(filter.clone(), TargetCount::Exactly(1)),
            &[ResolvedTarget::Object(creature)],
            1,
        )
        .is_ok(),
        "CR 613.1b: and now P1 does"
    );
    assert!(
        game.validate_targets(
            &EffectRecipient::Target(filter, TargetCount::Exactly(1)),
            &[ResolvedTarget::Object(creature)],
            0,
        )
        .is_err(),
        "and P0 does not"
    );
}

// ---------------------------------------------------------------------------
// The row itself
// ---------------------------------------------------------------------------

/// CR 611.2c fixes a resolution effect's "you" when the effect begins, so moving
/// the *source* afterwards must not move the effect's allegiance.
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

/// CR 108.4 — only a permanent or a spell has a controller.
///
/// `get_effective_controller` still has to return something for a card in a
/// hand, because `EffectiveCharacteristics.controller` is not an `Option`, and
/// owner is the only defensible value. This pins that fallback chain:
/// battlefield entry, then stack entry, then owner, then `None` for an object
/// that does not exist.
#[test]
fn test_controller_off_the_battlefield_falls_back_to_the_owner() {
    let mut game = setup_two_player_game();
    let onboard = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let in_hand = GameObject::new(vanilla_creature(2, 2, &[]), 1, Zone::Hand);
    let in_hand_id = in_hand.id;
    game.add_object(in_hand);

    assert_eq!(get_effective_controller(&game, onboard), Some(1));
    assert_eq!(get_effective_controller(&game, in_hand_id), Some(1));
    assert_eq!(get_effective_controller(&game, new_object_id()), None);
}

/// `any_control_changing` skips the layer walk and reads the pre-Layer-2 seed
/// instead. That is only sound if the two paths give the same answer, so this
/// asks the same three objects twice — once with the flag off, once with it on
/// — and requires every answer to be unchanged.
///
/// The flag is registry-wide, so stealing one unrelated permanent turns it on
/// for the whole board. Nothing else about these three objects changes.
#[test]
fn test_the_gate_never_changes_an_answer() {
    let mut game = setup_two_player_game();
    let onboard = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let bystander = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let in_hand = GameObject::new(vanilla_creature(2, 2, &[]), 1, Zone::Hand);
    let in_hand_id = in_hand.id;
    game.add_object(in_hand);

    assert!(!game.continuous_effects.summary().any_control_changing);
    let gated = [
        get_effective_controller(&game, onboard),
        get_effective_controller(&game, bystander),
        get_effective_controller(&game, in_hand_id),
    ];

    // A third permanent changes hands. `onboard`, `bystander` and the card in
    // hand are untouched, but the flag is now on for all of them.
    let victim = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    gain_control(&mut game, 1, victim, Duration::Indefinite);
    assert!(game.continuous_effects.summary().any_control_changing);

    let walked = [
        get_effective_controller(&game, onboard),
        get_effective_controller(&game, bystander),
        get_effective_controller(&game, in_hand_id),
    ];
    assert_eq!(gated, walked, "the gate is exact, not an approximation");
    assert_eq!(get_effective_controller(&game, victim), Some(1));
}

/// No CDA lives in Layer 2, so the intrinsic pass never produces a
/// `SetController`. Pins that a board with both still computes without tripping
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

// ---------------------------------------------------------------------------
// CR 302.6 — the window is the *controller's* turn, not the game's turn
//
// The engine used to answer "has control been continuous since their most
// recent turn began?" with `control_since_turn >= game.turn_number`, which is
// "was control gained during the turn now being played". Those agree on your
// own turn and disagree on everyone else's: the creature you cast on your turn
// went unsick the moment the turn passed, one full turn early. Nothing in the
// pool could tap a creature until Citanul Hierophants handed one a mana
// ability, which is why it survived this long.
// ---------------------------------------------------------------------------

/// The window itself, at two players.
#[test]
fn test_a_creature_cast_on_your_turn_stays_sick_through_the_opponents_turn() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    fill_library(&mut game, 1, 5);

    let bears = put_on_battlefield_this_turn(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert!(has_summoning_sickness(&game, bears), "cast this turn");

    pass_turn(&mut game);
    assert_eq!((game.turn_number, game.active_player), (2, 1));
    assert!(
        has_summoning_sickness(&game, bears),
        "CR 302.6: P0's most recent turn is still turn 1, and the creature was not there when it began"
    );

    pass_turn(&mut game);
    assert_eq!((game.turn_number, game.active_player), (3, 0));
    assert!(
        !has_summoning_sickness(&game, bears),
        "P0's next turn has begun, so the clock is satisfied"
    );
}

/// The reachable consequence, end to end: Citanul Hierophants hands a creature
/// "{T}: Add {G}", and the CR forbids paying that tap cost on the opponent's
/// turn. Instant-speed mana activation is a real path — it is how a player
/// holds up Counterspell mana — so this is a wrong answer a game can reach.
#[test]
fn test_a_granted_tap_ability_stays_locked_on_the_opponents_turn() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    fill_library(&mut game, 1, 5);

    let bears = put_on_battlefield_this_turn(&mut game, vanilla_creature(2, 2, &[]), 0);
    put_on_battlefield(&mut game, phase_lf_cards::citanul_hierophants(), 0);

    let granted = get_effective_abilities(&game, bears);
    assert_eq!(granted.len(), 1, "the Hierophants' grant, and nothing else");
    let granted_id = granted[0].id;

    pass_turn(&mut game);
    let err = game
        .activate_mana_ability(0, bears, granted_id, &test_ctx())
        .expect_err("CR 302.6 forbids the tap on P1's turn");
    assert!(
        err.contains("summoning sickness"),
        "rejected for the wrong reason: {err}"
    );
    assert!(!game.battlefield[&bears].tapped);

    pass_turn(&mut game);
    game.activate_mana_ability(0, bears, granted_id, &test_ctx())
        .expect("P0's turn has begun, so the ability is live");
    assert!(game.battlefield[&bears].tapped);
}

/// The N-player form, which is the whole reason the fix tracks turn starts per
/// player instead of subtracting one from the turn number: at four players the
/// creature is sick through *three* opponents' turns.
#[test]
fn test_summoning_sickness_spans_every_opponents_turn_in_a_four_player_game() {
    let mut game = setup_game(4);
    for player in 0..4 {
        fill_library(&mut game, player, 8);
    }

    let bears = put_on_battlefield_this_turn(&mut game, vanilla_creature(2, 2, &[]), 0);

    for turn in 2..=4 {
        pass_turn(&mut game);
        assert_eq!(game.turn_number, turn);
        assert!(
            has_summoning_sickness(&game, bears),
            "still sick on turn {turn} — P0's most recent turn is turn 1"
        );
    }

    pass_turn(&mut game);
    assert_eq!((game.turn_number, game.active_player), (5, 0));
    assert!(!has_summoning_sickness(&game, bears));
}

/// The turn-0 sentinel (CR 103.6 openers), for a controller who has had no turn
/// at all: a Leyline on P1's side is not sick during P0's first turn. The
/// corrected comparison has to keep answering this, and "control since the
/// start of the game" is what makes it come out right.
#[test]
fn test_a_pregame_permanent_is_not_sick_before_its_controllers_first_turn() {
    let mut game = setup_two_player_game();
    let leyline = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    assert!(!has_summoning_sickness(&game, leyline));
}

/// And its opposite: a creature that entered *during* P0's turn 1 under P1's
/// control has not been P1's since any turn of theirs began, because P1 has not
/// had one.
#[test]
fn test_a_creature_entering_before_its_controllers_first_turn_is_sick() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 1, 5);
    let flashed = put_on_battlefield_this_turn(&mut game, vanilla_creature(2, 2, &[]), 1);
    assert!(
        has_summoning_sickness(&game, flashed),
        "P1 has had no turn yet"
    );

    pass_turn(&mut game);
    assert!(
        !has_summoning_sickness(&game, flashed),
        "P1's first turn has now begun"
    );
}
