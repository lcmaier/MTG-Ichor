use std::collections::HashSet;
use std::sync::Arc;

use crate::types::card_types::{CardType, CardTypes, Supertype, Subtype, Subtypes};
use crate::types::colors::Color;
use crate::types::costs::{AdditionalCost, AlternativeCost, Cost};
use crate::types::cost_modification::{CostChange, CostModificationDef};
use crate::types::effects::{ObjectSet, AmountExpr, Effect, ManaOutput, ObjectFilter, PlayerRef, Primitive, EffectRecipient, SelectionFilter, TargetCount};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::ids::AbilityId;

/// The immutable "printed card" definition.
///
/// This is what's printed on the physical card — it never changes at runtime.
/// Game objects reference a CardData, and the layer system computes effective
/// characteristics on top of it.
#[derive(Debug, Clone, PartialEq)]
pub struct CardData {
    pub name: String,
    pub mana_cost: Option<ManaCost>,
    pub colors: HashSet<Color>,
    pub types: CardTypes,
    pub supertypes: HashSet<Supertype>,
    pub subtypes: Subtypes,
    /// The card's rules text (CR 207.1): its Oracle text without the reminder
    /// text, which has no game function (CR 207.2). Each ability carries its
    /// own paragraph of it as [`AbilityDef::rules_text`].
    pub rules_text: String,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub loyalty: Option<i32>,
    pub defense: Option<i32>,
    /// Shared with every frame the layer walk seeds from this card, and
    /// written only through `Arc::make_mut`, so a walk is a refcount bump
    /// rather than a clone of the ability tree (`layers-architecture.md` §12).
    pub abilities: Arc<Vec<AbilityDef>>,
    pub keyword_flags: HashSet<KeywordFlag>,
    /// Color indicator (rule 204) — used for cards with no mana cost that have
    /// an intrinsic color (e.g., back faces of DFCs, Ancestral Vision suspend).
    /// None means no color indicator; color is derived from mana cost instead.
    pub color_indicator: Option<Vec<Color>>,
    /// What this Aura can legally enchant (rule 303.4).
    /// None for non-Aura cards.
    pub enchant_filter: Option<SelectionFilter>,
    /// CR 601.2c's instances of "target" for this card cast as a spell, in
    /// printed order: those of [`spell_effect`], across all its spell
    /// abilities — except for an Aura, whose one instance is its enchant
    /// ability (CR 303.4a) and sits in no effect tree. One field, one rule:
    /// the castability pre-check, the announcement and CR 608.2b's re-check
    /// all read this, so all three see `enchant_filter`.
    ///
    /// Written by `CardDataBuilder::build` and nothing else, which its
    /// privacy keeps true. A `CardData` is built once and shared behind an
    /// `Arc`, so this is computed once per card rather than once per card in
    /// hand per priority pass.
    spell_instances: Vec<EffectRecipient>,
    /// Alternative costs this card can be cast for (rule 118.9).
    /// A player may choose at most one when casting.
    pub alternative_costs: Vec<AlternativeCost>,
    /// Additional costs this card can optionally pay (rule 118.8).
    /// Multiple may be paid in a single cast (e.g. kicker + buyback).
    pub additional_costs: Vec<AdditionalCost>,
    /// "X can't be 0" (Ertai's Meddling, Mind Grind): the least X its caster
    /// may announce for its mana cost's X is 1 (CR 601.2b).
    pub x_cant_be_zero: bool,
}

impl CardData {
    /// CR 601.2c's instances of "target" for this card cast as a spell, in
    /// printed order: its spell abilities', or an Aura's enchant ability
    /// (CR 303.4a).
    pub fn spell_instances(&self) -> &[EffectRecipient] {
        &self.spell_instances
    }
}

