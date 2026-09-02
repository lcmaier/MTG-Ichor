//! Phase CV-1 integration tests: the copy spine (CR 707, layer 1a).
//!
//! Four things this file is trying to prove, in the order the phase built them:
//!
//! 1. A copy is a **snapshot** — captured once at the end of layer 1 (CR
//!    613.2c) and never re-derived (CR 707.2b/2c, 611.2c).
//! 2. What the snapshot **excludes** — status, counters, control and every
//!    Layer 2–7 row on the copied object (CR 707.2's last sentence). This is
//!    `copy-effects-architecture.md` §3.1's third property, and it is asserted
//!    rather than argued because a future change to `compute_to_ceiling`'s
//!    default ceiling would silently start copying anthems.
//! 3. The two ETB scans a copy defeats (`codebase-state.md` item 16) — the
//!    gate legs and the static re-registration path.
//! 4. That none of it prompts a `DecisionProvider` when there is nothing to
//!    choose.

use std::cell::RefCell;
use std::sync::Arc;

use mtgsim::cards::phase5_pre_cards::glorious_anthem;
use mtgsim::cards::phase_cv_cards::{cytoshape, mirrorweave};
use mtgsim::cards::phase_rc_cards::containment_priest;
use mtgsim::engine::layers::copy::copiable_values;
use mtgsim::engine::layers::types::{EffectModification, Layer};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::objects::card_data::{AbilityDef, AbilityType, CardData, CardDataBuilder};
use mtgsim::objects::object::GameObject;
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_controller, get_effective_name, get_effective_power,
    get_effective_toughness, get_effective_types, has_keyword,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    place_bare, put_on_battlefield, setup_two_player_game, static_ability, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AffectedSet, AmountExpr, CounterType, Duration, Effect, EffectRecipient, PermanentFilter,
    PlayerRef, Primitive,
};
use mtgsim::types::ids::{new_ability_id, ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::mana::{ManaCost, ManaType};
use mtgsim::types::replacement::EventPattern;
use mtgsim::types::restriction::{Restriction, RestrictionDef, SourceFilter};
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceKind, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A `DecisionProvider` that records every `ChoiceKind` it is asked about and
/// picks a nominated option.
///
/// Recording the *kind* rather than only counting is what lets a test say "one
/// prompt, and it was `ChooseCopySource`" — the assertion CV-1's new decision
/// site actually owes. `ScriptedDecisionProvider` with an empty queue proves the
/// absence of prompts and is used for that half.
struct RecordingDp {
    pick: usize,
    seen: RefCell<Vec<String>>,
}

impl RecordingDp {
    fn picking(pick: usize) -> Self {
        RecordingDp { pick, seen: RefCell::new(Vec::new()) }
    }
    fn kinds(&self) -> Vec<String> {
        self.seen.borrow().clone()
    }
}

impl DecisionProvider for RecordingDp {
    fn pick_n(
        &self,
        _game: &GameState,
        _player: PlayerId,
        ctx: &ChoiceContext,
        options: &[ChoiceOption],
        _bounds: (usize, usize),
    ) -> Vec<usize> {
        self.seen.borrow_mut().push(format!("{:?}", ctx.kind));
        vec![self.pick.min(options.len().saturating_sub(1))]
    }

    fn pick_number(
        &self,
        _game: &GameState,
        _player: PlayerId,
        _ctx: &ChoiceContext,
        min: u64,
        _max: u64,
    ) -> u64 {
        self.seen.borrow_mut().push("pick_number".to_string());
        min
    }

    fn allocate(
        &self,
        _game: &GameState,
        _player: PlayerId,
        _ctx: &ChoiceContext,
        total: u64,
        buckets: &[ChoiceOption],
        _per_bucket_mins: &[u64],
        _per_bucket_maxs: Option<&[u64]>,
    ) -> Vec<u64> {
        self.seen.borrow_mut().push("allocate".to_string());
        let mut out = vec![0; buckets.len()];
        if !out.is_empty() {
            out[0] = total;
        }
        out
    }

    fn choose_ordering(
        &self,
        _game: &GameState,
        _player: PlayerId,
        _ctx: &ChoiceContext,
        items: &[ChoiceOption],
    ) -> Vec<usize> {
        self.seen.borrow_mut().push("choose_ordering".to_string());
        (0..items.len()).collect()
    }
}

/// Resolve a card's one spell ability with the given targets.
fn resolve_spell(
    game: &mut GameState,
    card: Arc<CardData>,
    controller: PlayerId,
    targets: Vec<ObjectId>,
    dp: &dyn DecisionProvider,
) {
    // The spell object itself, off the battlefield: `ResolutionContext.source`
    // is the stack object, which is what `copy-effects-architecture.md` §5.3
    // establishes and what keeps `remove_by_source` out of a copy row's
    // teardown.
    let source = GameObject::new(card.clone(), controller, Zone::Stack);
    let source_id = source.id;
    game.add_object(source);
    let effect = card.abilities[0].effect.clone();
    let ctx = ResolutionContext {
        source: source_id,
        controller,
        targets: targets.into_iter().map(ResolvedTarget::Object).collect(),
    };
    game.resolve_effect(&effect, &ctx, dp).expect("resolution failed");
}

/// A 2/2 named "Bear" with no abilities.
fn bear() -> Arc<CardData> {
    CardDataBuilder::new("Bear")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green], 1))
        .power_toughness(2, 2)
        .build()
}

