//! The grammar (`setup-architecture.md` §5.1), both ways: text to a
//! [`Scenario`], which checks the grammar only (names, references and the
//! rules are the loader's), and a `Scenario` back to its text.

use super::board::{Arrival, Attacked, CardLine, CardWord, LineKind, LineNumbered, NamedCard, PlayerWord, Scenario};
use super::error::{ScenarioError, ScenarioErrorKind};
use crate::state::game_state::{initial_step, next_step, Phase, PhaseType, StepType, TurnPlan};
use crate::types::card_types::CardType;
use crate::types::effects::CounterType;
use crate::types::history::{HistorySpan, TurnFact};
use crate::ui::display::{phase_name, step_name};

impl Scenario {
    /// Read a scenario file, refusing the first line the grammar does not.
    pub fn parse(text: &str) -> Result<Scenario, ScenarioError> {
        let mut scenario = Scenario::default();
        let mut stated: Vec<&'static str> = Vec::new();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let content = raw.split('#').next().unwrap_or_default().trim();
            if content.is_empty() {
                continue;
            }
            match content.split_once(':') {
                None => parse_header(&mut scenario, &mut stated, content, line)?,
                Some((head, rest)) => parse_headed_line(&mut scenario, head.trim(), rest.trim(), line)?,
            }
        }
        Ok(scenario)
    }
}

fn syntax_error(line: usize, message: impl Into<String>) -> ScenarioError {
    ScenarioError::at(ScenarioErrorKind::Syntax, line, message)
}

/// `text` as a number, or a refusal naming `what` it was for.
fn parse_number<T: std::str::FromStr>(text: &str, what: &str, line: usize) -> Result<T, ScenarioError> {
    text.trim().parse().map_err(|_| syntax_error(line, format!("`{what}` takes a number, not `{}`", text.trim())))
}

