// CLI DecisionProvider — interactive terminal play via stdin/stdout.
//
// Implements the four `DecisionProvider` methods. Uses oracle/mana_helpers to
// show affordable spells and suggest land taps. Retries on bad input.
//
// A prompt that reads one index also takes a command in place of it: `full`
// flips the full-control switch above this seat (`ui::full_control`), and at
// a priority prompt `yield turn`, `yield stack` or `yield next` passes and
// keeps passing (`ui::auto_yield`), `yield off` stops. Typed at a prompt, a
// command is in the input stream, which is what replaying the game needs of
// it.

use std::io::{self, BufRead, Write};

use crate::state::game_state::GameState;
use crate::types::ids::PlayerId;
use crate::ui::auto_yield::{Yield, Yields, pass_index};
use crate::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use crate::ui::decision::{DecisionProvider, SeatMode};
use crate::ui::display::{option_label, question};
use crate::ui::full_control::FullControlSwitch;

/// Interactive CLI decision provider for human play.
///
/// Reads from stdin and writes prompts to stdout. Implements the 4-primitive
/// `DecisionProvider` trait. The `ask_*` functions in `ui::ask` handle semantic
/// context; this provider handles the interactive I/O.
pub struct CliDecisionProvider {
    /// The switch above this seat, which `full` flips.
    full_control: FullControlSwitch,
    /// The seat's yield, which `yield` sets.
    yields: Yields,
}

/// What a line typed in place of an index did.
#[derive(Debug, PartialEq, Eq)]
enum Typed {
    /// Nothing: it is not a command, so read it as an index.
    Index,
    /// A command, carried out; ask again.
    Again,
    /// A command that answers the prompt with this index.
    Answer(usize),
}

impl CliDecisionProvider {
    pub fn new(full_control: FullControlSwitch, yields: Yields) -> Self {
        CliDecisionProvider { full_control, yields }
    }

