//! "You control" on a continuous effect's filter — CR 109.5.
//!
//! `AffectedSet::Filter` used to carry a `controller: Option<PlayerId>` that
//! `register_static_effects` resolved from `PlayerRef::You` at ETB. That is a
//! *snapshot* of who controlled the source when it entered, and CR 109.5 says
//! the opposite: "for a static ability, [you] is the **current** controller of
//! the object it's on". So an anthem whose controller changes has to change
//! whose team it buffs, and a snapshot cannot.
//!
//! These tests write `BattlefieldEntity.controller` directly rather than going
//! through a control-changing effect, because Layer 2 does not exist yet. That
//! is not a fixture cheat: CR 110.2 makes that field the permanent's *default*
//! controller, and `compute_to_ceiling` seeds `chars.controller` from it, so
//! writing it is exactly the pre-Layer-2 half of what gaining control does.
//! When Layer 2 lands, `EffectModification::SetController` writes
//! `chars.controller` further along the same walk and every assertion here
//! keeps its meaning.

use mtgsim::cards::{creatures, phase5_pre_cards};
use mtgsim::engine::layers::types::{
    AffectedSet, ContinuousEffect, EffectModification, EffectOrigin, Layer, PtValue,
};
use mtgsim::oracle::characteristics::{get_effective_power, get_effective_toughness};
use mtgsim::test_support::{put_land_on_battlefield, put_on_battlefield, setup_two_player_game};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{Duration, PermanentFilter, PlayerRef};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::state::game_state::GameState;

/// A `+1/+1` anthem over `filter`, registered as if a spell had resolved
/// (CR 613.7b) under `controller`, from `source`.
///
/// Built by hand because no card in the pool says "creatures your opponents
/// control" or names an owner — the `PlayerRef` variants below are reachable
/// from the type, not yet from any card text.
fn register_anthem(
    game: &mut GameState,
    source: ObjectId,
    controller: PlayerId,
    filter: PermanentFilter,
) {
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(ContinuousEffect {
        id: 0,
        source,
        origin: EffectOrigin::Resolution,
        layer: Layer::Layer7cModifyPT,
        duration: Duration::Indefinite,
        controller,
        created_on_turn: game.turn_number,
        timestamp,
        affected: AffectedSet::Filter { filter },
        modification: EffectModification::ModifyPowerToughness {
            power: PtValue::Fixed(1),
            toughness: PtValue::Fixed(1),
        },
    });
}

fn creature_and(inner: PlayerRef) -> PermanentFilter {
    PermanentFilter::And(
        Box::new(PermanentFilter::ByType(CardType::Creature)),
        Box::new(PermanentFilter::ByController(inner)),
    )
}

/// The bug this phase exists to fix: Glorious Anthem buffed the team of
/// whoever controlled it at ETB, forever.
#[test]
fn test_anthem_follows_its_source_when_control_changes() {
    let mut game = setup_two_player_game();

    let my_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let opp_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);
    let anthem = put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);

    assert_eq!(get_effective_power(&game, my_bears), Some(3));
    assert_eq!(get_effective_toughness(&game, my_bears), Some(3));
    assert_eq!(get_effective_power(&game, opp_bears), Some(2));

    // Player 1 takes the anthem.
    game.battlefield.get_mut(&anthem).unwrap().controller = 1;

    // "Creatures you control" now means player 1's creatures.
    assert_eq!(get_effective_power(&game, opp_bears), Some(3));
    assert_eq!(get_effective_toughness(&game, opp_bears), Some(3));
    assert_eq!(get_effective_power(&game, my_bears), Some(2));
    assert_eq!(get_effective_toughness(&game, my_bears), Some(2));
}

/// The same anthem, control handed back. Nothing latches: the filter is
/// re-resolved on every walk, so the answer tracks the board rather than the
/// first change to it.
#[test]
fn test_anthem_control_change_is_not_one_way() {
    let mut game = setup_two_player_game();

    let my_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let anthem = put_on_battlefield(&mut game, phase5_pre_cards::glorious_anthem(), 0);

    game.battlefield.get_mut(&anthem).unwrap().controller = 1;
    assert_eq!(get_effective_power(&game, my_bears), Some(2));

    game.battlefield.get_mut(&anthem).unwrap().controller = 0;
    assert_eq!(get_effective_power(&game, my_bears), Some(3));
}

