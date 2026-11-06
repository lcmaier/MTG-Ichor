//! `board-scale`: one event making many tokens, at a cost that grows with the
//! tokens and no faster. Every token of a run asks CR 614.12's question of one
//! frame, so these boards are the ones where a frame shared wrongly would show:
//! an anthem the frame must read, a replacement a frame must take into
//! account, and proposals a player's answer makes unequal. Each board runs
//! audited, so a served frame is compared with the walk it saved
//! (`shared_entering_frame`).

use std::sync::Arc;

use mtgsim::cards::fixtures::essence_of_the_wild_fixture;
use mtgsim::cards::printings::{blood_moon, glorious_anthem, sigarda_host_of_herons};
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::layers::compute_characteristics;
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{AbilityDef, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_abilities, get_effective_power};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_on_battlefield, setup_game, test_ctx, textless_static};
use mtgsim::types::card_types::{CardType, CreatureType};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{CounterType, Effect, ObjectFilter, ObjectSet, PlayerRef, TokenDef};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::replacement::{EnterModsTemplate, EventPattern, ReplacementDef, Rewrite};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::ScriptedDecisionProvider;

fn goblin() -> TokenDef {
    TokenDef::creature(1, 1, &[Color::Red], &[CreatureType::Goblin])
}

fn enchantment_with(name: &str, ability: AbilityDef) -> Arc<CardData> {
    CardDataBuilder::new(name).card_type(CardType::Enchantment).ability(ability).build()
}

/// "Each permanent with power `at_least` or greater enters ...": a replacement
/// whose reach is the entering object's frame, so it reads CR 614.12. No card
/// prints one by power (two searches, 2026-11-05: Slinza, the Spiked Stampede's
/// reads a subtype); power is what an anthem writes.
fn entry_at_power(at_least: i32, template: EnterModsTemplate) -> ReplacementDef {
    let reaches = ObjectFilter::Not(Box::new(ObjectFilter::PowerLE(at_least - 1)));
    ReplacementDef::new(
        EventPattern::EnterBattlefield { cast: None },
        ObjectSet::battlefield_filter(reaches),
        Rewrite::EnterWith(template),
    )
}

fn put_replacement(game: &mut GameState, def: ReplacementDef) -> ObjectId {
    let ability = textless_static(Effect::Replacement(Box::new(def)));
    put_on_battlefield(game, enchantment_with("Entry Replacement Fixture", ability), 0)
}

fn plus_one_counter() -> EnterModsTemplate {
    EnterModsTemplate::with_counters(CounterType::PlusOnePlusOne, 1)
}

fn create(game: &mut GameState, defs: Vec<TokenDef>, controller: PlayerId) {
    game.execute_action(GameAction::CreateTokens { defs, controller, attacking: None }, &test_ctx())
        .expect("the creation performs");
}

fn audited_game() -> GameState {
    let mut game = setup_game(4);
    game.enable_audit();
    game
}

/// A noncreature token of one type, with nothing else to it.
fn token_of_type(card_type: CardType) -> TokenDef {
    TokenDef { colors: Vec::new(), types: vec![card_type], subtypes: Vec::new(), power: None, toughness: None, ..goblin() }
}

/// The tokens `owner` owns, whoever controls them, oldest first.
fn tokens_owned_by(game: &GameState, owner: PlayerId) -> Vec<ObjectId> {
    game.battlefield_ids_ordered()
        .into_iter()
        .filter(|id| game.objects[id].is_token && game.objects[id].owner == owner)
        .collect()
}

fn plus_one_counters(game: &GameState, tokens: &[ObjectId]) -> Vec<u32> {
    tokens.iter().map(|id| game.battlefield[id].counter_count(CounterType::PlusOnePlusOne)).collect()
}

