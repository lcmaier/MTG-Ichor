//! Phase CM-1 — cost modification (`plans/cost-architecture.md`).
//!
//! CR 601.2f's order, end to end through `cast_spell`: base, plus additional
//! costs and increases, minus reductions in the order the caster chooses,
//! floored at {0}, then Trinisphere, then locked. **Every test here pays
//! from an exact pool**: the cast succeeds with exactly the locked total in
//! the pool and fails with one mana fewer, which is the strongest statement
//! a payment can make about what was determined — and one that no direct
//! read of the pipeline can make, since it is the cast that locks.
//!
//! Three printed cards, one per position in the order, and the fixtures in
//! `phase_cm_cards` for the boards the corpus names that no three printed
//! cards build.
//!
//! **CM-2** adds the spell's own cost abilities (CR 113.6d, 702.41a) at the
//! end of this file: Myr Enforcer and Frogmite, the gather's second source,
//! and the first reduction whose amount is read off the board.
//!
//! **CM-3** adds the payment side (CR 601.2h, 118.8b, 732.1): a sacrifice
//! paid through the chokepoint, a mandatory additional cost, and the boards
//! where a reducer leaves *after* the total is locked. The exact-pool
//! discipline above is what makes the lock-in observable — a cost recomputed
//! after the window would be payable from the same pool and would leave a
//! different amount of mana behind.

use std::cell::RefCell;

use mtgsim::cards::registry::CardRegistry;
use mtgsim::cards::{alpha, phase_cm_cards, phase_lf_cards};
use mtgsim::engine::actions::ZoneChangeCause;
use mtgsim::engine::layers::types::{EffectModification, Layer};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::mana_helpers::castable_spells;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_hand, put_on_battlefield, registered, setup_two_player_game, static_ability, test_ctx,
    vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::cost_modification::{CostChange, CostModificationDef};
use mtgsim::types::effects::{Effect, ObjectFilter};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Put exactly `pool` into the player's mana pool, cast `card` from hand with
/// `dp`, and report whether it was cast. The pool is emptied first, so a
/// success means the locked total was payable from `pool` and nothing else.
fn cast_from_pool(
    game: &mut GameState,
    player: PlayerId,
    card: Arc<CardData>,
    pool: &[(ManaType, u64)],
    dp: &dyn DecisionProvider,
) -> Result<ObjectId, String> {
    let id = put_in_hand(game, card, player);
    let types = [
        ManaType::White,
        ManaType::Blue,
        ManaType::Black,
        ManaType::Red,
        ManaType::Green,
        ManaType::Colorless,
    ];
    for t in types {
        let have = game.players[player].mana_pool.amount(t);
        if have > 0 {
            game.players[player].mana_pool.remove(t, have).unwrap();
        }
    }
    for &(t, n) in pool {
        game.players[player].mana_pool.add(t, n);
    }
    game.cast_spell(player, id, dp).map(|_| id)
}

/// The exact-pool assertion: castable from `pool`, and not from `pool` with
/// one `short` fewer. Each attempt is its own game state, built by `board`.
fn assert_costs_exactly(
    board: impl Fn() -> GameState,
    card: impl Fn() -> Arc<CardData>,
    pool: &[(ManaType, u64)],
    short: ManaType,
    what: &str,
) {
    let mut game = board();
    let cast = cast_from_pool(&mut game, 0, card(), pool, &RecordingDecisionProvider::picking(0));
    assert!(cast.is_ok(), "{what}: not castable from {pool:?}: {cast:?}");
    assert_eq!(game.players[0].mana_pool.total(), 0, "{what}: the whole pool is the cost");

    let mut less: Vec<(ManaType, u64)> = pool.to_vec();
    let slot = less.iter_mut().find(|(t, n)| *t == short && *n > 0).expect("something to remove");
    slot.1 -= 1;
    let mut game = board();
    let cast = cast_from_pool(&mut game, 0, card(), &less, &RecordingDecisionProvider::picking(0));
    assert!(cast.is_err(), "{what}: castable from {less:?}, one short");
}

fn bolt() -> Arc<CardData> {
    alpha::lightning_bolt()
}

/// A {1} artifact with no text — a colorless spell for the lock-in board,
/// whose generic-only cost keeps the payment's split trivial.
fn tin_trinket() -> Arc<CardData> {
    tin_trinket_costing(1)
}

/// The same fixture at any generic cost, for a total that is all generic.
fn tin_trinket_costing(n: u8) -> Arc<CardData> {
    CardDataBuilder::new("Tin Trinket")
        .mana_cost(ManaCost::build(&[], n))
        .card_type(CardType::Artifact)
        .build()
}

fn thalia_board(controller: PlayerId) -> GameState {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase_cm_cards::thalia_guardian_of_thraben(), controller);
    game
}

// ---------------------------------------------------------------------------
// The three positions, one card each
// ---------------------------------------------------------------------------

/// "Noncreature spells cost {1} more to cast" — an increase, and one that
/// taxes "each spell that's not a creature spell, including your own" (her
/// ruling): both players pay it, and a creature spell does not.
#[test]
fn test_thalia_taxes_every_players_noncreature_spells() {
    assert_costs_exactly(|| thalia_board(0), bolt, &[(ManaType::Red, 2)], ManaType::Red, "own Bolt under Thalia");
    assert_costs_exactly(|| thalia_board(1), bolt, &[(ManaType::Red, 2)], ManaType::Red, "Bolt under their Thalia");
    assert_costs_exactly(
        || thalia_board(0),
        || vanilla_creature(2, 2, &[]),
        &[(ManaType::Green, 2)],
        ManaType::Green,
        "a creature spell is not taxed",
    );
}

/// "Instant and sorcery spells you cast cost {1} less to cast" — a reduction,
/// on the controller's spells only, and only on generic (CR 118.7a: a {R}
/// Bolt has nothing for it to reduce).
// COVERS: ATOM-118.7-001
#[test]
fn test_electromancer_reduces_the_generic_of_its_controllers_instants() {
    let board = |controller: PlayerId| {
        move || {
            let mut game = setup_two_player_game();
            put_on_battlefield(&mut game, phase_cm_cards::goblin_electromancer(), controller);
            game
        }
    };
    // {1}{R} less {1} is {R}.
    assert_costs_exactly(board(0), phase_cm_cards::ember_lesson, &[(ManaType::Red, 1)], ManaType::Red, "Ember Lesson");
    // Theirs reduces nothing of yours.
    assert_costs_exactly(board(1), phase_cm_cards::ember_lesson, &[(ManaType::Red, 2)], ManaType::Red, "their Electromancer");
    // Nothing generic to reduce: Bolt is still {R}.
    assert_costs_exactly(board(0), bolt, &[(ManaType::Red, 1)], ManaType::Red, "Bolt has no generic");
}

