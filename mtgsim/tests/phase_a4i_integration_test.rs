//! A4i integration tests: several instances of the word "target" on one spell
//! (CR 115.3, 601.2c, 608.2b; `backlog.md` §2.20).
//!
//! **The boards are chosen so the instance model is what answers, not the
//! filters.** A spell whose two clauses read "target artifact" and "target
//! land" would pass under the old one-recipient model too, as long as the one
//! recipient happened to be the right one; what it could not do is announce
//! twice, keep the two answers apart, and let one of them go illegal without
//! the other. So the tests here press on exactly those three:
//!
//! - **Announce twice.** Seeds of Strength prints "target creature" three
//!   times with identical criteria, so nothing but the instance count can tell
//!   its three clauses from Ensoul Artifact's one.
//! - **Keep the answers apart.** Plague Spores names one artifact creature land
//!   for both clauses (CR 601.2c's own example), and Incremental Growth's
//!   "another" forbids exactly that.
//! - **Let one go illegal.** CR 608.2b is two rules — the spell resolves unless
//!   *every* target is illegal, and an illegal target is simply not affected —
//!   and the one-recipient model could only ask the first.
//!
//! Casting goes through `cast_spell` rather than a staged `StackEntry`
//! wherever the announcement is the claim: a fixture that writes the instances
//! by hand would prove the resolution reads them and nothing about whether
//! CR 601.2c filled them in.

use mtgsim::cards::creatures;
use mtgsim::cards::phase_a4i_cards::{
    incremental_growth, jagged_lightning, plague_spores, seat_of_the_synod, seeds_of_strength,
};
use mtgsim::cards::phase_ld_cards::ensoul_artifact_spell;
use mtgsim::cards::phase_re_cards::{skullcrack, vorinclex_monstrous_raider};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::oracle::characteristics::{get_effective_power, get_effective_toughness};
use mtgsim::oracle::legality::candidate_priority_actions;
use mtgsim::state::game_state::GameState;
use mtgsim::engine::actions::ZoneChangeCause;
use mtgsim::test_support::{
    put_in_hand, put_on_battlefield, setup_two_player_game, test_ctx, test_dp, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    ColorChange, CounterType, Duration, Effect, EffectRecipient, ObjectFilter, Primitive,
    SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::mana::ManaType;
use mtgsim::types::zones::Zone;
use mtgsim::ui::choice_types::ChoiceKind;
use mtgsim::ui::decision::{PriorityAction, ScriptedDecisionProvider};
use mtgsim::ui::mana_window_stop::ManaWindowStop;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Exactly one spell's cost, in the pool.
///
/// **Exactly, and that is the point.** A pool with spare colours makes the
/// generic component a real CR 601.2h choice and every test in this file would
/// have to script an allocation it is not about. With the cost's own mana and
/// nothing else the split is forced (CR 102.2), so the only prompts these tests
/// see are CR 601.2c's.
fn pay_for(game: &mut GameState, player: PlayerId, generic: u64, colored: &[(ManaType, u64)]) {
    if generic > 0 {
        game.players[player].mana_pool.add(ManaType::Colorless, generic);
    }
    for (mana, n) in colored {
        game.players[player].mana_pool.add(*mana, *n);
    }
}

fn pay_for_seeds(game: &mut GameState, player: PlayerId) {
    pay_for(game, player, 0, &[(ManaType::Green, 1), (ManaType::White, 1)]);
}

fn pay_for_growth(game: &mut GameState, player: PlayerId) {
    pay_for(game, player, 3, &[(ManaType::Green, 2)]);
}

fn pay_for_lightning(game: &mut GameState, player: PlayerId) {
    pay_for(game, player, 3, &[(ManaType::Red, 2)]);
}

fn pay_for_spores(game: &mut GameState, player: PlayerId) {
    pay_for(game, player, 4, &[(ManaType::Black, 1), (ManaType::Red, 1)]);
}

/// A scripted provider that declines the CR 601.2g window once the cost is
/// covered, as every shipped client does.
///
/// Without it a board holding an untapped land — which Plague Spores' board
/// must, since its target *is* a land — offers the window again after the
/// payment is covered, and the test would have to script a decline it is not
/// about (`codebase-state.md` item 83).
fn stopping_dp() -> ManaWindowStop<ScriptedDecisionProvider> {
    ManaWindowStop::new(test_dp())
}

/// Cast from hand and resolve, through `cast_spell` — so CR 601.2c's
/// announcement loop is what fills the instances.
fn cast_and_resolve(
    game: &mut GameState,
    player: PlayerId,
    card: ObjectId,
    dp: &ManaWindowStop<ScriptedDecisionProvider>,
) {
    game.cast_spell(player, card, dp).expect("castable");
    game.resolve_top_of_stack(dp).expect("resolves");
}

/// Remove a permanent from the battlefield the way a response would — through
/// the chokepoint, with a cause, so CR 614 sees it.
fn exile(game: &mut GameState, id: ObjectId) {
    game.change_zone(id, Zone::Exile, ZoneChangeCause::Exiled, &test_ctx())
        .expect("exiling a permanent");
}

/// `(power, toughness)` as the layer system computes it.
fn pt(game: &GameState, id: ObjectId) -> (i32, i32) {
    (
        get_effective_power(game, id).expect("has power"),
        get_effective_toughness(game, id).expect("has toughness"),
    )
}

fn counters(game: &GameState, id: ObjectId) -> u32 {
    game.battlefield
        .get(&id)
        .map(|e| e.counter_count(CounterType::PlusOnePlusOne))
        .unwrap_or(0)
}

/// Answer the next `SelectRecipients` prompt with these option indices.
fn answer_targets(
    dp: &ManaWindowStop<ScriptedDecisionProvider>,
    recipient: EffectRecipient,
    spell: ObjectId,
    picks: Vec<usize>,
) {
    dp.inner()
        .expect_pick_n(ChoiceKind::SelectRecipients { recipient, spell_id: spell }, picks);
}

fn target_creature() -> EffectRecipient {
    EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1))
}

