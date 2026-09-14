//! Phase RE-5 — counters, on permanents and players.
//!
//! CR 614.16's counter half, CR 122.1, CR 122.6 and CR 122.6a, against the six
//! printed cards in `cards::phase_re_cards` and the engine's own shape: a
//! counter's subject is an object or a player, the putter rides on the event,
//! and an entry that gives a permanent counters is watched through a second
//! door on `EventPattern::CounterChange` rather than proposed as a second
//! event.
//!
//! **Every board here is about which of two doors an effect meets** — an
//! `AddCounters` proposal, or an entry whose mods carry counters — and about
//! who is putting them on.

use std::sync::Arc;

use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeCause};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::events::event::{CounterSubject, GameEvent};
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    put_in_graveyard, put_in_hand, put_on_battlefield, setup_two_player_game, static_ability,
    test_ctx, test_dp, vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AffectedSet, AmountExpr, CounterType, Effect, EffectRecipient, ObjectFilter, PlayerRef,
    PlayerSet, Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::replacement::{
    AmountRewrite, EnterModsTemplate, EventPattern, ReplacementDef, Rewrite, Rounding,
};
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

/// The CR 616.1 prompt, matched by kind alone.
const PICK_REPLACEMENT: ChoiceKind = ChoiceKind::ChooseReplacementEffect { affected_object: None };

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
    targets: Vec<ResolvedTarget>,
    effect: &Effect,
    dp: &dyn DecisionProvider,
) {
    let source = put_in_hand(game, fixture_object(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        targets,
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(effect, &ctx, dp).expect("resolving");
}

/// "Put `n` `kind` counters on target permanent", resolved by `player` on
/// `target`.
fn put_counters_on(
    game: &mut GameState,
    player: PlayerId,
    target: ObjectId,
    kind: CounterType,
    n: u64,
    dp: &dyn DecisionProvider,
) {
    let effect = Effect::Atom(
        Primitive::AddCounters(kind, AmountExpr::Fixed(n)),
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)),
    );
    resolve_targeting(game, player, vec![ResolvedTarget::Object(target)], &effect, dp);
}

/// How many `kind` counters `id` carries.
fn count(game: &GameState, id: ObjectId, kind: CounterType) -> u32 {
    game.battlefield.get(&id).map(|e| e.counter_count(kind)).unwrap_or(0)
}

/// Return a card from a graveyard to the battlefield — CR 614.1c's entry,
/// proposed through the chokepoint.
fn reanimate(game: &mut GameState, card: Arc<CardData>, player: PlayerId) -> ObjectId {
    reanimate_with(game, card, player, &test_dp())
}

/// [`reanimate`] with a chosen provider, for a board that asks.
fn reanimate_with(
    game: &mut GameState,
    card: Arc<CardData>,
    player: PlayerId,
    dp: &dyn DecisionProvider,
) -> ObjectId {
    let id = put_in_graveyard(game, card, player);
    let ctx = ActionContext { dp, resolution: None };
    game.change_zone(id, Zone::Battlefield, ZoneChangeCause::Returned, &ctx).expect("it enters");
    id
}

/// A fixture enchantment carrying one static replacement.
fn fixture_static(name: &str, def: ReplacementDef) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Replacement(Box::new(def))))
        .build()
}

/// "If one or more counters would be put on a permanent you control, twice
/// that many are put on it instead" — the engine's own doubler, with no card
/// behind it.
fn fixture_doubler() -> Arc<CardData> {
    fixture_static(
        "Fixture Counter Doubler",
        ReplacementDef::new(
            EventPattern::CounterChange { counter: None, adding: true, by: None },
            AffectedSet::Filter { filter: ObjectFilter::ByController(PlayerRef::You) },
            Rewrite::Amount(AmountRewrite::Multiplier(2)),
        ),
    )
}

/// A doubler over every permanent, asking `by` — Vorinclex's first half with
/// no card behind it.
fn fixture_doubler_by(by: PlayerSet) -> Arc<CardData> {
    fixture_static(
        "Fixture Putter Doubler",
        ReplacementDef::new(
            EventPattern::CounterChange { counter: None, adding: true, by: Some(by) },
            AffectedSet::Filter { filter: ObjectFilter::All },
            Rewrite::Amount(AmountRewrite::Multiplier(2)),
        ),
    )
}

