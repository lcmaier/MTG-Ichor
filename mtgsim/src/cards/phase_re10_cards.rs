//! Cards for Phase RE-10 — extra phases (`replacement-architecture.md` §9).
//!
//! **One card, and it is the first producer CR 500.8 has ever had here.** The
//! turn's phases became data in this PR ([`crate::state::game_state::TurnPlan`]);
//! this is what splices into them.
//!
//! | Card | What it is the first of | CR |
//! |---|---|---|
//! | [`aggravated_assault`] | an effect that creates extra phases | 500.8 |
//!
//! It is also the first consumer of `Primitive::Untap` with an
//! `EffectRecipient::FilteredPermanents` — the arm `DealDamage` and
//! `CreateReplacement` already had. That arm rides in this PR rather than
//! waiting for a card of its own because this card's own first sentence is
//! "Untap all creatures you control", and without it the producer would ship
//! with no consumer.
//!
//! # What is not here, and what each waits for
//!
//! `o:"additional combat phase"` is **46 cards** (Scryfall, 2026-09-11) and
//! this is the only one in reach whole. **Seize the Day** needs flashback.
//! **World at War** needs rebound *and* "creatures that attacked this turn";
//! **Relentless Assault** needs the second of those. **Obeka, Splitter of
//! Seconds** is CR 500.9/500.10's extra *steps* and its ability is
//! **triggered**, so it is critical-path item 6's whatever RE does — the one
//! field it wants is named on `PlannedPhase` and `backlog.md` §2.17 stays open
//! for it. The rest of the 46 are triggers.
//!
//! # What a random deck can draw
//!
//! **Nothing — `PERFORMANCE_POOL` +0, and that is measured rather than
//! assumed.** Aggravated Assault is {2}{R} plus a {3}{R}{R} activation, which
//! is eight mana to use once, and every activation makes the game *bigger* —
//! the shape §3.1a rejected Altar's Reap for at +20.2%. Reachability is a
//! `--require` row instead, as it was for Circle of Protection: Red. The
//! engine's change needs no pooled card at all: the plan replaces
//! `next_phase`'s chain on every turn of every game, so the fuzz walks the new
//! cursor unforced.

use std::sync::Arc;

use crate::objects::card_data::{
    AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder,
};
use crate::state::game_state::PhaseType;
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::costs::Cost;
use crate::types::effects::{
    Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive,
};
use crate::types::ids::AbilityId;
use crate::types::mana::{ManaCost, ManaType};

/// "creatures you control" — CR 109.5's "you" is the source's controller.
fn creatures_you_control() -> ObjectFilter {
    ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    )
}

/// Aggravated Assault — {2}{R}
/// Enchantment
///
/// "{3}{R}{R}: Untap all creatures you control. After this main phase, there
///  is an additional combat phase followed by an additional main phase.
///  Activate only as a sorcery."
///
/// (Oracle text verified on Scryfall, 2026-09-14. Mercadian Masques; many
/// reprints.)
///
/// # Why this one
///
/// **The cheapest whole card for CR 500.8**, and the only one of the 46 that
/// needs nothing this engine does not have: `ActivationRestriction::OnlyAsSorcery`
/// has been enforced since Bonesplitter's equip (`cast.rs:345`), and its untap
/// is the recipient arm this PR adds.
///
/// **The two phases are one `Primitive::ExtraPhases`, not two.** "An additional
/// combat phase followed by an additional main phase" is one resolution
/// creating a run, and their printed order has to be the order they are
/// spliced rather than an accident of which splice ran second — which is also
/// what makes CR 500.8's *"the most recently created phase will occur first"*
/// fall out of a second activation's splice at the same index.
///
/// **The additional main phase is `PhaseType::Postcombat`, and the choice is
/// unobservable.** Checked rather than assumed: every production reader of the
/// two main types matches them as one arm asking "is this a main phase" —
/// `cast.rs:659`, `zones.rs:266` and `oracle/legality.rs:50`. CR 500.8 says
/// only "directly after the specified phase" and the CR names no additional
/// main phase either way. The axis a later card would grow is a field on
/// `PlannedPhase`, and its customer is the first card that prints a
/// distinction between the two main phases the engine must keep.
///
/// # Rulings
///
/// - *"You will normally activate this ability during your postcombat main
///   phase so you can untap any creatures that attacked."* (2023-09-01) —
///   advice rather than a rule, but it is the board the tests use: the untap
///   and the splice are one resolution, so the creatures that attacked are
///   untapped before the combat phase that lets them attack again.
/// - *"If you have enough mana, the ability may be activated more than once in
///   a turn."* (2004-10-04) — **this is CR 500.8's ordering test.** A second
///   activation splices at the same index and pushes the first activation's
///   pair later, which is the rule's "most recently created phase will occur
///   first" with no comparator anywhere.
pub fn aggravated_assault() -> Arc<CardData> {
    CardDataBuilder::new("Aggravated Assault")
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Enchantment)
        .rules_text(
            "{3}{R}{R}: Untap all creatures you control. After this main phase, \
             there is an additional combat phase followed by an additional main \
             phase. Activate only as a sorcery.",
        )
        .ability(AbilityDef {
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::OnlyAsSorcery,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::Mana(ManaCost::build(
                &[ManaType::Red, ManaType::Red],
                3,
            ))],
            effect: Effect::Sequence(vec![
                Effect::Atom(
                    Primitive::Untap,
                    EffectRecipient::FilteredPermanents(creatures_you_control()),
                ),
                Effect::Atom(
                    Primitive::ExtraPhases(vec![PhaseType::Combat, PhaseType::Postcombat]),
                    EffectRecipient::Controller,
                ),
            ]),
        })
        .build()
}
