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
use crate::state::game_state::{AbilityIdentity, GameState, StepType};
use crate::state::history::TurnSummary;
use crate::types::card_types::CardType;
use crate::types::history::TurnFact;
use crate::types::ids::PlayerId;
use crate::types::zones::Zone;

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
            | HistoryUpdate::DrawStepBegan { player }
            | HistoryUpdate::LifeGained { player, .. }
            | HistoryUpdate::LifeLost { player, .. }
            | HistoryUpdate::DamageTaken { player, .. }
            | HistoryUpdate::CreatureDied { controller: player }
            | HistoryUpdate::AttackersDeclared { player, .. } => *player,
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
        for (seq, update) in updates {
            let (in_turn, in_draw_step) = update.map_or((None, None), |update| self.count(update));
            self.events.note_counted(seq, in_turn, in_draw_step);
        }
    }

    /// Count one record's update, and return its place in its turn and in its
    /// player's draw step.
    fn count(&mut self, update: HistoryUpdate) -> (Option<u64>, Option<u64>) {
        let turn = self.turn_number;
        match update {
            HistoryUpdate::AbilityResolved { identity } => {
                let key = (identity.source, identity.ability.definition());
                *self.resolutions_this_turn.entry(key).or_insert(0) += 1;
                return (None, None);
            }
            HistoryUpdate::TurnBegan { player, turn: began } => {
                self.begin_turn_history(began, player);
                return (None, None);
            }
            _ => {}
        }
        // CR 103: the opening hands are drawn before any turn, on no row.
        if turn == 0 {
            return (None, None);
        }
        let in_draw_step = matches!(update, HistoryUpdate::CardDrawn { player } if self.in_own_draw_step(player));
        let Some(history) = update.whose_row().and_then(|p| self.players.get_mut(p)).map(|p| &mut p.history) else {
            return (None, None);
        };
        match update {
            HistoryUpdate::TurnBegan { .. } | HistoryUpdate::AbilityResolved { .. } => (None, None),
            HistoryUpdate::SpellCast { types, .. } => {
                for card_type in types {
                    history.add(turn, TurnFact::SpellsCastOfType(card_type), 1);
                }
                (Some(history.add(turn, TurnFact::SpellsCast, 1)), None)
            }
            HistoryUpdate::CardDrawn { .. } => {
                (Some(history.add(turn, TurnFact::CardsDrawn, 1)), in_draw_step.then(|| history.drew_in_draw_step()))
            }
            HistoryUpdate::DrawStepBegan { .. } => {
                history.draw_step_drawn(0);
                (None, None)
            }
            HistoryUpdate::LifeGained { amount, .. } => {
                history.add(turn, TurnFact::LifeGained, amount);
                (Some(history.add(turn, TurnFact::LifeGainEvents, 1)), None)
            }
            HistoryUpdate::LifeLost { amount, .. } => {
                history.add(turn, TurnFact::LifeLost, amount);
                (Some(history.add(turn, TurnFact::LifeLossEvents, 1)), None)
            }
            HistoryUpdate::DamageTaken { amount, .. } => {
                history.add(turn, TurnFact::DamageTaken, amount);
                (None, None)
            }
            HistoryUpdate::CreatureDied { .. } => {
                history.add(turn, TurnFact::ControlledCreaturesDied, 1);
                (None, None)
            }
            HistoryUpdate::AttackersDeclared { count, .. } => {
                history.add(turn, TurnFact::AttackersDeclared, count);
                (None, None)
            }
        }
    }

    /// Turn `turn` has begun, `active` taking it. Whoever took the turn
    /// before it has just had their last turn end, so their "since your last
    /// turn" starts from every player's count now (item 179's snapshot). The
    /// counts exclude the new turn's own, which a record earlier in the same
    /// window may already have added.
    pub(crate) fn begin_turn_history(&mut self, turn: u32, active: PlayerId) {
        if let Some(previous) = self.players.iter().position(|p| p.history.took_the_turn_before(turn)) {
            let totals: Vec<TurnSummary> = self.players.iter().map(|p| p.history.before(turn)).collect();
            self.players[previous].history.last_turn_ended(totals);
        }
        if let Some(player) = self.players.get_mut(active) {
            player.history.record_own_turn(turn);
        }
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
            // CR 508.1: the active player declares attackers.
            GameEvent::AttackersDeclared { attackers } if !attackers.is_empty() => {
                HistoryUpdate::AttackersDeclared { player: self.active_player, count: attackers.len() as u64 }
            }
            _ => return None,
        })
    }
}
