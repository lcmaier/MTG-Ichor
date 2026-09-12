//! Replacement and prevention effects (CR 614, 615) — the type surface.
//!
//! A replacement effect is neither a `ContinuousEffect` nor a `Primitive`. It
//! does not apply in a layer, so it cannot be a registry row; it does not run
//! at resolution, so it is not an `Effect::Atom`. It gets its own type, reached
//! through [`Effect::Replacement`](crate::types::effects::Effect::Replacement).
//!
//! The pipeline that consumes these lives in `engine::replacement`; this module
//! is data only, so that a `ReplacementDef` can be written in a card file
//! without the card file reaching into the engine.
//!
//! # Two growth contracts, and they are the point
//!
//! `replacement-architecture.md` §3.2 committed to **one** open-ended enum in
//! this phase, not two. Both contracts below are meant to be enforced in review
//! the way the layer system's "registry membership is not effect existence" is.
//!
//! - [`EventPattern`] grows on **exactly one axis**: one arm per
//!   `GameAction` variant. It is a mechanical projection of the event
//!   vocabulary, not an independent taxonomy. A card that needs a pattern this
//!   cannot express is telling you the missing thing is a `GameAction` variant
//!   or a field on one — a replacement effect can only watch for an event the
//!   engine actually proposes.
//! - [`Rewrite`] is a **closed algebra**. CR 614 and 615 enumerate what a
//!   replacement effect may do to an event and the list is short; a new arm
//!   is a claim that those rules permit an operation the list omits, and it
//!   should arrive with the rule number that says so. It ships **six** —
//!   `Prevent`, `Instead`, `EnterWith`, `EnterUnderControlOf`,
//!   `EnterAfterMoving` and `Amount` — still not §3.2's six: an arm the
//!   pipeline cannot apply is worse than a missing one, and `Retarget` waits
//!   for RD-4. `EnterWith` gained its performer in Phase RC-2,
//!   `EnterUnderControlOf` its CR 616.1b bucket in RC-4, `EnterAfterMoving`
//!   arrived in RC-5 with CR 614.13, and `Amount` in RD-1 with CR 701.10g's
//!   doublers and CR 615.10's partial prevention.
//!
//! Per-mechanic variety goes in [`ReplacementDef::then`], which is the existing
//! `Effect` tree — no new vocabulary at all.

use crate::types::effects::{
    AffectedSet, AmountExpr, CounterType, Effect, ObjectFilter, PlayerRef, PlayerSet,
};
use crate::state::game_state::{PhaseType, StepType};
use crate::types::ids::ObjectId;
use crate::types::zones::{DestructionSource, DrawCause, LifeLossCause, Zone, ZoneChangeCause};

/// One replacement or prevention effect.
///
/// CR 614.1: replacement effects "watch for a particular event that would
/// happen" around "whatever they're affecting". [`Self::pattern`] is what it
/// watches for and [`Self::affected`] is what it is affecting.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplacementDef {
    /// Which proposed events this watches (CR 614.1, 615.1).
    pub pattern: EventPattern,

    /// Which objects it applies to.
    ///
    /// Not "which objects it shields": CR 614.1's shield is a metaphor for
    /// every replacement effect, but in *this* codebase "shield" is taken —
    /// CR 701.19a's regeneration shield and CR 122.1c's shield counter — and
    /// Kalitas protects nothing it applies to. Same name as
    /// `ContinuousEffect::affected` because it is the same question.
    ///
    /// Reuses the layer system's `AffectedSet`, and the reuse is load-bearing:
    /// `SourceOnly` vs. `Filter` is exactly CR 614.12's "affects only that
    /// permanent (as opposed to a general subset of permanents that includes
    /// it)". If a future refactor collapses those variants, 614.12 breaks
    /// silently (`replacement-architecture.md` §11 item 2).
    pub affected: AffectedSet,

    /// Which **players** it applies to — the other half of CR 614.1's
    /// "whatever they're affecting", unioned with [`Self::affected`].
    ///
    /// A second field rather than an `AffectedSet` variant, for the reason
    /// [`PlayerSet`]'s own docs give. Furnace of Rath is `Filter { All }` plus
    /// `Everyone`, because "a permanent **or player**" is genuinely both
    /// questions; Angel of Suffering is `Fixed(vec![])` plus `You`, an effect
    /// about no object at all.
    ///
    /// [`PlayerSet::Nobody`] on every effect written before Phase RD, which is
    /// what [`ReplacementDef::new`] gives it.
    pub affected_players: PlayerSet,

    /// How it rewrites a matching event.
    pub rewrite: Rewrite,

    /// The "and also" half: CR 615.5's "the rest of the effect takes place
    /// immediately afterward", CR 701.19a's tap-and-remove-from-combat,
    /// CR 122.1c's counter removal on the prevention half.
    ///
    /// **This is the existing `Effect` tree** — no new vocabulary, and it is
    /// where per-mechanic variety goes.
    ///
    /// **Timing contract (§4.1a):** queued when this replacement is applied,
    /// resolved immediately *after* the final modified event is performed —
    /// never mid-loop. During the CR 616.1f loop nothing has happened yet; a
    /// rider resolved mid-loop runs before the event it rides on, which inverts
    /// the LKI frame order the moment triggers land. Unconditional once queued
    /// (CR 615.12: prevention effects applied to unpreventable damage prevent
    /// nothing, "but any additional effects they have will take place"), and it
    /// re-enters the pipeline with a fresh applied-set — a rider's actions are
    /// new events the replacement *caused*, not modified forms of the original.
    ///
    /// It resolves against a `ResolutionContext` whose single target is the
    /// event's subject, so `EffectRecipient::Target`/`Choose` name the
    /// permanent the event was about and `EffectRecipient::Controller` names
    /// its controller.
    pub then: Option<Effect>,

    /// CR 616.1a–d — which step of the choice ladder this falls in.
    pub class: ReplacementClass,

    /// How many times it can fire.
    pub uses: Uses,

    /// Is this a CR 701.19 regeneration shield?
    ///
    /// **One rule reads it, and it is a rules-level classification rather than
    /// per-mechanic variety.** CR 701.19c: "effects that say that a permanent
    /// can't be regenerated ... cause regeneration shields to not be applied" —
    /// so the pipeline has to be able to recognise a regeneration shield in
    /// order to withhold it, and nothing about the shield's pattern, rewrite or
    /// rider distinguishes it from any other `Prevent`-with-a-rider.
    ///
    /// A second reader would be the smell. This is not the place to record what
    /// a replacement effect *is about*; that is `pattern`.
    pub is_regeneration: bool,

    /// CR 903.9b is an explicit exception to CR 614.5 and is **the only one in
    /// the rules**. Must not grow a second user without a CR cite.
    pub exempt_from_614_5: bool,

    /// "you **may** ... instead" — Retriever Phoenix, Library of Leng, and 14
    /// others (Scryfall 2026-08-24). The affected player is asked before the
    /// effect is applied.
    ///
    /// **Declining marks it applied but does not consume a use.** Both halves
    /// are load-bearing: marking it applied is CR 614.5's "one opportunity" —
    /// being offered and refusing *is* the opportunity, and without it the loop
    /// re-gathers the same candidate forever, which is a hang rather than a
    /// wrong answer. Not consuming a use is what leaves a regeneration shield
    /// intact for the *next* event.
    pub optional: bool,
}

/// A predicate over a proposed `GameAction`.
///
/// Data rather than a closure for the same reason `ObjectFilter` is:
/// closures cannot be compared, cloned cheaply, or inspected by a loop
/// detector.
///
/// # The growth contract
///
/// **Exactly one arm per `GameAction` variant, and it grows on no other axis.**
/// Within an arm, constraints on the event's fields reuse existing vocabulary —
/// `ObjectFilter`, `ZoneChangeCause`, `CardType` — rather than inventing
/// per-mechanic predicates. A change that adds an arm here without a
/// corresponding `GameAction` change is the smell this contract exists to
/// catch.
///
/// # Why ten arms and not fourteen
///
/// `GameAction` ships fifteen variants and this enum ten. `CounterChange`
/// covers `AddCounters` and `RemoveCounters` through its `adding` field — the
/// one place the projection is not 1:1, and that arm's own doc says so.
///
/// The four with no arm at all are `Attach` — on purpose, since nothing
/// replaces an attach — and `DrawCard`, `GainLife` and `LoseLife`. Those three
/// land in RE-2 and RE-3, which is where `replacement-architecture.md` §9
/// schedules draw replacement (CR 614.11) and life-gain replacement
/// (CR 119.10) anyway. Adding an arm is a normal diff — this enum is matched
/// exhaustively and is not `#[non_exhaustive]`, so every reader fails to
/// compile rather than defaulting. **`gather::pattern_watches` is the reader
/// that does not**: it falls through to `false`, so a `GameAction` variant with
/// no arm there is silently unwatchable, which is `Attach`'s intended state and
/// the trap for everything else.
///
/// **Until RD-1 the reason given here was that no set could scope one to a
/// player, and that reason is gone.** [`ReplacementDef::affected_players`]
/// scopes an effect to a player, because the damage family needed it first
/// (§9's RD decision 0, §11 item 21). What is left is that no registered card
/// wants one yet — which is the ordinary "an arm the pipeline cannot apply is
/// worse than a missing one", not a missing mechanism.
#[derive(Debug, Clone, PartialEq)]
pub enum EventPattern {
    /// CR 614.2 / 615.1. The event's subject is the damage *target*.
    ///
    /// **Two fields, and both are about the event rather than about the
    /// effect.** `affected` says which targets the effect is around; these say
    /// which damage counts once it is. Both `None` is RB's and RD-1's and
    /// RD-2's shape — "damage would be dealt to whatever I am affecting" —
    /// and stays the common case.
    ///
    /// `source` is CR 609.7's source-side predicate; see [`SourcePattern`] for
    /// which half of 609.7 each of its own fields is.
    ///
    /// `combat` is CR 510.2's fact off the proposal's `is_combat`, and **both
    /// answers are printed.** Fog's "prevent all *combat* damage" is
    /// `Some(true)`; "prevent all *noncombat* damage" is `Some(false)`, which
    /// nine cards say (Scryfall 2026-09-09: Purity, Mark of Asylum, The
    /// Wanderer, Blessed Sanctuary, Magebane Armor, Stormwild Capridor, Tajic,
    /// Crystal Barricade, Drogskol Reinforcements). Purity is Reverse Damage's
    /// def exactly, with this field where the chosen source goes.
    ///
    /// So the `Option` is not a two-arm enum wearing a `bool`: there are
    /// **three** answers and `None` is the third — the effect does not ask
    /// about combat at all, which is what every def written before RD-3
    /// carries and what a card like Guardian Seraph means by "1 of any damage,
    /// not just combat damage".
    ///
    /// **Neither field reads the amount, which is load-bearing**:
    /// `pipeline::ordering_cannot_change_outcome` suppresses CR 616.1's prompt
    /// for a bucket of commuting multipliers on this pattern, and its
    /// termination argument holds only while no field of this arm can make a
    /// member fall out of applicability as another member changes the number
    /// (`codebase-state.md` item 47's condition (d)).
    DealDamage {
        source: Option<SourcePattern>,
        combat: Option<bool>,
    },

