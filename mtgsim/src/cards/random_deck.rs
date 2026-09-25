//! `fuzz_games`' deck, in the library so that a test can deal the same games:
//! the clone-bound test (`tests/clone_bound_test.rs`) plays the harness's
//! seeds, and a failure there reproduces with `fuzz_games --seed`.

use std::sync::Arc;

use rand::rngs::StdRng;
use rand::seq::IndexedRandom;

use crate::cards::registry::CardRegistry;
use crate::objects::card_data::CardData;
use crate::types::card_types::{CardType, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{Effect, Primitive};
use crate::types::mana::ManaType;

/// Map a Color to its corresponding basic land name.
fn color_to_land(color: Color) -> &'static str {
    match color {
        Color::White => "Plains",
        Color::Blue => "Island",
        Color::Black => "Swamp",
        Color::Red => "Mountain",
        Color::Green => "Forest",
    }
}

fn mana_type_to_color(mt: ManaType) -> Option<Color> {
    match mt {
        ManaType::White => Some(Color::White),
        ManaType::Blue => Some(Color::Blue),
        ManaType::Black => Some(Color::Black),
        ManaType::Red => Some(Color::Red),
        ManaType::Green => Some(Color::Green),
        ManaType::Colorless => None,
    }
}

/// How many of a deck's land slots are basic lands — one of each type.
///
/// Basics are what CR 305.7's "nonbasic" leaves alone, so a deck keeps one of
/// each for the contrast: under Blood Moon they are the only lands still making
/// anything but red. Every other land slot is nonbasic, below.
pub const BASIC_LANDS_PER_DECK: usize = 5;

/// The share of a deck that is not a land, in percent — **a proportion, not a
/// count**, so `--deck-size` scales it: 60 cards is 36 nonlands and 24 lands,
/// 100 is 60 and 40, and a bigger board is a longer game rather than a
/// different curve.
///
/// Percent rather than a fraction of the deck size, because the arithmetic has
/// to be exact: a float share would round, and a deck that came out one card
/// short of its own size would move every draw after it.
///
/// **The two land tiers below stay counts, and that is deliberate.**
/// `BASIC_LANDS_PER_DECK` is a guarantee — one of each of the five basic types,
/// which is what `every_deck_can_make_every_color_and_keeps_a_basic_of_each_type`
/// asserts — and `NONBASIC_LANDS_PER_DECK` is a flat placeholder over a static
/// pool by its own admission. The any-color fill tier absorbs the difference,
/// so a 100-card deck's mana base is proportionally more of it.
pub const NONLAND_PERCENT: usize = 60;

/// How many of a deck's land slots are drawn from the registry's nonbasic
/// lands — the ten original duals and Everywhere, uniformly.
///
/// A flat constant over a static pool, not a mana-base model. Replace it with a
/// real picker when card breadth (Phase 8) gives it something to choose between.
pub const NONBASIC_LANDS_PER_DECK: usize = 5;