/// CR 601.2f's first two positions on one spell: the increase is added, then
/// the reduction subtracted. Thalia's {1} more and Electromancer's {1} less on
/// a {R} Bolt is {R} — and it is not "{R} less {1} is {R}, plus {1} is
/// {1}{R}", which is what the other order would give.
// COVERS: ATOM-613.11-002
#[test]
fn test_increases_apply_before_reductions() {
    let board = || {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, phase_cm_cards::thalia_guardian_of_thraben(), 0);
        put_on_battlefield(&mut game, phase_cm_cards::goblin_electromancer(), 0);
        game
    };
    assert_costs_exactly(board, bolt, &[(ManaType::Red, 1)], ManaType::Red, "Thalia + Electromancer on Bolt");
}

/// Trinisphere is the third position: applied after the reduction has done
/// what it can, and only while untapped.
#[test]
fn test_trinisphere_raises_the_total_to_three_after_reductions_while_untapped() {
    let board = |tapped: bool| {
        move || {
            let mut game = setup_two_player_game();
            let sphere = put_on_battlefield(&mut game, phase_cm_cards::trinisphere(), 1);
            put_on_battlefield(&mut game, phase_cm_cards::goblin_electromancer(), 0);
            game.battlefield.get_mut(&sphere).unwrap().tapped = tapped;
            game
        }
    };
    // {R} less {1} is {R}; then three mana.
    assert_costs_exactly(board(false), bolt, &[(ManaType::Red, 3)], ManaType::Red, "Bolt under Trinisphere");
    // Tapped, the clause is off and the spell costs what the reduction left.
    assert_costs_exactly(board(true), bolt, &[(ManaType::Red, 1)], ManaType::Red, "Bolt under a tapped Trinisphere");
}

// ---------------------------------------------------------------------------
// The order's arithmetic: one component, the floor, the alternative cost
// ---------------------------------------------------------------------------

/// Base {3}{R}, plus the kicker's {2}, plus Thalia's {1}: {6}{R}, paid as one
/// mana component. Until CM-1 the kicker was a second `Cost::Mana` that the
/// generic split and the mana window both ignored.
// COVERS: ATOM-601.2f-001
#[test]
fn test_a_kicked_spell_under_a_tax_pays_one_mana_component() {
    assert_costs_exactly(
        || thalia_board(0),
        phase_cm_cards::kicked_lesson,
        &[(ManaType::Red, 7)],
        ManaType::Red,
        "kicked {3}{R} under Thalia",
    );
}

/// {1}{G} under "{1} less", "{1} less" and "{G} less": the generic goes to
/// nothing and stays there, the green pip goes, and the spell is cast from an
/// empty pool — "considered to be {0}", and free.
// COVERS: ATOM-601.2f-002, ATOM-118.7-002
#[test]
fn test_reductions_floor_the_component_at_zero_and_a_zero_spell_is_free() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase_cm_cards::generic_reducer(), 0);
    put_on_battlefield(&mut game, phase_cm_cards::generic_reducer(), 0);
    put_on_battlefield(&mut game, phase_cm_cards::green_reducer(), 0);
    let dp = RecordingDecisionProvider::picking(0);
    let cast = cast_from_pool(&mut game, 0, phase_cm_cards::verdant_lesson(), &[], &dp);
    assert!(cast.is_ok(), "free: {cast:?}");
    assert_eq!(game.players[0].mana_pool.total(), 0);
    // Three reductions is "multiple", so the caster was asked to order them.
    assert!(
        dp.kinds().iter().any(|k| k.starts_with("OrderCostReductions")),
        "CR 601.2f's order was the caster's to choose: {:?}",
        dp.kinds()
    );
}

/// CR 118.9d — the modifications apply to the alternative cost that was
/// chosen: "pay {R} rather than pay this spell's mana cost", under Thalia,
/// is {1}{R}.
// COVERS: ATOM-118.9d-001
#[test]
fn test_a_chosen_alternative_cost_is_modified_too() {
    let mut game = thalia_board(0);
    let card = phase_cm_cards::bargain_lesson();
    let dp = ScriptedDecisionProvider::new();
    // Options are [the normal cost, the alternative]; take the alternative.
    dp.expect_pick_n(ChoiceKind::ChooseAlternativeCost, vec![1]);
    dp.expect_allocation(ChoiceKind::GenericManaAllocation { mana_cost: ManaCost::zero() }, vec![1]);
    let cast = cast_from_pool(&mut game, 0, card, &[(ManaType::Red, 2)], &dp);
    assert!(cast.is_ok(), "{cast:?}");
    assert_eq!(game.players[0].mana_pool.total(), 0, "{{1}}{{R}}: the alternative plus the tax");
}

// ---------------------------------------------------------------------------
// "In any order they choose"
// ---------------------------------------------------------------------------

/// A `DecisionProvider` that records the options it was asked to order and
/// answers with a scripted permutation.
struct OrderingDp {
    order: Vec<usize>,
    offered: RefCell<Vec<Vec<ChoiceOption>>>,
}

impl DecisionProvider for OrderingDp {
    fn pick_n(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, _: &[ChoiceOption], _: (usize, usize)) -> Vec<usize> {
        vec![0]
    }
    fn pick_number(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, min: u64, _: u64) -> u64 {
        min
    }
    fn allocate(&self, _: &GameState, _: PlayerId, _: &ChoiceContext, total: u64, buckets: &[ChoiceOption], _: &[u64], _: Option<&[u64]>) -> Vec<u64> {
        let mut out = vec![0; buckets.len()];
        out[0] = total;
        out
    }
    fn choose_ordering(&self, _: &GameState, _: PlayerId, ctx: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        assert!(matches!(ctx.kind, ChoiceKind::OrderCostReductions { .. }), "{:?}", ctx.kind);
        self.offered.borrow_mut().push(items.to_vec());
        self.order.clone()
    }
}