/// The kind of an ability (CR 113.3, 605.1a). The two activated kinds carry
/// what an activation says besides its effect, and no other kind can.
#[derive(Debug, Clone, PartialEq)]
pub enum AbilityType {
    /// An activated mana ability (CR 605.1a), which doesn't use the stack.
    Mana(Activation),
    /// An activated ability (CR 113.3b).
    Activated(Activation),
    /// A triggered ability (when/whenever/at)
    Triggered,
    /// A static ability (continuous effect)
    Static,
    /// A spell ability (the effect of an instant/sorcery)
    Spell,
}

/// What an activated ability's text says besides its effect: "[Cost]:
/// [Effect.] [Activation instructions (if any).]" (CR 602.1). A mana ability
/// is an activated ability (CR 605.1a), so CR 602.5's restrictions reach it
/// as they reach any other. `permission-design` grows what this holds
/// (`permission-architecture.md#3-what-it-must-express-at-v1`): who may
/// activate, and every restriction a card prints beside its timing.
#[derive(Debug, Clone, PartialEq)]
pub struct Activation {
    /// The activation cost, everything before the colon (CR 602.1a).
    pub costs: Vec<Cost>,
    /// When it may be activated beyond having priority (CR 602.5).
    pub restriction: ActivationRestriction,
}

impl Activation {
    /// `costs`, activated at any time its controller has priority.
    pub fn new(costs: Vec<Cost>) -> Activation {
        Activation { costs, restriction: ActivationRestriction::None }
    }

    /// This activation, only as a sorcery (CR 602.5d).
    pub fn only_as_sorcery(self) -> Activation {
        Activation { restriction: ActivationRestriction::OnlyAsSorcery, ..self }
    }
}

/// An ability word given its cost alone: activated whenever its controller
/// has priority.
impl From<Vec<Cost>> for Activation {
    fn from(costs: Vec<Cost>) -> Activation {
        Activation::new(costs)
    }
}

/// When an activated ability may be activated (CR 602.5d), beyond having
/// priority.
///
/// One value: Equip (CR 702.6a) reads "Activate only as a sorcery", and that
/// is the whole of what this enum can say. `permission-design` owns the rest
/// — "activate only once each turn", "only during combat", a condition — and
/// grows it when a card needs it.
///
/// Honored at all three ability-index sites CLAUDE.md names:
/// `activatable_abilities` does not offer a restricted ability out of its
/// window, `activate_ability` refuses it (the enforcement), and
/// `priority.rs` reaches the second through the first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationRestriction {
    /// Any time the player has priority (CR 602.1, the default).
    None,
    /// CR 602.5d — "the player must follow the timing rules for casting a
    /// sorcery spell, though the ability isn't actually a sorcery": active
    /// player, main phase, empty stack.
    OnlyAsSorcery,
}

