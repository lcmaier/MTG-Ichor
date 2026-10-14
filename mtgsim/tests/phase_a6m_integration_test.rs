//! A6m, the first of its two PRs (`roadmap-v2.md` A6m): the set a resolution
//! affects (`codebase-state.md` item 244) and the board's queries (item 242).

use mtgsim::engine::resolve::{ResolutionContext, ResolvedTarget};
use mtgsim::engine::targeting::ChosenTargets;
use mtgsim::events::cause::GeneratedBy;
use mtgsim::oracle::characteristics::{
    get_effective_abilities, get_effective_colors, get_effective_controller, get_effective_power,
    get_effective_toughness, has_keyword, has_type,
};
use mtgsim::state::game_state::GameState;
use mtgsim::test_support::{
    card_of_type, creature_with_ability, place_bare, put_on_battlefield, set_attacking, setup_two_player_game,
    static_ability, vanilla_creature,
};
use mtgsim::types::card_types::CardType;
use mtgsim::types::colors::Color;
use mtgsim::types::effects::{
    AmountExpr, ColorChange, Condition, Duration, Effect, EffectRecipient, ObjectFilter, ObjectSet, PlayerRef,
    PlayerSet, Primitive, SelectionFilter, TargetCount, TypeChange,
};
use mtgsim::types::ids::{ObjectId, PlayerId};
use mtgsim::types::keywords::KeywordFlag;
use mtgsim::types::replacement::EventPattern;
use mtgsim::types::restriction::{Restriction, RestrictionDef};
use mtgsim::ui::decision::ScriptedDecisionProvider;

// ---------------------------------------------------------------------------
// The boards
// ---------------------------------------------------------------------------

/// "Creatures target player controls" in a two-player game, where the target
/// is the one opponent: a filter has no leaf for a targeted player, so the
/// fixture names the opponent.
fn opponents_creatures() -> EffectRecipient {
    EffectRecipient::FilteredPermanents(ObjectFilter::And(
        Box::new(ObjectFilter::ByType(CardType::Creature)),
        Box::new(ObjectFilter::ByController(PlayerRef::Opponent)),
    ))
}

/// Resolve `effect` as an instant `controller` cast, with `targets` chosen.
fn resolve(game: &mut GameState, controller: PlayerId, effect: &Effect, targets: Vec<ResolvedTarget>) {
    let source = place_bare(game, card_of_type("A6m Instant", CardType::Instant), controller);
    let ctx = ResolutionContext {
        source,
        ability_source: None,
        controller,
        generated_by: GeneratedBy::Spell,
        targets: ChosenTargets::one(targets),
        replaced_amount: None,
        damage_prevented: None,
        trigger: None,
    };
    game.resolve_effect(effect, &ctx, &ScriptedDecisionProvider::new()).expect("resolving");
}

/// "Destroy target creature", resolved by player 0.
fn destroy(game: &mut GameState, creature: ObjectId) {
    let effect = Effect::Atom(
        Primitive::Destroy,
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    resolve(game, 0, &effect, vec![ResolvedTarget::Object(creature)]);
}

fn power_toughness(game: &GameState, id: ObjectId) -> (Option<i32>, Option<i32>) {
    (get_effective_power(game, id), get_effective_toughness(game, id))
}

// ---------------------------------------------------------------------------
// CR 611.2c — a resolution's characteristic change, fixed as it begins
// ---------------------------------------------------------------------------

/// Partial for the atom, whose filter is a color and whose late arrival
/// becomes white; here the filter is a controller and the creature comes
/// under that player's control. The claim is the same: the set is fixed as
/// the effect begins.
// COVERS-PARTIAL: ATOM-611.2c-001
#[test]
fn test_a_resolving_set_power_toughness_over_a_filter_affects_each_creature_it_names() {
    // Sudden Spoiling's 0/2. Before the fix the primitive read only its
    // targets, of which a filtered recipient has none, so nothing changed.
    let mut game = setup_two_player_game();
    let theirs = [
        put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1),
        put_on_battlefield(&mut game, vanilla_creature(4, 5, &[]), 1),
    ];
    let mine = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    let effect = Effect::Atom(
        Primitive::SetPowerToughness(AmountExpr::Fixed(0), AmountExpr::Fixed(2), Duration::UntilEndOfTurn),
        opponents_creatures(),
    );
    resolve(&mut game, 0, &effect, Vec::new());

    for id in theirs {
        assert_eq!(power_toughness(&game, id), (Some(0), Some(2)), "each creature the filter names is 0/2");
    }
    assert_eq!(power_toughness(&game, mine), (Some(2), Some(2)), "the caster's own creature is not named");

    // Sudden Spoiling's ruling: a creature the player controls later in the
    // turn is not in the set, whether it arrives or changes hands.
    let later = put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1);
    assert_eq!(power_toughness(&game, later), (Some(3), Some(3)));
    let gain_control = Effect::Atom(
        Primitive::GainControl(Duration::Indefinite),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(1)),
    );
    resolve(&mut game, 1, &gain_control, vec![ResolvedTarget::Object(mine)]);
    assert_eq!(get_effective_controller(&game, mine), Some(1));
    assert_eq!(power_toughness(&game, mine), (Some(2), Some(2)));
}

