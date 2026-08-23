use crate::engine::actions::GameAction;
use crate::engine::layers::types::{
    AffectedSet, ContinuousEffect, EffectModification, EffectOrigin, Layer,
};
use crate::events::event::DamageTarget;
use crate::objects::card_data::AbilityDef;
use crate::state::game_state::GameState;
use crate::types::effects::{
    AmountExpr, Duration, Effect, Primitive, EffectRecipient, SelectionFilter,
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
                //
                // The zone change goes through `execute_action(ZoneChange)` so
                // that the Phase 6 replacement pipeline can observe it.
                // `move_object` → `remove_from_zone_collection(Stack)` also
                // tears down the `StackEntry` for us.
                for target in &ctx.targets {
                    if let ResolvedTarget::Object(id) = target {
                        let id = *id;
                        if self.stack.contains(&id) {
                            self.change_zone(id, crate::types::zones::Zone::Graveyard)?;
                            self.events.emit(crate::events::event::GameEvent::SpellCountered {
                                spell_id: id,
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
                            if crate::oracle::characteristics::has_keyword(self, *id, crate::types::keywords::KeywordFlag::Indestructible) {
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

            // === Phase LB: continuous effect primitives ===

            Primitive::ModifyPowerToughness(power_expr, toughness_expr, duration) => {
                let power = self.evaluate_amount(power_expr, ctx)? as i32;
                let toughness = self.evaluate_amount(toughness_expr, ctx)? as i32;
                let target_ids = self.collect_battlefield_targets(ctx);
                if target_ids.is_empty() {
                    return Ok(());
                }
                let timestamp = self.allocate_timestamp();
                let effect = crate::engine::layers::ContinuousEffect {
                    id: 0,
                    source: ctx.source,
                    origin: crate::engine::layers::EffectOrigin::Resolution,
                    layer: crate::engine::layers::Layer::Layer7cModifyPT,
                    duration: *duration,
                    controller: ctx.controller,
                    created_on_turn: self.turn_number,
                    timestamp,
                    affected: crate::engine::layers::AffectedSet::Fixed(target_ids),
                    // CR 608.2h — a resolving spell locks its value in as it
                    // resolves, so this is `Fixed` even though the card text
                    // said "X". Static abilities are the ones that stay live.
                    modification: crate::engine::layers::EffectModification::ModifyPowerToughness {
                        power: crate::engine::layers::types::PtValue::Fixed(power),
                        toughness: crate::engine::layers::types::PtValue::Fixed(toughness),
                    },
                };
                self.continuous_effects.add(effect);
                Ok(())
            }

            Primitive::SetPowerToughness(power_expr, toughness_expr, duration) => {
                let power = self.evaluate_amount(power_expr, ctx)? as i32;
                let toughness = self.evaluate_amount(toughness_expr, ctx)? as i32;
                let target_ids = self.collect_battlefield_targets(ctx);
                if target_ids.is_empty() {
                    return Ok(());
                }
                let timestamp = self.allocate_timestamp();
                let effect = crate::engine::layers::ContinuousEffect {
                    id: 0,
                    source: ctx.source,
                    origin: crate::engine::layers::EffectOrigin::Resolution,
                    layer: crate::engine::layers::Layer::Layer7bSetPT,
                    duration: *duration,
                    controller: ctx.controller,
                    created_on_turn: self.turn_number,
                    timestamp,
                    affected: crate::engine::layers::AffectedSet::Fixed(target_ids),
                    modification: crate::engine::layers::EffectModification::SetPowerToughness {
                        power: crate::engine::layers::types::PtValue::Fixed(power),
                        toughness: crate::engine::layers::types::PtValue::Fixed(toughness),
                    },
                };
                self.continuous_effects.add(effect);
                Ok(())
            }

            Primitive::SwitchPowerToughness(duration) => {
                let target_ids = self.collect_battlefield_targets(ctx);
                if target_ids.is_empty() {
                    return Ok(());
                }
                let timestamp = self.allocate_timestamp();
                let effect = crate::engine::layers::ContinuousEffect {
                    id: 0,
                    source: ctx.source,
                    origin: crate::engine::layers::EffectOrigin::Resolution,
                    layer: crate::engine::layers::Layer::Layer7dSwitchPT,
                    duration: *duration,
                    controller: ctx.controller,
                    created_on_turn: self.turn_number,
                    timestamp,
                    affected: crate::engine::layers::AffectedSet::Fixed(target_ids),
                    modification: crate::engine::layers::EffectModification::SwitchPowerToughness,
                };
                self.continuous_effects.add(effect);
                Ok(())
            }

            // === Phase LC: Layer 5 color-changing effects ===

            Primitive::ChangeColor(color_change, duration) => {
                use crate::types::effects::ColorChange;
                let target_ids = self.collect_battlefield_targets(ctx);
                if target_ids.is_empty() {
                    return Ok(());
                }
                let modification = match color_change {
                    ColorChange::Add(c) => crate::engine::layers::EffectModification::AddColor(*c),
                    ColorChange::Set(colors) => crate::engine::layers::EffectModification::SetColors(colors.clone()),
                    ColorChange::RemoveAll => crate::engine::layers::EffectModification::RemoveAllColors,
                };
                let timestamp = self.allocate_timestamp();
                let effect = crate::engine::layers::ContinuousEffect {
                    id: 0,
                    source: ctx.source,
                    origin: crate::engine::layers::EffectOrigin::Resolution,
                    layer: crate::engine::layers::Layer::Layer5Color,
                    duration: *duration,
                    controller: ctx.controller,
                    created_on_turn: self.turn_number,
                    timestamp,
                    affected: crate::engine::layers::AffectedSet::Fixed(target_ids),
                    modification,
                };
                self.continuous_effects.add(effect);
                Ok(())
            }

            // === Phase LD: Layer 4 type-changing effects ===

            Primitive::ChangeType(type_change, duration) => {
                let target_ids = self.collect_battlefield_targets(ctx);
                if target_ids.is_empty() {
                    return Ok(());
                }

                // A TypeChange may produce multiple EffectModification entries,
                // each belonging to Layer 4. They share a timestamp and source
                // per CR 613.1c (sibling entries for a single effect).
                let timestamp = self.allocate_timestamp();
                let mut modifications: Vec<crate::engine::layers::EffectModification> = Vec::new();

                // Types: set takes priority over add/remove
                if let Some(ref set_types) = type_change.set_types {
                    modifications.push(crate::engine::layers::EffectModification::SetTypes(set_types.clone()));
                } else {
                    for t in &type_change.add_types {
                        modifications.push(crate::engine::layers::EffectModification::AddType(*t));
                    }
                    for t in &type_change.remove_types {
                        modifications.push(crate::engine::layers::EffectModification::RemoveType(*t));
                    }
                }

                // Subtypes: set takes priority over add/remove
                if let Some(ref set_subtypes) = type_change.set_subtypes {
                    modifications.push(crate::engine::layers::EffectModification::SetSubtypes(set_subtypes.clone()));
                } else {
                    for s in &type_change.add_subtypes {
                        modifications.push(crate::engine::layers::EffectModification::AddSubtype(s.clone()));
                    }
                    for s in &type_change.remove_subtypes {
                        modifications.push(crate::engine::layers::EffectModification::RemoveSubtype(s.clone()));
                    }
                }

                // Supertypes: set takes priority over add/remove
                if let Some(ref set_supertypes) = type_change.set_supertypes {
                    modifications.push(crate::engine::layers::EffectModification::SetSupertypes(set_supertypes.clone()));
                } else {
                    for s in &type_change.add_supertypes {
                        modifications.push(crate::engine::layers::EffectModification::AddSupertype(*s));
                    }
                    for s in &type_change.remove_supertypes {
                        modifications.push(crate::engine::layers::EffectModification::RemoveSupertype(*s));
                    }
                }

                // Register one ContinuousEffect per modification (sibling entries
                // sharing timestamp+source, per CR 613.1c).
                for modification in modifications {
                    let effect = crate::engine::layers::ContinuousEffect {
                        id: 0,
                        source: ctx.source,
                        origin: crate::engine::layers::EffectOrigin::Resolution,
                        layer: crate::engine::layers::Layer::Layer4Type,
                        duration: *duration,
                        controller: ctx.controller,
                        created_on_turn: self.turn_number,
                        timestamp,
                        affected: crate::engine::layers::AffectedSet::Fixed(target_ids.clone()),
                        modification,
                    };
                    self.continuous_effects.add(effect);
                }
                Ok(())
            }

            // === Layer 6 — ability adding and removing (CR 613.1f) ===
            //
            // The resolution half. `register_static_effects` handles printed
            // static abilities; these are the "target creature gains/loses ..."
            // spells, whose affected set is locked to the targets at resolution
            // (CR 613.7b).

            Primitive::GrantKeyword(keyword, duration) => {
                self.register_layer6_resolution(
                    ctx,
                    *duration,
                    EffectModification::GrantKeyword(*keyword),
                );
                Ok(())
            }

            Primitive::RemoveKeyword(keyword, duration) => {
                self.register_layer6_resolution(
                    ctx,
                    *duration,
                    EffectModification::RemoveKeyword(*keyword),
                );
                Ok(())
            }

            Primitive::LoseAbility(ability_id, duration) => {
                self.register_layer6_resolution(
                    ctx,
                    *duration,
                    EffectModification::LoseAbility(*ability_id),
                );
                Ok(())
            }

            Primitive::LoseAllAbilities(duration) => {
                self.register_layer6_resolution(
                    ctx,
                    *duration,
                    EffectModification::LoseAllAbilities,
                );
                Ok(())
            }

            Primitive::GrantAbility(def, duration) => {
                let granted = self.register_layer6_resolution(
                    ctx,
                    *duration,
                    EffectModification::GrantAbility(def.clone()),
                );

                // CR 613.7a clause 2. If the granted ability is itself static,
                // it generates continuous effects of its own, and those are
                // timestamped from *this* effect rather than from the object.
                if let Some((targets, timestamp)) = granted {
                    for grantee in targets {
                        self.register_granted_static_effects(
                            def, grantee, timestamp, *duration, ctx.controller,
                        );
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
            | Primitive::GainControl(_) => {
                Err(format!("Primitive {:?} not yet implemented", primitive))
            }
        }
    }

    // --- Helpers: Layer 6 (CR 613.1f) ---

    /// Register one Layer 6 row over this resolution's battlefield targets.
    ///
    /// Returns the targets and the timestamp it used, or `None` if the spell
    /// had no legal battlefield target left — `GrantAbility` needs both to
    /// register the effects a granted static ability generates.
    fn register_layer6_resolution(
        &mut self,
        ctx: &ResolutionContext,
        duration: Duration,
        modification: EffectModification,
    ) -> Option<(Vec<ObjectId>, u64)> {
        let target_ids = self.collect_battlefield_targets(ctx);
        if target_ids.is_empty() {
            return None;
        }
        let timestamp = self.allocate_timestamp();
        self.continuous_effects.add(ContinuousEffect {
            id: 0,
            source: ctx.source,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer6Ability,
            duration,
            controller: ctx.controller,
            created_on_turn: self.turn_number,
            timestamp,
            affected: AffectedSet::Fixed(target_ids.clone()),
            modification,
        });
        Some((target_ids, timestamp))
    }

    /// CR 613.7a clause 2 — register the continuous effects generated by a
    /// static ability that an effect *granted* to `grantee`.
    ///
    /// > A continuous effect generated by a static ability has the same
    /// > timestamp as the object the static ability is on, **or the timestamp of
    /// > the effect that created the ability, whichever is later.**
    ///
    /// `granting_timestamp` is that second clause, and `static_effect_timestamp`
    /// takes the max. Without it a granted "creatures you control get +1/+1"
    /// would sort as though it had been printed on a permanent that has been on
    /// the battlefield for ten turns, and lose to effects it should beat.
    ///
    /// **Existence comes for free.** The rows are `EffectOrigin::StaticAbility`
    /// keyed on the *granted* ability's id, so `compute::static_ability_still_
    /// exists` re-asks at every layer whether `grantee` still has it. If the
    /// grant stops applying, or a later Layer 6 effect strips the ability, the
    /// derived effect retires with it. Nothing extra to maintain.
    ///
    /// **Only reachable when the grantee set is known at grant time**, which is
    /// every resolution-time grant (the affected set is locked to the targets by
    /// CR 613.7b). A *static* ability that grants a static ability over a filter
    /// has a set that changes with the board, so it derives nothing — that is
    /// the remaining half of Deferred Migrations item 7.
    fn register_granted_static_effects(
        &mut self,
        granted: &AbilityDef,
        grantee: ObjectId,
        granting_timestamp: u64,
        duration: Duration,
        controller: PlayerId,
    ) {
        use crate::objects::card_data::AbilityType;

        if granted.ability_type != AbilityType::Static {
            return;
        }
        // CR 604.3a(2) — a granted ability is never a CDA, so it never applies
        // intrinsically. `apply_modification` clears the flag on the frame; this
        // is the same rule reaching the registry side.
        if granted.is_characteristic_defining {
            return;
        }

        let atoms: Vec<(&Primitive, &EffectRecipient)> = match &granted.effect {
            Effect::Atom(p, r) => vec![(p, r)],
            Effect::Sequence(effects) => effects
                .iter()
                .filter_map(|e| if let Effect::Atom(p, r) = e { Some((p, r)) } else { None })
                .collect(),
            _ => return,
        };

        for (primitive, recipient) in atoms {
            let affected = match recipient {
                EffectRecipient::Implicit => AffectedSet::SourceOnly,
                EffectRecipient::FilteredPermanents(filter) => AffectedSet::Filter {
                    filter: filter.clone(),
                    controller: GameState::extract_controller_from_filter(filter, controller),
                },
                _ => continue,
            };

            let rows = GameState::static_primitive_rows(primitive);
            if rows.is_empty() {
                continue;
            }
            let timestamp =
                self.static_effect_timestamp(grantee, granted, Some(granting_timestamp));

            for (layer, modification) in rows {
                // A granted static ability whose own effect lands in layers 1-6
                // does not work, and it fails *silently* rather than wrongly:
                // the grant applies AT layer 6, so at any layer <= 6 the frame
                // the CR 613.7a existence check reads is the frame from before
                // the grant, and the derived effect finds no ability to justify
                // itself.
                //
                // This is not a corner. It is exactly CR 613.7a's own worked
                // example -- Rune of Flight grants "Equipped creature has
                // flying", a layer 6 effect. That card also needs Equip, so it
                // is out of reach twice over. Assert loudly at the authoring
                // site rather than let a card quietly do nothing; see Deferred
                // Migrations item 7b.
                debug_assert!(
                    layer > Layer::Layer6Ability,
                    "granted static ability generates a {:?} effect; layers 1-6                      are unreachable for a grant that applies at layer 6                      (CR 613.7a, Deferred Migrations item 7b)",
                    layer
                );

                self.continuous_effects.add(ContinuousEffect {
                    id: 0,
                    source: grantee,
                    origin: EffectOrigin::StaticAbility { ability: granted.id },
                    layer,
                    duration,
                    controller,
                    created_on_turn: self.turn_number,
                    timestamp,
                    affected: affected.clone(),
                    modification,
                });
            }
        }
    }

    // --- Helper: collect battlefield targets ---

    /// Extract object IDs from resolved targets that are currently on the battlefield.
    fn collect_battlefield_targets(&self, ctx: &ResolutionContext) -> Vec<ObjectId> {
        ctx.targets.iter()
            .filter_map(|t| {
                if let ResolvedTarget::Object(id) = t {
                    if self.battlefield.contains_key(id) {
                        return Some(*id);
                    }
                }
                None
            })
            .collect()
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
            AmountExpr::CardTypesAmong(_selector) => {
                Err("CardTypesAmong amount resolution not yet implemented".to_string())
            }
            AmountExpr::Plus(inner, n) => Ok(self.evaluate_amount(inner, _ctx)? + n),
            AmountExpr::TargetPower => {
                Err("TargetPower amount resolution not yet implemented".to_string())
            }
            AmountExpr::TargetToughness => {
                Err("TargetToughness amount resolution not yet implemented".to_string())
            }
            AmountExpr::DamageDealt => {
                Err("DamageDealt amount resolution not yet implemented".to_string())
            }
            // Meaningful only inside the layer walk, where "it" is the object
            // the continuous effect is being applied to. A resolving spell has
            // no such object — see `compute::evaluate_pt_value`.
            AmountExpr::AffectedManaValue => Err(
                "AffectedManaValue has no meaning at resolution time; it is evaluated \
                 per-object during the layer walk"
                    .to_string(),
            ),
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
        if !crate::oracle::characteristics::has_subtype(
            self, aura_id, &Subtype::Enchantment(EnchantmentType::Aura),
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

        let recipient = EffectRecipient::Choose(filter.clone(), TargetCount::Exactly(1));
        let legal = crate::oracle::legality::enumerate_legal_selections(self, &filter, Some(aura_id));
        let choices = crate::ui::ask::ask_select_recipients(
            dp, self, controller, &recipient, aura_id,
            &legal, 1, 1,
        );

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
    use crate::test_support::{pacifism as make_pacifism, test_dp};

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

    #[test]
    fn test_deal_damage_to_creature() {
        let (mut game, bears_id) = setup_game_with_creature();

        let bolt = Effect::Atom(
            Primitive::DealDamage(AmountExpr::Fixed(3)),
            EffectRecipient::Target(SelectionFilter::Any, crate::types::effects::TargetCount::Exactly(1)),
        );

        let ctx = bolt_ctx(bears_id, vec![ResolvedTarget::Object(bears_id)]);
        game.resolve_effect(&bolt, &ctx, &test_dp()).unwrap();

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
        game.resolve_effect(&bolt, &ctx, &test_dp()).unwrap();

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
        game.resolve_effect(&draw, &ctx, &test_dp()).unwrap();

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
        game.resolve_effect(&heal, &ctx, &test_dp()).unwrap();

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
        game.resolve_effect(&effect, &ctx, &test_dp()).unwrap();

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
            .keyword(crate::types::keywords::KeywordFlag::Indestructible)
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
        game.resolve_effect(&destroy, &ctx, &test_dp()).unwrap();

        // Creature should still be on the battlefield
        assert!(game.battlefield.contains_key(&target_id));
    }

    // --- attach_aura_on_etb tests (rule 303.4a) ---

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
        // Legal selections: [Object(creature_id)] — index 0
        dp.expect_pick_n(
            crate::ui::choice_types::ChoiceKind::SelectRecipients {
                recipient: crate::types::effects::EffectRecipient::Choose(
                    SelectionFilter::Creature,
                    crate::types::effects::TargetCount::Exactly(1),
                ),
                spell_id: aura_id,
            },
            vec![0],
        );

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
        let attached = game.attach_aura_on_etb(aura_id, 0, &test_dp()).unwrap();
        assert!(!attached);

        // Aura is still on battlefield but unattached
        assert!(game.battlefield.contains_key(&aura_id));
        assert_eq!(game.battlefield.get(&aura_id).unwrap().attached_to, None);

        // SBA should now kill it (704.5m)
        let performed = game.check_state_based_actions(&test_dp()).unwrap();
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

        let result = game.attach_aura_on_etb(creature_id, 0, &test_dp()).unwrap();
        assert!(!result);
    }
}
