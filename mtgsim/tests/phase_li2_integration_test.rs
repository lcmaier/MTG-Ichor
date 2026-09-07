//! Phase LI-2 — CR 613.8a, 613.8b, 613.8c (`layers-architecture.md` §13b).
//!
//! What changed is the *order* a layer applies its effects in: an effect
//! waits for anything it depends on (CR 613.8a/b), dependencies being
//! decided against the board as it is being built and re-decided after
//! every application (CR 613.8c). The boards here are the rulings' — Urborg,
//! Rootpath Purifier, Humility + Opalescence — each asserted in both
//! timestamp orders with the ruling quoted beside it; one printed board with
//! no ruling, marked CR-derived; and three fixture boards for the clauses no
//! printed pair reaches. The order itself, step by step, is asserted in
//! `engine/layers/board.rs`'s unit tests through the pass's trace hook.

use mtgsim::cards::{basic_lands, creatures, dual_lands, phase_ld_cards, phase_lf_cards, phase_li_cards};
use mtgsim::engine::layers::types::{
    AffectedSet, ContinuousEffect, EffectModification, EffectOrigin, Layer, PtValue,
};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_power, get_effective_subtypes, get_effective_supertypes,
    get_effective_toughness, get_effective_types, is_creature,
};
use mtgsim::oracle::mana_helpers::available_mana_sources;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_on_battlefield, setup_two_player_game, static_ability, vanilla_creature};
use mtgsim::types::card_types::{CardType, CreatureType, LandType, Subtype, Supertype};
use mtgsim::types::effects::{
    CounterType, Duration, Effect, EffectRecipient, PermanentFilter, Primitive, TypeChange,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use std::collections::HashSet;
use std::sync::Arc;

fn pt(game: &GameState, id: ObjectId) -> (Option<i32>, Option<i32>) {
    (get_effective_power(game, id), get_effective_toughness(game, id))
}

fn land(game: &GameState, id: ObjectId, land_type: LandType) -> bool {
    get_effective_subtypes(game, id).contains(&Subtype::Land(land_type))
}

/// The mana `id` can produce for `player`, sorted.
fn taps_for(game: &GameState, player: PlayerId, id: ObjectId) -> Vec<ManaType> {
    let mut mana: Vec<ManaType> = available_mana_sources(game, player)
        .into_iter()
        .filter(|s| s.permanent_id == id)
        .map(|s| s.produces)
        .collect();
    mana.sort_by_key(|m| *m as u8);
    mana.dedup();
    mana
}

/// A layer-4 `ChangeType` static over `filter`, as a card.
fn type_changer(name: &str, card_type: CardType, change: TypeChange, filter: PermanentFilter) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(card_type)
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeType(change, Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(filter),
        )))
        .build()
}

fn no_change() -> TypeChange {
    TypeChange {
        add_types: Vec::new(),
        remove_types: Vec::new(),
        set_types: None,
        add_subtypes: Vec::new(),
        remove_subtypes: Vec::new(),
        set_subtypes: None,
        add_supertypes: Vec::new(),
        remove_supertypes: Vec::new(),
        set_supertypes: None,
    }
}

fn creature_of_type(name: &str, creature_type: CreatureType) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(creature_type))
        .power_toughness(1, 1)
        .build()
}

// ---------------------------------------------------------------------------
// Urborg, Tomb of Yawgmoth + Blood Moon — the existence dependency
// (CR 613.8a(b), "the existence of the first effect").
//
// Urborg's ruling (Scryfall, 2021-03-19): "If an effect such as that of Magus
// of the Moon causes Urborg to lose its abilities by setting it to a basic
// land type not in addition to its other types, it won't turn lands into
// Swamps, no matter in what order those effects started to apply."
// ---------------------------------------------------------------------------

