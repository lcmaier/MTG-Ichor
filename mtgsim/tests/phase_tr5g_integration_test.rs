//! Phase TR-5g — the multiplier, tokens and scry (CR 603.2d, 701.7, 701.22).
//!
//! 1. CR 603.2d's count: a multiplier on the board the matched arm reads
//!    adds its `additional` to a trigger of a permanent it names whose
//!    triggering event is the event it names. Panharmonicon, and Gandalf the
//!    White's fixture for a leaving read off its frame. Every board here runs
//!    under the dispatch audit, which counts again with no shortcut.

use std::sync::Arc;

use mtgsim::cards::artifacts::sol_ring;
use mtgsim::cards::authoring::{
    another, becomes_the_target_of, creates_a_token, dies, enters, is_sacrificed, leaves_the_battlefield, scries, triggered_ability, when_you_do,
    whenever, Whose,
};
use mtgsim::cards::phase_ld_cards::march_of_the_machines;
use mtgsim::cards::basic_lands::forest;
use mtgsim::cards::phase_cv_cards::cytoshape;
use mtgsim::cards::phase_rc_cards::chainbreaker;
use mtgsim::cards::phase_re_cards::{divine_visitation, doubling_season};
use mtgsim::cards::phase_tr1_cards::{soul_warden, wild_growth};
use mtgsim::cards::phase_tr5g_cards::panharmonicon;
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeAct};
use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, Layer};
use mtgsim::engine::resolve::ResolutionContext;
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::state::game_state::{GameResult, GameState};
use mtgsim::test_support::{
    creature_with_ability, fill_library, put_in_graveyard, put_in_hand, put_on_battlefield, registered, setup_two_player_game, static_ability,
    test_dp, vanilla_creature,
};
use mtgsim::events::event::GameEvent;
use mtgsim::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    Antecedent, AmountExpr, Condition, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerFact, PlayerRef, PlayerSet, Primitive,
    SelectionFilter, TargetCount, TokenDef,
};
use mtgsim::types::history::CountIs;
use mtgsim::types::ids::{new_ability_id, AbilityId, ObjectId, ObjectOrPlayerId, PlayerId};
use mtgsim::oracle::x_values::LegalXValues;
use mtgsim::types::mana::{ManaCost, ManaSymbol, ManaType};
use mtgsim::types::triggers::{
    ParticipantFilter, DelayedDuration, DelayedTriggerTemplate, DelayedTurn, Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerLimit,
    TargetedBy, TriggerMultiplierDef, TriggerSubject, TriggerTier,
};
use mtgsim::types::zones::{Zone, ZoneChangePattern};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A two-player game whose every dispatch the audit answers again.
fn audited_game() -> GameState {
    let mut game = setup_two_player_game();
    game.enable_audit();
    game
}

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You))
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

/// The triggers `source` has waiting to be put on the stack.
fn waiting_from(game: &GameState, source: ObjectId) -> usize {
    game.pending_triggers.iter().filter(|p| p.origin.source() == source).count()
}

fn resolve_all(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).expect("SBAs and triggers");
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(dp).expect("it resolves");
        game.perform_sba_and_triggers(dp).expect("SBAs and triggers");
    }
}

/// Put a card from a graveyard onto the battlefield, through the chokepoint.
fn reanimate(game: &mut GameState, card: Arc<CardData>, player: PlayerId) -> ObjectId {
    let id = put_in_graveyard(game, card, player);
    game.change_zone(id, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(&test_dp())).expect("it enters");
    id
}

/// Put `cards` from their owner's graveyard onto the battlefield in one event.
fn reanimate_together(game: &mut GameState, cards: Vec<Arc<CardData>>, player: PlayerId) -> Vec<ObjectId> {
    let ids: Vec<ObjectId> = cards.into_iter().map(|card| put_in_graveyard(game, card, player)).collect();
    let entries = ids
        .iter()
        .map(|&object| GameAction::EnterBattlefield {
            object,
            from: Some(Zone::Graveyard),
            controller: player,
            mods: mtgsim::types::replacement::EnterMods::NONE,
            act: Some(ZoneChangeAct::Put),
        })
        .collect();
    game.execute_actions(entries, &ActionContext::new(&test_dp())).expect("they enter");
    ids
}

/// A 1/1 with one triggered ability.
fn watcher(name: &str, def: TriggerDef) -> Arc<CardData> {
    creature_with_ability(name, 1, 1, triggered_ability("", def))
}

fn a_creature() -> ObjectFilter {
    ObjectFilter::ByType(CardType::Creature)
}

/// "When this creature enters, you gain 1 life."
fn healer() -> Arc<CardData> {
    watcher("Healer", whenever(enters(TriggerSubject::Object(Antecedent::ThisObject)), gain_one()))
}

fn legendary_creature(name: &str) -> Arc<CardData> {
    CardDataBuilder::new(name).card_type(CardType::Creature).supertype(Supertype::Legendary).power_toughness(2, 2).build()
}

fn artifact_creature(name: &str) -> Arc<CardData> {
    CardDataBuilder::new(name).card_type(CardType::Artifact).card_type(CardType::Creature).power_toughness(1, 1).build()
}

/// Gandalf the White's third ability, on a 4/5 legendary creature: "If a
/// legendary permanent or an artifact entering or leaving the battlefield
/// causes a triggered ability of a permanent you control to trigger, that
/// ability triggers an additional time." Its flash and its permission to
/// cast as though with flash are permission's.
fn gandalf_the_white() -> Arc<CardData> {
    let legendary_or_artifact = || {
        TriggerSubject::Filter(ObjectFilter::Or(
            Box::new(ObjectFilter::BySupertype(Supertype::Legendary)),
            Box::new(ObjectFilter::ByType(CardType::Artifact)),
        ))
    };
    let multiplier = TriggerMultiplierDef {
        caused_by: Some(TriggerCondition::AnyOf(vec![
            TriggerEvent::EntersBattlefield {
                subject: legendary_or_artifact(),
                controller: PlayerSet::Everyone,
                from: None,
                was_cast: None,
                multiplicity: Multiplicity::PerOccurrence,
            },
            TriggerEvent::ZoneChange {
                subject: legendary_or_artifact(),
                change: ZoneChangePattern { from: Some(Zone::Battlefield), to: None, act: None },
                by: None,
                owner: PlayerSet::Everyone,
                multiplicity: Multiplicity::PerOccurrence,
            },
        ])),
        abilities_of: ObjectFilter::ByController(PlayerSet::You),
        additional: 1,
    };
    CardDataBuilder::new("Gandalf the White")
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .power_toughness(4, 5)
        .ability(multiplier.into_ability(""))
        .build()
}

