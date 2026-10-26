//! What one event does to the objects it names, and the choices a
//! permanent's entry makes: devour's sacrifices, a shockland's cost (CR
//! 614.12b), tribute's opponent, and a trigger reading a creature its own
//! event killed.

use std::sync::Arc;

use mtgsim::cards::authoring::triggers::{counters_removed_from, dies, triggered_ability, whenever};
use mtgsim::cards::phase_rc_cards::{containment_priest, master_biomancer, thunder_thrash_elder};
use mtgsim::cards::phase_re_cards::soldier_token;
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeAct};
use mtgsim::events::event::{BatchId, GameEvent};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_graveyard, put_in_hand, put_on_battlefield, put_on_battlefield_under, setup_game, setup_two_player_game,
    static_ability, test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::costs::Cost;
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerRef, Primitive, TokenDef,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, ObjectOrPlayer, ObjectOrPlayerId, PlayerId};
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::types::replacement::{
    AmountRewrite, AuxiliaryMove, EnterModsTemplate, EntryCountersTemplate, EventPattern, GameActionTemplate, PerChosen,
    ReplacementDef,
    Rewrite,
};
use mtgsim::types::triggers::TriggerSubject;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption, EntryOffer, EntryRole};

fn add_counters(game: &mut GameState, object: ObjectId, counter: CounterType, n: u32) {
    game.execute_action(GameAction::AddCounters { subject: ObjectOrPlayerId::Object(object), counter, n, by: 0 }, &test_ctx())
        .expect("on the battlefield");
}

fn counters(game: &GameState, id: ObjectId, kind: CounterType) -> u32 {
    game.battlefield.get(&id).map_or(0, |entry| entry.counter_count(kind))
}

/// Every prompt `dp` answered was `expected`, player and question, in order.
/// Rendered to compare, since a question carries no `PartialEq`.
fn assert_asked(dp: &ScriptedDecisionProvider, expected: &[(PlayerId, ChoiceKind)]) {
    let render = |asked: &[(PlayerId, ChoiceKind)]| asked.iter().map(|(p, kind)| format!("P{p} {kind:?}")).collect::<Vec<_>>();
    assert_eq!(render(&dp.asked_of()), render(expected));
}

/// Where in the record `pred`'s first and last matches fall, after `mark`.
fn record_span(game: &GameState, mark: usize, pred: impl Fn(&GameEvent) -> bool) -> (usize, usize) {
    let seqs: Vec<usize> = game.recorded_events().records_from(mark).iter().filter(|r| pred(&r.event)).map(|r| r.seq.0).collect();
    (*seqs.first().expect("a match"), *seqs.last().expect("a match"))
}

/// Devour, "as this enters, you may sacrifice any number of creatures. It
/// enters with `per` +1/+1 counters for each" (CR 702.82a).
fn devour(per: PerChosen) -> Effect {
    Effect::Replacement(Box::new(ReplacementDef::new(
        EventPattern::EnterBattlefield { cast: None },
        ObjectSet::SourceOnly,
        Rewrite::EnterAfterMoving(AuxiliaryMove {
            from: Zone::Battlefield,
            filter: ObjectFilter::And(
                Box::new(ObjectFilter::ByType(CardType::Creature)),
                Box::new(ObjectFilter::ByController(PlayerRef::You)),
            ),
            to: Zone::Graveyard,
            act: ZoneChangeAct::Sacrificed,
            up_to: None,
            per_chosen: Some((CounterType::PlusOnePlusOne, per)),
        }),
    )))
}

fn devourer(name: &str, per: PerChosen) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .power_toughness(1, 1)
        .ability(static_ability(devour(per)))
        .build()
}

/// Enter `entering` from its graveyard, answering devour's prompt with
/// `picks`, and check `chooser` was the one asked.
fn reanimate_devouring(game: &mut GameState, entering: ObjectId, chooser: PlayerId, picks: Vec<usize>) {
    let prompt = || ChoiceKind::ChooseAuxiliaryZoneChange { entering, source: entering, to: Zone::Graveyard };
    let dp = test_dp();
    dp.expect_pick_n(prompt(), picks);
    game.change_zone(entering, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&dp)).expect("it enters");
    assert_asked(&dp, &[(chooser, prompt())]);
}

