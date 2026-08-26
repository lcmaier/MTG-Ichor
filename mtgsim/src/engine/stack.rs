use crate::oracle::characteristics::{has_permanent_type, has_subtype};
use crate::engine::actions::{ActionContext, ZoneChangeCause};
use crate::engine::resolve::{ResolutionContext, ResolvedTarget};
use crate::events::event::GameEvent;
use crate::state::game_state::{GameState, ResolvingObject};
use crate::types::card_types::{EnchantmentType, Subtype};
use crate::types::effects::EffectRecipient;
use crate::types::zones::Zone;
use crate::ui::decision::DecisionProvider;

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
    /// 5. Post-resolution: instant or sorcery to its owner's graveyard and the
    ///    ability removed (608.2n), permanent spell onto the battlefield
    ///    (608.3a/c) — through `change_zone` like anything else.
    /// 6. Emit `AbilityResolved` if it was an ability.
    pub fn resolve_top_of_stack(&mut self, dp: &dyn DecisionProvider) -> Result<(), String> {
        if self.stack.is_empty() {
            return Err("Cannot resolve: stack is empty".to_string());
        }

        // Pop the top of stack (last element = top, LIFO).
        //
        // The object comes off the stack `Vec` before it resolves. **This is not
        // what the CR describes** — CR 608.2 keeps a resolving spell on the
        // stack until 608.2n or 608.3a moves it — and the reason the pattern was
        // originally given, that it stops an in-flight Counterspell from seeing
        // the resolving object, does not hold: CR 608.2g forbids casting a spell
        // during a resolution at all, and a spell cannot choose itself at
        // CR 601.2c because target enumeration already excludes it by
        // `exclude_id`. Kept for now, described honestly, and tracked for removal
        // in `codebase-state.md` — see `GameState::resolving`.
        let object_id = *self.stack.last().unwrap();

        // Before the pop, deliberately: a spell's controller lives on its
        // `StackEntry`, which the next two lines destroy. Note it is *not* used
        // for the graveyard below — a finished spell goes to its owner's
        // (CR 608.2n).
        //
        // **Two different controllers, and CR 110.2b is the reason.** The
        // *effective* controller is who controls the spell right now, which a
        // steal makes someone other than the caster; that player follows the
        // spell's instructions (CR 608.2c) and controls the permanent it
        // becomes. But "the permanent's controller **by default** is the player
        // who put that spell onto the stack" — and the default is precisely what
        // `BattlefieldEntity.controller` holds, since `compute.rs::base_controller`
        // reads it as the value Layer 2 modifies. Writing the effective value
        // there double-counts the steal: right today because CR 400.7a keeps the
        // Layer 2 row applying, wrong the moment it stops (CR 800.4c, a player
        // leaving a multiplayer game).
        let effective_controller =
            crate::oracle::characteristics::get_effective_controller(self, object_id)
                .ok_or_else(|| format!("No controller for resolving object {}", object_id))?;

        self.stack.pop();
        let entry = self.stack_entries.remove(&object_id)
            .ok_or_else(|| format!("No StackEntry for object {}", object_id))?;

        // CR 110.2b's default. `StackEntry.controller` is written once at
        // CR 601.2a / 602.2a and never mutated — a control-changing effect on
        // the spell goes through the continuous-effects registry, not this
        // field — so it *is* "the player who put that spell onto the stack".
        let default_controller = entry.controller;

        // Record the window the pop just opened. Two things downstream need to
        // know about it, and both used to be served by writing `obj.zone` by
        // hand at three sites instead: the stack removal in `move_object` (which
        // would otherwise report a missing object as a bug) and CR 110.2b's
        // controller (which the `StackEntry` just taken above was carrying).
        // Cleared on every path out, including the error ones — that is what
        // `resolve_popped` exists to make single-sited.
        self.resolving = Some(ResolvingObject {
            id: object_id,
            default_controller,
        });
        let result = self.resolve_popped(object_id, entry, effective_controller, dp);
        self.resolving = None;
        result
    }

    /// The body of [`Self::resolve_top_of_stack`], after the pop.
    ///
    /// Split out only so that `resolving` has exactly one place to be cleared.
    fn resolve_popped(
        &mut self,
        object_id: crate::types::ids::ObjectId,
        entry: crate::state::game_state::StackEntry,
        controller: crate::types::ids::PlayerId,
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        // --- Re-validate targets (rule 608.2b) ---
        let recipient = self.extract_recipient(&entry.effect);
        let has_targets = matches!(recipient, EffectRecipient::Target(_, _));

        if has_targets && !self.any_targets_still_legal(&recipient, &entry.chosen_targets, controller) {
            // All targets illegal — spell/ability fizzles (is countered by game rules)
            self.handle_fizzle(object_id, &entry, dp)?;
            return Ok(());
        }

        // --- Resolve the effect (rule 608.2c-m) ---
        let ctx = ResolutionContext {
            source: object_id,
            controller,
            targets: entry.chosen_targets.clone(),
        };
        self.resolve_effect(&entry.effect, &ctx, dp)?;

        // --- Post-resolution (rule 608.2n) ---
        // The zone change goes through the chokepoint like every other one.
        // Until RA-3 these were three `// REPLACEMENT-BYPASS:` sites that wrote
        // the zone field by hand, because `move_object` would have tried to
        // remove the object from the stack `Vec` a second time. `resolving`
        // makes that expected rather than a bug, so nothing has to bypass.
        //
        // Not optional for v1 Commander: a commander permanent spell resolves
        // through here, and CR 903.9b has to be able to offer the command zone
        // instead.
        let actx = ActionContext::resolving(dp, &ctx);
        if entry.is_spell {
            self.get_object(object_id)?;
            let is_permanent_type = has_permanent_type(self, object_id);

            if is_permanent_type {
                // Permanent spell: it becomes a permanent and enters the
                // battlefield (CR 608.3a; 608.3c for an Aura, handled below).
                // It enters under its CR 110.2b *default* controller, which
                // `init_zone_state` reads off `resolving`; the steal's Layer 2
                // row continues to apply on top per CR 400.7a, so the effective
                // controller is still the thief.
                self.change_zone(object_id, Zone::Battlefield, ZoneChangeCause::Resolved, &actx)?;
                // Carry X value from the stack entry to the permanent (rule 107.3f)
                if let Some(bf_entry) = self.battlefield.get_mut(&object_id) {
                    bf_entry.x_value = entry.x_value;
                }
                self.events.emit(GameEvent::PermanentEnteredBattlefield {
                    object_id,
                    controller,
                });

                // Rule 303.4f: Aura spell resolves → enters attached to its
                // target.  The fizzle check (608.2b) at the top of this
                // function guarantees the target is still legal — if it
                // weren't, the spell would have fizzled before reaching here.
                let is_aura = has_subtype(
                    self, object_id, &Subtype::Enchantment(EnchantmentType::Aura));
                if is_aura {
                    let host_id = match entry.chosen_targets.first().copied() {
                        Some(ResolvedTarget::Object(id)) => id,
                        _ => return Err(format!(
                            "Aura {} resolved from stack with no Object target — \
                             this should have been caught by the fizzle check",
                            object_id
                        )),
                    };
                    if let Some(aura_bf) = self.battlefield.get_mut(&object_id) {
                        aura_bf.attach_to(host_id);
                    }
                    if let Some(host_bf) = self.battlefield.get_mut(&host_id) {
                        host_bf.attached_by.push(object_id);
                    }
                }
            } else {
                // Instant/sorcery: to its owner's graveyard as the final part
                // of resolution (CR 608.2n).
                self.change_zone(object_id, Zone::Graveyard, ZoneChangeCause::Resolved, &actx)?;
            }
        } else {
            // Ability: ceases to exist — remove from objects entirely
            self.objects.remove(&object_id);
        }

        // --- Emit event ---
        // A spell finishing resolution is already recorded: the `ZoneChange`
        // above carries `ZoneChangeCause::Resolved` and says where it went. An
        // ability leaves no zone change, so this is where it is announced —
        // durably, by what it *is* (CR 603.7h), because the ephemeral object
        // CR 608.2n just destroyed identifies nothing afterward.
        if let Some(identity) = entry.ability_identity {
            self.events.emit(GameEvent::AbilityResolved {
                identity,
                controller: entry.controller,
            });
        }

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
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        if entry.is_spell {
            // CR 608.2b — countered by game rules, and a real zone change. It
            // belongs to no resolution: the spell never resolved.
            self.change_zone(
                object_id,
                Zone::Graveyard,
                ZoneChangeCause::Fizzled,
                &ActionContext::new(dp),
            )?;
        } else {
            // Ability: just remove from objects
            self.objects.remove(&object_id);
        }

        self.events.emit(GameEvent::SpellFizzled {
            spell_id: object_id,
        });

        Ok(())
    }

    /// Extract the EffectRecipient from an Effect for re-validation purposes.
    fn extract_recipient(&self, effect: &crate::types::effects::Effect) -> EffectRecipient {
        match effect {
            crate::types::effects::Effect::Atom(_, ts) => ts.clone(),
            crate::types::effects::Effect::Sequence(effects) => {
                effects.iter().find_map(|e| {
                    if let crate::types::effects::Effect::Atom(_, ts) = e {
                        Some(ts.clone())
                    } else {
                        None
                    }
                }).unwrap_or(EffectRecipient::Implicit)
            }
            _ => EffectRecipient::Implicit,
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
    use crate::types::effects::{AmountExpr, Effect, Primitive, EffectRecipient, SelectionFilter, TargetCount};
    use crate::types::mana::{ManaCost, ManaType};
    use crate::types::zones::Zone;
    use crate::test_support::{lightning_bolt as make_bolt, pacifism as make_pacifism, test_dp};

    fn make_recall() -> std::sync::Arc<crate::objects::card_data::CardData> {
        CardDataBuilder::new("Ancestral Recall")
            .card_type(CardType::Instant)
            .color(crate::types::colors::Color::Blue)
            .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
            .ability(AbilityDef {
                is_characteristic_defining: false,
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DrawCards(AmountExpr::Fixed(3)),
                    EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
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
        // PRE-LAYER ZONE: printed abilities, on the card being put on the stack --
        // the same exemption engine/cast.rs runs under.
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
            chosen_alternative_cost: None,
            additional_costs_paid: Vec::new(),
                    cast_from: Some(Zone::Hand),
                    ability_identity: None,
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

        game.resolve_top_of_stack(&test_dp()).unwrap();

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

        game.resolve_top_of_stack(&test_dp()).unwrap();

        // Player 0 should have drawn 3 cards
        assert_eq!(game.players[0].hand.len(), 3);
        assert_eq!(game.players[0].library.len(), 2);
        // Recall in graveyard
        assert_eq!(game.get_object(recall_id).unwrap().zone, Zone::Graveyard);
    }

    #[test]
    fn test_resolve_empty_stack_error() {
        let mut game = GameState::new(2, 20);
        assert!(game.resolve_top_of_stack(&test_dp()).is_err());
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
        game.resolve_top_of_stack(&test_dp()).unwrap();
        assert_eq!(game.players[1].life_total, 17); // Bolt did 3
        assert_eq!(game.players[1].hand.len(), 0); // Recall hasn't resolved

        // Resolve next — should be Recall
        game.resolve_top_of_stack(&test_dp()).unwrap();
        assert_eq!(game.players[1].hand.len(), 3); // Recall drew 3
    }

    fn make_grizzly_bears() -> std::sync::Arc<crate::objects::card_data::CardData> {
        CardDataBuilder::new("Grizzly Bears")
            .card_type(CardType::Creature)
            .color(crate::types::colors::Color::Green)
            .mana_cost(ManaCost::build(&[ManaType::Green], 1))
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
            chosen_alternative_cost: None,
            additional_costs_paid: Vec::new(),
                    cast_from: Some(Zone::Hand),
                    ability_identity: None,
});
        id
    }

    #[test]
    fn test_creature_spell_resolves_to_battlefield() {
        let mut game = GameState::new(2, 20);
        let bears_id = put_permanent_on_stack(&mut game, make_grizzly_bears(), 0);

        game.resolve_top_of_stack(&test_dp()).unwrap();

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

        game.resolve_top_of_stack(&test_dp()).unwrap();

        // Creature entered on turn 1, turn_number is 1, so it has summoning sickness
        assert!(crate::oracle::characteristics::has_summoning_sickness(&game, bears_id));
    }

    #[test]
    fn test_permanent_spell_not_on_stack_after_resolution() {
        let mut game = GameState::new(2, 20);
        let bears_id = put_permanent_on_stack(&mut game, make_grizzly_bears(), 0);

        // Verify it's on the stack before resolution
        assert!(game.stack.contains(&bears_id));

        game.resolve_top_of_stack(&test_dp()).unwrap();

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
            chosen_alternative_cost: None,
            additional_costs_paid: Vec::new(),
                    cast_from: Some(Zone::Hand),
                    ability_identity: None,
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

        game.resolve_top_of_stack(&test_dp()).unwrap();

        let bf_entry = game.battlefield.get(&id).unwrap();
        assert_eq!(bf_entry.x_value, Some(3));
    }

    #[test]
    fn test_x_value_none_for_non_x_spell() {
        let mut game = GameState::new(2, 20);
        let bears_id = put_permanent_on_stack(&mut game, make_grizzly_bears(), 0);

        game.resolve_top_of_stack(&test_dp()).unwrap();

        let bf_entry = game.battlefield.get(&bears_id).unwrap();
        assert_eq!(bf_entry.x_value, None);
    }

    /// Helper: put a permanent spell on the stack with chosen targets.
    fn put_permanent_on_stack_with_targets(
        game: &mut GameState,
        card_data: std::sync::Arc<crate::objects::card_data::CardData>,
        controller: usize,
        targets: Vec<ResolvedTarget>,
    ) -> crate::types::ids::ObjectId {
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
            effect: Effect::Sequence(vec![]),
            is_spell: true,
            chosen_alternative_cost: None,
            additional_costs_paid: Vec::new(),
                    cast_from: Some(Zone::Hand),
                    ability_identity: None,
});
        id
    }

    #[test]
    fn test_aura_attaches_to_target_on_resolve() {
        // Rule 303.4f: Aura spell resolves → attached_to set to target creature.
        let mut game = GameState::new(2, 20);

        // Put a creature on the battlefield
        let creature = GameObject::new(make_grizzly_bears(), 1, Zone::Battlefield);
        let creature_id = creature.id;
        game.add_object(creature);
        game.place_on_battlefield(creature_id, 1);

        // Put Pacifism on the stack targeting the creature
        let aura_id = put_permanent_on_stack_with_targets(
            &mut game,
            make_pacifism(),
            0,
            vec![ResolvedTarget::Object(creature_id)],
        );

        game.resolve_top_of_stack(&test_dp()).unwrap();

        // Aura should be on the battlefield
        assert_eq!(game.get_object(aura_id).unwrap().zone, Zone::Battlefield);
        assert!(game.battlefield.contains_key(&aura_id));

        // Aura should be attached to the creature
        let aura_entry = game.battlefield.get(&aura_id).unwrap();
        assert_eq!(aura_entry.attached_to, Some(creature_id));
    }

    #[test]
    fn test_aura_host_in_attached_by() {
        // Rule 303.4f: host's attached_by includes the Aura.
        let mut game = GameState::new(2, 20);

        let creature = GameObject::new(make_grizzly_bears(), 1, Zone::Battlefield);
        let creature_id = creature.id;
        game.add_object(creature);
        game.place_on_battlefield(creature_id, 1);

        let aura_id = put_permanent_on_stack_with_targets(
            &mut game,
            make_pacifism(),
            0,
            vec![ResolvedTarget::Object(creature_id)],
        );

        game.resolve_top_of_stack(&test_dp()).unwrap();

        // Host should have the Aura in its attached_by list
        let host_entry = game.battlefield.get(&creature_id).unwrap();
        assert!(host_entry.attached_by.contains(&aura_id));
    }

    /// `GameState::resolving` licenses two things that are bugs anywhere else:
    /// a stack removal that finds nothing, and an entering permanent taking a
    /// controller other than its owner. Leaving it set past the resolution
    /// would extend that licence to whatever the same object does next — a
    /// creature returned from the graveyard would enter under the dead spell's
    /// controller, and a genuinely missing stack object would go unreported.
    #[test]
    fn test_the_resolving_marker_does_not_outlive_the_resolution() {
        // (a) an ordinary resolution
        let mut game = GameState::new(2, 20);
        let bolt = put_spell_on_stack(&mut game, make_bolt(), 0, vec![ResolvedTarget::Player(1)]);
        game.resolve_top_of_stack(&test_dp()).unwrap();
        assert_eq!(game.resolving, None, "cleared after the spell resolved");
        assert_eq!(game.get_object(bolt).unwrap().zone, Zone::Graveyard);

        // (b) a fizzle, which returns from the middle of the function
        let mut game = GameState::new(2, 20);
        let gone = GameObject::new(
            CardDataBuilder::new("Test Creature").card_type(CardType::Creature)
                .power_toughness(2, 2).build(),
            1,
            Zone::Battlefield,
        );
        let gone_id = gone.id;
        let ts = game.allocate_timestamp();
        game.add_object(gone);
        game.battlefield.insert(
            gone_id,
            crate::state::battlefield::BattlefieldEntity::new(gone_id, 1, ts, 1),
        );
        put_spell_on_stack(&mut game, make_bolt(), 0, vec![ResolvedTarget::Object(gone_id)]);
        game.move_object(gone_id, Zone::Graveyard).unwrap();
        game.resolve_top_of_stack(&test_dp()).unwrap();
        assert_eq!(game.resolving, None, "cleared after the spell fizzled");

        // (c) an error out of the middle. An Aura whose stack entry names no
        // target reaches the battlefield and then fails, which is the shape a
        // bare `?` would leak through.
        let mut game = GameState::new(2, 20);
        let aura = GameObject::new(make_pacifism(), 0, Zone::Stack);
        let aura_id = aura.id;
        game.add_object(aura);
        game.stack.push(aura_id);
        game.stack_entries.insert(aura_id, StackEntry {
            object_id: aura_id,
            controller: 0,
            chosen_targets: Vec::new(),
            chosen_modes: Vec::new(),
            x_value: None,
            effect: Effect::Sequence(Vec::new()),
            is_spell: true,
            chosen_alternative_cost: None,
            additional_costs_paid: Vec::new(),
            cast_from: Some(Zone::Hand),
            ability_identity: None,
        });
        assert!(game.resolve_top_of_stack(&test_dp()).is_err());
        assert_eq!(game.resolving, None, "cleared even when resolution errors out");
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
        game.battlefield.insert(creature_id, crate::state::battlefield::BattlefieldEntity::new(creature_id, 1, ts, 1));

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
        game.resolve_top_of_stack(&test_dp()).unwrap();

        // Player 1's life should be unchanged (bolt didn't redirect to player)
        assert_eq!(game.players[1].life_total, 20);
        // Bolt should be in graveyard (fizzled spells go to graveyard)
        assert_eq!(game.get_object(bolt_id).unwrap().zone, Zone::Graveyard);
        assert!(game.stack.is_empty());
    }
}