/// Blood Moon and Urborg with a nonbasic land and a basic Forest, in both
/// timestamp orders. Urborg is a Mountain that is not a Swamp; the dual is a
/// Mountain that taps only for {R}; the Forest is a Forest and never a
/// Forest Swamp. Under timestamp order alone, Urborg entering first would
/// have painted Swamp onto everything before Blood Moon stripped it.
// COVERS: ATOM-613.8a-001
// COVERS-PARTIAL: ATOM-613.8a-002
#[test]
fn test_urborg_and_blood_moon_in_both_orders() {
    for urborg_first in [true, false] {
        let mut game = setup_two_player_game();
        let dual = put_on_battlefield(&mut game, dual_lands::tropical_island(), 0);
        let forest = put_on_battlefield(&mut game, basic_lands::forest(), 0);
        let (urborg, moon) = if urborg_first {
            let u = put_on_battlefield(&mut game, phase_li_cards::urborg_tomb_of_yawgmoth(), 0);
            let m = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 1);
            (u, m)
        } else {
            let m = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 1);
            let u = put_on_battlefield(&mut game, phase_li_cards::urborg_tomb_of_yawgmoth(), 0);
            (u, m)
        };
        let order = if urborg_first { "Urborg first" } else { "Blood Moon first" };

        assert!(land(&game, urborg, LandType::Mountain), "{order}: Urborg is a Mountain");
        assert!(!land(&game, urborg, LandType::Swamp), "{order}: Urborg is not a Swamp");
        assert!(get_effective_supertypes(&game, urborg).contains(&Supertype::Legendary), "{order}: still legendary");
        assert_eq!(taps_for(&game, 0, urborg), vec![ManaType::Red], "{order}: Urborg taps for {{R}} only");

        assert_eq!(
            get_effective_subtypes(&game, dual),
            HashSet::from([Subtype::Land(LandType::Mountain)]),
            "{order}: the dual is a Mountain and nothing else"
        );
        assert_eq!(taps_for(&game, 0, dual), vec![ManaType::Red], "{order}: the dual taps for {{R}} only");

        assert_eq!(
            get_effective_subtypes(&game, forest),
            HashSet::from([Subtype::Land(LandType::Forest)]),
            "{order}: a basic Forest is a Forest, never a Forest Swamp"
        );
        assert_eq!(taps_for(&game, 0, forest), vec![ManaType::Green]);
        assert!(!is_creature(&game, moon));
    }
}

/// Urborg on its own: a land, so a Swamp under its own effect, tapping for
/// {B} it has no printed ability for (CR 305.6 through 305.7's additive
/// clause); a Forest beside it is a Forest Swamp with both mana abilities.
#[test]
fn test_urborg_is_itself_a_swamp() {
    let mut game = setup_two_player_game();
    let forest = put_on_battlefield(&mut game, basic_lands::forest(), 0);
    let urborg = put_on_battlefield(&mut game, phase_li_cards::urborg_tomb_of_yawgmoth(), 0);

    assert_eq!(taps_for(&game, 0, urborg), vec![ManaType::Black]);
    assert!(land(&game, forest, LandType::Forest) && land(&game, forest, LandType::Swamp));
    assert_eq!(taps_for(&game, 0, forest), vec![ManaType::Black, ManaType::Green]);
    assert_eq!(get_effective_abilities(&game, urborg).len(), 2, "its printed static and the intrinsic {{B}}");
}

// ---------------------------------------------------------------------------
// The Rootpath Purifier ruling's board + Blood Moon — the applies-to
// dependency (CR 613.8a(b), "what it applies to"), with a ruling behind it.
//
// Rootpath Purifier's ruling (Scryfall, 2022-10-14): "Lands that become basic
// are no longer nonbasic lands. This may change what effects can apply to
// them. For example, if an opponent controls Blood Moon, an enchantment which
// says 'Nonbasic lands are Mountains,' and you play Rootpath Purifier, Blood
// Moon can no longer apply to the lands you control because they are all
// basic." And: "Being basic doesn't grant any abilities to a land that it
// didn't already have, and doesn't remove any card types, subtypes, or
// supertypes."
//
// The printed card waits on layers item 9 (its library clause);
// `phase_li_cards::purifier_clause` is its battlefield half under its own name.
// ---------------------------------------------------------------------------

