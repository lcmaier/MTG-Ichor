//! Limited Edition Alpha (`lea`), released 1993-08-05.

use std::sync::Arc;

use crate::cards::authoring::{
    activated_ability, add_mana, any_target, colored, counter_spell, create_replacement, create_replacement_from_a_chosen_source, creature,
    deal_damage, draw_cards, every_permanent_or_player, gain_life, gets, has_base_power_and_toughness, host, its_controller_adds, land, mana_ability,
    non, prevent_the_next_n_damage_this_turn, spell_ability, static_ability, static_replacement, take_an_extra_turn_after_this_one, target_creature,
    target_player, target_spell, the_next_damage_from_a_chosen_source, this_object, triggered_ability, whenever, with_subtype, you, you_control,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, EnchantmentType, LandType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{AffectedSet, AmountExpr, ManaOutput, ObjectSet, SelectionFilter};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{CopyDonor, EntryCopyTemplate, EventPattern, ReplacementDef, Rewrite};
use crate::types::triggers::{TriggerEvent, TriggerSubject};

/// Ancestral Recall — {U}
/// Instant
/// Target player draws 3 cards.
pub fn ancestral_recall() -> Arc<CardData> {
    CardDataBuilder::new("Ancestral Recall")
        .card_type(CardType::Instant)
        .color(Color::Blue)
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .rules_text("Target player draws three cards.")
        .ability(spell_ability(
            "Target player draws three cards.",
            draw_cards(target_player(), 3),
        ))
        .build()
}

/// The mana a basic land type produces (CR 305.6).
fn mana_for(land_type: LandType) -> ManaType {
    match land_type {
        LandType::Plains => ManaType::White,
        LandType::Island => ManaType::Blue,
        LandType::Swamp => ManaType::Black,
        LandType::Mountain => ManaType::Red,
        LandType::Forest => ManaType::Green,
        other => panic!("{other:?} is not a basic land type and produces no mana"),
    }
}

/// Build a dual land from its two basic land types.
///
/// Subtype order follows the printed type line, which is also the order
/// `land_types::basic_land_types_sorted` would produce — irrelevant to behavior,
/// but it keeps the fixture readable next to the real card.
///
/// **One modeling shortcut, recorded rather than hidden.** CR 305.6 makes a
/// land's mana abilities *intrinsic to its basic land types* — the reminder
/// text in parentheses on the printed card is not rules text. Each dual gets
/// two explicit `AbilityType::Mana` abilities instead, because base
/// characteristics are read straight from `CardData` and nothing derives
/// abilities from printed subtypes. The two models agree everywhere the
/// engine can observe: CR 305.7 clears `chars.abilities` wholesale before
/// granting the new intrinsic, and Humility removes mana abilities from an
/// animated land either way (per its Scryfall ruling). Revisit if an effect
/// ever needs to distinguish an intrinsic ability from a printed one.
fn dual(name: &str, first: LandType, second: LandType) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Land)
        .subtype(Subtype::Land(first))
        .subtype(Subtype::Land(second))
        .mana_ability_single(mana_for(first))
        .mana_ability_single(mana_for(second))
        .build()
}

/// Badlands — Land — Swamp Mountain
pub fn badlands() -> Arc<CardData> {
    dual("Badlands", LandType::Swamp, LandType::Mountain)
}

/// Bayou — Land — Swamp Forest
pub fn bayou() -> Arc<CardData> {
    dual("Bayou", LandType::Swamp, LandType::Forest)
}

