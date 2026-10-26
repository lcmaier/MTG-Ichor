//! Phase TR-5b — counters (CR 122.6, 122.7, 704.5q).
//!
//! A put and a removal are two records, each naming the player who did it
//! where the rules name one, and the count it left (`now`). Counters a
//! permanent enters with are announced as put on it (CR 122.6). The trigger
//! arms read the two records once per counter, once per object, or once per
//! event, and CR 122.7's threshold is the ability's own intervening "if". CR
//! 704.5q's two removals are members of the state-based batch. An entry
//! template reads the X of the spell becoming the object (CR 107.3m).

use std::sync::Arc;

use mtgsim::cards::authoring::triggers::{counters_put_on, counters_removed_from, persist, triggered_ability, whenever};
use mtgsim::cards::phase_rc_cards::{chainbreaker, master_biomancer};
use mtgsim::cards::phase_tr5b_cards::{protean_hydra, simic_ascendancy};
use mtgsim::cards::phase_tr4a_cards::strangleroot_geist;
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeAct};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::{EventCause, StateBasedAction};
use mtgsim::events::event::GameEvent;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::{GameResult, GameState, StepType};
use mtgsim::test_support::{
    fill_library, put_in_graveyard, put_in_hand, put_on_battlefield, RecordingDecisionProvider, setup_two_player_game, static_ability, test_ctx,
    test_dp, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    PlayerSet, Antecedent, AmountExpr, Condition, CounterType, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerRef, Primitive,
    SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayer, ObjectOrPlayerId, PlayerId};
use mtgsim::types::mana::{ManaCost, ManaSymbol, ManaType};
use mtgsim::types::replacement::{EnterModsTemplate, EventPattern, ReplacementDef, Rewrite};
use mtgsim::types::triggers::{Multiplicity, ParticipantFilter, TriggerCondition, TriggerDef, TriggerEvent, TriggerSubject};
use mtgsim::types::zones::{Zone, ZoneChangePattern};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::oracle::x_values::LegalXValues;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A nameless card with no abilities, to be the source of a fixture
/// resolution.
fn fixture_object() -> Arc<CardData> {
    CardDataBuilder::new("Fixture").build()
}

/// Resolve `effect` for `player` with `targets` already chosen, the way a
/// spell would.
fn resolve_targeting(
    game: &mut GameState,
    player: PlayerId,
    targets: Vec<ObjectOrPlayerId>,
    effect: &Effect,
    dp: &dyn DecisionProvider,
) {
    let source = put_in_hand(game, fixture_object(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        generated_by: mtgsim::events::cause::GeneratedBy::Spell,
        targets: ChosenTargets::one(targets),
        replaced_event: None,
        trigger: None,
    };
    game.resolve_effect(effect, &ctx, dp).expect("resolving");
}

/// "Put `n` `kind` counters on target permanent", or with `Remove`, "remove
/// `n`", resolved by `player` on `target`.
fn counters_on_target(game: &mut GameState, player: PlayerId, target: ObjectId, primitive: Primitive) {
    let effect = Effect::Atom(
        primitive,
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)),
    );
    resolve_targeting(game, player, vec![ObjectOrPlayerId::Object(target)], &effect, &test_dp());
}

fn put(kind: CounterType, n: u64) -> Primitive {
    Primitive::AddCounters { counter: kind, amount: AmountExpr::Fixed(n), by: PlayerRef::You }
}

fn remove(kind: CounterType, n: u64) -> Primitive {
    Primitive::RemoveCounters(kind, AmountExpr::Fixed(n))
}

/// Put a card from a graveyard onto the battlefield, through the chokepoint.
fn reanimate(game: &mut GameState, card: Arc<CardData>, player: PlayerId) -> ObjectId {
    let id = put_in_graveyard(game, card, player);
    game.change_zone(id, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&test_dp()))
        .expect("it enters");
    id
}

/// The counter records performed since `start`, in order.
fn counter_records(game: &GameState, start: usize) -> Vec<GameEvent> {
    game.recorded_events()
        .records_from(start)
        .iter()
        .map(|record| record.event.clone())
        .filter(|event| matches!(event, GameEvent::CountersPutOn { .. } | GameEvent::CountersRemovedFrom { .. }))
        .collect()
}

/// `object` as the counter records since it was last moved name it.
fn on(game: &GameState, object: ObjectId) -> ObjectOrPlayer {
    ObjectOrPlayer::Object(game.object_ref(object).expect("in a zone"))
}

/// Whether every counter record since `start` is about `object`.
fn records_name_only(game: &GameState, start: usize, object: ObjectId) -> bool {
    counter_records(game, start).iter().all(|event| match event {
        GameEvent::CountersPutOn { subject, .. } | GameEvent::CountersRemovedFrom { subject, .. } => {
            subject.object().is_some_and(|named| named.id == object)
        }
        _ => false,
    })
}

/// A planeswalker with printed loyalty 3 and nothing else.
fn loyalty_three() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Walker").card_type(CardType::Planeswalker).loyalty(3).build()
}

// ---------------------------------------------------------------------------
// The two records, their players and their counts
// ---------------------------------------------------------------------------

/// CR 608.2c: an instruction's controller puts the counters on, whoever
/// controls the permanent, and the record keeps the count they leave.
#[test]
fn a_put_names_the_instructions_controller_and_the_count_it_leaves() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    counters_on_target(&mut game, 0, bears, put(CounterType::PlusOnePlusOne, 1));
    let start = game.recorded_events().records().len();

    counters_on_target(&mut game, 1, bears, put(CounterType::PlusOnePlusOne, 2));

    assert_eq!(
        counter_records(&game, start),
        vec![GameEvent::CountersPutOn {
            subject: on(&game, bears),
            counter_type: CounterType::PlusOnePlusOne,
            n: 2,
            by: 1,
            now: 3,
        }],
    );
}

