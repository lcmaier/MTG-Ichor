//! Where a game began, as its record says it, and the game that builds.

use std::sync::Arc;

use crate::cards::registry::CardRegistry;
use crate::objects::card_data::CardData;
use crate::scenario::{BuiltScenario, Scenario, SetupDriver};
use crate::state::game::{Game, Halt, RandomStreams};
use crate::state::game_config::{GameConfig, MulliganRule};
use crate::state::game_state::GameResult;
use crate::ui::decision::DecisionProvider;
use crate::ui::replay::Replay;

use super::text::{LogError, Reader};

/// Where a game began, as its record says it.
#[derive(Debug, Clone, PartialEq)]
pub enum GameStart {
    /// Dealt decks: the seed the game's own randomness is drawn from
    /// (`RandomStreams::from_seed`), the configuration, and each seat's deck
    /// by name, in library order before the shuffle.
    Dealt { seed: u64, config: GameConfig, decks: Vec<Vec<String>> },
    /// A scenario: the file it was read from, the seed, and the file's text
    /// as it was read, so the record replays after the file changes.
    Scenario { path: String, seed: u64, text: String },
}

impl GameStart {
    /// The start's lines, after the record's format and engine.
    pub(super) fn lines(&self) -> Vec<String> {
        match self {
            GameStart::Dealt { seed, config, decks } => {
                // Every field named, so one the engine adds is written here or
                // said not to change play. The limits are checked as decks are
                // built, and a recorded deck was.
                let GameConfig {
                    starting_life,
                    starting_hand_size,
                    max_hand_size,
                    first_player_draws,
                    mulligan_rule,
                    deck_limits: _,
                } = config;
                let first_draw = match first_player_draws {
                    None => "rule",
                    Some(true) => "yes",
                    Some(false) => "no",
                };
                let mut lines = vec![
                    format!("seed {seed}"),
                    format!("life {starting_life}"),
                    format!("hand size {starting_hand_size}"),
                    format!("max hand size {max_hand_size}"),
                    format!("first player draws {first_draw}"),
                    format!("mulligans {}", mulligan_word(mulligan_rule)),
                ];
                lines.extend(decks.iter().enumerate().map(|(seat, deck)| format!("deck {seat} {}", deck.join(" | "))));
                lines
            }
            GameStart::Scenario { path, seed, text } => {
                let mut lines = vec![format!("scenario {path}"), format!("seed {seed}"), "begin scenario text".to_string()];
                lines.extend(text.lines().map(str::to_string));
                lines.push("end scenario text".to_string());
                lines
            }
        }
    }

    /// The start read back, each refusal naming its line.
    pub(super) fn read(reader: &mut Reader) -> Result<GameStart, LogError> {
        let (line, first) = reader.line("where the game began")?;
        if let Some(path) = first.strip_prefix("scenario ") {
            let seed = reader.number("seed ")?;
            reader.exactly("begin scenario text")?;
            let mut text = String::new();
            loop {
                let (_, line) = reader.line("`end scenario text`")?;
                if line == "end scenario text" {
                    return Ok(GameStart::Scenario { path: path.to_string(), seed, text });
                }
                text.push_str(line);
                text.push('\n');
            }
        }
        let seed = first.strip_prefix("seed ").and_then(|seed| seed.parse().ok()).ok_or_else(|| {
            LogError::at(line, format!("`{first}`: a game begins at `seed N` when dealt, or at `scenario <path>`"))
        })?;
        let starting_life = reader.number("life ")?;
        let starting_hand_size = reader.number("hand size ")?;
        let max_hand_size = reader.number("max hand size ")?;
        let (line, first_draw) = reader.after("first player draws ")?;
        let first_player_draws = match first_draw {
            "rule" => None,
            "yes" => Some(true),
            "no" => Some(false),
            other => return Err(LogError::at(line, format!("first player draws `{other}`: rule, yes or no"))),
        };
        let (line, mulligans) = reader.after("mulligans ")?;
        let mulligan_rule = [MulliganRule::None, MulliganRule::London, MulliganRule::Paris]
            .into_iter()
            .find(|rule| mulligan_word(rule) == mulligans)
            .ok_or_else(|| LogError::at(line, format!("mulligans `{mulligans}`: none, london or paris")))?;
        let mut decks = Vec::new();
        while reader.next_starts_with("deck ") {
            let (_, deck) = reader.after(&format!("deck {} ", decks.len()))?;
            decks.push(if deck.is_empty() { Vec::new() } else { deck.split(" | ").map(str::to_string).collect() });
        }
        let config = GameConfig {
            starting_life,
            starting_hand_size,
            max_hand_size,
            first_player_draws,
            mulligan_rule,
            deck_limits: GameConfig::unrestricted().deck_limits,
        };
        Ok(GameStart::Dealt { seed, config, decks })
    }

