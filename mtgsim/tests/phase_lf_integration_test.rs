//! Layer 6 — ability adding and removing (CR 613.1f).
//!
//! Two channels, because `EffectiveCharacteristics` has two fields:
//! `GrantKeywordFlag`/`RemoveKeywordFlag` write `keywords`, `GrantAbility`/
//! `LoseAbility` write `abilities`, and `LoseAllAbilities` clears both. See
//! `KeywordFlag`'s docs for which CR 702 keywords belong on which channel.
//!
//! The rows here are built with `test_support::registered`, which produces an
//! `EffectOrigin::Resolution` / `Duration::UntilEndOfTurn` row over a fixed
//! target set. That is not a stand-in for anything — it is exactly what "target
//! creature loses flying until end of turn" creates (CR 613.7b). What was
//! missing before this phase was not a card; it was the `EffectModification`
//! variant, so no card *could* produce one.

use mtgsim::engine::layers::types::{ContinuousEffect, EffectModification, Layer, Timestamp};
use mtgsim::objects::card_data::AbilityDef;
use mtgsim::cards::{phase_le_cards, phase_lf_cards};
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_colors, get_effective_power, get_effective_toughness,
    has_keyword,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    card_of_type, creature_with_ability, put_in_graveyard, put_on_battlefield, registered,
    setup_two_player_game, static_ability, vanilla_creature,
};
use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::test_support::test_dp;
use mtgsim::types::effects::{
    AmountExpr, CounterType, Duration, Effect, EffectRecipient, Primitive, SelectionFilter,
    TargetCount,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::ids::{AbilityId, ObjectId};
use mtgsim::types::keywords::KeywordFlag;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn row(id: ObjectId, timestamp: Timestamp, modification: EffectModification) -> ContinuousEffect {
    registered(id, Layer::Layer6Ability, timestamp, modification)
}

/// A static ability with no effect this file ever observes. Layer 6 tests care
/// about an ability's *identity*, not what it does.
fn inert_ability() -> AbilityDef {
    static_ability(Effect::Atom(
        Primitive::GrantKeywordFlag(KeywordFlag::Vigilance, Duration::WhileSourceOnBattlefield),
        EffectRecipient::Implicit,
    ))
}

fn ability_ids(game: &GameState, id: ObjectId) -> Vec<AbilityId> {
    get_effective_abilities(game, id).iter().map(|a| a.id).collect()
}

// ---------------------------------------------------------------------------
// CR 113.10b — removing an ability removes ALL instances of it
// ---------------------------------------------------------------------------

/// The substantive half of the rule. `chars.abilities` is a `Vec`, and the same
/// ability really can appear twice — printed on the card and granted on top of
/// it, sharing an `AbilityId`. A "remove the first match" implementation passes
/// every other test in this file and fails this one.
// COVERS: ATOM-113.10b-001
#[test]
fn test_losing_an_ability_removes_every_instance_of_it() {
    let mut game = setup_two_player_game();
    let printed = inert_ability();
    let id = put_on_battlefield(
        &mut game,
        creature_with_ability("Twice-Sworn", 2, 2, printed.clone()),
        0,
    );

    // Grant the *same* ability again — same id, so the creature now has two
    // instances of one ability, which is the board CR 113.10b describes.
    game.continuous_effects.add(row(
        id,
        1,
        EffectModification::GrantAbility(Box::new(printed.clone())),
    ));
    assert_eq!(
        ability_ids(&game, id),
        vec![printed.id, printed.id],
        "printed + granted = two instances of the same ability"
    );

    // One "loses [ability]" effect, and both go.
    game.continuous_effects
        .add(row(id, 2, EffectModification::LoseAbility(printed.id)));
    assert!(
        get_effective_abilities(&game, id).is_empty(),
        "CR 113.10b: removing an ability removes all instances of it, not the first"
    );
}

/// The keyword channel reaches the same answer structurally: `keywords` is a
/// `HashSet`, so a printed flying and a granted flying were never two entries
/// to begin with, and one `RemoveKeywordFlag` clears it.
// COVERS-PARTIAL: ATOM-113.10b-001
#[test]
fn test_removing_a_keyword_clears_it_however_many_effects_granted_it() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Flying]), 0);

    game.continuous_effects
        .add(row(id, 1, EffectModification::GrantKeywordFlag(KeywordFlag::Flying)));
    assert!(has_keyword(&game, id, KeywordFlag::Flying));

    game.continuous_effects
        .add(row(id, 2, EffectModification::RemoveKeywordFlag(KeywordFlag::Flying)));
    assert!(
        !has_keyword(&game, id, KeywordFlag::Flying),
        "CR 113.10b: one removal clears the printed and the granted instance alike"
    );
}

