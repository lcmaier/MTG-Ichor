//! Phase 4 integration tests — keyword abilities.
//!
//! Tests the full combat pipeline with keyword-bearing creatures:
//! flying evasion, reach, haste, vigilance, first/double strike,
//! trample, lifelink, deathtouch, and defender.

use std::sync::Arc;

use mtgsim::cards::creatures;
use mtgsim::cards::keyword_creatures;
use mtgsim::engine::combat::resolution::assign_combat_damage;
use mtgsim::engine::combat::validation::{
    validate_attackers, validate_blockers, AttackConstraints, BlockConstraints, CombatError,
};
use mtgsim::events::event::DamageTarget;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::state::battlefield::{AttackTarget, AttackingInfo, BattlefieldEntity, BlockingInfo};
use mtgsim::state::game_state::GameState;
use mtgsim::types::card_types::CardType;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordAbility;
use mtgsim::types::zones::Zone;
use mtgsim::ui::decision::{PassiveDecisionProvider, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn place_creature(
    game: &mut GameState,
    owner: PlayerId,
    card_factory: fn() -> Arc<CardData>,
) -> ObjectId {
    let data = card_factory();
    let obj = GameObject::new(data, owner, Zone::Battlefield);
    let id = obj.id;
    game.add_object(obj);
    let ts = game.allocate_timestamp();
    let entry = BattlefieldEntity::new(id, owner, ts, 0);
    game.battlefield.insert(id, entry);
    id
}

fn place_creature_sick(
    game: &mut GameState,
    owner: PlayerId,
    card_factory: fn() -> Arc<CardData>,
) -> ObjectId {
    let data = card_factory();
    let obj = GameObject::new(data, owner, Zone::Battlefield);
    let id = obj.id;
    game.add_object(obj);
    game.place_on_battlefield(id, owner); // entered this turn = summoning sick
    id
}

fn set_attacking(game: &mut GameState, id: ObjectId, target_player: PlayerId) {
    if let Some(entry) = game.battlefield.get_mut(&id) {
        entry.tapped = true;
        entry.attacking = Some(AttackingInfo {
            target: AttackTarget::Player(target_player),
            is_blocked: false,
            blocked_by: Vec::new(),
        });
    }
    game.attacks_declared = true;
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

// ---------------------------------------------------------------------------
// 1. Flying evasion: Serra Angel attacks, ground creature can't block
// ---------------------------------------------------------------------------

#[test]
fn test_flying_evasion_ground_cant_block() {
    let mut game = GameState::new(2, 20);
    let angel = place_creature(&mut game, 0, keyword_creatures::serra_angel);
    let bears = place_creature(&mut game, 1, creatures::grizzly_bears);
    set_attacking(&mut game, angel, 1);

    let result = validate_blockers(
        &game, 1,
        &[(bears, angel)],
        &BlockConstraints::none(),
    );
    assert_eq!(result, Err(CombatError::CantBlockFlyer(bears, angel)));
}

// ---------------------------------------------------------------------------
// 2. Reach blocks flyer: Giant Spider blocks Serra Angel
// ---------------------------------------------------------------------------

#[test]
fn test_reach_blocks_flyer() {
    let mut game = GameState::new(2, 20);
    let angel = place_creature(&mut game, 0, keyword_creatures::serra_angel);
    let spider = place_creature(&mut game, 1, keyword_creatures::giant_spider);
    set_attacking(&mut game, angel, 1);

    let result = validate_blockers(
        &game, 1,
        &[(spider, angel)],
        &BlockConstraints::none(),
    );
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// 3. Haste attacks immediately: Raging Cougar attacks on entry turn
// ---------------------------------------------------------------------------

#[test]
fn test_haste_attacks_immediately() {
    let mut game = GameState::new(2, 20);
    // Place with summoning sickness
    let cougar = place_creature_sick(&mut game, 0, keyword_creatures::raging_cougar);

    let result = validate_attackers(
        &game, 0,
        &[(cougar, AttackTarget::Player(1))],
        &AttackConstraints::none(),
    );
    // Should succeed because haste bypasses summoning sickness
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// 4. Vigilance doesn't tap: Serra Angel attacks, remains untapped
// ---------------------------------------------------------------------------

#[test]
fn test_vigilance_doesnt_tap() {
    let mut game = GameState::new(2, 20);
    let angel = place_creature(&mut game, 0, keyword_creatures::serra_angel);

    let scripted = ScriptedDecisionProvider::new();
    scripted.attack_decisions.borrow_mut().push(
        vec![(angel, AttackTarget::Player(1))],
    );
    game.process_declare_attackers(&scripted).unwrap();

    // Serra Angel should NOT be tapped (vigilance)
    assert!(!game.battlefield.get(&angel).unwrap().tapped);
    // But should be attacking
    assert!(game.battlefield.get(&angel).unwrap().attacking.is_some());
}

// ---------------------------------------------------------------------------
// 5. First strike kills first: Elvish Archers (2/1 FS) vs Grizzly Bears (2/2)
// ---------------------------------------------------------------------------

#[test]
fn test_first_strike_kills_before_normal_damage() {
    let mut game = GameState::new(2, 20);
    let archers = place_creature(&mut game, 0, keyword_creatures::elvish_archers);
    let bears = place_creature(&mut game, 1, creatures::grizzly_bears);
    set_attacking(&mut game, archers, 1);
    set_blocked_by(&mut game, archers, vec![bears]);
    set_blocking(&mut game, bears, vec![archers]);

    let passive = PassiveDecisionProvider;

    // First strike step: archers (2 power, FS) deal damage
    let assignments = assign_combat_damage(&game, &passive, 0, true);
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].source, archers);
    assert_eq!(assignments[0].target, DamageTarget::Object(bears));
    assert_eq!(assignments[0].amount, 2);

    // Apply first strike damage
    game.apply_combat_damage(assignments).unwrap();
    game.dealt_first_strike_damage.insert(archers);

    // Bears now have 2 damage marked on 2 toughness — SBA kills them
    game.check_state_based_actions(&mtgsim::ui::decision::PassiveDecisionProvider).unwrap();
    assert!(!game.battlefield.contains_key(&bears));

    // Normal damage step: archers already dealt (FS only), bears are dead
    let assignments = assign_combat_damage(&game, &passive, 0, false);
    // No damage from archers (already dealt FS, not double strike)
    // No damage from bears (dead)
    assert!(assignments.is_empty());

    // Archers survive!
    assert!(game.battlefield.contains_key(&archers));
}

// ---------------------------------------------------------------------------
// 6. Double strike twice: Ridgetop Raptor deals 2+2 = 4 total to player
// ---------------------------------------------------------------------------

#[test]
fn test_double_strike_deals_twice() {
    let mut game = GameState::new(2, 20);
    let raptor = place_creature(&mut game, 0, keyword_creatures::ridgetop_raptor);
    set_attacking(&mut game, raptor, 1);

    let passive = PassiveDecisionProvider;

    // First strike step
    let assignments = assign_combat_damage(&game, &passive, 0, true);
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].amount, 2);
    game.apply_combat_damage(assignments).unwrap();
    game.dealt_first_strike_damage.insert(raptor);

    assert_eq!(game.players[1].life_total, 18);

    // Normal damage step (double strike deals again)
    let assignments = assign_combat_damage(&game, &passive, 0, false);
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].amount, 2);
    game.apply_combat_damage(assignments).unwrap();

    assert_eq!(game.players[1].life_total, 16); // 20 - 2 - 2 = 16
}

