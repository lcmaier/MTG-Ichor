//! RH-3: the record's new object, its applied replacements, and its moment
//! (`replacement-architecture.md` §2b.4–§2b.6).
//!
//! What a zone change's record holds and when it is read:
//! - the object the move made (CR 400.7e), which "it" and "them" find
//!   afterwards, and only in the zone it went to (CR 603.6c);
//! - the replacements that applied to it, in order (CR 614.5), which a "this
//!   way" and a "who exiled it" read;
//! - the instant each trigger reads it at: immediately after the event, before
//!   the riders its replacements queued (CR 603.4, 615.5).

use std::sync::Arc;

use mtgsim::cards::authoring::{dies, triggered_ability, whenever};
use mtgsim::engine::actions::GameAction;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    creature_with_ability, put_on_battlefield, put_in_library, setup_two_player_game,
    static_ability, test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::{CardType, EnchantmentType, Subtype};
use mtgsim::types::effects::{
    Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerRef, Primitive, ReturnUnder, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use mtgsim::types::triggers::{Multiplicity, TriggerEvent, TriggerSubject};
use mtgsim::types::zones::{Zone, ZoneChangePattern};
use mtgsim::ui::decision::DecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Put every pending trigger on the stack.
fn place(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).expect("placing");
}

/// Place what triggered and resolve the stack down to empty.
fn resolve_everything(game: &mut GameState) {
    place(game, &test_dp());
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&test_dp()).expect("resolving");
        place(game, &test_dp());
    }
}

fn a_creature() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Creature)
}

/// "If a creature you control would die, put it into its owner's graveyard,
/// then exile it." A replacement that leaves the death as it was, and whose
/// rider (CR 615.5) moves the creature again right after it died.
fn exiles_your_creatures_after_they_die() -> Arc<CardData> {
    let replacement = ReplacementDef::new(
        EventPattern::ZoneChange {
            change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: None },
            object: None,
        },
        ObjectSet::battlefield_filter(ObjectFilter::And(
            Box::new(a_creature()),
            Box::new(ObjectFilter::ByController(PlayerRef::You)),
        )),
        Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Graveyard }),
    )
    .with_then(Effect::Atom(
        Primitive::Exile,
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)),
    ));
    card_of_type_with(
        "Fixture: exiles your creatures right after they die",
        CardType::Enchantment,
        static_ability(Effect::Replacement(Box::new(replacement))),
    )
}

/// A card of one type with one ability.
fn card_of_type_with(name: &str, card_type: CardType, ability: mtgsim::objects::card_data::AbilityDef) -> Arc<CardData> {
    CardDataBuilder::new(name).card_type(card_type).ability(ability).build()
}

/// "When this creature dies, return it to the battlefield under its owner's
/// control."
fn returns_when_it_dies() -> Arc<CardData> {
    creature_with_ability(
        "Fixture: returns when it dies",
        1,
        1,
        triggered_ability(
            "",
            whenever(
                dies(TriggerSubject::ThisObject),
                Effect::Atom(Primitive::ReturnToBattlefield(ReturnUnder::Owner), EffectRecipient::TriggeringObject),
            ),
        ),
    )
}

// ---------------------------------------------------------------------------
// The object the move made (CR 400.7e, 603.6c)
// ---------------------------------------------------------------------------

/// A rider moves the creature again before its dies trigger is dispatched.
/// "Return it" names the object its death made, in the graveyard, and CR
/// 603.6c looks for it only there: exiled since, it is a new object, and the
/// trigger does nothing. Read off the store at dispatch, "it" was the exiled
/// card, and it came back.
#[test]
fn a_rider_that_moves_the_dead_creature_again_leaves_return_it_nothing_to_find() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, exiles_your_creatures_after_they_die(), 0);
    let returner = put_on_battlefield(&mut game, returns_when_it_dies(), 0);

    game.execute_action(GameAction::Destroy { object: returner }, &test_ctx()).unwrap();
    assert_eq!(game.get_object(returner).unwrap().zone, Zone::Exile, "the rider exiled it after it died");
    assert_eq!(game.pending_triggers.len(), 1, "its dies trigger");

    resolve_everything(&mut game);
    assert_eq!(
        game.get_object(returner).unwrap().zone,
        Zone::Exile,
        "CR 603.6c: the dies trigger looks for the card only in the graveyard, and the exiled card is a new object"
    );
}

/// "When this Aura is put into a graveyard from the battlefield, return it to
/// its owner's hand", shaped like Rancor.
fn aura_returning_to_hand() -> Arc<CardData> {
    CardDataBuilder::new("Fixture: returns to hand from the graveyard")
        .card_type(CardType::Enchantment)
        .subtype(Subtype::Enchantment(EnchantmentType::Aura))
        .enchant_filter(SelectionFilter::Permanent(a_creature()))
        .ability(triggered_ability(
            "",
            whenever(
                dies(TriggerSubject::ThisObject),
                Effect::Atom(Primitive::ReturnToHand, EffectRecipient::TriggeringObject),
            ),
        ))
        .build()
}

