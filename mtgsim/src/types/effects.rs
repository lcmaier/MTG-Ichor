use super::colors::Color;
use super::ids::{ObjectId, PlayerId};
use super::keywords::KeywordFlag;
use super::mana::{ManaAtom, ManaType};
use super::zones::ZoneSet;
use crate::state::game_state::PhaseType;

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

/// How numeric amounts are determined at resolution time (rule 608.2h)
#[derive(Debug, Clone, PartialEq)]
pub enum AmountExpr {
    /// A constant known at definition time
    Fixed(u64),
    /// X, chosen when the spell/ability is cast/activated (rule 107.3)
    Variable,
    /// "equal to the number of [things matching selector]"
    CountOf(Selector),
    /// "equal to the number of card **types** among [things matching selector]"
    /// — the Lhurgoyf family. Distinct from `CountOf`, which counts objects:
    /// ten artifact creatures in a graveyard are ten cards but two card types.
    CardTypesAmong(Selector),
    /// "equal to that number plus N" — Tarmogoyf's toughness.
    Plus(Box<AmountExpr>, u64),
    /// "equal to its mana value", where "it" is the object the continuous
    /// effect is being applied to — March of the Machines' "power and toughness
    /// each equal to its mana value".
    ///
    /// Read off the affected object's *effective* mana cost mid-layer-walk, not
    /// off `CardData`: a Layer 1 copy effect changes mana cost, and CR 202.3b
    /// makes an object with no mana cost mana value 0.
    ///
    /// Distinct from `TargetPower` and friends below, which are resolution-time
    /// and read the target of a spell. This one has meaning in a static context,
    /// which is what lets `compute.rs` evaluate it at every layer.
    AffectedManaValue,
    /// "equal to **this** creature's power" — the power of the object whose
    /// ability this is, not of anything the ability points at.
    ///
    /// Master Biomancer's "a number of additional +1/+1 counters on it equal to
    /// this creature's power". Distinct from [`Self::TargetPower`], which reads
    /// a resolved target, and from [`Self::AffectedManaValue`], which reads the
    /// object a continuous effect is being *applied to*: this one reads the
    /// source, which for a replacement effect is a permanent somewhere else on
    /// the board.
    ///
    /// **Whose board is the whole question, and only one evaluator answers it.**
    /// `replacement::evaluate_enter_amount` reads the CR 614.12 frame when the
    /// source is the entering permanent and the real battlefield otherwise
    /// (`replacement-architecture.md` §5b). The layer walk and the resolution
    /// evaluator both refuse it rather than guessing at a source they were not
    /// given.
    SourcePower,
    /// "That many" on a triggered ability — the amount the matched records
    /// carry, summed over them (`triggers-architecture.md` §3.4): the damage
    /// dealt, the mana added. Read through the arm's `amount_of`, so an arm
    /// with no quantity refuses rather than answering 0.
    TriggeringAmount,
    /// "equal to that creature's power"
    TargetPower,
    /// "equal to that creature's toughness"
    TargetToughness,
    /// "equal to the damage dealt this way"
    DamageDealt,
    /// CR 615.5's "that much" / "that many" — the amount the *replaced* event
    /// carried when a CR 615.5 rider was queued.
    ///
    /// Angel of Suffering's "prevent that damage and mill twice that many
    /// cards": "that many" is the damage that would have been dealt, read
    /// before the prevention emptied the event. Only a rider has one, so every
    /// other evaluator refuses it rather than reading a number off the board —
    /// there is none to read.
    ///
    /// Distinct from [`Self::DamageDealt`], which is a resolving spell's
    /// question about damage it dealt itself.
    ReplacedAmount,
    /// "twice that many" — a factor on another amount.
    ///
    /// [`Self::Plus`]'s multiplicative twin, and it arrives with the same kind
    /// of customer: Angel of Suffering is `Multiply(ReplacedAmount, 2)`. Not a
    /// general `Times(Box, Box)`, because nothing printed multiplies one
    /// computed amount by another.
    Multiply(Box<AmountExpr>, u64),
    /// "The amount of … unspent mana you have" of one type — Doubling Cube's
    /// "double the amount of each type of unspent mana you have", read off
    /// the activating player's pool when the ability resolves, restricted
    /// units included: CR 106.6 says a restriction "doesn't affect the mana's
    /// type", and the Cube's ruling counts {U}{U}{U} spendable only on
    /// artifact spells as three blue.
    ///
    /// **Its own arm, on this enum's own convention**: one arm per printed
    /// quantity, evaluated where it is read. Not a [`Self::CountOf`], because a
    /// [`Selector`] selects *objects* and mana is not one. A category rather
    /// than one card's: eight printed cards read unspent mana (Scryfall,
    /// 2026-09-15), five as an amount. Two axes wait for the cards that need
    /// them, each a compiler-forced field here and one arm at the evaluator:
    /// a **total across types** (Glissa Sunseeker) as an `Option<ManaType>`,
    /// and **another player's pool** (Drain Power, Pygmy Hippo) as a
    /// `PlayerRef`. Omnath, Locus of Mana reads the same quantity on the layer
    /// side, where `layers::compute` would evaluate it.
    UnspentMana(crate::types::mana::ManaType),
    /// CR 615.5's "the amount of damage that was prevented" — how much the
    /// prevention effect that queued a CR 615.5 rider actually prevented.
    ///
    /// **A different number from [`Self::ReplacedAmount`], and CR 615.12 is
    /// what tells them apart.** Reverse Damage gains "life equal to the damage
    /// prevented this way"; Angel of Suffering mills "twice that many", where
    /// "that many" is the damage that *would have been dealt*. Under damage
    /// that can't be prevented both riders run (615.12), Angel still mills
    /// and Reverse Damage gains 0 — so the rider carries both numbers and each
    /// leaf reads its own. A 0 here makes the rider's `GainLife` or
    /// `DealDamage` a `never_happens` non-event (CR 614.7a, 119.10), which is
    /// why "if damage is prevented this way" needs no `Effect::Conditional`.
    ///
    /// Only a rider has one; every other evaluator refuses it.
    DamagePrevented,
    /// CR 103.3's starting life total — "your life total becomes equal to
    /// your starting life total" (Exquisite Archangel). A leaf rather than a
    /// `Fixed(20)` because v1 is Commander, where it is 40, and the number is
    /// the game's (`GameState::starting_life`) and not the card's.
    StartingLifeTotal,
}

/// Which objects an effect queries or iterates over
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    ControlledCreatures,
    CreaturesInGraveyard(PlayerRef),
    PermanentsMatching(ObjectFilter),
    CardsInHand(PlayerRef),
    /// Cards in graveyards. `None` means **all** graveyards — Tarmogoyf's "cards
    /// in all graveyards", which includes a Tarmogoyf sitting in one of them.
    ///
    /// One variant rather than a separate `CardsInAllGraveyards`: "whose
    /// graveyard" is a parameter of the same concept, and a second variant made
    /// the two look like different questions. Only `None` has an evaluator today
    /// — every `Some` form is unused by any card, and `compute::evaluate_amount`
    /// asserts rather than guessing at `PlayerRef` resolution nothing needs yet.
    CardsInGraveyard(Option<PlayerRef>),
}

/// Reference to a player in an effect context
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerRef {
    /// The controller of the spell/ability
    You,
    /// A targeted or otherwise identified opponent
    Opponent,
    /// Owner of the source object
    Owner,
    /// A specific player
    Player(PlayerId),
}

/// CR 701.9b — who picks which card is discarded.
///
/// > 701.9b By default, effects that cause a player to discard a card allow the
/// > affected player to choose which card to discard. Some effects, however,
/// > require a random discard or allow another player to choose which card is
/// > discarded.
///
/// **Two arms for the rule's three shapes, and the third waits for its card.**
/// "Another player chooses" is Coercion's, which also needs the reveal §2.9
/// owns, and an arm no card can reach is worse than a missing one
/// (`replacement-architecture.md` §3.2a). Adding it is one variant and one
/// branch in the performer, with the player to ask as its payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscardChooser {
    /// 701.9b's default — the discarding player chooses. Mind Rot.
    Affected,
    /// "…at random". Hymn to Tourach. Drawn from `GameState::rng` and never
    /// from `rand::rng()`, so a seeded game replays (`CLAUDE.md`: randomness
    /// is owned, never ambient), and asked of no `DecisionProvider` at all,
    /// which is `ATOM-701.9b-001`'s own expected result.
    AtRandom,
}