/// Incremental Growth's later clauses, as the card writes them: a creature,
/// and not one an earlier instance took.
fn another_target_creature(earlier_targets: &[usize]) -> EffectRecipient {
    let mut filter = ObjectFilter::ByType(CardType::Creature);
    for &ix in earlier_targets {
        filter = ObjectFilter::And(
            Box::new(filter),
            Box::new(ObjectFilter::OtherThanInstance(ix)),
        );
    }
    EffectRecipient::Target(SelectionFilter::Permanent(filter), TargetCount::Exactly(1))
}

/// `Diagnostics::decisions` is cumulative over the game, so a test that
/// staged a board by casting something reads the delta rather than the total.
fn decisions_during(game: &mut GameState, f: impl FnOnce(&mut GameState)) -> u64 {
    let before = game.diagnostics.decisions();
    f(game);
    game.diagnostics.decisions() - before
}

/// Seat of the Synod as a 5/5 artifact **creature** land — the one permanent
/// that satisfies "target nonblack creature" and "target land" at once, which
/// is CR 601.2c's own worked example and ATOM-608.2b-002's board.
///
/// Ensoul Artifact rather than March of the Machines, which would make it a
/// 0/0 and hand it to CR 704.5a before any spell could target it.
fn animated_artifact_land(game: &mut GameState, player: PlayerId) -> ObjectId {
    let land = put_on_battlefield(game, seat_of_the_synod(), player);
    let ensoul = put_in_hand(game, ensoul_artifact_spell(), player);
    pay_for(game, player, 1, &[(ManaType::Blue, 1)]);
    let dp = stopping_dp();
    cast_and_resolve(game, player, ensoul, &dp);
    land
}

/// Make a permanent black until end of turn — the step ATOM-608.2b-002 needs
/// between the announcement and the resolution. A fixture rather than a card:
/// Moonlace, the pool's colour-changer, removes colours rather than setting
/// one.
fn make_black(game: &mut GameState, id: ObjectId, controller: PlayerId) {
    let ctx = ResolutionContext {
        source: id,
        ability_source: None,
        controller,
        targets: ChosenTargets::one(vec![ResolvedTarget::Object(id)]),
        replaced_amount: None,
        damage_prevented: None,
    };
    let effect = Effect::Atom(
        Primitive::ChangeColor(
            ColorChange::Set(vec![Color::Black].into_iter().collect()),
            Duration::UntilEndOfTurn,
        ),
        EffectRecipient::Target(
            SelectionFilter::Permanent(ObjectFilter::All),
            TargetCount::Exactly(1),
        ),
    );
    game.resolve_effect(&effect, &ctx, &test_dp()).expect("recolours");
}

