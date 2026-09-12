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
//! second level, which §9 cut from this phase and RE-1's review took back in
//! as **RE-10** — where Aggravated Assault is the card, because Relentless
//! Assault also needs "creatures that attacked this turn" and nothing tracks
//! it.
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
//!
//! # RE-2 — draw (CR 614.11, 614.11a, 121.2, 121.2a, 121.6a/b, 616.1g)
//!
//! **Four printed cards on two axes: which of the two draw events the effect
//! watches, and whether the replacement keeps the draw or moves it.** CR 121.2a
//! makes the instruction an event of its own, and Alms Collector's ruling is
//! what makes the pair honest: *"count how many times the word 'draw' is
//! used."*
//!
//! | Card | Watches | Rewrite | Keeps the draw? |
//! |---|---|---|---|
//! | [`thought_reflection`] | any individual draw | `Instead(DrawCards { n: 2 })` | yes |
//! | [`teferis_ageless_insight`] | an individual draw that is not the draw step's first | `Instead(DrawCards { n: 2 })` | yes |
//! | [`alms_collector`] | an instruction of two or more | `Instead(DrawCards { n: 1 })` + a rider | yes, cut to one, plus one for you |
//! | [`notion_thief`] | an opponent's individual draw, not their draw step's first | `Instead(DrawCards { n: 1, player: You })` | yes, with a new subject |
//!
//! **Both "and" cards were mis-filed, and the same clause of CR 614.5 fixes
//! both.** A replacement gets one opportunity to affect "an event **or any
//! modified events that may replace that event**", so whatever the rewrite
//! outputs carries the applied set and whatever a rider proposes does not.
//! Notion Thief was `Prevent` plus a rider until RE's sizing; Alms Collector
//! was `Prevent` plus *two* riders until this PR's tests ran. Each card's own
//! ruling names the loop that encoding produces — two Thieves trading a draw
//! forever, Alms Collector and an opponent's Thought Reflection trading cards
//! forever — and each is fixed by putting the half that keeps the subject in
//! the rewrite. Notion Thief keeps the draw and changes its subject; Alms
//! Collector keeps the subject and changes the count. Only Alms Collector has a
//! genuinely new subject left over, and that one draw is its rider.
//!
//! # What a random deck can draw
//!
//! Thought Reflection is the pooled card, and it is `{4}{U}{U}{U}`. Seven mana
//! is the most any pooled card has cost, so its `--require` reachability is
//! read and recorded rather than assumed. It opens two paths nothing else in
//! the pool does: the gather sweep on every draw *instruction* and every
//! individual draw while it is on the battlefield, and the first `Instead`
//! whose output is decomposed. Alms Collector is `{3}{W}` and stays out —
//! it applies only to an opponent's multi-card instruction, and the pool's
//! multi-draws are Ancestral Recall and Night's Whisper, so it would sit on the
//! battlefield doing nothing in most games while paying for a creature slot.
//! Teferi's Ageless Insight and Notion Thief stay out as the same shape with a
//! narrower pattern.
//! # RE-3 — life (CR 119.3, 119.7, 119.10, 120.3a's contained loss)
//!
//! **Six printed cards over three events, and the axis is which event an effect
//! watches.** CR 119.3's gain, the loss CR 120.3a contains inside damage, and a
//! gain that CR 101.2 refuses outright:
//!
//! | Card | Watches | Does |
//! |---|---|---|
//! | [`rhox_faithmender`] | a gain, yours | `Amount(Multiplier(2))` |
//! | [`alhammarrets_archive`] | a gain, yours — and a draw | the same, plus RE-2's draw doubler |
//! | [`tainted_remedy`] | a gain, an opponent's | `Instead(LoseLife { ReplacedAmount })` |
//! | [`words_of_worship`] | a draw, yours, once | `Instead(GainLife { Fixed(5) })` |
//! | [`ali_from_cairo`] | the loss inside damage, yours | `Amount(LifeFloor(1))` |
//! | [`skullcrack`] | — (it forbids) | two `Primitive::Restrict` rows |
//!
//! **Ali from Cairo is why the loss inside damage is its own event**, and its
//! own ruling is the only thing that says so: *"this effect does not prevent
//! damage, it prevents the damage from turning into loss of life."* So it is a
//! `LoseLife { cause: Some(Damage) }` and **not** a prevention effect —
//! `ReplacementDef::is_prevention` tests the pattern for damage first, and this
//! pattern is not damage, which is what keeps Skullcrack's second sentence from
//! switching it off. `is_prevention` is untouched by this phase.
//!
//! **Skullcrack's first sentence is a "can't", not a replacement** (CR 101.2,
//! 614.17), and CR 119.7 spells out what that costs the pipeline: *"a
//! replacement effect that would replace a life gain event affecting that
//! player won't do anything."* Both directions fall out of the order of the two
//! checks rather than being coded — a gain replacement finds no event, and a
//! replacement that *produces* a gain has its substitute refused, which is
//! Leyline of Punishment's ruling about Words of Worship.
//!
//! # What a random deck can draw
//!
//! Rhox Faithmender is the pooled card, and it is the first RE consumer that
//! needs no second card to set it up: Knight of Meadowgrain and Vampire
//! Nighthawk are already in the pool, so lifelink's contained `GainLife` —
//! proposed in every measured game since RB with nothing watching it — is a
//! live proposal the moment this is on the battlefield. At `{3}{W}` for a 1/5
//! with lifelink it also doubles the life its own combat damage gains, so the
//! board it opens is one card wide.
//!
//! The other five stay out. Tainted Remedy and Words of Worship are enchantments
//! whose whole effect is a replacement nothing in the pool would trigger often
//! enough to pay for a slot; Ali from Cairo is a 0/1 for four mana whose clamp
//! only matters on a board that is already lethal; Alhammarret's Archive is the
//! same two engine paths as Thought Reflection and this card at five mana; and
//! Skullcrack would put a CR 101.2 restriction row on every turn it is cast,
//! which is RS-1's path rather than a new one. **Leyline of Punishment is
//! deliberately unregistered** — its opening-hand clause is §3.3 source 2's
//! zone-reaching static, which would be dead text under a real card name, and
//! the static form of its other two sentences is the fixture in
//! `tests/phase_rd4_integration_test.rs`. **Bloodletter of Aclazotz is recorded
//! as a shape and not written**: "if an opponent would lose life during your
//! turn" is a conditional static whose condition — it is your turn — the
//! `Condition` AST has no leaf for, with one customer. `EventPattern::LoseLife`
//! and `PlayerSet::Opponents` are built here for it, and its own ruling is what
//! the pattern's `cause: None` is about: *"[it] doesn't change the amount of
//! damage dealt to opponents ... they would lose 2 life, but you'd still gain
//! only 1."*
//!
//! # RE-6 — the game's end (CR 104.2b, 104.3e, 104.4a, 704.5a–c, 704.7, 119.5, 800.4j)
//!
//! **Four printed cards on one axis: what a card does to a loss, or to a win.**
//! CR 104's two ends are proposals from this phase on — the state-based
//! check's four loss loops are batch members, `Primitive::{LoseGame, WinGame}`
//! propose the effect-stated kinds — and the census says every printed card
//! is about *whether* the event happens rather than about which reason it had:
//!
//! | Card | Watches | Does |
//! |---|---|---|
//! | [`laboratory_maniac`] | a draw, yours, while your library is empty | `Instead(PlayerWins)` |
//! | [`exquisite_archangel`] | a loss, yours | `Prevent`, then exile itself and set your life to the starting total |
//! | [`stunning_reversal`] | the next loss, yours, this turn | `Prevent`, then draw seven and set your life to 1 |
//! | [`platinum_angel`] | — (it forbids) | two static `Restriction::Event` rows: you can't lose, your opponents can't win |
//!
//! **`EventPattern::PlayerLoses` carries no reason, and that is the census
//! talking**: Exquisite Archangel's ruling is "any time you would lose the
//! game", Stunning Reversal's is the same sentence, and Platinum Angel's list
//! of what it stops — 0 life, an empty library, ten poison, Phage — is a list
//! of reasons precisely because the card does not distinguish them. What
//! CR 704.7 collapses (a player at 0 life who also drew from an empty library
//! is *one* loss, Lich's Mirror's ruling) the pattern therefore never has to.
//!
//! **Laboratory Maniac is the first replacement effect with an "as long as"**,
//! and the condition is evaluated by the gather at each proposal rather than by
//! the rewrite at application: CR 604.2 makes a conditional static's effect
//! exist while its condition holds, and CR 614.4 asks whether the effect
//! exists *before* the event. CR 121.6a is what makes the board reachable at
//! all — a draw with nothing to draw still reaches the pipeline — and the
//! condition is true at exactly that gather.
//!
//! **Lich's Mirror is not registered**: its "shuffle your hand, your graveyard,
//! and all permanents you own into your library" needs a
//! `Primitive::ShuffleIntoLibrary` over three zones that nothing has built,
//! and the CR 704.7 board its ruling names is built here with the Archangel
//! instead (`ATOM-704.7-001`, partial for that reason).
//!
//! # What a random deck can draw
//!
//! Laboratory Maniac is the pooled card: a three-drop whose static is a draw
//! watcher gated on a library state, so it opens the conditional gather leg
//! on every draw while it is on the battlefield and — the first time in a
//! measured game — makes decking a *win*. Fuzz games deck out rarely, so the
//! rows to read are `--require`'s and the "wins by effect" outcome line rather
//! than the average. **Platinum Angel is registered and not pooled**: a
//! seven-drop that turns every lethal board into a stall would move average
//! turns by design and not by engine, and its whole effect is CR 101.2's
//! refusal, which RS-1's Sigarda already opens. Exquisite Archangel is seven
//! mana for a replacement of an event most games reach exactly once, and
//! Stunning Reversal is a one-shot whose engine path the Archangel already
//! opens; both stay in `stress`.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::state::game_state::{PhaseType, StepType};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    AffectedSet, AmountExpr, Condition, Duration, Effect, EffectRecipient, ObjectFilter,
    PatternFill, PlayerRef, PlayerSet, Primitive, SelectionFilter, TargetCount,
};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::keywords::KeywordFlag;
use crate::types::replacement::{
    AmountRewrite, EventPattern, GameActionTemplate, LifeLossCausePattern, ReplacementDef,
    Rewrite, TemplateAmount,
};
use crate::types::restriction::{ReplacementKindFilter, Restriction, RestrictionDef};
use crate::types::zones::DrawCause;

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

