// Non-combat keyword ability hooks.
//
// These functions handle keyword behaviors that trigger during damage
// resolution (lifelink, deathtouch) rather than during combat damage
// assignment. Called from perform_action in actions.rs.

use crate::events::event::{DamageTarget, GameEvent};
use crate::oracle::characteristics::has_keyword;
use crate::state::game_state::GameState;
use crate::types::ids::ObjectId;
use crate::types::keywords::KeywordAbility;

/// Apply the deathtouch flag to a damage target if the source has deathtouch.
///
/// Rule 702.2b: Any nonzero damage dealt by a source with deathtouch is
/// considered lethal for SBA purposes. We mark the target's
/// `damaged_by_deathtouch` flag, which is checked in SBA 704.5g and
/// cleared during cleanup (rule 514.2).
///
/// Returns Ok(()) always; the flag is only set if the target is on the
/// battlefield.
pub fn apply_deathtouch_flag(
    game: &mut GameState,
    source: ObjectId,
    target: &DamageTarget,
) {
    // Pre-check before mutable borrow (borrow checker: has_keyword reads objects)
    if !has_keyword(game, source, KeywordAbility::Deathtouch) {
        return;
    }
    if let DamageTarget::Object(id) = target {
        if let Some(entry) = game.battlefield.get_mut(id) {
            entry.damaged_by_deathtouch = true;
        }
    }
}

/// Apply lifelink: controller gains life equal to damage dealt.
///
/// Rule 702.15b: A source with lifelink causes its controller to gain
/// life equal to the damage dealt, simultaneously with that damage.
/// Multiple instances don't stack (rule 702.15f) — boolean check.
///
/// Emits a `LifeChanged` event for the life gain.
pub fn apply_lifelink(
    game: &mut GameState,
    source: ObjectId,
    amount: u64,
) -> Result<(), String> {
    if !has_keyword(game, source, KeywordAbility::Lifelink) {
        return Ok(());
    }
    if let Some(entry) = game.battlefield.get(&source) {
        let controller = entry.controller;
        let old_life = game.get_player(controller)?.life_total;
        let p = game.get_player_mut(controller)?;
        p.life_total += amount as i64;
        let new_life = p.life_total;
        game.events.emit(GameEvent::LifeChanged {
            player_id: controller,
            old: old_life,
            new: new_life,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::BattlefieldEntity;
    use crate::types::card_types::CardType;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;

    fn setup_creature(game: &mut GameState, keywords: &[KeywordAbility]) -> ObjectId {
        let mut builder = CardDataBuilder::new("Test Creature")
            .card_type(CardType::Creature)
            .mana_cost(ManaCost::single(ManaType::Green, 1, 1))
            .power_toughness(2, 3);
        for kw in keywords {
            builder = builder.keyword(*kw);
        }
        let data = builder.build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = BattlefieldEntity::new(id, 0, 0);
        game.battlefield.insert(id, entry);
        id
    }

    // --- Deathtouch flag tests ---

    #[test]
    fn test_deathtouch_flag_set_on_creature_target() {
        let mut game = GameState::new(2, 20);
        let source = setup_creature(&mut game, &[KeywordAbility::Deathtouch]);
        let target = setup_creature(&mut game, &[]);

        apply_deathtouch_flag(&mut game, source, &DamageTarget::Object(target));
        assert!(game.battlefield.get(&target).unwrap().damaged_by_deathtouch);
    }

    #[test]
    fn test_no_deathtouch_no_flag() {
        let mut game = GameState::new(2, 20);
        let source = setup_creature(&mut game, &[]); // no deathtouch
        let target = setup_creature(&mut game, &[]);

        apply_deathtouch_flag(&mut game, source, &DamageTarget::Object(target));
        assert!(!game.battlefield.get(&target).unwrap().damaged_by_deathtouch);
    }

    #[test]
    fn test_deathtouch_flag_ignored_for_player_target() {
        let mut game = GameState::new(2, 20);
        let source = setup_creature(&mut game, &[KeywordAbility::Deathtouch]);

        // Should not panic or error — just does nothing for player targets
        apply_deathtouch_flag(&mut game, source, &DamageTarget::Player(1));
    }

    // --- Lifelink tests ---

    #[test]
    fn test_lifelink_gains_life() {
        let mut game = GameState::new(2, 20);
        let source = setup_creature(&mut game, &[KeywordAbility::Lifelink]);

        apply_lifelink(&mut game, source, 3).unwrap();
        assert_eq!(game.players[0].life_total, 23);
    }

    #[test]
    fn test_no_lifelink_no_gain() {
        let mut game = GameState::new(2, 20);
        let source = setup_creature(&mut game, &[]);

        apply_lifelink(&mut game, source, 3).unwrap();
        assert_eq!(game.players[0].life_total, 20);
    }

    #[test]
    fn test_lifelink_emits_event() {
        let mut game = GameState::new(2, 20);
        let source = setup_creature(&mut game, &[KeywordAbility::Lifelink]);

        apply_lifelink(&mut game, source, 2).unwrap();
        assert_eq!(game.events.len(), 1);
    }
}
