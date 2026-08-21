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
    get_effective_abilities, get_effective_power, get_effective_subtypes,
    get_effective_supertypes, get_effective_toughness, get_effective_types, has_keyword,
    is_creature,
};
use mtgsim::types::keywords::KeywordAbility;
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

// ===========================================================================
// Part B: CR 305.7 ability stripping + CR 305.6 intrinsic mana abilities
// ===========================================================================

/// Which mana types a permanent's effective mana abilities can produce.
/// Sorted for stable comparison.
fn produced_mana_types(
    game: &mtgsim::state::game_state::GameState,
    id: mtgsim::types::ids::ObjectId,
) -> Vec<ManaType> {
    use mtgsim::objects::card_data::AbilityType;
    use mtgsim::types::effects::{Effect, Primitive};

    let mut out: Vec<ManaType> = get_effective_abilities(game, id)
        .iter()
        .filter(|a| a.ability_type == AbilityType::Mana)
        .filter_map(|a| match &a.effect {
            Effect::Atom(Primitive::ProduceMana(output), _) => {
                output.mana.first().map(|(mt, _)| *mt)
            }
            _ => None,
        })
        .collect();
    out.sort_by_key(|mt| format!("{:?}", mt));
    out
}

// COVERS: ATOM-305.7-002
#[test]
fn test_blood_moon_strips_printed_abilities_and_grants_intrinsic_red() {
    let mut game = setup_two_player_game();
    let land_id = put_on_battlefield(&mut game, phase_ld_cards::dual_land_ub(), 0);

    // Base: the printed {U} and {B} mana abilities.
    assert_eq!(
        produced_mana_types(&game, land_id),
        vec![ManaType::Black, ManaType::Blue],
    );

    let _blood_moon = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 0);

    // CR 305.7: printed abilities gone, intrinsic Mountain ability gained.
    assert_eq!(
        produced_mana_types(&game, land_id),
        vec![ManaType::Red],
        "land should tap only for red",
    );
    assert!(get_effective_subtypes(&game, land_id).contains(&Subtype::Land(LandType::Mountain)));
}

// COVERS-PARTIAL: ATOM-305.7-003
// PARTIAL: the atom grants a non-keyword activated ability ("{T}: Draw a card").
// Only the keyword channel (Primitive::GrantKeyword) exists today, so this
// proves the survival rule using flying instead.
#[test]
fn test_blood_moon_does_not_strip_ability_granted_by_another_effect() {
    use mtgsim::oracle::characteristics::has_keyword;
    use mtgsim::types::keywords::KeywordAbility;

    let mut game = setup_two_player_game();
    let land_id = put_on_battlefield(&mut game, phase_ld_cards::dual_land_ub(), 0);
    let _granter = put_on_battlefield(&mut game, phase_ld_cards::lands_have_flying(), 0);

    assert!(has_keyword(&game, land_id, KeywordAbility::Flying));

    let _blood_moon = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 0);

    // The granted ability survives; the land's own printed abilities do not.
    assert!(
        has_keyword(&game, land_id, KeywordAbility::Flying),
        "CR 305.7 does not remove abilities granted by other effects",
    );
    assert_eq!(produced_mana_types(&game, land_id), vec![ManaType::Red]);
}

/// The same board, but the Layer 6 granter is registered with an EARLIER
/// timestamp than Blood Moon.
///
/// This pins the assumption that makes `AbilityOrigin` unnecessary: Layer 6 runs
/// after Layer 4, so a granted ability is never in the frame when CR 305.7's
/// strip executes — regardless of the order the effects were registered in. If
/// anything ever seeds a granted ability into the frame before Layer 4, the
/// unconditional `clear()` in `land_types::apply_set_subtypes` starts eating it
/// and this test fails.
#[test]
fn test_layer6_grant_survives_blood_moon_registered_first() {
    use mtgsim::oracle::characteristics::has_keyword;
    use mtgsim::types::keywords::KeywordAbility;

    let mut game = setup_two_player_game();
    // Granter enters BEFORE the land and before Blood Moon — earliest timestamp.
    let _granter = put_on_battlefield(&mut game, phase_ld_cards::lands_have_flying(), 0);
    let land_id = put_on_battlefield(&mut game, phase_ld_cards::dual_land_ub(), 0);
    let _blood_moon = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 0);

    assert!(has_keyword(&game, land_id, KeywordAbility::Flying));
    assert_eq!(produced_mana_types(&game, land_id), vec![ManaType::Red]);
}

