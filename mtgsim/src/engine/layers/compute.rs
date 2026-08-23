//! Core computation: `compute_characteristics(game, id)`.
//!
//! Walks the continuous effect registry in layer order and produces
//! `EffectiveCharacteristics` for a given object. All oracle queries
//! route through this function.
//!
//! Reads base characteristics from CardData, then applies all continuous
//! effects in layer order (1→2→3→4→5→6→7b→7c→7d).

use std::collections::HashMap;

use crate::engine::layers::types::*;
use crate::state::game_state::GameState;
use crate::types::effects::CounterType;
use crate::types::ids::ObjectId;

/// The layers, in application order (CR 613.1). Index into this array is the
/// "layer ceiling" used by the frame cache: ceiling `n` means layers
/// `LAYER_ORDER[..n]` have been applied, i.e. the frame as of the end of
/// layer `n - 1`.
///
/// This mirrors the `Layer` enum exactly, and the enum is **not** the CR's full
/// list. One sublayer split is still missing, tracked as Deferred Migrations
/// item 10:
///
/// - **1a / 1b.** CR 613.2 splits layer 1 into face-down effects (1a) and copy
///   effects (1b); `Layer1Copy` collapses them, even though
///   `layers-architecture.md` §7 specifies both and says Phase LA ships them.
///   The order matters — a Clone copying a face-down creature must copy the
///   2/2 colorless characteristics, not the printed card (CR 707.2).
///
/// Not reachable today: nothing produces a layer 1 effect. Splitting it later
/// just lengthens this array — the ceiling is an index into it, computed at
/// runtime, so nothing else moves.
const LAYER_ORDER: [Layer; 10] = [
    Layer::Layer1Copy,
    Layer::Layer2Control,
    Layer::Layer3Text,
    Layer::Layer4Type,
    Layer::Layer5Color,
    Layer::Layer6Ability,
    Layer::Layer7aCdaPT,
    Layer::Layer7bSetPT,
    Layer::Layer7cModifyPT,
    Layer::Layer7dSwitchPT,
];

/// Memo for one top-level `compute_characteristics` call
/// (`layers-architecture.md` §5.2).
///
/// Deciding whether a CR 613.7a effect still exists means asking whether some
/// *other* object still has the static ability that generates it, which is a
/// characteristics query of its own. §5.2's answer is to answer it at a lower
/// **layer ceiling**: at layer index `i` we need the source's frame as of the
/// end of layer `i - 1`, which is ceiling `i`.
///
/// That is also the termination argument. Computing an object at ceiling `C`
/// only ever requests ceilings `< C`, so the recursion strictly descends and
/// bottoms out at ceiling 0, which applies no effects at all. There is no
/// fixpoint to iterate and nothing to cap.
///
/// Discarded when the top-level call returns, so it never has to be
/// invalidated.
pub(super) type FrameCache = HashMap<(ObjectId, usize), EffectiveCharacteristics>;

/// Compute the effective characteristics of a game object after applying
/// all active continuous effects in layer order.
///
/// Returns `None` if the object doesn't exist.
pub fn compute_characteristics(game: &GameState, id: ObjectId) -> Option<EffectiveCharacteristics> {
    let mut cache = FrameCache::new();
    compute_to_ceiling(game, id, LAYER_ORDER.len(), &mut cache)
}

/// `compute_characteristics` with layers `LAYER_ORDER[ceiling..]` left unapplied.
pub(super) fn compute_to_ceiling(
    game: &GameState,
    id: ObjectId,
    ceiling: usize,
    cache: &mut FrameCache,
) -> Option<EffectiveCharacteristics> {
    // Only sub-computations are worth memoizing. The top-level frame is
    // requested exactly once per call, so caching it would be a pure clone.
    let memoize = ceiling < LAYER_ORDER.len();
    if memoize {
        if let Some(cached) = cache.get(&(id, ceiling)) {
            return Some(cached.clone());
        }
    }

    let obj = game.objects.get(&id)?;
    let card = &obj.card_data;

    // Start from printed (base) characteristics
    let mut chars = EffectiveCharacteristics {
        name: card.name.clone(),
        mana_cost: card.mana_cost.clone(),
        colors: card.colors.clone(),
        types: card.types.clone(),
        subtypes: card.subtypes.clone(),
        supertypes: card.supertypes.clone(),
        keywords: card.keywords.clone(),
        abilities: card.abilities.clone(),
        power: card.power,
        toughness: card.toughness,
        controller: obj.owner, // default; overridden by battlefield entry or L2 effects
    };

    // If on the battlefield, use the actual controller from BattlefieldEntity
    if let Some(entry) = game.battlefield.get(&id) {
        chars.controller = entry.controller;
    }

    // Walk layers in order, applying all effects (registered + counters)
    apply_effects(game, id, &mut chars, ceiling, cache);

    if memoize {
        cache.insert((id, ceiling), chars.clone());
    }
    Some(chars)
}

