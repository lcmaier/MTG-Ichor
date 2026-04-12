use std::collections::{HashMap, HashSet};
use std::fmt;

use super::colors::Color;
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::ids::ObjectId;

/// Types of mana that can exist in a mana pool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManaType {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}

impl ManaType {
    /// Convert a Color to its corresponding ManaType
    pub fn from_color(color: Color) -> Self {
        match color {
            Color::White => ManaType::White,
            Color::Blue => ManaType::Blue,
            Color::Black => ManaType::Black,
            Color::Red => ManaType::Red,
            Color::Green => ManaType::Green,
        }
    }
}

/// A single mana symbol in a mana cost.
///
/// Each symbol represents one "slot" that must be paid. The variant describes
/// what types of mana (or alternative payments) can satisfy it.
///
/// Reference: rules 107.4, 107.4a–107.4h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManaSymbol {
    /// A single colored mana symbol: {W}, {U}, {B}, {R}, {G}
    Colored(ManaType),
    /// Generic mana: {1}. Each instance represents one generic mana.
    /// A cost like {3} is stored as three `Generic` symbols.
    Generic,
    /// Colorless mana: {C}. Must be paid with colorless mana specifically.
    Colorless,
    /// Hybrid mana: {W/U}, {B/R}, etc. Pay with either color.
    Hybrid(ManaType, ManaType),
    /// Mono-hybrid (a.k.a. "twobrid"): {2/W}. Pay 2 generic or 1 of the color.
    MonoHybrid(ManaType),
    /// Phyrexian mana: {W/P}. Pay with the color or 2 life.
    Phyrexian(ManaType),
    /// Hybrid Phyrexian: {W/U/P}. Pay with either color or 2 life.
    HybridPhyrexian(ManaType, ManaType),
    /// Snow mana: {S}. Must be paid with mana from a snow source.
    Snow,
    /// X cost: variable. Mana value contribution is 0 when printed,
    /// but X is chosen on cast and contributes to the total cost.
    X,
}

impl ManaSymbol {
    /// Mana value contribution of this symbol (rule 202.3).
    /// X contributes 0 to mana value. MonoHybrid contributes 2.
    pub fn mana_value(&self) -> u8 {
        match self {
            ManaSymbol::Colored(_) => 1,
            ManaSymbol::Generic => 1,
            ManaSymbol::Colorless => 1,
            ManaSymbol::Hybrid(_, _) => 1,
            ManaSymbol::MonoHybrid(_) => 2,
            ManaSymbol::Phyrexian(_) => 1,
            ManaSymbol::HybridPhyrexian(_, _) => 1,
            ManaSymbol::Snow => 1,
            ManaSymbol::X => 0,
        }
    }
}

impl fmt::Display for ManaSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManaSymbol::Colored(t) => write!(f, "{{{}}}", mana_type_letter(*t)),
            ManaSymbol::Generic => write!(f, "{{1}}"),
            ManaSymbol::Colorless => write!(f, "{{C}}"),
            ManaSymbol::Hybrid(a, b) => write!(f, "{{{}/{}}}", mana_type_letter(*a), mana_type_letter(*b)),
            ManaSymbol::MonoHybrid(t) => write!(f, "{{2/{}}}", mana_type_letter(*t)),
            ManaSymbol::Phyrexian(t) => write!(f, "{{{}/P}}", mana_type_letter(*t)),
            ManaSymbol::HybridPhyrexian(a, b) => write!(f, "{{{}/{}/P}}", mana_type_letter(*a), mana_type_letter(*b)),
            ManaSymbol::Snow => write!(f, "{{S}}"),
            ManaSymbol::X => write!(f, "{{X}}"),
        }
    }
}

fn mana_type_letter(t: ManaType) -> &'static str {
    match t {
        ManaType::White => "W",
        ManaType::Blue => "U",
        ManaType::Black => "B",
        ManaType::Red => "R",
        ManaType::Green => "G",
        ManaType::Colorless => "C",
    }
}

/// Represents the mana cost printed on a card.
///
/// Stored as an ordered sequence of mana symbols. This design naturally
/// supports hybrid, Phyrexian, mono-hybrid, snow, and X costs.
///
/// The symbols are stored in conventional MTG ordering:
/// generic/X first, then colored symbols in WUBRG order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManaCost {
    pub symbols: Vec<ManaSymbol>,
}

impl ManaCost {
    /// A zero mana cost (no symbols).
    pub fn zero() -> Self {
        ManaCost { symbols: Vec::new() }
    }

    /// Build a cost from raw symbols.
    pub fn from_symbols(symbols: Vec<ManaSymbol>) -> Self {
        ManaCost { symbols }
    }

    /// Build a mana cost from colored pips and a generic count.
    ///
    /// This is the canonical constructor for mana costs.
    /// Symbols are stored in conventional MTG ordering: generic first,
    /// then colored in the order provided (caller should use WUBRG).
    ///
    /// e.g. `ManaCost::build(&[ManaType::Red], 0)` = {R}
    /// e.g. `ManaCost::build(&[ManaType::Green], 1)` = {1}{G}
    /// e.g. `ManaCost::build(&[ManaType::Blue, ManaType::Black], 2)` = {2}{U}{B}
    /// e.g. `ManaCost::build(&[ManaType::Red, ManaType::Red], 0)` = {R}{R}
    pub fn build(colored: &[ManaType], generic: u8) -> Self {
        let mut symbols = Vec::with_capacity(generic as usize + colored.len());
        for _ in 0..generic {
            symbols.push(ManaSymbol::Generic);
        }
        for &t in colored {
            symbols.push(if t == ManaType::Colorless {
                ManaSymbol::Colorless
            } else {
                ManaSymbol::Colored(t)
            });
        }
        ManaCost { symbols }
    }

    /// Total mana value / converted mana cost (rule 202.3).
    pub fn mana_value(&self) -> u8 {
        self.symbols.iter().map(|s| s.mana_value()).sum()
    }

    /// Count how many symbols of a specific colored type appear.
    pub fn colored_count(&self, mana_type: ManaType) -> u8 {
        self.symbols.iter().filter(|s| matches!(s, ManaSymbol::Colored(t) if *t == mana_type)).count() as u8
    }

    /// Count how many generic symbols appear.
    pub fn generic_count(&self) -> u8 {
        self.symbols.iter().filter(|s| matches!(s, ManaSymbol::Generic)).count() as u8
    }

    /// Count how many X symbols appear.
    pub fn x_count(&self) -> u8 {
        self.symbols.iter().filter(|s| matches!(s, ManaSymbol::X)).count() as u8
    }

    /// Whether any symbol requires player choice (hybrid, Phyrexian, X, generic).
    pub fn has_choices(&self) -> bool {
        self.symbols.iter().any(|s| matches!(s,
            ManaSymbol::Generic
            | ManaSymbol::Hybrid(_, _)
            | ManaSymbol::MonoHybrid(_)
            | ManaSymbol::Phyrexian(_)
            | ManaSymbol::HybridPhyrexian(_, _)
            | ManaSymbol::X
        ))
    }
}

