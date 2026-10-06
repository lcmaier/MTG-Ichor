// Random DecisionProvider — makes random legal choices for fuzz testing.
//
// Implements the 4-primitive `DecisionProvider` trait by picking uniformly at
// random among the options the engine presents. Holds two pieces of interior-
// mutable state: a per-mana-ability-window activation counter that caps
// pathological filter-ability chains during fuzz, and the actions a priority
// window has rejected, which it does not choose again (see `pick_n` below).
//
// Tap-before-cast sequencing is *not* RandomDP's concern — the engine runs
// the 601.2g / 602.1b mana-ability-window loop inside `cast_spell` and
// `activate_ability`, prompting this DP once per mana-ability activation.
//
// **It is random among the choices that can pay, not among all of them.**
// Two of its answers are not uniform: in the mana window it taps a source
// that makes a pip the cost still needs before it taps anything else, least
// flexible source first, and declines when no source can make a pip that is
// still owed; and it splits the generic part of a cost only across mana the
// same payment does not need for a pip. Both are policy, not payment law —
// the prompt still offers every legal option and the engine still validates
// what comes back — and both exist because an any-color mana base
// (`cards/dual_lands.rs::everywhere`) turns a uniform tap into a five-sided
// die: land taps per spell cast read 7.66 uniform against 3.18 with the
// policy (2026-09-03), and a failed payment rewinds with the lands still
// tapped.
//
// Auto-tap as a *strategic* concern (which dual to tap, whether to save a
// Cavern of Souls for an uncounterable creature later) is a future
// middleware DP concern, not this type's job — see
// `plans/atomic-tests/supplemental-docs/dp-middleware-and-candidate-enumeration.md` §4.

use std::cell::{Cell, RefCell};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand::seq::SliceRandom;

use crate::oracle::mana_supply::available_mana_sources;
use crate::state::game_state::GameState;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption, Rejection};
use crate::ui::decision::{DecisionProvider, PriorityAction};