/// CR 611.2c — a continuous effect from a *resolution* has its "you" fixed when
/// it began. Moving the source permanent afterwards does not move the effect's
/// allegiance, which is the whole difference from the static-ability case
/// above. This is why `FilterPlayers::you` splits on `EffectOrigin` rather than
/// always asking the source.
///
/// **Deliberately carries no coverage annotation for ATOM-611.2c-001,** and the
/// annotation keyword is kept out of this comment on purpose: `specdb build`
/// scans for the token, not for the sentence around it, so writing out the pair
/// even to disclaim it creates the false link. That atom is 611.2c's other clause — the affected *set* locks in at resolution, so a
/// creature that turns white later is not caught by "all white creatures get
/// +1/+1". A `Resolution` effect over an `AffectedSet::Filter` does not do
/// that here; it re-filters every walk. No card produces the combination (all
/// three production `Filter` sites are static abilities, where re-filtering is
/// correct per CR 613.7a), so this is a note rather than a Deferred Migrations
/// item — but it is the reason the link would be a false one.
#[test]
fn test_resolution_effect_keeps_its_you_when_the_source_changes_hands() {
    let mut game = setup_two_player_game();

    let my_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let opp_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);
    // A land, so the source is neither buffed by the row nor a second source
    // of one — a Forest's mana ability is activated, so nothing registers.
    let source = put_land_on_battlefield(&mut game, mtgsim::cards::basic_lands::forest, 0);

    register_anthem(&mut game, source, 0, creature_and(PlayerRef::You));
    assert_eq!(get_effective_power(&game, my_bears), Some(3));
    assert_eq!(get_effective_power(&game, opp_bears), Some(2));

    game.battlefield.get_mut(&source).unwrap().controller = 1;

    assert_eq!(get_effective_power(&game, my_bears), Some(3));
    assert_eq!(get_effective_power(&game, opp_bears), Some(2));
}

/// `PlayerRef::Opponent` is matched as a predicate — "controlled by someone who
/// isn't you" — rather than resolved to an id. CR 102.2 makes it a single
/// player in a two-player game, but CR 102.3 makes "your opponents" a set in
/// multiplayer, so a `PlayerId` would be the wrong shape for half the CR.
#[test]
fn test_opponent_matches_every_controller_that_is_not_you() {
    let mut game = setup_two_player_game();

    let my_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let opp_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);
    let source = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);

    register_anthem(&mut game, source, 0, creature_and(PlayerRef::Opponent));

    assert_eq!(get_effective_power(&game, opp_bears), Some(3));
    assert_eq!(get_effective_power(&game, my_bears), Some(2));
    assert_eq!(get_effective_power(&game, source), Some(2));
}

/// `PlayerRef::Owner` means the *source's* owner (CR 108.3 / 110.2), which is
/// not the same player as "you" once a permanent has changed hands. No card
/// says this yet; the arm resolves rather than asserting because owner is
/// exactly determined, so there is nothing to guess at.
#[test]
fn test_owner_is_the_sources_owner_not_its_controller() {
    let mut game = setup_two_player_game();

    let p0_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let p1_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);
    // Owned by player 1; the effect's "you" is player 0, so the two differ.
    let source = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);

    register_anthem(&mut game, source, 0, creature_and(PlayerRef::Owner));

    assert_eq!(get_effective_power(&game, p1_bears), Some(3));
    assert_eq!(get_effective_power(&game, source), Some(3));
    assert_eq!(get_effective_power(&game, p0_bears), Some(2));
}

/// `PlayerRef::Player(id)` names a player outright and ignores both the
/// source's controller and the effect's.
#[test]
fn test_explicit_player_ignores_the_source() {
    let mut game = setup_two_player_game();

    let p0_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let p1_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);
    let source = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);

    register_anthem(&mut game, source, 0, creature_and(PlayerRef::Player(1)));

    assert_eq!(get_effective_power(&game, p1_bears), Some(3));
    assert_eq!(get_effective_power(&game, p0_bears), Some(2));
}

/// A bare `ByController` with no type constraint still filters by controller.
/// Under the old split this was the shape most at risk: `permanent_matches_filter`
/// returned `true` for `ByController(_)`, so correctness rested entirely on the
/// separate `controller` field — and `extract_controller_from_filter` only
/// walked `And` nodes, so a `ByController` under a `Not` was silently dropped.
#[test]
fn test_negated_controller_filter_is_honored() {
    let mut game = setup_two_player_game();

    let p0_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let p1_bears = put_on_battlefield(&mut game, creatures::grizzly_bears(), 1);
    let source = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);

    register_anthem(
        &mut game,
        source,
        0,
        PermanentFilter::Not(Box::new(PermanentFilter::ByController(PlayerRef::You))),
    );

    assert_eq!(get_effective_power(&game, p1_bears), Some(3));
    assert_eq!(get_effective_power(&game, p0_bears), Some(2));
}