/// "Creatures have devour `n`", a Layer 6 grant.
fn grants_devour_to_creatures(n: u32) -> Arc<CardData> {
    let mut granted = static_ability(devour(PerChosen::Fixed(n)));
    granted.id = new_ability_id();
    CardDataBuilder::new("Fixture Devour Banner")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::GrantAbility(Arc::new(granted), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(ObjectFilter::ByType(CardType::Creature)),
        )))
        .build()
}

/// Dragon Broodmother's token: "a 1/1 red and green Dragon creature token
/// with flying and devour 2".
fn devouring_dragon_token() -> TokenDef {
    let mut token = soldier_token();
    token.abilities.push(static_ability(devour(PerChosen::Fixed(2))));
    token.rules_text = "Devour 2".into();
    token
}

/// "Whenever one or more creatures die, you gain 1 life" (CR 603.2c).
fn gains_when_creatures_die() -> Arc<CardData> {
    let arm = dies(TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature))).once_per_event();
    CardDataBuilder::new("Fixture Mourning Bell")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(arm, Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You)))))
        .build()
}

/// Corpsejack Menace's "If one or more +1/+1 counters would be put on a
/// creature you control, twice that many +1/+1 counters are put on it
/// instead".
fn doubles_your_plus_one_counters() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Counter Doubler")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::AddCounters { counter: Some(CounterType::PlusOnePlusOne), by: None },
            ObjectSet::battlefield_filter(ObjectFilter::And(
                Box::new(ObjectFilter::ByType(CardType::Creature)),
                Box::new(ObjectFilter::ByController(PlayerRef::You)),
            )),
            Rewrite::Amount(AmountRewrite::Multiplier(2)),
        )))))
        .build()
}

/// Two of the token above, created at once, the first devouring two
/// creatures and the second one. `placeholder` stands for each token in the
/// scripted prompt, which matches the prompt's kind and not its fields: the
/// tokens have no id until they are created.
fn create_two_devouring_dragons(game: &mut GameState, placeholder: ObjectId) -> Vec<ObjectId> {
    let dp = test_dp();
    let devour_prompt =
        ChoiceKind::ChooseAuxiliaryZoneChange { entering: placeholder, source: placeholder, to: Zone::Graveyard };
    dp.expect_pick_n(devour_prompt.clone(), vec![0, 1]);
    dp.expect_pick_n(devour_prompt, vec![0]);
    let before: Vec<ObjectId> = game.battlefield_ids_ordered();
    game.execute_action(
        GameAction::CreateTokens { defs: vec![devouring_dragon_token(), devouring_dragon_token()], controller: 0, attacking: None },
        &ActionContext::new(&dp),
    )
    .expect("the tokens are created");
    let dragons: Vec<ObjectId> = game.battlefield_ids_ordered().into_iter().filter(|id| !before.contains(id)).collect();
    let prompt = |id| ChoiceKind::ChooseAuxiliaryZoneChange { entering: id, source: id, to: Zone::Graveyard };
    assert_asked(&dp, &[(0, prompt(dragons[0])), (0, prompt(dragons[1]))]);
    dragons
}

// ---------------------------------------------------------------------------
// One entry event's sacrifices are one event, right before the entries
// ---------------------------------------------------------------------------

