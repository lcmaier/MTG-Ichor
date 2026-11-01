//! Who and what an instruction acts on, as the words a card file writes in
//! an instruction's first argument: CR 115's targets in 115's order, then
//! every other recipient by the rule that defines it, then the filters a
//! recipient word takes. A recipient pairs with every verb in
//! [`super::instructions`], so a new one works with all of them at once.

use crate::types::card_types::CardType;
use crate::types::effects::{
    Antecedent, Choice, EffectRecipient, ObjectFilter, PlayerGroup, PlayerRef, PlayerSet, SelectionFilter, TargetCount,
};
use crate::types::triggers::EventRole;
use crate::types::zones::ZoneSet;

// ---------------------------------------------------------------------------
// CR 115's targets
// ---------------------------------------------------------------------------

/// "Target [something]" (CR 115.1): one instance of the word "target",
/// chosen as the spell or ability is put on the stack.
pub fn target(filter: SelectionFilter) -> EffectRecipient {
    targets(1, filter)
}

/// "Target creature".
pub fn target_creature() -> EffectRecipient {
    target(SelectionFilter::Creature)
}

/// "Target player".
pub fn target_player() -> EffectRecipient {
    target(SelectionFilter::Player)
}

/// "Target [permanent the filter describes]": "target land", "target
/// nonblack creature", "target permanent".
pub fn target_permanent(filter: ObjectFilter) -> EffectRecipient {
    target(SelectionFilter::Permanent(filter))
}

/// "Target spell" (CR 115.2's object that can't exist on the battlefield).
pub fn target_spell() -> EffectRecipient {
    target(SelectionFilter::Spell)
}

/// "Two target creatures": one instance of "target" naming `count` objects
/// (CR 115.3), each different.
pub fn targets(count: u32, filter: SelectionFilter) -> EffectRecipient {
    EffectRecipient::Target(filter, TargetCount::Exactly(count))
}

/// "It", "that creature", "them": what was chosen for the `ix`th instance of
/// "target" this effect declares (CR 115.3), counted across a spell's
/// abilities. A second change in one sentence is
/// [`super::instructions::Change`]'s to write, not this word's.
pub fn that_target(ix: usize) -> EffectRecipient {
    EffectRecipient::SameInstanceAs(ix)
}

/// "Any target" (CR 115.4): a creature, player, planeswalker or battle.
pub fn any_target() -> EffectRecipient {
    target(SelectionFilter::Any)
}

// ---------------------------------------------------------------------------
// Every other recipient, by its rule
// ---------------------------------------------------------------------------

/// "Each player", in APNAP order (CR 101.4).
pub fn each_player() -> EffectRecipient {
    EffectRecipient::EachOf(PlayerGroup::set(PlayerSet::Everyone))
}

/// "Each opponent", in APNAP order (CR 101.4).
pub fn each_opponent() -> EffectRecipient {
    EffectRecipient::EachOf(PlayerGroup::set(PlayerSet::Opponents))
}

/// "You and that player", in APNAP order (CR 101.4): you, and the player the
/// effect's first instance of "target" names.
pub fn you_and_that_player() -> EffectRecipient {
    EffectRecipient::EachOf(PlayerGroup::you_and_that_player())
}

/// "Creatures you control", "each creature": every permanent the filter
/// describes (CR 109.2), read against the effect's controller.
pub fn each(filter: ObjectFilter) -> EffectRecipient {
    EffectRecipient::FilteredPermanents(filter)
}

/// "Cards in graveyards", "creature cards in your library": every object the
/// filter describes in `zones` (CR 109.2's other zones) — a static
/// ability's recipient only.
pub fn each_in(filter: ObjectFilter, zones: ZoneSet) -> EffectRecipient {
    EffectRecipient::FilteredObjectsIn(filter, zones)
}

/// "You" (CR 109.5): the controller of the spell or ability.
pub fn you() -> EffectRecipient {
    EffectRecipient::Controller
}

/// "This creature", "this permanent", the card's own name (CR 201.5): the
/// object whose ability this is.
pub fn this_object() -> EffectRecipient {
    EffectRecipient::ThisObject
}

/// "Enchanted creature" (CR 303.4m), "equipped creature" (CR 301.5a): the
/// host, the permanent this one is attached to.
pub fn host() -> EffectRecipient {
    EffectRecipient::Host
}

/// "The player or planeswalker it's attacking" (CR 506.3), for the creature
/// `attacker` names.
pub fn attacked_by(attacker: Antecedent) -> EffectRecipient {
    EffectRecipient::AttackedBy(attacker)
}

/// "That card", "it" on a delayed triggered ability (CR 603.7c): what the
/// instruction that created it acted on.
pub fn referred() -> EffectRecipient {
    EffectRecipient::Referred
}

/// "That creature", "it" on a triggered ability: the object that played
/// `role` in the triggering event (CR 608.2c).
pub fn triggering(role: EventRole) -> EffectRecipient {
    EffectRecipient::Triggering(role)
}

/// "A creature of their choice" (CR 608.2d): objects a player chooses as
/// the effect applies.
pub fn chosen_by(choice: Choice) -> EffectRecipient {
    EffectRecipient::ChosenBy(Box::new(choice))
}

// ---------------------------------------------------------------------------
// The filters a recipient word takes
// ---------------------------------------------------------------------------

/// "Creatures you control" as a filter, which the layer walk reads against
/// the effect's controller (CR 109.5): off the battlefield, its owner (CR
/// 108.4), so a card in a graveyard means its owner's creatures.
pub fn creatures_you_control() -> ObjectFilter {
    ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ByController(PlayerRef::You)))
}