/// Two different reductions on one red instant — Electromancer's {1} and the
/// Red Reducer's {R} on {1}{R}{R}: the caster is asked which applies first,
/// the sources are offered in timestamp order, both apply, and every order
/// leaves {R} (`cost-architecture.md` §3.4's theorem, checked the other way).
// COVERS: ATOM-601.2f-004
#[test]
fn test_two_reductions_are_ordered_by_the_caster_and_every_order_agrees() {
    for order in [vec![0, 1], vec![1, 0]] {
        let mut game = setup_two_player_game();
        let electromancer = put_on_battlefield(&mut game, phase_cm_cards::goblin_electromancer(), 0);
        let reducer = put_on_battlefield(&mut game, phase_cm_cards::red_reducer(), 0);
        let dp = OrderingDp { order: order.clone(), offered: RefCell::new(Vec::new()) };
        let cast = cast_from_pool(&mut game, 0, phase_cm_cards::crimson_lesson(), &[(ManaType::Red, 1)], &dp);
        assert!(cast.is_ok(), "order {order:?}: {cast:?}");
        assert_eq!(game.players[0].mana_pool.total(), 0, "order {order:?}: {{R}} either way");
        let offered = dp.offered.borrow();
        assert_eq!(offered.len(), 1, "asked exactly once");
        let ids: Vec<ObjectId> = offered[0]
            .iter()
            .map(|o| match o {
                ChoiceOption::Object(id) => *id,
                other => panic!("a source is an object: {other:?}"),
            })
            .collect();
        assert_eq!(ids, vec![electromancer, reducer], "the sources, oldest first");
    }
    // One reduction is not "multiple": nothing is asked.
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase_cm_cards::goblin_electromancer(), 0);
    let dp = RecordingDecisionProvider::picking(0);
    cast_from_pool(&mut game, 0, phase_cm_cards::crimson_lesson(), &[(ManaType::Red, 2)], &dp).unwrap();
    assert!(!dp.kinds().iter().any(|k| k.starts_with("OrderCostReductions")), "{:?}", dp.kinds());
}

// ---------------------------------------------------------------------------
// Locked in
// ---------------------------------------------------------------------------

/// The Locked Sphere is Trinisphere's clause over a "{T}: Add {C}". A {1}
/// artifact is locked at {3} at CR 601.2f; in 601.2g's window the sphere is
/// tapped for the third mana, which turns its clause off; and 601.2h pays
/// the three that were locked, not the one the board now says. Trinisphere's
/// own ruling — "becomes tapped … as a cost to cast a spell, this cost is
/// paid after you've locked in the total cost" — through the mana window.
// COVERS-PARTIAL: ATOM-601.2f-003
#[test]
fn test_the_total_is_locked_before_the_mana_window_opens() {
    let mut game = setup_two_player_game();
    let sphere = put_on_battlefield(&mut game, phase_cm_cards::locked_sphere(), 0);
    let dp = RecordingDecisionProvider::picking(0);
    let cast = cast_from_pool(&mut game, 0, tin_trinket(), &[(ManaType::Colorless, 2)], &dp);
    assert!(cast.is_ok(), "{cast:?}");
    assert!(game.battlefield.get(&sphere).unwrap().tapped, "tapped for mana in the window");
    assert_eq!(game.players[0].mana_pool.total(), 0, "three paid, the locked total");
    assert!(
        dp.kinds().iter().any(|k| k.starts_with("ManaAbilityWindow")),
        "the window is where the sphere was tapped: {:?}",
        dp.kinds()
    );

    // The control: with the sphere already tapped, the same artifact is {1}.
    let mut game = setup_two_player_game();
    let sphere = put_on_battlefield(&mut game, phase_cm_cards::locked_sphere(), 0);
    game.battlefield.get_mut(&sphere).unwrap().tapped = true;
    let cast = cast_from_pool(&mut game, 0, tin_trinket(), &[(ManaType::Colorless, 1)], &RecordingDecisionProvider::picking(0));
    assert!(cast.is_ok(), "{cast:?}");
}

// ---------------------------------------------------------------------------
// CR 613.11: read after every layer, off the effective list
// ---------------------------------------------------------------------------

/// Humility strips Thalia's ability at layer 6, and CR 613.11 reads the cost
/// effect after that: a Bolt under both costs {R}. The registry was never
/// consulted — `cost_modification_ability_sources` still holds Thalia, and
/// the gate's over-approximation costs a walk, never an answer.
// COVERS-PARTIAL: ATOM-613.11-001
#[test]
fn test_thalia_under_humility_stops_taxing() {
    let board = || {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, phase_cm_cards::thalia_guardian_of_thraben(), 0);
        put_on_battlefield(&mut game, phase_lf_cards::humility(), 1);
        game
    };
    assert_costs_exactly(board, bolt, &[(ManaType::Red, 1)], ManaType::Red, "Bolt under Thalia under Humility");
    let game = board();
    assert_eq!(game.cost_modification_ability_sources.len(), 1, "the set is a gate, not an answer");
}

/// The granted leg of the gate: a Layer 6 row gives a vanilla creature
/// Thalia's text, nothing printed it, and the tax applies.
#[test]
fn test_a_granted_cost_effect_is_found_through_the_registry_summary() {
    let board = || {
        let mut game = setup_two_player_game();
        let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
        let tax = static_ability(Effect::CostModification(Box::new(CostModificationDef::spells(
            ObjectFilter::Not(Box::new(ObjectFilter::ByType(CardType::Creature))),
            CostChange::Increase(ManaCost::build(&[], 1)),
        ))));
        let ts = game.next_timestamp;
        game.next_timestamp += 1;
        game.continuous_effects.add(registered(
            bears,
            Layer::Layer6Ability,
            ts,
            EffectModification::GrantAbility(Box::new(tax)),
        ));
        assert!(game.cost_modification_ability_sources.is_empty(), "nothing printed one");
        assert!(game.continuous_effects.summary().any_granted_cost_modification);
        game
    };
    assert_costs_exactly(board, bolt, &[(ManaType::Red, 2)], ManaType::Red, "Bolt under a granted tax");
}

/// A source that leaves the battlefield leaves the gate too.
#[test]
fn test_a_cost_effect_source_that_leaves_stops_applying() {
    let mut game = thalia_board(0);
    let thalia = game.battlefield.keys().copied().next().unwrap();
    game.change_zone(thalia, Zone::Graveyard, ZoneChangeCause::Destroyed, &test_ctx()).unwrap();
    assert!(game.cost_modification_ability_sources.is_empty());
    let cast = cast_from_pool(&mut game, 0, bolt(), &[(ManaType::Red, 1)], &RecordingDecisionProvider::picking(0));
    assert!(cast.is_ok(), "{cast:?}");
}

// ---------------------------------------------------------------------------
// Enumeration agrees with enforcement (§3.6)
// ---------------------------------------------------------------------------

/// `castable_spells` reads the cost the cast will lock in: a Bolt under Thalia
/// is not offered from a pool of {R}, and a {1}{R} instant under Electromancer
/// is. Before CM-1 the first was offered and rolled back, and the second was
/// withheld.
#[test]
fn test_castable_spells_reads_the_modified_cost() {
    let mut game = thalia_board(0);
    let bolt_id = put_in_hand(&mut game, bolt(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);
    assert!(
        !castable_spells(&game, 0).iter().any(|(id, _)| *id == bolt_id),
        "{{1}}{{R}} is not payable from {{R}}"
    );
    game.players[0].mana_pool.add(ManaType::Red, 1);
    assert!(castable_spells(&game, 0).iter().any(|(id, _)| *id == bolt_id));

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase_cm_cards::goblin_electromancer(), 0);
    let lesson = put_in_hand(&mut game, phase_cm_cards::ember_lesson(), 0);
    game.players[0].mana_pool.add(ManaType::Red, 1);
    assert!(
        castable_spells(&game, 0).iter().any(|(id, _)| *id == lesson),
        "{{1}}{{R}} less {{1}} is payable from {{R}}"
    );
}

