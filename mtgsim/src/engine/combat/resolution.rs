// Combat damage assignment and resolution.
// See rules 510.1–510.2.

use crate::engine::actions::GameAction;
use crate::events::event::DamageTarget;
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::ui::decision::DecisionProvider;

/// A single combat damage assignment: source deals amount to target.
#[derive(Debug, Clone, PartialEq)]
pub struct CombatDamageAssignment {
    pub source: ObjectId,
    pub target: DamageTarget,
    pub amount: u64,
}

/// Assign combat damage for all creatures currently in combat (rule 510.1).
///
/// This is a **read-only** free function taking `&GameState`. The caller is
/// responsible for calling `apply_combat_damage` afterward with the result.
///
/// For attackers blocked by 2+ creatures, the damage division is delegated
/// to `decisions.choose_attacker_damage_assignment()` — under 2025 rules
/// (510.1c), the attacking player freely divides damage among blockers with
/// no ordering or lethal-first constraint.
///
/// If `first_strike_only` is true, only creatures with first strike or double
/// strike assign damage. Phase 3: always false (no first/double strike exists).
pub fn assign_combat_damage(
    game: &GameState,
    decisions: &dyn DecisionProvider,
    active_player: PlayerId,
    first_strike_only: bool,
) -> Vec<CombatDamageAssignment> {
    let mut assignments = Vec::new();

    // Iterate all creatures on the battlefield that are in combat
    for (id, entry) in &game.battlefield {
        // --- Attackers ---
        if let Some(ref attacking_info) = entry.attacking {
            // Phase 3 stub: skip first-strike filtering
            if first_strike_only {
                // Phase 4: check for first_strike / double_strike keyword
                // For now, no creatures have these keywords, so skip all.
                continue;
            }

            let power = game.get_effective_power(*id).unwrap_or(0);
            if power <= 0 {
                // Rule 510.1a: 0 or less power → no damage
                continue;
            }
            let damage = power as u64;

            if !attacking_info.is_blocked {
                // Unblocked attacker: damage goes to attack target (rule 510.1b)
                let target = match &attacking_info.target {
                    AttackTarget::Player(pid) => Some(DamageTarget::Player(*pid)),
                    AttackTarget::Planeswalker(oid) => Some(DamageTarget::Object(*oid)),
                    AttackTarget::Battle(oid) => Some(DamageTarget::Object(*oid)),
                };
                if let Some(t) = target {
                    assignments.push(CombatDamageAssignment {
                        source: *id,
                        target: t,
                        amount: damage,
                    });
                }
            } else if attacking_info.blocked_by.is_empty() {
                // Blocked but all blockers removed (rule 510.1c): no damage
                // (creature was blocked, blockers left combat)
            } else if attacking_info.blocked_by.len() == 1 {
                // Exactly one blocker: all damage to it (rule 510.1c)
                let blocker = attacking_info.blocked_by[0];
                if game.battlefield.contains_key(&blocker) {
                    assignments.push(CombatDamageAssignment {
                        source: *id,
                        target: DamageTarget::Object(blocker),
                        amount: damage,
                    });
                }
            } else {
                // Multiple blockers: delegate damage division to the player
                // (rule 510.1c). Under 2025 rules, the player freely divides
                // damage among blockers with no ordering constraint.
                let division = decisions.choose_attacker_damage_assignment(
                    game, active_player, *id, &attacking_info.blocked_by, damage,
                );

                // Phase 4 (trample): validate that excess goes to defending
                // player only if all blockers have been assigned lethal.

                for (blocker_id, amount) in division {
                    if amount > 0 && game.battlefield.contains_key(&blocker_id) {
                        assignments.push(CombatDamageAssignment {
                            source: *id,
                            target: DamageTarget::Object(blocker_id),
                            amount,
                        });
                    }
                }
            }
        }

        // --- Blockers ---
        if let Some(ref blocking_info) = entry.blocking {
            // Phase 3 stub: skip first-strike filtering
            if first_strike_only {
                continue;
            }

            let power = game.get_effective_power(*id).unwrap_or(0);
            if power <= 0 {
                continue;
            }
            let damage = power as u64;

            if blocking_info.blocking.is_empty() {
                // Not blocking anything anymore (attacker removed)
                continue;
            }

            if blocking_info.blocking.len() == 1 {
                // Blocking exactly one creature: all damage to it (rule 510.1d)
                let attacker = blocking_info.blocking[0];
                if game.battlefield.contains_key(&attacker) {
                    assignments.push(CombatDamageAssignment {
                        source: *id,
                        target: DamageTarget::Object(attacker),
                        amount: damage,
                    });
                }
            } else {
                // Blocking multiple creatures: divide damage (rule 510.1d).
                // Phase 3 stub: all damage to the first living attacker.
                // Multi-block requires Banding or "block additional creature"
                // effects (Phase 4/5).
                //
                // TODO (Phase 4/5): Delegate to
                // DecisionProvider::choose_blocker_damage_division.
                let alive: Vec<ObjectId> = blocking_info.blocking.iter()
                    .copied()
                    .filter(|aid| game.battlefield.contains_key(aid))
                    .collect();
                if let Some(&first) = alive.first() {
                    assignments.push(CombatDamageAssignment {
                        source: *id,
                        target: DamageTarget::Object(first),
                        amount: damage,
                    });
                }
            }
        }
    }

    assignments
}

