//! Cards for Phase RD — damage (CR 614.5, 615, 701.10g, 120.3).
//!
//! # RD-1 — the damage event's two subjects and its results
//!
//! **Four printed cards on two axes, and a fixture for the third result.**
//! `replacement-architecture.md` §9's RD-1 section names them; what follows is
//! why each is here rather than a cheaper one.
//!
//! The two axes are the ones the phase builds. *Which subject* an effect is
//! about — an object, a player, or (Furnace of Rath) both — and *what it does
//! to the amount*: multiply, halve, or prevent half. Every card below is a
//! distinct pair of those, and none of them is a second copy of another's
//! engine path (`engineering-practices.md` §3.3, tier 2):
//!
//! | Card | Subject | Amount |
//! |---|---|---|
//! | [`furnace_of_rath`] | every object **and** every player | `Multiplier(2)` |
//! | [`ghosts_of_the_innocent`] | the same | `Halve(Down)` |
//! | [`gisela_blade_of_goldnight`] | opponents' / yours, two rows on one card | `Multiplier(2)` and `PreventHalf(Up)` |
//! | [`angel_of_suffering`] | **you** and no object at all | whole-event `Prevent`, with a rider |
//!
//! **Two Furnaces are what CR 614.5's own example needs**, and the card is not
//! legendary, so the registered pool can build the board — which
//! `ATOM-614.5-001` had never had (§3.3's tier-1 finding, applied here).
//! Ghosts beside Furnace is the first **non-commuting** CR 616.1 choice a fuzz
//! game can reach: 3 damage becomes 1 then 2, or 6 then 3, and the affected
//! player picks. Dictate of the Twin Gods was the first draft's second doubler
//! and is dropped — same shape as Furnace, and two Furnaces already give the
//! commuting pair.
//!
//! Gisela is the card that needs [`PlayerSet`] twice on one object, in both of
//! its non-trivial arms, and both halves of `Rounding` are on the same
//! battlefield as her doubler. Angel of Suffering is the only prevention
//! consumer here and the only rider that rides on a *player* subject
//! (`codebase-state.md` item 27) — its "twice that many cards" is the whole
//! reason `AmountExpr::ReplacedAmount` and `Multiply` exist.
//!
//! # The fixture, and why it is a genuine consumer
//!
//! [`loyalty_probe`] is not a printed card. CR 120.3c — damage to a
//! planeswalker removes that many loyalty counters — needs a planeswalker on
//! the battlefield, and registering a *printed* one fails
//! `register-a-card-only-once-the-engine-can-play-it`: loyalty abilities have
//! no `AbilityType` and no activation path, so a real Jace would be a card with
//! three dead abilities wearing a real name, which `engineering-practices.md`
//! §3 forbids. Leaving the arm out instead would leave `perform_action` marking
//! damage on a permanent the targeting code already validates as "any target",
//! which is the wrong answer waiting for a card.
//!
//! So the fixture is the middle, on the `graveyard_probe` convention §3.3
//! already admits — and it is a consumer rather than a test prop.
//! `SelectionFilter::Any` enumerates planeswalkers, so the random agent bolts
//! it; three damage is exactly its printed loyalty, so **CR 704.5i becomes
//! reachable from a game** for the first time (`phase_sba_cards` measured it
//! at 0 across 200 stress games). It is registered in the stress pool and
//! stays out of `PERFORMANCE_POOL`.
//!
//! # What a random deck can draw, and why it is no longer about color
//!
//! **`fuzz_games::random_deck` has not filtered nonlands by color since the
//! `Everywhere` land landed (2026-09-03).** It draws 36 nonlands uniformly from
//! every registered nonland, gives every deck one basic of each of the five
//! types, and fills the rest of the mana base with a land that taps for any
//! color — so every deck can cast anything, and a gold card is exactly as
//! likely to be drawn as a mono-colored one. Earlier card files say otherwise
//! and are stale; `registry.rs`'s `Everywhere` note is the change.
//!
//! What still separates these four is **mana value**, which decides whether a
//! drawn card is ever cast. Furnace of Rath is the cheapest at four and is the
//! pooled one; Ghosts of the Innocent is seven, Gisela six, Angel of Suffering
//! five, and all three are registered for the stress pool, where the point is
//! breadth rather than frequency.
//!
//! # RD-2 — CR 615.7 prevention shields, and the loop's unit
//!
//! **Four printed cards, one per shape a resolution-created prevention effect
//! takes** (§9's RD-2 section; `engineering-practices.md` §3.3, tier 2). Every
//! one of them is a [`Primitive::CreateReplacement`] — the durational form
//! `Effect::Replacement` refused to be — and they differ in what the
//! resolution fills in and where the count lives:
//!
//! | Card | Recipient | Rows at resolution | Count |
//! |---|---|---|---|
//! | [`mending_hands`] | `Target(Any)` | one, on the object or player it targeted | `NextDamage(4)` |
//! | [`samite_healer`] | `Target(Any)`, from a `{T}` ability | one per activation | `NextDamage(1)` |
//! | [`safe_passage`] | `Implicit` | one, `Filter` + `You`, asked at each event | none — `Uses::Static` |
//! | [`samite_censer_bearer`] | `FilteredPermanents` | one **per creature** you control then (CR 615.11) | `NextDamage(1)` each |
//!
//! Mending Hands is the pooled one: the first registry row a damage event in
//! the pool meets, and the first CR 615.7 `allocate` prompt a fuzz game can
//! reach — two attackers into a shielded player is a board every combat step
//! builds. Healing Salve was the canonical printing and is modal, which
//! `Effect::Modal` cannot resolve yet, so its plain sibling ships instead. The
//! other three are registered for the stress pool: Samite Healer is the
//! repeatable source of rows the random agent will activate, Safe Passage is
//! the set evaluated at the event rather than fixed at resolution (its rulings
//! say so from both sides), and Samite Censer-Bearer is CR 615.11's own text.

//! # RD-3 — sources
//!
//! **Eight printed cards, and the axis is CR 609.7's source predicate: which
//! half of it each card writes, and what it does once it matches** (§9's RD-3
//! section; `engineering-practices.md` §3.3, tier 2).
//!
//! | Card | `source` | `combat` | Rewrite |
//! |---|---|---|---|
//! | [`circle_of_protection_red`] | chosen **and** a colour | — | whole-event `Prevent`, `Uses::Once` |
//! | [`reverse_damage`] | chosen, no property | — | `Prevent` with a rider that reads the prevented amount |
//! | [`dark_sphere`] | chosen, no property | — | `PreventHalf(Down)` from a registry row |
//! | [`guardian_seraph`] | a controller, nothing chosen | — | `PreventUpTo(1)` on a player |
//! | [`daunting_defender`] | none — any source | — | `PreventUpTo(1)` on a filtered object set |
//! | [`fog`] | none | `Some(true)` | `Prevent`, every permanent and every player |
//! | [`torbran_thane_of_red_fell`] | colour **and** controller | — | `Plus(2)` |
//! | [`pyroclasm`] | — (it is a *source*, not a watcher) | — | — |
//!
//! Pyroclasm is the odd row and belongs there: CR 615.10's example is a
//! two-card board, and Daunting Defender is only half of it. It is also the
//! pool's first "each creature" damage, which is one `execute_actions` batch
//! rather than a loop — the difference CR 704.3 and CR 615.7 are written
//! against.
//!
//! **Guardian Seraph is the pooled card**: the first source in
//! `PERFORMANCE_POOL` whose gather evaluates an `ObjectFilter` on the damage's
//! *source* rather than on its target. Circle of Protection: Red is registered
//! for the stress pool and its activation count read, because a `{1}`
//! activation competes for mana the random agent rarely has.
//!
//! **Sokrates, Athenian Teacher is recorded and not written.** Its granted "If
//! this creature would deal combat damage to a player, prevent that damage.
//! This creature's controller and that player each draw half that many cards,
//! rounded down" is a Layer 6 grant of a `Prevent` whose `SourcePattern` names
//! its own host — a `Self` leaf with exactly one customer, which §8c's guard
//! says to record rather than add. The card is unregisterable anyway until
//! RS-2 lands "hexproof as long as it's untapped", and a registered card with
//! a dead ability under a real name is what `engineering-practices.md` §3
//! forbids.
//!

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::ids::new_ability_id;
use crate::types::effects::{
    AffectedSet, AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, PatternFill,
    PlayerRef, PlayerSet, Primitive, SelectionFilter, TargetCount,
};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{
    AmountRewrite, EventPattern, ReplacementDef, Rewrite, Rounding, SourcePattern,
};