/// Two tokens with devour entering together sacrifice everything they devour
/// at the same moment (Thunder-Thrash Elder's ruling), so "whenever one or
/// more creatures die" triggers once, and the sacrifices are an event of
/// their own ahead of the entries (Wood Elemental's ruling).
#[test]
fn two_devour_tokens_entering_together_sacrifice_as_one_event() {
    let mut game = setup_two_player_game();
    let bell = put_on_battlefield(&mut game, gains_when_creatures_die(), 0);
    for _ in 0..3 {
        put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    }
    let mark = game.recorded_events().len();

    let dragons = create_two_devouring_dragons(&mut game, bell);

    assert_eq!(dragons.len(), 2);
    let mut given: Vec<u32> = dragons.iter().map(|&id| counters(&game, id, CounterType::PlusOnePlusOne)).collect();
    given.sort();
    assert_eq!(given, vec![2, 4], "devour 2 for one creature and for two");
    assert_eq!(game.players[0].graveyard.len(), 3);
    assert_eq!(game.pending_triggers.len(), 1, "one event of deaths, so one trigger (CR 603.2c)");

    let recorded = game.recorded_events();
    let batch_of = |pred: &dyn Fn(&GameEvent) -> bool| -> Vec<Option<BatchId>> {
        recorded.records_from(mark).iter().filter(|r| pred(&r.event)).map(|r| r.stamp.batch).collect()
    };
    let died = batch_of(&|e| matches!(e, GameEvent::ZoneChange { to: Zone::Graveyard, .. }));
    let entered = batch_of(&|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }));
    assert_eq!(died.len(), 3);
    assert!(died.iter().all(|b| *b == died[0]), "the three sacrifices are one event: {died:?}");
    assert!(entered.iter().all(|b| *b != died[0]), "and the entries another: {entered:?}");
    let (_, last_death) = record_span(&game, mark, |e| matches!(e, GameEvent::ZoneChange { to: Zone::Graveyard, .. }));
    let (first_entry, _) = record_span(&game, mark, |e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }));
    assert!(last_death < first_entry, "the sacrifices come before the entries");
}

/// Each token's counters are decided inside its own CR 616.1 loop, where a
/// doubler reads devour's count: four creatures' worth of counters doubled
/// is eight, two doubled is four.
#[test]
fn a_counter_doubler_doubles_each_devour_tokens_counters() {
    let mut game = setup_two_player_game();
    let doubler = put_on_battlefield(&mut game, doubles_your_plus_one_counters(), 0);
    for _ in 0..3 {
        put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    }

    let dragons = create_two_devouring_dragons(&mut game, doubler);

    let mut given: Vec<u32> = dragons.iter().map(|&id| counters(&game, id, CounterType::PlusOnePlusOne)).collect();
    given.sort();
    assert_eq!(given, vec![4, 8]);
}

/// Devour applies first and chooses a creature, then Containment Priest
/// replaces the entry with an exile. The entry devour modified never happens
/// (CR 614.6), so neither does the sacrifice that was part of it.
#[test]
fn an_entry_replaced_after_devour_chose_sacrifices_nothing() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, containment_priest(), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);

    let dp = test_dp();
    // Devour, the Elder's own effect, applies first.
    dp.expect_choice(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![ChoiceOption::Object(elder)]);
    dp.expect_pick_n(ChoiceKind::ChooseAuxiliaryZoneChange { entering: elder, source: elder, to: Zone::Graveyard }, vec![0]);
    game.change_zone(elder, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&dp)).expect("it is proposed");
    assert!(dp.is_empty());

    assert_eq!(game.get_object(elder).expect("its card").zone, Zone::Exile, "the Priest exiled it instead");
    assert!(game.battlefield.contains_key(&bear), "the sacrifice was part of an entry that never happened");
}

/// Thunder-Thrash Elder devours Master Biomancer, and Biomancer's own effect
/// still applies to the Elder's entry. Nothing is performed until the entry's
/// choices are all made, so Biomancer is on the battlefield when CR 616.1f
/// asks what else applies: it is an object that "will also change zones"
/// (CR 614.13a), and it leaves with the sacrifice, right before the Elder
/// enters. Three counters for devour, two for Biomancer's power.
#[test]
fn a_devoured_permanents_own_entry_effect_still_applies() {
    let mut game = setup_two_player_game();
    let biomancer = put_on_battlefield(&mut game, master_biomancer(), 0);
    let elder = put_in_graveyard(&mut game, thunder_thrash_elder(), 0);
    let dp = test_dp();
    dp.expect_choice(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![ChoiceOption::Object(elder)]);
    dp.expect_pick_n(ChoiceKind::ChooseAuxiliaryZoneChange { entering: elder, source: elder, to: Zone::Graveyard }, vec![0]);

    game.change_zone(elder, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&dp)).expect("it enters");

    assert!(dp.is_empty(), "devour first, then Biomancer's effect alone: {:?}", dp.asked());
    assert_eq!(game.get_object(biomancer).expect("its card").zone, Zone::Graveyard, "devoured");
    assert_eq!(counters(&game, elder, CounterType::PlusOnePlusOne), 5);
}

