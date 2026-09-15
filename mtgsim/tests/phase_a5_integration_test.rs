//! Phase A5 integration tests: CR 113.6, which abilities function in which
//! zone (`layers-architecture.md` §13d).
//!
//! The headline is Wonder, and it takes **three** tests rather than one,
//! because CR 113.6b is two claims and a default: the ability functions from
//! the graveyard (new), it does *not* function from the battlefield (the
//! "only" in "only from those zones"), and a card that states no zone keeps
//! the default (ATOM-113.6-001, which is the negative this phase must not
//! break).
//!
//! Two of the tests here are about something else and are here anyway. The
//! Yixlid Jailer board is LJ × A5 — one card from each phase, each reaching
//! the zone the other is in — and the Giant Growth one is a regression for
//! `cleanup_zone_state`'s new branch, which would have deleted every pump
//! spell in the game had it been written as the broad sweep beside it.

use mtgsim::cards::basic_lands;
use mtgsim::cards::creatures;
use mtgsim::cards::phase_a5_cards::{exiled_ancestor, wonder};
use mtgsim::cards::phase_le_cards::tarmogoyf;
use mtgsim::cards::phase_lj_cards::yixlid_jailer;
use mtgsim::engine::actions::ZoneChangeCause;
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_power, has_keyword,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_graveyard, put_on_battlefield, setup_two_player_game, static_ability, test_ctx,
    vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive,
};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::zones::Zone;

/// A vanilla 1/1 for player 0 — what Wonder's grant is asserted against.
fn a_creature(game: &mut GameState) -> ObjectId {
    put_on_battlefield(game, vanilla_creature(1, 1, &[]), 0)
}

// ---------------------------------------------------------------------------
// CR 113.6b — an ability that states its zone functions only from there
// ---------------------------------------------------------------------------

/// The facility, end to end: a static ability whose **source** is in a
/// graveyard changes a permanent on the battlefield.
///
/// Nothing before A5 could express this. LJ made a filter row *reach* another
/// zone, which is the affected side; here the affected set is an ordinary
/// "creatures you control" on the battlefield and it is the **source** that is
/// somewhere else. `register_static_effects` ran from exactly one call site
/// before this phase, so a card that never entered the battlefield registered
/// nothing at all.
#[test]
fn test_a_static_ability_functions_from_a_graveyard() {
    let mut game = setup_two_player_game();
    let bear = a_creature(&mut game);
    put_on_battlefield(&mut game, basic_lands::island(), 0);

    assert!(!has_keyword(&game, bear, KeywordFlag::Flying), "no Wonder yet");

    put_in_graveyard(&mut game, wonder(), 0);
    assert!(
        has_keyword(&game, bear, KeywordFlag::Flying),
        "CR 113.6b — Wonder's ability functions from the graveyard, and the grant \
         lands on a creature on the battlefield"
    );
}

/// The "only" in "only from those zones", which is what makes CR 113.6b a rule
/// rather than a permission.
///
/// A Wonder **on the battlefield** is a 2/2 flier that grants nothing — its
/// ability states the graveyard, so on the battlefield it does not function.
/// Before A5 the engine had it exactly backwards: registration happened at ETB
/// and nowhere else, so this was the only board on which Wonder did anything.
#[test]
fn test_the_same_ability_does_not_function_from_the_battlefield() {
    let mut game = setup_two_player_game();
    let bear = a_creature(&mut game);
    put_on_battlefield(&mut game, basic_lands::island(), 0);

    let wonder_id = put_on_battlefield(&mut game, wonder(), 0);
    assert!(
        has_keyword(&game, wonder_id, KeywordFlag::Flying),
        "its own printed flying is unaffected"
    );
    assert!(
        !has_keyword(&game, bear, KeywordFlag::Flying),
        "CR 113.6b — on the battlefield the granting ability does not function"
    );
}

