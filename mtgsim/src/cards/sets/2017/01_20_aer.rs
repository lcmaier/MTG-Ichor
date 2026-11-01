//! Aether Revolt (`aer`), released 2017-01-20.

use std::sync::Arc;

use crate::cards::authoring::{artifact, creature, exile, set_life_total, static_replacement, this_object, you, you_control};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Effect, ObjectSet, PlayerSet};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EventPattern, ReplacementDef, Rewrite};

/// Exquisite Archangel — {5}{W}{W}
/// Creature — Angel 5/5
///
/// > Flying
/// > If you would lose the game, instead exile this creature and your life
/// > total becomes equal to your starting life total.
///
/// `Prevent` with a rider, because "instead exile this creature and ..." is
/// two effects on two different subjects — the Archangel and you — and a
/// substitution produces one event about one (§3.2d). The rider's exile names
/// the Archangel through `this_object()`, the effect's own
/// source; its life total is `set_life_total` over
/// `AmountExpr::StartingLifeTotal`, so it is 40 in Commander and a 24-life
/// *gain* from -4 that Rhox Faithmender doubles (CR 119.5).
/// # The rulings, and where each is tested
///
/// - *"If Exquisite Archangel is dealt lethal damage at the same time that
///   you're dealt damage that brings your life total to 0 or less, its effect
///   applies and your life total becomes equal to your starting life total.
///   You choose whether Exquisite Archangel is moved to exile or to your
///   graveyard."* → the first half is a test: the loss and the death are two
///   members of one CR 704.3 batch decided against one board, so the
///   Archangel replaces the loss while it is still there. **The second half
///   is not offered**: riders resolve after the batch performs (CR 615.5,
///   §4.1a), so the death has happened when the rider's exile looks for the
///   creature, and CR 400.7 makes the card in the graveyard a new object the
///   exile does not find. The engine takes the graveyard outcome without the
///   choice — `codebase-state.md` item 125.
///   → `exquisite_archangel_replaces_the_loss_while_dying_in_the_same_check`
/// - *"If an effect says that you can't lose the game, Exquisite Archangel's
///   effect doesn't apply."* → CR 101.2's order: Platinum Angel's row refuses
///   the proposal ahead of the gather, and CR 614.17c leaves nothing for a
///   non-self-replacement to apply to.
///   → `under_platinum_angel_exquisite_archangel_does_not_apply`
/// - *"If you control two Exquisite Archangels, you choose which one's effect
///   applies. The other's effect won't be applicable after that until the next
///   time you would lose the game."* → a CR 616.1 prompt between two printed
///   statics — a `Prevent` with a rider is no suppression shape — and the loss
///   is gone after one applies.
///   → `two_exquisite_archangels_you_choose_which_applies`
/// - *"Exquisite Archangel's effect applies any time you would lose the game,
///   even if you're not losing due to your life total being 0 or less. If you
///   would have lost the game because you tried to draw from an empty library,
///   you won't lose again until you try to draw again and still can't do so."*
///   → the first sentence is `EventPattern::PlayerLoses` having no reason,
///   tested on a poison loss; the second is CR 704.5b's window closing at the
///   check that read it (item 112).
///   → `exquisite_archangel_applies_to_a_poison_loss_and_then_the_poison_still_loses`,
///   `a_replaced_empty_library_loss_is_not_proposed_again_until_the_next_draw`
/// - *"Exquisite Archangel's effect does nothing if you concede the game. A
///   player who concedes leaves the game."* → recorded, no harness offers
///   concession (CR 104.3a is a *leave* that then loses, not a proposed loss).
/// - *"For your life total to become your starting life total (normally 20),
///   you gain or lose the appropriate amount of life. For example, if your life
///   total is -4 when Exquisite Archangel's ability applies, it will cause you
///   to gain 24 life; alternatively, if your life total is 40 when it applies,
///   it will cause you to lose 20 life. Other cards that interact with life gain
///   or life loss will interact with this effect accordingly."* → both
///   directions, and the gain doubled to 48 by Rhox Faithmender.
///   → `exquisite_archangel_from_minus_four_is_a_gain_of_twenty_four_that_rhox_faithmender_doubles`,
///   `exquisite_archangel_applies_to_a_poison_loss_and_then_the_poison_still_loses`
///   (life 40 → 20 is the loss leg)
/// - *"If an effect states that an opponent wins the game, Exquisite Archangel's
///   ability doesn't apply."* → an opponent's win is not a loss event; the game
///   ends with the Archangel untouched.
///   → `an_opponents_win_is_not_a_loss_exquisite_archangel_can_replace`
pub fn exquisite_archangel() -> Arc<CardData> {
    CardDataBuilder::new("Exquisite Archangel")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 5))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Angel))
        .power_toughness(5, 5)
        .keyword_flag(KeywordFlag::Flying)
        .rules_text(
            "Flying\nIf you would lose the game, instead exile this creature and your life total becomes equal to your starting life total.",
        )
        .ability(static_replacement("If you would lose the game, instead exile this creature and your life total becomes equal to your starting life total.",
            ReplacementDef::new(
                EventPattern::PlayerLoses,
                PlayerSet::You,
                Rewrite::Prevent,
            )
            .with_then(Effect::Sequence(vec![
                exile(this_object()),
                set_life_total(you(), AmountExpr::StartingLifeTotal),
            ])),
        ))
        .build()
}

