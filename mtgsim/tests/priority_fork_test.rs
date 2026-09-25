//! A fork at a priority round start replays the game it was forked from.
//!
//! `codebase-state.md` item 40's invariant — no decision site may hold state
//! that changes the game's *outcome* and is not in `GameState` — has one
//! customer and it is not here yet: a search harness clones the state at a
//! decision point, plays the branch forward, and comes back to try the other
//! option. This is that clone, run as an assertion. Clone `GameState` and the
//! provider at every priority prompt a round can be resumed from, play the
//! branch to the same turn limit the original stopped at, and require the two
//! rendered event logs to be equal **verbatim, ids included** — the branch is a
//! replay of the game it came from, or the state was not a complete description
//! of that game.
//!
//! **Both streams are cloned, and that is a decision** (item 41).
//! `RandomDecisionProvider` owns its `StdRng` outside `GameState`, so a branch
//! carrying only the state re-randomizes every choice from the fork on and this
//! test would be asserting nothing. Replay-exact is the default; a search that
//! wants determinization reseeds the *branch's* provider, never the state's
//! `rng`.
//!
//! **What "resumable" means here.** `run_priority_round` begins every round at
//! the active player, and `consecutive_passes` and the retry `blacklist` are
//! loop locals — so a prompt is resumable exactly when `priority_player` is the
//! active player, which the state does carry. That is the round start and the
//! active player's own re-asks after a rejected action; the other seats'
//! prompts and CR 514.3a's cleanup re-loop are item 140's, and this test gains
//! them when it lands.
//!
//! **The re-asks are why this test could not be green before A4h.** Two things
//! made the offered list a function of the game's history rather than of its
//! state, and every one of the 119 branches that failed to replay across this
//! file's 192-game sweep was one of them:
//!
//! - `all_candidates` was enumerated once per priority window and re-offered
//!   minus the blacklist. A rejected cast's mana abilities stay activated (CR
//!   732.1 — the reversal is the player's option and the engine never offers
//!   it, item 72), so the re-ask offered casts no enumeration of *that* board
//!   would (item 139).
//! - `activatable_abilities` never checked target legality, so an Equipment's
//!   equip ability was offered on a creatureless board, rejected, and
//!   blacklisted — and a fresh enumeration offered it right back, which is the
//!   blacklist deciding the prompt (item 150).

use std::cell::RefCell;
use std::sync::Arc;

use mtgsim::cards::registry::CardRegistry;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::{GameState, PhaseType, StepType};
use mtgsim::types::card_types::CardType;
use mtgsim::types::ids::PlayerId;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::DecisionProvider;
use mtgsim::ui::random::RandomDecisionProvider;

/// Turns played before the harness stops, original and branch alike.
///
/// A cap rather than a game's natural end: these boards go long, and the
/// question is about the fork, not about who wins. Both sides stop at the same
/// turn number, so a game that would have run past it is compared over the
/// turns they both played.
const LAST_TURN: u32 = 7;

/// Share of a deck's slots that are not lands — `fuzz_games`' 60, so the boards
/// cast spells and run out of mana, which is what makes an action get rejected
/// at all.
const NONLAND_PERCENT: usize = 60;

/// One seed per board, each picked out of `fork_sweep_over_many_seeds`'s 64 to
/// **catch both mechanisms** against the pre-A4h tree. Without item 150's
/// target check these three diverge 3, 2 and 1 times; without item 139's fresh
/// enumeration, 1, 2 and 2. A seed that caught neither would leave the test
/// asserting only that nothing else broke.
const TWO_SEAT_SEED: u64 = 64;
const FOUR_SEAT_SEED: u64 = 52;
const COMMANDER_SEED: u64 = 6;

// ---------------------------------------------------------------------------
// The fork recorder
// ---------------------------------------------------------------------------

/// One branch point: the game, the provider that was about to answer, and the
/// list it was about to answer *about*.
struct Fork {
    state: GameState,
    dp: RandomDecisionProvider,
    /// Which prompt of the original run this was, for the failure message.
    prompt: usize,
    offered: Vec<String>,
}

/// Wraps a `RandomDecisionProvider` and watches the prompts going past.
///
/// Two roles, because a branch wants half of what the original run wants: over
/// the original it snapshots the game at every resumable prompt (`forks`); over
/// a branch it records the first priority list it is shown, which is what says
/// whether a divergence was announced at the fork or went silent past it.
struct ForkRecorder {
    inner: RandomDecisionProvider,
    snapshot: bool,
    forks: RefCell<Vec<Fork>>,
    prompts: RefCell<usize>,
    first_priority: RefCell<Option<Vec<String>>>,
}

