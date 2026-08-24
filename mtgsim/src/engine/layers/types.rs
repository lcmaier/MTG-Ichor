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
use crate::types::effects::{Duration, PermanentFilter, PlayerRef};
use crate::types::ids::{AbilityId, ObjectId, PlayerId};
use crate::types::keywords::KeywordFlag;
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
    /// Layer 7a — power/toughness from characteristic-defining abilities
    /// (CR 613.4a).
    ///
    /// **Nothing is ever registered into this layer.** CR 604.3a(3) says a CDA
    /// "does not directly affect the characteristics of any other objects", so
    /// a CDA always applies to exactly the object that has it — which means it
    /// needs no `AffectedSet`, no filter, and no registry row. `layers::cda`
    /// applies them straight off the object's own effective ability list.
    /// `ContinuousEffectRegistry::add` asserts this.
    ///
    /// The variant still has to exist because `LAYER_ORDER` is an array of
    /// `Layer` and the walk needs a slot here — after Layer 6, so that Humility
    /// strips the CDA before it would apply, and before 7b.
    Layer7aCdaPT,
    /// Layer 7b — effects that set P/T to specific values (CR 613.4b).
    Layer7bSetPT,
    /// Layer 7c — P/T modifications: +N/+N pumps, anthems (CR 613.4c).
    /// Counters are also applied here but read directly from BattlefieldEntity,
    /// not stored as registered effects.
    Layer7cModifyPT,
    /// Layer 7d — switch P/T (CR 613.4d).
    Layer7dSwitchPT,
}

/// One side of a power/toughness modification.
///
/// Two variants rather than one, because the two carry different things and
/// neither subsumes the other:
///
/// - `Fixed` is a **signed** literal. `ModifyPowerToughness { power: -1, .. }`
///   is ordinary (Weakness, `-1/-1` counters), and `AmountExpr::Fixed` is `u64`,
///   so the amount language cannot express it.
/// - `Dynamic` is an expression evaluated during the layer walk, every time, on
///   the frame as of the end of the previous layer. March of the Machines' "each
///   equal to its mana value" and Tarmogoyf's graveyard count are both this: the
///   value has to track the game rather than be snapshotted at registration,
///   which is what CR 604.7 and 613.4a require.
#[derive(Debug, Clone, PartialEq)]
pub enum PtValue {
    Fixed(i32),
    Dynamic(crate::types::effects::AmountExpr),
}

impl PtValue {
    /// Lower a card-definition amount into a P/T value.
    ///
    /// `Fixed` collapses to a literal so the common case stays a plain integer
    /// and never touches the evaluator; everything else is carried through as
    /// an expression and re-evaluated at every layer.
    pub fn from_amount(expr: &crate::types::effects::AmountExpr) -> Self {
        use crate::types::effects::AmountExpr;
        match expr {
            AmountExpr::Fixed(n) => PtValue::Fixed(*n as i32),
            other => PtValue::Dynamic(other.clone()),
        }
    }
}

