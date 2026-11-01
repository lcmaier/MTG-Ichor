//! Guildpact (`gpt`), released 2006-02-03.

use std::sync::Arc;

use crate::cards::authoring::{non, static_replacement, token};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{ObjectFilter, ObjectSet, PlayerRef};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use crate::types::zones::{Zone, ZoneChangePattern, ZoneSet};

/// **Leyline of the Void** — `{2}{B}{B}` Enchantment.
///
/// "If a card would be put into an opponent's graveyard from anywhere, exile it
/// instead."
///
/// # Why *this* second card and not another copy of the shape
///
/// A second card of the same shape buys nothing
/// (`engineering-practices.md#how-many-cards-a-mechanic-owes`). This one
/// disagrees with [`rest_in_peace`] on both axes the CR cares about: it
/// excludes tokens, and it is **owner-scoped** where that one is global. The
/// second difference is the load-bearing one — CR 616.1 asks the
/// *affected object's controller*, so a global effect and an opponent-scoped
/// effect can be offered to different players for different objects, which a
/// same-scope pair would never exercise.
///
/// # `ByOwner`, and why `ByController` would have been a different card
///
/// "an opponent's graveyard" is a question about **ownership**: a card put into
/// a graveyard goes to its owner's (CR 400.3), whoever was controlling it. The
/// two answers diverge the moment control moves, and the registered pool can
/// already reach that — Act of Treason steals a creature, it dies, and it goes
/// to the graveyard of the player who owns it, so a Leyline controlled by the
/// thief's opponent should replace a death the thief is controlling.
/// `ObjectFilter::ByOwner` is new for this card and is why this one is not
/// the "zero engine change" its sizing predicted.
///
/// # What is deferred
///
/// "If this card is in your opening hand, you may begin the game with it on the
/// battlefield" is a static ability functioning in another zone —
/// `replacement-architecture.md` §3.3's source 2, deferred past Phase RE. The
/// card is castable for `{2}{B}{B}` here and does nothing before it resolves.
///
/// **"From anywhere" is literal here too, and for a sharper reason.** This card's
/// filter is `Not(Token)` and `ByOwner`, which read `GameObject.is_token` and
/// `GameObject.owner` — both present in every zone, neither a characteristic the
/// layer system computes. So no part of it needs the object to be a permanent,
/// and the affected set reaches `ZoneSet::ALL` for [`rest_in_peace`]'s reason.
///
/// **Kalitas stays `from: Battlefield`, and that is not the same call.** CR
/// 700.4 *defines* "dies" as "put into a graveyard from the battlefield", so the
/// clause is the card's text rather than a narrowing of it. The filter language
/// still cannot describe a card by a *characteristic* off the battlefield — a
/// hypothetical "if a red card would be put into a graveyard" needs the
/// hidden-zone work — but neither card here asks it to.
pub fn leyline_of_the_void() -> Arc<CardData> {
    CardDataBuilder::new("Leyline of the Void")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("If this card is in your opening hand, you may begin the game with it on the battlefield.\nIf a card would be put into an opponent's graveyard from anywhere, exile it instead.")
        .ability(static_replacement(
            "If a card would be put into an opponent's graveyard from anywhere, exile it instead.",
            ReplacementDef::new(
                EventPattern::ZoneChange {
                    change: ZoneChangePattern { from: None, to: Some(Zone::Graveyard), act: None },
                    object: None,
                },
                ObjectSet::filter_in(
                    // "a card" — CR 111.1, a token is not one; "an opponent's
                    // graveyard" — CR 400.3 sends it to the owner's, so this is
                    // ownership and not control.
                    non(token()).and(ObjectFilter::ByOwner(PlayerRef::Opponent)),
                    // "from anywhere".
                    ZoneSet::ALL,
                ),
                Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
            ),
        ))
        .build()
}
