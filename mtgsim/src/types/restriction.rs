//! "Can't" effects (CR 101.2, 614.17) — the type surface.
//!
//! A "can't" is discovered exactly the way a replacement effect is and differs
//! from one only in what it is asked at (`cant-effects-architecture.md` §3.1).
//! So this module borrows [`EventPattern`] and [`AffectedSet`]
//! verbatim and adds one field of its own; the vocabulary that would have been
//! new is vocabulary the replacement pipeline already owns.
//!
//! Data only, like `types::replacement`, so a card file can write a
//! `RestrictionDef` without reaching into the engine. The enforcement points
//! live in `engine::restriction`.
//!
//! # The growth contract
//!
//! [`Restriction`] grows on **four axes and no other**, and each is an
//! enumeration the engine already owns (§3.3):
//!
//! | Axis | What a "can't" attaches to | Bounded by | Open? |
//! |---|---|---|---|
//! | 1 | an event the engine proposes | `GameAction`, via [`EventPattern`] | grows with the engine |
//! | 2 | a choice a player makes | `ChoiceKind` (`ui/choice_types.rs`) | grows with the engine |
//! | 3 | the application of an effect | `gather`'s one application site | **closed at one arm** |
//! | 4 | a rule's own action | [`RuleAction`]: the check's `StateBasedAction`, and each turn-based action a card switches off | grows with its first card |
//!
//! Axis 4 is asked where the rule acts: the state-based check of every
//! member it gathers, ahead of CR 704.7's collapse, and CR 514.2's wipe of
//! each damaged permanent.
//!
//! Axis 1 never grows *here*: a new replaceable event arrives as a `GameAction`
//! variant and reaches [`Restriction::Event`] through `EventPattern` for free.
//! An event-shaped arm added to this enum is the smell. Axis 2 is empty
//! until `cant-be-targeted`, `cant-cast-play-or-activate` and the combat phases; every arm it gains must name one `ChoiceKind`.
//!
//! Matched exhaustively and deliberately not `#[non_exhaustive]`, for
//! `types::replacement`'s reason: an arm no enforcement point consults is a card
//! that silently does nothing, and a normal diff fails to compile at every
//! reader instead.

use crate::state::game_state::GameState;
use crate::engine::object_filter::{player_ref_names, FilterReferents};
use crate::events::cause::EventCause;
use crate::types::effects::{AffectedSet, PlayerRef};
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
/// lands in it with CR 508.1c's "can't … unless", and a bare enum would have
/// to become a struct at that point across every reader.
#[derive(Debug, Clone, PartialEq)]
pub struct RestrictionDef {
    pub restriction: Restriction,
}

impl RestrictionDef {
    pub fn new(restriction: Restriction) -> Self {
        RestrictionDef { restriction }
    }
}

/// What a "can't" forbids.
///
/// See the module docs for the growth contract: one arm per axis that has
/// a consumer.
#[derive(Debug, Clone, PartialEq)]
pub enum Restriction {
    // ---- Axis 1: an event the engine proposes (CR 614.17) ----------------
    /// CR 614.17 — "some effects state that something can't happen". Checked
    /// ahead of the replacement pipeline and winning over it (CR 101.2).
    ///
    /// `pattern` and `affected` are the replacement pipeline's, reused verbatim:
    /// "this permanent can't be destroyed" is the same predicate over the same
    /// proposal as "if this permanent would be destroyed, instead …", minus the
    /// instead. [`Self::Event::by`] is the one addition — which source
    /// *caused* the event, §2.6's Sigarda family. That clause is printed on
    /// cards rather than stated by CR 101.2, which says only that a "can't"
    /// wins; see [`SourceFilter`].
    Event {
        pattern: EventPattern,
        /// The objects and players this forbids the event about, asked by the
        /// same `set_affects` a `ReplacementDef`'s are. Its player half is CR
        /// 101.2 about an event whose subject is a *player*: "players can't
        /// gain life" (Skullcrack, Leyline of Punishment, 25 cards) is the
        /// family, and CR 119.7 spells out what it costs the pipeline: *"a
        /// replacement effect that would replace a life gain event affecting
        /// that player won't do anything"*, which falls out of `is_prohibited`
        /// being asked ahead of `gather`.
        affected: AffectedSet,
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
        /// The objects and players this withholds an effect from: CR 615.12's
        /// "damage can't be prevented" is about damage dealt to *players* as
        /// much as to permanents, and a prevention effect's subject is a
        /// player whenever the damage is. Asked by the same `set_affects` a
        /// `ReplacementDef`'s are, which is what makes a "can't" the same
        /// predicate as the effect it withholds
        /// (`cant-effects-architecture.md` §3.1). Skullcrack's is every
        /// permanent and every player.
        affected: AffectedSet,
    },

    // ---- Axis 4: a rule's own action (CR 101.1) --------------------------
    /// A rule's own action does not happen to what `affected` names: "damage
    /// isn't removed from this creature during cleanup steps" (CR 514.2),
    /// "you don't lose the game for having 0 or less life" (704.5a), "the
    /// legend rule doesn't apply" (704.5j). Not a "can't" of an event: the
    /// rule's action is a turn-based or state-based one, which the rule
    /// performs or proposes only for what no such restriction names, so a
    /// player Phyrexian Unlife keeps at 0 life still loses to poison. CR 101.1:
    /// the card takes precedence over the rule.
    RuleDoesNotApply { rule: RuleAction, affected: AffectedSet },
}

