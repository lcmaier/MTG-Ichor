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
use crate::engine::targeting::TargetRef;
use crate::events::event::EventSeq;
use crate::state::battlefield::AttackTarget;
use crate::state::layer_memo::GatedWalkInput;
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

impl RowFact {
    /// Which kind of row it is: what the layer memo keys a read of the log
    /// by, so a write of one kind leaves a frame that read another standing.
    pub fn kind(self) -> RowKind {
        match self {
            RowFact::Cast { .. } => RowKind::Cast,
            RowFact::Drew => RowKind::Drew,
            RowFact::GainedLife(_) | RowFact::LostLife(_) => RowKind::Life,
            RowFact::Entered { .. } => RowKind::Entered,
            RowFact::LeftZone { .. } => RowKind::LeftZone,
            RowFact::LeftTheGame { .. } => RowKind::LeftTheGame,
            RowFact::DealtDamage { .. } => RowKind::DealtDamage,
            RowFact::Attacked(_) => RowKind::Attacked,
            RowFact::Blocked(_) => RowKind::Blocked,
            RowFact::RemovedFromCombat { .. } => RowKind::RemovedFromCombat,
            RowFact::BecameTarget { .. } => RowKind::BecameTarget,
        }
    }
}

/// [`RowFact`]'s variants, without their fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    Cast,
    Drew,
    Life,
    Entered,
    LeftZone,
    LeftTheGame,
    DealtDamage,
    Attacked,
    Blocked,
    RemovedFromCombat,
    BecameTarget,
}

impl RowKind {
    pub const COUNT: usize = 11;
}

