//! The game's log of what happened (`triggers-architecture.md` §12, TR-5h):
//! one row for each event a card can ask about later, kept for the whole game.
//!
//! A row records an event, never the board: its frame is the one object's
//! characteristics then. The rows sit in fixed chunks, chained newest first
//! behind `Arc`s, so a clone copies one pointer however long the game, and the
//! first write after a fork copies only the open chunk. Nothing truncates it:
//! "this game" reads all of it, for any card, printed or custom. A count a
//! scenario states is a row too, standing in for the events it does not name.

use std::ops::RangeInclusive;
use std::sync::Arc;

use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::targeting::TargetRef;
use crate::oracle::characteristics::{FrameMoment, ObjectInformation};
use crate::events::event::EventId;
use crate::state::battlefield::AttackTarget;
use crate::state::layer_memo::GatedWalkInput;
use crate::types::card_types::CardType;
use crate::types::counted_enum;
use crate::types::history::TurnFact;
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::{Lki, ObjectOrPlayer};
use crate::types::zones::{Zone, ZoneChangeAct};

/// The game's first turn. Turn 0 is the game before it, when the opening
/// hands are drawn (CR 103), and nothing then counts toward a turn: no count
/// reads it, and no record has a place in it.
pub const FIRST_TURN: u32 = 1;

/// How many rows a chunk holds. The log is a chain of chunks, each a block of
/// rows behind an `Arc`, and every copy of the game shares every chunk, so a
/// fork's first write copies only the newest, open one: this many rows at
/// most, under 2 KB. Smaller would make that copy cheaper and the chain
/// longer; larger, the reverse. A four-seat turn writes about ten rows
/// (2026-10-19).
pub const CHUNK_ROWS: usize = 16;

/// One fact about one subject that a reader of the past can ask about, as it
/// was then. An event with several subjects writes a row for each, a
/// declaration of three attackers three `Attacked` rows; a fact with a second
/// party names it in the fact, the creature a damage row was dealt to or the
/// attacker a `Blocked` row blocks.
#[derive(Debug, Clone)]
pub struct HistoryRow {
    /// The record it was written from, which orders the log: "the first",
    /// "the last", "for the first time each turn".
    pub seq: EventId,
    pub turn: u32,
    /// Which of the turn's combats had begun, from 1; 0 before the first.
    pub combat: u32,
    /// Whose event: the caster, the drawer, the player whose life moved, the
    /// player targeted, or who controlled the object as the event found it.
    pub player: Option<PlayerId>,
    /// The object it was about, as the existence it was (CR 400.7).
    pub object: Option<ObjectRef>,
    /// That object as it was then (CR 608.2h).
    pub frame: RowFrame,
    pub fact: RowFact,
}

/// What a row keeps of its object as it was then (CR 608.2h): one of the
/// two, so a row's frame and its facts cannot disagree.
#[derive(Debug, Clone)]
pub enum RowFrame {
    /// Nothing: a row about no object, or a count a scenario states.
    None,
    /// The object's characteristics as the event left it.
    AsTheEventLeftIt(Arc<EffectiveCharacteristics>),
    /// An object that had left by the event, a departure or damage from a
    /// source gone before it: the LKI store's last known information of it,
    /// its frame with what the object store no longer answers, whether it was
    /// a token, its counters, its owner (CR 113.7a). A move a replacement kept
    /// in its zone keeps its record's frame from just before it the same way.
    LastKnown(Lki),
}

impl RowFrame {
    /// The characteristics it keeps, either way.
    pub fn characteristics(&self) -> Option<&EffectiveCharacteristics> {
        self.frame().map(|frame| &**frame)
    }

