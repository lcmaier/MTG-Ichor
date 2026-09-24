//! `codebase-state.md` item 176: an "instead" that changes only the
//! destination keeps the act.
//!
//! Destroy, discard, mill, sacrifice and counter are each a move to the
//! graveyard (CR 701.8a, 701.9a, 701.17a, 701.21a, 701.6a), and CR 614.6
//! makes the modified event the one that happens. So a redirect changes the
//! record's `to` and leaves its `cause`, which is the act. CR 701.9c and the
//! Rest in Peace, Leyline of the Void and Nephalia Academy rulings say so for
//! discard, CR 701.17c for mill, and CR 608.2c's example for counter.
//!
//! Every board here is one registered replacement and one real act, and each
//! test asserts the performed record, which is what a trigger or a "this way"
//! clause will read.

use std::sync::Arc;

use mtgsim::cards::alpha::counterspell;
use mtgsim::cards::phase_rb_cards::{leyline_of_the_void, rest_in_peace};
use mtgsim::cards::phase_re8_cards::{mind_rot, nephalia_academy};
use mtgsim::cards::phase_rs_cards::diabolic_edict;
use mtgsim::engine::actions::ZoneChangeCause;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
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
    AmountExpr, Effect, EffectRecipient, ObjectFilter, Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
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
fn zone_changes(game: &GameState) -> Vec<(ObjectId, Zone, Zone, ZoneChangeCause)> {
    game.events
        .events()
        .filter_map(|e| match e {
            GameEvent::ZoneChange { object_id, from, to, cause, .. } => {
                Some((*object_id, *from, *to, *cause))
            }
            _ => None,
        })
        .collect()
}

/// The one zone change `object` made.
fn the_move_of(game: &GameState, object: ObjectId) -> (Zone, Zone, ZoneChangeCause) {
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
        targets: ChosenTargets::one(vec![ResolvedTarget::Player(target_player)]),
        replaced_amount: None,
        damage_prevented: None,
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
    // A one-card hand, so Mind Rot's choice is forced (CR 102.2).
    resolve_at_player(&mut game, mind_rot(), 0, 1, &dp);

    assert!(dp.is_empty(), "Academy was offered after Leyline applied");
    assert_eq!(game.players[1].library.last(), Some(&card), "on top of the library");
    assert_eq!(the_move_of(&game, card), (Zone::Hand, Zone::Library, ZoneChangeCause::Discarded));
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
    assert_eq!(the_move_of(&game, card), (Zone::Hand, Zone::Exile, ZoneChangeCause::Discarded));
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
        (Zone::Battlefield, Zone::Exile, ZoneChangeCause::Sacrificed)
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
            (rip, Zone::Battlefield, Zone::Exile, ZoneChangeCause::Destroyed),
            (breaker, Zone::Stack, Zone::Graveyard, ZoneChangeCause::Resolved),
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
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 1,
        targets: ChosenTargets::NONE,
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    let mill_two = Effect::Atom(Primitive::Mill(AmountExpr::Fixed(2)), EffectRecipient::Controller);
    game.resolve_effect(&mill_two, &ctx, &test_dp()).expect("resolving");

    for card in top_two {
        assert_eq!(the_move_of(&game, card), (Zone::Library, Zone::Exile, ZoneChangeCause::Milled));
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

    assert_eq!(the_move_of(&game, bears), (Zone::Stack, Zone::Exile, ZoneChangeCause::Countered));
    let resolved: Vec<_> = zone_changes(&game)
        .into_iter()
        .filter(|(id, from, ..)| *id == counter && *from == Zone::Stack)
        .collect();
    assert_eq!(resolved, vec![(counter, Zone::Stack, Zone::Exile, ZoneChangeCause::Resolved)]);
}