/// A multiplier that names no event, Roaming Throne's shape without its
/// chosen type: "a triggered ability of a permanent you control triggers
/// an additional time".
fn every_trigger_twice() -> Arc<CardData> {
    let multiplier = TriggerMultiplierDef { caused_by: None, abilities_of: ObjectFilter::ByController(PlayerSet::You), additional: 1 };
    CardDataBuilder::new("Echo Throne").card_type(CardType::Artifact).ability(multiplier.into_ability("")).build()
}

/// Panharmonicon's ability as a Layer 6 grant or a copy would put it on a
/// permanent nothing printed it on.
fn panharmonicons_ability() -> AbilityDef {
    static_ability(panharmonicon().abilities[0].effect.clone())
}

// ---------------------------------------------------------------------------
// Panharmonicon and the count (CR 603.2d)
// ---------------------------------------------------------------------------

// COVERS: ATOM-603.2d-001
// RULING: Panharmonicon #1 - it doubles other triggered abilities that trigger when a permanent enters.
#[test]
fn panharmonicon_doubles_soul_warden_on_a_creature_entering() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, panharmonicon(), 0);
    let warden = put_on_battlefield(&mut game, soul_warden(), 0);
    resolve_all(&mut game, &test_dp());
    let before = life(&game, 0);

    reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert_eq!(waiting_from(&game, warden), 2, "once, and once more");
    let ids: Vec<_> = game.pending_triggers.iter().map(|p| p.id).collect();
    assert_ne!(ids[0], ids[1], "each time is its own trigger");
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), before + 2);
}

// RULING: Panharmonicon #1 - it doubles a permanent's own enters-the-battlefield triggered abilities.
#[test]
fn panharmonicon_doubles_a_permanents_own_entering_trigger() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, panharmonicon(), 0);
    let before = life(&game, 0);
    let healer = reanimate(&mut game, healer(), 0);
    assert_eq!(waiting_from(&game, healer), 2);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), before + 2);
}

// RULING: Panharmonicon #2 - replacement effects are unaffected, so a creature that enters with counters gets no more.
// RULING: Panharmonicon #3 - an "as enters" ability is unaffected; in this engine it is a replacement (CR 614.12).
#[test]
fn panharmonicon_adds_no_counter_a_creature_enters_with() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, panharmonicon(), 0);
    let chainbreaker = reanimate(&mut game, chainbreaker(), 0);
    assert_eq!(game.battlefield[&chainbreaker].counter_count(CounterType::MinusOneMinusOne), 2);
}

// RULING: Panharmonicon #4 - you don't need to control the entering permanent, only the permanent with the triggered ability.
#[test]
fn panharmonicon_doubles_on_an_opponents_creature_entering() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, panharmonicon(), 0);
    let warden = put_on_battlefield(&mut game, soul_warden(), 0);
    resolve_all(&mut game, &test_dp());
    reanimate(&mut game, vanilla_creature(2, 2, &[]), 1);
    assert_eq!(waiting_from(&game, warden), 2);
}

/// "Of a permanent you control": an opponent's Panharmonicon is no
/// multiplier of yours, whoever's creature enters.
#[test]
fn an_opponents_panharmonicon_doubles_nothing_of_yours() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, panharmonicon(), 1);
    let warden = put_on_battlefield(&mut game, soul_warden(), 0);
    resolve_all(&mut game, &test_dp());
    reanimate(&mut game, vanilla_creature(2, 2, &[]), 1);
    assert_eq!(waiting_from(&game, warden), 1);
}

// RULING: Panharmonicon #5 - choices made on resolution are made individually for each instance.
// RULING: Teysa Karlov #2 - the same, for her count: each instance makes its own choices.
#[test]
fn each_doubled_instance_makes_its_own_choice() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, panharmonicon(), 0);
    let may_gain = Effect::Optional { chooser: PlayerRef::You, effect: Box::new(gain_one()) };
    let before = life(&game, 0);
    let healer = reanimate(&mut game, watcher("Hopeful", whenever(enters(TriggerSubject::Object(Antecedent::ThisObject)), may_gain)), 0);
    assert_eq!(waiting_from(&game, healer), 2);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::ApplyOptionalEffect { source: healer }, vec![0]);
    dp.expect_pick_n(ChoiceKind::ApplyOptionalEffect { source: healer }, Vec::new());
    resolve_all(&mut game, &dp);
    assert_eq!(life(&game, 0), before + 1, "one said yes and the other no");
}

// RULING: Panharmonicon #6 - two make three, not four.
#[test]
fn two_panharmonicons_make_three() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, panharmonicon(), 0);
    put_on_battlefield(&mut game, panharmonicon(), 0);
    let healer = reanimate(&mut game, healer(), 0);
    assert_eq!(waiting_from(&game, healer), 3);
}

// RULING: Panharmonicon #7 - a trigger on an artifact or creature entering at the same time as Panharmonicon, Panharmonicon included, triggers an additional time.
#[test]
fn a_trigger_on_entering_beside_panharmonicon_triggers_twice() {
    let mut game = audited_game();
    let warden = put_on_battlefield(&mut game, soul_warden(), 0);
    let artifacts_enter = whenever(enters(TriggerSubject::Filter(ObjectFilter::ByType(CardType::Artifact))), gain_one());
    let collector = put_on_battlefield(&mut game, watcher("Collector", artifacts_enter), 0);
    resolve_all(&mut game, &test_dp());

    reanimate_together(&mut game, vec![panharmonicon(), vanilla_creature(2, 2, &[])], 0);
    assert_eq!(waiting_from(&game, warden), 2, "the creature beside it");
    assert_eq!(waiting_from(&game, collector), 2, "Panharmonicon itself");
}

/// A creature leaving is no entering, so Panharmonicon leaves a dies trigger
/// alone.
#[test]
fn panharmonicon_leaves_a_dies_trigger_alone() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, panharmonicon(), 0);
    let mourner = put_on_battlefield(&mut game, watcher("Mourner", whenever(dies(a_creature()), gain_one())), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    resolve_all(&mut game, &test_dp());
    game.execute_action(GameAction::Destroy { object: bear }, &ActionContext::new(&test_dp())).unwrap();
    assert_eq!(waiting_from(&game, mourner), 1);
}

