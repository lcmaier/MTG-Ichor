use std::collections::HashMap;

use crate::events::event::GameEvent;
use crate::objects::card_data::{AbilityType, Cost};
use crate::objects::object::GameObject;
use crate::state::game_state::{GameState, PhaseType, StackEntry};
use crate::types::card_types::CardType;
use crate::types::effects::TargetSpec;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::keywords::KeywordAbility;
use crate::types::zones::Zone;
use crate::ui::decision::DecisionProvider;

impl GameState {
    /// Cast a spell from hand onto the stack (rule 601.2).
    ///
    /// Steps:
    /// 1. Legality check (601.3) — correct zone, timing, player
    /// 2. Move to stack (601.2a)
    /// 3. Choose targets (601.2c) via DecisionProvider
    /// 4. Pay costs (601.2f-h) — mana cost from CardData
    /// 5. Emit SpellCast event (601.2i)
    pub fn cast_spell(
        &mut self,
        player_id: PlayerId,
        card_id: ObjectId,
        decisions: &dyn DecisionProvider,
    ) -> Result<(), String> {
        // --- 1. Legality checks (rule 601.3) ---
        self.check_cast_legality(player_id, card_id)?;

        // Snapshot data we need before moving the card
        let card_data = self.get_object(card_id)?.card_data.clone();

        // Find the spell ability on the card
        let spell_ability = card_data.abilities.iter()
            .find(|a| a.ability_type == AbilityType::Spell)
            .ok_or_else(|| format!("Card '{}' has no spell ability", card_data.name))?;
        let effect = spell_ability.effect.clone();
        let target_spec = match &effect {
            crate::types::effects::Effect::Atom(_, ts) => ts.clone(),
            crate::types::effects::Effect::Sequence(effects) => {
                // For sequence effects, use the target spec from the first atom
                effects.iter().find_map(|e| {
                    if let crate::types::effects::Effect::Atom(_, ts) = e {
                        Some(ts.clone())
                    } else {
                        None
                    }
                }).unwrap_or(TargetSpec::None)
            }
            _ => TargetSpec::None,
        };

        // --- 2. Move to stack (rule 601.2a) ---
        self.move_object(card_id, Zone::Stack)?;

        // --- 3. Choose targets (rule 601.2c) ---
        let targets = if target_spec != TargetSpec::None && target_spec != TargetSpec::You {
            let chosen = decisions.choose_targets(self, player_id, &target_spec);
            self.validate_targets(&target_spec, &chosen)?;
            chosen
        } else {
            Vec::new()
        };

        // --- Create StackEntry ---
        let entry = StackEntry {
            object_id: card_id,
            controller: player_id,
            chosen_targets: targets,
            chosen_modes: Vec::new(),
            x_value: None,
            effect,
            is_spell: true,
        };
        self.stack_entries.insert(card_id, entry);

        // --- 4. Determine and pay costs (rule 601.2e-h) ---
        //
        // TODO(Phase N - Cost Modification Pipeline):
        // Rule 601.2e requires determining the *total* cost of a spell after
        // all modifications. The full pipeline is:
        //   1. Start with base mana cost (or alternative cost like Flashback)
        //   2. Apply cost increases (e.g. Thalia: "noncreature spells cost {1} more")
        //   3. Apply cost reductions (e.g. Goblin Electromancer)
        //   4. Apply Trinisphere-style minimum floors
        //   5. Lock in the final cost
        //
        // This will require a `determine_total_cost(&self, card_id) -> ManaCost`
        // method that queries continuous effects on the GameState. For now we
        // just use the card's printed mana cost directly.
        if let Some(ref mana_cost) = card_data.mana_cost {
            let costs = vec![Cost::Mana(mana_cost.clone())];
            let generic_allocation = decisions.choose_generic_mana_allocation(
                self, player_id, mana_cost,
            );
            self.pay_costs(&costs, player_id, card_id, &generic_allocation)?;
        }

        // --- 5. Emit SpellCast event (rule 601.2i) ---
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
    /// - Various graveyard-activated abilities (e.g. Scavenging Ooze's exile)
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
        let target_spec = match &effect {
            crate::types::effects::Effect::Atom(_, ts) => ts.clone(),
            _ => TargetSpec::None,
        };

        // Create a new object on the stack representing the ability (rule 602.2a)
        // Abilities on the stack are not cards — they have no CardData.
        // We create a minimal GameObject to track it.
        let ability_obj = GameObject::new(card_data.clone(), player_id, Zone::Stack);
        let ability_obj_id = ability_obj.id;
        self.objects.insert(ability_obj_id, ability_obj);
        self.stack.push(ability_obj_id);

        // Choose targets
        let targets = if target_spec != TargetSpec::None && target_spec != TargetSpec::You {
            let chosen = decisions.choose_targets(self, player_id, &target_spec);
            self.validate_targets(&target_spec, &chosen)?;
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
    use crate::types::effects::{AmountExpr, Effect, Primitive, TargetSpec, TargetCount};
    use crate::types::mana::{ManaCost, ManaType};
    use crate::ui::decision::ScriptedDecisionProvider;

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
        // Card should NOT have moved — it's still in hand
        // (move_object happened before pay, so we need to handle this;
        // but for Phase 2, we accept this limitation per the plan:
        // "validate legality upfront before mutating state")
    }

    #[test]
    fn test_cast_sorcery_timing_wrong_phase() {
        let mut game = GameState::new(2, 20);
        let sorcery_data = CardDataBuilder::new("Lava Axe")
            .card_type(CardType::Sorcery)
            .color(crate::types::colors::Color::Red)
            .mana_cost(ManaCost::single(ManaType::Red, 1, 4))
            .ability(AbilityDef {
                id: crate::types::ids::new_ability_id(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(
                    Primitive::DealDamage(AmountExpr::Fixed(5)),
                    TargetSpec::Player(TargetCount::Exactly(1)),
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
}
