//! Urza's Saga (`usg`), released 1998-10-12.

use std::sync::Arc;

use crate::cards::authoring::{
    add_mana, all_damage_to_you, at_beginning_of, creature, creatures_you_control, deal_damage_divided, each, gains_ability, gets, land,
    mana_ability, sacrifice, spell_ability, static_ability, static_replacement, tapped_for_mana, this_object, triggered_ability, unless,
    whenever, Whose,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{AmountExpr, ManaOutput, ObjectSet, PlayerRef, PlayerSet, SelectionFilter, TargetCount};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{GameActionTemplate, ReplacementDef, RetargetSpec, Rewrite, TemplateAmount};

/// Arc Lightning — {2}{R}
/// Sorcery
/// Arc Lightning deals 3 damage divided as you choose among one, two, or three targets.
///
/// # The rulings, and where each is tested
///
/// All in `phase_divided_among_targets_integration_test`.
/// - *"You choose how many targets Arc Lightning has and how the damage is
///   divided as you cast the spell. Each target must receive at least 1
///   damage."* (#1) → `arc_lightning_keeps_the_division_it_was_cast_with`,
///   `arc_lightning_refuses_a_target_with_no_share`.
/// - *"If some of the creatures are illegal targets as Arc Lightning tries to
///   resolve, the original division of damage still applies but no damage is
///   dealt to the illegal targets. If all targets are illegal, Arc Lightning
///   doesn't resolve."* (#2) → `an_illegal_target_takes_no_share_and_the_rest_stand`,
///   `arc_lightning_with_every_target_illegal_does_not_resolve`.
pub fn arc_lightning() -> Arc<CardData> {
    let text = "Arc Lightning deals 3 damage divided as you choose among one, two, or three targets.";
    CardDataBuilder::new("Arc Lightning")
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(text, deal_damage_divided(SelectionFilter::Any, TargetCount::Between(1, 3), 3)))
        .build()
}

/// Citanul Hierophants — {3}{G}
/// Creature — Human Druid, 3/2
/// Creatures you control have "{T}: Add {G}."
///
/// Real card, verbatim, and here because it is the **only** production card
/// exercising `Primitive::GrantAbility` on a *static* ability. That arm has
/// been in `static_primitive_rows` since the Layer 6 phase with nothing
/// reaching it: every `GrantAbility` in the pool arrives through a resolution
/// (`resolve::register_granted_static_effects`), which is a different code
/// path with a different `ObjectSet` shape — `Fixed`, locked to the spell's
/// targets, rather than a live `Filter`.
///
/// **Why this one works when "a static ability granting a static ability over a
/// filter" does not** (`codebase-state.md` item 7, the open filter half): the
/// granted ability here is a *mana* ability, so it generates no continuous
/// effect of its own. There is nothing to derive, nothing for CR 604.2 to
/// re-check, and the whole card is one Layer 6 row over a filter. Swap the
/// granted body for a static one and it lands in the open case immediately.
///
/// It also crosses the layer system into mana enumeration, which is the part
/// that broke last time: `oracle::mana_helpers::activatable_abilities`,
/// `engine::priority`'s id→index re-derivation, and `engine::put_on_stack::
/// activate_ability` must all index the *effective* ability list, or a creature
/// under this card taps for the wrong thing. That coupling is called out in
/// CLAUDE.md and had no card able to test it end to end until now.
pub fn citanul_hierophants() -> Arc<CardData> {
    // The granted body: `{T}: Add {G}`, exactly what a Forest carries.
    let granted = mana_ability(
        "{T}: Add {G}.",
        vec![Cost::TapSelf],
        add_mana(ManaOutput { mana: vec![(ManaType::Green, AmountExpr::Fixed(1))], special: vec![] }),
    );

    CardDataBuilder::new("Citanul Hierophants")
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Druid))
        .power_toughness(3, 2)
        .rules_text("Creatures you control have \"{T}: Add {G}.\"")
        .ability(static_ability(
            "Creatures you control have \"{T}: Add {G}.\"",
            gains_ability(each(creatures_you_control()), granted),
        ))
        .build()
}

