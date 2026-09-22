use std::collections::HashSet;
use std::sync::Arc;

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::events::event::EventLog;
use crate::objects::object::GameObject;
use crate::state::battlefield::PermanentState;
use crate::state::continuous_effects::ContinuousEffectRegistry;
use crate::state::diagnostics::Diagnostics;
use crate::state::layer_memo::LayerMemo;
use crate::state::replacement_effects::{
    EntrySelectionScope, PreventionAllocationScope, ReplacementEffectRegistry,
};
use crate::state::restrictions::RestrictionRegistry;
use crate::state::player::PlayerState;
use crate::types::costs::{AdditionalCost, AlternativeCost};
use crate::types::effects::{CounterType, Effect};
use crate::types::ids::{
    AbilityId, IdMap, IdSet, ObjectId, ObjectRef, PlayerId, Timestamp, ZoneChangeEpoch,
};
use crate::types::zones::Zone;
use crate::types::replacement::{EnterMods, ReplacementDef};
use crate::types::triggers::EventKindMask;

/// The outcome of a game that has ended (CR 104).
///
/// Lives on [`GameState::result`] rather than on the lifecycle wrapper because
/// CR 104.1 says a game ends *immediately*, and the moment it ends is inside a
/// performer — a `GameAction::PlayerWins` (104.2b), or the settlement of a batch
/// that performed one or more `PlayerLoses` (104.2a, 104.4a). `Game` reads it;
/// nothing outside `engine::actions` writes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameResult {
    Winner(PlayerId),
    Draw,
}

/// Metadata for a spell or ability on the stack.
///
/// This is the sidecar state for stack objects, analogous to how
/// `PermanentState` is the sidecar for battlefield permanents.
/// Created when a spell is cast or ability is activated, consumed
/// when the stack entry resolves or is removed.
#[derive(Debug, Clone)]
pub struct StackEntry {
    /// The object ID of this stack entry (matches the key in `stack`)
    pub object_id: ObjectId,
    /// The player who controls this spell/ability
    pub controller: PlayerId,
    /// CR 601.2c's instances of the word "target", in printed order: what each
    /// was announced against, and what was chosen for it. Locked in at
    /// cast/activation time.
    ///
    /// **One entry per instance, not per target and not per atom.** CR 115.3
    /// makes the instance the unit — "the same target can't be chosen multiple
    /// times for any one instance … the same object can be chosen once for each
    /// instance" — so Decimate's four clauses are four entries that may share an
    /// artifact land, while Victimize's "two target creature cards" is one entry
    /// holding two distinct cards. CR 603.3d asks the same question of a
    /// triggered ability, which is why this is the shape rather than a flat list
    /// with a width.
    ///
    /// **Each instance records its own clause** rather than the entry recording
    /// one, and for the reason the single `recipient` field had: the two must be
    /// the same question at CR 601.2c and at CR 608.2b, and an Aura's comes from
    /// its enchant ability (`CardData::spell_instances`), which `effect` cannot
    /// show. The resolution reads its clauses off this record too
    /// (`targeting::DeclaredInstances`).
    pub chosen_targets: Vec<crate::engine::targeting::TargetInstance>,
    /// Modes chosen at cast time (for modal spells, future-proofed)
    pub chosen_modes: Vec<usize>,
    /// X value if the spell has a variable cost
    pub x_value: Option<u64>,
    /// The effect to resolve (copied from CardData at cast time)
    pub effect: Effect,
    /// Whether this is a spell (true) or an ability (false).
    /// Spells go to graveyard after resolution; abilities cease to exist.
    pub is_spell: bool,
    /// The alternative cost chosen for this spell, if any (rule 118.9).
    /// At most one alternative cost may be chosen per cast.
    pub chosen_alternative_cost: Option<AlternativeCost>,
    /// Additional costs that were paid for this spell (rule 118.8).
    /// Multiple additional costs can be paid (e.g. kicker + buyback).
    pub additional_costs_paid: Vec<AdditionalCost>,
    /// The zone this spell was cast from (CR 601.2a), captured before the card
    /// moved to the stack.
    ///
    /// **Invariant: `cast_from.is_some() == is_spell`.** An activated ability is
    /// not cast from anywhere — CR 602.2a gives it a *source*, which is a
    /// different fact, and folding the two together is how a field starts
    /// drifting. `None` for abilities is the honest answer, not a missing value.
    ///
    /// Two known customers, neither implemented yet:
    /// - CR 903.8 commander tax, which counts casts **from the command zone**
    ///   specifically. The cast counter cannot be incremented correctly without
    ///   this, because a commander recast from hand after being bounced does not
    ///   add tax.
    /// - "Cast from exile" riders (Don't Blink and kin), which need the origin
    ///   at *resolution* time, by which point the card has already left it.
    ///
    /// Recorded at cast time because it is unrecoverable afterward: the object
    /// is on the stack and its `zone` field says so.
    pub cast_from: Option<Zone>,
    /// For an activated ability: which ability of which permanent this is.
    ///
    /// **Invariant: `ability_identity.is_some() == !is_spell`** — the mirror of
    /// [`Self::cast_from`], and for the same reason. An ability on the stack is
    /// a new object with a fresh `ObjectId` that ceases to exist on resolution
    /// (CR 608.2n), so the ephemeral id identifies nothing once it is gone.
    ///
    /// CR 603.7h needs the durable identity: a delayed trigger that fires when
    /// "this ability has resolved for the third time this turn" is counting
    /// *this* ability of *this* permanent (Ashling the Pilgrim), and neither
    /// half can be recovered from the ephemeral.
    pub ability_identity: Option<AbilityIdentity>,
    /// For a triggered ability: the def and the bound facts of its event
    /// (`triggers-architecture.md` §3.13). `None` for a spell and for an
    /// activated ability; the resolution reads the intervening "if" and
    /// "that object" off it.
    pub trigger: Option<crate::types::triggers::TriggerBinding>,
}

/// The stack object currently resolving, and the one thing about it that does
/// not survive resolution on its own. See [`GameState::resolving`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvingObject {
    pub id: ObjectId,
    /// CR 110.2b's **default** controller: "the player who put that spell onto
    /// the stack". Read off the `StackEntry` before resolution takes it.
    ///
    /// Not the effective controller. If an opponent stole the spell they control
    /// the permanent it becomes — but by a Layer 2 effect that CR 400.7a keeps
    /// applying, layered over this value, rather than by this value being
    /// theirs. The distinction is invisible until the effect ends, which is
    /// CR 800.4c and therefore 4-player Commander.
    pub default_controller: PlayerId,
    /// `StackEntry::cast_from`, carried the same way for the same reason:
    /// `place_on_battlefield` writes CR 400.7d's `PermanentState::cast` off
    /// it, and the entry is gone by then. `None` for an ability.
    pub cast_from: Option<Zone>,
}

/// Which ability of which object — the durable identity of an activated ability,
/// as opposed to the ephemeral stack object representing one activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AbilityIdentity {
    /// The permanent the ability was activated from, and which existence of
    /// it (CR 400.7): two activations across a bounce are two abilities'
    /// worth of counting (`triggers-architecture.md` §3.6). The same pair
    /// spelled once, since `ObjectRef` is that pair.
    pub source: ObjectRef,
    /// Which of its abilities, and for a granted one which grant
    /// (`AbilityId::granted_by`): two grants of one ability are two
    /// identities, each as long-lived as its grant (§3.6). Stable across
    /// activations; see `oracle::characteristics::get_effective_abilities`.
    pub ability: AbilityId,
}

/// How deep inside itself the engine is: three counters that answer one
/// question, so they are one field on `GameState` rather than three.
///
/// **Two of them are caps and one is not**, which is what "guard" is covering.
/// `batch_depth` and `dispatch_depth` are bounds in every build — the engine
/// stops with an `Err` naming the invariant it thinks it has lost.
/// `decomposition_depth` bounds nothing: CR 614.5's applied set already ends
/// that loop, and this counts the consequence (`d <= inherited.len() + 1`) in a
/// `debug_assert!`, so what it guards against is a *silent* lineage break —
/// the failure it converts is a stack overflow that takes the test binary with
/// it, into a red test that names the rule.
///
/// **On the state and not on a parameter**, all three, for `codebase-state.md`
/// item 40's reason: a nested call re-enters through `emit_event` and the
/// replacement pipeline, neither of which has a channel to thread a depth
/// through, and a clone taken at a prompt has to resume with the same answer.
#[derive(Debug, Clone, Default)]
pub(crate) struct NestingGuards {
    /// How many batches are nested inside one another right now — one per
    /// `execute_batch_inner` on the call stack, kept by its three wrappers.
    ///
    /// **A guard against the engine, not a rule.** Once every nested batch
    /// carries its lineage, CR 614.5 bounds every replacement chain (each level
    /// spends an instance), so a chain that nests without bound has lost a
    /// lineage and `execute_batch_inner` errors at `BATCH_NESTING_LIMIT` rather
    /// than answering with a rule. `decomposition_depth` is the per-lineage
    /// invariant asserted in debug builds; this is the whole-stack bound in
    /// every build, and `fuzz_games` reports the deepest nesting a run reached.
    pub(crate) batch_depth: usize,

    /// How many decomposing calls (`replacement-architecture.md` §3.2d) are on
    /// the stack — CR 121.2's draws inside draws, and nothing else today.
    ///
    /// **Not a cap.** CR 614.5's applied set is the loop's termination argument;
    /// this counts the invariant that set implies — a decomposing call at depth
    /// `d` exists because `d - 1` substitutions above it each inserted an
    /// instance, so `d <= inherited.len() + 1` — and
    /// `execute_actions_decomposing` asserts it in debug builds, turning a stack
    /// overflow into a red test that names the rule.
    ///
    /// **Per lineage, not per call stack.** A fresh-set batch (`execute_actions`,
    /// which a rider's proposal and every contained event go through) is a new
    /// lineage and zeroes this for its extent; counted across a rider the
    /// assertion fires on a legal board, while the loop it exists for is
    /// `batch_depth`'s.
    pub(crate) decomposition_depth: usize,

    /// Dispatches nested inside dispatches — a tier-2 trigger's
    /// `AbilityTriggered` dispatched from the dispatch that queued it. Bounded
    /// by the abilities present; past `engine::triggers::DISPATCH_NESTING_LIMIT`
    /// the dispatcher stops, which is the engine's mistake and not a rules answer.
    pub(crate) dispatch_depth: usize,
}

/// The complete state of a game of Magic.
///
/// All game objects live in the central `objects` store. Zones reference
/// objects by ID. This means zone transitions are just:
/// 1. Update the object's `zone` field
/// 2. Remove its ID from the old zone's collection
/// 3. Add its ID to the new zone's collection
/// 4. Initialize/clean up zone-specific state (e.g. PermanentState)
#[derive(Debug, Clone)]
pub struct GameState {
    // --- Central object store ---
    /// All game objects indexed by ID
    pub objects: IdMap<ObjectId, GameObject>,

    // --- Players ---
    pub players: Vec<PlayerState>,

    // --- Global zones (player zones are in PlayerState) ---
    /// The stack — LIFO order (last element = top of stack)
    pub stack: Vec<ObjectId>,
    /// Stack entry metadata — keyed by ObjectId
    pub stack_entries: IdMap<ObjectId, StackEntry>,
    /// The stack object currently resolving, if any.
    ///
    /// A rules question, not an engine artifact: `default_enter_controller`
    /// needs CR 110.2b's *default* controller — "the player who put that spell
    /// onto the stack" — and the `StackEntry` that recorded it has been taken by
    /// the time a permanent spell enters. The other two readers are the layer
    /// walk's controller lookup and `Primitive::Exile`'s "is the source still
    /// here" (CR 608.2m).
    ///
    /// Always `None` outside a resolution. `resolve_top_of_stack` clears it on
    /// every exit path, including the error ones.
    pub(crate) resolving: Option<ResolvingObject>,
    /// Battlefield state — keyed by ObjectId
    pub battlefield: IdMap<ObjectId, PermanentState>,

    /// How much work the engine has done this game — see
    /// [`Diagnostics`].
    ///
    /// **Diagnostic only; nothing may branch on it.** It is on `GameState`
    /// rather than in a thread-local for the reason `GameState.rng` is: ambient
    /// state that a game can reach is state a fork or a replay cannot account
    /// for. A read of these numbers changing behavior would make them
    /// unmeasurable, which is why every accessor is read-only and no engine
    /// module imports them.
    pub diagnostics: Diagnostics,
    /// One counter every write to a layer-walk input bumps — the coarse key
    /// of [`LayerMemo`]. Read through [`GameState::layer_epoch`], which folds
    /// in the registry's own count; written through
    /// [`GameState::bump_layer_epoch`]. → `layers-architecture.md` §12 "7a".
    layer_epoch: u64,
    /// The frames the walk has already computed at the current epoch. Read
    /// and filled by `compute_characteristics` and nothing else.
    pub(crate) layer_memo: LayerMemo,
    /// Exile zone
    pub exile: Vec<ObjectId>,
    /// Command zone
    pub command: Vec<ObjectId>,

