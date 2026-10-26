//! A6e's cleanup, `history-and-frame-records`: what a record keeps of an
//! object that is gone, and the history log's rows as every "this turn"
//! fact's one store (`plans/handoffs/item-6-audit.md`, pass 3's cleanup).

use mtgsim::cards::authoring::triggers::{becomes_blocked, becomes_blocked_by, blocks, triggered_ability, whenever};
use mtgsim::cards::phase_history_and_frame_records_cards::{fixture_first_creature_tithe, fixture_mourners_tithe};
use mtgsim::cards::phase_lh_cards::holy_strength;
use mtgsim::cards::phase_rd_cards::dark_sphere;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::event::{GameEvent, UnattachRoute};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_colors, get_effective_power};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_hand, put_in_library, put_on_battlefield, put_on_battlefield_this_turn, setup_game, setup_two_player_game, static_ability,
    test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    PlayerRef, AmountExpr, ColorChange, Duration, Effect, EffectRecipient, ObjectFilter, PlayerSet, Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::history::{HistoryCount, HistorySpan, ObjectFact, TurnFact};
use mtgsim::types::ids::{AbilityId, ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::triggers::{EventRole, Multiplicity, TriggerEvent, TriggerSubject};
use mtgsim::types::zones::{Zone, ZoneChangeAct, ZoneChangePattern, ZoneSet};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// An unattach, each end as it left (CR 603.10c)
// ---------------------------------------------------------------------------

/// A 2/2 of `player`'s enchanted with Holy Strength, a 3/4 while it is.
fn enchanted_creature(game: &mut GameState, player: PlayerId) -> (ObjectId, ObjectId) {
    let host = put_on_battlefield(game, vanilla_creature(2, 2, &[]), player);
    let aura = put_on_battlefield(game, holy_strength(), player);
    game.execute_action(GameAction::Attach { attachment: aura, host }, &ActionContext::new(&test_dp())).unwrap();
    assert_eq!(get_effective_power(game, host), Some(3));
    (host, aura)
}

/// The one unattach recorded since record `since`.
fn the_unattach(game: &GameState, since: usize) -> GameEvent {
    let recorded = game.recorded_events();
    let mut unattaches = recorded.events().skip(since).filter(|e| matches!(e, GameEvent::Unattached { .. }));
    let unattach = unattaches.next().expect("an unattach").clone();
    assert!(unattaches.next().is_none(), "one unattach for one pair");
    unattach
}

/// The creature and its Aura destroyed in one event: the unattach keeps
/// each end as it left, the host a 3/4 under its Aura, though the Aura's
/// own departure is the one that unattached them.
#[test]
fn an_unattach_keeps_both_ends_when_both_leave_in_one_event() {
    let mut game = setup_two_player_game();
    let (host, aura) = enchanted_creature(&mut game, 0);
    let since = game.recorded_events().len();
    let destroy = |object| GameAction::Destroy { object };
    game.execute_actions(vec![destroy(aura), destroy(host)], &ActionContext::new(&test_dp())).unwrap();

    let unattach = the_unattach(&game, since);
    assert!(matches!(unattach, GameEvent::Unattached { route: UnattachRoute::AttachmentLeft, .. }), "{unattach:?}");
    assert!(unattach.frame_of(aura, &game.lki_store).is_some(), "the end whose departure unattached them");
    let host_as_it_left = unattach.frame_of(host, &game.lki_store).expect("the host, which left in the same event");
    assert_eq!(host_as_it_left.characteristics().power, Some(3), "as it was, enchanted");
}

/// CR 800.4a: a player leaves a three-seat game with an enchanted creature.
/// Both leave in one event, and the unattach keeps both.
#[test]
fn an_unattach_keeps_both_ends_when_their_owner_leaves_the_game() {
    let mut game = setup_game(3);
    let (host, aura) = enchanted_creature(&mut game, 1);
    let since = game.recorded_events().len();
    game.execute_action(GameAction::PlayerLoses { player: 1 }, &ActionContext::new(&test_dp())).unwrap();

    let unattach = the_unattach(&game, since);
    assert!(unattach.frame_of(aura, &game.lki_store).is_some() && unattach.frame_of(host, &game.lki_store).is_some(), "{unattach:?}");
}

/// Only the host leaves: the Aura stays on the battlefield, so the unattach
/// keeps the host alone, and the Aura is read as it is.
#[test]
fn an_unattach_keeps_no_frame_of_an_end_that_stays() {
    let mut game = setup_two_player_game();
    let (host, aura) = enchanted_creature(&mut game, 0);
    let since = game.recorded_events().len();
    game.execute_action(GameAction::Destroy { object: host }, &ActionContext::new(&test_dp())).unwrap();

    let unattach = the_unattach(&game, since);
    assert!(unattach.frame_of(host, &game.lki_store).is_some());
    assert!(unattach.frame_of(aura, &game.lki_store).is_none(), "{unattach:?}");
}

// ---------------------------------------------------------------------------
// The object a departure from the game names (CR 800.4a, 603.6c)
// ---------------------------------------------------------------------------

/// Player 1 leaves a three-seat game with a 3/3, which leaves the
/// battlefield as it leaves the game (CR 603.6c). Player 0's watcher binds
/// that creature as it left, and gains its power.
#[test]
fn a_creature_that_left_the_game_is_its_triggers_object() {
    let mut game = setup_game(3);
    put_on_battlefield(&mut game, fixture_mourners_tithe(), 0);
    put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    game.execute_action(GameAction::PlayerLoses { player: 1 }, &ActionContext::new(&test_dp())).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&test_dp()).unwrap();
        game.perform_sba_and_triggers(&test_dp()).unwrap();
    }
    assert_eq!(game.players[0].life_total, 23);
}

