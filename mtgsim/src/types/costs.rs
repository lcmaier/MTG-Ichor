use super::effects::{CardFilter, CounterType, PermanentFilter};
use super::mana::ManaCost;

/// Costs that must be paid to activate an ability or cast a spell.
///
/// Only `Tap`, `Mana`, `SacrificeSelf`, and `PayLife` are fully implemented.
/// Other variants exist for forward-compatibility; `can_pay_costs` and
/// `pay_single_cost` return `Err("not yet implemented")` for them.
#[derive(Debug, Clone, PartialEq)]
pub enum Cost {
    /// Tap the source permanent
    Tap,
    /// Untap the source permanent (Devoted Druid)
    Untap,
    /// Pay a mana cost
    Mana(ManaCost),
    /// Pay N life
    PayLife(u64),
    /// Sacrifice the source permanent
    SacrificeSelf,
    /// Sacrifice N permanents matching a filter ("Sacrifice a creature")
    Sacrifice(PermanentFilter, u32),
    /// Discard N cards matching a filter ("Discard a card")
    Discard(CardFilter, u32),
    /// Exile N cards from your graveyard matching a filter
    ExileFromGraveyard(CardFilter, u32),
    /// Remove N counters of a type from the source
    RemoveCounters(CounterType, u32),
    /// Add N counters of a type to the source (e.g. blight counters)
    AddCounters(CounterType, u32),
}

/// An alternative cost that can replace a spell's mana cost (rule 118.9).
///
/// A player can only choose one alternative cost per spell cast.
/// The `Vec<Cost>` payload describes the costs to pay instead of the
/// normal mana cost. T18 will wire these into the casting pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum AlternativeCost {
    Flashback(Vec<Cost>),
    Overload(Vec<Cost>),
    Dash(Vec<Cost>),
    Escape(Vec<Cost>),
    Evoke(Vec<Cost>),
    Bestow(Vec<Cost>),
    Custom(String, Vec<Cost>),
}

/// An additional cost that can be paid on top of a spell's mana cost (rule 118.8).
///
/// A spell may have multiple additional costs, each optionally paid.
/// The `Vec<Cost>` payload describes the costs for each.
#[derive(Debug, Clone, PartialEq)]
pub enum AdditionalCost {
    Kicker(Vec<Cost>),
    Buyback(Vec<Cost>),
    Entwine(Vec<Cost>),
    Casualty(u32),
    Bargain,
    Strive(Vec<Cost>),
    Custom(String, Vec<Cost>),
}