/// A 5/5 named "Colossus" with flying, so a capture has an ability, a keyword
/// and a P/T to carry.
fn colossus() -> Arc<CardData> {
    CardDataBuilder::new("Colossus")
        .card_type(CardType::Creature)
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 4))
        .power_toughness(5, 5)
        .keyword(KeywordFlag::Flying)
        .build()
}

/// A creature that is also a Glorious Anthem — the Clone-of-an-Anthem probe of
/// `copy-effects-architecture.md` §4.7 leg 2, as a *creature* so Cytoshape can
/// choose it.
fn anthem_bear() -> Arc<CardData> {
    CardDataBuilder::new("Anthem Bear")
        .card_type(CardType::Creature)
        .color(Color::White)
        .mana_cost(ManaCost::build(&[ManaType::White], 2))
        .power_toughness(2, 2)
        .ability(static_ability(Effect::Atom(
            Primitive::ModifyPowerToughness(
                AmountExpr::Fixed(1),
                AmountExpr::Fixed(1),
                Duration::WhileSourceOnBattlefield,
            ),
            EffectRecipient::FilteredPermanents(PermanentFilter::And(
                Box::new(PermanentFilter::ByType(CardType::Creature)),
                Box::new(PermanentFilter::ByController(PlayerRef::You)),
            )),
        )))
        .build()
}

// ---------------------------------------------------------------------------
// 1. The capture, and that it is a snapshot
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-613.2a-001
/// CR 613.2a — a layer 1a row replaces every characteristic channel at once.
///
/// Partial against the atom, whose board is a Clone entering as a copy: CV-1
/// has no entry producer (that is CV-2), so the same claim is built from a
/// resolution. The claim being tested — that the copy has the donor's name,
/// types, abilities and P/T — is the atom's, verbatim.
#[test]
fn test_cytoshape_target_becomes_a_copy_of_the_chosen_creature() {
    let mut game = setup_two_player_game();
    let target = put_on_battlefield(&mut game, bear(), 0);
    let donor = put_on_battlefield(&mut game, colossus(), 0);

    let dp = RecordingDp::picking(1); // index 1 of the ordered candidates
    resolve_spell(&mut game, cytoshape(), 0, vec![target], &dp);

    // The donor is whichever of the two the DP picked; assert on the one that
    // is not the target, so the test does not depend on battlefield order.
    let chosen_is_donor = get_effective_name(&game, target) == "Colossus";
    assert!(
        chosen_is_donor || get_effective_name(&game, target) == "Bear",
        "target became something neither candidate is"
    );
    if chosen_is_donor {
        assert_eq!(get_effective_power(&game, target), Some(5));
        assert_eq!(get_effective_toughness(&game, target), Some(5));
        assert!(has_keyword(&game, target, KeywordFlag::Flying));
    }
    // The donor is untouched either way — a copy effect changes only its
    // affected set.
    assert_eq!(get_effective_name(&game, donor), "Colossus");
    assert_eq!(get_effective_power(&game, donor), Some(5));

    assert_eq!(
        dp.kinds().len(),
        1,
        "exactly one prompt, and it is the CR 707.4 choice: {:?}",
        dp.kinds()
    );
    assert!(dp.kinds()[0].starts_with("ChooseCopySource"), "{:?}", dp.kinds());
}

