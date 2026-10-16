//! The one table that answers an [`ObjectFilter`] for one object, and the one
//! predicate that answers whether a [`PlayerRef`] names a player.
//!
//! Every asker reaches them — the layer walk deciding a continuous effect's
//! affected set, and a selection, a replacement's or restriction's `ObjectSet`,
//! a trigger's subject and a cost from outside it — and each says through its
//! [`FilterReferents`] what its text's words for itself mean where it is asked.
//! What the askers differ in is what they hold, never how a leaf is answered
//! (`layers-architecture.md` §3.4, "One leaf table for both askers").
//!
//! A leaf may have no answer for an object — it is gone, or has no power —
//! and that refusal stays inside this module: `Not` and the connectives need
//! it, and every asker outside wants a yes or a no.

use crate::engine::layers::types::EffectiveCharacteristics;
use crate::engine::resolve::ResolvedTarget;
use crate::engine::targeting::{ChosenTargets, FilterIdentity, TargetInstance};
use crate::state::game_state::GameState;
use crate::types::effects::{CounterType, ObjectFilter, PlayerRef};
use crate::types::ids::{ObjectId, PlayerId};
use crate::types::triggers::Lki;

/// What a filter's words for its own text mean where it is asked: CR 109.5's
/// "you", CR 113.7a's "this" and its owner (CR 108.3), and the instances of
/// "target" announced before this one (CR 601.2c, 115.3).
///
/// `you` and `source_owner` take `&mut self` so the layer walk can resolve
/// them on first use: resolving them eagerly there is the cost
/// `layers-architecture.md` §3.4 measured.
pub(crate) trait FilterReferents {
    fn you(&mut self) -> PlayerId;
    /// The object `NotSource` is other than and whose owner `Owner` names;
    /// `None` where no text is being read.
    fn source(&self) -> Option<ObjectId>;
    fn source_owner(&mut self) -> Option<PlayerId>;
    fn earlier_targets(&self) -> EarlierTargets<'_>;
    /// How many counters of `kind`, or of every kind for `None`, the
    /// permanent `id` has: off the battlefield here, and off the entity the
    /// layer walk is building there, which under a look-ahead is the entering
    /// one. An object that is no permanent has none.
    fn counters_on(&self, game: &GameState, id: ObjectId, kind: Option<CounterType>) -> u32 {
        game.battlefield.get(&id).map_or(0, |entity| entity.counters_of(kind))
    }
}

/// The referents of an asker outside the layer walk, given as plain values:
/// "you" as a player, and "this" and the targets announced so far as a
/// [`FilterIdentity`]. Its one lookup is "its owner", the owner of "this" in
/// the store.
///
/// The layer walk resolves "you" lazily off a registry row (`FilterPlayers`)
/// and a trigger takes its words from its candidate (`TriggerReferents`);
/// every other asker already holds the values, and this is what lets them
/// reach the table: a target's legality, a cost's "another creature", a
/// resolving effect's filter, and through [`Self::for_effect`] a replacement's
/// or restriction's set, pattern and "caused by".
pub(crate) struct TextReferents<'g, 'a> {
    pub game: &'g GameState,
    pub you: PlayerId,
    pub identity: FilterIdentity<'a>,
}

impl<'g> TextReferents<'g, 'static> {
    /// The words of an effect's text: "you" its controller, "this" its
    /// source, and no announcement.
    pub(crate) fn for_effect(game: &'g GameState, controller: PlayerId, source: ObjectId) -> Self {
        TextReferents { game, you: controller, identity: FilterIdentity::for_text_of(source, EarlierTargets::None) }
    }
}

impl FilterReferents for TextReferents<'_, '_> {
    fn you(&mut self) -> PlayerId {
        self.you
    }

    fn source(&self) -> Option<ObjectId> {
        self.identity.source
    }

    fn source_owner(&mut self) -> Option<PlayerId> {
        self.identity.source.and_then(|source| self.game.objects.get(&source)).map(|object| object.owner)
    }

    fn earlier_targets(&self) -> EarlierTargets<'_> {
        self.identity.earlier_targets
    }
}