/// CR 608.2c again, for a removal: Vampire Hexmage's "remove all counters"
/// is its controller's, as a put is. CR 609.3: as many as there are.
#[test]
fn a_removal_by_an_instruction_names_its_controller_and_takes_what_is_there() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    counters_on_target(&mut game, 0, bears, put(CounterType::PlusOnePlusOne, 3));
    let start = game.recorded_events().records().len();

    counters_on_target(&mut game, 1, bears, remove(CounterType::PlusOnePlusOne, 2));
    counters_on_target(&mut game, 1, bears, remove(CounterType::PlusOnePlusOne, 5));

    let removed = |n, now| GameEvent::CountersRemovedFrom {
        subject: on(&game, bears),
        counter_type: CounterType::PlusOnePlusOne,
        n,
        by: Some(1),
        now,
    };
    assert_eq!(counter_records(&game, start), vec![removed(2, 1), removed(1, 0)]);
}

/// CR 120.3c words damage's loyalty removal with no player, where 120.3d
/// names wither's putter, so the removal has none.
#[test]
fn damage_to_a_planeswalker_removes_loyalty_that_no_player_removed() {
    let mut game = setup_two_player_game();
    let walker = reanimate(&mut game, loyalty_three(), 1);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let start = game.recorded_events().records().len();

    game.execute_action(
        GameAction::DealDamage {
            source: bears,
            target: ObjectOrPlayerId::Object(walker),
            amount: 2,
            is_combat: false,
            unpreventable: false,
            source_lki: None,
        },
        &test_ctx(),
    )
    .expect("the damage is dealt");

    assert_eq!(
        counter_records(&game, start),
        vec![GameEvent::CountersRemovedFrom {
            subject: on(&game, walker),
            counter_type: CounterType::Loyalty,
            n: 2,
            by: None,
            now: 1,
        }],
    );
}

/// CR 122.1d's replacement removes a stun counter in place of an untap, and
/// the effect's controller removes it, as a shield's rider does: the
/// permanent's controller.
#[test]
fn a_stun_counters_own_replacement_is_its_permanents_controllers_removal() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    counters_on_target(&mut game, 0, bears, put(CounterType::Stun, 1));
    game.battlefield.get_mut(&bears).expect("on the battlefield").tapped = true;
    let start = game.recorded_events().records().len();

    game.execute_action(GameAction::Untap { object: bears }, &test_ctx()).expect("the untap is replaced");

    assert_eq!(
        counter_records(&game, start),
        vec![GameEvent::CountersRemovedFrom {
            subject: on(&game, bears),
            counter_type: CounterType::Stun,
            n: 1,
            by: Some(0),
            now: 0,
        }],
    );
}

// ---------------------------------------------------------------------------
// An entry's counters (CR 122.6)
// ---------------------------------------------------------------------------

/// A fixture enchantment carrying one static replacement.
fn fixture_static(name: &str, def: ReplacementDef) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Replacement(Box::new(def))))
        .build()
}

/// A 2/2 that "enters with a +1/+1 counter on it".
fn enters_with_a_counter() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Grower")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::SourceOnly,
            Rewrite::EnterWith(EnterModsTemplate::with_counters(CounterType::PlusOnePlusOne, 1)),
        )))))
        .build()
}

/// CR 122.6: a permanent that enters with counters had them put on it, so
/// the entry announces them after itself and in its own batch, put on by the
/// controller it entered under (CR 122.6a).
#[test]
fn an_entry_announces_its_counters_after_itself_in_its_batch() {
    let mut game = setup_two_player_game();
    let start = game.recorded_events().records().len();

    let chained = reanimate(&mut game, chainbreaker(), 1);

    let log = game.recorded_events();
    let records = log.records_from(start);
    let entered = records
        .iter()
        .position(|r| matches!(r.event, GameEvent::PermanentEnteredBattlefield { object, .. } if object.id == chained))
        .expect("it entered");
    let put = records.iter().position(|r| matches!(r.event, GameEvent::CountersPutOn { .. })).expect("its counters were announced");
    assert!(entered < put, "the entry first, then what it entered with");
    assert_eq!(records[entered].batch(), records[put].batch(), "one event");
    assert_eq!(
        records[put].event,
        GameEvent::CountersPutOn {
            subject: on(&game, chained),
            counter_type: CounterType::MinusOneMinusOne,
            n: 2,
            by: 1,
            now: 2,
        },
    );
}

/// The anthem's row, put on by `putter` where it names one (CR 122.6a).
fn your_creatures_enter_with_a_counter_put_by(putter: Option<PlayerRef>) -> Arc<CardData> {
    let mut template = EnterModsTemplate::with_counters(CounterType::PlusOnePlusOne, 1);
    template.counters[0].by = putter;
    fixture_static(
        "Fixture Anthem of Counters",
        ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::battlefield_filter(ObjectFilter::And(
                Box::new(ObjectFilter::ByType(CardType::Creature)),
                Box::new(ObjectFilter::ByController(PlayerSet::You)),
            )),
            Rewrite::EnterWith(template),
        ),
    )
}

/// The entry's rows of one kind and putter are one row (`EnterMods`'s merge),
/// so the card's own counter and the anthem's are one record of two. A row
/// another player puts on is a record of its own, and each says the count it
/// left.
#[test]
fn an_entrys_rows_are_one_record_per_kind_and_putter_each_with_its_count() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, your_creatures_enter_with_a_counter_put_by(None), 0);
    let start = game.recorded_events().records().len();
    let grower = reanimate(&mut game, enters_with_a_counter(), 0);
    let put = |object, n, by, now| GameEvent::CountersPutOn {
        subject: on(&game, object),
        counter_type: CounterType::PlusOnePlusOne,
        n,
        by,
        now,
    };
    assert_eq!(counter_records(&game, start), vec![put(grower, 2, 0, 2)], "one putter, one row");

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, your_creatures_enter_with_a_counter_put_by(Some(PlayerRef::Opponent)), 0);
    let start = game.recorded_events().records().len();
    let grower = reanimate(&mut game, enters_with_a_counter(), 0);
    assert!(records_name_only(&game, start, grower));
    // Both effects are CR 616.1e's, and their order changes nothing the
    // records answer, so nobody is asked it: the counts run 1 then 2 in
    // whichever order they applied.
    let records = counter_records(&game, start);
    let (putters, counts): (Vec<PlayerId>, Vec<u32>) = records
        .iter()
        .map(|event| match event {
            GameEvent::CountersPutOn { by, now, .. } => (*by, *now),
            other => panic!("not a put: {other:?}"),
        })
        .unzip();
    assert_eq!(counts, [1, 2], "two putters, two rows: {records:?}");
    assert!(putters.contains(&0) && putters.contains(&1), "{records:?}");
}

