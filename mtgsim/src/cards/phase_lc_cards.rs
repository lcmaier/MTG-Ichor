use std::sync::Arc;

use crate::cards::authoring::{
    becomes_color, becomes_colorless, creatures_you_control, draw_cards, each, gains_keyword, is_also_color, spell_ability, static_ability, target_creature,
    you,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::mana::{ManaCost, ManaType};

/// Cerulean Wisps — {U}
/// Instant
/// Target creature becomes blue until end of turn.
/// Draw a card.
pub fn cerulean_wisps() -> Arc<CardData> {
    CardDataBuilder::new("Cerulean Wisps")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text("Target creature becomes blue until end of turn. Untap that creature.\nDraw a card.")
        .ability(spell_ability(
            "Target creature becomes blue until end of turn. Untap that creature.",
            becomes_color(target_creature(), [Color::Blue]).until_end_of_turn(),
        ))
        .ability(spell_ability(
            "Draw a card.",
            draw_cards(you(), 1),
        ))
        .build()
}

/// Moonlace — {U}
/// Instant
/// Target spell or permanent becomes colorless.
/// (Simplified: target creature becomes colorless until end of turn.)
pub fn moonlace() -> Arc<CardData> {
    CardDataBuilder::new("Moonlace")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text("Target spell or permanent becomes colorless.")
        .ability(spell_ability(
            "Target spell or permanent becomes colorless.",
            becomes_colorless(target_creature()).until_end_of_turn(),
        ))
        .build()
}

/// Crimson Wisps — {R}
/// Instant
/// Target creature gains haste and becomes red until end of turn.
/// Draw a card.
/// (Tests multi-layer effect from one spell: Layer 5 color + Layer 6 keyword)
pub fn crimson_wisps() -> Arc<CardData> {
    use crate::types::keywords::KeywordFlag;

    CardDataBuilder::new("Crimson Wisps")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .rules_text("Target creature becomes red and gains haste until end of turn.\nDraw a card.")
        .ability(spell_ability(
            "Target creature becomes red and gains haste until end of turn.",
            gains_keyword(target_creature(), KeywordFlag::Haste).and_becomes_color([Color::Red]).until_end_of_turn(),
        ))
        .ability(spell_ability(
            "Draw a card.",
            draw_cards(you(), 1),
        ))
        .build()
}

/// Chromatic Ward (made-up) — {1}{R}
/// Enchantment
/// Creatures you control are red in addition to their other colors.
/// (Tests static ability color-adding via Layer 5 + register_static_effects)
pub fn chromatic_ward() -> Arc<CardData> {
    CardDataBuilder::new("Chromatic Ward")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Enchantment)
        .ability(static_ability(
            "Creatures you control are red in addition to their other colors.",
            is_also_color(each(creatures_you_control()), Color::Red),
        ))
        .build()
}
