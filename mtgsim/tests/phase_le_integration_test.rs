//! Characteristic-defining abilities — CR 604.3, 613.3, 613.4a.
//!
//! Tarmogoyf is the Layer 7a card and Culling Drone (Devoid) the Layer 5 one.
//! The mechanism they share is that a CDA is never a registry effect: see
//! `engine::layers::cda` for why CR 604.3a(3) makes that the right shape.

use mtgsim::test_support::{
    card_of_type, put_in_graveyard, put_in_hand, put_on_battlefield, setup_two_player_game,
};
use mtgsim::cards::{alpha, creatures, phase_le_cards};
use mtgsim::engine::layers::types::{
    ContinuousEffect, EffectModification, Layer, PtValue,
};
use mtgsim::engine::priority::PriorityResult;
use mtgsim::objects::card_data::CardDataBuilder;
use mtgsim::oracle::characteristics::{
    get_effective_colors, get_effective_power, get_effective_toughness,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{EffectRecipient, SelectionFilter, TargetCount};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// One registry row applying to `id` alone. Every row in this file sits at
/// timestamp 1; `test_support::registered` takes it as a parameter.
fn registered(id: ObjectId, layer: Layer, modification: EffectModification) -> ContinuousEffect {
    mtgsim::test_support::registered(id, layer, 1, modification)
}

// ---------------------------------------------------------------------------
// Layer 7a — Tarmogoyf
// ---------------------------------------------------------------------------

#[test]
fn test_tarmogoyf_pt_tracks_card_types_in_all_graveyards() {
    let mut game = setup_two_player_game();
    let goyf = put_on_battlefield(&mut game, phase_le_cards::tarmogoyf(), 0);

    // Empty graveyards: 0 types, so 0/1.
    assert_eq!(get_effective_power(&game, goyf), Some(0));
    assert_eq!(get_effective_toughness(&game, goyf), Some(1));

    // One instant in player 0's graveyard: 1/2.
    put_in_graveyard(&mut game, card_of_type("Shock", CardType::Instant), 0);
    assert_eq!(get_effective_power(&game, goyf), Some(1));
    assert_eq!(get_effective_toughness(&game, goyf), Some(2));

    // "All graveyards" — the opponent's counts too.
    put_in_graveyard(&mut game, card_of_type("Wastes", CardType::Land), 1);
    assert_eq!(get_effective_power(&game, goyf), Some(2));
    assert_eq!(get_effective_toughness(&game, goyf), Some(3));
}

#[test]
fn test_tarmogoyf_counts_card_types_not_cards() {
    let mut game = setup_two_player_game();
    let goyf = put_on_battlefield(&mut game, phase_le_cards::tarmogoyf(), 0);

    // Ten artifact creatures are ten cards but two card types.
    for i in 0..10 {
        let artifact_creature = CardDataBuilder::new(&format!("Ornithopter {}", i))
            .card_type(CardType::Artifact)
            .card_type(CardType::Creature)
            .power_toughness(0, 2)
            .build();
        put_in_graveyard(&mut game, artifact_creature, 0);
    }

    assert_eq!(get_effective_power(&game, goyf), Some(2));
    assert_eq!(get_effective_toughness(&game, goyf), Some(3));
}

// COVERS: ATOM-613.4a-001
#[test]
fn test_tarmogoyf_cda_applies_in_7a_then_pump_in_7c() {
    let mut game = setup_two_player_game();
    let goyf = put_on_battlefield(&mut game, phase_le_cards::tarmogoyf(), 0);

    put_in_graveyard(&mut game, card_of_type("Shock", CardType::Instant), 0);
    put_in_graveyard(&mut game, card_of_type("Wastes", CardType::Land), 0);
    put_in_graveyard(&mut game, card_of_type("Ancestral Vision", CardType::Sorcery), 1);
    assert_eq!(get_effective_power(&game, goyf), Some(3));
    assert_eq!(get_effective_toughness(&game, goyf), Some(4));

    // Giant Growth — a Layer 7c modification, applied on top of the 7a value.
    game.continuous_effects.add(registered(
        goyf,
        Layer::Layer7cModifyPT,
        EffectModification::ModifyPowerToughness {
            power: PtValue::Fixed(3),
            toughness: PtValue::Fixed(3),
        },
    ));

    assert_eq!(get_effective_power(&game, goyf), Some(6));
    assert_eq!(get_effective_toughness(&game, goyf), Some(7));
}

// COVERS: ATOM-113.12-001
// COVERS-PARTIAL: COMP-613-TARMOGOYF-HUMILITY-001
//
// Partial on the COMP: its board is "Player A controls Humility", and there is
// no Humility here. Its two effects are written straight into the registry as
// `ContinuousEffect` values, rather than produced by putting a card with
// Humility's text onto the battlefield and letting `register_static_effects`
// derive them. That proves the layer ordering but not that any card reaches it.
// Kept as an isolation test of the ordering alone.
// The COMP's actual board is covered by
// `phase_lf_integration_test::test_humility_strips_tarmogoyfs_cda_before_layer_7a_can_read_it`,
// which uses `phase_lf_cards::humility`.
#[test]
fn test_tarmogoyf_pt_is_layer_7a_and_an_ability_strip_removes_it() {
    let mut game = setup_two_player_game();
    let goyf = put_on_battlefield(&mut game, phase_le_cards::tarmogoyf(), 0);

    for (name, card_type) in [
        ("Shock", CardType::Instant),
        ("Wastes", CardType::Land),
        ("Ancestral Vision", CardType::Sorcery),
        ("Sol Ring", CardType::Artifact),
        ("Grizzly Bears", CardType::Creature),
    ] {
        put_in_graveyard(&mut game, card_of_type(name, card_type), 0);
    }
    assert_eq!(get_effective_power(&game, goyf), Some(5));

    // Layer 7b, applied after 7a, overrides the CDA outright — which is how we
    // know the CDA landed in 7a and not in 7b, 7c, or 7d.
    game.continuous_effects.add(registered(
        goyf,
        Layer::Layer7bSetPT,
        EffectModification::SetPowerToughness {
            power: PtValue::Fixed(1),
            toughness: PtValue::Fixed(1),
        },
    ));
    assert_eq!(get_effective_power(&game, goyf), Some(1));
    assert_eq!(get_effective_toughness(&game, goyf), Some(1));

    // Humility's other half: "all creatures lose all abilities", Layer 6.
    // 613.4a puts 7a *after* 6, so the CDA is gone before it can define
    // anything, and 7b's 1/1 is the whole answer. Tarmogoyf is 1/1.
    game.continuous_effects.add(registered(
        goyf,
        Layer::Layer6Ability,
        EffectModification::LoseAllAbilities,
    ));
    assert_eq!(get_effective_power(&game, goyf), Some(1));
    assert_eq!(get_effective_toughness(&game, goyf), Some(1));
    assert!(mtgsim::oracle::characteristics::get_effective_abilities(&game, goyf).is_empty());
}

// COVERS-PARTIAL: ATOM-113.6a-001
//
// Partial: the atom pairs CR 113.6a (a CDA functions in all zones) with CR
// 113.6b ("activate this ability only from your graveyard"), which is ticket
// T19 and a different mechanism. Only the CDA half is under test here.
#[test]
fn test_tarmogoyf_in_a_graveyard_has_a_pt_and_counts_itself() {
    let mut game = setup_two_player_game();
    let goyf = put_in_graveyard(&mut game, phase_le_cards::tarmogoyf(), 0);

    // CR 604.3 — CDAs function in all zones. The only card in any graveyard is
    // Tarmogoyf itself, which is a creature card: one card type, so 1/2.
    assert_eq!(get_effective_power(&game, goyf), Some(1));
    assert_eq!(get_effective_toughness(&game, goyf), Some(2));

    put_in_graveyard(&mut game, card_of_type("Shock", CardType::Instant), 1);
    assert_eq!(get_effective_power(&game, goyf), Some(2));
    assert_eq!(get_effective_toughness(&game, goyf), Some(3));
}

/// CR 208.2, and the Merfolk Trickster ruling states it outright: "If the target
/// creature has power and toughness written as */* with an ability that defines
/// its power and toughness, it's 0/0 when it loses all abilities. If its power
/// and toughness are written as */*+1, it's 0/1, and so on."
///
/// Tarmogoyf's box is `*/1+*`, the `*/*+1` shape. So an ability strip with no
/// P/T-setting effect of its own leaves a **0/1**, not a 0/0 — and 0/1 survives
/// state-based actions where 0/0 would not.
#[test]
fn test_tarmogoyf_stripped_of_abilities_is_0_1_and_survives_sbas() {
    let mut game = setup_two_player_game();
    let goyf = put_on_battlefield(&mut game, phase_le_cards::tarmogoyf(), 0);
    put_in_graveyard(&mut game, card_of_type("Shock", CardType::Instant), 0);
    assert_eq!(get_effective_power(&game, goyf), Some(1));
    assert_eq!(get_effective_toughness(&game, goyf), Some(2));

    // Merfolk Trickster: "loses all abilities until end of turn" — no P/T set.
    game.continuous_effects.add(registered(
        goyf,
        Layer::Layer6Ability,
        EffectModification::LoseAllAbilities,
    ));

    assert_eq!(get_effective_power(&game, goyf), Some(0));
    assert_eq!(
        get_effective_toughness(&game, goyf),
        Some(1),
        "the printed box is */1+*, so the `1+` survives with * as 0"
    );

    let decisions = ScriptedDecisionProvider::new();
    game.check_state_based_actions_loop(&decisions).unwrap();
    assert_eq!(
        game.get_object(goyf).unwrap().zone,
        Zone::Battlefield,
        "a 0/1 has positive toughness — CR 704.5f does not apply"
    );
}

// ---------------------------------------------------------------------------
// The one everybody knows: Bolt the Goyf
// ---------------------------------------------------------------------------

/// Scryfall ruling on Tarmogoyf: "If an instant or sorcery spell deals damage
/// to Tarmogoyf or lowers its toughness, that spell is put into its owner's
/// graveyard before state-based actions are performed. If that card is the
/// first of its type to enter a graveyard, it will raise Tarmogoyf's toughness
/// before the game checks to see if Tarmogoyf dies."
///
/// This is the test that a snapshotted CDA would fail. The value has to be
/// recomputed at the moment the SBA asks, not cached from when damage landed.
#[test]
fn test_lightning_bolt_does_not_kill_tarmogoyf_because_bolt_itself_grows_it() {
    let mut game = setup_two_player_game();
    let goyf = put_on_battlefield(&mut game, phase_le_cards::tarmogoyf(), 0);

    // A land and a creature between the graveyards: 2 card types, so 2/3.
    put_in_graveyard(&mut game, card_of_type("Wastes", CardType::Land), 0);
    put_in_graveyard(&mut game, creatures::grizzly_bears(), 1);
    assert_eq!(get_effective_power(&game, goyf), Some(2));
    assert_eq!(get_effective_toughness(&game, goyf), Some(3));

    // Player 1 bolts it. Three damage against three toughness would be lethal.
    let bolt_id = put_in_hand(&mut game, alpha::lightning_bolt(), 1);
    game.players[1].mana_pool.add(ManaType::Red, 1);
    game.active_player = 1;

    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![1]);
    decisions.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
            spell_id: bolt_id,
        },
        // [Player(0), Player(1), Object(goyf)] for `Any`.
        vec![2],
    );
    assert_eq!(
        game.run_priority_round(&decisions).unwrap(),
        PriorityResult::ActionTaken
    );

    // Both pass — Bolt resolves, then SBAs run.
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    decisions.expect_pick_n(ChoiceKind::PriorityAction, vec![0]);
    assert_eq!(
        game.run_priority_round(&decisions).unwrap(),
        PriorityResult::StackResolved
    );

    // Bolt is an instant, and it is now in a graveyard — a third card type.
    assert_eq!(game.get_object(bolt_id).unwrap().zone, Zone::Graveyard);
    assert_eq!(
        game.battlefield.get(&goyf).map(|e| e.damage_marked),
        Some(3),
        "the damage was dealt"
    );
    assert_eq!(
        get_effective_toughness(&game, goyf),
        Some(4),
        "Bolt's own card type raised it before SBAs looked"
    );
    assert_eq!(
        game.get_object(goyf).unwrap().zone,
        Zone::Battlefield,
        "3 damage on a 3/4 is not lethal — Tarmogoyf survives its own Bolt"
    );
}

