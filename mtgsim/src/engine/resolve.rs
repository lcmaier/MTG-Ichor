use crate::engine::actions::{
    ActionContext, DestructionSource, DrawCause, GameAction, LifeLossCause, ZoneChangeCause,
};
use crate::engine::layers::types::{
    ObjectSet, ContinuousEffect, EffectId, EffectModification, EffectOrigin, Layer, Timestamp,
};
use crate::events::event::{CounterSubject, DamageTarget, LossReason};
use crate::engine::targeting::{instance_of, ChosenTargets, DeclaredInstances, TargetInstance};
use crate::objects::card_data::AbilityDef;
use crate::types::zones::Zone;
use crate::state::game_state::{GameState, PlannedPhase};
use crate::types::effects::{
    AmountExpr, CopyRoles, DiscardChooser, Duration, Effect, EffectRecipient, PatternFill,
    PlayerRef, Primitive,
    PlayerSet, SelectionFilter, TargetCount,
};
use crate::oracle::characteristics::{controls, get_effective_controller};
use crate::state::replacement_effects::RegisteredReplacementEffect;
use crate::state::restrictions::RegisteredRestriction;
use crate::types::restriction::{Restriction, RestrictionDef};
use crate::types::ids::{ObjectId, ObjectRef, PlayerId};
use crate::types::replacement::{EventPattern, ReplacementDef, Rewrite};
use crate::ui::decision::DecisionProvider;

/// Context passed through effect resolution.
///
/// Tracks the source of the spell/ability, its controller, and resolved
/// targets so that each `Primitive` knows what it's acting on.
#[derive(Debug, Clone)]
pub struct ResolutionContext {
    /// The resolving stack object — the spell, or for an ability the
    /// ephemeral object CR 608.2n deletes at the end of resolution.
    pub source: ObjectId,
    /// CR 113.7a — for an activated or triggered ability, the permanent whose
    /// ability is resolving, and which existence of it (CR 400.7); `None` for
    /// a spell (whose source is `source`) and for a CR 615.5 rider. "This
    /// permanent" in an ability's text reads this: `Primitive::Attach` attaches
    /// it and `EffectRecipient::ThisObject` finds it. Existing primitives that
    /// attribute to `source` (`DealDamage`'s source, `Destroy`'s
    /// `DestructionSource`) are unchanged by this field.
    pub ability_source: Option<ObjectRef>,
    /// The player who controls the spell/ability
    pub controller: PlayerId,
    /// What each instance of "target" holds **after** CR 608.2b's re-check —
    /// the survivors, not the announcement. An instance every one of whose
    /// targets went illegal is empty here, and the atom that reads it does
    /// nothing, which is 608.2b's "it won't affect that target".
    pub targets: ChosenTargets,
    /// CR 615.5's "that much" — the amount the replaced event carried, for a
    /// rider and for nothing else.
    ///
    /// `None` on every other resolution, which is what makes
    /// `AmountExpr::ReplacedAmount` refuse rather than read a wrong number: a
    /// resolving spell has no replaced event, so the question has no answer
    /// rather than a default one.
    pub replaced_amount: Option<u64>,
    /// CR 615.5's "the amount of damage that was prevented" — for a rider and
    /// for nothing else, read by `AmountExpr::DamagePrevented`.
    ///
    /// `Some(0)` for a rider whose effect prevented nothing — a plain
    /// replacement's, or a prevention that met CR 615.12's unpreventable damage
    /// — because that is the number Reverse Damage's "you gain life
    /// equal to the damage prevented this way" needs there. `None` outside a
    /// rider, for `replaced_amount`'s reason.
    pub damage_prevented: Option<u64>,
    /// A triggered ability's bound facts, for the resolution of one and for
    /// nothing else (`triggers-architecture.md` §6.3): `TriggeringObject`,
    /// `TriggeringPlayer` and `TriggeringAmount` read it, and refuse outside
    /// a trigger rather than reading a default. One field, not a third
    /// `Option` beside the rider pair (§15 item 8).
    pub trigger: Option<crate::types::triggers::TriggerBinding>,
}

impl ResolutionContext {
    /// A resolution that names nothing — no targets: ChosenTargets::one(targets), no rider, and no
    /// permanent distinct from `source`. What a mana ability's resolution uses.
    ///
    /// The two CR 615.5 numbers are one optional thing wearing two `Option`s;
    /// the type-side fix is a single `rider: Option<RiderAmounts>`, a 48-site
    /// sweep and its own PR (`codebase-state.md` "Found by RE-9", item 137).
    pub fn untargeted(source: ObjectId, controller: PlayerId) -> Self {
        ResolutionContext {
            source,
            ability_source: None,
            controller,
            targets: ChosenTargets::NONE,
            replaced_amount: None,
            damage_prevented: None,
            trigger: None,
        }
    }
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
    /// A combinator with no arm (`Conditional`, `Optional`, `Modal`, `ForEach`,
    /// `Repeat`) returns `Err` naming it rather than silently doing nothing.
    pub fn resolve_effect(
        &mut self,
        effect: &Effect,
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        // Nothing was announced for a bare effect — CR 615.5's rider, or a
        // test that staged `ctx` by hand — so a `SameInstanceAs` atom reads its
        // clause off the tree (`DeclaredInstances::Effect`). The stack's path
        // is [`Self::resolve_effect_with_announced_targets`].
        let mut cursor = 0usize;
        self.resolve_effect_at(effect, ctx, dp, DeclaredInstances::Effect(effect), &mut cursor)
    }

    /// [`Self::resolve_effect`] for a spell or ability leaving the stack.
    ///
    /// `announced` is the entry's CR 601.2c record, so an atom finds its own
    /// targets by the index the announcement filled and nothing is derived
    /// from the tree — see `DeclaredInstances`.
    pub fn resolve_effect_with_announced_targets(
        &mut self,
        effect: &Effect,
        announced: &[TargetInstance],
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        let mut cursor = 0usize;
        self.resolve_effect_at(effect, ctx, dp, DeclaredInstances::Announced(announced), &mut cursor)
    }