/// A line with no colon: one of the game's own facts, each stated once.
fn parse_header(scenario: &mut Scenario, stated: &mut Vec<&'static str>, content: &str, line: usize) -> Result<(), ScenarioError> {
    let keys: [&'static str; 6] = ["players", "starting life", "seed", "turn", "active", "step"];
    let Some(key) = keys.into_iter().find(|key| content.strip_prefix(key).is_some_and(|rest| rest.starts_with(' '))) else {
        return Err(syntax_error(line, format!("`{content}` is not a line a scenario has: see the template")));
    };
    if stated.contains(&key) {
        return Err(syntax_error(line, format!("`{key}` is stated twice")));
    }
    stated.push(key);
    let value = content[key.len()..].trim();
    match key {
        "players" => scenario.players = parse_number(value, key, line)?,
        "starting life" => scenario.starting_life = parse_number(value, key, line)?,
        "seed" => scenario.seed = parse_number(value, key, line)?,
        "turn" => scenario.turn = parse_number(value, key, line)?,
        "active" => scenario.active = parse_number(value, key, line)?,
        _ => {
            scenario.step = turn_positions().find(|&p| position_word(p) == value.to_lowercase()).ok_or_else(|| {
                let words: Vec<String> = turn_positions().map(position_word).collect();
                syntax_error(line, format!("`{value}` is not a step; the steps are {}", words.join(", ")))
            })?;
        }
    }
    Ok(())
}

/// Every position a turn passes through, in CR 500.1's order: a main phase,
/// or a phase's step.
pub(super) fn turn_positions() -> impl Iterator<Item = Phase> {
    TurnPlan::natural().phases.into_iter().flat_map(|planned| {
        let phase_type = planned.phase_type;
        let mut steps = Vec::new();
        let mut step = initial_step(phase_type);
        if step.is_none() {
            steps.push(Phase { phase_type, step: None });
        }
        while let Some(current) = step {
            steps.push(Phase { phase_type, step: Some(current) });
            step = next_step(phase_type, current);
        }
        steps
    })
}

/// Is `position` this turn's combat, at `from` or later?
pub(super) fn in_combat_from(position: Phase, from: StepType) -> bool {
    let index = |p: Phase| turn_positions().position(|q| q == p);
    position.phase_type == PhaseType::Combat && index(position) >= index(Phase { phase_type: PhaseType::Combat, step: Some(from) })
}

/// A position as `step` spells it: `format_phase`'s name, lower case.
pub(super) fn position_word(position: Phase) -> String {
    match position.step {
        Some(step) => step_name(step),
        None => phase_name(position.phase_type),
    }
    .to_lowercase()
}

/// A line with a head before its colon.
fn parse_headed_line(scenario: &mut Scenario, head: &str, rest: &str, line: usize) -> Result<(), ScenarioError> {
    let tokens: Vec<&str> = head.split_whitespace().collect();
    let seat = || -> Result<usize, ScenarioError> { parse_number(tokens.get(1).copied().unwrap_or_default(), tokens[0], line) };
    let head = match tokens.as_slice() {
        ["player", _] => return parse_player_line(scenario, seat()?, rest, line),
        ["player", _, head_words @ ..] => {
            let span = match head_words {
                ["this", "turn"] => HistorySpan::ThisTurn,
                ["last", "turn"] => HistorySpan::LastTurn,
                ["this", "game"] => HistorySpan::ThisGame,
                ["since", "your", "last", "turn"] => HistorySpan::SinceYourLastTurn,
                _ => return Err(syntax_error(line, format!("`{head}:` is not a history row: this turn, last turn or this game"))),
            };
            return parse_history_line(scenario, seat()?, span, rest, line);
        }
        ["hand", _] => LineKind::Hand(seat()?),
        ["library", _] => LineKind::Library { player: seat()?, shuffled: false },
        ["library", _, "shuffled"] => LineKind::Library { player: seat()?, shuffled: true },
        ["graveyard", _] => LineKind::Graveyard(seat()?),
        ["exile"] => LineKind::Exile,
        ["command"] => LineKind::Command,
        ["battlefield"] => LineKind::Battlefield,
        ["counters"] => LineKind::Counters,
        ["this", "turn"] => LineKind::ThisTurn,
        ["player" | "hand" | "library" | "graveyard"] => {
            return Err(syntax_error(line, format!("`{head}:` names no player: `{head} 0:`")));
        }
        _ => return Err(syntax_error(line, format!("`{head}:` is not a line a scenario has: see the template"))),
    };
    let card_line = parse_card_line(head, rest, line)?;
    scenario.cards.push(LineNumbered { line, value: card_line });
    Ok(())
}

/// `<name> [tag]`, the tag optional.
fn parse_named_card(text: &str, line: usize) -> Result<NamedCard, ScenarioError> {
    let text = text.trim();
    let (name, tag) = match text.split_once('[') {
        None => (text, None),
        Some((name, tagged)) => {
            let Some(tag) = tagged.trim_end().strip_suffix(']') else {
                return Err(syntax_error(line, format!("`{text}`: a tag ends with `]`, and nothing follows it")));
            };
            (name.trim_end(), Some(tag.trim().to_string()))
        }
    };
    if name.is_empty() {
        return Err(syntax_error(line, "a card's name is missing"));
    }
    Ok(NamedCard { name: name.to_string(), tag })
}

/// A card line, its words checked against what its kind of line can say.
fn parse_card_line(kind: LineKind, rest: &str, line: usize) -> Result<CardLine, ScenarioError> {
    let (card, words_text) = rest.split_once('|').unwrap_or((rest, ""));
    let card = parse_named_card(card, line)?;
    let (words, copies) = parse_card_words(words_text, line)?;
    if copies == 0 || (copies > 1 && (card.tag.is_some() || matches!(kind, LineKind::Counters | LineKind::ThisTurn))) {
        return Err(syntax_error(line, format!("`x{copies}` here would make the line, or its tag, mean {copies} cards")));
    }
    for word in &words {
        let this_turn = matches!(word, CardWord::Triggered { .. } | CardWord::Resolved { .. } | CardWord::TookOnceEachTurnAction { .. });
        let fits = match kind {
            LineKind::Battlefield => !this_turn,
            LineKind::Counters => matches!(word, CardWord::Counter(..)),
            LineKind::ThisTurn => this_turn,
            LineKind::Exile | LineKind::Command => matches!(word, CardWord::Owner(_) | CardWord::Commander),
            LineKind::Hand(_) | LineKind::Library { .. } | LineKind::Graveyard(_) => matches!(word, CardWord::Commander),
        };
        if !fits {
            return Err(syntax_error(line, format!("{word:?} is not a word this line has; see the template")));
        }
    }
    Ok(CardLine { kind, card, copies, words })
}

/// The words after a bar, split at commas, except that a word naming a card
/// takes the rest of the line; and the line's count, `xN`.
fn parse_card_words(text: &str, line: usize) -> Result<(Vec<CardWord>, u32), ScenarioError> {
    let mut words = Vec::new();
    let mut copies = None;
    let mut rest = text.trim();
    while !rest.is_empty() {
        let (word, after) = rest.split_once(',').unwrap_or((rest, ""));
        let reference = ["attached to ", "blocking ", "attacking "]
            .into_iter()
            .find_map(|prefix| rest.strip_prefix(prefix).map(|card| (prefix, card)));
        match reference {
            Some(("attacking ", _)) if attacked_player_number(word).is_some() => {
                let player = attacked_player_number(word).unwrap_or_default();
                words.push(CardWord::Attacking(Attacked::Player(parse_number(player, "attacking player", line)?)));
            }
            Some((prefix, card)) => {
                let card = parse_named_card(card, line)?;
                words.push(match prefix {
                    "attached to " => CardWord::AttachedTo(card),
                    "blocking " => CardWord::Blocking(card),
                    _ => CardWord::Attacking(Attacked::Permanent(card)),
                });
                break;
            }
            None => match word.trim().strip_prefix('x').filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())) {
                Some(n) if copies.is_none() => copies = Some(parse_number(n, "x", line)?),
                Some(_) => return Err(syntax_error(line, "the count `xN` is stated twice")),
                None => words.push(parse_card_word(word.trim(), line)?),
            },
        }
        rest = after.trim();
    }
    Ok((words, copies.unwrap_or(1)))
}

