//! Phase RG integration tests: the entry state, CR 614.1c's two halves as one
//! shape (`replacement-architecture.md` §9, Phase RG).
//!
//! What this file proves, in the order the phase built it:
//!
//! 1. **The shape.** What a permanent enters *as* is applied at its own
//!    timestamp at each edit's layer, in the CR 614.12 frame and on the
//!    permanent alike; what it enters *with* is state, and the last applied
//!    status wins (D1, D2, D5). That the frame and the performer build one
//!    permanent is a unit test beside `Lookahead`.
//! 2. **The feeds table** (D3). CR 616.1's order is asked exactly where one
//!    member's write can make another stop applying, or where two members set
//!    opposite statuses, and nowhere else.
//! 3. **Master Biomancer's Mutant** (D4). The first `CharacteristicEdit`: at
//!    layer 4 at the creature's timestamp, not copied, and kept after the
//!    Biomancer leaves.
//!
//! Fixtures are built inline, named for the printed card whose board they
//! stand in for, and never registered.

use std::collections::HashSet;
use std::sync::Arc;

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::keyword_creatures::wall_of_stone;
use mtgsim::cards::phase_rc_cards::master_biomancer;
use mtgsim::engine::actions::{ActionContext, ZoneChangeCause};
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, Layer};
use mtgsim::engine::layers::{compute_as_entering, copiable_values};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::{get_effective_name, has_subtype};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    creature_with_ability, put_in_graveyard, put_on_battlefield, registered, setup_two_player_game,
    static_ability, test_ctx,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::effects::{
    CharacteristicEdit, Condition, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet,
    PlayerRef, Primitive, TypeChange,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::{EnterMods, EnterModsTemplate, EventPattern, ReplacementDef, Rewrite, TapStatus};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

fn mutant() -> Subtype {
    Subtype::Creature(CreatureType::Mutant)
}

/// Master Biomancer's edit, "as a Mutant in addition to its other types".
fn as_a_mutant() -> CharacteristicEdit {
    CharacteristicEdit::Types(TypeChange { add_subtypes: vec![mutant()], ..TypeChange::NONE })
}

fn entering_as(edits: Vec<CharacteristicEdit>) -> EnterMods {
    EnterMods { edits: Some(Arc::from(edits)), ..EnterMods::NONE }
}

fn creatures() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Creature)
}

fn and(a: ObjectFilter, b: ObjectFilter) -> ObjectFilter {
    ObjectFilter::And(Box::new(a), Box::new(b))
}

/// An enchantment whose one ability modifies how the permanents `filter`
/// matches enter the battlefield.
fn entry_effect(name: &str, filter: ObjectFilter, template: EnterModsTemplate) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::battlefield_filter(filter),
            Rewrite::EnterWith(template),
        )))))
        .build()
}

/// Master Biomancer's Mutant clause alone: "Each other creature you control
/// enters ... as a Mutant in addition to its other types."
fn creatures_enter_as_mutants() -> Arc<CardData> {
    entry_effect(
        "Creatures enter as Mutants",
        and(creatures(), ObjectFilter::ByController(PlayerRef::You)),
        EnterModsTemplate { status: None, counters: Vec::new(), edits: vec![as_a_mutant()] },
    )
}

fn with_a_charge_counter() -> EnterModsTemplate {
    EnterModsTemplate::with_counters(CounterType::Charge, 1)
}

/// Return `card` from `owner`'s graveyard to the battlefield, answering each
/// CR 616.1 prompt about it from `picks` in order. An unscripted prompt, or a
/// scripted one left over, fails the test.
fn return_to_battlefield(game: &mut GameState, card: Arc<CardData>, owner: PlayerId, picks: &[usize]) -> ObjectId {
    let id = put_in_graveyard(game, card, owner);
    let dp = ScriptedDecisionProvider::new();
    for &pick in picks {
        dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: Some(id) }, vec![pick]);
    }
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &ActionContext::new(&dp))
        .expect("the entry is proposed");
    assert!(dp.is_empty(), "every scripted CR 616.1 prompt was asked");
    id
}

// ---------------------------------------------------------------------------
// 1. The shape
// ---------------------------------------------------------------------------

