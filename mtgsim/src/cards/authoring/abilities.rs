//! CR 113.3's kinds of ability, and CR 605.1a's activated mana ability, as
//! the words a card file writes: an ability is its words (CR 113.2c), its
//! effect, and for an activated one its cost. [`AbilityDef::new`] builds each,
//! so the builder's stamps (its id, its instances of "target") are written
//! in one place. CR 603's triggered ability is [`super::triggered_ability`].

use super::instructions::{attach_to, ContinuousChange};
use super::recipients::{creatures_you_control, target_permanent};
use crate::objects::card_data::{AbilityDef, AbilityText, AbilityType, Activation};
use crate::types::costs::Cost;
use crate::types::effects::{Condition, Duration, Effect, PlayerFact, PlayerSet};
use crate::types::replacement::ReplacementDef;
use crate::types::restriction::{Restriction, RestrictionDef};

/// A spell ability (CR 113.3a): one paragraph of an instant's or sorcery's
/// text, followed as the spell resolves.
pub fn spell_ability(rules_text: impl Into<AbilityText>, effect: Effect) -> AbilityDef {
    AbilityDef::new(AbilityType::Spell, rules_text, effect)
}

/// A static ability (CR 113.3d): a statement, true while the ability
/// functions. One whose whole effect is a replacement effect or a "can't" is
/// written `static_replacement` or `static_restriction`, which build the same
/// value, so a card file writes each one way.
pub fn static_ability(rules_text: impl Into<AbilityText>, effect: impl StaticBody) -> AbilityDef {
    AbilityDef::new(AbilityType::Static, rules_text, effect.into_static_body())
}

/// What a static ability's effect is written as: an [`Effect`], or a bare
/// [`ContinuousChange`], which applies while its source does (CR 611.3b) and so states
/// no duration.
pub trait StaticBody {
    fn into_static_body(self) -> Effect;
}

impl StaticBody for Effect {
    fn into_static_body(self) -> Effect {
        self
    }
}

impl StaticBody for ContinuousChange {
    fn into_static_body(self) -> Effect {
        self.lasting(Duration::WhileSourceOnBattlefield)
    }
}

/// "As long as [condition], [effect]" (CR 604.2): a static ability's
/// effect exists exactly while its condition holds.
pub fn as_long_as(condition: Condition, body: impl StaticBody) -> Effect {
    Effect::Conditional(condition, Box::new(body.into_static_body()))
}

/// "You control a Forest", "you have 40 or more life", "your library has no
/// cards in it": a fact about the ability's controller (CR 109.5) as it
/// stands when the condition is asked, for an "as long as", an intervening
/// "if" or a state trigger alike. A count over the turn's history ("if you
/// gained life this turn") is a history condition, not this.
pub fn you_have(fact: PlayerFact) -> Condition {
    Condition::Player { players: PlayerSet::You, fact }
}

/// A static ability whose effect is a replacement effect (CR 614.1a): never
/// a resolution, so it carries no `Duration`, and it is re-derived off its
/// source's effective ability list on every gather.
pub fn static_replacement(rules_text: impl Into<AbilityText>, def: ReplacementDef) -> AbilityDef {
    static_ability(rules_text, Effect::Replacement(Box::new(def)))
}

/// A static "can't" (CR 101.2, 614.17), read off its source's effective
/// ability list by `engine::restriction::is_prohibited` at each proposal.
pub fn static_restriction(rules_text: impl Into<AbilityText>, what: Restriction) -> AbilityDef {
    static_ability(rules_text, Effect::Restriction(Box::new(RestrictionDef::new(what))))
}

/// An activated ability (CR 113.3b): "[Cost]: [Effect.]". `activation` is
/// its cost, or `Activation::from(costs).only_as_sorcery()` for one with
/// activation instructions (CR 602.1b).
pub fn activated_ability(rules_text: impl Into<AbilityText>, activation: impl Into<Activation>, effect: Effect) -> AbilityDef {
    AbilityDef::new(AbilityType::Activated(activation.into()), rules_text, effect)
}

/// An activated mana ability (CR 605.1a): no target, it could add mana, and
/// no loyalty cost.
pub fn mana_ability(rules_text: impl Into<AbilityText>, activation: impl Into<Activation>, effect: Effect) -> AbilityDef {
    AbilityDef::new(AbilityType::Mana(activation.into()), rules_text, effect)
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
pub fn equip(rules_text: impl Into<AbilityText>, costs: Vec<Cost>) -> AbilityDef {
    activated_ability(rules_text, Activation::from(costs).only_as_sorcery(), attach_to(target_permanent(creatures_you_control())))
}
