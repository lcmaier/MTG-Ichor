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
//! 5. Item 249: a trigger on the card itself leaving a hidden zone is
//!    asked where the card went (CR 603.10, 113.6k, 702.29c). Guerrilla
//!    Tactics' three rulings, under Yixlid Jailer and under Rest in Peace.
//! 6. Item 260: an Aura's trigger on its creature dying finds the card the
//!    Aura became in its owner's graveyard, after CR 704.5m's move or in the
//!    same event (CR 603.6e, 400.7f). Angelic Destiny's two rulings, and
//!    Rancor's one.
//! 7. Item 266: a card whose every trigger is on itself moving is asked only
//!    by the window that moved it, so it keeps no other window past the gate.

use std::sync::Arc;

use mtgsim::cards::authoring::{leaves_the_battlefield, triggered_ability, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lj_cards::yixlid_jailer;
use mtgsim::cards::phase_rb_cards::{leyline_of_the_void, rest_in_peace};
use mtgsim::cards::phase_re8_cards::mind_rot;
use mtgsim::cards::phase_tr4c_cards::{angelic_destiny, bridge_from_below, guerrilla_tactics, rancor};
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::{ChosenTargets, TargetInstance};
use mtgsim::events::cause::{EventCause, GeneratedBy};
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    effect_ctx, put_in_exile, put_in_graveyard, put_in_hand, put_in_library, put_on_battlefield, put_on_battlefield_under,
    put_spell_on_stack, setup_game, setup_two_player_game, static_ability, test_ctx, test_dp, RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{Antecedent, AmountExpr, ColorChange, Duration, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
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

/// A card whose one ability triggers on the card itself moving `change`
/// (`TriggerSubject::Object(Antecedent::ThisObject)`).
fn mover(name: &str, card_type: CardType, change: ZoneChangePattern) -> Arc<CardData> {
    let arm = TriggerEvent::ZoneChange {
        subject: TriggerSubject::Object(Antecedent::ThisObject),
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
/// which does not fit `Rewrite` (dredge is `backlog.md` §2.49):
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
/// is one event, so a record after the card's move to the graveyard is no
/// later event.
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

/// The same board with Bridge's trigger printed "put into your graveyard
/// from anywhere" instead of "from the battlefield", which separates a
/// look-back trigger from one read after the event. CR 603.6c: a "from
/// anywhere" trigger is never a leaves-the-battlefield ability, so it does not
/// look back. It is checked after the wipe, when Yixlid Jailer has left the
/// battlefield and the card in the graveyard has its ability again, and it
/// triggers once for each creature card the wipe put into its owner's
/// graveyard: the Bears and Jailer itself, so twice. Bridge, which looks back
/// to the board with Jailer still on it, triggers for neither (the test
/// above).
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

/// Golgari Brownscale's seventh ruling, on its trigger alone: "The first
/// ability triggers when Golgari Brownscale returns to your hand from your
/// graveyard for any reason, not just when you use its dredge ability." A
/// card drawn is not one, and under Yixlid Jailer the card has no ability in
/// the graveyard it is read from.
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

/// Erebos's Titan's "whenever a creature card leaves an opponent's
/// graveyard", on a fixture, and its third ruling: "In a multiplayer game,
/// the last ability won't trigger when a player with a creature card in their
/// graveyard leaves the game." Leaving the game is CR 603.6c's event for a
/// leaves-the-battlefield ability and nothing else. A creature card exiled
/// from the same graveyard first shows the watcher works.
#[test]
fn creature_cards_leaving_the_game_from_an_opponents_graveyard_trigger_nothing() {
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
    let exiled = put_in_graveyard(&mut game, grizzly_bears(), 1);
    put_in_graveyard(&mut game, grizzly_bears(), 1);
    put_in_graveyard(&mut game, grizzly_bears(), 1);

    move_card(&mut game, exiled, Zone::Graveyard, Zone::Exile);
    assert_eq!(waiting_from(&game, watching), 1, "a creature card exiled from the graveyard");
    game.execute_action(GameAction::PlayerLoses { player: 1 }, &test_ctx()).unwrap();
    assert_eq!(waiting_from(&game, watching), 1, "the two left with their owner, and nothing triggered");
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
            .ability(triggered_ability("", whenever(leaves_the_battlefield(TriggerSubject::Object(Antecedent::ThisObject)), gain_one())))
            .build(),
        0,
    );

    move_card(&mut game, creature, Zone::Battlefield, Zone::Hand);
    assert_eq!(waiting_from(&game, creature), 1);
}

// ---------------------------------------------------------------------------
// 5. A card leaving a hidden zone, its trigger on itself (item 249): Guerrilla Tactics
// ---------------------------------------------------------------------------

/// Resolve `effect` as `caster`'s spell targeting `target`, its source a card
/// in `caster`'s graveyard.
fn resolve_spell_at_player(game: &mut GameState, effect: &Effect, caster: PlayerId, target: PlayerId, dp: &dyn DecisionProvider) {
    let source = put_in_graveyard(game, mind_rot(), caster);
    let mut ctx = ResolutionContext::untargeted(source, caster, GeneratedBy::Spell);
    ctx.targets = ChosenTargets::one(vec![ObjectOrPlayerId::Player(target)]);
    game.resolve_effect(effect, &ctx, dp).expect("resolving");
}

/// P0 holds Guerrilla Tactics and one other card, and P1's Mind Rot makes P0
/// discard both.
fn discarded_by_mind_rot(game: &mut GameState) -> ObjectId {
    let tactics = put_in_hand(game, guerrilla_tactics(), 0);
    put_in_hand(game, grizzly_bears(), 0);
    let rot = mind_rot().abilities[0].effect.clone();
    resolve_spell_at_player(game, &rot, 1, 0, &test_dp());
    tactics
}

/// The life both players have lost.
fn damage_dealt(game: &GameState) -> i64 {
    40 - life(game, 0) - life(game, 1)
}

/// The trigger is asked in the graveyard, where the card went, and deals 4,
/// as the card's ability rather than a spell.
// RULING: Guerrilla Tactics #1 - "If it is discarded, the damage done by it is done by this card due to an ability and not by a spell."
#[test]
fn guerrilla_tactics_discarded_by_an_opponents_spell_deals_four_damage() {
    let mut game = setup_two_player_game();
    let tactics = discarded_by_mind_rot(&mut game);
    assert_eq!(zone(&game, tactics), Some(Zone::Graveyard));
    assert_eq!(waiting_from(&game, tactics), 1);

    resolve_all(&mut game, &RecordingDecisionProvider::picking(0));
    assert_eq!(damage_dealt(&game), 4);
    let dealt = game.recorded_events().records().iter().find_map(|r| match &r.event {
        GameEvent::DamageDealt { source, .. } => Some((source.id, r.stamp.cause)),
        _ => None,
    });
    let (source, cause) = dealt.expect("damage was dealt");
    assert_eq!(source, tactics, "dealt by the card");
    assert!(matches!(cause, EventCause::Effect { generated_by: GeneratedBy::TriggeredAbility, .. }), "{cause:?}");
}

// RULING: Guerrilla Tactics #2 - "The second ability will trigger even on an optional discard caused by an opponent."
#[test]
fn guerrilla_tactics_triggers_on_an_optional_discard_an_opponent_causes() {
    let mut game = setup_two_player_game();
    let tactics = put_in_hand(&mut game, guerrilla_tactics(), 0);
    let discard = mind_rot().abilities[0].effect.clone();
    let Effect::Atom(Primitive::Discard(_, chooser), recipient) = discard else { panic!("Mind Rot's shape") };
    let may_discard = Effect::Optional {
        chooser: PlayerRef::Player(0),
        effect: Box::new(Effect::Atom(Primitive::Discard(AmountExpr::Fixed(1), chooser), recipient)),
    };
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::ApplyOptionalEffect { source: next_object(&game) }, vec![0]);
    resolve_spell_at_player(&mut game, &may_discard, 1, 0, &dp);

    assert_eq!(zone(&game, tactics), Some(Zone::Graveyard), "discarded by choice");
    assert_eq!(waiting_from(&game, tactics), 1);
}

/// The id the next object put into the game will have.
fn next_object(game: &GameState) -> ObjectId {
    let mut probe = game.clone();
    put_in_graveyard(&mut probe, grizzly_bears(), 0)
}

/// A cost is no spell or ability causing the discard ("costs aren't
/// effects"), nor is the cleanup discard, nor its owner's own spell.
// RULING: Guerrilla Tactics #3 - "Discarding as a cost to cast a spell will not trigger the ability. Only discarding as an effect will trigger the ability."
#[test]
fn guerrilla_tactics_discarded_as_a_cost_does_not_trigger() {
    let mut game = setup_two_player_game();
    let tactics = put_in_hand(&mut game, guerrilla_tactics(), 0);
    let paying_for = put_in_hand(&mut game, grizzly_bears(), 0);
    let dp = test_dp();
    game.execute_action(
        GameAction::ZoneChange { object: tactics, from: Zone::Hand, to: Zone::Graveyard, act: ZoneChangeAct::Discarded },
        &ActionContext::paying_cost(&dp, 0, paying_for),
    )
    .unwrap();
    assert_eq!(zone(&game, tactics), Some(Zone::Graveyard));
    assert_eq!(waiting_from(&game, tactics), 0, "a cost");

    let mut game = setup_two_player_game();
    let tactics = put_in_hand(&mut game, guerrilla_tactics(), 0);
    put_in_hand(&mut game, grizzly_bears(), 0);
    let rot = mind_rot().abilities[0].effect.clone();
    resolve_spell_at_player(&mut game, &rot, 0, 0, &test_dp());
    assert_eq!(zone(&game, tactics), Some(Zone::Graveyard));
    assert_eq!(waiting_from(&game, tactics), 0, "its owner's own spell");
}

/// Asked in the graveyard, where Yixlid Jailer takes the ability, as its
/// second ruling says of a cycling trigger: it does not trigger. A trigger read
/// in the hand it left would.
#[test]
fn guerrilla_tactics_discarded_under_yixlid_jailer_does_not_trigger() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, yixlid_jailer(), 1);
    let tactics = discarded_by_mind_rot(&mut game);
    assert_eq!(zone(&game, tactics), Some(Zone::Graveyard));
    assert_eq!(waiting_from(&game, tactics), 0);
}