/// `attacking player N`'s number: an attack on a player rather than on a card.
fn attacked_player_number(word: &str) -> Option<&str> {
    word.trim().strip_prefix("attacking player ").filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// `<words> N`: the words, and the number that ends them, which may be
/// negative: a life total below zero is at rest under Platinum Angel.
fn split_trailing_number(word: &str) -> Option<(&str, &str)> {
    let (words, n) = word.rsplit_once(' ')?;
    let digits = n.strip_prefix('-').unwrap_or(n);
    (!digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())).then_some((words.trim(), n))
}

fn parse_card_word(word: &str, line: usize) -> Result<CardWord, ScenarioError> {
    let unknown = || syntax_error(line, format!("`{word}` is not a word a card line has: see the template"));
    Ok(match word {
        "commander" => CardWord::Commander,
        "tapped" => CardWord::Tapped,
        "blocked" => CardWord::Blocked,
        "dealt first-strike damage" => CardWord::DealtFirstStrikeDamage,
        "arrived this turn" => CardWord::Arrived(Arrival::ThisTurn),
        "triggered" => CardWord::Triggered { ability: None },
        "took its once-each-turn action" => CardWord::TookOnceEachTurnAction { ability: None },
        _ => {
            if let Some(rest) = word.strip_prefix("ability ") {
                let (n, fact) = rest.split_once(' ').ok_or_else(unknown)?;
                let ability = Some(parse_number(n, "ability", line)?);
                return match parse_card_word(fact, line)? {
                    CardWord::Triggered { .. } => Ok(CardWord::Triggered { ability }),
                    CardWord::Resolved { times, .. } => Ok(CardWord::Resolved { ability, times }),
                    CardWord::TookOnceEachTurnAction { .. } => Ok(CardWord::TookOnceEachTurnAction { ability }),
                    _ => Err(unknown()),
                };
            }
            let (name, n) = split_trailing_number(word).ok_or_else(unknown)?;
            match name {
                "owner" => CardWord::Owner(parse_number(n, name, line)?),
                "controller" => CardWord::Controller(parse_number(n, name, line)?),
                "arrived turn" => CardWord::Arrived(Arrival::Turn(parse_number(n, name, line)?)),
                "damage" => CardWord::Damage(parse_number(n, name, line)?),
                "resolved" => CardWord::Resolved { ability: None, times: parse_number(n, name, line)? },
                kind => CardWord::Counter(CounterType::named(kind).ok_or_else(unknown)?, parse_number(n, kind, line)?),
            }
        }
    })
}

