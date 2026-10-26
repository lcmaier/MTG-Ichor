//! `codebase-state.md` item 176: an "instead" that changes only the
//! destination keeps the act.
//!
//! Destroy, discard, mill, sacrifice and counter are each a move to the
//! graveyard (CR 701.8a, 701.9a, 701.17a, 701.21a, 701.6a), and CR 614.6
//! makes the modified event the one that happens. So a redirect changes the
//! record's `to` and leaves its act and its cause. CR 701.9c and the Rest in
//! Peace, Leyline of the Void and Nephalia Academy rulings say so for discard,
//! CR 701.17c for mill, and CR 608.2c's example for counter. Every act a
//! replacement can see has a board here: the five above, a rule's `Put`, a
//! draw and a land play. A cast cannot be redirected — CR 601.2a's move is no
//! proposal, and CR 601.3 asks "can't cast" first.
//!
//! Every board here is one registered replacement and one real act, and each
//! test asserts the performed record, which is what a trigger or a "this way"
//! clause will read.

use std::sync::Arc;

use mtgsim::cards::alpha::counterspell;
use mtgsim::cards::phase_rb_cards::{leyline_of_the_void, rest_in_peace};
use mtgsim::cards::phase_rc_cards::{containment_priest, dryad_arbor};
use mtgsim::cards::phase_re8_cards::{mind_rot, nephalia_academy};
use mtgsim::cards::phase_rs_cards::diabolic_edict;
use mtgsim::engine::actions::ZoneChangeAct;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::{EventCause, GeneratedBy, SpecialAction, StateBasedAction};
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{
    AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    fill_library, place_vanilla_creature, put_in_graveyard, put_in_hand, put_on_battlefield,
    put_spell_on_stack, setup_two_player_game, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerRef, Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use mtgsim::types::zones::{Zone, ZoneChangePattern, ZoneSet};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::DecisionProvider;

/// CR 616.1's choice between two applicable effects.
const PICK_REPLACEMENT: ChoiceKind = ChoiceKind::ChooseReplacementEffect { affected_object: None };
/// CR 616.1's "you may … instead" prompt.
const APPLY_OPTIONAL: ChoiceKind = ChoiceKind::ApplyOptionalReplacement {
    affected_object: None,
    source: ObjectId::UNASSIGNED,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Every zone change in the log, as `(object, from, to, cause)`.
fn zone_changes(game: &GameState) -> Vec<(ObjectId, Zone, Zone, ZoneChangeAct)> {
    game.recorded_events()
        .events()
        .filter_map(|e| match e {
            GameEvent::ZoneChange { departed, from, to, act, .. } => {
                Some((departed.object().id, *from, *to, *act))
            }
            _ => None,
        })
        .collect()
}

/// `object`'s last zone change, with the cause its record names.
fn the_record_of(game: &GameState, object: ObjectId) -> (Zone, Zone, ZoneChangeAct, EventCause) {
    let recorded = game.recorded_events();
    recorded
        .records()
        .iter()
        .rev()
        .find_map(|r| match r.event {
            GameEvent::ZoneChange { ref departed, from, to, act, .. } if departed.object().id == object => {
                Some((from, to, act, r.stamp.cause))
            }
            _ => None,
        })
        .expect("it moved")
}

/// The one zone change `object` made.
fn the_move_of(game: &GameState, object: ObjectId) -> (Zone, Zone, ZoneChangeAct) {
    let moves: Vec<_> = zone_changes(game)
        .into_iter()
        .filter(|(id, ..)| *id == object)
        .map(|(_, from, to, cause)| (from, to, cause))
        .collect();
    assert_eq!(moves.len(), 1, "one move for {object:?}, got {moves:?}");
    moves[0]
}

/// Resolve `card`'s one spell ability for `caster` against `target_player`.
/// The card sits in its caster's graveyard as the resolution's source, out of
/// every hand and library a board here counts.
fn resolve_at_player(
    game: &mut GameState,
    card: Arc<CardData>,
    caster: PlayerId,
    target_player: PlayerId,
    dp: &dyn DecisionProvider,
) {
    let source = put_in_graveyard(game, card.clone(), caster);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: caster,
        generated_by: mtgsim::events::cause::GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ObjectOrPlayerId::Player(target_player)]),
        replaced_event: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, dp).expect("resolving");
}

/// **Fixture.** A free instant, "Destroy target enchantment." It wears no
/// printed card's name, and nothing registered destroys an enchantment.
fn fixture_enchantment_breaker() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Enchantment Breaker")
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            rules_text: "".into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: new_ability_id(),
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::Destroy,
                EffectRecipient::Target(
                    SelectionFilter::Permanent(ObjectFilter::ByType(CardType::Enchantment)),
                    TargetCount::Exactly(1),
                ),
            ),
        })
        .build()
}

// ---------------------------------------------------------------------------
// Discard: CR 616.1f's re-gather reads the act
// ---------------------------------------------------------------------------

