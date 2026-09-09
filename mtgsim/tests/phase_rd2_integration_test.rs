//! Phase RD-2 — CR 615.7 prevention shields, and the loop's unit.
//!
//! Two things change here and every test below is about one of them
//! (`replacement-architecture.md` §9, RD decisions 2, 3 and 7):
//!
//! - **A resolution can create a prevention effect with a count.** Mending
//!   Hands' "prevent the next 4 damage" is a registry row with
//!   `Uses::NextDamage(4)`, spent by exactly what each application prevented
//!   (CR 615.7, 609.7b), gone at zero or at cleanup (CR 615.3).
//! - **CR 616.1's loop decides per `(batch, subject)` and rewrites per
//!   member.** The regression is one board: a creature with two shield
//!   counters blocked by two attackers loses **one** counter; give one blocker
//!   first strike and it loses **two**, because CR 510.4 makes two combat
//!   damage steps. CR 615.7's allocation across simultaneous sources is the
//!   one place a rewrite is not member-uniform, and it is asked once per
//!   instance.
//!
//! The card file's tests are the rulings pass; these are the rules' own.

use std::sync::Arc;

use mtgsim::cards::keyword_creatures::knight_of_meadowgrain;
use mtgsim::cards::phase_rd_cards::{
    angel_of_suffering, furnace_of_rath, mending_hands, safe_passage, samite_censer_bearer,
};
use mtgsim::cards::basic_lands::plains;
use mtgsim::engine::actions::{ActionContext, GameAction};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::events::event::{BatchId, DamageTarget, GameEvent};
use mtgsim::objects::card_data::CardData;
use mtgsim::state::game_state::GameState;
use mtgsim::state::replacement_effects::RegisteredReplacementEffect;
use mtgsim::test_support::{
    fill_library, place_vanilla_creature, put_in_hand, put_land_on_battlefield,
    put_on_battlefield, set_attacking, set_blocked_by, set_blocking, setup_two_player_game,
    test_ctx, test_dp, RecordingDecisionProvider,
};
use mtgsim::types::effects::{
    AffectedSet, AmountExpr, CounterType, Duration, Effect, EffectRecipient, ObjectFilter,
    PatternFill, PlayerRef, PlayerSet, Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::replacement::{
    AmountRewrite, EventPattern, ReplacementDef, Rewrite, Rounding, Uses,
};
use mtgsim::types::zones::ZoneChangeCause;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::ScriptedDecisionProvider;
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve `card`'s spell effect for `controller` against `targets`, the way
/// the stack would; the card's own id is the resolution's source.
fn resolve_spell(
    game: &mut GameState,
    card: Arc<CardData>,
    controller: PlayerId,
    targets: Vec<ResolvedTarget>,
) -> ObjectId {
    let id = put_in_hand(game, card.clone(), controller);
    let ctx = ResolutionContext {
        source: id,
        ability_source: None,
        controller,
        targets,
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(&card.abilities[0].effect, &ctx, &test_dp()).unwrap();
    id
}

/// Mending Hands cast by `caster` on `target`.
fn mending_hands_on(game: &mut GameState, caster: PlayerId, target: ResolvedTarget) -> ObjectId {
    resolve_spell(game, mending_hands(), caster, vec![target])
}

/// A 1/1 for `owner` to be a damage source.
fn source_for(game: &mut GameState, owner: PlayerId) -> ObjectId {
    place_vanilla_creature(game, owner, 1, 1, &[])
}

fn bolt_with(
    game: &mut GameState,
    dp: &dyn mtgsim::ui::decision::DecisionProvider,
    source: ObjectId,
    target: DamageTarget,
    amount: u64,
) {
    let ctx = ActionContext::new(dp);
    game.execute_action(
        GameAction::DealDamage { source, target, amount, is_combat: false, unpreventable: false },
        &ctx,
    )
    .unwrap();
}

fn bolt(game: &mut GameState, source: ObjectId, target: DamageTarget, amount: u64) {
    bolt_with(game, &test_dp(), source, target, amount);
}

fn life(game: &GameState, player: PlayerId) -> i64 {
    game.players[player].life_total
}

fn marked(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].damage_marked
}

fn shield_counters(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield[&id].counter_count(CounterType::Shield)
}

/// Every row's CR 615.7 count, in registry order.
fn counts(game: &GameState) -> Vec<u64> {
    game.replacement_effects
        .iter()
        .filter_map(|r| match r.def.uses {
            Uses::NextDamage(n) => Some(n),
            _ => None,
        })
        .collect()
}

/// Counters placed the way the engine places them, through the chokepoint.
fn add_counters(game: &mut GameState, id: ObjectId, counter: CounterType, n: u32) {
    game.execute_action(GameAction::AddCounters { object: id, counter, n }, &test_ctx())
        .unwrap();
}

/// `(source, amount)` of every damage dealt since `from`.
fn damage_since(game: &GameState, from: usize) -> Vec<(ObjectId, u64)> {
    game.events
        .events()
        .skip(from)
        .filter_map(|e| match e {
            GameEvent::DamageDealt { source_id, amount, .. } => Some((*source_id, *amount)),
            _ => None,
        })
        .collect()
}

/// A registry row inserted directly — for the fixture shapes whose printed
/// consumer is a later PR, each named in its test.
fn fixture_row(
    game: &mut GameState,
    source: ObjectId,
    controller: PlayerId,
    def: ReplacementDef,
) {
    let turn = game.turn_number;
    game.replacement_effects.add(RegisteredReplacementEffect {
        id: 0,
        source,
        controller,
        duration: Duration::UntilEndOfTurn,
        created_on_turn: turn,
        targets: Vec::new(),
        def,
    });
}

/// Attackers of player 0 attacking player 1, unblocked.
fn attackers_into(game: &mut GameState, powers: &[i32]) -> Vec<ObjectId> {
    game.active_player = 0;
    powers
        .iter()
        .map(|&p| {
            let id = place_vanilla_creature(game, 0, p, p, &[]);
            set_attacking(game, id, 1);
            id
        })
        .collect()
}

// ---------------------------------------------------------------------------
// The loop's unit — decisions are per (batch, subject), rewrites per member
// ---------------------------------------------------------------------------

/// §11 item 15's board, and the regression for the whole PR. Two blockers
/// deal combat damage to one attacker with two shield counters in **one**
/// batch — CR 510.2's simultaneous combat damage — so CR 122.1c's single
/// prevention effect applies once to the pair: both damages are prevented and
/// **one** counter is removed (the SNC release notes' ruling: "that damage is
/// prevented and only one shield counter is removed").
///
/// Per member, which is what the loop was until RD-2, the second member's
/// loop re-gathers the counter's effect with a fresh applied set and spends a
/// second counter.
#[test]
fn two_shield_counters_under_two_blockers_lose_one_counter_and_take_no_damage() {
    let mut game = setup_two_player_game();
    game.active_player = 0;
    let attacker = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
    add_counters(&mut game, attacker, CounterType::Shield, 2);
    let first = place_vanilla_creature(&mut game, 1, 2, 2, &[]);
    let second = place_vanilla_creature(&mut game, 1, 2, 2, &[]);
    set_attacking(&mut game, attacker, 1);
    set_blocked_by(&mut game, attacker, vec![first, second]);
    set_blocking(&mut game, first, vec![attacker]);
    set_blocking(&mut game, second, vec![attacker]);

    let dp = ScriptedDecisionProvider::new();
    // The attacker divides its 3 among two blockers (CR 510.1c) — the one
    // prompt this board has, and it is the attacking player's.
    dp.expect_allocation(ChoiceKind::AssignCombatDamage { attacker_id: attacker }, vec![2, 1]);
    game.process_combat_damage(&dp, false).unwrap();

    assert_eq!(marked(&game, attacker), 0, "both damages prevented");
    assert_eq!(shield_counters(&game, attacker), 1, "and ONE counter spent — CR 122.1c");
    assert_eq!(marked(&game, first), 2);
    assert_eq!(marked(&game, second), 1);
}

/// The same board with one blocker a first striker, and the contrast that
/// proves the key is the **batch** and not the turn: CR 510.4 makes two combat
/// damage steps, so the two damages are two batches, two groups, and **two**
/// counters. Knight of Meadowgrain is in the pool, so the fuzz harness builds
/// this board on its own.
#[test]
fn two_shield_counters_under_a_first_striker_and_a_regular_blocker_lose_two() {
    let mut game = setup_two_player_game();
    game.active_player = 0;
    let attacker = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
    add_counters(&mut game, attacker, CounterType::Shield, 2);
    let regular = place_vanilla_creature(&mut game, 1, 2, 2, &[]);
    let knight = put_on_battlefield(&mut game, knight_of_meadowgrain(), 1);
    set_attacking(&mut game, attacker, 1);
    set_blocked_by(&mut game, attacker, vec![regular, knight]);
    set_blocking(&mut game, regular, vec![attacker]);
    set_blocking(&mut game, knight, vec![attacker]);

    let dp = ScriptedDecisionProvider::new();
    // First-strike step: only the Knight deals damage. One member, one batch.
    game.process_combat_damage(&dp, true).unwrap();
    assert_eq!(shield_counters(&game, attacker), 1, "the first step spent one");
    assert_eq!(marked(&game, attacker), 0);

    // Regular step: the other blocker and the attacker. The attacker's split
    // is the step's one prompt; the Knight has dealt its damage (CR 510.4).
    dp.expect_allocation(ChoiceKind::AssignCombatDamage { attacker_id: attacker }, vec![2, 1]);
    game.process_combat_damage(&dp, false).unwrap();
    assert_eq!(shield_counters(&game, attacker), 0, "and the second step spent the other");
    assert_eq!(marked(&game, attacker), 0);
    assert_eq!(marked(&game, regular), 2);
    assert_eq!(marked(&game, knight), 1);
}

/// A rider is queued **once per group**, with the members' amounts summed:
/// Angel of Suffering's "prevent that damage and mill twice that many cards"
/// against two attackers is one prevention of 5 and one mill of 10.
///
/// What the log can and cannot show: a rider's moves join the batch the
/// damage was in (CR 615.5's "immediately afterward" is inside the event), so
/// one rider milling 10 and two milling 4 and 6 leave the same ten records
/// under one id. This pins the *sum* — a group rider that read only its first
/// member would mill 4 — and the two-shield-counter board above is where the
/// rider count itself is observable.
#[test]
fn a_rider_on_a_group_reads_the_members_amounts_summed() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, angel_of_suffering(), 1);
    fill_library(&mut game, 1, 20);
    attackers_into(&mut game, &[2, 3]);
    let before = game.events.len();

    game.process_combat_damage(&test_dp(), false).unwrap();

    assert_eq!(life(&game, 1), 20, "both prevented");
    let mills: Vec<Option<BatchId>> = game
        .events
        .records_from(before)
        .iter()
        .filter(|r| matches!(
            r.event,
            GameEvent::ZoneChange { cause: ZoneChangeCause::Milled, .. }
        ))
        .map(|r| r.batch())
        .collect();
    assert_eq!(mills.len(), 10, "twice that many, where that many is 2 + 3");
    assert!(mills.iter().all(|b| b.is_some() && *b == mills[0]), "inside the damage's batch: {mills:?}");
}

