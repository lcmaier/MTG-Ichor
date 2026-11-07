//! The engine's shortcuts checked against themselves. In a game that turns
//! the audit on (`fuzz_games --audit`), each site that skips work on a
//! premise answers again with its shortcut off, and the two answers must
//! agree (`plans/elisions.md` §4). The dispatcher's audit is the first and
//! the largest (`engine::triggers::audit`, TR-1b). The replacement,
//! restriction and cost gates, the control sweep and the castability fold use
//! what this module holds: the switch, the observers an audit's read leaves as
//! it found them, the objects that could carry an ability of a kind, and the
//! panic.
//!
//! **Its reads leave no trace.** The layer memo and the diagnostics are put
//! back after each read and the trace is paused through it, so an audited
//! game's counters and trace are an unaudited one's.

use std::collections::HashMap;
use std::sync::Arc;

use crate::engine::layers::compute::{compute_characteristics, no_row_reaches};
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::state::game_state::GameState;
use crate::types::effects::Effect;
use crate::types::ids::ObjectId;
use crate::types::zones::Zone;

/// How a gated sweep runs: skipping what its gate says cannot answer, as the
/// game does, or asking everything, as the audit does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Sweep {
    Gated,
    Everything,
}

impl GameState {
    /// Audit every shortcut for the rest of this game (`fuzz_games --audit`):
    /// the dispatcher's, and each gate, sweep and fold `plans/elisions.md` §4
    /// gives an audit. The hooks are inside the engine, so the switch is too; a
    /// second call changes nothing.
    pub fn enable_audit(&mut self) {
        if self.dispatch_audit.is_none() {
            self.dispatch_audit = Some(Box::default());
        }
    }

    /// Whether this game audits its shortcuts.
    pub(crate) fn audit_on(&self) -> bool {
        self.dispatch_audit.is_some()
    }

    /// `read`, with the game's observers left as it found them: the layer
    /// memo and the diagnostics put back afterward, and the trace paused while
    /// it runs. What an audit asks, and an observer such as a record's names,
    /// then moves no counter and writes no trace.
    pub(crate) fn unobserved<T>(&self, read: impl FnOnce() -> T) -> T {
        let memo = self.layer_memo.clone();
        let diagnostics = self.diagnostics.clone();
        let paused = self.trace.as_ref().map(|trace| trace.pause());
        let answer = read();
        self.layer_memo.restore(memo);
        self.diagnostics.rewind_to(&diagnostics);
        if let (Some(trace), Some(was)) = (&self.trace, paused) {
            trace.resume(was);
        }
        answer
    }

    /// Every object in every zone that could carry an ability whose printed
    /// effect is of `kind`: all of them on the battlefield and the spells on
    /// the stack, and elsewhere every object a continuous effect could reach
    /// or that printed one. An object no row reaches has its printed
    /// abilities and nothing else. An ability on the stack is an object (CR
    /// 113.1c) without its source's abilities.
    pub(crate) fn objects_that_may_carry(&self, kind: impl Fn(&Effect) -> bool) -> Vec<ObjectId> {
        // NO ROW REACHES: asked only of an object no row reaches, whose
        // abilities are its printed ones.
        let printed = |id: &ObjectId| self.objects.get(id).is_some_and(|o| o.card_data.abilities.iter().any(|a| kind(&a.effect)));
        let mut ids = self.battlefield_ids_ordered();
        ids.extend(self.stack.iter().copied().filter(|&id| self.is_spell_on_stack(id)));
        for zone in [Zone::Exile, Zone::Command, Zone::Graveyard, Zone::Hand, Zone::Library] {
            ids.extend(self.zone_ids_ordered(zone).into_iter().filter(|id| !no_row_reaches(self, *id) || printed(id)));
        }
        ids
    }
}

/// The most things an audit tries in every order of, 120 orders, each a clone
/// of the game played forward. Every order of n things is n! of them, and a
/// fuzz run cannot pay that past here: 6 is 720 clones and 8 is 40,320, at
/// every payment or batch that size.
const MOST_AUDITED_IN_EVERY_ORDER: usize = 5;

