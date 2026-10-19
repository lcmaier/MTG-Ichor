//! The game's log of what happened (`triggers-architecture.md` §12, TR-5h):
//! one row for each event a card can ask about later, kept for the whole game.
//!
//! A row records an event, never the board: its frame is the one object's
//! characteristics then. The rows sit in fixed chunks, chained newest first
//! behind `Arc`s, so a clone copies one pointer however long the game, and the
//! first write after a fork copies only the open chunk. Nothing truncates it:
//! "this game" reads all of it, for any card, printed or custom.

use std::sync::Arc;

use crate::engine::layers::types::EffectiveCharacteristics;
use crate::events::event::EventSeq;
use crate::state::battlefield::AttackTarget;
use crate::types::ids::{ObjectRef, PlayerId};
use crate::types::triggers::ObjectOrPlayer;
use crate::types::zones::{Zone, ZoneChangeAct};

/// How many rows a chunk holds, so how many a fork's first write copies at most.
pub const CHUNK_ROWS: usize = 16;

/// One event a reader of the past can ask about, as it was then.
#[derive(Debug, Clone)]
pub struct HistoryRow {
    /// The record it was written from, which orders the log: "the first",
    /// "the last", "for the first time each turn".
    pub seq: EventSeq,
    pub turn: u32,
    /// Which of the turn's combats had begun, from 1; 0 before the first.
    pub combat: u32,
    /// Whose event: the caster, the drawer, the player whose life moved, the
    /// player targeted, or who controlled the object as the event found it.
    pub player: Option<PlayerId>,
    /// The object it was about, as the existence it was (CR 400.7).
    pub object: Option<ObjectRef>,
    /// That object as it was then (CR 608.2h): its record's frame where the
    /// record carries one, else its frame as the event left it.
    pub frame: Option<Arc<EffectiveCharacteristics>>,
    pub fact: RowFact,
}

/// What happened. A fact a later card asks is a variant here and an arm in
/// the writer, never a new store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowFact {
    /// CR 601.2i, from the zone it was cast from.
    Cast { from: Zone },
    /// CR 121.1.
    Drew,
    /// CR 119.9: each source's gain is its own event.
    GainedLife(u64),
    LostLife(u64),
    /// CR 603.6a, from the zone it came from; a token comes from none.
    Entered { from: Option<Zone> },
    /// A move that is no draw, cast or entry: died, discarded, milled,
    /// sacrificed, exiled, returned to a hand. `became` is the object the move
    /// made (CR 400.7), which "a card put into your graveyard this turn" reads.
    /// `from == to` is an act a replacement kept in its zone (CR 614.6).
    LeftZone { from: Zone, to: Zone, act: ZoneChangeAct, became: ObjectRef },
    /// CR 800.4a: an object leaving the game with its owner, which CR 603.6c
    /// counts as leaving the battlefield when it was there.
    LeftTheGame { from: Zone },
    /// CR 120.1: the row's object dealt it.
    DealtDamage { to: ObjectOrPlayer, amount: u64, combat: bool },
    /// CR 508.1: declared as an attacker.
    Attacked(AttackTarget),
    /// CR 509.1: declared as a blocker of this attacking creature, a row for
    /// each creature it blocks.
    Blocked(ObjectRef),
    /// CR 506.4, 511.3: what it was attacking as it left combat.
    RemovedFromCombat { attacking: Option<AttackTarget> },
    /// CR 115.1: by a spell rather than an ability, controlled by `controller`.
    BecameTarget { by_spell: bool, controller: PlayerId },
}

/// The game's rows, newest first.
#[derive(Debug, Clone, Default)]
pub struct HistoryLog {
    newest: Option<Arc<Chunk>>,
    rows: usize,
    /// The combat phases begun this turn, which CR 500.8 can add to.
    combats_this_turn: u32,
}

#[derive(Debug)]
struct Chunk {
    rows: Vec<HistoryRow>,
    older: Option<Arc<Chunk>>,
}

impl Clone for Chunk {
    /// A fork's first write into a shared open chunk copies it with room for
    /// the rest of the chunk, so its later writes do not reallocate.
    fn clone(&self) -> Self {
        let mut rows = Vec::with_capacity(CHUNK_ROWS);
        rows.extend(self.rows.iter().cloned());
        Chunk { rows, older: self.older.clone() }
    }
}