/// An ability that triggers an additional time has triggered twice (CR
/// 603.2d), so CR 603.3b's second tier sees two: Strict Proctor's shape, on
/// the Lenient Proctor fixture, triggers once for each of Soul Warden's.
#[test]
fn a_trigger_on_triggering_sees_each_time_a_multiplied_ability_triggers() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, panharmonicon(), 0);
    // An enchantment, so its own ability triggering is no creature's.
    let arm = TriggerEvent::AbilityTriggers { caused_by: None, source: Some(a_creature()) };
    let proctor = CardDataBuilder::new("Lenient Proctor")
        .card_type(CardType::Enchantment)
        .ability(triggered_ability("", whenever(arm, gain_one())))
        .build();
    let proctor = put_on_battlefield(&mut game, proctor, 1);
    let warden = put_on_battlefield(&mut game, soul_warden(), 0);
    resolve_all(&mut game, &test_dp());

    reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert_eq!(waiting_from(&game, warden), 2, "Panharmonicon's additional time");
    assert_eq!(waiting_from(&game, proctor), 2, "one for each time the Warden's ability triggered");
    let proctors = game.pending_triggers.iter().filter(|t| t.origin.source() == proctor);
    assert!(proctors.into_iter().all(|t| t.tier() == TriggerTier::Second));
}

/// "This ability triggers only once each turn" is read before the count: a
/// capped ability triggers once.
#[test]
fn a_capped_trigger_triggers_once_under_panharmonicon() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, panharmonicon(), 0);
    let capped = TriggerDef { limit: Some(TriggerLimit::TriggersOnlyOnceEachTurn), ..whenever(enters(another(a_creature())), gain_one()) };
    let capped = put_on_battlefield(&mut game, watcher("Capped", capped), 0);
    reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert_eq!(waiting_from(&game, capped), 1);
}

/// "Do this only once each turn" is read at the trigger, which both
/// instances pass, and as each resolves: the first takes the action.
#[test]
fn a_do_this_only_once_trigger_doubled_acts_once() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, panharmonicon(), 0);
    let grow = Effect::Optional {
        chooser: PlayerRef::You,
        effect: Box::new(Effect::Atom(
            Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You },
            EffectRecipient::Object(Antecedent::ThisObject),
        )),
    };
    let once = TriggerDef { limit: Some(TriggerLimit::DoThisOnlyOnceEachTurn), ..whenever(enters(a_creature()), grow) };
    let grower = put_on_battlefield(&mut game, watcher("Grower", once), 0);
    game.pending_triggers.clear();
    reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert_eq!(waiting_from(&game, grower), 2, "both trigger");
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::ApplyOptionalEffect { source: grower }, vec![0]);
    resolve_all(&mut game, &dp);
    assert_eq!(game.battlefield[&grower].counter_count(CounterType::PlusOnePlusOne), 1, "the second does nothing, and asks nothing");
}

/// CR 603.2d's last sentence: a delayed trigger an ability of the permanent
/// created is not that permanent's triggered ability.
#[test]
fn a_delayed_trigger_is_never_doubled() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, panharmonicon(), 0);
    let creator = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let template = DelayedTriggerTemplate {
        def: Arc::new(whenever(enters(a_creature()), gain_one())),
        duration: DelayedDuration::Once,
        turn: DelayedTurn::NoneNamed,
        rules_text: "".into(),
    };
    let create = Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(template)), EffectRecipient::Player(PlayerRef::You));
    let ctx = ResolutionContext::untargeted(creator, 0, GeneratedBy::ActivatedAbility);
    game.resolve_effect(&create, &ctx, &test_dp()).unwrap();
    reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert_eq!(waiting_from(&game, creator), 1);
}

/// A multiplier that names no event reaches a triggered mana ability, which
/// resolves at once as many times as it triggered (CR 605.4a): Wild Growth
/// doubled makes a Forest tap for three.
#[test]
fn a_doubled_triggered_mana_ability_adds_its_mana_twice() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, every_trigger_twice(), 0);
    let land = put_on_battlefield(&mut game, forest(), 0);
    let aura = put_on_battlefield(&mut game, wild_growth(), 0);
    assert!(game.attach(aura, land));
    game.activate_mana_ability(0, land, forest().abilities[0].id, &test_dp()).unwrap();
    assert_eq!(game.players[0].mana_pool.amount(ManaType::Green), 3);
    assert!(game.pending_triggers.is_empty() && game.stack.is_empty());
}

/// The granted leg: a Layer 6 row gives a vanilla creature Panharmonicon's
/// ability, which nothing printed.
#[test]
fn a_granted_multiplier_is_found_through_the_registry_summary() {
    let mut game = audited_game();
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let warden = put_on_battlefield(&mut game, soul_warden(), 0);
    resolve_all(&mut game, &test_dp());
    let ts = game.allocate_timestamp();
    let grant = EffectModification::GrantAbility(Arc::new(panharmonicons_ability()));
    game.continuous_effects.add(registered(bear, Layer::Layer6Ability, ts, grant));
    assert!(game.trigger_multiplier_sources.is_empty(), "nothing printed one");
    assert!(game.continuous_effects.summary().any_granted_trigger_multiplier);
    reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert_eq!(waiting_from(&game, warden), 2);
}

