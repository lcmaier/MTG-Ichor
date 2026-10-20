//! TR-5a — the binding's roles (`triggers-architecture.md` §12, TR-5a).
//!
//! 1. Item 267: a permanent whose controller changes is removed from combat
//!    (CR 506.4), and an attacker whose blocker is taken stays blocked (CR
//!    509.1h).
//! 2. Item 255: a damage record's two event roles, the source and the
//!    recipient, and the instruction names the one its "that creature" means
//!    (CR 608.2c). A source that left before dealing the damage is the
//!    existence that dealt it (CR 608.2h).
//! 3. Item 250: "them" over damage, an attack and entries, each occurrence's
//!    object, found while it is still that object (CR 400.7).
//! 4. `ObjectFilter::WithCounters`, on the permanent and as it last existed,
//!    and Sphere Grid's grant and its ruling (CR 506.4a).

use std::sync::Arc;

use mtgsim::cards::authoring::{attacks, dies, triggered_ability, whenever};
use mtgsim::cards::creatures::grizzly_bears;
use mtgsim::cards::phase_lg_cards::act_of_treason;
use mtgsim::cards::phase_re_cards::soldier_token;
use mtgsim::cards::phase_tr4b_cards::grafted_wargear;
use mtgsim::cards::phase_tr4d_cards::ray_of_command;
use mtgsim::cards::phase_tr5a_cards::sphere_grid;
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::objects::card_data::{AbilityDef, AbilityType, ActivationRestriction, CardData, CardDataBuilder};
use mtgsim::oracle::characteristics::has_keyword;
use mtgsim::state::game_state::{GameState, Phase, PhaseType, StepType};
use mtgsim::test_support::{
    fill_library, put_in_graveyard, put_in_hand, put_on_battlefield, set_active_player, set_attacking, set_blocked_by,
    set_blocking, setup_game, setup_two_player_game, test_dp, vanilla_creature, RecordingDecisionProvider,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::effects::{
    AmountExpr, Antecedent, Condition, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, PlayerGroup, PlayerRef, PlayerSet,
    Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::history::CountIs;
use mtgsim::types::ids::{AbilityId, ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::ManaType;
use mtgsim::types::triggers::{
    DamageRecipient, EventRole, Multiplicity, TriggerCondition, TriggerDef, TriggerEvent, TriggerSubject,
};
use mtgsim::ui::choice_types::{ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn stocked(players: usize) -> GameState {
    let mut game = if players == 2 { setup_two_player_game() } else { setup_game(players) };
    for p in 0..players {
        fill_library(&mut game, p, 10);
    }
    game
}

fn controller(game: &GameState, id: ObjectId) -> Option<PlayerId> {
    mtgsim::oracle::characteristics::get_effective_controller(game, id)
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

/// `card` cast by `player` from exactly `pool`, each instance of "target"
/// answered with the next of `targets`.
fn cast_at(game: &mut GameState, player: PlayerId, card: Arc<CardData>, pool: &[(ManaType, u64)], targets: &[ChoiceOption]) -> ObjectId {
    let aim = ScriptedDecisionProvider::new();
    for target in targets {
        aim.expect_choice(
            ChoiceKind::SelectRecipients { recipient: EffectRecipient::Implicit, spell_id: ObjectId::UNASSIGNED },
            vec![target.clone()],
        );
    }
    let id = put_in_hand(game, card, player);
    for &(t, n) in pool {
        game.players[player].mana_pool.add(t, n);
    }
    game.cast_spell(player, id, &ManaWindowStop::new(aim)).expect("castable from exactly its cost");
    id
}

/// Ray of Command cast by player 0 at `creature`, the one creature an
/// opponent controls, so no prompt asks for it; resolved.
fn ray_at(game: &mut GameState, creature: ObjectId) {
    cast_at(game, 0, ray_of_command(), &[(ManaType::Blue, 4)], &[]);
    game.resolve_top_of_stack(&test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(controller(game, creature), Some(0));
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
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(victim)]),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
}

/// Put every waiting trigger on the stack and resolve the stack, answering
/// every prompt with `dp`.
fn resolve_all(game: &mut GameState, dp: &dyn DecisionProvider) {
    game.perform_sba_and_triggers(dp).unwrap();
    while !game.stack.is_empty() {
        game.resolve_top_of_stack(dp).unwrap();
        game.perform_sba_and_triggers(dp).unwrap();
    }
}

/// `effect` resolving as `player`'s sorcery aimed at `targets`.
fn resolve_as(game: &mut GameState, player: PlayerId, effect: &Effect, targets: Vec<ResolvedTarget>) {
    let source = put_in_graveyard(game, act_of_treason(), player);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller: player,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(targets),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(effect, &ctx, &test_dp()).unwrap();
}

fn destroy(game: &mut GameState, id: ObjectId) {
    let destroy = Effect::Atom(Primitive::Destroy, EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)));
    resolve_as(game, 0, &destroy, vec![ResolvedTarget::Object(id)]);
}

fn plus_ones(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].counter_count(CounterType::PlusOnePlusOne)
}

/// An enchantment with one triggered ability.
fn watcher(name: &str, arm: TriggerEvent, effect: Effect) -> Arc<CardData> {
    CardDataBuilder::new(name).card_type(CardType::Enchantment).ability(triggered_ability("", whenever(arm, effect))).build()
}

fn a_counter_on(recipient: EffectRecipient) -> Effect {
    Effect::Atom(
        Primitive::AddCounters { counter: CounterType::PlusOnePlusOne, amount: AmountExpr::Fixed(1), by: PlayerRef::You },
        recipient,
    )
}

fn creature_you_control() -> ObjectFilter {
    ObjectFilter::And(Box::new(ObjectFilter::ByType(CardType::Creature)), Box::new(ObjectFilter::ByController(PlayerRef::You)))
}

/// "Whenever [a creature you control] deals combat damage to a player".
fn combat_damage_to_a_player(multiplicity: Multiplicity) -> TriggerEvent {
    TriggerEvent::DamageDealt {
        source: TriggerSubject::Filter(creature_you_control()),
        recipient: DamageRecipient::Player(None),
        combat: Some(true),
        excess: false, multiplicity,
    }
}

fn static_ability(effect: Effect) -> AbilityDef {
    AbilityDef {
        rules_text: "".into(),
        is_characteristic_defining: false,
        activation_restriction: ActivationRestriction::None,
        id: AbilityId::UNASSIGNED,
        instances: Vec::new(),
        ability_type: AbilityType::Static,
        costs: Vec::new(),
        effect,
    }
}

// ---------------------------------------------------------------------------
// 1. Item 267: a change of control removes the permanent from combat
// ---------------------------------------------------------------------------

/// Player 1's Grizzly Bears attack player 0, who takes them with Ray of
/// Command. CR 506.4 removes them from combat, so they deal no combat damage,
/// and least of all to the player who now controls them.
// COVERS: ATOM-506.4-002
#[test]
fn a_stolen_attacker_is_removed_from_combat() {
    let mut game = stocked(2);
    set_active_player(&mut game, 1);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    set_attacking(&mut game, bears, 0);

    ray_at(&mut game, bears);
    assert!(game.battlefield[&bears].attacking.is_none(), "CR 506.4: its controller changed");
    game.process_combat_damage(&test_dp(), false).unwrap();
    assert_eq!(life(&game, 0), 20, "no combat damage, and none to its own controller");
}

/// Player 0's Grizzly Bears block player 1's attacker, and player 1 takes the
/// blocker. The blocker leaves combat (CR 506.4), and the attacker stays
/// blocked with nothing blocking it (CR 509.1h), so it deals no damage.
#[test]
fn an_attacker_whose_blocker_is_taken_stays_blocked() {
    let mut game = stocked(2);
    set_active_player(&mut game, 1);
    let attacker = put_on_battlefield(&mut game, grizzly_bears(), 1);
    let blocker = put_on_battlefield(&mut game, grizzly_bears(), 0);
    set_attacking(&mut game, attacker, 0);
    set_blocked_by(&mut game, attacker, vec![blocker]);
    set_blocking(&mut game, blocker, vec![attacker]);

    steal(&mut game, 1, blocker);
    assert_eq!(controller(&game, blocker), Some(1));
    assert!(game.battlefield[&blocker].blocking.is_none(), "CR 506.4: the blocker left combat");
    let info = game.battlefield[&attacker].attacking.as_ref().expect("the attacker is still attacking");
    assert!(info.is_blocked && info.blocked_by.is_empty(), "CR 509.1h: blocked, by nothing");
    game.process_combat_damage(&test_dp(), false).unwrap();
    assert_eq!(life(&game, 0), 20, "a blocked creature with no blockers deals no damage");
}

// ---------------------------------------------------------------------------
// 2. Item 255: the damage record's two event roles
// ---------------------------------------------------------------------------

/// Sphere Grid's "that creature" is the creature that dealt the combat
/// damage, the source the instruction names (CR 608.2c).
#[test]
fn sphere_grid_puts_its_counter_on_the_creature_that_dealt_the_damage() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, sphere_grid(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    set_attacking(&mut game, bears, 1);

    game.process_combat_damage(&test_dp(), false).unwrap();
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 1), 18);
    assert_eq!(plus_ones(&game, bears), 1, "the counter went on the source");
}

