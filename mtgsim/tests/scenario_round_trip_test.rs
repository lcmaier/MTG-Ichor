//! The writer is the loader's test (`plans/setup-architecture.md` §4.3).
//! Seeded games played by the random agent; at every priority round's start
//! whose board the writer reports nothing for, the board is written, loaded,
//! and compared with the original over every written field (the text written
//! again) and every object's computed characteristics. Boards it skips are
//! counted by reason, which orders the vocabulary's next words.
//!
//! **Release for the sweep**, as CI's own step runs it (`cargo test --release
//! --test scenario_round_trip_test`): a debug build's layer-memo audit
//! re-walks every hit. Two games run in debug.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};

use mtgsim::cards::random_deck::random_deck;
use mtgsim::cards::registry::CardRegistry;
use mtgsim::engine::layers::compute::compute_characteristics;
use mtgsim::oracle::characteristics::has_summoning_sickness;
use mtgsim::scenario::Scenario;
use mtgsim::state::game::{Game, RandomStreams};
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::GameState;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, SeatMode};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[derive(Default)]
struct Tally {
    compared: usize,
    skipped: BTreeMap<String, usize>,
    mismatches: Vec<String>,
}

/// The random agent, checking each round start before it answers.
struct RoundTrip<'r> {
    agent: ManaWindowStop<RandomDecisionProvider>,
    registry: &'r CardRegistry,
    /// Registered names, longest first, to read a skip's reason without them.
    names: Vec<&'r str>,
    tally: RefCell<Tally>,
}

impl RoundTrip<'_> {
    fn check(&self, game: &GameState) {
        let written = Scenario::write(game);
        let mut tally = self.tally.borrow_mut();
        if !written.unwritten.is_empty() {
            let reasons: HashSet<String> = written.unwritten.iter().map(|line| self.reason(line)).collect();
            for reason in reasons {
                *tally.skipped.entry(reason).or_default() += 1;
            }
            return;
        }
        tally.compared += 1;
        let text = written.to_string();
        let loaded = match Scenario::parse(&text).and_then(|s| s.build(self.registry)).map(|built| built.game) {
            Ok(loaded) => loaded,
            Err(refusal) => return tally.mismatches.push(format!("refused: {refusal}\n{text}")),
        };
        let again = Scenario::write(&loaded.state).to_string();
        if again != text {
            return tally.mismatches.push(format!("written again, differs:\n{text}\n---\n{again}"));
        }
        let (before, after) = (places(game), places(&loaded.state));
        if before.len() != after.len() {
            return tally.mismatches.push(format!("{} objects became {}\n{text}", before.len(), after.len()));
        }
        for ((place, original), (_, rebuilt)) in before.iter().zip(&after) {
            let (was, is) = (frame(game, *original), frame(&loaded.state, *rebuilt));
            if was != is {
                tally.mismatches.push(format!("{place}: {was}\n   now {is}\n{text}"));
                return;
            }
        }
    }

    /// A report line with its card names and numbers taken out.
    fn reason(&self, line: &str) -> String {
        let mut reason = line.to_string();
        for name in &self.names {
            reason = reason.replace(name, "<card>");
        }
        let mut tagless = String::new();
        for (i, part) in reason.split('[').enumerate() {
            tagless.push_str(if i == 0 { part } else { part.split_once(']').map_or(part, |(_, rest)| rest) });
        }
        tagless.replace(" ,", ",").chars().map(|c| if c.is_ascii_digit() { 'N' } else { c }).collect()
    }
}

impl DecisionProvider for RoundTrip<'_> {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], bounds: (usize, usize)) -> Vec<usize> {
        if matches!(context.kind, ChoiceKind::PriorityAction) && game.priority_player == game.active_player {
            self.check(game);
        }
        self.agent.pick_n(game, player, context, options, bounds)
    }

    fn pick_number(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, min: u64, max: u64) -> u64 {
        self.agent.pick_number(game, player, context, min, max)
    }

    fn allocate(
        &self,
        game: &GameState,
        player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        mins: &[u64],
        maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.agent.allocate(game, player, context, total, buckets, mins, maxs)
    }

    fn choose_ordering(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        self.agent.choose_ordering(game, player, context, items)
    }

    fn seat_mode(&self, player: PlayerId) -> SeatMode {
        self.agent.seat_mode(player)
    }
}

