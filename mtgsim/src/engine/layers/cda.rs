//! CR 604.3 / 613.3 / 613.4a — characteristic-defining abilities.
//!
//! A CDA is not a registry effect. CR 604.3a(3) says a CDA "does not directly
//! affect the characteristics of any other objects", which is a criterion
//! rather than an observation about Tarmogoyf: **every CDA applies to exactly
//! the object that has it**. So there is nothing for an `AffectedSet` to
//! select, no filter to evaluate, and no row to register. `compute.rs` applies
//! them straight off the object's own effective ability list, and four things
//! fall out that the registry shape would have had to build:
//!
//! - **CR 613.3's ordering.** "Apply effects from characteristic-defining
//!   abilities first, then all other effects in timestamp order." Applying the
//!   intrinsic pass before the layer's registry slice *is* that sentence.
//!
//! - **CR 613.7a's existence check.** `chars.abilities` at Layer 7a is already
//!   the post-Layer-6 ability set, so Humility's "creatures lose all abilities"
//!   removes a Tarmogoyf's CDA before 7a can read it — Tarmogoyf is 1/1, with
//!   no `static_ability_still_exists` call involved. The same holds one layer
//!   earlier: `land_types` clears abilities for CR 305.7 in Layer 4, so a
//!   Blood-Mooned land has lost a colour CDA by the time Layer 5 looks.
//!
//! - **CR 613.8a(c)'s guard.** Since no CDA is ever in the registry, every pair
//!   the dependency algorithm will eventually see is non-CDA↔non-CDA — which is
//!   exactly the first clause of 613.8a(c), for free.
//!
//! - **CR 604.3's "function in all zones".** This reads `game.objects`, not
//!   `game.battlefield`, so a Tarmogoyf in a graveyard has a power and toughness
//!   without any of Deferred Migrations item 9's zone-aware `AffectedSet` work.
//!   That item is about *filter-based* effects reaching other zones, which is a
//!   different shape.
//!
//! What it does not cover: CDA↔CDA dependency, 613.8a(c)'s second clause. That
//! needs a CDA reading a characteristic another CDA sets **in the same layer** —
//! a Layer 7a CDA reading another creature's power, say. Every printed CDA reads
//! either non-layer information (graveyards, hands, life) or strictly
//! lower-layer information (Nightmare and Master of Etherium read Layer 4 type
//! counts at Layer 7a), and none of those can be dependent under 613.8a(a). If
//! such a card is ever printed, this pass has to publish into 613.8's ordering
//! step; recorded in `codebase-state.md` under the 613.8 item.

use crate::engine::layers::compute::{apply_modification, FrameCache};
use crate::engine::layers::types::{EffectModification, EffectiveCharacteristics, Layer, PtValue};
use crate::objects::card_data::{AbilityDef, AbilityType};
use crate::state::game_state::GameState;
use crate::types::effects::{ColorChange, Effect, EffectRecipient, Primitive};
use crate::types::ids::ObjectId;

/// The layers a CDA can occupy.
///
/// CR 604.3a(1) admits exactly four characteristics — colors, subtypes, power
/// and toughness — so this is the complete list, and `compute.rs` only calls
/// into this module for these three layers.
pub(super) const CDA_LAYERS: [Layer; 3] =
    [Layer::Layer4Type, Layer::Layer5Color, Layer::Layer7aCdaPT];

/// Apply this object's own CDAs for `layer`, ahead of the layer's registry
/// effects (CR 613.3).
///
/// Re-reads `chars.abilities` at every layer rather than caching a decision
/// from the start of the walk: that list is what earlier layers have already
/// done to the object, and the whole correctness argument above rests on
/// reading it late.
pub(super) fn apply_intrinsic_cdas(
    game: &GameState,
    chars: &mut EffectiveCharacteristics,
    object_id: ObjectId,
    layer: Layer,
    layer_index: usize,
    cache: &mut FrameCache,
) {
    // Collected first because applying mutates `chars`, which we are reading.
    // Empty `Vec` doesn't allocate, so the common case — an object with no CDA
    // — costs one flag scan over a list that is almost always 0-2 entries long.
    let pending = collect_modifications(chars, layer);

    for modification in pending {
        apply_modification(&modification, chars, object_id, game, layer_index, cache);
    }
}

/// Does this object have any CDA at all?
///
/// Only used for the `apply_effects` fast path, where the registry is empty and
/// the object is off the battlefield — so nothing can have added an ability and
/// the printed list is exact.
pub(super) fn has_any_cda(chars: &EffectiveCharacteristics) -> bool {
    chars.abilities.iter().any(|a| a.is_characteristic_defining)
}

