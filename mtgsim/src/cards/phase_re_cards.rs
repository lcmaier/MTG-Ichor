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
//! **Laboratory Maniac is the first replacement effect with a condition on its
//! static ability** — "while your library has no cards in it", the same
//! `Effect::Conditional` shape CR 604.2 gives Kird Ape's "as long as you
//! control a Forest" — and the condition is evaluated by the gather at each
//! proposal rather than by the rewrite at application: CR 604.2 makes a
//! conditional static's effect exist while its condition holds, and CR 614.4
//! asks whether the effect exists *before* the event. CR 121.6a is what makes
//! the board reachable at all — a draw with nothing to draw still reaches the
//! pipeline — and the condition is true at exactly that gather.
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
//!
//! # RE-4 — tokens (CR 614.16's token half, 111.5, 616.1g)
//!
//! **Four printed cards on two axes: what makes a plural creation, and what
//! the creation meets.** CR 111's "create three tokens" is one event, and
//! `GameAction::CreateTokens` is the first proposal in the crate whose
//! performer proposes a *batch* — one entry per token, decided together
//! against the board none of them has entered (CR 614.12; `codebase-state.md`
//! item 46).
//!
//! | Card | Is | Does |
//! |---|---|---|
//! | [`raise_the_alarm`] | an instant | two Soldiers — the first plural creation, so the first plural entry batch |
//! | [`hordeling_outburst`] | a sorcery | three Goblins |
//! | [`parallel_lives`] | a static, `You` | `Amount(Multiplier(2))` on the creation — the **outer** event |
//! | [`hallowed_moonlight`] | a resolution, until end of turn | `Instead(ZoneChangeTo { Exile })` on each entry — the **contained** one |
//! | [`divine_visitation`] | a static, `You`, *creature* tokens | `Instead(CreateTokens { Angel, ReplacedAmount, Replace })` — the kind-changing substitution over a creation |
//! | [`bard_king_of_dale`] | a static, `You`, twice | Alhammarret's Archive's draw half beside Parallel Lives' token half — both halves already built, so the card cost nothing but its registration |
//!
//! Divine Visitation and Bard came in at the review (`plans/handoffs/re-4-review.md`,
//! theme B): a template and a kind field whose customers were in print and
//! whose type this phase had open.
//!
//! **Parallel Lives is applied once, at the creation; Hallowed Moonlight once
//! per token.** That is CR 616.1g — "the second effect can't be chosen until
//! after the first effect has been chosen" — read as containment
//! (`replacement-architecture.md` §3.2d): the creation's CR 616.1 loop runs
//! to completion, then each entry's runs with a fresh applied set, so a
//! doubler applied to the creation is not offered again at any entry, and an
//! entry replacement is offered at every one. Beside Master Biomancer the
//! Moonlight is a real choice per token, and the test counts the prompts.
//!
//! **A token exiled instead was created in exile.** A card's substituted entry
//! is a zone change from where the card is; a token was nowhere, and its
//! ruling says where it goes — "put into exile instead and then ceases to
//! exist" — so the substitute is `GameAction::CreateTokenIn` and the log
//! holds `TokenCreated { Exile }` and CR 704.5d's `TokenCeasedToExist`, with
//! no `ZoneChange { from: Battlefield }` for a leaves-the-battlefield trigger
//! to misread (`codebase-state.md` item 52).
//!
//! # What a random deck can draw
//!
//! Parallel Lives and Raise the Alarm are the pooled pair. The Alarm is the
//! producer: a `{1}{W}` instant any white deck casts, and the first plural
//! entry batch a measured game builds — the engine path item 46 wanted
//! measured. Parallel Lives is the first `CreateTokens` watcher, and the
//! first board on which the creation's loop and the entries' loops are both
//! asked in one resolution. Kalitas's rider already makes single tokens in
//! `stress`, so the registered-but-unpooled arm moves by one gather per
//! Zombie and nothing else.
//!
//! Hordeling Outburst stays out: the same path as the Alarm at `{1}{R}{R}`,
//! and a second copy of one path buys a slower fuzz run rather than a wider
//! one. Hallowed Moonlight stays out too: its row is RC-4b's substituted
//! entry, which Containment Priest's shape already measures, and its one new
//! line — a token created in exile — needs a token creation under the row in
//! the same turn, which two pooled cards and a random agent reach rarely;
//! that reachability is a `--require` row rather than a slot.
//!
//! # RE-5 — counters, on permanents and players (CR 614.16's counter half, 122.1, 122.6, 122.6a)
//!
//! **Six printed cards on two axes: which subject the effect is around — a
//! permanent, a player, or either — and what it does to the count.** CR 122.6
//! makes an entry that gives a permanent counters the same "put on" event, so
//! every watcher here meets two doors, an `AddCounters` proposal and an
//! `EnterBattlefield` whose mods carry a kind, and every one of them has a
//! ruling saying it "affects permanents that enter with counters".
//!
//! | Card | Around | Does |
//! |---|---|---|
//! | [`doubling_season`] | permanents you control (and tokens, RE-4's half) | `Amount(Multiplier(2))`, every kind |
//! | [`hardened_scales`] | creatures you control, +1/+1 only | `Amount(Plus(1))` — RD-3's arm, second kind |
//! | [`vorinclex_monstrous_raider`] | every permanent and player, *by who puts them on* | `Multiplier(2)` if you, `Halve(Down)` if an opponent |
//! | [`winding_constrictor`] | artifacts and creatures you control; and you | `Plus(1)` of each kind, twice |
//! | [`live_fast`] | you | the producer — `Primitive::GetCounters`, two energy |
//! | [`primal_vigor`] | every creature, +1/+1; every token | `Multiplier(2)`, `Everyone` |
//!
//! **Who puts the counters on rides on the event.** `AddCounters::by` is the
//! player the effect names, else the resolving effect's controller; at the
//! entry door it is each row's named putter, else CR 122.6a's default — the
//! controller the permanent enters under, settled at CR 616.1b before
//! anything asks. Vorinclex is the only printed reader; Doubling Season's
//! counter half reads the *permanent's* controller and nothing about the
//! putter, which its text says and `ATOM-122.6a-001`'s example gets wrong.
//! No printed effect names a putter at an entry (Scryfall, 2026-09-14) and
//! the field is built anyway: the CR states it, and Bold Plagiarist names
//! one on a proposal (`engineering-practices.md` §4).
//!
//! **A multiplier and a plus do not commute, and the prompt is real.**
//! Doubling Season beside Hardened Scales is 1 → 2 → 3 or 1 → 2 → 4, and
//! Scales' own ruling says the creature's controller chooses "no matter who
//! controls the sources". Two Seasons are a bucket of multipliers and ask
//! nothing; two Scales are additive and ask, though their outcome is one —
//! `backlog.md` §2.29's next row, recorded there and not built here.
//!
//! # What a random deck can draw
//!
//! Hardened Scales is the pooled card: `{G}`, so every green deck casts it on
//! turn one, and it opens the sweep on every `AddCounters` (Battlegrowth is
//! pooled) and every counter-bearing entry — Chainbreaker's, Master
//! Biomancer's grants, Loyalty Probe's in `stress` — for as long as it is on
//! the battlefield. Doubling Season is registered and stays out: five mana,
//! and its token half would double the pool's Soldiers, a gameplay change
//! the A/B should not carry with the engine change. Vorinclex is six mana and
//! legendary, Winding Constrictor two colors, Primal Vigor the Season with
//! `Everyone`, and Live Fast a cantrip whose energy nothing pooled reads:
//! each is the same path Scales opens at one mana, or a path with no second
//! card to meet.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::state::game_state::{PhaseType, StepType};
use crate::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    ObjectSet, AmountExpr, Condition, CounterType, Duration, Effect, EffectRecipient, ObjectFilter,
    PatternFill, PlayerRef, PlayerSet, Primitive, SelectionFilter, TargetCount, TokenDef,
};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::keywords::KeywordFlag;
use crate::types::replacement::{
    AmountRewrite, EventPattern, GameActionTemplate, LifeLossCausePattern, ReplacementDef,
    Rewrite, Rounding, TemplateAmount, TokenKind, TokenSubstitution,
};
use crate::types::restriction::{ReplacementKindFilter, Restriction, RestrictionDef};
use crate::types::zones::{DrawCause, Zone, ZoneChangeCause};

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
    ReplacementDef::new(pattern, ObjectSet::NO_OBJECTS, Rewrite::Prevent)
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
        ObjectSet::NO_OBJECTS,
        Rewrite::Instead(GameActionTemplate::DrawCards {
            n: TemplateAmount::Fixed(2),
            player: None,
        }),
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
/// **Flash is not modeled** (`codebase-state.md`'s timing item), and it costs
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
                ObjectSet::NO_OBJECTS,
                // "That player draws a card": the same instruction with `n`
                // rewritten to 1, so it keeps this effect's applied set and
                // whatever doubles it afterwards cannot hand it back.
                Rewrite::Instead(GameActionTemplate::DrawCards {
                    n: TemplateAmount::Fixed(1),
                    player: None,
                }),
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
                ObjectSet::NO_OBJECTS,
                Rewrite::Instead(GameActionTemplate::DrawCards {
                    n: TemplateAmount::Fixed(1),
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
                ObjectSet::NO_OBJECTS,
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
                ObjectSet::NO_OBJECTS,
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
                            ObjectSet::NO_OBJECTS,
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
                ObjectSet::NO_OBJECTS,
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
                ObjectSet::NO_OBJECTS,
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
                            affected_objects: ObjectSet::NO_OBJECTS,
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
                            to_objects: ObjectSet::Filter { filter: ObjectFilter::All },
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
/// replacement effect in the crate whose static ability carries a condition —
/// the card's "while", which is CR 604.2's "as long as" shape:
/// `Condition::LibraryEmpty` on the ability, asked by the gather at each
/// proposal (CR 604.2, 614.4). The
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
                ObjectSet::NO_OBJECTS,
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
                ObjectSet::NO_OBJECTS,
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
                                ObjectSet::NO_OBJECTS,
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
            affected_objects: ObjectSet::NO_OBJECTS,
            affected_players: PlayerSet::You,
            by: None,
        }))
        .ability(static_restriction(Restriction::Event {
            pattern: EventPattern::PlayerWins,
            affected_objects: ObjectSet::NO_OBJECTS,
            affected_players: PlayerSet::Opponents,
            by: None,
        }))
        .build()
}