impl fmt::Display for ManaCost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Coalesce consecutive Generic symbols into a single number: {3} not {1}{1}{1}
        let mut i = 0;
        while i < self.symbols.len() {
            if self.symbols[i] == ManaSymbol::Generic {
                let mut count = 0u8;
                while i < self.symbols.len() && self.symbols[i] == ManaSymbol::Generic {
                    count += 1;
                    i += 1;
                }
                write!(f, "{{{}}}", count)?;
            } else {
                write!(f, "{}", self.symbols[i])?;
                i += 1;
            }
        }
        Ok(())
    }
}

// =============================================================================
// Mana restriction, grant, and persistence types (rule 106.6).
//
// These types model per-unit mana metadata and are wired into ManaPool via
// the dual-track sidecar (T12b). See ManaPool doc comment for architecture.
//
// Engine integration (T12c) will update cast.rs, costs.rs, and mana.rs to
// route through can_pay_with_context / pay_with_plan with a SpendContext.
// =============================================================================

/// Spending restrictions on a single unit of mana (rule 106.6).
///
/// Examples:
/// - "Spend this mana only to cast creature spells" (Gwenna, Cavern of Souls)
/// - "Spend this mana only to cast instant or sorcery spells" (Boseiju)
/// - "Spend this mana only to cast artifact spells" (Mishra's Workshop)
/// - "Spend this mana only to cast colorless Eldrazi spells or activate
///   abilities of Eldrazi" (Eldrazi Temple) → `AnyOf`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManaRestriction {
    /// Mana can only be spent to cast spells with at least one of these types.
    /// OR semantics: spell matches if it has ANY listed type.
    OnlyForSpellTypes(Vec<CardType>),

    /// Mana can only be spent to activate abilities of permanents with at least
    /// one of these types.
    OnlyForAbilityTypes(Vec<CardType>),

    /// Compound OR: satisfies if ANY inner restriction matches.
    /// Models "cast Eldrazi spells or activate abilities of Eldrazi."
    AnyOf(Vec<ManaRestriction>),

    /// Mana can only be spent to cast creature spells of a specific creature type.
    /// (Cavern of Souls' "chosen type" restriction.)
    OnlyForCreatureType(CreatureType),
}

impl ManaRestriction {
    /// Check if this restriction allows spending in the given context.
    fn allows(&self, ctx: &SpendContext) -> bool {
        match self {
            ManaRestriction::OnlyForSpellTypes(types) => {
                match &ctx.purpose {
                    SpendPurpose::CastSpell { card_types, .. } => {
                        types.iter().any(|t| card_types.contains(t))
                    }
                    _ => false,
                }
            }
            ManaRestriction::OnlyForAbilityTypes(types) => {
                match &ctx.purpose {
                    SpendPurpose::ActivateAbility { source_card_types, .. } => {
                        types.iter().any(|t| source_card_types.contains(t))
                    }
                    _ => false,
                }
            }
            ManaRestriction::AnyOf(inner) => {
                inner.iter().any(|r| r.allows(ctx))
            }
            ManaRestriction::OnlyForCreatureType(ct) => {
                match &ctx.purpose {
                    SpendPurpose::CastSpell { card_types, subtypes, .. } => {
                        card_types.contains(&CardType::Creature)
                            && subtypes.contains(&Subtype::Creature(*ct))
                    }
                    _ => false,
                }
            }
        }
    }
}

/// Additional effects granted when this mana is spent (rule 106.6).
///
/// Tracked per-atom so the engine knows to apply them when the mana is
/// consumed during spell casting or ability activation.
///
/// Wired into ManaPool via `pay_with_plan` / `drain_spent_grants`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManaGrant {
    /// The spell cast using this mana gains a keyword
    /// (e.g. haste from Arena of Glory, uncounterable from Cavern of Souls)
    GrantKeyword(crate::types::keywords::KeywordAbility),
    // Future: TriggerOnSpend { ... }
}

/// How long mana persists in the pool before being emptied.
///
/// Two fundamentally different persistence patterns exist in Magic:
///
/// **Time-gated persistence (per-atom, lives in `special`):**
/// - Birgi: "Until end of turn, you don't lose this mana as steps and phases end."
///   → `UntilEndOf(PersistenceExpiry::EndOfTurn)`
/// - Firebending: "Until end of combat, you don't lose this mana..."
///   → `UntilEndOf(PersistenceExpiry::EndOfCombat)`
///
/// **Blanket persistence (continuous effect, NOT per-atom):**
/// - Omnath, Locus of Mana: "Green mana doesn't empty from your mana pool..."
///   → Modeled via `BlanketPersistenceSet`, not per-atom metadata.
/// - Upwelling: "Mana pools don't empty as steps and phases end."
///   → Also `BlanketPersistenceSet`.
///
/// There is no `Indefinite` variant. Every "mana doesn't empty" effect is either:
/// - A blanket static ability (Omnath, Upwelling) → `BlanketPersistenceSet`
/// - A replacement effect (Kruphix, Horizon Stone) → Phase 6 replacement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManaPersistence {
    /// Normal mana — empties at every step/phase transition (rule 106.4)
    Normal,
    /// Persists until a specific game point, then empties.
    /// Produced by effects like Firebending ("until end of combat")
    /// or Birgi ("until end of turn").
    /// Lives in `special` sidecar.
    UntilEndOf(PersistenceExpiry),
}

/// When time-gated persistent mana expires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistenceExpiry {
    EndOfCombat,
    EndOfTurn,
    EndOfPhase,
}

impl PersistenceExpiry {
    /// Whether this expiry matches the given empty reason (i.e., the mana
    /// should be removed).
    pub fn matches(&self, reason: &ManaEmptyReason) -> bool {
        match (self, reason) {
            // EndOfCombat expires at StepOrPhase (end-of-combat step is a step)
            (PersistenceExpiry::EndOfCombat, ManaEmptyReason::StepOrPhase) => true,
            (PersistenceExpiry::EndOfCombat, ManaEmptyReason::TurnEnd) => true,
            // EndOfPhase expires at StepOrPhase
            (PersistenceExpiry::EndOfPhase, ManaEmptyReason::StepOrPhase) => true,
            (PersistenceExpiry::EndOfPhase, ManaEmptyReason::TurnEnd) => true,
            // EndOfTurn expires only at TurnEnd
            (PersistenceExpiry::EndOfTurn, ManaEmptyReason::StepOrPhase) => false,
            (PersistenceExpiry::EndOfTurn, ManaEmptyReason::TurnEnd) => true,
        }
    }
}

/// Why the mana pool is being emptied. Determines which time-gated atoms expire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManaEmptyReason {
    /// Step or phase transition (rule 106.4). Time-gated atoms with
    /// `EndOfTurn` persistence survive this.
    StepOrPhase,
    /// Turn end (cleanup step). All non-blanket-protected mana empties.
    TurnEnd,
}

