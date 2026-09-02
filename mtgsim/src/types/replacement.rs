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
//!   should arrive with the rule number that says so. It ships **four** —
//!   `Prevent`, `Instead`, `EnterWith` and `EnterUnderControlOf` — not §3.2's
//!   five plus one: an arm the pipeline cannot apply is worse than a missing
//!   one. `EnterWith` gained its performer in Phase RC-2 and
//!   `EnterUnderControlOf` its CR 616.1b bucket in RC-4.
//!
//! Per-mechanic variety goes in [`ReplacementDef::then`], which is the existing
//! `Effect` tree — no new vocabulary at all.

use crate::types::effects::{AffectedSet, CounterType, Effect, PermanentFilter, PlayerRef};
use crate::types::zones::{DestructionSource, Zone, ZoneChangeCause};

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
/// # Why seven arms and not ten
///
/// `GameAction` ships eleven variants and this enum seven. `CounterChange`
/// covers `AddCounters` and `RemoveCounters` through its `adding` field — the
/// one place the projection is not 1:1, and that arm's own doc says so.
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
    /// CR 614.2 / 615.1. The event's subject is the damage *target*.
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
        /// `affected` scopes *which* objects the effect applies to; this scopes
        /// the event. They coincide for every RB card, and the field exists
        /// because CR 903.9b's "if a **commander** would be put into its
        /// owner's hand or library" is a property of the object being moved
        /// while the shield is around a player's cards generally.
        object: Option<PermanentFilter>,
    },

    /// CR 122.1d. The event's subject is the permanent being untapped.
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
    /// **A prohibition on entering does not watch this event.** By the time an
    /// `EnterBattlefield` is proposed the object is already in the battlefield
    /// zone, so "Lands can't enter the battlefield" (Worms of the Earth) is a
    /// `Restriction::Event` over `ZoneChange { to: Some(Battlefield), .. }`,
    /// where stopping the event leaves the card where it was. CR 614.17d's
    /// frame reaches that zone change too — `engine::replacement::EntryFrame`.
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
/// | `EnterWith(..)` | 614.1c/d | **RC-2** |
/// | `EnterUnderControlOf(..)` | 616.1b, 614.1c | **RC-4** |
/// | `Amount(..)` | 614.5 doublers, 615.7 partial prevention | RD/RE |
/// | `Retarget(..)` | 614.9 redirection, 616.1b | RD |
///
/// The last two are absent from the enum rather than present and
/// unimplemented, because an arm the pipeline cannot apply is a card that
/// silently does nothing. They are *features* on the codebase's own triage —
/// a normal diff whenever they land, with no fact lost by waiting. What each
/// one is *for* is recorded here so the reason it is not `Instead` survives:
/// `Amount` has to compose (two doublers turn 2 damage into 8, "not just 4")
/// and `Retarget` has to survive CR 614.9's destination re-check at
/// application time.
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
    /// no `BattlefieldEntity` to tap and no counter map to write, which is the
    /// whole reason the modifications ride on the proposal instead of being
    /// applied as they are chosen.
    EnterWith(EnterMods),

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
}

/// How a permanent enters the battlefield, when something modified it
/// (CR 614.1c/d).
///
/// The payload of both [`Rewrite::EnterWith`] and
/// `GameAction::EnterBattlefield`: the same type describes what one effect
/// *adds* and what the permanent will *end up with*, which is what makes
/// [`Self::merge`] the whole of CR 616.1f's accumulation.
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
                // Plain addition, matching `BattlefieldEntity::add_counters`,
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
}

/// CR 616.1a–e — the forced-choice buckets, in the rule's own order.
///
/// `forced_bucket` returns the highest-priority non-empty class and only that
/// class; [`Self::Other`] is 616.1e's fallthrough, "any of the applicable
/// replacement and/or prevention effects may be chosen".
///
/// All five arms ship in Phase RB even though only `Other` has a producer,
/// because the *ordering* is what item 3 implements and a bucket that does not
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
    /// The CR 616.1 bucket a rewrite belongs to, read off the rewrite.
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
            Rewrite::Prevent | Rewrite::Instead(_) | Rewrite::EnterWith(_) => {
                ReplacementClass::Other
            }
        }
    }
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
        let class = ReplacementClass::from_rewrite(&rewrite);
        ReplacementDef {
            pattern,
            affected,
            rewrite,
            then: None,
            class,
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
/// against a `ResolutionContext` whose single resolved target is the event's
/// subject.
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
