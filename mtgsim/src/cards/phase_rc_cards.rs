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
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    AffectedSet, AmountExpr, CounterType, Duration, Effect, EffectRecipient, PermanentFilter,
    PlayerRef, Primitive, SelectionFilter, Selector, TargetCount,
};
use crate::types::ids::new_ability_id;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{
    EnterMods, EventPattern, GameActionTemplate, ReplacementDef, Rewrite,
};
use crate::types::zones::{Zone, ZoneChangeCause};

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
                EventPattern::EnterBattlefield { cast: None },
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
                EventPattern::EnterBattlefield { cast: None },
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
                EventPattern::EnterBattlefield { cast: None },
                AffectedSet::SourceOnly,
                Rewrite::EnterWith(EnterMods::with_counters(CounterType::PlusOnePlusOne, 3)),
            ))),
        })
        .build()
}

/// Root Maze — {G}
/// Enchantment
///
/// Artifacts and lands enter tapped.
///
/// (Oracle text verified on Scryfall, 2026-09-02.)
///
/// # Why RC-3 ships a card at all
///
/// `engineering-practices.md` §3.3's axis, not a quota: **RC-3's own claim
/// needs no new card, and that is argued rather than assumed.** The gate change
/// widens a path the pool already walks — Blood Moon and Humility are both in
/// `PERFORMANCE_POOL`, both are filter-scoped layer effects, and RC-2 put two
/// permanents that enter modified in beside them. A tapland under Blood Moon
/// and a Scarecrow under Humility are reachable in a fuzz game today and
/// answered differently the moment the gate moves.
///
/// This card is for a **different** gap, and it is one RC-2 left by mistake.
/// RC-2 recorded that CR 616.1's multi-candidate branch was unreachable on an
/// entry until RC-3 opened the gate, on the grounds that no `AffectedSet::Filter`
/// effect could match an entering permanent. That was never true of the
/// *replacement* pipeline: `gather::set_affects` matches a `ReplacementDef`'s
/// filter through `GameState::permanent_matches_filter`, which has no
/// battlefield gate and never had one. The branch has been reachable since RB
/// merged, one registered card away, and RB left the identical gap with Kalitas
/// — a Legendary whose two copies each apply only to the other player's
/// creatures, so the ordering choice, the applied set across instances and CR
/// 101.4's APNAP ordering among simultaneous choosers were all dead code that
/// tested green.
///
/// **Two applicable effects on one entry, from a registered pool, is what this
/// card buys.** Root Maze plus [`idyllic_beachfront`] is exactly that: two
/// `EnterWith(tapped)` rewrites on one land, one from a filter on another
/// permanent and one `SourceOnly` from the land itself, so the pipeline asks
/// CR 616.1's question of a real player in a real game.
///
/// # Why this card out of the four
///
/// Kismet, Loxodon Gatekeeper and Frozen Aether all scope to "your opponents",
/// which is `PermanentFilter::ByController(PlayerRef::Opponent)` — and that
/// leaf reads `chars.controller`, which for an entering permanent comes from
/// `compute::base_controller`'s owner fallback. Right for a land drop, wrong
/// for a permanent spell cast by a non-owner, and unowned until this phase
/// (see `codebase-state.md`). Root Maze scopes by *type* and asks nothing about
/// controllers, so it exercises the multi-candidate branch without standing on
/// a read RC-3 is still fixing.
///
/// Its reachability is the other half. `random_deck` picks one or two colours
/// out of five, so a mono-green card is in about a third of decks; `{G}` at one
/// mana means it lands on turn one and is out before most permanents arrive.
/// It also matches **artifacts**, which is what puts it and [`chainbreaker`] on
/// the same board — a permanent whose entry both a filter and CR 122.6a modify.
///
/// # What it is not
///
/// Not a look-ahead test. Root Maze is on the battlefield before the entry it
/// modifies, so it is one of CR 614.12 clause (3)'s effects that "already
/// exist" and needs no frame. The self-exclusion half of CR 614.12 — Orb of
/// Dreams entering untapped under its own "Permanents enter tapped" — is
/// covered by a fixture in `tests/phase_rc_integration_test.rs`, because Root
/// Maze is an Enchantment and its own filter excludes it by type. A registered
/// card that matches *itself* would be the sharper board, and Orb of Dreams is
/// the card for it whenever the pool wants a second one.
pub fn root_maze() -> Arc<CardData> {
    CardDataBuilder::new("Root Maze")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text("Artifacts and lands enter tapped.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                AffectedSet::Filter {
                    filter: PermanentFilter::Or(
                        Box::new(PermanentFilter::ByType(CardType::Artifact)),
                        Box::new(PermanentFilter::ByType(CardType::Land)),
                    ),
                },
                Rewrite::EnterWith(EnterMods::tapped()),
            ))),
        })
        .build()
}