/// Definition of a single ability on a card.
///
/// This is the printed ability — at runtime, activated/triggered abilities
/// become objects on the stack with their own identity.
#[derive(Debug, Clone, PartialEq)]
pub struct AbilityDef {
    pub id: AbilityId,
    /// The ability's own text: its paragraph of the card's rules text (CR
    /// 113.2c makes each paragraph one ability, keyword lists aside), which
    /// carries no reminder text (CR 207.2). An ability no text box prints
    /// carries the words of the rule that gives it: CR 305.6's "{T}: Add [mana
    /// symbol]." and CR 306.5b's loyalty ability. A granted ability carries
    /// the text its card quotes, and a copy carries the copied one (CR 707.2),
    /// so the window reads what an object's abilities say off its effective
    /// list.
    ///
    /// Empty only on a test fixture: `cards::registry`'s tests hold every
    /// registered card's abilities to its text.
    pub rules_text: AbilityText,
    pub ability_type: AbilityType,
    pub effect: Effect,
    /// CR 601.2c's instances of "target" that `effect` declares, in printed
    /// order — [`Effect::instances`], stored, since every priority check
    /// reads it ([`Self::instances`]).
    ///
    /// Private, so nothing but this module writes it: [`Self::new`] reads it
    /// off the effect it is given, so an ability built at run time has its
    /// list from birth, and `CardDataBuilder::build` reads it again on every
    /// def it reaches. `effect` stays public, so a write to it keeps this
    /// list right only by keeping the effect's instances; the one such write,
    /// `authoring::functioning_in`'s wrap, does (CR 113.6b's zone statement
    /// declares no target).
    instances: Vec<EffectRecipient>,
    /// CR 604.3 — this ability is a characteristic-defining ability.
    ///
    /// CR 604.3a lists five criteria. Four of them are properties of the
    /// ability's *text* and are what the card author asserts by setting this:
    /// (1) it defines colors, subtypes, power, or toughness; (3) it doesn't
    /// directly affect any other object's characteristics; (4) it isn't an
    /// ability the object grants itself; (5) it doesn't set those values only
    /// under a condition.
    ///
    /// Criterion (2) is *provenance* — printed on the object it affects,
    /// granted to a token by the effect that created it, or acquired by a copy
    /// or text-changing effect — and no bool on a definition can state it,
    /// because the same `AbilityDef` can reach an object by any route. It is
    /// maintained instead by whoever writes the ability onto an object:
    ///
    /// - Printed abilities keep whatever the card author wrote. ✅
    /// - A copy effect (Layer 1) or a text-changing effect (Layer 3) hands the
    ///   whole `AbilityDef` over, so the flag rides along — which is exactly
    ///   what 604.3a(2) asks for. ✅
    /// - **A Layer 6 `GrantAbility` clears this on the def it grants**, in
    ///   `layers::compute`'s grant arm: a Layer 6 grant is none of 604.3a(2)'s
    ///   routes, so the ability is never a CDA there, however its text reads.
    ///   A token's abilities are its `CardData`'s, 604.3a(2)'s second route,
    ///   and keep the flag as printed ones do.
    ///
    /// Read by `engine::layers::cda`, which applies CDAs off the object's own
    /// effective ability list. CDAs are never registered as continuous effects
    /// — see `Layer::Layer7aCdaPT`.
    pub is_characteristic_defining: bool,
}

impl AbilityDef {
    /// An ability of `ability_type` whose words are `rules_text` (CR 113.2c)
    /// and whose effect is `effect`, not characteristic-defining: what
    /// `cards::authoring`'s ability words build, and what the engine builds
    /// an ability it makes with. Its id is [`AbilityId::UNASSIGNED`] until
    /// `CardDataBuilder::build` stamps it or [`Self::with_id`] sets it.
    pub fn new(ability_type: AbilityType, rules_text: impl Into<AbilityText>, effect: Effect) -> AbilityDef {
        AbilityDef {
            id: AbilityId::UNASSIGNED,
            rules_text: rules_text.into(),
            ability_type,
            instances: effect.instances(),
            effect,
            is_characteristic_defining: false,
        }
    }

    /// What an activated ability's text says besides its effect (CR 602.1),
    /// for either activated kind; `None` for the others.
    pub fn activation(&self) -> Option<&Activation> {
        match &self.ability_type {
            AbilityType::Activated(activation) | AbilityType::Mana(activation) => Some(activation),
            AbilityType::Triggered | AbilityType::Static | AbilityType::Spell => None,
        }
    }

    /// The activation cost (CR 602.1a): an activated ability's, and none for
    /// any other kind, which has none to pay.
    pub fn costs(&self) -> &[Cost] {
        self.activation().map_or(&[], |activation| &activation.costs)
    }

    /// CR 601.2c's instances of "target" its effect declares, in printed
    /// order.
    pub fn instances(&self) -> &[EffectRecipient] {
        &self.instances
    }

    /// CR 305.6's "{T}: Add [mana symbol].", the ability a basic land type
    /// gives and a printed card's with the same words: one function, so the
    /// two stay one ability (`tap_to_add_text`).
    pub fn tap_for(mana_type: ManaType) -> AbilityDef {
        let produce = Primitive::ProduceMana(ManaOutput { mana: vec![(mana_type, AmountExpr::Fixed(1))], special: vec![] });
        let kind = AbilityType::Mana(Activation::new(vec![Cost::TapSelf]));
        AbilityDef::new(kind, tap_to_add_text(mana_type), Effect::Atom(produce, EffectRecipient::Implicit))
    }