    /// Carry out `input` if it is a command.
    fn command(&self, input: &str, game: &GameState, kind: &ChoiceKind, options: &[ChoiceOption]) -> Typed {
        let words: Vec<String> = input.split_whitespace().map(str::to_ascii_lowercase).collect();
        let words: Vec<&str> = words.iter().map(String::as_str).collect();
        let until = match words.as_slice() {
            ["full"] => {
                let on = !self.full_control.is_on();
                self.full_control.set(on);
                println!("{}", if on { "Full control on: asked at every priority point." } else { "Full control off." });
                return Typed::Again;
            }
            ["yield", "off"] => {
                self.yields.clear();
                println!("Yield off.");
                return Typed::Again;
            }
            ["yield", "turn"] => Yield::UntilEndOfTurn,
            ["yield", "stack"] => Yield::UntilStackChanges,
            ["yield", "next"] => Yield::UntilYourNextTurn,
            ["yield", ..] => {
                println!("yield turn (until end of turn), stack (until the stack changes), next (until your next turn), or off");
                return Typed::Again;
            }
            _ => return Typed::Index,
        };
        if !matches!(kind, ChoiceKind::PriorityAction) {
            println!("A yield is set where you have priority.");
            return Typed::Again;
        }
        if self.full_control.is_on() {
            println!("Full control supersedes yields; type 'full' to turn it off first.");
            return Typed::Again;
        }
        if !self.yields.set(game, until) {
            println!("Nothing is on the stack to wait on.");
            return Typed::Again;
        }
        println!("Passing {until:?}.");
        Typed::Answer(pass_index(options))
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

fn parse_index(input: &str, max: usize) -> Option<usize> {
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
        game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        println!("\n--- {} ---", question(game, &context.kind));
        for (i, opt) in options.iter().enumerate() {
            println!("  [{}] {}", i, option_label(game, opt));
        }

        if bounds.0 == bounds.1 {
            if bounds.0 == 1 {
                // Single selection
                loop {
                    println!("Select exactly 1 (0..{}), or 'full', or 'yield ...':", options.len() - 1);
                    let input = read_line();
                    match self.command(&input, game, &context.kind, options) {
                        Typed::Answer(idx) => return vec![idx],
                        Typed::Again => continue,
                        Typed::Index => {}
                    }
                    match parse_index(&input, options.len()) {
                        Some(idx) => return vec![idx],
                        None => println!("A selection is required."),
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
        game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        min: u64,
        max: u64,
    ) -> u64 {
        let prompt = question(game, &context.kind);

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
        game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        per_bucket_mins: &[u64],
        per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        println!("\n--- {} (total: {}) ---", question(game, &context.kind), total);
        for (i, bucket) in buckets.iter().enumerate() {
            let min_label = if per_bucket_mins[i] > 0 {
                format!(" (min {})", per_bucket_mins[i])
            } else {
                String::new()
            };
            let max_label = per_bucket_maxs
                .and_then(|maxs| if maxs[i] < u64::MAX { Some(format!(" (max {})", maxs[i])) } else { None })
                .unwrap_or_default();
            println!("  [{}] {}{}{}", i, option_label(game, bucket), min_label, max_label);
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
                if let Some(maxs) = per_bucket_maxs
                    && val > maxs[i] {
                    println!("Bucket {} allows at most {}, got {}.", i, maxs[i], val);
                    valid = false;
                    break;
                }
            }
            if valid {
                return values;
            }
        }
    }

    fn choose_ordering(
        &self,
        game: &GameState,
        _player: PlayerId,
        context: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize> {
        println!("\n--- {} ---", question(game, &context.kind));
        for (i, item) in items.iter().enumerate() {
            println!("  [{}] {}", i, option_label(game, item));
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

    fn seat_mode(&self, _player: PlayerId) -> SeatMode {
        SeatMode { person: true, ..SeatMode::default() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_support::setup_two_player_game;
    use crate::ui::decision::PriorityAction;

    fn cli() -> (CliDecisionProvider, FullControlSwitch, Yields) {
        let (switch, yields) = (FullControlSwitch::default(), Yields::default());
        (CliDecisionProvider::new(switch.clone(), yields.clone()), switch, yields)
    }

    /// `full` flips the switch above the seat either way, in any case, and an
    /// index is not a command.
    #[test]
    fn full_flips_the_switch_and_an_index_is_not_a_command() {
        let game = setup_two_player_game();
        let (cli, switch, _) = cli();
        let kind = ChoiceKind::PriorityAction;
        assert_eq!(cli.command("2", &game, &kind, &[]), Typed::Index);
        assert_eq!(cli.command("full", &game, &kind, &[]), Typed::Again);
        assert!(switch.is_on());
        assert_eq!(cli.command("FULL", &game, &kind, &[]), Typed::Again);
        assert!(!switch.is_on());
    }

    /// A yield is set where the seat has priority, and answers that prompt
    /// with `Pass`; anywhere else it asks again and sets nothing.
    #[test]
    fn a_yield_passes_the_priority_prompt_it_is_typed_at() {
        let game = setup_two_player_game();
        let (cli, _, yields) = cli();
        let options = [
            ChoiceOption::Action(PriorityAction::CastSpell(crate::types::ids::new_object_id())),
            ChoiceOption::Action(PriorityAction::Pass),
        ];
        assert_eq!(cli.command("yield turn", &game, &ChoiceKind::DeclareBlockers, &options), Typed::Again);
        assert!(!yields.holds(&game, 0));
        assert_eq!(cli.command("Yield Turn", &game, &ChoiceKind::PriorityAction, &options), Typed::Answer(1));
        assert!(yields.holds(&game, 0));
        assert_eq!(cli.command("yield off", &game, &ChoiceKind::PriorityAction, &options), Typed::Again);
        assert!(!yields.holds(&game, 0));
    }

    /// A stack yield on an empty stack, and any yield under full control,
    /// would be gone before it passed anything: both are refused, and the
    /// prompt is asked again.
    #[test]
    fn a_yield_with_nothing_to_wait_on_or_under_full_control_is_refused() {
        let game = setup_two_player_game();
        let (cli, switch, yields) = cli();
        let options = [ChoiceOption::Action(PriorityAction::Pass)];
        assert_eq!(cli.command("yield stack", &game, &ChoiceKind::PriorityAction, &options), Typed::Again);
        switch.set(true);
        assert_eq!(cli.command("yield turn", &game, &ChoiceKind::PriorityAction, &options), Typed::Again);
        assert!(!yields.holds(&game, 0));
    }
}
