//! Mana ability engine (rule 605).
//!
//! Mana abilities are special: they don't use the stack and resolve immediately.
//! This module handles activating mana abilities on permanents. Cost payment
//! is delegated to the shared `engine::costs::pay_costs` system.
//!
//! Note: Complex mana abilities (e.g. Metalworker) that involve choices or
//! non-mana effects will eventually go through the general effect resolution
//! pipeline, just without using the stack.

use crate::engine::actions::{ActionContext, GameAction};
use crate::engine::resolve::ResolutionContext;
use crate::events::cause::GeneratedBy;
use crate::objects::card_data::AbilityType;
use crate::types::costs::Cost;
use crate::oracle::characteristics::controller_or_owner;
use crate::types::effects::{Antecedent, Effect, EffectRecipient, ManaOutput, Primitive};
use crate::state::game_state::GameState;
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::triggers::Lki;
use crate::ui::decision::DecisionProvider;

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
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        // Effective, not printed — intrinsic land mana abilities (CR 305.6) are
        // synthesized by the layer system and absent from CardData.
        self.get_object(permanent_id)?;
        let ability = crate::oracle::characteristics::effective_ability(self, permanent_id, ability_id)
            .ok_or_else(|| format!("Ability {} not found on permanent {}", ability_id, permanent_id))?;

        if ability.ability_type != AbilityType::Mana {
            return Err("This is not a mana ability".to_string());
        }

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
        let paying = ActionContext::paying_cost(dp, player_id, permanent_id);
        let source = self.object_ref(permanent_id).ok_or_else(|| format!("Permanent {permanent_id} is in no zone"))?;
        let from = self.get_object(permanent_id)?.zone;
        let plan = self.plan_payment(&ability.costs, player_id, permanent_id, &paying)?;
        self.pay_costs(&plan, player_id, permanent_id, &paying)?;

        // CR 605.3a: activated, and announced as it is, before it resolves
        // (605.3b).
        let identity = crate::state::game_state::AbilityIdentity { source, ability: ability_id };
        // Where its cost moved it, as it left (CR 608.2h).
        let moved = self.lki(source).cloned();
        self.announce_activation(identity, player_id, true, from, moved.clone(), dp)?;

        // CR 106.12: "to 'tap [a permanent] for mana' is to activate a mana
        // ability of that permanent that includes the {T} symbol in its
        // activation cost" — the ability's costs, read here where the
        // effective ability is in hand, and never the payment plan.
        let tapped_for_mana = ability.costs.iter().any(|c| matches!(c, Cost::TapSelf));

        // Resolve effect immediately (mana abilities don't use the stack)
        let producer = ManaProducer { id: permanent_id, tapped_for_mana, lki: moved.as_ref() };
        self.resolve_mana_effect(&ability.effect, player_id, producer, dp)?;

        Ok(())
    }

    /// Resolve the effect of a mana ability.
    ///
    /// Mana abilities resolve immediately without the stack (rule 605.3b),
    /// so game state cannot change between activation and resolution, and a
    /// dynamic amount is safe to read off the board here. **It is a
    /// resolution**, so it gets a `ResolutionContext` like any other — the
    /// permanent is both `source` (there is no stack object, CR 605.3b) and
    /// `ability_source` (CR 113.7a's "the permanent whose ability is
    /// resolving") — and the amounts go through the same `evaluate_amount` a
    /// spell's do; a mana ability has no targets (CR 605.1a), so a
    /// target-dependent expression is refused by that function rather than
    /// here. Doubling Cube's `UnspentMana` is the first dynamic amount a mana
    /// ability carries; Selvala's "greatest power" is `CountOf`'s shape when
    /// it arrives.
    ///
    /// The production is a proposal (CR 106.6a's replaceable event), made
    /// **under this resolution's stamp** so the log says which ability added
    /// the mana, as it already does for Dark Ritual's; its batch is its own,
    /// separate from the cost's: CR 605.3b makes the resolution a step after
    /// the activation, and CR 106.12a's triggers fire "whenever such a mana
    /// ability resolves and produces mana", not when the permanent taps. A
    /// `Sequence` proposes one event per atom.
    fn resolve_mana_effect(
        &mut self,
        effect: &Effect,
        player_id: PlayerId,
        producer: ManaProducer<'_>,
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        match effect {
            Effect::Atom(Primitive::ProduceMana(output), _) => {
                let resolution = ResolutionContext::for_mana_ability(self, producer.id, player_id, GeneratedBy::ActivatedAbility);
                let actx = ActionContext::resolving(dp, &resolution);
                self.produce_mana(output, player_id, producer, &resolution, &actx)
            }
            Effect::Sequence(effects) => {
                for sub_effect in effects {
                    self.resolve_mana_effect(sub_effect, player_id, producer, dp)?;
                }
                Ok(())
            }
            _ => Err(format!("Unsupported effect in mana ability: {:?}", effect)),
        }
    }

    /// CR 106.6a's event: `output`'s mana for `player`, made by `producer`,
    /// each amount read for `resolution`, proposed under `actx` — a mana
    /// ability's, activated or triggered, and a spell's (Dark Ritual), whose
    /// `tapped_for_mana` is false by CR 106.12's definition.
    pub(crate) fn produce_mana(
        &mut self,
        output: &ManaOutput,
        player: PlayerId,
        producer: ManaProducer<'_>,
        resolution: &ResolutionContext,
        actx: &ActionContext,
    ) -> Result<(), String> {
        let mut mana = Vec::with_capacity(output.mana.len());
        for (mana_type, amount) in &output.mana {
            mana.push((*mana_type, self.evaluate_amount(amount, resolution, None)?));
        }
        let ManaProducer { id: source, tapped_for_mana, lki } = producer;
        let special = output.special.clone();
        self.execute_action(GameAction::ProduceMana { player, source, mana, special, tapped_for_mana, source_lki: lki.cloned() }, actx)
    }
}

