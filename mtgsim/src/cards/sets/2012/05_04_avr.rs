//! Avacyn Restored (`avr`), released 2012-05-04.

use std::sync::Arc;

use crate::cards::authoring::{
    an_opponent_controls, creature, dies, gain_life, lose_life, static_replacement, static_restriction, target_player, triggered_ability, whenever,
    you, you_control,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{AffectedSet, Effect, ObjectSet, PlayerRef, PlayerSet};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EventPattern, ReplacementDef, Rewrite, Rounding};
use crate::types::restriction::{Restriction, SourceFilter};
use crate::types::zones::{Zone, ZoneChangeAct, ZoneChangePattern};

/// Blood Artist — {1}{B}
/// Creature — Vampire 0/1
///
/// > Whenever this creature or another creature dies, target player loses 1
/// > life and you gain 1 life.
///
/// "This creature or another creature" is any creature, so the subject is
/// the plain type filter; its own death is found on the CR 603.10a frame the
/// record carries, and the target is chosen as the ability goes on the
/// stack (CR 603.3d), which is the placement prompt the pool measures.
///
/// It exercises CR 603.10a's look-back off the frame, a target chosen at
/// placement (603.3d).
///
/// # The rulings, and where each is tested
///
/// - *"If Blood Artist and one or more other creatures die at the same time,
///   its ability will trigger for each of those creatures."*
///   → `phase_tr1_integration_test::blood_artist_dying_beside_two_creatures_triggers_three_times`.
pub fn blood_artist() -> Arc<CardData> {
    CardDataBuilder::new("Blood Artist")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Vampire))
        .power_toughness(0, 1)
        .rules_text("Whenever this creature or another creature dies, target player loses 1 life and you gain 1 life.")
        .ability(triggered_ability("Whenever this creature or another creature dies, target player loses 1 life and you gain 1 life.", whenever(
            dies(creature()),
            Effect::Sequence(vec![
                lose_life(target_player(), 1),
                gain_life(you(), 1),
            ]),
        )))
        .build()
}

/// Gisela, Blade of Goldnight — {4}{R}{W}{W}
///
/// > Flying, first strike
/// >
/// > If a source would deal damage to an opponent or a permanent an opponent
/// > controls, that source deals double that damage to that player or permanent
/// > instead.
/// >
/// > If a source would deal damage to you or a permanent you control, prevent
/// > half that damage, rounded up.
///
/// **Two statics on one card, and each is a `Filter` beside a [`PlayerSet`].**
/// That is decision 0's shape used in both of its non-trivial arms — CR 109.5's
/// "you" and CR 102.1's "an opponent", resolved against the ability's current
/// controller on every gather — and both halves of `Rounding` on one
/// battlefield.
///
/// # The rulings, and where each is tested
///
/// - *Gisela doubles damage dealt to opponents and permanents your opponents
///   control from any source, **including sources controlled by those
///   opponents*** → the pattern carries no source-side constraint, so this is
///   true by construction; asserted anyway, because a `SourcePattern` arrives
///   in RD-3 and this is the assertion that would fail if one were added here
///   by mistake.
/// - *If multiple replacement effects would modify how damage would be dealt,
///   the player being dealt damage (or the controller of the permanent) chooses
///   the order* → the Ghosts/Furnace test generalized to a prevention:
///   Gisela's half beside an opponent's Furnace on 5 damage is prevent-3-then-
///   double-2 or double-to-10-then-prevent-5.
/// - *If damage … is being divided or assigned among multiple permanents an
///   opponent controls … divide the original amount and double the results* →
///   the War Mammoth test, from the other side of the table.
pub fn gisela_blade_of_goldnight() -> Arc<CardData> {
    CardDataBuilder::new("Gisela, Blade of Goldnight")
        .mana_cost(ManaCost::build(
            &[ManaType::Red, ManaType::White, ManaType::White],
            4,
        ))
        .color(Color::Red)
        .color(Color::White)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Angel))
        .power_toughness(5, 5)
        .keyword_flag(KeywordFlag::Flying)
        .keyword_flag(KeywordFlag::FirstStrike)
        .rules_text(
            "Flying, first strike\nIf a source would deal damage to an opponent or a \
             permanent an opponent controls, that source deals double that damage to that \
             player or permanent instead.\nIf a source would deal damage to you or a \
             permanent you control, prevent half that damage, rounded up.",
        )
        .ability(static_replacement("If a source would deal damage to an opponent or a permanent an opponent controls, that source deals double that damage to that player or permanent instead.",
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                AffectedSet { objects: ObjectSet::battlefield_filter(an_opponent_controls()), players: PlayerSet::Opponents },
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            ),
        ))
        .ability(static_replacement("If a source would deal damage to you or a permanent you control, prevent half that damage, rounded up.",
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                AffectedSet { objects: ObjectSet::battlefield_filter(you_control()), players: PlayerSet::You },
                Rewrite::Amount(AmountRewrite::PreventHalf(Rounding::Up)),
            ),
        ))
        .build()
}

