//! Fixtures for the layer system's paths (CR 613, `layers-architecture.md`),
//! with CR 113.6's zones.

use std::sync::Arc;

use crate::cards::authoring::{
    artifact, as_long_as, basic, becomes_colorless, becomes_types, colored, creature, creatures_you_control, draw_cards, each, each_in,
    functioning_in, gains_keyword, gets, has_base_power_and_toughness, host, is_also_color, land, non, spell_ability, static_ability,
    switch_power_and_toughness, target_creature, target_permanent, this_object, with_subtype, you, you_control,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, EnchantmentType, LandType, PlaneswalkerType, Subtype, Subtypes, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, Condition, Effect, ObjectFilter, PlayerFact, PlayerRef, PlayerSet, SelectionFilter, TypeChange};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::zones::ZoneSet;

/// Zhalfirin Shapecraft - {1}{U}
/// Instant
/// Target creature has base power and toughness 4/3 until end of turn.
/// Draw a card.
pub fn zhalfirin_shapecraft() -> Arc<CardData> {
    CardDataBuilder::new("Zhalfirin Shapecraft")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .ability(spell_ability(
            "Target creature has base power and toughness 4/3 until end of turn.\nDraw a card.",
            Effect::Sequence(vec![
                has_base_power_and_toughness(target_creature(), 4, 3).until_end_of_turn(),
                draw_cards(you(), 1),
            ]),
        ))
        .build()
}

/// Inside Out - {1}{U} **as modeled**; the printed cost is {1}{U/R}
/// Instant
/// Switch target creature's power and toughness until end of turn.
/// Draw a card.
///
/// The hybrid symbol is the whole reason this stays a fixture and never reaches
/// `CardRegistry` — hybrid mana is unimplemented (`codebase-state.md`, CR 6),
/// and a registry that feeds `cli_play` may not quietly recost a card. Layer 7d
/// gets its registered source from `printings::merfolk_thaumaturgist`,
/// whose text needs nothing the engine lacks.
pub fn inside_out() -> Arc<CardData> {
    CardDataBuilder::new("Inside Out")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .ability(spell_ability(
            "Switch target creature's power and toughness until end of turn.\nDraw a card.",
            Effect::Sequence(vec![
                switch_power_and_toughness(target_creature()).until_end_of_turn(),
                draw_cards(you(), 1),
            ]),
        ))
        .build()
}

/// Bull Rush - {R}
/// Instant
/// Target creature gets +2/+0 until end of turn.
pub fn bull_rush() -> Arc<CardData> {
    CardDataBuilder::new("Bull Rush")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .ability(spell_ability(
            "Target creature gets +2/+0 until end of turn.",
            gets(target_creature(), 2, 0).until_end_of_turn(),
        ))
        .build()
}

/// Chromatic Ward (made-up) — {1}{R}
/// Enchantment
/// Creatures you control are red in addition to their other colors.
/// (Tests static ability color-adding via Layer 5 + register_static_effects)
pub fn chromatic_ward() -> Arc<CardData> {
    CardDataBuilder::new("Chromatic Ward")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Enchantment)
        .ability(static_ability(
            "Creatures you control are red in addition to their other colors.",
            is_also_color(each(creatures_you_control()), Color::Red),
        ))
        .build()
}

/// Liquimetal Coating (simplified) — {2}
/// Artifact
/// {T}: Target permanent becomes an artifact in addition to its other types
/// until end of turn.
///
/// Modeled as an instant spell for testing (avoids activated ability infrastructure).
/// Tests: AddType (adding Artifact to any permanent).
pub fn liquimetal_coating_spell() -> Arc<CardData> {
    CardDataBuilder::new("Liquimetal Torque")
        .mana_cost(ManaCost::build(&[ManaType::Colorless], 1))
        .card_type(CardType::Instant)
        .ability(spell_ability(
            "Target permanent becomes an artifact in addition to its other types until end of turn.",
            becomes_types(target_permanent(ObjectFilter::All), TypeChange::adding(&[CardType::Artifact], &[], &[]))
                .until_end_of_turn(),
        ))
        .build()
}