// COVERS: ATOM-305.6-002
#[test]
fn test_urborg_grants_intrinsic_black_and_keeps_existing() {
    let mut game = setup_two_player_game();
    let mountain_id = put_on_battlefield(&mut game, basic_lands::mountain(), 0);

    assert_eq!(produced_mana_types(&game, mountain_id), vec![ManaType::Red]);

    let _urborg = put_on_battlefield(&mut game, phase_ld_cards::urborg_effect(), 0);

    // "In addition to" — keeps Mountain and its red mana, gains Swamp and black.
    let subtypes = get_effective_subtypes(&game, mountain_id);
    assert!(subtypes.contains(&Subtype::Land(LandType::Mountain)));
    assert!(subtypes.contains(&Subtype::Land(LandType::Swamp)));
    assert_eq!(
        produced_mana_types(&game, mountain_id),
        vec![ManaType::Black, ManaType::Red],
    );
}

#[test]
fn test_urborg_on_a_real_swamp_does_not_double_grant_black() {
    let mut game = setup_two_player_game();
    let swamp_id = put_on_battlefield(&mut game, basic_lands::swamp(), 0);
    let _urborg = put_on_battlefield(&mut game, phase_ld_cards::urborg_effect(), 0);

    assert_eq!(
        produced_mana_types(&game, swamp_id),
        vec![ManaType::Black],
        "a Swamp told it is a Swamp must not gain a second black mana ability",
    );
}

// NOTE: there is deliberately no Blood-Moon-vs-Urborg ordering test here.
//
// With the real cards the order does NOT matter: Urborg, Tomb of Yawgmoth is
// itself a nonbasic (Legendary) Land, so Blood Moon turns Urborg into a Mountain
// and CR 305.7 strips its rules-text ability — Urborg's effect ceases to exist.
// That is a CR 613.8a(b) dependency (applying Blood Moon changes the existence of
// Urborg's effect), and there is no reverse dependency, because Urborg grants the
// Swamp *subtype* and never the Basic *supertype* (CR 305.8). So Urborg is applied
// last, by which point it does nothing. Blood Moon wins in both orders.
//
// Testing that needs two things we don't have: the CR 613.8 dependency algorithm,
// and static-ability deregistration (stripping a permanent's static ability must
// retire the continuous effect it registered at ETB). Both are recorded in
// codebase-state.md → Deferred Migrations. `phase_ld_cards::urborg_effect()` is
// modeled as an Enchantment precisely to stay clear of this.

/// The CR 305.7 strip is a `clear()` on a frame rebuilt from `CardData` every
/// call, not a mutation — so removing Blood Moon needs no undo step.
#[test]
fn test_blood_moon_removed_restores_printed_abilities() {
    let mut game = setup_two_player_game();
    let land_id = put_on_battlefield(&mut game, phase_ld_cards::dual_land_ub(), 0);
    let blood_moon_id = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 0);

    assert_eq!(produced_mana_types(&game, land_id), vec![ManaType::Red]);

    game.continuous_effects.remove_by_source(blood_moon_id);

    assert_eq!(
        produced_mana_types(&game, land_id),
        vec![ManaType::Black, ManaType::Blue],
        "printed abilities return once the effect is gone",
    );
}

// COVERS: COMP-305.7+305.6-001
#[test]
fn test_activating_blood_mooned_land_produces_red() {
    use mtgsim::oracle::mana_helpers::available_mana_sources;

    let mut game = setup_two_player_game();
    let land_id = put_on_battlefield(&mut game, phase_ld_cards::dual_land_ub(), 0);
    let _blood_moon = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 0);

    // The mana-source enumeration offers only red for this land.
    let sources: Vec<_> = available_mana_sources(&game, 0)
        .into_iter()
        .filter(|s| s.permanent_id == land_id)
        .collect();
    assert_eq!(sources.len(), 1, "one intrinsic mana ability");
    assert_eq!(sources[0].produces, ManaType::Red);

    // Activate it by the id the enumeration handed out. This is the step that
    // fails if intrinsic ability ids aren't stable across compute calls.
    game.activate_mana_ability(0, land_id, sources[0].ability_id)
        .expect("intrinsic Mountain ability should be activatable");

    assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 1);
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Blue), 0);
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Black), 0);
}