impl GameState {
    /// Apply all combat damage assignments simultaneously (rule 510.2).
    ///
    /// Each assignment is routed through `execute_action(GameAction::DealDamage)`
    /// so that Phase 6 replacement effects automatically intercept combat damage.
    pub fn apply_combat_damage(
        &mut self,
        assignments: Vec<CombatDamageAssignment>,
    ) -> Result<(), String> {
        for assignment in assignments {
            self.execute_action(GameAction::DealDamage {
                source: assignment.source,
                target: assignment.target,
                amount: assignment.amount,
                is_combat: true,
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::{AttackingInfo, BlockingInfo, BattlefieldEntity};
    use crate::types::card_types::CardType;
    use crate::types::colors::Color;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;
    use crate::ui::decision::PassiveDecisionProvider;

    fn place_creature_with_pt(
        game: &mut GameState,
        owner: PlayerId,
        power: i32,
        toughness: i32,
    ) -> ObjectId {
        let data = CardDataBuilder::new("Test Creature")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .mana_cost(ManaCost::single(ManaType::Green, 1, 1))
            .power_toughness(power, toughness)
            .build();
        let obj = GameObject::new(data, owner, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = BattlefieldEntity::new(id, owner, ts);
        entry.summoning_sick = false;
        game.battlefield.insert(id, entry);
        id
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

    #[test]
    fn test_unblocked_attacker_damages_player() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature_with_pt(&mut game, 0, 3, 3);
        set_attacking(&mut game, attacker, 1);

        let passive = PassiveDecisionProvider;
        let assignments = assign_combat_damage(&game, &passive, 0, false);

        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].source, attacker);
        assert_eq!(assignments[0].target, DamageTarget::Player(1));
        assert_eq!(assignments[0].amount, 3);
    }

    #[test]
    fn test_blocked_attacker_damages_single_blocker() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature_with_pt(&mut game, 0, 2, 2);
        let blocker = place_creature_with_pt(&mut game, 1, 2, 2);
        set_attacking(&mut game, attacker, 1);
        set_blocked_by(&mut game, attacker, vec![blocker]);
        set_blocking(&mut game, blocker, vec![attacker]);

        let passive = PassiveDecisionProvider;
        let assignments = assign_combat_damage(&game, &passive, 0, false);

        // Attacker deals 2 to blocker, blocker deals 2 to attacker
        assert_eq!(assignments.len(), 2);
        let att_dmg: Vec<_> = assignments.iter().filter(|a| a.source == attacker).collect();
        let blk_dmg: Vec<_> = assignments.iter().filter(|a| a.source == blocker).collect();
        assert_eq!(att_dmg.len(), 1);
        assert_eq!(att_dmg[0].target, DamageTarget::Object(blocker));
        assert_eq!(att_dmg[0].amount, 2);
        assert_eq!(blk_dmg.len(), 1);
        assert_eq!(blk_dmg[0].target, DamageTarget::Object(attacker));
        assert_eq!(blk_dmg[0].amount, 2);
    }

    #[test]
    fn test_blocked_no_remaining_blockers_no_damage() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature_with_pt(&mut game, 0, 3, 3);
        set_attacking(&mut game, attacker, 1);
        // Blocked, but blocker was removed from combat (empty blocked_by)
        if let Some(entry) = game.battlefield.get_mut(&attacker) {
            entry.attacking = Some(AttackingInfo {
                target: AttackTarget::Player(1),
                is_blocked: true,
                blocked_by: Vec::new(),
            });
        }

        let passive = PassiveDecisionProvider;
        let assignments = assign_combat_damage(&game, &passive, 0, false);
        // No damage from attacker (blocked with no blockers remaining)
        assert!(assignments.is_empty());
    }

    #[test]
    fn test_zero_power_creature_no_damage() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature_with_pt(&mut game, 0, 0, 1);
        set_attacking(&mut game, attacker, 1);

        let passive = PassiveDecisionProvider;
        let assignments = assign_combat_damage(&game, &passive, 0, false);
        assert!(assignments.is_empty());
    }

