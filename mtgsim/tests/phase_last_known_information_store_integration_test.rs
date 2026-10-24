//! A6e's cleanup, `last-known-information-store`: CR 113.7a's last known
//! information in one store, written by the move that ends each existence
//! and read by existence (`triggers-architecture.md` §3.11).

use mtgsim::cards::alpha::counterspell;
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::events::event::GameEvent;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_in_hand, setup_two_player_game, test_dp};
use mtgsim::types::effects::{Effect, EffectRecipient, Primitive, SelectionFilter, TargetCount};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaType;
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
