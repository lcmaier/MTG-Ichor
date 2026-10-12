//! RH-1: the act and the cause (`replacement-architecture.md` §2b.2, §2b.3).
//!
//! Every record names who or what caused it, and the three shapes a cause
//! clause prints in ask that one value: a "can't" (Sigarda, Host of Herons), a
//! replacement effect (Nephalia Academy) and a trigger (Guerrilla Tactics), each
//! asking "a spell or ability an opponent controls". One board per kind of
//! cause, each run under all three: only the opponent's spell is caught.

use std::sync::Arc;

use mtgsim::cards::authoring::{triggered_ability, whenever};
use mtgsim::cards::phase_rb_cards::rest_in_peace;
use mtgsim::cards::phase_re8_cards::mind_rot;
use mtgsim::cards::phase_rs_cards::diabolic_edict;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::{EventCause, GameRule, GeneratedBy, SpecialAction, StateBasedAction, TurnBasedAction};
use mtgsim::events::event::{EventRecord, EventStamp, GameEvent};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::state::game::Game;
use mtgsim::state::game_config::GameConfig;
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    forest, put_in_graveyard, put_in_hand, put_on_battlefield, setup_two_player_game, test_dp,
    vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::costs::Cost;
use mtgsim::types::effects::{AmountExpr, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerRef, PlayerSet, Primitive};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::replacement::{EventPattern, GameActionTemplate, ReplacementDef, Rewrite};
use mtgsim::types::restriction::{Restriction, RestrictionDef, SourceFilter};
use mtgsim::types::triggers::{Multiplicity, TriggerEvent, TriggerSubject};
use mtgsim::types::zones::{Zone, ZoneChangeAct, ZoneChangePattern, ZoneSet};
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// The three shapes of a cause clause
// ---------------------------------------------------------------------------

/// Which shape a watcher prints its cause clause in.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Shape {
    /// Sigarda's: "spells and abilities your opponents control can't cause".
    Cant,
    /// Nephalia Academy's: "if a spell or ability an opponent controls causes
    /// … instead", here exiling the card.
    Replacement,
    /// Guerrilla Tactics': "when a spell or ability an opponent controls
    /// causes …", here gaining 1 life.
    Trigger,
}

fn gain_one() -> Effect {
    Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(1)), EffectRecipient::Controller)
}

fn ability(ability_type: AbilityType, costs: Vec<Cost>, effect: Effect) -> AbilityDef {
    AbilityDef {
        rules_text: "".into(),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
        id: new_ability_id(),
        instances: Vec::new(),
        ability_type,
        costs,
        effect,
    }
}

/// **Fixture.** One of three test enchantments, picked by `shape`, each with
/// a single ability that asks of `change` to its controller's cards whether a
/// spell or ability an opponent controls caused it: a "can't" (Sigarda's
/// shape), a replacement (Nephalia Academy's) or a trigger (Guerrilla
/// Tactics'). The `match` is how one board runs under all three readers of
/// the cause; a printed card has one shape and writes it directly.
fn watcher(shape: Shape, change: ZoneChangePattern) -> Arc<CardData> {
    let by = SourceFilter::ControlledBy(PlayerRef::Opponent);
    let yours = ObjectSet::filter_in(ObjectFilter::ByOwner(PlayerRef::You), ZoneSet::ALL);
    let pattern = EventPattern::ZoneChange { change, object: None };
    let ability = match shape {
        Shape::Cant => ability(
            AbilityType::Static,
            Vec::new(),
            Effect::Restriction(Box::new(RestrictionDef::new(Restriction::Event {
                pattern,
                affected_objects: yours,
                affected_players: PlayerSet::Nobody,
                by: Some(by),
            }))),
        ),
        Shape::Replacement => ability(
            AbilityType::Static,
            Vec::new(),
            Effect::Replacement(Box::new(
                ReplacementDef::new(pattern, yours, Rewrite::Instead(GameActionTemplate::ZoneChangeTo { to: Zone::Exile }))
                    .caused_by(by),
            )),
        ),
        Shape::Trigger => triggered_ability(
            "",
            whenever(
                TriggerEvent::ZoneChange {
                    subject: TriggerSubject::Any,
                    change,
                    by: Some(by),
                    owner: Some(PlayerRef::You),
                    multiplicity: Multiplicity::PerOccurrence,
                },
                gain_one(),
            ),
        ),
    };
    CardDataBuilder::new("Fixture Cause Watcher").card_type(CardType::Enchantment).ability(ability).build()
}

