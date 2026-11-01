//! Oath of the Gatewatch (`ogw`), released 2016-01-22.

use std::sync::Arc;

use crate::cards::authoring::{an_opponent_controls, create_tokens, creature, non, static_replacement, token, you};
use crate::cards::tokens::zombie_token;
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::effects::ObjectSet;
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use crate::types::zones::{Zone, ZoneChangePattern};

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
///   not the pattern: `ObjectSet::Filter`, which is CR 614.12's "a general
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
///   615.12 keeps it once queued. Its `you()` names
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
        .keyword_flag(KeywordFlag::Lifelink)
        .rules_text("Lifelink\nIf a nontoken creature an opponent controls would die, instead exile that card and create a 2/2 black Zombie creature token.\n{2}{B}, Sacrifice another Vampire or Zombie: Put two +1/+1 counters on Kalitas.")
        .ability(static_replacement(
            "If a nontoken creature an opponent controls would die, instead exile that card and create a 2/2 black Zombie creature token.",
            ReplacementDef::new(
                // CR 700.4's "dies", not CR 701.8's "destroyed".
                EventPattern::ZoneChange {
                    change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: None },
                    object: None,
                },
                ObjectSet::battlefield_filter(creature().and(non(token()).and(an_opponent_controls()))),
                Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
            )
            .with_then(create_tokens(
                // Kalitas's controller, not the dying creature's. The rider
                // resolves against a context whose source is Kalitas, so
                // `you` is CR 109.5's "you" for this ability.
                you(),
                1,
                zombie_token(),
            )),
        ))
        .build()
    // The {2}{B} sacrifice ability is deliberately absent: it needs
    // `AdditionalCost::Sacrifice` with a subtype filter, which is cost work
    // rather than replacement work, and leaving it off does not weaken anything
    // the replacement effect is here to test.
}