// ---------------------------------------------------------------------------
// Phase RC-4 — the look-ahead frame (CR 614.12)
// ---------------------------------------------------------------------------

/// Containment Priest — {1}{W}
/// Creature — Human Cleric, 2/2
///
/// Flash
/// If a nontoken creature would enter and it wasn't cast, exile it instead.
///
/// (Oracle text verified on Scryfall, 2026-09-02.)
///
/// # The card before the suppression — `replacement-architecture.md` §11 item 19
///
/// RC-4 stops asking CR 616.1's question when every member of the bucket is an
/// `EnterWith` (`pipeline::order_invariant_entry_bucket`): Root Maze beside
/// Idyllic Beachfront is a choice with one outcome, and the fuzz harness paid a
/// decision round-trip for it on every land drop. Suppressing that prompt with
/// nothing else registered would have returned the multi-candidate branch to
/// dead code in every fuzz game — the Kalitas gap a third time, caused by a
/// fix. This card is the member that is *not* an `EnterWith`: an `Instead`
/// beside one is a bucket the rule keeps asking about, because which CR 614.5
/// slot is spent first is a different event log even when the board is not.
///
/// # What it reads, and where
///
/// - **"wasn't cast"** is `EventPattern::EnterBattlefield { cast: Some(false) }`
///   — CR 601's fact projected off the entry's `ZoneChangeCause`. A creature
///   spell resolving arrives with `Resolved` and is not matched.
/// - **"nontoken creature"** is an `AffectedSet::Filter`, matched against
///   CR 614.12's frame: the object *as it would exist on the battlefield*. A
///   noncreature artifact returned under March of the Machines is a creature to
///   the Priest and is exiled — the clause (3) case RC-3's gate opened and this
///   card is the first replacement to read.
/// - **"exile it instead"** is `Instead(ZoneChangeTo { Exile })` on an entry.
///   The entry is the zone change (RC-4b), so the substitute is one move from
///   where the card is — its graveyard, or the hand for a Dryad Arbor played
///   as a land — and the card never becomes a permanent. The log holds that
///   one zone change and no entry.
///
/// # Reachability — why Dryad Arbor is registered beside it
///
/// No registered effect puts a creature card onto the battlefield without
/// casting it (`Primitive::ReturnToBattlefield` is a stub), and the card
/// excludes tokens. The one road a fuzz game has is a land drop, and a Land
/// Creature played as a land "wasn't cast": Containment Priest's ruling on
/// Dryad Arbor is exile. Both are in the stress pool and neither is in
/// `PERFORMANCE_POOL` — the path they open is the *branch*, not a cost, and
/// Keldon Warlord is RC-4's deliberate addition there.
pub fn containment_priest() -> Arc<CardData> {
    CardDataBuilder::new("Containment Priest")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(2, 2)
        .keyword(KeywordFlag::Flash)
        .rules_text(
            "Flash\nIf a nontoken creature would enter and it wasn't cast, exile it instead.",
        )
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: Some(false) },
                AffectedSet::Filter {
                    filter: PermanentFilter::And(
                        Box::new(PermanentFilter::ByType(CardType::Creature)),
                        Box::new(PermanentFilter::Not(Box::new(PermanentFilter::Token))),
                    ),
                },
                Rewrite::Instead(GameActionTemplate::ZoneChangeTo {
                    to: Zone::Exile,
                    cause: ZoneChangeCause::Exiled,
                }),
            ))),
        })
        .build()
}

