use crate::engine::keywords::{apply_deathtouch_flag, apply_lifelink};
use crate::events::event::{DamageTarget, GameEvent};
use crate::state::game_state::GameState;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::zones::Zone;

/// A game action that is *about to happen*.
///
/// This is the pre-mutation counterpart to `GameEvent` (which records what
/// *did* happen). The engine builds a `GameAction`, passes it through
/// `execute_action`, which performs the mutation and emits the corresponding
/// `GameEvent`.
///
/// In Phase 6, a replacement-effect pipeline will sit between "build action"
/// and "execute action", potentially modifying or replacing the action before
/// it is carried out. For now, `execute_action` is a direct passthrough.
#[derive(Debug, Clone)]
pub enum GameAction {
    /// Deal damage from a source to a target.
    DealDamage {
        source: ObjectId,
        target: DamageTarget,
        amount: u64,
        is_combat: bool,
    },

    /// A single card draw for a player.
    ///
    /// Drawing N cards is N individual `DrawCard` actions (rule 121.2).
    DrawCard {
        player: PlayerId,
    },

    /// A player gains life.
    GainLife {
        player: PlayerId,
        amount: u64,
        source: ObjectId,
    },

    /// A player loses life (not from damage).
    LoseLife {
        player: PlayerId,
        amount: u64,
    },

    /// Move an object from one zone to another.
    ZoneChange {
        object: ObjectId,
        from: Zone,
        to: Zone,
    },

    /// Untap a permanent.
    Untap {
        object: ObjectId,
    },

    /// Tap a permanent.
    Tap {
        object: ObjectId,
    },

    // === Phase 3+ actions — add variants here as primitives are implemented ===
    // Sacrifice { object: ObjectId },
    // Exile { object: ObjectId },
    // CreateToken { def: TokenDef, controller: PlayerId, count: u32 },
    // AddCounters { target: ObjectId, counter_type: CounterType, count: u32 },
    // etc.
}

impl GameState {
    /// Execute a game action: mutate state and emit the corresponding event.
    ///
    /// This is the central chokepoint for all game-state mutations that are
    /// observable (i.e., that triggered abilities and replacement effects care
    /// about).
    ///
    /// **Current behavior (pre-Phase 6):** direct passthrough — performs the
    /// mutation immediately and emits the event.
    ///
    /// **Phase 6:** A `apply_replacement_effects(action)` call will be inserted
    /// here, potentially modifying or replacing the action before execution.
    /// The replacement pipeline handles rule 614 (replacement effects),
    /// rule 615 (prevention effects), and rule 616 (interaction ordering).
    pub fn execute_action(&mut self, action: GameAction) -> Result<(), String> {
        // Phase 6: let action = self.apply_replacement_effects(action, decisions)?;
        self.perform_action(action)
    }

    /// Perform the actual state mutation and emit the event.
    ///
    /// This is separated from `execute_action` so that the replacement pipeline
    /// (Phase 6) can call this with the final, possibly-modified action.
    fn perform_action(&mut self, action: GameAction) -> Result<(), String> {
        match action {
            GameAction::DealDamage { source, target, amount, .. } => {
                if amount == 0 {
                    // Rule 614.7a: 0 damage is not dealt at all.
                    return Ok(());
                }

                match &target {
                    DamageTarget::Object(id) => {
                        if let Some(entry) = self.battlefield.get_mut(id) {
                            entry.damage_marked += amount as u32;
                        } else {
                            return Err(format!(
                                "Target object {} not on battlefield", id
                            ));
                        }
                    }
                    DamageTarget::Player(pid) => {
                        let player = self.get_player_mut(*pid)?;
                        player.life_total -= amount as i64;
                    }
                }

                // Keyword hooks (delegated to engine/keywords.rs)
                apply_deathtouch_flag(self, source, &target);
                apply_lifelink(self, source, amount)?;

                self.events.emit(GameEvent::DamageDealt {
                    source_id: source,
                    target: target.clone(),
                    amount,
                });

                // Emit LifeChanged for player damage
                if let DamageTarget::Player(pid) = &target {
                    let new_life = self.get_player(*pid)?.life_total;
                    self.events.emit(GameEvent::LifeChanged {
                        player_id: *pid,
                        old: new_life + amount as i64,
                        new: new_life,
                    });
                }

                Ok(())
            }

            GameAction::DrawCard { player } => {
                // Delegate to the existing draw_card method which handles
                // empty-library flagging and zone transitions.
                // draw_card already emits ZoneChange events via move_object.
                self.draw_card(player)?;
                Ok(())
            }

            GameAction::GainLife { player, amount, .. } => {
                if amount == 0 {
                    return Ok(());
                }
                let old_life = self.get_player(player)?.life_total;
                let p = self.get_player_mut(player)?;
                p.life_total += amount as i64;
                let new_life = p.life_total;

                self.events.emit(GameEvent::LifeChanged {
                    player_id: player,
                    old: old_life,
                    new: new_life,
                });

                Ok(())
            }

            GameAction::LoseLife { player, amount } => {
                if amount == 0 {
                    return Ok(());
                }
                let old_life = self.get_player(player)?.life_total;
                let p = self.get_player_mut(player)?;
                p.life_total -= amount as i64;
                let new_life = p.life_total;

                self.events.emit(GameEvent::LifeChanged {
                    player_id: player,
                    old: old_life,
                    new: new_life,
                });

                Ok(())
            }

            GameAction::ZoneChange { object, from: _, to } => {
                // Delegate to move_object which handles all zone bookkeeping
                // and emits its own ZoneChange event.
                self.move_object(object, to)?;
                Ok(())
            }

            GameAction::Untap { object } => {
                if let Some(entry) = self.battlefield.get_mut(&object) {
                    entry.tapped = false;
                }
                // No event emitted for untap yet — will be added when
                // tap/untap triggers are needed (Phase 6).
                Ok(())
            }

            GameAction::Tap { object } => {
                if let Some(entry) = self.battlefield.get_mut(&object) {
                    entry.tapped = true;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event::DamageTarget;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::types::card_types::*;
    use crate::types::keywords::KeywordAbility;
    use crate::types::mana::ManaType;

    fn setup_game_with_creature() -> (GameState, ObjectId) {
        let mut game = GameState::new(2, 20);

        let bears = CardDataBuilder::new("Grizzly Bears")
            .mana_cost(crate::types::mana::ManaCost::single(ManaType::Green, 1, 1))
            .color(crate::types::colors::Color::Green)
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Bear))
            .power_toughness(2, 2)
            .build();

        let obj = GameObject::new(bears, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        (game, id)
    }

    #[test]
    fn test_execute_deal_damage_to_creature() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Object(bears_id),
            amount: 3,
            is_combat: false,
        }).unwrap();

        assert_eq!(game.battlefield.get(&bears_id).unwrap().damage_marked, 3);
        // Should have emitted a DamageDealt event
        assert_eq!(game.events.len(), 1);
    }

