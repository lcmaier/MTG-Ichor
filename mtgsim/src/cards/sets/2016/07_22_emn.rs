//! Eldritch Moon (`emn`), released 2016-07-22.

use std::sync::Arc;

use crate::cards::authoring::{attacks, create_tokens, static_replacement, triggered_ability, whenever, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{ObjectFilter, ObjectSet, PlayerRef, TokenDef};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use crate::types::restriction::SourceFilter;
use crate::types::triggers::TriggerSubject;
use crate::types::zones::{Zone, ZoneChangeAct, ZoneChangePattern, ZoneSet};

/// Hanweir Garrison — {2}{R}
/// Creature — Human Soldier 2/3
///
/// > Whenever this creature attacks, create two 1/1 red Human creature tokens
/// > that are tapped and attacking.
/// > (Melds with Hanweir Battlements.)
///
/// It is one half of a meld pair, and the reminder text, which has no game
/// function and is not in its rules text (CR 207.2), is all it prints of
/// meld: Hanweir Battlements carries the meld ability, so nothing on this card
/// is dead.
///
/// It exercises tokens put onto the battlefield attacking, each defender chosen
/// as it enters (CR 508.4).
///
/// # The rulings, and where each is tested
///
/// In `phase_tr5f_integration_test`.
/// - *Each token's player, planeswalker or battle is chosen as the tokens are
///   created, and needn't be Hanweir Garrison's or the other token's* (#1) →
///   `each_token_chooses_what_it_attacks_at_four_seats`.
/// - *The tokens were never declared as attacking creatures* (#2) →
///   `a_token_put_onto_the_battlefield_attacking_never_attacked`.
/// - *Another attack trigger resolves before or after it, and a targeted one
///   can't target the tokens* (#3) →
///   `an_attack_trigger_beside_it_cannot_target_the_tokens`.
/// - #4 points at the Eldritch Moon mechanics article, a disposition.
pub fn hanweir_garrison() -> Arc<CardData> {
    let text = "Whenever this creature attacks, create two 1/1 red Human creature tokens that are tapped and attacking.";
    let two_humans =
        create_tokens(you(), 2, TokenDef::creature(1, 1, &[Color::Red], &[CreatureType::Human]).tapped_and_attacking());
    CardDataBuilder::new("Hanweir Garrison")
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Soldier))
        .power_toughness(2, 3)
        .rules_text(text)
        .ability(triggered_ability(text, whenever(attacks(TriggerSubject::ThisObject), two_humans)))
        .build()
}

/// Nephalia Academy — Land
///
/// > If a spell or ability an opponent controls causes you to discard a card,
/// > you may reveal that card and put it on top of your library instead of
/// > putting it anywhere else.
/// >
/// > {T}: Add {C}.
///
/// **`ReplacementDef::by`'s printed customer, and the one card in the
/// "causes you to discard" family that does not need CR 113.6.** It is a
/// Land, so the battlefield sweep finds it, and its `ObjectSet::Filter`
/// reaches a card in its controller's hand because it says so —
/// `ZoneSet::HAND`, which `set_affects` has asked since RF. The other sixteen
/// are either on a card in hand (the Dodecapod family, critical-path item 6a)
/// or need a second facility of their own.
///
/// Three things the def says, each a different rule:
///
/// - **`by`** is CR 101.2's provenance — "a spell or ability an opponent
///   controls" — and it is what makes the card leave CR 514.1's cleanup
///   discard alone: a turn-based action has no controller, so no
///   `SourceFilter` matches it. That is the printed behavior, and it is why
///   Library of Leng has to print "you have no maximum hand size" as a
///   separate sentence rather than relying on this one.
/// - **`optional`** is "you **may**", so this is a real CR 616.1 prompt of the
///   affected player rather than a forced redirection.
/// - **`to: Zone::Library`** is "on top of your library": a library is a
///   `Vec` whose last element is its top, and `add_to_zone_collection` pushes,
///   so a card moved there arrives on top. `Primitive::Mill` reads the same
///   end.
///
/// **"You may reveal that card" is not modeled, and nothing is lost today.**
/// A reveal is a disclosure, and this engine has no per-viewer visibility at
/// all (`backlog.md` §2.9, whose own verdict is that nothing in the tree
/// answers "can player N see this object?"). Every decision provider already
/// sees the whole board, so the reveal is a no-op — the same no-op CR 701.22a's
/// "look at the top N cards" is for [`opt`] in this same file. When §2.9 lands,
/// this card is one of its tests.
///
/// No printed rulings (Scryfall, 2026-09-14).
pub fn nephalia_academy() -> Arc<CardData> {
    CardDataBuilder::new("Nephalia Academy")
        .card_type(CardType::Land)
        .rules_text(
            "If a spell or ability an opponent controls causes you to discard a card, you may reveal that card and put it on top of your library instead of putting it anywhere else.\n{T}: Add {C}.",
        )
        .ability(static_replacement(
            "If a spell or ability an opponent controls causes you to discard a card, you may reveal that card and put it on top of your library instead of putting it anywhere else.",
            ReplacementDef::new(
                EventPattern::ZoneChange {
                    // "instead of putting it **anywhere else**" — the
                    // destination is not asked about, only the departure
                    // and the act.
                    change: ZoneChangePattern { from: Some(Zone::Hand), to: None, act: Some(ZoneChangeAct::Discarded) },
                    object: None,
                },
                // "causes **you** to discard a card": the cards in this
                // permanent's controller's hand. Ownership rather than
                // control, because a card in a hand has no controller and
                // CR 400.3 sends a discard to its owner's graveyard.
                ObjectSet::filter_in(ObjectFilter::ByOwner(PlayerRef::You), ZoneSet::HAND),
                Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Library }),
            )
            .caused_by(SourceFilter::ControlledBy(PlayerRef::Opponent))
            .optional(),
        ))
        .mana_ability_single(ManaType::Colorless)
        .build()
}
