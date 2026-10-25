//! TR-7 — the persist loop (`triggers-architecture.md` §12, TR-7).
//!
//! 1. Grumgully, the Generous: its three rulings.
//! 2. Wingrattle Scarecrow: its seven rulings, Kitchen Finks' seven, asked of
//!    the Scarecrow beside a black creature, and its flying.
//! 3. Viscera Seer: its two rulings.
//! 4. The loop: cast from hand under `ManaWindowStop` and stopped by its
//!    player, with the dispatch audit on; the order of its triggers, taken for
//!    a person or asked of them; the entry's two counter rows and their
//!    removal; and the frame persist is asked off.

use std::cell::{Cell, RefCell};
use std::sync::Arc;

use mtgsim::cards::authoring::persist;
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_cm_cards::thalia_guardian_of_thraben;
use mtgsim::cards::phase_cv_cards::clone;
use mtgsim::cards::phase_lh_cards::holy_strength;
use mtgsim::cards::phase_tr1_cards::blood_artist;
use mtgsim::cards::phase_tr7_cards::{grumgully_the_generous, viscera_seer, wingrattle_scarecrow};
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::combat::validation::can_block;
use mtgsim::engine::priority::PriorityResult;
use mtgsim::events::event::{CounterSubject, DamageTarget, GameEvent};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::{get_effective_name, get_effective_toughness, has_keyword, has_type};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    effect_ctx, put_in_graveyard, put_in_hand, put_on_battlefield, set_attacking, set_blocked_by, set_blocking, setup_two_player_game,
    static_ability, stock_libraries, test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::{CardType, CreatureType, Subtype};
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    CounterType, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerRef, Primitive, SelectionFilter, TypeChange,
};
use mtgsim::types::ids::{AbilityId, ObjectId, ObjectRef, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::{CopyDonor, EnterMods, EntryCopyTemplate, EventPattern, ReplacementDef, Rewrite};
use mtgsim::types::zones::{Zone, ZoneChangeAct};
use mtgsim::ui::auto_order_triggers::AutoOrderTriggers;
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, PriorityAction};
use mtgsim::ui::mana_window_stop::ManaWindowStop;
use mtgsim::ui::random::RandomDecisionProvider;

const PLUS: CounterType = CounterType::PlusOnePlusOne;
const MINUS: CounterType = CounterType::MinusOneMinusOne;

// ---------------------------------------------------------------------------
// Fixtures and helpers
// ---------------------------------------------------------------------------

/// A creature card of one color with no ability.
fn creature_of(name: &str, color: Color, mana: ManaType) -> Arc<CardData> {
    CardDataBuilder::new(name)
        .mana_cost(ManaCost::build(&[mana], 1))
        .color(color)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Bear))
        .power_toughness(2, 2)
        .build()
}

fn black_creature() -> Arc<CardData> {
    creature_of("Black Probe", Color::Black, ManaType::Black)
}

fn blue_creature() -> Arc<CardData> {
    creature_of("Blue Probe", Color::Blue, ManaType::Blue)
}

/// Clone's ability on a Human: "You may have this creature enter as a copy of
/// any creature on the battlefield." A fixture for Grumgully's third ruling's
/// second half; no Human with the plain ability is registered.
fn human_clone() -> Arc<CardData> {
    let text = "You may have this creature enter as a copy of any creature on the battlefield.";
    CardDataBuilder::new("Fixture: a Human clone")
        .mana_cost(ManaCost::build(&[ManaType::Blue], 3))
        .color(Color::Blue)
        .card_type(CardType::Creature)
        .subtype(Subtype::Creature(CreatureType::Human))
        .power_toughness(0, 0)
        .rules_text(text)
        .ability(AbilityDef {
            rules_text: text.into(),
            is_characteristic_defining: false,
            activation_restriction: ActivationRestriction::None,
            id: AbilityId::UNASSIGNED,
            instances: Vec::new(),
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Replacement(Box::new(ReplacementDef {
                optional: true,
                ..ReplacementDef::new(
                    EventPattern::EnterBattlefield { cast: None },
                    ObjectSet::SourceOnly,
                    Rewrite::EnterAsCopy(EntryCopyTemplate { donor: CopyDonor::Chosen(SelectionFilter::Creature), except: Vec::new() }),
                )
            })),
        })
        .build()
}

