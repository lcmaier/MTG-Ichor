use crate::engine::object_filter::{EarlierTargets, TextReferents};
use crate::engine::targeting::FilterIdentity;
use crate::engine::actions::{
    ActionContext, GameAction, ZoneChangeAct,
};
use crate::engine::layers::types::{
    ObjectSet, ContinuousEffect, EffectId, EffectModification, EffectOrigin, Layer, PtValue, Timestamp,
};
use crate::events::cause::{EventCause, GeneratedBy};
use crate::events::event::{EventId, GameEvent, ResolutionStamp};
use crate::engine::targeting::{instance_of, ChosenTargets, DeclaredInstances, TargetInstance};
use crate::oracle::characteristics::ObjectInformation;
use crate::objects::card_data::AbilityDef;
use crate::types::zones::{ZoneSet, Zone};
use crate::state::battlefield::AttackTarget;
use crate::state::game_state::{GameState, PhaseType, PlannedPhase};
use crate::types::effects::{
    AmountExpr, Antecedent, Choice, ChoiceScope, Condition, CopyException, CopyRoles, CostAnswer, DiscardChooser, Duration, Effect,
    EffectRecipient, NamedPlayers, ObjectFilter, PatternFill, PickCount, PlayerGroup, PlayerRef, PlayerSet, Primitive,
    SelectionFilter, TargetCount,
};
use crate::oracle::characteristics::{controls, defending_player_for, get_effective_controller};
use crate::state::replacement_effects::RegisteredReplacementEffect;
use crate::state::restrictions::RegisteredRestriction;
use crate::types::restriction::RestrictionDef;
use crate::types::ids::{ExtraTurnId, ObjectId, ObjectOrPlayerId, ObjectRef, PlayerId};
use crate::types::effects::{AffectedSet, EntersAttacking, ExtraPhasesAfter};
use crate::types::replacement::{EntersAttackingCandidates, EventPattern, ReplacementDef, Rewrite};
use crate::engine::returns::ReturningObject;
use crate::engine::triggers::binding_reads::TriggerBindingReads;
use crate::types::triggers::{
    CreatingResolution, DelayedProvenance, DelayedTurn, Lki, Referred, RememberedObject, TriggerEvent,
    TriggerTurn,
};
use crate::ui::choice_types::ChoiceKind;
use crate::ui::decision::DecisionProvider;

/// Context passed through effect resolution.
///
/// Tracks the source of the spell/ability, its controller, and resolved
/// targets so that each `Primitive` knows what it's acting on.
#[derive(Debug, Clone)]
pub struct ResolutionContext {
    /// The resolving stack object — the spell, or for an ability the
    /// ephemeral object CR 608.2n deletes at the end of resolution. What a
    /// prompt or a record names as the thing resolving; what an effect names
    /// as its source is [`Self::effect_source`].
    pub source: ObjectId,
    /// CR 113.7a — for an activated or triggered ability, the permanent whose
    /// ability is resolving, and which existence of it (CR 400.7); `None` for
    /// a spell (whose source is `source`) and for a CR 615.5 rider. "This
    /// permanent" in an ability's text reads this: `Primitive::Attach` attaches
    /// it and `EffectRecipient::ThisObject` finds it.
    pub ability_source: Option<ObjectRef>,
    /// The player who controls the spell/ability
    pub controller: PlayerId,
    /// What generated the effect this resolution is (CR 113.3, 122.1): what
    /// "triggered abilities you control can't cause" and "a white instant or
    /// sorcery spell causes" read off its cause.
    pub generated_by: GeneratedBy,
    /// What each instance of "target" holds **after** CR 608.2b's re-check —
    /// the survivors, not the announcement. An instance every one of whose
    /// targets went illegal is empty here, and the atom that reads it does
    /// nothing, which is 608.2b's "it won't affect that target".
    pub targets: ChosenTargets,
    /// The event a CR 615.5 rider's replacement or prevention effect applied
    /// to, for a rider and for nothing else.
    ///
    /// `None` on every other resolution, which is what makes
    /// `AmountExpr::ReplacedAmount` and `DamagePrevented` refuse rather than
    /// read a wrong number: a resolving spell has no replaced event, so the
    /// question has no answer rather than a default one.
    pub replaced_event: Option<ReplacedEvent>,
    /// A triggered ability's bound facts, for the resolution of one and for
    /// nothing else (`triggers-architecture.md` §6.3): `Triggering`,
    /// `TriggeringPlayer` and `TriggeringAmount` read it, and refuse outside
    /// a trigger rather than reading a default.
    pub trigger: Option<crate::types::triggers::TriggerBinding>,
}

/// What a CR 615.5 rider may refer to of the event its replacement or
/// prevention effect applied to, captured as the rider is queued.
///
/// The amount is taken from the event as the loop sees it, so a rider queued
/// after a doubler reads the doubled number, and is captured rather than
/// recomputed after the event, which Angel of Suffering's ruling that
/// unpreventable damage still mills needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacedEvent {
    /// Damage: what the event carried, "that much", and what this application
    /// prevented, "the damage prevented this way" (`AmountExpr::DamagePrevented`).
    ///
    /// The two differ under a partial shield, and under CR 615.12's
    /// unpreventable damage, where `prevented` is 0: Angel of Suffering mills
    /// `amount`, and Reverse Damage gains `prevented`.
    Damage { amount: u64, prevented: u64 },
    /// Any other event that carries a number (`pipeline::event_amount`): life
    /// gained or lost, cards drawn, tokens created, a scry, mana produced.
    Amount(u64),
    /// An event with no number: a zone change, a player losing the game.
    NoAmount,
}

impl ResolutionContext {
    /// A resolution that names nothing — no targets, no replaced event, no
    /// trigger, and no permanent distinct from `source`. What a mana ability's
    /// resolution uses.
    pub fn untargeted(source: ObjectId, controller: PlayerId, generated_by: GeneratedBy) -> Self {
        ResolutionContext {
            source,
            ability_source: None,
            controller,
            generated_by,
            targets: ChosenTargets::NONE,
            replaced_event: None,
            trigger: None,
        }
    }

    /// A mana ability's resolution, activated or triggered (CR 605.3b, 605.4a):
    /// with no stack object, the permanent is both `source` and CR 113.7a's
    /// ability source, and it has no targets (CR 605.1a).
    pub(crate) fn for_mana_ability(
        game: &GameState,
        source: ObjectId,
        controller: PlayerId,
        generated_by: GeneratedBy,
    ) -> Self {
        let mut resolution = ResolutionContext::untargeted(source, controller, generated_by);
        resolution.ability_source = game.object_ref(source);
        resolution
    }

    /// The object this resolution's effects are attributed to: the spell, or
    /// the object that has the ability (CR 113.7), which deals the damage and
    /// gains the life (CR 120.2b, 608.2h) after its stack object is gone. A
    /// rider's is the object whose replacement it is.
    pub fn effect_source(&self) -> ObjectId {
        self.ability_source.map_or(self.source, |r| r.id)
    }

    /// Which resolution this is, as its effect's records name it.
    pub fn stamp(&self) -> ResolutionStamp {
        ResolutionStamp { source: self.source, controller: self.controller }
    }

    /// What every event this resolution proposes is caused by: its effect.
    pub fn cause(&self) -> EventCause {
        EventCause::Effect { resolution: self.stamp(), generated_by: self.generated_by }
    }
}

/// One resolution's walk over its effect tree: its place among the instances
/// of "target" the effect declares (the glossary's *cursor*, sense 2), and
/// CR 118.12's answer for the clause after an action. Each resolution makes a
/// fresh one, so a rider never reads its parent's answer
/// (`triggers-architecture.md` §6.2).
#[derive(Debug)]
struct ResolutionWalk {
    instance_cursor: usize,
    last_cost_answer: Option<CostAnswer>,
    /// The extra turn this resolution's last `ExtraTurn` scheduled: what
    /// "that turn" names (Final Fortune; CR 500.7).
    extra_turn: Option<ExtraTurnId>,
    /// While `Effect::Remember`'s instruction runs, the objects and players
    /// its atom was handed.
    remembering: Option<Vec<ObjectOrPlayerId>>,
    /// CR 603.7c — what `Effect::Remember`'s instruction acted on, which a
    /// delayed trigger created after it refers to.
    remembered: Option<Referred>,
    /// The records of the last instruction that took an action, from its
    /// first to the one after its last: CR 603.12's "that action", which a
    /// reflexive trigger made after it asks about.
    last_action: Option<(EventId, EventId)>,
}