    /// The same, shared, for a held-memory reading that counts each frame
    /// once.
    pub fn frame(&self) -> Option<&Arc<EffectiveCharacteristics>> {
        match self {
            RowFrame::None => None,
            RowFrame::AsTheEventLeftIt(frame) => Some(frame),
            RowFrame::LastKnown(lki) => Some(lki.frame()),
        }
    }
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
    /// CR 603.6a: entered the battlefield, from the zone it came from; a token
    /// comes from none. Any other move is a `LeftZone`. The event, for a count
    /// of the turn's entries; whether an object entered this turn is its own
    /// fact (`ObjectFact::Entered`).
    EnteredBattlefield { from: Option<Zone> },
    /// A move that is no draw, cast or entering the battlefield: died, discarded, milled,
    /// sacrificed, exiled, returned to a hand. `became` is the object the move
    /// made (CR 400.7), which "a card put into your graveyard this turn" reads.
    /// `from == to` is an act a replacement kept in its zone (CR 614.6).
    LeftZone { from: Zone, to: Zone, act: ZoneChangeAct, became: ObjectRef },
    /// CR 800.4a: an object leaving the game with its owner, which CR 603.6c
    /// counts as leaving the battlefield when it was there.
    LeftTheGame { from: Zone },
    /// CR 120.1: the row's object dealt it, `excess` of it past lethal
    /// damage (CR 120.10).
    DealtDamage { to: ObjectOrPlayer, amount: u64, excess: u64, combat: bool },
    /// CR 508.1: declared as an attacker.
    Attacked(AttackTarget),
    /// CR 509.1: declared as a blocker of this attacking creature, a row for
    /// each creature it blocks.
    Blocked(ObjectRef),
    /// CR 509.1h: the row's attacking creature became blocked by an effect,
    /// which no creature blocks. A row of the combat's blocks.
    BecameBlocked,
    /// CR 506.4, 511.3: what it was attacking as it left combat.
    RemovedFromCombat { attacking: Option<AttackTarget> },
    /// CR 115.1: by a spell rather than an ability, controlled by `controller`.
    BecameTarget { by_spell: bool, controller: PlayerId },
    /// `count` of `fact`, the row's player's on the row's turn, as a scenario
    /// states it without naming the events (`setup-architecture.md` §5.1). A
    /// row of the kind it stands in for: a count reads it, and a reader asking
    /// which object did something finds none on it.
    Stated { fact: TurnFact, count: u64 },
}

impl RowFact {
    /// Which kind of row it is: what the layer memo keys a read of the log
    /// by, so a write of one kind leaves a frame that read another standing.
    pub fn kind(self) -> RowKind {
        match self {
            RowFact::Cast { .. } => RowKind::Cast,
            RowFact::Drew => RowKind::Drew,
            RowFact::GainedLife(_) | RowFact::LostLife(_) => RowKind::Life,
            RowFact::EnteredBattlefield { .. } => RowKind::EnteredBattlefield,
            RowFact::LeftZone { .. } => RowKind::LeftZone,
            RowFact::LeftTheGame { .. } => RowKind::LeftTheGame,
            RowFact::DealtDamage { .. } => RowKind::DealtDamage,
            RowFact::Attacked(_) => RowKind::Attacked,
            RowFact::Blocked(_) | RowFact::BecameBlocked => RowKind::Blocked,
            RowFact::RemovedFromCombat { .. } => RowKind::RemovedFromCombat,
            RowFact::BecameTarget { .. } => RowKind::BecameTarget,
            RowFact::Stated { fact, .. } => kind_counted(fact),
        }
    }
}

