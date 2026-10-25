//! `durations-and-control` (A6e's cleanup, PR 5): a resolution's "for as
//! long as" ends with its object in every registry, and its control leg.

use std::sync::Arc;

use mtgsim::cards::authoring::{enters, triggered_ability, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lg_cards::act_of_treason;
use mtgsim::engine::layers::types::Layer;
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::objects::card_data::{CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::get_effective_controller;
use mtgsim::state::game_state::GameState;
use mtgsim::events::cause::StateBasedAction;
use mtgsim::state::game_state::{GameResult, StepType};
use mtgsim::test_support::{
    card_of_type, put_in_graveyard, put_on_battlefield, put_on_battlefield_under, setup_game, setup_two_player_game,
    static_ability, test_ctx, test_dp,
};
use mtgsim::types::card_types::{CardType, Supertype};
use mtgsim::types::effects::{
    AffectedSet, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PatternFill, PlayerRef, PlayerSet,
    Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::types::replacement::{EventPattern, ReplacementDef, Rewrite};
use mtgsim::types::restriction::{Restriction, RestrictionDef, RuleAction};
use mtgsim::types::triggers::TriggerSubject;
use mtgsim::types::zones::{Zone, ZoneChangeAct};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

fn an_opponents_creature() -> EffectRecipient {
    EffectRecipient::Target(
        SelectionFilter::Permanent(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Creature)),
            Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
        )),
        TargetCount::Exactly(1),
    )
}

/// A 2/2 whose entry does `effect` to target creature an opponent controls.
fn on_entry(name: &str, text: &'static str, effect: Primitive) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(triggered_ability(
            text,
            whenever(enters(TriggerSubject::ThisObject), Effect::Atom(effect, an_opponents_creature())),
        ))
        .build()
}

/// **Fixture**, Sower of Temptation's shape.
fn tempter() -> Arc<CardData> {
    on_entry(
        "Fixture Tempter",
        "When this creature enters, gain control of target creature an opponent controls for as long as this creature remains on the battlefield.",
        Primitive::GainControl { to: PlayerRef::You, duration: Duration::WhileSourceOnBattlefield },
    )
}

/// **Fixture**, Dragonlord Silumgar's shape.
fn dragonlord() -> Arc<CardData> {
    on_entry(
        "Fixture Dragonlord",
        "When this creature enters, gain control of target creature an opponent controls for as long as you control this creature.",
        Primitive::GainControl { to: PlayerRef::You, duration: Duration::WhileYouControlSource },
    )
}

/// **Fixture**, a "can't" and a shield that last as long as their source:
/// "When this creature enters, target creature an opponent controls can't be
/// destroyed, and if it would be destroyed, prevent that, for as long as this
/// creature remains on the battlefield."
fn warden() -> Arc<CardData> {
    let cant = RestrictionDef::new(Restriction::Event {
        pattern: EventPattern::Destroy { source: None },
        affected: AffectedSet::NOTHING,
        by: None,
    });
    let shield = ReplacementDef::new(EventPattern::Destroy { source: None }, ObjectSet::NO_OBJECTS, Rewrite::Prevent);
    CardDataBuilder::new("Fixture Warden")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(triggered_ability(
            "When this creature enters, target creature an opponent controls can't be destroyed, and if it would be destroyed, prevent that, for as long as this creature remains on the battlefield.",
            whenever(
                enters(TriggerSubject::ThisObject),
                Effect::Sequence(vec![
                    Effect::Atom(Primitive::Restrict(cant, Duration::WhileSourceOnBattlefield), an_opponents_creature()),
                    Effect::Atom(
                        Primitive::CreateReplacement(Box::new(shield), Duration::WhileSourceOnBattlefield, PatternFill::Authored),
                        EffectRecipient::SameInstanceAs(0),
                    ),
                ]),
            ),
        ))
        .build()
}

fn controller(game: &GameState, id: ObjectId) -> Option<PlayerId> {
    get_effective_controller(game, id)
}

/// Put the entry's trigger on the stack and resolve it.
fn resolve_its_entry(game: &mut GameState) {
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
}

/// Act of Treason's resolution, `thief` taking `victim` until end of turn.
fn steal(game: &mut GameState, thief: PlayerId, victim: ObjectId) {
    let card = act_of_treason();
    let source = put_in_graveyard(game, card.clone(), thief);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: thief,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ObjectOrPlayerId::Object(victim)]),
        replaced_event: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
}

fn control_rows(game: &GameState) -> usize {
    game.continuous_effects.iter().filter(|row| row.layer == Layer::Layer2Control).count()
}