/// Ensoul Artifact (simplified) — {1}{U}
/// Instant
/// Target artifact becomes an artifact creature with base power and toughness
/// 5/5 until end of turn.
///
/// Both atoms are one instance of "target" (CR 115.3) — the second declares
/// nothing and refers back to the first.
/// Tests: AddType(Creature) + SetPowerToughness on the same target.
pub fn ensoul_artifact_spell() -> Arc<CardData> {
    CardDataBuilder::new("Ensoul Artifact")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .ability(spell_ability(
            "Target artifact becomes an artifact creature with base power and toughness 5/5 until end of turn.",
            becomes_types(target_permanent(artifact()), TypeChange::adding(&[CardType::Creature], &[], &[]))
                .and_has_base_power_and_toughness(5, 5)
                .until_end_of_turn(),
        ))
        .build()
}

/// Call to Serve (simplified) — {1}{W}
/// Instant (simplified from Aura for testing)
/// Target creature gets +1/+2 and is an Angel in addition to its other
/// types until end of turn.
///
/// Tests: AddSubtype (creature subtype — Angel) combined with P/T pump.
/// Note: the printed card also grants flying. That was omitted because Layer 6
/// had no resolution-time producer; it has one now (`Primitive::GrantKeywordFlag`
/// through `resolve::register_resolution_ability_effect`), so the omission is a
/// choice rather than a limit. Left as-is because this card is an Aura modeled as
/// an Instant, so completing the grant would not make it faithful — see the note
/// in `cards::registry` on why the `*_spell` cards stay unregistered.
pub fn call_to_serve_spell() -> Arc<CardData> {
    CardDataBuilder::new("Call to Serve")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Instant)
        .ability(spell_ability(
            "Target creature gets +1/+2 and is an Angel in addition to its other types until end of turn.",
            becomes_types(target_creature(), TypeChange::adding(&[], &[Subtype::Creature(CreatureType::Angel)], &[]))
                .and_gets(1, 2)
                .until_end_of_turn(),
        ))
        .build()
}