/// Item 266: Guerrilla Tactics' one trigger is on itself being discarded, and
/// works in the library it starts the game in (where CR 603.2f refuses it).
/// Only the window that moves it can trigger it, so a death elsewhere passes
/// no dispatch past the gate; and its own discard still triggers it.
#[test]
fn a_card_whose_trigger_is_on_itself_moving_keeps_no_other_window_past_the_gate() {
    let mut game = setup_two_player_game();
    put_in_library(&mut game, guerrilla_tactics(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);

    let before = game.diagnostics.trigger_dispatch().windows;
    destroy_all(&mut game, &[bears]);
    assert_eq!(game.diagnostics.trigger_dispatch().windows, before, "the gate returned");

    let mut game = setup_two_player_game();
    let tactics = discarded_by_mind_rot(&mut game);
    assert_eq!(waiting_from(&game, tactics), 1, "the window that moved it asks it");
}

/// **Fixture**, Library of Leng's replacement made mandatory and an
/// opponent's: "If a spell or ability an opponent controls causes you to
/// discard a card, put it on top of your library instead."
fn leng_shaped() -> Arc<CardData> {
    use mtgsim::types::effects::ObjectSet;
    use mtgsim::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
    use mtgsim::types::restriction::SourceFilter;
    let discard = ZoneChangePattern { from: Some(Zone::Hand), to: None, act: Some(ZoneChangeAct::Discarded) };
    let yours = ObjectSet::filter_in(ObjectFilter::ByOwner(PlayerRef::You), ZoneSet::ALL);
    let def = ReplacementDef::new(
        EventPattern::ZoneChange { change: discard, object: None },
        yours,
        Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Library }),
    )
    .caused_by(SourceFilter::ControlledBy(PlayerRef::Opponent));
    CardDataBuilder::new("Fixture Leng").card_type(CardType::Artifact).ability(static_ability(Effect::Replacement(Box::new(def)))).build()
}

