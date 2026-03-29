use crate::engine::actions::GameAction;
use crate::events::event::DamageTarget;
use crate::state::game_state::GameState;
use crate::types::effects::{
    AmountExpr, CountExpr, Effect, Primitive, TargetSpec,
};
use crate::types::ids::{ObjectId, PlayerId};

/// Context passed through effect resolution.
///
/// Tracks the source of the spell/ability, its controller, and resolved
/// targets so that each `Primitive` knows what it's acting on.
#[derive(Debug, Clone)]
pub struct ResolutionContext {
    /// The object that is the source of this spell/ability
    pub source: ObjectId,
    /// The player who controls the spell/ability
    pub controller: PlayerId,
    /// Resolved targets (validated before resolution begins)
    pub targets: Vec<ResolvedTarget>,
}

/// A resolved target — validated as legal when the spell/ability was put on the
/// stack. Legality is re-checked at resolution time (rule 608.2b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedTarget {
    Object(ObjectId),
    Player(PlayerId),
}

impl GameState {
    /// Resolve an effect tree in the given context.
    ///
    /// This is the main entry point for spell/ability resolution.
    /// It recursively walks the `Effect` combinator tree and dispatches
    /// each `Primitive` to the appropriate game-state mutation.
    ///
    /// **Phase 2 scope:** handles Atom, Sequence, and the Phase 2 primitives
    /// (DealDamage, DrawCards, GainLife, LoseLife, ProduceMana,
    /// ModifyPowerToughness, CounterSpell). Other combinators and primitives
    /// return `Err` until their phase is implemented.
    pub fn resolve_effect(
        &mut self,
        effect: &Effect,
        ctx: &ResolutionContext,
    ) -> Result<(), String> {
        match effect {
            Effect::Atom(primitive, target_spec) => {
                self.resolve_primitive(primitive, target_spec, ctx)
            }

            Effect::Sequence(effects) => {
                for sub in effects {
                    self.resolve_effect(sub, ctx)?;
                }
                Ok(())
            }

            Effect::Conditional(_condition, _inner) => {
                // Phase 6: evaluate condition, then resolve inner if true
                Err("Conditional effects not yet implemented".to_string())
            }

            Effect::Optional(_inner) => {
                // Phase 6: ask controller via DecisionProvider, then resolve
                Err("Optional effects not yet implemented".to_string())
            }

            Effect::Modal { .. } => {
                // Phase 6: mode selection via DecisionProvider
                Err("Modal effects not yet implemented".to_string())
            }

            Effect::ForEach(_, _) => {
                Err("ForEach effects not yet implemented".to_string())
            }

            Effect::Repeat(_, _) => {
                Err("Repeat effects not yet implemented".to_string())
            }
        }
    }