// ---------------------------------------------------------------------------
// The two arms: per counter, per object, per event (CR 603.2c)
// ---------------------------------------------------------------------------

fn gain(amount: AmountExpr) -> Effect {
    Effect::Atom(Primitive::GainLife(amount), EffectRecipient::Player(PlayerRef::You))
}

/// A 2/2 with one triggered ability, `event` gaining its controller `amount`.
fn creature_that_gains(name: &'static str, event: impl Into<TriggerEvent>, amount: AmountExpr) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(triggered_ability("", whenever(event, gain(amount))))
        .build()
}

/// An enchantment with one triggered ability, `event` gaining its controller
/// `amount`.
fn enchantment_that_gains(name: &'static str, event: impl Into<TriggerEvent>, amount: AmountExpr) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(event, gain(amount))))
        .build()
}

/// "A creature you control".
fn a_creature_you_control() -> ObjectFilter {
    ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ByController(PlayerSet::You)))
}

/// Resolve "put `n` `kind` counters on each creature you control" for `player`.
fn counters_on_each_creature(game: &mut GameState, player: PlayerId, kind: CounterType, n: u64) {
    let effect = Effect::Atom(put(kind, n), EffectRecipient::FilteredPermanents(a_creature_you_control()));
    resolve_targeting(game, player, Vec::new(), &effect, &test_dp());
}

/// The triggers `source` has waiting to be put on the stack.
fn waiting_from(game: &GameState, source: ObjectId) -> usize {
    game.pending_triggers.iter().filter(|p| p.origin.source() == source).count()
}

fn resolve_all(game: &mut GameState) {
    resolve_all_with(game, &test_dp());
}

/// [`resolve_all`] under `dp`, for a board whose simultaneous triggers ask
/// their order (CR 603.3b).
fn resolve_all_with(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).expect("SBAs and triggers");
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(dp).expect("it resolves");
        game.perform_sba_and_triggers(dp).expect("SBAs and triggers");
    }
}

/// Fathom Mage's seventh and ninth rulings: "a +1/+1 counter" triggers for
/// each counter put on at once, whatever put it there.
#[test]
fn a_per_counter_arm_triggers_once_for_each_counter_put_on_at_once() {
    let mut game = setup_two_player_game();
    let arm = counters_put_on(TriggerSubject::Object(Antecedent::ThisObject), CounterType::PlusOnePlusOne);
    let mage = put_on_battlefield(&mut game, creature_that_gains("Fixture Mage", arm, AmountExpr::Fixed(1)), 0);

    counters_on_target(&mut game, 1, mage, put(CounterType::PlusOnePlusOne, 3));
    assert_eq!(waiting_from(&game, mage), 3);

    resolve_all(&mut game);
    assert_eq!(game.players[0].life_total, 23, "\"that many\" of one counter is one");
}

/// Fathom Mage's eighth ruling and CR 122.6: a creature that enters with
/// counters triggers once for each, its own arm answering as it arrives.
#[test]
fn a_creature_entering_with_counters_triggers_its_own_per_counter_arm() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, your_creatures_enter_with_a_counter_put_by(None), 0);
    let grower = CardDataBuilder::new("Fixture Mage")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::SourceOnly,
            Rewrite::EnterWith(EnterModsTemplate::with_counters(CounterType::PlusOnePlusOne, 1)),
        )))))
        .ability(triggered_ability(
            "",
            whenever(counters_put_on(TriggerSubject::Object(Antecedent::ThisObject), CounterType::PlusOnePlusOne), gain(AmountExpr::Fixed(1))),
        ))
        .build();

    let mage = reanimate(&mut game, grower, 0);

    assert_eq!(waiting_from(&game, mage), 2, "its own counter and the anthem's");
}

/// The fixture Scalelord: "whenever one or more +1/+1 counters are put on a
/// creature you control, you gain that much life".
fn scalelord() -> Arc<CardData> {
    let arm = counters_put_on(a_creature_you_control(), CounterType::PlusOnePlusOne).once_per_object();
    enchantment_that_gains("Fixture Scalelord", arm, AmountExpr::TriggeringAmount)
}

/// Enduring Scalelord's second ruling: "one or more ... on another creature
/// you control" triggers once for each creature, and "that many" is that
/// creature's.
#[test]
fn a_once_per_object_arm_triggers_once_for_each_creature() {
    let mut game = setup_two_player_game();
    let scalelord = put_on_battlefield(&mut game, scalelord(), 0);
    for _ in 0..3 {
        put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    }

    counters_on_each_creature(&mut game, 0, CounterType::PlusOnePlusOne, 2);
    assert_eq!(waiting_from(&game, scalelord), 3);

    resolve_all(&mut game);
    assert_eq!(game.players[0].life_total, 26, "two each, three times");
}

/// Two records on one creature in one event are one trigger of the
/// once-per-object arm, and "that many" sums them.
#[test]
fn a_once_per_object_arm_folds_one_creatures_records_and_sums_them() {
    let mut game = setup_two_player_game();
    let scalelord = put_on_battlefield(&mut game, scalelord(), 0);
    put_on_battlefield(&mut game, your_creatures_enter_with_a_counter_put_by(Some(PlayerRef::Opponent)), 0);

    reanimate(&mut game, enters_with_a_counter(), 0);
    assert_eq!(waiting_from(&game, scalelord), 1, "two putters' records, one creature");

    resolve_all(&mut game);
    assert_eq!(game.players[0].life_total, 22);
}

/// Invisible Woman's and Cloaked Cadet's "on one or more [creatures]" is one
/// event, so one trigger however many creatures.
#[test]
fn a_once_per_event_arm_triggers_once_for_the_event() {
    let mut game = setup_two_player_game();
    let arm = counters_put_on(a_creature_you_control(), CounterType::PlusOnePlusOne).once_per_event();
    let cadet = put_on_battlefield(&mut game, enchantment_that_gains("Fixture Cadet", arm, AmountExpr::TriggeringAmount), 0);
    for _ in 0..3 {
        put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    }

    counters_on_each_creature(&mut game, 0, CounterType::PlusOnePlusOne, 1);
    assert_eq!(waiting_from(&game, cadet), 1);
}

