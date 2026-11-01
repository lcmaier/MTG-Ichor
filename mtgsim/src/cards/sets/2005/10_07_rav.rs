//! Ravnica: City of Guilds (`rav`), released 2005-10-07.

use std::sync::Arc;

use crate::cards::authoring::{
    becomes_attached, create_tokens, every_permanent_or_player, gets, spell_ability, static_replacement, target_creature, triggered_ability, whenever,
    with_subtype, you, you_control,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, EnchantmentType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{ObjectSet, PlayerSet, TokenDef};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EventPattern, ReplacementDef, Rewrite, Rounding};
use crate::types::triggers::TriggerSubject;

/// Bramble Elemental — {3}{G}{G}
/// Creature — Elemental 4/4
///
/// > Whenever an Aura becomes attached to this creature, create two 1/1
/// > green Saproling creature tokens.
///
/// "Becomes attached" on the transition only (CR 603.2e): an Aura spell
/// resolving onto it and one entering attached by any other means each
/// trigger it (CR 303.4f), as does one moved onto it from another object,
/// and an attach to it while the Aura is already there does not. Brood
/// Keeper's first ruling says so of the same text.
///
/// It exercises "becomes attached", an Aura entering attached included (CR
/// 303.4f).
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The benign consumer of the same records: it harms nobody a random agent
/// plays it against, so it carries the phase's new path into the
/// `performance` tables, which TR-4b re-recorded with it. A random Aura
/// seldom lands on it; what it measures is the dispatch's cost, since while
/// it is on the battlefield every attach a pooled Aura or Equipment makes is
/// matched against it.
///
/// # The rulings
///
/// Scryfall lists none.
pub fn bramble_elemental() -> Arc<CardData> {
    let text = "Whenever an Aura becomes attached to this creature, create two 1/1 green Saproling creature tokens.";
    CardDataBuilder::new("Bramble Elemental")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elemental))
        .power_toughness(4, 4)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(
                becomes_attached(
                    with_subtype(Subtype::Enchantment(EnchantmentType::Aura)),
                    TriggerSubject::ThisObject,
                ),
                create_tokens(you(), 2, TokenDef::creature(1, 1, &[Color::Green], &[CreatureType::Saproling])),
            ),
        ))
        .build()
}

/// CR 614.16's counter half over every kind, for a permanent you control —
/// Doubling Season's second ability, the def three cards here share a shape
/// with.
fn doubles_counters_on_your_permanents() -> ReplacementDef {
    ReplacementDef::new(
        EventPattern::AddCounters { counter: None, by: None },
        ObjectSet::battlefield_filter(you_control()),
        Rewrite::Amount(AmountRewrite::Multiplier(2)),
    )
}

/// Doubling Season — {4}{G}
/// Enchantment
///
/// > If an effect would create one or more tokens under your control, it
/// > creates twice that many of those tokens instead.
/// > If an effect would put one or more counters on a permanent you control,
/// > it puts twice that many of those counters on that permanent instead.
///
/// **Both halves of CR 614.16, on one card.** The token half is Parallel
/// Lives' def word for word (RE-4); the counter half is the first
/// `AddCounters` watcher, and it reads *the permanent's controller* — "a
/// permanent you control" — and nothing about who puts the counters on, which
/// is Vorinclex's question and not this card's. The two are two instances
/// with two CR 614.5 identities: the token half is applied at a creation and
/// the counter half at each entry the creation contains (CR 616.1g), and
/// neither is offered at the other's step.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"Planeswalkers will enter with double the normal number of loyalty
///   counters."* → CR 306.5b's seed through the entry door;
///   `doubling_season_doubles_a_planeswalkers_loyalty`.
/// - *"However, if you activate an ability whose cost has you put loyalty
///   counters on a planeswalker, the number you put on isn't doubled. This
///   is because those counters are put on as a cost, not as an effect."* →
///   not asserted: `Cost::AddCounters` is unimplemented, and the fact the
///   payment will need on the event is `codebase-state.md`'s "Found by
///   RE-5" line.
/// - *"Everything that is specified by the effect creating the original
///   token or tokens will also be true about the additional token or
///   tokens"* → RE-4's `the_extra_tokens_are_the_same_tokens`, on the def
///   this half shares.
/// - *"Doubling Season affects permanents that enter with counters."* →
///   Chainbreaker enters with four -1/-1 counters;
///   `doubling_season_doubles_the_counters_a_permanent_enters_with`. And
///   Master Biomancer's grant, the CR 616.2 board: the Season is not
///   applicable until Biomancer has written the counters, so nothing is
///   asked and the grant is doubled;
///   `doubling_season_doubles_master_biomancers_grant_and_asks_nothing`.
/// - *"Battles will enter with double the normal number of defense
///   counters."* → `backlog.md` §2.23; no battle exists.
/// - *"If there are two Doubling Seasons on the battlefield, then the number
///   of tokens or counters is four times the original number."* → both
///   halves, `test_two_doubling_seasons_quadruple` — §10's acid test, on the
///   counter half by the route RB could not reach.
///
/// **Registered and not pooled.** Five mana, and its token half would double
/// the pool's Soldiers — a gameplay change the A/B should not carry with the
/// engine change (`replacement-architecture.md` §9).
pub fn doubling_season() -> Arc<CardData> {
    CardDataBuilder::new("Doubling Season")
        .mana_cost(ManaCost::build(&[ManaType::Green], 4))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(
            "If an effect would create one or more tokens under your control, it creates \
             twice that many of those tokens instead.\nIf an effect would put one or more \
             counters on a permanent you control, it puts twice that many of those counters \
             on that permanent instead.",
        )
        .ability(static_replacement("If an effect would create one or more tokens under your control, it creates twice that many of those tokens instead.",
            ReplacementDef::new(
                EventPattern::CreateTokens { kind: None },
                PlayerSet::You,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            ),
        ))
        .ability(static_replacement("If an effect would put one or more counters on a permanent you control, it puts twice that many of those counters on that permanent instead.", doubles_counters_on_your_permanents()))
        .build()
}

