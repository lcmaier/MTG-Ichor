// src/game/ui/combat.rs

use std::io::{self, Write};

use crate::{game::gamestate::Game, utils::constants::{card_types::CardType, combat::{AttackTarget, CombatDamageAssignment, DamageRecipient}, id_types::{ObjectId, PlayerId}}};

/// Relevant CLI UI structs for combat
pub struct AttackerUI;
pub struct BlockerUI;
pub struct DamageAssignmentUI;

#[derive(Debug, Clone)]
pub struct AttackDeclaration {
    pub attacker_id: ObjectId,
    pub target: AttackTarget,
}

impl AttackerUI {
    /// Get attack decisions from the active player (main access function for declaring attackers)
    pub fn get_attack_decisions(game: &Game) -> Result<Vec<AttackDeclaration>, String> {
        let potential_attackers = Self::get_potential_attackers(game);

        if potential_attackers.is_empty() {
            println!("No creatures available to attack with.");
            return Ok(vec![]);
        }

        println!("\nDECLARE ATTACKERS");
        Self::display_potential_attackers(&potential_attackers);

        let selected_attackers = Self::select_attackers(&potential_attackers)?;

        if selected_attackers.is_empty() {
            println!("No attackers declared.");
            return Ok(vec![]);
        }

        // For now, all attackers attack the defending player
        // TODO: Allow selection of defending player(s), planeswalker(s), or battle(s)
        let defending_player_id = if game.active_player_id == 0 { 1 } else { 0 };
        let target = AttackTarget::Player(defending_player_id);

        // Generate attack declarations
        let declarations = selected_attackers
            .into_iter()
            .map(|attacker_id| AttackDeclaration {
                attacker_id,
                target: target.clone(),
            })
            .collect();

        Ok(declarations)

    }
    
    /// Get all creatures that can attack
    fn get_potential_attackers(game: &Game) -> Vec<(ObjectId, String, i32, i32)> {
        game.battlefield.values()
            .filter_map(|obj| {
                // Check if this is a valid attacker
                if obj.state.controller == game.active_player_id &&
                   obj.has_card_type(&crate::utils::constants::card_types::CardType::Creature) &&
                   !obj.state.tapped {
                    
                    if let Some(creature) = &obj.state.creature {
                        // Check summoning sickness
                        if !creature.summoning_sick {
                            // TODO: || obj.has_keyword(Keyword::Haste)
                            let name = obj.characteristics.name.clone()
                                .unwrap_or_else(|| format!("Creature {}", obj.id));
                            
                            return Some((
                                obj.id,
                                name,
                                creature.current_power,
                                creature.current_toughness
                            ));
                        }
                    }
                }
                None
            })
            .collect()
    }


    /// Display all creatures that can attack
    fn display_potential_attackers(attackers: &[(ObjectId, String, i32, i32)]) {
        for (i, (_, name, power, toughness)) in attackers.iter().enumerate() {
            println!("{}. {} ({}/{})", i + 1, name, power, toughness);
        }
    }

    /// Display currently selected attackers
    fn display_selected_attackers(selected: &[ObjectId], all_attackers: &[(ObjectId, String, i32, i32)]) {
        print!("\nCurrently selected attackers: ");
        if selected.is_empty() {
            println!("None");
        } else {
            let infos: Vec<String> = selected.iter()
                .filter_map(|id| {
                    all_attackers.iter()
                        .find(|(attacker_id, _, _, _)| attacker_id == id)
                        .map(|(_, name, power, toughness)| format!("{} ({}/{})", name, power, toughness))
                })
                .collect();
            println!("{}", infos.join(", "));
        }
    }


    /// Let player select which creatures will attack
    fn select_attackers(potential_attackers: &[(ObjectId, String, i32, i32)]) -> Result<Vec<ObjectId>, String> {
        let mut selected_attackers = Vec::new();
        
        loop {
            Self::display_selected_attackers(&selected_attackers, potential_attackers);
            println!("Select a creature to attack (1-{}), or 0 to finish declaring attackers:", 
                potential_attackers.len());
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)
                .map_err(|e| format!("Failed to read input: {}", e))?;
            
            match input.trim().parse::<usize>() {
                Ok(0) => break,
                Ok(num) if num > 0 && num <= potential_attackers.len() => {
                    let (attacker_id, name, _, _) = &potential_attackers[num - 1];
                    
                    if !selected_attackers.contains(attacker_id) {
                        selected_attackers.push(*attacker_id);
                        println!("{} added to attackers.", name);
                    } else {
                        selected_attackers.retain(|&id| id != *attacker_id);
                        println!("{} removed from attackers.", name);
                    }
                }
                _ => {
                    println!("Invalid input. Please enter a number between 1 and {} or 0 to finish.", 
                        potential_attackers.len());
                }
            }
        }
        
