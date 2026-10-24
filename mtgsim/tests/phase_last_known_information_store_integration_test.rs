//! A6e's cleanup, `last-known-information-store`: CR 113.7a's last known
//! information in one store, written by the move that ends each existence
//! and read by existence (`triggers-architecture.md` §3.11).

use std::sync::Arc;

use mtgsim::cards::alpha::counterspell;
use mtgsim::cards::authoring::{at_beginning_of, whenever, Whose};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::{EventCause, GeneratedBy};
use mtgsim::events::event::GameEvent;
use mtgsim::state::game_state::{GameState, StepType};
use mtgsim::state::history_log::RowFact;
use mtgsim::test_support::{
    put_in_command_zone, put_in_exile, put_in_graveyard, put_in_hand, put_in_library, put_on_battlefield,
    put_spell_on_stack, setup_game, setup_two_player_game, test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount};
use mtgsim::types::ids::{ObjectId, ObjectRef, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::triggers::{DelayedDuration, DelayedProvenance, DelayedTriggerTemplate, DelayedTurn, Referred, TriggerTurn};
use mtgsim::types::zones::{Zone, ZoneChangeAct};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

/// `player`'s Counterspell countering `spell` (CR 701.6a), resolved as an
/// effect.
fn counter(game: &mut GameState, spell: ObjectId, player: PlayerId) {
    let source = put_in_hand(game, counterspell(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(spell)]),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    let effect = Effect::Atom(Primitive::CounterSpell, EffectRecipient::Target(SelectionFilter::Spell, TargetCount::Exactly(1)));
    game.resolve_effect(&effect, &ctx, &test_dp()).expect("it resolves");
}

// ---------------------------------------------------------------------------
// The capture's two legs that no CR 603.10a class and no keeper reaches
// ---------------------------------------------------------------------------

/// CR 603.10e: a spell countered is framed as it leaves the stack by the
/// batch that moves it, though no keeper reads it, and the record of the
/// countering carries the store's own frame of it.
#[test]
fn a_countered_spell_is_in_the_store_as_it_left_the_stack() {
    let mut game = setup_two_player_game();
    let bears = put_in_hand(&mut game, grizzly_bears(), 0);
    game.players[0].mana_pool.add(ManaType::Green, 2);
    game.cast_spell(0, bears, &ManaWindowStop::new(test_dp())).expect("castable from exactly its cost");
    let spell = game.object_ref(bears).expect("on the stack");
    let before = game.recorded_events().len();

    counter(&mut game, bears, 1);

    let stored = game.last_known(spell).expect("framed as it left the stack");
    assert!(stored.entry().is_some(), "the stack object it was");
    let countered = game
        .recorded_events()
        .records_from(before)
        .iter()
        .find_map(|record| match &record.event {
            GameEvent::SpellCountered { lki, .. } => Some(lki.clone()),
            _ => None,
        })
        .expect("it was countered");
    assert_eq!(&countered, stored, "the record carries the store's frame");
}

/// CR 608.2h: the source an activation's own cost moves is read by the
/// activation's records as it was, from any zone, so the cost's batch frames
/// it though no keeper reads it: here a card discarded from its owner's hand,
/// as a mana ability with no stack entry would.
#[test]
fn a_cost_that_moves_its_own_source_frames_it_from_any_zone() {
    let mut game = setup_two_player_game();
    let card = put_in_hand(&mut game, grizzly_bears(), 0);
    let in_hand = game.object_ref(card).expect("in hand");
    let discard = GameAction::ZoneChange { object: card, from: Zone::Hand, to: Zone::Graveyard, act: ZoneChangeAct::Discarded };

    game.execute_action(discard, &ActionContext::paying_cost(&test_dp(), 0, card)).expect("discarded");

    assert_eq!(game.last_known(in_hand).map(|lki| lki.object()), Some(in_hand));
}

// ---------------------------------------------------------------------------
// The store's bound: fewer entries than the history log has rows, in any game
// ---------------------------------------------------------------------------

/// Every zone an object can be in.
const ZONES: [Zone; 7] = [Zone::Battlefield, Zone::Graveyard, Zone::Exile, Zone::Hand, Zone::Library, Zone::Command, Zone::Stack];

/// Every act a move proposal can carry but a cast, whose CR 601.2a move is
/// silent and no departure, and a draw, which is its own verb.
const ACTS: [ZoneChangeAct; 7] = [
    ZoneChangeAct::Destroyed,
    ZoneChangeAct::Sacrificed,
    ZoneChangeAct::Discarded,
    ZoneChangeAct::Milled,
    ZoneChangeAct::Put,
    ZoneChangeAct::Countered,
    ZoneChangeAct::PlayedAsLand,
];

/// A creature card of `owner`'s in `zone`, so that it can enter the
/// battlefield as well as leave it.
fn card_in(game: &mut GameState, zone: Zone, owner: PlayerId) -> ObjectId {
    let card = vanilla_creature(2, 2, &[]);
    match zone {
        Zone::Battlefield => put_on_battlefield(game, card, owner),
        Zone::Graveyard => put_in_graveyard(game, card, owner),
        Zone::Exile => put_in_exile(game, card, owner),
        Zone::Hand => put_in_hand(game, card, owner),
        Zone::Library => put_in_library(game, card, owner),
        Zone::Command => put_in_command_zone(game, card, owner),
        Zone::Stack => put_spell_on_stack(game, card, owner),
    }
}

/// A delayed trigger whose source is `object`: a keeper that reads it, so a
/// batch frames its move out of any zone (`capture_lki`).
fn read_by_a_keeper(game: &mut GameState, object: ObjectRef) {
    let template = DelayedTriggerTemplate {
        def: Arc::new(whenever(
            at_beginning_of(StepType::End, Whose::Each),
            Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
        )),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::NoneNamed,
        rules_text: "".into(),
    };
    let source_card = Arc::clone(&game.objects[&object.id].card_data);
    let provenance = DelayedProvenance {
        source: object,
        source_card,
        controller: 0,
        created_by: None,
        x_value: None,
        turn: TriggerTurn::Any,
        resolution: None,
        cause: EventCause::Harness,
    };
    game.create_delayed_trigger(&template, provenance, Referred::default()).expect("it is made");
}

/// The rows the history log holds about object `id`, any existence of it.
fn rows_about(game: &GameState, id: ObjectId) -> usize {
    game.history.log.rows_newest_first().filter(|row| row.object.is_some_and(|object| object.id == id)).count()
}

/// `id`'s move, already performed, framed and with a row of its own.
fn assert_framed_with_a_row(game: &GameState, existence: ObjectRef, rows_before: usize, what: &str) {
    assert!(game.last_known(existence).is_some(), "{what} was not framed");
    assert!(rows_about(game, existence.id) > rows_before, "{what} wrote no row for it");
}

/// The LKI store's write is `depart`'s alone, so every entry is a framed
/// departure; this asserts that every departure's records write a row for
/// its object, so the store never holds more entries than the history log
/// has rows, in any game (`triggers-architecture.md` §3.11, decision 3).
/// Each move a proposal can name is made once, framed by a keeper reading
/// the mover: from every zone, to every zone but the stack, by every act. A
/// move onto the battlefield is the case it pins: its zone change writes no
/// row, and its entry does.
#[test]
fn every_framed_departure_writes_a_row_for_its_object() {
    let mut moves = 0;
    for from in ZONES {
        for to in ZONES.into_iter().filter(|&to| to != from && to != Zone::Stack) {
            for act in ACTS {
                let mut game = setup_two_player_game();
                let id = card_in(&mut game, from, 0);
                let existence = game.object_ref(id).expect("it is there");
                read_by_a_keeper(&mut game, existence);
                let rows = rows_about(&game, id);

                let what = format!("{from:?} to {to:?} by {act:?}");
                game.change_zone(id, to, act, &test_ctx()).unwrap_or_else(|e| panic!("{what}: {e}"));

                assert_framed_with_a_row(&game, existence, rows, &what);
                moves += 1;
            }
        }
    }
    assert_eq!(moves, (6 * 5 + 6) * ACTS.len(), "each zone to each other, and the stack to each");
}

/// A draw, the one move its own verb makes, is framed with a row too.
#[test]
fn a_framed_draw_writes_a_row_for_its_card() {
    let mut game = setup_two_player_game();
    let id = card_in(&mut game, Zone::Library, 0);
    let existence = game.object_ref(id).expect("it is there");
    read_by_a_keeper(&mut game, existence);
    let rows = rows_about(&game, id);

    let drawn = game.draw_card(0, &test_ctx()).expect("a card to draw");

    assert_eq!(drawn, Some(id), "the top card");
    assert_framed_with_a_row(&game, existence, rows, "the draw");
}

/// CR 800.4a: a player leaving takes what they own from every zone, each a
/// departure of its own, framed here, and each writes a row. A row of an
/// object that leaves from anywhere but the battlefield names no object,
/// since the object store has dropped it by the time the row is drafted and
/// its record frames nothing (CR 603.6c), so the rows are counted by zone.
#[test]
fn a_player_leaving_writes_a_row_for_each_object_it_takes() {
    let mut game = setup_game(3);
    let leaving: Vec<ObjectRef> = ZONES
        .into_iter()
        .map(|zone| {
            let id = card_in(&mut game, zone, 1);
            let existence = game.object_ref(id).expect("it is there");
            read_by_a_keeper(&mut game, existence);
            existence
        })
        .collect();
    // ORDER-FREE: a count.
    let owned_in = |game: &GameState, zone: Zone| game.objects.iter().filter(|(_, object)| object.owner == 1 && object.zone == zone).count();
    let owned: Vec<usize> = ZONES.into_iter().map(|zone| owned_in(&game, zone)).collect();
    let rows_from = |game: &GameState, zone: Zone| {
        game.history.log.rows_newest_first().filter(|row| row.player == Some(1) && matches!(row.fact, RowFact::LeftTheGame { from } if from == zone)).count()
    };

    game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).expect("they leave");

    for (zone, owned) in ZONES.into_iter().zip(owned) {
        assert_eq!(rows_from(&game, zone), owned, "a row for each object leaving from the {zone:?}");
    }
    for existence in leaving {
        assert!(game.last_known(existence).is_some(), "{existence:?} was framed");
    }
}
