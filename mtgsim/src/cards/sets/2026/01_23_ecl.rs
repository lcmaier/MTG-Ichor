//! Lorwyn Eclipsed (`ecl`), released 2026-01-23.

use std::sync::Arc;

use crate::cards::authoring::{copy, land, non, spell_ability, target_permanent, with_subtype, you_control};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, EnchantmentType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::CopyRoles;
use crate::types::mana::{ManaCost, ManaType};

/// Mirrorform — {4}{U}{U}
/// Instant
///
/// Each nonland permanent you control becomes a copy of target non-Aura
/// permanent.
///
/// (Oracle text verified on Scryfall, 2026-09-02.)
///
/// # Why this card is here, and what it cost to find
///
/// **It is the counter-example that turned an exclusion from structure into
/// data.** CV-1 first shipped `CopyRoles::OthersCopyRecipient`, on the reading
/// that a class-scoped copy always means "each *other*" — so the exclusion of
/// the donor could be structural and no card would have to spell it. Mirrorform
/// says "each nonland permanent **you control**", which *includes* the target
/// whenever you control it, and the old arm could not express that at all. Found
/// in review, not by the census: `copy-census.py` partitions by mechanism and a
/// one-word difference inside one mechanism is exactly what it cannot see.
///
/// A permanent copying itself is very nearly a no-op — the capture is its own
/// post-layer-1 state. Not exactly one, and CR-correctly so: the row carries its
/// own `Duration`, so it holds those values past the expiry of an earlier copy
/// row that put them there.
///
/// # No duration, so until the end of the game
///
/// The card states none, and CR 611.2a makes such an effect last "until the
/// end of the game". It was spelled `UntilEndOfTurn` until CV-1b, because an
/// indefinite row is reached by neither CR 514.2's expiry nor
/// `remove_rows_ending_with`; CR 400.7 is what ends one now, when its subject
/// moves.
///
/// # Registered, not pooled
///
/// It opens no engine path Mirrorweave does not, so it stays out of
/// `PERFORMANCE_POOL` for the reason given there. Its job is to keep
/// `exclude_donor: false` from being scaffolding with no consumer.
///
/// # The rulings, and where each is tested
///
/// All four are in `tests/phase_cv1b_integration_test.rs`: nothing enters,
/// so no "enters" ability applies (1); only the printed values are copied
/// (2); a copy of a copy copies what it copied (3); X is 0 (4).
pub fn mirrorform() -> Arc<CardData> {
    CardDataBuilder::new("Mirrorform")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 4))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(
            "Each nonland permanent you control becomes a copy of target \
             non-Aura permanent.",
        )
        .ability(spell_ability(
            "Each nonland permanent you control becomes a copy of target non-Aura permanent.",
            copy(
                // "target non-Aura permanent" — the donor, and the only place
                // in CV-1 where a copy source is not required to be a creature.
                target_permanent(non(with_subtype(Subtype::Enchantment(EnchantmentType::Aura)))),
                CopyRoles::FilteredCopyRecipient {
                    // "Each nonland permanent you control" — and no
                    // "other", which is the whole reason this card is
                    // registered.
                    filter: non(land()).and(you_control()),
                    exclude_donor: false,
                },
                Vec::new(),
            )
            .until_end_of_game(),
        ))
        .build()
}