/// Hapatra's second ruling: "whenever you put one or more -1/-1 counters on
/// a creature" is the putter's, once for each creature, and another
/// player's put is not yours.
#[test]
fn a_put_arms_by_reads_who_put_the_counters_on() {
    let mut game = setup_two_player_game();
    let arm = counters_put_on(ObjectFilter::ByType(CardType::Creature), CounterType::MinusOneMinusOne)
        .by(PlayerSet::You)
        .once_per_object();
    let hapatra = put_on_battlefield(&mut game, enchantment_that_gains("Fixture Hapatra", arm, AmountExpr::Fixed(1)), 0);
    let bears = put_on_battlefield(&mut game, vanilla_creature(4, 4, &[]), 1);
    let other = put_on_battlefield(&mut game, vanilla_creature(4, 4, &[]), 1);

    counters_on_target(&mut game, 1, bears, put(CounterType::MinusOneMinusOne, 1));
    assert_eq!(waiting_from(&game, hapatra), 0, "an opponent put them on");

    counters_on_target(&mut game, 0, bears, put(CounterType::MinusOneMinusOne, 1));
    counters_on_target(&mut game, 0, other, put(CounterType::MinusOneMinusOne, 2));
    assert_eq!(waiting_from(&game, hapatra), 2);
}

/// Territorial Gorger's "whenever you get one or more {E}": counters on a
/// player, the subject a player rather than an object.
#[test]
fn a_put_arm_reads_counters_a_player_gets() {
    let mut game = setup_two_player_game();
    let arm = TriggerEvent::CountersPutOn {
        subject: ParticipantFilter::Player(PlayerSet::You),
        counter_type: Some(CounterType::Energy),
        by: PlayerSet::Everyone,
        multiplicity: Multiplicity::OncePerSubject,
    };
    let gorger = put_on_battlefield(&mut game, enchantment_that_gains("Fixture Gorger", arm, AmountExpr::TriggeringAmount), 0);
    let energy = |player| GameAction::AddCounters { subject: ObjectOrPlayerId::Player(player), counter: CounterType::Energy, n: 2, by: player };

    game.execute_action(energy(1), &test_ctx()).expect("an opponent gets energy");
    assert_eq!(waiting_from(&game, gorger), 0);

    game.execute_action(energy(0), &test_ctx()).expect("you get energy");
    assert_eq!(waiting_from(&game, gorger), 1);
    resolve_all(&mut game);
    assert_eq!(game.players[0].life_total, 22);
}

/// Protean Hydra's fourth ruling, on a fixture: several counters removed at
/// once trigger "whenever a +1/+1 counter is removed" that many times.
#[test]
fn a_per_counter_removal_arm_triggers_once_for_each_counter_removed() {
    let mut game = setup_two_player_game();
    let arm = counters_removed_from(TriggerSubject::Object(Antecedent::ThisObject), CounterType::PlusOnePlusOne);
    let hydra = put_on_battlefield(&mut game, creature_that_gains("Fixture Hydra", arm, AmountExpr::Fixed(1)), 0);
    counters_on_target(&mut game, 0, hydra, put(CounterType::PlusOnePlusOne, 3));

    counters_on_target(&mut game, 1, hydra, remove(CounterType::PlusOnePlusOne, 2));

    assert_eq!(waiting_from(&game, hydra), 2);
}

/// CR 609.3, as Amy Pond's "remove that many time counters" can ask for more
/// than a card has: an instruction asking three of a creature with one takes
/// the one, its record says one, and a per-counter arm triggers once. What
/// was asked is the proposal's, which a replacement reads; what was taken is
/// the record's, which a trigger reads. A removal from a creature with none
/// takes nothing, which is no event.
#[test]
fn a_removal_asking_more_than_is_there_records_and_triggers_on_what_it_took() {
    let mut game = setup_two_player_game();
    let arm = counters_removed_from(TriggerSubject::Object(Antecedent::ThisObject), CounterType::PlusOnePlusOne);
    let hydra = put_on_battlefield(&mut game, creature_that_gains("Fixture Hydra", arm, AmountExpr::Fixed(1)), 0);
    counters_on_target(&mut game, 0, hydra, put(CounterType::PlusOnePlusOne, 1));
    let start = game.recorded_events().records().len();

    counters_on_target(&mut game, 1, hydra, remove(CounterType::PlusOnePlusOne, 3));
    let took_one = GameEvent::CountersRemovedFrom {
        subject: on(&game, hydra),
        counter_type: CounterType::PlusOnePlusOne,
        n: 1,
        by: Some(1),
        now: 0,
    };
    assert_eq!(counter_records(&game, start), vec![took_one]);
    assert_eq!(waiting_from(&game, hydra), 1, "one counter taken, one trigger");

    let start = game.recorded_events().records().len();
    counters_on_target(&mut game, 1, hydra, remove(CounterType::PlusOnePlusOne, 3));
    assert_eq!(counter_records(&game, start), Vec::new(), "none there, none taken");
    assert_eq!(waiting_from(&game, hydra), 1, "and no second trigger");
}

// ---------------------------------------------------------------------------
// CR 122.7's threshold, the ability's own "if"
// ---------------------------------------------------------------------------

/// "When the third charge counter is put on this, draw a card."
fn third_charge_counter() -> Arc<CardData> {
    let arm = counters_put_on(TriggerSubject::Object(Antecedent::ThisObject), CounterType::Charge).once_per_object();
    CardDataBuilder::new("Fixture Countdown")
        .card_type(CardType::Artifact)
        .ability(triggered_ability(
            "When the third charge counter is put on this artifact, draw a card.",
            TriggerDef {
                condition: TriggerCondition::Event(arm),
                intervening_if: Some(Condition::CounterCountReached { counter_type: CounterType::Charge, n: 3 }),
                limit: None,
                effect: Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You)),
            },
        ))
        .build()
}

