//! Guilds of Ravnica (`grn`), released 2018-10-05.

use std::sync::Arc;

use crate::cards::authoring::static_replacement;
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType};
use crate::types::colors::Color;
use crate::types::effects::{PlayerSet, TokenDef};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite, TemplateAmount, TokenKind, TokenSubstitution};

/// Divine Visitation — {3}{W}{W}
/// Enchantment
///
/// > If one or more creature tokens would be created under your control, that
/// > many 4/4 white Angel creature tokens with flying and vigilance are
/// > created instead.
///
/// **The kind-changing substitution over a creation, and the kind field's
/// first customer.** `EventPattern::CreateTokens { kind: creature }` matches
/// the creation if any def is a creature, and the template replaces exactly
/// those defs — a Clue created beside a Soldier stays a Clue — with "that
/// many" Angels, where that many is the number the kind matched.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"The token's characteristics are entirely replaced … It doesn't have any
///   abilities the token would have been created with. Anything else
///   specified in the effect creating the token (such as tapped, attacking,
///   …) still applies."* → the template's def replaces the matched def whole,
///   and `enters_tapped` is carried over from the def it replaced.
///   → `divine_visitation_replaces_the_creatures_and_keeps_how_they_entered`
/// - *"If you create a noncreature token that will be a creature as it enters
///   the battlefield (March of the Machines), Divine Visitation's effect
///   doesn't apply"* → the kind is asked of the def's printed types, never of
///   the entry's frame. → `divine_visitation_reads_the_def_and_not_the_frame`
/// - *"If an effect changes under whose control a token would be created, that
///   effect applies before Divine Visitation's"* → CR 616.1b's ladder, which
///   `must_choose_among` already walks; no control-changing creation
///   replacement is registered to walk it with.
///
/// Beside Parallel Lives the affected player chooses the order and the
/// answer is four Angels either way — a multiplier and a replacement by
/// "that many" commute — which the test states by asking both ways.
///
/// **Registered and not pooled**: five mana for an effect two pooled cards
/// reach, and the creation path is measured by the Alarm already.
pub fn divine_visitation() -> Arc<CardData> {
    CardDataBuilder::new("Divine Visitation")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text(
            "If one or more creature tokens would be created under your control, that many \
             4/4 white Angel creature tokens with flying and vigilance are created instead.",
        )
        .ability(static_replacement("If one or more creature tokens would be created under your control, that many 4/4 white Angel creature tokens with flying and vigilance are created instead.",
            ReplacementDef::new(
                EventPattern::CreateTokens { kind: Some(TokenKind::of_type(CardType::Creature)) },
                PlayerSet::You,
                Rewrite::Instead(GameActionTemplate::CreateTokens {
                    def: TokenDef::creature(4, 4, &[Color::White], &[CreatureType::Angel]).with_keywords(&[KeywordFlag::Flying, KeywordFlag::Vigilance]),
                    count: TemplateAmount::ReplacedAmount,
                    mode: TokenSubstitution::Replace,
                }),
            ),
        ))
        .build()
}
