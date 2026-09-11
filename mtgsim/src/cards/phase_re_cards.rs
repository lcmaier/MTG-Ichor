//! Cards for Phase RE — the remaining event kinds (`replacement-architecture.md` §9).
//!
//! # RE-1 — skips, and the turn queue (CR 614.1b, 614.10, 614.10a, 500.7)
//!
//! **Five printed cards on two axes: which unit a skip names, and where the
//! effect comes from.** The unit is what CR 614.10 replaces — a turn, a phase
//! or a step — and the source decides whether the row can be stripped, counted
//! or targeted:
//!
//! | Card | Unit | Source | Uses |
//! |---|---|---|---|
//! | [`yawgmoths_bargain`] | the draw step | a static ability, `You` | `Static` |
//! | [`eon_hub`] | the upkeep step | a static ability, `Everyone` | `Static` |
//! | [`meditate`] | a turn | a resolution, `You` | `Once`, `Indefinite` |
//! | [`time_walk`] | — (it *makes* a turn) | a resolution | — |
//! | [`moment_of_silence`] | the combat phase | a resolution, a **target** | `Once`, `UntilEndOfTurn` |
//!
//! Meditate is the first `Duration::Indefinite` row in the crate that ends by
//! *use* and never by time, which is what "your **next** turn" means; the
//! variant has existed since the registry was written with nothing to carry.
//! Time Walk is in the same PR as the skips rather than in its own because
//! the Meditate-then-Time-Walk board — a skip consuming an extra turn,
//! CR 614.10a's "the first occurrence that isn't skipped" — is what proves the
//! queue and the pipeline meet, and building them apart means rewriting
//! `advance_turn` twice.
//!
//! **Chronatog and Relentless Assault are out**, each for a facility rather
//! than for size. Chronatog's "Activate only once each turn" is an activation
//! limit `ActivationRestriction` does not have and this PR has exactly one
//! customer for; Relentless Assault's "an additional combat phase followed by
//! an additional main phase" is CR 500.8's extra *phases*, the turn queue's
//! second level (`backlog.md` §2.17), which §9 cut from this phase and the
//! review re-opened as an ordering call.
//!
//! # The rulings pass (Scryfall, 2026-09-11)
//!
//! Every ruling on all five cards, with what became of it
//! (`engineering-practices.md` §3.4). Yawgmoth's Bargain and Time Walk's
//! ordering ruling are the two that carry no board of their own.
//!
//! - **Yawgmoth's Bargain** — Scryfall lists no rulings. Its tests are the
//!   rule's: the draw step's *contents* are not proposed at all
//!   (`ATOM-614.10-001`).
//! - **Eon Hub**, *"The upkeep step is skipped entirely. The turn proceeds from
//!   untap step to draw step."* → the event log, asserted as the absence of a
//!   `StepBegin { Upkeep }` between the untap and draw ones.
//! - **Eon Hub**, *"Upkeep-triggered abilities don't trigger, and 'activate
//!   only during your upkeep' abilities can't be activated."* → the first half
//!   is item 6's and falls out — a step that does not begin emits no event for
//!   a trigger to read, which is why this PR goes first. The second half has
//!   **no facility to assert against**: `ActivationRestriction` has no
//!   step-scoped arm, and no registered card carries one. Recorded, not
//!   skipped.
//! - **Eon Hub**, *"Any triggered abilities that triggered during the untap
//!   step will go onto the stack at the start of the draw step."* → item 6's,
//!   for the same reason. Nothing triggers yet. **Both Eon Hub rulings are
//!   booked as two tests item 6 owes** — `codebase-state.md` item 121, which
//!   names the board and sizes them.
//! - **Meditate**, *"You skip one turn as part of the effect."* → one row,
//!   `Uses::Once`, so one turn; and two Meditates skip two, which is
//!   CR 614.10a's own sentence and the board that needed a real queue.
//! - **Time Walk**, *"If multiple 'extra turn' effects resolve in the same
//!   turn, take them in the reverse of the order that the effects resolved."*
//!   → the queue is a stack, tested with two extra turns for two different
//!   players so that the order is observable at all.
//! - **Moment of Silence**, *"The player skips their next combat phase this
//!   turn (if any). If they manage to have two combat phases, then only their
//!   next one combat phase is skipped."* → `Uses::Once`, and the second
//!   sentence is tested against a second `BeginPhase { Combat }` proposal the
//!   fixture makes by moving the cursor, because CR 500.8's extra phases are
//!   unbuilt (`backlog.md` §2.17).
//! - **Moment of Silence**, *"It must be used before the combat phase starts or
//!   it has no effect."* → `ATOM-614.10-002`: a row created during combat meets
//!   no proposal and expires at cleanup unused.
//! - **Moment of Silence**, *"If cast on a player when it is not their turn, it
//!   has no effect."* → falls out of the subject rather than being coded: a
//!   `BeginPhase` event is about the **active** player, so a row scoped to
//!   anyone else watches nothing this turn.
//!
//! # What a random deck can draw
//!
//! Eon Hub is the pooled card: `{5}` colourless, so every deck can cast it, and
//! its effect is a *dropped* turn-structure proposal on every player's upkeep
//! for as long as it is on the battlefield — the first card in
//! `PERFORMANCE_POOL` whose cost is measured in proposals that go nowhere.
//! Yawgmoth's Bargain is registered and stays out: a random agent with one use
//! for its life total empties its library, which is the board RE-6's Laboratory
//! Maniac path wants and a distortion of every fixture until then. Time Walk
//! stays out because an extra turn in every blue deck moves `Avg turns/game` by
//! design. Meditate and Moment of Silence stay out as one-shots whose engine
//! path Eon Hub already opens.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::state::game_state::{PhaseType, StepType};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    AffectedSet, AmountExpr, Duration, Effect, EffectRecipient, PatternFill, PlayerSet, Primitive,
    SelectionFilter, TargetCount,
};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, ReplacementDef, Rewrite};

