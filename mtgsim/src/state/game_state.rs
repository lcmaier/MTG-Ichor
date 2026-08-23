use std::collections::{HashMap, HashSet};

use crate::engine::resolve::ResolvedTarget;
use crate::events::event::EventLog;
use crate::objects::object::GameObject;
use crate::state::battlefield::BattlefieldEntity;
use crate::state::continuous_effects::ContinuousEffectRegistry;
use crate::state::player::PlayerState;
use crate::types::costs::{AdditionalCost, AlternativeCost};
use crate::types::effects::{CounterType, Effect};
use crate::types::ids::{ObjectId, PlayerId};

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
        use crate::engine::layers::types::{
            AffectedSet, ContinuousEffect, EffectOrigin,
        };
        use crate::objects::card_data::AbilityType;
        use crate::types::effects::{Duration, Effect, EffectRecipient, Primitive};

        let abilities = if let Some(obj) = self.objects.get(&id) {
            obj.card_data.abilities.clone()
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

            // Collect atoms: flatten Sequence, or extract single Atom.
            // Static abilities are declarative — Optional/Modal/Conditional don't
            // apply at the card-definition level. Non-Atom entries in a Sequence
            // (if any exist) are safely skipped; extend here as needed.
            let atoms: Vec<(&Primitive, &EffectRecipient)> = match &ability.effect {
                Effect::Atom(p, r) => vec![(p, r)],
                Effect::Sequence(effects) => effects.iter().filter_map(|e| {
                    if let Effect::Atom(p, r) = e { Some((p, r)) } else { None }
                }).collect(),
                _ => continue,
            };

            for (primitive, recipient) in atoms {
                // Map recipient → AffectedSet
                let affected = match recipient {
                    EffectRecipient::FilteredPermanents(filter) => {
                        // Resolve PlayerRef in the filter to build AffectedSet
                        let ctrl = Self::extract_controller_from_filter(filter, controller);
                        AffectedSet::Filter {
                            filter: filter.clone(),
                            controller: ctrl,
                        }
                    }
                    EffectRecipient::Implicit => AffectedSet::SourceOnly,
                    _ => continue,
                };

                // Map primitive → the layer rows it generates. A type-
                // changing primitive produces several siblings; everything else
                // produces one or none.
                let rows = Self::static_primitive_rows(primitive);
                if rows.is_empty() {
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
    /// source. An empty `Vec` means the
    /// primitive generates no continuous effect — it is not an error.
    pub(crate) fn static_primitive_rows(
        primitive: &crate::types::effects::Primitive,
    ) -> Vec<(crate::engine::layers::types::Layer, crate::engine::layers::types::EffectModification)>
    {
        use crate::engine::layers::types::{EffectModification, Layer, PtValue};
        use crate::types::effects::{ColorChange, Primitive};

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

    /// Walk a `PermanentFilter` to find `ByController(PlayerRef)` and resolve
    /// it to a concrete `PlayerId`. Returns `Some(id)` if the filter contains
    /// a controller constraint, `None` otherwise.
    pub(crate) fn extract_controller_from_filter(
        filter: &crate::types::effects::PermanentFilter,
        controller: PlayerId,
    ) -> Option<PlayerId> {
        use crate::types::effects::{PermanentFilter, PlayerRef};
        match filter {
            PermanentFilter::ByController(PlayerRef::You) => Some(controller),
            PermanentFilter::ByController(PlayerRef::Player(pid)) => Some(*pid),
            PermanentFilter::And(a, b) => {
                Self::extract_controller_from_filter(a, controller)
                    .or_else(|| Self::extract_controller_from_filter(b, controller))
            }
            _ => None,
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
        };
        assert!(entry.chosen_alternative_cost.is_none());
        assert!(entry.additional_costs_paid.is_empty());
    }
}
