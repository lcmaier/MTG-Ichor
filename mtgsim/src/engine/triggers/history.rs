//! The histories' one writer (`triggers-architecture.md` §3.10, §4.1).
//!
//! Every record of every window advances the turn summaries, in window
//! order, **before the gate** asks whether anything could trigger: a card
//! that arrives later in the turn reads what happened before it arrived
//! (Vengeful Warchief's fourth ruling, Paladin of Atonement's first).

use crate::engine::layers::compute::compute_characteristics;
use crate::events::event::{DamageTarget, EventSeq, GameEvent};
use crate::state::game_state::GameState;
use crate::types::card_types::CardType;
use crate::types::ids::PlayerId;
use crate::types::zones::Zone;

/// What one record adds to whose row, read before anything is written so the
/// read can take the layer walk.
enum Tally {
    TurnBegan { player: PlayerId, turn: u32 },
    SpellCast { caster: PlayerId, types: Vec<CardType> },
    CardDrawn { player: PlayerId },
    LifeGained { player: PlayerId, amount: u64 },
    LifeLost { player: PlayerId, amount: u64 },
    DamageTaken { player: PlayerId, amount: u64 },
    CreatureDied { controller: PlayerId },
    AttackersDeclared { player: PlayerId, count: u32 },
}

impl Tally {
    /// Whose row it is counted on.
    fn player(&self) -> PlayerId {
        match self {
            Tally::TurnBegan { player, .. }
            | Tally::SpellCast { caster: player, .. }
            | Tally::CardDrawn { player }
            | Tally::LifeGained { player, .. }
            | Tally::LifeLost { player, .. }
            | Tally::DamageTaken { player, .. }
            | Tally::CreatureDied { controller: player }
            | Tally::AttackersDeclared { player, .. } => *player,
        }
    }
}

impl GameState {
    /// Advance the summaries of the turn in progress by `window`'s records.
    pub(crate) fn advance_history(&mut self, window: &[EventSeq]) {
        let tallies: Vec<Tally> = window.iter().filter_map(|&seq| self.tally(seq)).collect();
        let turn = self.turn_number;
        for tally in tallies {
            let Some(history) = self.history.get_mut(tally.player()) else { continue };
            if let Tally::TurnBegan { turn: began, .. } = tally {
                history.record_own_turn(began);
                continue;
            }
            let Some(row) = history.turn_mut(turn) else { continue };
            match tally {
                Tally::TurnBegan { .. } => {}
                Tally::SpellCast { types, .. } => {
                    row.spells_cast += 1;
                    for card_type in types {
                        row.count_spell_of_type(card_type);
                    }
                }
                Tally::CardDrawn { .. } => row.cards_drawn += 1,
                Tally::LifeGained { amount, .. } => {
                    row.life_gained += amount;
                    row.life_gain_events += 1;
                }
                Tally::LifeLost { amount, .. } => {
                    row.life_lost += amount;
                    row.life_loss_events += 1;
                }
                Tally::DamageTaken { amount, .. } => row.damage_taken += amount,
                Tally::CreatureDied { .. } => row.controlled_creatures_died += 1,
                Tally::AttackersDeclared { count, .. } => row.attackers_declared += count,
            }
        }
    }

    /// What `seq` adds, or `None` for a record no summary counts.
    fn tally(&self, seq: EventSeq) -> Option<Tally> {
        let record = self.events.record(seq)?;
        Some(match &record.event {
            GameEvent::TurnBegin { player, turn_number } => Tally::TurnBegan { player: *player, turn: *turn_number },
            // CR 601.2i: the spell is cast, and on the stack, as this record
            // is dispatched, so its types are the ones it was cast with.
            GameEvent::SpellCast { spell_id, caster } => {
                let mut types: Vec<CardType> = compute_characteristics(self, *spell_id)
                    .map(|chars| chars.types.iter().copied().collect())
                    .unwrap_or_default();
                types.sort_by_key(|t| *t as u8);
                Tally::SpellCast { caster: *caster, types }
            }
            GameEvent::CardDrawn { player_id, .. } => Tally::CardDrawn { player: *player_id },
            GameEvent::LifeChanged { player_id, old, new, .. } if new > old => {
                Tally::LifeGained { player: *player_id, amount: (new - old) as u64 }
            }
            GameEvent::LifeChanged { player_id, old, new, .. } if new < old => {
                Tally::LifeLost { player: *player_id, amount: (old - new) as u64 }
            }
            GameEvent::DamageDealt { target: DamageTarget::Player(pid), amount, .. } => {
                Tally::DamageTaken { player: *pid, amount: *amount }
            }
            // CR 700.4's "dies", under whoever controlled the creature as it did.
            GameEvent::ZoneChange { from: Zone::Battlefield, to: Zone::Graveyard, lki: Some(frame), .. }
                if frame.types.contains(&CardType::Creature) =>
            {
                Tally::CreatureDied { controller: frame.controller }
            }
            // CR 508.1: the active player declares attackers.
            GameEvent::AttackersDeclared { attackers } if !attackers.is_empty() => {
                Tally::AttackersDeclared { player: self.active_player, count: attackers.len() as u32 }
            }
            _ => return None,
        })
    }

    /// CR 103.5's opening hands are drawn before the first turn begins, so no
    /// turn's row holds them.
    pub(crate) fn forget_pregame_history(&mut self) {
        for history in &mut self.history {
            history.turns.clear();
        }
    }
}