/// Describes which mana types are protected from emptying by blanket
/// continuous effects (Omnath, Upwelling, etc.).
///
/// Built by the continuous effects layer each time the pool is emptied.
#[derive(Debug, Clone)]
pub struct BlanketPersistenceSet {
    /// If true, ALL mana persists (Upwelling).
    pub all: bool,
    /// Specific mana types that persist (Omnath = {Green}).
    pub types: HashSet<ManaType>,
}

impl BlanketPersistenceSet {
    /// Whether this mana type is protected from emptying by a blanket effect.
    pub fn persists(&self, mana_type: ManaType) -> bool {
        self.all || self.types.contains(&mana_type)
    }

    /// No blanket persistence active — all mana empties normally.
    pub fn none() -> Self {
        BlanketPersistenceSet { all: false, types: HashSet::new() }
    }
}

/// Context describing what the mana is being spent on.
/// Used by the payment pipeline to evaluate ManaRestriction.
#[derive(Debug)]
pub struct SpendContext<'a> {
    /// The spell or ability being paid for.
    pub purpose: SpendPurpose<'a>,
}

/// What kind of action the mana is paying for.
#[derive(Debug)]
pub enum SpendPurpose<'a> {
    /// Casting a spell. Provides the spell's characteristics for restriction checks.
    CastSpell {
        card_types: &'a HashSet<CardType>,
        subtypes: &'a HashSet<Subtype>,
        name: &'a str,
    },
    /// Activating an ability on a permanent.
    ActivateAbility {
        source_card_types: &'a HashSet<CardType>,
        source_subtypes: &'a HashSet<Subtype>,
    },
    /// Paying a special action cost (e.g., morph). No restrictions typically apply.
    SpecialAction,
}

/// A fully specified plan for paying a mana cost.
///
/// Built by the DecisionProvider (or auto-builder), validated and executed
/// by ManaPool::pay_with_plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManaPaymentPlan {
    /// Mana to draw from the unrestricted simple pool.
    /// Maps ManaType → amount.
    pub from_simple: HashMap<ManaType, u64>,

    /// Mana to draw from specific special groups.
    /// Each entry is (group_index, amount_to_spend).
    /// `pay_with_plan` decrements the group's count and removes it if
    /// the count reaches zero.
    pub from_special: Vec<(usize, u64)>,
}

impl ManaPaymentPlan {
    /// Convenience constructor for plans that only use the simple pool.
    /// Wraps a generic allocation HashMap into a plan with no special spending.
    pub fn simple_only(generic_allocation: HashMap<ManaType, u64>) -> Self {
        ManaPaymentPlan {
            from_simple: generic_allocation,
            from_special: Vec::new(),
        }
    }
}

/// A single unit of mana with metadata about its source, restrictions, and
/// persistence.
///
/// Stored in `ManaPool::special` as counted groups: identical atoms are
/// coalesced into a single `(ManaAtom, u64)` entry. This avoids Vec bloat
/// if a loop or repeated effect produces many identical restricted mana units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManaAtom {
    pub mana_type: ManaType,
    pub source_id: Option<ObjectId>,
    pub restrictions: Vec<ManaRestriction>,
    pub grants: Vec<ManaGrant>,
    pub persistence: ManaPersistence,
}

impl ManaAtom {
    /// Create a simple unrestricted mana atom
    pub fn simple(mana_type: ManaType, source_id: Option<ObjectId>) -> Self {
        ManaAtom {
            mana_type,
            source_id,
            restrictions: Vec::new(),
            grants: Vec::new(),
            persistence: ManaPersistence::Normal,
        }
    }

    /// Internal helper — checks all restrictions on this atom.
    /// An atom with no restrictions always allows spending.
    fn allows_spend(&self, ctx: &SpendContext) -> bool {
        self.restrictions.iter().all(|r| r.allows(ctx))
    }
}

/// A player's mana pool — tracks available mana by type.
///
/// Dual-track structure:
/// - `pool` (simple): `HashMap<ManaType, u64>` for O(1) lookups. Handles the
///   99% case of unrestricted mana with no per-unit metadata.
/// - `special`: `Vec<(ManaAtom, u64)>` counted groups for mana with restrictions,
///   grants, or time-gated persistence. Only populated when special lands/abilities
///   produce restricted mana.
///
/// **Invariant:** A unit of mana lives in exactly one of `pool` or `special`,
/// never both. Unrestricted mana without per-unit grants goes into `pool`.
/// Mana with restrictions, grants, or time-gated persistence goes into `special`.
///
/// **Generic mana payment is a player choice.** The `pay()` method handles
/// only specific color requirements automatically (since those MUST be paid
/// with that exact color). Generic costs require the caller to provide an
/// explicit `generic_allocation` specifying which types of mana to use,
/// because the player may want to save specific colors for later spells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManaPool {
    pool: HashMap<ManaType, u64>,

    /// Slow path: mana with per-unit metadata (restrictions, grants, or
    /// time-gated persistence). Stored as counted groups: identical atoms
    /// are coalesced into a single `(ManaAtom, u64)` entry.
    special: Vec<(ManaAtom, u64)>,

    /// Grants collected from special atoms that were spent in the last
    /// `pay_with_plan` call. Drained by `drain_spent_grants()`.
    last_spent_grants: Vec<ManaGrant>,
}

impl ManaPool {
    pub fn new() -> Self {
        ManaPool {
            pool: HashMap::new(),
            special: Vec::new(),
            last_spent_grants: Vec::new(),
        }
    }

    pub fn add(&mut self, mana_type: ManaType, amount: u64) {
        *self.pool.entry(mana_type).or_insert(0) += amount;
    }

    pub fn amount(&self, mana_type: ManaType) -> u64 {
        *self.pool.get(&mana_type).unwrap_or(&0)
    }

    pub fn has(&self, mana_type: ManaType, amount: u64) -> bool {
        self.amount(mana_type) >= amount
    }

    /// Remove mana from the pool. Returns Err if insufficient.
    pub fn remove(&mut self, mana_type: ManaType, amount: u64) -> Result<(), String> {
        let current = self.amount(mana_type);
        if current < amount {
            return Err(format!(
                "Cannot remove {} {:?} mana, only {} available",
                amount, mana_type, current
            ));
        }
        self.pool.insert(mana_type, current - amount);
        Ok(())
    }

    /// Total mana of any type available
    pub fn total(&self) -> u64 {
        self.pool.values().sum()
    }