/// A filter over an object's characteristics — type, subtype, supertype,
/// color, controller — plus the two object facts no layer reaches (`Token`,
/// `ByOwner`) and one relation to the filter's source (`NotSource`).
///
/// Named for what it matches: creature cards in a graveyard, cards in
/// libraries and spells on the stack, none of which is a permanent
/// (CR 110.1). The zone leaf `codebase-state.md`'s layers item 9 wants on
/// this type is still that item's.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectFilter {
    All,
    ByType(crate::types::card_types::CardType),
    BySubtype(crate::types::card_types::Subtype),
    BySupertype(crate::types::card_types::Supertype),
    ByColor(Color),
    ByController(PlayerRef),
    /// Power less than or equal to N (for "creature with power N or less")
    PowerLE(i32),
    /// CR 111.1 — the permanent is a token. "**Nontoken**" is `Not(Token)`.
    ///
    /// Its own leaf, and it earns its place
    /// on breadth rather than on one card: "nontoken" is a printed quality on
    /// hundreds of cards (Kalitas, Anointed Procession's mirror image, every
    /// "nontoken creature you control" anthem), and it is not derivable from
    /// any other leaf — `is_token` is a property of the *object*, not of its
    /// characteristics, so no combination of type, color or controller
    /// reaches it.
    ///
    /// It is a leaf rather than a `Not`-only helper because `Not` already
    /// composes; adding `Nontoken` as well would give one quality two spellings.
    Token,
    /// CR 108.3 — the player who started the game with the card in their deck.
    ///
    /// **Not `ByController` with extra steps.** A card put into a graveyard goes
    /// to its *owner's* graveyard (CR 400.3), so every card whose text says
    /// "an opponent's graveyard" is asking this question and not the control
    /// question. The two answers diverge whenever control has moved, which the
    /// registered pool can already reach: Act of Treason steals a creature, it
    /// dies, and it goes to the graveyard of the player who owns it.
    ///
    /// Read off the `GameObject` rather than the layer frame, for the reason
    /// `Token` gives — ownership is not a characteristic, so no layer can
    /// change the answer and there is nothing on `chars` to consult.
    ByOwner(PlayerRef),
    /// "Each **other** ..." — every permanent but the effect's own source.
    ///
    /// Opalescence's "each other non-Aura enchantment", and the word on
    /// Mirrorweave's shape that `CopyRoles::exclude_donor` carries as a
    /// field; here it is a leaf because it composes with the rest of the
    /// filter. Identity is not a characteristic, so it is answered from the
    /// object id and no layer can change it. Meaningful only where the filter
    /// has a source — a static ability's affected set, a CDA's count — and
    /// the selection-side matcher refuses it rather than guessing one.
    ///
    /// **The source is the object that *has* the ability, never the one that
    /// granted it** (CR 113.7 — "the object whose ability triggered"). A
    /// granted or copied "another creature" is other than its *carrier*: the
    /// trigger matcher answers this leaf against the candidate it is asking
    /// about (`dispatch::subject_matches` passes `candidate.id`), so a Layer 6
    /// grant of Soul Warden's ability excludes the creature that has it and
    /// never the object that granted it.
    NotSource,
    /// CR 601.2c — "**another** target creature", "a **third** target
    /// creature": not what an earlier instance of "target" on this same spell
    /// took.
    ///
    /// The rule allows one object to be chosen once for each instance "as long
    /// as it fits the targeting criteria", and "another" is how a card puts the
    /// exclusion *into* those criteria. Incremental Growth's three clauses are
    /// `Creature`, `And(Creature, OtherThanInstance(0))` and that filter with
    /// `OtherThanInstance(1)` as well.
    ///
    /// [`NotSource`](Self::NotSource)'s sibling, and answered the same way:
    /// identity is not a characteristic, so no layer can change it and no
    /// frame is read. Meaningful only where the asker holds the instances
    /// chosen so far — the CR 601.2c loop and the CR 608.2b re-check — and
    /// refused elsewhere rather than silently matching everything.
    OtherThanInstance(usize),
    And(Box<ObjectFilter>, Box<ObjectFilter>),
    /// Added for Root Maze, "Artifacts and lands enter tapped" — English "and"
    /// over two type leaves is set *union*, which is this node.
    ///
    /// Derivable as `Not(And(Not(a), Not(b)))` and deliberately not written
    /// that way: a card definition is read by whoever is checking it against
    /// the oracle text, and De Morgan's law is not something a reader should
    /// have to undo to see that a filter is right.
    Or(Box<ObjectFilter>, Box<ObjectFilter>),
    Not(Box<ObjectFilter>),
}

/// Selects which objects an effect applies to — read by the layer walk, the
/// restriction sweep and the replacement pipeline, which is why the name is
/// the set's contents rather than any one reader's verb.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectSet {
    /// The source permanent itself ("this creature has flying").
    SourceOnly,
    /// A data-driven filter ("creatures you control").
    ///
    /// **The filter is stored unresolved.** `ObjectFilter::ByController`
    /// carries a `PlayerRef`, and `compute::object_matches_filter` resolves it
    /// during the layer walk against the source's *effective* controller:
    /// CR 109.5 makes a static ability's "you" the **current** controller of the
    /// object it's on, so a snapshot taken when the source entered is wrong the
    /// moment control of the source changes.
    ///
    /// **`zones` is which zones the filter reaches**, and it is read at a
    /// different time from `filter`: `Board::seed` asks it once per pass to
    /// decide the working set, before any layer runs, while `filter` is asked
    /// per layer per candidate. That is why it is a field here and not a leaf
    /// inside [`ObjectFilter`] — reach must be readable syntactically for the
    /// fast path to be sound, which a `Not` inside a filter tree makes
    /// impossible (`layers-architecture.md` §13c decision 3).
    ///
    /// Build with [`ObjectSet::filter`] for the battlefield (almost every row)
    /// or [`ObjectSet::filter_in`] for a row that reaches further.
    Filter { filter: ObjectFilter, zones: ZoneSet },
    /// A fixed set captured at effect creation time.
    /// Pump spells use this — the target is locked at resolution.
    Fixed(Vec<ObjectId>),
    /// Whatever the source is attached to *now* — CR 303.4m's "enchanted
    /// creature". Resolved during the walk off the source's `attached_to`,
    /// never captured, for the same reason `Filter` stores its `PlayerRef`
    /// unresolved: `register_static_effects` runs inside
    /// `place_on_battlefield`, before the resolution attaches the Aura, so a
    /// snapshot would be empty for every Aura ever cast. An unattached source
    /// names nothing.
    Host,
}

impl ObjectSet {
    /// A set with no objects in it.
    ///
    /// Named rather than written as `Fixed(Vec::new())` at a call site, where
    /// an empty vector reads as an oversight rather than as a claim. Two kinds
    /// of effect mean it: a replacement effect that is about a **player** and
    /// no object (Angel of Suffering's "if damage would be dealt to you",
    /// paired with a [`PlayerSet`]), and a resolution whose targets have all
    /// left before the row was written.
    ///
    /// A constant and **not** an `ObjectSet::Nobody` variant: a new arm would
    /// have to be classified by all three of this type's readers — the layer
    /// walk, the restriction sweep and the replacement pipeline — where a
    /// spelling of an existing value costs them nothing.
    pub const NO_OBJECTS: ObjectSet = ObjectSet::Fixed(Vec::new());

    /// A filter over the battlefield — what "creatures you control" means, and
    /// what every row meant before the zone field existed.
    ///
    /// Named rather than written as a struct literal at ~60 call sites: the
    /// battlefield is the overwhelming default, and a constructor that says so
    /// keeps [`ObjectSet::filter_in`] visibly exceptional at the handful of
    /// sites that reach another zone. **`battlefield_filter` and not `filter`**
    /// (owner review, 2026-09-14): a bare `filter` reads as "the filter case"
    /// rather than as one of two, which is the reading that would let a
    /// zone-reaching row be written battlefield-scoped by accident.
    pub fn battlefield_filter(filter: ObjectFilter) -> ObjectSet {
        ObjectSet::Filter { filter, zones: ZoneSet::BATTLEFIELD }
    }

    /// A filter that reaches beyond the battlefield — Yixlid Jailer's "cards in
    /// graveyards", Mycosynth Lattice's "all cards that aren't on the
    /// battlefield".
    ///
    /// The zones are named positively and cannot be a complement; see
    /// [`ZoneSet`].
    pub fn filter_in(filter: ObjectFilter, zones: ZoneSet) -> ObjectSet {
        ObjectSet::Filter { filter, zones }
    }

    /// Which zones this set can name an object in — the union `Board::seed`
    /// and `RegistryScopeSummary` read.
    ///
    /// **Only `Filter` can answer from its own shape.** `Fixed` names objects
    /// directly and they are members wherever they are (`Board::seed` adds
    /// them unconditionally, and has since before this field existed);
    /// `SourceOnly` and `Host` name a permanent. So the other three report
    /// `EMPTY` — they add no *zone* to sweep, which is a different statement
    /// from adding no members.
    pub fn reachable_zones(&self) -> ZoneSet {
        match self {
            ObjectSet::Filter { zones, .. } => *zones,
            ObjectSet::SourceOnly | ObjectSet::Fixed(_) | ObjectSet::Host => ZoneSet::EMPTY,
        }
    }
}

/// Which **players** a replacement or prevention effect applies to — CR 614.1's
/// "whatever they're affecting", for the half [`ObjectSet`] cannot name.
///
/// **A second field on `ReplacementDef`, not an `ObjectSet` variant.** That
/// type has three readers — the layer walk's `row_affected`, the restriction
/// sweep and the replacement pipeline — and a `Player` arm would be a variant
/// two of the three must reject at every match. The two sets union: an event
/// about an object asks `ObjectSet`, an event about a player asks this one.
/// The damage family is what makes it necessary — Furnace of Rath's "a
/// permanent **or player**", Circle of Protection's "damage that would be
/// dealt to **you**" (23 printed), Fog's "all combat damage"
/// (`replacement-architecture.md` §9, RD's decision 0).
///
/// Resolved against the instance's controller exactly as
/// [`ObjectFilter::ByController`]'s [`PlayerRef`] is (CR 109.5): "you" is the
/// source's *current* controller, never a snapshot taken when it entered.
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerSet {
    /// No player at all — every object-scoped effect, and the default.
    Nobody,
    /// The effect's controller (CR 109.5).
    You,
    /// Everyone else. CR 102.1's "opponent" is every other player: this engine
    /// has no teams (CR 102.3), so "not you" is the whole answer and stays it
    /// until it does.
    Opponents,
    /// Every player, including the controller — Furnace of Rath, Fog.
    Everyone,
    /// A set captured when the effect was created, the way
    /// [`ObjectSet::Fixed`] captures objects: a resolution filling in the
    /// player it targeted, or an empty set for a static ability that scopes by
    /// [`Self::You`] and names no object.
    Fixed(Vec<PlayerId>),
}

impl PlayerSet {
    /// Is `player` in this set, for an effect controlled by `controller`?
    ///
    /// Needs no board: every arm is answerable from the two ids, which is what
    /// keeps this on the data type rather than in the pipeline.
    pub fn contains(&self, controller: PlayerId, player: PlayerId) -> bool {
        match self {
            PlayerSet::Nobody => false,
            PlayerSet::You => player == controller,
            PlayerSet::Opponents => player != controller,
            PlayerSet::Everyone => true,
            PlayerSet::Fixed(ids) => ids.contains(&player),
        }
    }
}