    /// This ability, with `id` in place of the one the builder would stamp:
    /// a test that reads its ability back by id, or an ability the engine
    /// derives an id for.
    pub fn with_id(mut self, id: AbilityId) -> AbilityDef {
        self.id = id;
        self
    }

    /// This ability, as one that defines a characteristic (CR 604.3).
    pub fn characteristic_defining(mut self) -> AbilityDef {
        self.is_characteristic_defining = true;
        self
    }

}

/// An ability's words, and which paragraph of its card's rules text they are.
/// A card file writes the words (`"Flying".into()`); `CardDataBuilder::build`
/// finds the paragraph. Words alone cannot say which paragraph an ability
/// is: CR 603.11 prints a static ability and the triggered abilities linked
/// to it as one paragraph, while Seeds of Strength prints one paragraph three
/// times as three abilities (CR 113.2c).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbilityText {
    pub words: &'static str,
    paragraph: Option<u16>,
}

impl AbilityText {
    /// Which paragraph of its card's rules text, counted from zero, the
    /// ability prints as. `None` for one no paragraph prints: CR 305.6's
    /// mana ability, a granted ability, a test fixture's.
    pub fn paragraph(self) -> Option<u16> {
        self.paragraph
    }
}

impl From<&'static str> for AbilityText {
    fn from(words: &'static str) -> AbilityText {
        AbilityText { words, paragraph: None }
    }
}

// --- Builder Pattern ---

/// Builder for constructing CardData with a fluent API.
///
/// # Example
/// ```
/// use mtgsim::objects::card_data::CardDataBuilder;
/// use mtgsim::types::card_types::{CardType, Supertype, Subtype, LandType};
/// use mtgsim::types::mana::ManaType;
///
/// let forest = CardDataBuilder::new("Forest")
///     .card_type(CardType::Land)
///     .supertype(Supertype::Basic)
///     .subtype(Subtype::Land(LandType::Forest))
///     .mana_ability_single(ManaType::Green)
///     .build();
/// ```
pub struct CardDataBuilder {
    data: CardData,
}

impl CardDataBuilder {
    pub fn new(name: &str) -> Self {
        CardDataBuilder {
            data: CardData {
                name: name.to_string(),
                mana_cost: None,
                colors: HashSet::new(),
                types: CardTypes::new(),
                supertypes: HashSet::new(),
                subtypes: Subtypes::new(),
                rules_text: String::new(),
                power: None,
                toughness: None,
                loyalty: None,
                defense: None,
                abilities: Arc::new(Vec::new()),
                keyword_flags: HashSet::new(),
                color_indicator: None,
                enchant_filter: None,
                spell_instances: Vec::new(),
                alternative_costs: Vec::new(),
                additional_costs: Vec::new(),
                x_cant_be_zero: false,
            },
        }
    }

    pub fn mana_cost(mut self, cost: ManaCost) -> Self {
        self.data.mana_cost = Some(cost);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.data.colors.insert(color);
        self
    }

    pub fn card_type(mut self, card_type: CardType) -> Self {
        self.data.types.insert(card_type);
        self
    }

    pub fn supertype(mut self, supertype: Supertype) -> Self {
        self.data.supertypes.insert(supertype);
        self
    }

    pub fn subtype(mut self, subtype: Subtype) -> Self {
        self.data.subtypes.insert(subtype);
        self
    }

    pub fn rules_text(mut self, text: &str) -> Self {
        self.data.rules_text = text.to_string();
        self
    }

    pub fn power_toughness(mut self, power: i32, toughness: i32) -> Self {
        self.data.power = Some(power);
        self.data.toughness = Some(toughness);
        self
    }