/// The copied leg: a creature that became a copy of a creature with
/// Panharmonicon's ability has it too, and the two make three.
#[test]
fn a_copied_multiplier_is_found_through_the_registry_summary() {
    let mut game = audited_game();
    let donor = put_on_battlefield(&mut game, creature_with_ability("Panharmonic Bear", 2, 2, panharmonicons_ability()), 0);
    let copyist = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let warden = put_on_battlefield(&mut game, soul_warden(), 0);
    resolve_all(&mut game, &test_dp());

    let spell = game.add_object(GameObject::new(cytoshape(), 0, Zone::Stack));
    let ctx = ResolutionContext {
        source: spell,
        ability_source: None,
        controller: 0,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(vec![ObjectOrPlayerId::Object(copyist)]),
        replaced_event: None,
        trigger: None,
    };
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(ChoiceKind::ChooseCopySource { source: spell }, vec![ChoiceOption::Object(donor)]);
    game.resolve_effect(&cytoshape().abilities[0].effect, &ctx, &dp).unwrap();
    assert!(game.continuous_effects.summary().any_copied_trigger_multiplier);

    reanimate(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert_eq!(waiting_from(&game, warden), 3);
}

// ---------------------------------------------------------------------------
// Gandalf the White: the multiplier's own event, read off the frames
// ---------------------------------------------------------------------------

// RULING: Gandalf the White #9 - the trigger event doesn't have to refer to legendary permanents, nor to leaving the battlefield.
#[test]
fn gandalf_doubles_a_from_anywhere_trigger_on_a_legendary_creature_dying() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, gandalf_the_white(), 0);
    let from_anywhere = TriggerEvent::ZoneChange {
        subject: TriggerSubject::Filter(a_creature()),
        change: ZoneChangePattern { from: None, to: Some(Zone::Graveyard), act: None },
        by: None,
        owner: PlayerSet::Everyone,
        multiplicity: Multiplicity::PerOccurrence,
    };
    let watcher = put_on_battlefield(&mut game, watcher("Gravewatcher", whenever(from_anywhere, gain_one())), 0);
    let hero = put_on_battlefield(&mut game, legendary_creature("Hero"), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    resolve_all(&mut game, &test_dp());

    game.execute_action(GameAction::Destroy { object: hero }, &ActionContext::new(&test_dp())).unwrap();
    assert_eq!(waiting_from(&game, watcher), 2, "a legendary creature left the battlefield");
    game.pending_triggers.clear();
    game.execute_action(GameAction::Destroy { object: bear }, &ActionContext::new(&test_dp())).unwrap();
    assert_eq!(waiting_from(&game, watcher), 1, "neither legendary nor an artifact");
}

// RULING: Gandalf the White #8 - an artifact leaving at the same time as Gandalf, Gandalf included, is doubled.
// RULING: Gandalf the White #10 - each permanent is looked at as it existed on the battlefield.
#[test]
fn gandalf_dying_beside_an_artifact_doubles_both_deaths() {
    let mut game = audited_game();
    let gandalf = put_on_battlefield(&mut game, gandalf_the_white(), 0);
    let golem = put_on_battlefield(&mut game, artifact_creature("Golem"), 0);
    let mourner = put_on_battlefield(&mut game, watcher("Mourner", whenever(dies(a_creature()), gain_one())), 0);
    resolve_all(&mut game, &test_dp());

    let wipe = vec![GameAction::Destroy { object: gandalf }, GameAction::Destroy { object: golem }];
    game.execute_actions(wipe, &ActionContext::new(&test_dp())).unwrap();
    assert_eq!(waiting_from(&game, mourner), 4, "each death twice, read before the event");
}

/// A sacrifice is the event that causes the death, not the death (Teysa
/// Karlov's fifth ruling, on Gandalf's board): "whenever you sacrifice"
/// triggers once and "whenever a creature dies" twice.
#[test]
fn gandalf_leaves_a_sacrifice_trigger_alone() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, gandalf_the_white(), 0);
    let sacrificed = put_on_battlefield(&mut game, watcher("Altar Keeper", whenever(is_sacrificed(a_creature()), gain_one())), 0);
    let mourner = put_on_battlefield(&mut game, watcher("Mourner", whenever(dies(a_creature()), gain_one())), 0);
    let hero = put_on_battlefield(&mut game, legendary_creature("Hero"), 0);
    resolve_all(&mut game, &test_dp());

    game.change_zone(hero, Zone::Graveyard, ZoneChangeAct::Sacrificed, &ActionContext::new(&test_dp())).unwrap();
    assert_eq!(waiting_from(&game, sacrificed), 1);
    assert_eq!(waiting_from(&game, mourner), 2);
}

// ---------------------------------------------------------------------------
// Tokens: the definition a creation made its token from (CR 111.2, 701.7)
// ---------------------------------------------------------------------------

/// "A 2/1 white Cat Warrior creature token."
fn cat_warrior() -> TokenDef {
    TokenDef {
        name: None,
        colors: vec![Color::White],
        types: vec![CardType::Creature],
        subtypes: vec![Subtype::Creature(CreatureType::Cat), Subtype::Creature(CreatureType::Warrior)],
        supertypes: Vec::new(),
        power: Some(2),
        toughness: Some(1),
        keyword_flags: Vec::new(),
        abilities: Vec::new(),
        rules_text: String::new(),
        enchant_filter: None,
        enters_tapped: false,
        enters_attacking: None,
    }
}

/// Ajani, Nacatl Avenger's 0 ability, which a loyalty ability is once `remaining-costs`
/// builds them: "Create a 2/1 white Cat Warrior creature token. When you
/// do, if you control a red permanent other than Ajani, he deals damage
/// equal to the number of creatures you control to any target."
fn ajanis_zero() -> Effect {
    let damage = Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::CountOf(ObjectSet::battlefield_filter(ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ByController(PlayerSet::You))))), unpreventable: false },
        EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
    );
    let mut when_you_do = when_you_do(creates_a_token(Whose::Yours, Some(cat_warrior())), damage, "");
    let red_other = ObjectFilter::And(Box::new(ObjectFilter::ByColor(Color::Red)), Box::new(ObjectFilter::NotSource));
    Arc::make_mut(&mut when_you_do.def).intervening_if =
        Some(Condition::Player { players: PlayerSet::You, fact: PlayerFact::ControlsPermanent(red_other) });
    Effect::Sequence(vec![
        Effect::Atom(Primitive::CreateToken(cat_warrior(), AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You)),
        Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(when_you_do)), EffectRecipient::Player(PlayerRef::You)),
    ])
}

/// Ajani, with a red creature beside him or not, his 0 ability activated
/// and resolved: an artifact's "{0}: ..." until `remaining-costs` builds loyalty.
fn ajani_activates(game: &mut GameState, red_beside: bool) -> ObjectId {
    let zero = AbilityDef {
        rules_text: "".into(),
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Activated,
        costs: Vec::new(),
        effect: ajanis_zero(),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    };
    let ajani = CardDataBuilder::new("Ajani, Nacatl Avenger").card_type(CardType::Artifact).ability(zero).build();
    let ajani = put_on_battlefield(game, ajani, 0);
    if red_beside {
        let red = CardDataBuilder::new("Red Bear").card_type(CardType::Creature).color(Color::Red).power_toughness(2, 2).build();
        put_on_battlefield(game, red, 0);
    }
    resolve_all(game, &test_dp());
    game.activate_ability(0, ajani, 0, &test_dp()).expect("it is activated, with no target");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    ajani
}