// ---------------------------------------------------------------------------
// RE-4 — tokens
// ---------------------------------------------------------------------------

/// A token that carries only what the effect said — CR 111.3's "a token
/// doesn't have any characteristics not defined by the spell or ability that
/// created it" — with CR 111.4's default name.
fn vanilla_token(
    color: Color,
    creature_type: CreatureType,
    power: i32,
    toughness: i32,
) -> TokenDef {
    TokenDef {
        name: None,
        colors: vec![color],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(creature_type)],
        supertypes: Vec::new(),
        power: Some(power),
        toughness: Some(toughness),
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
    }
}

/// "a 1/1 white Soldier creature token" — named "Soldier Token" (CR 111.4).
pub fn soldier_token() -> TokenDef {
    vanilla_token(Color::White, CreatureType::Soldier, 1, 1)
}

/// "a 1/1 red Goblin creature token" — named "Goblin Token" (CR 111.4).
pub fn goblin_token() -> TokenDef {
    vanilla_token(Color::Red, CreatureType::Goblin, 1, 1)
}

/// Raise the Alarm — {1}{W}
/// Instant
///
/// > Create two 1/1 white Soldier creature tokens.
///
/// **The first plural creation in the crate**, and so the first
/// `GameAction::CreateTokens` whose performer proposes a batch of more than
/// one entry — `codebase-state.md` item 46's producer. One `Primitive`, one
/// proposal carrying the def twice, two entries decided against the board
/// before either Soldier entered (CR 614.12).
///
/// Scryfall lists no rulings (2026-09-13). The boards it is on are the
/// rules' rather than the card's: CR 614.16 under Parallel Lives, CR 616.1g
/// beside Hallowed Moonlight, CR 614.12 under Master Biomancer, CR 111.5
/// under a "can't enter".
///
/// **Pooled**, with Parallel Lives — the module doc says why.
pub fn raise_the_alarm() -> Arc<CardData> {
    CardDataBuilder::new("Raise the Alarm")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text("Create two 1/1 white Soldier creature tokens.")
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Atom(
                Primitive::CreateToken(soldier_token(), AmountExpr::Fixed(2)),
                EffectRecipient::Controller,
            ),
        ))
        .build()
}

