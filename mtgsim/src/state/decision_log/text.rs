//! A record's text, written and read: the format and the engine, the start,
//! an answer a line, and the outcome.
//!
//! ```text
//! decision log 1
//! engine 1de83d3a1b2c
//! seed 41
//! …the start's lines (`GameStart`)…
//! answer 1 [turn 1, upkeep] player 0 PriorityAction forced picks pass
//! answer 2 [turn 1, precombat main] player 0 PriorityAction picks cast Lightning Bolt (#12)
//! answer 3 [turn 1, precombat main] player 0 SelectRecipients picks player 1
//! outcome player 0 wins
//! ```

use std::fmt;
use std::str::FromStr;

use crate::scenario::position_word;
use crate::state::game_state::GameState;
use crate::state::trace::COMMIT;
use crate::types::ids::PlayerId;
use crate::ui::choice_types::logged_choice;

use super::hook::{LoggedAnswer, LoggedDecision};
use super::start::GameStart;

/// A record's first line: the format and its version. The version goes up
/// when an older reader could not read the new text, which the pinned record
/// in this module's tests shows in review.
pub const FORMAT: &str = "decision log 1";

/// Between the options an answer names; no card name holds a `|`
/// (`setup-architecture.md` §4).
const BETWEEN: &str = " | ";

/// A record read back: what wrote it, where the game began, every answer,
/// and how it ended if it has.
#[derive(Debug, Clone, PartialEq)]
pub struct Log {
    /// The commit of the engine that wrote it (`state::trace::COMMIT`), which
    /// a reader reports and never refuses on (`setup-architecture.md` §7.2,
    /// decision 6).
    pub engine: String,
    pub start: GameStart,
    pub answers: Vec<AnswerLine>,
    /// `None` while the game went on when the record stopped.
    pub outcome: Option<Outcome>,
}

/// The lines a record opens with, as this engine writes them: its format, the
/// engine's commit, and where the game began.
pub fn opening(start: &GameStart) -> Vec<String> {
    opened_by(COMMIT, start)
}

fn opened_by(engine: &str, start: &GameStart) -> Vec<String> {
    let mut lines = vec![FORMAT.to_string(), format!("engine {engine}")];
    lines.extend(start.lines());
    lines
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in opened_by(&self.engine, &self.start) {
            writeln!(f, "{line}")?;
        }
        for answer in &self.answers {
            writeln!(f, "{answer}")?;
        }
        match &self.outcome {
            Some(outcome) => writeln!(f, "{outcome}"),
            None => Ok(()),
        }
    }
}

/// One answer, as its line says it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerLine {
    /// Its number in the record, from 1, which a divergence names.
    pub number: usize,
    pub turn: u32,
    /// The step as a scenario spells it ([`position_word`]).
    pub step: String,
    pub player: PlayerId,
    /// The kind as [`crate::ui::choice_types::ChoiceKind::as_str`] spells it.
    pub kind: String,
    /// The question had one legal answer.
    pub forced: bool,
    pub chosen: Chosen,
}

/// What an answer chose, each option as `logged_choice` names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chosen {
    Picks(Vec<String>),
    Number(u64),
    /// Each bucket's amount, in the buckets' order.
    Allocation(Vec<(u64, String)>),
    /// The items, first first.
    Order(Vec<String>),
}

impl AnswerLine {
    /// `decision` as the game's `number`th answer, at the turn and step the
    /// game is at as it asks.
    pub fn of(game: &GameState, number: usize, decision: &LoggedDecision) -> AnswerLine {
        let LoggedDecision { player, kind, options, answer, forced } = *decision;
        let named = |index: usize| logged_choice(options, game, index);
        let chosen = match answer {
            LoggedAnswer::Picks(picks) => Chosen::Picks(picks.iter().map(|&i| named(i)).collect()),
            LoggedAnswer::Number(number) => Chosen::Number(number),
            LoggedAnswer::Allocation(amounts) => {
                Chosen::Allocation(amounts.iter().enumerate().map(|(i, &amount)| (amount, named(i))).collect())
            }
            LoggedAnswer::Order(order) => Chosen::Order(order.iter().map(|&i| named(i)).collect()),
        };
        AnswerLine {
            number,
            turn: game.turn_number,
            step: position_word(game.phase),
            player,
            kind: kind.as_str().to_string(),
            forced,
            chosen,
        }
    }