/// A rule whose own action a card can switch off, named by the rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleAction {
    /// A state-based action (CR 704.5, 704.6), by its subrule. The check
    /// asks of every member it gathers.
    StateBased(crate::events::cause::StateBasedAction),
    /// CR 514.2's removal of the damage marked on a permanent in the cleanup
    /// step.
    CleanupDamageRemoval,
}

impl Restriction {
    /// What it is affecting, whichever arm it is. Every arm names one, and
    /// `Primitive::Restrict` fills an authored [`AffectedSet::NOTHING`] with the
    /// resolution's targets without caring which arm it is.
    pub fn affected(&self) -> &AffectedSet {
        match self {
            Restriction::Event { affected, .. }
            | Restriction::ApplyReplacement { affected, .. }
            | Restriction::RuleDoesNotApply { affected, .. } => affected,
        }
    }

    /// [`Self::affected`], to fill in.
    pub fn affected_mut(&mut self) -> &mut AffectedSet {
        match self {
            Restriction::Event { affected, .. }
            | Restriction::ApplyReplacement { affected, .. }
            | Restriction::RuleDoesNotApply { affected, .. } => affected,
        }
    }
}

/// What *caused* an event, as a predicate — §2.6's Sigarda family.
///
/// > Sigarda, Host of Herons — "Spells and abilities your opponents control
/// > can't cause you to sacrifice permanents."
///
/// **Not a rule of CR 101.2's**, though [`Restriction::Event::by`] sits under
/// one: 101.2 says only that a "can't" takes precedence, and the cause clause
/// is a shape printed on cards. The fact it reads is the event's
/// [`EventCause`]: the spell or ability whose effect caused it (CR 608.2) and
/// its controller, which CR 109.5 makes the effect's "you". `None` on
/// [`Restriction::Event::by`] means "however caused".
///
/// **Only a spell's or an ability's effect has a controller to match**, so no
/// `SourceFilter` matches a cost, a special action, a turn-based or
/// state-based action, or a rule's move. That is the right answer rather than
/// an omission — Sigarda does not stop CR 704.5's sacrifices, and there are none
/// to stop.
///
/// **Read by a replacement effect too.** The "can't" side and
/// the CR 614 side are one question asked at two sites: Tamiyo, Collector of
/// Tales'
/// "spells and abilities your opponents control can't cause you to discard
/// cards" and Nephalia Academy's "if a spell or ability an opponent controls
/// causes you to discard a card" name the same provenance, so
/// [`crate::types::replacement::ReplacementDef::by`] is this type and not a
/// second spelling of it.
#[derive(Debug, Clone, PartialEq)]
pub enum SourceFilter {
    /// The spell or ability that proposed the event is controlled by this
    /// player, read relative to the restriction's own controller — so
    /// `PlayerRef::Opponent` is Sigarda's "your opponents" and `PlayerRef::You`
    /// is Tamiyo, Collector of Tales' mirror ("Spells and abilities your
    /// opponents control can't cause you to discard cards" reads the same way).
    ControlledBy(PlayerRef),
}

impl SourceFilter {
    /// Was the event caused by a source this admits?
    ///
    /// `cause` is the proposal's (`ActionContext::cause`) or the record's
    /// (`EventStamp::cause`); `referents` are the words of the effect doing the
    /// asking — the restriction's, the replacement instance's, or the triggered
    /// ability's — whose "you" is its controller (CR 109.5) and whose "its
    /// owner" is its source's.
    ///
    /// On the type rather than in either enforcement point, because both ask
    /// it: `engine::restriction`'s `cause_matches` for a "can't" and
    /// `replacement::gather`'s `applies_to` for a replacement effect. A copy
    /// in each would be two answers to CR 101.2's one question.
    pub(crate) fn matches(&self, game: &GameState, cause: &EventCause, referents: &mut impl FilterReferents) -> bool {
        // Only an effect a spell or ability generated has a controller to
        // match. The right answer rather than an omission: Sigarda does not
        // stop CR 704.5's sacrifices, Nephalia Academy does not redirect a
        // card discarded to hand size, and costs "aren't effects" (Library of
        // Leng's ruling).
        let Some(cause) = cause.spell_or_ability_controller() else {
            return false;
        };
        match self {
            // Relative to the *asking* effect, whose "you" is CR 109.5's for a
            // static ability: Sigarda's "your opponents" is an opponent of
            // whoever currently controls Sigarda.
            SourceFilter::ControlledBy(player_ref) => player_ref_names(game, player_ref, cause, referents) == Some(true),
        }
    }
}

/// Which class of replacement effect a [`Restriction::ApplyReplacement`]
/// withholds.
///
/// Closed for the same reason its arm is: CR 701.19c and CR 615.12 name the two
/// things the rules permit withholding, and nothing else in the CR withholds an
/// effect at its application site.
///
/// [`Self::Prevention`] is derived, not authored: CR 615.1a defines a
/// prevention effect by the word "prevent", which the def already carries in
/// its pattern and rewrite, so `ReplacementDef::is_prevention()` reads it off
/// and no card can forget to set it (`replacement-architecture.md` §11 item
/// 25). `is_regeneration` keeps its authored bit, because nothing about a
/// regeneration def distinguishes it from any other `Prevent`-with-a-rider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplacementKindFilter {
    /// CR 701.19a's regeneration shield — "effects that say that a permanent
    /// can't be regenerated … cause regeneration shields to not be applied".
    Regeneration,
    /// CR 615.12's prevention effects — "damage can't be prevented".
    Prevention,
}