/// Circle of Protection: Red — {1}{W}
///
/// > {1}: The next time a red source of your choice would deal damage to you
/// > this turn, prevent that damage.
///
/// **Both halves of CR 609.7 on one card**, which is why it is RD-3's first
/// consumer: `object` is 609.7a's chosen source, written by the resolution,
/// and `filter` is 609.7b's property, rechecked when the damage comes. The two
/// disagree the moment the chosen creature stops being red, and the rule says
/// the shield then neither applies nor is used up.
///
/// `Uses::Once` with a whole-event [`Rewrite::Prevent`] is CR 615.8: "these
/// effects prevent the next instance of damage from that source, **regardless
/// of how much damage that is**".
///
/// # The rulings (Scryfall, 2026-09-08), and where each is tested
///
/// - *A source of damage is a permanent, a spell on the stack (including one
///   that creates a permanent), or any object referred to by an object on the
///   stack. A source doesn't need to be capable of dealing damage to be a legal
///   choice.* → `SelectionFilter::DamageSource`, tested in `oracle::legality`
///   and, for the referred-to category, in `phase_tr3c_integration_test`.
/// - *Can be used even when there is no damage to prevent. It prevents the next
///   damage (if any) from the source this turn.* → the row sits unused and
///   expires at the cleanup step (CR 615.3).
pub fn circle_of_protection_red() -> Arc<CardData> {
    CardDataBuilder::new("Circle of Protection: Red")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text(
            "{1}: The next time a red source of your choice would deal damage to you this \
             turn, prevent that damage.",
        )
        .ability(activated_ability("{1}: The next time a red source of your choice would deal damage to you this turn, prevent that damage.",
            vec![Cost::Mana(ManaCost::build(&[], 1))],
            create_replacement_from_a_chosen_source(the_next_damage_from_a_chosen_source(Some(colored(Color::Red)), Rewrite::Prevent))
                .until_end_of_turn(),
        ))
        .build()
}

/// Clone — {3}{U}
/// Creature — Shapeshifter, 0/0
///
/// You may have this creature enter as a copy of any creature on the
/// battlefield.
///
/// (Oracle text verified on Scryfall, 2026-09-28.)
///
/// # CV-2a's card: a chosen donor and no "except"
///
/// 52 of the 62 printed cards that enter as a copy choose their donor on the
/// battlefield, as Clone does, and 42 of the 62 print "except". Clone is the
/// base both kinds share; Spark Double, CV-2b's card, is the "except" half
/// (`copy-effects-architecture.md` §4.1a).
///
/// A `SourceOnly` entry replacement whose rewrite is
/// `Rewrite::EnterAsCopy`, so CR 616.1c's bucket has a printed producer and
/// the copy is the permanent's state from the moment it arrives (CV-2a's
/// D1, in `plans/archive/copy-effects-architecture-landed.md`). The donor is
/// chosen, not targeted, by the entering object's controller (CR 707.6), and
/// the "you may" is that choice's empty pick (D4). An unchosen Clone is the
/// 0/0 its card says, and CR 704.5f takes it.
///
/// In `PERFORMANCE_POOL`: it is the only card that opens the entry-copy path,
/// and a path no pooled card opens is RS-1's failure. At `{3}{U}` a random
/// game casts it, and it copies in about half its entries, since the random
/// provider declines a `(0, 1)` pick half the time.
///
/// # The rulings, and where each is tested
///
/// All eight are linked from `tests/phase_cv2a_integration_test.rs`: not a
/// target (1), X as 0 (2), the copied "enters" abilities (3), copying nothing
/// (4), nothing entering beside it (5), a token's values without being a
/// token (6), a copy's copy (7), and what is not copied (8).
pub fn clone() -> Arc<CardData> {
    CardDataBuilder::new("Clone")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 3))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Shapeshifter))
        .power_toughness(0, 0)
        .rules_text("You may have this creature enter as a copy of any creature on the battlefield.")
        .ability(static_replacement(
            "You may have this creature enter as a copy of any creature on the battlefield.",
            ReplacementDef {
                optional: true,
                ..ReplacementDef::new(
                    EventPattern::EnterBattlefield { cast: None },
                    ObjectSet::SourceOnly,
                    Rewrite::EnterAsCopy(EntryCopyTemplate {
                        donor: CopyDonor::Chosen(SelectionFilter::Creature),
                        except: Vec::new(),
                    }),
                )
            },
        ))
        .build()
}

/// Counterspell — {U}{U}
/// Instant
/// Counter target spell.
pub fn counterspell() -> Arc<CardData> {
    CardDataBuilder::new("Counterspell")
        .card_type(CardType::Instant)
        .color(Color::Blue)
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 0))
        .rules_text("Counter target spell.")
        .ability(spell_ability(
            "Counter target spell.",
            counter_spell(target_spell()),
        ))
        .build()
}

