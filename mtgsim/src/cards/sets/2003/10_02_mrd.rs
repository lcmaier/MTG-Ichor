//! Mirrodin (`mrd`), released 2003-10-02.

use std::sync::Arc;

use crate::cards::authoring::{
    artifact, becomes_types, create_tokens, creature, each, equip, gets, host, non, put_counters, spell_ability, static_ability, target_permanent,
    you,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::{ArtifactType, CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{AffectedSet, AmountExpr, CounterType, Effect, PlayerSet, TokenDef, TypeChange};
use crate::types::keywords::KeywordFlag;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::EventPattern;
use crate::types::restriction::{Restriction, RestrictionDef};

/// Battlegrowth — {G}
/// Instant
///
/// Put a +1/+1 counter on target creature.
///
/// (Oracle text verified on Scryfall, 2026-09-01. Mirrodin.)
///
/// # Why this one out of 72
///
/// Scryfall lists 72 instants and sorceries containing "put a +1/+1 counter on
/// target creature", and Battlegrowth is the **only one whose entire oracle
/// text is that sentence**. Every other one carries a rider the engine cannot
/// express yet — a keyword until end of turn, morbid, flashback, a mode, a
/// second target. So the card adds one rules line and no machinery, which is
/// the same bar [`crate::cards::printings::idyllic_beachfront`] was picked
/// against.
///
/// # What it makes reachable, and why it takes two cards to do it
///
/// CR 704.5q is defined over **one permanent holding two counter kinds**, so
/// what it needs is a source of each sign that can meet on one object.
/// [`crate::cards::printings::chainbreaker`] is already the -1/-1 half: it
/// is colorless, so `random_deck` puts it in every deck, and CR 122.6a gives it
/// two -1/-1 counters on entry. Nothing in either pool produced a +1/+1
/// counter, so the annihilation sweep had never run in a game — 0 occurrences
/// across 200 stress games, measured before this card and after.
///
/// **It targets any creature, not "a creature you control".** That is the whole
/// reason the pairing works: Chainbreaker is as likely to be across the table
/// as under it, and a controller-scoped counter spell would reach it only half
/// the time. The 72-card list has both shapes and this is the unrestricted one.
///
/// # The engine path it opens, which is why it is in `PERFORMANCE_POOL`
///
/// `Primitive::AddCounters` had **no reader anywhere** — not in a registered
/// card, not in a test. Chainbreaker's counters arrive through `EnterMods` and
/// `GameState::add_counters`, which is a direct write inside the entry
/// performer and deliberately not a proposal (CR 122.6a). So
/// `GameAction::AddCounters` reached `perform_action` from nothing, and
/// `subject_of`'s arm for it was dead. This card is the
/// first thing in the crate to propose one, which is a new engine path and so
/// carries the deliberate `PERFORMANCE_POOL` addition §3 asks for.
///
/// **`{G}` and not colorless, accepted knowingly.** `random_deck` gives green
/// to about a third of decks, so roughly half of two-player games have a
/// Battlegrowth in them at all — a real cost against a colorless card, and
/// there is no colorless printing of this shape to pick instead. It is paid
/// once and buys the exact text; the measured annihilation count is in
/// `codebase-state.md` and is what says the trade was worth making.
pub fn battlegrowth() -> Arc<CardData> {
    CardDataBuilder::new("Battlegrowth")
        .card_type(CardType::Instant)
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green], 0))
        .rules_text("Put a +1/+1 counter on target creature.")
        .ability(spell_ability(
            "Put a +1/+1 counter on target creature.",
            put_counters(target_permanent(creature()), CounterType::PlusOnePlusOne, 1),
        ))
        .build()
}

