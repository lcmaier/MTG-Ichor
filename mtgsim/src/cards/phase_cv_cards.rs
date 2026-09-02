//! Cards for Phase CV-1 — the copy spine (CR 707, layer 1a).
//!
//! **Two cards, and they are a pair for a structural reason rather than a
//! thematic one.** `CopyRoles` has two arms because these two cards bind the
//! atom's target to opposite roles: Cytoshape targets the permanent that
//! *becomes* a copy and chooses its donor, Mirrorweave targets the donor and
//! affects everything else. A phase that shipped only one would have defined
//! the primitive under half the pressure and found the other half in CV-2.
//!
//! Neither is a Clone: CR 707.5's "enters as a copy" is an entry replacement
//! and belongs to CV-2, which needs RC-2's `EnterBattlefield` event. The
//! Clone-shaped board CV-1 does need — a copy of a permanent with a static
//! continuous ability, which is `copy-effects-architecture.md` §4.7 leg 2 — is
//! reachable with Cytoshape and Glorious Anthem, both already registered, and
//! so needs no third card.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{
    CopyRoles, Duration, Effect, EffectRecipient, PermanentFilter, Primitive, SelectionFilter,
    TargetCount,
};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};

/// "Nonlegendary creature" — the filter both cards scope their copy source
/// with, and the reason CR 707 cards say it at all: a copy of a legend meets
/// CR 704.5j the moment it exists, so the printed text keeps the effect from
/// being a sacrifice.
fn nonlegendary_creature() -> PermanentFilter {
    PermanentFilter::And(
        Box::new(PermanentFilter::ByType(CardType::Creature)),
        Box::new(PermanentFilter::Not(Box::new(PermanentFilter::BySupertype(
            Supertype::Legendary,
        )))),
    )
}

/// Cytoshape — {1}{G}{U}
/// Instant
///
/// Choose a nonlegendary creature on the battlefield. Target creature becomes
/// a copy of that creature until end of turn.
///
/// (Oracle text verified on Scryfall, 2026-09-02.)
///
/// # The card the phase is sized against
///
/// `copy-effects-architecture.md` §7 names it CV-1's consumer, and it is the
/// minimum board that exercises the whole spine: a resolution capture (CR
/// 707.2), an `AffectedSet::Fixed` locked as the effect begins (CR 611.2c), a
/// turn-bounded `Duration`, and a CR 707.4 *choice* that is not a target.
///
/// # Two selections, and only one of them is targeting
///
/// The **target** is what becomes a copy: hexproof and shroud apply to it, and
/// CR 608.2b makes the spell do nothing if it has left the battlefield. The
/// **choice** is the donor: CR 707.4 says "choose", so protection does not
/// apply and nothing fizzles. The engine keeps them apart by putting the target
/// in the atom's `EffectRecipient` and the donor's filter inside `CopyRoles`,
/// which is what the enum exists for.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// It is the only card in the crate that can put a row in layer 1, and a gated
/// subsystem no pooled card can open is the failure RS-1 taught
/// (`registry.rs`'s `PERFORMANCE_POOL` doc). Registering it alone would leave
/// the A/B measuring an engine path no measured game ever walks. Mirrorweave is
/// deliberately *not* in the pool: the two cards open the same path, and a
/// second copy of a path buys a slower fuzz run rather than a wider one.
pub fn cytoshape() -> Arc<CardData> {
    CardDataBuilder::new("Cytoshape")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Blue], 1))
        .color(Color::Green)
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(
            "Choose a nonlegendary creature on the battlefield. Target creature \
             becomes a copy of that creature until end of turn.",
        )
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::Copy(
                    CopyRoles::RecipientsCopyChosen(SelectionFilter::Permanent(
                        nonlegendary_creature(),
                    )),
                    Duration::UntilEndOfTurn,
                ),
                EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
            ),
        })
        .build()
}

/// Mirrorweave — {2}{W/U}{W/U}
/// Instant
///
/// Each other creature becomes a copy of target nonlegendary creature until
/// end of turn.
///
/// (Oracle text verified on Scryfall, 2026-09-02.)
///
/// # Why the second card, and what it measures
///
/// It is `copy-effects-architecture.md` §9 item 4's named customer — one
/// capture applied to a whole class — and CV-1's answer to that question comes
/// from what this card actually builds: **one row, not one per creature.** CR
/// 611.2c locks the affected set as the effect begins, so the row is a single
/// `AffectedSet::Fixed` carrying a single `Box<CopiableValues>`. `Box` and `Arc`
/// allocate identically here, and the phases that would tell them apart are the
/// ones that create a row *per object* (CV-2's entry replacements, the
/// class-scoped statics).
///
/// It also makes the second half of §4.7 leg 2 observable: a copied static
/// ability's row takes **the copying permanent's** controller, not the spell's,
/// and "each other creature" reaches creatures both players control. Copy a
/// Glorious Anthem with Cytoshape and one controller answers for both; copy it
/// with Mirrorweave and they diverge.
///
/// # The hybrid cost, and a precedent this card deliberately does not follow
///
/// `{2}{W/U}{W/U}` is two generic plus two hybrid pips. `ManaSymbol::Hybrid`
/// exists, but no payment path handles it — `mana_helpers`' auto-tap returns
/// `None` on any non-`Colored`/`Generic` symbol — so a verbatim cost would make
/// the card *silently uncastable*, which is this project's named worst outcome.
/// Registered as **`{2}{W}{U}`**: a strict **subset** of the real card's legal
/// payments (it forbids WW and UU), so no game reaches a state the printed card
/// forbids, and CR 202.2's colours come out W and U exactly as printed.
///
/// `codebase-state.md`'s 2026-08-24 entry set the opposite precedent —
/// `phase5_pre_cards::inside_out` stays unregistered because simplifying its
/// hybrid cost would "misrepresent the card". That was the right call there and
/// is the wrong one here, for a reason that is about the *pool* rather than
/// about the card: Inside Out had a registered substitute for the engine path it
/// covered, and Mirrorweave is the only consumer of `CopyRoles`'
/// `OthersCopyRecipient` arm in the crate. Leaving it out would ship an arm with
/// no random-play coverage, which is the failure `PERFORMANCE_POOL`'s own doc
/// records RS-1 for. Recorded rather than quietly done: the first real hybrid
/// payment path should delete this paragraph and the approximation together.
pub fn mirrorweave() -> Arc<CardData> {
    CardDataBuilder::new("Mirrorweave")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::Blue], 2))
        .color(Color::White)
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(
            "Each other creature becomes a copy of target nonlegendary creature \
             until end of turn.",
        )
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                // "Each **other** creature" — the exclusion of the donor is
                // structural in `OthersCopyRecipient`, so the filter is plain
                // "creature" and cannot be written to include the target.
                Primitive::Copy(
                    CopyRoles::OthersCopyRecipient(PermanentFilter::ByType(CardType::Creature)),
                    Duration::UntilEndOfTurn,
                ),
                // The target is the *donor* here, and it is what the printed
                // "nonlegendary" scopes.
                EffectRecipient::Target(
                    SelectionFilter::Permanent(nonlegendary_creature()),
                    TargetCount::Exactly(1),
                ),
            ),
        })
        .build()
}
