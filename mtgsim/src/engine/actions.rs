use crate::engine::keywords::{apply_deathtouch_flag, apply_lifelink};
use crate::engine::resolve::ResolutionContext;
use crate::events::event::{DamageTarget, GameEvent, ResolutionStamp};
use crate::state::game_state::GameState;
use crate::types::effects::CounterType;
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::replacement::EnterMods;
use crate::types::zones::Zone;
use crate::ui::decision::DecisionProvider;

/// Re-exported for every existing reader of `engine::actions::ZoneChangeCause`.
///
/// The definition moved to `types::zones` in Phase RB, alongside `Zone`, which
/// is the vocabulary it qualifies. The mover is `EventPattern::ZoneChange`:
/// a replacement effect watching "would be put into a graveyard from the
/// battlefield" has to name the cause, `EventPattern` lives in `types`, and
/// `src/types/` has no `crate::engine` edge to spend.
pub use crate::types::zones::{DestructionSource, ZoneChangeCause};

/// Who is asking for a mutation, and what resolution it belongs to.
///
/// `execute_action` has no `DecisionProvider` of its own, and CR 616.1 needs
/// one: when two or more replacement effects want the same event, *the affected
/// object's controller* chooses which to apply — not the controller of the
/// effect. Rather than thread a bare `&dyn DecisionProvider`, this carries the
/// second thing the pipeline will want, so the plumbing is paid for once.
///
/// **Phase RA threads it; nothing reads either field yet.** Phase RB is where
/// `apply_replacements` starts consulting them:
///
/// - `dp` answers the CR 616.1 ordering prompt.
/// - `resolution` is where CR 614.15 self-replacement effects live (they belong
///   to the resolving spell or ability, not to any registry), and it is what
///   stamps every emitted `GameEvent` with the resolution that caused it.
pub struct ActionContext<'a> {
    pub dp: &'a dyn DecisionProvider,
    pub resolution: Option<&'a ResolutionContext>,
}

impl<'a> ActionContext<'a> {
    /// A mutation that belongs to no resolution: a turn-based action, a
    /// state-based action, cost payment, combat damage.
    pub fn new(dp: &'a dyn DecisionProvider) -> Self {
        ActionContext { dp, resolution: None }
    }

    /// A mutation proposed by a resolving spell or ability.
    pub fn resolving(dp: &'a dyn DecisionProvider, resolution: &'a ResolutionContext) -> Self {
        ActionContext { dp, resolution: Some(resolution) }
    }

    /// The resolution to stamp onto every event this proposal emits.
    ///
    /// Drops the targets: an event log wants to know *which resolution* caused a
    /// mutation, and the resolving object's id answers that. **That is a
    /// property of this engine, not a rule** — a cast produces one stack object
    /// and `activate_ability` mints a fresh ephemeral one per activation, each
    /// with its own v4 `ObjectId`, and CR 608.2n destroys the ability's object
    /// rather than recycling it. Carrying the target list would copy it onto
    /// every event for no reader.
    pub(crate) fn resolution_stamp(&self) -> Option<ResolutionStamp> {
        self.resolution.map(|r| ResolutionStamp {
            source: r.source,
            controller: r.controller,
        })
    }
}

/// A game action that is *about to happen*.
///
/// This is the pre-mutation counterpart to `GameEvent` (which records what
/// *did* happen). The engine builds a `GameAction`, passes it through
/// `execute_action`, which performs the mutation and emits the corresponding
/// `GameEvent`.
///
/// In Phase 6, a replacement-effect pipeline will sit between "build action"
/// and "execute action", potentially modifying or replacing the action before
/// it is carried out. For now, `execute_action` is a direct passthrough.
#[derive(Debug, Clone)]
pub enum GameAction {
    /// Deal damage from a source to a target.
    DealDamage {
        source: ObjectId,
        target: DamageTarget,
        amount: u64,
        is_combat: bool,
    },

    /// A single card draw for a player.
    ///
    /// Drawing N cards is N individual `DrawCard` actions (rule 121.2).
    DrawCard {
        player: PlayerId,
    },

    /// A player gains life.
    GainLife {
        player: PlayerId,
        amount: u64,
        source: ObjectId,
    },

    /// A player loses life (not from damage).
    LoseLife {
        player: PlayerId,
        amount: u64,
    },

    /// Move an object from one zone to another.
    ///
    /// `cause` is set by the caller and is required — see [`ZoneChangeCause`].
    ZoneChange {
        object: ObjectId,
        from: Zone,
        to: Zone,
        cause: ZoneChangeCause,
    },

    /// Untap a permanent.
    Untap {
        object: ObjectId,
    },

    /// Tap a permanent.
    Tap {
        object: ObjectId,
    },

    /// Put counters on a permanent (CR 122.1).
    ///
    /// A proposal rather than a direct write because CR 614.16's counter
    /// doublers replace it — "If one or more counters would be put on a
    /// permanent you control, twice that many are put on it instead" — and
    /// because CR 122.1c/d's own replacement effects have to be able to *make*
    /// one: a stun counter's effect is literally "instead remove a stun counter
    /// from it", which is a proposed event and not bookkeeping.
    AddCounters {
        object: ObjectId,
        counter: CounterType,
        n: u32,
    },

    /// Take counters off a permanent (CR 122.1).
    ///
    /// The substituted event for CR 122.1c's shield counter and CR 122.1d's
    /// stun counter, and the CR 615.5 rider for the shield's prevention half.
    ///
    /// `n` is a maximum: removing three counters from a permanent that has one
    /// removes one, which is CR 701.2's "as much as it can" and what
    /// `BattlefieldEntity::remove_counters` already reports.
    RemoveCounters {
        object: ObjectId,
        counter: CounterType,
        n: u32,
    },