/// Hordeling Outburst — {1}{R}{R}
/// Sorcery
///
/// > Create three 1/1 red Goblin creature tokens.
///
/// Raise the Alarm's shape one wider, and a sorcery: the second plural
/// creation, so that "the batch is the creation's order" (CR 613.7m's
/// decision point, not asked — `replacement-architecture.md` §9, RE
/// decision 3) is asserted on more than a pair.
///
/// Scryfall lists no rulings (2026-09-13). **Registered and not pooled** —
/// the same engine path as the Alarm at `{1}{R}{R}`.
pub fn hordeling_outburst() -> Arc<CardData> {
    CardDataBuilder::new("Hordeling Outburst")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 1))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text("Create three 1/1 red Goblin creature tokens.")
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Atom(
                Primitive::CreateToken(goblin_token(), AmountExpr::Fixed(3)),
                EffectRecipient::Controller,
            ),
        ))
        .build()
}

/// Parallel Lives — {3}{G}
/// Enchantment
///
/// > If an effect would create one or more tokens under your control, it
/// > creates twice that many of those tokens instead.
///
/// **CR 614.16's token half, and the first watcher of the outer event.** The
/// subject of a `CreateTokens` is the player the tokens are created under, so
/// "under your control" is [`PlayerSet::You`] and the object set is empty —
/// Rhox Faithmender's shape over a different event. The rewrite is the same
/// arm too, and over a `Vec` a multiplier repeats each def in place, which is
/// what "twice that many of those tokens" says.
///
/// Doubling Season's first ability is this card's text word for word; the
/// Season is registered whole in RE-5, when its counter half has an event to
/// watch.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"If you control two Parallel Lives, then the number of tokens created is
///   four times the original number. If you control three, then … eight times
///   the original number, and so on."* → the Furnace pair's third kind: two
///   commuting multipliers on one event, applied both without a CR 616.1
///   prompt. → `two_parallel_lives_create_four_times_as_many_and_ask_nothing`
/// - *"Everything that is specified by the effect creating the original token
///   or tokens will also be true about the additional token or tokens created
///   by Parallel Lives's replacement effect. For example, if an effect tells
///   you to create a token 'tapped and attacking,' the additional tokens will
///   also be tapped and attacking."* → structurally true of a repeated def,
///   asserted on every token's characteristics. "Tapped and attacking" is
///   not a shape this engine's creation can carry yet — a creation has no
///   `EnterMods` — and the ruling's *structure* is what the test proves.
///   → `the_extra_tokens_are_the_same_tokens`
///
/// **Pooled**, with Raise the Alarm — the module doc says why.
pub fn parallel_lives() -> Arc<CardData> {
    CardDataBuilder::new("Parallel Lives")
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(
            "If an effect would create one or more tokens under your control, it creates \
             twice that many of those tokens instead.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::CreateTokens { kind: None },
                ObjectSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Hallowed Moonlight — {1}{W}
/// Instant
///
/// > Until end of turn, if a creature would enter and it wasn't cast, exile
/// > it instead.
/// > Draw a card.
///
/// **Containment Priest's row from a resolution, and without the "nontoken"**
/// — which is the whole reason it is here. The Priest excludes tokens, so no
/// registered card ever substituted a *token's* entry, and the engine's answer
/// for one was the cheap one: a `ZoneChange { from: Battlefield }` for a token
/// that was never there (`codebase-state.md` item 52). This card reaches it,
/// and the answer is now `GameAction::CreateTokenIn` — the token is created in
/// exile.
///
/// Every piece exists since RC-4b and RD-2: `Primitive::CreateReplacement`
/// with an `UntilEndOfTurn` row, `EventPattern::EnterBattlefield { cast:
/// Some(false) }`, a creature filter over the CR 614.12 frame, and
/// `Instead(ZoneChangeTo { Exile })`. "A creature" is every creature — no
/// controller clause — so the row is about the object and names no player.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"After Hallowed Moonlight resolves, if a creature token would be put
///   onto the battlefield, it's put into exile instead and then ceases to
///   exist. Creature tokens are never cast, even if the spell that created
///   them was."* → the log holds `TokenCreated { Exile }` and CR 704.5d's
///   `TokenCeasedToExist`, and **no `ZoneChange`** for the token at all — the
///   line Dour Port-Mage would have read, asserted absent.
///   → `hallowed_moonlight_creates_the_token_in_exile_and_it_ceases_to_exist`
/// - *"Hallowed Moonlight won't affect any creature that was cast, no matter
///   which zone it was cast from and whether or not its mana cost was paid."*
///   → Grizzly Bears cast from hand resolves and enters under it.
///   → `hallowed_moonlight_does_not_affect_a_creature_that_was_cast`
///
/// **Registered and not pooled** — the module doc says why.
pub fn hallowed_moonlight() -> Arc<CardData> {
    CardDataBuilder::new("Hallowed Moonlight")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(
            "Until end of turn, if a creature would enter and it wasn't cast, exile it \
             instead.\nDraw a card.",
        )
        .ability(one_shot(
            AbilityType::Spell,
            Vec::new(),
            Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::CreateReplacement(
                        Box::new(ReplacementDef::new(
                            EventPattern::EnterBattlefield { cast: Some(false) },
                            ObjectSet::Filter {
                                filter: ObjectFilter::ByType(CardType::Creature),
                            },
                            Rewrite::Instead(GameActionTemplate::ZoneChangeTo {
                                to: Zone::Exile,
                                cause: ZoneChangeCause::Exiled,
                            }),
                        )),
                        Duration::UntilEndOfTurn,
                        PatternFill::Authored,
                    ),
                    EffectRecipient::Implicit,
                ),
                Effect::Atom(
                    Primitive::DrawCards(AmountExpr::Fixed(1)),
                    EffectRecipient::Controller,
                ),
            ]),
        ))
        .build()
}