/// "Creatures you control have persist": a second instance beside the
/// Scarecrow's own, for its third ruling.
fn persist_anthem() -> Arc<CardData> {
    let yours = ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ByController(PlayerRef::You)));
    CardDataBuilder::new("Fixture: creatures you control have persist")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::GrantAbility(Arc::new(persist()), Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(yours),
        )))
        .build()
}

/// "Artifacts are not creatures", for the Scarecrow's first ruling.
fn artifacts_are_not_creatures() -> Arc<CardData> {
    let not_creatures = TypeChange { remove_types: vec![CardType::Creature], ..TypeChange::NONE };
    CardDataBuilder::new("Fixture: artifacts are not creatures")
        .card_type(CardType::Enchantment)
        .ability(static_ability(Effect::Atom(
            Primitive::ChangeType(not_creatures, Duration::WhileSourceOnBattlefield),
            EffectRecipient::FilteredPermanents(ObjectFilter::ByType(CardType::Artifact)),
        )))
        .build()
}

/// `card` entering from `owner`'s graveyard as one event, choices answered by
/// `dp`, so an entry replacement applies to it.
fn enter(game: &mut GameState, card: Arc<CardData>, owner: PlayerId, dp: &dyn DecisionProvider) -> ObjectId {
    let id = put_in_graveyard(game, card, owner);
    game.change_zone(id, Zone::Battlefield, ZoneChangeAct::Put, &ActionContext::new(dp)).expect("it enters");
    id
}

/// A provider asked once, which copy source to choose, and choosing `donor`.
fn choosing_copy_source(donor: ObjectId) -> mtgsim::ui::decision::ScriptedDecisionProvider {
    let dp = test_dp();
    dp.expect_choice(ChoiceKind::ChooseCopySource { source: donor }, vec![ChoiceOption::Object(donor)]);
    dp
}

fn destroy(game: &mut GameState, object: ObjectId) {
    let by = mtgsim::types::ids::new_object_id();
    game.execute_action(GameAction::Destroy { object }, &effect_ctx(game, by)).unwrap();
}

fn add_counters(game: &mut GameState, object: ObjectId, counter: CounterType, n: u32) {
    let by = game.objects[&object].owner;
    game.execute_action(GameAction::AddCounters { subject: CounterSubject::Object(object), counter, n, by }, &test_ctx()).unwrap();
}

/// Put every waiting trigger on the stack and resolve the stack.
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

fn counters(game: &GameState, object: ObjectId, counter: CounterType) -> u32 {
    game.battlefield.get(&object).map_or(0, |entry| entry.counter_count(counter))
}

fn zone(game: &GameState, object: ObjectId) -> Option<Zone> {
    game.objects.get(&object).map(|o| o.zone)
}

/// The Scarecrow on `player`'s battlefield beside a black creature, so it
/// has persist.
fn scarecrow_with_persist(game: &mut GameState, player: PlayerId) -> ObjectId {
    put_on_battlefield(game, black_creature(), player);
    put_on_battlefield(game, wingrattle_scarecrow(), player)
}

// ---------------------------------------------------------------------------
// 1. Grumgully, the Generous
// ---------------------------------------------------------------------------

/// Grumgully's first ruling. A Human gets nothing, nor a creature an opponent
/// controls.
// RULING: Grumgully, the Generous #1 - "A non-Human creature you control that
//   would enter the battlefield with no +1/+1 counters on it enters with one
//   +1/+1 counter on it."
#[test]
fn grumgully_gives_each_non_human_you_control_its_counter() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grumgully_the_generous(), 0);
    let yours = enter(&mut game, grizzly_bears(), 0, &test_dp());
    let human = enter(&mut game, thalia_guardian_of_thraben(), 0, &test_dp());
    let theirs = enter(&mut game, grizzly_bears(), 1, &test_dp());

    assert_eq!(counters(&game, yours, PLUS), 1);
    assert_eq!(get_effective_toughness(&game, yours), Some(3));
    assert_eq!(counters(&game, human, PLUS), 0, "a Human");
    assert_eq!(counters(&game, theirs, PLUS), 0, "an opponent's");
}