    /// CR 400.6. `None` on a field means "any".
    ///
    /// **Watches an entry too, when `to` admits the battlefield.** Entering is
    /// the zone change onto it (CR 614.1c), proposed as
    /// `GameAction::EnterBattlefield` and nothing else, and `pattern_watches`
    /// compares `from` and `cause` against the entry's. That is Worms of the
    /// Earth's and Grafdigger's Cage's shape, and it shares CR 616.1's step
    /// with [`Self::EnterBattlefield`] — one event.
    ZoneChange {
        from: Option<Zone>,
        to: Option<Zone>,
        /// Nothing outside this pipeline and the trigger matcher may branch on
        /// a cause — see [`ZoneChangeCause`].
        cause: Option<ZoneChangeCause>,
        /// A constraint on the moving object beyond what `affected` says.
        ///
        /// `affected` scopes *which* objects the effect applies to; this scopes
        /// the event. They coincide for every RB card, and the field exists
        /// because CR 903.9b's "if a **commander** would be put into its
        /// owner's hand or library" is a property of the object being moved
        /// while the shield is around a player's cards generally.
        object: Option<ObjectFilter>,
    },

    /// CR 122.1d. The event's subject is the permanent being untapped.
    Untap,

    /// CR 121.2a / 614.11 — **the instruction**, "if an opponent would draw
    /// two or more cards". The event's subject is the drawing player.
    ///
    /// `at_least` is the only constraint printed on a draw instruction and it
    /// is printed once: Alms Collector. Two other cards say "two or more"
    /// about draws and both are triggers (Scryfall, 2026-09-11). `None` asks
    /// nothing about the count and has no printed customer either — it is the
    /// field's honest default rather than an arm, the same way
    /// [`Self::BeginStep`]'s `None` is.
    ///
    /// **This never watches a [`Self::DrawCard`], and that is the ruling
    /// rather than an implementation choice.** *"To determine whether a player
    /// is instructed to draw multiple once or instructed multiple times to
    /// draw one card, count how many times the word 'draw' is used."* One
    /// instruction of two cards is this event; two cantrips are two individual
    /// draws with no instruction over them for this to match.
    DrawCards {
        at_least: Option<u64>,
    },

    /// CR 121.1 / 614.11 — **one card being drawn**, which is what Thought
    /// Reflection, Teferi's Ageless Insight and Notion Thief watch. The event's
    /// subject is the drawing player.
    ///
    /// `cause` is the draw's own
    /// [`DrawCause`](crate::types::zones::DrawCause), not its instruction's:
    /// `Some(Effect)` is the ten cards' "except the first one you draw in each
    /// of your draw steps", and `None` — Thought Reflection's — matches either.
    /// A doubler applied to the draw step's draw leaves a first card that is
    /// still `TurnBased` and a second that is not, which is how the exception
    /// survives being doubled.
    DrawCard {
        cause: Option<DrawCause>,
    },

    /// CR 119.3 / 119.10 — "if you would gain life". The event's subject is
    /// the gaining player.
    ///
    /// **No fields, and the census is why.** CR 119.10 restates every printed
    /// one as "if a source would cause [a player] to gain life", and the
    /// twenty-one printed "would gain life" replacements scope themselves by
    /// *which* player — [`ReplacementDef::affected_players`]'s question, which
    /// is why Rhox Faithmender (`You`) and Tainted Remedy (`Opponents`) differ
    /// in nothing else. Nothing prints a constraint on the source or on the
    /// amount, and an arm the pipeline cannot apply is worse than a missing
    /// one (§3.2a).
    ///
    /// **Reading no amount is load-bearing**, and it is the same clause
    /// [`Self::DealDamage`] carries: `pipeline::ordering_cannot_change_outcome`
    /// suppresses CR 616.1's prompt for a bucket of commuting multipliers over
    /// this pattern, and the premise holds only while no field here can make a
    /// member fall out of applicability as another member changes the number.
    /// `pipeline::pattern_reads_the_amount` is the leaf table that says so.
    GainLife,

    /// CR 119.3 / 120.3a — "if you would lose life". The event's subject is
    /// the losing player.
    ///
    /// `cause` is the loss's own [`LifeLossCause`](crate::types::zones::LifeLossCause),
    /// and it is the field Ali from Cairo exists for: its clamp is on *damage*,
    /// and its own ruling is that the effect "does not prevent damage, it
    /// prevents the damage from turning into loss of life" — so it watches the
    /// loss CR 120.3a contains inside the damage rather than the damage.
    /// `None` matches any loss, which is Bloodletter of Aclazotz's shape and
    /// the reason the card prints "(Damage causes loss of life.)".
    ///
    /// Reads no amount, for [`Self::GainLife`]'s reason.
    LoseLife {
        cause: Option<LifeLossCausePattern>,
    },

    /// CR 603.2e's counterpart. No printed customer in RB; the arm exists
    /// because `GameAction::Tap` exists and the contract above says one arm
    /// per variant.
    Tap,

    /// CR 701.8b / 614.8. The **outer** event: performing it proposes an inner
    /// `ZoneChange { cause: Destroyed | DestroyedBySba }`.
    Destroy {
        /// CR 122.1c reads "would be destroyed **as the result of an effect**",
        /// which is CR 701.8b way 1 only. `None` matches either way.
        source: Option<DestructionSourcePattern>,
    },

    /// CR 614.1c/d — "modify how a permanent enters the battlefield". The
    /// event's subject is the permanent that is entering.
    ///
    /// **`affected` does the object-side work, and it reads the CR 614.12
    /// frame.** Root Maze's "artifacts and lands" is an `AffectedSet::Filter`,
    /// matched against the permanent *as it would exist on the battlefield*
    /// (`layers::compute_as_entering`), never against the card where it came
    /// from. So this arm carries no object filter of its own; its one
    /// constraint is about the *event*.
    ///
    /// `cast` is CR 601's fact, projected off the entry's [`ZoneChangeCause`]:
    /// `Some(true)` matches a permanent spell that resolved (`Resolved`),
    /// `Some(false)` everything else — a land drop, an effect putting a card
    /// onto the battlefield, a token. Containment Priest and Hallowed
    /// Moonlight read "and it wasn't cast", and a `ZoneChangeCause` field
    /// could not say *wasn't*. `None` matches either.
    ///
    /// One field today, and the axis it grows along is CR 400.7d: a permanent's
    /// abilities may reference facts about the spell it was — whether it was
    /// kicked (Gnarlid Pack's "enters with a +1/+1 counter" is a CR 614.1c
    /// effect reading one), what was spent on it, where it was cast from. A
    /// fact about *how the object arrived* lands here as a field; a fact about
    /// the card where it is stays on [`Self::ZoneChange`]'s `object`.
    ///
    /// **A prohibition on entering may watch this event** (CR 614.17d). The
    /// entry is the zone change, decided before anything moves, so refusing it
    /// leaves the card where it was. The printed family — Worms of the Earth,
    /// Grafdigger's Cage — is registered in [`Self::ZoneChange`]'s shape, which
    /// watches an entry too; both doors open onto one CR 616.1 step, and the
    /// frame (`engine::replacement::EntryFrame`) answers at either.
    EnterBattlefield {
        cast: Option<bool>,
    },

    /// CR 122.1's counter mutations. No RB customer watches one — the arm
    /// exists because `GameAction::AddCounters`/`RemoveCounters` do, and
    /// CR 614.16's counter doublers (Doubling Season's second ability, Vorinclex)
    /// are its Phase RE customers.
    CounterChange {
        counter: Option<CounterType>,
        /// `true` matches `AddCounters`, `false` matches `RemoveCounters`.
        adding: bool,
    },

    /// CR 614.1b / 614.10 — "skip your next turn". The event's subject is the
    /// player whose turn it would be.
    ///
    /// **No fields, and that is the census talking.** The twenty-two printed
    /// turn skips say "skip your next turn", "that player skips their next
    /// turn" or "players skip their turns" — every one of them scoped by
    /// *which player*, which is [`ReplacementDef::affected_players`]'s question
    /// and not this one's. Nothing prints a constraint on the turn itself, and
    /// an arm the pipeline cannot apply is worse than a missing one (§3.2a).
    /// The field a later card could want is CR 500.7's "is this an extra
    /// turn"; no printed card asks, and it lives on the *schedule* rather than
    /// on the event for that reason — see `GameAction::BeginTurn`.
    BeginTurn,

    /// CR 614.1b / 614.10 — "skips their next combat phase". The event's
    /// subject is the active player.
    ///
    /// `None` matches any phase. Nothing printed says "skips their next phase"
    /// unqualified, so the registered def names one; the field is here because
    /// six printed cards name the combat phase and two name a main phase, and
    /// a pattern that could not tell them apart would make Moment of Silence
    /// eat the beginning phase.
    BeginPhase {
        phase: Option<PhaseType>,
    },

    /// CR 614.1b / 614.10 — "skip your draw step", "players skip their upkeep
    /// steps". The event's subject is the active player.
    ///
    /// `None` matches any step, which is how "skip all of your steps" would be
    /// written if a card printed it; both registered defs name one.
    BeginStep {
        step: Option<StepType>,
    },
}