/// Every registered card of the phase lowers, enters and leaves cleanly for
/// either controller — the pool test's guarantee, asserted here by name so
/// the cards are exercised even if the registry walk is ever narrowed.
#[test]
fn test_the_phases_cards_are_registered() {
    let registry = CardRegistry::default_registry();
    for name in ["Thalia, Guardian of Thraben", "Goblin Electromancer", "Trinisphere"] {
        let card = registry.create(name).unwrap_or_else(|e| panic!("{name}: {e}"));
        let mut game = setup_two_player_game();
        let id = put_on_battlefield(&mut game, card, 1);
        assert!(game.cost_modification_ability_sources.contains(&id), "{name} is a source");
    }
    // CM-2's two are not: a spell's own cost ability functions on the stack
    // (CR 113.6d), so an affinity creature standing on the battlefield is a
    // source of nothing and must not widen the sweep on every cast.
    for name in ["Myr Enforcer", "Frogmite"] {
        let card = registry.create(name).unwrap_or_else(|e| panic!("{name}: {e}"));
        let mut game = setup_two_player_game();
        let id = put_on_battlefield(&mut game, card, 1);
        assert!(!game.cost_modification_ability_sources.contains(&id), "{name} is not a source");
        assert!(game.cost_modification_ability_sources.is_empty(), "{name} left the set empty");
    }
}

// ---------------------------------------------------------------------------
// CM-2 — the spell's own cost abilities (CR 113.6d, 702.41a)
// ---------------------------------------------------------------------------

/// `n` artifacts on `player`'s battlefield, each with no text of its own, so
/// the only thing they contribute is being counted.
fn artifacts(game: &mut GameState, n: usize, player: PlayerId) {
    for _ in 0..n {
        put_on_battlefield(game, tin_trinket(), player);
    }
}

fn artifact_board(n: usize, player: PlayerId) -> GameState {
    let mut game = setup_two_player_game();
    artifacts(&mut game, n, player);
    game
}

/// CR 702.41a — "This spell costs {1} less to cast for each [text] you
/// control". Myr Enforcer is {7} and nothing else, so what it is castable
/// for *is* the count, and the reduction comes off generic (CR 118.7a).
///
/// The last board is CR 601.2f's "reduced to nothing … considered to be
/// {0}": seven artifacts and the Enforcer is free.
// COVERS: ATOM-702.41a-001
#[test]
fn test_myr_enforcer_costs_one_less_for_each_artifact_you_control() {
    for (owned, pay) in [(0, 7), (1, 6), (4, 3)] {
        assert_costs_exactly(
            move || artifact_board(owned, 0),
            phase_cm_cards::myr_enforcer,
            &[(ManaType::Colorless, pay)],
            ManaType::Colorless,
            &format!("{owned} artifacts"),
        );
    }
    // "You control" — an opponent's artifacts are not yours to count.
    assert_costs_exactly(
        || artifact_board(4, 1),
        phase_cm_cards::myr_enforcer,
        &[(ManaType::Colorless, 7)],
        ManaType::Colorless,
        "their artifacts",
    );

    // Seven artifacts: the component is reduced to nothing, and an empty
    // pool pays it.
    let mut game = artifact_board(7, 0);
    let cast = cast_from_pool(&mut game, 0, phase_cm_cards::myr_enforcer(), &[], &RecordingDecisionProvider::picking(0));
    assert!(cast.is_ok(), "seven artifacts is {{0}}: {cast:?}");
}

/// The same ability on a second printed card, and the board two copies of one
/// card reach in a real deck: the first Enforcer is an artifact, so the
/// second one counts it. Frogmite's {4} is reduced by the pair.
#[test]
fn test_a_resolved_affinity_creature_counts_for_the_next_one() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase_cm_cards::myr_enforcer(), 0);
    let cast = cast_from_pool(&mut game, 0, phase_cm_cards::myr_enforcer(), &[(ManaType::Colorless, 6)], &RecordingDecisionProvider::picking(0));
    assert!(cast.is_ok(), "one Enforcer out: the next is {{6}}: {cast:?}");

    assert_costs_exactly(
        || {
            let mut game = setup_two_player_game();
            put_on_battlefield(&mut game, phase_cm_cards::myr_enforcer(), 0);
            put_on_battlefield(&mut game, phase_cm_cards::frogmite(), 0);
            game
        },
        phase_cm_cards::frogmite,
        &[(ManaType::Colorless, 2)],
        ManaType::Colorless,
        "Frogmite behind an Enforcer and a Frogmite",
    );
}

/// A reduction from the battlefield and the spell's own meet on one spell, so
/// CR 601.2f's "if multiple cost reductions apply" is asked — with the
/// *spell* as one of the candidates, offered last (`cost-architecture.md`
/// §4). Both orders leave {4}: two artifacts is {2} off, the Generic Reducer
/// is {1} more off, and generic subtraction commutes.
#[test]
fn test_affinity_joins_the_ordering_prompt_and_is_offered_last() {
    for order in [vec![0, 1], vec![1, 0]] {
        let mut game = setup_two_player_game();
        let reducer = put_on_battlefield(&mut game, phase_cm_cards::generic_reducer(), 0);
        artifacts(&mut game, 2, 0);
        let dp = OrderingDp { order: order.clone(), offered: RefCell::new(Vec::new()) };
        let spell = cast_from_pool(&mut game, 0, phase_cm_cards::myr_enforcer(), &[(ManaType::Colorless, 4)], &dp);
        let spell = spell.unwrap_or_else(|e| panic!("order {order:?}: {e}"));
        assert_eq!(game.players[0].mana_pool.total(), 0, "order {order:?}: {{4}} either way");
        let offered = dp.offered.borrow();
        assert_eq!(offered.len(), 1, "asked exactly once");
        let ids: Vec<ObjectId> = offered[0]
            .iter()
            .map(|o| match o {
                ChoiceOption::Object(id) => *id,
                other => panic!("a source is an object: {other:?}"),
            })
            .collect();
        assert_eq!(ids, vec![reducer, spell], "the battlefield's source first, the spell's own last");
    }
}