/// What one creation of `n` Goblins costs in passes over the board.
fn board_walks_for(n: usize, board: impl Fn(&mut GameState)) -> u64 {
    let mut game = setup_game(4);
    board(&mut game);
    let before = game.diagnostics.board_walks();
    create(&mut game, vec![goblin(); n], 0);
    game.diagnostics.board_walks() - before
}

#[test]
fn a_run_of_tokens_costs_the_same_passes_however_long_it_is() {
    let bare = |_: &mut GameState| {};
    assert_eq!(board_walks_for(2, bare), board_walks_for(200, bare));

    let anthem = |game: &mut GameState| {
        put_on_battlefield(game, glorious_anthem(), 0);
    };
    assert_eq!(board_walks_for(2, anthem), board_walks_for(200, anthem));

    let frame_reader = |game: &mut GameState| {
        put_on_battlefield(game, glorious_anthem(), 0);
        put_replacement(game, entry_at_power(2, plus_one_counter()));
    };
    assert_eq!(board_walks_for(2, frame_reader), board_walks_for(200, frame_reader));
}

/// What one creation asks of the board's permanents, as memo hits: it asks
/// each source of a "can't" and no other permanent, however many there are.
fn memo_hits_beside(goblins: usize) -> u64 {
    let mut game = setup_game(4);
    put_on_battlefield(&mut game, sigarda_host_of_herons(), 0);
    create(&mut game, vec![goblin(); goblins], 1);
    let before = game.diagnostics.memo_hits();
    create(&mut game, vec![goblin(); 2], 0);
    game.diagnostics.memo_hits() - before
}

#[test]
fn a_static_cant_costs_an_event_nothing_for_the_permanents_beside_it() {
    assert_eq!(memo_hits_beside(2), memo_hits_beside(200));
}

/// The sweep narrowed to its sources still finds what a row grants anywhere:
/// the audit compares it with every permanent's.
#[test]
fn a_cant_is_found_with_the_sweep_narrowed_to_its_sources() {
    let mut game = audited_game();
    let sigarda = put_on_battlefield(&mut game, sigarda_host_of_herons(), 0);
    create(&mut game, vec![goblin(); 3], 0);
    assert!(game.restriction_ability_sources.contains(&sigarda));

    create(&mut game, vec![goblin(); 3], 1);

    assert_eq!(tokens_owned_by(&game, 1).len(), 3, "she says nothing of an entry");
}


/// With no row to write a name, the copy a permanent entered as names it, read
/// off the entity and not off a pass: what the record says, and what a walk
/// would have said.
#[test]
fn a_token_that_entered_as_a_copy_is_announced_under_the_copys_name() {
    let mut game = audited_game();
    game.record_events();
    put_on_battlefield(&mut game, essence_of_the_wild_fixture(), 0);

    create(&mut game, vec![goblin(); 3], 0);

    for token in tokens_owned_by(&game, 0) {
        assert_eq!(compute_characteristics(&game, token).unwrap().name, "Essence of the Wild Fixture");
        let announced = game.recorded_events().records().iter().rev().find_map(|record| match &record.event {
            GameEvent::PermanentEnteredBattlefield { object, .. } if object.id == token => record.names.clone(),
            _ => None,
        });
        assert_eq!(announced.as_deref(), Some(&vec![(token, "Essence of the Wild Fixture".to_string())]));
    }
    assert_eq!(get_effective_power(&game, tokens_owned_by(&game, 0)[0]), Some(6));
}

/// An application that reads nothing is next without a check, in the key's
/// order: an anthem still lands on creatures that hold counters, whichever
/// came first.
#[test]
fn counters_and_an_anthem_apply_to_every_creature_of_a_board_of_counters() {
    let mut game = audited_game();
    create(&mut game, vec![goblin(); 5], 0);
    let goblins = tokens_owned_by(&game, 0);
    let put_on = |&id| GameAction::AddCounters { subject: ObjectOrPlayerId::Object(id), counter: CounterType::PlusOnePlusOne, n: 1, by: 0 };
    game.execute_actions(goblins.iter().map(put_on).collect(), &test_ctx()).expect("the counters go on");
    put_on_battlefield(&mut game, glorious_anthem(), 0);

    assert!(goblins.iter().all(|id| get_effective_power(&game, *id) == Some(3)));
}

