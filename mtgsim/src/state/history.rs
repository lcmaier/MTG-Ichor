//! Every player's turns, materialized (`triggers-architecture.md` §3.10).
//!
//! One `TurnSummary` per player per turn of the game, whoever's turn it was,
//! so "last turn" is an index, "this game" a sum and "since your last turn" a
//! range. The dispatcher is the one writer (`engine::triggers::history`);
//! nothing derives these from the event log.

use crate::types::card_types::CardType;
use crate::types::history::TurnFact;

/// One player's side of one turn: every "this turn" quantity a card or a rule
/// reads, one field each, each counted on one player's row.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TurnSummary {
    pub spells_cast: u32,
    /// Per card type, sorted by type, so two rows that counted the same
    /// spells are equal whatever order the types arrived in.
    pub spells_cast_of_type: Vec<(CardType, u32)>,
    pub cards_drawn: u32,
    pub life_gained: u64,
    pub life_gain_events: u32,
    pub life_lost: u64,
    pub life_loss_events: u32,
    pub damage_taken: u64,
    pub controlled_creatures_died: u32,
    pub attackers_declared: u32,
}

impl TurnSummary {
    /// The row's value for `fact`.
    pub fn count(&self, fact: TurnFact) -> u64 {
        match fact {
            TurnFact::SpellsCast => self.spells_cast as u64,
            TurnFact::SpellsCastOfType(card_type) => self
                .spells_cast_of_type
                .iter()
                .find(|(t, _)| *t == card_type)
                .map_or(0, |(_, n)| *n as u64),
            TurnFact::CardsDrawn => self.cards_drawn as u64,
            TurnFact::LifeGained => self.life_gained,
            TurnFact::LifeGainEvents => self.life_gain_events as u64,
            TurnFact::LifeLost => self.life_lost,
            TurnFact::LifeLossEvents => self.life_loss_events as u64,
            TurnFact::DamageTaken => self.damage_taken,
            TurnFact::ControlledCreaturesDied => self.controlled_creatures_died as u64,
            TurnFact::AttackersDeclared => self.attackers_declared as u64,
        }
    }

    pub(crate) fn count_spell_of_type(&mut self, card_type: CardType) {
        let key = |t: &CardType| *t as u8;
        match self.spells_cast_of_type.binary_search_by_key(&key(&card_type), |(t, _)| key(t)) {
            Ok(at) => self.spells_cast_of_type[at].1 += 1,
            Err(at) => self.spells_cast_of_type.insert(at, (card_type, 1)),
        }
    }
}

/// One player's whole game.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerHistory {
    /// Turn `t`'s row at index `t - 1`, for every turn of the game so far.
    pub turns: Vec<TurnSummary>,
    /// The turns this player began, ascending: "your last turn".
    pub own_turns: Vec<u32>,
}

impl PlayerHistory {
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
            self.turns.resize_with(index + 1, TurnSummary::default);
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
