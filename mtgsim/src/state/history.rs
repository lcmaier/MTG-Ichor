//! Every player's history, and the game's, materialized and bounded
//! (`triggers-architecture.md` §3.10; `codebase-state.md` item 179).
//!
//! Four views, each sized by the table and never by the turn count: this
//! turn's row and last turn's, the game's running total, and, for "since your
//! last turn", every player's total as it stood when this player's last turn
//! ended. Beside them, the draws in the player's draw step in progress, which
//! no turn row can hold: a turn can have two draw steps (CR 500.8). One writer,
//! `engine::triggers::history`, as each event ends; nothing derives these from
//! the event stream.
//!
//! [`GameHistory`] holds it all: each player's [`PlayerHistory`], each
//! ability's "this turn" facts, and what each creature did in combat.

use std::sync::Arc;

use crate::state::game_state::AbilityIdentity;
use crate::state::history_log::HistoryLog;
use crate::types::history::TurnFact;
use crate::types::ids::{AbilityId, IdMap, IdSet, ObjectRef, PlayerId};

/// A count for each [`TurnFact`], in the fact's slot: one player's side of one
/// turn, or of a span of turns. The fact is the key and the row holds nothing
/// else, so a new quantity is a `TurnFact` variant and an arm in the writer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnSummary {
    counts: [u64; TurnFact::COUNT],
}

impl TurnSummary {
    /// Nothing counted.
    pub const ZERO: TurnSummary = TurnSummary { counts: [0; TurnFact::COUNT] };

    /// The row's count of `fact`.
    pub fn count(&self, fact: TurnFact) -> u64 {
        self.counts[fact.slot()]
    }

    /// Add `n` to `fact`'s count and return the new count, which stays at
    /// `u64::MAX` past it.
    pub(crate) fn add(&mut self, fact: TurnFact, n: u64) -> u64 {
        let count = &mut self.counts[fact.slot()];
        *count = count.saturating_add(n);
        *count
    }

    /// What `self` counted that `earlier`, a total taken before it, did not.
    fn since(&self, earlier: &TurnSummary) -> TurnSummary {
        let mut counts = self.counts;
        for (count, before) in counts.iter_mut().zip(earlier.counts.iter()) {
            *count -= before;
        }
        TurnSummary { counts }
    }
}

/// One player's history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerHistory {
    /// The turn `this_turn` counts; `last_turn` counts the turn before it.
    turn: u32,
    this_turn: TurnSummary,
    last_turn: TurnSummary,
    /// Every turn so far.
    this_game: TurnSummary,
    /// The turn this player most recently began.
    own_turn: Option<u32>,
    /// The turn this player began before that one.
    previous_own_turn: Option<u32>,
    /// What "since your last turn" subtracts: every player's `this_game`, by
    /// `PlayerId`, as it stood when this player's most recent turn ended,
    /// taken as the next turn began. The span crosses the other seats' turns,
    /// which no row keeps, so it is read as a total now less this one: after
    /// player 0's turn 5, "since your last turn" on turn 9 is turns 6 to 9.
    /// One row per player, because "an opponent lost life since your last
    /// turn" reads the other players' counts. Empty until this player's first
    /// turn has ended, which reads as zeros.
    ///
    /// Shared, so a clone copies a pointer: it is seats × seats summaries in
    /// all, and its one writer replaces it whole once a turn rather than
    /// editing it, so no clone ever has to copy it.
    at_your_last_turn: Arc<[TurnSummary]>,
    /// The cards drawn since this player's draw step in progress began (CR
    /// 504), read only while it is in progress.
    this_draw_step: u64,
}

impl PlayerHistory {
    /// A player's history before the game's first turn: nothing counted.
    pub fn before_any_turn() -> Self {
        PlayerHistory {
            turn: 0,
            this_turn: TurnSummary::ZERO,
            last_turn: TurnSummary::ZERO,
            this_game: TurnSummary::ZERO,
            own_turn: None,
            previous_own_turn: None,
            at_your_last_turn: Arc::from([]),
            this_draw_step: 0,
        }
    }

    /// Turn `now`'s row.
    pub fn this_turn(&self, now: u32) -> TurnSummary {
        if self.turn == now { self.this_turn } else { TurnSummary::ZERO }
    }

    /// The row of turn `now - 1`, whoever's turn it was.
    pub fn last_turn(&self, now: u32) -> TurnSummary {
        if self.turn == now {
            self.last_turn
        } else if self.turn + 1 == now {
            self.this_turn
        } else {
            TurnSummary::ZERO
        }
    }

    /// Every turn of the game so far, summed.
    pub fn this_game(&self) -> TurnSummary {
        self.this_game
    }

