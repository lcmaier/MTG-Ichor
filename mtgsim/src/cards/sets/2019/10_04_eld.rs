//! Throne of Eldraine (`eld`), released 2019-10-04.

use std::sync::Arc;

use crate::cards::authoring::{an_opponent_controls, colored, creature, non, static_replacement, with_subtype, you_control};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{AffectedSet, CounterType, ObjectSet, PlayerSet};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EnterModsTemplate, EventPattern, ReplacementDef, Rewrite, SourcePattern};

/// Grumgully, the Generous — {1}{R}{G}
/// Legendary Creature — Goblin Shaman, 3/3
///
/// > Each other non-Human creature you control enters with an additional
/// > +1/+1 counter on it.
///
/// Master Biomancer's shape with a fixed amount: an entry replacement over the
/// creatures you control that are not Humans. "Other" needs no filter leaf.
/// The gather sweeps the battlefield, where an entering Grumgully is not, so
/// it neither counts itself nor a creature entering beside it.
///
/// It exercises an entry counter over "not a Human", read after CR 616.1c's
/// copy step.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr7_integration_test`.
/// - *A non-Human creature you control that would enter with no +1/+1
///   counters enters with one* (#1) → `grumgully_gives_each_non_human_you_control_its_counter`.
/// - *Other non-Humans entering at the same time as Grumgully get none* (#2)
///   → `a_creature_entering_beside_grumgully_gets_no_counter`.
/// - *A copy of a Human gets none, and a Human entering as a non-Human gets
///   one* (#3) → `grumgully_reads_the_creature_a_copy_enters_as`: CR 614.12's
///   look-ahead, after CR 616.1c's copy step.
pub fn grumgully_the_generous() -> Arc<CardData> {
    let text = "Each other non-Human creature you control enters with an additional +1/+1 counter on it.";
    let non_human_creature_you_control = creature().and(you_control()).and(non(with_subtype(Subtype::Creature(CreatureType::Human))));
    CardDataBuilder::new("Grumgully, the Generous")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Green], 1))
        .color(Color::Red)
        .color(Color::Green)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Goblin))
        .subtype(Subtype::Creature(CreatureType::Shaman))
        .power_toughness(3, 3)
        .rules_text(text)
        .ability(static_replacement(
            text,
            ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                ObjectSet::battlefield_filter(non_human_creature_you_control),
                Rewrite::EnterWith(EnterModsTemplate::with_counters(CounterType::PlusOnePlusOne, 1)),
            ),
        ))
        .build()
}

/// Torbran, Thane of Red Fell — {1}{R}{R}{R}
///
/// > If a red source you control would deal damage to an opponent or a
/// > permanent an opponent controls, it deals that much damage plus 2 instead.
///
/// `AmountRewrite::Plus`'s printed customer, and the phase's only **two-sided**
/// predicate that constrains both sides with a filter: a source-side
/// `ByColor(Red)` and `ByController(You)` beside a target-side
/// `ByController(Opponent)` and `PlayerSet::Opponents`. Both are asked at the
/// event, so a creature that stops being red stops adding 2 (CR 609.7c).
///
/// # The rulings (Scryfall, 2026-09-08), and where each is tested
///
/// - *The additional 2 damage is dealt by the same source as the original
///   source of damage. The damage isn't dealt by Torbran unless Torbran is the
///   original source of damage.* → asserted on `GameEvent::DamageDealt`'s
///   `source_id`, which the rewrite does not touch.
/// - *If another effect modifies how much damage your red source would deal,
///   including preventing some of it, the player being dealt damage … chooses
///   an order in which to apply those effects. **If all of the damage is
///   prevented, Torbran's effect no longer applies.*** → the sharp one: a
///   prevention that empties the event first makes `never_happens` drop it on
///   the next iteration, so Torbran is never gathered (CR 614.7a, re-asked per
///   iteration). And the first half is why `ordering_cannot_change_outcome`
///   must not admit `Plus`.
/// - *If damage dealt by a source you control is being divided or assigned
///   among multiple permanents an opponent controls … divide the original
///   amount before adding 2.* → the trample test: `assign_combat_damage`
///   divides before anything is proposed, so 2 and 3 become 4 and 5.
pub fn torbran_thane_of_red_fell() -> Arc<CardData> {
    CardDataBuilder::new("Torbran, Thane of Red Fell")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red, ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Dwarf))
        .subtype(Subtype::Creature(CreatureType::Noble))
        .power_toughness(2, 4)
        .rules_text(
            "If a red source you control would deal damage to an opponent or a permanent an \
             opponent controls, it deals that much damage plus 2 instead.",
        )
        .ability(static_replacement("If a red source you control would deal damage to an opponent or a permanent an opponent controls, it deals that much damage plus 2 instead.",
            ReplacementDef::new(
                EventPattern::DealDamage {
                    source: Some(SourcePattern::matching(colored(Color::Red).and(you_control()))),
                    combat: None,
                },
                AffectedSet { objects: ObjectSet::battlefield_filter(an_opponent_controls()), players: PlayerSet::Opponents },
                Rewrite::Amount(AmountRewrite::Plus(2)),
            ),
        ))
        .build()
}
