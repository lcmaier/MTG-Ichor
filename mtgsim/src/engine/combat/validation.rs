// Combat validation — attacker and blocker legality checks.
// See rules 508.1 (attackers) and 509.1 (blockers).

use std::collections::HashMap;

use crate::oracle::characteristics::{has_keyword, is_creature};
use crate::oracle::legality::can_attack;
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::keywords::KeywordAbility;

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
    HasDefender(ObjectId),
    CantBlockFlyer(ObjectId, ObjectId),
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
            CombatError::HasDefender(id) => write!(f, "Creature {} has defender and can't attack", id),
            CombatError::CantBlockFlyer(blocker, attacker) => {
                write!(f, "Creature {} can't block flyer {} (no flying or reach)", blocker, attacker)
            }
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
        if !is_creature(game, *creature_id) {
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
        if !can_attack(game, *creature_id) {
            return Err(CombatError::CreatureHasSummoningSickness(*creature_id));
        }

        // 6. Defender check (rule 702.3b)
        if has_keyword(game, *creature_id, KeywordAbility::Defender) {
            return Err(CombatError::HasDefender(*creature_id));
        }

        // 7. Attack target must be valid
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
    check_attack_set_constraints(proposed, constraints)?;

    Ok(())
}

/// Check set-level attack constraints (restrictions and requirements).
///
/// Phase 3: this is a no-op when constraints is `AttackConstraints::none()`.
/// Phase 4/5 will populate constraints from keywords and continuous effects.
fn check_attack_set_constraints(
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
// Per-pair blocker legality (SPECIAL-8)
// ---------------------------------------------------------------------------

/// Check whether a given `(blocker, attacker)` pair is *hard-legal* — i.e.
/// the pair passes every rule that depends only on the two creatures and
/// the current attack target (flying/reach evasion, attacker actually
/// attacking `defender`, blocker untapped, both on battlefield, etc.).
///
/// Per-creature uniqueness (CR 509.1 — "a creature can't block more than
/// one attacker unless it has an ability such as menace") is **set-level**
/// and therefore is NOT checked here. That remains in `validate_blockers`.
///
/// Used by `process_declare_blockers` to pre-filter the cross product of
/// blocker × attacker pairs before prompting the DP, so the DP never sees
/// pairs that are illegal regardless of strategy.
///
/// References: CR 509.1a, 509.1b, 702.9b (flying), 702.17b (reach).
pub fn can_block(
    game: &GameState,
    defender: PlayerId,
    blocker_id: ObjectId,
    attacker_id: ObjectId,
) -> Result<(), CombatError> {
    let entry = game.battlefield.get(&blocker_id)
        .ok_or(CombatError::NotOnBattlefield(blocker_id))?;
    if !is_creature(game, blocker_id) {
        return Err(CombatError::NotACreature(blocker_id));
    }
    if entry.controller != defender {
        return Err(CombatError::NotControlledByPlayer(blocker_id, defender));
    }
    if entry.tapped {
        return Err(CombatError::CreatureIsTapped(blocker_id));
    }

    // Attacker must be on the battlefield and attacking this defender.
    let att_entry = game.battlefield.get(&attacker_id)
        .ok_or(CombatError::NotOnBattlefield(attacker_id))?;
    let attacking_info = att_entry.attacking.as_ref()
        .ok_or(CombatError::AttackerNotAttackingThisPlayer(blocker_id, attacker_id))?;
    match &attacking_info.target {
        AttackTarget::Player(pid) if *pid == defender => {}
        _ => {
            return Err(CombatError::AttackerNotAttackingThisPlayer(blocker_id, attacker_id));
        }
    }

    // Flying evasion (rule 702.9b / 702.17b).
    if has_keyword(game, attacker_id, KeywordAbility::Flying)
        && !has_keyword(game, blocker_id, KeywordAbility::Flying)
        && !has_keyword(game, blocker_id, KeywordAbility::Reach)
    {
        return Err(CombatError::CantBlockFlyer(blocker_id, attacker_id));
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
        // Per-pair hard legality (shared with the pre-filter in
        // `process_declare_blockers`): zone/type/controller/tap + attacker
        // actually attacking `player_id` + flying/reach evasion.
        can_block(game, player_id, *blocker_id, *attacker_id)?;

        // Set-level: count blocks per creature (CR 509.1).
        let count = block_counts.entry(*blocker_id).or_insert(0);
        *count += 1;
        let max = constraints.max_blocks_for(*blocker_id);
        if *count > max {
            return Err(CombatError::TooManyBlocks(*blocker_id, max));
        }
    }

    // Set-level constraint checks
    check_block_set_constraints(proposed, constraints)?;

    Ok(())
}

/// Check set-level block constraints.
///
/// Phase 3: no-op with `BlockConstraints::none()`.
fn check_block_set_constraints(
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
    use crate::oracle::characteristics::has_keyword;
    use crate::state::battlefield::{AttackingInfo, BattlefieldEntity};
    use crate::types::card_types::CardType;
    use crate::types::keywords::KeywordAbility;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;
    use crate::types::colors::Color;

    fn make_bears(owner: PlayerId) -> (ObjectId, std::sync::Arc<crate::objects::card_data::CardData>) {
        let data = CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(Color::Green)
            .mana_cost(ManaCost::build(&[ManaType::Green], 1))
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
        let entry = BattlefieldEntity::new(id, owner, ts, 0);
        game.battlefield.insert(id, entry);
        id
    }

    /// Place a creature that still has summoning sickness.
    fn place_creature_sick(game: &mut GameState, owner: PlayerId) -> ObjectId {
        let (id, data) = make_bears(owner);
        let mut obj = GameObject::new(data, owner, Zone::Battlefield);
        obj.id = id;
        game.add_object(obj);
        game.place_on_battlefield(id, owner); // entered this turn = summoning sick
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
        let entry = BattlefieldEntity::new(id, 0, ts, 0);
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

    /// Place a creature with specific keywords on the battlefield (not summoning sick).
    fn place_creature_with_keywords(
        game: &mut GameState,
        owner: PlayerId,
        keywords: &[KeywordAbility],
        power: i32,
        toughness: i32,
    ) -> ObjectId {
        let mut builder = CardDataBuilder::new("Test Creature")
            .card_type(CardType::Creature)
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

    // --- has_keyword tests ---

    #[test]
    fn test_has_keyword_true() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Serra Angel")
            .card_type(CardType::Creature)
            .color(Color::White)
            .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 3))
            .power_toughness(4, 4)
            .keyword(KeywordAbility::Flying)
            .keyword(KeywordAbility::Vigilance)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);

        assert!(has_keyword(&game, id, KeywordAbility::Flying));
        assert!(has_keyword(&game, id, KeywordAbility::Vigilance));
    }

    #[test]
    fn test_has_keyword_false() {
        let mut game = GameState::new(2, 20);
        let creature_id = place_creature(&mut game, 0); // Grizzly Bears — no keywords

        assert!(!has_keyword(&game, creature_id, KeywordAbility::Flying));
        assert!(!has_keyword(&game, creature_id, KeywordAbility::Haste));
        assert!(!has_keyword(&game, creature_id, KeywordAbility::Trample));
    }

    // --- Flying / Reach tests (4b) ---

    #[test]
    fn test_ground_creature_cant_block_flyer() {
        let mut game = GameState::new(2, 20);
        let flyer = place_creature_with_keywords(&mut game, 0, &[KeywordAbility::Flying], 4, 4);
        let ground = place_creature(&mut game, 1);
        set_attacking(&mut game, flyer, 1);

        let result = validate_blockers(
            &game, 1,
            &[(ground, flyer)],
            &BlockConstraints::none(),
        );
        assert_eq!(result, Err(CombatError::CantBlockFlyer(ground, flyer)));
    }

    #[test]
    fn test_flyer_can_block_flyer() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature_with_keywords(&mut game, 0, &[KeywordAbility::Flying], 4, 4);
        let blocker = place_creature_with_keywords(&mut game, 1, &[KeywordAbility::Flying], 2, 2);
        set_attacking(&mut game, attacker, 1);

        let result = validate_blockers(
            &game, 1,
            &[(blocker, attacker)],
            &BlockConstraints::none(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_reach_can_block_flyer() {
        let mut game = GameState::new(2, 20);
        let flyer = place_creature_with_keywords(&mut game, 0, &[KeywordAbility::Flying], 4, 4);
        let spider = place_creature_with_keywords(&mut game, 1, &[KeywordAbility::Reach], 2, 4);
        set_attacking(&mut game, flyer, 1);

        let result = validate_blockers(
            &game, 1,
            &[(spider, flyer)],
            &BlockConstraints::none(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_ground_vs_ground_blocking_unaffected() {
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

    // --- Defender tests (4c) ---

    #[test]
    fn test_defender_cant_attack() {
        let mut game = GameState::new(2, 20);
        let wall = place_creature_with_keywords(&mut game, 0, &[KeywordAbility::Defender], 0, 8);

        let result = validate_attackers(
            &game, 0,
            &[(wall, AttackTarget::Player(1))],
            &AttackConstraints::none(),
        );
        assert_eq!(result, Err(CombatError::HasDefender(wall)));
    }

    // --- Haste tests (4d) ---

    #[test]
    fn test_haste_creature_can_attack_while_summoning_sick() {
        let mut game = GameState::new(2, 20);
        // Place creature with haste that still has summoning sickness
        let data = CardDataBuilder::new("Raging Cougar")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .keyword(KeywordAbility::Haste)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0); // entered this turn = summoning sick

        let result = validate_attackers(
            &game, 0,
            &[(id, AttackTarget::Player(1))],
            &AttackConstraints::none(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_haste_still_cant_attack_while_summoning_sick() {
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
    fn test_haste_creature_can_tap_for_ability_while_summoning_sick() {
        let mut game = GameState::new(2, 20);
        let data = CardDataBuilder::new("Haste Tapper")
            .card_type(CardType::Creature)
            .power_toughness(1, 1)
            .keyword(KeywordAbility::Haste)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0); // entered this turn = summoning sick

        // Should be able to pay tap cost despite summoning sickness
        let result = game.can_pay_costs(
            &[crate::types::costs::Cost::Tap],
            0,
            id,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_defender_can_block() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature(&mut game, 0);
        let wall = place_creature_with_keywords(&mut game, 1, &[KeywordAbility::Defender], 0, 8);
        set_attacking(&mut game, attacker, 1);

        let result = validate_blockers(
            &game, 1,
            &[(wall, attacker)],
            &BlockConstraints::none(),
        );
        assert!(result.is_ok());
    }

    // --- Vigilance tests (4e) ---

    #[test]
    fn test_vigilance_creature_doesnt_tap_when_attacking() {
        let mut game = GameState::new(2, 20);
        let angel = place_creature_with_keywords(
            &mut game, 0,
            &[KeywordAbility::Flying, KeywordAbility::Vigilance], 4, 4,
        );

        let scripted = crate::ui::decision::ScriptedDecisionProvider::new();
        // Legal pairs: [(angel, Player(1))] — index 0
        scripted.expect_pick_n(
            crate::ui::choice_types::ChoiceKind::DeclareAttackers,
            vec![0],
        );
        game.process_declare_attackers(&scripted).unwrap();

        // Angel should NOT be tapped
        assert!(!game.battlefield.get(&angel).unwrap().tapped);
        // But should be attacking
        assert!(game.battlefield.get(&angel).unwrap().attacking.is_some());
    }

    #[test]
    fn test_non_vigilance_creature_still_taps_when_attacking() {
        let mut game = GameState::new(2, 20);
        let bears = place_creature(&mut game, 0);

        let scripted = crate::ui::decision::ScriptedDecisionProvider::new();
        // Legal pairs: [(bears, Player(1))] — index 0
        scripted.expect_pick_n(
            crate::ui::choice_types::ChoiceKind::DeclareAttackers,
            vec![0],
        );
        game.process_declare_attackers(&scripted).unwrap();

        // Bears should be tapped
        assert!(game.battlefield.get(&bears).unwrap().tapped);
        assert!(game.battlefield.get(&bears).unwrap().attacking.is_some());
    }

    // --- can_block per-pair pre-filter tests (SPECIAL-8 / CR 509.1) ---

    #[test]
    fn test_can_block_basic_ok() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature(&mut game, 0);
        let blocker = place_creature(&mut game, 1);
        set_attacking(&mut game, attacker, 1);

        assert!(can_block(&game, 1, blocker, attacker).is_ok());
    }

    #[test]
    fn test_can_block_ground_vs_flyer_rejected() {
        let mut game = GameState::new(2, 20);
        let flyer = place_creature_with_keywords(&mut game, 0, &[KeywordAbility::Flying], 2, 2);
        let ground = place_creature(&mut game, 1);
        set_attacking(&mut game, flyer, 1);

        assert_eq!(
            can_block(&game, 1, ground, flyer),
            Err(CombatError::CantBlockFlyer(ground, flyer)),
        );
    }

    #[test]
    fn test_can_block_reach_blocks_flyer() {
        let mut game = GameState::new(2, 20);
        let flyer = place_creature_with_keywords(&mut game, 0, &[KeywordAbility::Flying], 2, 2);
        let spider = place_creature_with_keywords(&mut game, 1, &[KeywordAbility::Reach], 1, 3);
        set_attacking(&mut game, flyer, 1);

        assert!(can_block(&game, 1, spider, flyer).is_ok());
    }

    #[test]
    fn test_can_block_flyer_blocks_flyer() {
        let mut game = GameState::new(2, 20);
        let a = place_creature_with_keywords(&mut game, 0, &[KeywordAbility::Flying], 2, 2);
        let b = place_creature_with_keywords(&mut game, 1, &[KeywordAbility::Flying], 2, 2);
        set_attacking(&mut game, a, 1);

        assert!(can_block(&game, 1, b, a).is_ok());
    }

    #[test]
    fn test_can_block_attacker_not_attacking_rejected() {
        let mut game = GameState::new(2, 20);
        let not_attacking = place_creature(&mut game, 0);
        let blocker = place_creature(&mut game, 1);
        // `not_attacking` never calls set_attacking — it's not in combat.

        assert_eq!(
            can_block(&game, 1, blocker, not_attacking),
            Err(CombatError::AttackerNotAttackingThisPlayer(blocker, not_attacking)),
        );
    }

    #[test]
    fn test_can_block_attacker_attacking_other_defender_rejected() {
        let mut game = GameState::new(3, 20);
        let attacker = place_creature(&mut game, 0);
        let blocker = place_creature(&mut game, 1);
        set_attacking(&mut game, attacker, 2); // attacking player 2, not 1

        assert_eq!(
            can_block(&game, 1, blocker, attacker),
            Err(CombatError::AttackerNotAttackingThisPlayer(blocker, attacker)),
        );
    }

    #[test]
    fn test_can_block_tapped_blocker_rejected() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature(&mut game, 0);
        let blocker = place_creature(&mut game, 1);
        set_attacking(&mut game, attacker, 1);
        game.battlefield.get_mut(&blocker).unwrap().tapped = true;

        assert_eq!(
            can_block(&game, 1, blocker, attacker),
            Err(CombatError::CreatureIsTapped(blocker)),
        );
    }

    #[test]
    fn test_can_block_wrong_controller_rejected() {
        let mut game = GameState::new(2, 20);
        let attacker = place_creature(&mut game, 0);
        let own_creature = place_creature(&mut game, 0); // controlled by attacker's player
        set_attacking(&mut game, attacker, 1);

        assert_eq!(
            can_block(&game, 1, own_creature, attacker),
            Err(CombatError::NotControlledByPlayer(own_creature, 1)),
        );
    }

    // --- CR 509.1c retry loop test (SPECIAL-8) ---

    #[test]
    fn test_declare_blockers_retries_on_invalid_proposal() {
        // Scenario: defender has one blocker, attacker has two creatures in
        // combat. DP first proposes blocker-blocks-both (duplicate — violates
        // default 1-block-per-creature rule), then proposes a legal single
        // block. Retry loop must accept the second proposal.
        let mut game = GameState::new(2, 20);
        let att1 = place_creature(&mut game, 0);
        let att2 = place_creature(&mut game, 0);
        let blocker = place_creature(&mut game, 1);
        set_attacking(&mut game, att1, 1);
        set_attacking(&mut game, att2, 1);

        // legal_block_pairs will be ordered by HashMap iteration — we can
        // derive the index-of-each-pair at runtime to build the DP script.
        let blocker_ids = crate::oracle::legality::legal_blockers(&game, 1);
        let attackers_in_combat: Vec<ObjectId> = game.battlefield.iter()
            .filter_map(|(id, e)| e.attacking.as_ref().map(|_| *id))
            .collect();
        let pairs: Vec<(ObjectId, ObjectId)> = blocker_ids
            .iter()
            .flat_map(|&bid| attackers_in_combat.iter().map(move |&aid| (bid, aid)))
            .filter(|&(bid, aid)| can_block(&game, 1, bid, aid).is_ok())
            .collect();
        assert_eq!(pairs.len(), 2, "expected 2 legal pairs (blocker × 2 attackers)");
        let idx_att1 = pairs.iter().position(|p| *p == (blocker, att1)).unwrap();

        let scripted = crate::ui::decision::ScriptedDecisionProvider::new();
        // Invalid: pick both pairs — blocker is used twice.
        scripted.expect_pick_n(
            crate::ui::choice_types::ChoiceKind::DeclareBlockers,
            vec![0, 1],
        );
        // Retry: pick just the pair where blocker blocks att1.
        scripted.expect_pick_n(
            crate::ui::choice_types::ChoiceKind::DeclareBlockers,
            vec![idx_att1],
        );

        game.process_declare_blockers(&scripted).unwrap();

        // Blocker should be blocking att1 only.
        let binfo = game.battlefield.get(&blocker).unwrap().blocking.as_ref().unwrap();
        assert_eq!(binfo.blocking, vec![att1]);
        // att2 should not have been blocked.
        let att2_info = game.battlefield.get(&att2).unwrap().attacking.as_ref().unwrap();
        assert!(!att2_info.is_blocked);
    }
}