// ---------------------------------------------------------------------------
// 7. Trample overflow: War Mammoth (3/3 trample) blocked by Savannah Lions (2/1)
// ---------------------------------------------------------------------------

#[test]
fn test_trample_overflow_to_player() {
    let mut game = GameState::new(2, 20);
    let mammoth = place_creature(&mut game, 0, keyword_creatures::war_mammoth);
    let lions = place_creature(&mut game, 1, creatures::savannah_lions);
    set_attacking(&mut game, mammoth, 1);
    set_blocked_by(&mut game, mammoth, vec![lions]);
    set_blocking(&mut game, lions, vec![mammoth]);

    let passive = PassiveDecisionProvider;
    let assignments = assign_combat_damage(&game, &passive, 0, false);

    // Mammoth (3 power, trample) vs Lions (2/1): 1 to lions (lethal), 2 tramples
    let to_lions: Vec<_> = assignments.iter()
        .filter(|a| a.source == mammoth && a.target == DamageTarget::Object(lions))
        .collect();
    let to_player: Vec<_> = assignments.iter()
        .filter(|a| a.source == mammoth && a.target == DamageTarget::Player(1))
        .collect();
    assert_eq!(to_lions.len(), 1);
    assert_eq!(to_lions[0].amount, 1);
    assert_eq!(to_player.len(), 1);
    assert_eq!(to_player[0].amount, 2);
}

// ---------------------------------------------------------------------------
// 8. Deathtouch trades up: Thornweald Archer (2/1 DT) vs Hill Giant (3/3)
// ---------------------------------------------------------------------------

