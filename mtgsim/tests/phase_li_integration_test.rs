//! Phase LI-1 — the board-wide sequential pass
//! (`layers-architecture.md` §13b).
//!
//! What changed is *how a read is answered*: every read the walk makes of
//! another object now sees what applied earlier in the same layer, where the
//! per-object walk saw the end of the previous layer. The board the pool
//! builds is pinned in `phase_lf_integration_test.rs`
//! (`test_humility_before_hierophants_retires_the_grant`); the tests here are
//! the other things the pass changes, and two it must not.

use mtgsim::cards::{artifacts, phase_ld_cards, phase_le_cards, phase_lf_cards, phase_rc_cards};
use mtgsim::engine::layers::types::{
    AffectedSet, ContinuousEffect, EffectModification, EffectOrigin, Layer, PtValue,
};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::oracle::characteristics::{
    get_effective_power, get_effective_toughness, has_keyword, is_creature,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    card_of_type, equipment, put_in_graveyard, put_on_battlefield, setup_two_player_game,
    static_ability, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    CounterType, Duration, Effect, EffectRecipient, ObjectFilter, Primitive, SelectionFilter,
    TargetCount,
};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::keywords::KeywordFlag;

fn pt(game: &GameState, id: ObjectId) -> (Option<i32>, Option<i32>) {
    (get_effective_power(game, id), get_effective_toughness(game, id))
}

/// "Equipped creature has flying", as a static ability a spell could grant.
fn equipped_creature_has_flying() -> mtgsim::objects::card_data::AbilityDef {
    static_ability(Effect::Atom(
        Primitive::GrantKeywordFlag(KeywordFlag::Flying, Duration::WhileSourceOnBattlefield),
        EffectRecipient::Host,
    ))
}

/// Grant `ability` to `target` by resolution, from `source`, until end of turn.
fn grant(game: &mut GameState, source: ObjectId, target: ObjectId, ability: mtgsim::objects::card_data::AbilityDef) {
    let spell = Effect::Atom(
        Primitive::GrantAbility(Box::new(ability), Duration::UntilEndOfTurn),
        EffectRecipient::Target(
            SelectionFilter::Permanent(ObjectFilter::ByType(CardType::Artifact)),
            TargetCount::Exactly(1),
        ),
    );
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: 0,
        targets: vec![ResolvedTarget::Object(target)],
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(&spell, &ctx, &test_dp()).unwrap();
}

// ---------------------------------------------------------------------------
// "Before Layers" 7b's Layer 6 case — a granted static ability whose own
// effect lands in layer 6. CR 613.7a's own worked example is Rune of Flight
// granting an Equipment "Equipped creature has flying"; the grant applies at
// layer 6 and so does the effect it generates, and only a pass that lets the
// existence check see the grant applied earlier in the same layer can apply
// it. `register_granted_static_effects` asserted against this shape until
// LI-1; the shape is built here by resolution, since the printed card's draw
// is a trigger (item 6).
// ---------------------------------------------------------------------------

#[test]
fn test_a_static_ability_granted_by_resolution_applies_in_layer_6() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let wings = put_on_battlefield(&mut game, equipment("Bare Wings"), 0);
    let caster = put_on_battlefield(&mut game, phase_lf_cards::humility(), 0);
    game.continuous_effects.remove_by_source(caster);
    assert!(game.attach(wings, bears));
    assert!(!has_keyword(&game, bears, KeywordFlag::Flying));

    grant(&mut game, caster, wings, equipped_creature_has_flying());

    assert!(
        has_keyword(&game, bears, KeywordFlag::Flying),
        "the grant applied at layer 6, and the static ability it granted generated \
         a layer-6 effect whose existence check saw the grant already applied"
    );
    assert!(!has_keyword(&game, wings, KeywordFlag::Flying), "the Equipment itself does not fly");

    // CR 613.7a clause 2 and the tiebreak between them: the grant row sorts
    // at-or-before its own derived row, and both after a Humility that
    // entered earlier — so a creature Humility stripped still takes the
    // flying its Equipment was granted afterwards.
    let mut game = setup_two_player_game();
    let humility = put_on_battlefield(&mut game, phase_lf_cards::humility(), 1);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Vigilance]), 0);
    let wings = put_on_battlefield(&mut game, equipment("Bare Wings"), 0);
    assert!(game.attach(wings, bears));
    assert!(!has_keyword(&game, bears, KeywordFlag::Vigilance), "Humility took the printed keyword");

    grant(&mut game, humility, wings, equipped_creature_has_flying());

    assert!(has_keyword(&game, bears, KeywordFlag::Flying));
    assert!(!has_keyword(&game, bears, KeywordFlag::Vigilance));
    assert_eq!(pt(&game, bears), (Some(1), Some(1)), "and Humility's 7b part still applies");
}

