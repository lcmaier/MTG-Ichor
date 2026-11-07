//! CR 700.2's offer: the sets of modes a modal ability's chooser may
//! announce, each one whole choice (`choices-architecture.md`, decision 2's
//! B). The announcement lists them and castability asks for the first, so an
//! offer and a cast cannot disagree (CR 700.2a, 602.2b, 603.3c).

use crate::engine::layers::condition::{settled_holds, ConditionReferents};
use crate::oracle::mana_helpers::every_instance_has_a_choice;
use crate::state::game_state::GameState;
use crate::types::effects::{ModalCount, ModalNode, ModeIndex};
use crate::types::ids::{ObjectId, PlayerId};

/// What an offer of modes is read against: the board, the player announcing
/// (whose "you" the targets and the count's condition read), the object whose
/// text it is (CR 113.7a), what a trigger's targets name, and the X announced
/// before the modes, which a count of X reads.
#[derive(Clone, Copy)]
pub(crate) struct ModeAsk<'a> {
    pub game: &'a GameState,
    pub player: PlayerId,
    pub this_object: ObjectId,
    pub named: ConditionReferents<'a>,
    pub x_value: Option<u64>,
}

/// Every set of modes `node` lets its chooser announce now, in printed order
/// within a set, the smaller sets first. A mode whose instances of "target"
/// cannot all be chosen is in none (CR 700.2a), and a mode is never in a set
/// twice (CR 700.2d's default).
pub(crate) fn legal_mode_sets(ask: ModeAsk<'_>, node: &ModalNode) -> Vec<Vec<ModeIndex>> {
    let mut sets = Vec::new();
    walk_legal_sets(ask, node, &mut |set| {
        sets.push(set.to_vec());
        true
    });
    sets
}

/// Whether [`legal_mode_sets`] lists any: the same walk, stopped at the first.
pub(crate) fn has_a_legal_mode_set(ask: ModeAsk<'_>, node: &ModalNode) -> bool {
    let mut found = false;
    walk_legal_sets(ask, node, &mut |_| {
        found = true;
        false
    });
    found
}

/// Each legal set in [`legal_mode_sets`]' order, until `f` returns `false`.
fn walk_legal_sets(ask: ModeAsk<'_>, node: &ModalNode, f: &mut impl FnMut(&[ModeIndex]) -> bool) {
    let Some((fewest, most)) = set_sizes(ask, &node.count) else { return };
    // A size past the modes left is no set: a mode is chosen once.
    if fewest > node.modes.len() {
        return;
    }
    let legal: Vec<ModeIndex> = node
        .indexed_modes()
        .filter(|(_, mode)| every_instance_has_a_choice(ask.game, &mode.instances, ask.player, ask.this_object, ask.named))
        .map(|(place, _)| place)
        .collect();
    for size in fewest..=most.min(legal.len()) {
        if !for_each_set_of(&legal, size, f) {
            return;
        }
    }
}

/// The fewest and most modes `count` allows, read now: `None` for a count of
/// X with no X announced. The most may pass the modes printed, which the walk
/// caps.
fn set_sizes(ask: ModeAsk<'_>, count: &ModalCount) -> Option<(usize, usize)> {
    let size = |n: u32| usize::try_from(n).unwrap_or(usize::MAX);
    match count {
        ModalCount::Exactly(n) => Some((size(*n), size(*n))),
        ModalCount::Range { fewest, most } => Some((size(*fewest), size(*most))),
        ModalCount::X => {
            let x = usize::try_from(ask.x_value?).unwrap_or(usize::MAX);
            Some((x, x))
        }
        // Read once as the modes are chosen, against the board then (Will of
        // the Abzan's ruling).
        ModalCount::RaisedIf { usual, raised, condition } => {
            let holds = settled_holds(condition, ask.game, ask.this_object, Some(ask.player), ask.named);
            set_sizes(ask, if holds { raised } else { usual })
        }
    }
}

/// Each `size`-mode subset of `legal`, its modes in `legal`'s order, the
/// subsets in lexicographic order, until `f` returns `false`; whether it ran
/// to the end.
fn for_each_set_of(legal: &[ModeIndex], size: usize, f: &mut impl FnMut(&[ModeIndex]) -> bool) -> bool {
    if size > legal.len() {
        return true;
    }
    let mut picks: Vec<usize> = (0..size).collect();
    let mut set: Vec<ModeIndex> = Vec::with_capacity(size);
    loop {
        set.clear();
        set.extend(picks.iter().map(|&at| legal[at]));
        if !f(&set) {
            return false;
        }
        // The rightmost pick that can still move right, and every pick after
        // it packed in behind it.
        let Some(moves) = (0..size).rev().find(|&i| picks[i] < legal.len() - size + i) else { return true };
        picks[moves] += 1;
        for i in moves + 1..size {
            picks[i] = picks[i - 1] + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modes(places: &[u8]) -> Vec<ModeIndex> {
        places.iter().map(|&place| ModeIndex::printed(place)).collect()
    }

    fn sets_of(legal: &[u8], size: usize) -> Vec<Vec<ModeIndex>> {
        let mut out = Vec::new();
        for_each_set_of(&modes(legal), size, &mut |set| {
            out.push(set.to_vec());
            true
        });
        out
    }

    #[test]
    fn subsets_come_in_lexicographic_order_without_a_dropped_mode() {
        assert_eq!(sets_of(&[1, 2, 4], 2), vec![modes(&[1, 2]), modes(&[1, 4]), modes(&[2, 4])]);
        assert_eq!(sets_of(&[1, 2, 3], 3), vec![modes(&[1, 2, 3])]);
        assert_eq!(sets_of(&[1, 2], 0), vec![modes(&[])]);
        assert!(sets_of(&[1], 2).is_empty());
    }

    #[test]
    fn five_modes_one_or_more_are_thirty_one_sets() {
        let total: usize = (1..=5).map(|size| sets_of(&[1, 2, 3, 4, 5], size).len()).sum();
        assert_eq!(total, 31, "Casualties of War's count, by the design's table");
    }
}