/// "Whenever a creature is dealt damage, destroy that creature": the same
/// record, and the instruction names its recipient. The attacker that dealt
/// the damage is untouched.
#[test]
fn that_creature_is_the_recipient_when_the_instruction_names_it() {
    let mut game = stocked(2);
    let destroy_it = Effect::Atom(Primitive::Destroy, EffectRecipient::Triggering(EventRole::DamageRecipient));
    let dealt_damage = TriggerEvent::DamageDealt {
        source: TriggerSubject::Any,
        recipient: DamageRecipient::Object(Some(ObjectFilter::ByType(CardType::Creature))),
        combat: None,
        excess: false,
        multiplicity: Multiplicity::PerOccurrence,
    };
    put_on_battlefield(&mut game, watcher("Fixture Wound Keeper", dealt_damage, destroy_it), 1);
    let attacker = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let wall = put_on_battlefield(&mut game, vanilla_creature(0, 4, &[]), 1);
    set_attacking(&mut game, attacker, 1);
    set_blocked_by(&mut game, attacker, vec![wall]);
    set_blocking(&mut game, wall, vec![attacker]);

    game.process_combat_damage(&test_dp(), false).unwrap();
    resolve_all(&mut game, &test_dp());
    assert!(!game.battlefield.contains_key(&wall), "the creature dealt damage was destroyed");
    assert!(game.battlefield.contains_key(&attacker), "the one that dealt it was not");
}

