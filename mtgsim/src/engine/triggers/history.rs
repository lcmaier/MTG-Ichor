//! The histories' one writer (`triggers-architecture.md` §3.10, §4.1).
//!
//! Every record advances the histories once, in window order, **before the
//! gate** asks whether anything could trigger: a card that arrives later in
//! the turn reads what happened before it arrived (Vengeful Warchief's fourth
//! ruling, Paladin of Atonement's first). A record is counted as its event
//! ends: at its window's dispatch, or, for one card of a draw instruction, as
//! that card's draw ends, since the next card is decided after it (CR 121.2).

use crate::engine::layers::compute::compute_characteristics;
use crate::events::event::{DamageTarget, EventSeq, GameEvent};
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::{AbilityIdentity, GameState, PhaseType, StepType};
use crate::state::history::{CombatFact, PlayerHistory};
use crate::types::card_types::CardType;
use crate::types::history::TurnFact;
use crate::types::ids::{ObjectRef, PlayerId};
use crate::types::zones::Zone;
use crate::state::layer_memo::GatedWalkInput;

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
        let updates: Vec<(EventSeq, Option<HistoryUpdate>)> = window
            .iter()
            .filter(|&&seq| self.events.record(seq).is_some_and(|r| !r.counted()))
            .map(|&seq| (seq, self.history_update(seq)))
            .collect();
        let mut wrote = false;
        for (seq, update) in updates {
            let (mut in_turn, mut in_draw_step) = (None, None);
            if let Some(update) = update {
                self.count(&update);
                in_turn = self.place_in_turn(&update);
                in_draw_step = self.place_in_draw_step(&update);
                wrote = true;
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
            HistoryUpdate::CombatBegan => self.history.combat.combat_began(),
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
            GameEvent::PhaseBegin { phase: PhaseType::Combat, .. } => HistoryUpdate::CombatBegan,
            // CR 506.4, 511.3: what each was attacking as it left combat.
            GameEvent::RemovedFromCombat { removed } => HistoryUpdate::RemovedFromCombat {
                exits: removed.iter().map(|exit| (exit.object, exit.attacking)).collect(),
            },
            _ => return None,
        })
    }
}