/// An edit is what the permanent enters as, so the CR 614.12 frame reads it
/// before the entry and the permanent carries it after, at layer 4 both
/// times, with no registry row.
#[test]
fn an_entry_edit_is_in_the_frame_and_on_the_permanent() {
    let mut game = setup_two_player_game();
    let mods = entering_as(vec![as_a_mutant()]);

    let bears = put_in_graveyard(&mut game, grizzly_bears(), 0);
    let frame = compute_as_entering(&game, bears, 0, &mods).expect("the bears exist");
    assert!(frame.subtypes.contains(&mutant()), "the frame is the permanent as it would exist");
    assert!(!has_subtype(&game, bears, &mutant()), "the card in the graveyard is not a Mutant");

    let rows = game.continuous_effects.iter().count();
    let placed = game.add_object(GameObject::new(grizzly_bears(), 0, Zone::Battlefield));
    game.place_on_battlefield(placed, 0, &mods);
    assert!(has_subtype(&game, placed, &mutant()));
    assert_eq!(game.continuous_effects.iter().count(), rows, "state on the permanent, not a row");
}

/// CR 110.5b gives a permanent one tapped/untapped status, and the last
/// effect applied sets it (Spelunking's first ruling). An effect that names no
/// status leaves it, and counters still add.
#[test]
fn the_last_applied_status_is_the_one_the_permanent_enters_with() {
    let untapped = EnterMods { status: Some(TapStatus::Untapped), ..EnterMods::NONE };

    let mut mods = EnterMods::tapped();
    mods.merge(&untapped);
    assert!(!mods.enters_tapped(), "untapped applied last");
    mods.merge(&EnterMods::tapped());
    assert!(mods.enters_tapped(), "tapped applied last");
    mods.merge(&EnterMods::with_counters(CounterType::PlusOnePlusOne, 1));
    assert!(mods.enters_tapped(), "an effect that names no status leaves it");
    assert_eq!(mods.counters.len(), 1);
    assert!(!EnterMods::NONE.enters_tapped(), "CR 110.5b's default");
}

// ---------------------------------------------------------------------------
// 2. The feeds table
// ---------------------------------------------------------------------------

/// Premise (c): two statuses that differ are an order, since the last one
/// applied is the one the permanent enters with, and the entering permanent's
/// controller chooses it. Two that agree, or a status beside counters, are not.
#[test]
fn opposite_statuses_are_an_order_and_equal_ones_are_not() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, entry_effect("Permanents enter tapped", ObjectFilter::All, EnterModsTemplate::tapped()), 1);
    put_on_battlefield(&mut game, entry_effect("Permanents enter untapped", ObjectFilter::All, EnterModsTemplate::untapped()), 1);
    let tapped_first = return_to_battlefield(&mut game, grizzly_bears(), 0, &[0]);
    assert!(!game.battlefield[&tapped_first].tapped, "untapped applied last");
    let untapped_first = return_to_battlefield(&mut game, grizzly_bears(), 0, &[1]);
    assert!(game.battlefield[&untapped_first].tapped, "tapped applied last");

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, entry_effect("Permanents enter tapped", ObjectFilter::All, EnterModsTemplate::tapped()), 1);
    put_on_battlefield(&mut game, entry_effect("Creatures enter tapped", creatures(), EnterModsTemplate::tapped()), 1);
    put_on_battlefield(&mut game, entry_effect("Creatures enter charged", creatures(), with_a_charge_counter()), 1);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    let entry = &game.battlefield[&bears];
    assert!(entry.tapped);
    assert_eq!(entry.counter_count(CounterType::Charge), 1);
}

/// `PowerLE` can be unmatched only by something that raises power. Beside a
/// status and a charge counter nothing does, so nothing is asked; it was
/// asked before the table, whatever stood beside it.
#[test]
fn a_power_filter_beside_members_that_raise_no_power_is_not_asked() {
    let mut game = setup_two_player_game();
    let small = and(creatures(), ObjectFilter::PowerLE(2));
    put_on_battlefield(&mut game, entry_effect("Small creatures enter tapped", small, EnterModsTemplate::tapped()), 1);
    put_on_battlefield(&mut game, entry_effect("Creatures enter charged", creatures(), with_a_charge_counter()), 1);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    let entry = &game.battlefield[&bears];
    assert!(entry.tapped);
    assert_eq!(entry.counter_count(CounterType::Charge), 1);
}