/// The waiting triggers on the stack, each aimed at player 1, in the order
/// they triggered.
fn place_each_at_the_opponent(game: &mut GameState, triggers: usize) {
    // One ability's agreeing triggers are not ordered (CR 603.3b).
    let dp = ScriptedDecisionProvider::new();
    for _ in 0..triggers {
        dp.expect_choice(
            ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
            vec![ChoiceOption::Player(1)],
        );
    }
    game.perform_sba_and_triggers(&dp).unwrap();
    assert!(dp.is_empty(), "each target is chosen as its trigger goes on the stack");
}

// RULING: Ajani, Nacatl Avenger #7 - no target is chosen as the ability is activated; the reflexive ability's is chosen as it goes on the stack.
#[test]
fn ajanis_reflexive_trigger_chooses_its_target_as_it_goes_on_the_stack() {
    let mut game = audited_game();
    let ajani = ajani_activates(&mut game, true);
    assert_eq!(waiting_from(&game, ajani), 1);
    place_each_at_the_opponent(&mut game, 1);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 1), 20 - 2, "the Cat Warrior and the red creature");
}

// RULING: Ajani, Nacatl Avenger #8 - under Doubling Season the reflexive ability triggers once for each Cat Warrior.
#[test]
fn ajanis_reflexive_trigger_triggers_for_each_cat_warrior_doubling_season_makes() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, doubling_season(), 0);
    let ajani = ajani_activates(&mut game, true);
    assert_eq!(waiting_from(&game, ajani), 2);
    place_each_at_the_opponent(&mut game, 2);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 1), 20 - 3 - 3, "two Cat Warriors and the red creature, twice");
}

// RULING: Ajani, Nacatl Avenger #8 - tokens a replacement creates instead, as Divine Visitation's Angels, don't trigger it.
#[test]
fn ajanis_reflexive_trigger_ignores_the_angels_divine_visitation_makes_instead() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, divine_visitation(), 0);
    let ajani = ajani_activates(&mut game, true);
    let made_angels = game.recorded_events().events().any(|e| matches!(e, GameEvent::TokenCreated { definition, .. } if **definition != cat_warrior()));
    assert!(made_angels, "the creation made Angels instead");
    assert_eq!(waiting_from(&game, ajani), 0);
}

/// The intervening "if": with no red permanent but Ajani, nothing triggers.
#[test]
fn ajanis_reflexive_trigger_needs_a_red_permanent_other_than_ajani() {
    let mut game = audited_game();
    let ajani = ajani_activates(&mut game, false);
    assert_eq!(waiting_from(&game, ajani), 0);
}

/// "Whenever you create one or more tokens" triggers once for a creation of
/// three, and "whenever you create a token" three times; an opponent's
/// tokens are theirs.
#[test]
fn one_or_more_tokens_triggers_once_for_a_creation_of_several() {
    let mut game = audited_game();
    let once = whenever(creates_a_token(Whose::Yours, None).once_per_event(), gain_one());
    let once = put_on_battlefield(&mut game, watcher("Once", once), 0);
    let each = put_on_battlefield(&mut game, watcher("Each", whenever(creates_a_token(Whose::Yours, None), gain_one())), 0);
    resolve_all(&mut game, &test_dp());
    let three = Effect::Atom(Primitive::CreateToken(cat_warrior(), AmountExpr::Fixed(3)), EffectRecipient::Player(PlayerRef::You));
    game.resolve_effect(&three, &ResolutionContext::untargeted(once, 0, GeneratedBy::Spell), &test_dp()).unwrap();
    assert_eq!((waiting_from(&game, once), waiting_from(&game, each)), (1, 3));
    game.pending_triggers.clear();
    game.resolve_effect(&three, &ResolutionContext::untargeted(once, 1, GeneratedBy::Spell), &test_dp()).unwrap();
    assert_eq!((waiting_from(&game, once), waiting_from(&game, each)), (0, 0));
}

// ---------------------------------------------------------------------------
// Scry, and a count of targets the text defines (CR 701.22, 601.2c)
// ---------------------------------------------------------------------------

/// Elrond, Master of Healing, both abilities: "Whenever you scry, put a
/// +1/+1 counter on each of up to X target creatures, where X is the number
/// of cards looked at while scrying this way. Whenever a creature you
/// control with a +1/+1 counter on it becomes the target of a spell or
/// ability an opponent controls, you may draw a card."
fn elrond_master_of_healing() -> Arc<CardData> {
    let up_to_x = EffectRecipient::Target(SelectionFilter::Creature, TargetCount::UpToAmount(AmountExpr::TriggeringAmount));
    let counter = Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You };
    let yours_with_a_counter = ObjectFilter::And(
        Box::new(ObjectFilter::And(Box::new(a_creature()), Box::new(ObjectFilter::ByController(PlayerSet::You)))),
        Box::new(ObjectFilter::WithCounters { kind: Some(CounterType::PlusOnePlusOne), count: CountIs::AtLeast(1) }),
    );
    let draw = Effect::Optional {
        chooser: PlayerRef::You,
        effect: Box::new(Effect::Atom(Primitive::DrawCards(AmountExpr::Fixed(1)), EffectRecipient::Player(PlayerRef::You))),
    };
    let targeted = becomes_the_target_of(TriggerSubject::Filter(yours_with_a_counter), TargetedBy::SpellOrAbility, PlayerSet::Opponents);
    CardDataBuilder::new("Elrond, Master of Healing")
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .power_toughness(4, 4)
        .ability(triggered_ability("", whenever(scries(Whose::Yours), Effect::Atom(counter, up_to_x))))
        .ability(triggered_ability("", whenever(targeted, draw)))
        .build()
}

const PICK_SCRY: ChoiceKind = ChoiceKind::Scry { source: None, n: 0 };

/// `player` scries `n`, keeping every card on top in the order it was.
fn scry(game: &mut GameState, player: PlayerId, n: u64) {
    let dp = ScriptedDecisionProvider::new();
    let looked_at = game.players[player].library.len().min(n as usize);
    if looked_at > 0 {
        dp.expect_pick_n(PICK_SCRY, Vec::new());
    }
    if looked_at > 1 {
        dp.expect_ordering(ChoiceKind::ScryOrder { source: None, bottom: false }, (0..looked_at).collect());
    }
    game.execute_action(GameAction::Scry { player, n }, &ActionContext::new(&dp)).expect("the scry is performed");
}

/// Elrond's scry trigger on the stack, `targets` its chosen creatures.
fn place_choosing(game: &mut GameState, targets: Vec<ObjectId>) {
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
        targets.into_iter().map(ChoiceOption::Object).collect(),
    );
    game.perform_sba_and_triggers(&dp).unwrap();
}

