//! The module doc's grammar, read: text to a [`Scenario`]. Checks the
//! grammar only; names, references and the rules are the loader's.

use super::board::{Arrival, CardLine, CardRef, Head, HistoryRow, Located, PlayerFact, Scenario, Target, Word};
use super::refusal::{Refusal, RefusalKind};
use crate::state::game_state::{initial_step, next_step, Phase, TurnPlan};
use crate::types::card_types::CardType;
use crate::types::effects::CounterType;
use crate::types::history::TurnFact;
use crate::ui::display::{phase_name, step_name};

impl Scenario {
    /// Read a scenario file, refusing the first line the grammar does not.
    pub fn parse(text: &str) -> Result<Scenario, Refusal> {
        let mut scenario = Scenario::default();
        let mut stated: Vec<&'static str> = Vec::new();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let content = raw.split('#').next().unwrap_or_default().trim();
            if content.is_empty() {
                continue;
            }
            match content.split_once(':') {
                None => header(&mut scenario, &mut stated, content, line)?,
                Some((head, rest)) => headed_line(&mut scenario, head.trim(), rest.trim(), line)?,
            }
        }
        Ok(scenario)
    }
}

fn syntax(line: usize, message: impl Into<String>) -> Refusal {
    Refusal::at(RefusalKind::Syntax, line, message)
}

/// `text` as a number, or a refusal naming `what` it was for.
fn number<T: std::str::FromStr>(text: &str, what: &str, line: usize) -> Result<T, Refusal> {
    text.trim().parse().map_err(|_| syntax(line, format!("`{what}` takes a number, not `{}`", text.trim())))
}

