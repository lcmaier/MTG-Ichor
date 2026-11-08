//! Fixtures for the choices a card asks for (`choices-architecture.md`). A
//! division is announced on a cast, an activation and a trigger alike (CR
//! 601.2d, 602.2b, 603.3d), and divides damage, counters and shields; so are
//! modes (CR 700.2a, 700.2b), whose counts the last fixtures cover.

use std::sync::Arc;

use crate::cards::authoring::{
    activated_ability, an_opponent_chooses_one, an_opponent_controls, another, any_target, artifact, as_long_as, choose, choose_one, choose_one_or_both_if,
    choose_pawprints_worth, choose_repeatable, choose_up_to, creature, creatures_you_control, deal_damage, deal_damage_divided, destroy, dies,
    discard, distribute_counters, draw_cards, each, enters, exile, gain_life, gets, has_base_power_and_toughness, lose_life,
    may_choose_two_each_targeting_a_different_player, mill, planeswalker, prevent_the_next_damage_divided, return_to_the_battlefield,
    spell_ability, spree, static_ability, tap, target_creature, target_permanent, target_player, targets, that_target, this_object,
    triggered_ability, whenever, you, you_have,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    AmountExpr, Condition, CounterType, DiscardChooser, Effect, EffectRecipient, ModalCount, ObjectFilter, PlayerFact, PlayerRef, PlayerSet,
    ReturnUnder, SelectionFilter, TargetCount,
};
use crate::types::mana::{ManaCost, ManaSymbol, ManaType};
use crate::types::triggers::{Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerSubject};

/// Omnath, Locus of Mana Fixture — {2}{G}
/// Creature — Elemental, 1/1.
///
/// Omnath's second line, its power and toughness read off the pool, which is
/// what makes CR 601.2h's order change a game: sacrificed before or after the
/// mana is paid, it last existed as a different creature (`ATOM-601.2h-003`).
/// Its first line, "You don't lose unspent green mana as steps and phases
/// end", is `mana-provenance`'s, so this is not the card. Castable and not
/// registered: the order is a question no registered card's game asks.
pub fn omnath_locus_of_mana_fixture() -> Arc<CardData> {
    let text = "This creature gets +1/+1 for each unspent green mana you have.";
    let per_green = || AmountExpr::UnspentMana(ManaType::Green);
    CardDataBuilder::new("Omnath, Locus of Mana Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Green], 2))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elemental))
        .power_toughness(1, 1)
        .rules_text(text)
        .ability(static_ability(text, gets(this_object(), per_green(), per_green())))
        .build()
}

/// Tithing Engine — {2}
/// Artifact.
///
/// **A fixture, and an invented name.** Susur Secundi, Void Altar's cost without
/// its mana: two parts that each make an event, neither reading the other,
/// which only a watcher of one can tell apart.
pub fn tithing_engine() -> Arc<CardData> {
    let text = "Pay 2 life, Sacrifice a creature: Draw a card.";
    CardDataBuilder::new("Tithing Engine")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Artifact)
        .rules_text(text)
        .ability(activated_ability(text, vec![Cost::PayLife(2), Cost::Sacrifice(creature(), 1)], draw_cards(you(), 1)))
        .build()
}

/// Reckoner of Debts — {1}{B}
/// Enchantment.
///
/// **A fixture, and an invented name.** CR 603.4's intervening "if", read as
/// the event happens, so a payment's other part decides whether it triggers.
pub fn reckoner_of_debts() -> Arc<CardData> {
    let text = "Whenever a creature dies, if you have 10 or less life, you gain 5 life.";
    let def = TriggerDef {
        intervening_if: Some(Condition::Player { players: PlayerSet::You, fact: PlayerFact::LifeAtMost(AmountExpr::Fixed(10)) }),
        ..whenever(dies(TriggerSubject::Filter(creature())), gain_life(you(), 5))
    };
    CardDataBuilder::new("Reckoner of Debts")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(text, def))
        .build()
}

/// Grudge Clerk — {4}{R}
/// Creature — Warrior, 4/4.
///
/// **A fixture, and an invented name.** Sacrificed beside a life payment, it
/// sees the payment only if it was still there.
pub fn grudge_clerk() -> Arc<CardData> {
    let text = "Whenever you lose life, you gain 1 life.";
    let def = TriggerDef {
        condition: TriggerCondition::Event(TriggerEvent::LosesLife { player: Some(PlayerRef::You), multiplicity: Multiplicity::PerOccurrence }),
        intervening_if: None,
        limit: None,
        effect: gain_life(you(), 1),
    };
    CardDataBuilder::new("Grudge Clerk")
        .mana_cost(ManaCost::build(&[ManaType::Red], 4))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Warrior))
        .power_toughness(4, 4)
        .rules_text(text)
        .ability(triggered_ability(text, def))
        .build()
}