    #[test]
    fn test_execute_deal_damage_to_player() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Player(1),
            amount: 3,
            is_combat: false,
        }).unwrap();

        assert_eq!(game.players[1].life_total, 17);
        // DamageDealt + LifeChanged
        assert_eq!(game.events.len(), 2);
    }

    #[test]
    fn test_execute_zero_damage_is_noop() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Player(1),
            amount: 0,
            is_combat: false,
        }).unwrap();

        assert_eq!(game.players[1].life_total, 20);
        assert_eq!(game.events.len(), 0);
    }

    #[test]
    fn test_execute_gain_life() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::GainLife {
            player: 0,
            amount: 5,
            source: bears_id,
        }).unwrap();

        assert_eq!(game.players[0].life_total, 25);
        assert_eq!(game.events.len(), 1);
    }

    #[test]
    fn test_execute_lose_life() {
        let (mut game, _bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::LoseLife {
            player: 0,
            amount: 3,
        }).unwrap();

        assert_eq!(game.players[0].life_total, 17);
        assert_eq!(game.events.len(), 1);
    }

    #[test]
    fn test_execute_untap() {
        let (mut game, bears_id) = setup_game_with_creature();
        game.battlefield.get_mut(&bears_id).unwrap().tapped = true;

        game.execute_action(GameAction::Untap {
            object: bears_id,
        }).unwrap();

        assert!(!game.battlefield.get(&bears_id).unwrap().tapped);
    }

    // --- Lifelink tests (4h) ---

    fn setup_game_with_lifelink_creature() -> (GameState, ObjectId) {
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Lifelink Creature")
            .mana_cost(crate::types::mana::ManaCost::single(ManaType::White, 1, 1))
            .color(crate::types::colors::Color::White)
            .card_type(CardType::Creature)
            .power_toughness(2, 3)
            .keyword(KeywordAbility::Lifelink)
            .build();

        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0);

        (game, id)
    }

    #[test]
    fn test_lifelink_combat_damage_gains_life() {
        let (mut game, lifelinker) = setup_game_with_lifelink_creature();

        game.execute_action(GameAction::DealDamage {
            source: lifelinker,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }).unwrap();

        // Player 1 took 2 damage: 20 - 2 = 18
        assert_eq!(game.players[1].life_total, 18);
        // Player 0 (controller) gained 2 life: 20 + 2 = 22
        assert_eq!(game.players[0].life_total, 22);
    }

    #[test]
    fn test_lifelink_noncombat_damage_gains_life() {
        let (mut game, lifelinker) = setup_game_with_lifelink_creature();

        game.execute_action(GameAction::DealDamage {
            source: lifelinker,
            target: DamageTarget::Player(1),
            amount: 3,
            is_combat: false,
        }).unwrap();

        assert_eq!(game.players[1].life_total, 17);
        assert_eq!(game.players[0].life_total, 23);
    }

    #[test]
    fn test_no_lifelink_no_life_gain() {
        let (mut game, bears_id) = setup_game_with_creature(); // no lifelink

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }).unwrap();

        assert_eq!(game.players[1].life_total, 18);
        // Player 0 should NOT have gained life
        assert_eq!(game.players[0].life_total, 20);
    }
}