/// Duration for continuous effects (rule 611)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duration {
    /// "until end of turn"
    UntilEndOfTurn,
    /// "until your next turn"
    UntilYourNextTurn,
    /// As long as the source permanent is on the battlefield (static abilities)
    WhileSourceOnBattlefield,
    /// As long as the permanent is enchanted by the source
    WhileEnchanted,
    /// As long as the permanent is equipped by the source
    WhileEquipped,
    /// Lasts until end of game (or until removed)
    Indefinite,
}

/// Conditions for Conditional effects (rule 603.4 intervening "if")
///
/// Shared with a static ability's "as long as [X]" (CR 604.2): a conditional
/// static's effect exists exactly while its condition holds, and
/// `engine::layers::condition::holds` is the evaluator that says so during
/// the layer pass. That sharing is `layers-architecture.md` §13b decision 5 —
/// the enum is extended, never duplicated, and grows a leaf only when a
/// registered card needs one.
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    ControlPermanent(ObjectFilter),
    LifeAtLeast(AmountExpr),
    LifeAtMost(AmountExpr),
    OpponentControlsPermanent(ObjectFilter),
    CardInGraveyard(ObjectFilter),
    SpellWasKicked,
    ModeChosen(usize),
    /// CR 113.6b's clause — "as long as this card is in your graveyard"
    /// (Wonder), "if this card is in your graveyard" (Bridge from Below).
    ///
    /// **Two jobs, and the CR is what makes them one.** Evaluated at every
    /// layer like any other leaf, so the effect stops existing the moment the
    /// source leaves; and read *syntactically* by
    /// `engine::zone_function::functioning_zones`, because CR 113.6b says an
    /// ability that states which zones it functions in functions only from
    /// those zones — so the clause that gates the effect is the same sentence
    /// that places the ability. A condition about some *other* object's zone
    /// is [`Self::CardInGraveyard`], which is why this one says `Source`.
    ///
    /// Replaced `SourceOnBattlefield`, which was this question narrowed to one
    /// zone: the battlefield is `SourceInZone(ZoneSet::BATTLEFIELD)` and reads
    /// the same gate it always did.
    SourceInZone(ZoneSet),
    /// Every clause holds — Wonder's "in your graveyard **and** you control an
    /// Island".
    ///
    /// `layers-architecture.md` §15.1 planned `And`/`Or`/`Not` and Wonder is
    /// the first registered card to need any of them, which is §13b decision
    /// 5's rule for when the enum grows. The other two are not written ahead
    /// of a card, and `Not` in particular is the one
    /// `zone_function::stated_zones` will have to refuse rather than guess at
    /// (§13d decision 1b).
    ///
    /// An empty vector is vacuously true, which is what `all()` means and what
    /// no card writes.
    All(Vec<Condition>),
    /// "as long as enchanted/equipped [permanent] is [X]" — a predicate on
    /// whatever the source is attached to (CR 303.4m reads it fresh, as
    /// `ObjectSet::Host` does). Rune of Flight's two clauses are both this
    /// leaf. `Host` rather than `AttachedTo` for the reason §13a decision 4
    /// gives: one word for one relationship across the whole engine.
    ///
    /// False when the source is attached to nothing, which is what makes an
    /// unattached Aura's conditional effect simply not exist.
    HostMatches(ObjectFilter),
    /// "As long as this artifact is untapped" — Trinisphere, Winter Orb,
    /// Static Orb. Tapped is a status (CR 110.5), not a characteristic, so
    /// no layer writes it and `board::condition_reads` declares nothing for
    /// it: it can never be a CR 613.8 dependency.
    SourceUntapped,
    /// "While your library has no cards in it" — Laboratory Maniac, and its
    /// planeswalker twin Jace, Wielder of Mysteries. "Your" is CR 109.5's
    /// controller of the source, read the way `LifeAtLeast` reads it.
    ///
    /// **Written for one card and says so.** The leaf's first reader is a
    /// replacement effect's "as long as", evaluated by `replacement::gather`
    /// at each proposal through `condition::settled_holds` — the same
    /// evaluator the layer pass and CR 613.11's cost effects use, so a
    /// conditional static's condition is one question wherever it is asked.
    /// A library is off `GameState`, not off any frame, so
    /// `board::condition_reads` declares nothing for it.
    LibraryEmpty,
}

/// How many modes to choose (rule 700.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalCount {
    Exactly(u32),
    UpTo(u32),
    Any,
}

/// What an effect acts on.
///
/// Two orthogonal concerns, kept apart:
/// - **Who/what** the effect acts on (filter + count)
/// - **Whether targeting rules apply** (hexproof/shroud/protection)
///
/// `Target` = the MTG rules concept of "targeting" (hexproof, shroud,
/// protection all apply; fizzles if target becomes illegal).
/// `Choose` = "choose" / non-targeting selection (rule 303.4a — Aura ETB
/// without casting; hexproof/shroud do NOT apply, does NOT fizzle).
#[derive(Debug, Clone, PartialEq)]
pub enum EffectRecipient {
    /// No object involved (e.g. mana abilities, "draw a card" with no target)
    Implicit,
    /// The controller of this spell/ability (e.g. Night's Whisper "you draw",
    /// Angel's Mercy "you gain"). Not targeting.
    Controller,
    /// Select with targeting rules — hexproof/shroud/protection apply,
    /// fizzles if all targets become illegal (rule 608.2b).
    Target(SelectionFilter, TargetCount),
    /// Select without targeting rules — "choose" (rule 303.4a, etc.).
    /// Hexproof/shroud/protection do NOT apply.  Does not fizzle.
    Choose(SelectionFilter, TargetCount),
    /// CR 115.3 — an instance of "target" (or of a non-targeting "choose")
    /// that an **earlier atom of this same effect** already announced.
    ///
    /// `Target` and `Choose` each *declare* an instance; this refers back to
    /// one, by its position in [`Effect::instances`]' pre-order list.
    ///
    /// **An atom is an *effect*, not a clause**, and that is why the card has
    /// to say this rather than the engine working it out. The tree records what
    /// happens, not how many times the card said "target", and effects per
    /// clause is whatever the wording needs.
    ///
    /// Two registered cards make it exact. Written the way this crate encoded
    /// cards before A4i — each atom carrying the clause it acts on — these are
    /// the **same tree**:
    ///
    /// ```text
    /// Act of Treason     Sequence[ Atom(GainControl,          Target(Creature, Exactly(1))),
    ///                              Atom(Untap,                Target(Creature, Exactly(1))),
    ///                              Atom(GrantKeywordFlag,     Target(Creature, Exactly(1))) ]
    ///
    /// Seeds of Strength  Sequence[ Atom(ModifyPowerToughness, Target(Creature, Exactly(1))),
    ///                              Atom(ModifyPowerToughness, Target(Creature, Exactly(1))),
    ///                              Atom(ModifyPowerToughness, Target(Creature, Exactly(1))) ]
    /// ```
    ///
    /// Three atoms each, every recipient identical, and the only field that
    /// differs is the `Primitive` — which says *what happens*, never *to whom*.
    /// **Act of Treason is one instance and Seeds of Strength is three**,
    /// because Act of Treason prints "target creature" once and says "that
    /// creature" and "it" afterwards. No rule reading the tree separates them:
    /// the difference was never in the tree. It is in the card's text, which is
    /// what this variant carries.
    ///
    /// The atom resolves against the declaring instance's targets *and* its
    /// recipient, so `resolve_player_for_self` and the filtered-sweep arms see
    /// what the declaring atom saw.
    ///
    /// **Named for how it reads at a call site.** `Instance(0)` says "this
    /// atom's recipient *is* instance 0"; what it means is "this atom *reuses*
    /// the instance declared at 0", which is what a card author needs to see
    /// without opening this file.
    SameInstanceAs(usize),
    /// "That creature", "it" on a triggered ability — the record's subject,
    /// found by id **and** epoch (CR 603.6's "unable to be found", CR 400.7 in
    /// one comparison): a creature that died and was returned before the
    /// trigger resolves is a new object and this resolves to nothing.
    TriggeringObject,
    /// "That player" on a triggered ability — the arm's `player_of` on the
    /// matched records.
    TriggeringPlayer,
    /// Filter-based recipient: every permanent matching the filter.
    ///
    /// Read by the ETB hook to register a static ability's continuous effect,
    /// and at **resolution** by the primitives that act on more than one object
    /// at once: `CreateReplacement` makes one row per matching permanent
    /// (CR 615.11) and `DealDamage` proposes one batch member each. All three
    /// resolve it the same way, `battlefield_ids_ordered` filtered against the
    /// resolution's controller **now**, never captured.
    ///
    /// **The filter is not written into `ResolutionContext::targets`**: "each
    /// creature" is not a targeting fact (CR 115.1 announces targets as the
    /// spell is cast), and filling the targets would change what this variant
    /// means to the static-ability path that shares it.
    ///
    /// Use `ByController(PlayerRef::You)` in the filter to express "you control".
    /// The filter is stored verbatim; `compute::object_matches_filter`
    /// resolves the `PlayerRef` during the layer walk (CR 109.5).
    FilteredPermanents(ObjectFilter),
    /// Every object matching the filter, in any of the named zones — Yixlid
    /// Jailer's "cards in graveyards", Mycosynth Lattice's "all cards that
    /// aren't on the battlefield".
    ///
    /// **The general form; [`Self::FilteredPermanents`] is the battlefield
    /// spelling and stays the one to use for permanents**, exactly as
    /// [`ObjectSet::filter`] is the battlefield spelling of
    /// [`ObjectSet::filter_in`] one layer down. They lower through the same
    /// function to the same value, so there is nowhere for them to disagree.
    ///
    /// **Static abilities only.** A resolution primitive that sweeps this
    /// would need a zone sweep at resolution time, and no printed card asks
    /// for one, so `resolve.rs` refuses it by name rather than falling into a
    /// catch-all — an arm the engine cannot apply is worse than a missing one.
    FilteredObjectsIn(ObjectFilter, ZoneSet),
    /// The permanent this one is attached to — "enchanted creature" on an
    /// Aura (CR 303.4m), "equipped creature" on an Equipment (CR 301.5a).
    /// Static abilities only, like `FilteredPermanents`; lowers to
    /// `ObjectSet::Host`, which reads `attached_to` during the
    /// layer walk rather than capturing it, because registration happens
    /// before the attach.
    Host,
}

