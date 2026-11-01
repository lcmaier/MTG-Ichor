//! Streets of New Capenna (`snc`), released 2022-04-29.

use std::sync::Arc;

use crate::cards::authoring::{mill, static_replacement, target_player};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{AmountExpr, PlayerSet};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, ReplacementDef, Rewrite};

/// Angel of Suffering — {3}{B}{B}
///
/// > Flying
/// >
/// > If damage would be dealt to you, prevent that damage and mill twice that
/// > many cards.
///
/// The phase's only prevention consumer, and the only effect in it that is
/// about **no object at all**: `ObjectSet::Fixed(vec![])` beside
/// `PlayerSet::You`. Its rider is what makes `Rider.subject` an object or a player
/// rather than an `Option<ObjectId>` — "mill" needs to name the player the
/// damage was aimed at (`codebase-state.md` item 27) — and "twice that many"
/// is `AmountExpr::Multiply(ReplacedAmount, 2)`, read off the event as the
/// CR 616.1 loop saw it.
///
/// # The rulings, and where each is tested
///
/// - *If you would mill more cards than are in your library, you mill all
///   cards in your library* → CR 609.3's "only as much as possible", in
///   `Primitive::Mill`.
/// - *Damage that would be dealt to you will be prevented even if you can't
///   mill twice that many* → the prevention is not conditional on the rider,
///   which falls out of §4.1a's queue-then-perform order.
/// - *If the damage can't be prevented for some reason, you'll still mill
///   twice that many* → **RD-4's** dovetail test, beside Reverse Damage: the
///   rider is unconditional once queued (CR 615.12), and RD-1 has no way to
///   make damage unpreventable.
pub fn angel_of_suffering() -> Arc<CardData> {
    CardDataBuilder::new("Angel of Suffering")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black], 3))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Nightmare))
        .subtype(Subtype::Creature(CreatureType::Angel))
        .power_toughness(5, 3)
        .keyword_flag(KeywordFlag::Flying)
        .rules_text(
            "Flying\nIf damage would be dealt to you, prevent that damage and mill twice \
             that many cards.",
        )
        .ability(static_replacement("If damage would be dealt to you, prevent that damage and mill twice that many cards.",
            // About a player and no object at all: a `PlayerSet` alone says
            // that, where a `Filter` would have to describe an empty set and
            // `SourceOnly` would make the Angel shield *itself*.
            ReplacementDef::new(
                EventPattern::DealDamage { source: None, combat: None },
                PlayerSet::You,
                Rewrite::Prevent,
            )
            .with_then(mill(
                // The rider's single resolved target is the event's subject —
                // the player the damage was aimed at, which for this card is
                // always the Angel's controller and for the next one on this
                // path may not be.
                target_player(),
                AmountExpr::Multiply(Box::new(AmountExpr::ReplacedAmount), 2),
            )),
        ))
        .build()
}