/// What a continuous effect does to each affected object.
/// Each variant belongs to exactly one layer.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectModification {
    // --- Layer 2 ---
    /// CR 613.1b. Carries a `PlayerRef`, **not** a resolved `PlayerId`, and the
    /// distinction is the same one Deferred Migrations item 11 paid a redesign
    /// for on `AffectedSet::Filter`.
    ///
    /// A static ability's "you" is CR 109.5's *current* controller of the object
    /// the ability is on, so Mind Control's "You control enchanted creature"
    /// follows the Aura if the Aura itself changes hands. Storing the id at
    /// registration would snapshot whoever controlled the source at ETB — the
    /// exact bug `tests/filter_controller_test.rs` pins. `compute` resolves this
    /// during the walk through the same `FilterPlayers` that resolves a filter's
    /// `ByController`, so both halves of CR 109.5 have one implementation:
    /// `EffectOrigin::StaticAbility` asks the source, `EffectOrigin::Resolution`
    /// reads `ContinuousEffect.controller`, which CR 611.2c locked when the
    /// spell resolved.
    ///
    /// It also keeps `GameState::static_primitive_rows` a pure function of the
    /// primitive: that table cannot manufacture a `PlayerId`, so a `PlayerId`
    /// here would have left `Primitive::GainControl` in the `_ => Vec::new()`
    /// arm the loud-lowering work exists to empty.
    SetController(PlayerRef),

    // --- Layer 4 ---
    AddType(CardType),
    RemoveType(CardType),
    SetTypes(HashSet<CardType>),
    AddSubtype(Subtype),
    RemoveSubtype(Subtype),
    SetSubtypes(HashSet<Subtype>),
    AddSupertype(Supertype),
    RemoveSupertype(Supertype),
    SetSupertypes(HashSet<Supertype>),

    // --- Layer 5 ---
    AddColor(Color),
    SetColors(HashSet<Color>),
    RemoveAllColors,

    // --- Layer 6 ---
    // Two channels, because `EffectiveCharacteristics` genuinely has two
    // fields. `KeywordFlag` holds the CR 702 keywords whose whole meaning is
    // their presence; everything else — a keyword with a parameter, a keyword
    // with an ability body, or one-off granted text — is an `AbilityDef` and
    // goes through `GrantAbility`. See `KeywordFlag`'s docs for the map.
    GrantKeywordFlag(KeywordFlag),
    RemoveKeywordFlag(KeywordFlag),
    /// Boxed: `AbilityDef` carries a `Vec<Cost>` and an `Effect` tree, and this
    /// enum is stored per registry row and matched at every layer.
    GrantAbility(Box<AbilityDef>),
    /// CR 113.10b — removes *all* instances of the ability, not the first.
    LoseAbility(AbilityId),
    LoseAllAbilities,

    // --- Layer 7b ---
    SetPowerToughness { power: PtValue, toughness: PtValue },

    // --- Layer 7c ---
    ModifyPowerToughness { power: PtValue, toughness: PtValue },

    // --- Layer 7d ---
    SwitchPowerToughness,
}

/// Where a continuous effect came from, which determines whether its
/// existence is conditional (CR 613.7a vs. 613.7b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectOrigin {
    /// CR 613.7b — created by the resolution of a spell or ability. The effect
    /// exists unconditionally until its `Duration` ends; nothing can take away
    /// the ability that made it, because it was made by a one-shot resolution.
    Resolution,

    /// CR 613.7a — generated by a static ability of `source`. Existence is
    /// *conditional*: the effect only applies while `source` still has
    /// `ability`. CR 305.7 (Blood Moon) and Layer 6 ability removal can both
    /// take it away, so `compute.rs` re-checks this at every layer rather than
    /// trusting registry membership.
    ///
    /// `(source, ability)` is the identity pair — `AbilityId` names an ability
    /// *definition*, which two objects sharing an `Arc<CardData>` also share,
    /// so the object id is what makes it an ability *on an object*. Same
    /// pairing `mana_helpers::activatable_abilities` and
    /// `engine::mana::activate_mana_ability` already use.
    StaticAbility { ability: AbilityId },
}

/// Selects which objects a continuous effect applies to.
#[derive(Debug, Clone, PartialEq)]
pub enum AffectedSet {
    /// The source permanent itself ("this creature has flying").
    SourceOnly,
    /// A data-driven filter ("creatures you control").
    ///
    /// **The filter is stored unresolved.** `PermanentFilter::ByController`
    /// carries a `PlayerRef`, and `compute::effect_applies_to` resolves it
    /// during the layer walk against the source's *effective* controller.
    ///
    /// This used to carry a `controller: Option<PlayerId>` that
    /// `register_static_effects` filled in at ETB. CR 109.5 says the opposite —
    /// "for a static ability, [you] is the **current** controller of the object
    /// it's on" — so a snapshot taken when the source entered is wrong the
    /// moment control of the source changes, and Glorious Anthem kept buffing
    /// the team of whoever controlled it at ETB.
    Filter { filter: PermanentFilter },
    /// A fixed set captured at effect creation time.
    /// Pump spells use this — the target is locked at resolution.
    Fixed(Vec<ObjectId>),
}