/// Dark Ritual - {B}
/// Instant
/// Add {B}{B}{B}.
pub fn dark_ritual() -> Arc<CardData> {
    CardDataBuilder::new("Dark Ritual")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Instant)
        .rules_text("Add {B}{B}{B}.")
        .ability(spell_ability(
            "Add {B}{B}{B}.",
            add_mana(ManaOutput { mana: vec![(ManaType::Black, AmountExpr::Fixed(3))], special: vec![] }),
        ))
        .build()
}

/// Earth Elemental — {3}{R}{R}
/// Creature — Elemental
/// 4/5
pub fn earth_elemental() -> Arc<CardData> {
    CardDataBuilder::new("Earth Elemental")
        .card_type(CardType::Creature)
        .color(Color::Red)
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 3))
        .power_toughness(4, 5)
        .build()
}

/// Elvish Archers — {1}{G}
/// Creature — Elf Archer
/// 2/1 First Strike
pub fn elvish_archers() -> Arc<CardData> {
    CardDataBuilder::new("Elvish Archers")
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elf))
        .subtype(Subtype::Creature(CreatureType::Archer))
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .power_toughness(2, 1)
        .keyword_flag(KeywordFlag::FirstStrike)
        .rules_text("First strike")
        .build()
}

/// Fog — {G}
///
/// > Prevent all combat damage that would be dealt this turn.
///
/// The `combat` field's only consumer, and the widest set in the phase:
/// `Filter { All }` beside `PlayerSet::Everyone`, so it covers damage to
/// anything and anyone — including the caster's own opponents' creatures,
/// which is what "all" means and what a `You`-scoped shield would get wrong.
///
/// `Uses::Static` on a registry row with a duration: the row is not spent by
/// preventing, it expires at the cleanup step (CR 615.3).
///
/// Scryfall lists no rulings (2026-09-08). The rule's own boundary is the
/// test: **non-combat** damage under Fog goes through.
pub fn fog() -> Arc<CardData> {
    let (affected, players) = every_permanent_or_player();
    CardDataBuilder::new("Fog")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Instant)
        .rules_text("Prevent all combat damage that would be dealt this turn.")
        .ability(spell_ability("Prevent all combat damage that would be dealt this turn.",
            create_replacement(ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: Some(true) },
                AffectedSet { objects: affected, players },
                Rewrite::Prevent,
            ))
            .until_end_of_turn(),
        ))
        .build()
}

/// Create the CardData for a Forest
pub fn forest() -> Arc<CardData> {
    CardDataBuilder::new("Forest")
        .card_type(CardType::Land)
        .supertype(Supertype::Basic)
        .subtype(Subtype::Land(LandType::Forest))
        .mana_ability_single(ManaType::Green)
        .build()
}

/// Giant Growth — {G}
/// Instant
/// Target creature gets +3/+3 until end of turn.
pub fn giant_growth() -> Arc<CardData> {
    CardDataBuilder::new("Giant Growth")
        .card_type(CardType::Instant)
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .rules_text("Target creature gets +3/+3 until end of turn.")
        .ability(spell_ability(
            "Target creature gets +3/+3 until end of turn.",
            gets(target_creature(), 3, 3).until_end_of_turn(),
        ))
        .build()
}

/// Giant Spider — {3}{G}
/// Creature — Spider
/// 2/4 Reach
pub fn giant_spider() -> Arc<CardData> {
    CardDataBuilder::new("Giant Spider")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .power_toughness(2, 4)
        .keyword_flag(KeywordFlag::Reach)
        .rules_text("Reach")
        .build()
}

/// Grizzly Bears — {1}{G}
/// Creature — Bear
/// 2/2
pub fn grizzly_bears() -> Arc<CardData> {
    CardDataBuilder::new("Grizzly Bears")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .power_toughness(2, 2)
        .build()
}

/// Hill Giant — {3}{R}
/// Creature — Giant
/// 3/3
pub fn hill_giant() -> Arc<CardData> {
    CardDataBuilder::new("Hill Giant")
        .card_type(CardType::Creature)
        .color(Color::Red)
        .mana_cost(ManaCost::build(&[ManaType::Red], 3))
        .power_toughness(3, 3)
        .build()
}

