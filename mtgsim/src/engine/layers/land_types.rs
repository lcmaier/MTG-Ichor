//! CR 305.6 / 305.7 — basic land types and their intrinsic mana abilities.
//!
//! This is the one ability-adding/removing path that does **not** live in
//! Layer 6. The CR puts it in the object rules rather than in CR 613, and the
//! layer system models it as application-time logic inside Layer 4's
//! `apply_modification` (see `plans/layers-architecture.md` §3.5a).
//!
//! **CR 305.6:** "An object with the land card type and a basic land type has
//! the intrinsic ability `{T}: Add [mana symbol]`, even if the text box doesn't
//! actually contain that text or the object has no text box."
//!
//! **CR 305.7:** "If an effect sets a land's subtype to one or more of the basic
//! land types, the land no longer has its old land type. It loses all abilities
//! generated from its rules text, its old land types, and any copiable effects
//! affecting that land, and it gains the appropriate mana ability for each new
//! basic land type. Note that this doesn't remove any abilities that were
//! granted to the land by other effects. Setting a land's subtype doesn't add or
//! remove any card types (such as creature) or supertypes (such as basic,
//! legendary, and snow) the land may have. If a land gains one or more land
//! types in addition to its own, it keeps its land types and rules text, and it
//! gains the new land types and mana abilities."
//!
//! ## Why the strip is an unconditional `clear()`
//!
//! 305.7's "doesn't remove abilities granted by other effects" needs no marker
//! on the ability, because layer ordering already guarantees it: Layer 6 runs
//! *after* Layer 4, so a granted ability isn't in the frame yet when this code
//! runs. Everything that *is* in the frame at Layer 4 is stripped by 305.7 —
//! printed abilities ("generated from its rules text"), Layer 1 copy-derived
//! ones ("any copiable effects"), Layer 3 text-derived ones (still rules text),
//! and Layer 4 intrinsics ("its old land types"). Layer 2 doesn't touch
//! abilities. So there are no two buckets to tell apart.
//!
//! This is load-bearing. `test_layer6_grant_survives_blood_moon_registered_first` in
//! `tests/phase_ld_integration_test.rs` pins it: if anything ever seeds a
//! granted ability into the frame before Layer 4, that test fails.
//!
//! ## Why the strip needs no undo
//!
//! `compute_characteristics` rebuilds the frame from `CardData` on every call —
//! `clear()` mutates a local clone, never the card. When the effect is removed
//! from the registry the strip simply stops running, and the printed abilities
//! are back on the next call.

use uuid::Uuid;

use crate::engine::layers::types::EffectiveCharacteristics;
use crate::objects::card_data::{AbilityDef, AbilityType};
use crate::types::card_types::{CardType, LandType, Subtype};
use crate::types::costs::Cost;
use crate::types::effects::{AmountExpr, Effect, EffectRecipient, ManaOutput, Primitive};
use crate::types::ids::{AbilityId, ObjectId};
use crate::types::mana::ManaType;

/// The mana type a basic land type intrinsically produces (CR 305.6).
/// Returns `None` for non-basic land types (Gate, Locus, Desert, ...).
fn intrinsic_mana_type(land_type: LandType) -> Option<ManaType> {
    match land_type {
        LandType::Plains => Some(ManaType::White),
        LandType::Island => Some(ManaType::Blue),
        LandType::Swamp => Some(ManaType::Black),
        LandType::Mountain => Some(ManaType::Red),
        LandType::Forest => Some(ManaType::Green),
        _ => None,
    }
}

/// Stable `AbilityId` for the intrinsic mana ability that `object_id` gets from
/// `land_type`.
///
/// Derived (UUID v5) rather than random (v4) because intrinsic abilities are
/// synthesized inside `compute_characteristics`, which is a read-only query run
/// many times per turn and has nowhere to store anything. A `new_ability_id()`
/// here would mint a fresh id on every call, and ids are used as activation
/// handles: `available_mana_sources` hands one out in `ManaSource`, then
/// `activate_mana_ability` recomputes and looks it up. Random ids would never
/// match and every intrinsic mana ability would be unactivatable.
///
/// Keying on the object id also makes intrinsics unique per *object*, which
/// printed ability ids are not: `AbilityId` is minted once per `CardData`, and
/// `GameObject` holds an `Arc<CardData>`, so two objects sharing a card share
/// its ability ids. Nothing collides today only because `CardRegistry::create`
/// re-runs the factory for every copy; tokens and copy effects will not. Code
/// that needs to identify an ability *on an object* uses the pair — see
/// `mana_helpers::enumerate_activatable_mana_abilities` and `EffectOrigin::StaticAbility`.
fn intrinsic_ability_id(object_id: ObjectId, land_type: LandType) -> AbilityId {
    // Discriminant byte is enough: only the five basic types reach here, and
    // the object id supplies the uniqueness.
    let tag = [land_type as u8];
    Uuid::new_v5(&object_id, &tag)
}