// RULING: Sower of Temptation #2 - "If Sower of Temptation leaves the battlefield before the ability resolves, the ability will have no effect."
/// Flickered in response, the Tempter is a new object (CR 400.7), so the
/// duration its ability watches has ended before the effect would first
/// apply: CR 611.2b's "does nothing", and no row is written.
#[test]
fn a_steal_whose_source_was_flickered_in_response_never_starts() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let tempter = put_on_battlefield(&mut game, tempter(), 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    game.change_zone(tempter, Zone::Exile, ZoneChangeAct::Put, &test_ctx()).unwrap();
    game.change_zone(tempter, Zone::Battlefield, ZoneChangeAct::Put, &test_ctx()).unwrap();
    assert!(game.battlefield.contains_key(&tempter), "back, as a new object");

    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(controller(&game, bears), Some(1), "CR 611.2b: the effect does nothing");
    assert_eq!(control_rows(&game), 0, "and writes no row");
}

/// CR 611.2b in the other two registries: a resolution's "can't" and its
/// replacement effect "for as long as" their source remains end as it leaves.
#[test]
fn a_cant_and_a_shield_for_as_long_as_their_source_remains_end_as_it_leaves() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grizzly_bears(), 1);
    let warden = put_on_battlefield(&mut game, warden(), 0);
    resolve_its_entry(&mut game);
    assert_eq!((game.restrictions.len(), game.replacement_effects.len()), (1, 1), "a row in each");

    game.change_zone(warden, Zone::Graveyard, ZoneChangeAct::Sacrificed, &test_ctx()).unwrap();
    assert_eq!((game.restrictions.len(), game.replacement_effects.len()), (0, 0), "both ended with their source");
}

// RULING: Dragonlord Silumgar #3 - "If another player gains control of Dragonlord Silumgar, its control-change effect ends. Regaining control of Dragonlord Silumgar won't cause you to regain control of the creature or planeswalker."
/// "For as long as you control" ends as another player gains control of the
/// source, and the stolen creature goes back. Taking the source back later
/// does not begin it again.
#[test]
fn a_steal_for_as_long_as_you_control_its_source_ends_when_another_player_takes_it() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let dragonlord = put_on_battlefield(&mut game, dragonlord(), 0);
    resolve_its_entry(&mut game);
    assert_eq!(controller(&game, bears), Some(0));

    steal(&mut game, 1, dragonlord);
    assert_eq!(controller(&game, dragonlord), Some(1));
    assert_eq!(controller(&game, bears), Some(1), "the duration ended with the change of control");

    steal(&mut game, 0, dragonlord);
    assert_eq!(controller(&game, dragonlord), Some(0));
    assert_eq!(controller(&game, bears), Some(1), "and regaining control does not restore it");
}

// RULING: Dragonlord Silumgar #2 - "If Dragonlord Silumgar ceases to be under your control before its ability resolves, you won't gain control of the creature or planeswalker at all."
/// The Dragonlord changes hands with its trigger on the stack: its
/// controller when it triggered is no longer its controller, so the duration
/// never starts (CR 611.2b, 603.3a).
// COVERS: ATOM-611.2b-001
#[test]
fn a_steal_for_as_long_as_you_control_its_source_never_starts_once_it_changed_hands() {
    let mut game = setup_two_player_game();
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let dragonlord = put_on_battlefield(&mut game, dragonlord(), 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    steal(&mut game, 1, dragonlord);

    game.resolve_top_of_stack(&test_dp()).unwrap();
    assert_eq!(controller(&game, bears), Some(1));
    assert_eq!(control_rows(&game), 1, "the one row is the thief's");
}

/// **Fixture**, Akroan Horse's entry: "When this creature enters, an
/// opponent gains control of it."
fn trojan_horse() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Trojan Horse")
        .card_type(CardType::Creature)
        .power_toughness(0, 4)
        .ability(triggered_ability(
            "When this creature enters, an opponent gains control of it.",
            whenever(
                enters(TriggerSubject::ThisObject),
                Effect::Atom(
                    Primitive::GainControl { to: PlayerRef::Opponent, duration: Duration::Indefinite },
                    EffectRecipient::ThisObject,
                ),
            ),
        ))
        .build()
}

// RULING: Akroan Horse #1 - "Akroan Horse's enters-the-battlefield ability doesn't target any opponent. In a multiplayer game, you choose the opponent as the ability resolves."
/// CR 608.2d: with two opponents, the Horse's controller chooses which gains
/// control of it as the ability resolves.
// COVERS-PARTIAL: ATOM-608.2d-001
#[test]
fn an_opponent_gains_control_is_chosen_as_it_resolves_among_two_opponents() {
    let mut game = setup_game(3);
    let horse = put_on_battlefield(&mut game, trojan_horse(), 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::ChooseOpponent { source: horse }, vec![1]);

    game.resolve_top_of_stack(&dp).unwrap();
    assert_eq!(controller(&game, horse), Some(2), "the second of its two opponents, as chosen");
}

/// CR 102.2: with one opponent there is nothing to choose, and nothing is
/// asked.
#[test]
fn an_opponent_gains_control_asks_nothing_with_one_opponent() {
    let mut game = setup_two_player_game();
    let horse = put_on_battlefield(&mut game, trojan_horse(), 0);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    let dp = ScriptedDecisionProvider::new();

    game.resolve_top_of_stack(&dp).unwrap();
    assert_eq!(controller(&game, horse), Some(1));
    assert!(dp.asked().is_empty(), "asked {:?}", dp.asked());
}

