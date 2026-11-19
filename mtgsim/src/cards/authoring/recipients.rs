//! Who and what an instruction acts on, as the words a card file writes in
//! an instruction's first argument: CR 115's targets in 115's order, then
//! every other recipient by the rule that defines it, then the filters a
//! recipient word takes. A recipient pairs with every verb in
//! [`super::instructions`], so a new one works with all of them at once.

use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, Antecedent, Choice, ChoiceKind, ChoiceScope, ChoiceSide, DividedCount, EffectRecipient, ObjectFilter, ObjectValue, Pick,
    PlayerAntecedent, PlayerGroup, PlayerRef, PlayerSet, SelectionFilter, TargetCount, ValueIs,
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
pub fn divided_among_targeted(what: SelectionFilter, among: TargetCount, total: AmountExpr) -> EffectRecipient {
    EffectRecipient::Target(what, divided(among, total))
}

/// "[Total] distributed among [among]", "divided as you choose among": a
/// count that also splits `total` among what is chosen, at least one each
/// (CR 601.2d, 608.2d). A choose step's count for objects chosen and divided
/// as it runs, none of them a target.
pub fn divided(among: TargetCount, total: impl Into<AmountExpr>) -> TargetCount {
    TargetCount::Divided(Box::new(DividedCount { among, total: total.into() }))
}

/// "It", "that creature", "them": what was chosen for the `ix`th instance of
/// "target" this effect declares (CR 115.3), counted across a spell's
/// abilities. A second change in one sentence is
/// [`super::instructions::ContinuousChange`]'s to write, not this word's.
pub fn that_target(ix: usize) -> EffectRecipient {
    EffectRecipient::SameInstanceAs(ix)
}

/// "It", "that creature", "its controller"'s "it": what the effect's one
/// instance of "target" chose. A card with two names which with
/// [`that_target`].
pub fn the_target() -> EffectRecipient {
    that_target(0)
}

/// "Any target" (CR 115.4): a creature, player, planeswalker or battle.
pub fn any_target() -> EffectRecipient {
    target(SelectionFilter::Any)
}

/// "Any other target" (CR 115.4): any target but what the targets before it
/// chose — Arc Trail's second.
pub fn any_other_target() -> EffectRecipient {
    another_target(SelectionFilter::Any)
}

