// Combat validation — attacker and blocker legality checks.
// See rules 508.1 (attackers) and 509.1 (blockers).

use std::collections::HashMap;

use crate::state::battlefield::AttackTarget;
use crate::state::game_state::GameState;
use crate::types::card_types::CardType;
use crate::types::ids::{ObjectId, PlayerId};

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Structured error for combat validation failures.
#[derive(Debug, Clone, PartialEq)]
pub enum CombatError {
    NotOnBattlefield(ObjectId),
    NotACreature(ObjectId),
    NotControlledByPlayer(ObjectId, PlayerId),
    CreatureIsTapped(ObjectId),
    CreatureHasSummoningSickness(ObjectId),
    InvalidAttackTarget(ObjectId),
    AttackerNotAttackingThisPlayer(ObjectId, ObjectId),
    TooManyBlocks(ObjectId, usize),
    ConstraintViolation(String),
}

impl std::fmt::Display for CombatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CombatError::NotOnBattlefield(id) => write!(f, "Object {} is not on the battlefield", id),
            CombatError::NotACreature(id) => write!(f, "Object {} is not a creature", id),
            CombatError::NotControlledByPlayer(id, pid) => write!(f, "Object {} is not controlled by player {}", id, pid),
            CombatError::CreatureIsTapped(id) => write!(f, "Creature {} is tapped", id),
            CombatError::CreatureHasSummoningSickness(id) => write!(f, "Creature {} has summoning sickness", id),
            CombatError::InvalidAttackTarget(id) => write!(f, "Invalid attack target for creature {}", id),
            CombatError::AttackerNotAttackingThisPlayer(blocker, attacker) => {
                write!(f, "Blocker {} cannot block attacker {} (not attacking this player)", blocker, attacker)
            }
            CombatError::TooManyBlocks(id, max) => write!(f, "Creature {} cannot block more than {} attacker(s)", id, max),
            CombatError::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
        }
    }
}

// ---------------------------------------------------------------------------
// Attack constraints (skeleton — populated by Phase 4/5)
// ---------------------------------------------------------------------------

/// Restrictions and requirements that apply to the set of declared attackers.
///
/// Phase 3: always `AttackConstraints::none()`.
/// Phase 4: Defender keyword adds `CantAttack` restrictions.
/// Phase 5: Continuous effects populate restrictions/requirements.
pub struct AttackConstraints {
    pub restrictions: Vec<AttackRestriction>,
    pub requirements: Vec<AttackRequirement>,
}

/// An effect that prevents a creature from attacking.
#[derive(Debug, Clone)]
pub enum AttackRestriction {
    /// This creature can't attack (e.g. Defender keyword)
    CantAttack(ObjectId),
    /// This creature can't attack alone
    CantAttackAlone(ObjectId),
    /// No more than N creatures can attack this turn
    MaxAttackers(usize),
}

/// An effect that requires a creature to attack if able.
#[derive(Debug, Clone)]
pub enum AttackRequirement {
    /// This creature attacks each combat if able
    MustAttackIfAble(ObjectId),
}

