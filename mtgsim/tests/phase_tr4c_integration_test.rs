//! TR-4c — the other two look-back classes and the zone statement
//! (`triggers-architecture.md` §12, TR-4c).
//!
//! 1. `codebase-state.md` item 173: an intervening "if" that states its zone
//!    (CR 113.6b) is where the ability works, so Bridge from Below triggers
//!    from a graveyard and nowhere else. Its four rulings, and item 167's
//!    Yixlid Jailer board.
//! 2. Item 259: an object that arrived where it is in the event answers none
//!    of its look-back triggers (Bridge from Below's third ruling).
//! 3. Item 15: CR 603.10a's second and third classes. A card leaving a
//!    graveyard and a visible object put into a hand or library carry the
//!    frame from before the move, its own trigger is asked off it, and a
//!    watcher reads the subject as it was. A player leaving the game takes
//!    their graveyard with them, and those cards leave a graveyard too.
//! 4. CR 603.2f at the instant a candidate's list is read: before the event
//!    for a frame, after it for anything still there.

use std::sync::Arc;

use mtgsim::cards::authoring::{leaves_the_battlefield, triggered_ability, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lj_cards::yixlid_jailer;
use mtgsim::cards::phase_tr4c_cards::bridge_from_below;
use mtgsim::engine::actions::GameAction;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    effect_ctx, put_in_exile, put_in_graveyard, put_in_library, put_on_battlefield, put_on_battlefield_under, setup_game,
    setup_two_player_game, static_ability, test_ctx, test_dp,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{AmountExpr, ColorChange, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::triggers::{Multiplicity, TriggerEvent, TriggerSubject, TriggerTier};
use mtgsim::types::zones::{Zone, ZoneChangeAct, ZoneChangePattern, ZoneSet};
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

/// Move `object` from `from` to `to` as an effect would.
fn move_card(game: &mut GameState, object: ObjectId, from: Zone, to: Zone) {
    let by = mtgsim::types::ids::new_object_id();
    game.execute_action(GameAction::ZoneChange { object, from, to, act: ZoneChangeAct::Put }, &effect_ctx(game, by)).unwrap();
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

/// "Whenever [subject] moves `change`, you gain 1 life": an enchantment with
/// that one ability.
fn watcher(name: &str, subject: TriggerSubject, change: ZoneChangePattern, owner: Option<PlayerRef>) -> Arc<CardData> {
    let arm = TriggerEvent::ZoneChange { subject, change, by: None, owner, multiplicity: Multiplicity::PerOccurrence };
    CardDataBuilder::new(name).card_type(CardType::Enchantment).ability(triggered_ability("", whenever(arm, gain_one()))).build()
}

/// A card with one ability about its own move, `change`.
fn mover(name: &str, card_type: CardType, change: ZoneChangePattern) -> Arc<CardData> {
    let arm = TriggerEvent::ZoneChange {
        subject: TriggerSubject::ThisObject,
        change,
        by: None,
        owner: None,
        multiplicity: Multiplicity::PerOccurrence,
    };
    let card = CardDataBuilder::new(name).card_type(card_type);
    let card = if card_type == CardType::Creature { card.power_toughness(2, 2) } else { card };
    card.ability(triggered_ability("", whenever(arm, gain_one()))).build()
}

fn change(from: Option<Zone>, to: Option<Zone>) -> ZoneChangePattern {
    ZoneChangePattern { from, to, act: None }
}

/// **Fixture.** "When this card leaves your graveyard, you gain 1 life."
fn leaves_your_graveyard() -> Arc<CardData> {
    mover("Fixture Grave Leaver", CardType::Creature, change(Some(Zone::Graveyard), None))
}

/// **Fixture**, Golgari Brownscale's trigger on a creature with no dredge,
/// which does not fit `Rewrite` (`triggers-architecture.md` §15 item 5):
/// "When this card is put into your hand from your graveyard, you gain 1
/// life."
fn golgari_brownscale_shaped() -> Arc<CardData> {
    mover("Fixture Brownscale", CardType::Creature, change(Some(Zone::Graveyard), Some(Zone::Hand)))
}

/// **Fixture.** "Each card in [zones] is black": a Layer 5 effect over a zone
/// a card can leave, so the card is black as it was there and white wherever
/// it goes.
fn painted_black(zones: ZoneSet) -> Arc<CardData> {
    let black = std::collections::HashSet::from([Color::Black]);
    CardDataBuilder::new("Fixture Pall")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeColor(ColorChange::Set(black), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredObjectsIn(ObjectFilter::All, zones),
        )))
        .build()
}

fn white_card() -> Arc<CardData> {
    CardDataBuilder::new("White Card").color(Color::White).card_type(CardType::Creature).power_toughness(1, 1).build()
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

/// Item 259: the card reaching the graveyard in the event that killed the
/// creatures was not there immediately before it, and a leaves-the-battlefield
/// ability looks back to then (CR 603.10a). In either batch order: one window
/// is one event, so a record after the card's own move is no later event.
// RULING: Bridge from Below #3 - "If Bridge from Below isn't in your graveyard immediately before a creature dies, its abilities won't trigger at all. If Bridge from Below isn't still in your graveyard as its first ability resolves, you won't create a token."
#[test]
fn bridge_from_below_put_into_the_graveyard_by_the_same_wipe_does_not_trigger() {
    for bridge_first in [true, false] {
        let mut game = setup_two_player_game();
        let bridge = put_on_battlefield(&mut game, bridge_from_below(), 0);
        let yours = put_on_battlefield(&mut game, grizzly_bears(), 0);
        let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);
        let order = if bridge_first { [bridge, yours, theirs] } else { [yours, theirs, bridge] };

        destroy_all(&mut game, &order);
        assert_eq!(zone(&game, bridge), Some(Zone::Graveyard));
        assert_eq!(waiting_from(&game, bridge), 0, "bridge_first: {bridge_first}");

        // In the graveyard now, it answers the next death.
        let next = put_on_battlefield(&mut game, grizzly_bears(), 0);
        destroy_all(&mut game, &[next]);
        assert_eq!(waiting_from(&game, bridge), 1, "bridge_first: {bridge_first}");
    }
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

// ---------------------------------------------------------------------------
// 3. A card leaving a graveyard, and a visible object put into a hand or
//    library (item 15, CR 603.10a)
// ---------------------------------------------------------------------------

/// The card's own trigger, asked off the frame its departure carries: in the
/// graveyard, where the ability works, and not where it went.
#[test]
fn a_card_leaving_your_graveyard_triggers_its_own_ability() {
    let mut game = setup_two_player_game();
    let card = put_in_graveyard(&mut game, leaves_your_graveyard(), 0);

    move_card(&mut game, card, Zone::Graveyard, Zone::Exile);
    assert_eq!(waiting_from(&game, card), 1);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 21, "its owner's trigger");
}

/// Yixlid Jailer takes the ability in the graveyard, which is where a
/// look-back trigger reads it, so the card that left exiled with its ability
/// back does not trigger.
#[test]
fn under_yixlid_jailer_a_card_leaving_your_graveyard_does_not_trigger() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, yixlid_jailer(), 1);
    let card = put_in_graveyard(&mut game, leaves_your_graveyard(), 0);

    move_card(&mut game, card, Zone::Graveyard, Zone::Exile);
    assert_eq!(waiting_from(&game, card), 0);
}