/// CR-level identity of a continuous effect, for CR 613.6.
///
/// The registry stores one `ContinuousEffect` row per `EffectModification`,
/// because each modification belongs to exactly one layer. But CR 613.6 talks
/// about *the effect* — "if an effect starts to apply in one layer, it will
/// continue to be applied to the same set of objects in each other applicable
/// layer". March of the Machines is one effect with a Layer 4 part and a Layer
/// 7b part; in the registry that is two rows with two `EffectId`s. Rows that
/// belong to the same CR-level effect share an `EffectGroup`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectGroup {
    /// All rows generated by one static ability of one object. Keyed on the
    /// ability rather than the timestamp because `register_static_effects`
    /// allocates a separate timestamp per atom of a multi-atom ability.
    StaticAbility(ObjectId, AbilityId),
    /// All rows a single resolution registered together, which share a source
    /// and a timestamp (CR 613.7b).
    Resolution(ObjectId, Timestamp),
}

/// A single active continuous effect in the registry.
#[derive(Debug, Clone)]
pub struct ContinuousEffect {
    /// Unique ID for this effect instance.
    pub id: EffectId,
    /// The object that generates this effect.
    pub source: ObjectId,
    /// Whether this effect's existence is conditional on a static ability
    /// still being present (CR 613.7a) or unconditional (CR 613.7b).
    pub origin: EffectOrigin,
    /// Which layer this effect applies in.
    pub layer: Layer,
    /// When the effect becomes inactive (abstract — from card text).
    pub duration: Duration,
    /// The player who controlled the spell/ability that created this effect.
    /// Resolves "your" in durations like UntilYourNextTurn.
    pub controller: PlayerId,
    /// The turn number when this effect was created. Used with player-relative
    /// durations to prevent immediate expiry (e.g. "until your next turn"
    /// shouldn't expire on the same turn it was created).
    pub created_on_turn: u32,
    /// Timestamp for ordering within the same layer (CR 613.7).
    pub timestamp: Timestamp,
    /// Which objects this effect applies to.
    pub affected: AffectedSet,
    /// What the effect does to each affected object.
    pub modification: EffectModification,
}

impl ContinuousEffect {
    /// The CR-level effect this registry row belongs to. See `EffectGroup`.
    pub fn group(&self) -> EffectGroup {
        match self.origin {
            EffectOrigin::StaticAbility { ability } => {
                EffectGroup::StaticAbility(self.source, ability)
            }
            EffectOrigin::Resolution => EffectGroup::Resolution(self.source, self.timestamp),
        }
    }
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
    pub keywords: HashSet<KeywordFlag>,
    pub abilities: Vec<AbilityDef>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub controller: PlayerId,
    /// The turn `controller` took control (CR 302.6's "continuously since their
    /// most recent turn began"). Computed, never stored.
    ///
    /// Summoning sickness is the only reader, and it cannot use
    /// `BattlefieldEntity.controller_since_turn` once Layer 2 exists: control
    /// from a continuous effect is *derived*, so a `Duration::UntilEndOfTurn`
    /// steal reverts at cleanup with no mutation to hang a field update on and
    /// no event to hook. Deriving it alongside the controller it describes is
    /// what makes reversion need nothing at all — the value simply stops being
    /// computed when the row leaves the registry.
    ///
    /// Seeded from `BattlefieldEntity.controller_since_turn` (which still owns
    /// every control change that is *not* a Layer 2 effect — entering the
    /// battlefield, today the only one) and overwritten by the Layer 2 arm of
    /// `apply_modification`, but only when the controller actually changes.
    /// Act of Treason on your own creature must not make it summoning-sick.
    pub control_since_turn: u32,
}