/// CR 601.2f's positions 2 and 3 on one spell: affinity takes the Enforcer
/// below three, and Trinisphere puts it back. Trinisphere is itself an
/// artifact, so it is one of the five the reduction counts.
#[test]
fn test_trinisphere_applies_after_affinity_has_reduced() {
    let board = || {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, phase_cm_cards::trinisphere(), 0);
        artifacts(&mut game, 4, 0);
        game
    };
    // {7} less five artifacts is {2}; "each spell that would cost less than
    // three mana to cast costs three mana to cast".
    assert_costs_exactly(
        board,
        phase_cm_cards::myr_enforcer,
        &[(ManaType::Colorless, 3)],
        ManaType::Colorless,
        "affinity under Trinisphere",
    );
}

/// Enumeration agrees with enforcement for source 2 as well (§3.6): the
/// castability preview reads the card's own ability list in hand, so an
/// Enforcer the board has made affordable is offered. It is not a claim that
/// the ability *functions* in hand — CR 113.6d and CR 702.41a both put it on
/// the stack — it is the preview answering what the cast will lock in.
#[test]
fn test_castable_spells_offers_an_enforcer_the_board_made_affordable() {
    let mut game = artifact_board(4, 0);
    let enforcer = put_in_hand(&mut game, phase_cm_cards::myr_enforcer(), 0);
    game.players[0].mana_pool.add(ManaType::Colorless, 3);
    assert!(
        castable_spells(&game, 0).iter().any(|(id, _)| *id == enforcer),
        "{{7}} less four artifacts is {{3}}, and {{3}} is in the pool"
    );

    let mut game = setup_two_player_game();
    let enforcer = put_in_hand(&mut game, phase_cm_cards::myr_enforcer(), 0);
    game.players[0].mana_pool.add(ManaType::Colorless, 3);
    assert!(
        !castable_spells(&game, 0).iter().any(|(id, _)| *id == enforcer),
        "no artifacts: {{7}} is not payable from {{3}}"
    );
}

/// `CostChange::ReduceGeneric(SourcePower)` — Golden-Tail Trainer's shape, on
/// a fixture, because the printed card's other half is an attack trigger.
///
/// The claim is `cost-architecture.md` §3.7's, stated as a board: the amount
/// is read off the source's **effective** frame at CR 601.2f, so an anthem on
/// the reducer makes it reduce more. Nothing about that is available to the
/// layer walk, which is handed an affected object and refuses the leaf.
#[test]
fn test_a_reduction_by_source_power_reads_the_effective_frame() {
    let board = |anthem: bool| {
        move || {
            let mut game = setup_two_player_game();
            put_on_battlefield(&mut game, phase_cm_cards::power_reducer(), 0);
            if anthem {
                put_on_battlefield(&mut game, mtgsim::cards::phase5_pre_cards::glorious_anthem(), 0);
            }
            game
        }
    };
    // A 2/2 reducer takes {4} to {2}.
    assert_costs_exactly(
        board(false),
        || tin_trinket_costing(4),
        &[(ManaType::Colorless, 2)],
        ManaType::Colorless,
        "a 2/2 reducer",
    );
    // The anthem makes it a 3/3, and the reduction follows.
    assert_costs_exactly(
        board(true),
        || tin_trinket_costing(4),
        &[(ManaType::Colorless, 1)],
        ManaType::Colorless,
        "a 3/3 reducer under its own anthem",
    );
}


// ---------------------------------------------------------------------------
// CM-3 — lock-in's payment side (CR 601.2h, 118.8b, 732.1)
// ---------------------------------------------------------------------------

/// The board CR 601.2h's own example is written on.
fn familiar_board() -> GameState {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase_cm_cards::thunderscape_familiar(), 0);
    game
}

/// The ability index of a permanent's `n`th effective ability.
fn ability_id_at(game: &GameState, id: ObjectId, index: usize) -> mtgsim::types::ids::AbilityId {
    mtgsim::oracle::characteristics::get_effective_abilities(game, id)[index].id
}

/// CR 601.2h's worked example, verbatim: "You cast Altar's Reap, which costs
/// {1}{B} and has an additional cost of sacrificing a creature. You sacrifice
/// Thunderscape Familiar, whose effect makes your black spells cost {1} less
/// to cast. Because a spell's total cost is 'locked in' before payments are
/// actually made, you pay {B}, not {1}{B}, even though you're sacrificing the
/// Familiar."
///
/// The exact pool is the whole assertion: {B} pays for it, and the Familiar
/// is gone by the time the payment finishes.
// COVERS: ATOM-601.2h-001
#[test]
fn test_altars_reap_pays_the_cost_the_creature_it_sacrifices_reduced() {
    assert_costs_exactly(
        familiar_board,
        phase_cm_cards::altars_reap,
        &[(ManaType::Black, 1)],
        ManaType::Black,
        "Altar's Reap under its own sacrifice",
    );

    // ...and the Familiar is what paid for it.
    let mut game = familiar_board();
    let familiar = *game.battlefield.keys().next().unwrap();
    let dp = RecordingDecisionProvider::picking(0);
    let reap = cast_from_pool(
        &mut game, 0, phase_cm_cards::altars_reap(), &[(ManaType::Black, 1)], &dp,
    )
    .expect("castable for {B}");

    assert!(game.stack.contains(&reap), "the spell is on the stack");
    assert!(!game.battlefield.contains_key(&familiar), "the Familiar was sacrificed");
    assert!(game.players[0].graveyard.contains(&familiar));
    // One creature to sacrifice is a forced payment, so nobody was asked.
    assert_eq!(dp.prompts(), 0, "asked {:?}", dp.kinds());
}

/// CR 118.8d — an additional cost does not change the spell's mana cost.
/// Altar's Reap sacrifices a creature and is still a two-drop.
// COVERS: ATOM-118.8d-001
#[test]
fn test_a_mandatory_additional_cost_does_not_change_the_mana_cost() {
    let mut game = familiar_board();
    let reap = cast_from_pool(
        &mut game, 0, phase_cm_cards::altars_reap(), &[(ManaType::Black, 1)],
        &RecordingDecisionProvider::picking(0),
    )
    .expect("castable for {B}");

    let printed = game.get_object(reap).unwrap().card_data.mana_cost.clone().unwrap();
    assert_eq!(printed.mana_value(), 2, "{{1}}{{B}} is mana value 2, sacrifice or no");
}

/// CR 118.8b/118.8c — a mandatory additional cost is not announced, because
/// there is no intention to declare. The optional-cost prompt is not asked at
/// all when every additional cost the card has is mandatory.
#[test]
fn test_a_mandatory_additional_cost_is_not_offered() {
    let mut game = familiar_board();
    let dp = RecordingDecisionProvider::picking(0);
    cast_from_pool(&mut game, 0, phase_cm_cards::altars_reap(), &[(ManaType::Black, 1)], &dp)
        .expect("castable");
    assert!(
        !dp.kinds().iter().any(|k| k.starts_with("ChooseAdditionalCosts")),
        "a mandatory cost was offered as a choice: {:?}", dp.kinds(),
    );
}