/// Build the intrinsic `{T}: Add X` mana ability a land gets from `land_type`
/// (CR 305.6). Returns `None` for non-basic land types.
///
/// Mirrors the ability `CardDataBuilder::mana_ability_single` puts on a printed
/// basic land, so the two are indistinguishable to the activation path.
pub(crate) fn intrinsic_mana_ability(
    object_id: ObjectId,
    land_type: LandType,
) -> Option<AbilityDef> {
    let mana_type = intrinsic_mana_type(land_type)?;
    Some(AbilityDef {
        is_characteristic_defining: false,
        id: intrinsic_ability_id(object_id, land_type),
        ability_type: AbilityType::Mana,
        costs: vec![Cost::Tap],
        effect: Effect::Atom(
            Primitive::ProduceMana(ManaOutput {
                mana: vec![(mana_type, AmountExpr::Fixed(1))],
                special: vec![],
            }),
            EffectRecipient::Implicit,
        ),
    })
}

/// The basic land types in `subtypes`, in `LandType` declaration order (WUBRG).
///
/// Only matters when an effect sets a land to *several* basic types at once. No
/// shipped card does that — Blood Moon and Spreading Seas each set exactly one —
/// so today this is exercised only by `set_subtypes_to_multiple_basics_grants_
/// one_ability_each`. It is here because `std::collections::HashSet` seeds its
/// hasher per process, so iteration order differs between runs of the binary;
/// the push order becomes the order of `chars.abilities`, which
/// `activatable_abilities` turns into activation indices. Left unsorted, the
/// first such card would quietly make `fuzz_games` unreproducible at a fixed
/// seed — a bug that reads as nondeterministic engine behavior rather than as an
/// ordering problem. Three lines to never have to find that.
fn basic_land_types_sorted(subtypes: &std::collections::HashSet<Subtype>) -> Vec<LandType> {
    let mut found: Vec<LandType> = subtypes
        .iter()
        .filter_map(|s| match s {
            Subtype::Land(lt) if lt.is_basic_land_type() => Some(*lt),
            _ => None,
        })
        .collect();
    found.sort_by_key(|lt| *lt as u8);
    found
}

/// Layer 4 `SetSubtypes` — replace subtypes, applying CR 305.7 when the new
/// subtypes include a basic land type and the object is a land.
///
/// Called from `apply_modification`. Card types and supertypes are deliberately
/// untouched: Blood Moon does not make a land basic (ATOM-305.8-001).
pub(crate) fn apply_set_subtypes(
    chars: &mut EffectiveCharacteristics,
    new_subtypes: &std::collections::HashSet<Subtype>,
    object_id: ObjectId,
) {
    let is_land = chars.types.contains(&CardType::Land);
    let new_basics = basic_land_types_sorted(new_subtypes);

    // Set semantics: always replace, for lands and non-lands alike.
    chars.subtypes = new_subtypes.clone();

    if !is_land || new_basics.is_empty() {
        // Non-land, or set to non-basic land types only: no 305.7 side effects.
        return;
    }

    // CR 305.7 — lose abilities from rules text, old land types, and copiable
    // effects. See the module docs for why this is an unconditional clear.
    chars.abilities.clear();
    chars.keywords.clear();

    // ... and gain the appropriate mana ability for each new basic land type.
    for land_type in new_basics {
        if let Some(ability) = intrinsic_mana_ability(object_id, land_type) {
            chars.abilities.push(ability);
        }
    }
}