// ---------------------------------------------------------------------------
// CR 101.2a — add/remove is NOT "can't overrides can"
// ---------------------------------------------------------------------------

/// CR 101.2 makes a "can't" effect beat a "can" effect, and CR 101.2a carves
/// ability add/remove *out* of that rule: rule 613 decides, so the later
/// timestamp prevails (CR 113.10c). Both directions are asserted, because a
/// "removal always wins" bug and an "addition always wins" bug each pass one of
/// them alone.
// COVERS: ATOM-101.2a-001
#[test]
fn test_add_after_remove_prevails_by_timestamp_not_by_cant_beating_can() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[KeywordFlag::Flying]), 0);

    // Remove at ts 1, add back at ts 2. If "can't overrides can" applied, the
    // removal would win regardless of order. It does not apply.
    game.continuous_effects
        .add(row(id, 1, EffectModification::RemoveKeywordFlag(KeywordFlag::Flying)));
    game.continuous_effects
        .add(row(id, 2, EffectModification::GrantKeywordFlag(KeywordFlag::Flying)));
    assert!(
        has_keyword(&game, id, KeywordFlag::Flying),
        "CR 101.2a/113.10c: the later effect prevails, and it adds"
    );
}

// COVERS: ATOM-101.2a-001
#[test]
fn test_remove_after_add_prevails_by_timestamp() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    game.continuous_effects
        .add(row(id, 1, EffectModification::GrantKeywordFlag(KeywordFlag::Flying)));
    game.continuous_effects
        .add(row(id, 2, EffectModification::RemoveKeywordFlag(KeywordFlag::Flying)));
    assert!(
        !has_keyword(&game, id, KeywordFlag::Flying),
        "the mirror of the previous test: timestamp decides, not the operation"
    );
}

// ---------------------------------------------------------------------------
// CR 604.3a(2) — a granted ability is never characteristic-defining
// ---------------------------------------------------------------------------

/// The flag on `AbilityDef` asserts only the criteria that are properties of the
/// ability's *text*. Provenance is criterion (2), and it is maintained by
/// whoever writes the ability onto an object — so Layer 6 must clear it, while
/// copy (Layer 1) and text-changing (Layer 3) effects must not.
#[test]
fn test_granting_an_ability_clears_its_characteristic_defining_flag() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    let mut cda = inert_ability();
    cda.is_characteristic_defining = true;

    game.continuous_effects
        .add(row(id, 1, EffectModification::GrantAbility(Box::new(cda))));

    let granted = get_effective_abilities(&game, id);
    assert_eq!(granted.len(), 1);
    assert!(
        !granted[0].is_characteristic_defining,
        "CR 604.3a(2): an ability that arrived by being granted is never a CDA, \
         however its text reads"
    );
}

// ---------------------------------------------------------------------------
// CR 122.1b — keyword counters
// ---------------------------------------------------------------------------

