//! The typed value a scenario file parses into: what the text says, line by
//! line, before anything is checked against a registry or the rules.

use crate::state::game_state::{Phase, PhaseType};
use crate::types::effects::CounterType;
use crate::types::history::{HistorySpan, TurnFact};
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
    pub player_words: Vec<LineNumbered<PlayerWord>>,
    /// The lines that create a card or speak of one, in file order: the
    /// order the loader stamps them in (CR 613.7d).
    pub cards: Vec<LineNumbered<CardLine>>,
    /// The `then:` lines, in file order: played from the board once it is
    /// built, before anyone else is asked (§5.3).
    pub setup_actions: Vec<LineNumbered<SetupAction>>,
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
            player_words: Vec::new(),
            cards: Vec::new(),
            setup_actions: Vec::new(),
        }
    }
}

/// An item and the 1-based line it came from, so a refusal can name it.
#[derive(Debug, Clone, PartialEq)]
pub struct LineNumbered<T> {
    pub line: usize,
    pub value: T,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerWord {
    Life { player: PlayerId, life: i64 },
    Counter { player: PlayerId, kind: CounterType, count: u32 },
    LandsPlayed { player: PlayerId, count: u32 },
    LeftTheGame { player: PlayerId },
    CommanderDamage { player: PlayerId, damage: u32, from: NamedCard },
    History { player: PlayerId, span: HistorySpan, fact: TurnFact, count: u64 },
}

/// One card line: a card created in a zone, or more said about one.
#[derive(Debug, Clone, PartialEq)]
pub struct CardLine {
    pub kind: LineKind,
    pub card: NamedCard,
    pub copies: u32,
    pub words: Vec<CardWord>,
}

/// Where a line's card is, or what the line says about an earlier one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
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
pub struct NamedCard {
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
pub enum Attacked {
    Player(PlayerId),
    Permanent(NamedCard),
}

/// One word after a card's bar. The module doc's second table.
#[derive(Debug, Clone, PartialEq)]
pub enum CardWord {
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
    AttachedTo(NamedCard),
    Attacking(Attacked),
    Blocking(NamedCard),
    /// `this turn:` words, each naming the ability by its printed position
    /// (1 for the first), or none when the card has one that can.
    Triggered { ability: Option<usize> },
    Resolved { ability: Option<usize>, times: u32 },
    TookOnceEachTurnAction { ability: Option<usize> },
}

impl CardWord {
    /// Does this word name another card? Such a word takes the rest of its
    /// line, since a name may hold a comma, so it is written last.
    pub fn names_a_card(&self) -> bool {
        matches!(self, CardWord::AttachedTo(_) | CardWord::Blocking(_) | CardWord::Attacking(Attacked::Permanent(_)))
    }
}

/// A `then:` line (§5.3): one seat's action, played from the board.
#[derive(Debug, Clone, PartialEq)]
pub struct SetupAction {
    pub seat: PlayerId,
    pub verb: SetupVerb,
    /// The card cast from the seat's hand, or the permanent whose ability
    /// is activated.
    pub card: NamedCard,
    /// The `targeting` segments, in the line's order. Each CR 601.2c choice
    /// the spell or ability asks takes the next of them it offers.
    pub targets: Vec<Targeted>,
}

/// What a setup action does, with the answer only that verb has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupVerb {
    /// `casts`, from the seat's hand.
    Casts,
    /// `activates` (CR 602.2); `ability N` is the ability's place among the
    /// permanent's abilities as the layers give them, 1 for the first.
    Activates { ability: Option<usize> },
}

/// What a `targeting` segment names (CR 115.1): a player, or an object by
/// its card's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Targeted {
    Player(PlayerId),
    Card(NamedCard),
}
