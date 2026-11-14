//! Magic 2010 (`m10`), released 2009-07-17.

use std::sync::Arc;

use crate::cards::authoring::{
    an_opponent_controls, at_beginning_of, color, counters_removed_from, create_delayed_trigger, create_replacement, creature, creatures_you_control,
    destroy, gain_control_of, gain_life, gains_keyword, non, put_counters, remove_counters, spell_ability, static_replacement, target_creature,
    target_permanent, the_target, this_object, triggered_ability, untap, whenever, you, Whose,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AffectedSet, AmountExpr, CounterType, Effect, ObjectSet, PlayerSet};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::types::replacement::{AmountRewrite, EnterModsTemplate, EventPattern, ReplacementDef, Rewrite, SourcePattern};
use crate::types::triggers::{DelayedDuration, DelayedTriggerTemplate, DelayedTurn, TriggerSubject};

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
        .rules_text("Gain control of target creature until end of turn. Untap that creature. It gains haste until end of turn.")
        .ability(spell_ability(
            "Gain control of target creature until end of turn. Untap that creature. It gains haste until end of turn.",
            Effect::Sequence(vec![
                // One instance of "target" and three atoms (CR 115.3): the
                // card says "target creature" once, and "that creature" and
                // "it" are back-references to it, not new clauses.
                gain_control_of(target_creature()).until_end_of_turn(),
                untap(the_target()),
                gains_keyword(the_target(), KeywordFlag::Haste).until_end_of_turn(),
            ]),
        ))
        .build()
}

pub fn angels_mercy() -> Arc<CardData> {
    CardDataBuilder::new("Angel's Mercy")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text("You gain 7 life.")
        .ability(spell_ability(
            "You gain 7 life.",
            gain_life(you(), 7),
        ))
        .build()
}

/// Doom Blade - {1}{B}
/// Instant
/// Destroy target nonblack creature.
pub fn doom_blade() -> Arc<CardData> {
    CardDataBuilder::new("Doom Blade")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Instant)
        .rules_text("Destroy target nonblack creature.")
        .ability(spell_ability(
            "Destroy target nonblack creature.",
            destroy(target_permanent(creature().and(non(color(Color::Black))))),
        ))
        .build()
}

/// Guardian Seraph — {2}{W}{W}
///
/// > Flying
/// >
/// > If a source an opponent controls would deal damage to you, prevent 1 of
/// > that damage.
///
/// CR 615.10's static partial prevention with a **source-side** predicate, and
/// `AmountRewrite::PreventUpTo`'s first printed producer (`codebase-state.md`
/// item 91). The predicate is CR 609.7c's, not 609.7a's: nothing is chosen, so
/// the filter is asked of every source, on the battlefield or not.
///
/// **The pooled card of RD-3.** It is the first source in
/// `PERFORMANCE_POOL` whose gather evaluates an `ObjectFilter` on the damage's
/// *source* rather than on its target, which is a new read per damage event
/// for as long as it is on the battlefield.
///
/// # The rulings (Scryfall, 2026-09-08), and where each is tested
///
/// - *This ability prevents 1 damage from each source an opponent controls each
///   time that source would deal damage to you. It prevents 1 of any damage,
///   not just combat damage.* → two simultaneous attackers each have 1
///   prevented, and a non-combat bolt is reduced too. That is CR 615.10's
///   "separately to damage from other applicable events that would happen at
///   the same time", which is `ATOM-615.10-001`'s shape on a player.
/// - *Spells and permanents have controllers, but cards that aren't on the
///   stack or the battlefield don't. If a source without a controller (such as
///   a cycled Jund Sojourners) would deal damage to you, 1 of that damage is
///   prevented if an opponent owns that source.* → **not expressible, and not
///   tested.** `ObjectFilter::ByController` on an object in a hidden zone falls
///   through `compute::base_controller` to its owner, so the engine would give
///   this ruling's answer by accident rather than by rule — and no registered
///   card can produce a damage source outside the battlefield and the stack, so
///   there is no board to assert it on. CR 609.7c's non-battlefield source is
///   covered instead by Lightning Bolt, a spell on the stack
///   (`ATOM-609.7c-001`).
/// - *The effects from multiple Guardian Seraphs are cumulative.* → two on the
///   battlefield reduce 3 damage to 1, each applied once (CR 614.5).
pub fn guardian_seraph() -> Arc<CardData> {
    CardDataBuilder::new("Guardian Seraph")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Angel))
        .power_toughness(3, 4)
        .keyword_flag(KeywordFlag::Flying)
        .rules_text(
            "Flying\nIf a source an opponent controls would deal damage to you, prevent 1 \
             of that damage.",
        )
        .ability(static_replacement("If a source an opponent controls would deal damage to you, prevent 1 of that damage.",
            ReplacementDef::new(
                EventPattern::DealDamage {
                    source: Some(SourcePattern::matching(an_opponent_controls())),
                    combat: None,
                },
                PlayerSet::You,
                Rewrite::Amount(AmountRewrite::PreventUpTo(1)),
            ),
        ))
        .build()
}