fn plus_one_counters(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].counter_count(CounterType::PlusOnePlusOne)
}

/// A scry 3 from a library of two, with three creatures to target.
fn elrond_scries_three_from_two(game: &mut GameState) -> (ObjectId, Vec<ObjectId>) {
    let elrond = put_on_battlefield(game, elrond_master_of_healing(), 0);
    let bears: Vec<ObjectId> = (0..3).map(|_| put_on_battlefield(game, vanilla_creature(2, 2, &[]), 0)).collect();
    fill_library(game, 0, 2);
    scry(game, 0, 3);
    (elrond, bears)
}

// RULING: Elrond, Master of Healing #1 - X is the number of cards actually looked at: a scry 3 with two cards in the library makes X 2.
#[test]
fn elronds_x_is_the_cards_looked_at() {
    let mut game = audited_game();
    let (elrond, bears) = elrond_scries_three_from_two(&mut game);
    assert_eq!(waiting_from(&game, elrond), 1);
    place_choosing(&mut game, bears[..2].to_vec());
    resolve_all(&mut game, &test_dp());
    let counters: Vec<u32> = bears.iter().map(|&bear| plus_one_counters(&game, bear)).collect();
    assert_eq!(counters, vec![1, 1, 0]);
}

/// The count bounds the announcement (CR 601.2c): up to 2, so a third
/// target is never offered as a legal answer.
#[test]
#[should_panic(expected = "expected 0-2")]
fn elrond_cannot_target_more_creatures_than_x() {
    let mut game = audited_game();
    let (_, bears) = elrond_scries_three_from_two(&mut game);
    place_choosing(&mut game, bears);
}

/// "Up to": fewer than X, none included.
#[test]
fn elrond_may_choose_fewer_targets_than_x() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, elrond_master_of_healing(), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    fill_library(&mut game, 0, 3);
    scry(&mut game, 0, 3);
    place_choosing(&mut game, vec![bear]);
    resolve_all(&mut game, &test_dp());
    assert_eq!(plus_one_counters(&game, bear), 1);
}

/// A scry from an empty library is still a scry (CR 701.22d), with nothing
/// looked at: X is 0, so the trigger has no target to choose and asks for
/// none.
#[test]
fn a_scry_that_looks_at_nothing_makes_x_zero() {
    let mut game = audited_game();
    let elrond = put_on_battlefield(&mut game, elrond_master_of_healing(), 0);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert!(game.players[0].library.is_empty());
    scry(&mut game, 0, 2);
    assert_eq!(waiting_from(&game, elrond), 1);
    resolve_all(&mut game, &test_dp());
    assert_eq!(plus_one_counters(&game, elrond), 0);
}

// COVERS: ATOM-701.22b-001
/// CR 701.22b: a scry 0 is no scry, so "whenever you scry" does not trigger.
#[test]
fn a_scry_zero_triggers_nothing() {
    let mut game = audited_game();
    let elrond = put_on_battlefield(&mut game, elrond_master_of_healing(), 0);
    fill_library(&mut game, 0, 3);
    let effect = Effect::Atom(Primitive::Scry(AmountExpr::Fixed(0)), EffectRecipient::Player(PlayerRef::You));
    game.resolve_effect(&effect, &ResolutionContext::untargeted(elrond, 0, GeneratedBy::Spell), &test_dp()).unwrap();
    assert_eq!(waiting_from(&game, elrond), 0);
}

/// "Whenever you scry": an opponent's scry is theirs.
#[test]
fn an_opponents_scry_does_not_trigger_elrond() {
    let mut game = audited_game();
    let elrond = put_on_battlefield(&mut game, elrond_master_of_healing(), 0);
    fill_library(&mut game, 1, 3);
    scry(&mut game, 1, 1);
    assert_eq!(waiting_from(&game, elrond), 0);
}

// RULING: Elrond, Master of Healing #2 - two creatures you control with counters targeted by one opponent's spell or ability trigger the last ability twice.
#[test]
fn two_of_elronds_creatures_targeted_by_one_ability_trigger_it_twice() {
    let mut game = audited_game();
    let elrond = put_on_battlefield(&mut game, elrond_master_of_healing(), 0);
    let bears: Vec<ObjectId> = (0..2).map(|_| put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0)).collect();
    let plain = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    for &bear in &bears {
        let timestamp = game.allocate_timestamp();
        game.battlefield.get_mut(&bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1, timestamp);
    }
    game.bump_layer_epoch();
    let tap_two = AbilityDef {
        rules_text: "".into(),
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Activated,
        costs: Vec::new(),
        effect: Effect::Atom(Primitive::Tap, EffectRecipient::Target(SelectionFilter::Creature, TargetCount::UpTo(3))),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    };
    let wand = put_on_battlefield(&mut game, CardDataBuilder::new("Wand").card_type(CardType::Artifact).ability(tap_two).build(), 1);
    resolve_all(&mut game, &test_dp());
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
        vec![ChoiceOption::Object(bears[0]), ChoiceOption::Object(bears[1]), ChoiceOption::Object(plain)],
    );
    game.activate_ability(1, wand, 0, &dp).expect("the opponent's ability targets three creatures");
    assert_eq!(waiting_from(&game, elrond), 2, "the two with a counter, each once");
}

/// "The number of other creatures you control", counted as an ability
/// resolves: "other" is other than the permanent whose ability it is, not
/// the ability on the stack (CR 113.7a, 603.6a's "another").
#[test]
fn a_count_of_other_creatures_leaves_out_the_abilitys_own_permanent() {
    let mut game = audited_game();
    let others = AmountExpr::CountOf(ObjectSet::battlefield_filter(another(ObjectFilter::And(
        Box::new(a_creature()),
        Box::new(ObjectFilter::ByController(PlayerSet::You)),
    ))));
    let ping = AbilityDef {
        rules_text: "".into(),
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Activated,
        costs: Vec::new(),
        effect: Effect::Atom(Primitive::LoseLife(others), EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1))),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    };
    let pinger = put_on_battlefield(&mut game, creature_with_ability("Counter", 1, 1, ping), 0);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    resolve_all(&mut game, &test_dp());
    let dp = ScriptedDecisionProvider::new();
    dp.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
        vec![ChoiceOption::Player(1)],
    );
    game.activate_ability(0, pinger, 0, &dp).expect("it is activated");
    game.resolve_top_of_stack(&test_dp()).expect("it resolves");
    assert_eq!(life(&game, 1), 20 - 2, "the two others, not the counter itself");
}