/// "Another target [filter]", "a third target [filter]" (CR 601.2c): one more
/// instance of "target", other than what the targets before it chose. A
/// permanent's "another target permanent" with no target before it is other
/// than the permanent: `target_permanent(another(ObjectFilter::All))`, Flickerwisp's.
pub fn another_target(filter: SelectionFilter) -> EffectRecipient {
    target(SelectionFilter::Another(Box::new(filter)))
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

/// "Each player sacrifices all other creatures they control": every
/// permanent `filter` describes that each of `players` controls, each
/// player's own theirs to act on (CR 701.21a). "Sacrifice [those]" without
/// it is the effect's controller's alone.
pub fn all_their(players: EffectRecipient, filter: ObjectFilter) -> EffectRecipient {
    chosen_by(Choice { chooser: players, among: ChoiceScope::ChoosersPermanents, picks: vec![Pick::all(filter)], acts_on: ChoiceSide::Chosen })
}

// ---------------------------------------------------------------------------
// What a choose step before this one chose (CR 608.2d)
// ---------------------------------------------------------------------------
//
// One word for each thing a later step can read, each written where that kind
// of thing goes: a player or objects as what a verb acts on, a number as an
// amount, a color or a type wherever the card could print one. The step that
// chose it is `super::instructions::choose_player`, `choose_objects`,
// `choose_color` and their siblings.

/// "That player", "the chosen player": the player chosen, while they are in
/// the game.
pub fn the_chosen_player() -> EffectRecipient {
    EffectRecipient::ChosenAs(ChoiceKind::Player)
}

/// "That player" after "choose a second player".
pub fn the_second_chosen_player() -> EffectRecipient {
    EffectRecipient::ChosenAs(ChoiceKind::SecondPlayer)
}

/// "That player" after "choose a third player".
pub fn the_third_chosen_player() -> EffectRecipient {
    EffectRecipient::ChosenAs(ChoiceKind::ThirdPlayer)
}

/// "It", "that creature", "the chosen creatures": the objects chosen, each
/// while it is still the object chosen (CR 400.7), with its share where an
/// amount was divided among them.
pub fn the_chosen_objects() -> EffectRecipient {
    EffectRecipient::ChosenAs(ChoiceKind::Objects)
}

/// "The chosen color", where a card could print a color: Wash Out's
/// "permanents of the color of your choice" is
/// `each(color(the_chosen_color()))`, as "red permanents" is
/// `each(color(Color::Red))`.
pub fn the_chosen_color() -> PrintedOrChosen<Color> {
    PrintedOrChosen::Chosen
}

/// "The chosen creature type", where a card could print one: Kindred
/// Dominance's "creatures that aren't of the chosen type" is
/// `creature().other_than(creature_type(the_chosen_creature_type()))`.
pub fn the_chosen_creature_type() -> PrintedOrChosen<CreatureType> {
    PrintedOrChosen::Chosen
}

/// "A color chosen this way", after each player chose one: any of their
/// answers, where [`the_chosen_color`] is the one a single chooser gave.
pub fn a_color_chosen_this_way() -> PrintedOrChosen<Color> {
    PrintedOrChosen::Chosen
}

/// "A type chosen this way", after each player chose one: Harsh Mercy's
/// "creatures that aren't of a type chosen this way" is
/// `creature().other_than(creature_type(a_creature_type_chosen_this_way()))`.
/// Any of their answers, where [`the_chosen_creature_type`] is the one a
/// single chooser gave.
pub fn a_creature_type_chosen_this_way() -> PrintedOrChosen<CreatureType> {
    PrintedOrChosen::Chosen
}

/// "The chosen card type", where a card could print one: World Queller's
/// "a permanent of their choice of that type" picks among
/// `card_type(the_chosen_card_type())`.
pub fn the_chosen_card_type() -> PrintedOrChosen<CardType> {
    PrintedOrChosen::Chosen
}

/// A color or a type as a card names it: the one it prints, or the one a
/// choose step before this step chose. Each filter word for a quality takes
/// either, as an amount takes a number or "the chosen number".
pub enum PrintedOrChosen<T> {
    Printed(T),
    Chosen,
}

impl<T> From<T> for PrintedOrChosen<T> {
    fn from(printed: T) -> Self {
        PrintedOrChosen::Printed(printed)
    }
}

/// "One of the chosen objects", in a filter: Duneblast's "destroy the rest"
/// is `destroy(each(creature().other_than(one_of_the_chosen())))`.
pub fn one_of_the_chosen() -> ObjectFilter {
    ObjectFilter::ChosenAs(ChoiceKind::Objects)
}

/// "Chosen this way", in a filter, after each player chose objects: one any
/// of them chose. Slaughter the Strong's "all other creatures" is
/// `creature().other_than(chosen_this_way())`, where [`one_of_the_chosen`]
/// reads a single chooser's.
pub fn chosen_this_way() -> ObjectFilter {
    ObjectFilter::ChosenAs(ChoiceKind::Objects)
}

/// "One of the objects the other players chose", in a filter, after
/// `then_each_other_player_chooses_objects`: Fortunate Few's "hasn't been
/// chosen this way" and its "all other" leave these out beside
/// [`one_of_the_chosen`].
pub fn one_the_others_chose() -> ObjectFilter {
    ObjectFilter::ChosenAs(ChoiceKind::ObjectsTheOthersChose)
}

/// "The chosen number", "that many" after a number is chosen, as an amount:
/// Expel the Interlopers' "power greater than or equal to the chosen number"
/// is `power_at_least(the_chosen_number())`.
pub fn the_chosen_number() -> AmountExpr {
    AmountExpr::ChosenAs(ChoiceKind::Number)
}

// ---------------------------------------------------------------------------
// The player who does a resolving instruction
// ---------------------------------------------------------------------------

/// "Its controller" (CR 108.4): the player who controls what `it` names as
/// the instruction applies, Mana Leak's target spell's.
pub fn its_controller(it: EffectRecipient) -> PlayerRef {
    PlayerRef::NamedBy(Box::new(it))
}

/// "That player" on a triggered ability: the player of its triggering
/// event, ward's opponent whose spell or ability targeted (CR 702.21a).
pub fn that_player() -> PlayerRef {
    PlayerRef::NamedBy(Box::new(EffectRecipient::TriggeringPlayer))
}

/// "Any player" (CR 118.12a's "unless any player pays"): each player, asked
/// in APNAP order (CR 101.4) until one does.
pub fn any_player() -> PlayerRef {
    PlayerRef::AnyPlayer
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
/// others it is; or of the color a choose step chose.
pub fn color(which: impl Into<PrintedOrChosen<Color>>) -> ObjectFilter {
    match which.into() {
        PrintedOrChosen::Printed(color) => ObjectFilter::ByColor(color),
        PrintedOrChosen::Chosen => ObjectFilter::ChosenAs(ChoiceKind::Color),
    }
}

/// "Elf", "Goblin" (CR 205.3m): an object of the creature type, or of the
/// creature type a choose step chose.
pub fn creature_type(which: impl Into<PrintedOrChosen<CreatureType>>) -> ObjectFilter {
    match which.into() {
        PrintedOrChosen::Printed(creature_type) => ObjectFilter::BySubtype(Subtype::Creature(creature_type)),
        PrintedOrChosen::Chosen => ObjectFilter::ChosenAs(ChoiceKind::CreatureType),
    }
}

/// "Artifact", "enchantment" (CR 205.2a): an object of the card type, or of
/// the card type a choose step chose.
pub fn card_type(which: impl Into<PrintedOrChosen<CardType>>) -> ObjectFilter {
    match which.into() {
        PrintedOrChosen::Printed(card_type) => ObjectFilter::ByType(card_type),
        PrintedOrChosen::Chosen => ObjectFilter::ChosenAs(ChoiceKind::CardType),
    }
}

/// "You control" (CR 110.2), read against the effect's controller (CR
/// 109.5): off the battlefield, its owner (CR 108.4a), so a card in a
/// graveyard means its owner's.
pub fn you_control() -> ObjectFilter {
    ObjectFilter::ByController(PlayerRef::You)
}

/// "They control", in what a choose step chooses among: the player the
/// answer is for, which is each player in turn for "each player chooses a
/// creature they control", and the one chooser for "target opponent chooses
/// a creature they control". Beside [`you_control`], the effect's
/// controller, which the same text may also name. "They don't control" is
/// `non(they_control())`.
pub fn they_control() -> ObjectFilter {
    ObjectFilter::ByController(PlayerRef::Named(PlayerAntecedent::ChoosingFor))
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

/// "With power 3 or greater", "with power greater than or equal to the
/// chosen number" (CR 208).
pub fn power_at_least(amount: impl Into<AmountExpr>) -> ObjectFilter {
    ObjectFilter::WithValue { of: ObjectValue::Power, is: Box::new(ValueIs::AtLeast(amount.into())) }
}

/// "With power 2 or less" (CR 208).
pub fn power_at_most(amount: impl Into<AmountExpr>) -> ObjectFilter {
    ObjectFilter::WithValue { of: ObjectValue::Power, is: Box::new(ValueIs::AtMost(amount.into())) }
}

/// "With toughness 4 or greater" (CR 208).
pub fn toughness_at_least(amount: impl Into<AmountExpr>) -> ObjectFilter {
    ObjectFilter::WithValue { of: ObjectValue::Toughness, is: Box::new(ValueIs::AtLeast(amount.into())) }
}

/// "With toughness 2 or less" (CR 208).
pub fn toughness_at_most(amount: impl Into<AmountExpr>) -> ObjectFilter {
    ObjectFilter::WithValue { of: ObjectValue::Toughness, is: Box::new(ValueIs::AtMost(amount.into())) }
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