/// A characteristic-changing primitive, the creatures it is resolved over, and
/// whether a creature shows its effect.
struct Case {
    name: &'static str,
    primitive: Primitive,
    creature: fn() -> std::sync::Arc<mtgsim::objects::card_data::CardData>,
    shows: fn(&GameState, ObjectId) -> bool,
}

/// The one ability both creatures in `LoseAbility`'s case carry, by one id.
fn shared_ability() -> mtgsim::objects::card_data::AbilityDef {
    use std::sync::OnceLock;
    static ABILITY: OnceLock<mtgsim::objects::card_data::AbilityDef> = OnceLock::new();
    ABILITY
        .get_or_init(|| {
            static_ability(Effect::Atom(
                Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(0), Duration::Indefinite),
                EffectRecipient::ThisObject,
            ))
        })
        .clone()
}

/// "This creature gets +1/+1", granted.
fn granted_anthem() -> mtgsim::objects::card_data::AbilityDef {
    use std::sync::OnceLock;
    static ABILITY: OnceLock<mtgsim::objects::card_data::AbilityDef> = OnceLock::new();
    ABILITY
        .get_or_init(|| {
            static_ability(Effect::Atom(
                Primitive::ModifyPowerToughness(AmountExpr::Fixed(1), AmountExpr::Fixed(1), Duration::Indefinite),
                EffectRecipient::ThisObject,
            ))
        })
        .clone()
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "SwitchPowerToughness",
            primitive: Primitive::SwitchPowerToughness(Duration::UntilEndOfTurn),
            creature: || vanilla_creature(1, 3, &[]),
            shows: |game, id| power_toughness(game, id) == (Some(3), Some(1)),
        },
        Case {
            name: "ChangeColor",
            primitive: Primitive::ChangeColor(ColorChange::Add(Color::Blue), Duration::UntilEndOfTurn),
            creature: || vanilla_creature(2, 2, &[]),
            shows: |game, id| get_effective_colors(game, id).contains(&Color::Blue),
        },
        Case {
            name: "ChangeType",
            primitive: Primitive::ChangeType(
                TypeChange { add_types: vec![CardType::Artifact], ..TypeChange::NONE },
                Duration::UntilEndOfTurn,
            ),
            creature: || vanilla_creature(2, 2, &[]),
            shows: |game, id| has_type(game, id, CardType::Artifact),
        },
        // The grant and the rows its static ability generates on each grantee
        // (CR 613.7a), so the +1/+1 is the check.
        Case {
            name: "GrantAbility",
            primitive: Primitive::GrantAbility(std::sync::Arc::new(granted_anthem()), Duration::UntilEndOfTurn),
            creature: || vanilla_creature(2, 2, &[]),
            shows: |game, id| {
                get_effective_abilities(game, id).iter().any(|a| a.id.definition() == granted_anthem().id.definition())
                    && power_toughness(game, id) == (Some(3), Some(3))
            },
        },
        Case {
            name: "RemoveKeywordFlag",
            primitive: Primitive::RemoveKeywordFlag(KeywordFlag::Flying, Duration::UntilEndOfTurn),
            creature: || vanilla_creature(2, 2, &[KeywordFlag::Flying]),
            shows: |game, id| !has_keyword(game, id, KeywordFlag::Flying),
        },
        Case {
            name: "LoseAbility",
            primitive: Primitive::LoseAbility(shared_ability().id, Duration::UntilEndOfTurn),
            creature: || creature_with_ability("A6m Bear", 2, 2, shared_ability()),
            shows: |game, id| !get_effective_abilities(game, id).iter().any(|a| a.id == shared_ability().id),
        },
        Case {
            name: "LoseAllAbilities",
            primitive: Primitive::LoseAllAbilities(Duration::UntilEndOfTurn),
            creature: || vanilla_creature(2, 2, &[KeywordFlag::Flying]),
            shows: |game, id| !has_keyword(game, id, KeywordFlag::Flying),
        },
    ]
}