/// Two creatures is a choice, and the payment takes the one chosen.
#[test]
fn test_sacrificing_for_a_cost_is_the_payers_choice() {
    let mut game = familiar_board();
    let familiar = *game.battlefield.keys().next().unwrap();
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    let dp = ScriptedDecisionProvider::new();
    // Candidates are in battlefield timestamp order: the Familiar, then the
    // bear. Index 1 keeps the reducer on the board.
    dp.expect_pick_n(
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id: familiar, count: 1 },
        vec![1],
    );
    let reap = put_in_hand(&mut game, phase_cm_cards::altars_reap(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.cast_spell(0, reap, &dp).expect("castable for {B}");

    assert!(game.battlefield.contains_key(&familiar), "the reducer was not the one chosen");
    assert!(!game.battlefield.contains_key(&bear), "the bear was");
}

/// CR 601.2h — "unpayable costs can't be paid", and `cost-architecture.md`
/// §3.6 — enumeration and enforcement must agree. Altar's Reap with no
/// creature is not offered by the priority loop *and* not castable, and the
/// failed attempt spends nothing.
#[test]
fn test_altars_reap_is_uncastable_and_unoffered_with_no_creature() {
    let mut game = setup_two_player_game();
    let reap = put_in_hand(&mut game, phase_cm_cards::altars_reap(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);

    let offered = castable_spells(&game, 0);
    assert!(
        !offered.iter().any(|(id, _)| *id == reap),
        "offered a spell whose mandatory cost cannot be paid",
    );

    let attempt = game.cast_spell(0, reap, &RecordingDecisionProvider::picking(0));
    assert!(attempt.is_err(), "cast succeeded with nothing to sacrifice");
    assert!(game.players[0].hand.contains(&reap), "the card is back in hand");
    assert_eq!(game.players[0].mana_pool.total(), 2, "nothing was paid");
}

/// `cost-architecture.md` §3.11 steps 3–4, and the completion of
/// `ATOM-601.2f-003`: the reducer leaves the battlefield *inside* CR 601.2g's
/// window, after the total was locked and before it was paid.
///
/// Foundry Inspector makes Mind Stone cost {1}. Krark-Clan Ironworks then eats
/// the Inspector for {C}{C} while the cast is still in its mana window. The
/// engine pays {1} — the total nothing re-read — and one colorless is left
/// over. A cost recomputed after the window would be {2} and would leave none,
/// which is what makes the leftover mana the assertion.
// COVERS: ATOM-601.2f-003
#[test]
fn test_a_reducer_eaten_inside_the_mana_window_does_not_raise_the_locked_cost() {
    let mut game = setup_two_player_game();
    let inspector = put_on_battlefield(&mut game, phase_cm_cards::foundry_inspector(), 0);
    let ironworks = put_on_battlefield(&mut game, phase_cm_cards::krark_clan_ironworks(), 0);
    let stone = put_in_hand(&mut game, phase_cm_cards::mind_stone(), 0);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::ManaAbilityWindow {
            spell_or_ability_id: stone,
            remaining_cost: ManaCost::build(&[], 1),
        },
        vec![0],
    );
    dp.expect_pick_n(
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id: ironworks, count: 1 },
        vec![0],
    );
    dp.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: ManaCost::build(&[], 1) },
        vec![1],
    );

    game.cast_spell(0, stone, &dp).expect("Mind Stone costs {1} under the Inspector");

    assert!(!game.battlefield.contains_key(&inspector), "the Inspector paid for the mana");
    assert_eq!(
        game.players[0].mana_pool.amount(ManaType::Colorless), 1,
        "{{1}} was paid, not {{2}}: the total was locked before the window",
    );
    assert!(game.stack.contains(&stone));
}

/// `cost-architecture.md` §3.11 step 3 — "Ironworks to itself". The filter is
/// "an artifact", the source is an artifact, and nothing about paying a cost
/// excludes the permanent whose cost it is.
#[test]
fn test_ironworks_pays_its_own_cost_with_itself() {
    let mut game = setup_two_player_game();
    let ironworks = put_on_battlefield(&mut game, phase_cm_cards::krark_clan_ironworks(), 0);
    let ability = ability_id_at(&game, ironworks, 0);

    // The only artifact is the source, so the payment is forced.
    let dp = ScriptedDecisionProvider::new();
    game.activate_mana_ability(0, ironworks, ability, &mtgsim::engine::actions::ActionContext::new(&dp))
        .expect("Ironworks can eat itself");

    assert!(!game.battlefield.contains_key(&ironworks));
    assert!(game.players[0].graveyard.contains(&ironworks));
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Colorless), 2);
}

/// The CR 732.1 board (`cost-architecture.md` §3.11): Mind Stone's activation
/// has its own source sacrificed in its mana window and cannot pay its cost.
///
/// **The engine cancels nothing, and needs to cancel nothing.** The cost is
/// checked before any of it is paid, so the activation rewinds with no
/// payment made — 732.1's first sentence has an empty set to act on. The
/// Ironworks activation was legal when it happened, so it stands: the mana is
/// in the pool and Mind Stone is in the graveyard. Offering 732.1's
/// *reversal* of that mana ability is `codebase-state.md` item 72's, and
/// where the resulting trigger goes on the stack is critical-path item 6's —
/// this asserts only what both readings of that question share.
#[test]
fn test_an_activation_whose_source_is_sacrificed_rewinds_and_the_mana_stands() {
    let mut game = setup_two_player_game();
    let ironworks = put_on_battlefield(&mut game, phase_cm_cards::krark_clan_ironworks(), 0);
    let stone = put_on_battlefield(&mut game, phase_cm_cards::mind_stone(), 0);
    let hand_before = game.players[0].hand.len();

    let dp = ScriptedDecisionProvider::new();
    // In the window: activate Ironworks (offered first, by timestamp)...
    dp.expect_pick_n(
        ChoiceKind::ManaAbilityWindow {
            spell_or_ability_id: stone,
            remaining_cost: ManaCost::build(&[], 1),
        },
        vec![0],
    );
    // ...paying it with Mind Stone, the source of the pending activation.
    dp.expect_pick_n(
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id: ironworks, count: 1 },
        vec![1],
    );
    // The window offers again — Ironworks can still eat itself — and stops.
    dp.expect_pick_n(
        ChoiceKind::ManaAbilityWindow {
            spell_or_ability_id: stone,
            remaining_cost: ManaCost::build(&[], 0),
        },
        vec![],
    );

    // Ability 1 is "{1}, {T}, Sacrifice this artifact: Draw a card"; ability 0
    // is the mana ability.
    let attempt = game.activate_ability(0, stone, 1, &dp);

    assert!(attempt.is_err(), "the activation cannot pay a cost its own source owed");
    assert!(game.stack.is_empty(), "the ability object is off the stack");
    assert_eq!(game.players[0].hand.len(), hand_before, "no card was drawn");
    assert_eq!(
        game.players[0].mana_pool.amount(ManaType::Colorless), 2,
        "the mana ability was legal when activated, and CR 732.1 leaves it standing",
    );
    assert!(game.players[0].graveyard.contains(&stone), "and so does its cost");
    assert!(game.battlefield.contains_key(&ironworks));
}