#[test]
fn test_purifier_clause_and_blood_moon_in_both_orders() {
    for purifier_first in [true, false] {
        let mut game = setup_two_player_game();
        let mine = put_on_battlefield(&mut game, dual_lands::tropical_island(), 0);
        let theirs = put_on_battlefield(&mut game, dual_lands::taiga(), 1);
        if purifier_first {
            put_on_battlefield(&mut game, phase_li_cards::purifier_clause(), 0);
            put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 1);
        } else {
            put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 1);
            put_on_battlefield(&mut game, phase_li_cards::purifier_clause(), 0);
        }
        let order = if purifier_first { "Purifier first" } else { "Blood Moon first" };

        // "Blood Moon can no longer apply to the lands you control".
        assert!(get_effective_supertypes(&game, mine).contains(&Supertype::Basic), "{order}: basic");
        assert_eq!(
            get_effective_subtypes(&game, mine),
            HashSet::from([Subtype::Land(LandType::Forest), Subtype::Land(LandType::Island)]),
            "{order}: my dual keeps its land types"
        );
        assert_eq!(taps_for(&game, 0, mine), vec![ManaType::Blue, ManaType::Green], "{order}: and its abilities");

        // ...and still applies to the opponent's.
        assert_eq!(get_effective_subtypes(&game, theirs), HashSet::from([Subtype::Land(LandType::Mountain)]));
        assert_eq!(taps_for(&game, 1, theirs), vec![ManaType::Red], "{order}");
        assert!(!get_effective_supertypes(&game, theirs).contains(&Supertype::Basic));
    }
}

// ---------------------------------------------------------------------------
// Ashaya, Soul of the Wild + Blood Moon — the applies-to dependency on a
// printed card. **No ruling covers this pair**; the expected answer is
// derived from the CR and the derivation is the comment.
//
// CR 613.8a(b): applying Ashaya makes the nontoken creatures you control
// Forest lands — nonbasic lands (CR 305.8), which is what Blood Moon applies
// to — so Blood Moon depends on Ashaya. Ashaya does not depend back: Blood
// Moon reaches no creature until Ashaya has applied. So Ashaya applies first
// whatever the timestamps, and Blood Moon then sets every creature-land to
// Mountain; CR 305.7 strips their abilities, Ashaya's own two included, and
// Ashaya's printed `*/*` is 0/0 with its CDA gone. The 2020-09-25 ruling
// covers the half without Blood Moon: "it's affected by its second ability
// and thus its first ability counts itself."
// ---------------------------------------------------------------------------

#[test]
fn test_ashaya_and_blood_moon_in_both_orders_cr_derived() {
    for ashaya_first in [true, false] {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        let forest = put_on_battlefield(&mut game, basic_lands::forest(), 0);
        let ashaya = if ashaya_first {
            let a = put_on_battlefield(&mut game, phase_li_cards::ashaya_soul_of_the_wild(), 0);
            put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 1);
            a
        } else {
            put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 1);
            put_on_battlefield(&mut game, phase_li_cards::ashaya_soul_of_the_wild(), 0)
        };
        let order = if ashaya_first { "Ashaya first" } else { "Blood Moon first" };

        for (name, id) in [("Ashaya", ashaya), ("the Bears", bears)] {
            let types = get_effective_types(&game, id);
            assert!(types.contains(&CardType::Creature) && types.contains(&CardType::Land), "{order}: {name} is a creature land");
            assert!(land(&game, id, LandType::Mountain), "{order}: {name} is a Mountain");
            assert!(!land(&game, id, LandType::Forest), "{order}: {name} is no longer a Forest");
            assert_eq!(taps_for(&game, 0, id), vec![ManaType::Red], "{order}: {name} taps for {{R}} only");
            assert_eq!(get_effective_abilities(&game, id).len(), 1, "{order}: {name} has only the intrinsic ability");
        }
        assert_eq!(pt(&game, ashaya), (Some(0), Some(0)), "{order}: Ashaya's CDA is gone, `*/*` is 0/0");
        assert_eq!(pt(&game, bears), (Some(2), Some(2)));
        assert_eq!(taps_for(&game, 0, forest), vec![ManaType::Green], "{order}: a basic Forest is untouched");
    }
}

/// Ashaya without Blood Moon, per its 2020-09-25 rulings: creatures you
/// control are Forest lands with "{T}: Add {G}", and Ashaya's CDA — read at
/// layer 7a, after its own layer-4 effect — counts itself.
#[test]
fn test_ashaya_counts_itself_among_the_lands_it_makes() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    put_on_battlefield(&mut game, basic_lands::forest(), 0);
    let theirs = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);
    let ashaya = put_on_battlefield(&mut game, phase_li_cards::ashaya_soul_of_the_wild(), 0);

    // Forest, Bears, Ashaya: three lands you control.
    assert_eq!(pt(&game, ashaya), (Some(3), Some(3)));
    assert!(land(&game, bears, LandType::Forest) && get_effective_types(&game, bears).contains(&CardType::Land));
    assert_eq!(taps_for(&game, 0, bears), vec![ManaType::Green]);
    assert!(!get_effective_types(&game, theirs).contains(&CardType::Land), "not the opponent's");
}