/// Bonesplitter — {1}
/// Artifact — Equipment
///
/// Equipped creature gets +2/+0.
/// Equip {1}
///
/// (Oracle text verified on Scryfall, 2026-09-04. Mirrodin; many reprints.)
///
/// # Why this one
///
/// The first registered Equipment, and the consumer of `Primitive::Attach`,
/// `GameAction::Attach` and `ActivationRestriction::OnlyAsSorcery` — the
/// reattachment path that makes CR 613.7e observable at all, since an Aura
/// attaches once. **Asymmetric** for the reason Holy Strength's doc gives:
/// +2/+0 fails an assertion if the row is transposed or applied twice, where
/// Leonin Scimitar's and Short Sword's +1/+1 pass both. One generic mana to
/// cast and one to equip is the cheapest that leaves.
///
/// # What it makes reachable
///
/// - **CR 702.6a / 602.5d** — an activated ability with a timing restriction,
///   the first in the pool; the random agent is offered it only at sorcery
///   speed.
/// - **CR 613.7e / 701.3c** — a permanent whose CR 613.7 timestamp changes
///   after it entered, and with it the rows its static abilities registered
///   (`GameState::attach`; `layers-architecture.md` §13a, LH-2).
/// - **CR 704.5p** — the Equipment detach SBA, measured at **0** in the
///   2026-09-01 fuzz re-audit because nothing could attach an Equipment.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// A new primitive, a new `GameAction`, and the first activation restriction:
/// three engine paths no other pooled card opens, and §3 asks that each be
/// measured rather than assumed.
pub fn bonesplitter() -> Arc<CardData> {
    CardDataBuilder::new("Bonesplitter")
        .card_type(CardType::Artifact)
        .subtype(Subtype::Artifact(ArtifactType::Equipment))
        .mana_cost(ManaCost::build(&[], 1))
        .rules_text("Equipped creature gets +2/+0.
Equip {1}")
        .ability(static_ability(
            "Equipped creature gets +2/+0.",
            gets(host(), 2, 0),
        ))
        .ability(equip("Equip {1}", vec![Cost::Mana(ManaCost::build(&[], 1))]))
        .build()
}

/// Frogmite — {4}
/// Artifact Creature — Frog, 2/2
///
/// Affinity for artifacts (This spell costs {1} less to cast for each artifact
/// you control.)
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// The same ability on a smaller body, registered and not pooled. Two
/// affinity cards is what lets a test put one on the battlefield and count it
/// for the other — the board on which a self-reduction reads a board the
/// first copy is standing on.
pub fn frogmite() -> Arc<CardData> {
    CardDataBuilder::new("Frogmite")
        .mana_cost(ManaCost::build(&[], 4))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Frog))
        .power_toughness(2, 2)
        .rules_text("Affinity for artifacts")
        .affinity_for("Affinity for artifacts", artifact())
        .build()
}

/// Loxodon Warhammer — {3}
/// Artifact — Equipment
///
/// Equipped creature gets +3/+0 and has trample and lifelink.
/// Equip {3}
///
/// (Oracle text verified on Scryfall, 2026-10-08. No rulings.)
///
/// # Why this one
///
/// Lifelink a creature has only while it is equipped, so whether it had it
/// is a question about a moment: as its ability's damage is dealt, or as it
/// left the battlefield before that (CR 702.15c). That is
/// `codebase-state.md` item 223's board. One static ability in two layers,
/// a Layer 7c pump beside two Layer 6 grants, as Opalescence's is in two
/// (CR 613.6).
///
/// Registered, not pooled: it opens no engine path Bonesplitter and Cobbled
/// Wings do not.
pub fn loxodon_warhammer() -> Arc<CardData> {
    CardDataBuilder::new("Loxodon Warhammer")
        .card_type(CardType::Artifact)
        .subtype(Subtype::Artifact(ArtifactType::Equipment))
        .mana_cost(ManaCost::build(&[], 3))
        .rules_text("Equipped creature gets +3/+0 and has trample and lifelink.\nEquip {3}")
        .ability(static_ability(
            "Equipped creature gets +3/+0 and has trample and lifelink.",
            gets(host(), 3, 0).and_gains_keyword(KeywordFlag::Trample).and_gains_keyword(KeywordFlag::Lifelink),
        ))
        .ability(equip("Equip {3}", vec![Cost::Mana(ManaCost::build(&[], 3))]))
        .build()
}

/// March of the Machines (simplified) — {4}{U}
/// Enchantment
/// Each noncreature artifact is an artifact creature with power and toughness
/// each equal to its mana value.
///
/// The CR's own worked example for rule 613.6, and a real card rather than an
/// invented one. The static ability applies in two layers over one filter:
/// `AddType(Creature)` in Layer 4, `SetPowerToughness` in Layer 7b, both over
/// "artifact that is not a creature". After Layer 4 runs, nothing matches that
/// filter any more — which is exactly why 613.6 says the effect "will continue
/// to be applied to the same set of objects in each other applicable layer".
pub fn march_of_the_machines() -> Arc<CardData> {
    let noncreature_artifact = artifact().and(non(creature()));

    CardDataBuilder::new("March of the Machines")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 4))
        .color(Color::Blue)
        .card_type(CardType::Enchantment)
        .rules_text("Each noncreature artifact is an artifact creature with power and toughness each equal to its mana value.")
        .ability(static_ability(
            "Each noncreature artifact is an artifact creature with power and toughness each equal to its mana value.",
            becomes_types(each(noncreature_artifact), TypeChange::adding(&[], &[CardType::Creature], &[]))
                .and_has_base_power_and_toughness(AmountExpr::AffectedManaValue, AmountExpr::AffectedManaValue),
        ))
        .build()
}

