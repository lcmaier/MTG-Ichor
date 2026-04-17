// Random DecisionProvider — makes random legal choices for fuzz testing.
//
// Implements the 4-primitive `DecisionProvider` trait by picking uniformly at
// random among the options the engine presents. Holds one piece of interior-
// mutable state: a per-mana-ability-window activation counter used to cap
// pathological filter-ability chains during fuzz (see `pick_n` below). This
// is NOT the plan/queue-based replay state of the old stateful RandomDP — it
// is a bounded, single-window, policy-local counter.
//
// Tap-before-cast sequencing is *not* RandomDP's concern — the engine runs
// the 601.2g / 602.1b mana-ability-window loop inside `cast_spell` and
// `activate_ability`, prompting this DP once per mana-ability activation.
// During that loop RandomDP always picks an activation (never randomly
// declines) up to `WINDOW_ACTIVATION_CAP`, after which it declines so the
// engine bails out cleanly.
//
// Auto-tap as a *strategic* concern (choose which dual land to tap, whether
// to save a Cavern of Souls for an uncounterable creature later) is a
// future middleware DP concern, not this type's job — see
// `plans/atomic-tests/supplemental-docs/dp-middleware-and-candidate-enumeration.md` §4.

use std::cell::Cell;

use rand::Rng;
use rand::seq::SliceRandom;

use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use crate::ui::decision::DecisionProvider;

/// A decision provider that makes random legal choices.
///
/// Designed for fuzz testing: run many games of Random vs Random to surface
/// panics and edge cases in the engine.
///
/// Implements the 4-primitive `DecisionProvider` trait. The `ask_*` functions
/// in `ui::ask` handle semantic context; this provider just picks randomly
/// among the options presented to it — with one exception: during a
/// `ChoiceKind::ManaAbilityWindow`, it always activates (never randomly
/// declines) until the per-window activation cap is hit, at which point it
/// declines so the 601.2g / 602.1b loop exits and the engine rolls back any
/// unpayable cost. See `pick_n` for details.
pub struct RandomDecisionProvider {
    /// Current mana-ability window tracker: `(spell_or_ability_id, activations_so_far)`.
    /// Resets when a new window id is seen. See `pick_n` for the rationale.
    window: Cell<Option<(ObjectId, u32)>>,
}

impl RandomDecisionProvider {
    /// Max activations per mana-ability window before RandomDP declines.
    /// Bounds pathological filter-ability chains during fuzz without
    /// constraining legitimate mana plans (real plans rarely exceed ~10).
    pub const WINDOW_ACTIVATION_CAP: u32 = 32;

    pub fn new() -> Self {
        RandomDecisionProvider { window: Cell::new(None) }
    }
}

impl Default for RandomDecisionProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DecisionProvider for RandomDecisionProvider {
    fn pick_n(
        &self,
        _game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        if options.is_empty() || bounds.1 == 0 {
            return Vec::new();
        }
        let mut rng = rand::rng();

        // During a `ManaAbilityWindow`, RandomDP always picks an activation
        // (never randomly declines) so fuzz exercises full cost-payment
        // paths. Termination is controlled by the engine via
        // `can_pay_costs` success / enumeration-empty / failure blacklist,
        // plus the per-window activation cap here as a safety net against
        // pathological filter-ability chains (e.g., `{1}: Add one mana of
        // any color` cycled forever). Once the cap is hit we return empty
        // (decline), letting the engine exit the window; any unpayable cost
        // then triggers clean rollback via the caller.
        if let ChoiceKind::ManaAbilityWindow { spell_or_ability_id, .. } = &context.kind {
            let (win_id, count) = match self.window.get() {
                Some((id, n)) if id == *spell_or_ability_id => (id, n),
                _ => (*spell_or_ability_id, 0),
            };
            if count >= Self::WINDOW_ACTIVATION_CAP {
                self.window.set(Some((win_id, count)));
                return Vec::new();
            }
            let idx = rng.random_range(0..options.len());
            self.window.set(Some((win_id, count + 1)));
            return vec![idx];
        }

        let count = if bounds.0 == bounds.1 {
            bounds.0
        } else {
            rng.random_range(bounds.0..=bounds.1)
        };

        // SPECIAL-8 stretch: for `DeclareBlockers`, dedup on blocker-id so
        // RandomDP converges to a legal set in one shot instead of thrashing
        // the engine's CR 509.1c retry loop. Each blocker can block at most
        // one attacker by default (no Menace-opposite / multi-block keywords
        // yet). The engine's retry loop remains a safety net — this branch
        // just accelerates convergence.
        if matches!(context.kind, ChoiceKind::DeclareBlockers) {
            let mut shuffled: Vec<usize> = (0..options.len()).collect();
            shuffled.shuffle(&mut rng);
            let mut used_blockers: std::collections::HashSet<ObjectId> =
                std::collections::HashSet::new();
            let mut picked: Vec<usize> = Vec::new();
            for idx in shuffled {
                if picked.len() >= count {
                    break;
                }
                if let ChoiceOption::BlockerAttacker(blocker, _) = &options[idx] {
                    if used_blockers.insert(*blocker) {
                        picked.push(idx);
                    }
                } else {
                    // Unexpected option shape — include anyway (engine will validate).
                    picked.push(idx);
                }
            }
            picked.sort();
            return picked;
        }

        let mut indices: Vec<usize> = (0..options.len()).collect();
        indices.shuffle(&mut rng);
        indices.truncate(count);
        indices.sort(); // stable ordering for determinism in tests
        indices
    }