// ---------------------------------------------------------------------------
// Humility + Opalescence — CR 613.6, with the rulings' answers. This is the
// test `codebase-state.md` "Before Layers" 7c waited on: every earlier
// construction put the strip in the same layer as the effect's first part,
// so a stable answer needed CR 613.8.
//
// Humility's ruling (Scryfall, 2009-10-01): "This is the current interaction
// between Humility and Opalescence: The type-changing effect applies at
// layer 4, but the rest happens in the applicable layers. The rest of it will
// apply even if the permanent loses its ability before it's finished
// applying. So if Opalescence, Humility, and Worship are on the battlefield
// and Opalescence entered before Humility, the following is true: Layer 4:
// Humility and Worship each become creatures that are still enchantments.
// (Opalescence). Layer 6: Humility and Worship each lose their abilities.
// (Humility) Layer 7b: Humility becomes 4/4 and Worship becomes 4/4.
// (Opalescence). Humility becomes 1/1 and Worship becomes 1/1 (Humility).
// But if Humility entered before Opalescence, the following is true: Layer 4:
// Humility and Worship each become creatures that are still enchantments
// (Opalescence). Layer 6: Humility and Worship each lose their abilities
// (Humility). Layer 7b: Humility becomes 1/1 and Worship becomes 1/1
// (Humility). Humility becomes 4/4 and Worship becomes 4/4 (Opalescence)."
//
// Worship is not registered; "Votive Idol" is a bare enchantment with its
// mana value, which is all the ruling reads of it.
// ---------------------------------------------------------------------------

fn votive_idol() -> Arc<CardData> {
    CardDataBuilder::new("Votive Idol")
        .card_type(CardType::Enchantment)
        .mana_cost(ManaCost::build(&[ManaType::White], 3))
        .build()
}

// COVERS: ATOM-613.6-003
#[test]
fn test_humility_and_opalescence_in_both_orders_per_the_2009_ruling() {
    // Opalescence entered before Humility: 4/4 (Opalescence), then 1/1 (Humility).
    let mut game = setup_two_player_game();
    let idol = put_on_battlefield(&mut game, votive_idol(), 0);
    let opalescence = put_on_battlefield(&mut game, phase_li_cards::opalescence(), 0);
    let humility = put_on_battlefield(&mut game, phase_lf_cards::humility(), 0);
    for (name, id) in [("Humility", humility), ("Worship", idol)] {
        assert!(is_creature(&game, id) && get_effective_types(&game, id).contains(&CardType::Enchantment), "{name}: layer 4");
        assert!(get_effective_abilities(&game, id).is_empty(), "{name}: layer 6");
        assert_eq!(pt(&game, id), (Some(1), Some(1)), "{name}: layer 7b, Humility's part last");
    }
    assert!(!is_creature(&game, opalescence), "Opalescence does not animate itself");

    // Humility entered before Opalescence: 1/1 (Humility), then 4/4 (Opalescence).
    // Humility loses its own ability in layer 6 — it is a creature by then —
    // and its 7b part still applies to the set it locked (CR 613.6).
    let mut game = setup_two_player_game();
    let idol = put_on_battlefield(&mut game, votive_idol(), 0);
    let humility = put_on_battlefield(&mut game, phase_lf_cards::humility(), 0);
    put_on_battlefield(&mut game, phase_li_cards::opalescence(), 0);
    for (name, id) in [("Humility", humility), ("Worship", idol)] {
        assert!(is_creature(&game, id), "{name}: layer 4");
        assert!(get_effective_abilities(&game, id).is_empty(), "{name}: layer 6");
        assert_eq!(pt(&game, id), (Some(4), Some(4)), "{name}: layer 7b, Opalescence's part last");
    }
}