/// A member-uniform prevention applies to each member separately — CR 615.10's
/// "separately to each of those events that would happen at the same time".
/// A static "prevent 1 of that damage" over two simultaneous sources reduces
/// each by 1. Guardian Seraph and Daunting Defender are RD-3's, because they
/// need the source-side predicate the pattern does not carry yet; the row here
/// is that shape without it.
#[test]
fn a_static_prevent_one_reduces_each_simultaneous_source_separately_guardian_seraph_is_rd_3s() {
    let mut game = setup_two_player_game();
    let seraph = place_vanilla_creature(&mut game, 1, 1, 1, &[]);
    fixture_row(
        &mut game,
        seraph,
        1,
        ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::NO_OBJECTS,
            Rewrite::Amount(AmountRewrite::PreventUpTo(1)),
        )
        .affecting_players(PlayerSet::You),
    );
    let attackers = attackers_into(&mut game, &[2, 4]);
    let before = game.events.len();

    game.process_combat_damage(&test_dp(), false).unwrap();

    assert_eq!(life(&game, 1), 16, "1 and 3");
    assert_eq!(damage_since(&game, before), vec![(attackers[0], 1), (attackers[1], 3)]);
}

// ---------------------------------------------------------------------------
// CR 615.7 — the count, and its allocation
// ---------------------------------------------------------------------------