/// On Serra's Wings (simplified) — {3}{W}
/// Instant (simplified from Legendary Enchantment — Aura)
/// Target creature is legendary and gets +1/+1 until end of turn.
///
/// Tests: AddSupertype (Legendary) combined with P/T pump.
/// Note: the printed card also grants flying, vigilance and lifelink. Omitted
/// because Layer 6 had no resolution-time producer; it has one now, so this is a
/// choice rather than a limit. Same reasoning as `call_to_serve_spell`.
pub fn on_serras_wings_spell() -> Arc<CardData> {
    CardDataBuilder::new("On Serra's Wings")
        .mana_cost(ManaCost::build(&[ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Instant)
        .ability(spell_ability(
            "Target creature is legendary and gets +1/+1 until end of turn.",
            becomes_types(target_creature(), TypeChange::adding(&[], &[], &[Supertype::Legendary]))
                .and_gets(1, 1)
                .until_end_of_turn(),
        ))
        .build()
}

/// Urborg-style effect (simplified) — {B}
/// Enchantment
/// Each land is a Swamp in addition to its other land types.
/// (Static ability: filter = all lands, AddSubtype(Swamp))
///
/// **An Enchantment on purpose, and not interchangeable with the real card.**
/// Urborg, Tomb of Yawgmoth is a Legendary *Land*, so it is nonbasic and Blood
/// Moon turns it into a Mountain, stripping the very ability that generates this
/// effect (CR 305.7). That makes Urborg's effect dependent on Blood Moon under
/// CR 613.8a(b), so Blood Moon wins regardless of timestamps. Modeling it as an
/// enchantment keeps that dependency out of the 305.6 tests, which are about the
/// additive clause rather than about effect ordering. Do not use this card to
/// reason about the real Blood Moon / Urborg interaction — that is
/// `printings::urborg_tomb_of_yawgmoth`, the printed Land, registered
/// since LI-2 with its ruling asserted in `tests/phase_li2_integration_test.rs`.
pub fn urborg_effect() -> Arc<CardData> {
    CardDataBuilder::new("Urborg Effect")
        .mana_cost(ManaCost::build(&[ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .ability(static_ability(
            "Each land is a Swamp in addition to its other land types.",
            becomes_types(each(land()), TypeChange::adding(&[], &[Subtype::Land(LandType::Swamp)], &[])),
        ))
        .build()
}

/// Underground Sea (simplified) — nonbasic dual land
/// Land — Island Swamp
/// {T}: Add {U}.
/// {T}: Add {B}.
///
/// The board setup for ATOM-305.7-002 and COMP-305.7+305.6-001. The atoms
/// specify a single "{T}: Add {U} or {B}" — modal mana abilities aren't
/// supported yet, so it's modeled as two separate mana abilities. CR 305.7
/// strips both either way, so the distinction doesn't affect what's under test.
///
/// Deliberately has NO Basic supertype, so Blood Moon's "nonbasic lands" filter
/// matches it.
pub fn dual_land_ub() -> Arc<CardData> {
    CardDataBuilder::new("Underground Sea")
        .card_type(CardType::Land)
        .subtype(Subtype::Land(LandType::Island))
        .subtype(Subtype::Land(LandType::Swamp))
        .rules_text("{T}: Add {U}. {T}: Add {B}.")
        .mana_ability_single(ManaType::Blue)
        .mana_ability_single(ManaType::Black)
        .build()
}

/// Windswept Heights (invented) — {1}{W}
/// Enchantment
/// Lands have flying.
///
/// Not "lands you control": the filter is a bare `ByType(Land)`. Same as
/// [`land_creatures_have_flying`] — the text matches the filter.
///
/// Nonsense as a Magic card, but it is the only ability-granting channel that
/// exists today: `Primitive::GrantKeywordFlag` registers a real Layer 6 effect
/// through `register_static_effects`. Used by ATOM-305.7-003 to show that an
/// ability granted by another effect survives Blood Moon — CR 305.7's "this
/// doesn't remove any abilities that were granted to the land by other effects".
pub fn lands_have_flying() -> Arc<CardData> {
    CardDataBuilder::new("Windswept Heights")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("Lands have flying.")
        .ability(static_ability(
            "Lands have flying.",
            gains_keyword(each(land()), KeywordFlag::Flying),
        ))
        .build()
}

/// Cloudspire Mesa (invented) — nonbasic land
/// Land
/// Creatures have flying.
///
/// Not "creatures you control" — the filter is a bare `ByType(Creature)` with
/// no controller constraint, and the text says what the filter does. Adding
/// `ByController(PlayerRef::You)` would work (`compute::object_matches_filter`
/// resolves it), but controller scoping is not what this card is here to
/// exercise.
///
/// The Land-typed sibling of [`lands_have_flying`], and it exists because Blood
/// Moon's filter is "nonbasic **land**". Testing that a stripped static ability
/// retires the continuous effect it generated needs a permanent that (a) Blood
/// Moon can strip — so, a land without the `Basic` supertype — and (b) has a
/// static ability generating a registered effect. Nothing in the card pool is
/// both, which is why item 7 of Deferred Migrations went unnoticed: the bug is
/// live, but no card reaches it.
///
/// Nonsense as a Magic card, in the same way and for the same reason as
/// `lands_have_flying`. `Primitive::GrantKeywordFlag` is used because it is the one
/// Layer 6 channel wired end to end today.
///
/// Deliberately has NO `Basic` supertype, so Blood Moon's filter matches it.
pub fn land_creatures_have_flying() -> Arc<CardData> {
    CardDataBuilder::new("Cloudspire Mesa")
        .card_type(CardType::Land)
        .rules_text("Creatures have flying.")
        .ability(static_ability(
            "Creatures have flying.",
            gains_keyword(each(creature()), KeywordFlag::Flying),
        ))
        .build()
}

/// Moonlit Steppe (invented) — nonbasic land
/// Land
/// Nonbasic lands are Mountains.
///
/// Blood Moon's ability, printed on a land that its own effect then matches. A
/// deliberately compressed stand-in for a board the CR permits and real cards
/// reach: Opalescence makes Blood Moon a creature, Ashaya, Soul of the Wild
/// makes each nontoken creature you control a Forest land, and Blood Moon —
/// now a nonbasic land — strips its own ability under CR 305.7.
///
/// Compressed because our `ObjectFilter` has no way to say "each **other**
/// non-Aura enchantment", so a modeled Opalescence would also enchant itself
/// and put three layers of noise between the fixture and what it is testing.
/// One card reaches the same self-reference.
///
/// What it is for: proving the CR 604.2 existence check terminates. The check
/// asks whether this land still has its static ability, which is a
/// characteristics query on the land, which runs the check again. It resolves
/// because the question is asked at a strictly lower layer ceiling each time
/// (`layers-architecture.md` §5.2) — at Layer 4 it reads the frame as of the
/// end of Layer 3, where the strip has not run. Get that wrong and this card
/// overflows the stack.
pub fn self_stripping_land() -> Arc<CardData> {
    let mountain_set = Subtypes::from([Subtype::Land(LandType::Mountain)]);

    let nonbasic_land_filter = land().and(non(basic()));

    CardDataBuilder::new("Moonlit Steppe")
        .card_type(CardType::Land)
        .rules_text("Nonbasic lands are Mountains.")
        .ability(static_ability(
            "Nonbasic lands are Mountains.",
            becomes_types(each(nonbasic_land_filter), TypeChange { set_subtypes: Some(mountain_set), ..TypeChange::NONE }),
        ))
        .build()
}

/// Purifier Clause — {3}{G}
/// Creature — Elf Druid, 3/4
/// Lands you control are basic.
///
/// **A fixture, and an invented name**
/// (`engineering-practices.md#cards-and-pools`'s rule): the battlefield half of
/// Rootpath Purifier's "Lands you control and land cards in your library are
/// basic", without the library clause, which waits on `codebase-state.md`
/// layers item 9. Registering the printed card without that clause would wear
/// its name while behaving differently, so this one wears its own and is
/// registered in no pool.
///
/// It is the board of the Purifier's 2022-10-14 ruling: "Lands that become
/// basic are no longer nonbasic lands. This may change what effects can
/// apply to them. For example, if an opponent controls Blood Moon, an
/// enchantment which says 'Nonbasic lands are Mountains,' and you play
/// Rootpath Purifier, Blood Moon can no longer apply to the lands you
/// control because they are all basic." — the applies-to dependency with a
/// ruling behind it, in both timestamp orders.
pub fn purifier_clause() -> Arc<CardData> {
    CardDataBuilder::new("Purifier Clause")
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Elf))
        .subtype(Subtype::Creature(CreatureType::Druid))
        .power_toughness(3, 4)
        .rules_text("Lands you control are basic.")
        .ability(static_ability(
            "Lands you control are basic.",
            becomes_types(
                each(land().and(you_control())),
                TypeChange::adding(&[], &[], &[Supertype::Basic]),
            ),
        ))
        .build()
}

/// Flight Clause — {1}{U}
/// Enchantment — Aura
/// Enchant permanent
/// As long as enchanted permanent is a creature, it has flying.
///
/// **A fixture, and an invented name**
/// (`engineering-practices.md#cards-and-pools`'s rule): the third line of Rune
/// of Flight, whose printed text is "Enchant permanent / When this Aura enters,
/// draw a card. / As long as enchanted permanent is a creature, it has flying.
/// / As long as enchanted permanent is an Equipment, it has 'Equipped creature
/// has flying.'" (Scryfall, 2026-09-06). Two of those four lines are out of
/// reach — the draw is a trigger (critical-path item 6), and the Equipment
/// clause grants a *static* ability from a static ability, which registers no
/// continuous effect (`codebase-state.md` "Before Layers" item 7g). Registering
/// the printed card with either missing would wear its name while behaving
/// differently, so this one wears its own.
///
/// It is CR 613.7a's own worked example one line short, and three phases
/// meet on it: the layer-6 grant over `Host` (LH-1), an Aura whose host can
/// be reattached (LH-2), and the condition (LI-3). Against Humility in either
/// timestamp order it is the one board where all three are observable at
/// once — Humility strips at layer 6 and this grants at layer 6, so the
/// timestamps decide, and the condition decides whether there is anything to
/// order at all.
pub fn flight_clause() -> Arc<CardData> {
    CardDataBuilder::new("Flight Clause")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .rules_text(
            "Enchant permanent\nAs long as enchanted permanent is a creature, it has flying.",
        )
        // CR 702.5a — "Enchant permanent" is this Aura's targeting
        // restriction, and CR 303.4a makes it the spell's target. Permanent,
        // not creature: the condition is what decides whether the grant does
        // anything, and an Aura that could only enchant creatures could not
        // have a condition worth reading.
        .enchant_filter(SelectionFilter::Permanent(ObjectFilter::All))
        .ability(static_ability(
            "As long as enchanted permanent is a creature, it has flying.",
            as_long_as(Condition::HostMatches(creature()), gains_keyword(host(), KeywordFlag::Flying)),
        ))
        .build()
}

/// Simian Clause — {1}{G}
/// Enchantment
/// As long as you control a Forest, each creature you control is an Ape in
/// addition to its other types.
///
/// **A fixture, and an invented name**
/// (`engineering-practices.md#cards-and-pools`'s rule), for a shape no printed
/// card in reach has: a *layer-4* effect whose condition another layer-4 effect
/// flips. Blood Moon sets a Taiga to Mountain, the Forest goes away, and this
/// effect stops existing — so it depends on Blood Moon (CR 613.8a(b), the
/// existence clause) and waits for it whatever the timestamps say.
///
/// **It is the board `board::condition_reads` exists for.** Without the
/// condition's channels the pair is settled independent by the static check
/// and never reaches the hypothetical: this card's own reads would be its
/// filter's (types, controller), Blood Moon writes subtypes and abilities,
/// and the one read they share — the ability list, for CR 604.2 — is of
/// *this* card's source, which Blood Moon does not reach. The condition is
/// what reads a subtype off another object.
///
/// Kird Ape's board is the same shape two layers apart and needs none of
/// that, which is why this one is not the Ape.
pub fn simian_clause() -> Arc<CardData> {
    CardDataBuilder::new("Simian Clause")
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(
            "As long as you control a Forest, each creature you control is an Ape in addition \
             to its other types.",
        )
        .ability(static_ability(
            "As long as you control a Forest, each creature you control is an Ape in addition to its other types.",
            as_long_as(
                Condition::Player {
                    players: PlayerSet::You,
                    fact: PlayerFact::ControlsPermanent(with_subtype(Subtype::Land(LandType::Forest))),
                },
                becomes_types(each(creatures_you_control()), TypeChange::adding(&[], &[Subtype::Creature(CreatureType::Ape)], &[])),
            ),
        ))
        .build()
}

// ---------------------------------------------------------------------------
// Two fixtures, for the question "what can *see* a zone-reaching effect?"
// ---------------------------------------------------------------------------

/// **Fixture.** "Cards in graveyards are red in addition to their other colors."
///
/// Painter's Servant's second clause narrowed to one zone — the real card is
/// "all cards that aren't on the battlefield, spells, and permanents are the
/// chosen color", which is `ZoneSet::ALL` and an as-enters color choice.
///
/// **It exists because Yixlid Jailer's effect is not observable yet, and that
/// is a fair thing to ask of a first consumer** (owner review, 2026-09-14).
/// The Jailer strips *abilities* from graveyard cards, and abilities on a card
/// in a graveyard are read today by exactly nothing: flashback, retrace and
/// Bridge from Below's trigger are all gated on CR 113.6, which is A5. So the
/// Jailer's own tests assert the mechanism through `get_effective_abilities`,
/// which is a direct read rather than a consequence.
///
/// A *color* in a graveyard is different: `PlayerFact::CardInGraveyard` reads
/// it, and since LJ folded `CardFilter` into `ObjectFilter` that condition can
/// ask `ByColor`. So this fixture closes the loop — a zone-reaching row
/// changes a graveyard card's characteristics, and a **rule** reads the
/// change and turns another permanent's ability on. Nothing in that chain is
/// an oracle query from a test.
pub fn graveyard_painter() -> Arc<CardData> {
    CardDataBuilder::new("Graveyard Painter")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Artifact)
        .rules_text("Cards in graveyards are red in addition to their other colors.")
        .ability(static_ability(
            "Cards in graveyards are red in addition to their other colors.",
            is_also_color(each_in(ObjectFilter::All, ZoneSet::GRAVEYARD), Color::Red),
        ))
        .build()
}

/// **Fixture.** "This creature gets +2/+2 as long as there's a red card in
/// your graveyard."
///
/// The reader in [`graveyard_painter`]'s loop, and Kird Ape's shape with
/// `PlayerFact::CardInGraveyard` in place of `PlayerFact::ControlsPermanent`. The condition
/// is evaluated by `engine::layers::condition` against the *live* board at the
/// row's layer, so the color it asks about is the post-Layer-5 color — which
/// is what makes the Painter visible to it.
///
/// `ByColor` is the leaf that makes this fixture possible at all: before LJ
/// folded the two filter types together, the graveyard leaf took a `CardFilter`
/// whose three variants could ask about a type or a color but never about an
/// owner or a controller. The color half is what this needs.
pub fn graveyard_reveler() -> Arc<CardData> {
    CardDataBuilder::new("Graveyard Reveler")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Zombie))
        .power_toughness(1, 1)
        .rules_text("This creature gets +2/+2 as long as there's a red card in your graveyard.")
        .ability(static_ability(
            "This creature gets +2/+2 as long as there's a red card in your graveyard.",
            as_long_as(
                Condition::Player { players: PlayerSet::You, fact: PlayerFact::CardInGraveyard(colored(Color::Red)) },
                gets(this_object(), 2, 2),
            ),
        ))
        .build()
}

