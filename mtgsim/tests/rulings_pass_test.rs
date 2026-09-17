//! The retroactive rulings pass — `engineering-practices.md` §3.4a.
//!
//! §3.4 makes a rulings pass part of registering a card, and these are the
//! cards that were registered before that rule existed. The ledger
//! (`plans/rulings-ledger.json`) holds the rulings themselves and
//! `plans/check_rulings.py` is the gate; a `// RULING:` line above a test is
//! the link the gate looks for, the way `// COVERS:` links an atom.
//!
//! **Not a `phase_XX` file, and that is the exception rather than a drift.**
//! `CLAUDE.md` files an integration test under the phase that registered its
//! cards, and §3.4's forward rule keeps doing that: a phase reads its own
//! cards' rulings and its tests go in its own file. The retroactive half is
//! not a phase — it walks the pool in ruling-count order and its cards come
//! from LC, LE, LF, LJ, CV and RF at once — so filing it by phase would
//! scatter one sitting across six files and make the next sitting's diff
//! unreadable. It appends here instead.
//!
//! **The queue is pool-first** (§3.4a): the pool is what every measurement
//! walks, which is where `codebase-state.md` item 82 came from. What landed
//! here is the queue's head — Cytoshape, Culling Drone and Yixlid Jailer, the
//! three pooled cards carrying the most rulings — and
//! `python plans/check_rulings.py --queue` prints the rest.
//!
//! A ruling this file does not answer is answered in the ledger, by a
//! disposition naming what it is waiting for. Neither answer is silent.

use mtgsim::cards::alpha::giant_growth;
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lc_cards::cerulean_wisps;
use mtgsim::cards::phase_ld_cards::march_of_the_machines;
use mtgsim::cards::phase_le_cards::{culling_drone, tarmogoyf};
use mtgsim::cards::phase_lf_cards::humility;
use mtgsim::cards::phase_lj_cards::yixlid_jailer;
use mtgsim::cards::phase_rf_cards::darksteel_colossus;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{
    get_effective_colors, get_effective_name, get_effective_power, get_effective_toughness,
    get_effective_types,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    card_of_type, fill_library, put_in_graveyard, put_in_hand, put_in_library, put_on_battlefield,
    setup_two_player_game, test_ctx, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

use std::sync::Arc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Cast the one card in `player`'s hand and resolve it, with one legal target.
///
/// The LC file's idiom: with a single legal target CR 102.2 makes the choice
/// forced, so nothing is scripted but the cast itself.
fn cast_and_resolve(game: &mut GameState, cast_index: usize) {
    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![cast_index]);
    assert_eq!(
        game.run_priority_round(&decisions).unwrap(),
        PriorityResult::ActionTaken
    );
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    game.run_priority_round(&decisions).unwrap();
}

/// Cytoshape `donor` onto `target`, picking the donor by id rather than by
/// prompt index — `phase_cv_integration_test.rs`'s helper, for the same reason:
/// a prompt index is an ordering the test would then be asserting by accident.
fn cytoshape_onto(game: &mut GameState, target: ObjectId, donor: ObjectId) {
    struct PickById(ObjectId);
    impl DecisionProvider for PickById {
        fn pick_n(
            &self,
            _game: &GameState,
            _player: PlayerId,
            ctx: &ChoiceContext,
            options: &[ChoiceOption],
            _bounds: (usize, usize),
        ) -> Vec<usize> {
            assert!(
                matches!(ctx.kind, ChoiceKind::ChooseCopySource { .. }),
                "unexpected prompt: {:?}",
                ctx.kind
            );
            vec![options
                .iter()
                .position(|o| matches!(o, ChoiceOption::Object(id) if *id == self.0))
                .expect("the donor is on offer")]
        }
        fn pick_number(
            &self,
            _g: &GameState,
            _p: PlayerId,
            _c: &ChoiceContext,
            min: u64,
            _max: u64,
        ) -> u64 {
            min
        }
        fn allocate(
            &self,
            _g: &GameState,
            _p: PlayerId,
            _c: &ChoiceContext,
            total: u64,
            buckets: &[ChoiceOption],
            _mins: &[u64],
            _maxs: Option<&[u64]>,
        ) -> Vec<u64> {
            let mut out = vec![0; buckets.len()];
            if !out.is_empty() {
                out[0] = total;
            }
            out
        }
        fn choose_ordering(
            &self,
            _g: &GameState,
            _p: PlayerId,
            _c: &ChoiceContext,
            items: &[ChoiceOption],
        ) -> Vec<usize> {
            (0..items.len()).collect()
        }
    }
    let card = mtgsim::cards::phase_cv_cards::cytoshape();
    let source = mtgsim::objects::object::GameObject::new(card.clone(), 0, Zone::Stack);
    let source_id = game.add_object(source);
    let ctx = mtgsim::engine::resolve::ResolutionContext {
        source: source_id,
        ability_source: None,
        controller: 0,
        targets: vec![mtgsim::engine::resolve::ResolvedTarget::Object(target)],
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(&card.abilities[0].effect.clone(), &ctx, &PickById(donor))
        .expect("Cytoshape resolves");
}

/// A creature whose printed colors are what the test is about.
fn colored_bear(name: &str, color: Color) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .color(color)
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .power_toughness(2, 2)
        .build()
}