    // --- Turn tracking ---
    pub turn_number: u32,
    /// The turn number on which each player's most recent turn began, indexed
    /// by `PlayerId`. `0` means that player has not had a turn yet.
    ///
    /// This is what CR 302.6 measures against, and it cannot be derived from
    /// `turn_number` once there are more than two players — or once extra turns
    /// exist — so it is recorded. Written only by [`GameState::begin_turn`].
    pub last_turn_began: Vec<u32>,
    pub active_player: PlayerId,
    pub priority_player: PlayerId,
    pub phase: Phase,
    /// CR 500.7's extra turns, **as a stack**: "the most recently created turn
    /// will be taken first".
    ///
    /// One entry per extra turn, naming the player who takes it and nothing
    /// else. Not a `(player, turn)` pair: the turn *number* is
    /// `turn_number + 1` computed when the turn actually begins, and a skipped
    /// turn advances no number (CR 614.10a), so a number stored here would go
    /// stale the first time a skip met a queued turn — a field that means one
    /// thing on Tuesday and another on Wednesday.
    ///
    /// Pushed by `Primitive::ExtraTurn` and drained by
    /// [`Self::next_turn_taker`], which is the **only** reader. Extra *phases*
    /// and *steps* (CR 500.8, 500.10) are this queue's second level and wait
    /// for their first card.
    pub turn_queue: Vec<PlayerId>,
    /// This turn's phases and the drainer's place in them — CR 500.1's
    /// sequence, spliced by CR 500.8. See [`TurnPlan`].
    pub turn_plan: TurnPlan,
    /// The player the **natural** rotation has reached — CR 500.7's extra turns
    /// do not advance it.
    ///
    /// That is the whole of why it exists rather than being
    /// `(active_player + 1) % n`: an extra turn is inserted *after* a turn, so
    /// the rotation resumes from the player whose natural turn it was. With
    /// four players, P1 taking an extra turn during P0's turn is followed by
    /// P1's own natural turn, which the arithmetic on `active_player` skips.
    ///
    /// Advanced when a natural turn is **proposed**, not when one begins:
    /// CR 614.10a says the sequence proceeds past a skipped turn, so the turn
    /// after a skipped P2 is P3's and not P2's again.
    ///
    /// Starts at CR 103.7's starting player, because [`Self::new`] starts with
    /// that player's first turn already in progress — and a fixture that
    /// hand-writes `active_player` writes this too, for the same reason.
    pub turn_rotation: PlayerId,

    // --- Combat tracking ---
    pub attacks_declared: bool,
    pub blockers_declared: bool,
    /// Damage division for blockers blocking 2+ attackers (rule 510.1d).
    /// Maps blocker ObjectId → Vec<(attacker ObjectId, damage amount)>.
    /// Populated by `choose_blocker_damage_division` at declare blockers and
    /// read by nothing yet — `codebase-state.md`, "Before card breadth" item 6.
    pub blocker_damage_divisions: IdMap<ObjectId, Vec<(ObjectId, u64)>>,
    /// Tracks creatures that dealt damage during the first-strike combat damage step.
    /// Used to determine which creatures deal damage in the normal combat damage step:
    /// - First strikers: dealt first-strike damage, skip normal step.
    /// - Double strikers: dealt first-strike damage, deal again in normal step.
    /// - Normal creatures: skip first-strike step, deal in normal step.
    ///
    /// Cleared with other combat state in on_phase_end(Combat).
    pub dealt_first_strike_damage: IdSet<ObjectId>,

    // --- Timestamp counter for layer system (rule 613.7) ---
    /// Monotonically increasing counter. Each permanent that enters the
    /// battlefield gets the current value, then the counter increments.
    pub next_timestamp: Timestamp,

    /// The next `ObjectId`, stamped by `add_object` beside the timestamp —
    /// the one door into the store. Starts at one so that
    /// `ObjectId::UNASSIGNED` is never a stored object's id. Cloned with the
    /// state, so a fork mints where its parent left off.
    next_object_id: u64,

    // --- The game's end (CR 104) ---
    /// Per-player loss flags, written by the `GameAction::PlayerLoses`
    /// performer and by nothing else. CR 104.5 makes a player who has lost a
    /// player who has *left*, so this is also what the turn and priority
    /// rotations pass over (CR 800.4j/k); `player_left_the_game` takes their
    /// objects (CR 800.4a).
    pub player_lost: Vec<bool>,
    /// The outcome, once the game has one — see [`GameResult`]. Written by
    /// the `PlayerWins` performer and by [`Self::settle_game_result`], read by
    /// `Game::is_over` and by every loop that must stop when the game does.
    pub result: Option<GameResult>,
    /// CR 103.3's starting life total, which "your starting life total" on a
    /// card (Exquisite Archangel) reads and a two-player 20 would get wrong
    /// in Commander.
    pub starting_life: i64,

    // --- First-turn draw skip (rule 103.8a) ---
    /// If true, the first draw step is skipped (one-time flag for game setup).
    /// Every in-game "skip draw" is a CR 614.10 replacement instead.
    pub skip_first_draw: bool,

    // --- Continuous effects registry (CR 613) ---
    pub continuous_effects: ContinuousEffectRegistry,

    // --- Replacement effects registry (CR 614/615) ---
    /// Replacement and prevention effects created by *resolutions* — CR 614.3's
    /// "prevent all damage that would be dealt this turn", CR 615.7's shields,
    /// CR 701.19a's regeneration.
    ///
    /// **Only two of the five sources in `replacement-architecture.md` §3.3
    /// live here.** The other three — static abilities of permanents, static
    /// abilities functioning in other zones, and counters — are discovered by
    /// sweeping the battlefield at the instant an event is proposed, which is
    /// what makes Humility strip a replacement ability for free and what puts
    /// CR 614.4's "must exist before the event" question at the only moment it
    /// can honestly be asked.
    pub replacement_effects: ReplacementEffectRegistry,

    /// Battlefield objects that **printed** a static ability whose body is an
    /// `Effect::Replacement`.
    ///
    /// `engine::replacement::gather`'s fast path, and not an optimization:
    /// reading effective abilities is a full `compute_characteristics` walk, so
    /// an ungated sweep would run one per permanent per proposed action.
    ///
    /// **A set rather than a count, so it cannot drift**: insert at ETB, remove
    /// at `cleanup_zone_state`, both idempotent. It over-approximates in one
    /// direction only — CR 305.7 or Humility can take the printed ability away
    /// without touching the set, which costs a walk and never an answer. The
    /// gate's other legs are `RegistryScopeSummary::granted_replacement_zones`
    /// (Layer 6) and `copied_replacement_zones` (`copy-effects-architecture.md`
    /// §4.7); Layer 3 is the route still without one.
    ///
    /// **Engine-maintained.** `place_on_battlefield` inserts and
    /// `cleanup_zone_state` removes; a hand-written removal is a card that
    /// silently stops working.
    pub replacement_ability_sources: IdSet<ObjectId>,

    /// Objects **off the battlefield** that printed a static ability whose
    /// body is an `Effect::Replacement` and that **functions in the zone the
    /// object is in** (CR 113.6) — Darksteel Colossus in a library, Nexus of
    /// Fate on the stack.
    ///
    /// The gather's zone leg (`replacement-architecture.md` §3.3 source 2)
    /// sweeps this set instead of the zones themselves: four libraries are
    /// ~400 objects and a gather runs ~2,300 times a game, so the equivalent
    /// of the per-permanent gate for objects that are not permanents has to
    /// be a set that is empty on every board that plays no such card. A
    /// second set beside [`Self::replacement_ability_sources`] rather than
    /// one widened set, because the two are read by different sweeps —
    /// `battlefield_ids_ordered` probes the first by id, the zone leg
    /// iterates the second — and a single set would cost the zone leg a
    /// store probe per battlefield source on every gather to tell them apart.
    /// Disjoint by construction: the registration doors know the zone.
    ///
    /// The zone half of membership is the *printed* statement
    /// (`zone_function::functions_in` on printed types, `// PRE-LAYER ZONE:`);
    /// the sweep re-asks it of the effective list, which is the exact check.
    /// Over-approximates in the same one direction as the set above.
    ///
    /// **The value is the printed replacement defs**, kept so the leg can ask
    /// whether any of them could apply to the proposal *before* it reads the
    /// object's frame — which is nearly never, since a Colossus's clause
    /// watches one kind of event and a gather is proposed for every kind.
    /// Exact rather than a shortcut: an object's effective replacement defs
    /// are its printed ones or fewer, because the two other ways onto the
    /// effective list, a grant and a copy, are the gate's other two legs
    /// (`RegistryScopeSummary`) and a strip only removes. Kept here, at the
    /// one site that already reads printed abilities, so the leg never reads
    /// `card_data` itself (`CLAUDE.md`'s layer-system invariant).
    ///
    /// **Engine-maintained.** `register_static_effects` inserts from
    /// `arrive_in_zone` and `create_in_zone`; `cleanup_zone_state` removes on
    /// leaving any zone but the battlefield.
    pub zone_replacement_ability_sources: IdMap<ObjectId, Vec<ReplacementDef>>,

    /// Every CR 101.2 "can't" a resolution has created.
    ///
    /// Source 4 of `cant-effects-architecture.md` §3.4's five, and the only one
    /// that is a registry: static abilities are swept off *effective* ability
    /// lists and keywords are synthesized, for the reason `gather` gives.
    ///
    /// Read through `engine::restriction::is_prohibited` and nowhere else — one
    /// predicate is what makes CR 101.3's "if you can't" a caller rather than a
    /// parallel mechanism.
    pub restrictions: RestrictionRegistry,

    /// Objects that entered the battlefield printing a static ability whose
    /// effect is an `Effect::Restriction` — `is_prohibited`'s fast-path gate.
    ///
    /// The twin of [`Self::replacement_ability_sources`], and a **different
    /// set**: an object can have a restriction ability without having a
    /// replacement one. Its doc's rule applies unchanged — add a new source of
    /// static restriction abilities and it must add a leg to the gate, or the
    /// source is silently dead on every board the gate skips.
    ///
    /// **Engine-maintained. Read it; do not write it.** `place_on_battlefield`
    /// inserts and `cleanup_zone_state` removes.
    pub restriction_ability_sources: IdSet<ObjectId>,

    /// Objects that entered the battlefield printing a static ability whose
    /// body is an `Effect::CostModification`, through an "as long as"
    /// wrapper or not — `engine::cost_determination::cost_modifications_for`'s fast-path gate
    /// (`cost-architecture.md` §3.1).
    ///
    /// The third such set, and the two above's rule applies unchanged: a new
    /// source of static cost abilities must add a leg to the gate, or the
    /// source is silently dead on every board the gate skips. It is the set
    /// the gather *sweeps* — sorted by timestamp, not the whole battlefield —
    /// which is why it holds sources rather than a count.
    ///
    /// **Engine-maintained. Read it; do not write it.** `place_on_battlefield`
    /// inserts and `cleanup_zone_state` removes.
    pub cost_modification_ability_sources: IdSet<ObjectId>,

    /// CR 614.13a/b — the two sets an auxiliary zone change is chosen against,
    /// scoped to the batch whose entries are being decided.
    ///
    /// **On `GameState` because it is outcome-bearing** (`codebase-state.md`
    /// item 40). Both sets are read across the CR 616.1 prompt and item 40's
    /// test is "drop it and re-derive": drop `chosen` and Thunder-Thrash Elder
    /// sacrifices one Runeclaw Bear to devour 3 *and* to devour 5, which is
    /// CR 614.13b's own example of the wrong answer. A fork at the prompt has
    /// to see both, so neither may live on the pipeline's stack.
    ///
    /// Saved and restored by `execute_batch_inner` the way `open_batch` and
    /// `close_batch` handle the event stamp — the exclusions belong to *these*
    /// simultaneous entries, and a nested batch (an auxiliary move's own, a
    /// rider's) is a different event with its own.
    pub(crate) entry_selection: EntrySelectionScope,

    /// CR 615.7's allocation answers for the batch being decided — per
    /// instance, how much of its "next N damage" each member is given.
    ///
    /// On `GameState` for the reason `entry_selection` is: the CR 616.1 loop
    /// decides one subject group at a time, an instance spanning several
    /// subjects is asked once, and the groups decided after read the answer
    /// across their own prompts (`codebase-state.md` item 40). Saved and
    /// restored by `execute_batch_inner` with `entry_selection`.
    pub(crate) prevention_allocations: PreventionAllocationScope,

    /// How deep inside itself the engine is right now, on three axes
    /// ([`NestingGuards`]).
    pub(crate) nesting: NestingGuards,

    /// The applied set a rider's proposals start from, for the extent of
    /// `resolve_rider` — CR 614.5's "any modified events that may replace that
    /// event", read by `execute_actions` and cleared for the extent of the
    /// batch it seeds, so the rider's *own* nested batches (an entry inside a
    /// creation it makes) start fresh as every contained event does.
    ///
    /// On `GameState` rather than on `ResolutionContext` because it is scoped
    /// to a call, like `entry_selection`, and a rider is resolved through the
    /// same `resolve_effect` every spell is (`codebase-state.md` item 40).
    pub(crate) rider_lineage: Option<HashSet<crate::engine::replacement::ReplacementInstanceId>>,

    /// The next tick to stamp onto a moving object's
    /// [`zone_change_epoch`](crate::objects::object::GameObject::zone_change_epoch).
    ///
    /// Starts at 1 so that a pregame object's `0` is strictly earlier than any
    /// move. Allocated by `move_object`, which is the engine's one performer of
    /// zone changes.
    pub(crate) next_zone_change_epoch: ZoneChangeEpoch,

