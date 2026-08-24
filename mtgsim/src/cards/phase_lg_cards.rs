//! Cards for the Layer 2 phase — control-changing effects (CR 613.1b).

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{
    Duration, Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount,
};
use crate::types::ids::new_ability_id;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};

/// Act of Treason — {2}{R}
/// Sorcery
/// Gain control of target creature until end of turn. Untap that creature. It
/// gains haste until end of turn.
///
/// Real text, verbatim. This is the card the CR itself reaches for when it
/// explains CR 613.6 — an effect whose parts land in different layers — and the
/// corpus names it in ATOM-613.6-002: control in Layer 2, haste in Layer 6.
///
/// **The three atoms share one `AbilityDef` and therefore one target.**
/// `Effect::Sequence` resolves each atom against the same `ResolutionContext`,
/// which is what "that creature" means. Split into three abilities they would
/// be three spells.
///
/// **The haste clause is the card explaining CR 302.6 to you.** Gaining control
/// restarts the "continuously since their most recent turn began" clock, so the
/// stolen creature is summoning-sick for the thief and could neither attack nor
/// tap. Haste is what the card pays to make the theft useful, and it is why a
/// Layer 2 implementation that forgets summoning sickness looks correct against
/// this card and only this card.
///
/// **It is also the reason `SetController` compares before it assigns.** "Target
/// creature" has no controller restriction, so aiming this at your own creature
/// is legal, and the CR answer is that nothing about control changes. Haste
/// would hide a clock reset here; a control-gaining card without haste would
/// not.
///
/// Registry-eligible: real card, faithful definition, and the only production
/// card that puts a `SetController` row in the registry — which is what turns
/// `RegistryScopeSummary::any_control_changing` on in `fuzz_games`.
pub fn act_of_treason() -> Arc<CardData> {
    let target_creature =
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1));

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
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::GainControl(Duration::UntilEndOfTurn),
                    target_creature.clone(),
                ),
                Effect::Atom(Primitive::Untap, target_creature.clone()),
                Effect::Atom(
                    Primitive::GrantKeywordFlag(KeywordFlag::Haste, Duration::UntilEndOfTurn),
                    target_creature,
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
        for atom in atoms {
            let Effect::Atom(_, recipient) = atom else {
                panic!("expected an Atom, got {atom:?}");
            };
            assert_eq!(
                *recipient,
                EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
                "all three atoms must name the same target, or 'that creature' is a lie"
            );
        }
    }
}
