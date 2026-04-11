use crate::engine::resolve::ResolutionContext;
use crate::events::event::GameEvent;
use crate::state::game_state::GameState;
use crate::types::effects::TargetSpec;
use crate::types::zones::Zone;

impl GameState {
    /// Resolve the top entry on the stack (rule 608).
    ///
    /// Called when all players pass in succession and the stack is non-empty
    /// (rule 405.5 / 117.4).
    ///
    /// Steps:
    /// 1. Pop the top ObjectId from the stack.
    /// 2. Look up its StackEntry.
    /// 3. Re-validate targets (608.2b) — fizzle if all illegal.
    /// 4. Resolve the effect via resolve_effect().
    /// 5. Post-resolution: move spell to graveyard or remove ability (608.2n).
    /// 6. Emit SpellResolved event.
    pub fn resolve_top_of_stack(&mut self) -> Result<(), String> {
        if self.stack.is_empty() {
            return Err("Cannot resolve: stack is empty".to_string());
        }

        // Pop the top of stack (last element = top, LIFO).
        // The spell/ability is removed from the stack Vec BEFORE resolution so
        // that effects which inspect the stack (e.g. CounterSpell) don't see
        // the currently-resolving object. We handle the zone bookkeeping
        // manually below instead of going through move_object (which would try
        // to remove from the stack Vec a second time).
        let object_id = self.stack.pop().unwrap();
        let entry = self.stack_entries.remove(&object_id)
            .ok_or_else(|| format!("No StackEntry for object {}", object_id))?;

        // --- Re-validate targets (rule 608.2b) ---
        let target_spec = self.extract_target_spec(&entry.effect);
        let has_targets = !matches!(target_spec, TargetSpec::None | TargetSpec::You);

        if has_targets && !self.any_targets_still_legal(&target_spec, &entry.chosen_targets) {
            // All targets illegal — spell/ability fizzles (is countered by game rules)
            self.handle_fizzle(object_id, &entry)?;
            return Ok(());
        }

        // --- Resolve the effect (rule 608.2c-m) ---
        let ctx = ResolutionContext {
            source: object_id,
            controller: entry.controller,
            targets: entry.chosen_targets.clone(),
        };
        self.resolve_effect(&entry.effect, &ctx)?;

        // --- Post-resolution (rule 608.2n) ---
        // We already removed the object from self.stack above, so we handle
        // zone transitions manually to avoid move_object double-removing.
        if entry.is_spell {
            let obj = self.get_object(object_id)?;
            let is_permanent_type = obj.card_data.types.iter().any(|t| t.is_permanent());

            if is_permanent_type {
                // Permanent spell: move to battlefield.
                // We handle this manually (same as the instant/sorcery path below)
                // because move_object would try to remove from the stack Vec,
                // but we already popped the object above. No re-push needed.
                //
                // Rule 110.2: the controller of the permanent is whoever
                // controlled the spell on the stack when it resolved.
                let controller = entry.controller;
                let owner = self.get_object(object_id)?.owner;
                self.get_object_mut(object_id)?.zone = Zone::Battlefield;
                self.init_zone_state_with_controller(object_id, controller)?;
                // Carry X value from the stack entry to the permanent (rule 107.3f)
                if let Some(bf_entry) = self.battlefield.get_mut(&object_id) {
                    bf_entry.x_value = entry.x_value;
                }
                self.events.emit(GameEvent::ZoneChange {
                    object_id,
                    owner,
                    from: Zone::Stack,
                    to: Zone::Battlefield,
                });
                self.events.emit(GameEvent::PermanentEnteredBattlefield {
                    object_id,
                    controller,
                });
            } else {
                // Instant/sorcery: move to owner's graveyard
                let owner = self.get_object(object_id)?.owner;
                self.get_object_mut(object_id)?.zone = Zone::Graveyard;
                self.get_player_mut(owner)?.graveyard.push(object_id);
                self.events.emit(GameEvent::ZoneChange {
                    object_id,
                    owner,
                    from: Zone::Stack,
                    to: Zone::Graveyard,
                });
            }
        } else {
            // Ability: ceases to exist — remove from objects entirely
            self.objects.remove(&object_id);
        }

        // --- Emit event ---
        self.events.emit(GameEvent::SpellResolved {
            spell_id: object_id,
        });

        Ok(())
    }