/// **Fixture.** A 2/2: "When this creature dies, it deals damage equal to its
/// power to each opponent." Its damage is dealt by a source that has left.
fn martyr() -> Arc<CardData> {
    let deals = Effect::Atom(
        Primitive::DealDamage { amount: AmountExpr::TriggeringPower(EventRole::Object), unpreventable: false },
        EffectRecipient::EachOf(PlayerGroup::set(PlayerSet::Opponents)),
    );
    CardDataBuilder::new("Fixture Martyr")
        .card_type(CardType::Creature)
        .power_toughness(2, 2)
        .ability(triggered_ability("", whenever(dies(TriggerSubject::ThisObject), deals)))
        .build()
}

/// A source that left before dealing its damage is the existence that dealt
/// it (CR 608.2h): "its power" reads it as it last existed, and "that
/// creature" finds nothing, though the card it became is in a graveyard
/// (CR 400.7).
#[test]
fn a_source_that_left_before_dealing_damage_is_read_as_it_left() {
    let mut game = stocked(2);
    let gains_its_power = Effect::Sequence(vec![
        Effect::Atom(Primitive::GainLife(AmountExpr::TriggeringPower(EventRole::DamageSource)), EffectRecipient::Controller),
        a_counter_on(EffectRecipient::Triggering(EventRole::DamageSource)),
    ]);
    let dealt_damage_by_a_creature = TriggerEvent::DamageDealt {
        source: TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature)),
        recipient: DamageRecipient::Player(Some(PlayerRef::You)),
        combat: None,
        excess: false,
        multiplicity: Multiplicity::PerOccurrence,
    };
    put_on_battlefield(&mut game, watcher("Fixture Leech", dealt_damage_by_a_creature, gains_its_power), 1);
    let martyr = put_on_battlefield(&mut game, martyr(), 0);

    destroy(&mut game, martyr);
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    game.resolve_top_of_stack(&test_dp()).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    let top = *game.stack.last().expect("the leech's trigger");
    let binding = game.stack_entries[&top].trigger.as_ref().expect("a triggered ability");
    let dealt_by = binding.triggering_object(EventRole::DamageSource).expect("the damage's source");
    assert_ne!(Some(dealt_by), game.object_ref(martyr), "the existence that dealt it, not the card it became");
    assert!(game.bound_objects(binding, EventRole::DamageSource).is_empty(), "which nothing finds");
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 1), 20, "dealt 2, then gained its power as it last existed, 2");
}

