use std::collections::{HashMap, HashSet};

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::engine::resolve::ResolvedTarget;
use crate::events::event::EventLog;
use crate::objects::object::GameObject;
use crate::state::battlefield::BattlefieldEntity;
use crate::state::continuous_effects::ContinuousEffectRegistry;
use crate::state::player::PlayerState;
use crate::types::costs::{AdditionalCost, AlternativeCost};
use crate::types::effects::{CounterType, Effect};
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::zones::Zone;

/// Metadata for a spell or ability on the stack.
///
/// This is the sidecar state for stack objects, analogous to how
/// `BattlefieldEntity` is the sidecar for battlefield permanents.
/// Created when a spell is cast or ability is activated, consumed
/// when the stack entry resolves or is removed.
#[derive(Debug, Clone)]
pub struct StackEntry {
    /// The object ID of this stack entry (matches the key in `stack`)
    pub object_id: ObjectId,
    /// The player who controls this spell/ability
    pub controller: PlayerId,
    /// Targets chosen at cast/activation time (locked in)
    pub chosen_targets: Vec<ResolvedTarget>,
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
    /// (CR 608.2m), so the ephemeral id identifies nothing once it is gone.
    ///
    /// CR 603.7h needs the durable identity: a delayed trigger that fires when
    /// "this ability has resolved for the third time this turn" is counting
    /// *this* ability of *this* permanent (Ashling the Pilgrim), and neither
    /// half can be recovered from the ephemeral.
    pub ability_identity: Option<AbilityIdentity>,
}

/// Which ability of which object — the durable identity of an activated ability,
/// as opposed to the ephemeral stack object representing one activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbilityIdentity {
    /// The permanent the ability was activated from.
    pub source: ObjectId,
    /// Which of its abilities. Stable across activations; see
    /// `oracle::characteristics::get_effective_abilities`.
    pub ability: AbilityId,
}

/// The complete state of a game of Magic.
///
/// All game objects live in the central `objects` store. Zones reference
/// objects by ID. This means zone transitions are just:
/// 1. Update the object's `zone` field
/// 2. Remove its ID from the old zone's collection
/// 3. Add its ID to the new zone's collection
/// 4. Initialize/clean up zone-specific state (e.g. BattlefieldEntity)
#[derive(Debug, Clone)]
pub struct GameState {
    // --- Central object store ---
    /// All game objects indexed by ID
    pub objects: HashMap<ObjectId, GameObject>,

    // --- Players ---
    pub players: Vec<PlayerState>,

    // --- Global zones (player zones are in PlayerState) ---
    /// The stack — LIFO order (last element = top of stack)
    pub stack: Vec<ObjectId>,
    /// Stack entry metadata — keyed by ObjectId
    pub stack_entries: HashMap<ObjectId, StackEntry>,
    /// Battlefield state — keyed by ObjectId
    pub battlefield: HashMap<ObjectId, BattlefieldEntity>,
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

    // --- Combat tracking ---
    pub attacks_declared: bool,
    pub blockers_declared: bool,
    /// Damage division for blockers blocking 2+ attackers (rule 510.1d).
    /// Maps blocker ObjectId → Vec<(attacker ObjectId, damage amount)>.
    /// Populated during declare blockers step, consumed during combat damage.
    /// Phase 3: unused (multi-block requires Banding or "block additional" effects).
    /// Phase 4/5: populated via DecisionProvider::choose_blocker_damage_division.
    pub blocker_damage_divisions: HashMap<ObjectId, Vec<(ObjectId, u64)>>,
    /// Tracks creatures that dealt damage during the first-strike combat damage step.
    /// Used to determine which creatures deal damage in the normal combat damage step:
    /// - First strikers: dealt first-strike damage, skip normal step.
    /// - Double strikers: dealt first-strike damage, deal again in normal step.
    /// - Normal creatures: skip first-strike step, deal in normal step.
    /// Cleared with other combat state in on_phase_end(Combat).
    pub dealt_first_strike_damage: HashSet<ObjectId>,

    // --- Timestamp counter for layer system (rule 613.7) ---
    /// Monotonically increasing counter. Each permanent that enters the
    /// battlefield gets the current value, then the counter increments.
    pub next_timestamp: u64,

    // --- Game-end flags (set by SBAs, checked by Game) ---
    /// Per-player loss flags. SBAs set these; `Game::check_game_over` reads them.
    pub player_lost: Vec<bool>,

    // --- First-turn draw skip (rule 103.8a) ---
    /// If true, the first draw step is skipped (one-time flag for game setup).
    /// In-game "skip draw" effects use the replacement effect system (Phase 6).
    pub skip_first_draw: bool,