/// A static ability whose replacement effect exists only while `condition`
/// holds — CR 604.2's "as long as", asked by `replacement::gather` at each
/// proposal, the way the layer pass asks it of a Kird Ape.
fn static_conditional_replacement(condition: Condition, def: ReplacementDef) -> AbilityDef {
    AbilityDef {
        id: new_ability_id(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect: Effect::Conditional(condition, Box::new(Effect::Replacement(Box::new(def)))),
        is_characteristic_defining: false,
        activation_restriction: crate::objects::card_data::ActivationRestriction::None,
    }
}

/// A static "can't" (CR 101.2, 614.17) — read off the source's *effective*
/// ability list by `engine::restriction::is_prohibited` at each proposal.
fn static_restriction(what: Restriction) -> AbilityDef {
    AbilityDef {
        id: new_ability_id(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect: Effect::Restriction(Box::new(RestrictionDef::new(what))),
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
/// **The pooled card of the PR.** Colourless at five, so every deck can cast
/// it, and every player's upkeep for the rest of the game is then a proposal
/// that goes nowhere — the first measured card whose cost is a *dropped*
/// turn-structure event.
/// # The rulings, and where each is tested
///
/// - *"The upkeep step is skipped entirely. The turn proceeds from untap step
///   to draw step."* → the event log, asserted as the absence of a
///   `StepBegin { Upkeep }` between the untap and draw ones.
/// - *"Upkeep-triggered abilities don't trigger, and 'activate only during your
///   upkeep' abilities can't be activated."* → the first half falls out and is
///   item 6's: a step that does not begin emits no event for a trigger to read,
///   which is why this PR went first. The second half has **no facility to
///   assert against** — `ActivationRestriction` has no step-scoped arm and no
///   registered card carries one. Recorded, not skipped.
/// - *"Any triggered abilities that triggered during the untap step will go
///   onto the stack at the start of the draw step."* → item 6's, same reason.
///   Both trigger-shaped rulings are booked as two tests item 6 owes
///   (`codebase-state.md` item 121, which names the board and sizes them).
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
/// # The rulings, and where each is tested
///
/// - *"You skip one turn as part of the effect."* → one row, `Uses::Once`, so
///   one turn; and two Meditates skip two, which is CR 614.10a's own sentence
///   and the board that needed a real queue.
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
/// # The rulings, and where each is tested
///
/// - *"The player skips their next combat phase this turn (if any). If they
///   manage to have two combat phases, then only their next one combat phase is
///   skipped."* → `Uses::Once`, and the second sentence is tested against a
///   second `BeginPhase { Combat }` proposal the fixture makes by moving the
///   cursor, because CR 500.8's extra phases are unbuilt. **RE-10 replaces the
///   fixture with Aggravated Assault.**
/// - *"It must be used before the combat phase starts or it has no effect."* →
///   `ATOM-614.10-002`: a row created during combat meets no proposal and
///   expires at cleanup unused.
/// - *"If cast on a player when it is not their turn, it has no effect."* →
///   falls out of the subject rather than being coded: a `BeginPhase` event is
///   about the **active** player, so a row scoped to anyone else watches
///   nothing this turn.
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

// ---------------------------------------------------------------------------
// RE-2 — draw
// ---------------------------------------------------------------------------

/// "If you would draw a card, draw two cards instead" — the shape Thought
/// Reflection and Teferi's Ageless Insight share, differing only in the
/// [`DrawCause`] they except.
///
/// [`GameActionTemplate::DrawCards`]'s `player` is `None` on both: the draw
/// stays with the player who would have drawn it. Notion Thief is the arm's
/// other customer and the one that moves it.
fn draw_two_instead(cause: Option<DrawCause>) -> ReplacementDef {
    ReplacementDef::new(
        EventPattern::DrawCard { cause },
        AffectedSet::NO_OBJECTS,
        Rewrite::Instead(GameActionTemplate::DrawCards { n: 2, player: None }),
    )
    .affecting_players(PlayerSet::You)
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
        .ability(static_replacement(draw_two_instead(None)))
        .build()
}

/// Teferi's Ageless Insight — {2}{U}{U}
/// Legendary Enchantment
///
/// > If you would draw a card except the first one you draw in each of your
/// > draw steps, draw two cards instead.
///
/// **The card [`DrawCause`] exists for**, and the one that shows the stamping
/// rule doing work: `Some(DrawCause::Effect)` watches every draw but the first
/// of a draw step, and "the first" is not a count kept anywhere — it is the
/// first inner of the draw step's instruction, with every later inner stamped
/// `Effect` at every level of decomposition. So beside a Thought Reflection
/// this draws **three** in the draw step: the Reflection doubles the
/// instruction's one draw, the doubled instruction keeps its `TurnBased` cause
/// (CR 614.6), its first inner is the card this excepts, and its second is the
/// card this doubles.
///
/// Legendary, so the two-copy board is Thought Reflection's.
/// # The rulings, and where each is tested
///
/// - *"If a spell or ability causes you to put a card into your hand without
///   specifically using the word 'draw,' it's not a card drawn."* →
///   structurally true and asserted: a `ZoneChangeCause` that is not `Drawn`
///   proposes no draw at all.
///   → `a_card_put_into_hand_is_not_drawn_and_no_draw_replacement_sees_it`
/// - *"If two or more replacement effects would apply to a card-drawing event,
///   the player drawing the card chooses the order."* → as Thought Reflection's.
/// - *"Because [it] is legendary, it's unlikely that one player will control
///   two. However, if that happens, each card that player would draw after the
///   first will result in four cards being drawn."* → the legend rule makes this
///   Thought Reflection's test. This card's own board is the one decision 1
///   named: beside a Thought Reflection in the draw step it draws **three**.
///   → `teferi_beside_thought_reflection_draws_three_in_the_draw_step`, with
///   `teferi_excepts_the_draw_steps_first_card` and
///   `teferi_doubles_a_draw_that_is_not_the_draw_steps` either side of it.
pub fn teferis_ageless_insight() -> Arc<CardData> {
    CardDataBuilder::new("Teferi's Ageless Insight")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .supertype(Supertype::Legendary)
        .rules_text(
            "If you would draw a card except the first one you draw in each of your draw steps, draw two cards instead.",
        )
        .ability(static_replacement(draw_two_instead(Some(DrawCause::Effect))))
        .build()
}

/// Alms Collector — {3}{W}
/// Creature — Cat Cleric 3/3
///
/// > Flash
/// > If an opponent would draw two or more cards, instead you and that player
/// > each draw a card.
///
/// **The only printed customer for [`EventPattern::DrawCards`]**, and the card
/// that makes the instruction event necessary rather than tidy: its own ruling
/// says to count how many times the word "draw" is used, so "draw two cards" is
/// one event this watches and two cantrips are two events it does not.
///
/// **Only half of it is a rider, and the other half is the modified event.**
/// §9 filed the whole of "you and that player each draw a card" under `then` on
/// §3.2d's heterogeneous rule, and its own second ruling refuses that: *"once
/// Alms Collector's replacement effect has modified the effect of a player's
/// Divination, Thought Reflection can double that player's resulting card draw
/// **without Alms Collector's replacement effect applying again**."* CR 614.5
/// gives an effect one opportunity to affect "an event **or any modified events
/// that may replace that event**", and the affected player's one draw is such a
/// modified event — so it has to carry this effect's applied set, which only the
/// rewrite's own output does. As `Prevent` plus two riders it does not, and the
/// board is an infinite loop rather than a wrong number: the rider's draw is
/// doubled back to two, this applies again, and the two effects trade cards
/// until the game is a draw (CR 104.4b) or the engine's stack runs out.
///
/// So the split follows §3.2d's rule read one clause further in. The affected
/// player's half is the **same event with a smaller count** — homogeneous
/// multiplicity, which is a count field — and only the controller's draw is a
/// genuinely new subject, which is what `then` is for. One rider, not two.
///
/// **Flash is not modelled** (`codebase-state.md`'s timing item), and it costs
/// this card's tests nothing: every board here puts it on the battlefield
/// before the draw, which is the only state its replacement reads.
///
/// The two draws come out affected-player-first, because a rider resolves after
/// the event it rides on (§4.1a) — neither the card's text order nor CR 121.2c's
/// turn order, which `codebase-state.md` item 122 owns and sizes.
/// # The rulings, and where each is tested
///
/// - *"[Its] replacement effect applies to an instruction to draw more than one
///   card before any replacement effects apply to individual cards drawn."* →
///   `ATOM-616.1g-001`, with Thought Reflection on the other side of the board.
///   → `alms_collector_applies_to_the_instruction_before_thought_reflection_sees_a_draw`
/// - *"Once a replacement effect has been applied to an event, it can't be
///   applied again to the resulting events ... Thought Reflection can double
///   that player's resulting card draw without [this] applying again."* →
///   **the ruling that changed the card's encoding**, and a test. As `Prevent`
///   plus riders the board is an infinite loop; as an `Instead` on the count it
///   is three cards. → `alms_collector_does_not_apply_again_to_the_draws_it_produced`
/// - *"To determine whether a player is instructed to draw multiple once or
///   instructed multiple times to draw one card, count how many times the word
///   'draw' is used."* → test. Ancestral Recall (pooled) is one "draw" of three
///   and meets it; two `Primitive::DrawCards(1)` in one resolution are two
///   instructions and do not.
///   → `one_instruction_of_two_is_a_different_event_from_two_instructions_of_one`
/// - *"If an effect puts cards into a player's hand without using the word
///   'draw' at all, [it] doesn't apply."* → the same structural fact as
///   Teferi's first ruling, asserted once there.
/// - *"If two players each control [one] and an effect instructs them to each
///   draw two or more cards, the replacement effect of each ... is applied and
///   both players end up drawing two cards."* → test. Two separate
///   instructions, each meeting the other player's Collector and neither meeting
///   its own controller's. → `two_alms_collectors_facing_each_other_both_draw_two`
/// - *"If two players each control [one] and a third player would draw two or
///   more cards, the third player chooses which ... will apply."* → the
///   four-player test, and the only three-player CR 616.1 prompt reachable from
///   two printed cards. → `a_third_player_chooses_which_alms_collector_applies`
pub fn alms_collector() -> Arc<CardData> {
    CardDataBuilder::new("Alms Collector")
        .mana_cost(ManaCost::build(&[ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Cat))
        .subtype(Subtype::Creature(CreatureType::Cleric))
        .power_toughness(3, 3)
        .rules_text(
            "Flash\nIf an opponent would draw two or more cards, instead you and that player each draw a card.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DrawCards { at_least: Some(2) },
                AffectedSet::NO_OBJECTS,
                // "That player draws a card": the same instruction with `n`
                // rewritten to 1, so it keeps this effect's applied set and
                // whatever doubles it afterwards cannot hand it back.
                Rewrite::Instead(GameActionTemplate::DrawCards { n: 1, player: None }),
            )
            .affecting_players(PlayerSet::Opponents)
            // "And you draw a card": the half that is a different player's
            // draw, which nothing about the replaced event can carry.
            .with_then(Effect::Atom(
                Primitive::DrawCards(AmountExpr::Fixed(1)),
                EffectRecipient::Controller,
            )),
        ))
        .build()
}

/// Notion Thief — {2}{U}{B}
/// Creature — Human Rogue 3/1
///
/// > Flash
/// > If an opponent would draw a card except the first one they draw in each of
/// > their draw steps, instead that player skips that draw and you draw a card.
///
/// **The same event with a new subject**, and its own ruling is the only reason
/// to know that. Read as English it is Alms Collector's shape — a skip and a
/// draw joined by "and" — and §3.2d filed it as `Prevent` plus a rider until
/// the rulings pass. The ruling walks two Thieves, says each is *"applied to
/// the card draw only once"*, and concludes that in a two-player game *"it
/// really will be that player who draws a card"*. A rider's draw is a fresh
/// proposal with a fresh applied set, so two Thieves as riders would hand the
/// draw back and forth forever. As `Instead(DrawCards { n: 1, player: You })`
/// the draw keeps its lineage: each Thief applies once, and the draw comes home.
///
/// `n: 1` rather than a `DrawCard`, because the substitute for a draw is always
/// the instruction (CR 121.2a) — which also means the Thief's own draw is
/// `DrawCause::Effect` and a second Thief can take it.
/// # The rulings, and where each is tested
///
/// - *"If an opponent is instructed to draw a card then discard a card, and
///   [this] causes you to draw a card instead, that opponent still discards a
///   card. The same is true of any other actions that opponent is instructed to
///   do."* → tested on the second sentence: `Primitive::Discard` is
///   `NotImplemented` until RE-8, so the board is a draw-then-lose-life
///   resolution and the assertion is that the opponent still loses the life.
///   → `the_opponents_other_instructions_still_happen`
/// - *"If two or more players each control [one] ... that player chooses one ...
///   Then the player whose [effect] was chosen repeats this process among the
///   remaining ... Each effect can be applied to the card draw only once."* →
///   the three-player test, and every sentence falls out rather than being
///   coded. → `three_notion_thieves_pass_the_draw_once_each_in_the_rulings_order`
/// - *"[So] if each player in a two-player game controls [one] and one would
///   draw a card, it really will be that player who draws a card."* → test, and
///   the same mechanism observed from outside.
///   → `two_notion_thieves_hand_the_draw_across_the_table_and_back`
pub fn notion_thief() -> Arc<CardData> {
    CardDataBuilder::new("Notion Thief")
        .mana_cost(ManaCost::build(&[ManaType::Blue, ManaType::Black], 2))
        .color(Color::Blue)
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Rogue))
        .power_toughness(3, 1)
        .rules_text(
            "Flash\nIf an opponent would draw a card except the first one they draw in each of their draw steps, instead that player skips that draw and you draw a card.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::DrawCard { cause: Some(DrawCause::Effect) },
                AffectedSet::NO_OBJECTS,
                Rewrite::Instead(GameActionTemplate::DrawCards {
                    n: 1,
                    player: Some(PlayerRef::You),
                }),
            )
            .affecting_players(PlayerSet::Opponents),
        ))
        .build()
}

