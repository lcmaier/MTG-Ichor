// CLI DecisionProvider — interactive terminal play via stdin/stdout.
//
// Implements all 8 DecisionProvider methods. Uses oracle/mana_helpers to
// show affordable spells and suggest land taps. Retries on bad input.

use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use crate::engine::resolve::ResolvedTarget;
use crate::events::event::DamageTarget;
use crate::ui::display;
use crate::oracle::legality::{legal_attackers, legal_blockers, playable_lands};
use crate::oracle::mana_helpers::castable_spells;
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::GameState;
use crate::types::effects::TargetSpec;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaType};
use crate::ui::decision::{
    auto_allocate_generic, default_damage_assignment, default_trample_assignment,
    is_action_still_valid, queue_tap_and_cast, DecisionProvider, PriorityAction,
};

/// Interactive CLI decision provider for human play.
///
/// Reads from stdin and writes prompts to stdout. The internal action queue
/// supports "tap-and-cast" shortcuts: when the player selects a spell to cast,
/// we queue up the mana ability activations followed by the CastSpell action.
pub struct CliDecisionProvider {
    action_queue: std::cell::RefCell<std::collections::VecDeque<PriorityAction>>,
}

impl CliDecisionProvider {
    pub fn new() -> Self {
        CliDecisionProvider {
            action_queue: std::cell::RefCell::new(std::collections::VecDeque::new()),
        }
    }
}

impl Default for CliDecisionProvider {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Input helpers
// ---------------------------------------------------------------------------

fn read_line() -> String {
    print!("> ");
    io::stdout().flush().ok();
    let mut buf = String::new();
    io::stdin().lock().read_line(&mut buf).ok();
    buf.trim().to_string()
}

fn read_usize(prompt: &str, max: usize) -> Option<usize> {
    println!("{}", prompt);
    let input = read_line();
    if input.is_empty() || input.eq_ignore_ascii_case("none") {
        return None;
    }
    match input.parse::<usize>() {
        Ok(n) if n < max => Some(n),
        _ => {
            println!("Invalid choice (0..{})", max.saturating_sub(1));
            None
        }
    }
}

/// Parse a list of usize indices from user input.
///
/// Input format: comma-separated or whitespace-separated integers.
/// Examples: "0, 2, 3" or "0 2 3" or "0,2,3".
/// Enter "none" or empty string to return an empty list.
/// Values >= `max` are silently filtered out.
fn read_usize_list(prompt: &str, max: usize) -> Vec<usize> {
    println!("{}", prompt);
    let input = read_line();
    if input.is_empty() || input.eq_ignore_ascii_case("none") {
        return Vec::new();
    }
    input
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<usize>().ok())
        .filter(|&n| n < max)
        .collect()
}

// ---------------------------------------------------------------------------
// DecisionProvider implementation
// ---------------------------------------------------------------------------