// COVERS-PARTIAL: ATOM-613.2c-001
/// CR 613.2c / 707.2 — copiable values are read **after** layer 1, so a copy of
/// a copy gets the first copy's values, not its printed card.
///
/// The capture-ceiling test. If `copiable_values` took ceiling 0 instead of
/// `END_OF_LAYER_1`, the second Cytoshape would find the printed "Bear".
#[test]
fn test_a_copy_of_a_copy_captures_post_layer_1_values() {
    let mut game = setup_two_player_game();
    let first = put_on_battlefield(&mut game, bear(), 0);
    let donor = put_on_battlefield(&mut game, colossus(), 0);
    let second = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);

    // `first` becomes a Colossus.
    copy_onto(&mut game, first, donor);
    assert_eq!(get_effective_name(&game, first), "Colossus");

    // `second` becomes a copy of `first`, which is now a Colossus.
    copy_onto(&mut game, second, first);
    assert_eq!(get_effective_name(&game, second), "Colossus");
    assert_eq!(get_effective_power(&game, second), Some(5));
    assert!(has_keyword(&game, second, KeywordFlag::Flying));
}

// COVERS-PARTIAL: ATOM-707.2b-001
/// CR 707.2b — "changing the copiable values of the original object won't cause
/// the copy to change".
///
/// Satisfied by construction rather than by a severing step, because the row
/// stores values. This test is what proves the construction: the donor is
/// re-copied into something else afterwards and the first copy does not move.
#[test]
fn test_changing_the_donor_later_does_not_change_the_copy() {
    let mut game = setup_two_player_game();
    let copyist = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let donor = put_on_battlefield(&mut game, colossus(), 0);
    let third = put_on_battlefield(&mut game, bear(), 1);

    copy_onto(&mut game, copyist, donor);
    assert_eq!(get_effective_name(&game, copyist), "Colossus");

    // The donor itself becomes a Bear (CR 707.4's re-copy).
    copy_onto(&mut game, donor, third);
    assert_eq!(get_effective_name(&game, donor), "Bear");

    // The first copy is unmoved.
    assert_eq!(get_effective_name(&game, copyist), "Colossus");
    assert_eq!(get_effective_power(&game, copyist), Some(5));
}

// COVERS-PARTIAL: ATOM-707.4-001
/// CR 707.4 — a permanent that is copying a permanent can copy a *different*
/// object while remaining on the battlefield, and "this doesn't change any
/// noncopy effects presently affecting the permanent".
///
/// Partial: the atom also asserts that no ETB or LTB ability triggers, which
/// this engine cannot yet observe (critical-path item 6). What *is* asserted is
/// the second clause, which is the one the row model has to earn: a Layer 7c
/// pump registered before the re-copy still applies after it.
#[test]
fn test_re_copy_keeps_noncopy_effects_and_stays_on_the_battlefield() {
    let mut game = setup_two_player_game();
    let shifter = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let bear_id = put_on_battlefield(&mut game, bear(), 0);
    let colossus_id = put_on_battlefield(&mut game, colossus(), 0);

    // A Giant-Growth-shaped Layer 7c row on the shifter.
    pump(&mut game, shifter, 3, 3);
    copy_onto(&mut game, shifter, bear_id);
    assert_eq!(get_effective_power(&game, shifter), Some(2 + 3));

    copy_onto(&mut game, shifter, colossus_id);
    assert_eq!(get_effective_name(&game, shifter), "Colossus");
    // The pump survived the re-copy: layer 7c is untouched by a layer 1 row.
    assert_eq!(get_effective_power(&game, shifter), Some(5 + 3));
    assert!(
        game.battlefield.contains_key(&shifter),
        "CR 707.4 — a re-copy is not a zone change"
    );
}