/// Forked Reliquary — {3}
/// Artifact.
///
/// **A fixture, and an invented name.** Three parts where each pair matters:
/// the {T} must come ahead of the sacrifice of this artifact, and whether the
/// other creature last existed with its +1/+1 turns on the {T}. An order of all
/// three that completes is asked for, and the orders that do not are refused.
pub fn forked_reliquary() -> Arc<CardData> {
    let anthem = "As long as this artifact is untapped, other creatures you control get +1/+1.";
    let activated = "{T}, Sacrifice this artifact, Sacrifice another creature: Draw a card.";
    CardDataBuilder::new("Forked Reliquary")
        .mana_cost(ManaCost::build(&[], 3))
        .card_type(CardType::Artifact)
        .rules_text("As long as this artifact is untapped, other creatures you control get +1/+1.\n{T}, Sacrifice this artifact, Sacrifice another creature: Draw a card.")
        .ability(static_ability(anthem, as_long_as(Condition::SourceUntapped, gets(each(another(creatures_you_control())), 1, 1))))
        .ability(activated_ability(activated, vec![Cost::TapSelf, Cost::SacrificeSelf, Cost::Sacrifice(another(creature()), 1)], draw_cards(you(), 1)))
        .build()
}

/// Sentry of the Tapped Gate — {1}{W}
/// Creature — Soldier, 2/2.
///
/// **A fixture, and an invented name.** A static ability that reaches only its
/// own object, and reads only its own tapped status.
pub fn sentry_of_the_tapped_gate() -> Arc<CardData> {
    let text = "As long as this creature is untapped, it gets +1/+1.";
    CardDataBuilder::new("Sentry of the Tapped Gate")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Soldier))
        .power_toughness(2, 2)
        .rules_text(text)
        .ability(static_ability(text, as_long_as(Condition::SourceUntapped, gets(this_object(), 1, 1))))
        .build()
}

/// Kudo, King Among Bears Fixture — {G}{W}
/// Creature — Bear, 2/2.
///
/// Kudo's power and toughness half, not legendary so that two can stand on
/// one board: a layer 7b effect on every other creature, whose order against
/// [`godhead_of_awe_fixture`]'s is CR 613.7m's.
pub fn kudo_king_among_bears_fixture() -> Arc<CardData> {
    let text = "Other creatures have base power and toughness 2/2.";
    CardDataBuilder::new("Kudo, King Among Bears Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::White], 0))
        .color(Color::Green)
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Bear))
        .power_toughness(2, 2)
        .rules_text(text)
        .ability(static_ability(text, has_base_power_and_toughness(each(another(creature())), 2, 2)))
        .build()
}

/// Godhead of Awe Fixture — {5}
/// Creature — Spirit, 4/4.
///
/// Godhead of Awe's second line, unhybrid and without its flying.
pub fn godhead_of_awe_fixture() -> Arc<CardData> {
    let text = "Other creatures have base power and toughness 1/1.";
    CardDataBuilder::new("Godhead of Awe Fixture")
        .mana_cost(ManaCost::build(&[], 5))
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Spirit))
        .power_toughness(4, 4)
        .rules_text(text)
        .ability(static_ability(text, has_base_power_and_toughness(each(another(creature())), 1, 1)))
        .build()
}

/// Scattering Flicker — {2}{U}
/// Instant.
///
/// **A fixture, and an invented name.** One event putting several players'
/// objects onto the battlefield at once, as Living Death does.
pub fn scattering_flicker() -> Arc<CardData> {
    let text = "Exile any number of target creatures, then return those cards to the battlefield under their owners' control.";
    CardDataBuilder::new("Scattering Flicker")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                exile(EffectRecipient::Target(SelectionFilter::Creature, TargetCount::AnyNumber)),
                return_to_the_battlefield(that_target(0), ReturnUnder::Owner),
            ]),
        ))
        .build()
}

