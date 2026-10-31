//! CR 113.3's kinds of ability, and CR 605.1a's activated mana ability, as
//! the words a card file writes: an ability is its words (CR 113.2c), its
//! effect, and for an activated one its cost. [`AbilityDef::new`] builds each,
//! so the builder's stamps (its id, its instances of "target") are written
//! in one place. CR 603's triggered ability is [`super::triggered_ability`].

use crate::objects::card_data::{AbilityDef, AbilityText, AbilityType};
use crate::types::card_types::CardType;
use crate::types::costs::Cost;
use crate::types::effects::{Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive, SelectionFilter, TargetCount};
use crate::types::replacement::ReplacementDef;

/// A spell ability (CR 113.3a): one paragraph of an instant's or sorcery's
/// text, followed as the spell resolves.
pub fn spell_ability(rules_text: impl Into<AbilityText>, effect: Effect) -> AbilityDef {
    AbilityDef::new(AbilityType::Spell, rules_text, effect)
}

/// A static ability (CR 113.3d): a statement, true while the ability
/// functions.
pub fn static_ability(rules_text: impl Into<AbilityText>, effect: Effect) -> AbilityDef {
    AbilityDef::new(AbilityType::Static, rules_text, effect)
}

/// A static ability whose effect is a replacement effect (CR 614.1a): never
/// a resolution, so it carries no `Duration`, and it is re-derived off its
/// source's effective ability list on every gather.
pub fn static_replacement(rules_text: impl Into<AbilityText>, def: ReplacementDef) -> AbilityDef {
    static_ability(rules_text, Effect::Replacement(Box::new(def)))
}

/// An activated ability (CR 113.3b): "[Cost]: [Effect.]"
pub fn activated_ability(rules_text: impl Into<AbilityText>, costs: Vec<Cost>, effect: Effect) -> AbilityDef {
    let mut ability = AbilityDef::new(AbilityType::Activated, rules_text, effect);
    ability.costs = costs;
    ability
}

/// An activated mana ability (CR 605.1a): no target, it could add mana, and
/// no loyalty cost.
pub fn mana_ability(rules_text: impl Into<AbilityText>, costs: Vec<Cost>, effect: Effect) -> AbilityDef {
    let mut ability = AbilityDef::new(AbilityType::Mana, rules_text, effect);
    ability.costs = costs;
    ability
}

/// "Equip [cost]" — CR 702.6a's activated ability, spelled out: "[Cost]:
/// Attach this permanent to target creature you control. Activate only as a
/// sorcery."
///
/// `costs` is the whole `[Cost]`, not a generic count: Improvised Arsenal's
/// equip is `{R}`, Vorpal Sword's `{B}{B}`, 33 printed Equipment mix generic
/// and colored pips, and the "Equip—[cost]" form is any cost at all — Dark
/// Knight's Greatsword's "Equip—Pay 3 life" (whose "Activate only once each
/// turn" is CR 602.5b, `permission-architecture.md`). Phyrexian and hybrid
/// pips (Lashwrithe) are `hybrid-and-phyrexian-payment`'s.
///
/// The recipient carries the whole of "creature you control", so CR 608.2b's
/// re-check at resolution is what CR 301.5b means by "control of the creature
/// matters ... when it resolves": a creature stolen in response is an illegal
/// target and the Equipment doesn't move (CR 701.3b). An equip quality
/// (CR 702.6c, "Equip Knight {1}") is one more conjunct in that filter, not
/// a new shape; no registered card has one yet.
pub fn equip(rules_text: &'static str, costs: Vec<Cost>) -> AbilityDef {
    let creature_you_control = ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    );
    activated_ability(
        rules_text,
        costs,
        Effect::Atom(
            Primitive::Attach,
            EffectRecipient::Target(SelectionFilter::Permanent(creature_you_control), TargetCount::Exactly(1)),
        ),
    )
    .only_as_sorcery()
}