/// > 615.7 … Each 1 damage that would be dealt to the shielded permanent or
/// > player is prevented. Preventing 1 damage reduces the remaining shield
/// > by 1. … Once the shield has been reduced to 0, any remaining damage is
/// > dealt normally.
///
/// The atom's board with Mending Hands' number: a 4 against 5 prevents 4,
/// deals 1, and the row is gone at zero (CR 615.3's "until they're used up").
// COVERS: ATOM-615.7-001
#[test]
fn a_next_damage_count_depletes_per_point_and_the_rest_is_dealt() {
    let mut game = setup_two_player_game();
    let source = source_for(&mut game, 0);
    mending_hands_on(&mut game, 1, ResolvedTarget::Player(1));

    bolt(&mut game, source, DamageTarget::Player(1), 5);

    assert_eq!(life(&game, 1), 19);
    assert!(game.replacement_effects.is_empty(), "used up");
}

/// The count carries across events — "the next 4 damage", not "the next
/// damage event": 3 then 3 is 0 dealt, then 2, and the row goes when the last
/// point is spent.
#[test]
fn a_next_damage_count_depletes_across_events() {
    let mut game = setup_two_player_game();
    let source = source_for(&mut game, 0);
    mending_hands_on(&mut game, 1, ResolvedTarget::Player(1));

    bolt(&mut game, source, DamageTarget::Player(1), 3);
    assert_eq!(life(&game, 1), 20);
    assert_eq!(counts(&game), vec![1], "spent by exactly what it prevented");

    bolt(&mut game, source, DamageTarget::Player(1), 3);
    assert_eq!(life(&game, 1), 18);
    assert!(game.replacement_effects.is_empty());
}