// ---------------------------------------------------------------------------
// Two fixtures, for the question "what does a row reaching the hidden zones
// cost?" (`codebase-state.md` item 181, `tests/zone_reach_cost_test.rs`)
// ---------------------------------------------------------------------------

/// **Fixture.** "Creature cards you own that aren't on the battlefield have
/// flash." Teferi, Mage of Zhalfir's second line, alone.
///
/// The narrow shape, and the common one: 11 of the 13 printed cards whose
/// text reaches "cards … that aren't on the battlefield" name only cards
/// their controller owns (Scryfall `o:"that aren't on the battlefield"`,
/// 2026-09-25). The row matches one seat's creature cards, and the working set
/// it brings is every card in every zone it names, every seat's.
///
/// **Nothing reads the grant yet.** The cast-timing check reads a card's
/// printed flash (`// PRE-LAYER ZONE:`), so a game with this fixture plays as
/// it does without it, and the difference is cost alone (item 182).
///
/// Registered nowhere: either of these two in the stress pool moves every
/// game there.
pub fn teferi_flash_clause() -> Arc<CardData> {
    CardDataBuilder::new("Teferi's Flash Clause")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Artifact)
        .rules_text("Creature cards you own that aren't on the battlefield have flash.")
        .ability(static_ability(
            "Creature cards you own that aren't on the battlefield have flash.",
            gains_keyword(
                each_in(
                    creature().and(ObjectFilter::ByOwner(PlayerRef::You)),
                    ZoneSet::EVERYWHERE_BUT_BATTLEFIELD,
                ),
                KeywordFlag::Flash,
            ),
        ))
        .build()
}