/// Sigarda, Host of Herons — {2}{G}{W}{W}
/// Legendary Creature — Angel, 5/5
///
/// Flying, hexproof
/// Spells and abilities your opponents control can't cause you to sacrifice
/// permanents.
///
/// (Oracle text verified on Scryfall, 2026-08-31.)
///
/// # Why this clause is Tier 2 and not a cost restriction
///
/// A census regex that saw "sacrifice" filed it under CR 118's costs, and that
/// is wrong: **you never pay costs for an opponent's spell**
/// (`cant-effects-architecture.md` §2.6). Diabolic Edict resolving against you
/// is an *effect*, and what Sigarda forbids is the `ZoneChange { act:
/// Sacrificed }` it would propose. Tajuru Preserver prints the identical
/// sentence; The Master, Multiplied and Tamiyo, Collector of Tales are the same
/// shape with a different event.
///
/// # The three halves of the restriction, and where each one lives
///
/// - **"can't cause you to sacrifice"** is the *event*:
///   `EventPattern::ZoneChange { change: ZoneChangePattern { from: Battlefield, to: Graveyard, act: ///   Sacrificed },}`. CR 701.21a's sacrifice, and specifically not CR 701.8's
///   destruction — 701.21b says a sacrificed permanent is not destroyed, which
///   is why the act is its own variant and why Sigarda stops none of the
///   CR 704.5 state-based actions.
/// - **"permanents"**, scoped to *yours*, is the `ObjectSet`:
///   `ByController(PlayerRef::You)`, resolved against Sigarda's **current**
///   controller at the instant the question is asked (CR 109.5). Stealing
///   Sigarda moves the protection, and the filter being stored unresolved is
///   what makes that free.
/// - **"spells and abilities your opponents control"** is the `by` filter — the
///   one thing this arm needed beyond `EventPattern` + `ObjectSet`, and the
///   reason it exists at all. Read against the resolution `ActionContext`
///   already threads, so it is a field rather than a mechanism.
///
/// # Why she must produce *no prompt*, which is a rule and not a nicety
///
/// Her printed rulings are unusually explicit — "if it would force you to
/// sacrifice a permanent, **you just don't**", and of an optional sacrifice,
/// "**you can't take that option**". So a Diabolic Edict resolving against her
/// controller must not ask. Prompting and then refusing would violate CR 608.2d
/// ("the player can't choose an option that's illegal or impossible"), would
/// leak which permanent you would have picked, and would make an AI harness
/// spend a decision on a branch that cannot happen. §4.9 has the argument; the
/// filter is in `resolve.rs::sacrifice_one_of_choice`.
///
/// # What is deliberately absent
///
/// **Hexproof**, which she prints, is Tier 1d — a restriction on *targeting*,
/// which is a choice rather than an event, and is `cant-be-targeted`'s `BeTargeted` arm
/// (`codebase-state.md` item 15: the `KeywordFlag` is constructible and enforced
/// nowhere). Flying is on the card because `KeywordFlag::Flying` is enforced.
/// Shipping her without hexproof is the Blood Moon precedent: the clause the
/// phase is about, and no more.
pub fn sigarda_host_of_herons() -> Arc<CardData> {
    CardDataBuilder::new("Sigarda, Host of Herons")
        .mana_cost(ManaCost::build(
            &[ManaType::Green, ManaType::White, ManaType::White],
            2,
        ))
        .color(Color::Green)
        .color(Color::White)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Angel))
        .power_toughness(5, 5)
        .keyword_flag(KeywordFlag::Flying)
        .rules_text("Flying, hexproof\nSpells and abilities your opponents control can't cause you to sacrifice permanents.")
        .ability(static_restriction(
            "Spells and abilities your opponents control can't cause you to sacrifice permanents.",
            Restriction::Event {
                // CR 701.21a — "its controller moves it from the battlefield
                // directly to its owner's graveyard".
                pattern: EventPattern::ZoneChange {
                    change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: Some(ZoneChangeAct::Sacrificed) },
                    object: None,
                },
                affected: AffectedSet::objects(ObjectSet::battlefield_filter(you_control())),
                by: Some(SourceFilter::ControlledBy(PlayerRef::Opponent)),
            },
        ))
        .build()
}
