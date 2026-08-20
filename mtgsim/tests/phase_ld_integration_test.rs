//! Phase LD Integration Tests: Layer 4 Type-Changing Effects
//!
//! Tests type-changing spells and static abilities end-to-end:
//! cast → resolve (registers ContinuousEffect at Layer4Type) →
//! layer system computes effective types/subtypes/supertypes →
//! cleanup step expires effects.
//!
//! Part A tests exercise basic type operations (AddType, AddSubtype, AddSupertype)
//! using cards that don't involve CR 305.7 (basic-land-type side effects).
//! Part B tests (Blood Moon / Urborg) exercise SetSubtypes on lands with a
//! filter-based static ability; full 305.7 semantics (ability stripping) are
//! deferred until AbilityOrigin infrastructure exists.

mod common;

use mtgsim::cards::basic_lands;
use mtgsim::cards::creatures;
use mtgsim::cards::phase_ld_cards;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::oracle::characteristics::{
    get_effective_power, get_effective_subtypes, get_effective_supertypes,
    get_effective_toughness, get_effective_types, is_creature,
};
use mtgsim::types::card_types::{CardType, CreatureType, LandType, Subtype, Supertype};
use mtgsim::types::effects::{EffectRecipient, PermanentFilter, SelectionFilter, TargetCount};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

use common::{fill_library, put_in_hand, put_on_battlefield, setup_two_player_game};

/// Helper: cast a spell targeting a permanent with a PermanentFilter.
/// If `generic_allocation` is Some, queues the allocation for generic mana.
fn cast_and_resolve_targeted_perm_spell(
    game: &mut mtgsim::state::game_state::GameState,
    decisions: &ScriptedDecisionProvider,
    spell_id: mtgsim::types::ids::ObjectId,
    cast_index: usize,
    target_index: usize,
    filter: PermanentFilter,
    generic_allocation: Option<Vec<u64>>,
) {
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![cast_index]);
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(
                SelectionFilter::Permanent(filter),
                TargetCount::Exactly(1),
            ),
            spell_id,
        },
        vec![target_index],
    );
    if let Some(alloc) = generic_allocation {
        decisions.expect_allocation(
            ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
            alloc,
        );
    }
    let result = game.run_priority_round(decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);

    // Both pass → resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(decisions).unwrap();
}

// ===========================================================================
// Part A Test 1: Liquimetal Torque — AddType (creature becomes also artifact)
// ===========================================================================

// COVERS: ATOM-205.1b-004
#[test]
fn test_liquimetal_adds_artifact_type() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let spell_id = put_in_hand(&mut game, phase_ld_cards::liquimetal_coating_spell(), 0);
    game.players[0].mana_pool.add(ManaType::Colorless, 2);

    // Verify base: Creature, not Artifact
    let types = get_effective_types(&game, bears_id);
    assert!(types.contains(&CardType::Creature));
    assert!(!types.contains(&CardType::Artifact));

    let decisions = ScriptedDecisionProvider::new();
    // {2}: pool has [Colorless(2)], allocate 1 generic → [1]
    cast_and_resolve_targeted_perm_spell(
        &mut game,
        &decisions,
        spell_id,
        1,
        0,
        PermanentFilter::All,
        Some(vec![1]),
    );

    // Creature should now also be an Artifact
    let types = get_effective_types(&game, bears_id);
    assert!(types.contains(&CardType::Creature));
    assert!(types.contains(&CardType::Artifact));

    // Spell is in graveyard
    assert_eq!(game.get_object(spell_id).unwrap().zone, Zone::Graveyard);
}

// ===========================================================================
// Part A Test 2: Call to Serve — AddSubtype (creature gains Angel subtype)
// ===========================================================================

#[test]
fn test_call_to_serve_adds_angel_subtype() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let spell_id = put_in_hand(&mut game, phase_ld_cards::call_to_serve_spell(), 0);
    game.players[0].mana_pool.add(ManaType::White, 2);

    // Base: no subtypes (Grizzly Bears has no subtype in card data)
    let subtypes = get_effective_subtypes(&game, bears_id);
    assert!(!subtypes.contains(&Subtype::Creature(CreatureType::Angel)));

    let decisions = ScriptedDecisionProvider::new();
    // {1}{W}: pool has [White(2)], allocate 1 generic → [1]
    cast_and_resolve_targeted_perm_spell(
        &mut game,
        &decisions,
        spell_id,
        1,
        0,
        PermanentFilter::All, // helper uses SelectionFilter::Creature via card, but DP sees All here
        Some(vec![1]),
    );

    // Should now have Angel subtype
    let subtypes = get_effective_subtypes(&game, bears_id);
    assert!(subtypes.contains(&Subtype::Creature(CreatureType::Angel)));

    // +1/+2 applied (L7c)
    assert_eq!(get_effective_power(&game, bears_id), Some(3));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(4));
}

// ===========================================================================
// Part A Test 3: On Serra's Wings — AddSupertype (Legendary)
// ===========================================================================