// ---------------------------------------------------------------------------
// CR 115.3 / 601.2c — one object, once per instance
// ---------------------------------------------------------------------------

// COVERS: ATOM-115.3-001
// RULING: Seeds of Strength #1 - "You may choose the same creature as a target
// multiple times since the card says 'target creature' multiple times ... or a
// single creature +3/+3."
/// One creature on the board and three instances of "target": every clause
/// names it, and the pumps stack.
///
/// **Zero prompts, and that is the assertion that matters most.** A fixed count
/// of one with one legal choice is not a choice (CR 102.2), so each of the
/// three announcements is forced. Before this phase the spell had one recipient
/// and one prompt; the difference between "one clause" and "three clauses all
/// forced to the same creature" is invisible in the prompt log and entirely
/// visible in the creature's power.
#[test]
fn seeds_of_strength_can_name_one_creature_for_all_three_instances() {
    let mut game = setup_two_player_game();
    let bear = put_on_battlefield(&mut game, creatures::grizzly_bears(), 0);
    let seeds = put_in_hand(&mut game, seeds_of_strength(), 0);
    pay_for_seeds(&mut game, 0);

    let dp = stopping_dp();
    cast_and_resolve(&mut game, 0, seeds, &dp);

    assert_eq!(pt(&game, bear), (5, 5), "2/2 plus +1/+1 three times");
    assert_eq!(game.diagnostics.decisions(), 0, "one legal choice for a fixed count is forced");
}

// COVERS: ATOM-115.3-001
// RULING: Seeds of Strength #1 - "You may give three different creatures +1/+1
// each, one creature +2/+2 and another creature +1/+1 ..."
/// The ruling's other two distributions, which need the instances to be
/// genuinely separate answers rather than one answer read three times.
#[test]
fn seeds_of_strength_can_split_its_three_instances_across_creatures() {
    // Three different creatures, +1/+1 each.
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let c = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let seeds = put_in_hand(&mut game, seeds_of_strength(), 0);
    pay_for_seeds(&mut game, 0);

    let dp = stopping_dp();
    // Candidates are offered in `battlefield_ids_ordered` (CR 613.7 timestamp)
    // order, so index 0/1/2 is a/b/c.
    for pick in [0usize, 1, 2] {
        answer_targets(&dp, target_creature(), seeds, vec![pick]);
    }
    cast_and_resolve(&mut game, 0, seeds, &dp);

    assert_eq!(pt(&game, a), (3, 3));
    assert_eq!(pt(&game, b), (3, 3));
    assert_eq!(pt(&game, c), (3, 3));

    // One creature +2/+2 and another +1/+1: two instances on the first, one on
    // the second.
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let seeds = put_in_hand(&mut game, seeds_of_strength(), 0);
    pay_for_seeds(&mut game, 0);

    let dp = stopping_dp();
    for pick in [0usize, 0, 1] {
        answer_targets(&dp, target_creature(), seeds, vec![pick]);
    }
    cast_and_resolve(&mut game, 0, seeds, &dp);

    assert_eq!(pt(&game, a), (4, 4), "two of the three instances");
    assert_eq!(pt(&game, b), (3, 3), "the third");
}