/// Humility's ruling (Scryfall, 2006-02-01): "With a Humility and two
/// Opalescences on the battlefield, if Humility has the latest timestamp,
/// then all creatures are 1/1 with no abilities. If the timestamp order is
/// Opalescence, Humility, Opalescence, the second Opalescence is 1/1, and the
/// Humility and first Opalescence are 4/4. If Humility has the earliest
/// timestamp, then everything is 4/4."
///
/// Each Opalescence animates the other (its 2004-10-04 ruling: "Does not
/// animate itself. But can be animated by another Opalescence."), Humility
/// strips both in layer 6, and each one's 7b part still applies to the set
/// it locked in layer 4 — the ability being gone is what CR 613.6 is about.
#[test]
fn test_two_opalescences_and_humility_per_the_2006_ruling() {
    let all = |game: &GameState, ids: &[ObjectId], expected: i32| {
        for &id in ids {
            assert!(is_creature(game, id));
            assert!(get_effective_abilities(game, id).is_empty(), "everything lost its abilities");
            assert_eq!(pt(game, id), (Some(expected), Some(expected)));
        }
    };

    // Humility latest: all 1/1.
    let mut game = setup_two_player_game();
    let o1 = put_on_battlefield(&mut game, phase_li_cards::opalescence(), 0);
    let o2 = put_on_battlefield(&mut game, phase_li_cards::opalescence(), 0);
    let h = put_on_battlefield(&mut game, phase_lf_cards::humility(), 0);
    all(&game, &[o1, o2, h], 1);

    // Opalescence, Humility, Opalescence: the second Opalescence is 1/1, the
    // Humility and first Opalescence are 4/4.
    let mut game = setup_two_player_game();
    let o1 = put_on_battlefield(&mut game, phase_li_cards::opalescence(), 0);
    let h = put_on_battlefield(&mut game, phase_lf_cards::humility(), 0);
    let o2 = put_on_battlefield(&mut game, phase_li_cards::opalescence(), 0);
    all(&game, &[o1, h], 4);
    all(&game, &[o2], 1);

    // Humility earliest: everything 4/4.
    let mut game = setup_two_player_game();
    let h = put_on_battlefield(&mut game, phase_lf_cards::humility(), 0);
    let o1 = put_on_battlefield(&mut game, phase_li_cards::opalescence(), 0);
    let o2 = put_on_battlefield(&mut game, phase_li_cards::opalescence(), 0);
    all(&game, &[o1, o2, h], 4);
}

/// `PermanentFilter::EachOther` and the Aura exclusion: Blood Moon becomes a
/// 3/3 enchantment creature, an Aura does not, Opalescence itself does not.
#[test]
fn test_opalescence_animates_each_other_non_aura_enchantment() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let aura = put_on_battlefield(&mut game, mtgsim::cards::phase_lh_cards::holy_strength(), 0);
    assert!(game.attach(aura, bears));
    let moon = put_on_battlefield(&mut game, phase_ld_cards::blood_moon(), 0);
    let opalescence = put_on_battlefield(&mut game, phase_li_cards::opalescence(), 0);

    assert!(is_creature(&game, moon));
    assert_eq!(pt(&game, moon), (Some(3), Some(3)), "{{2}}{{R}} is mana value 3");
    assert!(get_effective_types(&game, moon).contains(&CardType::Enchantment), "in addition to its other types");
    assert!(!is_creature(&game, aura), "non-Aura");
    assert!(!is_creature(&game, opalescence), "each *other*");
    assert_eq!(pt(&game, bears), (Some(3), Some(4)), "the Aura still does its own job");
}

// ---------------------------------------------------------------------------
// CR 613.8b — a dependency loop is applied in timestamp order.
//
// "Elves are Goblins" against "Goblins are Elves", each a `SetSubtypes`, so
// each changes what the other applies to: a loop. The two orders give
// different boards, which is what makes the rule observable.
// ---------------------------------------------------------------------------

fn elves_are_goblins() -> Arc<CardData> {
    type_changer(
        "Elves Are Goblins",
        CardType::Enchantment,
        TypeChange { set_subtypes: Some(HashSet::from([Subtype::Creature(CreatureType::Goblin)])), ..no_change() },
        PermanentFilter::BySubtype(Subtype::Creature(CreatureType::Elf)),
    )
}

fn goblins_are_elves() -> Arc<CardData> {
    type_changer(
        "Goblins Are Elves",
        CardType::Enchantment,
        TypeChange { set_subtypes: Some(HashSet::from([Subtype::Creature(CreatureType::Elf)])), ..no_change() },
        PermanentFilter::BySubtype(Subtype::Creature(CreatureType::Goblin)),
    )
}