/// What makes a mana production, as its readers ask it: the object, whether
/// it was tapped for mana (CR 106.12), and the object as it last existed
/// where its own mana ability's cost moved it before the mana was made
/// (Crystal Vein, a Treasure), which "a land you control" still reads it as
/// (CR 608.2h).
#[derive(Debug, Clone, Copy)]
pub(crate) struct ManaProducer<'a> {
    pub(crate) id: ObjectId,
    pub(crate) tapped_for_mana: bool,
    pub(crate) lki: Option<&'a Lki>,
}

/// Whose pool a triggered mana ability's mana goes to: "its controller adds"
/// on an Aura is the host as recipient, the enchanted permanent's controller
/// (Wild Growth's ruling: the mana is not the land's ability), and anything
/// else the ability's controller's.
pub(crate) fn triggered_mana_recipient(
    game: &GameState,
    recipient: &EffectRecipient,
    source: ObjectId,
    controller: PlayerId,
) -> PlayerId {
    match recipient {
        EffectRecipient::Object(Antecedent::ThisObjectsHost) => crate::oracle::board::host_of(game, source)
            .and_then(|host| controller_or_owner(game, host))
            .unwrap_or(controller),
        _ => controller,
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::{test_dp};
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
        let id = game.add_object(obj);
        let entry = PermanentState::new(id, 0, 0);
        game.insert_battlefield_entity(id, entry);

        (game, id, ability_id)
    }

    #[test]
    fn test_activate_forest_mana_ability() {
        let (mut game, forest_id, ability_id) = setup_with_forest();

        assert_eq!(game.players[0].mana_pool.total(), 0);

        game.activate_mana_ability(0, forest_id, ability_id, &test_dp()).unwrap();

        assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 1);
        assert!(game.battlefield.get(&forest_id).unwrap().tapped);
    }

    #[test]
    fn test_cannot_activate_already_tapped() {
        let (mut game, forest_id, ability_id) = setup_with_forest();

        game.activate_mana_ability(0, forest_id, ability_id, &test_dp()).unwrap();
        let result = game.activate_mana_ability(0, forest_id, ability_id, &test_dp());
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_activate_opponents_permanent() {
        let (mut game, forest_id, ability_id) = setup_with_forest();

        let result = game.activate_mana_ability(1, forest_id, ability_id, &test_dp());
        assert!(result.is_err());
    }
}
