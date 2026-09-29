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
//!
//! Fixtures are built inline, named for the printed card whose board they
//! stand in for, and never registered.

use std::sync::Arc;

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::engine::layers::compute_as_entering;
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::has_subtype;
use mtgsim::test_support::{put_in_graveyard, setup_two_player_game};
use mtgsim::types::card_types::{CreatureType, Subtype};
use mtgsim::types::effects::{CharacteristicEdit, CounterType, TypeChange};
use mtgsim::types::replacement::{EnterMods, TapStatus};
use mtgsim::types::zones::Zone;

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