/// CR 615.3's other end: an unspent count expires with its duration at the
/// cleanup step, and neither end knows about the other.
#[test]
fn an_unspent_count_expires_at_cleanup() {
    let mut game = setup_two_player_game();
    mending_hands_on(&mut game, 1, ResolvedTarget::Player(1));
    assert_eq!(counts(&game), vec![4]);
    mtgsim::test_support::pass_turn(&mut game);
    assert!(game.replacement_effects.is_empty(), "this turn is over");
}

/// > 615.7 … If damage would be dealt to the shielded permanent or player by
/// > two or more applicable sources at the same time, the player or the
/// > controller of the permanent chooses which damage the shield prevents.
///
/// The atom's board: a count of 3 under sources of 2 and 4, one `allocate`
/// prompt for the shielded player, and every legal answer prevents the same
/// total. Mending Hands is brought to 3 by a 1-damage ping first, which also
/// shows the count decremented in place.
// COVERS: ATOM-615.7-002
#[test]
fn two_attackers_into_a_shielded_player_prompt_one_allocation_and_any_answer_prevents_three() {
    for (allocation, expected) in [(vec![2, 1], vec![3]), (vec![0, 3], vec![2, 1])] {
        let mut game = setup_two_player_game();
        let pinger = source_for(&mut game, 0);
        let spell = mending_hands_on(&mut game, 1, ResolvedTarget::Player(1));
        bolt(&mut game, pinger, DamageTarget::Player(1), 1);
        assert_eq!(counts(&game), vec![3]);
        let attackers = attackers_into(&mut game, &[2, 4]);
        let before = game.events.len();

        let dp = ScriptedDecisionProvider::new();
        // Buckets are the sources in batch order; the total is min(3, 6) = 3.
        dp.expect_allocation(
            ChoiceKind::AllocateNextDamage { source: spell, remaining: 3 },
            allocation.clone(),
        );
        game.process_combat_damage(&dp, false).unwrap();

        assert_eq!(life(&game, 1), 17, "6 offered, 3 prevented, for {allocation:?}");
        assert!(game.replacement_effects.is_empty(), "used up");
        assert!(dp.is_empty(), "one prompt, no CR 616.1 prompt for one candidate");
        let dealt: Vec<u64> = damage_since(&game, before).iter().map(|(_, n)| *n).collect();
        assert_eq!(dealt, expected, "for {allocation:?}");
        // A member given nothing is dealt in full, by its own source.
        if allocation[0] == 0 {
            assert_eq!(damage_since(&game, before)[0].0, attackers[0]);
        }
    }
}