    pub fn loyalty(mut self, loyalty: i32) -> Self {
        self.data.loyalty = Some(loyalty);
        self
    }

    pub fn defense(mut self, defense: i32) -> Self {
        self.data.defense = Some(defense);
        self
    }

    pub fn keyword_flag(mut self, keyword: KeywordFlag) -> Self {
        self.data.keyword_flags.insert(keyword);
        self
    }

    /// **Affinity for [text]** — CR 702.41a: "This spell costs {1} less to
    /// cast for each [text] you control."
    ///
    /// A keyword that is not a [`KeywordFlag`]: the rule *defines* affinity as
    /// that sentence, so the card carries the sentence and nothing in the
    /// engine knows the word — a static ability whose subject is the spell
    /// itself (CR 113.6d) and whose change is a generic reduction of a count.
    ///
    /// `filter` is the "[text]" — `ObjectFilter::ByType(CardType::Artifact)`
    /// for affinity for artifacts. "You control" is added here, so a card
    /// writes the noun and no more. CR 702.41b — "if a spell has multiple
    /// instances of affinity, each of them applies" — is calling this twice.
    pub fn affinity_for(self, rules_text: &'static str, filter: ObjectFilter) -> Self {
        let you_control = ObjectFilter::And(
            Box::new(filter),
            Box::new(ObjectFilter::ByController(PlayerRef::You)),
        );
        self.ability(
            CostModificationDef::itself(CostChange::ReduceGeneric(AmountExpr::CountOf(
                ObjectSet::battlefield_filter(you_control),
            )))
            .into_ability(rules_text),
        )
    }

    pub fn color_indicator(mut self, colors: Vec<Color>) -> Self {
        self.data.color_indicator = Some(colors);
        self
    }

    pub fn ability(mut self, ability: AbilityDef) -> Self {
        Arc::make_mut(&mut self.data.abilities).push(ability);
        self
    }

    /// Shorthand: add a mana ability that taps to produce one mana of the given type.
    /// This is the standard basic land ability.
    pub fn mana_ability_single(mut self, mana_type: ManaType) -> Self {
        Arc::make_mut(&mut self.data.abilities).push(AbilityDef::tap_for(mana_type));
        self
    }

    pub fn enchant_filter(mut self, filter: SelectionFilter) -> Self {
        self.data.enchant_filter = Some(filter);
        self
    }

    pub fn alternative_cost(mut self, cost: AlternativeCost) -> Self {
        self.data.alternative_costs.push(cost);
        self
    }

    pub fn additional_cost(mut self, cost: AdditionalCost) -> Self {
        self.data.additional_costs.push(cost);
        self
    }

    /// "X can't be 0."
    pub fn x_cant_be_zero(mut self) -> Self {
        self.data.x_cant_be_zero = true;
        self
    }