/// Grumgully's second ruling: the gather sweeps the battlefield, where
/// neither is as they enter together, so it reaches neither.
// RULING: Grumgully, the Generous #2 - "Any other non-Human creatures that
//   enter the battlefield at the same time as Grumgully won't get a +1/+1
//   counter."
#[test]
fn a_creature_entering_beside_grumgully_gets_no_counter() {
    let mut game = setup_two_player_game();
    let grumgully = put_in_graveyard(&mut game, grumgully_the_generous(), 0);
    let beside = put_in_graveyard(&mut game, grizzly_bears(), 0);
    let together = [grumgully, beside].map(|object| GameAction::EnterBattlefield {
        object,
        from: Some(Zone::Graveyard),
        controller: 0,
        mods: EnterMods::NONE,
        act: Some(ZoneChangeAct::Put),
    });
    let dp = test_dp();
    game.execute_actions(together.to_vec(), &ActionContext::new(&dp)).unwrap();
    assert_eq!((zone(&game, grumgully), zone(&game, beside)), (Some(Zone::Battlefield), Some(Zone::Battlefield)));
    assert_eq!(counters(&game, beside, PLUS), 0, "entered beside it");
    assert_eq!(counters(&game, grumgully, PLUS), 0, "\"other\"");

    let after = enter(&mut game, grizzly_bears(), 0, &test_dp());
    assert_eq!(counters(&game, after, PLUS), 1, "entered after it");
}

/// Grumgully's third ruling: CR 614.12 asks what the creature enters as, which
/// CR 616.1c's copy step has already decided.
// RULING: Grumgully, the Generous #3 - "If a non-Human creature enters the
//   battlefield as a copy of a Human creature, it won't get a +1/+1 counter.
//   Similarly, if a Human enters as a non-Human creature, it will get a +1/+1
//   counter."
#[test]
fn grumgully_reads_the_creature_a_copy_enters_as() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grumgully_the_generous(), 0);
    let thalia = put_on_battlefield(&mut game, thalia_guardian_of_thraben(), 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);

    let copy_of_a_human = enter(&mut game, clone(), 0, &choosing_copy_source(thalia));
    let human_as_bears = enter(&mut game, human_clone(), 0, &choosing_copy_source(bears));

    assert_eq!(get_effective_name(&game, copy_of_a_human), "Thalia, Guardian of Thraben");
    assert_eq!(counters(&game, copy_of_a_human, PLUS), 0, "a Shapeshifter that entered as a Human");
    assert_eq!(get_effective_name(&game, human_as_bears), "Grizzly Bears");
    assert_eq!(counters(&game, human_as_bears, PLUS), 1, "a Human that entered as a Bear");
}

// ---------------------------------------------------------------------------
// 2. Wingrattle Scarecrow
// ---------------------------------------------------------------------------

/// The grant is the condition's: with no black creature the Scarecrow has no
/// persist, and with the only one dying beside it, it has: the frame is from
/// before the event (CR 603.10a).
#[test]
fn the_scarecrows_persist_is_asked_of_the_board_it_died_on() {
    let mut game = setup_two_player_game();
    let alone = put_on_battlefield(&mut game, wingrattle_scarecrow(), 0);
    destroy(&mut game, alone);
    assert_eq!(waiting_from(&game, alone), 0, "no black creature, no persist");

    let scarecrow = put_on_battlefield(&mut game, wingrattle_scarecrow(), 1);
    let black = put_on_battlefield(&mut game, black_creature(), 1);
    let by = mtgsim::types::ids::new_object_id();
    game.execute_actions(vec![GameAction::Destroy { object: scarecrow }, GameAction::Destroy { object: black }], &effect_ctx(&game, by))
        .unwrap();
    assert_eq!(waiting_from(&game, scarecrow), 1, "the black creature was there as it left");
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, scarecrow), Some(Zone::Battlefield));
}

/// Once triggered, persist is independent of the grant that gave it (CR
/// 113.7a): the black creature leaving while it waits changes nothing.
#[test]
fn the_scarecrow_returns_after_its_black_creature_leaves_while_persist_waits() {
    let mut game = setup_two_player_game();
    let black = put_on_battlefield(&mut game, black_creature(), 0);
    let scarecrow = put_on_battlefield(&mut game, wingrattle_scarecrow(), 0);
    destroy(&mut game, scarecrow);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(game.stack.len(), 1, "persist waits");

    destroy(&mut game, black);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, scarecrow), Some(Zone::Battlefield));
    assert_eq!(counters(&game, scarecrow, MINUS), 1);
}

