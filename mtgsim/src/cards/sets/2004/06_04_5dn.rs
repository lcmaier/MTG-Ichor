//! Fifth Dawn (`5dn`), released 2004-06-04.

use std::sync::Arc;

use crate::cards::authoring::{
    add_mana, artifact, becomes_unattached, draw_cards, equip, gets, host, lose_life, mana_ability, sacrifice, skip, spell_ability, static_ability,
    static_replacement, triggered_ability, triggering_object, whenever, you,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::{ArtifactType, CardType, Subtype};
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{AmountExpr, Effect, ManaOutput, PlayerSet};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::EventPattern;
use crate::types::triggers::{EventRole, TriggerSubject};

/// Doubling Cube — {2}
/// Artifact
///
/// > {3}, {T}: Double the amount of each type of unspent mana you have.
///
/// **The corpus's own integration test for CR 106.6, and a mana ability with
/// {T} on a permanent that is not a land.** The atomic-test session that
/// wrote `ATOM-106.6-001` named this card as the integration test for
/// restricted mana under doubling and deferred it to the suite; it lands
/// here because RE-9 is where a production became an event. Its first ruling
/// is what makes it RE-9's: *"Doubling Cube's ability is a mana
/// ability"* — CR 605.1a, no target, adds mana — and its cost includes {T},
/// so by CR 106.12 tapping it is "tapping a permanent for mana" and Mana
/// Reflection doubles what it produces.
///
/// The amount is the first dynamic one a mana ability has carried:
/// `AmountExpr::UnspentMana(type)` per type, read off the activating
/// player's pool when the ability resolves — after the {3} is paid, since an
/// activation pays its costs before the ability resolves (CR 602.2, 605.3b).
/// The output lists all six types; the performer adds nothing for a type at
/// zero. Pooled since MA-1, whose check reads the doubling as a transform of
/// the demand (`mana-architecture.md` §3.3) and whose window offers it once
/// the pool can pay its {3}; no window had offered it before.
///
/// # The rulings, and where each is tested
///
/// - *"Doubling Cube's ability is a mana ability."* → activated through
///   `activate_mana_ability`, and its production says `tapped_for_mana`, so
///   Mana Reflection doubles the doubling
///   (`doubling_cube_is_tapped_for_mana_so_mana_reflection_doubles_its_doubling`).
/// - *"The 'type' of mana is its color, or lack thereof."* → `UnspentMana`
///   is asked per [`ManaType`], colorless included.
/// - *"Any restrictions on the unspent mana aren't copied. For example, if you
///   have {C}{W}{W}{B} with no restrictions on it and {U}{U}{U} that can be
///   used only to cast artifact spells, you'll end up with
///   {C}{C}{W}{W}{W}{W}{B}{B}, {U}{U}{U} that can be used only to cast
///   artifact spells, and {U}{U}{U} that can be used for anything."* → the
///   ruling's own board: restricted units counted by their type, the copies
///   free (`doubling_cube_counts_restricted_mana_and_copies_it_unrestricted`).
///   CR 106.6's sentence — a restriction "doesn't affect the mana's type" —
///   is what makes the restricted {U}{U}{U} count as three blue.
pub fn doubling_cube() -> Arc<CardData> {
    CardDataBuilder::new("Doubling Cube")
        .mana_cost(ManaCost::build(&[], 2))
        .card_type(CardType::Artifact)
        .rules_text("{3}, {T}: Double the amount of each type of unspent mana you have.")
        .ability(mana_ability(
            "{3}, {T}: Double the amount of each type of unspent mana you have.",
            vec![Cost::Mana(ManaCost::build(&[], 3)), Cost::TapSelf],
            add_mana(ManaOutput {
                mana: [ManaType::White, ManaType::Blue, ManaType::Black, ManaType::Red, ManaType::Green, ManaType::Colorless]
                    .into_iter()
                    .map(|t| (t, AmountExpr::UnspentMana(t)))
                    .collect(),
                special: Vec::new(),
            }),
        ))
        .build()
}

/// Eon Hub — {5}
/// Artifact
///
/// > Players skip their upkeep steps.
///
/// The static whose scope is **everyone**, which is what a per-player counter
/// could never have expressed: the row is one effect that applies to each
/// player's upkeep in turn, gathered off the artifact's *effective* ability
/// list, so Humility or CR 305.7 taking the ability away takes the skip away
/// with it. CR 616.1's chooser is the affected player — the one whose upkeep it
/// is — so on a four-player table this asks nobody, four times a round.
///
/// **The pooled card of the PR.** Colorless at five, so every deck can cast
/// it, and every player's upkeep for the rest of the game is then a proposal
/// that goes nowhere — the first measured card whose cost is a *dropped*
/// turn-structure event.
/// # The rulings, and where each is tested
///
/// - *"The upkeep step is skipped entirely. The turn proceeds from untap step
///   to draw step."* → the event log, asserted as the absence of a
///   `StepBegin { Upkeep }` between the untap and draw ones.
/// - *"Upkeep-triggered abilities don't trigger, and 'activate only during your
///   upkeep' abilities can't be activated."* → the first half falls out and is
///   item 6's: a step that does not begin emits no event for a trigger to read,
///   which is why RE-1 went first. The second half has **no facility to
///   assert against** — `ActivationRestriction` has no step-scoped arm and no
///   registered card carries one. Recorded, not skipped.
/// - *"Any triggered abilities that triggered during the untap step will go
///   onto the stack at the start of the draw step."* → item 6's, same reason.
///   Both trigger-shaped rulings are booked as two tests item 6 owes
///   (`codebase-state.md` item 121, which names the board and sizes them).
pub fn eon_hub() -> Arc<CardData> {
    CardDataBuilder::new("Eon Hub")
        .mana_cost(ManaCost::build(&[], 5))
        .card_type(CardType::Artifact)
        .rules_text("Players skip their upkeep steps.")
        .ability(static_replacement("Players skip their upkeep steps.", skip(
            EventPattern::BeginStep { step: Some(StepType::Upkeep) },
            PlayerSet::Everyone,
        )))
        .build()
}

/// Grafted Wargear — {3}
/// Artifact — Equipment
///
/// > Equipped creature gets +3/+2.
/// > Whenever this Equipment becomes unattached from a permanent, sacrifice
/// > that permanent.
/// > Equip {0} ({0}: Attach to target creature you control. Equip only as a
/// > sorcery.)
///
/// "Becomes unattached" is one record on every route (CR 701.3d), and it
/// looks back (CR 603.10c), so the Wargear's own trigger fires when it is
/// the one that left. "That permanent" is the object it was attached to, as
/// the existence it was (CR 400.7): a creature that has left is not
/// sacrificed, nor the new object it became.
///
/// It exercises "becomes unattached" on every route CR 701.3d counts, looking
/// back (CR 603.10c), and "that permanent" as the existence it was.
///
/// # Not pooled, and why
///
/// Equip {0} at sorcery speed is offered to the random agent every main
/// phase, and every move to another creature sacrifices the one it left:
/// forced into every `stress` deck, 200 games at seed 12345 and two seats,
/// it moved 262 times, and the average game ran 26.9 turns against 25.4
/// (`fuzz-record.md`, the TR-4b block). It ends no game, so it is registered
/// for `stress` to hunt with, and Bramble Elemental, pooled, carries
/// "becomes attached" beside it.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr4b_integration_test`.
/// - *"Grafted Wargear becomes unattached from the creature it's equipping if
///   you equip it to a new creature, if Grafted Wargear leaves the
///   battlefield, if the equipped creature ceases to be a creature, or if
///   Grafted Wargear ceases to be an Equipment. (It also becomes unattached
///   if the equipped creature leaves the battlefield, but the triggered
///   ability won't do anything in that case.)"* (#1) → one test per route.
/// - *"You may activate this card's equip ability targeting the creature
///   that it is currently attached to, even if it's the only creature you
///   control. ..."* (#2) →
///   `grafted_wargear_equipped_again_to_its_creature_sacrifices_nothing`.
/// - *"If your Grafted Wargear somehow ends up on an opponent's creature and
///   it then becomes unattached from that creature, you won't be able to
///   sacrifice that creature because you don't control it."* (#3) →
///   `grafted_wargear_does_not_sacrifice_an_opponents_creature`.
pub fn grafted_wargear() -> Arc<CardData> {
    let text = "Whenever this Equipment becomes unattached from a permanent, sacrifice that permanent.";
    CardDataBuilder::new("Grafted Wargear")
        .mana_cost(ManaCost::build(&[], 3))
        .card_type(CardType::Artifact)
        .subtype(Subtype::Artifact(ArtifactType::Equipment))
        .rules_text("Equipped creature gets +3/+2.\nWhenever this Equipment becomes unattached from a permanent, sacrifice that permanent.\nEquip {0}")
        .ability(static_ability(
            "Equipped creature gets +3/+2.",
            gets(host(), 3, 2),
        ))
        .ability(triggered_ability(
            text,
            whenever(
                becomes_unattached(TriggerSubject::ThisObject, TriggerSubject::Any),
                sacrifice(triggering_object(EventRole::Host)),
            ),
        ))
        .ability(equip("Equip {0}", Vec::new()))
        .build()
}

/// Krark-Clan Ironworks — {4}
/// Artifact
///
/// Sacrifice an artifact: Add {C}{C}.
///
/// (Oracle text verified on Scryfall, 2026-09-07.)
///
/// The mana ability whose cost is a sacrifice, and the reason CR 601.2g's
/// window is worth anything: activated *while* another cost is being paid, it
/// takes artifacts off the board after the total was locked. Banned in Modern
/// in 2019 with "excessively arcane rules interactions using mana ability
/// timing windows" named as a factor — every one of those windows is a step
/// of CR 601.2 (`cost-architecture.md` §3.11).
///
/// **It can eat itself.** "An artifact" is every artifact its controller has,
/// including this one; nothing about paying a cost excludes the permanent
/// whose cost it is, and step 3 of the judged loop depends on that.
pub fn krark_clan_ironworks() -> Arc<CardData> {
    CardDataBuilder::new("Krark-Clan Ironworks")
        .mana_cost(ManaCost::build(&[], 4))
        .card_type(CardType::Artifact)
        .rules_text("Sacrifice an artifact: Add {C}{C}.")
        .ability(mana_ability(
            "Sacrifice an artifact: Add {C}{C}.",
            vec![Cost::Sacrifice(artifact(), 1)],
            add_mana(ManaOutput { mana: vec![(ManaType::Colorless, AmountExpr::Fixed(2))], special: vec![] }),
        ))
        .build()
}

/// Night's Whisper - {1}{B}
/// Sorcery
/// You draw two cards and lose 2 life
pub fn nights_whisper() -> Arc<CardData> {
    CardDataBuilder::new("Night's Whisper")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Sorcery)
        .rules_text("You draw two cards and lose 2 life.")
        .ability(spell_ability(
            "You draw two cards and lose 2 life.",
            Effect::Sequence(vec![
                draw_cards(you(), 2),
                lose_life(you(), 2)
            ]),
        ))
        .build()
}