// ---------------------------------------------------------------------------
// An entry choice's cost, and CR 614.12b
// ---------------------------------------------------------------------------

/// "As this land enters, you may pay 2 life. If you don't, it enters tapped."
fn shock_replacement() -> Effect {
    Effect::Replacement(Box::new(ReplacementDef::new(
        EventPattern::EnterBattlefield { cast: None },
        ObjectSet::SourceOnly,
        Rewrite::EnterWith(EnterModsTemplate::tapped_unless_paid(Cost::PayLife(2))),
    )))
}

fn shockland() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Shock Land").card_type(CardType::Land).ability(static_ability(shock_replacement())).build()
}

/// A land token with the shockland's replacement, so two can enter at once
/// (one creation, two entries).
fn shockland_token() -> TokenDef {
    let mut token = soldier_token();
    token.name = Some("Fixture Shock Land".into());
    token.colors = Vec::new();
    token.types = vec![CardType::Land];
    token.subtypes = Vec::new();
    token.power = None;
    token.toughness = None;
    token.abilities = vec![static_ability(shock_replacement())];
    token.rules_text = "As this land enters, you may pay 2 life. If you don't, it enters tapped.".into();
    token
}

fn tapped(game: &GameState, id: ObjectId) -> bool {
    game.battlefield.get(&id).expect("on the battlefield").tapped
}

fn the_offer(object: ObjectId) -> ChoiceKind {
    ChoiceKind::AcceptAsObjectEnters { object, source: object, offer: EntryOffer::Pay(Cost::PayLife(2)) }
}

/// Played from hand and paid for: untapped, and 2 life lost (CR 119.4).
#[test]
fn a_shockland_paid_for_enters_untapped() {
    let mut game = setup_two_player_game();
    let land = put_in_hand(&mut game, shockland(), 0);
    let dp = test_dp();
    dp.expect_pick_n(the_offer(land), vec![0]);

    game.play_land(0, land, &dp).expect("the land drop");

    assert_asked(&dp, &[(0, the_offer(land))]);
    assert!(!tapped(&game, land));
    assert_eq!(game.players[0].life_total, 18);
}

/// Declined: it enters tapped, and no life is lost.
#[test]
fn a_shockland_declined_enters_tapped() {
    let mut game = setup_two_player_game();
    let land = put_in_hand(&mut game, shockland(), 0);
    let dp = test_dp();
    dp.expect_pick_n(the_offer(land), vec![]);

    game.play_land(0, land, &dp).expect("the land drop");

    assert_asked(&dp, &[(0, the_offer(land))]);
    assert!(tapped(&game, land));
    assert_eq!(game.players[0].life_total, 20);
}

/// With 1 life the payment cannot be made (CR 119.4), so it is not offered,
/// and the land enters tapped.
#[test]
fn an_unpayable_shockland_is_not_offered() {
    let mut game = setup_two_player_game();
    game.players[0].life_total = 1;
    let land = put_in_hand(&mut game, shockland(), 0);
    let dp = test_dp();

    game.play_land(0, land, &dp).expect("the land drop");

    assert_asked(&dp, &[]);
    assert!(tapped(&game, land));
    assert_eq!(game.players[0].life_total, 1);
}

