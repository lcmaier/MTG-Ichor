// CLI DecisionProvider — interactive terminal play via stdin/stdout.
//
// Implements all 8 DecisionProvider methods. Uses oracle/mana_helpers to
// show affordable spells and suggest land taps. Retries on bad input.

use std::io::{self, BufRead, Write};

use crate::state::game_state::GameState;
use crate::types::ids::PlayerId;
use crate::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use crate::ui::decision::DecisionProvider;

/// Interactive CLI decision provider for human play.
///
/// Reads from stdin and writes prompts to stdout. Implements the 4-primitive
/// `DecisionProvider` trait. The `ask_*` functions in `ui::ask` handle semantic
/// context; this provider handles the interactive I/O.
pub struct CliDecisionProvider;

impl CliDecisionProvider {
    pub fn new() -> Self {
        CliDecisionProvider
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
    fn pick_n(
        &self,
        _game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        let prompt = match &context.kind {
            ChoiceKind::PriorityAction => "Choose action:".to_string(),
            ChoiceKind::DeclareAttackers => "Choose attackers (indices):".to_string(),
            ChoiceKind::DeclareBlockers => "Choose blockers (indices):".to_string(),
            ChoiceKind::ChooseAlternativeCost => "Choose cost (0=normal, 1+=alternative):".to_string(),
            ChoiceKind::ChooseAdditionalCosts => "Choose additional costs (indices, or none):".to_string(),
            ChoiceKind::SelectRecipients { .. } => "Choose targets (indices):".to_string(),
            ChoiceKind::DiscardToHandSize => "Choose card to discard:".to_string(),
            ChoiceKind::LegendRule { legend_name } => {
                format!("Legend rule: choose which '{}' to keep:", legend_name)
            }
            _ => format!("Choose from options ({:?}):", context.kind),
        };

        println!("\n--- {} ---", prompt);
        for (i, opt) in options.iter().enumerate() {
            println!("  [{}] {:?}", i, opt);
        }

        if bounds.0 == bounds.1 {
            if bounds.0 == 1 {
                // Single selection
                loop {
                    match read_usize(&format!("Select exactly 1 (0..{}):", options.len() - 1), options.len()) {
                        Some(idx) => return vec![idx],
                        None => {
                            println!("A selection is required.");
                            continue;
                        }
                    }
                }
            }
            println!("(select exactly {})", bounds.0);
        } else {
            println!("(select {}-{}, comma-separated or 'none')", bounds.0, bounds.1);
        }

        loop {
            let indices = read_usize_list("Enter indices:", options.len());
            if indices.len() >= bounds.0 && indices.len() <= bounds.1 {
                return indices;
            }
            println!(
                "Invalid selection count: got {}, need {}-{}. Try again.",
                indices.len(),
                bounds.0,
                bounds.1,
            );
        }
    }

    fn pick_number(
        &self,
        _game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        min: u64,
        max: u64,
    ) -> u64 {
        let prompt = match &context.kind {
            ChoiceKind::ChooseXValue { .. } => "Choose value for X:".to_string(),
            _ => format!("Choose a number ({:?}):", context.kind),
        };

        // For very large ranges (like X value with u64::MAX), show "0 or more"
        let range_str = if max == u64::MAX {
            format!("{} or more", min)
        } else {
            format!("{}-{}", min, max)
        };

        println!("\n--- {} ({}) ---", prompt, range_str);

        loop {
            let input = read_line();
            match input.parse::<u64>() {
                Ok(n) if n >= min && n <= max => return n,
                Ok(n) => println!("Out of range: {}. Must be {}.", n, range_str),
                Err(_) => println!("Invalid number. Try again."),
            }
        }
    }

    fn allocate(
        &self,
        _game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        let prompt = match &context.kind {
            ChoiceKind::AssignCombatDamage { .. } => "Assign combat damage:".to_string(),
            ChoiceKind::AssignTrampleDamage { .. } => "Assign trample damage:".to_string(),
            ChoiceKind::GenericManaAllocation { .. } => "Allocate generic mana:".to_string(),
            _ => format!("Distribute {} ({:?}):", total, context.kind),
        };

        println!("\n--- {} (total: {}) ---", prompt, total);
        for (i, bucket) in buckets.iter().enumerate() {
            let min_label = if per_bucket_mins[i] > 0 {
                format!(" (min {})", per_bucket_mins[i])
            } else {
                String::new()
            };
            let max_label = per_bucket_maxs
                .and_then(|maxs| if maxs[i] < u64::MAX { Some(format!(" (max {})", maxs[i])) } else { None })
                .unwrap_or_default();
            println!("  [{}] {:?}{}{}", i, bucket, min_label, max_label);
        }

        loop {
            println!("Enter {} values, comma-separated:", buckets.len());
            let input = read_line();
            let values: Vec<u64> = input
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse::<u64>().ok())
                .collect();

            if values.len() != buckets.len() {
                println!("Need exactly {} values, got {}.", buckets.len(), values.len());
                continue;
            }
            let sum: u64 = values.iter().sum();
            if sum != total {
                println!("Sum is {} but must equal {}.", sum, total);
                continue;
            }
            let mut valid = true;
            for (i, &val) in values.iter().enumerate() {
                if val < per_bucket_mins[i] {
                    println!("Bucket {} needs at least {}, got {}.", i, per_bucket_mins[i], val);
                    valid = false;
                    break;
                }
                if let Some(maxs) = per_bucket_maxs {
                    if val > maxs[i] {
                        println!("Bucket {} allows at most {}, got {}.", i, maxs[i], val);
                        valid = false;
                        break;
                    }
                }
            }
            if valid {
                return values;
            }
        }
    }

    fn choose_ordering(
        &self,
        _game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize> {
        println!("\n--- Order items ({:?}) ---", context.kind);
        for (i, item) in items.iter().enumerate() {
            println!("  [{}] {:?}", i, item);
        }

        loop {
            println!(
                "Enter indices in desired order ({} values, comma-separated):",
                items.len()
            );
            let input = read_line();
            let order: Vec<usize> = input
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse::<usize>().ok())
                .collect();

            if order.len() != items.len() {
                println!("Need exactly {} indices, got {}.", items.len(), order.len());
                continue;
            }

            // Validate permutation
            let mut seen = vec![false; items.len()];
            let mut valid = true;
            for &idx in &order {
                if idx >= items.len() {
                    println!("Index {} out of range.", idx);
                    valid = false;
                    break;
                }
                if seen[idx] {
                    println!("Duplicate index {}.", idx);
                    valid = false;
                    break;
                }
                seen[idx] = true;
            }
            if valid {
                return order;
            }
        }
    }
}