/// Ghosts of the Innocent — {5}{W}{W}
///
/// > If a source would deal damage to a permanent or player, it deals half
/// > that damage, rounded down, to that permanent or player instead.
///
/// The printed **inverse** of a doubler, and the reason `Rounding` exists:
/// CR 107.1a puts the direction on the card, and this one says "rounded down"
/// where Gisela says "rounded up".
///
/// # The rulings, and where each is tested
///
/// - *Half of 1 rounded down is 0. A source that would deal 1 damage won't
///   deal damage at all* → the rewrite leaves a 0-damage proposal and
///   `never_happens` drops it on the next iteration (CR 614.7a).
/// - *Multiple effects are cumulative … with three on the battlefield, 14
///   damage becomes 7, then 3, then finally 1* → three instances, each applied
///   once (CR 614.5).
/// - *This isn't a damage prevention effect. If Excruciator … would deal 7
///   damage, it deals 3 instead* → the reason [`AmountRewrite::Halve`] is a
///   separate arm from `PreventHalf`, which CR 615.12 would stop. Excruciator's
///   half is RD-4's, when a damage event can be unpreventable.
/// - *If damage is redirected, it's only halved once* → **RD-4's**: CR 614.5's
///   applied set survives a `Retarget`, and there is no `Retarget` yet.
/// - *If both Ghosts of the Innocent and Furnace of Rath are on the
///   battlefield, the controller of the permanent being dealt damage or the
///   player being dealt damage can apply the effects in either order. This can
///   matter if the original amount of damage is odd* → the phase's sharpest
///   test, and the first non-commuting CR 616.1 choice reachable from two
///   printed statics.
/// - *If a damage prevention effect and this effect would apply to the same
///   damage, the player … may apply the effects in either order* → the same
///   question against a prevention, which Gisela's other half supplies.
pub fn ghosts_of_the_innocent() -> Arc<CardData> {
    CardDataBuilder::new("Ghosts of the Innocent")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 5))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Spirit))
        .power_toughness(4, 5)
        .rules_text(
            "If a source would deal damage to a permanent or player, it deals half that \
             damage, rounded down, to that permanent or player instead.",
        )
        .ability(static_replacement("If a source would deal damage to a permanent or player, it deals half that damage, rounded down, to that permanent or player instead.",
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                every_permanent_or_player(),
                Rewrite::Amount(AmountRewrite::Halve(Rounding::Down)),
            ),
        ))
        .build()
}

/// Seeds of Strength — {G}{W}
/// Instant
/// Target creature gets +1/+1 until end of turn.
/// Target creature gets +1/+1 until end of turn.
/// Target creature gets +1/+1 until end of turn.
///
/// **Three instances of "target" with identical clauses**, which is the case no
/// structural rule can decide. Written the way this crate encoded cards before
/// A4i, this card's effect tree is the same as **Act of Treason**'s — three
/// atoms, every recipient `Target(Creature, Exactly(1))`, differing only in the
/// `Primitive`, which says what happens and never to whom. Act of Treason is
/// **one** instance ("target creature", then "that creature", then "it") and
/// this is **three**. The difference is in the printed text and nowhere in the
/// tree, so the card carries it — see `EffectRecipient::SameInstanceAs`.
///
/// In `PERFORMANCE_POOL`, and why: several-clause casting is a new engine path
/// (`engineering-practices.md#cards-and-pools`), and this is the cheapest card
/// that walks it — two mana, both colors reachable from the pool's duals, and
/// castable with one creature on the board.
///
/// # The rulings, and where each is tested
///
/// - *"You may choose the same creature as a target multiple times since the
///   card says 'target creature' multiple times. You may give three different
///   creatures +1/+1 each, one creature +2/+2 and another creature +1/+1, or a
///   single creature +3/+3."* — **testable now**, and it is CR 601.2c's second
///   sentence with this card's numbers. All three distributions are asserted:
///   `seeds_of_strength_can_name_one_creature_for_all_three_instances`,
///   `seeds_of_strength_can_split_its_three_instances_across_creatures`.
pub fn seeds_of_strength() -> Arc<CardData> {
    let pump = |recipient| {
        gets(recipient, 1, 1).until_end_of_turn()
    };
    CardDataBuilder::new("Seeds of Strength")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::White], 0))
        .color(Color::Green)
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text("Target creature gets +1/+1 until end of turn.\nTarget creature gets +1/+1 until end of turn.\nTarget creature gets +1/+1 until end of turn.")
        .ability(spell_ability("Target creature gets +1/+1 until end of turn.", pump(target_creature())))
        .ability(spell_ability("Target creature gets +1/+1 until end of turn.", pump(target_creature())))
        .ability(spell_ability("Target creature gets +1/+1 until end of turn.", pump(target_creature())))
        .build()
}
