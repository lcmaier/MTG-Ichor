use crate::types::ids::ObjectId;

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

/// Why the engine is moving an object between zones.
///
/// This is the semantic carrier that makes CR 701.8b answerable: `(from, to)`
/// cannot distinguish a sacrifice from a destruction from an SBA, and 1,287
/// printed cards trigger on "dies" while 278 want "sacrifices" specifically.
///
/// **Derived from call sites, not researched from the card pool.** It records
/// what the engine was doing, so the input set is finite and readable off the
/// tree (`replacement-architecture.md` §11). No printed card asks for a cause
/// finer than a call site can name: "destroyed by" appears on 1 card in all of
/// Magic, "was sacrificed" on 3, "if it was destroyed" on 0.
///
/// Three rules, all learned the hard way elsewhere in this tree:
///
/// - **The caller sets it.** `Primitive::Sacrifice` knows it is sacrificing;
///   `perform_action` cannot recover that from `(from, to)`.
/// - **Nothing may branch on it outside the replacement pipeline and the
///   trigger matcher.** A third reader is a third place for it to drift.
/// - **No catchall variant. No `Other`, no `Unknown`, no `#[non_exhaustive]`.**
///   This is the whole of what makes the enum cheap to extend later. Widening
///   is only expensive when an existing site was labelled with a coarse variant
///   that should have been finer, and re-triaging it is guesswork that fails
///   silently — which requires a catchall to lump into. A genuinely new mutation
///   arrives with its own new call site, so it adds a variant and touches
///   nothing existing. A site with no honest reason to give is a site whose
///   reason nobody has worked out, which is the bug — see
///   `cast.rs::rollback_cast_to_hand` for what that looks like when it happens.
///
/// Several variants have no call site yet because their `Primitive` is still
/// `NotImplemented` (`resolve.rs`). They are listed anyway: the enum is the
/// statement of the vocabulary, and nothing matches on it exhaustively until
/// Phase RB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneChangeCause {
    // --- effects (CR 701), one per object-moving `Primitive` ---
    /// 701.8b way 1 — an effect using the word "destroy".
    Destroyed,
    /// 701.21 — NOT destruction. The distinction 278 cards care about.
    Sacrificed,
    /// 701.13.
    Exiled,
    /// 701.9 — includes the CR 514.1 cleanup discard.
    Discarded,
    /// 701.17.
    Milled,
    /// "return to hand" / "return to the battlefield" — an object going *back*
    /// somewhere. Not the same as [`Self::PutIntoHand`]; see there.
    Returned,
    /// A card put into a hand from somewhere it was never in — the CR 121.5
    /// non-draw, and specifically **not** a draw: Nadu, Winged Wisdom ("reveal
    /// the top card of your library … otherwise, put it into your hand"),
    /// impulse-style "look at the top N and put one in your hand".
    ///
    /// 857 cards move library→hand without the word "draw" (493 say "put it
    /// into your hand"), so this is a class, not a corner.
    ///
    /// **Kept separate from `Returned` on naming honesty, not on card demand.**
    /// The merge criterion in §11 asks whether a printed card distinguishes two
    /// reasons *as a cause*, and by that test these would collapse — nothing
    /// asks "was it returned or put". They stay apart because `Returned` means
    /// an object going back where it was, a Nadu card was never in hand, and a
    /// site labelled with a variant whose name does not describe it is exactly
    /// the coarse-label failure the no-catchall ban exists to prevent. The cost
    /// of the extra variant is zero while nothing matches exhaustively.
    ///
    /// Note this carries no draw/non-draw meaning by itself — `GameEvent::CardDrawn`
    /// is what CR 121.5 turns on. This is the *reason for the move*.
    PutIntoHand,
    /// Top, bottom, or shuffled in. *Position* is a field, not a cause.
    PutIntoLibrary,

    // --- state-based actions (CR 704.5) ---
    /// 704.5g lethal damage + 704.5h deathtouch. One variant, because CR 701.8b
    /// calls both "destroyed" and no card distinguishes them as a *cause*.
    DestroyedBySba,
    /// 704.5f — NOT destruction, so regeneration and indestructible do not help.
    ZeroToughness,
    /// 704.5i.
    ZeroLoyalty,
    /// 704.5j.
    LegendRule,
    /// 704.5m. (704.5n only unattaches an Equipment; it moves nothing.)
    AuraSba,

    // --- the stack ---
    /// Hand (or elsewhere) → stack.
    Cast,
    /// A stack object finished resolving: CR 608.2n for an instant or sorcery
    /// (to its owner's graveyard), CR 608.3a/c for a permanent spell (onto the
    /// battlefield, attached if it is an Aura).
    Resolved,
    /// 701.6.
    Countered,
    /// 608.2b — countered by game rules, all targets illegal.
    Fizzled,

    // --- Commander (CR 903) ---
    /// CR 903.9b — a commander that would go to its owner's hand or library
    /// goes to the command zone instead, if its owner chooses. A *replacement*,
    /// and the rules' only stated exception to CR 614.5.
    CommanderZoneReplacement,
    /// CR 704.6d / 903.9a — a commander in a graveyard or exile is moved to the
    /// command zone by a **state-based action**, not by a replacement effect.
    /// Two rules, two variants: the engine's reason for the move differs even
    /// though the destination does not.
    CommanderZoneSba,

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
/// bare `Option<ObjectId>` for the reason [`ZoneChangeCause`] has no catchall:
/// `None` would be a two-variant enum spelled as an absence, and the second
/// variant's meaning would live only in a doc comment.
///
/// **It has a printed customer, which is why it exists in Phase RB rather than
/// waiting.** CR 122.1c's shield counter reads "If this permanent would be
/// destroyed **as the result of an effect**, instead remove a shield counter
/// from it" — a shield counter does not answer lethal damage through this
/// path at all (its *other* half, the prevention effect, stops the damage
/// before 704.5g ever asks). Without the distinction the counter would save a
/// creature twice over.
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
    /// The zone-change cause the destruction lowers to (CR 701.8a).
    pub fn zone_change_cause(self) -> ZoneChangeCause {
        match self {
            DestructionSource::Effect(_) => ZoneChangeCause::Destroyed,
            DestructionSource::StateBasedAction => ZoneChangeCause::DestroyedBySba,
        }
    }
}