// COVERS: ATOM-613.8b-001
#[test]
fn test_a_dependency_loop_applies_in_timestamp_order() {
    let elf = |game: &GameState, id| get_effective_subtypes(game, id) == HashSet::from([Subtype::Creature(CreatureType::Elf)]);
    let goblin = |game: &GameState, id| get_effective_subtypes(game, id) == HashSet::from([Subtype::Creature(CreatureType::Goblin)]);

    // "Elves are Goblins" first: the Elf becomes a Goblin, then every Goblin
    // — both of them — becomes an Elf.
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, creature_of_type("An Elf", CreatureType::Elf), 0);
    let b = put_on_battlefield(&mut game, creature_of_type("A Goblin", CreatureType::Goblin), 0);
    put_on_battlefield(&mut game, elves_are_goblins(), 0);
    put_on_battlefield(&mut game, goblins_are_elves(), 0);
    assert!(elf(&game, a) && elf(&game, b), "both Elves");

    // "Goblins are Elves" first: the Goblin becomes an Elf, then every Elf
    // becomes a Goblin.
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, creature_of_type("An Elf", CreatureType::Elf), 0);
    let b = put_on_battlefield(&mut game, creature_of_type("A Goblin", CreatureType::Goblin), 0);
    put_on_battlefield(&mut game, goblins_are_elves(), 0);
    put_on_battlefield(&mut game, elves_are_goblins(), 0);
    assert!(goblin(&game, a) && goblin(&game, b), "both Goblins");
}

// ---------------------------------------------------------------------------
// CR 613.8c — dependencies are re-evaluated after each application.
//
// A: "Artifacts are Elves in addition"; B: "Elves are Goblins in addition";
// C: "Goblins are creatures", over one noncreature artifact. Before anything
// applies, B has nothing to apply to, so C does not depend on it; after A
// applies, it does. With C's timestamp *between* A's and B's, an order fixed
// once at the start of the layer would apply C before B and the artifact
// would never become a creature.
// ---------------------------------------------------------------------------

fn artifacts_are_elves() -> Arc<CardData> {
    type_changer(
        "Artifacts Are Elves",
        CardType::Enchantment,
        TypeChange { add_subtypes: vec![Subtype::Creature(CreatureType::Elf)], ..no_change() },
        PermanentFilter::ByType(CardType::Artifact),
    )
}

fn elves_are_also_goblins() -> Arc<CardData> {
    type_changer(
        "Elves Are Also Goblins",
        CardType::Enchantment,
        TypeChange { add_subtypes: vec![Subtype::Creature(CreatureType::Goblin)], ..no_change() },
        PermanentFilter::BySubtype(Subtype::Creature(CreatureType::Elf)),
    )
}

fn goblins_are_creatures() -> Arc<CardData> {
    type_changer(
        "Goblins Are Creatures",
        CardType::Enchantment,
        TypeChange { add_types: vec![CardType::Creature], ..no_change() },
        PermanentFilter::BySubtype(Subtype::Creature(CreatureType::Goblin)),
    )
}

// COVERS: ATOM-613.8c-001
#[test]
fn test_dependencies_are_re_evaluated_after_each_application() {
    // The atom's timestamps, A < B < C, where timestamp order happens to agree.
    let mut game = setup_two_player_game();
    let idol = put_on_battlefield(&mut game, CardDataBuilder::new("Idol").card_type(CardType::Artifact).build(), 0);
    put_on_battlefield(&mut game, artifacts_are_elves(), 0);
    put_on_battlefield(&mut game, elves_are_also_goblins(), 0);
    put_on_battlefield(&mut game, goblins_are_creatures(), 0);
    assert!(is_creature(&game, idol), "A, then B, then C");

    // C's timestamp between A's and B's: only re-evaluation puts B before C.
    let mut game = setup_two_player_game();
    let idol = put_on_battlefield(&mut game, CardDataBuilder::new("Idol").card_type(CardType::Artifact).build(), 0);
    put_on_battlefield(&mut game, artifacts_are_elves(), 0);
    put_on_battlefield(&mut game, goblins_are_creatures(), 0);
    put_on_battlefield(&mut game, elves_are_also_goblins(), 0);
    assert!(is_creature(&game, idol), "A, then B (C now depends on it), then C");
    assert_eq!(
        get_effective_subtypes(&game, idol),
        HashSet::from([Subtype::Creature(CreatureType::Elf), Subtype::Creature(CreatureType::Goblin)])
    );
}