/// The Scarecrow's first ruling. "This creature" names the object, so the
/// grant holds once it is a noncreature artifact.
// RULING: Wingrattle Scarecrow #1 - "If a creature with persist stops being a
//   creature, persist will still work."
#[test]
fn the_scarecrow_persists_after_it_stops_being_a_creature() {
    let mut game = setup_two_player_game();
    let scarecrow = scarecrow_with_persist(&mut game, 0);
    put_on_battlefield(&mut game, artifacts_are_not_creatures(), 0);
    assert!(!has_type(&game, scarecrow, CardType::Creature), "a noncreature artifact");

    destroy(&mut game, scarecrow);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, scarecrow), Some(Zone::Battlefield), "persist still works");
    assert_eq!(counters(&game, scarecrow, MINUS), 1);
}

/// The Scarecrow's second ruling, ATOM-702.79a-001's board: it returns with a
/// -1/-1 counter, and the next time it dies it had one.
// RULING: Wingrattle Scarecrow #2 - "Its last known information (that is, how
//   the creature last existed on the battlefield) is used to determine whether
//   it had a -1/-1 counter on it."
// COVERS: ATOM-702.79a-001
#[test]
fn the_scarecrow_returns_once_and_then_stays_dead() {
    let mut game = setup_two_player_game();
    let scarecrow = scarecrow_with_persist(&mut game, 0);
    destroy(&mut game, scarecrow);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, scarecrow), Some(Zone::Battlefield));
    assert_eq!(counters(&game, scarecrow, MINUS), 1);
    assert_eq!(get_effective_toughness(&game, scarecrow), Some(1));

    destroy(&mut game, scarecrow);
    assert_eq!(waiting_from(&game, scarecrow), 0, "it had a -1/-1 counter");
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, scarecrow), Some(Zone::Graveyard));
}

/// ATOM-702.79a-002's board: a -1/-1 counter from anywhere keeps it dead.
// RULING: Wingrattle Scarecrow #2 - "The persist ability triggers when the
//   permanent is put into a graveyard."
// COVERS: ATOM-702.79a-002
#[test]
fn a_scarecrow_that_died_with_a_minus_counter_stays_dead() {
    let mut game = setup_two_player_game();
    let scarecrow = scarecrow_with_persist(&mut game, 0);
    add_counters(&mut game, scarecrow, MINUS, 1);
    destroy(&mut game, scarecrow);
    assert_eq!(waiting_from(&game, scarecrow), 0);
    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, scarecrow), Some(Zone::Graveyard));
}

/// The Scarecrow's third ruling: its own grant and an anthem's are two
/// instances.
// RULING: Wingrattle Scarecrow #3 - "If a permanent has multiple instances of
//   persist, they'll each trigger separately, but the redundant instances will
//   have no effect."
#[test]
fn two_instances_of_persist_return_the_scarecrow_once() {
    let mut game = setup_two_player_game();
    let scarecrow = scarecrow_with_persist(&mut game, 0);
    put_on_battlefield(&mut game, persist_anthem(), 0);
    destroy(&mut game, scarecrow);
    assert_eq!(waiting_from(&game, scarecrow), 2, "each instance triggers");
    resolve_all(&mut game, &RandomDecisionProvider::seeded(7));
    assert_eq!(zone(&game, scarecrow), Some(Zone::Battlefield));
    assert_eq!(counters(&game, scarecrow, MINUS), 1, "returned once");
}

/// The Scarecrow's fourth ruling.
// RULING: Wingrattle Scarecrow #4 - "If a token with no -1/-1 counters on it
//   has persist, the ability will trigger when the token is put into the
//   graveyard. However, the token will cease to exist and can't return to the
//   battlefield."
#[test]
fn a_scarecrow_token_persists_into_nothing() {
    let mut game = setup_two_player_game();
    let token = scarecrow_with_persist(&mut game, 0);
    game.objects.get_mut(&token).unwrap().is_token = true;
    destroy(&mut game, token);
    assert_eq!(waiting_from(&game, token), 1, "it triggers");
    resolve_all(&mut game, &test_dp());
    assert!(!game.objects.contains_key(&token), "the token ceased to exist");
}