    /// The game this start builds, before anything is played: dealt decks
    /// before the shuffle and the opening hands, or a scenario's board and
    /// the setup actions its file names. A card the registry does not hold
    /// is refused by name.
    pub fn build(&self, registry: &CardRegistry) -> Result<BuiltStart, String> {
        match self {
            GameStart::Dealt { seed, config, decks } => {
                let created = |(seat, deck): (usize, &Vec<String>)| -> Result<Vec<Arc<CardData>>, String> {
                    deck.iter()
                        .map(|name| registry.create(name).map_err(|_| format!("deck {seat} names {name}, which is not registered")))
                        .collect()
                };
                let decks = decks.iter().enumerate().map(created).collect::<Result<Vec<_>, _>>()?;
                let mut game = Game::new(config.clone(), decks)?;
                game.reseed(RandomStreams::from_seed(*seed).game);
                Ok(BuiltStart::Dealt(game))
            }
            GameStart::Scenario { seed, text, .. } => {
                let scenario = Scenario { seed: *seed, ..Scenario::parse(text).map_err(|r| r.to_string())? };
                scenario.build(registry).map(BuiltStart::Scenario).map_err(|r| r.to_string())
            }
        }
    }
}

fn mulligan_word(rule: &MulliganRule) -> &'static str {
    match rule {
        MulliganRule::None => "none",
        MulliganRule::London => "london",
        MulliganRule::Paris => "paris",
    }
}

/// A game built from its start, to be played or replayed.
pub enum BuiltStart {
    /// Dealt decks, before the shuffle and the opening hands.
    Dealt(Game),
    /// A scenario's board, and the setup actions its file names.
    Scenario(BuiltScenario),
}

impl BuiltStart {
    pub fn game(&self) -> &Game {
        match self {
            BuiltStart::Dealt(game) | BuiltStart::Scenario(BuiltScenario { game, .. }) => game,
        }
    }

    pub fn game_mut(&mut self) -> &mut Game {
        match self {
            BuiltStart::Dealt(game) | BuiltStart::Scenario(BuiltScenario { game, .. }) => game,
        }
    }

    /// Play the game to its end with `seats` answering, under
    /// `Game::until_stopped`: a dealt game's opening hands first, or a
    /// scenario's setup actions through [`SetupDriver`], which stops the run
    /// naming a line it cannot play.
    pub fn play(&mut self, seats: &dyn DecisionProvider) -> Result<GameResult, Halt> {
        match self {
            BuiltStart::Dealt(game) => game.until_stopped(|game| {
                game.setup(seats)?;
                game.run(seats)
            }),
            BuiltStart::Scenario(BuiltScenario { game, setup }) => {
                let driver = SetupDriver::new(std::mem::take(setup), seats);
                game.until_stopped(|game| game.resume(&driver))
            }
        }
    }

    /// Replay the game, every answer from `replay`, under
    /// `Game::until_stopped`. A scenario's setup actions are replayed with
    /// the rest, since its record holds the answers the setup driver gave,
    /// so the driver does not run.
    pub fn replay(&mut self, replay: &Replay) -> Result<GameResult, Halt> {
        match self {
            BuiltStart::Dealt(game) => game.until_stopped(|game| {
                game.setup(replay)?;
                game.run(replay)
            }),
            BuiltStart::Scenario(BuiltScenario { game, .. }) => game.until_stopped(|game| game.resume(replay)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::random_deck::random_deck;
    use crate::ui::decision::ScriptedDecisionProvider;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn library(game: &Game, seat: usize) -> Vec<String> {
        game.state.players[seat].library.iter().map(|id| game.state.objects[id].card_data.name.clone()).collect()
    }

    /// A dealt start builds the game its dealer built: each library the
    /// same cards in the same order, which the same seed then shuffles alike.
    #[test]
    fn a_dealt_start_builds_the_game_its_dealer_built() {
        let pool = CardRegistry::performance_pool();
        let mut deck_rng = StdRng::seed_from_u64(7);
        let decks: Vec<Vec<Arc<CardData>>> = (0..2).map(|_| random_deck(&pool, &mut deck_rng, &[], 1, 60)).collect();
        let names = decks.iter().map(|deck| deck.iter().map(|card| card.name.clone()).collect()).collect();
        let mut dealt = Game::new(GameConfig::unrestricted(), decks).unwrap();
        dealt.reseed(RandomStreams::from_seed(7).game);
        let start = GameStart::Dealt { seed: 7, config: GameConfig::unrestricted(), decks: names };
        let mut built = start.build(&CardRegistry::default_registry()).unwrap();
        for game in [&mut dealt, built.game_mut()] {
            game.setup(&ScriptedDecisionProvider::new()).unwrap();
        }
        for seat in 0..2 {
            assert_eq!(library(built.game(), seat), library(&dealt, seat));
        }
    }

    /// A deck naming a card the registry lacks is refused by name.
    #[test]
    fn a_deck_naming_an_unregistered_card_is_refused() {
        let decks = vec![vec!["Forest".to_string()], vec!["Grizly Bears".to_string()]];
        let start = GameStart::Dealt { seed: 0, config: GameConfig::unrestricted(), decks };
        let refused = start.build(&CardRegistry::default_registry()).err();
        assert_eq!(refused.as_deref(), Some("deck 1 names Grizly Bears, which is not registered"));
    }
}