// ---------------------------------------------------------------------------
// CR 613.8a(c) — a CDA and a non-CDA in one layer are independent, and the
// CDA applies first (CR 613.3) whatever its object's timestamp.
//
// The atom's board mixes a layer 7a CDA with a layer 7c pump, which are two
// layers and so independent under clause (a) before (c) is reached; this is
// the same-layer pair — a layer 4 subtype CDA beside a layer 4 row that reads
// the subtype — where clause (c) is what settles it. Partial for that reason.
// The clause's other half, two CDAs depending on each other, has no card: no
// tournament-legal card reaches it (owner's search, 2026-09-06, and
// `cda.rs`), so there is no answer to pin and this test does not try.
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-613.8a-003
#[test]
fn test_a_cda_and_a_non_cda_in_one_layer_are_independent() {
    // "This creature is an Elf" as a CDA (CR 604.3a: subtypes qualify).
    let mut is_an_elf = static_ability(Effect::Atom(
        Primitive::ChangeType(
            TypeChange { set_subtypes: Some(HashSet::from([Subtype::Creature(CreatureType::Elf)])), ..no_change() },
            Duration::WhileSourceOnBattlefield,
        ),
        EffectRecipient::Implicit,
    ));
    is_an_elf.is_characteristic_defining = true;
    let changeling_ish = CardDataBuilder::new("Elf By Definition")
        .card_type(CardType::Creature)
        .power_toughness(1, 1)
        .ability(is_an_elf)
        .build();

    let mut game = setup_two_player_game();
    // The row has the earlier timestamp; the CDA still applies first.
    put_on_battlefield(&mut game, elves_are_also_goblins(), 0);
    let creature = put_on_battlefield(&mut game, changeling_ish, 0);

    let checks = game.counters.dependency_checks();
    assert_eq!(
        get_effective_subtypes(&game, creature),
        HashSet::from([Subtype::Creature(CreatureType::Elf), Subtype::Creature(CreatureType::Goblin)]),
        "the CDA made it an Elf before the row asked which creatures are Elves"
    );
    assert_eq!(game.counters.dependency_checks(), checks, "clause (c) settled the pair with no hypothetical");
}

// ---------------------------------------------------------------------------
// The order LI-1 left unpinned: a power-reading row older than a +1/+1
// counter. Timestamp order applies the row first (a 2/2 has power 2 or less:
// 3/3) and then the counter (4/4). Under CR 613.8 the row depends on the
// counter — applying it changes what the row applies to — so the counter
// applies first and the row finds a 3-power creature: 3/3, the same answer
// as `test_a_counter_older_than_a_power_reading_row_applies_first`.
// ---------------------------------------------------------------------------

#[test]
fn test_a_power_reading_row_older_than_a_counter_waits_for_the_counter() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    // "Creatures with power 2 or less get +1/+1", from a resolution, first.
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(ContinuousEffect {
        id: 0,
        source: bears,
        origin: EffectOrigin::Resolution,
        layer: Layer::Layer7cModifyPT,
        duration: Duration::UntilEndOfTurn,
        controller: 0,
        created_on_turn: 1,
        timestamp,
        affected: AffectedSet::Filter {
            filter: PermanentFilter::And(
                Box::new(PermanentFilter::ByType(CardType::Creature)),
                Box::new(PermanentFilter::PowerLE(2)),
            ),
        },
        modification: EffectModification::ModifyPowerToughness {
            power: PtValue::Fixed(1),
            toughness: PtValue::Fixed(1),
        },
    });
    assert_eq!(pt(&game, bears), (Some(3), Some(3)));

    game.add_counters(bears, CounterType::PlusOnePlusOne, 1);
    assert_eq!(
        pt(&game, bears),
        (Some(3), Some(3)),
        "the row waited for the counter it depends on, then found a 3-power creature"
    );
}

// ---------------------------------------------------------------------------
// The four-card board from the judge answer
// (`plans/references/blood-moon-urborg-ashaya-opalescence-judge-answer.md`):
// Opalescence, Ashaya, Blood Moon and Urborg, all in layer 4, under one
// player. "Opalescence makes Blood Moon a creature ... Ashaya adds Forest
// Land to herself, Blood Moon and any other nontoken creatures ... Blood Moon
// changes all nonbasic lands to type Mountain removing existing abilities and
// adding the ability to tap for red mana, affecting Ashaya and Urborg and any
// other nontoken creatures. Urborg no longer has an effect so we're done in
// layer 4." The sequence itself is asserted in `board.rs`'s unit tests; this
// is the board the sequence leaves, in the answer's entry order and its
// reverse.
// ---------------------------------------------------------------------------

