// Mana helper queries — shared utilities for finding lands to tap and
// determining which spells a player can afford to cast.
//
// Used by CLI (show affordable spells), Random DP (auto-tap), and future AI.
// All functions are read-only queries over &GameState.

use crate::objects::card_data::AbilityType;
use crate::state::game_state::GameState;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};

/// A mana source: a permanent with a mana ability that can currently be activated.
///
/// Note: mana abilities don't always require tapping (rule 605.1a/605.1b).
/// A tapped creature with "Sacrifice this creature: Add {U}{R}" is a valid
/// mana source. We check each ability's specific costs, not just tap state.
#[derive(Debug, Clone)]
pub struct ManaSource {
    pub permanent_id: ObjectId,
    pub ability_id: AbilityId,
    pub produces: ManaType,
}

/// Find a set of mana sources (lands, mana rocks, mana dorks, etc.) whose
/// mana abilities can pay a given mana cost.
///
/// Uses a greedy algorithm:
/// 1. Identify all available mana sources controlled by the player.
/// 2. Reserve sources that produce colors needed for specific (colored) requirements.
/// 3. Assign remaining sources to cover generic costs.
///
/// Returns `None` if insufficient mana sources exist.
/// Returns `Some(vec![])` if the cost is zero.
pub fn find_mana_sources(
    game: &GameState,
    player_id: PlayerId,
    mana_cost: &ManaCost,
) -> Option<Vec<ManaSource>> {
    if mana_cost.symbols.is_empty() {
        return Some(Vec::new());
    }

    // Collect all available mana sources (untapped permanents with mana abilities)
    let mut available = available_mana_sources(game, player_id);

    // Tally specific color requirements
    let mut color_needs: Vec<ManaType> = Vec::new();
    let mut generic_need: u64 = 0;

    for sym in &mana_cost.symbols {
        match sym {
            ManaSymbol::Colored(mt) => color_needs.push(*mt),
            ManaSymbol::Colorless => color_needs.push(ManaType::Colorless),
            ManaSymbol::Generic => generic_need += 1,
            // Hybrid/Phyrexian/X not handled by auto-tap yet
            _ => return None,
        }
    }

    let mut tapped: Vec<ManaSource> = Vec::new();

    // Phase 1: Reserve sources for colored requirements.
    // For each colored need, find a source that produces exactly that color.
    // TODO: Prefer single-color producers to avoid wasting dual-producers (not yet implemented).
    for needed_color in &color_needs {
        if let Some(idx) = available.iter().position(|s| s.produces == *needed_color) {
            tapped.push(available.remove(idx));
        } else {
            // Can't satisfy this colored requirement
            return None;
        }
    }

    // Phase 2: Assign remaining sources to cover generic cost.
    for _ in 0..generic_need {
        if let Some(source) = available.pop() {
            tapped.push(source);
        } else {
            return None;
        }
    }

    Some(tapped)
}

/// Get all mana sources controlled by a player whose costs can currently be paid.
///
/// A mana source is a permanent with at least one mana ability whose costs
/// can be paid right now. Mana abilities don't always require tapping
/// (rule 605.1a/605.1b) — e.g. "Sacrifice this creature: Add {U}{R}" can
/// be activated even if the creature is tapped. We check each ability's
/// cost vector individually.
///
/// **Limitation:** For mana abilities whose costs include `Cost::Mana`, the
/// check only looks at the current pool — it does NOT consider whether the
/// player could generate the needed mana by activating *other* mana abilities
/// first. A recursive "mana bootstrap" solver would be needed for that, but
/// no cards in the current pool have mana-costing mana abilities, so this
/// doesn't bite us yet.
// TODO(Phase 5+): Implement mana-bootstrap solver for mana abilities that
// themselves cost mana (e.g. "{1}, {T}: Add {G}{G}"). Requires computing
// a dependency graph of mana sources and checking reachability.
pub fn available_mana_sources(game: &GameState, player_id: PlayerId) -> Vec<ManaSource> {
    let mut sources = Vec::new();

    for (id, entry) in &game.battlefield {
        if entry.controller != player_id {
            continue;
        }

        let obj = match game.objects.get(id) {
            Some(o) => o,
            None => continue,
        };

        for ability in &obj.card_data.abilities {
            if ability.ability_type != AbilityType::Mana {
                continue;
            }

            // Delegate to the engine's authoritative cost checker.
            // This handles all Cost variants (Tap, Untap, Mana, PayLife,
            // SacrificeSelf, Sacrifice, Discard, etc.) and will correctly
            // reject costs it cannot validate rather than silently passing.
            if game.can_pay_costs(&ability.costs, player_id, *id).is_err() {
                continue;
            }

            // Extract what this mana ability produces
            if let crate::types::effects::Effect::Atom(
                crate::types::effects::Primitive::ProduceMana(output),
                _,
            ) = &ability.effect
            {
                for (&mana_type, &amount) in &output.mana {
                    if amount > 0 {
                        sources.push(ManaSource {
                            permanent_id: *id,
                            ability_id: ability.id,
                            produces: mana_type,
                        });
                    }
                }
            }
        }
    }

    sources
}