impl HistoryRow {
    /// Whether the row's object, as the row keeps it, satisfies `matches`,
    /// asked with its id and that frame, or its last known information where
    /// the row keeps one; a card a scenario named is in no store, and is
    /// `ObjectId::UNASSIGNED`. A row with no frame answers no, and a
    /// scenario's unnamed count is loud in a debug build: a card that narrows
    /// it asks what the file did not say (`setup-architecture.md` §5.1).
    pub(crate) fn frame_matches(&self, matches: impl FnOnce(ObjectInformation<'_>) -> bool) -> bool {
        let id = self.object.map_or(ObjectId::UNASSIGNED, |object| object.id);
        match &self.frame {
            RowFrame::LastKnown(lki) => matches(ObjectInformation::LastKnown(lki)),
            RowFrame::AsTheEventLeftIt(frame) => {
                matches(ObjectInformation::Characteristics(id, frame, FrameMoment::AsTheEventLeftIt(self.object)))
            }
            RowFrame::None => {
                debug_assert!(
                    !matches!(self.fact, RowFact::Stated { .. }),
                    "a card narrows a count a scenario stated without naming its objects: {:?}",
                    self.fact
                );
                false
            }
        }
    }
}

counted_enum! {
    /// Which kind of row a [`RowFact`] is, without its fields: the key the layer
    /// memo notes a read of the log by (`GatedWalkInput::Rows`), so a frame
    /// that read the attack rows stands through a damage row.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RowKind {
        Cast,
        Drew,
        Life,
        EnteredBattlefield,
        LeftZone,
        LeftTheGame,
        DealtDamage,
        Attacked,
        Blocked,
        RemovedFromCombat,
        BecameTarget,
    }
}

/// What `row` adds to the count of `fact` for a player `counts_for` admits:
/// one for a cast or a draw, the life for a life row, the damage for a damage
/// row; nothing for a row the fact does not count or for another player's.
/// Whose count a row adds to is the fact's to say: a damage row adds to the
/// player dealt the damage, where the row's player is the source's controller.
pub fn added_to_count(fact: TurnFact, row: &HistoryRow, counts_for: &dyn Fn(PlayerId) -> bool) -> u64 {
    let has_type = |card_type: CardType| row.frame.characteristics().is_some_and(|chars| chars.types.contains(&card_type));
    let Some(player) = row.player else { return 0 };
    let (whose, amount) = match (fact, row.fact) {
        (TurnFact::SpellsCast, RowFact::Cast { .. }) | (TurnFact::CardsDrawn, RowFact::Drew) => (player, 1),
        (TurnFact::LifeGained, RowFact::GainedLife(n)) | (TurnFact::LifeLost, RowFact::LostLife(n)) => (player, n),
        (TurnFact::LifeGainEvents, RowFact::GainedLife(_)) | (TurnFact::LifeLossEvents, RowFact::LostLife(_)) => (player, 1),
        (TurnFact::DamageTaken, RowFact::DealtDamage { to: ObjectOrPlayer::Player(dealt_to), amount, .. }) => (dealt_to, amount),
        // CR 700.4's "dies", under whoever controlled the creature as it did.
        (TurnFact::ControlledCreaturesDied, RowFact::LeftZone { from: Zone::Battlefield, to: Zone::Graveyard, .. })
            if has_type(CardType::Creature) =>
        {
            (player, 1)
        }
        (TurnFact::AttackersDeclared, RowFact::Attacked(_)) => (player, 1),
        (asked, RowFact::Stated { fact, count }) if asked == fact => (player, count),
        _ => return 0,
    };
    // A count notes the one kind of row it scans (`kind_counted`), so a fact
    // counted off a second kind would leave a walk standing through its write.
    debug_assert_eq!(row.fact.kind(), kind_counted(fact), "{fact:?} counted off a {:?} row", row.fact);
    if counts_for(whose) { amount } else { 0 }
}

/// The kind of row `fact` counts, which a count notes it read.
pub fn kind_counted(fact: TurnFact) -> RowKind {
    match fact {
        TurnFact::SpellsCast => RowKind::Cast,
        TurnFact::CardsDrawn => RowKind::Drew,
        TurnFact::LifeGained | TurnFact::LifeGainEvents | TurnFact::LifeLost | TurnFact::LifeLossEvents => RowKind::Life,
        TurnFact::DamageTaken => RowKind::DealtDamage,
        TurnFact::ControlledCreaturesDied => RowKind::LeftZone,
        TurnFact::AttackersDeclared => RowKind::Attacked,
    }
}

