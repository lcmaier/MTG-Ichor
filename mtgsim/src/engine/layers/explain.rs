//! What the layers did to one object: the pass run again with a recorder
//! watching it, so a client can say how the object got its characteristics
//! (`setup-architecture.md` §7c).
//!
//! **The recorder is the pass, not a second walk.** [`explain`] runs the walk
//! `compute_characteristics` would run for the object (`board::run_pass` for
//! a member, `compute::compute_non_member` for a card no pass holds) and hands
//! it a [`Recorder`] where every other caller hands `None`. Each application
//! is recorded as the pass applies it, in the pass's order, so the
//! explanation cannot disagree with the answer, and a debug build holds the
//! two equal. Nothing is memoized and nothing is traced, and a CR 613.8a
//! hypothetical (`board::depends_on`'s journaled application) records
//! nothing.
//!
//! The words are `ui::why`'s: this module says what happened, as facts.

use crate::engine::layers::board::{compute_board, compute_board_recorded, pass_membership, Board, PassMembership};
use crate::engine::layers::compute::{compute_non_member_recorded, LAYER_ORDER};
use crate::engine::layers::types::{EffectId, EffectOrigin, EffectiveCharacteristics, Layer, Timestamp};
use crate::state::game_state::GameState;
use crate::types::effects::CounterType;
use crate::types::ids::{AbilityId, ObjectId};
use crate::types::keywords::KeywordFlag;

/// What the layers made of one object, and how.
#[derive(Debug, Clone)]
pub struct LayerExplanation {
    /// The object as the layers begin from it: printed, under the controller
    /// CR 108.4 gives it.
    pub seed: EffectiveCharacteristics,
    /// Each application that affected the object, and each one whose set
    /// could have named it and did not, in the order the pass applied them.
    pub steps: Vec<LayerStep>,
    /// `compute_characteristics`'s answer for the object.
    pub result: EffectiveCharacteristics,
}

/// One application, as the object met it.
#[derive(Debug, Clone)]
pub struct LayerStep {
    pub layer: Layer,
    pub by: AppliedBy,
    /// CR 613.7's timestamp: an effect's, a counter's, or the object's own.
    /// CR 306.5b's intrinsic ability has none.
    pub timestamp: Option<Timestamp>,
    pub result: StepResult,
    /// Every object it affected in this layer, the explained one among them
    /// or not: what an effect that missed the object did apply to.
    pub affected: Vec<ObjectId>,
    /// CR 613.8: the objects whose applications this one waited for, when
    /// it depended on them and so applied after them, out of timestamp order.
    pub waited_for: Vec<ObjectId>,
}

/// What applied: an effect, or something of the object's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppliedBy {
    /// One continuous effect's rows in this layer.
    Effect { source: ObjectId, origin: EffectOrigin, effect: EffectId },
    /// The object's own characteristic-defining ability (CR 604.3).
    Cda { ability: AbilityId },
    /// A keyword counter on it (CR 122.1b).
    KeywordCounter { keyword: KeywordFlag },
    /// Its +1/+1 or -1/-1 counters (CR 122.1a).
    PtCounters { kind: CounterType, count: u32 },
    /// The copy it entered as (CR 614.1c, 707.5).
    EnteredAsCopy,
    /// An edit it entered with (CR 614.1c).
    EnteredWith,
    /// A planeswalker's intrinsic loyalty ability (CR 306.5b), given at the
    /// end of layer 4.
    IntrinsicLoyalty,
}

/// What an application did to the object.
#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    /// It applied: the object's frame before it and after it.
    Applied { before: EffectiveCharacteristics, after: EffectiveCharacteristics },
    /// Its set could name the object, and its filter does not match it.
    NotMatched,
    /// Its source no longer has the ability that generates it, so the effect
    /// does not exist (CR 604.2): a stripped CDA, or a static ability taken
    /// away earlier.
    Gone,
    /// CR 613.6: the effect began in an earlier layer, and the set it began
    /// with does not include the object.
    LockedOut,
}

/// What `explain` keeps of one walk: every application in the order applied,
/// the objects each affected, and what each did to the watched object.
///
/// With no object watched it keeps the order and the affected objects alone,
/// which is the layer tests' view of CR 613.8's sequence.
pub(super) struct Recorder {
    watched: Option<ObjectId>,
    seed: Option<EffectiveCharacteristics>,
    steps: Vec<RecordedStep>,
}

/// One application as the recorder saw it.
pub(super) struct RecordedStep {
    pub(super) layer: Layer,
    pub(super) by: AppliedBy,
    pub(super) timestamp: Option<Timestamp>,
    /// Every object it affected.
    pub(super) affected: Vec<ObjectId>,
    /// What it did to the watched object, when its set could name it.
    pub(super) watched: Option<StepResult>,
    pub(super) waited_for: Vec<ObjectId>,
}

impl Recorder {
    /// A recorder keeping each application's order and affected objects, and
    /// no frame.
    #[cfg(test)]
    pub(super) fn order_only() -> Recorder {
        Recorder { watched: None, seed: None, steps: Vec::new() }
    }

    fn watching(id: ObjectId) -> Recorder {
        Recorder { watched: Some(id), seed: None, steps: Vec::new() }
    }

    pub(super) fn watched(&self) -> Option<ObjectId> {
        self.watched
    }

    /// The watched object's frame as the layers begin from it.
    pub(super) fn seeded(&mut self, frame: Option<&EffectiveCharacteristics>) {
        self.seed = frame.cloned();
    }

    pub(super) fn push(&mut self, step: RecordedStep) {
        self.steps.push(step);
    }

    #[cfg(test)]
    pub(super) fn steps(&self) -> &[RecordedStep] {
        &self.steps
    }

    fn into_explanation(self, result: EffectiveCharacteristics) -> Option<LayerExplanation> {
        let steps = self
            .steps
            .into_iter()
            .filter_map(|step| {
                step.watched.map(|result| LayerStep {
                    layer: step.layer,
                    by: step.by,
                    timestamp: step.timestamp,
                    result,
                    affected: step.affected,
                    waited_for: step.waited_for,
                })
            })
            .collect();
        Some(LayerExplanation { seed: self.seed?, steps, result })
    }
}

/// What the layers did to `id`: the walk `compute_characteristics` runs for
/// it, recorded. `None` when the object does not exist.
///
/// A card the pass leaves out is walked with a pass's notes, as the memo's
/// own walk takes them, from a pass run here rather than from the memo, so
/// the memo is not written.
pub fn explain(game: &GameState, id: ObjectId) -> Option<LayerExplanation> {
    game.objects.get(&id)?;
    let mut recorder = Recorder::watching(id);
    let ceiling = LAYER_ORDER.len();
    let result = match pass_membership(game, id) {
        PassMembership::Member => compute_board_recorded(game, None, None, ceiling, Some(&mut recorder)).take(id),
        PassMembership::ZoneOnly => compute_board_recorded(game, None, Some(id), ceiling, Some(&mut recorder)).take(id),
        PassMembership::LeftOut => {
            let (_, notes) = compute_board(game, None).into_frames_and_notes();
            let notes = notes.unwrap_or_default();
            compute_non_member_recorded(game, &Board::settled(), id, ceiling, &notes, Some(&mut recorder))
        }
        PassMembership::NonMember => {
            compute_non_member_recorded(game, &Board::settled(), id, ceiling, &[], Some(&mut recorder))
        }
    }?;
    debug_assert_eq!(
        Some(&result),
        crate::engine::layers::compute::compute_characteristics(game, id).as_deref(),
        "the explanation of {id} reached another answer than the walk's"
    );
    recorder.into_explanation(result)
}