    /// Destroy a permanent (CR 701.8).
    ///
    /// **The outer event.** Performing it proposes an inner
    /// `ZoneChange { to: Graveyard }` whose cause is
    /// [`DestructionSource::zone_change_cause`], so a destruction is two events
    /// and a replacement can watch either. The CR draws that line itself: CR
    /// 122.1c's shield counter replaces "would be destroyed", CR 122.1h's
    /// finality counter replaces "would be put into a graveyard from the
    /// battlefield". The two overlap rather than nest — CR 701.8b keeps a
    /// sacrifice out of the first, and a destruction whose graveyard move is
    /// itself replaced never reaches the second. One event cannot answer both.
    ///
    /// Indestructible is **not** checked here and is not a replacement effect.
    /// CR 702.12b makes it a "can't" (CR 614.17), which is checked ahead of the
    /// pipeline and wins — see `engine::replacement::is_blocked`.
    Destroy {
        object: ObjectId,
        source: DestructionSource,
    },

    /// A permanent enters the battlefield (CR 614.1c/d).
    ///
    /// **A separate event from the `ZoneChange` that carries it**, proposed
    /// after it rather than folded into it. The CR asks two different questions
    /// at two different instants: a zone-change replacement rewrites *where the
    /// object goes* (CR 614.8's "would be put into a graveyard" — Kalitas, a
    /// finality counter), while an entry replacement rewrites *how it arrives*
    /// once the destination is settled (CR 614.12's "check the characteristics
    /// of the permanent as it would exist on the battlefield"). One action for
    /// both would let an ETB replacement see a destination a later zone-change
    /// replacement then changed.
    ///
    /// `controller` is CR 110.2b's **default**: the owner for a land drop or a
    /// token, the player who put the spell on the stack for a resolving
    /// permanent spell. It is the value Layer 2 modifies, not the answer
    /// `get_effective_controller` gives.
    ///
    /// `mods` starts as whatever the *rules* say the permanent enters with
    /// (`GameState::default_enter_mods`) and accumulates through
    /// [`Rewrite::EnterWith`](crate::types::replacement::Rewrite::EnterWith)
    /// as CR 616.1f iterates.
    EnterBattlefield {
        object: ObjectId,
        controller: PlayerId,
        mods: EnterMods,
    },

    // === Phase 3+ actions — add variants here as primitives are implemented ===
    // Sacrifice { object: ObjectId },
    // Exile { object: ObjectId },
    // CreateTokens { defs: Vec<TokenDef>, controller: PlayerId },
    // etc.
}

impl GameState {
    /// Execute a game action: mutate state and emit the corresponding event.
    ///
    /// This is the central chokepoint for all game-state mutations that are
    /// observable (i.e., that triggered abilities and replacement effects care
    /// about).
    ///
    /// **Current behavior (through Phase RA):** direct passthrough — performs
    /// the mutation immediately and emits the event. `ctx` is threaded but not
    /// yet read; RA's job is to make sure it is *available* everywhere a
    /// mutation happens.
    ///
    /// **Phase RB:** an `apply_replacements(action, ctx, ...)` call goes in
    /// between, potentially modifying or dropping the action before execution.
    /// The replacement pipeline handles rule 614 (replacement effects),
    /// rule 615 (prevention effects), and rule 616 (interaction ordering).
    pub fn execute_action(
        &mut self,
        action: GameAction,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        self.execute_actions(vec![action], ctx).map(|_| ())
    }

    /// Execute a set of actions as **one event** (CR 704.3, 510.2, 502.1).
    ///
    /// Three rules need an event *set* rather than an event, and none of them
    /// is reachable from a loop of `execute_action` calls:
    ///
    /// - **CR 704.3** — state-based actions are performed "simultaneously as a
    ///   single event".
    /// - **CR 704.7** — "if multiple state-based actions would have the same
    ///   result at the same time, a single replacement effect will replace all
    ///   of them". That is a same-result dedupe *on the batch*, upstream of the
    ///   pipeline, and it needs the whole set in hand.
    /// - **CR 615.7** — one prevention shield facing several simultaneous
    ///   damage sources: its controller chooses which damage it prevents, and
    ///   the choice cannot exist unless all of the damage is proposed at once.
    ///
    /// Every event the batch emits carries one [`BatchId`](crate::events::event::BatchId),
    /// which is what **CR 603.2c** needs: "an ability triggers only once each
    /// time its trigger event occurs. However, it can trigger repeatedly if one
    /// event contains multiple occurrences." Both readings hang off the batch
    /// boundary. "Whenever one or more creatures die" has the whole batch as its
    /// trigger event and fires once; "whenever a creature dies" fires once per
    /// occurrence inside it. Without a boundary the engine cannot tell them
    /// apart.
    ///
    /// **Each member keeps its own applied set** when the pipeline lands.
    /// CR 614.5 is per *event* and batch members are separate events: Kalitas
    /// dying alongside several opponent creatures exiles every one of them from
    /// one static replacement, applied once per death. Nothing here has to
    /// arrange that, but nothing here may assume otherwise
    /// (`replacement-architecture.md` §4.2).
    ///
    /// **Routing a sweep through here makes its order observable.** CR 616.1
    /// prompts when two effects want one event, so the order the batch is built
    /// in is part of a decision — build it from `battlefield_ids_ordered`, never
    /// from a raw `HashMap` walk.
    /// Returns the events that were actually performed, in batch order.
    ///
    /// Not the proposals: a CR 614 replacement can modify an event or drop it
    /// entirely, so the returned vector is what the game saw. The customer is
    /// the pipeline itself — `Primitive::Regenerate` and the SBA sweep both
    /// need to know whether the thing they proposed survived.
    pub fn execute_actions(
        &mut self,
        batch: Vec<GameAction>,
        ctx: &ActionContext,
    ) -> Result<Vec<GameAction>, String> {
        let previous = self.events.open_batch(ctx.resolution_stamp());
        let result = self.execute_batch_inner(batch, ctx);
        self.events.close_batch(previous);
        result
    }