    // --- Continuous effects registry (CR 613) ---
    pub continuous_effects: ContinuousEffectRegistry,

    // --- Event log ---
    pub events: EventLog,

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

/// Get the initial step for a phase (None for main phases which have no steps)
fn initial_step(phase_type: PhaseType) -> Option<StepType> {
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

/// Get the next phase in turn order.
///
/// **Future: TurnPlan for extra phases.** Effects like "after this phase, there
/// is an additional combat phase followed by an additional main phase" cannot
/// be expressed by a fixed state machine. When we implement combat (Phase 3),
/// `next_phase` will be replaced by a `TurnPlan` — a mutable Vec of
/// `(PhaseType, Vec<StepType>)` that the engine walks. Effects insert extra
/// entries into the plan, and `advance_turn` reads from it instead of calling
/// this function.
pub fn next_phase(phase_type: PhaseType) -> PhaseType {
    match phase_type {
        PhaseType::Beginning => PhaseType::Precombat,
        PhaseType::Precombat => PhaseType::Combat,
        PhaseType::Combat => PhaseType::Postcombat,
        PhaseType::Postcombat => PhaseType::Ending,
        PhaseType::Ending => PhaseType::Beginning, // wraps to next turn
    }
}

impl GameState {
    /// Create a new game with the given number of players
    pub fn new(num_players: usize, starting_life: i64) -> Self {
        let players: Vec<PlayerState> = (0..num_players)
            .map(|id| PlayerState::new(id, starting_life))
            .collect();

        GameState {
            objects: HashMap::new(),
            players,
            stack: Vec::new(),
            stack_entries: HashMap::new(),
            battlefield: HashMap::new(),
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
            attacks_declared: false,
            blockers_declared: false,
            blocker_damage_divisions: HashMap::new(),
            dealt_first_strike_damage: HashSet::new(),
            next_timestamp: 0,
            player_lost: vec![false; num_players],
            skip_first_draw: false,
            continuous_effects: ContinuousEffectRegistry::new(),
            events: EventLog::new(),
            rng: StdRng::seed_from_u64(Self::DEFAULT_RNG_SEED),
        }
    }

    /// The seed a `GameState` starts with when nobody supplies one.
    ///
    /// A fixed value, not entropy: an unseeded game that is nevertheless
    /// reproducible is the safer default, and it makes every test that shuffles
    /// deterministic without opting in.
    pub const DEFAULT_RNG_SEED: u64 = 0x4D54_4749_4348_4F52; // "MTGICHOR"

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

    // --- Deterministic iteration ---

    /// The battlefield, oldest permanent first.
    ///
    /// **Every sweep whose order can be observed goes through this, not
    /// `battlefield.iter()`.** `battlefield` is a `HashMap`, and `RandomState`
    /// reseeds itself per *process*, so a direct iteration hands the legal
    /// action list, the mana sources and the SBA sweeps to the caller in a
    /// different order on every run — which is how `fuzz_games --seed N` came
    /// to be irreproducible. Sorting by `ObjectId` is not a fix: ids are v4
    /// UUIDs, so the key is itself random.
    ///
    /// `BattlefieldEntity::timestamp` is the deterministic key. It is allocated
    /// once per `place_on_battlefield` from `next_timestamp`, a monotonic
    /// counter, and never reassigned — so it is unique across the battlefield
    /// and totally orders it. It is also the order CR 613.7 already cares
    /// about, oldest first.
    ///
    /// Order-irrelevant sweeps — "untap every permanent", "clear all damage" —
    /// may still iterate the map directly; they touch disjoint entries and emit
    /// nothing.
    pub fn battlefield_ordered(&self) -> Vec<(ObjectId, &BattlefieldEntity)> {
        let mut entries: Vec<(ObjectId, &BattlefieldEntity)> =
            self.battlefield.iter().map(|(&id, e)| (id, e)).collect();
        entries.sort_by_key(|(_, e)| e.timestamp);
        entries
    }

    /// The battlefield's object ids, oldest permanent first.
    /// See [`GameState::battlefield_ordered`] for why this exists.
    pub fn battlefield_ids_ordered(&self) -> Vec<ObjectId> {
        let mut ids: Vec<ObjectId> = self.battlefield.keys().copied().collect();
        ids.sort_by_key(|id| self.battlefield[id].timestamp);
        ids
    }

    /// Allocate and return the next timestamp value.
    pub fn allocate_timestamp(&mut self) -> u64 {
        let ts = self.next_timestamp;
        self.next_timestamp += 1;
        ts
    }

    // --- Battlefield convenience ---

    /// Create a `BattlefieldEntity` for the given object and insert it onto the
    /// battlefield. Allocates a fresh timestamp and uses the current turn number.
    /// Returns a mutable reference to the inserted entry so callers can tweak
    /// fields (e.g. `entry.tapped = true`) without a second lookup.
    pub fn place_on_battlefield(&mut self, id: ObjectId, controller: PlayerId) -> &mut BattlefieldEntity {
        let ts = self.allocate_timestamp();
        let current_turn = self.turn_number;
        let entry = BattlefieldEntity::new(id, controller, ts, current_turn);
        self.battlefield.insert(id, entry);

        self.init_etb_counters(id);
        self.register_static_effects(id, controller);

        self.battlefield.get_mut(&id).unwrap()
    }

    /// Put `n` counters of `counter_type` on a permanent, allocating the CR
    /// 613.7c timestamp.
    ///
    /// The normal entry point — `BattlefieldEntity::add_counters` needs a
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
    /// `register_granted_static_effects` is the caller that passes `Some`. See
    /// `layers-architecture.md` §15.2 item 4, which concluded the metadata this
    /// clause wants is a timestamp rather than the `AbilityOrigin` enum Phase LD
    /// declined to build — and it was right.
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
        match self.battlefield.get(&id) {
            // CR 613.7a: "...the same timestamp as the object the static
            // ability is on, or the timestamp of the effect that created the
            // ability, whichever is later."
            //
            // Two candidates, later wins. A printed ability has only the first,
            // so it is returned unchanged.
            Some(entry) => match granted_at {
                Some(created) => std::cmp::max(entry.timestamp, created),
                None => entry.timestamp,
            },
            // Unreachable because the only caller is `register_static_effects`,
            // which runs from `place_on_battlefield` after the entity is
            // inserted — *not* because a non-battlefield object cannot have a
            // functioning static ability. It can: CR 113.6b, and Wonder ("as
            // long as this card is in your graveyard and you control an Island,
            // creatures you control have flying") is the stock example.
            //
            // When those are modelled, this fallback is not the fix. CR 613.7d
            // gives an object a timestamp when it enters *any* zone, but we
            // only store one on `BattlefieldEntity`, so a graveyard Wonder has
            // nowhere to read one from. The timestamp has to move onto the
            // object. See Deferred Migrations item 9.
            None => {
                debug_assert!(false, "static_effect_timestamp for non-battlefield object {id}");
                self.next_timestamp
            }
        }
    }

    /// Register continuous effects from static abilities on a permanent.
    ///
    /// Called when a permanent enters the battlefield. Scans the card's
    /// abilities for `AbilityType::Static`, extracts the primitive and
    /// recipient, and registers a `ContinuousEffect` in the registry.
    ///
    /// Duration comes from the primitive (typically `WhileSourceOnBattlefield`).
    /// Effects are removed when the source leaves the battlefield via
    /// `cleanup_zone_state` → `remove_by_source`.
    ///
    /// Registration is *not* what decides whether an effect applies. CR 305.7
    /// and Layer 6 can take the generating ability away without touching the
    /// registry, so `compute.rs` re-checks existence at every layer. This
    /// function's job is only to put the row there with the right timestamp.
    ///
    /// Reads printed abilities on purpose: it runs inside
    /// `place_on_battlefield`, before this object's own effect is registered,
    /// so computing effective characteristics here would be circular.
    /// Lower a card-definition amount into a registry P/T value.
    ///
    /// `Fixed` collapses to a literal so the common case stays a plain integer;
    /// everything else is carried through as an expression and evaluated at
    /// every layer by `compute::evaluate_pt_value`.
    ///
    /// This used to `continue` on any non-`Fixed` amount, which silently
    /// dropped the whole atom — a static ability with a computed P/T registered
    /// nothing at all and failed no test (Deferred Migrations item 7e).
    fn register_static_effects(&mut self, id: ObjectId, controller: PlayerId) {
        use crate::engine::layers::types::{ContinuousEffect, EffectOrigin};
        use crate::objects::card_data::AbilityType;
        use crate::types::effects::Duration;

        let (abilities, card_name) = if let Some(obj) = self.objects.get(&id) {
            (obj.card_data.abilities.clone(), obj.card_data.name.clone())
        } else {
            return;
        };

        for ability in &abilities {
            if ability.ability_type != AbilityType::Static {
                continue;
            }

            // CR 604.3a(3) — a characteristic-defining ability affects only the
            // object that has it, so it needs no `AffectedSet` and no row here.
            // `engine::layers::cda` applies it off the object's own effective
            // ability list instead, which is also what makes it work in every
            // zone (CR 604.3) and what lets Layer 6 remove it before Layer 7a
            // reads it. Registering it as well would apply it twice.
            if ability.is_characteristic_defining {
                continue;
            }

            let atoms = Self::static_ability_atoms(ability, &card_name);

            for (primitive, recipient) in atoms {
                let Some(affected) = Self::static_affected_set(recipient, &card_name) else {
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

                // One timestamp for every row this atom generates. CR 613.6
                // makes them parts of one effect; CR 613.7a fixes the value —
                // "a continuous effect generated by a static ability has the
                // same timestamp as the object the static ability is on".
                //
                // `None`: `register_static_effects` runs at ETB off printed
                // abilities, so there is no "effect that created the ability"
                // and CR 613.7a's second clause has nothing to contribute. The
                // clause-2 caller is `register_granted_static_effects`.
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
                        affected: affected.clone(),
                        modification,
                    };
                    self.continuous_effects.add(effect);
                }
            }
        }
    }