/// Never with one source: 615.7's choice exists only among "two or more", so
/// one attacker into a count of 4 is prevented unasked. A scripted provider
/// with nothing queued is the assertion.
#[test]
fn a_single_source_is_prevented_without_an_allocation_prompt() {
    let mut game = setup_two_player_game();
    mending_hands_on(&mut game, 1, ResolvedTarget::Player(1));
    attackers_into(&mut game, &[3]);
    game.process_combat_damage(&ScriptedDecisionProvider::new(), false).unwrap();
    assert_eq!(life(&game, 1), 20);
    assert_eq!(counts(&game), vec![1]);
}

/// The allocation is per **instance**, across every member it applies to,
/// whatever their subjects: a count over "you and permanents you control"
/// facing damage to you and to your blocker at once is one prompt with both
/// sources as buckets, asked in the first group's loop and read by the
/// second's. Divine Deflection is this row's printed shape and waits only on
/// `AmountExpr::Variable`; Harm's Way's 2 is RD-5's.
#[test]
fn a_count_spanning_you_and_your_permanents_is_allocated_once_divine_deflection_waits_on_variable() {
    let mut game = setup_two_player_game();
    game.active_player = 0;
    let deflection = source_for(&mut game, 1);
    fixture_row(
        &mut game,
        deflection,
        1,
        ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::Filter { filter: ObjectFilter::ByController(PlayerRef::You) },
            Rewrite::Amount(AmountRewrite::PreventRemaining),
        )
        .affecting_players(PlayerSet::You)
        .next_damage(3),
    );
    // Attacker A (2) unblocked into player 1; attacker B (4) blocked by
    // player 1's 5/5, which deals 5 back — three members, three subjects.
    let a = place_vanilla_creature(&mut game, 0, 2, 2, &[]);
    let b = place_vanilla_creature(&mut game, 0, 4, 4, &[]);
    let wall = place_vanilla_creature(&mut game, 1, 5, 5, &[]);
    set_attacking(&mut game, a, 1);
    set_attacking(&mut game, b, 1);
    set_blocked_by(&mut game, b, vec![wall]);
    set_blocking(&mut game, wall, vec![b]);

    let dp = ScriptedDecisionProvider::new();
    // Buckets: A's 2 to you, B's 4 to the wall — in batch order. Total 3.
    dp.expect_allocation(
        ChoiceKind::AllocateNextDamage { source: deflection, remaining: 3 },
        vec![1, 2],
    );
    game.process_combat_damage(&dp, false).unwrap();

    assert_eq!(life(&game, 1), 19, "2 to you, 1 prevented");
    assert_eq!(marked(&game, wall), 2, "4 to the wall, 2 prevented");
    assert_eq!(marked(&game, b), 5, "the wall's own damage is nobody's business");
    assert!(game.replacement_effects.is_empty(), "3 spent across two groups");
    assert!(dp.is_empty(), "asked once");
}

