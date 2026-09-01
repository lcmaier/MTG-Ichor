//! Phase RS-1 integration tests: the Tier-2 "can't" spine (CR 101.2, 614.17).
//!
//! **Half of this file asserts on an absence**, which is unusual and is the
//! point. `cant-effects-architecture.md` §4.9 makes "produces no prompt" a rules
//! requirement rather than a UX nicety, so the headline assertion is that a
//! `DecisionProvider` was never called — and a test that passes because the
//! whole path never ran would assert exactly the same thing. Every such test
//! here is therefore paired with a control that removes only the restriction and
//! shows the prompt appearing.

use std::cell::RefCell;

use mtgsim::cards::phase_lf_cards;
use mtgsim::cards::phase_rs_cards::{diabolic_edict, sigarda_host_of_herons};
use mtgsim::engine::actions::{DestructionSource, GameAction};
use mtgsim::engine::layers::types::{EffectModification, Layer};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::state::game_state::GameState;
use mtgsim::state::restrictions::RegisteredRestriction;
use mtgsim::test_support::{
    place_bare, put_on_battlefield, registered, setup_two_player_game, test_ctx, vanilla_creature,
};
use mtgsim::types::effects::{
    AffectedSet, AmountExpr, Duration, Effect, EffectRecipient, PermanentFilter, PlayerRef,
    Primitive, SelectionFilter, TargetCount,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::replacement::EventPattern;
use mtgsim::types::restriction::{
    ReplacementKindFilter, Restriction, RestrictionDef, SourceFilter,
};
use mtgsim::types::zones::{Zone, ZoneChangeCause};
use mtgsim::ui::choice_types::{ChoiceContext, ChoiceOption};
use mtgsim::ui::decision::{DecisionProvider, ScriptedDecisionProvider};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A `DecisionProvider` that counts every call and always takes the first
/// option.
///
/// `ScriptedDecisionProvider` with an empty queue already *panics* on an
/// unexpected prompt, which is a strong zero-prompt assertion — but it can only
/// say "none", never "exactly one". The control tests need the second half, so
/// both halves come from one type and the two assertions are comparable.
struct CountingDp {
    prompts: RefCell<usize>,
}

impl CountingDp {
    fn new() -> Self {
        CountingDp { prompts: RefCell::new(0) }
    }
    fn count(&self) -> usize {
        *self.prompts.borrow()
    }
}

impl DecisionProvider for CountingDp {
    fn pick_n(
        &self,
        _game: &GameState,
        _player: PlayerId,
        _ctx: &ChoiceContext,
        options: &[ChoiceOption],
        bounds: (usize, usize),
    ) -> Vec<usize> {
        *self.prompts.borrow_mut() += 1;
        let (min, _max) = bounds;
        (0..min.min(options.len())).collect()
    }

    fn pick_number(
        &self,
        _game: &GameState,
        _player: PlayerId,
        _ctx: &ChoiceContext,
        min: u64,
        _max: u64,
    ) -> u64 {
        *self.prompts.borrow_mut() += 1;
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
        *self.prompts.borrow_mut() += 1;
        let mut out = vec![0; buckets.len()];
        if let Some(first) = out.first_mut() {
            *first = total;
        }
        out
    }

    fn choose_ordering(
        &self,
        _game: &GameState,
        _player: PlayerId,
        _ctx: &ChoiceContext,
        options: &[ChoiceOption],
    ) -> Vec<usize> {
        *self.prompts.borrow_mut() += 1;
        (0..options.len()).collect()
    }
}

/// Resolve Diabolic Edict, cast by `caster`, targeting `victim`.
///
/// Built as a `ResolutionContext` rather than cast through the stack, for the
/// reason RB's tests give: the question is what the *resolution* does, and going
/// through casting would add a mana base, a priority pass and a target-selection
/// prompt to every assertion about prompts.
fn resolve_edict(
    game: &mut GameState,
    caster: PlayerId,
    victim: PlayerId,
    dp: &dyn DecisionProvider,
) {
    let edict = place_bare(game, diabolic_edict(), caster);
    let effect = edict_effect();
    let ctx = ResolutionContext {
        source: edict,
        controller: caster,
        targets: vec![ResolvedTarget::Player(victim)],
    };
    game.resolve_effect(&effect, &ctx, dp).unwrap();
}

/// Diabolic Edict's one spell ability, read back off the registered card so the
/// tests exercise the shipped card data rather than a lookalike.
fn edict_effect() -> Effect {
    diabolic_edict().abilities[0].effect.clone()
}

/// An edict for `n` creatures — Barter in Blood's and Blasphemous Edict's shape.
///
/// Not a registered card: both of those say "**each player** sacrifices", and
/// `EffectRecipient` has no each-player variant, so neither is writable yet for
/// a reason that has nothing to do with this phase. The *count* is what RS-1
/// owes them, and this is what exercises it.
fn edict_for(n: u64) -> Effect {
    Effect::Atom(
        Primitive::Sacrifice(SelectionFilter::Creature, AmountExpr::Fixed(n)),
        EffectRecipient::Target(SelectionFilter::Player, TargetCount::Exactly(1)),
    )
}

/// Resolve an arbitrary edict-shaped effect against `victim`.
fn resolve_edict_effect(
    game: &mut GameState,
    caster: PlayerId,
    victim: PlayerId,
    effect: &Effect,
    dp: &dyn DecisionProvider,
) {
    let source = place_bare(game, diabolic_edict(), caster);
    let ctx = ResolutionContext {
        source,
        controller: caster,
        targets: vec![ResolvedTarget::Player(victim)],
    };
    game.resolve_effect(effect, &ctx, dp).unwrap();
}

/// A plain creature — one that is not the restriction's own source.
///
/// Sigarda protects everything her controller controls, herself included, so
/// this is not about protection scope. It is about telling `AffectedSet::Filter`
/// apart from `AffectedSet::SourceOnly`: with only Sigarda on the board, a
/// restriction that (wrongly) applied to its source alone would suppress the
/// same single prompt and pass the same assertion. A second creature is what
/// makes the two answers differ.
///
/// Verified rather than reasoned: flipping Sigarda's `affected` to
/// `AffectedSet::SourceOnly` fails both zero-prompt tests and nothing else.
fn bear(game: &mut GameState, owner: PlayerId) -> ObjectId {
    put_on_battlefield(game, vanilla_creature(2, 2, &[]), owner)
}

// ---------------------------------------------------------------------------
// §4.9 — the candidate filter. Sigarda, and her three boundaries.
// ---------------------------------------------------------------------------

#[test]
fn test_sigarda_makes_an_opponents_edict_produce_no_prompt_at_all() {
    // The headline, and it is a rules requirement rather than a UX nicety.
    // Sigarda's printed ruling: "As a spell or ability an opponent controls
    // resolves, if it would force you to sacrifice a permanent, **you just
    // don't. That part of the effect does nothing.**"
    //
    // Prompting and then refusing would be wrong three times over: CR 608.2d
    // ("the player can't choose an option that's illegal or impossible"), it
    // would leak which creature P0 would have picked, and it would make an AI
    // harness spend a decision on a branch that cannot happen.
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, sigarda_host_of_herons(), 0);
    let bear = bear(&mut game, 0);

    // An empty `ScriptedDecisionProvider` panics on *any* prompt, so this is
    // the assertion — the `CountingDp` control below is what proves the path
    // was reachable at all.
    let dp = ScriptedDecisionProvider::new();
    resolve_edict(&mut game, 1, 0, &dp);

    assert_eq!(
        game.get_object(bear).unwrap().zone,
        Zone::Battlefield,
        "CR 101.3 — the instruction was impossible and is ignored"
    );
}

