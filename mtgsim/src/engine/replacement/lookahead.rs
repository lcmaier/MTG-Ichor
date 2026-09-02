//! The CR 614.12 frame of the object a proposed event is about.
//!
//! Computed at most once per pipeline iteration, and only when a
//! filter-scoped `AffectedSet` actually asks — `SourceOnly` and `Fixed` match
//! by id and never need it. CR 616.1(1)'s "replacement effects that have
//! already modified how it enters" is why the frame is per *iteration*: the
//! loop rewrites the proposal between iterations, and the next gather reads
//! the rewritten `EnterMods` and controller.
//!
//! Two events have a frame. `EnterBattlefield` is the obvious one. The
//! `ZoneChange` onto the battlefield that precedes it has one too, for
//! CR 614.17d: "Lands can't enter the battlefield" (Worms of the Earth) has to
//! stop the *move*, since a stopped entry would leave the card in the
//! battlefield zone with no permanent (`GameState::propose_entry`) — and the
//! rule says the "can't" is decided against the characteristics the permanent
//! would have, so the frame is built from what the entry proposal would carry
//! (CR 110.2b's default controller, the rules' own `EnterMods`).
//!
//! **Only `affected` reads it.** An `EventPattern`'s object filter — Grafdigger's
//! Cage's "creature cards in graveyards" — is a question about the card where
//! it is, and its ruling says so: "look at the card as it exists in your
//! graveyard". `pattern_watches` keeps reading the source zone.

use std::cell::OnceCell;

use crate::engine::actions::GameAction;
use crate::engine::layers::compute_as_entering;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::EnterMods;
use crate::types::zones::Zone;

/// What the frame would be computed from, when the event has one.
#[derive(Debug)]
enum Subject {
    /// The entry proposal itself, or an entering object a synthetic query is
    /// about: everything is known.
    Proposed { object: ObjectId, controller: PlayerId, mods: EnterMods },
    /// The zone change ahead of an entry: the proposal that will follow, as far
    /// as it is knowable now. Derived lazily, because the rules' own
    /// `EnterMods` are a frame question themselves.
    Pending { object: ObjectId },
}

#[derive(Debug)]
pub(crate) struct EntryFrame<'g> {
    game: &'g GameState,
    subject: Option<Subject>,
    frame: OnceCell<Option<EffectiveCharacteristics>>,
}

impl<'g> EntryFrame<'g> {
    /// The frame for a proposed event — empty for any event that is not an
    /// entry or the zone change ahead of one.
    pub(crate) fn new(game: &'g GameState, action: &GameAction) -> Self {
        let subject = match action {
            GameAction::EnterBattlefield { object, controller, mods, .. } => {
                Some(Subject::Proposed {
                    object: *object,
                    controller: *controller,
                    mods: mods.clone(),
                })
            }
            GameAction::ZoneChange { object, to: Zone::Battlefield, .. } => {
                Some(Subject::Pending { object: *object })
            }
            _ => None,
        };
        EntryFrame { game, subject, frame: OnceCell::new() }
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
            subject: Some(Subject::Proposed { object, controller, mods: mods.clone() }),
            frame: OnceCell::new(),
        }
    }

    /// The frame, if `id` is the object this event is about and the event has
    /// one. Computed on first use.
    pub(crate) fn frame_of(&self, id: ObjectId) -> Option<&EffectiveCharacteristics> {
        let object = match &self.subject {
            Some(Subject::Proposed { object, .. }) | Some(Subject::Pending { object }) => *object,
            None => return None,
        };
        if object != id {
            return None;
        }
        self.frame
            .get_or_init(|| match &self.subject {
                Some(Subject::Proposed { object, controller, mods }) => {
                    compute_as_entering(self.game, *object, *controller, mods)
                }
                Some(Subject::Pending { object }) => {
                    let controller = self.game.default_enter_controller(*object).ok()?;
                    let mods = self.game.default_enter_mods(*object, controller);
                    compute_as_entering(self.game, *object, controller, &mods)
                }
                None => None,
            })
            .as_ref()
    }
}