// COVERS: ATOM-601.2c-004
/// CR 601.2c's own worked example: *"A spell that says 'Destroy target artifact
/// and target land' ... can target the same artifact land twice because it uses
/// the word 'target' in multiple places."*
///
/// Plague Spores is the same sentence with a tighter creature clause, and an
/// animated Seat of the Synod satisfies both. One object, two instances, and it
/// is destroyed — not destroyed twice, because the second `Destroy` finds it
/// already gone, which is CR 608.2b's partial resolution on the way out.
#[test]
fn plague_spores_can_name_one_artifact_land_for_both_instances() {
    let mut game = setup_two_player_game();
    let land = animated_artifact_land(&mut game, 0);
    // A second creature, so the nonblack-creature clause is a real choice
    // rather than a forced one and the prompt log says which was taken.
    let decoy = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let spores = put_in_hand(&mut game, plague_spores(), 0);
    pay_for_spores(&mut game, 0);

    let dp = stopping_dp();
    // Instance 0, "target nonblack creature": the land, not the decoy. The
    // restriction atoms declare the clauses, so they are the two prompts.
    let nonblack_creature = EffectRecipient::Target(
        SelectionFilter::Permanent(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Creature)),
            Box::new(ObjectFilter::Not(Box::new(ObjectFilter::ByColor(Color::Black)))),
        )),
        TargetCount::Exactly(1),
    );
    let candidates: Vec<ObjectId> = game.battlefield_ids_ordered();
    let land_ix = candidates.iter().position(|&id| id == land).expect("on the battlefield");
    answer_targets(&dp, nonblack_creature, spores, vec![land_ix]);
    // Instance 1, "target land": the only land, so it is forced and unasked.

    cast_and_resolve(&mut game, 0, spores, &dp);

    assert_eq!(
        game.get_object(land).unwrap().zone,
        Zone::Graveyard,
        "chosen for both instances, destroyed once"
    );
    assert!(game.battlefield.contains_key(&decoy), "not a target of either clause");
}

// ---------------------------------------------------------------------------
// CR 601.2c, first sentence — not twice for one instance
// ---------------------------------------------------------------------------

// COVERS: ATOM-601.2c-003
// COVERS: ATOM-115.3-002
/// *"The same target can't be chosen multiple times for any one instance of the
/// word 'target'."* Jagged Lightning's "each of **two** target creatures" is one
/// instance, so its two choices must differ — where Seeds of Strength's three
/// *instances* may all be the same creature.
///
/// Asked of `validate_targets`, which is the rule, rather than only of
/// `validate_pick_n`'s distinct-index assertion, which is the provider
/// contract. CR 115.7's target-changing effects will be checked against the
/// rule and not against the contract.
#[test]
fn the_same_creature_cannot_be_chosen_twice_for_one_instance() {
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let two_creatures =
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(2));

    assert!(
        game.validate_targets(
            &two_creatures,
            &[ResolvedTarget::Object(a), ResolvedTarget::Object(b)],
            0,
            &ChosenTargets::NONE,
        )
        .is_ok(),
        "two different creatures"
    );
    assert!(
        game.validate_targets(
            &two_creatures,
            &[ResolvedTarget::Object(a), ResolvedTarget::Object(a)],
            0,
            &ChosenTargets::NONE,
        )
        .is_err(),
        "the same creature twice for one instance (CR 601.2c)"
    );

    // And the same object *is* legal once for each of two instances.
    let one_creature = target_creature();
    let mut earlier_targets = ChosenTargets::NONE;
    earlier_targets.push(vec![ResolvedTarget::Object(a)]);
    assert!(
        game.validate_targets(&one_creature, &[ResolvedTarget::Object(a)], 0, &earlier_targets)
            .is_ok(),
        "CR 601.2c's second sentence"
    );
}

/// The enumeration agrees with the rule: a board with one creature does not
/// make "each of two target creatures" castable, so the spell is never offered
/// and then rewound.
#[test]
fn jagged_lightning_needs_two_creatures_to_be_castable() {
    let mut game = setup_two_player_game();
    let lightning = put_in_hand(&mut game, jagged_lightning(), 0);
    pay_for_lightning(&mut game, 0);

    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert!(
        !candidate_priority_actions(&game, 0)
            .iter()
            .any(|a| matches!(a, PriorityAction::CastSpell(id) if *id == lightning)),
        "one creature cannot fill an instance that needs two distinct choices"
    );

    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    assert!(
        candidate_priority_actions(&game, 0)
            .iter()
            .any(|a| matches!(a, PriorityAction::CastSpell(id) if *id == lightning)),
        "and two creatures does"
    );
}

