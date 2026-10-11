use crate::events::cause::EventCause;
use crate::events::event::EventSeq;
use crate::types::effects::ReturnUnder;
use crate::types::ids::{ObjectId, ObjectRef, PlayerId, UntilReturnId};
use crate::types::triggers::{RememberedObject, TriggerEvent};

/// CR 610.3 — the return an "until" exile waits to make: a one-shot effect
/// created immediately after its event, which uses no stack and is no
/// triggered ability. On `GameState::until_returns` from the exile until its
/// event happens.
#[derive(Debug, Clone, PartialEq)]
pub struct UntilReturn {
    pub id: UntilReturnId,
    /// The specified event, read as a trigger arm is.
    pub until: TriggerEvent,
    /// "This" in the event: the object whose ability exiled, or the spell,
    /// by identity.
    pub source: RememberedObject,
    /// What the event refers to: the target "until target enchantment
    /// leaves the battlefield" named, by identity.
    pub referred: Vec<RememberedObject>,
    /// Each object the exile moved, by identity where it put it, and the zone
    /// it returns to (CR 610.3: "its previous zone").
    pub returns: Vec<(ObjectRef, Zone)>,
    /// Whose control a returned permanent enters under (CR 610.3c).
    pub under: ReturnUnder,
    /// The exile's controller, the "you" of `ReturnUnder::You`.
    pub controller: PlayerId,
    /// "Its owner" in the event: the source's (CR 108.3), read as the return
    /// was made.
    pub owner: PlayerId,
    /// The first record performed after it was made: an event before it is
    /// not the one it waits for.
    pub created_at: EventSeq,
    /// What the return is caused by: the exile's own effect, which CR 610.3
    /// makes the second one-shot effect's.
    pub cause: EventCause,
}

/// Game zones (rule 400.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Zone {
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Stack,
    Exile,
    Command,
}

impl Zone {
    /// Whether objects in this zone are public information
    pub fn is_public(&self) -> bool {
        matches!(self, Zone::Battlefield | Zone::Graveyard | Zone::Stack | Zone::Exile | Zone::Command)
    }
}

/// A set of [`Zone`]s — which zones a continuous effect's filter reaches.
///
/// **The reach of an `ObjectSet::Filter`, and the reason it is a field on the
/// set rather than a leaf inside `ObjectFilter`** (`layers-architecture.md`
/// §13c decision 3). The layer walk asks two different questions at two
/// different times: *seeding* asks which objects need a frame at all, once per
/// pass and before any layer runs; *matching* asks whether one candidate
/// matches, per layer per candidate. A zone answers only the first. As a
/// filter leaf it would sit inside a boolean algebra that only *matching*
/// evaluates, and seeding would have to infer reach by walking the tree —
/// where `Not` makes the answer a complement (`Not(InZone(Battlefield))` is
/// every library in the game), so a sound inference must widen to "all zones"
/// at any `Not` and the fast path dies on exactly the cards it exists for.
///
/// **A complement is an ordinary value here, and that is the point of the set
/// being concrete.** `ALL.without(BATTLEFIELD)` is a bitmask like any other:
/// bounded, readable off the row, and costing `Board::seed` one `iter()`. The
/// limit is on where the complement lives, not on whether one exists.
///
/// Printed cards want it: Grist, the Hunger Tide is "as long as Grist isn't on
/// the battlefield, it's a 1/1 Insect creature in addition to its other
/// types" (ruling: "anywhere but on the battlefield", Scryfall, 2026-09-14),
/// and Mycosynth Lattice and Painter's Servant open on "all cards that aren't
/// on the battlefield". The rule outranks the card list anyway: a facility
/// the CR states is owed whether or not a card prints it
/// (`engineering-practices.md` §4).
///
/// A hand-rolled bitmask following `engine::layers::board::Channels` rather
/// than a `bitflags` dependency the crate does not have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct ZoneSet(u8);

