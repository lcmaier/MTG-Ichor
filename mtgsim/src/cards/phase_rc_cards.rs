//! Cards for Phase RC — replacement effects that modify how a permanent enters
//! the battlefield (CR 614.1c/d).
//!
//! **Two cards, and the axis they differ on is CR 614.1c's own.** A replacement
//! that modifies an entry can change the permanent's *status* (CR 110.5b) or
//! give it *counters* (CR 122.6a), and those are two fields of `EnterMods` with
//! two separate performers behind them — one writes `BattlefieldEntity.tapped`
//! before anything can look, the other allocates a CR 613.7c timestamp per kind
//! and puts counters on. A phase that shipped only the first would leave the
//! second exactly as reachable as CR 616.1's multi-candidate branch was after
//! RB shipped Kalitas alone: covered by a test, unreachable in a game
//! (`engineering-practices.md` §3.3).
//!
//! Neither is gated behind a colour a random deck may not have: `random_deck`
//! filters nonlands by colour, so a `{G}{W}` card reaches roughly one deck in
//! sixteen, while a colorless artifact is in every deck and a two-type nonbasic
//! land is in every deck that wants either of its colours. Reachability is the
//! measurement this phase owes, so it is a selection criterion and not a
//! footnote.
//!
//! # What is *not* reachable, measured rather than assumed (2026-09-01)
//!
//! Neither card makes **CR 616.1's multi-candidate branch** reachable on an
//! entry, and no printed card can while RC-2's consumers are limited to
//! `AffectedSet::SourceOnly`. Two applicable effects on one entry needs either
//!
//! - **two entry-modifying abilities on one card** — the whole printed
//!   population is Slumbering Trudge, Chocobo Camp, Steel Dromedary, Rotating
//!   Fireplace and Arixmethes (Scryfall, `o:/enters tapped./ o:/enters with/`
//!   plus the counter searches beside it), and every one of them needs {X},
//!   a condition, or a triggered ability this engine does not have yet; or
//! - **an `AffectedSet::Filter` effect** — Root Maze, Kismet, Loxodon
//!   Gatekeeper, Frozen Aether — which cannot match an object that is not on
//!   the battlefield until Phase **RC-3** removes `compute.rs`'s battlefield
//!   gate.
//!
//! So the branch belongs to RC-3, and that is a finding about RC-3 rather than
//! an excuse for RC-2.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, LandType, Subtype};
use crate::types::costs::Cost;
use crate::types::effects::{
    AffectedSet, AmountExpr, CounterType, Effect, EffectRecipient, PermanentFilter, Primitive,
    SelectionFilter, TargetCount,
};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EnterMods, EventPattern, ReplacementDef, Rewrite};

