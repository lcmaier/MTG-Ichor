use std::collections::HashMap;

use crate::engine::costs::assemble_total_cost;
use crate::events::event::GameEvent;
use crate::objects::card_data::AbilityType;
use crate::types::costs::Cost;
use crate::objects::object::GameObject;
use crate::state::game_state::{GameState, PhaseType, StackEntry};
use crate::types::card_types::CardType;
use crate::types::effects::EffectRecipient;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::keywords::KeywordAbility;
use crate::types::mana::ManaCost;
use crate::types::zones::Zone;
use crate::ui::decision::DecisionProvider;

impl GameState {
    /// Cast a spell from hand onto the stack (rule 601.2).
    ///
    /// Steps follow CR 601.2a–i:
    /// 1. Pre-proposal legality check (rule 601.3)
    /// 2. Move to stack (601.2a)
    /// 3. Choose alternative cost, additional costs, X value (601.2b)
    /// 4. Choose targets (601.2c)
    /// 5. Distribution placeholder (601.2d — T18c)
    /// 6. Post-proposal legality check with rollback (601.2e)
    /// 7. Assemble total cost (601.2f)
    /// 8. Mana ability window placeholder (601.2g)
    /// 9. Pay total cost (601.2h)
    /// 10. Emit SpellCast event (601.2i)
    pub fn cast_spell(
        &mut self,
        player_id: PlayerId,
        card_id: ObjectId,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        // --- Pre-proposal legality check (rule 601.3) ---
        self.check_cast_legality(player_id, card_id)?;

        // Snapshot data we need before moving the card
        let card_data = self.get_object(card_id)?.card_data.clone();

        // Find the spell ability on the card.
        // Permanent spells (creatures, enchantments, artifacts, planeswalkers)
        // may not have a spell ability — they resolve by entering the
        // battlefield. Use an empty Sequence as a no-op effect.
        let (effect, recipient) = if let Some(spell_ability) = card_data.abilities.iter()
            .find(|a| a.ability_type == AbilityType::Spell)
        {
            let effect = spell_ability.effect.clone();
            let recipient = match &effect {
                crate::types::effects::Effect::Atom(_, ts) => ts.clone(),
                crate::types::effects::Effect::Sequence(effects) => {
                    // For sequence effects, use the target spec from the first atom
                    effects.iter().find_map(|e| {
                        if let crate::types::effects::Effect::Atom(_, ts) = e {
                            Some(ts.clone())
                        } else {
                            None
                        }
                    }).unwrap_or(EffectRecipient::Implicit)
                }
                _ => EffectRecipient::Implicit,
            };
            (effect, recipient)
        } else if card_data.types.iter().any(|t| t.is_permanent()) {
            // Permanent spell with no spell ability — resolves by ETB alone
            (crate::types::effects::Effect::Sequence(Vec::new()), EffectRecipient::Implicit)
        } else {
            return Err(format!("Card '{}' has no spell ability", card_data.name));
        };

        // --- 601.2a: Move to stack ---
        self.move_object(card_id, Zone::Stack)?;

        // --- 601.2b: Choose alternative cost, additional costs, X value ---
        let chosen_alt_cost_idx = if !card_data.alternative_costs.is_empty() {
            decisions.choose_alternative_cost(self, player_id, &card_data.alternative_costs)
        } else {
            None
        };

        // Validate alt cost index is in range
        if let Some(idx) = chosen_alt_cost_idx {
            if idx >= card_data.alternative_costs.len() {
                self.move_object(card_id, Zone::Hand)?;
                return Err(format!(
                    "Alternative cost index {} out of range (card has {})",
                    idx, card_data.alternative_costs.len()
                ));
            }
        }

        let chosen_additional_cost_indices = if !card_data.additional_costs.is_empty() {
            decisions.choose_additional_costs(self, player_id, &card_data.additional_costs)
        } else {
            Vec::new()
        };

        // Validate additional cost indices are in range
        for &idx in &chosen_additional_cost_indices {
            if idx >= card_data.additional_costs.len() {
                self.move_object(card_id, Zone::Hand)?;
                return Err(format!(
                    "Additional cost index {} out of range (card has {})",
                    idx, card_data.additional_costs.len()
                ));
            }
        }

        // Choose X value if the cost has X symbols (rule 107.3a)
        let base_mana_cost = card_data.mana_cost.clone()
            .unwrap_or_else(ManaCost::zero);
        let x_count = base_mana_cost.x_count();
        let x_value = if x_count > 0 {
            decisions.choose_x_value(self, player_id, x_count)
        } else {
            0
        };

        // --- 601.2c: Choose targets ---
        let targets = if recipient != EffectRecipient::Implicit && recipient != EffectRecipient::Controller {
            let chosen = decisions.choose_targets(self, player_id, &recipient);
            if let Err(e) = self.validate_targets(&recipient, &chosen) {
                self.move_object(card_id, Zone::Hand)?;
                return Err(e);
            }
            chosen
        } else {
            Vec::new()
        };

        // --- 601.2d: Distribution placeholder (T18c) ---

        // --- Create StackEntry with all proposal data ---
        let chosen_alt = chosen_alt_cost_idx.map(|idx| card_data.alternative_costs[idx].clone());
        let chosen_additional: Vec<_> = chosen_additional_cost_indices.iter()
            .map(|&idx| card_data.additional_costs[idx].clone())
            .collect();

        let entry = StackEntry {
            object_id: card_id,
            controller: player_id,
            chosen_targets: targets,
            chosen_modes: Vec::new(),
            x_value: if x_count > 0 { Some(x_value) } else { None },
            effect,
            is_spell: true,
            chosen_alternative_cost: chosen_alt.clone(),
            additional_costs_paid: chosen_additional.clone(),
        };
        self.stack_entries.insert(card_id, entry);

        // --- 601.2e: Post-proposal legality check ---
        // At this point the only mutations are: card moved to stack + StackEntry created.
        // No costs paid yet. If the proposal is illegal, rollback via move_object(Hand)
        // which also cleans up the StackEntry.
        //
        // Currently a no-op (the pre-proposal check is sufficient for the cards we
        // support). Future: validate that chosen targets are still legal after all
        // proposal choices are made, and that the assembled cost is payable.

        // --- 601.2f: Assemble total cost ---
        let additional_refs: Vec<_> = chosen_additional.iter().collect();
        let total_costs = assemble_total_cost(
            &base_mana_cost,
            chosen_alt.as_ref(),
            &additional_refs,
            x_value,
        );

        // --- 601.2g: Mana ability window ---
        // TODO: Allow the player to activate mana abilities here before payment.
        // Currently mana abilities are activated before cast_spell via the
        // priority action queue in the DecisionProvider.

        // --- 601.2h: Pay total cost ---
        // Pre-check: can we pay? If not, roll back.
        if let Err(e) = self.can_pay_costs(&total_costs, player_id, card_id) {
            // Rollback: move card back to hand. move_object cleans up stack_entries.
            self.move_object(card_id, Zone::Hand)?;
            return Err(e);
        }

        // Find the mana cost component for generic allocation
        let mana_cost_for_alloc = total_costs.iter().find_map(|c| {
            if let Cost::Mana(mc) = c { Some(mc.clone()) } else { None }
        }).unwrap_or_else(ManaCost::zero);

        let generic_allocation = if mana_cost_for_alloc.generic_count() > 0 {
            decisions.choose_generic_mana_allocation(
                self, player_id, &mana_cost_for_alloc,
            )
        } else {
            HashMap::new()
        };

        self.pay_costs(&total_costs, player_id, card_id, &generic_allocation)?;

        // --- 601.2i: Emit SpellCast event ---
        self.events.emit(GameEvent::SpellCast {
            spell_id: card_id,
            caster: player_id,
        });

        Ok(())
    }