/// The order CR 601.2h leaves to the player, and the engine picks: an
/// object-moving cost is paid last, so a cost list that prints the sacrifice
/// first does not eat its own source and then fail the tap.
///
/// Paid as printed, this activation would sacrifice the Engine and then find
/// nothing to tap — a payment already made, with nothing in the engine that
/// could cancel it (CR 732.1). `payment_order_rank` is what makes that
/// unreachable, and this fixture is the only board on which it is visible:
/// every printed card puts its mana and tap first by convention.
#[test]
fn test_an_object_moving_cost_is_paid_after_the_tap_it_would_break() {
    let mut game = setup_two_player_game();
    let engine = put_on_battlefield(&mut game, phase_cm_cards::self_eating_engine(), 0);

    let dp = ScriptedDecisionProvider::new();
    game.activate_ability(0, engine, 0, &dp)
        .expect("the tap is paid before the sacrifice that would break it");

    assert!(!game.battlefield.contains_key(&engine), "the sacrifice was paid too");
    assert!(game.players[0].graveyard.contains(&engine));
    assert_eq!(game.stack.len(), 1, "the ability is on the stack, fully paid");
}

/// One cost that takes two permanents takes them as one event (CR 601.2h pays
/// a cost, not a list of them), which is what a "whenever one or more
/// creatures die" trigger will read.
#[test]
fn test_one_cost_taking_two_creatures_is_one_event() {
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let c = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    let dp = ScriptedDecisionProvider::new();
    // The plan takes the mana split first, then the sacrifices — one pass
    // over the ordered costs, and the mana component is first in it.
    dp.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: ManaCost::build(&[ManaType::Black], 2) },
        // Buckets sorted by discriminant: Black, Colorless. The Black is owed
        // to the pip, so both generic come from the colorless.
        vec![0, 2],
    );
    dp.expect_pick_n(
        ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id: a, count: 2 },
        vec![0, 1],
    );

    let offering = put_in_hand(&mut game, phase_cm_cards::twin_offering(), 0);
    game.players[0].mana_pool.add(ManaType::Black, 1);
    game.players[0].mana_pool.add(ManaType::Colorless, 2);
    let before = game.events.len();
    game.cast_spell(0, offering, &dp).expect("castable for {2}{B} with two creatures");

    assert!(!game.battlefield.contains_key(&a));
    assert!(!game.battlefield.contains_key(&b));
    assert!(game.battlefield.contains_key(&c), "only the two chosen");

    let batches: std::collections::HashSet<_> = game.events.records_from(before)
        .iter()
        .filter(|r| matches!(
            r.event,
            mtgsim::events::event::GameEvent::ZoneChange { cause: ZoneChangeCause::Sacrificed, .. },
        ))
        .map(|r| r.batch())
        .collect();
    assert_eq!(batches.len(), 1, "two sacrifices for one cost are one event");
}

// ---------------------------------------------------------------------------
// The rulings pass — CM-3's five cards, read on Scryfall 2026-09-08
//
// `engineering-practices.md` §3.4: a card's rulings are the cheapest source of
// boards the corpus does not have, because they were written about the cases
// players got wrong. What follows is every ruling on the five that this engine
// can answer today; the ones it cannot are listed there with the reason.
// ---------------------------------------------------------------------------

/// A coloured [`tin_trinket`]: a permanent spell needs no spell ability, so
/// the cost is the whole card and nothing else can move it.
fn colored_trinket(name: &str, cost: ManaCost, colors: &[mtgsim::types::colors::Color]) -> Arc<CardData> {
    let mut b = CardDataBuilder::new(name).mana_cost(cost).card_type(CardType::Artifact);
    for c in colors {
        b = b.color(*c);
    }
    b.build()
}

/// {2}{B}{G}, for the Familiar's "both black and green" ruling.
fn golgari_trinket() -> Arc<CardData> {
    colored_trinket(
        "Golgari Trinket",
        ManaCost::build(&[ManaType::Black, ManaType::Green], 2),
        &[mtgsim::types::colors::Color::Black, mtgsim::types::colors::Color::Green],
    )
}

/// {B}{B}: a black spell with no generic to give.
fn double_black_trinket() -> Arc<CardData> {
    colored_trinket(
        "Double Black Trinket",
        ManaCost::build(&[ManaType::Black, ManaType::Black], 0),
        &[mtgsim::types::colors::Color::Black],
    )
}

/// {2}{B}, for the cumulative ruling.
fn sable_trinket() -> Arc<CardData> {
    colored_trinket(
        "Sable Trinket",
        ManaCost::build(&[ManaType::Black], 2),
        &[mtgsim::types::colors::Color::Black],
    )
}

/// An {X} artifact, for Foundry Inspector's X ruling.
fn x_trinket() -> Arc<CardData> {
    CardDataBuilder::new("X Trinket")
        .mana_cost(ManaCost::from_symbols(vec![mtgsim::types::mana::ManaSymbol::X]))
        .card_type(CardType::Artifact)
        .build()
}

/// Thunderscape Familiar, 2004-10-04: "If a spell is both black and green, you
/// pay {1} less, not {2} less."
///
/// The filter is one `Or` in one ability, so the gather returns one instance
/// and the reduction applies once. Written as two colour leaves on two
/// abilities it would apply twice, and this board is the only one that says so.
#[test]
fn test_the_familiar_reduces_a_black_and_green_spell_once() {
    assert_costs_exactly(
        familiar_board,
        golgari_trinket,
        &[(ManaType::Black, 1), (ManaType::Green, 1), (ManaType::Colorless, 1)],
        ManaType::Colorless,
        "a black-and-green spell under one Familiar",
    );
}

