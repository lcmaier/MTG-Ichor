//! Onslaught (`ons`), released 2002-10-07.

use std::sync::Arc;

use crate::cards::authoring::{
    activated_ability, add_phases, create_replacement_for, creature, creatures_you_control, deal_damage_unpreventable, each, spell_ability,
    static_replacement, target_creature, untap, with_subtype, you, you_control,
};
use crate::objects::card_data::{Activation, CardData, CardDataBuilder};
use crate::state::game_state::PhaseType;
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{Effect, ExtraPhasesAfter, ObjectFilter, ObjectSet, PlayerSet};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EventPattern, GameActionTemplate, ReplacementDef, Rewrite, TemplateAmount};

/// Aggravated Assault — {2}{R}
/// Enchantment
///
/// "{3}{R}{R}: Untap all creatures you control. After this main phase, there
///  is an additional combat phase followed by an additional main phase.
///  Activate only as a sorcery."
///
/// (Oracle text verified on Scryfall, 2026-09-14. Mercadian Masques; many
/// reprints.)
///
/// # Why this one
///
/// **The cheapest whole card for CR 500.8**, and the only one of the 46 that
/// needs nothing this engine does not have: `ActivationRestriction::OnlyAsSorcery`
/// has been enforced since Bonesplitter's equip (`cast.rs:345`), and its untap
/// is the recipient arm RE-10 added.
///
/// **The two phases are one `Primitive::ExtraPhases`, not two.** "An additional
/// combat phase followed by an additional main phase" is one resolution
/// creating a run, and their printed order has to be the order they are
/// spliced rather than an accident of which splice ran second — which is also
/// what makes CR 500.8's *"the most recently created phase will occur first"*
/// fall out of a second activation's splice at the same index.
///
/// **The additional main phase is `PhaseType::Postcombat`, and the choice is
/// unobservable.** Checked rather than assumed: every production reader of the
/// two main types matches them as one arm asking "is this a main phase" —
/// `cast.rs:659`, `zones.rs:266` and `oracle/legality.rs:50`. CR 500.8 says
/// only "directly after the specified phase" and the CR names no additional
/// main phase either way. The axis a later card would grow is a field on
/// `PlannedPhase`, and its customer is the first card that prints a
/// distinction between the two main phases the engine must keep.
///
/// # Rulings
///
/// - *"You will normally activate this ability during your postcombat main
///   phase so you can untap any creatures that attacked."* (2023-09-01) —
///   advice rather than a rule, but it is the board the tests use: the untap
///   and the splice are one resolution, so the creatures that attacked are
///   untapped before the combat phase that lets them attack again.
/// - *"If you have enough mana, the ability may be activated more than once in
///   a turn."* (2004-10-04) — **this is CR 500.8's ordering test.** A second
///   activation splices at the same index and pushes the first activation's
///   pair later, which is the rule's "most recently created phase will occur
///   first" with no comparator anywhere.
pub fn aggravated_assault() -> Arc<CardData> {
    CardDataBuilder::new("Aggravated Assault")
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Enchantment)
        .rules_text(
            "{3}{R}{R}: Untap all creatures you control. After this main phase, \
             there is an additional combat phase followed by an additional main \
             phase. Activate only as a sorcery.",
        )
        .ability(activated_ability(
            "{3}{R}{R}: Untap all creatures you control. After this main phase, there is an additional combat phase followed by an additional main phase. Activate only as a sorcery.",
            Activation::from(vec![Cost::Mana(ManaCost::build(
                &[ManaType::Red, ManaType::Red],
                3,
            ))])
            .only_as_sorcery(),
            Effect::Sequence(vec![
                untap(each(creatures_you_control())),
                add_phases(you(), vec![PhaseType::Combat, PhaseType::Postcombat], ExtraPhasesAfter::ThisMainPhase),
            ]),
        ))
        .build()
}

/// "Cleric creature you control" — all three clauses, because the card says
/// all three and a subtype does not carry its card type on an object outside
/// the battlefield.
fn clerics_you_control() -> ObjectFilter {
    creature().and(with_subtype(Subtype::Creature(CreatureType::Cleric)).and(you_control()))
}