/// CR 609.7's source-side predicate — "a **red** source of your choice", "a
/// source **an opponent controls**" — in one struct with two fields, because
/// the rule has two halves and a card may write either, both or neither.
///
/// - [`Self::object`] is **CR 609.7a's chosen source**: "the source is chosen
///   when the effect is created". A card authors `None` and the resolution
///   that creates the row overwrites it with what the player picked
///   (`Primitive::CreateReplacement` with `PatternFill::ChosenDamageSource`).
///   A static ability leaves it `None` forever — Guardian Seraph chooses
///   nothing.
/// - [`Self::filter`] is **CR 609.7b and 609.7c's property**, and it is
///   rechecked at every proposal rather than captured: 609.7b says "when the
///   source would deal damage, the shield rechecks the source's properties",
///   and the recheck is free here because `gather` asks
///   `pattern_watches` off the board at the moment of the proposal. A source
///   that has stopped matching yields no candidate, so nothing is applied and
///   `consume_use` never runs — which is 609.7b's second sentence
///   ("the shield isn't used up") falling out rather than being implemented.
///
/// The two compose: Circle of Protection: Red is both, and it is both
/// *because* the CR is — the chosen creature turning blue stops the shield
/// even though the id still matches.
///
/// # The filter is general, and the guard that decided it
///
/// §8c's axis 2 says card breadth lands on predicates, and its second guard is
/// "two customers before a leaf". Applied live in RD-3: the leaves this field
/// actually reaches are `ByColor` (Circle of Protection: Red, Torbran),
/// `ByController` (Guardian Seraph, Torbran) and `And` — **all three already
/// in `ObjectFilter` with customers of their own**, so a general filter costs
/// no new leaf at all and narrower per-card leaves would have cost three. The
/// one leaf a consumer wanted and did not get is "the effect's own host as the
/// source" (Sokrates, Athenian Teacher's granted "if **this creature** would
/// deal combat damage to a player"); it has exactly one customer, that customer
/// is unregistered for an unrelated reason, so it is recorded here and not
/// written.
///
/// **`AffectedSet` is not reusable for this**, which is worth saying because
/// the shapes rhyme. That type answers "which objects is this effect around",
/// and its `SourceOnly`/`Host`/`Fixed` arms are all about the effect's own
/// source; this one answers "which object dealt the damage", where the same
/// three words would mean something else.
#[derive(Debug, Clone, PartialEq)]
pub struct SourcePattern {
    /// CR 609.7a's chosen source, by id. `None` matches any source.
    pub object: Option<ObjectId>,
    /// CR 609.7b/c's property, rechecked at the proposal. `None` asks nothing.
    ///
    /// Resolved against the **effect's controller** as CR 109.5's "you", the
    /// same player `set_affects` resolves the affected set against.
    pub filter: Option<ObjectFilter>,
}

impl SourcePattern {
    /// "A source of your choice", with no property — Reverse Damage, Dark
    /// Sphere. The `object` is filled in by the resolution.
    pub fn chosen() -> Self {
        SourcePattern { object: None, filter: None }
    }

    /// "A source of your choice with this property" — Circle of Protection:
    /// Red — or, on a static ability, the property alone.
    pub fn matching(filter: ObjectFilter) -> Self {
        SourcePattern { object: None, filter: Some(filter) }
    }
}

/// Which of CR 701.8b's ways destroyed the permanent.
///
/// A pattern rather than a [`DestructionSource`]: the event carries the
/// destroying object's id and a pattern must not have to name it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestructionSourcePattern {
    /// CR 701.8b way 1 — an effect that uses the word "destroy".
    Effect,
    /// CR 701.8b ways 2 and 3 — the CR 704.5g/h state-based actions.
    StateBasedAction,
}

impl DestructionSourcePattern {
    pub fn matches(self, source: DestructionSource) -> bool {
        matches!(
            (self, source),
            (DestructionSourcePattern::Effect, DestructionSource::Effect(_))
                | (
                    DestructionSourcePattern::StateBasedAction,
                    DestructionSource::StateBasedAction
                )
        )
    }
}

impl EventPattern {
    /// Does any field of this pattern constrain the event's **amount**?
    ///
    /// The premise `pipeline::ordering_cannot_change_outcome` needs for its
    /// multiplier shape: a bucket of commuting doublers reaches one outcome in
    /// any order *only if* no member can stop applying as another member
    /// changes the number. A pattern that reads no amount cannot.
    ///
    /// **It lives here rather than beside its caller, and that is the third
    /// time this premise has been got wrong.** RD-1 wrote it as "the pattern is
    /// `EventPattern::DealDamage`" and RE-2 found the same clause missing from
    /// the draw shape (`replacement-architecture.md` §11 items 19, 55); RE-3
    /// found a `Multiplier` over a life gain falling through the damage gate.
    /// Each time the axis that moved was **this enum**, which grows one arm per
    /// `GameAction` variant every replacement phase — so the classification
    /// belongs where the arm is written, in front of whoever writes it, rather
    /// than in a predicate they have no reason to open. Matched exhaustively,
    /// so a new arm has to answer rather than defaulting to "safe".
    ///
    /// The contrast is `pipeline::filter_is_mods_invariant`, which stays at its
    /// caller: that one classifies an `ObjectFilter` against a property of
    /// `EnterMods`, a relation between two types and so a fact about neither.
    /// This is a property of one arm, answerable from its own definition.
    pub fn reads_the_amount(&self) -> bool {
        match self {
            // CR 121.2a's "two or more cards" — Alms Collector, and the only
            // printed pattern in the engine that reads a count. A doubler
            // beside it is exactly the board the premise excludes: 1 → 2 puts
            // the event inside `at_least: Some(2)` where it was outside.
            EventPattern::DrawCards { .. } => true,

            // A source predicate and a combat flag (CR 609.7, 510.2).
            EventPattern::DealDamage { .. } => false,
            // No fields at all — CR 119.10 restates every printed life-gain
            // replacement as a question about the *source*, never the amount.
            EventPattern::GainLife => false,
            // A cause (CR 120.3a / 119.4).
            EventPattern::LoseLife { .. } => false,
            // Zones, a cause and a filter on the moving object.
            EventPattern::ZoneChange { .. } => false,
            // One individual draw. CR 121.2 makes it one card, so there is no
            // amount for a field to read.
            EventPattern::DrawCard { .. } => false,
            // CR 601's fact about how the permanent arrived.
            EventPattern::EnterBattlefield { .. } => false,
            // CR 701.8b's two ways.
            EventPattern::Destroy { .. } => false,
            // A counter kind and a direction, never a count: CR 122.6's
            // doublers are written about "one or more", which is every
            // `AddCounters` this can match.
            EventPattern::CounterChange { .. } => false,
            // A turn, a phase, a step. CR 614.10's units are not amounts.
            EventPattern::Untap
            | EventPattern::Tap
            | EventPattern::BeginTurn
            | EventPattern::BeginPhase { .. }
            | EventPattern::BeginStep { .. } => false,
        }
    }
}

/// [`LifeLossCause`] with its payload dropped — what
/// [`EventPattern::LoseLife`] can ask about a loss.
///
/// **A projection and not the cause itself**, exactly as
/// [`DestructionSourcePattern`] is of [`DestructionSource`]: `LifeLossCause::Damage`
/// carries the damage's source, and Ali from Cairo's "damage that would reduce
/// your life total" is about damage from *anything*. A pattern holding the cause
/// verbatim could only name one source object, which no printed card does — and
/// the field a card could want beside it is CR 609.7's source predicate, which
/// [`SourcePattern`] already is and which nothing prints about a life loss.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifeLossCausePattern {
    /// CR 120.3a — the loss contained in damage.
    Damage,
    /// A resolving spell or ability's "loses N life".
    Effect,
    /// CR 119.4's life payment.
    Cost,
}

impl LifeLossCausePattern {
    pub fn matches(self, cause: LifeLossCause) -> bool {
        matches!(
            (self, cause),
            (LifeLossCausePattern::Damage, LifeLossCause::Damage { .. })
                | (LifeLossCausePattern::Effect, LifeLossCause::Effect)
                | (LifeLossCausePattern::Cost, LifeLossCause::Cost)
        )
    }
}

/// What a replacement effect does to a matching event.
///
/// # The completeness claim, and where it was checked
///
/// CR 614 and 615 enumerate what a replacement effect may do to an event, and
/// the list is short. The claim was tested rather than asserted: every card
/// whose oracle text matches `o:/would.*instead/ -is:funny` was pulled from
/// Scryfall (2026-08-24) and each matching clause classified — **561 cards,
/// 574 clauses, and zero of them needed a sixth arm**
/// (`plans/references/replacement-census.py`). The pressure went entirely onto
/// the `GameAction` vocabulary, which is bounded by the engine's own mutations
/// plus CR 701 rather than by the card pool.
///
/// The full algebra, with the phase that gives each arm a customer:
///
/// | Arm | CR | Phase |
/// |---|---|---|
/// | `Prevent` | 614.6, 615.6 | **RB** |
/// | `Instead` | 614.1a | **RB** |
/// | `EnterWith(..)` | 614.1c/d | **RC-2** |
/// | `EnterUnderControlOf(..)` | 616.1b, 614.1c | **RC-4** |
/// | `Amount(..)` | 614.5 doublers, 615.7 partial prevention | **RD-1** |
/// | `Retarget(..)` | 614.9 redirection | **RD-4** |
///
/// Every arm now has a customer, so the enum is the algebra as claimed rather
/// than the algebra minus one.
#[derive(Debug, Clone, PartialEq)]
pub enum Rewrite {
    /// CR 614.6 / 615.6 — the event does not happen.
    ///
    /// Distinguishable from an `Instead` that produces nothing on purpose:
    /// CR 615.13 lets triggers fire on damage *being prevented*, and CR 615.12
    /// needs the engine to know a prevention was attempted.
    Prevent,

    /// CR 614.1a's general "instead" — replace the event with a different
    /// proposed action. The escape hatch, and the only unbounded arm; that is
    /// the CR's own shape, since 614.1a says replacement effects "use the word
    /// *instead* to indicate what events will be replaced with other events".
    ///
    /// It costs nothing, because the substitute is a `GameAction`, a
    /// vocabulary that already has to exist.
    Instead(GameActionTemplate),

    /// CR 614.1c/d — modify *how* a permanent enters the battlefield, without
    /// changing the fact that it enters.
    ///
    /// **Not an `Instead`, and the difference is accumulation.** CR 616.1f
    /// re-runs the loop against the modified event, so a permanent facing two
    /// applicable effects — "enters tapped" and "enters with two charge
    /// counters" — has to end up with *both*, not with whichever applied last.
    /// An `Instead` overwrites the event; this one merges into it, and
    /// [`EnterMods::merge`] is where the rule that each mod composes lives.
    ///
    /// The merge happens while the permanent **does not yet exist**: there is
    /// no `PermanentState` to tap and no counter map to write, which is the
    /// whole reason the modifications ride on the proposal instead of being
    /// applied as they are chosen.
    ///
    /// Carries an [`EnterModsTemplate`] rather than an [`EnterMods`]: the
    /// effect's *text* may name an amount ("equal to this creature's power")
    /// that the event's mods cannot, because the performer needs a number.
    EnterWith(EnterModsTemplate),