fn collect_modifications(
    chars: &EffectiveCharacteristics,
    layer: Layer,
) -> Vec<EffectModification> {
    let mut out = Vec::new();

    for ability in &chars.abilities {
        if !ability.is_characteristic_defining {
            continue;
        }
        debug_assert_eq!(
            ability.ability_type,
            AbilityType::Static,
            "CR 604.3: '{}' marks a non-static ability as characteristic-defining",
            chars.name
        );

        for (primitive, recipient) in atoms(ability) {
            // CR 604.3a(3) — a CDA affects only the object that has it, so the
            // recipient is always the object itself. A filter here would mean
            // the ability is not a CDA and was mis-flagged.
            debug_assert!(
                matches!(recipient, EffectRecipient::Implicit),
                "CR 604.3a(3): CDA on '{}' has a non-implicit recipient, so it \
                 affects other objects and is not characteristic-defining",
                chars.name
            );

            if cda_layer(primitive, &chars.name) == Some(layer) {
                push_modifications(primitive, &mut out, &chars.name);
            }
        }
    }

    out
}

/// Which layer a CDA's primitive belongs to, or `None` if the primitive cannot
/// be characteristic-defining at all (CR 604.3a(1)).
fn cda_layer(primitive: &Primitive, card_name: &str) -> Option<Layer> {
    match primitive {
        Primitive::ChangeColor(..) => Some(Layer::Layer5Color),
        Primitive::ChangeType(..) => Some(Layer::Layer4Type),
        Primitive::SetPowerToughness(..) => Some(Layer::Layer7aCdaPT),
        other => {
            debug_assert!(
                false,
                "CR 604.3a(1) — a CDA defines colors, subtypes, power or toughness \
                 and nothing else, but '{}' has a CDA carrying {:?}",
                card_name, other
            );
            None
        }
    }
}

fn push_modifications(primitive: &Primitive, out: &mut Vec<EffectModification>, card_name: &str) {
    match primitive {
        Primitive::ChangeColor(change, _) => out.push(match change {
            ColorChange::Add(c) => EffectModification::AddColor(*c),
            ColorChange::Set(colors) => EffectModification::SetColors(colors.clone()),
            ColorChange::RemoveAll => EffectModification::RemoveAllColors,
        }),

        Primitive::ChangeType(change, _) => {
            // CR 604.3a(1) lists subtypes, not card types or supertypes. A CDA
            // touching those is mis-flagged — Mistform Ultimus defines creature
            // types, it does not make itself a Land.
            debug_assert!(
                change.set_types.is_none()
                    && change.add_types.is_empty()
                    && change.remove_types.is_empty()
                    && change.set_supertypes.is_none()
                    && change.add_supertypes.is_empty()
                    && change.remove_supertypes.is_empty(),
                "CR 604.3a(1) — CDA on '{}' changes card types or supertypes; \
                 only subtypes are characteristic-defining",
                card_name
            );

            if let Some(ref set) = change.set_subtypes {
                out.push(EffectModification::SetSubtypes(set.clone()));
            } else {
                for s in &change.add_subtypes {
                    out.push(EffectModification::AddSubtype(s.clone()));
                }
                for s in &change.remove_subtypes {
                    out.push(EffectModification::RemoveSubtype(s.clone()));
                }
            }
        }

        // CR 613.4a. `power_expr`/`toughness_expr` are the amounts written on
        // the *ability* — Tarmogoyf's "the number of card types among cards in
        // all graveyards" — not the card's printed P/T box, which the walk has
        // already loaded into `chars` and which this is about to overwrite.
        //
        // `from_amount` keeps a literal amount as `PtValue::Fixed` and wraps
        // anything else as `Dynamic`, to be re-evaluated at every layer.
        Primitive::SetPowerToughness(power_expr, toughness_expr, _) => {
            out.push(EffectModification::SetPowerToughness {
                power: PtValue::from_amount(power_expr),
                toughness: PtValue::from_amount(toughness_expr),
            })
        }

        _ => {}
    }
}

