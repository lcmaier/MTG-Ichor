//! Printed cards checked one at a time: each builds the card it prints, and
//! a dual land does under Blood Moon what CR 305.7 says it does.

use mtgsim::cards::printings;
use mtgsim::cards::printings::{
    ancestral_recall, badlands, bayou, burst_of_energy, counterspell, earth_elemental, elvish_archers, forest, giant_growth, giant_spider,
    grizzly_bears, hill_giant, island, knight_of_meadowgrain, lightning_bolt, mountain, plains, plateau, raging_cougar, rhox_war_monk,
    ridgetop_raptor, savannah, savannah_lions, scrubland, serra_angel, swamp, taiga, thornweald_archer, tropical_island, tundra, underground_sea,
    vampire_nighthawk, volcanic_island, volcanic_upheaval, wall_of_stone, war_mammoth,
};
use mtgsim::objects::card_data::AbilityType;
use mtgsim::oracle::characteristics::{get_effective_abilities, get_effective_subtypes};
use mtgsim::test_support::{put_on_battlefield, setup_two_player_game};
use mtgsim::types::card_types::{CardType, LandType, Subtype, Supertype};
use mtgsim::types::effects::{Effect, Primitive};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::ManaType;

#[test]
fn test_lightning_bolt() {
    let bolt = lightning_bolt();
    assert_eq!(bolt.name, "Lightning Bolt");
    assert!(bolt.types.contains(&CardType::Instant));
    assert_eq!(bolt.mana_cost.as_ref().unwrap().mana_value(), 1);
    assert_eq!(bolt.abilities.len(), 1);
    assert_eq!(bolt.abilities[0].ability_type, AbilityType::Spell);
}

#[test]
fn test_ancestral_recall() {
    let recall = ancestral_recall();
    assert_eq!(recall.name, "Ancestral Recall");
    assert!(recall.types.contains(&CardType::Instant));
    assert_eq!(recall.mana_cost.as_ref().unwrap().mana_value(), 1);
}

#[test]
fn test_counterspell() {
    let cs = counterspell();
    assert_eq!(cs.name, "Counterspell");
    assert!(cs.types.contains(&CardType::Instant));
    assert_eq!(cs.mana_cost.as_ref().unwrap().mana_value(), 2);
}

#[test]
fn test_burst_of_energy() {
    let boe = burst_of_energy();
    assert_eq!(boe.name, "Burst of Energy");
    assert!(boe.types.contains(&CardType::Instant));
    assert_eq!(boe.mana_cost.as_ref().unwrap().mana_value(), 1);
}

#[test]
fn test_giant_growth() {
    let gg = giant_growth();
    assert_eq!(gg.name, "Giant Growth");
    assert!(gg.types.contains(&CardType::Instant));
    assert_eq!(gg.mana_cost.as_ref().unwrap().mana_value(), 1);
    assert_eq!(gg.abilities.len(), 1);
    assert_eq!(gg.abilities[0].ability_type, AbilityType::Spell);
}

#[test]
fn test_volcanic_upheaval() {
    let vu = volcanic_upheaval();
    assert_eq!(vu.name, "Volcanic Upheaval");
    assert!(vu.types.contains(&CardType::Instant));
    assert_eq!(vu.mana_cost.as_ref().unwrap().mana_value(), 4);
}

#[test]
fn test_all_basic_lands() {
    let lands = vec![
        (plains(), "Plains", ManaType::White),
        (island(), "Island", ManaType::Blue),
        (swamp(), "Swamp", ManaType::Black),
        (mountain(), "Mountain", ManaType::Red),
        (forest(), "Forest", ManaType::Green),
    ];

    for (land, expected_name, _expected_mana) in &lands {
        assert_eq!(land.name, *expected_name);
        assert!(land.types.contains(&CardType::Land));
        assert!(land.supertypes.contains(&Supertype::Basic));
        assert!(land.mana_cost.is_none());
        assert_eq!(land.abilities.len(), 1);
        assert!(matches!(land.abilities[0].ability_type, AbilityType::Mana(_)));
    }
}

