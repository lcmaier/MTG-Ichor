//! The typed value a scenario file parses into: what the text says, line by
//! line, before anything is checked against a registry or the rules.

use crate::state::game_state::{Phase, PhaseType};
use crate::types::effects::CounterType;
use crate::types::history::TurnFact;
use crate::types::ids::PlayerId;

/// A board described at rest (`setup-architecture.md` §2): the start of a
/// priority round in `step` of turn `turn`, active player `active` to act.
#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    pub players: usize,
    pub starting_life: i64,
    pub seed: u64,
    pub turn: u32,
    pub active: PlayerId,
    pub step: Phase,
    /// What the `player p…:` lines say, in file order.
    pub player_facts: Vec<Located<PlayerFact>>,
    /// The lines that create a card or speak of one, in file order: the
    /// order the loader stamps them in (CR 613.7d).
    pub cards: Vec<Located<CardLine>>,
}

impl Default for Scenario {
    /// What a file that says nothing describes: two players at 20, player
    /// 0's first precombat main phase.
    fn default() -> Self {
        Scenario {
            players: 2,
            starting_life: 20,
            seed: 0,
            turn: 1,
            active: 0,
            step: Phase { phase_type: PhaseType::Precombat, step: None },
            player_facts: Vec::new(),
            cards: Vec::new(),
        }
    }
}

/// An item and the 1-based line it came from, so a refusal can name it.
#[derive(Debug, Clone, PartialEq)]
pub struct Located<T> {
    pub line: usize,
    pub item: T,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerFact {
    Life { player: PlayerId, life: i64 },
    Counter { player: PlayerId, kind: CounterType, count: u32 },
    LandsPlayed { player: PlayerId, count: u32 },
    LeftTheGame { player: PlayerId },
    CommanderDamage { player: PlayerId, damage: u32, from: CardRef },
    History { player: PlayerId, row: HistoryRow, fact: TurnFact, count: u64 },
}

/// The three rows of `PlayerHistory` a file writes. "Since your last turn"
/// is derived from them (`setup-architecture.md` §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryRow {
    ThisTurn,
    LastTurn,
    ThisGame,
}

/// One card line: a card created in a zone, or more said about one.
#[derive(Debug, Clone, PartialEq)]
pub struct CardLine {
    pub head: Head,
    pub card: CardRef,
    pub copies: u32,
    pub words: Vec<Word>,
}

/// Where a line's card is, or what the line says about an earlier one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Head {
    Hand(PlayerId),
    /// Top first; `shuffled` orders the whole library from the game's stream.
    Library { player: PlayerId, shuffled: bool },
    Graveyard(PlayerId),
    Exile,
    Command,
    Battlefield,
    /// `counters:` — counter kinds stamped at this line, on a permanent
    /// listed above (CR 613.7c).
    Counters,
    /// `this turn:` — CR 603.2h and 603.7h's counts for a permanent's ability.
    ThisTurn,
}

/// A card by name, and the tag that tells two with one name apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardRef {
    pub name: String,
    pub tag: Option<String>,
}

/// When a permanent arrived: `arrived this turn`, or `arrived turn N` for
/// an earlier turn, which a non-active player's clock can still be reading
/// (CR 302.6 measures from its controller's most recent turn).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrival {
    ThisTurn,
    Turn(u32),
}

/// What an attacker attacks (CR 508.1b): a player, or a planeswalker or
/// battle by name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Player(PlayerId),
    Permanent(CardRef),
}

/// One word after a card's bar. The module doc's second table.
#[derive(Debug, Clone, PartialEq)]
pub enum Word {
    Owner(PlayerId),
    Controller(PlayerId),
    Commander,
    Tapped,
    /// When the permanent arrived, which starts CR 302.6's clock.
    Arrived(Arrival),
    Counter(CounterType, u32),
    Damage(u32),
    DealtFirstStrikeDamage,
    Blocked,
    AttachedTo(CardRef),
    Attacking(Target),
    Blocking(CardRef),
    /// `this turn:` words, each naming the ability by its printed position
    /// (1 for the first), or none when the card has one that can.
    Triggered { ability: Option<usize> },
    Resolved { ability: Option<usize>, times: u32 },
    TookOnceEachTurnAction { ability: Option<usize> },
}