// ---------------------------------------------------------------------------
// CR 613.7c / 613.4c — a +1/+1 counter is a layer 7c application on the
// same clock as the layer's rows, not something applied after them.
// ---------------------------------------------------------------------------

/// A counter older than a row that reads power is applied first. The
/// per-object walk applied every counter after the layer's rows, so the row
/// saw a 2/2 and pumped it: 4/4 where the CR says 3/3.
///
/// Only this order is pinned. In the other — the row older than the counter
/// — timestamp order gives 4/4, and CR 613.8 will not: applying the counter
/// changes what the row applies to, so the row depends on it and waits
/// (LI-2). A test of that order belongs there.
#[test]
fn test_a_counter_older_than_a_power_reading_row_applies_first() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.add_counters(bears, CounterType::PlusOnePlusOne, 1);
    assert_eq!(pt(&game, bears), (Some(3), Some(3)));

    // "Creatures with power 2 or less get +1/+1", from a resolution, later.
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(ContinuousEffect {
        id: 0,
        source: bears,
        origin: EffectOrigin::Resolution,
        layer: Layer::Layer7cModifyPT,
        duration: Duration::UntilEndOfTurn,
        controller: 0,
        created_on_turn: 1,
        timestamp,
        affected: AffectedSet::Filter {
            filter: ObjectFilter::And(
                Box::new(ObjectFilter::ByType(CardType::Creature)),
                Box::new(ObjectFilter::PowerLE(2)),
            ),
        },
        modification: EffectModification::ModifyPowerToughness {
            power: PtValue::Fixed(1),
            toughness: PtValue::Fixed(1),
        },
    });

    assert_eq!(
        pt(&game, bears),
        (Some(3), Some(3)),
        "the counter applied first, so the row found a 3-power creature and did not apply"
    );
}

// ---------------------------------------------------------------------------
// The two paths beside the pass: an object no row can reach, and a read that
// leaves the working set from inside one.
// ---------------------------------------------------------------------------

/// A CDA on a card in a graveyard reads the battlefield as the memo has it —
/// Keldon Warlord counting the non-Wall creatures its owner controls. The
/// card is no pass's member (no row can reach it), and the count it makes
/// goes through the settled board to the same frames a pass produced.
#[test]
fn test_a_cda_off_the_battlefield_counts_the_board() {
    let mut game = setup_two_player_game();
    let warlord = put_in_graveyard(&mut game, phase_rc_cards::keldon_warlord(), 0);
    assert_eq!(pt(&game, warlord), (Some(0), Some(0)), "nothing to count");

    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    assert_eq!(pt(&game, warlord), (Some(2), Some(2)), "two of yours, not the opponent's");

    // A count reads *effective* types: an animated Sol Ring is a creature.
    put_on_battlefield(&mut game, artifacts::sol_ring(), 0);
    assert_eq!(pt(&game, warlord), (Some(2), Some(2)));
    put_on_battlefield(&mut game, phase_ld_cards::march_of_the_machines(), 0);
    assert_eq!(pt(&game, warlord), (Some(3), Some(3)), "March made the Sol Ring a creature");
}

/// Tarmogoyf on the battlefield reads graveyard cards from inside the pass —
/// non-members, walked at layer 7a's ceiling — and those walks are not board
/// walks. One pass answers the whole board, and every member is then a hit.
#[test]
fn test_one_pass_answers_every_member_and_nested_reads_are_not_board_walks() {
    let mut game = setup_two_player_game();
    let goyf = put_on_battlefield(&mut game, phase_le_cards::tarmogoyf(), 0);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    put_in_graveyard(&mut game, card_of_type("Shock", CardType::Instant), 1);
    put_in_graveyard(&mut game, card_of_type("Forest", CardType::Land), 0);

    let (walks, board_walks, hits) = (
        game.counters.layer_walks(),
        game.counters.board_walks(),
        game.counters.memo_hits(),
    );
    assert_eq!(pt(&game, goyf), (Some(2), Some(3)), "instant and land: two types");
    assert_eq!(game.counters.layer_walks(), walks + 1, "one miss");
    assert_eq!(game.counters.board_walks(), board_walks + 1, "one board walk — the graveyard reads nest inside it");

    assert!(is_creature(&game, bears));
    assert_eq!(game.counters.layer_walks(), walks + 1, "the other member was filled by the same pass");
    // Two hits: `pt` asks twice, and the toughness query was already a hit.
    assert_eq!(game.counters.memo_hits(), hits + 2);
}