/// The item's board. P1 picks Leyline first, and the card is still being
/// discarded, so Academy ("instead of putting it anywhere else") still
/// applies on the re-gather (CR 616.1f) and is offered.
#[test]
fn leyline_first_still_offers_academy_because_the_card_was_still_discarded() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, leyline_of_the_void(), 0);
    put_on_battlefield(&mut game, nephalia_academy(), 1);
    fill_library(&mut game, 1, 3);
    let card = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 1);

    let dp = test_dp();
    // Leyline entered first, so it is the first candidate.
    dp.expect_pick_n(PICK_REPLACEMENT, vec![0]);
    dp.expect_pick_n(APPLY_OPTIONAL, vec![0]);
    // A one-card hand, so Mind Rot's choice is forced.
    resolve_at_player(&mut game, mind_rot(), 0, 1, &dp);

    assert!(dp.is_empty(), "Academy was offered after Leyline applied");
    assert_eq!(game.players[1].library.last(), Some(&card), "on top of the library");
    assert_eq!(the_move_of(&game, card), (Zone::Hand, Zone::Library, ZoneChangeAct::Discarded));
}

/// The other order. Academy is offered first and declined, which spends its
/// one opportunity (CR 614.5), so Leyline exiles the card. It was still
/// discarded, and the record says so.
#[test]
fn academy_declined_first_leaves_leyline_to_exile_a_card_that_was_still_discarded() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, leyline_of_the_void(), 0);
    put_on_battlefield(&mut game, nephalia_academy(), 1);
    fill_library(&mut game, 1, 3);
    let card = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 1);

    let dp = test_dp();
    dp.expect_pick_n(PICK_REPLACEMENT, vec![1]);
    dp.expect_pick_n(APPLY_OPTIONAL, vec![]);
    resolve_at_player(&mut game, mind_rot(), 0, 1, &dp);

    assert!(dp.is_empty());
    assert!(game.exile.contains(&card));
    assert_eq!(the_move_of(&game, card), (Zone::Hand, Zone::Exile, ZoneChangeAct::Discarded));
}

// ---------------------------------------------------------------------------
// Sacrifice, destroy, mill and counter under Rest in Peace
// ---------------------------------------------------------------------------

/// A one-creature board, so the edict's choice is forced.
#[test]
fn a_creature_sacrificed_under_rest_in_peace_is_recorded_as_sacrificed() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, rest_in_peace(), 0);
    let victim = place_vanilla_creature(&mut game, 1, 2, 2, &[]);

    resolve_at_player(&mut game, diabolic_edict(), 0, 1, &test_dp());

    assert_eq!(
        the_move_of(&game, victim),
        (Zone::Battlefield, Zone::Exile, ZoneChangeAct::Sacrificed)
    );
}

/// Rest in Peace's own ruling (2018-03-16): destroyed by a spell, it is
/// exiled, and then the spell reaches its owner's graveyard. Its ability
/// applies to its own destruction, since it is on the battlefield when the
/// event is proposed, and it is gone by CR 608.2n's move.
// COVERS-PARTIAL: ATOM-614.6-001
#[test]
fn rest_in_peace_destroyed_by_a_spell_is_exiled_as_destroyed_and_the_spell_reaches_the_graveyard() {
    let mut game = setup_two_player_game();
    let rip = put_on_battlefield(&mut game, rest_in_peace(), 0);
    let breaker = put_in_hand(&mut game, fixture_enchantment_breaker(), 1);

    // Free, and Rest in Peace is the only enchantment, so nothing is asked.
    game.cast_spell(1, breaker, &test_dp()).expect("castable");
    game.resolve_top_of_stack(&test_dp()).expect("resolves");

    let after_cast: Vec<_> = zone_changes(&game)
        .into_iter()
        .filter(|(_, from, ..)| *from != Zone::Hand)
        .collect();
    assert_eq!(
        after_cast,
        vec![
            (rip, Zone::Battlefield, Zone::Exile, ZoneChangeAct::Destroyed),
            (breaker, Zone::Stack, Zone::Graveyard, ZoneChangeAct::Put),
        ]
    );
}

#[test]
fn cards_milled_under_rest_in_peace_are_recorded_as_milled() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, rest_in_peace(), 0);
    fill_library(&mut game, 1, 5);
    let top_two: Vec<ObjectId> = game.players[1].library.iter().rev().take(2).copied().collect();

    let source = put_in_graveyard(&mut game, vanilla_creature(1, 1, &[]), 1);
    let ctx = ResolutionContext::untargeted(source, 1, mtgsim::events::cause::GeneratedBy::Spell);
    let mill_two = Effect::Atom(Primitive::Mill(AmountExpr::Fixed(2)), EffectRecipient::Player(PlayerRef::You));
    game.resolve_effect(&mill_two, &ctx, &test_dp()).expect("resolving");

    for card in top_two {
        assert_eq!(the_move_of(&game, card), (Zone::Library, Zone::Exile, ZoneChangeAct::Milled));
    }
}