    /// The three phases of one batch. Split out so `close_batch` runs on every
    /// exit path, including the error ones.
    ///
    /// **Deciding is separated from performing, and that is CR 704.3.** "The
    /// game checks for any of the listed conditions ... then performs all
    /// applicable state-based actions simultaneously as a single event" — so
    /// every member's replacements are chosen against one board, the board as
    /// it was before any of them happened. It is also what CR 614.4 wants for a
    /// simultaneous event: the effect must exist before *the* event, and the
    /// event is the whole batch.
    fn execute_batch_inner(
        &mut self,
        batch: Vec<GameAction>,
        ctx: &ActionContext,
    ) -> Result<Vec<GameAction>, String> {
        use crate::engine::replacement::{apply_replacements, Rider};

        // --- Phase 1: decide (CR 616.1), in APNAP order of chooser ----------
        //
        // CR 616.1's last sentence: "if two or more players have to make these
        // choices at the same time, choices are made in APNAP order (see rule
        // 101.4)". A batch whose members affect different players produces
        // different choosers, and this is the only place that can order them.
        //
        // CR 101.4d's restart — a nonactive player's choice forcing an
        // earlier player to choose again — is not implemented. It has not come
        // up for a narrower reason than "unreachable": nothing in phase 1 can
        // *create* a replacement effect, and each member's 616.1f loop runs to
        // completion before the next begins. That is a simplification, not a
        // proof; interleaving the members is deferred (rb-review F6).
        let mut riders: Vec<Rider> = Vec::new();
        let mut decided: Vec<Option<GameAction>> = vec![None; batch.len()];
        let inherited = std::collections::HashSet::new();
        for index in self.apnap_batch_order(&batch) {
            decided[index] = apply_replacements(
                self,
                batch[index].clone(),
                ctx,
                &inherited,
                &mut riders,
            )?;
        }

        // --- Phase 2: perform, in batch order -------------------------------
        //
        // Batch order rather than APNAP order: the choices were the thing
        // CR 101.4 sequences, and the performed events are simultaneous. The
        // order they are written in is still observable (a graveyard is
        // ordered), and it is the caller's `battlefield_ids_ordered` sweep.
        let mut performed = Vec::with_capacity(decided.len());
        for action in decided.into_iter().flatten() {
            self.perform_action(action.clone(), ctx)?;
            performed.push(action);
        }

        // --- Phase 3: the queued riders, in application order ----------------
        //
        // "The rest of the effect takes place immediately afterward", and
        // afterward means after the events happened — not mid-loop, where
        // nothing has happened yet. Unconditional once queued (CR 615.12), so
        // this runs even for a member whose event was dropped entirely.
        for rider in riders {
            self.resolve_rider(rider, ctx)?;
        }

        Ok(performed)
    }

    /// Indices into `batch`, ordered active-player-first by the CR 616.1
    /// chooser for each member (CR 101.4's APNAP).
    ///
    /// Stable within a player, so a sweep that built its batch from
    /// `battlefield_ids_ordered` keeps that order among its own members.
    fn apnap_batch_order(&self, batch: &[GameAction]) -> Vec<usize> {
        use crate::engine::replacement::chooser_for_event;

        let n = self.players.len();
        let mut order: Vec<usize> = (0..batch.len()).collect();
        order.sort_by_key(|&i| {
            let chooser = chooser_for_event(self, &batch[i]);
            // `None` — an object with neither controller nor owner — sorts
            // last; the pipeline errors on it rather than guessing, and this
            // keeps that error deterministic.
            match chooser {
                Some(p) => self.apnap_index(p),
                None => n,
            }
        });
        order
    }

    /// Resolve one queued rider — see [`Rider`](crate::engine::replacement::Rider)
    /// for which rule gives it this timing, which depends on whether a
    /// prevention effect queued it (CR 615.5) or a plain replacement did
    /// (CR 614.1a/614.6).
    ///
    /// Its `ResolutionContext` names the event's subject as the single resolved
    /// target, so a `then` written with `EffectRecipient::Target` acts on the
    /// permanent the replacement was about and one written with
    /// `EffectRecipient::Controller` acts for that permanent's controller. The
    /// actions it proposes re-enter the pipeline with a **fresh** applied set —
    /// a rider's actions are new events the replacement caused, not modified
    /// forms of the original (§3.2d containment).
    fn resolve_rider(
        &mut self,
        rider: crate::engine::replacement::Rider,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        use crate::engine::resolve::{ResolutionContext, ResolvedTarget};

        let rctx = ResolutionContext {
            source: rider.source,
            controller: rider.controller,
            targets: rider
                .subject
                .map(|id| vec![ResolvedTarget::Object(id)])
                .unwrap_or_default(),
        };
        self.resolve_effect(&rider.effect, &rctx, ctx.dp)
    }

