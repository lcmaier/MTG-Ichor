//! Return to Ravnica (`rtr`), released 2012-10-05.

use std::sync::Arc;

use crate::cards::authoring::{all_damage_to_you, other_than_this, static_replacement, you_control};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{ObjectFilter, ObjectSet};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, RetargetSpec, Rewrite};
use crate::types::zones::{Zone, ZoneChangePattern, ZoneSet};

/// Palisade Giant — {4}{W}{W}
///
/// > All damage that would be dealt to you and other permanents you control is
/// > dealt to this creature instead.
///
/// The first row in the crate carrying **both** a real object filter and a real
/// player set, which is what decision 0's union was for — and the first card
/// whose filter needs `ObjectFilter::NotSource`. "Other permanents you control"
/// is `ByController(You) ∧ NotSource`, and without the second leg the Giant
/// would redirect its own damage to itself.
///
/// `RetargetSpec::ToEffectSource`, because "this creature" is the object whose
/// ability this is. Reflect Damage's "that source's controller" is the *other*
/// source, which is why neither arm is called `ToSource`.
///
/// # The rulings (Scryfall, 2026-09-08), and where each is tested
///
/// - *Applying this redirection effect doesn't change whether the damage is
///   combat damage.* → Pariah's first ruling from the other card; asserted on
///   the same flag.
/// - *If you control more than one Palisade Giant, you choose which redirection
///   effect to apply. You can't divide damage dealt by one source … or choose
///   to have the 8 damage dealt to you.* → two candidates, one prompt, the
///   whole amount on one Giant. The last clause is what makes the effect
///   mandatory rather than `optional`: there is no "decline" on this card.
pub fn palisade_giant() -> Arc<CardData> {
    CardDataBuilder::new("Palisade Giant")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 4))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Giant))
        .subtype(Subtype::Creature(CreatureType::Soldier))
        .power_toughness(2, 7)
        .rules_text(
            "All damage that would be dealt to you and other permanents you control is dealt \
             to this creature instead.",
        )
        .ability(static_replacement("All damage that would be dealt to you and other permanents you control is dealt to this creature instead.", all_damage_to_you(
            ObjectSet::battlefield_filter(you_control().and(other_than_this())),
            RetargetSpec::ToEffectSource,
        )))
        .build()
}

/// **Rest in Peace** — `{1}{W}` Enchantment.
///
/// "If a card or token would be put into a graveyard from anywhere, exile it
/// instead."
///
/// # Why this card exists: CR 616.1 had never run
///
/// Kalitas was the only registered card producing a replacement effect, and it
/// is Legendary — CR 704.5j means no player controls two, and two opposing
/// copies each apply only to the *other* player's creatures, so every death had
/// exactly one candidate. CR 616.1 engages only among "two or more", so its
/// ordering choice, its applied set across instances, and its APNAP ordering
/// were unreachable in any game at any length. This card is the second
/// candidate. `engineering-practices.md#how-many-cards-a-mechanic-owes`
/// generalizes the lesson.
///
/// **The pair is observable, which is what makes it a test and not a claim.**
/// Both rewrites exile, but Kalitas carries a CR 615.5 rider and this does not:
/// pick Kalitas and a Zombie appears, pick this and none does. Whichever
/// applies first, the other stops matching — the card is on its way to exile,
/// and both patterns want `to: Graveyard`.
///
/// # `ObjectFilter::All`, deliberately
///
/// "a card **or token**" is the widest filter of RB's three cards, and the
/// absence of `Not(Token)` is the whole difference from Leyline of the Void.
/// It also means this competes with Kalitas on exactly the deaths Kalitas
/// cares about while applying to plenty it does not.
///
/// # What is deferred, and on what precedent
///
/// "When this enchantment enters, exile all graveyards" is a triggered ability
/// (CR 603), which does not exist. Shipping the static half alone is the trade
/// Blood Moon already made — it works while Aura casting does not — and the
/// replacement half is the half this card is here for.
///
/// # "From anywhere" is literal, and that took a correction
///
/// An earlier draft shipped this `from: Battlefield`, arguing that
/// `ObjectSet::Filter` carries an `ObjectFilter` and a card on the stack is
/// not a permanent. **That was wrong about this card.** The filter here is
/// `All`, which reads nothing at all; `object_matches_filter` resolves
/// it from `game.objects` in any zone. So `from: None` costs nothing, and the
/// narrowing shipped a card that did not do what it says — a resolving
/// Lightning Bolt went to the graveyard.
///
/// The widening is reachable and load-bearing: stack→graveyard happens on every
/// resolved instant and sorcery (CR 608.2n) and every fizzle (CR 608.3), and
/// hand→graveyard on the CR 514.1 cleanup discard. Milling would be the third
/// and `Primitive::Mill` is unimplemented, so the library case is out of reach
/// rather than out of scope.
///
/// **And "from anywhere" is the affected set's zones as well as the pattern's
/// `from` (RF, 2026-09-16).** `set_affects` asks where the affected object
/// *is* before it asks the filter — the check the LJ-era `debug_assert` stood
/// in for — so a row that reaches a card in a library, a hand or on the stack
/// says so with `ZoneSet::ALL`. Before RF the zone half was not asked and a
/// battlefield-scoped row reached everything by accident.
pub fn rest_in_peace() -> Arc<CardData> {
    CardDataBuilder::new("Rest in Peace")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("When this enchantment enters, exile all graveyards.\nIf a card or token would be put into a graveyard from anywhere, exile it instead.")
        .ability(static_replacement(
            "If a card or token would be put into a graveyard from anywhere, exile it instead.",
            ReplacementDef::new(
                EventPattern::ZoneChange {
                    change: ZoneChangePattern { from: None, to: Some(Zone::Graveyard), act: None },
                    object: None,
                },
                // "a card or token" — no owner clause, no type clause, and
                // no `Not(Token)`. Everything that would hit a graveyard,
                // "from anywhere".
                ObjectSet::filter_in(ObjectFilter::All, ZoneSet::ALL),
                Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
            ),
        ))
        .build()
}