// ---------------------------------------------------------------------------
// 3. Item 250: "them" over every record
// ---------------------------------------------------------------------------

/// "Whenever one or more creatures you control deal combat damage to a player,
/// put a +1/+1 counter on each of them": one trigger, and each creature that
/// dealt the damage.
#[test]
fn them_over_combat_damage_is_each_creature_that_dealt_it() {
    let mut game = stocked(2);
    let on_each = a_counter_on(EffectRecipient::Triggering(EventRole::DamageSource));
    put_on_battlefield(&mut game, watcher("Fixture Bloodied Banner", combat_damage_to_a_player(Multiplicity::OncePerEvent), on_each), 0);
    let first = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let second = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let idle = put_on_battlefield(&mut game, grizzly_bears(), 0);
    set_attacking(&mut game, first, 1);
    set_attacking(&mut game, second, 1);

    game.process_combat_damage(&test_dp(), false).unwrap();
    assert_eq!(game.pending_triggers.len(), 1, "one or more: one trigger");
    resolve_all(&mut game, &test_dp());
    assert_eq!((plus_ones(&game, first), plus_ones(&game, second), plus_ones(&game, idle)), (1, 1, 0));
}

/// "Whenever one or more creatures you control attack, put a +1/+1 counter on
/// each of them".
#[test]
fn them_over_an_attack_is_each_attacker() {
    let mut game = stocked(2);
    let on_each = a_counter_on(EffectRecipient::Triggering(EventRole::Attacker));
    let attack = attacks(TriggerSubject::Filter(creature_you_control())).once_per_event();
    put_on_battlefield(&mut game, watcher("Fixture War Drum", attack, on_each), 0);
    let first = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let second = put_on_battlefield(&mut game, grizzly_bears(), 0);
    game.set_turn_position(Phase { phase_type: PhaseType::Combat, step: Some(StepType::DeclareAttackers) });

    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::DeclareAttackers, vec![0, 1]);
    game.process_declare_attackers(&dp).unwrap();
    resolve_all(&mut game, &test_dp());
    assert_eq!((plus_ones(&game, first), plus_ones(&game, second)), (1, 1));
}

/// "Whenever one or more creatures enter under your control, put a +1/+1
/// counter on each of them": two tokens made by one instruction are one
/// event, and the one destroyed before the trigger resolves is not found.
#[test]
fn them_over_entries_is_each_creature_still_there() {
    let mut game = stocked(2);
    let on_each = a_counter_on(EffectRecipient::Triggering(EventRole::Object));
    let enters = TriggerEvent::EntersBattlefield {
        subject: TriggerSubject::Filter(ObjectFilter::ByType(CardType::Creature)),
        controller: Some(PlayerRef::You),
        from: None,
        was_cast: None,
        multiplicity: Multiplicity::OncePerEvent,
    };
    put_on_battlefield(&mut game, watcher("Fixture Muster Hall", enters, on_each), 0);
    let two_soldiers = Effect::Atom(Primitive::CreateToken(soldier_token(), AmountExpr::Fixed(2)), EffectRecipient::Controller);
    resolve_as(&mut game, 0, &two_soldiers, Vec::new());
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    let soldiers: Vec<ObjectId> = game
        .battlefield_ids_ordered()
        .into_iter()
        .filter(|&id| game.objects[&id].is_token)
        .collect();
    assert_eq!((soldiers.len(), game.stack.len()), (2, 1), "two tokens, one trigger");

    destroy(&mut game, soldiers[0]);
    resolve_all(&mut game, &test_dp());
    assert_eq!(plus_ones(&game, soldiers[1]), 1);
}

// ---------------------------------------------------------------------------
// 4. ObjectFilter::WithCounters, and Sphere Grid's grant
// ---------------------------------------------------------------------------