/// The Scarecrow's fifth ruling: untapped, unenchanted, and carrying only its
/// counter.
// RULING: Wingrattle Scarecrow #5 - "When a permanent with persist returns to
//   the battlefield, it's a new object with no memory of or connection to its
//   previous existence."
#[test]
fn the_scarecrow_returns_as_a_new_object() {
    let mut game = setup_two_player_game();
    let scarecrow = scarecrow_with_persist(&mut game, 0);
    let aura = put_on_battlefield(&mut game, holy_strength(), 0);
    assert!(game.attach(aura, scarecrow));
    game.execute_action(GameAction::Tap { object: scarecrow }, &test_ctx()).unwrap();
    let before = game.object_ref(scarecrow).unwrap();

    destroy(&mut game, scarecrow);
    resolve_all(&mut game, &test_dp());
    let entry = game.battlefield.get(&scarecrow).expect("returned");
    assert_ne!(game.object_ref(scarecrow), Some(before), "a new object");
    assert!(!entry.tapped && entry.attached_by.is_empty());
    assert_eq!(zone(&game, aura), Some(Zone::Graveyard), "the Aura did not follow it");
    assert_eq!(get_effective_toughness(&game, scarecrow), Some(1), "2/2 and its one counter");
}

/// The Scarecrow's sixth ruling: the active player's trigger goes on the stack
/// first, so the nonactive player's resolves first (CR 603.3b).
// RULING: Wingrattle Scarecrow #6 - "That means that in a two-player game, the
//   nonactive player's persist creatures will return to the battlefield first,
//   then the active player's persist creatures do the same."
#[test]
fn two_players_scarecrows_return_the_nonactive_players_first() {
    let mut game = setup_two_player_game();
    let actives = scarecrow_with_persist(&mut game, 0);
    let nonactives = scarecrow_with_persist(&mut game, 1);
    let by = mtgsim::types::ids::new_object_id();
    game.execute_actions(vec![GameAction::Destroy { object: actives }, GameAction::Destroy { object: nonactives }], &effect_ctx(&game, by))
        .unwrap();
    resolve_all(&mut game, &test_dp());
    let arrived = |id: ObjectId| game.battlefield.get(&id).map(|entry| entry.timestamp).expect("returned");
    assert!(arrived(nonactives) < arrived(actives), "one at a time, the nonactive player's first");
}

/// The Scarecrow's seventh ruling, on the board where the order inside the
/// check decides it: a 3/3 with Grumgully's counter and 2 damage gets a -1/-1
/// counter. Annihilated first, it would be a 3/3 with 2 damage; in one check
/// it is a 2/2 destroyed with both counters on it, so persist does not
/// trigger.
// RULING: Wingrattle Scarecrow #7 - "If a creature with persist that has +1/+1
//   counters on it receives enough -1/-1 counters to cause it to be destroyed
//   by lethal damage or put into its owner's graveyard for having 0 or less
//   toughness, persist won't trigger and the card won't return to the
//   battlefield."
#[test]
fn a_scarecrow_killed_through_its_plus_counter_stays_dead() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grumgully_the_generous(), 0);
    put_on_battlefield(&mut game, black_creature(), 0);
    let scarecrow = enter(&mut game, wingrattle_scarecrow(), 0, &test_dp());
    assert_eq!(get_effective_toughness(&game, scarecrow), Some(3), "Grumgully's counter");
    let source = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let damage = GameAction::DealDamage {
        source,
        target: DamageTarget::Object(scarecrow),
        amount: 2,
        is_combat: false,
        unpreventable: false,
        source_lki: None,
    };
    game.execute_action(damage, &effect_ctx(&game, source)).unwrap();
    add_counters(&mut game, scarecrow, MINUS, 1);

    resolve_all(&mut game, &test_dp());
    assert_eq!(zone(&game, scarecrow), Some(Zone::Graveyard), "it had a -1/-1 counter as it left");
}