/// A line with no colon: one of the game's own facts, each stated once.
fn header(scenario: &mut Scenario, stated: &mut Vec<&'static str>, content: &str, line: usize) -> Result<(), Refusal> {
    let keys: [&'static str; 6] = ["players", "starting life", "seed", "turn", "active", "step"];
    let Some(key) = keys.into_iter().find(|key| content.strip_prefix(key).is_some_and(|rest| rest.starts_with(' '))) else {
        return Err(syntax(line, format!("`{content}` is not a line a scenario has: see the template")));
    };
    if stated.contains(&key) {
        return Err(syntax(line, format!("`{key}` is stated twice")));
    }
    stated.push(key);
    let value = content[key.len()..].trim();
    match key {
        "players" => scenario.players = number(value, key, line)?,
        "starting life" => scenario.starting_life = number(value, key, line)?,
        "seed" => scenario.seed = number(value, key, line)?,
        "turn" => scenario.turn = number(value, key, line)?,
        "active" => scenario.active = number(value, key, line)?,
        _ => {
            scenario.step = positions().find(|&p| position_word(p) == value.to_lowercase()).ok_or_else(|| {
                let words: Vec<String> = positions().map(position_word).collect();
                syntax(line, format!("`{value}` is not a step; the steps are {}", words.join(", ")))
            })?;
        }
    }
    Ok(())
}

/// Every position a turn passes through, in CR 500.1's order: a main phase,
/// or a phase's step.
pub(super) fn positions() -> impl Iterator<Item = Phase> {
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

/// A position as `step` spells it: `format_phase`'s name, lower case.
pub(super) fn position_word(position: Phase) -> String {
    match position.step {
        Some(step) => step_name(step),
        None => phase_name(position.phase_type),
    }
    .to_lowercase()
}

/// A line with a head before its colon.
fn headed_line(scenario: &mut Scenario, head: &str, rest: &str, line: usize) -> Result<(), Refusal> {
    let tokens: Vec<&str> = head.split_whitespace().collect();
    let seat = || -> Result<usize, Refusal> { number(tokens.get(1).copied().unwrap_or_default(), tokens[0], line) };
    let head = match tokens.as_slice() {
        ["player", _] => return player_line(scenario, seat()?, rest, line),
        ["player", _, span @ ..] => {
            let row = match span {
                ["this", "turn"] => HistoryRow::ThisTurn,
                ["last", "turn"] => HistoryRow::LastTurn,
                ["this", "game"] => HistoryRow::ThisGame,
                _ => return Err(syntax(line, format!("`{head}:` is not a history row: this turn, last turn or this game"))),
            };
            return history_line(scenario, seat()?, row, rest, line);
        }
        ["hand", _] => Head::Hand(seat()?),
        ["library", _] => Head::Library { player: seat()?, shuffled: false },
        ["library", _, "shuffled"] => Head::Library { player: seat()?, shuffled: true },
        ["graveyard", _] => Head::Graveyard(seat()?),
        ["exile"] => Head::Exile,
        ["command"] => Head::Command,
        ["battlefield"] => Head::Battlefield,
        ["counters"] => Head::Counters,
        ["this", "turn"] => Head::ThisTurn,
        ["player" | "hand" | "library" | "graveyard"] => {
            return Err(syntax(line, format!("`{head}:` names no player: `{head} 0:`")));
        }
        _ => return Err(syntax(line, format!("`{head}:` is not a line a scenario has: see the template"))),
    };
    let card_line = card_line(head, rest, line)?;
    scenario.cards.push(Located { line, item: card_line });
    Ok(())
}

/// `<name> [tag]`, the tag optional.
fn card_ref(text: &str, line: usize) -> Result<CardRef, Refusal> {
    let text = text.trim();
    let (name, tag) = match text.split_once('[') {
        None => (text, None),
        Some((name, tagged)) => {
            let Some(tag) = tagged.trim_end().strip_suffix(']') else {
                return Err(syntax(line, format!("`{text}`: a tag ends with `]`, and nothing follows it")));
            };
            (name.trim_end(), Some(tag.trim().to_string()))
        }
    };
    if name.is_empty() {
        return Err(syntax(line, "a card's name is missing"));
    }
    Ok(CardRef { name: name.to_string(), tag })
}

/// A card line, its words checked against what its head can say.
fn card_line(head: Head, rest: &str, line: usize) -> Result<CardLine, Refusal> {
    let (card, words_text) = rest.split_once('|').unwrap_or((rest, ""));
    let card = card_ref(card, line)?;
    let (words, copies) = words(words_text, line)?;
    if copies == 0 || (copies > 1 && (card.tag.is_some() || matches!(head, Head::Counters | Head::ThisTurn))) {
        return Err(syntax(line, format!("`x{copies}` here would make the line, or its tag, mean {copies} cards")));
    }
    for word in &words {
        let this_turn = matches!(word, Word::Triggered { .. } | Word::Resolved { .. } | Word::TookOnceEachTurnAction { .. });
        let fits = match head {
            Head::Battlefield => !this_turn,
            Head::Counters => matches!(word, Word::Counter(..)),
            Head::ThisTurn => this_turn,
            Head::Exile | Head::Command => matches!(word, Word::Owner(_) | Word::Commander),
            Head::Hand(_) | Head::Library { .. } | Head::Graveyard(_) => matches!(word, Word::Commander),
        };
        if !fits {
            return Err(syntax(line, format!("{word:?} is not a word this line has; see the template")));
        }
    }
    Ok(CardLine { head, card, copies, words })
}

/// The words after a bar, split at commas, except that a word naming a card
/// takes the rest of the line; and the line's count, `xN`.
fn words(text: &str, line: usize) -> Result<(Vec<Word>, u32), Refusal> {
    let mut words = Vec::new();
    let mut copies = None;
    let mut rest = text.trim();
    while !rest.is_empty() {
        let (word, after) = rest.split_once(',').unwrap_or((rest, ""));
        let reference = ["attached to ", "blocking ", "attacking "]
            .into_iter()
            .find_map(|prefix| rest.strip_prefix(prefix).map(|card| (prefix, card)));
        match reference {
            Some(("attacking ", _)) if attacked_player(word).is_some() => {
                let player = attacked_player(word).unwrap_or_default();
                words.push(Word::Attacking(Target::Player(number(player, "attacking player", line)?)));
            }
            Some((prefix, card)) => {
                let card = card_ref(card, line)?;
                words.push(match prefix {
                    "attached to " => Word::AttachedTo(card),
                    "blocking " => Word::Blocking(card),
                    _ => Word::Attacking(Target::Permanent(card)),
                });
                break;
            }
            None => match word.trim().strip_prefix('x').filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())) {
                Some(n) if copies.is_none() => copies = Some(number(n, "x", line)?),
                Some(_) => return Err(syntax(line, "the count `xN` is stated twice")),
                None => words.push(simple_word(word.trim(), line)?),
            },
        }
        rest = after.trim();
    }
    Ok((words, copies.unwrap_or(1)))
}