#[test]
fn test_on_serras_wings_adds_legendary() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let spell_id = put_in_hand(&mut game, phase_ld_cards::on_serras_wings_spell(), 0);
    game.players[0].mana_pool.add(ManaType::White, 4);

    // Base: no supertypes
    let supertypes = get_effective_supertypes(&game, bears_id);
    assert!(!supertypes.contains(&Supertype::Legendary));

    let decisions = ScriptedDecisionProvider::new();
    // {3}{W}: pool has [White(4)], allocate 3 generic → [3]
    cast_and_resolve_targeted_perm_spell(
        &mut game,
        &decisions,
        spell_id,
        1,
        0,
        PermanentFilter::All,
        Some(vec![3]),
    );

    // Now Legendary
    let supertypes = get_effective_supertypes(&game, bears_id);
    assert!(supertypes.contains(&Supertype::Legendary));

    // +1/+1 applied
    assert_eq!(get_effective_power(&game, bears_id), Some(3));
    assert_eq!(get_effective_toughness(&game, bears_id), Some(3));
}

// ===========================================================================
// Part A Test 4: Ensoul Artifact — AddType(Creature) + SetPT on artifact
// ===========================================================================

// COVERS: ATOM-205.1b-001
#[test]
fn test_ensoul_artifact_makes_artifact_creature() {
    use mtgsim::objects::card_data::CardDataBuilder;

    let mut game = setup_two_player_game();

    // Create a non-creature artifact
    let artifact_data = CardDataBuilder::new("Darksteel Ingot")
        .card_type(CardType::Artifact)
        .build();
    let artifact_id = put_on_battlefield(&mut game, artifact_data, 0);

    let spell_id = put_in_hand(&mut game, phase_ld_cards::ensoul_artifact_spell(), 0);
    game.players[0].mana_pool.add(ManaType::Blue, 2);

    // Verify: not a creature yet
    assert!(!is_creature(&game, artifact_id));

    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(
                SelectionFilter::Permanent(PermanentFilter::ByType(CardType::Artifact)),
                TargetCount::Exactly(1),
            ),
            spell_id,
        },
        vec![0],
    );
    // {1}{U}: pool has [Blue(2)], allocate 1 generic → [1]
    decisions.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: mtgsim::types::mana::ManaCost::zero() },
        vec![1],
    );
    let result = game.run_priority_round(&decisions).unwrap();
    assert_eq!(result, PriorityResult::ActionTaken);

    // Both pass → resolve
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(&decisions).unwrap();

    // Now it should be an artifact creature
    let types = get_effective_types(&game, artifact_id);
    assert!(types.contains(&CardType::Artifact));
    assert!(types.contains(&CardType::Creature));

    // P/T should be 5/5 (Layer 7b set)
    assert_eq!(get_effective_power(&game, artifact_id), Some(5));
    assert_eq!(get_effective_toughness(&game, artifact_id), Some(5));
}

// ===========================================================================
// Part A Test 5: Type change expires at cleanup
// ===========================================================================

// COVERS: ATOM-611.2a-001
#[test]
fn test_type_change_expires_at_cleanup() {
    let mut game = setup_two_player_game();
    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let spell_id = put_in_hand(&mut game, phase_ld_cards::liquimetal_coating_spell(), 0);
    fill_library(&mut game, 0, 5);
    fill_library(&mut game, 1, 5);
    game.players[0].mana_pool.add(ManaType::Colorless, 2);

    let decisions = ScriptedDecisionProvider::new();
    cast_and_resolve_targeted_perm_spell(
        &mut game,
        &decisions,
        spell_id,
        1,
        0,
        PermanentFilter::All,
        Some(vec![1]),
    );

    // Verify effect is active
    let types = get_effective_types(&game, bears_id);
    assert!(types.contains(&CardType::Artifact));

    // Advance to cleanup (9 steps from precombat main)
    for _ in 0..9 {
        game.advance_turn().unwrap();
    }

    // Effect should be gone — back to just Creature
    let types = get_effective_types(&game, bears_id);
    assert!(types.contains(&CardType::Creature));
    assert!(!types.contains(&CardType::Artifact));
    assert_eq!(game.continuous_effects.len(), 0);
}

// ===========================================================================
// Part B Test 1: Blood Moon static ability (SetSubtypes on nonbasic lands)
// ===========================================================================