/// The 4/4 white Angel with flying and vigilance Divine Visitation makes —
/// "Angel Token" by CR 111.4.
pub fn angel_token() -> TokenDef {
    TokenDef {
        name: None,
        colors: vec![Color::White],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Angel)],
        supertypes: Vec::new(),
        power: Some(4),
        toughness: Some(4),
        keyword_flags: vec![KeywordFlag::Flying, KeywordFlag::Vigilance],
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
    }
}

/// Divine Visitation — {3}{W}{W}
/// Enchantment
///
/// > If one or more creature tokens would be created under your control, that
/// > many 4/4 white Angel creature tokens with flying and vigilance are
/// > created instead.
///
/// **The kind-changing substitution over a creation, and the kind field's
/// first customer.** `EventPattern::CreateTokens { kind: creature }` matches
/// the creation if any def is a creature, and the template replaces exactly
/// those defs — a Clue created beside a Soldier stays a Clue — with "that
/// many" Angels, where that many is the number the kind matched.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"The token's characteristics are entirely replaced … It doesn't have any
///   abilities the token would have been created with. Anything else
///   specified in the effect creating the token (such as tapped, attacking,
///   …) still applies."* → the template's def replaces the matched def whole,
///   and `enters_tapped` is carried over from the def it replaced.
///   → `divine_visitation_replaces_the_creatures_and_keeps_how_they_entered`
/// - *"If you create a noncreature token that will be a creature as it enters
///   the battlefield (March of the Machines), Divine Visitation's effect
///   doesn't apply"* → the kind is asked of the def's printed types, never of
///   the entry's frame. → `divine_visitation_reads_the_def_and_not_the_frame`
/// - *"If an effect changes under whose control a token would be created, that
///   effect applies before Divine Visitation's"* → CR 616.1b's ladder, which
///   `must_choose_among` already walks; no control-changing creation
///   replacement is registered to walk it with.
///
/// Beside Parallel Lives the affected player chooses the order and the
/// answer is four Angels either way — a multiplier and a replacement by
/// "that many" commute — which the test states by asking both ways.
///
/// **Registered and not pooled**: five mana for an effect two pooled cards
/// reach, and the creation path is measured by the Alarm already.
pub fn divine_visitation() -> Arc<CardData> {
    CardDataBuilder::new("Divine Visitation")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::White], 3))
        .color(Color::White)
        .card_type(CardType::Enchantment)
        .rules_text(
            "If one or more creature tokens would be created under your control, that many \
             4/4 white Angel creature tokens with flying and vigilance are created instead.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::CreateTokens { kind: Some(TokenKind::of_type(CardType::Creature)) },
                ObjectSet::NO_OBJECTS,
                Rewrite::Instead(GameActionTemplate::CreateTokens {
                    def: angel_token(),
                    count: TemplateAmount::ReplacedAmount,
                    mode: TokenSubstitution::Replace,
                }),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Bard, King of Dale — {4}{W}{U}
/// Legendary Creature — Human Noble Archer 3/5
///
/// > Reach, vigilance
/// > If you would draw a card except the first one you draw in each of your
/// > draw steps, draw two cards instead.
/// > If one or more tokens would be created under your control, twice that
/// > many of those tokens are created instead.
///
/// Alhammarret's Archive's draw half beside Parallel Lives' token half. RE-2's
/// ledger row named Bard as waiting on "RE-4's token doubler"; the review
/// found both halves built and registered it (`plans/handoffs/re-4-review.md`,
/// R5 — the review also caught this file's first draft calling it a card that
/// replaces draws with tokens, which is Hullbreacher).
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"If you control two, cards drawn will be multiplied by four … the last
///   ability is cumulative: two, four times the number of tokens."* → two
///   Bards, both halves. → `two_bards_quadruple_both_halves`
/// - *"If an effect creates more than one kind of token, it'll create twice as
///   many of each kind."* → a heterogeneous creation, `[A, B]` → `[A, A, B,
///   B]`. → `bard_doubles_each_kind_of_a_mixed_creation`
/// - *"Copies of permanent spells that resolve become tokens … not created and
///   will not be doubled."* → CR 111.13; `GameEvent::TokenCreated`'s doc is
///   where the engine draws that line, and CV-4 is where the spell copy
///   exists.
/// - *"All of the tokens enter the battlefield simultaneously … same name,
///   color, type …"* → RE-4's batch; `the_extra_tokens_are_the_same_tokens`.
/// - *"If the token … has 'enters with' abilities, first determine how many
///   tokens are being created, then apply those abilities individually for
///   each one."* → CR 616.1g as the order of two loops;
///   `two_devour_tokens_created_together_are_each_asked_and_never_offered_each_other`.
///
/// **Registered and not pooled**: six mana and legendary.
pub fn bard_king_of_dale() -> Arc<CardData> {
    CardDataBuilder::new("Bard, King of Dale")
        .mana_cost(ManaCost::build(&[ManaType::White, ManaType::Blue], 4))
        .color(Color::White)
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Noble))
        .subtype(Subtype::Creature(CreatureType::Archer))
        .power_toughness(3, 5)
        .keyword_flag(KeywordFlag::Reach)
        .keyword_flag(KeywordFlag::Vigilance)
        .rules_text(
            "Reach, vigilance\nIf you would draw a card except the first one you draw in each \
             of your draw steps, draw two cards instead.\nIf one or more tokens would be \
             created under your control, twice that many of those tokens are created instead.",
        )
        .ability(static_replacement(draw_two_instead(Some(DrawCause::Effect))))
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::CreateTokens { kind: None },
                ObjectSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