// COVERS: ATOM-122.7-001
#[test]
fn the_nth_counter_triggers_when_the_count_crosses_n() {
    let mut game = setup_two_player_game();
    fill_library(&mut game, 0, 5);
    let countdown = put_on_battlefield(&mut game, third_charge_counter(), 0);
    counters_on_target(&mut game, 0, countdown, put(CounterType::Charge, 2));
    assert_eq!(waiting_from(&game, countdown), 0, "two is not three");

    counters_on_target(&mut game, 0, countdown, put(CounterType::Charge, 1));
    assert_eq!(waiting_from(&game, countdown), 1, "two to three crosses three");

    let hand = game.players[0].hand.len();
    resolve_all(&mut game);
    assert_eq!(game.players[0].hand.len(), hand + 1, "the \"if\" still holds as it resolves, off the record");
}

/// One put that jumps past N crosses it; a put that starts at N does not.
#[test]
fn the_nth_counter_reads_the_count_before_and_after_one_event() {
    let mut game = setup_two_player_game();
    let countdown = put_on_battlefield(&mut game, third_charge_counter(), 0);

    counters_on_target(&mut game, 0, countdown, put(CounterType::Charge, 5));
    assert_eq!(waiting_from(&game, countdown), 1, "zero to five crosses three");
    game.pending_triggers.clear();

    counters_on_target(&mut game, 0, countdown, put(CounterType::Charge, 1));
    assert_eq!(waiting_from(&game, countdown), 0, "five to six does not");
}

/// `readings_its_arms_cannot_give` is the registry's check that a trigger
/// asks nothing of its event that the event's arm cannot supply; every
/// registered card must pass it. Each wrong def here breaks one of its two
/// rules: "once per subject" on a death, which is one record per object, so
/// that grouping by object would change nothing; and "the third counter" read
/// once per counter, where one put of three would make three triggers that
/// each see the count cross three. The countdown, written right, passes.
#[test]
fn a_trigger_asking_a_reading_its_arm_cannot_give_is_refused() {
    let once_per_subject_dies = TriggerDef {
        condition: TriggerCondition::Event(TriggerEvent::ZoneChange {
            subject: TriggerSubject::Filter(ObjectFilter::All),
            change: ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: None },
            by: None,
            owner: PlayerSet::Everyone,
            multiplicity: Multiplicity::OncePerSubject,
        }),
        intervening_if: None,
        limit: None,
        effect: gain(AmountExpr::Fixed(1)),
    };
    assert_eq!(once_per_subject_dies.readings_its_arms_cannot_give().len(), 1);

    let per_counter_threshold = TriggerDef {
        condition: TriggerCondition::Event(counters_put_on(TriggerSubject::Object(Antecedent::ThisObject), CounterType::Charge).into()),
        intervening_if: Some(Condition::CounterCountReached { counter_type: CounterType::Charge, n: 3 }),
        limit: None,
        effect: gain(AmountExpr::Fixed(1)),
    };
    assert_eq!(per_counter_threshold.readings_its_arms_cannot_give().len(), 1, "a counter each would trigger once per counter");

    let countdown = third_charge_counter();
    assert!(countdown.abilities.iter().all(|ability| match &ability.effect {
        Effect::Triggered(def) => def.readings_its_arms_cannot_give().is_empty(),
        _ => true,
    }));
}

// ---------------------------------------------------------------------------
// CR 704.5q's two removals, in the state-based batch
// ---------------------------------------------------------------------------

/// The records CR 704.5q's annihilation wrote since `start`, each with its
/// batch.
fn annihilations_since(game: &GameState, start: usize) -> Vec<(GameEvent, Option<mtgsim::events::event::BatchId>)> {
    let annihilating = EventCause::StateBasedAction(StateBasedAction::CountersAnnihilate);
    game.recorded_events()
        .records_from(start)
        .iter()
        .filter(|record| record.stamp.cause == annihilating)
        .map(|record| (record.event.clone(), record.batch()))
        .collect()
}

fn removed(subject: ObjectOrPlayer, counter_type: CounterType, n: u32, now: u32) -> GameEvent {
    GameEvent::CountersRemovedFrom { subject, counter_type, n, by: None, now }
}

// COVERS: ATOM-704.5q-001
#[test]
fn the_annihilation_is_two_removals_of_one_event_that_no_player_made() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(4, 4, &[]), 0);
    counters_on_target(&mut game, 0, bears, put(CounterType::PlusOnePlusOne, 3));
    counters_on_target(&mut game, 0, bears, put(CounterType::MinusOneMinusOne, 2));
    let start = game.recorded_events().records().len();

    assert!(game.check_state_based_actions(&test_dp()).expect("the check"));

    let records = annihilations_since(&game, start);
    let events: Vec<GameEvent> = records.iter().map(|(event, _)| event.clone()).collect();
    assert_eq!(
        events,
        vec![removed(on(&game, bears), CounterType::PlusOnePlusOne, 2, 1), removed(on(&game, bears), CounterType::MinusOneMinusOne, 2, 0)]
    );
    assert_eq!(records[0].1, records[1].1, "CR 704.3: one event");
    assert_eq!(game.battlefield[&bears].counter_count(CounterType::PlusOnePlusOne), 1);
}

// COVERS: ATOM-704.5q-002
#[test]
fn equal_counts_annihilate_to_none_of_either() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, vanilla_creature(4, 4, &[]), 0);
    counters_on_target(&mut game, 0, bears, put(CounterType::PlusOnePlusOne, 3));
    counters_on_target(&mut game, 0, bears, put(CounterType::MinusOneMinusOne, 3));
    let start = game.recorded_events().records().len();

    game.check_state_based_actions(&test_dp()).expect("the check");

    let events: Vec<GameEvent> = annihilations_since(&game, start).into_iter().map(|(event, _)| event).collect();
    assert_eq!(
        events,
        vec![removed(on(&game, bears), CounterType::PlusOnePlusOne, 3, 0), removed(on(&game, bears), CounterType::MinusOneMinusOne, 3, 0)]
    );
    let left = |kind| game.battlefield[&bears].counter_count(kind);
    assert_eq!((left(CounterType::PlusOnePlusOne), left(CounterType::MinusOneMinusOne)), (0, 0));
}