/// Thunderscape Familiar, 2004-10-04: "The effect is cumulative." Two
/// Familiars make a black spell cost {2} less — and, because two reductions
/// apply, CR 601.2f's ordering prompt is asked.
#[test]
fn test_two_familiars_reduce_cumulatively() {
    let board = || {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, phase_cm_cards::thunderscape_familiar(), 0);
        put_on_battlefield(&mut game, phase_cm_cards::thunderscape_familiar(), 0);
        game
    };
    // {2}{B} under two Familiars is {B}.
    assert_costs_exactly(
        board,
        sable_trinket,
        &[(ManaType::Black, 1)],
        ManaType::Black,
        "{2}{B} under two Familiars",
    );

    let mut game = board();
    let dp = RecordingDecisionProvider::picking(0);
    cast_from_pool(
        &mut game, 0,
        sable_trinket(),
        &[(ManaType::Black, 1)], &dp,
    )
    .unwrap();
    assert!(
        dp.kinds().iter().any(|k| k.starts_with("OrderCostReductions")),
        "two reductions is a choice: {:?}", dp.kinds(),
    );
}

/// Thunderscape Familiar, 2004-10-04: "Can never affect the colored part of
/// the cost" (CR 118.7a), and "the lower cost is not optional like with some
/// other cost reducers" — the engine offers no way to decline it, so the
/// second ruling is the absence of a prompt.
#[test]
fn test_the_familiar_cannot_touch_a_colored_only_cost() {
    assert_costs_exactly(
        familiar_board,
        double_black_trinket,
        &[(ManaType::Black, 2)],
        ManaType::Black,
        "{B}{B} under the Familiar",
    );

    let mut game = familiar_board();
    let dp = RecordingDecisionProvider::picking(0);
    cast_from_pool(&mut game, 0, double_black_trinket(), &[(ManaType::Black, 2)], &dp).unwrap();
    assert_eq!(dp.prompts(), 0, "a reduction is not offered, it applies: {:?}", dp.kinds());
}

/// Altar's Reap, 2013-04-15: "You must sacrifice exactly one creature ... you
/// cannot sacrifice additional creatures."
///
/// The bound is the assertion. `picking_all` takes everything the prompt lets
/// it, so with three creatures on the board it would sacrifice three if the
/// prompt's maximum were the candidate count rather than the cost's `n`.
#[test]
fn test_altars_reap_sacrifices_exactly_one_however_many_are_offered() {
    let mut game = familiar_board();
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert_eq!(game.battlefield.len(), 3);

    let dp = RecordingDecisionProvider::picking_all();
    cast_from_pool(&mut game, 0, phase_cm_cards::altars_reap(), &[(ManaType::Black, 1)], &dp)
        .expect("castable");

    assert_eq!(game.battlefield.len(), 2, "exactly one creature paid for it");
}

/// Altar's Reap, 2013-04-15: "Players can only respond once this spell has
/// been cast and all its costs have been paid. No one can try to destroy the
/// creature you sacrificed to prevent you from casting this spell." — and
/// Foundry Inspector, 2016-09-20, says the same thing about removing the
/// Inspector before the cost is locked in.
///
/// Both are one claim about the engine: nothing yields priority between
/// CR 601.2a and 601.2i. Asserted as the absence of any priority prompt across
/// a cast that locks a cost, opens a mana window and pays a sacrifice.
#[test]
fn test_no_player_gets_priority_inside_a_cast() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase_cm_cards::thunderscape_familiar(), 0);
    put_on_battlefield(&mut game, phase_cm_cards::foundry_inspector(), 0);

    let dp = RecordingDecisionProvider::picking(0);
    cast_from_pool(&mut game, 0, phase_cm_cards::altars_reap(), &[(ManaType::Black, 1)], &dp)
        .expect("castable");

    assert!(
        !dp.kinds().iter().any(|k| k.starts_with("PriorityAction")),
        "a cast is not interruptible: {:?}", dp.kinds(),
    );
}

/// Foundry Inspector, 2016-09-20: "If an artifact spell has {X} in its mana
/// cost, choose the value for X first, and then reduce the cost by {1}. For
/// example, an artifact that costs {X} with X chosen as 4 costs {3} to cast."
///
/// The ruling's own numbers. X is announced at CR 601.2b and expanded into
/// generic before 601.2f's reduction, which is why the answer is {3} and not
/// an X of 3.
#[test]
fn test_foundry_inspector_reduces_an_x_cost_after_x_is_chosen() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, phase_cm_cards::foundry_inspector(), 0);
    let trinket = put_in_hand(&mut game, x_trinket(), 0);

    let dp = ScriptedDecisionProvider::new();
    dp.expect_number(ChoiceKind::ChooseXValue { spell_id: trinket, x_count: 1 }, 4);
    dp.expect_allocation(
        ChoiceKind::GenericManaAllocation { mana_cost: ManaCost::build(&[], 3) },
        vec![3],
    );

    game.players[0].mana_pool.add(ManaType::Colorless, 3);
    game.cast_spell(0, trinket, &dp).expect("X=4 costs {3} under the Inspector");
    assert_eq!(game.players[0].mana_pool.total(), 0, "{{3}}, the ruling's own number");
}

/// The `Or`'s **other** leaf, and the controller clause — neither of which any
/// test above reaches.
///
/// Every board so far casts a black spell, so a Familiar written
/// `Or(Black, Black)` would pass all of them: the black-and-green board matches
/// on the left leaf and never asks about the right. Green alone is what makes
/// the right leaf load-bearing. The third case is "**you** cast" (CR 109.5) —
/// the Familiar taxes nobody and reduces only its controller's spells, which is
/// what separates it from Thalia.
#[test]
fn test_the_familiar_reduces_green_too_and_only_for_its_controller() {
    let green_trinket = || {
        colored_trinket(
            "Verdant Trinket",
            ManaCost::build(&[ManaType::Green], 1),
            &[mtgsim::types::colors::Color::Green],
        )
    };

    // The right leaf: {1}{G} under the Familiar is {G}.
    assert_costs_exactly(
        familiar_board, green_trinket, &[(ManaType::Green, 1)], ManaType::Green,
        "a green spell under the Familiar",
    );

    // Neither colour: {1}{R} is untouched.
    let red_trinket = || {
        colored_trinket(
            "Vermilion Trinket",
            ManaCost::build(&[ManaType::Red], 1),
            &[mtgsim::types::colors::Color::Red],
        )
    };
    assert_costs_exactly(
        familiar_board, red_trinket, &[(ManaType::Red, 1), (ManaType::Colorless, 1)],
        ManaType::Colorless, "a red spell under the Familiar",
    );

    // "You cast": the opponent's black spell pays full price.
    let board = || {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, phase_cm_cards::thunderscape_familiar(), 1);
        game
    };
    assert_costs_exactly(
        board, || colored_trinket(
            "Sable Trinket",
            ManaCost::build(&[ManaType::Black], 2),
            &[mtgsim::types::colors::Color::Black],
        ),
        &[(ManaType::Black, 1), (ManaType::Colorless, 2)], ManaType::Colorless,
        "a black spell under an opponent's Familiar",
    );
}
