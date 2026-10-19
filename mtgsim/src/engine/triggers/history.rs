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
use crate::events::event::{DamageTarget, EventSeq, GameEvent};
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::{AbilityIdentity, GameState, PhaseType, StepType};
use crate::state::history::{CombatFact, PlayerHistory, TargetingRow};
use crate::state::history_log::{HistoryRow, RowFact};
use crate::types::card_types::CardType;
use crate::types::history::TurnFact;
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::triggers::ObjectOrPlayer;
use crate::types::zones::{Zone, ZoneChangeAct};
use crate::state::layer_memo::GatedWalkInput;

/// A log row before the write step gives it its turn and combat.
struct RowDraft {
    player: Option<PlayerId>,
    object: Option<ObjectRef>,
    frame: Option<Arc<EffectiveCharacteristics>>,
    fact: RowFact,
}

/// What one record adds to whose row, read before anything is written so the
/// read can take the layer walk.
enum HistoryUpdate {
    TurnBegan { player: PlayerId, turn: u32 },
    AbilityResolved { identity: AbilityIdentity },
    SpellCast { caster: PlayerId, types: Vec<CardType> },
    CardDrawn { player: PlayerId },
    DrawStepBegan { player: PlayerId },
    LifeGained { player: PlayerId, amount: u64 },
    LifeLost { player: PlayerId, amount: u64 },
    DamageTaken { player: PlayerId, amount: u64 },
    CreatureDied { controller: PlayerId },
    /// The count is the attacking player's; the attacks are the combat's
    /// rows.
    AttackersDeclared { player: PlayerId, count: u64, attacks: Vec<(ObjectRef, AttackTarget)> },
    /// Each block, as (blocker, attacker): the combat's rows.
    BlockersDeclared { blocks: Vec<(ObjectRef, ObjectRef)> },
    /// Something became the target of a spell or ability: the turn's
    /// targeting rows.
    Targeted { row: TargetingRow },
    CombatBegan,
    RemovedFromCombat { exits: Vec<(ObjectRef, Option<AttackTarget>)> },
}

impl HistoryUpdate {
    /// The player and the count a record has a place among: "the first spell
    /// you cast each turn", "the second card you draw", "the first time you
    /// gain life each turn" (`triggers-architecture.md` §3.5). `None` for a
    /// kind no trigger reads by place.
    fn placed_among(&self) -> Option<(PlayerId, TurnFact)> {
        Some(match self {
            HistoryUpdate::SpellCast { caster, .. } => (*caster, TurnFact::SpellsCast),
            HistoryUpdate::CardDrawn { player } => (*player, TurnFact::CardsDrawn),
            HistoryUpdate::LifeGained { player, .. } => (*player, TurnFact::LifeGainEvents),
            HistoryUpdate::LifeLost { player, .. } => (*player, TurnFact::LifeLossEvents),
            HistoryUpdate::TurnBegan { .. }
            | HistoryUpdate::AbilityResolved { .. }
            | HistoryUpdate::DrawStepBegan { .. }
            | HistoryUpdate::DamageTaken { .. }
            | HistoryUpdate::CreatureDied { .. }
            | HistoryUpdate::AttackersDeclared { .. }
            | HistoryUpdate::BlockersDeclared { .. }
            | HistoryUpdate::Targeted { .. }
            | HistoryUpdate::CombatBegan
            | HistoryUpdate::RemovedFromCombat { .. } => return None,
        })
    }
}

