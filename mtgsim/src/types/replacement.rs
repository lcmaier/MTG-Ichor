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
//!   should arrive with the rule number that says so. It ships **two** —
//!   `Prevent` and `Instead` — not §3.2's five: an arm the pipeline cannot
//!   apply is worse than a missing one.
//!
//! Per-mechanic variety goes in [`ReplacementDef::then`], which is the existing
//! `Effect` tree — no new vocabulary at all.

use crate::types::effects::{AffectedSet, CounterType, Effect, PermanentFilter};
use crate::types::zones::{DestructionSource, Zone, ZoneChangeCause};

/// One replacement or prevention effect.
///
/// CR 614.1: replacement effects "act like shields around whatever they're
/// affecting". [`Self::affected`] is that shield's boundary and
/// [`Self::pattern`] is what it watches for.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplacementDef {
    /// Which proposed events this watches (CR 614.1, 615.1).
    pub pattern: EventPattern,

    /// Which objects it shields.
    ///
    /// Reuses the layer system's `AffectedSet`, and the reuse is load-bearing:
    /// `SourceOnly` vs. `Filter` is exactly CR 614.12's "affects only that
    /// permanent (as opposed to a general subset of permanents that includes
    /// it)". If a future refactor collapses those variants, 614.12 breaks
    /// silently (`replacement-architecture.md` §11 item 2).
    pub affected: AffectedSet,

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
    /// affected object, so `EffectRecipient::Target`/`Choose` name the shielded
    /// permanent and `EffectRecipient::Controller` names its controller.
    pub then: Option<Effect>,

    /// CR 616.1a–d — which forced-choice bucket this falls in.
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
/// Data rather than a closure for the same reason `PermanentFilter` is:
/// closures cannot be compared, cloned cheaply, or inspected by a loop
/// detector.
///
/// # The growth contract
///
/// **Exactly one arm per `GameAction` variant, and it grows on no other axis.**
/// Within an arm, constraints on the event's fields reuse existing vocabulary —
/// `PermanentFilter`, `ZoneChangeCause`, `CardType` — rather than inventing
/// per-mechanic predicates. A change that adds an arm here without a
/// corresponding `GameAction` change is the smell this contract exists to
/// catch.
///
/// # Why six arms and not nine
///
/// `GameAction` ships ten variants and this enum six. `CounterChange` covers
/// `AddCounters` and `RemoveCounters` through its `adding` field — the one
/// place the projection is not 1:1, and that arm's own doc says so.
///
/// The three with no arm at all are `DrawCard`, `GainLife`
/// and `LoseLife`. Each affects a **player**, and `AffectedSet` names only
/// objects — so an arm for one of them would have no scoping mechanism, would
/// match every player's draw, and would then be rejected by an `affected` check
/// that no object-shaped set can pass. That is a card that silently does
/// nothing, which is the failure mode this tree refuses at the door.
///
/// They land in Phase RE, which is where `replacement-architecture.md` §9
/// schedules draw replacement (CR 614.11) and life-gain replacement (CR 119.10)
/// anyway, and they land *with* the player-scoping mechanism CR 614.1's
/// "whatever they're affecting" needs for a player. Adding an arm is a normal
/// diff — this enum is matched exhaustively and is not `#[non_exhaustive]`, so
/// every reader fails to compile rather than defaulting.
#[derive(Debug, Clone, PartialEq)]
pub enum EventPattern {
    /// CR 614.2 / 615.1. The affected thing is the damage *target*.
    ///
    /// Fieldless: CR 122.1c's shield counter is RB's only customer and it
    /// watches "damage would be dealt to **this** permanent", which `affected`
    /// already says. A source-side constraint — CR 615.10's own example,
    /// Daunting Defender's "if a *red* source would deal damage to a *Cleric
    /// you control*" — is a `PermanentFilter` field this arm grows in Phase RD,
    /// when there is a card to test it with.
    DealDamage,

    /// CR 400.6. `None` on a field means "any".
    ZoneChange {
        from: Option<Zone>,
        to: Option<Zone>,
        /// Nothing outside this pipeline and the trigger matcher may branch on
        /// a cause — see [`ZoneChangeCause`].
        cause: Option<ZoneChangeCause>,
        /// A constraint on the moving object beyond what `affected` says.
        ///
        /// `affected` scopes *which* objects the effect shields; this scopes
        /// the event. They coincide for every RB card, and the field exists
        /// because CR 903.9b's "if a **commander** would be put into its
        /// owner's hand or library" is a property of the object being moved
        /// while the shield is around a player's cards generally.
        object: Option<PermanentFilter>,
    },

