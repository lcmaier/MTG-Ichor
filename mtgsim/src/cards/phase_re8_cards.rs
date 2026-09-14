//! Cards for Phase RE-8 — the producers (`replacement-architecture.md` §9).
//!
//! **Two keyword actions get their first producer, and one replacement effect
//! gets its first cause.** CR 701.9's discard and CR 701.22's scry were stubs
//! returning `Err`; the cleanup step was the only discard in the engine and
//! nothing scried at all. Five cards, on three axes:
//!
//! | Card | What it is the first of | CR |
//! |---|---|---|
//! | [`mind_rot`] | a discard the affected player chooses | 701.9b, default |
//! | [`hymn_to_tourach`] | a discard nobody chooses | 701.9b, "at random" |
//! | [`nephalia_academy`] | a replacement scoped by what *caused* the event | 101.2's "by" |
//! | [`opt`] | a scry | 701.22a |
//! | [`eligeth_crossroads_augur`] | a replacement *of* a scry | 614.1a |
//!
//! # What is not here, and what each waits for
//!
//! **Dodecapod, Wilt-Leaf Liege, Loxodon Smiter, Nullhide Ferox and Obstinate
//! Baloth** — the "if a spell or ability an opponent controls causes you to
//! discard this card, put it onto the battlefield instead" family, and every
//! printed customer of a to-battlefield entry substitution (Scryfall,
//! 2026-09-14: eleven cards say "onto the battlefield instead" and the other
//! six are a sorcery's own instruction). Their clause is on a card in *hand*,
//! and `replacement::gather`'s five sources are the CR 903.9b rule, the
//! entering permanent, the battlefield sweep, the counters and the registry —
//! none of which asks a card off the battlefield. That is CR 113.6, sized at
//! `replacement-architecture.md` §11 item 9 and owned by critical-path item 6a.
//!
//! **Library of Leng** needs a maximum hand size (`backlog.md` §2.15) and
//! **Guerrilla Tactics** needs critical-path item 6; the board they make together is
//! `plans/atomic-tests/supplemental-docs/603-2f-complexity.md`, whose
//! discriminator turns out to be neither — it is §2.9's information model, on
//! CR 603.10a's own words ("an object that all players can see").
//!
//! # What a random deck can draw
//!
//! **Mind Rot and Opt are the pooled pair**, and they are the pool's first
//! discard outside the cleanup step and its first scry. Each opens a path no
//! measured game has walked: `ZoneChange { cause: Discarded }` from a
//! resolution, and `GameAction::Scry` at all. Hymn to Tourach stays out
//! because a second two-card discard would double the first's measurement
//! rather than add to it; Nephalia Academy and Eligeth stay out as a land whose
//! ability needs an opponent's discard spell and a six-drop whose ability needs
//! a scry, neither of which the pool can arrange for them.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::ids::new_ability_id;
use crate::types::keywords::KeywordFlag;
use crate::types::effects::{
    ObjectSet, AmountExpr, DiscardChooser, Effect, EffectRecipient, ObjectFilter, PlayerRef,
    PlayerSet, Primitive, SelectionFilter, TargetCount,
};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{
    EventPattern, GameActionTemplate, ReplacementDef, Rewrite, TemplateAmount,
};
use crate::types::restriction::SourceFilter;
use crate::types::zones::{Zone, ZoneChangeCause};

/// A spell whose whole text is "target player discards N cards", differing
/// only in who picks (CR 701.9b).
fn target_player_discards(n: u64, chooser: DiscardChooser) -> AbilityDef {
    AbilityDef {
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
        id: new_ability_id(),
        ability_type: AbilityType::Spell,
        costs: Vec::new(),
        effect: Effect::Atom(
            Primitive::Discard(AmountExpr::Fixed(n), chooser),
            EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
        ),
    }
}

/// Mind Rot — {2}{B}
/// Sorcery
///
/// > Target player discards two cards.
///
/// **CR 701.9b's default chooser, and the pool's first discard outside the
/// cleanup step.** Two cards rather than one is what makes the batch
/// observable: a discard of two is one event with two members, which is what
/// CR 603.2c's "whenever one or more cards are discarded" will read once
/// rather than twice.
///
/// No printed rulings (Scryfall, 2026-09-14). The ruling it is tested against
/// belongs to another card: Notion Thief's *"the opponent still discards"* was
/// asserted in RE-2 against a fixture draw-then-discard resolution, because
/// `Primitive::Discard` did not exist. It is a printed board now — an opponent
/// draws under a Notion Thief and then Mind Rot takes two cards — and the
/// fixture goes.
pub fn mind_rot() -> Arc<CardData> {
    CardDataBuilder::new("Mind Rot")
        .mana_cost(ManaCost::build(&[ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Sorcery)
        .rules_text("Target player discards two cards.")
        .ability(target_player_discards(2, DiscardChooser::Affected))
        .build()
}

/// Hymn to Tourach — {B}{B}
/// Sorcery
///
/// > Target player discards two cards at random.
///
/// **CR 701.9b's second chooser, and the engine's first "at random" anything
/// that is not a shuffle.** The cards come from `GameState::rng` and never from
/// `rand::rng()` (`CLAUDE.md`: randomness is owned, never ambient), which is
/// what `tests/determinism_test.rs` covers by construction — its decks are
/// built from `default_registry`, so a registered Hymn is in every seeded game
/// it plays, and an ambient draw would make the second run of a process differ
/// from the first.
///
/// **No `DecisionProvider` is asked at all**, which is `ATOM-701.9b-001`'s own
/// expected result and not an optimization: the rule gives the affected player
/// no choice to make.
///
/// No printed rulings (Scryfall, 2026-09-14).
pub fn hymn_to_tourach() -> Arc<CardData> {
    CardDataBuilder::new("Hymn to Tourach")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black], 0))
        .color(Color::Black)
        .card_type(CardType::Sorcery)
        .rules_text("Target player discards two cards at random.")
        .ability(target_player_discards(2, DiscardChooser::AtRandom))
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
/// reaches a card in its controller's hand the way every `Filter` already
/// reaches any object in any zone. The other sixteen are either on a card in
/// hand (the Dodecapod family, critical-path item 6a) or need a second facility of their own.
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
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(
                ReplacementDef::new(
                    EventPattern::ZoneChange {
                        // "instead of putting it **anywhere else**" — the
                        // destination is not asked about, only the departure
                        // and the reason.
                        from: Some(Zone::Hand),
                        to: None,
                        cause: Some(ZoneChangeCause::Discarded),
                        object: None,
                    },
                    // "causes **you** to discard a card": the cards in this
                    // permanent's controller's hand. Ownership rather than
                    // control, because a card in a hand has no controller and
                    // CR 400.3 sends a discard to its owner's graveyard.
                    ObjectSet::filter(ObjectFilter::ByOwner(PlayerRef::You)),
                    Rewrite::Instead(GameActionTemplate::ZoneChangeTo {
                        to: Zone::Library,
                        cause: ZoneChangeCause::Discarded,
                    }),
                )
                .caused_by(SourceFilter::ControlledBy(PlayerRef::Opponent))
                .optional(),
            )),
        })
        .mana_ability_single(ManaType::Colorless)
        .build()
}