impl GameState {
    /// Count `window`'s records the summaries have not counted yet, and write
    /// on each where it falls in its turn and, for a draw, in its player's
    /// draw step (`EventRecord::place_in_turn`, `place_in_draw_step`).
    pub(crate) fn advance_history(&mut self, window: &[EventSeq]) {
        let mut drafts: Vec<(EventSeq, RowDraft)> = Vec::new();
        let updates: Vec<(EventSeq, Option<HistoryUpdate>)> = window
            .iter()
            .filter(|&&seq| self.events.record(seq).is_some_and(|r| !r.counted()))
            .map(|&seq| {
                self.draft_history_rows(seq, window, &mut drafts);
                (seq, self.history_update(seq))
            })
            .collect();
        let mut drafts = drafts.into_iter().peekable();
        let mut wrote = false;
        for (seq, update) in updates {
            let (mut in_turn, mut in_draw_step) = (None, None);
            if let Some(update) = update {
                self.count(&update);
                in_turn = self.place_in_turn(&update);
                in_draw_step = self.place_in_draw_step(&update);
                wrote = true;
            }
            // After the record's own count, so a combat's first rows carry
            // its number.
            while let Some((_, draft)) = drafts.next_if(|(drafted, _)| *drafted == seq) {
                let row = HistoryRow {
                    seq,
                    turn: self.turn_number,
                    combat: self.history.log.combats_this_turn(),
                    player: draft.player,
                    object: draft.object,
                    frame: draft.frame,
                    fact: draft.fact,
                };
                self.history.log.push(row);
            }
            self.events.note_counted(seq, in_turn, in_draw_step);
        }
        if wrote {
            self.bump_layer_epoch_if_read(GatedWalkInput::Histories);
        }
    }

    /// Write one record's update: the game's history, a player's row, or
    /// both.
    fn count(&mut self, update: &HistoryUpdate) {
        let turn = self.turn_number;
        match update {
            HistoryUpdate::TurnBegan { player, turn: began } => self.history.count_turn_began(*began, *player),
            HistoryUpdate::AbilityResolved { identity } => {
                let key = (identity.source, identity.ability.definition());
                *self.history.resolutions_this_turn.entry(key).or_insert(0) += 1;
            }
            HistoryUpdate::SpellCast { caster, types } => {
                if let Some(row) = self.row_to_count_on(*caster) {
                    for &card_type in types {
                        row.add(turn, TurnFact::SpellsCastOfType(card_type), 1);
                    }
                    row.add(turn, TurnFact::SpellsCast, 1);
                }
            }
            HistoryUpdate::CardDrawn { player } => {
                let in_draw_step = self.in_own_draw_step(*player);
                if let Some(row) = self.row_to_count_on(*player) {
                    row.add(turn, TurnFact::CardsDrawn, 1);
                    if in_draw_step {
                        row.drew_in_draw_step();
                    }
                }
            }
            // CR 504: the draw step's count starts over.
            HistoryUpdate::DrawStepBegan { player } => {
                if let Some(row) = self.row_to_count_on(*player) {
                    row.draw_step_drawn(0);
                }
            }
            HistoryUpdate::LifeGained { player, amount } => {
                if let Some(row) = self.row_to_count_on(*player) {
                    row.add(turn, TurnFact::LifeGained, *amount);
                    row.add(turn, TurnFact::LifeGainEvents, 1);
                }
            }
            HistoryUpdate::LifeLost { player, amount } => {
                if let Some(row) = self.row_to_count_on(*player) {
                    row.add(turn, TurnFact::LifeLost, *amount);
                    row.add(turn, TurnFact::LifeLossEvents, 1);
                }
            }
            HistoryUpdate::DamageTaken { player, amount } => {
                if let Some(row) = self.row_to_count_on(*player) {
                    row.add(turn, TurnFact::DamageTaken, *amount);
                }
            }
            HistoryUpdate::CreatureDied { controller } => {
                if let Some(row) = self.row_to_count_on(*controller) {
                    row.add(turn, TurnFact::ControlledCreaturesDied, 1);
                }
            }
            HistoryUpdate::AttackersDeclared { player, count, attacks } => {
                for &(attacker, target) in attacks {
                    self.history.combat.record(attacker, turn, CombatFact::Attacked(target));
                }
                if let Some(row) = self.row_to_count_on(*player) {
                    row.add(turn, TurnFact::AttackersDeclared, *count);
                }
            }
            HistoryUpdate::BlockersDeclared { blocks } => {
                for &(blocker, attacker) in blocks {
                    self.history.combat.record(blocker, turn, CombatFact::Blocked(attacker));
                }
            }
            HistoryUpdate::Targeted { row } => self.history.targetings.record(*row),
            HistoryUpdate::CombatBegan => {
                self.history.combat.combat_began();
                self.history.log.combat_began();
            }
            HistoryUpdate::RemovedFromCombat { exits } => {
                for &(object, attacking) in exits {
                    self.history.combat.record(object, turn, CombatFact::Removed { attacking });
                }
            }
        }
    }

