use crate::engine::actions::GameAction;
use crate::events::event::DamageTarget;
use crate::state::game_state::GameState;
use crate::types::effects::{
    AmountExpr, Effect, Primitive, EffectRecipient, SelectionFilter,
};
use crate::types::ids::{ObjectId, PlayerId};
use crate::ui::decision::DecisionProvider;

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
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        match effect {
            Effect::Atom(primitive, recipient) => {
                self.resolve_primitive(primitive, recipient, ctx, dp)
            }

            Effect::Sequence(effects) => {
                for sub in effects {
                    self.resolve_effect(sub, ctx, dp)?;
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
        recipient: &EffectRecipient,
        ctx: &ResolutionContext,
        _dp: &dyn DecisionProvider,
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

            Primitive::DrawCards(amount_expr) => {
                let count = self.evaluate_amount(amount_expr, ctx)?;
                // Drawing targets the controller (EffectRecipient::Controller or None)
                let player_id = self.resolve_player_for_self(recipient, ctx);
                for _ in 0..count {
                    self.execute_action(GameAction::DrawCard {
                        player: player_id,
                    })?;
                }
                Ok(())
            }

            Primitive::GainLife(amount_expr) => {
                let amount = self.evaluate_amount(amount_expr, ctx)?;
                let player_id = self.resolve_player_for_self(recipient, ctx);
                self.execute_action(GameAction::GainLife {
                    player: player_id,
                    amount,
                    source: ctx.source,
                })?;
                Ok(())
            }

            Primitive::LoseLife(amount_expr) => {
                let amount = self.evaluate_amount(amount_expr, ctx)?;
                let player_id = self.resolve_player_for_self(recipient, ctx);
                self.execute_action(GameAction::LoseLife {
                    player: player_id,
                    amount,
                })?;
                Ok(())
            }

            Primitive::ProduceMana(output) => {
                // Evaluate dynamic amounts before taking &mut player
                let resolved: Vec<_> = output.mana.iter()
                    .map(|(mt, expr)| Ok((*mt, self.evaluate_amount(expr, ctx)?)))
                    .collect::<Result<_, String>>()?;
                let player = self.get_player_mut(ctx.controller)?;
                for (mana_type, amount) in resolved {
                    player.mana_pool.add(mana_type, amount);
                }
                for atom in &output.special {
                    player.mana_pool.add_special(atom.clone());
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
                            // Clean up the StackEntry for the countered spell
                            self.stack_entries.remove(&countered_id);
                            let obj = self.get_object(countered_id)?;
                            let owner = obj.owner;
                            self.get_player_mut(owner)?.graveyard.push(countered_id);
                            let obj_mut = self.get_object_mut(countered_id)?;
                            obj_mut.zone = crate::types::zones::Zone::Graveyard;
                            // Emit ZoneChange event
                            self.events.emit(crate::events::event::GameEvent::ZoneChange {
                                object_id: countered_id,
                                owner,
                                from: crate::types::zones::Zone::Stack,
                                to: crate::types::zones::Zone::Graveyard,
                            });
                            self.events.emit(crate::events::event::GameEvent::SpellCountered {
                                spell_id: countered_id,
                                countered_by: ctx.source,
                            });
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
                            // Clean up the StackEntry for the countered ability
                            self.stack_entries.remove(&removed_id);
                            // Remove the object entirely — abilities on the
                            // stack are not cards and have no destination zone.
                            self.objects.remove(&removed_id);
                            self.events.emit(crate::events::event::GameEvent::AbilityCountered {
                                ability_id: removed_id,
                                countered_by: ctx.source,
                            });
                        }
                    }
                }
                Ok(())
            }

            // === Phase 2 primitives: Destroy & Untap ===

            Primitive::Destroy => {
                // Destroy target permanent (rule 701.7a).
                // Moves the permanent from battlefield to its owner's graveyard.
                // Indestructible permanents can't be destroyed (rule 702.12b).
                for target in &ctx.targets {
                    if let ResolvedTarget::Object(id) = target {
                        if self.battlefield.contains_key(id) {
                            if crate::oracle::characteristics::has_keyword(self, *id, crate::types::keywords::KeywordAbility::Indestructible) {
                                continue;
                            }
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

    // --- Helper: Aura non-stack ETB (rule 303.4a) ---

    /// When an Aura enters the battlefield *not* from the stack (e.g.
    /// returned from graveyard by an effect), it doesn't target — the
    /// controller chooses a legal object to attach to (rule 303.4a).
    ///
    /// If no legal host exists the Aura stays unattached; the 704.5m SBA
    /// will move it to the graveyard on the next SBA check.
    ///
    /// This must be called *after* the Aura is already on the battlefield
    /// (i.e. after `move_object` / `place_on_battlefield`).
    ///
    /// Returns `Ok(true)` if a host was chosen and attached, `Ok(false)` if
    /// no host was chosen (Aura left unattached for SBA), or `Err` on a
    /// hard failure.
    pub fn attach_aura_on_etb(
        &mut self,
        aura_id: ObjectId,
        controller: PlayerId,
        dp: &dyn DecisionProvider,
    ) -> Result<bool, String> {
        use crate::types::card_types::{EnchantmentType, Subtype};
        use crate::types::effects::{EffectRecipient, TargetCount};

        let obj = self.get_object(aura_id)?;

        // Only applies to Auras.
        if !obj.card_data.subtypes.contains(
            &Subtype::Enchantment(EnchantmentType::Aura),
        ) {
            return Ok(false);
        }

        // Read the enchant filter directly from card data.
        let filter = match &obj.card_data.enchant_filter {
            Some(f) => f.clone(),
            // Aura with no enchant_filter — card data bug.
            // Fall back to "enchant permanent" so the game doesn't crash,
            // but warn loudly so we catch it.
            None => {
                let name = &self.get_object(aura_id)?.card_data.name;
                eprintln!(
                    "[WARN] Aura {:?} (id={}) has no enchant_filter set — \
                     falling back to \"enchant permanent\". This is a card data bug.",
                    name, aura_id
                );
                SelectionFilter::Permanent(
                    crate::types::effects::PermanentFilter::All,
                )
            }
        };

        // Pre-check: is there at least one legal host?
        // Skip the DP prompt entirely if not — no point asking the player
        // to choose from an empty set.
        if !self.has_any_legal_choice(&filter, Some(aura_id)) {
            // No legal host exists. Aura stays unattached; 704.5m SBA
            // will put it into the graveyard.
            return Ok(false);
        }

        let recipient = EffectRecipient::Choose(filter, TargetCount::Exactly(1));
        let choices = dp.choose_targets(self, controller, &recipient);

        if let Some(ResolvedTarget::Object(host_id)) = choices.first() {
            let host_id = *host_id;
            if let Some(aura_bf) = self.battlefield.get_mut(&aura_id) {
                aura_bf.attach_to(host_id);
            }
            if let Some(host_bf) = self.battlefield.get_mut(&host_id) {
                host_bf.attached_by.push(aura_id);
            }
            Ok(true)
        } else {
            // No legal host chosen — Aura stays unattached.
            // 704.5m SBA will put it into the graveyard.
            Ok(false)
        }
    }

    // --- Helper: determine which player an effect applies to ---

    /// For effects that target "you" (the controller) or use EffectRecipient::Implicit,
    /// returns the controller. For targeted player effects, returns the first
    /// player target.
    fn resolve_player_for_self(
        &self,
        recipient: &EffectRecipient,
        ctx: &ResolutionContext,
    ) -> PlayerId {
        match recipient {
            EffectRecipient::Implicit | EffectRecipient::Controller => ctx.controller,
            EffectRecipient::Target(SelectionFilter::Player, _) => {
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
            .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::Green], 1))
            .color(crate::types::colors::Color::Green)
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Bear))
            .power_toughness(2, 2)
            .build();

        let obj = GameObject::new(bears, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        let entry = BattlefieldEntity::new(id, 0, 0, 1);
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

    fn passive_dp() -> crate::ui::decision::PassiveDecisionProvider {
        crate::ui::decision::PassiveDecisionProvider
    }

    #[test]
    fn test_deal_damage_to_creature() {
        let (mut game, bears_id) = setup_game_with_creature();

        let bolt = Effect::Atom(
            Primitive::DealDamage(AmountExpr::Fixed(3)),
            EffectRecipient::Target(SelectionFilter::Any, crate::types::effects::TargetCount::Exactly(1)),
        );

        let ctx = bolt_ctx(bears_id, vec![ResolvedTarget::Object(bears_id)]);
        game.resolve_effect(&bolt, &ctx, &passive_dp()).unwrap();

        assert_eq!(game.battlefield.get(&bears_id).unwrap().damage_marked, 3);
    }

    #[test]
    fn test_deal_damage_to_player() {
        let (mut game, bears_id) = setup_game_with_creature();

        let bolt = Effect::Atom(
            Primitive::DealDamage(AmountExpr::Fixed(3)),
            EffectRecipient::Target(SelectionFilter::Any, crate::types::effects::TargetCount::Exactly(1)),
        );

        let ctx = bolt_ctx(bears_id, vec![ResolvedTarget::Player(1)]);
        game.resolve_effect(&bolt, &ctx, &passive_dp()).unwrap();

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
            Primitive::DrawCards(AmountExpr::Fixed(2)),
            EffectRecipient::Controller,
        );
        let ctx = bolt_ctx(bears_id, vec![]);
        game.resolve_effect(&draw, &ctx, &passive_dp()).unwrap();

        assert_eq!(game.players[0].hand.len(), 2);
        assert_eq!(game.players[0].library.len(), 3);
    }

    #[test]
    fn test_gain_life() {
        let (mut game, bears_id) = setup_game_with_creature();

        let heal = Effect::Atom(
            Primitive::GainLife(AmountExpr::Fixed(5)),
            EffectRecipient::Controller,
        );
        let ctx = bolt_ctx(bears_id, vec![]);
        game.resolve_effect(&heal, &ctx, &passive_dp()).unwrap();

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
                EffectRecipient::Target(SelectionFilter::Any, crate::types::effects::TargetCount::Exactly(1)),
            ),
            Effect::Atom(
                Primitive::DrawCards(AmountExpr::Fixed(1)),
                EffectRecipient::Controller,
            ),
        ]);

        let ctx = bolt_ctx(bears_id, vec![ResolvedTarget::Player(1)]);
        game.resolve_effect(&effect, &ctx, &passive_dp()).unwrap();

        assert_eq!(game.players[1].life_total, 18);
        assert_eq!(game.players[0].hand.len(), 1);
    }

    // --- Indestructible guard tests ---

    #[test]
    fn test_sba_indestructible_survives_destroy() {
        // 702.12b — Destroy effect does nothing to an indestructible permanent.
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Darksteel Myr")
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Myr))
            .power_toughness(0, 1)
            .keyword(crate::types::keywords::KeywordAbility::Indestructible)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let target_id = obj.id;
        game.add_object(obj);
        let entry = BattlefieldEntity::new(target_id, 0, 0, 1);
        game.battlefield.insert(target_id, entry);

        // Create a source for the destroy effect
        let bolt_data = CardDataBuilder::new("Doom Blade")
            .card_type(CardType::Instant)
            .build();
        let bolt_obj = GameObject::new(bolt_data, 0, Zone::Hand);
        let source_id = bolt_obj.id;
        game.add_object(bolt_obj);

        let destroy = Effect::Atom(
            Primitive::Destroy,
            EffectRecipient::Target(SelectionFilter::Creature, crate::types::effects::TargetCount::Exactly(1)),
        );
        let ctx = bolt_ctx(source_id, vec![ResolvedTarget::Object(target_id)]);
        game.resolve_effect(&destroy, &ctx, &passive_dp()).unwrap();

        // Creature should still be on the battlefield
        assert!(game.battlefield.contains_key(&target_id));
    }

    // --- attach_aura_on_etb tests (rule 303.4a) ---

    fn make_pacifism() -> std::sync::Arc<crate::objects::card_data::CardData> {
        CardDataBuilder::new("Pacifism")
            .card_type(CardType::Enchantment)
            .subtype(Subtype::Enchantment(crate::types::card_types::EnchantmentType::Aura))
            .color(crate::types::colors::Color::White)
            .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::White], 1))
            .enchant_filter(SelectionFilter::Creature)
            .build()
    }

    #[test]
    fn test_aura_etb_non_stack_chooses_host() {
        // Rule 303.4a: Aura entering the battlefield not from the stack —
        // controller chooses a legal host. No targeting rules apply.
        let mut game = GameState::new(2, 20);

        // Put a creature on the battlefield (valid host)
        let creature = GameObject::new(
            CardDataBuilder::new("Grizzly Bears")
                .card_type(CardType::Creature)
                .power_toughness(2, 2)
                .build(),
            1,
            Zone::Battlefield,
        );
        let creature_id = creature.id;
        game.add_object(creature);
        game.place_on_battlefield(creature_id, 1);

        // Put the Aura on the battlefield (simulating a non-stack ETB)
        let aura = GameObject::new(make_pacifism(), 0, Zone::Battlefield);
        let aura_id = aura.id;
        game.add_object(aura);
        game.place_on_battlefield(aura_id, 0);

        // Script the DP to choose the creature as host
        let dp = crate::ui::decision::ScriptedDecisionProvider::new();
        dp.target_decisions.borrow_mut().push(vec![ResolvedTarget::Object(creature_id)]);

        let attached = game.attach_aura_on_etb(aura_id, 0, &dp).unwrap();
        assert!(attached);

        // Aura should be attached to the creature
        let aura_bf = game.battlefield.get(&aura_id).unwrap();
        assert_eq!(aura_bf.attached_to, Some(creature_id));

        // Host should list the Aura
        let host_bf = game.battlefield.get(&creature_id).unwrap();
        assert!(host_bf.attached_by.contains(&aura_id));
    }

    #[test]
    fn test_aura_etb_non_stack_no_legal_host() {
        // Rule 303.4a + 704.5m: Aura enters with no legal host —
        // stays unattached, SBA will handle it.
        let mut game = GameState::new(2, 20);

        // No creatures on the battlefield — Pacifism has no legal host

        // Put the Aura on the battlefield (simulating a non-stack ETB)
        let aura = GameObject::new(make_pacifism(), 0, Zone::Battlefield);
        let aura_id = aura.id;
        game.add_object(aura);
        game.place_on_battlefield(aura_id, 0);

        // PassiveDP returns empty — no host chosen
        let attached = game.attach_aura_on_etb(aura_id, 0, &passive_dp()).unwrap();
        assert!(!attached);

        // Aura is still on battlefield but unattached
        assert!(game.battlefield.contains_key(&aura_id));
        assert_eq!(game.battlefield.get(&aura_id).unwrap().attached_to, None);

        // SBA should now kill it (704.5m)
        let performed = game.check_state_based_actions(&passive_dp()).unwrap();
        assert!(performed);
        assert!(!game.battlefield.contains_key(&aura_id));
        assert_eq!(game.get_object(aura_id).unwrap().zone, Zone::Graveyard);
    }

    #[test]
    fn test_attach_aura_on_etb_non_aura_is_noop() {
        // Non-Aura permanents should return Ok(false) and do nothing.
        let mut game = GameState::new(2, 20);

        let creature = GameObject::new(
            CardDataBuilder::new("Grizzly Bears")
                .card_type(CardType::Creature)
                .power_toughness(2, 2)
                .build(),
            0,
            Zone::Battlefield,
        );
        let creature_id = creature.id;
        game.add_object(creature);
        game.place_on_battlefield(creature_id, 0);

        let result = game.attach_aura_on_etb(creature_id, 0, &passive_dp()).unwrap();
        assert!(!result);
    }
}