// ===========================================================================
// CR 613.6 — an effect continues to apply to the SAME SET OF OBJECTS in each
// applicable layer, even after an earlier layer's part breaks its own filter.
// ===========================================================================

// COVERS-PARTIAL: ATOM-613.6-003
//
// Partial: the atom's scenario has the *ability* removed mid-process by a
// Layer 6 producer, which does not exist yet. This proves the other half of
// the same sentence in CR 613.6 — "will continue to be applied to the same set
// of objects in each other applicable layer".
#[test]
fn test_613_6_effect_keeps_applying_after_its_own_filter_breaks() {
    use mtgsim::objects::card_data::CardDataBuilder;

    let mut game = setup_two_player_game();

    // A vanilla noncreature artifact with no printed P/T.
    let artifact_data = CardDataBuilder::new("Ornithopter Shell")
        .card_type(CardType::Artifact)
        .build();
    let artifact_id = put_on_battlefield(&mut game, artifact_data, 0);

    assert_eq!(get_effective_power(&game, artifact_id), None);

    put_on_battlefield(&mut game, phase_ld_cards::march_of_the_machines(), 0);

    // Layer 4: the artifact becomes a creature.
    assert!(
        is_creature(&game, artifact_id),
        "Layer 4 AddType(Creature) should have applied"
    );

    // Layer 7b: it is no longer a *noncreature* artifact, so re-evaluating the
    // filter here would drop the SetPowerToughness half. CR 613.6 says the
    // effect keeps applying to the set it started on.
    assert_eq!(
        get_effective_power(&game, artifact_id),
        Some(2),
        "CR 613.6: the Layer 7b part must still apply after Layer 4 broke the filter"
    );
    assert_eq!(get_effective_toughness(&game, artifact_id), Some(2));
}

// ===========================================================================
// CR 613.7a / Deferred Migrations item 7 — a static ability stripped by CR
// 305.7 must also retire the continuous effect it generated.
// ===========================================================================

// COVERS-PARTIAL: ATOM-305.7-002
//
// Partial: ATOM-305.7-002 is about the land's own mana abilities, which
// test_blood_moon_strips_printed_abilities_and_grants_intrinsic_red already
// covers. This adds the consequence that was missing — the *effect* the
// stripped ability had registered stops applying too.
#[test]
fn test_blood_moon_retires_the_effect_a_stripped_ability_registered() {
    let mut game = setup_two_player_game();

    let bear_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let land_id = put_on_battlefield(&mut game, phase_ld_cards::land_creatures_have_flying(), 0);

    assert!(
        has_keyword(&game, bear_id, KeywordAbility::Flying),
        "the land's static ability should grant flying before Blood Moon"
    );

    put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 0);

    // CR 305.7: the land is now a Mountain and has lost its printed abilities.
    let abilities = get_effective_abilities(&game, land_id);
    assert!(
        !abilities
            .iter()
            .any(|a| a.ability_type == mtgsim::objects::card_data::AbilityType::Static),
        "CR 305.7 should have stripped the land's static ability"
    );

    // ... so the effect that ability generated must stop applying (item 7).
    assert!(
        !has_keyword(&game, bear_id, KeywordAbility::Flying),
        "a stripped static ability must retire the continuous effect it registered"
    );
}

// The other direction: Blood Moon leaves, the ability is back, and so is its
// effect — with no re-registration, because the effect never left the registry.
#[test]
fn test_effect_returns_when_blood_moon_leaves() {
    let mut game = setup_two_player_game();

    let bear_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    put_on_battlefield(&mut game, phase_ld_cards::land_creatures_have_flying(), 0);
    let blood_moon_id = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 0);

    assert!(!has_keyword(&game, bear_id, KeywordAbility::Flying));

    game.change_zone(blood_moon_id, Zone::Graveyard).unwrap();

    assert!(
        has_keyword(&game, bear_id, KeywordAbility::Flying),
        "the land's ability is back, so its effect applies again"
    );
}