/// Each creature's creature types as its type line prints them, in printed
/// order (Scryfall, 2026-11-01).
#[test]
fn the_fourteen_creatures_print_their_creature_types() {
    use mtgsim::types::card_types::CreatureType::*;
    let printed = [
        (printings::citanul_hierophants(), vec![Human, Druid]),
        (earth_elemental(), vec![Elemental]),
        (giant_spider(), vec![Spider]),
        (grizzly_bears(), vec![Bear]),
        (hill_giant(), vec![Giant]),
        (knight_of_meadowgrain(), vec![Kithkin, Knight]),
        (raging_cougar(), vec![Cat]),
        (rhox_war_monk(), vec![Rhino, Monk]),
        (ridgetop_raptor(), vec![Dinosaur, Beast]),
        (savannah_lions(), vec![Cat]),
        (serra_angel(), vec![Angel]),
        (thornweald_archer(), vec![Elf, Archer]),
        (vampire_nighthawk(), vec![Vampire, Shaman]),
        (war_mammoth(), vec![Elephant]),
    ];
    for (card, types) in printed {
        let expected: Vec<Subtype> = types.into_iter().map(Subtype::Creature).collect();
        let subtypes: Vec<Subtype> = card.subtypes.iter().cloned().collect();
        assert_eq!(subtypes, expected, "{}", card.name);
    }
}

#[test]
fn test_grizzly_bears() {
    let bears = grizzly_bears();
    assert_eq!(bears.name, "Grizzly Bears");
    assert!(bears.types.contains(&CardType::Creature));
    assert_eq!(bears.mana_cost.as_ref().unwrap().mana_value(), 2);
    assert_eq!(bears.power, Some(2));
    assert_eq!(bears.toughness, Some(2));
}

#[test]
fn test_hill_giant() {
    let giant = hill_giant();
    assert_eq!(giant.name, "Hill Giant");
    assert!(giant.types.contains(&CardType::Creature));
    assert_eq!(giant.mana_cost.as_ref().unwrap().mana_value(), 4);
    assert_eq!(giant.power, Some(3));
    assert_eq!(giant.toughness, Some(3));
}

#[test]
fn test_savannah_lions() {
    let lions = savannah_lions();
    assert_eq!(lions.name, "Savannah Lions");
    assert!(lions.types.contains(&CardType::Creature));
    assert_eq!(lions.mana_cost.as_ref().unwrap().mana_value(), 1);
    assert_eq!(lions.power, Some(2));
    assert_eq!(lions.toughness, Some(1));
}

#[test]
fn test_earth_elemental() {
    let elemental = earth_elemental();
    assert_eq!(elemental.name, "Earth Elemental");
    assert!(elemental.types.contains(&CardType::Creature));
    assert_eq!(elemental.mana_cost.as_ref().unwrap().mana_value(), 5);
    assert_eq!(elemental.power, Some(4));
    assert_eq!(elemental.toughness, Some(5));
}

#[test]
fn test_serra_angel() {
    let card = serra_angel();
    assert_eq!(card.name, "Serra Angel");
    assert_eq!(card.power, Some(4));
    assert_eq!(card.toughness, Some(4));
    assert!(card.keyword_flags.contains(&KeywordFlag::Flying));
    assert!(card.keyword_flags.contains(&KeywordFlag::Vigilance));
    assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 5);
}

#[test]
fn test_thornweald_archer() {
    let card = thornweald_archer();
    assert_eq!(card.name, "Thornweald Archer");
    assert_eq!(card.power, Some(2));
    assert_eq!(card.toughness, Some(1));
    assert!(card.keyword_flags.contains(&KeywordFlag::Reach));
    assert!(card.keyword_flags.contains(&KeywordFlag::Deathtouch));
    assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 2);
}

#[test]
fn test_raging_cougar() {
    let card = raging_cougar();
    assert_eq!(card.name, "Raging Cougar");
    assert_eq!(card.power, Some(2));
    assert_eq!(card.toughness, Some(2));
    assert!(card.keyword_flags.contains(&KeywordFlag::Haste));
    assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 3);
}

#[test]
fn test_wall_of_stone() {
    let card = wall_of_stone();
    assert_eq!(card.name, "Wall of Stone");
    assert_eq!(card.power, Some(0));
    assert_eq!(card.toughness, Some(8));
    assert!(card.keyword_flags.contains(&KeywordFlag::Defender));
    assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 3);
}

#[test]
fn test_elvish_archers() {
    let card = elvish_archers();
    assert_eq!(card.name, "Elvish Archers");
    assert_eq!(card.power, Some(2));
    assert_eq!(card.toughness, Some(1));
    assert!(card.keyword_flags.contains(&KeywordFlag::FirstStrike));
    assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 2);
}