// ---------------------------------------------------------------------------
// 2. What the capture excludes — §3.1's third property
// ---------------------------------------------------------------------------

// COVERS-PARTIAL: ATOM-707.2-001
// COVERS-PARTIAL: ATOM-707.2-003
/// CR 707.2's last sentence — an anthem's Layer 7c row and a +1/+1 counter on
/// the donor are **not** copiable, and neither is its tapped status.
///
/// `copy-effects-architecture.md` §3.1's third property, and the arithmetic is
/// what makes it sharp rather than incidental. Everything here is player 0's, so
/// the anthem reaches the copy too: the copy is `2 (printed) + 1 (anthem
/// applying to it now)` = **3**. If the donor's anthem row or its counter had
/// been captured, the copy's base would be 3/3 and the same anthem would take it
/// to 4/4. Three versus four is the whole assertion.
#[test]
fn test_the_capture_excludes_counters_status_and_later_layers() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, bear(), 0);
    put_on_battlefield(&mut game, glorious_anthem(), 0);
    game.add_counters(donor, CounterType::PlusOnePlusOne, 1);
    game.battlefield.get_mut(&donor).unwrap().tapped = true;

    // The donor really is a 4/4 right now: 2/2, +1/+1 anthem, +1/+1 counter.
    assert_eq!(get_effective_power(&game, donor), Some(4));

    let copyist = place_bare(&mut game, vanilla_creature(1, 1, &[]), 0);
    copy_onto(&mut game, copyist, donor);

    assert_eq!(get_effective_name(&game, copyist), "Bear");
    assert_eq!(
        get_effective_power(&game, copyist),
        Some(3),
        "CR 707.2 — the donor's anthem row and its +1/+1 counter are not copiable; \
         the anthem applies to the copy once, on its own account"
    );
    assert_eq!(get_effective_toughness(&game, copyist), Some(3));
    assert!(
        !game.battlefield[&copyist].tapped,
        "CR 707.2 — tapped is a status, not a characteristic"
    );
    assert_eq!(
        game.battlefield[&copyist].counters.get(&CounterType::PlusOnePlusOne),
        None,
        "CR 707.2 — counters are not copied"
    );

    // And the same claim at the capture function, which is where it is
    // structural: a raw capture of a 4/4-on-the-board donor is a printed 2/2.
    let values = copiable_values(&game, donor).unwrap();
    assert_eq!(values.power, Some(2));
    assert_eq!(values.toughness, Some(2));
    assert_eq!(values.name, "Bear");

    // The donor is unchanged — the exclusion is about the capture, not the board.
    assert_eq!(get_effective_power(&game, donor), Some(4));
}

/// CR 707.2 excludes **control**, which is why `CopiableValues` is not
/// `EffectiveCharacteristics` reused: that type carries a `controller`, and
/// `compute.rs`'s `any_control_changing` fast path is sound only while Layer 2
/// is the one channel that writes one.
#[test]
fn test_the_capture_excludes_control() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, colossus(), 0);
    steal(&mut game, donor, 1);
    assert_eq!(get_effective_controller(&game, donor), Some(1));

    let copyist = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    copy_onto(&mut game, copyist, donor);

    assert_eq!(get_effective_name(&game, copyist), "Colossus");
    assert_eq!(
        get_effective_controller(&game, copyist),
        Some(0),
        "a copy of a stolen permanent does not change hands"
    );
    // And the flag that proves the fast path is unaffected: no copy row can set
    // a controller, because the type has no field for one.
    assert!(game.continuous_effects.summary().any_control_changing);
}

