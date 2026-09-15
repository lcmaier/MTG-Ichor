//! Phase LJ integration tests: a continuous effect that reaches another zone
//! (`layers-architecture.md` §13c).
//!
//! The headline test is ATOM-614.12-001, which is the CR's own worked example
//! for 614.12 and needs **both** halves of this phase at once: the filter has
//! to reach a graveyard at all (LJ's working-set change), and the look-ahead
//! has to keep it out of the entry (RC-4's overlay). A unit test of the filter
//! would prove the first and say nothing about the second, which is why the
//! atom is tested here rather than beside `Board::in_zones_or_entering`.
//!
//! The rest assert the guard: on a board with no zone-reaching row, nothing
//! about the working set moves. That is `RegistryScopeSummary::reachable_zones`
//! earning its place — the claim is a structural zero, so it is testable.

use mtgsim::cards::phase_le_cards::tarmogoyf;
use mtgsim::cards::phase_lf_cards::humility;
use mtgsim::cards::phase_lj_cards::{
    graveyard_painter, graveyard_reveler, scarwood_treefolk, yixlid_jailer,
};
use mtgsim::engine::actions::ZoneChangeCause;
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_power, get_effective_toughness,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_graveyard, put_in_hand, put_on_battlefield, setup_two_player_game, test_ctx,
    vanilla_creature,
};
use mtgsim::types::effects::ObjectSet;
use mtgsim::types::ids::ObjectId;
use mtgsim::types::zones::{Zone, ZoneSet};

/// Move a card from its owner's graveyard onto the battlefield, the way a
/// reanimation spell's resolution does.
fn reanimate(game: &mut GameState, id: ObjectId) {
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &test_ctx())
        .expect("it enters");
}

// ---------------------------------------------------------------------------
// The facility: a filter row reaches a graveyard
// ---------------------------------------------------------------------------

/// Yixlid Jailer strips a graveyard card's abilities — the thing that was
/// inexpressible before this phase.
///
/// Before LJ a graveyard card was a `NonMember` of every pass and received
/// printed characteristics plus its own CDAs; no filter row could name it,
/// whatever the filter said. The assertion is on the *oracle*, not on the
/// registry, because registry membership is not effect existence
/// (`CLAUDE.md`): the row has to be re-read and applied at Layer 6 for this
/// to come out empty.
#[test]
fn test_a_graveyard_cards_abilities_are_stripped_by_a_zone_reaching_row() {
    let mut game = setup_two_player_game();

    let treefolk = put_in_graveyard(&mut game, scarwood_treefolk(), 0);
    assert!(
        !get_effective_abilities(&game, treefolk).is_empty(),
        "the Treefolk prints an ability, and in a graveyard with no Jailer it keeps it"
    );

    put_on_battlefield(&mut game, yixlid_jailer(), 1);

    assert!(
        get_effective_abilities(&game, treefolk).is_empty(),
        "CR 613 layer 6: cards in graveyards lose all abilities"
    );
}

/// And it stops the moment the Jailer leaves — the row is re-read every pass,
/// never captured at ETB.
///
/// This is one of the card's own printed rulings, and it costs no code: the
/// effect exists exactly while its source has the ability
/// (CR 604.2), so removing the source retires it.
#[test]
fn test_the_strip_ends_when_the_source_leaves_the_battlefield() {
    let mut game = setup_two_player_game();

    let treefolk = put_in_graveyard(&mut game, scarwood_treefolk(), 0);
    let jailer = put_on_battlefield(&mut game, yixlid_jailer(), 1);
    assert!(get_effective_abilities(&game, treefolk).is_empty(), "stripped while it is out");

    game.change_zone(jailer, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx())
        .expect("it dies");

    assert!(
        !get_effective_abilities(&game, treefolk).is_empty(),
        "the Treefolk has its ability back the moment the Jailer stops having its"
    );
}

