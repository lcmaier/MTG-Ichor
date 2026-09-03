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
// **It is random among the choices that can pay, not among all of them
// (2026-09-03).** Two of its answers are not uniform: in the mana window it
// taps a source that makes a pip the cost still needs before it taps
// anything else, least flexible source first, and declines when no source
// can make a pip that is still owed; and it splits the generic part of a cost
// only across mana the same payment does not need for a pip. Both are
// policy, not payment law — the prompt still offers every legal option and
// the engine still validates what comes back — and both exist because an
// any-color mana base (`cards/dual_lands.rs::everywhere`) turned a uniform
// tap into a five-sided die: land taps per spell cast doubled, 3.86 → 7.66,
// and spells per game fell with them (`codebase-state.md` 16d). A failed
// payment rewinds with the lands still tapped, so a wrong tap is a turn's
// mana gone. With this policy taps per cast read 3.18.
//
// Auto-tap as a *strategic* concern (which dual to tap, whether to save a
// Cavern of Souls for an uncounterable creature later) is still a future
// middleware DP concern, not this type's job — see
// `plans/atomic-tests/supplemental-docs/dp-middleware-and-candidate-enumeration.md` §4.

use std::cell::{Cell, RefCell};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand::seq::SliceRandom;

use crate::oracle::mana_helpers::available_mana_sources;
use crate::state::game_state::GameState;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use crate::ui::decision::{DecisionProvider, PriorityAction};

/// What a mana window's options can do for the pips a cost still owes.
enum WindowPreference {
    /// Indices of the options that make a type some unpaid pip accepts, among
    /// the sources with the fewest such types — tap the Forest before the
    /// five-color land, so the land is still there for the pip only it can
    /// make.
    Useful(Vec<usize>),
    /// Nothing but generic (or nothing this policy reads) is owed: any source.
    AnyWillDo,
    /// A pip is owed that no offered source can make. Tapping anything now
    /// spends mana on a payment that will fail and rewind.
    Hopeless,
}

/// The policy behind [`WindowPreference`], read off `remaining_cost` — the
/// pips the pool does not already cover — and what each offered ability
/// produces. A hybrid pip accepts either half; a mono-hybrid or Phyrexian
/// pip is payable without its color and so expresses no preference; X and
/// generic never do.
fn mana_window_preference(
    game: &GameState,
    player: PlayerId,
    remaining: &ManaCost,
    options: &[ChoiceOption],
) -> WindowPreference {
    let mut wanted: Vec<ManaType> = Vec::new();
    let mut owes_a_pip = false;
    for sym in &remaining.symbols {
        match sym {
            ManaSymbol::Colored(t) => {
                owes_a_pip = true;
                wanted.push(*t);
            }
            ManaSymbol::Colorless => {
                owes_a_pip = true;
                wanted.push(ManaType::Colorless);
            }
            ManaSymbol::Hybrid(a, b) => {
                owes_a_pip = true;
                wanted.push(*a);
                wanted.push(*b);
            }
            _ => {}
        }
    }
    if !owes_a_pip {
        return WindowPreference::AnyWillDo;
    }

    let produces: std::collections::HashMap<(ObjectId, AbilityId), ManaType> =
        available_mana_sources(game, player)
            .into_iter()
            .map(|s| ((s.permanent_id, s.ability_id), s.produces))
            .collect();
    // How many wanted types each permanent can make: its flexibility.
    let mut flexibility: std::collections::HashMap<ObjectId, usize> =
        std::collections::HashMap::new();
    for ((perm, _), t) in &produces {
        if wanted.contains(t) {
            *flexibility.entry(*perm).or_insert(0) += 1;
        }
    }

    let mut useful: Vec<(usize, usize)> = Vec::new();
    let mut unreadable = false;
    for (i, opt) in options.iter().enumerate() {
        let ChoiceOption::Action(PriorityAction::ActivateAbility(perm, ab)) = opt else {
            unreadable = true;
            continue;
        };
        match produces.get(&(*perm, *ab)) {
            Some(t) if wanted.contains(t) => useful.push((i, flexibility[perm])),
            Some(_) => {}
            None => unreadable = true,
        }
    }
    if useful.is_empty() {
        // An option this policy cannot read might still be the right one;
        // only a fully-read window with nothing useful in it is hopeless.
        return if unreadable { WindowPreference::AnyWillDo } else { WindowPreference::Hopeless };
    }
    let least = useful.iter().map(|(_, f)| *f).min().unwrap();
    WindowPreference::Useful(useful.into_iter().filter(|(_, f)| *f == least).map(|(i, _)| i).collect())
}

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

    /// The one source of randomness for every decision this provider makes.
    ///
    /// Owned rather than pulled from `rand::rng()` per call: `ThreadRng` is
    /// seeded from the OS, so a provider built on it makes different choices
    /// every process even when the caller passed a seed. `fuzz_games --seed N`
    /// was in exactly that position — the seed reached deck construction and
    /// stopped there. `RefCell` because `DecisionProvider` takes `&self`.
    rng: RefCell<StdRng>,
}

