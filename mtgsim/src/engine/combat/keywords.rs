// Combat-related keyword ability helpers.
//
// Extracted from resolution.rs to keep keyword logic colocated and
// independently testable. Each function handles one keyword's combat
// behavior; assign_combat_damage in resolution.rs calls these as an
// orchestrator.

use crate::engine::combat::resolution::CombatDamageAssignment;
use crate::events::event::DamageTarget;
use crate::oracle::characteristics::{has_keyword, get_effective_toughness};
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::keywords::KeywordAbility;
use crate::ui::ask::ask_choose_trample_damage_assignment;
use crate::ui::decision::DecisionProvider;

/// Convert an `AttackTarget` to a `DamageTarget`.
///
/// Used by trample and unblocked-attacker logic to determine where
/// overflow or direct damage goes.
pub fn attack_target_to_damage_target(target: &AttackTarget) -> DamageTarget {
    match target {
        AttackTarget::Player(pid) => DamageTarget::Player(*pid),
        AttackTarget::Planeswalker(oid) => DamageTarget::Object(*oid),
        AttackTarget::Battle(oid) => DamageTarget::Object(*oid),
    }
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
        has_keyword(game, creature_id, KeywordAbility::FirstStrike)
            || has_keyword(game, creature_id, KeywordAbility::DoubleStrike)
    } else {
        // Normal step: skip if already dealt FS damage and not double strike
        if game.dealt_first_strike_damage.contains(&creature_id)
            && !has_keyword(game, creature_id, KeywordAbility::DoubleStrike)
        {
            return false;
        }
        true
    }
}

/// Compute the lethal damage threshold for a creature, accounting for
/// deathtouch on the source.
///
/// If the source has deathtouch, 1 damage is considered lethal (rule 702.2b).
/// Otherwise, lethal = effective_toughness - damage_already_marked.
///
/// Returns 0 if the creature can't be found or has 0 effective toughness.
pub fn lethal_damage_for(
    game: &GameState,
    target_id: ObjectId,
    source_has_deathtouch: bool,
) -> u64 {
    if source_has_deathtouch {
        return 1;
    }
    let toughness = get_effective_toughness(game, target_id).unwrap_or(0);
    if toughness <= 0 {
        return 0;
    }
    let damage_marked = game
        .battlefield
        .get(&target_id)
        .map(|e| e.damage_marked)
        .unwrap_or(0);
    let remaining = (toughness as u64).saturating_sub(damage_marked as u64);
    remaining.max(1) // at least 1 to assign
}