/// Myr Enforcer — {7}
/// Artifact Creature — Myr, 4/4
///
/// Affinity for artifacts (This spell costs {1} less to cast for each artifact
/// you control.)
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// That is the whole card: a body and the keyword. CR 702.41a *defines*
/// affinity as "This spell costs {1} less to cast for each [text] you
/// control", so [`CardDataBuilder::affinity_for`] writes that sentence and
/// nothing in the engine knows the word.
///
/// # In `PERFORMANCE_POOL`, and why
///
/// The pool's first cost ability that is the *spell's own*, so the first
/// measured game in which CR 601.2f's gather has a second source, the
/// castability preview reads a hand card's own ability list, and an
/// `AmountExpr::CountOf` runs at cast time rather than inside a layer walk.
/// Colorless, so every deck in the pool can cast it; an artifact itself, so
/// a second copy in a deck counts the first. Its `{7}` is why it is the one
/// pooled and Frogmite is not: the reduction has room to be large enough to
/// see, where a `{4}` body is castable without ever asking.
///
/// [`CardDataBuilder::affinity_for`]: crate::objects::card_data::CardDataBuilder::affinity_for
pub fn myr_enforcer() -> Arc<CardData> {
    CardDataBuilder::new("Myr Enforcer")
        .mana_cost(ManaCost::build(&[], 7))
        .card_type(CardType::Artifact)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Myr))
        .power_toughness(4, 4)
        .rules_text("Affinity for artifacts")
        .affinity_for("Affinity for artifacts", artifact())
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
/// is the second row's whole scope. **Registered and not pooled**, and
/// `PERFORMANCE_POOL`'s comment says why.
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
        // One paragraph, one ability (CR 113.2c), making two "can't"s.
        .ability(static_ability(
            "You can't lose the game and your opponents can't win the game.",
            Effect::Sequence(vec![
                Effect::Restriction(Box::new(RestrictionDef::new(Restriction::Event {
                    pattern: EventPattern::PlayerLoses,
                    affected: AffectedSet::players(PlayerSet::You),
                    by: None,
                }))),
                Effect::Restriction(Box::new(RestrictionDef::new(Restriction::Event {
                    pattern: EventPattern::PlayerWins,
                    affected: AffectedSet::players(PlayerSet::Opponents),
                    by: None,
                }))),
            ]),
        ))
        .build()
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
/// **Pooled**, with Parallel Lives — `PERFORMANCE_POOL`'s comment says why.
pub fn raise_the_alarm() -> Arc<CardData> {
    CardDataBuilder::new("Raise the Alarm")
        .mana_cost(ManaCost::build(&[ManaType::White], 1))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text("Create two 1/1 white Soldier creature tokens.")
        .ability(spell_ability("Create two 1/1 white Soldier creature tokens.",
            create_tokens(you(), 2, TokenDef::creature(1, 1, &[Color::White], &[CreatureType::Soldier])),
        ))
        .build()
}

/// Seat of the Synod
/// Artifact Land
/// {T}: Add {U}.
///
/// Here for one board and not for a rule: `ATOM-608.2b-002` needs a permanent
/// that can be both a land and a creature, and the engine's route to that is
/// March of the Machines animating an artifact land. An artifact land is also
/// CR 601.2c's own example ("Destroy target artifact and target land … can
/// target the same artifact land twice"), so it earns the registration twice
/// over.
///
/// Registered and unpooled: a land that taps for one blue adds nothing a basic
/// Island does not, and pooling it would move the stream to measure nothing.
///
/// # The rulings, and where each is tested
///
/// Scryfall lists none for this card.
pub fn seat_of_the_synod() -> Arc<CardData> {
    CardDataBuilder::new("Seat of the Synod")
        .card_type(CardType::Artifact)
        .card_type(CardType::Land)
        .rules_text("{T}: Add {U}.")
        .mana_ability_single(ManaType::Blue)
        .build()
}