    #[test]
    fn test_apply_combat_damage_deals_to_player() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature_with_pt(&mut game, 0, 3, 3);

        let assignments = vec![CombatDamageAssignment {
            source: attacker,
            target: DamageTarget::Player(1),
            amount: 3,
        }];

        game.apply_combat_damage(assignments).unwrap();
        assert_eq!(game.players[1].life_total, 17);
    }

    #[test]
    fn test_apply_combat_damage_marks_creature() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature_with_pt(&mut game, 0, 2, 2);
        let blocker = place_creature_with_pt(&mut game, 1, 2, 3);

        let assignments = vec![CombatDamageAssignment {
            source: attacker,
            target: DamageTarget::Object(blocker),
            amount: 2,
        }];

        game.apply_combat_damage(assignments).unwrap();
        assert_eq!(game.battlefield.get(&blocker).unwrap().damage_marked, 2);
    }

    // TODO: Remove this test when Phase 4 implements first/double strike.
    // It only verifies the Phase 3 stub behavior (skip all on first_strike_only=true).
    // Phase 4 replaces this with real first-strike filtering tests.
    #[test]
    fn test_first_strike_only_skips_all_in_phase3() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature_with_pt(&mut game, 0, 3, 3);
        set_attacking(&mut game, attacker, 1);

        // first_strike_only = true → no creatures have first strike → empty
        let passive = PassiveDecisionProvider;
        let assignments = assign_combat_damage(&game, &passive, 0, true);
        assert!(assignments.is_empty());
    }

    #[test]
    fn test_multiple_blockers_damage_division() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature_with_pt(&mut game, 0, 5, 5);
        let b1 = place_creature_with_pt(&mut game, 1, 1, 2);
        let b2 = place_creature_with_pt(&mut game, 1, 1, 3);
        set_attacking(&mut game, attacker, 1);
        set_blocked_by(&mut game, attacker, vec![b1, b2]);
        set_blocking(&mut game, b1, vec![attacker]);
        set_blocking(&mut game, b2, vec![attacker]);

        // PassiveDecisionProvider uses default_damage_assignment which
        // assigns lethal to each blocker in order as a strategic default.
        let passive = PassiveDecisionProvider;
        let assignments = assign_combat_damage(&game, &passive, 0, false);

        // Attacker (5 power) → default strategy: 2 to b1 (lethal), 3 to b2
        let att_dmg: Vec<_> = assignments.iter().filter(|a| a.source == attacker).collect();
        assert_eq!(att_dmg.len(), 2);
        // Verify total damage sums to power
        let total: u64 = att_dmg.iter().map(|a| a.amount).sum();
        assert_eq!(total, 5);
    }
}