/// Cast from a hand, so the whole of CR 601.2 runs once in this file. The
/// countered spell is exiled and still countered (CR 608.2c's example, which
/// CR 603.10e's "when a spell is countered" watches). Counterspell itself
/// resolved, and CR 608.2n's move for it is redirected too.
#[test]
fn a_spell_countered_under_rest_in_peace_is_recorded_as_countered() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, rest_in_peace(), 1);
    let bears = put_spell_on_stack(&mut game, vanilla_creature(2, 2, &[]), 1);
    let counter = put_in_hand(&mut game, counterspell(), 0);
    game.players[0].mana_pool.add(ManaType::Blue, 2);

    // One spell on the stack, so the target is forced.
    game.cast_spell(0, counter, &test_dp()).expect("{U}{U} is in the pool");
    game.resolve_top_of_stack(&test_dp()).expect("resolves");

    assert_eq!(the_move_of(&game, bears), (Zone::Stack, Zone::Exile, ZoneChangeAct::Countered));
    let resolved: Vec<_> = zone_changes(&game)
        .into_iter()
        .filter(|(id, from, ..)| *id == counter && *from == Zone::Stack)
        .collect();
    assert_eq!(resolved, vec![(counter, Zone::Stack, Zone::Exile, ZoneChangeAct::Put)]);
}

// ---------------------------------------------------------------------------
// A rule's move, a draw and a land play keep their act and their cause
// ---------------------------------------------------------------------------

/// CR 704.5f puts a 0-toughness creature into its owner's graveyard, an act
/// of `Put`; under Leyline it is exiled, still `Put`, still the rule's.
#[test]
fn a_creature_with_zero_toughness_under_leyline_is_put_by_the_rule() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, leyline_of_the_void(), 0);
    let creature = put_on_battlefield(&mut game, vanilla_creature(0, 0, &[]), 1);

    game.check_state_based_actions(&test_dp()).expect("the check");

    assert_eq!(
        the_record_of(&game, creature),
        (Zone::Battlefield, Zone::Exile, ZoneChangeAct::Put, EventCause::StateBasedAction(StateBasedAction::ZeroToughness))
    );
}

/// CR 704.5g destroys a creature with lethal damage, which CR 701.8b calls
/// destroyed; under Leyline it is exiled, still destroyed, still by the
/// state-based action.
#[test]
fn a_creature_with_lethal_damage_under_leyline_is_destroyed_by_the_rule() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, leyline_of_the_void(), 0);
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    game.battlefield.get_mut(&creature).unwrap().damage_marked = 2;

    game.check_state_based_actions(&test_dp()).expect("the check");

    assert_eq!(
        the_record_of(&game, creature),
        (Zone::Battlefield, Zone::Exile, ZoneChangeAct::Destroyed, EventCause::StateBasedAction(StateBasedAction::LethalDamage))
    );
}

/// **Fixture.** "If a card would be put into an opponent's hand from their
/// library, exile it instead." No printed card redirects a draw's move; the
/// CR is the customer (`engineering-practices.md#gaps-a-phase-finds`).
fn fixture_draw_exiler() -> Arc<CardData> {
    let def = ReplacementDef::new(
        EventPattern::ZoneChange {
            change: ZoneChangePattern { from: Some(Zone::Library), to: Some(Zone::Hand), act: None },
            object: None,
        },
        ObjectSet::filter_in(ObjectFilter::ByOwner(PlayerRef::Opponent), ZoneSet::LIBRARY),
        Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
    );
    CardDataBuilder::new("Fixture Draw Exiler")
        .card_type(CardType::Enchantment)
        .ability(AbilityDef {
            rules_text: "".into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: new_ability_id(),
            instances: Vec::new(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(def)),
        })
        .build()
}

/// A drawn card redirected to exile was still drawn (CR 121.5's move), and
/// the draw is still its spell's.
#[test]
fn a_card_drawn_into_exile_is_recorded_as_drawn() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_draw_exiler(), 0);
    fill_library(&mut game, 1, 3);
    let top = *game.players[1].library.last().unwrap();

    let source = put_in_graveyard(&mut game, vanilla_creature(1, 1, &[]), 1);
    let ctx = ResolutionContext::untargeted(source, 1, GeneratedBy::Spell);
    let draw = Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You));
    game.resolve_effect(&draw, &ctx, &test_dp()).expect("resolving");

    let (from, to, act, cause) = the_record_of(&game, top);
    assert_eq!((from, to, act), (Zone::Library, Zone::Exile, ZoneChangeAct::Drawn));
    assert!(matches!(cause, EventCause::Effect { resolution, .. } if resolution.source == source), "{cause:?}");
}

/// Containment Priest exiles a land creature played from a hand, which was
/// never cast (item 176's Dryad Arbor). It was still played as a land, by the
/// special action.
#[test]
fn a_dryad_arbor_containment_priest_exiles_is_recorded_as_played() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, containment_priest(), 1);
    let arbor = put_in_hand(&mut game, dryad_arbor(), 0);

    game.play_land(0, arbor, &test_dp()).expect("the land drop is legal");

    assert_eq!(
        the_record_of(&game, arbor),
        (
            Zone::Hand,
            Zone::Exile,
            ZoneChangeAct::PlayedAsLand,
            EventCause::SpecialAction { player: 0, action: SpecialAction::PlayLand }
        )
    );
}