/// A battlefield-scoped row does **not** reach a graveyard card, which is the
/// other half of the gate being a gate.
///
/// Humility is "all creatures lose all abilities" over
/// `ObjectFilter::ByType(Creature)` on the battlefield. The Treefolk in a
/// graveyard is a creature card by every characteristic, and Humility still
/// must not touch it — if it did, the zone field would be decorative.
#[test]
fn test_a_battlefield_scoped_row_does_not_reach_a_graveyard_card() {
    let mut game = setup_two_player_game();

    let treefolk = put_in_graveyard(&mut game, scarwood_treefolk(), 0);
    put_on_battlefield(&mut game, humility(), 1);

    assert!(
        !get_effective_abilities(&game, treefolk).is_empty(),
        "Humility is battlefield-scoped; a card in a graveyard is not a creature it reaches"
    );
}

// ---------------------------------------------------------------------------
// ATOM-614.12-001 — the CR's worked example, and why it needs both halves
// ---------------------------------------------------------------------------

/// **The atom.** Scarwood Treefolk enters from a graveyard while Yixlid Jailer
/// is out, and it enters **tapped** — even though it had no abilities at all
/// in the graveyard a moment earlier.
///
/// CR 614.12: an entry replacement is checked against the permanent as it
/// *would exist on the battlefield*. On the battlefield the Jailer does not
/// reach it, so "this creature enters tapped" is on the frame the look-ahead
/// builds, and the entry is modified.
///
/// **Both halves of this phase are load-bearing and the test fails without
/// either.** Without LJ's working-set change the Jailer reaches nothing, the
/// Treefolk keeps its ability in the graveyard, and it enters tapped for the
/// wrong reason — the assertion passes while proving nothing, which is why the
/// test above it asserts the strip separately. Without the entering arm of
/// `Board::in_zones_or_entering` the Jailer *does* reach the entering object,
/// the ability is stripped before `gather` reads it, and the Treefolk enters
/// untapped — the wrong answer, and the one a naive `zones.contains(obj.zone)`
/// gives, since an entering object is still in its source zone.
// COVERS: ATOM-614.12-001
#[test]
fn test_a_treefolk_reanimated_under_yixlid_jailer_still_enters_tapped() {
    let mut game = setup_two_player_game();

    put_on_battlefield(&mut game, yixlid_jailer(), 1);
    let treefolk = put_in_graveyard(&mut game, scarwood_treefolk(), 0);

    // The premise, asserted rather than assumed: in the graveyard it has
    // nothing, so a look-ahead that read the card *where it is* would find no
    // entry modification at all.
    assert!(
        get_effective_abilities(&game, treefolk).is_empty(),
        "premise: the Jailer has taken its abilities away while it is in the graveyard"
    );

    reanimate(&mut game, treefolk);

    assert!(
        game.battlefield.get(&treefolk).expect("it is on the battlefield").tapped,
        "CR 614.12: checked as it would exist on the battlefield, where the Jailer does not reach"
    );
    assert!(
        !get_effective_abilities(&game, treefolk).is_empty(),
        "and it has its ability on the battlefield, which is the same sentence from the other side"
    );
}

// ---------------------------------------------------------------------------
// The guard — `reachable_zones` is what makes the cost structurally zero
// ---------------------------------------------------------------------------

/// With no zone-reaching row registered, the registry reaches the battlefield
/// and nothing else.
///
/// The claim §13c decision 2 rests on: the seed's zone loop does not run, so a
/// board that plays none of these cards pays nothing for the facility. A
/// measurement could only say "small"; this says "none".
#[test]
fn test_an_ordinary_board_reaches_no_zone_beyond_the_battlefield() {
    let mut game = setup_two_player_game();

    put_on_battlefield(&mut game, humility(), 1);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    put_in_graveyard(&mut game, scarwood_treefolk(), 0);
    put_in_hand(&mut game, scarwood_treefolk(), 0);

    let reach = game.continuous_effects.summary().reachable_zones;
    assert_eq!(reach, ZoneSet::BATTLEFIELD, "every registered row is battlefield-scoped");
    assert!(
        reach.beyond_battlefield().is_empty(),
        "so the seed adds no member outside the battlefield"
    );
}