// ---------------------------------------------------------------------------
// RE-3 — life
// ---------------------------------------------------------------------------

/// Rhox Faithmender — {3}{W}
/// Creature — Rhino Monk 1/5
///
/// > Lifelink
/// > If you would gain life, you gain twice that much life instead.
///
/// **The first consumer in Phase RE that meets a live proposal without a
/// fixture.** Lifelink's contained `GainLife` has been proposed in every
/// measured game since RB, with nothing watching it; this card is on the
/// battlefield with lifelink of its own, so it doubles the life its own combat
/// damage gains.
///
/// # The rulings, and where each is tested
///
/// - *"If you control two Rhox Faithmenders, life you gain will be multiplied by
///   four. Three ... by eight, and so on."* → **the test this phase was built
///   around**, and it failed as an unexpected CR 616.1 *prompt* before it could
///   fail as a number (§11 item 58).
///   → `test_two_rhox_faithmenders_quadruple`, with
///   `three_rhox_faithmenders_multiply_by_eight` for the exponent and
///   `four_is_not_two_applications_of_one_faithmender` for CR 614.6's single
///   modified event.
/// - *"If an effect sets your life total to a specific number, and that number
///   is higher than your current life total, the effect will cause you to gain
///   life equal to the difference ... 3 life and 'becomes 10' ... will actually
///   become 17."* → **no facility**: CR 119.5's set-life-total is
///   `Primitive::SetLifeTotal`, which RE-6 builds. It is the same board as
///   Alhammarret's Archive's first ruling and Skullcrack's fourth, so one
///   primitive closes three (`codebase-state.md` item 123).
pub fn rhox_faithmender() -> Arc<CardData> {
    CardDataBuilder::new("Rhox Faithmender")
        .mana_cost(ManaCost::build(&[ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Rhino))
        .subtype(Subtype::Creature(CreatureType::Monk))
        .power_toughness(1, 5)
        .keyword_flag(KeywordFlag::Lifelink)
        .rules_text("Lifelink\nIf you would gain life, you gain twice that much life instead.")
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::GainLife,
                AffectedSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Tainted Remedy — {2}{B}
/// Enchantment
///
/// > If an opponent would gain life, that player loses that much life instead.
///
/// **The kind-changing substitution, and the first customer of
/// [`TemplateAmount::ReplacedAmount`].** "That much" is the gain's own number
/// read at the moment this applies, which is what makes its ordering ruling
/// arithmetic rather than a coin flip: beside Alhammarret's Archive the gaining
/// player picks double-then-lose-6, or lose-3-then-nothing.
///
/// Four-player: `PlayerSet::Opponents` is three opponents against one static
/// row, which is the shape CR 109.5 resolves per event rather than per
/// registration.
/// # The rulings, and where each is tested
///
/// - *"If more than one replacement effect tries to apply to a life gain event,
///   the player who would gain life chooses the order ... that player may choose
///   to have the 3 life become doubled to 6 life and then lose 6 life. The
///   player may also choose to apply [this] first, turning 'gain 3 life' into
///   'lose 3 life.' Alhammarret's Archive would then not apply."* → test, with
///   the ruling's own numbers, and the board `ordering_cannot_change_outcome`
///   must **not** suppress.
///   → `the_gaining_player_chooses_between_the_archive_and_tainted_remedy`
/// - *"Having more than one [of these] on the battlefield doesn't have any
///   noticeable effect on life gain. Once the effect of one applies, there is no
///   life gain for the others to apply to."* → two tests, because the engine
///   proves the order away and a suppressed choice can only be stated by making
///   it. → `a_second_tainted_remedy_has_no_gain_left_to_apply_to` and
///   `two_tainted_remedies_lose_the_same_three_whichever_applies`
pub fn tainted_remedy() -> Arc<CardData> {
    CardDataBuilder::new("Tainted Remedy")
        .mana_cost(ManaCost::build(&[ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Enchantment)
        .rules_text("If an opponent would gain life, that player loses that much life instead.")
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::GainLife,
                AffectedSet::NO_OBJECTS,
                Rewrite::Instead(GameActionTemplate::LoseLife {
                    amount: TemplateAmount::ReplacedAmount,
                }),
            )
            .affecting_players(PlayerSet::Opponents),
        ))
        .build()
}

/// Words of Worship — {2}{W}
/// Enchantment
///
/// > {1}: The next time you would draw a card this turn, you gain 5 life
/// > instead.
///
/// **A draw replaced by life — RE-2's pattern and RE-3's template on one
/// row**, and the first `Uses::Once` draw replacement in the crate. It is a
/// resolution's row rather than a static ability, so `Duration::UntilEndOfTurn`
/// is the card's "this turn" and `Uses::Once` is its "the next time"; neither
/// is derived, for CR 608.2c's reason.
///
/// [`EventPattern::DrawCard`] with `cause: None` — "the next time you would
/// draw a card" excepts nothing, so the draw step's own draw is a candidate.
///
/// **Leyline of Punishment's
/// ruling about this card is the CR 101.2 ordering test**: under a "players
/// can't gain life", the substituted gain is proposed, refused, and the draw
/// has been replaced with nothing.
/// # The rulings, and where each is tested
///
/// - *"If multiple Words have been used prior to drawing a card, then you can
///   choose which one to apply (and use up) each time you draw a card."* → test.
///   Two rows from one source, one prompt, and the other row still there for the
///   next draw — which is what "(and use up)" means.
///   → `two_words_rows_are_a_choice_and_each_draw_uses_one_up`
/// - **Leyline of Punishment's** ruling about this card, since the Leyline is
///   not registered: *"effects that replace an event with gaining life (like
///   Words of Worship's effect does) will end up replacing the event with
///   nothing."* → the CR 101.2 ordering test, on Skullcrack.
///   → `words_of_worship_under_skullcrack_replaces_the_draw_with_nothing`
pub fn words_of_worship() -> Arc<CardData> {
    CardDataBuilder::new("Words of Worship")
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text("{1}: The next time you would draw a card this turn, you gain 5 life instead.")
        .ability(one_shot(
            AbilityType::Activated,
            vec![Cost::Mana(ManaCost::build(&[], 1))],
            Effect::Atom(
                Primitive::CreateReplacement(
                    Box::new(
                        ReplacementDef::new(
                            EventPattern::DrawCard { cause: None },
                            AffectedSet::NO_OBJECTS,
                            Rewrite::Instead(GameActionTemplate::GainLife {
                                amount: TemplateAmount::Fixed(5),
                            }),
                        )
                        .affecting_players(PlayerSet::You)
                        .once(),
                    ),
                    Duration::UntilEndOfTurn,
                    PatternFill::Authored,
                ),
                EffectRecipient::Controller,
            ),
        ))
        .build()
}

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
/// the life total before any replacement is asked, and a payment's cause is
/// [`LifeLossCause::Cost`], which this pattern does not match. The card agrees
/// — "damage that would reduce" — and so does its first ruling, *"this effect
/// does not apply to effects which reduce your life without doing damage."*
///
/// # The rulings, and where each is tested
///
/// - *"This effect does not apply to effects which reduce your life without
///   doing damage."* → test: a `Primitive::LoseLife` is `LifeLossCause::Effect`,
///   which the pattern does not match, and the player goes to -7.
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
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::LoseLife { cause: Some(LifeLossCausePattern::Damage) },
                AffectedSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::LifeFloor(1)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Alhammarret's Archive — {5}
/// Legendary Artifact
///
/// > If you would gain life, you gain twice that much life instead.
/// > If you would draw a card except the first one you draw in each of your
/// > draw steps, draw two cards instead.
///
/// **Two statics on one permanent, one from each of the two RE phases** —
/// Rhox Faithmender's gain doubler and Teferi's Ageless Insight's draw doubler,
/// written as the same two defs because they *are* the same two defs. Gisela's
/// shape, and the reason RE-2 → RE-3 is a hard order in §9.
///
/// # The rulings, and where each is tested
///
/// - *"If an effect would set your life total to a specific number that's higher
///   ... your life total will actually become 17."* → RE-6's, as Rhox
///   Faithmender's twin.
/// - *"If two or more replacement effects would apply to a card-drawing event,
///   the player drawing the card chooses the order."* → already tested, in
///   RE-2: `a_draw_doubler_beside_a_notion_thief_is_a_real_choice`.
/// - *"Because [it] is legendary ... if that happens, life gained by that player
///   will be multiplied by four."* → the legend rule makes this Rhox
///   Faithmender's board. This card's own board is the one that shows the two
///   halves coexist.
///   → `alhammarrets_archive_doubles_a_gain_and_a_draw_from_one_permanent`
/// - *"Similarly, the effects of the last abilities of multiple Archives are
///   cumulative."* → already tested, in RE-2:
///   `test_two_thought_reflections_draw_four_not_infinity`.
pub fn alhammarrets_archive() -> Arc<CardData> {
    CardDataBuilder::new("Alhammarret's Archive")
        .mana_cost(ManaCost::build(&[], 5))
        .card_type(CardType::Artifact)
        .supertype(Supertype::Legendary)
        .rules_text(
            "If you would gain life, you gain twice that much life instead.\nIf you would draw a card except the first one you draw in each of your draw steps, draw two cards instead.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::GainLife,
                AffectedSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .ability(static_replacement(draw_two_instead(Some(DrawCause::Effect))))
        .build()
}

/// Skullcrack — {1}{R}
/// Instant
///
/// > Players can't gain life this turn. Damage can't be prevented this turn.
/// > Skullcrack deals 3 damage to target player or planeswalker.
///
/// **The card that lands RD-4's restriction row in a game** (§11 item 26). Its
/// second sentence is the `ApplyReplacement { Prevention }` row RD-4 could only
/// build as a fixture, because every printed carrier of it needed a facility
/// the engine lacked; its first is the `Event { GainLife }` row RE-3 gave
/// `Restriction::Event` the player set for (item 45).
///
/// **Three atoms in text order, and the order is the card's**: CR 608.2c reads
/// a spell's instructions in the order printed, and both restrictions are in
/// place before the damage is dealt — which is what makes a lifelinker's damage
/// gain nothing this turn.
///
/// **Leyline of Punishment is deliberately not registered.** It is the static
/// form of the same two rows — an `Effect::Restriction` on a permanent, which
/// RS-1's sweep already reads — and the RD-4 fixture extended with the life arm
/// is what proves that form. What keeps it out is its first line: "if this card
/// is in your opening hand, you may begin the game with it on the battlefield"
/// is §3.3 source 2's zone-reaching static, which would be dead text under a
/// real card name. Recorded so the omission reads as the rule and not as an
/// oversight.
/// # The rulings, and where each is tested
///
/// - *"[This] targets only the player or planeswalker. If that player or
///   planeswalker is an illegal target when [it] tries to resolve, it won't
///   resolve and none of its effects will happen."* → CR 608.2b's fizzle, which
///   is the stack's and predates this phase; the card adds no new claim to it.
///   Recorded rather than re-tested.
/// - *"Spells and abilities that would cause a player to gain life or that would
///   prevent damage still resolve, but the life-gain and damage-prevention parts
///   have no effect."* → test, and "still resolve" is the half that could have
///   been got wrong: a refused proposal is not an error.
///   → `a_life_gain_spell_still_resolves_under_skullcrack_and_gains_nothing`,
///   with `skullcrack_stops_life_gain_for_everyone_including_its_controller` for
///   `PlayerSet::Everyone`.
/// - *"Effects that would replace gaining life with another effect won't apply
///   because it's impossible for players to gain life."* → CR 119.7's own last
///   clause, and `ATOM-119.7-004`.
///   → `under_skullcrack_a_gain_replacement_has_no_event_to_replace`
/// - *"If an effect says to set a player's life total to a certain number and
///   that number is higher than the player's current life total, that part of
///   the effect won't do anything."* → RE-6's, as above.
pub fn skullcrack() -> Arc<CardData> {
    CardDataBuilder::new("Skullcrack")
        .mana_cost(ManaCost::build(&[ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .rules_text(
            "Players can't gain life this turn. Damage can't be prevented this turn. Skullcrack deals 3 damage to target player or planeswalker.",
        )
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Sequence(vec![
                // "Players can't gain life this turn." No object and every
                // player, so the row is complete as authored and the
                // resolution's target — the player it then damages — is not
                // what it is about.
                Effect::Atom(
                    Primitive::Restrict(
                        RestrictionDef::new(Restriction::Event {
                            pattern: EventPattern::GainLife,
                            affected_objects: AffectedSet::NO_OBJECTS,
                            affected_players: PlayerSet::Everyone,
                            by: None,
                        }),
                        Duration::UntilEndOfTurn,
                    ),
                    EffectRecipient::Controller,
                ),
                // "Damage can't be prevented this turn." CR 615.12's row, whose
                // two halves are a union: every object and every player.
                Effect::Atom(
                    Primitive::Restrict(
                        RestrictionDef::new(Restriction::ApplyReplacement {
                            kind: ReplacementKindFilter::Prevention,
                            to_objects: AffectedSet::Filter { filter: ObjectFilter::All },
                            to_players: PlayerSet::Everyone,
                        }),
                        Duration::UntilEndOfTurn,
                    ),
                    EffectRecipient::Controller,
                ),
                Effect::Atom(
                    Primitive::DealDamage { amount: AmountExpr::Fixed(3), unpreventable: false },
                    EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
                ),
            ]),
        ))
        .build()
}

// ---------------------------------------------------------------------------
// RE-6 — the game's end
// ---------------------------------------------------------------------------

/// Laboratory Maniac — {2}{U}
/// Creature — Human Wizard 2/2
///
/// > If you would draw a card while your library has no cards in it, you win
/// > the game instead.
///
/// The kind-changing substitution from a draw to the game's end, and the first
/// replacement effect in the crate with an "as long as": `Condition::LibraryEmpty`
/// on the ability, asked by the gather at each proposal (CR 604.2, 614.4). The
/// draw reaches the pipeline with nothing to draw because CR 121.6a says a
/// draw replacement applies "even if no cards could be drawn", which is the
/// whole of why `draw_card` flags rather than refuses — and with the draw
/// replaced, no flag is set and no loss is proposed. **The pooled card of the
/// PR.**
/// # The rulings, and where each is tested
///
/// - *"If for some reason you can't win the game (because your opponent has
///   cast Angel's Grace this turn, for example), you won't lose for having
///   tried to draw a card from a library with no cards in it. The draw was
///   still replaced."* → an opponent's Platinum Angel is the printed "can't
///   win" the crate has: the draw is replaced, the substituted win is refused
///   by CR 101.2 at the next iteration, no card is drawn and no flag is set.
///   → `laboratory_maniac_under_an_opponents_platinum_angel_neither_wins_nor_loses`
/// - *"If two or more players each control a Laboratory Maniac and each player
///   is instructed to draw a number of cards, first the player whose turn it is
///   draws that many cards. If this causes that player to win the game instead,
///   the game is immediately over."* → **not expressible**: an each-player draw
///   instruction needs a recipient `EffectRecipient` lacks and CR 121.2c's
///   APNAP ordering over an effect's recipients (`codebase-state.md` item 122,
///   RE-2's note). What *is* tested is the half the engine has — the game is
///   over at the first win and nothing after it performs —
///   → `a_win_ends_the_game_immediately_and_the_rest_of_the_batch_still_performs`.
pub fn laboratory_maniac() -> Arc<CardData> {
    CardDataBuilder::new("Laboratory Maniac")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 2))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .power_toughness(2, 2)
        .rules_text(
            "If you would draw a card while your library has no cards in it, you win the game instead.",
        )
        .ability(static_conditional_replacement(
            Condition::LibraryEmpty,
            // Any individual draw, whatever instructed it: the draw step's,
            // a cantrip's, the seventh of Stunning Reversal's. The instruction
            // (`DrawCards`) is not what this watches — CR 121.2 performs the
            // draws one at a time and the library empties between them.
            ReplacementDef::new(
                EventPattern::DrawCard { cause: None },
                AffectedSet::NO_OBJECTS,
                Rewrite::Instead(GameActionTemplate::PlayerWins),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Exquisite Archangel — {5}{W}{W}
/// Creature — Angel 5/5
///
/// > Flying
/// > If you would lose the game, instead exile this creature and your life
/// > total becomes equal to your starting life total.
///
/// `Prevent` with a rider, because "instead exile this creature and ..." is
/// two effects on two different subjects — the Archangel and you — and a
/// substitution produces one event about one (§3.2d). The rider's exile names
/// the Archangel through `EffectRecipient::Implicit`, which is the effect's
/// own source; its life total is `Primitive::SetLifeTotal` over
/// `AmountExpr::StartingLifeTotal`, so it is 40 in Commander and a 24-life
/// *gain* from -4 that Rhox Faithmender doubles (CR 119.5).
/// # The rulings, and where each is tested
///
/// - *"If Exquisite Archangel is dealt lethal damage at the same time that
///   you're dealt damage that brings your life total to 0 or less, its effect
///   applies and your life total becomes equal to your starting life total.
///   You choose whether Exquisite Archangel is moved to exile or to your
///   graveyard."* → the first half is a test: the loss and the death are two
///   members of one CR 704.3 batch decided against one board, so the
///   Archangel replaces the loss while it is still there. **The second half
///   is not offered**: riders resolve after the batch performs (CR 615.5,
///   §4.1a), so the death has happened when the rider's exile looks for the
///   creature, and CR 400.7 makes the card in the graveyard a new object the
///   exile does not find. The engine takes the graveyard outcome without the
///   choice — `codebase-state.md` item 125.
///   → `exquisite_archangel_replaces_the_loss_while_dying_in_the_same_check`
/// - *"If an effect says that you can't lose the game, Exquisite Archangel's
///   effect doesn't apply."* → CR 101.2's order: Platinum Angel's row refuses
///   the proposal ahead of the gather, and CR 614.17c leaves nothing for a
///   non-self-replacement to apply to.
///   → `under_platinum_angel_exquisite_archangel_does_not_apply`
/// - *"If you control two Exquisite Archangels, you choose which one's effect
///   applies. The other's effect won't be applicable after that until the next
///   time you would lose the game."* → a CR 616.1 prompt between two printed
///   statics — a `Prevent` with a rider is no suppression shape — and the loss
///   is gone after one applies.
///   → `two_exquisite_archangels_you_choose_which_applies`
/// - *"Exquisite Archangel's effect applies any time you would lose the game,
///   even if you're not losing due to your life total being 0 or less. If you
///   would have lost the game because you tried to draw from an empty library,
///   you won't lose again until you try to draw again and still can't do so."*
///   → the first sentence is `EventPattern::PlayerLoses` having no reason,
///   tested on a poison loss; the second is CR 704.5b's window closing at the
///   check that read it (item 112).
///   → `exquisite_archangel_applies_to_a_poison_loss_and_then_the_poison_still_loses`,
///   `a_replaced_empty_library_loss_is_not_proposed_again_until_the_next_draw`
/// - *"Exquisite Archangel's effect does nothing if you concede the game. A
///   player who concedes leaves the game."* → recorded, no harness offers
///   concession (CR 104.3a is a *leave* that then loses, not a proposed loss).
/// - *"For your life total to become your starting life total (normally 20),
///   you gain or lose the appropriate amount of life. For example, if your life
///   total is -4 when Exquisite Archangel's ability applies, it will cause you
///   to gain 24 life; alternatively, if your life total is 40 when it applies,
///   it will cause you to lose 20 life. Other cards that interact with life gain
///   or life loss will interact with this effect accordingly."* → both
///   directions, and the gain doubled to 48 by Rhox Faithmender.
///   → `exquisite_archangel_from_minus_four_is_a_gain_of_twenty_four_that_rhox_faithmender_doubles`,
///   `exquisite_archangel_applies_to_a_poison_loss_and_then_the_poison_still_loses`
///   (life 40 → 20 is the loss leg)
/// - *"If an effect states that an opponent wins the game, Exquisite Archangel's
///   ability doesn't apply."* → an opponent's win is not a loss event; the game
///   ends with the Archangel untouched.
///   → `an_opponents_win_is_not_a_loss_exquisite_archangel_can_replace`
pub fn exquisite_archangel() -> Arc<CardData> {
    CardDataBuilder::new("Exquisite Archangel")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 5))
        .color(Color::White)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Angel))
        .power_toughness(5, 5)
        .keyword_flag(KeywordFlag::Flying)
        .rules_text(
            "Flying\nIf you would lose the game, instead exile this creature and your life total becomes equal to your starting life total.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::PlayerLoses,
                AffectedSet::NO_OBJECTS,
                Rewrite::Prevent,
            )
            .affecting_players(PlayerSet::You)
            .with_then(Effect::Sequence(vec![
                Effect::Atom(Primitive::Exile, EffectRecipient::Implicit),
                Effect::Atom(
                    Primitive::SetLifeTotal(AmountExpr::StartingLifeTotal),
                    EffectRecipient::Controller,
                ),
            ])),
        ))
        .build()
}