/// The static ability wrapper every card in this file uses.
///
/// Local rather than shared with `test_support::static_ability`: that one is a
/// test helper, and a card file must not depend on one. It is the **second**
/// named helper of this shape in `src/cards/` (`phase_li_cards::static_ability`
/// is the first) and the shape is written out inline **31** more times, which
/// is the argument for the hoist `codebase-state.md` "Before card breadth"
/// item 10 now records: these belong in a `cards::helpers` module beside the
/// real card list, not duplicated per phase file and not borrowed from
/// `test_support`.
fn static_replacement(def: ReplacementDef) -> AbilityDef {
    AbilityDef {
        id: new_ability_id(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect: Effect::Replacement(Box::new(def)),
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
    }
}

/// "A permanent or player" — CR 614.1's two kinds of subject, as the pair of
/// sets that says "everything".
///
/// Furnace of Rath and Ghosts of the Innocent print the same scope word for
/// word, so they share this rather than each spelling it: a divergence between
/// two cards whose oracle text is identical would be an authoring bug nothing
/// could catch.
fn every_permanent_or_player() -> (AffectedSet, PlayerSet) {
    (AffectedSet::Filter { filter: ObjectFilter::All }, PlayerSet::Everyone)
}

/// Furnace of Rath — {1}{R}{R}{R}
///
/// > If a source would deal damage to a permanent or player, it deals double
/// > that damage to that permanent or player instead.
///
/// CR 701.10g's doubling, and the first `Rewrite::Amount` in the crate.
///
/// **The pooled card of the phase.** It is the first static `DealDamage`
/// source in `PERFORMANCE_POOL`, so it opens the gather sweep on every damage
/// event while it is on the battlefield — a new engine path, and the one this
/// PR measures. Mono-red at four mana, and *not legendary*, which is what lets
/// a random deck put two of them down: CR 614.5's own example ("if you have
/// two of these on the battlefield, the damage is multiplied by 4") needs two
/// instances, and `ATOM-614.5-001` had never had a registered board that could
/// build one.
///
/// # The rulings, and where each is tested
///
/// - *Two of these multiply by 4* → `tests/phase_rd_integration_test.rs`, and
///   it is CR 616.1's multi-candidate prompt from two **printed** cards.
/// - *The multiplied damage counts in all ways as if it came from the original
///   source; Furnace of Rath is not the source* → asserted on
///   `GameEvent::DamageDealt`'s `source_id`, which the rewrite does not touch.
/// - *If a spell or ability damages multiple things, divide up the damage
///   before applying this effect* and *the trample rules cause damage to be
///   divided before it is doubled* → structurally true rather than coded:
///   `assign_combat_damage` divides before anything is proposed, and the
///   pipeline never sees the undivided number. Asserted with War Mammoth,
///   which is already in the pool.
/// - *Prevent 4 then double the remaining 1, or double to 10 then prevent 4* →
///   **RD-2's**, when Mending Hands exists to be the prevention half. Its
///   ordering half is covered here by Ghosts of the Innocent, which is the same
///   CR 616.1 question with two rewrites this PR does build.
pub fn furnace_of_rath() -> Arc<CardData> {
    let (affected, players) = every_permanent_or_player();
    CardDataBuilder::new("Furnace of Rath")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red, ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Enchantment)
        .rules_text(
            "If a source would deal damage to a permanent or player, it deals double that \
             damage to that permanent or player instead.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                affected,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(players),
        ))
        .build()
}

/// Ghosts of the Innocent — {5}{W}{W}
///
/// > If a source would deal damage to a permanent or player, it deals half
/// > that damage, rounded down, to that permanent or player instead.
///
/// The printed **inverse** of a doubler, and the reason `Rounding` exists:
/// CR 107.1a puts the direction on the card, and this one says "rounded down"
/// where Gisela says "rounded up".
///
/// # The rulings, and where each is tested
///
/// - *Half of 1 rounded down is 0. A source that would deal 1 damage won't
///   deal damage at all* → the rewrite leaves a 0-damage proposal and
///   `never_happens` drops it on the next iteration (CR 614.7a).
/// - *Multiple effects are cumulative … with three on the battlefield, 14
///   damage becomes 7, then 3, then finally 1* → three instances, each applied
///   once (CR 614.5).
/// - *This isn't a damage prevention effect. If Excruciator … would deal 7
///   damage, it deals 3 instead* → the reason [`AmountRewrite::Halve`] is a
///   separate arm from `PreventHalf`, which CR 615.12 would stop. Excruciator's
///   half is RD-4's, when a damage event can be unpreventable.
/// - *If damage is redirected, it's only halved once* → **RD-4's**: CR 614.5's
///   applied set survives a `Retarget`, and there is no `Retarget` yet.
/// - *If both Ghosts of the Innocent and Furnace of Rath are on the
///   battlefield, the controller of the permanent being dealt damage or the
///   player being dealt damage can apply the effects in either order. This can
///   matter if the original amount of damage is odd* → the phase's sharpest
///   test, and the first non-commuting CR 616.1 choice reachable from two
///   printed statics.
/// - *If a damage prevention effect and this effect would apply to the same
///   damage, the player … may apply the effects in either order* → the same
///   question against a prevention, which Gisela's other half supplies.
pub fn ghosts_of_the_innocent() -> Arc<CardData> {
    let (affected, players) = every_permanent_or_player();
    CardDataBuilder::new("Ghosts of the Innocent")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 5))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Spirit))
        .power_toughness(4, 5)
        .rules_text(
            "If a source would deal damage to a permanent or player, it deals half that \
             damage, rounded down, to that permanent or player instead.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                affected,
                Rewrite::Amount(AmountRewrite::Halve(Rounding::Down)),
            )
            .affecting_players(players),
        ))
        .build()
}

/// Gisela, Blade of Goldnight — {4}{R}{W}{W}
///
/// > Flying, first strike
/// >
/// > If a source would deal damage to an opponent or a permanent an opponent
/// > controls, that source deals double that damage to that player or permanent
/// > instead.
/// >
/// > If a source would deal damage to you or a permanent you control, prevent
/// > half that damage, rounded up.
///
/// **Two statics on one card, and each is a `Filter` beside a [`PlayerSet`].**
/// That is decision 0's shape used in both of its non-trivial arms — CR 109.5's
/// "you" and CR 102.1's "an opponent", resolved against the ability's current
/// controller on every gather — and both halves of `Rounding` on one
/// battlefield.
///
/// # The rulings, and where each is tested
///
/// - *Gisela doubles damage dealt to opponents and permanents your opponents
///   control from any source, **including sources controlled by those
///   opponents*** → the pattern carries no source-side constraint, so this is
///   true by construction; asserted anyway, because a `SourcePattern` arrives
///   in RD-3 and this is the assertion that would fail if one were added here
///   by mistake.
/// - *If multiple replacement effects would modify how damage would be dealt,
///   the player being dealt damage (or the controller of the permanent) chooses
///   the order* → the Ghosts/Furnace test generalised to a prevention:
///   Gisela's half beside an opponent's Furnace on 5 damage is prevent-3-then-
///   double-2 or double-to-10-then-prevent-5.
/// - *If damage … is being divided or assigned among multiple permanents an
///   opponent controls … divide the original amount and double the results* →
///   the War Mammoth test, from the other side of the table.
pub fn gisela_blade_of_goldnight() -> Arc<CardData> {
    CardDataBuilder::new("Gisela, Blade of Goldnight")
        .mana_cost(ManaCost::build(
            &[ManaType::Red, ManaType::White, ManaType::White],
            4,
        ))
        .color(Color::Red)
        .color(Color::White)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Angel))
        .power_toughness(5, 5)
        .keyword_flag(KeywordFlag::Flying)
        .keyword_flag(KeywordFlag::FirstStrike)
        .rules_text(
            "Flying, first strike\nIf a source would deal damage to an opponent or a \
             permanent an opponent controls, that source deals double that damage to that \
             player or permanent instead.\nIf a source would deal damage to you or a \
             permanent you control, prevent half that damage, rounded up.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                AffectedSet::Filter {
                    filter: ObjectFilter::ByController(PlayerRef::Opponent),
                },
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::Opponents),
        ))
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                AffectedSet::Filter {
                    filter: ObjectFilter::ByController(PlayerRef::You),
                },
                Rewrite::Amount(AmountRewrite::PreventHalf(Rounding::Up)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Angel of Suffering — {3}{B}{B}
///
/// > Flying
/// >
/// > If damage would be dealt to you, prevent that damage and mill twice that
/// > many cards.
///
/// The phase's only prevention consumer, and the only effect in it that is
/// about **no object at all**: `AffectedSet::Fixed(vec![])` beside
/// `PlayerSet::You`. Its rider is what makes `Rider.subject` an `EventSubject`
/// rather than an `Option<ObjectId>` — "mill" needs to name the player the
/// damage was aimed at (`codebase-state.md` item 27) — and "twice that many"
/// is `AmountExpr::Multiply(ReplacedAmount, 2)`, read off the event as the
/// CR 616.1 loop saw it.
///
/// # The rulings, and where each is tested
///
/// - *If you would mill more cards than are in your library, you mill all
///   cards in your library* → CR 701.2's "as much as it can", in
///   `Primitive::Mill`.
/// - *Damage that would be dealt to you will be prevented even if you can't
///   mill twice that many* → the prevention is not conditional on the rider,
///   which falls out of §4.1a's queue-then-perform order.
/// - *If the damage can't be prevented for some reason, you'll still mill
///   twice that many* → **RD-4's** dovetail test, beside Reverse Damage: the
///   rider is unconditional once queued (CR 615.12), and RD-1 has no way to
///   make damage unpreventable.
pub fn angel_of_suffering() -> Arc<CardData> {
    CardDataBuilder::new("Angel of Suffering")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black], 3))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Nightmare))
        .subtype(Subtype::Creature(CreatureType::Angel))
        .power_toughness(5, 3)
        .keyword_flag(KeywordFlag::Flying)
        .rules_text(
            "Flying\nIf damage would be dealt to you, prevent that damage and mill twice \
             that many cards.",
        )
        .ability(static_replacement(
            // About a player and no object at all: `NO_OBJECTS` names that,
            // where a `Filter` would have to describe an empty set and
            // `SourceOnly` would make the Angel shield *itself*.
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                AffectedSet::NO_OBJECTS,
                Rewrite::Prevent,
            )
            .affecting_players(PlayerSet::You)
            .with_then(Effect::Atom(
                Primitive::Mill(AmountExpr::Multiply(Box::new(AmountExpr::ReplacedAmount), 2)),
                // The rider's single resolved target is the event's subject —
                // the player the damage was aimed at, which for this card is
                // always the Angel's controller and for the next one on this
                // path may not be.
                EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
            )),
        ))
        .build()
}

