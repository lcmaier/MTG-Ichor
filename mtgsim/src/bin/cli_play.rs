// CLI play binary — Human (CLI) vs Random bot.
//
// Usage: cargo run --bin cli_play
//        cargo run --bin cli_play -- --no-auto-pay
//
// **The two seats stack different decorators, and that is the point of a
// stack.** The human seat takes `AutoPayer` over `ManaWindowStop`: it has no
// payment policy of its own, so the generic split and the reduction order are
// answered for it and CR 601.2g's window closes once the cost is covered. It
// still *picks* which land to tap — `ManaWindowStop` only ever declines. The
// bot seat takes the stop alone; `RandomDecisionProvider` has its own tap
// preference and generic split and a payer answering those would be replacing
// the agent rather than paying for it.
//
// `--no-auto-pay` drops both, which is what CR 605.3a actually offers: the
// window keeps asking after the cost is covered, so you can float mana
// mid-cast — tap a fourth land while paying for a three-drop, or sacrifice to
// Krark-Clan Ironworks after its mana is already spoken for. That was
// unreachable before CM-4.

use std::sync::Arc;

use mtgsim::cards::registry::CardRegistry;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game::{Game, GameResult};
use mtgsim::state::game_config::GameConfig;
use mtgsim::ui::auto_payer::AutoPayer;
use mtgsim::ui::cli::CliDecisionProvider;
use mtgsim::ui::decision::{DecisionProvider, DispatchDecisionProvider};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
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
    // A `GameState` is seeded to a fixed default so tests replay; an actual game
    // of Magic wants a different shuffle every time.
    game.reseed_from_entropy();
    let auto_pay = !std::env::args().any(|a| a == "--no-auto-pay");
    if !auto_pay {
        println!("Auto-pay off: the mana window keeps asking after your cost is covered.");
    }
    let human: Box<dyn DecisionProvider> = if auto_pay {
        Box::new(AutoPayer::new(ManaWindowStop::new(CliDecisionProvider::new())))
    } else {
        Box::new(CliDecisionProvider::new())
    };
    let bot: Box<dyn DecisionProvider> = if auto_pay {
        Box::new(ManaWindowStop::new(RandomDecisionProvider::new()))
    } else {
        Box::new(RandomDecisionProvider::new())
    };
    let dp = DispatchDecisionProvider::new(vec![human, bot]);

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