// ---------------------------------------------------------------------------
// CR 608.2b — some legal, some not
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-608.2b-005
/// *"Creature A is illegal target, but creature B is still legal. Spell
/// resolves. Creature B takes 3 damage. Creature A is unaffected."*
///
/// Partial resolution **within one instance**, which needs no second clause and
/// was wrong before this phase for exactly that reason: `any_targets_still_legal`
/// answered for the whole list, and then the damage went to every entry in it,
/// illegal ones included.
///
/// COVERS-PARTIAL because the atom makes creature A illegal by giving it
/// protection from red, and protection is not checked at targeting yet —
/// `cant-effects-architecture.md`'s RS-2. A creature that has left the
/// battlefield is the other way to be an illegal target and proves the same
/// mechanism; the protection board belongs to the phase that can build it.
#[test]
fn jagged_lightning_damages_only_the_creature_that_is_still_legal() {
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, vanilla_creature(4, 4, &[]), 1);
    let b = put_on_battlefield(&mut game, vanilla_creature(4, 4, &[]), 1);
    let lightning = put_in_hand(&mut game, jagged_lightning(), 0);
    pay_for_lightning(&mut game, 0);

    // Both announced, then A leaves before the spell resolves.
    let dp = stopping_dp();
    game.cast_spell(0, lightning, &dp).expect("two creatures, so castable");
    exile(&mut game, a);

    game.resolve_top_of_stack(&dp).expect("one legal target is enough (CR 608.2b)");

    assert_eq!(
        game.battlefield[&b].damage_marked, 3,
        "the legal target still takes its three"
    );
    assert_eq!(
        game.get_object(lightning).unwrap().zone,
        Zone::Graveyard,
        "the spell resolved rather than being countered by game rules"
    );
}

// COVERS: ATOM-608.2b-002
/// *"Destroy target nonblack creature and target land."* The same artifact
/// creature land is chosen for both, and then it becomes black: the creature
/// clause is now illegal, the land clause is not.
///
/// Both halves of CR 608.2b in one board. The spell resolves, because not every
/// target is illegal; and the destruction that does happen is the **land**
/// clause's, because the creature clause's target is not affected. Under one
/// recipient this board had one answer for both clauses and no way to give
/// them different ones.
#[test]
fn plague_spores_destroys_the_land_when_the_creature_clause_went_illegal() {
    let mut game = setup_two_player_game();
    let land = animated_artifact_land(&mut game, 0);
    let spores = put_in_hand(&mut game, plague_spores(), 0);
    pay_for_spores(&mut game, 0);

    let dp = stopping_dp();
    // Both clauses have exactly one legal choice — the animated land — so both
    // announcements are forced (CR 102.2) and neither reaches the provider.
    // The scripted provider panics on a prompt it was not given, which is that
    // assertion; the one decision counted is the CR 601.2g mana window, which
    // a board whose target *is* an untapped land cannot avoid offering.
    let asked = decisions_during(&mut game, |game| {
        game.cast_spell(0, spores, &dp).expect("castable");
    });
    assert_eq!(asked, 1, "the mana window, and no CR 601.2c prompt");

    make_black(&mut game, land, 0);

    game.resolve_top_of_stack(&dp)
        .expect("the land clause is still legal, so the spell resolves");

    assert_eq!(
        game.get_object(land).unwrap().zone,
        Zone::Graveyard,
        "\"destroy target land\" still applies; \"target nonblack creature\" does not"
    );
}

// ---------------------------------------------------------------------------
// CR 601.2c — "another target"
// ---------------------------------------------------------------------------

// RULING: Incremental Growth #1 - "You must choose three different targets in
// order to cast Incremental Growth."
/// Two creatures is not enough: the third clause is "a third target creature"
/// and there is no third.
///
/// Enforced at enumeration, not at rollback. The engine offering a cast it then
/// rewinds is `codebase-state.md` item 139's class of defect, and the
/// exclusions are static enough to be answered before the spell is proposed.
#[test]
fn incremental_growth_needs_a_third_creature_to_be_castable() {
    let mut game = setup_two_player_game();
    let growth = put_in_hand(&mut game, incremental_growth(), 0);
    pay_for_growth(&mut game, 0);

    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert!(
        !candidate_priority_actions(&game, 0)
            .iter()
            .any(|a| matches!(a, PriorityAction::CastSpell(id) if *id == growth)),
        "two creatures, three instances that must all differ"
    );

    put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    assert!(
        candidate_priority_actions(&game, 0)
            .iter()
            .any(|a| matches!(a, PriorityAction::CastSpell(id) if *id == growth)),
        "and a third — an opponent's counts, the clause says \"creature\""
    );
}