    /// Finish the card, giving every ability def reachable from it an id and
    /// its instances of "target".
    ///
    /// A def still carrying `AbilityId::UNASSIGNED` — what every card file
    /// writes — gets `AbilityId::printed(name, ordinal)`. The printed list
    /// takes ordinals `0..n` in order, so a printed ability's ordinal is its
    /// index in `abilities`, the index `activatable_abilities` hands out; the
    /// defs nested in the printed effects (a granted ability, a token's
    /// abilities) follow from `n`, in `Effect::for_each_ability_def_mut`'s
    /// order. A def that already has an id keeps it, which is what lets a
    /// test author one and read it back through the card.
    ///
    /// `AbilityDef::instances` is overwritten on every def reached, id or no
    /// id, and `CardData::spell_instances` is filled last: from the enchant
    /// ability for an Aura (CR 303.4a), otherwise from the printed spell
    /// abilities' [`spell_effect`], otherwise empty — a permanent spell
    /// announces nothing.
    ///
    /// A printed def's paragraph is the first paragraph with its words that
    /// no earlier def took, or, when none is left, the paragraph of the def
    /// before it with the same words: CR 603.11's abilities that share one
    /// paragraph are written as neighbors.
    pub fn build(mut self) -> Arc<CardData> {
        let name = self.data.name.clone();
        let mut ordinal = 0u32;
        let mut stamp = |def: &mut AbilityDef| {
            if def.id == AbilityId::UNASSIGNED {
                def.id = AbilityId::printed(&name, ordinal);
            }
            ordinal += 1;
            def.instances = def.effect.instances();
        };
        let rules_text = &self.data.rules_text;
        let abilities = Arc::make_mut(&mut self.data.abilities);
        for at in 0..abilities.len() {
            let (earlier, rest) = abilities.split_at_mut(at);
            let def = &mut rest[0];
            stamp(def);
            let words = def.rules_text.words;
            // Defs take the paragraphs with their words in printed order.
            let last_taken = earlier
                .iter()
                .filter(|other| other.rules_text.words == words)
                .filter_map(|other| other.rules_text.paragraph)
                .max();
            let untaken = paragraphs(rules_text)
                .enumerate()
                .filter(|&(index, paragraph)| paragraph == words && last_taken.is_none_or(|last| index > usize::from(last)))
                .find_map(|(index, _)| u16::try_from(index).ok());
            let neighbor = earlier.last().map(|other| other.rules_text).filter(|text| text.words == words);
            def.rules_text.paragraph = untaken.or(neighbor.and_then(AbilityText::paragraph));
        }
        for def in abilities.iter_mut() {
            def.effect.for_each_ability_def_mut(&mut stamp);
        }
        // CR 702.5a — only an Aura carries an enchant ability, and CR 303.4a
        // makes it the spell's one instance of "target".
        self.data.spell_instances = match &self.data.enchant_filter {
            Some(filter) => vec![EffectRecipient::Target(filter.clone(), TargetCount::Exactly(1))],
            None => spell_effect(abilities).map(|effect| effect.instances()).unwrap_or_default(),
        };
        Arc::new(self.data)
    }
}

/// The effect a spell resolves with: its spell abilities in printed order.
/// Where among a card's printed `abilities` the ability `instance` is an
/// instance of is printed: a granted, copied or printed instance names its
/// definition, whichever it is.
pub fn printed_place(abilities: &[AbilityDef], instance: crate::types::ids::AbilityId) -> Option<usize> {
    abilities.iter().position(|a| a.id.definition() == instance.definition())
}

/// Each paragraph of an instant's or sorcery's text is a spell ability of its
/// own (CR 113.2c), and the spell follows them in the order written (CR
/// 608.2c), so several are one `Sequence`, whose instances of "target" are
/// counted across the whole spell (CR 601.2c). One ability is its effect as
/// written; a card with none has no spell effect.
pub fn spell_effect(abilities: &[AbilityDef]) -> Option<Effect> {
    let mut effects: Vec<Effect> = abilities
        .iter()
        .filter(|ability| ability.ability_type == AbilityType::Spell)
        .map(|ability| ability.effect.clone())
        .collect();
    match effects.len() {
        0 => None,
        1 => effects.pop(),
        _ => Some(Effect::Sequence(effects)),
    }
}

/// A spell's modes (CR 700.2): the modal node of its one spell ability. A
/// spell of several paragraphs is their `Sequence`, which carries no node at
/// its root (`Effect::modal`), so it has none. Read without building
/// [`spell_effect`]'s copy, since castability asks it of every card in hand.
pub fn spell_modes(abilities: &[AbilityDef]) -> Option<&crate::types::effects::ModalNode> {
    let mut spell_abilities = abilities.iter().filter(|ability| ability.ability_type == AbilityType::Spell);
    let only = spell_abilities.next()?;
    if spell_abilities.next().is_some() {
        return None;
    }
    only.effect.modal()
}

