//! Type definitions for the continuous effects / layer system (CR 613).
//!
//! These types represent the core data structures for tracking and applying
//! continuous effects in layer order. The layer system computes effective
//! characteristics for all game objects by walking registered effects in
//! layer + timestamp order.

use std::collections::HashSet;

use crate::objects::card_data::AbilityDef;
use crate::types::card_types::{CardType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{Duration, PermanentFilter};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::keywords::KeywordAbility;
use crate::types::mana::ManaCost;

/// Unique identifier for a registered continuous effect.
pub type EffectId = u64;

/// Timestamp for ordering effects within a layer (CR 613.7).
pub type Timestamp = u64;

/// CR 613 layers, including sublayers. The derived `Ord` gives correct
/// application order: effects are sorted by layer first, then by timestamp
/// within the same layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layer {
    /// Layer 1 — copy effects (CR 613.2). Face-down and copy.
    /// Stubbed for Phase LA; implemented in Phase LD.
    Layer1Copy,
    /// Layer 2 — control-changing effects (CR 613.3).
    Layer2Control,
    /// Layer 3 — text-changing effects. Deferred indefinitely (25 cards).
    Layer3Text,
    /// Layer 4 — type-changing effects (types, subtypes, supertypes).
    Layer4Type,
    /// Layer 5 — color-changing effects.
    Layer5Color,
    /// Layer 6 — ability-adding and ability-removing effects.
    Layer6Ability,
    /// Layer 7b — effects that set P/T to specific values (CR 613.4b).
    Layer7bSetPT,
    /// Layer 7c — P/T modifications: +N/+N pumps, anthems (CR 613.4c).
    /// Counters are also applied here but read directly from BattlefieldEntity,
    /// not stored as registered effects.
    Layer7cModifyPT,
    /// Layer 7d — switch P/T (CR 613.4d).
    Layer7dSwitchPT,
}

/// What a continuous effect does to each affected object.
/// Each variant belongs to exactly one layer.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectModification {
    // --- Layer 2 ---
    SetController(PlayerId),

    // --- Layer 4 ---
    AddType(CardType),
    RemoveType(CardType),
    SetTypes(HashSet<CardType>),
    AddSubtype(Subtype),
    RemoveSubtype(Subtype),
    SetSubtypes(HashSet<Subtype>),
    AddSupertype(Supertype),
    RemoveSupertype(Supertype),

    // --- Layer 5 ---
    AddColor(Color),
    SetColors(HashSet<Color>),
    RemoveAllColors,

    // --- Layer 6 ---
    GrantKeyword(KeywordAbility),
    RemoveKeyword(KeywordAbility),
    LoseAllAbilities,

    // --- Layer 7b ---
    SetPowerToughness { power: i32, toughness: i32 },

    // --- Layer 7c ---
    ModifyPowerToughness { power: i32, toughness: i32 },

    // --- Layer 7d ---
    SwitchPowerToughness,
}

/// Selects which objects a continuous effect applies to.
#[derive(Debug, Clone, PartialEq)]
pub enum AffectedSet {
    /// The source permanent itself ("this creature has flying").
    SourceOnly,
    /// A data-driven filter ("creatures you control").
    Filter {
        filter: PermanentFilter,
        controller: Option<PlayerId>,
    },
    /// A fixed set captured at effect creation time.
    /// Pump spells use this — the target is locked at resolution.
    Fixed(Vec<ObjectId>),
}

/// A single active continuous effect in the registry.
#[derive(Debug, Clone)]
pub struct ContinuousEffect {
    /// Unique ID for this effect instance.
    pub id: EffectId,
    /// The object that generates this effect.
    pub source: ObjectId,
    /// Which layer this effect applies in.
    pub layer: Layer,
    /// When the effect becomes inactive.
    pub duration: Duration,
    /// Timestamp for ordering within the same layer (CR 613.7).
    pub timestamp: Timestamp,
    /// Which objects this effect applies to.
    pub affected: AffectedSet,
    /// What the effect does to each affected object.
    pub modification: EffectModification,
}

/// The computed effective characteristics of a game object after all
/// continuous effects have been applied. This is the output of
/// `compute_characteristics`.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveCharacteristics {
    pub name: String,
    pub mana_cost: Option<ManaCost>,
    pub colors: HashSet<Color>,
    pub types: HashSet<CardType>,
    pub subtypes: HashSet<Subtype>,
    pub supertypes: HashSet<Supertype>,
    pub keywords: HashSet<KeywordAbility>,
    pub abilities: Vec<AbilityDef>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub controller: PlayerId,
}
