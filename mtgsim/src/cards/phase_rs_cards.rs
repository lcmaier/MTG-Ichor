//! Cards for Phase RS-1 — the Tier-2 "can't" spine (CR 101.2, 614.17).
//!
//! **Two cards, and they are a pair.** RS-1's other two consumers have no card
//! text at all — indestructible is CR 702.12b's words and "can't be
//! regenerated" is CR 701.19c's — so a phase that shipped only those would have
//! defined `Restriction` under no pressure and found out later what it could not
//! express. Sigarda is the pressure, and Diabolic Edict is what makes her
//! observable: without a resolution that asks a player to choose a permanent,
//! "no prompt at all" and "a prompt nobody answered" are the same board.
//!
//! Sigarda is `cant-effects-architecture.md` §2.6's motivating card for the one
//! field the model added — a *source* filter — and §4.9's motivating card for
//! the candidate filter. She is the only shape in RS-1's reach that needs both.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{
    AffectedSet, Effect, EffectRecipient, PermanentFilter, PlayerRef, Primitive, SelectionFilter,
    TargetCount,
};
use crate::types::ids::new_ability_id;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::EventPattern;
use crate::types::restriction::{Restriction, RestrictionDef, SourceFilter};
use crate::types::zones::{Zone, ZoneChangeCause};

/// Sigarda, Host of Herons — {2}{G}{W}{W}
/// Legendary Creature — Angel, 5/5
///
/// Flying, hexproof
/// Spells and abilities your opponents control can't cause you to sacrifice
/// permanents.
///
/// (Oracle text verified on Scryfall, 2026-08-31.)
///
/// # Why this clause is Tier 2 and not a cost restriction
///
/// A census regex that saw "sacrifice" filed it under CR 118's costs, and that
/// is wrong: **you never pay costs for an opponent's spell**
/// (`cant-effects-architecture.md` §2.6). Diabolic Edict resolving against you
/// is an *effect*, and what Sigarda forbids is the `ZoneChange { cause:
/// Sacrificed }` it would propose. Tajuru Preserver prints the identical
/// sentence; The Master, Multiplied and Tamiyo, Collector of Tales are the same
/// shape with a different event.
///
/// # The three halves of the restriction, and where each one lives
///
/// - **"can't cause you to sacrifice"** is the *event*:
///   `EventPattern::ZoneChange { from: Battlefield, to: Graveyard, cause:
///   Sacrificed }`. CR 701.21a's sacrifice, and specifically not CR 701.8's
///   destruction — 701.21b says a sacrificed permanent is not destroyed, which
///   is why the cause is its own variant and why Sigarda stops none of the
///   CR 704.5 state-based actions.
/// - **"permanents"**, scoped to *yours*, is the `AffectedSet`:
///   `ByController(PlayerRef::You)`, resolved against Sigarda's **current**
///   controller at the instant the question is asked (CR 109.5). Stealing
///   Sigarda moves the protection, and the filter being stored unresolved is
///   what makes that free.
/// - **"spells and abilities your opponents control"** is the `by` filter — the
///   one thing this arm needed beyond `EventPattern` + `AffectedSet`, and the
///   reason it exists at all. Read against the resolution `ActionContext`
///   already threads, so it is a field rather than a mechanism.
///
/// # Why she must produce *no prompt*, which is a rule and not a nicety
///
/// Her printed rulings are unusually explicit — "if it would force you to
/// sacrifice a permanent, **you just don't**", and of an optional sacrifice,
/// "**you can't take that option**". So a Diabolic Edict resolving against her
/// controller must not ask. Prompting and then refusing would violate CR 608.2d
/// ("the player can't choose an option that's illegal or impossible"), would
/// leak which permanent you would have picked, and would make an AI harness
/// spend a decision on a branch that cannot happen. §4.9 has the argument; the
/// filter is in `resolve.rs::sacrifice_one_of_choice`.
///
/// # What is deliberately absent
///
/// **Hexproof**, which she prints, is Tier 1d — a restriction on *targeting*,
/// which is a choice rather than an event, and is RS-2's `BeTargeted` arm
/// (`codebase-state.md` item 15: the `KeywordFlag` is constructible and enforced
/// nowhere). Flying is on the card because `KeywordFlag::Flying` is enforced.
/// Shipping her without hexproof is the Blood Moon precedent: the clause the
/// phase is about, and no more.
pub fn sigarda_host_of_herons() -> Arc<CardData> {
    CardDataBuilder::new("Sigarda, Host of Herons")
        .mana_cost(ManaCost::build(
            &[ManaType::Green, ManaType::White, ManaType::White],
            2,
        ))
        .color(Color::Green)
        .color(Color::White)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Angel))
        .power_toughness(5, 5)
        .keyword(KeywordFlag::Flying)
        .rules_text(
            "Flying\n\
             Spells and abilities your opponents control can't cause you to \
             sacrifice permanents.",
        )
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Restriction(Box::new(RestrictionDef::new(Restriction::Event {
                // CR 701.21a — "its controller moves it from the battlefield
                // directly to its owner's graveyard".
                pattern: EventPattern::ZoneChange {
                    from: Some(Zone::Battlefield),
                    to: Some(Zone::Graveyard),
                    cause: Some(ZoneChangeCause::Sacrificed),
                    object: None,
                },
                affected: AffectedSet::Filter {
                    filter: PermanentFilter::ByController(PlayerRef::You),
                },
                by: Some(SourceFilter::ControlledBy(PlayerRef::Opponent)),
            }))),
        })
        .build()
}

/// Diabolic Edict — {1}{B}
/// Instant
///
/// Target player sacrifices a creature of their choice.
///
/// (Oracle text verified on Scryfall, 2026-08-31.)
///
/// # "Of their choice" is the load-bearing phrase
///
/// The *target* chooses, not the caster — which is what makes this the
/// resolution-time selection path §4.9 puts its candidate filter in, and what
/// makes it the honest test of Sigarda. An edict where the caster picked would
/// exercise a different rule and would not distinguish "no prompt" from "a
/// prompt asked of the wrong player".
///
/// It also carries CR 608.2d for free: a targeted player who controls no
/// creatures is asked nothing, and no error is raised. That is the same empty
/// candidate list Sigarda produces, which is `cant-effects-architecture.md`
/// §4.9's whole claim — a restriction is a *second reason a candidate is
/// unavailable*, not a second mechanism.
///
/// # In the stress pool, not the frozen one
///
/// Registered, so `fuzz_games --pool stress` plays both cards; absent from
/// `PERFORMANCE_POOL`, so the recorded A/B baseline still measures the same
/// board it always did.
pub fn diabolic_edict() -> Arc<CardData> {
    CardDataBuilder::new("Diabolic Edict")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Instant)
        .rules_text("Target player sacrifices a creature of their choice.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                // What is sacrificed: a creature.
                Primitive::Sacrifice(SelectionFilter::Creature),
                // Who sacrifices it: the target (CR 115.1). Two filters because
                // they are two questions — see `Primitive::Sacrifice`.
                EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
            ),
        })
        .build()
}