#[test]
fn test_each_characteristic_changing_primitive_takes_a_filtered_set_at_resolution() {
    // Every case is run before any is judged, so a failure names them all.
    let mut wrong = Vec::new();
    for case in cases() {
        let mut game = setup_two_player_game();
        let theirs = [
            put_on_battlefield(&mut game, (case.creature)(), 1),
            put_on_battlefield(&mut game, (case.creature)(), 1),
        ];
        let mine = put_on_battlefield(&mut game, (case.creature)(), 0);
        assert!(!(case.shows)(&game, theirs[0]), "{}: the board starts unaffected", case.name);

        resolve(&mut game, 0, &Effect::Atom(case.primitive.clone(), opponents_creatures()), Vec::new());
        let later = put_on_battlefield(&mut game, (case.creature)(), 1);

        if !theirs.iter().all(|&id| (case.shows)(&game, id)) {
            wrong.push(format!("{}: misses a creature the filter names", case.name));
        }
        if (case.shows)(&game, mine) {
            wrong.push(format!("{}: reaches a creature the filter does not name", case.name));
        }
        if (case.shows)(&game, later) {
            wrong.push(format!("{}: reaches a creature that arrived after (CR 611.2c)", case.name));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn test_gaining_control_of_a_filtered_set_takes_the_permanents_it_names_then() {
    // Mob Rule's "gain control of all creatures with power 4 or greater":
    // CR 611.2c names a control change beside a characteristic change.
    let mut game = setup_two_player_game();
    let theirs = [
        put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1),
        put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 1),
    ];
    let effect = Effect::Atom(Primitive::GainControl(Duration::UntilEndOfTurn), opponents_creatures());
    resolve(&mut game, 0, &effect, Vec::new());
    let later = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    for id in theirs {
        assert_eq!(get_effective_controller(&game, id), Some(0), "each creature the filter named");
    }
    assert_eq!(get_effective_controller(&game, later), Some(1), "fixed as the effect began");
}

// ---------------------------------------------------------------------------
// One-shot instructions over a filtered set (CR 608.2h)
// ---------------------------------------------------------------------------

#[test]
fn test_regenerate_each_creature_you_control_shields_each_one() {
    // Golgari Charm's third mode. A shield per creature, for the creatures
    // there are as it resolves.
    let mut game = setup_two_player_game();
    let mine = [
        put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0),
        put_on_battlefield(&mut game, vanilla_creature(3, 3, &[]), 0),
    ];
    let effect = Effect::Atom(
        Primitive::Regenerate,
        EffectRecipient::FilteredPermanents(ObjectFilter::And(
            Box::new(ObjectFilter::ByType(CardType::Creature)),
            Box::new(ObjectFilter::ByController(PlayerRef::You)),
        )),
    );
    resolve(&mut game, 0, &effect, Vec::new());
    let later = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);

    for id in mine {
        destroy(&mut game, id);
        let entry = game.battlefield.get(&id).expect("regenerated, so still on the battlefield");
        assert!(entry.tapped, "CR 701.19a: regeneration taps it");
    }
    destroy(&mut game, later);
    assert!(!game.battlefield.contains_key(&later), "a creature arriving after has no shield");
}

