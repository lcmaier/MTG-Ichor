//! Dissension (`dis`), released 2006-05-05.

use std::sync::Arc;

use crate::cards::authoring::{copy, nonlegendary_creature, spell_ability, target_creature};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{CopyRoles, SelectionFilter};
use crate::types::mana::{ManaCost, ManaType};

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
/// 707.2), an `ObjectSet::Fixed` locked as the effect begins (CR 611.2c), a
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
        .ability(spell_ability(
            "Choose a nonlegendary creature on the battlefield. Target creature becomes a copy of that creature until end of turn.",
            copy(target_creature(), CopyRoles::RecipientsCopyChosen(SelectionFilter::Permanent(nonlegendary_creature())), Vec::new())
                .until_end_of_turn(),
        ))
        .build()
}