    /// Lower a static ability's body into the `(primitive, recipient)` atoms
    /// that become registry rows.
    ///
    /// Shared with `resolve::register_granted_static_effects` for the same
    /// reason `static_primitive_rows` is: the same card text has to behave the
    /// same whether it was printed or granted, and two copies of this match
    /// drift.
    ///
    /// # Every declining arm is loud
    ///
    /// This function used to `continue` on anything it could not lower, and so
    /// did its recipient twin below. That is the failure mode this codebase has
    /// already paid for twice — Deferred Migrations item 7e was a `continue` on
    /// a non-`Fixed` amount that "silently dropped the whole atom ... and failed
    /// no test", and item 7f is the same shape still open. A dropped atom
    /// produces a card that is *inert*: no panic, no wrong answer, no
    /// divergence. `fuzz_games` cannot see it, because a card that does nothing
    /// crashes nothing and stays perfectly deterministic. The only thing that
    /// catches it is refusing to be quiet at the door.
    ///
    /// `debug_assert!` rather than a hard error, matching
    /// `register_granted_static_effects`'s layer assert and
    /// `compute::evaluate_pt_value`: a card author running the test suite is
    /// stopped, and release builds keep the old skip-and-carry-on behavior
    /// rather than panicking mid-game.
    pub(crate) fn static_ability_atoms<'a>(
        ability: &'a crate::objects::card_data::AbilityDef,
        card_name: &str,
    ) -> Vec<(&'a crate::types::effects::Primitive, &'a crate::types::effects::EffectRecipient)> {
        use crate::types::effects::Effect;

        match &ability.effect {
            Effect::Atom(p, r) => vec![(p, r)],

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

            // Not an authoring error — an unimplemented feature, and the
            // largest one standing between this engine and card breadth. "As
            // long as [X], [Y]" is one of the most common static shapes in
            // Magic, and Layer 2 wants it specifically (Dog Umbra is a
            // conditional static whose condition is *control*).
            Effect::Conditional(..) => {
                debug_assert!(
                    false,
                    "static ability on {} is `Effect::Conditional`, which the \
                     lowering cannot express yet — see `codebase-state.md` \
                     Deferred Migrations item 7f. The card would register \
                     nothing and silently do nothing. Model it as an \
                     unconditional static for now, or implement 7f.",
                    card_name
                );
                Vec::new()
            }

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

    /// Lower a static atom's recipient into an `AffectedSet`.
    ///
    /// `None` means it could not be lowered; see `static_ability_atoms` for why
    /// that is loud rather than a quiet `continue`.
    pub(crate) fn static_affected_set(
        recipient: &crate::types::effects::EffectRecipient,
        card_name: &str,
    ) -> Option<crate::engine::layers::types::AffectedSet> {
        use crate::engine::layers::types::AffectedSet;
        use crate::types::effects::EffectRecipient;

        match recipient {
            // The filter is stored verbatim, `PlayerRef` and all. Resolving
            // "you" here would snapshot the source's controller at ETB; CR
            // 109.5 wants its *current* one, so `compute::permanent_matches_filter`
            // does it per layer.
            EffectRecipient::FilteredPermanents(filter) => {
                Some(AffectedSet::Filter { filter: filter.clone() })
            }
            EffectRecipient::Implicit => Some(AffectedSet::SourceOnly),

            // `Target` and `Choose` need a resolution to pick with, and
            // `Controller` names a player where an `AffectedSet` names objects.
            // A static ability has none of the three.
            _ => {
                debug_assert!(
                    false,
                    "static ability on {} has recipient {:?}, which cannot \
                     become an `AffectedSet`. `Target`/`Choose` require a \
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
            // CR 613.1b. `PlayerRef::You` rather than a resolved id, and that is
            // what keeps this function a pure map from primitive to rows: it has
            // no game, no source and no controller to resolve one with.
            // `compute::resolve_set_controller` does it during the walk, which
            // is also the only place CR 109.5's "current controller of the
            // object it's on" can be asked.
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

    /// Set initial counters for a permanent entering the battlefield.
    ///
    /// Currently handles:
    /// - Planeswalker loyalty (rule 306.5b): set loyalty counters equal to
    ///   printed loyalty. Replacement effects (e.g. Doubling Season) will
    ///   intercept this in the replacement-effect layer (Phase 7+).
    ///
    /// Future: Sagas (lore counters), other ETB counter patterns.
    fn init_etb_counters(&mut self, id: ObjectId) {
        let loyalty = match self.objects.get(&id) {
            Some(obj) => obj.card_data.loyalty,
            None => return,
        };
        if !crate::oracle::characteristics::has_type(
            self, id, crate::types::card_types::CardType::Planeswalker)
        {
            return;
        }
        if let Some(loyalty) = loyalty {
            if loyalty > 0 {
                self.add_counters(id, CounterType::Loyalty, loyalty as u32);
            }
        }
    }

    // --- Object management ---

    /// Register a game object in the central store
    pub fn add_object(&mut self, obj: GameObject) -> ObjectId {
        let id = obj.id;
        self.objects.insert(id, obj);
        id
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
    // Each arm of `static_ability_atoms` / `static_affected_set` that declines
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
            PermanentFilter, Primitive, SelectionFilter, TargetCount,
        };
        use crate::types::ids::new_ability_id;

        /// A static `AbilityDef` with the given body.
        fn static_ability(effect: Effect) -> AbilityDef {
            AbilityDef {
                is_characteristic_defining: false,
                id: new_ability_id(),
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
                EffectRecipient::FilteredPermanents(PermanentFilter::ByType(CardType::Creature)),
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
            assert!(GameState::static_affected_set(
                &EffectRecipient::FilteredPermanents(PermanentFilter::All), "T"
            ).is_some());
            assert!(GameState::static_affected_set(&EffectRecipient::Implicit, "T").is_some());
        }

        // --- the arms that decline, each proven loud ----------------------

        /// The big one. "As long as [X], [Y]" is one of the most common static
        /// shapes in Magic and the lowering cannot express it yet (Deferred
        /// Migrations item 7f). Before this assert it registered nothing and
        /// said nothing.
        #[test]
        #[should_panic(expected = "item 7f")]
        fn test_conditional_static_body_is_loud() {
            let ability = static_ability(Effect::Conditional(
                Condition::SourceOnBattlefield,
                Box::new(anthem_atom()),
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
        /// half-works. `filter_map` used to swallow it.
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
        #[should_panic(expected = "cannot become an `AffectedSet`")]
        fn test_targeting_recipient_on_a_static_is_loud() {
            let _ = GameState::static_affected_set(
                &EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                "Test Card",
            );
        }

        #[test]
        #[should_panic(expected = "cannot become an `AffectedSet`")]
        fn test_controller_recipient_on_a_static_is_loud() {
            let _ = GameState::static_affected_set(&EffectRecipient::Controller, "Test Card");
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
            let id = obj.id;
            game.add_object(obj);
            game.place_on_battlefield(id, 0);
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

    #[test]
    fn test_phase_progression() {
        assert_eq!(next_phase(PhaseType::Beginning), PhaseType::Precombat);
        assert_eq!(next_phase(PhaseType::Precombat), PhaseType::Combat);
        assert_eq!(next_phase(PhaseType::Combat), PhaseType::Postcombat);
        assert_eq!(next_phase(PhaseType::Postcombat), PhaseType::Ending);
        assert_eq!(next_phase(PhaseType::Ending), PhaseType::Beginning);
    }

    #[test]
    fn test_stack_entry_default_no_alt_cost() {
        use crate::engine::resolve::ResolvedTarget;
        use crate::types::effects::Effect;

        let entry = StackEntry {
            object_id: crate::types::ids::new_object_id(),
            controller: 0,
            chosen_targets: Vec::<ResolvedTarget>::new(),
            chosen_modes: Vec::new(),
            x_value: None,
            effect: Effect::Sequence(vec![]),
            is_spell: true,
            chosen_alternative_cost: None,
            additional_costs_paid: Vec::new(),
                    cast_from: Some(Zone::Hand),
                    ability_identity: None,
};
        assert!(entry.chosen_alternative_cost.is_none());
        assert!(entry.additional_costs_paid.is_empty());
    }
}