#[test]
fn test_remove_from_combat_and_remove_all_damage_take_a_filtered_set() {
    let mut game = setup_two_player_game();
    let theirs = [
        put_on_battlefield(&mut game, vanilla_creature(2, 4, &[]), 1),
        put_on_battlefield(&mut game, vanilla_creature(2, 4, &[]), 1),
    ];
    for id in theirs {
        set_attacking(&mut game, id, 0);
        game.battlefield.get_mut(&id).expect("on the battlefield").damage_marked = 2;
    }

    resolve(&mut game, 0, &Effect::Atom(Primitive::RemoveFromCombat, opponents_creatures()), Vec::new());
    resolve(&mut game, 0, &Effect::Atom(Primitive::RemoveAllDamage, opponents_creatures()), Vec::new());

    for id in theirs {
        let entry = game.battlefield.get(&id).expect("on the battlefield");
        assert!(entry.attacking.is_none(), "removed from combat");
        assert_eq!(entry.damage_marked, 0, "its damage removed");
    }
}

// ---------------------------------------------------------------------------
// CR 611.2c's second sentence — a resolution's "can't" over a filter is read
// live, so it reaches objects that were not there when it began
// ---------------------------------------------------------------------------

/// Partial for the atom, whose rules-modifying effect is a prevention; this
/// one is a "can't", and the claim is the same.
// COVERS-PARTIAL: ATOM-611.2c-002
#[test]
fn test_a_resolving_restriction_over_a_filter_reaches_creatures_that_arrive_later() {
    // Magmatic Chasm's ruling: "Creatures without flying that enter the
    // battlefield later in the turn won't be able to block." Here the
    // restriction is "can't be destroyed", over the opponent's creatures.
    let mut game = setup_two_player_game();
    let theirs = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let mine = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 0);
    let effect = Effect::Atom(
        Primitive::Restrict(
            RestrictionDef::new(Restriction::Event {
                pattern: EventPattern::Destroy { source: None },
                affected_objects: ObjectSet::NO_OBJECTS,
                affected_players: PlayerSet::Nobody,
                by: None,
            }),
            Duration::UntilEndOfTurn,
        ),
        opponents_creatures(),
    );
    resolve(&mut game, 0, &effect, Vec::new());
    let later = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);

    for id in [theirs, later] {
        destroy(&mut game, id);
        assert!(game.battlefield.contains_key(&id), "the filter names it, now as then");
    }
    destroy(&mut game, mine);
    assert!(!game.battlefield.contains_key(&mine), "the filter never named the caster's own");
}

// ---------------------------------------------------------------------------
// The prohibition gate's granted leg sees through "as long as"
// ---------------------------------------------------------------------------

#[test]
fn test_a_granted_conditional_restriction_is_the_boards_only_restriction_and_applies() {
    // "As long as this creature is untapped, it can't be destroyed", granted
    // by a resolution: no printed source, no registry row, so the gate opens
    // on the summary's granted leg or not at all.
    let cant_be_destroyed_while_untapped = static_ability(Effect::Conditional(
        Condition::SourceUntapped,
        Box::new(Effect::Restriction(Box::new(RestrictionDef::new(Restriction::Event {
            pattern: EventPattern::Destroy { source: None },
            affected_objects: ObjectSet::SourceOnly,
            affected_players: PlayerSet::Nobody,
            by: None,
        })))),
    ));
    let mut game = setup_two_player_game();
    let untapped = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let tapped = put_on_battlefield(&mut game, vanilla_creature(2, 2, &[]), 1);
    let grant = Effect::Atom(
        Primitive::GrantAbility(std::sync::Arc::new(cant_be_destroyed_while_untapped), Duration::UntilEndOfTurn),
        EffectRecipient::Target(SelectionFilter::Creature, TargetCount::Exactly(2)),
    );
    resolve(&mut game, 1, &grant, vec![ResolvedTarget::Object(untapped), ResolvedTarget::Object(tapped)]);
    game.battlefield.get_mut(&tapped).expect("on the battlefield").tapped = true;
    assert!(game.restriction_ability_sources.is_empty() && game.restrictions.is_empty());

    destroy(&mut game, untapped);
    assert!(game.battlefield.contains_key(&untapped), "the granted \"can't\" holds while its condition does");
    destroy(&mut game, tapped);
    assert!(!game.battlefield.contains_key(&tapped), "and not once it fails");
}