#[test]
fn test_ridgetop_raptor() {
    let card = ridgetop_raptor();
    assert_eq!(card.name, "Ridgetop Raptor");
    assert_eq!(card.power, Some(2));
    assert_eq!(card.toughness, Some(1));
    assert!(card.keyword_flags.contains(&KeywordFlag::DoubleStrike));
    assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 4);
}

#[test]
fn test_war_mammoth() {
    let card = war_mammoth();
    assert_eq!(card.name, "War Mammoth");
    assert_eq!(card.power, Some(3));
    assert_eq!(card.toughness, Some(3));
    assert!(card.keyword_flags.contains(&KeywordFlag::Trample));
    assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 4);
}

#[test]
fn test_knight_of_meadowgrain() {
    let card = knight_of_meadowgrain();
    assert_eq!(card.name, "Knight of Meadowgrain");
    assert_eq!(card.power, Some(2));
    assert_eq!(card.toughness, Some(2));
    assert!(card.keyword_flags.contains(&KeywordFlag::FirstStrike));
    assert!(card.keyword_flags.contains(&KeywordFlag::Lifelink));
    assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 2);
}

#[test]
fn test_rhox_war_monk() {
    let card = rhox_war_monk();
    assert_eq!(card.name, "Rhox War Monk");
    assert_eq!(card.power, Some(3));
    assert_eq!(card.toughness, Some(4));
    assert!(card.keyword_flags.contains(&KeywordFlag::Lifelink));
    assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 3);
}

#[test]
fn test_giant_spider() {
    let card = giant_spider();
    assert_eq!(card.name, "Giant Spider");
    assert_eq!(card.power, Some(2));
    assert_eq!(card.toughness, Some(4));
    assert!(card.keyword_flags.contains(&KeywordFlag::Reach));
    assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 4);
}

#[test]
fn test_vampire_nighthawk() {
    let card = vampire_nighthawk();
    assert_eq!(card.name, "Vampire Nighthawk");
    assert_eq!(card.power, Some(2));
    assert_eq!(card.toughness, Some(3));
    assert!(card.keyword_flags.contains(&KeywordFlag::Flying));
    assert!(card.keyword_flags.contains(&KeywordFlag::Lifelink));
    assert!(card.keyword_flags.contains(&KeywordFlag::Deathtouch));
    assert_eq!(card.mana_cost.as_ref().unwrap().mana_value(), 3);
}

/// The interaction these cards exist to make reachable in random play, using
/// the same factories the registry hands to `fuzz_games` rather than a
/// hand-built fixture.
///
/// CR 305.7: setting a land's subtypes to a basic land type strips its old
/// land types and the abilities that came with them, and grants the intrinsic
/// mana ability for the new type.
#[test]
fn blood_moon_turns_a_dual_into_a_mountain_that_taps_for_red() {
    let mut game = setup_two_player_game();
    let sea = put_on_battlefield(&mut game, underground_sea(), 0);

    let produced = |game: &_, id| -> Vec<ManaType> {
        get_effective_abilities(game, id)
            .iter()
            .filter_map(|a| match &a.effect {
                Effect::Atom(Primitive::ProduceMana(out), _) => Some(out.mana[0].0),
                _ => None,
            })
            .collect()
    };
    assert_eq!(produced(&game, sea), vec![ManaType::Blue, ManaType::Black]);

    put_on_battlefield(&mut game, printings::blood_moon(), 1);

    assert_eq!(
        get_effective_subtypes(&game, sea)
            .into_iter()
            .collect::<Vec<_>>(),
        vec![Subtype::Land(LandType::Mountain)],
        "CR 305.7: the old land types are gone"
    );
    assert_eq!(
        produced(&game, sea),
        vec![ManaType::Red],
        "blue and black went with the land types, and red came with Mountain"
    );
}

#[test]
fn duals_are_nonbasic_lands_with_two_types_and_two_mana_abilities() {
    for card in [
        tundra(), underground_sea(), badlands(), taiga(), savannah(),
        scrubland(), volcanic_island(), bayou(), plateau(), tropical_island(),
    ] {
        assert!(card.types.contains(&CardType::Land), "{}", card.name);
        // The whole reason these exist: CR 305.8 makes a land nonbasic on the
        // supertype alone, which is what Blood Moon's filter selects.
        assert!(
            !card.supertypes.contains(&Supertype::Basic),
            "{} must be nonbasic to be a Blood Moon target",
            card.name
        );
        assert_eq!(card.subtypes.iter().count(), 2, "{}", card.name);
        assert_eq!(card.abilities.len(), 2, "{}", card.name);
    }
}