/// `player p:`'s facts. `commander damage N from <card>` takes the rest of
/// the line, since a commander's name may hold a comma.
fn parse_player_line(scenario: &mut Scenario, player: usize, rest: &str, line: usize) -> Result<(), ScenarioError> {
    let mut rest = rest.trim();
    while !rest.is_empty() {
        let fact = if let Some(damage) = rest.strip_prefix("commander damage ") {
            let (n, card) = damage.split_once(" from ").ok_or_else(|| syntax_error(line, "`commander damage N from <card>`"))?;
            rest = "";
            PlayerWord::CommanderDamage { player, damage: parse_number(n, "commander damage", line)?, from: parse_named_card(card, line)? }
        } else {
            let (word, after) = rest.split_once(',').unwrap_or((rest, ""));
            rest = after.trim();
            let word = word.trim();
            let unknown = || syntax_error(line, format!("`{word}` is not a word a player line has: see the template"));
            if word == "left the game" {
                PlayerWord::LeftTheGame { player }
            } else {
                let (name, n) = split_trailing_number(word).ok_or_else(unknown)?;
                match name {
                    "life" => PlayerWord::Life { player, life: parse_number(n, name, line)? },
                    "lands played" => PlayerWord::LandsPlayed { player, count: parse_number(n, name, line)? },
                    kind => PlayerWord::Counter {
                        player,
                        kind: CounterType::named(kind).ok_or_else(unknown)?,
                        count: parse_number(n, kind, line)?,
                    },
                }
            }
        };
        scenario.player_words.push(LineNumbered { line, value: fact });
    }
    Ok(())
}

/// `player p this turn:` and its kin: `<fact> N`, comma-separated.
fn parse_history_line(scenario: &mut Scenario, player: usize, span: HistorySpan, rest: &str, line: usize) -> Result<(), ScenarioError> {
    for word in rest.split(',').map(str::trim).filter(|w| !w.is_empty()) {
        let unknown = || {
            let facts: Vec<String> = every_turn_fact().map(turn_fact_word).collect();
            syntax_error(line, format!("`{word}` is not a count a history row has; they are {}", facts.join(", ")))
        };
        let (name, n) = split_trailing_number(word).ok_or_else(unknown)?;
        let fact = every_turn_fact().find(|&fact| turn_fact_word(fact) == name).ok_or_else(unknown)?;
        let count = parse_number(n, name, line)?;
        scenario.player_words.push(LineNumbered { line, value: PlayerWord::History { player, span, fact, count } });
    }
    Ok(())
}

/// Every `TurnFact`, a slot each.
pub(super) fn every_turn_fact() -> impl Iterator<Item = TurnFact> {
    [
        TurnFact::SpellsCast,
        TurnFact::CardsDrawn,
        TurnFact::LifeGained,
        TurnFact::LifeGainEvents,
        TurnFact::LifeLost,
        TurnFact::LifeLossEvents,
        TurnFact::DamageTaken,
        TurnFact::ControlledCreaturesDied,
        TurnFact::AttackersDeclared,
    ]
    .into_iter()
    .chain(CardType::ALL.map(TurnFact::SpellsCastOfType))
}

