use std::collections::HashMap;
use std::fmt;

use super::colors::Color;
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

    /// Convenience constructor for a mono-colored cost.
    ///
    /// e.g. `ManaCost::single(ManaType::Red, 1, 0)` = {R}
    /// e.g. `ManaCost::single(ManaType::Green, 1, 1)` = {1}{G}
    pub fn single(mana_type: ManaType, colored: u8, generic: u8) -> Self {
        let mut symbols = Vec::with_capacity((generic + colored) as usize);
        for _ in 0..generic {
            symbols.push(ManaSymbol::Generic);
        }
        let sym = if mana_type == ManaType::Colorless {
            ManaSymbol::Colorless
        } else {
            ManaSymbol::Colored(mana_type)
        };
        for _ in 0..colored {
            symbols.push(sym);
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
// Phase 6 types — standalone, not wired into ManaPool yet.
//
// These types model mana spending restrictions (rule 106.6), mana grants, and
// persistence scoping. They will be integrated into ManaPool as a sidecar
// Vec<ManaAtom> when Phase 6 (triggered/replacement effects) adds cards like
// Cavern of Souls, Arena of Glory, Omnath, and Birgi.
//
// Design plan for Phase 6 integration:
//
//   pub struct ManaPool {
//       simple: HashMap<ManaType, u64>,   // fast path: unrestricted mana (99% case)
//       special: Vec<ManaAtom>,           // slow path: restricted/granted/persistent mana
//   }
//
// - add()/amount()/remove()/empty() operate on `simple` in O(1) and only
//   touch `special` when it's non-empty, preserving performance for AI training.
// - When `special` is non-empty, `pay()` routes through `DecisionProvider`
//   to let the player choose which special atoms to spend (e.g. using Cavern
//   of Souls mana to make a creature uncounterable).
// =============================================================================

/// Spending restrictions on a single unit of mana (rule 106.6).
///
/// These are NOT currently wired into ManaPool — they are standalone types
/// ready for Phase 6 integration.
///
/// Examples:
/// - "Spend this mana only to cast creature spells" (Gwenna, Cavern of Souls)
/// - "Spend this mana only to activate abilities" (future)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManaRestriction {
    /// Mana can only be spent to cast spells of the given types
    OnlyForSpellTypes(Vec<crate::types::card_types::CardType>),
    // Future: OnlyForAbilities { ... }
    // Future: OnlyForCreaturesSharing { ... }
}

/// Additional effects granted when this mana is spent (rule 106.6).
///
/// Tracked per-atom so the engine knows to apply them when the mana is
/// consumed during spell casting or ability activation.
///
/// NOT currently wired into ManaPool — standalone type for Phase 6.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManaGrant {
    /// The spell cast using this mana gains a keyword
    /// (e.g. haste from Arena of Glory, uncounterable from Cavern of Souls)
    GrantKeyword(crate::types::keywords::KeywordAbility),
    // Future: TriggerOnSpend { ... }
}

/// How long mana persists in the pool before being emptied.
///
/// Different effects grant different scopes of persistence:
/// - Birgi: "Red mana doesn't empty from your mana pool as steps and phases end"
///   → persists through phases but empties at end of turn
/// - Omnath, Locus of Mana: "Green mana doesn't empty from your mana pool as
///   steps and phases end" (plus the creature gets +1/+1 per green mana)
///   → effectively indefinite while Omnath is on the battlefield
///
/// NOT currently wired into ManaPool — standalone type for Phase 6.
/// When integrated, `ManaPool::empty()` will take a `ManaEmptyReason` parameter
/// to distinguish step/phase transitions from turn transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManaPersistence {
    /// Normal mana — empties at every step/phase transition (rule 106.4)
    Normal,
    /// Persists through steps and phases, empties at end of turn (Birgi)
    UntilEndOfTurn,
    /// Persists indefinitely until spent or the source effect ends (Omnath)
    Indefinite,
}

/// A single unit of mana with metadata about its source, restrictions, and
/// persistence.
///
/// NOT currently stored in ManaPool — this is a standalone type for Phase 6.
/// When special mana is introduced, ManaPool will gain a `special: Vec<ManaAtom>`
/// sidecar alongside the fast `simple: HashMap<ManaType, u64>` path.
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
}