/// "If an opponent would put one or more counters on a permanent, they put
/// half that many, rounded down" — Vorinclex's second half, as a fixture.
fn fixture_halver() -> Arc<CardData> {
    fixture_static(
        "Fixture Halver",
        ReplacementDef::new(
            EventPattern::CounterChange {
                counter: None,
                adding: true,
                by: Some(PlayerSet::Opponents),
            },
            AffectedSet::Filter { filter: ObjectFilter::All },
            Rewrite::Amount(AmountRewrite::Halve(Rounding::Down)),
        ),
    )
}

/// A doubler whose filter reads power — the one leaf `EnterMods` feeds.
fn fixture_doubler_on_small_creatures() -> Arc<CardData> {
    fixture_static(
        "Fixture Small Doubler",
        ReplacementDef::new(
            EventPattern::CounterChange { counter: None, adding: true, by: None },
            AffectedSet::Filter { filter: ObjectFilter::PowerLE(2) },
            Rewrite::Amount(AmountRewrite::Multiplier(2)),
        ),
    )
}

/// "Creatures you control enter with a +1/+1 counter on them" — an
/// `EnterWith` from another permanent, Master Biomancer's shape with a
/// constant.
fn your_creatures_enter_with_a_counter() -> Arc<CardData> {
    fixture_static(
        "Fixture Anthem of Counters",
        ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            AffectedSet::Filter {
                filter: ObjectFilter::And(
                    Box::new(ObjectFilter::ByType(CardType::Creature)),
                    Box::new(ObjectFilter::ByController(PlayerRef::You)),
                ),
            },
            Rewrite::EnterWith(EnterModsTemplate::with_counters(CounterType::PlusOnePlusOne, 1)),
        ),
    )
}

/// "Permanents you control enter with a charge counter on them".
fn your_permanents_enter_with_a_charge_counter() -> Arc<CardData> {
    fixture_static(
        "Fixture Charger",
        ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            AffectedSet::Filter { filter: ObjectFilter::ByController(PlayerRef::You) },
            Rewrite::EnterWith(EnterModsTemplate::with_counters(CounterType::Charge, 1)),
        ),
    )
}

/// The signed counter changes in the log since `start`.
fn counter_changes(game: &GameState, start: usize) -> Vec<(CounterSubject, CounterType, i32)> {
    game.events
        .records_from(start)
        .iter()
        .filter_map(|r| match &r.event {
            GameEvent::CountersChanged { subject, counter, added } => {
                Some((*subject, *counter, *added))
            }
            _ => None,
        })
        .collect()
}

/// A 3/3 that enters with two -1/-1 counters — Chainbreaker's shape, as a
/// fixture.
fn enters_with_two_minus_counters() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Chained")
        .card_type(CardType::Creature)
        .power_toughness(3, 3)
        .ability(static_ability(Effect::Replacement(Box::new(ReplacementDef::new(
            EventPattern::EnterBattlefield { cast: None },
            AffectedSet::SourceOnly,
            Rewrite::EnterWith(EnterModsTemplate::with_counters(CounterType::MinusOneMinusOne, 2)),
        )))))
        .build()
}

/// A planeswalker with printed loyalty 3 and nothing else.
fn loyalty_three() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Walker")
        .card_type(CardType::Planeswalker)
        .loyalty(3)
        .build()
}

// ---------------------------------------------------------------------------
// The engine's shape, with no card behind it
// ---------------------------------------------------------------------------

/// `Rewrite::Amount` over an `AddCounters` — the arm RB's pattern existed
/// for and nothing applied until now.
#[test]
fn a_multiplier_over_a_counter_proposal_multiplies_the_count() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_doubler(), 0);
    let bears = put_on_battlefield(&mut game, mtgsim::test_support::vanilla_creature(2, 2, &[]), 0);

    put_counters_on(&mut game, 0, bears, CounterType::PlusOnePlusOne, 1, &test_dp());

    assert_eq!(count(&game, bears, CounterType::PlusOnePlusOne), 2);
}