        Ok(selected_attackers)
    }
}







#[derive(Debug, Clone)]
pub struct BlockDeclaration {
    pub blocker_id: ObjectId,
    pub attacker_id: ObjectId,
}


impl BlockerUI {
    /// Get block decisions from the defending player
    pub fn get_block_decisions(game: &Game) -> Result<Vec<BlockDeclaration>, String> {
        let defending_player_id = if game.active_player_id == 0 { 1 } else { 0 };
        
        // Get all attacking creatures
        let attackers = Self::get_attackers(game);
        if attackers.is_empty() {
            println!("No attackers to block.");
            return Ok(vec![]);
        }

        // Get all potential blockers
        let potential_blockers = Self::get_potential_blockers(game, defending_player_id);
        if potential_blockers.is_empty() {
            println!("No creatures available to block with.");
            return Ok(vec![]);
        }

        println!("\nDECLARE BLOCKERS");
        println!("Defending player: Player {}", defending_player_id);
        
        let mut block_declarations: Vec<BlockDeclaration> = Vec::new();
        let mut blockers_used: Vec<ObjectId> = Vec::new();

        // For each attacker, allow the defending player to assign blockers
        for (attacker_id, attacker_name, power, toughness) in &attackers {
            println!("\n--- Blocking {} ({}/{}) ---", attacker_name, power, toughness);
            
            // Get available blockers (excluding those already blocking, unless they can block multiple)
            let available_blockers: Vec<_> = potential_blockers.iter()
                .filter(|(id, _, _, _, max_blocks)| {
                    let times_blocking = blockers_used.iter().filter(|&&bid| bid == *id).count();
                    times_blocking < *max_blocks as usize
                })
                .collect();

            if available_blockers.is_empty() {
                println!("No available blockers for this attacker.");
                continue;
            }

            // Display available blockers
            println!("Available blockers:");
            for (i, (_, name, power, toughness, _)) in available_blockers.iter().enumerate() {
                println!("  {}. {} ({}/{})", i + 1, name, power, toughness);
            }
            println!("  0. Done blocking this attacker");

            // Allow multiple blockers per attacker
            loop {
                println!("\nSelect a blocker for {} (or 0 to finish):", attacker_name);
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)
                    .map_err(|_| "Failed to read input")?;
                
                match input.trim().parse::<usize>() {
                    Ok(0) => break, // Done blocking this attacker
                    Ok(choice) if choice <= available_blockers.len() => {
                        let (blocker_id, blocker_name, _, _, _) = available_blockers[choice - 1];
                        
                        block_declarations.push(BlockDeclaration {
                            blocker_id: *blocker_id,
                            attacker_id: *attacker_id,
                        });
                        blockers_used.push(*blocker_id);
                        
                        println!("{} is now blocking {}.", blocker_name, attacker_name);
                        
                        // Check if we can add more blockers
                        let remaining_available = available_blockers.iter()
                            .filter(|(id, _, _, _, max_blocks)| {
                                let times_blocking = blockers_used.iter().filter(|&&bid| bid == *id).count();
                                times_blocking < *max_blocks as usize
                            })
                            .count();
                        
                        if remaining_available == 0 {
                            println!("No more blockers available for this attacker.");
                            break;
                        }
                    }
                    _ => {
                        println!("Invalid input. Please enter a number between 0 and {}.", 
                            available_blockers.len());
                    }
                }
            }
        }

        // Summary of blocks
        if !block_declarations.is_empty() {
            println!("\n--- Block Summary ---");
            let nameless = "No name".to_string();
            for declaration in &block_declarations {
                let blocker_name = game.battlefield.get(&declaration.blocker_id)
                    .and_then(|obj| obj.characteristics.name.as_ref())
                    .unwrap_or(&nameless);
                let attacker_name = game.battlefield.get(&declaration.attacker_id)
                    .and_then(|obj| obj.characteristics.name.as_ref())
                    .unwrap_or(&nameless);
                println!("{} blocks {}", blocker_name, attacker_name);
            }
        } else {
            println!("\nNo blocks declared.");
        }