    /// CR 614.13 — an entry modification that **causes other objects to change
    /// zones** as it is applied.
    ///
    /// > 614.13. An effect that modifies how a permanent enters the battlefield
    /// > may cause other objects to change zones.
    ///
    /// The rule that permits the arm, which is what the closed algebra asks of
    /// a new one.
    ///
    /// **The printed population, counted 2026-09-03.** Devour is **23 cards**
    /// (`keyword:devour`), of which four are CR 702.82c's *devour [quality]* —
    /// artifacts, lands, Foods — which cost nothing here because the payload's
    /// `filter` is what says "creatures". "As ~ enters, exile … from your
    /// graveyard" is **5** more: Sutured Ghoul, Living Lore, Dermotaxi,
    /// Mimeoplasm Revered One, Frankenstein's Monster. They differ only in the
    /// payload's fields.
    ///
    /// **Two of the 23 the payload does not reach**, and neither wants a new
    /// arm: Thromok the Insatiable's "devour X, where X is the number of
    /// creatures devoured this way" makes the multiplier *be* the count
    /// (`codebase-state.md` item 63), and Frankenstein's Monster exiles exactly
    /// X with a "if you can't" failure branch. **Sweep is not one of these** —
    /// "return any number of Mountains you control to their owner's hand" is an
    /// instruction of a resolving spell, not a modification of an entry, so it
    /// is a `Primitive` and never reaches CR 614.13.
    ///
    /// **Not an `EnterWith` with a rider.** A `then` runs *after* the performed
    /// event (§4.1a, CR 615.5), and these moves happen while the effect is
    /// being applied — the counters the entry arrives with are computed from
    /// them, so a rider is too late by construction. It is also the only
    /// rewrite that is not a pure function of the event: applying it prompts.
    ///
    /// CR 614.13a and 614.13b scope that prompt to a *batch*
    /// (`GameState::entry_selection`), not to one event, which is why the two
    /// exclusion sets are not fields here.
    EnterAfterMoving(AuxiliaryMove),

    /// CR 616.1b — modify *under whose control* a permanent enters.
    ///
    /// The rule names this class by what the effect does — "would modify
    /// under whose control an object would enter the battlefield" — and
    /// orders it ahead of every other entry replacement, because the
    /// controller is what the rest of them read: Kismet's "your opponents",
    /// Master Biomancer's "you control", the CR 616.1 chooser itself. Applying
    /// it first is what lets those questions be answered once.
    ///
    /// The `PlayerRef` is resolved when the effect is *applied*, relative to
    /// the effect's controller: `You` is Gather Specimens' "under your
    /// control", `Opponent` is Xantcha's "an opponent of your choice". With one
    /// opponent nothing is chosen (CR 102.2); with more, that player is asked
    /// — and CR 614.12a's "that choice is made before the permanent enters the
    /// battlefield" holds by construction, since the CR 616.1 loop runs ahead
    /// of the performer.
    ///
    /// Rewrites the proposal's `controller` field and nothing else. Not an
    /// `Instead`, because the entry still happens; not an `EnterWith`, because
    /// the controller is a field of the *event* rather than of what the
    /// permanent arrives with, so `EnterMods::merge` has nothing to merge.
    EnterUnderControlOf(PlayerRef),

    /// CR 614.5's doublers and CR 615.10's partial prevention — change the
    /// event's *amount* and nothing else.
    ///
    /// **Not an `Instead` carrying `DealDamage { amount: f(N) }`, and the
    /// reason is composition.** Furnace of Rath's printed ruling — prevent 4
    /// then double the remaining 1, or double to 10 then prevent 4 — is
    /// CR 616.1's ordering choice made observable only because each application
    /// reads the amount the previous one left. An `Amount` arm does that by
    /// construction; an `Instead` template does it only if it happens to read
    /// the right field, and the tree would then have two ways to spell one
    /// piece of arithmetic (`replacement-architecture.md` §9, RD decision 1).
    ///
    /// [`Rewrite::Prevent`] stays separate from the prevention arms here:
    /// "prevent that damage" (CR 615.6, the whole event never happens) and
    /// "prevent 3 of that damage" (a smaller event survives for the next
    /// iteration to see) are different claims. A prevention arm that empties
    /// the event leaves a 0-damage proposal, which `never_happens` drops on the
    /// next iteration (CR 614.7a), so the two routes agree on the board and
    /// differ only in what they assert.
    Amount(AmountRewrite),

    /// CR 614.9's redirection effects — change *where* the damage goes, and
    /// nothing else.
    ///
    /// > 614.9. Some effects replace damage dealt to one battle, creature,
    /// > planeswalker, or player with the same damage dealt to another …
    ///
    /// "The same damage" is the whole of why this rewrites the proposal's
    /// `target` field and leaves `source`, `amount`, `is_combat` and
    /// `unpreventable` alone: Pariah's ruling is that redirected combat damage
    /// is still combat damage, and Kor Chant's that it is still dealt by the
    /// original source.
    ///
    /// **Not an `Instead` carrying a fresh `DealDamage`, and CR 614.9's own
    /// second sentence is the reason.** A destination that has left the
    /// battlefield, stopped being a creature, planeswalker or battle, or a
    /// player who has left the game makes "the effect [do] nothing" — the
    /// event survives untouched and the effect is not spent (decision 7,
    /// `ATOM-614.9-001`). That is a question about an object the proposal does
    /// not mention, asked at the moment the rewrite applies, which a template
    /// evaluated against the event cannot ask. It is deliberately **not**
    /// routed through `gather`'s `applies_to` either: a redirect whose
    /// destination is gone must still be gathered, offered to CR 616.1 and
    /// chosen — it applies and does nothing, rather than vanishing from the
    /// list (`replacement-architecture.md` §9, RD-4's "As landed", decision 3).
    ///
    /// Whole-event only. Harm's Way redirects *part* of one event, which is
    /// one `DealDamage` becoming two and a phase-1 member insertion rather
    /// than a rewrite (`replacement-architecture.md` §11 item 23).
    Retarget(RetargetSpec),
}

/// Where a [`Rewrite::Retarget`] sends the damage.
///
/// **The three arms name three different objects, and two of them contain the
/// word "source", so neither is spelled `ToSource`.** CR 609.7's "source" is
/// the source of the *damage*; a `ReplacementInstance`'s `source` is the
/// object whose ability the effect is. Palisade Giant's "this creature" is the
/// second and Reflect Damage's "that source's controller" is the first, and a
/// pair of adjacent arms called `ToSource` and `ToSourceController` would have
/// read as one question with a `.controller` on the end.
///
/// Each arm ships with a printed customer, which is [`AmountRewrite`]'s rule
/// one level down. A fourth — a destination fixed at resolution, for Harm's
/// Way's "any target" and Divine Deflection's — is **not** here: a card cannot
/// author a target it has not chosen yet, so it would have to be filled from
/// `RegisteredReplacementEffect.targets`, and threading those onto the
/// instance is `codebase-state.md` item 90's work with its own card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetargetSpec {
    /// "…is dealt to **this creature** instead" — the object whose ability
    /// this effect is. Palisade Giant.
    ToEffectSource,
    /// "…is dealt to **enchanted creature** instead" — CR 303.4m's host, read
    /// off `attached_to` at application rather than captured, exactly as
    /// [`AffectedSet::Host`] reads it. Pariah.
    ToHost,
    /// "…is dealt to **that source's controller** instead" — the controller of
    /// the *damage's* source, not of this effect. Reflect Damage.
    ToDamageSourceController,
}

/// CR 107.1a — which way a halving rounds.
///
/// > If a spell or ability could generate a fractional number, the spell or
/// > ability will tell you whether to round up or down.
///
/// So it is **authored, never inferred**, and this type has no `Default`:
/// Ghosts of the Innocent rounds down, Gisela rounds up and Dark Sphere rounds
/// down, each saying so in its own text. The same doctrine as `Duration` on
/// `Primitive::Restrict` — a wrong default here is a rules bug wearing a style
/// choice's clothes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    Up,
    Down,
}

impl Rounding {
    /// Half of `n`, rounded this way.
    pub fn half(self, n: u64) -> u64 {
        match self {
            Rounding::Down => n / 2,
            Rounding::Up => n / 2 + n % 2,
        }
    }
}