/// CR 603.6c's qualifier: a leaves-the-battlefield ability triggers when "a
/// phased-in permanent leaves the game because its owner leaves the game".
/// Player 1 leaves with a phased-in 3/3 and a phased-out 5/5; the watcher
/// gains 3, for the one that was phased in.
#[test]
fn a_phased_out_permanent_leaving_with_its_owner_triggers_nothing() {
    let mut game = setup_game(3);
    put_on_battlefield(&mut game, fixture_mourners_tithe(), 0);
    put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    let phased_out = put_on_battlefield(&mut game, vanilla_creature(5, 5, &[]), 1);
    game.battlefield.get_mut(&phased_out).unwrap().phased_out = true;
    game.bump_layer_epoch();
    game.execute_action(GameAction::PlayerLoses { player: 1 }, &ActionContext::new(&test_dp())).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&test_dp()).unwrap();
        game.perform_sba_and_triggers(&test_dp()).unwrap();
    }
    assert_eq!(game.players[0].life_total, 23);
}

// ---------------------------------------------------------------------------
// Which moves carry a frame (CR 113.7a, 603.10a)
// ---------------------------------------------------------------------------

/// "Whenever a card is put into exile from your hand, you gain life equal to
/// its power."
fn fixture_exiles_tithe() -> std::sync::Arc<mtgsim::objects::card_data::CardData> {
    let exiled_from_hand = TriggerEvent::ZoneChange {
        subject: TriggerSubject::Filter(ObjectFilter::All),
        change: ZoneChangePattern { from: Some(Zone::Hand), to: Some(Zone::Exile), act: None },
        by: None,
        owner: None,
        multiplicity: Multiplicity::PerOccurrence,
    };
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringPower(EventRole::Object)), EffectRecipient::Player(PlayerRef::You));
    CardDataBuilder::new("Fixture Exile's Tithe")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(exiled_from_hand, gain)))
        .build()
}