    /// Empty the pool, respecting both per-atom persistence and blanket
    /// continuous effects.
    ///
    /// - Simple pool: retain mana types covered by blanket persistence.
    /// - Special pool: check per-atom persistence AND blanket. A time-gated
    ///   atom (e.g. `EndOfTurn`) survives `StepOrPhase` even without blanket
    ///   protection. A normal atom in special (has restrictions/grants but no
    ///   persistence) survives only if blanket protects its type.
    pub fn empty_with_reason(
        &mut self,
        reason: ManaEmptyReason,
        blanket_persist: &BlanketPersistenceSet,
    ) {
        // Simple pool: zero out mana types not covered by blanket persistence.
        // We leave zero-valued entries in the map — they're harmless (the map
        // has at most 6 keys) and add() recreates them via or_insert(0).
        for (_mana_type, amount) in self.pool.iter_mut() {
            if !blanket_persist.persists(*_mana_type) {
                *amount = 0;
            }
        }

        // Special pool: check per-atom persistence + blanket
        self.special.retain(|(atom, _count)| {
            match &atom.persistence {
                ManaPersistence::Normal => {
                    // Normal atom in special (has restrictions/grants but
                    // no persistence). Survives only if blanket protects it.
                    blanket_persist.persists(atom.mana_type)
                }
                ManaPersistence::UntilEndOf(expiry) => {
                    // Time-gated atom. Survives unless this empty reason
                    // matches its expiry.
                    !expiry.matches(&reason)
                }
            }
        });
    }

    /// Get a snapshot of available mana by type
    pub fn available(&self) -> &HashMap<ManaType, u64> {
        &self.pool
    }

    // =========================================================================
    // Special mana sidecar methods
    // =========================================================================

    /// Add special mana (restricted, granted, or persistent).
    ///
    /// Coalesces identical atoms into existing groups: if an atom with the
    /// same type, restrictions, grants, and persistence already exists,
    /// its count is incremented instead of creating a new group.
    pub fn add_special(&mut self, atom: ManaAtom) {
        // Linear scan for an identical group to coalesce into
        for (existing, count) in self.special.iter_mut() {
            if *existing == atom {
                *count += 1;
                return;
            }
        }
        // No matching group — create a new one
        self.special.push((atom, 1));
    }

    /// Whether any special mana exists in the pool.
    /// Used to short-circuit: if false, all existing callers can use the
    /// fast path unchanged.
    pub fn has_special(&self) -> bool {
        !self.special.is_empty()
    }

    /// Iterate over special atom groups (for UI display, DP queries).
    pub fn special_atoms(&self) -> &[(ManaAtom, u64)] {
        &self.special
    }

    /// Collect all grants from atoms that were spent in the last payment.
    /// Called after pay_with_plan to apply grants (e.g., uncounterable).
    /// Returns the grants and clears the internal "last spent" buffer.
    pub fn drain_spent_grants(&mut self) -> Vec<ManaGrant> {
        std::mem::take(&mut self.last_spent_grants)
    }

    /// Total mana of a given type available for a specific spend context.
    /// Counts unrestricted (simple) + eligible special atoms.
    pub fn amount_for(&self, mana_type: ManaType, ctx: &SpendContext) -> u64 {
        let simple = self.amount(mana_type);
        let special: u64 = self.special.iter()
            .filter(|(atom, _)| atom.mana_type == mana_type && atom.allows_spend(ctx))
            .map(|(_, count)| *count)
            .sum();
        simple + special
    }

    /// Total mana of any type available for a specific spend context.
    pub fn total_for(&self, ctx: &SpendContext) -> u64 {
        let simple: u64 = self.pool.values().sum();
        let special: u64 = self.special.iter()
            .filter(|(atom, _)| atom.allows_spend(ctx))
            .map(|(_, count)| *count)
            .sum();
        simple + special
    }

    /// Check if a ManaCost can be paid, considering restrictions.
    /// This is the restriction-aware version of `can_pay`.
    ///
    /// Counts eligible special atoms alongside simple pool mana. An atom
    /// is eligible if `atom.allows_spend(ctx)` passes.
    pub fn can_pay_with_context(&self, cost: &ManaCost, ctx: &SpendContext) -> bool {
        // Tally up specific color requirements
        let mut need: HashMap<ManaType, u64> = HashMap::new();
        let mut generic_count: u64 = 0;

        for sym in &cost.symbols {
            match sym {
                ManaSymbol::Colored(t) => *need.entry(*t).or_insert(0) += 1,
                ManaSymbol::Colorless => *need.entry(ManaType::Colorless).or_insert(0) += 1,
                ManaSymbol::Generic => generic_count += 1,
                _ => return false,
            }
        }

        // Check each specific color requirement using context-aware amounts
        for (&mana_type, &required) in &need {
            if self.amount_for(mana_type, ctx) < required {
                return false;
            }
        }

        // Check that remaining mana covers generic
        let specific_total: u64 = need.values().sum();
        let total_available = self.total_for(ctx);
        let remaining = total_available.saturating_sub(specific_total);
        remaining >= generic_count
    }

    /// Pay a ManaCost using a ManaPaymentPlan that specifies exactly which
    /// atoms/pool units to spend.
    ///
    /// Validates the plan, then executes it atomically. If any step fails,
    /// the pool may be in a partially-modified state (caller should not
    /// retry — this indicates a bug in plan construction).
    ///
    /// After successful payment, spent special atoms' grants are collected
    /// into `last_spent_grants`. Call `drain_spent_grants()` to retrieve them.
    pub fn pay_with_plan(&mut self, plan: &ManaPaymentPlan) -> Result<(), String> {
        // Validate simple pool amounts
        for (&mana_type, &amount) in &plan.from_simple {
            if amount == 0 { continue; }
            let available = self.amount(mana_type);
            if available < amount {
                return Err(format!(
                    "pay_with_plan: need {} {:?} from simple but only {} available",
                    amount, mana_type, available
                ));
            }
        }

        // Validate special group indices and amounts
        for &(group_idx, spend_count) in &plan.from_special {
            if group_idx >= self.special.len() {
                return Err(format!(
                    "pay_with_plan: special group index {} out of range (have {} groups)",
                    group_idx, self.special.len()
                ));
            }
            let (_, available) = &self.special[group_idx];
            if spend_count > *available {
                return Err(format!(
                    "pay_with_plan: need {} from special group {} but only {} available",
                    spend_count, group_idx, available
                ));
            }
        }

        // Clear last spent grants
        self.last_spent_grants.clear();

        // Execute: deduct from simple
        for (&mana_type, &amount) in &plan.from_simple {
            if amount > 0 {
                self.remove(mana_type, amount)?;
            }
        }

        // Execute: deduct from special groups and collect grants
        // Collect grants into a local vec to avoid double-mutable-borrow of self
        let mut collected_grants = Vec::new();
        for &(group_idx, spend_count) in &plan.from_special {
            if spend_count == 0 { continue; }
            let (atom, count) = &mut self.special[group_idx];
            // Collect grants proportionally (one set of grants per atom spent)
            for _ in 0..spend_count {
                collected_grants.extend(atom.grants.iter().cloned());
            }
            *count -= spend_count;
        }
        self.last_spent_grants.extend(collected_grants);

        // Remove zero-count groups
        self.special.retain(|(_, count)| *count > 0);

        Ok(())
    }

