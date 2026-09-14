//! Phase LJ — the zone-reaching `ObjectSet` (`layers-architecture.md` §13c).
//!
//! Two cards, and they are one board: the board CR 614.12's own worked
//! example describes, which is ATOM-614.12-001. Both oracle texts were
//! verified on Scryfall on 2026-09-14 and are quoted verbatim.
//!
//! **Why these two and not Wonder.** Every zone-reaching effect asks two zone
//! questions — where its *source* is, and where the objects it *affects* are.
//! LJ answers only the second; the first is CR 113.6 (`roadmap-v2.md` A5). The
//! Jailer's source is on the battlefield and only the cards it reaches are in
//! graveyards, so it needs nothing from A5. Wonder is the mirror image — a
//! static ability functioning *from* a graveyard — and is A5's card, not this
//! phase's.

use std::sync::Arc;

use crate::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::colors::Color;
use crate::types::effects::{Duration, Effect, EffectRecipient, ObjectFilter, Primitive};
use crate::types::ids::new_ability_id;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::zones::ZoneSet;

/// Yixlid Jailer — {1}{B}
/// Creature — Zombie Wizard, 2/1
/// "Cards in graveyards lose all abilities."
///
/// **The first consumer of a filter that reaches another zone**, and the card
/// `codebase-state.md` item 9's correction 1 named for the job. Nothing about
/// it is new except the `ZoneSet`: `Primitive::LoseAllAbilities` already
/// existed and already writes `Channels::ABILITIES | Channels::KEYWORDS`, so
/// the row is the one Humility registers with a different affected set.
///
/// `ObjectFilter::All` and not a type leaf: the card says *cards*, with no
/// qualifier, and `ZoneSet::GRAVEYARD` alone is the whole restriction. Every
/// graveyard, not just its controller's — there is no `ByOwner` here because
/// the card names none.
///
/// # What the layer walk does with it
///
/// The row makes every graveyard card a member of the pass (`Board::seed`),
/// which is the change LJ is: before it, a graveyard card was a `NonMember`
/// and got printed characteristics plus its own CDAs. It costs nothing on a
/// board without this card, because `RegistryScopeSummary::reachable_zones`
/// is `BATTLEFIELD` there and the seed's loop does not run.
///
/// # The ruling this card is famous for, and why it is not this phase's
///
/// Yixlid Jailer's own rulings are about cards *changing zones* — a creature
/// card that leaves the graveyard has its abilities again, and a card put into
/// the graveyard already having lost its abilities does not keep them lost.
/// Both fall out of the row being re-read every pass rather than captured, so
/// neither needs code here. What the card cannot yet reach is anything that
/// *cares* about a graveyard card's abilities — flashback, retrace, Bridge
/// from Below's trigger — because CR 113.6 decides which of those function in
/// a graveyard at all, and that is A5.
pub fn yixlid_jailer() -> Arc<CardData> {
    CardDataBuilder::new("Yixlid Jailer")
        .mana_cost(ManaCost::build(&[ManaType::Black], 1))
        .color(Color::Black)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Zombie))
        .subtype(Subtype::Creature(CreatureType::Wizard))
        .power_toughness(2, 1)
        .rules_text("Cards in graveyards lose all abilities.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::LoseAllAbilities(Duration::WhileSourceOnBattlefield),
                EffectRecipient::FilteredObjectsIn(ObjectFilter::All, ZoneSet::GRAVEYARD),
            ),
        })
        .build()
}

/// Scarwood Treefolk — {3}{G}
/// Creature — Treefolk, 3/5
/// "This creature enters tapped."
///
/// **The Jailer's partner, and the half of ATOM-614.12-001 that asks the
/// question.** The atom's board is the two of them, the Treefolk in a
/// graveyard, and the assertion is that reanimating it puts it onto the
/// battlefield **tapped** — even though, sitting in the graveyard, the Jailer
/// had taken its abilities away.
///
/// CR 614.12 is why: an entry replacement is checked against the permanent as
/// it *would exist on the battlefield*, and on the battlefield the Jailer does
/// not reach it. `Board::in_zones_or_entering` is that sentence in one line —
/// an entering object is admitted by a row iff the row reaches the
/// battlefield, whatever zone it is still sitting in.
///
/// A creature rather than `phase_rc_cards::idyllic_beachfront`'s land, and
/// that matters: the land's "enters tapped" is stripped by Blood Moon through
/// CR 305.7, which is a *battlefield*-scoped effect and so tests the opposite
/// leg. This one is only reachable through a graveyard.
pub fn scarwood_treefolk() -> Arc<CardData> {
    CardDataBuilder::new("Scarwood Treefolk")
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .color(Color::Green)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Treefolk))
        .power_toughness(3, 5)
        .rules_text("This creature enters tapped.")
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: crate::objects::card_data::ActivationRestriction::None,
            id: new_ability_id(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(
                crate::types::replacement::ReplacementDef::new(
                    crate::types::replacement::EventPattern::EnterBattlefield { cast: None },
                    crate::types::effects::ObjectSet::SourceOnly,
                    crate::types::replacement::Rewrite::EnterWith(
                        crate::types::replacement::EnterModsTemplate::tapped(),
                    ),
                ),
            )),
        })
        .build()
}