/// What a board's move came to: where the moved object is, how many
/// abilities triggered, and the cause its last move's record named.
#[derive(Debug)]
struct Outcome {
    zone: Zone,
    triggered: usize,
    cause: Option<EventCause>,
}

fn outcome(game: &GameState, moved: ObjectId) -> Outcome {
    let recorded = game.recorded_events();
    let records = recorded.records();
    Outcome {
        zone: game.get_object(moved).expect("still in the game").zone,
        triggered: records.iter().filter(|r| matches!(r.event, GameEvent::AbilityTriggered { .. })).count(),
        cause: records
            .iter()
            .rev()
            .find(|r| matches!(r.event, GameEvent::ZoneChange { object_id, .. } if object_id == moved))
            .map(|r| r.stamp.cause),
    }
}

/// Resolve `card`'s spell ability for `caster` against `target`, its source
/// out of every zone a board counts.
fn resolve_at_player(game: &mut GameState, card: Arc<CardData>, caster: PlayerId, target: PlayerId) {
    let source = put_in_graveyard(game, card.clone(), caster);
    let mut ctx = ResolutionContext::untargeted(source, caster, GeneratedBy::Spell);
    ctx.targets = ChosenTargets::one(vec![ResolvedTarget::Player(target)]);
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).expect("resolving");
}

const DISCARD: ZoneChangePattern = ZoneChangePattern { from: Some(Zone::Hand), to: None, act: Some(ZoneChangeAct::Discarded) };

// ---------------------------------------------------------------------------
// One board per kind of cause
// ---------------------------------------------------------------------------

/// An opponent's spell: P1's Mind Rot makes P0 discard.
fn an_opponents_spell(shape: Option<Shape>) -> Outcome {
    let mut game = setup_two_player_game();
    if let Some(shape) = shape {
        put_on_battlefield(&mut game, watcher(shape, DISCARD), 0);
    }
    let card = put_in_hand(&mut game, vanilla_creature(2, 2, &[]), 0);
    resolve_at_player(&mut game, mind_rot(), 1, 0);
    outcome(&game, card)
}

/// **Fixture.** "Sacrifice this creature: You gain 1 life."
fn sacrifices_itself_for_life() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Self-Sacrificer")
        .card_type(CardType::Creature)
        .power_toughness(1, 1)
        .ability(ability(AbilityType::Activated, vec![Cost::SacrificeSelf], gain_one()))
        .build()
}

/// A cost: P0 sacrifices a creature to pay for its own ability. A discard
/// cost has no payment yet (`Cost::Discard`, `codebase-state.md`), so this is
/// a sacrifice, which is Sigarda's ruling's case: "you can still sacrifice
/// permanents to pay costs".
fn a_cost(shape: Option<Shape>) -> Outcome {
    let mut game = setup_two_player_game();
    let change = ZoneChangePattern { from: Some(Zone::Battlefield), to: None, act: Some(ZoneChangeAct::Sacrificed) };
    if let Some(shape) = shape {
        put_on_battlefield(&mut game, watcher(shape, change), 0);
    }
    let source = put_on_battlefield(&mut game, sacrifices_itself_for_life(), 0);
    game.activate_ability(0, source, 0, &test_dp()).expect("the cost is paid");
    outcome(&game, source)
}

/// The cleanup discard (CR 514.1): a whole turn whose only decision is it.
fn the_cleanup_discard(shape: Option<Shape>) -> Outcome {
    let blank = |i: usize| CardDataBuilder::new(&format!("Deck {i}")).build();
    let deck: Vec<Arc<CardData>> = (0..30).map(blank).collect();
    let mut game = Game::new(GameConfig::test(), vec![deck.clone(), deck]).expect("game creation");
    game.state.record_events();
    let dp = ScriptedDecisionProvider::new();
    game.setup(&dp).expect("setup");
    let active = game.state.active_player;
    if let Some(shape) = shape {
        put_on_battlefield(&mut game.state, watcher(shape, DISCARD), active);
    }
    while game.state.players[active].hand.len() < 7 {
        put_in_hand(&mut game.state, CardDataBuilder::new("Extra").build(), active);
    }
    // Eight after the draw step: the first card goes.
    dp.expect_pick_n(ChoiceKind::Discard { source: None }, vec![0]);
    game.run_turn(&dp).expect("turn");
    let discarded = game
        .state
        .recorded_events()
        .records()
        .iter()
        .find_map(|r| match r.event {
            GameEvent::ZoneChange { object_id, act: ZoneChangeAct::Discarded, .. } => Some(object_id),
            _ => None,
        })
        .expect("a card was discarded");
    outcome(&game.state, discarded)
}