        Ok(block_declarations)
    }

    /// Get all attacking creatures
    fn get_attackers(game: &Game) -> Vec<(ObjectId, String, i32, i32)> {
        game.battlefield.values()
            .filter_map(|obj| {
                if let Some(creature) = &obj.state.creature {
                    if creature.attacking.is_some() {
                        let name = obj.characteristics.name.clone()
                            .unwrap_or_else(|| format!("Creature {}", obj.id));
                        let power = obj.characteristics.power.unwrap_or(0);
                        let toughness = obj.characteristics.toughness.unwrap_or(0);
                        return Some((obj.id, name, power, toughness));
                    }
                }
                None
            })
            .collect()
    }

    /// Get all potential blockers for the defending player
    fn get_potential_blockers(game: &Game, defending_player_id: PlayerId) -> Vec<(ObjectId, String, i32, i32, u32)> {
        game.battlefield.values()
            .filter_map(|obj| {
                // Check if this is a valid blocker
                if obj.state.controller == defending_player_id &&
                   obj.has_card_type(&CardType::Creature) &&
                   !obj.state.tapped {
                    
                    if let Some(creature) = &obj.state.creature {
                        // TODO: Check for "can't block" effects
                        let name = obj.characteristics.name.clone()
                            .unwrap_or_else(|| format!("Creature {}", obj.id));
                        let power = obj.characteristics.power.unwrap_or(0);
                        let toughness = obj.characteristics.toughness.unwrap_or(0);
                        let max_blocks = creature.blocking.as_ref()
                            .map(|b| b.max_can_block)
                            .unwrap_or(1); // Default: can block one creature
                        
                        return Some((obj.id, name, power, toughness, max_blocks));
                    }
                }
                None
            })
            .collect()
    }
}








impl DamageAssignmentUI {
    /// Get damage assignments from both players
    pub fn get_damage_assignments(game: &Game) -> Result<Vec<CombatDamageAssignment>, String> {
        let mut assignments = Vec::new();
        
        // Get damage assignments from active player first
        println!("\n--- Active Player Damage Assignment ---");
        let active_assignments = Self::get_assignments_for_player(game, game.active_player_id)?;
        assignments.extend(active_assignments);
        
        // Get damage assignments from defending player
        let defending_player_id = if game.active_player_id == 0 { 1 } else { 0 };
        println!("\n--- Defending Player Damage Assignment ---");
        let defending_assignments = Self::get_assignments_for_player(game, defending_player_id)?;
        assignments.extend(defending_assignments);
        
        Ok(assignments)
    }

    fn get_assignments_for_player(game: &Game, player_id: PlayerId) -> Result<Vec<CombatDamageAssignment>, String> {
        let mut assignments = Vec::new();

        // Find all creatures that need to assign damage
        let creatures_to_assign: Vec<_> = game.battlefield.values()
            .filter_map(|obj| {
                if obj.state.controller != player_id { return None; }

                if let Some(creature) = &obj.state.creature {
                    if creature.current_power < 0 { return None; } // Creatures with 0 or less power don't assign damage
                

                    let name = obj.characteristics.name.clone()
                        .unwrap_or_else(|| format!("Creature {}", obj.id));

                    //// check if this creature is attacking or blocking
                    // attacking
                    if let Some(attacking) = &creature.attacking {
                        let targets = if attacking.blocked_by.is_empty() {
                            // Attacker is unblocked, only target is the AttackTarget
                            vec![match &attacking.target {
                                AttackTarget::Player(pid) => DamageRecipient::Player(*pid),
                                AttackTarget::Planeswalker(id) => DamageRecipient::Planeswalker(*id),
                                AttackTarget::Battle(id) => DamageRecipient::Battle(*id),
                            }]
                        } else {
                            // Blocked -- targets are all the blockers TODO: handle trample (i.e. the defending player would still be a target (but only after all blockers have been assigned lethal damage))
                            let blocked_targets: Vec<DamageRecipient> = attacking.blocked_by.iter()
                                .map(|blocker_id| DamageRecipient::Creature(*blocker_id))
                                .collect();

                            // TODO: For trample, add the original target to the list after blockers
                            // if creature.has_keyword(Keyword::Trample) {
                            //     targets.push(match &attacking.target {
                            //         AttackTarget::Player(pid) => DamageRecipient::Player(*pid),
                            //         AttackTarget::Planeswalker(id) => DamageRecipient::Planeswalker(*id),
                            //         AttackTarget::Battle(id) => DamageRecipient::Battle(*id),
                            //     });
                            // }

                            blocked_targets
                        };

                        if !targets.is_empty() {
                            // is_first_strike and is_trample_damage are hardcoded false for now
                            // TODO: Handle first strike and trample in the future
                            return Some((obj.id, name, creature.current_power, targets, false, false))
                        } else {
                            return None
                        }
                    }

                    // blocking
                    if let Some(blocking) = &creature.blocking {
                        // If this creature is blocking, it can assign damage to the attacker(s)
                        let targets: Vec<DamageRecipient> = blocking.blocking.iter()
                            .map(|attacker_id| DamageRecipient::Creature(*attacker_id))
                            .collect();

                        if !targets.is_empty() {
                            return Some((obj.id, name, creature.current_power, targets, false, false))
                        } else {
                            return None
                        }
                    }
                }
                None
        })
        .collect();

        // Get assignments for each creature that will assign damage
        for (creature_id, creature_name, power, targets, is_first_strike, is_trample) in creatures_to_assign {
            println!("\nAssigning {} damage for {}", power, creature_name);
            
            if targets.len() == 1 {
                // Single target - all damage must go here
                let target_name = Self::get_target_name(game, &targets[0]);
                println!("All damage goes to {}", target_name);
                assignments.push(CombatDamageAssignment {
                    source_id: creature_id,
                    target_id: targets[0].clone(),
                    amount: power as u32,
                    is_first_strike,
                    is_trample,
                });
            } else {
                // Multiple targets - get manual assignment
                let distributed_assignments  = Self::get_damage_distribution(game, creature_id, power as u32, &targets, is_first_strike, is_trample)?;
                assignments.extend(distributed_assignments);
            }
        }
        Ok(assignments)
    }

