//! Phase TR-5g — the multiplier, tokens and scry (CR 603.2d, 701.7, 701.22).
//!
//! 1. CR 603.2d's count: a multiplier on the board the matched arm reads
//!    adds its `additional` to a trigger of a permanent it names whose
//!    triggering event is the event it names. Panharmonicon, and Gandalf the
//!    White's fixture for a leaving read off its frame. Every board here runs
//!    under the dispatch audit, which counts again with no shortcut.

use std::sync::Arc;

use mtgsim::cards::authoring::{another, creates_a_token, dies, enters, is_sacrificed, triggered_ability, when_you_do, whenever, Whose};
use mtgsim::cards::basic_lands::forest;
use mtgsim::cards::phase_cv_cards::cytoshape;
use mtgsim::cards::phase_rc_cards::chainbreaker;
use mtgsim::cards::phase_re_cards::{divine_visitation, doubling_season};
use mtgsim::cards::phase_tr1_cards::{soul_warden, wild_growth};
use mtgsim::cards::phase_tr5g_cards::panharmonicon;
use mtgsim::engine::actions::{ActionContext, GameAction, ZoneChangeAct};
use mtgsim::engine::layers::types::{EffectModification, Layer};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    creature_with_ability, put_in_graveyard, put_on_battlefield, registered, setup_two_player_game, static_ability, test_dp, vanilla_creature,
};
use mtgsim::events::event::GameEvent;
use mtgsim::types::card_types::{CardType, CreatureType, Subtype, Supertype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, Condition, CounterType, Effect, EffectRecipient, ObjectFilter, PlayerFact, PlayerRef, PlayerSet, Primitive, SelectionFilter,
    Selector, TargetCount, TokenDef,
};
use mtgsim::types::ids::{AbilityId, ObjectId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::triggers::{
    DelayedDuration, DelayedTriggerTemplate, DelayedTurn, Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerLimit,
    TriggerMultiplierDef, TriggerSubject, TriggerTier,
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
    game.enable_dispatch_audit();
    game
}

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
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
    watcher("Healer", whenever(enters(TriggerSubject::ThisObject), gain_one()))
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
/// cast as though with flash are PM's.
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
                controller: None,
                from: None,
                was_cast: None,
                multiplicity: Multiplicity::PerOccurrence,
            },
            TriggerEvent::ZoneChange {
                subject: legendary_or_artifact(),
                change: ZoneChangePattern { from: Some(Zone::Battlefield), to: None, act: None },
                by: None,
                owner: None,
                multiplicity: Multiplicity::PerOccurrence,
            },
        ])),
        abilities_of: ObjectFilter::ByController(PlayerRef::You),
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
    let multiplier = TriggerMultiplierDef { caused_by: None, abilities_of: ObjectFilter::ByController(PlayerRef::You), additional: 1 };
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
    let seqs: Vec<_> = game.pending_triggers.iter().map(|p| p.seq).collect();
    assert_ne!(seqs[0], seqs[1], "each time is its own trigger");
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
#[test]
fn each_doubled_instance_makes_its_own_choice() {
    let mut game = audited_game();
    put_on_battlefield(&mut game, panharmonicon(), 0);
    let may_gain = Effect::Optional { chooser: PlayerRef::You, effect: Box::new(gain_one()) };
    let before = life(&game, 0);
    let healer = reanimate(&mut game, watcher("Hopeful", whenever(enters(TriggerSubject::ThisObject), may_gain)), 0);
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

// RULING: Panharmonicon #7 - a creature entering at the same time as Panharmonicon, Panharmonicon included, is doubled.
#[test]
fn a_creature_entering_beside_panharmonicon_is_doubled() {
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
            EffectRecipient::ThisObject,
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
    let create = Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(template)), EffectRecipient::Controller);
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
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(copyist)]),
        replaced_amount: None,
        damage_prevented: None,
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
        owner: None,
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

/// Ajani, Nacatl Avenger's 0 ability, which a loyalty ability is once CP-2
/// builds them: "Create a 2/1 white Cat Warrior creature token. When you
/// do, if you control a red permanent other than Ajani, he deals damage
/// equal to the number of creatures you control to any target."
fn ajanis_zero() -> Effect {
    let damage = Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::CountOf(Selector::ControlledCreatures), unpreventable: false },
        EffectRecipient::Target(SelectionFilter::Any, TargetCount::Exactly(1)),
    );
    let mut when_you_do = when_you_do(creates_a_token(Whose::Yours, Some(cat_warrior())), damage, "");
    let red_other = ObjectFilter::And(Box::new(ObjectFilter::ByColor(Color::Red)), Box::new(ObjectFilter::NotSource));
    Arc::make_mut(&mut when_you_do.def).intervening_if =
        Some(Condition::Player { players: PlayerSet::You, fact: PlayerFact::ControlsPermanent(red_other) });
    Effect::Sequence(vec![
        Effect::Atom(Primitive::CreateToken(cat_warrior(), AmountExpr::Fixed(1)), EffectRecipient::Controller),
        Effect::Atom(Primitive::CreateDelayedTrigger(Box::new(when_you_do)), EffectRecipient::Controller),
    ])
}

/// Ajani, with a red creature beside him or not, his 0 ability activated
/// and resolved: an artifact's "{0}: ..." until CP-2 builds loyalty.
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
    let dp = ScriptedDecisionProvider::new();
    if triggers > 1 {
        dp.expect_ordering(ChoiceKind::OrderTriggers { player: 0, tier: TriggerTier::First }, (0..triggers).collect());
    }
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
    let three = Effect::Atom(Primitive::CreateToken(cat_warrior(), AmountExpr::Fixed(3)), EffectRecipient::Controller);
    game.resolve_effect(&three, &ResolutionContext::untargeted(once, 0, GeneratedBy::Spell), &test_dp()).unwrap();
    assert_eq!((waiting_from(&game, once), waiting_from(&game, each)), (1, 3));
    game.pending_triggers.clear();
    game.resolve_effect(&three, &ResolutionContext::untargeted(once, 1, GeneratedBy::Spell), &test_dp()).unwrap();
    assert_eq!((waiting_from(&game, once), waiting_from(&game, each)), (0, 0));
}