/// CR 614.12's third clause: the continuous effects that exist are in the
/// frame, so P0's Goblins are 2/2 as they enter and P1's are 1/1.
#[test]
fn every_token_of_a_run_is_read_with_the_anthem_it_enters_under() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, glorious_anthem(), 0);
    put_replacement(&mut game, entry_at_power(2, plus_one_counter()));

    create(&mut game, vec![goblin(); 5], 0);
    create(&mut game, vec![goblin(); 5], 1);

    assert_eq!(plus_one_counters(&game, &tokens_owned_by(&game, 0)), vec![1; 5]);
    assert_eq!(plus_one_counters(&game, &tokens_owned_by(&game, 1)), vec![0; 5]);
}

/// CR 614.12's first clause, for each token in turn: the counter the first
/// effect gave it is in the frame the second effect reads, and the first
/// token's rewritten proposal is not served to the second token's plain one.
#[test]
fn a_replacement_already_applied_is_in_every_tokens_next_frame() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, glorious_anthem(), 0);
    put_replacement(&mut game, entry_at_power(2, plus_one_counter()));
    put_replacement(&mut game, entry_at_power(3, EnterModsTemplate::tapped()));

    create(&mut game, vec![goblin(); 4], 0);

    let goblins = tokens_owned_by(&game, 0);
    assert_eq!(plus_one_counters(&game, &goblins), vec![1; 4]);
    assert!(goblins.iter().all(|id| game.battlefield[id].tapped), "each read 3 power, its counter in the frame");
}

/// A player's answer makes two entries of a run unequal, so each is read by
/// what its own proposal says: only the Goblin given the counter has the power
/// the second effect asks for.
#[test]
fn a_token_whose_proposal_differs_is_not_served_its_runs_frame() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, glorious_anthem(), 0);
    let welcome = put_replacement(&mut game, entry_at_power(2, plus_one_counter()).optional());
    put_replacement(&mut game, entry_at_power(3, EnterModsTemplate::tapped()));

    let dp = ScriptedDecisionProvider::new();
    let asked = ChoiceKind::ApplyOptionalReplacement { affected_object: None, source: welcome };
    dp.expect_choice(asked.clone(), vec![]);
    dp.expect_choice(asked.clone(), vec![ChoiceOption::Object(welcome)]);
    dp.expect_choice(asked, vec![]);
    let create = GameAction::CreateTokens { defs: vec![goblin(); 3], controller: 0, attacking: None };
    game.execute_action(create, &ActionContext::new(&dp)).expect("the creation performs");

    let goblins = tokens_owned_by(&game, 0);
    assert_eq!(plus_one_counters(&game, &goblins), vec![0, 1, 0]);
    let tapped: Vec<bool> = goblins.iter().map(|id| game.battlefield[id].tapped).collect();
    assert_eq!(tapped, vec![false, true, false]);
}

/// CR 616.1b's rewrite is a proposal's controller, and the anthem is P0's: the
/// Goblin given to P1 enters a 1/1 beside a run of P0's 2/2s.
#[test]
fn a_token_entering_under_another_player_is_read_under_that_player() {
    let mut game = setup_game(2);
    game.enable_audit();
    put_on_battlefield(&mut game, glorious_anthem(), 0);
    let to_the_opponent = Rewrite::EnterUnderControlOf(PlayerRef::Opponent);
    let defection = ReplacementDef::new(
        EventPattern::EnterBattlefield { cast: None },
        ObjectSet::battlefield_filter(ObjectFilter::All),
        to_the_opponent,
    );
    let defects = put_replacement(&mut game, defection.optional());
    put_replacement(&mut game, entry_at_power(2, plus_one_counter()));

    let dp = ScriptedDecisionProvider::new();
    let asked = ChoiceKind::ApplyOptionalReplacement { affected_object: None, source: defects };
    dp.expect_choice(asked.clone(), vec![]);
    dp.expect_choice(asked.clone(), vec![ChoiceOption::Object(defects)]);
    dp.expect_choice(asked, vec![]);
    let create = GameAction::CreateTokens { defs: vec![goblin(); 3], controller: 0, attacking: None };
    game.execute_action(create, &ActionContext::new(&dp)).expect("the creation performs");

    assert_eq!(plus_one_counters(&game, &tokens_owned_by(&game, 0)), vec![1, 0, 1]);
}

