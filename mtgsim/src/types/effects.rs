use super::colors::Color;
use super::ids::PlayerId;
use super::keywords::KeywordFlag;
use super::mana::{ManaAtom, ManaType};

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// How numeric amounts are determined at resolution time (rule 608.2h)
#[derive(Debug, Clone, PartialEq)]
pub enum AmountExpr {
    /// A constant known at definition time
    Fixed(u64),
    /// X, chosen when the spell/ability is cast/activated (rule 107.3)
    Variable,
    /// "equal to the number of [things matching selector]"
    CountOf(Selector),
    /// "equal to the number of card **types** among [things matching selector]"
    /// — the Lhurgoyf family. Distinct from `CountOf`, which counts objects:
    /// ten artifact creatures in a graveyard are ten cards but two card types.
    CardTypesAmong(Selector),
    /// "equal to that number plus N" — Tarmogoyf's toughness.
    Plus(Box<AmountExpr>, u64),
    /// "equal to its mana value", where "it" is the object the continuous
    /// effect is being applied to — March of the Machines' "power and toughness
    /// each equal to its mana value".
    ///
    /// Read off the affected object's *effective* mana cost mid-layer-walk, not
    /// off `CardData`: a Layer 1 copy effect changes mana cost, and CR 202.3b
    /// makes an object with no mana cost mana value 0.
    ///
    /// Distinct from `TargetPower` and friends below, which are resolution-time
    /// and read the target of a spell. This one has meaning in a static context,
    /// which is what lets `compute.rs` evaluate it at every layer.
    AffectedManaValue,
    /// "equal to that creature's power"
    TargetPower,
    /// "equal to that creature's toughness"
    TargetToughness,
    /// "equal to the damage dealt this way"
    DamageDealt,
}

/// Which objects an effect queries or iterates over
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    ControlledCreatures,
    CreaturesInGraveyard(PlayerRef),
    PermanentsMatching(PermanentFilter),
    CardsInHand(PlayerRef),
    /// Cards in graveyards. `None` means **all** graveyards — Tarmogoyf's "cards
    /// in all graveyards", which includes a Tarmogoyf sitting in one of them.
    ///
    /// One variant rather than a separate `CardsInAllGraveyards`: "whose
    /// graveyard" is a parameter of the same concept, and a second variant made
    /// the two look like different questions. Only `None` has an evaluator today
    /// — every `Some` form is unused by any card, and `compute::evaluate_amount`
    /// asserts rather than guessing at `PlayerRef` resolution nothing needs yet.
    CardsInGraveyard(Option<PlayerRef>),
}

/// Reference to a player in an effect context
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerRef {
    /// The controller of the spell/ability
    You,
    /// A targeted or otherwise identified opponent
    Opponent,
    /// Owner of the source object
    Owner,
    /// A specific player
    Player(PlayerId),
}

/// Filter for matching permanents (extensible)
#[derive(Debug, Clone, PartialEq)]
pub enum PermanentFilter {
    All,
    ByType(crate::types::card_types::CardType),
    BySubtype(crate::types::card_types::Subtype),
    BySupertype(crate::types::card_types::Supertype),
    ByColor(Color),
    ByController(PlayerRef),
    /// Power less than or equal to N (for "creature with power N or less")
    PowerLE(i32),
    And(Box<PermanentFilter>, Box<PermanentFilter>),
    Not(Box<PermanentFilter>),
}

/// Filter for matching cards (extensible)
#[derive(Debug, Clone, PartialEq)]
pub enum CardFilter {
    All,
    ByType(crate::types::card_types::CardType),
    ByColor(Color),
}

/// Duration for continuous effects (rule 611)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duration {
    /// "until end of turn"
    UntilEndOfTurn,
    /// "until your next turn"
    UntilYourNextTurn,
    /// As long as the source permanent is on the battlefield (static abilities)
    WhileSourceOnBattlefield,
    /// As long as the permanent is enchanted by the source
    WhileEnchanted,
    /// As long as the permanent is equipped by the source
    WhileEquipped,
    /// Lasts until end of game (or until removed)
    Indefinite,
}

