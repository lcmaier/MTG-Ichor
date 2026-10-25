//! The histories' one writer (`triggers-architecture.md` §3.10, §4.1).
//!
//! Every record advances the histories once, in window order, **before the
//! gate** asks whether anything could trigger: a card that arrives later in
//! the turn reads what happened before it arrived (Vengeful Warchief's fourth
//! ruling, Paladin of Atonement's first). A record is counted as its event
//! ends: at its window's dispatch, or, for one card of a draw instruction, as
//! that card's draw ends, since the next card is decided after it (CR 121.2).

use std::sync::Arc;

use crate::engine::layers::compute::compute_characteristics;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::targeting::TargetRef;
use crate::events::event::{DepartedObject, EventId, GameEvent, LifeChange};
use crate::state::game_state::{AbilityIdentity, GameState, PhaseType, StepType};
use crate::state::history_log::{HistoryRow, RowFact, RowFrame, RowKind, FIRST_TURN};
use crate::types::history::TurnFact;
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::{Lki, ObjectOrPlayer};
use crate::types::zones::{Zone, ZoneChangeAct};
use crate::state::layer_memo::GatedWalkInput;

/// When a counted record happened, which each of its rows carries.
#[derive(Clone, Copy)]
struct RowStamp {
    seq: EventId,
    turn: u32,
    /// Which of the turn's combats had begun, 0 before the first.
    combat: u32,
}

/// A log row before the write step gives it its stamp.
struct RowDraft {
    player: Option<PlayerId>,
    object: Option<ObjectRef>,
    frame: RowFrame,
    fact: RowFact,
}

/// What one record changes beside its rows, or has a place among.
enum HistoryUpdate {
    AbilityResolved { identity: AbilityIdentity },
    SpellCast { caster: PlayerId },
    CardDrawn { player: PlayerId },
    DrawStepBegan { player: PlayerId },
    LifeGained { player: PlayerId },
    LifeLost { player: PlayerId },
    CombatBegan,
}

impl HistoryUpdate {
    /// The player and the count a record has a place among: "the first spell
    /// you cast each turn", "the second card you draw", "the first time you
    /// gain life each turn" (`triggers-architecture.md` §3.5). `None` for a
    /// kind no trigger reads by place.
    fn placed_among(&self) -> Option<(PlayerId, TurnFact)> {
        Some(match self {
            HistoryUpdate::SpellCast { caster } => (*caster, TurnFact::SpellsCast),
            HistoryUpdate::CardDrawn { player } => (*player, TurnFact::CardsDrawn),
            HistoryUpdate::LifeGained { player } => (*player, TurnFact::LifeGainEvents),
            HistoryUpdate::LifeLost { player } => (*player, TurnFact::LifeLossEvents),
            HistoryUpdate::AbilityResolved { .. }
            | HistoryUpdate::DrawStepBegan { .. }
            | HistoryUpdate::CombatBegan => return None,
        })
    }
}