/// The Aura goes to the graveyard by CR 704.5m once its creature is gone, and
/// its own trigger finds the new object it became there (CR 400.7e).
// COVERS: ATOM-400.7e-001
#[test]
fn an_auras_own_graveyard_trigger_finds_the_aura_it_became() {
    let mut game = setup_two_player_game();
    let host = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let aura = put_on_battlefield(&mut game, aura_returning_to_hand(), 0);
    game.execute_action(GameAction::Attach { attachment: aura, host }, &test_ctx()).unwrap();

    game.execute_action(GameAction::Destroy { object: host }, &test_ctx()).unwrap();
    resolve_everything(&mut game);

    assert_eq!(game.get_object(aura).unwrap().zone, Zone::Hand, "CR 400.7e: the trigger finds the Aura in the graveyard");
}

/// "When this is put into a graveyard from anywhere, shuffle it into its
/// owner's library", shaped like Vigor's.
fn shuffles_in_from_anywhere() -> Arc<CardData> {
    creature_with_ability(
        "Fixture: shuffles in from anywhere",
        2,
        2,
        triggered_ability(
            "",
            whenever(
                TriggerEvent::ZoneChange {
                    subject: TriggerSubject::ThisObject,
                    change: ZoneChangePattern { from: None, to: Some(Zone::Graveyard), act: None },
                    by: None,
                    owner: None,
                    multiplicity: Multiplicity::PerOccurrence,
                },
                Effect::Atom(Primitive::ShuffleIntoLibrary, EffectRecipient::TriggeringObject),
            ),
        ),
    )
}

/// CR 400.7e's non-example: a "from anywhere" trigger is matched after the
/// move (CR 603.6c), against the card already in the graveyard, and needs no
/// look-back to find it.
// COVERS: ATOM-400.7e-002
#[test]
fn a_from_anywhere_trigger_finds_the_card_where_it_is() {
    let mut game = setup_two_player_game();
    let vigor = put_on_battlefield(&mut game, shuffles_in_from_anywhere(), 0);

    game.execute_action(GameAction::Destroy { object: vigor }, &test_ctx()).unwrap();
    resolve_everything(&mut game);

    assert_eq!(game.get_object(vigor).unwrap().zone, Zone::Library);
}

// ---------------------------------------------------------------------------
// "Them" (CR 603.2c)
// ---------------------------------------------------------------------------

/// "Whenever one or more cards are put into your graveyard from your library,
/// return them to your hand", shaped like Hedge Shredder's "put them onto the
/// battlefield".
fn returns_milled_cards() -> Arc<CardData> {
    creature_with_ability(
        "Fixture: returns them",
        1,
        1,
        triggered_ability(
            "",
            whenever(
                TriggerEvent::ZoneChange {
                    subject: TriggerSubject::Any,
                    change: ZoneChangePattern { from: Some(Zone::Library), to: Some(Zone::Graveyard), act: None },
                    by: None,
                    owner: Some(PlayerRef::You),
                    multiplicity: Multiplicity::OncePerEvent,
                },
                Effect::Atom(Primitive::ReturnToHand, EffectRecipient::TriggeringObjects),
            ),
        ),
    )
}

/// "Them" is each card its own record says the mill made, found while it is
/// still that card: one exiled from the graveyard before the trigger resolves
/// is a new object, and stays in exile.
#[test]
fn them_finds_each_card_by_its_own_record() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, returns_milled_cards(), 0);
    let first = put_in_library(&mut game, vanilla_creature(1, 1, &[]), 0);
    let second = put_in_library(&mut game, vanilla_creature(1, 1, &[]), 0);
    let third = put_in_library(&mut game, vanilla_creature(1, 1, &[]), 0);

    let milled: Vec<ObjectId> = vec![first, second, third];
    game.execute_actions(
        milled
            .iter()
            .map(|&object| GameAction::ZoneChange {
                object,
                from: Zone::Library,
                to: Zone::Graveyard,
                act: mtgsim::engine::actions::ZoneChangeAct::Milled,
            })
            .collect(),
        &test_ctx(),
    )
    .unwrap();
    assert_eq!(game.pending_triggers.len(), 1, "CR 603.2c: one trigger for the one event");

    place(&mut game, &test_dp());
    game.change_zone(second, Zone::Exile, mtgsim::engine::actions::ZoneChangeAct::Put, &test_ctx()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(game.get_object(first).unwrap().zone, Zone::Hand);
    assert_eq!(game.get_object(third).unwrap().zone, Zone::Hand);
    assert_eq!(game.get_object(second).unwrap().zone, Zone::Exile, "a new object since the mill (CR 400.7)");
}