/// The zone clause and the "as long as" clause are different sentences, and
/// only one of them places the ability.
///
/// With Wonder in the graveyard and no Island, the ability *functions* — the
/// row is registered — and its condition is false, so nothing is granted.
/// Playing an Island turns it on with no zone change at all, which is the
/// distinction `Condition::All` is carrying.
#[test]
fn test_the_island_clause_is_an_ordinary_condition_not_a_zone_statement() {
    let mut game = setup_two_player_game();
    let bear = a_creature(&mut game);
    put_in_graveyard(&mut game, wonder(), 0);

    assert!(!has_keyword(&game, bear, KeywordFlag::Flying), "no Island");

    put_on_battlefield(&mut game, basic_lands::island(), 0);
    assert!(has_keyword(&game, bear, KeywordFlag::Flying), "an Island");

    // A Forest is not an Island, and "you control" is CR 109.5's controller of
    // the source — which off the battlefield is its owner (CR 108.4). An
    // opponent's Island does not turn it on.
    let mut game = setup_two_player_game();
    let bear = a_creature(&mut game);
    put_in_graveyard(&mut game, wonder(), 0);
    put_on_battlefield(&mut game, basic_lands::island(), 1);
    assert!(
        !has_keyword(&game, bear, KeywordFlag::Flying),
        "CR 109.5 — 'you' is the source's own side, and this Island is not theirs"
    );
}

/// Leaving the named zone retires the effect, and **not because anything
/// reconciled the registry**.
///
/// This is §13d decision 3's whole argument in one assertion. The zone clause
/// is a `Condition::SourceInZone`, so CR 604.2's existence check — which runs
/// at every layer and already evaluates conditions — is what turns the grant
/// off. Exiling Wonder does remove its row as hygiene, but the answer would be
/// the same without that, which is what "registry membership is not effect
/// existence" buys.
#[test]
fn test_leaving_the_named_zone_retires_the_effect() {
    let mut game = setup_two_player_game();
    let bear = a_creature(&mut game);
    put_on_battlefield(&mut game, basic_lands::island(), 0);
    let wonder_id = put_in_graveyard(&mut game, wonder(), 0);
    assert!(has_keyword(&game, bear, KeywordFlag::Flying));

    game.change_zone(wonder_id, Zone::Exile, ZoneChangeCause::Exiled, &test_ctx())
        .expect("it is exiled");
    assert!(
        !has_keyword(&game, bear, KeywordFlag::Flying),
        "CR 113.6b — in exile the ability does not function"
    );
}

// ---------------------------------------------------------------------------
// CR 113.6 — the default, and the negative that must not break
// ---------------------------------------------------------------------------

/// ATOM-113.6-001 — a creature with "creatures you control get +1/+1" in a
/// graveyard does **not** apply it.
///
/// The card states no zone, so CR 113.6's first sentence answers: an ability
/// of a permanent card functions only on the battlefield. This passed before
/// A5 for a reason that no longer holds (registration ran at ETB and nowhere
/// else), and it has to keep passing now that `move_object` registers too —
/// which is exactly what the zone gate in `register_static_effects` is for.
#[test]
fn test_an_anthem_in_a_graveyard_does_not_apply_it() {
    let anthem = mtgsim::test_support::creature_with_ability(
        "Graveyard Anthem",
        2,
        2,
        static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(
                AmountExpr::Fixed(1),
                AmountExpr::Fixed(1),
                Duration::WhileSourceOnBattlefield,
            ),
            EffectRecipient::FilteredPermanents(ObjectFilter::And(
                Box::new(ObjectFilter::ByType(CardType::Creature)),
                Box::new(ObjectFilter::ByController(PlayerRef::You)),
            )),
        )),
    );

    let mut game = setup_two_player_game();
    let bear = a_creature(&mut game);
    put_in_graveyard(&mut game, anthem.clone(), 0);
    assert_eq!(
        get_effective_power(&game, bear),
        Some(1),
        "CR 113.6 — the ability names no zone, so it functions only on the battlefield"
    );

    // And the control: the same card on the battlefield does apply it, so the
    // assertion above is about the zone and not about the card being inert.
    put_on_battlefield(&mut game, anthem, 0);
    assert_eq!(get_effective_power(&game, bear), Some(2));
}