// ---------------------------------------------------------------------------
// RE-5 — counters, on permanents and players
// ---------------------------------------------------------------------------

/// CR 614.16's counter half over every kind, for a permanent you control —
/// Doubling Season's second ability, the def three cards here share a shape
/// with.
fn doubles_counters_on_your_permanents() -> ReplacementDef {
    ReplacementDef::new(
        EventPattern::AddCounters { counter: None, by: None },
        ObjectSet::Filter { filter: ObjectFilter::ByController(PlayerRef::You) },
        Rewrite::Amount(AmountRewrite::Multiplier(2)),
    )
}

/// Doubling Season — {4}{G}
/// Enchantment
///
/// > If an effect would create one or more tokens under your control, it
/// > creates twice that many of those tokens instead.
/// > If an effect would put one or more counters on a permanent you control,
/// > it puts twice that many of those counters on that permanent instead.
///
/// **Both halves of CR 614.16, on one card.** The token half is Parallel
/// Lives' def word for word (RE-4); the counter half is the first
/// `AddCounters` watcher, and it reads *the permanent's controller* — "a
/// permanent you control" — and nothing about who puts the counters on, which
/// is Vorinclex's question and not this card's. The two are two instances
/// with two CR 614.5 identities: the token half is applied at a creation and
/// the counter half at each entry the creation contains (CR 616.1g), and
/// neither is offered at the other's step.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"Planeswalkers will enter with double the normal number of loyalty
///   counters."* → CR 306.5b's seed through the entry door;
///   `doubling_season_doubles_a_planeswalkers_loyalty`.
/// - *"However, if you activate an ability whose cost has you put loyalty
///   counters on a planeswalker, the number you put on isn't doubled. This
///   is because those counters are put on as a cost, not as an effect."* →
///   not asserted: `Cost::AddCounters` is unimplemented, and the fact the
///   payment will need on the event is `codebase-state.md`'s "Found by
///   RE-5" line.
/// - *"Everything that is specified by the effect creating the original
///   token or tokens will also be true about the additional token or
///   tokens"* → RE-4's `the_extra_tokens_are_the_same_tokens`, on the def
///   this half shares.
/// - *"Doubling Season affects permanents that enter with counters."* →
///   Chainbreaker enters with four -1/-1 counters;
///   `doubling_season_doubles_the_counters_a_permanent_enters_with`. And
///   Master Biomancer's grant, the CR 616.2 board: the Season is not
///   applicable until Biomancer has written the counters, so nothing is
///   asked and the grant is doubled;
///   `doubling_season_doubles_master_biomancers_grant_and_asks_nothing`.
/// - *"Battles will enter with double the normal number of defense
///   counters."* → `backlog.md` §2.23; no battle exists.
/// - *"If there are two Doubling Seasons on the battlefield, then the number
///   of tokens or counters is four times the original number."* → both
///   halves, `test_two_doubling_seasons_quadruple` — §10's acid test, on the
///   counter half by the route RB could not reach.
///
/// **Registered and not pooled.** Five mana, and its token half would double
/// the pool's Soldiers — a gameplay change the A/B should not carry with the
/// engine change (`replacement-architecture.md` §9).
pub fn doubling_season() -> Arc<CardData> {
    CardDataBuilder::new("Doubling Season")
        .mana_cost(ManaCost::build(&[ManaType::Green], 4))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(
            "If an effect would create one or more tokens under your control, it creates \
             twice that many of those tokens instead.\nIf an effect would put one or more \
             counters on a permanent you control, it puts twice that many of those counters \
             on that permanent instead.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::CreateTokens { kind: None },
                ObjectSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .ability(static_replacement(doubles_counters_on_your_permanents()))
        .build()
}

/// Hardened Scales — {G}
/// Enchantment
///
/// > If one or more +1/+1 counters would be put on a creature you control,
/// > that many plus one +1/+1 counters are put on it instead.
///
/// **`AmountRewrite::Plus`'s second kind** — RD-3 landed the arm for Torbran's
/// damage — and the first additive counter replacement. It does not commute
/// with a multiplier, which is why beside Doubling Season the affected
/// permanent's controller is asked, and why beside a second Scales the
/// prompt is asked too though its outcome is one (`backlog.md` §2.29).
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"If a creature you control would enter the battlefield with a number of
///   +1/+1 counters on it, it enters with that many plus one instead."* → the
///   entry door on Master Biomancer's grant;
///   `hardened_scales_adds_one_to_the_counters_a_creature_enters_with`.
/// - *"If two or more effects attempt to modify how many counters would be
///   put on a creature you control, you choose the order to apply those
///   effects, no matter who controls the sources of those effects."* → an
///   opponent's Primal Vigor beside your Scales, and the creature's
///   controller is asked — 1 → 2 → 3 or 1 → 2 → 4;
///   `the_creatures_controller_orders_scales_beside_an_opponents_vigor`.
/// - *"Each additional Hardened Scales you control will increase the number
///   of +1/+1 counters placed on a creature you control by one."* → two
///   rows, each applied once; `two_hardened_scales_add_two`.
///
/// **Pooled**, `PERFORMANCE_POOL` +1 — the module doc says why.
pub fn hardened_scales() -> Arc<CardData> {
    CardDataBuilder::new("Hardened Scales")
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(
            "If one or more +1/+1 counters would be put on a creature you control, that many \
             plus one +1/+1 counters are put on it instead.",
        )
        .ability(static_replacement(ReplacementDef::new(
            EventPattern::AddCounters { counter: Some(CounterType::PlusOnePlusOne), by: None },
            ObjectSet::Filter {
                filter: ObjectFilter::And(
                    Box::new(ObjectFilter::ByType(CardType::Creature)),
                    Box::new(ObjectFilter::ByController(PlayerRef::You)),
                ),
            },
            Rewrite::Amount(AmountRewrite::Plus(1)),
        )))
        .build()
}

/// Vorinclex, Monstrous Raider — {4}{G}{G}
/// Legendary Creature — Phyrexian Praetor 6/6
///
/// > Trample, haste
/// > If you would put one or more counters on a permanent or player, put
/// > twice that many of each of those kinds of counters on that permanent or
/// > player instead.
/// > If an opponent would put one or more counters on a permanent or player,
/// > they put half that many of each of those kinds of counters on that
/// > permanent or player instead, rounded down.
///
/// **The reader of `AddCounters::by`, and the first `Halve` over counters.**
/// Both halves are `Filter { All }` plus `Everyone` — "a permanent or player"
/// is genuinely both questions, Furnace of Rath's shape — and differ only in
/// `by`: `PlayerSet::You` against `PlayerSet::Opponents`, resolved against
/// this card's controller. "Each of those kinds" is the entry door's own
/// arithmetic, applied per kind in the mods.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"Unlike many similar effects, Vorinclex cares deeply about who is
///   putting the counters on the permanent or player to determine which of
///   its two last abilities applies."* → an opponent's Battlegrowth on your
///   creature puts half of one, rounded down — none — and yours on theirs
///   puts two; `vorinclex_reads_who_is_putting_the_counters_on`.
/// - *"If a permanent enters the battlefield with counters on it, the effect
///   causing the permanent to be given counters may specify which player
///   puts those counters on it. If the effect doesn't specify a player, the
///   object's controller puts those counters on it."* → CR 122.6a's default
///   at the entry door: your Loyalty Probe enters with six, an opponent's
///   with one; `vorinclex_reads_the_entering_controller_as_the_putter`.
/// - *"If two or more effects attempt to modify how many counters would be
///   put onto a permanent you control, you choose the order to apply those
///   effects, no matter who controls the sources of those effects."* →
///   beside your Hardened Scales, both orders;
///   `you_order_vorinclex_beside_hardened_scales`.
///
/// The "or player" half is built, not recorded: your Live Fast under your
/// Vorinclex gets four energy, and an opponent's gets one;
/// `vorinclex_doubles_and_halves_the_counters_a_player_gets`.
///
/// **Registered and not pooled**: six mana, legendary, and a body whose
/// engine path Hardened Scales opens at one.
pub fn vorinclex_monstrous_raider() -> Arc<CardData> {
    CardDataBuilder::new("Vorinclex, Monstrous Raider")
        .mana_cost(ManaCost::build(&[ManaType::Green, ManaType::Green], 4))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .subtype(Subtype::Creature(CreatureType::Phyrexian))
        .subtype(Subtype::Creature(CreatureType::Praetor))
        .power_toughness(6, 6)
        .keyword_flag(KeywordFlag::Trample)
        .keyword_flag(KeywordFlag::Haste)
        .rules_text(
            "Trample, haste\nIf you would put one or more counters on a permanent or player, \
             put twice that many of each of those kinds of counters on that permanent or \
             player instead.\nIf an opponent would put one or more counters on a permanent \
             or player, they put half that many of each of those kinds of counters on that \
             permanent or player instead, rounded down.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::AddCounters { counter: None, by: Some(PlayerSet::You) },
                ObjectSet::Filter { filter: ObjectFilter::All },
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::Everyone),
        ))
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::AddCounters { counter: None, by: Some(PlayerSet::Opponents) },
                ObjectSet::Filter { filter: ObjectFilter::All },
                Rewrite::Amount(AmountRewrite::Halve(Rounding::Down)),
            )
            .affecting_players(PlayerSet::Everyone),
        ))
        .build()
}