/// Golgari Brownscale's seventh ruling, on its trigger alone: any return
/// from the graveyard to the hand. A card drawn is not one.
// RULING: Golgari Brownscale #7 - "The first ability triggers when Golgari Brownscale returns to your hand from your graveyard for any reason, not just when you use its dredge ability."
#[test]
fn a_card_put_into_your_hand_from_your_graveyard_triggers_its_own_ability() {
    let mut game = setup_two_player_game();
    let card = put_in_graveyard(&mut game, golgari_brownscale_shaped(), 0);
    move_card(&mut game, card, Zone::Graveyard, Zone::Hand);
    assert_eq!(waiting_from(&game, card), 1);

    let mut game = setup_two_player_game();
    let card = put_in_library(&mut game, golgari_brownscale_shaped(), 0);
    move_card(&mut game, card, Zone::Library, Zone::Hand);
    assert_eq!(waiting_from(&game, card), 0, "from the library");

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, yixlid_jailer(), 1);
    let card = put_in_graveyard(&mut game, golgari_brownscale_shaped(), 0);
    move_card(&mut game, card, Zone::Graveyard, Zone::Hand);
    assert_eq!(waiting_from(&game, card), 0, "under Yixlid Jailer");
}

/// A watcher reads the card as it was in the graveyard (CR 603.10a's second
/// class): black there, white in exile.
#[test]
fn a_card_leaving_a_graveyard_is_read_as_it_was_there() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, painted_black(ZoneSet::GRAVEYARD), 1);
    let watching = put_on_battlefield(
        &mut game,
        watcher("Fixture Black Leaver Watch", ObjectFilter::ByColor(Color::Black).into(), change(Some(Zone::Graveyard), None), None),
        0,
    );
    let card = put_in_graveyard(&mut game, white_card(), 1);

    move_card(&mut game, card, Zone::Graveyard, Zone::Exile);
    assert_eq!(waiting_from(&game, watching), 1, "black as it was");
}