    /// Convenience wrapper for the most common zone change: caller knows the
    /// destination but doesn't want to hand-roll the `from` lookup.
    ///
    /// This is the intended public path for zone changes. Routes through
    /// `execute_action(GameAction::ZoneChange)` so the replacement pipeline
    /// (CR 614) sees every movement. `draw_card` and `play_land` route through
    /// here too, which is why the one-emitter invariant holds and why a draw
    /// from an empty library reaches the pipeline at all (CR 121.6a). Two
    /// production callers sit below the chokepoint and no more:
    /// `perform_action`'s own `ZoneChange` arm, and `rollback_cast_to_hand` —
    /// the permanent `// CAST-ROLLBACK:` exemption, because a CR 601.2 rewind
    /// is not an event.
    pub fn change_zone(
        &mut self,
        object: ObjectId,
        to: Zone,
        cause: ZoneChangeCause,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        let from = self.get_object(object)?.zone;
        self.execute_action(GameAction::ZoneChange { object, from, to, cause }, ctx)
    }

    /// Perform the actual state mutation and emit the event.
    ///
    /// This is separated from `execute_action` so that the replacement pipeline
    /// (Phase RB) can call this with the final, possibly-modified action.
    ///
    /// `_ctx` is the one place in the RA sweep where the context is threaded but
    /// has nothing to read yet. It is a parameter here rather than absent
    /// because this is where it is used *first*: RA-2 routes lifelink's life
    /// gain through `execute_action`, and that proposal is made from inside the
    /// `DealDamage` arm below.
    fn perform_action(
        &mut self,
        action: GameAction,
        _ctx: &ActionContext,
    ) -> Result<(), String> {
        match action {
            GameAction::DealDamage { source, target, amount, is_combat } => {
                // No 0-amount guard here: CR 614.7a says a 0-damage event never
                // happens, which makes it the *proposal's* problem — a 0 that
                // reaches the CR 616.1 loop is one a prevention effect applies
                // to. `replacement::never_happens` owns the rule, ahead of the
                // pipeline, and this function's only caller runs it.
                match &target {
                    DamageTarget::Object(id) => {
                        if let Some(entry) = self.battlefield.get_mut(id) {
                            entry.damage_marked += amount as u32;
                        } else {
                            return Err(format!(
                                "Target object {} not on battlefield", id
                            ));
                        }
                    }
                    DamageTarget::Player(pid) => {
                        let player = self.get_player_mut(*pid)?;
                        player.life_total -= amount as i64;
                    }
                }

                // Keyword hooks (delegated to engine/keywords.rs)
                apply_deathtouch_flag(self, source, &target);
                apply_lifelink(self, source, amount, _ctx)?;

                // CR 903.10a — if a commander deals combat damage to a
                // player, accumulate it per-commander on the damaged player.
                // The 21-damage loss check is the SBA at CR 704.6c.
                if is_combat {
                    if let DamageTarget::Player(pid) = &target {
                        let is_cmdr = self.objects.get(&source)
                            .map(|o| o.is_commander)
                            .unwrap_or(false);
                        if is_cmdr {
                            let entry = self.get_player_mut(*pid)?
                                .commander_damage_taken
                                .entry(source)
                                .or_insert(0);
                            *entry = entry.saturating_add(amount as u32);
                        }
                    }
                }

                self.events.emit(GameEvent::DamageDealt {
                    source_id: source,
                    target: target.clone(),
                    amount,
                });

                // Emit LifeChanged for player damage
                if let DamageTarget::Player(pid) = &target {
                    let new_life = self.get_player(*pid)?.life_total;
                    self.events.emit(GameEvent::LifeChanged {
                        player_id: *pid,
                        old: new_life + amount as i64,
                        new: new_life,
                        source: Some(source),
                    });
                }

                Ok(())
            }

            GameAction::DrawCard { player } => {
                // Delegate to `draw_card`, which handles empty-library flagging
                // (CR 121.6a) and proposes the library→hand move through
                // `change_zone` — so the move is a nested batch member, not a
                // second emitter.
                self.draw_card(player, _ctx)?;
                Ok(())
            }

            GameAction::GainLife { player, amount, source } => {
                // Same as `DealDamage` above: CR 119.10 makes a 0-life gain a
                // non-event, so `replacement::never_happens` drops it upstream.
                let old_life = self.get_player(player)?.life_total;
                let p = self.get_player_mut(player)?;
                p.life_total += amount as i64;
                let new_life = p.life_total;

                self.events.emit(GameEvent::LifeChanged {
                    player_id: player,
                    old: old_life,
                    new: new_life,
                    source: Some(source),
                });

                Ok(())
            }

            GameAction::LoseLife { player, amount } => {
                // Stays here, unlike `GainLife`'s: CR 119.10 is written about
                // life *gain* only, and no rule makes a 0 life loss a
                // non-event. A local no-op guard, not CR 614.7a.
                if amount == 0 {
                    return Ok(());
                }
                let old_life = self.get_player(player)?.life_total;
                let p = self.get_player_mut(player)?;
                p.life_total -= amount as i64;
                let new_life = p.life_total;

                self.events.emit(GameEvent::LifeChanged {
                    player_id: player,
                    old: old_life,
                    new: new_life,
                    source: None,
                });

                Ok(())
            }

            // **The only production emitter of `GameEvent::ZoneChange`.**
            // `move_object` performs the move and says nothing; the cause and
            // the CR 603.10a look-back frame are known here and nowhere else,
            // and an event assembled anywhere else would be missing them.
            GameAction::ZoneChange { object, from, to, cause } => {
                // Loud: the proposal describes a board, and performing it
                // against a different one is a caller bug. The pipeline (RB)
                // matches on `from`, so a stale value is a wrong match, not a
                // cosmetic mismatch.
                let actual = self.get_object(object)?.zone;
                if actual != from {
                    return Err(format!(
                        "ZoneChange proposed {:?}→{:?} for {}, which is in {:?}",
                        from, to, object, actual
                    ));
                }
                if from == to {
                    // Nothing moved, so nothing is announced — the same
                    // no-op `move_object` has always performed, made visible.
                    return Ok(());
                }

                // CR 603.10a — capture the frame while the object is still a
                // permanent. A moment later `cleanup_zone_state` has retired
                // the continuous effects its static abilities generated
                // (CR 611.2a) and the answer is unrecoverable. This is the one
                // place in the engine that has to run the layer walk *before* a
                // mutation rather than after.
                let lki = if from == Zone::Battlefield {
                    crate::engine::layers::compute::compute_characteristics(self, object)
                        .map(Box::new)
                } else {
                    None
                };
                let owner = self.get_object(object)?.owner;

                self.move_object(object, to)?;

                self.events.emit(GameEvent::ZoneChange {
                    object_id: object,
                    owner,
                    from,
                    to,
                    cause,
                    lki,
                });

                // CR 614.1c — arriving on the battlefield is its own proposed
                // event, so that "this permanent enters tapped" has something to
                // replace. Nested rather than performed inline: a nested
                // `execute_actions` joins the enclosing batch, which keeps
                // CR 603.2c's boundary around the whole zone change.
                //
                // `move_object` has already written `obj.zone`, so between here
                // and the `EnterBattlefield` performer the object is *in* the
                // battlefield zone with no `BattlefieldEntity`. That window is
                // one `emit` wide, and it is why nothing may be inserted
                // between these two statements.
                if to == Zone::Battlefield {
                    let controller = self.default_enter_controller(object)?;
                    self.propose_entry(object, controller, _ctx)?;
                }
                Ok(())
            }

            GameAction::Untap { object } => {
                // Loud: an untap proposed for something not on the battlefield
                // is a caller bug, not a no-op. The caller checks legality —
                // see `Primitive::Untap` in resolve.rs and CR 608.2b.
                let entry = self.battlefield.get_mut(&object).ok_or_else(|| {
                    format!("Cannot untap {}: not on the battlefield", object)
                })?;
                // CR 701.26b — only tapped permanents can be untapped — and
                // CR 603.2e makes "becomes untapped" a transition, not a state.
                if !entry.tapped {
                    return Ok(());
                }
                entry.tapped = false;
                self.events.emit(GameEvent::Untapped { object_id: object });
                Ok(())
            }

            GameAction::Tap { object } => {
                let entry = self.battlefield.get_mut(&object).ok_or_else(|| {
                    format!("Cannot tap {}: not on the battlefield", object)
                })?;
                // CR 701.26a — only untapped permanents can be tapped — and
                // CR 603.2e makes "becomes tapped" a transition, not a state.
                if entry.tapped {
                    return Ok(());
                }
                entry.tapped = true;
                self.events.emit(GameEvent::Tapped { object_id: object });
                Ok(())
            }

            GameAction::AddCounters { object, counter, n } => {
                if n == 0 {
                    return Ok(());
                }
                if !self.battlefield.contains_key(&object) {
                    return Err(format!(
                        "Cannot put counters on {}: not on the battlefield", object
                    ));
                }
                self.add_counters(object, counter, n);
                // A permanent that just gained a CR 122.1 replacement counter is
                // a replacement source now. The hint set is what keeps
                // `gather`'s fast path exact for static abilities; counters are
                // scanned rather than cached, so nothing has to be recorded
                // here — see `gather::any_replacement_counter`.
                self.events.emit(GameEvent::CountersChanged {
                    object_id: object,
                    counter,
                    added: n as i32,
                });
                Ok(())
            }

            GameAction::RemoveCounters { object, counter, n } => {
                if n == 0 {
                    return Ok(());
                }
                let Some(entry) = self.battlefield.get_mut(&object) else {
                    return Err(format!(
                        "Cannot remove counters from {}: not on the battlefield", object
                    ));
                };
                // CR 701.2 — do as much as possible. `remove_counters` reports
                // how many were actually there, and a removal of nothing is not
                // an event: CR 603.2e's transition rule is the same shape the
                // `Tap`/`Untap` arms follow.
                let removed = entry.remove_counters(counter, n);
                if removed == 0 {
                    return Ok(());
                }
                self.events.emit(GameEvent::CountersChanged {
                    object_id: object,
                    counter,
                    added: -(removed as i32),
                });
                Ok(())
            }

            // CR 701.8a — "to destroy a permanent, move it from the battlefield
            // to its owner's graveyard". The **outer** event: this performer's
            // whole job is to propose the inner zone change, which re-enters
            // the pipeline with a fresh applied set because a zone change is a
            // different kind of event from a destruction (§3.2d containment).
            //
            // That containment is what lets CR 122.1h's finality counter turn a
            // destroyed creature's graveyard trip into an exile while CR 122.1c's
            // shield counter, watching the destruction itself, has already
            // declined to apply.
            GameAction::Destroy { object, source } => {
                // Loud, like `Tap`/`Untap`: destroying something that is not on
                // the battlefield does nothing (CR 701.8b), and a caller that
                // proposes it has not checked CR 608.2b's partial resolution.
                // Both current callers do — `Primitive::Destroy` filters its
                // targets and the SBA sweep only ever names permanents — so a
                // lenient arm here would buy nothing except somewhere for a
                // future bug to hide.
                if !self.battlefield.contains_key(&object) {
                    return Err(format!(
                        "Cannot destroy {}: not on the battlefield", object
                    ));
                }
                self.execute_action(
                    GameAction::ZoneChange {
                        object,
                        from: Zone::Battlefield,
                        to: Zone::Graveyard,
                        cause: source.zone_change_cause(),
                    },
                    _ctx,
                )?;
                Ok(())
            }

            // CR 614.1c/d's event. The performer is `place_on_battlefield`,
            // which is also the only emitter of
            // `GameEvent::PermanentEnteredBattlefield`.
            GameAction::EnterBattlefield { object, controller, mods } => {
                self.place_on_battlefield(object, controller, &mods);
                Ok(())
            }
        }
    }

