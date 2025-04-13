// src/bin/land_demo.rs
use std::io;
use std::io::Write;
use mtgsim::game::deck::Deck;
use mtgsim::game::player;
use mtgsim::game::player::Player;
use mtgsim::game::game_obj::GameObj;

fn main() {
    println!("MTG Simulator - Land Demo");

    // Create a new player
    let mut player = Player::new(1, 20, 1); // id, starting life, default lands per turn

    // Create and shuffle a basic land deck
    let mut deck = Deck::create_test_land_deck(player.id);
    println!("Created a deck with {} cards", deck.size());

    deck.shuffle();
    println!("Shuffled the deck");

    // Set the player's library
    player.set_library(deck.cards);
    
    // Draw a starting hand of 7 cards
    match player.draw_n_cards(7) {
        Ok(_) => println!("Drew 7 cards"),
        Err(e) => {
            println!("Error drawing cards: {}", e);
            return;
        }
    }

    // Display the hand to the player
    println!("\nYour hand:");
    for (i, card) in player.hand.iter().enumerate() {
        match card {
            GameObj::Card { characteristics, ..} => {
                match (&characteristics.name, &characteristics.rules_text) {
                    (Some(name), Some(rules_text)) => {
                        println!("{}: {} - {}", i + 1, name, rules_text);
                    },
                    _ => println!("{}: Unknown card", i + 1),
                }
            }
        }
    }
}