    /// The tick as of the **start of the previous** state-based-action check.
    ///
    /// CR 704.6d's window is "since the last time state-based actions were
    /// checked", and the boundary has to be read at the *start* of a check
    /// rather than at its end. A commander that CR 704.5g puts into a graveyard
    /// does so during a check, so an end-of-check boundary would place the move
    /// before the boundary it is supposed to be after, and the commander would
    /// never be offered its command zone at all.
    pub(crate) last_sba_check_epoch: ZoneChangeEpoch,

    // --- Triggered abilities (CR 603) ---
    /// Abilities that have triggered and not yet been put onto the stack
    /// (CR 117.2a). Written by the dispatcher at a batch's close, drained by
    /// `place_pending_triggers` inside `perform_sba_and_triggers`. On the
    /// state and nowhere else (item 40): the drain removes each entry as it
    /// places it, so a clone taken at the ordering prompt resumes by running
    /// the drain again.
    pub pending_triggers: Vec<crate::types::triggers::PendingTrigger>,
    pub(crate) next_trigger_seq: u64,
    /// Permanents that *printed* a triggered ability, each against the
    /// record kinds its printed defs read — the dispatcher's fast-path gate,
    /// `replacement_ability_sources`' twin: written by
    /// `register_static_effects` from `place_on_battlefield`, removed by
    /// `cleanup_zone_state`, over-approximating in one direction only. A
    /// triggered ability an object has without having printed one reaches
    /// the sweep through `RegistryScopeSummary::unattributed_trigger_zones`.
    ///
    /// **The mask is `triggers-architecture.md` §11's lever**, and it is a
    /// mask of the *printed* defs for the same reason the set was of printed
    /// abilities: a granted or copied trigger is the summary's leg, which
    /// keeps its whole-list walk. A window whose kinds no source's mask
    /// meets is a dispatch that returns before the battlefield list is
    /// built.
    pub trigger_sources: IdMap<ObjectId, EventKindMask>,
    /// Objects off the battlefield with a printed triggered ability that
    /// functions where they are (CR 113.6k, derived by
    /// `zone_function::functioning_zones`) — Guile's "from anywhere" in a
    /// graveyard. Kept by the same doors as `zone_replacement_ability_sources`.
    pub zone_trigger_sources: IdMap<ObjectId, Vec<AbilityId>>,
    /// The frames CR 603.10 looks back to for a surviving source, taken by
    /// each batch of the open windows that departs an ability list's source
    /// (item 167), and taken by the window's dispatch.
    pub(crate) look_back_snapshots: Vec<crate::engine::triggers::LookBackSnapshot>,

    // --- Event log ---
    pub events: EventLog,

    /// The trace sink's handle, if a sink is attached — see
    /// [`crate::state::trace`]. `None` in every game nobody traces, which
    /// is what every emit point checks and all it pays. A pointer and a
    /// branch number, never a buffer: the derived `Clone` forks it.
    pub(crate) trace: Option<crate::state::trace::TraceHandle>,

    // --- Randomness ---
    /// The game's one source of randomness: shuffles now, coin flips and
    /// "at random" choices later (CR 705).
    ///
    /// Owned by the state rather than taken from `rand::rng()` at the point of
    /// use, so that a caller who wants a replayable game gets one — see
    /// [`GameState::reseed`]. Seeded to `DEFAULT_RNG_SEED` at construction, so
    /// a game nobody reseeds is still the *same* game every run; the
    /// interactive binary reseeds from entropy to get variety.
    pub rng: StdRng,
}

/// Turn phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseType {
    Beginning,
    Precombat,
    Combat,
    Postcombat,
    Ending,
}

/// Steps within phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepType {
    // Beginning phase
    Untap,
    Upkeep,
    Draw,
    // Combat phase
    BeginCombat,
    DeclareAttackers,
    DeclareBlockers,
    FirstStrikeDamage,
    CombatDamage,
    EndCombat,
    // Ending phase
    End,
    Cleanup,
}

/// Current phase and optional step
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phase {
    pub phase_type: PhaseType,
    pub step: Option<StepType>,
}

impl Phase {
    pub fn new(phase_type: PhaseType) -> Self {
        let step = initial_step(phase_type);
        Phase { phase_type, step }
    }
}

/// The first step of a phase, or `None` for a main phase, which has none.
///
/// `pub(crate)` for `engine::turns`: the drainer proposes a phase and its first
/// step as two separate events (CR 614.10 replaces either), so it needs to ask
/// for the first step rather than read it off a `Phase` the performer built.
pub(crate) fn initial_step(phase_type: PhaseType) -> Option<StepType> {
    match phase_type {
        PhaseType::Beginning => Some(StepType::Untap),
        PhaseType::Precombat => None,
        PhaseType::Combat => Some(StepType::BeginCombat),
        PhaseType::Postcombat => None,
        PhaseType::Ending => Some(StepType::End),
    }
}

/// Get the next step within a phase, or None if we've reached the last step
pub fn next_step(phase_type: PhaseType, current_step: StepType) -> Option<StepType> {
    match (phase_type, current_step) {
        // Beginning phase steps
        (PhaseType::Beginning, StepType::Untap) => Some(StepType::Upkeep),
        (PhaseType::Beginning, StepType::Upkeep) => Some(StepType::Draw),
        (PhaseType::Beginning, StepType::Draw) => None,

        // Combat phase steps
        (PhaseType::Combat, StepType::BeginCombat) => Some(StepType::DeclareAttackers),
        (PhaseType::Combat, StepType::DeclareAttackers) => Some(StepType::DeclareBlockers),
        (PhaseType::Combat, StepType::DeclareBlockers) => Some(StepType::FirstStrikeDamage),
        (PhaseType::Combat, StepType::FirstStrikeDamage) => Some(StepType::CombatDamage),
        (PhaseType::Combat, StepType::CombatDamage) => Some(StepType::EndCombat),
        (PhaseType::Combat, StepType::EndCombat) => None,

        // Ending phase steps
        (PhaseType::Ending, StepType::End) => Some(StepType::Cleanup),
        (PhaseType::Ending, StepType::Cleanup) => None,

        _ => None,
    }
}

/// One phase of a turn, as the turn's plan holds it.
///
/// A phase and not its steps, and that is a decision rather than an omission.
/// CR 500.10's *"any other steps that phase would normally have are skipped"*
/// is an `Option<Vec<StepType>>` overriding the natural list, and **this struct
/// is where it goes**; its only producer is a triggered ability (Obeka,
/// Splitter of Seconds), so it cannot be built before critical-path item 6.
/// Until then a plan that carried steps would carry a copy of
/// [`initial_step`]/[`next_step`] that nothing could make differ — a second
/// spelling of the chain `next_phase`'s deletion just removed the first
/// spelling of. → `replacement-architecture.md` §9 RE-10 decisions 2 and 4,
/// `backlog.md` §2.17.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlannedPhase {
    pub phase_type: PhaseType,
}

/// The phases this turn will have, in order, and how far the drainer has read.
///
/// **CR 500.1's sequence as data rather than as a `match`**, which CR 500.8
/// requires: a turn can hold two combat phases, so "what follows" is
/// unanswerable from a phase *type*. The cursor is an index, so two entries
/// of the same type are two positions.
///
/// **Rebuilt per turn, by `on_turn_begin`**, so a skipped turn builds no plan
/// and splices nothing — CR 614.10a's "anything scheduled for a skipped turn
/// won't happen". Seeded by [`GameState::new`] because that constructor
/// already describes turn 1.
///
/// **The cursor is schedule, not board, and the drainer owns it.** No CR 614
/// replacement effect and no CR 603 trigger can see it, and it is spent
/// whether or not the phase begins, so it is maintained where the proposal
/// is built rather than in `GameAction::BeginPhase`'s performer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnPlan {
    /// CR 500.1's five phases, plus whatever CR 500.8 spliced in.
    pub phases: Vec<PlannedPhase>,
    /// The **last unit considered**, which is not the last unit that happened
    /// — that difference is the whole of CR 500.11. `None` for the instant
    /// after a turn begins and before any phase of it has been proposed.
    pub cursor: Option<usize>,
}

impl TurnPlan {
    /// CR 500.1's phases, in order.
    const NATURAL: [PhaseType; 5] = [
        PhaseType::Beginning,
        PhaseType::Precombat,
        PhaseType::Combat,
        PhaseType::Postcombat,
        PhaseType::Ending,
    ];

    /// CR 500.1's five phases, in order, with nothing spliced and nothing read.
    ///
    /// **`NATURAL` above is the static; this is not and cannot be.** A plan is
    /// per-game mutable state — CR 500.8 splices into it, so each game owns a
    /// `Vec` of its own — and the constant is the seed every one of them starts
    /// from.
    pub fn natural() -> Self {
        TurnPlan {
            phases: Self::NATURAL.map(|phase_type| PlannedPhase { phase_type }).to_vec(),
            cursor: None,
        }
    }

    /// Make this CR 500.1's five phases again, **keeping the allocation**.
    ///
    /// [`Self::natural`] with the `Vec` reused, and the reuse is the whole
    /// reason it is a separate method: this runs once per turn that begins, and
    /// assigning a fresh plan there would allocate on every turn of every game
    /// where the chain it replaced allocated nothing.
    pub fn reset(&mut self) {
        self.phases.clear();
        self.phases
            .extend(Self::NATURAL.map(|phase_type| PlannedPhase { phase_type }));
        self.cursor = None;
    }

    /// The phase at `index`, or `None` past the end of the turn.
    pub fn phase_at(&self, index: usize) -> Option<PhaseType> {
        self.phases.get(index).map(|planned| planned.phase_type)
    }

    /// The first entry of `phase_type`, for [`GameState::set_turn_position`].
    fn first_index_of(&self, phase_type: PhaseType) -> Option<usize> {
        self.phases.iter().position(|p| p.phase_type == phase_type)
    }
}

impl GameState {
    /// Create a new game with the given number of players
    pub fn new(num_players: usize, starting_life: i64) -> Self {
        let players: Vec<PlayerState> = (0..num_players)
            .map(|id| PlayerState::new(id, starting_life))
            .collect();

        GameState {
            objects: IdMap::default(),
            players,
            stack: Vec::new(),
            stack_entries: IdMap::default(),
            resolving: None,
            battlefield: IdMap::default(),
            diagnostics: Default::default(),
            layer_epoch: 0,
            layer_memo: LayerMemo::default(),
            exile: Vec::new(),
            command: Vec::new(),
            turn_number: 1,
            // Turn 1 has begun for the starting player; nobody else has had one.
            last_turn_began: {
                let mut v = vec![0; num_players];
                v[0] = 1;
                v
            },
            active_player: 0,
            priority_player: 0,
            phase: Phase::new(PhaseType::Beginning),
            turn_queue: Vec::new(),
            // Turn 1's plan, beside the turn 1 the rest of this constructor
            // describes. Its cursor is the beginning phase `phase` above names,
            // so a bare `GameState` a fixture never drains is already
            // self-consistent.
            turn_plan: TurnPlan {
                cursor: Some(0),
                ..TurnPlan::natural()
            },
            turn_rotation: 0,
            attacks_declared: false,
            blockers_declared: false,
            blocker_damage_divisions: IdMap::default(),
            dealt_first_strike_damage: IdSet::default(),
            next_timestamp: 0,
            next_object_id: 1,
            player_lost: vec![false; num_players],
            result: None,
            starting_life,
            skip_first_draw: false,
            continuous_effects: ContinuousEffectRegistry::new(),
            replacement_effects: ReplacementEffectRegistry::new(),
            replacement_ability_sources: IdSet::default(),
            zone_replacement_ability_sources: IdMap::default(),
            restrictions: RestrictionRegistry::new(),
            restriction_ability_sources: IdSet::default(),
            cost_modification_ability_sources: IdSet::default(),
            entry_selection: EntrySelectionScope::default(),
            nesting: NestingGuards::default(),
            rider_lineage: None,
            prevention_allocations: PreventionAllocationScope::default(),
            next_zone_change_epoch: 1,
            last_sba_check_epoch: 1,
            pending_triggers: Vec::new(),
            next_trigger_seq: 0,
            trigger_sources: IdMap::default(),
            zone_trigger_sources: IdMap::default(),
            look_back_snapshots: Vec::new(),
            events: EventLog::new(),
            trace: None,
            rng: StdRng::seed_from_u64(Self::DEFAULT_RNG_SEED),
        }
    }

    /// The seed a `GameState` starts with when nobody supplies one.
    ///
    /// A fixed value, not entropy: an unseeded game that is nevertheless
    /// reproducible is the safer default, and it makes every test that shuffles
    /// deterministic without opting in.
    pub const DEFAULT_RNG_SEED: u64 = 0x4D54_4749_4348_4F52; // "MTGICHOR"

    /// Put the game at `phase_type`/`step` by hand — **the one seam a fixture
    /// may move the position through.**
    ///
    /// The position is two facts, `phase` and `turn_plan.cursor`, and writing
    /// `phase` alone leaves the drainer to advance from wherever the cursor was;
    /// this makes the pair unwriteable apart. `advance_turn` debug-asserts that
    /// the two agree, which catches a fixture that goes around it.
    ///
    /// **The first entry of that type**: a plan with two combat phases is one
    /// CR 500.8 built, and a board that wants the second gets it by resolving
    /// the card that made it.
    pub fn set_turn_position(&mut self, phase: Phase) {
        self.turn_plan.cursor = self.turn_plan.first_index_of(phase.phase_type);
        debug_assert!(
            self.turn_plan.cursor.is_some(),
            "no {:?} in this turn's plan to set the position to",
            phase.phase_type
        );
        self.phase = phase;
    }