/// Idyllic Beachfront — Land — Plains Island
///
/// ({T}: Add {W} or {U}.)
/// This land enters tapped.
///
/// (Oracle text verified on Scryfall, 2026-09-01. Dominaria United; the
/// parenthesis is the whole of the mana half, because CR 305.6 makes it
/// intrinsic to the land types.)
///
/// # Why this one out of 773
///
/// CR 110.5b's shape — "permanents enter the battlefield untapped … unless a
/// spell or ability says otherwise" — is the most-printed replacement effect in
/// Magic, and nearly every card carrying it carries something else too: an ETB
/// trigger, a sacrifice ability, cycling, a storage counter. This cycle is the
/// clause and nothing else, and its mana is `dual_lands.rs`'s shape exactly —
/// two basic land types, two intrinsic abilities, and the same modelling
/// shortcut documented there. So the card adds one rules line and no machinery.
///
/// **It is also a land, and that is the point of picking a land.** A land drop
/// is the highest-frequency entry in the game and it reaches the battlefield by
/// a different road than a resolving permanent spell — `play_land` rather than
/// `resolve_top_of_stack`, with `GameState::resolving` empty, so CR 110.2b's
/// default controller comes from the owner rather than from the stack entry.
///
/// **`AffectedSet::SourceOnly`, which is CR 614.12's first sentence** — "such
/// effects may come from the permanent itself if they affect only that
/// permanent (as opposed to a general subset of permanents that includes it)".
/// That is what lets RC-2 ship without the look-ahead frame: the effect is found
/// on the entering object, applies to the entering object, and asks nothing
/// about the board.
///
/// # The interaction that makes it worth registering rather than fixturing —
/// and the one it exposes
///
/// Blood Moon is in `PERFORMANCE_POOL`, and CR 305.7 clears a nonbasic land's
/// abilities wholesale, "enters tapped" included. The real ruling is that a
/// tapland under Blood Moon enters **untapped**, and `gather` reading the
/// *effective* ability list is meant to deliver that for free
/// (`replacement-architecture.md` §3.3).
///
/// **It does not, and this card is what found out.** Blood Moon's row is an
/// `AffectedSet::Filter`, and `effect_applies_to` returns `false` for a filter
/// effect against an object that is not on the battlefield — so no `Filter`
/// effect reaches an entry at all. That gate is Phase **RC-3**'s one line, and
/// until it moves the strip is real everywhere except at the instant it
/// matters here. `tests/phase_rc_integration_test.rs` asserts the wrong answer
/// on purpose so RC-3 has to flip it.
pub fn idyllic_beachfront() -> Arc<CardData> {
    CardDataBuilder::new("Idyllic Beachfront")
        .card_type(CardType::Land)
        .subtype(Subtype::Land(LandType::Plains))
        .subtype(Subtype::Land(LandType::Island))
        .rules_text("This land enters tapped.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef::new(
                EventPattern::EnterBattlefield,
                AffectedSet::SourceOnly,
                Rewrite::EnterWith(EnterMods::tapped()),
            ))),
        })
        // CR 305.6 — intrinsic to the land types, modelled as two explicit mana
        // abilities for `dual_lands.rs`'s documented reason.
        .mana_ability_single(ManaType::White)
        .mana_ability_single(ManaType::Blue)
        .build()
}

/// Chainbreaker — {2}
/// Artifact Creature — Scarecrow, 3/3
///
/// This creature enters with two -1/-1 counters on it.
/// {3}, {T}: Remove a -1/-1 counter from target creature.
///
/// (Oracle text verified on Scryfall, 2026-09-01.)
///
/// # The second shape, and what it reaches that Charcoal Diamond cannot
///
/// CR 122.6a — "an object that's given counters as it enters the battlefield" —
/// is the other half of `EnterMods`, and it is not the same code path: the
/// counters go on inside the performer, before the entry is announced and
/// before any state-based action can look. That ordering is the claim, and a
/// 3/3 that arrives as a 1/1 is how a game shows it.
///
/// The activated ability is what keeps the counters from being decoration. It
/// costs `{3}, {T}` — both cost primitives the engine actually implements — and
/// removes a counter from **target creature**, which in a random game means the
/// board contains a permanent whose printed P/T and effective P/T disagree, and
/// then stops disagreeing. Nothing else in either pool produces a -1/-1
/// counter at all, and being colorless it is in every deck in every game.
///
/// # Why not a 0/0 with +1/+1 counters
///
/// It would be the sharper test — Faithful Watchdog enters as a 0/0 and only
/// the ordering inside the performer keeps CR 704.5f from killing it — and it
/// is `{G}{W}`, which `random_deck` puts in about one deck in sixteen. The ordering claim is made in the
/// integration test with exactly that fixture, unregistered and labelled as
/// one; the *registered* card is the one that plays in every game.
pub fn chainbreaker() -> Arc<CardData> {
    CardDataBuilder::new("Chainbreaker")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Scarecrow))
        .power_toughness(3, 3)
        .rules_text(
            "This creature enters with two -1/-1 counters on it.\n\
             {3}, {T}: Remove a -1/-1 counter from target creature.",
        )
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef::new(
                EventPattern::EnterBattlefield,
                AffectedSet::SourceOnly,
                Rewrite::EnterWith(EnterMods::with_counters(CounterType::MinusOneMinusOne, 2)),
            ))),
        })
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::Mana(ManaCost::build(&[], 3)), Cost::Tap],
            effect: Effect::Atom(
                Primitive::RemoveCounters(CounterType::MinusOneMinusOne, AmountExpr::Fixed(1)),
                EffectRecipient::Target(
                    SelectionFilter::Permanent(PermanentFilter::ByType(CardType::Creature)),
                    TargetCount::Exactly(1),
                ),
            ),
        })
        .build()
}