/// Two enter at once for a player at 3 life: after paying for the first, the
/// second's payment would make the two cost more than the player has (CR
/// 614.12b), so it is not offered and that land enters tapped. The payment
/// is its own event, right before the entries.
#[test]
fn two_shocklands_entering_together_cannot_both_be_paid_for_at_three_life() {
    let mut game = setup_two_player_game();
    game.players[0].life_total = 3;
    // The tokens have no id until they are created; a scripted answer
    // matches the prompt's kind.
    let placeholder = put_in_graveyard(&mut game, vanilla_creature(1, 1, &[]), 1);
    let dp = test_dp();
    dp.expect_pick_n(the_offer(placeholder), vec![0]);
    let mark = game.recorded_events().len();

    game.execute_action(
        GameAction::CreateTokens { defs: vec![shockland_token(), shockland_token()], controller: 0, attacking: None },
        &ActionContext::new(&dp),
    )
    .expect("the tokens are created");

    assert_eq!(game.players[0].life_total, 1);
    let lands: Vec<ObjectId> = game.battlefield_ids_ordered();
    let paid_for: Vec<ObjectId> = lands.iter().copied().filter(|&id| !tapped(&game, id)).collect();
    assert_eq!((lands.len(), paid_for.len()), (2, 1), "one paid for, one tapped");
    assert_asked(&dp, &[(0, the_offer(paid_for[0]))]);

    let recorded = game.recorded_events();
    let batch_of = |pred: &dyn Fn(&GameEvent) -> bool| -> Vec<Option<BatchId>> {
        recorded.records_from(mark).iter().filter(|r| pred(&r.event)).map(|r| r.stamp.batch).collect()
    };
    let paid = batch_of(&|e| matches!(e, GameEvent::LifeChanged { .. }));
    let entered = batch_of(&|e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }));
    assert_eq!(paid.len(), 1);
    assert!(entered.iter().all(|b| *b != paid[0]), "the payment is its own event: {paid:?} {entered:?}");
    let (payment, _) = record_span(&game, mark, |e| matches!(e, GameEvent::LifeChanged { .. }));
    let (first_entry, _) = record_span(&game, mark, |e| matches!(e, GameEvent::PermanentEnteredBattlefield { .. }));
    assert!(payment < first_entry, "the payment comes before the entries");
}

/// A shockland token that also says "if this land would enter, exile it
/// instead", printed after the choice, so the choice applies first.
fn vanishing_shockland_token() -> TokenDef {
    let mut token = shockland_token();
    token.abilities.push(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
        EventPattern::EnterBattlefield { cast: None },
        ObjectSet::SourceOnly,
        Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }),
    )))));
    token
}

/// Two lands enter in one event at 3 life, and their choices are made one
/// land at a time before either enters (CR 616.1, 614.12a). The first land's
/// controller chooses to pay; then that land's own exile replaces its entry,
/// so it never enters and its payment never happens (CR 614.6). The second
/// land's choice, made next, has no cost already chosen beside it (CR
/// 614.12b), so its payment is offered too.
#[test]
fn a_payment_whose_entry_never_happens_is_no_cost_chosen() {
    let mut game = setup_two_player_game();
    game.players[0].life_total = 3;
    let placeholder = put_in_graveyard(&mut game, vanilla_creature(1, 1, &[]), 1);
    let dp = test_dp();
    dp.expect_pick_n(the_offer(placeholder), vec![0]);
    dp.expect_pick_n(the_offer(placeholder), vec![0]);

    game.execute_action(
        GameAction::CreateTokens {
            defs: vec![vanishing_shockland_token(), vanishing_shockland_token()],
            controller: 0,
            attacking: None,
        },
        &ActionContext::new(&dp),
    )
    .expect("the tokens are created");

    let offered: Vec<PlayerId> = dp.asked_of().iter().map(|(player, _)| *player).collect();
    assert_eq!(offered, vec![0, 0], "both payments were offered, each to their controller: {:?}", dp.asked());
    assert_eq!(game.players[0].life_total, 3, "neither entry happened, so neither payment did");
}

/// At exactly 2 life the payment is still payable (CR 119.4), and paying it
/// leaves the player at 0.
#[test]
fn a_shockland_is_payable_at_exactly_its_cost() {
    let mut game = setup_two_player_game();
    game.players[0].life_total = 2;
    let land = put_in_hand(&mut game, shockland(), 0);
    let dp = test_dp();
    dp.expect_pick_n(the_offer(land), vec![0]);

    game.play_land(0, land, &dp).expect("the land drop");

    assert!(!tapped(&game, land));
    assert_eq!(game.players[0].life_total, 0);
}

