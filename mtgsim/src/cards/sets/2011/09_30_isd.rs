//! Innistrad (`isd`), released 2011-09-30.

use std::sync::Arc;

use crate::cards::authoring::{
    creature, draw_cards, equip, gains_keyword, host, spell_ability, static_ability, static_replacement, you, you_control, you_have,
};
use crate::objects::card_data::{AbilityDef, CardData, CardDataBuilder};
use crate::types::card_types::{ArtifactType, CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::{AdditionalCost, Cost};
use crate::types::effects::{Condition, Effect, ObjectSet, PlayerFact, PlayerSet};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, CopyDonor, EntryCopyTemplate, EventPattern, GameActionTemplate, ReplacementDef, Rewrite};

/// Altar's Reap — {1}{B}
/// Instant
///
/// As an additional cost to cast this spell, sacrifice a creature.
/// Draw two cards.
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// **CR 601.2h prints this card.** The rule's own example is Altar's Reap
/// sacrificing Thunderscape Familiar: the total is locked at {B} before any
/// payment, so the Familiar's reduction is counted even though the Familiar
/// is what pays. Nothing in the engine had a mandatory additional cost before
/// this — `AdditionalCost` was all optional, which CR 118.8b only ever
/// claimed of *some* of them.
///
/// The filter is `ByType(Creature)` and says nothing about control: CR
/// 701.21a supplies that ("a player can't sacrifice ... a permanent they
/// don't control"), and a card that repeated it would be claiming the rule
/// was the card's.
pub fn altars_reap() -> Arc<CardData> {
    CardDataBuilder::new("Altar's Reap")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Instant)
        .rules_text(
            "As an additional cost to cast this spell, sacrifice a creature.\nDraw two cards.",
        )
        .additional_cost(AdditionalCost::Mandatory(vec![Cost::Sacrifice(
            creature(),
            1,
        )]))
        .ability(spell_ability(
            "Draw two cards.",
            draw_cards(you(), 2),
        ))
        .build()
}

/// Cobbled Wings — {2}
/// Artifact — Equipment
///
/// Equipped creature has flying.
/// Equip {1}
///
/// (Oracle text verified on Scryfall, 2026-09-06. Innistrad; reprinted.)
///
/// # Why this one
///
/// CR 613.7e is unobservable through Bonesplitter alone — Layer 7c additions
/// commute — so the rule is pinned with a Layer 6 grant against Humility's
/// "lose all abilities": the Equipment enters first, Humility second, and
/// only the timestamp the equip gives it (CR 613.7e) lets its grant apply
/// after Humility. Cobbled Wings is the printed card with exactly that text;
/// an invented fixture was written first and replaced in review.
///
/// Registered, not pooled: it opens no engine path Bonesplitter does not,
/// and it puts a CR 613.7e reordering within reach of a `stress` game, where
/// Humility is.
pub fn cobbled_wings() -> Arc<CardData> {
    CardDataBuilder::new("Cobbled Wings")
        .card_type(CardType::Artifact)
        .subtype(Subtype::Artifact(ArtifactType::Equipment))
        .mana_cost(ManaCost::build(&[], 2))
        .rules_text("Equipped creature has flying.\nEquip {1}")
        .ability(static_ability(
            "Equipped creature has flying.",
            gains_keyword(host(), KeywordFlag::Flying),
        ))
        .ability(equip("Equip {1}", vec![Cost::Mana(ManaCost::build(&[], 1))]))
        .build()
}

/// A static ability whose replacement effect exists only while `condition`
/// holds — CR 604.2's "as long as", asked by `replacement::gather` at each
/// proposal, the way the layer pass asks it of a Kird Ape.
fn static_conditional_replacement(rules_text: &'static str, condition: Condition, def: ReplacementDef) -> AbilityDef {
    static_ability(
        rules_text,
        Effect::Conditional(condition, Box::new(Effect::Replacement(Box::new(def)))),
    )
}