/// Stunning Reversal — {3}{B}
/// Instant
///
/// > The next time you would lose the game this turn, instead draw seven cards
/// > and your life total becomes 1.
/// > Exile Stunning Reversal.
///
/// A `Primitive::CreateReplacement` row — `PlayerLoses`, `You`, `Uses::Once`,
/// `UntilEndOfTurn`, `Prevent` with the two-atom rider — and then the spell
/// exiling itself as CR 608.2c's second instruction, which is not part of the
/// row: the row outlives the card, and CR 608.2m lets the spell finish
/// resolving from exile.
/// # The rulings, and where each is tested
///
/// - *"If each player would lose the game at the same time, but Stunning
///   Reversal's effect applies to you losing the game, you win the game as
///   soon as everyone else has lost the game. This is true even if you'd lose
///   the game immediately afterwards, perhaps because you don't have seven
///   cards in your library to draw or because you couldn't gain life to raise
///   your life total to 1."* → the four-player board: four `PlayerLoses`
///   members, one replaced, and CR 104.2a settled by the *batch* — before the
///   rider draws, and never overwritten by the loss that follows.
///   → `stunning_reversal_when_everyone_would_lose_at_once_its_controller_wins`
/// - *"While the replacement effect it creates lasts until end of turn (or
///   until the event it replaces), Stunning Reversal is exiled as it
///   resolves."* → the card is in exile and not in a graveyard after
///   resolution, and the row is still registered.
///   → `stunning_reversal_is_exiled_as_it_resolves_and_its_row_survives_it`
/// - *"If an effect says you can't lose the game, Stunning Reversal's effect
///   can't apply."* → Platinum Angel's row refuses the proposal, and the
///   unspent row is still there for a loss the Angel no longer refuses.
///   → `under_platinum_angel_stunning_reversal_neither_applies_nor_is_spent`
/// - *"If an effect says that an opponent wins the game, Stunning Reversal's
///   effect doesn't apply."* → the same fact as Exquisite Archangel's seventh
///   ruling: an opponent's win is not a loss event.
///   → `an_opponents_win_is_not_a_loss_exquisite_archangel_can_replace`
/// - *"Stunning Reversal's effect does nothing if you concede the game. A
///   player who concedes leaves the game."* → recorded, no harness.
/// - *"Stunning Reversal's effect applies any time you would lose the game,
///   even if you're not losing due to your life total being 0 or less."* →
///   `EventPattern::PlayerLoses` has no reason; tested on the Archangel's
///   poison board, which is the same pattern on the same event.
///   → `exquisite_archangel_applies_to_a_poison_loss_and_then_the_poison_still_loses`
/// - *"For your life total to become 1, you gain or lose the appropriate
///   amount of life. For example, if your life total is 4 when Stunning
///   Reversal's effect applies, it will cause you to lose 3 life;
///   alternatively, if your life total is -5 when it applies, it will cause
///   you to gain 6 life."* → the gain leg is the natural board (lethal damage
///   leaves you below 0); the loss leg is `ATOM-119.5-001`'s fixture.
///   → `stunning_reversal_from_minus_five_is_a_gain_of_six`,
///   `setting_a_life_total_lower_is_a_loss_of_the_difference`
/// - *"If you have fewer than seven cards in your library, you'll lose the
///   game immediately after applying Stunning Reversal's replacement
///   effect."* → the rider's draws flag CR 704.5b, the check repeats because
///   the game changed, and the spent row cannot see the second proposal.
///   → `stunning_reversal_with_a_short_library_loses_immediately_after`
pub fn stunning_reversal() -> Arc<CardData> {
    CardDataBuilder::new("Stunning Reversal")
        .mana_cost(ManaCost::build(&[ManaType::Black], 3))
        .color(Color::Black)
        .card_type(CardType::Instant)
        .rules_text(
            "The next time you would lose the game this turn, instead draw seven cards and your life total becomes 1.\nExile Stunning Reversal.",
        )
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::CreateReplacement(
                        Box::new(
                            ReplacementDef::new(
                                EventPattern::PlayerLoses,
                                AffectedSet::NO_OBJECTS,
                                Rewrite::Prevent,
                            )
                            .affecting_players(PlayerSet::You)
                            .once()
                            .with_then(Effect::Sequence(vec![
                                Effect::Atom(
                                    Primitive::DrawCards(AmountExpr::Fixed(7)),
                                    EffectRecipient::Controller,
                                ),
                                Effect::Atom(
                                    Primitive::SetLifeTotal(AmountExpr::Fixed(1)),
                                    EffectRecipient::Controller,
                                ),
                            ])),
                        ),
                        Duration::UntilEndOfTurn,
                        PatternFill::Authored,
                    ),
                    EffectRecipient::Controller,
                ),
                // CR 608.2c's second instruction, on the spell itself.
                Effect::Atom(Primitive::Exile, EffectRecipient::Implicit),
            ]),
        ))
        .build()
}

