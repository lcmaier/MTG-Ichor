//! Mercadian Masques (`mmq`), released 1999-10-04.

use std::sync::Arc;

use crate::cards::authoring::{create_replacement_for, skip, spell_ability, target_player};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::PhaseType;
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::PlayerSet;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::EventPattern;

/// Moment of Silence — {W}
/// Instant
///
/// > Target player skips their next combat phase this turn.
///
/// The **targeted** skip and the only phase-scoped one here. Its three rulings
/// are all consequences of the shape rather than special cases: `Uses::Once`
/// makes it one phase, `Duration::UntilEndOfTurn` makes a row that meets no
/// proposal expire unused, and the event's subject being the *active* player
/// makes a row on anybody else watch nothing.
///
/// The affected set is authored empty in both halves, which
/// `Primitive::CreateReplacement`'s `Target` arm requires: the shape is the
/// card's and the player is the resolution's.
/// # The rulings, and where each is tested
///
/// - *"The player skips their next combat phase this turn (if any). If they
///   manage to have two combat phases, then only their next one combat phase is
///   skipped."* → `Uses::Once`, and the second sentence is tested against a
///   second `BeginPhase { Combat }` proposal the fixture makes by moving the
///   cursor, because CR 500.8's extra phases are unbuilt. **RE-10 replaces the
///   fixture with Aggravated Assault.**
/// - *"It must be used before the combat phase starts or it has no effect."* →
///   `ATOM-614.10-002`: a row created during combat meets no proposal and
///   expires at cleanup unused.
/// - *"If cast on a player when it is not their turn, it has no effect."* →
///   falls out of the subject rather than being coded: a `BeginPhase` event is
///   about the **active** player, so a row scoped to anyone else watches
///   nothing this turn.
pub fn moment_of_silence() -> Arc<CardData> {
    CardDataBuilder::new("Moment of Silence")
        .mana_cost(ManaCost::build(&[ManaType::White], 0))
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text("Target player skips their next combat phase this turn.")
        .ability(spell_ability("Target player skips their next combat phase this turn.",
            create_replacement_for(target_player(), skip(EventPattern::BeginPhase { phase: Some(PhaseType::Combat) }, PlayerSet::Nobody).once())
                .until_end_of_turn(),
        ))
        .build()
}
