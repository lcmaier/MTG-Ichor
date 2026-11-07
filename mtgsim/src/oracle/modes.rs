//! CR 700.2's offer: the sets of modes a modal ability's chooser may
//! announce, each one whole choice (`choices-architecture.md`, decision 2's
//! B). The announcement lists them and castability asks for the first, so an
//! offer and a cast cannot disagree (CR 700.2a, 602.2b, 603.3c).

use crate::engine::layers::condition::{settled_holds, ConditionReferents};
use crate::oracle::mana_helpers::every_instance_has_a_choice;
use crate::state::game_state::GameState;
use crate::types::effects::{ModalCount, ModalNode, ModeIndex};
use crate::types::ids::{ObjectId, PlayerId};

/// Every set of modes `node` lets its chooser announce now, in printed order
/// within a set, the smaller sets first. A mode whose instances of "target"
/// cannot all be chosen is in none (CR 700.2a), and a mode is never in a set
/// twice (CR 700.2d's default). Read as `every_instance_has_a_choice` reads a
/// spell's targets: `player` is the one announcing, whose "you" the targets
/// and the count's condition read; `this_object` the object whose text it is
/// (CR 113.7a); `named` what a trigger's event names; and `x_value` the X
/// announced before the modes, which a count of X reads.
pub(crate) fn legal_mode_sets(
    game: &GameState,
    node: &ModalNode,
    player: PlayerId,
    this_object: ObjectId,
    named: ConditionReferents<'_>,
    x_value: Option<u64>,
) -> Vec<Vec<ModeIndex>> {
    let mut sets = Vec::new();
    walk_legal_sets(game, node, player, this_object, named, x_value, &mut |set| {
        sets.push(set.to_vec());
        KeepGoing::Yes
    });
    sets
}

/// Whether [`legal_mode_sets`] lists any: the same walk, stopped at the first.
pub(crate) fn has_a_legal_mode_set(
    game: &GameState,
    node: &ModalNode,
    player: PlayerId,
    this_object: ObjectId,
    named: ConditionReferents<'_>,
    x_value: Option<u64>,
) -> bool {
    let mut found = false;
    walk_legal_sets(game, node, player, this_object, named, x_value, &mut |_| {
        found = true;
        KeepGoing::No
    });
    found
}

/// Whether a walk over the sets of modes goes on to the next set.
#[derive(Clone, Copy, PartialEq, Eq)]
enum KeepGoing {
    Yes,
    No,
}

/// Hands `visit` each legal set in [`legal_mode_sets`]' order, stopping when
/// it says to. The two callers differ only in `visit`: the list keeps every
/// set, castability stops at the first.
fn walk_legal_sets(
    game: &GameState,
    node: &ModalNode,
    player: PlayerId,
    this_object: ObjectId,
    named: ConditionReferents<'_>,
    x_value: Option<u64>,
    visit: &mut impl FnMut(&[ModeIndex]) -> KeepGoing,
) {
    let Some((fewest, most)) = set_sizes(game, &node.count, player, this_object, named, x_value) else { return };
    // A size past the modes left is no set: a mode is chosen once.
    if fewest > node.modes.len() {
        return;
    }
    let legal: Vec<ModeIndex> = node
        .indexed_modes()
        .filter(|(_, mode)| every_instance_has_a_choice(game, &mode.instances, player, this_object, named))
        .map(|(place, _)| place)
        .collect();
    for size in fewest..=most.min(legal.len()) {
        if for_each_combination(&legal, size, visit) == KeepGoing::No {
            return;
        }
    }
}

/// The fewest and most modes `count` allows, read now: `None` for a count of
/// X with no X announced. The most may pass the modes printed, which the walk
/// caps.
fn set_sizes(
    game: &GameState,
    count: &ModalCount,
    player: PlayerId,
    this_object: ObjectId,
    named: ConditionReferents<'_>,
    x_value: Option<u64>,
) -> Option<(usize, usize)> {
    let size = |n: u32| usize::try_from(n).unwrap_or(usize::MAX);
    match count {
        ModalCount::Exactly(n) => Some((size(*n), size(*n))),
        ModalCount::Range { fewest, most } => Some((size(*fewest), size(*most))),
        ModalCount::X => {
            let x = usize::try_from(x_value?).unwrap_or(usize::MAX);
            Some((x, x))
        }
        // Read once as the modes are chosen, against the board then (Will of
        // the Abzan's ruling).
        ModalCount::RaisedIf { usual, raised, condition } => {
            let holds = settled_holds(condition, game, this_object, Some(player), named);
            set_sizes(game, if holds { raised } else { usual }, player, this_object, named, x_value)
        }
    }
}

/// Hands `visit` every way to pick `size` of the modes in `legal`, each in
/// printed order: from modes 1, 2 and 4, two at a time, that is {1, 2},
/// {1, 4}, {2, 4}, in that order. Returns `KeepGoing::No` when `visit` stopped
/// it, so the walk does not go on to the next size.
fn for_each_combination(legal: &[ModeIndex], size: usize, visit: &mut impl FnMut(&[ModeIndex]) -> KeepGoing) -> KeepGoing {
    if size > legal.len() {
        return KeepGoing::Yes;
    }
    let mut picks: Vec<usize> = (0..size).collect();
    let mut set: Vec<ModeIndex> = Vec::with_capacity(size);
    loop {
        set.clear();
        set.extend(picks.iter().map(|&at| legal[at]));
        if visit(&set) == KeepGoing::No {
            return KeepGoing::No;
        }
        // `picks` are positions in `legal`, rising. The next combination
        // moves the rightmost pick that has room one step right, and packs
        // every pick after it in behind it; none has room after the last.
        let Some(moves) = (0..size).rev().find(|&i| picks[i] < legal.len() - size + i) else { return KeepGoing::Yes };
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
        for_each_combination(&modes(legal), size, &mut |set| {
            out.push(set.to_vec());
            KeepGoing::Yes
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