/// A static ability whose effect is a replacement effect — never a resolution,
/// so it carries no `Duration` and is re-derived off the source's *effective*
/// ability list on every gather.
fn static_replacement(def: ReplacementDef) -> AbilityDef {
    AbilityDef {
        id: new_ability_id(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect: Effect::Replacement(Box::new(def)),
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
    }
}

/// One ability with no costs beyond the ones given.
fn one_shot(ability_type: AbilityType, costs: Vec<Cost>, effect: Effect) -> AbilityDef {
    AbilityDef {
        id: new_ability_id(),
        ability_type,
        costs,
        effect,
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
    }
}

/// "Skip [unit]" as CR 614.10 defines it: *"instead of doing [something], do
/// nothing"*, which CR 614.1b says is a replacement effect and CR 614.6 says is
/// a [`Rewrite::Prevent`].
///
/// No new `Rewrite` arm, for that reason — "replaced with nothing" **is** 614.6,
/// and a `Rewrite::Skip` would be a second spelling of one algebra element
/// (`replacement-architecture.md` §9, RE decision 0).
///
/// The object set is empty on every one of these: CR 614.10's three units are
/// about a *player*, so the scope is a [`PlayerSet`] and nothing else.
fn skip(pattern: EventPattern, players: PlayerSet) -> ReplacementDef {
    ReplacementDef::new(pattern, AffectedSet::NO_OBJECTS, Rewrite::Prevent)
        .affecting_players(players)
}

/// Yawgmoth's Bargain — {4}{B}{B}
/// Enchantment
///
/// > Skip your draw step.
/// > Pay 1 life: Draw a card.
///
/// The plainest static skip there is, and the one that shows what a skipped
/// step costs: not the draw alone but the whole step — no `StepBegin`, no
/// turn-based action, no priority round (`ATOM-614.10-001`). Its second
/// ability is here because a card that only subtracted would never be cast by
/// anything, and both halves — `Cost::PayLife` and `Primitive::DrawCards` —
/// already exist.
///
/// Scryfall lists no rulings (2026-09-11).
///
/// **Registered and not pooled.** A random agent with one use for its life
/// total will empty its library, which is the board RE-6's Laboratory Maniac
/// path wants and a distortion of every fixture until it lands.
pub fn yawgmoths_bargain() -> Arc<CardData> {
    CardDataBuilder::new("Yawgmoth's Bargain")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Black], 4))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("Skip your draw step.\nPay 1 life: Draw a card.")
        .ability(static_replacement(skip(
            EventPattern::BeginStep { step: Some(StepType::Draw) },
            PlayerSet::You,
        )))
        .ability(one_shot(
            AbilityType::Activated,
            vec![Cost::PayLife(1)],
            Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Controller),
        ))
        .build()
}