// ---------------------------------------------------------------------------
// Tribute: an opponent of your choice may put the counters
// ---------------------------------------------------------------------------

/// Tribute 2 (CR 702.104a): "As this creature enters, choose an opponent.
/// That player may put an additional two +1/+1 counters on it as it enters."
fn tribute_beast() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Tribute Beast")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::SourceOnly,
            Rewrite::EnterWith(EnterModsTemplate {
                status: None,
                counters: vec![EntryCountersTemplate {
                    counter: CounterType::PlusOnePlusOne,
                    amount: AmountExpr::Fixed(2),
                    by: Some(PlayerRef::Opponent),
                    optional: true,
                }],
                edits: Vec::new(),
                unless_paid: None,
            }),
        )))))
        .build()
}

fn the_putter(object: ObjectId) -> ChoiceKind {
    ChoiceKind::ChoosePlayerAsObjectEnters { object, source: object, role: EntryRole::CounterPutter }
}

fn the_counters(object: ObjectId) -> ChoiceKind {
    ChoiceKind::AcceptAsObjectEnters { object, source: object, offer: EntryOffer::PutCounters { counter: CounterType::PlusOnePlusOne, n: 2 } }
}

/// Who put the counters `object` entered with (CR 122.6a), off its record.
fn who_put_counters_on(game: &GameState, mark: usize, object: ObjectId) -> Vec<usize> {
    game.recorded_events()
        .records_from(mark)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::CountersPutOn { subject: ObjectOrPlayer::Object(on), by, .. } if on.id == object => Some(*by),
            _ => None,
        })
        .collect()
}

/// Three seats: the entering creature's controller chooses player 2, who puts
/// the counters on (CR 702.104a), so they are player 2's (CR 122.6a).
#[test]
fn tribute_asks_its_controller_for_an_opponent_who_may_put_the_counters() {
    let mut game = setup_game(3);
    let beast = put_in_graveyard(&mut game, tribute_beast(), 0);
    let dp = test_dp();
    dp.expect_choice(the_putter(beast), vec![ChoiceOption::Player(2)]);
    dp.expect_pick_n(the_counters(beast), vec![0]);
    let mark = game.recorded_events().len();

    game.change_zone(beast, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&dp)).expect("it enters");

    assert_asked(&dp, &[(0, the_putter(beast)), (2, the_counters(beast))]);
    assert_eq!(counters(&game, beast, CounterType::PlusOnePlusOne), 2);
    assert_eq!(who_put_counters_on(&game, mark, beast), vec![2]);
}

/// The chosen opponent declines, and it enters with none: tribute wasn't paid
/// (CR 702.104b).
#[test]
fn tributes_chosen_opponent_may_decline() {
    let mut game = setup_game(3);
    let beast = put_in_graveyard(&mut game, tribute_beast(), 0);
    let dp = test_dp();
    dp.expect_choice(the_putter(beast), vec![ChoiceOption::Player(1)]);
    dp.expect_pick_n(the_counters(beast), vec![]);

    game.change_zone(beast, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&dp)).expect("it enters");

    assert_asked(&dp, &[(0, the_putter(beast)), (1, the_counters(beast))]);
    assert_eq!(counters(&game, beast, CounterType::PlusOnePlusOne), 0);
}

/// Two seats: the one opponent is the choice, unasked, and only the "may" is.
#[test]
fn tribute_with_one_opponent_asks_only_whether_to_put_them() {
    let mut game = setup_two_player_game();
    let beast = put_in_graveyard(&mut game, tribute_beast(), 0);
    let dp = test_dp();
    dp.expect_pick_n(the_counters(beast), vec![0]);
    let mark = game.recorded_events().len();

    game.change_zone(beast, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&dp)).expect("it enters");

    assert_asked(&dp, &[(1, the_counters(beast))]);
    assert_eq!(who_put_counters_on(&game, mark, beast), vec![1]);
}

// ---------------------------------------------------------------------------
// A granted devour is the entering creature's
// ---------------------------------------------------------------------------