/// What a [`Rewrite::Amount`] does to the amount it found.
///
/// **Every arm ships with a printed customer in the PR that lands it**, which
/// is the closed algebra's rule applied one level down: RD-1 has
/// [`Self::Multiplier`] (Furnace of Rath, CR 701.10g), [`Self::Halve`] (Ghosts
/// of the Innocent) and [`Self::PreventHalf`] (Gisela, Blade of Goldnight);
/// RD-2 has [`Self::PreventRemaining`] (Mending Hands, Samite Healer, Samite
/// Censer-Bearer) and the performer for [`Self::PreventUpTo`], whose printed
/// statics — Guardian Seraph, Daunting Defender — are RD-3's because they need
/// a source-side predicate the pattern does not carry yet. `Plus` waits for
/// Torbran in RD-3.
///
/// **Halving and prevention-halving are two arms, not one with a flag**, and
/// CR 615.12 is why. Ghosts of the Innocent "isn't a damage prevention effect"
/// and still halves Excruciator's unpreventable 7 to 3; Gisela prevents none of
/// it. Only the prevention arms report a prevented amount, and only they answer
/// to CR 615.12's consult (RD-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmountRewrite {
    /// "deals double that damage ... instead" — CR 701.10g. Not `Times`, which
    /// reads as a count of occurrences rather than as a factor.
    Multiplier(u64),
    /// "deals half that damage, rounded down, ... instead" — the doubler's
    /// printed inverse. Replaces the amount; prevents nothing.
    Halve(Rounding),
    /// "it deals that much damage **plus 2** instead" — CR 614.1a's additive
    /// modification. Torbran, Thane of Red Fell is the printed customer and
    /// the reason this arm lands in RD-3 rather than beside the doublers.
    ///
    /// **Not the same operation as [`Self::Multiplier`] with a different
    /// number, and CR 616.1's prompt is where the difference shows.**
    /// Multiplication commutes, so a bucket of doublers reaches one outcome in
    /// any order and `pipeline::ordering_cannot_change_outcome` may suppress
    /// the choice; addition beside a multiplier does not (3 → 6 → 8 or
    /// 3 → 5 → 10), which is exactly Torbran's own ruling that "the player
    /// being dealt damage ... chooses an order in which to apply those
    /// effects".
    ///
    /// Prevents nothing (CR 615.1a), and saturates rather than wrapping for
    /// [`Self::Multiplier`]'s reason.
    Plus(u64),
    /// "prevent half that damage, rounded up" — CR 615.10's partial prevention
    /// with CR 107.1a's rounding. The prevented half is what the rule removes;
    /// the rest of the event survives for the next iteration to see.
    PreventHalf(Rounding),
    /// "prevent N of that damage" — CR 615.10's static partial prevention, and
    /// the shape a CR 615.7 count is cut down to at application (see
    /// [`Self::PreventRemaining`]). Prevents `min(N, amount)`.
    PreventUpTo(u64),
    /// "prevent the next N damage" — CR 615.7. The cap is **not here**: it is
    /// the instance's [`Uses::NextDamage`] count, which lives in one place and
    /// is spent as it is used, and this arm names it rather than repeating it.
    ///
    /// The pipeline fills the cap in per application through
    /// [`Self::capped`] — `min(remaining, amount)` against one source, or the
    /// affected player's allocation when several sources deal damage at once
    /// (615.7's own choice). Uncapped, the arithmetic here prevents the whole
    /// amount: "the remaining" with no count is everything, and the pipeline
    /// refuses the pairing that would ever ask it.
    PreventRemaining,

    /// "Damage that would reduce your life total to less than N reduces it to N
    /// instead" — CR 614.1a's modification of the *loss*, clamped so the
    /// affected player's total does not end below the floor.
    ///
    /// **The floor is a life total, which is why it is `i64`.**
    /// `PlayerState::life_total` is signed, because CR 119.6 loses the game at
    /// 0 *or less* and the number below zero is a real board state between one
    /// SBA check and the next. A `u64` here would be a claim about the *scale*
    /// rather than about the cards. All three printed floors are 1 — Ali from
    /// Cairo, Worship, Angel's Grace.
    ///
    /// **It only ever reduces a loss, never reverses one.** From a total
    /// already at or below the floor the clamp takes the whole amount and the
    /// loss becomes 0; "reduces it to N" is a bound on how far the loss may
    /// carry the total, not an instruction to raise it. The arm's type says the
    /// same thing — a `LoseLife`'s amount is a `u64` and there is no negative
    /// loss to produce.
    ///
    /// **Not a prevention effect** ([`Self::prevents_damage`] is `false`), and
    /// the card's own ruling is the reason: *"this effect does not prevent
    /// damage, it prevents the damage from turning into loss of life"*. So
    /// CR 615.12's "damage can't be prevented" has nothing to say to it and
    /// Skullcrack does not turn it off.
    ///
    /// **The one arm whose arithmetic needs the board**, which is why
    /// [`Self::apply`] cannot answer for it: the clamp reads the affected
    /// player's life total *now*. `pipeline::apply_rewrite`'s `LoseLife` leg is
    /// its only evaluator, and that arm's other two legs refuse the pairing.
    LifeFloor(i64),
}

impl AmountRewrite {
    /// How much of `amount` this **prevents** — 0 for the arms that are not
    /// prevention effects (CR 615.1a).
    ///
    /// Separate from [`Self::apply`] because CR 615.5's rider may refer to "the
    /// amount of damage that was prevented", CR 615.7's count is reduced by
    /// exactly that amount, and CR 615.13 triggers on "some or all" of it.
    pub fn prevented(self, amount: u64) -> u64 {
        match self {
            AmountRewrite::Multiplier(_) | AmountRewrite::Halve(_) | AmountRewrite::Plus(_) => 0,
            AmountRewrite::PreventHalf(rounding) => rounding.half(amount),
            AmountRewrite::PreventUpTo(n) => amount.min(n),
            AmountRewrite::PreventRemaining => amount,
            // Ali from Cairo's ruling, in the one place the engine could get it
            // wrong: the clamp is not prevention, so nothing it does feeds
            // CR 615.5's rider, CR 615.7's count or CR 615.13's trigger.
            AmountRewrite::LifeFloor(_) => 0,
        }
    }

    /// The amount the event carries after this arm applies.
    pub fn apply(self, amount: u64) -> u64 {
        match self {
            AmountRewrite::Multiplier(n) => amount.saturating_mul(n),
            AmountRewrite::Halve(rounding) => rounding.half(amount),
            AmountRewrite::Plus(n) => amount.saturating_add(n),
            AmountRewrite::PreventHalf(_)
            | AmountRewrite::PreventUpTo(_)
            | AmountRewrite::PreventRemaining => amount - self.prevented(amount),
            // Unreachable, and loud about it rather than plausible: the clamp
            // needs the affected player's life total, which this signature has
            // no way to read. `pipeline::apply_rewrite`'s `LoseLife` leg is the
            // only evaluator and its sibling legs refuse the pairing, so an
            // arrival here is a leg that forgot to — and a wrong life total
            // with nothing pointing at it is the failure this assertion buys.
            AmountRewrite::LifeFloor(_) => {
                debug_assert!(
                    false,
                    "`AmountRewrite::LifeFloor` is clamped against the affected player's                      life total, which `apply` cannot read. Its only evaluator is                      `pipeline::apply_rewrite`'s `LoseLife` leg."
                );
                amount
            }
        }
    }

    /// This arm with CR 615.7's cap filled in: [`Self::PreventRemaining`]
    /// becomes `PreventUpTo(cap)`, and every other arm is itself.
    ///
    /// `cap` is what this application may prevent — the instance's remaining
    /// count against one source, or this source's share of the affected
    /// player's allocation when several deal damage at once.
    pub fn capped(self, cap: u64) -> AmountRewrite {
        match self {
            AmountRewrite::PreventRemaining => AmountRewrite::PreventUpTo(cap),
            other => other,
        }
    }

    /// Does this operation prevent damage — CR 615.1a's "uses the word
    /// 'prevent'", asked of one arm?
    ///
    /// **Not `is_prevention`, which is [`ReplacementDef`]'s.** That one asks
    /// whether an *effect* is a prevention effect, which is the CR's noun and
    /// needs the pattern as well as the rewrite: a `Prevent` on a destruction
    /// is regeneration, not prevention. This one is the arm-shaped half it
    /// delegates to. [`Self::prevented`] answers *how much*.
    pub fn prevents_damage(self) -> bool {
        match self {
            AmountRewrite::Multiplier(_) | AmountRewrite::Halve(_) | AmountRewrite::Plus(_) => false,
            AmountRewrite::PreventHalf(_)
            | AmountRewrite::PreventUpTo(_)
            | AmountRewrite::PreventRemaining => true,
            AmountRewrite::LifeFloor(_) => false,
        }
    }
}

/// CR 614.13's "other objects that will also change zones" — what one
/// [`Rewrite::EnterAfterMoving`] chooses, where it sends them, and what the
/// entering permanent gets for each.
///
/// **Where per-mechanic variety lives, so [`Rewrite`] does not grow again.**
/// Devour N is `(Battlefield, creatures you control) → Graveyard, Sacrificed`
/// with `per_chosen: Some((PlusOnePlusOne, N))`; Sutured Ghoul is
/// `(Graveyard, creature cards) → Exile, Exiled` with `per_chosen: None`. The
/// ~20 "as ~ enters, choose" cards with a board consequence are the same five
/// fields.
///
/// **`from` is a zone and not a `SelectionFilter`**, because
/// `oracle::legality::enumerate_legal_selections` enumerates the battlefield,
/// the stack and players and has no graveyard leaf. Adding one is the job
/// `codebase-state.md` item 46 sizes, and it is not this arm's.
#[derive(Debug, Clone, PartialEq)]
pub struct AuxiliaryMove {
    /// Which zone the choosable objects are in. Only the choosing player's own
    /// objects are candidates — CR 701.21a's "its controller moves it" for a
    /// sacrifice, and every printed graveyard variant says "your graveyard".
    pub from: Zone,

    /// What a candidate must be. Read off the layer walk on the battlefield and
    /// off the card anywhere else, which is the same split
    /// `EventPattern::ZoneChange`'s `object` filter already makes.
    ///
    /// **`ObjectFilter` is the wrong name for what this does** and has been
    /// since RB: CR 110.1 makes a permanent a card *on the battlefield*, and
    /// this matches creature cards in a graveyard. The type is right; the name
    /// is two phases stale, and the rename is `codebase-state.md` item 64 —
    /// ~120 mechanical call sites, no behaviour, so it wants a PR of its own.
    pub filter: ObjectFilter,

    /// Where the chosen objects go, and why. The `cause` is what separates
    /// devour's sacrifice from an exile, and 278 cards care (CR 701.21).
    pub to: Zone,
    pub cause: ZoneChangeCause,

    /// The upper bound on how many may be chosen. `None` is "any number", which
    /// is what both printed shapes say; a bounded form would be `Some(n)`.
    ///
    /// The lower bound is always zero. "Any number" includes none, so the
    /// choice to decline lives in the count rather than in
    /// [`ReplacementDef::optional`] — marking devour optional would ask the
    /// player twice and let a decline consume CR 614.5's one opportunity for a
    /// reason the card does not have.
    pub up_to: Option<u32>,

    /// CR 122.6a — what the entering permanent is given per object chosen.
    /// Devour 3 is `Some((PlusOnePlusOne, 3))`; Sutured Ghoul is `None`,
    /// because its power and toughness come from a *linked* ability (CR 614.14,
    /// 607) and not from the entry.
    ///
    /// **A constant per object, which Thromok the Insatiable is not**: "devour
    /// X, where X is the number of creatures devoured this way" makes the
    /// multiplier the count itself, so X creatures give X² counters. One more
    /// shape here and no new [`Rewrite`] arm — `codebase-state.md` item 63.
    pub per_chosen: Option<(CounterType, u32)>,
}