/// Conditions for Conditional effects (rule 603.4 intervening "if")
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    ControlPermanent(PermanentFilter),
    LifeAtLeast(AmountExpr),
    LifeAtMost(AmountExpr),
    OpponentControlsPermanent(PermanentFilter),
    CardInGraveyard(CardFilter),
    SpellWasKicked,
    ModeChosen(usize),
    SourceOnBattlefield,
}

/// How many modes to choose (rule 700.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalCount {
    Exactly(u32),
    UpTo(u32),
    Any,
}

/// What an effect acts on.
///
/// Separates two orthogonal concerns that were previously conflated:
/// - **Who/what** the effect acts on (filter + count)
/// - **Whether targeting rules apply** (hexproof/shroud/protection)
///
/// `Target` = the MTG rules concept of "targeting" (hexproof, shroud,
/// protection all apply; fizzles if target becomes illegal).
/// `Choose` = "choose" / non-targeting selection (rule 303.4a — Aura ETB
/// without casting; hexproof/shroud do NOT apply, does NOT fizzle).
#[derive(Debug, Clone, PartialEq)]
pub enum EffectRecipient {
    /// No object involved (e.g. mana abilities, "draw a card" with no target)
    Implicit,
    /// The controller of this spell/ability (e.g. Night's Whisper "you draw",
    /// Angel's Mercy "you gain"). Not targeting.
    Controller,
    /// Select with targeting rules — hexproof/shroud/protection apply,
    /// fizzles if all targets become illegal (rule 608.2b).
    Target(SelectionFilter, TargetCount),
    /// Select without targeting rules — "choose" (rule 303.4a, etc.).
    /// Hexproof/shroud/protection do NOT apply.  Does not fizzle.
    Choose(SelectionFilter, TargetCount),
    /// Filter-based continuous effect (static abilities / anthems).
    /// Applies to all permanents matching the filter. Not used at cast/resolution
    /// time — only read by the ETB hook to register continuous effects.
    /// Use `ByController(PlayerRef::You)` in the filter to express "you control";
    /// the registration hook resolves `PlayerRef` to a concrete `PlayerId`.
    FilteredPermanents(PermanentFilter),
}

/// What kind of object(s) can be selected.
///
/// Shared by both `Target` and `Choose` variants of `EffectRecipient`.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionFilter {
    /// Creature on the battlefield
    Creature,
    /// Player
    Player,
    /// "any target" — creature, player, or planeswalker
    Any,
    /// Permanent matching a filter
    Permanent(PermanentFilter),
    /// Spell on the stack
    Spell,
}

/// How many targets/choices to select
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetCount {
    Exactly(u32),
    UpTo(u32),
}

/// Mana output from a mana ability or mana-producing spell.
///
/// Dual-track, mirroring `ManaPool`:
/// - `mana`: fast-path unrestricted mana (added via `pool.add()`)
/// - `special`: sidecar atoms with restrictions, grants, or persistence
///   (added via `pool.add_special()`)
///
/// Amounts use `AmountExpr` so they can be evaluated at resolution time
/// (e.g. "Add {G} equal to target creature's power" → `TargetPower`).
/// Most cards use `AmountExpr::Fixed`.
///
/// Most cards only use `mana`. Cards like Cavern of Souls use `special`.
#[derive(Debug, Clone, PartialEq)]
pub struct ManaOutput {
    pub mana: Vec<(ManaType, AmountExpr)>,
    pub special: Vec<ManaAtom>,
}

/// Zone filter for Search effects
#[derive(Debug, Clone, PartialEq)]
pub enum ZoneFilter {
    Library,
    Graveyard,
    Exile,
}