/// For each spell in hand that passes timing checks, check if `find_mana_sources`
/// can cover its cost. Returns spell ID + the mana sources that would need tapping.
pub fn castable_spells(
    game: &GameState,
    player_id: PlayerId,
) -> Vec<(ObjectId, Vec<ManaSource>)> {
    let player = match game.players.get(player_id) {
        Some(p) => p,
        None => return Vec::new(),
    };

    let mut result = Vec::new();

    for &card_id in &player.hand {
        let obj = match game.objects.get(&card_id) {
            Some(o) => o,
            None => continue,
        };

        // Must have a spell ability
        let has_spell = obj.card_data.abilities.iter()
            .any(|a| a.ability_type == AbilityType::Spell);
        if !has_spell {
            continue;
        }

        // Timing check (sorcery-speed vs instant)
        if !passes_timing_check(game, player_id, card_id) {
            continue;
        }

        // Check mana affordability
        if let Some(ref mana_cost) = obj.card_data.mana_cost {
            // Account for mana already floating in the pool
            let pool = &game.players[player_id].mana_pool;
            if pool.can_pay(mana_cost) {
                // Already have enough floating mana, no tapping needed
                result.push((card_id, Vec::new()));
            } else {
                // Color-sensitive subtract pool mana from cost, then check taps
                let remaining = remaining_cost_after_pool(mana_cost, pool);
                if let Some(sources) = find_mana_sources(game, player_id, &remaining) {
                    result.push((card_id, sources));
                }
            }
        } else {
            // No mana cost (e.g., lands shouldn't have spell abilities, but handle gracefully)
            result.push((card_id, Vec::new()));
        }
    }

    result
}

/// Color-sensitive subtraction of pool mana from a mana cost.
///
/// For each colored symbol in the cost, if the pool has that color available
/// (beyond what earlier symbols already consumed), skip the symbol. For generic
/// symbols, subtract any excess pool mana. Returns a new ManaCost representing
/// only the portion that must still be covered by tapping sources.
fn remaining_cost_after_pool(
    cost: &ManaCost,
    pool: &crate::types::mana::ManaPool,
) -> ManaCost {
    // Snapshot pool amounts so we can "spend" conceptually without mutating
    let mut available: std::collections::HashMap<ManaType, u64> = pool.available().clone();

    let mut remaining_symbols: Vec<ManaSymbol> = Vec::new();

    // First pass: handle colored/colorless symbols
    let mut generic_symbols: Vec<ManaSymbol> = Vec::new();
    for sym in &cost.symbols {
        match sym {
            ManaSymbol::Colored(mt) => {
                let avail = available.entry(*mt).or_insert(0);
                if *avail > 0 {
                    *avail -= 1; // pool covers this symbol
                } else {
                    remaining_symbols.push(*sym);
                }
            }
            ManaSymbol::Colorless => {
                let avail = available.entry(ManaType::Colorless).or_insert(0);
                if *avail > 0 {
                    *avail -= 1;
                } else {
                    remaining_symbols.push(*sym);
                }
            }
            ManaSymbol::Generic => {
                generic_symbols.push(*sym);
            }
            // Hybrid/Phyrexian/X — can't auto-subtract, keep as-is
            other => remaining_symbols.push(*other),
        }
    }

    // Second pass: generic symbols can be paid by any remaining pool mana
    let mut excess: u64 = available.values().sum();
    for sym in generic_symbols {
        if excess > 0 {
            excess -= 1; // pool covers this generic
        } else {
            remaining_symbols.push(sym);
        }
    }

    ManaCost::from_symbols(remaining_symbols)
}

