//! Gatecrash (`gtc`), released 2013-02-01.

use std::sync::Arc;

use crate::cards::authoring::{cant, creature, deal_damage, spell_ability, static_replacement, target_player, you_control};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AffectedSet, AmountExpr, CharacteristicEdit, CounterType, Effect, ObjectFilter, ObjectSet, PlayerSet, TypeChange};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EnterModsTemplate, EventPattern, ReplacementDef, Rewrite};
use crate::types::restriction::{ReplacementKindFilter, Restriction};

/// Master Biomancer — {2}{G}{U}
/// Creature — Elf Wizard, 2/4
///
/// Each other creature you control enters with a number of additional +1/+1
/// counters on it equal to this creature's power and as a Mutant in addition to
/// its other types.
///
/// (Oracle text verified on Scryfall, 2026-09-03.)
///
/// # The card §5b argued the whole look-ahead frame against
///
/// "Equal to **this creature's** power" is read off the *real board*, not off
/// the entering permanent's hypothetical one, and the two answers differ on a
/// board the pool can build: Elvish Archdruid ("other Elf creatures you control
/// get +1/+1") entering under a Biomancer — itself an **Elf** — enters with
/// **2** counters and not 3, because the Archdruid's own anthem is in the
/// Archdruid's frame and in no other object's. RC-4 built that asymmetry;
/// RC-5's job was to give it an amount to read, and
/// `replacement::evaluate_enter_template` gets it right by asking
/// `EntryFrame::frame_of(source)`, which answers only for the entering object.
///
/// It is also the first `ObjectSet::Filter` entry replacement whose *amount*
/// is dynamic, which is what makes `ordering_cannot_change_outcome`'s new premise
/// load-bearing rather than theoretical (`codebase-state.md` item 47).
///
/// **"Other" needs no filter leaf.** A Biomancer's own ability cannot apply to
/// a Biomancer that is entering: gather source 1 sweeps the battlefield, where
/// the entering object is not, and source 1a admits only `ObjectSet::SourceOnly`
/// (CR 614.12's parenthesis). Two Biomancers entering as one event give each
/// other nothing for the same reason, which is §5b's other worked example.
///
/// # The Mutant is what the creature enters as
///
/// "As a Mutant in addition to its other types" is CR 614.1c's other half: not
/// state the creature enters with, like the counters, but a characteristic
/// its entry fixes (`replacement-architecture.md` §3.5). It is
/// a `CharacteristicEdit` on the same template, and the creature carries it
/// as `PermanentState::entered_as`, applied at layer 4 at the creature's own
/// timestamp: a later Layer 4 effect applies over it, it is not copied
/// (CR 707.2's last sentence), and it stays after the Biomancer leaves, since
/// it is how the creature entered and not the Biomancer's effect. The card's
/// one ruling is about the counters, so that duration is CR 614.1c's.
///
/// # In `PERFORMANCE_POOL`
///
/// The dynamic amount is a second new engine path RC-5 opened, and it is
/// the one that costs a layer read per application. With `Everywhere` in the
/// pool a `{2}{G}{U}` card is castable in every deck.
pub fn master_biomancer() -> Arc<CardData> {
    CardDataBuilder::new("Master Biomancer")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Blue], 2))
        .color(Color::Green)
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elf))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .power_toughness(2, 4)
        .rules_text(
            "Each other creature you control enters with a number of additional \
             +1/+1 counters on it equal to this creature's power and as a Mutant in \
             addition to its other types.",
        )
        .ability(static_replacement(
            "Each other creature you control enters with a number of additional +1/+1 counters on it equal to this creature's power and as a Mutant in addition to its other types.",
            ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                ObjectSet::battlefield_filter(creature().and(you_control())),
                Rewrite::EnterWith(EnterModsTemplate {
                    edits: vec![CharacteristicEdit::Types(TypeChange::adding(&[], &[], &[Subtype::Creature(CreatureType::Mutant)]))],
                    ..EnterModsTemplate::with_counter_amount(CounterType::PlusOnePlusOne, AmountExpr::SourcePower)
                }),
            ),
        ))
        .build()
}