/// Winding Constrictor — {B}{G}
/// Creature — Snake 2/3
///
/// > If one or more counters would be put on an artifact or creature you
/// > control, that many plus one of each of those kinds of counters are put
/// > on that permanent instead.
/// > If you would get one or more counters, you get that many plus one of
/// > each of those kinds of counters instead.
///
/// **Two `Plus(1)` rows, one per subject.** The object half is Hardened
/// Scales' shape over every kind and an artifact-or-creature filter; the
/// player half is the second player-subject watcher, no object and
/// `PlayerSet::You`, Rhox Faithmender's shape over a counter event.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"If an artifact or creature you control would enter the battlefield
///   with a number of any kind of counters on it, it enters with that many
///   plus one instead."* → the entry door on Master Biomancer's grant;
///   `winding_constrictor_adds_one_to_the_counters_a_creature_enters_with`.
/// - *"If an effect includes multiple instructions to put one or more
///   counters on an artifact or creature, such as Lifecrafter's Gift does,
///   Winding Constrictor's effect applies to each of those instructions."*
///   → two `Primitive::AddCounters` in one resolution are two proposals;
///   `winding_constrictor_applies_to_each_instruction`.
/// - *"If you control two Winding Constrictors, the number of counters
///   placed on the artifact or creature is the original number plus two."*
///   → `two_winding_constrictors_add_two`.
/// - *"If you would get counters of multiple kinds at the same time, Winding
///   Constrictor increases the number of each of those kinds of counters by
///   one. The same is true if counters of multiple kinds would be placed on
///   an artifact or creature you control."* → an entry carrying two kinds,
///   each plus one; `winding_constrictor_adds_one_of_each_kind_an_entry_carries`.
/// - *"Winding Constrictor's effect can't apply to itself as it's entering
///   the battlefield or to any other permanent entering the battlefield at
///   the same time as it."* → RC-3's membership rule, asserted on the first
///   half: the Constrictor entering under Master Biomancer gets Biomancer's
///   two and not its own plus one;
///   `winding_constrictor_does_not_apply_to_its_own_entry`. The second half
///   is RE-4's plural batch, and no registered card puts the Constrictor in
///   one.
/// - *"If a nonartifact, noncreature permanent … would enter the battlefield
///   with counters on it and become an artifact or a creature on the
///   battlefield due to another card's effect (such as that of Mycosynth
///   Lattice), Winding Constrictor's effect will give that permanent another
///   of those counters."* → the CR 614.12 frame, RC-4's; the filter is asked
///   of the permanent as it would exist. No such Layer 4 effect is
///   registered, so it is not asserted here.
///
/// **Registered and not pooled**: two colors, and the same engine path as
/// Hardened Scales with a second subject the pool has no producer for.
pub fn winding_constrictor() -> Arc<CardData> {
    CardDataBuilder::new("Winding Constrictor")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Green], 0))
        .color(Color::Black)
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Snake))
        .power_toughness(2, 3)
        .rules_text(
            "If one or more counters would be put on an artifact or creature you control, \
             that many plus one of each of those kinds of counters are put on that permanent \
             instead.\nIf you would get one or more counters, you get that many plus one of \
             each of those kinds of counters instead.",
        )
        .ability(static_replacement("If one or more counters would be put on an artifact or creature you control, that many plus one of each of those kinds of counters are put on that permanent instead.", ReplacementDef::new(
            EventPattern::AddCounters { counter: None, by: None },
            ObjectSet::battlefield_filter(artifact().or(creature()).and(you_control())),
            Rewrite::Amount(AmountRewrite::Plus(1)),
        )))
        .ability(static_replacement("If you would get one or more counters, you get that many plus one of each of those kinds of counters instead.",
            ReplacementDef::new(
                EventPattern::AddCounters { counter: None, by: None },
                PlayerSet::You,
                Rewrite::Amount(AmountRewrite::Plus(1)),
            ),
        ))
        .build()
}
