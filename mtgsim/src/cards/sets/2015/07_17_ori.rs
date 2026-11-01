//! Magic Origins (`ori`), released 2015-07-17.

use std::sync::Arc;

use crate::cards::authoring::{create_replacement, creature, draw_cards, draw_two_instead, spell_ability, static_replacement, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{ObjectSet, PlayerSet};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EventPattern, GameActionTemplate, ReplacementDef, Rewrite, TemplateAmount};
use crate::types::zones::{DrawFilter, Zone};

/// Alhammarret's Archive — {5}
/// Legendary Artifact
///
/// > If you would gain life, you gain twice that much life instead.
/// > If you would draw a card except the first one you draw in each of your
/// > draw steps, draw two cards instead.
///
/// **Two statics on one permanent, one from each of the two RE phases** —
/// Rhox Faithmender's gain doubler and Teferi's Ageless Insight's draw doubler,
/// written as the same two defs because they *are* the same two defs. Gisela's
/// shape, and the reason RE-2 → RE-3 is a hard order in §9.
///
/// # The rulings, and where each is tested
///
/// - *"If an effect would set your life total to a specific number that's higher
///   ... your life total will actually become 17."* → RE-6's, as Rhox
///   Faithmender's twin.
/// - *"If two or more replacement effects would apply to a card-drawing event,
///   the player drawing the card chooses the order."* → already tested, in
///   RE-2: `a_draw_doubler_beside_a_notion_thief_is_a_real_choice`.
/// - *"Because [it] is legendary ... if that happens, life gained by that player
///   will be multiplied by four."* → the legend rule makes this Rhox
///   Faithmender's board. This card's own board is the one that shows the two
///   halves coexist.
///   → `alhammarrets_archive_doubles_a_gain_and_a_draw_from_one_permanent`
/// - *"Similarly, the effects of the last abilities of multiple Archives are
///   cumulative."* → already tested, in RE-2:
///   `test_two_thought_reflections_draw_four_not_infinity`.
pub fn alhammarrets_archive() -> Arc<CardData> {
    CardDataBuilder::new("Alhammarret's Archive")
        .mana_cost(ManaCost::build(&[], 5))
        .card_type(CardType::Artifact)
        .supertype(Supertype::Legendary)
        .rules_text(
            "If you would gain life, you gain twice that much life instead.\nIf you would draw a card except the first one you draw in each of your draw steps, draw two cards instead.",
        )
        .ability(static_replacement("If you would gain life, you gain twice that much life instead.",
            ReplacementDef::new(
                EventPattern::GainLife,
                PlayerSet::You,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            ),
        ))
        .ability(static_replacement("If you would draw a card except the first one you draw in each of your draw steps, draw two cards instead.", draw_two_instead(Some(DrawFilter::NotFirstInDrawStep))))
        .build()
}

/// Hallowed Moonlight — {1}{W}
/// Instant
///
/// > Until end of turn, if a creature would enter and it wasn't cast, exile
/// > it instead.
/// > Draw a card.
///
/// **Containment Priest's row from a resolution, and without the "nontoken"**
/// — which is the whole reason it is here. The Priest excludes tokens, so no
/// registered card ever substituted a *token's* entry, and the engine's answer
/// for one was the cheap one: a `ZoneChange { from: Battlefield }` for a token
/// that was never there (`codebase-state.md` item 52). This card reaches it,
/// and the answer is now `GameAction::CreateTokenIn` — the token is created in
/// exile.
///
/// Every piece exists since RC-4b and RD-2: `Primitive::CreateReplacement`
/// with an `UntilEndOfTurn` row, `EventPattern::EnterBattlefield { cast:
/// Some(false) }`, a creature filter over the CR 614.12 frame, and
/// `Instead(ZoneChangeTo { Exile })`. "A creature" is every creature — no
/// controller clause — so the row is about the object and names no player.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"After Hallowed Moonlight resolves, if a creature token would be put
///   onto the battlefield, it's put into exile instead and then ceases to
///   exist. Creature tokens are never cast, even if the spell that created
///   them was."* → the log holds `TokenCreated { Exile }` and CR 704.5d's
///   `TokenCeasedToExist`, and **no `ZoneChange`** for the token at all — the
///   line Dour Port-Mage would have read, asserted absent.
///   → `hallowed_moonlight_creates_the_token_in_exile_and_it_ceases_to_exist`
/// - *"Hallowed Moonlight won't affect any creature that was cast, no matter
///   which zone it was cast from and whether or not its mana cost was paid."*
///   → Grizzly Bears cast from hand resolves and enters under it.
///   → `hallowed_moonlight_does_not_affect_a_creature_that_was_cast`
///
/// **Registered and not pooled** — `PERFORMANCE_POOL`'s comment says why.
pub fn hallowed_moonlight() -> Arc<CardData> {
    CardDataBuilder::new("Hallowed Moonlight")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(
            "Until end of turn, if a creature would enter and it wasn't cast, exile it \
             instead.\nDraw a card.",
        )
        .ability(spell_ability(
            "Until end of turn, if a creature would enter and it wasn't cast, exile it instead.",
            create_replacement(ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: Some(false) },
                ObjectSet::battlefield_filter(creature()),
                Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
            ))
            .until_end_of_turn(),
        ))
        .ability(spell_ability(
            "Draw a card.",
            draw_cards(you(), 1),
        ))
        .build()
}

/// Tainted Remedy — {2}{B}
/// Enchantment
///
/// > If an opponent would gain life, that player loses that much life instead.
///
/// **The kind-changing substitution, and the first customer of
/// [`TemplateAmount::ReplacedAmount`].** "That much" is the gain's own number
/// read at the moment this applies, which is what makes its ordering ruling
/// arithmetic rather than a coin flip: beside Alhammarret's Archive the gaining
/// player picks double-then-lose-6, or lose-3-then-nothing.
///
/// Four-player: `PlayerSet::Opponents` is three opponents against one static
/// row, which is the shape CR 109.5 resolves per event rather than per
/// registration.
/// # The rulings, and where each is tested
///
/// - *"If more than one replacement effect tries to apply to a life gain event,
///   the player who would gain life chooses the order ... that player may choose
///   to have the 3 life become doubled to 6 life and then lose 6 life. The
///   player may also choose to apply [this] first, turning 'gain 3 life' into
///   'lose 3 life.' Alhammarret's Archive would then not apply."* → test, with
///   the ruling's own numbers, and the board `ordering_cannot_change_outcome`
///   must **not** suppress.
///   → `the_gaining_player_chooses_between_the_archive_and_tainted_remedy`
/// - *"Having more than one [of these] on the battlefield doesn't have any
///   noticeable effect on life gain. Once the effect of one applies, there is no
///   life gain for the others to apply to."* → two tests, because the engine
///   proves the order away and a suppressed choice can only be stated by making
///   it. → `a_second_tainted_remedy_has_no_gain_left_to_apply_to` and
///   `two_tainted_remedies_lose_the_same_three_whichever_applies`
pub fn tainted_remedy() -> Arc<CardData> {
    CardDataBuilder::new("Tainted Remedy")
        .mana_cost(ManaCost::build(&[ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("If an opponent would gain life, that player loses that much life instead.")
        .ability(static_replacement("If an opponent would gain life, that player loses that much life instead.",
            ReplacementDef::new(
                EventPattern::GainLife,
                PlayerSet::Opponents,
                Rewrite::Instead(GameActionTemplate::LoseLife {
                    amount: TemplateAmount::ReplacedAmount,
                }),
            ),
        ))
        .build()
}