/// CR 613.7a — does the static ability that generates `effect` still exist?
///
/// A continuous effect from a static ability applies only while its source
/// actually has that ability. Registry membership does not answer this: the
/// effect is registered when the permanent enters the battlefield, but CR 305.7
/// (Blood Moon) and Layer 6 ability removal can take the ability away later
/// without touching the registry. So the question is re-asked at every layer,
/// against the source's frame as of the end of the previous layer.
///
/// `EffectOrigin::Resolution` effects (CR 613.7b) always exist: a resolution
/// already happened and cannot be taken back, so there is no ability to go
/// looking for.
///
/// Existence is not the same as surviving, and only existence is decided here.
/// An instant that grants first strike until end of turn creates an effect that
/// exists for the turn no matter what — but Humility, applying later in layer 6,
/// still clears the keyword it granted. That is ordering inside a layer, which
/// `effects_in_layer` handles (timestamp today, CR 613.8 eventually), not a
/// question about whether the effect is there to apply.
fn static_ability_still_exists(
    game: &GameState,
    effect: &ContinuousEffect,
    layer_index: usize,
    cache: &mut FrameCache,
) -> bool {
    let ability_id = match effect.origin {
        EffectOrigin::Resolution => return true,
        EffectOrigin::StaticAbility { ability } => ability,
    };

    match compute_to_ceiling(game, effect.source, layer_index, cache) {
        Some(source_frame) => source_frame.abilities.iter().any(|a| a.id == ability_id),
        // Source is gone from the object store entirely.
        None => false,
    }
}

/// Apply all continuous effects in layer order (rule 613).
///
/// Walks registered effects by layer, and also applies counter-derived
/// P/T modifications in layer 7c (rule 613.4c) alongside other modifiers.
fn apply_effects(
    game: &GameState,
    id: ObjectId,
    chars: &mut EffectiveCharacteristics,
    ceiling: usize,
    cache: &mut FrameCache,
) {
    let has_registered = !game.continuous_effects.is_empty();
    let on_battlefield = game.battlefield.contains_key(&id);

    // CR 613.6 — "if an effect starts to apply in one layer, it will continue
    // to be applied to the same set of objects in each other applicable layer".
    //
    // `effect_applies_to` reads `chars`, which earlier layers have already
    // mutated, so re-filtering from scratch at every layer is wrong: March of
    // the Machines' Layer 4 part makes a noncreature artifact a creature, and
    // its Layer 7b part then finds nothing matching "noncreature artifact".
    // Once a CR-level effect has started applying to this object, membership
    // here short-circuits the filter for the rest of the walk.
    //
    // Keyed by `EffectGroup`, not `EffectId`: the two halves of March of the
    // Machines are two registry rows, and it is the *effect* that started
    // applying, not the row.
    let mut started: std::collections::HashSet<EffectGroup> = std::collections::HashSet::new();

    // ...but only when some CR-level effect actually occupies more than one
    // row. For a single-row group the mark is written after its only row
    // applies and never read, so maintaining it is pure cost — two SipHashes
    // over a pair of UUIDs, per effect, per layer. That was 70% of the layer
    // walk on a static-heavy board before this check existed.
    let track_started = game.continuous_effects.summary().any_multi_row_group;

    // Fast path: nothing to apply.
    //
    // A CDA is not in the registry and does not need the battlefield (CR 604.3
    // — CDAs function in all zones), so it gets its own term. Reading the
    // printed list is exact here: with an empty registry and no battlefield
    // entry, nothing in the walk can add an ability.
    if !has_registered && !on_battlefield && !crate::engine::layers::cda::has_any_cda(chars) {
        return;
    }

    for (layer_index, &layer) in LAYER_ORDER.iter().enumerate() {
        if layer_index >= ceiling {
            break;
        }

        // CR 613.3 — "apply effects from characteristic-defining abilities
        // first, then all other effects in timestamp order". Intrinsic before
        // the registry slice is that sentence. Only three layers can hold a
        // CDA (CR 604.3a(1)), so the other seven skip the scan entirely.
        if crate::engine::layers::cda::CDA_LAYERS.contains(&layer) {
            crate::engine::layers::cda::apply_intrinsic_cdas(
                game, chars, id, layer, layer_index, cache,
            );
        }

        // Apply registered effects in this layer
        if has_registered {
            let effects = game.continuous_effects.effects_in_layer(layer);
            for effect in effects {
                let already_applying = track_started && started.contains(&effect.group());
                if !already_applying {
                    if !effect_applies_to(effect, id, chars, game) {
                        continue;
                    }
                    // CR 613.7a. Only asked before the effect starts applying:
                    // once it has, CR 613.6 keeps it applying for the rest of
                    // this walk even if a later layer removes the ability.
                    if !static_ability_still_exists(game, effect, layer_index, cache) {
                        continue;
                    }
                    if track_started {
                        started.insert(effect.group());
                    }
                }
                apply_modification(&effect.modification, chars, id, game, layer_index, cache);
            }
        }

        // Apply counter P/T in layer 7c (rule 613.4c)
        if layer == Layer::Layer7cModifyPT {
            if let Some(entry) = game.battlefield.get(&id) {
                let plus = entry.counter_count(CounterType::PlusOnePlusOne) as i32;
                if plus != 0 {
                    if let Some(ref mut p) = chars.power { *p += plus; }
                    if let Some(ref mut t) = chars.toughness { *t += plus; }
                }
                let minus = entry.counter_count(CounterType::MinusOneMinusOne) as i32;
                if minus != 0 {
                    if let Some(ref mut p) = chars.power { *p -= minus; }
                    if let Some(ref mut t) = chars.toughness { *t -= minus; }
                }
                // TODO: handle other P/T-modifying counter types (+2/+2, +0/+1, etc.)
                // when they are added to CounterType.
            }
        }
    }
}