impl RandomDecisionProvider {
    /// Max activations per mana-ability window before RandomDP declines.
    /// Bounds pathological filter-ability chains during fuzz without
    /// constraining legitimate mana plans (real plans rarely exceed ~10).
    pub const WINDOW_ACTIVATION_CAP: u32 = 32;

    /// A provider seeded from the OS — different choices every run.
    ///
    /// For anything that wants to be replayable (the fuzz harness, a test
    /// reproducing a reported panic), use [`RandomDecisionProvider::seeded`].
    pub fn new() -> Self {
        RandomDecisionProvider {
            window: Cell::new(None),
            rng: RefCell::new(StdRng::from_os_rng()),
        }
    }

    /// A provider whose whole decision stream is a function of `seed`.
    ///
    /// Reproducibility also needs the *options* to arrive in the same order —
    /// see `GameState::battlefield_ordered`. A seeded provider fed a
    /// differently-ordered candidate list picks a different action.
    pub fn seeded(seed: u64) -> Self {
        RandomDecisionProvider {
            window: Cell::new(None),
            rng: RefCell::new(StdRng::seed_from_u64(seed)),
        }
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
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        if options.is_empty() || bounds.1 == 0 {
            return Vec::new();
        }
        let mut rng = self.rng.borrow_mut();

        // During a `ManaAbilityWindow`, RandomDP picks an activation that
        // can still pay something (see `mana_window_preference`) and never
        // declines while one exists, so fuzz exercises full cost-payment
        // paths. Termination is controlled by the engine via
        // `can_pay_costs` success / enumeration-empty / failure blacklist,
        // plus the per-window activation cap here as a safety net against
        // pathological filter-ability chains (e.g., `{1}: Add one mana of
        // any color` cycled forever). Once the cap is hit — or once a pip is
        // owed that nothing offered can make — we return empty (decline),
        // letting the engine exit the window; any unpayable cost then
        // triggers clean rollback via the caller.
        if let ChoiceKind::ManaAbilityWindow { spell_or_ability_id, remaining_cost } =
            &context.kind
        {
            let (win_id, count) = match self.window.get() {
                Some((id, n)) if id == *spell_or_ability_id => (id, n),
                _ => (*spell_or_ability_id, 0),
            };
            if count >= Self::WINDOW_ACTIVATION_CAP {
                self.window.set(Some((win_id, count)));
                return Vec::new();
            }
            let pick_from: Vec<usize> =
                match mana_window_preference(game, player, remaining_cost, options) {
                    WindowPreference::Useful(indices) => indices,
                    WindowPreference::AnyWillDo => (0..options.len()).collect(),
                    WindowPreference::Hopeless => {
                        self.window.set(Some((win_id, count)));
                        return Vec::new();
                    }
                };
            let idx = pick_from[rng.random_range(0..pick_from.len())];
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
            shuffled.shuffle(&mut *rng);
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
        indices.shuffle(&mut *rng);
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
        let mut rng = self.rng.borrow_mut();

        // For ChooseXValue, self-limit based on available mana to avoid
        // degenerate rollback loops in fuzz testing. The ask function passes
        // (0, u64::MAX) — we inspect game state for a reasonable upper bound.
        if let ChoiceKind::ChooseXValue { .. } = &context.kind {
            let pool_total: u64 = game.players.get(player)
                .map(|p| p.mana_pool.total())
                .unwrap_or(0);
            // Count untapped lands as potential mana sources
            let untapped_lands: u64 = game.battlefield.iter()
                .filter(|(id, e)| {
                    !e.tapped
                        && crate::oracle::characteristics::controls(game, **id, player)
                })
                .filter(|(id, _)| {
                    crate::oracle::characteristics::has_type(
                        game, **id, crate::types::card_types::CardType::Land)
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
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        let n = buckets.len();
        if n == 0 {
            return Vec::new();
        }

        let mut caps: Vec<u64> = per_bucket_maxs.map_or(vec![u64::MAX; n], |m| m.to_vec());

        // A generic split spends from the pool the same payment pays its pips
        // from (CR 601.2h), and the prompt's caps are the pool's amounts, so
        // a split that spends a color a pip still needs passes the prompt
        // and fails at `ManaPool::pay` — which rewinds the cast with the
        // lands tapped. Cap each type at what is left after its own pips.
        // If that leaves less than `total` (it cannot, once `can_pay_costs`
        // passed), keep the prompt's caps so the answer is at least valid.
        if let ChoiceKind::GenericManaAllocation { mana_cost } = &context.kind {
            let mut clamped = caps.clone();
            for (i, bucket) in buckets.iter().enumerate() {
                if let ChoiceOption::ManaType(mt) = bucket {
                    let owed = match mt {
                        ManaType::Colorless => mana_cost
                            .symbols
                            .iter()
                            .filter(|s| matches!(s, ManaSymbol::Colorless))
                            .count() as u64,
                        t => mana_cost.colored_count(*t) as u64,
                    };
                    clamped[i] = clamped[i].saturating_sub(owed);
                }
            }
            let reach: u64 = clamped.iter().fold(0u64, |acc, c| acc.saturating_add(*c));
            if reach >= total {
                caps = clamped;
            }
        }

        // Start with minimums
        let mut alloc: Vec<u64> = per_bucket_mins.to_vec();
        let min_sum: u64 = alloc.iter().sum();
        let mut remaining = total.saturating_sub(min_sum);

        // Distribute remaining randomly across buckets, respecting caps
        let mut rng = self.rng.borrow_mut();
        while remaining > 0 {
            // Collect buckets that can still accept more
            let eligible: Vec<usize> = (0..n).filter(|&i| alloc[i] < caps[i]).collect();
            if eligible.is_empty() {
                break;
            }
            let bucket = eligible[rng.random_range(0..eligible.len())];
            let headroom = (caps[bucket] - alloc[bucket]).min(remaining);
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
        let mut rng = self.rng.borrow_mut();
        let mut indices: Vec<usize> = (0..items.len()).collect();
        indices.shuffle(&mut *rng);
        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::basic_lands::forest;
    use crate::cards::dual_lands::everywhere;
    use crate::oracle::mana_helpers::enumerate_activatable_mana_abilities;
    use crate::test_support::{put_on_battlefield, setup_two_player_game};
    use crate::test_support::setup_two_player_game as setup_basic_game;

    /// The window's options for player 0, as `ask_activate_mana_ability`
    /// builds them, and what each produces.
    fn window_options(game: &GameState) -> (Vec<ChoiceOption>, Vec<ManaType>) {
        let legal = enumerate_activatable_mana_abilities(game, 0);
        let produces: std::collections::HashMap<_, _> = available_mana_sources(game, 0)
            .into_iter()
            .map(|s| ((s.permanent_id, s.ability_id), s.produces))
            .collect();
        let options = legal
            .iter()
            .map(|(p, a)| ChoiceOption::Action(PriorityAction::ActivateAbility(*p, *a)))
            .collect();
        let types = legal.iter().map(|k| produces[k]).collect();
        (options, types)
    }

    fn window(remaining: ManaCost) -> ChoiceContext {
        ChoiceContext {
            kind: ChoiceKind::ManaAbilityWindow {
                spell_or_ability_id: crate::types::ids::new_object_id(),
                remaining_cost: remaining,
            },
        }
    }

    /// A five-color land offers five abilities; with `{G}` still owed the
    /// agent taps it for green every time, not one time in five.
    #[test]
    fn mana_window_taps_for_the_pip_still_owed() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, everywhere(), 0);
        let (options, types) = window_options(&game);
        assert_eq!(options.len(), 5);
        for seed in 0..20u64 {
            let dp = RandomDecisionProvider::seeded(seed);
            let ctx = window(ManaCost::build(&[ManaType::Green], 0));
            let pick = dp.pick_n(&game, 0, &ctx, &options, (0, 1));
            assert_eq!(types[pick[0]], ManaType::Green, "seed {seed}");
        }
    }

    /// `{G}{U}` owed, a Forest and a five-color land offered: the Forest goes
    /// first, because it is the source that can only make one of the two.
    #[test]
    fn mana_window_taps_the_least_flexible_source_first() {
        let mut game = setup_two_player_game();
        let forest = put_on_battlefield(&mut game, forest(), 0);
        put_on_battlefield(&mut game, everywhere(), 0);
        let (options, _) = window_options(&game);
        assert_eq!(options.len(), 6);
        for seed in 0..20u64 {
            let dp = RandomDecisionProvider::seeded(seed);
            let ctx = window(ManaCost::build(&[ManaType::Green, ManaType::Blue], 0));
            let pick = dp.pick_n(&game, 0, &ctx, &options, (0, 1));
            let ChoiceOption::Action(PriorityAction::ActivateAbility(perm, _)) = options[pick[0]]
            else {
                panic!("not an activation")
            };
            assert_eq!(perm, forest, "seed {seed}");
        }
    }

    /// Only generic owed: every source is fair game, and the agent still
    /// never declines while one is offered.
    #[test]
    fn mana_window_taps_anything_for_generic() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, everywhere(), 0);
        let (options, types) = window_options(&game);
        let mut seen = std::collections::HashSet::new();
        for seed in 0..40u64 {
            let dp = RandomDecisionProvider::seeded(seed);
            let ctx = window(ManaCost::build(&[], 1));
            let pick = dp.pick_n(&game, 0, &ctx, &options, (0, 1));
            assert_eq!(pick.len(), 1);
            seen.insert(types[pick[0]]);
        }
        assert!(seen.len() > 1, "generic is paid with whatever comes: {seen:?}");
    }

    /// `{U}` owed and only a Forest offered: decline, so the engine rewinds
    /// with the Forest untapped instead of after burning it.
    #[test]
    fn mana_window_declines_when_no_source_can_make_an_owed_pip() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, forest(), 0);
        let (options, _) = window_options(&game);
        assert_eq!(options.len(), 1);
        let dp = RandomDecisionProvider::seeded(1);
        let ctx = window(ManaCost::build(&[ManaType::Blue], 0));
        assert!(dp.pick_n(&game, 0, &ctx, &options, (0, 1)).is_empty());
    }