impl GameState {
    /// Write the log's rows for `window`'s records the history has not
    /// counted yet, and on each record where it falls in its turn and, for a
    /// draw, in its player's draw step (`EventRecord::place_in_turn`,
    /// `place_in_draw_step`).
    pub(crate) fn advance_history(&mut self, window: &[EventId]) {
        let updates: Vec<(EventId, Option<HistoryUpdate>)> = window
            .iter()
            .filter(|&&seq| self.events.record(seq).is_some_and(|r| !r.counted()))
            .map(|&seq| (seq, self.history_update(seq)))
            .collect();
        // Each counted record's stamp, taken after its own count so a combat's
        // first rows carry its number.
        let mut stamps: Vec<RowStamp> = Vec::with_capacity(updates.len());
        // Records of this window already placed, whose rows are written after
        // the loop: a second gain in one event is the second (CR 119.9). How
        // many, by player and fact.
        let mut placed: Vec<((PlayerId, TurnFact), u64)> = Vec::new();
        for (seq, update) in updates {
            let (mut in_turn, mut in_draw_step) = (None, None);
            if let Some(update) = update {
                self.count(&update);
                in_turn = self.place_in_turn(&update, &placed);
                if let Some(key) = update.placed_among() {
                    match placed.iter_mut().find(|(earlier, _)| *earlier == key) {
                        Some((_, count)) => *count += 1,
                        None => placed.push((key, 1)),
                    }
                }
                in_draw_step = self.place_in_draw_step(&update);
            }
            stamps.push(RowStamp { seq, turn: self.turn_number, combat: self.history.log.combats_this_turn() });
            self.events.note_counted(seq, in_turn, in_draw_step);
        }
        // The rows read their objects as the window left them (CR 603.10).
        let mut drafts: Vec<(EventId, RowDraft)> = Vec::new();
        for stamp in &stamps {
            self.draft_history_rows(stamp.seq, window, &mut drafts);
        }
        #[cfg(debug_assertions)]
        self.assert_each_zone_change_has_its_row(&stamps, &drafts);
        let mut stamp_of = stamps.iter().peekable();
        let mut kinds_written = [false; RowKind::COUNT];
        for (seq, draft) in drafts {
            while stamp_of.next_if(|stamp| stamp.seq != seq).is_some() {}
            let Some(&&RowStamp { turn, combat, .. }) = stamp_of.peek() else { break };
            let row = HistoryRow {
                seq,
                turn,
                combat,
                player: draft.player,
                object: draft.object,
                frame: draft.frame,
                fact: draft.fact,
            };
            kinds_written[row.fact.kind() as usize] = true;
            self.history.push_row(row);
        }
        let epoch = self.layer_epoch();
        self.bump_rows_written(&kinds_written);
        // A frame drafted above that read a kind of row this window wrote
        // left out the window's own rows, which the window left behind too:
        // take the frames again now that the log holds them.
        if self.layer_epoch() != epoch {
            let mut redrafts: Vec<(EventId, RowDraft)> = Vec::new();
            for stamp in &stamps {
                self.draft_history_rows(stamp.seq, window, &mut redrafts);
            }
            self.history.log.reframe_newest(redrafts.into_iter().map(|(_, draft)| draft.frame).collect());
            self.bump_rows_written(&kinds_written);
        }
    }

    /// Each zone change these records hold writes its row: a move that is
    /// no entry, cast or draw its own `LeftZone`, and an entry its
    /// `EnteredBattlefield` off the entry's record in the same window. A
    /// cast's row is its `SpellCast`'s and a draw's its `CardDrawn`'s, each
    /// announced after its move, outside any batch for an opening hand
    /// (CR 103.5, 121.5, 601.2i).
    #[cfg(debug_assertions)]
    fn assert_each_zone_change_has_its_row(&self, stamps: &[RowStamp], drafts: &[(EventId, RowDraft)]) {
        for stamp in stamps {
            let Some(GameEvent::ZoneChange { departed, to, act, .. }) = self.events.record(stamp.seq).map(|r| &r.event) else { continue };
            let object_id = departed.object().id;
            let written = match (to, act) {
                (_, ZoneChangeAct::Cast | ZoneChangeAct::Drawn) => true,
                (Zone::Battlefield, _) => drafts.iter().any(|(_, draft)| {
                    matches!(draft.fact, RowFact::EnteredBattlefield { .. }) && draft.object.is_some_and(|object| object.id == object_id)
                }),
                _ => drafts.iter().any(|(seq, draft)| *seq == stamp.seq && matches!(draft.fact, RowFact::LeftZone { .. })),
            };
            debug_assert!(written, "zone change {:?} of {object_id} to the {to:?} wrote no history row", stamp.seq);
        }
    }

    fn bump_rows_written(&mut self, kinds_written: &[bool; RowKind::COUNT]) {
        for kind in RowKind::ALL.into_iter().filter(|kind| kinds_written[*kind as usize]) {
            self.bump_layer_epoch_if_read(GatedWalkInput::Rows(kind));
        }
    }