/// Mogg Mob Fixture — {2}{R}
/// Creature — Goblin, 3/3.
///
/// A division on an activation (CR 602.2b), three among up to three, so two
/// targets ask. Sacrificing it is the cost, so the amount and the shares are
/// fixed before its source is gone, and the source deals the damage as it last
/// existed (CR 113.7a).
pub fn mogg_mob_fixture() -> Arc<CardData> {
    let text = "Sacrifice this creature: It deals 3 damage divided as you choose among one, two, or three targets.";
    CardDataBuilder::new("Mogg Mob Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Goblin))
        .power_toughness(3, 3)
        .rules_text(text)
        .ability(activated_ability(text, vec![Cost::SacrificeSelf], deal_damage_divided(SelectionFilter::Any, TargetCount::Between(1, 3), 3)))
        .build()
}

/// Dragonlord Atarka Fixture — {5}{R}{G}
/// Creature — Dragon, 8/8, unlegendary.
///
/// A division on a trigger (CR 603.3d), among "any number" of a filtered
/// target, the shape `tdc` prints.
pub fn dragonlord_atarka_fixture() -> Arc<CardData> {
    let text = "When this creature enters, it deals 5 damage divided as you choose among any number of target creatures and/or planeswalkers your opponents control.";
    let theirs = SelectionFilter::Permanent(creature().or(planeswalker()).and(an_opponent_controls()));
    CardDataBuilder::new("Dragonlord Atarka Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Green], 5))
        .color(Color::Red)
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Dragon))
        .power_toughness(8, 8)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(enters(TriggerSubject::ThisObject), deal_damage_divided(theirs, TargetCount::AnyNumber, 5)),
        ))
        .build()
}

/// Magma Opus Fixture — {4}{R}, a sorcery: its first two sentences.
///
/// Two instances of "target", one divided: CR 601.2d comes after 601.2c, so
/// the second instance is named before the first is divided.
pub fn magma_opus_fixture() -> Arc<CardData> {
    let text = "Magma Opus Fixture deals 4 damage divided as you choose among any number of targets. Tap two target permanents.";
    CardDataBuilder::new("Magma Opus Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Red], 4))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                deal_damage_divided(SelectionFilter::Any, TargetCount::AnyNumber, 4),
                tap(targets(2, SelectionFilter::Permanent(ObjectFilter::All))),
            ]),
        ))
        .build()
}

/// Elven Rite Fixture — {1}{G}, a sorcery.
///
/// Counters divided, the other half of CR 601.2d's "damage or counters".
pub fn elven_rite_fixture() -> Arc<CardData> {
    let text = "Distribute two +1/+1 counters among one or two target creatures.";
    CardDataBuilder::new("Elven Rite Fixture")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            distribute_counters(SelectionFilter::Creature, TargetCount::Between(1, 2), CounterType::PlusOnePlusOne, 2),
        ))
        .build()
}

/// Remedy Fixture — {1}{W}, an instant.
///
/// A prevention divided: each target a shield of its own share (CR 615.7).
pub fn remedy_fixture() -> Arc<CardData> {
    let text = "Prevent the next 5 damage that would be dealt this turn to any number of targets, divided as you choose.";
    CardDataBuilder::new("Remedy Fixture")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, prevent_the_next_damage_divided(SelectionFilter::Any, TargetCount::AnyNumber, 5)))
        .build()
}

/// Rolling Thunder Fixture — {X}{R}, a sorcery.
///
/// A total the X announced before the targets sets (CR 601.2b, 601.2d).
pub fn rolling_thunder_fixture() -> Arc<CardData> {
    let text = "Rolling Thunder Fixture deals X damage divided as you choose among any number of targets.";
    CardDataBuilder::new("Rolling Thunder Fixture")
        .mana_cost(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Red)]))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(text, deal_damage_divided(SelectionFilter::Any, TargetCount::AnyNumber, AmountExpr::X)))
        .build()
}

/// Meteor Swarm Fixture — {X}{R}, a sorcery.
///
/// A count the X sets and a fixed total: past 8 targets, no X can be divided
/// at least one each, and the announcement rewinds.
pub fn meteor_swarm_fixture() -> Arc<CardData> {
    let text = "Meteor Swarm Fixture deals 8 damage divided as you choose among X target creatures.";
    CardDataBuilder::new("Meteor Swarm Fixture")
        .mana_cost(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Red)]))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            deal_damage_divided(SelectionFilter::Creature, TargetCount::ExactlyAmount(AmountExpr::X), 8),
        ))
        .build()
}

// ---------------------------------------------------------------------------
// CR 700.2's modes
// ---------------------------------------------------------------------------

/// Branching Beacon — {2}, an artifact.
///
/// ATOM-603.3c-001's board, which names no card: a modal trigger whose first
/// mode targets a creature and whose second targets nothing, on a source that
/// is no creature itself, so an empty battlefield leaves the second mode alone.
pub fn branching_beacon() -> Arc<CardData> {
    let text = "When this artifact enters, choose one —\n• This artifact deals 2 damage to target creature.\n• You gain 3 life.";
    CardDataBuilder::new("Branching Beacon")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Artifact)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(enters(TriggerSubject::ThisObject), choose_one([deal_damage(target_creature(), 2), gain_life(you(), 3)])),
        ))
        .build()
}