/// What kind of object(s) can be selected.
///
/// Shared by both `Target` and `Choose` variants of `EffectRecipient`.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionFilter {
    /// Creature on the battlefield
    Creature,
    /// Player
    Player,
    /// "any target" — creature, player, or planeswalker
    Any,
    /// Permanent matching a filter
    Permanent(ObjectFilter),
    /// Spell on the stack
    Spell,
    /// CR 609.7a — a **source of damage**: any permanent, or any spell on the
    /// stack.
    ///
    /// > 609.7a If an effect requires a player to choose a source of damage,
    /// > they may choose a permanent; a spell on the stack (including a
    /// > permanent spell); any object referred to by an object on the stack, by
    /// > a replacement or prevention effect that's waiting to apply, or by a
    /// > delayed triggered ability that's waiting to trigger ...; or a face-up
    /// > object in the command zone. **A source doesn't need to be capable of
    /// > dealing damage to be a legal choice.**
    ///
    /// The last sentence is free — the enumeration does not ask — and it is
    /// the reason this is its own filter rather than `Any` with a wider net:
    /// `Any` is CR 115.4's target list, which is about what can be *dealt*
    /// damage.
    ///
    /// **Two of the rule's four categories are reachable and two are not**,
    /// and the gap is deliberate: "an object referred to by an object on the
    /// stack / by a waiting replacement / by a delayed trigger" needs a
    /// referred-to relation the engine has nowhere to read (delayed triggers
    /// are CR 603.7's and do not exist), and "a face-up object in the command
    /// zone" needs the command zone populated, which is the Commander track's.
    /// `ATOM-609.7a-001` is `COVERS-PARTIAL` for exactly these two.
    DamageSource,
}

/// What a [`Primitive::CreateReplacement`] fills into the def's **pattern** at
/// resolution, beside what the recipient fills into its affected set.
///
/// **One mechanism for one rule.** CR 609.7a's "the source is chosen when the
/// effect is created" is the only thing in the CR that puts a *resolution's*
/// choice inside a pattern rather than inside an affected set, and the two
/// halves genuinely differ: Circle of Protection: Red's row is around **you**
/// (its recipient) and watches damage from **one chosen object** (this). A
/// `Choose` recipient could carry the object but would then have nothing left
/// to say what the row is about.
///
/// A closed enum with two arms rather than a `bool`, because the arm names the
/// rule it serves; a second arm needs a second CR rule that puts a resolution's
/// choice in a pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternFill {
    /// Nothing is asked; the pattern is the card's, as written.
    Authored,
    /// CR 609.7a — the row's controller chooses a source of damage as the
    /// effect is created, and it is written into the def's
    /// `EventPattern::DealDamage`'s `source.object`.
    ///
    /// The card authors that field as `None` and the resolution overwrites it,
    /// which is [`Primitive::Restrict`]'s shape one level down: the *shape* is
    /// the card's and the *object* is the resolution's.
    ChosenDamageSource,
}

/// How many targets/choices to select
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetCount {
    Exactly(u32),
    UpTo(u32),
}

/// Mana output from a mana ability or mana-producing spell.
///
/// Dual-track, mirroring `ManaPool`:
/// - `mana`: fast-path unrestricted mana (added via `pool.add()`)
/// - `special`: sidecar atoms with restrictions, grants, or persistence
///   (added via `pool.add_special()`)
///
/// Amounts use `AmountExpr` so they can be evaluated at resolution time
/// (e.g. "Add {G} equal to target creature's power" → `TargetPower`).
/// Most cards use `AmountExpr::Fixed`.
///
/// Most cards only use `mana`. Cards like Cavern of Souls use `special`.
#[derive(Debug, Clone, PartialEq)]
pub struct ManaOutput {
    pub mana: Vec<(ManaType, AmountExpr)>,
    pub special: Vec<ManaAtom>,
}

/// Zone filter for Search effects
#[derive(Debug, Clone, PartialEq)]
pub enum ZoneFilter {
    Library,
    Graveyard,
    Exile,
}

/// What a token is created *as* — CR 111.3's "text", the characteristics
/// the creating effect defines.
///
/// **A description, not a card.** CR 111.4 says the effect *sets* the
/// token's name and subtypes, and CR 111.3 that a token "doesn't have any
/// characteristics not defined by the spell or ability that created it";
/// so this carries exactly what an effect can say and nothing a card has
/// that a token cannot — no mana cost (CR 111.6), no cast-time costs, no
/// color indicator. [`Self::card_data`] lowers it into the `CardData` a
/// `GameObject` reads, once per creation.
///
/// Every field a printed token needs is here. What is *not* here is
/// CR 111.10's twenty predefined tokens, which are constructors of this type
/// and wait on the facilities their abilities need (`backlog.md` §2.27).
#[derive(Debug, Clone, PartialEq)]
pub struct TokenDef {
    /// `None` is CR 111.4's default — "its name is the same as its
    /// subtype(s) plus the word 'Token'", so Kalitas's Zombie is named
    /// "Zombie Token". `Some` is CR 111.9's "create [name], a …" (Boo) and
    /// CR 111.10's named tokens (Walker).
    pub name: Option<String>,
    pub colors: Vec<Color>,
    pub types: Vec<crate::types::card_types::CardType>,
    /// CR 111.4: set by the effect together with the name, and the source
    /// of the name when none is given. Legendary is a *supertype* and goes
    /// in [`Self::supertypes`].
    pub subtypes: Vec<crate::types::card_types::Subtype>,
    /// CR 111.9's "a legendary 1/1 red Hamster" and CR 111.10d's Walker are
    /// the printed shapes; fifty-five printed effects create a legendary
    /// token (§2.27).
    pub supertypes: Vec<crate::types::card_types::Supertype>,
    /// `None` for a noncreature token — CR 208.3 gives a noncreature no
    /// power or toughness, and eighteen of CR 111.10's twenty predefined
    /// tokens are noncreature. A creature token carries both.
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub keyword_flags: Vec<KeywordFlag>,
    /// The abilities the effect writes onto the token — CR 111.10a's
    /// Treasure is an activated ability, the Roles are static ones, and
    /// 211 printed effects quote one inline (§2.27). Provenance for
    /// `AbilityDef::is_characteristic_defining` is the *creating effect*
    /// (CR 604.3a(2)'s second clause), so an author sets it as for a
    /// printed card.
    pub abilities: Vec<crate::objects::card_data::AbilityDef>,
    /// The token's text as a reader sees it. Display only, like
    /// `CardData::rules_text`; the engine reads [`Self::abilities`].
    pub rules_text: String,
    /// CR 303.4 for an Aura token — CR 111.10j–r's Roles print "enchant
    /// creature". `None` for everything that is not an Aura.
    pub enchant_filter: Option<SelectionFilter>,
    /// "Create a tapped Treasure token" — how the creating effect says the
    /// token enters. Merged into the entry's seed mods ahead of any
    /// replacement, the way CR 110.5b's "enters tapped" on a printed card is
    /// the card's own before anything else modifies the entry. "Tapped and
    /// attacking" is CR 508.4's and needs a combat-state write this does not
    /// carry (`codebase-state.md`).
    pub enters_tapped: bool,
}

impl TokenDef {
    /// CR 111.4's name: the given one, or "[subtypes] Token" when the
    /// effect gave none — "Dwarf Berserker Token" for the rule's own
    /// example, "Token" for a token with neither.
    pub fn effective_name(&self) -> String {
        match &self.name {
            Some(name) => name.clone(),
            None => {
                let mut words: Vec<String> =
                    self.subtypes.iter().map(|s| s.word()).collect();
                words.push("Token".to_string());
                words.join(" ")
            }
        }
    }

    /// Lower this description into the `CardData` a `GameObject` reads.
    ///
    /// CR 111.3: the values defined this way "are functionally equivalent
    /// to the characteristic values that are printed on a card" — which is
    /// why a token's `CardData` is built by the same builder a card's is
    /// and read by the same layer walk. No mana cost is set, so CR 111.6's
    /// mana value of 0 falls out of the builder's default.
    ///
    /// One `Arc` per call; a plural creation shares one across the tokens
    /// of one def, which is what `Arc<CardData>` means everywhere else in
    /// this engine — separate objects, one printed text.
    pub fn card_data(&self) -> std::sync::Arc<crate::objects::card_data::CardData> {
        let mut builder =
            crate::objects::card_data::CardDataBuilder::new(&self.effective_name());
        if let (Some(power), Some(toughness)) = (self.power, self.toughness) {
            builder = builder.power_toughness(power, toughness);
        }
        for color in &self.colors {
            builder = builder.color(*color);
        }
        for card_type in &self.types {
            builder = builder.card_type(*card_type);
        }
        for supertype in &self.supertypes {
            builder = builder.supertype(*supertype);
        }
        for subtype in &self.subtypes {
            builder = builder.subtype(subtype.clone());
        }
        for keyword in &self.keyword_flags {
            builder = builder.keyword_flag(*keyword);
        }
        for ability in &self.abilities {
            builder = builder.ability(ability.clone());
        }
        if !self.rules_text.is_empty() {
            builder = builder.rules_text(&self.rules_text);
        }
        if let Some(filter) = &self.enchant_filter {
            builder = builder.enchant_filter(filter.clone());
        }
        builder.build()
    }
}