    /// Handle a spell/ability that fizzles (all targets now illegal).
    ///
    /// The object has already been popped from self.stack before this is called.
    /// Spells go to their owner's graveyard. Abilities cease to exist.
    fn handle_fizzle(
        &mut self,
        object_id: crate::types::ids::ObjectId,
        entry: &crate::state::game_state::StackEntry,
    ) -> Result<(), String> {
        if entry.is_spell {
            // Move to graveyard manually (already removed from stack Vec)
            let owner = self.get_object(object_id)?.owner;
            self.get_object_mut(object_id)?.zone = Zone::Graveyard;
            self.get_player_mut(owner)?.graveyard.push(object_id);
            self.events.emit(GameEvent::ZoneChange {
                object_id,
                owner,
                from: Zone::Stack,
                to: Zone::Graveyard,
            });
        } else {
            // Ability: just remove from objects
            self.objects.remove(&object_id);
        }

        self.events.emit(GameEvent::SpellFizzled {
            spell_id: object_id,
        });

        Ok(())
    }

    /// Extract the TargetSpec from an Effect for re-validation purposes.
    fn extract_target_spec(&self, effect: &crate::types::effects::Effect) -> TargetSpec {
        match effect {
            crate::types::effects::Effect::Atom(_, ts) => ts.clone(),
            crate::types::effects::Effect::Sequence(effects) => {
                effects.iter().find_map(|e| {
                    if let crate::types::effects::Effect::Atom(_, ts) = e {
                        Some(ts.clone())
                    } else {
                        None
                    }
                }).unwrap_or(TargetSpec::None)
            }
            _ => TargetSpec::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::resolve::ResolvedTarget;
    use crate::objects::card_data::{AbilityDef, AbilityType, CardDataBuilder};
    use crate::objects::object::GameObject;
    use crate::state::game_state::{GameState, StackEntry};
    use crate::types::card_types::CardType;
    use crate::types::effects::{AmountExpr, CountExpr, Effect, Primitive, TargetSpec, TargetCount};
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;

    fn make_bolt() -> std::sync::Arc<crate::objects::card_data::CardData> {
        CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::single(ManaType::Red, 1, 0))
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage(AmountExpr::Fixed(3)),
                    TargetSpec::Any(TargetCount::Exactly(1)),
                ),
            })
            .build()
    }

    fn make_recall() -> std::sync::Arc<crate::objects::card_data::CardData> {
        CardDataBuilder::new("Ancestral Recall")
            .card_type(CardType::Instant)
            .color(crate::types::colors::Color::Blue)
            .mana_cost(ManaCost::single(ManaType::Blue, 1, 0))
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DrawCards(CountExpr::Fixed(3)),
                    TargetSpec::Player(TargetCount::Exactly(1)),
                ),
            })
            .build()
    }

    /// Helper: put a spell directly on the stack with a StackEntry.
    fn put_spell_on_stack(
        game: &mut GameState,
        card_data: std::sync::Arc<crate::objects::card_data::CardData>,
        controller: usize,
        targets: Vec<ResolvedTarget>,
    ) -> crate::types::ids::ObjectId {
        let ability = card_data.abilities.iter()
            .find(|a| a.ability_type == AbilityType::Spell)
            .unwrap();
        let effect = ability.effect.clone();

        let obj = GameObject::new(card_data, controller, Zone::Stack);
        let id = obj.id;
        game.add_object(obj);
        game.stack.push(id);
        game.stack_entries.insert(id, StackEntry {
            object_id: id,
            controller,
            chosen_targets: targets,
            chosen_modes: Vec::new(),
            x_value: None,
            effect,
            is_spell: true,
        });
        id
    }

    #[test]
    fn test_resolve_bolt_targeting_player() {
        let mut game = GameState::new(2, 20);
        let bolt_id = put_spell_on_stack(
            &mut game,
            make_bolt(),
            0,
            vec![ResolvedTarget::Player(1)],
        );

        game.resolve_top_of_stack().unwrap();

        // Player 1 should have lost 3 life
        assert_eq!(game.players[1].life_total, 17);
        // Bolt should be in graveyard
        assert_eq!(game.get_object(bolt_id).unwrap().zone, Zone::Graveyard);
        assert!(game.players[0].graveyard.contains(&bolt_id));
        // Stack should be empty
        assert!(game.stack.is_empty());
        assert!(game.stack_entries.is_empty());
    }

    #[test]
    fn test_resolve_recall_draws_cards() {
        let mut game = GameState::new(2, 20);
        // Give player 0 some cards in library
        for _ in 0..5 {
            let card = CardDataBuilder::new("Dummy").build();
            let obj = GameObject::new(card, 0, Zone::Library);
            let id = obj.id;
            game.add_object(obj);
            game.players[0].library.push(id);
        }

        let recall_id = put_spell_on_stack(
            &mut game,
            make_recall(),
            0,
            vec![ResolvedTarget::Player(0)],
        );

        game.resolve_top_of_stack().unwrap();

        // Player 0 should have drawn 3 cards
        assert_eq!(game.players[0].hand.len(), 3);
        assert_eq!(game.players[0].library.len(), 2);
        // Recall in graveyard
        assert_eq!(game.get_object(recall_id).unwrap().zone, Zone::Graveyard);
    }

    #[test]
    fn test_resolve_empty_stack_error() {
        let mut game = GameState::new(2, 20);
        assert!(game.resolve_top_of_stack().is_err());
    }

    #[test]
    fn test_lifo_order() {
        let mut game = GameState::new(2, 20);
        // Give player 1 cards in library for Recall
        for _ in 0..5 {
            let card = CardDataBuilder::new("Dummy").build();
            let obj = GameObject::new(card, 1, Zone::Library);
            let id = obj.id;
            game.add_object(obj);
            game.players[1].library.push(id);
        }

        // First on stack: Recall targeting player 1
        let _recall_id = put_spell_on_stack(
            &mut game,
            make_recall(),
            0,
            vec![ResolvedTarget::Player(1)],
        );

        // Second on stack (top): Bolt targeting player 1
        let _bolt_id = put_spell_on_stack(
            &mut game,
            make_bolt(),
            0,
            vec![ResolvedTarget::Player(1)],
        );

        // Resolve top — should be Bolt (LIFO)
        game.resolve_top_of_stack().unwrap();
        assert_eq!(game.players[1].life_total, 17); // Bolt did 3
        assert_eq!(game.players[1].hand.len(), 0); // Recall hasn't resolved

        // Resolve next — should be Recall
        game.resolve_top_of_stack().unwrap();
        assert_eq!(game.players[1].hand.len(), 3); // Recall drew 3
    }

    fn make_grizzly_bears() -> std::sync::Arc<crate::objects::card_data::CardData> {
        CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(crate::types::colors::Color::Green)
            .mana_cost(ManaCost::single(ManaType::Green, 1, 1))
            .power_toughness(2, 2)
            .build()
    }

    /// Helper: put a permanent spell on the stack (no targets, no spell ability effect).
    fn put_permanent_on_stack(
        game: &mut GameState,
        card_data: std::sync::Arc<crate::objects::card_data::CardData>,
        controller: usize,
    ) -> crate::types::ids::ObjectId {
        let obj = GameObject::new(card_data, controller, Zone::Stack);
        let id = obj.id;
        game.add_object(obj);
        game.stack.push(id);
        game.stack_entries.insert(id, StackEntry {
            object_id: id,
            controller,
            chosen_targets: Vec::new(),
            chosen_modes: Vec::new(),
            x_value: None,
            effect: Effect::Sequence(vec![]),
            is_spell: true,
        });
        id
    }

    #[test]
    fn test_creature_spell_resolves_to_battlefield() {
        let mut game = GameState::new(2, 20);
        let bears_id = put_permanent_on_stack(&mut game, make_grizzly_bears(), 0);

        game.resolve_top_of_stack().unwrap();

        // Creature should be on the battlefield, not on the stack or in graveyard
        assert_eq!(game.get_object(bears_id).unwrap().zone, Zone::Battlefield);
        assert!(game.battlefield.contains_key(&bears_id));
        assert!(game.stack.is_empty());
        assert!(!game.players[0].graveyard.contains(&bears_id));

        // BattlefieldEntity should have correct state
        let entry = game.battlefield.get(&bears_id).unwrap();
        assert_eq!(entry.controller, 0);
        assert!(!entry.tapped);

        // P/T comes from CardData
        let obj = game.get_object(bears_id).unwrap();
        assert_eq!(obj.card_data.power, Some(2));
        assert_eq!(obj.card_data.toughness, Some(2));
    }

    #[test]
    fn test_creature_has_summoning_sickness_on_entry() {
        let mut game = GameState::new(2, 20);
        let bears_id = put_permanent_on_stack(&mut game, make_grizzly_bears(), 0);

        game.resolve_top_of_stack().unwrap();

        let entry = game.battlefield.get(&bears_id).unwrap();
        assert!(entry.summoning_sick);
    }

    #[test]
    fn test_permanent_spell_not_on_stack_after_resolution() {
        let mut game = GameState::new(2, 20);
        let bears_id = put_permanent_on_stack(&mut game, make_grizzly_bears(), 0);

        // Verify it's on the stack before resolution
        assert!(game.stack.contains(&bears_id));

        game.resolve_top_of_stack().unwrap();

        // Stack should be completely empty — no re-push artifact
        assert!(game.stack.is_empty());
        assert!(game.stack_entries.is_empty());
    }

    /// Helper: put a permanent spell on the stack with a specific x_value.
    fn put_permanent_on_stack_with_x(
        game: &mut GameState,
        card_data: std::sync::Arc<crate::objects::card_data::CardData>,
        controller: usize,
        x_value: Option<u64>,
    ) -> crate::types::ids::ObjectId {
        let obj = GameObject::new(card_data, controller, Zone::Stack);
        let id = obj.id;
        game.add_object(obj);
        game.stack.push(id);
        game.stack_entries.insert(id, StackEntry {
            object_id: id,
            controller,
            chosen_targets: Vec::new(),
            chosen_modes: Vec::new(),
            x_value,
            effect: Effect::Sequence(vec![]),
            is_spell: true,
        });
        id
    }

    #[test]
    fn test_x_value_carried_to_permanent() {
        let mut game = GameState::new(2, 20);
        let card = CardDataBuilder::new("Hangarback Walker")
            .card_type(CardType::Creature)
            .power_toughness(0, 0)
            .build();
        let id = put_permanent_on_stack_with_x(&mut game, card, 0, Some(3));

        game.resolve_top_of_stack().unwrap();

        let bf_entry = game.battlefield.get(&id).unwrap();
        assert_eq!(bf_entry.x_value, Some(3));
    }

    #[test]
    fn test_x_value_none_for_non_x_spell() {
        let mut game = GameState::new(2, 20);
        let bears_id = put_permanent_on_stack(&mut game, make_grizzly_bears(), 0);

        game.resolve_top_of_stack().unwrap();

        let bf_entry = game.battlefield.get(&bears_id).unwrap();
        assert_eq!(bf_entry.x_value, None);
    }

    #[test]
    fn test_fizzle_target_gone() {
        let mut game = GameState::new(2, 20);
        // Create a "creature" on the battlefield for player 1
        let creature_data = CardDataBuilder::new("Test Creature")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build();
        let creature = GameObject::new(creature_data, 1, Zone::Battlefield);
        let creature_id = creature.id;
        let ts = game.allocate_timestamp();
        game.add_object(creature);
        game.battlefield.insert(creature_id, crate::state::battlefield::BattlefieldEntity::new(creature_id, 1, ts));

        // Put Bolt on stack targeting the creature
        let bolt_id = put_spell_on_stack(
            &mut game,
            make_bolt(),
            0,
            vec![ResolvedTarget::Object(creature_id)],
        );

        // Remove the creature from the battlefield before resolution (simulating it being destroyed)
        game.move_object(creature_id, Zone::Graveyard).unwrap();

        // Resolve — Bolt should fizzle
        game.resolve_top_of_stack().unwrap();

        // Player 1's life should be unchanged (bolt didn't redirect to player)
        assert_eq!(game.players[1].life_total, 20);
        // Bolt should be in graveyard (fizzled spells go to graveyard)
        assert_eq!(game.get_object(bolt_id).unwrap().zone, Zone::Graveyard);
        assert!(game.stack.is_empty());
    }
}