#[test]
fn test_without_sigarda_the_same_edict_prompts_once_and_takes_a_creature() {
    // The control. Identical board minus Sigarda: one prompt, one sacrifice.
    // Without this the test above would pass just as well against a
    // `Primitive::Sacrifice` that did nothing at all.
    let mut game = setup_two_player_game();
    let bear = bear(&mut game, 0);

    let dp = CountingDp::new();
    resolve_edict(&mut game, 1, 0, &dp);

    assert_eq!(dp.count(), 1, "the victim is asked exactly once");
    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Graveyard);
}

#[test]
fn test_sigarda_does_not_protect_a_player_whose_permanents_she_is_not_on() {
    // The `AffectedSet` half. "…can't cause **you** to sacrifice permanents" is
    // `ByController(PlayerRef::You)`, resolved against Sigarda's controller —
    // so P1's own creatures are untouched by P0's Sigarda.
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, sigarda_host_of_herons(), 0);
    let their_bear = bear(&mut game, 1);

    let dp = CountingDp::new();
    resolve_edict(&mut game, 0, 1, &dp);

    assert_eq!(dp.count(), 1);
    assert_eq!(game.get_object(their_bear).unwrap().zone, Zone::Graveyard);
}

#[test]
fn test_sigarda_does_not_stop_a_spell_her_own_controller_cast() {
    // The `by` half, and the reason `SourceFilter` exists at all. "Spells and
    // abilities **your opponents** control" — your own edict aimed at yourself
    // is not one, so the prompt appears and the creature dies.
    //
    // This is the assertion that would fail if `by` were dropped and the
    // restriction were modelled as `EventPattern` + `AffectedSet` alone, which
    // is what §2.6 found the census had assumed.
    // Sigarda is the only creature, so the one candidate offered is *her* — the
    // sharpest form of the assertion, since it shows the restriction not even
    // protecting its own source from its own controller.
    let mut game = setup_two_player_game();
    let sigarda = put_on_battlefield(&mut game, sigarda_host_of_herons(), 0);

    let dp = CountingDp::new();
    resolve_edict(&mut game, 0, 0, &dp);

    assert_eq!(dp.count(), 1);
    assert_eq!(game.get_object(sigarda).unwrap().zone, Zone::Graveyard);
}

