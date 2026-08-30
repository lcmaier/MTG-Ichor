//! Cards for Phase RB — the CR 616.1 replacement pipeline.
//!
//! **One card, chosen for difficulty rather than convenience.** RB's other two
//! consumers are counters (CR 122.1c/d/h) and regeneration (CR 701.19), and
//! neither has any card text at all — the rules state their effects verbatim.
//! That is good for shipping and bad for design: `EventPattern` would have been
//! defined under no pressure whatsoever, and the first real card would then
//! have found out what it could not express
//! (`replacement-architecture.md` §8c, "should the grammar work move earlier?").
//!
//! Kalitas is the pressure. It is the only replacement effect in reach with a
//! **two-sided filter** (a quality of the dying creature *and* whose it is), a
//! **`then` half**, and a **`PermanentFilter` leaf that does not exist yet**
//! (`nontoken`) — and it forces that leaf decision at the moment the "two
//! customers before a variant" guard is cheapest to apply.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::{
    AffectedSet, AmountExpr, Effect, EffectRecipient, PermanentFilter, PlayerRef, Primitive,
    TokenDef,
};
use crate::types::ids::new_ability_id;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use crate::types::zones::{Zone, ZoneChangeCause};

/// Kalitas, Traitor of Ghet — {2}{B}{B}
/// Legendary Creature — Vampire Warrior, 3/4
///
/// Lifelink
/// If a nontoken creature an opponent controls would die, instead exile that
/// card and create a 2/2 black Zombie creature token.
/// {2}{B}, Sacrifice another Vampire or Zombie: Put two +1/+1 counters on Kalitas.
///
/// (Oracle text verified on Scryfall, 2026-08-26.)
///
/// # What each half of the replacement is, and why
///
/// - **"would die"** is CR 700.4: "a permanent is put into a graveyard from the
///   battlefield". Not a destruction — sacrifice, lethal damage, zero toughness
///   and the legend rule all count, and CR 122.1h's finality counter watches the
///   same event for the same reason. So the pattern is a `ZoneChange` with
///   `from: Battlefield, to: Graveyard` and **no cause**.
/// - **"a nontoken creature an opponent controls"** is the shield's boundary,
///   not the pattern: `AffectedSet::Filter`, which is CR 614.12's "a general
///   subset of permanents" as opposed to `SourceOnly`'s "only that permanent".
///   `PlayerRef::Opponent` resolves as "controlled by someone who isn't you",
///   which is CR 102.2 in a two-player game and CR 102.3's *set* in
///   multiplayer, with no `bool` anywhere.
/// - **"instead exile that card"** is `Rewrite::Instead`, retargeting the
///   destination. The card really is exiled rather than merely not dying, so a
///   `Prevent` would be wrong twice over — the creature would stay on the
///   battlefield.
/// - **"and create a 2/2 black Zombie"** is the `then` rider (CR 615.5), and it
///   belongs to the *application* rather than to the event's survival: CR
///   615.12 keeps it once queued. Its `EffectRecipient::Controller` names
///   Kalitas's controller, not the dying creature's — the rider resolves
///   against a `ResolutionContext` whose source is Kalitas.
///
/// # Two things this card pins that no other RB consumer could
///
/// Kalitas's own printed ruling is §4.2's acid test: when several opponent
/// creatures die at once, **every one of them is exiled and each makes a
/// Zombie**. One static replacement, applied once per death — because CR 614.5
/// is per *event* and batch members are separate events. A batch-wide applied
/// set exiles the first and sends the rest to the graveyard, and nothing else
/// in Phase RB can tell the two apart, because a counter-derived effect is
/// keyed per permanent either way.
///
/// And the `nontoken` leaf is not derivable from anything else: CR 707.2
/// excludes tokenness from copiable values, so it is a property of the
/// `GameObject` that no layer walk can reach and no frame can carry.
///
/// # In the stress pool, not the frozen one
///
/// Registered, so `fuzz_games --pool stress` plays it; absent from
/// `PERFORMANCE_POOL`, so it does not move the recorded A/B baseline. Keeping
/// it out of the registry entirely would have kept it out of both pools *and*
/// out of `card_pool_lowering_test`, which is the check that its replacement
/// ability lowers at all.
pub fn kalitas_traitor_of_ghet() -> Arc<CardData> {
    CardDataBuilder::new("Kalitas, Traitor of Ghet")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Vampire))
        .subtype(Subtype::Creature(CreatureType::Warrior))
        .power_toughness(3, 4)
        .keyword(KeywordFlag::Lifelink)
        .rules_text(
            "Lifelink\n\
             If a nontoken creature an opponent controls would die, instead exile that \
             card and create a 2/2 black Zombie creature token.",
        )
        .ability(AbilityDef {
            is_characteristic_defining: false,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(
                ReplacementDef::new(
                    // CR 700.4's "dies", not CR 701.8's "destroyed".
                    EventPattern::ZoneChange {
                        from: Some(Zone::Battlefield),
                        to: Some(Zone::Graveyard),
                        cause: None,
                        object: None,
                    },
                    AffectedSet::Filter {
                        filter: PermanentFilter::And(
                            Box::new(PermanentFilter::ByType(CardType::Creature)),
                            Box::new(PermanentFilter::And(
                                Box::new(PermanentFilter::Not(Box::new(PermanentFilter::Token))),
                                Box::new(PermanentFilter::ByController(PlayerRef::Opponent)),
                            )),
                        ),
                    },
                    Rewrite::Instead(GameActionTemplate::ZoneChangeTo {
                        to: Zone::Exile,
                        cause: ZoneChangeCause::Exiled,
                    }),
                )
                .with_then(Effect::Atom(
                    Primitive::CreateToken(zombie_token(), AmountExpr::Fixed(1)),
                    // Kalitas's controller, not the dying creature's. The rider
                    // resolves against a context whose source is Kalitas, so
                    // `Controller` is CR 109.5's "you" for this ability.
                    EffectRecipient::Controller,
                )),
            )),
        })
        .build()
    // The {2}{B} sacrifice ability is deliberately absent: it needs
    // `AdditionalCost::Sacrifice` with a subtype filter, which is cost work
    // rather than replacement work, and leaving it off does not weaken anything
    // the replacement effect is here to test.
}