/// Forked Omen — {1}{B}, an enchantment.
///
/// ATOM-603.3c-002's board, which names no card: a modal trigger every mode
/// of which targets, so with no artifact or creature on the battlefield no
/// mode can be chosen and the ability is removed from the stack.
pub fn forked_omen() -> Arc<CardData> {
    let text = "When this enchantment enters, choose one —\n• Destroy target artifact.\n• Destroy target creature.";
    CardDataBuilder::new("Forked Omen")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(enters(TriggerSubject::ThisObject), choose_one([destroy(target_permanent(artifact())), destroy(target_creature())])),
        ))
        .build()
}

/// Hesitant Omen — {1}{W}, an enchantment.
///
/// "Choose up to one" on a trigger, Hullbreaker Horror's count: choosing no
/// mode is legal, and CR 603.3c removes an ability whose controller chose
/// none. Hullbreaker's trigger is a spell cast and its first mode targets a
/// spell; this one enters, so a test says when it triggers.
pub fn hesitant_omen() -> Arc<CardData> {
    let text = "When this enchantment enters, choose up to one —\n• Destroy target artifact.\n• You gain 2 life.";
    CardDataBuilder::new("Hesitant Omen")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(enters(TriggerSubject::ThisObject), choose_up_to(1, [destroy(target_permanent(artifact())), gain_life(you(), 2)])),
        ))
        .build()
}

/// Mode Lever — {1}, an artifact.
///
/// A modal activated ability (CR 700.2a, 602.2b): its modes are announced as
/// it is activated, before its targets. Bow of Nylea's shape; the Bow's fourth
/// mode targets cards in a graveyard, which no clause can announce yet.
pub fn mode_lever() -> Arc<CardData> {
    let text = "{1}, {T}: Choose one —\n• Target creature gets +1/+1 until end of turn.\n• You gain 1 life.";
    CardDataBuilder::new("Mode Lever")
        .mana_cost(ManaCost::build(&[], 1))
        .card_type(CardType::Artifact)
        .rules_text(text)
        .ability(activated_ability(
            text,
            vec![Cost::Mana(ManaCost::build(&[], 1)), Cost::TapSelf],
            choose_one([gets(target_creature(), 1, 1).until_end_of_turn(), gain_life(you(), 1)]),
        ))
        .build()
}

/// Tally of Choices — {X}{U}, a sorcery.
///
/// "Choose X" (Doomsday Confluence's count, without its repeats, which
/// `mode-rules-and-costs` builds): X is asked before the modes, which are the
/// sets of that many (`choices-architecture.md`, decision 2b). Past three, no
/// set of modes is that large, and the cast rewinds.
pub fn tally_of_choices() -> Arc<CardData> {
    let text = "Choose X —\n• Draw a card.\n• You gain 2 life.\n• Target player mills two cards.";
    CardDataBuilder::new("Tally of Choices")
        .mana_cost(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Blue)]))
        .color(Color::Blue)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            choose(ModalCount::X, [draw_cards(you(), 1), gain_life(you(), 2), mill(target_player(), 2)]),
        ))
        .build()
}

/// Echoing Reckoning — {2}{R}, a sorcery.
///
/// Moment of Reckoning's count, "Choose up to four. You may choose the same
/// mode more than once." (CR 700.2d): a mode chosen past the number of modes,
/// its copies resolving in sequence. Its modes are ATOM-700.2d-002's; the
/// Moment's second targets a card in a graveyard, which no clause can
/// announce until `ironworks-loop` builds graveyard targets.
pub fn echoing_reckoning() -> Arc<CardData> {
    let text = "Choose up to four. You may choose the same mode more than once.\n• This spell deals 2 damage to target creature.\n• Draw a card.";
    CardDataBuilder::new("Echoing Reckoning")
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            choose_repeatable(ModalCount::Range { fewest: 0, most: 4 }, [deal_damage(target_creature(), 2), draw_cards(you(), 1)]),
        ))
        .build()
}

/// Pawprint Parade — {1}{G}, a sorcery.
///
/// CR 700.2i's budget, ATOM-700.2i-001's board: modes of one, two and one
/// pawprints under a budget of three, so the second and third together are
/// legal and all three are not. Bloomburrow's Seasons print five {P} over
/// modes of one, two and three, and each prints a mode no card writes yet:
/// Season of the Burrow returns a card from a graveyard (`ironworks-loop`),
/// Season of Gathering chooses a card type as it resolves
/// (`values-chosen-at-resolution`).
pub fn pawprint_parade() -> Arc<CardData> {
    let text = "Choose up to three {P} worth of modes. You may choose the same mode more than once.\n{P} — You gain 1 life.\n{P}{P} — Draw a card.\n{P} — Target player mills two cards.";
    CardDataBuilder::new("Pawprint Parade")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            choose_pawprints_worth(3, [(1, gain_life(you(), 1)), (2, draw_cards(you(), 1)), (1, mill(target_player(), 2))]),
        ))
        .build()
}

