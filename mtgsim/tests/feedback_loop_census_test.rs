//! The feedback-loop census's written tests (`plans/references/feedback-loops.md`):
//! one board per class that a registered card closes and no test pinned. Each
//! is a write that changes a decision the same pass is still making.

use std::sync::Arc;

use mtgsim::cards::creatures::{grizzly_bears, savannah_lions};
use mtgsim::cards::phase5_pre_cards::glorious_anthem;
use mtgsim::cards::phase_li_cards::opalescence;
use mtgsim::cards::phase_rc_cards::keldon_warlord;
use mtgsim::cards::phase_rd_cards::{guardian_seraph, palisade_giant, pyroclasm};
use mtgsim::cards::phase_re_cards::{platinum_angel, raise_the_alarm};
use mtgsim::cards::phase_tr1_cards::{blood_artist, soul_warden};
use mtgsim::cards::phase_tr2a_cards::vengeful_warchief;
use mtgsim::cards::phase_tr2b_cards::nykthos_paragon;
use mtgsim::engine::actions::{ActionContext, DestructionSource, GameAction};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::{DamageTarget, GameEvent};
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::{GameResult, GameState};
use mtgsim::test_support::{
    place_vanilla_creature, put_in_hand, put_on_battlefield, setup_two_player_game, test_ctx, test_dp,
    RecordingDecisionProvider,
};
use mtgsim::types::effects::{CounterType, EffectRecipient, SelectionFilter, TargetCount};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

fn on_battlefield(game: &GameState, id: ObjectId) -> bool {
    game.battlefield.contains_key(&id)
}

fn mark(game: &mut GameState, id: ObjectId, damage: u32) {
    game.battlefield.get_mut(&id).unwrap().damage_marked = damage;
}

fn sba(game: &mut GameState) -> bool {
    game.check_state_based_actions(&test_dp()).unwrap()
}

fn resolve_spell(game: &mut GameState, card: Arc<CardData>, controller: PlayerId) {
    let source = put_in_hand(game, card.clone(), controller);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller,
        targets: ChosenTargets::one(Vec::new()),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
}

fn place(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).expect("placing");
}

fn plus_counters(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].counter_count(CounterType::PlusOnePlusOne)
}

// ---------------------------------------------------------------------------
// Loop 1 — CR 616.1f's re-gather after a rewrite changes the event's subject
// ---------------------------------------------------------------------------

/// The `subject` class. Palisade Giant moves damage aimed at its controller
/// onto itself, and Guardian Seraph prevents 1 of damage dealt to that player.
/// Seraph first: 1 prevented, and the rest still goes to the Giant (4). The
/// Giant first: the event no longer deals damage to the player, so the
/// re-gather no longer offers the Seraph (5). The player is asked once.
#[test]
fn palisade_giant_beside_guardian_seraph_is_an_order_the_redirect_can_end() {
    let mut taken = Vec::new();
    for pick in 0..2 {
        let mut game = setup_two_player_game();
        let giant = put_on_battlefield(&mut game, palisade_giant(), 0);
        put_on_battlefield(&mut game, guardian_seraph(), 0);
        let source = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
        let dp = RecordingDecisionProvider::picking(pick);
        let damage = GameAction::DealDamage {
            source,
            target: DamageTarget::Player(0),
            amount: 5,
            is_combat: false,
            unpreventable: false,
        };
        game.execute_action(damage, &ActionContext::new(&dp)).unwrap();
        assert_eq!(game.players[0].life_total, 20, "the player took none either way");
        assert_eq!(dp.prompts(), 1, "two candidates, one question");
        taken.push(game.battlefield[&giant].damage_marked);
    }
    taken.sort_unstable();
    assert_eq!(taken, vec![4, 5]);
}

// ---------------------------------------------------------------------------
// Loop 3 — CR 704.3's next check reads what the last one did
// ---------------------------------------------------------------------------

