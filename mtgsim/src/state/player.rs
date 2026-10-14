use std::collections::BTreeMap;

use crate::state::history::PlayerHistory;
use crate::types::effects::CounterType;
use crate::types::ids::{IdMap, ObjectId, PlayerId};
use crate::types::mana::ManaPool;
use crate::types::zones::DrawOrdinals;

/// Per-player state in the game.
///
/// Player-owned zones (hand, library, graveyard) store ObjectIds — the actual
/// GameObjects live in GameState's central object store.
#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: PlayerId,
    pub life_total: i64,
    pub mana_pool: ManaPool,

    // Player-owned zones (ordered collections of object IDs)
    pub library: Vec<ObjectId>,
    pub hand: Vec<ObjectId>,
    pub graveyard: Vec<ObjectId>,

    // Turn-specific state
    pub max_hand_size: i32,
    pub lands_per_turn: u32,
    pub lands_played_this_turn: u32,

    /// The counters this player has (CR 122.1's "placed on an object or
    /// player"): kind → count. Poison (CR 122.1f, read by CR 704.5c), energy
    /// (CR 107.14), experience and rad are all kinds here, sharing
    /// `CounterType` with a permanent's counters because CR 701.34a's
    /// proliferate sweeps both in one pass. A `BTreeMap` so a walk over it is
    /// in enum order, process-independent. No timestamps: CR 613.7c
    /// timestamps a player's counters too, but no layer computes a player.
    /// Written through [`Self::add_counters`] / [`Self::remove_counters`] by
    /// `perform_action`'s counter arms and by nothing else.
    pub counters: BTreeMap<CounterType, u32>,
    pub commander_damage_taken: IdMap<ObjectId, u32>,

    // SBA flags — these are ONLY for state-based action checks (rule 704).
    // General per-turn tracking (e.g. "cast a spell this turn") should live
    // in a separate TurnTracker struct when needed.
    pub has_drawn_from_empty_library: bool,

    /// This player's history (`triggers-architecture.md` §3.10): what "this
    /// turn", "last turn", "since your last turn" and "this game" read, bounded
    /// by the table. Advanced by the dispatcher, record by record, and by
    /// nothing else.
    pub history: PlayerHistory,

    /// This player's draws, counted by the draw's performer.
    pub draws: DrawCounts,
}

/// One player's draws this turn and in their draw step in progress, counted as
/// each is performed (CR 121.2). Not the history's count: that one advances as
/// a window is dispatched, and a doubled draw's second card is proposed inside
/// the window that holds its first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawCounts {
    /// The turn `this_turn` counts.
    turn: u32,
    this_turn: u64,
    /// Read only while this player's draw step is in progress.
    this_draw_step: u64,
}

impl DrawCounts {
    /// No card drawn.
    pub const NONE: DrawCounts = DrawCounts { turn: 0, this_turn: 0, this_draw_step: 0 };

    /// The counts a scenario states: `this_turn` draws on turn `turn`,
    /// `this_draw_step` of them in the draw step in progress.
    pub fn stated(turn: u32, this_turn: u64, this_draw_step: u64) -> Self {
        DrawCounts { turn, this_turn, this_draw_step }
    }

    /// Where this player's next draw would fall on turn `now`, `in_draw_step`
    /// saying whether it is their own draw step.
    pub fn next(&self, now: u32, in_draw_step: bool) -> DrawOrdinals {
        if now == 0 {
            return DrawOrdinals::BEFORE_ANY_TURN;
        }
        let this_turn = if self.turn == now { self.this_turn } else { 0 };
        DrawOrdinals {
            in_turn: Some(this_turn + 1),
            in_draw_step: in_draw_step.then_some(self.this_draw_step + 1),
        }
    }

    /// Count a draw, and return where it fell.
    pub(crate) fn drew(&mut self, now: u32, in_draw_step: bool) -> DrawOrdinals {
        let ordinals = self.next(now, in_draw_step);
        if let Some(n) = ordinals.in_turn {
            self.turn = now;
            self.this_turn = n;
        }
        if let Some(n) = ordinals.in_draw_step {
            self.this_draw_step = n;
        }
        ordinals
    }

    /// CR 504: this player's draw step has begun.
    pub(crate) fn draw_step_began(&mut self) {
        self.this_draw_step = 0;
    }

    /// The draws counted in the draw step in progress, if it is this player's.
    pub fn this_draw_step(&self) -> u64 {
        self.this_draw_step
    }
}

impl PlayerState {
    pub fn new(id: PlayerId, starting_life: i64) -> Self {
        PlayerState {
            id,
            life_total: starting_life,
            mana_pool: ManaPool::new(),
            library: Vec::new(),
            hand: Vec::new(),
            graveyard: Vec::new(),
            max_hand_size: 7,
            lands_per_turn: 1,
            lands_played_this_turn: 0,
            counters: BTreeMap::new(),
            commander_damage_taken: IdMap::default(),
            has_drawn_from_empty_library: false,
            history: PlayerHistory::before_any_turn(),
            draws: DrawCounts::NONE,
        }
    }