impl AttackConstraints {
    /// No constraints — used in Phase 3 where no restriction/requirement effects exist.
    pub fn none() -> Self {
        AttackConstraints {
            restrictions: Vec::new(),
            requirements: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Block constraints (skeleton — populated by Phase 4/5)
// ---------------------------------------------------------------------------

/// Restrictions, requirements, and blocking limits for declared blockers.
///
/// Phase 3: always `BlockConstraints::none()`.
/// Phase 4: Flying/reach adds evasion restrictions, menace adds min-blockers.
/// Phase 5: Continuous effects populate blocking_limits (e.g. "can block additional creature").
pub struct BlockConstraints {
    pub restrictions: Vec<BlockRestriction>,
    pub requirements: Vec<BlockRequirement>,
    /// Per-creature maximum number of attackers it can block. Default: 1.
    pub blocking_limits: HashMap<ObjectId, usize>,
}

/// An effect that prevents a creature from blocking.
#[derive(Debug, Clone)]
pub enum BlockRestriction {
    /// This creature can't block
    CantBlock(ObjectId),
    /// This creature can't block unless some condition (placeholder)
    CantBlockUnless(ObjectId, String),
}

/// An effect that requires a creature to block if able.
#[derive(Debug, Clone)]
pub enum BlockRequirement {
    /// This creature blocks each combat if able
    MustBlockIfAble(ObjectId),
}

impl BlockConstraints {
    /// No constraints — used in Phase 3.
    pub fn none() -> Self {
        BlockConstraints {
            restrictions: Vec::new(),
            requirements: Vec::new(),
            blocking_limits: HashMap::new(),
        }
    }

    /// How many attackers this creature can block. Defaults to 1.
    pub fn max_blocks_for(&self, creature_id: ObjectId) -> usize {
        self.blocking_limits.get(&creature_id).copied().unwrap_or(1)
    }
}

// ---------------------------------------------------------------------------
// Effective characteristic helpers
// ---------------------------------------------------------------------------
// These read card_data directly in Phase 3. Phase 5 will replace them with
// layer-system-aware lookups. Combat code calls these instead of reading
// card_data fields directly, so the transition is a single-point change.

impl GameState {
    /// Check if an object on the battlefield is currently a creature.
    /// Phase 3: reads printed types. Phase 5: reads effective types from layer system.
    pub fn is_creature(&self, id: ObjectId) -> bool {
        self.objects.get(&id)
            .map(|obj| obj.card_data.types.contains(&CardType::Creature))
            .unwrap_or(false)
    }

    /// Check if a creature can attack (not summoning-sick, or has haste).
    /// Phase 3: just checks summoning_sick. Phase 4: adds haste keyword bypass.
    pub fn can_attack(&self, id: ObjectId) -> bool {
        if let Some(entry) = self.battlefield.get(&id) {
            // Phase 4: || self.has_effective_keyword(id, KeywordAbility::Haste)
            !entry.summoning_sick
        } else {
            false
        }
    }

    /// Get effective power for a creature on the battlefield.
    /// Phase 3: base + modifier. Phase 5: computed through layer system.
    pub fn get_effective_power(&self, id: ObjectId) -> Option<i32> {
        let obj = self.objects.get(&id)?;
        let entry = self.battlefield.get(&id)?;
        let base = obj.card_data.power?;
        Some(base + entry.power_modifier)
    }

    /// Get effective toughness for a creature on the battlefield.
    /// Phase 3: base + modifier. Phase 5: computed through layer system.
    pub fn get_effective_toughness(&self, id: ObjectId) -> Option<i32> {
        let obj = self.objects.get(&id)?;
        let entry = self.battlefield.get(&id)?;
        let base = obj.card_data.toughness?;
        Some(base + entry.toughness_modifier)
    }
}

// ---------------------------------------------------------------------------
// Attacker validation (rule 508.1)
// ---------------------------------------------------------------------------

/// Validate a proposed set of attackers.
///
/// Checks per-creature legality (rule 508.1a) and set-level constraints
/// (rule 508.1c-d). Returns `Ok(())` if the declaration is legal.
pub fn validate_attackers(
    game: &GameState,
    player_id: PlayerId,
    proposed: &[(ObjectId, AttackTarget)],
    constraints: &AttackConstraints,
) -> Result<(), CombatError> {
    let num_players = game.num_players();

    for (creature_id, target) in proposed {
        // 1. Must be on the battlefield
        let entry = game.battlefield.get(creature_id)
            .ok_or(CombatError::NotOnBattlefield(*creature_id))?;

        // 2. Must be a creature
        if !game.is_creature(*creature_id) {
            return Err(CombatError::NotACreature(*creature_id));
        }

        // 3. Must be controlled by the attacking player
        if entry.controller != player_id {
            return Err(CombatError::NotControlledByPlayer(*creature_id, player_id));
        }

        // 4. Must be untapped (rule 508.1a)
        if entry.tapped {
            return Err(CombatError::CreatureIsTapped(*creature_id));
        }

        // 5. Must not have summoning sickness (unless haste — Phase 4)
        if !game.can_attack(*creature_id) {
            return Err(CombatError::CreatureHasSummoningSickness(*creature_id));
        }

        // 6. Attack target must be valid
        match target {
            AttackTarget::Player(pid) => {
                // Must be an opponent (not self, and within player range)
                if *pid == player_id || *pid >= num_players {
                    return Err(CombatError::InvalidAttackTarget(*creature_id));
                }
            }
            AttackTarget::Planeswalker(_) | AttackTarget::Battle(_) => {
                // Phase 3: planeswalkers and battles not yet supported as attack targets
                return Err(CombatError::InvalidAttackTarget(*creature_id));
            }
        }
    }

    // Set-level constraint checks (rule 508.1c-d)
    check_attack_constraints(proposed, constraints)?;

    Ok(())
}

/// Check set-level attack constraints (restrictions and requirements).
///
/// Phase 3: this is a no-op when constraints is `AttackConstraints::none()`.
/// Phase 4/5 will populate constraints from keywords and continuous effects.
fn check_attack_constraints(
    proposed: &[(ObjectId, AttackTarget)],
    constraints: &AttackConstraints,
) -> Result<(), CombatError> {
    // Check restrictions
    for restriction in &constraints.restrictions {
        match restriction {
            AttackRestriction::CantAttack(id) => {
                if proposed.iter().any(|(cid, _)| cid == id) {
                    return Err(CombatError::ConstraintViolation(
                        format!("Creature {} can't attack", id),
                    ));
                }
            }
            AttackRestriction::CantAttackAlone(id) => {
                if proposed.len() == 1 && proposed[0].0 == *id {
                    return Err(CombatError::ConstraintViolation(
                        format!("Creature {} can't attack alone", id),
                    ));
                }
            }
            AttackRestriction::MaxAttackers(max) => {
                if proposed.len() > *max {
                    return Err(CombatError::ConstraintViolation(
                        format!("At most {} creature(s) can attack", max),
                    ));
                }
            }
        }
    }

    // Check requirements — Phase 4/5 will implement the requirement-maximizing logic.
    // For now, just verify that required creatures are present if possible.
    for requirement in &constraints.requirements {
        match requirement {
            AttackRequirement::MustAttackIfAble(id) => {
                if !proposed.iter().any(|(cid, _)| cid == id) {
                    return Err(CombatError::ConstraintViolation(
                        format!("Creature {} must attack if able", id),
                    ));
                }
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Blocker validation (rule 509.1)
// ---------------------------------------------------------------------------

/// Validate a proposed set of blockers.
///
/// Checks per-creature legality (rule 509.1a) and set-level constraints.
/// `proposed` is a list of (blocker_id, attacker_id) pairs.
pub fn validate_blockers(
    game: &GameState,
    player_id: PlayerId,
    proposed: &[(ObjectId, ObjectId)],
    constraints: &BlockConstraints,
) -> Result<(), CombatError> {
    // Count how many times each blocker is used
    let mut block_counts: HashMap<ObjectId, usize> = HashMap::new();

    for (blocker_id, attacker_id) in proposed {
        // 1. Blocker must be on the battlefield
        let entry = game.battlefield.get(blocker_id)
            .ok_or(CombatError::NotOnBattlefield(*blocker_id))?;

        // 2. Must be a creature
        if !game.is_creature(*blocker_id) {
            return Err(CombatError::NotACreature(*blocker_id));
        }

        // 3. Controlled by the defending player
        if entry.controller != player_id {
            return Err(CombatError::NotControlledByPlayer(*blocker_id, player_id));
        }

        // 4. Must be untapped
        if entry.tapped {
            return Err(CombatError::CreatureIsTapped(*blocker_id));
        }

        // 5. The attacker must actually be attacking this player
        if let Some(attacker_entry) = game.battlefield.get(attacker_id) {
            if let Some(ref attacking_info) = attacker_entry.attacking {
                match &attacking_info.target {
                    AttackTarget::Player(pid) => {
                        if *pid != player_id {
                            return Err(CombatError::AttackerNotAttackingThisPlayer(*blocker_id, *attacker_id));
                        }
                    }
                    _ => {
                        // Phase 3: only player attacks supported
                        return Err(CombatError::AttackerNotAttackingThisPlayer(*blocker_id, *attacker_id));
                    }
                }
            } else {
                // Attacker isn't attacking at all
                return Err(CombatError::AttackerNotAttackingThisPlayer(*blocker_id, *attacker_id));
            }
        } else {
            return Err(CombatError::NotOnBattlefield(*attacker_id));
        }

        // 6. Count blocks per creature
        let count = block_counts.entry(*blocker_id).or_insert(0);
        *count += 1;
        let max = constraints.max_blocks_for(*blocker_id);
        if *count > max {
            return Err(CombatError::TooManyBlocks(*blocker_id, max));
        }
    }

    // Set-level constraint checks
    check_block_constraints(proposed, constraints)?;

    Ok(())
}

/// Check set-level block constraints.
///
/// Phase 3: no-op with `BlockConstraints::none()`.
fn check_block_constraints(
    proposed: &[(ObjectId, ObjectId)],
    constraints: &BlockConstraints,
) -> Result<(), CombatError> {
    for restriction in &constraints.restrictions {
        match restriction {
            BlockRestriction::CantBlock(id) => {
                if proposed.iter().any(|(bid, _)| bid == id) {
                    return Err(CombatError::ConstraintViolation(
                        format!("Creature {} can't block", id),
                    ));
                }
            }
            BlockRestriction::CantBlockUnless(id, condition) => {
                if proposed.iter().any(|(bid, _)| bid == id) {
                    return Err(CombatError::ConstraintViolation(
                        format!("Creature {} can't block unless {}", id, condition),
                    ));
                }
            }
        }
    }

    for requirement in &constraints.requirements {
        match requirement {
            BlockRequirement::MustBlockIfAble(id) => {
                if !proposed.iter().any(|(bid, _)| bid == id) {
                    return Err(CombatError::ConstraintViolation(
                        format!("Creature {} must block if able", id),
                    ));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::{AttackingInfo, BattlefieldEntity};
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;
    use crate::types::colors::Color;

    fn make_bears(owner: PlayerId) -> (ObjectId, std::sync::Arc<crate::objects::card_data::CardData>) {
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .mana_cost(ManaCost::single(ManaType::Green, 1, 1))
            .power_toughness(2, 2)
            .build();
        let obj = GameObject::new(data.clone(), owner, Zone::Battlefield);
        (obj.id, data)
    }

    /// Place a creature on the battlefield (not summoning sick).
    fn place_creature(game: &mut GameState, owner: PlayerId) -> ObjectId {
        let (id, data) = make_bears(owner);
        let mut obj = GameObject::new(data, owner, Zone::Battlefield);
        obj.id = id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = BattlefieldEntity::new(id, owner, ts);
        entry.summoning_sick = false; // simulate having been here since turn start
        game.battlefield.insert(id, entry);
        id
    }

    /// Place a creature that still has summoning sickness.
    fn place_creature_sick(game: &mut GameState, owner: PlayerId) -> ObjectId {
        let (id, data) = make_bears(owner);
        let mut obj = GameObject::new(data, owner, Zone::Battlefield);
        obj.id = id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let entry = BattlefieldEntity::new(id, owner, ts); // summoning_sick = true by default
        game.battlefield.insert(id, entry);
        id
    }

    // --- validate_attackers tests ---

    #[test]
    fn test_valid_attack() {
        let mut game = GameState::new(2, 20);
        let creature_id = place_creature(&mut game, 0);

        let result = validate_attackers(
            &game, 0,
            &[(creature_id, AttackTarget::Player(1))],
            &AttackConstraints::none(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_attack_empty_is_valid() {
        let game = GameState::new(2, 20);
        let result = validate_attackers(&game, 0, &[], &AttackConstraints::none());
        assert!(result.is_ok());
    }

    #[test]
    fn test_attack_not_a_creature() {
        let mut game = GameState::new(2, 20);
        // Place a non-creature (land) on the battlefield
        let data = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let ts = game.allocate_timestamp();
        let mut entry = BattlefieldEntity::new(id, 0, ts);
        entry.summoning_sick = false;
        game.battlefield.insert(id, entry);

        let result = validate_attackers(
            &game, 0,
            &[(id, AttackTarget::Player(1))],
            &AttackConstraints::none(),
        );
        assert_eq!(result, Err(CombatError::NotACreature(id)));
    }

    #[test]
    fn test_attack_wrong_controller() {
        let mut game = GameState::new(2, 20);
        let creature_id = place_creature(&mut game, 1); // owned by player 1

        let result = validate_attackers(
            &game, 0, // player 0 tries to attack with player 1's creature
            &[(creature_id, AttackTarget::Player(1))],
            &AttackConstraints::none(),
        );
        assert_eq!(result, Err(CombatError::NotControlledByPlayer(creature_id, 0)));
    }

    #[test]
    fn test_attack_tapped_creature() {
        let mut game = GameState::new(2, 20);
        let creature_id = place_creature(&mut game, 0);
        game.battlefield.get_mut(&creature_id).unwrap().tapped = true;

        let result = validate_attackers(
            &game, 0,
            &[(creature_id, AttackTarget::Player(1))],
            &AttackConstraints::none(),
        );
        assert_eq!(result, Err(CombatError::CreatureIsTapped(creature_id)));
    }

    #[test]
    fn test_attack_summoning_sick() {
        let mut game = GameState::new(2, 20);
        let creature_id = place_creature_sick(&mut game, 0);

        let result = validate_attackers(
            &game, 0,
            &[(creature_id, AttackTarget::Player(1))],
            &AttackConstraints::none(),
        );
        assert_eq!(result, Err(CombatError::CreatureHasSummoningSickness(creature_id)));
    }

    #[test]
    fn test_attack_self_invalid() {
        let mut game = GameState::new(2, 20);
        let creature_id = place_creature(&mut game, 0);

        let result = validate_attackers(
            &game, 0,
            &[(creature_id, AttackTarget::Player(0))], // attacking self
            &AttackConstraints::none(),
        );
        assert_eq!(result, Err(CombatError::InvalidAttackTarget(creature_id)));
    }

    #[test]
    fn test_attack_invalid_player_id() {
        let mut game = GameState::new(2, 20);
        let creature_id = place_creature(&mut game, 0);

        let result = validate_attackers(
            &game, 0,
            &[(creature_id, AttackTarget::Player(99))], // nonexistent player
            &AttackConstraints::none(),
        );
        assert_eq!(result, Err(CombatError::InvalidAttackTarget(creature_id)));
    }

    #[test]
    fn test_attack_multiple_creatures_valid() {
        let mut game = GameState::new(2, 20);
        let c1 = place_creature(&mut game, 0);
        let c2 = place_creature(&mut game, 0);

        let result = validate_attackers(
            &game, 0,
            &[(c1, AttackTarget::Player(1)), (c2, AttackTarget::Player(1))],
            &AttackConstraints::none(),
        );
        assert!(result.is_ok());
    }

    // --- validate_blockers tests ---

    /// Helper: set up a creature as attacking player 1
    fn set_attacking(game: &mut GameState, creature_id: ObjectId, target_player: PlayerId) {
        if let Some(entry) = game.battlefield.get_mut(&creature_id) {
            entry.attacking = Some(AttackingInfo {
                target: AttackTarget::Player(target_player),
                is_blocked: false,
                blocked_by: Vec::new(),
            });
        }
    }

    #[test]
    fn test_valid_block() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature(&mut game, 0);
        let blocker = place_creature(&mut game, 1);
        set_attacking(&mut game, attacker, 1);

        let result = validate_blockers(
            &game, 1,
            &[(blocker, attacker)],
            &BlockConstraints::none(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_block_empty_is_valid() {
        let game = GameState::new(2, 20);
        let result = validate_blockers(&game, 1, &[], &BlockConstraints::none());
        assert!(result.is_ok());
    }

    #[test]
    fn test_block_wrong_controller() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature(&mut game, 0);
        let blocker = place_creature(&mut game, 0); // controlled by player 0, not 1
        set_attacking(&mut game, attacker, 1);

        let result = validate_blockers(
            &game, 1,
            &[(blocker, attacker)],
            &BlockConstraints::none(),
        );
        assert_eq!(result, Err(CombatError::NotControlledByPlayer(blocker, 1)));
    }

    #[test]
    fn test_block_tapped_creature() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature(&mut game, 0);
        let blocker = place_creature(&mut game, 1);
        set_attacking(&mut game, attacker, 1);
        game.battlefield.get_mut(&blocker).unwrap().tapped = true;

        let result = validate_blockers(
            &game, 1,
            &[(blocker, attacker)],
            &BlockConstraints::none(),
        );
        assert_eq!(result, Err(CombatError::CreatureIsTapped(blocker)));
    }

    #[test]
    fn test_block_attacker_not_attacking_this_player() {
        let mut game = GameState::new(3, 20); // 3-player game
        let attacker = place_creature(&mut game, 0);
        let blocker = place_creature(&mut game, 1);
        set_attacking(&mut game, attacker, 2); // attacking player 2, not 1

        let result = validate_blockers(
            &game, 1,
            &[(blocker, attacker)],
            &BlockConstraints::none(),
        );
        assert_eq!(result, Err(CombatError::AttackerNotAttackingThisPlayer(blocker, attacker)));
    }

    #[test]
    fn test_block_same_creature_twice_rejected() {
        let mut game = GameState::new(2, 20);
        let att1 = place_creature(&mut game, 0);
        let att2 = place_creature(&mut game, 0);
        let blocker = place_creature(&mut game, 1);
        set_attacking(&mut game, att1, 1);
        set_attacking(&mut game, att2, 1);

        // Same blocker trying to block two different attackers (default max_blocks = 1)
        let result = validate_blockers(
            &game, 1,
            &[(blocker, att1), (blocker, att2)],
            &BlockConstraints::none(),
        );
        assert_eq!(result, Err(CombatError::TooManyBlocks(blocker, 1)));
    }

    #[test]
    fn test_block_same_creature_twice_allowed_with_limit() {
        let mut game = GameState::new(2, 20);
        let att1 = place_creature(&mut game, 0);
        let att2 = place_creature(&mut game, 0);
        let blocker = place_creature(&mut game, 1);
        set_attacking(&mut game, att1, 1);
        set_attacking(&mut game, att2, 1);

        let mut constraints = BlockConstraints::none();
        constraints.blocking_limits.insert(blocker, 2); // can block 2

        let result = validate_blockers(
            &game, 1,
            &[(blocker, att1), (blocker, att2)],
            &constraints,
        );
        assert!(result.is_ok());
    }
}