/// Library of Leng's board (`plans/atomic-tests/supplemental-docs/603-2f-complexity.md`):
/// the card goes from one hidden zone to another and is never visible to all
/// players, so CR 603.2f refuses the trigger. The library is a zone the
/// trigger works in; it is the card being hidden that refuses it, and a card
/// revealed there, under Future Sight, would trigger.
#[test]
fn guerrilla_tactics_put_on_its_library_instead_does_not_trigger() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, leng_shaped(), 0);
    let tactics = discarded_by_mind_rot(&mut game);
    assert_eq!(zone(&game, tactics), Some(Zone::Library));
    assert_eq!(waiting_from(&game, tactics), 0);
}

/// Under Rest in Peace the card winds up in exile, and is asked there (CR
/// 702.29c's "whatever zone the card winds up in").
#[test]
fn guerrilla_tactics_exiled_instead_by_rest_in_peace_still_triggers() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, rest_in_peace(), 1);
    let tactics = discarded_by_mind_rot(&mut game);
    assert_eq!(zone(&game, tactics), Some(Zone::Exile));
    assert_eq!(waiting_from(&game, tactics), 1);
}

// ---------------------------------------------------------------------------
// 6. An Aura's trigger finds the card it became (item 260): Angelic Destiny
// ---------------------------------------------------------------------------