/// How much of the turn a question about its combats spans: the whole turn,
/// or its latest combat, which CR 500.8's additional combats make two
/// different questions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSpan {
    ThisTurn,
    ThisCombat,
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

    /// Turn `turn`'s rows, newest first.
    pub fn rows_of_turn(&self, turn: u32) -> impl Iterator<Item = &HistoryRow> {
        self.rows_newest_first().skip_while(move |row| row.turn > turn).take_while(move |row| row.turn == turn)
    }

    /// The rows of `kind` a question about turn `turn`, the turn in
    /// progress, reads: every combat's for "this turn", the latest combat's
    /// for "this combat". Notes what it read into `note`, the layer walk's
    /// when a walk asks, so no caller can leave a dependency out.
    fn rows_during(
        &self,
        turn: u32,
        span: CombatSpan,
        kind: RowKind,
        note: &dyn Fn(GatedWalkInput),
    ) -> impl Iterator<Item = &HistoryRow> {
        note(GatedWalkInput::TurnNumber);
        note(GatedWalkInput::Rows(kind));
        if span == CombatSpan::ThisCombat {
            note(GatedWalkInput::CombatCount);
        }
        let latest = self.combats_this_turn;
        self.rows_of_turn(turn).filter(move |row| {
            row.fact.kind() == kind && match span {
                CombatSpan::ThisTurn => true,
                CombatSpan::ThisCombat => row.combat == latest,
            }
        })
    }

    /// Was `object` declared as an attacker in `span` of turn `turn`? A
    /// creature put onto the battlefield attacking never was (CR 508.4).
    pub(crate) fn attacked(&self, object: ObjectRef, turn: u32, span: CombatSpan, note: &dyn Fn(GatedWalkInput)) -> bool {
        self.rows_during(turn, span, RowKind::Attacked, note).any(|row| row.object == Some(object))
    }

    /// Was `object` declared as a blocker in `span` of turn `turn` (CR
    /// 509.1)? One put onto the battlefield blocking never was (CR 509.4).
    pub(crate) fn blocked(&self, object: ObjectRef, turn: u32, span: CombatSpan, note: &dyn Fn(GatedWalkInput)) -> bool {
        self.rows_during(turn, span, RowKind::Blocked, note).any(|row| row.object == Some(object))
    }

    /// Did `object` become a blocked creature as blockers were declared in
    /// `span` of turn `turn` (CR 509.1h)? The blockers' rows name it.
    pub(crate) fn was_blocked(&self, object: ObjectRef, turn: u32, span: CombatSpan, note: &dyn Fn(GatedWalkInput)) -> bool {
        self.rows_during(turn, span, RowKind::Blocked, note).any(|row| row.fact == RowFact::Blocked(object))
    }

    /// What `object` was attacking as it was last removed from combat in the
    /// latest combat of turn `turn` (CR 508.5's second sentence): `None` when
    /// it was not removed in that combat, or was attacking nothing.
    pub(crate) fn attacking_when_removed(&self, object: ObjectRef, turn: u32, note: &dyn Fn(GatedWalkInput)) -> Option<AttackTarget> {
        let removed = self
            .rows_during(turn, CombatSpan::ThisCombat, RowKind::RemovedFromCombat, note)
            .find(|row| row.object == Some(object))?;
        match removed.fact {
            RowFact::RemovedFromCombat { attacking } => attacking,
            _ => None,
        }
    }

    /// Turn `turn`'s targetings, newest first: what became the target, by a
    /// spell or not, whose, and which record said so (CR 115.1).
    pub fn targetings_of_turn(&self, turn: u32) -> impl Iterator<Item = (TargetRef, bool, PlayerId, EventSeq)> + '_ {
        self.rows_of_turn(turn).filter_map(|row| match row.fact {
            RowFact::BecameTarget { by_spell, controller } => {
                let target = match row.object {
                    Some(object) => TargetRef::Object(object),
                    None => TargetRef::Player(row.player?),
                };
                Some((target, by_spell, controller, row.seq))
            }
            _ => None,
        })
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

    fn object_ref(id: u64, zone_change_epoch: u64) -> ObjectRef {
        ObjectRef { id: crate::types::ids::ObjectId::from_trace(id), zone_change_epoch }
    }

    fn row(log: &mut HistoryLog, object: ObjectRef, turn: u32, fact: RowFact) {
        let seq = EventSeq(log.len());
        let combat = log.combats_this_turn();
        log.push(HistoryRow { seq, turn, combat, player: Some(0), object: Some(object), frame: None, fact });
    }

    const UNNOTED: &dyn Fn(GatedWalkInput) = &|_| {};

    /// Two combats in one turn: "defending player" reads the latest combat's
    /// removal only, and "attacked this turn" each combat's declaration. The
    /// same object after a zone change is a new one (CR 400.7) and did
    /// neither.
    #[test]
    fn the_rows_tell_combats_and_existences_apart() {
        let mut log = HistoryLog::default();
        let (bears, after_a_flicker) = (object_ref(7, 1), object_ref(7, 2));
        let player_1 = AttackTarget::Player(1);
        log.combat_began();
        row(&mut log, bears, 3, RowFact::Attacked(player_1));
        row(&mut log, bears, 3, RowFact::RemovedFromCombat { attacking: Some(player_1) });
        assert_eq!(log.attacking_when_removed(bears, 3, UNNOTED), Some(player_1));

        log.combat_began();
        assert_eq!(log.attacking_when_removed(bears, 3, UNNOTED), None, "not removed in the second combat");
        assert!(log.attacked(bears, 3, CombatSpan::ThisTurn, UNNOTED), "it attacked in the first");
        assert!(!log.attacked(bears, 3, CombatSpan::ThisCombat, UNNOTED), "and not in the second");
        assert!(!log.attacked(after_a_flicker, 3, CombatSpan::ThisTurn, UNNOTED));
        assert!(!log.attacked(bears, 4, CombatSpan::ThisTurn, UNNOTED));
    }

    /// A block is the blocker's row, naming the attacker, which is how the
    /// attacker was blocked; and "this combat" is the latest combat's.
    #[test]
    fn a_block_row_answers_both_sides_in_its_combat() {
        let mut log = HistoryLog::default();
        let (wall, bears) = (object_ref(7, 1), object_ref(8, 1));
        log.combat_began();
        row(&mut log, wall, 3, RowFact::Blocked(bears));
        assert!(log.blocked(wall, 3, CombatSpan::ThisCombat, UNNOTED));
        assert!(log.was_blocked(bears, 3, CombatSpan::ThisCombat, UNNOTED));
        assert!(!log.blocked(bears, 3, CombatSpan::ThisTurn, UNNOTED), "the attacker did not block");
        assert!(!log.was_blocked(wall, 3, CombatSpan::ThisTurn, UNNOTED), "the blocker was not blocked");

        log.combat_began();
        assert!(!log.blocked(wall, 3, CombatSpan::ThisCombat, UNNOTED), "not in the second combat");
        assert!(log.blocked(wall, 3, CombatSpan::ThisTurn, UNNOTED));
    }

    /// Each lookup notes the rows it scans, and "this combat" the combat
    /// count, so a walk that asked is invalidated by exactly those writes.
    #[test]
    fn a_lookup_notes_what_it_scans() {
        let log = HistoryLog::default();
        let noted = std::cell::RefCell::new(Vec::new());
        let note = |input| noted.borrow_mut().push(input);
        log.attacked(object_ref(7, 1), 1, CombatSpan::ThisTurn, &note);
        assert_eq!(*noted.borrow(), [GatedWalkInput::TurnNumber, GatedWalkInput::Rows(RowKind::Attacked)]);
        noted.borrow_mut().clear();
        log.was_blocked(object_ref(7, 1), 1, CombatSpan::ThisCombat, &note);
        assert_eq!(
            *noted.borrow(),
            [GatedWalkInput::TurnNumber, GatedWalkInput::Rows(RowKind::Blocked), GatedWalkInput::CombatCount]
        );
    }

    /// A row from an earlier turn is kept: the log answers "this game" too.
    #[test]
    fn earlier_turns_stay_and_a_turn_reads_its_own() {
        let mut log = HistoryLog::default();
        row(&mut log, object_ref(7, 1), 1, RowFact::Attacked(AttackTarget::Player(1)));
        row(&mut log, object_ref(8, 1), 2, RowFact::Attacked(AttackTarget::Player(1)));
        row(&mut log, object_ref(9, 1), 3, RowFact::Attacked(AttackTarget::Player(1)));
        assert_eq!(log.len(), 3);
        let of_turn_2: Vec<_> = log.rows_of_turn(2).map(|row| row.object).collect();
        assert_eq!(of_turn_2, [Some(object_ref(8, 1))]);
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
