//! TR-4c — the other two look-back classes and the zone statement
//! (`triggers-architecture.md` §12, TR-4c).
//!
//! 1. `codebase-state.md` item 173: an intervening "if" that states its zone
//!    (CR 113.6b) is where the ability works, so Bridge from Below triggers
//!    from a graveyard and nowhere else. Its four rulings, and item 167's
//!    Yixlid Jailer board.

use std::sync::Arc;

use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lj_cards::yixlid_jailer;
use mtgsim::cards::phase_tr4c_cards::bridge_from_below;
use mtgsim::engine::actions::GameAction;
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    effect_ctx, put_in_graveyard, put_on_battlefield, put_on_battlefield_under, setup_two_player_game, test_dp,
};
use mtgsim::types::ids::ObjectId;
use mtgsim::types::triggers::TriggerTier;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// One event destroying `objects`, in the order given (the batch's order).
fn destroy_all(game: &mut GameState, objects: &[ObjectId]) {
    let by = mtgsim::types::ids::new_object_id();
    let batch = objects.iter().map(|&object| GameAction::Destroy { object }).collect();
    game.execute_actions(batch, &effect_ctx(game, by)).unwrap();
}

/// Put every waiting trigger on the stack and resolve the stack, answering
/// every prompt with `dp`.
fn resolve_all(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(dp).unwrap();
        game.perform_sba_and_triggers(dp).unwrap();
    }
}

/// The triggers `source` has waiting to be put on the stack.
fn waiting_from(game: &GameState, source: ObjectId) -> usize {
    game.pending_triggers.iter().filter(|p| p.origin.source() == source).count()
}

fn zone(game: &GameState, object: ObjectId) -> Option<Zone> {
    game.objects.get(&object).map(|o| o.zone)
}

/// The tokens `player` controls on the battlefield.
fn tokens_of(game: &GameState, player: usize) -> usize {
    game.battlefield_ids_ordered()
        .into_iter()
        .filter(|id| game.objects[id].is_token && game.battlefield[id].controller == player)
        .count()
}

// ---------------------------------------------------------------------------
// 1. The zone statement (item 173): Bridge from Below
// ---------------------------------------------------------------------------