/// CR 704.8, on ATOM-704.8-001's board: Strangleroot Geist with a +1/+1
/// counter takes three -1/-1 counters. One check removes a pair and puts it
/// into the graveyard, and its last known information is from before
/// either, with its +1/+1 counter, so undying does not return it.
/// Annihilating first would leave it none, and it would.
// COVERS: ATOM-704.8-001
#[test]
fn a_creature_the_check_kills_is_framed_before_its_annihilation() {
    let mut game = setup_two_player_game();
    let geist = put_on_battlefield(&mut game, strangleroot_geist(), 0);
    counters_on_target(&mut game, 0, geist, put(CounterType::PlusOnePlusOne, 1));
    counters_on_target(&mut game, 1, geist, put(CounterType::MinusOneMinusOne, 3));
    let start = game.recorded_events().records().len();
    let annihilated = on(&game, geist);

    resolve_all(&mut game);

    let events: Vec<GameEvent> = annihilations_since(&game, start).into_iter().map(|(event, _)| event).collect();
    assert_eq!(
        events,
        vec![removed(annihilated, CounterType::PlusOnePlusOne, 1, 0), removed(annihilated, CounterType::MinusOneMinusOne, 1, 2)],
        "both removals are performed, the death beside them"
    );
    assert_eq!(game.get_object(geist).expect("its card").zone, Zone::Graveyard, "undying did not return it");
}

/// Persist's seventh ruling (Kitchen Finks, Wingrattle Scarecrow), where the
/// order shows: a 2/2 with a +1/+1 and a -1/-1 counter and lethal damage is
/// destroyed in the check that removes the pair, and persist reads the -1/-1
/// counter it had as it died.
#[test]
fn persist_reads_the_counters_a_creature_died_with_beside_its_annihilation() {
    let mut game = setup_two_player_game();
    let finks = CardDataBuilder::new("Fixture Finks")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(persist())
        .build();
    let finks = put_on_battlefield(&mut game, finks, 0);
    counters_on_target(&mut game, 0, finks, put(CounterType::PlusOnePlusOne, 1));
    counters_on_target(&mut game, 0, finks, put(CounterType::MinusOneMinusOne, 1));
    game.battlefield.get_mut(&finks).expect("on the battlefield").damage_marked = 2;

    resolve_all(&mut game);

    assert_eq!(game.get_object(finks).expect("its card").zone, Zone::Graveyard, "persist did not return it");
}

// ---------------------------------------------------------------------------
// X in an entry template (CR 107.3m)
// ---------------------------------------------------------------------------

/// A {X}{G} 0/0 that "enters with X +1/+1 counters on it".
fn enters_with_x_counters() -> Arc<CardData> {
    CardDataBuilder::new("Fixture X Grower")
        .card_type(CardType::Creature)
        .mana_cost(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Green)]))
        .power_toughness(0, 0)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            ObjectSet::SourceOnly,
            Rewrite::EnterWith(EnterModsTemplate::with_counter_amount(CounterType::PlusOnePlusOne, AmountExpr::X)),
        )))))
        .build()
}

/// Cast `card` from `player`'s hand with X = `x`, paying from a pool of
/// green mana, and resolve it.
fn cast_with_x(game: &mut GameState, player: PlayerId, card: Arc<CardData>, x: u64) -> ObjectId {
    cast_with_x_answering(game, player, card, x, |_, _| {})
}

/// [`cast_with_x`], with `script` adding the answers the resolution asks.
fn cast_with_x_answering(
    game: &mut GameState,
    player: PlayerId,
    card: Arc<CardData>,
    x: u64,
    script: impl Fn(&ScriptedDecisionProvider, ObjectId),
) -> ObjectId {
    let id = put_in_hand(game, card, player);
    cast_from_hand(game, player, id, x, script);
    id
}

/// Cast the card `id` in `player`'s hand with X = `x`, and resolve it.
fn cast_from_hand(game: &mut GameState, player: PlayerId, id: ObjectId, x: u64, script: impl Fn(&ScriptedDecisionProvider, ObjectId)) {
    game.players[player].mana_pool.add(ManaType::Green, x + 1);
    let decisions = ScriptedDecisionProvider::new();
    decisions.expect_number(ChoiceKind::ChooseXValue { spell_id: id, values: Box::new(LegalXValues::zero_through(x)) }, x);
    script(&decisions, id);
    game.cast_spell(player, id, &decisions).expect("it is castable");
    game.resolve_top_of_stack(&decisions).expect("it resolves");
}

// COVERS: ATOM-107.3m-001
#[test]
fn an_entry_template_reads_the_x_of_the_spell_becoming_the_permanent() {
    let mut game = setup_two_player_game();

    let grower = cast_with_x(&mut game, 0, enters_with_x_counters(), 3);
    game.check_state_based_actions(&test_dp()).expect("the check");

    assert_eq!(game.battlefield[&grower].counter_count(CounterType::PlusOnePlusOne), 3);
}

/// CR 107.3m reads the X of the spell that became the permanent; one put onto
/// the battlefield another way had none, and enters with none.
#[test]
fn an_entry_no_spell_became_reads_x_as_zero() {
    let mut game = setup_two_player_game();

    let grower = reanimate(&mut game, enters_with_x_counters(), 0);
    assert_eq!(game.battlefield[&grower].counter_count(CounterType::PlusOnePlusOne), 0);

    game.check_state_based_actions(&test_dp()).expect("the check");
    assert_eq!(game.get_object(grower).expect("its card").zone, Zone::Graveyard, "a 0/0 (CR 704.5f)");
}

// ---------------------------------------------------------------------------
// Simic Ascendancy
// ---------------------------------------------------------------------------

fn begin_step(game: &mut GameState, step: StepType, player: PlayerId) {
    game.execute_action(GameAction::BeginStep { step, player }, &test_ctx()).expect("the step begins");
}

fn growth(game: &GameState, ascendancy: ObjectId) -> u32 {
    game.battlefield.get(&ascendancy).map_or(0, |entry| entry.counter_count(CounterType::Growth))
}

/// Simic Ascendancy with `n` growth counters, under player 0.
fn ascendancy_with(game: &mut GameState, n: u64) -> ObjectId {
    let ascendancy = put_on_battlefield(game, simic_ascendancy(), 0);
    if n > 0 {
        counters_on_target(game, 0, ascendancy, put(CounterType::Growth, n));
    }
    ascendancy
}