/// Premise (a) with an edit. Adding Mutant can turn a leaf only on, so it can
/// unmatch "non-Mutant creatures" and nothing positive: a Mutant filter is
/// either not a candidate yet, and becomes one after the edit applies
/// (CR 616.2), or matches already; a type filter is untouched.
#[test]
fn an_added_subtype_asks_only_where_a_not_reads_it() {
    let non_mutants = and(creatures(), ObjectFilter::Not(Box::new(ObjectFilter::BySubtype(mutant()))));
    for (pick, tapped) in [(0, false), (1, true)] {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, creatures_enter_as_mutants(), 0);
        put_on_battlefield(&mut game, entry_effect("Non-Mutants enter tapped", non_mutants.clone(), EnterModsTemplate::tapped()), 1);
        let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[pick]);
        assert!(has_subtype(&game, bears, &mutant()));
        assert_eq!(game.battlefield[&bears].tapped, tapped, "the Mutant edit first unmatches the tapper");
    }

    let mutants = and(creatures(), ObjectFilter::BySubtype(mutant()));
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, creatures_enter_as_mutants(), 0);
    put_on_battlefield(&mut game, entry_effect("Mutants enter tapped", mutants, EnterModsTemplate::tapped()), 1);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    assert!(game.battlefield[&bears].tapped, "a candidate once the edit applied (CR 616.2)");
    let born_mutant = CardDataBuilder::new("Born Mutant")
        .card_type(CardType::Creature)
        .subtype(mutant())
        .power_toughness(2, 2)
        .build();
    let mutant_id = return_to_battlefield(&mut game, born_mutant, 0, &[]);
    assert!(game.battlefield[&mutant_id].tapped, "both applied, in either order");

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, creatures_enter_as_mutants(), 0);
    put_on_battlefield(&mut game, entry_effect("Creatures enter tapped", creatures(), EnterModsTemplate::tapped()), 1);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    assert!(game.battlefield[&bears].tapped && has_subtype(&game, bears, &mutant()));
}

/// "As long as this creature is untapped, it's an artifact in addition to its
/// other types." No printed card says this; it is the one condition leaf that
/// reads entry state today, on the object the entry is deciding.
fn artifact_while_untapped() -> Arc<CardData> {
    creature_with_ability(
        "Artifact While Untapped",
        2,
        2,
        static_ability(Effect::Conditional(
            Condition::SourceUntapped,
            Box::new(Effect::Atom(
                Primitive::ChangeType(
                    TypeChange { add_types: vec![CardType::Artifact], ..TypeChange::NONE },
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::ThisObject,
            )),
        )),
    )
}

/// The table's last row. The entering creature is an artifact only while
/// untapped, so "enters tapped" applied first unmatches "artifacts enter with
/// a charge counter", though no member writes a type: the two orders give a
/// counter or none, and the controller is asked.
#[test]
fn a_status_write_is_an_order_where_the_entering_permanents_type_hangs_on_its_status() {
    for (pick, charged) in [(0, 0), (1, 1)] {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, entry_effect("Permanents enter tapped", ObjectFilter::All, EnterModsTemplate::tapped()), 1);
        let artifacts = ObjectFilter::ByType(CardType::Artifact);
        put_on_battlefield(&mut game, entry_effect("Artifacts enter charged", artifacts, with_a_charge_counter()), 1);
        let id = return_to_battlefield(&mut game, artifact_while_untapped(), 0, &[pick]);
        let entry = &game.battlefield[&id];
        assert!(entry.tapped);
        assert_eq!(entry.counter_count(CounterType::Charge), charged);
    }
}

// ---------------------------------------------------------------------------
// 3. Master Biomancer's Mutant (D4)
// ---------------------------------------------------------------------------