/// Holy Strength — {W}
/// Enchantment — Aura
///
/// Enchant creature
/// Enchanted creature gets +1/+2.
///
/// (Oracle text verified on Scryfall, 2026-09-04. Alpha; Magic 2011 printing.)
///
/// # Why this one out of 26
///
/// Scryfall lists 26 Auras whose entire oracle text is "Enchant creature" and
/// one P/T sentence, and they differ only in cost and sign. Two things pick
/// this one. **The bonus is positive**, so the host survives the attach and
/// the Aura stays on the battlefield to be the *subject* of CR 704.5m/n
/// rather than dying with a host it killed (Dead Weight, Weakness). **And it
/// is asymmetric**: +1/+2 is the smallest bonus where a row that swapped
/// power for toughness, or applied twice, would fail an assertion — River's
/// Favor's +1/+1 would pass both. One white pip is the cheapest that leaves.
///
/// # What it makes reachable
///
/// - **CR 704.5m/n** — the Aura SBAs, measured at **0** across 200 stress games
///   in the 2026-09-01 fuzz re-audit because no registered card was an Aura.
/// - **CR 303.4a / 601.2c** — an Aura spell's target is defined by its enchant
///   ability, not by a spell ability, and this is the first spell in the pool
///   whose recipient comes from `enchant_filter`.
/// - **CR 608.3b** — a permanent spell whose target is gone fizzles. Item 8.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The first static ability in the pool that lowers to
/// `ObjectSet::Host`, so the first row whose membership is a
/// `battlefield` read per candidate per layer rather than a filter match. That
/// is a new arm in `board::affected_members`, and §3 asks that a new engine path be
/// measured rather than assumed.
pub fn holy_strength() -> Arc<CardData> {
    CardDataBuilder::new("Holy Strength")
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .rules_text("Enchant creature\nEnchanted creature gets +1/+2.")
        // CR 702.5a — "Enchant creature" is the Aura's targeting restriction,
        // and CR 303.4a makes it the spell's target.
        .enchant_filter(SelectionFilter::Creature)
        .ability(static_ability(
            "Enchanted creature gets +1/+2.",
            gets(host(), 1, 2),
        ))
        .build()
}

/// Create the CardData for an Island
pub fn island() -> Arc<CardData> {
    CardDataBuilder::new("Island")
        .card_type(CardType::Land)
        .supertype(Supertype::Basic)
        .subtype(Subtype::Land(LandType::Island))
        .mana_ability_single(ManaType::Blue)
        .build()
}

/// Keldon Warlord — {2}{R}{R}
/// Creature — Human Barbarian, */*
///
/// Keldon Warlord's power and toughness are each equal to the number of
/// non-Wall creatures you control.
///
/// (Oracle text verified on Scryfall, 2026-09-02.)
///
/// # §5a's probe — the count that does not see the entering object
///
/// A characteristic-defining ability (CR 604.3) whose value is a *count over
/// the battlefield*, which is the read `replacement-architecture.md` §5a's
/// enumeration row names and nothing in the layer walk had until RC-4:
/// `AmountExpr::CountOf` now has a static-context evaluator, and this is its
/// card. On the battlefield the Warlord counts itself. **In the CR 614.12
/// frame it does not** — the count runs over `battlefield_ids_ordered`, which
/// the entering object is not on. So a Warlord entering beside two other
/// creatures is 2/2 to a *replacement* that reads its power ("creatures with
/// power 2 or less enter tapped") and 3/3 to a *trigger* that does (Welcoming
/// Vampire): CR 603.10 checks a trigger against the board after the event,
/// where the Warlord is one of the creatures it counts. That is the pair of
/// Thassa rulings — for replacements "the mana symbols in its mana cost won't
/// be counted", for triggers the devotion "including the mana symbols in the
/// mana cost of the God itself" — with none of the devotion arithmetic and
/// none of Deferred Migrations item 7f in the way, which is why this card is
/// here and the God is not.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The evaluator is one frame per permanent per query — `layers-architecture.md`
/// §12's quadratic by design — and a card that opens a new engine path adds
/// itself to the measured pool (`engineering-practices.md#cards-and-pools`).
/// Every walk of the Warlord asks every permanent's frame at layer 7a's ceiling,
/// memoized within that walk, so `Frames/walk` is the fixture that moves; the
/// measurement is in RC-4's entry of `codebase-state.md`. Wall of Stone is in
/// the same pool, which is what makes "non-Wall" live.
pub fn keldon_warlord() -> Arc<CardData> {
    // Built once and cloned into the second slot: `SetPowerToughness` takes
    // its two amounts by value, and this runs at registry construction, not
    // in a game.
    let non_wall_creatures_you_control = AmountExpr::CountOf(ObjectSet::battlefield_filter(
        creature().and(you_control()).and(non(with_subtype(Subtype::Creature(CreatureType::Wall)))),
    ));
    CardDataBuilder::new("Keldon Warlord")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Barbarian))
        // `*/*` — the CDA supplies both numbers, in every zone (CR 208.2a).
        .power_toughness(0, 0)
        .rules_text(
            "Keldon Warlord's power and toughness are each equal to the number of \
             non-Wall creatures you control.",
        )
        .ability(static_ability(
            "Keldon Warlord's power and toughness are each equal to the number of non-Wall creatures you control.",
            has_base_power_and_toughness(this_object(), non_wall_creatures_you_control.clone(), non_wall_creatures_you_control),
        ).characteristic_defining())
        .build()
}

