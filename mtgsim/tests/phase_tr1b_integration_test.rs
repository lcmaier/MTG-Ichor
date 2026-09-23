//! Phase TR-1b — the dispatch audit, and the departure frame it was built
//! after (`triggers-architecture.md` §4.10, §12's TR-1b row).
//!
//! **Item 174 first.** CR 603.10a's frame is the permanent as it was
//! immediately before the event, and one event can take several permanents.
//! The frames are captured at the batch's seam, between deciding and
//! performing, so no member's move shows another the board after an earlier
//! member left. Each board runs in both batch orders, because the order is
//! exactly what the frame must not depend on.

use std::sync::Arc;

use mtgsim::cards::artifacts::sol_ring;
use mtgsim::cards::authoring::{dies, triggered_ability, whenever};
use mtgsim::cards::phase_ld_cards::march_of_the_machines;
use mtgsim::cards::phase_lf_cards::humility;
use mtgsim::cards::phase_tr1_cards::blood_artist;
use mtgsim::engine::actions::{DestructionSource, GameAction};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_on_battlefield, setup_two_player_game, test_ctx};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, ObjectFilter, Primitive};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::zones::Zone;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

fn pending(game: &GameState) -> usize {
    game.pending_triggers.len()
}

/// Destroy `objects` as one event (CR 608.2f), in the order given.
fn destroy_all(game: &mut GameState, objects: &[ObjectId], source: ObjectId) {
    let batch = objects
        .iter()
        .map(|&object| GameAction::Destroy { object, source: DestructionSource::Effect(source) })
        .collect();
    game.execute_actions(batch, &test_ctx()).expect("the wipe");
}

/// An enchantment that watches creatures die, and is not one itself.
fn mourning_shrine() -> Arc<CardData> {
    CardDataBuilder::new("Mourning Shrine")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability(whenever(dies(ObjectFilter::ByType(CardType::Creature)), gain_one())))
        .build()
}

// ---------------------------------------------------------------------------
// Item 174 — the departure frame is the board before the event (CR 603.10a)
// ---------------------------------------------------------------------------

/// Humility and Blood Artist destroyed as one event. Before it Blood Artist
/// had no abilities, so its death triggers nothing, whichever of the two
/// the batch moves first. With Humility first, a frame taken at Blood
/// Artist's own move saw Humility already gone and triggered once.
// COVERS-PARTIAL: ATOM-603.10a-001
#[test]
fn a_departure_frame_does_not_see_an_earlier_member_leave() {
    for humility_first in [true, false] {
        let mut game = setup_two_player_game();
        let artist = put_on_battlefield(&mut game, blood_artist(), 0);
        let enchantment = put_on_battlefield(&mut game, humility(), 1);
        let source = put_on_battlefield(&mut game, sol_ring(), 1);
        let order = if humility_first { [enchantment, artist] } else { [artist, enchantment] };

        destroy_all(&mut game, &order, source);

        assert_eq!(game.get_object(artist).unwrap().zone, Zone::Graveyard);
        assert_eq!(pending(&game), 0, "no ability before the event (humility_first: {humility_first})");
    }
}

/// The frame's appearance, the same fault's other half: an artifact March of
/// the Machines animates dies as a creature, even when March leaves first in
/// the same event. A frame taken after March left framed it as a noncreature
/// artifact, and "whenever a creature dies" missed it.
// COVERS-PARTIAL: ATOM-603.10a-001
#[test]
fn a_departure_frame_keeps_the_type_an_earlier_member_gave_it() {
    for march_first in [true, false] {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, mourning_shrine(), 0);
        let march = put_on_battlefield(&mut game, march_of_the_machines(), 1);
        let ring = put_on_battlefield(&mut game, sol_ring(), 1);
        let source = put_on_battlefield(&mut game, sol_ring(), 0);
        let order = if march_first { [march, ring] } else { [ring, march] };

        destroy_all(&mut game, &order, source);

        assert_eq!(pending(&game), 1, "the ring died a creature (march_first: {march_first})");
    }
}