impl Drop for Chunk {
    /// Down the chain in a loop, where the default drop would recurse once per
    /// chunk of a long game.
    fn drop(&mut self) {
        let mut older = self.older.take();
        while let Some(chunk) = older {
            match Arc::try_unwrap(chunk) {
                Ok(mut chunk) => older = chunk.older.take(),
                Err(_) => break,
            }
        }
    }
}

impl HistoryLog {
    /// Append `row`, the newest event.
    pub(crate) fn push(&mut self, row: HistoryRow) {
        match self.newest.as_mut() {
            Some(open) if open.rows.len() < CHUNK_ROWS => Arc::make_mut(open).rows.push(row),
            _ => {
                let mut rows = Vec::with_capacity(CHUNK_ROWS);
                rows.push(row);
                self.newest = Some(Arc::new(Chunk { rows, older: self.newest.take() }));
            }
        }
        self.rows += 1;
    }

    /// A combat phase of this turn has begun.
    pub(crate) fn combat_began(&mut self) {
        self.combats_this_turn += 1;
    }

    /// A turn has begun, with no combat yet.
    pub(crate) fn turn_began(&mut self) {
        self.combats_this_turn = 0;
    }

    /// The combats begun this turn: the latest one's number, or 0.
    pub fn combats_this_turn(&self) -> u32 {
        self.combats_this_turn
    }

    /// Every row, newest first.
    pub fn rows_newest_first(&self) -> impl Iterator<Item = &HistoryRow> {
        std::iter::successors(self.newest.as_deref(), |chunk| chunk.older.as_deref())
            .flat_map(|chunk| chunk.rows.iter().rev())
    }

    pub fn len(&self) -> usize {
        self.rows
    }

    pub fn is_empty(&self) -> bool {
        self.rows == 0
    }

    /// The bytes the chunks hold for their rows, the frames apart: what a
    /// held-memory reading adds the distinct frames to.
    pub fn row_bytes_held(&self) -> usize {
        let per_chunk = std::mem::size_of::<Chunk>() + 2 * std::mem::size_of::<usize>();
        std::iter::successors(self.newest.as_deref(), |chunk| chunk.older.as_deref())
            .map(|chunk| per_chunk + chunk.rows.capacity() * std::mem::size_of::<HistoryRow>())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drew(seq: usize, turn: u32) -> HistoryRow {
        HistoryRow { seq: EventSeq(seq), turn, combat: 0, player: Some(0), object: None, frame: None, fact: RowFact::Drew }
    }

    fn seqs(log: &HistoryLog) -> Vec<usize> {
        log.rows_newest_first().map(|row| row.seq.0).collect()
    }

    #[test]
    fn rows_read_newest_first_across_chunks() {
        let mut log = HistoryLog::default();
        for seq in 0..(2 * CHUNK_ROWS + 3) {
            log.push(drew(seq, 1));
        }
        assert_eq!(log.len(), 2 * CHUNK_ROWS + 3);
        assert_eq!(seqs(&log), (0..(2 * CHUNK_ROWS + 3)).rev().collect::<Vec<_>>());
    }

    #[test]
    fn a_fork_writes_its_own_rows_and_shares_the_rest() {
        let mut log = HistoryLog::default();
        for seq in 0..(CHUNK_ROWS + 2) {
            log.push(drew(seq, 1));
        }
        let mut fork = log.clone();
        fork.push(drew(100, 1));
        log.push(drew(200, 1));
        assert_eq!(fork.rows_newest_first().next().map(|row| row.seq.0), Some(100));
        assert_eq!(log.rows_newest_first().next().map(|row| row.seq.0), Some(200));
        assert_eq!(fork.len(), log.len());
        // The full chunk under both open ones is one allocation, shared.
        let full = |log: &HistoryLog| log.newest.as_ref().and_then(|open| open.older.as_ref()).map(Arc::as_ptr);
        assert_eq!(full(&fork), full(&log));
        assert!(full(&log).is_some());
    }

    #[test]
    fn a_long_chain_drops_without_recursing() {
        let mut log = HistoryLog::default();
        for seq in 0..(CHUNK_ROWS * 20_000) {
            log.push(drew(seq, 1));
        }
        drop(log);
    }

    #[test]
    fn a_turn_starts_with_no_combat() {
        let mut log = HistoryLog::default();
        log.combat_began();
        log.combat_began();
        assert_eq!(log.combats_this_turn(), 2);
        log.turn_began();
        assert_eq!(log.combats_this_turn(), 0);
    }
}