/// The Jailer turns exactly one zone on, and not the hidden ones.
///
/// The narrowing item 9 asks for in its own words, and the reason this is a
/// `ZoneSet` rather than §5.1's `touches_hidden_zones: bool` — a graveyard row
/// must not drag libraries and hands into the working set.
#[test]
fn test_the_jailer_reaches_graveyards_and_only_graveyards() {
    let mut game = setup_two_player_game();

    put_on_battlefield(&mut game, yixlid_jailer(), 1);

    let reach = game.continuous_effects.summary().reachable_zones;
    assert!(reach.contains(Zone::Graveyard), "the Jailer names graveyards");
    assert_eq!(
        reach.beyond_battlefield(),
        ZoneSet::GRAVEYARD,
        "and no other zone joins the working set"
    );
    assert!(
        !reach.touches_hidden_zones(),
        "a graveyard is public (CR 400.2); no library or hand is walked for this card"
    );
}

/// A card in a hand is untouched by a graveyard-scoped row, which is the
/// per-zone half of the same claim.
#[test]
fn test_a_graveyard_row_does_not_reach_a_card_in_hand() {
    let mut game = setup_two_player_game();

    let in_hand = put_in_hand(&mut game, scarwood_treefolk(), 0);
    let in_graveyard = put_in_graveyard(&mut game, scarwood_treefolk(), 0);
    put_on_battlefield(&mut game, yixlid_jailer(), 1);

    assert!(
        get_effective_abilities(&game, in_graveyard).is_empty(),
        "the graveyard copy is stripped"
    );
    assert!(
        !get_effective_abilities(&game, in_hand).is_empty(),
        "the hand copy is not: the row names graveyards, and the gate reads the object's zone"
    );
}

// ---------------------------------------------------------------------------
// Determinism — the walk order of a member outside the battlefield
// ---------------------------------------------------------------------------

/// Zone members enter the working set in their zone's own order, seat first,
/// and that order is the same on every run.
///
/// CR 613.7 orders *effects* by timestamp and says nothing about the objects
/// they apply to, so a member here needs a deterministic position rather than
/// a timestamp — which is what lets this phase leave CR 613.7d to A5. The
/// containers are `Vec`s, so nothing reaches a `HashMap` (`CLAUDE.md`,
/// determinism at the decision boundary).
#[test]
fn test_zone_members_are_enumerated_in_their_zones_own_order() {
    let mut game = setup_two_player_game();

    let p0: Vec<ObjectId> = (0..3)
        .map(|_| put_in_graveyard(&mut game, scarwood_treefolk(), 0))
        .collect();
    let p1: Vec<ObjectId> = (0..2)
        .map(|_| put_in_graveyard(&mut game, scarwood_treefolk(), 1))
        .collect();

    let expected: Vec<ObjectId> = p0.iter().chain(p1.iter()).copied().collect();
    assert_eq!(
        game.zone_ids_ordered(Zone::Graveyard),
        expected,
        "seat order, then each player's graveyard in its own order (CR 404.3)"
    );
    for _ in 0..8 {
        assert_eq!(
            game.zone_ids_ordered(Zone::Graveyard),
            expected,
            "and the same order every time it is asked"
        );
    }
}

/// `battlefield_entities` still means the battlefield, so a zone-reaching row
/// is invisible to a count.
///
/// The finding that shaped the phase: every CR-level count slices that prefix,
/// so a graveyard member appended anywhere but last would join every
/// "creatures you control" count in the game. Asserted through a count that a
/// card actually reads rather than through the field, which is private.
#[test]
fn test_a_graveyard_member_is_invisible_to_a_battlefield_count() {
    let mut game = setup_two_player_game();

    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    put_on_battlefield(&mut game, yixlid_jailer(), 1);
    let before = game.battlefield_ids_ordered().len();

    for _ in 0..4 {
        put_in_graveyard(&mut game, scarwood_treefolk(), 0);
    }

    assert_eq!(
        game.battlefield_ids_ordered().len(),
        before,
        "four new members of the working set, and none of them is on the battlefield"
    );
}