/// Every object by its place: the battlefield in timestamp order, then each
/// player's library, hand and graveyard, then exile and the command zone.
fn places(state: &GameState) -> Vec<(String, ObjectId)> {
    let mut places: Vec<(String, ObjectId)> =
        state.battlefield_ids_ordered().into_iter().enumerate().map(|(i, id)| (format!("battlefield {i}"), id)).collect();
    for (p, player) in state.players.iter().enumerate() {
        for (zone, ids) in [("library", &player.library), ("hand", &player.hand), ("graveyard", &player.graveyard)] {
            places.extend(ids.iter().enumerate().map(|(i, id)| (format!("{zone} {p} {i}"), *id)));
        }
    }
    for (zone, ids) in [("exile", &state.exile), ("command", &state.command)] {
        places.extend(ids.iter().enumerate().map(|(i, id)| (format!("{zone} {i}"), *id)));
    }
    places
}

/// What the layer walk computes for `id`, with nothing that names an id: an
/// ability by its kind, since its id carries the object's and the rows'.
fn frame(state: &GameState, id: ObjectId) -> String {
    let Some(chars) = compute_characteristics(state, id) else { return "no frame".to_string() };
    let sorted = |mut words: Vec<String>| {
        words.sort();
        words.join(" ")
    };
    format!(
        "{} {:?} {:?} [{}] [{}] [{}] [{}] {:?}/{:?} loyalty {:?} controller {} abilities [{}] sick {}",
        chars.name,
        chars.mana_cost,
        chars.types,
        sorted(chars.colors.iter().map(|c| format!("{c:?}")).collect()),
        sorted(chars.subtypes.iter().map(|s| format!("{s:?}")).collect()),
        sorted(chars.supertypes.iter().map(|s| format!("{s:?}")).collect()),
        sorted(chars.keyword_flags.iter().map(|k| format!("{k:?}")).collect()),
        chars.power,
        chars.toughness,
        chars.loyalty,
        chars.controller,
        sorted(chars.abilities.iter().map(|a| format!("{:?}", a.ability_type)).collect()),
        has_summoning_sickness(state, id),
    )
}

/// Games at `seats` seats from `seeds`, each checked at every round start.
fn round_trip(seeds: std::ops::Range<u64>, seats: usize) -> Tally {
    let decks = CardRegistry::performance_pool();
    let registry = CardRegistry::default_registry();
    let mut names = registry.card_names();
    names.sort_by_key(|name| std::cmp::Reverse(name.len()));
    let mut tally = Tally::default();
    for seed in seeds {
        let mut deck_rng = StdRng::seed_from_u64(seed);
        let lists = (0..seats).map(|_| random_deck(&decks, &mut deck_rng, &[], 1, 60)).collect();
        let mut game = Game::new(GameConfig::unrestricted(), lists).unwrap();
        let streams = RandomStreams::from_seed(seed);
        game.reseed(streams.game);
        let checker = RoundTrip {
            agent: ManaWindowStop::new(RandomDecisionProvider::seeded(streams.agents)),
            registry: &registry,
            names: names.clone(),
            tally: RefCell::new(Tally::default()),
        };
        game.setup(&checker).unwrap();
        for _ in 0..60 {
            if game.is_over() {
                break;
            }
            game.run_turn(&checker).unwrap();
        }
        let played = checker.tally.into_inner();
        tally.compared += played.compared;
        tally.mismatches.extend(played.mismatches);
        for (reason, n) in played.skipped {
            *tally.skipped.entry(reason).or_default() += n;
        }
    }
    let skipped: usize = tally.skipped.values().sum();
    println!("{seats} seats: {} boards compared, {skipped} skipped, by reason:", tally.compared);
    let mut reasons: Vec<(&String, &usize)> = tally.skipped.iter().collect();
    reasons.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
    for (reason, n) in reasons {
        println!("  {n:5}  {reason}");
    }
    tally
}

fn assert_round_trips(tally: &Tally) {
    assert!(tally.mismatches.is_empty(), "{} boards came back different; the first:\n{}", tally.mismatches.len(), tally.mismatches[0]);
}

#[test]
fn two_written_boards_load_as_played_in_debug() {
    let tally = round_trip(0..2, 2);
    assert_round_trips(&tally);
    assert!(tally.compared > 0);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release only: CI runs it with --release")]
fn written_boards_load_as_they_were_played() {
    let two = round_trip(100..124, 2);
    let four = round_trip(200..208, 4);
    assert_round_trips(&two);
    assert_round_trips(&four);
    assert!(two.compared >= 200, "only {} boards compared at two seats", two.compared);
}