impl ForkRecorder {
    fn watching(inner: RandomDecisionProvider) -> Self {
        ForkRecorder {
            inner,
            snapshot: true,
            forks: RefCell::new(Vec::new()),
            prompts: RefCell::new(0),
            first_priority: RefCell::new(None),
        }
    }

    fn replaying(inner: RandomDecisionProvider) -> Self {
        ForkRecorder {
            snapshot: false,
            ..Self::watching(inner)
        }
    }
}

fn describe(options: &[ChoiceOption]) -> Vec<String> {
    options.iter().map(|o| format!("{o:?}")).collect()
}

/// Is this prompt one [`Game::resume_turn_at_priority`] can take over from?
///
/// Read off `GameState` alone, because that is the whole claim: a harness that
/// holds a clone and nothing else has to be able to tell. `priority_player ==
/// active_player` is `consecutive_passes == 0` — a player is asked at most once
/// per round, in turn order from the active player — and it holds for that
/// player's re-asks too, which is where item 139 lived.
fn is_resumable(game: &GameState, ctx: &ChoiceContext) -> bool {
    matches!(ctx.kind, ChoiceKind::PriorityAction)
        && game.priority_player == game.active_player
        // CR 800.4j: with the active player gone the round starts elsewhere, so
        // `priority_player == active_player` stops meaning "no passes yet".
        && game.in_game(game.active_player)
        // CR 514.3a's re-loop is nested in `Game::run_turn`'s cleanup branch,
        // not in the step drainer — item 140.
        && !matches!(
            (game.phase.phase_type, game.phase.step),
            (PhaseType::Ending, Some(StepType::Cleanup))
        )
}

