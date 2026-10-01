//! A board described at rest, and its text (`plans/setup-architecture.md`).
//!
//! A scenario is the state at the start of a step's priority round, with the
//! stack empty and nothing waiting to trigger. Plain state is written down;
//! what a spell or an effect created is played from the board (§2). Three
//! ways in build one [`Scenario`]: [`Scenario::parse`] from text,
//! [`Scenario::write`] from a game, and in SU-3 the dev GUI's editor. One way
//! out, [`Scenario::build`], turns it into a `Game` through the engine's own
//! construction doors, emitting nothing (§3).
//!
//! # The grammar
//!
//! One line states one thing, and `#` starts a comment. A line that names a
//! card puts the name after the head's colon (only the first colon is the
//! head's, so "Circle of Protection: Red" parses), then an optional tag in
//! square brackets, then `|` and its words, separated by commas. A word that
//! names another card takes the rest of the line, so it comes last. No
//! printed name holds `|`, `#` or `[`, and thousands hold a comma, which is
//! why a line holds one card and a comma never separates names.
//!
//! The parser reads this table and the writer spells it; each word's default
//! is what the file means when it says nothing.
//!
//! | Line | CR | Writes | Default |
//! |---|---|---|---|
//! | `players N` | 102.1 | the seats | 2 |
//! | `starting life L` | 103.4 | `starting_life` and each life total | 20 |
//! | `seed S` | — | the game's and the agents' streams (`Streams`) | 0 |
//! | `turn T`, `active A` | 500.1 | the natural rotation's turns, ending with A's turn T | 1, player 0 |
//! | `step S`, S as `format_phase` names it, lower case | 500.1, 117.3a | the position, the active player holding priority | precombat main |
//! | `player p: life N` | 119 | `life_total` | starting life |
//! | `player p: <kind> N` | 122.1 | a player's counters: poison, energy | none |
//! | `player p: lands played N` | 305.2 | `lands_played_this_turn` | 0 |
//! | `player p: left the game` | 104.5, 800.4a | `player_lost` | in the game |
//! | `player p: commander damage N from <card>` | 903.10a | `commander_damage_taken` | none |
//! | `player p this turn:` / `last turn:` / `this game:` with `<fact> N` | — | `PlayerHistory`'s rows, a word per `TurnFact` (`HistoryRow`) | zero; this game sums the other two |
//! | `hand p:`, `graveyard p:`, `exile:`, `command:` `<card>` | 402, 404, 406, 408 | `create_in_zone`, each card on top of its zone | empty |
//! | `library p:` / `library p shuffled:` `<card>`, top first | 401, 701.24 | `create_in_zone`, each card under the last; then a shuffle | empty |
//! | `battlefield: <card>` | 613.7d | `create_on_battlefield`, in file order | — |
//! | `counters: <card> \| <kind> N` | 613.7c | a permanent's counters stamped at this line | — |
//! | `this turn: <card> \| …` | 603.2h, 603.7h | `triggered`, `resolved N`, `took its once-each-turn action`, each after an optional `ability N` | none |
//!
//! The words after a card's bar:
//!
//! | Word | CR | Writes | Default |
//! |---|---|---|---|
//! | `xN` | — | N copies of the line | 1 |
//! | `owner p`, `controller p` | 108.3, 110.2b | the owner; the default controller | each the other; a zone's player |
//! | `commander` | 903.3 | `is_commander` | no |
//! | `tapped` | 110.5 | `tapped` | untapped |
//! | `arrived this turn`, `arrived turn N` | 302.6 | the turn CR 302.6's clock starts | before the first turn |
//! | `<kind> N` | 122.1, 306.5b | that kind's count, replacing the intrinsic one | the intrinsic counters |
//! | `damage N` | 120.6 | `damage_marked` | 0 |
//! | `dealt first-strike damage` | 510.4 | `dealt_first_strike_damage` | no |
//! | `blocked` | 509.1h | the attacker stays blocked with no blocker | — |
//! | `attached to <card>` | 301.5, 613.7e | `attach`, at this line | — |
//! | `attacking player p`, `attacking <card>` | 506, 508.1 | `attacking` | — |
//! | `blocking <card>` | 509.1a | `blocking`, and the attacker's `blocked_by` in file order | — |
//!
//! A reference is a name, with its tag where two objects it could mean share
//! the name: a permanent among the battlefield's, a commander among the
//! commanders. A count is `xN` after the bar, never `N Forest`, since a name
//! may begin with a digit.

mod board;
mod build;
mod refusal;
mod text;

pub use board::{Arrival, CardLine, CardRef, Head, HistoryRow, Located, PlayerFact, Scenario, Target, Word};
pub use refusal::{Refusal, RefusalKind};