/// A keyword counter grants its keyword in layer 6, and is derived from
/// `BattlefieldEntity::counters` rather than registered — so it works with an
/// empty continuous-effect registry, which is what this asserts by never adding
/// a row.
// COVERS: ATOM-122.1b-001
#[test]
fn test_a_flying_counter_grants_flying() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    assert!(!has_keyword(&game, id, KeywordFlag::Flying));

    game.add_counters(id, CounterType::Flying, 1);

    assert!(
        has_keyword(&game, id, KeywordFlag::Flying),
        "CR 122.1b: a keyword counter causes the object to gain that keyword"
    );
    assert!(game.continuous_effects.is_empty(), "no registry row involved");
}

/// The counter is a layer 6 effect, not a printed keyword: removing it takes
/// the keyword away again, and a non-keyword counter grants nothing.
#[test]
fn test_keyword_counters_are_live_and_only_keyword_counters_grant() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    game.add_counters(id, CounterType::Menace, 2);
    game.add_counters(id, CounterType::PlusOnePlusOne, 1);
    assert!(has_keyword(&game, id, KeywordFlag::Menace));

    // A +1/+1 counter is layer 7c and grants no keyword — it must not have
    // fallen through the mapping into some default.
    assert!(!has_keyword(&game, id, KeywordFlag::Flying));

    game.battlefield
        .get_mut(&id)
        .unwrap()
        .remove_counters(CounterType::Menace, 2);
    assert!(
        !has_keyword(&game, id, KeywordFlag::Menace),
        "the keyword came from the counters, so it leaves with them"
    );
}

/// CR 613.1f puts keyword counters in layer 6 alongside ability-removing
/// effects, so the two meet in one layer and CR 613.7 decides by timestamp:
/// CR 613.7c timestamps each counter as it is put on, and `BattlefieldEntity`
/// stores it.
///
/// Strip *after* the counter, so the strip wins and the flying goes away.
#[test]
fn test_an_ability_strip_removes_a_flying_counters_keyword_when_it_is_later() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.add_counters(id, CounterType::Flying, 1);
    assert!(has_keyword(&game, id, KeywordFlag::Flying));

    let strip_at = game.allocate_timestamp();
    game.continuous_effects
        .add(row(id, strip_at, EffectModification::LoseAllAbilities));

    assert!(
        !has_keyword(&game, id, KeywordFlag::Flying),
        "CR 613.7: the strip has the later timestamp, so it applies after the          counter granted the keyword"
    );
}

/// The mirror, and the half that makes the first one meaningful: a counter put
/// on *after* the strip keeps its keyword, because it now has the later
/// timestamp. An implementation that always applied counters first, or always
/// applied them last, passes exactly one of this pair.
#[test]
fn test_a_flying_counter_put_on_after_a_strip_still_grants_flying() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    let strip_at = game.allocate_timestamp();
    game.continuous_effects
        .add(row(id, strip_at, EffectModification::LoseAllAbilities));

    // Allocated after the strip, so CR 613.7c gives it the later timestamp.
    game.add_counters(id, CounterType::Flying, 1);

    assert!(
        has_keyword(&game, id, KeywordFlag::Flying),
        "CR 613.7: the counter is later, so it applies after the strip"
    );
}

/// CR 613.7c's second sentence: "if that object already has a counter of that
/// kind on it, each counter of that kind receives a new timestamp identical to
/// that of the new counter." Adding a second flying counter after a strip
/// re-times *both*, so the keyword comes back.
#[test]
fn test_adding_a_counter_of_the_same_kind_retimestamps_the_whole_stack() {
    let mut game = setup_two_player_game();
    let id = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    game.add_counters(id, CounterType::Flying, 1);

    let strip_at = game.allocate_timestamp();
    game.continuous_effects
        .add(row(id, strip_at, EffectModification::LoseAllAbilities));
    assert!(!has_keyword(&game, id, KeywordFlag::Flying), "strip is later");

    game.add_counters(id, CounterType::Flying, 1);
    assert_eq!(
        game.battlefield[&id].counter_count(CounterType::Flying),
        2
    );
    assert!(
        has_keyword(&game, id, KeywordFlag::Flying),
        "CR 613.7c: the new counter re-timestamps every counter of its kind, so          the pair now applies after the strip"
    );
}