/// A 3/3 exiled from a hand, then put onto the battlefield from exile while
/// the trigger that read it waits on the stack. "Its power" is the card's as
/// it last existed in exile (CR 113.7a, 608.2h), so the move to the
/// battlefield hands its frame to the trigger, as a move to the graveyard
/// does.
#[test]
fn a_card_put_onto_the_battlefield_from_exile_leaves_its_frame_with_a_trigger_that_reads_it() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_exiles_tithe(), 0);
    let card = put_in_hand(&mut game, vanilla_creature(3, 3, &[]), 0);
    game.change_zone(card, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(game.stack.len(), 1, "the trigger waits");
    game.change_zone(card, Zone::Battlefield, ZoneChangeAct::Put, &test_ctx()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(game.players[0].life_total, 23);
}

// ---------------------------------------------------------------------------
// A source a cost moved, as it last existed (CR 113.7a, 601.2h)
// ---------------------------------------------------------------------------

/// Dark Sphere's "{T}, Sacrifice this artifact": the {T} is paid before the
/// sacrifice, so the artifact last existed on the battlefield tapped (CR
/// 113.7a). Its activation's record and its departure's keep that one
/// answer.
#[test]
fn a_source_sacrificed_for_its_cost_is_tapped_on_its_activation_and_its_departure() {
    let mut game = setup_two_player_game();
    let sphere = put_on_battlefield(&mut game, dark_sphere(), 0);
    let since = game.recorded_events().len();
    game.activate_ability(0, sphere, 0, &test_dp()).unwrap();

    let recorded = game.recorded_events();
    let tapped_on = |kind: fn(&GameEvent) -> bool| {
        let record = recorded.events().skip(since).find(|e| kind(e)).expect("recorded");
        record.frame_of(sphere, &game.lki_store).and_then(|lki| lki.was_tapped())
    };
    let departure = tapped_on(|e| matches!(e, GameEvent::ZoneChange { from: Zone::Battlefield, .. }));
    let activation = tapped_on(|e| matches!(e, GameEvent::AbilityActivated { .. }));
    assert_eq!(departure, Some(true));
    assert_eq!(activation, Some(true), "the activation reads it as it left, after its {{T}}");
}

// ---------------------------------------------------------------------------
// "The first ... each turn" counts the arm's own kind (item 314)
// ---------------------------------------------------------------------------

/// A spell of `card_type` with mana cost {1}: a 1/1, or an instant whose
/// instruction gains no life.
fn one_drop(name: &'static str, card_type: CardType) -> std::sync::Arc<CardData> {
    let builder = CardDataBuilder::new(name).mana_cost(ManaCost::build(&[], 1)).card_type(card_type);
    match card_type {
        CardType::Creature => builder.power_toughness(1, 1).build(),
        _ => builder
            .ability(AbilityDef {
                rules_text: "You gain 0 life.".into(),
                is_characteristic_defining: false,
                activation_restriction: ActivationRestriction::None,
                id: AbilityId::UNASSIGNED,
                instances: Vec::new(),
                ability_type: AbilityType::Spell,
                costs: Vec::new(),
                effect: Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(0)), EffectRecipient::Player(PlayerRef::You)),
            })
            .build(),
    }
}

/// `card` cast by player 0 from exactly {1}, and resolved with what it
/// triggered.
fn cast_and_resolve(game: &mut GameState, card: std::sync::Arc<CardData>) {
    let id = put_in_hand(game, card, 0);
    game.players[0].mana_pool.add(ManaType::Colorless, 1);
    game.cast_spell(0, id, &ManaWindowStop::new(test_dp())).expect("castable from exactly its cost");
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&test_dp()).unwrap();
        game.perform_sba_and_triggers(&test_dp()).unwrap();
    }
}

/// An instant, then two creatures: the first creature is the first
/// creature spell this turn, and the second is not.
#[test]
fn the_first_spell_of_a_kind_each_turn_counts_only_that_kind() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_first_creature_tithe(), 0);
    cast_and_resolve(&mut game, one_drop("Fixture Instant", CardType::Instant));
    assert_eq!(game.players[0].life_total, 20, "an instant is no creature spell");
    cast_and_resolve(&mut game, one_drop("Fixture Creature", CardType::Creature));
    assert_eq!(game.players[0].life_total, 21, "the first creature spell, though not the first spell");
    cast_and_resolve(&mut game, one_drop("Fixture Second Creature", CardType::Creature));
    assert_eq!(game.players[0].life_total, 21, "the second creature spell");
}

// ---------------------------------------------------------------------------
// The "this turn" facts the history's rows and the permanent answer (item 279)
// ---------------------------------------------------------------------------

/// "Cards in graveyards that [`fact`] this turn are black."
fn fixture_mourning_ink(fact: ObjectFact) -> std::sync::Arc<CardData> {
    CardDataBuilder::new("Fixture Mourning Ink")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeColor(ColorChange::Add(Color::Black), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredObjectsIn(ObjectFilter::ThisTurn(fact), ZoneSet::GRAVEYARD),
        )))
        .build()
}

fn is_black(game: &GameState, id: ObjectId) -> bool {
    get_effective_colors(game, id).contains(&Color::Black)
}