/// Daunting Defender — {4}{W}
///
/// > If a source would deal damage to a Cleric creature you control, prevent 1
/// > of that damage.
///
/// **CR 615.10's own example, and the card the rule is written around.** The
/// predicate is entirely on the *target* side — any source at all — which is
/// what makes it the second producer for `AmountRewrite::PreventUpTo` without
/// being a second copy of Guardian Seraph's path: one names an object set with
/// a three-clause filter, the other names a player.
///
/// > 615.10 Example: Daunting Defender says "If a source would deal damage to a
/// > Cleric creature you control, prevent 1 of that damage." Pyroclasm says
/// > "Pyroclasm deals 2 damage to each creature." Pyroclasm will deal 1 damage
/// > to each Cleric creature controlled by Daunting Defender's controller. It
/// > will deal 2 damage to each other creature.
///
/// It is itself a Cleric, so it is inside its own affected set — the rule's
/// example board has it taking 1 from that Pyroclasm.
///
/// Scryfall lists no rulings (2026-09-08); the test is the rule's example,
/// verbatim (`ATOM-615.10-001`).
pub fn daunting_defender() -> Arc<CardData> {
    CardDataBuilder::new("Daunting Defender")
        .mana_cost(ManaCost::build(&[ManaType::White], 4))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(3, 3)
        .rules_text(
            "If a source would deal damage to a Cleric creature you control, prevent 1 of \
             that damage.",
        )
        .ability(static_replacement("If a source would deal damage to a Cleric creature you control, prevent 1 of that damage.", ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            ObjectSet::battlefield_filter(clerics_you_control()),
            Rewrite::Amount(AmountRewrite::PreventUpTo(1)),
        )))
        .build()
}

/// Pinpoint Avalanche — {3}{R}{R}
///
/// > Pinpoint Avalanche deals 4 damage to target creature. The damage can't be
/// > prevented.
///
/// CR 615.12's **per-event** shape, and the only one of its nine printed cards
/// the engine can play whole: Skullcrack and Leyline of Punishment print
/// "players can't gain life", Unstable Footing has kicker, Stomp is an
/// adventure, Flaring Pain has flashback, Wild Slash a `Conditional`,
/// Everlasting Torment wither, and Combust "can't be countered", which is §8a's
/// missing counter event (§11 item 26).
///
/// The flag is on the *event*, not on the spell: it says nothing about the next
/// damage this card's controller deals, and it survives a redirect because
/// CR 614.9 moves "the same damage".
///
/// Scryfall lists no rulings (2026-09-09). The rule's own sentences are the
/// tests: a prevention effect applies and prevents 0
/// (`ATOM-615.12-001`/`-002`), a CR 615.7 count is not reduced
/// (`COMP-615-UNPREVENTABLE-SHIELD-001`), the rider still runs, and the
/// instance is offered once (`ATOM-615.12a-001`).
pub fn pinpoint_avalanche() -> Arc<CardData> {
    CardDataBuilder::new("Pinpoint Avalanche")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 3))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .rules_text(
            "Pinpoint Avalanche deals 4 damage to target creature. The damage can't be \
             prevented.",
        )
        .ability(spell_ability("Pinpoint Avalanche deals 4 damage to target creature. The damage can't be prevented.",
            deal_damage_unpreventable(target_creature(), 4),
        ))
        .build()
}

/// Words of Worship — {2}{W}
/// Enchantment
///
/// > {1}: The next time you would draw a card this turn, you gain 5 life
/// > instead.
///
/// **A draw replaced by life — RE-2's pattern and RE-3's template on one
/// row**, and the first `Uses::Once` draw replacement in the crate. It is a
/// resolution's row rather than a static ability, so `Duration::UntilEndOfTurn`
/// is the card's "this turn" and `Uses::Once` is its "the next time"; neither
/// is derived, for CR 608.2c's reason.
///
/// [`EventPattern::DrawCard`] with `cause: None` — "the next time you would
/// draw a card" excepts nothing, so the draw step's own draw is a candidate.
///
/// **Leyline of Punishment's
/// ruling about this card is the CR 101.2 ordering test**: under a "players
/// can't gain life", the substituted gain is proposed, refused, and the draw
/// has been replaced with nothing.
/// # The rulings, and where each is tested
///
/// - *"If multiple Words have been used prior to drawing a card, then you can
///   choose which one to apply (and use up) each time you draw a card."* → test.
///   Two rows from one source, one prompt, and the other row still there for the
///   next draw — which is what "(and use up)" means.
///   → `two_words_rows_are_a_choice_and_each_draw_uses_one_up`
/// - **Leyline of Punishment's** ruling about this card, since the Leyline is
///   not registered: *"effects that replace an event with gaining life (like
///   Words of Worship's effect does) will end up replacing the event with
///   nothing."* → the CR 101.2 ordering test, on Skullcrack.
///   → `words_of_worship_under_skullcrack_replaces_the_draw_with_nothing`
pub fn words_of_worship() -> Arc<CardData> {
    CardDataBuilder::new("Words of Worship")
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("{1}: The next time you would draw a card this turn, you gain 5 life instead.")
        .ability(activated_ability("{1}: The next time you would draw a card this turn, you gain 5 life instead.",
            vec![Cost::Mana(ManaCost::build(&[], 1))],
            create_replacement_for(
                you(),
                ReplacementDef::new(
                    EventPattern::DrawCard { place: None },
                    PlayerSet::You,
                    Rewrite::Instead(GameActionTemplate::GainLife { amount: TemplateAmount::Fixed(5) }),
                )
                .once(),
            )
            .until_end_of_turn(),
        ))
        .build()
}