// ---------------------------------------------------------------------------
// Humility — the card the corpus names
// ---------------------------------------------------------------------------

/// CR 613.1f puts ability removal in Layer 6, and CR 613.4a puts CDA power and
/// toughness in Layer 7a — *after* it. So Humility does not race Tarmogoyf's
/// characteristic-defining ability; it deletes it one layer earlier, and 7a
/// finds nothing to apply. Humility's own 7b half then supplies the whole
/// answer.
///
/// The graveyard is stocked with five card types, so a Tarmogoyf reading its
/// CDA would be 5/6 and no assertion here could be satisfied by accident.
// COVERS: ATOM-613.1f-001
// COVERS: COMP-613-TARMOGOYF-HUMILITY-001
#[test]
fn test_humility_strips_tarmogoyfs_cda_before_layer_7a_can_read_it() {
    let mut game = setup_two_player_game();
    let goyf = put_on_battlefield(&mut game, phase_le_cards::tarmogoyf(), 0);

    for (name, card_type) in [
        ("Shock", CardType::Instant),
        ("Wastes", CardType::Land),
        ("Ancestral Vision", CardType::Sorcery),
        ("Sol Ring", CardType::Artifact),
        ("Grizzly Bears", CardType::Creature),
    ] {
        put_in_graveyard(&mut game, card_of_type(name, card_type), 0);
    }
    assert_eq!(
        (get_effective_power(&game, goyf), get_effective_toughness(&game, goyf)),
        (Some(5), Some(6)),
        "five card types in graveyards, so the CDA reads 5/6 while it exists"
    );

    put_on_battlefield(&mut game, phase_lf_cards::humility(), 0);

    assert!(
        get_effective_abilities(&game, goyf).is_empty(),
        "Layer 6: Humility removes all abilities, the CDA among them"
    );
    assert_eq!(
        (get_effective_power(&game, goyf), get_effective_toughness(&game, goyf)),
        (Some(1), Some(1)),
        "Layer 7a has no CDA left to apply, so Humility's 7b base 1/1 is the answer"
    );
}

/// The other half of the same board: Humility is not a creature, so its own
/// ability survives its own effect and keeps applying.
#[test]
fn test_humility_does_not_strip_itself() {
    let mut game = setup_two_player_game();
    let humility = put_on_battlefield(&mut game, phase_lf_cards::humility(), 0);
    let bear = put_on_battlefield(&mut game, vanilla_creature(4, 4, &[KeywordFlag::Flying]), 0);

    assert_eq!(
        get_effective_abilities(&game, humility).len(),
        1,
        "Humility is an Enchantment; its filter is creatures, so CR 613.7a finds          its ability intact at every layer"
    );
    assert!(!has_keyword(&game, bear, KeywordFlag::Flying));
    assert_eq!(
        (get_effective_power(&game, bear), get_effective_toughness(&game, bear)),
        (Some(1), Some(1))
    );
}

/// CR 113.12 — an effect that *sets a characteristic* is not an ability grant,
/// so removing the ability afterwards cannot undo the set value.
///
/// Devoid is a CDA that makes Culling Drone colorless in Layer 5. Humility
/// removes it in Layer 6, one layer *later*, so the colorlessness has already
/// been applied and stays. The printed black from the mana cost does not come
/// back.
// COVERS: ATOM-113.12-002
#[test]
fn test_humility_does_not_restore_a_devoid_cards_printed_color() {
    let mut game = setup_two_player_game();
    let drone = put_on_battlefield(&mut game, phase_le_cards::culling_drone(), 0);
    assert!(
        get_effective_colors(&game, drone).is_empty(),
        "Devoid is a Layer 5 CDA: the Drone is colorless despite its black mana cost"
    );

    put_on_battlefield(&mut game, phase_lf_cards::humility(), 0);

    assert!(
        get_effective_abilities(&game, drone).is_empty(),
        "the Devoid ability itself is gone"
    );
    assert!(
        get_effective_colors(&game, drone).is_empty(),
        "CR 113.12: Devoid set a characteristic rather than granting an ability,          and Layer 5 ran before Layer 6 removed it"
    );
}