/// ATOM-113.6a-001 — a Tarmogoyf in a graveyard still has a computed P/T.
///
/// **An assertion, not a mechanism.** CR 113.6a's "characteristic-defining
/// abilities function everywhere" has been true since the CDA phase, through a
/// path that consults no predicate: `layers::cda` applies a CDA off the
/// object's own effective ability list and `compute_non_member` walks it in
/// any zone (CR 604.3a(3), and `CLAUDE.md`'s "CDAs are never registry
/// effects"). This is here so that A5's new zone gate is shown not to have
/// broken it — the gate is in `register_static_effects`, which skips CDAs one
/// line further down and always did.
#[test]
fn test_a_cda_still_functions_in_a_graveyard() {
    let mut game = setup_two_player_game();
    let goyf = put_in_graveyard(&mut game, tarmogoyf(), 0);
    // Its own card type is in a graveyard, so CR 208.2a's count is at least 1.
    assert_eq!(
        get_effective_power(&game, goyf),
        Some(1),
        "CR 113.6a — the CDA functions in the graveyard and counts its own type"
    );
}

/// CR 113.6c — "an ability that states which zones it *doesn't* function in
/// functions everywhere except for the specified zones".
///
/// The fixture exists to show this needs no second mechanism: it is the same
/// `SourceInZone` clause holding the complement `ZoneSet` already spells.
#[test]
fn test_a_stated_complement_functions_everywhere_else() {
    let mut game = setup_two_player_game();
    let bear = a_creature(&mut game);

    let ancestor = put_in_graveyard(&mut game, exiled_ancestor(), 0);
    assert_eq!(get_effective_power(&game, bear), Some(2), "in a graveyard, which is not the battlefield");

    game.change_zone(ancestor, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx())
        .expect("it enters");
    assert_eq!(
        get_effective_power(&game, bear),
        Some(1),
        "CR 113.6c — on the battlefield, which is the zone it excludes"
    );
}

// ---------------------------------------------------------------------------
// LJ × A5 — each phase's card reaching the zone the other's is in
// ---------------------------------------------------------------------------

/// Yixlid Jailer turns Wonder off, and every step of that is a rule.
///
/// The Jailer is LJ's card: a battlefield source whose row reaches graveyards
/// ("cards in graveyards lose all abilities"). Wonder is A5's: a graveyard
/// source whose row reaches the battlefield. With both out, the Jailer's
/// Layer 6 strip lands on Wonder's frame, CR 604.2's existence check finds no
/// such ability there, and Wonder's row stops applying — with nothing removed
/// from the registry and nothing reconciled at a chokepoint.
#[test]
fn test_yixlid_jailer_turns_wonder_off_through_the_existence_check() {
    let mut game = setup_two_player_game();
    let bear = a_creature(&mut game);
    put_on_battlefield(&mut game, basic_lands::island(), 0);
    let wonder_id = put_in_graveyard(&mut game, wonder(), 0);
    assert!(has_keyword(&game, bear, KeywordFlag::Flying));

    let jailer = put_on_battlefield(&mut game, yixlid_jailer(), 1);
    assert!(
        get_effective_abilities(&game, wonder_id).is_empty(),
        "LJ — the Jailer reaches into the graveyard and strips Wonder's ability"
    );
    assert!(
        !has_keyword(&game, bear, KeywordFlag::Flying),
        "CR 604.2 — an ability the source no longer has generates no effect"
    );

    // And back: the row was never removed, so the answer returns with the
    // ability rather than needing re-registration.
    game.change_zone(jailer, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx())
        .expect("it dies");
    assert!(
        has_keyword(&game, bear, KeywordFlag::Flying),
        "the Jailer is gone, so Wonder's ability is back on its frame"
    );
}