/// `aura` on the battlefield under its owner `player`, enchanting a Grizzly
/// Bears of theirs. Returns the Aura and the creature.
fn enchanting_a_bear(game: &mut GameState, aura: Arc<CardData>, player: PlayerId) -> (ObjectId, ObjectId) {
    let bears = put_on_battlefield(game, grizzly_bears(), player);
    let aura = put_on_battlefield(game, aura, player);
    assert!(game.attach(aura, bears));
    (aura, bears)
}

/// ATOM-603.6e-001 and ATOM-400.7f-001's board: the creature dies, the Aura
/// stays on the battlefield unattached, and CR 704.5m puts it into its
/// owner's graveyard before its trigger resolves. CR 400.7f's second
/// sentence lets "this card" find it there.
// COVERS: ATOM-603.6e-001
// COVERS: ATOM-400.7f-001
#[test]
fn angelic_destiny_returns_to_its_owners_hand_after_its_own_state_based_move() {
    let mut game = setup_two_player_game();
    let (destiny, bears) = enchanting_a_bear(&mut game, angelic_destiny(), 0);
    assert_eq!(mtgsim::oracle::characteristics::get_effective_power(&game, bears), Some(6), "+4/+4");

    destroy_all(&mut game, &[bears]);
    assert_eq!(zone(&game, destiny), Some(Zone::Battlefield), "unattached, until the check");
    assert_eq!(waiting_from(&game, destiny), 1);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, destiny), Some(Zone::Hand), "CR 400.7f");
    assert!(game.players[0].hand.contains(&destiny), "its owner's hand");
}

/// CR 400.7f's first sentence: the Aura put into the graveyard in the same
/// event, in either batch order.
#[test]
fn angelic_destiny_destroyed_beside_its_creature_returns_to_its_owners_hand() {
    for aura_first in [true, false] {
        let mut game = setup_two_player_game();
        let (destiny, bears) = enchanting_a_bear(&mut game, angelic_destiny(), 0);
        let order = if aura_first { [destiny, bears] } else { [bears, destiny] };

        destroy_all(&mut game, &order);
        assert_eq!(waiting_from(&game, destiny), 1, "aura_first: {aura_first}");
        resolve_all(&mut game, &test_dp());
        assert_eq!(zone(&game, destiny), Some(Zone::Hand), "aura_first: {aura_first}");
    }
}

/// The new object CR 400.7f finds is the one the Aura became in the
/// graveyard; once that card moves on, it is gone (CR 400.7).
// RULING: Angelic Destiny #2 - "If Angelic Destiny is no longer in a graveyard when its triggered ability resolves, it won't be returned to its owner's hand."
#[test]
fn angelic_destiny_gone_from_the_graveyard_is_not_returned() {
    let mut game = setup_two_player_game();
    let (destiny, bears) = enchanting_a_bear(&mut game, angelic_destiny(), 0);
    destroy_all(&mut game, &[bears]);
    game.check_state_based_actions(&test_dp()).unwrap();
    assert_eq!(zone(&game, destiny), Some(Zone::Graveyard));

    move_card(&mut game, destiny, Zone::Graveyard, Zone::Exile);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, destiny), Some(Zone::Exile));
}

/// CR 400.7f finds the Aura in its owner's graveyard and nowhere else. Under
/// Leyline of the Void the opponent's creature dies into its own graveyard and
/// the trigger fires, but CR 704.5m's move of the Aura is replaced by an exile,
/// so the card is never in the graveyard and stays in exile. The CR's text,
/// against a reading that would find the card in the first public zone it went
/// to: `engineering-practices.md#when-a-ruling-and-the-cr-disagree`'s
/// `aura-found-in-graveyard`.
#[test]
fn angelic_destiny_exiled_instead_of_its_state_based_move_is_not_returned() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, leyline_of_the_void(), 1);
    let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let destiny = put_on_battlefield(&mut game, angelic_destiny(), 0);
    assert!(game.attach(destiny, theirs));

    destroy_all(&mut game, &[theirs]);
    assert_eq!(zone(&game, theirs), Some(Zone::Graveyard), "its owner's graveyard, which Leyline does not touch");
    assert_eq!(waiting_from(&game, destiny), 1);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, destiny), Some(Zone::Exile), "never put into the graveyard, so not found");
}