/// A `TurnFact` as a history row spells it. Exhaustive, so a new fact fails
/// to compile until it has a word.
pub(super) fn turn_fact_word(fact: TurnFact) -> String {
    match fact {
        TurnFact::SpellsCast => "spells cast".to_string(),
        TurnFact::SpellsCastOfType(card_type) => format!("{} spells cast", format!("{card_type:?}").to_lowercase()),
        TurnFact::CardsDrawn => "cards drawn".to_string(),
        TurnFact::LifeGained => "life gained".to_string(),
        TurnFact::LifeGainEvents => "life gain events".to_string(),
        TurnFact::LifeLost => "life lost".to_string(),
        TurnFact::LifeLossEvents => "life loss events".to_string(),
        TurnFact::DamageTaken => "damage taken".to_string(),
        TurnFact::ControlledCreaturesDied => "creatures died".to_string(),
        TurnFact::AttackersDeclared => "attackers declared".to_string(),
    }
}

// The grammar, written: the parser's inverse, so a value written and read
// back is the value. A default is left out.

impl std::fmt::Display for Scenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let defaults = Scenario::default();
        let headers = [
            ("players", self.players != defaults.players, self.players.to_string()),
            ("starting life", self.starting_life != defaults.starting_life, self.starting_life.to_string()),
            ("seed", self.seed != defaults.seed, self.seed.to_string()),
            ("turn", self.turn != defaults.turn, self.turn.to_string()),
            ("active", self.active != defaults.active, self.active.to_string()),
            ("step", self.step != defaults.step, position_word(self.step)),
        ];
        for (key, _, value) in headers.iter().filter(|(_, stated, _)| *stated) {
            writeln!(f, "{key} {value}")?;
        }
        // Consecutive facts that share a head share a line, but for the
        // commander damage, which ends its line.
        let mut open: Option<String> = None;
        for fact in &self.player_words {
            let (head, word, ends) = player_word_text(&fact.value);
            match &open {
                Some(current) if *current == head => write!(f, ", {word}")?,
                _ => {
                    if open.is_some() {
                        writeln!(f)?;
                    }
                    write!(f, "{head}: {word}")?;
                }
            }
            open = if ends { writeln!(f)?; None } else { Some(head) };
        }
        if open.is_some() {
            writeln!(f)?;
        }
        for card in &self.cards {
            writeln!(f, "{}", card.value)?;
        }
        Ok(())
    }
}

/// A fact's head, its word, and whether the word ends its line.
fn player_word_text(fact: &PlayerWord) -> (String, String, bool) {
    match fact {
        PlayerWord::Life { player, life } => (format!("player {player}"), format!("life {life}"), false),
        PlayerWord::Counter { player, kind, count } => (format!("player {player}"), format!("{} {count}", kind.name()), false),
        PlayerWord::LandsPlayed { player, count } => (format!("player {player}"), format!("lands played {count}"), false),
        PlayerWord::LeftTheGame { player } => (format!("player {player}"), "left the game".to_string(), false),
        PlayerWord::CommanderDamage { player, damage, from } => {
            (format!("player {player}"), format!("commander damage {damage} from {from}"), true)
        }
        PlayerWord::History { player, span, fact, count } => {
            let span = match span {
                HistorySpan::ThisTurn => "this turn",
                HistorySpan::LastTurn => "last turn",
                HistorySpan::ThisGame => "this game",
                HistorySpan::SinceYourLastTurn => "since your last turn",
            };
            (format!("player {player} {span}"), format!("{} {count}", turn_fact_word(*fact)), false)
        }
    }
}

impl std::fmt::Display for NamedCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.tag {
            Some(tag) => write!(f, "{} [{tag}]", self.name),
            None => f.write_str(&self.name),
        }
    }
}

impl std::fmt::Display for CardLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            LineKind::Hand(p) => write!(f, "hand {p}")?,
            LineKind::Library { player, shuffled: false } => write!(f, "library {player}")?,
            LineKind::Library { player, shuffled: true } => write!(f, "library {player} shuffled")?,
            LineKind::Graveyard(p) => write!(f, "graveyard {p}")?,
            LineKind::Exile => f.write_str("exile")?,
            LineKind::Command => f.write_str("command")?,
            LineKind::Battlefield => f.write_str("battlefield")?,
            LineKind::Counters => f.write_str("counters")?,
            LineKind::ThisTurn => f.write_str("this turn")?,
        }
        write!(f, ": {}", self.card)?;
        let mut words: Vec<String> = Vec::new();
        if self.copies > 1 {
            words.push(format!("x{}", self.copies));
        }
        let (references, plain): (Vec<&CardWord>, Vec<&CardWord>) = self.words.iter().partition(|word| word.names_a_card());
        words.extend(plain.into_iter().chain(references).map(card_word_text));
        if !words.is_empty() {
            write!(f, " | {}", words.join(", "))?;
        }
        Ok(())
    }
}