/// Item 173: "if this card is in your graveyard" states where the ability
/// works (CR 113.6b), so the card in its owner's graveyard is asked, and its
/// "if" holds there.
#[test]
fn bridge_from_below_in_a_graveyard_makes_a_zombie_when_your_creature_dies() {
    let mut game = setup_two_player_game();
    let bridge = put_in_graveyard(&mut game, bridge_from_below(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);

    destroy_all(&mut game, &[bears]);
    assert_eq!(waiting_from(&game, bridge), 1, "asked from the graveyard");
    resolve_all(&mut game, &test_dp());
    assert_eq!(tokens_of(&game, 0), 1, "a 2/2 black Zombie");
    assert_eq!(zone(&game, bridge), Some(Zone::Graveyard));
}

// RULING: Bridge from Below #1 - "While Bridge from Below is on the battlefield, it has no effect. It does something only if it's in a graveyard. Yes, this is a bit weird."
#[test]
fn bridge_from_below_on_the_battlefield_does_nothing() {
    let mut game = setup_two_player_game();
    let bridge = put_on_battlefield(&mut game, bridge_from_below(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);

    destroy_all(&mut game, &[bears, theirs]);
    assert_eq!(waiting_from(&game, bridge), 0);
    assert!(
        !game.trigger_sources.contains_key(&bridge),
        "CR 113.6b's \"only\": nothing on the battlefield is asked"
    );
}

/// The graveyard a creature goes to is its owner's, whoever controlled it.
// RULING: Bridge from Below #2 - "Neither ability cares who controlled the creature that died, only which graveyard it was put into. This means that if an opponent controls a creature you own, its death causes Bridge from Below's first ability to trigger, not its second."
#[test]
fn bridge_from_below_reads_whose_graveyard_not_who_controlled_the_creature() {
    let mut game = setup_two_player_game();
    let bridge = put_in_graveyard(&mut game, bridge_from_below(), 0);
    let yours_they_control = put_on_battlefield_under(&mut game, grizzly_bears(), 0, 1);

    destroy_all(&mut game, &[yours_they_control]);
    resolve_all(&mut game, &test_dp());
    assert_eq!(tokens_of(&game, 0), 1, "the first ability");
    assert_eq!(zone(&game, bridge), Some(Zone::Graveyard), "not the second");

    let theirs_you_control = put_on_battlefield_under(&mut game, grizzly_bears(), 1, 0);
    destroy_all(&mut game, &[theirs_you_control]);
    resolve_all(&mut game, &test_dp());
    assert_eq!(tokens_of(&game, 0), 1, "not the first");
    assert_eq!(zone(&game, bridge), Some(Zone::Exile), "the second");
}

/// The intervening "if" again at resolution (CR 603.4).
// RULING: Bridge from Below #3 - "If Bridge from Below isn't in your graveyard immediately before a creature dies, its abilities won't trigger at all. If Bridge from Below isn't still in your graveyard as its first ability resolves, you won't create a token."
#[test]
fn bridge_from_below_gone_from_the_graveyard_makes_no_token() {
    let mut game = setup_two_player_game();
    let bridge = put_in_graveyard(&mut game, bridge_from_below(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);

    destroy_all(&mut game, &[bears]);
    assert_eq!(waiting_from(&game, bridge), 1);
    game.execute_action(
        GameAction::ZoneChange {
            object: bridge,
            from: Zone::Graveyard,
            to: Zone::Exile,
            act: mtgsim::types::zones::ZoneChangeAct::Put,
        },
        &effect_ctx(&game, bears),
    )
    .unwrap();
    resolve_all(&mut game, &test_dp());
    assert_eq!(tokens_of(&game, 0), 0);
}

/// Both abilities from one event, ordered by their controller: the token
/// first, then the exile, or the exile first, and then no token.
// RULING: Bridge from Below #4 - "If a nontoken creature you control and a creature an opponent controls die at the same time, you choose the order in which Bridge from Below's triggered abilities resolve. You can create a token before you exile Bridge from Below."
#[test]
fn bridge_from_below_can_make_its_token_before_it_exiles_itself() {
    for token_first in [true, false] {
        let mut game = setup_two_player_game();
        let bridge = put_in_graveyard(&mut game, bridge_from_below(), 0);
        let yours = put_on_battlefield(&mut game, grizzly_bears(), 0);
        let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);

        destroy_all(&mut game, &[yours, theirs]);
        assert_eq!(waiting_from(&game, bridge), 2);
        // Options in trigger order, the token's first. The last put on the
        // stack resolves first.
        let dp = ScriptedDecisionProvider::new();
        let order = if token_first { vec![1, 0] } else { vec![0, 1] };
        dp.expect_ordering(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First }, order);
        resolve_all(&mut game, &dp);
        assert_eq!(tokens_of(&game, 0), usize::from(token_first), "token_first: {token_first}");
        assert_eq!(zone(&game, bridge), Some(Zone::Exile));
    }
}

/// Item 167's board: under Yixlid Jailer the card in the graveyard has no
/// abilities, and a wipe that takes Jailer gives them back only after the
/// event. A leaves-the-battlefield ability reads the existence of its
/// abilities before (CR 603.10a), so it does not trigger, in either order.
#[test]
fn bridge_from_below_under_yixlid_jailer_does_not_trigger_on_the_wipe_that_takes_jailer() {
    for jailer_first in [true, false] {
        let mut game = setup_two_player_game();
        let bridge = put_in_graveyard(&mut game, bridge_from_below(), 0);
        let jailer = put_on_battlefield(&mut game, yixlid_jailer(), 0);
        let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
        let order = if jailer_first { [jailer, bears] } else { [bears, jailer] };

        destroy_all(&mut game, &order);
        assert_eq!(waiting_from(&game, bridge), 0, "jailer_first: {jailer_first}");
    }
}

/// Its "from anywhere" twin: the same board with the zone change printed
/// "from anywhere" is no leaves-the-battlefield ability (CR 603.6c), so it
/// is read after the wipe, with Jailer gone, and triggers for each creature
/// card put into the graveyard: Jailer is one.
#[test]
fn a_from_anywhere_twin_under_yixlid_jailer_triggers_after_the_wipe_that_takes_jailer() {
    for jailer_first in [true, false] {
        let mut game = setup_two_player_game();
        let twin = put_in_graveyard(&mut game, from_anywhere_twin(), 0);
        let jailer = put_on_battlefield(&mut game, yixlid_jailer(), 0);
        let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
        let order = if jailer_first { [jailer, bears] } else { [bears, jailer] };

        destroy_all(&mut game, &order);
        assert_eq!(waiting_from(&game, twin), 2, "jailer_first: {jailer_first}");
    }
}

/// **Fixture.** Bridge from Below's first ability with "from the
/// battlefield" printed "from anywhere".
fn from_anywhere_twin() -> Arc<CardData> {
    use mtgsim::cards::authoring::{triggered_ability, whenever};
    use mtgsim::objects::card_data::CardDataBuilder;
    use mtgsim::types::card_types::CardType;
    use mtgsim::types::effects::{AmountExpr, Condition, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive};
    use mtgsim::types::triggers::{Multiplicity, TriggerDef, TriggerEvent, TriggerSubject};
    use mtgsim::types::zones::{ZoneChangePattern, ZoneSet};
    CardDataBuilder::new("Fixture Bridge from Anywhere")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability(
            "",
            TriggerDef {
                intervening_if: Some(Condition::SourceInZone(ZoneSet::GRAVEYARD)),
                ..whenever(
                    TriggerEvent::ZoneChange {
                        subject: TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature)),
                        change: ZoneChangePattern { from: None, to: Some(Zone::Graveyard), act: None },
                        by: None,
                        owner: Some(PlayerRef::You),
                        multiplicity: Multiplicity::PerOccurrence,
                    },
                    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
                )
            },
        ))
        .build()
}