    /// Write what one record changes beside its rows: an ability's
    /// resolutions, the draw step's count, the turn's combats.
    fn count(&mut self, update: &HistoryUpdate) {
        match update {
            HistoryUpdate::AbilityResolved { identity } => {
                *self.history.resolutions_this_turn.entry(identity.ignoring_ability_grant()).or_insert(0) += 1;
            }
            HistoryUpdate::CardDrawn { player } if self.turn_number >= FIRST_TURN && self.in_own_draw_step(*player) => {
                if let Some(history) = self.history.players.get_mut(*player) {
                    history.drew_in_draw_step();
                }
            }
            // CR 504: the draw step's count starts over.
            HistoryUpdate::DrawStepBegan { player } => {
                if let Some(history) = self.history.players.get_mut(*player) {
                    history.draw_step_drawn(0);
                }
            }
            HistoryUpdate::CombatBegan => {
                self.history.log.combat_began();
                self.bump_layer_epoch_if_read(GatedWalkInput::CombatCount);
            }
            HistoryUpdate::SpellCast { .. }
            | HistoryUpdate::CardDrawn { .. }
            | HistoryUpdate::LifeGained { .. }
            | HistoryUpdate::LifeLost { .. } => {}
        }
    }

    /// The counted record's place among its player's records of its kind this
    /// turn, 1 for the first, for the kinds a trigger reads by place: the
    /// log's rows of the turn, this window's earlier records, then it.
    fn place_in_turn(&self, update: &HistoryUpdate, placed_this_window: &[((PlayerId, TurnFact), u64)]) -> Option<u64> {
        let (player, fact) = update.placed_among()?;
        if self.turn_number < FIRST_TURN {
            return None;
        }
        let logged = self.history.count_in_turn(player, fact, self.turn_number);
        let earlier = placed_this_window.iter().find(|(placed, _)| *placed == (player, fact)).map_or(0, |&(_, count)| count);
        Some(logged.saturating_add(earlier).saturating_add(1))
    }

    /// A counted draw's place in its player's own draw step (CR 504).
    fn place_in_draw_step(&self, update: &HistoryUpdate) -> Option<u64> {
        match update {
            HistoryUpdate::CardDrawn { player } if self.turn_number >= FIRST_TURN && self.in_own_draw_step(*player) => {
                Some(self.history.players.get(*player)?.this_draw_step())
            }
            _ => None,
        }
    }

    /// How many times `identity`'s ability has resolved this turn (CR 603.7h).
    pub(crate) fn resolutions_this_turn_of(&self, identity: AbilityIdentity) -> u32 {
        self.history.resolutions_this_turn.get(&identity.ignoring_ability_grant()).copied().unwrap_or(0)
    }