    /// One `answer` line read back, or what is wrong with it.
    pub fn read(text: &str) -> Result<AnswerLine, String> {
        let rest = text.strip_prefix("answer ").ok_or("an answer line begins `answer`")?;
        let (number, rest) = rest.split_once(" [turn ").ok_or("`[turn` after the answer's number")?;
        let number = parsed(number, "the answer's number")?;
        let (turn, rest) = rest.split_once(", ").ok_or("`, ` after the turn")?;
        let turn = parsed(turn, "the turn")?;
        let (step, rest) = rest.split_once("] player ").ok_or("`] player` after the step")?;
        let (player, rest) = rest.split_once(' ').ok_or("the kind after the player")?;
        let player = parsed(player, "the player")?;
        let (kind, rest) = rest.split_once(' ').ok_or("the answer after the kind")?;
        let (forced, rest) = match rest.strip_prefix("forced ") {
            Some(rest) => (true, rest),
            None => (false, rest),
        };
        let (shape, items) = rest.split_once(' ').unwrap_or((rest, ""));
        let listed = |items: &str| -> Vec<String> {
            if items.is_empty() { Vec::new() } else { items.split(BETWEEN).map(str::to_string).collect() }
        };
        let chosen = match shape {
            "picks" => Chosen::Picks(listed(items)),
            "number" => Chosen::Number(parsed(items, "the number")?),
            "allocation" => Chosen::Allocation(
                listed(items)
                    .iter()
                    .map(|item| {
                        let (amount, option) = item.split_once(' ').ok_or(format!("`{item}`: an amount, then a bucket"))?;
                        Ok((parsed(amount, "a bucket's amount")?, option.to_string()))
                    })
                    .collect::<Result<_, String>>()?,
            ),
            "order" => Chosen::Order(listed(items)),
            other => return Err(format!("`{other}` is no answer's shape: picks, number, allocation or order")),
        };
        Ok(AnswerLine { number, turn, step: step.to_string(), player, kind: kind.to_string(), forced, chosen })
    }
}

impl fmt::Display for AnswerLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let AnswerLine { number, turn, step, player, kind, forced, chosen } = self;
        write!(f, "answer {number} [turn {turn}, {step}] player {player} {kind}")?;
        if *forced {
            f.write_str(" forced")?;
        }
        let listed = |items: Vec<String>| if items.is_empty() { String::new() } else { format!(" {}", items.join(BETWEEN)) };
        match chosen {
            Chosen::Picks(options) => write!(f, " picks{}", listed(options.clone())),
            Chosen::Number(number) => write!(f, " number {number}"),
            Chosen::Allocation(amounts) => {
                write!(f, " allocation{}", listed(amounts.iter().map(|(amount, bucket)| format!("{amount} {bucket}")).collect()))
            }
            Chosen::Order(items) => write!(f, " order{}", listed(items.clone())),
        }
    }
}

/// How a game ended, or why the run recording it did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Won(PlayerId),
    Draw,
    /// The run's own error.
    Error(String),
    /// A provider stopped the run (`ui::decision::Stop`).
    Stopped(String),
}

impl Outcome {
    fn read(text: &str) -> Result<Outcome, String> {
        let rest = text.strip_prefix("outcome ").ok_or_else(|| format!("`{text}`: an answer line, or the outcome"))?;
        if rest == "draw" {
            return Ok(Outcome::Draw);
        }
        if let Some(error) = rest.strip_prefix("error ") {
            return Ok(Outcome::Error(error.to_string()));
        }
        if let Some(why) = rest.strip_prefix("stopped ") {
            return Ok(Outcome::Stopped(why.to_string()));
        }
        let winner = rest.strip_prefix("player ").and_then(|rest| rest.strip_suffix(" wins")).ok_or_else(|| {
            format!("outcome `{rest}`: `player N wins`, `draw`, `error …` or `stopped …`")
        })?;
        Ok(Outcome::Won(parsed(winner, "the winner")?))
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // One line a record entry, whatever an error's text holds.
        let one_line = |text: &str| text.replace('\n', " ");
        match self {
            Outcome::Won(player) => write!(f, "outcome player {player} wins"),
            Outcome::Draw => write!(f, "outcome draw"),
            Outcome::Error(error) => write!(f, "outcome error {}", one_line(error)),
            Outcome::Stopped(why) => write!(f, "outcome stopped {}", one_line(why)),
        }
    }
}

/// Why a record does not read, and its line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogError {
    pub line: usize,
    pub why: String,
}

impl LogError {
    pub(super) fn at(line: usize, why: impl Into<String>) -> LogError {
        LogError { line, why: why.into() }
    }
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.why)
    }
}

