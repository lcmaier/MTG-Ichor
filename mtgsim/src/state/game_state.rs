use std::collections::{HashMap, HashSet};

use crate::engine::resolve::ResolvedTarget;
use crate::events::event::EventLog;
use crate::objects::object::GameObject;
use crate::state::battlefield::BattlefieldEntity;
use crate::state::continuous_effects::ContinuousEffectRegistry;
use crate::state::player::PlayerState;
use crate::types::costs::{AdditionalCost, AlternativeCost};
use crate::types::effects::Effect;
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

    /// The timestamp a continuous effect generated by `ability` on `id` gets
    /// (CR 613.7a).
    ///
    /// > A continuous effect generated by a static ability has the same
    /// > timestamp as the object the static ability is on, or the timestamp of
    /// > the effect that created the ability, whichever is later.
    ///
    /// Only the first clause is implemented. The second needs a producer that
    /// *creates* an ability — `GrantAbility(AbilityDef)` with a Static body,
    /// which is Layer 6 work. `GrantKeyword`, the one ability-granting channel
    /// wired today, writes `EffectiveCharacteristics::keywords`, and nothing
    /// derives a `ContinuousEffect` from a keyword, so the second clause has no
    /// reachable code path yet.
    ///
    /// It is a `max()` inside this function when it does: that is the whole
    /// reason every static-effect timestamp is routed through here instead of
    /// reading `battlefield[id].timestamp` at each construction site. See
    /// `layers-architecture.md` §15.2 item 4, which concluded the metadata that
    /// clause wants is a timestamp rather than the `AbilityOrigin` enum Phase
    /// LD declined to build.
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
    ) -> u64 {
        match self.battlefield.get(&id) {
            Some(entry) => entry.timestamp,
            // A static ability of an object that isn't on the battlefield
            // generates no effect, so this is unreachable from
            // `register_static_effects`, which runs after the entity is
            // inserted. "Now" is the least-wrong answer if it ever isn't.
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
    fn register_static_effects(&mut self, id: ObjectId, controller: PlayerId) {
        use crate::engine::layers::types::{
            AffectedSet, ContinuousEffect, EffectModification, EffectOrigin, Layer,
        };
        use crate::objects::card_data::AbilityType;
        use crate::types::effects::{AmountExpr, Duration, Effect, EffectRecipient, Primitive};

        let abilities = if let Some(obj) = self.objects.get(&id) {
            obj.card_data.abilities.clone()
        } else {
            return;
        };

        for ability in &abilities {
            if ability.ability_type != AbilityType::Static {
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

                // Map primitive → (layer, modification)
                let (layer, modification) = match primitive {
                    Primitive::ModifyPowerToughness(p_expr, t_expr, _dur) => {
                        let p = match p_expr { AmountExpr::Fixed(n) => *n as i32, _ => continue };
                        let t = match t_expr { AmountExpr::Fixed(n) => *n as i32, _ => continue };
                        (Layer::Layer7cModifyPT, EffectModification::ModifyPowerToughness { power: p, toughness: t })
                    }
                    Primitive::SetPowerToughness(p_expr, t_expr, _dur) => {
                        let p = match p_expr { AmountExpr::Fixed(n) => *n as i32, _ => continue };
                        let t = match t_expr { AmountExpr::Fixed(n) => *n as i32, _ => continue };
                        (Layer::Layer7bSetPT, EffectModification::SetPowerToughness { power: p, toughness: t })
                    }
                    Primitive::SwitchPowerToughness(_dur) => {
                        (Layer::Layer7dSwitchPT, EffectModification::SwitchPowerToughness)
                    }
                    Primitive::GrantKeyword(kw, _dur) => {
                        (Layer::Layer6Ability, EffectModification::GrantKeyword(*kw))
                    }
                    Primitive::ChangeColor(color_change, _dur) => {
                        use crate::types::effects::ColorChange;
                        let modification = match color_change {
                            ColorChange::Add(c) => EffectModification::AddColor(*c),
                            ColorChange::Set(colors) => EffectModification::SetColors(colors.clone()),
                            ColorChange::RemoveAll => EffectModification::RemoveAllColors,
                        };
                        (Layer::Layer5Color, modification)
                    }
                    Primitive::ChangeType(type_change, _dur) => {
                        // Static type-changing effects register multiple sibling
                        // effects (one per modification). We handle them inline
                        // here to share the same timestamp.
                        let ts = self.static_effect_timestamp(id, ability);
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

                        for modification in mods {
                            let effect = ContinuousEffect {
                                id: 0,
                                source: id,
                                origin: EffectOrigin::StaticAbility { ability: ability.id },
                                layer: Layer::Layer4Type,
                                duration: Duration::WhileSourceOnBattlefield,
                                controller,
                                created_on_turn: self.turn_number,
                                timestamp: ts,
                                affected: affected.clone(),
                                modification,
                            };
                            self.continuous_effects.add(effect);
                        }
                        continue; // already registered, skip the common path below
                    }
                    _ => continue,
                };

                let timestamp = self.static_effect_timestamp(id, ability);
                let effect = ContinuousEffect {
                    id: 0, // assigned by registry
                    source: id,
                    origin: EffectOrigin::StaticAbility { ability: ability.id },
                    layer,
                    duration: Duration::WhileSourceOnBattlefield,
                    controller,
                    created_on_turn: self.turn_number,
                    timestamp,
                    affected,
                    modification,
                };
                self.continuous_effects.add(effect);
            }
        }
    }

    /// Walk a `PermanentFilter` to find `ByController(PlayerRef)` and resolve
    /// it to a concrete `PlayerId`. Returns `Some(id)` if the filter contains
    /// a controller constraint, `None` otherwise.
    fn extract_controller_from_filter(
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
        if let Some(obj) = self.objects.get(&id) {
            if crate::oracle::characteristics::has_type(
                self, id, crate::types::card_types::CardType::Planeswalker) {
                if let Some(loyalty) = obj.card_data.loyalty {
                    if loyalty > 0 {
                        self.battlefield.get_mut(&id).unwrap()
                            .add_counters(crate::types::effects::CounterType::Loyalty, loyalty as u32);
                    }
                }
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
