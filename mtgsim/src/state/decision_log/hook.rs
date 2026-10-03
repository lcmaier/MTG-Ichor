//! The hook a client attaches: every decision the game makes, handed to
//! the client's writer as it is made (`super`'s doc says where it is
//! called and why it observes and never participates).

use std::fmt;
use std::sync::Arc;

use crate::state::game_state::GameState;
use crate::types::ids::PlayerId;
use crate::ui::choice_types::{ChoiceKind, ChoiceOption};

/// An answer, in the shape of the primitive that asked.
#[derive(Clone, Copy, Debug)]
pub enum LoggedAnswer<'a> {
    Picks(&'a [usize]),
    Number(u64),
    Allocation(&'a [u64]),
    Order(&'a [usize]),
}

/// One line of the log.
#[derive(Clone, Copy, Debug)]
pub struct LoggedDecision<'a> {
    pub player: PlayerId,
    pub kind: &'a ChoiceKind,
    /// What the question offered: a `pick_n`'s options, an `allocate`'s
    /// buckets, a `choose_ordering`'s items, and none for a `pick_number`.
    /// The log writes an answer as what it chose among them, never where
    /// (`ChoiceOption::as_logged`).
    pub options: &'a [ChoiceOption],
    pub answer: LoggedAnswer<'a>,
    /// The question had one legal answer: a priority point where the player
    /// could only pass, whether the engine passed or the seat stopped for it.
    /// A fact about the question, so it is the same whatever answered; a
    /// reader that wants only the choices skips these lines. A question with
    /// several answers that all give the same game is not forced: where the
    /// engine can show that, as for two triggers of one ability, it does not
    /// ask (`triggers::placement`), so there is no line, and where it cannot,
    /// the choice is the player's (CR 603.3b) and logged as one.
    pub forced: bool,
}

/// What a game logs its decisions to, if anything: [`GameState::log_decisions`].
pub struct DecisionLogHandle(Option<Arc<DecisionWriter>>);

impl DecisionLogHandle {
    /// No log attached, which is every game until a client attaches one.
    pub const NONE: DecisionLogHandle = DecisionLogHandle(None);
}

type DecisionWriter = dyn Fn(&GameState, &LoggedDecision) + Send + Sync;

/// `GameState` derives `Clone`, and a clone is a fork: a search clones the
/// game at a decision to try another answer. A derived clone would share this
/// writer, so the fork's answers would land in the game's log among its own
/// lines; a fork writes nothing until it is given a log of its own.
impl Clone for DecisionLogHandle {
    fn clone(&self) -> Self {
        DecisionLogHandle::NONE
    }
}

impl fmt::Debug for DecisionLogHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(if self.0.is_some() { "DecisionLogHandle(attached)" } else { "DecisionLogHandle(none)" })
    }
}

impl GameState {
    /// Hand every decision from here on to `write`, in the order the game
    /// makes them.
    pub fn log_decisions(&mut self, write: impl Fn(&GameState, &LoggedDecision) + Send + Sync + 'static) {
        self.decision_log = DecisionLogHandle(Some(Arc::new(write)));
    }

    pub(crate) fn log_decision(&self, decision: LoggedDecision) {
        if let Some(write) = &self.decision_log.0 {
            write(self, &decision);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::test_support::setup_two_player_game;

    /// A clone is a fork, and a fork writes nothing to the game's log.
    #[test]
    fn a_fork_logs_nothing() {
        let mut game = setup_two_player_game();
        let written = Arc::new(Mutex::new(0));
        let count = Arc::clone(&written);
        game.log_decisions(move |_, _| *count.lock().unwrap() += 1);
        let options = [ChoiceOption::Action(crate::ui::decision::PriorityAction::Pass)];
        let pass = LoggedDecision { player: 0, kind: &ChoiceKind::PriorityAction, options: &options, answer: LoggedAnswer::Picks(&[0]), forced: true };

        game.clone().log_decision(pass);
        assert_eq!(*written.lock().unwrap(), 0, "the fork wrote to the game's log");
        game.log_decision(pass);
        assert_eq!(*written.lock().unwrap(), 1);
    }
}