/// The exclusion is a *criterion*, so it shapes the options rather than
/// rejecting an answer: by the third clause the two creatures already taken are
/// not offered, and the counters land one, two and three.
#[test]
fn incremental_growth_counters_three_different_creatures() {
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let c = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let growth = put_in_hand(&mut game, incremental_growth(), 0);
    pay_for_growth(&mut game, 0);

    let dp = stopping_dp();
    // Instance 0 offers all three; instance 1 offers the two left; instance 2's
    // single remaining creature is forced and unasked.
    answer_targets(&dp, target_creature(), growth, vec![0]);
    answer_targets(&dp, another_target_creature(&[0]), growth, vec![0]);
    let asked = decisions_during(&mut game, |game| {
        cast_and_resolve(game, 0, growth, &dp);
    });

    assert_eq!(counters(&game, a), 1);
    assert_eq!(counters(&game, b), 2);
    assert_eq!(counters(&game, c), 3);
    assert_eq!(
        asked, 2,
        "three instances, and the last one's choice is forced by the exclusions"
    );
}

// RULING: Incremental Growth #2 - "If some of the creatures are illegal targets
// as Incremental Growth tries to resolve, the remaining legal targets still get
// the appropriate number of +1/+1 counters."
/// One of the three leaves in response, and the other two still get **their
/// own** clause's counters — two and three, not one and two shuffled up.
#[test]
fn incremental_growth_still_counters_the_creatures_that_are_left() {
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let c = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let growth = put_in_hand(&mut game, incremental_growth(), 0);
    pay_for_growth(&mut game, 0);

    let dp = stopping_dp();
    answer_targets(&dp, target_creature(), growth, vec![0]);
    answer_targets(&dp, another_target_creature(&[0]), growth, vec![0]);
    game.cast_spell(0, growth, &dp).expect("three creatures, so castable");
    exile(&mut game, a);

    game.resolve_top_of_stack(&dp).expect("two legal targets remain");

    assert_eq!(counters(&game, b), 2, "its own clause, not the first's");
    assert_eq!(counters(&game, c), 3);
}

// RULING: Incremental Growth #2 - "If all targets are illegal, Incremental
// Growth doesn't resolve."
/// Every target gone is the other half of CR 608.2b, and the spell is countered
/// by game rules.
#[test]
fn incremental_growth_does_not_resolve_with_every_creature_gone() {
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let c = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let growth = put_in_hand(&mut game, incremental_growth(), 0);
    pay_for_growth(&mut game, 0);

    let dp = stopping_dp();
    answer_targets(&dp, target_creature(), growth, vec![0]);
    answer_targets(&dp, another_target_creature(&[0]), growth, vec![0]);
    game.cast_spell(0, growth, &dp).expect("three creatures, so castable");
    for id in [a, b, c] {
        exile(&mut game, id);
    }

    game.resolve_top_of_stack(&dp).expect("fizzling is not an error");

    assert_eq!(
        game.get_object(growth).unwrap().zone,
        Zone::Graveyard,
        "countered by game rules (CR 608.2b)"
    );
    assert!(
        game.events
            .events()
            .any(|e| matches!(e, mtgsim::events::event::GameEvent::SpellFizzled { .. })),
        "and it says so"
    );
}

// ---------------------------------------------------------------------------
// What the A/B found: a registered card whose target was never asked for
// ---------------------------------------------------------------------------