/// A card's rules text cut into paragraphs, each one ability (CR 113.2c): a
/// line, with the mode lines after it, since a modal ability's modes are
/// part of its paragraph (CR 700.2). A mode line starts with a bullet, a
/// spree mode's plus sign (CR 702.172b) or a pawprint mode's {P} (700.2i).
pub fn paragraphs(rules_text: &str) -> impl Iterator<Item = &str> {
    let is_a_mode = |line: &str| line.starts_with('•') || line.starts_with("+ ") || line.starts_with("{P}");
    let mut rest = Some(rules_text).filter(|text| !text.is_empty());
    std::iter::from_fn(move || {
        let text = rest?;
        let end = text.match_indices('\n').map(|(at, _)| at).find(|&at| !is_a_mode(&text[at + 1..]));
        match end {
            Some(at) => {
                rest = Some(&text[at + 1..]);
                Some(&text[..at])
            }
            None => {
                rest = None;
                Some(text)
            }
        }
    })
}

/// "{T}: Add {G}." — CR 305.6's words for the mana ability a basic land type
/// gives, and a printed card's for the same ability ("{T}: Add {C}." too).
/// One function for both, so a printed basic land's ability and the intrinsic
/// one `layers::land_types` gives stay the same ability.
pub(crate) fn tap_to_add_text(mana_type: ManaType) -> &'static str {
    match mana_type {
        ManaType::White => "{T}: Add {W}.",
        ManaType::Blue => "{T}: Add {U}.",
        ManaType::Black => "{T}: Add {B}.",
        ManaType::Red => "{T}: Add {R}.",
        ManaType::Green => "{T}: Add {G}.",
        ManaType::Colorless => "{T}: Add {C}.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::card_types::LandType;

    #[test]
    fn test_build_basic_land() {
        let forest = CardDataBuilder::new("Forest")
            .card_type(CardType::Land)
            .supertype(Supertype::Basic)
            .subtype(Subtype::Land(LandType::Forest))
            .mana_ability_single(ManaType::Green)
            .build();

        assert_eq!(forest.name, "Forest");
        assert!(forest.types.contains(&CardType::Land));
        assert!(forest.supertypes.contains(&Supertype::Basic));
        assert!(forest.mana_cost.is_none());
        assert_eq!(forest.abilities.len(), 1);
        assert!(matches!(forest.abilities[0].ability_type, AbilityType::Mana(_)));
        assert_eq!(forest.abilities[0].rules_text.words, "{T}: Add {G}.");
    }

    /// The builder finds each printed ability's paragraph (CR 113.2c): Seeds
    /// of Strength prints one paragraph three times as three abilities, the
    /// first, second and third; Platinum Angel's one paragraph after Flying
    /// is its one ability, two "can't"s; CR 603.11's two abilities in one
    /// paragraph share it; a basic land's mana ability is printed only as
    /// reminder text, so it has none. A modal ability's paragraph takes its
    /// bullets (CR 700.2).
    #[test]
    fn each_printed_ability_knows_its_paragraph() {
        let found = |card: Arc<CardData>| -> Vec<Option<u16>> {
            card.abilities.iter().map(|ability| ability.rules_text.paragraph()).collect()
        };
        assert_eq!(found(crate::cards::printings::seeds_of_strength()), [Some(0), Some(1), Some(2)]);
        assert_eq!(found(crate::cards::printings::platinum_angel()), [Some(1)]);
        assert_eq!(found(crate::test_support::one_paragraph_two_abilities()), [Some(0), Some(0)]);
        assert_eq!(found(crate::cards::printings::forest()), [None]);
        assert_eq!(
            paragraphs("Choose one —\n• Draw a card.\n• You gain 2 life.\nFlying").collect::<Vec<_>>(),
            ["Choose one —\n• Draw a card.\n• You gain 2 life.", "Flying"]
        );
    }

    /// Each paragraph of a spell is its own spell ability (CR 113.2c), and
    /// the spell resolves them as one effect in printed order: Opt scries,
    /// then draws. Seeds of Strength's three abilities announce their three
    /// instances of "target" as the spell's (CR 601.2c). One ability is its
    /// own effect, and a creature card has none.
    #[test]
    fn a_spell_resolves_its_abilities_in_printed_order_as_one_effect() {
        let opt = crate::cards::printings::opt();
        assert_eq!(opt.abilities[0].rules_text.words, "Scry 1.");
        assert_eq!(
            spell_effect(&opt.abilities),
            Some(Effect::Sequence(vec![opt.abilities[0].effect.clone(), opt.abilities[1].effect.clone()]))
        );
        let seeds = crate::cards::printings::seeds_of_strength();
        assert_eq!(seeds.abilities.len(), 3);
        assert_eq!(seeds.spell_instances.len(), 3);
        let bolt = crate::cards::printings::lightning_bolt();
        assert_eq!(spell_effect(&bolt.abilities), Some(bolt.abilities[0].effect.clone()));
        assert_eq!(spell_effect(&crate::cards::printings::grizzly_bears().abilities), None);
    }

    #[test]
    fn test_build_creature() {
        let bears = CardDataBuilder::new("Grizzly Bears")
            .mana_cost(ManaCost::build(&[ManaType::Green], 1))
            .color(Color::Green)
            .card_type(CardType::Creature)
            .subtype(Subtype::Creature(crate::types::card_types::CreatureType::Bear))
            .power_toughness(2, 2)
            .build();

        assert_eq!(bears.name, "Grizzly Bears");
        assert_eq!(bears.mana_cost.as_ref().unwrap().mana_value(), 2);
        assert!(bears.types.contains(&CardType::Creature));
        assert_eq!(bears.power, Some(2));
        assert_eq!(bears.toughness, Some(2));
    }

    #[test]
    fn test_card_data_color_indicator_none_default() {
        let card = CardDataBuilder::new("Test Card").build();
        assert!(card.color_indicator.is_none());
    }

    #[test]
    fn test_card_data_color_indicator_set() {
        let card = CardDataBuilder::new("Archangel Avacyn")
            .color_indicator(vec![Color::Red])
            .build();
        let indicator = card.color_indicator.as_ref().unwrap();
        assert_eq!(indicator.len(), 1);
        assert_eq!(indicator[0], Color::Red);

        // Multi-color indicator
        let card2 = CardDataBuilder::new("Nicol Bolas Back")
            .color_indicator(vec![Color::Blue, Color::Black, Color::Red])
            .build();
        let indicator2 = card2.color_indicator.as_ref().unwrap();
        assert_eq!(indicator2.len(), 3);
    }

    #[test]
    fn test_card_data_default_no_costs() {
        let card = CardDataBuilder::new("Vanilla Creature").build();
        assert!(card.alternative_costs.is_empty());
        assert!(card.additional_costs.is_empty());
    }

    #[test]
    fn test_card_data_with_kicker() {
        let card = CardDataBuilder::new("Goblin Bushwhacker")
            .card_type(CardType::Creature)
            .mana_cost(ManaCost::build(&[ManaType::Red], 0))
            .additional_cost(AdditionalCost::Kicker(vec![Cost::Mana(
                ManaCost::build(&[ManaType::Red], 0),
            )]))
            .build();

        assert_eq!(card.additional_costs.len(), 1);
        assert!(matches!(&card.additional_costs[0], AdditionalCost::Kicker(_)));
        assert!(card.alternative_costs.is_empty());
    }

    #[test]
    fn test_card_data_with_alternative_cost() {
        let card = CardDataBuilder::new("Force of Will")
            .card_type(CardType::Instant)
            .alternative_cost(AlternativeCost::Custom(
                "Exile a blue card and pay 1 life".to_string(),
                vec![Cost::PayLife(1)],
            ))
            .build();

        assert_eq!(card.alternative_costs.len(), 1);
        assert!(matches!(&card.alternative_costs[0], AlternativeCost::Custom(_, _)));
        assert!(card.additional_costs.is_empty());
    }
}