/// "Died this turn" is the card a creature's death made (CR 700.4, 400.7):
/// a creature destroyed, not a creature card discarded, and not the same
/// card once it has moved again.
#[test]
fn died_this_turn_is_the_card_its_death_made() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_mourning_ink(ObjectFact::Died), 0);
    let died = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let discarded = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.execute_action(GameAction::Destroy { object: died }, &test_ctx()).unwrap();
    game.change_zone(discarded, Zone::Graveyard, ZoneChangeAct::Discarded, &test_ctx()).unwrap();
    assert!(is_black(&game, died), "the card its death put there");
    assert!(!is_black(&game, discarded), "a card discarded did not die");

    game.change_zone(died, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).unwrap();
    game.change_zone(died, Zone::Graveyard, ZoneChangeAct::Put, &test_ctx()).unwrap();
    assert!(!is_black(&game, died), "a new object, which died never (CR 400.7)");
}

/// "Milled this turn", "discarded this turn": the act its move was (CR
/// 701.17, 701.9), asked of the card the move made.
#[test]
fn milled_and_discarded_this_turn_read_the_act_of_the_move() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_mourning_ink(ObjectFact::PutHere { from: None, act: Some(ZoneChangeAct::Milled) }), 0);
    let milled = put_in_library(&mut game, vanilla_creature(2, 2, &[]), 0);
    let discarded = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.change_zone(milled, Zone::Graveyard, ZoneChangeAct::Milled, &test_ctx()).unwrap();
    game.change_zone(discarded, Zone::Graveyard, ZoneChangeAct::Discarded, &test_ctx()).unwrap();
    assert!(is_black(&game, milled));
    assert!(!is_black(&game, discarded));
}

/// "Creatures [that did `fact`] this turn get +1/+0."
fn banner_over(fact: ObjectFact) -> std::sync::Arc<CardData> {
    CardDataBuilder::new("Fixture Banner")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(0), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(ObjectFilter::ThisTurn(fact)),
        )))
        .build()
}

/// Rock Jockey's "if this creature was cast this turn": the permanent a
/// spell became, which keeps the spell's facts (CR 400.7d). A creature put
/// onto the battlefield this turn was not cast.
#[test]
fn was_cast_this_turn_is_a_permanent_its_spell_became() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, banner_over(ObjectFact::WasCast), 0);
    let put = put_on_battlefield_this_turn(&mut game, vanilla_creature(2, 2, &[]), 0);
    cast_and_resolve(&mut game, one_drop("Fixture Creature", CardType::Creature));
    let cast = game
        .battlefield_ids_ordered()
        .into_iter()
        .find(|id| game.objects[id].card_data.name == "Fixture Creature")
        .expect("it resolved");
    assert_eq!(get_effective_power(&game, cast), Some(2), "a 1/1 cast this turn");
    assert_eq!(get_effective_power(&game, put), Some(2), "a 2/2 put onto the battlefield, not cast");
}

/// Erg Raiders' "unless it came under your control this turn": entering
/// this turn starts its control (CR 302.6's clock), and a permanent there
/// since before the turn has not come under anyone's control this turn.
#[test]
fn came_under_control_this_turn_starts_with_the_entry() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, banner_over(ObjectFact::CameUnderControl), 0);
    let old = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let new = put_on_battlefield_this_turn(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert_eq!(get_effective_power(&game, old), Some(2));
    assert_eq!(get_effective_power(&game, new), Some(3));
}

// ---------------------------------------------------------------------------
// A departure row keeps what the store forgets (item 286)
// ---------------------------------------------------------------------------

/// "You gain 1 life for each [`of`] creature you controlled that died this
/// turn", an instant costing {1}.
fn fixture_count_of_the_dead(name: &'static str, of: ObjectFilter) -> std::sync::Arc<CardData> {
    let died = HistoryCount { players: PlayerSet::You, fact: TurnFact::ControlledCreaturesDied, of: Some(Box::new(of)) };
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[], 1))
        .card_type(CardType::Instant)
        .ability(AbilityDef {
            rules_text: "".into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Spell,
            costs: Vec::new(),
            effect: Effect::Atom(
                Primitive::GainLife(AmountExpr::History(HistorySpan::ThisTurn, Box::new(died))),
                EffectRecipient::Player(PlayerRef::You),
            ),
        })
        .build()
}