/// A record read back from its text.
pub fn read(text: &str) -> Result<Log, LogError> {
    let mut reader = Reader { lines: text.lines().collect(), next: 0 };
    let (line, format) = reader.line("the record's format")?;
    if format != FORMAT {
        let why = match format.strip_prefix("decision log ") {
            Some(version) => format!("format {version}, and this engine reads `{FORMAT}`"),
            None => format!("`{format}`: a record of play opens with `{FORMAT}`; one written before records named their format does not"),
        };
        return Err(LogError::at(line, why));
    }
    let (_, engine) = reader.after("engine ")?;
    let start = GameStart::read(&mut reader)?;
    let mut answers: Vec<AnswerLine> = Vec::new();
    while reader.next_starts_with("answer ") {
        let (line, text) = reader.line("an answer")?;
        let answer = AnswerLine::read(text).map_err(|why| LogError::at(line, why))?;
        if answer.number != answers.len() + 1 {
            return Err(LogError::at(line, format!("answer {} follows answer {}", answer.number, answers.len())));
        }
        answers.push(answer);
    }
    let outcome = match reader.optional() {
        Some((line, text)) => Some(Outcome::read(text).map_err(|why| LogError::at(line, why))?),
        None => None,
    };
    if let Some((line, text)) = reader.optional() {
        return Err(LogError::at(line, format!("`{text}` after the record's outcome")));
    }
    Ok(Log { engine: engine.to_string(), start, answers, outcome })
}

fn parsed<T: FromStr>(text: &str, what: &str) -> Result<T, String> {
    text.parse().map_err(|_| format!("{what} `{text}` is not a number"))
}

/// A record's lines being read, numbered from 1.
pub(super) struct Reader<'t> {
    lines: Vec<&'t str>,
    next: usize,
}

impl<'t> Reader<'t> {
    /// The next line and its number, or a refusal naming what was expected.
    pub(super) fn line(&mut self, expected: &str) -> Result<(usize, &'t str), LogError> {
        self.optional().ok_or_else(|| LogError::at(self.lines.len() + 1, format!("the record ends where {expected} was expected")))
    }

    fn optional(&mut self) -> Option<(usize, &'t str)> {
        let line = self.lines.get(self.next).copied()?;
        self.next += 1;
        Some((self.next, line))
    }

    pub(super) fn next_starts_with(&self, head: &str) -> bool {
        self.lines.get(self.next).is_some_and(|line| line.starts_with(head))
    }

    /// The next line's text after `head`.
    pub(super) fn after(&mut self, head: &str) -> Result<(usize, &'t str), LogError> {
        let (line, text) = self.line(&format!("`{}`", head.trim_end()))?;
        let rest = text.strip_prefix(head).ok_or_else(|| LogError::at(line, format!("`{text}` where `{}` was expected", head.trim_end())))?;
        Ok((line, rest))
    }

    /// The next line, which must be `text`.
    pub(super) fn exactly(&mut self, text: &str) -> Result<(), LogError> {
        let (line, found) = self.line(&format!("`{text}`"))?;
        if found != text {
            return Err(LogError::at(line, format!("`{found}` where `{text}` was expected")));
        }
        Ok(())
    }

