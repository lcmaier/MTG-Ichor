//! `codebase-state.md` item 223's event half: an ability's effects belong to
//! the object that has the ability (CR 113.7, 120.2b, 608.2h), not to the
//! stack object CR 608.2n removes.
//!
//! 1. The life an ability gains is the permanent's, and the log says so.
//! 2. Lifelink and Circle of Protection: Red read the permanent that deals an
//!    ability's damage.
//!
//! The boards resolve real activations and real triggers, so the stack
//! object exists and is gone by the time anything reads the source.

use std::sync::Arc;

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lh_cards::loxodon_warhammer;
use mtgsim::cards::phase_rd_cards::circle_of_protection_red;
use mtgsim::cards::phase_tr1_cards::soul_warden;
use mtgsim::engine::actions::GameAction;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_on_battlefield, setup_two_player_game, test_ctx, test_dp, RecordingDecisionProvider};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, PlayerGroup, PlayerSet, Primitive};
use mtgsim::types::ids::{AbilityId, ObjectId, PlayerId};
use mtgsim::types::replacement::EventPattern;
use mtgsim::ui::display::format_event_log;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

/// "This creature deals `n` damage to each opponent."
fn damage_each_opponent(n: u64) -> Effect {
    Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::Fixed(n), unpreventable: false },
        EffectRecipient::EachOf(PlayerGroup::set(PlayerSet::Opponents)),
    )
}

/// A red 1/1: "{T}: This creature deals 1 damage to each opponent." Red, so
/// Circle of Protection: Red may choose it; no lifelink of its own, so
/// whatever lifelink it has comes from the Warhammer.
fn pinger() -> Arc<CardData> {
    CardDataBuilder::new("Ember Pinger")
        .card_type(CardType::Creature)
        .color(Color::Red)
        .power_toughness(1, 1)
        .ability(AbilityDef {
            rules_text: "{T}: This creature deals 1 damage to each opponent.".into(),
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Activated,
            costs: vec![Cost::TapSelf],
            effect: damage_each_opponent(1),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
        })
        .build()
}

fn equip(game: &mut GameState, equipment: ObjectId, host: ObjectId) {
    game.execute_action(GameAction::Attach { attachment: equipment, host }, &test_ctx()).unwrap();
}

/// Activate `source`'s first ability for `player` and resolve it.
fn activate_and_resolve(game: &mut GameState, player: PlayerId, source: ObjectId) {
    game.activate_ability(player, source, 0, &test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
}

fn life_changes(game: &GameState) -> Vec<(PlayerId, i64, Option<ObjectId>)> {
    game.recorded_events()
        .records()
        .iter()
        .filter_map(|r| match r.event {
            GameEvent::LifeChanged { player_id, old, new, source, .. } => Some((player_id, new - old, source)),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 1. The life an ability gains
// ---------------------------------------------------------------------------

/// #231's `fizzle_log.png`: Blood Artist's gain was logged as `(source: #22)`,
/// its trigger's stack object, gone by the time the log read it. Soul
/// Warden's gain is Soul Warden's (CR 113.7), and its line names it.
#[test]
fn an_abilitys_life_gain_is_logged_as_the_permanents() {
    let mut game = setup_two_player_game();
    let warden = put_on_battlefield(&mut game, soul_warden(), 0);
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();

    assert_eq!(life_changes(&game), vec![(0, 1, Some(warden))]);
    let line = format!("LifeChanged: P0 20 -> 21 (source: Soul Warden ({warden}))");
    let log = format_event_log(&game);
    assert!(log.contains(&line), "no {line:?} in {log:#?}");
}

// ---------------------------------------------------------------------------
// 2. The permanent deals an ability's damage
// ---------------------------------------------------------------------------

/// CR 120.2b, 608.2h: "This creature deals 1 damage" is the creature's
/// damage, so the Warhammer's lifelink on it gains its controller that much
/// (CR 702.15b). The ability's stack object never had lifelink: the grant is
/// on the creature.
#[test]
fn lifelink_reads_the_permanent_whose_ability_deals_the_damage() {
    let mut game = setup_two_player_game();
    let pinger = put_on_battlefield(&mut game, pinger(), 0);
    let warhammer = put_on_battlefield(&mut game, loxodon_warhammer(), 0);
    equip(&mut game, warhammer, pinger);

    activate_and_resolve(&mut game, 0, pinger);

    assert_eq!(life(&game, 1), 19);
    assert_eq!(life(&game, 0), 21, "the pinger has lifelink, and it dealt the damage");
    assert_eq!(life_changes(&game), vec![(1, -1, Some(pinger)), (0, 1, Some(pinger))], "the loss and the gain both the pinger's");
}

/// CR 609.7a: "If the player chooses a permanent, the effect will apply to
/// the next damage dealt by that permanent, regardless of whether it's
/// combat damage or damage dealt as the result of a spell or ability." The
/// Circle chooses the pinger, and the pinger's ability's damage is the
/// pinger's.
#[test]
fn circle_of_protection_red_prevents_an_abilitys_damage_from_the_chosen_permanent() {
    let mut game = setup_two_player_game();
    // Oldest, so the CR 609.7a prompt lists it first.
    let pinger = put_on_battlefield(&mut game, pinger(), 1);
    let circle_card = circle_of_protection_red();
    let circle = put_on_battlefield(&mut game, circle_card.clone(), 0);
    let ctx = ResolutionContext { ability_source: game.object_ref(circle), ..ResolutionContext::untargeted(circle, 0) };
    game.resolve_effect(&circle_card.abilities[0].effect, &ctx, &RecordingDecisionProvider::picking(0)).unwrap();
    let chosen = game.replacement_effects.iter().find_map(|row| match &row.def.pattern {
        EventPattern::DealDamage { source: Some(pattern), .. } => pattern.object,
        _ => None,
    });
    assert_eq!(chosen, Some(pinger), "CR 609.7a's choice, made as the effect was created");

    activate_and_resolve(&mut game, 1, pinger);

    assert_eq!(life(&game, 0), 20, "the chosen permanent's damage, prevented");
    assert_eq!(game.replacement_effects.iter().count(), 0, "and the shield is spent (CR 615.8)");
}