/// Player 0's 2/2 and Goblin token both destroyed.
fn a_creature_and_a_token_died() -> GameState {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let create = GameAction::CreateTokens { defs: vec![mtgsim::cards::phase_re_cards::goblin_token()], controller: 0, attacking: None };
    game.execute_action(create, &test_ctx()).unwrap();
    let goblin = *game.battlefield_ids_ordered().iter().find(|&&id| game.objects[&id].is_token).expect("the Goblin");
    game.execute_actions(vec![GameAction::Destroy { object: bears }, GameAction::Destroy { object: goblin }], &test_ctx()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert!(!game.objects.contains_key(&goblin), "the token ceased to exist (CR 704.5d)");
    game
}

/// Caller of the Claw's "for each nontoken creature put into your graveyard
/// from the battlefield this turn": the 2/2, and not the Goblin.
#[test]
fn a_count_of_nontoken_creatures_that_died_reads_whether_each_was_a_token() {
    let mut game = a_creature_and_a_token_died();
    cast_and_resolve(&mut game, fixture_count_of_the_dead("Fixture Nontoken Tally", ObjectFilter::Not(Box::new(ObjectFilter::Token))));
    assert_eq!(game.players[0].life_total, 21);
}

/// Its twin, "for each token creature that died this turn": the Goblin,
/// which the store lost once it died, known by its departure's row.
#[test]
fn a_count_of_tokens_that_died_reads_the_token_the_store_has_lost() {
    let mut game = a_creature_and_a_token_died();
    cast_and_resolve(&mut game, fixture_count_of_the_dead("Fixture Token Tally", ObjectFilter::Token));
    assert_eq!(game.players[0].life_total, 21);
}

// ---------------------------------------------------------------------------
// An effect that makes a block (item 278, CR 509.1h, 509.3c)
// ---------------------------------------------------------------------------

/// "Whenever `event`, you gain `life` life."
fn fixture_watcher(name: &'static str, event: impl Into<TriggerEvent>, life: u64) -> std::sync::Arc<CardData> {
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(life)), EffectRecipient::Player(PlayerRef::You));
    CardDataBuilder::new(name).card_type(CardType::Enchantment).ability(triggered_ability("", whenever(event, gain))).build()
}

/// Curtain of Light's instruction, "target unblocked attacking creature
/// becomes blocked", resolved by player 1 at `attacker`.
fn becomes_blocked_by_an_effect(game: &mut GameState, attacker: ObjectId) {
    let curtain = put_in_hand(game, CardDataBuilder::new("Fixture Curtain").card_type(CardType::Instant).build(), 1);
    let ctx = ResolutionContext {
        source: curtain,
        ability_source: None,
        controller: 1,
        generated_by: mtgsim::events::cause::GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ObjectOrPlayerId::Object(attacker)]),
        replaced_event: None,
        trigger: None,
    };
    let effect = Effect::Atom(Primitive::BecomeBlocked, EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)));
    game.resolve_effect(&effect, &ctx, &test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(&test_dp()).unwrap();
        game.perform_sba_and_triggers(&test_dp()).unwrap();
    }
}

/// An attacker that becomes blocked by an effect triggers "whenever a
/// creature becomes blocked" (CR 509.3c), and neither "becomes blocked by a
/// creature" (509.3d) nor "blocks", since no creature blocks it. Made
/// blocked again, it was not unblocked, so nothing triggers; and it was
/// blocked this turn.
#[test]
fn an_effect_that_makes_a_block_triggers_becomes_blocked_and_nothing_a_blocker_reads() {
    let mut game = setup_two_player_game();
    let creature = || TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature));
    put_on_battlefield(&mut game, fixture_watcher("Fixture Blocked Watcher", becomes_blocked(creature()), 1), 0);
    put_on_battlefield(&mut game, fixture_watcher("Fixture Blocker Watcher", becomes_blocked_by(creature(), creature()), 10), 0);
    put_on_battlefield(&mut game, fixture_watcher("Fixture Blocks Watcher", blocks(creature()), 100), 0);
    put_on_battlefield(&mut game, banner_over(ObjectFact::WasBlocked), 0);
    let attacker = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    mtgsim::test_support::set_attacking(&mut game, attacker, 1);

    becomes_blocked_by_an_effect(&mut game, attacker);
    assert_eq!(game.players[0].life_total, 21);
    assert!(game.battlefield[&attacker].attacking.as_ref().is_some_and(|attack| attack.is_blocked && attack.blocked_by.is_empty()));
    assert_eq!(get_effective_power(&game, attacker), Some(3), "it was blocked this turn");

    becomes_blocked_by_an_effect(&mut game, attacker);
    assert_eq!(game.players[0].life_total, 21, "already blocked, it becomes nothing (CR 603.2e)");
}
