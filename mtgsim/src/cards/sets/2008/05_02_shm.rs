//! Shadowmoor (`shm`), released 2008-05-02.

use std::sync::Arc;

use crate::cards::authoring::{
    activated_ability, as_long_as, becomes_color, colored, copy, creature, draw_cards, draw_two_instead, gains_ability, gains_keyword,
    nonlegendary_creature, persist, remove_counters, spell_ability, static_ability, static_replacement, tapped_for_mana, target_creature,
    target_permanent, this_object, you, ContinuousChange,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{Condition, CopyRoles, CounterType, ObjectSet, PlayerFact, PlayerSet};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EnterModsTemplate, EventPattern, ReplacementDef, Rewrite};

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

/// Chainbreaker — {2}
/// Artifact Creature — Scarecrow, 3/3
///
/// This creature enters with two -1/-1 counters on it.
/// {3}, {T}: Remove a -1/-1 counter from target creature.
///
/// (Oracle text verified on Scryfall, 2026-09-01.)
///
/// # The second shape, and what it reaches that a tapland cannot
///
/// CR 122.6a — "an object that's given counters as it enters the battlefield" —
/// is the other half of `EnterMods`, and it is not the same code path: the
/// counters go on inside the performer, before the entry is announced and
/// before any state-based action can look. That ordering is the claim, and a
/// 3/3 that arrives as a 1/1 is how a game shows it.
///
/// The activated ability is what keeps the counters from being decoration. It
/// costs `{3}, {T}` — both cost primitives the engine actually implements — and
/// removes a counter from **target creature**, which in a random game means the
/// board contains a permanent whose printed P/T and effective P/T disagree, and
/// then stops disagreeing. Nothing else in either pool produces a -1/-1
/// counter at all, and being colorless it is in every deck in every game.
///
/// # What it is *not* the card for
///
/// A 0/0 that enters with +1/+1 counters is the sharper test of the ordering
/// inside the performer, and it is [`adaptive_shimmerer`]'s job rather than a
/// reason to prefer one over the other. Chainbreaker earns its place on a
/// different axis: it is `{2}` rather than `{5}`, so a random game casts it far
/// more often, and its ability is the only thing in either pool that *spends* a
/// counter a permanent entered with.
///
/// **The -1/-1 sign is not decoration either.** Layer 7c reads both, and a
/// board where a permanent's printed and effective P/T disagree *downward* is
/// one no other registered card produces.
pub fn chainbreaker() -> Arc<CardData> {
    CardDataBuilder::new("Chainbreaker")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Scarecrow))
        .power_toughness(3, 3)
        .rules_text(
            "This creature enters with two -1/-1 counters on it.\n\
             {3}, {T}: Remove a -1/-1 counter from target creature.",
        )
        .ability(static_replacement(
            "This creature enters with two -1/-1 counters on it.",
            ReplacementDef::new(
                EventPattern::EnterBattlefield { cast: None },
                ObjectSet::SourceOnly,
                Rewrite::EnterWith(EnterModsTemplate::with_counters(CounterType::MinusOneMinusOne, 2)),
            ),
        ))
        .ability(activated_ability(
            "{3}, {T}: Remove a -1/-1 counter from target creature.",
            vec![Cost::Mana(ManaCost::build(&[], 3)), Cost::TapSelf],
            remove_counters(target_permanent(creature()), CounterType::MinusOneMinusOne, 1),
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
            becomes_color(target_creature(), [Color::Red]).and_gains_keyword(KeywordFlag::Haste).until_end_of_turn(),
        ))
        .ability(spell_ability(
            "Draw a card.",
            draw_cards(you(), 1),
        ))
        .build()
}