/// Winding Constrictor — {B}{G}
/// Creature — Snake 2/3
///
/// > If one or more counters would be put on an artifact or creature you
/// > control, that many plus one of each of those kinds of counters are put
/// > on that permanent instead.
/// > If you would get one or more counters, you get that many plus one of
/// > each of those kinds of counters instead.
///
/// **Two `Plus(1)` rows, one per subject.** The object half is Hardened
/// Scales' shape over every kind and an artifact-or-creature filter; the
/// player half is the second player-subject watcher, `NO_OBJECTS` plus
/// `PlayerSet::You`, Rhox Faithmender's shape over a counter event.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"If an artifact or creature you control would enter the battlefield
///   with a number of any kind of counters on it, it enters with that many
///   plus one instead."* → the entry door on Master Biomancer's grant;
///   `winding_constrictor_adds_one_to_the_counters_a_creature_enters_with`.
/// - *"If an effect includes multiple instructions to put one or more
///   counters on an artifact or creature, such as Lifecrafter's Gift does,
///   Winding Constrictor's effect applies to each of those instructions."*
///   → two `Primitive::AddCounters` in one resolution are two proposals;
///   `winding_constrictor_applies_to_each_instruction`.
/// - *"If you control two Winding Constrictors, the number of counters
///   placed on the artifact or creature is the original number plus two."*
///   → `two_winding_constrictors_add_two`.
/// - *"If you would get counters of multiple kinds at the same time, Winding
///   Constrictor increases the number of each of those kinds of counters by
///   one. The same is true if counters of multiple kinds would be placed on
///   an artifact or creature you control."* → an entry carrying two kinds,
///   each plus one; `winding_constrictor_adds_one_of_each_kind_an_entry_carries`.
/// - *"Winding Constrictor's effect can't apply to itself as it's entering
///   the battlefield or to any other permanent entering the battlefield at
///   the same time as it."* → RC-3's membership rule, asserted on the first
///   half: the Constrictor entering under Master Biomancer gets Biomancer's
///   two and not its own plus one;
///   `winding_constrictor_does_not_apply_to_its_own_entry`. The second half
///   is RE-4's plural batch, and no registered card puts the Constrictor in
///   one.
/// - *"If a nonartifact, noncreature permanent … would enter the battlefield
///   with counters on it and become an artifact or a creature on the
///   battlefield due to another card's effect (such as that of Mycosynth
///   Lattice), Winding Constrictor's effect will give that permanent another
///   of those counters."* → the CR 614.12 frame, RC-4's; the filter is asked
///   of the permanent as it would exist. No such Layer 4 effect is
///   registered, so it is not asserted here.
///
/// **Registered and not pooled**: two colors, and the same engine path as
/// Hardened Scales with a second subject the pool has no producer for.
pub fn winding_constrictor() -> Arc<CardData> {
    CardDataBuilder::new("Winding Constrictor")
        .mana_cost(ManaCost::build(&[ManaType::Black, ManaType::Green], 0))
        .color(Color::Black)
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Snake))
        .power_toughness(2, 3)
        .rules_text(
            "If one or more counters would be put on an artifact or creature you control, \
             that many plus one of each of those kinds of counters are put on that permanent \
             instead.\nIf you would get one or more counters, you get that many plus one of \
             each of those kinds of counters instead.",
        )
        .ability(static_replacement(ReplacementDef::new(
            EventPattern::AddCounters { counter: None, by: None },
            ObjectSet::Filter {
                filter: ObjectFilter::And(
                    Box::new(ObjectFilter::Or(
                        Box::new(ObjectFilter::ByType(CardType::Artifact)),
                        Box::new(ObjectFilter::ByType(CardType::Creature)),
                    )),
                    Box::new(ObjectFilter::ByController(PlayerRef::You)),
                ) },
            Rewrite::Amount(AmountRewrite::Plus(1)),
        )))
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::AddCounters { counter: None, by: None },
                ObjectSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Plus(1)),
            )
            .affecting_players(PlayerSet::You),
        ))
        .build()
}