/// A special action: P0 plays a land (CR 116.2a).
fn a_land_play(shape: Option<Shape>) -> Outcome {
    let mut game = setup_two_player_game();
    let change = ZoneChangePattern { from: Some(Zone::Hand), to: Some(Zone::Battlefield), act: None };
    if let Some(shape) = shape {
        put_on_battlefield(&mut game, watcher(shape, change), 0);
    }
    let land = put_in_hand(&mut game, forest(), 0);
    game.play_land(0, land, &test_dp()).expect("the land drop is legal");
    outcome(&game, land)
}

/// A state-based action: P0's creature has 0 toughness (CR 704.5f).
fn zero_toughness(shape: Option<Shape>) -> Outcome {
    let mut game = setup_two_player_game();
    let change = ZoneChangePattern { from: Some(Zone::Battlefield), to: None, act: None };
    if let Some(shape) = shape {
        put_on_battlefield(&mut game, watcher(shape, change), 0);
    }
    let creature = put_on_battlefield(&mut game, vanilla_creature(0, 0, &[]), 0);
    game.check_state_based_actions(&test_dp()).expect("the check");
    outcome(&game, creature)
}

/// **Fixture.** A free instant, "You gain 1 life."
fn free_instant() -> Arc<CardData> {
    CardDataBuilder::new("Fixture Free Instant")
        .card_type(CardType::Instant)
        .ability(ability(AbilityType::Spell, Vec::new(), gain_one()))
        .build()
}

/// A rule: P0's instant goes to the graveyard as the last step of its
/// resolution (CR 608.2n).
fn an_instants_trip(shape: Option<Shape>) -> Outcome {
    let mut game = setup_two_player_game();
    let change = ZoneChangePattern { from: Some(Zone::Stack), to: None, act: None };
    if let Some(shape) = shape {
        put_on_battlefield(&mut game, watcher(shape, change), 0);
    }
    let instant = put_in_hand(&mut game, free_instant(), 0);
    game.cast_spell(0, instant, &test_dp()).expect("castable");
    game.resolve_top_of_stack(&test_dp()).expect("resolves");
    outcome(&game, instant)
}

/// Each board's move with no watcher: where it goes, and the cause its record
/// names. The opponent's spell is the only cause a "spell or ability an
/// opponent controls" admits.
#[test]
fn each_kind_of_cause_is_named_on_its_record() {
    let p1s_spell = |o: &Outcome| matches!(o.cause, Some(EventCause::Effect { resolution, generated_by: GeneratedBy::Spell }) if resolution.controller == 1);
    let spell = an_opponents_spell(None);
    assert_eq!(spell.zone, Zone::Graveyard);
    assert!(p1s_spell(&spell), "{:?}", spell.cause);
    assert!(matches!(a_cost(None).cause, Some(EventCause::Cost { player: 0, .. })));
    assert_eq!(the_cleanup_discard(None).cause, Some(EventCause::TurnBasedAction(TurnBasedAction::CleanupDiscard)));
    assert_eq!(
        a_land_play(None).cause,
        Some(EventCause::SpecialAction { player: 0, action: SpecialAction::PlayLand })
    );
    assert_eq!(zero_toughness(None).cause, Some(EventCause::StateBasedAction(StateBasedAction::ZeroToughness)));
    assert_eq!(an_instants_trip(None).cause, Some(EventCause::GameRule(GameRule::SpellResolves)));
}