/// Opt — {U}
/// Instant
///
/// > Scry 1. (Look at the top card of your library. You may put that card on
/// > the bottom.)
/// >
/// > Draw a card.
///
/// **The scry producer, and CR 608.2c's "A, then B" in one resolution.** The
/// two instructions are an `Effect::Sequence`, so the scry is complete before
/// the draw is proposed — which is what makes the Eligeth board below say
/// something: under Eligeth the first instruction becomes a draw too, and the
/// card draws two.
///
/// A scry 1 asks exactly one question — which is CR 701.22a's "put any number
/// of them on the bottom" with one card — and never asks the ordering one,
/// because a group of one has no order to choose (CR 102.2).
///
/// **The reminder text's "look at" is the same non-event as Nephalia Academy's
/// "reveal"**: this engine's decision providers see the whole board, so
/// looking changes nothing until §2.9 exists.
///
/// No printed rulings (Scryfall, 2026-09-14).
pub fn opt() -> Arc<CardData> {
    CardDataBuilder::new("Opt")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 0))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text("Scry 1.\nDraw a card.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::Scry(AmountExpr::Fixed(1)),
                    EffectRecipient::Controller,
                ),
                Effect::Atom(
                    Primitive::DrawCards(AmountExpr::Fixed(1)),
                    EffectRecipient::Controller,
                ),
            ]),
        })
        .build()
}

/// Eligeth, Crossroads Augur — {4}{U}{U}
/// Legendary Creature — Sphinx 5/6
///
/// > Flying
/// >
/// > If you would scry a number of cards, draw that many cards instead.
/// >
/// > Partner (You can have two commanders if both have partner.)
///
/// **The kind-changing `Instead` that reads the replaced event's own amount**,
/// and §10's acid test with a producer that is not a draw:
/// `test_opt_with_eligeth_draws_two_and_never_scrys`. Opt under Eligeth draws
/// two and the log holds no `Scried` line at all, which proves two separate
/// things at once — §4.1a's instruction split, since only the *first* of Opt's
/// two instructions is replaced, and `TemplateAmount::ReplacedAmount` reading
/// an amount off a non-draw event.
///
/// **Partner is not dead text under the name.** CR 702.124 makes it a
/// deck-construction ability with no in-game effect, so a card carrying it has
/// nothing to model in a game (`cost-architecture.md`'s Commander track owns
/// deck construction). Flying is a keyword flag.
///
/// **The other printed "would scry" card is Kenessos, Priest of Thassa** —
/// "scry that many cards plus one instead", which is `Rewrite::Amount` over
/// the same pattern rather than an `Instead`, built here and tested against a
/// fixture. The card itself waits on `backlog.md` §2.9: its second ability
/// looks at the top card of the library and acts on what it is. The cards that
/// *watch* a scry — Elrond, Master of Healing, and Goggles of Night, which
/// scries from a trigger rather than replacing one — are critical-path item 6's.
///
/// Its printed rulings are seven and every one of them is about Partner —
/// color identity, two commanders in the command zone, commander damage
/// counted separately (Scryfall, 2026-09-14). None is about the replacement,
/// and none is testable before the Commander track builds the command zone.
pub fn eligeth_crossroads_augur() -> Arc<CardData> {
    CardDataBuilder::new("Eligeth, Crossroads Augur")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 4))
        .color(Color::Blue)
        .supertype(Supertype::Legendary)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Sphinx))
        .power_toughness(5, 6)
        .keyword_flag(KeywordFlag::Flying)
        .rules_text(
            "Flying\nIf you would scry a number of cards, draw that many cards instead.\nPartner",
        )
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(
                ReplacementDef::new(
                    EventPattern::Scry,
                    ObjectSet::NO_OBJECTS,
                    Rewrite::Instead(GameActionTemplate::DrawCards {
                        // "that many" — CR 615.5's amount, read off the scry.
                        n: TemplateAmount::ReplacedAmount,
                        // "draw" with no player named: the scrying player is
                        // the one who draws.
                        player: None,
                    }),
                )
                .affecting_players(PlayerSet::You),
            )),
        })
        .build()
}
