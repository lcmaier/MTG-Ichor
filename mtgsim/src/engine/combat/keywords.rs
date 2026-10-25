// Combat-related keyword ability helpers.
//
// Each function handles one keyword's combat behavior;
// `assign_combat_damage` in resolution.rs calls these as an orchestrator.

use crate::engine::combat::resolution::CombatDamageAssignment;
use crate::oracle::characteristics::{has_keyword, get_effective_toughness};
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
use crate::types::keywords::KeywordFlag;
use crate::ui::ask::ask_choose_trample_damage_assignment;
use crate::ui::decision::DecisionProvider;

/// Convert an `AttackTarget` to an `ObjectOrPlayerId`.
///
/// Used by trample and unblocked-attacker logic to determine where
/// overflow or direct damage goes.
pub fn attack_target_to_damage_target(target: &AttackTarget) -> ObjectOrPlayerId {
    match target {
        AttackTarget::Player(pid) => ObjectOrPlayerId::Player(*pid),
        AttackTarget::Planeswalker(oid) => ObjectOrPlayerId::Object(*oid),
        AttackTarget::Battle(oid) => ObjectOrPlayerId::Object(*oid),
    }
}

/// Whether `creature` has first strike or double strike (CR 702.7b,
/// 702.4b): what puts it in the first-strike combat damage step.
pub fn has_first_or_double_strike(game: &GameState, creature: ObjectId) -> bool {
    has_keyword(game, creature, KeywordFlag::FirstStrike) || has_keyword(game, creature, KeywordFlag::DoubleStrike)
}

/// Determine whether a creature should deal damage in the current
/// combat damage step, given first strike / double strike rules.
///
/// - `first_strike_only == true` (first-strike damage step):
///   Only creatures with FirstStrike or DoubleStrike deal damage.
/// - `first_strike_only == false` (normal damage step):
///   Skip creatures that already dealt first-strike damage, UNLESS
///   they have DoubleStrike (they deal damage again).
///
/// Rules 702.7 (first strike), 702.4 (double strike).
pub fn should_deal_damage_this_step(
    game: &GameState,
    creature_id: ObjectId,
    first_strike_only: bool,
) -> bool {
    if first_strike_only {
        has_first_or_double_strike(game, creature_id)
    } else {
        // Normal step: skip if already dealt FS damage and not double strike
        if game.dealt_first_strike_damage.contains(&creature_id)
            && !has_keyword(game, creature_id, KeywordFlag::DoubleStrike)
        {
            return false;
        }
        true
    }
}

/// CR 702.19b's lethal damage still owed to `blocker` by a creature assigning
/// its combat damage now: what its toughness leaves after the damage marked on
/// it and the damage `assigned` gives it from other creatures this step. Any
/// nonzero deathtouch damage is lethal (CR 702.2c), so a deathtouch source owes
/// at most 1, and a blocker one already assigned to owes nothing more. A
/// blocker already dealt lethal damage is owed nothing. With nothing
/// assigned, it is CR 120.10's lethal damage too, which a batch's excess
/// damage is measured past.
pub fn lethal_damage_owed(
    game: &GameState,
    blocker: ObjectId,
    assigned: &[CombatDamageAssignment],
    source_has_deathtouch: bool,
) -> u64 {
    let mut assigned_amount = 0u64;
    for assignment in assigned.iter().filter(|a| a.target == ObjectOrPlayerId::Object(blocker) && a.amount > 0) {
        if has_keyword(game, assignment.source, KeywordFlag::Deathtouch) {
            return 0;
        }
        assigned_amount += assignment.amount;
    }
    let toughness = get_effective_toughness(game, blocker).unwrap_or(0).max(0) as u64;
    let marked = game.battlefield.get(&blocker).map_or(0, |entry| entry.damage_marked as u64);
    let owed = toughness.saturating_sub(marked).saturating_sub(assigned_amount);
    if source_has_deathtouch { owed.min(1) } else { owed }
}

