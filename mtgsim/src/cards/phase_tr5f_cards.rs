//! Phase TR-5f — creatures put onto the battlefield attacking, and removal
//! from combat (`triggers-architecture.md` §12).
//!
//! Every oracle text below was verified on Scryfall on 2026-10-16 and is
//! quoted verbatim; each card's rulings were read the same day and sit in its
//! own doc comment (`engineering-practices.md` §3.4).
//!
//! | Card | The path | Pooled |
//! |---|---|---|
//! | Hanweir Garrison | tokens put onto the battlefield attacking, each defender chosen as it enters (CR 508.4) | yes |
//! | Relentless Assault | "creatures that attacked this turn", the combat history's `Attacked` rows, and a second combat (CR 500.8) | no |

use std::sync::Arc;

use crate::cards::authoring::{attacks, triggered_ability, whenever};
use crate::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use crate::state::game_state::PhaseType;
use crate::types::card_types::{CardType, CreatureType, Subtype};
use crate::types::history::ObjectFact;
use crate::types::colors::Color;
use crate::types::effects::{
    AmountExpr, Effect, EffectRecipient, EntersAttacking, ExtraPhasesAfter, ObjectFilter, Primitive, TokenDef,
};
use crate::types::ids::AbilityId;
use crate::types::mana::{ManaCost, ManaType};
use crate::types::triggers::TriggerSubject;

/// A 1/1 red Human creature token, created tapped and attacking.
pub fn attacking_human_token() -> TokenDef {
    TokenDef {
        name: None,
        colors: vec![Color::Red],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Human)],
        supertypes: Vec::new(),
        power: Some(1),
        toughness: Some(1),
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: true,
        enters_attacking: Some(EntersAttacking::Chosen),
    }
}

/// Hanweir Garrison — {2}{R}
/// Creature — Human Soldier 2/3
///
/// > Whenever this creature attacks, create two 1/1 red Human creature tokens
/// > that are tapped and attacking.
/// > (Melds with Hanweir Battlements.)
///
/// It is one half of a meld pair, and the reminder text, which has no game
/// function and is not in its rules text (CR 207.2), is all it prints of
/// meld: Hanweir Battlements carries the meld ability, so nothing on this card
/// is dead.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr5f_integration_test`.
/// - *Each token's player, planeswalker or battle is chosen as the tokens are
///   created, and needn't be Hanweir Garrison's or the other token's* (#1) →
///   `each_token_chooses_what_it_attacks_at_four_seats`.
/// - *The tokens were never declared as attacking creatures* (#2) →
///   `a_token_put_onto_the_battlefield_attacking_never_attacked`.
/// - *Another attack trigger resolves before or after it, and a targeted one
///   can't target the tokens* (#3) →
///   `an_attack_trigger_beside_it_cannot_target_the_tokens`.
/// - #4 points at the Eldritch Moon mechanics article, a disposition.
pub fn hanweir_garrison() -> Arc<CardData> {
    let text = "Whenever this creature attacks, create two 1/1 red Human creature tokens that are tapped and attacking.";
    let two_humans =
        Effect::Atom(Primitive::CreateToken(attacking_human_token(), AmountExpr::Fixed(2)), EffectRecipient::Controller);
    CardDataBuilder::new("Hanweir Garrison")
        .mana_cost(ManaCost::build(&[ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .subtype(Subtype::Creature(CreatureType::Soldier))
        .power_toughness(2, 3)
        .rules_text(text)
        .ability(triggered_ability(text, whenever(attacks(TriggerSubject::ThisObject), two_humans)))
        .build()
}

/// Relentless Assault — {2}{R}{R}
/// Sorcery
///
/// > Untap all creatures that attacked this turn. After this main phase,
/// > there is an additional combat phase followed by an additional main
/// > phase.
///
/// "Creatures that attacked this turn" are those declared as attackers this
/// turn (CR 508.1), so a creature put onto the battlefield attacking is not
/// one (CR 508.4). The extra phases are Aggravated Assault's, anchored to
/// this main phase.
///
/// # The rulings, and where each is tested
///
/// In `phase_tr5f_integration_test`.
/// - *Only if it resolves during a main phase* (#1) →
///   `relentless_assault_adds_no_phase_outside_a_main_phase`.
/// - *A creature that attacks each turn if able must attack only once* (#2):
///   attack requirements are `cant-effects-architecture.md`'s `combat-restrictions`, a
///   disposition.
/// - *Cast on an opponent's turn, that opponent's creatures untap and that
///   opponent attacks again* (#3) →
///   `relentless_assault_on_an_opponents_turn_is_that_players_combat`.
/// - *Attack triggers trigger during each combat* (#4) →
///   `relentless_assault_untaps_what_attacked_and_attack_triggers_fire_again`.
pub fn relentless_assault() -> Arc<CardData> {
    let text = "Untap all creatures that attacked this turn. After this main phase, there is an additional combat phase followed by an additional main phase.";
    let attacked_this_turn =
        ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ThisTurn(ObjectFact::Attacked)));
    CardDataBuilder::new("Relentless Assault")
        .mana_cost(ManaCost::build(&[ManaType::Red, ManaType::Red], 2))
        .color(Color::Red)
        .card_type(CardType::Sorcery)
        .rules_text(text)
        .ability(AbilityDef {
            rules_text: text.into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Sequence(vec![
                Effect::Atom(Primitive::Untap, EffectRecipient::FilteredPermanents(attacked_this_turn)),
                Effect::Atom(
                    Primitive::ExtraPhases {
                        phases: vec![PhaseType::Combat, PhaseType::Postcombat],
                        after: ExtraPhasesAfter::ThisMainPhase,
                    },
                    EffectRecipient::Controller,
                ),
            ]),
        })
        .build()
}