/// Essence of the Wild — {3}{G}{G}{G}
/// Creature — Avatar 6/6
///
/// > Creatures you control enter as a copy of this creature.
///
/// (Oracle text verified on Scryfall, 2026-11-15.)
///
/// A copy as a creature enters (CR 614.1c, 707.9): every creature of an
/// event is a copy from the moment it is announced, and CR 616.1c orders
/// copy effects ahead of the rest of what modifies how it enters.
///
/// # The rulings, and where each is tested
///
/// - *"External abilities may still affect how a creature enters. ..."* (#1)
///   → `phase_cv2a_integration_test::an_enters_tapped_from_outside_still_taps_the_essence_copy`.
/// - *"If a creature such as Cryptoplasm that's already on the battlefield
///   copies Essence of the Wild and modifies its own copy effect, that
///   modification is also copied ..."* (#2) →
///   `phase_cv1b_integration_test::a_cryptoplasm_copy_of_essence_makes_copies_that_keep_its_ability`.
/// - *"If you control more than one Essence of the Wild, creatures you
///   control will enter as a copy of the one whose copy effect you apply
///   last."* (#3) → `phase_cv2a_integration_test::two_essences_give_the_one_applied_last`.
/// - *"Creatures you control don't copy whether Essence of the Wild is
///   tapped or untapped, ... counters ..., or any non-copy effects ..."*
///   (#4) → `phase_cv2a_integration_test::an_essence_copy_takes_none_of_its_status_counters_or_pumps`.
/// - *"... any "enters" triggered abilities printed on such creatures won't
///   trigger."* (#5) →
///   `phase_cv2a_integration_test::a_creature_entering_as_an_essence_does_not_trigger_its_enters_ability`.
/// - *"Replacement effects that modify how a creature enters are applied in
///   the following order: first control-changing effects ..., then copy
///   effects ..., then all other effects. ..."* (#6) →
///   `phase_cv2a_integration_test::a_control_change_on_entry_decides_whose_essence_applies`;
///   the copy-before-the-rest half is #8's board.
/// - *"If a creature such as Clone is entering under your control, there will
///   be two copy effects to apply ..."* (#7) →
///   `phase_cv2a_integration_test::a_clone_beside_essence_of_the_wild_enters_as_the_essence`.
/// - *"Other "enters" replacement abilities printed on the creature entering
///   won't be applied ..."* (#8) →
///   `phase_cv2a_integration_test::rusted_sentinel_under_essence_of_the_wild_enters_untapped`.
pub fn essence_of_the_wild() -> Arc<CardData> {
    let text = "Creatures you control enter as a copy of this creature.";
    CardDataBuilder::new("Essence of the Wild")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green, ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Avatar))
        .power_toughness(6, 6)
        .rules_text(text)
        .ability(static_replacement(
            text,
            ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                ObjectSet::battlefield_filter(creature().and(you_control())),
                Rewrite::EnterAsCopy(EntryCopyTemplate { donor: CopyDonor::ThisObject, except: Vec::new() }),
            ),
        ))
        .build()
}

