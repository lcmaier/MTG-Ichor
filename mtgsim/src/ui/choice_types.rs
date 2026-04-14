use crate::events::event::DamageTarget;
use crate::state::battlefield::AttackTarget;
use crate::types::colors::Color;
use crate::types::costs::{AdditionalCost, AlternativeCost};
use crate::types::effects::{CounterType, EffectRecipient};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::mana::{ManaCost, ManaType};

use super::decision::PriorityAction;

/// What kind of decision is being made. UIs use this to render appropriate
/// screens. AI agents can match on this for specialized heuristics.
/// Adding a new variant here is the ONLY change needed when a new decision
/// type is introduced — no trait methods or impl changes.
///
/// Exhaustive matching is intentional: single-crate project, compiler flags
/// every match site when a variant is added.
///
/// Only variants that correspond to currently-implemented engine decisions
/// are included. New variants are added as the engine grows — the exhaustive
/// matching ensures every DP impl gets updated at compile time.
#[derive(Debug, Clone)]
pub enum ChoiceKind {
    // --- Priority & Turn Structure ---
    PriorityAction,

    // --- Combat ---
    DeclareAttackers,
    DeclareBlockers,
    AssignCombatDamage { attacker_id: ObjectId },
    AssignTrampleDamage { attacker_id: ObjectId, defending_target: DamageTarget },

    // --- Casting Pipeline (601.2) ---
    ChooseXValue { spell_id: ObjectId, x_count: u64 },
    ChooseAlternativeCost,
    ChooseAdditionalCosts,
    /// Select recipients for an effect (covers both MTG "target" and non-targeting
    /// "choose" — the `EffectRecipient` field distinguishes them).
    SelectRecipients { recipient: EffectRecipient, spell_id: ObjectId },
    GenericManaAllocation { mana_cost: ManaCost },

    // --- State-Based & Cleanup ---
    DiscardToHandSize,
    LegendRule { legend_name: String },
}

/// Wrapper carrying the semantic kind. No display text — each DP impl formats
/// its own prompts by matching on `kind`. This keeps choice types pure (no
/// presentation leakage into the engine boundary).
#[derive(Debug, Clone)]
pub struct ChoiceContext {
    pub kind: ChoiceKind,
}

/// A single selectable option presented to the DP.
#[derive(Debug, Clone)]
pub enum ChoiceOption {
    /// A game object (creature, card in hand, permanent, etc.)
    Object(ObjectId),
    /// A player
    Player(PlayerId),
    /// A game action (for priority)
    Action(PriorityAction),
    /// An attacker-target pair (for declare attackers)
    AttackerTarget(ObjectId, AttackTarget),
    /// A blocker-attacker pair (for declare blockers)
    BlockerAttacker(ObjectId, ObjectId),
    /// Pay the normal mana cost (used in alternative cost selection)
    NormalCost,
    /// An alternative cost option
    AlternativeCost(AlternativeCost),
    /// An additional cost option
    AdditionalCost(AdditionalCost),
    /// A number (for X value ranges presented as discrete options)
    Number(u64),
    /// A color
    Color(Color),
    /// A counter type
    CounterType(CounterType),
    /// A mana type (for generic allocation)
    ManaType(ManaType),
}
