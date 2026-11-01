//! Mirage (`mir`), released 1996-10-08.

use std::sync::Arc;

use crate::cards::authoring::{
    at_beginning_of, create_delayed_trigger, create_replacement_from_a_chosen_source, every_permanent_or_player, lose_the_game, spell_ability,
    take_an_extra_turn_after_this_one, whenever, you, Whose,
};
use crate::objects::card_data::{CardData, CardDataBuilder};
use crate::state::game_state::StepType;
use crate::types::card_types::CardType;
use crate::types::colors::Color;
use crate::types::effects::Effect;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::replacement::{EventPattern, ReplacementDef, RetargetSpec, Rewrite, SourcePattern};
use crate::types::triggers::{DelayedDuration, DelayedTriggerTemplate, DelayedTurn};

/// Final Fortune — {R}{R}
/// Instant
///
/// > Take an extra turn after this one. At the beginning of that turn's end
/// > step, you lose the game.
///
/// One spell ability: the extra turn, then the delayed trigger in "that
/// turn", which names the queue entry the first instruction made
/// (`DelayedTurn::ThatExtraTurn`), so "that turn's end step" is whichever end
/// step that turn has. The trigger is the spell's and its controller's as it
/// resolved (CR 603.7d), so "you" is that player wherever the card has gone.
///
/// **Registered, not pooled**, for Time Walk's reason: an extra turn moves
/// `Avg turns/game` by design, and a loss at its end moves the outcomes.
///
/// It exercises a spell's delayed trigger (CR 603.7d) in "that turn", the extra
/// turn it made (CR 500.7).
///
/// # The rulings, and where each is tested
///
/// Both below.
/// - *"If multiple 'extra turn' effects resolve in the same turn, take them in
///   the reverse of the order that the effects resolved. In other words, the
///   most recently created extra turn is taken first."* (#1)
///   → `final_fortunes_turn_is_its_own_among_several`: a Time Walk resolving
///   after it is taken first, and the loss waits for Final Fortune's own turn.
/// - *"If you end up skipping the extra turn that is gained, you do not lose
///   the game."* (#2) → `a_skipped_final_fortune_turn_loses_nothing`.
pub fn final_fortune() -> Arc<CardData> {
    let text = "Take an extra turn after this one. At the beginning of that turn's end step, you lose the game.";
    let lose = DelayedTriggerTemplate {
        def: Arc::new(whenever(
            at_beginning_of(StepType::End, Whose::Each),
            lose_the_game(you()),
        )),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::ThatExtraTurn,
        rules_text: "At the beginning of that turn's end step, you lose the game.".into(),
    };
    CardDataBuilder::new("Final Fortune")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 0))
        .color(Color::Red)
        .card_type(CardType::Instant)
        .rules_text(text)
        .ability(spell_ability(
            text,
            Effect::Sequence(vec![
                take_an_extra_turn_after_this_one(you()),
                create_delayed_trigger(you(), lose),
            ]),
        ))
        .build()
}

/// Reflect Damage — {3}{R}{W}
///
/// > The next time a source of your choice would deal damage this turn, that
/// > damage is dealt to that source's controller instead.
///
/// **The card `ATOM-614.9-001`'s second half was waiting for**, and it is here
/// because RD-3 landed the source choice it needs: a `Uses::Once` redirect
/// whose destination is a *player*. When that player has left the game the
/// redirect does nothing — and the row is still there, which is the atom's "the
/// shield is NOT used up" and decision 7's `took_effect: false` in one board.
///
/// Two things separate it from every other chosen-source card (Circle of
/// Protection: Red, Reverse Damage, Dark Sphere).
/// Its affected sets are **everything** — "would deal damage", with no "to you"
/// — so it is the first row whose `Filter { All }` and `PlayerSet::Everyone`
/// are load-bearing on a redirect. And its rewrite reads the *damage's* source
/// rather than its own, which is `RetargetSpec::ToDamageSourceController`.
///
/// Scryfall lists no rulings (2026-09-09).
pub fn reflect_damage() -> Arc<CardData> {
    CardDataBuilder::new("Reflect Damage")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::White], 3))
        .color(Color::Red)
        .color(Color::White)
        .card_type(CardType::Instant)
        .rules_text(
            "The next time a source of your choice would deal damage this turn, that damage \
             is dealt to that source's controller instead.",
        )
        .ability(spell_ability("The next time a source of your choice would deal damage this turn, that damage is dealt to that source's controller instead.",
            create_replacement_from_a_chosen_source(
                ReplacementDef::new(
                    EventPattern::DealDamage { source: Some(SourcePattern::chosen()), combat: None },
                    every_permanent_or_player(),
                    Rewrite::Retarget(RetargetSpec::ToDamageSourceController),
                )
                .once(),
            )
            .until_end_of_turn(),
        ))
        .build()
}