// COVERS-PARTIAL: ATOM-613.7a-001
//
// Partial: the atom is the CR's Rune of Flight / Colossus Hammer example, which
// turns on 613.7a's second clause (the timestamp of the effect that *created*
// the ability). That needs GrantAbility, a Layer 6 producer that doesn't exist.
// This covers the first clause — the effect shares the object's timestamp.
#[test]
fn test_static_effect_shares_its_objects_timestamp() {
    let mut game = setup_two_player_game();

    // Something else on the battlefield first, so the land's own timestamp is
    // not 0 and the assertion can't pass by coincidence.
    put_on_battlefield(&mut game, basic_lands::forest(), 0);
    put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);

    let land_id = put_on_battlefield(&mut game, phase_ld_cards::land_creatures_have_flying(), 0);
    let object_timestamp = game.battlefield.get(&land_id).unwrap().timestamp;

    let effect_timestamps: Vec<u64> = game
        .continuous_effects
        .iter()
        .filter(|e| e.source == land_id)
        .map(|e| e.timestamp)
        .collect();

    assert_eq!(
        effect_timestamps.len(),
        1,
        "the land's one static ability should have registered one effect"
    );
    assert_eq!(
        effect_timestamps[0], object_timestamp,
        "CR 613.7a: a static ability's effect has the same timestamp as the object it is on"
    );
}

// COVERS: ATOM-113.6-001
#[test]
fn test_static_ability_does_not_function_from_the_graveyard() {
    let mut game = setup_two_player_game();

    let bear_id = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let land_id = put_on_battlefield(&mut game, phase_ld_cards::land_creatures_have_flying(), 0);

    assert!(has_keyword(&game, bear_id, KeywordAbility::Flying));

    game.change_zone(land_id, Zone::Graveyard).unwrap();

    // CR 113.6 — an ability of a permanent functions only while that permanent
    // is on the battlefield.
    assert!(
        !has_keyword(&game, bear_id, KeywordAbility::Flying),
        "a static ability must stop functioning once its source is in the graveyard"
    );
    assert_eq!(
        game.continuous_effects
            .iter()
            .filter(|e| e.source == land_id)
            .count(),
        0,
        "the effect should have been deregistered on the zone change"
    );
}

// The CR 613.7a existence check asks whether an effect's source still has the
// ability that generates it — which is itself a characteristics query on that
// source. When the source is what its own effect strips, that question is
// self-referential, and the only reason it terminates is that each round asks
// at a strictly lower layer ceiling (layers-architecture.md §5.2).
//
// See `self_stripping_land`'s doc comment for the real board this stands in for.
// Assertions are deliberately confined to what holds under either ordering:
// which of two Layer 4 effects wins is CR 613.8's business, and 613.8 is not
// implemented (Deferred Migrations item 8).
#[test]
fn test_self_stripping_land_terminates_and_is_stable() {
    let mut game = setup_two_player_game();

    let steppe_id = put_on_battlefield(&mut game, phase_ld_cards::self_stripping_land(), 0);

    let other_data = mtgsim::objects::card_data::CardDataBuilder::new("Steam Vents")
        .card_type(CardType::Land)
        .subtype(Subtype::Land(LandType::Island))
        .build();
    let other_id = put_on_battlefield(&mut game, other_data, 0);

    // Terminates rather than recursing forever, and gives the same answer twice.
    let first = get_effective_subtypes(&game, steppe_id);
    let second = get_effective_subtypes(&game, steppe_id);
    assert_eq!(first, second, "repeated queries must agree");

    // The effect still exists: an effect that stripped itself out of existence
    // would leave the other nonbasic land alone. CR 613.6 — the walk restarts
    // from CardData every time, so the strip is a within-walk consequence and
    // never persistent state.
    assert!(
        get_effective_subtypes(&game, other_id).contains(&Subtype::Land(LandType::Mountain)),
        "the self-stripping land's effect must still apply to other nonbasic lands"
    );
}