    /// Count `n` of `fact` on turn `now`'s row, moving the rows along if `now`
    /// is a turn this history has not counted on yet, and return the row's
    /// new count. Turn 0 has no row: nothing happens before the first turn but
    /// CR 103's setup.
    pub(crate) fn add(&mut self, now: u32, fact: TurnFact, n: u64) -> u64 {
        debug_assert!(now >= 1, "turn 0 has no row");
        if self.turn != now {
            self.last_turn = self.last_turn(now);
            self.this_turn = TurnSummary::ZERO;
            self.turn = now;
        }
        self.this_game.add(fact, n);
        self.this_turn.add(fact, n)
    }

    /// The counts on turns before `turn`.
    pub(crate) fn before(&self, turn: u32) -> TurnSummary {
        self.this_game.since(&self.this_turn(turn))
    }

    /// Whether this player's most recent turn is the one before `turn`.
    pub(crate) fn took_the_turn_before(&self, turn: u32) -> bool {
        self.own_turn.is_some_and(|own| own + 1 == turn)
    }

    /// This player's last turn has ended, and `totals` is every player's
    /// count up to its end.
    pub(crate) fn last_turn_ended(&mut self, totals: Arc<[TurnSummary]>) {
        self.at_your_last_turn = totals;
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

/// What the game remembers: each player's history, and what belongs to no one
/// player.
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
    /// (`state::history_log`): what each object did, the combats', the
    /// targetings'. Written beside the players' counts until their readers
    /// move onto it.
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

    /// `them`'s counts since `you`'s last turn, the turn in progress
    /// included. "Your last turn" is your most recent turn to have ended, so
    /// on your own turn it is the one before. Zeros for a player the game
    /// does not seat.
    pub fn since_your_last_turn(&self, you: PlayerId, them: PlayerId) -> TurnSummary {
        let (Some(yours), Some(theirs)) = (self.players.get(you), self.players.get(them)) else {
            return TurnSummary::ZERO;
        };
        theirs.this_game.since(yours.at_your_last_turn.get(them).unwrap_or(&TurnSummary::ZERO))
    }

    /// Count turn `turn` as begun, `active` taking it. Whoever took the turn
    /// before it has just had their last turn end, so their "since your last
    /// turn" starts from every player's count now. The counts exclude the new turn's own, which a record earlier in the same
    /// window may already have added.
    pub(crate) fn count_turn_began(&mut self, turn: u32, active: PlayerId) {
        if let Some(previous) = self.players.iter().position(|p| p.took_the_turn_before(turn)) {
            let totals: Arc<[TurnSummary]> = self.players.iter().map(|p| p.before(turn)).collect();
            self.players[previous].last_turn_ended(totals);
        }
        if let Some(player) = self.players.get_mut(active) {
            player.record_own_turn(turn);
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
    use crate::state::game_state::GameState;

    const DRAWN: TurnFact = TurnFact::CardsDrawn;

    /// The two rows move along with the turn: last turn's is the row of the
    /// turn before the one asked about, whoever's turn it was, and a turn
    /// with nothing counted reads as zero.
    #[test]
    fn this_turn_and_last_turn_follow_the_turn_asked_about() {
        let mut history = PlayerHistory::before_any_turn();
        history.add(3, DRAWN, 2);
        assert_eq!(history.this_turn(3).count(DRAWN), 2);
        assert_eq!(history.last_turn(4).count(DRAWN), 2);
        assert_eq!(history.this_turn(4).count(DRAWN), 0);
        assert_eq!(history.last_turn(5).count(DRAWN), 0, "two turns later it is nobody's last turn");

        history.add(5, DRAWN, 1);
        assert_eq!(history.last_turn(5).count(DRAWN), 0, "turn 4 counted nothing");
        assert_eq!(history.this_game().count(DRAWN), 3);
    }

    /// "Since your last turn" for player 0, read off player 1's counts: the
    /// span starts when player 0's turn ends, so a count on player 0's own
    /// turn is before it and a count on any later turn is inside it.
    #[test]
    fn since_your_last_turn_starts_when_your_turn_ends() {
        let mut game = GameState::new(2, 20);
        game.history.players[1].add(1, DRAWN, 5);
        game.history.count_turn_began(2, 1);
        game.history.players[1].add(2, DRAWN, 1);
        let since = |game: &GameState| game.history.since_your_last_turn(0, 1);
        assert_eq!(since(&game).count(DRAWN), 1);

        game.history.count_turn_began(3, 0);
        game.history.players[1].add(3, DRAWN, 1);
        assert_eq!(since(&game).count(DRAWN), 2, "on your own turn, your last turn is the one before");
    }

    /// An extra turn: player 0 takes turns 1 and 2, and on turn 2 "your last
    /// turn" is turn 1, just ended.
    #[test]
    fn after_an_extra_turn_your_last_turn_is_the_one_just_ended() {
        let mut game = GameState::new(2, 20);
        game.history.players[0].add(1, DRAWN, 4);
        game.history.count_turn_began(2, 0);
        game.history.players[0].add(2, DRAWN, 1);
        assert_eq!(game.history.since_your_last_turn(0, 0).count(DRAWN), 1);
    }

    /// The draw step's count is not a turn row: it starts over when a draw
    /// step begins, and a turn can have two (CR 500.8).
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