/// What an entry replacement *adds*, before its amounts are evaluated
/// (CR 614.1c/d).
///
/// **The half of [`EnterMods`] that cannot be a number yet.** Master
/// Biomancer's "additional +1/+1 counters equal to this creature's power" is
/// known only when the effect is applied, and it is read off the *source* —
/// so the definition carries an [`AmountExpr`] and
/// `replacement::evaluate_enter_amount` turns it into an [`EnterMods`] inside
/// the CR 616.1 loop.
///
/// Split out in RC-5. Until then one type did both jobs, which §3.2 recorded as
/// a virtue and which held exactly as long as every amount was a literal.
#[derive(Debug, Clone, PartialEq)]
pub struct EnterModsTemplate {
    /// CR 110.5b — the permanent enters tapped. A status, so no amount.
    pub tapped: bool,

    /// CR 122.6a — the counters, and how many of each.
    pub counters: Vec<(CounterType, AmountExpr)>,
}

impl EnterModsTemplate {
    /// CR 110.5b — "this permanent enters tapped".
    pub fn tapped() -> Self {
        EnterModsTemplate { tapped: true, counters: Vec::new() }
    }

    /// CR 122.6a — "this permanent enters with `n` `counter` counters on it".
    pub fn with_counters(counter: CounterType, n: u32) -> Self {
        EnterModsTemplate {
            tapped: false,
            counters: vec![(counter, AmountExpr::Fixed(n as u64))],
        }
    }

    /// CR 122.6a with an amount the board decides — Master Biomancer.
    pub fn with_counter_amount(counter: CounterType, amount: AmountExpr) -> Self {
        EnterModsTemplate { tapped: false, counters: vec![(counter, amount)] }
    }

    /// Does every amount here read a constant?
    ///
    /// The premise `pipeline::ordering_cannot_change_outcome` grew for RC-5:
    /// an amount that reads the CR 614.12 frame changes with what already
    /// applied, so two such applications do not commute and CR 616.1's
    /// ordering prompt is real. `codebase-state.md` item 47 carries the
    /// expiry conditions this is one of.
    pub fn is_fixed(&self) -> bool {
        self.counters.iter().all(|(_, a)| matches!(a, AmountExpr::Fixed(_)))
    }
}

/// How a permanent enters the battlefield, when something modified it
/// (CR 614.1c/d).
///
/// The payload of `GameAction::EnterBattlefield`, and what
/// [`EnterModsTemplate`] evaluates to: what one effect *adds* and what the
/// permanent will *end up with* are the same shape once the amounts are
/// numbers, which is what makes [`Self::merge`] the whole of CR 616.1f's
/// accumulation.
///
/// **Two fields, and each is a rule rather than a convenience.** CR 110.5b —
/// "permanents enter the battlefield untapped … unless a spell or ability says
/// otherwise" — makes `tapped` the exception to a *default*, so `false` is the
/// rule speaking rather than a missing value. CR 122.6a covers `counters`:
/// "an object that's given counters as it enters the battlefield".
///
/// # The other two statuses, and what adding one would actually cost
///
/// CR 110.5b names four: tapped, flipped, face down, phased in. Phasing is not
/// something a permanent can enter with (CR 702.26). The other two are absent,
/// and it is worth being precise about why, because "face down" looks like a
/// third `bool` and is not one.
///
/// **A new status is a field here, not an arm anywhere.** That is the growth
/// contract working: [`Rewrite`] does not grow, [`EventPattern`] does not grow,
/// and no reader outside the performer learns a new shape. So the *plumbing*
/// really is one line.
///
/// **What is not one line is what face down means.** CR 707.2 makes a face-down
/// permanent a 2/2 colorless creature with no name, no mana cost, no creature
/// types and no abilities — a change to its **copiable values**, which is
/// Layer 1a. A `face_down: true` that only set a flag would leave every layer
/// query answering off the printed card, so the field wants Layer 1 (Phase CV)
/// underneath it and CR 614.12's frame (RC-4) beside it, since the entry is
/// changing the very characteristics the frame is asked about.
///
/// **The printed population says the same thing from the other side.** Nothing
/// prints "permanents enter the battlefield face down" as an effect over
/// someone else's permanents (Scryfall, 2026-09-01). Face-down entry is morph,
/// manifest, disguise and cloak, and those are *how the object gets there* —
/// CR 701.34a's "put it onto the battlefield face down as a 2/2 creature card"
/// is an instruction the mover carries, not a replacement effect watching for
/// an entry. Which is the shape this type already has: `EnterMods` is the
/// payload of both [`Rewrite::EnterWith`] **and** the proposal's seed
/// (`GameState::default_enter_mods`), so manifest would set the field at the
/// proposal, exactly the way CR 306.5b's loyalty does today.
///
/// A *hypothetical* "creatures your opponents control enter face down" would
/// additionally need `AffectedSet::Filter` to reach an entering permanent,
/// which is Phase RC-3 — the same gate that stops Root Maze and Kismet.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EnterMods {
    /// CR 110.5b — the permanent enters tapped.
    pub tapped: bool,

    /// CR 122.6a — the counters the permanent is given as it enters.
    ///
    /// Coalesced by kind at [`Self::merge`], so the performer puts each kind on
    /// once and CR 613.7c allocates one timestamp per kind. Insertion order,
    /// which is a `Vec` rather than a `HashMap` for the reason every ordered
    /// collection in this engine is one: a `HashMap` walk is not reproducible
    /// across processes, and this list reaches `add_counters`.
    pub counters: Vec<(CounterType, u32)>,
}

impl EnterMods {
    /// Nothing modifies how this permanent enters — CR 110.5b's default.
    pub const NONE: EnterMods = EnterMods { tapped: false, counters: Vec::new() };

    /// CR 110.5b — "this permanent enters tapped".
    pub fn tapped() -> Self {
        EnterMods { tapped: true, counters: Vec::new() }
    }

    /// CR 122.6a — "this permanent enters with `n` `counter` counters on it".
    pub fn with_counters(counter: CounterType, n: u32) -> Self {
        EnterMods { tapped: false, counters: vec![(counter, n)] }
    }

    /// Is this the CR 110.5b default — nothing to apply?
    pub fn is_none(&self) -> bool {
        !self.tapped && self.counters.is_empty()
    }

    /// Fold `other`'s modifications into this one — CR 616.1f's accumulation.
    ///
    /// **Tapped is a status, counters are a quantity, and the CR treats them
    /// differently.** CR 110.5b gives a permanent one tapped/untapped value, so
    /// two effects that both say "enters tapped" leave it tapped once; CR 122.6a
    /// is about counters being *put on* it, so two effects that each give it a
    /// counter give it two. `|=` and addition, and neither is a choice this
    /// engine is making.
    pub fn merge(&mut self, other: &EnterMods) {
        self.tapped |= other.tapped;
        for (counter, n) in &other.counters {
            match self.counters.iter_mut().find(|(c, _)| c == counter) {
                // Plain addition, matching `PermanentState::add_counters`,
                // which is where this number ends up. A saturating add here
                // would be the only place in the engine with a different
                // overflow story, and clamping at `u32::MAX` is not a rules
                // answer — it is a width this type has no business choosing.
                Some((_, existing)) => *existing += *n,
                None => self.counters.push((*counter, *n)),
            }
        }
    }
}

/// The substitute event an [`Rewrite::Instead`] produces.
///
/// A **template**, not a constant: several cards build the replacement out of
/// the event they are replacing (Chatterfang's "those tokens *plus that many*
/// Squirrels", Rain of Gore's "loses *that much* life instead"), so its fields
/// may reference the incoming event's. Every arm here is written that way —
/// each one names what it takes from the event and what it overrides.
///
/// **Grows per card, and that is the design.** It is the unbounded arm's
/// payload; the bound is that a template can only produce a `GameAction` the
/// engine already proposes.
#[derive(Debug, Clone, PartialEq)]
pub enum GameActionTemplate {
    /// Send the event's object somewhere else instead, keeping its `from`.
    ///
    /// Three customers in RB: CR 122.1h's finality counter ("If this permanent
    /// would be put into a graveyard from the battlefield, exile it instead"),
    /// CR 903.9b's commander redirection, and Kalitas, Traitor of Ghet's exile.
    ZoneChangeTo { to: Zone, cause: ZoneChangeCause },

    /// Remove counters from the *affected* object instead.
    ///
    /// Two customers in RB, both spelled out verbatim by the CR: 122.1c's
    /// "If this permanent would be destroyed as the result of an effect,
    /// instead remove a shield counter from it" and 122.1d's "If a permanent
    /// with a stun counter on it would become untapped, instead remove a stun
    /// counter from it".
    RemoveCountersFromAffected { counter: CounterType, n: u32 },

    /// Draw instead (CR 121.2a) — the substitute for an individual draw, and
    /// it is the **instruction**, not another individual draw.
    ///
    /// Two customers, and they differ only in `player`. Thought Reflection's
    /// "draw two cards instead" is `{ n: 2, player: None }` — the draw stays
    /// with the player who would have drawn. Notion Thief's "instead that
    /// player skips that draw and you draw a card" is
    /// `{ n: 1, player: Some(PlayerRef::You) }`: **the same event with a new
    /// subject**, which is CR 614.5's "modified events that may replace that
    /// event" and the only encoding its own ruling admits. As a `Prevent` with
    /// a rider the Thief's draw would be a fresh proposal with a fresh applied
    /// set, and two Thieves would trade one draw forever instead of each
    /// applying once (`replacement-architecture.md` §3.2d, corrected
    /// 2026-09-11).
    ///
    /// `None` on `player` is the affected player rather than an absent value:
    /// the field exists because one of the two customers moves the draw, and
    /// the other one saying so explicitly would be a `PlayerRef` the pipeline
    /// resolves to the same answer it already has.
    DrawCards { n: u64, player: Option<PlayerRef> },

    /// Gain life instead — CR 614.1a, from an event of any kind.
    ///
    /// One customer, Words of Worship ("the next time you would draw a card
    /// this turn, you gain 5 life instead"), and it is a *kind-changing*
    /// substitution: the pattern is a draw and the substitute is a life gain.
    /// The affected player gains; nothing printed moves a substituted gain to
    /// somebody else, so there is no `player` field here and the day one is
    /// printed it arrives as [`Self::DrawCards`]'s already is.
    GainLife { amount: TemplateAmount },

    /// Lose life instead — CR 614.1a.
    ///
    /// One customer, Tainted Remedy ("if an opponent would gain life, that
    /// player loses that much life instead"), and it is the arm
    /// [`TemplateAmount::ReplacedAmount`] exists for: "that much" is the
    /// replaced event's own number.
    ///
    /// `cause` is [`LifeLossCause::Effect`] and is not a field: a substituted
    /// loss is produced by the replacement effect, which is an effect, and no
    /// printed substitution claims to be damage or a payment. Ali from Cairo
    /// reading `Some(Damage)` is what makes that distinction load-bearing
    /// rather than cosmetic.
    LoseLife { amount: TemplateAmount },
}

