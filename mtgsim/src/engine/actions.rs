use std::collections::HashSet;

use crate::engine::keywords::{apply_deathtouch_flag, apply_lifelink};
use crate::engine::replacement::ReplacementInstanceId;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::resolve::ResolutionContext;
use crate::events::event::{DamageTarget, GameEvent, LossReason, ResolutionStamp};
use crate::state::game_state::{GameResult, GameState, Phase, PhaseType, StepType};
use crate::objects::object::GameObject;
use crate::types::effects::{CounterType, TokenDef};
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
pub use crate::types::zones::{DestructionSource, DrawCause, LifeLossCause, ZoneChangeCause};

/// How many batches may nest before the engine calls it a loop of its own
/// making — see `GameState::batch_depth`.
///
/// **An engine invariant, not a rule.** With every nested batch carrying its
/// lineage (a decomposition inherits, a rider inherits, a contained event
/// starts fresh), CR 614.5 bounds every replacement chain — each instance
/// applies once — so a chain deeper than any legitimate one means a lineage
/// was lost or a rider re-proposes its own event. **Not CR 104.4b's
/// detector**: a mandatory loop the rules allow runs through triggers, which
/// do not exist yet, and its detector is `backlog.md`'s loop entry when it is
/// written; this cannot fire on a rules loop, only on the engine. `fuzz_games`
/// prints the deepest nesting a run reached (`Max batch depth`); the bound
/// here is that number with headroom — **7** across 1,600 games, both pools
/// at two seats and four, 2026-09-13 — and a run that moves it is the place
/// to re-read it.
const BATCH_NESTING_LIMIT: usize = 32;

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
/// `PartialEq` because `ordering_cannot_change_outcome`'s fourth shape claims
/// two members would substitute the *same* event, and its debug check has to be
/// able to say so. Structural equality is the right meaning here: two proposals
/// are the same proposal when every field agrees.
#[derive(Debug, Clone, PartialEq)]
pub enum GameAction {
    /// Deal damage from a source to a target.
    DealDamage {
        source: ObjectId,
        target: DamageTarget,
        amount: u64,
        is_combat: bool,
        /// CR 615.12 — "the damage can't be prevented", as a fact about **this
        /// event** rather than about its source.
        ///
        /// The rule prints in three shapes and only this one is a property of
        /// the event: Pinpoint Avalanche's "the damage can't be prevented"
        /// says nothing about the next damage the same spell's controller
        /// deals. The other two — a resolution's "damage can't be prevented
        /// this turn" and a static ability's "damage can't be prevented" —
        /// are `Restriction::ApplyReplacement { kind: Prevention }`, asked at
        /// the same site this flag is (`replacement-architecture.md` §9, RD
        /// decision 6).
        ///
        /// It rides on the event and not on the source because it has to
        /// survive a redirect: CR 614.9 moves "the same damage", so
        /// `Rewrite::Retarget` copies this along with `is_combat`.
        unpreventable: bool,
    },

    /// **The instruction to draw** (CR 121.2, 121.2a) — "draw N cards",
    /// including N = 1.
    ///
    /// > 121.2a An instruction to draw multiple cards can be modified by
    /// > replacement effects that refer to the number of cards drawn. This
    /// > modification occurs before considering any of the individual card
    /// > draws.
    ///
    /// **Every draw instruction proposes this, "draw a card" included**, and
    /// Alms Collector's ruling is why it has to: *"to determine whether a player
    /// is instructed to draw multiple once or instructed multiple times to draw
    /// one card, count how many times the word 'draw' is used."* One "draw" of
    /// two cards is one of these with `n = 2`; two "draw a card"s are two of
    /// these with `n = 1`. An engine that skipped the instruction level for
    /// `n = 1` would have no event to tell the two apart on.
    ///
    /// Performing it proposes `n` individual [`Self::DrawCard`]s **one at a
    /// time**, each completing before the next is proposed (CR 121.2, 121.6b),
    /// and each inheriting this event's CR 614.5 applied set: the draws are this
    /// event at finer grain rather than events it caused, which is §3.2d's
    /// lineage rule and the reason two Teferi's Ageless Insights draw four cards
    /// instead of hanging.
    ///
    /// `n = 0` is an instruction that draws nothing. It still reaches the
    /// pipeline: CR 614.7a's "never happens" is 120.8's and 119.10's, each
    /// printed about its own event, and CR 121.2 says only that the player
    /// performs that many individual draws.
    DrawCards {
        player: PlayerId,
        n: u64,
        cause: DrawCause,
    },

    /// A single card draw (CR 121.1) — **the inner event**, and
    /// [`Self::DrawCards`]'s performer is its only producer.
    ///
    /// `cause` is *this draw's*, not its instruction's: an instruction's first
    /// individual draw carries the instruction's cause and every later one is
    /// [`DrawCause::Effect`]. See [`DrawCause`] for why that is the whole of
    /// "except the first one you draw in each of your draw steps".
    DrawCard {
        player: PlayerId,
        cause: DrawCause,
    },

    /// A player gains life.
    GainLife {
        player: PlayerId,
        amount: u64,
        source: ObjectId,
    },