    /// Activate a non-mana activated ability and put it on the stack (rule 602.2).
    ///
    /// Creates a new stack object representing the ability. The source permanent
    /// remains where it is. Mana abilities are handled separately in engine/mana.rs.
    ///
    /// # Future extensibility
    /// Currently assumes the source is on the battlefield. This will need to
    /// become zone-aware once we implement:
    /// - **Cycling** (activated from hand, rule 702.29)
    /// - **Unearth** (activated from graveyard, rule 702.84)
    /// - **Channel** (activated from hand, rule 702.47)
    /// - Various graveyard-activated abilities (e.g. Reassembling Skeleton's self-recursion)
    ///
    /// Planned approach: each AbilityDef gains an `activation_zone: Option<Zone>`
    /// field (None = battlefield, the default). This function would check the
    /// source is in the ability's declared activation zone.
    pub fn activate_ability(
        &mut self,
        player_id: PlayerId,
        source_id: ObjectId,
        ability_index: usize,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        // Verify the source is on the battlefield and controlled by this player
        // (see doc comment for future zone-aware activation plan)
        let entry = self.battlefield.get(&source_id)
            .ok_or_else(|| format!("Permanent {} not on battlefield", source_id))?;
        if entry.controller != player_id {
            return Err("Can only activate abilities of permanents you control".to_string());
        }

        let card_data = self.get_object(source_id)?.card_data.clone();
        let ability = card_data.abilities.get(ability_index)
            .ok_or_else(|| format!("Ability index {} out of range", ability_index))?;

        if ability.ability_type == AbilityType::Mana {
            return Err("Use activate_mana_ability for mana abilities".to_string());
        }
        if ability.ability_type != AbilityType::Activated {
            return Err(format!("Ability at index {} is not an activated ability", ability_index));
        }

        let effect = ability.effect.clone();
        let ability_costs = ability.costs.clone();
        let recipient = match &effect {
            crate::types::effects::Effect::Atom(_, ts) => ts.clone(),
            _ => EffectRecipient::Implicit,
        };

        // Create a new object on the stack representing the ability (rule 602.2a)
        // Abilities on the stack are not cards — they have no CardData.
        // We create a minimal GameObject to track it.
        let ability_obj = GameObject::new(card_data.clone(), player_id, Zone::Stack);
        let ability_obj_id = ability_obj.id;
        self.objects.insert(ability_obj_id, ability_obj);
        self.stack.push(ability_obj_id);

        // Choose targets
        let targets = if recipient != EffectRecipient::Implicit && recipient != EffectRecipient::Controller {
            let chosen = decisions.choose_targets(self, player_id, &recipient);
            self.validate_targets(&recipient, &chosen)?;
            chosen
        } else {
            Vec::new()
        };

        // Create StackEntry
        let stack_entry = StackEntry {
            object_id: ability_obj_id,
            controller: player_id,
            chosen_targets: targets,
            chosen_modes: Vec::new(),
            x_value: None,
            effect,
            is_spell: false,
            chosen_alternative_cost: None,
            additional_costs_paid: Vec::new(),
        };
        self.stack_entries.insert(ability_obj_id, stack_entry);

        // Pay ability costs
        let generic_allocation = HashMap::new();
        self.pay_costs(&ability_costs, player_id, source_id, &generic_allocation)?;

        Ok(())
    }