/// Loyalty Probe — a fixture planeswalker, and CR 120.3c's consumer.
///
/// **Not a printed card**, and the module docs say why a printed one could not
/// take its place. Printed loyalty 3, no abilities at all, `{2}` so any deck
/// can cast it.
///
/// Three is chosen rather than five: `SelectionFilter::Any` enumerates
/// planeswalkers, so a random agent's Lightning Bolt finds this one, and three
/// damage takes it to exactly zero — which is what makes **CR 704.5i**
/// reachable from a fuzz game for the first time (`phase_sba_cards` measured
/// that state-based action at 0 across 200 stress games, blocked on "loyalty
/// abilities + CR 120.3c"; RD-1 supplies the second half and the first is
/// combat's and Phase 8's).
///
/// It has no subtype. CR 205.3j gives every printed planeswalker one, and every
/// name in `PlaneswalkerType` belongs to a real character — so borrowing one
/// would put an invented card under a real planeswalker's name, which is
/// exactly the thing `engineering-practices.md` §3 keeps out of the registry.
///
/// Attacking one stays out of reach: `combat::validation` refuses planeswalker
/// attack targets, and that is combat's phase rather than this one's.
pub fn loyalty_probe() -> Arc<CardData> {
    CardDataBuilder::new("Loyalty Probe")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Planeswalker)
        .loyalty(3)
        .rules_text("")
        .build()
}

// ---------------------------------------------------------------------------
// RD-2 — CR 615.7 prevention shields
// ---------------------------------------------------------------------------

/// A spell or activated ability whose whole effect is one atom.
fn one_shot(ability_type: AbilityType, costs: Vec<Cost>, effect: Effect) -> AbilityDef {
    AbilityDef {
        id: new_ability_id(),
        ability_type,
        costs,
        effect,
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
    }
}

/// "Prevent the next `n` damage that would be dealt to … this turn" — the def
/// every count-carrying card here shares.
///
/// [`AmountRewrite::PreventRemaining`] reads the count off
/// `Uses::NextDamage(n)`, so the number is written once; the object set is the
/// empty `Fixed` the resolution fills with its target — or, for Samite
/// Censer-Bearer, with each creature its filter finds — and the duration is the
/// card's "this turn", authored here because CR 608.2c will not let the engine
/// infer it.
fn prevent_the_next_this_turn(n: u64) -> Primitive {
    Primitive::CreateReplacement(
        Box::new(
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                AffectedSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::PreventRemaining),
            )
            .next_damage(n),
        ),
        Duration::UntilEndOfTurn,
        PatternFill::Authored,
    )
}

/// "any target" — CR 115.4's creature, player or planeswalker.
fn any_target() -> EffectRecipient {
    EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1))
}

/// "creatures you control", as a filter the layer walk resolves against the
/// row's controller (CR 109.5).
fn creatures_you_control() -> ObjectFilter {
    ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    )
}

/// Mending Hands — {W}
///
/// > Prevent the next 4 damage that would be dealt to any target this turn.
///
/// The plain CR 615.7 shield: one row, `NextDamage(4)`, the target filled in at
/// resolution as an object or a player, gone when the count reaches zero or at
/// the cleanup step (CR 615.3's "until they're used up or their duration has
/// expired"). Scryfall lists no rulings (2026-09-08), so its tests are the
/// rule's own: the count depletes per point across events (`ATOM-615.7-001`), a
/// 4 against 5 lets 1 through, two attackers into a shielded player are one
/// `allocate` prompt whose every answer prevents the same total
/// (`ATOM-615.7-002`), and a shield made after the damage prevents nothing
/// (`ATOM-615.4-001`).
///
/// **The pooled card of the phase**, because it is the first registry row a
/// pooled damage event meets and the first CR 615.7 prompt a fuzz game can
/// reach. Healing Salve was the canonical printing and is modal.
pub fn mending_hands() -> Arc<CardData> {
    CardDataBuilder::new("Mending Hands")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text("Prevent the next 4 damage that would be dealt to any target this turn.")
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Atom(prevent_the_next_this_turn(4), any_target()),
        ))
        .build()
}

/// Samite Healer — {1}{W}
///
/// > {T}: Prevent the next 1 damage that would be dealt to any target this
/// > turn.
///
/// The activated shape — a repeatable source of rows on a creature the random
/// agent will tap — and the card that pins CR 113.7a on the row: an ability's
/// source is the permanent that has it, so the row names the Healer, not the
/// stack object CR 608.2n deletes the moment the ability finishes resolving.
/// Scryfall lists no rulings (2026-09-08).
pub fn samite_healer() -> Arc<CardData> {
    CardDataBuilder::new("Samite Healer")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(1, 1)
        .rules_text("{T}: Prevent the next 1 damage that would be dealt to any target this turn.")
        .ability(one_shot(
            AbilityType::Activated,
            vec![Cost::Tap],
            Effect::Atom(prevent_the_next_this_turn(1), any_target()),
        ))
        .build()
}

/// Safe Passage — {2}{W}
///
/// > Prevent all damage that would be dealt to you and creatures you control
/// > this turn.
///
/// A `Filter` + `You` row with no count and `Uses::Static` — the shape whose
/// set is evaluated **at the event**, which is the whole difference between it
/// and Samite Censer-Bearer below.
///
/// # The rulings (Scryfall, 2026-09-08), and where each is tested
///
/// - *prevents all damage, not just combat damage* → a non-combat bolt to its
///   caster is prevented.
/// - *will prevent damage dealt to creatures that weren't on the battlefield
///   at the time it resolved* → the load-bearing one: a creature placed after
///   resolution is covered, because the row carries a `Filter` and not a
///   `Fixed` — the opposite of CR 615.11's ruling on Censer-Bearer.
/// - *doesn't prevent damage that would be dealt to planeswalkers you
///   control* → Loyalty Probe under Safe Passage loses loyalty; the filter is
///   `ByType(Creature)`.
/// - *has no effect on damage that's already been dealt* → damage marked
///   before it resolves stays marked (CR 615.4 from the other side).
pub fn safe_passage() -> Arc<CardData> {
    CardDataBuilder::new("Safe Passage")
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(
            "Prevent all damage that would be dealt to you and creatures you control this turn.",
        )
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Atom(
                Primitive::CreateReplacement(
                    Box::new(
                        ReplacementDef::new(
                            EventPattern::DealDamage { source: None, combat: None },
                            AffectedSet::Filter { filter: creatures_you_control() },
                            Rewrite::Prevent,
                        )
                        .affecting_players(PlayerSet::You),
                    ),
                    Duration::UntilEndOfTurn,
                    PatternFill::Authored,
                ),
                EffectRecipient::Implicit,
            ),
        ))
        .build()
}

/// Samite Censer-Bearer — {W}
///
/// > {W}, Sacrifice this creature: Prevent the next 1 damage that would be
/// > dealt to each creature you control this turn.
///
/// CR 615.11's consumer: "creates a prevention shield for each applicable
/// creature when the spell or ability … resolves". The recipient is
/// `FilteredPermanents`, so the resolution makes **one row per creature** it
/// finds, each with its own `NextDamage(1)` — the Censer-Bearer itself is
/// already in the graveyard, sacrificed as the cost.
///
/// The ruling (Scryfall, 2026-09-08): *this ability sets up a separate 1-point
/// damage prevention shield on each creature you control at the time the
/// ability resolves* → a creature entering afterwards has none
/// (`ATOM-615.11-001`), and two creatures each taking 2 each take 1, because
/// the counts are separate. Kitsune Palliator's "each creature and each
/// player" is this card plus an each-player recipient `EffectRecipient` lacks;
/// one customer, so it waits.
pub fn samite_censer_bearer() -> Arc<CardData> {
    CardDataBuilder::new("Samite Censer-Bearer")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Rebel))
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(1, 1)
        .rules_text(
            "{W}, Sacrifice this creature: Prevent the next 1 damage that would be dealt to \
             each creature you control this turn.",
        )
        .ability(one_shot(
            AbilityType::Activated,
            vec![Cost::Mana(ManaCost::build(&[ManaType::White], 0)), Cost::SacrificeSelf],
            Effect::Atom(
                prevent_the_next_this_turn(1),
                EffectRecipient::FilteredPermanents(creatures_you_control()),
            ),
        ))
        .build()
}

// ---------------------------------------------------------------------------
// RD-3 — sources (CR 609.7, 615.8, 615.9, 615.10)
// ---------------------------------------------------------------------------

/// "The next time a source of your choice would deal damage to you this turn"
/// — the def the three CR 609.7a cards share, differing only in the property
/// the chosen source must still have and in what the effect then does.
///
/// The `SourcePattern`'s `object` is `None` here and stays `None` on the card:
/// CR 609.7a chooses the source *when the effect is created*, so the
/// resolution writes it (`PatternFill::ChosenDamageSource`). The `filter` is
/// the card's own and is rechecked at every damage event, which is CR 609.7b.
fn the_next_damage_from_a_chosen_source(
    filter: Option<ObjectFilter>,
    rewrite: Rewrite,
) -> ReplacementDef {
    let source = match filter {
        Some(f) => SourcePattern::matching(f),
        None => SourcePattern::chosen(),
    };
    ReplacementDef::new(
        EventPattern::DealDamage { source: Some(source), combat: None },
        AffectedSet::NO_OBJECTS,
        rewrite,
    )
    .affecting_players(PlayerSet::You)
    .once()
}