// ---------------------------------------------------------------------------
// The recipient lowers to the set it says it does
// ---------------------------------------------------------------------------

/// `FilteredObjectsIn` lowers to `ObjectSet::filter_in`, and the registered
/// row carries the zones.
///
/// One assertion on the seam between a card definition and the registry,
/// because everything above reads through the oracle and would pass on a row
/// that reached the right objects for the wrong reason.
#[test]
fn test_the_jailers_registered_row_is_graveyard_scoped() {
    let mut game = setup_two_player_game();

    let jailer = put_on_battlefield(&mut game, yixlid_jailer(), 1);

    let rows: Vec<&ObjectSet> = game
        .continuous_effects
        .iter()
        .filter(|e| e.source == jailer)
        .map(|e| &e.affected_objects)
        .collect();
    assert_eq!(rows.len(), 1, "one ability, one row");
    assert!(
        matches!(rows[0], ObjectSet::Filter { zones, .. } if *zones == ZoneSet::GRAVEYARD),
        "the row names graveyards and not the battlefield: {:?}",
        rows[0]
    );
}

// ---------------------------------------------------------------------------
// What can *see* a zone-reaching effect — the owner's review question
// ---------------------------------------------------------------------------

/// **A zone-reaching row changes a graveyard card's characteristics, and a
/// game rule reads the change.** No oracle query anywhere in the chain.
///
/// Asked at the review, and it is the right question: Yixlid Jailer strips
/// *abilities* from graveyard cards, and nothing in the engine reads a
/// graveyard card's abilities yet — flashback, retrace and Bridge from Below
/// are each gated on CR 113.6, which is A5. So the Jailer's tests above assert
/// the mechanism directly, and something had to assert a *consequence*.
///
/// A **color** in a graveyard is read today, by `Condition::CardInGraveyard`,
/// and since this PR folded `CardFilter` into `ObjectFilter` that condition can
/// ask `ByColor`. The loop is therefore: Graveyard Painter's row reaches the
/// graveyard at Layer 5, so a black card there is now also red; Graveyard
/// Reveler's CR 604.2 condition sees a red card in its controller's graveyard;
/// its `ModifyPowerToughness` row exists; the Reveler is 3/3.
///
/// **Every link is a rule.** The assertion is the Reveler's power, which is
/// two layers and two cards away from the row under test.
#[test]
fn test_a_zone_reaching_row_changes_a_characteristic_a_rule_reads() {
    let mut game = setup_two_player_game();

    // A black card in seat 0's graveyard, and the Reveler watching for a red one.
    put_in_graveyard(&mut game, yixlid_jailer(), 0);
    let reveler = put_on_battlefield(&mut game, graveyard_reveler(), 0);

    assert_eq!(
        get_effective_power(&game, reveler),
        Some(1),
        "premise: the only card in the graveyard is black, so the condition is false"
    );

    put_on_battlefield(&mut game, graveyard_painter(), 0);

    assert_eq!(
        get_effective_power(&game, reveler),
        Some(3),
        "the row reached the graveyard, the card there is red, and CR 604.2 turned the row on"
    );
}

/// And it switches back off when the row goes, through the same chain.
///
/// The half that proves the condition is re-asked rather than latched: CR
/// 604.2's existence check runs every pass, so removing the Painter removes
/// the color, which removes the Reveler's bonus.
#[test]
fn test_the_rule_stops_reading_it_when_the_row_goes() {
    let mut game = setup_two_player_game();

    put_in_graveyard(&mut game, yixlid_jailer(), 0);
    let reveler = put_on_battlefield(&mut game, graveyard_reveler(), 0);
    let painter = put_on_battlefield(&mut game, graveyard_painter(), 0);
    assert_eq!(get_effective_power(&game, reveler), Some(3), "on while the Painter is out");

    game.change_zone(painter, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx())
        .expect("it dies");

    assert_eq!(
        get_effective_power(&game, reveler),
        Some(1),
        "the row is gone, so the graveyard card is black again and the condition is false"
    );
}