// ---------------------------------------------------------------------------
// CR 613.7a clause 2 — the timestamp of the effect that created the ability
// ---------------------------------------------------------------------------

/// "…or the timestamp of the effect that created the ability, whichever is
/// later."
///
/// The board is built so the two clauses give *different* answers, which is the
/// only way to test a `max()`:
///
/// - The grantee entered the battlefield first, so its own timestamp is tiny.
/// - A competing Layer 7b `SetPowerToughness(9, 9)` sits at timestamp 50.
/// - A spell resolving *now* grants it a static "base power and toughness 3/3".
///
/// Under clause 1 alone the granted ability's effect would inherit the
/// creature's ancient timestamp, sort before the 9/9, and lose: the creature
/// would be 9/9. Clause 2 gives it the granting spell's timestamp instead, so it
/// sorts last within 7b and wins: 3/3.
#[test]
fn test_a_granted_static_ability_takes_the_granting_effects_timestamp() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let caster = put_on_battlefield(&mut game, phase_lf_cards::humility(), 0);

    // A 7b effect that would win on the creature's own timestamp.
    game.continuous_effects.add(registered(
        creature,
        Layer::Layer7bSetPT,
        50,
        EffectModification::SetPowerToughness {
            power: mtgsim::engine::layers::types::PtValue::Fixed(9),
            toughness: mtgsim::engine::layers::types::PtValue::Fixed(9),
        },
    ));
    // Humility is on the battlefield only to be a legal source; strip its row
    // so the P/T under test is decided by 7b alone.
    game.continuous_effects.remove_by_source(caster);
    assert_eq!(get_effective_power(&game, creature), Some(9));

    // Push the clock well past 50 so the granting effect is unambiguously later.
    while game.next_timestamp <= 60 {
        game.allocate_timestamp();
    }

    let granted = static_ability(Effect::Atom(
        Primitive::SetPowerToughness(
            AmountExpr::Fixed(3),
            AmountExpr::Fixed(3),
            Duration::WhileSourceOnBattlefield,
        ),
        EffectRecipient::Implicit,
    ));
    let spell = Effect::Atom(
        Primitive::GrantAbility(Box::new(granted), Duration::UntilEndOfTurn),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    let ctx = ResolutionContext {
        source: caster,
        controller: 0,
        targets: vec![ResolvedTarget::Object(creature)],
    };
    game.resolve_effect(&spell, &ctx, &test_dp()).unwrap();

    assert_eq!(
        (get_effective_power(&game, creature), get_effective_toughness(&game, creature)),
        (Some(3), Some(3)),
        "CR 613.7a: the granted ability's effect takes the granting spell's          timestamp, which is later than the creature's, so it applies last in 7b"
    );
}