// RULING: Simic Ascendancy #1 - "An ability that triggers when counters are put
//   on a permanent will trigger if that permanent somehow enters the
//   battlefield with those counters."
#[test]
fn simic_ascendancy_counts_the_counters_a_creature_enters_with() {
    let mut game = setup_two_player_game();
    let ascendancy = ascendancy_with(&mut game, 0);
    put_on_battlefield(&mut game, your_creatures_enter_with_a_counter_put_by(None), 0);

    reanimate(&mut game, enters_with_a_counter(), 0);
    resolve_all(&mut game);

    assert_eq!(growth(&game, ascendancy), 2, "that many: its own counter and the anthem's");
}

/// The growth counters are its own, so its activated ability and the put it
/// watches meet: one growth counter for each +1/+1 counter, creature by
/// creature, and none for a counter on an opponent's creature.
#[test]
fn simic_ascendancy_puts_that_many_growth_counters_for_each_creature() {
    let mut game = setup_two_player_game();
    let ascendancy = ascendancy_with(&mut game, 0);
    let (ours, theirs) = (put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0), put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1));

    counters_on_target(&mut game, 0, ours, put(CounterType::PlusOnePlusOne, 3));
    counters_on_target(&mut game, 1, theirs, put(CounterType::PlusOnePlusOne, 2));
    resolve_all(&mut game);

    assert_eq!(growth(&game, ascendancy), 3);
}

/// Twenty growth counters as your upkeep begins wins the game (CR 603.4).
#[test]
fn simic_ascendancy_wins_at_your_upkeep_with_twenty_growth_counters() {
    let mut game = setup_two_player_game();
    ascendancy_with(&mut game, 20);

    begin_step(&mut game, StepType::Upkeep, 0);
    resolve_all(&mut game);

    assert_eq!(game.result, Some(GameResult::Winner(0)));
}

// RULING: Simic Ascendancy #2 - "If Simic Ascendancy doesn't have twenty or
//   more growth counters on it as your upkeep begins, its last ability won't
//   trigger."
#[test]
fn simic_ascendancy_asks_for_twenty_as_the_upkeep_begins() {
    let mut game = setup_two_player_game();
    let ascendancy = ascendancy_with(&mut game, 19);

    begin_step(&mut game, StepType::Upkeep, 0);
    assert_eq!(waiting_from(&game, ascendancy), 0, "nineteen as the upkeep begins");
    counters_on_target(&mut game, 0, ascendancy, put(CounterType::Growth, 1));
    resolve_all(&mut game);

    assert_eq!(game.result, None, "the twentieth arrived after the upkeep began");
}

// RULING: Simic Ascendancy #3 - "If the last ability does trigger, but
//   counters are removed from Simic Ascendancy so it has fewer than twenty
//   remaining on it, you won't win the game."
#[test]
fn simic_ascendancy_does_not_win_once_its_counters_fall_below_twenty() {
    let mut game = setup_two_player_game();
    let ascendancy = ascendancy_with(&mut game, 20);

    begin_step(&mut game, StepType::Upkeep, 0);
    assert_eq!(waiting_from(&game, ascendancy), 1);
    counters_on_target(&mut game, 1, ascendancy, remove(CounterType::Growth, 1));
    resolve_all(&mut game);

    assert_eq!(game.result, None);
}

// RULING: Simic Ascendancy #4 - "If the last ability does trigger, but Simic
//   Ascendancy leaves the battlefield, use the number of counters it had on it
//   immediately before it left the battlefield to determine whether you win
//   the game."
#[test]
fn simic_ascendancy_wins_off_the_count_it_had_as_it_left() {
    let mut game = setup_two_player_game();
    let ascendancy = ascendancy_with(&mut game, 20);

    begin_step(&mut game, StepType::Upkeep, 0);
    assert_eq!(waiting_from(&game, ascendancy), 1);
    game.perform_sba_and_triggers(&test_dp()).expect("the trigger is put on the stack");
    game.change_zone(ascendancy, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).expect("it is exiled");
    resolve_all(&mut game);

    assert_eq!(game.result, Some(GameResult::Winner(0)));
}

// ---------------------------------------------------------------------------
// Protean Hydra
// ---------------------------------------------------------------------------

fn plus_counters(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield.get(&id).map_or(0, |entry| entry.counter_count(CounterType::PlusOnePlusOne))
}

/// Protean Hydra under player 0 with `n` +1/+1 counters, cast with X = `n`.
fn hydra_with(game: &mut GameState, n: u64) -> ObjectId {
    let hydra = cast_with_x(game, 0, protean_hydra(), n);
    game.check_state_based_actions(&test_dp()).expect("the check");
    hydra
}

fn damage(game: &mut GameState, source: ObjectId, target: ObjectId, amount: u64, unpreventable: bool) {
    game.execute_action(
        GameAction::DealDamage { source, target: ObjectOrPlayerId::Object(target), amount, is_combat: false, unpreventable, source_lki: None },
        &test_ctx(),
    )
    .expect("the damage event");
}

/// CR 107.3m on the printed card: it enters with X counters, X beside Master
/// Biomancer's counters, and one no spell became has none.
#[test]
fn protean_hydra_enters_with_x_counters_and_none_put_onto_the_battlefield() {
    let mut game = setup_two_player_game();
    let hydra = hydra_with(&mut game, 3);
    assert_eq!(plus_counters(&game, hydra), 3);

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, master_biomancer(), 0);
    // Two entry effects, so CR 616.1e asks its controller which first.
    let hydra = cast_with_x_answering(&mut game, 0, protean_hydra(), 2, |decisions, id| {
        decisions.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: Some(id) }, vec![0]);
    });
    assert_eq!(plus_counters(&game, hydra), 4, "X and the Biomancer's power");

    let mut game = setup_two_player_game();
    let hydra = reanimate(&mut game, protean_hydra(), 0);
    game.check_state_based_actions(&test_dp()).expect("the check");
    assert_eq!(game.get_object(hydra).expect("its card").zone, Zone::Graveyard);
}

