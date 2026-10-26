//! What a card asks about the past: "this turn", "last turn", "since your
//! last turn", "this game" (`triggers-architecture.md` §3.10, §12's TR-5h).
//!
//! The answers are the history log's rows (`state::history_log`), written
//! record by record as the dispatcher reads each window. This file is the
//! vocabulary a `Condition` leaf, an amount and a filter use to ask it.

use crate::types::effects::{ObjectFilter, PlayerSet};
use crate::types::zones::{Zone, ZoneChangeAct};

/// Which turns a count spans. A `Condition` leaf and an amount read any of
/// the four; a scenario states the first three, and derives "since your last
/// turn" from them (`setup-architecture.md` §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistorySpan {
    ThisTurn,
    LastTurn,
    SinceYourLastTurn,
    ThisGame,
}

counted_enum! {
    /// One quantity of one player's turn. Each is counted on exactly one row, and
    /// the name says whose: the caster's, the drawer's, the player whose life
    /// total moved, the player dealt the damage, the player who controlled the
    /// creature as it died, the player who declared the attackers.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TurnFact {
        /// Spells this player cast (CR 601.2i). A copy is not cast (CR 707.10).
        /// [`HistoryCount::of`] narrows them by what they were as they were cast.
        SpellsCast,
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
}

/// What an object did, which `ObjectFilter::ThisTurn` and `ThisCombat` ask:
/// "creatures that attacked this turn", "each creature that blocked or was
/// blocked this turn", "a creature that entered this turn". In the card's own
/// past tense. A fact a later card asks is a variant with its writer; the
/// ones counted and not built are listed in `codebase-state.md`, "Found by
/// TR-5c".
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectFact {
    /// Declared as an attacker (CR 508.1); a creature put onto the
    /// battlefield attacking never attacked (CR 508.4).
    Attacked,
    /// Declared as a blocker (CR 509.1); one put onto the battlefield
    /// blocking never blocked (CR 509.4).
    Blocked,
    /// Became a blocked creature as blockers were declared (CR 509.1h).
    WasBlocked,
    /// Entered the battlefield. A turn's, never a combat's. The permanent's
    /// own fact (`PermanentState::entered_battlefield_turn`, which its last
    /// known information keeps), never the log's entry rows: CR 614.12's
    /// look-ahead asks it of a permanent not yet on the battlefield, and the
    /// batch it enters in may ask it before the window that writes its row
    /// closes.
    Entered,
    /// Was dealt damage (CR 120.3).
    WasDealtDamage,
    /// Was dealt excess damage (CR 120.10): damage past lethal, the batch's
    /// sources together.
    WasDealtExcessDamage,
    /// Dealt damage to a player (CR 120.1), only combat damage when `combat`
    /// (CR 510.2). `Everyone` is any player, `You` the asker's "you".
    DealtDamageTo { player: PlayerSet, combat: bool },
    /// Died (CR 700.4): put into a graveyard from the battlefield as a
    /// creature, as it left. Asked of the card the move made (CR 400.7), "a
    /// creature card in your graveyard that died this turn".
    Died,
    /// Put into the zone it is in from `from` by `act`, asked of the object
    /// the move made (CR 400.7): "a card in a graveyard that was milled this
    /// turn" (`act: Milled`), "discarded", "put there from anywhere".
    PutHere { from: Option<Zone>, act: Option<ZoneChangeAct> },
    /// Cast (CR 601.2i): a permanent that was a spell cast this turn, by
    /// the facts it keeps of the spell (CR 400.7d). A turn's.
    WasCast,
    /// Came under its controller's control this turn, entering or by a
    /// change of control: its control's start, which Layer 2 keeps (CR
    /// 613.1b). A turn's.
    CameUnderControl,
}

/// How a count must compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountIs {
    AtLeast(u64),
    AtMost(u64),
    /// CR 509.3e's "a particular number".
    Exactly(u64),
}

impl CountIs {
    pub fn satisfied_by(self, count: u64) -> bool {
        match self {
            CountIs::AtLeast(n) => count >= n,
            CountIs::AtMost(n) => count <= n,
            CountIs::Exactly(n) => count == n,
        }
    }
}

/// "[Players'] [fact] [of what]": the rows of a span of turns that answer,
/// summed. `players` is resolved against the asker's "you" (CR 109.5): "an
/// opponent lost life this turn" is `Opponents`. `of` narrows the rows by what
/// their object was then, the frame each keeps: "noncreature spells you've
/// cast this turn" is `SpellsCast` of `Not(ByType(Creature))`. A `Condition`
/// leaf compares it with a [`CountIs`]; `AmountExpr::History` reads it as a
/// number.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryCount {
    pub players: PlayerSet,
    pub fact: TurnFact,
    pub of: Option<Box<ObjectFilter>>,
}