    /// Begin turn `turn` with `player` as the active player, recording the turn
    /// start that CR 302.6 measures against.
    ///
    /// The one writer of `last_turn_began`. A caller that assigns `turn_number`
    /// on its own leaves the clock stale and every summoning-sickness question
    /// answers against the wrong turn.
    pub fn begin_turn(&mut self, turn: u32, player: PlayerId) {
        self.turn_number = turn;
        self.active_player = player;
        self.last_turn_began[player] = turn;
    }

    /// Where `player` sits in **APNAP order**: 0 for the active player, then
    /// each other player in turn order.
    ///
    /// > 101.4. If multiple players would make choices and/or take actions at
    /// > the same time, the active player (referred to as AP) makes any choices
    /// > required, then the next player in turn order (referred to as NAP) does
    /// > the same, and so on.
    ///
    /// **A sort key rather than an iterator**, because every caller already has
    /// its own collection to order and they share only the question "who goes
    /// first" — a batch's CR 616.1 choices, CR 704.6d's command-zone offers,
    /// CR 704.5j's legend groups. Sort *stably* on it and each caller keeps its
    /// own order within one player, which is what leaves a sweep built from
    /// `battlefield_ids_ordered` in board order among that player's permanents.
    pub fn apnap_index(&self, player: PlayerId) -> usize {
        let n = self.players.len();
        (player + n - self.active_player) % n
    }

    /// Is `player` still in the game? CR 104.5: a player who has lost has left.
    pub fn in_game(&self, player: PlayerId) -> bool {
        !self.player_lost[player]
    }

    /// CR 800.1 — "a multiplayer game is a game that begins with more than two
    /// players", which is the scope of every rule in CR 800.
    ///
    /// **The gate on CR 800.4, and it is the rule's own.** A two-player game
    /// ends the moment a player leaves (CR 104.2a), so the section that says
    /// what becomes of their objects has nothing to be about there — 800.4's
    /// own first sentence is "unlike two-player games, multiplayer games can
    /// continue after one or more players have left the game". Read off the
    /// seat count the game *began* with, which is what 800.1 says and what
    /// `players` still holds after any number of departures: a four-player
    /// game down to two is a multiplayer game.
    pub fn is_multiplayer(&self) -> bool {
        self.players.len() > 2
    }

    /// The next player in turn order after `after` who is still in the game,
    /// or `None` when nobody is.
    ///
    /// CR 800.4j's "the next player in turn order" and the priority loop's
    /// rotation. `next_turn_taker` is the turn's own reading of the same rule
    /// and consumes what it reads; this one is a pure lookup.
    pub fn next_player_in_game(&self, after: PlayerId) -> Option<PlayerId> {
        let n = self.players.len();
        (1..=n).map(|k| (after + k) % n).find(|&p| self.in_game(p))
    }

    /// CR 104.2a and 104.4a, asked of the batch that performed one or more
    /// losses once every member has performed.
    ///
    /// A recorded result is never overwritten: CR 104.1 ended the game at the
    /// first one, and a loss performed after it — Stunning Reversal's survivor
    /// drawing seven from a short library on the next check — is the game
    /// continuing to be over rather than a second result.
    pub(crate) fn settle_game_result(&mut self) {
        if self.result.is_some() {
            return;
        }
        let survivors: Vec<PlayerId> =
            (0..self.players.len()).filter(|&p| self.in_game(p)).collect();
        self.result = match survivors.as_slice() {
            // 104.4a — "all the players remaining in a game lose
            // simultaneously".
            [] => Some(GameResult::Draw),
            // 104.2a — "that player's opponents have all left the game".
            [winner] => Some(GameResult::Winner(*winner)),
            _ => None,
        };
    }

    /// The turn on which `player`'s most recent turn began, or `None` if they
    /// have not had one yet.
    pub fn most_recent_turn_began(&self, player: PlayerId) -> Option<u32> {
        match self.last_turn_began.get(player) {
            Some(0) | None => None,
            Some(turn) => Some(*turn),
        }
    }

    /// Point the game's randomness at `seed`. Call before `Game::setup` —
    /// after it, the opening hands have already been dealt.
    pub fn reseed(&mut self, seed: u64) {
        self.rng = StdRng::seed_from_u64(seed);
    }

    /// Point the game's randomness at the OS — for interactive play, where a
    /// fresh shuffle each time is the point.
    pub fn reseed_from_entropy(&mut self) {
        self.rng = StdRng::from_os_rng();
    }

    /// Shuffle a player's library with the game's RNG (CR 701.20).
    ///
    /// Lives here rather than on `Game` because it needs `rng` and `players`
    /// borrowed at once, and because in-game shuffle effects will want it.
    pub fn shuffle_library(&mut self, player: PlayerId) {
        use rand::seq::SliceRandom;
        let Self { players, rng, .. } = self;
        if let Some(p) = players.get_mut(player) {
            p.library.shuffle(rng);
        }
    }

    /// CR 701.9b's "at random" — `count` distinct cards out of `from`, drawn
    /// from the game's own RNG.
    ///
    /// Returns them **in `from`'s order**, which for Hymn to Tourach is hand
    /// order: the rule chooses *which* cards, and nothing in CR 701.9 gives
    /// the order they reach the graveyard in, so the two choosers agreeing is
    /// worth more than an order no rule names.
    ///
    /// Beside `shuffle_library` for its reason — it needs `rng` and `players`
    /// borrowed at once — and drawn from `rng` and never `rand::rng()`, which
    /// is what makes a seeded game replay.
    pub(crate) fn random_cards_from(
        &mut self,
        from: &[ObjectId],
        count: usize,
    ) -> Vec<ObjectId> {
        use rand::seq::SliceRandom;
        // CR 101.3 — as much as it can, which for a short hand is all of it.
        let count = count.min(from.len());
        if count == 0 {
            return Vec::new();
        }
        let mut positions: Vec<usize> = (0..from.len()).collect();
        positions.shuffle(&mut self.rng);
        positions.truncate(count);
        positions.sort();
        positions.into_iter().map(|i| from[i]).collect()
    }

    // --- Deterministic iteration ---

    /// The battlefield, oldest permanent first.
    ///
    /// **Every sweep whose order can be observed goes through this, not
    /// `battlefield.iter()`.** `battlefield` is a `HashMap`, and its hasher
    /// (`types::ids::IdHash`) is seeded per *process*, so a direct iteration
    /// hands the legal action list, the mana sources and the SBA sweeps to
    /// the caller in a different order on every run — which is how
    /// `fuzz_games --seed N` came to be irreproducible. Sorting by `ObjectId`
    /// is not a fix either: an id is a counter now, but it is not an order
    /// any rule names.
    ///
    /// The CR 613.7 timestamp is the deterministic key — read off the entry,
    /// which carries a copy of the object's for exactly this sweep's sake
    /// (`PermanentState::timestamp`). Every value of it comes from `next_timestamp`, one monotonic
    /// counter, so it is unique across the battlefield and totally orders it,
    /// and CR 613.7e's reassignment on attach keeps both properties: a
    /// reattached permanent moves to the end of the order, in every run
    /// alike. The sweep order is therefore the CR's own timestamp order, and
    /// nothing reads it as entry order.
    ///
    /// Order-irrelevant sweeps — "untap every permanent", "clear all damage" —
    /// may still iterate the map directly; they touch disjoint entries and emit
    /// nothing.
    pub fn battlefield_ordered(&self) -> Vec<(ObjectId, &PermanentState)> {
        let mut entries: Vec<(ObjectId, &PermanentState)> =
            self.battlefield.iter().map(|(&id, e)| (id, e)).collect();
        entries.sort_by_key(|(_, e)| e.timestamp);
        entries
    }

    /// The battlefield's object ids, oldest permanent first.
    /// See [`GameState::battlefield_ordered`] for why this exists.
    pub fn battlefield_ids_ordered(&self) -> Vec<ObjectId> {
        // Collect (timestamp, id) and sort *that*: `sort_by_key` calls its closure
        // O(n log n) times, so keying on a HashMap lookup paid ~20× (measured
        // 2026-08-25), and this runs eight times per SBA sweep. Stable, keyed on
        // timestamp alone — tiebreaking on a v4 `ObjectId` would be the exact
        // non-determinism the ordered sweeps exist to avoid (`CLAUDE.md`).
        let mut pairs: Vec<(Timestamp, ObjectId)> = self.battlefield
            .iter()
            .map(|(&id, e)| (e.timestamp, id))
            .collect();
        pairs.sort_by_key(|&(ts, _)| ts);
        pairs.into_iter().map(|(_, id)| id).collect()
    }

    /// Every object in `zone`, in that zone's own order — seat order first,
    /// then the player's own list, for the four zones a player owns.
    ///
    /// **The walk order for a member outside the battlefield** (LJ,
    /// `layers-architecture.md` §13c decision 4). CR 613.7 orders *effects* by
    /// timestamp and says nothing about the objects they apply to, so a member
    /// here needs a deterministic position and not a timestamp — which is why
    /// this phase needs none of CR 613.7d's object timestamps. Every container
    /// below is already a `Vec`, so the order is the zone's own and no sort is
    /// involved; nothing reaches a `HashMap`, which is what
    /// `CLAUDE.md`'s determinism invariant asks. For a graveyard the engine's
    /// order is the rule's: CR 404.3 makes it an ordered zone.
    ///
    /// `Zone::Battlefield` answers `battlefield_ids_ordered` — CR 613.7
    /// timestamp order — so a caller sweeping a `ZoneSet` that happens to
    /// include it still gets the one order counts agree with.
    pub fn zone_ids_ordered(&self, zone: Zone) -> Vec<ObjectId> {
        match zone {
            Zone::Battlefield => self.battlefield_ids_ordered(),
            Zone::Stack => self.stack.clone(),
            Zone::Exile => self.exile.clone(),
            Zone::Command => self.command.clone(),
            Zone::Library | Zone::Hand | Zone::Graveyard => {
                let mut ids = Vec::new();
                for player in self.players.iter() {
                    ids.extend(match zone {
                        Zone::Library => player.library.iter().copied(),
                        Zone::Hand => player.hand.iter().copied(),
                        _ => player.graveyard.iter().copied(),
                    });
                }
                ids
            }
        }
    }

    /// Allocate and return the next timestamp value.
    pub fn allocate_timestamp(&mut self) -> Timestamp {
        let ts = self.next_timestamp;
        self.next_timestamp += 1;
        ts
    }

    /// Put `entry` on the battlefield for `id`, stamping it from the object.
    ///
    /// **The one door, and that is what keeps the order key honest.**
    /// `PermanentState::timestamp` is a copy of `GameObject::timestamp` kept
    /// for the ordered sweeps (see that field), and an entry inserted without
    /// it would carry `0` — which is not merely wrong but *tied*, and a tie in
    /// `battlefield_ids_ordered` is resolved by `HashMap` order. That is the
    /// exact non-determinism the ordered sweeps exist to prevent
    /// (`CLAUDE.md`), so it is prevented here rather than asserted about.
    ///
    /// Callers are `place_on_battlefield` and the test helpers that build a
    /// board without ETB hooks.
    pub(crate) fn insert_battlefield_entity(&mut self, id: ObjectId, mut entry: PermanentState) {
        entry.timestamp = self.object_timestamp(id);
        // Spelled through a local so the sweep that routed every other
        // `battlefield.insert` here did not route this one into itself.
        let battlefield = &mut self.battlefield;
        battlefield.insert(id, entry);
    }