    /// Resolve a single primitive action against its targets.
    fn resolve_primitive(
        &mut self,
        primitive: &Primitive,
        target_spec: &TargetSpec,
        ctx: &ResolutionContext,
    ) -> Result<(), String> {
        match primitive {
            // === Phase 2 primitives ===

            Primitive::DealDamage(amount_expr) => {
                let amount = self.evaluate_amount(amount_expr, ctx)?;
                for target in &ctx.targets {
                    let damage_target = match target {
                        ResolvedTarget::Object(id) => DamageTarget::Object(*id),
                        ResolvedTarget::Player(pid) => DamageTarget::Player(*pid),
                    };
                    self.execute_action(GameAction::DealDamage {
                        source: ctx.source,
                        target: damage_target,
                        amount,
                        is_combat: false,
                    })?;
                }
                Ok(())
            }

            Primitive::DrawCards(count_expr) => {
                let count = self.evaluate_count(count_expr, ctx)?;
                // Drawing targets the controller (TargetSpec::You or None)
                let player_id = self.resolve_player_for_self(target_spec, ctx);
                for _ in 0..count {
                    self.execute_action(GameAction::DrawCard {
                        player: player_id,
                    })?;
                }
                Ok(())
            }

            Primitive::GainLife(amount_expr) => {
                let amount = self.evaluate_amount(amount_expr, ctx)?;
                let player_id = self.resolve_player_for_self(target_spec, ctx);
                self.execute_action(GameAction::GainLife {
                    player: player_id,
                    amount,
                    source: ctx.source,
                })?;
                Ok(())
            }

            Primitive::LoseLife(amount_expr) => {
                let amount = self.evaluate_amount(amount_expr, ctx)?;
                let player_id = self.resolve_player_for_self(target_spec, ctx);
                self.execute_action(GameAction::LoseLife {
                    player: player_id,
                    amount,
                })?;
                Ok(())
            }

            Primitive::ProduceMana(output) => {
                let player = self.get_player_mut(ctx.controller)?;
                for (mana_type, amount) in &output.mana {
                    player.mana_pool.add(*mana_type, *amount);
                }
                Ok(())
            }

            Primitive::CounterSpell => {
                // Counter target spell on the stack (rule 701.6a).
                // The countered spell is put into its owner's graveyard.
                for target in &ctx.targets {
                    if let ResolvedTarget::Object(id) = target {
                        if let Some(pos) = self.stack.iter().position(|s| s == id) {
                            let countered_id = self.stack.remove(pos);
                            let obj = self.get_object(countered_id)?;
                            let owner = obj.owner;
                            self.get_player_mut(owner)?.graveyard.push(countered_id);
                            let obj_mut = self.get_object_mut(countered_id)?;
                            obj_mut.zone = crate::types::zones::Zone::Graveyard;
                        }
                    }
                }
                Ok(())
            }

            Primitive::CounterAbility => {
                // Counter target activated or triggered ability on the stack
                // (rule 701.6b). The ability ceases to exist — it is simply
                // removed from the stack. It does NOT go to any zone.
                for target in &ctx.targets {
                    if let ResolvedTarget::Object(id) = target {
                        if let Some(pos) = self.stack.iter().position(|s| s == id) {
                            let removed_id = self.stack.remove(pos);
                            // Remove the object entirely — abilities on the
                            // stack are not cards and have no destination zone.
                            self.objects.remove(&removed_id);
                        }
                    }
                }
                Ok(())
            }

            // === Phase 2 primitives: Destroy & Untap ===

            Primitive::Destroy => {
                // Destroy target permanent (rule 701.7a).
                // Moves the permanent from battlefield to its owner's graveyard.
                // TODO: check for indestructible (Phase 5)
                for target in &ctx.targets {
                    if let ResolvedTarget::Object(id) = target {
                        if self.battlefield.contains_key(id) {
                            self.execute_action(GameAction::ZoneChange {
                                object: *id,
                                from: crate::types::zones::Zone::Battlefield,
                                to: crate::types::zones::Zone::Graveyard,
                            })?;
                        }
                        // If not on battlefield, destroy does nothing (rule 701.7b)
                    }
                }
                Ok(())
            }

            Primitive::Untap => {
                // Untap target permanent (rule 701.21a).
                for target in &ctx.targets {
                    if let ResolvedTarget::Object(id) = target {
                        self.execute_action(GameAction::Untap {
                            object: *id,
                        })?;
                    }
                }
                Ok(())
            }

            // === Phase 3+ primitives — stubs ===

            Primitive::Exile
            | Primitive::Sacrifice
            | Primitive::ReturnToHand
            | Primitive::ReturnToBattlefield
            | Primitive::PutOnTopOfLibrary
            | Primitive::PutOnBottomOfLibrary
            | Primitive::ShuffleIntoLibrary
            | Primitive::Mill(_)
            | Primitive::Discard(_)
            | Primitive::Scry(_)
            | Primitive::Surveil(_)
            | Primitive::AddCounters(_, _)
            | Primitive::RemoveCounters(_, _)
            | Primitive::CreateToken(_, _)
            | Primitive::Fight
            | Primitive::Tap
            | Primitive::SetPowerToughness(_, _)
            | Primitive::ModifyPowerToughness(_, _)
            | Primitive::AddAbility(_, _)
            | Primitive::RemoveAbility(_, _)
            | Primitive::ChangeColor(_, _)
            | Primitive::ChangeType(_, _)
            | Primitive::GainControl(_) => {
                Err(format!("Primitive {:?} not yet implemented", primitive))
            }
        }
    }

    // --- Helper: evaluate AmountExpr ---

    fn evaluate_amount(
        &self,
        expr: &AmountExpr,
        _ctx: &ResolutionContext,
    ) -> Result<u64, String> {
        match expr {
            AmountExpr::Fixed(n) => Ok(*n),
            AmountExpr::Variable => {
                // X is stored on the stack object when cast; for now stub
                Err("Variable (X) amount resolution not yet implemented".to_string())
            }
            AmountExpr::CountOf(_selector) => {
                Err("CountOf amount resolution not yet implemented".to_string())
            }
            AmountExpr::TargetPower => {
                Err("TargetPower amount resolution not yet implemented".to_string())
            }
            AmountExpr::TargetToughness => {
                Err("TargetToughness amount resolution not yet implemented".to_string())
            }
            AmountExpr::DamageDealt => {
                Err("DamageDealt amount resolution not yet implemented".to_string())
            }
        }
    }

    // --- Helper: evaluate CountExpr ---

    fn evaluate_count(
        &self,
        expr: &CountExpr,
        _ctx: &ResolutionContext,
    ) -> Result<u64, String> {
        match expr {
            CountExpr::Fixed(n) => Ok(*n),
            CountExpr::Variable => {
                Err("Variable (X) count resolution not yet implemented".to_string())
            }
            CountExpr::CountOf(_selector) => {
                Err("CountOf count resolution not yet implemented".to_string())
            }
        }
    }

