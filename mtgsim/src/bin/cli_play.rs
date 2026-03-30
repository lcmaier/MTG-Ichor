// CLI play binary — Human (CLI) vs Random bot.
//
// Usage: cargo run --bin cli_play

use std::sync::Arc;

use mtgsim::cards::registry::CardRegistry;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game::{Game, GameResult};
use mtgsim::state::game_config::GameConfig;
use mtgsim::ui::cli::CliDecisionProvider;
use mtgsim::ui::decision::DispatchDecisionProvider;
use mtgsim::ui::random::RandomDecisionProvider;

/// Build a simple test deck: lands + creatures + spells.
fn build_test_deck(registry: &CardRegistry) -> Vec<Arc<CardData>> {
    let mut deck: Vec<Arc<CardData>> = Vec::new();

    // Lands: 10 Mountains, 10 Forests
    for _ in 0..10 {
        deck.push(registry.create("Mountain").unwrap());
    }
    for _ in 0..10 {
        deck.push(registry.create("Forest").unwrap());
    }

    // Creatures
    for _ in 0..4 {
        deck.push(registry.create("Grizzly Bears").unwrap());
    }
    for _ in 0..4 {
        deck.push(registry.create("Hill Giant").unwrap());
    }

    // Spells
    for _ in 0..4 {
        deck.push(registry.create("Lightning Bolt").unwrap());
    }

    // Pad to 40 with more lands
    while deck.len() < 40 {
        deck.push(registry.create("Mountain").unwrap());
    }

    deck
}

fn main() {
    println!("=== MTG Simulator — CLI Play ===");
    println!("You are Player 0. Your opponent (Player 1) is a random bot.");
    println!();

    let registry = CardRegistry::default_registry();
    let config = GameConfig::test();

    let deck0 = build_test_deck(&registry);
    let deck1 = build_test_deck(&registry);

    let mut game = Game::new(config, vec![deck0, deck1]).expect("Failed to create game");
    let dp = DispatchDecisionProvider::new(vec![
        Box::new(CliDecisionProvider::new()),
        Box::new(RandomDecisionProvider::new()),
    ]);

    game.setup(&dp).expect("Failed to setup game");

    println!("Game started! Each player drew 7 cards.");
    println!();

    match game.run(&dp) {
        Ok(result) => match result {
            GameResult::Winner(pid) => {
                if pid == 0 {
                    println!("\n*** YOU WIN! ***");
                } else {
                    println!("\n*** You lost. Player {} wins. ***", pid);
                }
            }
            GameResult::Draw => println!("\n*** DRAW ***"),
        },
        Err(e) => println!("\nGame error: {}", e),
    }
}