/// Check whether a continuous effect applies to the given object.
fn effect_applies_to(
    effect: &ContinuousEffect,
    id: ObjectId,
    chars: &EffectiveCharacteristics,
    game: &GameState,
) -> bool {
    match &effect.affected {
        AffectedSet::SourceOnly => effect.source == id,
        AffectedSet::Fixed(ids) => ids.contains(&id),
        AffectedSet::Filter { filter, controller } => {
            // Object must be on the battlefield for filter-based effects
            if !game.battlefield.contains_key(&id) {
                return false;
            }
            // Check controller constraint
            if let Some(ctrl) = controller {
                if chars.controller != *ctrl {
                    return false;
                }
            }
            // Check the permanent filter against current characteristics
            permanent_matches_filter(filter, chars)
        }
    }
}

/// Check if a permanent's current characteristics match a filter.
fn permanent_matches_filter(
    filter: &crate::types::effects::PermanentFilter,
    chars: &EffectiveCharacteristics,
) -> bool {
    use crate::types::effects::PermanentFilter;
    match filter {
        PermanentFilter::All => true,
        PermanentFilter::ByType(t) => chars.types.contains(t),
        PermanentFilter::BySubtype(s) => chars.subtypes.contains(s),
        PermanentFilter::BySupertype(s) => chars.supertypes.contains(s),
        PermanentFilter::ByColor(c) => chars.colors.contains(c),
        PermanentFilter::ByController(_) => {
            // Controller filtering is handled by the AffectedSet::Filter.controller field
            true
        }
        PermanentFilter::PowerLE(n) => {
            chars.power.map(|p| p <= *n).unwrap_or(false)
        }
        PermanentFilter::And(a, b) => {
            permanent_matches_filter(a, chars) && permanent_matches_filter(b, chars)
        }
        PermanentFilter::Not(inner) => !permanent_matches_filter(inner, chars),
    }
}

/// Resolve one side of a P/T modification against the frame so far.
///
/// `None` means the expression has no meaning in a static context — `Variable`
/// is CR 107.3's X, chosen as a spell is cast, and the `Target*`/`DamageDealt`
/// arms read a resolution that already happened. A continuous effect asking for
/// one of those is a card-authoring error, so it asserts in debug and declines
/// to apply in release rather than inventing a number.
///
/// Evaluated fresh at every layer: that is the point of `PtValue::Dynamic`.
pub(super) fn evaluate_pt_value(
    value: &PtValue,
    game: &GameState,
    chars: &EffectiveCharacteristics,
    layer_index: usize,
    cache: &mut FrameCache,
) -> Option<i32> {
    match value {
        PtValue::Fixed(n) => Some(*n),
        PtValue::Dynamic(expr) => evaluate_amount(expr, game, chars, layer_index, cache),
    }
}

