//! War of the Spark (`war`), released 2019-05-03.

use std::sync::Arc;

use crate::cards::authoring::{creature, planeswalker, static_replacement, you_control};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{CharacteristicEdit, CopyException, CounterType, ObjectSet, SelectionFilter, TypeChange};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{CopyDonor, EnterModsTemplate, EntryCopyTemplate, EventPattern, ReplacementDef, Rewrite};

/// Spark Double — {3}{U}
/// Creature — Illusion, 0/0
///
/// You may have this creature enter as a copy of a creature or planeswalker
/// you control, except it enters with an additional +1/+1 counter on it if
/// it's a creature, it enters with an additional loyalty counter on it if
/// it's a planeswalker, and it isn't legendary.
///
/// (Oracle text verified on Scryfall, 2026-09-29.)
///
/// # CV-2b's card: CR 707.9's exceptions on Clone's entry copy
///
/// Three exceptions, in printed order, and each is one arm of
/// `CopyException` (`copy-effects-architecture.md` §4.1a): two CR 707.9f
/// conditions, each over a CR 707.9e additional counter, and a CR 707.9b
/// edit that is part of the copiable values, so a copy of this is not
/// legendary either. The conditions read the CR 614.12 frame of the copy,
/// each checked without itself and with the other where the other applies,
/// which is what the words say and what a planeswalker whose type hangs on
/// its counters makes observable (§4.1a, "The Kaito board").
///
/// # The rulings, and where each is tested
///
/// All nine are linked from `tests/phase_cv2b_integration_test.rs`: what is
/// not copied (1), not legendary, and a copy of it neither (2), X as 0 (3), a
/// copy's copy (4), a token's values without being a token (5), the copied
/// "enters" abilities and triggers (6), printed loyalty plus one (7, a
/// `// RULING-DEVIATION:` on Kaito's board, by the register row
/// `lookahead-entry-counters`), the characteristics as it enters, not the
/// donor's (8), and nothing entering beside it (9).
pub fn spark_double() -> Arc<CardData> {
    let additionally = |counter| CopyException::Additionally(EnterModsTemplate::with_counters(counter, 1));
    let creature_or_planeswalker_you_control = creature().or(planeswalker()).and(you_control());
    CardDataBuilder::new("Spark Double")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 3))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Illusion))
        .power_toughness(0, 0)
        .rules_text(
            "You may have this creature enter as a copy of a creature or planeswalker you control, \
             except it enters with an additional +1/+1 counter on it if it's a creature, it enters \
             with an additional loyalty counter on it if it's a planeswalker, and it isn't legendary.",
        )
        .ability(static_replacement(
            "You may have this creature enter as a copy of a creature or planeswalker you control, except it enters with an additional +1/+1 counter on it if it's a creature, it enters with an additional loyalty counter on it if it's a planeswalker, and it isn't legendary.",
            ReplacementDef {
                optional: true,
                ..ReplacementDef::new(
                    EventPattern::EnterBattlefield { cast: None },
                    ObjectSet::SourceOnly,
                    Rewrite::EnterAsCopy(EntryCopyTemplate {
                        donor: CopyDonor::Chosen(SelectionFilter::Permanent(creature_or_planeswalker_you_control)),
                        except: vec![
                            CopyException::If(
                                creature(),
                                vec![additionally(CounterType::PlusOnePlusOne)],
                            ),
                            CopyException::If(
                                planeswalker(),
                                vec![additionally(CounterType::Loyalty)],
                            ),
                            CopyException::Modifies(CharacteristicEdit::Types(TypeChange {
                                remove_supertypes: vec![Supertype::Legendary],
                                ..TypeChange::NONE
                            })),
                        ],
                    }),
                )
            },
        ))
        .build()
}