/// An enchantment whose one static ability says `rule` does not apply to
/// what `affected` names.
fn rule_does_not_apply(name: &str, rule: RuleAction, affected: AffectedSet) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Restriction(Box::new(RestrictionDef::new(Restriction::RuleDoesNotApply {
            rule,
            affected,
        })))))
        .build()
}

fn creatures_you_control() -> AffectedSet {
    AffectedSet::objects(ObjectSet::battlefield_filter(ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    )))
}

/// CR 514.2's wipe skips what "damage isn't removed from creatures you
/// control during cleanup steps" names, and only that: the opponent's
/// creature's damage wears off.
#[test]
fn damage_a_card_says_is_not_removed_survives_the_cleanup_step() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        rule_does_not_apply("Fixture Festering Ward", RuleAction::CleanupDamageRemoval, creatures_you_control()),
        0,
    );
    let ours = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let theirs = put_on_battlefield(&mut game, grizzly_bears(), 1);
    for id in [ours, theirs] {
        game.battlefield.get_mut(&id).unwrap().damage_marked = 1;
    }

    while !(game.active_player == 1 && game.phase.step == Some(StepType::Upkeep)) {
        game.advance_turn(&test_dp()).unwrap();
    }
    let damage = |id: ObjectId| game.battlefield[&id].damage_marked;
    assert_eq!((damage(ours), damage(theirs)), (1, 0), "kept into the next turn, and worn off");
}

/// Phyrexian Unlife's "you don't lose the game for having 0 or less life":
/// CR 704.5a does not apply to its controller, and 704.5c still does. The
/// check's one loss per player names its first reason (CR 704.7), so the
/// exemption is asked of each reason before they collapse.
#[test]
fn a_player_a_card_keeps_at_zero_life_still_loses_to_poison() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        rule_does_not_apply(
            "Fixture Unlife",
            RuleAction::StateBased(StateBasedAction::ZeroLife),
            AffectedSet::players(PlayerSet::You),
        ),
        0,
    );
    game.players[0].life_total = 0;
    game.check_state_based_actions(&test_dp()).unwrap();
    assert!(game.result.is_none(), "CR 704.5a does not apply to player 0");

    game.players[0].add_counters(CounterType::Poison, 10);
    game.check_state_based_actions(&test_dp()).unwrap();
    assert_eq!(game.result, Some(GameResult::Winner(1)), "CR 704.5c does");
}

/// Mirror Gallery's "the legend rule doesn't apply": two legends of one name
/// under one controller stay, and nobody is asked which to keep.
#[test]
fn legends_the_legend_rule_does_not_apply_to_stay_unasked() {
    let mut game = setup_two_player_game();
    put_on_battlefield(
        &mut game,
        rule_does_not_apply(
            "Fixture Gallery",
            RuleAction::StateBased(StateBasedAction::LegendRule),
            AffectedSet::objects(ObjectSet::battlefield_filter(ObjectFilter::All)),
        ),
        0,
    );
    let legend = || {
        CardDataBuilder::new("Fixture Legend")
            .supertype(Supertype::Legendary)
            .card_type(CardType::Creature)
            .power_toughness(2, 2)
            .build()
    };
    let first = put_on_battlefield(&mut game, legend(), 0);
    let second = put_on_battlefield(&mut game, legend(), 0);
    let dp = ScriptedDecisionProvider::new();

    game.check_state_based_actions(&dp).unwrap();
    assert!(game.battlefield.contains_key(&first) && game.battlefield.contains_key(&second));
    assert!(dp.asked().is_empty(), "asked {:?}", dp.asked());
}

/// Homeward Path's activated ability, "each player gains control of all
/// creatures they own", resolved by player 0: each creature goes to its own
/// owner (CR 108.3), read per object as the walk reads `Owner`, never as
/// the owner of the ability's source.
#[test]
fn each_player_gains_control_of_what_they_own_object_by_object() {
    let mut game = setup_two_player_game();
    let path = put_on_battlefield(&mut game, card_of_type("Fixture Homeward Path", CardType::Land), 0);
    let mine_with_them = put_on_battlefield_under(&mut game, grizzly_bears(), 0, 1);
    let theirs_with_me = put_on_battlefield_under(&mut game, grizzly_bears(), 1, 0);
    let homecoming = Effect::Atom(
        Primitive::GainControl { to: PlayerRef::Owner, duration: Duration::Indefinite },
        EffectRecipient::FilteredPermanents(ObjectFilter::ByType(CardType::Creature)),
    );
    let ctx = ResolutionContext {
        source: path,
        ability_source: Some(game.object_ref(path).unwrap()),
        controller: 0,
        generated_by: GeneratedBy::ActivatedAbility,
        targets: ChosenTargets::one(Vec::new()),
        replaced_event: None,
        trigger: None,
    };
    game.resolve_effect(&homecoming, &ctx, &test_dp()).unwrap();

    assert_eq!((controller(&game, mine_with_them), controller(&game, theirs_with_me)), (Some(0), Some(1)));
}