    /// The log's rows for record `seq` of `window`, each its object as the
    /// event left it, read before anything is written.
    fn draft_history_rows(&self, seq: EventId, window: &[EventId], drafts: &mut Vec<(EventId, RowDraft)>) {
        let Some(record) = self.events.record(seq) else { return };
        // A row about an object that had left by the event keeps its
        // record's last known information, the LKI store's, which the object
        // store no longer answers for; every other row keeps a frame.
        let mut draft_history_row = |player: Option<PlayerId>, object: Option<ObjectRef>, frame, fact| {
            drafts.push((seq, RowDraft { player, object, frame, fact }));
        };
        let departed = |lki: &Lki| (Some(lki.object()), RowFrame::LastKnown(lki.clone()));
        let as_left = |frame: Option<Arc<EffectiveCharacteristics>>| frame.map_or(RowFrame::None, RowFrame::Characteristics);
        // `object`'s characteristics as this window left it. Read now if it is
        // still the object the record named. If it moved again later in this
        // window it became a new object (CR 400.7), and the record of that
        // later move holds how it looked as it left.
        let frame_as_left = |object: ObjectRef| {
            if self.object_ref(object.id) == Some(object) {
                compute_characteristics(self, object.id)
            } else {
                window.iter().filter_map(|&later| self.events.record(later)).find_map(|later| {
                    later.event.lki().filter(|(lki, ..)| lki.object() == object).map(|(lki, ..)| Arc::clone(lki.frame()))
                })
            }
        };
        // The object `id` is now, and its characteristics as the event left it.
        let current = |id: ObjectId| self.object_ref(id).map(|object| (object, as_left(compute_characteristics(self, id))));
        let controller = |frame: &RowFrame| frame.characteristics().map(|chars| chars.controller);
        match &record.event {
            GameEvent::SpellCast { spell, caster } => {
                if let (Some((object, frame)), Some(from)) =
                    (current(spell.id), self.stack_entries.get(&spell.id).and_then(|entry| entry.cast_from))
                {
                    draft_history_row(Some(*caster), Some(object), frame, RowFact::Cast { from });
                }
            }
            GameEvent::CardDrawn { player_id, card } => {
                if let Some((object, frame)) = current(card.id) {
                    draft_history_row(Some(*player_id), Some(object), frame, RowFact::Drew);
                }
            }
            GameEvent::LifeChanged { player_id, .. } => match record.event.life_change() {
                Some(LifeChange::Gained(n)) => draft_history_row(Some(*player_id), None, RowFrame::None, RowFact::GainedLife(n)),
                Some(LifeChange::Lost(n)) => draft_history_row(Some(*player_id), None, RowFrame::None, RowFact::LostLife(n)),
                None => {}
            },
            GameEvent::PermanentEnteredBattlefield { object, controller } => {
                let object_id = &object.id;
                let from = self.entry_origin(*object_id, seq);
                if let Some((object, frame)) = current(*object_id) {
                    debug_assert!(
                        self.battlefield.get(object_id).is_none_or(|entry| entry.entered_battlefield_turn == self.turn_number),
                        "{object_id}'s entry row and its own entry turn disagree"
                    );
                    draft_history_row(Some(*controller), Some(object), frame, RowFact::EnteredBattlefield { from });
                }
            }
            // A draw, a cast and an entry are their own rows.
            GameEvent::ZoneChange { to: Zone::Battlefield, .. }
            | GameEvent::ZoneChange { act: ZoneChangeAct::Cast | ZoneChangeAct::Drawn, .. } => {}
            GameEvent::ZoneChange { departed: moved, from, to, act, new_epoch, .. } => {
                let became = ObjectRef { id: moved.object().id, zone_change_epoch: *new_epoch };
                let fact = RowFact::LeftZone { from: *from, to: *to, act: *act, became };
                match moved {
                    DepartedObject::WithLki(lki) => {
                        let (object, frame) = departed(lki);
                        draft_history_row(Some(lki.characteristics().controller), object, frame, fact);
                    }
                    DepartedObject::WithoutLki { owner, .. } => draft_history_row(Some(*owner), Some(became), as_left(frame_as_left(became)), fact),
                }
            }
            GameEvent::SameZoneMove { zone, act, lki, .. } => {
                let fact = RowFact::LeftZone { from: *zone, to: *zone, act: *act, became: lki.object() };
                draft_history_row(Some(lki.owner()), Some(lki.object()), RowFrame::LastKnown(lki.clone()), fact);
            }
            GameEvent::LeftTheGame { departed: DepartedObject::WithLki(lki), from } => {
                let (object, frame) = departed(lki);
                draft_history_row(Some(lki.owner()), object, frame, RowFact::LeftTheGame { from: *from });
            }
            GameEvent::LeftTheGame { departed: DepartedObject::WithoutLki { object, owner }, from } => {
                draft_history_row(Some(*owner), Some(*object), RowFrame::None, RowFact::LeftTheGame { from: *from });
            }
            // A source that left before dealing it deals it as it last existed
            // (CR 608.2h).
            GameEvent::DamageDealt { source, target, amount, excess, is_combat, source_lki } => {
                let (object, frame) = match source_lki {
                    Some(lki) => departed(lki),
                    None => match current(source.id) {
                        Some((object, frame)) => (Some(object), frame),
                        None => (None, RowFrame::None),
                    },
                };
                let to = match target {
                    ObjectOrPlayer::Object(object) => self.object_ref(object.id).map(ObjectOrPlayer::Object),
                    ObjectOrPlayer::Player(player) => Some(ObjectOrPlayer::Player(*player)),
                };
                if let Some(to) = to {
                    let fact = RowFact::DealtDamage { to, amount: *amount, excess: *excess, combat: *is_combat };
                    draft_history_row(controller(&frame), object, frame, fact);
                }
            }
            GameEvent::AttackersDeclared { attackers } => {
                for &(attacker, target) in attackers {
                    if let Some((object, frame)) = current(attacker) {
                        draft_history_row(Some(self.active_player), Some(object), frame, RowFact::Attacked(target));
                    }
                }
            }
            GameEvent::BlockersDeclared { blocks, .. } => {
                for &(blocker, attacker) in blocks {
                    if let (Some((object, frame)), Some(attacker)) = (current(blocker), self.object_ref(attacker)) {
                        draft_history_row(controller(&frame), Some(object), frame, RowFact::Blocked(attacker));
                    }
                }
            }
            GameEvent::BecameBlocked { attacker } => {
                if let Some((object, frame)) = current(*attacker) {
                    draft_history_row(controller(&frame), Some(object), frame, RowFact::BecameBlocked);
                }
            }
            GameEvent::RemovedFromCombat { removed } => {
                for exit in removed {
                    let frame = as_left(frame_as_left(exit.object));
                    draft_history_row(controller(&frame), Some(exit.object), frame, RowFact::RemovedFromCombat { attacking: exit.attacking });
                }
            }
            GameEvent::Targeted { target, ability_source, controller: targeting, .. } => {
                let fact = RowFact::BecameTarget { by_spell: ability_source.is_none(), controller: *targeting };
                match target {
                    TargetRef::Object(object) => {
                        let frame = as_left(frame_as_left(*object));
                        draft_history_row(controller(&frame), Some(*object), frame, fact);
                    }
                    TargetRef::Player(player) => draft_history_row(Some(*player), None, RowFrame::None, fact),
                }
            }
            _ => {}
        }
    }