/// Devour granted by player 1's enchantment is still the entering creature's
/// own ability, so CR 614.13a's "you" is its controller, player 0 (CR 109.5):
/// player 0 chooses among player 0's creatures, and player 1's is no
/// candidate.
#[test]
fn a_granted_devour_is_chosen_by_the_entering_creatures_controller() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grants_devour_to_creatures(1), 1);
    let mine = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let entering = put_in_graveyard(&mut game, vanilla_creature(1, 1, &[]), 0);

    reanimate_devouring(&mut game, entering, 0, vec![0]);

    assert_eq!(game.get_object(mine).expect("its card").zone, Zone::Graveyard, "player 0 sacrificed their own creature");
    assert!(game.battlefield.contains_key(&theirs), "player 1's creature was never a candidate");
    assert_eq!(counters(&game, entering, CounterType::PlusOnePlusOne), 1);
}

// ---------------------------------------------------------------------------
// Devour by a count
// ---------------------------------------------------------------------------

/// Thromok the Insatiable's "devour X, where X is the number of creatures
/// devoured this way": its ruling's three creatures give nine counters.
#[test]
fn devour_by_the_count_gives_the_count_squared() {
    let mut game = setup_two_player_game();
    for _ in 0..3 {
        put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    }
    let gorger = put_in_graveyard(&mut game, devourer("Fixture Squaring Gorger", PerChosen::Count), 0);

    reanimate_devouring(&mut game, gorger, 0, vec![0, 1, 2]);

    assert_eq!(counters(&game, gorger, CounterType::PlusOnePlusOne), 9);
    assert_eq!(game.players[0].graveyard.len(), 3, "the three it devoured");
}

/// Two creatures give four (the ruling's second example), so the count is
/// read per entry rather than fixed when the ability is written.
#[test]
fn devour_by_the_count_reads_this_entrys_count() {
    let mut game = setup_two_player_game();
    for _ in 0..3 {
        put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    }
    let gorger = put_in_graveyard(&mut game, devourer("Fixture Squaring Gorger", PerChosen::Count), 0);

    reanimate_devouring(&mut game, gorger, 0, vec![0, 2]);

    assert_eq!(counters(&game, gorger, CounterType::PlusOnePlusOne), 4);
}

// ---------------------------------------------------------------------------
// A trigger's filter reads an object its own event moved as the event found it
// ---------------------------------------------------------------------------

/// "Whenever one or more counters are removed from a permanent you control,
/// you gain 1 life", Cloudsculpt Armorer's trigger with a gain for its seek.
fn gains_when_counters_leave_your_permanent() -> Arc<CardData> {
    let arm = counters_removed_from(TriggerSubject::Filter(ObjectFilter::ByController(PlayerRef::You)), None).once_per_object();
    CardDataBuilder::new("Fixture Counter Warden")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(arm, Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You)))))
        .build()
}

/// A 1/1 with a +1/+1 counter and two -1/-1 counters is 0/0. One state-based
/// check removes a counter of each kind (CR 704.5q) and puts it into its
/// graveyard (CR 704.5f) as one event (CR 704.3), so the counters come off a
/// permanent its controller controls, and the trigger reads it as the check
/// found it. The creature is player 1's, controlled by player 0, so the card
/// in player 1's graveyard is player 1's to ask about (CR 108.4a) and the two
/// reads disagree.
#[test]
fn a_counter_trigger_reads_a_creature_its_check_also_killed_as_the_check_found_it() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, gains_when_counters_leave_your_permanent(), 0);
    let soldier = put_on_battlefield_under(&mut game, vanilla_creature(1, 1, &[]), 1, 0);
    add_counters(&mut game, soldier, CounterType::PlusOnePlusOne, 1);
    add_counters(&mut game, soldier, CounterType::MinusOneMinusOne, 2);

    game.perform_sba_and_triggers(&test_dp()).expect("the check and its triggers");
    assert_eq!(game.get_object(soldier).expect("its card").zone, Zone::Graveyard, "the check killed it");
    assert_eq!(game.stack.len(), 1, "one permanent lost counters, so the warden triggered once");
    let life = game.players[0].life_total;
    game.resolve_top_of_stack(&test_dp()).expect("the trigger resolves");
    assert_eq!(game.players[0].life_total, life + 1);
}
