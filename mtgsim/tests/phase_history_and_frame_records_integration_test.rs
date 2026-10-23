//! A6e's cleanup, `history-and-frame-records`: what a record keeps of an
//! object that is gone, and the history log's rows as every "this turn"
//! fact's one store (`plans/handoffs/item-6-audit.md`, pass 3's cleanup).

use mtgsim::cards::authoring::triggers::{leaves_the_battlefield, triggered_ability, whenever};
use mtgsim::cards::phase_lh_cards::holy_strength;
use mtgsim::cards::phase_rd_cards::dark_sphere;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::events::event::{GameEvent, UnattachRoute};
use mtgsim::oracle::characteristics::get_effective_power;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{put_in_hand, put_on_battlefield, setup_game, setup_two_player_game, test_ctx, test_dp, vanilla_creature};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::types::ids::AbilityId;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::triggers::{TriggerCondition, TriggerDef, TriggerLimit};
use mtgsim::types::effects::PlayerRef;
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, ObjectFilter, Primitive};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::triggers::{EventRole, Multiplicity, TriggerEvent, TriggerSubject};
use mtgsim::types::zones::{Zone, ZoneChangeAct, ZoneChangePattern};

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
    assert!(unattach.frame_of(aura).is_some(), "the end whose departure unattached them");
    let host_as_it_left = unattach.frame_of(host).expect("the host, which left in the same event");
    assert_eq!(host_as_it_left.chars().power, Some(3), "as it was, enchanted");
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
    assert!(unattach.frame_of(aura).is_some() && unattach.frame_of(host).is_some(), "{unattach:?}");
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
    assert!(unattach.frame_of(host).is_some());
    assert!(unattach.frame_of(aura).is_none(), "{unattach:?}");
}

// ---------------------------------------------------------------------------
// The object a departure from the game names (CR 800.4a, 603.6c)
// ---------------------------------------------------------------------------

/// "Whenever a creature leaves the battlefield, you gain life equal to its
/// power": its power is the creature's as it last existed (CR 608.2h).
fn fixture_mourners_tithe() -> std::sync::Arc<mtgsim::objects::card_data::CardData> {
    let creature = TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature));
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringPower(EventRole::Object)), EffectRecipient::Controller);
    CardDataBuilder::new("Fixture Mourner's Tithe")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(leaves_the_battlefield(creature), gain)))
        .build()
}

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
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringPower(EventRole::Object)), EffectRecipient::Controller);
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
        record.frame_of(sphere).and_then(|lki| lki.was_tapped())
    };
    let departure = tapped_on(|e| matches!(e, GameEvent::ZoneChange { from: Zone::Battlefield, .. }));
    let activation = tapped_on(|e| matches!(e, GameEvent::AbilityActivated { .. }));
    assert_eq!(departure, Some(true));
    assert_eq!(activation, Some(true), "the activation reads it as it left, after its {{T}}");
}

// ---------------------------------------------------------------------------
// "The first ... each turn" counts the arm's own kind (item 314)
// ---------------------------------------------------------------------------

/// "Whenever you cast your first creature spell each turn, you gain 1
/// life." Kalamax's and Plan for All Outcomes' rulings: the first of the
/// spells of the kind it names this turn, whatever other spells came first.
fn fixture_first_creature_tithe() -> std::sync::Arc<CardData> {
    let first_creature = TriggerDef {
        condition: TriggerCondition::Event(TriggerEvent::CastsSpell {
            caster: Some(PlayerRef::You),
            spell: Some(ObjectFilter::ByType(CardType::Creature)),
        }),
        intervening_if: None,
        limit: Some(TriggerLimit::FirstTimeEachTurn),
        effect: Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller),
    };
    CardDataBuilder::new("Fixture First Creature's Tithe")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", first_creature))
        .build()
}

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
                effect: Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(0)), EffectRecipient::Controller),
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
