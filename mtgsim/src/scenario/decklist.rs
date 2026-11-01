//! A decklist, in the deck sites' text (`setup-architecture.md` §7d): the
//! names a player brings, resolved against the registry into the [`Deck`]
//! CR 103.3 makes their library.
//!
//! A line is a count and a name, `4 Lightning Bolt`, under a `Commander`,
//! `Companion`, `Deck` or `Sideboard` header; a line before any header is
//! the deck's. `#` starts a comment, a name may stand on several lines, and
//! Arena's and Moxfield's `(SET) 123` after a name is read past. The parser
//! judges no section: whether a commander or a sideboard is legal is the
//! format's.

use crate::cards::registry::CardRegistry;
use crate::state::game::Deck;

use super::error::{ScenarioError, ScenarioErrorKind};

/// One line of a decklist: how many copies of which card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckEntry {
    pub count: u32,
    pub name: String,
    /// 1-based, for a refusal to name.
    pub line: usize,
}

/// A decklist's sections, each in the order its lines were written.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Decklist {
    /// "An attribute of the card itself" (CR 903.3), which the format
    /// designates as the game is built; two for partners (CR 702.124).
    pub commander: Vec<DeckEntry>,
    /// Revealed from outside the game (CR 702.139a, 103.2b).
    pub companion: Vec<DeckEntry>,
    /// The cards that become the library (CR 103.3).
    pub deck: Vec<DeckEntry>,
    /// Set aside before the game begins (CR 100.4a, 103.2a).
    pub sideboard: Vec<DeckEntry>,
}

impl Decklist {
    /// Read a decklist, refusing a line that is neither a known header nor
    /// a count and a name.
    pub fn parse(text: &str) -> Result<Decklist, ScenarioError> {
        let mut list = Decklist::default();
        let mut section = Section::Deck;
        for (ix, raw) in text.lines().enumerate() {
            let line = ix + 1;
            let content = raw.split('#').next().unwrap_or("").trim();
            if content.is_empty() {
                continue;
            }
            if let Some(header) = Section::header(content) {
                section = header;
                continue;
            }
            let (count, name) = split_count(content).ok_or_else(|| {
                ScenarioError::at(
                    ScenarioErrorKind::Syntax,
                    line,
                    format!("`{content}` is neither a count and a name nor a header (Commander, Companion, Deck, Sideboard)"),
                )
            })?;
            if count == 0 {
                return Err(ScenarioError::at(ScenarioErrorKind::Syntax, line, format!("`{name}` has a count of 0")));
            }
            let entry = DeckEntry { count, name: without_printing(name).to_string(), line };
            match section {
                Section::Commander => list.commander.push(entry),
                Section::Companion => list.companion.push(entry),
                Section::Deck => list.deck.push(entry),
                Section::Sideboard => list.sideboard.push(entry),
            }
        }
        Ok(list)
    }

    /// The deck section's cards, each built from the registry as many times
    /// as its line counts, in the order written: CR 103.3's deck, which
    /// `Game::new` makes a library. A name the registry lacks is refused with
    /// its line.
    pub fn deck(&self, registry: &CardRegistry) -> Result<Deck, ScenarioError> {
        let mut deck = Deck::new();
        for entry in &self.deck {
            let card = registry
                .create(&entry.name)
                .map_err(|_| ScenarioError::at(ScenarioErrorKind::NotACard, entry.line, format!("no registered card is named `{}`", entry.name)))?;
            deck.extend(std::iter::repeat_n(card, entry.count as usize));
        }
        Ok(deck)
    }

    /// Every line of every section, commander first.
    pub fn entries(&self) -> impl Iterator<Item = &DeckEntry> {
        self.commander.iter().chain(&self.companion).chain(&self.deck).chain(&self.sideboard)
    }

    /// The names no registered card has, each once, in the order they first
    /// appear: what a list from print still waits on.
    pub fn unregistered_names(&self, registry: &CardRegistry) -> Vec<&str> {
        let mut names: Vec<&str> = Vec::new();
        for entry in self.entries() {
            if registry.create(&entry.name).is_err() && !names.contains(&entry.name.as_str()) {
                names.push(&entry.name);
            }
        }
        names
    }
}