// ---------------------------------------------------------------------------
// Layer 5 — Devoid
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-702.114a-001
//
// Partial: the atom's point is "this applies in all zones" and it checks a card
// in *hand*. Devoid is a CDA, so the battlefield case this builds is the one
// the layer system can answer today; the in-hand read needs the pre-layer zone
// to route through `oracle/characteristics.rs` first.
#[test]
fn test_devoid_makes_a_black_card_colorless() {
    let mut game = setup_two_player_game();
    let drone = put_on_battlefield(&mut game, phase_le_cards::culling_drone(), 0);

    assert!(
        get_effective_colors(&game, drone).is_empty(),
        "CR 702.114a — Devoid is a CDA and defines the card as colorless"
    );
    assert_eq!(
        game.continuous_effects.len(),
        0,
        "a CDA is never a registry effect (CR 604.3a(3))"
    );
}

// COVERS-PARTIAL: ATOM-113.12-002
//
// Partial: the "remove all abilities" effect is a `ContinuousEffect` written
// directly into the registry, not one derived from a card's text at ETB. Kept
// because it isolates the Layer 5 / Layer 6 ordering from anything
// Humility also does; the atom's full scenario is covered by
// `phase_lf_integration_test::test_humility_does_not_restore_a_devoid_cards_printed_color`.
#[test]
fn test_ability_strip_does_not_restore_devoids_printed_color() {
    let mut game = setup_two_player_game();
    let drone = put_on_battlefield(&mut game, phase_le_cards::culling_drone(), 0);

    game.continuous_effects.add(registered(
        drone,
        Layer::Layer6Ability,
        EffectModification::LoseAllAbilities,
    ));

    // Layer 5 runs before Layer 6, so the color was already defined when the
    // ability was removed. CR 113.12: the characteristic was *set*, not
    // granted, and losing the ability afterwards cannot un-set it.
    assert!(
        get_effective_colors(&game, drone).is_empty(),
        "the card stays colorless"
    );
    assert!(!get_effective_colors(&game, drone).contains(&Color::Black));

    // The mirror image, and the reason ATOM-113.12-001 had to be corrected:
    // for P/T the same strip *does* win, because 7a follows 6.
    let goyf = put_on_battlefield(&mut game, phase_le_cards::tarmogoyf(), 0);
    put_in_graveyard(&mut game, card_of_type("Shock", CardType::Instant), 0);
    assert_eq!(get_effective_power(&game, goyf), Some(1));
    game.continuous_effects.add(registered(
        goyf,
        Layer::Layer6Ability,
        EffectModification::LoseAllAbilities,
    ));
    assert_eq!(get_effective_power(&game, goyf), Some(0));
}
