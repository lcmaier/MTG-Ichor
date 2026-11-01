//! Nemesis (`nem`), released 2000-02-14.

use std::sync::Arc;

use crate::cards::authoring::{basic, create_replacement, land, non, spell_ability, tapped_for_mana};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::PlayerSet;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{GameActionTemplate, ReplacementDef, Rewrite, TemplateAmount};

/// Pale Moon — {1}{U}
/// Instant
///
/// > Until end of turn, if a player taps a nonbasic land for mana, it
/// > produces colorless mana instead of any other type.
///
/// **The second registrable type-changer, found at review** — the census
/// regex read the card's phrase "tap … for mana" and this card says "taps a
/// nonbasic land for mana" (`replacement-architecture.md` §11 item 98). Deep
/// Water's shape on an instant: Fog's `CreateReplacement` until end of turn,
/// `PlayerSet::Everyone` for "a player", and "nonbasic land" as the filter on
/// the tapped permanent. Registered and unpooled: a one-shot whose engine path
/// Deep Water already opens.
///
/// # The rulings, and where each is tested
///
/// - *"The ability does not change the amount of mana produced, only the
///   color."* → `ReplacedAmount`: an opponent's two-mana nonbasic land
///   produces two colorless, and a basic Forest is left alone
///   (`pale_moon_retypes_any_players_nonbasic_land_and_leaves_a_basic_alone`).
pub fn pale_moon() -> Arc<CardData> {
    CardDataBuilder::new("Pale Moon")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text(
            "Until end of turn, if a player taps a nonbasic land for mana, it produces colorless mana instead of any other type.",
        )
        .ability(spell_ability(
            "Until end of turn, if a player taps a nonbasic land for mana, it produces colorless mana instead of any other type.",
            create_replacement(ReplacementDef::new(
                tapped_for_mana(Some(land().and(non(basic())))),
                PlayerSet::Everyone,
                Rewrite::Instead(GameActionTemplate::ProduceMana { mana_type: ManaType::Colorless, amount: TemplateAmount::ReplacedAmount }),
            ))
            .until_end_of_turn(),
        ))
        .build()
}