// ---------------------------------------------------------------------------
// The regression `cleanup_zone_state`'s new branch could have caused
// ---------------------------------------------------------------------------

/// A pump spell's effect survives the spell reaching the graveyard.
///
/// `cleanup_zone_state` gained a branch for every zone but the battlefield,
/// and the obvious way to write it — `remove_by_source`, the call the
/// battlefield branch makes — would have been catastrophic and silent. A
/// resolving instant registers its continuous effect with `source` = the spell
/// object, and the spell moves stack → graveyard the moment it finishes
/// resolving (CR 608.2n), so a broad sweep there deletes the effect on the
/// next statement. The branch retires static-ability rows only.
#[test]
fn test_a_resolutions_effect_survives_its_spell_reaching_the_graveyard() {
    let mut game = setup_two_player_game();
    let bear = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let spell = put_in_graveyard(&mut game, mtgsim::test_support::lightning_bolt(), 0);

    // A resolution's row, registered the way `resolve` registers one: sourced
    // at the spell, `EffectOrigin::Resolution`, lasting the turn.
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(mtgsim::engine::layers::types::ContinuousEffect {
        id: 0,
        source: spell,
        origin: mtgsim::engine::layers::types::EffectOrigin::Resolution,
        layer: mtgsim::engine::layers::types::Layer::Layer7cModifyPT,
        duration: Duration::UntilEndOfTurn,
        controller: 0,
        created_on_turn: game.turn_number,
        timestamp,
        affected_objects: mtgsim::types::effects::ObjectSet::Fixed(vec![bear]),
        modification: mtgsim::engine::layers::types::EffectModification::ModifyPowerToughness {
            power: mtgsim::engine::layers::types::PtValue::Fixed(3),
            toughness: mtgsim::engine::layers::types::PtValue::Fixed(3),
        },
    });
    assert_eq!(get_effective_power(&game, bear), Some(5));

    // The spell moves on — exile, a graveyard-hate effect, anything. The row
    // is a resolution's and CR 611.2a gives it the duration the spell stated,
    // not the source's lifetime.
    game.change_zone(spell, Zone::Exile, ZoneChangeCause::Exiled, &test_ctx())
        .expect("it is exiled");
    assert_eq!(
        get_effective_power(&game, bear),
        Some(5),
        "CR 611.2a — a resolution's effect does not care where its source went"
    );
}

// ---------------------------------------------------------------------------
// CR 613.7d — the object timestamp
// ---------------------------------------------------------------------------

/// Every zone entry stamps the object, and the stamps keep the order the
/// battlefield sweeps depend on.
///
/// The field moved off `PermanentState` for Wonder's sake — CR 613.7a wants
/// "the same timestamp as the object the static ability is on" and a graveyard
/// card had none — and the thing to check is that the battlefield's own use of
/// it is unchanged: monotonic, unique, and reassigned by CR 613.7e.
#[test]
fn test_an_object_is_stamped_on_every_zone_entry() {
    let mut game = setup_two_player_game();
    let card = mtgsim::test_support::put_in_hand(&mut game, creatures::grizzly_bears(), 0);
    let in_hand = game.object_timestamp(card);

    game.change_zone(card, Zone::Graveyard, ZoneChangeCause::Discarded, &test_ctx())
        .expect("it is discarded");
    let in_graveyard = game.object_timestamp(card);
    assert!(
        in_graveyard > in_hand,
        "CR 613.7d — a new zone, a new timestamp ({in_hand} then {in_graveyard})"
    );

    // And the battlefield order is still the one `battlefield_ids_ordered`
    // promises: oldest first, by the same monotonic counter.
    let first = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let second = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let order = game.battlefield_ids_ordered();
    let pos = |id| order.iter().position(|&o| o == id).expect("on the battlefield");
    assert!(pos(first) < pos(second));
    assert!(game.object_timestamp(first) < game.object_timestamp(second));
}