/// The `anthem` class. Opalescence makes Glorious Anthem a 3/3, 4/4 under its
/// own anthem, and four damage kills it. The Bears were 3/3 with two damage
/// and survive that check; the next one reads them as 2/2.
#[test]
fn an_animated_glorious_anthem_dies_and_the_next_check_takes_what_it_held_up() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, opalescence(), 0);
    let anthem = put_on_battlefield(&mut game, glorious_anthem(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    mark(&mut game, anthem, 4);
    mark(&mut game, bears, 2);

    assert!(sba(&mut game));
    assert!(!on_battlefield(&game, anthem), "the first check destroys the anthem");
    assert!(on_battlefield(&game, bears), "and reads the Bears as 3/3");

    assert!(sba(&mut game));
    assert!(!on_battlefield(&game, bears), "the next check reads them as 2/2");
    assert!(!sba(&mut game));
}

/// The `count` class. Keldon Warlord counts the non-Wall creatures its
/// controller has, itself included: 3/3 beside two others. Pyroclasm deals 2
/// to each; the first check takes the other two, and the next one reads the
/// Warlord as 1/1 with 2 damage.
#[test]
fn keldon_warlord_dies_on_the_check_after_pyroclasms_other_victims() {
    let mut game = setup_two_player_game();
    let warlord = put_on_battlefield(&mut game, keldon_warlord(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let lions = put_on_battlefield(&mut game, savannah_lions(), 0);

    resolve_spell(&mut game, pyroclasm(), 1);

    assert!(sba(&mut game));
    assert!(!on_battlefield(&game, bears) && !on_battlefield(&game, lions));
    assert!(on_battlefield(&game, warlord), "3/3 with 2 damage when the first check looked");

    assert!(sba(&mut game));
    assert!(!on_battlefield(&game, warlord), "1/1 with 2 damage at the next");
}

/// The `token` class on ATOM-117.5-001's board: a creature token with lethal
/// damage is destroyed, and the token in the graveyard then ceases to exist,
/// both before anyone has priority.
// COVERS: ATOM-117.5-001
#[test]
fn a_soldier_token_dies_and_ceases_to_exist_in_one_cycle() {
    let mut game = setup_two_player_game();
    game.record_events();
    resolve_spell(&mut game, raise_the_alarm(), 0);
    let token = game
        .battlefield_ids_ordered()
        .into_iter()
        .find(|id| game.objects.get(id).is_some_and(|o| o.is_token))
        .expect("two Soldiers");
    mark(&mut game, token, 1);

    place(&mut game, &test_dp());

    assert!(!game.objects.contains_key(&token), "CR 704.5d: it ceased to exist");
    assert!(!game.players[0].graveyard.contains(&token));
    let died_then_ceased: Vec<bool> = game
        .recorded_events()
        .records()
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::ZoneChange { object_id, .. } if *object_id == token => Some(true),
            GameEvent::TokenCeasedToExist { object_id } if *object_id == token => Some(false),
            _ => None,
        })
        .collect();
    assert_eq!(died_then_ceased, vec![true, false], "it died first, so a dies trigger saw it");
}

/// The `cant-lose` class. One check finds Platinum Angel's controller at 0
/// life and the Angel with lethal damage. It reads one board (CR 704.3), so
/// the loss is refused while the Angel is destroyed; the next check loses.
#[test]
fn platinum_angel_dying_in_the_check_that_refused_the_loss_loses_on_the_next() {
    let mut game = setup_two_player_game();
    let angel = put_on_battlefield(&mut game, platinum_angel(), 0);
    game.players[0].life_total = 0;
    mark(&mut game, angel, 4);

    assert!(sba(&mut game));
    assert!(!on_battlefield(&game, angel));
    assert!(!game.player_lost[0], "the Angel was there when the check looked");

    assert!(sba(&mut game));
    assert!(game.player_lost[0]);
    assert_eq!(game.result, Some(GameResult::Winner(1)));
}

// ---------------------------------------------------------------------------
// Loop 4 — a trigger's resolution is an event, dispatched like any other
// ---------------------------------------------------------------------------

/// The `gain` class, and the path every loop 4 row cites: Soul Warden's gain
/// is performed through the chokepoint, so its batch's close dispatches it,
/// and Nykthos Paragon triggers on it for the next placement.
#[test]
fn soul_wardens_gain_triggers_nykthos_paragon_at_the_next_placement() {
    let mut game = setup_two_player_game();
    let paragon = put_on_battlefield(&mut game, nykthos_paragon(), 0);
    put_on_battlefield(&mut game, soul_warden(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    assert_eq!(game.pending_triggers.len(), 1, "Soul Warden saw the Bears enter");

    place(&mut game, &test_dp());
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.players[0].life_total, 21);
    assert_eq!(game.pending_triggers.len(), 1, "the gain is an event");
    assert_eq!(game.pending_triggers[0].origin.source(), paragon);

    place(&mut game, &test_dp());
    let ability = *game.stack.last().unwrap();
    let yes = ScriptedDecisionProvider::new();
    yes.expect_pick_n(ChoiceKind::ApplyOptionalEffect { source: ability }, vec![0]);
    game.resolve_top_of_stack(&yes).unwrap();
    assert_eq!((plus_counters(&game, paragon), plus_counters(&game, bears)), (1, 1));
}

/// The `loss` class across seats: Blood Artist's drain is player 1's life
/// loss, and player 1's Vengeful Warchief triggers on it.
#[test]
fn blood_artists_drain_triggers_the_opponents_vengeful_warchief() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, blood_artist(), 0);
    let warchief = put_on_battlefield(&mut game, vengeful_warchief(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let wrath = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let destroy = GameAction::Destroy { object: bears, source: DestructionSource::Effect(wrath) };
    game.execute_action(destroy, &test_ctx()).unwrap();
    assert_eq!(game.pending_triggers.len(), 1, "Blood Artist saw the Bears die");

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(
        ChoiceKind::SelectRecipients {
            recipient: EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
            spell_id: ObjectId::UNASSIGNED,
        },
        vec![1],
    );
    place(&mut game, &dp);
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!((game.players[0].life_total, game.players[1].life_total), (21, 19));
    assert_eq!(game.pending_triggers.len(), 1, "the drain is an event");
    assert_eq!(game.pending_triggers[0].origin.source(), warchief);

    place(&mut game, &test_dp());
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(plus_counters(&game, warchief), 1);
}
