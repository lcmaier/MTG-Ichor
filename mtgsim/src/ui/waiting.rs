//! What the game is holding for a later event or turn: each delayed
//! triggered ability waiting for its event (CR 603.7), each extra turn
//! waiting in the queue (CR 500.7) and each "until" return waiting for its
//! event (CR 610.3), as plain data for a client to lay out.
//! Not a triggered ability waiting to be put on the stack, which goes there
//! the next time a player would receive priority (CR 117.2a) and is the
//! stack's to show.
//!
//! Read off the state, so a row is gone the moment the thing it describes
//! is: a delayed trigger that has triggered once or whose duration or turn
//! has passed, an extra turn taken, skipped (CR 614.10a) or lost with the
//! player who would have taken it (CR 800.4), a return made.

use crate::state::game_state::GameState;
use crate::types::ids::{DelayedTriggerId, ExtraTurnId, PlayerId, UntilReturnId};
use crate::types::triggers::{DelayedDuration, DelayedTrigger, TriggerSubject, TriggerTurn};
use crate::ui::display::{named, object_label};

/// The delayed triggers, the extra turns and the returns waiting, each kind
/// in its own order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Waiting {
    /// The delayed triggered abilities, in the order they were created.
    pub delayed_triggers: Vec<WaitingTrigger>,
    /// The extra turns, in the order they will be taken: the most recently
    /// created first (CR 500.7).
    pub extra_turns: Vec<WaitingTurn>,
    /// CR 610.3's "until" returns, in the order they were made.
    pub until_returns: Vec<WaitingReturn>,
}

/// One "until" return, waiting for the event it names (CR 610.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitingReturn {
    /// The number its making's log line gave it.
    pub id: UntilReturnId,
    /// The object its event is about, named as it is: "this" or the target
    /// it named. `None` for an event about no object.
    pub watched: Option<String>,
    /// What returns, each named as it is.
    pub returns: Vec<String>,
    /// The exile's controller.
    pub controller: PlayerId,
}

/// One delayed triggered ability, waiting for its event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitingTrigger {
    /// The number its creation's log line gave it.
    pub id: DelayedTriggerId,
    /// The words its card prints for it.
    pub text: &'static str,
    /// Its source (CR 603.7d–g), named as it is, or as it last existed once
    /// it has left (CR 113.7a).
    pub source: String,
    pub controller: PlayerId,
    pub turn: TriggersIn,
    /// Once, or each time this turn.
    pub duration: DelayedDuration,
}

/// The turn a delayed trigger can trigger in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggersIn {
    AnyTurn,
    /// A turn after this numbered one: "the next turn's".
    TurnAfter(u32),
    /// One extra turn (CR 500.7), `player`'s: the turn in progress, or one
    /// still in the queue.
    ExtraTurn { player: PlayerId, in_progress: bool },
}

/// One extra turn in the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaitingTurn {
    pub id: ExtraTurnId,
    pub player: PlayerId,
}

/// What is waiting in `game` now.
pub fn what_is_waiting(game: &GameState) -> Waiting {
    Waiting {
        delayed_triggers: game.delayed_triggers.iter().filter_map(|delayed| waiting_trigger(game, delayed)).collect(),
        extra_turns: game
            .turn_queue
            .iter()
            .rev()
            .filter(|turn| game.in_game(turn.player))
            .map(|turn| WaitingTurn { id: turn.id, player: turn.player })
            .collect(),
        until_returns: game
            .until_returns
            .iter()
            .map(|until| WaitingReturn {
                id: until.id,
                watched: match until.until.subject() {
                    Some(TriggerSubject::ThisObject) => Some(named(game, until.source.object.id)),
                    Some(TriggerSubject::Referred) => until.referred.first().map(|r| named(game, r.object.id)),
                    _ => None,
                },
                returns: until.returns.iter().map(|(object, _)| named(game, object.id)).collect(),
                controller: until.controller,
            })
            .collect(),
    }
}

/// `None` for one bound to an extra turn no longer coming, which the next
/// turn's beginning removes.
fn waiting_trigger(game: &GameState, delayed: &DelayedTrigger) -> Option<WaitingTrigger> {
    let turn = match delayed.turn {
        TriggerTurn::Any => TriggersIn::AnyTurn,
        TriggerTurn::LaterThan(n) => TriggersIn::TurnAfter(n),
        TriggerTurn::Extra(id) if game.extra_turn == Some(id) => {
            TriggersIn::ExtraTurn { player: game.active_player, in_progress: true }
        }
        TriggerTurn::Extra(id) => {
            let queued = game.turn_queue.iter().find(|turn| turn.id == id && game.in_game(turn.player))?;
            TriggersIn::ExtraTurn { player: queued.player, in_progress: false }
        }
    };
    let id = delayed.source.id;
    let source = match &delayed.source_frame {
        Some(frame) if game.object_ref(id) != Some(delayed.source) => object_label(game, id, &frame.name),
        _ => named(game, id),
    };
    Some(WaitingTrigger {
        id: delayed.id,
        text: delayed.rules_text.words,
        source,
        controller: delayed.controller,
        turn,
        duration: delayed.duration,
    })
}