/// **Skullcrack's three damage never landed when the card was cast.**
///
/// *"Players can't gain life this turn. Damage can't be prevented this turn.
/// Skullcrack deals 3 damage to target player or planeswalker."* Three atoms,
/// and the first two are `EffectRecipient::Controller`. The pre-A4i rule took a
/// `Sequence`'s **first** atom's recipient as the whole spell's, so the spell
/// announced no target at all, `chosen_targets` was empty, and the damage atom
/// read an empty list.
///
/// RE-3's Skullcrack tests did not catch it because they stage a
/// `ResolutionContext` with the target written in by hand — which proves the
/// resolution reads a target and nothing about whether CR 601.2c ever asked for
/// one. This one casts from hand.
///
/// Found by A4i's A/B: `performance` was `IDENTICAL` and `stress` was not, on
/// the same 161-card pool, and Skullcrack is registered but unpooled.
/// `codebase-state.md` carries the item. Shown to fail against `main`
/// (aafb79a): the opponent stays at 20 there, and reaches 17 here.
#[test]
fn skullcrack_cast_from_hand_deals_its_three_damage() {
    let mut game = setup_two_player_game();
    let card = put_in_hand(&mut game, skullcrack(), 0);
    pay_for(&mut game, 0, 1, &[(ManaType::Red, 1)]);

    let dp = stopping_dp();
    answer_targets(
        &dp,
        EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
        card,
        vec![1],
    );
    cast_and_resolve(&mut game, 0, card, &dp);

    assert_eq!(
        game.players[1].life_total, 17,
        "the damage clause declares an instance of its own, whatever the atoms before it say"
    );
}

// ---------------------------------------------------------------------------
// Review questions: the n-1 boundary, and a doubler per instruction
// ---------------------------------------------------------------------------

/// **Two of three illegal, which is the boundary between CR 608.2b's two rules.**
///
/// The 1-of-3 board proves "some illegal, the rest still land" and the 3-of-3
/// board proves "all illegal, it does not resolve". Neither can tell an `any`
/// from an `all` in `surviving_targets`' fizzle predicate — this one can: with
/// two gone and one left, an `all` would fizzle the spell and an `any` resolves
/// it, and only the third creature's counters say which happened.
#[test]
fn incremental_growth_resolves_on_its_last_legal_creature() {
    let mut game = setup_two_player_game();
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let c = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let growth = put_in_hand(&mut game, incremental_growth(), 0);
    pay_for_growth(&mut game, 0);

    let dp = stopping_dp();
    answer_targets(&dp, target_creature(), growth, vec![0]);
    answer_targets(&dp, another_target_creature(&[0]), growth, vec![0]);
    game.cast_spell(0, growth, &dp).expect("three creatures, so castable");
    exile(&mut game, a);
    exile(&mut game, b);

    game.resolve_top_of_stack(&dp).expect("one legal target is still enough");

    assert_eq!(counters(&game, c), 3, "the third clause's three, and nothing else's");
    assert_eq!(
        game.get_object(growth).unwrap().zone,
        Zone::Graveyard,
        "resolved, not countered by game rules"
    );
}

/// **A doubler applies per instruction, not per spell** — Vorinclex against
/// three clauses that put one, two and three counters on three creatures.
///
/// *"If you would put one or more counters on a permanent or player, put twice
/// that many of each of those kinds of counters on that permanent or player
/// instead."* Each clause is its own `AddCounters` proposal, so CR 614.5 asks
/// Vorinclex three times and the answer is **2, 4, 6** — not twelve on one
/// creature, and not six doubled once.
///
/// RE-5 asserted the shape against a two-atom fixture on **one** instance
/// (Winding Constrictor's ruling, "if an effect includes multiple instructions
/// … the effect applies to each of those instructions"). This is the same claim
/// where the instructions also land on different subjects, which is what A4i
/// made reachable — and both cards are registered, so a `stress` game can
/// build this board.
#[test]
fn vorinclex_doubles_each_of_incremental_growths_three_instructions() {
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, vorinclex_monstrous_raider(), 0);
    let a = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let b = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let c = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let growth = put_in_hand(&mut game, incremental_growth(), 0);
    pay_for_growth(&mut game, 0);

    // Vorinclex is itself a creature, so the board has four and every
    // announcement is a real prompt. Battlefield order is timestamp order, so
    // index 1 skips past Vorinclex each time.
    let dp = stopping_dp();
    answer_targets(&dp, target_creature(), growth, vec![1]);
    answer_targets(&dp, another_target_creature(&[0]), growth, vec![1]);
    answer_targets(&dp, another_target_creature(&[0, 1]), growth, vec![1]);
    cast_and_resolve(&mut game, 0, growth, &dp);

    assert_eq!(counters(&game, a), 2, "one, doubled");
    assert_eq!(counters(&game, b), 4, "two, doubled");
    assert_eq!(counters(&game, c), 6, "three, doubled");
}