/// "Cleric creature you control" — all three clauses, because the card says
/// all three and a subtype does not carry its card type on an object outside
/// the battlefield.
fn clerics_you_control() -> ObjectFilter {
    ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::And(
            Box::new(ObjectFilter::BySubtype(Subtype::Creature(CreatureType::Cleric))),
            Box::new(ObjectFilter::ByController(PlayerRef::You)),
        )),
    )
}

/// Circle of Protection: Red — {1}{W}
///
/// > {1}: The next time a red source of your choice would deal damage to you
/// > this turn, prevent that damage.
///
/// **Both halves of CR 609.7 on one card**, which is why it is RD-3's first
/// consumer: `object` is 609.7a's chosen source, written by the resolution,
/// and `filter` is 609.7b's property, rechecked when the damage comes. The two
/// disagree the moment the chosen creature stops being red, and the rule says
/// the shield then neither applies nor is used up.
///
/// `Uses::Once` with a whole-event [`Rewrite::Prevent`] is CR 615.8: "these
/// effects prevent the next instance of damage from that source, **regardless
/// of how much damage that is**".
///
/// # The rulings (Scryfall, 2026-09-08), and where each is tested
///
/// - *A source of damage is a permanent, a spell on the stack (including one
///   that creates a permanent), or any object referred to by an object on the
///   stack. A source doesn't need to be capable of dealing damage to be a legal
///   choice.* → `SelectionFilter::DamageSource`, tested in `oracle::legality`.
///   The "referred to by an object on the stack" category is unreachable and
///   `ATOM-609.7a-001` is `COVERS-PARTIAL` for it.
/// - *Can be used even when there is no damage to prevent. It prevents the next
///   damage (if any) from the source this turn.* → the row sits unused and
///   expires at the cleanup step (CR 615.3).
pub fn circle_of_protection_red() -> Arc<CardData> {
    CardDataBuilder::new("Circle of Protection: Red")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text(
            "{1}: The next time a red source of your choice would deal damage to you this \
             turn, prevent that damage.",
        )
        .ability(one_shot(
            AbilityType::Activated,
            vec![Cost::Mana(ManaCost::build(&[], 1))],
            Effect::Atom(
                Primitive::CreateReplacement(
                    Box::new(the_next_damage_from_a_chosen_source(
                        Some(ObjectFilter::ByColor(Color::Red)),
                        Rewrite::Prevent,
                    )),
                    Duration::UntilEndOfTurn,
                    PatternFill::ChosenDamageSource,
                ),
                EffectRecipient::Implicit,
            ),
        ))
        .build()
}

/// Reverse Damage — {1}{W}{W}
///
/// > The next time a source of your choice would deal damage to you this turn,
/// > prevent that damage. You gain life equal to the damage prevented this way.
///
/// The chosen source with **no property at all** — so 609.7b has nothing to
/// recheck and the id is the whole predicate — and the printed reader of
/// RD-2's prevented-amount channel: `AmountExpr::DamagePrevented` in a
/// CR 615.5 rider, which the fixture in `tests/phase_rd2_integration_test.rs`
/// was standing in for.
///
/// The rider's recipient is `Target`, which a rider resolves against the
/// event's subject — here the player the damage was headed for, who is also
/// the row's affected player.
///
/// # The ruling (Scryfall, 2026-09-08), and where it is tested
///
/// - *It only affects damage dealt by the source one time. If the source
///   damages you a second time this turn, the damage will not be reversed.* →
///   CR 615.8's `Uses::Once`, and the second instance is dealt in full with no
///   life gained.
pub fn reverse_damage() -> Arc<CardData> {
    CardDataBuilder::new("Reverse Damage")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(
            "The next time a source of your choice would deal damage to you this turn, \
             prevent that damage. You gain life equal to the damage prevented this way.",
        )
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Atom(
                Primitive::CreateReplacement(
                    Box::new(
                        the_next_damage_from_a_chosen_source(None, Rewrite::Prevent).with_then(
                            Effect::Atom(
                                Primitive::GainLife(AmountExpr::DamagePrevented),
                                EffectRecipient::Target(
                                    SelectionFilter::Player,
                                    TargetCount::Exactly(1),
                                ),
                            ),
                        ),
                    ),
                    Duration::UntilEndOfTurn,
                    PatternFill::ChosenDamageSource,
                ),
                EffectRecipient::Implicit,
            ),
        ))
        .build()
}

/// Dark Sphere — {0}
///
/// > {T}, Sacrifice this artifact: The next time a source of your choice would
/// > deal damage to you this turn, prevent half that damage, rounded down.
///
/// The prevention half of RD-1's rounding decision, reached from a **registry
/// row** rather than from a static ability — and the board that makes
/// `consume_use`'s order matter: half of 1 rounded down is 0, so the row
/// prevents nothing and CR 609.7b leaves it for the next instance. RD-2's
/// `a_once_prevention_that_prevents_nothing_is_not_used_up_dark_sphere_is_rd_3s`
/// is the fixture this card replaces.
///
/// # The ruling (Scryfall, 2026-09-08), and where it is tested
///
/// - *If two or more of these effects would apply, you apply them sequentially.
///   So if the source would deal 5 damage after two of these abilities have
///   resolved, the first one prevents 2 damage, reducing it to 3 damage, then
///   the second one prevents a further 1 damage, reducing the total damage
///   dealt to 2.* → two rows, each applied once (CR 614.5), and 5 → 3 → 2.
pub fn dark_sphere() -> Arc<CardData> {
    CardDataBuilder::new("Dark Sphere")
        .mana_cost(ManaCost::build(&[], 0))
        .card_type(CardType::Artifact)
        .rules_text(
            "{T}, Sacrifice this artifact: The next time a source of your choice would deal \
             damage to you this turn, prevent half that damage, rounded down.",
        )
        .ability(one_shot(
            AbilityType::Activated,
            vec![Cost::Tap, Cost::SacrificeSelf],
            Effect::Atom(
                Primitive::CreateReplacement(
                    Box::new(the_next_damage_from_a_chosen_source(
                        None,
                        Rewrite::Amount(AmountRewrite::PreventHalf(Rounding::Down)),
                    )),
                    Duration::UntilEndOfTurn,
                    PatternFill::ChosenDamageSource,
                ),
                EffectRecipient::Implicit,
            ),
        ))
        .build()
}

/// Guardian Seraph — {2}{W}{W}
///
/// > Flying
/// >
/// > If a source an opponent controls would deal damage to you, prevent 1 of
/// > that damage.
///
/// CR 615.10's static partial prevention with a **source-side** predicate, and
/// `AmountRewrite::PreventUpTo`'s first printed producer (`codebase-state.md`
/// item 91). The predicate is CR 609.7c's, not 609.7a's: nothing is chosen, so
/// the filter is asked of every source, on the battlefield or not.
///
/// **The pooled card of this PR.** It is the first source in
/// `PERFORMANCE_POOL` whose gather evaluates an `ObjectFilter` on the damage's
/// *source* rather than on its target, which is a new read per damage event
/// for as long as it is on the battlefield.
///
/// # The rulings (Scryfall, 2026-09-08), and where each is tested
///
/// - *This ability prevents 1 damage from each source an opponent controls each
///   time that source would deal damage to you. It prevents 1 of any damage,
///   not just combat damage.* → two simultaneous attackers each have 1
///   prevented, and a non-combat bolt is reduced too. That is CR 615.10's
///   "separately to damage from other applicable events that would happen at
///   the same time", which is `ATOM-615.10-001`'s shape on a player.
/// - *Spells and permanents have controllers, but cards that aren't on the
///   stack or the battlefield don't. If a source without a controller (such as
///   a cycled Jund Sojourners) would deal damage to you, 1 of that damage is
///   prevented if an opponent owns that source.* → **not expressible, and not
///   tested.** `ObjectFilter::ByController` on an object in a hidden zone falls
///   through `compute::base_controller` to its owner, so the engine would give
///   this ruling's answer by accident rather than by rule — and no registered
///   card can produce a damage source outside the battlefield and the stack, so
///   there is no board to assert it on. CR 609.7c's non-battlefield source is
///   covered instead by Lightning Bolt, a spell on the stack
///   (`ATOM-609.7c-001`).
/// - *The effects from multiple Guardian Seraphs are cumulative.* → two on the
///   battlefield reduce 3 damage to 1, each applied once (CR 614.5).
pub fn guardian_seraph() -> Arc<CardData> {
    CardDataBuilder::new("Guardian Seraph")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Angel))
        .power_toughness(3, 4)
        .keyword_flag(KeywordFlag::Flying)
        .rules_text(
            "Flying\nIf a source an opponent controls would deal damage to you, prevent 1 \
             of that damage.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DealDamage {
                    source: Some(SourcePattern::matching(ObjectFilter::ByController(
                        PlayerRef::Opponent,
                    ))),
                    combat: None,
                },
                AffectedSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::PreventUpTo(1)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Daunting Defender — {4}{W}
///
/// > If a source would deal damage to a Cleric creature you control, prevent 1
/// > of that damage.
///
/// **CR 615.10's own example, and the card the rule is written around.** The
/// predicate is entirely on the *target* side — any source at all — which is
/// what makes it the second producer for `AmountRewrite::PreventUpTo` without
/// being a second copy of Guardian Seraph's path: one names an object set with
/// a three-clause filter, the other names a player.
///
/// > 615.10 Example: Daunting Defender says "If a source would deal damage to a
/// > Cleric creature you control, prevent 1 of that damage." Pyroclasm says
/// > "Pyroclasm deals 2 damage to each creature." Pyroclasm will deal 1 damage
/// > to each Cleric creature controlled by Daunting Defender's controller. It
/// > will deal 2 damage to each other creature.
///
/// It is itself a Cleric, so it is inside its own affected set — the rule's
/// example board has it taking 1 from that Pyroclasm.
///
/// Scryfall lists no rulings (2026-09-08); the test is the rule's example,
/// verbatim (`ATOM-615.10-001`).
pub fn daunting_defender() -> Arc<CardData> {
    CardDataBuilder::new("Daunting Defender")
        .mana_cost(ManaCost::build(&[ManaType::White], 4))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(3, 3)
        .rules_text(
            "If a source would deal damage to a Cleric creature you control, prevent 1 of \
             that damage.",
        )
        .ability(static_replacement(ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::Filter { filter: clerics_you_control() },
            Rewrite::Amount(AmountRewrite::PreventUpTo(1)),
        )))
        .build()
}