/// Three runs, the last equal to the first: a run's frame goes with its card,
/// and the Plant's power is its own.
#[test]
fn each_run_of_a_creation_is_read_as_its_own_definition() {
    let plant = TokenDef::creature(0, 1, &[Color::Green], &[CreatureType::Plant]);
    let mut game = audited_game();
    put_on_battlefield(&mut game, glorious_anthem(), 0);
    put_replacement(&mut game, entry_at_power(2, plus_one_counter()));

    create(&mut game, vec![goblin(), goblin(), plant.clone(), plant, goblin()], 0);

    assert_eq!(plus_one_counters(&game, &tokens_owned_by(&game, 0)), vec![1, 1, 0, 0, 1]);
}

/// A Mountain's mana ability, given by Blood Moon (CR 305.7), is derived on
/// its object, so a frame holding one is not another token's: served one, the
/// audit panics.
#[test]
fn a_frame_holding_an_ability_of_its_object_stays_that_objects() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, blood_moon(), 0);

    create(&mut game, vec![token_of_type(CardType::Land); 3], 0);

    let lands = tokens_owned_by(&game, 0);
    let mut ids: Vec<_> = lands.iter().map(|id| get_effective_abilities(&game, *id)[0].id).collect();
    ids.dedup();
    assert_eq!(ids.len(), 3, "each has an instance of its own");
}

/// CR 306.5b's "enters with" ability is derived on its object too, and it is
/// the frame that hands it to the pipeline. The audit is this test's proof,
/// since a frame served whole would still enter two planeswalkers. Their
/// loyalty is CR 704.5i's to read, and no check runs here.
#[test]
fn a_run_of_planeswalker_tokens_is_walked_token_by_token() {
    let mut game = audited_game();

    create(&mut game, vec![token_of_type(CardType::Planeswalker); 2], 0);

    assert_eq!(tokens_owned_by(&game, 0).len(), 2);
}

/// CR 304.4 and 111.5, asked once of a run and true of each token of it.
#[test]
fn a_run_of_instants_is_refused_whole() {
    let mut game = audited_game();
    let objects_before = game.objects.len();

    create(&mut game, vec![token_of_type(CardType::Instant); 3], 0);

    assert_eq!(game.objects.len(), objects_before, "CR 111.5: none is created");
}

/// A run's answer ends with the run, either way round.
#[test]
fn the_run_after_a_refused_one_asks_for_itself() {
    let mut game = audited_game();

    create(&mut game, vec![token_of_type(CardType::Instant), goblin(), goblin()], 0);
    assert_eq!(tokens_owned_by(&game, 0).len(), 2);

    create(&mut game, vec![goblin(), token_of_type(CardType::Instant)], 0);
    assert_eq!(tokens_owned_by(&game, 0).len(), 3);
}

/// The run's tokens are the anthem's creatures once they have entered.
#[test]
fn the_tokens_of_a_shared_frame_are_each_a_permanent_of_their_own() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, glorious_anthem(), 0);

    create(&mut game, vec![goblin(); 3], 0);

    let goblins = tokens_owned_by(&game, 0);
    assert_eq!(goblins.len(), 3);
    assert!(goblins.iter().all(|id| get_effective_power(&game, *id) == Some(2)));
}
