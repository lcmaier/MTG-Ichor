//! What the game remembers (`triggers-architecture.md` §3.10, §12's TR-5h).
//!
//! Every count a card reads — "this turn", "last turn", "since your last
//! turn", "this game" — is a sum over the history log's rows
//! (`state::history_log`), which keeps every turn. Beside it, each player's
//! own turns, which "your last turn" names, and the draws in their draw step
//! in progress, which no turn can hold: a turn can have two draw steps (CR
//! 500.8). One writer, `engine::triggers::history`, as each event ends.
//!
//! [`GameHistory`] holds it all: the log, each player's [`PlayerHistory`],
//! and each ability's "this turn" facts.

use crate::state::game_state::AbilityIdentity;
use crate::state::history_log::{HistoryLog, TurnsCounted};
use crate::types::history::{HistorySpan, TurnFact};
use crate::types::ids::{AbilityId, IdMap, IdSet, ObjectRef, PlayerId};

/// One player's history beside the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerHistory {
    /// The turn this player most recently began.
    own_turn: Option<u32>,
    /// The turn this player began before that one.
    previous_own_turn: Option<u32>,
    /// The cards drawn since this player's draw step in progress began (CR
    /// 504), read only while it is in progress.
    this_draw_step: u64,
}

impl PlayerHistory {
    /// A player's history before the game's first turn.
    pub fn before_any_turn() -> Self {
        PlayerHistory { own_turn: None, previous_own_turn: None, this_draw_step: 0 }
    }

    pub(crate) fn record_own_turn(&mut self, turn: u32) {
        if self.own_turn != Some(turn) {
            self.previous_own_turn = self.own_turn;
            self.own_turn = Some(turn);
        }
    }

    /// "Your last turn" on turn `now`: this player's most recent turn to
    /// have ended, so on their own turn the one before. `None` before any
    /// has.
    pub fn your_last_turn(&self, now: u32) -> Option<u32> {
        if self.own_turn == Some(now) { self.previous_own_turn } else { self.own_turn }
    }

    /// The cards drawn in this player's draw step in progress.
    pub fn this_draw_step(&self) -> u64 {
        self.this_draw_step
    }

    /// This player's draw step has begun (CR 504), or a scenario states how
    /// many cards were drawn in it so far.
    pub(crate) fn draw_step_drawn(&mut self, n: u64) {
        self.this_draw_step = n;
    }

    /// Count a card drawn in this player's draw step.
    pub(crate) fn drew_in_draw_step(&mut self) {
        self.this_draw_step = self.this_draw_step.saturating_add(1);
    }
}

/// What the game remembers: the log, each player's history, and what belongs
/// to no one player.
#[derive(Debug, Clone)]
pub struct GameHistory {
    /// Each player's history, by `PlayerId`. A player who has left the game
    /// keeps theirs, since an effect can still find what they did (CR
    /// 800.4i).
    pub players: Vec<PlayerHistory>,
    /// CR 603.2h — "do this only once each turn": each ability whose action
    /// its controller has taken this turn, with that controller, since the
    /// rule reads "its source's controller" (`triggers-architecture.md` §3.5).
    /// Written by the resolution that takes the action; read at the trigger
    /// and at resolution.
    pub action_taken_this_turn: IdSet<(AbilityIdentity, PlayerId)>,
    /// "This ability triggers only once each turn" (Elvish Warmaster's
    /// ruling): each ability that has triggered this turn. Written by the
    /// dispatcher as it queues; read at the trigger.
    pub triggered_this_turn: IdSet<AbilityIdentity>,
    /// CR 603.7h — how many times each ability has resolved this turn, keyed
    /// by the ability rather than by who controlled it (Ashling the Pilgrim's
    /// ruling), so a control change does not restart it. Advanced off
    /// `AbilityResolved` by the dispatcher.
    pub resolutions_this_turn: IdMap<(ObjectRef, AbilityId), u32>,
    /// Every event a card can ask about later, for the whole game
    /// (`state::history_log`): what each player and each object did.
    pub log: HistoryLog,
}

impl GameHistory {
    /// The history of a game of `seats` players before its first turn:
    /// nothing counted.
    pub fn new(seats: usize) -> Self {
        GameHistory {
            players: vec![PlayerHistory::before_any_turn(); seats],
            action_taken_this_turn: IdSet::default(),
            triggered_this_turn: IdSet::default(),
            resolutions_this_turn: IdMap::default(),
            log: HistoryLog::default(),
        }
    }

    /// `player`'s count of `fact` over `turns`.
    pub fn count(&self, player: PlayerId, fact: TurnFact, turns: TurnsCounted) -> u64 {
        self.log.count(fact, turns, &|counted| counted == player, &mut |_| true, &|_| {})
    }

