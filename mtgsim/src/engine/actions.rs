use std::collections::HashSet;

use crate::engine::keywords::{add_lifelink_gain, apply_deathtouch_flag};
use crate::engine::replacement::ReplacementInstanceId;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::resolve::ResolutionContext;
use crate::events::event::{CounterSubject, DamageTarget, GameEvent, LossReason, ResolutionStamp};
use crate::state::game_state::{GameResult, GameState, Phase, PhaseType, StepType};
use crate::objects::object::GameObject;
use crate::types::effects::{CounterType, TokenDef};
use crate::types::ids::{IdSet, ObjectId, PlayerId};
use crate::types::mana::{ManaAtom, ManaType};
use crate::types::replacement::EnterMods;
use crate::types::zones::Zone;
use crate::ui::ask::ask_scry;
use crate::ui::decision::DecisionProvider;

/// Re-exported for every existing reader of `engine::actions::ZoneChangeCause`.
///
/// The definition lives in `types::zones`, alongside `Zone`, which
/// is the vocabulary it qualifies. The mover is `EventPattern::ZoneChange`:
/// a replacement effect watching "would be put into a graveyard from the
/// battlefield" has to name the cause, `EventPattern` lives in `types`, and
/// `src/types/` has no `crate::engine` edge to spend.
pub use crate::types::zones::{DestructionSource, DrawCause, LifeLossCause, ZoneChangeCause};

/// How many batches may nest before the engine calls it a loop of its own
/// making — see `NestingGuards::batch_depth`.
///
/// **An engine invariant, not a rule.** With every nested batch carrying its
/// lineage, CR 614.5 bounds every replacement chain, so a chain deeper than
/// any legitimate one means a lineage was lost or a rider re-proposes its
/// own event. **Not CR 104.4b's detector**: a mandatory loop the rules allow
/// runs through triggers, and its detector is `backlog.md`'s loop entry.
/// `fuzz_games` prints the deepest nesting a run reached (`Max batch
/// depth`); the bound is that number with headroom — 7 across 1,600 games,
/// both pools at two seats and four, 2026-09-13.
const BATCH_NESTING_LIMIT: usize = 32;