/// The 2/2 black Zombie Kalitas makes.
fn zombie_token() -> TokenDef {
    TokenDef {
        name: "Zombie".to_string(),
        colors: vec![Color::Black],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Zombie)],
        power: 2,
        toughness: 2,
        keyword_flags: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kalitas_watches_dying_rather_than_being_destroyed() {
        // The distinction is the card's, not ours: CR 700.4's "dies" covers
        // sacrifice and zero toughness, which CR 701.8b's "destroyed" does not.
        // A `cause` on the pattern would silently narrow the card to
        // destruction only, and every sacrificed creature would reach the
        // graveyard.
        let card = kalitas_traitor_of_ghet();
        let Effect::Replacement(def) = &card.abilities[0].effect else {
            panic!("expected a replacement, got {:?}", card.abilities[0].effect);
        };
        assert_eq!(
            def.pattern,
            EventPattern::ZoneChange {
                from: Some(Zone::Battlefield),
                to: Some(Zone::Graveyard),
                cause: None,
                object: None,
            },
        );
    }

    #[test]
    fn kalitas_shields_a_general_subset_rather_than_itself() {
        // CR 614.12's distinction, and §11 item 2's reason for reusing
        // `AffectedSet`: `SourceOnly` would make Kalitas replace its *own*
        // death, which is the opposite of what it does.
        let card = kalitas_traitor_of_ghet();
        let Effect::Replacement(def) = &card.abilities[0].effect else {
            panic!("expected a replacement");
        };
        assert!(matches!(def.affected, AffectedSet::Filter { .. }));
    }
}