/// Pyroclasm — {1}{R}
///
/// > Pyroclasm deals 2 damage to each creature.
///
/// CR 615.10's example needs the other half of it, and this is the pool's
/// first "each creature" damage: one source, every creature, **one event**.
/// `EffectRecipient::FilteredPermanents` is what names them, and the batch is
/// what makes CR 615.7's allocation and CR 704.3's simultaneity reachable at
/// all — a loop of one-member batches would be a different game.
///
/// Scryfall lists no rulings (2026-09-08).
pub fn pyroclasm() -> Arc<CardData> {
    CardDataBuilder::new("Pyroclasm")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text("Pyroclasm deals 2 damage to each creature.")
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Atom(
                Primitive::DealDamage { amount: AmountExpr::Fixed(2), unpreventable: false },
                EffectRecipient::FilteredPermanents(ObjectFilter::ByType(CardType::Creature)),
            ),
        ))
        .build()
}

/// Fog — {G}
///
/// > Prevent all combat damage that would be dealt this turn.
///
/// The `combat` field's only consumer, and the widest set in the phase:
/// `Filter { All }` beside `PlayerSet::Everyone`, so it covers damage to
/// anything and anyone — including the caster's own opponents' creatures,
/// which is what "all" means and what a `You`-scoped shield would get wrong.
///
/// `Uses::Static` on a registry row with a duration: the row is not spent by
/// preventing, it expires at the cleanup step (CR 615.3).
///
/// Scryfall lists no rulings (2026-09-08). The rule's own boundary is the
/// test: **non-combat** damage under Fog goes through.
pub fn fog() -> Arc<CardData> {
    let (affected, players) = every_permanent_or_player();
    CardDataBuilder::new("Fog")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Instant)
        .rules_text("Prevent all combat damage that would be dealt this turn.")
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Atom(
                Primitive::CreateReplacement(
                    Box::new(
                        ReplacementDef::new(
                            EventPattern::DealDamage { source: None, combat: Some(true) },
                            affected,
                            Rewrite::Prevent,
                        )
                        .affecting_players(players),
                    ),
                    Duration::UntilEndOfTurn,
                    PatternFill::Authored,
                ),
                EffectRecipient::Implicit,
            ),
        ))
        .build()
}