/// Flatten an ability's effect into its atoms.
///
/// Mirrors `register_static_effects`: a static ability is declarative, so it is
/// either one `Atom` or a `Sequence` of them.
fn atoms(ability: &AbilityDef) -> Vec<(&Primitive, &EffectRecipient)> {
    match &ability.effect {
        Effect::Atom(p, r) => vec![(p, r)],
        Effect::Sequence(effects) => effects
            .iter()
            .filter_map(|e| match e {
                Effect::Atom(p, r) => Some((p, r)),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::layers::compute_characteristics;
    use crate::objects::card_data::{CardData, CardDataBuilder};
    use std::sync::Arc;
    use crate::types::card_types::CardType;
    use crate::types::colors::Color;
    use crate::types::effects::{AmountExpr, Duration};
    use crate::types::ids::new_ability_id;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::test_support::{put_on_battlefield_this_turn, registered};

    /// Every CDA test here uses player 0.
    fn put_on_battlefield(game: &mut GameState, data: Arc<CardData>) -> ObjectId {
        put_on_battlefield_this_turn(game, data, 0)
    }

    /// A CDA-flagged static ability carrying one atom about the object itself.
    fn cda(primitive: Primitive) -> AbilityDef {
        AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(primitive, EffectRecipient::Implicit),
            is_characteristic_defining: true,
        }
    }

    /// "This card is colorless" on a card whose mana cost is colored — CR
    /// 702.114a, Devoid. A Layer 5 CDA.
    fn devoid_card(name: &str) -> Arc<CardData> {
        CardDataBuilder::new(name)
            .card_type(CardType::Creature)
            .color(Color::Black)
            .mana_cost(ManaCost::build(&[ManaType::Black], 1))
            .power_toughness(2, 2)
            .ability(cda(Primitive::ChangeColor(
                ColorChange::RemoveAll,
                Duration::WhileSourceOnBattlefield,
            )))
            .build()
    }

    #[test]
    fn test_color_cda_applies_without_a_registry_row() {
        let mut game = GameState::new(2, 20);
        let id = put_on_battlefield(&mut game, devoid_card("Eldrazi Skyspawner"));

        // Printed black, but the CDA defines it colorless.
        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.colors.is_empty(), "Layer 5 CDA should have applied");
        assert_eq!(
            game.continuous_effects.len(),
            0,
            "a CDA affects only its own object (CR 604.3a(3)), so nothing registers"
        );
    }

    // COVERS-PARTIAL: ATOM-613.3-001
    //
    // Partial: the atom's CDA is "this creature's color is the color of the most
    // recent spell cast", which needs machinery this phase doesn't build. The
    // ordering claim is the same one and is what's under test — and the
    // timestamps are chosen so that timestamp order alone gives a different
    // answer.
    #[test]
    fn test_cda_applies_before_a_non_cda_with_an_earlier_timestamp() {
        let mut game = GameState::new(2, 20);
        let id = put_on_battlefield(&mut game, devoid_card("Eldrazi Skyspawner"));

        // "This creature is blue until end of turn" — a non-CDA Layer 5 effect
        // with an earlier timestamp than the CDA would have.
        let mut blue = std::collections::HashSet::new();
        blue.insert(Color::Blue);
        game.continuous_effects.add(registered(
            id,
            Layer::Layer5Color,
            1,
            EffectModification::SetColors(blue),
        ));

        // CR 613.3: the CDA goes first (colorless), then the non-CDA overrides
        // it (blue). Pure timestamp order would set blue first and let the CDA
        // wipe it back to colorless.
        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(
            chars.colors.iter().copied().collect::<Vec<_>>(),
            vec![Color::Blue],
            "CR 613.3: CDA first, then the non-CDA effect wins"
        );
    }

    // COVERS-PARTIAL: ATOM-113.12-002
    //
    // Partial: the layer-6 strip is a hand-built registry row rather than a real
    // "creatures lose all abilities" card, which needs a Layer 6 producer.
    #[test]
    fn test_layer_6_ability_strip_does_not_restore_a_cda_color() {
        let mut game = GameState::new(2, 20);
        let id = put_on_battlefield(&mut game, devoid_card("Eldrazi Skyspawner"));

        game.continuous_effects.add(registered(
            id,
            Layer::Layer6Ability,
            5,
            EffectModification::LoseAllAbilities,
        ));

        // Layer 5 runs before Layer 6, so the color was already set when the
        // ability was removed. Removing the CDA afterwards cannot un-apply it.
        let chars = compute_characteristics(&game, id).unwrap();
        assert!(
            chars.colors.is_empty(),
            "CR 113.12: the characteristic is set, not granted — losing the \
             ability later does not restore the printed color"
        );
        assert!(chars.abilities.is_empty(), "the ability itself is gone");
    }

    #[test]
    fn test_pt_cda_applies_in_layer_7a_and_7b_overrides_it() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Defined Beast")
            .card_type(CardType::Creature)
            .power_toughness(0, 0)
            .ability(cda(Primitive::SetPowerToughness(
                AmountExpr::Fixed(4),
                AmountExpr::Fixed(4),
                Duration::WhileSourceOnBattlefield,
            )))
            .build();
        let id = put_on_battlefield(&mut game, data);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!((chars.power, chars.toughness), (Some(4), Some(4)));

        // CR 613.4a/b — 7b applies after 7a, so a set-P/T effect wins over the
        // CDA no matter how the timestamps fall.
        game.continuous_effects.add(registered(
            id,
            Layer::Layer7bSetPT,
            1,
            EffectModification::SetPowerToughness {
                power: PtValue::Fixed(1),
                toughness: PtValue::Fixed(1),
            },
        ));
        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!((chars.power, chars.toughness), (Some(1), Some(1)));
    }

    #[test]
    fn test_layer_6_ability_strip_removes_a_pt_cda() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Defined Beast")
            .card_type(CardType::Creature)
            .power_toughness(0, 0)
            .ability(cda(Primitive::SetPowerToughness(
                AmountExpr::Fixed(4),
                AmountExpr::Fixed(4),
                Duration::WhileSourceOnBattlefield,
            )))
            .build();
        let id = put_on_battlefield(&mut game, data);

        game.continuous_effects.add(registered(
            id,
            Layer::Layer6Ability,
            5,
            EffectModification::LoseAllAbilities,
        ));

        // The mirror image of the color case: Layer 6 runs before 7a, so the
        // CDA is gone before it can define anything. This is exactly why
        // ATOM-113.12-001's original expected result was wrong.
        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(
            (chars.power, chars.toughness),
            (Some(0), Some(0)),
            "CR 613.4a: 7a follows 6, so an ability strip removes the CDA first"
        );
    }
}