/// Assign trample damage for a blocked attacker with trample.
///
/// Delegates the actual damage division to `DecisionProvider`, then
/// returns the resulting `CombatDamageAssignment` entries.
///
/// Rule 702.19b: must assign at least lethal to each blocker, excess
/// may be assigned to the defending player/planeswalker. `assigned` is what
/// the creatures before this one assign this step, which lethal counts.
pub fn assign_trample_damage(
    game: &GameState,
    decisions: &dyn DecisionProvider,
    active_player: PlayerId,
    attacker_id: ObjectId,
    blocked_by: &[ObjectId],
    attack_target: &AttackTarget,
    damage: u64,
    assigned: &[CombatDamageAssignment],
) -> Vec<CombatDamageAssignment> {
    let mut assignments = Vec::new();
    let has_deathtouch = has_keyword(game, attacker_id, KeywordFlag::Deathtouch);
    let defending_target = attack_target_to_damage_target(attack_target);

    let alive_blockers: Vec<ObjectId> = blocked_by
        .iter()
        .copied()
        .filter(|bid| game.battlefield.contains_key(bid))
        .collect();

    let per_blocker_mins: Vec<u64> = alive_blockers
        .iter()
        .map(|&bid| lethal_damage_owed(game, bid, assigned, has_deathtouch))
        .collect();

    // CR 702.19b: lethal to each blocker before any tramples over. Below the
    // lethal total the defender's bucket is capped at 0 and the minimums drop to
    // 0, the provider dividing freely. Mins are per blocker (the ask appends the
    // defender's 0); maxs cover every bucket (`+ 1`), passed straight to `allocate`.
    let min_sum: u64 = per_blocker_mins.iter().sum();
    let (effective_blocker_mins, maxs) = if min_sum > damage {
        let mins = vec![0u64; alive_blockers.len()];
        let mut maxs = vec![u64::MAX; alive_blockers.len() + 1];
        *maxs.last_mut().unwrap() = 0; // defender bucket capped at 0
        (mins, maxs)
    } else {
        (per_blocker_mins.clone(), vec![u64::MAX; alive_blockers.len() + 1])
    };

    let (blocker_assignments, overflow) = ask_choose_trample_damage_assignment(
        decisions,
        game,
        active_player,
        attacker_id,
        &alive_blockers,
        defending_target,
        damage,
        &effective_blocker_mins,
        Some(&maxs),
    );

    for (blocker_id, amount) in blocker_assignments {
        if amount > 0 && game.battlefield.contains_key(&blocker_id) {
            assignments.push(CombatDamageAssignment {
                source: attacker_id,
                target: ObjectOrPlayerId::Object(blocker_id),
                amount,
            });
        }
    }
    if overflow > 0 {
        assignments.push(CombatDamageAssignment {
            source: attacker_id,
            target: defending_target,
            amount: overflow,
        });
    }

    assignments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ids::ObjectOrPlayerId;
    use crate::ui::choice_types::ChoiceKind;
    use crate::ui::decision::ScriptedDecisionProvider;
    use crate::test_support::{
        place_vanilla_creature as place_creature_with_keywords, set_attacking, set_blocked_by,
        set_blocking,
    };

    fn place_creature(
        game: &mut GameState,
        owner: PlayerId,
        power: i32,
        toughness: i32,
    ) -> ObjectId {
        place_creature_with_keywords(game, owner, power, toughness, &[])
    }

    // --- should_deal_damage_this_step tests ---

    #[test]
    fn test_first_strike_deals_in_first_step() {
        let mut game = GameState::new(2, 20);
        let fs = place_creature_with_keywords(&mut game, 0, 2, 1, &[KeywordFlag::FirstStrike]);
        assert!(should_deal_damage_this_step(&game, fs, true));
    }

    #[test]
    fn test_normal_creature_skips_first_step() {
        let mut game = GameState::new(2, 20);
        let normal = place_creature(&mut game, 0, 3, 3);
        assert!(!should_deal_damage_this_step(&game, normal, true));
    }

    #[test]
    fn test_first_striker_skips_normal_step_after_dealing() {
        let mut game = GameState::new(2, 20);
        let fs = place_creature_with_keywords(&mut game, 0, 2, 1, &[KeywordFlag::FirstStrike]);
        game.dealt_first_strike_damage.insert(fs);
        assert!(!should_deal_damage_this_step(&game, fs, false));
    }

    #[test]
    fn test_double_striker_deals_in_both_steps() {
        let mut game = GameState::new(2, 20);
        let ds = place_creature_with_keywords(&mut game, 0, 2, 2, &[KeywordFlag::DoubleStrike]);
        assert!(should_deal_damage_this_step(&game, ds, true));
        game.dealt_first_strike_damage.insert(ds);
        assert!(should_deal_damage_this_step(&game, ds, false));
    }

    #[test]
    fn test_normal_creature_deals_in_normal_step() {
        let mut game = GameState::new(2, 20);
        let normal = place_creature(&mut game, 0, 3, 3);
        assert!(should_deal_damage_this_step(&game, normal, false));
    }

    // --- lethal_damage_owed tests ---

    #[test]
    fn test_lethal_damage_normal() {
        let mut game = GameState::new(2, 20);
        let c = place_creature(&mut game, 0, 2, 4);
        assert_eq!(lethal_damage_owed(&game, c, &[], false), 4);
    }

    #[test]
    fn test_lethal_damage_with_existing_damage() {
        let mut game = GameState::new(2, 20);
        let c = place_creature(&mut game, 0, 2, 4);
        game.battlefield.get_mut(&c).unwrap().damage_marked = 2;
        assert_eq!(lethal_damage_owed(&game, c, &[], false), 2);
    }

    #[test]
    fn test_lethal_damage_deathtouch_is_one() {
        let mut game = GameState::new(2, 20);
        let c = place_creature(&mut game, 0, 2, 8);
        assert_eq!(lethal_damage_owed(&game, c, &[], true), 1);
    }

    #[test]
    fn test_lethal_damage_counts_what_others_assign_this_step() {
        // CR 702.19b, and CR 702.2c for an earlier deathtouch source.
        let mut game = GameState::new(2, 20);
        let blocker = place_creature(&mut game, 1, 0, 5);
        let plain = place_creature(&mut game, 0, 2, 2);
        let deathtouch = place_creature_with_keywords(&mut game, 0, 1, 1, &[KeywordFlag::Deathtouch]);
        let to_blocker = |source, amount| CombatDamageAssignment { source, target: ObjectOrPlayerId::Object(blocker), amount };
        assert_eq!(lethal_damage_owed(&game, blocker, &[to_blocker(plain, 2)], false), 3);
        assert_eq!(lethal_damage_owed(&game, blocker, &[to_blocker(deathtouch, 1)], false), 0);
        assert_eq!(lethal_damage_owed(&game, blocker, &[to_blocker(plain, 5)], true), 0);
    }

    // --- attack_target_to_damage_target tests ---

    #[test]
    fn test_attack_target_player() {
        assert_eq!(
            attack_target_to_damage_target(&AttackTarget::Player(1)),
            ObjectOrPlayerId::Player(1)
        );
    }

    // --- assign_trample_damage tests ---

    #[test]
    fn test_trample_excess_to_player() {
        let mut game = GameState::new(2, 20);
        let trampler = place_creature_with_keywords(&mut game, 0, 4, 4, &[KeywordFlag::Trample]);
        let blocker = place_creature(&mut game, 1, 1, 2);
        set_attacking(&mut game, trampler, 1);
        set_blocked_by(&mut game, trampler, vec![blocker]);
        set_blocking(&mut game, blocker, vec![trampler]);

        // Script allocation: [2 to blocker, 2 to player]
        let scripted = ScriptedDecisionProvider::new();
        scripted.expect_allocation(
            ChoiceKind::AssignTrampleDamage {
                attacker_id: trampler,
                defending_target: ObjectOrPlayerId::Player(1),
            },
            vec![2, 2],
        );
        let assignments = assign_trample_damage(
            &game, &scripted, 0, trampler, &[blocker],
            &AttackTarget::Player(1), 4, &[],
        );

        let to_blocker: Vec<_> = assignments.iter()
            .filter(|a| a.target == ObjectOrPlayerId::Object(blocker))
            .collect();
        let to_player: Vec<_> = assignments.iter()
            .filter(|a| a.target == ObjectOrPlayerId::Player(1))
            .collect();
        assert_eq!(to_blocker.len(), 1);
        assert_eq!(to_blocker[0].amount, 2);
        assert_eq!(to_player.len(), 1);
        assert_eq!(to_player[0].amount, 2);
    }

    #[test]
    fn test_trample_deathtouch_maximizes_overflow() {
        let mut game = GameState::new(2, 20);
        let trampler = place_creature_with_keywords(
            &mut game, 0, 4, 4,
            &[KeywordFlag::Trample, KeywordFlag::Deathtouch],
        );
        let blocker = place_creature(&mut game, 1, 2, 5);
        set_attacking(&mut game, trampler, 1);
        set_blocked_by(&mut game, trampler, vec![blocker]);
        set_blocking(&mut game, blocker, vec![trampler]);

        // Deathtouch: lethal=1. Script allocation: [1 to blocker, 3 to player]
        let scripted = ScriptedDecisionProvider::new();
        scripted.expect_allocation(
            ChoiceKind::AssignTrampleDamage {
                attacker_id: trampler,
                defending_target: ObjectOrPlayerId::Player(1),
            },
            vec![1, 3],
        );
        let assignments = assign_trample_damage(
            &game, &scripted, 0, trampler, &[blocker],
            &AttackTarget::Player(1), 4, &[],
        );

        // Deathtouch: 1 is lethal → 1 to blocker, 3 tramples
        let to_blocker: Vec<_> = assignments.iter()
            .filter(|a| a.target == ObjectOrPlayerId::Object(blocker))
            .collect();
        let to_player: Vec<_> = assignments.iter()
            .filter(|a| a.target == ObjectOrPlayerId::Player(1))
            .collect();
        assert_eq!(to_blocker.len(), 1);
        assert_eq!(to_blocker[0].amount, 1);
        assert_eq!(to_player.len(), 1);
        assert_eq!(to_player[0].amount, 3);
    }

    #[test]
    fn test_trample_not_enough_power() {
        let mut game = GameState::new(2, 20);
        let trampler = place_creature_with_keywords(&mut game, 0, 2, 2, &[KeywordFlag::Trample]);
        let blocker = place_creature(&mut game, 1, 1, 3);
        set_attacking(&mut game, trampler, 1);
        set_blocked_by(&mut game, trampler, vec![blocker]);
        set_blocking(&mut game, blocker, vec![trampler]);

        // Power(2) < lethal(3): defender bucket maxed at 0, blocker mins dropped
        // to 0 — so the blocker is the only bucket that can take the two, the
        // split is forced and nothing is asked. The assertions below
        // are the engine's own answer, not a scripted one.
        let scripted = ScriptedDecisionProvider::new();
        let assignments = assign_trample_damage(
            &game, &scripted, 0, trampler, &[blocker],
            &AttackTarget::Player(1), 2, &[],
        );

        // 2 power vs 3 toughness: all 2 to blocker, no overflow
        let to_blocker: Vec<_> = assignments.iter()
            .filter(|a| a.target == ObjectOrPlayerId::Object(blocker))
            .collect();
        let to_player: Vec<_> = assignments.iter()
            .filter(|a| a.target == ObjectOrPlayerId::Player(1))
            .collect();
        assert_eq!(to_blocker.len(), 1);
        assert_eq!(to_blocker[0].amount, 2);
        assert!(to_player.is_empty());
    }
}