/// A multiplier a granter gave a creature, the granter dying in the event:
/// a leaving is read on the board from before it (CR 603.10), where the
/// creature still had the ability, so the granter's own death is doubled.
/// Only a list from before the batch holds it, which the count asks of a
/// survivor no source set reaches now.
#[test]
fn a_granted_multiplier_counts_on_the_board_before_its_granter_left() {
    let mut game = audited_game();
    let carrier = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let granter = put_on_battlefield(&mut game, legendary_creature("Granter"), 0);
    let mourner = put_on_battlefield(&mut game, watcher("Mourner", whenever(dies(a_creature()), gain_one())), 0);
    resolve_all(&mut game, &test_dp());
    let mut gandalfs = gandalf_the_white().abilities[0].clone();
    gandalfs.id = new_ability_id();
    let ts = game.allocate_timestamp();
    game.continuous_effects.add(ContinuousEffect {
        duration: Duration::WhileSourceOnBattlefield,
        affected_objects: ObjectSet::Fixed(vec![carrier]),
        ..registered(granter, Layer::Layer6Ability, ts, EffectModification::GrantAbility(Arc::new(gandalfs)))
    });
    game.execute_action(GameAction::Destroy { object: granter }, &ActionContext::new(&test_dp())).unwrap();
    assert!(!game.continuous_effects.summary().any_granted_trigger_multiplier, "the grant ended with its granter");
    assert_eq!(waiting_from(&game, mourner), 2);
}

// ---------------------------------------------------------------------------
// The game's end (CR 104.1)
// ---------------------------------------------------------------------------

/// A trigger that is waiting when the game ends is never put on the stack,
/// since no player receives priority again (CR 104.1, 117.5), so the
/// finished game holds none: here the damage that triggers it is the damage
/// whose state-based check ends the game.
#[test]
fn a_game_that_ends_drops_the_triggers_still_waiting() {
    let mut game = audited_game();
    let damage_to_a_player = TriggerEvent::DamageDealt {
        source: TriggerSubject::Filter(a_creature()),
        recipient: ParticipantFilter::Player(PlayerSet::Everyone),
        combat: None,
        excess: false,
        multiplicity: Multiplicity::PerOccurrence,
    };
    let grid = put_on_battlefield(&mut game, watcher("Grid", whenever(damage_to_a_player, gain_one())), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    resolve_all(&mut game, &test_dp());
    game.players[1].life_total = 2;
    let hit = GameAction::DealDamage { source: bear, target: ObjectOrPlayerId::Player(1), amount: 2, is_combat: false, unpreventable: false, source_lki: None };
    game.execute_action(hit, &ActionContext::new(&test_dp())).unwrap();
    assert_eq!(waiting_from(&game, grid), 1, "the damage triggered it");

    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(game.result, Some(GameResult::Winner(0)));
    assert!(game.pending_triggers.is_empty() && game.stack.is_empty(), "nothing waits in a finished game");
}

// ---------------------------------------------------------------------------
// Teysa Karlov: a death, and the events that cause one
// ---------------------------------------------------------------------------

/// Teysa Karlov's first ability, on a 2/4 legendary creature: "If a creature
/// dying causes a triggered ability of a permanent you control to trigger,
/// that ability triggers an additional time." Her second, creature tokens
/// you control having vigilance and lifelink, has no part in the count.
fn teysa_karlov() -> Arc<CardData> {
    let multiplier = TriggerMultiplierDef {
        caused_by: Some(TriggerCondition::Event(dies(a_creature()).into())),
        abilities_of: ObjectFilter::ByController(PlayerSet::You),
        additional: 1,
    };
    CardDataBuilder::new("Teysa Karlov")
        .card_type(CardType::Creature)
        .supertype(Supertype::Legendary)
        .power_toughness(2, 4)
        .ability(multiplier.into_ability(""))
        .build()
}

fn destroy(game: &mut GameState, objects: &[ObjectId]) {
    let destroys = objects.iter().map(|&object| GameAction::Destroy { object }).collect();
    game.execute_actions(destroys, &ActionContext::new(&test_dp())).unwrap();
}

/// "Whenever [subject] is put into a graveyard from the battlefield"
/// without the word "dies": the arm is the same.
fn put_into_a_graveyard_from_the_battlefield(subject: ObjectFilter) -> TriggerDef {
    whenever(dies(TriggerSubject::Filter(subject)), gain_one())
}

// RULING: Teysa Karlov #1 - a creature's own dies trigger is doubled, as are other triggers on that creature dying.
#[test]
fn teysa_doubles_a_creatures_own_dies_trigger_and_one_watching_it() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, teysa_karlov(), 0);
    let own = put_on_battlefield(&mut game, watcher("Martyr", whenever(dies(TriggerSubject::Object(Antecedent::ThisObject)), gain_one())), 0);
    let mourner = put_on_battlefield(&mut game, watcher("Mourner", whenever(dies(another(a_creature())), gain_one())), 0);
    resolve_all(&mut game, &test_dp());
    destroy(&mut game, &[own]);
    assert_eq!((waiting_from(&game, own), waiting_from(&game, mourner)), (2, 2));
}

// RULING: Teysa Karlov #3 - an artifact put into a graveyard from the battlefield triggers twice if it was a creature.
// RULING: Teysa Karlov #6 - each permanent is looked at as it was on the battlefield: an animated artifact dying is a creature dying.
#[test]
fn teysa_doubles_an_artifact_trigger_when_the_artifact_was_a_creature() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, teysa_karlov(), 0);
    let scrap = put_on_battlefield(&mut game, watcher("Scrapper", put_into_a_graveyard_from_the_battlefield(ObjectFilter::ByType(CardType::Artifact))), 0);
    let golem = put_on_battlefield(&mut game, artifact_creature("Golem"), 0);
    let ring = put_on_battlefield(&mut game, sol_ring(), 0);
    resolve_all(&mut game, &test_dp());
    destroy(&mut game, &[ring]);
    assert_eq!(waiting_from(&game, scrap), 1, "a noncreature artifact does not die");
    game.pending_triggers.clear();

    put_on_battlefield(&mut game, march_of_the_machines(), 1);
    let ring = put_on_battlefield(&mut game, sol_ring(), 0);
    resolve_all(&mut game, &test_dp());
    destroy(&mut game, &[ring, golem]);
    assert_eq!(waiting_from(&game, scrap), 4, "a Sol Ring March of the Machines animated, and a Golem: each was a creature");
}