/// The opponent's spell is caught by all three shapes: the "can't" keeps the
/// card in hand, the replacement exiles it, and the trigger triggers.
#[test]
fn an_opponents_spell_is_caught_by_all_three_shapes() {
    assert_eq!(an_opponents_spell(Some(Shape::Cant)).zone, Zone::Hand, "it could not be discarded");
    assert_eq!(an_opponents_spell(Some(Shape::Replacement)).zone, Zone::Exile);
    let triggered = an_opponents_spell(Some(Shape::Trigger));
    assert_eq!((triggered.zone, triggered.triggered), (Zone::Graveyard, 1));
}

/// Every other kind of cause is caught by none: costs "aren't effects"
/// (Library of Leng's ruling), a turn-based action, a special action, a
/// state-based action and a rule's move are "the result of game rules"
/// (Sigarda's), and each of them has a watcher shaped to its own move.
#[test]
fn no_other_cause_is_a_spell_or_ability_an_opponent_controls() {
    let boards: [(&str, fn(Option<Shape>) -> Outcome, Zone); 5] = [
        ("a cost", a_cost, Zone::Graveyard),
        ("the cleanup discard", the_cleanup_discard, Zone::Graveyard),
        ("a land play", a_land_play, Zone::Battlefield),
        ("0 toughness", zero_toughness, Zone::Graveyard),
        ("an instant's trip", an_instants_trip, Zone::Graveyard),
    ];
    for (name, board, natural) in boards {
        for shape in [Shape::Cant, Shape::Replacement, Shape::Trigger] {
            let o = board(Some(shape));
            assert_eq!((o.zone, o.triggered), (natural, 0), "{name} under {shape:?}: {o:?}");
        }
    }
}

// ---------------------------------------------------------------------------
// A destination word reads where the move went
// ---------------------------------------------------------------------------

/// **Fixture.** A trigger on any of its controller's opponents' creatures'
/// moves fitting `change`, however caused.
fn move_watcher(change: ZoneChangePattern) -> Arc<CardData> {
    let def = whenever(
        TriggerEvent::ZoneChange {
            subject: TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature)),
            change,
            by: None,
            owner: Some(PlayerRef::Opponent),
            multiplicity: Multiplicity::PerOccurrence,
        },
        gain_one(),
    );
    CardDataBuilder::new("Fixture Move Watcher").card_type(CardType::Enchantment).ability(triggered_ability("", def)).build()
}

fn triggers_on_a_sacrifice_rest_in_peace_exiles(change: ZoneChangePattern) -> usize {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, rest_in_peace(), 0);
    put_on_battlefield(&mut game, move_watcher(change), 0);
    let victim = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    resolve_at_player(&mut game, diabolic_edict(), 0, 1);
    assert_eq!(game.get_object(victim).unwrap().zone, Zone::Exile, "Rest in Peace exiled it");
    outcome(&game, victim).triggered
}

/// Item 176's trap, closed: "is exiled" is a destination word, so it reads
/// `to`, and catches a creature that was sacrificed and that Rest in Peace
/// exiled. The act is kept (CR 614.6), so "sacrifices" catches it too, and
/// "dies" does not, since it never reached a graveyard.
#[test]
fn a_destination_pattern_catches_a_sacrifice_rest_in_peace_exiled() {
    let exiled = ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Exile), act: None };
    let sacrificed = ZoneChangePattern { from: Some(Zone::Battlefield), to: None, act: Some(ZoneChangeAct::Sacrificed) };
    let dies = ZoneChangePattern { from: Some(Zone::Battlefield), to: Some(Zone::Graveyard), act: None };
    assert_eq!(triggers_on_a_sacrifice_rest_in_peace_exiles(exiled), 1, "\"is exiled\"");
    assert_eq!(triggers_on_a_sacrifice_rest_in_peace_exiles(sacrificed), 1, "\"is sacrificed\"");
    assert_eq!(triggers_on_a_sacrifice_rest_in_peace_exiles(dies), 0, "\"dies\"");
}

// ---------------------------------------------------------------------------
// The record keeps its size
// ---------------------------------------------------------------------------

/// The cause fits where the resolution stamp was (§2b.10): no record copies
/// more bytes for carrying it.
#[test]
fn the_stamp_and_the_record_keep_their_sizes() {
    assert_eq!(std::mem::size_of::<EventStamp>(), 40);
    assert_eq!(std::mem::size_of::<EventRecord>(), 144);
}