    /// How many counters of `kind` this player has (0 if none).
    pub fn counter_count(&self, kind: CounterType) -> u32 {
        self.counters.get(&kind).copied().unwrap_or(0)
    }

    /// Give this player `n` counters of `kind`. No rule caps a count, but the
    /// type does, so a count past `u32::MAX` stays there.
    pub fn add_counters(&mut self, kind: CounterType, n: u32) {
        if n == 0 {
            return;
        }
        let count = self.counters.entry(kind).or_insert(0);
        *count = count.saturating_add(n);
    }

    /// Take up to `n` counters of `kind` off this player; the number actually
    /// taken — CR 701.2's "as much as it can", `PermanentState::remove_counters`'
    /// mirror, and for an *effect's* instruction only: a cost paid in
    /// counters ("you can't pay more energy counters than you have", CR 118.3)
    /// is validated in full before it proposes a removal (`engine::costs`).
    /// A kind that reaches zero leaves the map, so "has a counter"
    /// (CR 701.34a's proliferate) is `counters` being non-empty.
    pub fn remove_counters(&mut self, kind: CounterType, n: u32) -> u32 {
        let Some(count) = self.counters.get_mut(&kind) else {
            return 0;
        };
        let removed = (*count).min(n);
        *count -= removed;
        if *count == 0 {
            self.counters.remove(&kind);
        }
        removed
    }

    pub fn can_play_land(&self) -> bool {
        self.lands_played_this_turn < self.lands_per_turn
    }

    pub fn reset_lands_played(&mut self) {
        self.lands_played_this_turn = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = PlayerState::new(0, 20);
        assert_eq!(player.id, 0);
        assert_eq!(player.life_total, 20);
        assert_eq!(player.max_hand_size, 7);
        assert_eq!(player.lands_per_turn, 1);
        assert!(player.library.is_empty());
        assert!(player.hand.is_empty());
        assert!(player.graveyard.is_empty());
    }

    #[test]
    fn test_player_has_no_counters_by_default() {
        let player = PlayerState::new(0, 20);
        assert_eq!(player.counter_count(CounterType::Poison), 0);
        assert!(player.counters.is_empty());
    }

    #[test]
    fn test_player_counters_add_and_remove_as_much_as_possible() {
        let mut player = PlayerState::new(0, 20);
        player.add_counters(CounterType::Energy, 2);
        player.add_counters(CounterType::Energy, 3);
        assert_eq!(player.counter_count(CounterType::Energy), 5);
        assert_eq!(player.remove_counters(CounterType::Energy, 7), 5, "CR 701.2");
        assert!(player.counters.is_empty(), "a kind at zero leaves the map");
        assert_eq!(player.remove_counters(CounterType::Poison, 1), 0);
    }

    /// A scenario's words add, so two can say more than a count holds.
    #[test]
    fn test_player_counters_past_what_a_count_holds_stay_at_the_most() {
        let mut player = PlayerState::new(0, 20);
        player.add_counters(CounterType::Poison, u32::MAX);
        player.add_counters(CounterType::Poison, 1);
        assert_eq!(player.counter_count(CounterType::Poison), u32::MAX);
    }

    #[test]
    fn test_player_commander_damage_default() {
        let player = PlayerState::new(0, 20);
        assert!(player.commander_damage_taken.is_empty());
    }

    #[test]
    fn test_land_play_tracking() {
        let mut player = PlayerState::new(0, 20);
        assert!(player.can_play_land());

        player.lands_played_this_turn = 1;
        assert!(!player.can_play_land());

        player.reset_lands_played();
        assert!(player.can_play_land());
    }

    /// The turn's count starts over on a new turn, the draw step's when the
    /// step begins, and CR 103.5's opening hand is in neither.
    #[test]
    fn test_draw_counts_by_turn_and_draw_step() {
        let mut draws = DrawCounts::NONE;
        assert_eq!(draws.drew(0, false), DrawOrdinals::BEFORE_ANY_TURN);
        assert_eq!(draws.drew(1, false), DrawOrdinals { in_turn: Some(1), in_draw_step: None });
        draws.draw_step_began();
        assert_eq!(draws.drew(1, true), DrawOrdinals { in_turn: Some(2), in_draw_step: Some(1) });
        assert_eq!(draws.drew(1, true), DrawOrdinals { in_turn: Some(3), in_draw_step: Some(2) });
        assert_eq!(draws.next(2, false), DrawOrdinals { in_turn: Some(1), in_draw_step: None });
        draws.draw_step_began();
        assert_eq!(draws.drew(2, true), DrawOrdinals { in_turn: Some(1), in_draw_step: Some(1) });
    }
}