/// Eon Hub — {5}
/// Artifact
///
/// > Players skip their upkeep steps.
///
/// The static whose scope is **everyone**, which is what a per-player counter
/// could never have expressed: the row is one effect that applies to each
/// player's upkeep in turn, gathered off the artifact's *effective* ability
/// list, so Humility or CR 305.7 taking the ability away takes the skip away
/// with it. CR 616.1's chooser is the affected player — the one whose upkeep it
/// is — so on a four-player table this asks nobody, four times a round.
///
/// Its rulings are in this module's doc comment; two of the three are item 6's.
///
/// **The pooled card of the PR.** Colourless at five, so every deck can cast
/// it, and every player's upkeep for the rest of the game is then a proposal
/// that goes nowhere — the first measured card whose cost is a *dropped*
/// turn-structure event.
pub fn eon_hub() -> Arc<CardData> {
    CardDataBuilder::new("Eon Hub")
        .mana_cost(ManaCost::build(&[], 5))
        .card_type(CardType::Artifact)
        .rules_text("Players skip their upkeep steps.")
        .ability(static_replacement(skip(
            EventPattern::BeginStep { step: Some(StepType::Upkeep) },
            PlayerSet::Everyone,
        )))
        .build()
}

/// Meditate — {2}{U}
/// Instant
///
/// > Draw four cards. You skip your next turn.
///
/// The **consumable** skip, and the first row in the crate whose duration is
/// `Indefinite` and whose end is a use: "your next turn" is not a length of
/// time, so nothing but `Uses::Once` can retire it. CR 614.10a's second
/// sentence is the board two of these build — *"if two effects each cause a
/// player to skip their next occurrence, that player must skip the next
/// two"* — and it is the test that could not be written against a per-player
/// counter, which has no way to be offered to CR 616.1 twice.
///
/// Ruling (2026-09-11): *"You skip one turn as part of the effect."* → one row,
/// one turn.
pub fn meditate() -> Arc<CardData> {
    CardDataBuilder::new("Meditate")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Instant)
        .rules_text("Draw four cards. You skip your next turn.")
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::DrawCards(AmountExpr::Fixed(4)),
                    EffectRecipient::Controller,
                ),
                Effect::Atom(
                    // `PlayerSet::You` and a `Controller` recipient: the row is
                    // the def as authored, and CR 109.5 resolves "you" against
                    // the row's controller at each event rather than at
                    // resolution.
                    Primitive::CreateReplacement(
                        Box::new(skip(EventPattern::BeginTurn, PlayerSet::You).once()),
                        Duration::Indefinite,
                        PatternFill::Authored,
                    ),
                    EffectRecipient::Controller,
                ),
            ]),
        ))
        .build()
}

/// Time Walk — {1}{U}
/// Sorcery
///
/// > Take an extra turn after this one.
///
/// The turn queue's only producer, and the card that makes a skip's "next
/// occurrence" mean something: with Meditate's row already in the registry the
/// extra turn is the occurrence that gets skipped, and the natural turn after
/// it begins (CR 614.10a).
///
/// Ruling (2026-09-11): *"If multiple 'extra turn' effects resolve in the same
/// turn, take them in the reverse of the order that the effects resolved."* →
/// CR 500.7's "most recently created turn will be taken first", which is the
/// queue being a stack.
///
/// **Registered and not pooled**: an extra turn in every blue deck moves
/// `Avg turns/game` by design, which is a worse baseline rather than a wider
/// one.
pub fn time_walk() -> Arc<CardData> {
    CardDataBuilder::new("Time Walk")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 1))
        .color(Color::Blue)
        .card_type(CardType::Sorcery)
        .rules_text("Take an extra turn after this one.")
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Atom(Primitive::ExtraTurn, EffectRecipient::Controller),
        ))
        .build()
}

/// Moment of Silence — {W}
/// Instant
///
/// > Target player skips their next combat phase this turn.
///
/// The **targeted** skip and the only phase-scoped one here. Its three rulings
/// are all consequences of the shape rather than special cases: `Uses::Once`
/// makes it one phase, `Duration::UntilEndOfTurn` makes a row that meets no
/// proposal expire unused, and the event's subject being the *active* player
/// makes a row on anybody else watch nothing.
///
/// The affected set is authored empty in both halves, which
/// `Primitive::CreateReplacement`'s `Target` arm requires: the shape is the
/// card's and the player is the resolution's.
pub fn moment_of_silence() -> Arc<CardData> {
    CardDataBuilder::new("Moment of Silence")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text("Target player skips their next combat phase this turn.")
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Atom(
                Primitive::CreateReplacement(
                    Box::new(
                        skip(
                            EventPattern::BeginPhase { phase: Some(PhaseType::Combat) },
                            PlayerSet::Nobody,
                        )
                        .once(),
                    ),
                    Duration::UntilEndOfTurn,
                    PatternFill::Authored,
                ),
                EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
            ),
        ))
        .build()
}