    /// `player`'s row of the turn in progress, or `None` before the first
    /// turn: the opening hands are drawn before any turn, on no row (CR 103).
    fn row_to_count_on(&mut self, player: PlayerId) -> Option<&mut PlayerHistory> {
        if self.turn_number == 0 {
            return None;
        }
        self.history.players.get_mut(player)
    }

    /// The counted record's place among its player's records of its kind this
    /// turn, 1 for the first, for the kinds a trigger reads by place.
    fn place_in_turn(&self, update: &HistoryUpdate) -> Option<u64> {
        let (player, fact) = update.placed_among()?;
        if self.turn_number == 0 {
            return None;
        }
        Some(self.history.players.get(player)?.this_turn(self.turn_number).count(fact))
    }

    /// A counted draw's place in its player's own draw step (CR 504).
    fn place_in_draw_step(&self, update: &HistoryUpdate) -> Option<u64> {
        match update {
            HistoryUpdate::CardDrawn { player } if self.turn_number != 0 && self.in_own_draw_step(*player) => {
                Some(self.history.players.get(*player)?.this_draw_step())
            }
            _ => None,
        }
    }

    /// How many times `identity`'s ability has resolved this turn (CR 603.7h).
    pub(crate) fn resolutions_this_turn_of(&self, identity: AbilityIdentity) -> u32 {
        self.history.resolutions_this_turn.get(&(identity.source, identity.ability.definition())).copied().unwrap_or(0)
    }