/// Contamination — {2}{B}
/// Enchantment
///
/// > At the beginning of your upkeep, sacrifice this enchantment unless you
/// > sacrifice a creature.
/// > If a land is tapped for mana, it produces {B} instead of any other type
/// > and amount.
///
/// CR 118.12a's sentence, the action a cost that is not mana: "you may
/// sacrifice a creature. If you don't, sacrifice this enchantment", offered
/// only when you control a creature (CR 118.3). Its second line is CR
/// 106.12b's retype at a fixed amount, which `phase_re9_integration_test`
/// plays beside Mana Reflection.
///
/// It exercises a payment that sacrifices, chosen as the effect resolves.
///
/// # The rulings, and where each is tested
///
/// - *"The second ability of this card is a replacement effect, not a
///   triggered ability."* (#1) → `contamination_line_beside_mana_reflection_is_a_real_choice`,
///   where it and Mana Reflection are two replacements CR 616.1 orders.
/// - *"External effects that generate more mana still generate their
///   additional mana as directed without being affected by this card.
///   Effects which generate more mana based on what type of mana the land
///   produces will be affected in color but not in amount."* (#2) →
///   `contamination_leaves_an_auras_additional_mana_as_it_is` (Wild Growth's
///   {G}), and the same Mana Reflection test, Contamination applied first:
///   two {B}.
/// - *"You choose whether to sacrifice a creature or not on resolution. If
///   not, then you sacrifice Contamination. You can choose to not sacrifice
///   even if you no longer control Contamination on resolution."* (#3) →
///   `contamination_asks_as_its_trigger_resolves`,
///   `contamination_lost_to_an_opponent_is_not_sacrificed_on_a_decline`.
pub fn contamination() -> Arc<CardData> {
    let upkeep = "At the beginning of your upkeep, sacrifice this enchantment unless you sacrifice a creature.";
    let black = "If a land is tapped for mana, it produces {B} instead of any other type and amount.";
    CardDataBuilder::new("Contamination")
        .mana_cost(ManaCost::build(&[ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("At the beginning of your upkeep, sacrifice this enchantment unless you sacrifice a creature.\nIf a land is tapped for mana, it produces {B} instead of any other type and amount.")
        .ability(triggered_ability(
            upkeep,
            whenever(
                at_beginning_of(StepType::Upkeep, Whose::Yours),
                unless(PlayerRef::You, vec![Cost::Sacrifice(creature(), 1)], sacrifice(this_object())),
            ),
        ))
        .ability(static_replacement(
            black,
            ReplacementDef::new(
                tapped_for_mana(Some(land())),
                PlayerSet::Everyone,
                Rewrite::Instead(GameActionTemplate::ProduceMana { mana_type: ManaType::Black, amount: TemplateAmount::Fixed(1) }),
            ),
        ))
        .build()
}

/// Glorious Anthem - {1}{W}{W}
/// Enchantment
/// Creatures you control get +1/+1.
pub fn glorious_anthem() -> Arc<CardData> {
    CardDataBuilder::new("Glorious Anthem")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("Creatures you control get +1/+1.")
        .ability(static_ability(
            "Creatures you control get +1/+1.",
            gets(each(creatures_you_control()), 1, 1),
        ))
        .build()
}

/// Pariah — {2}{W}
///
/// > Enchant creature
/// > All damage that would be dealt to you is dealt to enchanted creature
/// > instead.
///
/// The phase's `RetargetSpec::ToHost`, and the first Aura in the crate whose
/// static ability is a *replacement* effect rather than a continuous one — Holy
/// Strength opened the Aura path in LH-1 and this reuses it whole: CR 702.5a's
/// "Enchant creature" is the spell's target, and `attached_to` is read at the
/// damage event rather than captured, so the redirect follows the Aura if it
/// moves.
///
/// It affects no object and `PlayerSet::You`: the card is about
/// damage to a **player**, and the creature it names is the destination rather
/// than a member of the set. Mixing the two up would make it redirect damage
/// dealt *to the enchanted creature*, which is the opposite of the card.
///
/// # The rulings (Scryfall, 2026-09-08), and where each is tested
///
/// - *If you would be dealt combat damage, the damage dealt to the enchanted
///   creature instead is still combat damage.* → asserted on
///   `GameEvent::DamageDealt`'s combat flag, which `Rewrite::Retarget` does not
///   touch. It matters beyond bookkeeping: lifelink and first strike both read
///   it, and CR 510.2 is what a Phase 6 trigger will.
/// - *If you control multiple Pariahs enchanting different creatures, you
///   choose which redirection effect to apply. You can't divide damage dealt by
///   one source … you can't choose to have 3 damage dealt to each one.* →
///   CR 616.1's prompt with two candidates, and the whole 6 landing on one
///   host. It falls out of the loop rather than being coded: the first redirect
///   moves the event's subject off the player, so the second Pariah's
///   `PlayerSet::You` no longer matches and it is never offered again.
///
/// **`PERFORMANCE_POOL`'s RD-4 addition**, predicted: the first `Retarget` and
/// the first `attached_to` read on a damage event.
pub fn pariah() -> Arc<CardData> {
    CardDataBuilder::new("Pariah")
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(crate::types::card_types::EnchantmentType::Aura))
        .enchant_filter(SelectionFilter::Creature)
        .rules_text(
            "Enchant creature\nAll damage that would be dealt to you is dealt to enchanted \
             creature instead.",
        )
        .ability(static_replacement("All damage that would be dealt to you is dealt to enchanted creature instead.", all_damage_to_you(
            ObjectSet::NO_OBJECTS,
            RetargetSpec::ToHost,
        )))
        .build()
}