    /// Check whether a player can legally begin casting a spell (rule 601.3).
    ///
    /// # Future extensibility
    /// Currently hard-codes Zone::Hand as the only legal cast zone. This will
    /// need to become a query against "cast permissions" once we implement:
    /// - **Flashback** (cast from graveyard, rule 702.33)
    /// - **Cascade / Impulse draw** (cast from exile)
    /// - **Cycling-adjacent** cast-from-zone effects
    ///
    /// The planned approach: introduce a `CastPermission` enum or trait that
    /// cards/effects register on the GameState (e.g. "player X may cast card Y
    /// from zone Z this turn"). `check_cast_legality` would then check the
    /// card's current zone against any active permissions, defaulting to Hand.
    fn check_cast_legality(
        &self,
        player_id: PlayerId,
        card_id: ObjectId,
    ) -> Result<(), String> {
        let obj = self.get_object(card_id)?;

        // Card must be in hand (see doc comment for future zone-casting plan)
        if obj.zone != Zone::Hand {
            return Err(format!("Card is in {:?}, not in hand", obj.zone));
        }

        // Card must belong to (or be controlled by) this player
        if obj.owner != player_id {
            return Err("Cannot cast another player's spell".to_string());
        }

        // Timing check (rule 117.1a):
        // - Instants and spells with flash: anytime you have priority
        // - Everything else: main phase, stack empty, active player only
        let is_instant = obj.card_data.types.contains(&CardType::Instant);
        let has_flash = obj.card_data.keywords.contains(&KeywordAbility::Flash);

        if !is_instant && !has_flash {
            // Sorcery-speed timing
            if player_id != self.active_player {
                return Err("Only the active player can cast sorcery-speed spells".to_string());
            }
            match self.phase.phase_type {
                PhaseType::Precombat | PhaseType::Postcombat => {}
                _ => return Err("Sorcery-speed spells can only be cast during a main phase".to_string()),
            }
            if !self.stack.is_empty() {
                return Err("Sorcery-speed spells can only be cast when the stack is empty".to_string());
            }
        }

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::resolve::ResolvedTarget;
    use crate::objects::card_data::{AbilityDef, CardDataBuilder};
    use crate::types::card_types::*;
    use crate::types::effects::{AmountExpr, Effect, Primitive, EffectRecipient, SelectionFilter, TargetCount};
    use crate::types::mana::{ManaCost, ManaType};
    use crate::ui::decision::ScriptedDecisionProvider;

    fn make_bolt() -> std::sync::Arc<crate::objects::card_data::CardData> {
        CardDataBuilder::new("Lightning Bolt")
            .card_type(CardType::Instant)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::build(&[ManaType::Red], 0))
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage(AmountExpr::Fixed(3)),
                    EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
                ),
            })
            .build()
    }

    fn setup_for_casting() -> (GameState, ObjectId, ScriptedDecisionProvider) {
        let mut game = GameState::new(2, 20);
        // Give player 0 a bolt in hand and red mana
        let bolt = make_bolt();
        let obj = GameObject::new(bolt, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        game.players[0].mana_pool.add(ManaType::Red, 1);
        // Set to precombat main phase so sorcery-speed works too
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let decisions = ScriptedDecisionProvider::new();
        decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Player(1)]);

        (game, card_id, decisions)
    }

    #[test]
    fn test_cast_instant_spell() {
        let (mut game, card_id, decisions) = setup_for_casting();
        game.cast_spell(0, card_id, &decisions).unwrap();

        // Card should be on the stack
        assert!(game.stack.contains(&card_id));
        assert!(game.stack_entries.contains_key(&card_id));
        assert_eq!(game.get_object(card_id).unwrap().zone, Zone::Stack);

        // Mana should be spent
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 0);

        // Hand should be empty
        assert!(game.players[0].hand.is_empty());

        // StackEntry should have correct targets
        let entry = game.stack_entries.get(&card_id).unwrap();
        assert_eq!(entry.chosen_targets, vec![ResolvedTarget::Player(1)]);
        assert!(entry.is_spell);
    }

    #[test]
    fn test_cast_from_wrong_zone() {
        let mut game = GameState::new(2, 20);
        let bolt = make_bolt();
        let obj = GameObject::new(bolt, 0, Zone::Graveyard);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].graveyard.push(card_id);

        let decisions = ScriptedDecisionProvider::new();
        assert!(game.cast_spell(0, card_id, &decisions).is_err());
    }

    #[test]
    fn test_cast_not_enough_mana() {
        let (mut game, card_id, decisions) = setup_for_casting();
        // Drain the mana pool
        let _ = game.players[0].mana_pool.remove(ManaType::Red, 1);

        assert!(game.cast_spell(0, card_id, &decisions).is_err());
    }

    #[test]
    fn test_cast_sorcery_timing_wrong_phase() {
        let mut game = GameState::new(2, 20);
        let sorcery_data = CardDataBuilder::new("Lava Axe")
            .card_type(CardType::Sorcery)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::build(&[ManaType::Red], 4))
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage(AmountExpr::Fixed(5)),
                    EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
                ),
            })
            .build();
        let obj = GameObject::new(sorcery_data, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        // Set to combat phase — sorceries can't be cast here
        game.phase = crate::state::game_state::Phase::new(PhaseType::Combat);

        let decisions = ScriptedDecisionProvider::new();
        assert!(game.cast_spell(0, card_id, &decisions).is_err());
    }

    #[test]
    fn test_cast_instant_during_combat() {
        let (mut game, card_id, decisions) = setup_for_casting();
        // Instants can be cast during any phase
        game.phase = crate::state::game_state::Phase::new(PhaseType::Combat);
        game.cast_spell(0, card_id, &decisions).unwrap();
        assert!(game.stack.contains(&card_id));
    }

    // --- T18a: X value, alternative cost, additional cost, rollback tests ---

    fn make_x_spell() -> std::sync::Arc<crate::objects::card_data::CardData> {
        use crate::types::mana::ManaSymbol;
        // Blaze: {X}{R} — deal X damage to any target
        CardDataBuilder::new("Blaze")
            .card_type(CardType::Sorcery)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Red)]))
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage(AmountExpr::Variable),
                    EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
                ),
            })
            .build()
    }

    #[test]
    fn test_cast_x_spell_x_equals_3() {
        let mut game = GameState::new(2, 20);
        let blaze = make_x_spell();
        let obj = GameObject::new(blaze, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        // Need {R} + 3 generic = 4 total mana
        game.players[0].mana_pool.add(ManaType::Red, 4);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let decisions = ScriptedDecisionProvider::new();
        decisions.x_value_decisions.borrow_mut().push(3);
        decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Player(1)]);

        game.cast_spell(0, card_id, &decisions).unwrap();

        // Card on stack
        assert!(game.stack.contains(&card_id));
        // X value stored in StackEntry
        let entry = game.stack_entries.get(&card_id).unwrap();
        assert_eq!(entry.x_value, Some(3));
        // Mana spent: 1 Red + 3 generic (from Red pool) = 4 Red total
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 0);
    }

    #[test]
    fn test_cast_x_spell_x_equals_0() {
        let mut game = GameState::new(2, 20);
        let blaze = make_x_spell();
        let obj = GameObject::new(blaze, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        // Only need {R} for X=0
        game.players[0].mana_pool.add(ManaType::Red, 1);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let decisions = ScriptedDecisionProvider::new();
        decisions.x_value_decisions.borrow_mut().push(0);
        decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Player(1)]);

        game.cast_spell(0, card_id, &decisions).unwrap();

        let entry = game.stack_entries.get(&card_id).unwrap();
        assert_eq!(entry.x_value, Some(0));
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 0);
    }

    #[test]
    fn test_cast_x_spell_insufficient_mana_rollback() {
        let mut game = GameState::new(2, 20);
        let blaze = make_x_spell();
        let obj = GameObject::new(blaze, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        // Only 2 Red, but X=3 needs 4 total
        game.players[0].mana_pool.add(ManaType::Red, 2);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let decisions = ScriptedDecisionProvider::new();
        decisions.x_value_decisions.borrow_mut().push(3);
        decisions.target_decisions.borrow_mut().push(vec![ResolvedTarget::Player(1)]);

        let result = game.cast_spell(0, card_id, &decisions);
        assert!(result.is_err());

        // Card should be back in hand (rollback)
        assert_eq!(game.get_object(card_id).unwrap().zone, Zone::Hand);
        assert!(game.players[0].hand.contains(&card_id));
        assert!(!game.stack.contains(&card_id));
        assert!(!game.stack_entries.contains_key(&card_id));
        // Mana should not have been spent
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 2);
    }

    #[test]
    fn test_cast_with_alternative_cost() {
        use crate::types::costs::AlternativeCost;

        // Card with alt cost: "Pay 3 life instead of mana cost"
        let card = CardDataBuilder::new("Force Spike Variant")
            .card_type(CardType::Instant)
            .color(crate::types::colors::Color::Blue)
            .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Sequence(Vec::new()),
            })
            .alternative_cost(AlternativeCost::Custom(
                "Pay 3 life".to_string(),
                vec![Cost::PayLife(3)],
            ))
            .build();

        let mut game = GameState::new(2, 20);
        let obj = GameObject::new(card, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        // No mana needed — paying life instead
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let decisions = ScriptedDecisionProvider::new();
        decisions.alternative_cost_decisions.borrow_mut().push(Some(0)); // choose alt cost index 0

        game.cast_spell(0, card_id, &decisions).unwrap();

        // Card on stack
        assert!(game.stack.contains(&card_id));
        let entry = game.stack_entries.get(&card_id).unwrap();
        assert!(entry.chosen_alternative_cost.is_some());
        // Life paid
        assert_eq!(game.players[0].life_total, 17);
    }

    #[test]
    fn test_cast_with_kicker_additional_cost() {
        use crate::types::costs::AdditionalCost;

        // Card: {1}{R} with kicker {R}
        let card = CardDataBuilder::new("Goblin Bushwhacker")
            .card_type(CardType::Creature)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::build(&[ManaType::Red], 1))
            .power_toughness(1, 1)
            .additional_cost(AdditionalCost::Kicker(vec![
                Cost::Mana(ManaCost::build(&[ManaType::Red], 0)),
            ]))
            .build();

        let mut game = GameState::new(2, 20);
        let obj = GameObject::new(card, 0, Zone::Hand);
        let card_id = obj.id;
        game.add_object(obj);
        game.players[0].hand.push(card_id);
        // Need {1}{R} (base) + {R} (kicker) = 3 red total
        game.players[0].mana_pool.add(ManaType::Red, 3);
        game.phase = crate::state::game_state::Phase::new(PhaseType::Precombat);
        game.active_player = 0;

        let decisions = ScriptedDecisionProvider::new();
        decisions.additional_cost_decisions.borrow_mut().push(vec![0]); // pay kicker

        game.cast_spell(0, card_id, &decisions).unwrap();

        assert!(game.stack.contains(&card_id));
        let entry = game.stack_entries.get(&card_id).unwrap();
        assert_eq!(entry.additional_costs_paid.len(), 1);
        assert!(matches!(&entry.additional_costs_paid[0], AdditionalCost::Kicker(_)));
        // All 3 red mana spent
        assert_eq!(game.players[0].mana_pool.amount(ManaType::Red), 0);
    }

    #[test]
    fn test_cast_normal_cost_no_x_no_alt() {
        // Verify the normal path still sets x_value=None and no alt/additional
        let (mut game, card_id, decisions) = setup_for_casting();
        game.cast_spell(0, card_id, &decisions).unwrap();

        let entry = game.stack_entries.get(&card_id).unwrap();
        assert_eq!(entry.x_value, None);
        assert!(entry.chosen_alternative_cost.is_none());
        assert!(entry.additional_costs_paid.is_empty());
    }
}