/// Lightning Bolt — {R}
/// Instant
/// Lightning Bolt deals 3 damage to any target.
pub fn lightning_bolt() -> Arc<CardData> {
    CardDataBuilder::new("Lightning Bolt")
        .card_type(CardType::Instant)
        .color(Color::Red)
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .rules_text("Lightning Bolt deals 3 damage to any target.")
        .ability(spell_ability(
            "Lightning Bolt deals 3 damage to any target.",
            deal_damage(any_target(), 3),
        ))
        .build()
}

/// Create the CardData for a Mountain
pub fn mountain() -> Arc<CardData> {
    CardDataBuilder::new("Mountain")
        .card_type(CardType::Land)
        .supertype(Supertype::Basic)
        .subtype(Subtype::Land(LandType::Mountain))
        .mana_ability_single(ManaType::Red)
        .build()
}

/// Create the CardData for a Plains
pub fn plains() -> Arc<CardData> {
    CardDataBuilder::new("Plains")
        .card_type(CardType::Land)
        .supertype(Supertype::Basic)
        .subtype(Subtype::Land(LandType::Plains))
        .mana_ability_single(ManaType::White)
        .build()
}

/// Plateau — Land — Mountain Plains
pub fn plateau() -> Arc<CardData> {
    dual("Plateau", LandType::Mountain, LandType::Plains)
}

/// Reverse Damage — {1}{W}{W}
///
/// > The next time a source of your choice would deal damage to you this turn,
/// > prevent that damage. You gain life equal to the damage prevented this way.
///
/// The chosen source with **no property at all** — so 609.7b has nothing to
/// recheck and the id is the whole predicate — and the printed reader of
/// RD-2's prevented-amount channel: `AmountExpr::DamagePrevented` in a
/// CR 615.5 rider, which the fixture in `tests/phase_rd2_integration_test.rs`
/// was standing in for.
///
/// The rider's recipient is `Target`, which a rider resolves against the
/// event's subject — here the player the damage was headed for, who is also
/// the row's affected player.
///
/// # The ruling (Scryfall, 2026-09-08), and where it is tested
///
/// - *It only affects damage dealt by the source one time. If the source
///   damages you a second time this turn, the damage will not be reversed.* →
///   CR 615.8's `Uses::Once`, and the second instance is dealt in full with no
///   life gained.
pub fn reverse_damage() -> Arc<CardData> {
    CardDataBuilder::new("Reverse Damage")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(
            "The next time a source of your choice would deal damage to you this turn, \
             prevent that damage. You gain life equal to the damage prevented this way.",
        )
        .ability(spell_ability("The next time a source of your choice would deal damage to you this turn, prevent that damage. You gain life equal to the damage prevented this way.",
            create_replacement_from_a_chosen_source(
                the_next_damage_from_a_chosen_source(None, Rewrite::Prevent).with_then(gain_life(target_player(), AmountExpr::DamagePrevented)),
            )
            .until_end_of_turn(),
        ))
        .build()
}

