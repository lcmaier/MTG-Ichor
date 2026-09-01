//! Cards for Phase RC — replacement effects that modify how a permanent enters
//! the battlefield (CR 614.1c/d).
//!
//! **Two cards on CR 614.1c's own axis, and a third for the ordering.** A
//! replacement that modifies an entry can change the permanent's *status*
//! (CR 110.5b) or give it *counters* (CR 122.6a), and those are two fields of
//! `EnterMods` with two separate performer paths behind them — one writes
//! `BattlefieldEntity.tapped` before anything can look, the other allocates a
//! CR 613.7c timestamp per kind and puts counters on. A phase that shipped only
//! the first would leave the second exactly as reachable as CR 616.1's
//! multi-candidate branch was after RB shipped Kalitas alone: covered by a test,
//! unreachable in a game (`engineering-practices.md` §3.3).
//!
//! **The third is [`adaptive_shimmerer`], and it is §3.3's own finding applied
//! to this phase.** "A bespoke fixture can cover an atom while the registered
//! pool cannot build the same scenario." The claim at stake is that CR 122.6a's
//! counters go on *before* anything can observe the permanent — and the only
//! board state that can falsify it is a 0/0 that CR 704.5f would kill without
//! them. A fixture made that claim; a registered card is what makes the *pool*
//! able to.
//!
//! None is gated behind a colour a random deck may not have: `random_deck`
//! filters nonlands by colour, so a `{G}{W}` card reaches roughly one deck in
//! sixteen, while a colorless card is in every deck and a two-type nonbasic land
//! is in every deck that wants either of its colours. Reachability is the
//! measurement this phase owes, so it is a selection criterion and not a
//! footnote.
//!
//! # What is *not* reachable — corrected 2026-09-02, and the correction is
//! the finding
//!
//! **RC-2 recorded that no `AffectedSet::Filter` effect reaches an entering
//! permanent. That is false, and it was never true.** There are two filter
//! paths and they are different functions:
//!
//! - `compute.rs::effect_applies_to` gates on `game.battlefield.contains_key`
//!   and governs **`ContinuousEffect`** — the layer registry. That is the gate
//!   RC-3 removes, and Blood Moon / Humility / Dress Down are the effects
//!   behind it.
//! - `gather::set_affects` → `GameState::permanent_matches_filter` governs
//!   **`ReplacementDef.affected`**, and it has **no gate at all**.
//!
//! So a Root-Maze-shaped `ReplacementDef` (`Filter { ByType(Land) }`,
//! `EnterWith(tapped)`) already taps an entering land, and with
//! [`idyllic_beachfront`] entering under it the pipeline produces two
//! candidates and asks CR 616.1's question. **The multi-candidate branch was
//! reachable the day RC-2 merged**; Root Maze, Kismet, Loxodon Gatekeeper and
//! Frozen Aether were not blocked and never were. `root_maze` is the card
//! that closes it, in RC-3 rather than a third phase from now.
//!
//! What is genuinely out of reach at `AffectedSet::SourceOnly` is **two
//! entry-modifying abilities on one card** — the whole printed population is
//! Slumbering Trudge, Chocobo Camp, Steel Dromedary, Rotating Fireplace and
//! Arixmethes (Scryfall, `o:/enters tapped./ o:/enters with/` plus the counter
//! searches beside it), and every one needs {X}, a condition, or a triggered
//! ability this engine does not have yet. That was one of RC-2's two routes to
//! the branch; the mistake was ruling out the other.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, LandType, Subtype};
use crate::types::costs::Cost;
use crate::types::effects::{
    AffectedSet, AmountExpr, CounterType, Effect, EffectRecipient, PermanentFilter, Primitive,
    SelectionFilter, TargetCount,
};
use crate::types::ids::new_ability_id;
use crate::types::keywords::KeywordFlag;
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
/// **It does not, and this card is what found out.** Blood Moon's row is a
/// `ContinuousEffect` whose `AffectedSet` is a `Filter`, and
/// `compute.rs::effect_applies_to` returns `false` for a filter effect against
/// an object that is not on the battlefield — so the *layer registry* reaches
/// no entering permanent. That gate is Phase **RC-3**'s one line, and until it
/// moves the strip is real everywhere except at the instant it matters here.
///
/// **The gate is not "no filter reaches an entry", which is what RC-2 wrote
/// (corrected 2026-09-02).** `ReplacementDef.affected` is matched by
/// `gather::set_affects` through a different and ungated function, so
/// `root_maze` taps this land on the way in without any help from RC-3. The
/// two paths are separated in this module's doc comment.
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
/// # The second shape, and what it reaches that a tapland cannot
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
/// # What it is *not* the card for
///
/// A 0/0 that enters with +1/+1 counters is the sharper test of the ordering
/// inside the performer, and it is [`adaptive_shimmerer`]'s job rather than a
/// reason to prefer one over the other. Chainbreaker earns its place on a
/// different axis: it is `{2}` rather than `{5}`, so a random game casts it far
/// more often, and its ability is the only thing in either pool that *spends* a
/// counter a permanent entered with.
///
/// **The -1/-1 sign is not decoration either.** Layer 7c reads both, and a
/// board where a permanent's printed and effective P/T disagree *downward* is
/// one no other registered card produces.
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

/// Adaptive Shimmerer — {5}
/// Creature — Insect, 0/0
///
/// Flash
/// This creature enters with three +1/+1 counters on it.
///
/// (Oracle text verified on Scryfall, 2026-09-01.)
///
/// # A 0/0 is the only board state that can falsify the ordering
///
/// CR 122.6a's counters go on inside the performer, before the entry is
/// announced and long before state-based actions next run. That ordering is a
/// claim, and every *other* card in either pool would pass whether it held or
/// not — a 3/3 that briefly reads 3/3 looks exactly like a 3/3 that reads 1/1.
/// **This one dies if the counters arrive a moment late**: a 0/0 is what
/// CR 704.5f is written about.
///
/// **Registered for `engineering-practices.md` §3.3's sharpest finding**, which
/// is not about counts: *a bespoke fixture can cover an atom while the
/// registered pool cannot build the same scenario*. The ordering was asserted
/// against a hand-built 0/0 before this card existed, which proved the engine
/// and proved nothing about the pool. Colorless, so `random_deck` puts it in
/// every deck in every game.
///
/// **Not in `PERFORMANCE_POOL`.** Registering a card and adding one to that pool
/// are different acts (§3), and RC-2's new engine path is already measured by
/// the two cards that are in it. This one grows the stress pool, which is read
/// as a threshold rather than a baseline.
///
/// Flash is a printed keyword the engine already enforces (`engine/cast.rs`
/// reads `KeywordFlag::Flash` for timing), so the card ships whole rather than
/// trimmed — which matters here, because a flashed-in 0/0 enters during an
/// opponent's turn and reaches the ordering from a second direction.
pub fn adaptive_shimmerer() -> Arc<CardData> {
    CardDataBuilder::new("Adaptive Shimmerer")
        .mana_cost(ManaCost::build(&[], 5))
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Insect))
        .power_toughness(0, 0)
        .keyword(KeywordFlag::Flash)
        .rules_text("Flash\nThis creature enters with three +1/+1 counters on it.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef::new(
                EventPattern::EnterBattlefield,
                AffectedSet::SourceOnly,
                Rewrite::EnterWith(EnterMods::with_counters(CounterType::PlusOnePlusOne, 3)),
            ))),
        })
        .build()
}