#[test]
fn test_a_sacrifice_with_no_controller_satisfies_no_source_filter() {
    // A turn-based or state-based action has no controller, so no
    // `SourceFilter` matches it — which is the right answer rather than an
    // omission: Sigarda's text names spells and abilities, and CR 704.5's
    // sacrifices are neither.
    //
    // Proposed straight through the chokepoint with a causeless `ActionContext`,
    // because nothing in the engine produces a rules-driven sacrifice yet and a
    // test that waited for one would assert nothing today.
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, sigarda_host_of_herons(), 0);
    let bear = bear(&mut game, 0);

    game.execute_action(
        GameAction::ZoneChange {
            object: bear,
            from: Zone::Battlefield,
            to: Zone::Graveyard,
            cause: ZoneChangeCause::Sacrificed,
        },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(game.get_object(bear).unwrap().zone, Zone::Graveyard);
}

// COVERS-PARTIAL: ATOM-608.2d-001
#[test]
fn test_a_player_with_no_creatures_is_asked_nothing_and_no_error_is_raised() {
    // CR 608.2d's own example — "a player who controls no creatures can't
    // choose the sacrifice option" — reached by the *same* empty candidate list
    // Sigarda produces. That is §4.9's whole claim: a restriction is a second
    // reason a candidate is unavailable, not a second mechanism.
    //
    // Partial rather than full: 608.2d's atom is the general
    // announce-choices-at-resolution rule, and this builds only the
    // impossible-option half of it.
    let mut game = setup_two_player_game();

    let dp = ScriptedDecisionProvider::new();
    resolve_edict(&mut game, 1, 0, &dp);
}

// COVERS: ATOM-701.21a-002
#[test]
fn test_you_cannot_be_made_to_sacrifice_a_permanent_you_do_not_control() {
    // CR 701.21a — "**its controller** moves it from the battlefield directly
    // to its owner's graveyard". P0 controls nothing, so P1's creature is not a
    // candidate however the edict is aimed, and the instruction is ignored.
    let mut game = setup_two_player_game();
    let their_bear = bear(&mut game, 1);

    let dp = ScriptedDecisionProvider::new();
    resolve_edict(&mut game, 1, 0, &dp);

    assert_eq!(game.get_object(their_bear).unwrap().zone, Zone::Battlefield);
}