/// Mana Reflection — {4}{G}{G}
/// Enchantment
///
/// > If you tap a permanent for mana, it produces twice as much of that mana
/// > instead.
///
/// **The pooled card, and CR 106.6a's first customer.** `Amount(Multiplier(2))`
/// over every unit the production carries — the plain ones scaled, the
/// restricted atoms repeated, which is the rule's "any restrictions … will
/// apply to all mana produced". `PlayerSet::You` is "if *you* tap".
///
/// # The rulings, and where each is tested
///
/// - *"You're 'tapping a permanent for mana' only if you're activating a mana
///   ability of that permanent that includes the {T} symbol in its cost. A
///   mana ability produces mana as part of its effect."* → CR 106.12, read off
///   the activation cost: a Forest is doubled, Dark Ritual is not
///   (`dark_ritual_is_not_tapping_a_permanent_for_mana`), and neither is
///   Krark-Clan Ironworks' sacrifice
///   (`a_mana_ability_without_a_tap_symbol_is_not_doubled`).
/// - *"If an ability triggers 'whenever you tap' something for mana and
///   produces mana, that triggered mana ability won't be affected by Mana
///   Reflection."* → CR 605.1b's triggered mana abilities are critical-path
///   item 6's; the day one exists its own production proposes
///   `tapped_for_mana: false` by the same definition, and this ruling is
///   that PR's test rather than this one's.
/// - *"Mana Reflection doesn't produce any mana itself. Rather, it causes
///   permanents you tap for mana to produce more mana. If the mana ability of
///   that permanent puts any restrictions or riders on the mana it produces,
///   that will apply to all the mana it produces this way."* →
///   `ATOM-106.6a-001`, on a fixture land whose {G} is spendable only on
///   creature spells: two restricted atoms, no free one
///   (`a_restricted_production_doubles_into_two_restricted_units`).
/// - *"The effects of multiple Mana Reflections are cumulative. For example,
///   if you have two Mana Reflections on the battlefield, you'll get four
///   times the original amount and type of mana. If you have three, you'll
///   get eight times the mana, and so on."* → four, and the prompt is the
///   first assertion: two multipliers commute, so CR 616.1 has nothing to ask
///   (`two_mana_reflections_quadruple_with_no_prompt`,
///   `three_mana_reflections_multiply_by_eight`).
pub fn mana_reflection() -> Arc<CardData> {
    CardDataBuilder::new("Mana Reflection")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 4))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text("If you tap a permanent for mana, it produces twice as much of that mana instead.")
        .ability(static_replacement("If you tap a permanent for mana, it produces twice as much of that mana instead.",
            ReplacementDef::new(
                tapped_for_mana(None),
                PlayerSet::You,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            ),
        ))
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
/// `ObjectSet::Fixed` carrying a single `Box<CopiableValues>`. `Box` and `Arc`
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
/// forbids, and CR 202.2's colors come out W and U exactly as printed.
///
/// `codebase-state.md`'s 2026-08-24 entry set the opposite precedent —
/// `fixtures::inside_out` stays unregistered because simplifying its
/// hybrid cost would "misrepresent the card". That was the right call there and
/// is the wrong one here, for a reason that is about the *pool* rather than
/// about the card: Inside Out had a registered substitute for the engine path it
/// covered, and Mirrorweave and Mirrorform are the only consumers of
/// `CopyRoles::FilteredCopyRecipient` in the crate. Leaving Mirrorweave out
/// would ship the arm's `exclude_donor: true` leg with no random-play coverage,
/// which is the failure `PERFORMANCE_POOL`'s own doc records RS-1 for. Recorded rather than quietly done: the first real hybrid
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
        .ability(spell_ability(
            "Each other creature becomes a copy of target nonlegendary creature until end of turn.",
            copy(
                // The target is the *donor* here, and it is what the printed
                // "nonlegendary" scopes.
                target_permanent(nonlegendary_creature()),
                // "Each **other** creature" — `exclude_donor` is the word
                // "other", and it is a field rather than structure because
                // Mirrorform prints the same shape without it.
                CopyRoles::FilteredCopyRecipient { filter: creature(), exclude_donor: true },
                Vec::new(),
            )
            .until_end_of_turn(),
        ))
        .build()
}

