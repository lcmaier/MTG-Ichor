//! The CR 614.12 frame of the object a proposed event is about.
//!
//! Computed at most once per pipeline iteration, and only when something asks
//! — a filter-scoped `AffectedSet`, or `gather`'s source 1a reading the
//! entering permanent's own abilities. `SourceOnly` and `Fixed` match by id
//! and never need it. CR 616.1(1)'s "replacement effects that have already
//! modified how it enters" is why the frame is per *iteration*: the loop
//! rewrites the proposal between iterations, and the next gather reads the
//! rewritten `EnterMods` and controller.
//!
//! One event has a frame: `EnterBattlefield`, which is also the zone change
//! onto the battlefield (RC-4b), so a CR 614.17d "can't enter" — Worms of the
//! Earth — is decided against the same frame, before anything has moved.
//!
//! **Only `affected` and source 1a read it.** An `EventPattern`'s object
//! filter — Grafdigger's Cage's "creature cards in graveyards" — is a question
//! about the card where it is, and its ruling says so: "look at the card as
//! it exists in your graveyard". `pattern_watches` keeps reading the source
//! zone.

use std::cell::OnceCell;

use crate::engine::actions::GameAction;
use crate::engine::layers::compute_as_entering;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::EnterMods;

/// What the frame is computed from: the entry proposal itself, or an entering
/// object a synthetic query is about. Everything is known.
#[derive(Debug)]
struct FrameBasis {
    object: ObjectId,
    controller: PlayerId,
    mods: EnterMods,
}

#[derive(Debug)]
pub(crate) struct EntryFrame<'g> {
    game: &'g GameState,
    basis: Option<FrameBasis>,
    frame: OnceCell<Option<EffectiveCharacteristics>>,
}

impl<'g> EntryFrame<'g> {
    /// The frame for a proposed event — empty for any event that is not an
    /// entry.
    pub(crate) fn new(game: &'g GameState, action: &GameAction) -> Self {
        let basis = match action {
            GameAction::EnterBattlefield { object, controller, mods, .. } => Some(FrameBasis {
                object: *object,
                controller: *controller,
                mods: mods.clone(),
            }),
            _ => None,
        };
        EntryFrame { game, basis, frame: OnceCell::new() }
    }

    /// The frame for an entering object a *synthetic* event is about — the
    /// `AddCounters` a CR 614.17d "can't have counters put on it" is asked of,
    /// whose action names no `EnterMods`.
    pub(crate) fn for_entering(
        game: &'g GameState,
        object: ObjectId,
        controller: PlayerId,
        mods: &EnterMods,
    ) -> Self {
        EntryFrame {
            game,
            basis: Some(FrameBasis { object, controller, mods: mods.clone() }),
            frame: OnceCell::new(),
        }
    }

    /// The frame, if `id` is the object this event is about and the event has
    /// one. Computed on first use.
    pub(crate) fn frame_of(&self, id: ObjectId) -> Option<&EffectiveCharacteristics> {
        let basis = self.basis.as_ref()?;
        if basis.object != id {
            return None;
        }
        self.frame
            .get_or_init(|| {
                compute_as_entering(self.game, basis.object, basis.controller, &basis.mods)
            })
            .as_ref()
    }
}