/// Evaluate a card-definition amount inside the layer walk.
///
/// Anything reading *another* object goes through `compute_to_ceiling` at the
/// current `layer_index`, never through `card_data`. Both halves of that matter:
/// it keeps the layer-system invariant (effective characteristics, not printed
/// ones), and it preserves §5.2's termination argument — a request at ceiling
/// `layer_index` is strictly below the ceiling of the walk that made it, so the
/// recursion descends. A Tarmogoyf in a graveyard counting itself bottoms out
/// for exactly that reason.
fn evaluate_amount(
    expr: &crate::types::effects::AmountExpr,
    game: &GameState,
    chars: &EffectiveCharacteristics,
    layer_index: usize,
    cache: &mut FrameCache,
) -> Option<i32> {
    use crate::types::effects::{AmountExpr, Selector};

    match expr {
        AmountExpr::Fixed(n) => Some(*n as i32),

        // CR 202.3b — an object with no mana cost has mana value 0.
        AmountExpr::AffectedManaValue => {
            Some(chars.mana_cost.as_ref().map(|c| c.mana_value()).unwrap_or(0) as i32)
        }

        AmountExpr::Plus(inner, n) => {
            evaluate_amount(inner, game, chars, layer_index, cache).map(|v| v + *n as i32)
        }

        // Card *types*, not cards: ten artifact creatures in a graveyard are
        // still two types.
        AmountExpr::CardTypesAmong(selector) => match selector {
            Selector::CardsInGraveyard(None) => {
                let mut types: std::collections::HashSet<crate::types::card_types::CardType> =
                    std::collections::HashSet::new();
                for player in &game.players {
                    for card_id in &player.graveyard {
                        if let Some(card) = compute_to_ceiling(game, *card_id, layer_index, cache) {
                            types.extend(card.types.iter().copied());
                        }
                    }
                }
                Some(types.len() as i32)
            }
            other => {
                debug_assert!(
                    false,
                    "CardTypesAmong({:?}) has no evaluator yet (on '{}')",
                    other, chars.name
                );
                None
            }
        },

        // CR 107.3's X is chosen as a spell is cast, and the Target*/DamageDealt
        // arms read a resolution that already happened. A continuous effect
        // asking for one of those is a card-authoring error: assert in debug,
        // decline to apply in release, never invent a number.
        other => {
            debug_assert!(
                false,
                "continuous effect on '{}' carries {:?}, which has no static-context evaluator",
                chars.name, other
            );
            None
        }
    }
}