// ===========================================================================
// Culling Drone — devoid (CR 702.114a), the pool's Layer 5 CDA
// ===========================================================================

// RULING: Culling Drone #2 - "A card with devoid is just colorless. It's not
//   colorless and the colors of mana in its mana cost."
/// The card's mana cost is {1}{B} and its printed color is black. Devoid is a
/// CDA, so Layer 5 answers before anything else can, and the answer is the
/// empty set rather than {black} plus colorless — colorless is the absence of
/// a color (CR 105.1), not a sixth one.
#[test]
fn test_devoid_makes_the_card_colorless_rather_than_colorless_and_black() {
    let mut game = setup_two_player_game();
    let drone = put_on_battlefield(&mut game, culling_drone(), 0);

    assert!(
        culling_drone().colors.contains(&Color::Black),
        "the printed card is black; devoid is what takes it away"
    );
    assert!(
        get_effective_colors(&game, drone).is_empty(),
        "CR 702.114a - devoid is a CDA, and a colorless object has no colors"
    );
}

// RULING: Culling Drone #3 - "Other cards and abilities can give a card with
//   devoid color. If that happens, it's just the new color, not that color and
//   colorless."
/// Cerulean Wisps is the pool's Layer 5 setter, and its row applies after the
/// CDA (CR 613.3: CDAs first, then everything else in timestamp order). The
/// assertion is the count: exactly one color, not blue alongside some residue
/// of the devoid application.
#[test]
fn test_a_color_given_to_a_devoid_card_replaces_colorlessness_rather_than_joining_it() {
    let mut game = setup_two_player_game();
    let drone = put_on_battlefield(&mut game, culling_drone(), 0);
    put_in_hand(&mut game, cerulean_wisps(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 1);

    assert!(get_effective_colors(&game, drone).is_empty());
    cast_and_resolve(&mut game, 1);

    let colors = get_effective_colors(&game, drone);
    assert_eq!(colors.len(), 1, "just the new color: {colors:?}");
    assert!(colors.contains(&Color::Blue));
}

// RULING: Culling Drone #4 - "Devoid works in all zones, not just on the
//   battlefield."
/// CR 604.3's "function in all zones", on the card whose whole text is one
/// CDA. `cda.rs` reads `game.objects` rather than `game.battlefield`, which is
/// the sentence being asserted here: the same object in a graveyard, a hand
/// and a library answers the same way.
#[test]
fn test_devoid_answers_the_same_in_every_zone() {
    let mut game = setup_two_player_game();
    let in_graveyard = put_in_graveyard(&mut game, culling_drone(), 0);
    let in_hand = put_in_hand(&mut game, culling_drone(), 0);
    let in_library = put_in_library(&mut game, culling_drone(), 0);

    for (zone, id) in [
        ("graveyard", in_graveyard),
        ("hand", in_hand),
        ("library", in_library),
    ] {
        assert!(
            get_effective_colors(&game, id).is_empty(),
            "CR 604.3 - devoid functions in a {zone} too"
        );
    }
}

// RULING: Culling Drone #5 - "If a card loses devoid, it will still be
//   colorless. This is because effects that change an object's color (like the
//   one created by devoid) are considered before the object loses devoid."
/// The ruling is a statement about layer order, and the engine's answer has to
/// come from the walk rather than from a rule about devoid: Layer 5 applies the
/// CDA while the ability is still there, and Humility's Layer 6 removes it one
/// layer later, by which time the color has been set.
///
/// The control assertion is Humility's other half — a 1/1 — which proves the
/// ability really was stripped rather than the row failing to apply at all.
#[test]
fn test_losing_devoid_at_layer_6_leaves_the_layer_5_colorlessness_in_place() {
    let mut game = setup_two_player_game();
    let drone = put_on_battlefield(&mut game, culling_drone(), 0);
    put_on_battlefield(&mut game, humility(), 0);

    assert_eq!(
        get_effective_power(&game, drone),
        Some(1),
        "Humility's layer 7b half really did apply, so its layer 6 half did too"
    );
    assert!(
        get_effective_colors(&game, drone).is_empty(),
        "CR 613.1 - layer 5 is walked before layer 6, so losing devoid afterwards \
         cannot un-apply it"
    );
}

// ===========================================================================
// Yixlid Jailer — "Cards in graveyards lose all abilities" (CR 613, layer 6)
// ===========================================================================

// RULING: Yixlid Jailer #6 - "If a card in a graveyard has an ability that
//   defines a * in its power or toughness, that * is 0. For example, Tarmogoyf
//   is a 0/1 creature card in your graveyard while you control Yixlid Jailer."
/// The ruling's own board, and both cards are in `PERFORMANCE_POOL`.
///
/// Tarmogoyf's P/T is a Layer 7a CDA and the Jailer's strip is Layer 6, so the
/// ability is gone before 7a looks for it. What is left is the printed box,
/// `*/1+*`, with nothing to supply the `*`: CR 208.2a makes that 0, and the
/// printed `1+` survives on its own — 0/1, not 0/0, which is also why it does
/// not die to a state-based action on the way.
///
/// The first assertion is `ATOM-604.3-001` whole — the corpus's own board for
/// "CDAs function in all zones" is a Tarmogoyf in a graveyard beside an instant
/// card, reading 2/3 — and it had been uncovered since the atom was written.
/// The rulings pass walked into it from the other direction, which is the
/// argument §3.4 makes for reading rulings at all.
// COVERS: ATOM-604.3-001
#[test]
fn test_a_stripped_tarmogoyf_in_a_graveyard_is_zero_one() {
    let mut game = setup_two_player_game();
    // Two card types in the graveyards, so the CDA's answer is visibly not 0.
    put_in_graveyard(&mut game, card_of_type("Filler Instant", CardType::Instant), 1);
    let goyf = put_in_graveyard(&mut game, tarmogoyf(), 0);

    assert_eq!(
        (get_effective_power(&game, goyf), get_effective_toughness(&game, goyf)),
        (Some(2), Some(3)),
        "CR 604.3 - the CDA functions in the graveyard, counting instant and creature"
    );

    put_on_battlefield(&mut game, yixlid_jailer(), 0);

    assert_eq!(
        (get_effective_power(&game, goyf), get_effective_toughness(&game, goyf)),
        (Some(0), Some(1)),
        "CR 613 layer 6 strips the CDA before layer 7a reads it; the printed box is */1+*"
    );
}

// RULING: Yixlid Jailer #3 - "Some replacement effects cause a card to be put
//   somewhere else instead of being put into a graveyard (such as that of
//   Darksteel Colossus). These effects mean the card is never actually put into
//   the graveyard, so Yixlid Jailer doesn't affect that ability."
/// The ruling names Darksteel Colossus and the crate registers it, so this is
/// the ruling's own board twice over — once from the battlefield, where the
/// Jailer plainly does not reach, and once from a library, where the card is
/// one zone change away from the graveyard the Jailer owns and still keeps its
/// ability. Both are the same sentence: the strip applies to cards that *are*
/// in a graveyard, and this card never becomes one.
#[test]
fn test_yixlid_jailer_does_not_stop_a_replacement_that_keeps_a_card_out_of_the_graveyard() {
    let mut game = setup_two_player_game();
    for _ in 0..4 {
        put_in_library(&mut game, card_of_type("Filler", CardType::Instant), 0);
    }
    put_on_battlefield(&mut game, yixlid_jailer(), 1);

    let from_battlefield = put_on_battlefield(&mut game, darksteel_colossus(), 0);
    game.change_zone(
        from_battlefield,
        Zone::Graveyard,
        ZoneChangeCause::Sacrificed,
        &test_ctx(),
    )
    .unwrap();
    assert_eq!(
        game.get_object(from_battlefield).unwrap().zone,
        Zone::Library,
        "the Jailer reaches graveyards, and this card never reached one"
    );

    let from_library = put_in_library(&mut game, darksteel_colossus(), 0);
    game.change_zone(from_library, Zone::Graveyard, ZoneChangeCause::Milled, &test_ctx())
        .unwrap();
    assert_eq!(game.get_object(from_library).unwrap().zone, Zone::Library);
    assert!(
        game.players[0].graveyard.is_empty(),
        "neither card was put into the graveyard, so neither lost anything"
    );
}

// ===========================================================================
// Cytoshape — the copy spine (CR 707)
// ===========================================================================

// RULING: Cytoshape #2 - "The creature copies the printed values of the chosen
//   creature ... It won't copy effects that have changed the creature's power,
//   toughness, types, color, or so on."
/// The half of the ruling `phase_cv_integration_test.rs` does not press.
/// `test_the_capture_excludes_counters_status_and_later_layers` proves the
/// power-and-toughness clause with an anthem and a counter; this is the
/// **types and color** clause, and it needs a donor whose types and colors are
/// wrong on the board rather than on the card.
///
/// March of the Machines supplies the type (a Layer 4 row making an artifact a
/// creature) and Cerulean Wisps the color (Layer 5). Neither is inside layer 1,
/// so neither is captured — the copy takes the printed artifact, not the
/// animated blue creature.
///
/// Partial for `ATOM-707.2-001`: the atom's board is a Clone *entering* as a
/// copy of a self-animating Chimeric Staff, so the copy is visibly a
/// noncreature artifact. Here the animator is March of the Machines, which
/// animates by class and so reaches the copy too — the claim survives at the
/// capture rather than on the board, which is where the assertion is.
// COVERS-PARTIAL: ATOM-707.2-001
#[test]
fn test_the_capture_excludes_layer_4_types_and_layer_5_colors() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, march_of_the_machines(), 0);
    let donor = put_on_battlefield(&mut game, mana_rock(), 0);
    put_in_hand(&mut game, cerulean_wisps(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Blue, 1);
    cast_and_resolve(&mut game, 1);

    // On the board the donor is an animated, blue artifact creature.
    assert!(get_effective_types(&game, donor).contains(&CardType::Creature));
    assert_eq!(get_effective_colors(&game, donor), [Color::Blue].into());

    let copyist = put_on_battlefield(&mut game, colored_bear("Green Bear", Color::Green), 0);
    cytoshape_onto(&mut game, copyist, donor);

    assert_eq!(get_effective_name(&game, copyist), "Mana Rock");
    assert!(
        get_effective_colors(&game, copyist).is_empty(),
        "CR 707.2 - the Wisps' layer 5 row is not a copiable value; the printed \
         artifact is colorless"
    );
    // The copy is a creature again, but only because March reaches it too — on
    // its own account, not through the capture. Its *printed* types are the
    // claim, and `copiable_values` is where that is visible.
    let values = mtgsim::engine::layers::copy::copiable_values(&game, donor).unwrap();
    assert!(
        !values.types.contains(&CardType::Creature),
        "CR 707.2 - March's layer 4 row is not captured either"
    );
    assert!(values.types.contains(&CardType::Artifact));
}