/// Samite Healer — {1}{W}
///
/// > {T}: Prevent the next 1 damage that would be dealt to any target this
/// > turn.
///
/// The activated shape — a repeatable source of rows on a creature the random
/// agent will tap — and the card that pins CR 113.7a on the row: an ability's
/// source is the permanent that has it, so the row names the Healer, not the
/// stack object CR 608.2n deletes the moment the ability finishes resolving.
/// Scryfall lists no rulings (2026-09-08).
pub fn samite_healer() -> Arc<CardData> {
    CardDataBuilder::new("Samite Healer")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(1, 1)
        .rules_text("{T}: Prevent the next 1 damage that would be dealt to any target this turn.")
        .ability(activated_ability("{T}: Prevent the next 1 damage that would be dealt to any target this turn.",
            vec![Cost::TapSelf],
            prevent_the_next_n_damage_this_turn(any_target(), 1),
        ))
        .build()
}

/// Savannah — Land — Forest Plains
pub fn savannah() -> Arc<CardData> {
    dual("Savannah", LandType::Forest, LandType::Plains)
}

/// Savannah Lions — {W}
/// Creature — Cat
/// 2/1
pub fn savannah_lions() -> Arc<CardData> {
    CardDataBuilder::new("Savannah Lions")
        .card_type(CardType::Creature)
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .power_toughness(2, 1)
        .build()
}

/// Scrubland — Land — Plains Swamp
pub fn scrubland() -> Arc<CardData> {
    dual("Scrubland", LandType::Plains, LandType::Swamp)
}

/// Serra Angel — {3}{W}{W}
/// Creature — Angel
/// 4/4 Flying, Vigilance
pub fn serra_angel() -> Arc<CardData> {
    CardDataBuilder::new("Serra Angel")
        .card_type(CardType::Creature)
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 3))
        .power_toughness(4, 4)
        .keyword_flag(KeywordFlag::Flying)
        .keyword_flag(KeywordFlag::Vigilance)
        .rules_text("Flying\nVigilance")
        .build()
}

/// Sol Ring — {1}
/// Artifact
/// `{T}: Add {C}{C}.`
///
/// Verbatim, and chosen for three reasons beyond being the most-played card in
/// Commander: it is colorless, so `random_deck` puts it in *every* deck rather
/// than only the ones sharing its colors; its mana value is 1, so under March
/// of the Machines it animates into a 1/1 that survives its own SBA check; and
/// the animated form is a creature with a mana ability, which puts CR 302.6 on
/// a permanent that was not a creature when the turn began.
pub fn sol_ring() -> Arc<CardData> {
    CardDataBuilder::new("Sol Ring")
        .card_type(CardType::Artifact)
        .mana_cost(ManaCost::build(&[], 1))
        .rules_text("{T}: Add {C}{C}.")
        .ability(mana_ability(
            "{T}: Add {C}{C}.",
            vec![Cost::TapSelf],
            add_mana(ManaOutput {
                // One ability producing two, not two abilities producing one:
                // `{C}{C}` is a single activation, and a mana ability that
                // could be activated twice per tap would be a different card.
                mana: vec![(ManaType::Colorless, AmountExpr::Fixed(2))],
                special: vec![],
            }),
        ))
        .build()
}

/// Create the CardData for a Swamp
pub fn swamp() -> Arc<CardData> {
    CardDataBuilder::new("Swamp")
        .card_type(CardType::Land)
        .supertype(Supertype::Basic)
        .subtype(Subtype::Land(LandType::Swamp))
        .mana_ability_single(ManaType::Black)
        .build()
}

/// Taiga — Land — Mountain Forest
pub fn taiga() -> Arc<CardData> {
    dual("Taiga", LandType::Mountain, LandType::Forest)
}