    /// Check if a ManaCost can be paid from this pool.
    /// Does NOT modify the pool.
    ///
    /// Currently handles `Colored`, `Colorless`, and `Generic` symbols.
    /// Hybrid/Phyrexian/X/Snow payment requires `DecisionProvider` choices
    /// and will be handled via the full `pay()` path in a future phase.
    pub fn can_pay(&self, cost: &ManaCost) -> bool {
        // Tally up specific color requirements
        let mut need: HashMap<ManaType, u64> = HashMap::new();
        let mut generic_count: u64 = 0;

        for sym in &cost.symbols {
            match sym {
                ManaSymbol::Colored(t) => *need.entry(*t).or_insert(0) += 1,
                ManaSymbol::Colorless => *need.entry(ManaType::Colorless).or_insert(0) += 1,
                ManaSymbol::Generic => generic_count += 1,
                // Future: hybrid/phyrexian would need choice-aware checking
                _ => return false, // can't auto-check these yet
            }
        }

        // Check each specific color requirement
        for (&mana_type, &required) in &need {
            if !self.has(mana_type, required) {
                return false;
            }
        }

        // Check that remaining mana covers generic
        let specific_total: u64 = need.values().sum();
        let remaining = self.total().saturating_sub(specific_total);
        remaining >= generic_count
    }

    /// Pay a ManaCost from this pool.
    ///
    /// Specific color/colorless requirements are paid automatically.
    ///
    /// Generic costs require the caller to provide `generic_allocation`: a
    /// HashMap specifying how many of each ManaType to spend on the generic
    /// portion. The values must sum to `cost.generic_count()`. This is a
    /// player choice because the player may want to preserve specific colors.
    ///
    /// Returns Err if the pool has insufficient mana or the allocation is invalid.
    pub fn pay(
        &mut self,
        cost: &ManaCost,
        generic_allocation: &HashMap<ManaType, u64>,
    ) -> Result<(), String> {
        if !self.can_pay(cost) {
            return Err("Insufficient mana to pay cost".to_string());
        }

        // Tally specific requirements
        let mut need: HashMap<ManaType, u64> = HashMap::new();
        let mut generic_need: u64 = 0;

        for sym in &cost.symbols {
            match sym {
                ManaSymbol::Colored(t) => *need.entry(*t).or_insert(0) += 1,
                ManaSymbol::Colorless => *need.entry(ManaType::Colorless).or_insert(0) += 1,
                ManaSymbol::Generic => generic_need += 1,
                _ => return Err(format!("Cannot pay symbol {:?} yet", sym)),
            }
        }

        // Validate generic allocation sums to generic cost
        let alloc_total: u64 = generic_allocation.values().sum();
        if alloc_total != generic_need {
            return Err(format!(
                "Generic allocation sums to {} but generic cost is {}",
                alloc_total, generic_need
            ));
        }

        // Validate the allocation doesn't exceed what remains after specific costs
        for (&mana_type, &alloc_amount) in generic_allocation {
            let specific_need = need.get(&mana_type).copied().unwrap_or(0);
            let available = self.amount(mana_type);
            if specific_need + alloc_amount > available {
                return Err(format!(
                    "Cannot spend {} {:?} on generic + {} specific — only {} available",
                    alloc_amount, mana_type, specific_need, available
                ));
            }
        }

        // Pay specific colors
        for (&mana_type, &required) in &need {
            self.remove(mana_type, required)?;
        }

        // Pay generic using the player's chosen allocation
        for (&mana_type, &alloc_amount) in generic_allocation {
            if alloc_amount > 0 {
                self.remove(mana_type, alloc_amount)?;
            }
        }

        Ok(())
    }

    /// Pay a ManaCost that has no generic component.
    ///
    /// Convenience method for costs where all mana is specific colors (e.g. {W},
    /// {G}{G}, {R}). Errors if the cost has a generic component — use `pay()` instead.
    pub fn pay_specific_only(&mut self, cost: &ManaCost) -> Result<(), String> {
        if cost.generic_count() > 0 {
            return Err("Cost has generic component — use pay() with a generic allocation".to_string());
        }
        self.pay(cost, &HashMap::new())
    }
}

impl Default for ManaPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mana_cost_display() {
        let cost = ManaCost::build(&[ManaType::Red], 0);
        assert_eq!(format!("{}", cost), "{R}");