/// Counter types that can be placed on permanents/players.
///
/// One enum for both subjects — CR 701.34a's proliferate gives "each one
/// additional counter of each kind that permanent or player already has" in
/// one sweep, and a kind is a kind wherever it sits. `Ord` because a player's
/// counters are a `BTreeMap` keyed on this: a map that iterates in enum order
/// is process-independent, which `CLAUDE.md`'s determinism rule asks of any
/// collection that reaches a count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CounterType {
    PlusOnePlusOne,
    MinusOneMinusOne,
    Loyalty,
    Charge,
    // --- Counters a player has (CR 122.1's "or player") ---
    /// CR 122.1f — ten or more and the player loses (CR 704.5c). Read off
    /// `PlayerState::counter_count`; infect's poison half (CR 120.3b) is the
    /// producer that is not built yet.
    Poison,
    /// CR 107.14 — the energy symbol {E} is one of these. Live Fast's "get
    /// {E}{E}" is the producer; paying {E} is a cost that waits for its card.
    Energy,
    // Keyword counters (rule 122.1b)
    Flying,
    Deathtouch,
    Lifelink,
    Trample,
    FirstStrike,
    DoubleStrike,
    Hexproof,
    Indestructible,
    Menace,
    Reach,
    Vigilance,
    Haste,

    // --- Counters that create a replacement effect (rule 122.1c/d/h) ---
    //
    // Nothing on any card says what these three do; the rule does, and between
    // them they exercise destroy replacement, damage prevention, untap
    // replacement and zone-change replacement across 164 printed cards.
    // `engine::replacement::gather` synthesizes their effects from the counter
    // itself, quoting the rule verbatim.
    /// CR 122.1c. Creates *two* effects: a replacement against destruction by
    /// an effect, and a prevention effect against damage.
    Shield,
    /// CR 122.1d. "If a permanent with a stun counter on it would become
    /// untapped, instead remove a stun counter from it."
    Stun,
    /// CR 122.1h. "If this permanent would be put into a graveyard from the
    /// battlefield, exile it instead."
    Finality,
    // Non-evergreen counter types added as relevant cards are implemented
}

impl CounterType {
    /// The keyword this counter grants (CR 122.1b), or `None` if it is not a
    /// keyword counter.
    ///
    /// > A keyword counter on a permanent or on a card in a zone other than the
    /// > battlefield causes that object to gain that keyword.
    ///
    /// CR 122.1b names fifteen keywords plus their variants. Twelve of them are
    /// `CounterType` variants today; the missing three are decayed, exalted and
    /// shadow. Decayed and exalted are not `KeywordFlag`s at all -- they are
    /// quadrant-3 keywords with ability bodies (CR 702.147, 702.83), so they
    /// will arrive as `AbilityDef`s and want a different bridge than this one.
    /// Shadow is a plain flag and just has no card needing it yet.
    ///
    /// Applied in Layer 6 by `compute::apply_effects`, read straight off
    /// `PermanentState::counters` rather than registered as a continuous
    /// effect -- same treatment as the +1/+1 counters in Layer 7c, and for the
    /// same reason: the state is already owned, and reconciling registry rows
    /// against every counter mutation is the pattern that turns effect
    /// existence into a fixpoint.
    pub fn keyword_granted(self) -> Option<crate::types::keywords::KeywordFlag> {
        use crate::types::keywords::KeywordFlag as K;
        Some(match self {
            CounterType::Flying => K::Flying,
            CounterType::Deathtouch => K::Deathtouch,
            CounterType::Lifelink => K::Lifelink,
            CounterType::Trample => K::Trample,
            CounterType::FirstStrike => K::FirstStrike,
            CounterType::DoubleStrike => K::DoubleStrike,
            CounterType::Hexproof => K::Hexproof,
            CounterType::Indestructible => K::Indestructible,
            CounterType::Menace => K::Menace,
            CounterType::Reach => K::Reach,
            CounterType::Vigilance => K::Vigilance,
            CounterType::Haste => K::Haste,
            // CR 122.1b names fifteen keywords; none of these is one. The
            // three replacement counters are emphatically not keyword counters
            // — an indestructible counter grants a keyword and a shield counter
            // creates a replacement effect, and conflating them would give a
            // shielded permanent permanent protection instead of one use.
            CounterType::PlusOnePlusOne
            | CounterType::MinusOneMinusOne
            | CounterType::Loyalty
            | CounterType::Charge
            | CounterType::Poison
            | CounterType::Energy
            | CounterType::Shield
            | CounterType::Stun
            | CounterType::Finality => return None,
        })
    }
}

/// Color change description for ChangeColor primitive (layer 5).
///
/// Three operations map 1:1 to the three `EffectModification` variants:
/// - `Add(Color)` → `AddColor(Color)` — adds a color without removing existing ones
/// - `Set(HashSet<Color>)` → `SetColors(HashSet<Color>)` — replaces all colors
/// - `RemoveAll` → `RemoveAllColors` — makes the object colorless
#[derive(Debug, Clone, PartialEq)]
pub enum ColorChange {
    /// Add a single color (e.g. "becomes blue in addition to its other colors")
    Add(Color),
    /// Set colors to exactly this set (e.g. "becomes red" = Set({Red}))
    Set(std::collections::HashSet<Color>),
    /// Remove all colors (e.g. "becomes colorless")
    RemoveAll,
}

/// Type change description for ChangeType primitive (layer 4).
///
/// Supports both additive/subtractive operations and overwrite ("set") operations.
/// When a `set_*` field is `Some`, it takes priority over the corresponding
/// add/remove fields. A single card effect can combine these
/// (e.g., "becomes an artifact creature" sets types while adding subtypes).
#[derive(Debug, Clone, PartialEq)]
pub struct TypeChange {
    pub add_types: Vec<crate::types::card_types::CardType>,
    pub remove_types: Vec<crate::types::card_types::CardType>,
    /// If Some, replaces all card types with this set (ignores add_types/remove_types).
    pub set_types: Option<std::collections::HashSet<crate::types::card_types::CardType>>,
    pub add_subtypes: Vec<crate::types::card_types::Subtype>,
    pub remove_subtypes: Vec<crate::types::card_types::Subtype>,
    /// If Some, replaces all subtypes with this set (ignores add_subtypes/remove_subtypes).
    pub set_subtypes: Option<std::collections::HashSet<crate::types::card_types::Subtype>>,
    pub add_supertypes: Vec<crate::types::card_types::Supertype>,
    pub remove_supertypes: Vec<crate::types::card_types::Supertype>,
    /// If Some, replaces all supertypes with this set (ignores add_supertypes/remove_supertypes).
    pub set_supertypes: Option<std::collections::HashSet<crate::types::card_types::Supertype>>,
}

/// Which role a `Primitive::Copy`'s own recipients play, and where the other
/// role comes from (CR 707.4).
///
/// **One arm per role binding, not per card**, and the printed corpus needs
/// both because cards bind the atom's target to opposite ends of the same
/// sentence: Cytoshape targets the permanent that *becomes* a copy, Mirrorweave
/// and Mirrorform target the one being copied.
///
/// Rejected: a `{ from, to }` product of two enums. Two of its four
/// combinations are nonsense — a recipient copying itself, and "each other"
/// with no donor to be other than — and an unreachable state is how an arm
/// eventually gets written for one.
#[derive(Debug, Clone, PartialEq)]
pub enum CopyRoles {
    /// The recipients become copies of a permanent **chosen** as the effect
    /// resolves (CR 707.4). A choice, not a target: hexproof and shroud do not
    /// apply, and nothing fizzles if it leaves. Cytoshape, Polymorphous Rush.
    RecipientsCopyChosen(SelectionFilter),
    /// The atom's target supplies the values, and every permanent matching
    /// `filter` becomes a copy of it. Mirrorweave, Mirrorform.
    FilteredCopyRecipient {
        /// Which permanents become copies, evaluated as the effect begins to
        /// apply and then locked (CR 611.2c).
        filter: ObjectFilter,
        /// **Whether the donor is excluded, and it is data because the cards
        /// disagree.** Mirrorweave says "each **other** creature"; Mirrorform
        /// says "each nonland permanent you control", which *includes* the
        /// target whenever you control it.
        ///
        /// A permanent copying *itself* is very nearly a no-op — the capture
        /// is its own post-layer-1 state, so applying it changes nothing at the
        /// instant it is made. Not exactly one, and CR-correctly so: the row
        /// has its own `Duration`, so it holds those values after an earlier
        /// copy row would have expired.
        exclude_donor: bool,
    },
}

// ---------------------------------------------------------------------------
// Primitives — atomic game actions (rule 610, 701)
// ---------------------------------------------------------------------------