/// Protean Hydra — {X}{G}
/// Creature — Hydra 0/0
///
/// > This creature enters with X +1/+1 counters on it.
/// > If damage would be dealt to this creature, prevent that damage and
/// > remove that many +1/+1 counters from it.
/// > Whenever a +1/+1 counter is removed from this creature, put two +1/+1
/// > counters on it at the beginning of the next end step.
///
/// X is the X of the spell becoming it (CR 107.3m), 0 for any other way onto
/// the battlefield. The prevention is Angel of Suffering's shape: `Prevent`,
/// and a CR 615.5 rider reading "that many", the damage that would have been
/// dealt. The removal is read once for each counter, and each makes a delayed
/// trigger whose "it" is this creature, found by identity (CR 400.7).
///
/// It exercises an entry template's X (CR 107.3m), a prevention whose rider
/// removes that many counters, and a removal read once per counter that makes a
/// delayed trigger.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr5b_integration_test`.
/// - *"All damage that would be dealt to Protean Hydra is prevented, even if
///   it doesn't have that many +1/+1 counters on it. This is true even if
///   Protean Hydra has no +1/+1 counters on it at all (and is able to remain
///   on the battlefield because some other effect is boosting its
///   toughness). If the amount of damage that would be dealt to Protean
///   Hydra is greater than the number of +1/+1 counters on it, all the +1/+1
///   counters on Protean Hydra are removed from it."* (#1) →
///   `protean_hydra_prevents_all_damage_and_loses_the_counters_it_has`.
/// - *"If unpreventable damage is dealt to Protean Hydra, the Hydra's second
///   ability will try to prevent it and fail (meaning that damage has its
///   normal results), and it will also remove that many +1/+1 counters from
///   Protean Hydra."* (#2) →
///   `protean_hydra_loses_counters_to_damage_that_cannot_be_prevented`.
/// - *"Protean Hydra's last ability triggers whenever a +1/+1 counter is
///   removed from it for any reason, not just when a +1/+1 counter is
///   removed by its second ability."* (#3) →
///   `protean_hydra_grows_back_from_any_counter_removal`.
/// - *"If multiple +1/+1 counters are removed from Protean Hydra at once, its
///   last ability will trigger that many times."* (#4) →
///   `protean_hydra_triggers_once_for_each_counter_removed`.
/// - *"If a -1/-1 counter is put on Protean Hydra while it has +1/+1 counters
///   on it, that -1/-1 counter and a +1/+1 counter will be removed from it as
///   a state-based action. This will cause Protean Hydra's last ability to
///   trigger."* (#5) → `protean_hydra_triggers_on_counter_annihilation`.
/// - *"If any +1/+1 counters are removed from Protean Hydra during the end
///   step, its last ability will trigger and set up a delayed triggered
///   ability that will trigger at the beginning of the next end step (which
///   is near the end of the following turn)."* (#6) →
///   `protean_hydra_removed_during_an_end_step_grows_at_the_next_turns`.
pub fn protean_hydra() -> Arc<CardData> {
    let enters_text = "This creature enters with X +1/+1 counters on it.";
    let prevent_text =
        "If damage would be dealt to this creature, prevent that damage and remove that many +1/+1 counters from it.";
    let grow_text = "Whenever a +1/+1 counter is removed from this creature, put two +1/+1 counters on it at the beginning of the next end step.";
    let two_more = DelayedTriggerTemplate {
        def: Arc::new(whenever(
            at_beginning_of(StepType::End, Whose::Each),
            put_counters(this_object(), CounterType::PlusOnePlusOne, 2),
        )),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::NoneNamed,
        rules_text: "Put two +1/+1 counters on it at the beginning of the next end step.".into(),
    };
    let prevent = ReplacementDef::new(EventPattern::DealDamage { source: None, combat: None }, ObjectSet::SourceOnly, Rewrite::Prevent)
        .with_then(remove_counters(this_object(), CounterType::PlusOnePlusOne, AmountExpr::ReplacedAmount));
    CardDataBuilder::new("Protean Hydra")
        .mana_cost(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Green)]))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Hydra))
        .power_toughness(0, 0)
        .rules_text(&format!("{enters_text}\n{prevent_text}\n{grow_text}"))
        .ability(static_replacement(
            enters_text,
            ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                ObjectSet::SourceOnly,
                Rewrite::EnterWith(EnterModsTemplate::with_counter_amount(CounterType::PlusOnePlusOne, AmountExpr::X)),
            ),
        ))
        .ability(static_replacement(prevent_text, prevent))
        .ability(triggered_ability(
            grow_text,
            whenever(
                counters_removed_from(TriggerSubject::ThisObject, CounterType::PlusOnePlusOne),
                create_delayed_trigger(you(), two_more),
            ),
        ))
        .build()
}

/// Safe Passage — {2}{W}
///
/// > Prevent all damage that would be dealt to you and creatures you control
/// > this turn.
///
/// A `Filter` + `You` row with no count and `Uses::Static` — the shape whose
/// set is evaluated **at the event**, which is the whole difference between it
/// and Samite Censer-Bearer.
///
/// # The rulings (Scryfall, 2026-09-08), and where each is tested
///
/// - *prevents all damage, not just combat damage* → a non-combat bolt to its
///   caster is prevented.
/// - *will prevent damage dealt to creatures that weren't on the battlefield
///   at the time it resolved* → the load-bearing one: a creature placed after
///   resolution is covered, because the row carries a `Filter` and not a
///   `Fixed` — the opposite of CR 615.11's ruling on Censer-Bearer.
/// - *doesn't prevent damage that would be dealt to planeswalkers you
///   control* → Loyalty Probe under Safe Passage loses loyalty; the filter is
///   `ByType(Creature)`.
/// - *has no effect on damage that's already been dealt* → damage marked
///   before it resolves stays marked (CR 615.4 from the other side).
pub fn safe_passage() -> Arc<CardData> {
    CardDataBuilder::new("Safe Passage")
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(
            "Prevent all damage that would be dealt to you and creatures you control this turn.",
        )
        .ability(spell_ability("Prevent all damage that would be dealt to you and creatures you control this turn.",
            create_replacement(ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                AffectedSet { objects: ObjectSet::battlefield_filter(creatures_you_control()), players: PlayerSet::You },
                Rewrite::Prevent,
            ))
            .until_end_of_turn(),
        ))
        .build()
}
