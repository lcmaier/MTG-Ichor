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

/// Represents the mana cost printed on a card.
///
/// Each field represents the number of mana symbols of that type in the cost.
/// `generic` is the number in the gray circle (payable by any type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManaCost {
    pub white: u8,
    pub blue: u8,
    pub black: u8,
    pub red: u8,
    pub green: u8,
    pub colorless: u8,
    pub generic: u8,
}

impl ManaCost {
    pub const ZERO: ManaCost = ManaCost {
        white: 0, blue: 0, black: 0, red: 0, green: 0, colorless: 0, generic: 0,
    };

    /// Convenience constructor for a mono-colored cost: e.g. `ManaCost::single(ManaType::Red, 1, 0)` = {R}
    pub fn single(mana_type: ManaType, colored: u8, generic: u8) -> Self {
        let mut cost = ManaCost::ZERO;
        cost.generic = generic;
        match mana_type {
            ManaType::White => cost.white = colored,
            ManaType::Blue => cost.blue = colored,
            ManaType::Black => cost.black = colored,
            ManaType::Red => cost.red = colored,
            ManaType::Green => cost.green = colored,
            ManaType::Colorless => cost.colorless = colored,
        }
        cost
    }

    /// Total mana value (converted mana cost)
    pub fn mana_value(&self) -> u8 {
        self.white + self.blue + self.black + self.red + self.green + self.colorless + self.generic
    }
}

impl fmt::Display for ManaCost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.generic > 0 {
            write!(f, "{{{}}}", self.generic)?;
        }
        for _ in 0..self.white { write!(f, "{{W}}")?; }
        for _ in 0..self.blue { write!(f, "{{U}}")?; }
        for _ in 0..self.black { write!(f, "{{B}}")?; }
        for _ in 0..self.red { write!(f, "{{R}}")?; }
        for _ in 0..self.green { write!(f, "{{G}}")?; }
        for _ in 0..self.colorless { write!(f, "{{C}}")?; }
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
    pub fn can_pay(&self, cost: &ManaCost) -> bool {
        if !self.has(ManaType::White, cost.white as u64) { return false; }
        if !self.has(ManaType::Blue, cost.blue as u64) { return false; }
        if !self.has(ManaType::Black, cost.black as u64) { return false; }
        if !self.has(ManaType::Red, cost.red as u64) { return false; }
        if !self.has(ManaType::Green, cost.green as u64) { return false; }
        if !self.has(ManaType::Colorless, cost.colorless as u64) { return false; }

        let remaining = self.total()
            - cost.white as u64
            - cost.blue as u64
            - cost.black as u64
            - cost.red as u64
            - cost.green as u64
            - cost.colorless as u64;

        remaining >= cost.generic as u64
    }

    /// Pay a ManaCost from this pool.
    ///
    /// Specific color requirements are paid automatically (they must be paid
    /// with that exact color — no choice involved).
    ///
    /// Generic costs require the caller to provide `generic_allocation`: a
    /// HashMap specifying how many of each ManaType to spend on the generic
    /// portion. The values must sum to `cost.generic`. This is a player choice
    /// because the player may want to preserve specific colors for future spells.
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

        // Validate generic allocation sums to generic cost
        let alloc_total: u64 = generic_allocation.values().sum();
        if alloc_total != cost.generic as u64 {
            return Err(format!(
                "Generic allocation sums to {} but generic cost is {}",
                alloc_total, cost.generic
            ));
        }

        // Validate the allocation doesn't exceed what remains after specific costs
        for (&mana_type, &alloc_amount) in generic_allocation {
            let specific_need = match mana_type {
                ManaType::White => cost.white as u64,
                ManaType::Blue => cost.blue as u64,
                ManaType::Black => cost.black as u64,
                ManaType::Red => cost.red as u64,
                ManaType::Green => cost.green as u64,
                ManaType::Colorless => cost.colorless as u64,
            };
            let available = self.amount(mana_type);
            if specific_need + alloc_amount > available {
                return Err(format!(
                    "Cannot spend {} {:?} on generic + {} specific — only {} available",
                    alloc_amount, mana_type, specific_need, available
                ));
            }
        }

        // Pay specific colors
        self.remove(ManaType::White, cost.white as u64)?;
        self.remove(ManaType::Blue, cost.blue as u64)?;
        self.remove(ManaType::Black, cost.black as u64)?;
        self.remove(ManaType::Red, cost.red as u64)?;
        self.remove(ManaType::Green, cost.green as u64)?;
        self.remove(ManaType::Colorless, cost.colorless as u64)?;

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
        if cost.generic > 0 {
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