/// Platinum Angel — {7}
/// Artifact Creature — Angel 4/4
///
/// > Flying
/// > You can't lose the game and your opponents can't win the game.
///
/// Two `Restriction::Event` rows over the two new patterns, refused ahead of
/// the pipeline (CR 101.2, 614.17): the state-based check proposes your loss
/// at every check and `is_prohibited` drops it every time, which is the ruling
/// — "you keep playing". Its own controller may still win: `PlayerSet::Opponents`
/// is the second row's whole scope. **Registered and not pooled**, and the
/// module doc says why.
/// # The rulings, and where each is tested
///
/// - *"No game effect can cause you to lose the game or cause any opponent to
///   win the game while you control Platinum Angel. It doesn't matter whether
///   you have 0 or less life, you're forced to draw a card while your library
///   is empty, you have ten or more poison counters, you're dealt combat damage
///   by Phage the Untouchable, your opponent has Mortal Combat with twenty or
///   more creature cards in their graveyard, or so on. You keep playing."* →
///   all three state-based reasons at once, refused at every check with the
///   game continuing, and the loss performed at the first check after the
///   Angel leaves; and an opponent's Laboratory Maniac win refused.
///   → `platinum_angel_refuses_every_state_based_loss_and_the_game_goes_on`,
///   `platinum_angel_leaving_the_battlefield_lets_the_next_check_lose`,
///   `laboratory_maniac_under_an_opponents_platinum_angel_neither_wins_nor_loses`
/// - *"Other circumstances can still cause you to lose the game, however. You
///   will lose a game if you concede, if you're penalized with a Game Loss or
///   a Match Loss during a sanctioned tournament ... or if your Magic Online
///   game clock runs out of time."* → **not expressible**: none of the three is
///   a game event the engine proposes; concession is CR 104.3a's leave, which
///   no harness offers.
/// - *"Effects that say the game is a draw, such as the Legends card Divine
///   Intervention, are not affected by Platinum Angel. They'll still work."* →
///   **not expressible**: CR 104.4c has no producer (`Primitive` has no
///   "the game is a draw"), recorded in `replacement-architecture.md` §9's
///   "Out of RE".
/// - *"You can concede a game while Platinum Angel on the battlefield. A
///   concession causes you to leave the game, which then causes you to lose
///   the game."* → recorded with the first ruling's concession half.
pub fn platinum_angel() -> Arc<CardData> {
    CardDataBuilder::new("Platinum Angel")
        .mana_cost(ManaCost::build(&[], 7))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Angel))
        .power_toughness(4, 4)
        .keyword_flag(KeywordFlag::Flying)
        .rules_text("Flying\nYou can't lose the game and your opponents can't win the game.")
        .ability(static_restriction(Restriction::Event {
            pattern: EventPattern::PlayerLoses,
            affected_objects: AffectedSet::NO_OBJECTS,
            affected_players: PlayerSet::You,
            by: None,
        }))
        .ability(static_restriction(Restriction::Event {
            pattern: EventPattern::PlayerWins,
            affected_objects: AffectedSet::NO_OBJECTS,
            affected_players: PlayerSet::Opponents,
            by: None,
        }))
        .build()
}