/// The existence half, which the same rows give for free. The derived effect is
/// `EffectOrigin::StaticAbility` keyed on the granted ability's id, so stripping
/// that ability retires the effect it generated — no bookkeeping, just CR
/// 613.7a's existence check doing its job one layer later.
#[test]
fn test_stripping_a_granted_ability_retires_the_effect_it_generated() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let caster = put_on_battlefield(&mut game, phase_lf_cards::humility(), 0);
    game.continuous_effects.remove_by_source(caster);

    let granted = static_ability(Effect::Atom(
        Primitive::SetPowerToughness(
            AmountExpr::Fixed(7),
            AmountExpr::Fixed(7),
            Duration::WhileSourceOnBattlefield,
        ),
        EffectRecipient::Implicit,
    ));
    let granted_id = granted.id;
    let spell = Effect::Atom(
        Primitive::GrantAbility(Box::new(granted), Duration::UntilEndOfTurn),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    let ctx = ResolutionContext {
        source: caster,
        controller: 0,
        targets: vec![ResolvedTarget::Object(creature)],
    };
    game.resolve_effect(&spell, &ctx, &test_dp()).unwrap();
    assert_eq!(get_effective_power(&game, creature), Some(7));

    // Take the granted ability away again. The 7b row is still in the registry.
    game.continuous_effects
        .add(row(creature, 99, EffectModification::LoseAbility(granted_id)));

    assert_eq!(
        (get_effective_power(&game, creature), get_effective_toughness(&game, creature)),
        (Some(2), Some(2)),
        "CR 613.7a: registry membership is not existence — the ability is gone,          so the effect it generated no longer applies"
    );
}

/// A granted ability that is *not* static generates no continuous effect, but
/// still lands on the object as an ability.
#[test]
fn test_granting_a_non_static_ability_registers_no_derived_effect() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let caster = put_on_battlefield(&mut game, phase_lf_cards::humility(), 0);
    game.continuous_effects.remove_by_source(caster);

    let mut activated = inert_ability();
    activated.ability_type = mtgsim::objects::card_data::AbilityType::Activated;

    let spell = Effect::Atom(
        Primitive::GrantAbility(Box::new(activated), Duration::UntilEndOfTurn),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    let ctx = ResolutionContext {
        source: caster,
        controller: 0,
        targets: vec![ResolvedTarget::Object(creature)],
    };
    game.resolve_effect(&spell, &ctx, &test_dp()).unwrap();

    assert_eq!(
        get_effective_abilities(&game, creature).len(),
        1,
        "the ability is on the creature"
    );
    assert_eq!(
        game.continuous_effects.len(),
        1,
        "only the Layer 6 grant itself — an activated ability generates no          continuous effect"
    );
}

/// CR 604.3a(2) has to be applied consistently on both sides. `apply_modification`
/// clears `is_characteristic_defining` on the granted def, so the ability lands on
/// the object as an ordinary static ability — which means the effects it generates
/// have to be registered like any other static ability's.
///
/// Skipping registration because the *author* marked the def characteristic-defining
/// leaves the object holding a static ability that does nothing at all.
#[test]
fn test_a_card_authors_cda_flag_does_not_suppress_a_granted_abilitys_effect() {
    let mut game = setup_two_player_game();
    let creature = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let caster = put_on_battlefield(&mut game, phase_lf_cards::humility(), 0);
    game.continuous_effects.remove_by_source(caster);

    let mut granted = static_ability(Effect::Atom(
        Primitive::SetPowerToughness(
            AmountExpr::Fixed(6),
            AmountExpr::Fixed(6),
            Duration::WhileSourceOnBattlefield,
        ),
        EffectRecipient::Implicit,
    ));
    // The author asserts CDA-ness. CR 604.3a(2) overrules them for a *granted*
    // ability, and overruling means "treat it as an ordinary ability", not
    // "drop it".
    granted.is_characteristic_defining = true;

    let spell = Effect::Atom(
        Primitive::GrantAbility(Box::new(granted), Duration::UntilEndOfTurn),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    let ctx = ResolutionContext {
        source: caster,
        controller: 0,
        targets: vec![ResolvedTarget::Object(creature)],
    };
    game.resolve_effect(&spell, &ctx, &test_dp()).unwrap();

    let abilities = get_effective_abilities(&game, creature);
    assert_eq!(abilities.len(), 1);
    assert!(!abilities[0].is_characteristic_defining, "the flag is cleared");
    assert_eq!(
        (get_effective_power(&game, creature), get_effective_toughness(&game, creature)),
        (Some(6), Some(6)),
        "the ability is on the creature as an ordinary static ability, so the \
         effect it generates must be registered"
    );
}