/// The orders of `n` things an audit of an order tries, as places in the
/// order it was handed: every one up to [`MOST_AUDITED_IN_EVERY_ORDER`], and
/// past it that order and its reverse.
pub(crate) fn orders_to_audit(n: usize) -> Vec<Vec<usize>> {
    if n > MOST_AUDITED_IN_EVERY_ORDER {
        return vec![(0..n).collect(), (0..n).rev().collect()];
    }
    let mut orders = vec![Vec::new()];
    for thing in 0..n {
        orders = orders
            .into_iter()
            .flat_map(|order: Vec<usize>| {
                (0..=order.len()).map(move |at| {
                    let mut longer = order.clone();
                    longer.insert(at, thing);
                    longer
                })
            })
            .collect();
    }
    orders
}

/// What objects are, by id, whole: what an audit that replays an order
/// compares. Keyed, since which came first is no difference.
pub(crate) type Frames = HashMap<ObjectId, Arc<EffectiveCharacteristics>>;

/// Every permanent as it now is.
pub(crate) fn permanents_as_they_are(game: &GameState) -> Frames {
    game.battlefield_ids_ordered().into_iter().filter_map(|id| compute_characteristics(game, id).map(|frame| (id, frame))).collect()
}

/// Panic unless each object is one thing in `gated` and in `everything`,
/// [`assert_shortcut_agrees`]' check for what objects are, which reads every
/// characteristic an object has.
pub(crate) fn assert_frames_agree(site: &str, question: impl FnOnce() -> String, gated: &Frames, everything: &Frames) {
    let differs = |one: &Frames, other: &Frames, id: &ObjectId| one.get(id) != other.get(id);
    // ORDER-FREE: `assert_shortcut_agrees` sorts what it is handed.
    let in_one_only_or_unlike: Vec<String> =
        gated.keys().chain(everything.keys()).filter(|id| differs(gated, everything, id)).map(ObjectId::to_string).collect();
    assert_shortcut_agrees(site, question, in_one_only_or_unlike, Vec::new());
}

/// Panic unless a shortcut's answer is the one with the shortcut off: at
/// `site`, asked `question`, the game's read found `gated` and the audit's
/// `everything`, each a list of what it found, compared as sets.
pub(crate) fn assert_shortcut_agrees(site: &str, question: impl FnOnce() -> String, mut gated: Vec<String>, mut everything: Vec<String>) {
    gated.sort();
    everything.sort();
    assert!(
        gated == everything,
        "{site}: its shortcut answered {question} with {gated:?}, and with the shortcut off {everything:?} \
         (plans/elisions.md §4: a premise the shortcut rests on does not hold on this board)",
        question = question(),
    );
}

#[cfg(test)]
mod tests {
    use crate::engine::actions::ActionContext;
    use crate::test_support::{install_trace, put_in_graveyard, setup_two_player_game, RecordingDecisionProvider};
    use crate::types::zones::{Zone, ZoneChangeAct};

    /// A fork an unobserved read makes and plays on, as the payment-order
    /// audit does, writes no `fork` record and none of what it plays, and the
    /// game it was made from goes on writing as its own branch.
    #[test]
    fn an_unobserved_fork_writes_nothing() {
        let mut game = setup_two_player_game();
        let bears = put_in_graveyard(&mut game, crate::cards::printings::grizzly_bears(), 0);
        let trace = install_trace(&mut game, "an unobserved fork");
        let before = trace.lines().len();
        let dp = RecordingDecisionProvider::picking(0);
        game.unobserved(|| {
            let mut fork = game.clone();
            fork.change_zone(bears, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&dp)).unwrap();
        });
        assert_eq!(trace.lines().len(), before, "{:#?}", &trace.lines()[before..]);

        game.change_zone(bears, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&dp)).unwrap();
        let after = trace.records();
        assert!(after.len() > before && after[before..].iter().all(|record| record.branch == 0), "the game's own branch");
    }
}