/// How a life template gets its number.
///
/// Two arms, one printed customer each, and the pair is the whole of what
/// CR 614.1a's life substitutions say: Words of Worship names a constant and
/// Tainted Remedy names the event's own amount.
///
/// **Not [`AmountExpr`].** That type is a resolving effect's arithmetic over
/// the board — "X", "equal to this creature's power" — evaluated against a
/// `ResolutionContext` a replacement effect does not have. This one is
/// evaluated against the *event*, which is the only thing a rewrite may read,
/// and its two arms are the two things an event can supply. The day a card
/// prints "gain life equal to the number of creatures you control instead",
/// the arm it wants is an `AmountExpr` leg here rather than a third constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateAmount {
    /// A number printed on the card — Words of Worship's 5.
    Fixed(u64),
    /// CR 615.5's "that much", read off the replaced event through
    /// `pipeline::event_amount` — Tainted Remedy's.
    ///
    /// An event with no amount is a card-authoring error rather than a rules
    /// corner, and the pipeline reports it the way every other half-disagreeing
    /// `ReplacementDef` is reported.
    ReplacedAmount,
}

/// CR 616.1a–e — the steps of the rule's choice ladder, in its own order.
///
/// `must_choose_among` returns the first non-empty step's candidates and only
/// those; [`Self::Other`] is 616.1e's fallthrough, "any of the applicable
/// replacement and/or prevention effects may be chosen".
///
/// All five arms ship in Phase RB even though only `Other` has a producer,
/// because the *ordering* is what item 3 implements and a step that does not
/// exist cannot be ordered. `ControlChanging` gained its producer in RC-4
/// ([`Rewrite::EnterUnderControlOf`]); `SelfReplacement` gets one with the
/// first CR 614.15 card and `CopyOnEnter` with Phase CV-2's copy spine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReplacementClass {
    /// CR 616.1a / 614.15.
    SelfReplacement,
    /// CR 616.1b — modifies under whose control an object enters.
    ControlChanging,
    /// CR 616.1c — causes an object to enter as a copy of another.
    CopyOnEnter,
    /// CR 616.1d — causes a card to enter with its back face up.
    BackFaceUp,
    /// CR 616.1e — free choice.
    Other,
}

impl ReplacementClass {
    /// The CR 616.1 step a rewrite belongs to, read off the rewrite.
    ///
    /// Derived rather than authored. CR 616.1b and 616.1c name their classes
    /// by what the effect *does*, so a field a card could set is a field a
    /// card could forget, and a control-changing effect filed under `Other`
    /// would be chosen in the wrong order silently. `SelfReplacement` is the
    /// one class no rewrite implies — CR 614.15 is about where the effect
    /// *came from* — and it lands with `ActionContext::resolution`'s field
    /// (`replacement-architecture.md` §11 item 3). `BackFaceUp` waits on
    /// transform.
    pub fn from_rewrite(rewrite: &Rewrite) -> Self {
        match rewrite {
            Rewrite::EnterUnderControlOf(_) => ReplacementClass::ControlChanging,
            // CR 616.1's ladder has no step for redirection — 616.1b is
            // about *entering* under someone's control, and damage does not
            // enter anything — so a redirect is a free choice like a doubler.
            Rewrite::Prevent
            | Rewrite::Instead(_)
            | Rewrite::EnterWith(_)
            | Rewrite::EnterAfterMoving(_)
            | Rewrite::Retarget(_)
            | Rewrite::Amount(_) => ReplacementClass::Other,
        }
    }
}

/// How many times a replacement effect can fire.
///
/// **There is no `CounterBacked`.** CR 122.1c/d make the counter removal the
/// substituted event or the CR 615.5 rider, never bookkeeping, so a use that
/// removed one would write `PermanentState.counters` from inside
/// `consume_use` — off the chokepoint. Existence is asked at gather time, which
/// is where CR 614.4 wants it asked.
///
/// **A use is spent by what an application did, not by being chosen**
/// (`replacement-architecture.md` §9, RD decision 7). CR 609.7b: "if for any
/// reason the shield prevents no damage or replaces no damage, the shield
/// isn't used up" — so `Once` is spent only when the rewrite took effect, and
/// [`Self::NextDamage`] by exactly the amount prevented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Uses {
    /// CR 614.1a static abilities, 615.10, 701.19b — every time, forever.
    Static,
    /// CR 701.19a's regeneration shield, CR 615.8's "next time [source] would
    /// deal damage" — one application, then the effect is gone.
    Once,
    /// CR 615.7 — "prevent the next N damage": the amount still to be
    /// prevented, decremented in place as damage is prevented and removed at
    /// zero.
    ///
    /// Named for the rule's own phrase because it **counts damage and never
    /// uses** — 615.7's last sentence is "such effects count only the amount of
    /// damage; the number of events or sources dealing it doesn't matter", and
    /// a first name, `DamagePoints`, could be read as either. The word "shield"
    /// is deliberately not in the name: [`ReplacementDef::affected`]'s docs
    /// reserve it for CR 122.1c's counter and CR 701.19a's regeneration, and
    /// §9's glossary says why this is a third thing.
    ///
    /// Pairs with [`AmountRewrite::PreventRemaining`] and nothing else, in both
    /// directions — the pipeline refuses either without the other, since a
    /// count of damage on an effect that prevents no damage has nothing to
    /// count down. Lives only on a registry row: a static ability and a
    /// counter are re-derived on every gather, so "spent" would have nowhere
    /// to be recorded (the same argument `consume_use` makes for `Once`).
    NextDamage(u64),
}

impl ReplacementDef {
    /// The common shape: a mandatory, unremarkable, always-on replacement.
    ///
    /// Named constructors rather than a `Default`, because `class` and
    /// `exempt_from_614_5` are the two fields where a wrong default is a rules
    /// bug rather than a style choice.
    pub fn new(pattern: EventPattern, affected: AffectedSet, rewrite: Rewrite) -> Self {
        let class = ReplacementClass::from_rewrite(&rewrite);
        ReplacementDef {
            pattern,
            affected,
            affected_players: PlayerSet::Nobody,
            rewrite,
            then: None,
            class,
            uses: Uses::Static,
            is_regeneration: false,
            exempt_from_614_5: false,
            optional: false,
        }
    }

    /// Builder: also apply to these players (CR 614.1's other half).
    ///
    /// A builder rather than a fourth argument to [`Self::new`]: every effect
    /// written before Phase RD names no player, so `PlayerSet::Nobody` is the
    /// honest default and a card that wants one says so. Furnace of Rath's
    /// "a permanent **or** player" is [`Self::new`] plus this.
    ///
    /// **One mechanism, not two.** A `for_players` *constructor* shipped
    /// alongside this for one commit and was removed on review: two functions
    /// whose names differ by an inflection, one a constructor and one a
    /// builder, is a coin flip at every call site. An effect about players and
    /// no object writes `AffectedSet::NO_OBJECTS` for its object half, which
    /// names the empty set where the call site can see it.
    pub fn affecting_players(mut self, players: PlayerSet) -> Self {
        self.affected_players = players;
        self
    }

    /// Builder: attach the CR 615.5 rider.
    pub fn with_then(mut self, then: Effect) -> Self {
        self.then = Some(then);
        self
    }

    /// Builder: one application, then the effect is gone (CR 701.19a).
    pub fn once(mut self) -> Self {
        self.uses = Uses::Once;
        self
    }

    /// Builder: "prevent the next `n` damage" (CR 615.7). Pairs with
    /// [`AmountRewrite::PreventRemaining`], which is where the pipeline reads
    /// the count from.
    pub fn next_damage(mut self, n: u64) -> Self {
        self.uses = Uses::NextDamage(n);
        self
    }

    /// CR 615.1a — is this a prevention effect?
    ///
    /// > 615.1a Effects that use the word "prevent" are prevention effects.
    ///
    /// **Derived, never authored**, on the CDA and CR 614.15 argument
    /// (`replacement-architecture.md` §11 items 12 and 25): the rule makes it a
    /// fact about the effect's *text*, which the def carries as its rewrite and
    /// its pattern, so a card cannot forget to set it. A `Prevent` on a
    /// destruction is regeneration, not prevention — 615.1 is about damage,
    /// and CR 701.19c treats the two differently, which is why
    /// [`Self::is_regeneration`] keeps its own authored bit.
    ///
    /// Two readers: RD-2's pairing check — a [`Uses::NextDamage`] count on an
    /// effect that prevents no damage is an authoring error — and RD-4's
    /// CR 615.12 consult, where an unpreventable event lets a prevention
    /// effect apply and prevent nothing.
    pub fn is_prevention(&self) -> bool {
        matches!(self.pattern, EventPattern::DealDamage { .. })
            && match &self.rewrite {
                Rewrite::Prevent => true,
                Rewrite::Amount(arm) => arm.prevents_damage(),
                // CR 614.9's redirection is not prevention: the damage is
                // still dealt, to something else. So CR 615.12 has nothing to
                // say to it — unpreventable damage is redirected normally.
                Rewrite::Retarget(_)
                | Rewrite::Instead(_)
                | Rewrite::EnterWith(_)
                | Rewrite::EnterAfterMoving(_)
                | Rewrite::EnterUnderControlOf(_) => false,
            }
    }

    /// Builder: mark this as a CR 701.19 regeneration shield, which CR 701.19c
    /// can withhold.
    pub fn regeneration(mut self) -> Self {
        self.is_regeneration = true;
        self
    }