/// What the announced instances look like to `ObjectFilter::OtherThanInstance`,
/// without copying them.
///
/// Read only by a spell or ability with more than one "target" whose later
/// one says "another": Incremental Growth's "target creature, another target
/// creature, and a third target creature" (CR 115.3 lets one object be chosen
/// once per instance, so "another" is what forbids the repeat). The second
/// instance's filter carries `OtherThanInstance(0)`, which asks what the
/// first instance chose.
///
/// Two shapes answer the same question. Inside CR 601.2c's loop the earlier
/// choices are a [`ChosenTargets`] being built; inside CR 608.2b's re-check
/// they are the `&[TargetInstance]` already sitting on the `StackEntry`, and
/// collecting *that* into a `ChosenTargets` was a whole copy of the
/// announcement per resolution to serve one by-index read.
///
/// An enum rather than a trait object because both arms are known here, it
/// stays `Copy`, and `NONE` needs no promoted static.
#[derive(Clone, Copy)]
pub enum EarlierTargets<'a> {
    /// Outside both, where the leaf is refused rather than answered.
    None,
    /// CR 601.2c's loop: the instances announced so far.
    Chosen(&'a ChosenTargets),
    /// CR 608.2b's re-check: the announcement on the entry, borrowed.
    Announced(&'a [TargetInstance]),
}

impl EarlierTargets<'_> {
    fn len(&self) -> usize {
        match self {
            EarlierTargets::None => 0,
            EarlierTargets::Chosen(c) => c.len(),
            EarlierTargets::Announced(i) => i.len(),
        }
    }

    /// Was `target` chosen for instance `ix`? By id: "another target" asks
    /// whether the announcement named this object, and an announcement names
    /// objects as they were at one moment.
    fn instance_names(&self, ix: usize, target: ResolvedTarget) -> bool {
        match self {
            EarlierTargets::None => false,
            EarlierTargets::Chosen(c) => c.instance(ix).contains(&target),
            EarlierTargets::Announced(i) => i.get(ix).is_some_and(|inst| inst.as_resolved_targets().any(|t| t == target)),
        }
    }
}

/// Why a leaf could not say whether an object matches it: every refusal the
/// table can make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterRefusal {
    /// The object is gone from the store, and nothing the asker holds
    /// answers the leaf: whether it is a token, which last known information
    /// does not carry, its owner without that information, or any leaf asked
    /// with no frame at all.
    ObjectGone(ObjectId),
    /// `NotSource`, or a `PlayerRef::Owner`, asked where nothing names a
    /// source.
    NoSource(ObjectId),
    /// `OtherThanInstance` naming an instance of "target" not yet announced.
    InstanceNotAnnounced { object: ObjectId, instance: usize },
    /// `PowerLE` asked of an object with no power (CR 208.3).
    NoPower(ObjectId),
}

impl FilterRefusal {
    /// A question the filter should never have been asked: a leaf the card
    /// wrote where the asking site cannot answer it, or a site that did not
    /// pass what it knows. Not a gone object, which is the rules' "no" until
    /// last known information answers it, nor an object with no power, of
    /// which CR 208.3 makes "power N or less" and its negation both false.
    fn is_malformed_question(self) -> bool {
        match self {
            FilterRefusal::ObjectGone(_) | FilterRefusal::NoPower(_) => false,
            FilterRefusal::NoSource(_) | FilterRefusal::InstanceNotAnnounced { .. } => true,
        }
    }
}

impl std::fmt::Display for FilterRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterRefusal::ObjectGone(id) => write!(f, "Object {id} not found"),
            FilterRefusal::NoSource(id) => {
                write!(f, "a filter asked of {id} names a source, \"another\" or \"its owner\", and nothing is being read")
            }
            FilterRefusal::InstanceNotAnnounced { object, instance } => write!(
                f,
                "ObjectFilter::OtherThanInstance({instance}) on {object} names an instance of \
                 \"target\" that has not been announced (CR 601.2c). Only the \
                 announcement loop and the CR 608.2b re-check hold the earlier_targets \
                 instances; a filter asked anywhere else cannot carry this leaf."
            ),
            FilterRefusal::NoPower(id) => write!(f, "Object {id} has no power"),
        }
    }
}

/// Does `id` match `filter`, over a frame `frame` supplies on demand (`None`
/// for an object with none) and the words `referents` gives its text?
/// `last_known` is the last known information the asker holds the frame
/// with, which says who owned the object once the store has lost it (CR
/// 108.3) and which counters it had (CR 122.1), neither a characteristic.
///
/// A refusal reads as no. A malformed question — a leaf the card wrote where
/// the asker cannot answer it — is loud in a debug build and `cargo test`,
/// since its card would otherwise silently do nothing (`codebase-state.md`
/// item 103).
pub(crate) fn evaluate<'f>(
    game: &GameState,
    filter: &ObjectFilter,
    id: ObjectId,
    frame: &impl Fn() -> Option<&'f EffectiveCharacteristics>,
    last_known: Option<&Lki>,
    referents: &mut impl FilterReferents,
) -> bool {
    match answer_or_refuse(game, filter, id, frame, last_known, referents) {
        Ok(matches) => matches,
        Err(refusal) => {
            debug_assert!(!refusal.is_malformed_question(), "a filter was asked a malformed question: {refusal}");
            false
        }
    }
}

