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
//! What belongs to no one player is [`GameHistory`]'s: each ability's "this
//! turn" facts, and what each creature did in combat.

use std::sync::Arc;

use crate::state::battlefield::AttackTarget;
use crate::state::game_state::AbilityIdentity;
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
    /// What "since your last turn" subtracts: every player's `this_game`, by
    /// `PlayerId`, as it stood when this player's most recent turn ended,
    /// taken as the next turn began. The span crosses the other seats' turns,
    /// which no row keeps, so it is read as a total now less this one: after
    /// player 0's turn 5, "since your last turn" on turn 9 is turns 6 to 9.
    /// One row per player, because "an opponent lost life since your last
    /// turn" reads the other players' counts. Empty until this player's first
    /// turn has ended, which reads as zeros.
    at_your_last_turn: Vec<TurnSummary>,
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
            at_your_last_turn: Vec::new(),
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

    /// The cards drawn in this player's draw step in progress.
    pub fn this_draw_step(&self) -> u64 {
        self.this_draw_step
    }

    /// This player's draw step has begun (CR 504), or a scenario states how
    /// many cards were drawn in it so far.
    pub(crate) fn draw_step_drawn(&mut self, n: u64) {
        self.this_draw_step = n;
    }

    /// Count a card drawn in this player's draw step, and return its place.
    pub(crate) fn drew_in_draw_step(&mut self) -> u64 {
        self.this_draw_step = self.this_draw_step.saturating_add(1);
        self.this_draw_step
    }
}

/// What the game remembers that belongs to no one player.
#[derive(Debug, Clone, Default)]
pub struct GameHistory {
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
    /// What each creature did in this round's combats.
    pub combat: CombatHistory,
}

impl GameHistory {
    /// A turn has begun. "Each turn" is the game's turn, not a controller's
    /// (§3.5), so the abilities' facts start over; the combat's rows keep the
    /// turns from `keep_from` on.
    pub(crate) fn turn_began(&mut self, keep_from: u32) {
        self.action_taken_this_turn.clear();
        self.triggered_this_turn.clear();
        self.resolutions_this_turn.clear();
        self.combat.turn_began(keep_from);
    }
}

/// What each creature did in combat: a row for each attacker declared (CR
/// 508.1) and for each creature removed from combat (CR 506.4, 511.3).
///
/// Kept for a round, from the earliest turn any player's "your last turn"
/// can still name (Giant Turtle's "if it attacked during your last turn").
/// The rows are shared, so a clone copies a pointer, and a write copies them
/// only while a clone still shares them.
#[derive(Debug, Clone, Default)]
pub struct CombatHistory {
    /// The combat phases begun this turn, which CR 500.8 can add to.
    combats_this_turn: u32,
    rows: Arc<Vec<CombatRow>>,
}

/// One thing a creature did in one combat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatRow {
    /// The existence that did it (CR 400.7).
    pub object: ObjectRef,
    pub turn: u32,
    /// Which of the turn's combats, from 1.
    pub combat: u32,
    pub fact: CombatFact,
}

/// What a [`CombatRow`] says the creature did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatFact {
    /// CR 508.1: declared as an attacker, attacking this. A creature put onto
    /// the battlefield attacking never "attacked" (CR 508.4) and has none.
    Attacked(AttackTarget),
    /// CR 506.4, 511.3: removed from combat, attacking this as it was, or
    /// nothing.
    Removed { attacking: Option<AttackTarget> },
}

impl CombatHistory {
    /// A combat phase of this turn has begun.
    pub(crate) fn combat_began(&mut self) {
        self.combats_this_turn += 1;
    }

    /// `object` did `fact` in turn `turn`'s latest combat.
    pub(crate) fn record(&mut self, object: ObjectRef, turn: u32, fact: CombatFact) {
        let row = CombatRow { object, turn, combat: self.combats_this_turn, fact };
        Arc::make_mut(&mut self.rows).push(row);
    }

    /// A turn has begun, and nothing reads a row from before `keep_from`.
    fn turn_began(&mut self, keep_from: u32) {
        self.combats_this_turn = 0;
        if self.rows.iter().any(|row| row.turn < keep_from) {
            Arc::make_mut(&mut self.rows).retain(|row| row.turn >= keep_from);
        }
    }

    /// Every row, oldest first.
    pub fn rows(&self) -> &[CombatRow] {
        &self.rows
    }

