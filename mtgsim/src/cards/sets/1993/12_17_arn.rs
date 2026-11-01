//! Arabian Nights (`arn`), released 1993-12-17.

use std::sync::Arc;

use crate::cards::authoring::{as_long_as, gets, static_ability, static_replacement, this_object, with_subtype, you_have};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, LandType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{PlayerFact, PlayerSet};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{AmountRewrite, EventPattern, LifeLossCausePattern, ReplacementDef, Rewrite};

/// Ali from Cairo — {2}{R}{R}
/// Creature — Human 0/1
///
/// > Damage that would reduce your life total to less than 1 reduces it to 1
/// > instead.
///
/// **It watches the loss, not the damage**, and its own ruling is the only
/// reason to know that: *"this effect does not prevent damage, it prevents the
/// damage from turning into loss of life. So the full damage is dealt (and
/// abilities that trigger on damage being dealt still trigger), but the full
/// loss of life is not applied."* CR 120.3a's contained `LoseLife` is that
/// loss, built in RD-1 for this card.
///
/// So the def is `LoseLife { cause: Some(Damage) }` and **not** a prevention
/// effect: `is_prevention` tests the pattern for damage first, and this pattern
/// is not damage, which is why Skullcrack's "damage can't be prevented" does
/// not switch it off.
///
/// `cause: Some(Damage)` is also the whole answer to whether it clamps a life
/// *payment*. It does not, twice over: CR 119.4 refuses a payment larger than
/// the life total before any replacement is asked, and a payment is a cost's
/// loss with no damage source, which this pattern does not match. The card agrees
/// — "damage that would reduce" — and so does its first ruling, *"this effect
/// does not apply to effects which reduce your life without doing damage."*
///
/// # The rulings, and where each is tested
///
/// - *"This effect does not apply to effects which reduce your life without
///   doing damage."* → test: a `Primitive::LoseLife` is an effect's loss with
///   no damage source, which the pattern does not match, and the player goes to -7.
///   → `ali_from_cairo_does_not_clamp_a_loss_that_is_not_damage`, with
///   `ali_from_cairo_does_not_clamp_a_life_payment` for the `Cost` arm nothing
///   had watched.
/// - *"The ability works up until Ali enters the graveyard, so if he takes
///   lethal damage or is destroyed at the same time you take damage, the ability
///   helps you."* → test, and it falls out of CR 704.3's decide-then-perform
///   rather than being coded: a batch decides every member against one board.
///   → `ali_helps_on_the_earthquake_that_kills_him`
/// - *"This effect does not prevent damage, it prevents the damage from turning
///   into loss of life. So the full damage is dealt (and abilities that trigger
///   on damage being dealt still trigger), but the full loss of life is not
///   applied."* → **the ruling that decided the card's pattern**, and three
///   tests: the `DamageDealt` event carries the whole amount, Skullcrack does
///   not switch the clamp off, and a lifelinker still gains the full damage.
///   → `the_full_damage_is_still_dealt_and_only_the_loss_is_clamped`,
///   `skullcrack_does_not_turn_off_ali_from_cairo` and
///   `ali_clamps_the_loss_and_lifelink_still_gains_the_full_damage`
pub fn ali_from_cairo() -> Arc<CardData> {
    CardDataBuilder::new("Ali from Cairo")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .power_toughness(0, 1)
        .rules_text("Damage that would reduce your life total to less than 1 reduces it to 1 instead.")
        .ability(static_replacement("Damage that would reduce your life total to less than 1 reduces it to 1 instead.",
            ReplacementDef::new(
                EventPattern::LoseLife { cause: Some(LifeLossCausePattern::Damage) },
                PlayerSet::You,
                Rewrite::Amount(AmountRewrite::LifeFloor(1)),
            ),
        ))
        .build()
}

/// Kird Ape — {R}
/// Creature — Ape, 1/1
/// This creature gets +1/+2 as long as you control a Forest.
///
/// (Oracle text verified on Scryfall, 2026-09-06.)
///
/// **The cheapest printed conditional static there is**, and the one that
/// needs no new condition leaf: "you control a Forest", `PlayerFact::ControlsPermanent`, over an
/// `Implicit` recipient, which is one layer-7c row whose *existence* is a
/// question asked every pass (CR 604.2, `board::static_ability_still_exists`).
/// Asymmetric, so a row applied twice or transposed fails an assertion.
///
/// It reads LI-2's board without depending on anything. A Taiga is a Forest
/// until Blood Moon sets it to Mountain in layer 4, and the Ape's condition
/// is read at 7c against a board where that has already happened — so the
/// bonus is simply gone, two layers later, with no dependency involved
/// (CR 613.8a(a) confines dependency to one layer).
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The pool's first row whose existence is a *condition* rather than an
/// ability lookup, which is a new path in the check and one that runs for
/// every application in every layer of every pass the Ape is on the board
/// for. A one-mana creature any red deck casts on turn one, and the pool
/// already has five basic Forests and four nonbasic lands with the subtype,
/// so both answers happen in a measured game.
pub fn kird_ape() -> Arc<CardData> {
    CardDataBuilder::new("Kird Ape")
        .mana_cost(ManaCost::build(&[ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Ape))
        .power_toughness(1, 1)
        .rules_text("This creature gets +1/+2 as long as you control a Forest.")
        .ability(static_ability(
            "This creature gets +1/+2 as long as you control a Forest.",
            as_long_as(
                you_have(PlayerFact::ControlsPermanent(with_subtype(Subtype::Land(LandType::Forest)))),
                gets(this_object(), 1, 2),
            ),
        ))
        .build()
}