    /// The number after `head` on the next line.
    pub(super) fn number<T: FromStr>(&mut self, head: &str) -> Result<T, LogError> {
        let (line, text) = self.after(head)?;
        parsed(text, head.trim_end()).map_err(|why| LogError::at(line, why))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::game_config::GameConfig;

    fn dealt() -> GameStart {
        let decks = vec![vec!["Forest".to_string(), "Grizzly Bears".to_string()], vec!["Mountain".to_string(), "Lightning Bolt".to_string()]];
        GameStart::Dealt { seed: 41, config: GameConfig::unrestricted(), decks }
    }

    fn answer(number: usize, step: &str, player: PlayerId, kind: &str, forced: bool, chosen: Chosen) -> AnswerLine {
        AnswerLine { number, turn: 1, step: step.to_string(), player, kind: kind.to_string(), forced, chosen }
    }

    fn lines(lines: &[&str]) -> String {
        lines.iter().map(|line| format!("{line}\n")).collect()
    }

    /// A record's whole text, pinned: a change here is a change to the
    /// format, and its version goes up when an older reader could not read
    /// the new text.
    #[test]
    fn a_record_s_text_is_pinned() {
        let named = |names: &[&str]| names.iter().map(|name| name.to_string()).collect::<Vec<_>>();
        let log = Log {
            engine: "1de83d3a1b2c".to_string(),
            start: dealt(),
            answers: vec![
                answer(1, "upkeep", 0, "PriorityAction", true, Chosen::Picks(named(&["pass"]))),
                answer(2, "precombat main", 0, "PriorityAction", false, Chosen::Picks(named(&["cast Lightning Bolt (#12)"]))),
                answer(3, "precombat main", 0, "SelectRecipients", false, Chosen::Picks(named(&["player 1", "Grizzly Bears (#7)"]))),
                answer(4, "precombat main", 0, "ChooseAdditionalCosts", false, Chosen::Picks(Vec::new())),
                answer(5, "precombat main", 0, "ChooseXValue", false, Chosen::Number(3)),
                answer(6, "combat damage", 0, "AssignCombatDamage", false, Chosen::Allocation(vec![
                    (2, "Wall of Stone (#8)".to_string()),
                    (1, "Hill Giant (#10)".to_string()),
                ])),
                answer(7, "end", 1, "OrderTriggers", false, Chosen::Order(named(&["Soul Warden (#31)", "Soul Warden (#32)"]))),
            ],
            outcome: Some(Outcome::Won(1)),
        };
        let text = lines(&[
            "decision log 1",
            "engine 1de83d3a1b2c",
            "seed 41",
            "life 20",
            "hand size 7",
            "max hand size 7",
            "first player draws rule",
            "mulligans none",
            "deck 0 Forest | Grizzly Bears",
            "deck 1 Mountain | Lightning Bolt",
            "answer 1 [turn 1, upkeep] player 0 PriorityAction forced picks pass",
            "answer 2 [turn 1, precombat main] player 0 PriorityAction picks cast Lightning Bolt (#12)",
            "answer 3 [turn 1, precombat main] player 0 SelectRecipients picks player 1 | Grizzly Bears (#7)",
            "answer 4 [turn 1, precombat main] player 0 ChooseAdditionalCosts picks",
            "answer 5 [turn 1, precombat main] player 0 ChooseXValue number 3",
            "answer 6 [turn 1, combat damage] player 0 AssignCombatDamage allocation 2 Wall of Stone (#8) | 1 Hill Giant (#10)",
            "answer 7 [turn 1, end] player 1 OrderTriggers order Soul Warden (#31) | Soul Warden (#32)",
            "outcome player 1 wins",
        ]);
        assert_eq!(log.to_string(), text);
        assert_eq!(read(&text), Ok(log));
    }

    /// A scenario's start reads back as written, its text verbatim, and a
    /// record that stopped reads back without an outcome or with its stop.
    #[test]
    fn a_scenario_start_reads_back_as_written() {
        let text = "turn 2\nhand 0: Lightning Bolt\n".to_string();
        let start = GameStart::Scenario { path: "boards/x/x.scenario".to_string(), seed: 5, text };
        let stopped = Outcome::Stopped("line 3, `then: player 0 casts Lightning Bolt`: a reason".to_string());
        for outcome in [None, Some(stopped), Some(Outcome::Draw)] {
            let log = Log { engine: "unknown".to_string(), start: start.clone(), answers: Vec::new(), outcome };
            assert_eq!(read(&log.to_string()), Ok(log));
        }
    }

    /// A record that does not read names its line and what is wrong.
    #[test]
    fn a_record_that_does_not_read_names_its_line() {
        let refused = |text: &str| read(text).expect_err(text);
        assert_eq!(
            refused("seed 41\n"),
            LogError::at(1, "`seed 41`: a record of play opens with `decision log 1`; one written before records named their format does not")
        );
        assert_eq!(refused("decision log 2\n").why, "format 2, and this engine reads `decision log 1`");
        let open = lines(&opened_by("x", &dealt()).iter().map(String::as_str).collect::<Vec<_>>());
        let after_opening = |rest: &str| refused(&format!("{open}{rest}\n"));
        assert_eq!(after_opening("answer 2 [turn 1, upkeep] player 0 PriorityAction forced picks pass"), LogError::at(11, "answer 2 follows answer 0"));
        assert_eq!(after_opening("answer 1 [turn one, upkeep] player 0 PriorityAction picks pass").why, "the turn `one` is not a number");
        assert_eq!(
            after_opening("answer 1 [turn 1, upkeep] player 0 PriorityAction chooses pass").why,
            "`chooses` is no answer's shape: picks, number, allocation or order"
        );
        assert_eq!(after_opening("outcome draw\noutcome draw"), LogError::at(12, "`outcome draw` after the record's outcome"));
        assert_eq!(refused("decision log 1\nengine x\nseed 41\nlife twenty\n"), LogError::at(4, "life `twenty` is not a number"));
    }
}