/// The copy is a permanent in its own right, and a Layer 7c effect that applies
/// to it *after* the copy row still does.
///
/// The other direction of the test above: the copy is not frozen, only its
/// layer-1 values are.
#[test]
fn test_later_layers_still_apply_to_the_copy() {
    let mut game = setup_two_player_game();
    let copyist = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let donor = put_on_battlefield(&mut game, colossus(), 0);

    copy_onto(&mut game, copyist, donor);
    game.add_counters(copyist, CounterType::PlusOnePlusOne, 2);
    assert_eq!(get_effective_power(&game, copyist), Some(7));
}

// ---------------------------------------------------------------------------
// 3. The two ETB scans a copy defeats — codebase-state.md item 16
// ---------------------------------------------------------------------------

/// `copy-effects-architecture.md` §4.7 **leg 2** — a copy of a permanent with a
/// static continuous ability must register a row for it.
///
/// The Clone-of-a-Glorious-Anthem probe. Before CV-1 the copy had the ability on
/// its effective ability list and pumped nothing, because
/// `register_static_effects` reads printed abilities and runs only at ETB.
#[test]
fn test_a_copied_anthem_pumps() {
    let mut game = setup_two_player_game();
    let anthem_source = put_on_battlefield(&mut game, anthem_bear(), 0);
    let copyist = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let bystander = put_on_battlefield(&mut game, bear(), 0);

    // One anthem on the board: everything player 0 controls is +1/+1.
    assert_eq!(get_effective_power(&game, bystander), Some(3));

    copy_onto(&mut game, copyist, anthem_source);

    // The copy *has* the ability...
    assert!(
        get_effective_abilities(&game, copyist)
            .iter()
            .any(|a| a.ability_type == AbilityType::Static),
        "CR 707.2a — the copy acquired the static ability"
    );
    // ...and now it also *does* something: two anthems, so +2/+2.
    assert_eq!(
        get_effective_power(&game, bystander),
        Some(4),
        "§4.7 leg 2 — a copied static ability needs its own registry row"
    );
    // Including on the copy itself and on the original anthem.
    assert_eq!(get_effective_power(&game, copyist), Some(2 + 2));
    assert_eq!(get_effective_power(&game, anthem_source), Some(2 + 2));
}

/// CR 613.7a is what tears the derived rows down, and it does so before any
/// registry mutation: re-copy the anthem-copy into something else and its
/// derived row stops applying, because the ability is no longer on the source's
/// frame.
///
/// This is the invariant `register_copied_static_effects`' doc rests on —
/// "registry membership is not effect existence" — asserted rather than assumed.
#[test]
fn test_a_superseded_copy_stops_its_derived_row_applying() {
    let mut game = setup_two_player_game();
    let anthem_source = put_on_battlefield(&mut game, anthem_bear(), 0);
    let copyist = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let bystander = put_on_battlefield(&mut game, bear(), 0);
    let plain = put_on_battlefield(&mut game, colossus(), 0);

    copy_onto(&mut game, copyist, anthem_source);
    assert_eq!(get_effective_power(&game, bystander), Some(4));

    // CR 707.4 — the copy now copies something with no static ability.
    copy_onto(&mut game, copyist, plain);
    assert_eq!(
        get_effective_power(&game, bystander),
        Some(3),
        "CR 613.7a — the derived row's ability is gone from the source's frame"
    );
    // The superseded derived row is still *in* the registry; it is inert, not
    // absent. Recorded in Deferred Migrations.
    assert!(
        game.continuous_effects
            .iter()
            .any(|e| e.source == copyist && e.layer == Layer::Layer7cModifyPT),
        "the row is still registered — teardown is by existence check, not removal"
    );
}