#[test]
fn test_deathtouch_trades_up() {
    let mut game = GameState::new(2, 20);
    let archer = place_creature(&mut game, 0, keyword_creatures::thornweald_archer);
    let giant = place_creature(&mut game, 1, creatures::hill_giant);
    set_attacking(&mut game, archer, 1);
    set_blocked_by(&mut game, archer, vec![giant]);
    set_blocking(&mut game, giant, vec![archer]);

    let passive = PassiveDecisionProvider;
    let assignments = assign_combat_damage(&game, &passive, 0, false);

    // Apply all combat damage
    game.apply_combat_damage(assignments).unwrap();

    // Giant took 2 damage from deathtouch source → damaged_by_deathtouch = true
    assert!(game.battlefield.get(&giant).unwrap().damaged_by_deathtouch);
    assert_eq!(game.battlefield.get(&giant).unwrap().damage_marked, 2);

    // Archer took 3 damage on 1 toughness → lethal normally
    assert_eq!(game.battlefield.get(&archer).unwrap().damage_marked, 3);

    // SBA: both should die
    game.check_state_based_actions(&mtgsim::ui::decision::PassiveDecisionProvider).unwrap();
    assert!(!game.battlefield.contains_key(&archer));
    assert!(!game.battlefield.contains_key(&giant));
}

// ---------------------------------------------------------------------------
// 9. Lifelink heals: Vampire Nighthawk deals 2, controller gains 2
// ---------------------------------------------------------------------------

#[test]
fn test_lifelink_heals_on_combat_damage() {
    let mut game = GameState::new(2, 20);
    let nighthawk = place_creature(&mut game, 0, keyword_creatures::vampire_nighthawk);
    set_attacking(&mut game, nighthawk, 1);

    let passive = PassiveDecisionProvider;
    let assignments = assign_combat_damage(&game, &passive, 0, false);
    game.apply_combat_damage(assignments).unwrap();

    // Player 1 took 2 damage
    assert_eq!(game.players[1].life_total, 18);
    // Player 0 gained 2 life from lifelink
    assert_eq!(game.players[0].life_total, 22);
}

// ---------------------------------------------------------------------------
// 10. Defender can't attack: Wall of Stone can't declare as attacker
// ---------------------------------------------------------------------------

#[test]
fn test_defender_cant_attack() {
    let mut game = GameState::new(2, 20);
    let wall = place_creature(&mut game, 0, keyword_creatures::wall_of_stone);

    let result = validate_attackers(
        &game, 0,
        &[(wall, AttackTarget::Player(1))],
        &AttackConstraints::none(),
    );
    assert_eq!(result, Err(CombatError::HasDefender(wall)));
}

// ---------------------------------------------------------------------------
// 11. Defender blocks normally: Wall of Stone blocks
// ---------------------------------------------------------------------------

#[test]
fn test_defender_blocks_normally() {
    let mut game = GameState::new(2, 20);
    let bears = place_creature(&mut game, 0, creatures::grizzly_bears);
    let wall = place_creature(&mut game, 1, keyword_creatures::wall_of_stone);
    set_attacking(&mut game, bears, 1);

    let result = validate_blockers(
        &game, 1,
        &[(wall, bears)],
        &BlockConstraints::none(),
    );
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// 12. Trample + deathtouch: ad-hoc 4/4 trampler+DT blocked by 2/2
// ---------------------------------------------------------------------------

#[test]
fn test_trample_with_deathtouch_maximum_overflow() {
    let mut game = GameState::new(2, 20);

    // Ad-hoc 4/4 with trample + deathtouch
    let data = CardDataBuilder::new("Test Trampler")
        .card_type(CardType::Creature)
        .power_toughness(4, 4)
        .keyword(KeywordAbility::Trample)
        .keyword(KeywordAbility::Deathtouch)
        .build();
    let obj = GameObject::new(data, 0, Zone::Battlefield);
    let trampler = obj.id;
    game.add_object(obj);
    let ts = game.allocate_timestamp();
    let entry = BattlefieldEntity::new(trampler, 0, ts, 0);
    game.battlefield.insert(trampler, entry);

    let blocker = place_creature(&mut game, 1, creatures::grizzly_bears); // 2/2
    set_attacking(&mut game, trampler, 1);
    set_blocked_by(&mut game, trampler, vec![blocker]);
    set_blocking(&mut game, blocker, vec![trampler]);

    let passive = PassiveDecisionProvider;
    let assignments = assign_combat_damage(&game, &passive, 0, false);

    // Deathtouch: 1 damage is lethal. 4 power → 1 to blocker, 3 tramples to player
    let to_blocker: Vec<_> = assignments.iter()
        .filter(|a| a.source == trampler && a.target == DamageTarget::Object(blocker))
        .collect();
    let to_player: Vec<_> = assignments.iter()
        .filter(|a| a.source == trampler && a.target == DamageTarget::Player(1))
        .collect();
    assert_eq!(to_blocker.len(), 1);
    assert_eq!(to_blocker[0].amount, 1);
    assert_eq!(to_player.len(), 1);
    assert_eq!(to_player[0].amount, 3);

    // Apply and check SBA
    game.apply_combat_damage(assignments).unwrap();
    game.check_state_based_actions(&mtgsim::ui::decision::PassiveDecisionProvider).unwrap();

    // Blocker should be dead (1 deathtouch damage)
    assert!(!game.battlefield.contains_key(&blocker));
    // Player took 3
    assert_eq!(game.players[1].life_total, 17);
}