// RULING: Protean Hydra #1 - "All damage that would be dealt to Protean Hydra
//   is prevented, even if it doesn't have that many +1/+1 counters on it. ...
//   all the +1/+1 counters on Protean Hydra are removed from it."
#[test]
fn protean_hydra_prevents_all_damage_and_loses_the_counters_it_has() {
    let mut game = setup_two_player_game();
    let hydra = hydra_with(&mut game, 2);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let start = game.recorded_events().records().len();

    damage(&mut game, bears, hydra, 3, false);

    assert_eq!(game.battlefield[&hydra].damage_marked, 0, "prevented, all of it");
    assert_eq!(plus_counters(&game, hydra), 0);
    let removals: Vec<GameEvent> = counter_records(&game, start);
    assert_eq!(
        removals,
        vec![GameEvent::CountersRemovedFrom {
            subject: on(&game, hydra),
            counter_type: CounterType::PlusOnePlusOne,
            n: 2,
            by: Some(0),
            now: 0,
        }],
        "three would have been removed, and two were there"
    );
    assert!(!game.recorded_events().records_from(start).iter().any(|r| matches!(r.event, GameEvent::DamageDealt { .. })));
}

// RULING: Protean Hydra #2 - "If unpreventable damage is dealt to Protean
//   Hydra, the Hydra's second ability will try to prevent it and fail ... and
//   it will also remove that many +1/+1 counters from Protean Hydra."
#[test]
fn protean_hydra_loses_counters_to_damage_that_cannot_be_prevented() {
    let mut game = setup_two_player_game();
    let hydra = hydra_with(&mut game, 3);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    damage(&mut game, bears, hydra, 2, true);

    assert_eq!(game.battlefield[&hydra].damage_marked, 2, "dealt");
    assert_eq!(plus_counters(&game, hydra), 1, "and two counters removed");
}

// RULING: Protean Hydra #3 - "Protean Hydra's last ability triggers whenever a
//   +1/+1 counter is removed from it for any reason, not just when a +1/+1
//   counter is removed by its second ability."
#[test]
fn protean_hydra_grows_back_from_any_counter_removal() {
    let mut game = setup_two_player_game();
    let hydra = hydra_with(&mut game, 3);

    counters_on_target(&mut game, 1, hydra, remove(CounterType::PlusOnePlusOne, 1));
    resolve_all(&mut game);
    assert_eq!((plus_counters(&game, hydra), game.delayed_triggers.len()), (2, 1), "one delayed trigger waits");

    begin_step(&mut game, StepType::End, 0);
    resolve_all(&mut game);
    assert_eq!(plus_counters(&game, hydra), 4, "two back at the next end step");
}

// RULING: Protean Hydra #4 - "If multiple +1/+1 counters are removed from
//   Protean Hydra at once, its last ability will trigger that many times."
#[test]
fn protean_hydra_triggers_once_for_each_counter_removed() {
    let mut game = setup_two_player_game();
    let hydra = hydra_with(&mut game, 4);

    counters_on_target(&mut game, 1, hydra, remove(CounterType::PlusOnePlusOne, 3));
    assert_eq!(waiting_from(&game, hydra), 3);
    // Each of the three makes a delayed trigger, which reads everything, so
    // their order is asked (CR 603.3b).
    resolve_all_with(&mut game, &RecordingDecisionProvider::picking(0));

    // The three delayed triggers share a def and a source still there, so
    // they go on the stack unasked: a scripted provider with nothing queued
    // is the assertion.
    begin_step(&mut game, StepType::End, 0);
    resolve_all_with(&mut game, &ScriptedDecisionProvider::new());
    assert_eq!(plus_counters(&game, hydra), 1 + 6);
}

// RULING: Protean Hydra #5 - "If a -1/-1 counter is put on Protean Hydra while
//   it has +1/+1 counters on it, that -1/-1 counter and a +1/+1 counter will be
//   removed from it as a state-based action. This will cause Protean Hydra's
//   last ability to trigger."
#[test]
fn protean_hydra_triggers_on_counter_annihilation() {
    let mut game = setup_two_player_game();
    let hydra = hydra_with(&mut game, 2);

    counters_on_target(&mut game, 1, hydra, put(CounterType::MinusOneMinusOne, 1));
    game.check_state_based_actions(&test_dp()).expect("the check");

    assert_eq!(plus_counters(&game, hydra), 1);
    assert_eq!(waiting_from(&game, hydra), 1, "CR 704.5q removed a +1/+1 counter");
}

/// A Hydra that left and came back is a new object (CR 400.7): the delayed
/// trigger its old self made finds no "it".
#[test]
fn protean_hydras_delayed_trigger_finds_nothing_once_the_hydra_is_a_new_object() {
    let mut game = setup_two_player_game();
    let hydra = hydra_with(&mut game, 2);
    counters_on_target(&mut game, 1, hydra, remove(CounterType::PlusOnePlusOne, 1));
    resolve_all(&mut game);

    game.change_zone(hydra, Zone::Hand, ZoneChangeAct::Put, &test_ctx()).expect("it is returned");
    cast_from_hand(&mut game, 0, hydra, 1, |_, _| {});
    resolve_all(&mut game);
    assert_eq!(game.delayed_triggers.len(), 1, "the old existence's trigger still waits");
    begin_step(&mut game, StepType::End, 0);
    resolve_all(&mut game);

    assert_eq!(plus_counters(&game, hydra), 1, "the same card, a new object (CR 400.7): it got nothing");
}

// RULING: Protean Hydra #6 - "If any +1/+1 counters are removed from Protean
//   Hydra during the end step, its last ability will trigger and set up a
//   delayed triggered ability that will trigger at the beginning of the next
//   end step (which is near the end of the following turn)."
#[test]
fn protean_hydra_removed_during_an_end_step_grows_at_the_next_turns() {
    let mut game = setup_two_player_game();
    let hydra = hydra_with(&mut game, 2);
    begin_step(&mut game, StepType::End, 0);

    counters_on_target(&mut game, 1, hydra, remove(CounterType::PlusOnePlusOne, 1));
    resolve_all(&mut game);
    assert_eq!((plus_counters(&game, hydra), game.delayed_triggers.len()), (1, 1), "this end step has begun already");

    game.begin_turn(game.turn_number + 1, 1);
    begin_step(&mut game, StepType::End, 1);
    resolve_all(&mut game);
    assert_eq!(plus_counters(&game, hydra), 3, "the next turn's end step");
}