/// "Creatures you control with +1/+1 counters on them have reach and
/// trample": the creature has both while it has a counter, and neither once
/// it has none.
#[test]
fn sphere_grid_grants_reach_and_trample_to_a_creature_with_a_counter() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, sphere_grid(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let keywords = |game: &GameState| (has_keyword(game, bears, KeywordFlag::Reach), has_keyword(game, bears, KeywordFlag::Trample));
    assert_eq!(keywords(&game), (false, false));

    game.add_counters(bears, CounterType::PlusOnePlusOne, 1);
    assert_eq!(keywords(&game), (true, true));
    game.remove_counters(bears, CounterType::PlusOnePlusOne, 1);
    assert_eq!(keywords(&game), (false, false));
}

/// **Fixture.** "Creatures with no counters on them have flying": any kind of
/// counter counts.
#[test]
fn a_filter_with_no_kind_counts_every_kind_of_counter() {
    let mut game = stocked(2);
    let bare = ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::WithCounters { kind: None, count: CountIs::AtMost(0) }),
    );
    let flying = Effect::Atom(
        Primitive::GrantKeywordFlag(KeywordFlag::Flying, Duration::WhileSourceOnBattlefield),
        EffectRecipient::FilteredPermanents(bare),
    );
    put_on_battlefield(&mut game, CardDataBuilder::new("Fixture Bare Sky").card_type(CardType::Enchantment).ability(static_ability(flying)).build(), 0);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 1);
    assert!(has_keyword(&game, bears, KeywordFlag::Flying));

    game.add_counters(bears, CounterType::Charge, 1);
    assert!(!has_keyword(&game, bears, KeywordFlag::Flying), "a charge counter is a counter");
}

/// "Whenever a creature with a +1/+1 counter on it dies, you gain 3 life": a
/// dies trigger looks back (CR 603.10a), so the filter counts the counters
/// the creature died with.
#[test]
fn a_look_back_filter_counts_the_counters_it_died_with() {
    let mut game = stocked(2);
    let with_a_counter = ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::WithCounters { kind: Some(CounterType::PlusOnePlusOne), count: CountIs::AtLeast(1) }),
    );
    let gain = Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(3)), EffectRecipient::Controller);
    put_on_battlefield(&mut game, watcher("Fixture Wake", dies(TriggerSubject::Filter(with_a_counter)).into(), gain), 0);
    let countered = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let bare = put_on_battlefield(&mut game, grizzly_bears(), 0);
    game.add_counters(countered, CounterType::PlusOnePlusOne, 1);

    destroy(&mut game, bare);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 20, "no counter, no trigger");
    destroy(&mut game, countered);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 23);
}

/// Angelic Sleuth's shape, "Whenever another permanent you control leaves the
/// battlefield, if it had counters on it, [you gain 2 life]": an intervening
/// "if" (CR 603.4) that counts every kind of counter, read off the frame the
/// permanent left with.
#[test]
fn an_intervening_if_counts_every_kind_of_counter_it_had() {
    let mut game = stocked(2);
    let had_counters = Condition::CounterCount {
        object: Antecedent::Triggering(EventRole::Object),
        kind: None,
        count: CountIs::AtLeast(1),
    };
    let leaves = mtgsim::cards::authoring::leaves_the_battlefield(TriggerSubject::Filter(ObjectFilter::And(
        Box::new(ObjectFilter::NotSource),
        Box::new(ObjectFilter::ByController(PlayerRef::You)),
    )));
    let def = TriggerDef {
        condition: TriggerCondition::Event(leaves.into()),
        intervening_if: Some(had_counters),
        limit: None,
        effect: Effect::Atom(Primitive::GainLife(AmountExpr::Fixed(2)), EffectRecipient::Controller),
    };
    put_on_battlefield(
        &mut game,
        CardDataBuilder::new("Fixture Sleuth").card_type(CardType::Enchantment).ability(triggered_ability("", def)).build(),
        0,
    );
    let charged = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let bare = put_on_battlefield(&mut game, grizzly_bears(), 0);
    game.add_counters(charged, CounterType::Charge, 1);

    destroy(&mut game, bare);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 20, "it had no counters");
    destroy(&mut game, charged);
    resolve_all(&mut game, &test_dp());
    assert_eq!(life(&game, 0), 22, "a charge counter is a counter");
}