/// The entry door — CR 122.6's "also to an object that's given counters as it
/// enters the battlefield". The counters are the entry's mods, and the doubler
/// meets them at the entry's own CR 616.1 step, after the `EnterWith` that put
/// them there.
#[test]
fn a_multiplier_meets_the_counters_a_permanent_enters_with() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_doubler(), 0);

    let chained = reanimate(&mut game, enters_with_two_minus_counters(), 0);

    assert_eq!(count(&game, chained, CounterType::MinusOneMinusOne), 4);
}

/// CR 306.5b's loyalty is the entry's seed, and the seed goes through the same
/// door — Doubling Season's own ruling: "planeswalkers will enter with double
/// the normal number of loyalty counters".
#[test]
fn a_multiplier_meets_a_planeswalkers_printed_loyalty() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_doubler(), 0);

    let walker = reanimate(&mut game, loyalty_three(), 0);

    assert_eq!(count(&game, walker, CounterType::Loyalty), 6);
}

/// Two multipliers are the bucket `ordering_cannot_change_outcome` proves —
/// on a proposal and on an entry alike — so neither board asks. The entry
/// half is the one item 47's condition (c) is about: the door reads the
/// entry's mods, and a multiplier of one or more leaves every kind on "one
/// or more"'s side.
#[test]
fn two_multipliers_ask_nothing_at_either_door() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_doubler(), 0);
    put_on_battlefield(&mut game, fixture_doubler(), 0);
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let dp = RecordingDecisionProvider::picking(0);

    put_counters_on(&mut game, 0, bears, CounterType::PlusOnePlusOne, 1, &dp);
    let walker = reanimate_with(&mut game, loyalty_three(), 0, &dp);

    assert_eq!(count(&game, bears, CounterType::PlusOnePlusOne), 4);
    assert_eq!(count(&game, walker, CounterType::Loyalty), 12);
    assert_eq!(dp.prompts(), 0, "a bucket of multipliers has one outcome");
}

/// The premise's entry clause, checked from the side that fails it: a
/// multiplier whose filter reads power stops applying once another member
/// has raised the +1/+1 count past it, so over an *entry* the pair is a real
/// two-outcome order and is asked — and over a proposal, where the filter
/// reads the finished board, both apply and nothing is asked.
#[test]
fn a_multiplier_reading_power_is_asked_at_the_entry_door_only() {
    let at_the_entry = |pick: usize| -> u32 {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, fixture_doubler(), 0);
        put_on_battlefield(&mut game, fixture_doubler_on_small_creatures(), 0);
        put_on_battlefield(&mut game, your_creatures_enter_with_a_counter(), 0);
        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(PICK_REPLACEMENT, vec![pick]);
        let bears = reanimate_with(&mut game, vanilla_creature(1, 1, &[]), 0, &dp);
        assert!(dp.is_empty(), "one prompt");
        count(&game, bears, CounterType::PlusOnePlusOne)
    };
    let outcomes = [at_the_entry(0), at_the_entry(1)];
    assert!(
        outcomes.contains(&2) && outcomes.contains(&4),
        "the plain doubler first takes the 1/1 to 3 power and the small doubler          falls out; the small doubler first leaves both applied — got {:?}",
        outcomes
    );

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_doubler(), 0);
    put_on_battlefield(&mut game, fixture_doubler_on_small_creatures(), 0);
    let bears = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let dp = RecordingDecisionProvider::picking(0);
    put_counters_on(&mut game, 0, bears, CounterType::PlusOnePlusOne, 1, &dp);
    assert_eq!(count(&game, bears, CounterType::PlusOnePlusOne), 4);
    assert_eq!(dp.prompts(), 0, "the filter reads the board, which the count does not touch");
}

/// The CR 616.2 board with no card: a doubler is not applicable to an entry
/// until an `EnterWith` puts counters in its mods, so the two never share an
/// iteration and nothing is asked — and the doubler sees what the `EnterWith`
/// wrote.
#[test]
fn a_multiplier_is_offered_only_once_an_enters_with_gives_the_entry_counters() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_doubler(), 0);
    put_on_battlefield(&mut game, your_creatures_enter_with_a_counter(), 0);
    let dp = RecordingDecisionProvider::picking(0);

    let bears = reanimate_with(&mut game, vanilla_creature(2, 2, &[]), 0, &dp);

    assert_eq!(count(&game, bears, CounterType::PlusOnePlusOne), 2);
    assert_eq!(dp.prompts(), 0);
}

