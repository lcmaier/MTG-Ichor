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

use crate::engine::layers::lookahead::compute_as_entering;
use crate::engine::layers::types::EffectiveCharacteristics;
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData};
use crate::state::game_state::GameState;
use crate::types::card_types::CardType;
use crate::types::effects::{AmountExpr, CounterType, Effect, ObjectSet};
use crate::types::ids::{AbilityId, ObjectId, PlayerId, SynthesizedAbility};
use crate::types::replacement::{EnterMods, EnterModsTemplate, EventPattern, ReplacementDef, Rewrite};

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
        id: AbilityId::derived_on(object, SynthesizedAbility::PlaneswalkerLoyalty),
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
    ability.id == AbilityId::derived_on(object, SynthesizedAbility::PlaneswalkerLoyalty)
}

/// The kind of counter an intrinsic "enters with" ability could give `card`:
/// loyalty, when it prints a loyalty number (CR 306.5b). Read off the printed
/// card on purpose, as the gate before a frame is walked; whether the object
/// has the ability as it enters is the frame's question.
pub fn intrinsic_entry_counter_kind(card: &CardData) -> Option<CounterType> {
    card.loyalty.is_some_and(|n| n > 0).then_some(CounterType::Loyalty)
}

/// What `id`'s own intrinsic "enters with" abilities give it as it enters
/// under `controller`, and nothing else: CR 306.5b's loyalty counters, read
/// off the ability the walk synthesizes onto its frame. So a planeswalker
/// gets its loyalty, and one Humility has stripped gets none, as through the
/// pipeline; the pipeline's other replacement effects are what a caller of
/// this skips. A scenario's permanents and `test_support`'s builders enter
/// with it.
pub fn intrinsic_entry_mods(game: &GameState, id: ObjectId, controller: PlayerId) -> EnterMods {
    let mut mods = EnterMods::NONE;
    // The frame is a walk a test counting walks would see; without a printed
    // number the ability gives none.
    if game.objects.get(&id).and_then(|obj| intrinsic_entry_counter_kind(&obj.card_data)).is_none() {
        return mods;
    }
    let Some(frame) = compute_as_entering(game, id, controller, &EnterMods::NONE) else {
        return mods;
    };
    for ability in frame.abilities.iter() {
        if !is_intrinsic_entry_ability(ability, id) {
            continue;
        }
        let Effect::Replacement(def) = &ability.effect else { continue };
        let Rewrite::EnterWith(template) = &def.rewrite else { continue };
        for row in &template.counters {
            if let AmountExpr::Fixed(n) = row.amount
                && n > 0
            {
                mods.merge(&EnterMods::with_counters(row.counter, n as u32));
            }
        }
    }
    mods
}