/// Assign trample damage for a blocked attacker with trample.
///
/// Delegates the actual damage division to `DecisionProvider`, then
/// returns the resulting `CombatDamageAssignment` entries.
///
/// Rule 702.19b: must assign at least lethal to each blocker, excess
/// may be assigned to the defending player/planeswalker.
pub fn assign_trample_damage(
    game: &GameState,
    decisions: &dyn DecisionProvider,
    active_player: PlayerId,
    attacker_id: ObjectId,
    blocked_by: &[ObjectId],
    attack_target: &AttackTarget,
    damage: u64,
) -> Vec<CombatDamageAssignment> {
    let mut assignments = Vec::new();
    let has_deathtouch = has_keyword(game, attacker_id, KeywordAbility::Deathtouch);
    let defending_target = attack_target_to_damage_target(attack_target);

    let alive_blockers: Vec<ObjectId> = blocked_by
        .iter()
        .copied()
        .filter(|bid| game.battlefield.contains_key(bid))
        .collect();

    // Compute per-blocker minimums: 1 if deathtouch, else toughness − damage_marked
    let per_blocker_mins: Vec<u64> = alive_blockers
        .iter()
        .map(|&bid| {
            if has_deathtouch {
                1
            } else {
                let toughness = get_effective_toughness(game, bid).unwrap_or(0) as u64;
                let marked = game.battlefield.get(&bid).map(|e| e.damage_marked as u64).unwrap_or(0);
                toughness.saturating_sub(marked)
            }
        })
        .collect();

    // Rule 702.19b: trample requires assigning at least lethal damage to
    // each blocker before excess can trample through to the defending target.
    //
    // When power < sum(lethals), the attacker cannot meet the lethal
    // requirement for all blockers, so NO overflow to the defender is
    // possible. We express this via per_bucket_maxs: blocker buckets are
    // uncapped, but the defending-target bucket is capped at 0. The DP
    // freely divides among blockers with per-blocker mins dropped to 0
    // (since the total can't satisfy them all anyway).
    //
    // Note on array lengths: we pass blocker-only mins (length =
    // alive_blockers.len()) because ask_choose_trample_damage_assignment
    // appends the defender's min (0) internally. But we pass maxs for ALL
    // buckets (length = alive_blockers.len() + 1, including the defender)
    // because ask.rs passes maxs straight through to dp.allocate().
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
        defending_target.clone(),
        damage,
        &effective_blocker_mins,
        Some(&maxs),
    );

    for (blocker_id, amount) in blocker_assignments {
        if amount > 0 && game.battlefield.contains_key(&blocker_id) {
            assignments.push(CombatDamageAssignment {
                source: attacker_id,
                target: DamageTarget::Object(blocker_id),
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
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::{AttackingInfo, BattlefieldEntity, BlockingInfo};
    use crate::types::card_types::CardType;
    use crate::types::colors::Color;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;
    use crate::events::event::DamageTarget;
    use crate::ui::choice_types::ChoiceKind;
    use crate::ui::decision::ScriptedDecisionProvider;

    fn place_creature_with_keywords(
        game: &mut GameState,
        owner: PlayerId,
        power: i32,
        toughness: i32,
        keywords: &[KeywordAbility],
    ) -> ObjectId {
        let mut builder = CardDataBuilder::new("Test Creature")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .mana_cost(ManaCost::build(&[ManaType::Green], 1))
            .power_toughness(power, toughness);
        for kw in keywords {
            builder = builder.keyword(*kw);
        }
        let data = builder.build();
        let obj = GameObject::new(data, owner, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = BattlefieldEntity::new(id, owner, ts, 0);
        game.battlefield.insert(id, entry);
        id
    }

    fn place_creature(
        game: &mut GameState,
        owner: PlayerId,
        power: i32,
        toughness: i32,
    ) -> ObjectId {
        place_creature_with_keywords(game, owner, power, toughness, &[])
    }

    fn set_attacking(game: &mut GameState, id: ObjectId, target_player: PlayerId) {
        if let Some(entry) = game.battlefield.get_mut(&id) {
            entry.attacking = Some(AttackingInfo {
                target: AttackTarget::Player(target_player),
                is_blocked: false,
                blocked_by: Vec::new(),
            });
        }
    }

    fn set_blocked_by(game: &mut GameState, attacker: ObjectId, blockers: Vec<ObjectId>) {
        if let Some(entry) = game.battlefield.get_mut(&attacker) {
            if let Some(ref mut info) = entry.attacking {
                info.is_blocked = true;
                info.blocked_by = blockers;
            }
        }
    }

    fn set_blocking(game: &mut GameState, blocker: ObjectId, blocking: Vec<ObjectId>) {
        if let Some(entry) = game.battlefield.get_mut(&blocker) {
            entry.blocking = Some(BlockingInfo { blocking });
        }
    }

    // --- should_deal_damage_this_step tests ---

    #[test]
    fn test_first_strike_deals_in_first_step() {
        let mut game = GameState::new(2, 20);
        let fs = place_creature_with_keywords(&mut game, 0, 2, 1, &[KeywordAbility::FirstStrike]);
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
        let fs = place_creature_with_keywords(&mut game, 0, 2, 1, &[KeywordAbility::FirstStrike]);
        game.dealt_first_strike_damage.insert(fs);
        assert!(!should_deal_damage_this_step(&game, fs, false));
    }

    #[test]
    fn test_double_striker_deals_in_both_steps() {
        let mut game = GameState::new(2, 20);
        let ds = place_creature_with_keywords(&mut game, 0, 2, 2, &[KeywordAbility::DoubleStrike]);
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

    // --- lethal_damage_for tests ---

    #[test]
    fn test_lethal_damage_normal() {
        let mut game = GameState::new(2, 20);
        let c = place_creature(&mut game, 0, 2, 4);
        assert_eq!(lethal_damage_for(&game, c, false), 4);
    }

    #[test]
    fn test_lethal_damage_with_existing_damage() {
        let mut game = GameState::new(2, 20);
        let c = place_creature(&mut game, 0, 2, 4);
        game.battlefield.get_mut(&c).unwrap().damage_marked = 2;
        assert_eq!(lethal_damage_for(&game, c, false), 2);
    }

    #[test]
    fn test_lethal_damage_deathtouch_is_one() {
        let mut game = GameState::new(2, 20);
        let c = place_creature(&mut game, 0, 2, 8);
        assert_eq!(lethal_damage_for(&game, c, true), 1);
    }

    // --- attack_target_to_damage_target tests ---

    #[test]
    fn test_attack_target_player() {
        assert_eq!(
            attack_target_to_damage_target(&AttackTarget::Player(1)),
            DamageTarget::Player(1)
        );
    }

    // --- assign_trample_damage tests ---

    #[test]
    fn test_trample_excess_to_player() {
        let mut game = GameState::new(2, 20);
        let trampler = place_creature_with_keywords(&mut game, 0, 4, 4, &[KeywordAbility::Trample]);
        let blocker = place_creature(&mut game, 1, 1, 2);
        set_attacking(&mut game, trampler, 1);
        set_blocked_by(&mut game, trampler, vec![blocker]);
        set_blocking(&mut game, blocker, vec![trampler]);

        // Script allocation: [2 to blocker, 2 to player]
        let scripted = ScriptedDecisionProvider::new();
        scripted.expect_allocation(
            ChoiceKind::AssignTrampleDamage {
                attacker_id: trampler,
                defending_target: DamageTarget::Player(1),
            },
            vec![2, 2],
        );
        let assignments = assign_trample_damage(
            &game, &scripted, 0, trampler, &[blocker],
            &AttackTarget::Player(1), 4,
        );

        let to_blocker: Vec<_> = assignments.iter()
            .filter(|a| a.target == DamageTarget::Object(blocker))
            .collect();
        let to_player: Vec<_> = assignments.iter()
            .filter(|a| a.target == DamageTarget::Player(1))
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
            &[KeywordAbility::Trample, KeywordAbility::Deathtouch],
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
                defending_target: DamageTarget::Player(1),
            },
            vec![1, 3],
        );
        let assignments = assign_trample_damage(
            &game, &scripted, 0, trampler, &[blocker],
            &AttackTarget::Player(1), 4,
        );

        // Deathtouch: 1 is lethal → 1 to blocker, 3 tramples
        let to_blocker: Vec<_> = assignments.iter()
            .filter(|a| a.target == DamageTarget::Object(blocker))
            .collect();
        let to_player: Vec<_> = assignments.iter()
            .filter(|a| a.target == DamageTarget::Player(1))
            .collect();
        assert_eq!(to_blocker.len(), 1);
        assert_eq!(to_blocker[0].amount, 1);
        assert_eq!(to_player.len(), 1);
        assert_eq!(to_player[0].amount, 3);
    }

    #[test]
    fn test_trample_not_enough_power() {
        let mut game = GameState::new(2, 20);
        let trampler = place_creature_with_keywords(&mut game, 0, 2, 2, &[KeywordAbility::Trample]);
        let blocker = place_creature(&mut game, 1, 1, 3);
        set_attacking(&mut game, trampler, 1);
        set_blocked_by(&mut game, trampler, vec![blocker]);
        set_blocking(&mut game, blocker, vec![trampler]);

        // Power(2) < lethal(3): defender bucket maxed at 0, blocker mins dropped to 0.
        // DP freely divides 2 among blockers. Script: [2 to blocker, 0 to player]
        let scripted = ScriptedDecisionProvider::new();
        scripted.expect_allocation(
            ChoiceKind::AssignTrampleDamage {
                attacker_id: trampler,
                defending_target: DamageTarget::Player(1),
            },
            vec![2, 0],
        );
        let assignments = assign_trample_damage(
            &game, &scripted, 0, trampler, &[blocker],
            &AttackTarget::Player(1), 2,
        );

        // 2 power vs 3 toughness: all 2 to blocker, no overflow
        let to_blocker: Vec<_> = assignments.iter()
            .filter(|a| a.target == DamageTarget::Object(blocker))
            .collect();
        let to_player: Vec<_> = assignments.iter()
            .filter(|a| a.target == DamageTarget::Player(1))
            .collect();
        assert_eq!(to_blocker.len(), 1);
        assert_eq!(to_blocker[0].amount, 2);
        assert!(to_player.is_empty());
    }
}