#[test]
fn test_urborg_never_applies_beside_blood_moon_ashaya_and_opalescence() {
    for reversed in [false, true] {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
        let forest = put_on_battlefield(&mut game, basic_lands::forest(), 0);
        let cards: [fn() -> Arc<CardData>; 4] = [
            phase_li_cards::opalescence,
            phase_li_cards::ashaya_soul_of_the_wild,
            phase_ld_cards::blood_moon,
            phase_li_cards::urborg_tomb_of_yawgmoth,
        ];
        let mut ids: Vec<ObjectId> = Vec::new();
        if reversed {
            for card in cards.iter().rev() {
                ids.push(put_on_battlefield(&mut game, card(), 0));
            }
            ids.reverse();
        } else {
            for card in cards.iter() {
                ids.push(put_on_battlefield(&mut game, card(), 0));
            }
        }
        let [opalescence, ashaya, moon, urborg] = [ids[0], ids[1], ids[2], ids[3]];
        let order = if reversed { "reverse entry order" } else { "entry order" };

        // Opalescence: made Blood Moon a creature, and is itself untouched.
        assert!(is_creature(&game, moon), "{order}: Opalescence animated Blood Moon");
        assert!(!is_creature(&game, opalescence) && !get_effective_types(&game, opalescence).contains(&CardType::Land));
        assert_eq!(pt(&game, moon), (Some(3), Some(3)), "{order}: Opalescence's 7b part, to the set it locked");

        // Ashaya: every nontoken creature — Blood Moon now among them — is a
        // land; then Blood Moon: every nonbasic land is a Mountain with no
        // abilities but the intrinsic one.
        for (name, id) in [("Blood Moon", moon), ("Ashaya", ashaya), ("the Bears", bears), ("Urborg", urborg)] {
            assert!(get_effective_types(&game, id).contains(&CardType::Land), "{order}: {name} is a land");
            assert!(land(&game, id, LandType::Mountain), "{order}: {name} is a Mountain");
            assert!(!land(&game, id, LandType::Forest), "{order}: {name} is not a Forest");
            assert!(!land(&game, id, LandType::Swamp), "{order}: {name} is not a Swamp — Urborg never applied");
            assert_eq!(taps_for(&game, 0, id), vec![ManaType::Red], "{order}: {name} taps for {{R}} only");
            assert_eq!(get_effective_abilities(&game, id).len(), 1, "{order}: {name} lost every printed ability");
        }
        assert_eq!(pt(&game, ashaya), (Some(0), Some(0)), "{order}: Ashaya's CDA went with the rest");
        assert_eq!(pt(&game, bears), (Some(2), Some(2)));

        // The basic Forest: a Forest, not a Swamp, not a Mountain.
        assert_eq!(get_effective_subtypes(&game, forest), HashSet::from([Subtype::Land(LandType::Forest)]), "{order}");
        assert_eq!(taps_for(&game, 0, forest), vec![ManaType::Green]);
    }
}

// ---------------------------------------------------------------------------
// Humility + Citanul Hierophants with the Hierophants *earlier*: LI-1's pass
// gave the answer in this order only because a later strip undid the grant;
// CR 613.8a(b) makes the grant wait for Humility, so the grant never applies
// at all — observable as no hypothetical being needed twice, and as the same
// answer. Kept beside the LI-1 pin in `phase_lf_integration_test.rs`.
// ---------------------------------------------------------------------------

#[test]
fn test_the_hierophants_grant_waits_for_humility_whatever_the_timestamps() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let hierophants = put_on_battlefield(&mut game, phase_lf_cards::citanul_hierophants(), 0);
    assert_eq!(get_effective_abilities(&game, bears).len(), 1);
    put_on_battlefield(&mut game, phase_lf_cards::humility(), 1);
    assert!(get_effective_abilities(&game, bears).is_empty());
    assert!(get_effective_abilities(&game, hierophants).is_empty());
    assert!(game.counters.dependency_checks() > 0, "the pair reached the hypothetical: a static reads its own abilities, Humility writes them");
}

/// The lowering the tests above rest on, in one place: an `AbilityDef` built
/// the way the cards build theirs is a static ability the registry lowers.
#[test]
fn test_fixture_abilities_are_static() {
    let def: AbilityDef = static_ability(Effect::Atom(
        Primitive::ChangeType(no_change(), Duration::WhileSourceOnBattlefield),
        EffectRecipient::Implicit,
    ));
    assert_eq!(def.ability_type, AbilityType::Static);
    assert_ne!(def.id, new_ability_id());
}