/// `copy-effects-architecture.md` §4.7 **leg 1**, replacement half — a copied
/// replacement ability turns `RegistryScopeSummary::any_copied_replacement` on,
/// which is what lets `gather`'s fast-path gate see it.
#[test]
fn test_a_copied_replacement_ability_lights_the_gather_gate() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, replacement_creature(), 0);
    let copyist = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);

    assert!(
        !game.continuous_effects.summary().any_copied_replacement,
        "no copy row yet"
    );
    copy_onto(&mut game, copyist, donor);
    assert!(
        game.continuous_effects.summary().any_copied_replacement,
        "§4.7 leg 1 — the gate's third leg"
    );
    assert!(
        !game.continuous_effects.summary().any_copied_restriction,
        "narrower than 'any copy at all': the two sweeps read different bodies"
    );
}

/// The same leg on the **other** gate — `engine::restriction::is_prohibited`'s,
/// which RS-1 built to the same recipe and which
/// `copy-effects-architecture.md` §4.7 had not counted.
#[test]
fn test_a_copied_restriction_ability_lights_the_restriction_gate() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, restriction_creature(), 0);
    let copyist = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);

    copy_onto(&mut game, copyist, donor);
    assert!(
        game.continuous_effects.summary().any_copied_restriction,
        "the second gate's third leg"
    );
    assert!(!game.continuous_effects.summary().any_copied_replacement);
}

/// A copy of a vanilla creature must turn **neither** gate on. The flags are a
/// scan of the captured ability list, not a count of copy rows, and a flag that
/// was permanently on would buy nothing.
#[test]
fn test_a_copy_of_a_vanilla_creature_lights_neither_gate() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, colossus(), 0);
    let copyist = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);

    copy_onto(&mut game, copyist, donor);
    let summary = game.continuous_effects.summary();
    assert!(!summary.any_copied_replacement);
    assert!(!summary.any_copied_restriction);
}

// ---------------------------------------------------------------------------
// 4. Mirrorweave, the other `CopyRoles` arm, and the prompt discipline
// ---------------------------------------------------------------------------

/// CR 707.4 — "each other creature becomes a copy of target nonlegendary
/// creature". The target is the **donor**, and it is excluded from the affected
/// set structurally.
#[test]
fn test_mirrorweave_copies_the_target_onto_every_other_creature() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, colossus(), 0);
    let mine = put_on_battlefield(&mut game, bear(), 0);
    let theirs = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 1);
    let noncreature = put_on_battlefield(&mut game, glorious_anthem(), 0);

    // No prompt: Mirrorweave targets, it does not choose.
    let dp = ScriptedDecisionProvider::new();
    resolve_spell(&mut game, mirrorweave(), 0, vec![donor], &dp);

    assert_eq!(get_effective_name(&game, mine), "Colossus");
    assert_eq!(get_effective_name(&game, theirs), "Colossus");
    assert_eq!(get_effective_power(&game, theirs), Some(5));
    assert_eq!(
        get_effective_name(&game, donor),
        "Colossus",
        "the donor is 'other'-excluded, so it is unaffected either way"
    );
    assert_eq!(
        get_effective_name(&game, noncreature),
        "Glorious Anthem",
        "the filter is creatures"
    );
    // Control is not copiable: the opponent's creature stays theirs.
    assert_eq!(get_effective_controller(&game, theirs), Some(1));
}

/// One row for the whole class, which is `copy-effects-architecture.md` §9 item
/// 4's answer: CR 611.2c locks the affected set, so `Box` and `Arc` allocate the
/// same single capture here.
#[test]
fn test_mirrorweave_registers_one_row_for_the_whole_class() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, colossus(), 0);
    for _ in 0..4 {
        put_on_battlefield(&mut game, bear(), 0);
    }

    let dp = ScriptedDecisionProvider::new();
    resolve_spell(&mut game, mirrorweave(), 0, vec![donor], &dp);

    let copy_rows: Vec<_> = game
        .continuous_effects
        .iter()
        .filter(|e| matches!(e.modification, EffectModification::CopyFrom(_)))
        .collect();
    assert_eq!(copy_rows.len(), 1, "one capture, one row, four affected objects");
    assert_eq!(copy_rows[0].layer, Layer::Layer1Copy);
}