        let cost2 = ManaCost::build(&[ManaType::Green], 1);
        assert_eq!(format!("{}", cost2), "{1}{G}");
    }

    #[test]
    fn test_mana_cost_mana_value() {
        let bolt_cost = ManaCost::build(&[ManaType::Red], 0);
        assert_eq!(bolt_cost.mana_value(), 1);

        let bears_cost = ManaCost::build(&[ManaType::Green], 1);
        assert_eq!(bears_cost.mana_value(), 2);
    }

    #[test]
    fn test_mana_cost_multi() {
        // {2}{U}{B} = Urza's Guilt
        let cost = ManaCost::build(&[ManaType::Blue, ManaType::Black], 2);
        assert_eq!(cost.mana_value(), 4);
        assert_eq!(cost.generic_count(), 2);
        assert_eq!(cost.colored_count(ManaType::Blue), 1);
        assert_eq!(cost.colored_count(ManaType::Black), 1);
        assert_eq!(format!("{}", cost), "{2}{U}{B}");
    }

    #[test]
    fn test_mana_cost_multi_no_generic() {
        // {G}{W}{U} = Rhox War Monk style
        let cost = ManaCost::build(&[ManaType::Green, ManaType::White, ManaType::Blue], 0);
        assert_eq!(cost.mana_value(), 3);
        assert_eq!(cost.generic_count(), 0);
        assert_eq!(format!("{}", cost), "{G}{W}{U}");
    }

    #[test]
    fn test_mana_cost_multi_heavy_generic() {
        // {5}{U}{B}
        let cost = ManaCost::build(&[ManaType::Blue, ManaType::Black], 5);
        assert_eq!(cost.mana_value(), 7);
        assert_eq!(cost.generic_count(), 5);
        assert_eq!(format!("{}", cost), "{5}{U}{B}");
    }

    #[test]
    fn test_mana_pool_add_and_check() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 3);

        assert!(pool.has(ManaType::Red, 3));
        assert!(!pool.has(ManaType::Red, 4));
        assert_eq!(pool.total(), 3);
    }

    #[test]
    fn test_mana_pool_remove() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 3);

        assert!(pool.remove(ManaType::Red, 2).is_ok());
        assert_eq!(pool.amount(ManaType::Red), 1);
        assert!(pool.remove(ManaType::Red, 2).is_err());
    }

    #[test]
    fn test_mana_pool_empty() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 3);
        pool.add(ManaType::Blue, 2);
        pool.empty_with_reason(ManaEmptyReason::TurnEnd, &BlanketPersistenceSet::none());
        assert_eq!(pool.total(), 0);
    }

    #[test]
    fn test_mana_pool_can_pay_specific() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 1);

        let bolt = ManaCost::build(&[ManaType::Red], 0);
        assert!(pool.can_pay(&bolt));

        let bears = ManaCost::build(&[ManaType::Green], 1);
        assert!(!pool.can_pay(&bears));
    }

    #[test]
    fn test_mana_pool_can_pay_generic() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 1);
        pool.add(ManaType::Red, 1);

        let bears = ManaCost::build(&[ManaType::Green], 1);
        assert!(pool.can_pay(&bears));
    }

    #[test]
    fn test_mana_pool_pay_with_generic_allocation() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 2);
        pool.add(ManaType::Red, 1);

        let bears = ManaCost::build(&[ManaType::Green], 1);
        // Player chooses to spend Red for the generic cost (save Green)
        let mut alloc = HashMap::new();
        alloc.insert(ManaType::Red, 1);
        assert!(pool.pay(&bears, &alloc).is_ok());

        assert_eq!(pool.amount(ManaType::Green), 1);
        assert_eq!(pool.amount(ManaType::Red), 0);
    }

    #[test]
    fn test_mana_pool_pay_generic_with_same_color() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 3);

        let bears = ManaCost::build(&[ManaType::Green], 1);
        // Player pays generic with Green too
        let mut alloc = HashMap::new();
        alloc.insert(ManaType::Green, 1);
        assert!(pool.pay(&bears, &alloc).is_ok());

        assert_eq!(pool.amount(ManaType::Green), 1);
    }

    #[test]
    fn test_mana_pool_pay_insufficient() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 1);

        let bears = ManaCost::build(&[ManaType::Green], 1);
        let alloc = HashMap::new();
        assert!(pool.pay(&bears, &alloc).is_err());
        // Pool should be unchanged after failed payment
        assert_eq!(pool.amount(ManaType::Red), 1);
    }

    #[test]
    fn test_mana_pool_pay_bad_allocation_sum() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 2);
        pool.add(ManaType::Red, 1);

        let bears = ManaCost::build(&[ManaType::Green], 1);
        // Allocation sums to 2, but generic cost is 1
        let mut alloc = HashMap::new();
        alloc.insert(ManaType::Red, 1);
        alloc.insert(ManaType::Green, 1);
        assert!(pool.pay(&bears, &alloc).is_err());
    }

    #[test]
    fn test_mana_pool_pay_allocation_exceeds_available() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 1); // need 1G specific, 0 left for generic
        pool.add(ManaType::Red, 1);

        let bears = ManaCost::build(&[ManaType::Green], 1);
        // Try to allocate Green for generic — but only 1G exists and it's needed for specific
        let mut alloc = HashMap::new();
        alloc.insert(ManaType::Green, 1);
        assert!(pool.pay(&bears, &alloc).is_err());
    }

    #[test]
    fn test_mana_pool_pay_specific_only() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 1);

        let bolt = ManaCost::build(&[ManaType::Red], 0);
        assert!(pool.pay_specific_only(&bolt).is_ok());
        assert_eq!(pool.amount(ManaType::Red), 0);
    }

    #[test]
    fn test_mana_pool_pay_specific_only_rejects_generic() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 2);

        let bears = ManaCost::build(&[ManaType::Green], 1);
        assert!(pool.pay_specific_only(&bears).is_err());
    }

    #[test]
    fn test_mana_persistence_enum() {
        // Verify the enum exists and variants are distinct
        assert_ne!(ManaPersistence::Normal, ManaPersistence::UntilEndOf(PersistenceExpiry::EndOfTurn));
        assert_ne!(
            ManaPersistence::UntilEndOf(PersistenceExpiry::EndOfTurn),
            ManaPersistence::UntilEndOf(PersistenceExpiry::EndOfCombat),
        );
    }

    #[test]
    fn test_mana_atom_simple() {
        let atom = ManaAtom::simple(ManaType::Green, None);
        assert_eq!(atom.mana_type, ManaType::Green);
        assert_eq!(atom.persistence, ManaPersistence::Normal);
        assert!(atom.restrictions.is_empty());
        assert!(atom.grants.is_empty());
    }

    // =========================================================================
    // T12b: ManaPool sidecar tests
    // =========================================================================

    fn creature_spend_ctx() -> SpendContext<'static> {
        use std::sync::LazyLock;
        static TYPES: LazyLock<HashSet<CardType>> = LazyLock::new(|| {
            let mut s = HashSet::new();
            s.insert(CardType::Creature);
            s
        });
        static SUBTYPES: LazyLock<HashSet<Subtype>> = LazyLock::new(|| {
            let mut s = HashSet::new();
            s.insert(Subtype::Creature(CreatureType::Bear));
            s
        });
        SpendContext {
            purpose: SpendPurpose::CastSpell {
                card_types: &TYPES,
                subtypes: &SUBTYPES,
                name: "Grizzly Bears",
            },
        }
    }

    fn instant_spend_ctx() -> SpendContext<'static> {
        use std::sync::LazyLock;
        static TYPES: LazyLock<HashSet<CardType>> = LazyLock::new(|| {
            let mut s = HashSet::new();
            s.insert(CardType::Instant);
            s
        });
        static SUBTYPES: LazyLock<HashSet<Subtype>> = LazyLock::new(|| HashSet::new());
        SpendContext {
            purpose: SpendPurpose::CastSpell {
                card_types: &TYPES,
                subtypes: &SUBTYPES,
                name: "Lightning Bolt",
            },
        }
    }

    fn creature_only_green_atom() -> ManaAtom {
        ManaAtom {
            mana_type: ManaType::Green,
            source_id: None,
            restrictions: vec![ManaRestriction::OnlyForSpellTypes(vec![CardType::Creature])],
            grants: Vec::new(),
            persistence: ManaPersistence::Normal,
        }
    }

    #[test]
    fn test_add_special_atom() {
        let mut pool = ManaPool::new();
        assert!(!pool.has_special());

        pool.add_special(creature_only_green_atom());

        assert!(pool.has_special());
        assert_eq!(pool.special_atoms().len(), 1);
        assert_eq!(pool.special_atoms()[0].1, 1); // count = 1
        assert_eq!(pool.special_atoms()[0].0.mana_type, ManaType::Green);
    }

    #[test]
    fn test_special_coalesce_identical_atoms() {
        let mut pool = ManaPool::new();
        for _ in 0..5 {
            pool.add_special(creature_only_green_atom());
        }
        // All 5 identical atoms coalesce into one group
        assert_eq!(pool.special_atoms().len(), 1);
        assert_eq!(pool.special_atoms()[0].1, 5);
    }

    #[test]
    fn test_special_no_coalesce_different_grants() {
        let mut pool = ManaPool::new();
        // Atom with no grants
        pool.add_special(creature_only_green_atom());
        // Atom with a grant — different, should NOT coalesce
        pool.add_special(ManaAtom {
            mana_type: ManaType::Green,
            source_id: None,
            restrictions: vec![ManaRestriction::OnlyForSpellTypes(vec![CardType::Creature])],
            grants: vec![ManaGrant::GrantKeyword(crate::types::keywords::KeywordAbility::Haste)],
            persistence: ManaPersistence::Normal,
        });
        assert_eq!(pool.special_atoms().len(), 2);
    }

    #[test]
    fn test_amount_for_with_eligible_special() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 2); // 2 unrestricted green
        pool.add_special(creature_only_green_atom()); // 1 creature-only green

        let ctx = creature_spend_ctx();
        // Creature spell: sees 2 simple + 1 special = 3
        assert_eq!(pool.amount_for(ManaType::Green, &ctx), 3);
    }

    #[test]
    fn test_amount_for_with_ineligible_special() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 2);
        pool.add_special(creature_only_green_atom());

        let ctx = instant_spend_ctx();
        // Instant spell: creature-only mana is ineligible → 2 simple only
        assert_eq!(pool.amount_for(ManaType::Green, &ctx), 2);
    }

    #[test]
    fn test_can_pay_with_context_uses_special() {
        let mut pool = ManaPool::new();
        // Only 1 unrestricted green — can't pay {1}{G} alone
        pool.add(ManaType::Green, 1);
        let bears = ManaCost::build(&[ManaType::Green], 1);
        assert!(!pool.can_pay(&bears)); // old can_pay doesn't see special

        // Add creature-only green → now {1}{G} creature spell is payable
        pool.add_special(creature_only_green_atom());
        let ctx = creature_spend_ctx();
        assert!(pool.can_pay_with_context(&bears, &ctx));
    }

    #[test]
    fn test_can_pay_with_context_rejects_ineligible() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 1);
        pool.add_special(creature_only_green_atom());

        // {1}{G} instant: creature-only green can't be used
        let cost = ManaCost::build(&[ManaType::Green], 1);
        let ctx = instant_spend_ctx();
        assert!(!pool.can_pay_with_context(&cost, &ctx));
    }

    #[test]
    fn test_pay_with_plan_simple() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 2);
        pool.add(ManaType::Green, 1);

        let mut from_simple = HashMap::new();
        from_simple.insert(ManaType::Red, 1);
        from_simple.insert(ManaType::Green, 1);
        let plan = ManaPaymentPlan { from_simple, from_special: vec![] };
        assert!(pool.pay_with_plan(&plan).is_ok());

        assert_eq!(pool.amount(ManaType::Red), 1);
        assert_eq!(pool.amount(ManaType::Green), 0);
    }

    #[test]
    fn test_pay_with_plan_mixed() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 1); // 1 unrestricted green
        pool.add_special(creature_only_green_atom()); // 1 creature-only green (group 0)

        let mut from_simple = HashMap::new();
        from_simple.insert(ManaType::Green, 1);
        let plan = ManaPaymentPlan {
            from_simple,
            from_special: vec![(0, 1)], // spend 1 from special group 0
        };
        assert!(pool.pay_with_plan(&plan).is_ok());

        assert_eq!(pool.amount(ManaType::Green), 0);
        assert!(!pool.has_special()); // special group fully spent → removed
    }

    #[test]
    fn test_pay_with_plan_collects_grants() {
        let mut pool = ManaPool::new();
        let atom_with_grant = ManaAtom {
            mana_type: ManaType::Green,
            source_id: None,
            restrictions: vec![ManaRestriction::OnlyForSpellTypes(vec![CardType::Creature])],
            grants: vec![ManaGrant::GrantKeyword(crate::types::keywords::KeywordAbility::Haste)],
            persistence: ManaPersistence::Normal,
        };
        pool.add_special(atom_with_grant);

        let plan = ManaPaymentPlan {
            from_simple: HashMap::new(),
            from_special: vec![(0, 1)],
        };
        assert!(pool.pay_with_plan(&plan).is_ok());

        let grants = pool.drain_spent_grants();
        assert_eq!(grants.len(), 1);
        assert_eq!(
            grants[0],
            ManaGrant::GrantKeyword(crate::types::keywords::KeywordAbility::Haste)
        );
        // Second drain returns empty
        assert!(pool.drain_spent_grants().is_empty());
    }

    #[test]
    fn test_empty_with_reason_step_no_blanket() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 3); // normal simple mana
        pool.add_special(ManaAtom {
            mana_type: ManaType::Red,
            source_id: None,
            restrictions: Vec::new(),
            grants: Vec::new(),
            persistence: ManaPersistence::UntilEndOf(PersistenceExpiry::EndOfTurn),
        });
        pool.add_special(creature_only_green_atom()); // Normal persistence in special

        pool.empty_with_reason(ManaEmptyReason::StepOrPhase, &BlanketPersistenceSet::none());

        // Normal simple mana: emptied
        assert_eq!(pool.amount(ManaType::Red), 0);
        // EndOfTurn atom: survives StepOrPhase
        assert_eq!(pool.special_atoms().len(), 1);
        assert_eq!(pool.special_atoms()[0].0.persistence,
            ManaPersistence::UntilEndOf(PersistenceExpiry::EndOfTurn));
        // Normal-persistence creature-only green: emptied (no blanket)
    }

    #[test]
    fn test_empty_with_reason_turn_end() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 3);
        pool.add_special(ManaAtom {
            mana_type: ManaType::Red,
            source_id: None,
            restrictions: Vec::new(),
            grants: Vec::new(),
            persistence: ManaPersistence::UntilEndOf(PersistenceExpiry::EndOfTurn),
        });

        pool.empty_with_reason(ManaEmptyReason::TurnEnd, &BlanketPersistenceSet::none());

        assert_eq!(pool.amount(ManaType::Red), 0);
        // EndOfTurn atom also empties at TurnEnd
        assert!(pool.special_atoms().is_empty());
    }

    #[test]
    fn test_empty_with_blanket_green() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 5);
        pool.add(ManaType::Red, 2);

        let mut blanket = BlanketPersistenceSet::none();
        blanket.types.insert(ManaType::Green);

        pool.empty_with_reason(ManaEmptyReason::StepOrPhase, &blanket);

        // Green survives blanket, red empties
        assert_eq!(pool.amount(ManaType::Green), 5);
        assert_eq!(pool.amount(ManaType::Red), 0);
    }

    #[test]
    fn test_empty_with_blanket_all() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 3);
        pool.add(ManaType::Red, 2);
        pool.add(ManaType::Blue, 1);

        let blanket = BlanketPersistenceSet { all: true, types: HashSet::new() };
        pool.empty_with_reason(ManaEmptyReason::StepOrPhase, &blanket);

        // All mana survives
        assert_eq!(pool.amount(ManaType::Green), 3);
        assert_eq!(pool.amount(ManaType::Red), 2);
        assert_eq!(pool.amount(ManaType::Blue), 1);
    }

    #[test]
    fn test_blanket_removed_then_empties() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 5);

        // First empty: blanket protects green
        let mut blanket = BlanketPersistenceSet::none();
        blanket.types.insert(ManaType::Green);
        pool.empty_with_reason(ManaEmptyReason::StepOrPhase, &blanket);
        assert_eq!(pool.amount(ManaType::Green), 5);

        // Second empty: no blanket (Omnath died) → green empties
        pool.empty_with_reason(ManaEmptyReason::StepOrPhase, &BlanketPersistenceSet::none());
        assert_eq!(pool.amount(ManaType::Green), 0);
    }

    #[test]
    fn test_restriction_allows_spell_type_match() {
        let restriction = ManaRestriction::OnlyForSpellTypes(vec![CardType::Creature]);
        let ctx = creature_spend_ctx();
        // Private method — test via ManaAtom::allows_spend
        let atom = ManaAtom {
            mana_type: ManaType::Green,
            source_id: None,
            restrictions: vec![restriction],
            grants: Vec::new(),
            persistence: ManaPersistence::Normal,
        };
        // Creature spell: should be eligible
        let mut pool = ManaPool::new();
        pool.add_special(atom);
        assert_eq!(pool.amount_for(ManaType::Green, &ctx), 1);
    }

    #[test]
    fn test_restriction_rejects_spell_type_mismatch() {
        let restriction = ManaRestriction::OnlyForSpellTypes(vec![CardType::Creature]);
        let ctx = instant_spend_ctx();
        let atom = ManaAtom {
            mana_type: ManaType::Green,
            source_id: None,
            restrictions: vec![restriction],
            grants: Vec::new(),
            persistence: ManaPersistence::Normal,
        };
        let mut pool = ManaPool::new();
        pool.add_special(atom);
        // Instant spell: creature-only mana not eligible
        assert_eq!(pool.amount_for(ManaType::Green, &ctx), 0);
    }

    #[test]
    fn test_restriction_any_of() {
        use std::sync::LazyLock;
        // AnyOf: creature spells OR artifact ability activation
        let restriction = ManaRestriction::AnyOf(vec![
            ManaRestriction::OnlyForSpellTypes(vec![CardType::Creature]),
            ManaRestriction::OnlyForAbilityTypes(vec![CardType::Artifact]),
        ]);
        let atom = ManaAtom {
            mana_type: ManaType::Colorless,
            source_id: None,
            restrictions: vec![restriction],
            grants: Vec::new(),
            persistence: ManaPersistence::Normal,
        };

        let mut pool = ManaPool::new();
        pool.add_special(atom);

        // Creature spell: matches first branch
        let creature_ctx = creature_spend_ctx();
        assert_eq!(pool.amount_for(ManaType::Colorless, &creature_ctx), 1);

        // Instant spell: matches neither branch
        let instant_ctx = instant_spend_ctx();
        assert_eq!(pool.amount_for(ManaType::Colorless, &instant_ctx), 0);

        // Artifact ability activation: matches second branch
        static ART_TYPES: LazyLock<HashSet<CardType>> = LazyLock::new(|| {
            let mut s = HashSet::new();
            s.insert(CardType::Artifact);
            s
        });
        static ART_SUBTYPES: LazyLock<HashSet<Subtype>> = LazyLock::new(|| HashSet::new());
        let ability_ctx = SpendContext {
            purpose: SpendPurpose::ActivateAbility {
                source_card_types: &ART_TYPES,
                source_subtypes: &ART_SUBTYPES,
            },
        };
        assert_eq!(pool.amount_for(ManaType::Colorless, &ability_ctx), 1);
    }

    #[test]
    fn test_restriction_creature_type() {
        use std::sync::LazyLock;
        let restriction = ManaRestriction::OnlyForCreatureType(CreatureType::Elf);
        let atom = ManaAtom {
            mana_type: ManaType::Green,
            source_id: None,
            restrictions: vec![restriction],
            grants: Vec::new(),
            persistence: ManaPersistence::Normal,
        };

        let mut pool = ManaPool::new();
        pool.add_special(atom);

        // Elf creature spell: matches
        static ELF_TYPES: LazyLock<HashSet<CardType>> = LazyLock::new(|| {
            let mut s = HashSet::new();
            s.insert(CardType::Creature);
            s
        });
        static ELF_SUBTYPES: LazyLock<HashSet<Subtype>> = LazyLock::new(|| {
            let mut s = HashSet::new();
            s.insert(Subtype::Creature(CreatureType::Elf));
            s
        });
        let elf_ctx = SpendContext {
            purpose: SpendPurpose::CastSpell {
                card_types: &ELF_TYPES,
                subtypes: &ELF_SUBTYPES,
                name: "Llanowar Elves",
            },
        };
        assert_eq!(pool.amount_for(ManaType::Green, &elf_ctx), 1);

        // Bear creature spell: doesn't match (wrong creature type)
        let bear_ctx = creature_spend_ctx();
        assert_eq!(pool.amount_for(ManaType::Green, &bear_ctx), 0);

        // Instant spell: doesn't match (not a creature)
        let instant_ctx = instant_spend_ctx();
        assert_eq!(pool.amount_for(ManaType::Green, &instant_ctx), 0);
    }

    #[test]
    fn test_restriction_changeling_matches_any_type() {
        use std::sync::LazyLock;
        let restriction = ManaRestriction::OnlyForCreatureType(CreatureType::Elf);
        let atom = ManaAtom {
            mana_type: ManaType::Green,
            source_id: None,
            restrictions: vec![restriction],
            grants: Vec::new(),
            persistence: ManaPersistence::Normal,
        };

        let mut pool = ManaPool::new();
        pool.add_special(atom);

        // Changeling creature: has ALL creature types including Elf
        static CHANGELING_TYPES: LazyLock<HashSet<CardType>> = LazyLock::new(|| {
            let mut s = HashSet::new();
            s.insert(CardType::Creature);
            s
        });
        static CHANGELING_SUBTYPES: LazyLock<HashSet<Subtype>> = LazyLock::new(|| {
            let mut s = HashSet::new();
            // Changeling has every creature type — include Elf among others
            s.insert(Subtype::Creature(CreatureType::Elf));
            s.insert(Subtype::Creature(CreatureType::Bear));
            s.insert(Subtype::Creature(CreatureType::Shapeshifter));
            s
        });
        let changeling_ctx = SpendContext {
            purpose: SpendPurpose::CastSpell {
                card_types: &CHANGELING_TYPES,
                subtypes: &CHANGELING_SUBTYPES,
                name: "Changeling Outcast",
            },
        };
        assert_eq!(pool.amount_for(ManaType::Green, &changeling_ctx), 1);
    }

    #[test]
    fn test_persistence_expiry_matches() {
        // EndOfCombat expires at both StepOrPhase and TurnEnd
        assert!(PersistenceExpiry::EndOfCombat.matches(&ManaEmptyReason::StepOrPhase));
        assert!(PersistenceExpiry::EndOfCombat.matches(&ManaEmptyReason::TurnEnd));

        // EndOfTurn survives StepOrPhase, expires at TurnEnd
        assert!(!PersistenceExpiry::EndOfTurn.matches(&ManaEmptyReason::StepOrPhase));
        assert!(PersistenceExpiry::EndOfTurn.matches(&ManaEmptyReason::TurnEnd));

        // EndOfPhase expires at both
        assert!(PersistenceExpiry::EndOfPhase.matches(&ManaEmptyReason::StepOrPhase));
        assert!(PersistenceExpiry::EndOfPhase.matches(&ManaEmptyReason::TurnEnd));
    }
}
