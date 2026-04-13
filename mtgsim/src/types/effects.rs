use super::colors::Color;
use super::ids::PlayerId;
use super::keywords::KeywordAbility;
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
    CardsInGraveyard(PlayerRef),
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
    pub keywords: Vec<KeywordAbility>,
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

/// Type change description for ChangeType primitive
#[derive(Debug, Clone, PartialEq)]
pub struct TypeChange {
    pub add_types: Vec<crate::types::card_types::CardType>,
    pub remove_types: Vec<crate::types::card_types::CardType>,
    pub add_subtypes: Vec<crate::types::card_types::Subtype>,
    pub remove_subtypes: Vec<crate::types::card_types::Subtype>,
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
    SetPowerToughness(AmountExpr, AmountExpr),
    /// Modify power/toughness by +X/+Y (layer 7c)
    ModifyPowerToughness(AmountExpr, AmountExpr),
    /// Grant a keyword ability (layer 6)
    AddAbility(KeywordAbility, Duration),
    /// Remove a keyword ability (layer 6)
    RemoveAbility(KeywordAbility, Duration),
    /// Change color (layer 5)
    ChangeColor(Color, Duration),
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