/// Live Fast — {2}{B}
/// Sorcery
///
/// > You draw two cards, lose 2 life, and get {E}{E} (two energy counters).
///
/// **The producer of a player's counters, with nothing else in it**: RE-2's
/// draw instruction, RA's life loss, and `Primitive::GetCounters` — the one
/// new primitive, resolved to its controller.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"Energy counters are a kind of counter that a player may have. They're
///   not associated with any specific permanents."* → `PlayerState::counters`
///   has the kind and no permanent does — the engine's own test,
///   `a_player_gets_counters_through_the_same_event`; the card as printed
///   is card breadth's to test, when fixtures move to set folders.
/// - *"Any effects that interact with counters a player gets, has, or loses
///   can interact with energy counters."* → Vorinclex's and Winding
///   Constrictor's player halves, on this card.
/// - *"To pay one or more {E}, you lose that many energy counters. You can't
///   pay more energy counters than you have."* → a cost, which waits for its
///   first card (`backlog.md` §2.16's close).
///
/// **Registered and not pooled**: a three-mana cantrip whose counters nothing
/// pooled reads. Its job in this phase is to be Vorinclex's and Winding
/// Constrictor's producer.
pub fn live_fast() -> Arc<CardData> {
    CardDataBuilder::new("Live Fast")
        .mana_cost(ManaCost::build(&[ManaType::Black], 2))
        .color(Color::Black)
        .card_type(CardType::Sorcery)
        .rules_text("You draw two cards, lose 2 life, and get {E}{E} (two energy counters).")
        .ability(AbilityDef {
            id: new_ability_id(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(2)), EffectRecipient::Controller),
                Effect::Atom(Primitive::LoseLife(AmountExpr::Fixed(2)), EffectRecipient::Controller),
                Effect::Atom(
                    Primitive::GetCounters {
                        counter: CounterType::Energy,
                        amount: AmountExpr::Fixed(2),
                        by: PlayerRef::You },
                    EffectRecipient::Controller,
                ),
            ]),
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
        })
        .build()
}