    /// A player loses life.
    ///
    /// **Including from damage, from RD-1 on.** CR 120.3a makes life loss one
    /// of damage's *results* — "damage dealt to a player causes that player to
    /// lose that much life" — so `DealDamage`'s performer proposes one of
    /// these, contained in the damage's batch the way lifelink's gain is. That
    /// is what lets an RE-era Bloodletter of Aclazotz double the loss while a
    /// shield that already applied to the damage does not apply again (§3.2d
    /// containment, `replacement-architecture.md` §9 RD decision 4).
    ///
    /// `cause` is the fact that decomposition would otherwise destroy — see
    /// [`LifeLossCause`]. There is no catchall arm, for the reason
    /// [`ZoneChangeCause`] has none.
    LoseLife {
        player: PlayerId,
        amount: u64,
        cause: LifeLossCause,
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

    /// Attach an Aura, Equipment or Fortification to a permanent (CR 701.3a).
    ///
    /// A proposal because CR 603's "becomes attached" triggers read the
    /// performed stream, and because `GameState::attach` is the one writer
    /// of both sides of the link — a resolution that wrote it directly would
    /// be a second one. The performer writes through it and is loud when
    /// either end is off the battlefield; attaching to the host it is already
    /// on performs nothing (CR 701.3b). No `EventPattern` arm yet: no card
    /// replaces an attach, and an arm the pipeline cannot apply is worse than
    /// a missing one (`replacement-architecture.md` §3.2a).
    Attach {
        attachment: ObjectId,
        host: ObjectId,
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
    /// `PermanentState::remove_counters` already reports.
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
    /// pipeline and wins — see `engine::restriction::is_prohibited`.
    Destroy {
        object: ObjectId,
        source: DestructionSource,
    },

    /// A permanent enters the battlefield (CR 614.1c/d) — **the one proposal
    /// for entering, and it is the zone change.**
    ///
    /// Entering *is* the move onto the battlefield (CR 614.1c, 603.6a), so no
    /// `ZoneChange { to: Battlefield }` is ever proposed: `change_zone` routes
    /// a battlefield destination here, and the performer moves the card,
    /// announces that `ZoneChange`, builds the entity and announces the entry.
    /// Two questions are still asked of the one event — a zone-change-shaped
    /// pattern watches it as the move (Worms of the Earth, Grafdigger's Cage)
    /// and an entry-shaped one as the arrival (Root Maze) — and they share one
    /// CR 616.1 step, which is what the rule says entering is. RC-2 proposed
    /// this from *inside* the zone change's performer, and the log then held
    /// half of an event the CR says never happened whenever a replacement
    /// substituted the entry (`replacement-architecture.md` §11 item 20).
    ///
    /// `from` is the zone the card is coming from, or `None` for a token,
    /// which is created in the battlefield zone rather than moved into it
    /// (`GameState::create_tokens`). Its performer moves only when there is
    /// a `from`, and announces the creation when there is not.
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
    ///
    /// `cause` is the zone change's, or `None` for the same token. A fact about
    /// the event rather than about the object, recorded because CR 601's "was
    /// it cast" is unrecoverable a moment later: `Resolved` is a permanent
    /// spell that was cast, and everything else — a land drop, `Returned`, a
    /// token — was not. Read by `EventPattern::EnterBattlefield { cast }`, and
    /// by `EventPattern::ZoneChange`'s `cause` when it watches an entry.
    EnterBattlefield {
        object: ObjectId,
        from: Option<Zone>,
        controller: PlayerId,
        mods: EnterMods,
        cause: Option<ZoneChangeCause>,
    },

    /// An effect creates one or more tokens (CR 111, 701.7a) — **the outer
    /// event**, the one CR 614.16's doublers replace: "if an effect would
    /// create one or more tokens under your control, it creates twice that
    /// many of those tokens instead".
    ///
    /// `defs` is a `Vec` and not a `(def, n)` pair, as §3.1 decided for
    /// Academy Manufactor's "one of each" and Anointed Procession's ruling
    /// ("twice as many of each kind"): a multiplier repeats each def in
    /// place, and a heterogeneous creation stays one event. The subject is
    /// `controller`, the player the tokens are created under — their owner
    /// (CR 111.2) and CR 616.1's chooser.
    ///
    /// Performing it creates the objects and proposes **every entry as one
    /// batch** — contained events with fresh applied sets (CR 616.1g),
    /// joining this event's batch id — so two tokens entering together are
    /// each decided against the board before either entered (CR 614.12;
    /// `codebase-state.md` item 46), and a "can't enter" that drops one
    /// un-creates it (CR 111.5). The event is reported as decided whatever
    /// became of its entries; the log counts creations by
    /// [`GameEvent::TokenCreated`].
    CreateTokens {
        defs: Vec<TokenDef>,
        controller: PlayerId,
    },

    /// A token is created in a zone that is not the battlefield — the
    /// substituted form of a token's entry, Hallowed Moonlight's "exile it
    /// instead" applied to something that was never anywhere.
    ///
    /// **An appearance, not a move.** A card's substituted entry is a
    /// [`Self::ZoneChange`] from where the card is; a token has no `from`,
    /// and the honest event is the token being created in exile (its ruling:
    /// "put into exile instead and then ceases to exist"). `ZoneChange.from`
    /// stays a `Zone` — an `Option` there is a catchall-shaped `None` on
    /// every card move for one token's sake — so `pipeline::substitute`
    /// returns this instead, and CR 704.5d takes it from there.
    ///
    /// **No `EventPattern` arm, deliberately** — the second exemption from
    /// `replacement-architecture.md` §3.2a's one-arm-per-variant, after
    /// [`Self::Attach`]. Nothing prints "if a token would be created in
    /// exile", and an arm the pipeline cannot apply is worse than a missing
    /// one. `substitute` is its only producer.
    CreateTokenIn {
        object: ObjectId,
        zone: Zone,
    },

    /// A turn begins (CR 500.11, 614.10) — **the unit a "skip your next turn"
    /// replaces**.
    ///
    /// The subject is `player`, so CR 616.1's chooser is the player whose turn
    /// it is and Eon Hub's four-player form asks nobody. Its performer writes
    /// `turn_number` / `active_player` and announces
    /// [`GameEvent::TurnBegin`](crate::events::event::GameEvent::TurnBegin),
    /// which has been in the log since the log was written and emitted at zero
    /// sites: §8b counts 2,656 "at the beginning of" triggers, and none of them
    /// has anything to read until this proposal exists.
    ///
    /// `turn` is the number this turn *would* be — `turn_number + 1` — and a
    /// dropped proposal never advances it. That is CR 614.10a's "anything
    /// scheduled for a skipped turn won't happen" in the one place the engine
    /// can say it: `begin_turn` is the only writer of `last_turn_began`, so a
    /// turn that does not begin expires no "until your next turn" effect and
    /// starts no CR 302.6 clock.
    ///
    /// **Who takes the turn is not on the event**, and that is deliberate:
    /// CR 500.7's extra turns and the natural rotation are the *schedule* the
    /// proposal is built from, not a fact about the event. See
    /// `GameState::next_turn_taker`.
    BeginTurn {
        player: PlayerId,
        turn: u32,
    },

    /// A phase begins (CR 500.11, 614.10) — Moment of Silence's unit.
    ///
    /// The subject is the active player, which is what makes Moment of
    /// Silence's ruling fall out rather than be coded: a row on a player who
    /// is not the active player watches nothing, so "if cast on a player when
    /// it is not their turn, it has no effect".
    ///
    /// A skipped phase proposes none of its steps (CR 500.11's "proceed past
    /// it as though it didn't exist"), which is the drainer's rule, not this
    /// performer's.
    BeginPhase {
        phase: PhaseType,
        player: PlayerId,
    },

    /// A step begins (CR 500.11, 614.10) — Yawgmoth's Bargain's and Eon Hub's
    /// unit.
    ///
    /// The subject is the active player, as for [`Self::BeginPhase`]. Turn-based
    /// actions are **not** performed here: CR 703.4's actions are their own
    /// events, the untap sweep needs its own batch for CR 603.2c, and a
    /// performer nests a proposal only when the outer event is real without it
    /// (`replacement-architecture.md` §11 item 20). The drainer runs them after
    /// this event, and only for a step that began.
    BeginStep {
        step: StepType,
        player: PlayerId,
    },

    /// CR 104.3 — this player would lose the game. The event's subject is the
    /// player.
    ///
    /// Proposed by the state-based-action check for CR 704.5a–c and 704.6c,
    /// one member per player whatever the number of reasons (CR 704.7 — Lich's
    /// Mirror's ruling: "a single Lich's Mirror will replace all of them"),
    /// carrying the first reason in CR order; and by `Primitive::LoseGame` for
    /// CR 104.3e. Concession (104.3a) is a *leave* that then loses and is not
    /// proposed through here.
    PlayerLoses {
        player: PlayerId,
        reason: LossReason,
    },

    /// CR 104.2b — this player would win the game. The event's subject is the
    /// player.
    ///
    /// Proposed by `Primitive::WinGame` and substituted by Laboratory Maniac's
    /// `GameActionTemplate::PlayerWins`. CR 104.2a's win is **not** proposed:
    /// it is the outcome a batch of losses implies, recorded when that batch
    /// settles (`GameState::settle_game_result`), because "overrides all
    /// effects that would preclude that player from winning" is a rule with
    /// nothing for a replacement or a "can't" to see.
    PlayerWins {
        player: PlayerId,
    },

    // === Phase 3+ actions — add variants here as primitives are implemented ===
    // Sacrifice { object: ObjectId },
    // Exile { object: ObjectId },
    // CreateTokens { defs: Vec<TokenDef>, controller: PlayerId },
    // etc.
}

/// Which of CR 120.3's results a damage event has, read off the target.
///
/// > 120.3. Damage dealt to a permanent or player has one or more of the
/// > following results, depending on ...
///
/// A struct rather than an `if` chain because "one or more" is the rule's own
/// word: a creature planeswalker takes 120.3c **and** 120.3e, and an `else`
/// anywhere in the arm would silently make it take one. One
/// `compute_characteristics` call answers both questions — the target's
/// *effective* types, so March of the Machines' animated artifact takes marked
/// damage and Gideon does not stop being a planeswalker.
///
/// The four absent results are named in the arm that reads this; each is one
/// more field here and one more block there.
#[derive(Debug, Clone, Copy)]
struct DamageResults {
    /// CR 120.3e.
    mark_damage: bool,
    /// CR 120.3a.
    lose_life: bool,
    /// CR 120.3c.
    remove_loyalty: bool,
}

impl DamageResults {
    /// A player takes exactly one of the eight results today (CR 120.3a);
    /// 120.3b and 120.3g will add poison beside it.
    const PLAYER: Self = DamageResults {
        mark_damage: false,
        lose_life: true,
        remove_loyalty: false,
    };

    /// The results damage to `id` has, read off its **effective** types.
    fn for_object(game: &GameState, id: ObjectId) -> Self {
        use crate::types::card_types::CardType;
        // One walk, two questions, and no clone of the type set: this runs on
        // every damage event, and combat is where damage lives.
        let (creature, planeswalker) = crate::engine::layers::compute_characteristics(game, id)
            .map(|chars| {
                (
                    chars.types.contains(&CardType::Creature),
                    chars.types.contains(&CardType::Planeswalker),
                )
            })
            .unwrap_or((false, false));
        DamageResults {
            mark_damage: creature,
            lose_life: false,
            remove_loyalty: planeswalker,
        }
    }
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
        // A rider's proposals continue the replaced event's applied set
        // (CR 614.5, `Rider::lineage`); every other batch starts fresh.
        // Taken, not read: the batch this seeds is the rider's own event, and
        // the batches nested inside it are contained events with sets of
        // their own. Put back afterwards for the rider's next proposal.
        let rider_lineage = self.rider_lineage.take();
        let empty = HashSet::new();
        let inherited = rider_lineage.as_ref().unwrap_or(&empty);
        // A new lineage either way for the decomposition count: a rider's
        // draw inside a doubled draw starts counting from zero, or the
        // invariant `execute_actions_decomposing` asserts — depth bounded by
        // the inherited set — would be asked across two lineages at once.
        let outer_depth = std::mem::replace(&mut self.decomposition_depth, 0);
        self.batch_depth += 1;
        let result = self.execute_batch_inner(batch, ctx, inherited);
        self.batch_depth -= 1;
        self.decomposition_depth = outer_depth;
        self.rider_lineage = rider_lineage;
        self.events.close_batch(previous);
        result
    }

    /// [`Self::execute_actions`], but the batch's CR 616.1 loop starts from
    /// `inherited` rather than from an empty applied set.
    ///
    /// **One caller, and it needs a rule to exist** — CR 121.2's decomposition
    /// of a draw instruction into individual draws. The batch joins the
    /// enclosing one exactly as [`Self::execute_actions`] does, because the
    /// inner draws *are* the result of the outer instruction; the only
    /// difference is what CR 614.5's applied set starts as.
    ///
    /// A second caller needs §3.2d's argument made again, and it is **not**
    /// "these are nested" — every nested call in the crate is nested. It is
    /// that the inner event is the outer one **at finer grain** rather than an
    /// event the outer one caused. `DealDamage`'s contained `LoseLife` and a
    /// token creation's entries are the other shape, and each takes the fresh
    /// set `execute_actions` gives it. The name says *decomposing* rather than
    /// *inheriting* because the inheritance is the consequence and the
    /// decomposition is the claim a second caller has to make.
    ///
    /// **The debug assertion is the lineage rule computed the other way.** A
    /// call at depth `d` exists because `d - 1` substitutions happened above it,
    /// and a substitution that is not CR 903.9b-exempt inserts an instance into
    /// the applied set — so `d <= inherited.len() + 1` on any correct board, and
    /// the bound is derived rather than chosen. Break the inheritance and depth
    /// climbs while the set does not, which fires here at depth 2, before the
    /// recursion is deep enough to overflow the stack and take the whole test
    /// binary with it. No exempt draw replacement exists (CR 903.9b is about
    /// commanders), and the first one would relax this by exactly its count.
    pub(crate) fn execute_actions_decomposing(
        &mut self,
        batch: Vec<GameAction>,
        ctx: &ActionContext,
        inherited: &HashSet<ReplacementInstanceId>,
    ) -> Result<Vec<GameAction>, String> {
        self.decomposition_depth += 1;
        debug_assert!(
            self.decomposition_depth <= inherited.len() + 1,
            "a decomposed event at depth {} inherited only {} applied effects: \
             CR 614.5's set is what bounds the nesting, so the lineage is broken \
             (replacement-architecture.md section 3.2d)",
            self.decomposition_depth,
            inherited.len()
        );
        let previous = self.events.open_batch(ctx.resolution_stamp());
        self.batch_depth += 1;
        let result = self.execute_batch_inner(batch, ctx, inherited);
        self.batch_depth -= 1;
        self.events.close_batch(previous);
        self.decomposition_depth -= 1;
        result
    }

    /// [`Self::execute_actions`], but the batch gets a **new** id instead of
    /// joining the enclosing one.
    ///
    /// **One caller, and it needs a rule to exist** — CR 614.13's auxiliary
    /// zone changes (`// AUXILIARY-MOVE:` in `engine::replacement::pipeline`).
    /// §4.2's default is right on CR 120.3f's grounds: lifelink's life gain is
    /// a *result of* the damage, so a second batch id would tell a CR 603.2c
    /// trigger that two events happened. 614.13's moves are the converse. They
    /// are performed in phase 1, while the entry is still being *decided*, so
    /// there is no entry event yet for them to be part of; and two devour
    /// creatures entering together apply their replacements one after the
    /// other, so joining would hand "whenever one or more creatures die" one
    /// event where the rules have two.
    ///
    /// A second caller needs the same argument made again: not "these are
    /// different", but "the CR does not make these a result of the enclosing
    /// event".
    pub(crate) fn execute_actions_new_batch(
        &mut self,
        batch: Vec<GameAction>,
        ctx: &ActionContext,
    ) -> Result<Vec<GameAction>, String> {
        let previous = self.events.open_new_batch(ctx.resolution_stamp());
        // A new lineage, as in `execute_actions`.
        let outer_lineage = std::mem::replace(&mut self.decomposition_depth, 0);
        self.batch_depth += 1;
        let result = self.execute_batch_inner(batch, ctx, &HashSet::new());
        self.batch_depth -= 1;
        self.decomposition_depth = outer_lineage;
        self.events.close_batch(previous);
        result
    }

    /// The three phases of one batch — decide, perform, riders.
    ///
    /// **Split out from its two callers because Rust has no `finally`.**
    /// [`Self::execute_actions`] and [`Self::execute_actions_new_batch`] differ
    /// in exactly one thing — whether the batch joins the enclosing id or opens
    /// a fresh one (§4.2's `// AUXILIARY-MOVE:` rule) — and both must run
    /// `close_batch` on every exit path, including the several `?` that can
    /// leave this function with an `Err`. Putting the body here lets each
    /// wrapper be open / call / close with no early return of its own. A `Drop`
    /// guard is the other way to get that and cannot be used here: the guard
    /// would have to hold `&mut GameState` to close the batch, which is the
    /// borrow this function is already holding.
    ///
    /// Nothing else lives in the split — it is not a phase boundary or an
    /// extension point. [`Self::execute_actions_decomposing`] is the third caller
    /// and it makes §3.2d's argument about *lineage* rather than §4.2's about
    /// batch identity, so a fourth needs one or the other — not a reason to
    /// reuse this body.
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
        inherited: &HashSet<ReplacementInstanceId>,
    ) -> Result<Vec<GameAction>, String> {
        use crate::engine::replacement::{apply_replacements, subject_of, EventSubject, Rider};

        // CR 104.1 — "a game ends immediately when a player wins [or] the
        // game is a draw". Asked here, at the chokepoint, so every proposal
        // after the batch that ended the game stops at one line: the rest of
        // a resolution's instructions, a decomposition's remaining inners,
        // and the riders of the batch that ended it. Stunning Reversal's
        // survivor "wins the game as soon as everyone else has lost", and
        // the seven cards the rider would then draw are the game continuing
        // to be over. The ending batch's own members all perform — they were
        // one event — and its settlement is what makes this true afterwards.
        if self.result.is_some() {
            return Ok(Vec::new());
        }

        // Loud rather than a draw: CR 614.5 leaves no replacement chain
        // unbounded once each nested batch carries its lineage, so a nesting
        // this deep is the engine's mistake — a lost lineage — and a rules
        // answer here would hide it. The `Err` unwinds through every `?` to
        // the harness, which counts it as an error (`BATCH_NESTING_LIMIT`).
        self.counters.record_batch_depth(self.batch_depth as u64);
        if self.batch_depth > BATCH_NESTING_LIMIT {
            return Err(format!(
                "batches nested {} deep, past the {} any legitimate chain reaches: a proposal \
                 loop the applied set did not end (CR 614.5), so a nested batch lost its \
                 lineage or a rider re-proposes its own event",
                self.batch_depth, BATCH_NESTING_LIMIT
            ));
        }

        // Entering is the zone change, and `EnterBattlefield` is its only
        // proposal: a `ZoneChange` onto the battlefield here has bypassed
        // `change_zone`'s routing and would be performed with no entity.
        debug_assert!(
            !batch
                .iter()
                .any(|a| matches!(a, GameAction::ZoneChange { to: Zone::Battlefield, .. })),
            "a ZoneChange onto the battlefield is not a proposal: entering is proposed as \
             GameAction::EnterBattlefield, which change_zone routes to \
             (replacement-architecture.md section 9, RC-4b)"
        );

        // --- Phase 1: decide (CR 616.1), per subject, in APNAP order of chooser
        //
        // CR 616.1's last sentence: "if two or more players have to make these
        // choices at the same time, choices are made in APNAP order (see rule
        // 101.4)". A batch whose members affect different players produces
        // different choosers, and this is the only place that can order them.
        //
        // **The unit is the subject group, not the member** (RD-2; §9's RD
        // decision 3): members about one object or player — two blockers'
        // damage to one attacker — are decided by one CR 616.1 loop with one
        // applied set, so an effect applies to the pair once, which is what
        // CR 122.1c's "only one shield counter is removed" needs and what
        // Kalitas's N Zombies still get, since N deaths are N subjects.
        // Members sharing a subject share a chooser, so grouping them keeps
        // the APNAP order between players intact; within one player the
        // groups run in first-appearance order.
        //
        // CR 101.4d's restart — a nonactive player's choice forcing an
        // earlier player to choose again — is not implemented. It has not come
        // up for a narrower reason than "unreachable": nothing in phase 1 can
        // *create* a replacement effect, and each group's 616.1f loop runs to
        // completion before the next begins. That is a simplification, not a
        // proof; interleaving the groups is deferred (rb-review F6).
        // CR 614.13a/b's exclusion sets belong to *these* simultaneous entries.
        // Saved and restored like the event stamp: an auxiliary move's own
        // nested batch, and a rider's, are different events with their own.
        // Populated before the first member is decided, because 614.13a is
        // about what is entering rather than about what has entered.
        // CR 615.7's allocation answers are scoped the same way, for the same
        // reason.
        let outer_selection = std::mem::take(&mut self.entry_selection);
        self.entry_selection.entering = batch
            .iter()
            .filter_map(|a| match a {
                GameAction::EnterBattlefield { object, .. } => Some(*object),
                _ => None,
            })
            .collect();
        let outer_allocations = std::mem::take(&mut self.prevention_allocations);

        let mut groups: Vec<(EventSubject, Vec<usize>)> = Vec::new();
        for index in self.apnap_batch_order(&batch) {
            let subject = subject_of(&batch[index]);
            match groups.iter_mut().find(|(s, _)| *s == subject) {
                Some((_, members)) => members.push(index),
                None => groups.push((subject, vec![index])),
            }
        }

        let mut riders: Vec<Rider> = Vec::new();
        let mut decided: Vec<Option<GameAction>> = vec![None; batch.len()];
        // What each member's CR 616.1 loop applied, carried into phase 2 for
        // the one performer that decomposes. Per member rather than per group
        // only because `decided` is indexed that way; a group's members share
        // one loop and therefore one set. The clone is of an empty `HashSet`
        // for every event no replacement touched, which allocates nothing.
        let mut applied_to: Vec<HashSet<ReplacementInstanceId>> =
            vec![HashSet::new(); batch.len()];
        let mut decided_ok = Ok(());
        for g in 0..groups.len() {
            let members: Vec<(usize, GameAction)> =
                groups[g].1.iter().map(|&i| (i, batch[i].clone())).collect();
            let later: Vec<(usize, &GameAction)> = groups[g + 1..]
                .iter()
                .flat_map(|(_, idxs)| idxs.iter().map(|&i| (i, &batch[i])))
                .collect();
            let riders_before = riders.len();
            match apply_replacements(self, members, &later, ctx, inherited, &mut riders) {
                Ok((results, applied)) => {
                    // CR 614.5 — the riders this group queued are the rest of
                    // its replacements' effects, and carry everything that
                    // applied to the event, including what applied after
                    // they were queued.
                    for rider in &mut riders[riders_before..] {
                        rider.lineage = applied.clone();
                    }
                    for (i, action) in results {
                        decided[i] = action;
                        applied_to[i] = applied.clone();
                    }
                }
                Err(e) => {
                    decided_ok = Err(e);
                    break;
                }
            }
        }
        self.entry_selection = outer_selection;
        self.prevention_allocations = outer_allocations;
        decided_ok?;

        // --- Phase 2: perform, in batch order -------------------------------
        //
        // Batch order rather than APNAP order: the choices were the thing
        // CR 101.4 sequences, and the performed events are simultaneous. The
        // order they are written in is still observable (a graveyard is
        // ordered), and it is the caller's `battlefield_ids_ordered` sweep.
        let mut performed = Vec::with_capacity(decided.len());
        for (i, action) in decided.into_iter().enumerate() {
            let Some(action) = action else { continue };
            self.perform_action(action.clone(), ctx, &applied_to[i])?;
            performed.push(action);
        }

        // CR 104.2a / 104.4a are read off the batch, not off a member. Two
        // players losing in one state-based check is one simultaneous event
        // whose outcome is a draw; a performer that asked "is anyone left"
        // after the first of them would have crowned the second. Stunning
        // Reversal's four-player ruling is the same rule from the other side:
        // four losses proposed, one replaced, and the survivor "wins the game
        // as soon as everyone else has lost" — settled here, before the rider
        // that then makes them draw seven.
        if performed.iter().any(|a| matches!(a, GameAction::PlayerLoses { .. })) {
            self.settle_game_result();
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
        use crate::engine::replacement::chooser_for;

        let n = self.players.len();
        let mut order: Vec<usize> = (0..batch.len()).collect();
        order.sort_by_key(|&i| {
            let chooser = chooser_for(self, &batch[i]);
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
    /// object *or the player* the replacement was about and one written with
    /// `EffectRecipient::Controller` acts for the effect's own controller. The
    /// actions it proposes **carry the replaced event's applied set**
    /// (`Rider::lineage`, CR 614.5): they are the rest of the replacement's
    /// effect, and an effect that already applied to the event does not get
    /// a second opportunity on them. What they do *not* carry is the lineage
    /// of events nested inside them, which are contained and start fresh.
    ///
    /// A player subject becomes a `ResolvedTarget::Player` rather than being
    /// flattened away: Angel of Suffering's "prevent that damage and mill twice
    /// that many cards" mills the player the damage was aimed at, and until
    /// RD-1 the rider had no way to name one (`codebase-state.md` item 27).
    fn resolve_rider(
        &mut self,
        rider: crate::engine::replacement::Rider,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        use crate::engine::replacement::EventSubject;
        use crate::engine::resolve::{ResolutionContext, ResolvedTarget};

        let rctx = ResolutionContext {
            source: rider.source,
            ability_source: None,
            controller: rider.controller,
            targets: vec![match rider.subject {
                EventSubject::Object(id) => ResolvedTarget::Object(id),
                EventSubject::Player(pid) => ResolvedTarget::Player(pid),
            }],
            replaced_amount: rider.replaced_amount,
            damage_prevented: Some(rider.prevented),
        };
        let outer = self.rider_lineage.replace(rider.lineage);
        let result = self.resolve_effect(&rider.effect, &rctx, ctx.dp);
        self.rider_lineage = outer;
        result
    }

    /// Convenience wrapper for the most common zone change: caller knows the
    /// destination but doesn't want to hand-roll the `from` lookup.
    ///
    /// This is the intended public path for zone changes. A battlefield
    /// destination is routed to [`Self::propose_entry`], because entering is
    /// the zone change (`GameAction::EnterBattlefield`); every other
    /// destination proposes `GameAction::ZoneChange`. Either way the
    /// replacement pipeline (CR 614) sees the movement. `draw_card` and
    /// `play_land` route through here too, which is why a draw from an empty
    /// library reaches the pipeline at all (CR 121.6a). One production mover
    /// sits below the chokepoint and no more: `cast_spell`'s CR 601.2a move,
    /// in both directions — the permanent `// CAST-ROLLBACK:` exemption, which
    /// is announced at 601.2i once the spell is cast.
    pub fn change_zone(
        &mut self,
        object: ObjectId,
        to: Zone,
        cause: ZoneChangeCause,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        let from = self.get_object(object)?.zone;
        if to == Zone::Battlefield {
            if from == Zone::Battlefield {
                // Already there: nothing moves and nothing enters — the same
                // no-op `perform_zone_change` makes of any `from == to`.
                return Ok(());
            }
            let controller = self.default_enter_controller(object)?;
            self.propose_entry(object, Some(from), controller, Some(cause), ctx)?;
            return Ok(());
        }
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
    ///
    /// `lineage` is this event's own CR 616.1 applied set, and exactly one arm
    /// reads it — the one that **decomposes**. Every other nested proposal here
    /// is **containment** and takes the fresh set `execute_action` gives it.
    /// Both words are defined in `plans/glossary.md`, and the discriminator they
    /// turn on is `replacement-architecture.md` §3.2d's.
    fn perform_action(
        &mut self,
        action: GameAction,
        _ctx: &ActionContext,
        lineage: &HashSet<ReplacementInstanceId>,
    ) -> Result<(), String> {
        match action {
            // `unpreventable` is read by the pipeline and by nothing here:
            // CR 615.12 is a fact about which *effects* may prevent this
            // damage, and by the time the event is performed every effect has
            // had its say.
            GameAction::DealDamage { source, target, amount, is_combat, unpreventable: _ } => {
                // No 0-amount guard here: CR 614.7a says a 0-damage event never
                // happens, which makes it the *proposal's* problem — a 0 that
                // reaches the CR 616.1 loop is one a prevention effect applies
                // to. `replacement::never_happens` owns the rule, ahead of the
                // pipeline, and this function's only caller runs it.
                //
                // **CR 120.3 is a list of results, and this arm is that list.**
                // "Damage dealt ... has one or more of the following results" —
                // so each result is decided independently off the target's own
                // type, and a creature planeswalker gets 120.3c *and* 120.3e
                // rather than whichever arm ran first. Two of the eight are
                // here; 120.3e and 120.3f were already. The four that are
                // absent each have an owner and a dated Deferred Migrations
                // line: 120.3b and 120.3g (poison, from infect and toxic) and
                // 120.3d (wither's and infect's -1/-1 counters) are
                // `backlog.md` §2.6's and land as one more arm apiece off the
                // *source's* keywords; 120.3h (a battle's defense counters) is
                // §2.23's and needs the card type first.
                let results = match &target {
                    DamageTarget::Object(id) => {
                        if !self.battlefield.contains_key(id) {
                            return Err(format!(
                                "Target object {} not on battlefield", id
                            ));
                        }
                        DamageResults::for_object(self, *id)
                    }
                    DamageTarget::Player(_) => DamageResults::PLAYER,
                };

                // > 120.3e Damage dealt to a creature ... causes that much
                // > damage to be marked on that creature.
                //
                // Gated on the type from RD-1 on: an object that is neither a
                // creature nor a planeswalker takes no result at all, and
                // marking damage on it was bookkeeping the CR does not have.
                if results.mark_damage {
                    if let DamageTarget::Object(id) = &target {
                        self.battlefield.get_mut(id).expect("membership checked")
                            .damage_marked += amount as u32;
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

                // > 120.3a Damage dealt to a player causes that player to lose
                // > that much life.
                //
                // **A contained proposal, not a subtraction**, and it joins
                // this damage's batch exactly as lifelink's gain does
                // (CR 120.4c/d): a CR 603.2c trigger sees one event, and the
                // loss re-enters `apply_replacements` with a *fresh* applied
                // set (§3.2d containment) so an RE-era Bloodletter can double
                // it while a shield that already applied to the damage does
                // not apply again. Nothing can prevent it — a prevention
                // effect is one whose rewrite prevents on
                // `EventPattern::DealDamage`, and `LoseLife` has no pattern arm
                // at all, so Ali from Cairo's family is unwritable rather than
                // wrongly answered.
                //
                // After the `DamageDealt` emit, which is where the life change
                // already sat: the loss's own performer emits `LifeChanged`,
                // and `LifeLossCause::Damage` is what keeps `source` on it.
                if results.lose_life {
                    if let DamageTarget::Player(pid) = &target {
                        self.execute_action(
                            GameAction::LoseLife {
                                player: *pid,
                                amount,
                                cause: LifeLossCause::Damage { source },
                            },
                            _ctx,
                        )?;
                    }
                }

                // > 120.3c Damage dealt to a planeswalker causes that many
                // > loyalty counters to be removed from that planeswalker.
                //
                // A proposal for the same reason: CR 614.16's counter doublers
                // replace "counters would be put on", and a removal a card
                // watches is the mirror of one. `n` is a ceiling —
                // `PermanentState::remove_counters` reports what it took — and
                // CR 704.5i does the killing.
                if results.remove_loyalty {
                    if let DamageTarget::Object(id) = &target {
                        self.execute_action(
                            GameAction::RemoveCounters {
                                object: *id,
                                counter: CounterType::Loyalty,
                                n: amount as u32,
                            },
                            _ctx,
                        )?;
                    }
                }

                Ok(())
            }

            // > 121.2. Cards may only be drawn one at a time. If a player is
            // > instructed to draw multiple cards, that player performs that
            // > many individual card draws.
            //
            // **One at a time is literal**, and CR 121.6b says why it has to
            // be: "if an effect replaces a draw within a sequence of card
            // draws, the replacement effect is completed before resuming the
            // sequence." Each inner is its own batch, so its replacements are
            // decided and its riders run before the next one is proposed.
            //
            // `lineage` is what each inner starts its CR 614.5 applied set
            // from. These are this instruction at finer grain, not events it
            // caused, so the set continues (§3.2d) — which is the difference
            // between two Teferi's Ageless Insights drawing four cards and the
            // game hanging.
            //
            // No guard on `n == 0`: the loop is the no-op.
            GameAction::DrawCards { player, n, cause } => {
                for i in 0..n {
                    self.execute_actions_decomposing(
                        vec![GameAction::DrawCard {
                            player,
                            // CR 121.1's turn-based action is one card. Every
                            // draw after an instruction's first belongs to the
                            // effect that produced it, whatever the instruction
                            // was — which is what "except the first one you
                            // draw in each of your draw steps" means once a
                            // doubler has been applied to the draw step's draw.
                            cause: if i == 0 { cause } else { DrawCause::Effect },
                        }],
                        _ctx,
                        lineage,
                    )?;
                }
                Ok(())
            }

            GameAction::DrawCard { player, cause: _ } => {
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

            GameAction::LoseLife { player, amount, cause } => {
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
                    // CR 120.3a's loss names the damage's source, which is what
                    // keeps the log line unchanged now that damage to a player
                    // reaches life through here rather than around it.
                    source: match cause {
                        LifeLossCause::Damage { source } => Some(source),
                        LifeLossCause::Effect | LifeLossCause::Cost => None,
                    },
                });

                Ok(())
            }

            // The move and its announcement are `perform_zone_change`'s, which
            // the `EnterBattlefield` arm shares — the log cannot tell which arm
            // moved a card, and must not be able to.
            GameAction::ZoneChange { object, from, to, cause } => {
                // Loud: entering is `EnterBattlefield`'s, and a move into the
                // zone here would leave an object with no entity.
                if to == Zone::Battlefield {
                    return Err(format!(
                        "ZoneChange {:?}→Battlefield for {} reached the performer; entering is \
                         proposed as EnterBattlefield, which change_zone routes to",
                        from, object
                    ));
                }
                self.perform_zone_change(object, from, to, cause)
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

            GameAction::Attach { attachment, host } => {
                // Loud at both ends, like `Tap`: the caller checks CR 608.2b
                // legality and this asserts its precondition. `attach` itself
                // refuses silently, which is right for the state writer and
                // wrong for a performer.
                if !self.battlefield.contains_key(&attachment) {
                    return Err(format!("Cannot attach {}: not on the battlefield", attachment));
                }
                if !self.battlefield.contains_key(&host) {
                    return Err(format!("Cannot attach to {}: not on the battlefield", host));
                }
                let former_host = self.battlefield[&attachment].attached_to;
                // CR 701.3b — already there: the effect does nothing, and
                // nothing "becomes attached" (CR 603.2e's transition rule).
                if self.attach(attachment, host) {
                    self.events.emit(GameEvent::Attached { attachment, host, former_host });
                }
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
                if !self.battlefield.contains_key(&object) {
                    return Err(format!(
                        "Cannot remove counters from {}: not on the battlefield", object
                    ));
                }
                // CR 701.2 — do as much as possible. `remove_counters` reports
                // how many were actually there, and a removal of nothing is not
                // an event: CR 603.2e's transition rule is the same shape the
                // `Tap`/`Untap` arms follow.
                let removed = self.remove_counters(object, counter, n);
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

            // CR 614.1c/d's event, and the zone change that entering is. The
            // move is `perform_zone_change`'s; the entity and the entry's
            // announcement are `place_on_battlefield`'s, the only emitter of
            // `GameEvent::PermanentEnteredBattlefield`.
            GameAction::EnterBattlefield { object, from, controller, mods, cause } => {
                match (from, cause) {
                    (Some(Zone::Battlefield), _) => {
                        return Err(format!(
                            "entry of {} from the battlefield: it is already a permanent",
                            object
                        ));
                    }
                    (Some(from), Some(cause)) => {
                        self.perform_zone_change(object, from, Zone::Battlefield, cause)?;
                    }
                    // A token: created in the zone, so there is no move to
                    // perform (`create_tokens`). What there is to announce is
                    // CR 111.2's first sentence — the token is created — and
                    // its second is `place_on_battlefield`'s.
                    (None, None) => {
                        self.announce_token_created(object, Zone::Battlefield)?;
                    }
                    (from, cause) => {
                        return Err(format!(
                            "entry of {} names from={:?} and cause={:?}; a move has both and a \
                             token has neither",
                            object, from, cause
                        ));
                    }
                }
                self.place_on_battlefield(object, controller, &mods);
                Ok(())
            }

            // CR 111 / 701.7a — the outer event. The objects, then every
            // entry as one contained batch; `create_tokens` says why.
            GameAction::CreateTokens { defs, controller } => {
                self.create_tokens(&defs, controller, _ctx)
            }

            // The substituted form of a token's entry: an appearance in a
            // zone that is not the battlefield, announced as the creation it
            // is. `put_token_into` performs; this arm announces.
            GameAction::CreateTokenIn { object, zone } => {
                self.put_token_into(object, zone)?;
                self.announce_token_created(object, zone)
            }

            // --- Turn structure (CR 500, 614.10) ----------------------------
            //
            // Three small performers, and each writes exactly the one field
            // that makes its unit "the one that is happening". They announce
            // the three `GameEvent`s that have existed since the log was
            // written and were emitted nowhere, which is what item 6's
            // "at the beginning of" triggers will read.
            //
            // None of them runs a turn-based action or an expiry: those are
            // separate events (CR 703.4, 500.4), and the drainer runs them
            // after the proposal survives — see `engine::turns`.
            GameAction::BeginTurn { player, turn } => {
                // `begin_turn` is the one writer of `last_turn_began`, so a
                // turn that is skipped starts no CR 302.6 clock and expires no
                // "until your next turn" effect (CR 614.10a).
                self.begin_turn(turn, player);
                self.priority_player = player;
                self.events.emit(GameEvent::TurnBegin { player, turn_number: turn });
                Ok(())
            }

            GameAction::BeginPhase { phase, player: _ } => {
                // `Phase::new` would fill in the phase's first step; the step is
                // its own proposal, and a phase whose first step is skipped
                // must not look as though that step is happening.
                self.phase = Phase { phase_type: phase, step: None };
                self.events.emit(GameEvent::PhaseBegin { phase });
                Ok(())
            }

            GameAction::BeginStep { step, player: _ } => {
                self.phase.step = Some(step);
                self.events.emit(GameEvent::StepBegin { step });
                Ok(())
            }

            // --- The game's end (CR 104) ------------------------------------
            //
            // Loud on a player who has already left, like `Tap` on something
            // off the battlefield: the SBA check gates on `player_lost` before
            // proposing, so a second loss is a caller bug and not CR 800.4a.
            //
            // `has_drawn_from_empty_library` is **not** cleared here. CR 704.5b's
            // window closes at the check that reads it, whether or not the loss
            // it proposed was then replaced or refused — see
            // `check_state_based_actions`.
            //
            // CR 104.2a and 104.4a are not decided here either: whether the
            // survivors have won or everyone has drawn is a fact about the
            // *batch* these losses were performed in, and `execute_batch_inner`
            // settles it once the whole batch has performed.
            GameAction::PlayerLoses { player, reason } => {
                if self.player_lost[player] {
                    return Err(format!("player {} has already left the game", player));
                }
                self.player_lost[player] = true;
                self.events.emit(GameEvent::PlayerLost { player_id: player, reason });
                // CR 104.3 — a player who loses the game leaves it — and
                // CR 800.4a's four clauses follow here rather than at the next
                // state-based check, because the rule says "this is not a
                // state-based action. It happens as soon as the player leaves
                // the game". Clause 4's exile is a result of the departure, so
                // its nested batch joins this one (§4.2).
                self.player_left_the_game(player, _ctx)
            }

            // CR 104.1 — "immediately". The result is recorded here and read by
            // everything that asks whether the game is over; a second winner
            // in the same batch cannot arise (the first ends the game) and a
            // recorded result is never overwritten. CR 104.3f — win and lose at
            // once → lose — has no producer and is recorded, not built.
            GameAction::PlayerWins { player } => {
                if self.player_lost[player] {
                    return Err(format!("player {} has left the game and cannot win it", player));
                }
                if self.result.is_none() {
                    self.result = Some(GameResult::Winner(player));
                }
                self.events.emit(GameEvent::PlayerWon { player_id: player });
                Ok(())
            }
        }
    }

    /// The one performer of a move between zones: the stale check, the
    /// CR 603.10a frame, `move_object`, and the announcement.
    ///
    /// Shared by the `ZoneChange` arm and the `EnterBattlefield` arm rather
    /// than being the first one's body, because entering is a zone change and
    /// the log must not be able to tell which arm moved a card. `from == to`
    /// performs and announces nothing.
    fn perform_zone_change(
        &mut self,
        object: ObjectId,
        from: Zone,
        to: Zone,
        cause: ZoneChangeCause,
    ) -> Result<(), String> {
        // Loud: the proposal describes a board, and performing it against a
        // different one is a caller bug. The pipeline matches on `from`, so a
        // stale value is a wrong match, not a cosmetic mismatch.
        let actual = self.get_object(object)?.zone;
        if actual != from {
            return Err(format!(
                "ZoneChange proposed {:?}→{:?} for {}, which is in {:?}",
                from, to, object, actual
            ));
        }
        if from == to {
            return Ok(());
        }

        // CR 603.10a — capture the frame while the object is still a
        // permanent. A moment later `cleanup_zone_state` has retired the
        // continuous effects its static abilities generated (CR 611.2a) and
        // the answer is unrecoverable. This is the one place in the engine that
        // has to run the layer walk *before* a mutation rather than after.
        //
        // A permanent, not merely an object in the zone: a token whose entry
        // is being decided is in the battlefield zone with no entity, and there
        // is nothing to look back at (`create_tokens`).
        let lki = if from == Zone::Battlefield && self.battlefield.contains_key(&object) {
            crate::engine::layers::compute::compute_characteristics_uncached(self, object)
                .map(Box::new)
        } else {
            None
        };

        self.move_object(object, to)?;
        self.announce_zone_change(object, from, to, cause, lki)
    }

    /// **The only emitter of `GameEvent::ZoneChange`.** Three callers, each of
    /// which performed the move it announces: [`Self::perform_zone_change`] for
    /// the `ZoneChange` and `EnterBattlefield` arms, and `cast_spell` at
    /// CR 601.2i for the move it made silently at 601.2a — the moment the spell
    /// becomes cast, which is the first moment that move is an event.
    pub(crate) fn announce_zone_change(
        &mut self,
        object: ObjectId,
        from: Zone,
        to: Zone,
        cause: ZoneChangeCause,
        lki: Option<Box<EffectiveCharacteristics>>,
    ) -> Result<(), String> {
        let owner = self.get_object(object)?.owner;
        self.events.emit(GameEvent::ZoneChange { object_id: object, owner, from, to, cause, lki });
        Ok(())
    }

    /// **The only emitter of `GameEvent::TokenCreated`.** Two callers, each
    /// of which performed the placement it announces: the `EnterBattlefield`
    /// arm for a token entering, and the `CreateTokenIn` arm for a token
    /// created anywhere else. `owner` is CR 111.2's — the player who created
    /// it.
    pub(crate) fn announce_token_created(
        &mut self,
        object: ObjectId,
        zone: Zone,
    ) -> Result<(), String> {
        let owner = self.get_object(object)?.owner;
        self.events.emit(GameEvent::TokenCreated { object_id: object, owner, zone });
        Ok(())
    }

    /// CR 614.1c's entry as a proposal, or `None` where a rule refuses it
    /// ahead of any event — built here for [`Self::propose_entry`]'s one
    /// entry and [`Self::create_tokens`]' batch of them alike.
    ///
    /// CR 800.4b — "if an object would be put onto the battlefield ... under
    /// the control of a player who has left the game, that object remains in
    /// its current zone". A rule, checked at the site like CR 508.8's and
    /// CR 800.4k's: there is no event here for a replacement effect to see.
    ///
    /// The seed's counters (CR 306.5b's loyalty) go through the same CR 101.2
    /// door as a replacement's: a "can't have counters put on it" that would
    /// stop them later stops them here (CR 614.17d), with no cause, because a
    /// rule put them there.
    fn entry_proposal(
        &mut self,
        object: ObjectId,
        from: Option<Zone>,
        controller: PlayerId,
        cause: Option<ZoneChangeCause>,
    ) -> Option<GameAction> {
        if self.is_multiplayer() && !self.in_game(controller) {
            return None;
        }
        let seed = self.default_enter_mods(object, controller);
        let mods = crate::engine::replacement::strip_prohibited_counters(
            self, object, controller, &EnterMods::NONE, &seed, None,
        );
        Some(GameAction::EnterBattlefield { object, from, controller, mods, cause })
    }

    /// Propose CR 614.1c's entry — the one proposal for a card entering the
    /// battlefield, from `from`. (A token's entry is proposed by
    /// [`Self::create_tokens`], as a member of its creation's batch.)
    ///
    /// Returns whether the entry was performed. `false` means the pipeline
    /// dropped it — CR 614.6, or a CR 614.17d "can't enter" — or CR 800.4b
    /// refused it, and the object is where it was, with nothing moved and
    /// nothing announced. An entry *substituted* by a zone change (Containment
    /// Priest's "exile it instead") was performed as that zone change and is
    /// `true`; a caller that needs to know where the card ended up asks the
    /// card (`resolve_top_of_stack`, CR 608.3e).
    pub(crate) fn propose_entry(
        &mut self,
        object: ObjectId,
        from: Option<Zone>,
        controller: PlayerId,
        cause: Option<ZoneChangeCause>,
        ctx: &ActionContext,
    ) -> Result<bool, String> {
        let Some(entry) = self.entry_proposal(object, from, controller, cause) else {
            return Ok(false);
        };
        let performed = self.execute_actions(vec![entry], ctx)?;
        Ok(!performed.is_empty())
    }

    /// The `CreateTokens` performer: the objects, then their entries as **one
    /// batch**.
    ///
    /// Each token is created in the battlefield zone with no entity and in no
    /// collection — the state the look-ahead frame's membership gate reads as
    /// "entering" and CR 704.5d reads as "on the battlefield" — and its entry
    /// carries no `from` and no cause. CR 110.2b's default controller is the
    /// player the creating effect gave it to, who is already its owner
    /// (CR 111.2); passed explicitly because a token never passed through the
    /// stack and so is never `GameState::resolving`.
    ///
    /// The entries are **contained** events (`replacement-architecture.md`
    /// §3.2d): one `execute_actions`, joining this batch's id, each with a
    /// fresh applied set. That order is CR 616.1g — "the second effect can't
    /// be chosen until after the first effect has been chosen" — read as the
    /// order of two loops: a doubler applied to the creation has been applied
    /// before any entry exists, and it is not offered again at one. And it is
    /// `codebase-state.md` item 46's plural entry: every member is decided
    /// against the board before any of them entered (CR 614.12), which is
    /// what "can't apply to any other permanent entering at the same time"
    /// means for a printed card.
    ///
    /// A member that did not perform — dropped by a "can't enter" (CR 614.17d)
    /// or by a `Prevent` — is CR 111.5's "the token is not created": the
    /// object goes, and un-creating it is no more an event than `add_object`
    /// was. A *substituted* member was created somewhere else
    /// (`CreateTokenIn`) and stays. The outer event is reported as decided
    /// whichever way its members went, on `DrawCards`' precedent against an
    /// empty library; the log counts creations by `TokenCreated`.
    fn create_tokens(
        &mut self,
        defs: &[TokenDef],
        controller: PlayerId,
        ctx: &ActionContext,
    ) -> Result<(), String> {
        // One `Arc<CardData>` per run of equal defs: a doubled creation
        // repeats each def in place, and the repeats share one printed text.
        let mut lowered: Option<(&TokenDef, std::sync::Arc<crate::objects::card_data::CardData>)> =
            None;
        let mut ids = Vec::with_capacity(defs.len());
        let mut entries = Vec::with_capacity(defs.len());
        for def in defs {
            let data = match &lowered {
                Some((seen, data)) if *seen == def => data.clone(),
                _ => {
                    let data = def.card_data();
                    lowered = Some((def, data.clone()));
                    data
                }
            };
            // CR 111.2 — a token's owner is the player who controls the effect
            // that created it — and CR 111.1's `is_token` is what makes
            // CR 704.5d and `ObjectFilter::Token` able to see it. Both are set
            // before it reaches the battlefield, because
            // `register_static_effects` runs inside `place_on_battlefield`
            // and would otherwise register against an object that does not
            // yet know what it is.
            let mut obj = GameObject::new(data, controller, Zone::Battlefield);
            obj.is_token = true;
            let id = self.add_object(obj);
            ids.push(id);
            if let Some(mut entry) = self.entry_proposal(id, None, controller, None) {
                // "Create a tapped …" is the creating effect's own word on
                // how the token enters, so it joins the seed the rules give
                // the entry, ahead of any replacement (CR 614.1c).
                if def.enters_tapped {
                    if let GameAction::EnterBattlefield { mods, .. } = &mut entry {
                        mods.merge(&EnterMods::tapped());
                    }
                }
                entries.push(entry);
            }
        }
        let performed = self.execute_actions(entries, ctx)?;
        let created: HashSet<ObjectId> = performed
            .iter()
            .filter_map(|a| match a {
                GameAction::EnterBattlefield { object, .. }
                | GameAction::CreateTokenIn { object, .. } => Some(*object),
                _ => None,
            })
            .collect();
        for id in ids {
            if !created.contains(&id) {
                self.remove_object(id);
            }
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
            unpreventable: false
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
            unpreventable: false
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
            unpreventable: false
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
            cause: LifeLossCause::Cost,
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
            .keyword_flag(KeywordFlag::Lifelink)
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
            unpreventable: false
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
            unpreventable: false
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
            unpreventable: false
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
            unpreventable: false
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
                .keyword_flag(KeywordFlag::Lifelink)
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
            unpreventable: false
        }, &test_ctx()).unwrap();
        game.execute_action(GameAction::DealDamage {
            source: creature_b,
            target: DamageTarget::Player(1),
            amount: 2,
            is_combat: true,
            unpreventable: false
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
            unpreventable: false
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
                unpreventable: false
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
            unpreventable: false
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
            unpreventable: false
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
            source: cmdr_a, target: DamageTarget::Player(1), amount: 3, is_combat: true, unpreventable: false
        }, &test_ctx()).unwrap();
        game.execute_action(GameAction::DealDamage {
            source: cmdr_b, target: DamageTarget::Player(1), amount: 3, is_combat: true, unpreventable: false
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
            cause: LifeLossCause::Cost,
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

    /// The nesting guard is loud and names the invariant it stands for. With
    /// every nested batch carrying its lineage nothing legitimate reaches it,
    /// so the test sets the depth by hand rather than building a chain.
    #[test]
    fn a_batch_nested_past_the_limit_is_an_error_not_a_draw() {
        let (mut game, bears) = setup_game_with_creature();
        game.batch_depth = BATCH_NESTING_LIMIT;

        let err = game
            .execute_action(GameAction::Tap { object: bears }, &test_ctx())
            .unwrap_err();

        assert!(err.contains("nested"), "{err}");
        assert!(game.result.is_none(), "an engine loop is an error, never a rules answer");
    }
}
