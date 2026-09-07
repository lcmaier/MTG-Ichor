use super::effects::{CardFilter, CounterType, ObjectFilter};
use super::mana::ManaCost;

/// Costs that must be paid to activate an ability or cast a spell.
///
/// Only `Tap`, `Untap`, `Mana`, `PayLife`, `SacrificeSelf` and `Sacrifice` are
/// fully implemented.
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
    Sacrifice(ObjectFilter, u32),
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

impl AlternativeCost {
    /// Extract the cost payload from any variant.
    pub fn costs(&self) -> &[Cost] {
        match self {
            AlternativeCost::Flashback(c)
            | AlternativeCost::Overload(c)
            | AlternativeCost::Dash(c)
            | AlternativeCost::Escape(c)
            | AlternativeCost::Evoke(c)
            | AlternativeCost::Bestow(c)
            | AlternativeCost::Custom(_, c) => c,
        }
    }
}

/// An additional cost paid on top of a spell's mana cost (rule 118.8).
///
/// A spell may have several, and **they are not all optional** — CR 118.8b
/// ("Some additional costs are optional") is a claim about *some*, and
/// 118.8c names the other kind outright: "a mandatory additional cost".
/// Every named variant here is a keyword mechanic whose keyword makes it
/// optional; [`AdditionalCost::Mandatory`] is the unnamed printed cost that
/// is not, and [`AdditionalCost::is_optional`] is the question 601.2b asks.
#[derive(Debug, Clone, PartialEq)]
pub enum AdditionalCost {
    Kicker(Vec<Cost>),
    Buyback(Vec<Cost>),
    Entwine(Vec<Cost>),
    Casualty(u32),
    Bargain,
    Strive(Vec<Cost>),
    Custom(String, Vec<Cost>),
    /// CR 118.8c's "mandatory additional cost" — Altar's Reap's "As an
    /// additional cost to cast this spell, sacrifice a creature."
    ///
    /// It carries no name because the card prints none: the keyword variants
    /// above are named for the mechanic that makes them *optional*, and an
    /// unnamed additional cost has only the costs themselves to identify it.
    /// Not announced at 601.2b — there is no intention to declare — and so
    /// not offered by `ask_choose_additional_costs`; it is simply in the total.
    Mandatory(Vec<Cost>),
}

impl AdditionalCost {
    /// Extract the cost payload from any variant.
    ///
    /// Variants without an explicit `Vec<Cost>` (e.g. `Casualty`, `Bargain`)
    /// return an empty slice **temporarily**. Both decompose into sacrifice
    /// primitives once `ObjectFilter` supports the required predicates:
    /// - `Casualty(n)` → `Sacrifice(power_n_or_greater, 1)`
    /// - `Bargain` → `Sacrifice(artifact_or_enchantment_or_token, 1)`
    /// After cost primitive consolidation, every variant will return a
    /// non-empty slice and the empty-slice fallback can be removed.
    pub fn costs(&self) -> &[Cost] {
        match self {
            AdditionalCost::Kicker(c)
            | AdditionalCost::Buyback(c)
            | AdditionalCost::Entwine(c)
            | AdditionalCost::Strive(c)
            | AdditionalCost::Custom(_, c)
            | AdditionalCost::Mandatory(c) => c,
            AdditionalCost::Casualty(_) | AdditionalCost::Bargain => &[],
        }
    }

    /// Whether CR 601.2b offers this cost to the player, or the total simply
    /// includes it (CR 118.8b/118.8c).
    ///
    /// Matched exhaustively on purpose: optionality is not recoverable from a
    /// variant's payload, so a new additional cost has to answer for itself
    /// rather than inherit a default that happens to be right for keywords.
    pub fn is_optional(&self) -> bool {
        match self {
            AdditionalCost::Kicker(_)
            | AdditionalCost::Buyback(_)
            | AdditionalCost::Entwine(_)
            | AdditionalCost::Casualty(_)
            | AdditionalCost::Bargain
            | AdditionalCost::Strive(_)
            | AdditionalCost::Custom(_, _) => true,
            AdditionalCost::Mandatory(_) => false,
        }
    }
}