// RULING: Cytoshape #6 - "Effects that have already applied to the target
//   creature will continue to apply to it. For example, if Giant Growth had
//   given it +3/+3 earlier in the turn, then Cytoshape makes it a copy of
//   Grizzly Bears, it will be a 5/5 Grizzly Bears."
/// The ruling's own board, card for card, and all three are pooled.
///
/// The direction matters and is why `test_later_layers_still_apply_to_the_copy`
/// does not answer this: that test adds its Layer 7c modification *after* the
/// copy row, where this one is already on the board when the copy lands. The
/// layer walk is what makes the two the same answer — a Layer 7c row applies at
/// Layer 7c whatever order the rows were created in — so the arithmetic is the
/// assertion: 2/2 printed plus +3/+3, not the 4/4 a re-based pump would give.
#[test]
fn test_a_pump_that_predates_the_copy_still_applies_to_it() {
    let mut game = setup_two_player_game();
    let target = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    put_in_hand(&mut game, giant_growth(), 0);
    fill_library(&mut game, 0, 5);
    game.players[0].mana_pool.add(ManaType::Green, 1);

    // The one creature on the battlefield while the Growth is cast, so CR
    // 102.2 forces its target and the test scripts no index into a prompt.
    cast_and_resolve(&mut game, 1);
    assert_eq!(get_effective_power(&game, target), Some(4), "1/1 and +3/+3");

    let donor = put_on_battlefield(&mut game, grizzly_bears(), 0);

    cytoshape_onto(&mut game, target, donor);

    assert_eq!(get_effective_name(&game, target), "Grizzly Bears");
    assert_eq!(
        (get_effective_power(&game, target), get_effective_toughness(&game, target)),
        (Some(5), Some(5)),
        "a 5/5 Grizzly Bears - the Giant Growth applies to the copy, on top of 2/2"
    );
}