fn card_word_text(word: &CardWord) -> String {
    let ability = |n: &Option<usize>| n.map(|n| format!("ability {n} ")).unwrap_or_default();
    match word {
        CardWord::Owner(p) => format!("owner {p}"),
        CardWord::Controller(p) => format!("controller {p}"),
        CardWord::Commander => "commander".to_string(),
        CardWord::Tapped => "tapped".to_string(),
        CardWord::Arrived(Arrival::ThisTurn) => "arrived this turn".to_string(),
        CardWord::Arrived(Arrival::Turn(n)) => format!("arrived turn {n}"),
        CardWord::Counter(kind, n) => format!("{} {n}", kind.name()),
        CardWord::Damage(n) => format!("damage {n}"),
        CardWord::DealtFirstStrikeDamage => "dealt first-strike damage".to_string(),
        CardWord::Blocked => "blocked".to_string(),
        CardWord::AttachedTo(card) => format!("attached to {card}"),
        CardWord::Attacking(Attacked::Player(p)) => format!("attacking player {p}"),
        CardWord::Attacking(Attacked::Permanent(card)) => format!("attacking {card}"),
        CardWord::Blocking(card) => format!("blocking {card}"),
        CardWord::Triggered { ability: n } => format!("{}triggered", ability(n)),
        CardWord::Resolved { ability: n, times } => format!("{}resolved {times}", ability(n)),
        CardWord::TookOnceEachTurnAction { ability: n } => format!("{}took its once-each-turn action", ability(n)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::game_state::{PhaseType, StepType};

    fn refused(text: &str) -> ScenarioError {
        Scenario::parse(text).expect_err(text)
    }

    #[test]
    fn the_design_example_parses_line_by_line() {
        let scenario = Scenario::parse(include_str!("../../scenarios/holy-strength.scenario")).unwrap();
        assert_eq!((scenario.players, scenario.seed, scenario.turn, scenario.active), (2, 7, 3, 0));
        assert_eq!(scenario.step, Phase { phase_type: PhaseType::Combat, step: Some(StepType::DeclareBlockers) });
        let line = |name: &str, tag: Option<&str>| {
            let wanted = NamedCard { name: name.to_string(), tag: tag.map(str::to_string) };
            scenario.cards.iter().find(|c| c.value.card == wanted).map(|c| &c.value).unwrap()
        };
        let bears = NamedCard { name: "Grizzly Bears".to_string(), tag: Some("a".to_string()) };
        assert_eq!(line("Grizzly Bears", Some("a")).words, [CardWord::Controller(0), CardWord::Tapped, CardWord::Attacking(Attacked::Player(1))]);
        assert_eq!(line("Holy Strength", None).words, [CardWord::Controller(0), CardWord::AttachedTo(bears.clone())]);
        assert_eq!(line("Wall of Stone", None).words, [CardWord::Controller(1), CardWord::Blocking(bears)]);
        assert_eq!(line("Loyalty Probe", None).words[1], CardWord::Counter(CounterType::Loyalty, 1));
        assert_eq!(line("Humility", None).words[1], CardWord::Arrived(Arrival::ThisTurn));
        assert_eq!(line("Isamaru, Hound of Konda", None).words, [CardWord::Owner(0), CardWord::Commander]);
        let forests: Vec<(LineKind, u32)> =
            scenario.cards.iter().filter(|c| c.value.card.name == "Forest").map(|c| (c.value.kind, c.value.copies)).collect();
        assert_eq!(forests, [(LineKind::Library { player: 0, shuffled: false }, 10), (LineKind::Library { player: 1, shuffled: true }, 20)]);
        assert!(scenario.player_words.iter().any(|f| f.value
            == PlayerWord::History { player: 0, span: HistorySpan::ThisTurn, fact: TurnFact::AttackersDeclared, count: 1 }));
        assert!(scenario.player_words.iter().any(|f| f.line == 10 && f.value == PlayerWord::Counter { player: 1, kind: CounterType::Poison, count: 2 }));
    }

    /// A reference takes the rest of its line, so a name with a comma or a
    /// colon in it is one card.
    #[test]
    fn a_reference_runs_to_the_end_of_its_line() {
        let scenario = Scenario::parse(
            "battlefield: Circle of Protection: Red | controller 0, attached to Isamaru, Hound of Konda [b]\n\
             player 1: life 3, commander damage 18 from Isamaru, Hound of Konda",
        )
        .unwrap();
        let isamaru = NamedCard { name: "Isamaru, Hound of Konda".to_string(), tag: None };
        assert_eq!(scenario.cards[0].value.card.name, "Circle of Protection: Red");
        assert_eq!(scenario.cards[0].value.words[1], CardWord::AttachedTo(NamedCard { tag: Some("b".to_string()), ..isamaru.clone() }));
        assert_eq!(scenario.player_words[1].value, PlayerWord::CommanderDamage { player: 1, damage: 18, from: isamaru });
    }

    #[test]
    fn the_grammar_refuses_what_it_does_not_read_naming_the_line() {
        for (text, says) in [
            ("players two", "takes a number"),
            ("turn 2\nturn 3", "stated twice"),
            ("step combat", "is not a step"),
            ("step untap\nwhatever", "is not a line"),
            ("graveyard: Forest", "names no player"),
            ("battlefield: Forest | sideways", "is not a word"),
            ("hand 0: Forest | tapped", "not a word this line has"),
            ("battlefield: Grizzly Bears [a | controller 0", "a tag ends with"),
            ("battlefield: Grizzly Bears [a] | controller 0, x2", "mean 2 cards"),
            ("player 0 this turn: spells resolved 1", "not a count a history row has"),
            ("player 0 next turn: spells cast 1", "not a history row"),
        ] {
            let refusal = refused(text);
            assert_eq!(refusal.kind, ScenarioErrorKind::Syntax, "{text}");
            assert!(refusal.message.contains(says), "{text}: {refusal}");
            assert_eq!(refusal.line, Some(text.lines().count()), "{text}");
        }
    }

    /// Every word, every kind of line and every span, written and read
    /// back. The matches are exhaustive, so a new variant fails to compile
    /// here until the parser reads what `Display` writes.
    #[test]
    fn every_word_reads_back_as_written() {
        let card = NamedCard { name: "Isamaru, Hound of Konda".to_string(), tag: Some("a".to_string()) };
        let card_words = [
            CardWord::Owner(1),
            CardWord::Controller(0),
            CardWord::Commander,
            CardWord::Tapped,
            CardWord::Arrived(Arrival::ThisTurn),
            CardWord::Arrived(Arrival::Turn(2)),
            CardWord::Counter(CounterType::FirstStrike, 1),
            CardWord::Damage(2),
            CardWord::DealtFirstStrikeDamage,
            CardWord::Blocked,
            CardWord::AttachedTo(card.clone()),
            CardWord::Attacking(Attacked::Player(1)),
            CardWord::Attacking(Attacked::Permanent(card.clone())),
            CardWord::Blocking(card.clone()),
            CardWord::Triggered { ability: Some(2) },
            CardWord::Resolved { ability: None, times: 3 },
            CardWord::TookOnceEachTurnAction { ability: Some(1) },
        ];
        let mut scenario = Scenario { players: 3, starting_life: -4, seed: 9, turn: 4, active: 2, ..Scenario::default() };
        for word in card_words {
            let kind = match word {
                CardWord::Triggered { .. } | CardWord::Resolved { .. } | CardWord::TookOnceEachTurnAction { .. } => LineKind::ThisTurn,
                CardWord::Owner(_) | CardWord::Controller(_) | CardWord::Commander | CardWord::Tapped | CardWord::Arrived(_)
                | CardWord::Counter(..) | CardWord::Damage(_) | CardWord::DealtFirstStrikeDamage | CardWord::Blocked
                | CardWord::AttachedTo(_) | CardWord::Attacking(_) | CardWord::Blocking(_) => LineKind::Battlefield,
            };
            scenario.cards.push(LineNumbered { line: 0, value: CardLine { kind, card: card.clone(), copies: 1, words: vec![word] } });
        }
        for kind in [
            LineKind::Hand(0),
            LineKind::Library { player: 1, shuffled: true },
            LineKind::Graveyard(2),
            LineKind::Exile,
            LineKind::Command,
            LineKind::Counters,
        ] {
            let words = match kind {
                LineKind::Exile | LineKind::Command => vec![CardWord::Owner(0)],
                LineKind::Counters => vec![CardWord::Counter(CounterType::PlusOnePlusOne, 2)],
                LineKind::Hand(_) | LineKind::Library { .. } | LineKind::Graveyard(_) | LineKind::Battlefield | LineKind::ThisTurn => vec![],
            };
            let copies = if words.is_empty() { 3 } else { 1 };
            scenario.cards.push(LineNumbered { line: 0, value: CardLine { kind, card: NamedCard { tag: None, ..card.clone() }, copies, words } });
        }
        for value in [
            PlayerWord::Life { player: 0, life: -3 },
            PlayerWord::Counter { player: 0, kind: CounterType::Poison, count: 2 },
            PlayerWord::LandsPlayed { player: 0, count: 1 },
            PlayerWord::LeftTheGame { player: 1 },
            PlayerWord::CommanderDamage { player: 0, damage: 7, from: card.clone() },
            PlayerWord::History { player: 2, span: HistorySpan::ThisTurn, fact: TurnFact::SpellsCastOfType(CardType::Instant), count: 1 },
            PlayerWord::History { player: 2, span: HistorySpan::LastTurn, fact: TurnFact::CardsDrawn, count: 2 },
            PlayerWord::History { player: 2, span: HistorySpan::ThisGame, fact: TurnFact::LifeLost, count: 5 },
            PlayerWord::History { player: 2, span: HistorySpan::SinceYourLastTurn, fact: TurnFact::LifeGained, count: 1 },
        ] {
            scenario.player_words.push(LineNumbered { line: 0, value });
        }
        let text = scenario.to_string();
        let read = Scenario::parse(&text).unwrap_or_else(|e| panic!("{e}
{text}"));
        let values = |s: &Scenario| (s.cards.iter().map(|c| c.value.clone()).collect::<Vec<_>>(), s.player_words.iter().map(|p| p.value.clone()).collect::<Vec<_>>());
        assert_eq!(values(&read), values(&scenario), "{text}");
        assert_eq!((read.players, read.starting_life, read.seed, read.turn, read.active), (3, -4, 9, 4, 2));
    }

    /// A word naming a card is written last, wherever a value holds it, so
    /// the line reads back.
    #[test]
    fn a_word_naming_a_card_is_written_last() {
        let host = NamedCard { name: "Grizzly Bears".to_string(), tag: None };
        let line = CardLine {
            kind: LineKind::Battlefield,
            card: NamedCard { name: "Holy Strength".to_string(), tag: None },
            copies: 1,
            words: vec![CardWord::AttachedTo(host), CardWord::Controller(0)],
        };
        assert_eq!(line.to_string(), "battlefield: Holy Strength | controller 0, attached to Grizzly Bears");
    }

    #[test]
    fn every_turn_fact_has_a_word_and_a_slot() {
        let facts: Vec<TurnFact> = every_turn_fact().collect();
        let mut slots: Vec<usize> = facts.iter().map(|f| f.slot()).collect();
        slots.sort();
        slots.dedup();
        assert_eq!(slots.len(), TurnFact::COUNT);
        let mut words: Vec<String> = facts.into_iter().map(turn_fact_word).collect();
        words.sort();
        words.dedup();
        assert_eq!(words.len(), TurnFact::COUNT);
    }
}
