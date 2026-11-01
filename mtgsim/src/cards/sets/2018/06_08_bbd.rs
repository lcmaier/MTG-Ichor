//! Battlebond (`bbd`), released 2018-06-08.

use std::sync::Arc;

use crate::cards::authoring::{create_replacement_for, draw_cards, exile, set_life_total, spell_ability, this_object, you};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::{Effect, PlayerSet};
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, ReplacementDef, Rewrite};

/// Stunning Reversal — {3}{B}
/// Instant
///
/// > The next time you would lose the game this turn, instead draw seven cards
/// > and your life total becomes 1.
/// > Exile Stunning Reversal.
///
/// A `Primitive::CreateReplacement` row — `PlayerLoses`, `You`, `Uses::Once`,
/// `UntilEndOfTurn`, `Prevent` with the two-atom rider — and then the spell
/// exiling itself as CR 608.2c's second instruction, which is not part of the
/// row: the row outlives the card, and CR 608.2m lets the spell finish
/// resolving from exile.
/// # The rulings, and where each is tested
///
/// - *"If each player would lose the game at the same time, but Stunning
///   Reversal's effect applies to you losing the game, you win the game as
///   soon as everyone else has lost the game. This is true even if you'd lose
///   the game immediately afterwards, perhaps because you don't have seven
///   cards in your library to draw or because you couldn't gain life to raise
///   your life total to 1."* → the four-player board: four `PlayerLoses`
///   members, one replaced, and CR 104.2a settled by the *batch* — before the
///   rider draws, and never overwritten by the loss that follows.
///   → `stunning_reversal_when_everyone_would_lose_at_once_its_controller_wins`
/// - *"While the replacement effect it creates lasts until end of turn (or
///   until the event it replaces), Stunning Reversal is exiled as it
///   resolves."* → the card is in exile and not in a graveyard after
///   resolution, and the row is still registered.
///   → `stunning_reversal_is_exiled_as_it_resolves_and_its_row_survives_it`
/// - *"If an effect says you can't lose the game, Stunning Reversal's effect
///   can't apply."* → Platinum Angel's row refuses the proposal, and the
///   unspent row is still there for a loss the Angel no longer refuses.
///   → `under_platinum_angel_stunning_reversal_neither_applies_nor_is_spent`
/// - *"If an effect says that an opponent wins the game, Stunning Reversal's
///   effect doesn't apply."* → the same fact as Exquisite Archangel's seventh
///   ruling: an opponent's win is not a loss event.
///   → `an_opponents_win_is_not_a_loss_exquisite_archangel_can_replace`
/// - *"Stunning Reversal's effect does nothing if you concede the game. A
///   player who concedes leaves the game."* → recorded, no harness.
/// - *"Stunning Reversal's effect applies any time you would lose the game,
///   even if you're not losing due to your life total being 0 or less."* →
///   `EventPattern::PlayerLoses` has no reason; tested on the Archangel's
///   poison board, which is the same pattern on the same event.
///   → `exquisite_archangel_applies_to_a_poison_loss_and_then_the_poison_still_loses`
/// - *"For your life total to become 1, you gain or lose the appropriate
///   amount of life. For example, if your life total is 4 when Stunning
///   Reversal's effect applies, it will cause you to lose 3 life;
///   alternatively, if your life total is -5 when it applies, it will cause
///   you to gain 6 life."* → the gain leg is the natural board (lethal damage
///   leaves you below 0); the loss leg is `ATOM-119.5-001`'s fixture.
///   → `stunning_reversal_from_minus_five_is_a_gain_of_six`,
///   `setting_a_life_total_lower_is_a_loss_of_the_difference`
/// - *"If you have fewer than seven cards in your library, you'll lose the
///   game immediately after applying Stunning Reversal's replacement
///   effect."* → the rider's draws flag CR 704.5b, the check repeats because
///   the game changed, and the spent row cannot see the second proposal.
///   → `stunning_reversal_with_a_short_library_loses_immediately_after`
pub fn stunning_reversal() -> Arc<CardData> {
    CardDataBuilder::new("Stunning Reversal")
        .mana_cost(ManaCost::build(&[ManaType::Black], 3))
        .color(Color::Black)
        .card_type(CardType::Instant)
        .rules_text(
            "The next time you would lose the game this turn, instead draw seven cards and your life total becomes 1.\nExile Stunning Reversal.",
        )
        .ability(spell_ability(
            "The next time you would lose the game this turn, instead draw seven cards and your life total becomes 1.",
            create_replacement_for(
                you(),
                ReplacementDef::new(EventPattern::PlayerLoses, PlayerSet::You, Rewrite::Prevent)
                    .once()
                    .with_then(Effect::Sequence(vec![draw_cards(you(), 7), set_life_total(you(), 1)])),
            )
            .until_end_of_turn(),
        ))
        // CR 608.2c's second instruction, on the spell itself.
        .ability(spell_ability(
            "Exile Stunning Reversal.",
            exile(this_object()),
        ))
        .build()
}