/// Apply a single effect modification to the characteristics frame.
///
/// `object_id` is the object being computed. Layer 4's subtype arms need it to
/// derive stable ids for intrinsic mana abilities (CR 305.6) — see
/// `land_types::intrinsic_mana_ability`.
pub(super) fn apply_modification(
    modification: &EffectModification,
    chars: &mut EffectiveCharacteristics,
    object_id: ObjectId,
    game: &GameState,
    layer_index: usize,
    cache: &mut FrameCache,
) {
    match modification {
        // Layer 2
        EffectModification::SetController(pid) => {
            chars.controller = *pid;
        }

        // Layer 4
        EffectModification::AddType(t) => { chars.types.insert(*t); }
        EffectModification::RemoveType(t) => { chars.types.remove(t); }
        EffectModification::SetTypes(types) => { chars.types = types.clone(); }
        // CR 305.6/305.7 land semantics live in `land_types` — see the module
        // docs there for why this is not a Layer 6 concern.
        EffectModification::AddSubtype(s) => {
            crate::engine::layers::land_types::apply_add_subtype(chars, s, object_id);
        }
        EffectModification::RemoveSubtype(s) => { chars.subtypes.remove(s); }
        EffectModification::SetSubtypes(subtypes) => {
            crate::engine::layers::land_types::apply_set_subtypes(chars, subtypes, object_id);
        }
        EffectModification::AddSupertype(s) => { chars.supertypes.insert(*s); }
        EffectModification::RemoveSupertype(s) => { chars.supertypes.remove(s); }
        EffectModification::SetSupertypes(supertypes) => { chars.supertypes = supertypes.clone(); }

        // Layer 5
        EffectModification::AddColor(c) => { chars.colors.insert(*c); }
        EffectModification::SetColors(colors) => { chars.colors = colors.clone(); }
        EffectModification::RemoveAllColors => { chars.colors.clear(); }

        // Layer 6
        EffectModification::GrantKeyword(kw) => { chars.keywords.insert(*kw); }
        // CR 113.10b — "effects that remove an ability remove all instances of
        // it". For a keyword flag that is structural: a `HashSet` never held
        // more than one.
        EffectModification::RemoveKeyword(kw) => { chars.keywords.remove(kw); }
        EffectModification::GrantAbility(def) => {
            // CR 604.3a(2) — an ability that reached an object by being granted
            // is never a characteristic-defining ability, however its text
            // reads. The flag on `AbilityDef` asserts only the four criteria
            // that are properties of the text; provenance is maintained by
            // whoever writes the ability onto an object, and this is that
            // place. Copy (Layer 1) and text-changing (Layer 3) effects hand
            // the def over whole and keep the flag, which is the *other* half
            // of 604.3a(2) and equally deliberate.
            //
            // Not clearing it would let a granted Tarmogoyf ability define P/T
            // at Layer 7a, which is exactly what 604.3a(2) forbids.
            let mut granted = (**def).clone();
            granted.is_characteristic_defining = false;
            chars.abilities.push(granted);
        }
        // CR 113.10b again, and here it is *not* structural: `abilities` is a
        // `Vec` and the same ability can genuinely appear twice — printed on
        // the card and granted on top of it. `retain`, never "remove the first
        // match".
        EffectModification::LoseAbility(ability_id) => {
            chars.abilities.retain(|a| a.id != *ability_id);
        }
        EffectModification::LoseAllAbilities => {
            chars.keywords.clear();
            chars.abilities.clear();
        }

        // Layer 7b
        EffectModification::SetPowerToughness { power, toughness } => {
            // Evaluated before mutating: `AffectedManaValue` reads `chars`, and
            // setting power first would let it observe a half-applied frame.
            let p = evaluate_pt_value(power, game, chars, layer_index, cache);
            let t = evaluate_pt_value(toughness, game, chars, layer_index, cache);
            if let (Some(p), Some(t)) = (p, t) {
                chars.power = Some(p);
                chars.toughness = Some(t);
            }
        }

        // Layer 7c
        EffectModification::ModifyPowerToughness { power, toughness } => {
            let dp = evaluate_pt_value(power, game, chars, layer_index, cache);
            let dt = evaluate_pt_value(toughness, game, chars, layer_index, cache);
            if let Some(dp) = dp {
                if let Some(ref mut p) = chars.power {
                    *p += dp;
                }
            }
            if let Some(dt) = dt {
                if let Some(ref mut t) = chars.toughness {
                    *t += dt;
                }
            }
        }

        // Layer 7d
        EffectModification::SwitchPowerToughness => {
            let old_power = chars.power;
            chars.power = chars.toughness;
            chars.toughness = old_power;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::types::card_types::CardType;
    use crate::types::colors::Color;
    use crate::types::keywords::KeywordFlag;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;
    use crate::test_support::registered;

    #[test]
    fn test_base_characteristics_from_card_data() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .mana_cost(ManaCost::build(&[ManaType::Green], 1))
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.name, "Grizzly Bears");
        assert_eq!(chars.power, Some(2));
        assert_eq!(chars.toughness, Some(2));
        assert!(chars.types.contains(&CardType::Creature));
        assert!(chars.colors.contains(&Color::Green));
        assert_eq!(chars.controller, 0);
    }

    // COVERS-PARTIAL: ATOM-613.4c-001
    #[test]
    fn test_registered_effect_pump_power_only() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register a +3/+0 effect
        let effect = registered(
            id,
            Layer::Layer7cModifyPT,
            1,
            EffectModification::ModifyPowerToughness { power: PtValue::Fixed(3), toughness: PtValue::Fixed(0) },
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.power, Some(5));
        assert_eq!(chars.toughness, Some(2));
    }

    // COVERS: ATOM-122.1a-001
    #[test]
    fn test_counters_modify_pt() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = game.place_on_battlefield(id, 0);
        entry.add_counters(CounterType::PlusOnePlusOne, 2);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.power, Some(4));
        assert_eq!(chars.toughness, Some(4));
    }

    // COVERS-PARTIAL: ATOM-122.1a-002
    #[test]
    fn test_counters_plus_and_minus() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Big Creature")
            .card_type(CardType::Creature)
            .power_toughness(5, 5)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = game.place_on_battlefield(id, 0);
        entry.add_counters(CounterType::PlusOnePlusOne, 3);
        entry.add_counters(CounterType::MinusOneMinusOne, 1);

        let chars = compute_characteristics(&game, id).unwrap();
        // Net: +3 -1 = +2
        assert_eq!(chars.power, Some(7));
        assert_eq!(chars.toughness, Some(7));
    }

    #[test]
    fn test_nonexistent_object_returns_none() {
        let game = GameState::new(2, 20);
        let fake_id = crate::types::ids::new_object_id();
        assert!(compute_characteristics(&game, fake_id).is_none());
    }

    #[test]
    fn test_non_battlefield_object_no_modifiers() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .color(Color::Red)
            .build();
        let obj = GameObject::new(data, 0, Zone::Hand);
        let id = obj.id;
        game.add_object(obj);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.name, "Lightning Bolt");
        assert!(chars.types.contains(&CardType::Instant));
        assert!(chars.colors.contains(&Color::Red));
        // Not on battlefield, so controller defaults to owner
        assert_eq!(chars.controller, 0);
    }

    #[test]
    fn test_keywords_from_card_data() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Serra Angel")
            .card_type(CardType::Creature)
            .power_toughness(4, 4)
            .keyword(KeywordFlag::Flying)
            .keyword(KeywordFlag::Vigilance)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.keywords.contains(&KeywordFlag::Flying));
        assert!(chars.keywords.contains(&KeywordFlag::Vigilance));
        assert!(!chars.keywords.contains(&KeywordFlag::Trample));
    }

    // COVERS-PARTIAL: ATOM-613.4c-001
    #[test]
    fn test_registered_effect_modifies_pt() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register a +3/+3 effect targeting this creature
        let effect = registered(
            id,
            Layer::Layer7cModifyPT,
            1,
            EffectModification::ModifyPowerToughness { power: PtValue::Fixed(3), toughness: PtValue::Fixed(3) },
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert_eq!(chars.power, Some(5));
        assert_eq!(chars.toughness, Some(5));
    }

    #[test]
    fn test_registered_effect_grants_keyword() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register a "gains flying" effect
        let effect = registered(
            id,
            Layer::Layer6Ability,
            1,
            EffectModification::GrantKeyword(KeywordFlag::Flying),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.keywords.contains(&KeywordFlag::Flying));
    }

    #[test]
    fn test_filter_based_effect() {
        use crate::types::effects::{Duration, PermanentFilter};

        let mut game = GameState::new(2, 20);

        // Two creatures controlled by player 0
        let bears_data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let bears = GameObject::new(bears_data, 0, Zone::Battlefield);
        let bears_id = bears.id;
        game.add_object(bears);
        game.place_on_battlefield(bears_id, 0);

        let giant_data = CardDataBuilder::new("Hill Giant")
            .card_type(CardType::Creature)
            .power_toughness(3, 3)
            .build();
        let giant = GameObject::new(giant_data, 0, Zone::Battlefield);
        let giant_id = giant.id;
        game.add_object(giant);
        game.place_on_battlefield(giant_id, 0);

        // Register an anthem: "Creatures you control get +1/+1"
        let anthem_source = crate::types::ids::new_object_id();
        let effect = ContinuousEffect {
            id: 0,
            source: anthem_source,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer7cModifyPT,
            duration: Duration::WhileSourceOnBattlefield,
            controller: 0,
            created_on_turn: 1,
            timestamp: 1,
            affected: AffectedSet::Filter {
                filter: PermanentFilter::ByType(CardType::Creature),
                controller: Some(0),
            },
            modification: EffectModification::ModifyPowerToughness { power: PtValue::Fixed(1), toughness: PtValue::Fixed(1) },
        };
        game.continuous_effects.add(effect);

        let bears_chars = compute_characteristics(&game, bears_id).unwrap();
        assert_eq!(bears_chars.power, Some(3));
        assert_eq!(bears_chars.toughness, Some(3));

        let giant_chars = compute_characteristics(&game, giant_id).unwrap();
        assert_eq!(giant_chars.power, Some(4));
        assert_eq!(giant_chars.toughness, Some(4));
    }

    #[test]
    fn test_set_colors_replaces_base_colors() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register a "becomes blue" effect (SetColors)
        let mut blue = std::collections::HashSet::new();
        blue.insert(Color::Blue);
        let effect = registered(
            id,
            Layer::Layer5Color,
            game.allocate_timestamp(),
            EffectModification::SetColors(blue),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.colors.contains(&Color::Blue));
        assert!(!chars.colors.contains(&Color::Green));
        assert_eq!(chars.colors.len(), 1);
    }

    #[test]
    fn test_add_color_preserves_existing() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register an "also red" effect (AddColor)
        let effect = registered(
            id,
            Layer::Layer5Color,
            game.allocate_timestamp(),
            EffectModification::AddColor(Color::Red),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.colors.contains(&Color::Green));
        assert!(chars.colors.contains(&Color::Red));
        assert_eq!(chars.colors.len(), 2);
    }

    #[test]
    fn test_remove_all_colors_makes_colorless() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register a "becomes colorless" effect
        let effect = registered(
            id,
            Layer::Layer5Color,
            game.allocate_timestamp(),
            EffectModification::RemoveAllColors,
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.colors.is_empty());
    }

    #[test]
    fn test_color_change_independent_of_pt() {

        // Color change (L5) should not affect P/T (L7) and vice versa
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // L5: becomes blue
        let mut blue = std::collections::HashSet::new();
        blue.insert(Color::Blue);
        let color_effect = registered(
            id,
            Layer::Layer5Color,
            game.allocate_timestamp(),
            EffectModification::SetColors(blue),
        );
        game.continuous_effects.add(color_effect);

        // L7c: +3/+3
        let pt_effect = registered(
            id,
            Layer::Layer7cModifyPT,
            game.allocate_timestamp(),
            EffectModification::ModifyPowerToughness { power: PtValue::Fixed(3), toughness: PtValue::Fixed(3) },
        );
        game.continuous_effects.add(pt_effect);

        let chars = compute_characteristics(&game, id).unwrap();
        // Color should be blue (not green)
        assert!(chars.colors.contains(&Color::Blue));
        assert!(!chars.colors.contains(&Color::Green));
        // P/T should be 5/5 (2+3)
        assert_eq!(chars.power, Some(5));
        assert_eq!(chars.toughness, Some(5));
    }

    #[test]
    fn test_filter_based_color_effect() {
        use crate::types::effects::{Duration, PermanentFilter};

        // Static ability: "Creatures you control are also red"
        let mut game = GameState::new(2, 20);

        let bears_data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .power_toughness(2, 2)
            .build();
        let bears = GameObject::new(bears_data, 0, Zone::Battlefield);
        let bears_id = bears.id;
        game.add_object(bears);
        game.place_on_battlefield(bears_id, 0);

        // Opponent's creature should NOT be affected
        let opp_data = CardDataBuilder::new("Savannah Lions")
            .card_type(CardType::Creature)
            .color(Color::White)
            .power_toughness(2, 1)
            .build();
        let opp = GameObject::new(opp_data, 1, Zone::Battlefield);
        let opp_id = opp.id;
        game.add_object(opp);
        game.place_on_battlefield(opp_id, 1);

        let source_id = crate::types::ids::new_object_id();
        let effect = ContinuousEffect {
            id: 0,
            source: source_id,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer5Color,
            duration: Duration::WhileSourceOnBattlefield,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Filter {
                filter: PermanentFilter::ByType(CardType::Creature),
                controller: Some(0),
            },
            modification: EffectModification::AddColor(Color::Red),
        };
        game.continuous_effects.add(effect);

        let bears_chars = compute_characteristics(&game, bears_id).unwrap();
        assert!(bears_chars.colors.contains(&Color::Green));
        assert!(bears_chars.colors.contains(&Color::Red));

        let opp_chars = compute_characteristics(&game, opp_id).unwrap();
        assert!(opp_chars.colors.contains(&Color::White));
        assert!(!opp_chars.colors.contains(&Color::Red));
    }

    // COVERS-PARTIAL: ATOM-613.4d-004
    #[test]
    fn test_counters_applied_before_switch_pt() {

        // Regression test: counters are in 7c, switch is 7d.
        // A 1/4 creature with two +1/+1 counters and a switch effect:
        // 7c: 1+2=3 / 4+2=6, then 7d: swap → 6/3
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Wall")
            .card_type(CardType::Creature)
            .power_toughness(1, 4)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = game.place_on_battlefield(id, 0);
        entry.add_counters(CounterType::PlusOnePlusOne, 2);

        // Register a switch P/T effect (layer 7d)
        let effect = registered(
            id,
            Layer::Layer7dSwitchPT,
            1,
            EffectModification::SwitchPowerToughness,
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        // 7c: 1+2=3, 4+2=6; 7d: swap → 6/3
        assert_eq!(chars.power, Some(6));
        assert_eq!(chars.toughness, Some(3));
    }

    // === Layer 4 type-changing tests ===

    // COVERS-PARTIAL: ATOM-205.1b-004
    #[test]
    fn test_add_type_preserves_existing() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Darksteel Ingot")
            .card_type(CardType::Artifact)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Register "becomes also a creature" effect
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::AddType(CardType::Creature),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.types.contains(&CardType::Artifact));
        assert!(chars.types.contains(&CardType::Creature));
    }

    #[test]
    fn test_remove_type() {

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Mycosynth Lattice")
            .card_type(CardType::Artifact)
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Remove Creature type
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::RemoveType(CardType::Creature),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.types.contains(&CardType::Artifact));
        assert!(!chars.types.contains(&CardType::Creature));
    }

    // COVERS-PARTIAL: ATOM-205.1a-003
    #[test]
    fn test_set_subtypes_replaces_all() {
        use crate::types::card_types::{LandType, Subtype};
        use std::collections::HashSet;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Steam Vents")
            .card_type(CardType::Land)
            .subtype(Subtype::Land(LandType::Island))
            .subtype(Subtype::Land(LandType::Mountain))
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // SetSubtypes to just Forest
        let mut forest_set = HashSet::new();
        forest_set.insert(Subtype::Land(LandType::Forest));
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::SetSubtypes(forest_set),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.subtypes.contains(&Subtype::Land(LandType::Forest)));
        assert!(!chars.subtypes.contains(&Subtype::Land(LandType::Island)));
        assert!(!chars.subtypes.contains(&Subtype::Land(LandType::Mountain)));
        assert_eq!(chars.subtypes.len(), 1);
    }

    #[test]
    fn test_add_subtype_preserves_existing() {
        use crate::types::card_types::{LandType, Subtype};

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Mountain")
            .card_type(CardType::Land)
            .subtype(Subtype::Land(LandType::Mountain))
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Add Swamp subtype ("in addition to")
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::AddSubtype(Subtype::Land(LandType::Swamp)),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.subtypes.contains(&Subtype::Land(LandType::Mountain)));
        assert!(chars.subtypes.contains(&Subtype::Land(LandType::Swamp)));
        assert_eq!(chars.subtypes.len(), 2);
    }

    #[test]
    fn test_add_supertype() {
        use crate::types::card_types::Supertype;

        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // Add Legendary supertype
        let effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::AddSupertype(Supertype::Legendary),
        );
        game.continuous_effects.add(effect);

        let chars = compute_characteristics(&game, id).unwrap();
        assert!(chars.supertypes.contains(&Supertype::Legendary));
    }

    // COVERS-PARTIAL: ATOM-613.1d-001
    #[test]
    fn test_type_change_before_color_change() {
        use crate::types::effects::{Duration, PermanentFilter};

        // Test layer ordering: L4 (type) applies before L5 (color)
        // A filter-based color effect that checks types should see the
        // type as it stands after L4.
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Darksteel Ingot")
            .card_type(CardType::Artifact)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        // L4: Add Creature type
        let l4_effect = registered(
            id,
            Layer::Layer4Type,
            game.allocate_timestamp(),
            EffectModification::AddType(CardType::Creature),
        );
        game.continuous_effects.add(l4_effect);

        // L5: "Creatures are also red" (filter-based)
        let l5_source = crate::types::ids::new_object_id();
        let l5_effect = ContinuousEffect {
            id: 0,
            source: l5_source,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer5Color,
            duration: Duration::WhileSourceOnBattlefield,
            controller: 0,
            created_on_turn: 1,
            timestamp: game.allocate_timestamp(),
            affected: AffectedSet::Filter {
                filter: PermanentFilter::ByType(CardType::Creature),
                controller: None,
            },
            modification: EffectModification::AddColor(Color::Red),
        };
        game.continuous_effects.add(l5_effect);

        let chars = compute_characteristics(&game, id).unwrap();
        // Should be both Artifact and Creature (L4 applied)
        assert!(chars.types.contains(&CardType::Artifact));
        assert!(chars.types.contains(&CardType::Creature));
        // Should be Red (L5 filter sees the L4-modified type = Creature)
        assert!(chars.colors.contains(&Color::Red));
    }
}