impl ZoneSet {
    pub const EMPTY: ZoneSet = ZoneSet(0);
    pub const LIBRARY: ZoneSet = ZoneSet(1 << 0);
    pub const HAND: ZoneSet = ZoneSet(1 << 1);
    pub const BATTLEFIELD: ZoneSet = ZoneSet(1 << 2);
    pub const GRAVEYARD: ZoneSet = ZoneSet(1 << 3);
    pub const STACK: ZoneSet = ZoneSet(1 << 4);
    pub const EXILE: ZoneSet = ZoneSet(1 << 5);
    pub const COMMAND: ZoneSet = ZoneSet(1 << 6);
    /// CR 400.1's seven zones — Mycosynth Lattice's "all cards that aren't on
    /// the battlefield, spells, and permanents".
    pub const ALL: ZoneSet = ZoneSet((1 << 7) - 1);

    /// Everywhere but the battlefield — Grist's "as long as [it] isn't on the
    /// battlefield", Mycosynth Lattice's "all cards that aren't on the
    /// battlefield". Spelled as a constant because it is the complement the
    /// CR actually names; any other is [`Self::without`].
    pub const EVERYWHERE_BUT_BATTLEFIELD: ZoneSet = ZoneSet(ZoneSet::ALL.0 & !ZoneSet::BATTLEFIELD.0);

    /// CR 400.2's hidden zones, a library and a hand: the zones a layer pass
    /// leaves out (`layers-architecture.md` §13e). [`Zone::is_public`] is the
    /// same partition asked of one zone, and a test pins the two together.
    pub const HIDDEN: ZoneSet = ZoneSet(ZoneSet::LIBRARY.0 | ZoneSet::HAND.0);

    /// This set minus `other` — the general complement.
    pub const fn without(self, other: ZoneSet) -> ZoneSet {
        ZoneSet(self.0 & !other.0)
    }

    /// The zones this set holds that are **not** the battlefield.
    ///
    /// What [`ZoneSet::BATTLEFIELD`]-only rows cost nothing for, and the exact
    /// question `Board::seed` asks: the battlefield is already every pass's
    /// working set, so only these zones add members.
    pub fn beyond_battlefield(self) -> ZoneSet {
        self.without(ZoneSet::BATTLEFIELD)
    }

    const fn bit(zone: Zone) -> u8 {
        match zone {
            Zone::Library => ZoneSet::LIBRARY.0,
            Zone::Hand => ZoneSet::HAND.0,
            Zone::Battlefield => ZoneSet::BATTLEFIELD.0,
            Zone::Graveyard => ZoneSet::GRAVEYARD.0,
            Zone::Stack => ZoneSet::STACK.0,
            Zone::Exile => ZoneSet::EXILE.0,
            Zone::Command => ZoneSet::COMMAND.0,
        }
    }

    pub const fn of(zone: Zone) -> ZoneSet {
        ZoneSet(ZoneSet::bit(zone))
    }

