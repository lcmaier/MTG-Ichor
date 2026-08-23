//! Every registered card, put onto the battlefield with assertions live.
//!
//! `register_static_effects` now asserts on every shape it cannot lower
//! (`GameState::static_ability_atoms`). Those assertions are only useful if
//! something actually runs them over the card pool, and the two obvious
//! candidates do not:
//!
//! - **`fuzz_games` cannot.** It is built `--release`, where `debug_assert!`
//!   compiles out. And even in debug it would only reach cards its random decks
//!   happen to draw and cast.
//! - **The per-phase integration tests cannot.** They each place the handful of
//!   cards that phase is about.
//!
//! So this file walks `CardRegistry::default_registry()` and puts *every* card
//! onto the battlefield. Any card whose static ability would silently register
//! nothing panics here, by name, the moment it is added to the registry — which
//! is the whole point: a dropped atom produces an inert card, and an inert card
//! is invisible to every other check we have.
//!
//! It deliberately uses `put_on_battlefield` (which fires `place_on_battlefield`
//! → `register_static_effects`) rather than `place_bare`, and deliberately
//! ignores whether the card is *legal* on the battlefield — an Instant on the
//! battlefield is nonsense in Magic but exercises the lowering just the same,
//! and no SBA runs here to object.

use mtgsim::cards::registry::CardRegistry;
use mtgsim::test_support::{put_on_battlefield, setup_two_player_game};

#[test]
fn test_every_registered_card_lowers_without_dropping_anything() {
    let registry = CardRegistry::default_registry();
    let names = registry.card_names();
    assert!(names.len() > 40, "registry looks empty — {} cards", names.len());

    for name in &names {
        let card = registry
            .create(name)
            .unwrap_or_else(|e| panic!("registry cannot build {name:?}: {e}"));

        // A fresh game per card, so one card's registry rows cannot change
        // what the next card's lowering sees.
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, card, 0);
    }
}

/// The same walk, with both players controlling a copy. `ByController` filters
/// resolve against the source's controller (CR 109.5), so a card that lowers
/// cleanly for player 0 and not player 1 would be a real asymmetry.
#[test]
fn test_every_registered_card_lowers_for_either_controller() {
    let registry = CardRegistry::default_registry();

    for name in registry.card_names() {
        let Ok(card) = registry.create(name) else { continue };
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, card.clone(), 0);
        put_on_battlefield(&mut game, card, 1);
    }
}