// ---------------------------------------------------------------------------
// CR 615.4, 615.5, 615.6, 615.11
// ---------------------------------------------------------------------------

/// > 615.4 Prevention effects apply only to events that would happen, so a
/// > prevention effect must exist before the damage event it prevents.
///
/// The CR's own example from the other side: Mending Hands after the bolt
/// prevents nothing of it, and all of the next.
// COVERS: ATOM-615.4-001
#[test]
fn a_shield_made_after_the_damage_prevents_none_of_it() {
    let mut game = setup_two_player_game();
    let source = source_for(&mut game, 0);

    bolt(&mut game, source, DamageTarget::Player(1), 3);
    assert_eq!(life(&game, 1), 17);
    mending_hands_on(&mut game, 1, ResolvedTarget::Player(1));
    assert_eq!(life(&game, 1), 17, "nothing to prevent retroactively");
    assert_eq!(counts(&game), vec![4], "and nothing spent");

    bolt(&mut game, source, DamageTarget::Player(1), 2);
    assert_eq!(life(&game, 1), 17, "the shield was there for this one");
    assert_eq!(counts(&game), vec![2]);
}

/// > 615.5 Some prevention effects also include an additional effect, which
/// > may refer to the amount of damage that was prevented. The prevention
/// > takes place at the time the original event would have happened; the rest
/// > of the effect takes place immediately afterward.
///
/// The atom verbatim, on a fixture: "Prevent the next 3 damage that would be
/// dealt to you. If damage is prevented this way, you gain that much life."
/// Reverse Damage is the printed reader of `AmountExpr::DamagePrevented` and
/// is RD-3's, since it chooses a source; the channel is tested here so RD-3
/// inherits it working.
// COVERS: ATOM-615.5-001
#[test]
fn a_fixture_rider_reads_the_prevented_amount_until_reverse_damage_lands_in_rd_3() {
    let mut game = setup_two_player_game();
    let source = source_for(&mut game, 0);
    let spell = put_in_hand(&mut game, mending_hands(), 1);
    let effect = Effect::Atom(
        Primitive::CreateReplacement(
            Box::new(
                ReplacementDef::new(
                    EventPattern::DealDamage { source: None, combat: None },
                    AffectedSet::NO_OBJECTS,
                    Rewrite::Amount(AmountRewrite::PreventRemaining),
                )
                .next_damage(3)
                .with_then(Effect::Atom(
                    Primitive::GainLife(AmountExpr::DamagePrevented),
                    EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
                )),
            ),
            Duration::UntilEndOfTurn,
            PatternFill::Authored,
        ),
        EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
    );
    let ctx = ResolutionContext {
        source: spell,
        ability_source: None,
        controller: 1,
        targets: vec![ResolvedTarget::Player(1)],
        replaced_amount: None,
        damage_prevented: None,
    };
    game.resolve_effect(&effect, &ctx, &test_dp()).unwrap();
    let before = game.events.len();

    bolt(&mut game, source, DamageTarget::Player(1), 5);

    assert_eq!(life(&game, 1), 21, "20 − 2 + 3");
    assert!(game.replacement_effects.is_empty(), "the count is at 0");
    // Prevention first, the rest immediately afterward: the damage is dealt
    // and its life loss taken before the gain.
    let order: Vec<String> = game
        .events
        .events()
        .skip(before)
        .filter_map(|e| match e {
            GameEvent::DamageDealt { amount, .. } => Some(format!("dealt {amount}")),
            GameEvent::LifeChanged { old, new, .. } => Some(format!("life {old}->{new}")),
            _ => None,
        })
        .collect();
    assert_eq!(order, vec!["dealt 2", "life 20->18", "life 18->21"]);
}