/// A multiplier beside an `EnterWith` of a *different* kind, both applicable
/// at once, is CR 616.1e's real choice: CR 614.5 gives the doubler one
/// opportunity, so counters an `EnterWith` adds after it are not doubled.
#[test]
fn a_multiplier_beside_an_enters_with_is_a_real_order() {
    let run = |pick: usize| -> (u32, u32) {
        let mut game = setup_two_player_game();
        put_on_battlefield(&mut game, fixture_doubler(), 0);
        put_on_battlefield(&mut game, your_permanents_enter_with_a_charge_counter(), 0);
        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(PICK_REPLACEMENT, vec![pick]);
        let walker = reanimate_with(&mut game, loyalty_three(), 0, &dp);
        assert!(dp.is_empty(), "one prompt");
        (count(&game, walker, CounterType::Loyalty), count(&game, walker, CounterType::Charge))
    };
    let outcomes = [run(0), run(1)];
    assert!(
        outcomes.contains(&(6, 1)) && outcomes.contains(&(6, 2)),
        "doubler first: 6 loyalty and 1 charge; charger first: 6 and 2 — got {:?}",
        outcomes
    );
}

// ---------------------------------------------------------------------------
// Who is putting them on (CR 122.6a)
// ---------------------------------------------------------------------------

/// `by` reads the proposal's putter — the resolving effect's controller — and,
/// at the entry door, CR 122.6a's default: the controller the permanent enters
/// under.
#[test]
fn the_putter_is_the_effects_controller_and_the_entering_controller() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_doubler_by(PlayerSet::You), 0);
    let mine = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    put_counters_on(&mut game, 0, theirs, CounterType::PlusOnePlusOne, 1, &test_dp());
    put_counters_on(&mut game, 1, mine, CounterType::PlusOnePlusOne, 1, &test_dp());
    let my_walker = reanimate(&mut game, loyalty_three(), 0);
    let their_walker = reanimate(&mut game, loyalty_three(), 1);

    assert_eq!(count(&game, theirs, CounterType::PlusOnePlusOne), 2, "I put them on");
    assert_eq!(count(&game, mine, CounterType::PlusOnePlusOne), 1, "the opponent put them on");
    assert_eq!(count(&game, my_walker, CounterType::Loyalty), 6, "I control what enters");
    assert_eq!(count(&game, their_walker, CounterType::Loyalty), 3, "the opponent does");
}

/// A halving that reaches zero: "one or more" is asked by the pattern, so
/// nothing further matches, and the zero meets `perform_action`'s no-op guard
/// — the event is *performed as decided* and announces nothing — rather than
/// `never_happens`, which would have dropped it from the batch.
#[test]
fn a_count_halved_to_zero_meets_the_performers_guard_not_never_happens() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_halver(), 0);
    put_on_battlefield(&mut game, fixture_doubler_by(PlayerSet::Opponents), 0);
    let mine = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let start = game.events.records().len();
    let dp = RecordingDecisionProvider::picking(0);

    let performed = game
        .execute_actions(
            vec![GameAction::AddCounters {
                subject: CounterSubject::Object(mine),
                counter: CounterType::PlusOnePlusOne,
                n: 1,
                by: 1,
            }],
            &ActionContext { dp: &dp, resolution: None },
        )
        .expect("it performs");

    assert_eq!(count(&game, mine, CounterType::PlusOnePlusOne), 0);
    assert_eq!(counter_changes(&game, start), vec![], "a no-op announces nothing");
    assert!(
        matches!(performed.as_slice(), [GameAction::AddCounters { n: 0, .. }]),
        "performed as decided, with the count the halving left: {:?}",
        performed
    );
    assert_eq!(dp.prompts(), 1, "a halving beside a doubler is a real order: 1 to 0, or 1 to 2 to 1");
}

