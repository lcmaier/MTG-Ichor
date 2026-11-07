//! Who and what an instruction acts on, as the words a card file writes in
//! an instruction's first argument: CR 115's targets in 115's order, then
//! every other recipient by the rule that defines it, then the filters a
//! recipient word takes. A recipient pairs with every verb in
//! [`super::instructions`], so a new one works with all of them at once.

use crate::types::card_types::{CardType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, Antecedent, Choice, DividedCount, EffectRecipient, ObjectFilter, PlayerGroup, PlayerRef, PlayerSet,
    SelectionFilter, TargetCount,
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
    target(SelectionFilter::Player(PlayerSet::Everyone))
}

/// "Target opponent": a player other than the controller (CR 102.1, 109.5).
pub fn target_opponent() -> EffectRecipient {
    target(SelectionFilter::Player(PlayerSet::Opponents))
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

/// `total` "divided as you choose among" `among` targets matching `what`: one
/// instance of "target" (CR 115.3) whose announcement also divides (CR 601.2d).
pub fn divided_among(what: SelectionFilter, among: TargetCount, total: AmountExpr) -> EffectRecipient {
    EffectRecipient::Target(what, TargetCount::Divided(Box::new(DividedCount { among, total })))
}

/// "It", "that creature", "them": what was chosen for the `ix`th instance of
/// "target" this effect declares (CR 115.3), counted across a spell's
/// abilities. A second change in one sentence is
/// [`super::instructions::ContinuousChange`]'s to write, not this word's.
pub fn that_target(ix: usize) -> EffectRecipient {
    EffectRecipient::SameInstanceAs(ix)
}

/// "Any target" (CR 115.4): a creature, player, planeswalker or battle.
pub fn any_target() -> EffectRecipient {
    target(SelectionFilter::Any)
}

/// "Any other target" (CR 115.4): any target but what the instance of
/// "target" `than` chose — Arc Trail's second.
pub fn any_other_target(than: usize) -> EffectRecipient {
    another_target(SelectionFilter::Any, &[than])
}

/// "Another target [filter]", "a third target [filter]" (CR 601.2c): one more
/// instance of "target", other than what each instance in `than` chose.
pub fn another_target(filter: SelectionFilter, than: &[usize]) -> EffectRecipient {
    target(SelectionFilter::other_than(filter, than))
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
pub fn triggering_object(role: EventRole) -> EffectRecipient {
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
//
// A word for each quality two cards print, in the CR's order, joined by
// `ObjectFilter::and` and `ObjectFilter::or` as the text joins them. A
// quality one card prints is the enum written out, as `ObjectFilter::All` is.

/// "Non[quality]": "nonblack" (CR 105.2), "nontoken", "nonbasic" (CR
/// 205.4c): every object `filter` does not describe.
pub fn non(filter: ObjectFilter) -> ObjectFilter {
    ObjectFilter::Not(Box::new(filter))
}

/// "Black", "red source" (CR 105.2): an object that is `which`, among any
/// others it is.
pub fn color(which: Color) -> ObjectFilter {
    ObjectFilter::ByColor(which)
}

/// "You control" (CR 110.2), read against the effect's controller (CR
/// 109.5): off the battlefield, its owner (CR 108.4a), so a card in a
/// graveyard means its owner's.
pub fn you_control() -> ObjectFilter {
    ObjectFilter::ByController(PlayerRef::You)
}

/// "An opponent controls" (CR 110.2), an opponent of the effect's
/// controller (CR 102.1).
pub fn an_opponent_controls() -> ObjectFilter {
    ObjectFilter::ByController(PlayerRef::Opponent)
}

/// "Your opponents control" (CR 110.2): the objects every opponent of the
/// effect's controller controls (CR 102.1), which in a filter is the same
/// set as [`an_opponent_controls`]. A teammate's are not among them (CR
/// 102.3), though today's reader takes every other player as an opponent.
pub fn your_opponents_control() -> ObjectFilter {
    ObjectFilter::ByController(PlayerRef::Opponent)
}

/// "Spells you cast", for a cost effect (CR 601.2f), read while the spell is
/// being cast, when its controller is its caster (CR 601.2a). Once control of
/// a spell changes the two differ (CR 108.4), and a reader after casting asks
/// who cast it, as `TriggerEvent::CastsSpell`'s `caster` does.
pub fn you_cast(filter: ObjectFilter) -> ObjectFilter {
    filter.and(you_control())
}

/// "Token" (CR 111.1); "nontoken" is `non(token())`.
pub fn token() -> ObjectFilter {
    ObjectFilter::Token
}

/// "Other", "each other": any object but the one whose ability this is
/// (CR 113.7), which a granted or copied ability reads as its carrier.
pub fn other_than_this() -> ObjectFilter {
    ObjectFilter::NotSource
}

/// "Forest", "Aura", "Elf" (CR 205.3): an object with `subtype`.
pub fn with_subtype(subtype: Subtype) -> ObjectFilter {
    ObjectFilter::BySubtype(subtype)
}

/// "Basic" (CR 205.4c); "nonbasic land" is `land().and(non(basic()))`.
pub fn basic() -> ObjectFilter {
    ObjectFilter::BySupertype(Supertype::Basic)
}

/// "Artifact" (CR 301).
pub fn artifact() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Artifact)
}

/// "Creature" (CR 302).
pub fn creature() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Creature)
}

/// "Enchantment" (CR 303).
pub fn enchantment() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Enchantment)
}

/// "Land" (CR 305).
pub fn land() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Land)
}

/// "Planeswalker" (CR 306).
pub fn planeswalker() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Planeswalker)
}

/// "Creatures you control" as a filter, which the layer walk reads against
/// the effect's controller (CR 109.5): off the battlefield, its owner (CR
/// 108.4), so a card in a graveyard means its owner's creatures.
pub fn creatures_you_control() -> ObjectFilter {
    creature().and(you_control())
}

/// "Nonlegendary creature" — the filter Cytoshape and Mirrorweave scope
/// their copy source with, and the reason CR 707 cards say it at all: a copy
/// of a legend meets CR 704.5j the moment it exists, so the printed text
/// keeps the effect from being a sacrifice.
pub fn nonlegendary_creature() -> ObjectFilter {
    creature().and(non(ObjectFilter::BySupertype(Supertype::Legendary)))
}