/// A player's mana pool — tracks available mana by type.
///
/// Uses a `HashMap<ManaType, u64>` for O(1) lookups and mutations.
/// This handles the common case (unrestricted mana) efficiently.
///
/// **Generic mana payment is a player choice.** The `pay()` method handles
/// only specific color requirements automatically (since those MUST be paid
/// with that exact color). Generic costs require the caller to provide an
/// explicit `generic_allocation` specifying which types of mana to use,
/// because the player may want to save specific colors for later spells.
///
/// **Phase 6 plan:** When cards with mana restrictions/grants are added,
/// a `special: Vec<ManaAtom>` sidecar will be added alongside `pool`. The
/// `DecisionProvider` trait will gain a `choose_mana_payment` method so
/// players/AI can choose which special atoms to spend. See `ManaAtom` docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManaPool {
    pool: HashMap<ManaType, u64>,
}

impl ManaPool {
    pub fn new() -> Self {
        ManaPool {
            pool: HashMap::new(),
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

    /// Empty the entire mana pool (rule 106.4).
    ///
    /// Phase 6: This will take a `ManaEmptyReason` parameter
    /// (StepOrPhaseEnd vs TurnEnd) to support `ManaPersistence` scoping.
    pub fn empty(&mut self) {
        self.pool.clear();
    }

    /// Get a snapshot of available mana by type
    pub fn available(&self) -> &HashMap<ManaType, u64> {
        &self.pool
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
        let cost = ManaCost::single(ManaType::Red, 1, 0);
        assert_eq!(format!("{}", cost), "{R}");

        let cost2 = ManaCost::single(ManaType::Green, 1, 1);
        assert_eq!(format!("{}", cost2), "{1}{G}");
    }

    #[test]
    fn test_mana_cost_mana_value() {
        let bolt_cost = ManaCost::single(ManaType::Red, 1, 0);
        assert_eq!(bolt_cost.mana_value(), 1);

        let bears_cost = ManaCost::single(ManaType::Green, 1, 1);
        assert_eq!(bears_cost.mana_value(), 2);
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
        pool.empty();
        assert_eq!(pool.total(), 0);
    }

    #[test]
    fn test_mana_pool_can_pay_specific() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 1);

        let bolt = ManaCost::single(ManaType::Red, 1, 0);
        assert!(pool.can_pay(&bolt));

        let bears = ManaCost::single(ManaType::Green, 1, 1);
        assert!(!pool.can_pay(&bears));
    }

    #[test]
    fn test_mana_pool_can_pay_generic() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 1);
        pool.add(ManaType::Red, 1);

        let bears = ManaCost::single(ManaType::Green, 1, 1);
        assert!(pool.can_pay(&bears));
    }

    #[test]
    fn test_mana_pool_pay_with_generic_allocation() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 2);
        pool.add(ManaType::Red, 1);

        let bears = ManaCost::single(ManaType::Green, 1, 1);
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

        let bears = ManaCost::single(ManaType::Green, 1, 1);
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

        let bears = ManaCost::single(ManaType::Green, 1, 1);
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

        let bears = ManaCost::single(ManaType::Green, 1, 1);
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

        let bears = ManaCost::single(ManaType::Green, 1, 1);
        // Try to allocate Green for generic — but only 1G exists and it's needed for specific
        let mut alloc = HashMap::new();
        alloc.insert(ManaType::Green, 1);
        assert!(pool.pay(&bears, &alloc).is_err());
    }

    #[test]
    fn test_mana_pool_pay_specific_only() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Red, 1);

        let bolt = ManaCost::single(ManaType::Red, 1, 0);
        assert!(pool.pay_specific_only(&bolt).is_ok());
        assert_eq!(pool.amount(ManaType::Red), 0);
    }

    #[test]
    fn test_mana_pool_pay_specific_only_rejects_generic() {
        let mut pool = ManaPool::new();
        pool.add(ManaType::Green, 2);

        let bears = ManaCost::single(ManaType::Green, 1, 1);
        assert!(pool.pay_specific_only(&bears).is_err());
    }

    #[test]
    fn test_mana_persistence_enum() {
        // Verify the enum exists and variants are distinct
        assert_ne!(ManaPersistence::Normal, ManaPersistence::UntilEndOfTurn);
        assert_ne!(ManaPersistence::UntilEndOfTurn, ManaPersistence::Indefinite);
    }

    #[test]
    fn test_mana_atom_standalone() {
        // ManaAtom is a standalone type not wired into ManaPool yet
        let atom = ManaAtom::simple(ManaType::Green, None);
        assert_eq!(atom.mana_type, ManaType::Green);
        assert_eq!(atom.persistence, ManaPersistence::Normal);
        assert!(atom.restrictions.is_empty());
        assert!(atom.grants.is_empty());
    }
}
