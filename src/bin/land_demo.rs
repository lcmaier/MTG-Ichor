// src/bin/land_demo.rs
use std::io;
use std::io::Write;
use mtgsim::game::deck::Deck;
use mtgsim::game::player::Player;
use mtgsim::game::game_obj::GameObj;
use mtgsim::game::gamestate::Game;
use mtgsim::utils::constants::card_types::CardType;

fn main() {
    println!("MTG Simulator - Land Demo");

    // Create a new game
    let mut game = Game::new();

    // Create a new player
    let mut pid_counter = 0;
    let player1_id = pid_counter;
    let mut player1 = Player::new(player1_id, 20, 7, 1);
    pid_counter += 1;

    // Create and shuffle a test deck with basic lands
    let mut deck = Deck::create_test_land_deck(player1.id);
    println!("Created a deck with {} cards", deck.size());

    deck.shuffle();
    println!("Deck shuffled.");

    // Set the player's library to the shuffled deck
    player1.set_library(deck.cards.clone());

    // add the player to the game (consume the player, will be accessed via the game object)
    game.players.push(player1);

    // Draw a starting hand of 7 cards
    {
        let player = game.get_player_mut(player1_id).unwrap();
        match player.draw_n_cards(7) {
            Ok(_) => println!("Player {} drew 7 cards.", player.id),
            Err(e) => {
                println!("Error drawing cards: {}", e);
                return;
            }
        }
    }

    // Game Loop
    loop {  
        {
            let player = game.get_player_mut(player1_id).unwrap();
            // display the player's hand
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

        // Display the battlefield
        println!("\nBattlefield:");
        for (i, card) in game.battlefield.iter().enumerate() {
            match card {
                GameObj::Card { id, characteristics, owner, .. } => {
                    match (&characteristics.name, &characteristics.rules_text) {
                        (Some(name), Some(rules_text)) => {
                            println!("{}: {} - {} (Owner: {}, ID: {})", i + 1, name, rules_text, owner, id);
                        },
                        _ => println!("{}: Unknown card (ID: {})", i + 1, id),
                    }
                }
            }
        }

        // For demo purposes only, we'll give the player a choice to play a land, pass, or quit
        // Ask the player what they want to do
        println!("\nWhat would you like to do?");
        println!("1. Play a land");
        println!("2. End turn");
        println!("3. Quit");

        let mut choice = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                // play a land
                // find all lands in hand
                let player = game.get_player_ref(player1_id).unwrap();
                let land_cards: Vec<(usize, &GameObj)> = player.hand.iter()
                    .enumerate()
                    .filter(|(_, card)| {
                        let GameObj::Card { characteristics, ..} = card;
                        if let Some(card_types) = &characteristics.card_type {
                            card_types.iter().any(|t| *t == CardType::Land)
                        } else {
                            false
                        }
                    })
                    .collect();
                
                // ensure there are lands to show
                if land_cards.is_empty() {
                    println!("No lands in hand to play.");
                    continue;
                }

                // Show only the lands
                println!("Select a land to play:");
                for (i, (_, card)) in land_cards.iter().enumerate() {
                    let GameObj::Card { characteristics, ..} = card;
                    if let (Some(name), Some(rules_text)) = (&characteristics.name, &characteristics.rules_text) {
                        println!("{}: {} - {}", i + 1, name, rules_text);
                    }
                }

                // Get user selection
                let mut selection = String::new();
                print!("> ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut selection).unwrap();

                // Process selection
                let selected_index = match selection.trim().parse::<usize>() {
                    Ok(i) if i > 0 && i <= land_cards.len() => i - 1, // Convert to 0-indexed in our filtered list
                    _ => {
                        println!("Invalid selection");
                        continue;
                    }
                };

                // get the original hand index and card_id of selected card
                let (_, card) = land_cards[selected_index];
                let card_id = match card {
                    GameObj::Card { id, .. } => *id,
                };

                // Play the land using the card ID
                match game.play_land_from_hand(player1_id, card_id) {
                    Ok(_) => println!("Land played successfully!"),
                    Err(e) => println!("Error playing land: {}", e),
                }
            },
            "2" => {
                // End turn - reset land play count
                let player = game.get_player_mut(player1_id).unwrap();
                player.reset_lands_played();
                println!("Turn ended. You can play a land again.");
            },
            "3" => {
                println!("Goodbye!");
                break;
            },
            _ => println!("Invalid choice"),
        }

    }
}