    /// CR 122.1d. The affected thing is the permanent being untapped.
    Untap,

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

    /// CR 122.1's counter mutations. No RB customer watches one — the arm
    /// exists because `GameAction::AddCounters`/`RemoveCounters` do, and
    /// CR 614.16's counter doublers (Doubling Season's second ability, Vorinclex)
    /// are its Phase RE customers.
    CounterChange {
        counter: Option<CounterType>,
        /// `true` matches `AddCounters`, `false` matches `RemoveCounters`.
        adding: bool,
    },
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
/// | `Amount(..)` | 614.5 doublers, 615.7 partial prevention, 122.6a | RD/RE |
/// | `Retarget(..)` | 614.9 redirection, 616.1b | RD |
/// | `EnterWith(..)` | 614.1c/d | RC |
///
/// The last three are absent from the enum rather than present and
/// unimplemented, because an arm the pipeline cannot apply is a card that
/// silently does nothing. They are *features* on the codebase's own triage —
/// a normal diff whenever they land, with no fact lost by waiting. What each
/// one is *for* is recorded here so the reason it is not `Instead` survives:
/// `Amount` has to compose (two doublers turn 2 damage into 8, "not just 4"),
/// `Retarget` has to survive CR 614.9's destination re-check at application
/// time, and `EnterWith` has to accumulate across CR 616.1f iterations while
/// the permanent does not yet exist.
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
}

/// CR 616.1a–e — the forced-choice buckets, in the rule's own order.
///
/// `forced_bucket` returns the highest-priority non-empty class and only that
/// class; [`Self::Other`] is 616.1e's fallthrough, "any of the applicable
/// replacement and/or prevention effects may be chosen".
///
/// All five arms ship in Phase RB even though only `Other` has a producer,
/// because the *ordering* is what item 3 implements and a bucket that does not
/// exist cannot be ordered. `SelfReplacement` gets its producer with the first
/// CR 614.15 card, `ControlChanging` and `CopyOnEnter` in Phase RC-B.
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

/// How many times a replacement effect can fire.
///
/// **There is no `CounterBacked`.** CR 122.1c/d make the counter removal the
/// substituted event or the CR 615.5 rider, never bookkeeping, so a use that
/// removed one would write `BattlefieldEntity.counters` from inside
/// `consume_use` — off the chokepoint. Existence is asked at gather time, which
/// is where CR 614.4 wants it asked. CR 615.7's `Shield(u64)` is real and lands
/// with Phase RD. → `replacement-architecture.md` §3.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Uses {
    /// CR 614.1a static abilities, 615.10, 701.19b — every time, forever.
    Static,
    /// CR 701.19a's regeneration shield, CR 615.8's "next time [source] would
    /// deal damage" — one application, then the effect is gone.
    Once,
}

impl ReplacementDef {
    /// The common shape: a mandatory, unremarkable, always-on replacement.
    ///
    /// Named constructors rather than a `Default`, because `class` and
    /// `exempt_from_614_5` are the two fields where a wrong default is a rules
    /// bug rather than a style choice.
    pub fn new(pattern: EventPattern, affected: AffectedSet, rewrite: Rewrite) -> Self {
        ReplacementDef {
            pattern,
            affected,
            rewrite,
            then: None,
            class: ReplacementClass::Other,
            uses: Uses::Static,
            is_regeneration: false,
            exempt_from_614_5: false,
            optional: false,
        }
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
/// against a `ResolutionContext` whose single resolved target is the affected
/// object.
pub fn regeneration_rider() -> Effect {
    use crate::types::effects::{EffectRecipient, PermanentFilter, Primitive, SelectionFilter,
                                TargetCount};
    let it = || {
        EffectRecipient::Target(
            SelectionFilter::Permanent(PermanentFilter::All),
            TargetCount::Exactly(1),
        )
    };
    Effect::Sequence(vec![
        Effect::Atom(Primitive::RemoveAllDamage, it()),
        Effect::Atom(Primitive::Tap, it()),
        Effect::Atom(Primitive::RemoveFromCombat, it()),
    ])
}
