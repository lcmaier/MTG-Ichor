use std::collections::HashMap;

use crate::engine::resolve::ResolvedTarget;
use crate::events::event::EventLog;
use crate::objects::object::GameObject;
use crate::state::battlefield::BattlefieldEntity;
use crate::state::player::PlayerState;
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
            next_timestamp: 0,
            player_lost: vec![false; num_players],
            skip_first_draw: false,
            events: EventLog::new(),
        }
    }

    /// Allocate and return the next timestamp value.
    pub fn allocate_timestamp(&mut self) -> u64 {
        let ts = self.next_timestamp;
        self.next_timestamp += 1;
        ts
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

    // --- Zone queries ---

    /// Get all object IDs on the battlefield controlled by a player
    pub fn permanents_controlled_by(&self, player_id: PlayerId) -> Vec<ObjectId> {
        self.battlefield.iter()
            .filter(|(_, entry)| entry.controller == player_id)
            .map(|(id, _)| *id)
            .collect()
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
}