/// Laboratory Maniac — {2}{U}
/// Creature — Human Wizard 2/2
///
/// > If you would draw a card while your library has no cards in it, you win
/// > the game instead.
///
/// The kind-changing substitution from a draw to the game's end, and the first
/// replacement effect in the crate whose static ability carries a condition —
/// the card's "while", which is CR 604.2's "as long as" shape:
/// `PlayerFact::LibraryEmpty` on the ability, asked by the gather at each
/// proposal (CR 604.2, 614.4). The
/// draw reaches the pipeline with nothing to draw because CR 121.6a says a
/// draw replacement applies "even if no cards could be drawn", which is the
/// whole of why `draw_card` flags rather than refuses — and with the draw
/// replaced, no flag is set and no loss is proposed. **The pooled card of the
/// PR.**
/// # The rulings, and where each is tested
///
/// - *"If for some reason you can't win the game (because your opponent has
///   cast Angel's Grace this turn, for example), you won't lose for having
///   tried to draw a card from a library with no cards in it. The draw was
///   still replaced."* → an opponent's Platinum Angel is the printed "can't
///   win" the crate has: the draw is replaced, the substituted win is refused
///   by CR 101.2 at the next iteration, no card is drawn and no flag is set.
///   → `laboratory_maniac_under_an_opponents_platinum_angel_neither_wins_nor_loses`
/// - *"If two or more players each control a Laboratory Maniac and each player
///   is instructed to draw a number of cards, first the player whose turn it is
///   draws that many cards. If this causes that player to win the game instead,
///   the game is immediately over."* → **not expressible**: an each-player draw
///   instruction needs a recipient `EffectRecipient` lacks and CR 121.2c's
///   APNAP ordering over an effect's recipients (`codebase-state.md` item 122,
///   RE-2's note). What *is* tested is the half the engine has — the game is
///   over at the first win and nothing after it performs —
///   → `a_win_ends_the_game_immediately_and_the_rest_of_the_batch_still_performs`.
pub fn laboratory_maniac() -> Arc<CardData> {
    CardDataBuilder::new("Laboratory Maniac")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .power_toughness(2, 2)
        .rules_text(
            "If you would draw a card while your library has no cards in it, you win the game instead.",
        )
        .ability(static_conditional_replacement("If you would draw a card while your library has no cards in it, you win the game instead.",
            you_have(PlayerFact::LibraryEmpty),
            // Any individual draw, whatever instructed it: the draw step's,
            // a cantrip's, the seventh of Stunning Reversal's. The instruction
            // (`DrawCards`) is not what this watches — CR 121.2 performs the
            // draws one at a time and the library empties between them.
            ReplacementDef::new(
                EventPattern::DrawCard { place: None },
                PlayerSet::You,
                Rewrite::Instead(GameActionTemplate::PlayerWins),
            ),
        ))
        .build()
}

/// Parallel Lives — {3}{G}
/// Enchantment
///
/// > If an effect would create one or more tokens under your control, it
/// > creates twice that many of those tokens instead.
///
/// **CR 614.16's token half, and the first watcher of the outer event.** The
/// subject of a `CreateTokens` is the player the tokens are created under, so
/// "under your control" is [`PlayerSet::You`] and the object set is empty —
/// Rhox Faithmender's shape over a different event. The rewrite is the same
/// arm too, and over a `Vec` a multiplier repeats each def in place, which is
/// what "twice that many of those tokens" says.
///
/// Doubling Season's first ability is this card's text word for word; the
/// Season is registered whole in RE-5, when its counter half has an event to
/// watch.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"If you control two Parallel Lives, then the number of tokens created is
///   four times the original number. If you control three, then … eight times
///   the original number, and so on."* → the Furnace pair's third kind: two
///   commuting multipliers on one event, applied both without a CR 616.1
///   prompt. → `two_parallel_lives_create_four_times_as_many_and_ask_nothing`
/// - *"Everything that is specified by the effect creating the original token
///   or tokens will also be true about the additional token or tokens created
///   by Parallel Lives's replacement effect. For example, if an effect tells
///   you to create a token 'tapped and attacking,' the additional tokens will
///   also be tapped and attacking."* → structurally true of a repeated def,
///   asserted on every token's characteristics. "Tapped and attacking" is
///   not a shape this engine's creation can carry yet — a creation has no
///   `EnterMods` — and the ruling's *structure* is what the test proves.
///   → `the_extra_tokens_are_the_same_tokens`
///
/// **Pooled**, with Raise the Alarm — `PERFORMANCE_POOL`'s comment says why.
pub fn parallel_lives() -> Arc<CardData> {
    CardDataBuilder::new("Parallel Lives")
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(
            "If an effect would create one or more tokens under your control, it creates \
             twice that many of those tokens instead.",
        )
        .ability(static_replacement("If an effect would create one or more tokens under your control, it creates twice that many of those tokens instead.",
            ReplacementDef::new(
                EventPattern::CreateTokens { kind: None },
                PlayerSet::You,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            ),
        ))
        .build()
}