    /// [`Self::resolve_effect`]'s body, carrying CR 601.2c's instance cursor.
    ///
    /// **The one place `ctx.targets` is indexed.** Each atom is handed its own
    /// instance as a flat slice, so no primitive can reach a neighboring
    /// instance's targets, and an atom whose instance fizzled is handed an
    /// empty one rather than the spell's first (CR 608.2b).
    fn resolve_effect_at(
        &mut self,
        effect: &Effect,
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
        declared: DeclaredInstances<'_>,
        cursor: &mut usize,
    ) -> Result<(), String> {
        match effect {
            // "That creature", "that player": not an instance of "target" but
            // a fact bound at dispatch, handed to the primitive as the target
            // slice it already reads — empty when CR 603.6 finds nothing.
            Effect::Atom(
                primitive,
                recipient @ (EffectRecipient::TriggeringObject | EffectRecipient::TriggeringPlayer),
            ) => {
                let bound = self.bound_targets(recipient, ctx)?;
                self.resolve_primitive(primitive, recipient, &bound, ctx, dp)
            }
            // "This creature": no instance of "target" either (CR 113.7a), and
            // empty when the source is no longer the object the ability is of.
            Effect::Atom(primitive, recipient @ EffectRecipient::ThisObject) => {
                let this: Vec<ResolvedTarget> =
                    self.this_object(ctx).map(ResolvedTarget::Object).into_iter().collect();
                self.resolve_primitive(primitive, recipient, &this, ctx, dp)
            }
            Effect::Atom(primitive, recipient) => {
                match instance_of(recipient, declared, cursor) {
                    Some((ix, clause)) => {
                        self.resolve_primitive(primitive, clause, ctx.targets.instance(ix), ctx, dp)
                    }
                    // `Instance(ix)` naming a clause that does not exist: a
                    // card-authoring error, loud rather than silently
                    // targetless. `cards::registry`'s own test refuses one at
                    // registration, so this is the belt to that's braces.
                    None if matches!(recipient, EffectRecipient::SameInstanceAs(_)) => Err(format!(
                        "{:?} on {:?} names an instance of \"target\" the effect never declared (CR 115.3)",
                        recipient, ctx.source
                    )),
                    // Implicit, Controller, the filtered sweeps, Host — none
                    // reads a chosen target.
                    None => self.resolve_primitive(primitive, recipient, &[], ctx, dp),
                }
            }

            Effect::Sequence(effects) => {
                for sub in effects {
                    self.resolve_effect_at(sub, ctx, dp, declared, cursor)?;
                }
                Ok(())
            }

            // CR 614.1a. A *static* replacement ability never reaches here —
            // `engine::replacement::gather` reads it off the effective list — so this
            // arm is a replacement effect created by a *resolution*, which CR 614.3
            // gives a duration. Loud, because the duration is the missing piece:
            // `Primitive::CreateReplacement` takes it as an argument, and CR 701.19a's
            // shield comes through `Primitive::Regenerate`, which knows its own.
            Effect::Replacement(_) => Err(
                "a replacement effect created by a resolution needs a CR 614.3 \
                 duration, which `Effect::Replacement` does not carry. Use \
                 `Primitive::CreateReplacement`, which takes a `Duration` \
                 argument. A static ability's replacement effect does not \
                 resolve at all — put it on an `AbilityType::Static` ability and \
                 `engine::replacement::gather` will find it. For CR 701.19a's \
                 regeneration shield, use `Primitive::Regenerate`."
                    .to_string(),
            ),

            // CR 101.2's twin of the arm above, loud for the same reason: a static
            // "can't" is read off the effective list by
            // `engine::restriction::is_prohibited`, and one a *resolution* creates
            // needs a scope CR 608.2c hands to a human reader
            // (`cant-effects-architecture.md` §9 finding 1).
            Effect::Restriction(_) => Err(
                "a \"can't\" effect created by a resolution needs a duration, \
                 which `Effect::Restriction` does not carry. CR 608.2c makes the \
                 scope unrecoverable from the restriction's own text, so it has \
                 to be authored: use `Primitive::Restrict`, which takes a \
                 `Duration` argument. A static ability's restriction does not \
                 resolve at all — put it on an `AbilityType::Static` ability and \
                 `engine::restriction::is_prohibited` will find it."
                    .to_string(),
            ),

            // CR 601.2f's twin: a static cost effect is read off the effective list
            // when a cost is determined and never resolves; one a *resolution*
            // creates needs a CR 611.2a duration this carries none of
            // (`cost-architecture.md` §3.10).
            Effect::CostModification(_) => Err(
                "a cost modification created by a resolution needs a CR 611.2a \
                 duration, which `Effect::CostModification` does not carry. A \
                 static ability's cost effect does not resolve at all — put it on \
                 an `AbilityType::Static` ability and \
                 `engine::cost_determination::cost_modifications_for` will find it at CR 601.2f."
                    .to_string(),
            ),

            // "If [condition], [effect]" inside a resolving effect — CR 603.4's
            // intervening "if" is the def's own field and is checked in
            // `resolve_taken`; an `if` anywhere else is this, read against the
            // board as the atom is reached (CR 608.2c's "in the order written").
            Effect::Conditional(condition, inner) => {
                let source = ctx.ability_source.map_or(ctx.source, |r| r.id);
                if crate::engine::layers::condition::settled_holds(condition, self, source) {
                    self.resolve_effect_at(inner, ctx, dp, declared, cursor)
                } else {
                    Ok(())
                }
            }

            // A triggered ability is never resolved as written: the
            // dispatcher reads it off the effective list, and what reaches
            // the stack is its inner effect with the binding beside it. A
            // spell cannot carry one (CR 603.1 — a triggered ability is an
            // ability of an object), so reaching here is a wiring error.
            Effect::Triggered(_) => Err(format!(
                "a triggered ability reached resolution as an effect of {:?}; `Effect::Triggered` belongs on an `AbilityType::Triggered` ability and is read by `engine::triggers`",
                ctx.source
            )),

            Effect::Optional(_inner) => {
                // A yes/no ask — `codebase-state.md` main item 24.
                Err("Optional effects not yet implemented".to_string())
            }

            Effect::Modal { .. } => {
                // Mode choice — `backlog.md` §2.7.
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
    ///
    /// `targets` is **this atom's instance** of "target" (CR 115.3), already
    /// filtered by CR 608.2b — not the spell's whole list. `recipient` is the
    /// clause that instance was announced with, which for an
    /// `EffectRecipient::SameInstanceAs` atom is the *declaring* atom's clause, so
    /// the two atoms of Ensoul Artifact behave identically.
    fn resolve_primitive(
        &mut self,
        primitive: &Primitive,
        recipient: &EffectRecipient,
        targets: &[ResolvedTarget],
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        // Every mutation a primitive proposes belongs to *this* resolution
        // (CR 614.15 / the resolution stamp on each emitted event).
        let actx = ActionContext::resolving(dp, ctx);
        match primitive {
            // === One-shot primitives ===

            // **One batch, however many things it hits.** Three rules read the batch
            // rather than the events in it — CR 704.3's simultaneity, CR 615.7's "two
            // or more applicable sources" and CR 603.2c's "one or more" — and a loop
            // of `execute_action` is unreachable from all three (`CLAUDE.md`).
            // `FilteredPermanents` is resolved here rather than filled into
            // `ctx.targets`: the recipient means "every permanent matching this
            // **now**". Ordered, because the members reach a CR 616.1 prompt and a log.
            Primitive::DealDamage { amount: amount_expr, unpreventable } => {
                let amount = self.evaluate_amount(amount_expr, ctx)?;
                let targets: Vec<DamageTarget> = match recipient {
                    // CR 119.3 — damage is dealt to permanents and players, so
                    // a zone-reaching recipient has nothing to be dealt to.
                    EffectRecipient::FilteredObjectsIn(..) => {
                        return Err(format!(
                            "a `Primitive::DealDamage` on {:?} has a zone-reaching recipient (CR 119.3)",
                            ctx.source
                        ));
                    }
                    EffectRecipient::FilteredPermanents(filter) => self
                        .battlefield_ids_ordered()
                        .into_iter()
                        .filter(|&id| {
                            self.object_matches_filter(id, filter, ctx.controller)
                                .unwrap_or(false)
                        })
                        .map(DamageTarget::Object)
                        .collect(),
                    _ => targets
                        .iter()
                        .map(|t| match t {
                            ResolvedTarget::Object(id) => DamageTarget::Object(*id),
                            ResolvedTarget::Player(pid) => DamageTarget::Player(*pid),
                        })
                        .collect(),
                };
                if targets.is_empty() {
                    return Ok(());
                }
                self.execute_actions(
                    targets
                        .into_iter()
                        // CR 615.12's per-event flag, set by the effect that proposes the damage
                        // and carried onto every member: Pinpoint Avalanche's "can't be
                        // prevented" is about this resolution's damage and nothing else.
                        .map(|target| GameAction::DealDamage {
                            source: ctx.source,
                            target,
                            amount,
                            is_combat: false,
                            unpreventable: *unpreventable,
                        })
                        .collect(),
                    &actx,
                )?;
                Ok(())
            }

            Primitive::DrawCards(amount_expr) => {
                let count = self.evaluate_amount(amount_expr, ctx)?;
                // Drawing targets the controller (EffectRecipient::Controller or None)
                let player_id = self.resolve_player_for_self(recipient, targets, ctx);
                // **One instruction, whatever `count` is** (CR 121.2a); its performer
                // does CR 121.2's individual draws. A loop here would make "draw three
                // cards" three instructions, the distinction Alms Collector's ruling turns
                // on ("count how many times the word 'draw' is used").
                self.execute_action(
                    GameAction::DrawCards {
                        player: player_id,
                        n: count,
                        cause: DrawCause::Effect,
                    },
                    &actx,
                )?;
                Ok(())
            }

            // > 701.17a For a player to mill a number of cards, that player
            // > puts that many cards from the top of their library into their
            // > graveyard.
            //
            // **One batch, N members** — "puts that many cards" is one simultaneous
            // move, with no CR 121.2 "one at a time" making it the exception, so a
            // CR 603.2c "one or more cards" trigger fires once for a mill of five.
            // Each member is still its own event for CR 614.5, so Leyline of the Void
            // applies to every card (§4.2). CR 701.17b makes a short library "as many
            // as possible" rather than a failure, and the cards are taken before any
            // moves because they all move at once.
            Primitive::Mill(amount_expr) => {
                let count = self.evaluate_amount(amount_expr, ctx)? as usize;
                let player_id = self.resolve_player_for_self(recipient, targets, ctx);
                let library = &self.get_player(player_id)?.library;
                let batch: Vec<GameAction> = library
                    .iter()
                    .rev()
                    .take(count)
                    .map(|&object| GameAction::ZoneChange {
                        object,
                        from: Zone::Library,
                        to: Zone::Graveyard,
                        cause: ZoneChangeCause::Milled,
                    })
                    .collect();
                if batch.is_empty() {
                    return Ok(());
                }
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // > 701.9a To discard a card, move it from its owner's hand to
            // > that player's graveyard.
            //
            // **One batch, N members**, on `Primitive::Mill`'s argument: the cards
            // are chosen and then move together, so CR 603.2c's "one or more cards"
            // fires once for a Mind Rot, while each card is its own member and subject
            // — Library of Leng's ruling is one CR 616.1 decision per member, in the
            // batch's order. The chooser is CR 701.9b's: [`DiscardChooser::Affected`]
            // asks the player, `AtRandom` draws from the game's own `rng`.
            Primitive::Discard(amount_expr, chooser) => {
                let count = self.evaluate_amount(amount_expr, ctx)? as usize;
                let player_id = self.resolve_player_for_self(recipient, targets, ctx);
                let hand = self.get_player(player_id)?.hand.clone();
                let chosen = match chooser {
                    DiscardChooser::Affected => {
                        crate::ui::ask::ask_discard(
                            actx.dp, self, player_id, &hand, count, Some(ctx.source),
                        )
                    }
                    DiscardChooser::AtRandom => self.random_cards_from(&hand, count),
                };
                if chosen.is_empty() {
                    return Ok(());
                }
                let batch: Vec<GameAction> = chosen
                    .into_iter()
                    .map(|object| GameAction::ZoneChange {
                        object,
                        from: Zone::Hand,
                        to: Zone::Graveyard,
                        cause: ZoneChangeCause::Discarded,
                    })
                    .collect();
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // CR 701.22 — the instruction, and the performer does the rest.
            Primitive::Scry(amount_expr) => {
                let n = self.evaluate_amount(amount_expr, ctx)?;
                let player_id = self.resolve_player_for_self(recipient, targets, ctx);
                self.execute_action(GameAction::Scry { player: player_id, n }, &actx)?;
                Ok(())
            }

            Primitive::GainLife(amount_expr) => {
                let amount = self.evaluate_amount(amount_expr, ctx)?;
                let player_id = self.resolve_player_for_self(recipient, targets, ctx);
                self.execute_action(GameAction::GainLife {
                    player: player_id,
                    amount,
                    source: ctx.source,
                }, &actx)?;
                Ok(())
            }

            Primitive::LoseLife(amount_expr) => {
                let amount = self.evaluate_amount(amount_expr, ctx)?;
                let player_id = self.resolve_player_for_self(recipient, targets, ctx);
                self.execute_action(GameAction::LoseLife {
                    player: player_id,
                    amount,
                    cause: LifeLossCause::Effect,
                }, &actx)?;
                Ok(())
            }

            // CR 119.5 — "gains or loses the necessary amount of life". A proposal of
            // whichever one it is, never a write to the total: Exquisite Archangel's
            // "-4 becomes 20 is a 24-life gain" is what a Rhox Faithmender doubles and
            // a Skullcrack refuses. Equal totals propose nothing; a 0 gain is a
            // non-event (CR 119.10) and a 0 loss a no-op.
            Primitive::SetLifeTotal(amount_expr) => {
                let target = self.evaluate_amount(amount_expr, ctx)? as i64;
                let player = self.resolve_player_for_self(recipient, targets, ctx);
                let current = self.get_player(player)?.life_total;
                if target > current {
                    self.execute_action(GameAction::GainLife {
                        player,
                        amount: (target - current) as u64,
                        source: ctx.source,
                    }, &actx)?;
                } else if target < current {
                    self.execute_action(GameAction::LoseLife {
                        player,
                        amount: (current - target) as u64,
                        cause: LifeLossCause::Effect,
                    }, &actx)?;
                }
                Ok(())
            }

            // CR 104.3e / 104.2b — the game's end as an effect. Proposed, so
            // "you can't lose" refuses one and "if you would lose" replaces
            // one, exactly as for a state-based loss; a player who has
            // already left is gated here, as the SBA check gates them.
            Primitive::LoseGame => {
                let player = self.resolve_player_for_self(recipient, targets, ctx);
                if self.in_game(player) {
                    self.execute_action(
                        GameAction::PlayerLoses { player, reason: LossReason::Effect },
                        &actx,
                    )?;
                }
                Ok(())
            }
            Primitive::WinGame => {
                let player = self.resolve_player_for_self(recipient, targets, ctx);
                if self.in_game(player) {
                    self.execute_action(GameAction::PlayerWins { player }, &actx)?;
                }
                Ok(())
            }

            // CR 701.13a — move to exile from wherever the object is: the resolved
            // targets (CR 608.2b has re-checked them), or `ThisObject`'s slice —
            // "exile this creature" on a rider (Exquisite Archangel) and "Exile
            // Stunning Reversal" as a spell's last instruction, which CR 608.2m lets
            // finish resolving from exile. One batch, for `Destroy`'s reason (CR 608.2f).
            Primitive::Exile => {
                let objects: Vec<ObjectId> = match recipient {
                    EffectRecipient::Implicit => {
                        return Err(format!(
                            "a `Primitive::Exile` on {:?} names nothing to exile; use `ThisObject` or a target",
                            ctx.source
                        ));
                    }
                    _ => targets
                        .iter()
                        .filter_map(|t| match t {
                            ResolvedTarget::Object(id) if self.objects.contains_key(id) => Some(*id),
                            _ => None,
                        })
                        .collect(),
                };
                let mut batch = Vec::with_capacity(objects.len());
                for object in objects {
                    let from = self.get_object(object)?.zone;
                    batch.push(GameAction::ZoneChange {
                        object,
                        from,
                        to: Zone::Exile,
                        cause: ZoneChangeCause::Exiled,
                    });
                }
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // CR 500.7 — scheduling, not a mutation: the extra turn becomes a
            // `GameAction::BeginTurn` proposal when the drainer reaches it, which is
            // what a skip replaces (CR 614.10a). Pushed, because "the most recently
            // created turn will be taken first".
            Primitive::ExtraTurn => {
                let player = self.resolve_player_for_self(recipient, targets, ctx);
                self.turn_queue.push(player);
                Ok(())
            }

            // CR 500.8 — scheduling, like the extra turn: the phases become
            // `GameAction::BeginPhase` proposals when the drainer reaches them,
            // inserted at `cursor + 1` for "directly after the specified phase". A
            // resolution outside a phase splices nothing: the cursor is `None` only
            // between a turn beginning and its first phase, when nothing can be
            // resolving, and a `Vec` insert past its end would panic.
            Primitive::ExtraPhases(phases) => {
                let Some(cursor) = self.turn_plan.cursor else {
                    return Ok(());
                };
                let at = cursor + 1;
                self.turn_plan.phases.splice(
                    at..at,
                    phases.iter().map(|&phase_type| PlannedPhase { phase_type }),
                );
                Ok(())
            }

            // CR 106.6a's event from a spell — Dark Ritual. `tapped_for_mana`
            // is `false` by CR 106.12's definition and CR 605.5b ("a spell can
            // never be a mana ability"), which is why Mana Reflection leaves
            // it at three: its first ruling, and the definition says it first.
            Primitive::ProduceMana(output) => {
                let mana: Vec<_> = output.mana.iter()
                    .map(|(mt, expr)| Ok((*mt, self.evaluate_amount(expr, ctx)?)))
                    .collect::<Result<_, String>>()?;
                self.execute_action(
                    GameAction::ProduceMana {
                        player: ctx.controller,
                        source: ctx.source,
                        mana,
                        special: output.special.clone(),
                        tapped_for_mana: false,
                    },
                    &actx,
                )
            }

            Primitive::CounterSpell => {
                // CR 701.6a — the countered spell goes to its owner's graveyard, through
                // `execute_action(ZoneChange)` so the CR 614 pipeline sees it;
                // `remove_from_zone_collection(Stack)` tears down the `StackEntry`.
                for target in targets {
                    if let ResolvedTarget::Object(id) = target {
                        let id = *id;
                        if self.stack.contains(&id) {
                            self.change_zone(id, crate::types::zones::Zone::Graveyard, ZoneChangeCause::Countered, &actx)?;
                            self.emit_event(crate::events::event::GameEvent::SpellCountered {
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
                for target in targets {
                    if let ResolvedTarget::Object(id) = target
                        && let Some(pos) = self.stack.iter().position(|s| s == id) {
                        let removed_id = self.stack.remove(pos);
                        self.take_stack_entry(removed_id);
                        // Remove the object entirely — abilities on the
                        // stack are not cards and have no destination zone.
                        self.remove_object(removed_id);
                        self.emit_event(crate::events::event::GameEvent::AbilityCountered {
                            ability_id: removed_id,
                            countered_by: ctx.source,
                        });
                    }
                }
                Ok(())
            }

            // === Destroy and untap ===

            Primitive::Destroy => {
                // CR 701.8a — destroy target permanent.
                //
                // **One batch, because CR 608.2f says so**: a board wipe destroys
                // everything at one instant, so the deaths are one event, which is what
                // lets a CR 614 replacement apply once *per death* and what CR 615.7's
                // shield allocation and CR 603.2c's "one or more creatures die" read.
                // Indestructible is not filtered here: CR 702.12b is a "can't"
                // (CR 614.17), asked ahead of the pipeline by
                // `engine::restriction::is_prohibited`, so a CR 614.15 self-replacement
                // (614.17c) can still see the proposal.
                let mut batch = Vec::new();
                for target in targets {
                    if let ResolvedTarget::Object(id) = target
                        && self.battlefield.contains_key(id) {
                        batch.push(GameAction::Destroy {
                            object: *id,
                            source: DestructionSource::Effect(ctx.source),
                        });
                    }
                        // If not on battlefield, destroy does nothing (rule 701.8b)
                }
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            Primitive::Attach => {
                // "Attach this permanent to target ..." (CR 702.6a). The
                // attachment is the ability's source; a spell has none to attach.
                let attachment = ctx.ability_source.map(|r| r.id).ok_or_else(|| {
                    "Primitive::Attach resolved from a spell: only an ability has a permanent to attach"
                        .to_string()
                })?;
                // CR 608.2b partial resolution: the performer is loud, so the caller
                // checks. The Equipment may have left after activation (CR 301.5b: nothing
                // happens), and the target's legality was re-checked against the recipient
                // before `resolve_effect`, which is CR 701.3b's "doesn't move".
                if !self.battlefield.contains_key(&attachment) {
                    return Ok(());
                }
                for target in targets {
                    if let ResolvedTarget::Object(host) = target {
                        if !self.battlefield.contains_key(host) {
                            continue;
                        }
                        self.execute_action(
                            GameAction::Attach { attachment, host: *host },
                            &actx,
                        )?;
                    }
                }
                Ok(())
            }

            Primitive::Untap => {
                // CR 701.26b — untap permanents. `FilteredPermanents` is resolved here
                // rather than filled into `ctx.targets`, for `DealDamage`'s reason: the
                // recipient means "every permanent matching this **now**". Ordered,
                // because the members reach a CR 616.1 prompt and a log — stun counters
                // (CR 122.1d) make two effects want one untap, and the order they are
                // proposed in is observable.
                let ids: Vec<ObjectId> = match recipient {
                    EffectRecipient::FilteredPermanents(filter) => self
                        .battlefield_ids_ordered()
                        .into_iter()
                        .filter(|&id| {
                            self.object_matches_filter(id, filter, ctx.controller)
                                .unwrap_or(false)
                        })
                        .collect(),
                    // CR 608.2b: a spell whose targets are not *all* illegal still resolves
                    // and does as much as it can, so a target that has left is skipped. The
                    // performer is loud, which makes checking here the caller's job — the
                    // shape `Primitive::Destroy` above has.
                    _ => targets
                        .iter()
                        .filter_map(|t| match t {
                            ResolvedTarget::Object(id) if self.battlefield.contains_key(id) => {
                                Some(*id)
                            }
                            _ => None,
                        })
                        .collect(),
                };
                if ids.is_empty() {
                    return Ok(());
                }
                // One batch: CR 701.26b's untaps here happen simultaneously,
                // and CR 603.2c's "whenever one or more permanents untap"
                // reads the batch rather than its members — the same argument
                // the untap step's own sweep makes.
                self.execute_actions(
                    ids.into_iter().map(|object| GameAction::Untap { object }).collect(),
                    &actx,
                )?;
                Ok(())
            }

            // === Phase LB: continuous effect primitives ===

            Primitive::ModifyPowerToughness(power_expr, toughness_expr, duration) => {
                let power = self.evaluate_amount(power_expr, ctx)? as i32;
                let toughness = self.evaluate_amount(toughness_expr, ctx)? as i32;
                let target_ids = self.collect_battlefield_targets(targets);
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
                    affected_objects: crate::engine::layers::ObjectSet::Fixed(target_ids),
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
                let target_ids = self.collect_battlefield_targets(targets);
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
                    affected_objects: crate::engine::layers::ObjectSet::Fixed(target_ids),
                    modification: crate::engine::layers::EffectModification::SetPowerToughness {
                        power: crate::engine::layers::types::PtValue::Fixed(power),
                        toughness: crate::engine::layers::types::PtValue::Fixed(toughness),
                    },
                };
                self.continuous_effects.add(effect);
                Ok(())
            }

            // === Copy effects (CR 707, layer 1a) ===
            Primitive::Copy(roles, duration) => {
                self.apply_copy(roles, *duration, targets, ctx, dp)
            }

            Primitive::SwitchPowerToughness(duration) => {
                let target_ids = self.collect_battlefield_targets(targets);
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
                    affected_objects: crate::engine::layers::ObjectSet::Fixed(target_ids),
                    modification: crate::engine::layers::EffectModification::SwitchPowerToughness,
                };
                self.continuous_effects.add(effect);
                Ok(())
            }

            // === Phase LC: Layer 5 color-changing effects ===

            Primitive::ChangeColor(color_change, duration) => {
                use crate::types::effects::ColorChange;
                let target_ids = self.collect_battlefield_targets(targets);
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
                    affected_objects: crate::engine::layers::ObjectSet::Fixed(target_ids),
                    modification,
                };
                self.continuous_effects.add(effect);
                Ok(())
            }

            // === Phase LD: Layer 4 type-changing effects ===

            Primitive::ChangeType(type_change, duration) => {
                let target_ids = self.collect_battlefield_targets(targets);
                if target_ids.is_empty() {
                    return Ok(());
                }

                // A TypeChange may produce several Layer 4 entries: CR 613.6 makes them
                // siblings ("the parts of the effect each apply in their appropriate
                // layers") and CR 613.7b makes them share a timestamp — one resolution,
                // one moment of creation.
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
                        affected_objects: crate::engine::layers::ObjectSet::Fixed(target_ids.clone()),
                        modification,
                    };
                    self.continuous_effects.add(effect);
                }
                Ok(())
            }

            // === Layer 6 — ability adding and removing (CR 613.1f) ===
            //
            // The resolution half: "target creature gains/loses ..." spells, whose
            // affected set is locked to the targets at resolution (CR 613.7b).

            Primitive::GrantKeywordFlag(keyword, duration) => {
                self.register_resolution_ability_effect(
                    ctx,
                    targets,
                    *duration,
                    EffectModification::GrantKeywordFlag(*keyword),
                );
                Ok(())
            }

            Primitive::RemoveKeywordFlag(keyword, duration) => {
                self.register_resolution_ability_effect(
                    ctx,
                    targets,
                    *duration,
                    EffectModification::RemoveKeywordFlag(*keyword),
                );
                Ok(())
            }

            Primitive::LoseAbility(ability_id, duration) => {
                self.register_resolution_ability_effect(
                    ctx,
                    targets,
                    *duration,
                    EffectModification::LoseAbility(*ability_id),
                );
                Ok(())
            }

            Primitive::LoseAllAbilities(duration) => {
                self.register_resolution_ability_effect(
                    ctx,
                    targets,
                    *duration,
                    EffectModification::LoseAllAbilities,
                );
                Ok(())
            }

            Primitive::GrantAbility(def, duration) => {
                let granted = self.register_resolution_ability_effect(
                    ctx,
                    targets,
                    *duration,
                    EffectModification::GrantAbility(def.clone()),
                );

                // CR 613.7a clause 2: a granted static ability's own effects take the
                // *later* of this effect's timestamp and the grantee's, so a permanent
                // that entered after the grant was created keeps its own.
                // `static_effect_timestamp` takes the max; this supplies the second candidate.
                if let Some((granted_at, row)) = granted {
                    // Re-derived rather than handed back from the call above, so
                    // that the row can own its target `Vec` instead of cloning
                    // it. Safe because registering an effect moves nothing
                    // between zones, so battlefield membership is unchanged.
                    for grantee in self.collect_battlefield_targets(targets) {
                        self.register_granted_static_effects(
                            def, row, grantee, granted_at, *duration, ctx.controller,
                        );
                    }
                }
                Ok(())
            }

            // CR 701.7a — create N tokens: **one proposal, whatever N is**. "Create
            // three 1/1 Soldiers" is one event by CR 111's shape and by CR 614.16's
            // ("one or more tokens"), and the performer is where its entries become
            // one batch (`GameState::create_tokens`). The def is repeated `count`
            // times so a doubler can repeat each def in place and a "one of each"
            // stay one event (`replacement-architecture.md` §3.1).
            Primitive::CreateToken(token_def, amount_expr) => {
                let count = self.evaluate_amount(amount_expr, ctx)?;
                let controller = self.resolve_player_for_self(recipient, targets, ctx);
                // CR 800.4b — no token is created for a player who has left — and
                // CR 800.4d's first sentence at the same line, since CR 111.2 makes the
                // two rules name one player here. A rule checked ahead of the proposal,
                // like CR 508.8's: there is no event here for a replacement to see.
                if self.is_multiplayer() && !self.in_game(controller) {
                    return Ok(());
                }
                let defs = vec![token_def.clone(); count as usize];
                self.execute_action(
                    GameAction::CreateTokens { defs, controller },
                    &ActionContext::resolving(dp, ctx),
                )
            }

            // === Regeneration (CR 701.19) ===

            // CR 701.19a — "creates a replacement effect that protects the permanent
            // the next time it would be destroyed this turn."
            //
            // Every part of the shield comes from the rule — `Uses::Once` is "the
            // next time", `Duration::UntilEndOfTurn` "this turn", `Prevent` "instead",
            // the `then` rider its sentence — so the engine builds it, not a card author.
            Primitive::Regenerate => {
                for object in self.collect_battlefield_targets(targets) {
                    let controller = get_effective_controller(self, object)
                        .unwrap_or(ctx.controller);
                    let def = ReplacementDef::new(
                        EventPattern::Destroy { source: None },
                        ObjectSet::Fixed(vec![object]),
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
                        targets: targets.to_vec(),
                        def,
                    });
                }
                Ok(())
            }

            // === Replacement and prevention effects from a resolution (CR 614.3, 615.7) ===

            // The durational form of `Effect::Replacement`, and `Regenerate`'s
            // general case: the card authors the def and the duration, the
            // resolution supplies its targets, its controller (CR 611.2c) and the turn.
            //
            // **Which permanents at resolution, and which at the event, is the
            // recipient's question.** `Target`/`Choose` and `FilteredPermanents` fix
            // the set *now* — one row per object or player, CR 615.11's shield "for
            // each applicable creature when the spell or ability … resolves" (Samite
            // Censer-Bearer's ruling). `Implicit` leaves the def's own `Filter`/
            // `PlayerSet` to be asked at each event (Safe Passage's opposite ruling).
            // The authored object set has to be the empty `Fixed` on the per-target
            // shapes, for `Primitive::Restrict`'s reason; a `debug_assert` says so.
            Primitive::CreateReplacement(def, duration, pattern_fill) => {
                // CR 113.7a — an ability's source is the object that has it,
                // so a row an activated ability makes names the permanent, not
                // the ephemeral stack object CR 608.2n deletes at the end of
                // resolution. A spell's is the spell.
                let source = ctx.ability_source.map_or(ctx.source, |r| r.id);

                // CR 609.7a — the source is chosen when the effect is created, before the
                // rows are built, since every row a recipient makes watches the same
                // source. `None` means nothing to choose from, a no-op by CR 101.3.
                let def = match pattern_fill {
                    PatternFill::Authored => (**def).clone(),
                    PatternFill::ChosenDamageSource => {
                        match self.fill_chosen_damage_source(def, ctx, dp)? {
                            Some(filled) => filled,
                            None => return Ok(()),
                        }
                    }
                };
                let def = &def;

                let authored_empty = matches!(def.affected_objects, ObjectSet::Fixed(ref ids) if ids.is_empty())
                    && def.affected_players == PlayerSet::Nobody;
                let fill_object = |id: ObjectId| {
                    let mut row = (*def).clone();
                    row.affected_objects = ObjectSet::Fixed(vec![id]);
                    row
                };
                let fill_player = |pid: PlayerId| {
                    let mut row = (*def).clone();
                    row.affected_players = PlayerSet::Fixed(vec![pid]);
                    row
                };
                let rows: Vec<ReplacementDef> = match recipient {
                    EffectRecipient::Target(..)
                    | EffectRecipient::Choose(..)
                    | EffectRecipient::ThisObject
                    | EffectRecipient::TriggeringObject
                    | EffectRecipient::TriggeringPlayer => {
                        debug_assert!(
                            authored_empty,
                            "a `Primitive::CreateReplacement` on {:?} with a targeting \
                             recipient authored a non-empty affected set, which the \
                             resolution then overwrote with its own targets. Write \
                             `ObjectSet::NO_OBJECTS` and `PlayerSet::Nobody`.",
                            ctx.source
                        );
                        targets
                            .iter()
                            .filter_map(|t| match t {
                                // A target that left the battlefield since it
                                // was chosen is CR 608.2b's illegal one: the
                                // rest resolve, it gets nothing.
                                ResolvedTarget::Object(id) if self.battlefield.contains_key(id) => {
                                    Some(fill_object(*id))
                                }
                                ResolvedTarget::Object(_) => None,
                                ResolvedTarget::Player(pid) => Some(fill_player(*pid)),
                            })
                            .collect()
                    }
                    EffectRecipient::SameInstanceAs(_) => return Err(back_reference(recipient, ctx)),
                    // CR 615.11 — one row per applicable *permanent*, fixed at resolution and
                    // ordered because the rows are offered to CR 616.1 prompts in registration
                    // order. A row on a card in another zone is §3.3 source 2 and needs
                    // CR 113.6 (`roadmap-v2.md` A5) before it could fire.
                    EffectRecipient::FilteredObjectsIn(..) => {
                        return Err(format!(
                            "a `Primitive::CreateReplacement` on {:?} has a zone-reaching recipient; see roadmap-v2.md A5",
                            ctx.source
                        ));
                    }
                    EffectRecipient::FilteredPermanents(filter) => {
                        debug_assert!(
                            authored_empty,
                            "a `Primitive::CreateReplacement` on {:?} with a filter \
                             recipient authored a non-empty affected set; the filter \
                             is what names the permanents, one row each (CR 615.11).",
                            ctx.source
                        );
                        self.battlefield_ids_ordered()
                            .into_iter()
                            .filter(|id| {
                                self.object_matches_filter(*id, filter, ctx.controller)
                                    .unwrap_or(false)
                            })
                            .map(fill_object)
                            .collect()
                    }
                    // As authored — a `Filter` and/or a `PlayerSet`, asked at
                    // each event. "You" is `PlayerSet::You`, resolved against
                    // the row's controller; there is no second way to say it.
                    EffectRecipient::Implicit | EffectRecipient::Controller => {
                        if authored_empty {
                            return Err(format!(
                                "a `Primitive::CreateReplacement` on {:?} names no target, \
                                 no filter and no player, so its row could never apply. \
                                 Give it a `Target` recipient, a `FilteredPermanents` \
                                 recipient, or an `ObjectSet::Filter`/`PlayerSet` of \
                                 its own.",
                                ctx.source
                            ));
                        }
                        vec![(*def).clone()]
                    }
                    EffectRecipient::Host => {
                        return Err(format!(
                            "a `Primitive::CreateReplacement` on {:?} has a `Host` recipient, \
                             which only a static ability's continuous effect can have.",
                            ctx.source
                        ))
                    }
                };
                for row in rows {
                    self.replacement_effects.add(RegisteredReplacementEffect {
                        id: 0,
                        source,
                        controller: ctx.controller,
                        duration: *duration,
                        created_on_turn: self.turn_number,
                        targets: targets.to_vec(),
                        def: row,
                    });
                }
                Ok(())
            }

            // === "Can't" effects (CR 101.2) ===

            // One registry row per resolved target, and the **resolution supplies
            // the affected set** — a card file cannot name a target it has not yet
            // chosen (`Primitive::Regenerate`'s reason). The authored set is therefore
            // required to be an empty `Fixed`, and the `debug_assert` says so. The
            // `Duration` is the card's, never the engine's: CR 608.2c hands scope to a
            // human reader (`cant-effects-architecture.md` §9 finding 1).
            Primitive::Restrict(def, duration) => {
                // **A restriction that names nobody at all is the one waiting for the
                // resolution's targets**: a card file writes an empty `Fixed` beside
                // `Nobody` and this fills the object in, one row per target, because one
                // row naming both would make a second copy of the spell a no-op under
                // CR 614.5. Everything else is complete as authored and gets one row —
                // Skullcrack's "players can't gain life this turn" is `Everyone` with no
                // object while its target is the player it then damages. Not CR 608.2b:
                // a spell whose only target is illegal never reaches `resolve_effect`;
                // this decides whether a resolved spell's restriction is about its target.
                let mut rows: Vec<RestrictionDef> = Vec::new();
                let (objects, players) = restriction_scope(def);
                if matches!(objects, ObjectSet::Fixed(ids) if ids.is_empty())
                    && matches!(players, PlayerSet::Nobody)
                {
                    for object in self.collect_battlefield_targets(targets) {
                        let mut filled = def.clone();
                        *restriction_object_set_mut(&mut filled) =
                            ObjectSet::Fixed(vec![object]);
                        rows.push(filled);
                    }
                } else {
                    rows.push(def.clone());
                }
                for def in rows {
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

            // CR 701.21a — "its controller moves it from the battlefield directly to
            // its owner's graveyard".
            //
            // **The choosing player is the *targeted* player, not the spell's
            // controller** (Diabolic Edict), which makes this the resolution-time
            // selection path `cant-effects-architecture.md` §4.9 puts its candidate
            // filter in, so Sigarda produces no prompt rather than a refused one.
            // Not destruction (CR 701.21b): regeneration and indestructible do not
            // apply, which is why the cause is its own `ZoneChangeCause` variant.
            Primitive::Sacrifice(filter, amount) => {
                let count = self.evaluate_amount(amount, ctx)?;
                for target in targets {
                    let ResolvedTarget::Player(player) = target else {
                        continue;
                    };
                    self.sacrifice_of_choice(*player, filter, count, ctx, dp)?;
                }
                Ok(())
            }

            // === Combat and damage housekeeping ===

            Primitive::Tap => {
                // One batch: CR 608.2f processes a spell's actions over several
                // objects simultaneously.
                let batch = self
                    .collect_battlefield_targets(targets)
                    .into_iter()
                    .map(|object| GameAction::Tap { object })
                    .collect();
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // CR 506.4. Writes `PermanentState` directly: 506.4 defines a
            // *consequence* with seven causes, and this arm is one of them. The CR
            // does not forbid "can't be removed from combat"; it puts that card's
            // enforcement at all seven causes, which makes it a `RestrictionDef`
            // consulted by each (`cant-effects-architecture.md`) and not a
            // replaceable event here → `replacement-architecture.md` §8a.
            Primitive::RemoveFromCombat => {
                for object in self.collect_battlefield_targets(targets) {
                    self.remove_from_combat(object);
                }
                Ok(())
            }

            // A direct write because nothing replaces *this* removal — Pyramids, the
            // one printed card that removes all damage, uses it as a replacement's
            // substituted event. Not by analogy to the CR 514.2 cleanup wipe, which
            // seven cards restrict and which owes an enforcement point
            // (`codebase-state.md`, Before Replacement item 20).
            Primitive::RemoveAllDamage => {
                for object in self.collect_battlefield_targets(targets) {
                    if let Some(entry) = self.battlefield.get_mut(&object) {
                        entry.damage_marked = 0;
                        entry.damaged_by_deathtouch = false;
                    }
                }
                Ok(())
            }

            // === Counters (CR 122) ===
            //
            // Both propose rather than writing `PermanentState.counters`: CR 614.16's
            // doublers replace a counter mutation, and CR 122.1c/d's own replacement
            // effects *produce* one ("instead remove a stun counter from it").

            Primitive::AddCounters { counter, amount, by } => {
                let n = self.evaluate_amount(amount, ctx)? as u32;
                let by = self.resolve_putter(by, targets, ctx)?;
                // One batch: CR 608.2f processes a spell's actions over several
                // objects simultaneously, which is what lets a single CR 614.16
                // doubler see all of them.
                let batch = self
                    .collect_battlefield_targets(targets)
                    .into_iter()
                    .map(|object| GameAction::AddCounters {
                        subject: CounterSubject::Object(object),
                        counter: *counter,
                        n,
                        by,
                    })
                    .collect();
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            Primitive::RemoveCounters(counter_type, amount_expr) => {
                let n = self.evaluate_amount(amount_expr, ctx)? as u32;
                let batch = self
                    .collect_battlefield_targets(targets)
                    .into_iter()
                    .map(|object| GameAction::RemoveCounters {
                        subject: CounterSubject::Object(object),
                        counter: *counter_type,
                        n,
                    })
                    .collect();
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // "You get {E}{E}" — the same event with a player as its subject,
            // so Winding Constrictor's "if you would get one or more counters"
            // watches it through the one arm.
            Primitive::GetCounters { counter, amount, by } => {
                let n = self.evaluate_amount(amount, ctx)? as u32;
                let player = self.resolve_player_for_self(recipient, targets, ctx);
                let by = self.resolve_putter(by, targets, ctx)?;
                self.execute_action(
                    GameAction::AddCounters {
                        subject: CounterSubject::Player(player),
                        counter: *counter,
                        n,
                        by,
                    },
                    &actx,
                )?;
                Ok(())
            }

            // === Layer 2 — control-changing effects (CR 613.1b) ===

            Primitive::GainControl(duration) => {
                let target_ids = self.collect_permanent_or_spell_targets(targets);
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
                    affected_objects: ObjectSet::Fixed(target_ids),
                    // `You` rather than `ctx.controller`, though they name the
                    // same player: on a `Resolution` row, `FilterPlayers::you()`
                    // reads `ContinuousEffect.controller`, which CR 611.2c
                    // locked at resolution.
                    modification: EffectModification::SetController(PlayerRef::You),
                });
                Ok(())
            }

            // CR 701.24a — whose library is the recipient's whole question.
            // `Controller` is "shuffle your library"; `ThisObject` is the *source's
            // owner's* — "shuffle it into **its owner's** library" as the rider of
            // a replacement whose substitute has already made the move, where "it"
            // is the source (Darksteel Colossus). The owner survives the move (CR
            // 108.3), so it is read wherever the card went. Moving nothing here is CR 701.24c:
            // a 903.9b that sent a commander to the command zone instead leaves it
            // there, and its owner's library is shuffled all the same. A target is
            // a player, or an object standing for its owner. The filter recipients
            // are refused by name: no card shuffles a library per matching object.
            Primitive::ShuffleLibrary => {
                let players: Vec<PlayerId> = match recipient {
                    EffectRecipient::Controller => vec![ctx.controller],
                    EffectRecipient::ThisObject => {
                        vec![self.get_object(ctx.ability_source.map_or(ctx.source, |r| r.id))?.owner]
                    }
                    EffectRecipient::Implicit => {
                        return Err(format!(
                            "a `Primitive::ShuffleLibrary` on {:?} names no library; use `Controller` or `ThisObject`",
                            ctx.source
                        ));
                    }
                    EffectRecipient::Target(..)
                    | EffectRecipient::Choose(..)
                    | EffectRecipient::TriggeringObject
                    | EffectRecipient::TriggeringPlayer => targets
                        .iter()
                        .filter_map(|t| match t {
                            ResolvedTarget::Player(pid) => Some(*pid),
                            ResolvedTarget::Object(id) => {
                                self.objects.get(id).map(|obj| obj.owner)
                            }
                        })
                        .collect(),
                    EffectRecipient::SameInstanceAs(_) => return Err(back_reference(recipient, ctx)),
                    EffectRecipient::FilteredPermanents(_)
                    | EffectRecipient::FilteredObjectsIn(..)
                    | EffectRecipient::Host => {
                        return Err(format!(
                            "a `Primitive::ShuffleLibrary` on {:?} has a filter recipient, \
                             and a library is a player's, not an object's",
                            ctx.source
                        ));
                    }
                };
                // One batch, not a loop: "each player shuffles" (Timetwister) is N
                // libraries shuffled at once, which CR 704.3's batching and CR 101.4's
                // APNAP order want as one event of N members, one per library — a
                // shuffle stays a fact about one player's library (CR 701.24a).
                let batch: Vec<GameAction> = players
                    .into_iter()
                    .map(|player| GameAction::ShuffleLibrary { player })
                    .collect();
                if !batch.is_empty() {
                    self.execute_actions(batch, &actx)?;
                }
                Ok(())
            }

            // === Unimplemented primitives — `backlog.md` §2.5 ===

            Primitive::ReturnToHand
            | Primitive::ReturnToBattlefield
            | Primitive::PutOnTopOfLibrary
            | Primitive::PutOnBottomOfLibrary
            | Primitive::ShuffleIntoLibrary
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
    /// Returns the timestamp allocated for the row and the row's id, so a caller
    /// that must register further rows against the same moment, or name the
    /// instance a grant made, can. `None` means no target survived, in which case
    /// no row is written and no timestamp is burned.
    fn register_resolution_ability_effect(
        &mut self,
        ctx: &ResolutionContext,
        targets: &[ResolvedTarget],
        duration: Duration,
        modification: EffectModification,
    ) -> Option<(Timestamp, EffectId)> {
        let targets = self.collect_battlefield_targets(targets);
        if targets.is_empty() {
            return None;
        }
        let timestamp = self.allocate_timestamp();
        let row = self.continuous_effects.add(ContinuousEffect {
            id: 0,
            source: ctx.source,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer6Ability,
            duration,
            controller: ctx.controller,
            created_on_turn: self.turn_number,
            timestamp,
            affected_objects: ObjectSet::Fixed(targets),
            modification,
        });
        Some((timestamp, row))
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
    /// keyed on the id the grant's instance carries on `grantee` (`row` is the
    /// granting row), so `compute::static_ability_still_exists` re-asks at every
    /// layer whether `grantee` still has that instance. If the
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
        row: EffectId,
        grantee: ObjectId,
        granting_timestamp: Timestamp,
        duration: Duration,
        controller: PlayerId,
    ) {
        use crate::objects::card_data::AbilityType;

        // The expected path, not a guard: most granted abilities are triggered,
        // activated or mana abilities, and only a static one generates a
        // continuous effect, so everything else registers nothing and still lands
        // on the object via the Layer 6 row.
        if granted.ability_type != AbilityType::Static {
            return;
        }

        // Deliberately does NOT check `is_characteristic_defining`: a Layer 6
        // grant is none of CR 604.3a(2)'s routes to a CDA, and the grant arm in
        // `layers::compute` clears the flag as it lands, so the object holds an
        // *ordinary* static ability and treating it as one is the only right
        // answer (`CLAUDE.md`).

        // Same two helpers `register_static_effects` uses, for the reason
        // `static_primitive_rows`' doc gives: a granted "creatures you control
        // get +1/+1" must produce the row a printed one does. Assertions included —
        // a grant that lowers to nothing is exactly as inert and as invisible.
        let context = format!("granted ability {:?}", granted.id);
        let atoms = GameState::static_ability_atoms(granted, &context);

        for (primitive, recipient) in atoms {
            let Some(affected) = GameState::static_object_set(recipient, &context) else {
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
                // A granted static ability whose own effect lands in layers 1–5 cannot
                // apply, and would fail silently: the grant applies AT layer 6, so below
                // it the CR 604.2 existence check reads a frame that predates the grant.
                // Assert at the authoring site rather than let a card quietly do nothing.
                //
                // Layer 6 itself is fine: the pass applies a layer board-wide in one
                // sequence, so the existence check at the derived row's turn sees the
                // grant (CR 613.7a's own example, Rune of Flight), and clause 2's
                // `max(grantee, grant)` timestamp with the lower id on a tie sorts the
                // grant at-or-before its derived effect. Layers 1–5 have no CR mechanism
                // for a layer 6 grant to reach back (CR 613.8a), and a Scryfall search
                // finds no printed grant of a type-, color- or subtype-defining static.
                debug_assert!(
                    layer >= Layer::Layer6Ability,
                    concat!(
                        "granted static ability generates a {:?} effect. A grant ",
                        "applies at layer 6, so the CR 604.2 existence check ",
                        "reads a pre-grant frame at any layer below it and this ",
                        "effect will not apply; layers 1-5 have no CR mechanism ",
                        "and no known card."
                    ),
                    layer
                );

                self.continuous_effects.add(ContinuousEffect {
                    id: 0,
                    source: grantee,
                    origin: EffectOrigin::StaticAbility { ability: granted.id.granted_by(row) },
                    layer,
                    duration,
                    controller,
                    created_on_turn: self.turn_number,
                    timestamp,
                    affected_objects: affected.clone(),
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
    /// CR 707.4 — "[objects] become a copy of [object] [for a duration]".
    ///
    /// Three steps, in this order and for CR reasons rather than convenience:
    /// resolve the two roles (which is where the CR 707.4 choice is made),
    /// capture (CR 707.2, once — 707.2b/2c), register one layer 1a row whose
    /// affected set is `Fixed` (CR 611.2c).
    ///
    /// **CR 707.4's three free clauses.** "The change doesn't cause
    /// enters-the-battlefield or leaves-the-battlefield abilities to trigger.
    /// This also doesn't change any noncopy effects presently affecting the
    /// permanent." Both fall out of the carrier: registering a row is not a zone
    /// change, and layers 2—7 are not touched. That is the clearest single
    /// piece of evidence that a row is the right carrier and a `CardData` swap
    /// is not.
    fn apply_copy(
        &mut self,
        roles: &CopyRoles,
        duration: Duration,
        targets: &[ResolvedTarget],
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        use crate::engine::layers::types::{
            ObjectSet, ContinuousEffect, EffectModification, EffectOrigin, Layer,
        };

        // --- 1. The two roles -------------------------------------------
        let (donor, affected): (ObjectId, Vec<ObjectId>) = match roles {
            CopyRoles::RecipientsCopyChosen(filter) => {
                // CR 608.2b — a target that has left the battlefield is simply
                // skipped, and with nothing left to affect there is no effect
                // and so no choice to make. Asking first would prompt for a
                // decision that changes nothing.
                let recipients = self.collect_battlefield_targets(targets);
                if recipients.is_empty() {
                    return Ok(());
                }
                let candidates: Vec<ObjectId> = crate::oracle::legality::
                    enumerate_legal_selections(self, filter, None, ctx.controller)
                    .into_iter()
                    .filter_map(|t| match t {
                        ResolvedTarget::Object(id) => Some(id),
                        ResolvedTarget::Player(_) => None,
                    })
                    .collect();
                // CR 102.2 — one candidate is not a choice, none is CR 101.3's
                // impossible instruction and the whole effect does nothing.
                let donor = match candidates.len() {
                    0 => return Ok(()),
                    1 => candidates[0],
                    _ => crate::ui::ask::ask_choose_copy_source(
                        dp, self, ctx.controller, ctx.source, &candidates,
                    ),
                };
                (donor, recipients)
            }
            CopyRoles::FilteredCopyRecipient { filter, exclude_donor } => {
                let Some(&donor) = self.collect_battlefield_targets(targets).first() else {
                    return Ok(());
                };
                // Ordered, because the row's `Fixed` set is read back by
                // `battlefield_ids_ordered`'s consumers and reaches a log and a
                // count. CR 613.7's timestamp order is the key either way.
                let affected: Vec<ObjectId> = self
                    .battlefield_ids_ordered()
                    .into_iter()
                    .filter(|&id| !(*exclude_donor && id == donor))
                    .filter(|&id| {
                        self.object_matches_filter(id, filter, ctx.controller)
                            .unwrap_or(false)
                    })
                    .collect();
                (donor, affected)
            }
        };
        if affected.is_empty() {
            return Ok(());
        }

        // --- 2. The capture (CR 707.2) ----------------------------------
        //
        // Once, here, and never re-derived (CR 707.2b; CR 611.2c for a
        // resolution's) — the opposite of every other continuous effect here,
        // and the whole reason `CopyFrom` carries values rather than an `ObjectId`.
        let Some(values) = crate::engine::layers::copiable_values(self, donor) else {
            return Ok(());
        };

        // --- 3. The row (CR 613.2a) -------------------------------------
        let timestamp = self.allocate_timestamp();
        self.continuous_effects.add(ContinuousEffect {
            id: 0,
            source: ctx.source,
            origin: EffectOrigin::Resolution,
            layer: Layer::Layer1Copy,
            duration,
            controller: ctx.controller,
            created_on_turn: self.turn_number,
            timestamp,
            affected_objects: ObjectSet::Fixed(affected.clone()),
            modification: EffectModification::CopyFrom(Box::new(values.clone())),
        });

        // `copy-effects-architecture.md` §4.7 leg 2: the row alone makes the
        // copy *have* the ability; it does not make the ability *do* anything.
        self.register_copied_static_effects(
            &values, &affected, timestamp, duration,
        );
        Ok(())
    }

    /// CR 609.7a — ask for the source of damage a
    /// [`Primitive::CreateReplacement`] names and write it into the def's
    /// pattern.
    ///
    /// > 609.7a ... The source is chosen when the effect is created.
    ///
    /// **The pattern field, not the affected set** — which is the whole reason
    /// `PatternFill` exists beside the recipient. Circle of Protection: Red's
    /// row is around *you* and watches *one object*; a `Choose` recipient
    /// could carry the object and would then have nothing left to say the row
    /// is about a player.
    ///
    /// `Ok(None)` is CR 101.3's impossible instruction: with no legal source
    /// there is nothing to choose and no row worth adding. `Err` is reserved
    /// for a def whose pattern has no `SourcePattern` to fill, which is a card
    /// that would silently do nothing.
    fn fill_chosen_damage_source(
        &self,
        def: &ReplacementDef,
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
    ) -> Result<Option<ReplacementDef>, String> {
        use crate::types::replacement::EventPattern;

        let EventPattern::DealDamage { source: Some(pattern), .. } = &def.pattern else {
            return Err(format!(
                "a `Primitive::CreateReplacement` on {:?} asks for CR 609.7a's chosen \
                 source, but its pattern is {:?} — only an \
                 `EventPattern::DealDamage` with a `SourcePattern` has a field to \
                 write the choice into.",
                ctx.source, def.pattern
            ));
        };
        // The shape is the card's and the object is the resolution's, so an
        // authored id would be one the choice then overwrote —
        // `Primitive::Restrict`'s `debug_assert` one level down.
        debug_assert!(
            pattern.object.is_none(),
            "a `Primitive::CreateReplacement` on {:?} authored a chosen damage source \
             ({:?}) that the resolution then overwrote. Write \
             `SourcePattern::chosen()` or `SourcePattern::matching(..)`.",
            ctx.source,
            pattern.object
        );

        let candidates: Vec<ObjectId> = crate::oracle::legality::enumerate_legal_selections(
            self,
            &SelectionFilter::DamageSource,
            None,
            ctx.controller,
        )
        .into_iter()
        .filter_map(|t| match t {
            ResolvedTarget::Object(id) => Some(id),
            ResolvedTarget::Player(_) => None,
        })
        .collect();
        // CR 102.2 — one candidate is not a choice; none is CR 101.3's
        // impossible instruction and the effect does nothing.
        let chosen = match candidates.len() {
            0 => return Ok(None),
            1 => candidates[0],
            _ => crate::ui::ask::ask_choose_damage_source(
                dp,
                self,
                ctx.controller,
                ctx.source,
                &candidates,
            ),
        };

        let mut filled = def.clone();
        if let EventPattern::DealDamage { source: Some(pattern), .. } = &mut filled.pattern {
            pattern.object = Some(chosen);
        }
        Ok(Some(filled))
    }

    fn sacrifice_of_choice(
        &mut self,
        player: PlayerId,
        filter: &SelectionFilter,
        count: u64,
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        let candidates: Vec<ResolvedTarget> =
            crate::oracle::legality::enumerate_legal_selections(self, filter, None, player)
                .into_iter()
                .filter(|t| match t {
                    // "Its controller moves it": only your own permanents.
                    ResolvedTarget::Object(id) => {
                        controls(self, *id, player)
                            && !self.sacrifice_is_prohibited(*id, ctx.controller)
                    }
                    ResolvedTarget::Player(_) => false,
                })
                .collect();

        // CR 101.3 — "if a player is instructed to do something impossible,
        // only the possible portion is performed". Blasphemous Edict asks for
        // thirteen and a player with two sacrifices two; the same clamp is what
        // makes an empty pool a silent no-op rather than an error.
        let n = (count as usize).min(candidates.len());
        if n == 0 {
            return Ok(());
        }

        let recipient =
            EffectRecipient::Choose(filter.clone(), TargetCount::Exactly(n as u32));
        let chosen = crate::ui::ask::ask_select_recipients(
            dp, self, player, &recipient, ctx.source, &candidates, n, n,
        );

        // One batch, not a loop: CR 701.21 sacrifices happen simultaneously
        // (Barter in Blood's two creatures die as one event), and CR 704.3's
        // single event and CR 615.7's allocation are unreachable from a loop.
        let actx = ActionContext::resolving(dp, ctx);
        let batch: Vec<GameAction> = chosen
            .iter()
            .filter_map(|t| match t {
                ResolvedTarget::Object(id) => Some(GameAction::ZoneChange {
                    object: *id,
                    from: Zone::Battlefield,
                    to: Zone::Graveyard,
                    cause: ZoneChangeCause::Sacrificed,
                }),
                ResolvedTarget::Player(_) => None,
            })
            .collect();
        self.execute_actions(batch, &actx)?;
        Ok(())
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
                lookahead: None,
            },
        )
    }

    fn collect_battlefield_targets(&self, targets: &[ResolvedTarget]) -> Vec<ObjectId> {
        targets.iter()
            .filter_map(|t| {
                if let ResolvedTarget::Object(id) = t
                    && self.battlefield.contains_key(id) {
                    return Some(*id);
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
    fn collect_permanent_or_spell_targets(&self, targets: &[ResolvedTarget]) -> Vec<ObjectId> {
        targets
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

    pub(crate) fn evaluate_amount(
        &self,
        expr: &AmountExpr,
        _ctx: &ResolutionContext,
    ) -> Result<u64, String> {
        match expr {
            AmountExpr::Fixed(n) => Ok(*n),
            AmountExpr::X => {
                // `StackEntry::x_value` has held it since the cast; reading it here is main
                // item 90's PR (`codebase-state.md`), with the card that needs it.
                Err("X amount resolution not yet implemented".to_string())
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
            // "That many" on a triggered ability: the matched records' amount,
            // through the arm's projection. Refused outside a trigger and for
            // an arm that carries no quantity, rather than answering 0.
            AmountExpr::TriggeringAmount => {
                let binding = _ctx.trigger.as_ref().ok_or_else(|| {
                    "TriggeringAmount has no meaning outside a triggered ability's resolution".to_string()
                })?;
                self.bound_amount(binding).ok_or_else(|| {
                    format!("{:?} carries no amount for TriggeringAmount to read", binding.event())
                })
            }
            // "Its power", "its toughness" (CR 608.2h). A negative value is
            // no amount: Paladin of Atonement's ruling gains nothing, and
            // loses nothing, for toughness below 0.
            AmountExpr::TriggeringPower | AmountExpr::TriggeringToughness => {
                let binding = _ctx.trigger.as_ref().ok_or_else(|| {
                    format!("{:?} has no meaning outside a triggered ability's resolution", expr)
                })?;
                let chars = self.bound_characteristics(binding).ok_or_else(|| {
                    format!(
                        "{:?}: the bound object has left since the event and no frame answers for it (TR-2b's departed frames)",
                        expr
                    )
                })?;
                let value = match expr {
                    AmountExpr::TriggeringPower => chars.power,
                    _ => chars.toughness,
                };
                Ok(value.unwrap_or(0).max(0) as u64)
            }
            // "This creature's power" is a *replacement effect's* question: CR 614.12
            // asks it of a permanent about to enter, and
            // `replacement::evaluate_enter_template` is the one evaluator that knows
            // whether to read the board or the look-ahead frame (Master Biomancer).
            AmountExpr::SourcePower => Err(
                "SourcePower has no meaning at resolution time; it is evaluated against the CR 614.12 frame when an entry replacement is applied"
                    .to_string(),
            ),
            AmountExpr::TargetToughness => {
                Err("TargetToughness amount resolution not yet implemented".to_string())
            }
            AmountExpr::DamageDealtThisWay => {
                Err("DamageDealt amount resolution not yet implemented".to_string())
            }
            // CR 615.5's "that much"/"that many". Only a rider sets the field, so
            // outside a `ReplacementDef::then` this leaf asks a question its context
            // cannot answer, and a 0 would be silently wrong.
            AmountExpr::ReplacedAmount => _ctx.replaced_amount.ok_or_else(|| {
                "ReplacedAmount has no meaning outside a CR 615.5 rider".to_string()
            }),
            // Doubling Cube's "each type of unspent mana you have": the
            // controller's pool now, restricted units counted by their type
            // (CR 106.6 — a restriction "doesn't affect the mana's type").
            AmountExpr::UnspentMana(mana_type) => {
                Ok(self.get_player(_ctx.controller)?.mana_pool.unspent(*mana_type))
            }
            // The other number a CR 615.5 rider may refer to, and the same
            // refusal outside one.
            AmountExpr::DamagePrevented => _ctx.damage_prevented.ok_or_else(|| {
                "DamagePrevented has no meaning outside a CR 615.5 rider".to_string()
            }),
            // Saturating rather than checked: the inner amount is a damage or life
            // number and the factor is printed on a card, and a debug panic or a
            // release wrap is not an answer. `AmountRewrite::Multiplier` saturates too.
            AmountExpr::Multiply(inner, n) => {
                Ok(self.evaluate_amount(inner, _ctx)?.saturating_mul(*n))
            }
            // CR 103.4's number is the game's, and never below zero.
            AmountExpr::StartingLifeTotal => Ok(self.starting_life.max(0) as u64),
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

    // --- Helper: determine which player an effect applies to ---

    /// For effects that target "you" (the controller) or use EffectRecipient::Implicit,
    /// returns the controller. For targeted player effects, returns the first
    /// player target.
    /// Who puts the counters on, for `Primitive::AddCounters` and
    /// `GetCounters` (CR 122.6a's shape on a proposal).
    ///
    /// `You` is the effect's controller — every printed one-shot, and the
    /// card writes it. `Opponent` is the resolution's player target when it
    /// has one, else the only opponent still in the game; with several and
    /// no target it is an authoring error and loud. Bold Plagiarist's
    /// "*they* put" is the printed customer, a trigger whose effect names the
    /// player who triggered it — `Player(id)` once CR 603 fills it.
    fn resolve_putter(
        &self,
        by: &PlayerRef,
        targets: &[ResolvedTarget],
        ctx: &ResolutionContext,
    ) -> Result<PlayerId, String> {
        Ok(match by {
            PlayerRef::You => ctx.controller,
            PlayerRef::Player(pid) => *pid,
            PlayerRef::Owner => self.get_object(ctx.source)?.owner,
            PlayerRef::Opponent => {
                let targeted = targets.iter().find_map(|t| match t {
                    ResolvedTarget::Player(pid) if *pid != ctx.controller => Some(*pid),
                    _ => None,
                });
                match targeted {
                    Some(pid) => pid,
                    None => {
                        let opponents: Vec<PlayerId> = (0..self.num_players())
                            .filter(|&p| p != ctx.controller && !self.player_lost[p])
                            .collect();
                        match opponents.as_slice() {
                            [only] => *only,
                            others => {
                                return Err(format!(
                                    "an effect names \"an opponent\" as the player putting \
                                     counters on, targets none, and player {} has {} \
                                     opponents in the game",
                                    ctx.controller,
                                    others.len()
                                ))
                            }
                        }
                    }
                }
            }
        })
    }

    /// CR 113.7a's "this [object]" for a resolution: the ability's source,
    /// found by identity (CR 400.7), else the spell or replacement source itself
    /// while it is still where the effect found it. `None` is the object
    /// being gone, which the primitive meets as an empty target slice.
    pub(crate) fn this_object(&self, ctx: &ResolutionContext) -> Option<ObjectId> {
        match ctx.ability_source {
            Some(source) => (self.object_ref(source.id) == Some(source)).then_some(source.id),
            None => {
                let id = ctx.source;
                let here = self.battlefield.contains_key(&id)
                    || self.stack_entries.contains_key(&id)
                    || self.resolving.as_ref().is_some_and(|r| r.id == id);
                here.then_some(id)
            }
        }
    }

    fn resolve_player_for_self(
        &self,
        recipient: &EffectRecipient,
        targets: &[ResolvedTarget],
        ctx: &ResolutionContext,
    ) -> PlayerId {
        match recipient {
            EffectRecipient::Implicit | EffectRecipient::Controller => ctx.controller,
            EffectRecipient::Target(SelectionFilter::Player, _) | EffectRecipient::TriggeringPlayer => {
                for t in targets {
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

/// `resolve_primitive` is handed the clause an instance was **declared** with,
/// never a back-reference to it — `targeting::instance_of` resolves
/// `EffectRecipient::SameInstanceAs` before the primitive sees it. Reaching one here
/// means the walk was bypassed, which is a wiring error rather than a card's.
fn back_reference(recipient: &EffectRecipient, ctx: &ResolutionContext) -> String {
    format!(
        "a primitive on {:?} was handed {:?}, a back-reference to an instance of \"target\" rather than the clause that declared it (CR 115.3). Resolve it through `targeting::instance_of`.",
        ctx.source, recipient
    )
}

/// The affected set inside a [`RestrictionDef`], whichever arm it is.
///
/// One accessor rather than a match at each caller: every `Restriction` arm
/// names the objects it applies to, and `Primitive::Restrict` overwrites that
/// set with the resolution's targets without caring which arm it is. Matched
/// exhaustively, so a new arm has to say where its objects live rather than
/// silently keeping whatever the card wrote.
fn restriction_object_set_mut(def: &mut RestrictionDef) -> &mut ObjectSet {
    match &mut def.what {
        Restriction::Event { affected_objects, .. } => affected_objects,
        Restriction::ApplyReplacement { to_objects, .. } => to_objects,
    }
}

/// Both halves of a [`RestrictionDef`]'s scope, whichever arm it is.
///
/// The read-only sibling of [`restriction_object_set_mut`], and it returns
/// the *pair* because that is the question `Primitive::Restrict` asks: a
/// restriction naming no objects and no players is the one whose subject the
/// resolution supplies. Either half alone would answer it wrongly — Skullcrack
/// names no object and every player.
fn restriction_scope(def: &RestrictionDef) -> (&ObjectSet, &PlayerSet) {
    match &def.what {
        Restriction::Event { affected_objects, affected_players, .. } => (affected_objects, affected_players),
        Restriction::ApplyReplacement { to_objects, to_players, .. } => (to_objects, to_players),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::state::battlefield::PermanentState;
    use crate::types::card_types::*;
    use crate::types::mana::ManaType;
    use crate::types::zones::Zone;
    use crate::test_support::test_dp;

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
        let id = game.add_object(obj);
        let entry = PermanentState::new(id, 0, 1);
        game.insert_battlefield_entity(id, entry);

        (game, id)
    }

    fn bolt_ctx(source: ObjectId, targets: Vec<ResolvedTarget>) -> ResolutionContext {
        ResolutionContext {
            source,
            ability_source: None,
            controller: 0,
            targets: ChosenTargets::one(targets),
            replaced_amount: None,
            damage_prevented: None,
            trigger: None,
        }
    }

    #[test]
    fn test_deal_damage_to_creature() {
        let (mut game, bears_id) = setup_game_with_creature();

        let bolt = Effect::Atom(
            Primitive::DealDamage { amount: AmountExpr::Fixed(3), unpreventable: false },
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
            Primitive::DealDamage { amount: AmountExpr::Fixed(3), unpreventable: false },
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
            let oid = game.add_object(obj);
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
            let oid = game.add_object(obj);
            game.players[0].library.push(oid);
        }

        let effect = Effect::Sequence(vec![
            Effect::Atom(
                Primitive::DealDamage { amount: AmountExpr::Fixed(2), unpreventable: false },
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
            .keyword_flag(crate::types::keywords::KeywordFlag::Indestructible)
            .build();
        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let target_id = game.add_object(obj);
        let entry = PermanentState::new(target_id, 0, 1);
        game.insert_battlefield_entity(target_id, entry);

        // Create a source for the destroy effect
        let bolt_data = CardDataBuilder::new("Doom Blade")
            .card_type(CardType::Instant)
            .build();
        let bolt_obj = GameObject::new(bolt_data, 0, Zone::Hand);
        let source_id = game.add_object(bolt_obj);

        let destroy = Effect::Atom(
            Primitive::Destroy,
            EffectRecipient::Target(SelectionFilter::Creature, crate::types::effects::TargetCount::Exactly(1)),
        );
        let ctx = bolt_ctx(source_id, vec![ResolvedTarget::Object(target_id)]);
        game.resolve_effect(&destroy, &ctx, &test_dp()).unwrap();

        // Creature should still be on the battlefield
        assert!(game.battlefield.contains_key(&target_id));
    }
}