/// The flying clause (CR 702.9b): blockable by a ground creature without a
/// blue creature beside it, not with one.
#[test]
fn the_scarecrow_flies_while_you_control_a_blue_creature() {
    let mut game = setup_two_player_game();
    let scarecrow = put_on_battlefield(&mut game, wingrattle_scarecrow(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    set_attacking(&mut game, scarecrow, 1);
    assert!(!has_keyword(&game, scarecrow, KeywordFlag::Flying));
    assert!(can_block(&game, 1, bears, scarecrow).is_ok());

    let blue = put_on_battlefield(&mut game, blue_creature(), 0);
    assert!(has_keyword(&game, scarecrow, KeywordFlag::Flying));
    assert!(can_block(&game, 1, bears, scarecrow).is_err(), "a ground creature can't block it");

    destroy(&mut game, blue);
    assert!(!has_keyword(&game, scarecrow, KeywordFlag::Flying), "as long as");
}

// ---------------------------------------------------------------------------
// 3. Viscera Seer
// ---------------------------------------------------------------------------

/// The Seer's first ruling. Alone, it is the one creature the cost can take,
/// so nothing is asked.
// RULING: Viscera Seer #1 - "You can sacrifice Viscera Seer to activate its
//   own ability."
#[test]
fn viscera_seer_can_sacrifice_itself() {
    let mut game = setup_two_player_game();
    stock_libraries(&mut game, 3);
    let seer = put_on_battlefield(&mut game, viscera_seer(), 0);
    let dp = test_dp();
    dp.expect_pick_n(ChoiceKind::Scry { source: Some(seer), n: 1 }, vec![0]);
    game.activate_ability(0, seer, 0, &dp).unwrap();
    assert_eq!(zone(&game, seer), Some(Zone::Graveyard), "paid before the ability is on the stack");
    assert_eq!(game.stack.len(), 1);
    resolve_all(&mut game, &dp);
    assert_eq!(dp.asked().len(), 1, "the scry, and nothing else");
}

/// The Seer's second ruling. A blocked creature sacrificed after blockers
/// deals no combat damage; one dealt lethal damage is gone before anyone has
/// priority to sacrifice it (CR 117.5).
// RULING: Viscera Seer #2 - "If you sacrifice an attacking or blocking creature
//   during the declare blockers step, it won't deal combat damage. If you wait
//   until the combat damage step, but that creature is dealt lethal damage,
//   it'll be destroyed before you get a chance to sacrifice it."
#[test]
fn a_blocked_creature_sacrificed_to_the_seer_deals_no_combat_damage() {
    let mut game = setup_two_player_game();
    let seer = put_on_battlefield(&mut game, viscera_seer(), 0);
    let attacker = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let blocker = put_on_battlefield(&mut game, grizzly_bears(), 1);
    set_attacking(&mut game, attacker, 1);
    set_blocked_by(&mut game, attacker, vec![blocker]);
    set_blocking(&mut game, blocker, vec![attacker]);

    let dp = test_dp();
    dp.expect_choice(ChoiceKind::ChooseSacrificeForCost { spell_or_ability_id: seer, count: 1 }, vec![ChoiceOption::Object(attacker)]);
    game.activate_ability(0, seer, 0, &dp).unwrap();
    resolve_all(&mut game, &dp);
    game.process_combat_damage(&test_dp(), false).unwrap();
    assert_eq!(game.battlefield[&blocker].damage_marked, 0, "the attacker was gone");

    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, viscera_seer(), 0);
    let attacker = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let blocker = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    set_attacking(&mut game, attacker, 1);
    set_blocked_by(&mut game, attacker, vec![blocker]);
    set_blocking(&mut game, blocker, vec![attacker]);
    game.process_combat_damage(&test_dp(), false).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(zone(&game, attacker), Some(Zone::Graveyard), "destroyed before priority");
}

// ---------------------------------------------------------------------------
// 4. The loop
// ---------------------------------------------------------------------------

/// The player at seat 0 plays the loop and seat 1 passes. Once the Seer and
/// the Scarecrow are named, seat 0 activates the Seer `times` times with the
/// stack empty, sacrificing the Scarecrow each time, then passes. Blood Artist
/// targets seat 1, a scry keeps its card on top, and a trigger order is taken
/// as offered and recorded. Any other prompt fails the test.
struct LoopPlayer {
    seer: Cell<Option<ObjectId>>,
    scarecrow: Cell<Option<ObjectId>>,
    times: Cell<usize>,
    /// The Scarecrow as it was each time it was sacrificed.
    sacrificed: RefCell<Vec<ObjectRef>>,
    /// The sources each trigger-order prompt listed.
    orders_asked: RefCell<Vec<Vec<ObjectId>>>,
}

impl LoopPlayer {
    fn new(times: usize) -> Self {
        LoopPlayer {
            seer: Cell::new(None),
            scarecrow: Cell::new(None),
            times: Cell::new(times),
            sacrificed: RefCell::new(Vec::new()),
            orders_asked: RefCell::new(Vec::new()),
        }
    }
}

fn position(options: &[ChoiceOption], wanted: impl Fn(&ChoiceOption) -> bool) -> usize {
    options.iter().position(wanted).expect("offered")
}

impl DecisionProvider for &LoopPlayer {
    fn pick_n(&self, game: &GameState, player: PlayerId, context: &ChoiceContext, options: &[ChoiceOption], _: (usize, usize)) -> Vec<usize> {
        match &context.kind {
            ChoiceKind::PriorityAction => {
                let seer = options.iter().position(
                    |o| matches!(o, ChoiceOption::Action(PriorityAction::ActivateAbility(id, _)) if Some(*id) == self.seer.get()),
                );
                match seer {
                    Some(i) if player == 0 && game.stack.is_empty() && self.times.get() > 0 => {
                        self.times.set(self.times.get() - 1);
                        vec![i]
                    }
                    _ => vec![position(options, |o| matches!(o, ChoiceOption::Action(PriorityAction::Pass)))],
                }
            }
            ChoiceKind::ChooseSacrificeForCost { .. } => {
                let scarecrow = self.scarecrow.get().expect("the Scarecrow is named");
                self.sacrificed.borrow_mut().push(game.object_ref(scarecrow).expect("on the battlefield"));
                vec![position(options, |o| matches!(o, ChoiceOption::Object(id) if *id == scarecrow))]
            }
            ChoiceKind::SelectRecipients { .. } => vec![position(options, |o| matches!(o, ChoiceOption::Player(1)))],
            ChoiceKind::Scry { .. } => Vec::new(),
            other => panic!("the loop asks no {other:?}"),
        }
    }

    fn pick_number(&self, _: &GameState, _: PlayerId, context: &ChoiceContext, _: u64, _: u64) -> u64 {
        panic!("the loop asks no number: {:?}", context.kind)
    }

    fn allocate(
        &self,
        _: &GameState,
        _: PlayerId,
        context: &ChoiceContext,
        _: u64,
        _: &[ChoiceOption],
        _: &[u64],
        _: Option<&[u64]>,
    ) -> Vec<u64> {
        panic!("the loop asks no split: {:?}", context.kind)
    }

    fn choose_ordering(&self, _: &GameState, _: PlayerId, context: &ChoiceContext, items: &[ChoiceOption]) -> Vec<usize> {
        match &context.kind {
            ChoiceKind::OrderTriggers { .. } => {
                let sources = items
                    .iter()
                    .map(|o| match o {
                        ChoiceOption::Object(id) => *id,
                        other => panic!("a trigger's source, not {other:?}"),
                    })
                    .collect();
                self.orders_asked.borrow_mut().push(sources);
                (0..items.len()).collect()
            }
            other => panic!("the loop orders nothing but triggers: {other:?}"),
        }
    }
}

/// Cast `card` from seat 0's hand with exactly `pool` under `seat`, and
/// resolve it.
fn cast_from_hand(game: &mut GameState, card: Arc<CardData>, pool: &[(ManaType, u64)], seat: &dyn DecisionProvider) -> ObjectId {
    let id = put_in_hand(game, card, 0);
    for &(mana, n) in pool {
        game.players[0].mana_pool.add(mana, n);
    }
    game.cast_spell(0, id, seat).expect("castable from exactly its cost");
    assert_eq!(game.players[0].mana_pool.total(), 0, "the pool was the cost");
    assert_eq!(game.run_priority_round(seat).unwrap(), PriorityResult::StackResolved);
    id
}

/// The four cards cast from hand, and then the loop, as `player` chooses it
/// under `seat`, until both players pass with the stack empty. Returns Blood
/// Artist and the Scarecrow.
fn play_the_loop(game: &mut GameState, player: &LoopPlayer, seat: &dyn DecisionProvider) -> (ObjectId, ObjectId) {
    use ManaType::{Black, Colorless, Green, Red};
    stock_libraries(game, 5);
    cast_from_hand(game, grumgully_the_generous(), &[(Red, 1), (Green, 1), (Colorless, 1)], seat);
    let blood_artist = cast_from_hand(game, blood_artist(), &[(Black, 1), (Colorless, 1)], seat);
    player.seer.set(Some(cast_from_hand(game, viscera_seer(), &[(Black, 1)], seat)));
    let scarecrow = cast_from_hand(game, wingrattle_scarecrow(), &[(Colorless, 3)], seat);
    player.scarecrow.set(Some(scarecrow));
    assert_eq!(counters(game, scarecrow, PLUS), 1, "Grumgully's counter, as it was cast");

    for _ in 0..100 {
        if game.run_priority_round(seat).unwrap() == PriorityResult::PhaseEnds {
            return (blood_artist, scarecrow);
        }
    }
    panic!("the loop did not stop");
}

/// The capstone: three turns of the loop, each the player's choice, then a
/// pass. Every dispatch is asked again under the audit. Each death drains one,
/// each return is a new object, and each return's two counters are gone by the
/// next priority. With full control off, the person is never asked the order
/// of persist and Blood Artist's trigger.
///
/// What it cannot show yet: the engine sees three activations, not a loop.
/// Nothing here recognizes the repeated state, offers CR 731.2's shortcut
/// ("do this N times"), or tells this optional loop from a mandatory one (CR
/// 731.4); the player stops by a count the test gave it. Loop detection is
/// the half of the capstone still to come, and this board is its first
/// optional loop.
#[test]
fn the_loop_cast_from_hand_runs_three_times_and_stops_by_choice() {
    let mut game = setup_two_player_game();
    game.enable_audit();
    let player = LoopPlayer::new(3);
    let seat = AutoOrderTriggers::new(ManaWindowStop::new(&player));
    let (_, scarecrow) = play_the_loop(&mut game, &player, &seat);

    assert_eq!(player.times.get(), 0, "it stopped by passing, after three");
    assert_eq!((game.players[0].life_total, game.players[1].life_total), (23, 17), "three drains");
    let mut existences = player.sacrificed.borrow().clone();
    existences.push(game.object_ref(scarecrow).expect("back on the battlefield"));
    let distinct = existences.iter().enumerate().all(|(i, a)| existences[..i].iter().all(|b| b != a));
    assert!(existences.len() == 4 && distinct, "each return a new object (CR 400.7): {existences:?}");
    assert_eq!((counters(&game, scarecrow, PLUS), counters(&game, scarecrow, MINUS)), (0, 0));
    assert_eq!(get_effective_toughness(&game, scarecrow), Some(2));
    assert!(player.orders_asked.borrow().is_empty(), "the decorator took each order");
}

/// The same loop with full control on: the person is asked at each death, and
/// is offered the triggers in timestamp order, Blood Artist's first since it
/// is the older source. Taking them as offered plays the game the decorator
/// did.
#[test]
fn the_loop_asks_its_trigger_order_of_a_person_in_full_control() {
    let mut game = setup_two_player_game();
    let player = LoopPlayer::new(3);
    let seat = ManaWindowStop::new(&player);
    let (blood_artist, scarecrow) = play_the_loop(&mut game, &player, &seat);

    assert_eq!(*player.orders_asked.borrow(), vec![vec![blood_artist, scarecrow]; 3]);
    assert_eq!((game.players[0].life_total, game.players[1].life_total), (23, 17));
}

/// Each return enters with two counter rows, persist's and Grumgully's,
/// announced with the entry (CR 122.6); the next state-based check removes one
/// of each as one batch (CR 704.5q), naming no player.
#[test]
fn the_return_announces_two_counter_rows_and_one_check_removes_both() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, grumgully_the_generous(), 0);
    let scarecrow = scarecrow_with_persist(&mut game, 0);
    let start = game.recorded_events().records().len();
    destroy(&mut game, scarecrow);
    resolve_all(&mut game, &test_dp());

    let log = game.recorded_events();
    let records = log.records_from(start);
    let entry = records
        .iter()
        .find(|r| matches!(r.event, GameEvent::PermanentEnteredBattlefield { object, .. } if object.id == scarecrow))
        .expect("it returned")
        .batch();
    let put: Vec<_> = records
        .iter()
        .filter_map(|r| match r.event {
            GameEvent::CountersPutOn { subject, counter_type, n, .. } if subject == CounterSubject::Object(scarecrow) => {
                Some((r.batch(), counter_type, n))
            }
            _ => None,
        })
        .collect();
    assert_eq!(put, vec![(entry, MINUS, 1), (entry, PLUS, 1)], "persist's row, then Grumgully's");
    let removed: Vec<_> = records
        .iter()
        .filter_map(|r| match r.event {
            GameEvent::CountersRemovedFrom { subject, counter_type, n, by, .. } if subject == CounterSubject::Object(scarecrow) => {
                Some((r.batch(), counter_type, n, by))
            }
            _ => None,
        })
        .collect();
    assert_eq!(removed.len(), 2, "one of each");
    assert_eq!(removed[0].0, removed[1].0, "one batch");
    assert_ne!(removed[0].0, entry, "the check after the entry");
    assert!(removed.iter().all(|&(_, _, n, by)| n == 1 && by.is_none()), "a rule removed them");
}
