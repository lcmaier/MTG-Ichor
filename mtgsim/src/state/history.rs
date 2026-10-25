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

use std::ops::RangeInclusive;

use crate::oracle::characteristics::ObjectInformation;
use crate::state::game_state::AbilityIdentity;
use crate::state::history_log::{HistoryLog, HistoryRow, added_to_count};
use crate::state::layer_memo::GatedWalkInput;
use crate::types::effects::ObjectFilter;
use crate::types::history::{HistoryCount, HistorySpan, TurnFact};
use crate::types::ids::{IdMap, IdSet, PlayerId};

/// One player's history beside the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerHistory {
    /// The turn this player most recently began.
    most_recent_turn: Option<u32>,
    /// The turn this player began before that one.
    previous_turn: Option<u32>,
    /// The cards drawn since this player's draw step in progress began (CR
    /// 504), read only while it is in progress.
    this_draw_step: u64,
    /// This player's count of each fact a record is placed by, on the newest
    /// turn the log has a row of.
    turn_tally: TurnTally,
}

/// The facts a record's place in its turn is counted among (CR 119.9's
/// second gain this turn, a second spell, a third draw): each player's
/// [`TurnTally`] keeps them, so placing a record does not walk the turn.
pub(crate) const PLACED_FACTS: [TurnFact; 4] = [TurnFact::SpellsCast, TurnFact::CardsDrawn, TurnFact::LifeGainEvents, TurnFact::LifeLossEvents];

/// One player's [`PLACED_FACTS`] summed over the rows of turn `turn`, the
/// newest turn written, as [`HistoryLog::count`] sums them.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TurnTally {
    turn: u32,
    counts: [u64; PLACED_FACTS.len()],
}

impl TurnTally {
    const NONE: TurnTally = TurnTally { turn: 0, counts: [0; PLACED_FACTS.len()] };

    /// Add `row` for `player`. A row of a later turn starts the tally over;
    /// one of an earlier turn adds nothing, since rows are written in turn
    /// order.
    fn add(&mut self, row: &HistoryRow, player: PlayerId) {
        if row.turn > self.turn {
            *self = TurnTally { turn: row.turn, ..TurnTally::NONE };
        }
        if row.turn == self.turn {
            for (count, fact) in self.counts.iter_mut().zip(PLACED_FACTS) {
                *count = count.saturating_add(added_to_count(fact, row, &|whose| whose == player));
            }
        }
    }

    /// The count of `fact` on turn `turn`; `None` for a fact it does not keep
    /// or a turn before the one it holds.
    fn count(&self, fact: TurnFact, turn: u32) -> Option<u64> {
        let at = PLACED_FACTS.iter().position(|&placed| placed == fact)?;
        match turn.cmp(&self.turn) {
            std::cmp::Ordering::Equal => Some(self.counts[at]),
            std::cmp::Ordering::Greater => Some(0),
            std::cmp::Ordering::Less => None,
        }
    }
}

impl PlayerHistory {
    /// A player's history before the game's first turn.
    pub fn before_any_turn() -> Self {
        PlayerHistory { most_recent_turn: None, previous_turn: None, this_draw_step: 0, turn_tally: TurnTally::NONE }
    }

    /// The turn this player most recently began, which CR 302.6 measures
    /// against (`GameState::most_recent_turn_began`).
    pub fn most_recent_turn(&self) -> Option<u32> {
        self.most_recent_turn
    }

    pub(crate) fn record_turn_began(&mut self, turn: u32) {
        if self.most_recent_turn != Some(turn) {
            self.previous_turn = self.most_recent_turn;
            self.most_recent_turn = Some(turn);
        }
    }