/// Thought Reflection — {4}{U}{U}{U}
/// Enchantment
///
/// > If you would draw a card, draw two cards instead.
///
/// **The acid test's card.** It is not legendary, so a board can hold two, and
/// its ruling gives the arithmetic verbatim: two draw four times the original
/// number, three draw eight. CR 616.1 asks nothing between them — the order
/// provably cannot change the total, which is `ordering_cannot_change_outcome`'s
/// third shape (§11 item 55). That 2ⁿ is what §3.2d's lineage rule buys — each
/// doubled draw inherits the applied set of the draw it came from, so a
/// Reflection that has applied cannot apply to its own output. Without the
/// inheritance the game does not answer wrongly; it hangs.
///
/// **The pooled card of the PR**, at seven mana, which is the most any pooled
/// card has cost — so its reachability is measured with `--require` rather than
/// assumed.
/// # The rulings, and where each is tested
///
/// - *"If a spell or ability causes you to draw multiple cards, [this] doubles
///   each card draw ... Harmonize ('Draw three cards') ... you'll draw six."* →
///   `ATOM-121.2a-001` from the inner side: the instruction is not what it
///   watches, so three individual draws each become two.
///   → `thought_reflection_doubles_each_of_a_three_card_instruction`
/// - *"The effects of multiple Thought Reflections are cumulative ... two ...
///   four times the original number ... three ... eight times."* → **the acid
///   test**, `test_two_thought_reflections_draw_four_not_infinity`, with
///   `three_thought_reflections_draw_eight` for the exponent.
/// - *"If two or more replacement effects would apply to a card-drawing event,
///   the player who's drawing the card chooses what order to apply them."* →
///   falls out of CR 616.1's chooser being the affected player. **Not asserted
///   on two Reflections**: their order provably cannot change the answer, so the
///   engine does not ask (§11 item 55). The boards that do ask are
///   `a_draw_doubler_beside_a_notion_thief_is_a_real_choice` and the three-Thief
///   board, where the chooser moves with the event's subject.
pub fn thought_reflection() -> Arc<CardData> {
    CardDataBuilder::new("Thought Reflection")
        .mana_cost(ManaCost::build(
            &[ManaType::Blue, ManaType::Blue, ManaType::Blue],
            4,
        ))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text("If you would draw a card, draw two cards instead.")
        .ability(static_replacement("If you would draw a card, draw two cards instead.", draw_two_instead(None)))
        .build()
}

/// "You control a [color] creature."
fn you_control_a_creature_of(color: Color) -> Condition {
    let creature_of_color = creature().and(colored(color));
    Condition::Player { players: PlayerSet::You, fact: PlayerFact::ControlsPermanent(creature_of_color) }
}

/// Wingrattle Scarecrow — {3}
/// Artifact Creature — Scarecrow, 2/2
///
/// > This creature has flying as long as you control a blue creature.
/// > This creature has persist as long as you control a black creature.
///
/// Two conditional statics over the Scarecrow itself, Kird Ape's shape: one
/// grants a keyword, the other persist, a triggered ability. Persist is
/// asked off the frame of the Scarecrow that died (CR 603.10a), so it
/// triggers if a black creature was there as it left, the black creature
/// dying beside it included, and once on the stack it no longer needs one.
///
/// It exercises a triggered ability granted under a condition, asked off the
/// frame of the object that left.
///
/// # The rulings, and where each is tested
///
/// Kitchen Finks' seven, word for word. `phase_tr4a_integration_test` proves
/// them on printed persist; each is asked again in
/// `phase_tr7_integration_test` on the Scarecrow, beside a black creature.
/// - *A creature with persist that stops being a creature still persists*
///   (#1) → `the_scarecrow_persists_after_it_stops_being_a_creature`.
/// - *Whether it had a -1/-1 counter is read off how it last existed on the
///   battlefield* (#2) → `the_scarecrow_returns_once_and_then_stays_dead` and
///   `a_scarecrow_that_died_with_a_minus_counter_stays_dead`.
/// - *Several instances each trigger, and the next to resolve does nothing*
///   (#3) → `two_instances_of_persist_return_the_scarecrow_once`.
/// - *A token with persist triggers and cannot return* (#4) →
///   `a_scarecrow_token_persists_into_nothing`.
/// - *It returns as a new object with no memory of the last* (#5) →
///   `the_scarecrow_returns_as_a_new_object`.
/// - *Simultaneous persists go on in APNAP order and return one at a time,
///   the nonactive player's first* (#6) →
///   `two_players_scarecrows_return_the_nonactive_players_first`.
/// - *A creature with +1/+1 counters killed by -1/-1 counters dies with both
///   on it, and persist does not trigger* (#7) →
///   `a_scarecrow_killed_through_its_plus_counter_stays_dead`.
pub fn wingrattle_scarecrow() -> Arc<CardData> {
    let flying = "This creature has flying as long as you control a blue creature.";
    let persists = "This creature has persist as long as you control a black creature.";
    let has_while_you_control_a =
        |text: &'static str, color: Color, grant: ContinuousChange| static_ability(text, as_long_as(you_control_a_creature_of(color), grant));
    CardDataBuilder::new("Wingrattle Scarecrow")
        .mana_cost(ManaCost::build(&[], 3))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Scarecrow))
        .power_toughness(2, 2)
        .rules_text("This creature has flying as long as you control a blue creature.\nThis creature has persist as long as you control a black creature.")
        .ability(has_while_you_control_a(flying, Color::Blue, gains_keyword(this_object(), KeywordFlag::Flying)))
        .ability(has_while_you_control_a(persists, Color::Black, gains_ability(this_object(), persist())))
        .build()
}