    /// Propose CR 614.1c's entry for an object that is already in the
    /// battlefield zone but has no `BattlefieldEntity` yet.
    ///
    /// **Loud if the entry is dropped.** Nothing in RC-2 can drop one — only
    /// `Rewrite::EnterWith` matches an `EnterBattlefield`, and no card writes a
    /// CR 614.17d prohibition on entering — but a dropped entry would leave the
    /// object in `Zone::Battlefield` with nothing on the battlefield, a state no
    /// rule describes and no query survives. CR 614.17d belongs to Phase RC-4
    /// and has to stop the *zone change*; this error is what makes an attempt to
    /// do it here fail at the attempt rather than three queries later.
    pub(crate) fn propose_entry(
        &mut self,
        object: ObjectId,
        controller: PlayerId,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        let mods = self.default_enter_mods(object);
        let performed = self.execute_actions(
            vec![GameAction::EnterBattlefield { object, controller, mods }],
            ctx,
        )?;
        if performed.is_empty() {
            return Err(format!(
                "the entry of {} onto the battlefield was replaced away, which leaves \
                 it in the battlefield zone with no permanent. A CR 614.17d \
                 prohibition on entering has to stop the zone change, not the entry \
                 - see `replacement-architecture.md` section 9, RC-4.",
                object
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_ctx;
    use crate::events::event::DamageTarget;
    use crate::objects::card_data::CardDataBuilder;
    use crate::objects::object::GameObject;
    use crate::types::card_types::*;
    use crate::types::keywords::KeywordFlag;
    use crate::types::mana::ManaType;

    fn setup_game_with_creature() -> (GameState, ObjectId) {
        let mut game = GameState::new(2, 20);

        let bears = CardDataBuilder::new("Grizzly Bears")
            .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::Green], 1))
            .color(crate::types::colors::Color::Green)
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(CreatureType::Bear))
            .power_toughness(2, 2)
            .build();

        // `place_bare`, not `place_on_battlefield`: entering the battlefield is
        // an event now (CR 614.1c), and every assertion below counts the events
        // the *action under test* emitted. A fixture that announces itself is
        // the reason `test_support` keeps the two idioms apart.
        let id = crate::test_support::place_bare(&mut game, bears, 0);

        (game, id)
    }