/// Torbran, Thane of Red Fell — {1}{R}{R}{R}
///
/// > If a red source you control would deal damage to an opponent or a
/// > permanent an opponent controls, it deals that much damage plus 2 instead.
///
/// `AmountRewrite::Plus`'s printed customer, and the phase's only **two-sided**
/// predicate that constrains both sides with a filter: a source-side
/// `ByColor(Red)` and `ByController(You)` beside a target-side
/// `ByController(Opponent)` and `PlayerSet::Opponents`. Both are asked at the
/// event, so a creature that stops being red stops adding 2 (CR 609.7c).
///
/// # The rulings (Scryfall, 2026-09-08), and where each is tested
///
/// - *The additional 2 damage is dealt by the same source as the original
///   source of damage. The damage isn't dealt by Torbran unless Torbran is the
///   original source of damage.* → asserted on `GameEvent::DamageDealt`'s
///   `source_id`, which the rewrite does not touch.
/// - *If another effect modifies how much damage your red source would deal,
///   including preventing some of it, the player being dealt damage … chooses
///   an order in which to apply those effects. **If all of the damage is
///   prevented, Torbran's effect no longer applies.*** → the sharp one: a
///   prevention that empties the event first makes `never_happens` drop it on
///   the next iteration, so Torbran is never gathered (CR 614.7a, re-asked per
///   iteration). And the first half is why `ordering_cannot_change_outcome`
///   must not admit `Plus`.
/// - *If damage dealt by a source you control is being divided or assigned
///   among multiple permanents an opponent controls … divide the original
///   amount before adding 2.* → the trample test: `assign_combat_damage`
///   divides before anything is proposed, so 2 and 3 become 4 and 5.
pub fn torbran_thane_of_red_fell() -> Arc<CardData> {
    CardDataBuilder::new("Torbran, Thane of Red Fell")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red, ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Dwarf))
        .subtype(Subtype::Creature(CreatureType::Noble))
        .power_toughness(2, 4)
        .rules_text(
            "If a red source you control would deal damage to an opponent or a permanent an \
             opponent controls, it deals that much damage plus 2 instead.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DealDamage {
                    source: Some(SourcePattern::matching(ObjectFilter::And(
                        Box::new(ObjectFilter::ByColor(Color::Red)),
                        Box::new(ObjectFilter::ByController(PlayerRef::You)),
                    ))),
                    combat: None,
                },
                AffectedSet::Filter {
                    filter: ObjectFilter::ByController(PlayerRef::Opponent),
                },
                Rewrite::Amount(AmountRewrite::Plus(2)),
            )
            .affecting_players(PlayerSet::Opponents),
        ))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::actions::{ActionContext, GameAction};
    use crate::events::event::DamageTarget;
    use crate::state::game_state::GameState;
    use crate::test_support::{
        place_vanilla_creature, put_on_battlefield, setup_two_player_game, test_ctx,
        RecordingDecisionProvider,
    };
    use crate::types::effects::CounterType;

    /// A 2/2 for P0 and a board with no replacement source but the one under
    /// test. Damage to a *creature* throughout: RD-1's tests of damage to a
    /// **player** need CR 120.3a's decomposition, which is two commits away,
    /// and they live in `tests/phase_rd_integration_test.rs`.
    fn board() -> (GameState, ObjectIdPair) {
        let mut game = setup_two_player_game();
        let bolt_source = place_vanilla_creature(&mut game, 0, 1, 1, &[]);
        let victim = place_vanilla_creature(&mut game, 1, 5, 5, &[]);
        (game, ObjectIdPair { source: bolt_source, victim })
    }

    struct ObjectIdPair {
        source: crate::types::ids::ObjectId,
        victim: crate::types::ids::ObjectId,
    }

    fn deal(game: &mut GameState, ids: &ObjectIdPair, amount: u64) {
        game.execute_action(
            GameAction::DealDamage {
                source: ids.source,
                target: DamageTarget::Object(ids.victim),
                amount,
                is_combat: false,
                unpreventable: false
            },
            &test_ctx(),
        )
        .unwrap();
    }

    fn marked(game: &GameState, ids: &ObjectIdPair) -> u32 {
        game.battlefield[&ids.victim].damage_marked
    }

    // CR 701.10g on a real card: 3 becomes 6.
    #[test]
    fn furnace_of_rath_doubles_damage_to_a_permanent() {
        let (mut game, ids) = board();
        put_on_battlefield(&mut game, furnace_of_rath(), 0);
        deal(&mut game, &ids, 3);
        assert_eq!(marked(&game, &ids), 6);
    }

    // "The multiplied damage counts in all ways as if it came from the
    // original source. Furnace of Rath is not the source."
    #[test]
    fn furnace_of_rath_is_not_the_source_of_the_doubled_damage() {
        use crate::events::event::GameEvent;
        let (mut game, ids) = board();
        let furnace = put_on_battlefield(&mut game, furnace_of_rath(), 0);
        deal(&mut game, &ids, 3);
        let dealt: Vec<_> = game
            .events
            .events()
            .filter_map(|e| match e {
                GameEvent::DamageDealt { source_id, amount, .. } => Some((*source_id, *amount)),
                _ => None,
            })
            .collect();
        assert_eq!(dealt, vec![(ids.source, 6)]);
        assert_ne!(dealt[0].0, furnace);
    }

    // "Half of 1 rounded down is 0. A source that would deal 1 damage won't
    // deal damage at all" — CR 614.7a drops the emptied event, so nothing is
    // marked and no `DamageDealt` is emitted.
    //
    // **Three boards, because "nothing happened" is not evidence on its own.**
    // The absence of a `DamageDealt` event cannot, by itself, distinguish
    // "Ghosts halved 1 to 0 and CR 614.7a dropped it" from "the damage never
    // reached the pipeline at all" — so the same 1 damage without Ghosts is the
    // control, and 2 damage with Ghosts shows the instance applying rather than
    // being skipped. What is still not observable here is *which rule* dropped
    // the event, which is a question for the trace sink (`roadmap-v2.md` A4c).
    #[test]
    fn ghosts_of_the_innocent_halves_one_damage_to_nothing() {
        use crate::events::event::GameEvent;

        let (mut control, ids) = board();
        deal(&mut control, &ids, 1);
        assert_eq!(marked(&control, &ids), 1, "1 damage lands when nothing halves it");

        let (mut game, ids) = board();
        put_on_battlefield(&mut game, ghosts_of_the_innocent(), 0);
        deal(&mut game, &ids, 2);
        assert_eq!(marked(&game, &ids), 1, "and Ghosts is applying on this board");

        let (mut game, ids) = board();
        put_on_battlefield(&mut game, ghosts_of_the_innocent(), 0);
        deal(&mut game, &ids, 1);
        assert_eq!(marked(&game, &ids), 0);
        assert!(!game
            .events
            .events()
            .any(|e| matches!(e, GameEvent::DamageDealt { .. })));
    }

    // "Multiple Ghosts of the Innocent effects are cumulative … with three on
    // the battlefield, 14 damage becomes 7, then 3, then finally 1."
    //
    // Three instances is three CR 616.1 candidates, so the affected side is
    // asked twice — and the answer cannot matter, because halving commutes with
    // itself. `picking_all` is the provider that would notice a candidate the
    // list should not have offered.
    #[test]
    fn three_ghosts_take_fourteen_damage_to_one() {
        let (mut game, ids) = board();
        for _ in 0..3 {
            put_on_battlefield(&mut game, ghosts_of_the_innocent(), 0);
        }
        let dp = RecordingDecisionProvider::picking_all();
        let ctx = ActionContext::new(&dp);
        game.execute_action(
            GameAction::DealDamage {
                source: ids.source,
                target: DamageTarget::Object(ids.victim),
                amount: 14,
                is_combat: false,
                unpreventable: false
            },
            &ctx,
        )
        .unwrap();
        assert_eq!(marked(&game, &ids), 1);
        assert_eq!(dp.prompts(), 2, "three candidates, then two, then one");
    }

    // Gisela's doubler is scoped by CR 102.1's "an opponent", resolved against
    // her controller: P0's Gisela doubles damage to P1's creature and leaves
    // damage to P0's own alone.
    #[test]
    fn gisela_doubles_only_damage_to_an_opponents_permanent() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, gisela_blade_of_goldnight(), 0);
        let source = place_vanilla_creature(&mut game, 0, 1, 1, &[]);
        let theirs = place_vanilla_creature(&mut game, 1, 9, 9, &[]);
        let mine = place_vanilla_creature(&mut game, 0, 9, 9, &[]);

        for victim in [theirs, mine] {
            game.execute_action(
                GameAction::DealDamage {
                    source,
                    target: DamageTarget::Object(victim),
                    amount: 3,
                    is_combat: false,
                    unpreventable: false
                },
                &test_ctx(),
            )
            .unwrap();
        }
        assert_eq!(game.battlefield[&theirs].damage_marked, 6);
        // Her *other* half applies here: prevent half of 3, rounded up, is 2.
        assert_eq!(game.battlefield[&mine].damage_marked, 1);
    }

    // "Gisela doubles damage … from any source, including sources controlled
    // by those opponents." The pattern carries no source-side constraint, and
    // RD-3 is where one could be added by mistake.
    #[test]
    fn gisela_doubles_an_opponents_own_source() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, gisela_blade_of_goldnight(), 0);
        let their_source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        let their_victim = place_vanilla_creature(&mut game, 1, 9, 9, &[]);
        game.execute_action(
            GameAction::DealDamage {
                source: their_source,
                target: DamageTarget::Object(their_victim),
                amount: 2,
                is_combat: false,
                unpreventable: false
            },
            &test_ctx(),
        )
        .unwrap();
        assert_eq!(game.battlefield[&their_victim].damage_marked, 4);
    }

    // CR 107.1a, on the board rather than on the arithmetic: Gisela rounds up,
    // so 5 damage to something she protects leaves 2.
    #[test]
    fn gisela_prevents_half_rounded_up() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, gisela_blade_of_goldnight(), 0);
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        let mine = place_vanilla_creature(&mut game, 0, 9, 9, &[]);
        game.execute_action(
            GameAction::DealDamage {
                source,
                target: DamageTarget::Object(mine),
                amount: 5,
                is_combat: false,
                unpreventable: false
            },
            &test_ctx(),
        )
        .unwrap();
        assert_eq!(game.battlefield[&mine].damage_marked, 2);
    }

    // The fixture is a planeswalker that enters with its printed loyalty
    // (CR 306.5b), which is what CR 120.3c will take counters off.
    #[test]
    fn loyalty_probe_enters_with_three_loyalty() {
        let mut game = setup_two_player_game();
        let probe = put_on_battlefield(&mut game, loyalty_probe(), 0);
        assert_eq!(
            game.battlefield[&probe].counter_count(CounterType::Loyalty),
            3
        );
    }

    // -----------------------------------------------------------------------
    // RD-2
    // -----------------------------------------------------------------------

    use crate::engine::resolve::{ResolutionContext, ResolvedTarget};
    use crate::events::event::GameEvent;
    use crate::state::replacement_effects::RegisteredReplacementEffect;
    use crate::test_support::put_in_hand;
    use crate::types::replacement::Uses;

    /// Resolve `card`'s spell effect for `controller` against `targets`, the
    /// way the stack would — the card's own id is the resolution's source.
    fn resolve_spell(
        game: &mut GameState,
        card: Arc<CardData>,
        controller: crate::types::ids::PlayerId,
        targets: Vec<ResolvedTarget>,
    ) -> crate::types::ids::ObjectId {
        let id = put_in_hand(game, card.clone(), controller);
        let ctx = ResolutionContext {
            source: id,
            ability_source: None,
            controller,
            targets,
            replaced_amount: None,
            damage_prevented: None,
        };
        let dp = crate::test_support::test_dp();
        game.resolve_effect(&card.abilities[0].effect, &ctx, &dp).unwrap();
        id
    }

    fn rows(game: &GameState) -> Vec<&RegisteredReplacementEffect> {
        game.replacement_effects.iter().collect()
    }

    fn bolt(game: &mut GameState, source: crate::types::ids::ObjectId, target: DamageTarget, amount: u64) {
        game.execute_action(
            GameAction::DealDamage { source, target, amount, is_combat: false, unpreventable: false },
            &test_ctx(),
        )
        .unwrap();
    }

    fn damage_dealt(game: &GameState) -> usize {
        game.events.events().filter(|e| matches!(e, GameEvent::DamageDealt { .. })).count()
    }

    // The plain shield: one row, on the player it targeted, counting four,
    // for this turn, with the resolution's target kept on it.
    #[test]
    fn mending_hands_makes_one_row_counting_four_on_its_target() {
        let mut game = setup_two_player_game();
        let spell = resolve_spell(&mut game, mending_hands(), 0, vec![ResolvedTarget::Player(1)]);
        let rows = rows(&game);
        assert_eq!(rows.len(), 1);
        let row = rows[0];
        assert_eq!(row.def.uses, Uses::NextDamage(4));
        assert_eq!(row.def.affected, AffectedSet::NO_OBJECTS);
        assert_eq!(row.def.affected_players, PlayerSet::Fixed(vec![1]));
        assert_eq!(row.duration, Duration::UntilEndOfTurn);
        assert_eq!(row.source, spell);
        assert_eq!(row.controller, 0);
        assert_eq!(row.targets, vec![ResolvedTarget::Player(1)]);
        assert!(row.def.is_prevention());
    }

    // Targeting a creature fills the object half instead, and leaves the
    // player half empty.
    #[test]
    fn mending_hands_on_a_creature_fills_the_object_set() {
        let mut game = setup_two_player_game();
        let bear = place_vanilla_creature(&mut game, 1, 2, 2, &[]);
        resolve_spell(&mut game, mending_hands(), 0, vec![ResolvedTarget::Object(bear)]);
        let row = rows(&game)[0];
        assert_eq!(row.def.affected, AffectedSet::Fixed(vec![bear]));
        assert_eq!(row.def.affected_players, PlayerSet::Nobody);
    }

    // "Safe Passage prevents all damage, not just combat damage, that would be
    // dealt to you and creatures you control this turn."
    #[test]
    fn safe_passage_prevents_noncombat_damage_to_its_caster() {
        let mut game = setup_two_player_game();
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        resolve_spell(&mut game, safe_passage(), 0, Vec::new());
        bolt(&mut game, source, DamageTarget::Player(0), 3);
        assert_eq!(game.players[0].life_total, 20);
        assert_eq!(damage_dealt(&game), 0, "CR 615.6 — the damage never happened");
    }

    // "Safe Passage will prevent damage dealt to creatures that weren't on
    // the battlefield at the time it resolved." The row is a `Filter`, asked
    // at the event — the opposite of CR 615.11's fixed-at-resolution rows.
    #[test]
    fn safe_passage_covers_a_creature_that_entered_after_it_resolved() {
        let mut game = setup_two_player_game();
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        resolve_spell(&mut game, safe_passage(), 0, Vec::new());
        let latecomer = place_vanilla_creature(&mut game, 0, 4, 4, &[]);
        let theirs = place_vanilla_creature(&mut game, 1, 4, 4, &[]);
        bolt(&mut game, source, DamageTarget::Object(latecomer), 3);
        bolt(&mut game, source, DamageTarget::Object(theirs), 3);
        assert_eq!(game.battlefield[&latecomer].damage_marked, 0);
        assert_eq!(game.battlefield[&theirs].damage_marked, 3, "an opponent's creature is not covered");
    }

    // "Safe Passage doesn't prevent damage that would be dealt to
    // planeswalkers you control." The filter is `ByType(Creature)`.
    #[test]
    fn safe_passage_does_not_cover_a_planeswalker_you_control() {
        let mut game = setup_two_player_game();
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        let probe = put_on_battlefield(&mut game, loyalty_probe(), 0);
        resolve_spell(&mut game, safe_passage(), 0, Vec::new());
        bolt(&mut game, source, DamageTarget::Object(probe), 2);
        assert_eq!(game.battlefield[&probe].counter_count(CounterType::Loyalty), 1);
    }

    // "Safe Passage has no effect on damage that's already been dealt."
    #[test]
    fn safe_passage_has_no_effect_on_damage_already_dealt() {
        let mut game = setup_two_player_game();
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        let mine = place_vanilla_creature(&mut game, 0, 4, 4, &[]);
        bolt(&mut game, source, DamageTarget::Object(mine), 2);
        resolve_spell(&mut game, safe_passage(), 0, Vec::new());
        assert_eq!(game.battlefield[&mine].damage_marked, 2, "still marked");
        bolt(&mut game, source, DamageTarget::Object(mine), 2);
        assert_eq!(game.battlefield[&mine].damage_marked, 2, "and the next 2 are prevented");
    }

    // CR 113.7a — the row an ability makes names the permanent, which outlives
    // the stack object CR 608.2n deletes when the ability finishes resolving.
    // A real activation: `{T}` opens no mana window (CR 601.2g), so the one
    // prompt is the target.
    #[test]
    fn samite_healer_taps_for_a_count_of_one_and_the_row_names_the_healer() {
        use crate::ui::choice_types::ChoiceKind;
        use crate::ui::decision::ScriptedDecisionProvider;

        let mut game = setup_two_player_game();
        let healer = put_on_battlefield(&mut game, samite_healer(), 0);
        let dp = ScriptedDecisionProvider::new();
        // `Any` offers the players first; index 1 is player 1.
        dp.expect_pick_n(
            ChoiceKind::SelectRecipients { recipient: any_target(), spell_id: healer },
            vec![1],
        );
        game.activate_ability(0, healer, 0, &dp).expect("{T} is payable");
        assert!(game.battlefield[&healer].tapped);
        game.resolve_top_of_stack(&dp).unwrap();

        let rows = rows(&game);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].source, healer);
        assert!(game.objects.contains_key(&rows[0].source), "and it still exists");
        assert_eq!(rows[0].def.uses, Uses::NextDamage(1));
        assert_eq!(rows[0].def.affected_players, PlayerSet::Fixed(vec![1]));
    }

    // The cost is the card's: {W} and itself, in that order.
    #[test]
    fn samite_censer_bearer_costs_white_and_itself() {
        let card = samite_censer_bearer();
        let ability = &card.abilities[0];
        assert_eq!(ability.ability_type, AbilityType::Activated);
        assert_eq!(
            ability.costs,
            vec![Cost::Mana(ManaCost::build(&[ManaType::White], 0)), Cost::SacrificeSelf]
        );
        assert!(matches!(
            ability.effect,
            Effect::Atom(_, EffectRecipient::FilteredPermanents(_))
        ));
    }

    // -----------------------------------------------------------------------
    // RD-3 — the rulings pass
    // -----------------------------------------------------------------------

    use crate::engine::layers::types::{EffectModification, Layer};
    use crate::test_support::{lightning_bolt, put_spell_on_stack, registered};
    use crate::types::colors::Color as TestColor;
    use crate::types::replacement::SourcePattern as TestSourcePattern;

    /// Resolve `card`'s **activated** ability for `controller`, the way the
    /// stack would — CR 113.7a's source is the permanent, so the row an
    /// ability makes names it rather than the ephemeral stack object.
    fn activate(
        game: &mut GameState,
        card: Arc<CardData>,
        controller: crate::types::ids::PlayerId,
        dp: &dyn crate::ui::decision::DecisionProvider,
    ) -> crate::types::ids::ObjectId {
        let permanent = put_on_battlefield(game, card.clone(), controller);
        let ctx = ResolutionContext {
            source: permanent,
            ability_source: Some(permanent),
            controller,
            targets: Vec::new(),
            replaced_amount: None,
            damage_prevented: None,
        };
        game.resolve_effect(&card.abilities[0].effect, &ctx, dp).unwrap();
        permanent
    }

    /// Take the **first** option at every prompt.
    ///
    /// Every board below that reaches CR 609.7a's prompt puts the source it
    /// means at the oldest timestamp, so index 0 is that source — and every
    /// board that reaches a CR 616.1 prompt is one the ruling says either
    /// order answers the same way, which each test asserts rather than
    /// assumes.
    fn first() -> crate::test_support::RecordingDecisionProvider {
        crate::test_support::RecordingDecisionProvider::picking(0)
    }

    /// The chosen source on the one row in the registry.
    fn chosen_source(game: &GameState) -> Option<crate::types::ids::ObjectId> {
        match &rows(game)[0].def.pattern {
            EventPattern::DealDamage { source: Some(p), .. } => p.object,
            _ => None,
        }
    }

    fn life(game: &GameState, player: crate::types::ids::PlayerId) -> i64 {
        game.players[player].life_total
    }

    // CR 615.8 — "these effects prevent the next instance of damage from that
    // source, regardless of how much damage that is". Seven is prevented
    // whole, and the next instance from the same source is dealt in full.
    //
    // COVERS: ATOM-615.8-001
    #[test]
    fn circle_of_protection_red_prevents_the_whole_next_instance_however_big() {
        let mut game = setup_two_player_game();
        // A red 1/1 for the opponent, and nothing else that could be chosen
        // but the Circle itself — so the choice is forced and unasked.
        let red_source = put_on_battlefield(&mut game, lightning_bolt_creature(), 1);
        activate(&mut game, circle_of_protection_red(), 0, &first());
        assert_eq!(chosen_source(&game), Some(red_source), "CR 609.7a, at creation");

        bolt(&mut game, red_source, DamageTarget::Player(0), 7);
        assert_eq!(life(&game, 0), 20, "the whole instance, whatever its size");
        assert!(rows(&game).is_empty(), "and the row is spent (CR 615.8)");

        bolt(&mut game, red_source, DamageTarget::Player(0), 3);
        assert_eq!(life(&game, 0), 17, "a later instance is dealt normally");
    }

    // CR 615.9 / 609.7b — "when the source would deal damage, the shield
    // rechecks the source's properties. If the properties no longer match, the
    // damage isn't prevented or replaced ... the shield isn't used up."
    //
    // The id still matches; the colour does not, and that is what makes this
    // two fields rather than one.
    //
    // COVERS: ATOM-615.9-001
    // COVERS: ATOM-609.7b-001
    #[test]
    fn circle_of_protection_red_does_not_apply_and_is_not_spent_once_the_source_turns_blue() {
        let mut game = setup_two_player_game();
        let source = put_on_battlefield(&mut game, lightning_bolt_creature(), 1);
        activate(&mut game, circle_of_protection_red(), 0, &first());
        assert_eq!(chosen_source(&game), Some(source));

        let ts = game.allocate_timestamp();
        let mut blue = std::collections::HashSet::new();
        blue.insert(TestColor::Blue);
        game.continuous_effects.add(registered(
            source,
            Layer::Layer5Color,
            ts,
            EffectModification::SetColors(blue),
        ));

        bolt(&mut game, source, DamageTarget::Player(0), 3);
        assert_eq!(life(&game, 0), 17, "no longer red, so nothing is prevented");
        assert_eq!(rows(&game).len(), 1, "and the shield is not used up");
    }

    // The ruling: "Can be used even when there is no damage to prevent. It
    // prevents the next damage (if any) from the source this turn." So the row
    // is real, sits unused, and goes at the cleanup step rather than at zero.
    #[test]
    fn circle_of_protection_red_can_be_activated_with_no_damage_to_prevent() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, lightning_bolt_creature(), 1);
        activate(&mut game, circle_of_protection_red(), 0, &first());
        assert_eq!(rows(&game).len(), 1);
        assert_eq!(rows(&game)[0].def.uses, Uses::Once);

        crate::test_support::pass_turn(&mut game);
        assert!(rows(&game).is_empty(), "CR 615.3 — the duration expired");
    }

    // The ruling: "It only affects damage dealt by the source one time. If the
    // source damages you a second time this turn, the damage will not be
    // reversed." And the rider is the printed reader of RD-2's prevented
    // channel: 4 prevented is 4 life gained, once.
    #[test]
    fn reverse_damage_reverses_the_first_instance_only() {
        let mut game = setup_two_player_game();
        let source = put_on_battlefield(&mut game, lightning_bolt_creature(), 1);
        resolve_spell(&mut game, reverse_damage(), 0, Vec::new());
        assert_eq!(chosen_source(&game), Some(source));

        bolt(&mut game, source, DamageTarget::Player(0), 4);
        assert_eq!(life(&game, 0), 24, "prevented, and gained that much");

        bolt(&mut game, source, DamageTarget::Player(0), 4);
        assert_eq!(life(&game, 0), 20, "the second instance is not reversed");
    }

    // The ruling: "if the source would deal 5 damage after two of these
    // abilities have resolved, the first one prevents 2 damage, reducing it to
    // 3 damage, then the second one prevents a further 1 damage, reducing the
    // total damage dealt to 2." Two rows, each applied once (CR 614.5), and
    // the order does not matter because both halve the same way.
    #[test]
    fn two_dark_spheres_take_five_to_three_to_two() {
        let mut game = setup_two_player_game();
        let source = put_on_battlefield(&mut game, lightning_bolt_creature(), 1);
        activate(&mut game, dark_sphere(), 0, &first());
        activate(&mut game, dark_sphere(), 0, &first());
        assert_eq!(rows(&game).len(), 2);

        // Two candidates, so CR 616.1 asks — and either answer is this one,
        // because both rows halve the same way.
        let dp = first();
        game.execute_action(
            GameAction::DealDamage {
                source,
                target: DamageTarget::Player(0),
                amount: 5,
                is_combat: false,
                unpreventable: false
            },
            &ActionContext::new(&dp),
        )
        .unwrap();
        assert_eq!(life(&game, 0), 18, "5 - 2 - 1");
        assert!(rows(&game).is_empty(), "both spent");
    }

    // The ruling: "This ability prevents 1 damage from each source an opponent
    // controls each time that source would deal damage to you. It prevents 1
    // of any damage, not just combat damage." Two sources, one point off each.
    #[test]
    fn guardian_seraph_prevents_one_from_each_opposing_source() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, guardian_seraph(), 0);
        let first = place_vanilla_creature(&mut game, 1, 3, 3, &[]);
        let second = place_vanilla_creature(&mut game, 1, 3, 3, &[]);

        bolt(&mut game, first, DamageTarget::Player(0), 3);
        bolt(&mut game, second, DamageTarget::Player(0), 3);
        assert_eq!(life(&game, 0), 20 - 2 - 2, "1 off each event, not 1 in total");
    }

    // The other half of the source predicate: your own creature hitting you is
    // not "a source an opponent controls".
    #[test]
    fn guardian_seraph_does_not_prevent_damage_from_your_own_source() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, guardian_seraph(), 0);
        let yours = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
        bolt(&mut game, yours, DamageTarget::Player(0), 3);
        assert_eq!(life(&game, 0), 17);
    }

    // The ruling: "The effects from multiple Guardian Seraphs are cumulative."
    // Two instances, each applied once (CR 614.5), so 3 becomes 1.
    #[test]
    fn two_guardian_seraphs_are_cumulative() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, guardian_seraph(), 0);
        put_on_battlefield(&mut game, guardian_seraph(), 0);
        let source = place_vanilla_creature(&mut game, 1, 3, 3, &[]);

        // Two candidates on one event: CR 616.1 asks, and both answers agree.
        let dp = first();
        game.execute_action(
            GameAction::DealDamage {
                source,
                target: DamageTarget::Player(0),
                amount: 3,
                is_combat: false,
                unpreventable: false
            },
            &ActionContext::new(&dp),
        )
        .unwrap();
        assert_eq!(life(&game, 0), 19, "3 - 1 - 1");
    }

    // CR 609.7c — "the prevention or replacement applies to sources that are
    // permanents with that property **and to any sources that aren't on the
    // battlefield** that have that property." Lightning Bolt is a spell on the
    // stack: not a permanent, and inside Guardian Seraph's predicate because
    // an opponent controls it.
    //
    // COVERS: ATOM-609.7c-001
    #[test]
    fn guardian_seraph_reaches_a_spell_on_the_stack() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, guardian_seraph(), 0);
        let spell = put_spell_on_stack(&mut game, lightning_bolt(), 1);
        bolt(&mut game, spell, DamageTarget::Player(0), 3);
        assert_eq!(life(&game, 0), 18, "a non-battlefield source, 3 - 1");
    }

    // Any source at all, and the target side does the work: a Cleric creature
    // you control takes one less. The CR's own Pyroclasm board is
    // `tests/phase_rd3_integration_test.rs`.
    #[test]
    fn daunting_defender_prevents_one_to_a_cleric_you_control() {
        let mut game = setup_two_player_game();
        let defender = put_on_battlefield(&mut game, daunting_defender(), 0);
        let not_a_cleric = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);

        bolt(&mut game, source, DamageTarget::Object(defender), 2);
        bolt(&mut game, source, DamageTarget::Object(not_a_cleric), 2);
        assert_eq!(game.battlefield[&defender].damage_marked, 1, "it is its own Cleric");
        assert_eq!(game.battlefield[&not_a_cleric].damage_marked, 2);
    }

    // The `combat` field's whole job, from both sides: combat damage is
    // prevented and non-combat damage is not.
    #[test]
    fn fog_prevents_combat_damage_and_lets_a_bolt_through() {
        let mut game = setup_two_player_game();
        resolve_spell(&mut game, fog(), 0, Vec::new());
        let source = place_vanilla_creature(&mut game, 1, 3, 3, &[]);

        game.execute_action(
            GameAction::DealDamage {
                source,
                target: DamageTarget::Player(0),
                amount: 3,
                is_combat: true,
                unpreventable: false
            },
            &test_ctx(),
        )
        .unwrap();
        assert_eq!(life(&game, 0), 20, "combat damage");

        bolt(&mut game, source, DamageTarget::Player(0), 3);
        assert_eq!(life(&game, 0), 17, "and non-combat damage goes through");
    }

    // Fog says "all", and its own caster's opponents are inside it: a creature
    // an opponent controls takes no combat damage either.
    #[test]
    fn fog_covers_every_permanent_and_every_player() {
        let mut game = setup_two_player_game();
        resolve_spell(&mut game, fog(), 0, Vec::new());
        let mine = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
        let theirs = place_vanilla_creature(&mut game, 1, 2, 2, &[]);

        for (source, target) in [(mine, theirs), (theirs, mine)] {
            game.execute_action(
                GameAction::DealDamage {
                    source,
                    target: DamageTarget::Object(target),
                    amount: 2,
                    is_combat: true,
                    unpreventable: false
                },
                &test_ctx(),
            )
            .unwrap();
        }
        assert_eq!(game.battlefield[&mine].damage_marked, 0);
        assert_eq!(game.battlefield[&theirs].damage_marked, 0);
    }

    // The ruling: "The additional 2 damage is dealt by the same source as the
    // original source of damage. The damage isn't dealt by Torbran unless
    // Torbran is the original source of damage."
    #[test]
    fn torbran_adds_two_and_the_damage_is_still_the_original_sources() {
        let mut game = setup_two_player_game();
        let torbran = put_on_battlefield(&mut game, torbran_thane_of_red_fell(), 0);
        let red_source = put_on_battlefield(&mut game, lightning_bolt_creature(), 0);

        bolt(&mut game, red_source, DamageTarget::Player(1), 3);
        assert_eq!(life(&game, 1), 15, "3 + 2");
        let dealt: Vec<_> = game
            .events
            .events()
            .filter_map(|e| match e {
                GameEvent::DamageDealt { source_id, amount, .. } => Some((*source_id, *amount)),
                _ => None,
            })
            .collect();
        assert_eq!(dealt, vec![(red_source, 5)]);
        assert!(!dealt.iter().any(|(s, _)| *s == torbran), "Torbran deals none of it");
    }

    // Both sides of the predicate, each tested by the board that fails it: a
    // source you control that is not red, and a red source you do not control.
    #[test]
    fn torbran_asks_both_halves_of_its_source_predicate() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, torbran_thane_of_red_fell(), 0);
        let yours_not_red = place_vanilla_creature(&mut game, 0, 1, 1, &[]);
        let red_not_yours = put_on_battlefield(&mut game, lightning_bolt_creature(), 1);

        bolt(&mut game, yours_not_red, DamageTarget::Player(1), 3);
        assert_eq!(life(&game, 1), 17, "colorless: not a red source");
        bolt(&mut game, red_not_yours, DamageTarget::Player(1), 3);
        assert_eq!(life(&game, 1), 14, "red, but not yours");
    }

    // The ruling: "If all of the damage is prevented, Torbran's effect no
    // longer applies." A whole-event prevention leaves a 0-damage proposal,
    // CR 614.7a drops it on the next iteration, and Torbran is never gathered
    // — so nothing is added to nothing.
    #[test]
    fn torbran_does_not_apply_once_all_of_the_damage_is_prevented() {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, torbran_thane_of_red_fell(), 0);
        let red_source = put_on_battlefield(&mut game, lightning_bolt_creature(), 0);
        // Fog, cast by the player about to be hit, over combat damage.
        resolve_spell(&mut game, fog(), 1, Vec::new());

        // Fog and Torbran both apply, so CR 616.1 asks the damaged player for
        // an order — the ruling's own sentence. Either answer is 0: Fog first
        // empties the event and CR 614.7a drops it before Torbran is gathered;
        // Torbran first makes it 5 and Fog prevents all 5.
        let before = damage_dealt(&game);
        let dp = first();
        game.execute_action(
            GameAction::DealDamage {
                source: red_source,
                target: DamageTarget::Player(1),
                amount: 3,
                is_combat: true,
                unpreventable: false
            },
            &ActionContext::new(&dp),
        )
        .unwrap();
        assert_eq!(life(&game, 1), 20, "prevented, and nothing added to it");
        assert_eq!(damage_dealt(&game), before, "no damage event at all");
    }

    // A red creature, so a `SourcePattern`'s colour clause has a printed board
    // to be asked on. Not a registered card: it exists to *be* a source.
    fn lightning_bolt_creature() -> Arc<CardData> {
        CardDataBuilder::new("Red Probe")
            .mana_cost(ManaCost::build(&[ManaType::Red], 0))
            .color(Color::Red)
            .card_type(CardType::Creature)
            .power_toughness(1, 1)
            .rules_text("")
            .build()
    }

    // The `SourcePattern` a card writes before a resolution touches it: the
    // object empty, the filter the card's. `PatternFill::ChosenDamageSource`
    // asserts on the first half.
    #[test]
    fn a_chosen_source_card_authors_no_object() {
        for card in [circle_of_protection_red(), reverse_damage(), dark_sphere()] {
            let Effect::Atom(Primitive::CreateReplacement(def, _, fill), _) =
                &card.abilities[0].effect
            else {
                panic!("{} is a CreateReplacement", card.name);
            };
            assert_eq!(*fill, PatternFill::ChosenDamageSource);
            match &def.pattern {
                EventPattern::DealDamage { source: Some(p), .. } => {
                    assert_eq!(p.object, None, "{} authors no source id", card.name)
                }
                other => panic!("{} watches {:?}", card.name, other),
            }
        }
        // And the two that ask nothing say so.
        assert_eq!(
            TestSourcePattern::chosen(),
            TestSourcePattern { object: None, filter: None }
        );
    }
}