impl DecisionProvider for ForkRecorder {
    fn pick_n(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        let prompt = *self.prompts.borrow();
        *self.prompts.borrow_mut() = prompt + 1;
        if matches!(context.kind, ChoiceKind::PriorityAction)
            && self.first_priority.borrow().is_none()
        {
            *self.first_priority.borrow_mut() = Some(describe(options));
        }
        if self.snapshot && is_resumable(game, context) {
            self.forks.borrow_mut().push(Fork {
                state: game.clone(),
                dp: self.inner.clone(),
                prompt,
                offered: describe(options),
            });
        }
        self.inner.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        min: u64,
        max: u64,
    ) -> u64 {
        *self.prompts.borrow_mut() += 1;
        self.inner.pick_number(game, player, context, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        *self.prompts.borrow_mut() += 1;
        self.inner.allocate(
            game,
            player,
            context,
            total,
            buckets,
            per_bucket_mins,
            per_bucket_maxs,
        )
    }

    fn choose_ordering(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize> {
        *self.prompts.borrow_mut() += 1;
        self.inner.choose_ordering(game, player, context, items)
    }
}

// ---------------------------------------------------------------------------
// Boards
// ---------------------------------------------------------------------------

/// One deck of `size` cards from `registry`: the nonlands in name order, then
/// the lands, cycling each list to fill its share of the slots.
///
/// Not `fuzz_games::random_deck` and not trying to be — that one draws from an
/// `StdRng` a test cannot reach, and the randomness this test needs is the
/// in-game shuffle, which `Game::reseed` gives it. What it does copy is the one
/// property that matters here: enough lands to cast things and not enough to
/// cast everything, so actions get rejected.
fn build_deck(registry: &CardRegistry, size: usize) -> Vec<Arc<CardData>> {
    let names = registry.card_names();
    let built: Vec<(&str, Arc<CardData>)> = names
        .iter()
        .filter_map(|n| registry.create(n).ok().map(|c| (*n, c)))
        .collect();
    let (lands, nonlands): (Vec<_>, Vec<_>) = built
        .iter()
        .partition(|(_, c)| c.types.contains(&CardType::Land));

    let nonland_slots = size * NONLAND_PERCENT / 100;
    let mut deck: Vec<Arc<CardData>> = Vec::with_capacity(size);
    for (name, _) in nonlands.iter().cycle().take(nonland_slots) {
        deck.push(registry.create(name).expect("registered"));
    }
    for (name, _) in lands.iter().cycle().take(size - nonland_slots) {
        deck.push(registry.create(name).expect("registered"));
    }
    deck
}

/// A board to fork on, named the way the harness names it. `performance` and
/// `stress` are `engineering-practices.md` §3's two pools; Commander scale is
/// `fuzz_games --deck-size 100 --life 40 --players 4`.
struct Board {
    name: &'static str,
    registry: CardRegistry,
    players: usize,
    deck_size: usize,
    life: i64,
}

fn performance_board(players: usize) -> Board {
    Board {
        name: "performance",
        registry: CardRegistry::performance_pool(),
        players,
        deck_size: 60,
        life: 20,
    }
}

fn stress_board(players: usize) -> Board {
    Board {
        name: "stress",
        registry: CardRegistry::default_registry(),
        players,
        deck_size: 60,
        life: 20,
    }
}

fn commander_board() -> Board {
    Board {
        name: "commander scale",
        registry: CardRegistry::performance_pool(),
        players: 4,
        deck_size: 100,
        life: 40,
    }
}

impl Board {
    fn config(&self) -> GameConfig {
        let mut config = GameConfig::test();
        config.starting_life = self.life;
        config
    }

    fn new_game(&self) -> Game {
        let deck = build_deck(&self.registry, self.deck_size);
        let mut game = Game::new(self.config(), vec![deck; self.players]).expect("game creation");
        game.state.record_events();
        game
    }
}

/// Play until the game ends or turn `LAST_TURN` is over.
///
/// Driven by `turn_number` rather than a loop counter so a branch that resumes
/// mid-turn stops where the original did. It terminates: `run_turn` returns only
/// once it has produced a turn or ended the game.
fn play_out(game: &mut Game, dp: &dyn DecisionProvider) {
    while !game.is_over() && game.state.turn_number <= LAST_TURN {
        game.run_turn(dp).expect("turn");
    }
}

// ---------------------------------------------------------------------------
// The assertion
// ---------------------------------------------------------------------------

/// How one seeded game's forks came out.
struct Run {
    /// Branch points taken — "every round-start fork", counted.
    forks: usize,
    /// One line per branch that did not replay its original, in fork order.
    diverged: Vec<String>,
}

/// Play one seeded game on `board`, fork it at every resumable priority prompt,
/// and replay each branch to the same turn limit.
///
/// Collects the divergences rather than panicking at the first: what a tree
/// missing one of the two fixes owes the record is a *count*, and one message
/// cannot say whether the mechanism fired once or eighty times.
fn fork_every_round_start(board: &Board, seed: u64) -> Run {
    let mut game = board.new_game();
    game.reseed(seed);
    let recorder = ForkRecorder::watching(RandomDecisionProvider::seeded(seed));
    game.setup(&recorder).expect("setup");
    play_out(&mut game, &recorder);

    let original = game.event_log_snapshot();
    assert!(
        original.len() > 50,
        "{} seed {seed}: game was too short to prove anything ({} events)",
        board.name,
        original.len()
    );

    let forks = recorder.forks.into_inner();
    assert!(
        !forks.is_empty(),
        "{} seed {seed}: no resumable priority prompt in the whole game",
        board.name
    );

    let config = board.config();
    let mut diverged = Vec::new();
    for fork in &forks {
        let mut branch = Game {
            state: fork.state.clone(),
            config: config.clone(),
        };
        let replay = ForkRecorder::replaying(fork.dp.clone());
        branch
            .resume_turn_at_priority(&replay)
            .expect("resumed turn");
        play_out(&mut branch, &replay);

        let replayed = branch.event_log_snapshot();
        if replayed == original {
            continue;
        }
        let at = replayed
            .iter()
            .zip(original.iter())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| replayed.len().min(original.len()));
        // Announced at the fork or silent past it: the first says the branch
        // could not rebuild the prompt, the second that it rebuilt the prompt
        // and the game still went elsewhere. Only the first has ever been seen,
        // and which of the two it is is the whole diagnosis.
        let same_prompt = replay.first_priority.borrow().as_deref() == Some(&fork.offered[..]);
        diverged.push(format!(
            "{} seed {seed}: the branch forked at prompt {} played a different game \
             ({}). First difference at event {at} of {}/{} (branch/original): \
             branch {:?}, original {:?}. Offered at the fork: {:?}; offered to the \
             branch: {:?}",
            board.name,
            fork.prompt,
            if same_prompt {
                "same list at the fork, so it diverged later"
            } else {
                "a different list at the fork itself"
            },
            replayed.len(),
            original.len(),
            replayed.get(at),
            original.get(at),
            fork.offered,
            replay.first_priority.borrow(),
        ));
    }

    Run {
        forks: forks.len(),
        diverged,
    }
}

/// Every branch of every seed replays its original. Returns the branch count,
/// which the callers assert is a board still playing Magic.
fn every_fork_replays(board: &Board, seeds: &[u64]) -> usize {
    let mut forks = 0;
    let mut diverged: Vec<String> = Vec::new();
    for &seed in seeds {
        let run = fork_every_round_start(board, seed);
        forks += run.forks;
        diverged.extend(run.diverged);
    }
    assert!(
        diverged.is_empty(),
        "{} of {forks} branches did not replay the game they were forked from.\n{}",
        diverged.len(),
        diverged.join("\n"),
    );
    forks
}

/// Two seats on the `performance` pool — §3's timing board, and the one the A/B
/// reads.
#[test]
fn a_fork_at_a_round_start_replays_the_game_at_two_seats() {
    let forks = every_fork_replays(&performance_board(2), &[TWO_SEAT_SEED]);
    assert!(forks > 40, "only {forks} forks — the board stopped playing");
}

/// Four seats on `stress` — every registered card, and the table size where a
/// round passes over a player who has left (CR 800.4j).
#[test]
fn a_fork_at_a_round_start_replays_the_game_at_four_seats() {
    let forks = every_fork_replays(&stress_board(4), &[FOUR_SEAT_SEED]);
    assert!(forks > 40, "only {forks} forks — the board stopped playing");
}

/// Commander scale: four seats, 100-card decks, 40 life. The board item 138
/// reads the ratchet on, and the one whose object count is v1's.
#[test]
fn a_fork_at_a_round_start_replays_the_game_at_commander_scale() {
    let forks = every_fork_replays(&commander_board(), &[COMMANDER_SEED]);
    assert!(forks > 40, "only {forks} forks — the board stopped playing");
}

/// The guard on the three above: a fork test passes trivially if a branch could
/// not have diverged by *any* means, so this pins that the provider's stream is
/// what carries the game — a branch reseeded instead of cloned plays a
/// different one, which is also the shape a determinizing search would use.
#[test]
fn a_branch_that_reseeds_its_provider_is_not_a_replay() {
    let board = performance_board(2);
    let mut game = board.new_game();
    game.reseed(TWO_SEAT_SEED);
    let recorder = ForkRecorder::watching(RandomDecisionProvider::seeded(TWO_SEAT_SEED));
    game.setup(&recorder).expect("setup");
    play_out(&mut game, &recorder);
    let original = game.event_log_snapshot();

    let forks = recorder.forks.into_inner();
    let early = forks.first().expect("at least one fork");
    let mut branch = Game {
        state: early.state.clone(),
        config: board.config(),
    };
    let reseeded = RandomDecisionProvider::seeded(0xDEAD_BEEF);
    branch
        .resume_turn_at_priority(&reseeded)
        .expect("resumed turn");
    play_out(&mut branch, &reseeded);

    assert_ne!(
        branch.event_log_snapshot(),
        original,
        "a branch whose provider was reseeded replayed the game anyway — the \
         provider is not making the choices"
    );
}

/// The instrument the three board tests are one sitting of, kept because item
/// 140 inherits the question and will want the same reading.
///
/// 64 seeds on each of the three boards. **Release, or it is minutes:**
///
/// ```text
/// cargo test --release --test priority_fork_test -- --ignored --nocapture fork_sweep
/// ```
///
/// Read 2026-09-16 at four trees, 192 games each: `main` 119 diverging branches
/// over 69 of the games; item 139's fix alone 38 over 22; item 150's alone 92
/// over 54; **both, 0 of 13,530 branches**.
#[test]
#[ignore = "a sweep, not a gate — the three board tests are the gate"]
fn fork_sweep_over_many_seeds() {
    let mut forks = 0;
    let mut diverged = 0;
    for board in [performance_board(2), stress_board(4), commander_board()] {
        for seed in 1u64..=64 {
            let run = fork_every_round_start(&board, seed);
            eprintln!(
                "{} seed {seed}: {} forks, {} did not replay",
                board.name,
                run.forks,
                run.diverged.len()
            );
            for line in &run.diverged {
                eprintln!("  {line}");
            }
            forks += run.forks;
            diverged += run.diverged.len();
        }
    }
    eprintln!("{forks} forks over 192 games, {diverged} did not replay");
    assert_eq!(diverged, 0);
}