/// One thing becoming the target of a spell or ability (CR 115.1), as the
/// log's row says it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Targeting {
    pub target: TargetRef,
    /// By a spell rather than an ability.
    pub by_spell: bool,
    /// Who controlled the spell or ability.
    pub controller: PlayerId,
    /// The record that said so.
    pub seq: EventId,
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
    /// Append `row`, the newest event: `GameHistory::push_row`'s, which keeps
    /// each player's tally with it.
    pub(super) fn push(&mut self, row: HistoryRow) {
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

    /// Give the newest `frames.len()` rows these frames, oldest first: the
    /// window that wrote them took its objects again once its rows were in.
    pub(crate) fn reframe_newest(&mut self, frames: Vec<RowFrame>) {
        let mut frames = frames.into_iter().rev();
        let mut chunk = self.newest.as_mut();
        while let Some(open) = chunk {
            let open = Arc::make_mut(open);
            for row in open.rows.iter_mut().rev() {
                let Some(frame) = frames.next() else { return };
                row.frame = frame;
            }
            chunk = open.older.as_mut();
        }
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
        self.rows_of_turns(turn..=turn)
    }

    /// The rows of `turns`, newest first: rows are written in turn order, so
    /// the walk stops at the first one before them.
    pub fn rows_of_turns(&self, turns: RangeInclusive<u32>) -> impl Iterator<Item = &HistoryRow> {
        let (first, last) = (*turns.start(), *turns.end());
        self.rows_newest_first().skip_while(move |row| row.turn > last).take_while(move |row| row.turn >= first)
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
        self.rows_during(turn, span, RowKind::Blocked, note).any(|row| row.object == Some(object) && matches!(row.fact, RowFact::Blocked(_)))
    }

    /// Did `object` become a blocked creature in `span` of turn `turn` (CR
    /// 509.1h)? The rows of the creatures declared blocking it name it, and
    /// an effect's block is its own row.
    pub(crate) fn was_blocked(&self, object: ObjectRef, turn: u32, span: CombatSpan, note: &dyn Fn(GatedWalkInput)) -> bool {
        self.rows_during(turn, span, RowKind::Blocked, note).any(|row| match row.fact {
            RowFact::Blocked(attacker) => attacker == object,
            RowFact::BecameBlocked => row.object == Some(object),
            _ => false,
        })
    }

    /// Did the card `object` get where it is by dying in `span` of turn
    /// `turn` (CR 700.4)? The row of its move names it as the object the move
    /// made, and keeps the creature it was as it left.
    pub(crate) fn died(&self, object: ObjectRef, turn: u32, span: CombatSpan, note: &dyn Fn(GatedWalkInput)) -> bool {
        self.rows_during(turn, span, RowKind::LeftZone, note).any(|row| {
            matches!(row.fact, RowFact::LeftZone { from: Zone::Battlefield, to: Zone::Graveyard, became, .. } if became == object)
                && row.frame.characteristics().is_some_and(|frame| frame.types.contains(&CardType::Creature))
        })
    }

    /// Was `object` put where it is from `from` by `act` in `span` of turn
    /// `turn`? `None` asks neither.
    pub(crate) fn put_there(
        &self,
        object: ObjectRef,
        from: Option<Zone>,
        act: Option<ZoneChangeAct>,
        turn: u32,
        span: CombatSpan,
        note: &dyn Fn(GatedWalkInput),
    ) -> bool {
        self.rows_during(turn, span, RowKind::LeftZone, note).any(|row| match row.fact {
            RowFact::LeftZone { from: moved_from, act: moved_by, became, .. } => {
                became == object && from.is_none_or(|from| from == moved_from) && act.is_none_or(|act| act == moved_by)
            }
            _ => false,
        })
    }

    /// Was `object` dealt damage in `span` of turn `turn` (CR 120.3)?
    pub(crate) fn was_dealt_damage(&self, object: ObjectRef, turn: u32, span: CombatSpan, note: &dyn Fn(GatedWalkInput)) -> bool {
        self.excess_of_each_damage_to(object, turn, span, note).next().is_some()
    }

    /// Was `object` dealt excess damage in `span` of turn `turn` (CR
    /// 120.10)? A row's excess is its share of the event's, so any row with
    /// some says the event had some.
    pub(crate) fn was_dealt_excess_damage(&self, object: ObjectRef, turn: u32, span: CombatSpan, note: &dyn Fn(GatedWalkInput)) -> bool {
        self.excess_of_each_damage_to(object, turn, span, note).any(|excess| excess > 0)
    }

    /// The excess of each damage row dealt to `object` in `span` of turn
    /// `turn`, 0 for damage within lethal.
    fn excess_of_each_damage_to(
        &self,
        object: ObjectRef,
        turn: u32,
        span: CombatSpan,
        note: &dyn Fn(GatedWalkInput),
    ) -> impl Iterator<Item = u64> {
        self.rows_during(turn, span, RowKind::DealtDamage, note).filter_map(move |row| match row.fact {
            RowFact::DealtDamage { to: ObjectOrPlayer::Object(to), excess, .. } if to == object => Some(excess),
            _ => None,
        })
    }

    /// The players `object` dealt damage to in `span` of turn `turn`, only
    /// combat damage when `combat_only` (CR 120.1, 510.2).
    pub(crate) fn players_dealt_damage_by(
        &self,
        object: ObjectRef,
        combat_only: bool,
        turn: u32,
        span: CombatSpan,
        note: &dyn Fn(GatedWalkInput),
    ) -> impl Iterator<Item = PlayerId> {
        self.rows_during(turn, span, RowKind::DealtDamage, note).filter(move |row| row.object == Some(object)).filter_map(move |row| {
            match row.fact {
                RowFact::DealtDamage { to: ObjectOrPlayer::Player(player), combat, .. } if combat || !combat_only => Some(player),
                _ => None,
            }
        })
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

    /// Turn `turn`'s targetings, newest first (CR 115.1).
    pub fn targetings_of_turn(&self, turn: u32) -> impl Iterator<Item = Targeting> + '_ {
        self.rows_of_turn(turn).filter_map(|row| match row.fact {
            RowFact::BecameTarget { by_spell, controller } => {
                let target = match row.object {
                    Some(object) => TargetRef::Object(object),
                    None => TargetRef::Player(row.player?),
                };
                Some(Targeting { target, by_spell, controller, seq: row.seq })
            }
            _ => None,
        })
    }

    /// `fact` summed over the rows of `turns` whose count is a player
    /// `counts_for` admits and which `of` admits, staying at `u64::MAX` past
    /// it. Turn 0's rows are none of them: the opening hands are drawn before
    /// any turn (CR 103). Notes what it read into `note`.
    pub(crate) fn count(
        &self,
        fact: TurnFact,
        turns: RangeInclusive<u32>,
        counts_for: &dyn Fn(PlayerId) -> bool,
        of: &mut dyn FnMut(&HistoryRow) -> bool,
        note: &dyn Fn(GatedWalkInput),
    ) -> u64 {
        note(GatedWalkInput::TurnNumber);
        note(GatedWalkInput::Rows(kind_counted(fact)));
        let mut total: u64 = 0;
        for row in self.rows_of_turns((*turns.start()).max(FIRST_TURN)..=*turns.end()) {
            let added = added_to_count(fact, row, counts_for);
            if added > 0 && of(row) {
                total = total.saturating_add(added);
            }
        }
        total
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
        let last_known: usize = self
            .rows_newest_first()
            .filter_map(|row| match &row.frame {
                RowFrame::LastKnown(lki) => Some(lki.bytes_beside_its_frame()),
                RowFrame::None | RowFrame::AsTheEventLeftIt(_) => None,
            })
            .sum();
        let rows: usize = std::iter::successors(self.newest.as_deref(), |chunk| chunk.older.as_deref())
            .map(|chunk| per_chunk + chunk.rows.capacity() * std::mem::size_of::<HistoryRow>())
            .sum();
        rows + last_known
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drew(seq: usize, turn: u32) -> HistoryRow {
        HistoryRow { seq: EventId(seq), turn, combat: 0, player: Some(0), object: None, frame: RowFrame::None, fact: RowFact::Drew }
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

    /// A window's rows can straddle a chunk: the reframe reaches back into
    /// the full chunk, and a fork taken before it keeps its own frames.
    #[test]
    fn a_reframe_reaches_across_a_chunk_and_leaves_a_fork_alone() {
        use crate::engine::layers::compute::compute_characteristics;
        use crate::state::game_state::GameState;
        use crate::test_support::{put_on_battlefield, vanilla_creature};

        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        let frame = compute_characteristics(&game, bears);
        let mut log = HistoryLog::default();
        for seq in 0..(CHUNK_ROWS + 2) {
            log.push(drew(seq, 1));
        }
        let fork = log.clone();
        let framed = || frame.clone().map_or(RowFrame::None, RowFrame::AsTheEventLeftIt);
        log.reframe_newest(vec![framed(), framed(), framed(), RowFrame::None]);
        let framed: Vec<bool> = log.rows_newest_first().take(5).map(|row| row.frame.characteristics().is_some()).collect();
        assert_eq!(framed, [false, true, true, true, false], "oldest first: the newest row took the last frame");
        assert!(fork.rows_newest_first().all(|row| row.frame.characteristics().is_none()), "the fork's chunks were copied, not written");
    }

    /// A row's frame answers for the existence the row names (CR 400.7),
    /// whatever has its id now: a creature that attacked, then left and came
    /// back, is a new object that did not attack, and a row about the one that
    /// did still reads "attacked this turn" of it.
    #[test]
    fn a_rows_frame_answers_for_the_existence_the_row_names() {
        use crate::engine::layers::compute::compute_characteristics;
        use crate::state::game_state::GameState;
        use crate::test_support::{put_on_battlefield, vanilla_creature};
        use crate::types::effects::ObjectFilter;
        use crate::types::history::ObjectFact;

        let mut game = GameState::new(2, 20);
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        let (Some(attacker), Some(frame)) = (game.object_ref(bears), compute_characteristics(&game, bears)) else {
            panic!("the bears are on the battlefield");
        };
        let turn = game.turn_number;
        let attacked_row = RowFact::Attacked(AttackTarget::Player(1));
        game.history.push_row(HistoryRow { seq: EventId(0), turn, combat: 0, player: Some(0), object: Some(attacker), frame: RowFrame::None, fact: attacked_row });
        let cast = HistoryRow {
            seq: EventId(1),
            turn,
            combat: 0,
            player: Some(0),
            object: Some(attacker),
            frame: RowFrame::AsTheEventLeftIt(frame),
            fact: RowFact::Cast { from: Zone::Hand },
        };
        if let Some(object) = game.objects.get_mut(&bears) {
            object.zone_change_epoch += 1;
        }

        let attacked_this_turn = ObjectFilter::ThisTurn(ObjectFact::Attacked);
        assert!(
            cast.frame_matches(|object| game.object_matches_filter_of_source(object, &attacked_this_turn, 0, bears)),
            "the row's object attacked this turn, though the object with its id now did not"
        );
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
        let seq = EventId(log.len());
        let combat = log.combats_this_turn();
        log.push(HistoryRow { seq, turn, combat, player: Some(0), object: Some(object), frame: RowFrame::None, fact });
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