/// CR 102.2 — with one candidate the CR 707.4 choice is forced, so nothing is
/// asked. `ScriptedDecisionProvider` with an empty queue panics on any prompt,
/// which is the assertion.
#[test]
fn test_cytoshape_asks_nothing_when_only_one_creature_can_be_chosen() {
    let mut game = setup_two_player_game();
    let target = put_on_battlefield(&mut game, bear(), 0);
    // `target` is itself the only nonlegendary creature on the battlefield.
    let dp = ScriptedDecisionProvider::new();
    resolve_spell(&mut game, cytoshape(), 0, vec![target], &dp);
    assert_eq!(get_effective_name(&game, target), "Bear");
}

/// CR 608.2b — a target that has left the battlefield leaves nothing to affect,
/// and no prompt is produced for a choice that could not change anything.
#[test]
fn test_cytoshape_with_no_surviving_target_asks_nothing_and_registers_nothing() {
    let mut game = setup_two_player_game();
    let gone = put_on_battlefield(&mut game, bear(), 0);
    put_on_battlefield(&mut game, colossus(), 0);
    put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0);
    game.battlefield.remove(&gone);

    let dp = ScriptedDecisionProvider::new();
    resolve_spell(&mut game, cytoshape(), 0, vec![gone], &dp);
    assert!(
        !game
            .continuous_effects
            .iter()
            .any(|e| matches!(e.modification, EffectModification::CopyFrom(_))),
        "no target, no effect, no row"
    );
}

/// CR 514.2 — a turn-bounded copy expires at cleanup, and so do the rows its
/// copied static ability generated, because they share the `Duration`.
#[test]
fn test_a_turn_bounded_copy_and_its_derived_rows_expire_together() {
    let mut game = setup_two_player_game();
    let anthem_source = put_on_battlefield(&mut game, anthem_bear(), 0);
    let copyist = put_on_battlefield(&mut game, vanilla_creature(1, 1, &[]), 0);
    let bystander = put_on_battlefield(&mut game, bear(), 0);

    copy_onto(&mut game, copyist, anthem_source);
    assert_eq!(get_effective_power(&game, bystander), Some(4));

    game.continuous_effects.remove_expired_at_cleanup(0, game.turn_number);

    assert_eq!(get_effective_name(&game, copyist), "Test Creature");
    assert_eq!(
        get_effective_power(&game, bystander),
        Some(3),
        "the derived row carried the copy row's duration"
    );
    assert!(!game.continuous_effects.summary().any_copied_replacement);
}

// ---------------------------------------------------------------------------
// Shared fixtures for the tests above
// ---------------------------------------------------------------------------

/// Cytoshape `donor` onto `target`, picking the donor deterministically by id
/// rather than by prompt index.
fn copy_onto(game: &mut GameState, target: ObjectId, donor: ObjectId) {
    struct PickById(ObjectId);
    impl DecisionProvider for PickById {
        fn pick_n(
            &self,
            _game: &GameState,
            _player: PlayerId,
            ctx: &ChoiceContext,
            options: &[ChoiceOption],
            _bounds: (usize, usize),
        ) -> Vec<usize> {
            assert!(
                matches!(ctx.kind, ChoiceKind::ChooseCopySource { .. }),
                "unexpected prompt: {:?}",
                ctx.kind
            );
            let idx = options
                .iter()
                .position(|o| matches!(o, ChoiceOption::Object(id) if *id == self.0))
                .expect("donor was not offered as a candidate");
            vec![idx]
        }
        fn pick_number(
            &self,
            _g: &GameState,
            _p: PlayerId,
            _c: &ChoiceContext,
            min: u64,
            _max: u64,
        ) -> u64 {
            min
        }
        fn allocate(
            &self,
            _g: &GameState,
            _p: PlayerId,
            _c: &ChoiceContext,
            total: u64,
            buckets: &[ChoiceOption],
            _mins: &[u64],
            _maxs: Option<&[u64]>,
        ) -> Vec<u64> {
            let mut out = vec![0; buckets.len()];
            if !out.is_empty() {
                out[0] = total;
            }
            out
        }
        fn choose_ordering(
            &self,
            _g: &GameState,
            _p: PlayerId,
            _c: &ChoiceContext,
            items: &[ChoiceOption],
        ) -> Vec<usize> {
            (0..items.len()).collect()
        }
    }
    resolve_spell(game, cytoshape(), 0, vec![target], &PickById(donor));
}