// RULING: Teysa Karlov #4 - a creature leaving the battlefield by dying triggers a leaves-the-battlefield ability twice.
#[test]
fn teysa_doubles_a_leaving_by_dying_and_not_by_exile() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, teysa_karlov(), 0);
    let leaving = put_on_battlefield(&mut game, watcher("Watcher", whenever(leaves_the_battlefield(another(a_creature())), gain_one())), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let other = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    resolve_all(&mut game, &test_dp());
    destroy(&mut game, &[bear]);
    assert_eq!(waiting_from(&game, leaving), 2, "it died");
    game.pending_triggers.clear();
    game.change_zone(other, Zone::Exile, ZoneChangeAct::Put, &ActionContext::new(&test_dp())).unwrap();
    assert_eq!(waiting_from(&game, leaving), 1, "it was exiled, which is no death");
}

// RULING: Teysa Karlov #5 - an ability that triggers on an event that causes a creature to die, "whenever you sacrifice a creature", triggers only once.
#[test]
fn teysa_leaves_a_sacrifice_trigger_alone() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, teysa_karlov(), 0);
    let altar = put_on_battlefield(&mut game, watcher("Altar Keeper", whenever(is_sacrificed(another(a_creature())), gain_one())), 0);
    let mourner = put_on_battlefield(&mut game, watcher("Mourner", whenever(dies(another(a_creature())), gain_one())), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    resolve_all(&mut game, &test_dp());
    game.change_zone(bear, Zone::Graveyard, ZoneChangeAct::Sacrificed, &ActionContext::new(&test_dp())).unwrap();
    assert_eq!(waiting_from(&game, altar), 1, "the sacrifice is its own event");
    assert_eq!(waiting_from(&game, mourner), 2, "the death it caused is doubled");
}

// RULING: Teysa Karlov #7 - a creature dying at the same time another permanent you control leaves doubles that permanent's trigger.
#[test]
fn teysa_doubles_a_trigger_of_a_permanent_leaving_beside_the_creature() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, teysa_karlov(), 0);
    let pact = CardDataBuilder::new("Pact").card_type(CardType::Enchantment).ability(triggered_ability("", whenever(dies(a_creature()), gain_one()))).build();
    let pact = put_on_battlefield(&mut game, pact, 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    resolve_all(&mut game, &test_dp());
    destroy(&mut game, &[pact, bear]);
    assert_eq!(waiting_from(&game, pact), 2, "read off the enchantment's frame as it left");
}

// RULING: Teysa Karlov #8 - a creature dying at the same time as Teysa, Teysa included, is doubled.
#[test]
fn teysa_dying_beside_a_creature_doubles_both_deaths() {
    let mut game = audited_game();
    let teysa = put_on_battlefield(&mut game, teysa_karlov(), 0);
    let mourner = put_on_battlefield(&mut game, watcher("Mourner", whenever(dies(another(a_creature())), gain_one())), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    resolve_all(&mut game, &test_dp());
    destroy(&mut game, &[teysa, bear]);
    assert_eq!(waiting_from(&game, mourner), 4);
}

// RULING: Teysa Karlov #9 - two Teysas make three, not four, so a Teysa dying to the legend rule beside another triggers three times.
#[test]
fn two_teysas_make_three_as_the_legend_rule_kills_one() {
    let mut game = audited_game();
    let mourner = put_on_battlefield(&mut game, watcher("Mourner", whenever(dies(a_creature()), gain_one())), 0);
    put_on_battlefield(&mut game, teysa_karlov(), 0);
    resolve_all(&mut game, &test_dp());
    put_on_battlefield(&mut game, teysa_karlov(), 0);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::LegendRule { legend_name: "Teysa Karlov".into() }, vec![0]);
    game.perform_sba_and_triggers(&dp).unwrap();
    assert!(dp.is_empty(), "the legend rule chose");
    let stacked = game.stack.iter().filter(|id| game.stack_entries[*id].ability_identity.is_some_and(|i| i.source.id == mourner)).count();
    assert_eq!(stacked, 3, "the dying Teysa is read as it was, beside the other");
}

// ---------------------------------------------------------------------------
// "X target creatures" (CR 601.2b-c)
// ---------------------------------------------------------------------------

/// {X}{U} sorcery: "Tap X target creatures."
fn tap_x() -> Arc<CardData> {
    let spell = AbilityDef {
        rules_text: "".into(),
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Spell,
        costs: Vec::new(),
        effect: Effect::Atom(Primitive::Tap, EffectRecipient::Target(SelectionFilter::Creature, TargetCount::ExactlyAmount(AmountExpr::X))),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
    };
    CardDataBuilder::new("Tap X")
        .mana_cost(ManaCost::from_symbols(vec![ManaSymbol::X, ManaSymbol::Colored(ManaType::Blue)]))
        .card_type(CardType::Sorcery)
        .ability(spell)
        .build()
}

/// X is announced before the targets (CR 601.2b), and then exactly X are
/// chosen: X = 2 taps two of three, and X = 0 asks for none.
#[test]
fn x_target_creatures_takes_exactly_the_announced_x() {
    let mut game = audited_game();
    let bears: Vec<ObjectId> = (0..3).map(|_| put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1)).collect();
    resolve_all(&mut game, &test_dp());
    game.players[0].mana_pool.add(ManaType::Blue, 3);
    let card = put_in_hand(&mut game, tap_x(), 0);
    let seat = ScriptedDecisionProvider::new();
    seat.expect_number(ChoiceKind::ChooseXValue { spell_id: card, values: Box::new(LegalXValues::zero_through(2)) }, 2);
    seat.expect_choice(
        ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: card },
        vec![ChoiceOption::Object(bears[0]), ChoiceOption::Object(bears[1])],
    );
    game.cast_spell(0, card, &seat).expect("castable");
    assert!(seat.is_empty());
    resolve_all(&mut game, &test_dp());
    let tapped: Vec<bool> = bears.iter().map(|b| game.battlefield[b].tapped).collect();
    assert_eq!(tapped, vec![true, true, false]);

    // One blue mana pays X = 0 only, which is no choice; a scripted seat with
    // nothing queued fails on any prompt, so none is asked.
    game.players[0].mana_pool.add(ManaType::Blue, 1);
    let card = put_in_hand(&mut game, tap_x(), 0);
    game.cast_spell(0, card, &ScriptedDecisionProvider::new()).expect("castable with no target");
    assert_eq!(game.stack_entries[&card].x_value, Some(0));
}
