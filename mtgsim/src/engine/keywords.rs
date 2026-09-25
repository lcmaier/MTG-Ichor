// Non-combat keyword ability hooks.
//
// These functions handle keyword behaviors that trigger during damage
// resolution (lifelink, deathtouch) rather than during combat damage
// assignment. Called from perform_action in actions.rs.

use crate::events::event::DamageTarget;
use crate::oracle::characteristics::has_keyword;
use crate::state::game_state::GameState;
use crate::types::ids::ObjectId;
use crate::types::keywords::KeywordFlag;

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
    if !has_keyword(game, source, KeywordFlag::Deathtouch) {
        return;
    }
    if let DamageTarget::Object(id) = target
        && let Some(entry) = game.battlefield.get_mut(id) {
        entry.damaged_by_deathtouch = true;
    }
}

/// One lifelink source's damage in a batch: who gains, and how much (CR
/// 702.15e).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifelinkGain {
    pub source: ObjectId,
    pub player: crate::types::ids::PlayerId,
    pub amount: u64,
}

/// Add `amount` of damage `source` has just dealt to `gains`, if it has
/// lifelink. CR 702.15b / 120.3f: the gain is one of the damage's results.
/// CR 702.15e: sources dealing damage at the same time cause separate
/// life-gain events, so one source's damage to several recipients at once
/// sums into one gain (Nykthos Paragon's sixth ruling). Multiple instances are
/// redundant (CR 702.15f). Who gains is read as the damage is dealt: the
/// source's controller, or its owner if it has none.
pub fn add_lifelink_gain(game: &GameState, gains: &mut Vec<LifelinkGain>, source: ObjectId, amount: u64) {
    if !has_keyword(game, source, KeywordFlag::Lifelink) {
        return;
    }
    let Some(player) = crate::oracle::characteristics::get_effective_controller(game, source)
        .or_else(|| game.objects.get(&source).map(|obj| obj.owner))
    else {
        return;
    };
    match gains.iter_mut().find(|gain| gain.source == source) {
        Some(gain) => gain.amount += amount,
        None => gains.push(LifelinkGain { source, player, amount }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::actions::GameAction;
    use crate::test_support::test_ctx;
    use crate::events::event::GameEvent;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::PermanentState;
    use crate::types::card_types::CardType;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;

    fn setup_creature(game: &mut GameState, keywords: &[KeywordFlag]) -> ObjectId {
        let mut builder = CardDataBuilder::new("Test Creature")
            .card_type(CardType::Creature)
            .mana_cost(ManaCost::build(&[ManaType::Green], 1))
            .power_toughness(2, 3);
        for kw in keywords {
            builder = builder.keyword_flag(*kw);
        }
        let data = builder.build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
        let entry = PermanentState::new(id, 0, 1);
        game.insert_battlefield_entity(id, entry);
        id
    }

    // --- Deathtouch flag tests ---

    #[test]
    fn test_deathtouch_flag_set_on_creature_target() {
        let mut game = GameState::new(2, 20);
        let source = setup_creature(&mut game, &[KeywordFlag::Deathtouch]);
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
        let source = setup_creature(&mut game, &[KeywordFlag::Deathtouch]);

        // Should not panic or error — just does nothing for player targets
        apply_deathtouch_flag(&mut game, source, &DamageTarget::Player(1));
    }

    // --- Lifelink tests ---

    /// `source` deals `amount` to player 1, through the chokepoint.
    fn deal_to_player_one(game: &mut GameState, source: ObjectId, amount: u64) {
        game.execute_action(
            GameAction::DealDamage {
                source,
                target: DamageTarget::Player(1),
                amount,
                is_combat: false,
                unpreventable: false,
            },
            &test_ctx(),
        )
        .unwrap();
    }

    #[test]
    fn test_lifelink_gains_life() {
        let mut game = GameState::new(2, 20);
        let source = setup_creature(&mut game, &[KeywordFlag::Lifelink]);

        deal_to_player_one(&mut game, source, 3);
        assert_eq!(game.players[0].life_total, 23);
    }

    #[test]
    fn test_no_lifelink_no_gain() {
        let mut game = GameState::new(2, 20);
        let source = setup_creature(&mut game, &[]);

        deal_to_player_one(&mut game, source, 3);
        assert_eq!(game.players[0].life_total, 20);
    }

    #[test]
    fn test_lifelink_gain_is_attributed_to_the_source() {
        let mut game = GameState::new(2, 20);
        game.record_events();
        let source = setup_creature(&mut game, &[KeywordFlag::Lifelink]);

        deal_to_player_one(&mut game, source, 2);

        // The gain goes through a `GainLife` proposal rather than being written
        // into life_total, so a CR 614 watcher (Tainted Remedy) sees it.
        let recorded = game.recorded_events();
        let gains: Vec<&GameEvent> = recorded
            .events()
            .filter(|e| matches!(e, GameEvent::LifeChanged { player_id: 0, .. }))
            .collect();
        match gains.as_slice() {
            [GameEvent::LifeChanged { old, new, source: src, .. }] => {
                assert_eq!((*old, *new), (20, 22));
                assert_eq!(*src, Some(source), "CR 702.15b attributes the gain to the lifelinker");
            }
            other => panic!("expected one gain for player 0, got {:?}", other),
        }
    }

    #[test]
    fn test_lifelinks_gain_joins_the_damage_batch() {
        // CR 120.3f makes the life gain one of the damage's *results*, and
        // CR 120.4c/d process the results and then let the one damage event
        // occur. So the nested `execute_action` must join the damage's batch
        // rather than opening one of its own: two batch ids would tell a
        // CR 603.2c trigger that two events happened.
        let mut game = GameState::new(2, 20);
        game.record_events();
        let source = crate::test_support::place_vanilla_creature(
            &mut game, 0, 2, 2, &[KeywordFlag::Lifelink]);

        game.execute_action(
            crate::engine::actions::GameAction::DealDamage {
                source,
                target: crate::events::event::DamageTarget::Player(1),
                amount: 2,
                is_combat: false,
                unpreventable: false
            },
            &test_ctx(),
        ).unwrap();

        let batches: Vec<_> = game.recorded_events().records().iter().map(|r| r.batch()).collect();
        assert!(batches.len() >= 2, "damage plus the life it gains");
        let first = batches[0].expect("a performed action is in a batch");
        assert!(
            batches.iter().all(|b| *b == Some(first)),
            "lifelink's gain is simultaneous with the damage, so it shares its batch",
        );
    }
}
