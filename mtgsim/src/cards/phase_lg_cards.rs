//! Cards for the Layer 2 phase — control-changing effects (CR 613.1b).

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{
    Duration, Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount,
};
use crate::types::ids::AbilityId;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};

/// Act of Treason — {2}{R}
/// Sorcery
/// Gain control of target creature until end of turn. Untap that creature. It
/// gains haste until end of turn.
///
/// Real text, verbatim, and the corpus's example for CR 613.6 in
/// ATOM-613.6-002: control in Layer 2, haste in Layer 6.
///
/// The three atoms share one `AbilityDef` and so resolve against one
/// `ResolutionContext`, which is what "that creature" means — split into three
/// abilities they would be three spells.
///
/// The haste clause is the card paying for CR 302.6: gaining control restarts
/// the summoning-sickness clock, so without it the stolen creature could
/// neither attack nor tap. That also makes this card a poor witness for the
/// rule; the tests use a bare `GainControl` for that.
///
/// Registry-eligible, and the only production card that registers a
/// `SetController` row — so it is what exercises Layer 2 in `fuzz_games`.
pub fn act_of_treason() -> Arc<CardData> {
    CardDataBuilder::new("Act of Treason")
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(
            "Gain control of target creature until end of turn. Untap that creature. \
             It gains haste until end of turn.",
        )
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                // One instance of "target" and three atoms (CR 115.3): the
                // card says "target creature" once, and "that creature" and
                // "it" are back-references to it, not new clauses.
                Effect::Atom(
                    Primitive::GainControl(Duration::UntilEndOfTurn),
                    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                ),
                Effect::Atom(Primitive::Untap, EffectRecipient::SameInstanceAs(0)),
                Effect::Atom(
                    Primitive::GrantKeywordFlag(KeywordFlag::Haste, Duration::UntilEndOfTurn),
                    EffectRecipient::SameInstanceAs(0),
                ),
            ]),
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn act_of_treason_is_one_ability_with_three_atoms_on_one_target() {
        let card = act_of_treason();
        assert_eq!(
            card.abilities.len(),
            1,
            "one AbilityDef, so all three atoms resolve against one target"
        );
        let Effect::Sequence(atoms) = &card.abilities[0].effect else {
            panic!("expected a Sequence, got {:?}", card.abilities[0].effect);
        };
        assert_eq!(atoms.len(), 3);

        // CR 115.3 — one instance of "target", three atoms. The card prints
        // "target creature" once; a second declaration would let the untap and
        // the haste land on a different creature from the steal, and would ask
        // the player three times.
        assert_eq!(
            card.spell_instances.len(),
            1,
            "\"that creature\" and \"it\" are back-references, not new clauses"
        );
        let recipients: Vec<&EffectRecipient> = atoms
            .iter()
            .map(|atom| match atom {
                Effect::Atom(_, recipient) => recipient,
                other => panic!("expected an Atom, got {other:?}"),
            })
            .collect();
        assert_eq!(
            *recipients[0],
            EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1))
        );
        assert_eq!(*recipients[1], EffectRecipient::SameInstanceAs(0));
        assert_eq!(*recipients[2], EffectRecipient::SameInstanceAs(0));
    }
}