    /// What `seq` changes beside its rows, or `None` for a record that
    /// changes nothing else and has no place.
    fn history_update(&self, seq: EventId) -> Option<HistoryUpdate> {
        let record = self.events.record(seq)?;
        Some(match &record.event {
            GameEvent::AbilityResolved { identity, .. } => HistoryUpdate::AbilityResolved { identity: *identity },
            GameEvent::SpellCast { caster, .. } => HistoryUpdate::SpellCast { caster: *caster },
            GameEvent::CardDrawn { player_id, .. } => HistoryUpdate::CardDrawn { player: *player_id },
            // CR 504: the draw step's count starts over.
            GameEvent::StepBegin { step: StepType::Draw, player } => HistoryUpdate::DrawStepBegan { player: *player },
            GameEvent::LifeChanged { player_id, .. } => match record.event.life_change() {
                Some(LifeChange::Gained(_)) => HistoryUpdate::LifeGained { player: *player_id },
                Some(LifeChange::Lost(_)) => HistoryUpdate::LifeLost { player: *player_id },
                None => return None,
            },
            GameEvent::PhaseBegin { phase: PhaseType::Combat, .. } => HistoryUpdate::CombatBegan,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::history::PLACED_FACTS;
    use crate::types::triggers::{Multiplicity, TriggerEvent};

    /// "The first time each turn" is one list, read three ways: the arms the
    /// registry admits it on (`TriggerEvent::placed_fact`), the records the
    /// writer places (`placed_among`) and the facts each player's tally keeps
    /// (`PLACED_FACTS`). An arm admitted whose records got no place would
    /// never trigger, and say nothing.
    #[test]
    fn the_placed_facts_agree() {
        let each = Multiplicity::PerOccurrence;
        let arms_and_their_records = [
            (TriggerEvent::CastsSpell { caster: None, spell: None }, HistoryUpdate::SpellCast { caster: 0 }),
            (TriggerEvent::DrawsCard { player: None, multiplicity: each, place: None }, HistoryUpdate::CardDrawn { player: 0 }),
            (TriggerEvent::GainsLife { player: None, multiplicity: each }, HistoryUpdate::LifeGained { player: 0 }),
            (TriggerEvent::LosesLife { player: None, multiplicity: each }, HistoryUpdate::LifeLost { player: 0 }),
        ];
        let mut counted = Vec::new();
        for (arm, record) in &arms_and_their_records {
            let fact = arm.placed_fact().unwrap_or_else(|| panic!("{arm:?} can say the first time"));
            assert_eq!(record.placed_among(), Some((0, fact)), "{arm:?}'s records are placed among {fact:?}");
            assert!(PLACED_FACTS.contains(&fact), "the tally keeps {fact:?}");
            counted.push(fact);
        }
        assert!(PLACED_FACTS.iter().all(|fact| counted.contains(fact)), "every fact the tally keeps is an arm's");
    }
}
