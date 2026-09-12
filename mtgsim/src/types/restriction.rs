//! "Can't" effects (CR 101.2, 614.17) — the type surface.
//!
//! A "can't" is discovered exactly the way a replacement effect is and differs
//! from one only in what it is asked at (`cant-effects-architecture.md` §3.1).
//! So this module borrows [`EventPattern`], [`AffectedSet`] and [`PlayerSet`]
//! verbatim and adds one field of its own; the vocabulary that would have been
//! new is vocabulary the replacement pipeline already owns.
//!
//! Data only, like `types::replacement`, so a card file can write a
//! `RestrictionDef` without reaching into the engine. The enforcement points
//! live in `engine::restriction`.
//!
//! # The growth contract
//!
//! [`Restriction`] grows on **three axes and no other**, and each is an
//! enumeration the engine already owns (§3.3):
//!
//! | Axis | What a "can't" attaches to | Bounded by | Open? |
//! |---|---|---|---|
//! | 1 | an event the engine proposes | `GameAction`, via [`EventPattern`] | grows with the engine |
//! | 2 | a choice a player makes | `ChoiceKind` (`ui/choice_types.rs`) | grows with the engine |
//! | 3 | the application of an effect | `gather`'s one application site | **closed at one arm** |
//!
//! Axis 1 never grows *here*: a new replaceable event arrives as a `GameAction`
//! variant and reaches [`Restriction::Event`] through `EventPattern` for free.
//! An event-shaped arm added to this enum is the smell. Axis 2 ships empty in
//! RS-1 and is RS-2/RS-3's; every arm it gains must name one `ChoiceKind`.
//!
//! Matched exhaustively and deliberately not `#[non_exhaustive]`, for
//! `types::replacement`'s reason: an arm no enforcement point consults is a card
//! that silently does nothing, and a normal diff fails to compile at every
//! reader instead.

use crate::types::effects::{AffectedSet, PlayerRef, PlayerSet};
use crate::types::replacement::EventPattern;

/// One CR 101.2 "can't" — a prohibition on an action or an event.
///
/// Deliberately *not* shaped like `ReplacementDef`. A replacement effect is a
/// shield around a thing (CR 614.1's own words), so it splits into a `pattern`
/// and an `affected`. A restriction is a prohibition on an **action**, and
/// CR 508.1c, 509.1b and 601.3 all phrase it that way — so the subject varies
/// per arm and lives inside it.
///
/// There is no `rewrite`, no `then`, no `uses`, no `class`, no `optional`. A
/// "can't" does not replace, has no rest-of-the-effect, is never spent, is never
/// ordered against another "can't" (CR 101.2 needs no tiebreak: two prohibitions
/// agree), and is never optional. **Five fields `ReplacementDef` needs that this
/// does not is the evidence the two are different types rather than one type
/// with a flag.**
///
/// A one-field wrapper today on purpose: `unless: Option<Condition>` (§2.4)
/// lands in it when Phase 6 gives `Condition` a meaning, and a bare enum would
/// have to become a struct at that point across every reader.
#[derive(Debug, Clone, PartialEq)]
pub struct RestrictionDef {
    pub what: Restriction,
}

impl RestrictionDef {
    pub fn new(what: Restriction) -> Self {
        RestrictionDef { what }
    }
}

/// What a "can't" forbids.
///
/// See the module docs for the growth contract. Two arms in RS-1: one per axis
/// that has a consumer.
#[derive(Debug, Clone, PartialEq)]
pub enum Restriction {
    // ---- Axis 1: an event the engine proposes (CR 614.17) ----------------
    /// CR 614.17 — "some effects state that something can't happen". Checked
    /// ahead of the replacement pipeline and winning over it (CR 101.2).
    ///
    /// `pattern` and `affected` are the replacement pipeline's, reused verbatim:
    /// "this permanent can't be destroyed" is the same predicate over the same
    /// proposal as "if this permanent would be destroyed, instead …", minus the
    /// instead. [`Self::Event::by`] is the one addition — CR 101.2 scoped by
    /// what *caused* the event, which is §2.6's Sigarda family.
    Event {
        pattern: EventPattern,
        /// The objects this forbids the event about. **Half a pair**, like
        /// [`Self::ApplyReplacement::to_objects`] and for the same reason: the
        /// two sets are unioned and neither is primary.
        affected: AffectedSet,
        /// The other half — CR 101.2 about an event whose subject is a
        /// *player*. "Players can't gain life" (Skullcrack, Leyline of
        /// Punishment, 25 cards) is the family, and CR 119.7 spells out what it
        /// costs the pipeline: *"a replacement effect that would replace a life
        /// gain event affecting that player won't do anything"*, which falls
        /// out of `is_prohibited` being asked ahead of `gather`.
        ///
        /// Unioned with `affected` by the same `set_affects` that unions a
        /// `ReplacementDef`'s two sets — so an event about an object asks
        /// `affected` and one about a player asks this, and neither set has any
        /// say about the other's kind of subject. [`PlayerSet::Nobody`] is
        /// therefore what every object-scoped restriction written before RE-3
        /// means and what it keeps meaning: "this is not about players".
        affected_players: PlayerSet,
        by: Option<SourceFilter>,
    },