    /// "Your last turn" on turn `now`: this player's most recent turn to
    /// have ended, so on their own turn the one before. `None` before any
    /// has.
    pub fn your_last_turn(&self, now: u32) -> Option<u32> {
        if self.most_recent_turn == Some(now) { self.previous_turn } else { self.most_recent_turn }
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
    pub resolutions_this_turn: IdMap<AbilityIdentity, u32>,
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
    pub fn count(&self, player: PlayerId, fact: TurnFact, turns: RangeInclusive<u32>) -> u64 {
        self.log.count(fact, turns, &|counted| counted == player, &mut |_| true, &|_| {})
    }

    /// `player`'s count of `fact` on turn `turn`, which a record's place in
    /// its turn starts from: off the player's tally where it keeps the fact,
    /// so a turn of thousands of life changes does not read the turn once for
    /// each.
    pub fn count_in_turn(&self, player: PlayerId, fact: TurnFact, turn: u32) -> u64 {
        self.players
            .get(player)
            .and_then(|history| history.turn_tally.count(fact, turn))
            .unwrap_or_else(|| self.count(player, fact, turn..=turn))
    }

    /// Write `row`, the newest event, into the log and each player's tally:
    /// the one way a row is written.
    pub(crate) fn push_row(&mut self, row: HistoryRow) {
        for (player, history) in self.players.iter_mut().enumerate() {
            history.turn_tally.add(&row, player);
        }
        self.log.push(row);
    }

    /// `history` totalled over `span` for `you` on turn `now`: the rows of the
    /// players it names, narrowed by its filter over each row's frame, which
    /// `filter_matches` asks in its reader's context, a layer walk's or a
    /// resolution's. Notes what it read into `note`.
    pub(crate) fn total(
        &self,
        history: &HistoryCount,
        span: HistorySpan,
        you: PlayerId,
        now: u32,
        filter_matches: &mut dyn FnMut(&ObjectFilter, ObjectInformation<'_>) -> bool,
        note: &dyn Fn(GatedWalkInput),
    ) -> u64 {
        let counts_for = |player: PlayerId| history.players.contains(you, player);
        let mut of = |row: &HistoryRow| match &history.of {
            None => true,
            Some(filter) => row.frame_matches(|object| filter_matches(filter, object)),
        };
        self.log.count(history.fact, self.turns_of(span, you, now), &counts_for, &mut of, note)
    }

    /// The turns `span` names for `you` on turn `now`.
    pub fn turns_of(&self, span: HistorySpan, you: PlayerId, now: u32) -> RangeInclusive<u32> {
        match span {
            HistorySpan::ThisTurn => now..=now,
            HistorySpan::LastTurn => now.saturating_sub(1)..=now.saturating_sub(1),
            HistorySpan::SinceYourLastTurn => self.since_your_last_turn(you, now),
            HistorySpan::ThisGame => 1..=now,
        }
    }

    /// The turns "since your last turn" spans for `you` on turn `now`: every
    /// turn after your most recent one to have ended, this one included, so
    /// the whole game before you have had one.
    pub fn since_your_last_turn(&self, you: PlayerId, now: u32) -> RangeInclusive<u32> {
        match self.players.get(you).and_then(|yours| yours.your_last_turn(now)) {
            Some(last) => last + 1..=now,
            None => 1..=now,
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
    use crate::events::event::EventId;
    use crate::state::history_log::{RowFact, RowFrame};

    const DRAWN: TurnFact = TurnFact::CardsDrawn;

    /// `count` cards drawn by `player` on `turn`, as a scenario states them.
    fn drew(history: &mut GameHistory, player: PlayerId, turn: u32, count: u64) {
        let fact = RowFact::Stated { fact: DRAWN, count };
        let seq = EventId(history.log.len());
        history.push_row(HistoryRow { seq, turn, combat: 0, player: Some(player), object: None, frame: RowFrame::None, fact });
    }

    /// A turn's count is its rows, whoever's turn it was; a turn with none
    /// reads zero, and "this game" is every turn's.
    #[test]
    fn a_turns_count_is_its_rows() {
        let mut history = GameHistory::new(2);
        drew(&mut history, 0, 3, 2);
        drew(&mut history, 0, 5, 1);
        assert_eq!(history.count(0, DRAWN, 3..=3), 2);
        assert_eq!(history.count(0, DRAWN, 4..=4), 0);
        assert_eq!(history.count(0, DRAWN, 5..=5), 1);
        assert_eq!(history.count(0, DRAWN, 1..=5), 3);
        assert_eq!(history.count(1, DRAWN, 1..=5), 0, "the rows are player 0's");
    }

    /// A record's place starts from the tally, which counts what the log's
    /// count counts: a stated count, each row after it, each player's own,
    /// and a new turn starting over. A turn before the tally's is the log's.
    #[test]
    fn the_turn_tally_counts_what_the_log_counts() {
        let mut history = GameHistory::new(2);
        drew(&mut history, 0, 3, 2);
        drew(&mut history, 1, 3, 1);
        for gained in [3, 1] {
            let seq = EventId(history.log.len());
            let fact = RowFact::GainedLife(gained);
            history.push_row(HistoryRow { seq, turn: 3, combat: 0, player: Some(1), object: None, frame: RowFrame::None, fact });
        }
        for (player, fact) in [(0, DRAWN), (1, DRAWN), (0, TurnFact::LifeGainEvents), (1, TurnFact::LifeGainEvents)] {
            assert_eq!(history.count_in_turn(player, fact, 3), history.count(player, fact, 3..=3), "{fact:?} for player {player}");
        }
        assert_eq!(history.count_in_turn(1, TurnFact::LifeGainEvents, 3), 2, "two gains, whatever their amounts");

        drew(&mut history, 0, 4, 5);
        assert_eq!(history.count_in_turn(0, DRAWN, 4), 5);
        assert_eq!(history.count_in_turn(1, DRAWN, 4), 0, "the new turn starts over");
        assert_eq!(history.count_in_turn(0, DRAWN, 3), 2, "an earlier turn, read off the log");
        assert_eq!(history.count_in_turn(0, DRAWN, 6), 0, "a turn with no rows yet");
    }

    /// "Since your last turn" for player 0, read off player 1's counts: the
    /// span starts when player 0's turn ends, so a count on player 0's own
    /// turn is before it and a count on any later turn is inside it.
    #[test]
    fn since_your_last_turn_starts_when_your_turn_ends() {
        let mut history = GameHistory::new(2);
        history.players[0].record_turn_began(1);
        drew(&mut history, 1, 1, 5);
        history.players[1].record_turn_began(2);
        drew(&mut history, 1, 2, 1);
        let since = |history: &GameHistory, now| history.count(1, DRAWN, history.since_your_last_turn(0, now));
        assert_eq!(since(&history, 2), 1);

        history.players[0].record_turn_began(3);
        drew(&mut history, 1, 3, 1);
        assert_eq!(since(&history, 3), 2, "on your own turn, your last turn is the one before");
    }

    /// An extra turn: player 0 takes turns 1 and 2, and on turn 2 "your last
    /// turn" is turn 1, just ended.
    #[test]
    fn after_an_extra_turn_your_last_turn_is_the_one_just_ended() {
        let mut history = GameHistory::new(2);
        history.players[0].record_turn_began(1);
        drew(&mut history, 0, 1, 4);
        history.players[0].record_turn_began(2);
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