impl DecisionProvider for CliDecisionProvider {
    fn choose_priority_action(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> PriorityAction {
        // Drain queue if non-empty, validating each action
        {
            let mut queue = self.action_queue.borrow_mut();
            if let Some(action) = queue.pop_front() {
                if is_action_still_valid(game, player_id, &action) {
                    return action;
                }
                // Stale plan — discard remaining queued actions
                queue.clear();
            }
        }

        // Show game state
        println!("\n{}", "=".repeat(60));
        println!(
            "Turn {} — {} — Active: P{}",
            game.turn_number,
            display::format_phase(game),
            game.active_player,
        );
        for pid in 0..game.num_players() {
            println!("{}", display::format_player_summary(game, pid));
        }
        println!("Mana pool: {}", display::format_mana_pool(game, player_id));
        println!("Stack:\n{}", display::format_stack(game));
        println!(
            "Your battlefield:\n{}",
            display::format_battlefield(game, player_id)
        );
        println!("Your hand:\n{}", display::format_hand(game, player_id));
        println!("{}", "-".repeat(60));

        // Collect options
        let lands = playable_lands(game, player_id);
        let castable = castable_spells(game, player_id);

        if !lands.is_empty() {
            println!("Playable lands:");
            for (i, &id) in lands.iter().enumerate() {
                println!("  L{}: {}", i, display::card_name(game, id));
            }
        }
        if !castable.is_empty() {
            println!("Castable spells (auto-tap):");
            for (i, (id, _sources)) in castable.iter().enumerate() {
                let name = display::card_name(game, *id);
                let cost_str = game
                    .objects
                    .get(id)
                    .and_then(|o| o.card_data.mana_cost.as_ref())
                    .map(|c| format!(" {}", c))
                    .unwrap_or_default();
                println!("  C{}: {}{}", i, name, cost_str);
            }
        }

        println!("\nActions: [P]ass | L<n> play land | C<n> cast spell");

        loop {
            let input = read_line();
            let input_upper = input.to_uppercase();

            if input_upper == "P" || input_upper == "PASS" {
                return PriorityAction::Pass;
            }

            // L<n> — play land
            if input_upper.starts_with('L') {
                if let Ok(idx) = input[1..].trim().parse::<usize>() {
                    if idx < lands.len() {
                        return PriorityAction::PlayLand(lands[idx]);
                    }
                }
                println!("Invalid land index.");
                continue;
            }

            // C<n> — cast spell (with auto-tap)
            if input_upper.starts_with('C') {
                if let Ok(idx) = input[1..].trim().parse::<usize>() {
                    if idx < castable.len() {
                        let (card_id, ref sources) = castable[idx];
                        return queue_tap_and_cast(
                            &self.action_queue,
                            sources,
                            card_id,
                        );
                    }
                }
                println!("Invalid spell index.");
                continue;
            }

            println!("Unknown command. Try P, L0, C0, etc.");
        }
    }

    fn choose_attackers(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> Vec<(ObjectId, AttackTarget)> {
        let available = legal_attackers(game, player_id);
        if available.is_empty() {
            return Vec::new();
        }

        println!("\n--- Declare Attackers ---");
        println!("Available attackers:");
        for (i, &id) in available.iter().enumerate() {
            println!("  {}: {}", i, display::format_permanent(game, id));
        }

        // Build list of valid attack targets (opponents + planeswalkers/battles)
        let mut targets: Vec<(String, AttackTarget)> = Vec::new();
        for pid in 0..game.num_players() {
            if pid != player_id {
                targets.push((format!("Player {}", pid), AttackTarget::Player(pid)));
            }
        }
        // TODO: Add planeswalker and battle targets when those card types
        // are implemented. They use AttackTarget::Planeswalker(id) and
        // AttackTarget::Battle(id) respectively.

        let indices = read_usize_list(
            "Enter attacker indices (comma-separated, or 'none'):",
            available.len(),
        );

        if indices.is_empty() {
            return Vec::new();
        }

        // In 2-player games, skip the target prompt — only one opponent
        if targets.len() == 1 {
            return indices
                .into_iter()
                .map(|i| (available[i], targets[0].1.clone()))
                .collect();
        }

        // Multiplayer: ask which target for each attacker
        println!("Attack targets:");
        for (i, (label, _)) in targets.iter().enumerate() {
            println!("  {}: {}", i, label);
        }

        let mut result = Vec::new();
        for idx in indices {
            let creature = available[idx];
            println!(
                "Target for {} (0..{}, default 0):",
                display::card_name(game, creature),
                targets.len() - 1,
            );
            let target_idx = read_line()
                .parse::<usize>()
                .unwrap_or(0)
                .min(targets.len() - 1);
            result.push((creature, targets[target_idx].1.clone()));
        }
        result
    }

    fn choose_blockers(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> Vec<(ObjectId, ObjectId)> {
        let available = legal_blockers(game, player_id);
        if available.is_empty() {
            return Vec::new();
        }

        // Find attacking creatures
        let attackers: Vec<ObjectId> = game
            .battlefield
            .iter()
            .filter(|(_, e)| e.attacking.is_some())
            .map(|(id, _)| *id)
            .collect();

        if attackers.is_empty() {
            return Vec::new();
        }

        println!("\n--- Declare Blockers ---");
        println!("Attacking creatures:");
        for (i, &id) in attackers.iter().enumerate() {
            println!("  A{}: {}", i, display::format_permanent(game, id));
        }
        println!("Your creatures:");
        for (i, &id) in available.iter().enumerate() {
            println!("  B{}: {}", i, display::format_permanent(game, id));
        }

        println!("Enter blocks as 'B<n> A<m>' pairs (one per line, empty to finish):");

        let mut blocks = Vec::new();
        loop {
            let input = read_line();
            if input.is_empty() {
                break;
            }

            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() != 2 {
                println!("Format: B<n> A<m>");
                continue;
            }

            let blocker_idx = parts[0]
                .trim_start_matches(|c: char| c == 'B' || c == 'b')
                .parse::<usize>();
            let attacker_idx = parts[1]
                .trim_start_matches(|c: char| c == 'A' || c == 'a')
                .parse::<usize>();

            match (blocker_idx, attacker_idx) {
                (Ok(bi), Ok(ai)) if bi < available.len() && ai < attackers.len() => {
                    blocks.push((available[bi], attackers[ai]));
                }
                _ => println!("Invalid indices."),
            }
        }

        blocks
    }

    fn choose_discard(
        &self,
        game: &GameState,
        player_id: PlayerId,
    ) -> Option<ObjectId> {
        let player = game.players.get(player_id)?;
        if player.hand.is_empty() {
            return None;
        }

        println!("\n--- Discard ---");
        println!("Hand:\n{}", display::format_hand(game, player_id));

        let idx = read_usize("Choose card to discard:", player.hand.len())?;
        Some(player.hand[idx])
    }

    fn choose_targets(
        &self,
        game: &GameState,
        _player_id: PlayerId,
        target_spec: &TargetSpec,
    ) -> Vec<ResolvedTarget> {
        println!("\n--- Choose Targets ---");
        println!("Target spec: {:?}", target_spec);

        match target_spec {
            TargetSpec::None | TargetSpec::You => Vec::new(),
            TargetSpec::Player(_) => {
                println!("Players:");
                for pid in 0..game.num_players() {
                    println!(
                        "  {}: {}",
                        pid,
                        display::format_player_summary(game, pid)
                    );
                }
                match read_usize("Choose target player:", game.num_players()) {
                    Some(pid) => vec![ResolvedTarget::Player(pid)],
                    None => Vec::new(),
                }
            }
            TargetSpec::Creature(_) => {
                let creatures: Vec<ObjectId> = game
                    .battlefield
                    .keys()
                    .copied()
                    .filter(|&id| crate::oracle::characteristics::is_creature(game, id))
                    .collect();
                println!("Creatures:");
                for (i, &id) in creatures.iter().enumerate() {
                    println!("  {}: {}", i, display::format_permanent(game, id));
                }
                match read_usize("Choose target creature:", creatures.len()) {
                    Some(idx) => vec![ResolvedTarget::Object(creatures[idx])],
                    None => Vec::new(),
                }
            }
            TargetSpec::Any(_) => {
                println!("Players:");
                for pid in 0..game.num_players() {
                    println!(
                        "  P{}: {}",
                        pid,
                        display::format_player_summary(game, pid)
                    );
                }
                let creatures: Vec<ObjectId> = game
                    .battlefield
                    .keys()
                    .copied()
                    .filter(|&id| crate::oracle::characteristics::is_creature(game, id))
                    .collect();
                if !creatures.is_empty() {
                    println!("Creatures:");
                    for (i, &id) in creatures.iter().enumerate() {
                        println!(
                            "  C{}: {}",
                            i,
                            display::format_permanent(game, id)
                        );
                    }
                }
                println!("Enter P<n> for player or C<n> for creature:");
                let input = read_line();
                let upper = input.to_uppercase();
                if upper.starts_with('P') {
                    if let Ok(pid) = input[1..].trim().parse::<usize>() {
                        if pid < game.num_players() {
                            return vec![ResolvedTarget::Player(pid)];
                        }
                    }
                } else if upper.starts_with('C') {
                    if let Ok(idx) = input[1..].trim().parse::<usize>() {
                        if idx < creatures.len() {
                            return vec![ResolvedTarget::Object(creatures[idx])];
                        }
                    }
                }
                Vec::new()
            }
            _ => {
                println!("Unsupported target spec {:?}, returning empty.", target_spec);
                Vec::new()
            }
        }
    }

    fn choose_attacker_damage_assignment(
        &self,
        game: &GameState,
        _player_id: PlayerId,
        attacker_id: ObjectId,
        blockers: &[ObjectId],
        power: u64,
    ) -> Vec<(ObjectId, u64)> {
        println!(
            "\n--- Damage Assignment for {} (power {}) ---",
            display::card_name(game, attacker_id),
            power,
        );
        println!("Blockers:");
        for (i, &id) in blockers.iter().enumerate() {
            println!("  {}: {}", i, display::format_permanent(game, id));
        }
        println!("Using default (lethal to each in order). Press Enter.");
        read_line();
        default_damage_assignment(game, blockers, power)
    }

    fn choose_trample_damage_assignment(
        &self,
        game: &GameState,
        _player_id: PlayerId,
        attacker_id: ObjectId,
        blockers: &[ObjectId],
        _defending_target: DamageTarget,
        power: u64,
        has_deathtouch: bool,
    ) -> (Vec<(ObjectId, u64)>, u64) {
        println!(
            "\n--- Trample Damage Assignment for {} (power {}) ---",
            display::card_name(game, attacker_id),
            power,
        );
        println!("Using default (lethal to blockers, rest tramples). Press Enter.");
        read_line();
        default_trample_assignment(game, blockers, power, has_deathtouch)
    }

    fn choose_generic_mana_allocation(
        &self,
        game: &GameState,
        player_id: PlayerId,
        mana_cost: &ManaCost,
    ) -> HashMap<ManaType, u64> {
        auto_allocate_generic(game, player_id, mana_cost).unwrap_or_default()
    }
}

