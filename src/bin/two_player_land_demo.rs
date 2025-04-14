// src/bin/two_player_land_demo.rs
use std::io;
use std::io::Write;
use mtgsim::game::deck::Deck;
use mtgsim::game::player::Player;
use mtgsim::game::game_obj::GameObj;
use mtgsim::game::gamestate::Game;
use mtgsim::utils::constants::card_types::CardType;
use mtgsim::utils::constants::turns::{Phase, Step};

fn main() {
    println!("MTG Simulator - Two Player Land Demo");

    // Create a new game
    let mut game = Game::new();

    // Create two players
    let mut pid_counter = 0;
    let player1_id = pid_counter;
    let mut player1 = Player::new(player1_id, 20, 7, 1);
    pid_counter += 1;

    let player2_id = pid_counter;
    let mut player2 = Player::new(player2_id, 20, 7, 1);

    // Create and shuffle test basic land decks
    let mut deck1 = Deck::create_test_land_deck(player1.id);
    let mut deck2 = Deck::create_test_land_deck(player2.id);

    println!("Created decks with {} cards each", deck1.size());

    deck1.shuffle();
    deck2.shuffle();
    println!("Decks shuffled.");

    // Set the player's libraries to these shuffled decks
    player1.set_library(deck1.cards.clone());
    player2.set_library(deck2.cards.clone());

    // Add the players to the game
    game.players.push(player1);
    game.players.push(player2);

    // Have both players draw 7 to start
    for player_id in [player1_id, player2_id] {
        let player = game.get_player_mut(player_id).unwrap();
        match player.draw_n_cards(player.max_hand_size as u64) {
            Ok(_) => println!("Player {} drew {} cards.", player.id, player.max_hand_size),
            Err(e) => {
                println!("Error drawing cards: {}", e);
                return;
            }
        }
    }

    // Any pregame actions would happen here

    // Setting up starting phase and step
    game.phase = Phase::Beginning;

    game.step = Some(Step::Untap);
    game.process_current_phase_and_step().unwrap();

    // game loop
    loop {
        // Get current player
        let current_player_id = game.active_player_id;
        
        println!("\n=== Player {}'s Turn ===", current_player_id);
        println!("Phase: {:?}", game.phase);
        if let Some(step) = &game.step {
            println!("Step: {:?}", step);
        }

        if game.priority_player_id == current_player_id {
            // display active player's hand 
            let active_player = game.get_player_mut(game.active_player_id).unwrap();
            match active_player.show_hand() {
                Ok(_) => println!("Displayed active player's hand."),
                Err(e) => {
                    println!("Error showing hand: {}", e);
                    return;
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

            // Player options
            println!("\nWhat would you like to do?");
                
            // Only show play land option during main phases
            if game.phase == Phase::Precombat || game.phase == Phase::Postcombat {
                println!("1. Play a land");
            }
            
            println!("2. Pass priority");
            println!("3. Quit");

            let mut choice = String::new();
            print!("> ");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut choice).unwrap();

            match choice.trim() {
                "1" => {
                    // Only allow playing land during main phases
                    if game.phase == Phase::Precombat || game.phase == Phase::Postcombat {
                        // play a land
                        // find all lands in hand
                        let player = game.get_player_ref(current_player_id).unwrap();
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

                        // get the card_id of selected card
                        let (_, card) = land_cards[selected_index];
                        let card_id = match card {
                            GameObj::Card { id, .. } => *id,
                        };

                        // Play the land using the card ID
                        match game.play_land_from_hand(current_player_id, card_id) {
                            Ok(_) => println!("Land played successfully!"),
                            Err(e) => println!("Error playing land: {}", e),
                        }
                    } else {
                        println!("You can only play lands during main phases.");
                    }
                },
                "2" => {
                    // Pass priority
                    println!("Passing priority...");
                    match game.pass_priority() {
                        Ok(phase_changed) => {
                            if phase_changed {
                                println!("All players passed, moving to next phase/step");
                            }
                        },
                        Err(e) => println!("Error passing priority: {}", e),
                    }
                },
                "3" => {
                    println!("Goodbye!");
                    break;
                },
                _ => println!("Invalid choice"),
            }
        } else {
            // Non-active player's turn (or active player doesn't have priority)
            println!("\nWaiting for Player {} to act...", game.priority_player_id);
            println!("1. Pass priority");
            println!("2. Quit");

            let mut choice = String::new();
            print!("> ");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut choice).unwrap();

            match choice.trim() {
                "1" => {
                    // Pass priority
                    println!("Passing priority...");
                    match game.pass_priority() {
                        Ok(phase_changed) => {
                            if phase_changed {
                                println!("All players passed, moving to next phase/step");
                            }
                        },
                        Err(e) => println!("Error passing priority: {}", e),
                    }
                },
                "2" => {
                    println!("Goodbye!");
                    break;
                },
                _ => println!("Invalid choice"),
            }
        }
    }
}