// COVERS-PARTIAL: ATOM-701.21a-001
#[test]
fn test_sacrifice_is_not_destruction_so_indestructible_does_not_save_it() {
    // CR 701.21b — a sacrificed permanent is not destroyed, so CR 702.12b never
    // engages. The pair that proves the restriction spine reads the *cause*
    // rather than the destination: this creature is immune to the test below
    // and dies here, from the same `ZoneChange` to the same zone.
    //
    // Partial: the atom also asserts the non-permanent and not-yours halves,
    // which are `ATOM-701.21a-002/003`.
    let mut game = setup_two_player_game();
    let tank = put_on_battlefield(
        &mut game,
        vanilla_creature(2, 2, &[KeywordFlag::Indestructible]),
        0,
    );

    let dp = CountingDp::new();
    resolve_edict(&mut game, 1, 0, &dp);

    assert_eq!(dp.count(), 1);
    assert_eq!(game.get_object(tank).unwrap().zone, Zone::Graveyard);
}

#[test]
fn test_an_edict_for_two_takes_two_in_one_batch() {
    // Barter in Blood's shape. **One prompt, not two** — the player picks both
    // at once, and CR 701.21's sacrifices are simultaneous, so they go through
    // `execute_actions` as one batch rather than a loop of `change_zone`. A loop
    // would be invisible here and wrong the moment CR 704.3 or a CR 615.7
    // shield allocation has to see the batch.
    let mut game = setup_two_player_game();
    let a = bear(&mut game, 0);
    let b = bear(&mut game, 0);

    let dp = CountingDp::new();
    resolve_edict_effect(&mut game, 1, 0, &edict_for(2), &dp);

    assert_eq!(dp.count(), 1, "one choice of two, not two choices of one");
    assert_eq!(game.get_object(a).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.get_object(b).unwrap().zone, Zone::Graveyard);
}

#[test]
fn test_an_edict_for_more_than_you_have_takes_only_what_you_have() {
    // Blasphemous Edict asks for thirteen. CR 101.3: "only the possible portion
    // is performed" — so a player with two creatures loses two, and the missing
    // eleven are not an error. The same clamp is what makes an *empty* pool a
    // silent no-op, which is why Sigarda needs no separate code path.
    let mut game = setup_two_player_game();
    let a = bear(&mut game, 0);
    let b = bear(&mut game, 0);

    let dp = CountingDp::new();
    resolve_edict_effect(&mut game, 1, 0, &edict_for(13), &dp);

    assert_eq!(dp.count(), 1);
    assert_eq!(game.get_object(a).unwrap().zone, Zone::Graveyard);
    assert_eq!(game.get_object(b).unwrap().zone, Zone::Graveyard);
}

#[test]
fn test_sigarda_suppresses_a_multi_edict_entirely_rather_than_partially() {
    // The clamp and the restriction compose the right way round: the filter runs
    // *before* the count, so protecting every candidate leaves nothing to clamp
    // and the whole instruction is ignored. Clamping first and filtering second
    // would have asked for two and then sacrificed zero — the same board, but
    // reached through a prompt that CR 608.2d forbids.
    let mut game = setup_two_player_game();
    put_on_battlefield(&mut game, sigarda_host_of_herons(), 0);
    let a = bear(&mut game, 0);
    let b = bear(&mut game, 0);

    let dp = ScriptedDecisionProvider::new();
    resolve_edict_effect(&mut game, 1, 0, &edict_for(2), &dp);

    assert_eq!(game.get_object(a).unwrap().zone, Zone::Battlefield);
    assert_eq!(game.get_object(b).unwrap().zone, Zone::Battlefield);
}

// ---------------------------------------------------------------------------
// Tier 2 — the event chokepoint, and indestructible arriving through it
// ---------------------------------------------------------------------------

// COVERS: ATOM-702.12b-002
#[test]
fn test_indestructible_still_blocks_a_destroy_now_that_it_is_a_restriction() {
    // CR 702.12b through `Restriction::Event { pattern: Destroy, affected:
    // SourceOnly }`, synthesized from the keyword during the sweep — where it
    // used to be a hardcoded `match` arm inside `is_blocked`. Same answer, and
    // the point of the move is that it is now the same *mechanism* as every
    // other "can't".
    let mut game = setup_two_player_game();
    let tank = put_on_battlefield(
        &mut game,
        vanilla_creature(2, 2, &[KeywordFlag::Indestructible]),
        0,
    );
    let killer = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);

    game.execute_action(
        GameAction::Destroy { object: tank, source: DestructionSource::Effect(killer) },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(game.get_object(tank).unwrap().zone, Zone::Battlefield);
}

