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
    // ORDER-FREE: `assert_shortcut_agrees` sorts what it is handed.
    let differ = |one: &Frames, other: &Frames| one.iter().filter(|(id, frame)| other.get(id) != Some(frame)).map(|(id, _)| id.to_string()).collect::<Vec<_>>();
    assert_shortcut_agrees(site, question, differ(gated, everything), differ(everything, gated));
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