// ---------------------------------------------------------------------------
// The working set shrinks when the row that widened it leaves
// ---------------------------------------------------------------------------

/// The mask closes again when the last zone-reaching row is removed, so the
/// working set returns to the battlefield.
///
/// Asked at the review beside the strip test, and it is a different claim from
/// "the effect stopped applying": an effect can stop applying while the
/// objects it reached stay in the working set, which would be a permanent cost
/// for a card that left play. `RegistryScopeSummary` is recomputed from the
/// rows on every mutation (`ContinuousEffectRegistry::mutating`), so there is
/// no drift to accumulate — this asserts that rather than assuming it.
#[test]
fn test_the_working_set_narrows_again_when_the_last_zone_row_leaves() {
    let mut game = setup_two_player_game();

    for _ in 0..3 {
        put_in_graveyard(&mut game, scarwood_treefolk(), 0);
    }
    let jailer = put_on_battlefield(&mut game, yixlid_jailer(), 1);
    assert_eq!(
        game.continuous_effects.summary().reachable_zones.beyond_battlefield(),
        ZoneSet::GRAVEYARD,
        "while the Jailer is out, graveyards are in the working set"
    );

    game.change_zone(jailer, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx())
        .expect("it dies");

    assert!(
        game.continuous_effects.summary().reachable_zones.beyond_battlefield().is_empty(),
        "the row left with its source, so the seed stops adding graveyard members"
    );
}

/// Two zone-reaching rows, and removing one does not close the zone the other
/// still names.
///
/// The mask is a union, so it must narrow to what is *left* rather than to
/// empty — the failure mode a hand-maintained counter has and a recomputed
/// summary does not.
#[test]
fn test_removing_one_of_two_rows_leaves_the_zone_the_other_names() {
    let mut game = setup_two_player_game();

    let jailer = put_on_battlefield(&mut game, yixlid_jailer(), 1);
    put_on_battlefield(&mut game, graveyard_painter(), 0);

    game.change_zone(jailer, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx())
        .expect("it dies");

    assert_eq!(
        game.continuous_effects.summary().reachable_zones.beyond_battlefield(),
        ZoneSet::GRAVEYARD,
        "the Painter still names graveyards, so the zone stays open"
    );
}

// ---------------------------------------------------------------------------
// Complements — a `ZoneSet` can say "everywhere but the battlefield"
// ---------------------------------------------------------------------------

/// `EVERYWHERE_BUT_BATTLEFIELD` is an ordinary set, and `without` is general.
///
/// The first cut of `ZoneSet`'s doc comment claimed a complement was
/// inexpressible and called that the correct limit. It is not: Grist, the
/// Hunger Tide is "as long as Grist isn't on the battlefield, it's a 1/1 Insect
/// creature in addition to its other types", and Mycosynth Lattice and
/// Painter's Servant both open on "all cards that aren't on the battlefield".
/// Corrected at the review — the limit that matters is that a complement may
/// not hide inside a **filter tree**, where recovering the reach needs an
/// abstract interpretation. On a concrete bitmask it is bit arithmetic.
#[test]
fn test_a_zone_set_can_name_everywhere_but_the_battlefield() {
    let complement = ZoneSet::EVERYWHERE_BUT_BATTLEFIELD;

    assert!(!complement.contains(Zone::Battlefield), "the one zone it excludes");
    for zone in [
        Zone::Library,
        Zone::Hand,
        Zone::Graveyard,
        Zone::Stack,
        Zone::Exile,
        Zone::Command,
    ] {
        assert!(complement.contains(zone), "{zone:?} is everywhere else");
    }
    assert_eq!(
        complement,
        ZoneSet::ALL.without(ZoneSet::BATTLEFIELD),
        "the constant is the general operation, not a special case"
    );
    assert_eq!(
        complement.beyond_battlefield(),
        complement,
        "and it is entirely beyond the battlefield, so every zone of it is swept"
    );
}