    fn get_damage_distribution(
        game: &Game,
        source_id: ObjectId,
        total_damage: u32, 
        targets: &[DamageRecipient],
        is_first_strike: bool,
        is_trample: bool,
    ) -> Result<Vec<CombatDamageAssignment>, String> {
        let mut assignments = Vec::new();
        let mut remaining_damage = total_damage;
        
        println!("Distribute {} damage among {} targets:", total_damage, targets.len());
        
        for (i, target) in targets.iter().enumerate() {
            if remaining_damage == 0 {
                break;
            }
            
            let target_name = Self::get_target_name(game, target);
            let target_info = Self::get_target_info(game, target);
            
            println!("\n{}", target_name);
            if let Some((toughness, damage_marked)) = target_info {
                println!("  Toughness: {}, Damage marked: {}", 
                    toughness, damage_marked);
            }
            println!("Remaining damage to assign: {}", remaining_damage);
            
            let damage_to_assign = if i == targets.len() - 1 {
                // Last target gets all remaining damage
                println!("Assigning remaining {} damage", remaining_damage);
                remaining_damage
            } else {
                print!("Damage to assign (0-{}): ", remaining_damage);
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                
                match input.trim().parse::<u32>() {
                    Ok(damage) if damage <= remaining_damage => damage,
                    _ => {
                        println!("Invalid input. Assigning 0 damage.");
                        0
                    }
                }
            };

            if damage_to_assign > 0 {
                // Create the assignment
                assignments.push(CombatDamageAssignment {
                    source_id,
                    target_id: target.clone(),
                    amount: damage_to_assign,
                    is_first_strike,
                    is_trample,
                });
                remaining_damage -= damage_to_assign;
                
                println!("Assigned {} damage to {}", damage_to_assign, target_name);
            } else {
                println!("No damage assigned to {}", target_name);
            }
        }
        
        Ok(assignments)
    }

    /// Get a display name for a damage target
    fn get_target_name(game: &Game, target: &DamageRecipient) -> String {
        match target {
            DamageRecipient::Player(pid) => format!("Player {}", pid),
            DamageRecipient::Creature(id) => {
                game.battlefield.get(id)
                    .and_then(|obj| obj.characteristics.name.clone())
                    .unwrap_or_else(|| format!("Creature {}", id))
            }
            DamageRecipient::Planeswalker(id) => {
                game.battlefield.get(id)
                    .and_then(|obj| obj.characteristics.name.clone())
                    .unwrap_or_else(|| format!("Planeswalker {}", id))
            }
            DamageRecipient::Battle(id) => {
                game.battlefield.get(id)
                    .and_then(|obj| obj.characteristics.name.clone())
                    .unwrap_or_else(|| format!("Battle {}", id))
            }
        }
    }
    
    /// Get toughness/damage info for a target (if applicable)
    fn get_target_info(game: &Game, target: &DamageRecipient) -> Option<(i32, u32)> {
        match target {
            DamageRecipient::Creature(id) => {
                game.battlefield.get(id)
                    .and_then(|obj| obj.state.creature.as_ref())
                    .map(|creature| {
                        let toughness = creature.current_toughness;
                        let damage_marked = creature.damage_marked;
                        (toughness, damage_marked)
                    })
            }
            DamageRecipient::Planeswalker(id) => {
                // TODO: Get loyalty counters when planeswalkers are implemented
                None
            }
            DamageRecipient::Battle(id) => {
                // TODO: Get loyalty counters when battles are implemented
                None
            }
            DamageRecipient::Player(_) => {
                // Players don't have toughness or damage marked, so we return None
                None
            }
        }
    }
}