    /// Was `object` declared as an attacker in turn `turn`, the turn in
    /// progress? A creature put onto the battlefield attacking never was (CR
    /// 508.4).
    pub fn attacked_this_turn(&self, object: ObjectRef, turn: u32) -> bool {
        self.rows.iter().any(|row| row.object == object && row.turn == turn && matches!(row.fact, CombatFact::Attacked(_)))
    }

    /// What `object` was attacking as it was last removed from combat in the
    /// latest combat of turn `turn`, the turn in progress (CR 508.5's second
    /// sentence): `None` when it was not removed in that combat, or was
    /// attacking nothing.
    pub fn attacking_when_removed(&self, object: ObjectRef, turn: u32) -> Option<AttackTarget> {
        let removed = self.rows.iter().rev().find(|row| {
            row.object == object
                && row.turn == turn
                && row.combat == self.combats_this_turn
                && matches!(row.fact, CombatFact::Removed { .. })
        })?;
        match removed.fact {
            CombatFact::Removed { attacking } => attacking,
            CombatFact::Attacked(_) => None,
        }
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

    fn existence(raw: u64, epoch: u64) -> ObjectRef {
        ObjectRef { id: crate::types::ids::ObjectId::from_trace(raw), zone_change_epoch: epoch }
    }

    /// Two combats in one turn: "defending player" reads the latest combat's
    /// removal only, and "attacked this turn" each combat's declaration. The
    /// same object after a zone change is a new one (CR 400.7) and did
    /// neither.
    #[test]
    fn the_rows_tell_combats_and_existences_apart() {
        let mut combat = CombatHistory::default();
        let (bears, after_a_flicker) = (existence(7, 1), existence(7, 2));
        let player_1 = AttackTarget::Player(1);
        combat.combat_began();
        combat.record(bears, 3, CombatFact::Attacked(player_1));
        combat.record(bears, 3, CombatFact::Removed { attacking: Some(player_1) });
        assert_eq!(combat.attacking_when_removed(bears, 3), Some(player_1));

        combat.combat_began();
        assert_eq!(combat.attacking_when_removed(bears, 3), None, "not removed in the second combat");
        assert!(combat.attacked_this_turn(bears, 3), "it attacked in the first");
        assert!(!combat.attacked_this_turn(after_a_flicker, 3));
        assert!(!combat.attacked_this_turn(bears, 4));
    }

    /// The rows keep a round: from the earliest turn any player's "your last
    /// turn" can name. On turn 3 player 0's last turn is turn 1, so its row
    /// stays; on turn 4 both players' last turns are 2 and 3, so it goes.
    #[test]
    fn the_rows_keep_a_round() {
        let mut game = GameState::new(2, 20);
        game.history.combat.record(existence(7, 1), 1, CombatFact::Attacked(AttackTarget::Player(1)));
        let turns = |game: &GameState| game.history.combat.rows().iter().map(|row| row.turn).collect::<Vec<_>>();
        game.begin_turn(2, 1);
        game.begin_turn(3, 0);
        assert_eq!(turns(&game), [1], "player 0's last turn, on its own turn");
        game.begin_turn(4, 1);
        assert!(turns(&game).is_empty());
    }

    /// A clone shares the rows until one of the two writes, so a fork that
    /// plays through a combat copies them once.
    #[test]
    fn a_clone_shares_the_rows_until_it_writes() {
        let mut first = CombatHistory::default();
        first.record(existence(7, 1), 1, CombatFact::Attacked(AttackTarget::Player(1)));
        let second = first.clone();
        assert!(Arc::ptr_eq(&first.rows, &second.rows));
        first.record(existence(8, 1), 1, CombatFact::Attacked(AttackTarget::Player(1)));
        assert!(!Arc::ptr_eq(&first.rows, &second.rows));
        assert_eq!((first.rows().len(), second.rows().len()), (2, 1));
    }

    /// The draw step's count is not a turn row: it starts over when a draw
    /// step begins, and a turn can have two (CR 500.8).
    #[test]
    fn the_draw_steps_count_starts_over_with_each_draw_step() {
        let mut history = PlayerHistory::before_any_turn();
        assert_eq!((history.drew_in_draw_step(), history.drew_in_draw_step()), (1, 2));
        history.draw_step_drawn(0);
        assert_eq!(history.drew_in_draw_step(), 1, "a second draw step in the same turn");
    }
}
