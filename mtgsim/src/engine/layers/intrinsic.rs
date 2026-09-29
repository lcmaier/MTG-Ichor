//! CR 306.5b — a planeswalker's intrinsic "enters with" ability, synthesized
//! onto its frame at the end of layer 4 (`replacement-architecture.md`
//! §3.5).
//!
//! > 306.5b A planeswalker has the intrinsic ability "This permanent enters
//! > with a number of loyalty counters on it equal to its printed loyalty
//! > number." This ability creates a replacement effect (see rule 614.1c).
//!
//! CR 305.6's intrinsic mana abilities are `land_types`', synthesized while
//! layer 4 applies. This one waits for the end of layer 4, because CR 306.5b
//! gives it to a planeswalker and an object's types are settled only there.
//! Being on the frame is what makes it an ability like any other: Layer 6
//! can remove it (Humility over a planeswalker that is a creature), a Layer 4
//! effect that makes a permanent stop being a planeswalker takes it away,
//! and the replacement gather's source 1a finds it on the entering object as
//! it finds a printed "enters with", so CR 616.1 can order it. Source 1a is
//! the only gather leg it needs: it modifies its own object's entry and
//! nothing else, and that entry is gathered off the frame with no gate.
//!
//! CR 310.4b gives a battle the same ability with defense counters, and joins
//! this module with battles (`backlog.md` §2.23).

use std::sync::Arc;

use crate::engine::layers::types::EffectiveCharacteristics;
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction};
use crate::types::card_types::CardType;
use crate::types::effects::{CounterType, Effect, ObjectSet};
use crate::types::ids::{AbilityId, ObjectId};
use crate::types::replacement::{EnterModsTemplate, EventPattern, ReplacementDef, Rewrite};

/// The tag `AbilityId::derived_on` gives CR 306.5b's ability. CR 305.6's
/// abilities use the land types' discriminants, which stay far below it.
const LOYALTY_TAG: u8 = 0xF0;

/// Put CR 306.5b's ability on `chars`, the frame of `object` at the end of
/// layer 4, if the object is a planeswalker there.
///
/// The count is read here, off the frame the gather reads the ability from,
/// so it is a constant by the time anything applies it
/// (`EnterModsTemplate::is_fixed`): CR 306.5b's printed loyalty number, which
/// for a copy is the copy's (CR 707.2), or 0 when there is none (CR 107.2).
pub(super) fn add_intrinsic_entry_abilities(chars: &mut EffectiveCharacteristics, object: ObjectId) {
    if !chars.types.contains(&CardType::Planeswalker) {
        return;
    }
    let loyalty = chars.loyalty.filter(|n| *n > 0).unwrap_or(0) as u32;
    Arc::make_mut(&mut chars.abilities).push(AbilityDef {
        id: AbilityId::derived_on(object, LOYALTY_TAG),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect: Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::SourceOnly,
            Rewrite::EnterWith(EnterModsTemplate::with_counters(CounterType::Loyalty, loyalty)),
        ))),
        instances: Vec::new(),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    });
}

/// Is `ability` an intrinsic "enters with" ability this module gave `object`?
pub fn is_intrinsic_entry_ability(ability: &AbilityDef, object: ObjectId) -> bool {
    ability.id == AbilityId::derived_on(object, LOYALTY_TAG)
}