    fn tap_events(game: &GameState) -> Vec<&'static str> {
        game.events.events().filter_map(|e| match e {
            GameEvent::Tapped { .. } => Some("tapped"),
            GameEvent::Untapped { .. } => Some("untapped"),
            _ => None,
        }).collect()
    }

    #[test]
    fn test_tap_emits_only_on_the_transition() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::Tap { object: bears_id }, &test_ctx()).unwrap();
        assert!(game.battlefield.get(&bears_id).unwrap().tapped);
        assert_eq!(tap_events(&game), vec!["tapped"]);

        // CR 701.26a: only untapped permanents can be tapped. CR 603.2e: the
        // trigger event is the *change*, so a redundant tap announces nothing.
        game.execute_action(GameAction::Tap { object: bears_id }, &test_ctx()).unwrap();
        assert!(game.battlefield.get(&bears_id).unwrap().tapped);
        assert_eq!(tap_events(&game), vec!["tapped"], "a redundant tap is not a second event");
    }

    #[test]
    fn test_untap_emits_only_on_the_transition() {
        let (mut game, bears_id) = setup_game_with_creature();

        // Already untapped — CR 701.26b, nothing to untap, nothing announced.
        game.execute_action(GameAction::Untap { object: bears_id }, &test_ctx()).unwrap();
        assert!(tap_events(&game).is_empty());

        game.battlefield.get_mut(&bears_id).unwrap().tapped = true;
        game.execute_action(GameAction::Untap { object: bears_id }, &test_ctx()).unwrap();
        assert!(!game.battlefield.get(&bears_id).unwrap().tapped);
        assert_eq!(tap_events(&game), vec!["untapped"]);
    }

    #[test]
    fn test_tap_and_untap_are_loud_off_the_battlefield() {
        let (mut game, bears_id) = setup_game_with_creature();
        game.battlefield.remove(&bears_id);

        // Previously a silent no-op, against the loud-lowering doctrine. The
        // caller checks legality (CR 608.2b partial resolution); the performer
        // asserts its precondition.
        assert!(game.execute_action(GameAction::Tap { object: bears_id }, &test_ctx()).is_err());
        assert!(game.execute_action(GameAction::Untap { object: bears_id }, &test_ctx()).is_err());
        assert!(tap_events(&game).is_empty());
    }

    #[test]
    fn test_execute_deal_damage_to_creature() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Object(bears_id),
            amount: 3,
            is_combat: false,
        }, &test_ctx()).unwrap();

        assert_eq!(game.battlefield.get(&bears_id).unwrap().damage_marked, 3);
        // Should have emitted a DamageDealt event
        assert_eq!(game.events.len(), 1);
    }

    #[test]
    fn test_execute_deal_damage_to_player() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Player(1),
            amount: 3,
            is_combat: false,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].life_total, 17);
        // DamageDealt + LifeChanged
        assert_eq!(game.events.len(), 2);
    }

    #[test]
    fn test_execute_zero_damage_is_noop() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Player(1),
            amount: 0,
            is_combat: false,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].life_total, 20);
        assert_eq!(game.events.len(), 0);
    }

    #[test]
    fn test_execute_gain_life() {
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::GainLife {
            player: 0,
            amount: 5,
            source: bears_id,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[0].life_total, 25);
        assert_eq!(game.events.len(), 1);
    }

    #[test]
    fn test_execute_lose_life() {
        let (mut game, _bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::LoseLife {
            player: 0,
            amount: 3,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[0].life_total, 17);
        assert_eq!(game.events.len(), 1);
    }

    #[test]
    fn test_execute_untap() {
        let (mut game, bears_id) = setup_game_with_creature();
        game.battlefield.get_mut(&bears_id).unwrap().tapped = true;

        game.execute_action(GameAction::Untap {
            object: bears_id,
        }, &test_ctx()).unwrap();

        assert!(!game.battlefield.get(&bears_id).unwrap().tapped);
    }

    // --- Lifelink tests (4h) ---

    fn setup_game_with_lifelink_creature() -> (GameState, ObjectId) {
        let mut game = GameState::new(2, 20);

        let data = CardDataBuilder::new("Lifelink Creature")
            .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::White], 1))
            .color(crate::types::colors::Color::White)
            .card_type(CardType::Creature)
            .power_toughness(2, 3)
            .keyword(KeywordFlag::Lifelink)
            .build();

        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        (game, id)
    }

    #[test]
    fn test_lifelink_combat_damage_gains_life() {
        let (mut game, lifelinker) = setup_game_with_lifelink_creature();

        game.execute_action(GameAction::DealDamage {
            source: lifelinker,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }, &test_ctx()).unwrap();

        // Player 1 took 2 damage: 20 - 2 = 18
        assert_eq!(game.players[1].life_total, 18);
        // Player 0 (controller) gained 2 life: 20 + 2 = 22
        assert_eq!(game.players[0].life_total, 22);
    }

    #[test]
    fn test_lifelink_noncombat_damage_gains_life() {
        let (mut game, lifelinker) = setup_game_with_lifelink_creature();

        game.execute_action(GameAction::DealDamage {
            source: lifelinker,
            target: DamageTarget::Player(1),
            amount: 3,
            is_combat: false,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].life_total, 17);
        assert_eq!(game.players[0].life_total, 23);
    }

    #[test]
    fn test_no_lifelink_no_life_gain() {
        let (mut game, bears_id) = setup_game_with_creature(); // no lifelink

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].life_total, 18);
        // Player 0 should NOT have gained life
        assert_eq!(game.players[0].life_total, 20);
    }

    // --- T11: LifeChanged source field tests ---

    #[test]
    fn test_life_changed_event_includes_source() {
        // Deal combat damage with a lifelink creature; the resulting
        // LifeChanged events should carry the creature as source.
        let (mut game, lifelinker) = setup_game_with_lifelink_creature();

        game.execute_action(GameAction::DealDamage {
            source: lifelinker,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }, &test_ctx()).unwrap();

        // Events: DamageDealt, LifeChanged (damage to P1), LifeChanged (lifelink gain for P0)
        let life_events: Vec<_> = game.events.events().filter_map(|e| {
            if let GameEvent::LifeChanged { player_id, old, new, source } = e {
                Some((*player_id, *old, *new, *source))
            } else {
                None
            }
        }).collect();

        assert_eq!(life_events.len(), 2);

        // P0 gained 2 life from lifelink (emitted first, inside apply_lifelink)
        let (pid, old, new, src) = life_events[0];
        assert_eq!(pid, 0);
        assert_eq!(old, 20);
        assert_eq!(new, 22);
        assert_eq!(src, Some(lifelinker));

        // P1 lost 2 life from damage (emitted after keyword hooks)
        let (pid, old, new, src) = life_events[1];
        assert_eq!(pid, 1);
        assert_eq!(old, 20);
        assert_eq!(new, 18);
        assert_eq!(src, Some(lifelinker));
    }

    #[test]
    fn test_simultaneous_lifelink() {
        // Two lifelink creatures deal damage; each produces its own LifeChanged event.
        let mut game = GameState::new(2, 20);

        let make_lifelinker = |game: &mut GameState, name: &str| -> ObjectId {
            let data = CardDataBuilder::new(name)
                .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::White], 1))
                .color(crate::types::colors::Color::White)
                .card_type(CardType::Creature)
                .power_toughness(2, 2)
                .keyword(KeywordFlag::Lifelink)
                .build();
            let obj = GameObject::new(data, 0, Zone::Battlefield);
            let id = obj.id;
            game.add_object(obj);
            game.place_on_battlefield(id, 0, &EnterMods::NONE);
            id
        };

        let creature_a = make_lifelinker(&mut game, "Lifelinker A");
        let creature_b = make_lifelinker(&mut game, "Lifelinker B");

        // Both deal combat damage to opponent
        game.execute_action(GameAction::DealDamage {
            source: creature_a,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }, &test_ctx()).unwrap();
        game.execute_action(GameAction::DealDamage {
            source: creature_b,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }, &test_ctx()).unwrap();

        // P1 took 4 total damage
        assert_eq!(game.players[1].life_total, 16);
        // P0 gained 4 total life from lifelink
        assert_eq!(game.players[0].life_total, 24);

        // Collect all LifeChanged events for P0 (lifelink gains)
        let lifelink_gains: Vec<_> = game.events.events().filter_map(|e| {
            if let GameEvent::LifeChanged { player_id: 0, source, .. } = e {
                Some(*source)
            } else {
                None
            }
        }).collect();

        assert_eq!(lifelink_gains.len(), 2);
        assert_eq!(lifelink_gains[0], Some(creature_a));
        assert_eq!(lifelink_gains[1], Some(creature_b));
    }

    // --- Commander damage tracking (CR 903.10a) ---

    fn setup_game_with_commander() -> (GameState, ObjectId) {
        let mut game = GameState::new(2, 40);

        let general = CardDataBuilder::new("Test General")
            .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::Red], 2))
            .color(crate::types::colors::Color::Red)
            .card_type(CardType::Creature)
            .supertype(crate::types::card_types::Supertype::Legendary)
            .power_toughness(4, 4)
            .build();

        let mut obj = GameObject::new(general, 0, Zone::Battlefield);
        obj.is_commander = true;
        let id = obj.id;
        game.add_object(obj);
        game.place_on_battlefield(id, 0, &EnterMods::NONE);

        (game, id)
    }

    #[test]
    fn test_commander_combat_damage_accumulates() {
        let (mut game, cmdr) = setup_game_with_commander();

        game.execute_action(GameAction::DealDamage {
            source: cmdr,
            target: DamageTarget::Player(1),
            amount: 4,
            is_combat: true,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].life_total, 36);
        assert_eq!(game.players[1].commander_damage_taken.get(&cmdr).copied(), Some(4));
    }

    #[test]
    fn test_commander_combat_damage_stacks_across_hits() {
        let (mut game, cmdr) = setup_game_with_commander();

        for _ in 0..3 {
            game.execute_action(GameAction::DealDamage {
                source: cmdr,
                target: DamageTarget::Player(1),
                amount: 7,
                is_combat: true,
            }, &test_ctx()).unwrap();
        }

        // 3 × 7 = 21 — triggers the loss SBA when checked.
        assert_eq!(game.players[1].commander_damage_taken.get(&cmdr).copied(), Some(21));
    }

    #[test]
    fn test_commander_noncombat_damage_not_tracked() {
        // CR 903.10a applies only to combat damage.
        let (mut game, cmdr) = setup_game_with_commander();

        game.execute_action(GameAction::DealDamage {
            source: cmdr,
            target: DamageTarget::Player(1),
            amount: 4,
            is_combat: false,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].life_total, 36);
        assert!(game.players[1].commander_damage_taken.get(&cmdr).is_none());
    }

    #[test]
    fn test_noncommander_combat_damage_not_tracked() {
        // Only sources flagged `is_commander` contribute.
        let (mut game, bears_id) = setup_game_with_creature();

        game.execute_action(GameAction::DealDamage {
            source: bears_id,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
        }, &test_ctx()).unwrap();

        assert!(game.players[1].commander_damage_taken.get(&bears_id).is_none());
    }

    #[test]
    fn test_commander_damage_per_source_not_shared() {
        // Each commander accumulates its own counter on the damaged player.
        let mut game = GameState::new(2, 40);

        let build_cmdr = |game: &mut GameState, name: &str| -> ObjectId {
            let data = CardDataBuilder::new(name)
                .card_type(CardType::Creature)
                .supertype(crate::types::card_types::Supertype::Legendary)
                .power_toughness(3, 3)
                .build();
            let mut obj = GameObject::new(data, 0, Zone::Battlefield);
            obj.is_commander = true;
            let id = obj.id;
            game.add_object(obj);
            game.place_on_battlefield(id, 0, &EnterMods::NONE);
            id
        };

        let cmdr_a = build_cmdr(&mut game, "General A");
        let cmdr_b = build_cmdr(&mut game, "General B");

        game.execute_action(GameAction::DealDamage {
            source: cmdr_a, target: DamageTarget::Player(1), amount: 3, is_combat: true,
        }, &test_ctx()).unwrap();
        game.execute_action(GameAction::DealDamage {
            source: cmdr_b, target: DamageTarget::Player(1), amount: 3, is_combat: true,
        }, &test_ctx()).unwrap();

        assert_eq!(game.players[1].commander_damage_taken.get(&cmdr_a).copied(), Some(3));
        assert_eq!(game.players[1].commander_damage_taken.get(&cmdr_b).copied(), Some(3));
    }

    #[test]
    fn test_lose_life_event_has_no_source() {
        // LoseLife (e.g., paying life as a cost) has no source object.
        let (mut game, _) = setup_game_with_creature();

        game.execute_action(GameAction::LoseLife {
            player: 0,
            amount: 3,
        }, &test_ctx()).unwrap();

        let life_events: Vec<_> = game.events.events().filter_map(|e| {
            if let GameEvent::LifeChanged { source, .. } = e {
                Some(*source)
            } else {
                None
            }
        }).collect();

        assert_eq!(life_events.len(), 1);
        assert_eq!(life_events[0], None);
    }
}