// COVERS: ATOM-305.7-001, ATOM-305.7-004
#[test]
fn test_blood_moon_makes_nonbasic_lands_mountains() {
    use mtgsim::objects::card_data::CardDataBuilder;

    let mut game = setup_two_player_game();

    // Create a nonbasic land (no Basic supertype)
    let nonbasic_data = CardDataBuilder::new("Steam Vents")
        .card_type(CardType::Land)
        .subtype(Subtype::Land(LandType::Island))
        .subtype(Subtype::Land(LandType::Mountain))
        .build();
    let nonbasic_id = put_on_battlefield(&mut game, nonbasic_data, 0);

    // Create a basic land (has Basic supertype)
    let basic_id = put_on_battlefield(&mut game, basic_lands::forest(), 0);

    // Put Blood Moon on the battlefield (static ability registers on ETB)
    let blood_moon_id = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 0);

    // Nonbasic land should now have only Mountain subtype
    let subtypes = get_effective_subtypes(&game, nonbasic_id);
    assert!(subtypes.contains(&Subtype::Land(LandType::Mountain)));
    assert!(!subtypes.contains(&Subtype::Land(LandType::Island)));
    assert_eq!(subtypes.len(), 1);

    // Basic Forest should be unaffected
    let basic_subtypes = get_effective_subtypes(&game, basic_id);
    assert!(basic_subtypes.contains(&Subtype::Land(LandType::Forest)));
    assert!(!basic_subtypes.contains(&Subtype::Land(LandType::Mountain)));

    // Blood Moon should NOT affect non-land permanents
    assert!(game.battlefield.contains_key(&blood_moon_id));
}

// ===========================================================================
// Part B Test 2: Blood Moon does NOT add Basic supertype (ATOM-305.8-001)
// ===========================================================================

// COVERS: ATOM-305.8-001, COMP-305.7+305.8-001, COMP-205.4c+L10-001
#[test]
fn test_blood_moon_does_not_add_basic_supertype() {
    use mtgsim::objects::card_data::CardDataBuilder;

    let mut game = setup_two_player_game();

    // Create a nonbasic land
    let nonbasic_data = CardDataBuilder::new("Stomping Ground")
        .card_type(CardType::Land)
        .subtype(Subtype::Land(LandType::Mountain))
        .subtype(Subtype::Land(LandType::Forest))
        .build();
    let nonbasic_id = put_on_battlefield(&mut game, nonbasic_data, 0);

    // Put Blood Moon on the battlefield
    let _blood_moon_id = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 0);

    // Supertypes should NOT include Basic
    let supertypes = get_effective_supertypes(&game, nonbasic_id);
    assert!(!supertypes.contains(&Supertype::Basic));
}

// ===========================================================================
// Part B Test 3: Urborg-style "in addition to" adds subtype without removing
// ===========================================================================

// COVERS: ATOM-305.7-005
#[test]
fn test_urborg_adds_swamp_in_addition() {
    let mut game = setup_two_player_game();

    // Create a Mountain
    let mountain_id = put_on_battlefield(&mut game, basic_lands::mountain(), 0);

    // Put Urborg Effect on battlefield
    let _urborg_id = put_on_battlefield(&mut game, phase_ld_cards::urborg_effect(), 0);

    // Mountain should now also be a Swamp (in addition to Mountain)
    let subtypes = get_effective_subtypes(&game, mountain_id);
    assert!(subtypes.contains(&Subtype::Land(LandType::Mountain)));
    assert!(subtypes.contains(&Subtype::Land(LandType::Swamp)));
}

// ===========================================================================
// Part B Test 4: Blood Moon doesn't affect creatures
// ===========================================================================

#[test]
fn test_blood_moon_ignores_non_lands() {
    let mut game = setup_two_player_game();

    let bears_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let _blood_moon_id = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 0);

    // Bears should still be a creature, unaffected
    assert!(is_creature(&game, bears_id));
    let types = get_effective_types(&game, bears_id);
    assert!(types.contains(&CardType::Creature));
    assert!(!types.contains(&CardType::Land));
}

// ===========================================================================
// Part B Test 5: Blood Moon removed → effect disappears
// ===========================================================================

// COVERS-PARTIAL: ATOM-611.3b-001
#[test]
fn test_blood_moon_removed_restores_subtypes() {
    use mtgsim::objects::card_data::CardDataBuilder;

    let mut game = setup_two_player_game();

    // Create a nonbasic dual land
    let dual_data = CardDataBuilder::new("Tropical Island")
        .card_type(CardType::Land)
        .subtype(Subtype::Land(LandType::Forest))
        .subtype(Subtype::Land(LandType::Island))
        .build();
    let dual_id = put_on_battlefield(&mut game, dual_data, 0);

    // Put Blood Moon on the battlefield
    let blood_moon_id = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 0);

    // Verify Blood Moon is active
    let subtypes = get_effective_subtypes(&game, dual_id);
    assert!(subtypes.contains(&Subtype::Land(LandType::Mountain)));
    assert!(!subtypes.contains(&Subtype::Land(LandType::Forest)));
    assert!(!subtypes.contains(&Subtype::Land(LandType::Island)));

    // Remove Blood Moon from battlefield (simulate destruction)
    game.continuous_effects.remove_by_source(blood_moon_id);

    // Subtypes should be restored to base
    let subtypes = get_effective_subtypes(&game, dual_id);
    assert!(subtypes.contains(&Subtype::Land(LandType::Forest)));
    assert!(subtypes.contains(&Subtype::Land(LandType::Island)));
    assert!(!subtypes.contains(&Subtype::Land(LandType::Mountain)));
}