#[derive(Debug, Clone, Copy)]
enum Section {
    Commander,
    Companion,
    Deck,
    Sideboard,
}

impl Section {
    fn header(line: &str) -> Option<Section> {
        match line.to_ascii_lowercase().as_str() {
            "commander" => Some(Section::Commander),
            "companion" => Some(Section::Companion),
            "deck" => Some(Section::Deck),
            "sideboard" => Some(Section::Sideboard),
            _ => None,
        }
    }
}

/// `4 Lightning Bolt` or Archidekt's `4x Lightning Bolt`: the count and the
/// rest of the line.
fn split_count(line: &str) -> Option<(u32, &str)> {
    let (count, rest) = line.split_once(char::is_whitespace)?;
    let count = count.strip_suffix(['x', 'X']).unwrap_or(count).parse().ok()?;
    let name = rest.trim();
    (!name.is_empty()).then_some((count, name))
}

/// The name without a trailing `(SET)` or `(SET) 123`, which names a
/// printing and not the card.
fn without_printing(name: &str) -> &str {
    let Some(open) = name.rfind(" (") else { return name };
    let tail = &name[open + 2..];
    let Some((code, number)) = tail.split_once(')') else { return name };
    let number = number.trim();
    let is_printing = !code.is_empty()
        && code.chars().all(|c| c.is_ascii_alphanumeric())
        && (number.is_empty() || !number.contains(char::is_whitespace));
    if is_printing { name[..open].trim_end() } else { name }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(entries: &[DeckEntry]) -> Vec<(u32, &str)> {
        entries.iter().map(|e| (e.count, e.name.as_str())).collect()
    }

    #[test]
    fn each_section_holds_its_lines_in_the_order_written() {
        let list = Decklist::parse(
            "# a comment\nCommander\n1 Isamaru, Hound of Konda\n\nDeck\n10 Mountain\n4 Lightning Bolt # burn\n2 Mountain\nSideboard\n3 Shock\n",
        )
        .unwrap();
        assert_eq!(names(&list.commander), vec![(1, "Isamaru, Hound of Konda")]);
        assert_eq!(names(&list.deck), vec![(10, "Mountain"), (4, "Lightning Bolt"), (2, "Mountain")]);
        assert_eq!(names(&list.sideboard), vec![(3, "Shock")]);
        assert!(list.companion.is_empty());
        assert_eq!(list.deck[1].line, 7);
    }

    #[test]
    fn a_line_before_any_header_is_the_decks() {
        let list = Decklist::parse("20 Forest\n").unwrap();
        assert_eq!(names(&list.deck), vec![(20, "Forest")]);
    }

    #[test]
    fn a_printing_after_a_name_is_read_past() {
        let list = Decklist::parse("1 Lightning Bolt (LEA) 161\n1 Counterspell (M21)\n2x Grizzly Bears\n").unwrap();
        assert_eq!(names(&list.deck), vec![(1, "Lightning Bolt"), (1, "Counterspell"), (2, "Grizzly Bears")]);
    }

    #[test]
    fn a_word_that_is_no_header_and_no_count_is_refused() {
        let err = Decklist::parse("Deck\n4 Lightning Bolt\nPlanes\n").unwrap_err();
        assert_eq!((err.kind, err.line), (ScenarioErrorKind::Syntax, Some(3)));
        let err = Decklist::parse("0 Lightning Bolt\n").unwrap_err();
        assert_eq!((err.kind, err.line), (ScenarioErrorKind::Syntax, Some(1)));
    }

    #[test]
    fn the_deck_is_built_in_order_and_an_unknown_name_is_refused_with_its_line() {
        let registry = CardRegistry::default_registry();
        let list = Decklist::parse("Commander\n1 Nobody's Card\nDeck\n2 Mountain\n1 Lightning Bolt\n").unwrap();
        let deck = list.deck(&registry).unwrap();
        let built: Vec<&str> = deck.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(built, vec!["Mountain", "Mountain", "Lightning Bolt"]);
        assert_eq!(list.unregistered_names(&registry), vec!["Nobody's Card"]);

        let err = Decklist::parse("2 Mountain\n1 Nobody's Card\n").unwrap().deck(&registry).unwrap_err();
        assert_eq!((err.kind, err.line), (ScenarioErrorKind::NotACard, Some(2)));
    }
}