    fn pick_number(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        min: u64,
        max: u64,
    ) -> u64 {
        let mut rng = rand::rng();

        // For ChooseXValue, self-limit based on available mana to avoid
        // degenerate rollback loops in fuzz testing. The ask function passes
        // (0, u64::MAX) — we inspect game state for a reasonable upper bound.
        if let ChoiceKind::ChooseXValue { .. } = &context.kind {
            let pool_total: u64 = game.players.get(player)
                .map(|p| p.mana_pool.total())
                .unwrap_or(0);
            // Count untapped lands as potential mana sources
            let untapped_lands: u64 = game.battlefield.iter()
                .filter(|(_, e)| {
                    e.controller == player && !e.tapped
                })
                .filter(|(id, _)| {
                    game.objects.get(id)
                        .map(|o| o.card_data.types.contains(&crate::types::card_types::CardType::Land))
                        .unwrap_or(false)
                })
                .count() as u64;
            let reasonable_max = pool_total + untapped_lands;
            let effective_max = reasonable_max.min(max);
            if effective_max <= min {
                return min;
            }
            return rng.random_range(min..=effective_max);
        }

        // General case: pick in the given range
        // Guard against u64::MAX range causing overflow
        if max == u64::MAX && min == 0 {
            // Pick a small reasonable number to avoid degenerate behavior
            return rng.random_range(0..=20);
        }
        rng.random_range(min..=max)
    }

    fn allocate(
        &self,
        _game: &GameState,
        _player: PlayerId,
        _context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        let n = buckets.len();
        if n == 0 {
            return Vec::new();
        }

        // Start with minimums
        let mut alloc: Vec<u64> = per_bucket_mins.to_vec();
        let min_sum: u64 = alloc.iter().sum();
        let mut remaining = total.saturating_sub(min_sum);

        // Distribute remaining randomly across buckets, respecting maxs
        let mut rng = rand::rng();
        while remaining > 0 {
            // Collect buckets that can still accept more
            let eligible: Vec<usize> = (0..n)
                .filter(|&i| {
                    per_bucket_maxs
                        .map_or(true, |maxs| alloc[i] < maxs[i])
                })
                .collect();
            if eligible.is_empty() {
                break;
            }
            let bucket = eligible[rng.random_range(0..eligible.len())];
            let headroom = per_bucket_maxs
                .map_or(remaining, |maxs| (maxs[bucket] - alloc[bucket]).min(remaining));
            let give = if headroom <= 1 { 1 } else { rng.random_range(1..=headroom) };
            alloc[bucket] += give;
            remaining -= give;
        }

        alloc
    }

    fn choose_ordering(
        &self,
        _game: &GameState,
        _player: PlayerId,
        _context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize> {
        let mut rng = rand::rng();
        let mut indices: Vec<usize> = (0..items.len()).collect();
        indices.shuffle(&mut rng);
        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::BattlefieldEntity;
    use crate::state::game_state::{GameState, Phase, PhaseType};
    use crate::types::card_types::*;
    use crate::types::ids::ObjectId;
    use crate::types::mana::ManaType;
    use crate::types::zones::Zone;
    use crate::ui::decision::PriorityAction;

    fn setup_basic_game() -> GameState {
        let mut game = GameState::new(2, 20);
        game.phase = Phase::new(PhaseType::Precombat);
        game.active_player = 0;
        game
    }

    #[allow(dead_code)]
    fn place_forest(game: &mut GameState, player_id: PlayerId) -> ObjectId {
        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Forest))
            .mana_ability_single(ManaType::Green)
            .build();
        let obj = GameObject::new(forest, player_id, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = BattlefieldEntity::new(id, player_id, ts, 0);
        game.battlefield.insert(id, entry);
        id
    }

    #[test]
    fn test_random_dp_pick_n_empty() {
        let dp = RandomDecisionProvider::new();
        let game = setup_basic_game();
        let ctx = ChoiceContext { kind: ChoiceKind::PriorityAction };
        let result = dp.pick_n(&game, 0, &ctx, &[], (0, 0));
        assert!(result.is_empty());
    }

    #[test]
    fn test_random_dp_pick_n_selects_within_bounds() {
        let dp = RandomDecisionProvider::new();
        let game = setup_basic_game();
        let ctx = ChoiceContext { kind: ChoiceKind::PriorityAction };
        let options = vec![ChoiceOption::Action(PriorityAction::Pass); 3];
        let result = dp.pick_n(&game, 0, &ctx, &options, (1, 2));
        assert!(result.len() >= 1 && result.len() <= 2);
        for &idx in &result {
            assert!(idx < 3);
        }
    }

    #[test]
    fn test_random_dp_pick_number_in_range() {
        let dp = RandomDecisionProvider::new();
        let game = setup_basic_game();
        let spell_id = crate::types::ids::new_object_id();
        let ctx = ChoiceContext { kind: ChoiceKind::ChooseXValue { spell_id, x_count: 1 } };
        let result = dp.pick_number(&game, 0, &ctx, 0, 10);
        assert!(result <= 10);
    }

    #[test]
    fn test_random_dp_allocate_sums_to_total() {
        let dp = RandomDecisionProvider::new();
        let game = setup_basic_game();
        let id_a = crate::types::ids::new_object_id();
        let id_b = crate::types::ids::new_object_id();
        let ctx = ChoiceContext { kind: ChoiceKind::AssignCombatDamage { attacker_id: id_a } };
        let buckets = vec![ChoiceOption::Object(id_a), ChoiceOption::Object(id_b)];
        let mins = vec![0, 0];
        let result = dp.allocate(&game, 0, &ctx, 5, &buckets, &mins, None);
        assert_eq!(result.len(), 2);
        assert_eq!(result.iter().sum::<u64>(), 5);
    }
}
