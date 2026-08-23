//! Layer 6 — ability adding and removing (CR 613.1f).
//!
//! Two channels, because `EffectiveCharacteristics` has two fields:
//! `GrantKeyword`/`RemoveKeyword` write `keywords`, `GrantAbility`/
//! `LoseAbility` write `abilities`, and `LoseAllAbilities` clears both. See
//! `KeywordFlag`'s docs for which CR 702 keywords belong on which channel.
//!
//! The rows here are built with `test_support::registered`, which produces an
//! `EffectOrigin::Resolution` / `Duration::UntilEndOfTurn` row over a fixed
//! target set. That is not a stand-in for anything — it is exactly what "target
//! creature loses flying until end of turn" creates (CR 613.7b). What was
//! missing before this phase was not a card; it was the `EffectModification`
//! variant, so no card *could* produce one.

use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, Layer, Timestamp};
use mtgsim::objects::card_data::AbilityDef;
use mtgsim::oracle::characteristics::{get_effective_abilities, has_keyword};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    creature_with_ability, put_on_battlefield, registered, setup_two_player_game, static_ability,
    vanilla_creature,
};
use mtgsim::types::effects::{Duration, Effect, EffectRecipient, Primitive};
use mtgsim::types::ids::{AbilityId, ObjectId};
use mtgsim::types::keywords::KeywordFlag;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn row(id: ObjectId, timestamp: Timestamp, modification: EffectModification) -> ContinuousEffect {
    registered(id, Layer::Layer6Ability, timestamp, modification)
}

/// A static ability with no effect this file ever observes. Layer 6 tests care
/// about an ability's *identity*, not what it does.
fn inert_ability() -> AbilityDef {
    static_ability(Effect::Atom(
        Primitive::GrantKeyword(KeywordFlag::Vigilance, Duration::WhileSourceOnBattlefield),
        EffectRecipient::Implicit,
    ))
}

fn ability_ids(game: &GameState, id: ObjectId) -> Vec<AbilityId> {
    get_effective_abilities(game, id).iter().map(|a| a.id).collect()
}

// ---------------------------------------------------------------------------
// CR 113.10b — removing an ability removes ALL instances of it
// ---------------------------------------------------------------------------

/// The substantive half of the rule. `chars.abilities` is a `Vec`, and the same
/// ability really can appear twice — printed on the card and granted on top of
/// it, sharing an `AbilityId`. A "remove the first match" implementation passes
/// every other test in this file and fails this one.
// COVERS: ATOM-113.10b-001
#[test]
fn test_losing_an_ability_removes_every_instance_of_it() {
    let mut game = setup_two_player_game();
    let printed = inert_ability();
    let id = put_on_battlefield(
        &mut game,
        creature_with_ability("Twice-Sworn", 2, 2, printed.clone()),
        0,
    );

    // Grant the *same* ability again — same id, so the creature now has two
    // instances of one ability, which is the board CR 113.10b describes.
    game.continuous_effects.add(row(
        id,
        1,
        EffectModification::GrantAbility(Box::new(printed.clone())),
    ));
    assert_eq!(
        ability_ids(&game, id),
        vec![printed.id, printed.id],
        "printed + granted = two instances of the same ability"
    );

    // One "loses [ability]" effect, and both go.
    game.continuous_effects
        .add(row(id, 2, EffectModification::LoseAbility(printed.id)));
    assert!(
        get_effective_abilities(&game, id).is_empty(),
        "CR 113.10b: removing an ability removes all instances of it, not the first"
    );
}

/// The keyword channel reaches the same answer structurally: `keywords` is a
/// `HashSet`, so a printed flying and a granted flying were never two entries
/// to begin with, and one `RemoveKeyword` clears it.
// COVERS-PARTIAL: ATOM-113.10b-001
#[test]
fn test_removing_a_keyword_clears_it_however_many_effects_granted_it() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Flying]), 0);

    game.continuous_effects
        .add(row(id, 1, EffectModification::GrantKeyword(KeywordFlag::Flying)));
    assert!(has_keyword(&game, id, KeywordFlag::Flying));

    game.continuous_effects
        .add(row(id, 2, EffectModification::RemoveKeyword(KeywordFlag::Flying)));
    assert!(
        !has_keyword(&game, id, KeywordFlag::Flying),
        "CR 113.10b: one removal clears the printed and the granted instance alike"
    );
}

// ---------------------------------------------------------------------------
// CR 101.2a — add/remove is NOT "can't overrides can"
// ---------------------------------------------------------------------------

/// CR 101.2 makes a "can't" effect beat a "can" effect, and CR 101.2a carves
/// ability add/remove *out* of that rule: rule 613 decides, so the later
/// timestamp prevails (CR 113.10c). Both directions are asserted, because a
/// "removal always wins" bug and an "addition always wins" bug each pass one of
/// them alone.
// COVERS: ATOM-101.2a-001
#[test]
fn test_add_after_remove_prevails_by_timestamp_not_by_cant_beating_can() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Flying]), 0);

    // Remove at ts 1, add back at ts 2. If "can't overrides can" applied, the
    // removal would win regardless of order. It does not apply.
    game.continuous_effects
        .add(row(id, 1, EffectModification::RemoveKeyword(KeywordFlag::Flying)));
    game.continuous_effects
        .add(row(id, 2, EffectModification::GrantKeyword(KeywordFlag::Flying)));
    assert!(
        has_keyword(&game, id, KeywordFlag::Flying),
        "CR 101.2a/113.10c: the later effect prevails, and it adds"
    );
}

// COVERS: ATOM-101.2a-001
#[test]
fn test_remove_after_add_prevails_by_timestamp() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    game.continuous_effects
        .add(row(id, 1, EffectModification::GrantKeyword(KeywordFlag::Flying)));
    game.continuous_effects
        .add(row(id, 2, EffectModification::RemoveKeyword(KeywordFlag::Flying)));
    assert!(
        !has_keyword(&game, id, KeywordFlag::Flying),
        "the mirror of the previous test: timestamp decides, not the operation"
    );
}

// ---------------------------------------------------------------------------
// CR 604.3a(2) — a granted ability is never characteristic-defining
// ---------------------------------------------------------------------------

/// The flag on `AbilityDef` asserts only the criteria that are properties of the
/// ability's *text*. Provenance is criterion (2), and it is maintained by
/// whoever writes the ability onto an object — so Layer 6 must clear it, while
/// copy (Layer 1) and text-changing (Layer 3) effects must not.
#[test]
fn test_granting_an_ability_clears_its_characteristic_defining_flag() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    let mut cda = inert_ability();
    cda.is_characteristic_defining = true;

    game.continuous_effects
        .add(row(id, 1, EffectModification::GrantAbility(Box::new(cda))));

    let granted = get_effective_abilities(&game, id);
    assert_eq!(granted.len(), 1);
    assert!(
        !granted[0].is_characteristic_defining,
        "CR 604.3a(2): an ability that arrived by being granted is never a CDA, \
         however its text reads"
    );
}