/// Build one deck of `deck_size` cards — sixty unless `--deck-size` says
/// otherwise.
///
/// 1. `NONLAND_PERCENT` of the slots are nonlands, drawn uniformly with
///    repeats from **every** nonland the registry holds. No color filter: the
///    mana base below can pay for anything, so a filter would only decide which
///    slice of the pool a card gets to meet.
/// 2. `NONBASIC_LANDS_PER_DECK` slots from the registry's nonbasic lands.
/// 3. `BASIC_LANDS_PER_DECK` basics, one of each type.
/// 4. Every remaining land slot is a land that taps for all five colors —
///    Everywhere, today. A pool with no such land falls back to basics.
///
/// Deliberately crude — a fuzz harness, not a deckbuilder. It exists to
/// produce a legal deck that casts spells and attacks. **Not singleton**, at
/// any size: Commander's one-of rule (CR 903.5b) would need a pool larger than
/// the `performance` one, and what the 100-card board is here for is the length
/// and the object count, not the deck-building law.
///
/// `copies` is `--copies`: how many of each required nonland the deck gets.
/// It multiplies the slot fill below and nothing else â the deck size, the
/// land count and every RNG draw are what they were, so a heavy board differs
/// from a one-copy one in the required cards' share of the 36 nonland slots
/// and in no other way. 1 reproduces every recorded `--require` row.
///
/// `required` is `--require`'s list, and **an empty list must leave this
/// function exactly as it was**: every RNG draw below is guarded so that the
/// stream, the deck and therefore every recorded number are unchanged when the
/// flag is absent, which is what lets a reachability run and a timing run come
/// from the same binary.
pub fn random_deck(
    registry: &CardRegistry,
    rng: &mut StdRng,
    required: &[Arc<CardData>],
    copies: usize,
    deck_size: usize,
) -> Vec<Arc<CardData>> {
    let build = |name: &str| registry.create(name).ok();
    let is_land = |card: &CardData| card.types.contains(&CardType::Land);

    let nonland_names: Vec<&str> = registry
        .card_names()
        .into_iter()
        .filter(|name| build(name).is_some_and(|card| !is_land(&card)))
        .collect();

    let mut deck: Vec<Arc<CardData>> = Vec::with_capacity(deck_size);

    for _ in 0..(deck_size * NONLAND_PERCENT / 100) {
        if nonland_names.is_empty() {
            break;
        }
        let name = nonland_names.choose(rng).unwrap();
        if let Some(card) = build(name) {
            deck.push(card);
        }
    }

    // One copy of each required **nonland** card, replacing a nonland slot so
    // the deck size holds and mana density and the draw curve are untouched.
    // The first slots, no RNG draw: the library is shuffled in-game anyway, and a draw here
    // would move the stream for every later card. A required *land* is held
    // back to the land section below for the same reason: a nonland slot spent
    // on it would quietly make the deck 35/25.
    let mut required_lands: Vec<&Arc<CardData>> = Vec::new();
    let mut slot = 0usize;
    for card in required {
        if is_land(card) {
            required_lands.push(card);
            continue;
        }
        // `--copies` multiplies here and only here.
        for _ in 0..copies {
            if slot < deck.len() {
                deck[slot] = card.clone();
            } else {
                deck.push(card.clone());
            }
            slot += 1;
        }
    }

    // Pad remaining nonland slots with lands if card pool is too small
    let nonland_count = deck.len();
    let land_count = deck_size.saturating_sub(nonland_count);

    let nonbasic_names: Vec<&str> = registry
        .card_names()
        .into_iter()
        .filter(|name| {
            build(name)
                .is_some_and(|card| is_land(&card) && !card.supertypes.contains(&Supertype::Basic))
        })
        .collect();

    // Nonbasic lands first: `NONBASIC_LANDS_PER_DECK` draws whatever `required`
    // holds, so the RNG stream does not depend on the flag. A required land goes
    // in ahead of them and takes its slot from the any-color fill at the end,
    // so the land count does not depend on it either.
    let mut lands_added = 0usize;
    for card in &required_lands {
        if lands_added < land_count {
            deck.push((*card).clone());
            lands_added += 1;
        }
    }
    if !nonbasic_names.is_empty() {
        for _ in 0..NONBASIC_LANDS_PER_DECK.min(land_count) {
            let name = nonbasic_names.choose(rng).unwrap();
            if let Some(card) = build(name) {
                deck.push(card);
                lands_added += 1;
            }
        }
    }

    // One basic of each type.
    let basics = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
    for i in 0..BASIC_LANDS_PER_DECK.min(land_count.saturating_sub(lands_added)) {
        if let Some(card) = build(color_to_land(basics[i % basics.len()])) {
            deck.push(card);
            lands_added += 1;
        }
    }

    // Everything else taps for any color.
    let any_color: Vec<&str> = nonbasic_names
        .iter()
        .copied()
        .filter(|name| build(name).is_some_and(|card| land_mana_colors(&card).len() == 5))
        .collect();
    let mut i = 0usize;
    while lands_added < land_count {
        let name = if any_color.is_empty() {
            color_to_land(basics[i % basics.len()])
        } else {
            any_color[i % any_color.len()]
        };
        if let Some(card) = build(name) {
            deck.push(card);
        }
        lands_added += 1;
        i += 1;
    }

    deck
}

/// The colors of mana a land's abilities can produce.
///
/// Reads printed abilities on purpose — this runs before the game exists, so
/// there is no object to compute effective characteristics for. Deck
/// construction is a `// PRE-LAYER ZONE:` concern in the same sense as
/// `oracle::legality`'s hand queries.
pub fn land_mana_colors(card: &CardData) -> Vec<Color> {
    let mut colors = Vec::new();
    for ability in card.abilities.iter() {
        let Effect::Atom(Primitive::ProduceMana(output), _) = &ability.effect else {
            continue;
        };
        for (mana_type, _) in &output.mana {
            if let Some(color) = mana_type_to_color(*mana_type)
                && !colors.contains(&color) {
                colors.push(color);
            }
        }
    }
    colors
}