/// Dryad Arbor
/// Land Creature — Forest Dryad, 1/1 (green by colour indicator, CR 204.1)
///
/// (This land isn't a spell, it's affected by summoning sickness, and it has
/// "{T}: Add {G}.")
///
/// (Oracle text verified on Scryfall, 2026-09-02. No mana cost, and no rules
/// text of its own: the parenthetical is reminder text, so `rules_text` is
/// left to `mana_ability_single`, which writes the intrinsic "{T}: Add {G}.")
///
/// # The land drop that "wasn't cast"
///
/// Registered for Containment Priest's reachability (see there), and it earns
/// the stress pool on its own: it is the only registered permanent with two
/// permanent types, so it is a creature to Doom Blade and Humility, a land to
/// Root Maze and Blood Moon, summoning-sick for its own mana ability (CR 302.6
/// — `Cost::Tap` asks, and the reminder text says so), and the land that
/// CR 305.7 turns into a Mountain *Dryad* rather than a Mountain — the
/// CR 205.1a case `land_types::apply_set_subtypes` got wrong until this card
/// made it reachable.
///
/// The mana ability is written out, as `basic_lands` and `dual_lands` write
/// theirs: CR 305.6 makes it intrinsic to the Forest type, and the layer walk
/// synthesizes the intrinsic one only when a land type is *gained*.
pub fn dryad_arbor() -> Arc<CardData> {
    CardDataBuilder::new("Dryad Arbor")
        .card_type(CardType::Land)
        .card_type(CardType::Creature)
        .subtype(Subtype::Land(LandType::Forest))
        .subtype(Subtype::Creature(CreatureType::Dryad))
        .color(Color::Green)
        .color_indicator(vec![Color::Green])
        .power_toughness(1, 1)
        .mana_ability_single(ManaType::Green)
        .build()
}

/// Keldon Warlord — {2}{R}{R}
/// Creature — Human Barbarian, */*
///
/// Keldon Warlord's power and toughness are each equal to the number of
/// non-Wall creatures you control.
///
/// (Oracle text verified on Scryfall, 2026-09-02.)
///
/// # §5a's probe — the count that does not see the entering object
///
/// A characteristic-defining ability (CR 604.3) whose value is a *count over
/// the battlefield*, which is the read `replacement-architecture.md` §5a's
/// enumeration row names and nothing in the layer walk had until RC-4:
/// `AmountExpr::CountOf` now has a static-context evaluator, and this is its
/// card. On the battlefield the Warlord counts itself. **In the CR 614.12
/// frame it does not** — the count runs over `battlefield_ids_ordered`, which
/// the entering object is not on. So a Warlord entering beside two other
/// creatures is 2/2 to a *replacement* that reads its power ("creatures with
/// power 2 or less enter tapped") and 3/3 to a *trigger* that does (Welcoming
/// Vampire): CR 603.10 checks a trigger against the board after the event,
/// where the Warlord is one of the creatures it counts. That is the pair of
/// Thassa rulings — for replacements "the mana symbols in its mana cost won't
/// be counted", for triggers the devotion "including the mana symbols in the
/// mana cost of the God itself" — with none of the devotion arithmetic and
/// none of Deferred Migrations item 7f in the way, which is why this card is
/// here and the God is not.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The evaluator is one frame per permanent per query — `layers-architecture.md`
/// §12's quadratic by design — and a card that opens a new engine path adds
/// itself to the measured pool (`engineering-practices.md` §3). Every walk of
/// the Warlord asks every permanent's frame at layer 7a's ceiling, memoized
/// within that walk, so `Frames/walk` is the fixture that moves; the
/// measurement is in RC-4's entry of `codebase-state.md`. Wall of Stone is in
/// the same pool, which is what makes "non-Wall" live.
pub fn keldon_warlord() -> Arc<CardData> {
    // Built once and cloned into the second slot: `SetPowerToughness` takes
    // its two amounts by value, and this runs at registry construction, not
    // in a game.
    let non_wall_creatures_you_control =
        AmountExpr::CountOf(Selector::PermanentsMatching(PermanentFilter::And(
            Box::new(PermanentFilter::And(
                Box::new(PermanentFilter::ByType(CardType::Creature)),
                Box::new(PermanentFilter::ByController(PlayerRef::You)),
            )),
            Box::new(PermanentFilter::Not(Box::new(PermanentFilter::BySubtype(
                Subtype::Creature(CreatureType::Wall),
            )))),
        )));
    CardDataBuilder::new("Keldon Warlord")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Barbarian))
        // `*/*` — the CDA supplies both numbers, in every zone (CR 208.2a).
        .power_toughness(0, 0)
        .rules_text(
            "Keldon Warlord's power and toughness are each equal to the number of \
             non-Wall creatures you control.",
        )
        .ability(AbilityDef {
            is_characteristic_defining: true,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::SetPowerToughness(
                    non_wall_creatures_you_control.clone(),
                    non_wall_creatures_you_control,
                    Duration::WhileSourceOnBattlefield,
                ),
                EffectRecipient::Implicit,
            ),
        })
        .build()
}