/// **Fixture.** "All cards that aren't on the battlefield are colorless."
/// Mycosynth Lattice's second line without its "and permanents", so the
/// battlefield plays as it does without the fixture.
///
/// The wide shape: every card in every zone but the battlefield, every
/// seat's, which is also the stack's spells and abilities. Painter's Servant's
/// second line reaches the same objects.
pub fn lattice_colorless_clause() -> Arc<CardData> {
    CardDataBuilder::new("Lattice's Colorless Clause")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Artifact)
        .rules_text("All cards that aren't on the battlefield are colorless.")
        .ability(static_ability(
            "All cards that aren't on the battlefield are colorless.",
            becomes_colorless(each_in(ObjectFilter::All, ZoneSet::EVERYWHERE_BUT_BATTLEFIELD)),
        ))
        .build()
}

/// **Fixture.** "As long as this card isn't on the battlefield, creatures you
/// control get +1/+1."
///
/// CR 113.6c — "an ability that states which zones it *doesn't* function in
/// functions everywhere except for the specified zones" — and the point of the
/// fixture is that it needs no new arm: it is Wonder's clause holding
/// `ZoneSet::EVERYWHERE_BUT_BATTLEFIELD`, the complement LJ already spelled as
/// a constant for Grist and Mycosynth Lattice.
///
/// # This text is a shape no real card has, and the reason is information
///
/// Raised at the LK review, and it is a sharper observation than "the printed
/// population is thin". Every printed card in this family affects **only
/// itself** — Grist, the Hunger Tide is "as long as Grist isn't on the
/// battlefield, **it's** a 1/1 Insect creature" (Scryfall, verified
/// 2026-09-14) — and that is not a coincidence about design taste. A card in
/// a **hidden** zone is `ZoneSet::EVERYWHERE_BUT_BATTLEFIELD`'s problem: an
/// anthem functioning from a library or a hand would change the board while
/// nobody at the table could see why, and there is no good way to tell them.
/// Self-affecting clauses have no such problem, because the object whose
/// characteristics changed is the object nobody can see either.
///
/// **So the fixture deliberately overstates what a card may say**, and keeps
/// doing it: what it is testing is that the *predicate* reads a complement off
/// the same field as a plain zone, and an anthem is what makes that observable
/// from outside the card (`get_effective_power` on another permanent) rather
/// than by querying the fixture's own frame. A self-scoped version would test
/// the CDA path instead, which is `layers::cda`'s and not this predicate's.
///
/// The constraint the card list implies is real and already has an owner:
/// §13c decision 4 filed "a zone-reaching row over a hidden zone must not make
/// that zone's order or contents observable" against `backlog.md` §2.9, the
/// information model. This fixture is a source in a *graveyard*, which CR 400.2
/// makes public, so it does not reach that constraint — but a card genuinely
/// printing this text would, and §2.9 is where it gets answered.
pub fn exiled_ancestor() -> Arc<CardData> {
    CardDataBuilder::new("Exiled Ancestor")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Spirit))
        .power_toughness(1, 1)
        .rules_text("As long as this card isn't on the battlefield, creatures you control get +1/+1.")
        .ability(functioning_in(
            ZoneSet::EVERYWHERE_BUT_BATTLEFIELD,
            static_ability("As long as this card isn't on the battlefield, creatures you control get +1/+1.", gets(each(creatures_you_control()), 1, 1)),
        ))
        .build()
}