/// What an effect does when it resolves (one-shot effects, rule 610).
///
/// Each variant is a single atomic game action. Complex effects are built
/// by combining primitives via the `Effect` combinator enum.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    // === Zone movement (rule 701) ===
    /// Destroy a permanent (rule 701.8) — respects indestructible/regenerate
    Destroy,
    /// Exile an object (rule 701.13)
    Exile,
    /// Sacrifice N permanents (rule 701.21).
    ///
    /// **The filter is what gets sacrificed, the amount is how many, and the
    /// `EffectRecipient` is who does the sacrificing.** Diabolic Edict's "target
    /// *player* sacrifices *a creature* of their choice" needs all three and
    /// they are three different questions: the recipient is CR 115.1's target,
    /// the filter is CR 701.21a's "its controller moves **it**", and the amount
    /// is what separates Diabolic Edict from Barter in Blood ("two creatures")
    /// and Blasphemous Edict ("thirteen creatures"). Carrying only the filter
    /// would make every edict sacrifice a player or target a creature; carrying
    /// no amount would silently turn all three cards into the first.
    ///
    /// **The amount is a ceiling, not a requirement** — CR 101.3 performs "only
    /// the possible portion", so Blasphemous Edict against a player with two
    /// creatures takes two. The sacrifices are simultaneous (CR 701.21, one
    /// batch), which is what a second permanent makes observable.
    Sacrifice(SelectionFilter, AmountExpr),
    /// Return to owner's hand ("bounce")
    ReturnToHand,
    /// Return to the battlefield (from exile/graveyard)
    ReturnToBattlefield,
    /// Put on top of owner's library
    PutOnTopOfLibrary,
    /// Put on bottom of owner's library
    PutOnBottomOfLibrary,
    /// Shuffle into owner's library
    ShuffleIntoLibrary,
    /// Shuffle a library (CR 701.24a). Whose is the recipient's: `Controller`
    /// is "shuffle your library", `Implicit` the source's **owner's** — the
    /// rider of "shuffle it into its owner's library" once the substitute
    /// has made the move (Darksteel Colossus), where "it" is the source — and
    /// a target is a player or an object standing for its owner. Proposes
    /// `GameAction::ShuffleLibrary`.
    ShuffleLibrary,
    /// Mill N cards (rule 701.17)
    Mill(AmountExpr),
    /// Discard N cards (rule 701.9), chosen as [`DiscardChooser`] says.
    ///
    /// The recipient is who discards — `EffectRecipient::Target` on a player
    /// for Mind Rot's and Hymn to Tourach's "target player discards two
    /// cards". The amount is a ceiling: CR 101.3 does as much as it can, so a
    /// hand shorter than the count discards the whole hand.
    Discard(AmountExpr, DiscardChooser),

    // === Damage & life ===
    /// Deal damage (rule 120).
    ///
    /// A struct variant because of the field: CR 615.12's "the damage can't be
    /// prevented" is a property of the event this primitive proposes, so the
    /// primitive is where the card says it. Every site spells `unpreventable`
    /// out rather than reaching a constructor — nine cards in the whole game
    /// print the clause, and a default would let the tenth forget it silently.
    DealDamage {
        amount: AmountExpr,
        /// Pinpoint Avalanche's last sentence. Becomes
        /// `GameAction::DealDamage::unpreventable` on every member of the
        /// batch this proposes.
        unpreventable: bool,
    },
    /// Gain life
    GainLife(AmountExpr),
    /// Lose life
    LoseLife(AmountExpr),
    /// CR 119.5 — "your life total becomes N": the player gains or loses the
    /// difference, proposed as a `GainLife` or a `LoseLife` so that both
    /// replacement families and every "can't gain life" see it. Rhox
    /// Faithmender's ruling — "becomes 10" from 3 becomes 17 — and Skullcrack's
    /// — "becomes N" higher than the current total does nothing — both fall
    /// out of that rather than being coded.
    SetLifeTotal(AmountExpr),

    // === The game's end (CR 104.2b, 104.3e) ===
    /// "Target player loses the game" — a `GameAction::PlayerLoses` with
    /// `LossReason::Effect`, so it can be replaced or refused like a
    /// state-based loss.
    LoseGame,
    /// "You win the game" — a `GameAction::PlayerWins`.
    WinGame,

    // === Card flow ===
    /// Draw N cards
    DrawCards(AmountExpr),
    /// Scry N (rule 701.22) — one `GameAction::Scry` for the recipient.
    ///
    /// The whole keyword action is the performer's: it looks, asks, and
    /// reorders the library, proposing no zone change (CR 701.22 moves no card
    /// between zones).
    Scry(AmountExpr),
    /// Surveil N (rule 701.25)
    Surveil(AmountExpr),

    // === Turn structure (rule 500.7) ===
    /// Take an extra turn after this one (CR 500.7).
    ///
    /// **The turn queue's only producer**, and it schedules rather than
    /// mutates: the extra turn is an entry on `GameState::turn_queue`, and the
    /// turn itself is proposed as a `GameAction::BeginTurn` when the drainer
    /// reaches it — which is what lets a skip replace it (CR 614.10a's "the
    /// first occurrence that isn't skipped").
    ///
    /// "The most recently created turn will be taken first" is the push: the
    /// queue is a stack, so two of these resolving in one turn are taken in
    /// the reverse of the order they resolved, which is Time Walk's own ruling.
    ///
    /// The recipient names the player, which is `Controller` for every printed
    /// card in reach ("take an extra turn"). CR 500.7's other sentence — extra
    /// turns for *several* players are added in APNAP order — has no producer,
    /// because no recipient this primitive accepts resolves to more than one
    /// player; the first that does adds the sort and says so.
    ExtraTurn,

    /// Extra phases, spliced into **this** turn directly after the one the
    /// effect resolved in (CR 500.8).
    ///
    /// A `Vec` rather than one phase, because one effect creates a run of them
    /// — Aggravated Assault's *"an additional combat phase followed by an
    /// additional main phase"* is one resolution and two phases, and their
    /// printed order has to be the order they are spliced in rather than an
    /// accident of which of two splices ran second.
    ///
    /// > 500.8. ... If a phase or step is created after the current one, ...
    /// > if multiple extra phases are created after the same phase, the most
    /// > recently created phase will occur first.
    ///
    /// That last sentence needs no comparator: a later splice at the same
    /// index pushes the earlier one further back, which is the ordering the
    /// rule describes — the same shape CR 500.7's "most recently created turn"
    /// turned out to be `Vec::pop` for.
    ///
    /// Scheduling and not a board mutation, as [`Self::ExtraTurn`] is: each
    /// spliced phase becomes a `GameAction::BeginPhase` proposal when the
    /// drainer reaches it, which is what a skip replaces (CR 614.10).
    ExtraPhases(Vec<PhaseType>),

    // === Mana ===
    /// Produce mana (for mana abilities, rule 605)
    ProduceMana(ManaOutput),

    // === Counters ===
    /// Put `amount` `counter` counters on each recipient permanent
    /// (CR 122.1).
    ///
    /// `by` is who puts them on — the fact Vorinclex, Monstrous Raider reads
    /// off `GameAction::AddCounters::by`. `PlayerRef::You` is the effect's
    /// controller, which is every printed one-shot and is written out rather
    /// than defaulted; anything else is an effect whose text names another
    /// player, Bold Plagiarist's "*they* put the same number and kind of
    /// counters on this creature" — the opponent puts counters on a creature
    /// they do not control, and neither the effect's controller nor the
    /// object's is the answer. Resolved at resolution (`resolve_putter`):
    /// `You` the controller, `Player` itself, `Owner` the source's owner,
    /// `Opponent` the resolution's player target or the only opponent.
    AddCounters {
        counter: CounterType,
        amount: AmountExpr,
        by: PlayerRef,
    },
    /// Remove N counters of a type from target
    RemoveCounters(CounterType, AmountExpr),
    /// A player gets `amount` `counter` counters — Oracle's verb for a player
    /// ("you get {E}{E}", "that player gets a poison counter"), and its own
    /// primitive because [`Self::AddCounters`] resolves its recipient as
    /// permanents and one primitive answering for both would make
    /// `EffectRecipient::Controller` mean two things. The recipient is
    /// resolved as `GainLife`'s is; `by` as [`Self::AddCounters`]'s.
    GetCounters {
        counter: CounterType,
        amount: AmountExpr,
        by: PlayerRef,
    },

    // === Tokens ===
    /// Create N tokens (CR 701.7a) — resolved as one
    /// `GameAction::CreateTokens` carrying the def N times, so CR 614.16's
    /// doublers see one event and the N entries are one batch.
    CreateToken(TokenDef, AmountExpr),

    // === Regeneration (rule 701.19) ===
    /// Regenerate a permanent (CR 701.19a).
    ///
    /// A *keyword action*, not a shorthand: CR 701.19a spells out what it means
    /// and the engine implements that text rather than a paraphrase — "the next
    /// time [permanent] would be destroyed this turn, instead remove all damage
    /// marked on it and its controller taps it. If it's an attacking or
    /// blocking creature, remove it from combat."
    ///
    /// A keyword action rather than a [`Self::CreateReplacement`] because the
    /// rule spells out the whole def — duration, rewrite and rider — and the
    /// engine builds it once here rather than asking each card to spell it
    /// again.
    Regenerate,

    // === Replacement and prevention effects from a resolution (CR 614.3, 615.7) ===
    /// A resolving spell or ability creates a replacement or prevention effect
    /// with a duration — Mending Hands' "prevent the next 4 damage that would
    /// be dealt to any target this turn", Safe Passage's "prevent all damage
    /// that would be dealt to you and creatures you control this turn".
    ///
    /// **The third instance of one pattern**, beside [`Self::Restrict`] and
    /// `cost-architecture.md`'s `ModifyCost`: a def the card authors, a
    /// `Duration` the card authors (CR 608.2c hands scope to a human reader, so
    /// no engine may infer it — `cant-effects-architecture.md` §9 finding 1),
    /// and a registry row carrying controller and turn that expires through the
    /// CR 514.2 hooks the registry already runs. A prevention effect *is* a
    /// `ReplacementDef` whose rewrite prevents (CR 615.1 opens "like
    /// replacement effects"), so there is no second def type for it.
    ///
    /// **What the resolution fills in follows [`Self::Regenerate`] and
    /// [`Self::Restrict`].** With a `Target`/`Choose` recipient, one row per
    /// resolved target, the authored empty `Fixed` filled with that object or
    /// player. With a `FilteredPermanents` recipient, one row per matching
    /// permanent at resolution — CR 615.11's "creates a prevention shield for
    /// each applicable creature when the spell or ability … resolves", so a
    /// creature that enters afterwards has none. With an `Implicit` recipient,
    /// the def as authored: a `Filter` and/or a `PlayerSet`, evaluated at each
    /// event, which is Safe Passage's ruling that creatures entering after it
    /// resolved are covered.
    ///
    /// The row keeps the resolution's targets. A rider that must act on "the
    /// thing this effect targeted at resolution" rather than on the event's
    /// subject — Divine Deflection's "deals that much damage to any target",
    /// chosen at cast — has nowhere else to read them from
    /// (`codebase-state.md` item 90).
    ///
    /// The [`PatternFill`] is the other half of "what the resolution fills
    /// in", and it is a *third* argument rather than a recipient because the
    /// recipient is spoken for: Circle of Protection: Red's row is about
    /// **you**, and the source it chooses is a fact about the events it
    /// watches. See that type.
    CreateReplacement(
        Box<crate::types::replacement::ReplacementDef>,
        Duration,
        PatternFill,
    ),

    // === "Can't" effects (CR 101.2, 614.17) ===
    /// A resolving spell or ability creates a CR 101.2 prohibition.
    ///
    /// **The `Duration` is authored, never inferred, and that is the whole
    /// point of it being an argument.** CR 608.2c says of "Destroy target
    /// creature. It can't be regenerated" that later text modifies the meaning
    /// of earlier text, and instructs the reader to "apply the rules of English
    /// to the text" — scope determination handed to a human. Two cards with
    /// identical restriction text can have different scopes because of the
    /// sentence before them, so no engine can derive this
    /// (`cant-effects-architecture.md` §9 finding 1). Same reason
    /// [`Self::ModifyPowerToughness`] takes one.
    ///
    /// `ObjectSet::Fixed` inside the def is how a card names its resolved
    /// targets; the primitive does not fill it in, because a restriction on
    /// *players* ("players can't gain life this turn") has no targets to fill.
    Restrict(crate::types::restriction::RestrictionDef, Duration),

    // === Combat ===
    /// Remove a permanent from combat (CR 506.4).
    ///
    /// Half of CR 701.19a's regeneration rider. Not a zone change and not
    /// CR 614-observable: no card replaces "is removed from combat", so it has
    /// no `GameAction` and writes `PermanentState` directly, the way the
    /// cleanup step's damage wipe does.
    RemoveFromCombat,

    /// Remove all damage marked on a permanent (CR 120.3, the CR 514.2 wipe's
    /// on-demand form).
    ///
    /// The other half of CR 701.19a's rider, and the half that makes
    /// regeneration mean anything: without it a regenerated creature meets
    /// CR 704.5g again on the very next state-based check, spends nothing
    /// (the shield is `Uses::Once` and already gone), and dies.
    RemoveAllDamage,

    /// Attach the source permanent to the target (CR 701.3a) — Equip's verb,
    /// "[Cost]: Attach this permanent to target creature you control"
    /// (CR 702.6a).
    ///
    /// The *attachment* is the resolving ability's source permanent
    /// (`ResolutionContext::ability_source`), the *host* is the recipient.
    /// Legality is the recipient's job: CR 608.2b re-checks the target against
    /// the filter it was chosen under, which is how "creature you control"
    /// at resolution (CR 301.5b) and "doesn't move" against a non-creature
    /// (CR 701.3b) both fall out with no second check here. Attaching to the
    /// host it is already on does nothing (CR 701.3b), which the performer
    /// decides — it is the transition that gets a timestamp (CR 613.7e) and an
    /// event, not the state.
    Attach,
    /// Two creatures fight (rule 701.14)
    Fight,
    /// Tap a permanent (rule 701.26)
    Tap,
    /// Untap a permanent (rule 701.26)
    Untap,

    // === Continuous effect primitives (applied via layer system) ===
    /// Set power/toughness to specific values (layer 7b)
    SetPowerToughness(AmountExpr, AmountExpr, Duration),
    /// Modify power/toughness by +X/+Y (layer 7c)
    ModifyPowerToughness(AmountExpr, AmountExpr, Duration),
    /// Switch power and toughness (layer 7d)
    SwitchPowerToughness(Duration),
    /// Grant a keyword flag (layer 6).
    ///
    /// **Named for what it carries, not for what a card says.** Puresteel
    /// Paladin's "Equipment you control have equip {0}" grants a keyword, and it
    /// is a `GrantAbility`, not this — because `equip {0}` is an activated
    /// ability with a cost and `KeywordFlag` holds no cost. Only the CR 702
    /// keywords whose entire meaning is their presence live in `KeywordFlag`,
    /// and this variant reaches exactly those. If the keyword you want is not in
    /// that enum, that is the answer, not an omission: use `GrantAbility` with
    /// a fully parameterized `AbilityDef`. See `KeywordFlag` for the map.
    GrantKeywordFlag(KeywordFlag, Duration),
    /// Remove a keyword flag (layer 6). CR 113.10b — removing an ability
    /// removes all instances of it; a `HashSet` gives that structurally.
    ///
    /// Named `RemoveAbility` until the Layer 6 phase, which was simply wrong:
    /// it takes a `KeywordFlag`, never an ability. To remove a parameterized
    /// keyword — an equip ability, protection from a quality — use
    /// `LoseAbility` with the ability's id.
    RemoveKeywordFlag(KeywordFlag, Duration),
    /// Grant a whole ability (layer 6).
    ///
    /// The channel for one-off granted text *and* for every CR 702 keyword that
    /// is not a bare flag — equip, ward, protection, landwalk, cycling, and the
    /// ~170 others whose keyword name abbreviates an ability with a body. A card
    /// that says "gains equip {2}" comes through here, not through
    /// `GrantKeywordFlag`.
    ///
    /// Boxed because `AbilityDef` contains an `Effect`, which contains
    /// `Primitive` — the recursion is real and needs an indirection. It also
    /// keeps `Primitive` small, since this variant is otherwise the largest.
    ///
    /// CR 604.3a(2): the granted def's `is_characteristic_defining` is cleared
    /// when it is applied, whatever the card author wrote. A granted ability is
    /// never a CDA.
    GrantAbility(Box<crate::objects::card_data::AbilityDef>, Duration),
    /// Remove one ability by id (layer 6). CR 113.10b — *all* instances of it.
    LoseAbility(crate::types::ids::AbilityId, Duration),
    /// Remove every ability and keyword (layer 6). Humility, Merfolk Trickster.
    LoseAllAbilities(Duration),
    /// Change color (layer 5)
    ChangeColor(ColorChange, Duration),
    /// Change types (layer 4)
    ChangeType(TypeChange, Duration),
    /// Gain control (layer 2)
    GainControl(Duration),

    // === Copy effects (CR 707, layer 1a) ===
    /// One or more permanents become a copy of another (CR 707.4).
    ///
    /// The `Duration` is authored for the reason [`Self::Restrict`]'s is: CR
    /// 611.2's scope comes from the card's English, not from the mechanism. The
    /// turn-bounded shapes only — `Duration::Indefinite` needs CR 400.7 first
    /// (CV-1b), because a row reachable by neither expiry nor `remove_by_source`
    /// outlives its subject without bound (`copy-effects-architecture.md` §5.3).
    ///
    /// The affected set is `ObjectSet::Fixed`, locked as the effect begins
    /// (CR 611.2c), and the captured values are locked with it (CR 707.2b/2c) —
    /// which is what makes a copy row independent of every other layer 1 effect
    /// and so keeps this off critical-path item 7.
    Copy(CopyRoles, Duration),

    // === Counter spells/abilities (rule 701.6) ===
    /// Counter a spell on the stack (rule 701.6a).
    /// The countered spell is moved to its owner's graveyard.
    CounterSpell,
    /// Counter an activated or triggered ability on the stack (rule 701.6b).
    /// The countered ability ceases to exist — it is simply removed from the stack.
    CounterAbility,
}