/// Token definition for CreateToken
#[derive(Debug, Clone, PartialEq)]
pub struct TokenDef {
    pub name: String,
    pub colors: Vec<Color>,
    pub types: Vec<crate::types::card_types::CardType>,
    pub subtypes: Vec<crate::types::card_types::Subtype>,
    pub power: i32,
    pub toughness: i32,
    pub keywords: Vec<KeywordFlag>,
}

/// Counter types that can be placed on permanents/players
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CounterType {
    PlusOnePlusOne,
    MinusOneMinusOne,
    Loyalty,
    Charge,
    // Keyword counters (rule 122.1b)
    Flying,
    Deathtouch,
    Lifelink,
    Trample,
    FirstStrike,
    DoubleStrike,
    Hexproof,
    Indestructible,
    Menace,
    Reach,
    Vigilance,
    Haste,
    // Non-evergreen counter types added as relevant cards are implemented
}

impl CounterType {
    /// The keyword this counter grants (CR 122.1b), or `None` if it is not a
    /// keyword counter.
    ///
    /// > A keyword counter on a permanent or on a card in a zone other than the
    /// > battlefield causes that object to gain that keyword.
    ///
    /// CR 122.1b names fifteen keywords plus their variants. Twelve of them are
    /// `CounterType` variants today; the missing three are decayed, exalted and
    /// shadow. Decayed and exalted are not `KeywordFlag`s at all -- they are
    /// quadrant-3 keywords with ability bodies (CR 702.147, 702.83), so they
    /// will arrive as `AbilityDef`s and want a different bridge than this one.
    /// Shadow is a plain flag and just has no card needing it yet.
    ///
    /// Applied in Layer 6 by `compute::apply_effects`, read straight off
    /// `BattlefieldEntity::counters` rather than registered as a continuous
    /// effect -- same treatment as the +1/+1 counters in Layer 7c, and for the
    /// same reason: the state is already owned, and reconciling registry rows
    /// against every counter mutation is the pattern that turns effect
    /// existence into a fixpoint.
    pub fn keyword_granted(self) -> Option<crate::types::keywords::KeywordFlag> {
        use crate::types::keywords::KeywordFlag as K;
        Some(match self {
            CounterType::Flying => K::Flying,
            CounterType::Deathtouch => K::Deathtouch,
            CounterType::Lifelink => K::Lifelink,
            CounterType::Trample => K::Trample,
            CounterType::FirstStrike => K::FirstStrike,
            CounterType::DoubleStrike => K::DoubleStrike,
            CounterType::Hexproof => K::Hexproof,
            CounterType::Indestructible => K::Indestructible,
            CounterType::Menace => K::Menace,
            CounterType::Reach => K::Reach,
            CounterType::Vigilance => K::Vigilance,
            CounterType::Haste => K::Haste,
            CounterType::PlusOnePlusOne
            | CounterType::MinusOneMinusOne
            | CounterType::Loyalty
            | CounterType::Charge => return None,
        })
    }
}

/// Color change description for ChangeColor primitive (layer 5).
///
/// Three operations map 1:1 to the three `EffectModification` variants:
/// - `Add(Color)` → `AddColor(Color)` — adds a color without removing existing ones
/// - `Set(HashSet<Color>)` → `SetColors(HashSet<Color>)` — replaces all colors
/// - `RemoveAll` → `RemoveAllColors` — makes the object colorless
#[derive(Debug, Clone, PartialEq)]
pub enum ColorChange {
    /// Add a single color (e.g. "becomes blue in addition to its other colors")
    Add(Color),
    /// Set colors to exactly this set (e.g. "becomes red" = Set({Red}))
    Set(std::collections::HashSet<Color>),
    /// Remove all colors (e.g. "becomes colorless")
    RemoveAll,
}