/// Who is asking for a mutation, and what resolution it belongs to.
///
/// `execute_action` has no `DecisionProvider` of its own, and CR 616.1 needs
/// one: when two or more replacement effects want the same event, *the affected
/// object's controller* chooses which to apply — not the controller of the
/// effect. Rather than thread a bare `&dyn DecisionProvider`, this carries the
/// second thing the pipeline will want, so the plumbing is paid for once.
///
/// `apply_replacements` reads both:
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
    /// with its own `ObjectId`, and CR 608.2n destroys the ability's object
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
/// The pre-mutation counterpart to `GameEvent`, which records what *did*
/// happen. The engine builds a `GameAction` and passes it through
/// `execute_action`; the CR 614 pipeline (`apply_replacements`, inside
/// `execute_actions`) sits between the proposal and the mutation, so a
/// proposal can be modified, replaced or dropped before it is carried out.
///
/// `PartialEq` because `ordering_cannot_change_outcome`'s fourth shape claims
/// two members would substitute the *same* event, and its debug check has to
/// be able to say so; two proposals are the same proposal when every field
/// agrees.
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
    /// **Including from damage.** CR 120.3a makes life loss one of damage's
    /// *results*, so `DealDamage`'s performer proposes one of these, contained
    /// in the damage's batch the way lifelink's gain is — which lets Bloodletter
    /// of Aclazotz double the loss while a shield that already applied to the
    /// damage does not apply again (§3.2d containment,
    /// `replacement-architecture.md` §9 RD decision 4).
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

    /// Put counters on a permanent, or give them to a player (CR 122.1).
    ///
    /// A proposal rather than a direct write because CR 614.16's counter
    /// doublers replace it, and because CR 122.1c/d's own replacement effects
    /// have to be able to *make* one: "instead remove a stun counter from it" is
    /// a proposed event, not bookkeeping.
    ///
    /// **Not the event for counters a permanent enters with.** CR 122.6 folds
    /// those into the entry, so they ride on `EnterBattlefield`'s `mods` and
    /// `EventPattern::AddCounters` watches the entry through a second door.
    ///
    /// `by` is the player putting them on (Vorinclex, Monstrous Raider's
    /// ruling "cares deeply about who is putting the counters on"), a `PlayerId`
    /// rather than an `Option` because every producer has one. A cost that puts
    /// counters is not an effect and must not be matched by CR 614.16's
    /// watchers; no cost produces this yet (`codebase-state.md`, "Found by RE-5").
    AddCounters {
        subject: CounterSubject,
        counter: CounterType,
        n: u32,
        by: PlayerId,
    },

    /// Take counters off a permanent, or away from a player (CR 122.1).
    ///
    /// The substituted event for CR 122.1c's shield counter and CR 122.1d's
    /// stun counter, and the CR 615.5 rider for the shield's prevention half.
    ///
    /// `n` is a maximum: removing three counters from a permanent that has one
    /// removes one, which is CR 701.2's "as much as it can" and what
    /// `PermanentState::remove_counters` already reports. **That is the rule
    /// for an effect's instruction, and a cost never reaches it**: CR 118.3
    /// lets no player pay a cost they cannot pay in full, so a cost that
    /// removes counters — paying {E}, a loyalty ability's minus — is validated
    /// against the count before it proposes anything, where `Cost::Sacrifice`
    /// counts its candidates (`engine::costs`; `Cost::RemoveCounters` is the
    /// unimplemented arm there). No `by`: nothing printed asks who *removes* a
    /// counter.
    RemoveCounters {
        subject: CounterSubject,
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
    /// A zone-change-shaped pattern watches it as the move (Worms of the Earth,
    /// Grafdigger's Cage) and an entry-shaped one as the arrival (Root Maze);
    /// they share one CR 616.1 step (`replacement-architecture.md` §11 item 20).
    ///
    /// `from` is the zone the card is coming from, or `None` for a token,
    /// which is created in the battlefield zone (`GameState::create_tokens`).
    ///
    /// `controller` is CR 110.2b's **default** — the owner for a land drop or a
    /// token, the player who put the spell on the stack for a resolving
    /// permanent spell — the value Layer 2 modifies, not the answer
    /// `get_effective_controller` gives.
    ///
    /// `mods` starts as whatever the *rules* say the permanent enters with
    /// (`GameState::default_enter_mods`) and accumulates through
    /// [`Rewrite::EnterWith`](crate::types::replacement::Rewrite::EnterWith)
    /// as CR 616.1f iterates.
    ///
    /// `cause` is the zone change's, or `None` for a token. A fact about the
    /// event, recorded because CR 601's "was it cast" is unrecoverable a moment
    /// later: `Resolved` is a permanent spell that was cast, everything else was
    /// not. Read by `EventPattern::EnterBattlefield { cast }` and by
    /// `EventPattern::ZoneChange`'s `cause` when it watches an entry.
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
    /// which item 6's "at the beginning of" triggers will read.
    ///
    /// `turn` is the number this turn *would* be — `turn_number + 1` — and a
    /// dropped proposal never advances it: `begin_turn` is the only writer of
    /// `last_turn_began`, so a turn that does not begin expires no "until your
    /// next turn" effect and starts no CR 302.6 clock (CR 614.10a).
    ///
    /// **Who takes the turn is not on the event**: CR 500.7's extra turns and
    /// the natural rotation are the *schedule* the proposal is built from. See
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

    /// CR 701.22 — a player scries N. The event's subject is the player.
    ///
    /// **An event because a card replaces it**: Eligeth, Crossroads Augur's
    /// "if you would scry a number of cards, draw that many cards instead"
    /// (the other of the two printed "would scry" clauses is a trigger).
    ///
    /// Its performer looks at the top `n`, asks where they go, and reorders
    /// the library **in the arm** — it proposes nothing, because CR 701.22
    /// moves no card between zones and there is no `ZoneChange` to make. What
    /// it announces is [`GameEvent::Scried`](crate::events::event::GameEvent::Scried),
    /// which CR 701.22d's "an ability that triggers whenever a player scries"
    /// will read.
    ///
    /// `n` is the *instruction's* number, not the count of cards actually
    /// there: CR 701.22a looks at the top N and a shorter library has fewer,
    /// which CR 701.22d's "even if some or all of those actions were
    /// impossible" then covers. `n = 0` never reaches the performer —
    /// CR 701.22b says no scry event occurs, which is `replacement::
    /// never_happens`' business ahead of the pipeline.
    Scry {
        player: PlayerId,
        n: u64,
    },

    /// CR 701.24a — a player shuffles their library. The event's subject is
    /// the player.
    ///
    /// **An event because abilities trigger on it** (CR 701.24b, e, f —
    /// Psychic Surgery's "whenever an opponent shuffles their library"), and
    /// because the chokepoint invariant admits no other writer of a library's
    /// order in play: `GameState::shuffle_library` is the performer, and the
    /// pre-game shuffle in `Game::setup` is the one direct call, before any
    /// event can be watched. No `EventPattern` arm, like `Attach` and
    /// `CreateTokenIn`: nothing printed says "would shuffle … instead".
    ///
    /// Proposed by `Primitive::ShuffleLibrary`; a "shuffle it into its owner's
    /// library" is a `ZoneChange` of its own ahead of this (CR 701.24c).
    /// Announced as
    /// [`GameEvent::LibraryShuffled`](crate::events::event::GameEvent::LibraryShuffled).
    ShuffleLibrary {
        player: PlayerId,
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

    /// CR 106.6a / 106.12b — **the mana production event**: a spell or
    /// ability adds mana to a player's pool. The event's subject is `player`.
    ///
    /// Proposed by `resolve_mana_effect` for a mana ability and
    /// `Primitive::ProduceMana` for a spell, performed by the one arm that
    /// writes the pool and announces [`GameEvent::ManaAdded`]. The subject is
    /// the player and not the permanent because CR 616.1's chooser is "the
    /// affected player" and a spell's production (Dark Ritual) has no
    /// permanent; which permanent was tapped is `EventPattern::ProduceMana`'s
    /// `source` question.
    ///
    /// `mana` is the plain units by type and `special` the restricted ones,
    /// **one [`ManaAtom`] per unit**, because CR 106.6a's "any restrictions …
    /// will apply to all mana produced" is a fact about each unit: a
    /// multiplier repeats the atoms. A `Vec` and not a map, since the event
    /// reaches the log.
    ///
    /// `tapped_for_mana` is CR 106.12's definition — a mana ability of that
    /// permanent whose activation cost includes {T} — read off the ability's
    /// costs at activation, never off the payment plan. A spell (CR 605.5b)
    /// and a triggered mana ability (CR 605.1b) both propose `false`, which is
    /// why Mana Reflection doubles neither. Read by a replacement off this
    /// proposal (CR 106.12b) and by a trigger off the performed event
    /// (CR 106.12a); no rewrite touches it.
    ProduceMana {
        player: PlayerId,
        source: ObjectId,
        mana: Vec<(ManaType, u64)>,
        special: Vec<ManaAtom>,
        tapped_for_mana: bool,
    },

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
    /// `apply_replacements` sits between the proposal and the mutation
    /// (CR 614, 615, 616) and may modify or drop the action before it is
    /// performed — the one-member form of `execute_actions`.
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
        let (mark, window) = (self.events.next_seq(), self.events.current_stamp().batch);
        // A rider's proposals continue the replaced event's applied set
        // (CR 614.5, `Rider::lineage`); every other batch starts fresh. Taken, not
        // read: the batches nested inside the rider's own event are contained and
        // carry sets of their own. Put back afterwards for the rider's next proposal.
        let rider_lineage = self.rider_lineage.take();
        let empty = HashSet::new();
        let inherited = rider_lineage.as_ref().unwrap_or(&empty);
        // A new lineage either way for the decomposition count: a rider's
        // draw inside a doubled draw starts counting from zero, or the
        // invariant `execute_actions_decomposing` asserts — depth bounded by
        // the inherited set — would be asked across two lineages at once.
        let outer_depth = std::mem::replace(&mut self.nesting.decomposition_depth, 0);
        self.nesting.batch_depth += 1;
        let result = self.execute_batch_inner(batch, ctx, inherited);
        self.nesting.batch_depth -= 1;
        self.nesting.decomposition_depth = outer_depth;
        self.rider_lineage = rider_lineage;
        self.events.close_batch(previous);
        // CR 603.2, at the close of the *outermost* batch and after its riders:
        // the window is the event, so two permanents entering together each
        // see the other's static abilities before either is checked (603.6a),
        // and a "one or more" trigger reads the batch (603.2c). A nested call
        // joined this window and dispatches nothing of its own.
        self.dispatch_after_batch(result, mark, window, ctx)
    }

    /// The dispatch at a batch door: the window, when this was the outermost
    /// batch and it performed. `triggers-architecture.md` §4.1.
    fn dispatch_after_batch(
        &mut self,
        result: Result<Vec<GameAction>, String>,
        mark: crate::events::event::EventSeq,
        window: Option<crate::events::event::BatchId>,
        ctx: &ActionContext,
    ) -> Result<Vec<GameAction>, String> {
        let performed = result?;
        if self.nesting.batch_depth == 0 {
            self.dispatch_batch(mark, window, ctx)?;
        }
        Ok(performed)
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
        self.nesting.decomposition_depth += 1;
        debug_assert!(
            self.nesting.decomposition_depth <= inherited.len() + 1,
            "a decomposed event at depth {} inherited only {} applied effects: \
             CR 614.5's set is what bounds the nesting, so the lineage is broken \
             (replacement-architecture.md section 3.2d)",
            self.nesting.decomposition_depth,
            inherited.len()
        );
        let previous = self.events.open_batch(ctx.resolution_stamp());
        let (mark, window) = (self.events.next_seq(), self.events.current_stamp().batch);
        self.nesting.batch_depth += 1;
        let result = self.execute_batch_inner(batch, ctx, inherited);
        self.nesting.batch_depth -= 1;
        self.events.close_batch(previous);
        self.nesting.decomposition_depth -= 1;
        self.dispatch_after_batch(result, mark, window, ctx)
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
        let (mark, window) = (self.events.next_seq(), self.events.current_stamp().batch);
        // A new lineage, as in `execute_actions`.
        let outer_lineage = std::mem::replace(&mut self.nesting.decomposition_depth, 0);
        self.nesting.batch_depth += 1;
        let result = self.execute_batch_inner(batch, ctx, &HashSet::new());
        self.nesting.batch_depth -= 1;
        self.nesting.decomposition_depth = outer_lineage;
        self.events.close_batch(previous);
        // Its own window, dispatched at its own close, mid-phase-1 of the
        // enclosing batch — CR 614.13's moves are not a result of the entry
        // they are nested inside, so a devoured creature's death triggers
        // before the entry's ETB (§4.1).
        let performed = result?;
        self.dispatch_batch(mark, window, ctx)?;
        Ok(performed)
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
        use crate::engine::trace_records;

        // CR 104.1 — "a game ends immediately". Asked at the chokepoint so every
        // proposal after the batch that ended the game stops at one line: the rest
        // of a resolution, a decomposition's remaining inners, the ending batch's
        // riders (Stunning Reversal's survivor wins before the rider's seven draws).
        // The ending batch's own members all perform; they were one event.
        if self.result.is_some() {
            return Ok(Vec::new());
        }

        // Loud rather than a draw: a nesting this deep is a lost lineage, the
        // engine's mistake (`BATCH_NESTING_LIMIT`), and a rules answer would hide
        // it. The `Err` unwinds to the harness, which counts it as an error.
        self.diagnostics.record_batch_depth(self.nesting.batch_depth as u64);
        if self.nesting.batch_depth > BATCH_NESTING_LIMIT {
            return Err(format!(
                "batches nested {} deep, past the {} any legitimate chain reaches: a proposal \
                 loop the applied set did not end (CR 614.5), so a nested batch lost its \
                 lineage or a rider re-proposes its own event",
                self.nesting.batch_depth, BATCH_NESTING_LIMIT
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
        // CR 616.1's last sentence sends simultaneous choices to CR 101.4's APNAP
        // order, and this is the only place that can order them.
        //
        // **The unit is the subject group, not the member** (§9's RD decision 3):
        // members about one object or player share one CR 616.1 loop and one
        // applied set, so an effect applies to the pair once (CR 122.1c's "only
        // one shield counter is removed") while N deaths stay N subjects. A shared
        // subject means a shared chooser, so the APNAP order between players holds.
        //
        // CR 101.4d's restart is not implemented: nothing in phase 1 can *create*
        // a replacement effect, and each group's 616.1f loop completes before the
        // next begins. A simplification, deferred in RB's review.
        //
        // CR 614.13a/b's exclusion sets and CR 615.7's allocation answers belong
        // to *these* simultaneous entries, saved and restored like the event
        // stamp, and populated before the first member is decided because 614.13a
        // is about what is entering rather than what has entered.
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

        // The trace sink's first emit point: the proposals as they entered,
        // and the subject groups CR 616.1 will decide them in.
        self.trace(|| trace_records::batch(self, &batch, &groups, inherited.len()));

        let mut riders: Vec<Rider> = Vec::new();
        let mut decided: Vec<Option<GameAction>> = vec![None; batch.len()];
        // What each member's CR 616.1 loop applied, carried into phase 2 for the
        // one performer that decomposes. Per member only because `decided` is
        // indexed that way; a group's members share one set.
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

        // CR 603.10 decides a leaves-the-battlefield trigger from "the existence
        // of those abilities ... immediately prior to the event". A surviving
        // object's abilities change across this batch only if it removes the
        // source of an effect that copies, grants or removes abilities, and then
        // the lists from before are saved now, while they still exist
        // (`LookBackSnapshot`). Asked of the decided members, since replacement
        // decides whether anything departs.
        let snapshot = self
            .departs_an_ability_list_source(&decided)
            .then(|| self.look_back_frames())
            .filter(|frames| !frames.is_empty());
        // The departing members' own frames, for the same reason: taken as each
        // moved, a later one would see an earlier one gone (item 174).
        let frames_from = self.departure_frames.len();
        self.capture_departure_frames(&decided);
        // The dispatch audit's snapshot of every object that could carry a
        // triggered ability, when it is on (§4.10).
        let audit_frames = self.audit_frames();
        let performed_from = self.events.next_seq().0;

        // --- Phase 2: perform, in batch order -------------------------------
        //
        // Batch order, not APNAP: the choices were what CR 101.4 sequences, and
        // the performed events are simultaneous. The order is still observable
        // (a graveyard is ordered), and it is the caller's `battlefield_ids_ordered`.
        // Rendered ahead of the loop that consumes `decided`, and only in a
        // traced game: the `batch_end` record says what each proposal became.
        let decided_rendered: Option<Vec<Option<String>>> = self.trace_on().then(|| {
            decided
                .iter()
                .map(|d| d.as_ref().map(crate::state::trace::render_debug))
                .collect()
        });
        let mut performed = Vec::with_capacity(decided.len());
        let mut performed_ok = Ok(());
        // Lifelink's gain is a result of this batch's damage (CR 120.3f), one
        // event per source across its members (CR 702.15e), proposed once they
        // have all performed and inside the batch (CR 120.4c). Damage dealt by a
        // nested batch or a CR 615.5 rider is summed by that call's own loop, so
        // it gains in a separate event and is never counted twice.
        let mut lifelink_gains = Vec::new();
        for (i, action) in decided.into_iter().enumerate() {
            let Some(action) = action else { continue };
            if let Err(e) = self.perform_action(action.clone(), ctx, &applied_to[i]) {
                performed_ok = Err(e);
                break;
            }
            if let GameAction::DealDamage { source, amount, .. } = &action {
                add_lifelink_gain(self, &mut lifelink_gains, *source, *amount);
            }
            performed.push(action);
        }
        if performed_ok.is_ok() {
            for gain in lifelink_gains {
                if let Err(e) = self.execute_action(
                    GameAction::GainLife { player: gain.player, amount: gain.amount, source: gain.source },
                    ctx,
                ) {
                    performed_ok = Err(e);
                    break;
                }
            }
        }
        self.departure_frames.truncate(frames_from);
        performed_ok?;
        self.trace(|| {
            trace_records::batch_end(self, decided_rendered.as_deref().unwrap_or(&[]), riders.len())
        });
        let window = self.events.current_stamp().batch;
        let performed_range = performed_from..self.events.next_seq().0;
        if let Some(frames) = snapshot {
            self.look_back_snapshots.push(crate::engine::triggers::LookBackSnapshot {
                window,
                performed: performed_range.clone(),
                frames,
            });
        }
        if let Some(frames) = audit_frames {
            self.file_audit_snapshot(crate::engine::triggers::LookBackSnapshot {
                window,
                performed: performed_range,
                frames,
            });
        }

        // CR 104.2a / 104.4a are read off the batch, not off a member: two players
        // losing in one state-based check is one simultaneous event whose outcome
        // is a draw, and a performer that asked "is anyone left" after the first
        // would have crowned the second (Stunning Reversal's four-player ruling).
        if performed.iter().any(|a| matches!(a, GameAction::PlayerLoses { .. })) {
            self.settle_game_result();
        }

        // --- Phase 3: the queued riders, in application order ----------------
        //
        // "Immediately afterward" (CR 615.5) means after the events happened, not
        // mid-loop. Unconditional once queued (CR 615.12), so this runs even for a
        // member whose event was dropped entirely.
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
    /// for which rule gives it this timing (CR 615.5 for a prevention effect,
    /// CR 614.1a/614.6 for a plain replacement).
    ///
    /// Its `ResolutionContext` names the event's subject as the single resolved
    /// target, so a `then` written with `EffectRecipient::Target` acts on the
    /// object *or the player* the replacement was about and one written with
    /// `EffectRecipient::Controller` acts for the effect's own controller. The
    /// actions it proposes **carry the replaced event's applied set**
    /// (`Rider::lineage`, CR 614.5): they are the rest of the replacement's
    /// effect. Events nested inside them are contained and start fresh.
    ///
    /// A player subject becomes a `ResolvedTarget::Player` rather than being
    /// flattened away: Angel of Suffering's "prevent that damage and mill twice
    /// that many cards" mills the player the damage was aimed at.
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
            // CR 615.5's rider acts on the replaced event's subject, which is
            // one thing and therefore one instance.
            targets: crate::engine::targeting::ChosenTargets::one(vec![match rider.subject {
                EventSubject::Object(id) => ResolvedTarget::Object(id),
                EventSubject::Player(pid) => ResolvedTarget::Player(pid),
            }]),
            replaced_amount: rider.replaced_amount,
            damage_prevented: Some(rider.prevented),
            trigger: None,
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
    /// Separated from `execute_action` so the replacement pipeline can call it
    /// with the final, possibly-modified action.
    ///
    /// `lineage` is this event's own CR 616.1 applied set, and exactly one arm
    /// reads it — the one that **decomposes**. Every other nested proposal here
    /// is **containment** and takes the fresh set `execute_action` gives it
    /// (`plans/glossary.md`; `replacement-architecture.md` §3.2d).
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
                // No 0-amount guard: CR 614.7a makes a 0-damage event never happen, which
                // is the *proposal's* problem — `replacement::never_happens` owns it ahead
                // of the pipeline, and a 0 that reaches CR 616.1 is one a prevention
                // effect applied to.
                //
                // **CR 120.3 is a list of results, and this arm is that list**: each result
                // is decided off the target's own type, so a creature planeswalker gets
                // 120.3c *and* 120.3e. The four absent results have owners — 120.3b/g
                // (poison) and 120.3d (wither, infect) are `backlog.md` §2.6's, 120.3h
                // (battles) is §2.23's.
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
                // Gated on the type: an object that is neither a creature nor a
                // planeswalker takes no result at all.
                if results.mark_damage
                    && let DamageTarget::Object(id) = &target {
                    self.battlefield.get_mut(id).expect("membership checked")
                        .damage_marked += amount as u32;
                }

                apply_deathtouch_flag(self, source, &target);

                // CR 903.10a — if a commander deals combat damage to a
                // player, accumulate it per-commander on the damaged player.
                // The 21-damage loss check is the SBA at CR 704.6c.
                if is_combat
                    && let DamageTarget::Player(pid) = &target {
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

                self.emit_event(GameEvent::DamageDealt {
                    source_id: source,
                    target,
                    amount,
                    is_combat,
                });

                // > 120.3a Damage dealt to a player causes that player to lose
                // > that much life.
                //
                // **A contained proposal, not a subtraction**, joining this damage's
                // batch as lifelink's gain does (CR 120.4c/d): a CR 603.2c trigger sees one
                // event, and the loss re-enters `apply_replacements` with a fresh applied
                // set (§3.2d) so Bloodletter can double it while a shield that already
                // applied to the damage does not apply again. `LoseLife` has no pattern
                // arm, so Ali from Cairo's family is unwritable rather than wrongly
                // answered. After the `DamageDealt` emit; the loss's own performer emits
                // `LifeChanged`, with `LifeLossCause::Damage` keeping `source` on it.
                if results.lose_life
                    && let DamageTarget::Player(pid) = &target {
                    self.execute_action(
                        GameAction::LoseLife {
                            player: *pid,
                            amount,
                            cause: LifeLossCause::Damage { source },
                        },
                        _ctx,
                    )?;
                }

                // > 120.3c Damage dealt to a planeswalker causes that many
                // > loyalty counters to be removed from that planeswalker.
                //
                // A proposal because CR 614.16's counter doublers replace "counters would
                // be put on", and a removal a card watches is the mirror of one. `n` is a
                // ceiling (`PermanentState::remove_counters` reports what it took), and
                // CR 704.5i does the killing.
                if results.remove_loyalty
                    && let DamageTarget::Object(id) = &target {
                    self.execute_action(
                        GameAction::RemoveCounters {
                            subject: CounterSubject::Object(*id),
                            counter: CounterType::Loyalty,
                            n: amount as u32,
                        },
                        _ctx,
                    )?;
                }

                Ok(())
            }

            // > 121.2. Cards may only be drawn one at a time. If a player is
            // > instructed to draw multiple cards, that player performs that
            // > many individual card draws.
            //
            // Literal, because of CR 121.6b: a replacement of one draw in a sequence
            // "is completed before resuming the sequence", so each inner is its own
            // batch, decided and its riders run before the next is proposed. `lineage`
            // is what each inner starts its CR 614.5 applied set from — these are this
            // instruction at finer grain, not events it caused (§3.2d), which is the
            // difference between two Teferi's Ageless Insights drawing four and the
            // game hanging. No guard on `n == 0`: the loop is the no-op.
            GameAction::DrawCards { player, n, cause } => {
                for i in 0..n {
                    self.execute_actions_decomposing(
                        vec![GameAction::DrawCard {
                            player,
                            // CR 121.1's turn-based action is one card; every draw after an
                            // instruction's first belongs to the effect that produced it, which
                            // is what "except the first one you draw in each of your draw steps"
                            // means once a doubler has applied to the draw step's draw.
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

                self.emit_event(GameEvent::LifeChanged {
                    player_id: player,
                    old: old_life,
                    new: new_life,
                    source: Some(source),
                    cause: None,
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

                self.emit_event(GameEvent::LifeChanged {
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
                    cause: Some(cause),
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
                self.emit_event(GameEvent::Untapped { object_id: object });
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
                    self.emit_event(GameEvent::Attached { attachment, host, former_host });
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
                self.emit_event(GameEvent::Tapped { object_id: object });
                Ok(())
            }

            // A count of zero is a no-op here and not in `replacement::never_happens`:
            // CR 614.7a is about damage and life gain, and "one or more" is asked by
            // the pattern (`gather::pattern_watches`), so a count a halving took to
            // zero matches nothing further and arrives here.
            GameAction::AddCounters { subject, counter, n, by: _ } => {
                if n == 0 {
                    return Ok(());
                }
                match subject {
                    CounterSubject::Object(object) => {
                        if !self.battlefield.contains_key(&object) {
                            return Err(format!(
                                "Cannot put counters on {}: not on the battlefield", object
                            ));
                        }
                        self.add_counters(object, counter, n);
                        // A permanent that just gained a CR 122.1 replacement counter is a
                        // replacement source now, and nothing has to be recorded: counters are
                        // scanned, not cached (`gather::any_replacement_counter`).
                    }
                    CounterSubject::Player(player) => {
                        self.get_player_mut(player)?.add_counters(counter, n);
                    }
                }
                self.emit_event(GameEvent::CountersChanged { subject, counter, added: n as i32 });
                Ok(())
            }

            GameAction::RemoveCounters { subject, counter, n } => {
                if n == 0 {
                    return Ok(());
                }
                // CR 701.2 — do as much as possible. Both removers report how
                // many were actually there, and a removal of nothing is not an
                // event: CR 603.2e's transition rule is the same shape the
                // `Tap`/`Untap` arms follow.
                let removed = match subject {
                    CounterSubject::Object(object) => {
                        if !self.battlefield.contains_key(&object) {
                            return Err(format!(
                                "Cannot remove counters from {}: not on the battlefield", object
                            ));
                        }
                        self.remove_counters(object, counter, n)
                    }
                    CounterSubject::Player(player) => {
                        self.get_player_mut(player)?.remove_counters(counter, n)
                    }
                };
                if removed == 0 {
                    return Ok(());
                }
                self.emit_event(GameEvent::CountersChanged {
                    subject,
                    counter,
                    added: -(removed as i32),
                });
                Ok(())
            }

            // CR 701.8a — "move it from the battlefield to its owner's graveyard".
            // The **outer** event: this performer proposes the inner zone change,
            // which re-enters the pipeline with a fresh applied set (§3.2d
            // containment). That is what lets CR 122.1h's finality counter turn the
            // graveyard trip into an exile while CR 122.1c's shield counter, watching
            // the destruction itself, has already declined.
            GameAction::Destroy { object, source } => {
                // Loud, like `Tap`/`Untap`: destroying something off the battlefield does
                // nothing (CR 701.8b), and CR 608.2b's partial resolution is the caller's
                // check (`CLAUDE.md`); a lenient arm would only hide a future bug.
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
            // Three small performers, each writing the one field that makes its unit
            // "the one that is happening" and announcing the `GameEvent` item 6's
            // "at the beginning of" triggers will read. None runs a turn-based action
            // or an expiry: those are separate events (CR 703.4, 500.4) the drainer
            // runs after the proposal survives — see `engine::turns`.
            GameAction::BeginTurn { player, turn } => {
                // `begin_turn` is the one writer of `last_turn_began`, so a
                // turn that is skipped starts no CR 302.6 clock and expires no
                // "until your next turn" effect (CR 614.10a).
                self.begin_turn(turn, player);
                self.priority_player = player;
                self.emit_event(GameEvent::TurnBegin { player, turn_number: turn });
                Ok(())
            }

            GameAction::BeginPhase { phase, player } => {
                // `Phase::new` would fill in the phase's first step; the step is
                // its own proposal, and a phase whose first step is skipped
                // must not look as though that step is happening.
                self.phase = Phase { phase_type: phase, step: None };
                self.emit_event(GameEvent::PhaseBegin { phase, player });
                Ok(())
            }

            GameAction::BeginStep { step, player } => {
                self.phase.step = Some(step);
                self.emit_event(GameEvent::StepBegin { step, player });
                Ok(())
            }

            // > 701.22a To "scry N" means to look at the top N cards of your
            // > library, then put any number of them on the bottom of your
            // > library in any order and the rest on top of your library in
            // > any order.
            //
            // The whole keyword action is in this arm and it proposes nothing:
            // "top" and "bottom" are positions inside one library, not zones
            // (CR 400.1), so there is no `ZoneChange` for a replacement to watch.
            // `looked_at` is what is actually there — a short library gives fewer,
            // and CR 701.22d makes that still a scry. `n = 0` never arrives
            // (CR 701.22b; `replacement::never_happens`).
            GameAction::Scry { player, n } => {
                let library = &self.get_player(player)?.library;
                let k = (n as usize).min(library.len());
                // The library's last element is its top (`Primitive::Mill`
                // reads the same end), so the cards looked at, top-most
                // first, are its tail reversed.
                let looked_at: Vec<ObjectId> =
                    library.iter().rev().take(k).copied().collect();
                let source = _ctx.resolution.map(|r| r.source);
                let (top, bottom) = ask_scry(_ctx.dp, self, player, &looked_at, n, source);

                // Rebuilt rather than rotated, since each group may have been reordered.
                // Bottom-most first is the vector's direction, so each group goes in
                // reversed: element 0 of `top` and `bottom` is the card nearest the top.
                let p = self.get_player_mut(player)?;
                let keep = p.library.len() - k;
                let mut rebuilt: Vec<ObjectId> = Vec::with_capacity(p.library.len());
                rebuilt.extend(bottom.iter().rev().copied());
                rebuilt.extend(p.library[..keep].iter().copied());
                rebuilt.extend(top.iter().rev().copied());
                p.library = rebuilt;

                // CR 701.22d — announced after the process, and announced even
                // when the library had nothing to look at. `k` is Elrond,
                // Master of Healing's count and `n` the instruction's; the
                // two differ exactly when the library was short.
                self.emit_event(GameEvent::Scried {
                    player_id: player,
                    n,
                    looked_at: k as u64,
                });
                Ok(())
            }

            // CR 701.24a — the one in-game writer of a library's order. Announced
            // even for a library of zero or one cards, which is CR 701.24e's
            // "abilities that trigger when a library is shuffled will still trigger".
            GameAction::ShuffleLibrary { player } => {
                self.get_player(player)?;
                self.shuffle_library(player);
                self.emit_event(GameEvent::LibraryShuffled { player_id: player });
                Ok(())
            }

            // --- The game's end (CR 104) ------------------------------------
            //
            // Loud on a player who has already left: the SBA check gates on
            // `player_lost` before proposing, so a second loss is a caller bug and
            // not CR 800.4a. `has_drawn_from_empty_library` is **not** cleared here —
            // CR 704.5b's window closes at the check that reads it, whether or not
            // the loss was then replaced or refused (`check_state_based_actions`).
            // CR 104.2a and 104.4a are the *batch*'s question, settled in
            // `execute_batch_inner` once the whole batch has performed.
            GameAction::PlayerLoses { player, reason } => {
                if self.player_lost[player] {
                    return Err(format!("player {} has already left the game", player));
                }
                self.player_lost[player] = true;
                self.emit_event(GameEvent::PlayerLost { player_id: player, reason });
                // CR 104.3 — a player who loses leaves — and CR 800.4a's four clauses
                // follow here rather than at the next state-based check: "it happens as
                // soon as the player leaves the game". Clause 4's exile is a result of
                // the departure, so its nested batch joins this one (§4.2).
                self.player_left_the_game(player, _ctx)
            }

            // CR 104.1 — "immediately". Recorded here and read by everything that
            // asks whether the game is over; a recorded result is never overwritten.
            // CR 104.3f — win and lose at once → lose — has no producer and is
            // recorded, not built.
            GameAction::PlayerWins { player } => {
                if self.player_lost[player] {
                    return Err(format!("player {} has left the game and cannot win it", player));
                }
                if self.result.is_none() {
                    self.result = Some(GameResult::Winner(player));
                }
                self.emit_event(GameEvent::PlayerWon { player_id: player });
                Ok(())
            }

            // CR 106.6a / 106.12b — the one writer of the pool.
            //
            // A production of nothing performs and announces nothing, and the guard
            // is here rather than in `never_happens`: no rule makes a zero production
            // no event (CR 106.5 is about an *undefined type*; CR 107.1b's Viridian
            // Joiner "adds no mana" is a statement about the pool).
            //
            // The announced `mana` folds the restricted atoms into the plain counts by
            // type, in proposal order: a restriction "doesn't affect the mana's type"
            // (CR 106.6), so the log reports {G}{G} for a doubled restricted Forest
            // and the pool keeps the restriction.
            GameAction::ProduceMana { player, source, mana, special, tapped_for_mana } => {
                if mana.iter().all(|(_, n)| *n == 0) && special.is_empty() {
                    return Ok(());
                }
                self.diagnostics.record_mana_production();
                let pool = &mut self.get_player_mut(player)?.mana_pool;
                for (mana_type, n) in &mana {
                    if *n > 0 {
                        pool.add(*mana_type, *n);
                    }
                }
                for atom in &special {
                    pool.add_special(atom.clone());
                }
                let mut announced: Vec<(ManaType, u64)> = Vec::with_capacity(mana.len());
                let units = mana.iter().copied().chain(special.iter().map(|a| (a.mana_type, 1)));
                for (mana_type, n) in units {
                    if n == 0 {
                        continue;
                    }
                    match announced.iter_mut().find(|(t, _)| *t == mana_type) {
                        Some((_, total)) => *total += n,
                        None => announced.push((mana_type, n)),
                    }
                }
                self.emit_event(GameEvent::ManaAdded {
                    player_id: player,
                    source_id: source,
                    mana: announced,
                    tapped_for_mana,
                });
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

        // CR 603.10a — the frame the batch took before any member moved
        // (`take_departure_frame`). A permanent, not merely an object in the
        // zone: a token whose entry is being decided has no entity and nothing
        // to look back at (`create_tokens`).
        let lki = if from == Zone::Battlefield && self.battlefield.contains_key(&object) {
            self.take_departure_frame(object)
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
        lki: Option<std::sync::Arc<EffectiveCharacteristics>>,
    ) -> Result<(), String> {
        let owner = self.get_object(object)?.owner;
        self.emit_event(GameEvent::ZoneChange { object_id: object, owner, from, to, cause, lki });
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
        self.emit_event(GameEvent::TokenCreated { object_id: object, owner, zone });
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
            // CR 111.2 — a token's owner is the controller of the effect that created
            // it, and `is_token` (CR 111.1) is what CR 704.5d and `ObjectFilter::Token`
            // read. Both set before it reaches the battlefield, since
            // `register_static_effects` runs inside `place_on_battlefield`.
            let mut obj = GameObject::new(data, controller, Zone::Battlefield);
            obj.is_token = true;
            let id = self.add_object(obj);
            ids.push(id);
            if let Some(mut entry) = self.entry_proposal(id, None, controller, None) {
                // "Create a tapped …" is the creating effect's own word on
                // how the token enters, so it joins the seed the rules give
                // the entry, ahead of any replacement (CR 614.1c).
                if def.enters_tapped
                    && let GameAction::EnterBattlefield { mods, .. } = &mut entry {
                    mods.merge(&EnterMods::tapped());
                }
                entries.push(entry);
            }
        }
        let performed = self.execute_actions(entries, ctx)?;
        let created: IdSet<ObjectId> = performed
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
        game.record_events();

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
        game.recorded_events().events().filter_map(|e| match e {
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
        assert_eq!(game.recorded_events().len(), 1);
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
        assert_eq!(game.recorded_events().len(), 2);
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
        assert_eq!(game.recorded_events().len(), 0);
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
        assert_eq!(game.recorded_events().len(), 1);
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
        assert_eq!(game.recorded_events().len(), 1);
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
        game.record_events();

        let data = CardDataBuilder::new("Lifelink Creature")
            .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::White], 1))
            .color(crate::types::colors::Color::White)
            .card_type(CardType::Creature)
            .power_toughness(2, 3)
            .keyword_flag(KeywordFlag::Lifelink)
            .build();

        let obj = GameObject::new(data, 0, Zone::Battlefield);
        let id = game.add_object(obj);
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

        // Events: DamageDealt, LifeChanged (damage to P1), LifeChanged (the
        // lifelink gain for P0, which the batch proposes once its damage has
        // all been dealt, CR 702.15e).
        let life_events: Vec<_> = game.recorded_events().events().filter_map(|e| {
            if let GameEvent::LifeChanged { player_id, old, new, source, .. } = e {
                Some((*player_id, *old, *new, *source))
            } else {
                None
            }
        }).collect();

        assert_eq!(life_events.len(), 2);

        // P1 lost 2 life from the damage
        let (pid, old, new, src) = life_events[0];
        assert_eq!(pid, 1);
        assert_eq!(old, 20);
        assert_eq!(new, 18);
        assert_eq!(src, Some(lifelinker));

        // P0 gained 2 life from lifelink
        let (pid, old, new, src) = life_events[1];
        assert_eq!(pid, 0);
        assert_eq!(old, 20);
        assert_eq!(new, 22);
        assert_eq!(src, Some(lifelinker));
    }

    #[test]
    fn test_simultaneous_lifelink() {
        // Two lifelink creatures deal damage; each produces its own LifeChanged event.
        let mut game = GameState::new(2, 20);
        game.record_events();

        let make_lifelinker = |game: &mut GameState, name: &str| -> ObjectId {
            let data = CardDataBuilder::new(name)
                .mana_cost(crate::types::mana::ManaCost::build(&[ManaType::White], 1))
                .color(crate::types::colors::Color::White)
                .card_type(CardType::Creature)
                .power_toughness(2, 2)
                .keyword_flag(KeywordFlag::Lifelink)
                .build();
            let obj = GameObject::new(data, 0, Zone::Battlefield);
            let id = game.add_object(obj);
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
        let lifelink_gains: Vec<_> = game.recorded_events().events().filter_map(|e| {
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
        let id = game.add_object(obj);
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
        assert!(!game.players[1].commander_damage_taken.contains_key(&cmdr));
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

        assert!(!game.players[1].commander_damage_taken.contains_key(&bears_id));
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
            let id = game.add_object(obj);
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

        let life_events: Vec<_> = game.recorded_events().events().filter_map(|e| {
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
        game.nesting.batch_depth = BATCH_NESTING_LIMIT;

        let err = game
            .execute_action(GameAction::Tap { object: bears }, &test_ctx())
            .unwrap_err();

        assert!(err.contains("nested"), "{err}");
        assert!(game.result.is_none(), "an engine loop is an error, never a rules answer");
    }
}