#[test]
fn test_losing_the_keyword_lifts_the_prohibition_within_the_same_turn() {
    // The sweep reads *effective* characteristics, so a Layer 6 removal lifts
    // the restriction with no registry to reconcile — the same property that
    // makes Humility strip a printed one below.
    let mut game = setup_two_player_game();
    let tank = put_on_battlefield(
        &mut game,
        vanilla_creature(2, 2, &[KeywordFlag::Indestructible]),
        0,
    );
    let killer = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);

    game.continuous_effects.add(registered(
        tank,
        Layer::Layer6Ability,
        1,
        EffectModification::RemoveKeywordFlag(KeywordFlag::Indestructible),
    ));

    game.execute_action(
        GameAction::Destroy { object: tank, source: DestructionSource::Effect(killer) },
        &test_ctx(),
    )
    .unwrap();

    assert_eq!(game.get_object(tank).unwrap().zone, Zone::Graveyard);
}

// COVERS-PARTIAL: ATOM-702.12b-001
#[test]
fn test_losing_indestructible_with_lethal_damage_marked_dies_to_the_sba() {
    // The *other* half of CR 702.12b, and a different road to the same
    // restriction: CR 704.5g's destruction carries
    // `DestructionSource::StateBasedAction`, where the test above carries
    // `Effect`. `EventPattern::Destroy { source: None }` matches both, which is
    // 702.12b's own words — "such permanents aren't destroyed by lethal damage,
    // **and** they ignore the state-based action that checks for lethal damage".
    //
    // `sba_indestructible_survives_lethal_damage` asserts the positive case and
    // has since Phase 7. This is its negative: nothing had shown that removing
    // the keyword lets CR 704.5g through, so "the SBA is filtered somewhere
    // else" and "the restriction answers the SBA" were indistinguishable.
    let mut game = setup_two_player_game();
    let tank = put_on_battlefield(
        &mut game,
        vanilla_creature(2, 2, &[KeywordFlag::Indestructible]),
        0,
    );
    game.battlefield.get_mut(&tank).unwrap().damage_marked = 2;

    let dp = ScriptedDecisionProvider::new();
    assert!(
        !game.check_state_based_actions(&dp).unwrap(),
        "CR 702.12b — indestructible ignores the CR 704.5g check"
    );
    assert_eq!(game.get_object(tank).unwrap().zone, Zone::Battlefield);

    game.continuous_effects.add(registered(
        tank,
        Layer::Layer6Ability,
        1,
        EffectModification::RemoveKeywordFlag(KeywordFlag::Indestructible),
    ));

    assert!(game.check_state_based_actions(&dp).unwrap());
    assert_eq!(
        game.get_object(tank).unwrap().zone,
        Zone::Graveyard,
        "the damage was already marked; only the prohibition was holding it up"
    );
}

#[test]
fn test_humility_strips_sigardas_restriction_for_free() {
    // The claim `cant-effects-architecture.md` §3.1 makes for reading the
    // *effective* ability list rather than a registry, asserted rather than
    // asserted-about: Humility removes all abilities of all creatures (CR
    // 613.1f, Layer 6), Sigarda's restriction goes with them, and nothing in
    // this phase had to know Humility exists.
    let mut game = setup_two_player_game();
    let sigarda = put_on_battlefield(&mut game, sigarda_host_of_herons(), 0);
    put_on_battlefield(&mut game, phase_lf_cards::humility(), 1);

    let dp = CountingDp::new();
    resolve_edict(&mut game, 1, 0, &dp);

    assert_eq!(dp.count(), 1, "the prompt is back — the restriction is gone");
    assert_eq!(game.get_object(sigarda).unwrap().zone, Zone::Graveyard);
}

// ---------------------------------------------------------------------------
// The registry — source 4, and the duration that is the card's rather than
// the engine's
// ---------------------------------------------------------------------------