    /// The log's rows for record `seq` of `window`, each its object as the
    /// event left it, read before anything is written.
    fn draft_history_rows(&self, seq: EventSeq, window: &[EventSeq], drafts: &mut Vec<(EventSeq, RowDraft)>) {
        let Some(record) = self.events.record(seq) else { return };
        let mut draft = |player: Option<PlayerId>, object: Option<ObjectRef>, frame, fact| {
            drafts.push((seq, RowDraft { player, object, frame, fact }));
        };
        // An object still the existence the record named, read now. One that
        // has moved again in this window is a new object (CR 400.7); the move
        // that ended it carries it as it was, where its record keeps a frame.
        let still = |object: ObjectRef| {
            if self.object_ref(object.id) == Some(object) {
                compute_characteristics(self, object.id)
            } else {
                window.iter().filter_map(|&later| self.events.record(later)).find_map(|later| {
                    later.event.lki().filter(|(lki, ..)| lki.object() == object).map(|(lki, ..)| Arc::clone(lki.frame()))
                })
            }
        };
        let now = |id: ObjectId| self.object_ref(id).map(|object| (object, compute_characteristics(self, id)));
        let controller = |frame: &Option<Arc<EffectiveCharacteristics>>| frame.as_ref().map(|chars| chars.controller);
        match &record.event {
            GameEvent::SpellCast { spell_id, caster } => {
                if let (Some((object, frame)), Some(from)) =
                    (now(*spell_id), self.stack_entries.get(spell_id).and_then(|entry| entry.cast_from))
                {
                    draft(Some(*caster), Some(object), frame, RowFact::Cast { from });
                }
            }
            // No frame: no card reads what an earlier drawn card was, and a
            // filter over one is refused until one does. Nothing else asks
            // about a card in hand, so each frame would be a layer walk.
            GameEvent::CardDrawn { player_id, card_id } => {
                draft(Some(*player_id), self.object_ref(*card_id), None, RowFact::Drew);
            }
            GameEvent::LifeChanged { player_id, old, new, .. } if new > old => {
                draft(Some(*player_id), None, None, RowFact::GainedLife((new - old) as u64));
            }
            GameEvent::LifeChanged { player_id, old, new, .. } if new < old => {
                draft(Some(*player_id), None, None, RowFact::LostLife((old - new) as u64));
            }
            // Where it came from is its zone change's, in the same window; a
            // token comes from none (CR 111.2).
            GameEvent::PermanentEnteredBattlefield { object_id, controller } => {
                let from = window.iter().filter_map(|&other| self.events.record(other)).find_map(|other| match &other.event {
                    GameEvent::ZoneChange { object_id: moved, from, to: Zone::Battlefield, .. } if moved == object_id => Some(*from),
                    _ => None,
                });
                if let Some((object, frame)) = now(*object_id) {
                    draft(Some(*controller), Some(object), frame, RowFact::Entered { from });
                }
            }
            // A draw, a cast and an entry are their own rows.
            GameEvent::ZoneChange { to: Zone::Battlefield, .. }
            | GameEvent::ZoneChange { act: ZoneChangeAct::Cast | ZoneChangeAct::Drawn, .. } => {}
            GameEvent::ZoneChange { object_id, owner, from, to, act, lki, new_epoch, .. } => {
                let became = ObjectRef { id: *object_id, zone_change_epoch: *new_epoch };
                let fact = RowFact::LeftZone { from: *from, to: *to, act: *act, became };
                match lki {
                    Some(lki) => draft(Some(lki.chars().controller), Some(lki.object()), Some(Arc::clone(lki.frame())), fact),
                    None => draft(Some(*owner), Some(became), still(became), fact),
                }
            }
            GameEvent::SameZoneMove { owner, zone, act, lki, .. } => {
                let fact = RowFact::LeftZone { from: *zone, to: *zone, act: *act, became: lki.object() };
                draft(Some(*owner), Some(lki.object()), Some(Arc::clone(lki.frame())), fact);
            }
            GameEvent::LeftTheGame { owner, from, lki: Some(lki), .. } => {
                draft(Some(*owner), Some(lki.object()), Some(Arc::clone(lki.frame())), RowFact::LeftTheGame { from: *from });
            }
            GameEvent::LeftTheGame { object_id, owner, from, lki: None } => {
                draft(Some(*owner), self.object_ref(*object_id), None, RowFact::LeftTheGame { from: *from });
            }
            // A source that left before dealing it deals it as it last existed
            // (CR 608.2h).
            GameEvent::DamageDealt { source_id, target, amount, is_combat, source_lki } => {
                let (object, frame) = match source_lki {
                    Some(lki) => (Some(lki.object()), Some(Arc::clone(lki.frame()))),
                    None => match now(*source_id) {
                        Some((object, frame)) => (Some(object), frame),
                        None => (None, None),
                    },
                };
                let to = match target {
                    DamageTarget::Object(id) => self.object_ref(*id).map(ObjectOrPlayer::Object),
                    DamageTarget::Player(player) => Some(ObjectOrPlayer::Player(*player)),
                };
                if let Some(to) = to {
                    let fact = RowFact::DealtDamage { to, amount: *amount, combat: *is_combat };
                    draft(controller(&frame), object, frame, fact);
                }
            }
            GameEvent::AttackersDeclared { attackers } => {
                for &(attacker, target) in attackers {
                    if let Some((object, frame)) = now(attacker) {
                        draft(Some(self.active_player), Some(object), frame, RowFact::Attacked(target));
                    }
                }
            }
            GameEvent::BlockersDeclared { blocks, .. } => {
                for &(blocker, attacker) in blocks {
                    if let (Some((object, frame)), Some(attacker)) = (now(blocker), self.object_ref(attacker)) {
                        draft(controller(&frame), Some(object), frame, RowFact::Blocked(attacker));
                    }
                }
            }
            GameEvent::RemovedFromCombat { removed } => {
                for exit in removed {
                    let frame = still(exit.object);
                    draft(controller(&frame), Some(exit.object), frame, RowFact::RemovedFromCombat { attacking: exit.attacking });
                }
            }
            GameEvent::Targeted { target, ability_source, controller: targeting, .. } => {
                let fact = RowFact::BecameTarget { by_spell: ability_source.is_none(), controller: *targeting };
                match target {
                    TargetRef::Object(object) => {
                        let frame = still(*object);
                        draft(controller(&frame), Some(*object), frame, fact);
                    }
                    TargetRef::Player(player) => draft(Some(*player), None, None, fact),
                }
            }
            _ => {}
        }
    }