/// What a mana window's options can do for the pips a cost still owes.
pub(crate) enum WindowPreference {
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
pub(crate) fn mana_window_preference(
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

    // Keyed on the definition, as the window is: two grants of one mana
    // ability are one ability the permanent has, not two. An ability can
    // make several types, a land Wild Growth enchants its own and green.
    let mut produces: crate::types::ids::IdMap<(ObjectId, AbilityId), Vec<ManaType>> =
        crate::types::ids::IdMap::default();
    for source in available_mana_sources(game, player) {
        let types = produces.entry((source.permanent_id, source.ability_id.definition())).or_default();
        if !types.contains(&source.produces) {
            types.push(source.produces);
        }
    }
    // How many wanted types each permanent can make: its flexibility.
    let mut flexibility: crate::types::ids::IdMap<ObjectId, usize> =
        crate::types::ids::IdMap::default();
    for ((perm, _), types) in &produces {
        let made_and_wanted = types.iter().filter(|t| wanted.contains(t)).count();
        if made_and_wanted > 0 {
            *flexibility.entry(*perm).or_insert(0) += made_and_wanted;
        }
    }

    let mut useful: Vec<(usize, usize)> = Vec::new();
    let mut unreadable = false;
    for (i, opt) in options.iter().enumerate() {
        let ChoiceOption::Action(PriorityAction::ActivateAbility(perm, ab)) = opt else {
            unreadable = true;
            continue;
        };
        match produces.get(&(*perm, ab.definition())) {
            Some(types) if types.iter().any(|t| wanted.contains(t)) => useful.push((i, flexibility[perm])),
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
///
/// **`Clone` is the other half of a fork** (`codebase-state.md` item 41). A
/// harness that clones `GameState` at a decision point and plays the branch
/// forward has cloned half the game: this provider's `StdRng` lives outside
/// the state deliberately (`CLAUDE.md`'s second randomness opt-out), so a
/// branch that does not carry it re-randomizes every choice from the fork on.
/// Cloning both streams is what makes a branch a replay; a search that wants
/// determinization reseeds the branch's provider instead.
#[derive(Clone)]
pub struct RandomDecisionProvider {
    /// Current mana-ability window tracker: `(spell_or_ability_id, activations_so_far)`.
    /// Resets when a new window id is seen. See `pick_n` for the rationale.
    window: Cell<Option<(ObjectId, u32)>>,

    /// The actions the engine rejected in the priority window this provider
    /// is answering. The engine offers them again, since a person may take one
    /// again (CR 732.2); not choosing them is this agent's policy, which is
    /// what makes its re-picks end (`codebase-state.md` item 193). A window's
    /// first prompt carries no rejection, which is where the list empties.
    rejected_in_window: RefCell<Vec<PriorityAction>>,

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
            rejected_in_window: RefCell::new(Vec::new()),
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
            rejected_in_window: RefCell::new(Vec::new()),
            rng: RefCell::new(StdRng::seed_from_u64(seed)),
        }
    }

    /// The actions not to choose at this prompt: at a priority prompt, every
    /// action its window has rejected, `context`'s included; none elsewhere.
    fn actions_to_skip(&self, context: &ChoiceContext) -> Vec<PriorityAction> {
        if !matches!(context.kind, ChoiceKind::PriorityAction) {
            return Vec::new();
        }
        let mut rejected = self.rejected_in_window.borrow_mut();
        match &context.rejected {
            Some(Rejection::Reversed(action)) => rejected.push(action.clone()),
            _ => rejected.clear(),
        }
        rejected.clone()
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

        // During a `ManaAbilityWindow`, RandomDP picks an activation that can
        // still pay something (see `mana_window_preference`) and never declines
        // while one exists, so fuzz exercises full cost-payment paths. The
        // per-window activation cap is a safety net against pathological
        // filter-ability chains (`{1}: Add one mana of any color` cycled forever);
        // once it is hit, or once a pip is owed that nothing offered can make, it
        // declines and any unpayable cost rolls back at the caller.
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

        // For `DeclareBlockers`, dedup on blocker-id so RandomDP declares a
        // legal set in one shot instead of being asked again: each blocker
        // blocks one attacker (CR 509.1a) unless an effect lets it block more,
        // and this policy never uses one. The engine's re-ask remains the net —
        // this branch just spares it.
        if matches!(context.kind, ChoiceKind::DeclareBlockers) {
            let mut shuffled: Vec<usize> = (0..options.len()).collect();
            shuffled.shuffle(&mut *rng);
            let mut used_blockers: crate::types::ids::IdSet<ObjectId> =
                crate::types::ids::IdSet::default();
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

        // Filtered before the shuffle, whose draws depend on the length alone,
        // so this agent draws what it drew when the engine filtered the list
        // for it.
        let skip = self.actions_to_skip(context);
        let mut indices: Vec<usize> = (0..options.len())
            .filter(|&i| !matches!(&options[i], ChoiceOption::Action(action) if skip.contains(action)))
            .collect();
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

        // The prompt's caps already leave every pip of the cost its own mana
        // (`ask_choose_generic_mana_allocation`), so a generic split needs no
        // payment law here — taking them at face value is what a DP is owed.
        let caps: Vec<u64> = per_bucket_maxs.map_or(vec![u64::MAX; n], |m| m.to_vec());

        let mut alloc: Vec<u64> = per_bucket_mins.to_vec();
        let min_sum: u64 = alloc.iter().sum();
        let mut remaining = total.saturating_sub(min_sum);

        // Distribute remaining randomly across buckets, respecting caps
        let mut rng = self.rng.borrow_mut();
        while remaining > 0 {
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
    use crate::oracle::mana_supply::ManaAbilityWindowOffer;
    use crate::test_support::{put_on_battlefield, setup_two_player_game};
    use crate::test_support::setup_two_player_game as setup_basic_game;

    /// The window's options for player 0, as `ask_activate_mana_ability`
    /// builds them, and what each produces.
    fn window_options(game: &GameState) -> (Vec<ChoiceOption>, Vec<ManaType>) {
        let legal = ManaAbilityWindowOffer::read(game, 0).options(game);
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
        ChoiceContext::new(ChoiceKind::ManaAbilityWindow {
            spell_or_ability_id: crate::types::ids::new_object_id(),
            remaining_cost: remaining,
        })
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

    #[test]
    fn test_random_dp_pick_n_empty() {
        let dp = RandomDecisionProvider::new();
        let game = setup_basic_game();
        let ctx = ChoiceContext::new(ChoiceKind::PriorityAction);
        let result = dp.pick_n(&game, 0, &ctx, &[], (0, 0));
        assert!(result.is_empty());
    }

    #[test]
    fn test_random_dp_pick_n_selects_within_bounds() {
        let dp = RandomDecisionProvider::new();
        let game = setup_basic_game();
        let ctx = ChoiceContext::new(ChoiceKind::PriorityAction);
        let options = vec![ChoiceOption::Action(PriorityAction::Pass); 3];
        let result = dp.pick_n(&game, 0, &ctx, &options, (1, 2));
        assert!(!result.is_empty() && result.len() <= 2);
        for &idx in &result {
            assert!(idx < 3);
        }
    }

    /// `[Pass, Cast, Cast, Cast]`, the casts of fresh ids.
    fn priority_options() -> Vec<ChoiceOption> {
        let mut options = vec![ChoiceOption::Action(PriorityAction::Pass)];
        options.extend((0..3).map(|_| ChoiceOption::Action(PriorityAction::CastSpell(crate::types::ids::new_object_id()))));
        options
    }

    fn action(option: &ChoiceOption) -> PriorityAction {
        match option {
            ChoiceOption::Action(action) => action.clone(),
            other => panic!("{other:?} is no priority action"),
        }
    }

    /// The window asking again, having reversed `rejected`.
    fn re_ask(rejected: &ChoiceOption) -> ChoiceContext {
        ChoiceContext { kind: ChoiceKind::PriorityAction, rejected: Some(Rejection::Reversed(action(rejected))) }
    }

    /// Item 193: the agent skips what its window rejected, which the engine no
    /// longer does for it, and draws what it drew when the engine did. One
    /// agent is offered the whole list and told a cast was rejected; its twin,
    /// at the same seed, is offered the list without that cast.
    #[test]
    fn a_rejected_action_is_skipped_and_the_draw_is_the_filtered_lists() {
        let game = setup_basic_game();
        let first = ChoiceContext::new(ChoiceKind::PriorityAction);
        let options = priority_options();
        let filtered: Vec<ChoiceOption> =
            options.iter().enumerate().filter(|(i, _)| *i != 2).map(|(_, option)| option.clone()).collect();
        for seed in 0..64 {
            let (told, twin) = (RandomDecisionProvider::seeded(seed), RandomDecisionProvider::seeded(seed));
            assert_eq!(told.pick_n(&game, 0, &first, &options, (1, 1)), twin.pick_n(&game, 0, &first, &options, (1, 1)));
            let picked = told.pick_n(&game, 0, &re_ask(&options[2]), &options, (1, 1));
            let twin_picked = twin.pick_n(&game, 0, &first, &filtered, (1, 1));
            assert_ne!(picked, [2], "seed {seed}: the rejected cast was chosen again");
            assert_eq!(action(&options[picked[0]]), action(&filtered[twin_picked[0]]), "seed {seed}");
        }
    }

    /// A window's rejections add up, and a prompt that rejects nothing starts
    /// a new window, where every action may be chosen again.
    #[test]
    fn a_windows_rejections_add_up_and_a_new_window_forgets_them() {
        let game = setup_basic_game();
        let first = ChoiceContext::new(ChoiceKind::PriorityAction);
        let options = priority_options();
        let dp = RandomDecisionProvider::seeded(7);
        dp.pick_n(&game, 0, &first, &options, (1, 1));
        dp.pick_n(&game, 0, &re_ask(&options[1]), &options, (1, 1));
        dp.pick_n(&game, 0, &re_ask(&options[2]), &options, (1, 1));
        assert_eq!(dp.pick_n(&game, 0, &re_ask(&options[3]), &options, (1, 1)), [0], "every cast was rejected");

        let picks: Vec<usize> = (0..64).map(|_| dp.pick_n(&game, 0, &first, &options, (1, 1))[0]).collect();
        assert!(picks.iter().any(|&pick| pick != 0), "a new window offers the casts again: {picks:?}");
    }

    #[test]
    fn test_random_dp_pick_number_in_range() {
        let dp = RandomDecisionProvider::new();
        let game = setup_basic_game();
        let spell_id = crate::types::ids::new_object_id();
        let ctx = ChoiceContext::new(ChoiceKind::ChooseXValue { spell_id, x_count: 1 });
        let result = dp.pick_number(&game, 0, &ctx, 0, 10);
        assert!(result <= 10);
    }

    #[test]
    fn test_random_dp_allocate_sums_to_total() {
        let dp = RandomDecisionProvider::new();
        let game = setup_basic_game();
        let id_a = crate::types::ids::new_object_id();
        let id_b = crate::types::ids::new_object_id();
        let ctx = ChoiceContext::new(ChoiceKind::AssignCombatDamage { attacker_id: id_a });
        let buckets = vec![ChoiceOption::Object(id_a), ChoiceOption::Object(id_b)];
        let mins = vec![0, 0];
        let result = dp.allocate(&game, 0, &ctx, 5, &buckets, &mins, None);
        assert_eq!(result.len(), 2);
        assert_eq!(result.iter().sum::<u64>(), 5);
    }
}
