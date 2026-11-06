//! Every decklist in `mtgsim/decks/` against the registry
//! (`setup-architecture.md#7d-the-decklist`). A list whose cards are all
//! registered builds its deck; a Commander list holds what CR 903.3 and
//! 903.5a–b ask of one before any of its cards is registered, and prints the
//! names the registry still lacks (`cargo test --test decklists_test --
//! --nocapture`).

use std::fs;
use std::path::PathBuf;

use mtgsim::cards::registry::CardRegistry;
use mtgsim::scenario::Decklist;

fn decklists() -> Vec<(String, Decklist)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("decks");
    let mut lists: Vec<(String, Decklist)> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let text = fs::read_to_string(&path).unwrap();
            let list = Decklist::parse(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
            (name, list)
        })
        .collect();
    lists.sort_by(|a, b| a.0.cmp(&b.0));
    lists
}

/// A basic land's name (CR 205.4c's supertype, printed on these alone), which
/// CR 903.5b lets a Commander deck repeat.
fn is_basic_land(name: &str) -> bool {
    let name = name.strip_prefix("Snow-Covered ").unwrap_or(name);
    matches!(name, "Plains" | "Island" | "Swamp" | "Mountain" | "Forest" | "Wastes")
}

#[test]
fn every_decklist_parses_and_a_registered_one_builds() {
    let registry = CardRegistry::default_registry();
    let lists = decklists();
    for (name, list) in &lists {
        if list.unregistered_names(&registry).is_empty() {
            let deck = list.build_deck(&registry).unwrap_or_else(|e| panic!("{name}: {e}"));
            let written: u32 = list.deck.iter().map(|entry| entry.count).sum();
            assert_eq!(deck.len(), written as usize, "{name}");
        }
    }
}

/// No deck in `decks/` is registered yet, so this one states its own list.
#[test]
fn a_decklist_builds_in_the_order_it_was_written() {
    let list = Decklist::parse("Deck\n2 Mountain\n1 Grizzly Bears\n2 Forest\n").unwrap();
    let deck = list.build_deck(&CardRegistry::default_registry()).unwrap();
    let names: Vec<&str> = deck.iter().map(|card| card.name.as_str()).collect();
    assert_eq!(names, ["Mountain", "Mountain", "Grizzly Bears", "Forest", "Forest"]);
}

#[test]
fn a_commander_list_holds_a_hundred_cards_one_of_each_but_basics() {
    let registry = CardRegistry::default_registry();
    let lists = decklists();
    for (name, list) in lists.iter().filter(|(name, _)| name.starts_with("tdc-")) {
        assert!(!list.commander.is_empty(), "{name}: a Commander deck names its commander");
    }
    for (name, list) in lists.iter().filter(|(_, list)| !list.commander.is_empty()) {
        let commanders: u32 = list.commander.iter().map(|line| line.count).sum();
        assert!((1..=2).contains(&commanders), "{name}: one commander, or two that partner (CR 903.3, 702.124)");
        let total: u32 = list.commander.iter().chain(&list.deck).map(|entry| entry.count).sum();
        assert_eq!(total, 100, "{name}: CR 903.5a, a hundred cards with the commander");
        let mut seen: Vec<&str> = Vec::new();
        for entry in list.commander.iter().chain(&list.deck) {
            if !is_basic_land(&entry.name) {
                assert_eq!(entry.count, 1, "{name}: CR 903.5b, line {}", entry.line);
                assert!(!seen.contains(&entry.name.as_str()), "{name}: `{}` twice (CR 903.5b)", entry.name);
                seen.push(&entry.name);
            }
        }
        let missing = list.unregistered_names(&registry);
        println!("{name}: {} of its names the registry lacks", missing.len());
        for card in missing {
            println!("    {card}");
        }
    }
}
