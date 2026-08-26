// Non-combat keyword ability hooks.
//
// These functions handle keyword behaviors that trigger during damage
// resolution (lifelink, deathtouch) rather than during combat damage
// assignment. Called from perform_action in actions.rs.

use crate::events::event::DamageTarget;
use crate::oracle::characteristics::has_keyword;
use crate::engine::actions::{ActionContext, GameAction};
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
/// The gain is **proposed**, not written. Until RA-2 this function subtracted
/// into `life_total` and emitted `LifeChanged` itself — emitting without
/// proposing, which is exactly how a census of emission sites missed it. A
/// CR 614 life-gain watcher (Tainted Remedy: "If an opponent would gain life,
/// they lose that much life instead") must see lifelink, and could not.
///
/// **This is the first re-entrant `execute_action`.** It runs from inside
/// `perform_action(DealDamage)`, so the `GainLife` proposal nests inside a
/// performance already in flight. Harmless while the pipeline is a passthrough;
/// in RB it is a *contained* event with fresh lineage rather than a
/// decomposition of the damage (`replacement-architecture.md` §3.2d), and RD's
/// CR 120.3 results-of-damage decomposition generalizes exactly this shape.
pub fn apply_lifelink(
    game: &mut GameState,
    source: ObjectId,
    amount: u64,
    ctx: &ActionContext,
) -> Result<(), String> {
    if !has_keyword(game, source, KeywordFlag::Lifelink) {
        return Ok(());
    }
    // CR 702.15b gives the life to the source's controller — effective, so a
    // stolen lifelinker gains life for the thief.
    if let Some(controller) = crate::oracle::characteristics::get_effective_controller(game, source) {
        game.execute_action(
            GameAction::GainLife { player: controller, amount, source },
            ctx,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_ctx;
    use crate::events::event::GameEvent;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::BattlefieldEntity;
    use crate::types::card_types::CardType;
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;

    fn setup_creature(game: &mut GameState, keywords: &[KeywordFlag]) -> ObjectId {
        let mut builder = CardDataBuilder::new("Test Creature")
            .card_type(CardType::Creature)
            .mana_cost(ManaCost::build(&[ManaType::Green], 1))
            .power_toughness(2, 3);
        for kw in keywords {
            builder = builder.keyword(*kw);
        }
        let data = builder.build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = BattlefieldEntity::new(id, 0, 0, 1);
        game.battlefield.insert(id, entry);
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

    #[test]
    fn test_lifelink_gains_life() {
        let mut game = GameState::new(2, 20);
        let source = setup_creature(&mut game, &[KeywordFlag::Lifelink]);

        apply_lifelink(&mut game, source, 3, &test_ctx()).unwrap();
        assert_eq!(game.players[0].life_total, 23);
    }

    #[test]
    fn test_no_lifelink_no_gain() {
        let mut game = GameState::new(2, 20);
        let source = setup_creature(&mut game, &[]);

        apply_lifelink(&mut game, source, 3, &test_ctx()).unwrap();
        assert_eq!(game.players[0].life_total, 20);
    }

    #[test]
    fn test_lifelink_emits_event() {
        let mut game = GameState::new(2, 20);
        let source = setup_creature(&mut game, &[KeywordFlag::Lifelink]);

        apply_lifelink(&mut game, source, 2, &test_ctx()).unwrap();
        assert_eq!(game.events.len(), 1);
    }

    #[test]
    fn test_lifelink_gain_is_attributed_to_the_source() {
        let mut game = GameState::new(2, 20);
        let source = setup_creature(&mut game, &[KeywordFlag::Lifelink]);

        apply_lifelink(&mut game, source, 2, &test_ctx()).unwrap();

        // The gain now goes through execute_action(GainLife) rather than being
        // written straight into life_total. Nothing observable changes today —
        // the same LifeChanged comes out — which is the point: it is the
        // *proposal* a CR 614 watcher needs (Tainted Remedy must see lifelink),
        // and there was none. Locking the shape here so the routing cannot be
        // quietly undone.
        let emitted: Vec<&GameEvent> = game.events.events().collect();
        match emitted.as_slice() {
            [GameEvent::LifeChanged { player_id, old, new, source: src }] => {
                assert_eq!(*player_id, 0);
                assert_eq!((*old, *new), (20, 22));
                assert_eq!(*src, Some(source), "CR 702.15b attributes the gain to the lifelinker");
            }
            other => panic!("expected one LifeChanged, got {:?}", other),
        }
    }

    #[test]
    fn test_lifelinks_gain_joins_the_damage_batch() {
        // CR 702.15b — the life gain "happens simultaneously with that damage",
        // so the nested `execute_action` must join the damage's batch rather
        // than opening one of its own. Two batch ids would tell a CR 603.2c
        // trigger that two events happened.
        let mut game = GameState::new(2, 20);
        let source = crate::test_support::place_vanilla_creature(
            &mut game, 0, 2, 2, &[KeywordFlag::Lifelink]);

        game.execute_action(
            crate::engine::actions::GameAction::DealDamage {
                source,
                target: crate::events::event::DamageTarget::Player(1),
                amount: 2,
                is_combat: false,
            },
            &test_ctx(),
        ).unwrap();

        let batches: Vec<_> = game.events.records().iter().map(|r| r.batch()).collect();
        assert!(batches.len() >= 2, "damage plus the life it gains");
        let first = batches[0].expect("a performed action is in a batch");
        assert!(
            batches.iter().all(|b| *b == Some(first)),
            "lifelink's gain is simultaneous with the damage, so it shares its batch",
        );
    }
}