/// "Each other creature you control enters with ... and as a Mutant in
/// addition to its other types." The Mutant is how the creature entered, so
/// it outlives the Biomancer, whose effect it never was. The counters are its
/// power as the creature enters (its one ruling).
// COVERS: ATOM-614.1c-001
#[test]
fn a_creature_entering_under_master_biomancer_stays_a_mutant_after_it_leaves() {
    let mut game = setup_two_player_game();
    let biomancer = put_on_battlefield(&mut game, master_biomancer(), 0);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    assert_eq!(game.battlefield[&bears].counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(has_subtype(&game, bears, &mutant()));

    game.change_zone(biomancer, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx()).unwrap();
    assert!(has_subtype(&game, bears, &mutant()), "how it entered, not the Biomancer's effect");
}

/// The Mutant applies at layer 4 at the creature's own timestamp (CR 613.7),
/// so an effect that sets creature types with an earlier timestamp applies
/// under it, and one with a later timestamp over it (CR 205.1a).
// COVERS-PARTIAL: ATOM-614.1c-001
#[test]
fn a_later_layer_4_effect_applies_over_the_mutant_and_an_earlier_one_under_it() {
    let slivers = |game: &mut GameState, source: ObjectId| {
        let timestamp = game.allocate_timestamp();
        let set = EffectModification::SetSubtypes(HashSet::from([Subtype::Creature(CreatureType::Sliver)]));
        game.continuous_effects.add(ContinuousEffect {
            affected_objects: ObjectSet::battlefield_filter(creatures()),
            ..registered(source, Layer::Layer4Type, timestamp, set)
        });
    };
    let sliver = Subtype::Creature(CreatureType::Sliver);

    let mut game = setup_two_player_game();
    let biomancer = put_on_battlefield(&mut game, master_biomancer(), 0);
    slivers(&mut game, biomancer);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    assert!(has_subtype(&game, bears, &sliver) && has_subtype(&game, bears, &mutant()), "earlier: under it");

    let mut game = setup_two_player_game();
    let biomancer = put_on_battlefield(&mut game, master_biomancer(), 0);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    slivers(&mut game, biomancer);
    assert!(has_subtype(&game, bears, &sliver) && !has_subtype(&game, bears, &mutant()), "later: over it");
}

/// The Mutant is not a copiable value (CR 707.2's last sentence), so a copy
/// of the creature is not a Mutant; and a copy effect over the creature
/// replaces its copiable values and leaves the Mutant, which is not one.
#[test]
fn the_mutant_is_not_copied_and_a_copy_effect_over_it_leaves_it() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, master_biomancer(), 0);
    let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[]);
    let values = copiable_values(&game, bears).expect("the bears are on the battlefield");
    assert!(!values.subtypes.contains(&mutant()), "a Clone of it would not be a Mutant");

    let wall = put_on_battlefield(&mut game, wall_of_stone(), 1);
    let cytoshape_row = EffectModification::CopyFrom(Arc::new(copiable_values(&game, wall).unwrap()));
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(registered(bears, Layer::Layer1Copy, timestamp, cytoshape_row));
    assert_eq!(get_effective_name(&game, bears), "Wall of Stone");
    assert!(has_subtype(&game, bears, &mutant()), "the copy replaced the copiable values, and the Mutant is not one");
}

/// D3's example on the printed card: beside "non-Mutant creatures enter
/// tapped", Master Biomancer applied first unmatches the tapper, so the
/// entering creature's controller is asked.
#[test]
fn master_biomancer_beside_a_non_mutant_filter_is_an_order() {
    let non_mutants = and(creatures(), ObjectFilter::Not(Box::new(ObjectFilter::BySubtype(mutant()))));
    for (pick, tapped) in [(0, false), (1, true)] {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, master_biomancer(), 0);
        put_on_battlefield(&mut game, entry_effect("Non-Mutants enter tapped", non_mutants.clone(), EnterModsTemplate::tapped()), 1);
        let bears = return_to_battlefield(&mut game, grizzly_bears(), 0, &[pick]);
        assert_eq!(game.battlefield[&bears].counter_count(CounterType::PlusOnePlusOne), 2);
        assert_eq!(game.battlefield[&bears].tapped, tapped);
    }
}