    pub fn contains(self, zone: Zone) -> bool {
        self.0 & ZoneSet::bit(zone) != 0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Whether any zone in this set is hidden (CR 400.2 — library and hand).
    ///
    /// **Not used to refuse anything**, and deliberately so: madness functions
    /// in hand and Aminatou grants into it, so a hidden zone is a legitimate
    /// reach. It is here for the constraint §13c decision 4 records for
    /// `backlog.md` §2.9 — a row over a hidden zone must not make that zone's
    /// order or contents observable — so whoever builds the information model
    /// has the predicate already written.
    pub fn touches_hidden_zones(self) -> bool {
        self.contains(Zone::Library) || self.contains(Zone::Hand)
    }

    /// Every zone in this set, in [`Zone`] declaration order.
    pub fn iter(self) -> impl Iterator<Item = Zone> {
        const ZONES: [Zone; 7] = [
            Zone::Library,
            Zone::Hand,
            Zone::Battlefield,
            Zone::Graveyard,
            Zone::Stack,
            Zone::Exile,
            Zone::Command,
        ];
        ZONES.into_iter().filter(move |&z| self.contains(z))
    }
}

impl std::ops::BitOr for ZoneSet {
    type Output = ZoneSet;
    fn bitor(self, rhs: ZoneSet) -> ZoneSet {
        ZoneSet(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ZoneSet {
    fn bitor_assign(&mut self, rhs: ZoneSet) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for ZoneSet {
    type Output = ZoneSet;
    fn bitand(self, rhs: ZoneSet) -> ZoneSet {
        ZoneSet(self.0 & rhs.0)
    }
}

/// What a zone change did, in the CR's word for it (`replacement-architecture.md`
/// §2b.2): a keyword action's (CR 701), a cast's (601.2a), a draw's (121.1), a
/// land play's (305.1), or [`Self::Put`], a move named only by where it goes.
///
/// This is what makes CR 701.8b answerable: `(from, to)` cannot tell a
/// sacrifice from a destruction, and 278 printed cards want "sacrifices". Who
/// or what made the move is the cause, `EventStamp`'s, and not this.
///
/// **A redirect keeps it** (CR 614.6). A replacement that changes only where the
/// move goes leaves the event the one that happens, so a card Leyline of the
/// Void exiles was still discarded (Fiery Temper's ruling). That is why there is
/// no exile act to name: "is exiled" reads `to`, and a pattern asking an act
/// for it would miss a sacrifice Rest in Peace exiled.
///
/// Three rules (`CLAUDE.md`, the chokepoint invariant):
///
/// - **The caller sets it.** `Primitive::Sacrifice` knows it is sacrificing;
///   `perform_action` cannot recover that from `(from, to)`.
/// - **Nothing may branch on it outside the replacement pipeline and the
///   trigger matcher.** A third reader is a third place for it to drift.
/// - **No catchall variant. No `Other`, no `Unknown`, no `#[non_exhaustive]`.**
///   A site with no honest act to give is a site whose act nobody has worked
///   out — see `cast.rs::rollback_cast_to_hand`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneChangeAct {
    // --- keyword actions (CR 701), one per object-moving `Primitive` ---
    /// 701.8a.
    Destroyed,
    /// 701.21a — NOT destruction.
    Sacrificed,
    /// 701.9a — includes the CR 514.1 cleanup discard.
    Discarded,
    /// 701.17a.
    Milled,
    /// A move named by where it goes and nothing else: CR 701.13a's exile
    /// ("move it to the exile zone from wherever it is"), a return, a card put
    /// into a hand or a library, and every move a rule makes — a state-based
    /// action's (CR 704.5f is "put into its owner's graveyard", which 701.8b
    /// says is not destroyed), a resolution's last step (608.2n, 608.3a, 608.3e,
    /// 608.2b) and a departed player's exiles (800.4a). The cause names the rule.
    /// Not a draw, which CR 121.5 tells apart from "put into your hand".
    Put,

    // --- the stack ---
    /// CR 601.2a's move, announced at 601.2i.
    Cast,
    /// 701.6a. Not CR 608.2b's spell whose targets are all illegal, which a
    /// rule puts into the graveyard (`Put`, `GameRule::TargetsIllegal`).
    Countered,

    // --- turn structure and special actions ---
    /// CR 121.5 makes this trigger-visibly distinct from "put into hand".
    Drawn,
    /// 305.1 / 505.6b.
    PlayedAsLand,
}

/// What destroyed a permanent (CR 701.8b).
///
/// CR 701.8b enumerates exactly two ways: "as a result of an effect that uses
/// the word 'destroy'" or "as a result of the state-based actions that check
/// for lethal damage (704.5g) or damage from a source with deathtouch
/// (704.5h)". This is that enumeration, and it is a field on
/// [`GameAction::Destroy`](crate::engine::actions::GameAction) rather than a
/// bare `Option<ObjectId>` for the reason [`ZoneChangeAct`] has no catchall:
/// `None` would be a two-variant enum spelled as an absence, and the second
/// variant's meaning would live only in a doc comment.
///
/// **It has a printed customer.** CR 122.1c's shield counter reads "If this
/// permanent would be destroyed **as the result of an effect**, instead remove
/// a shield counter from it" — a shield counter does not answer lethal damage
/// through this path at all (its *other* half, the prevention effect, stops
/// the damage before 704.5g ever asks). Without the distinction the counter
/// would save a creature twice over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestructionSource {
    /// CR 701.8b way 1 — a resolving spell or ability that uses the word
    /// "destroy". Carries the object whose effect it was.
    Effect(ObjectId),
    /// CR 701.8b ways 2 and 3 — the CR 704.5g/704.5h state-based actions. One
    /// variant for both, because CR 701.8b calls both "destroyed" and no card
    /// distinguishes lethal damage from deathtouch as a *cause*.
    StateBasedAction,
}

impl DestructionSource {
    /// The act the destruction lowers to (CR 701.8a): destroyed, either way,
    /// and the cause says which (CR 701.8b).
    pub fn zone_change_cause(self) -> ZoneChangeAct {
        match self {
            DestructionSource::Effect(_) | DestructionSource::StateBasedAction => ZoneChangeAct::Destroyed,
        }
    }
}

/// Why a player is losing life — the third of this module's "why" carriers,
/// beside [`ZoneChangeAct`] and [`DestructionSource`].
///
/// **A fact, not bookkeeping.** Whether a life loss was a result of damage is
/// unrecoverable a moment later, and three separate things read it: CR 120.3a's
/// decomposition needs the damage's source so `GameEvent::LifeChanged` can keep
/// naming it, Ali from Cairo's family watches "damage that would reduce your
/// life total" and is a `LoseLife` replacement rather than a prevention effect
/// (its own ruling: "this effect does not prevent damage, it prevents the
/// damage from turning into loss of life"), and CR 120.3b's poison counters
/// hang off the same distinction. Recording it costs one field at six
/// construction sites; re-deriving it later is impossible.
///
/// Life *gain* needs no counterpart: `GameAction::GainLife` already carries the
/// source object, because lifelink was the first thing that gained any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifeLossCause {
    /// CR 120.3a — "damage dealt to a player causes that player to lose that
    /// much life". Carries the damage's source, which is the object
    /// `GameEvent::LifeChanged` names.
    Damage { source: ObjectId },
    /// A resolving spell or ability that says "loses N life".
    Effect,
    /// CR 118.4's life payment — Phyrexian mana and the additional costs that
    /// spell it out. Not a loss of life "caused by" anything with a source.
    Cost,
}

/// Why a card is being drawn — the fourth of this module's "why" carriers.
///
/// **Ten printed cards ask, and none of them can recover the answer a moment
/// later.** "Except the first one you draw in each of your draw steps" —
/// Teferi's Ageless Insight, Notion Thief, the five *Words of* cycle and three
/// more — is a question about the *instruction*, and CR 121.1 is where the list
/// comes from: a draw "is done as a turn-based action during each player's draw
/// step. It may also be done as part of a cost or effect of a spell or
/// ability." Two arms, because those ten cards distinguish two things.
///
/// **The stamping rule is the whole of "the first one", and it needs no
/// counter.** An instruction's first individual draw carries the instruction's
/// cause and every later one is [`Self::Effect`] (CR 121.2), and a *substituted*
/// instruction keeps the cause it replaced, because CR 614.6 makes a modified
/// event occur in the original's place. So Thought Reflection doubling the draw
/// step's draw yields one `TurnBased` card and one `Effect` card, and Teferi's
/// Ageless Insight excepts the first and doubles the second — three cards, from
/// the shape of the event tree rather than from anything counting cards drawn
/// this step.
///
/// **No `Cost` arm**, although CR 121.1 names one: nothing prints "draw a card"
/// as a cost, and an arm no card reaches is worse than a missing one
/// (`replacement-architecture.md` §9, RE "Cut, and argued").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawCause {
    /// CR 121.1 / 504.1 — the draw step's turn-based action. The engine has
    /// exactly one producer, `GameState::process_draw_step`.
    TurnBased,
    /// A spell, an ability, or a replacement effect's own draw.
    Effect,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ZoneSet::HIDDEN` and `Zone::is_public` are one partition of CR 400.2's
    /// seven zones, asked of a set and of a zone.
    #[test]
    fn hidden_is_every_zone_that_is_not_public() {
        for zone in ZoneSet::ALL.iter() {
            assert_eq!(ZoneSet::HIDDEN.contains(zone), !zone.is_public(), "{zone:?}");
        }
    }
}