/// A Giant-Growth-shaped Layer 7c row, registered the way a resolution does.
fn pump(game: &mut GameState, id: ObjectId, power: i32, toughness: i32) {
    use mtgsim::engine::layers::types::{ContinuousEffect, EffectOrigin, PtValue};
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(ContinuousEffect {
        id: 0,
        source: id,
        origin: EffectOrigin::Resolution,
        layer: Layer::Layer7cModifyPT,
        duration: Duration::UntilEndOfTurn,
        controller: 0,
        created_on_turn: game.turn_number,
        timestamp,
        affected: AffectedSet::Fixed(vec![id]),
        modification: EffectModification::ModifyPowerToughness {
            power: PtValue::Fixed(power),
            toughness: PtValue::Fixed(toughness),
        },
    });
}

/// An Act-of-Treason-shaped Layer 2 row.
fn steal(game: &mut GameState, id: ObjectId, to: PlayerId) {
    use mtgsim::engine::layers::types::{ContinuousEffect, EffectOrigin};
    let timestamp = game.allocate_timestamp();
    game.continuous_effects.add(ContinuousEffect {
        id: 0,
        source: id,
        origin: EffectOrigin::Resolution,
        layer: Layer::Layer2Control,
        duration: Duration::UntilEndOfTurn,
        controller: to,
        created_on_turn: game.turn_number,
        timestamp,
        affected: AffectedSet::Fixed(vec![id]),
        modification: EffectModification::SetController(PlayerRef::Player(to)),
    });
}

/// A creature carrying a static *replacement* ability, for the leg-1 gate test.
///
/// Containment Priest verbatim {D} a registered, nonlegendary creature whose only
/// ability is an `Effect::Replacement`, so Cytoshape can choose it and the test
/// asserts about the shipped card rather than about a fixture.
fn replacement_creature() -> Arc<CardData> {
    containment_priest()
}

/// A creature carrying a static *restriction* ability, for the second gate.
///
/// Sigarda's clause on a nonlegendary body. Sigarda herself is the crate's only
/// restriction card and is Legendary, which Cytoshape's printed filter excludes
/// {D} so the fixture keeps the ability and drops the supertype rather than
/// weakening the card's filter to suit a test.
fn restriction_creature() -> Arc<CardData> {
    CardDataBuilder::new("Restriction Bear")
        .card_type(CardType::Creature)
        .color(Color::Green)
        .mana_cost(ManaCost::build(&[ManaType::Green], 3))
        .power_toughness(3, 3)
        .ability(AbilityDef {
            id: new_ability_id(),
            is_characteristic_defining: false,
            ability_type: AbilityType::Static,
            costs: Vec::new(),
            effect: Effect::Restriction(Box::new(RestrictionDef::new(Restriction::Event {
                pattern: EventPattern::ZoneChange {
                    from: Some(Zone::Battlefield),
                    to: Some(Zone::Graveyard),
                    cause: Some(ZoneChangeCause::Sacrificed),
                    object: None,
                },
                affected: AffectedSet::Filter {
                    filter: PermanentFilter::ByController(PlayerRef::You),
                },
                by: Some(SourceFilter::ControlledBy(PlayerRef::Opponent)),
            }))),
        })
        .build()
}

/// CR 707.2 {D} the capture carries types, and they are the *effective* types,
/// not the printed ones.
#[test]
fn test_capture_carries_effective_types() {
    let mut game = setup_two_player_game();
    let donor = put_on_battlefield(&mut game, colossus(), 0);
    let values = copiable_values(&game, donor).unwrap();
    assert!(values.types.contains(&CardType::Creature));
    assert_eq!(get_effective_types(&game, donor), values.types);
}