/// Layer 4 `AddSubtype` — CR 305.7's additive clause ("If a land gains one or
/// more land types *in addition to* its own, it keeps its land types and rules
/// text, and it gains the new land types and mana abilities").
///
/// Urborg-style. Nothing is stripped; the intrinsic mana ability for a newly
/// gained basic land type is added.
pub(crate) fn apply_add_subtype(
    chars: &mut EffectiveCharacteristics,
    subtype: &Subtype,
    object_id: ObjectId,
) {
    let newly_added = chars.subtypes.insert(subtype.clone());

    // Only grant on a genuine gain — Urborg hitting an actual Swamp must not
    // produce a second {B} alongside the printed one.
    if !newly_added || !chars.types.contains(&CardType::Land) {
        return;
    }

    if let Subtype::Land(land_type) = subtype {
        if let Some(ability) = intrinsic_mana_ability(object_id, *land_type) {
            chars.abilities.push(ability);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ids::new_object_id;
    use std::collections::HashSet;

    /// Minimal land frame with one printed mana ability, for exercising the
    /// 305.6/305.7 helpers directly without building a whole GameState.
    fn land_frame(printed_subtypes: &[LandType]) -> EffectiveCharacteristics {
        let mut types = HashSet::new();
        types.insert(CardType::Land);
        let mut subtypes = HashSet::new();
        for lt in printed_subtypes {
            subtypes.insert(Subtype::Land(*lt));
        }
        EffectiveCharacteristics {
            name: "Test Land".to_string(),
            mana_cost: None,
            colors: HashSet::new(),
            types,
            subtypes,
            supertypes: HashSet::new(),
            keywords: HashSet::new(),
            abilities: vec![AbilityDef {
                is_characteristic_defining: false,
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Mana,
                costs: vec![Cost::Tap],
                effect: Effect::Atom(
                    Primitive::ProduceMana(ManaOutput {
                        mana: vec![(ManaType::Blue, AmountExpr::Fixed(1))],
                        special: vec![],
                    }),
                    EffectRecipient::Implicit,
                ),
            }],
            power: None,
            toughness: None,
            controller: 0,
            control_since_turn: 0,
        }
    }

    fn produced_mana(ability: &AbilityDef) -> Option<ManaType> {
        match &ability.effect {
            Effect::Atom(Primitive::ProduceMana(output), _) => {
                output.mana.first().map(|(mt, _)| *mt)
            }
            _ => None,
        }
    }

    fn subtype_set(types: &[LandType]) -> HashSet<Subtype> {
        types.iter().map(|lt| Subtype::Land(*lt)).collect()
    }

    #[test]
    fn intrinsic_ability_id_is_stable_across_calls() {
        let id = new_object_id();
        let first = intrinsic_ability_id(id, LandType::Mountain);
        let second = intrinsic_ability_id(id, LandType::Mountain);
        assert_eq!(
            first, second,
            "intrinsic ids must reproduce — they are activation handles"
        );
    }

    #[test]
    fn intrinsic_ability_id_differs_per_object_and_per_type() {
        let a = new_object_id();
        let b = new_object_id();
        assert_ne!(
            intrinsic_ability_id(a, LandType::Mountain),
            intrinsic_ability_id(b, LandType::Mountain),
            "different lands get different ids"
        );
        assert_ne!(
            intrinsic_ability_id(a, LandType::Mountain),
            intrinsic_ability_id(a, LandType::Swamp),
            "different land types on one land get different ids"
        );
    }

    #[test]
    fn intrinsic_mana_ability_maps_all_five_basics() {
        let id = new_object_id();
        let expected = [
            (LandType::Plains, ManaType::White),
            (LandType::Island, ManaType::Blue),
            (LandType::Swamp, ManaType::Black),
            (LandType::Mountain, ManaType::Red),
            (LandType::Forest, ManaType::Green),
        ];
        for (land_type, mana_type) in expected {
            let ability = intrinsic_mana_ability(id, land_type)
                .unwrap_or_else(|| panic!("{:?} should have an intrinsic ability", land_type));
            assert_eq!(ability.ability_type, AbilityType::Mana);
            assert_eq!(ability.costs, vec![Cost::Tap]);
            assert_eq!(produced_mana(&ability), Some(mana_type));
        }
    }

    #[test]
    fn intrinsic_mana_ability_none_for_nonbasic_land_types() {
        let id = new_object_id();
        assert!(intrinsic_mana_ability(id, LandType::Gate).is_none());
        assert!(intrinsic_mana_ability(id, LandType::Locus).is_none());
        assert!(intrinsic_mana_ability(id, LandType::Desert).is_none());
    }

    // COVERS-PARTIAL: ATOM-305.7-002
    #[test]
    fn set_subtypes_to_basic_strips_and_regrants() {
        let id = new_object_id();
        let mut chars = land_frame(&[LandType::Island]);

        apply_set_subtypes(&mut chars, &subtype_set(&[LandType::Mountain]), id);

        assert_eq!(chars.subtypes, subtype_set(&[LandType::Mountain]));
        assert_eq!(chars.abilities.len(), 1, "printed ability stripped");
        assert_eq!(produced_mana(&chars.abilities[0]), Some(ManaType::Red));
    }

    // COVERS-PARTIAL: ATOM-305.8-001
    #[test]
    fn set_subtypes_leaves_types_and_supertypes_alone() {
        let id = new_object_id();
        let mut chars = land_frame(&[LandType::Island]);
        chars.supertypes.insert(crate::types::card_types::Supertype::Legendary);

        apply_set_subtypes(&mut chars, &subtype_set(&[LandType::Mountain]), id);

        assert!(chars.types.contains(&CardType::Land), "still a land");
        assert!(
            chars.supertypes.contains(&crate::types::card_types::Supertype::Legendary),
            "supertypes survive"
        );
        assert!(
            !chars.supertypes.contains(&crate::types::card_types::Supertype::Basic),
            "305.7 must not add Basic"
        );
    }

    #[test]
    fn set_subtypes_to_multiple_basics_grants_one_ability_each() {
        let id = new_object_id();
        let mut chars = land_frame(&[LandType::Island]);

        apply_set_subtypes(
            &mut chars,
            &subtype_set(&[LandType::Mountain, LandType::Forest]),
            id,
        );

        assert_eq!(chars.abilities.len(), 2);
        let produced: Vec<_> = chars.abilities.iter().filter_map(produced_mana).collect();
        // Declaration order: Mountain precedes Forest.
        assert_eq!(produced, vec![ManaType::Red, ManaType::Green]);
    }

    #[test]
    fn set_subtypes_to_nonbasic_replaces_without_stripping() {
        let id = new_object_id();
        let mut chars = land_frame(&[LandType::Island]);

        apply_set_subtypes(&mut chars, &subtype_set(&[LandType::Gate]), id);

        assert_eq!(chars.subtypes, subtype_set(&[LandType::Gate]));
        assert_eq!(chars.abilities.len(), 1, "no basic type, so no 305.7 strip");
        assert_eq!(produced_mana(&chars.abilities[0]), Some(ManaType::Blue));
    }

    #[test]
    fn set_subtypes_on_non_land_does_not_strip() {
        let id = new_object_id();
        let mut chars = land_frame(&[LandType::Island]);
        chars.types.clear();
        chars.types.insert(CardType::Creature);

        apply_set_subtypes(&mut chars, &subtype_set(&[LandType::Mountain]), id);

        assert_eq!(chars.subtypes, subtype_set(&[LandType::Mountain]));
        assert_eq!(chars.abilities.len(), 1, "305.7 is land-only");
    }

    #[test]
    fn set_subtypes_strips_printed_keywords() {
        let id = new_object_id();
        let mut chars = land_frame(&[LandType::Island]);
        chars.keywords.insert(crate::types::keywords::KeywordFlag::Hexproof);

        apply_set_subtypes(&mut chars, &subtype_set(&[LandType::Mountain]), id);

        assert!(
            chars.keywords.is_empty(),
            "keywords are abilities (CR 702) and 305.7 strips them too"
        );
    }

    // COVERS-PARTIAL: ATOM-305.6-002
    #[test]
    fn add_subtype_keeps_old_abilities_and_grants_new() {
        let id = new_object_id();
        let mut chars = land_frame(&[LandType::Island]);

        apply_add_subtype(&mut chars, &Subtype::Land(LandType::Swamp), id);

        assert_eq!(chars.subtypes, subtype_set(&[LandType::Island, LandType::Swamp]));
        assert_eq!(chars.abilities.len(), 2, "printed ability kept, {{B}} added");
        assert_eq!(produced_mana(&chars.abilities[0]), Some(ManaType::Blue));
        assert_eq!(produced_mana(&chars.abilities[1]), Some(ManaType::Black));
    }

    #[test]
    fn add_subtype_already_present_does_not_double_grant() {
        let id = new_object_id();
        let mut chars = land_frame(&[LandType::Swamp]);

        apply_add_subtype(&mut chars, &Subtype::Land(LandType::Swamp), id);

        assert_eq!(
            chars.abilities.len(),
            1,
            "Urborg on a real Swamp must not add a second {{B}}"
        );
    }

    #[test]
    fn add_subtype_nonbasic_grants_nothing() {
        let id = new_object_id();
        let mut chars = land_frame(&[LandType::Island]);

        apply_add_subtype(&mut chars, &Subtype::Land(LandType::Gate), id);

        assert_eq!(chars.abilities.len(), 1, "Gate has no intrinsic mana ability");
    }

    #[test]
    fn add_subtype_on_non_land_grants_nothing() {
        let id = new_object_id();
        let mut chars = land_frame(&[]);
        chars.types.clear();
        chars.types.insert(CardType::Creature);

        apply_add_subtype(&mut chars, &Subtype::Land(LandType::Swamp), id);

        assert!(chars.abilities.len() == 1, "305.6 requires the land card type");
    }
}