    /// `{1}{G}{U}` from a pool of one each of G, U and R: the generic mana is
    /// the red, every time — the prompt's caps would have let it be the green
    /// or the blue, and `ManaPool::pay` would then have refused the payment.
    #[test]
    fn generic_split_spends_only_what_the_pips_leave_over() {
        let game = setup_basic_game();
        let cost = ManaCost::build(&[ManaType::Green, ManaType::Blue], 1);
        let ctx = ChoiceContext { kind: ChoiceKind::GenericManaAllocation { mana_cost: cost } };
        let buckets = vec![
            ChoiceOption::ManaType(ManaType::Green),
            ChoiceOption::ManaType(ManaType::Blue),
            ChoiceOption::ManaType(ManaType::Red),
        ];
        for seed in 0..20u64 {
            let dp = RandomDecisionProvider::seeded(seed);
            let alloc = dp.allocate(&game, 0, &ctx, 1, &buckets, &[0, 0, 0], Some(&[1, 1, 1]));
            assert_eq!(alloc, vec![0, 0, 1], "seed {seed}");
        }
    }

    /// The same cost from `{G}{G}{U}`: the surplus green pays the generic.
    #[test]
    fn generic_split_uses_a_color_s_surplus_beyond_its_pips() {
        let game = setup_basic_game();
        let cost = ManaCost::build(&[ManaType::Green, ManaType::Blue], 1);
        let ctx = ChoiceContext { kind: ChoiceKind::GenericManaAllocation { mana_cost: cost } };
        let buckets =
            vec![ChoiceOption::ManaType(ManaType::Green), ChoiceOption::ManaType(ManaType::Blue)];
        let dp = RandomDecisionProvider::seeded(3);
        let alloc = dp.allocate(&game, 0, &ctx, 1, &buckets, &[0, 0], Some(&[2, 1]));
        assert_eq!(alloc, vec![1, 0]);
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