// ---------------------------------------------------------------------------
// Effect — the combinator layer
// ---------------------------------------------------------------------------

/// What an ability or spell does when it resolves.
///
/// Effects are composable: `Sequence` chains multiple effects,
/// `Conditional` gates on a condition, `Modal` offers choices, etc.
/// Each leaf is an `Atom` that applies a `Primitive` to targets.
///
/// **Continuous effects** (e.g. "+3/+3 until end of turn") are modeled as
/// an `Atom` containing a continuous `Primitive` (like `ModifyPowerToughness`)
/// that registers a modifier in the GameState. The layer system (rule 613)
/// reads these modifiers to compute effective characteristics.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Apply a primitive to resolved targets
    Atom(Primitive, EffectRecipient),

    /// Execute effects in order (e.g. "deal 3 damage and draw a card")
    Sequence(Vec<Effect>),

    /// "If [condition], [effect]" — intervening if (rule 603.4)
    Conditional(Condition, Box<Effect>),

    /// "You may [effect]" (rule 603.5)
    Optional(Box<Effect>),

    /// "Choose N mode(s):" (rule 700.2)
    Modal {
        count: ModalCount,
        modes: Vec<Effect>,
    },

    /// "For each [thing], [effect]"
    ForEach(Selector, Box<Effect>),

    /// "Do this N times"
    Repeat(AmountExpr, Box<Effect>),

    /// CR 614/615 — this ability generates a replacement or prevention effect.
    ///
    /// On a **static** ability it produces no layer rows: `register_static_effects`
    /// skips it, and `engine::replacement::gather` discovers it by reading the
    /// source's *effective* ability list at the instant an event is proposed.
    /// That is not a shortcut — it is what makes Humility and Blood Moon strip a
    /// replacement ability for free, and it is CR 614.4's "must exist before the
    /// event" asked at the one moment that matters.
    ///
    /// A **resolving** spell or ability does not create one through this
    /// variant: CR 614.3 gives such an effect a duration ("prevent all damage
    /// that would be dealt this turn") and this carries none, so
    /// `resolve_effect` rejects it by name. The durational form is
    /// [`Primitive::CreateReplacement`], which takes the `Duration` as an
    /// argument; CR 701.19a's regeneration shield is a keyword action and
    /// comes through [`Primitive::Regenerate`], which knows its duration and
    /// builds its own def.
    ///
    /// Boxed because `ReplacementDef` carries an `Effect` of its own (the
    /// CR 615.5 rider), so the recursion is real.
    Replacement(Box<crate::types::replacement::ReplacementDef>),

    /// CR 101.2 / 614.17 — this ability states that something can't happen.
    ///
    /// The same shape as [`Self::Replacement`] and for the same reasons. On a
    /// **static** ability it produces no layer rows: `register_static_effects`
    /// skips it and `engine::restriction::is_prohibited` discovers it by reading
    /// the source's *effective* ability list at the instant the question is
    /// asked, which is what makes Humility strip a "can't" for free.
    ///
    /// A **resolving** spell or ability does not create one through this
    /// variant, because this carries no duration and CR 608.2c will not let the
    /// engine infer one (§9 finding 1). Skullcrack's "Players can't gain life
    /// this turn" comes through [`Primitive::Restrict`], which takes the
    /// `Duration` as an argument.
    ///
    /// Boxed to mirror [`Self::Replacement`] and to keep `Effect` small;
    /// `RestrictionDef` grows an `unless: Option<Condition>` when Phase 6 gives
    /// `Condition` a meaning, and `Condition` is not small.
    Restriction(Box<crate::types::restriction::RestrictionDef>),

    /// CR 601.2f / 613.11 — this ability changes what spells cost to cast.
    ///
    /// The third static shape with no layer rows, and the same reasons as
    /// the two above: a cost effect has no layer (CR 613.11 applies it after
    /// all of them) and applies to no object — it applies to a *cost being
    /// determined* — so `register_static_effects` skips it and
    /// `engine::cost_determination::cost_modifications_for` reads it off the source's
    /// *effective* ability list at CR 601.2f. That read is CR 604.2's
    /// existence check, which is what makes Humility strip a tax for free.
    /// Through an "as long as" wrapper too — Trinisphere's shape — which
    /// [`Self::as_cost_modification`] peels.
    ///
    /// A **resolving** spell or ability does not create one through this
    /// variant: "spells cost {1} more this turn" needs a CR 611.2a duration
    /// this carries none of (`cost-architecture.md` §3.10).
    CostModification(Box<crate::types::cost_modification::CostModificationDef>),

    /// CR 603 — this ability is a triggered ability: a condition, an optional
    /// intervening "if", an optional once-per-turn limit, and the effect.
    ///
    /// The fourth arm-in-the-tree shape, on an `AbilityDef` whose
    /// `ability_type` is `Triggered`. It produces no layer rows and is never
    /// resolved as written: the dispatcher reads it off the source's
    /// *effective* ability list as a batch closes (`engine::triggers`), and
    /// what reaches the stack is the def's inner `effect` with the binding
    /// beside it. Reaching this arm in `resolve_effect` is a wiring error.
    ///
    /// Boxed because `TriggerDef` carries an `Effect` of its own.
    Triggered(Box<crate::types::triggers::TriggerDef>),

    // Future phases:
    // ApplyContinuous(ContinuousEffectDef),
    // Custom(CardId),  // escape hatch
}