/// Tribute Offering — {1}{R}, an instant.
///
/// CR 700.2e's other chooser, ATOM-700.2e-001's board: "an opponent chooses
/// one", the targets still the caster's. Fatal Lore prints it, and its second
/// mode's "that player", the opponent who chose, has no player word yet
/// (`multiplayer-rules`).
pub fn tribute_offering() -> Arc<CardData> {
    let text = "An opponent chooses one —\n• You draw a card.\n• This spell deals 3 damage to target creature.";
    CardDataBuilder::new("Tribute Offering")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(text, an_opponent_chooses_one([draw_cards(you(), 1), deal_damage(target_creature(), 3)])))
        .build()
}

/// Divided Envoy — {3}{W}{B}, a creature, 3/3.
///
/// Shadrix Silverquill's trigger, which waits for `multiplayer-rules`: "you
/// may choose two. Each mode must target a different player." — zero modes or
/// two, never one (its ruling), and no player the target of both. Its first
/// two modes are the Dragon's in words; its third, "puts a +1/+1 counter on
/// each creature they control", names its target player in a filter, which
/// no recipient can declare yet, so a mode that gains life stands in. An
/// enters trigger in place of the Dragon's beginning of combat.
pub fn divided_envoy() -> Arc<CardData> {
    let text = "When this creature enters, you may choose two. Each mode must target a different player.\n• Target player mills two cards.\n• Target player draws a card and loses 1 life.\n• Target player gains 2 life.";
    let draws_and_loses = Effect::Sequence(vec![draw_cards(target_player(), 1), lose_life(that_target(0), 1)]);
    CardDataBuilder::new("Divided Envoy")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::Black], 3))
        .color(Color::White)
        .color(Color::Black)
        .card_type(CardType::Creature)
        .power_toughness(3, 3)
        .rules_text(text)
        .ability(triggered_ability(
            text,
            whenever(
                enters(TriggerSubject::ThisObject),
                may_choose_two_each_targeting_a_different_player([mill(target_player(), 2), draws_and_loses, gain_life(target_player(), 2)]),
            ),
        ))
        .build()
}

/// Tithed Ultimatum — {1}{B}, a sorcery.
///
/// CR 700.2h's own cost on each mode, ATOM-700.2h-001's board: one mode
/// behind a life payment, the other behind a sacrifice, each paid as an
/// additional cost when chosen and both when both are. Spree's shape (CR
/// 702.172a); a printed spree card's costs are mana.
pub fn tithed_ultimatum() -> Arc<CardData> {
    let text = "Spree\n+ Pay 2 life — This spell deals 3 damage to any target.\n+ Sacrifice a creature — Destroy target creature.";
    CardDataBuilder::new("Tithed Ultimatum")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(spell_ability(
            text,
            spree([
                (vec![Cost::PayLife(2)], deal_damage(any_target(), 3)),
                (vec![Cost::Sacrifice(creature(), 1)], destroy(target_creature())),
            ]),
        ))
        .build()
}

/// Kindled Option — {1}{R}, an instant.
///
/// The Will of the [clan] cycle's count on a condition the engine reads: one
/// mode, or either or both while its caster controls an artifact as the modes
/// are chosen. Soul Transfer prints the count on "an artifact and an
/// enchantment", and its second mode targets a card in a graveyard, which no
/// clause can announce yet. Its second mode is Prismari Command's, whose
/// "then discards" names the mode's own first instance of "target": the
/// spell's second when both are chosen.
pub fn kindled_option() -> Arc<CardData> {
    let text = "Choose one. If you control an artifact as you cast this spell, you may choose both instead.\n• This spell deals 2 damage to target creature.\n• Target player draws a card, then discards a card.";
    let draws_then_discards =
        Effect::Sequence(vec![draw_cards(target_player(), 1), discard(that_target(0), 1, DiscardChooser::Affected)]);
    CardDataBuilder::new("Kindled Option")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(
            text,
            choose_one_or_both_if(you_have(PlayerFact::ControlsPermanent(artifact())), [deal_damage(target_creature(), 2), draws_then_discards]),
        ))
        .build()
}