/// Type change description for ChangeType primitive (layer 4).
///
/// Supports both additive/subtractive operations and overwrite ("set") operations.
/// When a `set_*` field is `Some`, it takes priority over the corresponding
/// add/remove fields. A single card effect can combine these
/// (e.g., "becomes an artifact creature" sets types while adding subtypes).
#[derive(Debug, Clone, PartialEq)]
pub struct TypeChange {
    pub add_types: Vec<crate::types::card_types::CardType>,
    pub remove_types: Vec<crate::types::card_types::CardType>,
    /// If Some, replaces all card types with this set (ignores add_types/remove_types).
    pub set_types: Option<std::collections::HashSet<crate::types::card_types::CardType>>,
    pub add_subtypes: Vec<crate::types::card_types::Subtype>,
    pub remove_subtypes: Vec<crate::types::card_types::Subtype>,
    /// If Some, replaces all subtypes with this set (ignores add_subtypes/remove_subtypes).
    pub set_subtypes: Option<std::collections::HashSet<crate::types::card_types::Subtype>>,
    pub add_supertypes: Vec<crate::types::card_types::Supertype>,
    pub remove_supertypes: Vec<crate::types::card_types::Supertype>,
    /// If Some, replaces all supertypes with this set (ignores add_supertypes/remove_supertypes).
    pub set_supertypes: Option<std::collections::HashSet<crate::types::card_types::Supertype>>,
}

// ---------------------------------------------------------------------------
// Primitives — atomic game actions (rule 610, 701)
// ---------------------------------------------------------------------------

/// What an effect does when it resolves (one-shot effects, rule 610).
///
/// Each variant is a single atomic game action. Complex effects are built
/// by combining primitives via the `Effect` combinator enum.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    // === Zone movement (rule 701) ===
    /// Destroy a permanent (rule 701.8) — respects indestructible/regenerate
    Destroy,
    /// Exile an object (rule 701.13)
    Exile,
    /// Sacrifice a permanent (rule 701.21)
    Sacrifice,
    /// Return to owner's hand ("bounce")
    ReturnToHand,
    /// Return to the battlefield (from exile/graveyard)
    ReturnToBattlefield,
    /// Put on top of owner's library
    PutOnTopOfLibrary,
    /// Put on bottom of owner's library
    PutOnBottomOfLibrary,
    /// Shuffle into owner's library
    ShuffleIntoLibrary,
    /// Mill N cards (rule 701.17)
    Mill(AmountExpr),
    /// Discard N cards (rule 701.9)
    Discard(AmountExpr),

    // === Damage & life ===
    /// Deal damage (rule 120)
    DealDamage(AmountExpr),
    /// Gain life
    GainLife(AmountExpr),
    /// Lose life
    LoseLife(AmountExpr),

    // === Card flow ===
    /// Draw N cards
    DrawCards(AmountExpr),
    /// Scry N (rule 701.22)
    Scry(AmountExpr),
    /// Surveil N (rule 701.25)
    Surveil(AmountExpr),

    // === Mana ===
    /// Produce mana (for mana abilities, rule 605)
    ProduceMana(ManaOutput),

    // === Counters ===
    /// Add N counters of a type to target
    AddCounters(CounterType, AmountExpr),
    /// Remove N counters of a type from target
    RemoveCounters(CounterType, AmountExpr),

    // === Tokens ===
    /// Create N tokens (rule 701.7)
    CreateToken(TokenDef, AmountExpr),

    // === Combat ===
    /// Two creatures fight (rule 701.14)
    Fight,
    /// Tap a permanent (rule 701.26)
    Tap,
    /// Untap a permanent (rule 701.26)
    Untap,

    // === Continuous effect primitives (applied via layer system) ===
    /// Set power/toughness to specific values (layer 7b)
    SetPowerToughness(AmountExpr, AmountExpr, Duration),
    /// Modify power/toughness by +X/+Y (layer 7c)
    ModifyPowerToughness(AmountExpr, AmountExpr, Duration),
    /// Switch power and toughness (layer 7d)
    SwitchPowerToughness(Duration),
    /// Grant a keyword flag (layer 6). See `KeywordFlag` for which keywords
    /// belong on this channel: only those whose meaning is their presence.
    /// A keyword with a parameter or an ability body goes through
    /// `GrantAbility` instead.
    GrantKeyword(KeywordFlag, Duration),
    /// Remove a keyword flag (layer 6). CR 113.10b — removing an ability
    /// removes all instances of it; a `HashSet` gives that structurally.
    ///
    /// Named `RemoveAbility` until the Layer 6 phase, which was simply wrong:
    /// it takes a `KeywordFlag`, never an ability.
    RemoveKeyword(KeywordFlag, Duration),
    /// Grant a whole ability (layer 6). The channel for every CR 702 keyword
    /// that is not a bare flag, and for one-off granted text.
    ///
    /// Boxed because `AbilityDef` contains an `Effect`, which contains
    /// `Primitive` — the recursion is real and needs an indirection. It also
    /// keeps `Primitive` small, since this variant is otherwise the largest.
    ///
    /// CR 604.3a(2): the granted def's `is_characteristic_defining` is cleared
    /// when it is applied, whatever the card author wrote. A granted ability is
    /// never a CDA.
    GrantAbility(Box<crate::objects::card_data::AbilityDef>, Duration),
    /// Remove one ability by id (layer 6). CR 113.10b — *all* instances of it.
    LoseAbility(crate::types::ids::AbilityId, Duration),
    /// Remove every ability and keyword (layer 6). Humility, Merfolk Trickster.
    LoseAllAbilities(Duration),
    /// Change color (layer 5)
    ChangeColor(ColorChange, Duration),
    /// Change types (layer 4)
    ChangeType(TypeChange, Duration),
    /// Gain control (layer 2)
    GainControl(Duration),

    // === Counter spells/abilities (rule 701.6) ===
    /// Counter a spell on the stack (rule 701.6a).
    /// The countered spell is moved to its owner's graveyard.
    CounterSpell,
    /// Counter an activated or triggered ability on the stack (rule 701.6b).
    /// The countered ability ceases to exist — it is simply removed from the stack.
    CounterAbility,
}