    /// What `seq` adds, or `None` for a record no summary counts.
    fn history_update(&self, seq: EventSeq) -> Option<HistoryUpdate> {
        let record = self.events.record(seq)?;
        Some(match &record.event {
            GameEvent::TurnBegin { player, turn_number } => HistoryUpdate::TurnBegan { player: *player, turn: *turn_number },
            GameEvent::AbilityResolved { identity, .. } => HistoryUpdate::AbilityResolved { identity: *identity },
            // CR 601.2i: the spell is cast, and on the stack, as this record
            // is dispatched, so its types are the ones it was cast with.
            GameEvent::SpellCast { spell_id, caster } => {
                let types: Vec<CardType> = compute_characteristics(self, *spell_id)
                    .map(|chars| chars.types.iter().copied().collect())
                    .unwrap_or_default();
                HistoryUpdate::SpellCast { caster: *caster, types }
            }
            GameEvent::CardDrawn { player_id, .. } => HistoryUpdate::CardDrawn { player: *player_id },
            // CR 504: the draw step's count starts over.
            GameEvent::StepBegin { step: StepType::Draw, player } => HistoryUpdate::DrawStepBegan { player: *player },
            GameEvent::LifeChanged { player_id, old, new, .. } if new > old => {
                HistoryUpdate::LifeGained { player: *player_id, amount: (new - old) as u64 }
            }
            GameEvent::LifeChanged { player_id, old, new, .. } if new < old => {
                HistoryUpdate::LifeLost { player: *player_id, amount: (old - new) as u64 }
            }
            GameEvent::DamageDealt { target: DamageTarget::Player(pid), amount, .. } => {
                HistoryUpdate::DamageTaken { player: *pid, amount: *amount }
            }
            // CR 700.4's "dies", under whoever controlled the creature as it did.
            GameEvent::ZoneChange { from: Zone::Battlefield, to: Zone::Graveyard, lki: Some(lki), .. }
                if lki.chars().types.contains(&CardType::Creature) =>
            {
                HistoryUpdate::CreatureDied { controller: lki.chars().controller }
            }
            // CR 508.1: the active player declares attackers, each as the
            // existence it is now, since the record's dispatch is the
            // declaration's own instant.
            GameEvent::AttackersDeclared { attackers } if !attackers.is_empty() => {
                let attacks =
                    attackers.iter().filter_map(|&(attacker, target)| Some((self.object_ref(attacker)?, target))).collect();
                HistoryUpdate::AttackersDeclared { player: self.active_player, count: attackers.len() as u64, attacks }
            }
            // CR 509.1: each block, its two creatures as the existences they
            // are now, at the declaration's own instant.
            GameEvent::BlockersDeclared { blocks, .. } if !blocks.is_empty() => {
                let blocks = blocks
                    .iter()
                    .filter_map(|&(blocker, attacker)| Some((self.object_ref(blocker)?, self.object_ref(attacker)?)))
                    .collect();
                HistoryUpdate::BlockersDeclared { blocks }
            }
            GameEvent::Targeted { target, ability_source, controller, .. } => HistoryUpdate::Targeted {
                row: TargetingRow { target: *target, controller: *controller, by_spell: ability_source.is_none(), seq },
            },
            GameEvent::PhaseBegin { phase: PhaseType::Combat, .. } => HistoryUpdate::CombatBegan,
            // CR 506.4, 511.3: what each was attacking as it left combat.
            GameEvent::RemovedFromCombat { removed } => HistoryUpdate::RemovedFromCombat {
                exits: removed.iter().map(|exit| (exit.object, exit.attacking)).collect(),
            },
            _ => return None,
        })
    }
}