/// Time Walk — {1}{U}
/// Sorcery
///
/// > Take an extra turn after this one.
///
/// The turn queue's only producer, and the card that makes a skip's "next
/// occurrence" mean something: with Meditate's row already in the registry the
/// extra turn is the occurrence that gets skipped, and the natural turn after
/// it begins (CR 614.10a).
///
/// Ruling (2026-09-11): *"If multiple 'extra turn' effects resolve in the same
/// turn, take them in the reverse of the order that the effects resolved."* →
/// CR 500.7's "most recently created turn will be taken first", which is the
/// queue being a stack.
///
/// **Registered and not pooled**: an extra turn in every blue deck moves
/// `Avg turns/game` by design, which is a worse baseline rather than a wider
/// one.
pub fn time_walk() -> Arc<CardData> {
    CardDataBuilder::new("Time Walk")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Sorcery)
        .rules_text("Take an extra turn after this one.")
        .ability(spell_ability("Take an extra turn after this one.",
            take_an_extra_turn_after_this_one(you()),
        ))
        .build()
}

/// Tropical Island — Land — Forest Island
pub fn tropical_island() -> Arc<CardData> {
    dual("Tropical Island", LandType::Forest, LandType::Island)
}

/// Tundra — Land — Plains Island
pub fn tundra() -> Arc<CardData> {
    dual("Tundra", LandType::Plains, LandType::Island)
}

/// Underground Sea — Land — Island Swamp
pub fn underground_sea() -> Arc<CardData> {
    dual("Underground Sea", LandType::Island, LandType::Swamp)
}

/// Wall of Stone — {1}{R}{R}
/// Creature — Wall
/// 0/8 Defender
///
/// The Wall subtype was missing until RC-4 registered Keldon Warlord, whose
/// "non-Wall creatures you control" is the first thing in the crate to read it.
pub fn wall_of_stone() -> Arc<CardData> {
    CardDataBuilder::new("Wall of Stone")
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Wall))
        .color(Color::Red)
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 1))
        .power_toughness(0, 8)
        .keyword_flag(KeywordFlag::Defender)
        .rules_text("Defender")
        .build()
}

/// War Mammoth — {3}{G}
/// Creature — Elephant
/// 3/3 Trample
pub fn war_mammoth() -> Arc<CardData> {
    CardDataBuilder::new("War Mammoth")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .power_toughness(3, 3)
        .keyword_flag(KeywordFlag::Trample)
        .rules_text("Trample")
        .build()
}

/// Wild Growth — {G}
/// Enchantment — Aura
///
/// > Enchant land
/// > Whenever enchanted land is tapped for mana, its controller adds an
/// > additional {G}.
///
/// CR 605.1b's triggered mana ability, derived and never tagged: no target,
/// triggers from mana being added, could add mana — so the dispatcher
/// resolves it at once (CR 605.4a), inside the CR 601.2g window when the
/// land was tapped there, and the stack never sees it. "Its controller adds"
/// is the `Host` recipient: the land's controller, whoever controls the Aura.
///
/// It exercises CR 605.1b's stackless mana trigger inside the mana window.
///
/// # The rulings, and where each is tested
///
/// - *"The additional mana is not an ability of the land and is not
///   something the land can produce."*
///   → `phase_tr1_integration_test::wild_growths_mana_is_the_auras_and_does_not_tap_the_land_for_mana_again`:
///   the record's source is the Aura and `tapped_for_mana` is false, so a
///   second Wild Growth on the same land adds one more {G}, not two.
pub fn wild_growth() -> Arc<CardData> {
    CardDataBuilder::new("Wild Growth")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .rules_text("Enchant land\nWhenever enchanted land is tapped for mana, its controller adds an additional {G}.")
        .enchant_filter(SelectionFilter::Permanent(land()))
        .ability(triggered_ability("Whenever enchanted land is tapped for mana, its controller adds an additional {G}.", whenever(
            TriggerEvent::ManaAdded { source: TriggerSubject::Host, tapped_for_mana: Some(true), mana: None },
            its_controller_adds(host(), ManaOutput { mana: vec![(ManaType::Green, AmountExpr::Fixed(1))], special: vec![] }),
        )))
        .build()
}