// ---------------------------------------------------------------------------
// Effect — the combinator layer
// ---------------------------------------------------------------------------

/// What an ability or spell does when it resolves.
///
/// Effects are composable: `Sequence` chains multiple effects,
/// `Conditional` gates on a condition, `Modal` offers choices, etc.
/// Each leaf is an `Atom` that applies a `Primitive` to targets.
///
/// **Continuous effects** (e.g. "+3/+3 until end of turn") are modeled as
/// an `Atom` containing a continuous `Primitive` (like `ModifyPowerToughness`)
/// that registers a modifier in the GameState. The layer system (rule 613)
/// reads these modifiers to compute effective characteristics.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Apply a primitive to resolved targets
    Atom(Primitive, EffectRecipient),

    /// Execute effects in order (e.g. "deal 3 damage and draw a card")
    Sequence(Vec<Effect>),

    /// "If [condition], [effect]" — intervening if (rule 603.4)
    Conditional(Condition, Box<Effect>),

    /// "You may [effect]" (rule 603.5)
    Optional(Box<Effect>),

    /// "Choose N mode(s):" (rule 700.2)
    Modal {
        count: ModalCount,
        modes: Vec<Effect>,
    },

    /// "For each [thing], [effect]"
    ForEach(Selector, Box<Effect>),

    /// "Do this N times"
    Repeat(AmountExpr, Box<Effect>),

    // Future phases:
    // ApplyContinuous(ContinuousEffectDef),
    // ApplyReplacement(ReplacementEffectDef),
    // ApplyPrevention(PreventionEffectDef),
    // CreateDelayedTrigger(TriggerCondition, Box<Effect>, Duration),
    // Custom(CardId),  // escape hatch
}