    // ---- Axis 3: applying an effect (CR 701.19c, 615.12) -----------------
    /// Withholds a *replacement or prevention effect* rather than blocking an
    /// event or a choice.
    ///
    /// This **is** a third axis rather than a shape of the first: CR 101.2's
    /// "it" can be the application of an effect rather than an event. CR 701.19c
    /// says a regeneration shield is "not applied", and the destruction it would
    /// have replaced still happens — no `EventPattern` can say that.
    ///
    /// **The axis is closed at one arm, and that is what makes it safe.** Axes 1
    /// and 2 are open because `GameAction` and `ChoiceKind` grow; this one
    /// cannot, because there is exactly one place in the engine where an effect
    /// is applied to an event (`gather`'s `push_if_applicable`) and the CR names
    /// exactly two things that may be withheld there — regeneration shields
    /// (701.19c) and prevention effects (615.12). A second arm here would be a
    /// claim that the engine applies effects somewhere else.
    ApplyReplacement {
        kind: ReplacementKindFilter,
        /// The objects this withholds an effect from. **Named as half a pair**,
        /// which `ReplacementDef`'s `affected`/`affected_players` is not: a
        /// bare `to` beside a `to_players` reads as the whole set with a
        /// modifier hung off it, and it is not — the two are unioned and
        /// neither is primary (RD-4 review, 2026-09-09).
        to_objects: AffectedSet,
        /// The other half of "whatever they're affecting" — CR 615.12's
        /// "damage can't be prevented" is about damage dealt to *players* as
        /// much as to permanents, and a prevention effect's subject is a
        /// player whenever the damage is.
        ///
        /// Unioned with [`Self::ApplyReplacement::to`] by the same
        /// `set_affects` that unions a `ReplacementDef`'s two sets, which is
        /// what makes a "can't" the same predicate as the effect it withholds
        /// (`cant-effects-architecture.md` §3.1). Skullcrack's is `Everyone`
        /// plus `Filter { All }`; a hypothetical "damage to you can't be
        /// prevented" would be `You` plus `NO_OBJECTS`.
        ///
        /// [`Self::Event`] gained the same field in RE-3, with Skullcrack —
        /// which is how RD-4 said it would arrive (`replacement-architecture.md`
        /// §9, RD decision 0: "the day one is, it adds the field with its
        /// card").
        to_players: PlayerSet,
    },
}

/// CR 101.2 scoped by what caused the event — §2.6's Sigarda family.
///
/// > Sigarda, Host of Herons — "Spells and abilities your opponents control
/// > can't cause you to sacrifice permanents."
///
/// The provenance this reads is already threaded: `ActionContext::resolution`
/// carries the resolving spell or ability, so this is a field rather than a
/// mechanism. `None` on [`Restriction::Event::by`] means "however caused".
///
/// **A turn-based or state-based action has no controller**, so no
/// `SourceFilter` matches one. That is the right answer rather than an
/// omission — Sigarda does not stop CR 704.5's sacrifices, and there are none
/// to stop.
#[derive(Debug, Clone, PartialEq)]
pub enum SourceFilter {
    /// The spell or ability that proposed the event is controlled by this
    /// player, read relative to the restriction's own controller — so
    /// `PlayerRef::Opponent` is Sigarda's "your opponents" and `PlayerRef::You`
    /// is Tamiyo, Collector of Tales' mirror ("Spells and abilities your
    /// opponents control can't cause you to discard cards" reads the same way).
    ControlledBy(PlayerRef),
}

/// Which class of replacement effect a [`Restriction::ApplyReplacement`]
/// withholds.
///
/// Closed for the same reason its arm is: CR 701.19c and CR 615.12 name the two
/// things the rules permit withholding, and nothing else in the CR withholds an
/// effect at its application site.
///
/// [`Self::Prevention`] got its producer in RD-4, and **not** by widening
/// `ReplacementDef::is_regeneration` into a `ReplacementKind` the way
/// `cant-effects-architecture.md` §4.7 predicted: CR 615.1a defines a
/// prevention effect by the word "prevent", which the def already carries in
/// its pattern and rewrite, so `ReplacementDef::is_prevention()` is derived and
/// no card can forget to set it (`replacement-architecture.md` §11 item 25).
/// `is_regeneration` keeps its authored bit, because nothing about a
/// regeneration def distinguishes it from any other `Prevent`-with-a-rider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementKindFilter {
    /// CR 701.19a's regeneration shield — "effects that say that a permanent
    /// can't be regenerated … cause regeneration shields to not be applied".
    Regeneration,
    /// CR 615.12's prevention effects — "damage can't be prevented".
    Prevention,
}