/// Grist, the Hunger Tide — {1}{B}{G}
/// Legendary Planeswalker — Grist, loyalty 3
/// "As long as Grist isn't on the battlefield, it's a 1/1 Insect creature in
/// addition to its other types."
///
/// **Its first ability on its printed frame, and none of its three loyalty
/// abilities**, which wait on `backlog.md` §2.11 (no loyalty ability exists)
/// and, for the −2's reflexive trigger, TR-3. The only printed card whose
/// static ability changes its own card everywhere but the battlefield
/// (Scryfall, two phrasings, 2026-09-25).
///
/// Its rulings are the tests: "Anywhere but on the battlefield, Grist is a
/// Legendary Planeswalker Creature — Grist Insect", so on the stack it is a
/// creature spell (Essence Scatter can counter it, and Thalia, Guardian of
/// Thraben does not tax it), and once it enters it is a planeswalker and
/// nothing else. The clause is a `SourceOnly` row, which reaches Grist only
/// because Grist is a member of every pass wherever it is
/// (`layers::board::source_joins`).
pub fn grist_insect_clause() -> Arc<CardData> {
    CardDataBuilder::new("Grist's Insect Clause")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Green], 1))
        .color(Color::Black)
        .color(Color::Green)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Planeswalker)
        .subtype(Subtype::Planeswalker(PlaneswalkerType::Grist))
        .loyalty(3)
        .rules_text("As long as Grist isn't on the battlefield, it's a 1/1 Insect creature in addition to its other types.")
        .ability(functioning_in(
            // CR 113.6c — the ability says where it does not function.
            ZoneSet::EVERYWHERE_BUT_BATTLEFIELD,
            static_ability(
                "As long as Grist isn't on the battlefield, it's a 1/1 Insect creature in addition to its other types.",
                becomes_types(this_object(), TypeChange::adding(&[CardType::Creature], &[Subtype::Creature(CreatureType::Insect)], &[]))
                    .and_has_base_power_and_toughness(1, 1),
            ),
        ))
        .build()
}