/// > 609.7b … If for any reason the shield prevents no damage or replaces no
/// > damage, the shield isn't used up.
///
/// RD decision 7: a use is spent by what an application did. A "prevent half
/// that damage, rounded down" that fires once — Dark Sphere's shape, RD-3's
/// card — chosen against 1 damage prevents 0, so it is not used up and its
/// "you gain that much life" rider gains 0, which is no event at all
/// (CR 119.10). The same row against 5 prevents 2 and is gone.
#[test]
fn a_once_prevention_that_prevents_nothing_is_not_used_up_dark_sphere_is_rd_3s() {
    let mut game = setup_two_player_game();
    let source = source_for(&mut game, 0);
    let sphere = source_for(&mut game, 1);
    fixture_row(
        &mut game,
        sphere,
        1,
        ReplacementDef::new(
            EventPattern::DealDamage { source: None, combat: None },
            AffectedSet::NO_OBJECTS,
            Rewrite::Amount(AmountRewrite::PreventHalf(Rounding::Down)),
        )
        .affecting_players(PlayerSet::You)
        .once()
        .with_then(Effect::Atom(
            Primitive::GainLife(AmountExpr::DamagePrevented),
            EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
        )),
    );
    let before = game.events.len();

    bolt(&mut game, source, DamageTarget::Player(1), 1);
    assert_eq!(life(&game, 1), 19, "half of 1 rounded down is 0");
    assert_eq!(game.replacement_effects.len(), 1, "chosen, did nothing, not used up");
    let gains = game
        .events
        .events()
        .skip(before)
        .filter(|e| matches!(e, GameEvent::LifeChanged { old, new, .. } if new > old))
        .count();
    assert_eq!(gains, 0, "a rider that gains 0 life is a non-event (CR 119.10)");

    bolt(&mut game, source, DamageTarget::Player(1), 5);
    assert_eq!(life(&game, 1), 19 - 3 + 2, "prevents 2 of 5, gains 2");
    assert!(game.replacement_effects.is_empty(), "and now it is used up");
}

/// > 615.6 If damage that would be dealt is prevented, it never happens.
///
/// Safe Passage against a bolt to its caster: no `DamageDealt`, no life
/// change, nothing in the log. The atom's other half — "a 'whenever damage is
/// dealt' trigger does not fire" — is critical-path item 6's, when triggers
/// exist to not fire.
// COVERS-PARTIAL: ATOM-615.6-001
#[test]
fn prevented_damage_never_happens_and_the_trigger_half_is_item_6s() {
    let mut game = setup_two_player_game();
    let source = source_for(&mut game, 1);
    resolve_spell(&mut game, safe_passage(), 0, Vec::new());
    let before = game.events.len();

    bolt(&mut game, source, DamageTarget::Player(0), 3);

    assert_eq!(life(&game, 0), 20);
    assert_eq!(game.events.len(), before, "nothing was announced");
}

/// > 615.11 Some effects create a prevention shield for each applicable
/// > creature when the spell or ability that creates the effect resolves.
///
/// Samite Censer-Bearer, activated for real: `{W}` from a Plains through the
/// mana window, itself sacrificed as the cost. Three creatures at resolution
/// are three rows of `NextDamage(1)`; a fourth entering afterwards has none,
/// so 2 damage to each marks 1, 1, 1 and 2 — and each row is used up by its
/// own creature's damage.
// COVERS: ATOM-615.11-001
#[test]
fn samite_censer_bearer_makes_a_separate_count_on_each_creature_at_resolution() {
    let mut game = setup_two_player_game();
    let source = source_for(&mut game, 1);
    put_land_on_battlefield(&mut game, plains, 0);
    let bearer = put_on_battlefield(&mut game, samite_censer_bearer(), 0);
    let originals: Vec<ObjectId> =
        (0..3).map(|_| place_vanilla_creature(&mut game, 0, 3, 3, &[])).collect();

    // The Recording provider taps the Plains when the window offers it; the
    // stop declines the window once {W} is covered (CR 605.3a).
    let dp = ManaWindowStop::new(RecordingDecisionProvider::picking(0));
    game.activate_ability(0, bearer, 0, &dp).expect("{W} and itself are payable");
    assert!(game.players[0].graveyard.contains(&bearer), "sacrificed as the cost");
    game.resolve_top_of_stack(&dp).unwrap();

    let rows: Vec<&RegisteredReplacementEffect> = game.replacement_effects.iter().collect();
    assert_eq!(rows.len(), 3, "one per creature, and none for the Censer-Bearer");
    for (row, creature) in rows.iter().zip(&originals) {
        assert_eq!(row.def.affected, AffectedSet::Fixed(vec![*creature]));
        assert_eq!(row.def.uses, Uses::NextDamage(1));
        assert_eq!(row.source, bearer, "CR 113.7a");
    }

    let latecomer = place_vanilla_creature(&mut game, 0, 3, 3, &[]);
    for &creature in originals.iter().chain(std::iter::once(&latecomer)) {
        bolt(&mut game, source, DamageTarget::Object(creature), 2);
    }
    assert_eq!(originals.iter().map(|&c| marked(&game, c)).collect::<Vec<_>>(), vec![1, 1, 1]);
    assert_eq!(marked(&game, latecomer), 2, "entered after resolution — no shield");
    assert!(game.replacement_effects.is_empty(), "each count used up by its own creature");
}