impl ResolutionWalk {
    /// Before the effect's first atom: no instance used, no action taken.
    fn start() -> Self {
        ResolutionWalk {
            instance_cursor: 0,
            last_cost_answer: None,
            extra_turn: None,
            remembering: None,
            remembered: None,
            last_action: None,
        }
    }
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
        let mut walk = ResolutionWalk::start();
        self.resolve_effect_at(effect, ctx, dp, DeclaredInstances::Effect(effect), &mut walk)
    }

    /// [`Self::resolve_effect`] for a spell or ability leaving the stack.
    ///
    /// `announced` is the entry's CR 601.2c record, so an atom finds its own
    /// targets by the index the announcement filled and nothing is derived
    /// from the tree — see `DeclaredInstances`.
    ///
    /// Returns CR 118.12's last answer, which CR 603.2h's writer reads: an
    /// action declined is not an action taken.
    pub fn resolve_effect_with_announced_targets(
        &mut self,
        effect: &Effect,
        announced: &[TargetInstance],
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
    ) -> Result<Option<CostAnswer>, String> {
        let mut walk = ResolutionWalk::start();
        self.resolve_effect_at(effect, ctx, dp, DeclaredInstances::Announced(announced), &mut walk)?;
        Ok(walk.last_cost_answer)
    }

    /// [`Self::resolve_effect`]'s body, carrying the walk, which keeps the
    /// records of each instruction that takes an action: an atom, or a "you
    /// may" around one, declined or not. Making a delayed trigger takes none.
    fn resolve_effect_at(
        &mut self,
        effect: &Effect,
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
        declared: DeclaredInstances<'_>,
        walk: &mut ResolutionWalk,
    ) -> Result<(), String> {
        let takes_an_action = match effect {
            Effect::Atom(Primitive::CreateDelayedTrigger(_), _) => false,
            Effect::Atom(..) | Effect::Optional { .. } => true,
            Effect::Sequence(_)
            | Effect::Replacement(_)
            | Effect::Restriction(_)
            | Effect::CostModification(_)
            | Effect::TriggerMultiplier(_)
            | Effect::Conditional(..)
            | Effect::FunctionsIn(..)
            | Effect::Triggered(_)
            | Effect::Remember(_)
            | Effect::Modal { .. }
            | Effect::ForEach(..)
            | Effect::Repeat(..) => false,
        };
        let from = self.events.next_seq();
        let result = self.resolve_effect_step(effect, ctx, dp, declared, walk);
        if takes_an_action {
            walk.last_action = Some((from, self.events.next_seq()));
        }
        result
    }

    /// One instruction of [`Self::resolve_effect_at`]'s walk.
    ///
    /// **The one place `ctx.targets` is indexed.** Each atom is handed its own
    /// instance as a flat slice, so no primitive can reach a neighboring
    /// instance's targets, and an atom whose instance fizzled is handed an
    /// empty one rather than the spell's first (CR 608.2b).
    fn resolve_effect_step(
        &mut self,
        effect: &Effect,
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
        declared: DeclaredInstances<'_>,
        walk: &mut ResolutionWalk,
    ) -> Result<(), String> {
        // Every atom takes its action, which is CR 118.12's "started to pay"
        // for the clause after it.
        if let Effect::Atom(..) = effect {
            walk.last_cost_answer = Some(CostAnswer::Does);
        }
        match effect {
            // "That creature", "that player": not an instance of "target" but
            // a fact bound at dispatch, handed to the primitive as the target
            // slice it already reads — empty when CR 603.6 finds nothing.
            Effect::Atom(
                primitive,
                recipient @ (EffectRecipient::Triggering(_)
                | EffectRecipient::TriggeringPlayer
                | EffectRecipient::Referred),
            ) => {
                let bound = self.bound_targets(recipient, ctx)?;
                self.resolve_primitive(primitive, recipient, &bound, ctx, dp, walk)
            }
            // "The player or planeswalker it's attacking", "defending player":
            // read as the instruction applies. Once its creature attacks
            // nothing, there is nothing to act on, and the instruction does
            // nothing rather than reaching a primitive's default player.
            Effect::Atom(primitive, recipient @ EffectRecipient::AttackedBy(creature)) => {
                let attacked = self.attacked_by(*creature, ctx);
                if attacked.is_empty() {
                    return Ok(());
                }
                self.resolve_primitive(primitive, recipient, &attacked, ctx, dp, walk)
            }
            Effect::Atom(primitive, recipient @ EffectRecipient::DefendingPlayer(creature)) => {
                let Some(defending) = self.defending_player_of(*creature, ctx) else { return Ok(()) };
                self.resolve_primitive(primitive, recipient, &[ObjectOrPlayerId::Player(defending)], ctx, dp, walk)
            }
            // "Each player", "you and that player": every player the group names,
            // in APNAP order (CR 101.4), handed to the primitive as its players.
            // Only a primitive that says what several players at once means takes it.
            Effect::Atom(primitive, recipient @ EffectRecipient::EachOf(group)) => {
                // A draw is one instruction per player (CR 121.2c); damage to
                // them, or a loss of life, is one event, a member each.
                if !matches!(
                    primitive,
                    Primitive::DrawCards(_) | Primitive::DealDamage { .. } | Primitive::LoseLife(_)
                ) {
                    return Err(format!(
                        "{:?} on {:?} is not built for {:?}; a draw, damage and a life loss are",
                        recipient, ctx.source, primitive
                    ));
                }
                let players: Vec<ObjectOrPlayerId> =
                    self.players_in(group, ctx).into_iter().map(ObjectOrPlayerId::Player).collect();
                self.resolve_primitive(primitive, recipient, &players, ctx, dp, walk)
            }
            // "That creature and this creature": each member found as it would
            // be alone, and every object they find handed to the primitive at
            // once, so it acts on them in one instruction (CR 608.2f).
            Effect::Atom(primitive, recipient @ EffectRecipient::Group(members)) => {
                let mut found: Vec<ObjectOrPlayerId> = Vec::new();
                for member in members {
                    for target in self.resolve_group_member(member, ctx)? {
                        if !found.contains(&target) {
                            found.push(target);
                        }
                    }
                }
                self.resolve_primitive(primitive, recipient, &found, ctx, dp, walk)
            }
            // "This creature": no instance of "target" either (CR 113.7a), and
            // empty when the source is no longer the object the ability is of.
            Effect::Atom(primitive, recipient @ EffectRecipient::ThisObject) => {
                let this: Vec<ObjectOrPlayerId> =
                    self.this_object(ctx).map(ObjectOrPlayerId::Object).into_iter().collect();
                self.resolve_primitive(primitive, recipient, &this, ctx, dp, walk)
            }
            // "Each [permanent]": every permanent the filter matches as the
            // instruction applies (CR 611.2c's set), handed to the primitive as
            // the objects it acts on in one instruction (CR 608.2f), so a
            // primitive that reads only its objects reaches them too.
            Effect::Atom(primitive, recipient @ EffectRecipient::FilteredPermanents(_)) => {
                let each: Vec<ObjectOrPlayerId> =
                    self.affected_permanents(recipient, &[], ctx).into_iter().map(ObjectOrPlayerId::Object).collect();
                self.resolve_primitive(primitive, recipient, &each, ctx, dp, walk)
            }
            // CR 608.2d — objects a player chooses as the effect applies,
            // checked against the verb's own event (CR 101.2), so a "can't"
            // removes a candidate rather than refusing a choice.
            Effect::Atom(primitive, recipient @ EffectRecipient::ChosenBy(choice)) => {
                let chosen = self.choose_as_it_applies(primitive, choice, declared, walk, ctx, dp)?;
                self.resolve_primitive(primitive, recipient, &chosen, ctx, dp, walk)
            }
            Effect::Atom(primitive, recipient) => {
                match instance_of(recipient, declared, &mut walk.instance_cursor) {
                    Some((ix, clause)) => {
                        self.resolve_primitive(primitive, clause, ctx.targets.instance(ix), ctx, dp, walk)
                    }
                    // `Instance(ix)` naming a clause that does not exist: a
                    // card-authoring error, loud rather than silently
                    // targetless. `cards::registry`'s own test refuses one at
                    // registration, so this is the belt to that's braces.
                    None if matches!(recipient, EffectRecipient::SameInstanceAs(_)) => Err(format!(
                        "{:?} on {:?} names an instance of \"target\" the effect never declared (CR 115.3)",
                        recipient, ctx.source
                    )),
                    // Implicit, Controller, the other filtered sweeps, Host —
                    // none reads a chosen target.
                    None => self.resolve_primitive(primitive, recipient, &[], ctx, dp, walk),
                }
            }

            Effect::Sequence(effects) => {
                for sub in effects {
                    self.resolve_effect_at(sub, ctx, dp, declared, walk)?;
                }
                Ok(())
            }

            // CR 614.1a. A static replacement ability is `gather`'s, so this is one a
            // resolution creates, which needs the duration CR 614.3 gives it: loud,
            // since `CreateReplacement` and `Regenerate` are the arms that carry one.
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
            // "can't" is `is_prohibited`'s, and a resolution's needs the scope CR
            // 608.2c leaves to a reader (`cant-effects-architecture.md` §9 finding 1).
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

            // CR 603.2d's: a multiplier is read off the effective list as the
            // dispatch queues, and never resolves.
            Effect::TriggerMultiplier(_) => Err(
                "a trigger multiplier is a static ability's, read off its source's \
                 effective ability list as the dispatch queues what a window \
                 triggered; it does not resolve. Put it on an `AbilityType::Static` \
                 ability."
                    .to_string(),
            ),

            // CR 113.6b's statement wraps an ability's whole effect and never
            // resolves; a trigger's binding carries it for the re-check before.
            Effect::FunctionsIn(..) => Err(
                "`Effect::FunctionsIn` states where an ability functions, at its effect's root; it does not resolve, and a triggered ability's binding carries it for the resolution's check before its effect."
                    .to_string(),
            ),

            // An "if" inside a resolving effect, read as the atom is reached (CR
            // 608.2c's order); CR 603.4's intervening "if" is the def's field,
            // `resolve_taken`'s. "If you do … if you don't …" reads one answer.
            Effect::Conditional(condition, inner) => {
                if self.resolution_condition_holds(condition, ctx, walk) {
                    let answer = walk.last_cost_answer;
                    self.resolve_effect_at(inner, ctx, dp, declared, walk)?;
                    walk.last_cost_answer = answer;
                }
                Ok(())
            }

            // What reaches the stack is a trigger's inner effect with its binding,
            // and a spell cannot carry a triggered ability (CR 603.1), so reaching
            // here is a wiring error.
            Effect::Triggered(_) => Err(format!(
                "a triggered ability reached resolution as an effect of {:?}; `Effect::Triggered` belongs on an `AbilityType::Triggered` ability and is read by `engine::triggers`",
                ctx.source
            )),

            // CR 603.5 — "you may" is chosen as the effect resolves. The answer
            // is the choice: declined is `Doesnt`, and taken is the action's
            // own, with a `Cant` becoming `Doesnt`, since CR 118.3 lets no
            // player pay a cost they can't.
            Effect::Optional { chooser, effect: inner } => {
                let chooser = self.player_the_instruction_names(chooser, &[], ctx, dp)?;
                let kind = ChoiceKind::ApplyOptionalEffect { source: ctx.source };
                if !crate::ui::ask::ask_take_or_leave(dp, self, chooser, ctx.source, kind, "apply_optional_effect") {
                    // Its targets were announced all the same, so the instances
                    // after it are read past them.
                    walk.instance_cursor += inner.instances().len();
                    walk.last_cost_answer = Some(CostAnswer::Doesnt);
                    return Ok(());
                }
                walk.last_cost_answer = None;
                self.resolve_effect_at(inner, ctx, dp, declared, walk)?;
                walk.last_cost_answer = Some(match walk.last_cost_answer {
                    Some(CostAnswer::Cant | CostAnswer::Doesnt) => CostAnswer::Doesnt,
                    Some(CostAnswer::Does) | None => CostAnswer::Does,
                });
                Ok(())
            }

            // CR 603.7c — what this instruction acts on is "that card" for a
            // delayed trigger created after it, read back off the records it
            // performed once it has.
            Effect::Remember(inner) => {
                let Effect::Atom(primitive, _) = inner.as_ref() else {
                    return Err(format!(
                        "`Effect::Remember` on {:?} marks {:?}, which is not one instruction",
                        ctx.source, inner
                    ));
                };
                if walk.remembered.is_some() {
                    return Err(format!("{:?} remembers a second instruction; a resolution remembers one", ctx.source));
                }
                let mark = self.events.next_seq();
                walk.remembering = Some(Vec::new());
                let result = self.resolve_effect_at(inner, ctx, dp, declared, walk);
                let acted_on = walk.remembering.take().unwrap_or_default();
                result?;
                walk.remembered = Some(self.remembered_since(primitive, &acted_on, ctx, mark));
                Ok(())
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
        targets: &[ObjectOrPlayerId],
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
        walk: &mut ResolutionWalk,
    ) -> Result<(), String> {
        // Every mutation a primitive proposes belongs to *this* resolution
        // (CR 614.15 / the resolution stamp on each emitted event).
        let actx = ActionContext::resolving(dp, ctx);
        if let Some(acted_on) = walk.remembering.as_mut() {
            acted_on.extend_from_slice(targets);
        }
        match primitive {
            // === One-shot primitives ===

            // **One batch, however many things it hits**: CR 615.7's "two or more
            // applicable sources" and CR 603.2c's "one or more" read the batch
            // (`CLAUDE.md`). `FilteredPermanents` is "every permanent matching this
            // **now**", resolved here and ordered for the CR 616.1 prompt and the log.
            Primitive::DealDamage { amount: amount_expr, unpreventable } => {
                let amount = self.evaluate_amount(amount_expr, ctx, None)?;
                let targets: Vec<ObjectOrPlayerId> = match recipient {
                    // CR 120.1 — damage is dealt to permanents and players, so
                    // a zone-reaching recipient has nothing to be dealt to.
                    EffectRecipient::FilteredObjectsIn(..) => {
                        return Err(format!(
                            "a `Primitive::DealDamage` on {:?} has a zone-reaching recipient (CR 120.1)",
                            ctx.source
                        ));
                    }
                    EffectRecipient::FilteredPermanents(filter) => self
                        .battlefield_ids_ordered()
                        .into_iter()
                        .filter(|&id| self.matches_for_effect(id, filter, ctx))
                        .map(ObjectOrPlayerId::Object)
                        .collect(),
                    // "Deals 2 damage to you": Char, a painland.
                    EffectRecipient::Controller => vec![ObjectOrPlayerId::Player(ctx.controller)],
                    EffectRecipient::Implicit => {
                        return Err(format!(
                            "a `Primitive::DealDamage` on {:?} names no recipient; damage is dealt to \
                             a permanent or a player (CR 120.1), and \"you\" is `Controller`",
                            ctx.source
                        ));
                    }
                    // Refused at the atom until its card: once the Aura has
                    // left, "enchanted creature" is what it last enchanted
                    // (CR 608.2h), which the live attachment no longer says.
                    EffectRecipient::Host => {
                        return Err(format!("a `Host` recipient on a `Primitive::DealDamage` on {:?} is not built", ctx.source));
                    }
                    EffectRecipient::SameInstanceAs(_) => return Err(back_reference(recipient, ctx)),
                    // Each of these was found before the primitive, into
                    // `targets`.
                    EffectRecipient::ThisObject
                    | EffectRecipient::Target(..)
                    | EffectRecipient::Choose(..)
                    | EffectRecipient::Triggering(_)
                    | EffectRecipient::TriggeringPlayer
                    | EffectRecipient::Referred
                    | EffectRecipient::AttackedBy(_)
                    | EffectRecipient::DefendingPlayer(_)
                    | EffectRecipient::EachOf(_)
                    | EffectRecipient::Group(_)
                    | EffectRecipient::ChosenBy(_) => targets.to_vec(),
                };
                if targets.is_empty() {
                    return Ok(());
                }
                let source_lki = self.source_lki(ctx);
                self.execute_actions(
                    targets
                        .into_iter()
                        // CR 615.12's per-event flag, set by the effect that proposes the damage
                        // and carried onto every member: Pinpoint Avalanche's "can't be
                        // prevented" is about this resolution's damage and nothing else.
                        .map(|target| GameAction::DealDamage {
                            source: ctx.effect_source(),
                            source_lki: source_lki.clone(),
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
                let count = self.evaluate_amount(amount_expr, ctx, None)?;
                // **One instruction per player, whatever `count` is** (CR 121.2a;
                // Alms Collector's ruling counts the word "draw"), its performer
                // doing the draws. Players draw in `players_in`'s APNAP order (CR 121.2c).
                let players = self.each_player_acted_on(recipient, targets, ctx);
                for player in players {
                    self.execute_action(
                        GameAction::DrawCards { player, n: count },
                        &actx,
                    )?;
                }
                Ok(())
            }

            // CR 701.17a — **one batch, N members**: "puts that many cards" is one
            // move, so a "one or more cards" trigger fires once (CR 603.2c) and each
            // card is its own event for Leyline of the Void (CR 614.5). A short
            // library mills what it has (CR 701.17b), every card taken before any moves.
            Primitive::Mill(amount_expr) => {
                let count = self.evaluate_amount(amount_expr, ctx, None)? as usize;
                let player_id = self.resolve_player_for_self(recipient, targets, ctx);
                let batch: Vec<GameAction> = self
                    .get_player(player_id)?
                    .top_of_library(count)
                    .map(|object| GameAction::ZoneChange {
                        object,
                        from: Zone::Library,
                        to: Zone::Graveyard,
                        act: ZoneChangeAct::Milled,
                    })
                    .collect();
                if batch.is_empty() {
                    return Ok(());
                }
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // CR 121.5 — "put the top N cards of your library into your hand" is
            // not a draw. It is built like `Mill`, with the hand for the
            // graveyard: one batch, the cards taken before any moves, so a short
            // library gives as many as there are and an empty one gives nothing.
            Primitive::PutTopCardsIntoHand(amount_expr) => {
                let count = self.evaluate_amount(amount_expr, ctx, None)? as usize;
                let player_id = self.resolve_player_for_self(recipient, targets, ctx);
                let batch: Vec<GameAction> = self
                    .get_player(player_id)?
                    .top_of_library(count)
                    .map(|object| GameAction::ZoneChange {
                        object,
                        from: Zone::Library,
                        to: Zone::Hand,
                        act: ZoneChangeAct::Put,
                    })
                    .collect();
                if batch.is_empty() {
                    return Ok(());
                }
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // CR 701.9a — **one batch, N members**, as `Mill`: chosen, then moved
            // together, so "one or more cards" fires once for a Mind Rot, and each
            // card is a CR 616.1 decision of its own (Library of Leng's ruling). CR
            // 701.9b's chooser: `Affected` asks the player, `AtRandom` reads `rng`.
            Primitive::Discard(amount_expr, chooser) => {
                let count = self.evaluate_amount(amount_expr, ctx, None)? as usize;
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
                        act: ZoneChangeAct::Discarded,
                    })
                    .collect();
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // CR 701.22 — the instruction, and the performer does the rest.
            Primitive::Scry(amount_expr) => {
                let n = self.evaluate_amount(amount_expr, ctx, None)?;
                let player_id = self.resolve_player_for_self(recipient, targets, ctx);
                self.execute_action(GameAction::Scry { player: player_id, n }, &actx)?;
                Ok(())
            }

            Primitive::GainLife(amount_expr) => {
                let amount = self.evaluate_amount(amount_expr, ctx, None)?;
                let player_id = self.resolve_player_for_self(recipient, targets, ctx);
                self.execute_action(GameAction::GainLife {
                    player: player_id,
                    amount,
                    source: ctx.effect_source(),
                }, &actx)?;
                Ok(())
            }

            // "Each opponent loses 1 life" is one event, a member each, in the
            // APNAP order `players_in` gave them.
            Primitive::LoseLife(amount_expr) => {
                let amount = self.evaluate_amount(amount_expr, ctx, None)?;
                let players = self.each_player_acted_on(recipient, targets, ctx);
                let batch = players
                    .into_iter()
                    .map(|player| GameAction::LoseLife { player, amount, damage_source: None })
                    .collect();
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // CR 119.5 — "gains or loses the necessary amount of life". A proposal of
            // whichever one it is, never a write to the total: Exquisite Archangel's
            // "-4 becomes 20 is a 24-life gain" is what a Rhox Faithmender doubles and
            // a Skullcrack refuses. Equal totals propose nothing; a 0 gain is a
            // non-event (CR 119.10) and a 0 loss a no-op.
            Primitive::SetLifeTotal(amount_expr) => {
                let target = self.evaluate_amount(amount_expr, ctx, None)? as i64;
                let player = self.resolve_player_for_self(recipient, targets, ctx);
                let current = self.get_player(player)?.life_total;
                if target > current {
                    self.execute_action(GameAction::GainLife {
                        player,
                        amount: (target - current) as u64,
                        source: ctx.effect_source(),
                    }, &actx)?;
                } else if target < current {
                    self.execute_action(GameAction::LoseLife {
                        player,
                        amount: (current - target) as u64,
                        damage_source: None,
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
                        GameAction::PlayerLoses { player },
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
                if let EffectRecipient::Implicit = recipient {
                    return Err(format!(
                        "a `Primitive::Exile` on {:?} names nothing to exile; use `ThisObject` or a target",
                        ctx.source
                    ));
                }
                let batch = self.events_for(primitive, targets, ctx)?;
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // CR 610.3 — the exile, unless its event has already happened
            // since the spell was cast or the ability triggered (610.3a, b).
            // Then the return it waits to make, of what it exiled.
            Primitive::ExileUntil { until, refers_to, under } => {
                let referred: Vec<ObjectRef> = match refers_to {
                    None => Vec::new(),
                    Some(_) => {
                        let ix = walk.instance_cursor;
                        walk.instance_cursor += 1;
                        ctx.targets
                            .instance(ix)
                            .iter()
                            .filter_map(|t| match t {
                                ObjectOrPlayerId::Object(id) => self.object_ref(*id),
                                ObjectOrPlayerId::Player(_) => None,
                            })
                            .collect()
                    }
                };
                let spell = || self.object_ref(ctx.source).ok_or_else(|| format!("{} is not in the game", ctx.source));
                let source = match ctx.ability_source {
                    Some(source) => source,
                    None => spell()?,
                };
                if self.until_has_happened(until, ctx.ability_source, &referred)? {
                    return Ok(());
                }
                let mark = self.events.next_seq();
                let batch = self.events_for(&Primitive::Exile, targets, ctx)?;
                if batch.is_empty() {
                    return Ok(());
                }
                self.execute_actions(batch, &actx)?;
                let returns = self.exiled_since(ctx.stamp(), mark);
                self.wait_to_return(until.as_ref().clone(), source, &referred, returns, *under, ctx.controller, ctx.cause())?;
                Ok(())
            }

            // CR 500.7 — scheduling, not a mutation: the extra turn becomes a
            // `GameAction::BeginTurn` proposal when the drainer reaches it, which is
            // what a skip replaces (CR 614.10a). Pushed, because "the most recently
            // created turn will be taken first".
            Primitive::ExtraTurn => {
                let player = self.resolve_player_for_self(recipient, targets, ctx);
                walk.extra_turn = Some(self.schedule_extra_turn(player));
                Ok(())
            }

            // CR 603.7, registered as the resolution reaches it. The source is CR
            // 603.7d–f's: an ability's source (603.7e), else the spell (603.7d) or the
            // object whose replacement a rider is (603.7f) — never an ability's stack
            // object, which CR 608.2n removes. X is CR 107.3n's. A reflexive one
            // (CR 603.12) asks about "that action", the walk's last.
            Primitive::CreateDelayedTrigger(template) => {
                let resolving = self.resolving_for(ctx);
                let resolution =
                    resolving.map(|_| CreatingResolution { stamp: ctx.stamp(), that_action: walk.last_action });
                let source = match ctx.ability_source {
                    Some(source) => source,
                    None => self
                        .object_ref(ctx.source)
                        .ok_or_else(|| format!("the source {} of a delayed triggered ability is not in the game", ctx.source))?,
                };
                let turn = match template.turn {
                    DelayedTurn::NoneNamed => TriggerTurn::Any,
                    DelayedTurn::NextTurn => TriggerTurn::LaterThan(self.turn_number),
                    DelayedTurn::ThatExtraTurn => TriggerTurn::Extra(walk.extra_turn.ok_or_else(|| {
                        format!(
                            "a delayed trigger on {} names \"that turn\", and its resolution scheduled no extra turn before it",
                            ctx.source
                        )
                    })?),
                };
                let provenance = DelayedProvenance {
                    source,
                    // The resolving object's card: the spell's, or for an
                    // ability the source's, which its stack object was built
                    // from.
                    source_card: std::sync::Arc::clone(&self.get_object(ctx.source)?.card_data),
                    controller: ctx.controller,
                    created_by: resolving.and_then(|r| r.identity),
                    x_value: resolving.and_then(|r| r.x_value),
                    turn,
                    resolution,
                    cause: ctx.cause(),
                };
                // CR 603.7c — its "that card" is what this resolution
                // remembered; a template that names one with nothing
                // remembered is a card missing its `Effect::Remember`.
                let names_referred_object = template.def.trigger_binding_reads().contains(TriggerBindingReads::REFERRED)
                    || template.def.condition.events().iter().any(TriggerEvent::watches_referred);
                if names_referred_object && walk.remembered.is_none() {
                    return Err(format!(
                        "a delayed trigger on {} refers to an object its resolution never remembered (`Effect::Remember`)",
                        ctx.source
                    ));
                }
                self.create_delayed_trigger(template, provenance, walk.remembered.clone().unwrap_or_default())?;
                Ok(())
            }

            // CR 500.8 — scheduling, like the extra turn: the phases become
            // `GameAction::BeginPhase` proposals when the drainer reaches them,
            // inserted at `cursor + 1` for "directly after the specified phase". A
            // resolution outside a phase splices nothing: the cursor is `None` only
            // between a turn beginning and its first phase, when nothing can be
            // resolving, and a `Vec` insert past its end would panic.
            Primitive::ExtraPhases { phases, after } => {
                let Some(cursor) = self.turn_plan.cursor else {
                    return Ok(());
                };
                let in_a_main_phase = matches!(self.phase.phase_type, PhaseType::Precombat | PhaseType::Postcombat);
                if *after == ExtraPhasesAfter::ThisMainPhase && !in_a_main_phase {
                    return Ok(());
                }
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
                let producer = crate::engine::mana::ManaProducer { id: ctx.effect_source(), tapped_for_mana: false, lki: None };
                self.produce_mana(output, ctx.controller, producer, ctx, &actx)
            }

            Primitive::CounterSpell => {
                for target in targets {
                    if let ObjectOrPlayerId::Object(id) = target {
                        self.counter_spell(*id, ctx, &actx)?;
                    }
                }
                Ok(())
            }

            Primitive::CounterAbility => {
                for target in targets {
                    if let ObjectOrPlayerId::Object(id) = target {
                        self.counter_ability(*id, ctx, &actx)?;
                    }
                }
                Ok(())
            }

            // CR 701.6a — "counter that spell or ability", by what it is.
            Primitive::Counter => {
                for target in targets {
                    if let ObjectOrPlayerId::Object(id) = target {
                        if self.is_spell_on_stack(*id) {
                            self.counter_spell(*id, ctx, &actx)?;
                        } else {
                            self.counter_ability(*id, ctx, &actx)?;
                        }
                    }
                }
                Ok(())
            }

            // === Destroy and untap ===

            Primitive::Destroy => {
                // CR 701.8a. **One batch** (CR 608.2f): a wipe's deaths are one event,
                // which CR 615.7's allocation and CR 603.2c's "one or more" read.
                // Indestructible is a "can't" (CR 702.12b, 614.17) that `is_prohibited`
                // asks ahead of the pipeline, so a self-replacement still sees the
                // proposal (614.17c). Off the battlefield, destroy does nothing (701.8b).
                let batch = self.events_for(primitive, targets, ctx)?;
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            Primitive::Attach => {
                // "Attach this permanent to target ..." (CR 702.6a). The
                // attachment is the ability's source; a spell has none to attach.
                if ctx.ability_source.is_none() {
                    return Err("Primitive::Attach resolved from a spell: only an ability has a permanent to attach".to_string());
                }
                // CR 608.2b partial resolution: the performer is loud, so the caller
                // checks. The Equipment may have left after activation (CR 301.5b: nothing
                // happens), by identity: one that left and came back is a new object
                // the ability does not know (CR 400.7). The target's legality was
                // re-checked against the recipient before `resolve_effect`, which is CR
                // 701.3b's "doesn't move".
                let Some(attachment) = self.this_object(ctx).filter(|id| self.battlefield.contains_key(id)) else {
                    return Ok(());
                };
                for target in targets {
                    if let ObjectOrPlayerId::Object(host) = target {
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
                // CR 701.26b. `FilteredPermanents` is resolved now, as `DealDamage`'s
                // is, and ordered: stun counters (CR 122.1d) make two effects want one
                // untap, so the members reach a CR 616.1 prompt.
                let ids: Vec<ObjectId> = match recipient {
                    EffectRecipient::FilteredPermanents(filter) => self
                        .battlefield_ids_ordered()
                        .into_iter()
                        .filter(|&id| self.matches_for_effect(id, filter, ctx))
                        .collect(),
                    // CR 608.2b: a spell whose targets are not *all* illegal still resolves
                    // and does as much as it can, so a target that has left is skipped. The
                    // performer is loud, which makes checking here the caller's job — the
                    // shape `Primitive::Destroy` above has.
                    _ => targets
                        .iter()
                        .filter_map(|t| match t {
                            ObjectOrPlayerId::Object(id) if self.battlefield.contains_key(id) => {
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
                let objects: Vec<ObjectOrPlayerId> = ids.into_iter().map(ObjectOrPlayerId::Object).collect();
                let batch = self.events_for(primitive, &objects, ctx)?;
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // === Continuous effect primitives ===

            Primitive::ModifyPowerToughness(power_expr, toughness_expr, duration) => {
                let power = self.evaluate_amount(power_expr, ctx, None)? as i32;
                let toughness = self.evaluate_amount(toughness_expr, ctx, None)? as i32;
                // CR 608.2h — a resolving spell locks its value in as it
                // resolves, so this is `Fixed` even though the card text
                // said "X". Static abilities are the ones that stay live.
                self.add_resolution_effect(
                    ctx,
                    &actx,
                    Layer::Layer7cModifyPT,
                    *duration,
                    self.affected_permanents(recipient, targets, ctx),
                    EffectModification::ModifyPowerToughness { power: PtValue::Fixed(power), toughness: PtValue::Fixed(toughness) },
                )?;
                Ok(())
            }

            Primitive::SetPowerToughness(power_expr, toughness_expr, duration) => {
                let power = self.evaluate_amount(power_expr, ctx, None)? as i32;
                let toughness = self.evaluate_amount(toughness_expr, ctx, None)? as i32;
                self.add_resolution_effect(
                    ctx,
                    &actx,
                    Layer::Layer7bSetPT,
                    *duration,
                    self.affected_permanents(recipient, targets, ctx),
                    EffectModification::SetPowerToughness { power: PtValue::Fixed(power), toughness: PtValue::Fixed(toughness) },
                )?;
                Ok(())
            }

            // === Copy effects (CR 707, layer 1a) ===
            Primitive::Copy { roles, except, duration } => {
                self.apply_copy(roles, except, *duration, targets, ctx, dp)
            }

            Primitive::SwitchPowerToughness(duration) => {
                self.add_resolution_effect(
                    ctx,
                    &actx,
                    Layer::Layer7dSwitchPT,
                    *duration,
                    self.affected_permanents(recipient, targets, ctx),
                    EffectModification::SwitchPowerToughness,
                )?;
                Ok(())
            }

            // === Layer 5: color-changing effects ===

            Primitive::ChangeColor(color_change, duration) => {
                use crate::types::effects::ColorChange;
                let modification = match color_change {
                    ColorChange::Add(c) => EffectModification::AddColor(*c),
                    ColorChange::Set(colors) => EffectModification::SetColors(colors.clone()),
                    ColorChange::RemoveAll => EffectModification::RemoveAllColors,
                };
                self.add_resolution_effect(
                    ctx,
                    &actx,
                    Layer::Layer5Color,
                    *duration,
                    self.affected_permanents(recipient, targets, ctx),
                    modification,
                )?;
                Ok(())
            }

            // === Layer 4: type-changing effects ===

            Primitive::ChangeType(type_change, duration) => {
                let affected = self.affected_permanents(recipient, targets, ctx);
                if affected.is_empty() || self.never_starts(*duration, ctx) {
                    return Ok(());
                }
                // A TypeChange may produce several Layer 4 rows: CR 613.6 makes them
                // siblings ("the parts of the effect each apply in their appropriate
                // layers") and CR 613.7b makes them share a timestamp — one resolution,
                // one moment of creation.
                self.settle_before_a_resolution_effect(&actx)?;
                let timestamp = self.allocate_timestamp();
                for modification in type_change.modifications() {
                    self.add_resolution_row(ctx, Layer::Layer4Type, *duration, timestamp, affected.clone(), modification);
                }
                Ok(())
            }

            // === Layer 6 — ability adding and removing (CR 613.1f) ===
            //
            // The resolution half: "target creature gains/loses ..." spells, whose
            // affected set is fixed as the effect begins (CR 611.2c).

            Primitive::GrantKeywordFlag(keyword, duration) => {
                self.add_resolution_effect(
                    ctx,
                    &actx,
                    Layer::Layer6Ability,
                    *duration,
                    self.affected_permanents(recipient, targets, ctx),
                    EffectModification::GrantKeywordFlag(*keyword),
                )?;
                Ok(())
            }

            Primitive::RemoveKeywordFlag(keyword, duration) => {
                self.add_resolution_effect(
                    ctx,
                    &actx,
                    Layer::Layer6Ability,
                    *duration,
                    self.affected_permanents(recipient, targets, ctx),
                    EffectModification::RemoveKeywordFlag(*keyword),
                )?;
                Ok(())
            }

            Primitive::LoseAbility(ability_id, duration) => {
                self.add_resolution_effect(
                    ctx,
                    &actx,
                    Layer::Layer6Ability,
                    *duration,
                    self.affected_permanents(recipient, targets, ctx),
                    EffectModification::LoseAbility(*ability_id),
                )?;
                Ok(())
            }

            Primitive::LoseAllAbilities(duration) => {
                self.add_resolution_effect(
                    ctx,
                    &actx,
                    Layer::Layer6Ability,
                    *duration,
                    self.affected_permanents(recipient, targets, ctx),
                    EffectModification::LoseAllAbilities,
                )?;
                Ok(())
            }

            Primitive::GrantAbility(def, duration) => {
                let grantees = self.affected_permanents(recipient, targets, ctx);
                let granted = self.add_resolution_effect(
                    ctx,
                    &actx,
                    Layer::Layer6Ability,
                    *duration,
                    grantees.clone(),
                    EffectModification::GrantAbility(def.clone()),
                )?;

                // CR 613.7a clause 2: a granted static ability's own effects take the
                // *later* of this effect's timestamp and the grantee's, so a permanent
                // that entered after the grant was created keeps its own.
                // `static_effect_timestamp` takes the max; this supplies the second candidate.
                if let Some((granted_at, row)) = granted {
                    for grantee in grantees {
                        self.register_granted_static_effects(
                            def, row, grantee, granted_at, *duration, ctx.controller,
                        );
                    }
                }
                Ok(())
            }

            // CR 701.7a — **one proposal, whatever N is**, one event by CR 614.16's
            // "one or more tokens"; `create_tokens` makes its entries one batch. The
            // def repeated `count` times lets a doubler repeat each def in place
            // (`replacement-architecture.md` §3.1).
            Primitive::CreateToken(token_def, amount_expr) => {
                let count = self.evaluate_amount(amount_expr, ctx, None)?;
                let controller = self.resolve_player_for_self(recipient, targets, ctx);
                // CR 800.4b — no token is created for a player who has left — and
                // CR 800.4d's first sentence at the same line, since CR 111.2 makes the
                // two rules name one player here. A rule checked ahead of the proposal,
                // like CR 508.8's: there is no event here for a replacement to see.
                if self.left_a_multiplayer_game(controller) {
                    return Ok(());
                }
                let defs = vec![token_def.clone(); count as usize];
                let attacking = token_def.enters_attacking.as_ref().map(|word| self.enters_attacking_candidates(word, ctx)).transpose()?;
                self.execute_action(
                    GameAction::CreateTokens { defs, controller, attacking },
                    &ActionContext::resolving(dp, ctx),
                )
            }

            // === Regeneration (CR 701.19) ===

            // CR 701.19a's shield, built by the engine because the rule names every
            // part: `Uses::Once` "the next time", `UntilEndOfTurn` "this turn",
            // `Prevent` "instead", and the `then` rider its sentence.
            Primitive::Regenerate => {
                let recorded = self.target_refs(targets);
                for object in self.affected_permanents(recipient, targets, ctx) {
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
                        source: ctx.effect_source(),
                        controller,
                        generated_by: ctx.generated_by,
                        duration: Duration::UntilEndOfTurn,
                        created_on_turn: self.turn_number,
                        targets: recorded.clone(),
                        def,
                    });
                }
                Ok(())
            }

            // === Replacement and prevention effects from a resolution (CR 614.3, 615.7) ===

            // `Effect::Replacement` with a duration: the card authors the def and the
            // duration, the resolution supplies targets, controller (CR 611.2c) and
            // turn. The recipient says when the set is fixed: `Target`, `Choose` and
            // `FilteredPermanents` now, one row each (CR 615.11, Samite
            // Censer-Bearer's ruling); `Implicit` at each event (Safe Passage's).
            // Per-target shapes author an empty `Fixed`, as `Restrict` does.
            Primitive::CreateReplacement(def, duration, pattern_fill) => {
                if self.never_starts(*duration, ctx) {
                    return Ok(());
                }
                // CR 113.7a — an ability's source is the object that has it,
                // so a row an activated ability makes names the permanent, not
                // the ephemeral stack object CR 608.2n deletes at the end of
                // resolution. A spell's is the spell.
                let source = ctx.effect_source();

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

                let authored_empty = def.affected == AffectedSet::NOTHING;
                let fill_object = |id: ObjectId| {
                    let mut row = (*def).clone();
                    row.affected.objects = ObjectSet::Fixed(vec![id]);
                    row
                };
                let fill_player = |pid: PlayerId| {
                    let mut row = (*def).clone();
                    row.affected.players = PlayerSet::Fixed(vec![pid]);
                    row
                };
                let rows: Vec<ReplacementDef> = match recipient {
                    EffectRecipient::Target(..)
                    | EffectRecipient::Choose(..)
                    | EffectRecipient::Group(_)
                    | EffectRecipient::ThisObject
                    | EffectRecipient::Triggering(_)
                    | EffectRecipient::TriggeringPlayer
                    | EffectRecipient::Referred
                    | EffectRecipient::AttackedBy(_)
                    | EffectRecipient::DefendingPlayer(_) => {
                        debug_assert!(
                            authored_empty,
                            "a `Primitive::CreateReplacement` on {:?} with a targeting \
                             recipient authored a non-empty affected set, which the \
                             resolution then overwrote with its own targets. Write \
                             `AffectedSet::NOTHING`.",
                            ctx.source
                        );
                        targets
                            .iter()
                            .filter_map(|t| match t {
                                // A target that left the battlefield since it
                                // was chosen is CR 608.2b's illegal one: the
                                // rest resolve, it gets nothing.
                                ObjectOrPlayerId::Object(id) if self.battlefield.contains_key(id) => {
                                    Some(fill_object(*id))
                                }
                                ObjectOrPlayerId::Object(_) => None,
                                ObjectOrPlayerId::Player(pid) => Some(fill_player(*pid)),
                            })
                            .collect()
                    }
                    EffectRecipient::SameInstanceAs(_) => return Err(back_reference(recipient, ctx)),
                    // A row per player (Kitsune Palliator) is `codebase-state.md` item 94's.
                    EffectRecipient::EachOf(_) => {
                        return Err(format!(
                            "a `Primitive::CreateReplacement` on {:?} names each of several players; item 94",
                            ctx.source
                        ));
                    }
                    // A row on a permanent of a player's choice waits for its card.
                    EffectRecipient::ChosenBy(_) => {
                        return Err(format!("{:?} on a `Primitive::CreateReplacement` is not built", recipient));
                    }
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
                            .filter(|id| self.matches_for_effect(*id, filter, ctx))
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
                let recorded = self.target_refs(targets);
                for row in rows {
                    self.replacement_effects.add(RegisteredReplacementEffect {
                        id: 0,
                        source,
                        controller: ctx.controller,
                        generated_by: ctx.generated_by,
                        duration: *duration,
                        created_on_turn: self.turn_number,
                        targets: recorded.clone(),
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
                if self.never_starts(*duration, ctx) {
                    return Ok(());
                }
                // **A restriction naming nobody waits for the resolution's targets**:
                // an empty `Fixed` beside `Nobody`, filled one row per target, since one
                // row naming both would make a second copy a no-op under CR 614.5.
                // Anything else is complete as authored (Skullcrack's `Everyone`).
                let mut rows: Vec<RestrictionDef> = Vec::new();
                let fill = |set: ObjectSet| {
                    let mut filled = def.clone();
                    let affected = filled.restriction.affected_mut();
                    affected.objects = set;
                    filled
                };
                if *def.restriction.affected() == AffectedSet::NOTHING {
                    match recipient {
                        // CR 611.2c's second sentence: a "can't" modifies no
                        // characteristic, so its set is not fixed, and a creature
                        // arriving later is in it (Magmatic Chasm's ruling). One
                        // row whose set is the filter, asked of each query with
                        // this row's controller and source.
                        EffectRecipient::FilteredPermanents(filter) => {
                            rows.push(fill(ObjectSet::battlefield_filter(filter.clone())));
                        }
                        _ => {
                            for object in self.affected_permanents(recipient, targets, ctx) {
                                rows.push(fill(ObjectSet::Fixed(vec![object])));
                            }
                        }
                    }
                } else {
                    rows.push(def.clone());
                }
                for def in rows {
                    self.restrictions.add(RegisteredRestriction {
                        id: 0,
                        source: ctx.effect_source(),
                        controller: ctx.controller,
                        duration: *duration,
                        created_on_turn: self.turn_number,
                        def,
                    });
                }
                Ok(())
            }

            // CR 701.21a. **The targeted player chooses, not the spell's controller**
            // (Diabolic Edict): the resolution-time selection whose candidates
            // `cant-effects-architecture.md` §4.9 filters, so Sigarda asks nothing.
            // Not destruction (CR 701.21b), hence its own `ZoneChangeAct`.
            Primitive::Sacrifice => {
                // A named permanent is the resolution's controller's to
                // sacrifice, and only while they control it and no "can't"
                // forbids it; a chosen one was chosen from its chooser's own,
                // past the same "can't".
                let named = !matches!(recipient, EffectRecipient::ChosenBy(_));
                let mut batch = Vec::new();
                for target in targets {
                    let ObjectOrPlayerId::Object(id) = *target else { continue };
                    if named && (!controls(self, id, ctx.controller) || !self.admits(primitive, id, targets, ctx)?) {
                        continue;
                    }
                    batch.extend(self.event_for(primitive, id, targets, ctx)?);
                }
                // CR 118.12 — nothing to sacrifice is an action that could not
                // be started.
                if batch.is_empty() {
                    walk.last_cost_answer = Some(CostAnswer::Cant);
                    return Ok(());
                }
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // === Combat and damage housekeeping ===

            Primitive::Tap | Primitive::BecomeBlocked => {
                // One batch: CR 608.2f processes a spell's actions over several
                // objects simultaneously.
                let batch = self.events_for(primitive, targets, ctx)?;
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // NOT REPLACEABLE: CR 506.4. Writes `PermanentState` directly: 506.4 defines a
            // *consequence* with seven causes, and this arm is one of them. The CR
            // does not forbid "can't be removed from combat"; it puts that card's
            // enforcement at all seven causes, which makes it a `RestrictionDef`
            // consulted by each (`cant-effects-architecture.md`) and not a
            // replaceable event here → `replacement-architecture.md` §8a. It is
            // announced, as every cause announces it.
            Primitive::RemoveFromCombat => {
                let objects = self.affected_permanents(recipient, targets, ctx);
                let removed = self.remove_from_combat(&objects);
                self.announce_removal_from_combat(removed, ctx.cause())
            }

            // NOT REPLACEABLE: a direct write because nothing replaces *this* removal — Pyramids, the
            // one printed card that removes all damage, uses it as a replacement's
            // substituted event. Not by analogy to the CR 514.2 cleanup wipe, a
            // rule's action a card can switch off (`Restriction::RuleDoesNotApply`),
            // where this is an effect's.
            Primitive::RemoveAllDamage => {
                for object in self.affected_permanents(recipient, targets, ctx) {
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

            // One batch: CR 608.2f processes a spell's actions over several
            // objects simultaneously, which is what lets a single CR 614.16
            // doubler see all of them.
            // "Each creature you control" is the permanents the filter matches
            // as the effect resolves (CR 611.2c's set, fixed then).
            Primitive::AddCounters { .. } | Primitive::RemoveCounters(..) => {
                let objects: Vec<ObjectOrPlayerId> = self
                    .affected_permanents(recipient, targets, ctx)
                    .into_iter()
                    .map(ObjectOrPlayerId::Object)
                    .collect();
                // Who puts them on is the instruction's to name, asked once as
                // it applies (CR 608.2d), and `event_for` reads it per object.
                let named;
                let primitive = match primitive {
                    Primitive::AddCounters { counter, amount, by } if !objects.is_empty() => {
                        let by = PlayerRef::Player(self.player_the_instruction_names(by, targets, ctx, dp)?);
                        named = Primitive::AddCounters { counter: *counter, amount: amount.clone(), by };
                        &named
                    }
                    _ => primitive,
                };
                let batch = self.events_for(primitive, &objects, ctx)?;
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // "You get {E}{E}" — the same event with a player as its subject,
            // so Winding Constrictor's "if you would get one or more counters"
            // watches it through the one arm.
            Primitive::GetCounters { counter, amount, by } => {
                let n = self.evaluate_amount(amount, ctx, None)? as u32;
                let player = self.resolve_player_for_self(recipient, targets, ctx);
                let by = self.player_the_instruction_names(by, targets, ctx, dp)?;
                self.execute_action(
                    GameAction::AddCounters {
                        subject: ObjectOrPlayerId::Player(player),
                        counter: *counter,
                        n,
                        by,
                    },
                    &actx,
                )?;
                Ok(())
            }

            // === Layer 2 — control-changing effects (CR 613.1b) ===

            Primitive::GainControl { to, duration } => {
                // A filter names permanents (Mob Rule); a target may also be a
                // spell, which has a controller too (CR 108.4).
                let affected = match recipient {
                    EffectRecipient::FilteredPermanents(_) => self.affected_permanents(recipient, targets, ctx),
                    _ => self.collect_permanent_or_spell_targets(targets),
                };
                // Nothing to take is no choice of who takes it.
                if affected.is_empty() {
                    return Ok(());
                }
                // `You` rather than `ctx.controller`, though they name the
                // same player: on a `Resolution` row, `FilterPlayers::you()`
                // reads `ContinuousEffect.controller`, which CR 611.2c
                // locked at resolution. `Owner` is each object's own (CR
                // 108.3), read per object by the walk (Homeward Path).
                // Anyone else is settled now.
                let to = match to {
                    PlayerRef::You | PlayerRef::Owner => to.clone(),
                    other => PlayerRef::Player(self.player_the_instruction_names(other, targets, ctx, dp)?),
                };
                self.add_resolution_effect(
                    ctx,
                    &actx,
                    Layer::Layer2Control,
                    *duration,
                    affected,
                    EffectModification::SetController(to),
                )?;
                Ok(())
            }

            // CR 701.24a — the recipient says whose library. `ThisObject` is the
            // source's owner's, "shuffle it into its owner's library" on a rider that
            // already moved it (Darksteel Colossus), read wherever the card went (CR
            // 108.3); a commander sent to the command zone instead still shuffles it
            // (CR 701.24c). No card shuffles per matching object, so filters are refused.
            Primitive::ShuffleLibrary => {
                let players: Vec<PlayerId> = match recipient {
                    EffectRecipient::Controller => vec![ctx.controller],
                    EffectRecipient::ThisObject => vec![self.source_owner_of(ctx)?],
                    EffectRecipient::Implicit => {
                        return Err(format!(
                            "a `Primitive::ShuffleLibrary` on {:?} names no library; use `Controller` or `ThisObject`",
                            ctx.source
                        ));
                    }
                    EffectRecipient::Target(..)
                    | EffectRecipient::Choose(..)
                    | EffectRecipient::Group(_)
                    | EffectRecipient::Triggering(_)
                    | EffectRecipient::TriggeringPlayer
                    | EffectRecipient::Referred
                    | EffectRecipient::AttackedBy(_)
                    | EffectRecipient::DefendingPlayer(_) => targets
                        .iter()
                        .filter_map(|t| match t {
                            ObjectOrPlayerId::Player(pid) => Some(*pid),
                            ObjectOrPlayerId::Object(id) => {
                                self.objects.get(id).map(|obj| obj.owner)
                            }
                        })
                        .collect(),
                    EffectRecipient::SameInstanceAs(_) => return Err(back_reference(recipient, ctx)),
                    // Refused at the atom: "each player shuffles" waits for its card,
                    // and a library is no object to choose.
                    EffectRecipient::EachOf(_) | EffectRecipient::ChosenBy(_) => {
                        return Err(format!("{:?} on a `Primitive::ShuffleLibrary` is not built", recipient));
                    }
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

            // Each object from where it is now, as one event (CR 608.2f),
            // under its owner's control or the effect's controller's.
            Primitive::ReturnToBattlefield { under, counters, tapped, attacking } => {
                let attacking = attacking.as_ref().map(|word| self.enters_attacking_candidates(word, ctx)).transpose()?;
                let mut returns = Vec::new();
                for target in targets {
                    let ObjectOrPlayerId::Object(object) = *target else { continue };
                    let controller = self.return_controller(object, *under, ctx.controller)?;
                    returns.push(ReturningObject {
                        object,
                        to: Zone::Battlefield,
                        controller,
                        counters: counters.clone(),
                        tapped: *tapped,
                        attacking: attacking.clone(),
                        cause: actx.cause,
                    });
                }
                self.return_objects(&returns, &actx)
            }

            Primitive::ReturnToHand => {
                let batch = self.events_for(primitive, targets, ctx)?;
                self.execute_actions(batch, &actx)?;
                Ok(())
            }

            // Into its owner's library, then each such library is shuffled
            // (CR 701.24a), as one event of one member per library.
            Primitive::ShuffleIntoLibrary => {
                let batch = self.events_for(primitive, targets, ctx)?;
                let mut owners: Vec<PlayerId> = Vec::new();
                for action in &batch {
                    if let GameAction::ZoneChange { object, .. } = action
                        && let Ok(owner) = self.get_object(*object).map(|o| o.owner)
                        && !owners.contains(&owner)
                    {
                        owners.push(owner);
                    }
                }
                self.execute_actions(batch, &actx)?;
                let shuffles: Vec<GameAction> = owners.into_iter().map(|player| GameAction::ShuffleLibrary { player }).collect();
                if !shuffles.is_empty() {
                    self.execute_actions(shuffles, &actx)?;
                }
                Ok(())
            }

            // === Unimplemented primitives — `backlog.md` §2.5 ===

            Primitive::PutOnTopOfLibrary
            | Primitive::PutOnBottomOfLibrary
            | Primitive::Surveil(_)
            | Primitive::Fight => {
                Err(format!("Primitive {:?} not yet implemented", primitive))
            }
        }
    }

    // --- Helpers: a resolution's continuous effect (CR 611.2c) ---

    /// Record a one-row continuous effect a resolving spell or ability
    /// creates over `affected`, its set fixed as the effect begins
    /// (CR 611.2c) — the resolution-time counterpart of
    /// `GameState::register_static_effects`.
    ///
    /// **This changes no object.** It appends one row to the registry; the
    /// characteristics it implies are recomputed from that row on every
    /// `compute_characteristics` call. Duration expiry removes the row, and
    /// there is nothing to undo because nothing was ever written.
    ///
    /// Returns the row's timestamp and id, so a caller that must register
    /// further rows against the same moment, or name the instance a grant
    /// made, can. `None` means `affected` is empty, in which case no row is
    /// written and no timestamp is burned.
    fn add_resolution_effect(
        &mut self,
        ctx: &ResolutionContext,
        actx: &ActionContext,
        layer: Layer,
        duration: Duration,
        affected: Vec<ObjectId>,
        modification: EffectModification,
    ) -> Result<Option<(Timestamp, EffectId)>, String> {
        if affected.is_empty() || self.never_starts(duration, ctx) {
            return Ok(None);
        }
        self.settle_before_a_resolution_effect(actx)?;
        let timestamp = self.allocate_timestamp();
        Ok(Some((timestamp, self.add_resolution_row(ctx, layer, duration, timestamp, affected, modification))))
    }

    /// CR 611.2b — "if the 'for as long as' duration never starts, the
    /// effect does nothing": the object it lasts as long as is the ability's
    /// own, and it has left the battlefield or become a new object there (CR
    /// 400.7, Sower of Temptation's ruling), or, for "as long as you control"
    /// it, is no longer this resolution's controller's. A spell is never on
    /// the battlefield to last as long as.
    fn never_starts(&self, duration: Duration, ctx: &ResolutionContext) -> bool {
        if !duration.ends_with_its_source() {
            return false;
        }
        let Some(watched) = ctx.ability_source else { return true };
        if !self.battlefield.contains_key(&watched.id) || self.object_ref(watched.id) != Some(watched) {
            return true;
        }
        duration.ends_with_control_of_its_source() && get_effective_controller(self, watched.id) != Some(ctx.controller)
    }

    /// CR 603.2: what an earlier instruction's effect did to a controller or
    /// a combatant (CR 603.10d, 506.4) is an event as it happens, so it is
    /// announced before the next effect is written rather than merged into
    /// the next batch's. Inside a batch, as a rider's resolution is, the
    /// batch's close settles it.
    fn settle_before_a_resolution_effect(&mut self, actx: &ActionContext) -> Result<(), String> {
        if self.nesting.batch_depth > 0 || self.nesting.dispatch_depth > 0 {
            return Ok(());
        }
        self.settle_layer_consequences(actx)
    }

    /// One row of a resolution's continuous effect at `timestamp`, which the
    /// caller allocates: a type change's sibling rows share one (CR 613.7b),
    /// so the caller settles once, before the effect's first row.
    fn add_resolution_row(
        &mut self,
        ctx: &ResolutionContext,
        layer: Layer,
        duration: Duration,
        timestamp: Timestamp,
        affected: Vec<ObjectId>,
        modification: EffectModification,
    ) -> EffectId {
        // A row a resolution writes may move control, and the change looks
        // back (CR 603.10d): Act of Treason's steal is read with whatever its
        // own resolution did first.
        self.keep_frames_before_control_change(layer == Layer::Layer2Control);
        self.continuous_effects.add(ContinuousEffect {
            id: 0,
            source: ctx.effect_source(),
            origin: EffectOrigin::Resolution,
            layer,
            duration,
            controller: ctx.controller,
            created_on_turn: self.turn_number,
            timestamp,
            affected_objects: ObjectSet::Fixed(affected),
            modification,
        })
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
    /// every resolution-time grant (CR 611.2c fixes the affected set as the
    /// effect begins). A *static* ability that grants a static ability over a filter
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

        // The lowering `register_static_effects` uses, for the reason
        // `static_primitive_rows`' doc gives: a granted "creatures you control
        // get +1/+1" must produce the row a printed one does. Assertions included —
        // a grant that lowers to nothing is exactly as inert and as invisible.
        let label = format!("granted ability {:?}", granted.id);
        for (affected, rows) in GameState::static_ability_rows(granted, &label) {
            let timestamp =
                self.static_effect_timestamp(grantee, granted, Some(granting_timestamp));

            for (layer, modification) in rows {
                // A granted static whose effect lands in layers 1–5 would fail silently:
                // below layer 6 the existence check (CR 604.2) reads a frame from before
                // the grant, and nothing lets a layer 6 grant reach back (CR 613.8a).
                // No printed grant is of a type-, color- or subtype-defining static.
                // Layer 6 itself is fine: the pass sees the grant at the derived row's
                // turn (CR 613.7a's Rune of Flight), and clause 2's timestamp sorts it first.
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
        except: &[CopyException],
        duration: Duration,
        targets: &[ObjectOrPlayerId],
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
    ) -> Result<(), String> {
        if self.never_starts(duration, ctx) {
            return Ok(());
        }
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
                        ObjectOrPlayerId::Object(id) => Some(id),
                        ObjectOrPlayerId::Player(_) => None,
                    })
                    .collect();
                // None is CR 101.3's impossible instruction, and the whole
                // effect does nothing.
                if candidates.is_empty() {
                    return Ok(());
                }
                let donor = crate::ui::ask::ask_choose_copy_source(dp, self, ctx.controller, ctx.source, &candidates);
                (donor, recipients)
            }
            CopyRoles::ThisObjectCopiesRecipient => {
                // "This creature" is found by identity (CR 400.7): a source
                // that left and came back is not the creature the ability is
                // of, and the copy finds nothing to change.
                let (Some(&donor), Some(this)) =
                    (self.collect_battlefield_targets(targets).first(), self.this_object(ctx))
                else {
                    return Ok(());
                };
                if !self.battlefield.contains_key(&this) {
                    return Ok(());
                }
                (donor, vec![this])
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
                    .filter(|&id| self.matches_for_effect(id, filter, ctx))
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
        let Some(captured) = crate::engine::layers::copiable_values(self, donor) else {
            return Ok(());
        };
        let except = if except.is_empty() { Vec::new() } else { self.with_this_ability(except, ctx)? };
        // CR 707.9c: "doesn't copy" keeps each copying object's own value, so a
        // copy over several objects that keeps one is a row per object.
        let keeps_own_value = except.iter().any(|exception| matches!(exception, CopyException::DoesNotCopy(_)));
        let row_subjects: Vec<Vec<ObjectId>> =
            if keeps_own_value { affected.iter().map(|&id| vec![id]).collect() } else { vec![affected] };
        // One effect, whatever its rows: settled before the first write.
        self.settle_before_a_resolution_effect(&ActionContext::resolving(dp, ctx))?;
        for affected in row_subjects {
            let mut values = captured.clone();
            if !except.is_empty() {
                self.make_copy_exceptions(&mut values, &except, &affected)?;
            }
            // CR 613.7, 707.4: a copy that lasts as long as its object hides
            // every earlier copy of it for that whole time, so the earlier
            // ones retire rather than stay one row more per re-copy. After the
            // capture, since 707.9c's own value is read through them.
            if duration == Duration::Indefinite {
                for &object in &affected {
                    self.continuous_effects.retire_earlier_copies_of(object);
                }
            }

            // --- 3. The row (CR 613.2a) ---------------------------------
            //
            // Each copied ability is tagged with the row, as a Layer 6 grant
            // is: a second copy of the same donor is a second instance of each
            // of its abilities, and the rows a copied static ability generates
            // must apply only while their own copy is the one showing (item 16b).
            let row = self.continuous_effects.next_id();
            for ability in std::sync::Arc::make_mut(&mut values.abilities) {
                ability.id = ability.id.copied_by(row);
            }
            // A timestamp per row, though CR 613.7b gives the effect one: the
            // rows are allocated together and affect different objects, so
            // their order is unobservable, while one timestamp would make them
            // one `EffectGroup`, and CR 613.6 would hold every row to the
            // first one's object.
            let timestamp = self.allocate_timestamp();
            self.add_resolution_row(
                ctx,
                Layer::Layer1Copy,
                duration,
                timestamp,
                affected.clone(),
                EffectModification::CopyFrom(std::sync::Arc::new(values.clone())),
            );

            // `copy-effects-architecture.md` §4.7 leg 2: the row alone makes
            // the copy *have* the ability; it does not make the ability *do*
            // anything.
            self.register_copied_static_effects(&values, &affected, timestamp, duration);
        }
        Ok(())
    }

    /// `except` with "it has this ability" made the ability it names: the one
    /// resolving, which [`GameState::resolving`] identifies.
    ///
    /// An ability on the stack exists apart from its source (CR 113.7a), so
    /// the def is the source's while the source still has that instance, and
    /// otherwise a trigger's own: an effect in response that took the ability
    /// away does not take "this ability" from the copy. A rebuilt def is held
    /// to what `CardDataBuilder::build` does, its instances of "target" listed.
    fn with_this_ability(&self, except: &[CopyException], ctx: &ResolutionContext) -> Result<Vec<CopyException>, String> {
        use crate::objects::card_data::{AbilityType, ActivationRestriction};
        use crate::types::effects::CharacteristicEdit;
        let names_it = |e: &CopyException| matches!(e, CopyException::Modifies(CharacteristicEdit::GainsThisAbility));
        if !except.iter().any(names_it) {
            return Ok(except.to_vec());
        }
        let identity = self
            .resolving
            .as_ref()
            .and_then(|resolving| resolving.identity)
            .ok_or("\"except it has this ability\" on a resolution that is not an ability's")?;
        let source = identity.source.id;
        let this = match crate::oracle::characteristics::effective_ability(self, source, identity.ability) {
            Some(def) => (*def).clone(),
            None => {
                let trigger = ctx.trigger.as_ref().ok_or_else(|| {
                    format!("{source}'s ability {} is gone and only a trigger carries its own def", identity.ability)
                })?;
                let effect = Effect::Triggered(std::sync::Arc::clone(&trigger.def));
                // PRE-LAYER ZONE: the printed text, which a rebuilt def carries
                // for the window and no rule reads.
                let printed = self.objects.get(&source).and_then(|obj| {
                    crate::objects::card_data::printed_place(&obj.card_data.abilities, identity.ability)
                        .map(|place| obj.card_data.abilities[place].rules_text)
                });
                crate::objects::card_data::AbilityDef {
                    id: identity.ability,
                    rules_text: printed.unwrap_or_else(|| "".into()),
                    ability_type: AbilityType::Triggered,
                    costs: Vec::new(),
                    instances: effect.instances(),
                    effect,
                    is_characteristic_defining: false,
                    activation_restriction: ActivationRestriction::None,
                }
            }
        };
        Ok(except
            .iter()
            .map(|e| if names_it(e) { CopyException::Modifies(CharacteristicEdit::GainsAbility(this.clone())) } else { e.clone() })
            .collect())
    }

    /// CR 707.9a–c — make a resolution's copy exceptions on the captured
    /// values, so they are the copy's copiable values (707.9b) and a copy of
    /// the copy has them too.
    ///
    /// "It doesn't copy that creature's color" keeps the copying object's own
    /// value (707.9c), read here, which is why [`Self::apply_copy`] hands a
    /// copy that keeps one a single object per row. 707.9e's additions and
    /// 707.9f's conditions are an entry's, and refused.
    fn make_copy_exceptions(
        &self,
        values: &mut crate::engine::layers::copy::CopiableValues,
        except: &[CopyException],
        affected: &[ObjectId],
    ) -> Result<(), String> {
        for exception in except {
            match exception {
                CopyException::Modifies(_) | CopyException::DoesNotCopy(_) => {}
                CopyException::Additionally(_) | CopyException::If(..) => {
                    return Err(format!("{exception:?} is about a permanent entering (CR 707.9e, 707.9f), not a resolution's copy"));
                }
            }
        }
        let own = match affected {
            [only] => crate::engine::layers::copiable_values(self, *only),
            _ => None,
        };
        let exceptions: Vec<&CopyException> = except.iter().collect();
        values.except(&exceptions, own.as_ref())
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

        let candidates = crate::oracle::legality::damage_sources(self, None);
        // None is CR 101.3's impossible instruction, and the effect does nothing.
        if candidates.is_empty() {
            return Ok(None);
        }
        let chosen = crate::ui::ask::ask_choose_damage_source(dp, self, ctx.controller, ctx.source, &candidates);

        let mut filled = def.clone();
        if let EventPattern::DealDamage { source: Some(pattern), .. } = &mut filled.pattern {
            pattern.object = Some(chosen);
        }
        Ok(Some(filled))
    }

    /// [`EffectRecipient::ChosenBy`]: each player the chooser names picks, in
    /// APNAP order (CR 101.4), from the choice's scope, and every pick comes
    /// back in that order for the verb to act on at once. A candidate is one
    /// the verb's own event may act on (CR 101.2).
    fn choose_as_it_applies(
        &mut self,
        primitive: &Primitive,
        choice: &Choice,
        declared: DeclaredInstances<'_>,
        walk: &mut ResolutionWalk,
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
    ) -> Result<Vec<ObjectOrPlayerId>, String> {
        let choosers: Vec<PlayerId> = match &choice.chooser {
            chooser @ (EffectRecipient::Target(..) | EffectRecipient::Choose(..) | EffectRecipient::SameInstanceAs(_)) => {
                match instance_of(chooser, declared, &mut walk.instance_cursor) {
                    Some((ix, _)) => ctx
                        .targets
                        .instance(ix)
                        .iter()
                        .filter_map(|t| match t {
                            ObjectOrPlayerId::Player(pid) => Some(*pid),
                            ObjectOrPlayerId::Object(_) => None,
                        })
                        .collect(),
                    None => Vec::new(),
                }
            }
            chooser => self
                .players_named_by(chooser, ctx)?
                .ok_or_else(|| format!("{:?} names no player to make a choice", chooser))?,
        };
        let mut chosen = Vec::new();
        for player in choosers {
            for pick in &choice.picks {
                let mut candidates = Vec::new();
                match choice.among {
                    ChoiceScope::ChoosersPermanents => {
                        for id in crate::oracle::board::permanents_controlled_by(self, player) {
                            if self.matches_for_effect(id, &pick.filter, ctx)
                                && self.admits(primitive, id, &[], ctx)?
                            {
                                candidates.push(ObjectOrPlayerId::Object(id));
                            }
                        }
                    }
                }
                let count = match &pick.count {
                    PickCount::Exactly(amount) => self.evaluate_amount(amount, ctx, None)? as usize,
                };
                let n = count.min(candidates.len());
                let asked = EffectRecipient::Choose(
                    SelectionFilter::Permanent(pick.filter.clone()),
                    TargetCount::Exactly(n as u32),
                );
                chosen.extend(crate::ui::ask::ask_select_recipients(
                    dp, self, player, &asked, ctx.source, &candidates, n, n,
                ));
            }
        }
        Ok(chosen)
    }

    /// The event `primitive` proposes for one object: the one table an object
    /// verb's batch and a choice's "can't" check both read, so the event a
    /// candidate is checked against is the event performed. `None` for an
    /// object the verb cannot act on where it is; `Err` for a verb that acts
    /// on no object.
    fn event_for(
        &self,
        primitive: &Primitive,
        object: ObjectId,
        targets: &[ObjectOrPlayerId],
        ctx: &ResolutionContext,
    ) -> Result<Option<GameAction>, String> {
        let is_permanent = self.battlefield.contains_key(&object);
        Ok(match primitive {
            Primitive::Destroy => {
                is_permanent.then(|| GameAction::Destroy { object })
            }
            // CR 701.13a — from wherever the object is.
            Primitive::Exile => self.objects.get(&object).map(|obj| GameAction::ZoneChange {
                object,
                from: obj.zone,
                to: Zone::Exile,
                act: ZoneChangeAct::Put,
            }),
            Primitive::Sacrifice => is_permanent.then_some(GameAction::ZoneChange {
                object,
                from: Zone::Battlefield,
                to: Zone::Graveyard,
                act: ZoneChangeAct::Sacrificed,
            }),
            // To its owner's hand or library (CR 400.3), from wherever it is.
            Primitive::ReturnToHand | Primitive::ShuffleIntoLibrary => {
                let to = if matches!(primitive, Primitive::ReturnToHand) { Zone::Hand } else { Zone::Library };
                self.objects.get(&object).map(|obj| GameAction::ZoneChange { object, from: obj.zone, to, act: ZoneChangeAct::Put })
            }
            Primitive::Tap => is_permanent.then_some(GameAction::Tap { object }),
            Primitive::BecomeBlocked => is_permanent.then_some(GameAction::BecomeBlocked { attacker: object }),
            Primitive::Untap => is_permanent.then_some(GameAction::Untap { object }),
            Primitive::AddCounters { counter, amount, by } => match is_permanent {
                false => None,
                true => Some(GameAction::AddCounters {
                    subject: ObjectOrPlayerId::Object(object),
                    counter: *counter,
                    n: self.evaluate_amount(amount, ctx, None)? as u32,
                    by: self.resolve_player_ref(by, targets, ctx)?,
                }),
            },
            // CR 608.2c: the resolution's controller removes them.
            Primitive::RemoveCounters(counter, amount) => match is_permanent {
                false => None,
                true => Some(GameAction::RemoveCounters {
                    subject: ObjectOrPlayerId::Object(object),
                    counter: *counter,
                    n: self.evaluate_amount(amount, ctx, None)? as u32,
                    by: Some(ctx.controller),
                }),
            },
            other => return Err(format!("{:?} acts on no object, so it has no event for {}", other, object)),
        })
    }

    /// [`Self::event_for`] over the objects of a target slice, in its order.
    fn events_for(
        &self,
        primitive: &Primitive,
        targets: &[ObjectOrPlayerId],
        ctx: &ResolutionContext,
    ) -> Result<Vec<GameAction>, String> {
        let mut batch = Vec::new();
        for target in targets {
            if let ObjectOrPlayerId::Object(id) = *target {
                batch.extend(self.event_for(primitive, id, targets, ctx)?);
            }
        }
        Ok(batch)
    }

    /// May the verb act on `object` at all: it has an event there, and no
    /// "can't" forbids the event this resolution would cause (CR 101.2 —
    /// `cant-effects-architecture.md` §4.9's candidate filter).
    fn admits(
        &self,
        primitive: &Primitive,
        object: ObjectId,
        targets: &[ObjectOrPlayerId],
        ctx: &ResolutionContext,
    ) -> Result<bool, String> {
        Ok(match self.event_for(primitive, object, targets, ctx)? {
            None => false,
            Some(action) => !crate::engine::restriction::is_prohibited(
                self,
                &crate::engine::restriction::Query::Event {
                    action: &action,
                    cause: ctx.cause(),
                    lookahead: None,
                },
            ),
        })
    }

    /// `filter` asked of `id` for this resolution's effect: "you" is its
    /// controller (CR 109.5) and "another" is other than its source, the
    /// object `ResolutionContext::effect_source` names (`codebase-state.md`
    /// item 229).
    fn matches_for_effect(&self, id: ObjectId, filter: &ObjectFilter, ctx: &ResolutionContext) -> bool {
        let named = self.condition_objects(ctx.ability_source, ctx.trigger.as_ref());
        let identity = FilterIdentity::for_text_of(ctx.effect_source(), EarlierTargets::None).naming(named);
        self.object_matches_filter(ObjectInformation::Current(id), filter, &mut TextReferents { game: self, you: ctx.controller, identity })
    }

    /// CR 701.6a — a spell countered goes to its owner's graveyard, through
    /// `change_zone` so the CR 614 pipeline sees it, and the `StackEntry`
    /// goes with it. An object that is no spell on the stack is not one to
    /// counter this way.
    fn counter_spell(&mut self, id: ObjectId, ctx: &ResolutionContext, actx: &ActionContext) -> Result<(), String> {
        // Its controller read while it is a spell (CR 108.4a: a card in a
        // graveyard has none), and its frame as the move took it, for CR
        // 603.10e's look-back at the spell.
        if !self.stack.contains(&id) || !self.is_spell_on_stack(id) {
            return Ok(());
        }
        let Some(controller) = crate::oracle::characteristics::get_effective_controller(self, id) else { return Ok(()) };
        let spell = self.object_ref(id).ok_or_else(|| format!("countering {id}, which is in no zone"))?;
        self.change_zone(id, crate::types::zones::Zone::Graveyard, ZoneChangeAct::Countered, actx)?;
        let lki = self.lki(spell).cloned().ok_or_else(|| format!("countering {id}, which left the stack unframed"))?;
        debug_assert_eq!(lki.characteristics().controller, controller, "{id}'s frame names another controller than the stack's");
        let countered = crate::events::event::GameEvent::SpellCountered { countered_by: self.existence_for_record(ctx.source)?, lki };
        self.emit_event(countered, actx.cause)
    }

    /// CR 701.6b's ability: it ceases to exist, removed from the stack and
    /// put into no zone.
    fn counter_ability(&mut self, id: ObjectId, ctx: &ResolutionContext, actx: &ActionContext) -> Result<(), String> {
        if self.stack.contains(&id)
            && let Some(entry) = self.ability_leaves_the_stack(id)
            && let Some(identity) = entry.ability_identity
        {
            let countered_by = self.existence_for_record(ctx.source)?;
            let countered = crate::events::event::GameEvent::AbilityCountered { identity, controller: entry.controller, countered_by };
            self.emit_event(countered, actx.cause)?;
        }
        Ok(())
    }

    /// The objects or players one member of a `Group` recipient names, as that
    /// recipient would on its own: this object, the object or player that
    /// played a part in the triggering event, the object a delayed trigger
    /// refers to, or each permanent a filter matches now. A member that
    /// targets or chooses is refused loudly: its instance would need a place
    /// in the announcement, which a group does not give it yet.
    fn resolve_group_member(&self, member: &EffectRecipient, ctx: &ResolutionContext) -> Result<Vec<ObjectOrPlayerId>, String> {
        Ok(match member {
            EffectRecipient::ThisObject => self.this_object(ctx).map(ObjectOrPlayerId::Object).into_iter().collect(),
            EffectRecipient::Triggering(_) | EffectRecipient::TriggeringPlayer | EffectRecipient::Referred => {
                self.bound_targets(member, ctx)?
            }
            EffectRecipient::FilteredPermanents(_) => {
                self.affected_permanents(member, &[], ctx).into_iter().map(ObjectOrPlayerId::Object).collect()
            }
            other => {
                return Err(format!(
                    "{other:?} is no member a `Group` resolves on {:?}: a member is this object, a trigger's bound fact or a filtered sweep",
                    ctx.source
                ));
            }
        })
    }

    /// The permanents a one-shot continuous effect applies to: its resolved
    /// targets, or for "each [permanent] you control" every permanent the
    /// filter matches now. CR 611.2c fixes that set as the effect begins, so
    /// a creature that becomes an Elf later is not in it (Elvish Warmaster's
    /// third ruling).
    fn affected_permanents(
        &self,
        recipient: &EffectRecipient,
        targets: &[ObjectOrPlayerId],
        ctx: &ResolutionContext,
    ) -> Vec<ObjectId> {
        match recipient {
            EffectRecipient::FilteredPermanents(filter) => self
                .battlefield_ids_ordered()
                .into_iter()
                .filter(|&id| self.matches_for_effect(id, filter, ctx))
                .collect(),
            _ => self.collect_battlefield_targets(targets),
        }
    }

    /// `targets` as a row keeps them: each object by identity, at the epoch
    /// it has now. An object an earlier part of this effect moved is the one
    /// it moved, which CR 400.7j lets the rest of the effect find.
    fn target_refs(&self, targets: &[ObjectOrPlayerId]) -> Vec<crate::types::ids::ObjectOrPlayer> {
        targets.iter().filter_map(|&target| self.target_ref(target)).collect()
    }

    /// The resolved targets that are on the battlefield now.
    fn collect_battlefield_targets(&self, targets: &[ObjectOrPlayerId]) -> Vec<ObjectId> {
        targets.iter()
            .filter_map(|t| {
                if let ObjectOrPlayerId::Object(id) = t
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
    fn collect_permanent_or_spell_targets(&self, targets: &[ObjectOrPlayerId]) -> Vec<ObjectId> {
        targets
            .iter()
            .filter_map(|t| match t {
                ObjectOrPlayerId::Object(id)
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

    /// Where the resolution in progress starts in the event window: the
    /// number its first record would take, whatever that record turns out to
    /// be. "Dealt this way" reads the resolution's damage records from there.
    fn resolution_began_at(&self) -> Result<crate::events::event::EventId, String> {
        self.resolving
            .as_ref()
            .map(|resolving| resolving.began_at)
            .ok_or_else(|| "damage dealt this way has no meaning outside a resolution".to_string())
    }

    /// An amount's value. `announced_x` is the X announced for a spell or
    /// ability that is not resolving yet (CR 601.2b), which a count of
    /// targets reads as its targets are announced (CR 601.2c); `None` reads
    /// the X of the object resolving.
    pub(crate) fn evaluate_amount(
        &self,
        expr: &AmountExpr,
        _ctx: &ResolutionContext,
        announced_x: Option<u64>,
    ) -> Result<u64, String> {
        match expr {
            AmountExpr::Fixed(n) => Ok(*n),
            // CR 107.3 — the X chosen for the spell or ability resolving, which
            // its stack entry carried since the cast, and a delayed trigger's
            // its creator's (107.3n). A rider resolves no stack object, and has
            // none.
            AmountExpr::X => {
                self.note_binding_read(_ctx, TriggerBindingReads::X);
                announced_x
                    .or_else(|| self.resolving_for(_ctx).and_then(|resolving| resolving.x_value))
                    .ok_or_else(|| format!("{} resolves with no X chosen for AmountExpr::X to read", _ctx.source))
            }
            // The permanents a selector names, as they are now (CR 608.2h),
            // "you" the resolution's controller: Ajani, Nacatl Avenger's "the
            // number of creatures you control".
            // A hand's size, which any player may count (CR 402.3): "discard
            // your hand" is a discard of this many.
            AmountExpr::CountOf(set) => {
                if let Some(whose) = set.whose_hand() {
                    let player = self.resolve_player_ref(whose, _ctx.targets.all(), _ctx)?;
                    return Ok(self.get_player(player)?.hand.len() as u64);
                }
                let filter = match set {
                    ObjectSet::Filter { filter, zones } if *zones == ZoneSet::BATTLEFIELD => filter,
                    other => return Err(format!("CountOf({other:?}) resolution not yet implemented")),
                };
                let counted = self.battlefield_ids_ordered().into_iter().filter(|&id| self.matches_for_effect(id, filter, _ctx));
                Ok(counted.count() as u64)
            }
            AmountExpr::CardTypesAmong(_set) => {
                Err("CardTypesAmong amount resolution not yet implemented".to_string())
            }
            AmountExpr::Plus(inner, n) => Ok(self.evaluate_amount(inner, _ctx, announced_x)? + n),
            // "You" is the controller, and the text's "this" its source: the
            // permanent whose ability it is (CR 113.7a), not the ability.
            AmountExpr::History(span, history) => {
                let (you, source) = (_ctx.controller, _ctx.effect_source());
                let mut filter_matches =
                    |filter: &ObjectFilter, object: ObjectInformation<'_>| self.object_matches_filter_of_source(object, filter, you, source);
                Ok(self.history.total(history, *span, you, self.turn_number, &mut filter_matches, &|_| {}))
            }
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
                self.note_binding_read(_ctx, TriggerBindingReads::AMOUNT);
                self.bound_amount(binding).ok_or_else(|| {
                    format!("{:?} carries no amount for TriggeringAmount to read", binding.event())
                })
            }
            // "Its power", "its toughness" (CR 608.2h). A negative value is
            // no amount: Paladin of Atonement's ruling gains nothing, and
            // loses nothing, for toughness below 0.
            AmountExpr::TriggeringPower(role) | AmountExpr::TriggeringToughness(role) => {
                let binding = _ctx.trigger.as_ref().ok_or_else(|| {
                    format!("{:?} has no meaning outside a triggered ability's resolution", expr)
                })?;
                self.note_binding_read(_ctx, TriggerBindingReads::CHARACTERISTICS | TriggerBindingReads::event_role(*role));
                let chars = self.bound_characteristics(binding, *role).ok_or_else(|| {
                    format!(
                        "{:?}: the bound object has left since the event and no last known information answers for it, which is a missed capture",
                        expr
                    )
                })?;
                let value = match expr {
                    AmountExpr::TriggeringPower(_) => chars.power,
                    _ => chars.toughness,
                };
                Ok(value.unwrap_or(0).max(0) as u64)
            }
            // The source's counters as CR 608.2h reads it: as it last existed
            // once it has left, a dies trigger's own or a sacrifice paid as the
            // cost; on the battlefield, where the ability expected it, while
            // it is still that object. Neither is a missed capture.
            AmountExpr::SourceCounters(kind) => {
                self.note_binding_read(_ctx, TriggerBindingReads::SOURCE);
                if let Some(lki) = self.source_lki(_ctx) {
                    return Ok(u64::from(lki.counters(*kind)));
                }
                let source = self.this_object(_ctx).ok_or_else(|| {
                    format!(
                        "{:?}: the source of {} has left and no last known information answers for it, which is a missed capture",
                        expr, _ctx.source
                    )
                })?;
                Ok(self.battlefield.get(&source).map_or(0, |entry| u64::from(entry.counter_count(*kind))))
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
            // The damage records this resolution performed, whichever object
            // dealt them (Windswift Slice's creature); a rider's or a mana
            // ability's are another resolution's. The window holds them
            // until the resolution ends.
            AmountExpr::DamageDealtThisWay | AmountExpr::ExcessDamageDealtThisWay => {
                let excess_only = matches!(expr, AmountExpr::ExcessDamageDealtThisWay);
                Ok(self
                    .events
                    .resolution_records(_ctx.stamp(), self.resolution_began_at()?)
                    .map(|record| match &record.event {
                        GameEvent::DamageDealt { excess, .. } if excess_only => *excess,
                        GameEvent::DamageDealt { amount, .. } => *amount,
                        _ => 0,
                    })
                    .sum())
            }
            // The discards this resolution performed, one record a card (CR
            // 701.9a), as the damage above is this resolution's own.
            AmountExpr::DiscardedThisWay => Ok(self
                .events
                .resolution_records(_ctx.stamp(), self.resolution_began_at()?)
                .filter(|record| matches!(record.event, GameEvent::ZoneChange { act: ZoneChangeAct::Discarded, .. }))
                .count() as u64),
            // CR 615.5's "that much"/"that many". Only a rider sets the field, so
            // outside a `ReplacementDef::then` this leaf asks a question its context
            // cannot answer, and a 0 would be silently wrong.
            AmountExpr::ReplacedAmount => match _ctx.replaced_event {
                Some(ReplacedEvent::Damage { amount, .. } | ReplacedEvent::Amount(amount)) => Ok(amount),
                Some(ReplacedEvent::NoAmount) => {
                    Err("ReplacedAmount: the event this rider replaced carried no amount".to_string())
                }
                None => Err("ReplacedAmount has no meaning outside a CR 615.5 rider".to_string()),
            },
            // Doubling Cube's "each type of unspent mana you have": the
            // controller's pool now, restricted units counted by their type
            // (CR 106.6 — a restriction "doesn't affect the mana's type").
            AmountExpr::UnspentMana(mana_type) => {
                Ok(self.get_player(_ctx.controller)?.mana_pool.unspent(*mana_type))
            }
            // The other number a CR 615.5 rider may refer to, which only damage
            // has, and the same refusal outside one.
            AmountExpr::DamagePrevented => match _ctx.replaced_event {
                Some(ReplacedEvent::Damage { prevented, .. }) => Ok(prevented),
                Some(ReplacedEvent::Amount(_) | ReplacedEvent::NoAmount) => {
                    Err("DamagePrevented: the event this rider replaced was not damage".to_string())
                }
                None => Err("DamagePrevented has no meaning outside a CR 615.5 rider".to_string()),
            },
            // Saturating rather than checked: the inner amount is a damage or life
            // number and the factor is printed on a card, and a debug panic or a
            // release wrap is not an answer. `AmountRewrite::Multiplier` saturates too.
            AmountExpr::Multiply(inner, n) => {
                Ok(self.evaluate_amount(inner, _ctx, announced_x)?.saturating_mul(*n))
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

    /// "Its owner" of the resolving effect's source (CR 108.3), wherever the
    /// source has gone (`owner_of`).
    fn source_owner_of(&self, ctx: &ResolutionContext) -> Result<PlayerId, String> {
        let source = ctx.effect_source();
        self.owner_of(source).ok_or_else(|| format!("{source} is gone, and nothing kept its last known information to say who owned it"))
    }

    /// The player an instruction names as it is applied (CR 608.2d): who
    /// puts the counters on for `Primitive::AddCounters` and `GetCounters`
    /// (CR 122.6a's shape on a proposal), who chooses for `Effect::Optional`,
    /// and who gains control for `GainControl`. "An opponent" it does not
    /// target is its controller's choice among the opponents still in the
    /// game, asked only when there are two or more (Akroan Horse's ruling).
    /// Every other reference, and a target, reads as
    /// [`Self::resolve_player_ref`] reads it.
    fn player_the_instruction_names(
        &self,
        who: &PlayerRef,
        targets: &[ObjectOrPlayerId],
        ctx: &ResolutionContext,
        dp: &dyn DecisionProvider,
    ) -> Result<PlayerId, String> {
        if matches!(who, PlayerRef::Opponent) && targeted_opponent(targets, ctx).is_none() {
            let opponents = self.opponents_of(ctx.controller);
            if opponents.len() > 1 {
                return Ok(crate::ui::ask::ask_choose_opponent(dp, self, ctx.controller, ctx.effect_source(), &opponents));
            }
        }
        self.resolve_player_ref(who, targets, ctx)
    }

    /// `You` is the effect's controller — every printed one-shot, and the
    /// card writes it. `Opponent` is the resolution's player target when it
    /// has one, else the only opponent still in the game; with several and
    /// no target an instruction asks ([`Self::player_the_instruction_names`]),
    /// and a read, which cannot ask, is loud. Bold Plagiarist's
    /// "*they* put" is the printed customer, a trigger whose effect names the
    /// player who triggered it — `Player(id)` once CR 603 fills it.
    fn resolve_player_ref(
        &self,
        by: &PlayerRef,
        targets: &[ObjectOrPlayerId],
        ctx: &ResolutionContext,
    ) -> Result<PlayerId, String> {
        Ok(match by {
            PlayerRef::You => ctx.controller,
            PlayerRef::Player(pid) => *pid,
            PlayerRef::Owner => self.source_owner_of(ctx)?,
            PlayerRef::Opponent => {
                match targeted_opponent(targets, ctx) {
                    Some(pid) => pid,
                    None => {
                        let opponents = self.opponents_of(ctx.controller);
                        match opponents.as_slice() {
                            [only] => *only,
                            others => {
                                return Err(format!(
                                    "a read names \"an opponent\" it does not target, and player {} has {} \
                                     opponents in the game: the choice is the instruction's (CR 608.2d), \
                                     which a read cannot ask",
                                    ctx.controller,
                                    others.len()
                                ))
                            }
                        }
                    }
                }
            }
            PlayerRef::DefendingPlayer(creature) => self.defending_player_of(*creature, ctx).ok_or_else(|| {
                format!("{:?} on {:?} names \"defending player\", and its creature attacks nothing", by, ctx.source)
            })?,
            // CR 802.2a: with two or more, which one is the controller's choice.
            PlayerRef::AnyDefendingPlayer => {
                let defending: Vec<PlayerId> =
                    self.players_in_game_apnap().into_iter().filter(|&player| self.is_defending_player(player)).collect();
                match defending.as_slice() {
                    [only] => *only,
                    others => {
                        return Err(format!(
                            "{:?} on {:?} names \"defending player\" with no creature to settle which, and \
                             there are {} defending players: the controller's choice (CR 802.2a) is not asked here",
                            by,
                            ctx.source,
                            others.len()
                        ))
                    }
                }
            }
        })
    }

    /// The players `group` names, over the seats still in the game, each once,
    /// in APNAP order (CR 101.4): the active player first, then the rest in
    /// turn order.
    fn players_in(&self, group: &PlayerGroup, ctx: &ResolutionContext) -> Vec<PlayerId> {
        let named = match group.named {
            NamedPlayers::Nobody => None,
            NamedPlayers::FirstInstance => ctx.targets.instance(0).iter().find_map(|t| match t {
                ObjectOrPlayerId::Player(pid) => Some(*pid),
                ObjectOrPlayerId::Object(_) => None,
            }),
        };
        let in_group = |player: PlayerId| group.relation.contains(ctx.controller, player) || named == Some(player);
        self.players_in_game_apnap().into_iter().filter(|&p| in_group(p)).collect()
    }

    /// A resolving effect's own "if" (CR 608.2c). CR 118.12's answer is the
    /// walk's to give, inside `All` too; every other leaf is the board's.
    fn resolution_condition_holds(
        &self,
        condition: &Condition,
        ctx: &ResolutionContext,
        walk: &ResolutionWalk,
    ) -> bool {
        match condition {
            Condition::CostAnswer(answer) => walk.last_cost_answer == Some(*answer),
            Condition::All(clauses) => clauses.iter().all(|c| self.resolution_condition_holds(c, ctx, walk)),
            // CR 109.5 — a resolving spell's or ability's "you" is its
            // controller, whatever has become of its source since; and the
            // source and "it" are read as they last existed once they have
            // left (CR 113.7a, 608.2h).
            other => {
                let source = ctx.effect_source();
                let objects = self.condition_objects(ctx.ability_source, ctx.trigger.as_ref());
                crate::engine::layers::condition::settled_holds(other, self, source, Some(ctx.controller), objects)
            }
        }
    }

    /// "The player or planeswalker it's attacking" (CR 506.3): what
    /// `creature` is attacking, read as CR 608.2h reads it. A creature still
    /// on the battlefield answers with its attack now, and nothing once it has
    /// left combat; one that left the battlefield answers with the attack it
    /// left with (Fathom Fleet Swordjack's ruling).
    fn attacked_by(&self, creature: Antecedent, ctx: &ResolutionContext) -> Vec<ObjectOrPlayerId> {
        let objects = self.condition_objects(ctx.ability_source, ctx.trigger.as_ref());
        let attack = objects.object_for(creature, ctx.effect_source()).and_then(|it| it.attacking(self));
        match attack {
            Some(AttackTarget::Player(player)) => vec![ObjectOrPlayerId::Player(player)],
            Some(AttackTarget::Planeswalker(id) | AttackTarget::Battle(id)) => vec![ObjectOrPlayerId::Object(id)],
            None => Vec::new(),
        }
    }

    /// CR 508.4: what a creature this resolution puts onto the battlefield
    /// attacking may attack, read as it resolves, since only the resolution
    /// holds the binding that "that player" and "the same as that creature"
    /// name: everything that can be attacked, for its controller's choice;
    /// the named player's that `among` matches; or what the other creature is
    /// attacking, or was as it left (CR 702.49c). The entry's performer
    /// checks the candidate again as the creature enters (CR 508.4a).
    fn enters_attacking_candidates(&self, word: &EntersAttacking, ctx: &ResolutionContext) -> Result<EntersAttackingCandidates, String> {
        let candidates = match word {
            EntersAttacking::Chosen => self.what_can_be_attacked(),
            // "That player or a planeswalker they control": the named player's
            // defenses that `among` admits, with "you" the controller's and
            // each filter read from the source.
            EntersAttacking::Player { player, among } => match self.named_player(player, ctx)? {
                Some(named) => {
                    let names_player = |who: &PlayerRef, player: PlayerId| self.resolve_player_ref(who, &[], ctx).ok() == Some(player);
                    let matches_permanent = |filter: &ObjectFilter, id: ObjectId| {
                        self.object_matches_filter_of_source(ObjectInformation::Current(id), filter, ctx.controller, ctx.effect_source())
                    };
                    self.what_can_be_attacked()
                        .into_iter()
                        .filter(|&target| {
                            defending_player_for(self, target) == Some(named) && among.admits(target, &names_player, &matches_permanent)
                        })
                        .collect()
                }
                None => Vec::new(),
            },
            EntersAttacking::SameAs(creature) => {
                let objects = self.condition_objects(ctx.ability_source, ctx.trigger.as_ref());
                objects.object_for(*creature, ctx.effect_source()).and_then(|it| it.attacking(self)).into_iter().collect()
            }
        };
        Ok(EntersAttackingCandidates { candidates })
    }

    /// The one player `recipient` names as this resolution reads it, for an
    /// instruction that names a player beside its own recipient ("attacking
    /// that player"). Loud for a recipient that names no one player: "each
    /// opponent" names several, and an instance of "target" is the atom's own.
    fn named_player(&self, recipient: &EffectRecipient, ctx: &ResolutionContext) -> Result<Option<PlayerId>, String> {
        match (recipient, self.players_named_by(recipient, ctx)?) {
            (EffectRecipient::EachOf(_), _) | (_, None) => {
                Err(format!("{recipient:?} names no one player to attack, on {:?}", ctx.source))
            }
            (_, Some(players)) => Ok(players.first().copied()),
        }
    }

    /// The players `recipient` names from the resolution's context alone:
    /// "you", "each opponent" in APNAP order (CR 101.4), "defending player",
    /// "that player". `None` for a recipient that names objects, or players
    /// only through an instance of "target", which the walk reads.
    fn players_named_by(&self, recipient: &EffectRecipient, ctx: &ResolutionContext) -> Result<Option<Vec<PlayerId>>, String> {
        Ok(Some(match recipient {
            EffectRecipient::Implicit | EffectRecipient::Controller => vec![ctx.controller],
            EffectRecipient::EachOf(group) => self.players_in(group, ctx),
            EffectRecipient::DefendingPlayer(creature) => self.defending_player_of(*creature, ctx).into_iter().collect(),
            EffectRecipient::TriggeringPlayer | EffectRecipient::Triggering(_) | EffectRecipient::Referred => self
                .bound_targets(recipient, ctx)?
                .into_iter()
                .filter_map(|target| match target {
                    ObjectOrPlayerId::Player(player) => Some(player),
                    ObjectOrPlayerId::Object(_) => None,
                })
                .collect(),
            EffectRecipient::ThisObject
            | EffectRecipient::Target(..)
            | EffectRecipient::Choose(..)
            | EffectRecipient::SameInstanceAs(_)
            | EffectRecipient::Group(_)
            | EffectRecipient::AttackedBy(_)
            | EffectRecipient::FilteredPermanents(_)
            | EffectRecipient::FilteredObjectsIn(..)
            | EffectRecipient::Host
            | EffectRecipient::ChosenBy(_) => return Ok(None),
        }))
    }

    /// "Defending player" for `creature` (CR 508.5), read as `attacked_by`
    /// reads it.
    fn defending_player_of(&self, creature: Antecedent, ctx: &ResolutionContext) -> Option<PlayerId> {
        let objects = self.condition_objects(ctx.ability_source, ctx.trigger.as_ref());
        objects.object_for(creature, ctx.effect_source())?.defending_player(self, &|_| {})
    }

    /// CR 113.7a, 608.2h — the ability's source's last known information,
    /// once it has left the zone it was in, as the resolving object reads it
    /// (`lki_as_resolving_reads`): the creature that died for a trigger
    /// on its own death. `None` while the source is there, and for a spell,
    /// which resolves where it was cast. A source nothing framed reads where
    /// it is now.
    fn source_lki(&self, ctx: &ResolutionContext) -> Option<Lki> {
        self.lki_as_resolving_reads(ctx.ability_source?).cloned()
    }

    /// What `Effect::Remember`'s instruction acted on, as it left them (CR
    /// 603.7c, 400.7). A verb that moves things: each object its records put
    /// into a public zone, and each token it made in one, if it is still
    /// there (`put_into_since`), so a move a replacement sent elsewhere (Rest
    /// in Peace's exile) is still "that card". A verb that moves nothing: each
    /// object it acted on, as it is. Either way, each player it named.
    fn remembered_since(
        &self,
        primitive: &Primitive,
        acted_on: &[ObjectOrPlayerId],
        ctx: &ResolutionContext,
        mark: crate::events::event::EventId,
    ) -> Referred {
        let players = acted_on
            .iter()
            .filter_map(|t| match t {
                ObjectOrPlayerId::Player(pid) => Some(*pid),
                ObjectOrPlayerId::Object(_) => None,
            })
            .collect();
        let objects = match primitive.moves_into() {
            Some(_) => self
                .put_into_since(ctx.stamp(), mark)
                .into_iter()
                .map(|(object, _)| RememberedObject::now(object))
                .collect(),
            None => {
                let mut objects: Vec<RememberedObject> = Vec::new();
                for t in acted_on {
                    if let ObjectOrPlayerId::Object(id) = t
                        && let Some(object) = self.object_ref(*id)
                        && !objects.iter().any(|r| r.object == object)
                    {
                        objects.push(RememberedObject::now(object));
                    }
                }
                objects
            }
        };
        Referred { objects, players }
    }

    /// What resolution `stamp` put into a public zone from `mark` on and is
    /// still there, once each: every object it moved, as the object its move
    /// made (CR 400.7e), with the zone it came from, and every token it
    /// created, which came from no zone (CR 111.2). Wherever the move went,
    /// a replacement's destination included, the effect can find it (CR
    /// 400.7j; 701.17c for a mill; Rescue from the Underworld's fourth
    /// ruling), and in a hand or a library it can't.
    pub(crate) fn put_into_since(
        &self,
        stamp: ResolutionStamp,
        mark: crate::events::event::EventId,
    ) -> Vec<(ObjectRef, Option<Zone>)> {
        let mut put: Vec<(ObjectRef, Option<Zone>)> = Vec::new();
        for record in self.events.resolution_records(stamp, mark) {
            let moved = match &record.event {
                GameEvent::ZoneChange { from, to, .. } => record.event.new_object().map(|o| (o, Some(*from), *to)),
                GameEvent::TokenCreated { token, zone, .. } => Some((*token, None, *zone)),
                _ => None,
            };
            if let Some((object, from, to)) = moved
                && to.is_public()
                && self.find(object).is_some_and(|o| o.zone == to)
                && !put.iter().any(|(seen, _)| *seen == object)
            {
                put.push((object, from));
            }
        }
        put
    }

    /// Note that the triggered ability resolving for `ctx` read `fact` of its
    /// entry, which its def must name (`stack::binding_reads_are_named`); a
    /// debug build's check, so a release build notes nothing.
    pub(crate) fn note_binding_read(&self, ctx: &ResolutionContext, fact: TriggerBindingReads) {
        #[cfg(debug_assertions)]
        if let Some(resolving) = self.resolving_for(ctx) {
            resolving.binding_reads_noted.set(resolving.binding_reads_noted.get() | fact);
        }
        #[cfg(not(debug_assertions))]
        let _ = (ctx, fact);
    }

    /// The stack object resolving for `ctx`; a CR 615.5 rider resolves none.
    fn resolving_for(&self, ctx: &ResolutionContext) -> Option<&crate::state::game_state::ResolvingObject> {
        self.resolving.as_ref().filter(|resolving| resolving.id == ctx.source)
    }

    /// CR 113.7a's "this [object]" for a resolution: the ability's source,
    /// found by identity (CR 400.7), else the spell or replacement source itself
    /// while it is still where the effect found it. `None` is the object
    /// being gone, which the primitive meets as an empty target slice.
    pub(crate) fn this_object(&self, ctx: &ResolutionContext) -> Option<ObjectId> {
        self.note_binding_read(ctx, TriggerBindingReads::SOURCE);
        match ctx.ability_source {
            // Or, for an Aura's trigger on its enchanted permanent leaving, the
            // card the Aura became in its owner's graveyard (CR 400.7f).
            Some(source) => self.find(source).map(|_| source.id).or_else(|| {
                let found = ctx.trigger.as_ref()?.source_found_in_graveyard?;
                self.find(found).map(|_| found.id)
            }),
            None => {
                let id = ctx.source;
                let here = self.battlefield.contains_key(&id)
                    || self.stack_entries.contains_key(&id)
                    || self.resolving_for(ctx).is_some();
                here.then_some(id)
            }
        }
    }

    /// The players a player verb acts on: each of an `EachOf` group, which
    /// `players_in` put in the targets in APNAP order, or else the one the
    /// recipient names.
    fn each_player_acted_on(&self, recipient: &EffectRecipient, targets: &[ObjectOrPlayerId], ctx: &ResolutionContext) -> Vec<PlayerId> {
        match recipient {
            EffectRecipient::EachOf(_) => targets
                .iter()
                .filter_map(|t| match t {
                    ObjectOrPlayerId::Player(pid) => Some(*pid),
                    ObjectOrPlayerId::Object(_) => None,
                })
                .collect(),
            _ => vec![self.resolve_player_for_self(recipient, targets, ctx)],
        }
    }

    fn resolve_player_for_self(
        &self,
        recipient: &EffectRecipient,
        targets: &[ObjectOrPlayerId],
        ctx: &ResolutionContext,
    ) -> PlayerId {
        match recipient {
            EffectRecipient::Implicit | EffectRecipient::Controller => ctx.controller,
            EffectRecipient::Target(SelectionFilter::Player, _)
            | EffectRecipient::TriggeringPlayer
            | EffectRecipient::Referred
            | EffectRecipient::DefendingPlayer(_) => {
                for t in targets {
                    if let ObjectOrPlayerId::Player(pid) = t {
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
/// "An opponent" the resolution targets, when it targets one: a player
/// target who is not its controller.
fn targeted_opponent(targets: &[ObjectOrPlayerId], ctx: &ResolutionContext) -> Option<PlayerId> {
    targets.iter().find_map(|t| match t {
        ObjectOrPlayerId::Player(pid) if *pid != ctx.controller => Some(*pid),
        _ => None,
    })
}

fn back_reference(recipient: &EffectRecipient, ctx: &ResolutionContext) -> String {
    format!(
        "a primitive on {:?} was handed {:?}, a back-reference to an instance of \"target\" rather than the clause that declared it (CR 115.3). Resolve it through `targeting::instance_of`.",
        ctx.source, recipient
    )
}

#[cfg(test)]
mod tests {
    /// The X announced for a count of targets, before anything resolves, is
    /// read through the evaluator's own arithmetic (CR 601.2b-c); with
    /// nothing announced and nothing resolving there is no X.
    #[test]
    fn an_announced_x_is_read_through_the_amounts_arithmetic() {
        let game = GameState::new(2, 20);
        let ctx = ResolutionContext::untargeted(crate::types::ids::ObjectId::UNASSIGNED, 0, GeneratedBy::Spell);
        let twice_x_plus_one = AmountExpr::Plus(Box::new(AmountExpr::Multiply(Box::new(AmountExpr::X), 2)), 1);
        assert_eq!(game.evaluate_amount(&twice_x_plus_one, &ctx, Some(3)), Ok(7));
        assert!(game.evaluate_amount(&AmountExpr::X, &ctx, None).is_err());
    }

    /// CR 615.5's two numbers are a rider's: outside one both leaves refuse.
    /// Inside one, each refuses an event that lacks its number, saying so
    /// rather than "outside a rider", and damage that was not prevented reads 0.
    #[test]
    fn a_riders_numbers_are_read_only_inside_a_rider() {
        let game = GameState::new(2, 20);
        let outside = ResolutionContext::untargeted(crate::types::ids::ObjectId::UNASSIGNED, 0, GeneratedBy::Spell);
        let read = |expr: AmountExpr, ctx: &ResolutionContext| game.evaluate_amount(&expr, ctx, None);
        let inside = |event: ReplacedEvent| ResolutionContext { replaced_event: Some(event), ..outside.clone() };
        assert!(read(AmountExpr::ReplacedAmount, &outside).is_err_and(|e| e.contains("outside a CR 615.5 rider")));
        assert!(read(AmountExpr::DamagePrevented, &outside).is_err_and(|e| e.contains("outside a CR 615.5 rider")));

        let zone_change = inside(ReplacedEvent::NoAmount);
        assert!(read(AmountExpr::ReplacedAmount, &zone_change).is_err_and(|e| e.contains("carried no amount")));
        assert!(read(AmountExpr::DamagePrevented, &zone_change).is_err_and(|e| e.contains("was not damage")));

        let draw = inside(ReplacedEvent::Amount(2));
        assert_eq!(read(AmountExpr::ReplacedAmount, &draw), Ok(2));
        assert!(read(AmountExpr::DamagePrevented, &draw).is_err_and(|e| e.contains("was not damage")));

        let unpreventable = inside(ReplacedEvent::Damage { amount: 3, prevented: 0 });
        assert_eq!(read(AmountExpr::ReplacedAmount, &unpreventable), Ok(3));
        assert_eq!(read(AmountExpr::DamagePrevented, &unpreventable), Ok(0));
    }
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

    fn bolt_ctx(source: ObjectId, targets: Vec<ObjectOrPlayerId>) -> ResolutionContext {
        ResolutionContext {
            source,
            ability_source: None,
            controller: 0,
            generated_by: crate::events::cause::GeneratedBy::Spell,
            targets: ChosenTargets::one(targets),
            replaced_event: None,
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

        let ctx = bolt_ctx(bears_id, vec![ObjectOrPlayerId::Object(bears_id)]);
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

        let ctx = bolt_ctx(bears_id, vec![ObjectOrPlayerId::Player(1)]);
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

        let ctx = bolt_ctx(bears_id, vec![ObjectOrPlayerId::Player(1)]);
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
        let ctx = bolt_ctx(source_id, vec![ObjectOrPlayerId::Object(target_id)]);
        game.resolve_effect(&destroy, &ctx, &test_dp()).unwrap();

        // Creature should still be on the battlefield
        assert!(game.battlefield.contains_key(&target_id));
    }
}