/// Titania's Song — {3}{G}, Enchantment
/// "Each noncreature artifact loses all abilities and becomes an artifact
/// creature with power and toughness each equal to its mana value."
///
/// **The first sentence.** The second, "If this enchantment leaves the
/// battlefield, this effect continues until end of turn", extends a duration
/// past its source, which `Duration` cannot say.
///
/// **The test board for noting each row at its own layer** (§13e decision 1),
/// in the shape every layer-6 "loses all abilities" has: Humility on
/// Painter's Servant is the same board.
/// Beside Mycosynth Lattice's colorless line the artifact loses the ability
/// at layer 6, but the line applies at layer 5, while the ability is still
/// there — so a card in a library is colorless. A walk that asked the memo's
/// settled Lattice would find no ability and skip the row. One ability, three
/// rows, one effect: CR 613.6 locks the noncreature artifacts at layer 4 and
/// applies the strip and the P/T to the same set.
pub fn titanias_song_clause() -> Arc<CardData> {
    let noncreature_artifact = each(artifact().and(non(creature())));
    CardDataBuilder::new("Titania's Song's First Sentence")
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text("Each noncreature artifact loses all abilities and becomes an artifact creature with power and toughness each equal to its mana value.")
        .ability(static_ability(
            "Each noncreature artifact loses all abilities and becomes an artifact creature with power and toughness each equal to its mana value.",
            becomes_types(noncreature_artifact, TypeChange::adding(&[CardType::Creature], &[], &[]))
                .and_loses_all_abilities()
                .and_has_base_power_and_toughness(AmountExpr::AffectedManaValue, AmountExpr::AffectedManaValue),
        ))
        .build()
}

