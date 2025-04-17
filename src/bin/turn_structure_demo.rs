// src/bin/turn_structure_demo.rs
use std::io;
use std::io::Write;
use mtgsim::game::player::Player;
use mtgsim::utils::constants::abilities::AbilityType;
use mtgsim::utils::constants::events::{EventHandler, GameEvent};
use mtgsim::utils::constants::game_objects::{BattlefieldState, GameObj, HandState};
use mtgsim::game::gamestate::Game;
use mtgsim::utils::constants::card_types::CardType;
use mtgsim::utils::constants::deck::Deck;
use mtgsim::utils::constants::turns::PhaseType;

fn main() {
    println!("MTG Simulator - Turn Structure Demo");

    // Create a new game
    let mut game = Game::new();

    // Create two players
    let mut pid_counter = 0;
    let player1_id = pid_counter;
    let mut player1 = Player::new(player1_id, 20, 7, 1);
    pid_counter += 1;

    let player2_id = pid_counter;
    let mut player2 = Player::new(player2_id, 20, 7, 1);

    let mut deck1 = Deck::create_test_deck(player1.id);
    let mut deck2 = Deck::create_test_deck(player2.id);

    println!("Created decks with {} cards each", deck1.size());

    deck1.shuffle();
    deck2.shuffle();
    println!("Decks shuffled.");

    // Set the player's libraries
    player1.set_library(deck1.cards.clone());
    player2.set_library(deck2.cards.clone());

    // Add the players to the game
    game.players.push(player1);
    game.players.push(player2);

    // Have both players draw a starting hand
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

    // Process initial phase and step
    match game.process_current_phase() {
        Ok(_) => println!("Initial phase/step processed."),
        Err(e) => {
            println!("Error processing initial phase: {}", e);
            return;
        }
    }

    // Game loop
    loop {
        // get turn player and phase info
        let turn_player_id = game.active_player_id;

        println!("\n=== Player {}'s Turn ===", turn_player_id);
        println!("Turn Number: {}", game.turn_number);
        println!("Phase: {:?}", game.phase.phase_type);

        if let Some(ref step) = game.phase.current_step {
            println!("Step: {:?}", step.step_type);
        } else {
            println!("No current step (Main Phase)");
        }

        // Display mana pool for both players
        for player_id in [player1_id, player2_id] {
            let player = game.get_player_ref(player_id).unwrap();
            println!("Player {}'s mana pool:", player_id);
            let mana_pool = player.mana_pool.get_available_mana();
            if mana_pool.is_empty() {
                println!("  (Empty)");
            } else {
                for (mana_type, amount) in mana_pool {
                    println!("  {:?}: {}", mana_type, amount);
                }
            }
        }

        if game.priority_player_id == turn_player_id {
            // Display active player's hand 
            let active_player = game.get_player_ref(game.active_player_id).unwrap();

            println!("\nYour hand:");
            for (i, card) in active_player.hand.iter().enumerate() {
                if let Some(name) = &card.characteristics.name {
                    if let Some(rules_text) = &card.characteristics.rules_text {
                        println!("{}: {} - {}", i + 1, name, rules_text);
                    } else {
                        println!("{}: {} - (No rules text)", i + 1, name);
                    }
                } else {
                    println!("{}: Unknown card", i + 1);
                }
            }

            // Display the battlefield
            println!("\nBattlefield:");
            if game.battlefield.is_empty() {
                println!("(Empty)");
            } else {
                for (i, card) in game.battlefield.iter().enumerate() {
                    if let Some(name) = &card.characteristics.name {
                        if let Some(rules_text) = &card.characteristics.rules_text {
                            println!("{}: {} - {} (Owner: {})", i + 1, name, rules_text, card.owner);
                        } else {
                            println!("{}: {} - (No rules text) (Owner: {})", i + 1, name, card.owner);
                        }
                    } else {
                        println!("{}: Unknown card (Owner: {})", i + 1, card.owner);
                    }
                }
            }


            // Player options
            println!("\nWhat would you like to do?");
                
            // Only show play land option during main phases
            if game.phase.phase_type == PhaseType::Precombat || game.phase.phase_type == PhaseType::Postcombat {
                println!("1. Play a land");
            }
            
            println!("2. Tap a land for mana");
            println!("3. Pass priority");
            println!("4. Quit");

            let mut choice = String::new();
            print!("> ");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut choice).unwrap();

            match choice.trim() {
                "1" if game.phase.phase_type == PhaseType::Precombat || game.phase.phase_type == PhaseType::Postcombat => {
                    // Find all lands in hand
                    let player = game.get_player_ref(turn_player_id).unwrap();
                    let land_cards: Vec<(usize, &GameObj<HandState>)> = player.hand.iter()
                        .enumerate()
                        .filter(|(_, card)| card.has_card_type(&CardType::Land))
                        .collect();

                    // Ensure there are lands to show
                    if land_cards.is_empty() {
                        println!("No lands in hand to play.");
                        continue;
                    }

                    // Show (only) the lands to the user
                    println!("Select a land to play:");
                    for (i, (_, card)) in land_cards.iter().enumerate() {
                        if let (Some(name), Some(rules_text)) = (&card.characteristics.name, &card.characteristics.rules_text) {
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

                    // Get the card_id of selected card
                    let (_, card) = land_cards[selected_index];
                    let card_id = card.id;


                    // Play the land using the card ID
                    match game.play_land_from_hand(turn_player_id, card_id) {
                        Ok(_) => println!("Land played successfully!"),
                        Err(e) => println!("Error playing land: {}", e),
                    }
                },
                "2" => {
                    // Find all untapped lands on the battlefield that the player controls 
                    // (this isn't the most robust it could be, since things that aren't lands can have mana abilities, but it wil work for the alpha)
                    let untapped_lands: Vec<(usize, &GameObj<BattlefieldState>)> = game.battlefield.iter()
                        .enumerate()
                        .filter(|(_, card)| {
                            card.has_card_type(&CardType::Land) &&
                            card.state.controller == game.priority_player_id && // lands can be tapped any time you have priority
                            !card.state.tapped
                        })
                        .collect();
                    
                    if untapped_lands.is_empty() {
                        println!("You control no untapped lands.");
                        continue;
                    }

                    // Show available lands
                    println!("Select a land to tap for mana:");
                    for (i, (_, card)) in untapped_lands.iter().enumerate() {
                        if let Some(name) = &card.characteristics.name {
                            println!("{}: {}", i + 1, name);
                        } else {
                            println!("{}: Unknown land", i + 1);
                        }
                    }

                    // Get user selection
                    let mut selection = String::new();
                    print!("> ");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut selection).unwrap();

                    // Process selection
                    let selected_index = match selection.trim().parse::<usize>() {
                        Ok(i) if i > 0 && i <= untapped_lands.len() => i - 1,
                        _ => {
                            println!("Invalid selection");
                            continue;
                        }
                    };

                    // get the land from the filtered vector we created (ok to index since we built it, so no index instability)
                    let (_, land) = untapped_lands[selected_index];
                    let land_id = land.id;

                    // attempt to activate the mana ability
                    // NOTE: This implementation precludes multiple mana abilities on the same card, find only returns the first value it finds
                    // will need to refactor this to be more robust later, but in our alpha the only mana sources are Forest and Mountain basic lands,
                    // which only have a single mana ability each, so we're good for now.
                    if let Some(abilities) = &land.characteristics.abilities {
                        if let Some(mana_ability_definition) = abilities.iter()
                            .find(|ability| ability.ability_type == AbilityType::Mana) {
                            // If we found a mana ability on the land, create and process mana ability activation event
                            let mana_activation_event = GameEvent::ManaAbilityActivated { 
                                source_id: land_id, 
                                player_id: game.priority_player_id 
                            };

                            match game.handle_event(&mana_activation_event) {
                                Ok(_) => println!("Mana ability activated successfully!"),
                                Err(e) => println!("Error activating mana ability: {}", e),
                            }
                        } else {
                            println!("Selected land has no mana ability.");
                        }
                    } else {
                        println!("Selected land has no abilities.");
                    }
                },
                "3" => {
                    // Pass priority
                    println!("Passing priority...");
                    match game.pass_priority() {
                        Ok(phase_changed) => {
                            if phase_changed {
                                println!("Phase or step advanced");
                            }
                        },
                        Err(e) => println!("Error passing priority: {}", e),
                    }
                },
                "4" => {
                    println!("Goodbye!");
                    break;
                },
                _ => println!("Invalid choice"),
            }
        } else {
            // Non-active player's turn (or active player doesn't have priority)
            println!("\nWaiting for Player {} to act...", game.priority_player_id);
             // numbers are offset to line up with active player options
            println!("2. Tap a land for mana");
            println!("3. Pass priority");
            println!("4. Quit");

            // Get user selection
            let mut selection = String::new();
            print!("> ");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut selection).unwrap();

            match selection.trim() {
                "2" => {
                    // Find all untapped lands on the battlefield that the player controls 
                    // (this isn't the most robust it could be, since things that aren't lands can have mana abilities, but it wil work for the alpha)
                    let untapped_lands: Vec<(usize, &GameObj<BattlefieldState>)> = game.battlefield.iter()
                        .enumerate()
                        .filter(|(_, card)| {
                            card.has_card_type(&CardType::Land) &&
                            card.state.controller == game.priority_player_id && // lands can be tapped any time you have priority
                            !card.state.tapped
                        })
                        .collect();
                    
                    if untapped_lands.is_empty() {
                        println!("You control no untapped lands.");
                        continue;
                    }

                    // Show available lands
                    println!("Select a land to tap for mana:");
                    for (i, (_, card)) in untapped_lands.iter().enumerate() {
                        if let Some(name) = &card.characteristics.name {
                            println!("{}: {}", i + 1, name);
                        } else {
                            println!("{}: Unknown land", i + 1);
                        }
                    }

                    // Get user selection
                    let mut selection = String::new();
                    print!("> ");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut selection).unwrap();

                    // Process selection
                    let selected_index = match selection.trim().parse::<usize>() {
                        Ok(i) if i > 0 && i <= untapped_lands.len() => i - 1,
                        _ => {
                            println!("Invalid selection");
                            continue;
                        }
                    };

                    // get the land from the filtered vector we created (ok to index since we built it, so no index instability)
                    let (_, land) = untapped_lands[selected_index];
                    let land_id = land.id;

                    // attempt to activate the mana ability
                    // NOTE: This implementation precludes multiple mana abilities on the same card, find only returns the first value it finds
                    // will need to refactor this to be more robust later, but in our alpha the only mana sources are Forest and Mountain basic lands,
                    // which only have a single mana ability each, so we're good for now.
                    if let Some(abilities) = &land.characteristics.abilities {
                        if let Some(mana_ability_definition) = abilities.iter()
                            .find(|ability| ability.ability_type == AbilityType::Mana) {
                            // If we found a mana ability on the land, create and process mana ability activation event
                            let mana_activation_event = GameEvent::ManaAbilityActivated { 
                                source_id: land_id, 
                                player_id: game.priority_player_id 
                            };

                            match game.handle_event(&mana_activation_event) {
                                Ok(_) => println!("Mana ability activated successfully!"),
                                Err(e) => println!("Error activating mana ability: {}", e),
                            }
                        } else {
                            println!("Selected land has no mana ability.");
                        }
                    } else {
                        println!("Selected land has no abilities.");
                    }
                },
                "3" => {
                    // Pass priority
                    println!("Passing priority...");
                    match game.pass_priority() {
                        Ok(phase_changed) => {
                            if phase_changed {
                                println!("Phase or step advanced");
                            }
                        },
                        Err(e) => println!("Error passing priority: {}", e),
                    }
                },
                "4" => {
                    println!("Goodbye!");
                    break;
                },
                _ => println!("Invalid choice"),
            }
        }
    }
}