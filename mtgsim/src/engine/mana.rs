use crate::engine::actions::{ActionContext, GameAction};
use crate::engine::resolve::ResolutionContext;
use crate::objects::card_data::AbilityType;
use crate::types::costs::Cost;
use crate::types::effects::{Effect, Primitive};
use crate::state::game_state::GameState;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};

/// Mana ability engine (rule 605).
///
/// Mana abilities are special: they don't use the stack and resolve immediately.
/// This module handles activating mana abilities on permanents. Cost payment
/// is delegated to the shared `engine::costs::pay_costs` system.
///
/// Note: Complex mana abilities (e.g. Metalworker) that involve choices or
/// non-mana effects will eventually go through the general effect resolution
/// pipeline, just without using the stack.

impl GameState {
    /// Activate a mana ability on a permanent.
    ///
    /// Mana abilities resolve immediately (they don't go on the stack).
    /// Cost payment is handled by the shared `pay_costs` system.
    pub fn activate_mana_ability(
        &mut self,
        player_id: PlayerId,
        permanent_id: ObjectId,
        ability_id: AbilityId,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        // Snapshot the ability definition (clone to release borrow).
        // Effective, not printed — intrinsic land mana abilities (CR 305.6) are
        // synthesized by the layer system and absent from CardData.
        self.get_object(permanent_id)?;
        let abilities = crate::oracle::characteristics::get_effective_abilities(self, permanent_id);

        let ability = abilities.iter()
            .find(|a| a.id == ability_id)
            .ok_or_else(|| format!("Ability {} not found on permanent {}", ability_id, permanent_id))?;

        if ability.ability_type != AbilityType::Mana {
            return Err("This is not a mana ability".to_string());
        }

        // Verify controller
        if !self.battlefield.contains_key(&permanent_id) {
            return Err(format!("Permanent {} not on battlefield", permanent_id));
        }
        // Effective controller (CR 613.1b): stealing a land steals its mana.
        if !crate::oracle::characteristics::controls(self, permanent_id, player_id) {
            return Err("You don't control this permanent".to_string());
        }

        // Pay costs via shared cost payment system: plan, then perform.
        // A mana ability's cost is usually just {T}, but Krark-Clan Ironworks'
        // is a sacrifice, and which artifact pays it is a choice — taken here,
        // before anything moves.
        let plan = self.plan_payment(&ability.costs, player_id, permanent_id, ctx)?;
        self.pay_costs(&plan, player_id, permanent_id, ctx)?;

        // CR 106.12: "to 'tap [a permanent] for mana' is to activate a mana
        // ability of that permanent that includes the {T} symbol in its
        // activation cost" — the ability's costs, read here where the
        // effective ability is in hand, and never the payment plan.
        let tapped_for_mana = ability.costs.iter().any(|c| matches!(c, Cost::Tap));

        // Resolve effect immediately (mana abilities don't use the stack)
        self.resolve_mana_effect(&ability.effect, player_id, permanent_id, tapped_for_mana, ctx)?;

        Ok(())
    }

    /// Resolve the effect of a mana ability.
    ///
    /// Mana abilities resolve immediately without the stack (rule 605.3b),
    /// so game state cannot change between activation and resolution, and a
    /// dynamic amount is safe to read off the board here. The amounts go
    /// through the same `evaluate_amount` a resolving spell's do, against a
    /// resolution context with no targets — a mana ability has none (CR
    /// 605.1a) — so a target-dependent expression is refused by that
    /// function rather than here. Doubling Cube's `UnspentMana` is the first
    /// dynamic amount a mana ability carries; Selvala's "greatest power" is
    /// `CountOf`'s shape when it arrives.
    ///
    /// The production is a proposal (CR 106.6a's replaceable event) and its
    /// batch is its own, separate from the cost's: CR 605.3b makes the
    /// resolution a step after the activation, and CR 106.12a's triggers
    /// fire "whenever such a mana ability resolves and produces mana", not
    /// when the permanent taps. A `Sequence` proposes one event per atom.
    fn resolve_mana_effect(
        &mut self,
        effect: &Effect,
        player_id: PlayerId,
        source: ObjectId,
        tapped_for_mana: bool,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        match effect {
            Effect::Atom(Primitive::ProduceMana(output), _) => {
                let resolution = ResolutionContext {
                    source,
                    ability_source: None,
                    controller: player_id,
                    targets: Vec::new(),
                    replaced_amount: None,
                    damage_prevented: None,
                };
                let mut mana = Vec::with_capacity(output.mana.len());
                for (mana_type, amount_expr) in &output.mana {
                    mana.push((*mana_type, self.evaluate_amount(amount_expr, &resolution)?));
                }
                self.execute_action(
                    GameAction::ProduceMana {
                        player: player_id,
                        source,
                        mana,
                        special: output.special.clone(),
                        tapped_for_mana,
                    },
                    ctx,
                )
            }
            Effect::Sequence(effects) => {
                for sub_effect in effects {
                    self.resolve_mana_effect(sub_effect, player_id, source, tapped_for_mana, ctx)?;
                }
                Ok(())
            }
            _ => Err(format!("Unsupported effect in mana ability: {:?}", effect)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::test_ctx;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::PermanentState;
    use crate::state::game_state::GameState;
    use crate::types::card_types::*;
    use crate::types::mana::ManaType;
    use crate::types::zones::Zone;

    fn setup_with_forest() -> (GameState, crate::types::ids::ObjectId, crate::types::ids::AbilityId) {
        let mut game = GameState::new(2, 20);

        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Forest))
            .mana_ability_single(ManaType::Green)
            .build();

        let ability_id = forest.abilities[0].id;

        let obj = GameObject::new(forest, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = PermanentState::new(id, 0, 0);
        game.insert_battlefield_entity(id, entry);

        (game, id, ability_id)
    }

    #[test]
    fn test_activate_forest_mana_ability() {
        let (mut game, forest_id, ability_id) = setup_with_forest();

        assert_eq!(game.players[0].mana_pool.total(), 0);

        game.activate_mana_ability(0, forest_id, ability_id, &test_ctx()).unwrap();

        assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 1);
        assert!(game.battlefield.get(&forest_id).unwrap().tapped);
    }

    #[test]
    fn test_cannot_activate_already_tapped() {
        let (mut game, forest_id, ability_id) = setup_with_forest();

        game.activate_mana_ability(0, forest_id, ability_id, &test_ctx()).unwrap();
        let result = game.activate_mana_ability(0, forest_id, ability_id, &test_ctx());
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_activate_opponents_permanent() {
        let (mut game, forest_id, ability_id) = setup_with_forest();

        let result = game.activate_mana_ability(1, forest_id, ability_id, &test_ctx());
        assert!(result.is_err());
    }
}
