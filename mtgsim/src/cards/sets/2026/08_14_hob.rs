//! The Hobbit (`hob`), released 2026-08-14.

use std::sync::Arc;

use crate::cards::authoring::{draw_two_instead, static_replacement};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::PlayerSet;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EventPattern, ReplacementDef, Rewrite};
use crate::types::zones::DrawFilter;

/// Bard, King of Dale — {4}{W}{U}
/// Legendary Creature — Human Noble Archer 3/5
///
/// > Reach, vigilance
/// > If you would draw a card except the first one you draw in each of your
/// > draw steps, draw two cards instead.
/// > If one or more tokens would be created under your control, twice that
/// > many of those tokens are created instead.
///
/// Alhammarret's Archive's draw half beside Parallel Lives' token half. RE-2's
/// ledger row named Bard as waiting on "RE-4's token doubler"; the review
/// found both halves built and registered it (`plans/handoffs/re-4-review.md`,
/// R5 — the review also caught the card's first draft calling it a card that
/// replaces draws with tokens, which is Hullbreacher).
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"If you control two, cards drawn will be multiplied by four … the last
///   ability is cumulative: two, four times the number of tokens."* → two
///   Bards, both halves. → `two_bards_quadruple_both_halves`
/// - *"If an effect creates more than one kind of token, it'll create twice as
///   many of each kind."* → a heterogeneous creation, `[A, B]` → `[A, A, B,
///   B]`. → `bard_doubles_each_kind_of_a_mixed_creation`
/// - *"Copies of permanent spells that resolve become tokens … not created and
///   will not be doubled."* → CR 111.13; `GameEvent::TokenCreated`'s doc is
///   where the engine draws that line, and `spell-copies` is where the spell copy
///   exists.
/// - *"All of the tokens enter the battlefield simultaneously … same name,
///   color, type …"* → RE-4's batch; `the_extra_tokens_are_the_same_tokens`.
/// - *"If the token … has 'enters with' abilities, first determine how many
///   tokens are being created, then apply those abilities individually for
///   each one."* → CR 616.1g as the order of two loops;
///   `two_devour_tokens_created_together_are_each_asked_and_never_offered_each_other`.
///
/// **Registered and not pooled**: six mana and legendary.
pub fn bard_king_of_dale() -> Arc<CardData> {
    CardDataBuilder::new("Bard, King of Dale")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::Blue], 4))
        .color(Color::White)
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Noble))
        .subtype(Subtype::Creature(CreatureType::Archer))
        .power_toughness(3, 5)
        .keyword_flag(KeywordFlag::Reach)
        .keyword_flag(KeywordFlag::Vigilance)
        .rules_text(
            "Reach, vigilance\nIf you would draw a card except the first one you draw in each \
             of your draw steps, draw two cards instead.\nIf one or more tokens would be \
             created under your control, twice that many of those tokens are created instead.",
        )
        .ability(static_replacement("If you would draw a card except the first one you draw in each of your draw steps, draw two cards instead.", draw_two_instead(Some(DrawFilter::NotFirstInDrawStep))))
        .ability(static_replacement("If one or more tokens would be created under your control, twice that many of those tokens are created instead.",
            ReplacementDef::new(
                EventPattern::CreateTokens { kind: None },
                PlayerSet::You,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            ),
        ))
        .build()
}