/// Player 1's 2/2 flier attacks player 0, whose Grizzly Bears with a +1/+1
/// counter block it with Sphere Grid's reach. Returns the flier and the
/// blocker, the block declared.
fn a_reach_block(game: &mut GameState) -> (ObjectId, ObjectId) {
    set_active_player(game, 1);
    let flier = put_on_battlefield(game, vanilla_creature(2, 2, &[KeywordFlag::Flying]), 1);
    let blocker = put_on_battlefield(game, grizzly_bears(), 0);
    game.add_counters(blocker, CounterType::PlusOnePlusOne, 1);
    set_attacking(game, flier, 0);
    let dp = ScriptedDecisionProvider::new();
    dp.expect_pick_n(ChoiceKind::DeclareBlockers, vec![0]);
    game.process_declare_blockers(&dp).unwrap();
    assert!(dp.is_empty(), "reach let it block the flier");
    (flier, blocker)
}

// RULING: Sphere Grid #1 - "Once a creature that has reach because of Sphere Grid's last ability has blocked a creature, removing all +1/+1 counters from that creature or causing Sphere Grid to leave the battlefield won't cause that creature to stop blocking."
#[test]
fn a_sphere_grid_blocker_keeps_blocking_once_its_counters_go() {
    let mut game = stocked(2);
    put_on_battlefield(&mut game, sphere_grid(), 0);
    let (flier, blocker) = a_reach_block(&mut game);

    game.remove_counters(blocker, CounterType::PlusOnePlusOne, 1);
    assert!(!has_keyword(&game, blocker, KeywordFlag::Reach));
    assert!(game.battlefield[&blocker].blocking.is_some(), "CR 506.4a: still blocking");
    game.process_combat_damage(&test_dp(), false).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(life(&game, 0), 20, "the flier stayed blocked");
    assert!(!game.battlefield.contains_key(&flier), "and the blocker's 2 damage killed it");
}

// RULING: Sphere Grid #1 - "Once a creature that has reach because of Sphere Grid's last ability has blocked a creature, removing all +1/+1 counters from that creature or causing Sphere Grid to leave the battlefield won't cause that creature to stop blocking."
#[test]
fn a_sphere_grid_blocker_keeps_blocking_once_the_grid_leaves() {
    let mut game = stocked(2);
    let grid = put_on_battlefield(&mut game, sphere_grid(), 0);
    let (flier, blocker) = a_reach_block(&mut game);

    let destroy_grid = Effect::Atom(Primitive::Destroy, EffectRecipient::Target(SelectionFilter::Permanent(ObjectFilter::All), TargetCount::Exactly(1)));
    resolve_as(&mut game, 1, &destroy_grid, vec![ResolvedTarget::Object(grid)]);
    assert!(!has_keyword(&game, blocker, KeywordFlag::Reach));
    assert!(game.battlefield[&blocker].blocking.is_some(), "CR 506.4a: still blocking");
    game.process_combat_damage(&test_dp(), false).unwrap();
    game.perform_sba_and_triggers(&test_dp()).unwrap();
    assert_eq!(life(&game, 0), 20, "the flier stayed blocked");
    assert!(!game.battlefield.contains_key(&flier), "and the 3/3 blocker killed it");
}

// ---------------------------------------------------------------------------
// 5. The elision compares the event roles a def reads, and no others
// ---------------------------------------------------------------------------

/// Two Grafted Wargears on one creature, which dies: each becomes unattached,
/// and each trigger sacrifices "that permanent", the host they share. The def
/// reads the host and not the attachment, so the two entries agree on every
/// event role it reads and go on the stack unasked (§5.2's elision), as
/// before the binding kept every event role.
#[test]
fn entries_differing_only_in_an_event_role_the_def_does_not_read_are_not_ordered() {
    let mut game = stocked(2);
    let bears = put_on_battlefield(&mut game, grizzly_bears(), 0);
    let first = put_on_battlefield(&mut game, grafted_wargear(), 0);
    let second = put_on_battlefield(&mut game, grafted_wargear(), 0);
    assert!(game.attach(first, bears) && game.attach(second, bears));

    destroy(&mut game, bears);
    let dp = RecordingDecisionProvider::picking(0);
    game.perform_sba_and_triggers(&dp).unwrap();
    assert_eq!(game.stack.len(), 2, "both triggered");
    assert!(!dp.kinds().contains(&"OrderTriggers".to_string()), "and nothing asked their order: {:?}", dp.kinds());
}
