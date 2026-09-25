//! Every player's history, materialized and bounded (`triggers-architecture.md`
//! §3.10; `codebase-state.md` item 179).
//!
//! Four views, each sized by the table and never by the turn count: this
//! turn's row and last turn's, the game's running total, and, for "since your
//! last turn", every player's total as it stood when this player's last turn
//! ended. The dispatcher is the one writer (`engine::triggers::history`);
//! nothing derives these from the event stream.

use crate::types::history::TurnFact;
use crate::types::ids::PlayerId;

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

    /// Add `n` to `fact`'s count and return the new count.
    pub(crate) fn add(&mut self, fact: TurnFact, n: u64) -> u64 {
        let count = &mut self.counts[fact.slot()];
        *count += n;
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
    /// What "since your last turn" subtracts: every player's `this_game`, by
    /// `PlayerId`, as it stood when this player's most recent turn ended,
    /// taken as the next turn began. The span crosses the other seats' turns,
    /// which no row keeps, so it is read as a total now less this one: after
    /// player 0's turn 5, "since your last turn" on turn 9 is turns 6 to 9.
    /// One row per player, because "an opponent lost life since your last
    /// turn" reads the other players' counts. Empty until this player's first
    /// turn has ended, which reads as zeros.
    at_your_last_turn: Vec<TurnSummary>,
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
            at_your_last_turn: Vec::new(),
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

    /// `player`'s counts since this player's last turn, the turn in progress
    /// included, where `theirs` is `player`'s history. "Your last turn" is
    /// your most recent turn to have ended, so on your own turn it is the one
    /// before.
    pub fn since_your_last_turn(&self, player: PlayerId, theirs: &PlayerHistory) -> TurnSummary {
        theirs.this_game.since(self.at_your_last_turn.get(player).unwrap_or(&TurnSummary::ZERO))
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
    pub(crate) fn last_turn_ended(&mut self, totals: Vec<TurnSummary>) {
        self.at_your_last_turn = totals;
    }

    pub(crate) fn record_own_turn(&mut self, turn: u32) {
        self.own_turn = Some(turn);
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
        game.players[1].history.add(1, DRAWN, 5);
        game.begin_turn_history(2, 1);
        game.players[1].history.add(2, DRAWN, 1);
        let since = |game: &GameState| game.players[0].history.since_your_last_turn(1, &game.players[1].history);
        assert_eq!(since(&game).count(DRAWN), 1);

        game.begin_turn_history(3, 0);
        game.players[1].history.add(3, DRAWN, 1);
        assert_eq!(since(&game).count(DRAWN), 2, "on your own turn, your last turn is the one before");
    }

    /// An extra turn: player 0 takes turns 1 and 2, and on turn 2 "your last
    /// turn" is turn 1, just ended.
    #[test]
    fn after_an_extra_turn_your_last_turn_is_the_one_just_ended() {
        let mut game = GameState::new(2, 20);
        game.players[0].history.add(1, DRAWN, 4);
        game.begin_turn_history(2, 0);
        game.players[0].history.add(2, DRAWN, 1);
        let mine = &game.players[0].history;
        assert_eq!(mine.since_your_last_turn(0, mine).count(DRAWN), 1);
    }
}