#[test]
fn test_a_resolution_created_restriction_expires_with_its_stated_duration() {
    // What `turns.rs`'s deleted `cant_be_regenerated.clear()` used to do by
    // hand, now done by the duration the card wrote. The CR 514.2 hook is the
    // generic's, shared with both other registries — which is the whole reason
    // RS-0 existed.
    let mut game = setup_two_player_game();
    let bear = bear(&mut game, 0);

    for duration in [Duration::UntilEndOfTurn, Duration::Indefinite] {
        game.restrictions.add(RegisteredRestriction {
            id: 0,
            source: bear,
            controller: 0,
            duration,
            created_on_turn: 1,
            def: RestrictionDef::new(Restriction::ApplyReplacement {
                kind: ReplacementKindFilter::Regeneration,
                to: AffectedSet::Fixed(vec![bear]),
            }),
        });
    }

    assert_eq!(game.restrictions.len(), 2);
    game.restrictions.remove_expired_at_cleanup(0, 1);
    assert_eq!(game.restrictions.len(), 1);
    assert_eq!(game.restrictions.iter().next().unwrap().duration, Duration::Indefinite);
}

#[test]
fn test_primitive_restrict_takes_its_affected_set_from_the_resolution() {
    // A card file cannot name a target it has not yet chosen, so
    // `Primitive::Restrict` fills the affected set from the resolution the same
    // way `Primitive::Regenerate` does. Two targets, two rows, each naming its
    // own permanent — the assertion that matters, because one row naming both
    // would make a second copy of the spell a no-op under CR 614.5.
    let mut game = setup_two_player_game();
    let a = bear(&mut game, 0);
    let b = bear(&mut game, 0);
    let source = place_bare(&mut game, vanilla_creature(1, 1, &[]), 1);

    let effect = Effect::Atom(
        Primitive::Restrict(
            RestrictionDef::new(Restriction::ApplyReplacement {
                kind: ReplacementKindFilter::Regeneration,
                to: AffectedSet::Fixed(Vec::new()),
            }),
            Duration::UntilEndOfTurn,
        ),
        mtgsim::types::effects::EffectRecipient::Target(
            SelectionFilter::Creature,
            mtgsim::types::effects::TargetCount::Exactly(2),
        ),
    );
    let ctx = ResolutionContext {
        source,
        controller: 1,
        targets: vec![ResolvedTarget::Object(a), ResolvedTarget::Object(b)],
    };
    game.resolve_effect(&effect, &ctx, &ScriptedDecisionProvider::new()).unwrap();

    let sets: Vec<AffectedSet> = game
        .restrictions
        .iter()
        .map(|r| match &r.def.what {
            Restriction::ApplyReplacement { to, .. } => to.clone(),
            Restriction::Event { affected, .. } => affected.clone(),
        })
        .collect();
    assert_eq!(
        sets,
        vec![AffectedSet::Fixed(vec![a]), AffectedSet::Fixed(vec![b])],
        "one row per target, each naming only its own permanent"
    );
}

#[test]
fn test_a_restriction_written_as_a_resolving_effect_is_rejected_loudly() {
    // CR 608.2c hands scope determination to a human reader, so an
    // `Effect::Restriction` that reaches `resolve_effect` carries no duration
    // and there is nowhere honest to read one from. Loud rather than silently
    // registering an endless one — the same shape as `Effect::Replacement`'s
    // arm and for the same reason.
    let mut game = setup_two_player_game();
    let source = place_bare(&mut game, vanilla_creature(1, 1, &[]), 0);

    let effect = Effect::Restriction(Box::new(RestrictionDef::new(Restriction::Event {
        pattern: EventPattern::Destroy { source: None },
        affected: AffectedSet::Filter { filter: PermanentFilter::ByController(PlayerRef::You) },
        by: Some(SourceFilter::ControlledBy(PlayerRef::Opponent)),
    })));
    let ctx = ResolutionContext { source, controller: 0, targets: Vec::new() };

    let err = game
        .resolve_effect(&effect, &ctx, &ScriptedDecisionProvider::new())
        .unwrap_err();
    assert!(err.contains("Primitive::Restrict"), "the error names the fix: {err}");
}