// ---------------------------------------------------------------------------
// Yixlid Jailer + Tarmogoyf — two pooled cards, and the strip reaches a CDA
// ---------------------------------------------------------------------------

/// **Stripping a graveyard card's abilities changes its power and toughness**,
/// because one of the abilities is a CDA.
///
/// Raised at the review against this document's claim that the Jailer's effect
/// is not observable yet, and it is the sharper case: the *ability list* is
/// what nothing reads until CR 113.6, but a CDA's removal cascades into a
/// characteristic that is read directly. CR 208.2a gives Tarmogoyf's CDA "this
/// ability functions everywhere, even outside the game", which is why a
/// Tarmogoyf in a graveyard has a computed P/T at all; take the ability away
/// and the seed is what is left — `power_toughness(0, 1)`, the printed `*/1+*`.
///
/// **A different code path from every other test here.** `engine/layers/cda.rs`
/// applies CDAs off the object's *own effective ability list* at layers 4, 5
/// and 7a, never from the registry (CR 604.3a(3), `CLAUDE.md`). So this asserts
/// that the Jailer's Layer 6 strip lands on the list that Layer 7a then reads,
/// for an object in a graveyard — the mechanism `layers-architecture.md` §6
/// describes for Humility on the battlefield, now one zone over.
///
/// **Both cards are in `PERFORMANCE_POOL`**, so this is a board a measured
/// fuzz game can build rather than a fixture.
#[test]
fn test_the_jailer_strips_a_graveyard_tarmogoyfs_cda_and_its_pt_falls_to_the_seed() {
    let mut game = setup_two_player_game();

    // Two card types in graveyards: the Goyf itself (creature) and a land.
    let goyf = put_in_graveyard(&mut game, tarmogoyf(), 0);
    put_in_graveyard(&mut game, mtgsim::cards::basic_lands::forest(), 1);

    assert_eq!(
        (get_effective_power(&game, goyf), get_effective_toughness(&game, goyf)),
        (Some(2), Some(3)),
        "CR 208.2a — the CDA functions in the graveyard, and two card types are there"
    );

    put_on_battlefield(&mut game, yixlid_jailer(), 1);

    assert_eq!(
        (get_effective_power(&game, goyf), get_effective_toughness(&game, goyf)),
        (Some(0), Some(1)),
        "the CDA is gone with the rest of its abilities, so the printed seed is what is left"
    );
}

/// The same board with the Jailer on the *battlefield* side: a Tarmogoyf in
/// play is untouched, because it is not a card in a graveyard.
///
/// The half that keeps the test above honest. "Cards in graveyards lose all
/// abilities" is scoped by zone and not by what the ability *counts* — a
/// battlefield Goyf keeps its CDA and keeps counting the graveyards, which
/// are exactly the zones the Jailer is reaching into. Getting this wrong in
/// the other direction is the more tempting bug: the Jailer does not reduce
/// the count, because it removes abilities and not card **types**.
#[test]
fn test_the_jailer_does_not_touch_a_tarmogoyf_on_the_battlefield() {
    let mut game = setup_two_player_game();

    put_in_graveyard(&mut game, mtgsim::cards::basic_lands::forest(), 1);
    let goyf = put_on_battlefield(&mut game, tarmogoyf(), 0);
    put_on_battlefield(&mut game, yixlid_jailer(), 1);

    assert_eq!(
        (get_effective_power(&game, goyf), get_effective_toughness(&game, goyf)),
        (Some(1), Some(2)),
        "one card type in graveyards (the land); the Goyf is a permanent, so the row misses it"
    );
}