/// Check if a card in hand passes the timing check for casting.
/// Mirrors the logic in `check_cast_legality` but as a read-only query.
fn passes_timing_check(game: &GameState, player_id: PlayerId, card_id: ObjectId) -> bool {
    let obj = match game.objects.get(&card_id) {
        Some(o) => o,
        None => return false,
    };

    // Must own the card (rule 601.3)
    if obj.owner != player_id {
        return false;
    }

    // Must be in hand
    if obj.zone != crate::types::zones::Zone::Hand {
        return false;
    }

    let is_instant = obj.card_data.types.contains(&crate::types::card_types::CardType::Instant);
    let has_flash = obj.card_data.keywords.contains(&crate::types::keywords::KeywordAbility::Flash);

    if is_instant || has_flash {
        return true; // can cast anytime with priority
    }

    // Sorcery-speed: active player, main phase, empty stack
    if player_id != game.active_player {
        return false;
    }
    let is_main = matches!(
        game.phase.phase_type,
        crate::state::game_state::PhaseType::Precombat | crate::state::game_state::PhaseType::Postcombat
    );
    if !is_main {
        return false;
    }
    if !game.stack.is_empty() {
        return false;
    }

    true
}

/// Non-mana activated abilities the player can currently pay for.
///
/// For abilities with a mana cost component, checks both pool mana and
/// available mana sources (lands to tap), mirroring `castable_spells`.
/// Returns (source_permanent_id, ability_index, ability_id).
pub fn activatable_abilities(
    game: &GameState,
    player_id: PlayerId,
) -> Vec<(ObjectId, usize, AbilityId)> {
    let mut result = Vec::new();

    for (id, entry) in &game.battlefield {
        if entry.controller != player_id {
            continue;
        }

        let obj = match game.objects.get(id) {
            Some(o) => o,
            None => continue,
        };

        for (idx, ability) in obj.card_data.abilities.iter().enumerate() {
            if ability.ability_type != AbilityType::Activated {
                continue;
            }

            // Single-pass check: non-mana costs via engine, mana costs via
            // pool + available sources. No double-check.
            if !can_afford_ability_costs(game, player_id, *id, &ability.costs) {
                continue;
            }

            result.push((*id, idx, ability.id));
        }
    }

    result
}

