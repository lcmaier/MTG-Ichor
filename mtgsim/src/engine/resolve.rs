use crate::engine::actions::{ActionContext, DestructionSource, GameAction, ZoneChangeCause};
use crate::engine::layers::types::{
    AffectedSet, ContinuousEffect, EffectModification, EffectOrigin, Layer, Timestamp,
};
use crate::events::event::DamageTarget;
use crate::objects::card_data::AbilityDef;
use crate::objects::object::GameObject;
use crate::types::zones::Zone;
use crate::state::game_state::GameState;
use crate::types::effects::{
    AmountExpr, Duration, Effect, Primitive, EffectRecipient, PlayerRef, SelectionFilter,
    TargetCount,
};
use crate::oracle::characteristics::get_effective_controller;
use crate::state::replacement_effects::RegisteredReplacementEffect;
use crate::state::restrictions::RegisteredRestriction;
use crate::types::restriction::{Restriction, RestrictionDef};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::{EventPattern, ReplacementDef, Rewrite};
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

            // CR 614.1a. A replacement effect written as a *static* ability
            // never reaches here — `engine::replacement::gather` reads it off
            // the source's effective ability list, which is what lets Humility
            // strip it. This arm is the other half: a replacement effect
            // created by a *resolution*, which CR 614.3 gives a duration
            // ("prevent all damage that would be dealt this turn").
            //
            // Loud rather than silently registering an endless one, because the
            // duration is the missing piece and there is nowhere honest to read
            // it from. CR 701.19a's regeneration shield — the one resolution-
            // created replacement Phase RB has — is a keyword action and comes
            // through `Primitive::Regenerate`, which knows both its duration
            // (this turn) and its affected set (the targets).
            Effect::Replacement(_) => Err(
                "a replacement effect created by a resolution needs a CR 614.3 \
                 duration, which `Effect::Replacement` does not carry (Phase RD). \
                 A static ability's replacement effect does not resolve at all — \
                 put it on an `AbilityType::Static` ability and \
                 `engine::replacement::gather` will find it. For CR 701.19a's \
                 regeneration shield, use `Primitive::Regenerate`."
                    .to_string(),
            ),

            // CR 101.2's twin of the arm above, and loud for the same reason.
            // A "can't" written as a *static* ability never reaches here —
            // `engine::restriction::is_prohibited` reads it off the source's
            // effective ability list. A "can't" created by a *resolution* needs
            // a scope, and CR 608.2c will not let the engine infer one: it says
            // to "apply the rules of English to the text", which is the CR
            // handing scope determination to a human reader (§9 finding 1).
            Effect::Restriction(_) => Err(
                "a \"can't\" effect created by a resolution needs a duration,                  which `Effect::Restriction` does not carry. CR 608.2c makes the                  scope unrecoverable from the restriction's own text, so it has                  to be authored: use `Primitive::Restrict`, which takes a                  `Duration` argument. A static ability's restriction does not                  resolve at all — put it on an `AbilityType::Static` ability and                  `engine::restriction::is_prohibited` will find it."
                    .to_string(),
            ),

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
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        // Every mutation a primitive proposes belongs to *this* resolution
        // (CR 614.15 / the resolution stamp on each emitted event).
        let actx = ActionContext::resolving(dp, ctx);
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
                    }, &actx)?;
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
                    }, &actx)?;
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
                }, &actx)?;
                Ok(())
            }

            Primitive::LoseLife(amount_expr) => {
                let amount = self.evaluate_amount(amount_expr, ctx)?;
                let player_id = self.resolve_player_for_self(recipient, ctx);
                self.execute_action(GameAction::LoseLife {
                    player: player_id,
                    amount,
                }, &actx)?;
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
                            self.change_zone(id, crate::types::zones::Zone::Graveyard, ZoneChangeCause::Countered, &actx)?;
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
                //
                // **One batch, because CR 608.2f says so.** "Some spells and
                // abilities include actions taken on multiple players and/or
                // objects. In most cases, each such action is processed
                // simultaneously." A board wipe destroys everything at one
                // instant, so the deaths are one event — which is what lets a
                // CR 614 replacement apply once *per death* rather than the
                // pipeline seeing N unrelated events, and what CR 615.7's
                // shield allocation and CR 603.2c's "one or more creatures die"
                // both read. This was a loop of `execute_action` until
                // 2026-08-26; §4.2 named this caller class ("any 'each
                // player ...' effect") and RA-3 converted only two of the three.
                // **Indestructible is no longer filtered here.** CR 702.12b
                // makes it a "can't" (CR 614.17), not a replacement effect, so
                // it belongs ahead of the pipeline rather than ahead of the
                // proposal — `engine::replacement::is_blocked`. Filtering here
                // was observationally right and structurally wrong: a "can't"
                // that never becomes a proposal cannot be replaced by a
                // CR 614.15 self-replacement (614.17c), and nothing downstream
                // could tell "no such permanent" from "it can't be destroyed".
                let mut batch = Vec::new();
                for target in &ctx.targets {
                    if let ResolvedTarget::Object(id) = target {
                        if self.battlefield.contains_key(id) {
                            batch.push(GameAction::Destroy {
                                object: *id,
                                source: DestructionSource::Effect(ctx.source),
                            });
                        }
                        // If not on battlefield, destroy does nothing (rule 701.8b)
                    }
                }
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            Primitive::Untap => {
                // Untap target permanent (rule 701.26b).
                for target in &ctx.targets {
                    if let ResolvedTarget::Object(id) = target {
                        // CR 608.2b: a spell whose targets are not *all* illegal
                        // still resolves and does as much as it can, so a target
                        // that has left the battlefield is skipped rather than
                        // erroring. The performer is loud (RA-2), which makes
                        // checking here the caller's job — same shape as
                        // `Primitive::Destroy` above.
                        if !self.battlefield.contains_key(id) {
                            continue;
                        }
                        self.execute_action(GameAction::Untap {
                            object: *id,
                        }, &actx)?;
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
                // each belonging to Layer 4. CR 613.6 is why they are siblings
                // ("the parts of the effect each apply in their appropriate"
                // layers), and CR 613.7b is why they share a timestamp — one
                // resolution, one moment of creation.
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

                // Register one ContinuousEffect per modification: siblings of
                // one CR 613.6 effect, sharing the source and the CR 613.7b
                // creation timestamp.
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

            Primitive::GrantKeywordFlag(keyword, duration) => {
                self.register_resolution_ability_effect(
                    ctx,
                    *duration,
                    EffectModification::GrantKeywordFlag(*keyword),
                );
                Ok(())
            }

            Primitive::RemoveKeywordFlag(keyword, duration) => {
                self.register_resolution_ability_effect(
                    ctx,
                    *duration,
                    EffectModification::RemoveKeywordFlag(*keyword),
                );
                Ok(())
            }

            Primitive::LoseAbility(ability_id, duration) => {
                self.register_resolution_ability_effect(
                    ctx,
                    *duration,
                    EffectModification::LoseAbility(*ability_id),
                );
                Ok(())
            }

            Primitive::LoseAllAbilities(duration) => {
                self.register_resolution_ability_effect(
                    ctx,
                    *duration,
                    EffectModification::LoseAllAbilities,
                );
                Ok(())
            }

            Primitive::GrantAbility(def, duration) => {
                let granted_at = self.register_resolution_ability_effect(
                    ctx,
                    *duration,
                    EffectModification::GrantAbility(def.clone()),
                );

                // CR 613.7a clause 2. If the granted ability is itself static,
                // it generates continuous effects of its own, and those take
                // the *later* of this effect's timestamp and the grantee's —
                // not this effect's unconditionally. The grantee usually
                // entered the battlefield first, so the granting effect usually
                // wins, but a permanent that entered after the grant was
                // created keeps its own. `static_effect_timestamp` takes the
                // max; this call only supplies the second candidate.
                if let Some(granted_at) = granted_at {
                    // Re-derived rather than handed back from the call above, so
                    // that the row can own its target `Vec` instead of cloning
                    // it. Safe because registering an effect moves nothing
                    // between zones, so battlefield membership is unchanged.
                    for grantee in self.collect_battlefield_targets(ctx) {
                        self.register_granted_static_effects(
                            def, grantee, granted_at, *duration, ctx.controller,
                        );
                    }
                }
                Ok(())
            }

            // CR 701.7 — create a token.
            //
            // **Not a `GameAction`, and that is deliberate.** §3.1 schedules
            // `CreateTokens { defs: Vec<TokenDef> }` for Phase RE, where the
            // CR 614.16 doublers that replace it live (Doubling Season,
            // Academy Manufactor, Chatterfang). Until one of those exists there
            // is nothing to replace, and a token's *entering* is not a
            // proposal either — CR 614.1c's ETB replacements are Phase RC's,
            // and `place_on_battlefield` is still what a resolving permanent
            // spell uses too.
            //
            // Kalitas's rider is the first customer, and it needs the tokens to
            // exist, not to be replaceable.
            Primitive::CreateToken(token_def, amount_expr) => {
                let count = self.evaluate_amount(amount_expr, ctx)?;
                let controller = self.resolve_player_for_self(recipient, ctx);
                let data = token_card_data(token_def);
                for _ in 0..count {
                    // CR 111.2 — a token's owner is the player who controls the
                    // effect that created it, and CR 111.1's `is_token` is what
                    // makes CR 704.5d and `PermanentFilter::Token` able to see
                    // it. Both are set before it reaches the battlefield,
                    // because `register_static_effects` runs inside
                    // `place_on_battlefield` and would otherwise register
                    // against an object that does not yet know what it is.
                    let mut obj = GameObject::new(data.clone(), controller, Zone::Battlefield);
                    obj.is_token = true;
                    let id = obj.id;
                    self.add_object(obj);
                    self.place_on_battlefield(id, controller);
                    self.events.emit(crate::events::event::GameEvent::PermanentEnteredBattlefield {
                        object_id: id,
                        controller,
                    });
                }
                Ok(())
            }

            // === Regeneration (CR 701.19) ===

            // CR 701.19a — "if the effect of a resolving spell or ability
            // regenerates a permanent, it creates a replacement effect that
            // protects the permanent the next time it would be destroyed this
            // turn."
            //
            // Every part of the shield's shape comes straight from the rule:
            // `Uses::Once` is "the next time", `Duration::UntilEndOfTurn` is
            // "this turn", `Prevent` is "instead", and the `then` rider is the
            // sentence the rule spells out — which is why the engine builds it
            // here rather than asking a card author to spell it out again.
            Primitive::Regenerate => {
                for object in self.collect_battlefield_targets(ctx) {
                    let controller = get_effective_controller(self, object)
                        .unwrap_or(ctx.controller);
                    let def = ReplacementDef::new(
                        EventPattern::Destroy { source: None },
                        AffectedSet::Fixed(vec![object]),
                        Rewrite::Prevent,
                    )
                    .once()
                    .regeneration()
                    .with_then(crate::types::replacement::regeneration_rider());
                    self.replacement_effects.add(RegisteredReplacementEffect {
                        id: 0,
                        source: ctx.source,
                        controller,
                        duration: Duration::UntilEndOfTurn,
                        created_on_turn: self.turn_number,
                        def,
                    });
                }
                Ok(())
            }

            // === "Can't" effects (CR 101.2) ===

            // One registry row per resolved target, and the **resolution
            // supplies the affected set** — a card file cannot name a target it
            // has not yet chosen, which is the same reason
            // `Primitive::Regenerate` above builds its own `AffectedSet::Fixed`.
            // The authored set is therefore required to be an empty `Fixed`, and
            // the `debug_assert` says so rather than silently discarding what an
            // author wrote there.
            //
            // The `Duration` is the card's, never the engine's — CR 608.2c hands
            // scope determination to a human reader, so two cards with identical
            // restriction text can have different scopes because of the sentence
            // before them (`cant-effects-architecture.md` §9 finding 1).
            Primitive::Restrict(def, duration) => {
                for object in self.collect_battlefield_targets(ctx) {
                    let mut def = def.clone();
                    let set = restriction_affected_set_mut(&mut def);
                    debug_assert!(
                        matches!(set, AffectedSet::Fixed(ids) if ids.is_empty()),
                        "a `Primitive::Restrict` on {:?} authored a non-empty                          affected set, which the resolution then overwrote with                          its own targets. Write `AffectedSet::Fixed(Vec::new())`:                          the shape is the card's and the objects are the                          resolution's.",
                        ctx.source
                    );
                    *set = AffectedSet::Fixed(vec![object]);
                    self.restrictions.add(RegisteredRestriction {
                        id: 0,
                        source: ctx.source,
                        controller: ctx.controller,
                        duration: *duration,
                        created_on_turn: self.turn_number,
                        def,
                    });
                }
                Ok(())
            }

            // CR 701.21a — "to sacrifice a permanent, its controller moves it
            // from the battlefield directly to its owner's graveyard".
            //
            // **The choosing player is the *targeted* player, not the spell's
            // controller** — Diabolic Edict's "target player sacrifices a
            // creature" is a choice its target makes. That is what makes this
            // the resolution-time selection path `cant-effects-architecture.md`
            // §4.9 puts its candidate filter in, and the reason Sigarda produces
            // no prompt at all rather than a prompt that is then refused.
            //
            // Not destruction (CR 701.21b): regeneration and indestructible do
            // not apply, which is why the cause is its own `ZoneChangeCause`
            // variant and 278 cards care.
            Primitive::Sacrifice(filter) => {
                for target in &ctx.targets {
                    let ResolvedTarget::Player(player) = target else {
                        continue;
                    };
                    self.sacrifice_one_of_choice(*player, filter, ctx, dp)?;
                }
                Ok(())
            }

            // === Combat and damage housekeeping ===

            Primitive::Tap => {
                // One batch: CR 608.2f processes a spell's actions over several
                // objects simultaneously.
                let batch = self
                    .collect_battlefield_targets(ctx)
                    .into_iter()
                    .map(|object| GameAction::Tap { object })
                    .collect();
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // CR 506.4. Writes `BattlefieldEntity` directly, because 506.4
            // defines a *consequence* with seven causes, and this arm is one of
            // them — the other six follow from a zone change, a control change,
            // a type change, phasing or CR 701.19's regeneration. **The CR does
            // not forbid "can't be removed from combat"**; what it does is put
            // that card's enforcement in six other places as well, which makes
            // it a `RestrictionDef` consulted by each cause
            // (`cant-effects-architecture.md`) and not a replaceable event here.
            // → `replacement-architecture.md` §8a.
            Primitive::RemoveFromCombat => {
                for object in self.collect_battlefield_targets(ctx) {
                    self.remove_from_combat(object);
                }
                Ok(())
            }

            // A direct write because nothing replaces *this* removal — the one
            // printed card that removes all damage, Pyramids, uses it as a
            // replacement's substituted event, never as the replaced one.
            //
            // **Not by analogy to the CR 514.2 cleanup wipe**, which is a
            // different site with a different answer: seven cards restrict
            // *that* one and it owes an enforcement point
            // (`codebase-state.md`, Before Replacement item 20).
            Primitive::RemoveAllDamage => {
                for object in self.collect_battlefield_targets(ctx) {
                    if let Some(entry) = self.battlefield.get_mut(&object) {
                        entry.damage_marked = 0;
                        entry.damaged_by_deathtouch = false;
                    }
                }
                Ok(())
            }

            // === Counters (CR 122) ===
            //
            // Both propose rather than writing `BattlefieldEntity.counters`.
            // A counter mutation is a CR 614-observable event in its own right
            // — CR 614.16's doublers replace it — and CR 122.1c/d's own
            // replacement effects *produce* one, since "instead remove a stun
            // counter from it" is a substituted event and not bookkeeping.

            Primitive::AddCounters(counter_type, amount_expr) => {
                let n = self.evaluate_amount(amount_expr, ctx)? as u32;
                // One batch: CR 608.2f processes a spell's actions over several
                // objects simultaneously, which is what lets a single CR 614.16
                // doubler see all of them.
                let batch = self
                    .collect_battlefield_targets(ctx)
                    .into_iter()
                    .map(|object| GameAction::AddCounters {
                        object,
                        counter: *counter_type,
                        n,
                    })
                    .collect();
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            Primitive::RemoveCounters(counter_type, amount_expr) => {
                let n = self.evaluate_amount(amount_expr, ctx)? as u32;
                let batch = self
                    .collect_battlefield_targets(ctx)
                    .into_iter()
                    .map(|object| GameAction::RemoveCounters {
                        object,
                        counter: *counter_type,
                        n,
                    })
                    .collect();
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // === Layer 2 — control-changing effects (CR 613.1b) ===

            Primitive::GainControl(duration) => {
                let target_ids = self.collect_permanent_or_spell_targets(ctx);
                if target_ids.is_empty() {
                    return Ok(());
                }
                let timestamp = self.allocate_timestamp();
                self.continuous_effects.add(ContinuousEffect {
                    id: 0,
                    source: ctx.source,
                    origin: EffectOrigin::Resolution,
                    layer: Layer::Layer2Control,
                    duration: *duration,
                    controller: ctx.controller,
                    created_on_turn: self.turn_number,
                    timestamp,
                    affected: AffectedSet::Fixed(target_ids),
                    // `You` rather than `ctx.controller`, though they name the
                    // same player: on a `Resolution` row, `FilterPlayers::you()`
                    // reads `ContinuousEffect.controller`, which CR 611.2c
                    // locked at resolution.
                    modification: EffectModification::SetController(PlayerRef::You),
                });
                Ok(())
            }

            // === Phase 3+ primitives — stubs ===

            Primitive::Exile
            | Primitive::ReturnToHand
            | Primitive::ReturnToBattlefield
            | Primitive::PutOnTopOfLibrary
            | Primitive::PutOnBottomOfLibrary
            | Primitive::ShuffleIntoLibrary
            | Primitive::Mill(_)
            | Primitive::Discard(_)
            | Primitive::Scry(_)
            | Primitive::Surveil(_)
            | Primitive::Fight => {
                Err(format!("Primitive {:?} not yet implemented", primitive))
            }
        }
    }

    // --- Helpers: Layer 6 (CR 613.1f) ---

    /// Record the Layer 6 continuous effect that a resolving spell or ability
    /// creates (CR 613.7b) — the resolution-time counterpart of
    /// `GameState::register_static_effects`, which does the same job at ETB for
    /// printed static abilities.
    ///
    /// **This grants nothing.** It appends one row to the registry; the
    /// characteristics it implies are recomputed from that row on every
    /// `compute_characteristics` call, and the affected objects' `CardData` is
    /// never touched. Duration expiry removes the row, and there is nothing to
    /// undo because nothing was ever written.
    ///
    /// The affected set is frozen to the targets that are still on the
    /// battlefield (CR 613.7b): a creature that died in response is dropped, and
    /// the survivors stay affected even if they later stop matching whatever the
    /// card described.
    ///
    /// Returns the timestamp allocated for the row, so a caller that must
    /// register further rows against the same moment can. `None` means no target
    /// survived, in which case no row is written and no timestamp is burned.
    fn register_resolution_ability_effect(
        &mut self,
        ctx: &ResolutionContext,
        duration: Duration,
        modification: EffectModification,
    ) -> Option<Timestamp> {
        let targets = self.collect_battlefield_targets(ctx);
        if targets.is_empty() {
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
            affected: AffectedSet::Fixed(targets),
            modification,
        });
        Some(timestamp)
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
        granting_timestamp: Timestamp,
        duration: Duration,
        controller: PlayerId,
    ) {
        use crate::objects::card_data::AbilityType;

        // Not a guard against misuse — the expected path. Every `GrantAbility`
        // calls this, and most granted abilities are triggered, activated or
        // mana abilities ("target creature gains '{T}: add {G}'"). Only a static
        // ability generates a continuous effect, so everything else correctly
        // registers nothing and still lands on the object via the Layer 6 row.
        // `test_granting_a_non_static_ability_registers_no_derived_effect`.
        if granted.ability_type != AbilityType::Static {
            return;
        }

        // Deliberately does NOT check `is_characteristic_defining`. CR 604.3a(2)
        // says a granted ability is never a CDA, and `apply_modification`
        // enforces that by clearing the flag as the ability lands — so the
        // object holds an *ordinary* static ability. Overruling the card author
        // means treating it as ordinary, not dropping it: an early return here
        // left the creature holding a static ability that generated nothing.

        // Same two helpers `register_static_effects` uses, and for the reason
        // `static_primitive_rows`' doc gives: a granted "creatures you control
        // get +1/+1" must produce the row a printed one does, or identical card
        // text behaves differently depending on how it arrived. That includes
        // the assertions — a grant that lowers to nothing is exactly as inert
        // as a printed ability that does, and exactly as invisible.
        let context = format!("granted ability {:?}", granted.id);
        let atoms = GameState::static_ability_atoms(granted, &context);

        for (primitive, recipient) in atoms {
            let Some(affected) = GameState::static_affected_set(recipient, &context) else {
                continue;
            };

            let rows = GameState::static_primitive_rows(primitive);
            if rows.is_empty() {
                debug_assert!(
                    false,
                    "{} lowers to no layer rows; `static_primitive_rows` has no \
                     arm for {:?}, so the grant registers nothing.",
                    context, primitive
                );
                continue;
            }
            let timestamp =
                self.static_effect_timestamp(grantee, granted, Some(granting_timestamp));

            for (layer, modification) in rows {
                // A granted static ability whose own effect lands in layers
                // 1-6 does not apply, and it fails silently: the grant applies
                // AT layer 6, so at any layer <= 6 the frame the CR 613.7a
                // existence check reads (`compute_to_ceiling` at `layer_index`)
                // predates the grant, and the derived effect finds no ability
                // to justify itself. Assert at the authoring site rather than
                // let a card quietly do nothing.
                //
                // The two cases below the assert are not the same problem.
                //
                // **Layer 6 exactly** is real and is CR 613.7a's own worked
                // example: Rune of Flight grants enchanted Equipment "Equipped
                // creature has flying". The CR resolves it purely by timestamp
                // within layer 6, and our ordering is already right for it --
                // clause 2 makes the derived timestamp `max(grantee, grant)`,
                // which is >= the grant's, and when they tie the grant row is
                // added first so it takes the lower `EffectId` tiebreak. So the
                // grant always sorts at-or-before its own derived effect. The
                // only missing piece is that the existence check cannot see a
                // *partially applied* layer.
                //
                // That is exactly what `codebase-state.md` item 8 step 4 builds:
                // apply a layer board-wide in one sequential pass over ordered
                // applications, so the check at position k sees everything
                // applied earlier in the same layer. It is the same fix 613.8b's
                // loop rule needs, which is why the two are scheduled together.
                // Not a workaround waiting for a rewrite -- the ordering work is
                // done, only the frame the check reads has to change.
                //
                // **Layers 1-5** is a different animal, and we are not waiting
                // on it. CR 613.8a(a) confines dependency to a single layer, so
                // the CR supplies no mechanism for a layer 6 grant to reach back
                // into layer 5, and any answer would be invented. Searched
                // Scryfall for granted statics that define a type, color or
                // subtype: the hits are all false positives (quoted text inside
                // activated abilities, and Animate Dead's enchant clause). Real
                // grants are of triggered abilities, activated abilities,
                // keywords, or layer 7 statics. Nothing to build against.
                debug_assert!(
                    layer > Layer::Layer6Ability,
                    concat!(
                        "granted static ability generates a {:?} effect. A grant ",
                        "applies at layer 6, so the CR 613.7a existence check ",
                        "reads a pre-grant frame at any layer <= 6 and this ",
                        "effect will not apply. Layer 6 itself needs the ",
                        "board-wide sequential pass (codebase-state.md item 8 ",
                        "step 4); layers 1-5 have no CR mechanism and no known card."
                    ),
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
    /// One player sacrifices one permanent of their choice (CR 701.21a).
    ///
    /// **CR 608.2d and `cant-effects-architecture.md` §4.9 are one mechanism
    /// here, not two.** 608.2d's own example is "a player who controls no
    /// creatures can't choose the sacrifice option"; Sigarda's ruling is "if it
    /// would force you to sacrifice a permanent, you just don't" — and both are
    /// answered by the same candidate list being empty. Prompting and then
    /// refusing would violate 608.2d, would tell every other player which
    /// permanent you would have picked, and would make an AI harness spend a
    /// decision on a branch that cannot happen.
    ///
    /// An empty list is CR 101.3's "any part of an instruction that's impossible
    /// to perform is ignored" — no prompt, no sacrifice, no error. The
    /// *fallback* half ("each player who can't discards a card") waits on
    /// `Effect::Conditional`, and that split is safe in one direction only:
    /// suppressing a prompt with no fallback is a resolved effect that does
    /// nothing, which is 101.3's own answer.
    fn sacrifice_one_of_choice(
        &mut self,
        player: PlayerId,
        filter: &SelectionFilter,
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        let candidates: Vec<ResolvedTarget> =
            crate::oracle::legality::enumerate_legal_selections(self, filter, None, player)
                .into_iter()
                .filter(|t| match t {
                    // "Its controller moves it": only your own permanents.
                    ResolvedTarget::Object(id) => {
                        get_effective_controller(self, *id) == Some(player)
                            && !self.sacrifice_is_prohibited(*id, ctx.controller)
                    }
                    ResolvedTarget::Player(_) => false,
                })
                .collect();

        if candidates.is_empty() {
            return Ok(());
        }

        let recipient = EffectRecipient::Choose(filter.clone(), TargetCount::Exactly(1));
        let chosen = crate::ui::ask::ask_select_recipients(
            dp, self, player, &recipient, ctx.source, &candidates, 1, 1,
        );
        let Some(ResolvedTarget::Object(id)) = chosen.first().copied() else {
            return Ok(());
        };
        let actx = ActionContext::resolving(dp, ctx);
        self.change_zone(id, Zone::Graveyard, ZoneChangeCause::Sacrificed, &actx)
    }

    /// Would sacrificing this permanent be prohibited (CR 101.2)?
    ///
    /// The candidate-filter half of §4.9, asked of the event the choice would
    /// produce rather than of the choice — which is what keeps this an axis-1
    /// question and leaves the axis-2 choice sites to RS-2.
    fn sacrifice_is_prohibited(&self, id: ObjectId, cause: PlayerId) -> bool {
        let action = GameAction::ZoneChange {
            object: id,
            from: Zone::Battlefield,
            to: Zone::Graveyard,
            cause: ZoneChangeCause::Sacrificed,
        };
        crate::engine::restriction::is_prohibited(
            self,
            &crate::engine::restriction::Query::Event {
                action: &action,
                cause: Some(cause),
            },
        )
    }

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

    /// Resolved targets that are on the battlefield **or** the stack — CR
    /// 108.4's "a card doesn't have a controller unless that card represents a
    /// permanent or spell".
    ///
    /// The wider sibling of `collect_battlefield_targets`, and only Layer 2
    /// wants it: every other continuous effect describes a characteristic a
    /// permanent has, while control is the one thing a spell also has.
    fn collect_permanent_or_spell_targets(&self, ctx: &ResolutionContext) -> Vec<ObjectId> {
        ctx.targets
            .iter()
            .filter_map(|t| match t {
                ResolvedTarget::Object(id)
                    if self.battlefield.contains_key(id)
                        || self.stack_entries.contains_key(id) =>
                {
                    Some(*id)
                }
                _ => None,
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
        // CR 109.5: "you" on the Aura's enchant clause is the Aura's
        // controller, which is what makes "Enchant creature you control" mean
        // something different from "Enchant creature".
        if !self.has_any_legal_choice(&filter, Some(aura_id), controller) {
            // No legal host exists. Aura stays unattached; 704.5m SBA
            // will put it into the graveyard.
            return Ok(false);
        }

        let recipient = EffectRecipient::Choose(filter.clone(), TargetCount::Exactly(1));
        let legal = crate::oracle::legality::enumerate_legal_selections(
            self, &filter, Some(aura_id), controller,
        );
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

/// The affected set inside a [`RestrictionDef`], whichever arm it is.
///
/// One accessor rather than a match at each caller: every `Restriction` arm
/// names the objects it applies to, and `Primitive::Restrict` overwrites that
/// set with the resolution's targets without caring which arm it is. Matched
/// exhaustively, so a new arm has to say where its objects live rather than
/// silently keeping whatever the card wrote.
fn restriction_affected_set_mut(def: &mut RestrictionDef) -> &mut AffectedSet {
    match &mut def.what {
        Restriction::Event { affected, .. } => affected,
        Restriction::ApplyReplacement { to, .. } => to,
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

/// Lower a [`TokenDef`](crate::types::effects::TokenDef) into the `CardData`
/// its `GameObject` reads.
///
/// CR 111.4: "a token has the characteristics of the spell or ability that
/// created it" — and no mana cost (CR 111.6 makes its mana value 0), which
/// falls out of `CardDataBuilder`'s default rather than being set.
///
/// One `Arc` per `CreateToken` resolution, shared by every token that
/// resolution makes. They are separate objects with separate `ObjectId`s; what
/// they share is their printed characteristics, which is exactly what
/// `Arc<CardData>` means everywhere else in this engine.
fn token_card_data(
    def: &crate::types::effects::TokenDef,
) -> std::sync::Arc<crate::objects::card_data::CardData> {
    let mut builder = crate::objects::card_data::CardDataBuilder::new(&def.name)
        .power_toughness(def.power, def.toughness);
    for color in &def.colors {
        builder = builder.color(*color);
    }
    for card_type in &def.types {
        builder = builder.card_type(*card_type);
    }
    for subtype in &def.subtypes {
        builder = builder.subtype(subtype.clone());
    }
    for keyword in &def.keyword_flags {
        builder = builder.keyword(*keyword);
    }
    builder.build()
}