    // --- Helper: determine which player an effect applies to ---

    /// For effects that target "you" (the controller) or use TargetSpec::None,
    /// returns the controller. For targeted player effects, returns the first
    /// player target.
    fn resolve_player_for_self(
        &self,
        target_spec: &TargetSpec,
        ctx: &ResolutionContext,
    ) -> PlayerId {
        match target_spec {
            TargetSpec::None | TargetSpec::You => ctx.controller,
            TargetSpec::Player(_) => {
                // Use the first resolved player target
                for t in &ctx.targets {
                    if let ResolvedTarget::Player(pid) = t {
                        return *pid;
                    }
                }
                ctx.controller
            }
            _ => ctx.controller,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::BattlefieldEntity;
    use crate::types::card_types::*;
    use crate::types::mana::ManaType;
    use crate::types::zones::Zone;

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
        let entry = BattlefieldEntity::new(id, 0, 0);
        game.battlefield.insert(id, entry);

        (game, id)
    }

    fn bolt_ctx(source: ObjectId, targets: Vec<ResolvedTarget>) -> ResolutionContext {
        ResolutionContext {
            source,
            controller: 0,
            targets,
        }
    }

    #[test]
    fn test_deal_damage_to_creature() {
        let (mut game, bears_id) = setup_game_with_creature();

        let bolt = Effect::Atom(
            Primitive::DealDamage(AmountExpr::Fixed(3)),
            TargetSpec::Any(crate::types::effects::TargetCount::Exactly(1)),
        );

        let ctx = bolt_ctx(bears_id, vec![ResolvedTarget::Object(bears_id)]);
        game.resolve_effect(&bolt, &ctx).unwrap();

        assert_eq!(game.battlefield.get(&bears_id).unwrap().damage_marked, 3);
    }

    #[test]
    fn test_deal_damage_to_player() {
        let (mut game, bears_id) = setup_game_with_creature();

        let bolt = Effect::Atom(
            Primitive::DealDamage(AmountExpr::Fixed(3)),
            TargetSpec::Any(crate::types::effects::TargetCount::Exactly(1)),
        );

        let ctx = bolt_ctx(bears_id, vec![ResolvedTarget::Player(1)]);
        game.resolve_effect(&bolt, &ctx).unwrap();

        assert_eq!(game.players[1].life_total, 17);
    }

    #[test]
    fn test_draw_cards() {
        let (mut game, bears_id) = setup_game_with_creature();

        // Put some cards in player 0's library
        for _ in 0..5 {
            let card = CardDataBuilder::new("Forest")
                .card_type(CardType::Land)
                .build();
            let obj = GameObject::in_library(card, 0);
            let oid = obj.id;
            game.add_object(obj);
            game.players[0].library.push(oid);
        }

        let draw = Effect::Atom(
            Primitive::DrawCards(CountExpr::Fixed(2)),
            TargetSpec::You,
        );
        let ctx = bolt_ctx(bears_id, vec![]);
        game.resolve_effect(&draw, &ctx).unwrap();

        assert_eq!(game.players[0].hand.len(), 2);
        assert_eq!(game.players[0].library.len(), 3);
    }

    #[test]
    fn test_gain_life() {
        let (mut game, bears_id) = setup_game_with_creature();

        let heal = Effect::Atom(
            Primitive::GainLife(AmountExpr::Fixed(5)),
            TargetSpec::You,
        );
        let ctx = bolt_ctx(bears_id, vec![]);
        game.resolve_effect(&heal, &ctx).unwrap();

        assert_eq!(game.players[0].life_total, 25);
    }

    #[test]
    fn test_sequence_bolt_and_draw() {
        let (mut game, bears_id) = setup_game_with_creature();

        // Put cards in library
        for _ in 0..3 {
            let card = CardDataBuilder::new("Forest")
                .card_type(CardType::Land)
                .build();
            let obj = GameObject::in_library(card, 0);
            let oid = obj.id;
            game.add_object(obj);
            game.players[0].library.push(oid);
        }

        let effect = Effect::Sequence(vec![
            Effect::Atom(
                Primitive::DealDamage(AmountExpr::Fixed(2)),
                TargetSpec::Any(crate::types::effects::TargetCount::Exactly(1)),
            ),
            Effect::Atom(
                Primitive::DrawCards(CountExpr::Fixed(1)),
                TargetSpec::You,
            ),
        ]);

        let ctx = bolt_ctx(bears_id, vec![ResolvedTarget::Player(1)]);
        game.resolve_effect(&effect, &ctx).unwrap();

        assert_eq!(game.players[1].life_total, 18);
        assert_eq!(game.players[0].hand.len(), 1);
    }
}
