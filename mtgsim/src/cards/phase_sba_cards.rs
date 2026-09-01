//! Cards that make a state-based action reachable in a `fuzz_games` run.
//!
//! Every SBA in `engine/sba.rs` has a unit test. That is a different
//! measurement from whether any *registered* card can build the scenario, and
//! `engineering-practices.md` §3.3 is where the two came apart: "a bespoke
//! fixture can cover an atom while the registered pool cannot build the same
//! scenario, so `specdb` coverage and fuzz reachability are different
//! measurements and neither implies the other."
//!
//! This file is that gap closed one rule at a time. The entry bar is the one
//! §3.3 sets — **name the blocker before reaching for a card.** An SBA whose
//! blocker is a missing subsystem does not get a card here; it gets a line in
//! the audit saying which subsystem, because "adding a card buys errors rather
//! than coverage".
//!
//! # What the 2026-09-01 audit found, and why the file is one card long
//!
//! Six SBA/token paths were named as fuzz-unreachable. Measured against 200
//! stress games at seed 12345 with `--dump-events`, **two of the six were
//! already reachable** — the claim behind them dated from 2026-08-26, when
//! `Primitive::CreateToken` was a stub and combat could not yet produce a
//! multi-member batch. Of the four that really do measure zero, three are
//! blocked on a subsystem rather than on card selection:
//!
//! | SBA | Measured | Blocker |
//! |---|---|---|
//! | 704.5q counter annihilation | 0 | **none** — this file |
//! | 704.5m/n Aura | 0 | CR 608.3b, *and* no `AffectedSet` reaches an Aura's host |
//! | 704.5p Equipment detach | 0 | Equip (CR 702.6); nothing can attach an Equipment |
//! | 704.5i planeswalker death | 0 | loyalty abilities + CR 120.3c |
//!
//! `plans/codebase-state.md` carries the counts and the reasoning.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, CounterType, Effect, EffectRecipient, PermanentFilter, Primitive, SelectionFilter,
    TargetCount,
};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};

/// Battlegrowth — {G}
/// Instant
///
/// Put a +1/+1 counter on target creature.
///
/// (Oracle text verified on Scryfall, 2026-09-01. Mirrodin.)
///
/// # Why this one out of 72
///
/// Scryfall lists 72 instants and sorceries containing "put a +1/+1 counter on
/// target creature", and Battlegrowth is the **only one whose entire oracle
/// text is that sentence**. Every other one carries a rider the engine cannot
/// express yet — a keyword until end of turn, morbid, flashback, a mode, a
/// second target. So the card adds one rules line and no machinery, which is
/// the same bar [`crate::cards::phase_rc_cards::idyllic_beachfront`] was picked
/// against.
///
/// # What it makes reachable, and why it takes two cards to do it
///
/// CR 704.5q is defined over **one permanent holding two counter kinds**, so
/// what it needs is a source of each sign that can meet on one object.
/// [`crate::cards::phase_rc_cards::chainbreaker`] is already the -1/-1 half: it
/// is colorless, so `random_deck` puts it in every deck, and CR 122.6a gives it
/// two -1/-1 counters on entry. Nothing in either pool produced a +1/+1
/// counter, so the annihilation sweep had never run in a game — 0 occurrences
/// across 200 stress games, measured before this card and after.
///
/// **It targets any creature, not "a creature you control".** That is the whole
/// reason the pairing works: Chainbreaker is as likely to be across the table
/// as under it, and a controller-scoped counter spell would reach it only half
/// the time. The 72-card list has both shapes and this is the unrestricted one.
///
/// # The engine path it opens, which is why it is in `PERFORMANCE_POOL`
///
/// `Primitive::AddCounters` had **no reader anywhere** — not in a registered
/// card, not in a test. Chainbreaker's counters arrive through `EnterMods` and
/// `GameState::add_counters`, which is a direct write inside the entry
/// performer and deliberately not a proposal (CR 122.6a). So
/// `GameAction::AddCounters` reached `perform_action` from nothing, and
/// `gather`'s `EventSubject::Object` arm for it was dead. This card is the
/// first thing in the crate to propose one, which is a new engine path and so
/// carries the deliberate `PERFORMANCE_POOL` addition §3 asks for.
///
/// **`{G}` and not colorless, accepted knowingly.** `random_deck` gives green
/// to about a third of decks, so roughly half of two-player games have a
/// Battlegrowth in them at all — a real cost against a colorless card, and
/// there is no colorless printing of this shape to pick instead. It is paid
/// once and buys the exact text; the measured annihilation count is in
/// `codebase-state.md` and is what says the trade was worth making.
pub fn battlegrowth() -> Arc<CardData> {
    CardDataBuilder::new("Battlegrowth")
        .card_type(CardType::Instant)
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .rules_text("Put a +1/+1 counter on target creature.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::AddCounters(CounterType::PlusOnePlusOne, AmountExpr::Fixed(1)),
                EffectRecipient::Target(
                    SelectionFilter::Permanent(PermanentFilter::ByType(CardType::Creature)),
                    TargetCount::Exactly(1),
                ),
            ),
        })
        .build()
}