/// Check if an ability's costs can be met right now.
///
/// Single authoritative check for all cost types:
/// - **Mana costs:** pool mana is subtracted first; `find_mana_sources` checks
///   whether available mana sources (lands, mana rocks, mana dorks, etc.) can
///   cover the remainder.
/// - **Non-mana costs:** delegated to `game.can_pay_costs` which is the
///   engine's authoritative per-variant checker. Unknown/unimplemented cost
///   variants return `Err` there (conservative rejection, not silent pass).
fn can_afford_ability_costs(
    game: &GameState,
    player_id: PlayerId,
    source_id: ObjectId,
    costs: &[crate::objects::card_data::Cost],
) -> bool {
    let pool = &game.players[player_id].mana_pool;

    for cost in costs {
        match cost {
            crate::objects::card_data::Cost::Mana(mana_cost) => {
                if pool.can_pay(mana_cost) {
                    continue;
                }
                let remaining = remaining_cost_after_pool(mana_cost, pool);
                if find_mana_sources(game, player_id, &remaining).is_none() {
                    return false;
                }
            }
            other => {
                if game.can_pay_costs(&[other.clone()], player_id, source_id).is_err() {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::{AbilityDef, CardDataBuilder};
    use crate::objects::object::GameObject;
    use crate::state::battlefield::BattlefieldEntity;
    use crate::state::game_state::{GameState, Phase, PhaseType};
    use crate::types::card_types::*;
    use crate::types::effects::{AmountExpr, Effect, Primitive, TargetSpec, TargetCount};
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;

    fn place_forest(game: &mut GameState, player_id: PlayerId) -> (ObjectId, AbilityId) {
        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Forest))
            .mana_ability_single(ManaType::Green)
            .build();
        let ability_id = forest.abilities[0].id;
        let obj = GameObject::new(forest, player_id, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = BattlefieldEntity::new(id, player_id, ts, 0);
        game.battlefield.insert(id, entry);
        (id, ability_id)
    }

    fn place_mountain(game: &mut GameState, player_id: PlayerId) -> (ObjectId, AbilityId) {
        let mountain = CardDataBuilder::new("Mountain")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Mountain))
            .mana_ability_single(ManaType::Red)
            .build();
        let ability_id = mountain.abilities[0].id;
        let obj = GameObject::new(mountain, player_id, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = BattlefieldEntity::new(id, player_id, ts, 0);
        game.battlefield.insert(id, entry);
        (id, ability_id)
    }

    #[test]
    fn test_find_mana_sources_zero_cost() {
        let game = GameState::new(2, 20);
        let result = find_mana_sources(&game, 0, &ManaCost::zero());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_find_mana_sources_single_green() {
        let mut game = GameState::new(2, 20);
        place_forest(&mut game, 0);
        let cost = ManaCost::build(&[ManaType::Green], 0); // {G}
        let result = find_mana_sources(&game, 0, &cost);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_find_mana_sources_colored_plus_generic() {
        let mut game = GameState::new(2, 20);
        place_forest(&mut game, 0);
        place_forest(&mut game, 0);
        let cost = ManaCost::build(&[ManaType::Green], 1); // {1}{G}
        let result = find_mana_sources(&game, 0, &cost);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_find_mana_sources_insufficient() {
        let mut game = GameState::new(2, 20);
        place_forest(&mut game, 0);
        let cost = ManaCost::build(&[ManaType::Red], 0); // {R} — no mountains
        let result = find_mana_sources(&game, 0, &cost);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_mana_sources_generic_with_any_color() {
        let mut game = GameState::new(2, 20);
        place_mountain(&mut game, 0);
        // {1} — any color pays for generic
        let cost = ManaCost::from_symbols(vec![ManaSymbol::Generic]);
        let result = find_mana_sources(&game, 0, &cost);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_available_mana_sources_skips_tapped() {
        let mut game = GameState::new(2, 20);
        let (id, _) = place_forest(&mut game, 0);
        game.battlefield.get_mut(&id).unwrap().tapped = true;

        let sources = available_mana_sources(&game, 0);
        assert!(sources.is_empty());
    }

    #[test]
    fn test_available_mana_sources_skips_opponent() {
        let mut game = GameState::new(2, 20);
        place_forest(&mut game, 1); // opponent's forest

        let sources = available_mana_sources(&game, 0);
        assert!(sources.is_empty());
    }

    #[test]
    fn test_castable_spells_finds_affordable() {
        let mut game = GameState::new(2, 20);
        game.phase = Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        place_mountain(&mut game, 0);

        // Put a bolt in hand
        let bolt = CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::build(&[ManaType::Red], 0))
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage(AmountExpr::Fixed(3)),
                    TargetSpec::Any(TargetCount::Exactly(1)),
                ),
            })
            .build();
        let obj = GameObject::new(bolt, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);

        let castable = castable_spells(&game, 0);
        assert_eq!(castable.len(), 1);
        assert_eq!(castable[0].0, card_id);
    }

    #[test]
    fn test_castable_spells_empty_when_unaffordable() {
        let mut game = GameState::new(2, 20);
        game.phase = Phase::new(PhaseType::Precombat);
        game.active_player = 0;
        // No lands

        let bolt = CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::build(&[ManaType::Red], 0))
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage(AmountExpr::Fixed(3)),
                    TargetSpec::Any(TargetCount::Exactly(1)),
                ),
            })
            .build();
        let obj = GameObject::new(bolt, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);

        let castable = castable_spells(&game, 0);
        assert!(castable.is_empty());
    }

    #[test]
    fn test_available_mana_sources_sacrifice_ability_on_tapped_creature() {
        // A tapped creature with "Sacrifice: Add {U}{R}" should still be a valid source
        use crate::objects::card_data::{AbilityType, Cost};
        use crate::types::effects::{ManaOutput, Primitive, Effect, TargetSpec};

        let mut game = GameState::new(2, 20);
        let mut mana_produced = std::collections::HashMap::new();
        mana_produced.insert(ManaType::Blue, 1u64);
        mana_produced.insert(ManaType::Red, 1u64);

        let card = CardDataBuilder::new("Morgue Toad")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Mana,
                costs: vec![Cost::SacrificeSelf],
                effect: Effect::Atom(
                    Primitive::ProduceMana(ManaOutput { mana: mana_produced }),
                    TargetSpec::None,
                ),
            })
            .build();
        let obj = GameObject::new(card, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = BattlefieldEntity::new(id, 0, ts, 0);
        entry.tapped = true; // tapped — but ability doesn't require tap
        game.battlefield.insert(id, entry);

        let sources = available_mana_sources(&game, 0);
        // Should find 2 sources (one for U, one for R) despite being tapped
        assert_eq!(sources.len(), 2);
    }

    #[test]
    fn test_remaining_cost_after_pool_partial_colored() {
        use crate::types::mana::ManaPool;

        // Cost: {1}{G}{G}, pool has 1G → remaining should be {1}{G}
        let cost = ManaCost::from_symbols(vec![
            ManaSymbol::Generic,
            ManaSymbol::Colored(ManaType::Green),
            ManaSymbol::Colored(ManaType::Green),
        ]);
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 1);

        let remaining = remaining_cost_after_pool(&cost, &pool);
        assert_eq!(remaining.generic_count(), 1);
        assert_eq!(remaining.colored_count(ManaType::Green), 1);
    }

    #[test]
    fn test_remaining_cost_after_pool_generic_covered() {
        use crate::types::mana::ManaPool;

        // Cost: {2}{R}, pool has 1R 1G → remaining should be {1}
        // Pool covers {R} (specific) and {1} of the generic with {G}
        let cost = ManaCost::from_symbols(vec![
            ManaSymbol::Generic,
            ManaSymbol::Generic,
            ManaSymbol::Colored(ManaType::Red),
        ]);
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 1);
        pool.add(ManaType::Green, 1);

        let remaining = remaining_cost_after_pool(&cost, &pool);
        assert_eq!(remaining.colored_count(ManaType::Red), 0);
        assert_eq!(remaining.generic_count(), 1);
    }

    #[test]
    fn test_castable_spells_with_partial_pool_mana() {
        // {1}{R} bolt with 1G in pool + 1 Mountain on battlefield
        // Pool covers the {1} generic, Mountain covers {R}
        let mut game = GameState::new(2, 20);
        game.phase = Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        place_mountain(&mut game, 0);
        game.players[0].mana_pool.add(ManaType::Green, 1);

        // A spell costing {1}{R}
        let spell = CardDataBuilder::new("Shock Plus")
            .card_type(CardType::Instant)
            .mana_cost(ManaCost::build(&[ManaType::Red], 1))
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage(AmountExpr::Fixed(2)),
                    TargetSpec::Any(TargetCount::Exactly(1)),
                ),
            })
            .build();
        let obj = GameObject::new(spell, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);

        let castable = castable_spells(&game, 0);
        assert_eq!(castable.len(), 1, "Should be castable with pool + tap");
    }

    #[test]
    fn test_castable_spells_respects_timing() {
        let mut game = GameState::new(2, 20);
        // Combat phase — sorceries can't be cast
        game.phase = Phase::new(PhaseType::Combat);
        game.active_player = 0;

        place_mountain(&mut game, 0);

        let sorcery = CardDataBuilder::new("Lava Axe")
            .card_type(CardType::Sorcery)
            .mana_cost(ManaCost::build(&[ManaType::Red], 4))
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage(AmountExpr::Fixed(5)),
                    TargetSpec::Player(TargetCount::Exactly(1)),
                ),
            })
            .build();
        let obj = GameObject::new(sorcery, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);

        let castable = castable_spells(&game, 0);
        assert!(castable.is_empty());
    }
}