/// The third class: a card all players could see put into a hand, read as it
/// was where it was seen. Black in exile, white in the hand.
#[test]
fn a_card_put_into_a_hand_from_exile_is_read_as_it_was_in_exile() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, painted_black(ZoneSet::EXILE), 1);
    let watching = put_on_battlefield(
        &mut game,
        watcher("Fixture Black Returner Watch", ObjectFilter::ByColor(Color::Black).into(), change(None, Some(Zone::Hand)), None),
        0,
    );
    let card = put_in_exile(&mut game, white_card(), 1);

    move_card(&mut game, card, Zone::Exile, Zone::Hand);
    assert_eq!(waiting_from(&game, watching), 1, "black as it was");
}

/// A player leaving the game takes their graveyard with them (CR 800.4a),
/// and each card in it leaves a graveyard: Erebos's Titan's "whenever a
/// creature card leaves an opponent's graveyard", on a fixture. Nothing is
/// left to read afterwards, so only the frame can answer.
#[test]
fn creature_cards_leaving_the_game_from_an_opponents_graveyard_trigger_a_watcher() {
    let mut game = setup_game(3);
    let watching = put_on_battlefield(
        &mut game,
        watcher(
            "Fixture Titan Watch",
            ObjectFilter::ByType(CardType::Creature).into(),
            change(Some(Zone::Graveyard), None),
            Some(PlayerRef::Opponent),
        ),
        0,
    );
    put_in_graveyard(&mut game, grizzly_bears(), 1);
    put_in_graveyard(&mut game, grizzly_bears(), 1);
    put_in_graveyard(&mut game, CardDataBuilder::new("Fixture Enchantment Card").card_type(CardType::Enchantment).build(), 1);

    game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).unwrap();
    assert_eq!(waiting_from(&game, watching), 2);
}

// ---------------------------------------------------------------------------
// 4. CR 603.2f, asked when the list is read
// ---------------------------------------------------------------------------

/// A leaves-the-battlefield ability is read off the frame, immediately before
/// the event, when the creature was on the battlefield for all to see. Its
/// owner's hand, where it is now, is hidden, and that does not matter.
#[test]
fn a_creature_returned_to_its_owners_hand_triggers_its_leaves_the_battlefield_ability() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(
        &mut game,
        CardDataBuilder::new("Fixture Departer")
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .ability(triggered_ability("", whenever(leaves_the_battlefield(TriggerSubject::ThisObject), gain_one())))
            .build(),
        0,
    );

    move_card(&mut game, creature, Zone::Battlefield, Zone::Hand);
    assert_eq!(waiting_from(&game, creature), 1);
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