/// The same halving at the entry door: an opponent's permanent entering with
/// one counter enters with none, and the kind leaves the mods rather than
/// arriving as a zero.
#[test]
fn an_entry_kind_halved_to_zero_leaves_the_mods() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_halver(), 0);
    put_on_battlefield(&mut game, your_creatures_enter_with_a_counter(), 1);
    let theirs = reanimate(&mut game, vanilla_creature(2, 2, &[]), 1);
    assert_eq!(count(&game, theirs, CounterType::PlusOnePlusOne), 0);
    assert!(game.battlefield[&theirs].counters.is_empty(), "no kind at zero");

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, fixture_halver(), 0);
    let chained = reanimate(&mut game, enters_with_two_minus_counters(), 1);
    assert_eq!(count(&game, chained, CounterType::MinusOneMinusOne), 1, "two, halved down");
}

/// Nothing prints a remover, so a removal pattern that asks `by` matches
/// nothing and the removal goes through.
#[test]
fn a_removal_pattern_naming_a_putter_matches_nothing() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        fixture_static(
            "Fixture Misauthored",
            ReplacementDef::new(
                EventPattern::CounterChange { counter: None, adding: false, by: Some(PlayerSet::You) },
                AffectedSet::Filter { filter: ObjectFilter::All },
                Rewrite::Prevent,
            ),
        ),
        0,
    );
    let bears = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.add_counters(bears, CounterType::PlusOnePlusOne, 2);

    let effect = Effect::Atom(
        Primitive::RemoveCounters(CounterType::PlusOnePlusOne, AmountExpr::Fixed(1)),
        EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)),
    );
    resolve_targeting(&mut game, 0, vec![ResolvedTarget::Object(bears)], &effect, &test_dp());

    assert_eq!(count(&game, bears, CounterType::PlusOnePlusOne), 1);
}

// ---------------------------------------------------------------------------
// A player as the subject (CR 122.1)
// ---------------------------------------------------------------------------

/// "You get {E}{E}": the same event with a player as its subject. The map
/// has a kind, the log has the line, and a doubler scoped to the player
/// watches it through the one arm.
#[test]
fn a_player_gets_counters_through_the_same_event() {
    let mut game = setup_two_player_game();
    let start = game.events.records().len();
    let effect = Effect::Atom(
        Primitive::GetCounters(CounterType::Energy, AmountExpr::Fixed(2)),
        EffectRecipient::Controller,
    );

    resolve_targeting(&mut game, 0, vec![], &effect, &test_dp());
    assert_eq!(game.players[0].counter_count(CounterType::Energy), 2);
    assert_eq!(
        counter_changes(&game, start),
        vec![(CounterSubject::Player(0), CounterType::Energy, 2)]
    );

    put_on_battlefield(
        &mut game,
        fixture_static(
            "Fixture Energy Doubler",
            ReplacementDef::new(
                EventPattern::CounterChange { counter: Some(CounterType::Energy), adding: true, by: None },
                AffectedSet::NO_OBJECTS,
                Rewrite::Amount(AmountRewrite::Multiplier(2)),
            )
            .affecting_players(PlayerSet::You),
        ),
        0,
    );
    resolve_targeting(&mut game, 0, vec![], &effect, &test_dp());
    assert_eq!(game.players[0].counter_count(CounterType::Energy), 6, "2, then 2 doubled");
    resolve_targeting(&mut game, 1, vec![], &effect, &test_dp());
    assert_eq!(game.players[1].counter_count(CounterType::Energy), 2, "not mine to double");
}

/// A removal from a player takes as much as it can (CR 701.2) and announces
/// only a transition, the object arm's mirror.
#[test]
fn a_player_removal_takes_as_much_as_it_can() {
    let mut game = setup_two_player_game();
    game.players[0].add_counters(CounterType::Energy, 3);
    let start = game.events.records().len();
    let remove = |n: u32| GameAction::RemoveCounters {
        subject: CounterSubject::Player(0),
        counter: CounterType::Energy,
        n,
    };

    game.execute_action(remove(5), &test_ctx()).expect("it performs");
    game.execute_action(remove(1), &test_ctx()).expect("removing nothing is not an error");

    assert_eq!(game.players[0].counter_count(CounterType::Energy), 0);
    assert!(game.players[0].counters.is_empty());
    assert_eq!(
        counter_changes(&game, start),
        vec![(CounterSubject::Player(0), CounterType::Energy, -3)],
        "one transition, and the removal of nothing announced nothing"
    );
}