/// `attacking player N`'s number: an attack on a player rather than on a card.
fn attacked_player(word: &str) -> Option<&str> {
    word.trim().strip_prefix("attacking player ").filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// `<words> N`: the words, and the number that ends them.
fn counted(word: &str) -> Option<(&str, &str)> {
    let (words, n) = word.rsplit_once(' ')?;
    n.chars().all(|c| c.is_ascii_digit()).then_some((words.trim(), n))
}

fn simple_word(word: &str, line: usize) -> Result<Word, Refusal> {
    let unknown = || syntax(line, format!("`{word}` is not a word a card line has: see the template"));
    Ok(match word {
        "commander" => Word::Commander,
        "tapped" => Word::Tapped,
        "blocked" => Word::Blocked,
        "dealt first-strike damage" => Word::DealtFirstStrikeDamage,
        "arrived this turn" => Word::Arrived(Arrival::ThisTurn),
        "triggered" => Word::Triggered { ability: None },
        "took its once-each-turn action" => Word::TookOnceEachTurnAction { ability: None },
        _ => {
            if let Some(rest) = word.strip_prefix("ability ") {
                let (n, fact) = rest.split_once(' ').ok_or_else(unknown)?;
                let ability = Some(number(n, "ability", line)?);
                return match simple_word(fact, line)? {
                    Word::Triggered { .. } => Ok(Word::Triggered { ability }),
                    Word::Resolved { times, .. } => Ok(Word::Resolved { ability, times }),
                    Word::TookOnceEachTurnAction { .. } => Ok(Word::TookOnceEachTurnAction { ability }),
                    _ => Err(unknown()),
                };
            }
            let (name, n) = counted(word).ok_or_else(unknown)?;
            match name {
                "owner" => Word::Owner(number(n, name, line)?),
                "controller" => Word::Controller(number(n, name, line)?),
                "arrived turn" => Word::Arrived(Arrival::Turn(number(n, name, line)?)),
                "damage" => Word::Damage(number(n, name, line)?),
                "resolved" => Word::Resolved { ability: None, times: number(n, name, line)? },
                kind => Word::Counter(CounterType::named(kind).ok_or_else(unknown)?, number(n, kind, line)?),
            }
        }
    })
}

/// `player p:`'s facts. `commander damage N from <card>` takes the rest of
/// the line, since a commander's name may hold a comma.
fn player_line(scenario: &mut Scenario, player: usize, rest: &str, line: usize) -> Result<(), Refusal> {
    let mut rest = rest.trim();
    while !rest.is_empty() {
        let fact = if let Some(damage) = rest.strip_prefix("commander damage ") {
            let (n, card) = damage.split_once(" from ").ok_or_else(|| syntax(line, "`commander damage N from <card>`"))?;
            rest = "";
            PlayerFact::CommanderDamage { player, damage: number(n, "commander damage", line)?, from: card_ref(card, line)? }
        } else {
            let (word, after) = rest.split_once(',').unwrap_or((rest, ""));
            rest = after.trim();
            let word = word.trim();
            let unknown = || syntax(line, format!("`{word}` is not a word a player line has: see the template"));
            if word == "left the game" {
                PlayerFact::LeftTheGame { player }
            } else {
                let (name, n) = counted(word).ok_or_else(unknown)?;
                match name {
                    "life" => PlayerFact::Life { player, life: number(n, name, line)? },
                    "lands played" => PlayerFact::LandsPlayed { player, count: number(n, name, line)? },
                    kind => PlayerFact::Counter {
                        player,
                        kind: CounterType::named(kind).ok_or_else(unknown)?,
                        count: number(n, kind, line)?,
                    },
                }
            }
        };
        scenario.player_facts.push(Located { line, item: fact });
    }
    Ok(())
}

/// `player p this turn:` and its kin: `<fact> N`, comma-separated.
fn history_line(scenario: &mut Scenario, player: usize, row: HistoryRow, rest: &str, line: usize) -> Result<(), Refusal> {
    for word in rest.split(',').map(str::trim).filter(|w| !w.is_empty()) {
        let unknown = || {
            let facts: Vec<String> = every_fact().map(fact_word).collect();
            syntax(line, format!("`{word}` is not a count a history row has; they are {}", facts.join(", ")))
        };
        let (name, n) = counted(word).ok_or_else(unknown)?;
        let fact = every_fact().find(|&fact| fact_word(fact) == name).ok_or_else(unknown)?;
        let count = number(n, name, line)?;
        scenario.player_facts.push(Located { line, item: PlayerFact::History { player, row, fact, count } });
    }
    Ok(())
}

/// Every `TurnFact`, a slot each.
pub(super) fn every_fact() -> impl Iterator<Item = TurnFact> {
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
pub(super) fn fact_word(fact: TurnFact) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::game_state::{PhaseType, StepType};

    fn refused(text: &str) -> Refusal {
        Scenario::parse(text).expect_err(text)
    }

    #[test]
    fn the_design_example_parses_line_by_line() {
        let scenario = Scenario::parse(include_str!("../../scenarios/holy-strength.scenario")).unwrap();
        assert_eq!((scenario.players, scenario.seed, scenario.turn, scenario.active), (2, 7, 3, 0));
        assert_eq!(scenario.step, Phase { phase_type: PhaseType::Combat, step: Some(StepType::DeclareBlockers) });
        let line = |name: &str, tag: Option<&str>| {
            let wanted = CardRef { name: name.to_string(), tag: tag.map(str::to_string) };
            scenario.cards.iter().find(|c| c.item.card == wanted).map(|c| &c.item).unwrap()
        };
        let bears = CardRef { name: "Grizzly Bears".to_string(), tag: Some("a".to_string()) };
        assert_eq!(line("Grizzly Bears", Some("a")).words, [Word::Controller(0), Word::Tapped, Word::Attacking(Target::Player(1))]);
        assert_eq!(line("Holy Strength", None).words, [Word::Controller(0), Word::AttachedTo(bears.clone())]);
        assert_eq!(line("Wall of Stone", None).words, [Word::Controller(1), Word::Blocking(bears)]);
        assert_eq!(line("Loyalty Probe", None).words[1], Word::Counter(CounterType::Loyalty, 1));
        assert_eq!(line("Humility", None).words[1], Word::Arrived(Arrival::ThisTurn));
        assert_eq!(line("Isamaru, Hound of Konda", None).words, [Word::Owner(0), Word::Commander]);
        let forests: Vec<(Head, u32)> =
            scenario.cards.iter().filter(|c| c.item.card.name == "Forest").map(|c| (c.item.head, c.item.copies)).collect();
        assert_eq!(forests, [(Head::Library { player: 0, shuffled: false }, 10), (Head::Library { player: 1, shuffled: true }, 20)]);
        assert!(scenario.player_facts.iter().any(|f| f.item
            == PlayerFact::History { player: 0, row: HistoryRow::ThisTurn, fact: TurnFact::AttackersDeclared, count: 1 }));
        assert!(scenario.player_facts.iter().any(|f| f.line == 10 && f.item == PlayerFact::Counter { player: 1, kind: CounterType::Poison, count: 2 }));
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
        let isamaru = CardRef { name: "Isamaru, Hound of Konda".to_string(), tag: None };
        assert_eq!(scenario.cards[0].item.card.name, "Circle of Protection: Red");
        assert_eq!(scenario.cards[0].item.words[1], Word::AttachedTo(CardRef { tag: Some("b".to_string()), ..isamaru.clone() }));
        assert_eq!(scenario.player_facts[1].item, PlayerFact::CommanderDamage { player: 1, damage: 18, from: isamaru });
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
            assert_eq!(refusal.kind, RefusalKind::Syntax, "{text}");
            assert!(refusal.message.contains(says), "{text}: {refusal}");
            assert_eq!(refusal.line, Some(text.lines().count()), "{text}");
        }
    }

    #[test]
    fn every_turn_fact_has_a_word_and_a_slot() {
        let facts: Vec<TurnFact> = every_fact().collect();
        let mut slots: Vec<usize> = facts.iter().map(|f| f.slot()).collect();
        slots.sort();
        slots.dedup();
        assert_eq!(slots.len(), TurnFact::COUNT);
        let mut words: Vec<String> = facts.into_iter().map(fact_word).collect();
        words.sort();
        words.dedup();
        assert_eq!(words.len(), TurnFact::COUNT);
    }
}