    /// Write a CR 613.7 timestamp to `id`: on the object, and on its
    /// battlefield entry if it has one.
    ///
    /// **The one writer of the pair**, which is what makes
    /// `PermanentState::timestamp` a copy that cannot drift rather than a
    /// second source of truth — see that field for why the copy exists at
    /// all. Its callers are CR 613.7d (`arrive_in_zone`, where there is no
    /// entry yet) and CR 613.7e (`attach`, where there is).
    pub(crate) fn set_object_timestamp(&mut self, id: ObjectId, timestamp: Timestamp) {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.timestamp = timestamp;
        }
        if let Some(entry) = self.battlefield.get_mut(&id) {
            entry.timestamp = timestamp;
        }
    }

    /// `id`'s CR 613.7d timestamp, or `u64::MAX` for an object the store has
    /// never heard of.
    ///
    /// The fallback is unreachable for every caller — an ordered sweep reads
    /// ids off a collection the store also holds — and is `MAX` rather than 0
    /// so that a hypothetical unknown sorts last instead of silently claiming
    /// to be the oldest permanent on the battlefield.
    pub fn object_timestamp(&self, id: ObjectId) -> Timestamp {
        self.objects.get(&id).map(|obj| obj.timestamp).unwrap_or(u64::MAX)
    }

    // --- Layer memo epoch (layers-architecture.md §12 "7a") ---

    /// The epoch the layer memo keys on: a number that changes at every write
    /// to a layer-walk input, and only then.
    ///
    /// Two monotone parts. `layer_epoch` is bumped by the writers listed on
    /// [`GameState::bump_layer_epoch`]; `ContinuousEffectRegistry::mutations`
    /// is bumped by the registry's own funnel, which is the one route to its
    /// rows and cannot see this struct. Both only ever grow, so their sum
    /// strictly increases at every write and two reads are equal exactly when
    /// nothing was written between them.
    pub fn layer_epoch(&self) -> u64 {
        self.layer_epoch + self.continuous_effects.mutations()
    }

    /// A layer-walk input was written; every memoized frame is now stale.
    ///
    /// **After the write, never before it.** A performer may query between
    /// the two — `place_on_battlefield` asks for the effective controller after
    /// inserting the entity — and a bump on entry would hand that query a frame
    /// from before the write.
    ///
    /// The inputs, and the one writer of each: `battlefield`, `objects[id].zone`
    /// and the players' graveyards in `move_object`; the entity in
    /// `place_on_battlefield`; its counters in `add_counters` /
    /// `remove_counters`; its `attached_to` in `attach` / `detach`
    /// (`ObjectSet::Host` reads it at every layer); the
    /// `objects` map in `add_object` / `remove_object`; `stack_entries` in
    /// `set_stack_entry` / `take_stack_entry`; `resolving` in
    /// `resolve_top_of_stack`; the registry's rows through its own
    /// `mutating`, including the re-stamp `attach` asks of it (CR 613.7e) —
    /// the entity's own `timestamp` is not a walk input, only registration
    /// reads it. Status the walk never reads — `tapped`, damage, combat,
    /// the mana pool — has no bump, and must not get one: every bump costs
    /// one walk per queried object. `&mut self` on purpose, so a read path
    /// cannot call this.
    pub fn bump_layer_epoch(&mut self) {
        self.layer_epoch += 1;
    }

    // --- Battlefield convenience ---

    /// **The performer for `GameAction::EnterBattlefield`** (CR 614.1c/d).
    ///
    /// Creates a `PermanentState` for the object, applies the `mods` the
    /// CR 616.1 loop settled on, and announces the arrival. Returns a mutable
    /// reference to the inserted entry so callers can tweak fields without a
    /// second lookup.
    ///
    /// **Not the entry point.** Engine code proposes an entry with
    /// [`GameState::propose_entry`], which routes through `execute_actions` so
    /// that a CR 614.1c replacement sees it. This runs after the pipeline has
    /// decided what the permanent enters *as*, and it is the only emitter of
    /// `GameEvent::PermanentEnteredBattlefield` — the `mods` and the resulting
    /// controller are both known here and nowhere else.
    ///
    /// **The order inside the entry is the CR's.** Status and counters
    /// (CR 110.5b, 122.6a) are part of *arriving*, so they are written before
    /// anything can observe the permanent: a creature that enters with +1/+1
    /// counters is never momentarily a 0/0 for CR 704.5f to kill.
    /// `register_static_effects` comes last because CR 613.7a timestamps what
    /// it registers off the entry that now exists.
    pub fn place_on_battlefield(
        &mut self,
        id: ObjectId,
        controller: PlayerId,
        mods: &EnterMods,
    ) -> &mut PermanentState {
        // No timestamp allocated here: CR 613.7d stamped it as the object
        // entered the battlefield *zone* (`move_object`), or as it was created
        // there (`add_object`, a token). This performer runs after both.
        let current_turn = self.turn_number;
        let mut entry = PermanentState::new(id, controller, current_turn);
        // CR 110.5b — the one status a permanent can currently enter with.
        entry.tapped = mods.tapped;
        // CR 400.7d — who cast it and from where, off the resolving spell's
        // entry; a permanent that arrives any other way was not cast.
        entry.cast = match self.resolving {
            Some(r) if r.id == id => r
                .cast_from
                .map(|from| crate::state::battlefield::CastFacts { by: r.default_controller, from }),
            _ => None,
        };
        self.insert_battlefield_entity(id, entry);
        self.bump_layer_epoch();

        // CR 122.6a. Deliberately not a nested `AddCounters` proposal: these
        // counters are part of the entry event rather than a separate "counters
        // would be put on it" one, which is why CR 614.16's doublers replace the
        // `EnterMods` this performer is handed rather than an event of their own.
        for row in &mods.counters {
            self.add_counters(id, row.counter, row.n);
        }

        self.register_static_effects(id, controller, Zone::Battlefield);

        // The controller the *game* sees, not the CR 110.2b default it entered
        // under: a stolen permanent spell enters under its caster's control and
        // CR 400.7a's Layer 2 row has already moved it to the thief by the time
        // anything can look.
        let effective = crate::oracle::characteristics::get_effective_controller(self, id)
            .unwrap_or(controller);
        self.emit_event(crate::events::event::GameEvent::PermanentEnteredBattlefield {
            object_id: id,
            controller: effective,
        });

        self.battlefield.get_mut(&id).unwrap()
    }

    /// CR 110.2b's **default** controller for an object that is entering the
    /// battlefield — the value `GameAction::EnterBattlefield` carries.
    ///
    /// > 110.2b. A permanent's controller is, by default, the player under
    /// > whose control it entered the battlefield.
    ///
    /// The owner for a land drop and for a token; the player who put the spell
    /// onto the stack for a resolving permanent spell. This is the **default** —
    /// the value Layer 2 modifies, not the answer `get_effective_controller`
    /// gives. A stolen permanent spell enters under its caster's control here
    /// and is moved to the thief by the Layer 2 row CR 400.7a keeps alive.
    ///
    /// **Reads [`GameState::resolving`].** The `StackEntry` that recorded
    /// CR 110.2b's answer has been taken by the time a permanent spell reaches
    /// the battlefield, so that field is how the answer survives the resolution.
    pub(crate) fn default_enter_controller(&self, id: ObjectId) -> Result<PlayerId, String> {
        Ok(match self.resolving {
            Some(r) if r.id == id => r.default_controller,
            _ => self.get_object(id)?.owner,
        })
    }

    /// What the *rules* say this object enters the battlefield with, before any
    /// replacement effect has been consulted.
    ///
    /// The seed for `GameAction::EnterBattlefield`'s `mods`, and the successor
    /// to `init_etb_counters`. Being part of the proposal rather than of the
    /// performer is the point: CR 614.16's counter doublers replace what a
    /// permanent enters *with*, and a loyalty count written straight into the
    /// entity would be invisible to them.
    ///
    /// **CR 306.5b is a rule, not an ability**, which is why it lives here
    /// rather than in a `ReplacementDef` — nothing on a planeswalker's card
    /// says it enters with loyalty counters, the same way nothing on a
    /// commander card says CR 903.9b.
    ///
    /// **Reads the CR 614.12 frame, not the printed card.** CR 306.5b gives
    /// the ability to "a planeswalker", so the question is whether the object
    /// is one *as it would exist on the battlefield* under `controller` —
    /// `layers::compute_as_entering` with no mods yet. The direction that
    /// observably differs from a printed read is a planeswalker card whose
    /// type a filter-scoped Layer 4 effect removes: it is not a planeswalker
    /// on the battlefield, has no such ability, and enters with no loyalty
    /// counters. (The other direction — a non-planeswalker made one by a
    /// Layer 4 effect — reads the *printed* loyalty, which is `None`, and so
    /// enters with none either way; CR 306.5b says "printed" and means it.)
    pub(crate) fn default_enter_mods(&self, id: ObjectId, controller: PlayerId) -> EnterMods {
        let mut mods = EnterMods::NONE;

        // CR 306.5b — "a planeswalker enters the battlefield with a number of
        // loyalty counters on it equal to its printed loyalty number".
        let loyalty = match self.objects.get(&id) {
            Some(obj) => obj.card_data.loyalty,
            None => return mods,
        };
        if let Some(loyalty) = loyalty
            && loyalty > 0 {
            let is_planeswalker = crate::engine::layers::compute_as_entering(
                self, id, controller, &EnterMods::NONE,
            )
            .is_some_and(|chars| {
                chars.types.contains(&crate::types::card_types::CardType::Planeswalker)
            });
            if is_planeswalker {
                mods.counters.push(crate::types::replacement::EntryCounters {
                    counter: CounterType::Loyalty,
                    n: loyalty as u32,
                    by: None,
                });
            }
        }

        mods
    }

    /// Put `n` counters of `counter_type` on a permanent, allocating the CR
    /// 613.7c timestamp.
    ///
    /// The normal entry point — `PermanentState::add_counters` needs a
    /// timestamp and cannot allocate one, since it has no access to the game.
    /// No-op if `id` is not on the battlefield, and no timestamp is burned in
    /// that case.
    pub fn add_counters(&mut self, id: ObjectId, counter_type: CounterType, n: u32) {
        if !self.battlefield.contains_key(&id) {
            return;
        }
        let timestamp = self.allocate_timestamp();
        if let Some(entry) = self.battlefield.get_mut(&id) {
            entry.add_counters(counter_type, n, timestamp);
        }
        self.bump_layer_epoch();
    }

    /// Remove up to `n` counters of `counter_type` from a permanent; the number
    /// actually removed.
    ///
    /// The counterpart of [`GameState::add_counters`] and, like it, the one
    /// route to the entity's method: a counter is a layer-walk input (CR
    /// 122.1a/b), so the write bumps the epoch — when it is a write. Removing
    /// nothing from a permanent carrying nothing writes nothing.
    pub fn remove_counters(&mut self, id: ObjectId, counter_type: CounterType, n: u32) -> u32 {
        let Some(entry) = self.battlefield.get_mut(&id) else {
            return 0;
        };
        let removed = entry.remove_counters(counter_type, n);
        if removed > 0 {
            self.bump_layer_epoch();
        }
        removed
    }

    /// Attach a permanent to a host (CR 301.5, 303.4): writes the attachment's
    /// `attached_to` and the host's `attached_by`, the two fields that together
    /// record one attachment.
    ///
    /// The one writer of those fields, for the reason `add_counters` is the one
    /// route to the entity's counters: `attached_to` is a layer-walk input —
    /// `ObjectSet::Host` reads it at every layer — so the write bumps the
    /// epoch, and a writer that bypassed this would leave the memo serving
    /// "enchanted creature gets +1/+2" to nobody until something else bumped.
    /// Writes nothing, and burns no bump, unless both permanents are on the
    /// battlefield: CR 303.4i sends an Aura whose host is gone elsewhere, and
    /// CR 301.5c never lets an Equipment point off the battlefield either.
    /// Nor when `attachment` is already on `host` — CR 701.3b, "the effect
    /// does nothing" — so a caller can tell a transition from a no-op by the
    /// return: `true` iff the link was written. The `GameAction::Attach`
    /// performer announces only the transition.
    pub fn attach(&mut self, attachment: ObjectId, host: ObjectId) -> bool {
        if !self.battlefield.contains_key(&attachment) || !self.battlefield.contains_key(&host) {
            return false;
        }
        if self.battlefield[&attachment].attached_to == Some(host) {
            return false;
        }
        // A reattachment leaves the old host's back-pointer behind otherwise.
        self.detach(attachment);
        // CR 613.7e — a new timestamp each time it becomes attached, from the one
        // counter (`battlefield_ordered`). CR 613.7a's third sentence then
        // re-stamps the rows the attachment's static abilities registered, through
        // the registry's own funnel; the bump below is `attached_to`'s.
        let timestamp = self.allocate_timestamp();
        self.battlefield.get_mut(&attachment).unwrap().attached_to = Some(host);
        self.set_object_timestamp(attachment, timestamp);
        self.continuous_effects.retime_static_rows(attachment, timestamp);
        self.battlefield.get_mut(&host).unwrap().attached_by.push(attachment);
        self.bump_layer_epoch();
        true
    }

    /// Detach a permanent from whatever it is attached to — clears its
    /// `attached_to` and removes it from the host's `attached_by` — and return
    /// the former host, if there was one.
    ///
    /// The counterpart of [`GameState::attach`], and the other writer of the
    /// same two fields. Not a state-based action by itself: the SBA that
    /// decides an Equipment is on a non-creature (CR 704.5n) calls this and
    /// then announces. No bump when there was nothing to detach — that is not
    /// a write.
    pub fn detach(&mut self, attachment: ObjectId) -> Option<ObjectId> {
        let host = self.battlefield.get_mut(&attachment)?.attached_to.take()?;
        if let Some(host_entry) = self.battlefield.get_mut(&host) {
            host_entry.attached_by.retain(|&id| id != attachment);
        }
        self.bump_layer_epoch();
        Some(host)
    }

    /// Remove a permanent from combat (CR 506.4).
    ///
    /// > 506.4. A permanent is removed from combat if it leaves the
    /// > battlefield, if its controller changes, if it phases out, or if an
    /// > effect specifically says it's removed from combat. ... A creature
    /// > that's removed from combat stops being an attacking, blocking,
    /// > blocked, and/or unblocked creature.
    ///
    /// Both directions, which is the part a one-line `entry.attacking = None`
    /// would get wrong: an attacker leaving combat also stops being *blocked
    /// by* its blockers, and a blocker leaving stops appearing in the
    /// attackers' `blocked_by` lists. CR 506.4b keeps the attacker blocked in
    /// the sense that matters for damage — "an attacking creature that's been
    /// blocked remains blocked even if all creatures blocking it are removed
    /// from combat" — so `is_blocked` is deliberately left alone.
    pub(crate) fn remove_from_combat(&mut self, id: ObjectId) {
        let Some(entry) = self.battlefield.get(&id) else {
            return;
        };
        let was_blocked_by = entry
            .attacking
            .as_ref()
            .map(|a| a.blocked_by.clone())
            .unwrap_or_default();
        let was_blocking = entry
            .blocking
            .as_ref()
            .map(|b| b.blocking.clone())
            .unwrap_or_default();

        if let Some(entry) = self.battlefield.get_mut(&id) {
            entry.attacking = None;
            entry.blocking = None;
        }
        // This creature was attacking: its blockers stop blocking it.
        for blocker in was_blocked_by {
            if let Some(b) = self.battlefield.get_mut(&blocker)
                && let Some(info) = b.blocking.as_mut() {
                info.blocking.retain(|&a| a != id);
            }
        }
        // This creature was blocking: the attackers stop being blocked by it.
        // CR 506.4b leaves them *blocked* — they simply have no blockers.
        for attacker in was_blocking {
            if let Some(a) = self.battlefield.get_mut(&attacker)
                && let Some(info) = a.attacking.as_mut() {
                info.blocked_by.retain(|&b| b != id);
            }
        }
    }

    /// The timestamp a continuous effect generated by `ability` on `id` gets
    /// (CR 613.7a).
    ///
    /// > A continuous effect generated by a static ability has the same
    /// > timestamp as the object the static ability is on, or the timestamp of
    /// > the effect that created the ability, whichever is later.
    ///
    /// Both clauses are implemented. `granted_at` is the timestamp of the
    /// effect that created the ability, and `None` means the ability was
    /// printed on the object — the overwhelmingly common case, and the only one
    /// `register_static_effects` ever sees, since it runs at ETB off printed
    /// text.
    ///
    /// Clause 2 is the `max()` below, which is the whole reason every
    /// static-effect timestamp routes through this function rather than reading
    /// `battlefield[id].timestamp` at each construction site. The CR's own
    /// example: Rune of Flight grants enchanted Equipment "Equipped creature has
    /// flying"; that granted ability's effect takes Rune of Flight's timestamp
    /// because it is later than the Equipment's, which is how it beats Colossus
    /// Hammer's "loses flying" instead of losing to it.
    ///
    /// `register_granted_static_effects` is the caller that passes `Some`. The
    /// metadata this clause wants is a timestamp, not an `AbilityOrigin` enum
    /// (`layers-architecture.md` §15.2 item 4).
    ///
    /// Clause 3 — "if the object the ability is on receives a new timestamp,
    /// each continuous effect generated by static abilities of that object
    /// receives a new timestamp as well, but the relative order of those
    /// timestamps remains the same" — needs nothing extra: every such effect
    /// reads the object's timestamp, and relative order among them is the
    /// `EffectId` tiebreak in `effects_in_layer`.
    pub(crate) fn static_effect_timestamp(
        &self,
        id: ObjectId,
        _ability: &crate::objects::card_data::AbilityDef,
        granted_at: Option<crate::engine::layers::types::Timestamp>,
    ) -> crate::engine::layers::types::Timestamp {
        match self.objects.get(&id) {
            // CR 613.7a — the object's timestamp or the granting effect's, whichever
            // is later; a printed ability has only the first. The *object*, not the
            // battlefield entry: CR 613.7d gives an object a timestamp in every zone,
            // which is what lets a Wonder in a graveyard generate an ordered effect.
            Some(obj) => match granted_at {
                Some(created) => std::cmp::max(obj.timestamp, created),
                None => obj.timestamp,
            },
            // An object that is not in the store has no static abilities to
            // generate effects from, so this is a guard rather than a case.
            None => {
                debug_assert!(false, "static_effect_timestamp for unknown object {id}");
                self.next_timestamp
            }
        }
    }

    /// Register the continuous effects the static abilities of `id` generate
    /// **in `zone`** (CR 113.6).
    ///
    /// Scans the card's abilities for `AbilityType::Static`, asks
    /// `engine::zone_function` whether each functions where the object now is,
    /// and registers a `ContinuousEffect` per lowered atom.
    ///
    /// **Two callers, and the zone is why** (`layers-architecture.md` §13d):
    ///
    /// - `place_on_battlefield`, for `Zone::Battlefield` — there rather than in
    ///   `move_object` because the entity does not exist until the CR 614.1c
    ///   pipeline has decided what the permanent enters *as*.
    /// - `move_object`, for every other zone — Wonder arriving in a graveyard.
    ///   There is no entity, and CR 108.4 makes the controller the owner.
    ///
    /// Rows are removed when the source leaves: `cleanup_zone_state` →
    /// `remove_by_source` off the battlefield, `remove_static_by_source`
    /// elsewhere. Registration is *not* what decides whether an effect applies —
    /// CR 305.7 and Layer 6 can take the ability away without touching the
    /// registry, so `compute.rs` re-checks existence, zone included, at every
    /// layer (`CLAUDE.md`).
    ///
    /// Reads printed abilities on purpose: it runs inside
    /// `place_on_battlefield`, before this object's own effect is registered,
    /// so computing effective characteristics here would be circular. A
    /// *copied* static ability therefore registers nothing here;
    /// `register_copied_static_effects` is the path beside it
    /// (`copy-effects-architecture.md` §4.7 leg 2).
    pub(crate) fn register_static_effects(
        &mut self,
        id: ObjectId,
        controller: PlayerId,
        zone: Zone,
    ) {
        use crate::engine::layers::types::{ContinuousEffect, EffectOrigin};
        use crate::objects::card_data::AbilityType;
        use crate::types::effects::{Duration, Effect};

        // An `Arc` bump rather than a `Vec<AbilityDef>` clone: `move_object` calls
        // this on every zone change, so a deep clone would land on every draw, mill
        // and discard. The clone exists only because the loop mutates `self`.
        let Some(card) = self.objects.get(&id).map(|obj| Arc::clone(&obj.card_data)) else {
            return;
        };
        let card_name = card.name.as_str();

        for ability in card.abilities.iter() {
            // CR 603 — a triggered ability generates no row either; what the
            // dispatcher needs is to know this object is worth asking about,
            // filed by where it will look (`engine::triggers::dispatch`):
            // the battlefield set, or the zone map when CR 113.6k puts the
            // ability's function somewhere else. The same CR 113.6 gate as
            // the static path below, on printed types for the same reason.
            if ability.ability_type == AbilityType::Triggered {
                if let Effect::Triggered(def) = &ability.effect
                    && crate::engine::zone_function::functions_in(ability, &card.types, zone)
                {
                    if zone == Zone::Battlefield {
                        // Accumulated, not replaced: a permanent with two
                        // triggered abilities is one entry reading both
                        // kinds.
                        *self.trigger_sources.entry(id).or_default() |= def.record_kinds();
                    } else {
                        self.zone_trigger_sources.entry(id).or_default().push(ability.id);
                    }
                }
                continue;
            }
            if ability.ability_type != AbilityType::Static {
                continue;
            }

            // CR 113.6 — does this ability function where the object is?
            //
            // PRE-LAYER ZONE: printed types, for the reason the whole function reads
            // printed abilities. Exact rather than an over-approximation: the only
            // thing `functioning_zones` asks the types is CR 113.6's instant-or-sorcery
            // split, and no continuous effect can make a permanent an instant (CR 205.1b).
            if !crate::engine::zone_function::functions_in(ability, &card.types, zone) {
                continue;
            }

            // CR 614.1a — a replacement effect generates no continuous effect and so
            // has no row; what `engine::replacement::gather` needs is to know this
            // permanent is worth asking about. Through the "as long as" wrapper too:
            // the gather evaluates the condition at each proposal, and a conditional
            // source never recorded would be a card that silently does nothing.
            // Filed by where the gather will look for it — the CR 113.6 gate above
            // has already said the ability functions here. Off the battlefield the
            // zone leg sweeps its own map (Darksteel Colossus in a library), whose
            // value is the printed def, so the leg can ask whether it could apply
            // before reading a frame; the two sets below stay battlefield-only,
            // since their sweeps still visit the battlefield alone.
            if let Some(def) = ability.effect.replacement_body() {
                if zone == Zone::Battlefield {
                    self.replacement_ability_sources.insert(id);
                } else {
                    self.zone_replacement_ability_sources
                        .entry(id)
                        .or_default()
                        .push(def.clone());
                }
            }

            // CR 101.2 — the same shape for the same reason. A static
            // restriction ability generates no continuous effect either; what it
            // needs is for `engine::restriction::is_prohibited` to know this
            // permanent is worth asking about.
            let is_restriction = match &ability.effect {
                Effect::Restriction(_) => true,
                Effect::Conditional(_, inner) => matches!(**inner, Effect::Restriction(_)),
                _ => false,
            };
            if is_restriction && zone == Zone::Battlefield {
                self.restriction_ability_sources.insert(id);
            }

            // CR 601.2f / 613.11 — the third shape with no rows: a cost effect applies
            // to a cost being determined, at no layer, and
            // `engine::cost_determination::cost_modifications_for` reads it off the
            // effective list. Through the "as long as" wrapper (`cost-architecture.md`
            // §8 item 1). Only from the battlefield, and the CR 113.6 gate above is why:
            // a spell's own cost ability functions on the stack (CR 113.6d), so an
            // affinity permanent never reaches this line.
            if zone == Zone::Battlefield && ability.effect.as_cost_modification().is_some() {
                self.cost_modification_ability_sources.insert(id);
            }

            // CR 604.3a(3) — a CDA affects only the object that has it, so it needs no
            // `ObjectSet` and no row; `engine::layers::cda` applies it off the object's
            // own effective ability list, in every zone (CR 604.3) and after Layer 6 has
            // had its say. Registering it as well would apply it twice (`CLAUDE.md`).
            if ability.is_characteristic_defining {
                continue;
            }

            let atoms = Self::static_ability_atoms(ability, card_name);

            for (primitive, recipient) in atoms {
                let Some(affected) = Self::static_object_set(recipient, card_name) else {
                    continue;
                };

                // Map primitive → the layer rows it generates. A type-
                // changing primitive produces several siblings; everything else
                // produces one or none.
                let rows = Self::static_primitive_rows(primitive);
                if rows.is_empty() {
                    debug_assert!(
                        false,
                        "static ability on {} lowers to no layer rows. \
                         `static_primitive_rows` has no arm for {:?}, so this \
                         ability is registered nowhere and the card does \
                         nothing. Either the primitive belongs on a non-static \
                         ability, or the lowering table needs an arm.",
                        card_name, primitive
                    );
                    continue;
                }

                // One timestamp for every row this atom generates: CR 613.6 makes them
                // parts of one effect, and CR 613.7a fixes the value. `None` because a
                // printed ability has no "effect that created" it; the clause-2 caller is
                // `register_granted_static_effects`.
                let timestamp = self.static_effect_timestamp(id, ability, None);

                for (layer, modification) in rows {
                    let effect = ContinuousEffect {
                        id: 0, // assigned by registry
                        source: id,
                        origin: EffectOrigin::StaticAbility { ability: ability.id },
                        layer,
                        duration: Duration::WhileSourceOnBattlefield,
                        controller,
                        created_on_turn: self.turn_number,
                        timestamp,
                        affected_objects: affected.clone(),
                        modification,
                    };
                    self.continuous_effects.add(effect);
                }
            }
        }
    }

    /// CR 707.2a + 613.7a — register the continuous effects a *copied* static
    /// ability generates, for each permanent the copy row affects.
    ///
    /// The second leg of `copy-effects-architecture.md` §4.7:
    /// `register_static_effects` reads printed abilities (it runs before this
    /// object's own effect is registered, so an effective read would be
    /// circular), so a permanent that *becomes* a copy of Glorious Anthem would
    /// otherwise have the ability and pump nothing. This is that row, added
    /// beside the printed path rather than by changing its read.
    ///
    /// **A stale row is inert rather than wrong.** These rows are
    /// `EffectOrigin::StaticAbility`, so CR 604.2 re-checks at every layer
    /// whether the source still *has* the ability, against a frame that
    /// includes layer 1; a copy row that expired or that a CR 707.4 re-copy
    /// superseded stops applying on the next walk whatever the registry holds.
    /// Each derived row gets the copy row's own `Duration` so both expire in the
    /// same CR 514.2 sweep; a re-copy within a turn leaves superseded rows inert
    /// until cleanup (Deferred Migrations).
    ///
    /// **Two differences from `register_granted_static_effects`**: CDAs are
    /// skipped, since CR 604.3a(2)'s third clause makes a copied CDA still a
    /// CDA and `layers::cda` applies it; and the row's controller is the copy's
    /// controller (CR 613.7a, CR 109.5), which Mirrorweave makes observable by
    /// copying onto both players' creatures.
    pub(crate) fn register_copied_static_effects(
        &mut self,
        values: &crate::engine::layers::copy::CopiableValues,
        affected: &[ObjectId],
        copy_timestamp: crate::engine::layers::types::Timestamp,
        duration: crate::types::effects::Duration,
    ) {
        use crate::engine::layers::types::{ContinuousEffect, EffectOrigin};

        for &id in affected {
            // Skipped rather than asserted: `apply_copy` filtered to the
            // battlefield, but `static_effect_timestamp` reads the entity and a
            // future producer may not have.
            if !self.battlefield.contains_key(&id) {
                continue;
            }
            // Never `unwrap_or(0)` here: this *writes* the controller into a registry
            // row that resolves `PlayerRef::You` for as long as the row lives, so a
            // guessed P0 would be a silently wrong board. `controller_or_owner` already
            // falls back to CR 108.3's owner, so `None` means the object is not in
            // `game.objects` at all — unreachable after the battlefield check above.
            let Some(controller) = crate::oracle::characteristics::controller_or_owner(self, id)
            else {
                debug_assert!(
                    false,
                    "copied static ability on {id}, which is on the battlefield but has no \
                     object entry; no controller can be derived and none may be invented"
                );
                continue;
            };

            for ability in values.registrable_static_abilities() {
                let context = format!("copied ability {:?} on {}", ability.id, values.name);
                let atoms = Self::static_ability_atoms(ability, &context);

                for (primitive, recipient) in atoms {
                    let Some(affected_set) = Self::static_object_set(recipient, &context) else {
                        continue;
                    };
                    let rows = Self::static_primitive_rows(primitive);
                    if rows.is_empty() {
                        debug_assert!(
                            false,
                            "{} lowers to no layer rows; `static_primitive_rows` has no \
                             arm for {:?}, so the copied ability does nothing.",
                            context, primitive
                        );
                        continue;
                    }

                    // CR 613.7a clause 2 — the copy effect is "the effect that created the
                    // ability", so its timestamp is the second candidate
                    // (`static_effect_timestamp`).
                    let timestamp =
                        self.static_effect_timestamp(id, ability, Some(copy_timestamp));

                    for (layer, modification) in rows {
                        self.continuous_effects.add(ContinuousEffect {
                            id: 0,
                            source: id,
                            origin: EffectOrigin::StaticAbility { ability: ability.id },
                            layer,
                            duration,
                            controller,
                            created_on_turn: self.turn_number,
                            timestamp,
                            affected_objects: affected_set.clone(),
                            modification,
                        });
                    }
                }
            }
        }
    }

    /// Lower a static ability's body into the `(primitive, recipient)` atoms
    /// that become registry rows.
    ///
    /// "Lowering" is the one-way translation from the authored `Effect` tree
    /// to the `ContinuousEffect`s the layer walk applies, in the compiler's
    /// sense. It happens once, at registration, and the rows carry no
    /// back-pointer to the text — so nothing downstream can tell a card that
    /// lowered to nothing from a card that had nothing to say.
    ///
    /// Shared with `resolve::register_granted_static_effects`: the same card
    /// text has to behave the same whether printed or granted, and two copies
    /// of this match drift.
    ///
    /// # Every declining arm is loud
    ///
    /// A dropped atom produces a card that is *inert* — no panic, no wrong
    /// answer, no divergence, nothing `fuzz_games` can see (Deferred Migrations
    /// items 7e and 7f are that shape). `debug_assert!` rather than a hard
    /// error, matching `register_granted_static_effects`'s layer assert: a card
    /// author running the test suite is stopped, and release builds skip and
    /// carry on rather than panicking mid-game.
    pub(crate) fn static_ability_atoms<'a>(
        ability: &'a crate::objects::card_data::AbilityDef,
        card_name: &str,
    ) -> Vec<(&'a crate::types::effects::Primitive, &'a crate::types::effects::EffectRecipient)> {
        Self::atoms_of_static_body(&ability.effect, card_name)
    }

    /// [`static_ability_atoms`](Self::static_ability_atoms) over an effect
    /// *body* rather than an ability, so `Effect::Conditional` can lower what
    /// it wraps through the very same arms — including the loud ones.
    ///
    /// The public entry above stays an *ability* function because that is
    /// what its twelve callers hold and what its name promises; only the
    /// recursion needs an `Effect`. Collapsing the two would push
    /// `&ability.effect` into every call site to save one line here.
    fn atoms_of_static_body<'a>(
        body: &'a crate::types::effects::Effect,
        card_name: &str,
    ) -> Vec<(&'a crate::types::effects::Primitive, &'a crate::types::effects::EffectRecipient)> {
        use crate::types::effects::Effect;

        match body {
            Effect::Atom(p, r) => vec![(p, r)],

            // CR 614.1a — a replacement effect generates no continuous effect and no
            // layer rows; `engine::replacement::gather` reads it off the *effective*
            // ability list at each proposal. A row here would be one ability applying
            // through two channels, the CDA mistake in a second costume.
            Effect::Replacement(_) => Vec::new(),

            // CR 101.2 — the same, for the same reason. `engine::restriction::is_prohibited`
            // reads a "can't" off the effective list when the question is asked, which
            // is what lets Humility strip it.
            Effect::Restriction(_) => Vec::new(),

            // CR 601.2f / 613.11 — the third of the same shape. A cost effect
            // has no layer and applies to no object, so it generates no row;
            // `engine::cost_determination::cost_modifications_for` reads it off this object's
            // *effective* ability list when a cost is determined.
            Effect::CostModification(_) => Vec::new(),
            // CR 603 — the fourth static shape with no rows; the dispatcher
            // reads it off the effective list.
            Effect::Triggered(_) => Vec::new(),

            Effect::Sequence(effects) => {
                let mut atoms = Vec::with_capacity(effects.len());
                for effect in effects {
                    match effect {
                        Effect::Atom(p, r) => atoms.push((p, r)),
                        _ => debug_assert!(
                            false,
                            "static ability on {} has a non-atomic entry inside \
                             its `Effect::Sequence`. Only `Effect::Atom` lowers \
                             to a continuous effect, so that entry registers \
                             nothing while its siblings register normally — a \
                             half-working card, which is worse than one that \
                             does nothing at all.",
                            card_name
                        ),
                    }
                }
                atoms
            }

            // "As long as [X], [Y]": the rows are [Y]'s, registered unconditionally;
            // [X] stays on the ability, where CR 604.2's existence check evaluates it
            // against the live board at the row's layer (`layers-architecture.md` §13b
            // decision 5). A card whose condition is false as it enters still registers
            // its rows, the only reading that survives a condition changing with no zone
            // change to notice.
            Effect::Conditional(_, inner) => Self::atoms_of_static_body(inner, card_name),

            _ => {
                debug_assert!(
                    false,
                    "static ability on {} has a body the lowering cannot \
                     express: `Optional`, `Modal`, `ForEach` and `Repeat` are \
                     all resolution-time shapes, and a static ability is \
                     declarative — it does not resolve, so there is no moment \
                     at which a mode is chosen or a loop runs. If the card \
                     really reads this way, it wants a triggered or activated \
                     ability instead.",
                    card_name
                );
                Vec::new()
            }
        }
    }

    /// Lower a static atom's recipient into an `ObjectSet`.
    ///
    /// `None` means it could not be lowered; see `static_ability_atoms` for why
    /// that is loud rather than a quiet `continue`.
    pub(crate) fn static_object_set(
        recipient: &crate::types::effects::EffectRecipient,
        card_name: &str,
    ) -> Option<crate::engine::layers::types::ObjectSet> {
        use crate::engine::layers::types::ObjectSet;
        use crate::types::effects::EffectRecipient;

        match recipient {
            // Stored verbatim, `PlayerRef` and all: resolving "you" here would snapshot
            // the source's controller at ETB, and CR 109.5 wants its *current* one, so
            // `compute::object_matches_filter` does it per layer. The zone-reaching form
            // is lowered through the same constructor so the two recipients cannot mean
            // different things.
            EffectRecipient::FilteredObjectsIn(filter, zones) => {
                Some(ObjectSet::filter_in(filter.clone(), *zones))
            }
            EffectRecipient::FilteredPermanents(filter) => {
                Some(ObjectSet::battlefield_filter(filter.clone()))
            }
            EffectRecipient::Implicit => Some(ObjectSet::SourceOnly),
            // Likewise unresolved: the host is read during the walk, which is
            // what makes it fine that this runs before the Aura is attached.
            EffectRecipient::Host => Some(ObjectSet::Host),

            // `Target` and `Choose` need a resolution to pick with, and
            // `Controller` names a player where an `ObjectSet` names objects.
            // A static ability has none of the three.
            _ => {
                debug_assert!(
                    false,
                    "static ability on {} has recipient {:?}, which cannot \
                     become an `ObjectSet`. `Target`/`Choose` require a \
                     resolution to select with, and a static ability never \
                     resolves; `Controller` names a player, not a set of \
                     objects. Use `FilteredPermanents` for \"permanents you \
                     control\", or `Implicit` for \"this permanent\".",
                    card_name, recipient
                );
                None
            }
        }
    }

    /// Lower one atom of a static ability into the layer rows it generates.
    ///
    /// Shared by `register_static_effects` (printed abilities, at ETB) and
    /// `register_granted_static_effects` (CR 613.7a clause 2, when an effect
    /// grants a static-bodied ability). They must agree: a granted "creatures
    /// you control get +1/+1" has to produce the same row a printed one does,
    /// or the same card text behaves differently depending on how it arrived.
    ///
    /// Returns several rows only for `ChangeType`, whose parts CR 613.6 sends
    /// to different layers while they stay one effect sharing a timestamp and
    /// source. An empty `Vec` means the primitive generates no continuous
    /// effect; whether that is an error is the caller's call, and both current
    /// callers `debug_assert!` on it — a static ability that lowers to nothing
    /// is a card that silently does nothing.
    pub(crate) fn static_primitive_rows(
        primitive: &crate::types::effects::Primitive,
    ) -> Vec<(crate::engine::layers::types::Layer, crate::engine::layers::types::EffectModification)>
    {
        use crate::engine::layers::types::{EffectModification, Layer, PtValue};
        use crate::types::effects::{ColorChange, Primitive, PlayerRef};

        let single = |layer, modification| vec![(layer, modification)];

        match primitive {
            Primitive::ModifyPowerToughness(p_expr, t_expr, _dur) => single(
                Layer::Layer7cModifyPT,
                EffectModification::ModifyPowerToughness {
                    power: PtValue::from_amount(p_expr),
                    toughness: PtValue::from_amount(t_expr),
                },
            ),
            Primitive::SetPowerToughness(p_expr, t_expr, _dur) => single(
                Layer::Layer7bSetPT,
                EffectModification::SetPowerToughness {
                    power: PtValue::from_amount(p_expr),
                    toughness: PtValue::from_amount(t_expr),
                },
            ),
            Primitive::SwitchPowerToughness(_dur) => {
                single(Layer::Layer7dSwitchPT, EffectModification::SwitchPowerToughness)
            }
            Primitive::GrantKeywordFlag(kw, _dur) => {
                single(Layer::Layer6Ability, EffectModification::GrantKeywordFlag(*kw))
            }
            Primitive::RemoveKeywordFlag(kw, _dur) => {
                single(Layer::Layer6Ability, EffectModification::RemoveKeywordFlag(*kw))
            }
            Primitive::GrantAbility(def, _dur) => single(
                Layer::Layer6Ability,
                EffectModification::GrantAbility(def.clone()),
            ),
            Primitive::LoseAbility(ability_id, _dur) => single(
                Layer::Layer6Ability,
                EffectModification::LoseAbility(*ability_id),
            ),
            Primitive::LoseAllAbilities(_dur) => {
                single(Layer::Layer6Ability, EffectModification::LoseAllAbilities)
            }
            Primitive::ChangeColor(color_change, _dur) => single(
                Layer::Layer5Color,
                match color_change {
                    ColorChange::Add(c) => EffectModification::AddColor(*c),
                    ColorChange::Set(colors) => EffectModification::SetColors(colors.clone()),
                    ColorChange::RemoveAll => EffectModification::RemoveAllColors,
                },
            ),
            // CR 613.1b. `PlayerRef::You` rather than a resolved id keeps this a pure
            // map from primitive to rows; `compute::resolve_set_controller` resolves it
            // during the walk, the only place CR 109.5's *current* controller can be asked.
            Primitive::GainControl(_dur) => single(
                Layer::Layer2Control,
                EffectModification::SetController(PlayerRef::You),
            ),
            Primitive::ChangeType(type_change, _dur) => {
                let mut mods: Vec<EffectModification> = Vec::new();
                if let Some(ref set_types) = type_change.set_types {
                    mods.push(EffectModification::SetTypes(set_types.clone()));
                } else {
                    for t in &type_change.add_types {
                        mods.push(EffectModification::AddType(*t));
                    }
                    for t in &type_change.remove_types {
                        mods.push(EffectModification::RemoveType(*t));
                    }
                }
                if let Some(ref set_subtypes) = type_change.set_subtypes {
                    mods.push(EffectModification::SetSubtypes(set_subtypes.clone()));
                } else {
                    for s in &type_change.add_subtypes {
                        mods.push(EffectModification::AddSubtype(s.clone()));
                    }
                    for s in &type_change.remove_subtypes {
                        mods.push(EffectModification::RemoveSubtype(s.clone()));
                    }
                }
                if let Some(ref set_supertypes) = type_change.set_supertypes {
                    mods.push(EffectModification::SetSupertypes(set_supertypes.clone()));
                } else {
                    for s in &type_change.add_supertypes {
                        mods.push(EffectModification::AddSupertype(*s));
                    }
                    for s in &type_change.remove_supertypes {
                        mods.push(EffectModification::RemoveSupertype(*s));
                    }
                }
                mods.into_iter().map(|m| (Layer::Layer4Type, m)).collect()
            }
            _ => Vec::new(),
        }
    }

    // --- Object management ---

    /// Register a game object in the central store, and give it its id.
    ///
    /// The id comes from the state's counter here and nowhere else, so it is
    /// the same in every process that plays the same game; whatever the
    /// object carried before is discarded. Callers read the id off the
    /// return value.
    pub fn add_object(&mut self, mut obj: GameObject) -> ObjectId {
        let id = ObjectId::from_counter(&mut self.next_object_id);
        obj.id = id;
        // CR 613.7d — an object created in a zone has entered it. The other
        // stamping site is `move_object`; between them every object carries a
        // real timestamp, which `battlefield_ordered` and `static_effect_timestamp`
        // rely on.
        obj.timestamp = self.allocate_timestamp();
        self.objects.insert(id, obj);
        self.bump_layer_epoch();
        id
    }

    /// Take an object out of the store: an ability that resolved, fizzled or
    /// was countered (CR 608.2n, 701.6b), a token that ceased to exist (CR
    /// 704.5d) or was never created (CR 111.5), an activation rewound.
    ///
    /// Not a zone change — none of those has a destination zone, which is why
    /// `move_object` cannot do it — but a layer-walk input all the same: a
    /// memo still answering for a deleted object would say it existed.
    pub fn remove_object(&mut self, id: ObjectId) -> Option<GameObject> {
        let removed = self.objects.remove(&id);
        self.bump_layer_epoch();
        removed
    }

    /// Record the `StackEntry` of a spell or ability on the stack (CR 601.2a,
    /// 602.2a).
    ///
    /// A stack object's controller is read off its entry (CR 108.4, through
    /// `compute::base_controller`), which makes the entry a layer-walk input.
    pub fn set_stack_entry(&mut self, entry: StackEntry) {
        self.stack_entries.insert(entry.object_id, entry);
        self.bump_layer_epoch();
    }

    /// Take the `StackEntry` of `id` — at resolution, when countered, or on
    /// a rewind. Taking nothing writes nothing.
    pub fn take_stack_entry(&mut self, id: ObjectId) -> Option<StackEntry> {
        let entry = self.stack_entries.remove(&id);
        if entry.is_some() {
            self.bump_layer_epoch();
        }
        entry
    }

    /// CR 112.1 — is `id` a **spell** on the stack, as opposed to an activated
    /// ability's ephemeral object?
    ///
    /// One predicate rather than a copy per arm, because two `SelectionFilter`s
    /// ask it — `Spell` and CR 609.7a's `DamageSource` — at three sites each:
    /// the validator, the count, and the enumeration. The `Spell` arms asked
    /// `stack.contains` instead until A4o, and CR 701.6a then put an ability's
    /// object, whose `CardData` is a clone of its source's, into a graveyard as
    /// a second copy of the card.
    ///
    /// **The entry, not the stack, is what answers.** An ability on the stack
    /// is a `GameObject` like any other (CR 113.7a) and nothing about the
    /// object says which it is; the `StackEntry` does, and it is the same field
    /// the resolution and the fizzle already branch on. The one object this
    /// declines is the spell **currently resolving**, whose entry
    /// `resolve_top_of_stack` has already taken — no rule reaches it, since
    /// CR 601.2c and CR 603.3d both choose before a resolution starts.
    pub fn is_spell_on_stack(&self, id: ObjectId) -> bool {
        self.stack_entries.get(&id).is_some_and(|e| e.is_spell)
    }

    /// Get an immutable reference to a game object
    pub fn get_object(&self, id: ObjectId) -> Result<&GameObject, String> {
        self.objects.get(&id).ok_or_else(|| format!("Object {} not found", id))
    }

    /// Get a mutable reference to a game object
    pub fn get_object_mut(&mut self, id: ObjectId) -> Result<&mut GameObject, String> {
        self.objects.get_mut(&id).ok_or_else(|| format!("Object {} not found", id))
    }

    // --- Player accessors ---

    pub fn get_player(&self, id: PlayerId) -> Result<&PlayerState, String> {
        self.players.get(id).ok_or_else(|| format!("Player {} not found", id))
    }

    pub fn get_player_mut(&mut self, id: PlayerId) -> Result<&mut PlayerState, String> {
        self.players.get_mut(id).ok_or_else(|| format!("Player {} not found", id))
    }

    pub fn num_players(&self) -> usize {
        self.players.len()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // The lowering refuses to be quiet
    //
    // Each arm of `static_ability_atoms` / `static_object_set` that declines
    // to lower something now asserts. These tests exist because an assertion
    // nothing exercises is indistinguishable from one that does not fire — and
    // the whole point of this batch is that the failure mode being guarded is
    // *invisible*: a dropped atom yields an inert card, which panics nothing,
    // computes nothing wrong, and stays perfectly deterministic under
    // `fuzz_games`.
    //
    // `debug_assert!` panics under `cargo test` (debug_assertions on) and
    // compiles out in release, so `#[should_panic]` is the right shape and
    // release builds keep the old skip-and-carry-on behavior.
    // -----------------------------------------------------------------------

    mod static_lowering {
        use super::*;
        use crate::objects::card_data::{AbilityDef, AbilityType};
        use crate::types::card_types::CardType;
        use crate::types::effects::{
            AmountExpr, Condition, Duration, Effect, EffectRecipient, ModalCount,
            ObjectFilter, Primitive, SelectionFilter, TargetCount,
        };
        use crate::types::ids::new_ability_id;

        /// A static `AbilityDef` with the given body.
        fn static_ability(effect: Effect) -> AbilityDef {
            AbilityDef {
                is_characteristic_defining: false,
                activation_restriction: crate::objects::card_data::ActivationRestriction::None,
                id: new_ability_id(),
                instances: Vec::new(),
                ability_type: AbilityType::Static,
                costs: Vec::new(),
                effect,
            }
        }

        /// "Creatures you control get +1/+1", as an atom.
        fn anthem_atom() -> Effect {
            Effect::Atom(
                Primitive::ModifyPowerToughness(
                    AmountExpr::Fixed(1),
                    AmountExpr::Fixed(1),
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::FilteredPermanents(ObjectFilter::ByType(CardType::Creature)),
            )
        }

        // --- positive controls: the shapes that DO lower ------------------

        #[test]
        fn test_atom_body_lowers_to_one_row() {
            let ability = static_ability(anthem_atom());
            assert_eq!(GameState::static_ability_atoms(&ability, "T").len(), 1);
        }

        #[test]
        fn test_sequence_body_lowers_to_one_row_per_atom() {
            // Humility's shape: one ability, two atoms, two layers.
            let ability = static_ability(Effect::Sequence(vec![anthem_atom(), anthem_atom()]));
            assert_eq!(GameState::static_ability_atoms(&ability, "T").len(), 2);
        }

        #[test]
        fn test_filtered_and_implicit_recipients_lower() {
            assert!(GameState::static_object_set(
                &EffectRecipient::FilteredPermanents(ObjectFilter::All), "T"
            ).is_some());
            assert!(GameState::static_object_set(&EffectRecipient::Implicit, "T").is_some());
        }

        // --- the arms that decline, each proven loud ----------------------

        /// Not a declining arm: "as long as
        /// [X], [Y]" lowers to [Y]'s atoms, and [X] stays on the ability for
        /// CR 604.2's existence check to read every layer. The rows carry
        /// nothing about the condition, which is what decision 5 buys.
        #[test]
        fn test_conditional_static_body_lowers_to_the_inner_atoms() {
            let ability = static_ability(Effect::Conditional(
                Condition::SourceInZone(crate::types::zones::ZoneSet::BATTLEFIELD),
                Box::new(anthem_atom()),
            ));
            let atoms = GameState::static_ability_atoms(&ability, "Test Card");
            assert_eq!(atoms.len(), 1);
            assert_eq!(
                GameState::static_ability_atoms(&static_ability(anthem_atom()), "Test Card").len(),
                1,
                "and to exactly what the same atom lowers to unconditionally"
            );
        }

        /// The declining arms are declining *through* the wrapper too: a
        /// conditional body the lowering cannot express is as inert as an
        /// unconditional one, and says so at the same volume.
        #[test]
        #[should_panic(expected = "cannot express")]
        fn test_a_conditional_wrapping_an_unlowerable_body_is_loud() {
            let ability = static_ability(Effect::Conditional(
                Condition::SourceInZone(crate::types::zones::ZoneSet::BATTLEFIELD),
                Box::new(Effect::Optional(Box::new(anthem_atom()))),
            ));
            let _ = GameState::static_ability_atoms(&ability, "Test Card");
        }

        /// `Modal`, `Optional`, `ForEach` and `Repeat` are resolution-time
        /// shapes; a static ability never resolves.
        #[test]
        #[should_panic(expected = "cannot express")]
        fn test_modal_static_body_is_loud() {
            let ability = static_ability(Effect::Modal {
                count: ModalCount::Exactly(1),
                modes: vec![anthem_atom()],
            });
            let _ = GameState::static_ability_atoms(&ability, "Test Card");
        }

        /// The nastiest of the set, because it is *partial*: the atomic
        /// siblings register normally and only this entry vanishes, so the card
        /// half-works.
        #[test]
        #[should_panic(expected = "non-atomic entry")]
        fn test_non_atom_inside_a_sequence_is_loud() {
            let ability = static_ability(Effect::Sequence(vec![
                anthem_atom(),
                Effect::Optional(Box::new(anthem_atom())),
            ]));
            let _ = GameState::static_ability_atoms(&ability, "Test Card");
        }

        #[test]
        #[should_panic(expected = "cannot become an `ObjectSet`")]
        fn test_targeting_recipient_on_a_static_is_loud() {
            let _ = GameState::static_object_set(
                &EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                "Test Card",
            );
        }

        #[test]
        #[should_panic(expected = "cannot become an `ObjectSet`")]
        fn test_controller_recipient_on_a_static_is_loud() {
            let _ = GameState::static_object_set(&EffectRecipient::Controller, "Test Card");
        }

        /// A primitive with no arm in `static_primitive_rows`. This one is
        /// checked in `register_static_effects` rather than in a helper, so it
        /// needs the real ETB path.
        #[test]
        #[should_panic(expected = "lowers to no layer rows")]
        fn test_primitive_with_no_lowering_arm_is_loud() {
            use crate::objects::card_data::CardDataBuilder;
            use crate::objects::object::GameObject;
            use crate::types::zones::Zone;

            let card = CardDataBuilder::new("Nonsense Enchantment")
                .card_type(CardType::Enchantment)
                .ability(static_ability(Effect::Atom(
                    // Drawing a card is not a continuous effect; there is no
                    // layer for it and never will be.
                    Primitive::DrawCards(AmountExpr::Fixed(1)),
                    EffectRecipient::Implicit,
                )))
                .build();

            let mut game = GameState::new(2, 20);
            let obj = GameObject::new(card, 0, Zone::Battlefield);
            let id = game.add_object(obj);
            game.place_on_battlefield(id, 0, &EnterMods::NONE);
        }
    }

    #[test]
    fn test_game_creation() {
        let game = GameState::new(2, 20);
        assert_eq!(game.players.len(), 2);
        assert_eq!(game.players[0].life_total, 20);
        assert_eq!(game.players[1].life_total, 20);
        assert_eq!(game.turn_number, 1);
        assert_eq!(game.active_player, 0);
        assert_eq!(game.phase.phase_type, PhaseType::Beginning);
        assert_eq!(game.phase.step, Some(StepType::Untap));
    }

    #[test]
    fn test_phase_step_progression() {
        // Beginning phase: Untap -> Upkeep -> Draw -> (end)
        assert_eq!(next_step(PhaseType::Beginning, StepType::Untap), Some(StepType::Upkeep));
        assert_eq!(next_step(PhaseType::Beginning, StepType::Upkeep), Some(StepType::Draw));
        assert_eq!(next_step(PhaseType::Beginning, StepType::Draw), None);

        // Combat phase: BeginCombat -> ... -> EndCombat -> (end)
        assert_eq!(next_step(PhaseType::Combat, StepType::BeginCombat), Some(StepType::DeclareAttackers));
        assert_eq!(next_step(PhaseType::Combat, StepType::EndCombat), None);

        // Main phases have no steps
        assert_eq!(initial_step(PhaseType::Precombat), None);
        assert_eq!(initial_step(PhaseType::Postcombat), None);
    }

    /// CR 500.1's order, now read off the plan rather than off a chain.
    #[test]
    fn the_natural_plan_is_cr_500_1s_five_phases_in_order() {
        let plan = TurnPlan::natural();
        assert_eq!(
            plan.phases.iter().map(|p| p.phase_type).collect::<Vec<_>>(),
            vec![
                PhaseType::Beginning,
                PhaseType::Precombat,
                PhaseType::Combat,
                PhaseType::Postcombat,
                PhaseType::Ending,
            ]
        );
        assert_eq!(plan.cursor, None, "nothing has been proposed yet");
        assert_eq!(plan.phase_at(4), Some(PhaseType::Ending));
        assert_eq!(plan.phase_at(5), None, "past the end is the turn boundary");
    }

    #[test]
    fn test_stack_entry_default_no_alt_cost() {
        use crate::types::effects::Effect;

        let entry = StackEntry {
            object_id: crate::types::ids::new_object_id(),
            controller: 0,
            chosen_targets: Vec::new(),
            chosen_modes: Vec::new(),
            x_value: None,
            effect: Effect::Sequence(vec![]),
            is_spell: true,
            chosen_alternative_cost: None,
            additional_costs_paid: Vec::new(),
                    cast_from: Some(Zone::Hand),
                    ability_identity: None,
    trigger: None,
};
        assert!(entry.chosen_alternative_cost.is_none());
        assert!(entry.additional_costs_paid.is_empty());
    }
}