// RULING: Cytoshape #7 - "A creature may become a copy of itself this way. This
//   generally won't have any visible effect."
/// "Generally" is the interesting word: the copy row is real, so the assertion
/// is both halves — the characteristics do not move, and a row was registered
/// all the same. A short-circuit that declined to build the row would pass the
/// first assertion and fail the second, and would be wrong for the case the
/// ruling's hedge is about.
#[test]
fn test_a_creature_may_become_a_copy_of_itself() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);

    cytoshape_onto(&mut game, bears, bears);

    assert_eq!(get_effective_name(&game, bears), "Grizzly Bears");
    assert_eq!(get_effective_power(&game, bears), Some(2));
    assert_eq!(
        game.continuous_effects
            .iter()
            .filter(|e| matches!(
                e.modification,
                mtgsim::engine::layers::types::EffectModification::CopyFrom(_)
            ))
            .count(),
        1,
        "the effect applied; it just had nothing to change"
    );
}

// RULING: Cytoshape #8 - "At the end of the turn, the creature reverts to what
//   it was before. If two Cytoshapes affect the same creature on the same turn,
//   they'll both wear off at the same time."
/// The second sentence. `test_a_turn_bounded_copy_and_its_derived_rows_expire_together`
/// has the first; this is the one about *two* rows, which is not the same claim
/// — a later copy of a copy could plausibly have been built by rewriting the
/// first row, and then only one thing would expire.
#[test]
fn test_two_copies_on_one_creature_wear_off_together() {
    let mut game = setup_two_player_game();
    let target = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let first = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let second = put_on_battlefield(&mut game, colored_bear("Second Donor", Color::Red), 0);

    cytoshape_onto(&mut game, target, first);
    assert_eq!(get_effective_name(&game, target), "Grizzly Bears");
    cytoshape_onto(&mut game, target, second);
    assert_eq!(get_effective_name(&game, target), "Second Donor");
    assert_eq!(
        game.continuous_effects
            .iter()
            .filter(|e| matches!(
                e.modification,
                mtgsim::engine::layers::types::EffectModification::CopyFrom(_)
            ))
            .count(),
        2,
        "two spells, two rows - the second did not rewrite the first"
    );

    game.continuous_effects
        .remove_expired_at_cleanup(0, game.turn_number);

    assert_eq!(
        get_effective_name(&game, target),
        "Test Creature",
        "CR 514.2 - both rows carried the same duration, so neither outlives the other"
    );
}

/// An artifact with a mana value March of the Machines animates into a creature
/// with a positive toughness — Sol Ring's shape, without Sol Ring's ability,
/// so the capture assertion is about types and colors alone.
fn mana_rock() -> Arc<CardData> {
    CardDataBuilder::new("Mana Rock")
        .card_type(CardType::Artifact)
        .mana_cost(ManaCost::build(&[], 3))
        .build()
}