/// The same board's other half (`aura-found-in-graveyard`): the Aura and the
/// creature destroyed in one event, the Aura exiled instead, in either batch
/// order. CR 400.7f's first sentence also names the graveyard.
#[test]
fn angelic_destiny_exiled_beside_its_creature_is_not_returned() {
    for aura_first in [true, false] {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, leyline_of_the_void(), 1);
        let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);
        let destiny = put_on_battlefield(&mut game, angelic_destiny(), 0);
        assert!(game.attach(destiny, theirs));
        let order = if aura_first { [destiny, theirs] } else { [theirs, destiny] };

        destroy_all(&mut game, &order);
        assert_eq!(zone(&game, destiny), Some(Zone::Exile), "aura_first: {aura_first}");
        assert_eq!(waiting_from(&game, destiny), 1, "aura_first: {aura_first}");
        resolve_all(&mut game, &test_dp());
        assert_eq!(zone(&game, destiny), Some(Zone::Exile), "not found, aura_first: {aura_first}");
    }
}

/// The Aura spell whose target died never resolves, so it is never on the
/// battlefield to see a creature die (CR 608.3b).
// RULING: Angelic Destiny #1 - "If the creature dies before the Angelic Destiny spell resolves, Angelic Destiny will go to its owner's graveyard. It won't return to its owner's hand."
#[test]
fn angelic_destiny_whose_creature_died_before_it_resolved_stays_in_the_graveyard() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let destiny = aura_spell_targeting(&mut game, angelic_destiny(), 0, bears);
    destroy_all(&mut game, &[bears]);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, destiny), Some(Zone::Graveyard));
    assert!(game.players[0].hand.is_empty());
}

// RULING: Rancor #1 - "If the creature this Aura would enchant is an illegal target by the time Rancor tries to resolve, the Aura spell doesn't resolve. It won't enter the battlefield, so it won't be put into a graveyard from the battlefield and its ability won't trigger."
#[test]
fn rancor_whose_creature_left_before_it_resolved_does_not_trigger() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let aura = aura_spell_targeting(&mut game, rancor(), 0, bears);
    destroy_all(&mut game, &[bears]);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, aura), Some(Zone::Graveyard));
    assert!(game.players[0].hand.is_empty(), "no trigger returned it");
}

/// Rancor's trigger on itself going to the graveyard: "it" is the card it
/// became there (CR 400.7e), after CR 704.5m put it there.
#[test]
fn rancor_returns_to_its_owners_hand_when_its_creature_dies() {
    let mut game = setup_two_player_game();
    let (aura, bears) = enchanting_a_bear(&mut game, rancor(), 0);
    destroy_all(&mut game, &[bears]);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, aura), Some(Zone::Hand));
}

/// `aura` on the stack as `controller`'s spell, its enchant clause targeting
/// `target`, as `cast_spell` leaves it.
fn aura_spell_targeting(game: &mut GameState, aura: Arc<CardData>, controller: PlayerId, target: ObjectId) -> ObjectId {
    let enchant = aura.spell_instances.last().cloned().expect("an Aura announces its enchant clause");
    let id = put_spell_on_stack(game, aura, controller);
    let chosen = TargetInstance::from_announcement(game, enchant, &[ObjectOrPlayerId::Object(target)]).unwrap();
    game.stack_entries.get_mut(&id).unwrap().chosen_targets = vec![chosen];
    id
}

/// **Fixture.** Bridge from Below's first ability with "from the
/// battlefield" printed "from anywhere".
fn from_anywhere_twin() -> Arc<CardData> {
    use mtgsim::cards::authoring::{functioning_in, triggered_ability, whenever};
    use mtgsim::objects::card_data::CardDataBuilder;
    use mtgsim::types::card_types::CardType;
    use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, ObjectFilter, PlayerRef, Primitive};
    use mtgsim::types::triggers::{Multiplicity, TriggerEvent, TriggerSubject};
    use mtgsim::types::zones::{ZoneChangePattern, ZoneSet};
    CardDataBuilder::new("Fixture Bridge from Anywhere")
        .card_type(CardType::Enchantment)
        .ability(functioning_in(
            ZoneSet::GRAVEYARD,
            triggered_ability(
                "",
                whenever(
                    TriggerEvent::ZoneChange {
                        subject: TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature)),
                        change: ZoneChangePattern { from: None, to: Some(Zone::Graveyard), act: None },
                        by: None,
                        owner: Some(PlayerRef::You),
                        multiplicity: Multiplicity::PerOccurrence,
                    },
                    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
                ),
            ),
        ))
        .build()
}