impl Effect {
    /// Visit every `AbilityDef` nested in this effect: a granted ability
    /// (`Primitive::GrantAbility`), a created token's abilities
    /// (`Primitive::CreateToken`, and the token a replacement creates
    /// instead), and whatever a replacement's rider nests in turn. The order
    /// is the recursion's — each def, then the defs inside its own effect —
    /// and nothing reads it except as a fixed order: `CardDataBuilder::build`
    /// is the caller, numbering these after the printed list so that a def
    /// that never sits in that list still gets an id derived from the card.
    pub fn for_each_ability_def_mut(
        &mut self,
        f: &mut impl FnMut(&mut crate::objects::card_data::AbilityDef),
    ) {
        use crate::types::replacement::{GameActionTemplate, Rewrite};
        match self {
            Effect::Atom(Primitive::GrantAbility(def, _), _) => {
                f(def);
                def.effect.for_each_ability_def_mut(f);
            }
            Effect::Atom(Primitive::CreateToken(token, _), _) => {
                for def in &mut token.abilities {
                    f(def);
                    def.effect.for_each_ability_def_mut(f);
                }
            }
            Effect::Atom(..) | Effect::Restriction(_) | Effect::CostModification(_) => {}
            Effect::Sequence(effects) | Effect::Modal { modes: effects, .. } => {
                for effect in effects {
                    effect.for_each_ability_def_mut(f);
                }
            }
            Effect::Conditional(_, effect)
            | Effect::Optional(effect)
            | Effect::ForEach(_, effect)
            | Effect::Repeat(_, effect) => effect.for_each_ability_def_mut(f),
            Effect::Replacement(def) => {
                if let Rewrite::Instead(GameActionTemplate::CreateTokens { def: token, .. }) =
                    &mut def.rewrite
                {
                    for def in &mut token.abilities {
                        f(def);
                        def.effect.for_each_ability_def_mut(f);
                    }
                }
                if let Some(then) = &mut def.then {
                    then.for_each_ability_def_mut(f);
                }
            }
            Effect::Triggered(def) => def.effect.for_each_ability_def_mut(f),
        }
    }

    /// The instances of "target" this effect declares, in printed order
    /// (CR 601.2c) — what the announcement asks for and CR 608.2b re-checks.
    ///
    /// Each `Target`/`Choose` atom **declares** an instance; an
    /// `EffectRecipient::SameInstanceAs` atom refers back to one and declares
    /// nothing. That is what separates Ensoul Artifact's two atoms — one
    /// instance, acted on twice — from Seeds of Strength's three clauses, which
    /// are three; written without the back-reference the two cards are the same
    /// shape, and `EffectRecipient::SameInstanceAs`'s doc has the worked
    /// comparison.
    ///
    /// **Run once per card, by `CardDataBuilder::build`**, which stores the
    /// answer on every def it reaches (`AbilityDef::instances`) and on the card
    /// (`CardData::spell_instances`). The engine reads the stored lists: the
    /// castability check asks per card in hand per priority pass, and this
    /// allocates. `cards::registry`'s test checks the stored lists against this
    /// walk. CR 603.3d will want the same walk for a triggered ability.
    ///
    /// **`Atom` and `Sequence` only** — the scope the one-recipient rule this
    /// replaced also had. `Modal` is the one that will need more than a wider
    /// walk: CR 601.2b chooses modes *before* 601.2c, so an unchosen mode
    /// announces no targets, and a walk that descended into every branch would
    /// announce all of them. It resolves to an error today (`resolve_effect`),
    /// and `codebase-state.md` item 153 carries it.
    pub fn instances(&self) -> Vec<EffectRecipient> {
        let mut out = Vec::new();
        self.for_each_instance(&mut |recipient| {
            out.push(recipient.clone());
            true
        });
        out
    }

    /// The clause of the `ix`th instance this effect declares, found without
    /// collecting them — what a `SameInstanceAs(ix)` atom resolves against when
    /// nothing was announced (`targeting::DeclaredInstances::Effect`).
    pub fn instance(&self, ix: usize) -> Option<&EffectRecipient> {
        let mut remaining = ix;
        let mut found = None;
        self.for_each_instance(&mut |recipient| {
            if remaining == 0 {
                found = Some(recipient);
                return false;
            }
            remaining -= 1;
            true
        });
        found
    }

    /// [`Self::instances`]' walk: each declaring atom in pre-order, stopping
    /// when `f` returns `false`.
    fn for_each_instance<'a>(&'a self, f: &mut impl FnMut(&'a EffectRecipient) -> bool) -> bool {
        match self {
            Effect::Atom(_, recipient @ (EffectRecipient::Target(_, _) | EffectRecipient::Choose(_, _))) => {
                f(recipient)
            }
            Effect::Sequence(effects) => effects.iter().all(|sub| sub.for_each_instance(f)),
            // CR 603.3d — a trigger's targets are its effect's, announced at
            // placement; item 153's note said this walk would want the arm.
            Effect::Triggered(def) => def.effect.for_each_instance(f),
            _ => true,
        }
    }

    /// The cost modification a static body is, with the "as long as" clause
    /// wrapped around it if there is one.
    ///
    /// One peel, used by every leg of `engine::cost_determination::cost_modifications_for`'s
    /// gate and by the gather itself, so a conditional cost effect is seen
    /// everywhere an unconditional one is. The replacement and restriction
    /// gates match the body without peeling `Conditional` and so miss a
    /// conditional one — `cost-architecture.md` §8 item 1.
    pub fn as_cost_modification(
        &self,
    ) -> Option<(Option<&Condition>, &crate::types::cost_modification::CostModificationDef)> {
        match self {
            Effect::CostModification(def) => Some((None, def)),
            Effect::Conditional(condition, inner) => match inner.as_ref() {
                Effect::CostModification(def) => Some((Some(condition), def)),
                _ => None,
            },
            _ => None,
        }
    }

    /// The replacement effect this static ability is, if it is one — looking
    /// through an "as long as" clause if there is one, since "as long as you
    /// control an Island, if X would happen, Y happens instead" is still a
    /// replacement effect.
    ///
    /// Three callers ask only *whether* an ability carries a replacement
    /// effect, to decide if its object is worth the gather's attention:
    /// `register_static_effects` filing the object as a source,
    /// `RegistryScopeSummary::of` noting that a grant or copy row carries
    /// one, and the gather's named leg reading such rows. None of them cares
    /// whether the "as long as" clause is true right now — that is asked at
    /// each proposal, against the board as it is then, by the gather itself.
    /// So this function never evaluates the condition; it only looks past
    /// it. The same peel as [`Self::as_cost_modification`].
    pub fn replacement_body(&self) -> Option<&crate::types::replacement::ReplacementDef> {
        match self {
            Effect::Replacement(def) => Some(def),
            Effect::Conditional(_, inner) => match inner.as_ref() {
                Effect::Replacement(def) => Some(def),
                _ => None,
            },
            _ => None,
        }
    }
}
