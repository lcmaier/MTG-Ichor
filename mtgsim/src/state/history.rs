//! Every player's turns, materialized (`triggers-architecture.md` §3.10).
//!
//! One `TurnSummary` per player per turn of the game, whoever's turn it was,
//! so "last turn" is an index, "this game" a sum and "since your last turn" a
//! range. The dispatcher is the one writer (`engine::triggers::history`);
//! nothing derives these from the event log.

use crate::types::history::TurnFact;

/// One player's side of one turn: a count for each [`TurnFact`], in the
/// fact's slot. The fact is the key and the row holds nothing else, so a new
/// quantity is a `TurnFact` variant and an arm in the writer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnSummary {
    counts: [u64; TurnFact::COUNT],
}

impl TurnSummary {
    /// A turn with nothing counted on it.
    pub const ZERO: TurnSummary = TurnSummary { counts: [0; TurnFact::COUNT] };

    /// The row's count of `fact`.
    pub fn count(&self, fact: TurnFact) -> u64 {
        self.counts[fact.slot()]
    }

    /// Add `n` to `fact`'s count and return the new count.
    pub(crate) fn add(&mut self, fact: TurnFact, n: u64) -> u64 {
        let count = &mut self.counts[fact.slot()];
        *count += n;
        *count
    }
}

/// One player's whole game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerHistory {
    /// Turn `t`'s row at index `t - 1`, for every turn of the game so far.
    pub turns: Vec<TurnSummary>,
    /// The turns this player began, ascending: "your last turn".
    pub own_turns: Vec<u32>,
}

impl PlayerHistory {
    /// A player's history before the game's first turn: no rows, no turns.
    pub fn before_any_turn() -> Self {
        PlayerHistory { turns: Vec::new(), own_turns: Vec::new() }
    }

    /// Turn `turn`'s row, or `None` for a turn with nothing recorded, which
    /// reads as all zeros.
    pub fn turn(&self, turn: u32) -> Option<&TurnSummary> {
        turn.checked_sub(1).and_then(|i| self.turns.get(i as usize))
    }

    /// Turn `turn`'s row, created as it is first written. Turn 0 has none:
    /// nothing happens before the first turn but CR 103's setup.
    pub(crate) fn turn_mut(&mut self, turn: u32) -> Option<&mut TurnSummary> {
        let index = turn.checked_sub(1)? as usize;
        if self.turns.len() <= index {
            self.turns.resize(index + 1, TurnSummary::ZERO);
        }
        self.turns.get_mut(index)
    }

    /// The sum of `fact` over turns `first..=last`.
    pub fn sum(&self, fact: TurnFact, first: u32, last: u32) -> u64 {
        (first.max(1)..=last).filter_map(|t| self.turn(t)).map(|row| row.count(fact)).sum()
    }

    /// The most recent turn this player began before `turn`, if any.
    pub fn own_turn_before(&self, turn: u32) -> Option<u32> {
        self.own_turns.iter().rev().copied().find(|&t| t < turn)
    }

    pub(crate) fn record_own_turn(&mut self, turn: u32) {
        if self.own_turns.last() != Some(&turn) {
            self.own_turns.push(turn);
        }
    }
}
