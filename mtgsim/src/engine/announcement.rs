//! CR 601.2b–c's record of a stack object's modes and targets
//! ([`Announcement`]), and what each scope holds after CR 608.2b's re-check
//! ([`SurvivingTargets`]). Apart from `targeting` because a modal effect's
//! record is its chosen modes first, and a chosen mode may announce no target
//! at all; `targeting` owns the instances of "target" each scope holds.

use crate::engine::targeting::{ChosenTargets, TargetInstance};
use crate::types::effects::ModeIndex;

/// CR 601.2b–c's record of a stack object's modes and targets. A modal node is
/// all of its ability, so an effect's instances of "target" are either its
/// own or its chosen modes', never both: each chosen mode holds its own (CR
/// 700.2c), so no mode's targets are found by counting, and a later writer
/// that changes a target (CR 115.7, 707.10c) cannot move one into another
/// mode's place.
#[derive(Debug, Clone, PartialEq)]
pub enum Announcement {
    /// An effect with no modes: every instance of "target", in printed order
    /// (an Aura's is its enchant ability's, CR 303.4a).
    Targets(Vec<TargetInstance>),
    /// A modal effect: the chosen modes in printed order, a repeated mode's
    /// copies adjacent (CR 700.2d), each with its own instances. None chosen
    /// is a choice of no modes.
    Modes(Vec<ChosenMode>),
}

/// One mode CR 601.2b chose, and the instances of "target" its choice
/// announced (CR 700.2c), counted from its own first: a mode is its own scope
/// for "another target" and a back-reference.
#[derive(Debug, Clone, PartialEq)]
pub struct ChosenMode {
    pub mode: ModeIndex,
    pub targets: Vec<TargetInstance>,
}

impl Announcement {
    /// Nothing announced: an effect with no modes and no instance of "target".
    pub const NONE: Announcement = Announcement::Targets(Vec::new());

    /// Every instance announced, in the order announced: the effect's, or each
    /// chosen mode's after the one before. What a reader that does not ask
    /// which mode reads: the targeted records, a label, a source of damage.
    pub fn targets(&self) -> impl Iterator<Item = &TargetInstance> + '_ {
        let (own, modes): (&[TargetInstance], &[ChosenMode]) = match self {
            Announcement::Targets(own) => (own, &[]),
            Announcement::Modes(modes) => (&[], modes),
        };
        own.iter().chain(modes.iter().flat_map(|chosen| chosen.targets.iter()))
    }

    /// The chosen modes by printed place, in the order they resolve; none
    /// for an effect with no modes.
    pub fn chosen_modes(&self) -> impl Iterator<Item = ModeIndex> + '_ {
        let modes: &[ChosenMode] = match self {
            Announcement::Targets(_) => &[],
            Announcement::Modes(modes) => modes,
        };
        modes.iter().map(|chosen| chosen.mode)
    }
}

/// What each instance of "target" holds after CR 608.2b's re-check, scope by
/// scope as [`Announcement`] holds them.
#[derive(Debug, Clone, PartialEq)]
pub enum SurvivingTargets<'a> {
    /// An effect with no modes: its own instances'.
    Targets(ChosenTargets),
    /// A modal effect: each chosen mode with its own, in the order announced.
    Modes(Vec<SurvivingMode<'a>>),
}

/// One chosen mode as it begins to resolve: the mode as CR 601.2b–c announced
/// it, and what its instances hold after CR 608.2b's re-check, kept together
/// so neither is found by the other's place.
#[derive(Debug, Clone, PartialEq)]
pub struct SurvivingMode<'a> {
    pub chosen: &'a ChosenMode,
    pub survivors: ChosenTargets,
}