/// [`evaluate`]'s three values: yes, no, or no answer. A refusal is a leaf with
/// no truth value. `Not` keeps it, and `And` and `Or` are Kleene's: an arm
/// that decides decides on either side, so a filter's answer does not depend
/// on the order its arms were written in.
fn answer_or_refuse<'f>(
    game: &GameState,
    filter: &ObjectFilter,
    id: ObjectId,
    frame: &impl Fn() -> Option<&'f EffectiveCharacteristics>,
    last_known: Option<&Lki>,
    referents: &mut impl FilterReferents,
) -> Result<bool, FilterRefusal> {
    let chars = || frame().ok_or(FilterRefusal::ObjectGone(id));
    // Read only by the leaves no frame answers: a frame may stand for an
    // object the store has lost (CR 608.2h).
    let object = || game.objects.get(&id).ok_or(FilterRefusal::ObjectGone(id));
    match filter {
        ObjectFilter::All => Ok(true),
        ObjectFilter::ByType(card_type) => Ok(chars()?.types.contains(card_type)),
        ObjectFilter::BySubtype(subtype) => Ok(chars()?.subtypes.contains(subtype)),
        ObjectFilter::BySupertype(supertype) => Ok(chars()?.supertypes.contains(supertype)),
        ObjectFilter::ByColor(color) => Ok(chars()?.colors.contains(color)),
        ObjectFilter::ByController(who) => {
            let controller = chars()?.controller;
            player_ref_names(who, controller, referents).ok_or(FilterRefusal::NoSource(id))
        }
        // CR 111.1 / 707.2 — being a token is a property of the object, not of
        // its characteristics: a copy effect does not make a nontoken
        // permanent a token.
        ObjectFilter::Token => Ok(object()?.is_token),
        // CR 108.3 / 400.3 — ownership, not control, which no layer touches.
        ObjectFilter::ByOwner(who) => {
            let owner = match last_known {
                Some(lki) => lki.owner(),
                None => object()?.owner,
            };
            player_ref_names(who, owner, referents).ok_or(FilterRefusal::NoSource(id))
        }
        ObjectFilter::PowerLE(max_power) => {
            chars()?.power.map(|power| power <= *max_power).ok_or(FilterRefusal::NoPower(id))
        }
        ObjectFilter::WithCounters { kind, count } => {
            let n = match last_known {
                Some(lki) => lki.counters_of(*kind),
                None => referents.counters_on(game, id, *kind),
            };
            Ok(count.met_by(u64::from(n)))
        }
        // Identity, off the ids: no layer makes an object other than itself.
        ObjectFilter::NotSource => referents.source().map(|source| id != source).ok_or(FilterRefusal::NoSource(id)),
        // CR 601.2c's "another target". An instance that announced nothing (an
        // "up to" taken at zero, CR 115.6) excludes nothing.
        ObjectFilter::OtherThanInstance(ix) => {
            let earlier = referents.earlier_targets();
            if *ix >= earlier.len() {
                return Err(FilterRefusal::InstanceNotAnnounced { object: id, instance: *ix });
            }
            Ok(!earlier.instance_names(*ix, ResolvedTarget::Object(id)))
        }
        ObjectFilter::And(a, b) => {
            let left = answer_or_refuse(game, a, id, frame, last_known, referents);
            if left == Ok(false) {
                return left;
            }
            let right = answer_or_refuse(game, b, id, frame, last_known, referents);
            if right == Ok(false) || left.is_ok() { right } else { left }
        }
        ObjectFilter::Or(a, b) => {
            let left = answer_or_refuse(game, a, id, frame, last_known, referents);
            if left == Ok(true) {
                return left;
            }
            let right = answer_or_refuse(game, b, id, frame, last_known, referents);
            if right == Ok(true) || left.is_ok() { right } else { left }
        }
        ObjectFilter::Not(inner) => answer_or_refuse(game, inner, id, frame, last_known, referents).map(|matches| !matches),
    }
}

/// Does `who` name `player`, for text whose words mean `referents`? `None`
/// when it names the source's owner and nothing names a source.
///
/// `Opponent` is a predicate, not an id: CR 102.3 makes "your opponents" a set
/// above two players. `Owner` is the source's owner, what "its owner" names in
/// a predicate (Xantcha's "planeswalkers its owner controls"); an action's
/// `Owner` is the owner of the object acted on, and its readers' own
/// (`compute::resolve_set_controller`).
pub(crate) fn player_ref_names(who: &PlayerRef, player: PlayerId, referents: &mut impl FilterReferents) -> Option<bool> {
    Some(match who {
        PlayerRef::You => player == referents.you(),
        PlayerRef::Opponent => player != referents.you(),
        PlayerRef::Owner => player == referents.source_owner()?,
        PlayerRef::Player(pid) => player == *pid,
    })
}