/// Primal Vigor — {4}{G}
/// Enchantment
///
/// > If one or more tokens would be created, twice that many of those tokens
/// > are created instead.
/// > If one or more +1/+1 counters would be put on a creature, twice that
/// > many +1/+1 counters are put on that creature instead.
///
/// **Doubling Season with `Everyone` on the token half and no controller on
/// the counter half** — "it doesn't matter who controls the tokens or the
/// creature", the ruling says, and the N-player table is where that shows.
///
/// # The rulings (Scryfall, 2026-09-13), and where each is tested
///
/// - *"Everything that is specified by the effect creating the original
///   token or tokens will also be true about the additional token or
///   tokens"* → RE-4's `the_extra_tokens_are_the_same_tokens`.
/// - *"It doesn't matter who controls the tokens or the creature that the
///   +1/+1 counters are being placed on."* → four players: an opponent's
///   Raise the Alarm makes four, and a third player's Battlegrowth on a
///   fourth's creature puts two;
///   `primal_vigor_does_not_care_who_controls_the_tokens_or_the_creature`.
/// - *"Primal Vigor affects permanents that 'enter the battlefield with' a
///   certain number of counters. For example, if a creature would normally
///   enter the battlefield with three +1/+1 counters on it, it will enter
///   with six."* → Adaptive Shimmerer enters with six;
///   `primal_vigor_doubles_the_counters_a_creature_enters_with`.
/// - *"If there are two Primal Vigors on the battlefield, the number of
///   tokens or +1/+1 counters is four times the original number."* → the
///   Season's acid test on the symmetric card; `two_primal_vigors_quadruple`.
///
/// **Registered and not pooled**, for Doubling Season's reason with
/// `Everyone` on top of it.
pub fn primal_vigor() -> Arc<CardData> {
    CardDataBuilder::new("Primal Vigor")
        .mana_cost(ManaCost::build(&[ManaType::Green], 4))
        .color(Color::Green)
        .card_type(CardType::Enchantment)
        .rules_text(
            "If one or more tokens would be created, twice that many of those tokens are \
             created instead.\nIf one or more +1/+1 counters would be put on a creature, \
             twice that many +1/+1 counters are put on that creature instead.",
        )
        .ability(static_replacement(
            ReplacementDef::new(
                EventPattern::CreateTokens { kind: None },
                ObjectSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::Everyone),
        ))
        .ability(static_replacement(ReplacementDef::new(
            EventPattern::AddCounters { counter: Some(CounterType::PlusOnePlusOne), by: None },
            ObjectSet::Filter { filter: ObjectFilter::ByType(CardType::Creature) },
            Rewrite::Amount(AmountRewrite::Multiplier(2)),
        )))
        .build()
}
