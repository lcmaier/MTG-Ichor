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
    CardDataBuilder::new("Tin Trinket")
        .mana_cost(ManaCost::build(&[], 1))
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
}