    /// The turns `span` names for `you` on turn `now`.
    pub fn turns_of(&self, span: HistorySpan, you: PlayerId, now: u32) -> TurnsCounted {
        match span {
            HistorySpan::ThisTurn => TurnsCounted::Turn(now),
            HistorySpan::LastTurn => TurnsCounted::Turn(now.saturating_sub(1)),
            HistorySpan::SinceYourLastTurn => self.since_your_last_turn(you, now),
            HistorySpan::ThisGame => TurnsCounted::From(1),
        }
    }

    /// The turns "since your last turn" spans for `you` on turn `now`: every
    /// turn after your most recent one to have ended, this one included, so
    /// the whole game before you have had one.
    pub fn since_your_last_turn(&self, you: PlayerId, now: u32) -> TurnsCounted {
        match self.players.get(you).and_then(|yours| yours.your_last_turn(now)) {
            Some(last) => TurnsCounted::From(last + 1),
            None => TurnsCounted::From(1),
        }
    }

    /// A turn has begun. "Each turn" is the game's turn, not a controller's
    /// (§3.5), so the abilities' facts start over; the log keeps every turn.
    pub(crate) fn turn_began(&mut self) {
        self.action_taken_this_turn.clear();
        self.triggered_this_turn.clear();
        self.resolutions_this_turn.clear();
        self.log.turn_began();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event::EventSeq;
    use crate::state::history_log::{HistoryRow, RowFact};

    const DRAWN: TurnFact = TurnFact::CardsDrawn;

    /// `count` cards drawn by `player` on `turn`, as a scenario states them.
    fn drew(history: &mut GameHistory, player: PlayerId, turn: u32, count: u64) {
        let fact = RowFact::Stated { fact: DRAWN, count };
        let seq = EventSeq(history.log.len());
        history.log.push(HistoryRow { seq, turn, combat: 0, player: Some(player), object: None, frame: None, fact });
    }

    /// A turn's count is its rows, whoever's turn it was; a turn with none
    /// reads zero, and "this game" is every turn's.
    #[test]
    fn a_turns_count_is_its_rows() {
        let mut history = GameHistory::new(2);
        drew(&mut history, 0, 3, 2);
        drew(&mut history, 0, 5, 1);
        assert_eq!(history.count(0, DRAWN, TurnsCounted::Turn(3)), 2);
        assert_eq!(history.count(0, DRAWN, TurnsCounted::Turn(4)), 0);
        assert_eq!(history.count(0, DRAWN, TurnsCounted::Turn(5)), 1);
        assert_eq!(history.count(0, DRAWN, TurnsCounted::From(1)), 3);
        assert_eq!(history.count(1, DRAWN, TurnsCounted::From(1)), 0, "the rows are player 0's");
    }

    /// "Since your last turn" for player 0, read off player 1's counts: the
    /// span starts when player 0's turn ends, so a count on player 0's own
    /// turn is before it and a count on any later turn is inside it.
    #[test]
    fn since_your_last_turn_starts_when_your_turn_ends() {
        let mut history = GameHistory::new(2);
        history.players[0].record_own_turn(1);
        drew(&mut history, 1, 1, 5);
        history.players[1].record_own_turn(2);
        drew(&mut history, 1, 2, 1);
        let since = |history: &GameHistory, now| history.count(1, DRAWN, history.since_your_last_turn(0, now));
        assert_eq!(since(&history, 2), 1);

        history.players[0].record_own_turn(3);
        drew(&mut history, 1, 3, 1);
        assert_eq!(since(&history, 3), 2, "on your own turn, your last turn is the one before");
    }

    /// An extra turn: player 0 takes turns 1 and 2, and on turn 2 "your last
    /// turn" is turn 1, just ended.
    #[test]
    fn after_an_extra_turn_your_last_turn_is_the_one_just_ended() {
        let mut history = GameHistory::new(2);
        history.players[0].record_own_turn(1);
        drew(&mut history, 0, 1, 4);
        history.players[0].record_own_turn(2);
        drew(&mut history, 0, 2, 1);
        assert_eq!(history.count(0, DRAWN, history.since_your_last_turn(0, 2)), 1);
    }

    /// The draw step's count is not a turn's: it starts over when a draw step
    /// begins, and a turn can have two (CR 500.8).
    #[test]
    fn the_draw_steps_count_starts_over_with_each_draw_step() {
        let mut history = PlayerHistory::before_any_turn();
        history.drew_in_draw_step();
        history.drew_in_draw_step();
        assert_eq!(history.this_draw_step(), 2);
        history.draw_step_drawn(0);
        history.drew_in_draw_step();
        assert_eq!(history.this_draw_step(), 1, "a second draw step in the same turn");
    }
}
