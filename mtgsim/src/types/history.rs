//! What a card asks about the past: "this turn", "last turn", "since your
//! last turn", "this game" (`triggers-architecture.md` §3.10).
//!
//! The answers are materialized, never scanned: `state::history` holds each
//! player's rows and totals, advanced record by record as the dispatcher reads
//! each window. This file is the vocabulary a `Condition` leaf uses to ask it.

use crate::types::card_types::CardType;
use crate::types::effects::PlayerSet;

/// One quantity of one player's turn. Each is counted on exactly one row, and
/// the name says whose: the caster's, the drawer's, the player whose life
/// total moved, the player dealt the damage, the player who controlled the
/// creature as it died, the player who declared the attackers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnFact {
    /// Spells this player cast (CR 601.2i). A copy is not cast (CR 707.10).
    SpellsCast,
    /// Of those, the spells that had this card type as they were cast: an
    /// artifact creature spell counts under both types.
    SpellsCastOfType(CardType),
    /// Cards this player drew (CR 121.1). A card put into a hand without the
    /// word "draw" is not drawn (CR 121.5).
    CardsDrawn,
    /// Life this player gained, in total.
    LifeGained,
    /// Life-gain events: CR 119.9 makes each source's gain its own.
    LifeGainEvents,
    /// Life this player lost, in total: to damage, to a payment, to an effect.
    LifeLost,
    /// Life-loss events, one per record.
    LifeLossEvents,
    /// Damage dealt to this player (CR 120.3a), in total. Damage dealt to a
    /// permanent they control is not dealt to them.
    DamageTaken,
    /// Creatures this player controlled that died (CR 700.4), counted off
    /// each creature's last known information. Morbid's "a creature died
    /// this turn" sums every player's row.
    ControlledCreaturesDied,
    /// Creatures this player declared as attackers (CR 508.1a). Raid's "if
    /// you attacked this turn" is at least one.
    AttackersDeclared,
}

impl TurnFact {
    /// How many counts a turn's row holds: the nine above, then one spell
    /// count per card type.
    pub const COUNT: usize = 9 + CardType::COUNT;

    /// Where this fact's count sits in a row.
    pub const fn slot(self) -> usize {
        match self {
            TurnFact::SpellsCast => 0,
            TurnFact::CardsDrawn => 1,
            TurnFact::LifeGained => 2,
            TurnFact::LifeGainEvents => 3,
            TurnFact::LifeLost => 4,
            TurnFact::LifeLossEvents => 5,
            TurnFact::DamageTaken => 6,
            TurnFact::ControlledCreaturesDied => 7,
            TurnFact::AttackersDeclared => 8,
            TurnFact::SpellsCastOfType(card_type) => 9 + card_type.slot(),
        }
    }
}

/// How a count must compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountIs {
    AtLeast(u64),
    AtMost(u64),
}

impl CountIs {
    pub fn met_by(self, count: u64) -> bool {
        match self {
            CountIs::AtLeast(n) => count >= n,
            CountIs::AtMost(n) => count <= n,
        }
    }
}

/// "[Whose] [fact] [is]" over a span of turns. `whose` is resolved against
/// the condition's "you" (CR 109.5), and the rows it names are summed: "an
/// opponent lost life this turn" is `Opponents` at least 1.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryCount {
    pub whose: PlayerSet,
    pub fact: TurnFact,
    pub is: CountIs,
}
