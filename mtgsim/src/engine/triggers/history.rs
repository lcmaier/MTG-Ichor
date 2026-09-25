//! The histories' one writer (`triggers-architecture.md` §3.10, §4.1).
//!
//! Every record of every window advances the turn summaries, in window
//! order, **before the gate** asks whether anything could trigger: a card
//! that arrives later in the turn reads what happened before it arrived
//! (Vengeful Warchief's fourth ruling, Paladin of Atonement's first).

use crate::engine::layers::compute::compute_characteristics;
use crate::events::event::{DamageTarget, EventSeq, GameEvent};
use crate::state::game_state::{AbilityIdentity, GameState};
use crate::types::card_types::CardType;
use crate::types::history::TurnFact;
use crate::types::ids::PlayerId;
use crate::types::zones::Zone;

/// Each record's place among its player's records of the same kind this
/// turn, 1 for the first: what "for the first time each turn" reads (§3.5).
/// Only the kinds a summary counts events of have one: a cast, a draw, a
/// gain, a loss.
#[derive(Debug)]
pub(crate) struct TurnOrdinals(Vec<(EventSeq, u64)>);

impl TurnOrdinals {
    pub(crate) fn place_in_turn(&self, seq: EventSeq) -> Option<u64> {
        self.0.iter().find(|(s, _)| *s == seq).map(|(_, n)| *n)
    }
}

/// What one record adds to whose row, read before anything is written so the
/// read can take the layer walk.
enum HistoryUpdate {
    TurnBegan { player: PlayerId, turn: u32 },
    AbilityResolved { identity: AbilityIdentity },
    SpellCast { caster: PlayerId, types: Vec<CardType> },
    CardDrawn { player: PlayerId },
    LifeGained { player: PlayerId, amount: u64 },
    LifeLost { player: PlayerId, amount: u64 },
    DamageTaken { player: PlayerId, amount: u64 },
    CreatureDied { controller: PlayerId },
    AttackersDeclared { player: PlayerId, count: u64 },
}

impl HistoryUpdate {
    /// Whose row it is counted on; `None` for a count that is the game's.
    fn whose_row(&self) -> Option<PlayerId> {
        Some(match self {
            HistoryUpdate::AbilityResolved { .. } => return None,
            HistoryUpdate::TurnBegan { player, .. }
            | HistoryUpdate::SpellCast { caster: player, .. }
            | HistoryUpdate::CardDrawn { player }
            | HistoryUpdate::LifeGained { player, .. }
            | HistoryUpdate::LifeLost { player, .. }
            | HistoryUpdate::DamageTaken { player, .. }
            | HistoryUpdate::CreatureDied { controller: player }
            | HistoryUpdate::AttackersDeclared { player, .. } => *player,
        })
    }
}

impl GameState {
    /// Advance the summaries of the turn in progress by `window`'s records,
    /// and say where each record the event counts count falls in its turn.
    pub(crate) fn advance_history(&mut self, window: &[EventSeq]) -> TurnOrdinals {
        let updates: Vec<(EventSeq, HistoryUpdate)> =
            window.iter().filter_map(|&seq| self.history_update(seq).map(|u| (seq, u))).collect();
        let turn = self.turn_number;
        let mut ordinals = TurnOrdinals(Vec::with_capacity(updates.len()));
        for (seq, update) in updates {
            if let HistoryUpdate::AbilityResolved { identity } = update {
                let key = (identity.source, identity.ability.definition());
                *self.resolutions_this_turn.entry(key).or_insert(0) += 1;
                continue;
            }
            let Some(history) = update.whose_row().and_then(|p| self.players.get_mut(p)).map(|p| &mut p.history)
            else {
                continue;
            };
            if let HistoryUpdate::TurnBegan { turn: began, .. } = update {
                history.record_own_turn(began);
                continue;
            }
            let Some(row) = history.turn_mut(turn) else { continue };
            let place = match update {
                HistoryUpdate::TurnBegan { .. } | HistoryUpdate::AbilityResolved { .. } => None,
                HistoryUpdate::SpellCast { types, .. } => {
                    for card_type in types {
                        row.add(TurnFact::SpellsCastOfType(card_type), 1);
                    }
                    Some(row.add(TurnFact::SpellsCast, 1))
                }
                HistoryUpdate::CardDrawn { .. } => Some(row.add(TurnFact::CardsDrawn, 1)),
                HistoryUpdate::LifeGained { amount, .. } => {
                    row.add(TurnFact::LifeGained, amount);
                    Some(row.add(TurnFact::LifeGainEvents, 1))
                }
                HistoryUpdate::LifeLost { amount, .. } => {
                    row.add(TurnFact::LifeLost, amount);
                    Some(row.add(TurnFact::LifeLossEvents, 1))
                }
                HistoryUpdate::DamageTaken { amount, .. } => {
                    row.add(TurnFact::DamageTaken, amount);
                    None
                }
                HistoryUpdate::CreatureDied { .. } => {
                    row.add(TurnFact::ControlledCreaturesDied, 1);
                    None
                }
                HistoryUpdate::AttackersDeclared { count, .. } => {
                    row.add(TurnFact::AttackersDeclared, count);
                    None
                }
            };
            if let Some(n) = place {
                ordinals.0.push((seq, n));
            }
        }
        ordinals
    }

    /// How many times `identity`'s ability has resolved this turn (CR 603.7h).
    pub(crate) fn resolutions_this_turn_of(&self, identity: AbilityIdentity) -> u32 {
        self.resolutions_this_turn.get(&(identity.source, identity.ability.definition())).copied().unwrap_or(0)
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
            GameEvent::ZoneChange { from: Zone::Battlefield, to: Zone::Graveyard, lki: Some(frame), .. }
                if frame.types.contains(&CardType::Creature) =>
            {
                HistoryUpdate::CreatureDied { controller: frame.controller }
            }
            // CR 508.1: the active player declares attackers.
            GameEvent::AttackersDeclared { attackers } if !attackers.is_empty() => {
                HistoryUpdate::AttackersDeclared { player: self.active_player, count: attackers.len() as u64 }
            }
            _ => return None,
        })
    }
}
