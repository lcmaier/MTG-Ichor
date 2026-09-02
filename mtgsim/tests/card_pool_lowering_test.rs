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

/// CR 202.2 — a card's colors must be the colors of its mana cost.
///
/// # Why a test rather than a derivation
///
/// `CardDataBuilder::color()` is hand-written on all 52 call sites and is
/// **redundant with the mana cost on every card in the registry today** — this
/// test measured zero disagreements the day it was written. The right end state
/// is deriving it in the builder, which is one small PR and is not this one:
/// CR 202.2e's colour indicator is printed data that no mana cost implies
/// (Dryad Arbor is a green land with no mana cost; Ancestral Vision is blue with
/// none), so the derivation needs an override rather than a deletion, and CR
/// 702.114a's devoid is a CDA that belongs in Layer 5 rather than in `CardData`.
///
/// Until then this is the guard that makes deferring safe. A miscoloured card is
/// exactly the failure class this file exists for: it produces a card that is
/// quietly the wrong colour, which no other check we have would notice.
///
/// **When it starts mattering: hybrid.** A `{W/U}` card is *both* white and blue
/// (CR 202.2d), and that is the first place a human writing `.color()` by hand
/// gets it wrong. The registry has no hybrid card yet, so the error class has
/// had no chance to appear.
///
/// **A card with a colour indicator is checked against the indicator** (CR
/// 204.1 / 202.2e), not against its mana cost — Dryad Arbor has no cost and is
/// green. A card that fails here is wrong about its colour or its cost, never
/// a case for bending the card to the check.
#[test]
fn test_every_registered_cards_color_matches_its_mana_cost() {
    use mtgsim::types::colors::Color;
    use mtgsim::types::mana::{ManaSymbol, ManaType};

    fn color_of(m: ManaType) -> Option<Color> {
        match m {
            ManaType::White => Some(Color::White),
            ManaType::Blue => Some(Color::Blue),
            ManaType::Black => Some(Color::Black),
            ManaType::Red => Some(Color::Red),
            ManaType::Green => Some(Color::Green),
            // CR 202.2b — {C} is not a colour.
            ManaType::Colorless => None,
        }
    }

    /// CR 202.2 + 202.2d: every colored symbol contributes, and a hybrid or
    /// Phyrexian symbol contributes *all* of its colours.
    fn derived(symbols: &[ManaSymbol]) -> Vec<Color> {
        let mut out: Vec<Color> = Vec::new();
        let push = |out: &mut Vec<Color>, c: Option<Color>| {
            if let Some(c) = c {
                if !out.contains(&c) {
                    out.push(c);
                }
            }
        };
        for s in symbols {
            match s {
                ManaSymbol::Colored(m) | ManaSymbol::MonoHybrid(m) | ManaSymbol::Phyrexian(m) => {
                    push(&mut out, color_of(*m))
                }
                ManaSymbol::Hybrid(a, b) | ManaSymbol::HybridPhyrexian(a, b) => {
                    push(&mut out, color_of(*a));
                    push(&mut out, color_of(*b));
                }
                // Generic, {C}, snow and X carry no colour.
                ManaSymbol::Generic
                | ManaSymbol::Colorless
                | ManaSymbol::Snow
                | ManaSymbol::X => {}
            }
        }
        out
    }

    let registry = CardRegistry::default_registry();
    let mut mismatches = Vec::new();

    for name in registry.card_names() {
        let Ok(card) = registry.create(&name) else { continue };
        let mut declared: Vec<Color> = card.colors.iter().copied().collect();
        // CR 202.2b — no mana cost at all means no coloured symbols, so
        // colorless. CR 204.1 is the exception: a colour indicator defines
        // the colour outright, whatever the cost says. Dryad Arbor is the
        // first registered card with one.
        let mut expected = match (&card.color_indicator, &card.mana_cost) {
            (Some(indicator), _) => indicator.clone(),
            (None, Some(cost)) => derived(&cost.symbols),
            (None, None) => Vec::new(),
        };
        let key = |c: &Color| format!("{c:?}");
        declared.sort_by_key(key);
        expected.sort_by_key(key);
        if declared != expected {
            mismatches.push(format!("  {name}: declared {declared:?}, mana cost implies {expected:?}"));
        }
    }

    assert!(
        mismatches.is_empty(),
        "CR 202.2 — these cards' colors disagree with their mana costs:\n{}",
        mismatches.join("\n"),
    );
}