/// Skullcrack — {1}{R}
/// Instant
///
/// > Players can't gain life this turn. Damage can't be prevented this turn.
/// > Skullcrack deals 3 damage to target player or planeswalker.
///
/// **The card that lands RD-4's restriction row in a game** (§11 item 26). Its
/// second sentence is the `ApplyReplacement { Prevention }` row RD-4 could only
/// build as a fixture, because every printed carrier of it needed a facility
/// the engine lacked; its first is the `Event { GainLife }` row RE-3 gave
/// `Restriction::Event` the player set for (item 45).
///
/// **Three atoms in text order, and the order is the card's**: CR 608.2c reads
/// a spell's instructions in the order printed, and both restrictions are in
/// place before the damage is dealt — which is what makes a lifelinker's damage
/// gain nothing this turn.
///
/// **Leyline of Punishment is deliberately not registered.** It is the static
/// form of the same two rows — an `Effect::Restriction` on a permanent, which
/// RS-1's sweep already reads — and the RD-4 fixture extended with the life arm
/// is what proves that form. What keeps it out is its first line: "if this card
/// is in your opening hand, you may begin the game with it on the battlefield"
/// is §3.3 source 2's zone-reaching static, which would be dead text under a
/// real card name. Recorded so the omission reads as the rule and not as an
/// oversight.
/// # The rulings, and where each is tested
///
/// - *"[This] targets only the player or planeswalker. If that player or
///   planeswalker is an illegal target when [it] tries to resolve, it won't
///   resolve and none of its effects will happen."* → CR 608.2b's fizzle, which
///   is the stack's and predates RE-3; the card adds no new claim to it.
///   Recorded rather than re-tested.
/// - *"Spells and abilities that would cause a player to gain life or that would
///   prevent damage still resolve, but the life-gain and damage-prevention parts
///   have no effect."* → test, and "still resolve" is the half that could have
///   been got wrong: a refused proposal is not an error.
///   → `a_life_gain_spell_still_resolves_under_skullcrack_and_gains_nothing`,
///   with `skullcrack_stops_life_gain_for_everyone_including_its_controller` for
///   `PlayerSet::Everyone`.
/// - *"Effects that would replace gaining life with another effect won't apply
///   because it's impossible for players to gain life."* → CR 119.7's own last
///   clause, and `ATOM-119.7-004`.
///   → `under_skullcrack_a_gain_replacement_has_no_event_to_replace`
/// - *"If an effect says to set a player's life total to a certain number and
///   that number is higher than the player's current life total, that part of
///   the effect won't do anything."* → RE-6's, as above.
pub fn skullcrack() -> Arc<CardData> {
    CardDataBuilder::new("Skullcrack")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .rules_text(
            "Players can't gain life this turn. Damage can't be prevented this turn. Skullcrack deals 3 damage to target player or planeswalker.",
        )
        .ability(spell_ability("Players can't gain life this turn. Damage can't be prevented this turn. Skullcrack deals 3 damage to target player or planeswalker.",
            Effect::Sequence(vec![
                // "Players can't gain life this turn." No object and every
                // player, so the row is complete as authored and the
                // resolution's target — the player it then damages — is not
                // what it is about.
                cant(Restriction::Event { pattern: EventPattern::GainLife, affected: AffectedSet::players(PlayerSet::Everyone), by: None })
                    .until_end_of_turn(),
                // "Damage can't be prevented this turn." CR 615.12's row, whose
                // two halves are a union: every object and every player.
                cant(
                    Restriction::ApplyReplacement {
                        kind: ReplacementKindFilter::Prevention,
                        affected: AffectedSet { objects: ObjectSet::battlefield_filter(ObjectFilter::All), players: PlayerSet::Everyone },
                    },
                )
                .until_end_of_turn(),
                deal_damage(target_player(), 3),
            ]),
        ))
        .build()
}