// ---------------------------------------------------------------------------
// CR 616.1 — a count beside another effect on one event
// ---------------------------------------------------------------------------

/// Furnace of Rath's printed ruling, at last with a printed prevention to
/// order against: "if the damage would be prevented, you can choose whether
/// to prevent 4 and then double the remaining 1, or double to 10 and then
/// prevent 4". The damaged player chooses (CR 616.1), each application reads
/// the amount the other left, and the count is spent by 4 either way.
#[test]
fn mending_hands_beside_furnace_prevents_then_doubles_or_doubles_then_prevents() {
    // `gather` reads the battlefield sweep before the registry: index 0 is
    // Furnace of Rath, index 1 is Mending Hands.
    for (pick, expected_life) in [(1usize, 20 - 2), (0usize, 20 - 6)] {
        let mut game = setup_two_player_game();
        let source = source_for(&mut game, 0);
        put_on_battlefield(&mut game, furnace_of_rath(), 0);
        mending_hands_on(&mut game, 1, ResolvedTarget::Player(1));

        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![pick]);
        bolt_with(&mut game, &dp, source, DamageTarget::Player(1), 5);

        assert_eq!(life(&game, 1), expected_life, "picking index {pick} first");
        assert!(game.replacement_effects.is_empty(), "4 spent either way");
    }
}

/// The board on which "nothing was consumed" is the whole answer: Safe
/// Passage beside Mending Hands. Chosen first, Safe Passage empties the event
/// and the count is never chosen, so it keeps its 4; chosen first, Mending
/// Hands prevents the 3 and leaves a 0 CR 614.7a drops before Safe Passage is
/// gathered again. Both are CR 616.1's choice showing through, and neither
/// spends anything it did not prevent.
#[test]
fn safe_passage_beside_mending_hands_spends_the_count_only_when_it_prevented() {
    // Registration order: Safe Passage's row first, then Mending Hands'.
    for (pick, expected_counts) in [(0usize, vec![4]), (1usize, vec![1])] {
        let mut game = setup_two_player_game();
        let source = source_for(&mut game, 0);
        resolve_spell(&mut game, safe_passage(), 1, Vec::new());
        mending_hands_on(&mut game, 1, ResolvedTarget::Player(1));

        let dp = ScriptedDecisionProvider::new();
        dp.expect_pick_n(ChoiceKind::ChooseReplacementEffect { affected_object: None }, vec![pick]);
        bolt_with(&mut game, &dp, source, DamageTarget::Player(1), 3);

        assert_eq!(life(&game, 1), 20, "picking index {pick} first");
        assert_eq!(counts(&game), expected_counts, "picking index {pick} first");
        assert!(dp.is_empty(), "one prompt: the second effect never gets a second question");
    }
}