    /// Builder: "you **may** ... instead" (CR 614.1a).
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

/// CR 701.19a's rider, verbatim.
///
/// > ... instead remove all damage marked on it and its controller taps it. If
/// > it's an attacking or blocking creature, remove it from combat.
///
/// In that order, because the order is observable in the event log once
/// triggers land.
///
/// **Shared by both halves of CR 701.19 on purpose.** 701.19a (a resolving
/// spell or ability) and 701.19b (a static ability) differ in exactly one
/// thing — `Uses::Once` versus `Uses::Static` — and the rule states the same
/// replacement text for both. A card with static regeneration that wrote its
/// own rider would be a second copy of that text, free to drift; a card that
/// wrote *no* rider would be a bare prevention wearing regeneration's name,
/// which is the mistake this function exists to make impossible.
///
/// `EffectRecipient::Target` names the shielded permanent: a rider resolves
/// against a `ResolutionContext` whose single resolved target is the event's
/// subject.
pub fn regeneration_rider() -> Effect {
    use crate::types::effects::{EffectRecipient, ObjectFilter, Primitive, SelectionFilter,
                                TargetCount};
    let it = || {
        EffectRecipient::Target(
            SelectionFilter::Permanent(ObjectFilter::All),
            TargetCount::Exactly(1),
        )
    };
    Effect::Sequence(vec![
        Effect::Atom(Primitive::RemoveAllDamage, it()),
        Effect::Atom(Primitive::Tap, it()),
        Effect::Atom(Primitive::RemoveFromCombat, it()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::effects::PlayerSet;

    // CR 107.1a puts the direction on the card, so the two directions are two
    // answers to one question and the odd amounts are the whole test.
    #[test]
    fn rounding_is_the_cards_and_only_odd_amounts_can_tell() {
        for n in [0, 2, 4, 100] {
            assert_eq!(Rounding::Down.half(n), n / 2);
            assert_eq!(Rounding::Up.half(n), n / 2);
        }
        assert_eq!(Rounding::Down.half(1), 0);
        assert_eq!(Rounding::Up.half(1), 1);
        assert_eq!(Rounding::Down.half(5), 2);
        assert_eq!(Rounding::Up.half(5), 3);
    }

    // Ghosts of the Innocent's ruling, verbatim: "with three on the
    // battlefield, 14 damage becomes 7, then 3, then finally 1".
    #[test]
    fn three_halvings_take_fourteen_to_one() {
        let ghosts = AmountRewrite::Halve(Rounding::Down);
        assert_eq!(ghosts.apply(14), 7);
        assert_eq!(ghosts.apply(7), 3);
        assert_eq!(ghosts.apply(3), 1);
    }

    // "Half of 1 rounded down is 0. A source that would deal 1 damage won't
    // deal damage at all" — the 0 is what `never_happens` drops (CR 614.7a).
    #[test]
    fn halving_one_leaves_a_zero_for_cr_614_7a() {
        assert_eq!(AmountRewrite::Halve(Rounding::Down).apply(1), 0);
    }

    // CR 614.5's own example, as arithmetic: two doublers, not one applied
    // twice. The applied set is the pipeline's job; this is the factor.
    #[test]
    fn a_doubler_is_a_factor_and_composes() {
        let furnace = AmountRewrite::Multiplier(2);
        assert_eq!(furnace.apply(furnace.apply(2)), 8);
    }

    // Gisela prevents half rounded *up*, so the event keeps the smaller half —
    // and the prevented amount is what RD-2's rider and shield will read.
    #[test]
    fn prevent_half_reports_what_it_prevented_and_keeps_the_rest() {
        let gisela = AmountRewrite::PreventHalf(Rounding::Up);
        assert_eq!(gisela.prevented(5), 3);
        assert_eq!(gisela.apply(5), 2);
        // Dark Sphere's direction, from the other side (RD-3): 5 becomes 3.
        let sphere = AmountRewrite::PreventHalf(Rounding::Down);
        assert_eq!(sphere.prevented(5), 2);
        assert_eq!(sphere.apply(5), 3);
    }

    // CR 615.1a defines a prevention effect by the word "prevent", so the
    // non-prevention arms must report 0 rather than "not applicable".
    #[test]
    fn only_the_prevention_arms_prevent_anything() {
        assert_eq!(AmountRewrite::Multiplier(2).prevented(7), 0);
        assert_eq!(AmountRewrite::Halve(Rounding::Down).prevented(7), 0);
    }

    // CR 615.10 — "prevent 1 of that damage" takes 1 from any amount that has
    // one to give, and all of a smaller one.
    #[test]
    fn prevent_up_to_takes_the_smaller_of_the_two() {
        let seraph = AmountRewrite::PreventUpTo(1);
        assert_eq!(seraph.prevented(4), 1);
        assert_eq!(seraph.apply(4), 3);
        assert_eq!(AmountRewrite::PreventUpTo(3).prevented(2), 2);
        assert_eq!(AmountRewrite::PreventUpTo(3).apply(2), 0);
    }

    // CR 615.7 — the cap is the instance's count, filled in per application.
    // Uncapped, "the remaining" is everything; capped at 3 it is `PreventUpTo(3)`,
    // and the arms that carry their own number are untouched by `capped`.
    #[test]
    fn prevent_remaining_reads_its_cap_from_the_instance() {
        assert_eq!(AmountRewrite::PreventRemaining.prevented(5), 5);
        assert_eq!(AmountRewrite::PreventRemaining.capped(3), AmountRewrite::PreventUpTo(3));
        assert_eq!(AmountRewrite::PreventRemaining.capped(3).prevented(5), 3);
        assert_eq!(AmountRewrite::PreventRemaining.capped(3).apply(5), 2);
        assert_eq!(AmountRewrite::PreventRemaining.capped(9).apply(5), 0);
        assert_eq!(AmountRewrite::Multiplier(2).capped(3), AmountRewrite::Multiplier(2));
        assert_eq!(AmountRewrite::PreventUpTo(1).capped(3), AmountRewrite::PreventUpTo(1));
    }

    // > 615.1a Effects that use the word "prevent" are prevention effects.
    //
    // **The negative half is the load-bearing one**, and it is why this is a
    // test rather than a restatement: asserting that a def written to prevent
    // damage is a prevention effect is close to tautological, since the def is
    // hand-written three lines up. *Regeneration* is the one that is not —
    // a `Prevent` that is **not** a prevention effect, because CR 615.1 is
    // about damage and regeneration's pattern is a destruction. Get that wrong
    // and CR 701.19c's "can't be regenerated" and CR 615.12's "damage can't be
    // prevented" (RD-4) begin answering for each other, since both consult a
    // `ReplacementKindFilter`. A doubler is the other negative: same pattern,
    // no "prevent" in its text.
    //
    // COVERS-PARTIAL: BOUNDARY-DEF-615.1a-001 — the atom's out-of-set member is
    // a *triggered ability* ("whenever damage is dealt … you gain that much
    // life"), which no type here can express until critical-path item 6; the
    // two negatives below are the nearest members it can. The in-set member is
    // built whole.
    #[test]
    fn regeneration_is_a_prevent_that_is_not_a_prevention_effect() {
        let next_three = ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::NO_OBJECTS,
            Rewrite::Amount(AmountRewrite::PreventRemaining),
        )
        .next_damage(3);
        assert!(next_three.is_prevention());
        assert_eq!(next_three.uses, Uses::NextDamage(3));

        let that_damage = ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::SourceOnly,
            Rewrite::Prevent,
        );
        assert!(that_damage.is_prevention());
        assert!(ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::SourceOnly,
            Rewrite::Amount(AmountRewrite::PreventHalf(Rounding::Up)),
        )
        .is_prevention());

        let doubler = ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::SourceOnly,
            Rewrite::Amount(AmountRewrite::Multiplier(2)),
        );
        assert!(!doubler.is_prevention());

        let regeneration = ReplacementDef::new(
            EventPattern::Destroy { source: None },
            AffectedSet::SourceOnly,
            Rewrite::Prevent,
        )
        .once()
        .regeneration();
        assert!(!regeneration.is_prevention(), "CR 615.1 is about damage");
    }

    // CR 102.1 — "opponent" is every other player, and CR 109.5 resolves "you"
    // against the effect's current controller. Gisela's two halves are these
    // two sets, and a three-player board is where they stop agreeing with a
    // two-player shortcut.
    #[test]
    fn player_sets_resolve_against_the_effects_controller() {
        let you = PlayerSet::You;
        let opponents = PlayerSet::Opponents;
        assert!(you.contains(1, 1));
        assert!(!you.contains(1, 0));
        assert!(!opponents.contains(1, 1));
        assert!(opponents.contains(1, 0));
        assert!(opponents.contains(1, 2));
        assert!(PlayerSet::Everyone.contains(1, 1));
        assert!(PlayerSet::Everyone.contains(1, 2));
        // What every effect written before Phase RD says.
        assert!(!PlayerSet::Nobody.contains(0, 0));
        // A resolution's captured set ignores the controller entirely.
        assert!(PlayerSet::Fixed(vec![2]).contains(0, 2));
        assert!(!PlayerSet::Fixed(vec![2]).contains(0, 0));
    }

    // CR 614.1a's additive arm beside the two that already read an amount.
    // Torbran's "plus 2" prevents nothing (CR 615.1a has no "prevent" in its
    // text), replaces the amount, and saturates at the ceiling rather than
    // wrapping to zero the way `Multiplier` does.
    #[test]
    fn plus_adds_prevents_nothing_and_saturates() {
        let plus = AmountRewrite::Plus(2);
        assert_eq!(plus.apply(3), 5);
        assert_eq!(plus.apply(0), 2, "614.7a drops a 0 event before this is asked");
        assert_eq!(plus.prevented(3), 0);
        assert!(!plus.prevents_damage());
        assert_eq!(AmountRewrite::Plus(u64::MAX).apply(5), u64::MAX);

        let torbran = ReplacementDef::new(
            EventPattern::DealDamage {
                source: Some(SourcePattern::matching(ObjectFilter::ByColor(
                    crate::types::colors::Color::Red,
                ))),
                combat: None,
            },
            AffectedSet::NO_OBJECTS,
            Rewrite::Amount(AmountRewrite::Plus(2)),
        );
        assert!(!torbran.is_prevention(), "adding damage is not preventing it");
    }

    // A `SourcePattern` with neither half is what a card writes when the
    // resolution is going to fill the object in (CR 609.7a). Both halves are
    // `None` and the pattern is still `Some`, which is what tells
    // `PatternFill::ChosenDamageSource` there is a field to write into.
    #[test]
    fn a_chosen_source_pattern_starts_empty() {
        assert_eq!(
            SourcePattern::chosen(),
            SourcePattern { object: None, filter: None }
        );
        assert_eq!(
            SourcePattern::matching(ObjectFilter::All),
            SourcePattern { object: None, filter: Some(ObjectFilter::All) }
        );
    }

    // A `ReplacementDef` written the way every pre-RD card writes one names no
    // player, so RD-1's second field cannot change any of their answers.
    #[test]
    fn new_defs_name_no_player() {
        let def = ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::SourceOnly,
            Rewrite::Prevent,
        );
        assert_eq!(def.affected_players, PlayerSet::Nobody);
    }
}
