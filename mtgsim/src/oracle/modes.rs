//! CR 700.2's offer: the sets of modes a modal ability's chooser may
//! announce, each one whole choice (`choices-architecture.md`, decision 2's
//! B). The announcement lists them and castability asks for the first, so an
//! offer and a cast cannot disagree (CR 700.2a, 602.2b, 603.3c).

use crate::engine::layers::condition::{settled_holds, ConditionReferents};
use crate::engine::object_filter::EarlierTargets;
use crate::engine::targeting::FilterIdentity;
use crate::oracle::legality::enumerate_legal_selections_excluding;
use crate::oracle::mana_helpers::every_instance_has_a_choice;
use crate::state::game_state::{AbilityIdentity, GameState};
use crate::types::effects::{EffectRecipient, ModalCount, ModalNode, ModeIndex, SelectionFilter};
use crate::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};

/// Every set of modes `node` lets its chooser announce now, in printed order
/// within a set, a repeated mode's copies adjacent (CR 700.2d), the smaller
/// sets first. A mode whose instances of "target" cannot all be chosen is in
/// none (CR 700.2a); a mode is in a set once unless the node says it may
/// repeat; a set whose pawprints pass a pawprint count's budget is none
/// (700.2i); and under "each mode must target a different player" a set
/// whose modes cannot each take a player of their own is none. Read as
/// `every_instance_has_a_choice` reads a spell's targets: `player` is the one
/// announcing, whose "you" the targets and the count's condition read;
/// `this_object` the object whose text it is (CR 113.7a); `named` what a
/// trigger's event names; `x_value` the X announced before the modes, which a
/// count of X reads; and `identity` the ability announcing, whose modes
/// chosen before "that hasn't been chosen" leaves out (none for a spell).
pub(crate) fn legal_mode_sets(
    game: &GameState,
    node: &ModalNode,
    player: PlayerId,
    this_object: ObjectId,
    named: ConditionReferents<'_>,
    x_value: Option<u64>,
    identity: Option<AbilityIdentity>,
) -> Vec<Vec<ModeIndex>> {
    let mut sets = Vec::new();
    walk_legal_sets(game, node, player, this_object, named, x_value, identity, &mut |set| {
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
    identity: Option<AbilityIdentity>,
) -> bool {
    let mut found = false;
    walk_legal_sets(game, node, player, this_object, named, x_value, identity, &mut |_| {
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
    identity: Option<AbilityIdentity>,
    visit: &mut impl FnMut(&[ModeIndex]) -> KeepGoing,
) {
    let Some(count) = count_now(game, &node.count, player, this_object, named, x_value) else { return };
    let chosen_before = match (node.rules.not_chosen_before, identity) {
        (Some(scope), Some(identity)) => game.modes_chosen_before(identity, scope),
        _ => &[],
    };
    let legal: Vec<ModeIndex> = node
        .indexed_modes()
        .filter(|(place, mode)| {
            !chosen_before.contains(place) && every_instance_has_a_choice(game, &mode.instances, player, this_object, named)
        })
        .map(|(place, _)| place)
        .collect();
    let pawprints = |set: &[ModeIndex]| set.iter().filter_map(|&mode| node.mode(mode)).map(|mode| mode.pawprints).sum::<u32>();
    let mut admitted = |set: &[ModeIndex]| {
        let within_budget = match count {
            CountNow::Pawprints(budget) => pawprints(set) <= budget,
            CountNow::Sizes(..) => true,
        };
        if !within_budget
            || (node.rules.each_mode_targets_a_different_player && !players_can_differ(game, node, set, player, this_object, named))
        {
            return KeepGoing::Yes;
        }
        visit(set)
    };
    for size in count.sizes() {
        // A mode chosen once fills at most the modes there are.
        if !node.rules.may_repeat && size > legal.len() {
            break;
        }
        let walked = if node.rules.may_repeat {
            for_each_multiset(&legal, size, &mut admitted)
        } else {
            for_each_combination(&legal, size, &mut admitted)
        };
        if walked == KeepGoing::No {
            return;
        }
    }
}

/// A count read now: the sizes of set it allows, or a pawprint budget.
#[derive(Clone, Copy)]
enum CountNow {
    /// Every size from the first through the second, and the third if any:
    /// "you may choose two" is 0 and 2.
    Sizes(usize, usize, Option<usize>),
    /// CR 700.2i: any size, its pawprints within the budget. Every printed
    /// mode has a pawprint, so no set is larger than the budget.
    Pawprints(u32),
}

impl CountNow {
    fn sizes(self) -> impl Iterator<Item = usize> {
        let (fewest, most, also) = match self {
            CountNow::Sizes(fewest, most, also) => (fewest, most, also),
            CountNow::Pawprints(budget) => (0, usize::try_from(budget).unwrap_or(usize::MAX), None),
        };
        (fewest..=most).chain(also)
    }
}

/// What `count` allows, read now: `None` for a count of X with no X
/// announced. A most may pass the modes printed, which the walk caps where a
/// mode is chosen once.
fn count_now(
    game: &GameState,
    count: &ModalCount,
    player: PlayerId,
    this_object: ObjectId,
    named: ConditionReferents<'_>,
    x_value: Option<u64>,
) -> Option<CountNow> {
    let size = |n: u32| usize::try_from(n).unwrap_or(usize::MAX);
    match count {
        ModalCount::Exactly(n) => Some(CountNow::Sizes(size(*n), size(*n), None)),
        ModalCount::Range { fewest, most } => Some(CountNow::Sizes(size(*fewest), size(*most), None)),
        ModalCount::X => {
            let x = usize::try_from(x_value?).unwrap_or(usize::MAX);
            Some(CountNow::Sizes(x, x, None))
        }
        ModalCount::NoneOr(n) => Some(CountNow::Sizes(0, 0, Some(size(*n)).filter(|&n| n > 0))),
        ModalCount::Pawprints(budget) => Some(CountNow::Pawprints(*budget)),
        // Read once as the modes are chosen, against the board then (Will of
        // the Abzan's ruling).
        ModalCount::RaisedIf { usual, raised, condition } => {
            let holds = settled_holds(condition, game, this_object, Some(player), named);
            count_now(game, if holds { raised } else { usual }, player, this_object, named, x_value)
        }
    }
}

/// The most modes a count of X can take now, where its modes are each
/// chosen once: the X a cast may announce for it (item 412). `None` for any
/// other count, or one whose modes may repeat, which every X fills.
pub(crate) fn most_modes_for_x(
    game: &GameState,
    node: &ModalNode,
    player: PlayerId,
    this_object: ObjectId,
    named: ConditionReferents<'_>,
) -> Option<u64> {
    if node.count != ModalCount::X || node.rules.may_repeat {
        return None;
    }
    let legal = node.modes.iter().filter(|mode| every_instance_has_a_choice(game, &mode.instances, player, this_object, named)).count();
    u64::try_from(legal).ok()
}

/// Whether each mode of `set` can target a player no other mode of it does:
/// "each mode must target a different player". Each mode's player-targeting
/// instances take the first players they may that no earlier mode took, in
/// printed order. Greedy, which is exact for the shapes that print, every mode
/// of a card naming the same players ("target player", or "target opponent");
/// two modes with different player filters that each could take the other's
/// player are `codebase-state.md` item 155's shape.
fn players_can_differ(
    game: &GameState,
    node: &ModalNode,
    set: &[ModeIndex],
    player: PlayerId,
    this_object: ObjectId,
    named: ConditionReferents<'_>,
) -> bool {
    let identity = FilterIdentity::for_text_of(this_object, EarlierTargets::None).naming(named);
    let mut taken: Vec<PlayerId> = Vec::new();
    for mode in set.iter().filter_map(|&mode| node.mode(mode)) {
        for instance in &mode.instances {
            let (EffectRecipient::Target(filter @ SelectionFilter::Player(_), count)
            | EffectRecipient::Choose(filter @ SelectionFilter::Player(_), count)) = instance
            else {
                continue;
            };
            let needed = count.fewest();
            let free: Vec<PlayerId> = enumerate_legal_selections_excluding(game, filter, None, player, identity)
                .into_iter()
                .filter_map(|candidate| match candidate {
                    ObjectOrPlayerId::Player(seat) if !taken.contains(&seat) => Some(seat),
                    _ => None,
                })
                .take(needed)
                .collect();
            if free.len() < needed {
                return false;
            }
            taken.extend(free);
        }
    }
    true
}

/// Hands `visit` every way to pick `size` of the modes in `legal`, a mode as
/// often as it likes, each in printed order (CR 700.2d): from modes 1 and 2,
/// two at a time, that is {1, 1}, {1, 2}, {2, 2}. Returns `KeepGoing::No` when
/// `visit` stopped it. A size of 0 is the empty set, even with no mode legal.
fn for_each_multiset(legal: &[ModeIndex], size: usize, visit: &mut impl FnMut(&[ModeIndex]) -> KeepGoing) -> KeepGoing {
    if size == 0 {
        return visit(&[]);
    }
    if legal.is_empty() {
        return KeepGoing::Yes;
    }
    // `picks` are positions in `legal`, never falling. The next multiset
    // moves the rightmost pick that has room one step right, and sets every
    // pick after it equal to it; none has room once all are the last.
    let mut picks: Vec<usize> = vec![0; size];
    let mut set: Vec<ModeIndex> = Vec::with_capacity(size);
    loop {
        set.clear();
        set.extend(picks.iter().map(|&at| legal[at]));
        if visit(&set) == KeepGoing::No {
            return KeepGoing::No;
        }
        let Some(moves) = (0..size).rev().find(|&i| picks[i] + 1 < legal.len()) else { return KeepGoing::Yes };
        picks[moves] += 1;
        for i in moves + 1..size {
            picks[i] = picks[moves];
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

    fn multisets_of(legal: &[u8], size: usize) -> Vec<Vec<ModeIndex>> {
        let mut out = Vec::new();
        for_each_multiset(&modes(legal), size, &mut |set| {
            out.push(set.to_vec());
            KeepGoing::Yes
        });
        out
    }

    /// CR 700.2d: a mode's copies adjacent, in printed order, and a size past
    /// the modes there are filled by repeats.
    #[test]
    fn multisets_repeat_a_mode_in_printed_order() {
        assert_eq!(multisets_of(&[1, 2], 2), vec![modes(&[1, 1]), modes(&[1, 2]), modes(&[2, 2])]);
        assert_eq!(multisets_of(&[1, 2], 0), vec![modes(&[])]);
        assert_eq!(multisets_of(&[], 0), vec![modes(&[])]);
        assert!(multisets_of(&[], 1).is_empty());
        let up_to_four_of_two: usize = (0..=4).map(|size| multisets_of(&[1, 2], size).len()).sum();
        assert_eq!(up_to_four_of_two, 15, "Moment of Reckoning's count, by the design's table");
    }

    #[test]
    fn five_modes_one_or_more_are_thirty_one_sets() {
        let total: usize = (1..=5).map(|size| sets_of(&[1, 2, 3, 4, 5], size).len()).sum();
        assert_eq!(total, 31, "Casualties of War's count, by the design's table");
    }
}