/// **Fixture.** "As long as this card is in your hand, creatures you control
/// have flying." — Wonder's clause, one zone over, and a text no card prints
/// (`fixtures::exiled_ancestor` says why none does).
///
/// **A hidden static source** (§13e decision 2). Beside Hollow Hands the
/// grant and the strip both apply at layer 6, and the grant waits on the strip
/// (CR 613.8a) only if the pass sees the strip reach the card, so the card
/// joins the pass. Walked alone it would be read as of the end of layer 5,
/// still granting.
pub fn pocket_griffin() -> Arc<CardData> {
    CardDataBuilder::new("Pocket Griffin")
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Griffin))
        .power_toughness(2, 2)
        .rules_text("As long as this card is in your hand, creatures you control have flying.")
        .ability(functioning_in(
            ZoneSet::HAND,
            static_ability("As long as this card is in your hand, creatures you control have flying.", gains_keyword(each(creatures_you_control()), KeywordFlag::Flying)),
        ))
        .build()
}

/// Arcane Adaptation — {2}{U}, Enchantment
/// "As this enchantment enters, choose a creature type. Creatures you control
/// are the chosen type in addition to their other types. The same is true for
/// creature spells you control and creature cards you own that aren't on the
/// battlefield."
///
/// **The last clause, with Elf chosen**: the half that reaches a library and a
/// hand. Beside Grist in a hand it is the printed case of a CR 613.8
/// dependency decided through a hidden card — Grist's own effect makes it a
/// creature card, so this one depends on it — and Grist joins the pass as its
/// row's source, which is how the pass sees it (§13e).
pub fn arcane_adaptation_elf_clause() -> Arc<CardData> {
    CardDataBuilder::new("Arcane Adaptation's Elf Clause")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text("Creature cards you own that aren't on the battlefield are Elves in addition to their other types.")
        .ability(static_ability(
            "Creature cards you own that aren't on the battlefield are Elves in addition to their other types.",
            becomes_types(
                each_in(
                    creature().and(ObjectFilter::ByOwner(PlayerRef::You)),
                    ZoneSet::EVERYWHERE_BUT_BATTLEFIELD,
                ),
                TypeChange::adding(&[], &[Subtype::Creature(CreatureType::Elf)], &[]),
            ),
        ))
        .build()
}

/// **Fixture.** "Creature cards in libraries are artifacts in addition to
/// their other types." — Biotransference's shape, reaching libraries alone.
///
/// Half of the guard's pair (§13e decision 4, (b)); `library_assassins` is
/// the other.
pub fn library_artificer() -> Arc<CardData> {
    CardDataBuilder::new("Library Artificer")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text("Creature cards in libraries are artifacts in addition to their other types.")
        .ability(static_ability(
            "Creature cards in libraries are artifacts in addition to their other types.",
            becomes_types(each_in(creature(), ZoneSet::LIBRARY), TypeChange::adding(&[CardType::Artifact], &[], &[])),
        ))
        .build()
}

/// **Fixture.** "Artifact creature cards in libraries are Assassins in
/// addition to their other types."
///
/// **A dependency no printed card makes, decided through cards in a
/// library** (§13e decision 4, (b)). Beside `library_artificer` it depends on
/// the artificer's effect, which makes the creature cards artifacts. The
/// dependency is real only through a library card, so the guard keeps the
/// libraries in the pass: left out, the pass would see no dependency and
/// apply the older effect first.
pub fn library_assassins() -> Arc<CardData> {
    CardDataBuilder::new("Library Assassins")
        .mana_cost(ManaCost::build(&[ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("Artifact creature cards in libraries are Assassins in addition to their other types.")
        .ability(static_ability(
            "Artifact creature cards in libraries are Assassins in addition to their other types.",
            becomes_types(
                each_in(
                    artifact().and(creature()),
                    ZoneSet::LIBRARY,
                ),
                TypeChange::adding(&[], &[Subtype::Creature(CreatureType::Assassin)], &[]),
            ),
        ))
        .build()
}
